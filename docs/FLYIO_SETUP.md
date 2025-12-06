# Fly.io + Tigris セットアップガイド

IncDBをFly.io + Tigrisでデプロイする手順です。

## 前提条件

- [Fly.io CLI](https://fly.io/docs/getting-started/installing-flyctl/) がインストールされていること
- [Tigris](https://www.tigrisdata.com/) アカウントが作成されていること
- Rust と Docker がインストールされていること

## 1. Fly.io アカウントのセットアップ

```bash
# Fly.io CLI のインストール（macOS）
curl -L https://fly.io/install.sh | sh

# Fly.io にログイン
flyctl auth login

# アプリの初期化（APIサーバー）
cd /path/to/IncDB
flyctl launch --config fly.api.toml --name incdb-api

# アプリの初期化（Web UI）
flyctl launch --config fly.web.toml --name incdb-web
```

## 2. Tigris のセットアップ

### 2.1 Tigris プロジェクトの作成

1. [Tigris Dashboard](https://www.tigrisdata.com/) にログイン
2. 新しいプロジェクトを作成（例: `incdb-production`）
3. API Key を生成して保存

### 2.2 Tigris バケットの作成

```bash
# Tigris CLI のインストール
npm install -g @tigrisdata/tigris-cli

# Tigris にログイン
tigris login

# バケットの作成
tigris bucket create incdb-events --project incdb-production
```

## 3. 環境変数の設定

### 3.1 APIサーバーの環境変数

```bash
# Tigris の認証情報を設定
flyctl secrets set \
  TIGRIS_PROJECT_ID=incdb-production \
  TIGRIS_CLIENT_ID=your-client-id \
  TIGRIS_CLIENT_SECRET=your-client-secret \
  TIGRIS_ENDPOINT=https://api.tigrisdata.cloud \
  --app incdb-api

# ストレージパスを設定（Tigrisを使用する場合は不要）
flyctl secrets set INCDB_STORAGE_PATH=/data --app incdb-api
```

### 3.2 Web UIの環境変数

```bash
# API URLを設定
flyctl secrets set API_URL=https://incdb-api.fly.dev --app incdb-web
```

## 4. ストレージボリュームの作成（オプション）

Fly.ioの永続ボリュームを作成する場合（Tigrisを使用しない場合）:

```bash
# データボリュームの作成
flyctl volumes create incdb_data --size 10 --region iad --app incdb-api
```

## 5. デプロイ

### 5.1 APIサーバーのデプロイ

```bash
# ビルドとデプロイ
flyctl deploy --config fly.api.toml --app incdb-api

# ログの確認
flyctl logs --app incdb-api

# ステータスの確認
flyctl status --app incdb-api
```

### 5.2 Web UIのデプロイ

```bash
# ビルドとデプロイ
flyctl deploy --config fly.web.toml --app incdb-web

# ログの確認
flyctl logs --app incdb-web

# ステータスの確認
flyctl status --app incdb-web
```

## 6. Tigris バックエンドの実装（オプション）

現在のIncDBはSledBackendを使用していますが、Tigrisに対応するにはバックエンド実装を追加する必要があります。

### 6.1 TigrisBackendの実装例

`incdb-storage/src/backend/tigris_backend.rs` を作成:

```rust
//! Tigris Backend
//!
//! Tigris を使用したストレージバックエンド実装

use crate::backend::traits::{Backend, BackendError};
use async_trait::async_trait;
use incdb_core::model::IId;
use std::sync::Arc;

/// Tigris バックエンド
pub struct TigrisBackend {
    client: Arc<tigris::TigrisClient>,
    bucket: String,
}

impl TigrisBackend {
    /// 新しい TigrisBackend を作成
    pub fn new(
        project_id: String,
        client_id: String,
        client_secret: String,
        bucket: String,
    ) -> Result<Self, BackendError> {
        let client = tigris::TigrisClient::new(
            project_id,
            client_id,
            client_secret,
        )?;
        
        Ok(Self {
            client: Arc::new(client),
            bucket,
        })
    }
}

#[async_trait]
impl Backend for TigrisBackend {
    async fn put_incidence(&self, id: IId, data: &[u8]) -> Result<(), BackendError> {
        let key = format!("incidences/{}", id.0);
        self.client
            .bucket(&self.bucket)
            .put(&key, data)
            .await
            .map_err(|e| BackendError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn get_incidence(&self, id: IId) -> Result<Option<Vec<u8>>, BackendError> {
        let key = format!("incidences/{}", id.0);
        match self.client
            .bucket(&self.bucket)
            .get(&key)
            .await
        {
            Ok(data) => Ok(Some(data)),
            Err(tigris::Error::NotFound) => Ok(None),
            Err(e) => Err(BackendError::Storage(e.to_string())),
        }
    }

    async fn delete_incidence(&self, id: IId) -> Result<(), BackendError> {
        let key = format!("incidences/{}", id.0);
        self.client
            .bucket(&self.bucket)
            .delete(&key)
            .await
            .map_err(|e| BackendError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn list_incidences(&self) -> Result<Vec<IId>, BackendError> {
        let keys = self.client
            .bucket(&self.bucket)
            .list("incidences/")
            .await
            .map_err(|e| BackendError::Storage(e.to_string()))?;
        
        let ids: Result<Vec<_>, _> = keys
            .iter()
            .map(|key| {
                let id_str = key.strip_prefix("incidences/")
                    .ok_or_else(|| BackendError::Storage("Invalid key format".to_string()))?;
                let id = id_str.parse::<u64>()
                    .map_err(|e| BackendError::Storage(e.to_string()))?;
                Ok(IId(id))
            })
            .collect();
        
        ids
    }

    async fn close(&self) -> Result<(), BackendError> {
        // Tigris クライアントは自動的にクローズされる
        Ok(())
    }
}
```

### 6.2 EventSourcedGraphBuilderの更新

`incdb-storage/src/graph/event_sourced_graph_builder.rs` を更新して、TigrisBackendを選択可能にする:

```rust
use crate::backend::tigris_backend::TigrisBackend;

impl EventSourcedGraphBuilder {
    // ... 既存のコード ...

    pub fn with_tigris_backend(
        mut self,
        project_id: String,
        client_id: String,
        client_secret: String,
        bucket: String,
    ) -> Self {
        self.backend_type = BackendType::Tigris {
            project_id,
            client_id,
            client_secret,
            bucket,
        };
        self
    }
}
```

## 7. コスト最適化

### 7.1 Fly.io リザベーション（40%割引）

```bash
# リザベーションの作成
flyctl platform vm reservation create \
  --size performance-4x \
  --region iad \
  --app incdb-api
```

### 7.2 Tigris ストレージ階層の選択

- **Standard Tier**: $0.02/GB/月（頻繁にアクセス）
- **Infrequent Access**: $0.01/GB/月（推奨、Event Sourcing向け）
- **Archive Tier**: $0.004/GB/月（長期保存）

## 8. モニタリングとログ

### 8.1 ログの確認

```bash
# APIサーバーのログ
flyctl logs --app incdb-api

# Web UIのログ
flyctl logs --app incdb-web

# リアルタイムログ
flyctl logs --app incdb-api --follow
```

### 8.2 メトリクスの確認

```bash
# APIサーバーのメトリクス
flyctl metrics --app incdb-api

# Web UIのメトリクス
flyctl metrics --app incdb-web
```

## 9. トラブルシューティング

### 9.1 デプロイエラー

```bash
# ビルドログの確認
flyctl logs --app incdb-api

# ローカルでビルドテスト
docker build -f Dockerfile.api -t incdb-api .
```

### 9.2 接続エラー

```bash
# ヘルスチェックの確認
flyctl status --app incdb-api

# SSH接続でデバッグ
flyctl ssh console --app incdb-api
```

### 9.3 ストレージエラー

```bash
# ボリュームの確認
flyctl volumes list --app incdb-api

# ボリュームの拡張
flyctl volumes extend <volume-id> --size 20 --app incdb-api
```

## 10. スケーリング

### 10.1 水平スケーリング

```bash
# APIサーバーのスケールアウト
flyctl scale count 2 --app incdb-api

# Web UIのスケールアウト
flyctl scale count 2 --app incdb-web
```

### 10.2 リソースのスケーリング

```bash
# CPU/メモリの増加
flyctl scale vm performance-8x --app incdb-api
```

## 11. バックアップとリストア

### 11.1 Tigris バックアップ

```bash
# Tigris バケットのバックアップ
tigris bucket backup incdb-events --project incdb-production --output backup.tar.gz
```

### 11.2 Fly.io ボリュームのスナップショット

```bash
# ボリュームのスナップショット作成
flyctl volumes snapshot create <volume-id> --app incdb-api
```

## 12. セキュリティ

### 12.1 シークレットの管理

```bash
# シークレットの設定
flyctl secrets set SECRET_KEY=your-secret-key --app incdb-api

# シークレットの確認（値は表示されない）
flyctl secrets list --app incdb-api

# シークレットの削除
flyctl secrets unset SECRET_KEY --app incdb-api
```

### 12.2 ネットワークセキュリティ

```bash
# プライベートネットワークの設定
flyctl ips allocate-v6 --private --app incdb-api
```

## 13. コスト見積もり（100億レコード規模）

- **Fly.io APIサーバー**: $244/月（performance-4x: 4 vCPU, 32GB RAM）
- **Fly.io Web UI**: $3/月（shared-cpu-1x: 1 CPU, 512MB RAM）
- **Fly.io ストレージ**: $2/月（一時データ用10GB）
- **Fly.io データ転送**: $2/月（100GB outbound）
- **Tigris ストレージ**: $31/月（Infrequent Access: 3.06 TB）
- **合計**: **$281/月**

## 14. 参考リンク

- [Fly.io Documentation](https://fly.io/docs/)
- [Tigris Documentation](https://www.tigrisdata.com/docs/)
- [IncDB Architecture](./ARCHITECTURE.jsonld)
