//! Backend Traits
//!
//! ストレージバックエンドの抽象トレイト

use async_trait::async_trait;
use incdb_core::model::IId;
use thiserror::Error;

/// バックエンドエラー
#[derive(Error, Debug)]
pub enum BackendError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Sled error: {0}")]
    Sled(#[from] sled::Error),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Not found: {0}")]
    NotFound(IId),
    #[error("Storage error: {0}")]
    Storage(String),
}

/// ストレージバックエンドの抽象トレイト
#[async_trait]
pub trait Backend: Send + Sync {
    /// Incidence を保存
    async fn put_incidence(&self, id: IId, data: &[u8]) -> Result<(), BackendError>;

    /// Incidence を取得
    async fn get_incidence(&self, id: IId) -> Result<Option<Vec<u8>>, BackendError>;

    /// Incidence を削除
    async fn delete_incidence(&self, id: IId) -> Result<(), BackendError>;

    /// すべての Incidence ID を取得
    async fn list_incidences(&self) -> Result<Vec<IId>, BackendError>;

    /// バックエンドを閉じる
    async fn close(&self) -> Result<(), BackendError>;
}

