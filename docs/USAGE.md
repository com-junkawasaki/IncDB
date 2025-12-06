# IncDB Usage Guide

## クイックスタート

### 1. 基本的な使用

```rust
use incdb_core::model::{WorldGraph, Incidence, Level, Value};

// WorldGraph の作成
let mut graph = WorldGraph::new();

// Incidence の追加
let id = graph.new_id();
let inc = Incidence::new(id, Level::zero())
    .with_val(Value::string("Hello, IncDB!"));

graph.add_incidence(inc);

// Incidence の取得
if let Some(inc) = graph.get(id) {
    println!("Found: {:?}", inc);
}
```

### 2. Type と Role の使用

```rust
let type_id = graph.new_id();
let person1 = graph.new_id();
let person2 = graph.new_id();

// Person インスタンス
let p1 = Incidence::new(person1, Level::zero())
    .with_type(type_id)
    .with_val(Value::string("Alice"));

// 関係の作成
let relation = graph.new_id();
let rel = Incidence::new(relation, Level::zero())
    .with_type(relation_type)
    .add_arg(person1, RoleId(1))  // subject
    .add_arg(person2, RoleId(2)); // object

graph.add_incidence(p1);
graph.add_incidence(rel);
```

### 3. ベクトル検索

```rust
use incdb_core::vector::embedding::Embedding;
use incdb_storage::index::vector_index::SimpleVectorIndex;

// 埋め込みの設定
Embedding::set_embedding(&mut graph, id1, vec![1.0, 0.0, 0.0])?;

// インデックスの作成
let mut index = SimpleVectorIndex::new();
if let Some(emb) = Embedding::get_embedding(&graph, id1) {
    index.add(id1, emb)?;
}

// 検索
let query = vec![1.0, 0.0, 0.0];
let results = index.search(&query, 10)?;
```

### 4. JSON エクスポート/インポート

```rust
use incdb_core::ir::InternalJsonConverter;

// エクスポート
let jsonld = InternalJsonConverter::from_world_graph(&graph);
let json_str = serde_json::to_string_pretty(&jsonld)?;

// インポート（実装予定）
// let graph = InternalJsonConverter::to_world_graph(&jsonld)?;
```

### 5. Coinductive と Bisimulation

```rust
use incdb_core::foundation::coinduction::CoinductiveChecker;
use incdb_core::foundation::bisimulation::BisimulationComputer;

// Coinductive チェック
let mut checker = CoinductiveChecker::new(&graph, 10);
let are_equal = checker.check_equality(id1, id2, 0);

// Bisimulation チェック
let computer = BisimulationComputer::new(&graph);
let result = computer.compute_bisimulation(id1, id2);
```

## GraphQL API の使用

### サーバーの起動

```rust
use incdb_api::graphql::GraphQLServer;
use std::sync::{Arc, Mutex};

let graph = Arc::new(Mutex::new(WorldGraph::new()));
GraphQLServer::serve("127.0.0.1:8080", graph).await?;
```

### クエリの実行

```bash
curl -X POST http://localhost:8080/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "{ incidences { id level } }"}'
```

## CLI の使用

```bash
# Incidence の追加
incdb add --level 0 --value "Hello"

# Incidence の取得
incdb get 1

# クエリの実行
incdb query "Inc(1)"

# JSON のエクスポート
incdb export output.jsonld
```

