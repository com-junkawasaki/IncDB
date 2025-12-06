//! GraphQL Schema
//!
//! Juniper を使用した GraphQL スキーマ定義

use incdb_core::model::{IId, Incidence, Level, RoleId, Value, WorldGraph};
use juniper::{EmptySubscription, FieldResult, RootNode};
use std::sync::{Arc, Mutex};

/// GraphQL Context
pub struct Context {
    pub graph: Arc<Mutex<WorldGraph>>,
}

impl juniper::Context for Context {}

/// GraphQL Query
pub struct Query;

#[juniper::graphql_object(Context = Context)]
impl Query {
    /// Incidence を取得
    fn incidence(context: &Context, id: String) -> FieldResult<Option<IncidenceType>> {
        let graph = context.graph.lock().map_err(|e| {
            juniper::FieldError::new("Failed to lock graph", juniper::Value::null())
        })?;

        let id = IId(id.parse().map_err(|_| {
            juniper::FieldError::new("Invalid ID", juniper::Value::null())
        })?);

        Ok(graph.get(id).map(|inc| IncidenceType::from(inc)))
    }

    /// すべての Incidence を取得
    fn incidences(context: &Context) -> FieldResult<Vec<IncidenceType>> {
        let graph = context.graph.lock().map_err(|e| {
            juniper::FieldError::new("Failed to lock graph", juniper::Value::null())
        })?;

        let results: Vec<IncidenceType> = graph.iter().map(IncidenceType::from).collect();
        Ok(results)
    }
}

/// GraphQL Mutation
pub struct Mutation;

#[juniper::graphql_object(Context = Context)]
impl Mutation {
    /// Incidence を追加
    fn add_incidence(
        context: &Context,
        id: Option<String>,
        level: i32,
        type_id: Option<String>,
    ) -> FieldResult<IncidenceType> {
        let mut graph = context.graph.lock().map_err(|e| {
            juniper::FieldError::new("Failed to lock graph", juniper::Value::null())
        })?;

        let id = if let Some(id_str) = id {
            IId(id_str.parse().map_err(|_| {
                juniper::FieldError::new("Invalid ID", juniper::Value::null())
            })?)
        } else {
            graph.new_id()
        };

        let level = Level(level as u8);
        let incidence = Incidence::new(id, level);

        if let Some(type_id_str) = type_id {
            let type_id = IId(type_id_str.parse().map_err(|_| {
                juniper::FieldError::new("Invalid type ID", juniper::Value::null())
            })?);
            let incidence = incidence.with_type(type_id);
            graph.add_incidence(incidence.clone());
            Ok(IncidenceType::from(&incidence))
        } else {
            graph.add_incidence(incidence.clone());
            Ok(IncidenceType::from(&incidence))
        }
    }
}

/// Incidence GraphQL Type
#[derive(juniper::GraphQLObject)]
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
#[derive(juniper::GraphQLObject)]
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
pub type Schema = RootNode<'static, Query, Mutation, EmptySubscription<Context>>;

pub fn create_schema() -> Schema {
    Schema::new(Query, Mutation, EmptySubscription::new())
}

