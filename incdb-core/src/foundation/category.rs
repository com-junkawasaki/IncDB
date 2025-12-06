//! Category Theory
//!
//! Category を Incidence パターンとして実装

use crate::model::IId;

/// Category ID
pub type CategoryId = IId;

/// Object（対象）
///
/// Category 内の対象を表す Incidence
pub struct Object {
    /// Object の ID
    pub id: IId,
    /// 所属する Category
    pub category: CategoryId,
}

/// Morphism（射）
///
/// Category 内の射を表す Incidence
pub struct Morphism {
    /// Morphism の ID
    pub id: IId,
    /// 所属する Category
    pub category: CategoryId,
    /// ドメイン（source）
    pub dom: IId,
    /// コドメイン（target）
    pub cod: IId,
}

impl Morphism {
    /// 新しい射を作成
    pub fn new(id: IId, category: CategoryId, dom: IId, cod: IId) -> Self {
        Self {
            id,
            category,
            dom,
            cod,
        }
    }
}

/// Composition（合成）
///
/// 二つの射の合成を表す Incidence
pub struct Composition {
    /// Composition の ID
    pub id: IId,
    /// 第一の射
    pub f: IId,
    /// 第二の射
    pub g: IId,
    /// 合成結果の射
    pub composed: IId,
}

impl Composition {
    /// 新しい合成を作成
    pub fn new(id: IId, f: IId, g: IId, composed: IId) -> Self {
        Self {
            id,
            f,
            g,
            composed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_morphism() {
        let morph_id = IId(1);
        let category = IId(10);
        let dom = IId(2);
        let cod = IId(3);
        let morph = Morphism::new(morph_id, category, dom, cod);

        assert_eq!(morph.id, morph_id);
        assert_eq!(morph.category, category);
        assert_eq!(morph.dom, dom);
        assert_eq!(morph.cod, cod);
    }
}

