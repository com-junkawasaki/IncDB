# IncDB

Incidence-only Foundation を実装するグラフ+ベクターデータベースシステム。

## 概要

IncDB は、Incidence を唯一の基本実体とし、Type・Set・Category を Incidence のパターンとして定義する自己参照的データモデルを実装したデータベースです。

## アーキテクチャ

- **incdb-core**: コアライブラリ（Incidence Foundation、WorldGraph）
- **incdb-storage**: ストレージ層（永続化、インデックス）
- **incdb-query**: クエリエンジン（Incidence Datalog、パターンマッチ）
- **incdb-api**: API層（gRPC/GraphQL）
- **incdb-cli**: CLIツール
- **incdb-web**: Web UI（SvelteKit）

## 設計原則

- **Rust 借用チェッカー活用**: WorldGraph 中心設計でメモリ安全性を保証
- **Incidence-only**: すべての構造を Incidence のパターンとして定義
- **Coinduction**: 無限構造を安全に扱う
- **ベクター統合**: 構造検索とベクター検索のハイブリッド

## 開発

```bash
# ビルド
cargo build --workspace

# テスト
cargo test --workspace

# サンプルコードの実行
cargo run --example basic_usage -p incdb-core

# ドキュメント生成
cargo doc --workspace --open

# Web UI 開発
cd incdb-web
npm install
npm run dev
```

## 機能

- ✅ **Incidence-only Foundation**: すべての構造を Incidence のパターンとして定義
- ✅ **Coinduction サポート**: 無限構造を安全に扱う
- ✅ **Bisimulation Equality**: 構造的等価性の判定
- ✅ **ベクトル検索**: 埋め込みベースの類似度検索
- ✅ **Datalog クエリ**: 宣言的クエリ言語
- ✅ **GraphQL API**: async-graphql による GraphQL サーバー
- ⚠️ **gRPC API**: オプショナル機能（macOS では追加設定が必要）

## デプロイ

### Docker Compose

```bash
docker-compose up --build
```

詳細は [README.docker.md](README.docker.md) を参照してください。

### Fly.io + Tigris（推奨）

最もコスト効率の良い構成（$281/月、100億レコード規模）。

```bash
# Fly.io CLI のインストール
curl -L https://fly.io/install.sh | sh

# デプロイ
flyctl deploy --config fly.api.toml --app incdb-api
flyctl deploy --config fly.web.toml --app incdb-web
```

詳細は [Fly.io Setup Guide](docs/FLYIO_SETUP.md) を参照してください。

## スケーリング

IncDBのスケーリング構成とコスト比較については、[ARCHITECTURE.jsonld](ARCHITECTURE.jsonld) の `incdb:scalingPatterns` を参照してください。

推奨構成:
- **コスト重視**: Fly.io + Tigris ($281/月)
- **バランス**: Scaleway Kapsule + Object Storage ($466/月)
- **エンタープライズ**: AWS EKS + S3 ($1,533/月)

## ドキュメント

- [API Documentation](docs/API.md)
- [Usage Guide](docs/USAGE.md)
- [gRPC API](docs/GRPC.md)
- [Fly.io Setup Guide](docs/FLYIO_SETUP.md)

## ライセンス

MIT OR Apache-2.0

