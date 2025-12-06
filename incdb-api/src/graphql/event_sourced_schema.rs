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
    BenchmarkResult,
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
        // TODO: EventSourcedGraph用のベンチマーク実装
        Err(async_graphql::Error::new("Benchmark not yet implemented for EventSourcedGraph"))
    }

    /// ベンチマーク: 読み込み性能を計測
    async fn benchmark_read(
        &self,
        ctx: &Context<'_>,
        config: ReadBenchmarkConfig,
    ) -> async_graphql::Result<BenchmarkResult> {
        // TODO: EventSourcedGraph用のベンチマーク実装
        Err(async_graphql::Error::new("Benchmark not yet implemented for EventSourcedGraph"))
    }

    /// ベンチマーク: 多段hop性能を計測
    async fn benchmark_multi_hop(
        &self,
        ctx: &Context<'_>,
        config: MultiHopBenchmarkConfig,
    ) -> async_graphql::Result<BenchmarkResult> {
        // TODO: EventSourcedGraph用のベンチマーク実装
        Err(async_graphql::Error::new("Benchmark not yet implemented for EventSourcedGraph"))
    }

    /// ベンチマーク: Vector Index Hop 性能を計測
    async fn benchmark_vector_hop(
        &self,
        ctx: &Context<'_>,
        config: VectorHopBenchmarkConfig,
    ) -> async_graphql::Result<BenchmarkResult> {
        // TODO: EventSourcedGraph用のベンチマーク実装
        Err(async_graphql::Error::new("Benchmark not yet implemented for EventSourcedGraph"))
    }

    /// ベンチマーク: グラフをクリア
    async fn benchmark_clear_graph(&self, _ctx: &Context<'_>) -> async_graphql::Result<bool> {
        // TODO: EventSourcedGraph用のクリア実装
        Err(async_graphql::Error::new("Clear graph not yet implemented for EventSourcedGraph"))
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

