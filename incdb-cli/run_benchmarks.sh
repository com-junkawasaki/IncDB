#!/bin/bash
# IncDB Benchmark Suite
# すべてのベンチマークを連続実行

set -e

BIN="./target/debug/incdb"

echo "=== IncDB Benchmark Suite ==="
echo ""

# 1. Write Benchmark
echo "1. Write Benchmark (1K, vector_dim=128)"
$BIN benchmark-write --data-size 1000 --batch-size 100 --vector-dim 128 --args-per-incidence 2
echo ""

# 2. Read Benchmark (書き込み後に実行)
echo "2. Read Benchmark (1K queries)"
$BIN benchmark-read --query-count 1000
echo ""

# 3. Multi-Hop Benchmark (書き込み後に実行)
echo "3. Multi-Hop Benchmark (10 nodes, depth 3)"
$BIN benchmark-multi-hop --start-nodes 10 --depth 3 --avg-edges 3
echo ""

# 4. Vector Hop Benchmark (書き込み後に実行)
echo "4. Vector Hop Benchmark (1K vectors, depth 3, k=10)"
$BIN benchmark-vector-hop --data-size 1000 --vector-dim 128 --depth 3 --k-per-hop 10
echo ""

echo "=== All benchmarks completed ==="

