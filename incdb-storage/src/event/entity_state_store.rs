//! Entity State Store
//!
//! エンティティの現在状態管理

use crate::backend::traits::{Backend, BackendError};
use async_trait::async_trait;
use incdb_core::model::{Event, EventType, IId, Value};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// エンティティタイプ
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum EntityType {
    /// 人物
    Person,
    /// IPアドレス
    IPAddress,
    /// 場所
    Location,
    /// トランザクション
    Transaction,
    /// 暗号資産アドレス
    Address,
    /// その他
    Other(String),
}

impl EntityType {
    /// イベントタイプからエンティティタイプを推測
    pub fn from_event_type(event_type: &EventType) -> Self {
        match event_type {
            EventType::PersonCreated | EventType::PersonMoved => EntityType::Person,
            EventType::IPConnection => EntityType::IPAddress,
            EventType::LocationVisited => EntityType::Location,
            EventType::TransactionOccurred => EntityType::Transaction,
            EventType::AddressCreated | EventType::AddressTransfer => EntityType::Address,
            EventType::Other(s) => EntityType::Other(s.clone()),
        }
    }
}

/// エンティティ状態
#[derive(Clone, Debug)]
pub struct EntityState {
    /// エンティティID
    pub entity_id: IId,
    /// エンティティタイプ
    pub entity_type: EntityType,
    /// 最終更新時刻
    pub last_updated: i64,
    /// 属性
    pub attributes: HashMap<String, Value>,
    /// 現在の位置（オプション）
    pub current_location: Option<IId>,
    /// 現在のIPアドレス（オプション、文字列として保存）
    pub current_ip: Option<String>,
    /// 関連エンティティ
    pub related_entities: Vec<(IId, incdb_core::model::RoleId)>,
}

impl EntityState {
    /// 新しいエンティティ状態を作成
    pub fn new(entity_id: IId, entity_type: EntityType, timestamp: i64) -> Self {
        Self {
            entity_id,
            entity_type,
            last_updated: timestamp,
            attributes: HashMap::new(),
            current_location: None,
            current_ip: None,
            related_entities: Vec::new(),
        }
    }

    /// 属性を設定
    pub fn with_attribute(mut self, key: String, value: Value) -> Self {
        self.attributes.insert(key, value);
        self
    }

    /// 現在の位置を設定
    pub fn with_location(mut self, location_id: IId) -> Self {
        self.current_location = Some(location_id);
        self
    }

    /// 現在のIPアドレスを設定
    pub fn with_ip(mut self, ip: String) -> Self {
        self.current_ip = Some(ip);
        self
    }

    /// 関連エンティティを追加
    pub fn add_related_entity(
        mut self,
        entity_id: IId,
        role: incdb_core::model::RoleId,
    ) -> Self {
        self.related_entities.push((entity_id, role));
        self
    }
}

/// エンティティ状態ストアエラー
#[derive(Error, Debug)]
pub enum EntityStateStoreError {
    #[error("Backend error: {0}")]
    Backend(#[from] BackendError),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Entity not found: {0}")]
    NotFound(IId),
}

/// エンティティ状態ストア
pub struct EntityStateStore {
    /// エンティティID → 最新状態（メモリキャッシュ）
    states: HashMap<IId, EntityState>,
    backend: Arc<dyn Backend>,
}

impl EntityStateStore {
    /// 新しいエンティティ状態ストアを作成
    pub fn new(backend: Arc<dyn Backend>) -> Self {
        Self {
            states: HashMap::new(),
            backend,
        }
    }

