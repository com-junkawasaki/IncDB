//! Basic Usage Example
//!
//! IncDB の基本機能の動作確認

use incdb_core::model::{Incidence, Level, RoleId, Value, WorldGraph};
use incdb_core::foundation::coinduction::CoinductiveChecker;
use incdb_core::foundation::bisimulation::BisimulationComputer;
use incdb_core::vector::embedding::Embedding;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== IncDB Basic Usage Example ===\n");

    // 1. WorldGraph の作成と Incidence の追加
    println!("1. Creating WorldGraph and adding Incidences...");
    let mut graph = WorldGraph::new();
    
    let id1 = graph.new_id();
    let id2 = graph.new_id();
    let id3 = graph.new_id();
    let type_id = graph.new_id();

    let inc1 = Incidence::new(id1, Level::zero())
        .with_type(type_id)
        .with_val(Value::string("Hello"))
        .add_arg(id2, RoleId(1));

    let inc2 = Incidence::new(id2, Level::zero())
        .with_val(Value::int(42));

    let inc3 = Incidence::new(id3, Level::zero())
        .with_type(type_id)
        .add_arg(id2, RoleId(1));

    graph.add_incidence(inc1);
    graph.add_incidence(inc2);
    graph.add_incidence(inc3);

    println!("   Added {} incidences", graph.len());
    println!("   IDs: {}, {}, {}", id1.0, id2.0, id3.0);

    // 2. Incidence の取得
    println!("\n2. Retrieving Incidences...");
    if let Some(inc) = graph.get(id1) {
        println!("   Found incidence {}: level={}, type={:?}, args={:?}", 
                 inc.id.0, inc.level.0, inc.ty, inc.args);
    }

    // 3. Type で検索
    println!("\n3. Searching by Type...");
    let found: Vec<_> = graph.find_by_type(type_id).collect();
    println!("   Found {} incidences with type {}", found.len(), type_id.0);

    // 4. ベクトル埋め込みの設定
    println!("\n4. Setting vector embeddings...");
    let embedding1 = vec![0.1, 0.2, 0.3, 0.4];
    let embedding2 = vec![0.5, 0.6, 0.7, 0.8];
    
    Embedding::set_embedding(&mut graph, id1, embedding1.clone())?;
    Embedding::set_embedding(&mut graph, id3, embedding2.clone())?;
    
    if let Some(emb) = Embedding::get_embedding(&graph, id1) {
        println!("   Embedding for id1: {:?}", emb);
    }

    // 5. Coinductive チェック
    println!("\n5. Coinductive equality check...");
    let mut checker = CoinductiveChecker::new(&graph, 10);
    let are_equal = checker.check_equality(id1, id3, 0);
    println!("   id1 and id3 are coinductively equal: {}", are_equal);

    // 6. Bisimulation チェック
    println!("\n6. Bisimulation check...");
    let computer = BisimulationComputer::new(&graph);
    let result = computer.compute_bisimulation(id1, id3);
    println!("   id1 and id3 are bisimilar: {}", result.are_bisimilar);

    // 7. JSON エクスポート
    println!("\n7. Exporting to JSON-LD...");
    use incdb_core::ir::InternalJsonConverter;
    let jsonld = InternalJsonConverter::from_world_graph(&graph);
    println!("   Exported {} incidences to JSON-LD", jsonld.graph.len());

    println!("\n=== Example completed successfully! ===");
    Ok(())
}

