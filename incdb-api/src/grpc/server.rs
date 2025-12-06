//! gRPC Server
//!
//! tonic を使用した gRPC サーバーの起動

use crate::grpc::service::IncDBServiceImpl;
use incdb_core::model::WorldGraph;
use std::sync::Arc;
use std::sync::Mutex;
use tonic::transport::Server;

/// IncDB gRPC サーバー
pub struct IncDBServer;

impl IncDBServer {
    /// サーバーを起動
    pub async fn serve(
        addr: std::net::SocketAddr,
        graph: Arc<Mutex<WorldGraph>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let service = IncDBServiceImpl::new(graph);

        Server::builder()
            .add_service(incdb::inc_db_service_server::IncDbServiceServer::new(service))
            .serve(addr)
            .await?;

        Ok(())
    }
}

// プロトコルバッファの生成コード
mod incdb {
    tonic::include_proto!("incdb");
}

