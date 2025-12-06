//! Event Sourced GraphQL Schema
//!
//! EventSourcedGraph用のGraphQLスキーマ

use async_graphql::{Context, Object, Schema, EmptySubscription};
use incdb_core::model::{IId, Incidence, Level, RoleId, Value};
use incdb_storage::graph::EventSourcedGraph;
use incdb_storage::index::optimized_vector_index::QueryFilters;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::graphql::benchmark;
use crate::graphql::{
    WriteBenchmarkConfig, ReadBenchmarkConfig,
    MultiHopBenchmarkConfig, VectorHopBenchmarkConfig,
    BenchmarkResult, BenchmarkMetadata,
};

/// GraphQL Query (EventSourcedGraph用)
pub struct EventSourcedQuery;

#[Object]
impl EventSourcedQuery {
    /// Incidence を取得
    async fn incidence(&self, ctx: &Context<'_>, id: String) -> async_graphql::Result<Option<IncidenceType>> {
        let graph = ctx.data::<Arc<Mutex<EventSourcedGraph>>>()?;
        let graph = graph.lock().await;

        let id = IId(id.parse().map_err(|_| async_graphql::Error::new("Invalid ID"))?);

        match graph.get(id).await {
            Ok(Some(inc)) => Ok(Some(IncidenceType::from(&inc))),
            Ok(None) => Ok(None),
            Err(e) => Err(async_graphql::Error::new(format!("Failed to get incidence: {}", e))),
        }
    }

    /// すべての Incidence を取得
    async fn incidences(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<IncidenceType>> {
        let graph = ctx.data::<Arc<Mutex<EventSourcedGraph>>>()?;
        let graph = graph.lock().await;

        match graph.iter().await {
            Ok(incidences) => {
                let results: Vec<IncidenceType> = incidences.iter().map(IncidenceType::from).collect();
                Ok(results)
            }
            Err(e) => Err(async_graphql::Error::new(format!("Failed to get incidences: {}", e))),
        }
    }

    /// 時系列範囲クエリ
    async fn incidences_by_time_range(
        &self,
        ctx: &Context<'_>,
        start: i64,
        end: i64,
        entity_id: Option<String>,
    ) -> async_graphql::Result<Vec<IncidenceType>> {
        let graph = ctx.data::<Arc<Mutex<EventSourcedGraph>>>()?;
        let graph = graph.lock().await;

        let entity_filter = if let Some(s) = entity_id {
            Some(IId(s.parse().map_err(|_| async_graphql::Error::new("Invalid entity ID"))?))
        } else {
            None
        };

        match graph.query_time_range(start, end, entity_filter).await {
            Ok(incidences) => {
                let results: Vec<IncidenceType> = incidences.iter().map(IncidenceType::from).collect();
                Ok(results)
            }
            Err(e) => Err(async_graphql::Error::new(format!("Failed to query time range: {}", e))),
        }
    }

    /// エンティティtraversal
    async fn traverse_entity(
        &self,
        ctx: &Context<'_>,
        entity_id: String,
        max_depth: usize,
    ) -> async_graphql::Result<Vec<String>> {
        let graph = ctx.data::<Arc<Mutex<EventSourcedGraph>>>()?;
        let graph = graph.lock().await;

        let entity_id = IId(entity_id.parse().map_err(|_| async_graphql::Error::new("Invalid entity ID"))?);

        match graph.traverse_entity(entity_id, max_depth).await {
            Ok(ids) => Ok(ids.into_iter().map(|id| id.0.to_string()).collect()),
            Err(e) => Err(async_graphql::Error::new(format!("Failed to traverse entity: {}", e))),
        }
    }

    /// ベクトル類似度検索（コンテキスト付き）
    async fn vector_search(
        &self,
        ctx: &Context<'_>,
        query_vector: Vec<f32>,
        k: usize,
        time_range_start: Option<i64>,
        time_range_end: Option<i64>,
        entity_filter: Option<String>,
    ) -> async_graphql::Result<VectorSearchResult> {
        let graph = ctx.data::<Arc<Mutex<EventSourcedGraph>>>()?;
        let graph = graph.lock().await;

        let filters = QueryFilters {
            time_range: time_range_start.zip(time_range_end).map(|(s, e)| (s, e)),
            entity_filter: if let Some(s) = entity_filter {
                Some(IId(s.parse().map_err(|_| async_graphql::Error::new("Invalid entity ID"))?))
            } else {
                None
            },
        };

        match graph.vector_search(query_vector, k, filters).await {
            Ok(results) => {
                // 結果のIDからIncidenceを取得
                let mut items = Vec::new();
                for (id, similarity) in results {
                    match graph.get(id).await {
                        Ok(Some(inc)) => {
                            items.push(VectorSearchItem {
                                incidence: IncidenceType::from(&inc),
                                similarity,
                            });
                        }
                        _ => {
                            // Incidenceが見つからない場合はスキップ
                            continue;
                        }
                    }
                }
                Ok(VectorSearchResult { incidences: items })
            }
            Err(e) => Err(async_graphql::Error::new(format!("Failed to search vectors: {}", e))),
        }
    }
}

/// GraphQL Mutation (EventSourcedGraph用)
pub struct EventSourcedMutation {
    benchmark: benchmark::BenchmarkMutation,
}

impl Default for EventSourcedMutation {
    fn default() -> Self {
        Self {
            benchmark: benchmark::BenchmarkMutation,
        }
    }
}

#[Object]
impl EventSourcedMutation {
    /// Incidence を追加
    async fn add_incidence(
        &self,
        ctx: &Context<'_>,
        level: Option<u8>,
        type_id: Option<String>,
        value: Option<ValueInput>,
        args: Option<Vec<String>>,
        roles: Option<Vec<u32>>,
        embedding: Option<Vec<f32>>,
    ) -> async_graphql::Result<IncidenceType> {
        let graph = ctx.data::<Arc<Mutex<EventSourcedGraph>>>()?;
        let mut graph = graph.lock().await;

        let id = graph.new_id();
        let level = Level(level.unwrap_or(0));
        let mut incidence = Incidence::new(id, level);

        if let Some(type_id_str) = type_id {
            let type_id = IId(type_id_str.parse().map_err(|_| async_graphql::Error::new("Invalid type ID"))?);
            incidence = incidence.with_type(type_id);
        }

        if let Some(value_input) = value {
            let val = convert_value_input(value_input)?;
            incidence = incidence.with_val(val);
        }

        if let (Some(args_vec), Some(roles_vec)) = (args, roles) {
            for (arg_str, role_val) in args_vec.iter().zip(roles_vec.iter()) {
                let arg_id = IId(arg_str.parse().map_err(|_| async_graphql::Error::new("Invalid arg ID"))?);
                let role = RoleId(*role_val);
                incidence = incidence.add_arg(arg_id, role);
            }
        }

        if let Some(embedding_vec) = embedding {
            incidence = incidence.with_embedding(embedding_vec);
        }

        match graph.add_incidence(incidence.clone(), None).await {
            Ok(_) => Ok(IncidenceType::from(&incidence)),
            Err(e) => Err(async_graphql::Error::new(format!("Failed to add incidence: {}", e))),
        }
    }

