//! Event Stream
//!
//! 時系列順のイベント保存・取得

use crate::backend::traits::{Backend, BackendError};
use async_trait::async_trait;
use incdb_core::model::{Event, IId};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use thiserror::Error;

/// イベントストリームエラー
#[derive(Error, Debug)]
pub enum EventStreamError {
    #[error("Backend error: {0}")]
    Backend(#[from] BackendError),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Event not found: {0}")]
    NotFound(IId),
}

/// イベントストリーム（時系列順）
///
/// イベントを時系列順に保存し、時系列範囲クエリやエンティティベースのクエリを提供
pub struct EventStream {
    backend: Arc<dyn Backend>,
    /// 時系列インデックス（タイムスタンプ → イベントID）
    temporal_index: BTreeMap<i64, Vec<IId>>,
    /// エンティティインデックス（エンティティID → イベントID）
    entity_index: HashMap<IId, Vec<IId>>,
}

impl EventStream {
    /// 新しいイベントストリームを作成
    pub fn new(backend: Arc<dyn Backend>) -> Self {
        Self {
            backend,
            temporal_index: BTreeMap::new(),
            entity_index: HashMap::new(),
        }
    }

    /// イベントを追加（時系列順）
    pub async fn append(&mut self, event: Event) -> Result<(), EventStreamError> {
        // イベントをシリアライズ
        let serialized = bincode::serialize(&event)
            .map_err(|e| EventStreamError::Serialization(e.to_string()))?;

        // バックエンドに保存
        self.backend
            .put_incidence(event.id, &serialized)
            .await
            .map_err(EventStreamError::Backend)?;

        // 時系列インデックスを更新
        self.temporal_index
            .entry(event.timestamp)
            .or_insert_with(Vec::new)
            .push(event.id);

        // エンティティインデックスを更新
        self.entity_index
            .entry(event.entity_id)
            .or_insert_with(Vec::new)
            .push(event.id);

        // 関連エンティティのインデックスも更新
        for (related_id, _role) in &event.related_entities {
            self.entity_index
                .entry(*related_id)
                .or_insert_with(Vec::new)
                .push(event.id);
        }

        Ok(())
    }

    /// イベントを取得
    pub async fn get(&self, id: IId) -> Result<Event, EventStreamError> {
        let data = self
            .backend
            .get_incidence(id)
            .await
            .map_err(EventStreamError::Backend)?
            .ok_or(EventStreamError::NotFound(id))?;

        let event: Event = bincode::deserialize(&data)
            .map_err(|e| EventStreamError::Serialization(e.to_string()))?;

        Ok(event)
    }

    /// 時系列範囲クエリ
    ///
    /// startからendまでのイベントを取得
    pub async fn range(&self, start: i64, end: i64) -> Result<Vec<Event>, EventStreamError> {
        let mut events = Vec::new();

        // 時系列インデックスから該当するタイムスタンプのイベントIDを取得
        for (timestamp, event_ids) in self.temporal_index.range(start..=end) {
            for event_id in event_ids {
                let event = self.get(*event_id).await?;
                events.push(event);
            }
        }

        // タイムスタンプ順にソート（念のため）
        events.sort_by_key(|e| e.timestamp);

        Ok(events)
    }

    /// エンティティのイベントを取得
    ///
    /// 指定されたエンティティに関連するすべてのイベントを取得
    pub async fn events_for_entity(&self, entity_id: IId) -> Result<Vec<Event>, EventStreamError> {
        let event_ids = self
            .entity_index
            .get(&entity_id)
            .cloned()
            .unwrap_or_default();

        let mut events = Vec::new();
        for event_id in event_ids {
            let event = self.get(event_id).await?;
            events.push(event);
        }

        // タイムスタンプ順にソート
        events.sort_by_key(|e| e.timestamp);

        Ok(events)
    }

    /// 最新のN件のイベントを取得
    pub async fn latest(&self, n: usize) -> Result<Vec<Event>, EventStreamError> {
        let mut events = Vec::new();

        // 時系列インデックスを逆順に走査
        for (_, event_ids) in self.temporal_index.iter().rev() {
            for event_id in event_ids.iter().rev() {
                if events.len() >= n {
                    break;
                }
                let event = self.get(*event_id).await?;
                events.push(event);
            }
            if events.len() >= n {
                break;
            }
        }

        // タイムスタンプ順にソート（最新順）
        events.sort_by_key(|e| std::cmp::Reverse(e.timestamp));

        Ok(events)
    }

    /// イベントストリームのサイズを取得
    pub fn len(&self) -> usize {
        self.temporal_index.values().map(|v| v.len()).sum()
    }

    /// イベントストリームが空かどうか
    pub fn is_empty(&self) -> bool {
        self.temporal_index.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::sled_backend::SledBackend;
    use incdb_core::model::{Event, EventType, Value};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_event_stream_append() {
        let temp_dir = TempDir::new().unwrap();
        let backend = Arc::new(SledBackend::new(temp_dir.path()).unwrap());
        let mut stream = EventStream::new(backend);

        let event = Event::new(
            IId(1),
            1234567890000i64,
            EventType::PersonCreated,
            IId(2),
            Value::string("John Doe"),
        );

        stream.append(event.clone()).await.unwrap();
        assert_eq!(stream.len(), 1);
    }

    #[tokio::test]
    async fn test_event_stream_range() {
        let temp_dir = TempDir::new().unwrap();
        let backend = Arc::new(SledBackend::new(temp_dir.path()).unwrap());
        let mut stream = EventStream::new(backend);

        let event1 = Event::new(
            IId(1),
            1000i64,
            EventType::PersonCreated,
            IId(2),
            Value::string("John"),
        );
        let event2 = Event::new(
            IId(2),
            2000i64,
            EventType::PersonMoved,
            IId(2),
            Value::string("Jane"),
        );
        let event3 = Event::new(
            IId(3),
            3000i64,
            EventType::IPConnection,
            IId(3),
            Value::string("192.168.1.1"),
        );

        stream.append(event1).await.unwrap();
        stream.append(event2).await.unwrap();
        stream.append(event3).await.unwrap();

        let events = stream.range(1500, 2500).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, IId(2));
    }

    #[tokio::test]
    async fn test_events_for_entity() {
        let temp_dir = TempDir::new().unwrap();
        let backend = Arc::new(SledBackend::new(temp_dir.path()).unwrap());
        let mut stream = EventStream::new(backend);

        let entity_id = IId(2);
        let event1 = Event::new(
            IId(1),
            1000i64,
            EventType::PersonCreated,
            entity_id,
            Value::string("John"),
        );
        let event2 = Event::new(
            IId(2),
            2000i64,
            EventType::PersonMoved,
            entity_id,
            Value::string("Jane"),
        );

        stream.append(event1).await.unwrap();
        stream.append(event2).await.unwrap();

        let events = stream.events_for_entity(entity_id).await.unwrap();
        assert_eq!(events.len(), 2);
    }
}

