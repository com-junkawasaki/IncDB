# IncDB API Documentation

## 概要

IncDB は Incidence-only Foundation を実装したグラフ+ベクターデータベースシステムです。

## コア API

### WorldGraph

すべての Incidence を管理する中央リポジトリです。

```rust
use incdb_core::model::{WorldGraph, Incidence, Level};

let mut graph = WorldGraph::new();
let id = graph.new_id();
let inc = Incidence::new(id, Level::zero());
graph.add_incidence(inc);
```

### Incidence の操作

```rust
use incdb_core::model::{Incidence, Level, RoleId, Value};

// 基本的な Incidence の作成
let inc = Incidence::new(id, Level::zero())
    .with_type(type_id)
    .with_val(Value::string("Hello"))
    .add_arg(arg_id, RoleId(1));
```

### ベクトル埋め込み

```rust
use incdb_core::vector::embedding::Embedding;

let embedding = vec![0.1, 0.2, 0.3];
Embedding::set_embedding(&mut graph, id, embedding)?;
let retrieved = Embedding::get_embedding(&graph, id);
```

### Datalog クエリ

```rust
use incdb_query::datalog::{DatalogProgram, Predicate};

let mut program = DatalogProgram::new();
program.add_fact(Predicate::Inc(id));
let results = program.evaluate(&graph)?;
```

## GraphQL API

### クエリ

```graphql
query {
  incidence(id: "1") {
    id
    level
    typeId
    args
  }
  
  incidences {
    id
    value {
      str
    }
  }
}
```

### ミューテーション

```graphql
mutation {
  addIncidence(level: 0, typeId: "10") {
    id
    level
  }
}
```

## 使用例

詳細な使用例は `examples/` ディレクトリを参照してください。

- `basic_usage.rs`: 基本機能の使用例
- `query_example.rs`: クエリの使用例
- `storage_example.rs`: ストレージの使用例