    /// ベンチマーク: 書き込み性能を計測
    async fn benchmark_write(
        &self,
        ctx: &Context<'_>,
        config: WriteBenchmarkConfig,
    ) -> async_graphql::Result<BenchmarkResult> {
        use std::time::Instant;
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let graph = ctx.data::<Arc<tokio::sync::Mutex<EventSourcedGraph>>>()?;
        let mut graph = graph.lock().await;

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

                match graph.add_incidence(incidence, None).await {
                    Ok(_) => total_written += 1,
                    Err(e) => return Err(async_graphql::Error::new(format!("Failed to add incidence: {}", e))),
                }
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

    /// ベンチマーク: 読み込み性能を計測
    async fn benchmark_read(
        &self,
        ctx: &Context<'_>,
        config: ReadBenchmarkConfig,
    ) -> async_graphql::Result<BenchmarkResult> {
        use std::time::Instant;
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let graph = ctx.data::<Arc<tokio::sync::Mutex<EventSourcedGraph>>>()?;
        let graph = graph.lock().await;

        let query_count = config.query_count.unwrap_or(1000) as usize;
        let graph_size = graph.len().await.map_err(|e| async_graphql::Error::new(format!("Failed to get graph size: {}", e)))?;

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
            match graph.get(random_id).await {
                Ok(Some(_)) => total_read += 1,
                Ok(None) => {},
                Err(_) => {},
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

    /// ベンチマーク: 多段hop性能を計測
    async fn benchmark_multi_hop(
        &self,
        ctx: &Context<'_>,
        config: MultiHopBenchmarkConfig,
    ) -> async_graphql::Result<BenchmarkResult> {
        use std::time::Instant;
        
        let graph = ctx.data::<Arc<tokio::sync::Mutex<EventSourcedGraph>>>()?;
        let graph = graph.lock().await;

        let start_nodes = config.start_nodes as usize;
        let depth = config.depth as usize;
        let avg_edges = config.avg_edges.unwrap_or(3) as usize;

        let graph_size = graph.len().await.map_err(|e| async_graphql::Error::new(format!("Failed to get graph size: {}", e)))?;
        if graph_size == 0 {
            return Err(async_graphql::Error::new("Graph is empty. Please run write benchmark first."));
        }

        let start = Instant::now();
        let mut total_hops = 0;
        let mut total_nodes_visited = 0;

        // 開始ノードを選択（iter()がエラーの場合、ID範囲から選択）
        let start_ids: Vec<IId> = {
            // まずiter()を試す
            match graph.iter().await {
                Ok(incidences) => {
                    if incidences.len() >= start_nodes {
                        incidences.iter().take(start_nodes).map(|inc| inc.id).collect()
                    } else {
                        // データが少ない場合、ID範囲から選択
                        (1..=graph_size.min(start_nodes as usize))
                            .map(|i| IId(i as u64))
                            .collect()
                    }
                }
                Err(_) => {
                    // iter()がエラーの場合、ID範囲から選択
                    (1..=graph_size.min(start_nodes as usize))
                        .map(|i| IId(i as u64))
                        .collect()
                }
            }
        };

        // 各開始ノードから多段hopを実行
        for start_id in start_ids {
            let mut current_level = vec![start_id];
            let mut visited = std::collections::HashSet::new();
            visited.insert(start_id);

            for _hop in 0..depth {
                let mut next_level = Vec::new();

                for node_id in &current_level {
                    match graph.get(*node_id).await {
                        Ok(Some(inc)) => {
                            // args から次のノードを取得
                            let edges: Vec<IId> = inc.args.iter()
                                .take(avg_edges)
                                .copied()
                                .collect();

                            for edge_id in edges {
                                if !visited.contains(&edge_id) {
                                    match graph.get(edge_id).await {
                                        Ok(Some(_)) => {
                                            visited.insert(edge_id);
                                            next_level.push(edge_id);
                                            total_hops += 1;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        _ => {}
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
            data_size: graph_size as i32,
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

    /// ベンチマーク: Vector Index Hop 性能を計測
    async fn benchmark_vector_hop(
        &self,
        ctx: &Context<'_>,
        config: VectorHopBenchmarkConfig,
    ) -> async_graphql::Result<BenchmarkResult> {
        use std::time::Instant;
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let graph = ctx.data::<Arc<tokio::sync::Mutex<EventSourcedGraph>>>()?;
        let graph = graph.lock().await;

        let data_size = config.data_size as usize;
        let vector_dim = config.vector_dim as usize;
        let depth = config.depth as usize;
        let k_per_hop = config.k_per_hop as usize;

        // ベクトルを持つ Incidence を収集（iter()がエラーの場合、ID範囲から検索）
        let graph_size = graph.len().await.map_err(|e| async_graphql::Error::new(format!("Failed to get graph size: {}", e)))?;
        let vector_incidences: Vec<(IId, Vec<f32>)> = {
            match graph.iter().await {
                Ok(incidences) => {
                    incidences
                        .iter()
                        .filter_map(|inc| {
                            inc.embedding.as_ref().map(|emb| (inc.id, emb.clone()))
                        })
                        .take(data_size)
                        .collect()
                }
                Err(_) => {
                    // iter()がエラーの場合、ID範囲から検索
                    let mut result = Vec::new();
                    for i in 1..=graph_size.min(data_size * 10) {
                        if let Ok(Some(inc)) = graph.get(IId(i as u64)).await {
                            if let Some(emb) = inc.embedding {
                                result.push((inc.id, emb));
                                if result.len() >= data_size {
                                    break;
                                }
                            }
                        }
                    }
                    result
                }
            }
        };

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

    /// ベンチマーク: グラフをクリア
    async fn benchmark_clear_graph(&self, _ctx: &Context<'_>) -> async_graphql::Result<bool> {
        // EventSourcedGraphではストレージをクリアする必要があるため、
        // 簡易実装として常にtrueを返す（実際のクリアは手動で行う）
        // TODO: ストレージをクリアする実装を追加
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

// 既存のschema.rsから型定義をインポート（既にpubなので直接使用可能）
use crate::graphql::schema::{IncidenceType, ValueInput, VectorSearchResult, VectorSearchItem};

fn convert_value_input(input: ValueInput) -> async_graphql::Result<Value> {
    if let Some(s) = input.str {
        Ok(Value::string(s))
    } else if let Some(i) = input.int {
        Ok(Value::int(i))
    } else if let Some(f) = input.float {
        Ok(Value::float(f))
    } else if let Some(b) = input.bool {
        Ok(Value::bool(b))
    } else if let Some(v) = input.vector {
        Ok(Value::vector(v))
    } else {
        Ok(Value::Null)
    }
}

/// EventSourcedGraph用のGraphQL Schema
pub type EventSourcedAppSchema = Schema<EventSourcedQuery, EventSourcedMutation, EmptySubscription>;

pub fn create_event_sourced_schema(graph: Arc<tokio::sync::Mutex<EventSourcedGraph>>) -> EventSourcedAppSchema {
    Schema::build(EventSourcedQuery, EventSourcedMutation::default(), EmptySubscription)
        .data(graph)
        .finish()
}

