//! Storage Example
//!
//! ストレージ層の使用例

use incdb_core::model::{IId, Incidence, Level, WorldGraph};
use incdb_storage::backend::SledBackend;
use incdb_storage::backend::Backend;
use incdb_storage::index::{TypeIndex, RoleIndex};
use tempfile::TempDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== IncDB Storage Example ===\n");

    // 1. Sled バックエンドの作成
    println!("1. Creating Sled backend...");
    let temp_dir = TempDir::new()?;
    let backend = SledBackend::new(temp_dir.path())?;
    println!("   Backend created at: {:?}", temp_dir.path());

    // 2. データの保存
    println!("\n2. Storing data...");
    let id1 = IId(1);
    let data1 = b"incidence data 1";
    backend.put_incidence(id1, data1).await?;
    println!("   Stored incidence {}", id1.0);

    // 3. データの取得
    println!("\n3. Retrieving data...");
    let retrieved = backend.get_incidence(id1).await?;
    if let Some(data) = retrieved {
        println!("   Retrieved: {:?}", String::from_utf8_lossy(&data));
    }

    // 4. インデックスの使用
    println!("\n4. Using indexes...");
    let mut type_index = TypeIndex::new();
    let type_id = IId(10);
    type_index.add(type_id, id1);
    println!("   Added to type index");

    let mut role_index = RoleIndex::new();
    let role_id = incdb_core::model::RoleId(1);
    role_index.add(role_id, id1);
    println!("   Added to role index");

    // 5. リスト取得
    println!("\n5. Listing all incidences...");
    let ids = backend.list_incidences().await?;
    println!("   Found {} incidences", ids.len());

    println!("\n=== Storage Example completed! ===");
    Ok(())
}

