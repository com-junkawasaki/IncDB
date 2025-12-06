//! IncDB CLI Tool

use clap::{Parser, Subcommand};
use incdb_core::model::{IId, Incidence, Level, RoleId, Value, WorldGraph};
use incdb_core::ir::InternalJsonConverter;
use incdb_query::datalog::{DatalogProgram, Predicate};
use std::sync::{Arc, Mutex};
use std::time::Instant;

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
    /// ベンチマーク: 書き込み性能を計測
    BenchmarkWrite {
        /// データサイズ
        #[arg(short, long, default_value = "1000")]
        data_size: usize,
        /// バッチサイズ
        #[arg(short, long, default_value = "100")]
        batch_size: usize,
        /// ベクトル次元数（0 の場合はベクトルなし）
        #[arg(short, long, default_value = "0")]
        vector_dim: usize,
        /// 各 Incidence の args 数
        #[arg(short, long, default_value = "2")]
        args_per_incidence: usize,
    },
    /// ベンチマーク: 読み込み性能を計測
    BenchmarkRead {
        /// クエリ数
        #[arg(short, long, default_value = "1000")]
        query_count: usize,
    },
    /// ベンチマーク: 多段hop性能を計測
    BenchmarkMultiHop {
        /// 開始ノード数
        #[arg(short, long, default_value = "10")]
        start_nodes: usize,
        /// hop の深さ
        #[arg(short, long, default_value = "3")]
        depth: usize,
        /// 各ノードの平均エッジ数
        #[arg(short, long, default_value = "3")]
        avg_edges: usize,
    },
    /// ベンチマーク: Vector Index Hop 性能を計測
    BenchmarkVectorHop {
        /// データサイズ
        #[arg(long, default_value = "1000")]
        data_size: usize,
        /// ベクトル次元数
        #[arg(short = 'v', long, default_value = "128")]
        vector_dim: usize,
        /// hop の深さ
        #[arg(short = 'd', long, default_value = "3")]
        depth: usize,
        /// 各 hop での検索数（k）
        #[arg(short = 'k', long, default_value = "10")]
        k_per_hop: usize,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
            let start = Instant::now();
            
            // Datalog クエリを解析
            let mut program = DatalogProgram::new();
            for line in query.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with("//") {
                    continue;
                }
                
                // 簡易パーサー: "Inc(1)" 形式を解析
                if let Some(predicate) = parse_predicate(line) {
                    program.add_fact(predicate);
                }
            }
            
            // クエリを評価
            match program.evaluate(&g) {
                Ok(results) => {
                    let duration = start.elapsed();
                    println!("Query: {}", query);
                    println!("Results: {} predicates", results.len());
                    println!("Duration: {:.2}ms", duration.as_secs_f64() * 1000.0);
                    
                    // 結果を表示
                    for (i, pred) in results.iter().take(10).enumerate() {
                        println!("  [{}] {:?}", i + 1, pred);
                    }
                    if results.len() > 10 {
                        println!("  ... and {} more", results.len() - 10);
                    }
                }
                Err(e) => {
                    eprintln!("Query error: {}", e);
                }
            }
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
        Commands::BenchmarkWrite {
            data_size,
            batch_size,
            vector_dim,
            args_per_incidence,
        } => {
            let mut g = graph.lock().unwrap();
            let start = Instant::now();
            let mut total_written = 0;

            println!("Starting write benchmark:");
            println!("  Data size: {}", data_size);
            println!("  Batch size: {}", batch_size);
            println!("  Vector dimension: {}", vector_dim);
            println!("  Args per incidence: {}", args_per_incidence);
            println!();

            // バッチごとに書き込み
            for batch_start in (0..data_size).step_by(batch_size) {
                let batch_end = (batch_start + batch_size).min(data_size);
                
                for i in batch_start..batch_end {
                    let id = g.new_id();
                    let mut incidence = Incidence::new(id, Level::zero());

                    // ベクトルを追加
                    if vector_dim > 0 {
                        let embedding: Vec<f32> = (0..vector_dim)
                            .map(|j| ((i + j) as f32 * 0.001).sin())
                            .collect();
                        incidence = incidence.with_embedding(embedding);
                    }

                    // args を追加（前のノードへのリンク）
                    if args_per_incidence > 0 && i > 0 {
                        for j in 0..args_per_incidence.min(i) {
                            let arg_id = IId((i - j) as u64);
                            let role = RoleId((j % 10) as u32);
                            incidence = incidence.add_arg(arg_id, role);
                        }
                    }

                    g.add_incidence(incidence);
                    total_written += 1;
                }
                
                // 進捗表示
                if (batch_start / batch_size) % 10 == 0 {
                    let progress = (batch_start as f64 / data_size as f64) * 100.0;
                    println!("  Progress: {:.1}% ({}/{})", progress, batch_start, data_size);
                }
            }

            let duration = start.elapsed();
            let duration_ms = duration.as_secs_f64() * 1000.0;
            let throughput = total_written as f64 / duration.as_secs_f64();
            let latency_ms = duration_ms / total_written as f64;

            println!();
            println!("Write Benchmark Results:");
            println!("  Total written: {}", total_written);
            println!("  Duration: {:.2}ms", duration_ms);
            println!("  Throughput: {:.2} ops/sec", throughput);
            println!("  Latency: {:.4}ms", latency_ms);
        }
        Commands::BenchmarkRead { query_count } => {
            let mut g = graph.lock().unwrap();
            let graph_size = g.len();

            // グラフが空の場合は、テストデータを生成
            if graph_size == 0 {
                println!("Graph is empty. Generating test data for read benchmark...");
                let test_size = query_count.max(1000);
                for i in 0..test_size {
                    let id = g.new_id();
                    let mut incidence = Incidence::new(id, Level::zero());
                    incidence = incidence.with_val(Value::string(format!("Test_{}", i)));
                    g.add_incidence(incidence);
                }
                println!("Generated {} test incidences", test_size);
            }
            let graph_size = g.len(); // 再取得

            let start = Instant::now();
            let mut total_read = 0;

            println!("Starting read benchmark:");
            println!("  Query count: {}", query_count);
            println!("  Graph size: {}", graph_size);
            println!();

            // ランダムな ID で読み込み
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            
            // 利用可能なIDを収集
            let available_ids: Vec<IId> = g.iter().map(|inc| inc.id).collect();
            let graph_size = available_ids.len();
            
            if graph_size == 0 {
                eprintln!("Error: No incidences available for read benchmark.");
                return Ok(());
            }
            
            let mut hasher = DefaultHasher::new();
            
            for i in 0..query_count {
                i.hash(&mut hasher);
                let hash = hasher.finish();
                let idx = (hash % graph_size as u64) as usize;
                if let Some(&id) = available_ids.get(idx) {
                    if g.get(id).is_some() {
                        total_read += 1;
                    }
                }
                
                // 進捗表示
                if i > 0 && i % (query_count / 10) == 0 {
                    let progress = (i as f64 / query_count as f64) * 100.0;
                    println!("  Progress: {:.1}% ({}/{})", progress, i, query_count);
                }
            }

            let duration = start.elapsed();
            let duration_ms = duration.as_secs_f64() * 1000.0;
            let throughput = total_read as f64 / duration.as_secs_f64();
            let latency_ms = duration_ms / total_read as f64;

            println!();
            println!("Read Benchmark Results:");
            println!("  Successful reads: {}", total_read);
            println!("  Duration: {:.2}ms", duration_ms);
            println!("  Throughput: {:.2} ops/sec", throughput);
            println!("  Latency: {:.4}ms", latency_ms);
        }
        Commands::BenchmarkMultiHop {
            start_nodes,
            depth,
            avg_edges,
        } => {
            let mut g = graph.lock().unwrap();
            let graph_size = g.len();

            // グラフが空の場合は、テストデータを生成
            if graph_size == 0 {
                println!("Graph is empty. Generating test data for multi-hop benchmark...");
                let test_size = start_nodes * avg_edges * depth;
                for i in 0..test_size {
                    let id = g.new_id();
                    let mut incidence = Incidence::new(id, Level::zero());
                    
                    // 前のノードへのリンクを作成
                    if i > 0 {
                        for j in 0..avg_edges.min(i) {
                            let arg_id = IId((i - j) as u64);
                            let role = RoleId((j % 10) as u32);
                            incidence = incidence.add_arg(arg_id, role);
                        }
                    }
                    
                    g.add_incidence(incidence);
                }
                println!("Generated {} test incidences", test_size);
            }
            let g = g; // 再借用

            let start = Instant::now();
            let mut total_hops = 0;
            let mut total_nodes_visited = 0;

            println!("Starting multi-hop benchmark:");
            println!("  Start nodes: {}", start_nodes);
            println!("  Depth: {}", depth);
            println!("  Avg edges: {}", avg_edges);
            println!();

            // 開始ノードを選択
            let start_ids: Vec<IId> = g.iter()
                .take(start_nodes)
                .map(|inc| inc.id)
                .collect();

            // 各開始ノードから多段hopを実行
            for (idx, start_id) in start_ids.iter().enumerate() {
                let mut current_level = vec![*start_id];
                let mut visited = std::collections::HashSet::new();
                visited.insert(*start_id);

                for hop in 0..depth {
                    let mut next_level = Vec::new();

                    for node_id in &current_level {
                        if let Some(inc) = g.get(*node_id) {
                            // args から次のノードを取得
                            let edges: Vec<IId> = inc.args.iter()
                                .take(avg_edges)
                                .copied()
                                .collect();

                            for edge_id in edges {
                                if !visited.contains(&edge_id) && g.get(edge_id).is_some() {
                                    visited.insert(edge_id);
                                    next_level.push(edge_id);
                                    total_hops += 1;
                                }
                            }
                        }
                    }

                    if next_level.is_empty() {
                        break;
                    }

                    current_level = next_level;
                }

                total_nodes_visited += visited.len();
                
                // 進捗表示
                if (idx + 1) % (start_nodes.max(1) / 10).max(1) == 0 {
                    let progress = ((idx + 1) as f64 / start_nodes as f64) * 100.0;
                    println!("  Progress: {:.1}% ({}/{})", progress, idx + 1, start_nodes);
                }
            }

            let duration = start.elapsed();
            let duration_ms = duration.as_secs_f64() * 1000.0;
            let throughput = if duration.as_secs_f64() > 0.0 {
                total_hops as f64 / duration.as_secs_f64()
            } else {
                0.0
            };
            let latency_ms = if total_hops > 0 {
                duration_ms / total_hops as f64
            } else {
                0.0
            };

            println!();
            println!("Multi-Hop Benchmark Results:");
            println!("  Total hops: {}", total_hops);
            println!("  Total nodes visited: {}", total_nodes_visited);
            println!("  Duration: {:.2}ms", duration_ms);
            println!("  Throughput: {:.2} hops/sec", throughput);
            println!("  Latency: {:.4}ms", latency_ms);
        }
        Commands::BenchmarkVectorHop {
            data_size,
            vector_dim,
            depth,
            k_per_hop,
        } => {
            let mut g = graph.lock().unwrap();
            
            // ベクトルを持つ Incidence を収集
            let vector_incidences: Vec<(IId, Vec<f32>)> = g.iter()
                .filter_map(|inc| {
                    inc.embedding.as_ref().map(|emb| (inc.id, emb.clone()))
                })
                .take(data_size)
                .collect();

            // ベクトルが見つからない場合は、テストデータを生成
            if vector_incidences.is_empty() {
                println!("No vectors found. Generating test data with vectors for vector-hop benchmark...");
                for i in 0..data_size {
                    let id = g.new_id();
                    let mut incidence = Incidence::new(id, Level::zero());
                    
                    // ベクトルを生成
                    let embedding: Vec<f32> = (0..vector_dim)
                        .map(|j| ((i + j) as f32 * 0.001).sin())
                        .collect();
                    incidence = incidence.with_embedding(embedding);
                    
                    g.add_incidence(incidence);
                }
                println!("Generated {} test incidences with vectors", data_size);
            }
            let g = g; // 再借用
            
            // ベクトルを持つ Incidence を再収集
            let vector_incidences: Vec<(IId, Vec<f32>)> = g.iter()
                .filter_map(|inc| {
                    inc.embedding.as_ref().map(|emb| (inc.id, emb.clone()))
                })
                .take(data_size)
                .collect();

            let start = Instant::now();
            let mut total_searches = 0;
            let mut total_hops = 0;

            println!("Starting vector hop benchmark:");
            println!("  Data size: {}", vector_incidences.len());
            println!("  Vector dimension: {}", vector_dim);
            println!("  Depth: {}", depth);
            println!("  K per hop: {}", k_per_hop);
            println!();

            // 最初のクエリベクトル（ランダム）
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            "benchmark_query".hash(&mut hasher);
            let seed = hasher.finish();
            let mut query_vector: Vec<f32> = (0..vector_dim)
                .map(|i| {
                    let mut h = DefaultHasher::new();
                    (seed + i as u64).hash(&mut h);
                    let hash = h.finish();
                    (hash % 10000) as f32 / 10000.0
                })
                .collect();

            // 正規化
            let norm: f32 = query_vector.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                query_vector.iter_mut().for_each(|x| *x /= norm);
            }

            // 各 hop でベクトル検索を実行
            for hop in 0..depth {
                // コサイン類似度で検索
                let mut similarities: Vec<(IId, f32)> = vector_incidences.iter()
                    .map(|(id, vec)| {
                        let similarity = cosine_similarity(&query_vector, vec);
                        (*id, similarity)
                    })
                    .collect();

                // ソートして上位 k を取得
                similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                let top_k: Vec<IId> = similarities.iter()
                    .take(k_per_hop)
                    .map(|(id, _)| *id)
                    .collect();

                total_searches += 1;
                total_hops += top_k.len();

                println!("  Hop {}: Found {} similar vectors", hop + 1, top_k.len());

                // 次の hop のクエリベクトルを更新（上位 k の平均）
                if hop < depth - 1 && !top_k.is_empty() {
                    let mut avg_vector = vec![0.0f32; vector_dim];
                    for id in &top_k {
                        if let Some((_, vec)) = vector_incidences.iter().find(|(vid, _)| vid == id) {
                            for (i, val) in vec.iter().enumerate() {
                                avg_vector[i] += val;
                            }
                        }
                    }
                    let k_f32 = top_k.len() as f32;
                    avg_vector.iter_mut().for_each(|x| *x /= k_f32);
                    query_vector = avg_vector;
                }
            }

            let duration = start.elapsed();
            let duration_ms = duration.as_secs_f64() * 1000.0;
            let throughput = if duration.as_secs_f64() > 0.0 {
                total_searches as f64 / duration.as_secs_f64()
            } else {
                0.0
            };
            let latency_ms = if total_searches > 0 {
                duration_ms / total_searches as f64
            } else {
                0.0
            };

            println!();
            println!("Vector Hop Benchmark Results:");
            println!("  Total searches: {}", total_searches);
            println!("  Total hops: {}", total_hops);
            println!("  Duration: {:.2}ms", duration_ms);
            println!("  Throughput: {:.2} searches/sec", throughput);
            println!("  Latency: {:.4}ms", latency_ms);
        }
    }

    Ok(())
}

