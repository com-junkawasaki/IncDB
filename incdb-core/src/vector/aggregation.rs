//! Embedding Aggregation
//!
//! Incidence 埋め込みから Node/Relation 埋め込みを集約

use crate::model::{IId, WorldGraph};

/// ノード埋め込みを集約
///
/// ノードに関連するすべての Incidence の埋め込みから、ノードの埋め込みを計算
pub fn aggregate_node_embedding(
    graph: &WorldGraph,
    node_id: IId,
) -> Option<Vec<f32>> {
    // このノードを引数として持つすべての Incidence を取得
    let incidences: Vec<_> = graph
        .iter()
        .filter(|inc| inc.args.contains(&node_id))
        .collect();

    if incidences.is_empty() {
        return None;
    }

    // 埋め込みを持つ Incidence をフィルタ
    let embeddings: Vec<&Vec<f32>> = incidences
        .iter()
        .filter_map(|inc| inc.embedding.as_ref())
        .collect();

    if embeddings.is_empty() {
        return None;
    }

    // 次元を確認
    let dim = embeddings[0].len();
    if !embeddings.iter().all(|e| e.len() == dim) {
        return None;
    }

    // 平均を計算
    let mut aggregated = vec![0.0; dim];
    for embedding in &embeddings {
        for (i, &val) in embedding.iter().enumerate() {
            aggregated[i] += val;
        }
    }

    let count = embeddings.len() as f32;
    for val in &mut aggregated {
        *val /= count;
    }

    Some(aggregated)
}

/// 関係埋め込みを集約
///
/// 関係に関連するすべての Incidence の埋め込みから、関係の埋め込みを計算
pub fn aggregate_relation_embedding(
    graph: &WorldGraph,
    relation_id: IId,
) -> Option<Vec<f32>> {
    // この関係を Type として持つすべての Incidence を取得
    let incidences: Vec<_> = graph
        .iter()
        .filter(|inc| inc.ty == Some(relation_id))
        .collect();

    if incidences.is_empty() {
        return None;
    }

    // 埋め込みを持つ Incidence をフィルタ
    let embeddings: Vec<&Vec<f32>> = incidences
        .iter()
        .filter_map(|inc| inc.embedding.as_ref())
        .collect();

    if embeddings.is_empty() {
        return None;
    }

    // 次元を確認
    let dim = embeddings[0].len();
    if !embeddings.iter().all(|e| e.len() == dim) {
        return None;
    }

    // 平均を計算
    let mut aggregated = vec![0.0; dim];
    for embedding in &embeddings {
        for (i, &val) in embedding.iter().enumerate() {
            aggregated[i] += val;
        }
    }

    let count = embeddings.len() as f32;
    for val in &mut aggregated {
        *val /= count;
    }

    Some(aggregated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Level, RoleId};
    use crate::vector::embedding::Embedding;

    #[test]
    fn test_aggregate_node_embedding() {
        let mut graph = WorldGraph::new();
        let node_id = graph.new_id();
        let inc1_id = graph.new_id();
        let inc2_id = graph.new_id();

        let inc1 = Incidence::new(inc1_id, Level::zero())
            .add_arg(node_id, RoleId(1));
        let inc2 = Incidence::new(inc2_id, Level::zero())
            .add_arg(node_id, RoleId(2));

        graph.add_incidence(inc1);
        graph.add_incidence(inc2);

        Embedding::set_embedding(&mut graph, inc1_id, vec![1.0, 0.0]).unwrap();
        Embedding::set_embedding(&mut graph, inc2_id, vec![0.0, 1.0]).unwrap();

        let aggregated = aggregate_node_embedding(&graph, node_id).unwrap();
        assert_eq!(aggregated, vec![0.5, 0.5]);
    }
}

