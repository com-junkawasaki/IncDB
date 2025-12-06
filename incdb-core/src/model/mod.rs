//! Core Model
//!
//! Incidence 構造と WorldGraph の実装

pub mod incidence;
pub mod world_graph;
pub mod value;

pub use incidence::{IId, Level, Incidence, RoleId};
pub use world_graph::WorldGraph;
pub use value::Value;

