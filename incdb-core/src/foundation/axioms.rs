//! Incidence Foundation Axioms (AF0-AF5)
//!
//! 公理系の型定義と基本構造

use crate::model::{IId, Level, Value, RoleId};

/// AF0: Coinductive Universe
///
/// I は自己参照・循環構造を含む final coalgebra として定義される
pub trait CoinductiveUniverse {
    /// Final coalgebra の構造を取得
    fn structure(&self, id: IId) -> Option<Structure>;
}

/// AF1: 構造写像
///
/// 各 Incidence は有限個の Incidence を args に持つ
#[derive(Clone, Debug, PartialEq)]
pub struct Structure {
    /// この Incidence が参照する他の Incidence の集合
    pub args: Vec<IId>,
    /// 値（オプション）
    pub val: Option<Value>,
}

impl Structure {
    /// 空の構造を作成
    pub fn empty() -> Self {
        Self {
            args: Vec::new(),
            val: None,
        }
    }

    /// args のみの構造を作成
    pub fn with_args(args: Vec<IId>) -> Self {
        Self { args, val: None }
    }

    /// val のみの構造を作成
    pub fn with_val(val: Value) -> Self {
        Self {
            args: Vec::new(),
            val: Some(val),
        }
    }
}

/// AF2: Bisimulation Equality
///
/// 二つの Incidence が同じ世界的存在であることを表す
pub trait BisimulationEquality {
    /// 二つの Incidence が bisimulation 関係にあるかチェック
    fn are_bisimilar(&self, i: IId, j: IId) -> bool;
}

/// AF3: Type = Incidence pattern
///
/// Type は特殊な構造を満たす Incidence として定義される
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TypeId(pub IId);

/// Type の signature（必須ロールなど）
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeSignature {
    /// 必須ロール
    pub required_roles: Vec<RoleId>,
    /// Universe level
    pub level: Level,
}

/// AF4: Set = extensional Incidence class
///
/// Set は extensionality を満たす Incidence のクラスとして定義される
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SetId(pub IId);

/// AF5: Category = structured incidence
///
/// Category（Obj/Mor/dom/cod/composition）はすべて Incidence のパターンとして表現
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CategoryId(pub IId);

// RoleId は model モジュールから再エクスポート

