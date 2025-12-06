//! Vector Query
//!
//! SIMILAR, DIST 述語の実装

use incdb_core::model::{IId, WorldGraph};
#[cfg(feature = "storage")]
use incdb_storage::index::vector_index::{VectorIndex, VectorIndexError};
use thiserror::Error;

/// ベクタークエリエラー
#[derive(Error, Debug)]
pub enum VectorQueryError {
    #[cfg(feature = "storage")]
    #[error("Vector index error: {0}")]
    Index(#[from] VectorIndexError),
    #[error("Query error: {0}")]
    Query(String),
}

/// ベクタークエリ
pub struct VectorQuery {
    /// クエリベクトル
    pub query_vector: Vec<f32>,
    /// 検索数
    pub k: usize,
}

impl VectorQuery {
    /// 新しいベクタークエリを作成
    pub fn new(query_vector: Vec<f32>, k: usize) -> Self {
        Self { query_vector, k }
    }

    /// クエリを実行
    #[cfg(feature = "storage")]
    pub fn execute<I: VectorIndex>(
        &self,
        index: &I,
    ) -> Result<Vec<(IId, f32)>, VectorQueryError> {
        index
            .search(&self.query_vector, self.k)
            .map_err(VectorQueryError::from)
    }
}

/// ハイブリッドクエリ（構造 + ベクター）
pub struct HybridQuery {
    /// ベクタークエリ
    pub vector_query: VectorQuery,
    /// 構造フィルタ（オプション）
    pub structure_filter: Option<StructureFilter>,
}

/// 構造フィルタ
pub struct StructureFilter {
    /// Type フィルタ
    pub type_filter: Option<IId>,
    /// Role フィルタ
    pub role_filter: Option<incdb_core::model::RoleId>,
}

impl HybridQuery {
    /// 新しいハイブリッドクエリを作成
    pub fn new(vector_query: VectorQuery) -> Self {
        Self {
            vector_query,
            structure_filter: None,
        }
    }

    /// 構造フィルタを追加
    pub fn with_structure_filter(mut self, filter: StructureFilter) -> Self {
        self.structure_filter = Some(filter);
        self
    }

    /// クエリを実行
    #[cfg(feature = "storage")]
    pub fn execute<I: VectorIndex>(
        &self,
        index: &I,
        graph: &WorldGraph,
    ) -> Result<Vec<(IId, f32)>, VectorQueryError> {
        // まずベクター検索
        let vector_results = self.vector_query.execute(index)?;

        // 構造フィルタを適用
        if let Some(filter) = &self.structure_filter {
            let filtered: Vec<_> = vector_results
                .into_iter()
                .filter(|(id, _)| {
                    graph.get(*id).map_or(false, |inc| {
                        // Type フィルタ
                        if let Some(type_id) = filter.type_filter {
                            if inc.ty != Some(type_id) {
                                return false;
                            }
                        }
                        // Role フィルタ
                        if let Some(role_id) = filter.role_filter {
                            if !inc.roles.contains(&role_id) {
                                return false;
                            }
                        }
                        true
                    })
                })
                .collect();
            Ok(filtered)
        } else {
            Ok(vector_results)
        }
    }
}

#[cfg(test)]
#[cfg(feature = "storage")]
mod tests {
    use super::*;
    use incdb_storage::index::vector_index::SimpleVectorIndex;

    #[test]
    fn test_vector_query() {
        let mut index = SimpleVectorIndex::new();
        let id1 = IId(1);
        let id2 = IId(2);
        let vec1 = vec![1.0, 0.0];
        let vec2 = vec![0.0, 1.0];
        let query_vec = vec![1.0, 0.0];

        index.add(id1, &vec1).unwrap();
        index.add(id2, &vec2).unwrap();

        let query = VectorQuery::new(query_vec, 2);
        let results = query.execute(&index).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, id1); // vec1 が最も近い
    }
}

