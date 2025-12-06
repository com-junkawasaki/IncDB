//! HNSW Benchmark
//!
//! HNSWインデックスの性能を測定

use incdb_storage::index::hnsw_index::HNSWVectorIndex;
use incdb_core::model::IId;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dimension = 128;
    let data_sizes = vec![1000, 10000, 100000];
    
    for &data_size in &data_sizes {
        println!("\n=== HNSW Benchmark: {} vectors ===", data_size);
        
        // HNSWインデックスを作成
        let mut index = HNSWVectorIndex::new_default(dimension)?;
        
        // ベクトルを生成して追加
        let start = Instant::now();
        for i in 0..data_size {
            let mut vector = Vec::with_capacity(dimension);
            for j in 0..dimension {
                vector.push(((i + j) as f32 * 0.001).sin());
            }
            // 正規化
            let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                vector.iter_mut().for_each(|x| *x /= norm);
            }
            index.add(IId(i as u64 + 1), &vector)?;
        }
        let insert_duration = start.elapsed();
        let insert_throughput = data_size as f64 / insert_duration.as_secs_f64();
        
        println!("Insert Performance:");
        println!("  Duration: {:.2}ms", insert_duration.as_secs_f64() * 1000.0);
        println!("  Throughput: {:.2} vectors/sec", insert_throughput);
        
        // 検索性能を測定
        let query_count = 100;
        let k = 10;
        
        // クエリベクトルを生成
        let mut query_vectors = Vec::new();
        for i in 0..query_count {
            let mut query = Vec::with_capacity(dimension);
            for j in 0..dimension {
                query.push(((i + j) as f32 * 0.001).cos());
            }
            // 正規化
            let norm: f32 = query.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                query.iter_mut().for_each(|x| *x /= norm);
            }
            query_vectors.push(query);
        }
        
        let search_start = Instant::now();
        let mut total_results = 0;
        for query in &query_vectors {
            match index.search(query, k) {
                Ok(results) => {
                    total_results += results.len();
                }
                Err(e) => {
                    eprintln!("Search error: {}", e);
                }
            }
        }
        let search_duration = search_start.elapsed();
        let search_throughput = query_count as f64 / search_duration.as_secs_f64();
        let avg_latency = search_duration.as_secs_f64() * 1000.0 / query_count as f64;
        
        println!("Search Performance:");
        println!("  Queries: {}", query_count);
        println!("  K: {}", k);
        println!("  Duration: {:.2}ms", search_duration.as_secs_f64() * 1000.0);
        println!("  Throughput: {:.2} queries/sec", search_throughput);
        println!("  Avg Latency: {:.4}ms", avg_latency);
        println!("  Total Results: {}", total_results);
    }
    
    Ok(())
}

