//! Incidence Structure
//!
//! Incidence の基本構造定義

use crate::model::Value;
use serde::{Deserialize, Serialize};

/// Incidence ID
///
/// インデックスベースの参照により、借用チェッカーを活用
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IId(pub u64);

/// Type Universe Level
///
/// U₀, U₁, U₂, ... の階層を表現
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Level(pub u8);

impl Level {
    /// Level 0 を作成
    pub fn zero() -> Self {
        Self(0)
    }

    /// 次の level に進む
    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }

    /// level が指定された level 以下かチェック
    pub fn le(self, other: Level) -> bool {
        self.0 <= other.0
    }
}

/// Role ID
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoleId(pub u32);

/// Incidence 構造
///
/// 借用チェッカーを活用するため、IId による参照を使用
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Incidence {
    /// Incidence ID
    pub id: IId,
    /// Universe level
    pub level: Level,
    /// Type（オプション）
    pub ty: Option<IId>,
    /// 引数（自己参照可能）
    pub args: Vec<IId>,
    /// ロール（args と同じ長さ）
    pub roles: Vec<RoleId>,
    /// 値（オプション）
    pub val: Option<Value>,
    /// ベクトル埋め込み（オプション）
    pub embedding: Option<Vec<f32>>,
}

impl Incidence {
    /// 新しい Incidence を作成
    pub fn new(id: IId, level: Level) -> Self {
        Self {
            id,
            level,
            ty: None,
            args: Vec::new(),
            roles: Vec::new(),
            val: None,
            embedding: None,
        }
    }

    /// Type を設定
    pub fn with_type(mut self, ty: IId) -> Self {
        self.ty = Some(ty);
        self
    }

    /// 引数とロールを追加
    pub fn add_arg(mut self, arg: IId, role: RoleId) -> Self {
        self.args.push(arg);
        self.roles.push(role);
        self
    }

    /// 値を設定
    pub fn with_val(mut self, val: Value) -> Self {
        self.val = Some(val);
        self
    }

    /// ベクトル埋め込みを設定
    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }

    /// args と roles の整合性をチェック
    pub fn is_valid(&self) -> bool {
        self.args.len() == self.roles.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incidence_creation() {
        let id = IId(1);
        let level = Level::zero();
        let incidence = Incidence::new(id, level);

        assert_eq!(incidence.id, id);
        assert_eq!(incidence.level, level);
        assert!(incidence.ty.is_none());
        assert!(incidence.args.is_empty());
        assert!(incidence.is_valid());
    }

    #[test]
    fn test_incidence_with_args() {
        let id = IId(1);
        let arg_id = IId(2);
        let role = RoleId(1);
        let incidence = Incidence::new(id, Level::zero()).add_arg(arg_id, role);

        assert_eq!(incidence.args.len(), 1);
        assert_eq!(incidence.roles.len(), 1);
        assert_eq!(incidence.args[0], arg_id);
        assert_eq!(incidence.roles[0], role);
        assert!(incidence.is_valid());
    }
}

