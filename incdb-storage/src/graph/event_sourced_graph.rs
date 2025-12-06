//! Event Sourced Graph
//!
//! イベントソーシングベースのWorldGraph代替

use incdb_core::foundation::axioms::{CoinductiveUniverse, Structure};
use incdb_core::model::{Event, IId, Incidence, Level, RoleId, Value};
use crate::compression::{ProductQuantization, TemporalCompression};
use crate::event::{
    EntityStateStore, EntityStateStoreError, EventStream, EventStreamError, SnapshotStore,
    SnapshotStoreError,
};
use crate::index::optimized_vector_index::{
    OptimizedVectorIndex, QueryFilters,
};
use crate::index::vector_index::SimpleVectorIndex;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use thiserror::Error;

/// EventSourcedGraphエラー
#[derive(Error, Debug)]
pub enum EventSourcedGraphError {
    #[error("Event stream error: {0}")]
    EventStream(#[from] EventStreamError),
    #[error("Snapshot store error: {0}")]
    SnapshotStore(#[from] SnapshotStoreError),
    #[error("Entity state store error: {0}")]
    EntityState(#[from] EntityStateStoreError),
    #[error("Vector index error: {0}")]
    VectorIndex(String),
    #[error("Invalid state: {0}")]
    InvalidState(String),
}

/// イベントソーシングベースのWorldGraph代替
pub struct EventSourcedGraph {
    event_stream: Arc<Mutex<EventStream>>,
    snapshot_store: Arc<Mutex<SnapshotStore>>,
    entity_states: Arc<Mutex<EntityStateStore>>,
    vector_index: Arc<Mutex<OptimizedVectorIndex>>,
    compression: TemporalCompression,
    /// 次のID
    next_id: u64,
}

impl EventSourcedGraph {
    /// 新しいEventSourcedGraphを作成
    pub fn new(
        event_stream: Arc<Mutex<EventStream>>,
        snapshot_store: Arc<Mutex<SnapshotStore>>,
        entity_states: Arc<Mutex<EntityStateStore>>,
        vector_index: Arc<Mutex<OptimizedVectorIndex>>,
        compression: TemporalCompression,
    ) -> Self {
        Self {
            event_stream,
            snapshot_store,
            entity_states,
            vector_index,
            compression,
            next_id: 1,
        }
    }

    /// Incidenceを追加（Eventに変換して保存）
    ///
    /// # Arguments
    /// * `incidence` - 追加するIncidence
    /// * `timestamp` - タイムスタンプ（Noneの場合は現在時刻）
    ///
    /// # Returns
    /// Incidence ID
    pub async fn add_incidence(
        &mut self,
        mut incidence: Incidence,
        timestamp: Option<i64>,
    ) -> Result<IId, EventSourcedGraphError> {
        // IDが未設定の場合は自動生成
        if incidence.id.0 == 0 {
            incidence.id = IId(self.next_id);
            self.next_id += 1;
        }

        let timestamp = timestamp.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64
        });

        // IncidenceをEventに変換
        let mut event: Event = incidence.clone().into();
        event.timestamp = timestamp;

        // イベントストリームに追加
        {
            let mut stream = self.event_stream.lock().await;
            stream.append(event.clone()).await?;
        }

        // エンティティ状態を更新
        {
            let mut states = self.entity_states.lock().await;
            states.update(event.entity_id, &event).await?;
        }

        // ベクトル埋め込みがあればベクトルインデックスに追加
        if let Some(embedding) = &event.embedding {
            let mut index = self.vector_index.lock().await;
            index.add(
                event.id,
                embedding,
                Some(event.entity_id),
                Some(event.timestamp),
            )
            .map_err(|e| EventSourcedGraphError::VectorIndex(e.to_string()))?;
        }

        Ok(incidence.id)
    }

    /// Incidenceを取得（イベントリプレイ）
    ///
    /// # Arguments
    /// * `id` - Incidence ID
    ///
    /// # Returns
    /// Incidence（見つからない場合はNone）
    pub async fn get(&self, id: IId) -> Result<Option<Incidence>, EventSourcedGraphError> {
        // イベントストリームからイベントを取得
        let stream = self.event_stream.lock().await;
        match stream.get(id).await {
            Ok(event) => {
                // EventをIncidenceに変換
                let incidence: Incidence = event.into();
                Ok(Some(incidence))
            }
            Err(EventStreamError::NotFound(_)) => Ok(None),
            Err(e) => Err(EventSourcedGraphError::EventStream(e)),
        }
    }

    /// 時系列範囲クエリ
    ///
    /// # Arguments
    /// * `start` - 開始タイムスタンプ
    /// * `end` - 終了タイムスタンプ
    /// * `entity_filter` - エンティティフィルタ（オプション）
    ///
    /// # Returns
    /// 該当するIncidenceのリスト
    pub async fn query_time_range(
        &self,
        start: i64,
        end: i64,
        entity_filter: Option<IId>,
    ) -> Result<Vec<Incidence>, EventSourcedGraphError> {
        let stream = self.event_stream.lock().await;
        let events = stream.range(start, end).await?;

        // エンティティフィルタを適用
        let filtered_events: Vec<Event> = if let Some(entity_id) = entity_filter {
            events
                .into_iter()
                .filter(|e| e.entity_id == entity_id)
                .collect()
        } else {
            events
        };

        // EventをIncidenceに変換
        let incidences: Vec<Incidence> = filtered_events.into_iter().map(|e| e.into()).collect();
        Ok(incidences)
    }

    /// エンティティtraversal
    ///
    /// # Arguments
    /// * `entity_id` - エンティティID
    /// * `max_depth` - 最大深度
    ///
    /// # Returns
    /// 関連するIncidence IDのリスト
    pub async fn traverse_entity(
        &self,
        entity_id: IId,
        max_depth: usize,
    ) -> Result<Vec<IId>, EventSourcedGraphError> {
        let stream = self.event_stream.lock().await;
        let events = stream.events_for_entity(entity_id).await?;

        let mut visited = std::collections::HashSet::new();
        let mut result = Vec::new();
        let mut current_level = vec![entity_id];
        visited.insert(entity_id);

        for _depth in 0..max_depth {
            if current_level.is_empty() {
                break;
            }

            let mut next_level = Vec::new();

            for entity_id in current_level {
                // このエンティティに関連するイベントを取得
                let related_events: Vec<Event> = events
                    .iter()
                    .filter(|e| e.entity_id == entity_id || e.related_entities.iter().any(|(id, _)| *id == entity_id))
                    .cloned()
                    .collect();

                for event in related_events {
                    if !visited.contains(&event.id) {
                        visited.insert(event.id);
                        result.push(event.id);
                    }

                    // 関連エンティティを次のレベルに追加
                    for (related_id, _role) in &event.related_entities {
                        if !visited.contains(related_id) {
                            visited.insert(*related_id);
                            next_level.push(*related_id);
                        }
                    }
                }
            }

            current_level = next_level;
        }

        Ok(result)
    }

    /// ベクトル検索（コンテキスト付き）
    ///
    /// # Arguments
    /// * `query_vector` - クエリベクトル
    /// * `k` - 取得するベクトル数
    /// * `filters` - クエリフィルタ
    ///
    /// # Returns
    /// (Incidence ID, 類似度) のリスト
    pub async fn vector_search(
        &self,
        query_vector: Vec<f32>,
        k: usize,
        filters: QueryFilters,
    ) -> Result<Vec<(IId, f32)>, EventSourcedGraphError> {
        let index = self.vector_index.lock().await;
        index
            .search_with_context(&query_vector, k, filters)
            .map_err(|e| EventSourcedGraphError::VectorIndex(e.to_string()))
    }

    /// 新しいIDを生成
    pub fn new_id(&mut self) -> IId {
        let id = IId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Incidenceの数を取得（概算）
    pub async fn len(&self) -> Result<usize, EventSourcedGraphError> {
        let stream = self.event_stream.lock().await;
        Ok(stream.len())
    }

    /// 空かどうかチェック
    pub async fn is_empty(&self) -> Result<bool, EventSourcedGraphError> {
        let stream = self.event_stream.lock().await;
        Ok(stream.is_empty())
    }

    /// TypeでIncidenceを検索
    pub async fn find_by_type(
        &self,
        ty: IId,
    ) -> Result<Vec<Incidence>, EventSourcedGraphError> {
        // エンティティ状態からタイプでフィルタ
        let states = self.entity_states.lock().await;
        let entity_states = states.all_states();

        let mut result = Vec::new();
        for state in entity_states {
            // タイプが一致するエンティティのイベントを取得
            let stream = self.event_stream.lock().await;
            let events = stream.events_for_entity(state.entity_id).await?;
                for event in events {
                    // EventからIncidenceに変換してタイプをチェック
                    let incidence: Incidence = event.into();
                    if incidence.ty == Some(ty) {
                        result.push(incidence);
                    }
                }
        }

        Ok(result)
    }

    /// RoleでIncidenceを検索
    pub async fn find_by_role(
        &self,
        role: RoleId,
    ) -> Result<Vec<Incidence>, EventSourcedGraphError> {
        let stream = self.event_stream.lock().await;
        let all_ids = stream.len();
        let mut result = Vec::new();

        // 簡易実装: すべてのイベントを走査
        // 実際の実装では、roleインデックスを使用
        for i in 1..=all_ids {
            if let Ok(Some(incidence)) = self.get(IId(i as u64)).await {
                if incidence.roles.contains(&role) {
                    result.push(incidence);
                }
            }
        }

        Ok(result)
    }

    /// すべてのIncidenceをイテレート
    pub async fn iter(&self) -> Result<Vec<Incidence>, EventSourcedGraphError> {
        let stream = self.event_stream.lock().await;
        let latest = stream.latest(10000).await?; // 最新10000件を取得
        Ok(latest.into_iter().map(|e| e.into()).collect())
    }
}

/// CoinductiveUniverseトレイトの実装
impl CoinductiveUniverse for EventSourcedGraph {
    fn structure(&self, id: IId) -> Option<Structure> {
        // 非同期メソッドを同期的に呼び出すため、tokio::runtimeを使用
        // 注意: これは簡易実装で、実際の実装では適切な非同期処理が必要
        let handle = tokio::runtime::Handle::try_current().ok()?;
        if let Ok(Some(incidence)) = handle.block_on(self.get(id)) {
            Some(Structure {
                args: incidence.args,
                val: incidence.val,
            })
        } else {
            None
        }
    }
}

impl Default for EventSourcedGraph {
    fn default() -> Self {
        // デフォルト実装は提供しない（必要なコンポーネントを明示的に作成する必要がある）
        panic!("EventSourcedGraph::default() is not implemented. Use EventSourcedGraph::new() instead.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::sled_backend::SledBackend;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_event_sourced_graph() {
        let temp_dir = TempDir::new().unwrap();
        let backend = Arc::new(SledBackend::new(temp_dir.path()).unwrap());

        let event_stream = Arc::new(tokio::sync::Mutex::new(EventStream::new(backend.clone())));
        let pq = ProductQuantization::new(128, 8, 256).ok();
        let compression = TemporalCompression::new(true, pq, false);
        let snapshot_store = Arc::new(tokio::sync::Mutex::new(SnapshotStore::new(
            backend.clone(),
            Duration::from_secs(3600),
            compression.clone(),
        )));
        let entity_states = Arc::new(tokio::sync::Mutex::new(EntityStateStore::new(backend.clone())));
        let base_index = Box::new(SimpleVectorIndex::new());
        let vector_index = Arc::new(tokio::sync::Mutex::new(OptimizedVectorIndex::new(base_index, 128)));

        let graph = EventSourcedGraph::new(
            event_stream,
            snapshot_store,
            entity_states,
            vector_index,
            compression,
        );
        let graph = Arc::new(tokio::sync::Mutex::new(graph));

        let mut graph_guard = graph.lock().await;
        let id = graph_guard.new_id();
        let incidence = Incidence::new(id, Level::zero())
            .with_val(Value::string("Test"));

        let result_id = graph_guard.add_incidence(incidence, None).await.unwrap();
        assert_eq!(result_id, id);
        drop(graph_guard);

        let graph_guard = graph.lock().await;
        let retrieved = graph_guard.get(id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, id);
    }
}

