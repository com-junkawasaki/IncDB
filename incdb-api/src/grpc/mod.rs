//! gRPC API
//!
//! tonic を使用した gRPC サーバー実装

pub mod server;
pub mod service;

pub use server::IncDBServer;
pub use service::IncDBServiceImpl;

