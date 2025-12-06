//! Vector Index
//!
//! ベクターインデックスの抽象化

use incdb_core::model::IId;
use thiserror::Error;

/// ベクターインデックスエラー
#[derive(Error, Debug)]
pub enum VectorIndexError {
    #[error("Index error: {0}")]
    Index(String),
    #[error("Dimension mismatch")]
    DimensionMismatch,
}

/// ベクターインデックスの抽象トレイト
pub trait VectorIndex: Send + Sync {
    /// ベクターを追加
    fn add(&mut self, id: IId, vector: &[f32]) -> Result<(), VectorIndexError>;

    /// ベクターを削除
    fn remove(&mut self, id: IId) -> Result<(), VectorIndexError>;

    /// 類似ベクターを検索
    fn search(&self, query: &[f32], k: usize) -> Result<Vec<(IId, f32)>, VectorIndexError>;

    /// インデックスのサイズ
    fn len(&self) -> usize;

    /// インデックスが空か
    fn is_empty(&self) -> bool;
}

/// 簡易ベクターインデックス（全探索）
///
/// 初期実装として、全探索による類似度計算を提供
pub struct SimpleVectorIndex {
    vectors: Vec<(IId, Vec<f32>)>,
}

impl SimpleVectorIndex {
    /// 新しい SimpleVectorIndex を作成
    pub fn new() -> Self {
        Self {
            vectors: Vec::new(),
        }
    }

    /// コサイン類似度を計算
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }

        let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot / (norm_a * norm_b)
    }
}

impl VectorIndex for SimpleVectorIndex {
    fn add(&mut self, id: IId, vector: &[f32]) -> Result<(), VectorIndexError> {
        // 既存のエントリを削除
        self.vectors.retain(|(existing_id, _)| *existing_id != id);
        self.vectors.push((id, vector.to_vec()));
        Ok(())
    }

    fn remove(&mut self, id: IId) -> Result<(), VectorIndexError> {
        self.vectors.retain(|(existing_id, _)| *existing_id != id);
        Ok(())
    }

    fn search(&self, query: &[f32], k: usize) -> Result<Vec<(IId, f32)>, VectorIndexError> {
        let mut results: Vec<(IId, f32)> = self
            .vectors
            .iter()
            .map(|(id, vec)| {
                let similarity = Self::cosine_similarity(query, vec);
                (*id, similarity)
            })
            .collect();

        // 類似度でソート（降順）
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Top-K を返す
        results.truncate(k);
        Ok(results)
    }

    fn len(&self) -> usize {
        self.vectors.len()
    }

    fn is_empty(&self) -> bool {
        self.vectors.is_empty()
    }
}

impl Default for SimpleVectorIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_vector_index() {
        let mut index = SimpleVectorIndex::new();
        let id1 = IId(1);
        let id2 = IId(2);
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![0.0, 1.0, 0.0];
        let query = vec![1.0, 0.0, 0.0];

        index.add(id1, &vec1).unwrap();
        index.add(id2, &vec2).unwrap();

        let results = index.search(&query, 2).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, id1); // vec1 が query に最も近い
    }
}

