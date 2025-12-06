//! GraphQL Server
//!
//! Poem を使用した GraphQL サーバーの起動

use crate::graphql::schema::{create_schema, AppSchema};
use async_graphql_poem::{GraphQLRequest, GraphQLResponse};
use incdb_core::model::WorldGraph;
use poem::{
    handler,
    listener::TcpListener,
    route,
    route::get,
    route::post,
    web::Data,
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
        let schema = create_schema(graph);

        let app = route()
            .at("/graphql", post(graphql_handler).get(graphql_playground))
            .data(schema);

        Server::new(TcpListener::bind(addr))
            .run(app)
            .await?;

        Ok(())
    }
}

/// GraphQL ハンドラー
#[handler]
async fn graphql_handler(
    schema: Data<&AppSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.0).await.into()
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
