//! Embedding Management
//!
//! Incidence への埋め込み付与

use crate::model::{IId, Incidence, WorldGraph};
use thiserror::Error;

/// 埋め込みエラー
#[derive(Error, Debug)]
pub enum EmbeddingError {
    #[error("Embedding dimension mismatch")]
    DimensionMismatch,
    #[error("Incidence not found: {0}")]
    NotFound(IId),
}

/// 埋め込みの管理
pub struct Embedding;

impl Embedding {
    /// Incidence に埋め込みを設定
    pub fn set_embedding(
        graph: &mut WorldGraph,
        id: IId,
        embedding: Vec<f32>,
    ) -> Result<(), EmbeddingError> {
        let inc = graph
            .get_mut(id)
            .ok_or(EmbeddingError::NotFound(id))?;
        inc.embedding = Some(embedding);
        Ok(())
    }

    /// Incidence の埋め込みを取得
    pub fn get_embedding(graph: &WorldGraph, id: IId) -> Option<&Vec<f32>> {
        graph.get(id)?.embedding.as_ref()
    }

    /// 埋め込みの次元を取得
    pub fn dimension(graph: &WorldGraph, id: IId) -> Option<usize> {
        graph.get(id)?.embedding.as_ref().map(|v| v.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Level;

    #[test]
    fn test_set_embedding() {
        let mut graph = WorldGraph::new();
        let id = graph.new_id();
        let inc = Incidence::new(id, Level::zero());
        graph.add_incidence(inc);

        let embedding = vec![0.1, 0.2, 0.3];
        Embedding::set_embedding(&mut graph, id, embedding.clone()).unwrap();

        let retrieved = Embedding::get_embedding(&graph, id).unwrap();
        assert_eq!(retrieved, &embedding);
    }
}

