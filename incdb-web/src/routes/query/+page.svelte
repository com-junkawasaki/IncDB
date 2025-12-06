<script lang="ts">
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import { executeDatalog, query } from '$lib/graphql/client';
	import type { GraphQLDatalogResult } from '$lib/graphql/types';
	import { sampleQueries, type SampleQuery } from '$lib/sample-queries';

	let queryType: 'datalog' | 'graphql' = 'datalog';
	let datalogQuery = 'Inc(1)\nInc(2)';
	let graphqlQuery = `query {
  incidences {
    id
    level
    typeId
  }
}`;
	let result: GraphQLDatalogResult | null = null;
	let loading = false;
	let error: string | null = null;
	let datalogEditor: any = null;
	let graphqlEditor: any = null;
	let queryHistory: string[] = [];
	let showSamples = false;
	let filteredSamples = sampleQueries;
	let graphqlResult: any = null;

	onMount(async () => {
		// クライアントサイドでのみ Monaco Editor をインポート
		if (!browser) return;

		try {
			const monacoModule = await import('monaco-editor');
			const monaco = monacoModule.default;

			// Monaco Editor を初期化
			const datalogContainer = document.getElementById('datalog-editor');
			const graphqlContainer = document.getElementById('graphql-editor');

			if (datalogContainer) {
				datalogEditor = monaco.editor.create(datalogContainer, {
					value: datalogQuery,
					language: 'plaintext',
					theme: 'vs',
					minimap: { enabled: false },
					scrollBeyondLastLine: false,
				});
			}

			if (graphqlContainer) {
				graphqlEditor = monaco.editor.create(graphqlContainer, {
					value: graphqlQuery,
					language: 'graphql',
					theme: 'vs',
					minimap: { enabled: false },
					scrollBeyondLastLine: false,
				});
			}
		} catch (e) {
			console.error('Failed to load Monaco Editor:', e);
			error = 'Failed to load editor. Please refresh the page.';
		}

		return () => {
			datalogEditor?.dispose();
			graphqlEditor?.dispose();
		};
	});

	async function executeQuery() {
		loading = true;
		error = null;
		result = null;

		try {
			if (queryType === 'datalog') {
				const queryStr = datalogEditor?.getValue() || datalogQuery;
				result = await executeDatalog(queryStr);
				queryHistory.unshift(queryStr);
				if (queryHistory.length > 10) {
					queryHistory = queryHistory.slice(0, 10);
				}
			} else {
				const queryStr = graphqlEditor?.getValue() || graphqlQuery;
				const response = await query<any>(queryStr);
				// GraphQL クエリの結果を処理
				if (response.errors) {
					error = response.errors[0]?.message || 'GraphQL query error';
				} else {
					graphqlResult = response.data;
					// 結果を表示用に変換（incidences がある場合）
					if (response.data?.incidences) {
						result = {
							incidences: response.data.incidences,
							predicateCount: response.data.incidences.length,
						};
					} else {
						// その他の結果を表示
						result = {
							incidences: [],
							predicateCount: 0,
						};
					}
				}
				queryHistory.unshift(queryStr);
				if (queryHistory.length > 10) {
					queryHistory = queryHistory.slice(0, 10);
				}
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Unknown error';
		} finally {
			loading = false;
		}
	}

	function loadHistory(queryStr: string) {
		if (queryType === 'datalog' && datalogEditor) {
			datalogEditor.setValue(queryStr);
		} else if (queryType === 'graphql' && graphqlEditor) {
			graphqlEditor.setValue(queryStr);
		}
	}

	function loadSample(sample: SampleQuery) {
		if (sample.queryType === 'datalog' && datalogEditor) {
			datalogEditor.setValue(sample.query);
			queryType = 'datalog';
		} else if (sample.queryType === 'graphql' && graphqlEditor) {
			graphqlEditor.setValue(sample.query);
			queryType = 'graphql';
		}
		showSamples = false;
	}

	function filterSamples(category: string | null) {
		if (category === null) {
			filteredSamples = sampleQueries;
		} else {
			filteredSamples = sampleQueries.filter((s) => s.category === category);
		}
	}
</script>

<div class="query-page">
	<h1>Query</h1>
	<p class="subtitle">Execute Datalog or GraphQL queries</p>

	<div class="query-tabs">
		<button
			class="tab-button"
			class:active={queryType === 'datalog'}
			onclick={() => (queryType = 'datalog')}
		>
			Datalog
		</button>
		<button
			class="tab-button"
			class:active={queryType === 'graphql'}
			onclick={() => (queryType = 'graphql')}
		>
			GraphQL
		</button>
		<button
			class="samples-button"
			onclick={() => {
				showSamples = !showSamples;
			}}
		>
			{showSamples ? 'Hide' : 'Show'} Samples
		</button>
	</div>

	{#if showSamples}
		<div class="samples-panel">
			<div class="samples-header">
				<h3>Sample Queries</h3>
				<div class="sample-filters">
					<button
						class="filter-button"
						onclick={() => filterSamples(null)}
					>
						All
					</button>
					<button
						class="filter-button"
						onclick={() => filterSamples('sample')}
					>
						Sample Data
					</button>
					<button
						class="filter-button"
						onclick={() => filterSamples('data')}
					>
						Create Data
					</button>
					<button
						class="filter-button"
						onclick={() => filterSamples('query')}
					>
						Queries
					</button>
				</div>
			</div>
			<div class="samples-list">
				{#each filteredSamples as sample}
					<div class="sample-item" onclick={() => loadSample(sample)}>
						<div class="sample-header">
							<span class="sample-name">{sample.name}</span>
							<span class="sample-type">{sample.queryType}</span>
						</div>
						<div class="sample-description">{sample.description}</div>
					</div>
				{/each}
			</div>
		</div>
	{/if}

	<div class="query-content">
		<div class="query-panel">
			<div class="query-header">
				<h2>{queryType === 'datalog' ? 'Datalog' : 'GraphQL'} Query</h2>
				<button class="execute-button" onclick={executeQuery} disabled={loading}>
					{loading ? 'Executing...' : 'Execute'}
				</button>
			</div>
			<div
				id={queryType === 'datalog' ? 'datalog-editor' : 'graphql-editor'}
				class="editor-container"
			></div>
			{#if queryHistory.length > 0}
				<div class="history-section">
					<h3>Query History</h3>
					<div class="history-list">
						{#each queryHistory as historyItem}
							<button class="history-item" onclick={() => loadHistory(historyItem)}>
								{historyItem.substring(0, 50)}...
							</button>
						{/each}
					</div>
				</div>
			{/if}
		</div>

		<div class="result-panel">
			<h2>Results</h2>
			{#if loading}
				<div class="loading">Executing query...</div>
			{:else if error}
				<div class="error">Error: {error}</div>
			{:else if result}
				<div class="result-info">
					Found {result.incidences.length} incidences ({result.predicateCount} predicates)
				</div>
				<div class="results-list">
					{#each result.incidences as inc}
						<div class="result-item">
							<div class="result-header">
								<span class="result-id">ID: {inc.id}</span>
								{#if inc.typeId}
									<span class="result-type">Type: {inc.typeId}</span>
								{/if}
							</div>
							<div class="result-details">
								<div>Level: {inc.level}</div>
								{#if inc.args.length > 0}
									<div>Args: {inc.args.join(', ')}</div>
								{/if}
								{#if inc.roles.length > 0}
									<div>Roles: {inc.roles.join(', ')}</div>
								{/if}
								{#if inc.value}
									<div>
										Value:{' '}
										{inc.value.str ||
											inc.value.int ||
											inc.value.float ||
											inc.value.bool ||
											'N/A'}
									</div>
								{/if}
							</div>
						</div>
					{/each}
				</div>
			{:else if graphqlResult}
				<div class="graphql-result">
					<pre class="result-json">{JSON.stringify(graphqlResult, null, 2)}</pre>
				</div>
			{:else}
				<div class="empty-state">No results yet. Execute a query to see results.</div>
			{/if}
		</div>
	</div>
</div>

<style>
	.query-page {
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

	.query-tabs {
		display: flex;
		gap: 0.5rem;
		margin-bottom: 1.5rem;
		align-items: center;
	}

	.samples-button {
		margin-left: auto;
		padding: 0.75rem 1.5rem;
		background: #34c759;
		color: #ffffff;
		border: none;
		border-radius: 8px;
		font-size: 0.9375rem;
		font-weight: 500;
		cursor: pointer;
		transition: background-color 0.2s;
	}

	.samples-button:hover {
		background: #28a745;
	}

	.samples-panel {
		background: #ffffff;
		border-radius: 12px;
		padding: 1.5rem;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
		margin-bottom: 1.5rem;
	}

	.samples-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
	}

	.samples-header h3 {
		font-size: 1.25rem;
		font-weight: 600;
		color: #1d1d1f;
		margin: 0;
	}

	.sample-filters {
		display: flex;
		gap: 0.5rem;
	}

	.filter-button {
		padding: 0.5rem 1rem;
		background: #f5f5f7;
		border: none;
		border-radius: 6px;
		font-size: 0.875rem;
		font-weight: 500;
		color: #1d1d1f;
		cursor: pointer;
		transition: background-color 0.2s;
	}

	.filter-button:hover {
		background: #e5e5e7;
	}

	.samples-list {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
		gap: 1rem;
	}

	.sample-item {
		padding: 1rem;
		background: #f5f5f7;
		border-radius: 8px;
		cursor: pointer;
		transition: background-color 0.2s, transform 0.2s;
	}

	.sample-item:hover {
		background: #e5e5e7;
		transform: translateY(-2px);
	}

	.sample-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.5rem;
	}

	.sample-name {
		font-weight: 600;
		color: #1d1d1f;
		font-size: 0.9375rem;
	}

	.sample-type {
		font-size: 0.75rem;
		padding: 0.25rem 0.5rem;
		background: #007aff;
		color: #ffffff;
		border-radius: 4px;
		text-transform: uppercase;
	}

	.sample-description {
		font-size: 0.875rem;
		color: #86868b;
	}

	.tab-button {
		padding: 0.75rem 1.5rem;
		background: #f5f5f7;
		border: none;
		border-radius: 8px;
		font-size: 0.9375rem;
		font-weight: 500;
		color: #1d1d1f;
		cursor: pointer;
		transition: background-color 0.2s;
	}

	.tab-button:hover {
		background: #e5e5e7;
	}

	.tab-button.active {
		background: #007aff;
		color: #ffffff;
	}

	.query-content {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 2rem;
	}

	.query-panel,
	.result-panel {
		background: #ffffff;
		border-radius: 12px;
		padding: 1.5rem;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
	}

	.query-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
	}

	.query-header h2 {
		font-size: 1.25rem;
		font-weight: 600;
		color: #1d1d1f;
	}

	.execute-button {
		padding: 0.5rem 1.5rem;
		background: #007aff;
		color: #ffffff;
		border: none;
		border-radius: 8px;
		font-size: 0.9375rem;
		font-weight: 500;
		cursor: pointer;
		transition: background-color 0.2s;
	}

	.execute-button:hover:not(:disabled) {
		background: #0051d5;
	}

	.execute-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.editor-container {
		height: 400px;
		border: 1px solid #e5e5e7;
		border-radius: 8px;
		margin-bottom: 1rem;
	}

	.history-section {
		margin-top: 1rem;
		padding-top: 1rem;
		border-top: 1px solid #e5e5e7;
	}

	.history-section h3 {
		font-size: 0.875rem;
		font-weight: 600;
		color: #86868b;
		margin-bottom: 0.5rem;
	}

	.history-list {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.history-item {
		padding: 0.5rem;
		background: #f5f5f7;
		border: none;
		border-radius: 4px;
		text-align: left;
		font-size: 0.875rem;
		color: #1d1d1f;
		cursor: pointer;
		transition: background-color 0.2s;
	}

	.history-item:hover {
		background: #e5e5e7;
	}

	.result-panel h2 {
		font-size: 1.25rem;
		font-weight: 600;
		color: #1d1d1f;
		margin-bottom: 1rem;
	}

	.loading,
	.error,
	.empty-state {
		padding: 2rem;
		text-align: center;
		color: #86868b;
	}

	.error {
		color: #ff3b30;
		background: #ffebee;
		border-radius: 8px;
	}

	.result-info {
		padding: 0.75rem;
		background: #f5f5f7;
		border-radius: 8px;
		margin-bottom: 1rem;
		font-size: 0.875rem;
		color: #1d1d1f;
	}

	.results-list {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		max-height: 600px;
		overflow-y: auto;
	}

	.result-item {
		padding: 1rem;
		background: #f5f5f7;
		border-radius: 8px;
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

	.result-type {
		font-size: 0.875rem;
		color: #86868b;
	}

	.result-details {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		font-size: 0.875rem;
		color: #86868b;
	}

	.graphql-result {
		max-height: 600px;
		overflow-y: auto;
	}

	.result-json {
		background: #f5f5f7;
		padding: 1rem;
		border-radius: 8px;
		font-family: 'Monaco', 'Courier New', monospace;
		font-size: 0.875rem;
		color: #1d1d1f;
		overflow-x: auto;
		margin: 0;
	}

	@media (max-width: 1024px) {
		.query-content {
			grid-template-columns: 1fr;
		}

		.editor-container {
			height: 300px;
		}
	}
</style>

