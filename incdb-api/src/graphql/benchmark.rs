//! Benchmark Operations
//!
//! 性能計測用の GraphQL エンドポイント

use async_graphql::{Context, Object, SimpleObject, InputObject};
use incdb_core::model::{IId, Incidence, Level, RoleId, WorldGraph};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// 書き込みベンチマーク設定
#[derive(InputObject, Clone)]
pub struct WriteBenchmarkConfig {
    /// データサイズ（100, 1000, 10000, 100000, 1000000）
    pub data_size: i32,
    /// バッチサイズ（一度に書き込む数）
    pub batch_size: Option<i32>,
    /// ベクトル次元数（0 の場合はベクトルなし）
    pub vector_dim: Option<i32>,
    /// 各 Incidence の args 数
    pub args_per_incidence: Option<i32>,
}

/// 読み込みベンチマーク設定
#[derive(InputObject, Clone)]
pub struct ReadBenchmarkConfig {
    /// データサイズ
    pub data_size: i32,
    /// クエリ数
    pub query_count: Option<i32>,
}

/// 多段hopベンチマーク設定
#[derive(InputObject, Clone)]
pub struct MultiHopBenchmarkConfig {
    /// 開始ノード数
    pub start_nodes: i32,
    /// hop の深さ
    pub depth: i32,
    /// 各ノードの平均エッジ数
    pub avg_edges: Option<i32>,
}

/// Vector Index Hop ベンチマーク設定
#[derive(InputObject, Clone)]
pub struct VectorHopBenchmarkConfig {
    /// データサイズ
    pub data_size: i32,
    /// ベクトル次元数
    pub vector_dim: i32,
    /// hop の深さ
    pub depth: i32,
    /// 各 hop での検索数（k）
    pub k_per_hop: i32,
}

/// ベンチマーク結果
#[derive(SimpleObject, Clone)]
pub struct BenchmarkResult {
    /// 操作名
    pub operation: String,
    /// データサイズ
    pub data_size: i32,
    /// 実行時間（ミリ秒）
    pub duration_ms: f64,
    /// スループット（ops/sec）
    pub throughput: f64,
    /// レイテンシ（ミリ秒）
    pub latency_ms: f64,
    /// 追加情報
    pub metadata: Vec<BenchmarkMetadata>,
}

/// ベンチマークメタデータ
#[derive(SimpleObject, Clone)]
pub struct BenchmarkMetadata {
    pub key: String,
    pub value: String,
}


/// ベンチマーク Mutation
pub struct BenchmarkMutation;

#[Object]
impl BenchmarkMutation {
    /// 書き込み性能を計測
    pub async fn benchmark_write(
        &self,
        ctx: &Context<'_>,
        config: WriteBenchmarkConfig,
    ) -> async_graphql::Result<BenchmarkResult> {
        let graph = ctx.data::<Arc<Mutex<WorldGraph>>>()?;
        let mut graph = graph.lock().map_err(|e| async_graphql::Error::new(format!("Failed to lock graph: {}", e)))?;

        let data_size = config.data_size as usize;
        let batch_size = config.batch_size.unwrap_or(100) as usize;
        let vector_dim = config.vector_dim.unwrap_or(0) as usize;
        let args_per_incidence = config.args_per_incidence.unwrap_or(2) as usize;

        let start = Instant::now();
        let mut total_written = 0;

        // バッチごとに書き込み
        for batch_start in (0..data_size).step_by(batch_size) {
            let batch_end = (batch_start + batch_size).min(data_size);
            
            for i in batch_start..batch_end {
                let id = graph.new_id();
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

                graph.add_incidence(incidence);
                total_written += 1;
            }
        }

        let duration = start.elapsed();
        let duration_ms = duration.as_secs_f64() * 1000.0;
        let throughput = (total_written as f64) / duration.as_secs_f64();
        let latency_ms = duration_ms / total_written as f64;

        Ok(BenchmarkResult {
            operation: "write".to_string(),
            data_size: total_written as i32,
            duration_ms,
            throughput,
            latency_ms,
            metadata: vec![
                BenchmarkMetadata {
                    key: "batch_size".to_string(),
                    value: batch_size.to_string(),
                },
                BenchmarkMetadata {
                    key: "vector_dim".to_string(),
                    value: vector_dim.to_string(),
                },
                BenchmarkMetadata {
                    key: "args_per_incidence".to_string(),
                    value: args_per_incidence.to_string(),
                },
            ],
        })
    }

