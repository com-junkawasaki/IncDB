//! Role Index
//!
//! Role 別のインデックス

use incdb_core::model::RoleId;
use incdb_core::model::IId;
use std::collections::HashMap;

/// Role インデックス
pub struct RoleIndex {
    /// Role ID -> Incidence IDs
    index: HashMap<RoleId, Vec<IId>>,
}

impl RoleIndex {
    /// 新しい RoleIndex を作成
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }

    /// Role と Incidence の関連を追加
    pub fn add(&mut self, role_id: RoleId, incidence_id: IId) {
        self.index.entry(role_id).or_default().push(incidence_id);
    }

    /// Role で Incidence を検索
    pub fn find(&self, role_id: RoleId) -> Option<&Vec<IId>> {
        self.index.get(&role_id)
    }

    /// Role で Incidence を検索（イテレータ）
    pub fn find_iter(&self, role_id: RoleId) -> impl Iterator<Item = &IId> {
        self.index
            .get(&role_id)
            .into_iter()
            .flat_map(|ids| ids.iter())
    }

    /// インデックスをクリア
    pub fn clear(&mut self) {
        self.index.clear();
    }
}

impl Default for RoleIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_index() {
        let mut index = RoleIndex::new();
        let role_id = RoleId(1);
        let inc_id1 = IId(10);
        let inc_id2 = IId(11);

        index.add(role_id, inc_id1);
        index.add(role_id, inc_id2);

        let found: Vec<_> = index.find_iter(role_id).collect();
        assert_eq!(found.len(), 2);
        assert!(found.contains(&&inc_id1));
        assert!(found.contains(&&inc_id2));
    }
}

