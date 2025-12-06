<script lang="ts">
	import { onMount } from 'svelte';
	import { getSchemaStats } from '$lib/graphql/client';
	import type { GraphQLSchemaStats } from '$lib/graphql/types';

	let stats: GraphQLSchemaStats | null = null;
	let loading = true;
	let error: string | null = null;

	onMount(async () => {
		try {
			stats = await getSchemaStats();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Unknown error';
		} finally {
			loading = false;
		}
	});
</script>

<div class="home">
	<h1>IncDB Dashboard</h1>
	<p class="subtitle">Incidence-only Foundation Database</p>

	{#if loading}
		<div class="loading">Loading...</div>
	{:else if error}
		<div class="error">Error: {error}</div>
	{:else if stats}
		<div class="stats-grid">
			<div class="stat-card">
				<div class="stat-value">{stats.totalIncidences}</div>
				<div class="stat-label">Total Incidences</div>
			</div>
			<div class="stat-card">
				<div class="stat-value">{stats.typeCount}</div>
				<div class="stat-label">Types</div>
			</div>
			<div class="stat-card">
				<div class="stat-value">{stats.roleCount}</div>
				<div class="stat-label">Roles</div>
			</div>
			<div class="stat-card">
				<div class="stat-value">{stats.vectorCount}</div>
				<div class="stat-label">Vectors</div>
			</div>
		</div>

		<div class="quick-actions">
			<h2>Quick Actions</h2>
			<div class="actions-grid">
				<a href="/schema" class="action-card">
					<span class="action-icon">📋</span>
					<span class="action-label">View Schema</span>
				</a>
				<a href="/query" class="action-card">
					<span class="action-icon">🔍</span>
					<span class="action-label">Run Query</span>
				</a>
				<a href="/graph" class="action-card">
					<span class="action-icon">🕸️</span>
					<span class="action-label">Visualize Graph</span>
				</a>
				<a href="/vector" class="action-card">
					<span class="action-icon">📊</span>
					<span class="action-label">Vector Analysis</span>
				</a>
			</div>
		</div>
	{/if}
</div>

<style>
	.home {
		max-width: 1200px;
		margin: 0 auto;
	}

	h1 {
		font-size: 2.5rem;
		font-weight: 600;
		color: #1d1d1f;
		margin-bottom: 0.5rem;
	}

	.subtitle {
		font-size: 1.125rem;
		color: #86868b;
		margin-bottom: 2rem;
	}

	.loading,
	.error {
		padding: 2rem;
		text-align: center;
		color: #86868b;
	}

	.error {
		color: #ff3b30;
		background: #ffebee;
		border-radius: 8px;
	}

	.stats-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: 1.5rem;
		margin-bottom: 3rem;
	}

	.stat-card {
		background: #ffffff;
		border-radius: 12px;
		padding: 1.5rem;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
		text-align: center;
	}

	.stat-value {
		font-size: 2.5rem;
		font-weight: 600;
		color: #007aff;
		margin-bottom: 0.5rem;
	}

	.stat-label {
		font-size: 0.875rem;
		color: #86868b;
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.quick-actions {
		margin-top: 3rem;
	}

	.quick-actions h2 {
		font-size: 1.5rem;
		font-weight: 600;
		color: #1d1d1f;
		margin-bottom: 1.5rem;
	}

	.actions-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: 1.5rem;
	}

	.action-card {
		background: #ffffff;
		border-radius: 12px;
		padding: 2rem;
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
		text-decoration: none;
		color: #1d1d1f;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		transition: transform 0.2s, box-shadow 0.2s;
		cursor: pointer;
	}

	.action-card:hover {
		transform: translateY(-2px);
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
	}

	.action-icon {
		font-size: 3rem;
		margin-bottom: 1rem;
	}

	.action-label {
		font-size: 1rem;
		font-weight: 500;
	}

	@media (max-width: 768px) {
		h1 {
			font-size: 2rem;
		}

		.stats-grid,
		.actions-grid {
			grid-template-columns: repeat(2, 1fr);
		}
	}
</style>