    /// 読み込み性能を計測
    pub async fn benchmark_read(
        &self,
        ctx: &Context<'_>,
        config: ReadBenchmarkConfig,
    ) -> async_graphql::Result<BenchmarkResult> {
        let graph = ctx.data::<Arc<Mutex<WorldGraph>>>()?;
        let graph = graph.lock().map_err(|e| async_graphql::Error::new(format!("Failed to lock graph: {}", e)))?;

        let query_count = config.query_count.unwrap_or(1000) as usize;
        let graph_size = graph.len();

        if graph_size == 0 {
            return Err(async_graphql::Error::new("Graph is empty. Please run write benchmark first."));
        }

        let start = Instant::now();
        let mut total_read = 0;

        // ランダムな ID で読み込み
        let mut hasher = DefaultHasher::new();
        for i in 0..query_count {
            i.hash(&mut hasher);
            let hash = hasher.finish();
            let random_id = IId((hash % graph_size as u64) + 1);
            if graph.get(random_id).is_some() {
                total_read += 1;
            }
        }

        let duration = start.elapsed();
        let duration_ms = duration.as_secs_f64() * 1000.0;
        let throughput = (total_read as f64) / duration.as_secs_f64();
        let latency_ms = duration_ms / total_read as f64;

        Ok(BenchmarkResult {
            operation: "read".to_string(),
            data_size: graph_size as i32,
            duration_ms,
            throughput,
            latency_ms,
            metadata: vec![
                BenchmarkMetadata {
                    key: "query_count".to_string(),
                    value: query_count.to_string(),
                },
                BenchmarkMetadata {
                    key: "successful_reads".to_string(),
                    value: total_read.to_string(),
                },
            ],
        })
    }

