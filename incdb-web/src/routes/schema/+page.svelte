<script lang="ts">
	import { onMount } from 'svelte';
	import { getTypes, getType, getSchemaStats } from '$lib/graphql/client';
	import type { GraphQLTypeInfo, GraphQLTypeDetail, GraphQLSchemaStats } from '$lib/graphql/types';

	let types: GraphQLTypeInfo[] = [];
	let selectedType: GraphQLTypeDetail | null = null;
	let stats: GraphQLSchemaStats | null = null;
	let loading = true;
	let error: string | null = null;

	onMount(async () => {
		try {
			[types, stats] = await Promise.all([getTypes(), getSchemaStats()]);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Unknown error';
		} finally {
			loading = false;
		}
	});

	async function selectType(typeId: string) {
		try {
			selectedType = await getType(typeId);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Unknown error';
		}
	}
</script>

<div class="schema-page">
	<h1>Schema</h1>
	<p class="subtitle">Type definitions and structure patterns</p>

	{#if loading}
		<div class="loading">Loading...</div>
	{:else if error}
		<div class="error">Error: {error}</div>
	{:else}
		<div class="schema-content">
			<div class="types-panel">
				<h2>Types ({types.length})</h2>
				{#if stats}
					<div class="stats-summary">
						<div class="stat-item">
							<span class="stat-label">Total:</span>
							<span class="stat-value">{stats.totalIncidences}</span>
						</div>
						<div class="stat-item">
							<span class="stat-label">Types:</span>
							<span class="stat-value">{stats.typeCount}</span>
						</div>
						<div class="stat-item">
							<span class="stat-label">Roles:</span>
							<span class="stat-value">{stats.roleCount}</span>
						</div>
					</div>
				{/if}
				<div class="types-list">
					{#each types as type}
						<button
							class="type-item"
							class:active={selectedType?.id === type.id}
							onclick={() => selectType(type.id)}
						>
							<div class="type-header">
								<span class="type-id">{type.id}</span>
								<span class="type-level">Level {type.level}</span>
							</div>
							<div class="type-count">{type.incidenceCount} incidences</div>
						</button>
					{/each}
				</div>
			</div>

			<div class="type-detail-panel">
				{#if selectedType}
					<h2>Type Details</h2>
					<div class="type-info">
						<div class="info-row">
							<span class="info-label">ID:</span>
							<span class="info-value">{selectedType.id}</span>
						</div>
						<div class="info-row">
							<span class="info-label">Level:</span>
							<span class="info-value">{selectedType.level}</span>
						</div>
						<div class="info-row">
							<span class="info-label">Incidence Count:</span>
							<span class="info-value">{selectedType.incidenceCount}</span>
						</div>
					</div>

					{#if selectedType.structurePatterns.length > 0}
						<h3>Structure Patterns</h3>
						<div class="patterns-list">
							{#each selectedType.structurePatterns as pattern}
								<div class="pattern-item">
									<span class="pattern-pattern">{pattern.pattern}</span>
									<span class="pattern-count">{pattern.count}</span>
								</div>
							{/each}
						</div>
					{/if}

					{#if selectedType.incidences.length > 0}
						<h3>Incidences ({selectedType.incidences.length})</h3>
						<div class="incidences-list">
							{#each selectedType.incidences.slice(0, 10) as inc}
								<div class="incidence-item">
									<div class="incidence-header">
										<span class="incidence-id">{inc.id}</span>
										{#if inc.typeId}
											<span class="incidence-type">Type: {inc.typeId}</span>
										{/if}
									</div>
									{#if inc.args.length > 0}
										<div class="incidence-args">
											Args: {inc.args.join(', ')}
										</div>
									{/if}
									{#if inc.value}
										<div class="incidence-value">
											Value: {inc.value.str || inc.value.int || inc.value.float || inc.value.bool || 'N/A'}
										</div>
									{/if}
								</div>
							{/each}
							{#if selectedType.incidences.length > 10}
								<div class="more-indicator">
									+{selectedType.incidences.length - 10} more
								</div>
							{/if}
						</div>
					{/if}
				{:else}
					<div class="empty-state">
						<p>Select a type to view details</p>
					</div>
				{/if}
			</div>
		</div>
	{/if}
</div>

<style>
	.schema-page {
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

	.loading,
	.error {
		padding: 2rem;
		text-align: center;
	}

	.error {
		color: #ff3b30;
		background: #ffebee;
		border-radius: 8px;
	}

	.schema-content {
		display: grid;
		grid-template-columns: 400px 1fr;
		gap: 2rem;
	}

	.types-panel {
		background: #ffffff;
		border-radius: 12px;
		padding: 1.5rem;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
		max-height: calc(100vh - 200px);
		overflow-y: auto;
	}

	.types-panel h2 {
		font-size: 1.25rem;
		font-weight: 600;
		color: #1d1d1f;
		margin-bottom: 1rem;
	}

	.stats-summary {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		margin-bottom: 1.5rem;
		padding-bottom: 1.5rem;
		border-bottom: 1px solid #e5e5e7;
	}

	.stat-item {
		display: flex;
		justify-content: space-between;
		font-size: 0.875rem;
	}

	.stat-label {
		color: #86868b;
	}

	.stat-value {
		font-weight: 600;
		color: #1d1d1f;
	}

	.types-list {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.type-item {
		background: #f5f5f7;
		border: none;
		border-radius: 8px;
		padding: 1rem;
		text-align: left;
		cursor: pointer;
		transition: background-color 0.2s;
	}

	.type-item:hover {
		background: #e5e5e7;
	}

	.type-item.active {
		background: #007aff;
		color: #ffffff;
	}

	.type-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.5rem;
	}

	.type-id {
		font-weight: 600;
		font-size: 0.9375rem;
	}

	.type-level {
		font-size: 0.75rem;
		opacity: 0.8;
	}

	.type-count {
		font-size: 0.875rem;
		opacity: 0.7;
	}

	.type-detail-panel {
		background: #ffffff;
		border-radius: 12px;
		padding: 2rem;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
		max-height: calc(100vh - 200px);
		overflow-y: auto;
	}

	.type-detail-panel h2 {
		font-size: 1.5rem;
		font-weight: 600;
		color: #1d1d1f;
		margin-bottom: 1.5rem;
	}

	.type-detail-panel h3 {
		font-size: 1.125rem;
		font-weight: 600;
		color: #1d1d1f;
		margin-top: 2rem;
		margin-bottom: 1rem;
	}

	.type-info {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		margin-bottom: 2rem;
	}

	.info-row {
		display: flex;
		gap: 1rem;
	}

	.info-label {
		font-weight: 600;
		color: #86868b;
		min-width: 120px;
	}

	.info-value {
		color: #1d1d1f;
	}

	.patterns-list {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.pattern-item {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.75rem;
		background: #f5f5f7;
		border-radius: 8px;
	}

	.pattern-pattern {
		font-family: 'Monaco', 'Courier New', monospace;
		font-size: 0.875rem;
		color: #1d1d1f;
	}

	.pattern-count {
		font-weight: 600;
		color: #007aff;
	}

	.incidences-list {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.incidence-item {
		padding: 1rem;
		background: #f5f5f7;
		border-radius: 8px;
	}

	.incidence-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.5rem;
	}

	.incidence-id {
		font-weight: 600;
		color: #1d1d1f;
	}

	.incidence-type {
		font-size: 0.875rem;
		color: #86868b;
	}

	.incidence-args,
	.incidence-value {
		font-size: 0.875rem;
		color: #86868b;
		margin-top: 0.25rem;
	}

	.more-indicator {
		text-align: center;
		padding: 1rem;
		color: #86868b;
		font-size: 0.875rem;
	}

	.empty-state {
		text-align: center;
		padding: 4rem 2rem;
		color: #86868b;
	}

	@media (max-width: 1024px) {
		.schema-content {
			grid-template-columns: 1fr;
		}

		.types-panel {
			max-height: 400px;
		}
	}
</style>