/// Datalog 述語を解析（簡易実装）
fn parse_predicate(s: &str) -> Option<Predicate> {
    // "Inc(1)" 形式を解析
    if let Some(rest) = s.strip_prefix("Inc(") {
        if let Some(id_str) = rest.strip_suffix(')') {
            if let Ok(id) = id_str.parse::<u64>() {
                return Some(Predicate::Inc(IId(id)));
            }
        }
    }
    
    // "Type(1, 2)" 形式を解析
    if let Some(rest) = s.strip_prefix("Type(") {
        if let Some(rest) = rest.strip_suffix(')') {
            let parts: Vec<&str> = rest.split(',').map(|s| s.trim()).collect();
            if parts.len() == 2 {
                if let (Ok(inc_id), Ok(type_id)) = (parts[0].parse::<u64>(), parts[1].parse::<u64>()) {
                    return Some(Predicate::Type(IId(inc_id), IId(type_id)));
                }
            }
        }
    }

    // "Role(1, 0, 2)" 形式を解析
    if let Some(rest) = s.strip_prefix("Role(") {
        if let Some(rest) = rest.strip_suffix(')') {
            let parts: Vec<&str> = rest.split(',').map(|s| s.trim()).collect();
            if parts.len() == 3 {
                if let (Ok(inc_id), Ok(role_idx), Ok(arg_id)) = (
                    parts[0].parse::<u64>(),
                    parts[1].parse::<u32>(),
                    parts[2].parse::<u64>(),
                ) {
                    return Some(Predicate::Role(IId(inc_id), role_idx, IId(arg_id)));
                }
            }
        }
    }

    None
}

/// コサイン類似度を計算
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}
