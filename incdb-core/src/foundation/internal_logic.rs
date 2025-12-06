//! Internal Logic
//!
//! トポス的構造（Ω, Sub, Π/Σ）の実装

use crate::model::IId;

/// Subobject classifier (Ω)
///
/// 命題を表す特別な型
pub struct Omega {
    /// Ω の ID
    pub id: IId,
}

impl Omega {
    /// 新しい Ω を作成
    pub fn new(id: IId) -> Self {
        Self { id }
    }
}

/// 述語
///
/// X → Ω 型の Incidence
pub struct Predicate {
    /// 述語の ID
    pub id: IId,
    /// ドメイン（X）
    pub domain: IId,
    /// コドメイン（Ω）
    pub codomain: IId,
}

impl Predicate {
    /// 新しい述語を作成
    pub fn new(id: IId, domain: IId, codomain: IId) -> Self {
        Self {
            id,
            domain,
            codomain,
        }
    }
}

/// 存在量化（Σ型）
pub struct Exists {
    /// Σ の ID
    pub id: IId,
    /// ドメイン
    pub domain: IId,
    /// 述語
    pub predicate: IId,
}

/// 全称量化（Π型）
pub struct ForAll {
    /// Π の ID
    pub id: IId,
    /// ドメイン
    pub domain: IId,
    /// 述語
    pub predicate: IId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_omega() {
        let omega_id = IId(1);
        let omega = Omega::new(omega_id);
        assert_eq!(omega.id, omega_id);
    }

    #[test]
    fn test_predicate() {
        let pred_id = IId(1);
        let domain = IId(2);
        let codomain = IId(3);
        let pred = Predicate::new(pred_id, domain, codomain);

        assert_eq!(pred.id, pred_id);
        assert_eq!(pred.domain, domain);
        assert_eq!(pred.codomain, codomain);
    }
}

