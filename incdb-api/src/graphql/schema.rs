//! GraphQL Schema
//!
//! async-graphql を使用した GraphQL スキーマ定義

use async_graphql::{Context, Object, Schema, EmptySubscription};
use incdb_core::model::{IId, Incidence, Level, RoleId, Value, WorldGraph};
use incdb_query::datalog::{DatalogProgram, Predicate, Rule, DatalogError};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// GraphQL Query
pub struct Query;

#[Object]
impl Query {
    /// Incidence を取得
    async fn incidence(&self, ctx: &Context<'_>, id: String) -> async_graphql::Result<Option<IncidenceType>> {
        let graph = ctx.data::<Arc<Mutex<WorldGraph>>>()?;
        let graph = graph.lock().map_err(|e| async_graphql::Error::new(format!("Failed to lock graph: {}", e)))?;

        let id = IId(id.parse().map_err(|_| async_graphql::Error::new("Invalid ID"))?);

        Ok(graph.get(id).map(|inc| IncidenceType::from(inc)))
    }

    /// すべての Incidence を取得
    async fn incidences(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<IncidenceType>> {
        let graph = ctx.data::<Arc<Mutex<WorldGraph>>>()?;
        let graph = graph.lock().map_err(|e| async_graphql::Error::new(format!("Failed to lock graph: {}", e)))?;

        let results: Vec<IncidenceType> = graph.iter().map(IncidenceType::from).collect();
        Ok(results)
    }

    /// すべての Type を取得
    async fn types(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<TypeInfo>> {
        let graph = ctx.data::<Arc<Mutex<WorldGraph>>>()?;
        let graph = graph.lock().map_err(|e| async_graphql::Error::new(format!("Failed to lock graph: {}", e)))?;

        // by_type のキーから Type ID を取得
        let mut type_ids: Vec<IId> = graph.iter()
            .filter_map(|inc| inc.ty)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        type_ids.sort_by_key(|id| id.0);

        let mut types = Vec::new();
        for type_id in type_ids {
            if let Some(type_inc) = graph.get(type_id) {
                let count = graph.find_by_type(type_id).count();
                types.push(TypeInfo {
                    id: type_id.0.to_string(),
                    level: type_inc.level.0 as i32,
                    incidence_count: count,
                });
            }
        }

        Ok(types)
    }

    /// 特定の Type の詳細を取得
    async fn r#type(&self, ctx: &Context<'_>, id: String) -> async_graphql::Result<Option<TypeDetail>> {
        let graph = ctx.data::<Arc<Mutex<WorldGraph>>>()?;
        let graph = graph.lock().map_err(|e| async_graphql::Error::new(format!("Failed to lock graph: {}", e)))?;

        let type_id = IId(id.parse().map_err(|_| async_graphql::Error::new("Invalid ID"))?);
        
        let type_inc = match graph.get(type_id) {
            Some(inc) => inc,
            None => return Ok(None),
        };
        
        // この Type を持つ Incidence を取得
        let incidences: Vec<IncidenceType> = graph.find_by_type(type_id)
            .map(IncidenceType::from)
            .collect();

        // 構造パターンを分析（args と roles の組み合わせ）
        let mut patterns: HashMap<String, i32> = HashMap::new();
        for inc in graph.find_by_type(type_id) {
            let pattern = format!("args:{},roles:{}", inc.args.len(), inc.roles.len());
            *patterns.entry(pattern).or_insert(0) += 1;
        }

        let structure_patterns: Vec<StructurePattern> = patterns.into_iter()
            .map(|(pattern, count)| StructurePattern {
                pattern,
                count,
            })
            .collect();

        Ok(Some(TypeDetail {
            id: type_id.0.to_string(),
            level: type_inc.level.0 as i32,
            incidence_count: incidences.len(),
            incidences,
            structure_patterns,
        }))
    }

    /// スキーマ統計情報を取得
    async fn schema_stats(&self, ctx: &Context<'_>) -> async_graphql::Result<SchemaStats> {
        let graph = ctx.data::<Arc<Mutex<WorldGraph>>>()?;
        let graph = graph.lock().map_err(|e| async_graphql::Error::new(format!("Failed to lock graph: {}", e)))?;

        let total_incidences = graph.len();
        
        // Type の数をカウント
        let type_count = graph.iter()
            .filter_map(|inc| inc.ty)
            .collect::<std::collections::HashSet<_>>()
            .len();

        // Role の数をカウント
        let role_count = graph.iter()
            .flat_map(|inc| &inc.roles)
            .collect::<std::collections::HashSet<_>>()
            .len();

        // ベクトル埋め込みを持つ Incidence の数
        let vector_count = graph.iter()
            .filter(|inc| inc.embedding.is_some())
            .count();

        Ok(SchemaStats {
            total_incidences,
            type_count,
            role_count,
            vector_count,
        })
    }

    /// Datalog クエリを実行
    async fn execute_datalog(
        &self,
        ctx: &Context<'_>,
        query: String,
    ) -> async_graphql::Result<DatalogResult> {
        let graph = ctx.data::<Arc<Mutex<WorldGraph>>>()?;
        let graph = graph.lock().map_err(|e| async_graphql::Error::new(format!("Failed to lock graph: {}", e)))?;

        // 簡易パーサー: 基本的な Datalog クエリを解析
        // 形式: "Inc(1)" または "Type(1, 2)" など
        let mut program = DatalogProgram::new();
        
        // クエリを解析（簡易実装）
        // 実際の実装では、より高度なパーサーを使用する
        for line in query.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            // ファクトを解析: "Inc(1)" 形式
            if let Some(predicate) = parse_predicate(line) {
                program.add_fact(predicate);
            }
        }

        // プログラムを評価
        let results = program.evaluate(&graph)
            .map_err(|e| async_graphql::Error::new(format!("Datalog evaluation error: {}", e)))?;

        // 結果を IncidenceType に変換
        let incidences: Vec<IncidenceType> = results
            .iter()
            .filter_map(|pred| {
                match pred {
                    Predicate::Inc(id) => graph.get(*id).map(IncidenceType::from),
                    _ => None,
                }
            })
            .collect();

        Ok(DatalogResult {
            incidences,
            predicate_count: results.len(),
        })
    }

