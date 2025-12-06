//! Coinductive Query
//!
//! 無限構造の探索（深さ制限付き）

use incdb_core::model::{IId, WorldGraph};
use incdb_core::foundation::coinduction::CoinductiveChecker;
use std::collections::HashSet;

/// Coinductive クエリ
pub struct CoinductiveQuery {
    /// 開始 Incidence
    pub start: IId,
    /// 最大深さ
    pub max_depth: usize,
    /// 訪問済み集合
    visited: HashSet<IId>,
}

impl CoinductiveQuery {
    /// 新しい Coinductive クエリを作成
    pub fn new(start: IId, max_depth: usize) -> Self {
        Self {
            start,
            max_depth,
            visited: HashSet::new(),
        }
    }

    /// クエリを実行
    ///
    /// 深さ制限付きで Incidence を探索
    pub fn execute(&mut self, graph: &WorldGraph) -> Vec<IId> {
        let mut results = Vec::new();
        self.dfs(graph, self.start, 0, &mut results);
        results
    }

    /// 深さ優先探索
    fn dfs(
        &mut self,
        graph: &WorldGraph,
        current: IId,
        depth: usize,
        results: &mut Vec<IId>,
    ) {
        if depth > self.max_depth {
            return;
        }

        if self.visited.contains(&current) {
            return; // 循環を検出
        }

        self.visited.insert(current);
        results.push(current);

        // 現在の Incidence の args を探索
        if let Some(inc) = graph.get(current) {
            for &arg_id in &inc.args {
                self.dfs(graph, arg_id, depth + 1, results);
            }
        }
    }
}

/// Bisimulation クエリ
///
/// 二つの Incidence が bisimilar かチェック
pub struct BisimulationQuery {
    /// 第一の Incidence
    pub i: IId,
    /// 第二の Incidence
    pub j: IId,
}

impl BisimulationQuery {
    /// 新しい Bisimulation クエリを作成
    pub fn new(i: IId, j: IId) -> Self {
        Self { i, j }
    }

    /// クエリを実行
    pub fn execute(&self, graph: &WorldGraph) -> bool {
        use incdb_core::foundation::bisimulation::BisimulationComputer;
        let computer = BisimulationComputer::new(graph);
        let result = computer.compute_bisimulation(self.i, self.j);
        result.are_bisimilar
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use incdb_core::model::{Level, RoleId};

    #[test]
    fn test_coinductive_query() {
        let mut graph = WorldGraph::new();
        let id1 = graph.new_id();
        let id2 = graph.new_id();
        let id3 = graph.new_id();

        let inc1 = incdb_core::model::Incidence::new(id1, Level::zero())
            .add_arg(id2, RoleId(1));
        let inc2 = incdb_core::model::Incidence::new(id2, Level::zero())
            .add_arg(id3, RoleId(1));
        let inc3 = incdb_core::model::Incidence::new(id3, Level::zero());

        graph.add_incidence(inc1);
        graph.add_incidence(inc2);
        graph.add_incidence(inc3);

        let mut query = CoinductiveQuery::new(id1, 10);
        let results = query.execute(&graph);

        assert!(results.contains(&id1));
        assert!(results.contains(&id2));
        assert!(results.contains(&id3));
    }
}

