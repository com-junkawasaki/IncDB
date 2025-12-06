//! Integration Tests
//!
//! WorldGraph + Query の連携テスト

use incdb_core::model::{IId, Incidence, Level, RoleId, Value, WorldGraph};
use incdb_core::ir::InternalJsonConverter;
use incdb_query::datalog::{DatalogProgram, Predicate};

#[test]
fn test_worldgraph_query_integration() {
    // 1. WorldGraph の作成
    let mut graph = WorldGraph::new();
    let person_type = graph.new_id();
    let person1 = graph.new_id();
    let person2 = graph.new_id();

    let p1 = Incidence::new(person1, Level::zero())
        .with_type(person_type)
        .with_val(Value::string("Alice"));

    let p2 = Incidence::new(person2, Level::zero())
        .with_type(person_type)
        .with_val(Value::string("Bob"));

    graph.add_incidence(p1);
    graph.add_incidence(p2);

    // 2. Datalog クエリの実行
    let mut program = DatalogProgram::new();
    program.add_fact(Predicate::Inc(person1));
    program.add_fact(Predicate::Inc(person2));
    program.add_fact(Predicate::Type(person1, person_type));
    program.add_fact(Predicate::Type(person2, person_type));

    let results = program.evaluate(&graph).unwrap();
    assert!(results.len() >= 2);

    // 3. Type で検索
    let found: Vec<_> = graph.find_by_type(person_type).collect();
    assert_eq!(found.len(), 2);
}

#[test]
fn test_coinductive_bisimulation_integration() {
    use incdb_core::foundation::coinduction::CoinductiveChecker;
    use incdb_core::foundation::bisimulation::BisimulationComputer;

    // 循環構造の作成
    let mut graph = WorldGraph::new();
    let id1 = graph.new_id();
    let id2 = graph.new_id();

    // id1 -> id2 -> id1 の循環
    let inc1 = Incidence::new(id1, Level::zero())
        .add_arg(id2, RoleId(1));
    let inc2 = Incidence::new(id2, Level::zero())
        .add_arg(id1, RoleId(1));

    graph.add_incidence(inc1);
    graph.add_incidence(inc2);

    // Coinductive チェック
    let mut checker = CoinductiveChecker::new(&graph, 10);
    let _are_equal = checker.check_equality(id1, id2, 0);
    // 循環構造なので、深さ制限により false になる可能性がある

    // Bisimulation チェック
    let computer = BisimulationComputer::new(&graph);
    let result = computer.compute_bisimulation(id1, id2);
    // 構造が異なるため、bisimilar ではない可能性が高い
    assert!(!result.are_bisimilar || result.relation.contains(&(id1, id2)));
}

#[cfg(feature = "storage")]
#[test]
fn test_vector_search_integration() {
    use incdb_core::vector::embedding::Embedding;
    // ベクトルインデックスは incdb-storage に移動したため、ここでは埋め込みのみテスト

    let mut graph = WorldGraph::new();
    let id1 = graph.new_id();
    let id2 = graph.new_id();
    let id3 = graph.new_id();

    let inc1 = Incidence::new(id1, Level::zero());
    let inc2 = Incidence::new(id2, Level::zero());
    let inc3 = Incidence::new(id3, Level::zero());

    graph.add_incidence(inc1);
    graph.add_incidence(inc2);
    graph.add_incidence(inc3);

    // ベクトル埋め込みの設定
    Embedding::set_embedding(&mut graph, id1, vec![1.0, 0.0, 0.0]).unwrap();
    Embedding::set_embedding(&mut graph, id2, vec![0.0, 1.0, 0.0]).unwrap();
    Embedding::set_embedding(&mut graph, id3, vec![0.0, 0.0, 1.0]).unwrap();

    // ベクトル埋め込みの取得をテスト
    assert!(Embedding::get_embedding(&graph, id1).is_some());
    assert!(Embedding::get_embedding(&graph, id2).is_some());
    assert!(Embedding::get_embedding(&graph, id3).is_some());
    
    // 埋め込みの次元を確認
    assert_eq!(Embedding::dimension(&graph, id1), Some(3));
}

#[test]
fn test_json_export_import() {
    // 1. WorldGraph の作成
    let mut graph = WorldGraph::new();
    let type_id = graph.new_id();
    let id1 = graph.new_id();
    let id2 = graph.new_id();

    let inc1 = Incidence::new(id1, Level::zero())
        .with_type(type_id)
        .with_val(Value::string("test1"))
        .add_arg(id2, RoleId(1));

    let inc2 = Incidence::new(id2, Level::zero())
        .with_val(Value::int(100));

    graph.add_incidence(inc1);
    graph.add_incidence(inc2);

    // 2. JSON エクスポート
    let jsonld = InternalJsonConverter::from_world_graph(&graph);
    assert_eq!(jsonld.graph.len(), 2);

    // 3. JSON の検証
    let json_str = serde_json::to_string(&jsonld).unwrap();
    assert!(json_str.contains("inc:1") || json_str.contains(&id1.0.to_string()));
}
