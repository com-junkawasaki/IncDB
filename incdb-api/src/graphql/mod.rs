//! GraphQL API
//!
//! Poem + async-graphql を使用した GraphQL サーバー実装

pub mod schema;
pub mod server;
pub mod benchmark;

pub use schema::{create_schema, AppSchema, Query, Mutation};
pub use server::GraphQLServer;
pub use benchmark::{
    BenchmarkResult, BenchmarkMetadata,
    WriteBenchmarkConfig, ReadBenchmarkConfig,
    MultiHopBenchmarkConfig, VectorHopBenchmarkConfig,
};

