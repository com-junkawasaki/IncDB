//! HNSW Vector Index
//!
//! hnsw_rsを使用したHNSWベースのベクトルインデックス

use crate::index::vector_index::{VectorIndex, VectorIndexError};
use hnsw_rs::prelude::*;
use incdb_core::model::IId;
use std::collections::HashMap;
use std::sync::Mutex;
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
pub struct HNSWVectorIndex {
    /// HNSWインデックス（Mutexで保護）
    hnsw: Mutex<Hnsw<f32, DistL2>>,
    /// ベクトルID → インデックス内のID
    id_to_index: Mutex<HashMap<IId, usize>>,
    /// インデックス内のID → ベクトルID
    index_to_id: Mutex<HashMap<usize, IId>>,
    /// 次のインデックスID
    next_index: Mutex<usize>,
    /// ベクトルの次元数
    dimension: usize,
    /// 検索時のefパラメータ
    ef_search: usize,
}

impl HNSWVectorIndex {
    /// 新しいHNSWベクトルインデックスを作成
    ///
    /// # Arguments
    /// * `dimension` - ベクトルの次元数
    /// * `m` - HNSWパラメータM（各ノードの最大接続数、デフォルト16）
    /// * `ef_construction` - 構築時の探索幅（デフォルト200）
    /// * `ef_search` - 検索時の探索幅（デフォルト50）
    pub fn new(dimension: usize, m: usize, ef_construction: usize, ef_search: usize) -> Result<Self, HNSWIndexError> {
        let nb_layer = 16; // レイヤー数
        let hnsw = Hnsw::<f32, DistL2>::new(m, dimension, nb_layer, ef_construction, DistL2);
        
        Ok(Self {
            hnsw: Mutex::new(hnsw),
            id_to_index: Mutex::new(HashMap::new()),
            index_to_id: Mutex::new(HashMap::new()),
            next_index: Mutex::new(0),
            dimension,
            ef_search,
        })
    }

    /// デフォルトパラメータで作成
    pub fn new_default(dimension: usize) -> Result<Self, HNSWIndexError> {
        Self::new(dimension, 16, 200, 50)
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

        // 既存のエントリを削除（マッピングのみ）
        let mut id_to_index = self.id_to_index.lock().unwrap();
        let mut index_to_id = self.index_to_id.lock().unwrap();
        if let Some(&old_index) = id_to_index.get(&id) {
            index_to_id.remove(&old_index);
            id_to_index.remove(&id);
        }

        // 次のインデックスIDを取得
        let mut next_index = self.next_index.lock().unwrap();
        let index = *next_index;
        *next_index += 1;

        // HNSWにベクトルを追加
        let mut hnsw = self.hnsw.lock().unwrap();
        hnsw.insert((vector, index));

        // マッピングを更新
        id_to_index.insert(id, index);
        index_to_id.insert(index, id);

        Ok(())
    }

    fn remove(&mut self, id: IId) -> Result<(), VectorIndexError> {
        // HNSWは削除をサポートしていないため、マッピングのみ削除
        let mut id_to_index = self.id_to_index.lock().unwrap();
        let mut index_to_id = self.index_to_id.lock().unwrap();
        if let Some(index) = id_to_index.remove(&id) {
            index_to_id.remove(&index);
        }
        Ok(())
    }

    fn search(&self, query: &[f32], k: usize) -> Result<Vec<(IId, f32)>, VectorIndexError> {
        if query.len() != self.dimension {
            return Err(VectorIndexError::from(HNSWIndexError::DimensionMismatch {
                expected: self.dimension,
                actual: query.len(),
            }));
        }

        // HNSWで検索
        let hnsw = self.hnsw.lock().unwrap();
        let ef_search = self.ef_search.max(k);
        let results = hnsw.search(query, ef_search, k);

        // 結果をIIdと類似度に変換
        let index_to_id = self.index_to_id.lock().unwrap();
        let mut result_vec = Vec::new();
        
        for (index, distance) in results {
            if let Some(&id) = index_to_id.get(&index) {
                // 距離を類似度に変換（負の距離 = 類似度、L2距離なので小さいほど類似）
                // コサイン類似度に近づけるため、1 / (1 + distance)を使用
                let similarity = 1.0 / (1.0 + distance);
                result_vec.push((id, similarity));
            }
        }

        // 類似度でソート（降順）
        result_vec.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(result_vec)
    }

    fn len(&self) -> usize {
        self.id_to_index.lock().unwrap().len()
    }

    fn is_empty(&self) -> bool {
        self.id_to_index.lock().unwrap().is_empty()
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

