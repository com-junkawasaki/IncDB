<script lang="ts">
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import { vectorSearch, getIncidences } from '$lib/graphql/client';
	import type { GraphQLVectorSearchItem } from '$lib/graphql/types';
	import { performPCA, calculateSimilarityMatrix, cosineSimilarity } from '$lib/vector/utils';

	let visualizationMode: 'tsne' | 'pca' | 'similarity' = 'tsne';
	let queryVector: number[] = [];
	let k = 10;
	let searchResults: GraphQLVectorSearchItem[] = [];
	let allVectors: { id: string; vector: number[] }[] = [];
	let loading = false;
	let error: string | null = null;
	let plotContainer: HTMLDivElement | null = null;
	let Plotly: any = null;

	onMount(async () => {
		// クライアントサイドでのみ Plotly を読み込み（CDN から）
		if (browser) {
			try {
				// CDN から Plotly を読み込む
				if (!(window as any).Plotly) {
					const script = document.createElement('script');
					script.src = 'https://cdn.plot.ly/plotly-2.27.1.min.js';
					script.onload = () => {
						Plotly = (window as any).Plotly;
						console.log('Plotly loaded from CDN');
					};
					script.onerror = () => {
						console.error('Failed to load Plotly from CDN');
						error = 'Failed to load visualization library. Please check your internet connection.';
					};
					document.head.appendChild(script);
				} else {
					Plotly = (window as any).Plotly;
					console.log('Plotly already loaded');
				}
			} catch (e) {
				console.error('Failed to load Plotly:', e);
				error = 'Failed to load visualization library.';
			}
		}

		try {
			const incidences = await getIncidences();
			allVectors = incidences
				.filter((inc) => inc.embedding && inc.embedding.length > 0)
				.map((inc) => ({
					id: inc.id,
					vector: inc.embedding!,
				}));
		} catch (e) {
			error = e instanceof Error ? e.message : 'Unknown error';
		}
	});

	async function performSearch() {
		if (queryVector.length === 0) {
			error = 'Please enter a query vector';
			return;
		}

		loading = true;
		error = null;
		searchResults = [];

		try {
			const result = await vectorSearch(queryVector, k);
			searchResults = result.incidences;
			updateVisualization();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Unknown error';
		} finally {
			loading = false;
		}
	}

	function updateVisualization() {
		if (!plotContainer) return;

		const vectors = searchResults.length > 0
			? searchResults.map((item) => item.incidence.embedding!).filter((v) => v !== null)
			: allVectors.map((v) => v.vector);

		if (vectors.length === 0) return;

		if (visualizationMode === 'similarity') {
			renderSimilarityMatrix();
		} else {
			renderDimensionReduction();
		}
	}

	function renderDimensionReduction() {
		if (!plotContainer || !Plotly) return;

		const vectors = searchResults.length > 0
			? searchResults.map((item) => item.incidence.embedding!).filter((v) => v !== null && v.length > 0)
			: allVectors.map((v) => v.vector).filter((v) => v.length > 0);

		if (vectors.length === 0) return;

		// PCA を使用（t-SNE は計算コストが高いため、簡易的に PCA を使用）
		const projected = performPCA(vectors, 2);

		const x = projected.map((p) => p[0]);
		const y = projected.map((p) => p[1]);

		const trace = {
			x,
			y,
			mode: 'markers',
			type: 'scatter',
			text: searchResults.length > 0
				? searchResults.map((item) => `ID: ${item.incidence.id}<br>Similarity: ${item.similarity.toFixed(3)}`)
				: allVectors.map((v) => `ID: ${v.id}`),
			marker: {
				size: 10,
				color: searchResults.length > 0
					? searchResults.map((item) => item.similarity)
					: undefined,
				colorscale: 'Viridis',
			},
		} as Plotly.PlotData;

		const layout = {
			title: visualizationMode === 'tsne' ? 't-SNE Visualization (PCA-based)' : 'PCA Visualization',
			xaxis: { title: 'PC1' },
			yaxis: { title: 'PC2' },
		};

		Plotly.newPlot(plotContainer, [trace], layout);
	}

	function renderSimilarityMatrix() {
		if (!plotContainer || !Plotly || searchResults.length === 0) return;

		const vectors = searchResults.map((item) => item.incidence.embedding!).filter((v) => v !== null && v.length > 0);
		if (vectors.length === 0) return;

		const matrix = calculateSimilarityMatrix(vectors);

		const trace = {
			z: matrix,
			type: 'heatmap',
			colorscale: 'Viridis',
		} as Plotly.PlotData;

		const layout = {
			title: 'Similarity Matrix',
			xaxis: { title: 'Vector Index' },
			yaxis: { title: 'Vector Index' },
		};

		Plotly.newPlot(plotContainer, [trace], layout);
	}

	async function loadDemoData() {
		loading = true;
		error = null;
		
		try {
			// すべての Incidence を取得
			const incidences = await getIncidences();
			allVectors = incidences
				.filter((inc) => inc.embedding && inc.embedding.length > 0)
				.map((inc) => ({
					id: inc.id,
					vector: inc.embedding!,
				}));
			
			if (allVectors.length === 0) {
				error = 'No vectors found. Please load crypto investigation data first.';
				loading = false;
				return;
			}
			
			// 高リスクスコアのベクトルをクエリとして使用（容疑者のベクトル）
			const highRiskVector = allVectors.find((v) => v.vector[0] > 0.8)?.vector || allVectors[0].vector;
			queryVector = highRiskVector;
			
			// 自動的に検索と可視化を実行
			await performSearch();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load demo data';
			loading = false;
		}
	}
