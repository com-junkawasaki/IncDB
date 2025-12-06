<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { browser } from '$app/environment';
	import { getGraphStructure } from '$lib/graphql/client';
	import type { GraphQLGraphStructure } from '$lib/graphql/types';
	import { convertToVisNetwork, getNodeColor } from '$lib/graph/utils';

	let viewMode: 'force' | 'hierarchical' = 'force';
	let selectedIds: string[] = [];
	let depth = 2;
	let structure: GraphQLGraphStructure | null = null;
	let loading = false;
	let error: string | null = null;
	let network: any = null;
	let networkContainer: HTMLDivElement | null = null;
	let Network: any = null;

	onMount(async () => {
		// クライアントサイドでのみ vis-network をインポート
		if (browser) {
			const visNetworkModule = await import('vis-network');
			Network = visNetworkModule.Network;
		}
	});

	onDestroy(() => {
		if (network) {
			network.destroy();
		}
	});

	function initializeNetwork() {
		if (!networkContainer || !structure || !Network) return;

		const { nodes, edges } = convertToVisNetwork(structure.nodes, structure.edges);

		const data = { nodes, edges };
		const options = {
			layout: {
				hierarchical: viewMode === 'hierarchical',
			},
			physics: {
				enabled: viewMode === 'force',
			},
			interaction: {
				dragNodes: true,
				zoomView: true,
			},
		};

		network = new Network(networkContainer, data, options);
	}

	async function loadGraph() {
		if (selectedIds.length === 0) {
			error = 'Please enter at least one ID';
			return;
		}

		loading = true;
		error = null;
		structure = null;

		try {
			structure = await getGraphStructure(selectedIds, depth);
			if (networkContainer) {
				initializeNetwork();
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Unknown error';
		} finally {
			loading = false;
		}
	}

	function updateViewMode() {
		if (structure && networkContainer) {
			initializeNetwork();
		}
	}
</script>

<div class="graph-page">
	<h1>Graph Visualization</h1>
	<p class="subtitle">Visualize incidence relationships</p>

	<div class="graph-controls">
		<div class="control-group">
			<label>
				<span>IDs (comma-separated)</span>
				<input
					type="text"
					placeholder="1, 2, 3"
					value={selectedIds.join(', ')}
					oninput={(e) => {
						selectedIds = (e.target as HTMLInputElement).value
							.split(',')
							.map((s) => s.trim())
							.filter((s) => s.length > 0);
					}}
				/>
			</label>
		</div>
		<div class="control-group">
			<label>
				<span>Depth</span>
				<input type="number" bind:value={depth} min="1" max="5" />
			</label>
		</div>
		<div class="control-group">
			<label>
				<span>View Mode</span>
				<select bind:value={viewMode} onchange={updateViewMode}>
					<option value="force">Force-directed</option>
					<option value="hierarchical">Hierarchical</option>
				</select>
			</label>
		</div>
		<button class="load-button" onclick={loadGraph} disabled={loading}>
			{loading ? 'Loading...' : 'Load Graph'}
		</button>
	</div>

	{#if error}
		<div class="error">Error: {error}</div>
	{/if}

	{#if structure}
		<div class="graph-info">
			<div class="info-item">
				<span class="info-label">Nodes:</span>
				<span class="info-value">{structure.nodes.length}</span>
			</div>
			<div class="info-item">
				<span class="info-label">Edges:</span>
				<span class="info-value">{structure.edges.length}</span>
			</div>
		</div>
	{/if}

	<div class="graph-container">
		{#if structure}
			<div bind:this={networkContainer} class="network-canvas"></div>
		{:else}
			<div class="empty-state">
				<p>Enter IDs and click "Load Graph" to visualize</p>
			</div>
		{/if}
	</div>
</div>

<style>
	.graph-page {
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

	.graph-controls {
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

	.load-button {
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

	.load-button:hover:not(:disabled) {
		background: #0051d5;
	}

	.load-button:disabled {
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

	.graph-info {
		display: flex;
		gap: 2rem;
		margin-bottom: 1rem;
		padding: 1rem;
		background: #f5f5f7;
		border-radius: 8px;
	}

	.info-item {
		display: flex;
		gap: 0.5rem;
	}

	.info-label {
		font-weight: 600;
		color: #86868b;
	}

	.info-value {
		color: #1d1d1f;
	}

	.graph-container {
		background: #ffffff;
		border-radius: 12px;
		padding: 1rem;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
		min-height: 600px;
	}

	.network-canvas {
		width: 100%;
		height: 600px;
		border: 1px solid #e5e5e7;
		border-radius: 8px;
	}

	.empty-state {
		text-align: center;
		padding: 4rem 2rem;
		color: #86868b;
	}

	@media (max-width: 768px) {
		.graph-controls {
			flex-direction: column;
		}

		.control-group {
			min-width: 100%;
		}

		.network-canvas {
			height: 400px;
		}
	}
</style>

