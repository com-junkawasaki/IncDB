//! Sled Backend
//!
//! sled を使用したストレージバックエンド実装

use crate::backend::traits::{Backend, BackendError};
use async_trait::async_trait;
use incdb_core::model::IId;
use sled::{Db, Tree};
use std::sync::Arc;

/// Sled バックエンド
pub struct SledBackend {
    db: Arc<Db>,
    tree: Arc<Tree>,
}

impl SledBackend {
    /// 新しい SledBackend を作成
    pub fn new(path: impl AsRef<std::path::Path>) -> Result<Self, BackendError> {
        let db = sled::open(path)?;
        let tree = db.open_tree("incidences")?;

        Ok(Self {
            db: Arc::new(db),
            tree: Arc::new(tree),
        })
    }

    /// ID をキーに変換
    fn id_to_key(id: IId) -> Vec<u8> {
        id.0.to_be_bytes().to_vec()
    }

    /// キーを ID に変換
    fn key_to_id(key: &[u8]) -> Option<IId> {
        if key.len() == 8 {
            let bytes: [u8; 8] = key.try_into().ok()?;
            Some(IId(u64::from_be_bytes(bytes)))
        } else {
            None
        }
    }
}

#[async_trait]
impl Backend for SledBackend {
    async fn put_incidence(&self, id: IId, data: &[u8]) -> Result<(), BackendError> {
        let key = Self::id_to_key(id);
        self.tree.insert(key, data)?;
        self.tree.flush_async().await?;
        Ok(())
    }

    async fn get_incidence(&self, id: IId) -> Result<Option<Vec<u8>>, BackendError> {
        let key = Self::id_to_key(id);
        match self.tree.get(key)? {
            Some(ivec) => Ok(Some(ivec.to_vec())),
            None => Ok(None),
        }
    }

    async fn delete_incidence(&self, id: IId) -> Result<(), BackendError> {
        let key = Self::id_to_key(id);
        self.tree.remove(key)?;
        self.tree.flush_async().await?;
        Ok(())
    }

    async fn list_incidences(&self) -> Result<Vec<IId>, BackendError> {
        let mut ids = Vec::new();
        for result in self.tree.iter() {
            let (key, _) = result?;
            if let Some(id) = Self::key_to_id(&key) {
                ids.push(id);
            }
        }
        Ok(ids)
    }

    async fn close(&self) -> Result<(), BackendError> {
        self.tree.flush_async().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_sled_backend() {
        let temp_dir = TempDir::new().unwrap();
        let backend = SledBackend::new(temp_dir.path()).unwrap();

        let id = IId(1);
        let data = b"test data";

        // Put
        backend.put_incidence(id, data).await.unwrap();

        // Get
        let retrieved = backend.get_incidence(id).await.unwrap();
        assert_eq!(retrieved, Some(data.to_vec()));

        // List
        let ids = backend.list_incidences().await.unwrap();
        assert!(ids.contains(&id));

        // Delete
        backend.delete_incidence(id).await.unwrap();
        let retrieved = backend.get_incidence(id).await.unwrap();
        assert_eq!(retrieved, None);
    }
}

