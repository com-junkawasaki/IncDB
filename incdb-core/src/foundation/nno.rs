//! Natural Number Object (NNO)
//!
//! 自然数対象の構成

use crate::model::{IId, Level};
use crate::foundation::axioms::TypeId;

/// NNO の定義
///
/// Nat, zero, succ を Incidence として表現
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NatDef {
    /// Nat 型の ID
    pub nat_ty: TypeId,
    /// zero の ID
    pub zero: IId,
    /// succ の ID
    pub succ: IId,
}

impl NatDef {
    /// 新しい NNO 定義を作成
    pub fn new(nat_ty: TypeId, zero: IId, succ: IId) -> Self {
        Self { nat_ty, zero, succ }
    }
}

/// NNO の witness
///
/// 普遍性（初期代数）の証明スケッチ
pub struct NnoWitness {
    /// NNO の定義
    pub nno: NatDef,
    // 初期性の検証関数（オプション）
    // 実際の証明は実行時に検証される
}

impl NnoWitness {
    /// 新しい witness を作成
    pub fn new(nno: NatDef) -> Self {
        Self { nno }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nno_def() {
        let nat_ty = TypeId(IId(1));
        let zero = IId(2);
        let succ = IId(3);
        let nno = NatDef::new(nat_ty.clone(), zero, succ);

        assert_eq!(nno.nat_ty, nat_ty);
        assert_eq!(nno.zero, zero);
        assert_eq!(nno.succ, succ);
    }
}

