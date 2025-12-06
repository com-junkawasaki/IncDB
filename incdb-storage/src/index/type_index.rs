//! Type Index
//!
//! Type 別のインデックス

use incdb_core::model::IId;
use std::collections::HashMap;

/// Type インデックス
pub struct TypeIndex {
    /// Type ID -> Incidence IDs
    index: HashMap<IId, Vec<IId>>,
}

impl TypeIndex {
    /// 新しい TypeIndex を作成
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }

    /// Type と Incidence の関連を追加
    pub fn add(&mut self, type_id: IId, incidence_id: IId) {
        self.index.entry(type_id).or_default().push(incidence_id);
    }

    /// Type で Incidence を検索
    pub fn find(&self, type_id: IId) -> Option<&Vec<IId>> {
        self.index.get(&type_id)
    }

    /// Type で Incidence を検索（イテレータ）
    pub fn find_iter(&self, type_id: IId) -> impl Iterator<Item = &IId> {
        self.index
            .get(&type_id)
            .into_iter()
            .flat_map(|ids| ids.iter())
    }

    /// インデックスをクリア
    pub fn clear(&mut self) {
        self.index.clear();
    }
}

impl Default for TypeIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_index() {
        let mut index = TypeIndex::new();
        let type_id = IId(1);
        let inc_id1 = IId(10);
        let inc_id2 = IId(11);

        index.add(type_id, inc_id1);
        index.add(type_id, inc_id2);

        let found: Vec<_> = index.find_iter(type_id).collect();
        assert_eq!(found.len(), 2);
        assert!(found.contains(&&inc_id1));
        assert!(found.contains(&&inc_id2));
    }
}

