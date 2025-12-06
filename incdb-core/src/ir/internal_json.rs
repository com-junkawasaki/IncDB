//! Internal JSON Format
//!
//! {nodes: [], incidences: []} 形式の実装

use crate::model::{IId, Incidence, Level, RoleId, Value};
use serde::{Deserialize, Serialize};

/// 内部 JSON 形式
///
/// 計算・圧縮・ベクター処理に最適化された形式
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InternalJson {
    /// ノード（軽量な属性のみ）
    pub nodes: Vec<NodeJson>,
    /// インシデンス（主役）
    pub incidences: Vec<IncidenceJson>,
}

impl InternalJson {
    /// 空の InternalJson を作成
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            incidences: Vec::new(),
        }
    }

    /// Incidence を追加
    pub fn add_incidence(&mut self, inc: IncidenceJson) {
        self.incidences.push(inc);
    }

    /// Node を追加
    pub fn add_node(&mut self, node: NodeJson) {
        self.nodes.push(node);
    }
}

impl Default for InternalJson {
    fn default() -> Self {
        Self::new()
    }
}

/// Node JSON（軽量）
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeJson {
    /// Node ID
    pub id: String,
    /// Type
    pub r#type: Option<String>,
    /// 属性
    pub attrs: Option<serde_json::Value>,
}

/// Incidence JSON（主役）
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IncidenceJson {
    /// Incidence ID
    pub id: String,
    /// 関係ノード ID
    pub relation: String,
    /// 引数ノード ID
    pub arg: String,
    /// ロール
    pub role: String,
    /// 順序
    pub order: Option<u8>,
    /// 属性
    pub attrs: Option<serde_json::Value>,
    /// ベクトル埋め込み
    pub vector: Option<Vec<f32>>,
}

impl From<&crate::model::Incidence> for IncidenceJson {
    fn from(inc: &crate::model::Incidence) -> Self {
        // 簡易実装: 最初の arg と role を使用
        // 実際の実装では、すべての args を展開する必要がある
        Self {
            id: format!("inc:{}", inc.id.0),
            relation: inc.ty.map(|t| format!("rel:{}", t.0)).unwrap_or_default(),
            arg: inc.args.first().map(|a| format!("arg:{}", a.0)).unwrap_or_default(),
            role: inc.roles.first().map(|r| format!("role:{}", r.0)).unwrap_or_default(),
            order: Some(0),
            attrs: inc.val.as_ref().map(|v| serde_json::to_value(v).unwrap_or_default()),
            vector: inc.embedding.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_json() {
        let mut json = InternalJson::new();
        let inc = IncidenceJson {
            id: "inc:1".to_string(),
            relation: "rel:1".to_string(),
            arg: "arg:2".to_string(),
            role: "role:1".to_string(),
            order: Some(0),
            attrs: None,
            vector: None,
        };
        json.add_incidence(inc);

        assert_eq!(json.incidences.len(), 1);
    }
}

