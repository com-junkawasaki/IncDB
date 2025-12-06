<script lang="ts">
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import { GraphQLClient } from 'graphql-request';
	import { gql } from 'graphql-request';

	const API_URL = browser ? '/api/graphql' : 'http://api:8080/graphql';
	const client = new GraphQLClient(API_URL);

	// ベンチマーク設定
	let writeConfig = {
		dataSize: 1000,
		batchSize: 100,
		vectorDim: 128,
		argsPerIncidence: 2
	};

	let readConfig = {
		dataSize: 0,
		queryCount: 1000
	};

	let multiHopConfig = {
		startNodes: 10,
		depth: 3,
		avgEdges: 3
	};

	let vectorHopConfig = {
		dataSize: 1000,
		vectorDim: 128,
		depth: 3,
		kPerHop: 10
	};

	// 結果
	let results: any[] = [];
	let loading = false;
	let error: string | null = null;
	
	// 進捗表示
	let progress = {
		current: 0,
		total: 0,
		percentage: 0,
		message: '',
		elapsedTime: 0,
		estimatedTime: 0
	};
	let progressInterval: any = null;
	let startTime: number = 0;

	// グラフデータ
	let chartData: any = null;
	let Plotly: any = null;
	let plotlyLoaded = false;

	onMount(async () => {
		if (browser) {
			try {
				// CDN から Plotly を読み込む
				if (!(window as any).Plotly) {
					const script = document.createElement('script');
					script.src = 'https://cdn.plot.ly/plotly-2.27.1.min.js';
					script.onload = () => {
						Plotly = (window as any).Plotly;
						plotlyLoaded = true;
						console.log('Plotly loaded from CDN');
						if (results.length > 0) {
							updateChart();
						}
					};
					script.onerror = () => {
						console.error('Failed to load Plotly from CDN');
					};
					document.head.appendChild(script);
				} else {
					Plotly = (window as any).Plotly;
					plotlyLoaded = true;
				}
			} catch (e) {
				console.error('Failed to load Plotly:', e);
			}
		}
	});

	async function clearGraph() {
		loading = true;
		error = null;

		try {
			const mutation = gql`
				mutation {
					benchmarkClearGraph
				}
			`;
			await client.request(mutation);
			results = [];
			chartData = null;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Unknown error';
		} finally {
			loading = false;
		}
	}

	function startProgress(total: number, message: string) {
		progress = {
			current: 0,
			total: total,
			percentage: 0,
			message: message,
			elapsedTime: 0,
			estimatedTime: 0
		};
		startTime = Date.now();
		
		progressInterval = setInterval(() => {
			const elapsed = (Date.now() - startTime) / 1000;
			progress.elapsedTime = elapsed;
			
			// 進捗をシミュレート（実際の進捗は取得できないため、時間ベースで推定）
			if (progress.total > 0) {
				// 線形補間で進捗を推定（実際の処理速度に応じて調整）
				const estimatedProgress = Math.min(95, (elapsed / (progress.total / 1000)) * 100);
				progress.current = Math.floor((estimatedProgress / 100) * progress.total);
				progress.percentage = estimatedProgress;
				
				if (progress.current > 0 && elapsed > 0) {
					const rate = progress.current / elapsed;
					const remaining = progress.total - progress.current;
					progress.estimatedTime = remaining / rate;
				}
			}
		}, 100);
	}
	
	function stopProgress() {
		if (progressInterval) {
			clearInterval(progressInterval);
			progressInterval = null;
		}
		progress = {
			current: progress.total,
			total: progress.total,
			percentage: 100,
			message: 'Completed',
			elapsedTime: (Date.now() - startTime) / 1000,
			estimatedTime: 0
		};
	}

	async function runWriteBenchmark() {
		loading = true;
		error = null;
		startProgress(writeConfig.dataSize, `Writing ${writeConfig.dataSize.toLocaleString()} incidences...`);

		try {
			const mutation = gql`
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
			`;
			const result = await client.request(mutation, {
				config: {
					dataSize: writeConfig.dataSize,
					batchSize: writeConfig.batchSize,
					vectorDim: writeConfig.vectorDim,
					argsPerIncidence: writeConfig.argsPerIncidence
				}
			});
			stopProgress();
			results.push({
				...result.benchmarkWrite,
				timestamp: new Date().toISOString()
			});
			updateChart();
		} catch (e) {
			stopProgress();
			error = e instanceof Error ? e.message : 'Unknown error';
		} finally {
			loading = false;
		}
	}

	async function runReadBenchmark() {
		loading = true;
		error = null;
		startProgress(readConfig.queryCount, `Reading ${readConfig.queryCount.toLocaleString()} queries...`);

		try {
			const mutation = gql`
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
			`;
			const result = await client.request(mutation, {
				config: {
					dataSize: readConfig.dataSize,
					queryCount: readConfig.queryCount
				}
			});
			stopProgress();
			results.push({
				...result.benchmarkRead,
				timestamp: new Date().toISOString()
			});
			updateChart();
		} catch (e) {
			stopProgress();
			error = e instanceof Error ? e.message : 'Unknown error';
		} finally {
			loading = false;
		}
	}

	async function runMultiHopBenchmark() {
		loading = true;
		error = null;
		const totalOps = multiHopConfig.startNodes * multiHopConfig.depth * (multiHopConfig.avgEdges || 3);
		startProgress(totalOps, `Multi-hop traversal: ${multiHopConfig.startNodes} nodes × depth ${multiHopConfig.depth}...`);

		try {
			const mutation = gql`
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
			`;
			const result = await client.request(mutation, {
				config: {
					startNodes: multiHopConfig.startNodes,
					depth: multiHopConfig.depth,
					avgEdges: multiHopConfig.avgEdges
				}
			});
			stopProgress();
			results.push({
				...result.benchmarkMultiHop,
				timestamp: new Date().toISOString()
			});
			updateChart();
		} catch (e) {
			stopProgress();
			error = e instanceof Error ? e.message : 'Unknown error';
		} finally {
			loading = false;
		}
	}

	async function runVectorHopBenchmark() {
		loading = true;
		error = null;
		const totalOps = vectorHopConfig.dataSize * vectorHopConfig.depth * vectorHopConfig.kPerHop;
		startProgress(totalOps, `Vector hop search: ${vectorHopConfig.dataSize} vectors × depth ${vectorHopConfig.depth} × k=${vectorHopConfig.kPerHop}...`);

		try {
			const mutation = gql`
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
			`;
			const result = await client.request(mutation, {
				config: {
					dataSize: vectorHopConfig.dataSize,
					vectorDim: vectorHopConfig.vectorDim,
					depth: vectorHopConfig.depth,
					kPerHop: vectorHopConfig.kPerHop
				}
			});
			stopProgress();
			results.push({
				...result.benchmarkVectorHop,
				timestamp: new Date().toISOString()
			});
			updateChart();
		} catch (e) {
			stopProgress();
			error = e instanceof Error ? e.message : 'Unknown error';
		} finally {
			loading = false;
		}
	}

	function updateChart() {
		if (!plotlyLoaded || !Plotly || results.length === 0) {
			return;
		}

		const container = document.querySelector('.chart-container') as HTMLDivElement;
		if (!container) {
			return;
		}

		// データサイズ別のスループットを可視化
		const writeResults = results.filter((r) => r.operation === 'write');
		const readResults = results.filter((r) => r.operation === 'read');
		const multiHopResults = results.filter((r) => r.operation === 'multi_hop');
		const vectorHopResults = results.filter((r) => r.operation === 'vector_hop');

		const traces: any[] = [];

		if (writeResults.length > 0) {
			traces.push({
				x: writeResults.map((r) => r.dataSize),
				y: writeResults.map((r) => r.throughput),
				name: 'Write',
				type: 'scatter',
				mode: 'lines+markers',
				marker: { size: 10 }
			});
		}

		if (readResults.length > 0) {
			traces.push({
				x: readResults.map((r) => r.dataSize),
				y: readResults.map((r) => r.throughput),
				name: 'Read',
				type: 'scatter',
				mode: 'lines+markers',
				marker: { size: 10 }
			});
		}

		if (multiHopResults.length > 0) {
			traces.push({
				x: multiHopResults.map((r) => r.metadata.find((m: any) => m.key === 'depth')?.value || 0),
				y: multiHopResults.map((r) => r.throughput),
				name: 'Multi-Hop',
				type: 'scatter',
				mode: 'lines+markers',
				marker: { size: 10 }
			});
		}

		if (vectorHopResults.length > 0) {
			traces.push({
				x: vectorHopResults.map((r) => r.metadata.find((m: any) => m.key === 'depth')?.value || 0),
				y: vectorHopResults.map((r) => r.throughput),
				name: 'Vector Hop',
				type: 'scatter',
				mode: 'lines+markers',
				marker: { size: 10 }
			});
		}

		const layout = {
			title: 'Benchmark Results: Throughput',
			xaxis: { title: 'Data Size / Depth', type: 'log' },
			yaxis: { title: 'Throughput (ops/sec)', type: 'log' },
			hovermode: 'closest'
		};

		Plotly.newPlot(container, traces, layout);
	}

	function formatNumber(num: number): string {
		if (num >= 1000000) {
			return (num / 1000000).toFixed(2) + 'M';
		} else if (num >= 1000) {
			return (num / 1000).toFixed(2) + 'K';
		}
		return num.toFixed(2);
	}
