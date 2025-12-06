//! Integration Tests for Benchmark Write + Multi-Hop
//!
//! benchmark_writeで作成されたIncidenceのargsが正しく保存・復元されているか確認

use incdb_core::model::{IId, Incidence, Level, RoleId};
use incdb_storage::backend::sled_backend::SledBackend;
use incdb_storage::graph::EventSourcedGraph;
use incdb_storage::graph::event_sourced_graph_builder::EventSourcedGraphBuilder;
use tempfile::TempDir;

#[tokio::test]
async fn test_benchmark_write_args_preservation() {
    // テスト用の一時ディレクトリを作成
    let temp_dir = TempDir::new().unwrap();
    let backend = std::sync::Arc::new(SledBackend::new(temp_dir.path()).unwrap());
    
    // EventSourcedGraphを作成
    let graph = EventSourcedGraphBuilder::new()
        .with_backend(backend)
        .build()
        .await
        .unwrap();
    
    let graph = std::sync::Arc::new(tokio::sync::Mutex::new(graph));
    let mut graph_guard = graph.lock().await;

    // benchmark_writeと同様の処理を実行
    let data_size = 100;
    let args_per_incidence = 2;
    let vector_dim = 128;

    // 事前にIDを生成
    let mut node_ids: Vec<IId> = Vec::new();
    for _ in 0..data_size {
        node_ids.push(graph_guard.new_id());
    }

    // バッチごとに書き込み
    for i in 0..data_size {
        let id = node_ids[i];
        let mut incidence = Incidence::new(id, Level::zero());

        // ベクトルを追加
        if vector_dim > 0 {
            let embedding: Vec<f32> = (0..vector_dim)
                .map(|j| ((i + j) as f32 * 0.001).sin())
                .collect();
            incidence = incidence.with_embedding(embedding);
        }

        // args を追加（前のノードへのリンク）
        if args_per_incidence > 0 && i > 0 {
            for j in 1..=args_per_incidence.min(i) {
                let prev_idx = i - j;
                let arg_id = node_ids[prev_idx];
                let role = RoleId((j % 10) as u32);
                incidence = incidence.add_arg(arg_id, role);
            }
        }

        graph_guard.add_incidence(incidence, None).await.unwrap();
    }

    // 作成されたIncidenceのargsを確認
    let mut args_preserved = 0;
    let mut args_missing = 0;
    
    for i in 0..data_size {
        let id = node_ids[i];
        if let Ok(Some(inc)) = graph_guard.get(id).await {
            let expected_args_count = if i > 0 {
                args_per_incidence.min(i)
            } else {
                0
            };
            
            if inc.args.len() == expected_args_count {
                args_preserved += 1;
                
                // argsの内容も確認
                if i > 0 {
                    for j in 1..=args_per_incidence.min(i) {
                        let prev_idx = i - j;
                        let expected_arg_id = node_ids[prev_idx];
                        if !inc.args.contains(&expected_arg_id) {
                            panic!("Expected arg {} not found in Incidence {}", expected_arg_id.0, id.0);
                        }
                    }
                }
            } else {
                args_missing += 1;
                eprintln!("Incidence {}: expected {} args, got {}", id.0, expected_args_count, inc.args.len());
            }
        } else {
            panic!("Failed to get Incidence {}", id.0);
        }
    }

    // すべてのIncidenceのargsが正しく保存されていることを確認
    assert_eq!(args_missing, 0, "Some Incidences have missing args");
    assert_eq!(args_preserved, data_size, "All Incidences should have preserved args");
}

