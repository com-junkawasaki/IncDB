//! Optimized Vector Index
//!
//! HNSWベースの最適化ベクトルインデックス（コンテキスト付き検索）

use crate::index::vector_index::{VectorIndex, VectorIndexError};
use incdb_core::model::IId;
use std::collections::HashMap;
use thiserror::Error;

/// 最適化ベクトルインデックスエラー
#[derive(Error, Debug)]
pub enum OptimizedVectorIndexError {
    #[error("Vector index error: {0}")]
    VectorIndex(#[from] VectorIndexError),
    #[error("Dimension mismatch")]
    DimensionMismatch,
    #[error("Entity not found: {0}")]
    EntityNotFound(IId),
}

/// コンテキスト情報
#[derive(Clone, Debug)]
pub struct VectorContext {
    /// エンティティID（オプション）
    pub entity_id: Option<IId>,
    /// タイムスタンプ（オプション）
    pub timestamp: Option<i64>,
}

/// クエリフィルタ
#[derive(Clone, Debug)]
pub struct QueryFilters {
    /// タイムスタンプ範囲（start, end）
    pub time_range: Option<(i64, i64)>,
    /// エンティティフィルタ
    pub entity_filter: Option<IId>,
}

/// 最適化ベクトルインデックス
///
/// HNSWベースのベクトルインデックスにコンテキスト情報を追加
pub struct OptimizedVectorIndex {
    /// ベクトルインデックス（HNSWまたはSimpleVectorIndex）
    base_index: Box<dyn VectorIndex>,
    /// ベクトルID → コンテキスト情報
    contexts: HashMap<IId, VectorContext>,
    /// エンティティID → ベクトルIDのリスト
    entity_to_vectors: HashMap<IId, Vec<IId>>,
    /// ベクトルの次元数
    dimension: usize,
}

impl OptimizedVectorIndex {
    /// 新しい最適化ベクトルインデックスを作成（HNSW使用）
    pub fn new_hnsw(dimension: usize) -> Result<Self, OptimizedVectorIndexError> {
        use crate::index::hnsw_index::HNSWVectorIndex;
        let base_index = Box::new(HNSWVectorIndex::new_default(dimension)
            .map_err(|e| OptimizedVectorIndexError::VectorIndex(VectorIndexError::Index(e.to_string())))?);
        Ok(Self {
            base_index,
            contexts: HashMap::new(),
            entity_to_vectors: HashMap::new(),
            dimension,
        })
    }

    /// HNSWパラメータを指定して作成
    pub fn new_hnsw_with_params(
        dimension: usize,
        m: usize,
        ef_construction: usize,
        ef_search: usize,
    ) -> Result<Self, OptimizedVectorIndexError> {
        use crate::index::hnsw_index::HNSWVectorIndex;
        let base_index = Box::new(HNSWVectorIndex::new(dimension, m, ef_construction, ef_search)
            .map_err(|e| OptimizedVectorIndexError::VectorIndex(VectorIndexError::Index(e.to_string())))?);
        Ok(Self {
            base_index,
            contexts: HashMap::new(),
            entity_to_vectors: HashMap::new(),
            dimension,
        })
    }

    /// 新しい最適化ベクトルインデックスを作成（SimpleVectorIndex使用、後方互換性のため）
    pub fn new(base_index: Box<dyn VectorIndex>, dimension: usize) -> Self {
        Self {
            base_index,
            contexts: HashMap::new(),
            entity_to_vectors: HashMap::new(),
            dimension,
        }
    }

    /// ベクトルを追加
    ///
    /// # Arguments
    /// * `id` - ベクトルID（通常はEvent ID）
    /// * `vector` - ベクトル
    /// * `entity_id` - エンティティID（オプション）
    /// * `timestamp` - タイムスタンプ（オプション）
    pub fn add(
        &mut self,
        id: IId,
        vector: &[f32],
        entity_id: Option<IId>,
        timestamp: Option<i64>,
    ) -> Result<(), OptimizedVectorIndexError> {
        if vector.len() != self.dimension {
            return Err(OptimizedVectorIndexError::DimensionMismatch);
        }

        // ベースインデックスに追加
        self.base_index
            .add(id, vector)
            .map_err(OptimizedVectorIndexError::VectorIndex)?;

        // コンテキスト情報を保存
        let context = VectorContext {
            entity_id,
            timestamp,
        };
        self.contexts.insert(id, context.clone());

        // エンティティインデックスを更新
        if let Some(eid) = entity_id {
            self.entity_to_vectors
                .entry(eid)
                .or_insert_with(Vec::new)
                .push(id);
        }

        Ok(())
    }