</script>

<div class="vector-page">
	<h1>Vector Visualization</h1>
	<p class="subtitle">Visualize vector embeddings and similarity</p>

	<div class="vector-controls">
		<div class="control-group">
			<label>
				<span>Query Vector (comma-separated)</span>
				<input
					type="text"
					placeholder="0.1, 0.2, 0.3"
					value={queryVector.join(', ')}
					oninput={(e) => {
						const value = (e.target as HTMLInputElement).value;
						queryVector = value
							.split(',')
							.map((s) => parseFloat(s.trim()))
							.filter((n) => !isNaN(n));
					}}
				/>
			</label>
		</div>
		<div class="control-group">
			<label>
				<span>K (number of results)</span>
				<input type="number" bind:value={k} min="1" max="100" />
			</label>
		</div>
		<div class="control-group">
			<label>
				<span>Visualization Mode</span>
				<select bind:value={visualizationMode} onchange={updateVisualization}>
					<option value="tsne">t-SNE</option>
					<option value="pca">PCA</option>
					<option value="similarity">Similarity Matrix</option>
				</select>
			</label>
		</div>
		<button class="search-button" onclick={performSearch} disabled={loading}>
			{loading ? 'Searching...' : 'Search'}
		</button>
		<button class="demo-button" onclick={loadDemoData} disabled={loading}>
			🚀 Load Demo Data
		</button>
	</div>

	{#if error}
		<div class="error">Error: {error}</div>
	{/if}

	{#if searchResults.length > 0}
		<div class="results-summary">
			Found {searchResults.length} similar vectors
		</div>
	{/if}

	<div class="visualization-container">
		<div bind:this={plotContainer} class="plot-canvas"></div>
	</div>

	{#if searchResults.length > 0}
		<div class="results-list">
			<h2>Search Results</h2>
			{#each searchResults as item}
				<div class="result-item">
					<div class="result-header">
						<span class="result-id">ID: {item.incidence.id}</span>
						<span class="result-similarity">Similarity: {item.similarity.toFixed(4)}</span>
					</div>
					{#if item.incidence.value}
						<div class="result-value">
							Value:{' '}
							{item.incidence.value.str ||
								item.incidence.value.int ||
								item.incidence.value.float ||
								item.incidence.value.bool ||
								'N/A'}
						</div>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.vector-page {
		max-width: 1400px;
		margin: 0 auto;
	}

	h1 {
		font-size: 2rem;
		font-weight: 600;
		color: #1d1d1f;
		margin-bottom: 0.5rem;
	}

	.subtitle {
		font-size: 1rem;
		color: #86868b;
		margin-bottom: 2rem;
	}

	.vector-controls {
		display: flex;
		gap: 1rem;
		margin-bottom: 1.5rem;
		flex-wrap: wrap;
		align-items: flex-end;
	}

	.control-group {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		flex: 1;
		min-width: 200px;
	}

	.control-group label span {
		font-size: 0.875rem;
		font-weight: 500;
		color: #1d1d1f;
	}

	.control-group input,
	.control-group select {
		padding: 0.75rem;
		border: 1px solid #e5e5e7;
		border-radius: 8px;
		font-size: 0.9375rem;
		font-family: inherit;
	}

	.search-button {
		padding: 0.75rem 1.5rem;
		background: #007aff;
		color: #ffffff;
		border: none;
		border-radius: 8px;
		font-size: 0.9375rem;
		font-weight: 500;
		cursor: pointer;
		transition: background-color 0.2s;
		height: fit-content;
	}

	.search-button:hover:not(:disabled) {
		background: #0051d5;
	}

	.search-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.demo-button {
		padding: 0.75rem 1.5rem;
		background: #34c759;
		color: #ffffff;
		border: none;
		border-radius: 8px;
		font-size: 0.9375rem;
		font-weight: 500;
		cursor: pointer;
		transition: background-color 0.2s;
		height: fit-content;
		margin-left: 0.5rem;
	}

	.demo-button:hover:not(:disabled) {
		background: #28a745;
	}

	.demo-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.error {
		padding: 1rem;
		background: #ffebee;
		color: #ff3b30;
		border-radius: 8px;
		margin-bottom: 1rem;
	}

	.results-summary {
		padding: 1rem;
		background: #f5f5f7;
		border-radius: 8px;
		margin-bottom: 1rem;
		font-size: 0.875rem;
		color: #1d1d1f;
	}

	.visualization-container {
		background: #ffffff;
		border-radius: 12px;
		padding: 1rem;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
		margin-bottom: 2rem;
	}

	.plot-canvas {
		width: 100%;
		height: 600px;
	}

	.results-list {
		background: #ffffff;
		border-radius: 12px;
		padding: 1.5rem;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
	}

	.results-list h2 {
		font-size: 1.25rem;
		font-weight: 600;
		color: #1d1d1f;
		margin-bottom: 1rem;
	}

	.result-item {
		padding: 1rem;
		background: #f5f5f7;
		border-radius: 8px;
		margin-bottom: 0.5rem;
	}

	.result-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.5rem;
	}

	.result-id {
		font-weight: 600;
		color: #1d1d1f;
	}

	.result-similarity {
		font-size: 0.875rem;
		color: #007aff;
		font-weight: 600;
	}

	.result-value {
		font-size: 0.875rem;
		color: #86868b;
	}

	@media (max-width: 768px) {
		.vector-controls {
			flex-direction: column;
		}

		.control-group {
			min-width: 100%;
		}

		.plot-canvas {
			height: 400px;
		}
	}
</style>

