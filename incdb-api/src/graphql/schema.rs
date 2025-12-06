//! GraphQL Schema
//!
//! async-graphql を使用した GraphQL スキーマ定義

use async_graphql::{Context, Object, Schema, EmptySubscription};
use incdb_core::model::{IId, Incidence, Level, Value, WorldGraph};
use std::sync::{Arc, Mutex};

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

/// GraphQL Schema を作成
pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;

pub fn create_schema(graph: Arc<Mutex<WorldGraph>>) -> AppSchema {
    Schema::build(Query, Mutation, EmptySubscription)
        .data(graph)
        .finish()
}