#[tokio::test]
async fn test_benchmark_write_multi_hop_integration() {
    // テスト用の一時ディレクトリを作成
    let temp_dir = TempDir::new().unwrap();
    let backend = std::sync::Arc::new(SledBackend::new(temp_dir.path()).unwrap());
    
    // EventSourcedGraphを作成
    let graph = EventSourcedGraphBuilder::new()
        .with_backend(backend)
        .build()
        .await
        .unwrap();
    
    let graph = std::sync::Arc::new(tokio::sync::Mutex::new(graph));
    let mut graph_guard = graph.lock().await;

    // benchmark_writeと同様の処理を実行
    let data_size = 100;
    let args_per_incidence = 2;
    let vector_dim = 128;

    // 事前にIDを生成
    let mut node_ids: Vec<IId> = Vec::new();
    for _ in 0..data_size {
        node_ids.push(graph_guard.new_id());
    }

    // バッチごとに書き込み
    for i in 0..data_size {
        let id = node_ids[i];
        let mut incidence = Incidence::new(id, Level::zero());

        // ベクトルを追加
        if vector_dim > 0 {
            let embedding: Vec<f32> = (0..vector_dim)
                .map(|j| ((i + j) as f32 * 0.001).sin())
                .collect();
            incidence = incidence.with_embedding(embedding);
        }

        // args を追加（前のノードへのリンク）
        if args_per_incidence > 0 && i > 0 {
            for j in 1..=args_per_incidence.min(i) {
                let prev_idx = i - j;
                let arg_id = node_ids[prev_idx];
                let role = RoleId((j % 10) as u32);
                incidence = incidence.add_arg(arg_id, role);
            }
        }

        graph_guard.add_incidence(incidence, None).await.unwrap();
    }

    drop(graph_guard);

    // benchmark_multi_hopと同様の処理を実行
    let start_nodes = 10;
    let depth = 3;
    let avg_edges = 3;

    let graph_guard = graph.lock().await;
    let graph_size = graph_guard.len().await.unwrap();
    assert!(graph_size > 0, "Graph should not be empty");

    // 開始ノードを選択
    let start_ids: Vec<IId> = {
        match graph_guard.iter().await {
            Ok(incidences) => {
                if incidences.len() >= start_nodes {
                    incidences.iter().take(start_nodes).map(|inc| inc.id).collect()
                } else {
                    (1..=graph_size.min(start_nodes))
                        .map(|i| IId(i as u64))
                        .collect()
                }
            }
            Err(_) => {
                (1..=graph_size.min(start_nodes))
                    .map(|i| IId(i as u64))
                    .collect()
            }
        }
    };

    let mut total_hops = 0;
    let mut total_nodes_visited = 0;
    let mut nodes_with_args = 0;
    let mut nodes_without_args = 0;

    // 各開始ノードから多段hopを実行
    for start_id in start_ids {
        let mut current_level = vec![start_id];
        let mut visited = std::collections::HashSet::new();
        visited.insert(start_id);

        for _hop_num in 0..depth {
            let mut next_level = Vec::new();

            for node_id in &current_level {
                match graph_guard.get(*node_id).await {
                    Ok(Some(inc)) => {
                        // argsの有無を確認
                        if inc.args.is_empty() {
                            nodes_without_args += 1;
                        } else {
                            nodes_with_args += 1;
                        }

                        // args から次のノードを取得
                        let edges: Vec<IId> = if inc.args.is_empty() {
                            // argsが空の場合、連続するIDを試す
                            (1..=avg_edges)
                                .map(|offset| IId(node_id.0 + offset as u64))
                                .collect()
                        } else {
                            inc.args.iter()
                                .take(avg_edges)
                                .copied()
                                .collect()
                        };

                        for edge_id in edges {
                            match graph_guard.get(edge_id).await {
                                Ok(Some(_)) => {
                                    if !visited.contains(&edge_id) {
                                        visited.insert(edge_id);
                                        next_level.push(edge_id);
                                        total_hops += 1;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }

            if next_level.is_empty() {
                break;
            }

            current_level = next_level;
        }

        total_nodes_visited += visited.len();
    }

    // 検証
    assert!(total_nodes_visited > 0, "Should visit at least some nodes");
    
    // argsを持つノードが存在することを確認
    assert!(nodes_with_args > 0, "Should have nodes with args");
    
    // hopsが実行されたことを確認（argsが正しく保存されていれば）
    if nodes_without_args == 0 {
        assert!(total_hops > 0, "Should have hops if all nodes have args");
    }

    println!("Integration test results:");
    println!("  Total nodes visited: {}", total_nodes_visited);
    println!("  Total hops: {}", total_hops);
    println!("  Nodes with args: {}", nodes_with_args);
    println!("  Nodes without args: {}", nodes_without_args);
}