    /// ベクトルを削除
    pub fn remove(&mut self, id: IId) -> Result<(), OptimizedVectorIndexError> {
        // コンテキスト情報を取得
        if let Some(context) = self.contexts.remove(&id) {
            // エンティティインデックスから削除
            if let Some(entity_id) = context.entity_id {
                if let Some(vectors) = self.entity_to_vectors.get_mut(&entity_id) {
                    vectors.retain(|&vid| vid != id);
                }
            }
        }

        // ベースインデックスから削除
        self.base_index
            .remove(id)
            .map_err(OptimizedVectorIndexError::VectorIndex)?;

        Ok(())
    }

    /// コンテキスト付きベクトル検索
    ///
    /// # Arguments
    /// * `query` - クエリベクトル
    /// * `k` - 取得するベクトル数
    /// * `filters` - クエリフィルタ
    ///
    /// # Returns
    /// (ベクトルID, 類似度) のリスト
    pub fn search_with_context(
        &self,
        query: &[f32],
        k: usize,
        filters: QueryFilters,
    ) -> Result<Vec<(IId, f32)>, OptimizedVectorIndexError> {
        if query.len() != self.dimension {
            return Err(OptimizedVectorIndexError::DimensionMismatch);
        }

        // まずベースインデックスで検索（kを大きく取って後でフィルタ）
        let search_k = k * 10; // フィルタ後の結果がk個になるように多めに取得
        let mut candidates = self
            .base_index
            .search(query, search_k)
            .map_err(OptimizedVectorIndexError::VectorIndex)?;

        // フィルタを適用
        if let Some(entity_filter) = filters.entity_filter {
            candidates.retain(|(id, _)| {
                self.contexts
                    .get(id)
                    .and_then(|ctx| ctx.entity_id)
                    .map(|eid| eid == entity_filter)
                    .unwrap_or(false)
            });
        }

        if let Some((start, end)) = filters.time_range {
            candidates.retain(|(id, _)| {
                self.contexts
                    .get(id)
                    .and_then(|ctx| ctx.timestamp)
                    .map(|ts| ts >= start && ts <= end)
                    .unwrap_or(false)
            });
        }

        // Top-Kを返す
        candidates.truncate(k);
        Ok(candidates)
    }

    /// エンティティのベクトルを検索
    ///
    /// # Arguments
    /// * `entity_id` - エンティティID
    /// * `query` - クエリベクトル
    /// * `k` - 取得するベクトル数
    ///
    /// # Returns
    /// (ベクトルID, 類似度) のリスト
    pub fn search_for_entity(
        &self,
        entity_id: IId,
        query: &[f32],
        k: usize,
    ) -> Result<Vec<(IId, f32)>, OptimizedVectorIndexError> {
        let filters = QueryFilters {
            time_range: None,
            entity_filter: Some(entity_id),
        };
        self.search_with_context(query, k, filters)
    }

    /// インデックスのサイズ
    pub fn len(&self) -> usize {
        self.base_index.len()
    }

    /// インデックスが空かどうか
    pub fn is_empty(&self) -> bool {
        self.base_index.is_empty()
    }

    /// エンティティのベクトル数を取得
    pub fn vector_count_for_entity(&self, entity_id: IId) -> usize {
        self.entity_to_vectors
            .get(&entity_id)
            .map(|v| v.len())
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::vector_index::SimpleVectorIndex;

    #[test]
    fn test_optimized_vector_index() {
        let base_index = Box::new(SimpleVectorIndex::new());
        let mut index = OptimizedVectorIndex::new(base_index, 3);

        let entity_id = IId(1);
        let vector1 = vec![1.0, 0.0, 0.0];
        let vector2 = vec![0.0, 1.0, 0.0];
        let query = vec![1.0, 0.0, 0.0];

        index
            .add(IId(1), &vector1, Some(entity_id), Some(1000))
            .unwrap();
        index
            .add(IId(2), &vector2, Some(entity_id), Some(2000))
            .unwrap();

        let filters = QueryFilters {
            time_range: None,
            entity_filter: Some(entity_id),
        };
        let results = index.search_with_context(&query, 2, filters).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, IId(1)); // vector1が最も近い
    }

    #[test]
    fn test_time_range_filter() {
        let base_index = Box::new(SimpleVectorIndex::new());
        let mut index = OptimizedVectorIndex::new(base_index, 3);

        let vector1 = vec![1.0, 0.0, 0.0];
        let vector2 = vec![0.0, 1.0, 0.0];
        let query = vec![1.0, 0.0, 0.0];

        index.add(IId(1), &vector1, None, Some(1000)).unwrap();
        index.add(IId(2), &vector2, None, Some(3000)).unwrap();

        let filters = QueryFilters {
            time_range: Some((1500, 4000)),
            entity_filter: None,
        };
        let results = index.search_with_context(&query, 2, filters).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, IId(2)); // タイムスタンプが範囲内のもののみ
    }
}