</script>

<div class="benchmark-page">
	<h1>Benchmark</h1>
	<p class="subtitle">Performance measurement for 100-1M data, multi-hop, and vector index</p>

	{#if error}
		<div class="error">Error: {error}</div>
	{/if}

	{#if loading && progress.total > 0}
		<div class="progress-section">
			<div class="progress-header">
				<h3>{progress.message}</h3>
				<span class="progress-percentage">{progress.percentage.toFixed(1)}%</span>
			</div>
			<div class="progress-bar-container">
				<div class="progress-bar" style="width: {progress.percentage}%"></div>
			</div>
			<div class="progress-stats">
				<div class="stat">
					<span class="stat-label">Progress:</span>
					<span class="stat-value">{progress.current.toLocaleString()} / {progress.total.toLocaleString()}</span>
				</div>
				<div class="stat">
					<span class="stat-label">Elapsed:</span>
					<span class="stat-value">{progress.elapsedTime.toFixed(1)}s</span>
				</div>
				{#if progress.estimatedTime > 0}
					<div class="stat">
						<span class="stat-label">Estimated:</span>
						<span class="stat-value">{progress.estimatedTime.toFixed(1)}s</span>
					</div>
				{/if}
			</div>
		</div>
	{/if}

	<div class="benchmark-controls">
		<div class="control-section">
			<h2>Write Benchmark</h2>
			<div class="control-group">
				<label>
					<span>Data Size</span>
					<select bind:value={writeConfig.dataSize}>
						<option value={100}>100</option>
						<option value={1000}>1K</option>
						<option value={10000}>10K</option>
						<option value={100000}>100K</option>
						<option value={1000000}>1M</option>
					</select>
				</label>
			</div>
			<div class="control-group">
				<label>
					<span>Batch Size</span>
					<input type="number" bind:value={writeConfig.batchSize} min="1" max="10000" />
				</label>
			</div>
			<div class="control-group">
				<label>
					<span>Vector Dimension</span>
					<input type="number" bind:value={writeConfig.vectorDim} min="0" max="1024" />
				</label>
			</div>
			<div class="control-group">
				<label>
					<span>Args per Incidence</span>
					<input type="number" bind:value={writeConfig.argsPerIncidence} min="0" max="100" />
				</label>
			</div>
			<button class="run-button" onclick={runWriteBenchmark} disabled={loading}>
				Run Write Benchmark
			</button>
		</div>

		<div class="control-section">
			<h2>Read Benchmark</h2>
			<div class="control-group">
				<label>
					<span>Query Count</span>
					<input type="number" bind:value={readConfig.queryCount} min="1" max="1000000" />
				</label>
			</div>
			<button class="run-button" onclick={runReadBenchmark} disabled={loading}>
				Run Read Benchmark
			</button>
		</div>

		<div class="control-section">
			<h2>Multi-Hop Benchmark</h2>
			<div class="control-group">
				<label>
					<span>Start Nodes</span>
					<input type="number" bind:value={multiHopConfig.startNodes} min="1" max="1000" />
				</label>
			</div>
			<div class="control-group">
				<label>
					<span>Depth</span>
					<input type="number" bind:value={multiHopConfig.depth} min="1" max="10" />
				</label>
			</div>
			<div class="control-group">
				<label>
					<span>Avg Edges</span>
					<input type="number" bind:value={multiHopConfig.avgEdges} min="1" max="100" />
				</label>
			</div>
			<button class="run-button" onclick={runMultiHopBenchmark} disabled={loading}>
				Run Multi-Hop Benchmark
			</button>
		</div>

		<div class="control-section">
			<h2>Vector Hop Benchmark</h2>
			<div class="control-group">
				<label>
					<span>Data Size</span>
					<select bind:value={vectorHopConfig.dataSize}>
						<option value={100}>100</option>
						<option value={1000}>1K</option>
						<option value={10000}>10K</option>
						<option value={100000}>100K</option>
					</select>
				</label>
			</div>
			<div class="control-group">
				<label>
					<span>Vector Dimension</span>
					<input type="number" bind:value={vectorHopConfig.vectorDim} min="1" max="1024" />
				</label>
			</div>
			<div class="control-group">
				<label>
					<span>Depth</span>
					<input type="number" bind:value={vectorHopConfig.depth} min="1" max="10" />
				</label>
			</div>
			<div class="control-group">
				<label>
					<span>K per Hop</span>
					<input type="number" bind:value={vectorHopConfig.kPerHop} min="1" max="1000" />
				</label>
			</div>
			<button class="run-button" onclick={runVectorHopBenchmark} disabled={loading}>
				Run Vector Hop Benchmark
			</button>
		</div>

		<div class="control-section">
			<button class="clear-button" onclick={clearGraph} disabled={loading}>
				Clear Graph
			</button>
		</div>
	</div>

	{#if results.length > 0}
		<div class="results-section">
			<h2>Results</h2>
			<div class="chart-container"></div>
			<div class="results-table">
				<table>
					<thead>
						<tr>
							<th>Operation</th>
							<th>Data Size</th>
							<th>Duration (ms)</th>
							<th>Throughput (ops/sec)</th>
							<th>Latency (ms)</th>
							<th>Timestamp</th>
						</tr>
					</thead>
					<tbody>
						{#each results as result}
							<tr>
								<td>{result.operation}</td>
								<td>{formatNumber(result.dataSize)}</td>
								<td>{result.durationMs.toFixed(2)}</td>
								<td>{formatNumber(result.throughput)}</td>
								<td>{result.latencyMs.toFixed(4)}</td>
								<td>{new Date(result.timestamp).toLocaleTimeString()}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</div>
	{/if}
</div>

<style>
	.benchmark-page {
		padding: 2rem;
		max-width: 1400px;
		margin: 0 auto;
	}

	h1 {
		font-size: 2.5rem;
		margin-bottom: 0.5rem;
		color: #1d1d1f;
	}

	.subtitle {
		color: #86868b;
		margin-bottom: 2rem;
		font-size: 1.1rem;
	}

	.error {
		background: #ff3b30;
		color: white;
		padding: 1rem;
		border-radius: 8px;
		margin-bottom: 1rem;
	}

	.benchmark-controls {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
		gap: 1.5rem;
		margin-bottom: 2rem;
	}

	.control-section {
		background: white;
		border: 1px solid #d2d2d7;
		border-radius: 12px;
		padding: 1.5rem;
	}

	.control-section h2 {
		font-size: 1.3rem;
		margin-bottom: 1rem;
		color: #1d1d1f;
	}

	.control-group {
		margin-bottom: 1rem;
	}

	.control-group label {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.control-group span {
		font-weight: 500;
		color: #1d1d1f;
	}

	.control-group input,
	.control-group select {
		padding: 0.5rem;
		border: 1px solid #d2d2d7;
		border-radius: 6px;
		font-size: 1rem;
	}

	.run-button,
	.clear-button {
		width: 100%;
		padding: 0.75rem;
		border: none;
		border-radius: 8px;
		font-size: 1rem;
		font-weight: 500;
		cursor: pointer;
		transition: background 0.2s;
	}

	.run-button {
		background: #007aff;
		color: white;
	}

	.run-button:hover:not(:disabled) {
		background: #0051d5;
	}

	.run-button:disabled {
		background: #86868b;
		cursor: not-allowed;
	}

	.clear-button {
		background: #ff3b30;
		color: white;
	}

	.clear-button:hover:not(:disabled) {
		background: #d70015;
	}

	.clear-button:disabled {
		background: #86868b;
		cursor: not-allowed;
	}

	.results-section {
		background: white;
		border: 1px solid #d2d2d7;
		border-radius: 12px;
		padding: 1.5rem;
		margin-top: 2rem;
	}

	.results-section h2 {
		font-size: 1.5rem;
		margin-bottom: 1rem;
		color: #1d1d1f;
	}

	.chart-container {
		width: 100%;
		height: 400px;
		margin-bottom: 2rem;
	}

	.results-table {
		overflow-x: auto;
	}

	.results-table table {
		width: 100%;
		border-collapse: collapse;
	}

	.results-table th,
	.results-table td {
		padding: 0.75rem;
		text-align: left;
		border-bottom: 1px solid #d2d2d7;
	}

	.results-table th {
		background: #f5f5f7;
		font-weight: 600;
		color: #1d1d1f;
	}

	.results-table tr:hover {
		background: #f5f5f7;
	}

	.progress-section {
		background: white;
		border: 1px solid #d2d2d7;
		border-radius: 12px;
		padding: 1.5rem;
		margin-bottom: 2rem;
	}

	.progress-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
	}

	.progress-header h3 {
		font-size: 1.1rem;
		color: #1d1d1f;
		margin: 0;
	}

	.progress-percentage {
		font-size: 1.2rem;
		font-weight: 600;
		color: #007aff;
	}

	.progress-bar-container {
		width: 100%;
		height: 24px;
		background: #f5f5f7;
		border-radius: 12px;
		overflow: hidden;
		margin-bottom: 1rem;
	}

	.progress-bar {
		height: 100%;
		background: linear-gradient(90deg, #007aff, #34c759);
		transition: width 0.1s ease-out;
		border-radius: 12px;
	}

	.progress-stats {
		display: flex;
		gap: 2rem;
		flex-wrap: wrap;
	}

	.stat {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.stat-label {
		font-size: 0.875rem;
		color: #86868b;
	}

	.stat-value {
		font-size: 1rem;
		font-weight: 600;
		color: #1d1d1f;
	}
</style>

