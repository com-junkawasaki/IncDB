//! Query Example
//!
//! Incidence Datalog とパターンマッチクエリの使用例

use incdb_core::model::{IId, Incidence, Level, RoleId, Value, WorldGraph};
use incdb_query::datalog::{DatalogProgram, Predicate};
use incdb_query::pattern::{Pattern, PatternQuery, PatternVar, PatternExpr, PatternOp};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== IncDB Query Example ===\n");

    // データの準備
    let mut graph = WorldGraph::new();
    let person_type = graph.new_id();
    let event_type = graph.new_id();
    
    let person1 = graph.new_id();
    let person2 = graph.new_id();
    let event1 = graph.new_id();

    // Person インスタンス
    let p1 = Incidence::new(person1, Level::zero())
        .with_type(person_type)
        .with_val(Value::string("Alice"));
    let p2 = Incidence::new(person2, Level::zero())
        .with_type(person_type)
        .with_val(Value::string("Bob"));

    // Event インスタンス
    let e1 = Incidence::new(event1, Level::zero())
        .with_type(event_type)
        .with_val(Value::string("Meeting"))
        .add_arg(person1, RoleId(1))
        .add_arg(person2, RoleId(2));

    graph.add_incidence(p1);
    graph.add_incidence(p2);
    graph.add_incidence(e1);

    println!("Created graph with {} incidences", graph.len());

    // 1. Datalog クエリ
    println!("\n1. Datalog Query...");
    let mut program = DatalogProgram::new();
    program.add_fact(Predicate::Inc(person1));
    program.add_fact(Predicate::Inc(person2));
    program.add_fact(Predicate::Inc(event1));
    
    let results = program.evaluate(&graph)?;
    println!("   Found {} facts", results.len());

    // 2. パターンマッチクエリ（簡易版）
    println!("\n2. Pattern Match Query...");
    let mut query = PatternQuery::new();
    // 簡易実装のため、実際のパターンマッチは実装が必要
    
    println!("   Pattern query created (implementation pending)");

    println!("\n=== Query Example completed! ===");
    Ok(())
}

