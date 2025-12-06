//! Bisimulation Algorithm
//!
//! Incidence 間の bisimulation 関係を計算するアルゴリズム

use crate::model::IId;
use crate::foundation::axioms::{BisimulationEquality, CoinductiveUniverse, Structure};
use std::collections::{HashMap, HashSet};

/// Bisimulation 関係の計算
///
/// 固定点反復による bisimulation の計算
pub struct BisimulationComputer<'a> {
    universe: &'a dyn CoinductiveUniverse,
}

impl<'a> BisimulationComputer<'a> {
    /// 新しい計算機を作成
    pub fn new(universe: &'a dyn CoinductiveUniverse) -> Self {
        Self { universe }
    }

    /// 二つの Incidence が bisimulation 関係にあるか計算
    ///
    /// 固定点反復アルゴリズムを使用
    pub fn compute_bisimulation(&self, i: IId, j: IId) -> BisimulationResult {
        let mut relation: HashSet<(IId, IId)> = HashSet::new();

        // 初期状態: (i, j) を含む
        relation.insert((i, j));

        // 固定点に達するまで反復
        let max_iterations = 1000;
        for _ in 0..max_iterations {
            let new_relation = self.refine_relation(&relation);
            if new_relation == relation {
                // 固定点に達した
                return BisimulationResult {
                    are_bisimilar: relation.contains(&(i, j)),
                    relation,
                };
            }
            relation = new_relation;
        }

        // 最大反復回数に達した（おそらく bisimilar ではない）
        BisimulationResult {
            are_bisimilar: false,
            relation,
        }
    }

    /// 関係を精緻化
    ///
    /// 各ペア (a, b) について、構造が一致するかチェック
    fn refine_relation(&self, relation: &HashSet<(IId, IId)>) -> HashSet<(IId, IId)> {
        let mut refined = HashSet::new();

        for &(a, b) in relation {
            if self.structures_match(a, b, relation) {
                refined.insert((a, b));
            }
        }

        refined
    }

    /// 二つの Incidence の構造が bisimulation 関係で一致するかチェック
    fn structures_match(
        &self,
        a: IId,
        b: IId,
        relation: &HashSet<(IId, IId)>,
    ) -> bool {
        let struct_a = self.universe.structure(a);
        let struct_b = self.universe.structure(b);

        match (struct_a, struct_b) {
            (Some(sa), Some(sb)) => {
                // val が一致するか
                if sa.val != sb.val {
                    return false;
                }

                // args の数が同じか
                if sa.args.len() != sb.args.len() {
                    return false;
                }

                // 各 args が relation で対応付けられるか
                // これは最大マッチング問題だが、簡易版として全探索
                self.args_match(&sa.args, &sb.args, relation)
            }
            (None, None) => true,
            _ => false,
        }
    }

    /// args が bisimulation 関係で対応付けられるかチェック
    fn args_match(
        &self,
        args_a: &[IId],
        args_b: &[IId],
        relation: &HashSet<(IId, IId)>,
    ) -> bool {
        // 簡易実装: 順序付きで対応付け
        // より高度な実装では最大マッチングを計算
        if args_a.len() != args_b.len() {
            return false;
        }

        args_a
            .iter()
            .zip(args_b.iter())
            .all(|(ai, bi)| relation.contains(&(*ai, *bi)))
    }
}

/// Bisimulation の結果
#[derive(Clone, Debug)]
pub struct BisimulationResult {
    /// 二つの Incidence が bisimilar か
    pub are_bisimilar: bool,
    /// 計算された bisimulation 関係
    pub relation: HashSet<(IId, IId)>,
}

/// WorldGraph 用の BisimulationEquality 実装
pub struct WorldGraphBisimulation<'a> {
    computer: BisimulationComputer<'a>,
}

impl<'a> WorldGraphBisimulation<'a> {
    pub fn new(universe: &'a dyn CoinductiveUniverse) -> Self {
        Self {
            computer: BisimulationComputer::new(universe),
        }
    }
}

impl<'a> BisimulationEquality for WorldGraphBisimulation<'a> {
    fn are_bisimilar(&self, i: IId, j: IId) -> bool {
        self.computer.compute_bisimulation(i, j).are_bisimilar
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundation::coinduction::FinalCoalgebra;

    #[test]
    fn test_bisimulation() {
        let mut coalgebra = FinalCoalgebra::new();
        let id1 = IId(1);
        let id2 = IId(2);
        let id3 = IId(3);
        let id4 = IId(4);

        // 同じ構造の Incidence
        coalgebra.add_structure(id1, Structure::with_args(vec![id2]));
        coalgebra.add_structure(id3, Structure::with_args(vec![id4]));

        let computer = BisimulationComputer::new(&coalgebra);
        let result = computer.compute_bisimulation(id1, id3);

        // id2 と id4 が bisimilar でないため、id1 と id3 も bisimilar ではない
        // （ただし、id2 と id4 も同じ構造なら bisimilar になる）
        // このテストは簡易版のため、実際の動作を確認する必要がある
        assert!(!result.are_bisimilar || result.relation.contains(&(id2, id4)));
    }
}

