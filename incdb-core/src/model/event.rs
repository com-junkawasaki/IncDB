//! Event Model
//!
//! イベントソーシングのためのイベントモデル

use crate::model::{IId, Incidence, RoleId, Value};
use serde::{Deserialize, Serialize};

/// イベントタイプ
///
/// 暗号資産犯罪捜査に関連するイベントタイプ
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    /// 人物が作成された
    PersonCreated,
    /// 人物が移動した
    PersonMoved,
    /// IP接続が発生した
    IPConnection,
    /// トランザクションが発生した
    TransactionOccurred,
    /// 場所を訪問した
    LocationVisited,
    /// アドレスが作成された
    AddressCreated,
    /// アドレス間の転送
    AddressTransfer,
    /// その他のイベント
    Other(String),
}

impl EventType {
    /// イベントタイプを文字列に変換
    pub fn as_str(&self) -> &str {
        match self {
            EventType::PersonCreated => "PersonCreated",
            EventType::PersonMoved => "PersonMoved",
            EventType::IPConnection => "IPConnection",
            EventType::TransactionOccurred => "TransactionOccurred",
            EventType::LocationVisited => "LocationVisited",
            EventType::AddressCreated => "AddressCreated",
            EventType::AddressTransfer => "AddressTransfer",
            EventType::Other(s) => s.as_str(),
        }
    }
}

/// イベント構造
///
/// 時系列順に保存されるイベント
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Event {
    /// イベントID（Incidence IDと同じ）
    pub id: IId,
    /// Unix timestamp (milliseconds)
    pub timestamp: i64,
    /// イベントタイプ
    pub event_type: EventType,
    /// 関連エンティティ（人物、IPアドレスなど）
    pub entity_id: IId,
    /// イベントデータ
    pub payload: Value,
    /// ベクトル埋め込み（オプション）
    pub embedding: Option<Vec<f32>>,
    /// 関連エンティティ（args + roles）
    pub related_entities: Vec<(IId, RoleId)>,
}

impl Event {
    /// 新しいイベントを作成
    pub fn new(
        id: IId,
        timestamp: i64,
        event_type: EventType,
        entity_id: IId,
        payload: Value,
    ) -> Self {
        Self {
            id,
            timestamp,
            event_type,
            entity_id,
            payload,
            embedding: None,
            related_entities: Vec::new(),
        }
    }

    /// ベクトル埋め込みを設定
    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }

    /// 関連エンティティを追加
    pub fn add_related_entity(mut self, entity_id: IId, role: RoleId) -> Self {
        self.related_entities.push((entity_id, role));
        self
    }

    /// 現在時刻でイベントを作成
    pub fn now(
        id: IId,
        event_type: EventType,
        entity_id: IId,
        payload: Value,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        Self::new(id, timestamp, event_type, entity_id, payload)
    }
}

/// IncidenceからEventへの変換
impl From<Incidence> for Event {
    fn from(inc: Incidence) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        // Incidenceの値からイベントタイプを推測
        let event_type = if let Some(ref val) = inc.val {
            match val {
                Value::Str(s) => {
                    if s.contains("Person") {
                        EventType::PersonCreated
                    } else if s.contains("Address") {
                        EventType::AddressCreated
                    } else if s.contains("Transaction") {
                        EventType::TransactionOccurred
                    } else {
                        EventType::Other(s.clone())
                    }
                }
                _ => EventType::Other("Unknown".to_string()),
            }
        } else {
            EventType::Other("Incidence".to_string())
        };

        // エンティティIDは最初のarg、またはid自体
        let entity_id = inc.args.first().copied().unwrap_or(inc.id);

        // ペイロードは値、またはデフォルト値
        let payload = inc.val.clone().unwrap_or(Value::Null);

        let mut event = Event::new(
            inc.id,
            timestamp,
            event_type,
            entity_id,
            payload,
        );

        // ベクトル埋め込みをコピー
        if let Some(embedding) = &inc.embedding {
            event = event.with_embedding(embedding.clone());
        }

        // argsとrolesをrelated_entitiesに変換
        for (arg_id, role_id) in inc.args.iter().zip(inc.roles.iter()) {
            event = event.add_related_entity(*arg_id, *role_id);
        }

        event
    }
}

