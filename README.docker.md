# Docker Compose セットアップ

IncDB を Docker Compose で起動する手順です。

## 前提条件

- Docker と Docker Compose がインストールされていること

## 起動方法

```bash
# ビルドと起動
docker-compose up --build

# バックグラウンドで起動
docker-compose up -d --build

# ログを確認
docker-compose logs -f

# 停止
docker-compose down
```

## サービス

- **API**: GraphQL サーバー (http://localhost:8080)
  - GraphQL endpoint: http://localhost:8080/graphql
  - Health check: http://localhost:8080/health
- **Web**: SvelteKit UI (http://localhost:5173)

## データ永続化

データは `./data` ディレクトリに保存されます。

## トラブルシューティング

### ビルドエラー

```bash
# キャッシュをクリアして再ビルド
docker-compose build --no-cache
```

### ポートが既に使用されている場合

`docker-compose.yml` のポート番号を変更してください。

### Web UI が API に接続できない場合

環境変数 `VITE_API_URL` を確認してください。デフォルトは `http://localhost:8080` です。

