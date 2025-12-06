//! gRPC Service Implementation

use incdb_core::model::{IId, Incidence, Level, RoleId, Value, WorldGraph};
use incdb_query::vector_query::{HybridQuery, VectorQuery};
use std::sync::{Arc, Mutex};
use tonic::{Request, Response, Status};

// プロトコルバッファの生成コード（build.rs で生成される）
mod incdb {
    tonic::include_proto!("incdb");
}

use incdb::inc_db_service_server::IncDbService;
use incdb::{
    AddIncidenceRequest, AddIncidenceResponse, GetIncidenceRequest, GetIncidenceResponse,
    QueryIncidenceRequest, QueryIncidenceResponse, VectorSearchRequest, VectorSearchResponse,
    VectorResult,
};

/// IncDB gRPC Service の実装
pub struct IncDBServiceImpl {
    /// WorldGraph（共有状態）
    graph: Arc<Mutex<WorldGraph>>,
}

impl IncDBServiceImpl {
    /// 新しいサービスを作成
    pub fn new(graph: Arc<Mutex<WorldGraph>>) -> Self {
        Self { graph }
    }
}

#[tonic::async_trait]
impl IncDbService for IncDBServiceImpl {
    async fn add_incidence(
        &self,
        request: Request<AddIncidenceRequest>,
    ) -> Result<Response<AddIncidenceResponse>, Status> {
        let req = request.into_inner();
        let mut graph = self.graph.lock().map_err(|e| {
            Status::internal(format!("Failed to lock graph: {}", e))
        })?;

        let id = if req.id.is_empty() {
            graph.new_id()
        } else {
            IId(req.id.parse().map_err(|_| Status::invalid_argument("Invalid ID"))?)
        };

        let level = Level(req.level);
        let mut incidence = Incidence::new(id, level);

        if let Some(type_id_str) = req.type_id {
            let type_id = IId(type_id_str.parse().map_err(|_| Status::invalid_argument("Invalid type ID"))?);
            incidence = incidence.with_type(type_id);
        }

        for (arg_str, role_val) in req.args.iter().zip(req.roles.iter()) {
            let arg_id = IId(arg_str.parse().map_err(|_| Status::invalid_argument("Invalid arg ID"))?);
            let role = RoleId(*role_val);
            incidence = incidence.add_arg(arg_id, role);
        }

        if let Some(value) = req.value {
            incidence = incidence.with_val(convert_value(value)?);
        }

        if !req.embedding.is_empty() {
            incidence = incidence.with_embedding(req.embedding);
        }

        graph.add_incidence(incidence);

        Ok(Response::new(AddIncidenceResponse {
            id: id.0.to_string(),
            success: true,
        }))
    }

    async fn get_incidence(
        &self,
        request: Request<GetIncidenceRequest>,
    ) -> Result<Response<GetIncidenceResponse>, Status> {
        let req = request.into_inner();
        let graph = self.graph.lock().map_err(|e| {
            Status::internal(format!("Failed to lock graph: {}", e))
        })?;

        let id = IId(req.id.parse().map_err(|_| Status::invalid_argument("Invalid ID"))?);
        let incidence = graph.get(id).map(convert_incidence);

        Ok(Response::new(GetIncidenceResponse { incidence }))
    }

    async fn query_incidence(
        &self,
        request: Request<QueryIncidenceRequest>,
    ) -> Result<Response<QueryIncidenceResponse>, Status> {
        let req = request.into_inner();
        let graph = self.graph.lock().map_err(|e| {
            Status::internal(format!("Failed to lock graph: {}", e))
        })?;

        // 簡易実装: すべての Incidence を返す
        // 実際の実装では Datalog またはパターンクエリを評価
        let results: Vec<_> = graph.iter().map(convert_incidence).collect();

        Ok(Response::new(QueryIncidenceResponse { results }))
    }

    async fn vector_search(
        &self,
        request: Request<VectorSearchRequest>,
    ) -> Result<Response<VectorSearchResponse>, Status> {
        let req = request.into_inner();
        let graph = self.graph.lock().map_err(|e| {
            Status::internal(format!("Failed to lock graph: {}", e))
        })?;

        // 簡易実装: 全探索によるベクター検索
        // 実際の実装では VectorIndex を使用
        let query_vector = req.query_vector;
        let k = req.k as usize;

        let mut results: Vec<(IId, f32)> = Vec::new();
        for inc in graph.iter() {
            if let Some(embedding) = &inc.embedding {
                if embedding.len() == query_vector.len() {
                    let similarity = cosine_similarity(&query_vector, embedding);
                    results.push((inc.id, similarity));
                }
            }
        }

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(k);

        let vector_results: Vec<VectorResult> = results
            .into_iter()
            .map(|(id, score)| VectorResult {
                id: id.0.to_string(),
                score,
            })
            .collect();

        Ok(Response::new(VectorSearchResponse {
            results: vector_results,
        }))
    }
}

/// Value を変換
fn convert_value(value: incdb::Value) -> Result<Value, Status> {
    match value.value {
        Some(incdb::value::Value::Str(s)) => Ok(Value::string(s)),
        Some(incdb::value::Value::Int(i)) => Ok(Value::int(i)),
        Some(incdb::value::Value::Float(f)) => Ok(Value::float(f)),
        Some(incdb::value::Value::Bool(b)) => Ok(Value::bool(b)),
        Some(incdb::value::Value::Vector(v)) => Ok(Value::vector(v.values)),
        _ => Err(Status::invalid_argument("Invalid value")),
    }
}

/// Incidence を変換
fn convert_incidence(inc: &Incidence) -> incdb::Incidence {
    incdb::Incidence {
        id: inc.id.0.to_string(),
        level: inc.level.0,
        type_id: inc.ty.map(|t| t.0.to_string()),
        args: inc.args.iter().map(|a| a.0.to_string()).collect(),
        roles: inc.roles.iter().map(|r| r.0).collect(),
        value: inc.val.as_ref().map(convert_value_to_proto),
        embedding: inc.embedding.clone().unwrap_or_default(),
    }
}

/// Value をプロトコルバッファに変換
fn convert_value_to_proto(value: &Value) -> incdb::Value {
    match value {
        Value::Str(s) => incdb::Value {
            value: Some(incdb::value::Value::Str(s.clone())),
        },
        Value::Int(i) => incdb::Value {
            value: Some(incdb::value::Value::Int(*i)),
        },
        Value::Float(f) => incdb::Value {
            value: Some(incdb::value::Value::Float(*f)),
        },
        Value::Bool(b) => incdb::Value {
            value: Some(incdb::value::Value::Bool(*b)),
        },
        Value::Vector(v) => incdb::Value {
            value: Some(incdb::value::Value::Vector(incdb::Vector {
                values: v.clone(),
            })),
        },
        _ => incdb::Value { value: None },
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
        return 0.0;
    }

    dot / (norm_a * norm_b)
}

