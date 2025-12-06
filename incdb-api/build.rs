fn main() -> Result<(), Box<dyn std::error::Error>> {
    // gRPC プロトコルバッファのコンパイル
    // macOS での iconv リンクエラーを回避するため、環境変数で制御
    if std::env::var("SKIP_PROTO_BUILD").is_err() {
        tonic_build::configure()
            .build_server(true)
            .build_client(true)
            .compile(&["proto/incdb.proto"], &["proto"])?;
    }
    Ok(())
}