    /// 多段hop性能を計測
    pub async fn benchmark_multi_hop(
        &self,
        ctx: &Context<'_>,
        config: MultiHopBenchmarkConfig,
    ) -> async_graphql::Result<BenchmarkResult> {
        let graph = ctx.data::<Arc<Mutex<WorldGraph>>>()?;
        let graph = graph.lock().map_err(|e| async_graphql::Error::new(format!("Failed to lock graph: {}", e)))?;

        let start_nodes = config.start_nodes as usize;
        let depth = config.depth as usize;
        let avg_edges = config.avg_edges.unwrap_or(3) as usize;

        if graph.len() == 0 {
            return Err(async_graphql::Error::new("Graph is empty. Please run write benchmark first."));
        }

        let start = Instant::now();
        let mut total_hops = 0;
        let mut total_nodes_visited = 0;

        // 開始ノードを選択
        let start_ids: Vec<IId> = graph.iter()
            .take(start_nodes)
            .map(|inc| inc.id)
            .collect();

        // 各開始ノードから多段hopを実行
        for start_id in start_ids {
            let mut current_level = vec![start_id];
            let mut visited = std::collections::HashSet::new();
            visited.insert(start_id);

            for _hop in 0..depth {
                let mut next_level = Vec::new();

                for node_id in &current_level {
                    if let Some(inc) = graph.get(*node_id) {
                        // args から次のノードを取得
                        let edges: Vec<IId> = inc.args.iter()
                            .take(avg_edges)
                            .copied()
                            .collect();

                        for edge_id in edges {
                            if !visited.contains(&edge_id) && graph.get(edge_id).is_some() {
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
        }

        let duration = start.elapsed();
        let duration_ms = duration.as_secs_f64() * 1000.0;
        let throughput = (total_hops as f64) / duration.as_secs_f64();
        let latency_ms = if total_hops > 0 {
            duration_ms / total_hops as f64
        } else {
            0.0
        };

        Ok(BenchmarkResult {
            operation: "multi_hop".to_string(),
            data_size: graph.len() as i32,
            duration_ms,
            throughput,
            latency_ms,
            metadata: vec![
                BenchmarkMetadata {
                    key: "start_nodes".to_string(),
                    value: start_nodes.to_string(),
                },
                BenchmarkMetadata {
                    key: "depth".to_string(),
                    value: depth.to_string(),
                },
                BenchmarkMetadata {
                    key: "total_hops".to_string(),
                    value: total_hops.to_string(),
                },
                BenchmarkMetadata {
                    key: "total_nodes_visited".to_string(),
                    value: total_nodes_visited.to_string(),
                },
            ],
        })
    }

    /// Vector Index Hop 性能を計測
    pub async fn benchmark_vector_hop(
        &self,
        ctx: &Context<'_>,
        config: VectorHopBenchmarkConfig,
    ) -> async_graphql::Result<BenchmarkResult> {
        let graph = ctx.data::<Arc<Mutex<WorldGraph>>>()?;
        let graph = graph.lock().map_err(|e| async_graphql::Error::new(format!("Failed to lock graph: {}", e)))?;

        let data_size = config.data_size as usize;
        let vector_dim = config.vector_dim as usize;
        let depth = config.depth as usize;
        let k_per_hop = config.k_per_hop as usize;

        // ベクトルを持つ Incidence を収集
        let vector_incidences: Vec<(IId, Vec<f32>)> = graph.iter()
            .filter_map(|inc| {
                inc.embedding.as_ref().map(|emb| (inc.id, emb.clone()))
            })
            .take(data_size)
            .collect();

        if vector_incidences.is_empty() {
            return Err(async_graphql::Error::new("No vectors found. Please run write benchmark with vector_dim > 0 first."));
        }

        let start = Instant::now();
        let mut total_searches = 0;
        let mut total_hops = 0;

        // 最初のクエリベクトル（ランダム）
        let mut hasher = DefaultHasher::new();
        "benchmark_query".hash(&mut hasher);
        let seed = hasher.finish();
        let mut query_vector: Vec<f32> = (0..vector_dim)
            .map(|i| {
                let mut h = DefaultHasher::new();
                (seed + i as u64).hash(&mut h);
                let hash = h.finish();
                // 0-1 の範囲に正規化
                (hash % 10000) as f32 / 10000.0
            })
            .collect();

        // 正規化
        let norm: f32 = query_vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            query_vector.iter_mut().for_each(|x| *x /= norm);
        }

        // 各 hop でベクトル検索を実行
        for _hop in 0..depth {
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

            // 次の hop のクエリベクトルを更新（上位 k の平均）
            if _hop < depth - 1 && !top_k.is_empty() {
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
        let throughput = (total_searches as f64) / duration.as_secs_f64();
        let latency_ms = if total_searches > 0 {
            duration_ms / total_searches as f64
        } else {
            0.0
        };

        Ok(BenchmarkResult {
            operation: "vector_hop".to_string(),
            data_size: vector_incidences.len() as i32,
            duration_ms,
            throughput,
            latency_ms,
            metadata: vec![
                BenchmarkMetadata {
                    key: "vector_dim".to_string(),
                    value: vector_dim.to_string(),
                },
                BenchmarkMetadata {
                    key: "depth".to_string(),
                    value: depth.to_string(),
                },
                BenchmarkMetadata {
                    key: "k_per_hop".to_string(),
                    value: k_per_hop.to_string(),
                },
                BenchmarkMetadata {
                    key: "total_searches".to_string(),
                    value: total_searches.to_string(),
                },
                BenchmarkMetadata {
                    key: "total_hops".to_string(),
                    value: total_hops.to_string(),
                },
            ],
        })
    }

    /// グラフをクリア（ベンチマーク前の準備）
    pub async fn clear_graph(&self, ctx: &Context<'_>) -> async_graphql::Result<bool> {
        let graph = ctx.data::<Arc<Mutex<WorldGraph>>>()?;
        let mut graph = graph.lock().map_err(|e| async_graphql::Error::new(format!("Failed to lock graph: {}", e)))?;
        
        // WorldGraph をリセット（簡易実装）
        // 実際の実装では、新しい WorldGraph を作成して置き換える
        *graph = WorldGraph::new();
        
        Ok(true)
    }
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

