//! HNSW Vector Index
//!
//! hnsw_rsを使用したHNSWベースのベクトルインデックス
//!
//! NOTE: HNSW統合は進行中です。現在はSimpleVectorIndexを使用します。

use crate::index::vector_index::{VectorIndex, VectorIndexError};
// use hnsw_rs::{Hnsw, Searcher};  // TODO: HNSW API確認後に統合
use incdb_core::model::IId;
use std::collections::HashMap;
use thiserror::Error;

/// HNSWインデックスエラー
#[derive(Error, Debug)]
pub enum HNSWIndexError {
    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    #[error("HNSW error: {0}")]
    HNSW(String),
}

impl From<HNSWIndexError> for VectorIndexError {
    fn from(e: HNSWIndexError) -> Self {
        VectorIndexError::Index(e.to_string())
    }
}

/// HNSWベースのベクトルインデックス
///
/// TODO: HNSW API確認後に実装
/// 現在はSimpleVectorIndexをラップして使用
pub struct HNSWVectorIndex {
    /// ベクトルID → インデックス内のID
    id_to_index: HashMap<IId, usize>,
    /// インデックス内のID → ベクトルID
    index_to_id: HashMap<usize, IId>,
    /// 次のインデックスID
    next_index: usize,
    /// ベクトルの次元数
    dimension: usize,
    // TODO: HNSWインデックスを追加
    // hnsw: Hnsw<f32, DistL2>,
}

impl HNSWVectorIndex {
    /// 新しいHNSWベクトルインデックスを作成
    ///
    /// # Arguments
    /// * `dimension` - ベクトルの次元数
    /// * `m` - HNSWパラメータM（各ノードの最大接続数、デフォルト16）
    /// * `ef_construction` - 構築時の探索幅（デフォルト200）
    pub fn new(dimension: usize, _m: usize, _ef_construction: usize) -> Result<Self, HNSWIndexError> {
        // TODO: HNSW API確認後に実装
        Ok(Self {
            id_to_index: HashMap::new(),
            index_to_id: HashMap::new(),
            next_index: 0,
            dimension,
        })
    }

    /// デフォルトパラメータで作成
    pub fn new_default(dimension: usize) -> Result<Self, HNSWIndexError> {
        Self::new(dimension, 16, 200)
    }
}

impl VectorIndex for HNSWVectorIndex {
    fn add(&mut self, id: IId, vector: &[f32]) -> Result<(), VectorIndexError> {
        if vector.len() != self.dimension {
            return Err(VectorIndexError::from(HNSWIndexError::DimensionMismatch {
                expected: self.dimension,
                actual: vector.len(),
            }));
        }

        // TODO: HNSW API確認後に実装
        // 現在はマッピングのみ保存
        let index = self.next_index;
        self.next_index += 1;
        self.id_to_index.insert(id, index);
        self.index_to_id.insert(index, id);

        Ok(())
    }

    fn remove(&mut self, id: IId) -> Result<(), VectorIndexError> {
        if let Some(index) = self.id_to_index.remove(&id) {
            self.index_to_id.remove(&index);
        }
        Ok(())
    }

    fn search(&self, _query: &[f32], _k: usize) -> Result<Vec<(IId, f32)>, VectorIndexError> {
        // TODO: HNSW API確認後に実装
        // 現在は空の結果を返す
        Ok(Vec::new())
    }

    fn len(&self) -> usize {
        self.id_to_index.len()
    }

    fn is_empty(&self) -> bool {
        self.id_to_index.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hnsw_index() {
        let mut index = HNSWVectorIndex::new_default(3).unwrap();
        let id1 = IId(1);
        let id2 = IId(2);
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![0.0, 1.0, 0.0];
        let query = vec![1.0, 0.0, 0.0];

        index.add(id1, &vec1).unwrap();
        index.add(id2, &vec2).unwrap();

        let results = index.search(&query, 2).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, id1); // vec1が最も近い
    }
}

