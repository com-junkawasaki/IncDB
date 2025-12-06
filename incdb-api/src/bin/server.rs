//! IncDB GraphQL Server
//!
//! GraphQL サーバーを起動するためのバイナリ

use incdb_api::graphql::GraphQLServer;
use incdb_core::model::WorldGraph;
use incdb_storage::graph::EventSourcedGraphBuilder;
use std::sync::{Arc, Mutex};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 環境変数から設定を取得（デフォルトはEventSourcingモード）
    let addr = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let use_event_sourcing = env::var("USE_EVENT_SOURCING")
        .unwrap_or_else(|_| "true".to_string())
        .parse::<bool>()
        .unwrap_or(true);
    
    if use_event_sourcing {
        println!("Using EventSourcedGraph (event sourcing mode)");
        let storage_path = env::var("INCDB_STORAGE_PATH")
            .unwrap_or_else(|_| "./data".to_string());
        
        let graph = EventSourcedGraphBuilder::new()
            .with_storage_path(storage_path)
            .build()
            .await?;
        
        println!("Starting IncDB GraphQL server on {} (Event Sourcing)", addr);
        println!("GraphQL endpoint: http://{}/graphql", addr);
        
        // EventSourcedGraph用のサーバーを起動
        GraphQLServer::serve_event_sourced(&addr, graph).await?;
    } else {
        println!("Using WorldGraph (in-memory mode)");
        // WorldGraph の作成
        let graph = Arc::new(Mutex::new(WorldGraph::new()));
        
        println!("Starting IncDB GraphQL server on {}", addr);
        println!("GraphQL endpoint: http://{}/graphql", addr);
        
        // サーバーを起動
        GraphQLServer::serve(&addr, graph).await?;
    }
    
    Ok(())
}

