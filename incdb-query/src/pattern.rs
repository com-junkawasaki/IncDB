//! Pattern Match DSL
//!
//! MATCH/WHERE/RETURN 構文の実装

use incdb_core::model::{IId, WorldGraph};
use thiserror::Error;

/// パターンマッチエラー
#[derive(Error, Debug)]
pub enum PatternError {
    #[error("Pattern error: {0}")]
    Pattern(String),
    #[error("Evaluation error: {0}")]
    Evaluation(String),
}

/// パターン変数
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PatternVar(pub String);

/// パターン
#[derive(Clone, Debug)]
pub enum Pattern {
    /// Incidence パターン
    Incidence {
        var: PatternVar,
        r#type: Option<String>,
        args: Vec<PatternArg>,
    },
    /// 条件パターン
    Condition {
        left: PatternExpr,
        op: PatternOp,
        right: PatternExpr,
    },
}

/// パターン引数
#[derive(Clone, Debug)]
pub struct PatternArg {
    /// 変数または値
    pub value: PatternExpr,
    /// ロール
    pub role: Option<String>,
}

/// パターン式
#[derive(Clone, Debug)]
pub enum PatternExpr {
    /// 変数
    Var(PatternVar),
    /// 値
    Value(incdb_core::model::Value),
    /// 属性アクセス
    Attr(PatternVar, String),
}

/// パターン演算子
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatternOp {
    Eq,
    Ne,
    Gt,
    Lt,
    Ge,
    Le,
}

/// パターンマッチクエリ
pub struct PatternQuery {
    /// MATCH 節
    pub matches: Vec<Pattern>,
    /// WHERE 節
    pub where_clause: Vec<Pattern>,
    /// RETURN 節
    pub returns: Vec<PatternVar>,
}

impl PatternQuery {
    /// 新しいクエリを作成
    pub fn new() -> Self {
        Self {
            matches: Vec::new(),
            where_clause: Vec::new(),
            returns: Vec::new(),
        }
    }

    /// クエリを実行
    pub fn execute(&self, graph: &WorldGraph) -> Result<Vec<PatternResult>, PatternError> {
        // 簡易実装: 全探索
        let mut results = Vec::new();
        let mut bindings = std::collections::HashMap::new();

        // MATCH 節を評価
        for pattern in &self.matches {
            self.match_pattern(pattern, graph, &mut bindings)?;
        }

        // WHERE 節を評価
        for condition in &self.where_clause {
            if !self.check_condition(condition, &bindings, graph)? {
                return Ok(Vec::new());
            }
        }

        // RETURN 節を構築
        let result = PatternResult {
            bindings: bindings.clone(),
        };
        results.push(result);

        Ok(results)
    }

    /// パターンをマッチ
    fn match_pattern(
        &self,
        pattern: &Pattern,
        graph: &WorldGraph,
        bindings: &mut std::collections::HashMap<PatternVar, IId>,
    ) -> Result<(), PatternError> {
        match pattern {
            Pattern::Incidence { var, r#type: _type, args: _args } => {
                // 簡易実装: 最初のマッチを返す
                for inc in graph.iter() {
                    bindings.insert(var.clone(), inc.id);
                    break;
                }
                Ok(())
            }
            Pattern::Condition { .. } => {
                // WHERE 節で処理
                Ok(())
            }
        }
    }

    /// 条件をチェック
    fn check_condition(
        &self,
        condition: &Pattern,
        bindings: &std::collections::HashMap<PatternVar, IId>,
        graph: &WorldGraph,
    ) -> Result<bool, PatternError> {
        match condition {
            Pattern::Condition { left, op, right } => {
                let left_val = self.eval_expr(left, bindings, graph)?;
                let right_val = self.eval_expr(right, bindings, graph)?;
                Ok(self.compare_values(&left_val, op, &right_val))
            }
            _ => Err(PatternError::Pattern("Invalid condition".to_string())),
        }
    }

    /// 式を評価
    fn eval_expr(
        &self,
        expr: &PatternExpr,
        bindings: &std::collections::HashMap<PatternVar, IId>,
        graph: &WorldGraph,
    ) -> Result<incdb_core::model::Value, PatternError> {
        match expr {
            PatternExpr::Var(var) => {
                let id = bindings
                    .get(var)
                    .ok_or_else(|| PatternError::Evaluation(format!("Variable not found: {:?}", var)))?;
                graph
                    .get(*id)
                    .and_then(|inc| inc.val.clone())
                    .ok_or_else(|| PatternError::Evaluation("No value".to_string()))
            }
            PatternExpr::Value(val) => Ok(val.clone()),
            PatternExpr::Attr(_var, _key) => {
                // 簡易実装
                Err(PatternError::Evaluation("Attr not implemented".to_string()))
            }
        }
    }

    /// 値を比較
    fn compare_values(
        &self,
        left: &incdb_core::model::Value,
        op: &PatternOp,
        right: &incdb_core::model::Value,
    ) -> bool {
        match (left, right, op) {
            (incdb_core::model::Value::Int(l), incdb_core::model::Value::Int(r), PatternOp::Eq) => {
                l == r
            }
            (incdb_core::model::Value::Int(l), incdb_core::model::Value::Int(r), PatternOp::Gt) => {
                l > r
            }
            (incdb_core::model::Value::Int(l), incdb_core::model::Value::Int(r), PatternOp::Lt) => {
                l < r
            }
            _ => false,
        }
    }
}

impl Default for PatternQuery {
    fn default() -> Self {
        Self::new()
    }
}

/// パターンマッチの結果
#[derive(Clone, Debug)]
pub struct PatternResult {
    /// 変数バインディング
    pub bindings: std::collections::HashMap<PatternVar, IId>,
}

