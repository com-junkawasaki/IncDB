//! Intermediate Representation
//!
//! JSON-LD と内部 JSON 形式の実装

pub mod jsonld;
pub mod internal_json;
pub mod converter;

pub use jsonld::{JsonLd, JsonLdContext, JsonLdIncidence};
pub use internal_json::{InternalJson, NodeJson, IncidenceJson};
pub use converter::{JsonLdConverter, InternalJsonConverter};

