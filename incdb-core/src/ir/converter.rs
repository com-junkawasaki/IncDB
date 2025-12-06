//! Converter
//!
//! JSON-LD ↔ 内部 JSON の変換

use crate::ir::{InternalJson, JsonLd};
use crate::ir::internal_json::IncidenceJson;
use crate::ir::jsonld::JsonLdIncidence;
use crate::model::WorldGraph;

/// JSON-LD から内部 JSON への変換器
pub struct JsonLdConverter;

impl JsonLdConverter {
    /// JSON-LD を内部 JSON に変換
    pub fn to_internal(jsonld: &JsonLd) -> InternalJson {
        let mut internal = InternalJson::new();

        for inc in &jsonld.graph {
            let incidence = IncidenceJson {
                id: inc.id.clone(),
                relation: inc.y.clone().unwrap_or_default(),
                arg: inc.x.clone().unwrap_or_default(),
                role: inc.role.clone().unwrap_or_default(),
                order: Some(0),
                attrs: None,
                vector: inc.vector.clone(),
            };
            internal.add_incidence(incidence);
        }

        internal
    }
}

/// 内部 JSON から JSON-LD への変換器
pub struct InternalJsonConverter;

impl InternalJsonConverter {
    /// 内部 JSON を JSON-LD に変換
    pub fn to_jsonld(internal: &InternalJson) -> JsonLd {
        let context = crate::ir::JsonLdContext::default();
        let mut jsonld = JsonLd::new(context);

        for inc in &internal.incidences {
            let jsonld_inc = JsonLdIncidence {
                id: inc.id.clone(),
                r#type: "Incidence".to_string(),
                x: Some(inc.arg.clone()),
                y: Some(inc.relation.clone()),
                role: Some(inc.role.clone()),
                vector: inc.vector.clone(),
            };
            jsonld.add_incidence(jsonld_inc);
        }

        jsonld
    }

    /// WorldGraph から JSON-LD に変換
    pub fn from_world_graph(graph: &WorldGraph) -> JsonLd {
        let context = crate::ir::JsonLdContext::default();
        let mut jsonld = JsonLd::new(context);

        for inc in graph.iter() {
            let jsonld_inc: JsonLdIncidence = inc.into();
            jsonld.add_incidence(jsonld_inc);
        }

        jsonld
    }

    /// WorldGraph から内部 JSON に変換
    pub fn to_internal_from_graph(graph: &WorldGraph) -> InternalJson {
        let mut internal = InternalJson::new();

        for inc in graph.iter() {
            let incidence: IncidenceJson = inc.into();
            internal.add_incidence(incidence);
        }

        internal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion() {
        let context = crate::ir::JsonLdContext::default();
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

        let internal = JsonLdConverter::to_internal(&jsonld);
        assert_eq!(internal.incidences.len(), 1);

        let back = InternalJsonConverter::to_jsonld(&internal);
        assert_eq!(back.graph.len(), 1);
    }
}

