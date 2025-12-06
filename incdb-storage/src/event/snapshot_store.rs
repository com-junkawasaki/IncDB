//! Snapshot Store
//!
//! 圧縮されたスナップショットの保存・読み込み

use crate::backend::traits::{Backend, BackendError};
use crate::compression::{CompressedEvents, TemporalCompression};
use async_trait::async_trait;
use incdb_core::model::{Event, IId};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;

/// スナップショットストアエラー
#[derive(Error, Debug)]
pub enum SnapshotStoreError {
    #[error("Backend error: {0}")]
    Backend(#[from] BackendError),
    #[error("Compression error: {0}")]
    Compression(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Snapshot not found")]
    NotFound,
}

/// エンティティ状態
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EntityState {
    /// エンティティID
    pub entity_id: IId,
    /// エンティティタイプ
    pub entity_type: String,
    /// 最終更新時刻
    pub last_updated: i64,
    /// 属性
    pub attributes: HashMap<String, incdb_core::model::Value>,
}

/// スナップショット
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Snapshot {
    /// スナップショットのタイムスタンプ
    pub timestamp: i64,
    /// 圧縮されたイベント
    pub compressed_events: CompressedEvents,
    /// エンティティ状態
    pub entity_states: HashMap<IId, EntityState>,
}

/// スナップショットストア
pub struct SnapshotStore {
    backend: Arc<dyn Backend>,
    /// スナップショット間隔（例: 1時間ごと）
    snapshot_interval: Duration,
    /// 圧縮エンジン
    compression: TemporalCompression,
}

impl SnapshotStore {
    /// 新しいスナップショットストアを作成
    pub fn new(
        backend: Arc<dyn Backend>,
        snapshot_interval: Duration,
        compression: TemporalCompression,
    ) -> Self {
        Self {
            backend,
            snapshot_interval,
            compression,
        }
    }

    /// スナップショットを作成
    ///
    /// # Arguments
    /// * `events` - スナップショットに含めるイベント（時系列順）
    /// * `entity_states` - エンティティ状態
    ///
    /// # Returns
    /// 作成されたスナップショット
    pub async fn create_snapshot(
        &mut self,
        events: Vec<Event>,
        entity_states: HashMap<IId, EntityState>,
    ) -> Result<Snapshot, SnapshotStoreError> {
        if events.is_empty() {
            return Err(SnapshotStoreError::Compression(
                "Cannot create snapshot with empty events".to_string(),
            ));
        }

        // イベントを圧縮
        let compressed_events = self
            .compression
            .compress_events(events)
            .map_err(|e| SnapshotStoreError::Compression(e.to_string()))?;

        // スナップショットのタイムスタンプは最後のイベントのタイムスタンプ
        let timestamp = compressed_events.base_timestamp
            + compressed_events
                .timestamp_deltas
                .iter()
                .sum::<i64>();

        let snapshot = Snapshot {
            timestamp,
            compressed_events,
            entity_states,
        };

        // スナップショットを保存
        self.save_snapshot(&snapshot).await?;

        Ok(snapshot)
    }

    /// スナップショットを保存
    async fn save_snapshot(&self, snapshot: &Snapshot) -> Result<(), SnapshotStoreError> {
        // スナップショットIDはタイムスタンプから生成
        let snapshot_id = IId(snapshot.timestamp as u64);

        // スナップショットをシリアライズ
        let serialized = bincode::serialize(snapshot)
            .map_err(|e| SnapshotStoreError::Serialization(e.to_string()))?;

        // バックエンドに保存
        self.backend
            .put_incidence(snapshot_id, &serialized)
            .await
            .map_err(SnapshotStoreError::Backend)?;

        Ok(())
    }

    /// 指定時刻以前の最新スナップショットを取得
    ///
    /// # Arguments
    /// * `timestamp` - この時刻以前の最新スナップショットを取得
    ///
    /// # Returns
    /// スナップショット（見つからない場合はNone）
    pub async fn latest_before(&self, timestamp: i64) -> Result<Option<Snapshot>, SnapshotStoreError> {
        // すべてのスナップショットIDを取得
        let all_ids = self.backend.list_incidences().await.map_err(SnapshotStoreError::Backend)?;

        // タイムスタンプ以前のスナップショットをフィルタ
        let mut candidates: Vec<(IId, i64)> = Vec::new();
        for id in all_ids {
            let id_timestamp = id.0 as i64;
            if id_timestamp <= timestamp {
                candidates.push((id, id_timestamp));
            }
        }

        // 最新のスナップショットを取得
        if let Some((snapshot_id, _)) = candidates.iter().max_by_key(|(_, ts)| ts) {
            let data = self
                .backend
                .get_incidence(*snapshot_id)
                .await
                .map_err(SnapshotStoreError::Backend)?
                .ok_or(SnapshotStoreError::NotFound)?;

            let snapshot: Snapshot = bincode::deserialize(&data)
                .map_err(|e| SnapshotStoreError::Serialization(e.to_string()))?;

            Ok(Some(snapshot))
        } else {
            Ok(None)
        }
    }

    /// スナップショットを取得
    ///
    /// # Arguments
    /// * `timestamp` - スナップショットのタイムスタンプ
    ///
    /// # Returns
    /// スナップショット（見つからない場合はNone）
    pub async fn get_snapshot(&self, timestamp: i64) -> Result<Option<Snapshot>, SnapshotStoreError> {
        let snapshot_id = IId(timestamp as u64);
        let data = self
            .backend
            .get_incidence(snapshot_id)
            .await
            .map_err(SnapshotStoreError::Backend)?;

        if let Some(data) = data {
            let snapshot: Snapshot = bincode::deserialize(&data)
                .map_err(|e| SnapshotStoreError::Serialization(e.to_string()))?;
            Ok(Some(snapshot))
        } else {
            Ok(None)
        }
    }

    /// スナップショットからイベントを復元
    ///
    /// # Arguments
    /// * `snapshot` - スナップショット
    ///
    /// # Returns
    /// 復元されたイベント
    pub fn restore_events(&self, snapshot: &Snapshot) -> Result<Vec<Event>, SnapshotStoreError> {
        self.compression
            .decompress_events(snapshot.compressed_events.clone())
            .map_err(|e| SnapshotStoreError::Compression(e.to_string()))
    }

    /// スナップショット間隔を取得
    pub fn snapshot_interval(&self) -> Duration {
        self.snapshot_interval
    }

    /// 次のスナップショット時刻を計算
    ///
    /// # Arguments
    /// * `current_timestamp` - 現在のタイムスタンプ
    ///
    /// # Returns
    /// 次のスナップショット時刻
    pub fn next_snapshot_time(&self, current_timestamp: i64) -> i64 {
        let interval_ms = self.snapshot_interval.as_millis() as i64;
        let snapshots_passed = current_timestamp / interval_ms;
        (snapshots_passed + 1) * interval_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::sled_backend::SledBackend;
    use crate::compression::ProductQuantization;
    use incdb_core::model::{Event, EventType, Value};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_snapshot_store() {
        let temp_dir = TempDir::new().unwrap();
        let backend = Arc::new(SledBackend::new(temp_dir.path()).unwrap());
        let pq = ProductQuantization::new(128, 8, 256).ok();
        let compression = TemporalCompression::new(true, pq, false);
        let mut store = SnapshotStore::new(
            backend,
            Duration::from_secs(3600), // 1時間
            compression,
        );

        let mut events = Vec::new();
        for i in 0..10 {
            let event = Event::new(
                IId(i as u64 + 1),
                1000 + i * 100,
                EventType::PersonCreated,
                IId(1),
                Value::string(format!("Event {}", i)),
            );
            events.push(event);
        }

        let entity_states = HashMap::new();
        let snapshot = store.create_snapshot(events, entity_states).await.unwrap();

        // スナップショットを取得
        let retrieved = store.get_snapshot(snapshot.timestamp).await.unwrap();
        assert!(retrieved.is_some());

        // イベントを復元
        let restored = store.restore_events(&retrieved.unwrap()).unwrap();
        assert_eq!(restored.len(), 10);
    }

    #[tokio::test]
    async fn test_latest_before() {
        let temp_dir = TempDir::new().unwrap();
        let backend = Arc::new(SledBackend::new(temp_dir.path()).unwrap());
        let compression = TemporalCompression::new(true, None, false);
        let mut store = SnapshotStore::new(backend, Duration::from_secs(3600), compression);

        // 複数のスナップショットを作成
        for i in 0..5 {
            let mut events = Vec::new();
            let event = Event::new(
                IId(i as u64 + 1),
                1000 + i * 1000,
                EventType::PersonCreated,
                IId(1),
                Value::string(format!("Snapshot {}", i)),
            );
            events.push(event);
            let entity_states = HashMap::new();
            store.create_snapshot(events, entity_states).await.unwrap();
        }

        // 2500以前の最新スナップショットを取得
        let snapshot = store.latest_before(2500).await.unwrap();
        assert!(snapshot.is_some());
        assert_eq!(snapshot.unwrap().timestamp, 2000);
    }
}