    /// ベクトル類似度検索
    async fn vector_search(
        &self,
        ctx: &Context<'_>,
        query_vector: Vec<f32>,
        k: i32,
    ) -> async_graphql::Result<VectorSearchResult> {
        let graph = ctx.data::<Arc<Mutex<WorldGraph>>>()?;
        let graph = graph.lock().map_err(|e| async_graphql::Error::new(format!("Failed to lock graph: {}", e)))?;

        let k = k as usize;
        let mut results: Vec<(IId, f32)> = Vec::new();

        // すべての Incidence を走査してベクトル類似度を計算
        for inc in graph.iter() {
            if let Some(embedding) = inc.embedding.as_ref() {
                if embedding.len() == query_vector.len() {
                    let similarity = cosine_similarity(&query_vector, embedding);
                    results.push((inc.id, similarity));
                }
            }
        }

        // 類似度でソート（降順）
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(k);

        // IncidenceType に変換
        let incidences: Vec<VectorSearchItem> = results
            .into_iter()
            .filter_map(|(id, similarity)| {
                graph.get(id).map(|inc| VectorSearchItem {
                    incidence: IncidenceType::from(inc),
                    similarity,
                })
            })
            .collect();

        Ok(VectorSearchResult {
            incidences,
        })
    }

    /// グラフ構造を取得
    async fn graph_structure(
        &self,
        ctx: &Context<'_>,
        ids: Vec<String>,
        depth: i32,
    ) -> async_graphql::Result<GraphStructure> {
        let graph = ctx.data::<Arc<Mutex<WorldGraph>>>()?;
        let graph = graph.lock().map_err(|e| async_graphql::Error::new(format!("Failed to lock graph: {}", e)))?;

        let depth = depth as usize;
        let mut visited: std::collections::HashSet<IId> = std::collections::HashSet::new();
        let mut nodes: Vec<GraphNode> = Vec::new();
        let mut edges: Vec<GraphEdge> = Vec::new();

        // 初期ノードを追加
        let mut queue: Vec<(IId, usize)> = ids
            .iter()
            .filter_map(|id| id.parse::<u64>().ok().map(IId))
            .map(|id| (id, 0))
            .collect();

        // BFS でグラフを探索
        while let Some((current_id, current_depth)) = queue.pop() {
            if visited.contains(&current_id) || current_depth > depth {
                continue;
            }
            visited.insert(current_id);

            if let Some(inc) = graph.get(current_id) {
                // ノードを追加
                nodes.push(GraphNode {
                    id: current_id.0.to_string(),
                    level: inc.level.0 as i32,
                    type_id: inc.ty.map(|t| t.0.to_string()),
                    value: inc.val.as_ref().map(ValueType::from),
                });

                // エッジを追加
                for (i, arg_id) in inc.args.iter().enumerate() {
                    let role = inc.roles.get(i).map(|r| r.0 as i32).unwrap_or(0);
                    edges.push(GraphEdge {
                        from: current_id.0.to_string(),
                        to: arg_id.0.to_string(),
                        role,
                    });

                    // 次の深さのノードをキューに追加
                    if current_depth < depth {
                        queue.push((*arg_id, current_depth + 1));
                    }
                }
            }
        }

        Ok(GraphStructure { nodes, edges })
    }
}

/// コサイン類似度を計算
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

/// グラフノード
#[derive(async_graphql::SimpleObject, Clone)]
pub struct GraphNode {
    pub id: String,
    pub level: i32,
    pub type_id: Option<String>,
    pub value: Option<ValueType>,
}

/// グラフエッジ
#[derive(async_graphql::SimpleObject, Clone)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub role: i32,
}

