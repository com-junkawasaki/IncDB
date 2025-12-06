<script lang="ts">
	import { onMount } from 'svelte';

	interface View {
		id: string;
		name: string;
		query: string;
		queryType: 'datalog' | 'graphql';
		createdAt: string;
	}

	let views: View[] = [];
	let selectedView: View | null = null;
	let showCreateModal = false;
	let newViewName = '';
	let newViewQuery = '';
	let newViewType: 'datalog' | 'graphql' = 'datalog';

	onMount(() => {
		// ローカルストレージからビューを読み込み
		const saved = localStorage.getItem('incdb-views');
		if (saved) {
			views = JSON.parse(saved);
		}
	});

	function saveViews() {
		localStorage.setItem('incdb-views', JSON.stringify(views));
	}

	function createView() {
		if (!newViewName.trim()) return;

		const view: View = {
			id: Date.now().toString(),
			name: newViewName,
			query: newViewQuery,
			queryType: newViewType,
			createdAt: new Date().toISOString(),
		};

		views.push(view);
		saveViews();
		showCreateModal = false;
		newViewName = '';
		newViewQuery = '';
	}

	function deleteView(id: string) {
		views = views.filter((v) => v.id !== id);
		saveViews();
		if (selectedView?.id === id) {
			selectedView = null;
		}
	}

	function formatDate(dateString: string): string {
		return new Date(dateString).toLocaleDateString();
	}
</script>

