//! JSON-LD Format
//!
//! 外部公開 / 知識共有 / Web連携用の JSON-LD 形式

use crate::model::{IId, Incidence};
use serde::{Deserialize, Serialize};

/// JSON-LD ドキュメント
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonLd {
    /// @context
    #[serde(rename = "@context")]
    pub context: JsonLdContext,
    /// @graph（Incidence のリスト）
    #[serde(rename = "@graph")]
    pub graph: Vec<JsonLdIncidence>,
}

impl JsonLd {
    /// 新しい JSON-LD ドキュメントを作成
    pub fn new(context: JsonLdContext) -> Self {
        Self {
            context,
            graph: Vec::new(),
        }
    }

    /// Incidence を追加
    pub fn add_incidence(&mut self, inc: JsonLdIncidence) {
        self.graph.push(inc);
    }
}

/// JSON-LD Context
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonLdContext {
    /// Vocabulary base
    #[serde(rename = "@vocab")]
    pub vocab: Option<String>,
    /// その他の定義
    #[serde(flatten)]
    pub definitions: serde_json::Map<String, serde_json::Value>,
}

impl JsonLdContext {
    /// デフォルトの context を作成
    pub fn default() -> Self {
        let mut definitions = serde_json::Map::new();
        definitions.insert(
            "inc".to_string(),
            serde_json::json!("http://example.org/incidence#"),
        );
        definitions.insert(
            "x".to_string(),
            serde_json::json!({
                "@id": "http://example.org/incidence#x",
                "@type": "@id"
            }),
        );
        definitions.insert(
            "y".to_string(),
            serde_json::json!({
                "@id": "http://example.org/incidence#y",
                "@type": "@id"
            }),
        );
        definitions.insert(
            "role".to_string(),
            serde_json::json!("http://example.org/incidence#role"),
        );
        definitions.insert(
            "vector".to_string(),
            serde_json::json!("http://example.org/incidence#vector"),
        );

        Self {
            vocab: Some("http://example.org/incidence#".to_string()),
            definitions,
        }
    }
}

impl Default for JsonLdContext {
    fn default() -> Self {
        Self::default()
    }
}

/// JSON-LD Incidence
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonLdIncidence {
    /// @id
    #[serde(rename = "@id")]
    pub id: String,
    /// @type
    #[serde(rename = "@type")]
    pub r#type: String,
    /// x（引数）
    pub x: Option<String>,
    /// y（関係）
    pub y: Option<String>,
    /// role
    pub role: Option<String>,
    /// vector
    pub vector: Option<Vec<f32>>,
}

impl From<&Incidence> for JsonLdIncidence {
    fn from(inc: &Incidence) -> Self {
        Self {
            id: format!("inc:{}", inc.id.0),
            r#type: "Incidence".to_string(),
            x: inc.args.first().map(|a| format!("arg:{}", a.0)),
            y: inc.ty.map(|t| format!("rel:{}", t.0)),
            role: inc.roles.first().map(|r| format!("role:{}", r.0)),
            vector: inc.embedding.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonld_context() {
        let context = JsonLdContext::default();
        assert!(context.vocab.is_some());
    }

    #[test]
    fn test_jsonld() {
        let context = JsonLdContext::default();
        let mut jsonld = JsonLd::new(context);
        let inc = JsonLdIncidence {
            id: "inc:1".to_string(),
            r#type: "Incidence".to_string(),
            x: Some("arg:2".to_string()),
            y: Some("rel:1".to_string()),
            role: Some("role:1".to_string()),
            vector: None,
        };
        jsonld.add_incidence(inc);

        assert_eq!(jsonld.graph.len(), 1);
    }
}