/// グラフ構造
#[derive(async_graphql::SimpleObject, Clone)]
pub struct GraphStructure {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
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

/// GraphQL Mutation
pub struct Mutation;

#[Object]
impl Mutation {
    /// Incidence を追加
    async fn add_incidence(
        &self,
        ctx: &Context<'_>,
        id: Option<String>,
        level: i32,
        type_id: Option<String>,
    ) -> async_graphql::Result<IncidenceType> {
        let graph = ctx.data::<Arc<Mutex<WorldGraph>>>()?;
        let mut graph = graph.lock().map_err(|e| async_graphql::Error::new(format!("Failed to lock graph: {}", e)))?;

        let id = if let Some(id_str) = id {
            IId(id_str.parse().map_err(|_| async_graphql::Error::new("Invalid ID"))?)
        } else {
            graph.new_id()
        };

        let level = Level(level as u8);
        let mut incidence = Incidence::new(id, level);

        if let Some(type_id_str) = type_id {
            let type_id = IId(type_id_str.parse().map_err(|_| async_graphql::Error::new("Invalid type ID"))?);
            incidence = incidence.with_type(type_id);
        }

        graph.add_incidence(incidence.clone());
        Ok(IncidenceType::from(&incidence))
    }
}

/// Incidence GraphQL Type
#[derive(async_graphql::SimpleObject, Clone)]
pub struct IncidenceType {
    pub id: String,
    pub level: i32,
    pub type_id: Option<String>,
    pub args: Vec<String>,
    pub roles: Vec<i32>,
    pub value: Option<ValueType>,
    pub embedding: Option<Vec<f32>>,
}

impl From<&Incidence> for IncidenceType {
    fn from(inc: &Incidence) -> Self {
        Self {
            id: inc.id.0.to_string(),
            level: inc.level.0 as i32,
            type_id: inc.ty.map(|t| t.0.to_string()),
            args: inc.args.iter().map(|a| a.0.to_string()).collect(),
            roles: inc.roles.iter().map(|r| r.0 as i32).collect(),
            value: inc.val.as_ref().map(ValueType::from),
            embedding: inc.embedding.clone(),
        }
    }
}

/// Value GraphQL Type
#[derive(async_graphql::SimpleObject, Clone)]
pub struct ValueType {
    pub str: Option<String>,
    pub int: Option<i64>,
    pub float: Option<f64>,
    pub bool: Option<bool>,
    pub vector: Option<Vec<f32>>,
}

impl From<&Value> for ValueType {
    fn from(value: &Value) -> Self {
        match value {
            Value::Str(s) => Self {
                str: Some(s.clone()),
                int: None,
                float: None,
                bool: None,
                vector: None,
            },
            Value::Int(i) => Self {
                str: None,
                int: Some(*i),
                float: None,
                bool: None,
                vector: None,
            },
            Value::Float(f) => Self {
                str: None,
                int: None,
                float: Some(*f),
                bool: None,
                vector: None,
            },
            Value::Bool(b) => Self {
                str: None,
                int: None,
                float: None,
                bool: Some(*b),
                vector: None,
            },
            Value::Vector(v) => Self {
                str: None,
                int: None,
                float: None,
                bool: None,
                vector: Some(v.clone()),
            },
            _ => Self {
                str: None,
                int: None,
                float: None,
                bool: None,
                vector: None,
            },
        }
    }
}

/// Type 情報
#[derive(async_graphql::SimpleObject, Clone)]
pub struct TypeInfo {
    pub id: String,
    pub level: i32,
    pub incidence_count: usize,
}

/// Type 詳細
#[derive(async_graphql::SimpleObject, Clone)]
pub struct TypeDetail {
    pub id: String,
    pub level: i32,
    pub incidence_count: usize,
    pub incidences: Vec<IncidenceType>,
    pub structure_patterns: Vec<StructurePattern>,
}

/// 構造パターン
#[derive(async_graphql::SimpleObject, Clone)]
pub struct StructurePattern {
    pub pattern: String,
    pub count: i32,
}

/// スキーマ統計情報
#[derive(async_graphql::SimpleObject, Clone)]
pub struct SchemaStats {
    pub total_incidences: usize,
    pub type_count: usize,
    pub role_count: usize,
    pub vector_count: usize,
}

/// Datalog クエリ結果
#[derive(async_graphql::SimpleObject, Clone)]
pub struct DatalogResult {
    pub incidences: Vec<IncidenceType>,
    pub predicate_count: usize,
}

/// ベクトル検索結果
#[derive(async_graphql::SimpleObject, Clone)]
pub struct VectorSearchResult {
    pub incidences: Vec<VectorSearchItem>,
}

/// ベクトル検索アイテム
#[derive(async_graphql::SimpleObject, Clone)]
pub struct VectorSearchItem {
    pub incidence: IncidenceType,
    pub similarity: f32,
}

/// GraphQL Schema を作成
pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;

pub fn create_schema(graph: Arc<Mutex<WorldGraph>>) -> AppSchema {
    Schema::build(Query, Mutation, EmptySubscription)
        .data(graph)
        .finish()
}
