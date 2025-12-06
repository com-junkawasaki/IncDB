fn main() -> Result<(), Box<dyn std::error::Error>> {
    // gRPC プロトコルバッファのコンパイル
    // macOS での iconv リンクエラーを回避するため、gRPC 機能が有効な場合のみコンパイル
    #[cfg(feature = "grpc")]
    {
        tonic_build::configure()
            .build_server(true)
            .build_client(true)
            .compile(&["proto/incdb.proto"], &["proto"])?;
    }
    Ok(())
}

