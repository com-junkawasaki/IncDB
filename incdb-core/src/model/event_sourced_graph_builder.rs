//! Event Sourced Graph Builder
//!
//! EventSourcedGraphを簡単に作成するためのビルダー

use crate::model::EventSourcedGraph;
use incdb_storage::backend::sled_backend::SledBackend;
use incdb_storage::compression::{ProductQuantization, TemporalCompression};
use incdb_storage::event::{EntityStateStore, EventStream, SnapshotStore};
use incdb_storage::index::optimized_vector_index::OptimizedVectorIndex;
use incdb_storage::index::vector_index::SimpleVectorIndex;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

/// EventSourcedGraphビルダー
pub struct EventSourcedGraphBuilder {
    storage_path: Option<String>,
    vector_dim: usize,
    snapshot_interval: Duration,
}

impl EventSourcedGraphBuilder {
    /// 新しいビルダーを作成
    pub fn new() -> Self {
        Self {
            storage_path: None,
            vector_dim: 128,
            snapshot_interval: Duration::from_secs(3600), // 1時間
        }
    }

    /// ストレージパスを設定
    pub fn with_storage_path(mut self, path: impl Into<String>) -> Self {
        self.storage_path = Some(path.into());
        self
    }

    /// ベクトル次元数を設定
    pub fn with_vector_dim(mut self, dim: usize) -> Self {
        self.vector_dim = dim;
        self
    }

    /// スナップショット間隔を設定
    pub fn with_snapshot_interval(mut self, interval: Duration) -> Self {
        self.snapshot_interval = interval;
        self
    }

    /// EventSourcedGraphを構築
    pub async fn build(self) -> Result<Arc<tokio::sync::Mutex<EventSourcedGraph>>, Box<dyn std::error::Error>> {
        // ストレージパスを決定
        let storage_path = self.storage_path.unwrap_or_else(|| {
            std::env::var("INCDB_STORAGE_PATH")
                .unwrap_or_else(|_| "./data".to_string())
        });

        // ストレージディレクトリを作成
        std::fs::create_dir_all(&storage_path)?;

        // バックエンドを作成
        let backend = Arc::new(SledBackend::new(&storage_path)?);

        // イベントストリームを作成
        let event_stream = Arc::new(std::sync::Mutex::new(EventStream::new(backend.clone())));

        // 圧縮エンジンを作成
        let pq = ProductQuantization::new(self.vector_dim, 8, 256).ok();
        let compression = TemporalCompression::new(true, pq, false);

        // スナップショットストアを作成
        let snapshot_store = Arc::new(std::sync::Mutex::new(SnapshotStore::new(
            backend.clone(),
            self.snapshot_interval,
            compression.clone(),
        )));

        // エンティティ状態ストアを作成
        let entity_states = Arc::new(std::sync::Mutex::new(EntityStateStore::new(backend.clone())));

        // ベクトルインデックスを作成（HNSW使用、API確認中はSimpleVectorIndexを使用）
        use incdb_storage::index::vector_index::SimpleVectorIndex;
        let base_index = Box::new(SimpleVectorIndex::new());
        let vector_index = Arc::new(std::sync::Mutex::new(
            OptimizedVectorIndex::new(base_index, self.vector_dim),
        ));

        // EventSourcedGraphを作成
        let graph = EventSourcedGraph::new(
            event_stream,
            snapshot_store,
            entity_states,
            vector_index,
            compression,
        );

        Ok(Arc::new(tokio::sync::Mutex::new(graph)))
    }
}

impl Default for EventSourcedGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

