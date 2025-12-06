<script lang="ts">
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import { getGraphStructure, getIncidences } from '$lib/graphql/client';
	import type { GraphQLGraphStructure } from '$lib/graphql/types';
	import { convertToSvelteFlow } from '$lib/graph/utils';
	import { SvelteFlow, Background, Controls, MiniMap } from '@xyflow/svelte';
	import '@xyflow/svelte/dist/style.css';

	let viewMode: 'force' | 'hierarchical' = 'force';
	let selectedIds: string[] = [];
	let depth = 2;
	let structure: GraphQLGraphStructure | null = null;
	let loading = false;
	let error: string | null = null;
	let nodes: any[] = [];
	let edges: any[] = [];

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
			const converted = convertToSvelteFlow(structure.nodes, structure.edges);
			nodes = converted.nodes;
			edges = converted.edges;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Unknown error';
			console.error('Error loading graph:', e);
		} finally {
			loading = false;
		}
	}

	async function loadDemoData() {
		loading = true;
		error = null;
		
		try {
			// すべての Incidence を取得して、Type 以外の ID を抽出
			const incidences = await getIncidences();
			const demoIds = incidences
				.filter((inc) => {
					// Type を除外（value.str が "Person", "CryptoAddress", "Transaction" でないもの）
					const typeNames = ['Person', 'CryptoAddress', 'Transaction'];
					return inc.value?.str && !typeNames.includes(inc.value.str);
				})
				.map((inc) => inc.id);
			
			if (demoIds.length === 0) {
				error = 'No demo data found. Please load crypto investigation data first.';
				return;
			}

			selectedIds = demoIds.slice(0, 20); // 最初の20個に制限
			depth = 3;
			await loadGraph();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Unknown error';
			console.error('Error loading demo data:', e);
		} finally {
			loading = false;
		}
	}

	function handleIdsInput(e: Event) {
		const target = e.target as HTMLInputElement;
		selectedIds = target.value
			.split(',')
			.map((id) => id.trim())
			.filter((id) => id.length > 0);
	}

	function handleDepthInput(e: Event) {
		const target = e.target as HTMLInputElement;
		depth = parseInt(target.value, 10) || 2;
	}
</script>

<div class="graph-page">
	<h1>Graph Visualization</h1>
	<p class="subtitle">Visualize incidence relationships</p>

	<div class="controls">
		<div class="control-group">
			<label>
				<span>IDs (comma-separated)</span>
				<input
					type="text"
					value={selectedIds.join(', ')}
					oninput={handleIdsInput}
					placeholder="4, 5, 6, 7"
				/>
			</label>
		</div>
		<div class="control-group">
			<label>
				<span>Depth</span>
				<input type="number" value={depth} oninput={handleDepthInput} min="1" max="10" />
			</label>
		</div>
		<div class="control-group">
			<label>
				<span>View Mode</span>
				<select bind:value={viewMode}>
					<option value="force">Force-directed</option>
					<option value="hierarchical">Hierarchical</option>
				</select>
			</label>
		</div>
		<button class="load-button" onclick={loadGraph} disabled={loading}>
			{loading ? 'Loading...' : 'Load Graph'}
		</button>
		<button class="demo-button" onclick={loadDemoData} disabled={loading}>
			🚀 Load Demo Data
		</button>
	</div>

	{#if error}
		<div class="error">Error: {error}</div>
	{/if}

	{#if structure}
		<div class="stats">
			<span class="info-label">Nodes:</span>
			<span class="info-value">{nodes.length}</span>
			<span class="info-label">Edges:</span>
			<span class="info-value">{edges.length}</span>
		</div>
	{/if}

	<div class="graph-container">
		{#if nodes.length > 0 && browser}
			<SvelteFlow {nodes} {edges} class="svelte-flow">
				<Background />
				<Controls />
				<MiniMap />
			</SvelteFlow>
		{:else if !browser}
			<div class="empty-state">
				<p>Loading...</p>
			</div>
		{:else}
			<div class="empty-state">
				<p>Enter IDs and click "Load Graph" to visualize</p>
			</div>
		{/if}
	</div>
</div>

<style>
	.graph-page {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
		padding: 2rem;
		max-width: 100%;
	}

	h1 {
		font-size: 2rem;
		font-weight: 700;
		color: #1d1d1f;
		margin: 0;
	}

	.subtitle {
		font-size: 1rem;
		color: #86868b;
		margin: 0;
	}

	.controls {
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

	.control-group input:focus,
	.control-group select:focus {
		outline: none;
		border-color: #007aff;
		box-shadow: 0 0 0 3px rgba(0, 122, 255, 0.1);
	}

	.load-button,
	.demo-button {
		padding: 0.75rem 1.5rem;
		border: none;
		border-radius: 8px;
		font-size: 0.9375rem;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s;
		font-family: inherit;
	}

	.load-button {
		background: #007aff;
		color: #ffffff;
	}

	.load-button:hover:not(:disabled) {
		background: #0051d5;
	}

	.load-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.demo-button {
		background: #34c759;
		color: #ffffff;
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
		background: #ff3b30;
		color: #ffffff;
		border-radius: 8px;
		font-size: 0.9375rem;
	}

	.stats {
		display: flex;
		gap: 1rem;
		padding: 0.75rem 1rem;
		background: #f5f5f7;
		border-radius: 8px;
		font-size: 0.875rem;
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
		padding: 0;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
		min-height: 600px;
		height: 70vh;
		position: relative;
		overflow: hidden;
	}

	:global(.svelte-flow) {
		width: 100%;
		height: 100%;
	}

	.empty-state {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		min-height: 400px;
		color: #86868b;
		font-size: 1rem;
	}
</style>
