//! IncDB CLI Tool

use clap::{Parser, Subcommand};
use incdb_core::model::{IId, Incidence, Level, RoleId, Value, WorldGraph};
use incdb_core::ir::InternalJsonConverter;
use std::sync::{Arc, Mutex};
use tokio;

#[derive(Parser)]
#[command(name = "incdb")]
#[command(about = "IncDB CLI Tool", version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Incidence を追加
    Add {
        /// Level
        #[arg(short, long, default_value = "0")]
        level: u8,
        /// Type ID
        #[arg(short, long)]
        type_id: Option<String>,
        /// Value
        #[arg(short, long)]
        value: Option<String>,
    },
    /// Incidence を取得
    Get {
        /// Incidence ID
        id: String,
    },
    /// クエリを実行
    Query {
        /// クエリ文字列（Datalog またはパターン）
        query: String,
    },
    /// JSON をインポート
    Import {
        /// JSON ファイルパス
        path: String,
    },
    /// JSON をエクスポート
    Export {
        /// 出力ファイルパス
        path: String,
    },
    /// サーバーを起動
    Serve {
        /// ポート
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let graph = Arc::new(Mutex::new(WorldGraph::new()));

    match cli.command {
        Commands::Add { level, type_id, value } => {
            let mut g = graph.lock().unwrap();
            let id = g.new_id();
            let mut inc = Incidence::new(id, Level(level));

            if let Some(type_id_str) = type_id {
                let type_id = IId(type_id_str.parse()?);
                inc = inc.with_type(type_id);
            }

            if let Some(value_str) = value {
                inc = inc.with_val(Value::string(value_str));
            }

            g.add_incidence(inc);
            println!("Added incidence: {}", id.0);
        }
        Commands::Get { id } => {
            let g = graph.lock().unwrap();
            let id = IId(id.parse()?);
            if let Some(inc) = g.get(id) {
                println!("Incidence: {:?}", inc);
            } else {
                println!("Incidence not found: {}", id.0);
            }
        }
        Commands::Query { query } => {
            let g = graph.lock().unwrap();
            println!("Query: {}", query);
            println!("Results: {} incidences", g.len());
            // 実際の実装では Datalog またはパターンクエリを評価
        }
        Commands::Import { path } => {
            let content = std::fs::read_to_string(&path)?;
            let json: serde_json::Value = serde_json::from_str(&content)?;
            println!("Importing from: {}", path);
            // 実際の実装では JSON をパースして WorldGraph に追加
        }
        Commands::Export { path } => {
            let g = graph.lock().unwrap();
            let json = InternalJsonConverter::from_world_graph(&g);
            let content = serde_json::to_string_pretty(&json)?;
            std::fs::write(&path, content)?;
            println!("Exported to: {}", path);
        }
        Commands::Serve { port } => {
            println!("Starting server on port {}", port);
            // 実際の実装では gRPC または GraphQL サーバーを起動
            println!("Server started (not implemented yet)");
        }
    }

    Ok(())
}
