//! Coinduction Support
//!
//! Final coalgebra と coinductive な構造の実装

use crate::model::IId;
use crate::foundation::axioms::{CoinductiveUniverse, Structure};
use std::collections::{HashMap, HashSet};

/// Final coalgebra の実装
///
/// I ≅ P_fin(I) の構造を表現
pub struct FinalCoalgebra {
    /// Incidence の構造マップ
    structures: HashMap<IId, Structure>,
}

impl FinalCoalgebra {
    /// 新しい FinalCoalgebra を作成
    pub fn new() -> Self {
        Self {
            structures: HashMap::new(),
        }
    }

    /// 構造を追加
    pub fn add_structure(&mut self, id: IId, structure: Structure) {
        self.structures.insert(id, structure);
    }

    /// 構造を取得
    pub fn get_structure(&self, id: IId) -> Option<&Structure> {
        self.structures.get(&id)
    }
}

impl CoinductiveUniverse for FinalCoalgebra {
    fn structure(&self, id: IId) -> Option<Structure> {
        self.get_structure(id).cloned()
    }
}

impl Default for FinalCoalgebra {
    fn default() -> Self {
        Self::new()
    }
}

/// Coinductive な性質のチェック
///
/// 無限構造に対する性質を coinduction で検証
pub struct CoinductiveChecker<'a> {
    universe: &'a dyn CoinductiveUniverse,
    visited: HashSet<(IId, IId)>,
    max_depth: usize,
}

impl<'a> CoinductiveChecker<'a> {
    /// 新しいチェッカーを作成
    pub fn new(universe: &'a dyn CoinductiveUniverse, max_depth: usize) -> Self {
        Self {
            universe,
            visited: HashSet::new(),
            max_depth,
        }
    }

    /// 二つの Incidence が coinductive に等しいかチェック
    ///
    /// 深さ制限付きで bisimulation を計算
    pub fn check_equality(&mut self, i: IId, j: IId, depth: usize) -> bool {
        if depth > self.max_depth {
            return false; // 深さ制限に達した
        }

        if self.visited.contains(&(i, j)) {
            return true; // 循環を検出、coinductive に等しい
        }

        self.visited.insert((i, j));

        let struct_i = self.universe.structure(i);
        let struct_j = self.universe.structure(j);

        match (struct_i, struct_j) {
            (Some(si), Some(sj)) => {
                // args の数が同じか
                if si.args.len() != sj.args.len() {
                    return false;
                }

                // val が一致するか
                if si.val != sj.val {
                    return false;
                }

                // 各 args が coinductive に等しいか
                si.args
                    .iter()
                    .zip(sj.args.iter())
                    .all(|(ai, aj)| self.check_equality(*ai, *aj, depth + 1))
            }
            (None, None) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Value;

    #[test]
    fn test_final_coalgebra() {
        let mut coalgebra = FinalCoalgebra::new();
        let id1 = IId(1);
        let id2 = IId(2);

        coalgebra.add_structure(id1, Structure::with_args(vec![id2]));
        coalgebra.add_structure(id2, Structure::with_args(vec![id1])); // 循環

        assert!(coalgebra.get_structure(id1).is_some());
        assert!(coalgebra.get_structure(id2).is_some());
    }

    #[test]
    fn test_coinductive_equality() {
        let mut coalgebra = FinalCoalgebra::new();
        let id1 = IId(1);
        let id2 = IId(2);
        let id3 = IId(3);
        let id4 = IId(4);

        // id1 -> id2 -> id1 (循環)
        coalgebra.add_structure(id1, Structure::with_args(vec![id2]));
        coalgebra.add_structure(id2, Structure::with_args(vec![id1]));

        // id3 -> id4 -> id3 (同じパターンの循環)
        coalgebra.add_structure(id3, Structure::with_args(vec![id4]));
        coalgebra.add_structure(id4, Structure::with_args(vec![id3]));

        let mut checker = CoinductiveChecker::new(&coalgebra, 10);
        assert!(checker.check_equality(id1, id3, 0));
    }
}

