//! Incidence Datalog
//!
//! Datalog パーサ・評価器

use incdb_core::model::{IId, WorldGraph};
use thiserror::Error;

/// Datalog エラー
#[derive(Error, Debug)]
pub enum DatalogError {
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Evaluation error: {0}")]
    Evaluation(String),
}

/// 述語
#[derive(Clone, Debug, PartialEq)]
pub enum Predicate {
    /// Incidence 述語
    Inc(IId),
    /// Type 述語
    Type(IId, IId),
    /// Role 述語
    Role(IId, u32, IId),
    /// 属性述語
    Attr(IId, String, incdb_core::model::Value),
}

/// Datalog ルール
#[derive(Clone, Debug)]
pub struct Rule {
    /// ヘッド（結論）
    pub head: Predicate,
    /// ボディ（条件）
    pub body: Vec<Predicate>,
}

/// Datalog プログラム
pub struct DatalogProgram {
    /// ルールのリスト
    pub rules: Vec<Rule>,
    /// ファクトのリスト
    pub facts: Vec<Predicate>,
}

impl DatalogProgram {
    /// 新しいプログラムを作成
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            facts: Vec::new(),
        }
    }

    /// ルールを追加
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    /// ファクトを追加
    pub fn add_fact(&mut self, fact: Predicate) {
        self.facts.push(fact);
    }

    /// プログラムを評価
    pub fn evaluate(&self, graph: &WorldGraph) -> Result<Vec<Predicate>, DatalogError> {
        // 簡易実装: 固定点反復
        let mut results: Vec<Predicate> = self.facts.clone();
        let mut changed = true;
        let max_iterations = 1000;

        for _ in 0..max_iterations {
            if !changed {
                break;
            }
            changed = false;

            for rule in &self.rules {
                // ボディの条件をチェック
                if self.check_body(&rule.body, graph, &results) {
                    // ヘッドを追加（重複チェック）
                    if !results.contains(&rule.head) {
                        results.push(rule.head.clone());
                        changed = true;
                    }
                }
            }
        }

        Ok(results)
    }

    /// ボディの条件をチェック
    fn check_body(
        &self,
        body: &[Predicate],
        graph: &WorldGraph,
        results: &[Predicate],
    ) -> bool {
        body.iter().all(|pred| self.check_predicate(pred, graph, results))
    }

    /// 述語をチェック
    fn check_predicate(
        &self,
        pred: &Predicate,
        graph: &WorldGraph,
        results: &[Predicate],
    ) -> bool {
        match pred {
            Predicate::Inc(id) => graph.get(*id).is_some(),
            Predicate::Type(inc_id, type_id) => {
                graph
                    .get(*inc_id)
                    .and_then(|inc| inc.ty)
                    .map(|ty| ty == *type_id)
                    .unwrap_or(false)
            }
            Predicate::Role(inc_id, role_idx, arg_id) => {
                graph
                    .get(*inc_id)
                    .and_then(|inc| {
                        inc.roles.get(*role_idx as usize).and_then(|_| {
                            inc.args.get(*role_idx as usize).map(|a| *a == *arg_id)
                        })
                    })
                    .unwrap_or(false)
            }
            Predicate::Attr(inc_id, key, expected_value) => {
                graph
                    .get(*inc_id)
                    .and_then(|inc| inc.val.as_ref())
                    .map(|val| val == expected_value)
                    .unwrap_or(false)
            }
        }
    }
}

impl Default for DatalogProgram {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use incdb_core::model::{Level, RoleId, Value};

    #[test]
    fn test_datalog_evaluation() {
        let mut graph = WorldGraph::new();
        let inc_id = graph.new_id();
        let type_id = graph.new_id();
        let arg_id = graph.new_id();

        let inc = incdb_core::model::Incidence::new(inc_id, Level::zero())
            .with_type(type_id)
            .add_arg(arg_id, RoleId(1))
            .with_val(Value::string("test"));

        graph.add_incidence(inc);

        let mut program = DatalogProgram::new();
        program.add_fact(Predicate::Inc(inc_id));

        let results = program.evaluate(&graph).unwrap();
        assert!(results.contains(&Predicate::Inc(inc_id)));
    }
}

