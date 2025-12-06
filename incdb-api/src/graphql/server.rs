//! GraphQL Server
//!
//! Poem を使用した GraphQL サーバーの起動

use crate::graphql::schema::{create_schema, Context, Schema};
use incdb_core::model::WorldGraph;
use poem::{
    handler,
    http::Method,
    listener::TcpListener,
    route,
    route::get,
    route::post,
    web::{Data, Json, Query as WebQuery},
    EndpointExt, Server,
};
use std::sync::{Arc, Mutex};

/// GraphQL サーバー
pub struct GraphQLServer;

impl GraphQLServer {
    /// サーバーを起動
    pub async fn serve(
        addr: &str,
        graph: Arc<Mutex<WorldGraph>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let schema = create_schema();
        let context = Context { graph };

        let app = route()
            .at("/graphql", post(graphql_handler).get(graphql_playground))
            .data(schema)
            .data(context);

        Server::new(TcpListener::bind(addr))
            .run(app)
            .await?;

        Ok(())
    }
}

/// GraphQL ハンドラー
#[handler]
async fn graphql_handler(
    schema: Data<&Schema>,
    context: Data<&Context>,
    req: Json<juniper::http::GraphQLRequest>,
) -> Json<juniper::http::GraphQLResponse> {
    let res = req.execute(&schema, context.0);
    Json(res)
}

/// GraphQL Playground ハンドラー
#[handler]
async fn graphql_playground() -> &'static str {
    r#"
<!DOCTYPE html>
<html>
<head>
    <title>GraphQL Playground</title>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/graphql-playground-react/build/static/css/index.css" />
    <link rel="shortcut icon" href="https://cdn.jsdelivr.net/npm/graphql-playground-react/build/favicon.png" />
    <script src="https://cdn.jsdelivr.net/npm/graphql-playground-react/build/static/js/middleware.js"></script>
</head>
<body>
    <div id="root">
        <style>
            body {
                margin: 0;
                background-color: #172a3a;
                font-family: 'Open Sans', sans-serif;
                overflow: hidden;
            }
        </style>
        <script>
            window.addEventListener('load', function (event) {
                GraphQLPlayground.init(document.getElementById('root'), {
                    endpoint: '/graphql'
                })
            });
        </script>
    </div>
</body>
</html>
"#
}

