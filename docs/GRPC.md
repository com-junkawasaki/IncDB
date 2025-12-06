# gRPC API

## 概要

IncDB は gRPC API を提供しています（オプショナル機能）。

## 有効化

gRPC 機能を有効にするには、`--features grpc` フラグを使用します：

```bash
cargo build --features grpc
```

## macOS での注意事項

macOS で gRPC 機能をビルドする際、`iconv` ライブラリのリンクエラーが発生する場合があります。

### 解決方法

1. **Xcode Command Line Tools の確認**:
   ```bash
   xcode-select --install
   ```

2. **環境変数の設定**:
   ```bash
   export LIBRARY_PATH=/usr/lib:$LIBRARY_PATH
   ```

3. **Homebrew で gRPC をインストール**:
   ```bash
   brew install grpc
   ```

## プロトコル定義

gRPC のプロトコル定義は `incdb-api/proto/incdb.proto` にあります。

## 使用例

```rust
use incdb_api::grpc::{IncDBServer, IncDBServiceImpl};
use std::sync::{Arc, Mutex};

let graph = Arc::new(Mutex::new(WorldGraph::new()));
IncDBServer::serve("127.0.0.1:50051".parse()?, graph).await?;
```

