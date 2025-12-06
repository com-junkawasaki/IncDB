//! GraphQL API
//!
//! Poem + async-graphql を使用した GraphQL サーバー実装

pub mod schema;
pub mod server;
pub mod benchmark;
pub mod event_sourced_schema;

pub use schema::{create_schema, AppSchema, Query, Mutation};
pub use server::GraphQLServer;
pub use event_sourced_schema::{create_event_sourced_schema, EventSourcedAppSchema, EventSourcedQuery, EventSourcedMutation};
pub use benchmark::{
    BenchmarkResult, BenchmarkMetadata,
    WriteBenchmarkConfig, ReadBenchmarkConfig,
    MultiHopBenchmarkConfig, VectorHopBenchmarkConfig,
};

