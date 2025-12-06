//! WorldGraph
//!
//! Incidence の世界モデル
//!
//! 借用チェッカーを活用するため、WorldGraph を唯一の可変所有者とする

use crate::model::{IId, Incidence, Level, RoleId};
use crate::foundation::axioms::{CoinductiveUniverse, Structure};
use std::collections::HashMap;

/// WorldGraph
///
/// すべての Incidence を管理する世界モデル
///
/// 借用チェッカーを活用するため、この構造体が唯一の可変所有者となる
pub struct WorldGraph {
    /// Incidence のストレージ
    /// Arena パターンで借用チェッカーの安全性を保証
    incidences: Vec<Incidence>,
    /// ID からインデックスへのマップ
    id_to_index: HashMap<IId, usize>,
    /// Type 別インデックス
    by_type: HashMap<IId, Vec<IId>>,
    /// Role 別インデックス
    by_role: HashMap<RoleId, Vec<IId>>,
    /// 次の ID
    next_id: u64,
}

impl WorldGraph {
    /// 新しい WorldGraph を作成
    pub fn new() -> Self {
        Self {
            incidences: Vec::new(),
            id_to_index: HashMap::new(),
            by_type: HashMap::new(),
            by_role: HashMap::new(),
            next_id: 1,
        }
    }

    /// 新しい Incidence を追加
    ///
    /// 借用チェッカーにより、&mut self を通じてのみ操作可能
    pub fn add_incidence(&mut self, mut incidence: Incidence) -> IId {
        let id = incidence.id;
        let index = self.incidences.len();

        // ID が未設定の場合は自動生成
        let id = if id.0 == 0 {
            let new_id = IId(self.next_id);
            self.next_id += 1;
            incidence.id = new_id;
            new_id
        } else {
            id
        };

        // インデックスを更新
        self.id_to_index.insert(incidence.id, index);

        // Type インデックスを更新
        if let Some(ty) = incidence.ty {
            self.by_type.entry(ty).or_default().push(incidence.id);
        }

        // Role インデックスを更新
        for role in &incidence.roles {
            self.by_role.entry(*role).or_default().push(incidence.id);
        }

        self.incidences.push(incidence);
        id
    }

    /// Incidence を取得（不変参照）
    ///
    /// 借用チェッカーにより、&self で安全にアクセス可能
    pub fn get(&self, id: IId) -> Option<&Incidence> {
        self.id_to_index
            .get(&id)
            .and_then(|&index| self.incidences.get(index))
    }

    /// Incidence を取得（可変参照）
    ///
    /// 借用チェッカーにより、&mut self で排他的にアクセス可能
    pub fn get_mut(&mut self, id: IId) -> Option<&mut Incidence> {
        self.id_to_index
            .get(&id)
            .and_then(|&index| self.incidences.get_mut(index))
    }

    /// 新しい ID を生成
    pub fn new_id(&mut self) -> IId {
        let id = IId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Type で Incidence を検索
    pub fn find_by_type(&self, ty: IId) -> impl Iterator<Item = &Incidence> {
        self.by_type
            .get(&ty)
            .into_iter()
            .flat_map(|ids| ids.iter())
            .filter_map(|id| self.get(*id))
    }

    /// Role で Incidence を検索
    pub fn find_by_role(&self, role: RoleId) -> impl Iterator<Item = &Incidence> {
        self.by_role
            .get(&role)
            .into_iter()
            .flat_map(|ids| ids.iter())
            .filter_map(|id| self.get(*id))
    }

    /// すべての Incidence をイテレート
    pub fn iter(&self) -> impl Iterator<Item = &Incidence> {
        self.incidences.iter()
    }

    /// Incidence の数を取得
    pub fn len(&self) -> usize {
        self.incidences.len()
    }

    /// 空かどうかチェック
    pub fn is_empty(&self) -> bool {
        self.incidences.is_empty()
    }
}

impl CoinductiveUniverse for WorldGraph {
    fn structure(&self, id: IId) -> Option<Structure> {
        self.get(id).map(|inc| Structure {
            args: inc.args.clone(),
            val: inc.val.clone(),
        })
    }
}

impl Default for WorldGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Value;

    #[test]
    fn test_world_graph() {
        let mut graph = WorldGraph::new();
        let id1 = graph.new_id();
        let id2 = graph.new_id();

        let inc1 = Incidence::new(id1, Level::zero())
            .with_val(Value::string("test"));
        let inc2 = Incidence::new(id2, Level::zero())
            .add_arg(id1, RoleId(1));

        graph.add_incidence(inc1);
        graph.add_incidence(inc2);

        assert_eq!(graph.len(), 2);
        assert!(graph.get(id1).is_some());
        assert!(graph.get(id2).is_some());
    }

    #[test]
    fn test_find_by_type() {
        let mut graph = WorldGraph::new();
        let type_id = graph.new_id();
        let id1 = graph.new_id();
        let id2 = graph.new_id();

        let inc1 = Incidence::new(id1, Level::zero()).with_type(type_id);
        let inc2 = Incidence::new(id2, Level::zero()).with_type(type_id);

        graph.add_incidence(inc1);
        graph.add_incidence(inc2);

        let found: Vec<_> = graph.find_by_type(type_id).collect();
        assert_eq!(found.len(), 2);
    }
}