    /// イベントからエンティティ状態を更新
    ///
    /// # Arguments
    /// * `entity_id` - エンティティID
    /// * `event` - イベント
    ///
    /// # Returns
    /// 更新後のエンティティ状態
    pub async fn update(&mut self, entity_id: IId, event: &Event) -> Result<EntityState, EntityStateStoreError> {
        // 既存の状態を取得、または新規作成
        let mut state = self
            .states
            .get(&entity_id)
            .cloned()
            .unwrap_or_else(|| {
                EntityState::new(
                    entity_id,
                    EntityType::from_event_type(&event.event_type),
                    event.timestamp,
                )
            });

        // イベントタイプに応じて状態を更新
        match event.event_type {
            EventType::PersonCreated => {
                // 人物が作成された
                if let Value::Str(name) = &event.payload {
                    state = state.with_attribute("name".to_string(), event.payload.clone());
                }
            }
            EventType::PersonMoved => {
                // 人物が移動した
                if let Some(location_id) = event.related_entities.first() {
                    state = state.with_location(location_id.0);
                }
            }
            EventType::IPConnection => {
                // IP接続が発生した
                if let Value::Str(ip) = &event.payload {
                    state = state.with_ip(ip.clone());
                }
            }
            EventType::LocationVisited => {
                // 場所を訪問した
                if let Some(location_id) = event.related_entities.first() {
                    state = state.with_location(location_id.0);
                }
            }
            EventType::TransactionOccurred => {
                // トランザクションが発生した
                state = state.with_attribute("last_transaction".to_string(), event.payload.clone());
            }
            EventType::AddressCreated => {
                // アドレスが作成された
                state = state.with_attribute("address".to_string(), event.payload.clone());
            }
            EventType::AddressTransfer => {
                // アドレス間の転送
                state = state.with_attribute("last_transfer".to_string(), event.payload.clone());
            }
            EventType::Other(_) => {
                // その他のイベント
                state = state.with_attribute("last_event".to_string(), event.payload.clone());
            }
        }

        // 関連エンティティを更新
        for (related_id, role) in &event.related_entities {
            state = state.add_related_entity(*related_id, *role);
        }

        // 最終更新時刻を更新
        state.last_updated = event.timestamp;

        // メモリキャッシュを更新
        self.states.insert(entity_id, state.clone());

        // バックエンドに保存
        self.save_entity_state(&state).await?;

        Ok(state)
    }

    /// エンティティ状態を取得
    ///
    /// # Arguments
    /// * `entity_id` - エンティティID
    ///
    /// # Returns
    /// エンティティ状態（見つからない場合はNone）
    pub async fn get(&self, entity_id: IId) -> Result<Option<EntityState>, EntityStateStoreError> {
        // メモリキャッシュから取得
        if let Some(state) = self.states.get(&entity_id) {
            return Ok(Some(state.clone()));
        }

        // バックエンドから取得
        let data = self
            .backend
            .get_incidence(entity_id)
            .await
            .map_err(EntityStateStoreError::Backend)?;

        if let Some(data) = data {
            let state: EntityState = bincode::deserialize(&data)
                .map_err(|e| EntityStateStoreError::Serialization(e.to_string()))?;
            Ok(Some(state))
        } else {
            Ok(None)
        }
    }

    /// エンティティ状態を保存
    async fn save_entity_state(&self, state: &EntityState) -> Result<(), EntityStateStoreError> {
        let serialized = bincode::serialize(state)
            .map_err(|e| EntityStateStoreError::Serialization(e.to_string()))?;

        self.backend
            .put_incidence(state.entity_id, &serialized)
            .await
            .map_err(EntityStateStoreError::Backend)?;

        Ok(())
    }

    /// すべてのエンティティ状態を取得
    pub fn all_states(&self) -> Vec<EntityState> {
        self.states.values().cloned().collect()
    }

    /// エンティティタイプでフィルタ
    pub fn filter_by_type(&self, entity_type: &EntityType) -> Vec<EntityState> {
        self.states
            .values()
            .filter(|state| &state.entity_type == entity_type)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::sled_backend::SledBackend;
    use incdb_core::model::{EventType, RoleId};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_entity_state_store() {
        let temp_dir = TempDir::new().unwrap();
        let backend = Arc::new(SledBackend::new(temp_dir.path()).unwrap());
        let mut store = EntityStateStore::new(backend);

        let entity_id = IId(1);
        let event = Event::new(
            IId(1),
            1000i64,
            EventType::PersonCreated,
            entity_id,
            Value::string("John Doe"),
        );

        let state = store.update(entity_id, &event).await.unwrap();
        assert_eq!(state.entity_id, entity_id);
        assert_eq!(state.entity_type, EntityType::Person);
        assert_eq!(state.attributes.get("name"), Some(&Value::string("John Doe")));

        // 状態を取得
        let retrieved = store.get(entity_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().entity_id, entity_id);
    }

    #[tokio::test]
    async fn test_entity_state_update() {
        let temp_dir = TempDir::new().unwrap();
        let backend = Arc::new(SledBackend::new(temp_dir.path()).unwrap());
        let mut store = EntityStateStore::new(backend);

        let entity_id = IId(1);
        let location_id = IId(2);

        // 人物を作成
        let create_event = Event::new(
            IId(1),
            1000i64,
            EventType::PersonCreated,
            entity_id,
            Value::string("John Doe"),
        );
        store.update(entity_id, &create_event).await.unwrap();

        // 人物が移動
        let move_event = Event::new(
            IId(2),
            2000i64,
            EventType::PersonMoved,
            entity_id,
            Value::string("Moved"),
        )
        .add_related_entity(location_id, RoleId(1));

        let state = store.update(entity_id, &move_event).await.unwrap();
        assert_eq!(state.current_location, Some(location_id));
        assert_eq!(state.last_updated, 2000);
    }
}

