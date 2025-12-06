//! Core Model
//!
//! Incidence 構造と WorldGraph の実装

pub mod incidence;
pub mod world_graph;
pub mod value;
pub mod event;
pub mod event_sourced_graph;
pub mod event_sourced_graph_builder;

pub use incidence::{IId, Level, Incidence, RoleId};
pub use world_graph::WorldGraph;
pub use value::Value;
pub use event::{Event, EventType};
pub use event_sourced_graph::{EventSourcedGraph, EventSourcedGraphError};
pub use event_sourced_graph_builder::EventSourcedGraphBuilder;

