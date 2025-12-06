//! Storage Backend
//!
//! 永続化バックエンドの抽象化

pub mod traits;
pub mod sled_backend;

pub use traits::{Backend, BackendError};
pub use sled_backend::SledBackend;