<div class="view-page">
	<h1>Views</h1>
	<p class="subtitle">Saved queries and views</p>

	<div class="views-content">
		<div class="views-panel">
			<div class="views-header">
				<h2>Saved Views ({views.length})</h2>
				<button class="create-button" onclick={() => (showCreateModal = true)}>
					+ New View
				</button>
			</div>
			<div class="views-list">
				{#if views.length === 0}
					<div class="empty-state">No views yet. Create a new view to get started.</div>
				{:else}
					{#each views as view}
						<div
							class="view-item"
							class:active={selectedView?.id === view.id}
							onclick={() => (selectedView = view)}
						>
							<div class="view-header">
								<span class="view-name">{view.name}</span>
								<button
									class="delete-button"
									onclick|stopPropagation={() => deleteView(view.id)}
								>
									×
								</button>
							</div>
							<div class="view-meta">
								<span class="view-type">{view.queryType}</span>
								<span class="view-date">{formatDate(view.createdAt)}</span>
							</div>
						</div>
					{/each}
				{/if}
			</div>
		</div>

		<div class="view-detail-panel">
			{#if selectedView}
				<h2>{selectedView.name}</h2>
				<div class="view-info">
					<div class="info-row">
						<span class="info-label">Type:</span>
						<span class="info-value">{selectedView.queryType}</span>
					</div>
					<div class="info-row">
						<span class="info-label">Created:</span>
						<span class="info-value">{formatDate(selectedView.createdAt)}</span>
					</div>
				</div>
				<div class="view-query">
					<h3>Query</h3>
					<pre class="query-code">{selectedView.query}</pre>
				</div>
				<div class="view-actions">
					<a href="/query" class="action-button">Execute</a>
					<button class="action-button secondary">Edit</button>
				</div>
			{:else}
				<div class="empty-state">
					<p>Select a view to see details</p>
				</div>
			{/if}
		</div>
	</div>
</div>

{#if showCreateModal}
	<div class="modal-overlay" onclick={() => (showCreateModal = false)}>
		<div class="modal" onclick|stopPropagation>
			<h2>Create New View</h2>
			<div class="modal-content">
				<label>
					<span>Name</span>
					<input type="text" bind:value={newViewName} placeholder="View name" />
				</label>
				<label>
					<span>Query Type</span>
					<select bind:value={newViewType}>
						<option value="datalog">Datalog</option>
						<option value="graphql">GraphQL</option>
					</select>
				</label>
				<label>
					<span>Query</span>
					<textarea bind:value={newViewQuery} placeholder="Enter query..." rows="10"></textarea>
				</label>
			</div>
			<div class="modal-actions">
				<button class="button secondary" onclick={() => (showCreateModal = false)}>Cancel</button>
				<button class="button primary" onclick={createView}>Create</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.view-page {
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

	.views-content {
		display: grid;
		grid-template-columns: 400px 1fr;
		gap: 2rem;
	}

	.views-panel {
		background: #ffffff;
		border-radius: 12px;
		padding: 1.5rem;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
		max-height: calc(100vh - 200px);
		overflow-y: auto;
	}

	.views-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
	}

	.views-header h2 {
		font-size: 1.25rem;
		font-weight: 600;
		color: #1d1d1f;
	}

	.create-button {
		padding: 0.5rem 1rem;
		background: #007aff;
		color: #ffffff;
		border: none;
		border-radius: 8px;
		font-size: 0.875rem;
		font-weight: 500;
		cursor: pointer;
		transition: background-color 0.2s;
	}

	.create-button:hover {
		background: #0051d5;
	}

	.views-list {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.view-item {
		padding: 1rem;
		background: #f5f5f7;
		border-radius: 8px;
		cursor: pointer;
		transition: background-color 0.2s;
	}

	.view-item:hover {
		background: #e5e5e7;
	}

	.view-item.active {
		background: #007aff;
		color: #ffffff;
	}

	.view-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.5rem;
	}

	.view-name {
		font-weight: 600;
		font-size: 0.9375rem;
	}

	.delete-button {
		background: transparent;
		border: none;
		color: inherit;
		font-size: 1.5rem;
		cursor: pointer;
		opacity: 0.7;
		transition: opacity 0.2s;
	}

	.delete-button:hover {
		opacity: 1;
	}

	.view-meta {
		display: flex;
		justify-content: space-between;
		font-size: 0.75rem;
		opacity: 0.8;
	}

	.view-type {
		text-transform: uppercase;
	}

	.view-detail-panel {
		background: #ffffff;
		border-radius: 12px;
		padding: 2rem;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
		max-height: calc(100vh - 200px);
		overflow-y: auto;
	}

	.view-detail-panel h2 {
		font-size: 1.5rem;
		font-weight: 600;
		color: #1d1d1f;
		margin-bottom: 1rem;
	}

	.view-info {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
		margin-bottom: 1.5rem;
		padding-bottom: 1.5rem;
		border-bottom: 1px solid #e5e5e7;
	}

	.info-row {
		display: flex;
		gap: 1rem;
	}

	.info-label {
		font-weight: 600;
		color: #86868b;
		min-width: 100px;
	}

	.info-value {
		color: #1d1d1f;
	}

	.view-query h3 {
		font-size: 1rem;
		font-weight: 600;
		color: #1d1d1f;
		margin-bottom: 0.75rem;
	}

	.query-code {
		background: #f5f5f7;
		padding: 1rem;
		border-radius: 8px;
		font-family: 'Monaco', 'Courier New', monospace;
		font-size: 0.875rem;
		overflow-x: auto;
		color: #1d1d1f;
	}

	.view-actions {
		display: flex;
		gap: 1rem;
		margin-top: 1.5rem;
	}

	.action-button {
		padding: 0.75rem 1.5rem;
		background: #007aff;
		color: #ffffff;
		border: none;
		border-radius: 8px;
		font-size: 0.9375rem;
		font-weight: 500;
		text-decoration: none;
		cursor: pointer;
		transition: background-color 0.2s;
		display: inline-block;
	}

	.action-button:hover {
		background: #0051d5;
	}

	.action-button.secondary {
		background: #f5f5f7;
		color: #1d1d1f;
	}

	.action-button.secondary:hover {
		background: #e5e5e7;
	}

	.empty-state {
		text-align: center;
		padding: 4rem 2rem;
		color: #86868b;
	}

	.modal-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: rgba(0, 0, 0, 0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal {
		background: #ffffff;
		border-radius: 12px;
		padding: 2rem;
		max-width: 600px;
		width: 90%;
		max-height: 90vh;
		overflow-y: auto;
	}

	.modal h2 {
		font-size: 1.5rem;
		font-weight: 600;
		color: #1d1d1f;
		margin-bottom: 1.5rem;
	}

	.modal-content {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		margin-bottom: 1.5rem;
	}

	.modal-content label {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.modal-content label span {
		font-weight: 500;
		color: #1d1d1f;
		font-size: 0.875rem;
	}

	.modal-content input,
	.modal-content select,
	.modal-content textarea {
		padding: 0.75rem;
		border: 1px solid #e5e5e7;
		border-radius: 8px;
		font-size: 0.9375rem;
		font-family: inherit;
	}

	.modal-content textarea {
		resize: vertical;
		font-family: 'Monaco', 'Courier New', monospace;
	}

	.modal-actions {
		display: flex;
		justify-content: flex-end;
		gap: 1rem;
	}

	.button {
		padding: 0.75rem 1.5rem;
		border: none;
		border-radius: 8px;
		font-size: 0.9375rem;
		font-weight: 500;
		cursor: pointer;
		transition: background-color 0.2s;
	}

	.button.primary {
		background: #007aff;
		color: #ffffff;
	}

	.button.primary:hover {
		background: #0051d5;
	}

	.button.secondary {
		background: #f5f5f7;
		color: #1d1d1f;
	}

	.button.secondary:hover {
		background: #e5e5e7;
	}

	@media (max-width: 1024px) {
		.views-content {
			grid-template-columns: 1fr;
		}

		.views-panel {
			max-height: 400px;
		}
	}
</style>

