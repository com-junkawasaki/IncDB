//! Type System
//!
//! Type を Incidence パターンとして定義

use crate::model::{IId, Level};
use crate::foundation::axioms::{TypeId, TypeSignature};

/// Type の定義
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Type {
    /// Type ID
    pub id: TypeId,
    /// Type の signature
    pub signature: TypeSignature,
    /// Type の名前
    pub name: String,
}

impl Type {
    /// 新しい Type を作成
    pub fn new(id: TypeId, name: String, level: Level) -> Self {
        Self {
            id,
            signature: TypeSignature {
                required_roles: Vec::new(),
                level,
            },
            name,
        }
    }

    /// 必須ロールを追加
    pub fn with_required_role(mut self, role: crate::foundation::axioms::RoleId) -> Self {
        self.signature.required_roles.push(role);
        self
    }
}

/// Type チェッカー
///
/// Incidence が Type の signature を満たすかチェック
pub struct TypeChecker;

impl TypeChecker {
    /// Incidence が Type の signature を満たすかチェック
    pub fn check_signature(
        incidence: &crate::model::Incidence,
        type_def: &Type,
    ) -> bool {
        // Level チェック
        if !incidence.level.le(type_def.signature.level) {
            return false;
        }

        // 必須ロールのチェック
        for required_role in &type_def.signature.required_roles {
            if !incidence.roles.contains(required_role) {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundation::axioms::RoleId;

    #[test]
    fn test_type_creation() {
        let type_id = TypeId(IId(1));
        let level = Level::zero();
        let ty = Type::new(type_id, "TestType".to_string(), level);

        assert_eq!(ty.name, "TestType");
        assert_eq!(ty.signature.level, level);
    }

    #[test]
    fn test_type_with_roles() {
        let type_id = TypeId(IId(1));
        let role = RoleId(1);
        let ty = Type::new(type_id, "TestType".to_string(), Level::zero())
            .with_required_role(role);

        assert_eq!(ty.signature.required_roles.len(), 1);
        assert_eq!(ty.signature.required_roles[0], role);
    }
}