/// EventからIncidenceへの変換
impl From<Event> for Incidence {
    fn from(event: Event) -> Self {
        use crate::model::Level;

        let mut incidence = Incidence::new(event.id, Level::zero())
            .with_val(event.payload);

        // ベクトル埋め込みをコピー
        if let Some(embedding) = event.embedding {
            incidence = incidence.with_embedding(embedding);
        }

        // related_entitiesをargsとrolesに変換
        for (entity_id, role_id) in event.related_entities {
            incidence = incidence.add_arg(entity_id, role_id);
        }

        // entity_idを最初のargとして追加（まだ追加されていない場合）
        if !incidence.args.contains(&event.entity_id) {
            incidence = incidence.add_arg(event.entity_id, RoleId(0));
        }

        incidence
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Level;

    #[test]
    fn test_event_creation() {
        let id = IId(1);
        let entity_id = IId(2);
        let timestamp = 1234567890000i64;
        let event = Event::new(
            id,
            timestamp,
            EventType::PersonCreated,
            entity_id,
            Value::string("John Doe"),
        );

        assert_eq!(event.id, id);
        assert_eq!(event.timestamp, timestamp);
        assert_eq!(event.event_type, EventType::PersonCreated);
        assert_eq!(event.entity_id, entity_id);
    }

    #[test]
    fn test_incidence_to_event() {
        let id = IId(1);
        let entity_id = IId(2);
        let incidence = Incidence::new(id, Level::zero())
            .with_val(Value::string("Person"))
            .add_arg(entity_id, RoleId(1));

        let event: Event = incidence.clone().into();

        assert_eq!(event.id, id);
        assert_eq!(event.entity_id, entity_id);
        assert_eq!(event.related_entities.len(), 1);
    }

    #[test]
    fn test_event_to_incidence() {
        let id = IId(1);
        let entity_id = IId(2);
        let event = Event::new(
            id,
            1234567890000i64,
            EventType::PersonCreated,
            entity_id,
            Value::string("John Doe"),
        )
        .add_related_entity(entity_id, RoleId(1));

        let incidence: Incidence = event.clone().into();

        assert_eq!(incidence.id, id);
        assert_eq!(incidence.val, Some(Value::string("John Doe")));
        // related_entitiesにentity_idが含まれている場合、argsには1つだけ追加される
        // entity_idがrelated_entitiesに含まれていない場合のみ、argsに追加される
        assert_eq!(incidence.args.len(), 1); // related_entityのみ（entity_idは既に含まれているため追加されない）
    }

    #[test]
    fn test_event_serialization_roundtrip() {
        // Eventを作成（related_entitiesを含む）
        let event = Event::new(
            IId(1),
            1234567890000i64,
            EventType::PersonCreated,
            IId(2),
            Value::string("John Doe"),
        )
        .add_related_entity(IId(3), RoleId(1))
        .add_related_entity(IId(4), RoleId(2));

        // シリアライズ
        let serialized = bincode::serialize(&event).expect("Failed to serialize");
        
        // デシリアライズ
        let deserialized: Event = bincode::deserialize(&serialized).expect("Failed to deserialize");

        // 検証
        assert_eq!(event.id, deserialized.id);
        assert_eq!(event.timestamp, deserialized.timestamp);
        assert_eq!(event.entity_id, deserialized.entity_id);
        assert_eq!(event.related_entities.len(), deserialized.related_entities.len());
        assert_eq!(event.related_entities, deserialized.related_entities);
    }

    #[test]
    fn test_incidence_to_event_to_incidence_roundtrip() {
        // Incidenceを作成（argsとrolesを含む）
        let original_incidence = Incidence::new(IId(1), Level::zero())
            .with_val(Value::string("Test"))
            .add_arg(IId(2), RoleId(1))
            .add_arg(IId(3), RoleId(2));

        // Incidence -> Event
        let event: Event = original_incidence.clone().into();
        
        // Eventをシリアライズ
        let serialized = bincode::serialize(&event).expect("Failed to serialize");
        
        // デシリアライズ
        let deserialized_event: Event = bincode::deserialize(&serialized).expect("Failed to deserialize");
        
        // Event -> Incidence
        let restored_incidence: Incidence = deserialized_event.into();

        // 検証
        assert_eq!(original_incidence.id, restored_incidence.id);
        assert_eq!(original_incidence.args.len(), restored_incidence.args.len());
        assert_eq!(original_incidence.roles.len(), restored_incidence.roles.len());
        assert_eq!(original_incidence.args, restored_incidence.args);
        assert_eq!(original_incidence.roles, restored_incidence.roles);
    }

    #[test]
    fn test_event_with_empty_related_entities() {
        // related_entitiesが空のEvent
        let event = Event::new(
            IId(1),
            1234567890000i64,
            EventType::PersonCreated,
            IId(2),
            Value::string("John Doe"),
        );

        // シリアライズ/デシリアライズ
        let serialized = bincode::serialize(&event).expect("Failed to serialize");
        let deserialized: Event = bincode::deserialize(&serialized).expect("Failed to deserialize");

        // 検証
        assert_eq!(event.related_entities.len(), deserialized.related_entities.len());
        assert!(deserialized.related_entities.is_empty());
    }

    #[test]
    fn test_event_with_multiple_related_entities() {
        // 複数のrelated_entitiesを持つEvent
        let event = Event::new(
            IId(1),
            1234567890000i64,
            EventType::PersonCreated,
            IId(2),
            Value::string("John Doe"),
        )
        .add_related_entity(IId(3), RoleId(1))
        .add_related_entity(IId(4), RoleId(2))
        .add_related_entity(IId(5), RoleId(3));

        // シリアライズ/デシリアライズ
        let serialized = bincode::serialize(&event).expect("Failed to serialize");
        let deserialized: Event = bincode::deserialize(&serialized).expect("Failed to deserialize");

        // 検証
        assert_eq!(event.related_entities.len(), 3);
        assert_eq!(deserialized.related_entities.len(), 3);
        assert_eq!(event.related_entities, deserialized.related_entities);
    }
}

