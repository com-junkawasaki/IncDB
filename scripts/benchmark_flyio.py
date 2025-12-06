#!/usr/bin/env python3
"""
Fly.io API ベンチマークスクリプト
GraphQL API経由でベンチマークを実行し、結果をJSON形式で出力
"""

import json
import requests
import sys
from typing import Dict, Any

API_URL = "https://incdb-api.fly.dev/graphql"

def run_graphql_query(query: str, variables: Dict[str, Any] = None) -> Dict[str, Any]:
    """GraphQLクエリを実行"""
    response = requests.post(
        API_URL,
        json={"query": query, "variables": variables or {}},
        headers={"Content-Type": "application/json"},
        timeout=600  # 10分のタイムアウト
    )
    response.raise_for_status()
    result = response.json()
    if "errors" in result:
        raise Exception(f"GraphQL errors: {result['errors']}")
    return result

def benchmark_write(data_size: int = 1000, batch_size: int = 100, vector_dim: int = 128, args_per_incidence: int = 2):
    """書き込みベンチマーク"""
    query = """
    mutation BenchmarkWrite($config: WriteBenchmarkConfig!) {
        benchmarkWrite(config: $config) {
            operation
            dataSize
            durationMs
            throughput
            latencyMs
            metadata {
                key
                value
            }
        }
    }
    """
    variables = {
        "config": {
            "dataSize": data_size,
            "batchSize": batch_size,
            "vectorDim": vector_dim,
            "argsPerIncidence": args_per_incidence
        }
    }
    return run_graphql_query(query, variables)

def benchmark_read(query_count: int = 1000):
    """読み込みベンチマーク"""
    query = """
    mutation BenchmarkRead($config: ReadBenchmarkConfig!) {
        benchmarkRead(config: $config) {
            operation
            dataSize
            durationMs
            throughput
            latencyMs
            metadata {
                key
                value
            }
        }
    }
    """
    variables = {
        "config": {
            "dataSize": 0,
            "queryCount": query_count
        }
    }
    return run_graphql_query(query, variables)

def benchmark_multi_hop(start_nodes: int = 10, depth: int = 3, avg_edges: int = 3):
    """多段hopベンチマーク"""
    query = """
    mutation BenchmarkMultiHop($config: MultiHopBenchmarkConfig!) {
        benchmarkMultiHop(config: $config) {
            operation
            dataSize
            durationMs
            throughput
            latencyMs
            metadata {
                key
                value
            }
        }
    }
    """
    variables = {
        "config": {
            "startNodes": start_nodes,
            "depth": depth,
            "avgEdges": avg_edges
        }
    }
    return run_graphql_query(query, variables)

def benchmark_vector_hop(data_size: int = 1000, vector_dim: int = 128, depth: int = 3, k_per_hop: int = 10):
    """ベクトルhopベンチマーク"""
    query = """
    mutation BenchmarkVectorHop($config: VectorHopBenchmarkConfig!) {
        benchmarkVectorHop(config: $config) {
            operation
            dataSize
            durationMs
            throughput
            latencyMs
            metadata {
                key
                value
            }
        }
    }
    """
    variables = {
        "config": {
            "dataSize": data_size,
            "vectorDim": vector_dim,
            "depth": depth,
            "kPerHop": k_per_hop
        }
    }
    return run_graphql_query(query, variables)

def main():
    """メイン関数"""
    results = {}
    
    print("=== IncDB Fly.io Benchmark Suite ===")
    print(f"API URL: {API_URL}\n")
    
    # 1. Write Benchmark
    print("1. Write Benchmark (1K, vector_dim=128)...")
    try:
        write_result = benchmark_write(1000, 100, 128, 2)
        results["write"] = write_result["data"]["benchmarkWrite"]
        print(f"   ✓ Completed: {write_result['data']['benchmarkWrite']['throughput']:.2f} ops/sec")
    except Exception as e:
        print(f"   ✗ Failed: {e}")
        results["write"] = {"error": str(e)}
    
    # 2. Read Benchmark
    print("\n2. Read Benchmark (1K queries)...")
    try:
        read_result = benchmark_read(1000)
        results["read"] = read_result["data"]["benchmarkRead"]
        print(f"   ✓ Completed: {read_result['data']['benchmarkRead']['throughput']:.2f} ops/sec")
    except Exception as e:
        print(f"   ✗ Failed: {e}")
        results["read"] = {"error": str(e)}
    
    # 3. Multi-Hop Benchmark
    print("\n3. Multi-Hop Benchmark (10 nodes, depth 3)...")
    try:
        multi_hop_result = benchmark_multi_hop(10, 3, 3)
        results["multi_hop"] = multi_hop_result["data"]["benchmarkMultiHop"]
        print(f"   ✓ Completed: {multi_hop_result['data']['benchmarkMultiHop']['throughput']:.2f} hops/sec")
    except Exception as e:
        print(f"   ✗ Failed: {e}")
        results["multi_hop"] = {"error": str(e)}
    
    # 4. Vector Hop Benchmark
    print("\n4. Vector Hop Benchmark (1K vectors, depth 3, k=10)...")
    try:
        vector_hop_result = benchmark_vector_hop(1000, 128, 3, 10)
        results["vector_hop"] = vector_hop_result["data"]["benchmarkVectorHop"]
        print(f"   ✓ Completed: {vector_hop_result['data']['benchmarkVectorHop']['throughput']:.2f} searches/sec")
    except Exception as e:
        print(f"   ✗ Failed: {e}")
        results["vector_hop"] = {"error": str(e)}
    
    print("\n=== All benchmarks completed ===")
    
    # 結果をJSONファイルに保存
    output_file = "benchmark_results_flyio.json"
    with open(output_file, "w") as f:
        json.dump(results, f, indent=2)
    print(f"\nResults saved to: {output_file}")
    
    return results

if __name__ == "__main__":
    try:
        results = main()
        sys.exit(0)
    except KeyboardInterrupt:
        print("\n\nBenchmark interrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"\n\nError: {e}")
        sys.exit(1)
