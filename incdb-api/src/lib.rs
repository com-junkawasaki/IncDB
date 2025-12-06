//! IncDB API Layer
//!
//! gRPC と GraphQL API

pub mod graphql;

#[cfg(feature = "grpc")]
pub mod grpc;

