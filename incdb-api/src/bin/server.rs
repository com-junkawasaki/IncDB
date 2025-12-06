//! IncDB GraphQL Server
//!
//! GraphQL サーバーを起動するためのバイナリ

use incdb_api::graphql::GraphQLServer;
use incdb_core::model::WorldGraph;
use std::sync::{Arc, Mutex};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 環境変数から設定を取得
    let addr = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    
    // WorldGraph の作成
    let graph = Arc::new(Mutex::new(WorldGraph::new()));
    
    println!("Starting IncDB GraphQL server on {}", addr);
    println!("GraphQL endpoint: http://{}/graphql", addr);
    
    // サーバーを起動
    GraphQLServer::serve(&addr, graph).await?;
    
    Ok(())
}

