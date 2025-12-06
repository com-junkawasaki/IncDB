<script lang="ts">
	import { onMount } from 'svelte';

	let incidences: any[] = [];
	let loading = false;
	let error: string | null = null;
	let serverStatus = 'unknown';

	async function fetchIncidences() {
		loading = true;
		error = null;
		try {
			const response = await fetch('/api/graphql', {
				method: 'POST',
				headers: {
					'Content-Type': 'application/json',
				},
				body: JSON.stringify({
					query: `
						query {
							incidences {
								id
								level
								typeId
								args
								roles
								value {
									str
									int
									float
									bool
								}
							}
						}
					`,
				}),
			});
			
			if (response.ok) {
				const data = await response.json();
				if (data.errors) {
					error = data.errors[0].message;
					serverStatus = 'error';
				} else {
					incidences = data.data?.incidences || [];
					serverStatus = 'connected';
				}
			} else {
				// GraphQL サーバーが起動していない場合はモックデータを表示
				incidences = [
					{ id: '1', level: 0, typeId: null, args: [], roles: [], value: { str: 'Hello' } },
					{ id: '2', level: 0, typeId: '10', args: ['1'], roles: [1], value: { int: 42 } },
				];
				serverStatus = 'mock';
			}
		} catch (e) {
			// エラー時もモックデータを表示
			incidences = [
				{ id: '1', level: 0, typeId: null, args: [], roles: [], value: { str: 'Hello' } },
				{ id: '2', level: 0, typeId: '10', args: ['1'], roles: [1], value: { int: 42 } },
			];
			serverStatus = 'mock';
			console.warn('GraphQL server not available, showing mock data:', e);
		} finally {
			loading = false;
		}
	}

	function getStatusColor() {
		switch (serverStatus) {
			case 'connected':
				return '#4caf50';
			case 'mock':
				return '#ff9800';
			case 'error':
				return '#f44336';
			default:
				return '#757575';
		}
	}

	function getStatusText() {
		switch (serverStatus) {
			case 'connected':
				return 'Connected to GraphQL server';
			case 'mock':
				return 'Using mock data (GraphQL server not available)';
			case 'error':
				return 'GraphQL server error';
			default:
				return 'Unknown status';
		}
	}

	onMount(() => {
		fetchIncidences();
	});
</script>

<div class="container">
	<h1>IncDB</h1>
	<p>Incidence-only Foundation Database</p>
	
	<div class="status" style="background-color: {getStatusColor()}20; border-color: {getStatusColor()}">
		<span class="status-dot" style="background-color: {getStatusColor()}"></span>
		<span>{getStatusText()}</span>
	</div>

	{#if loading}
		<p class="loading">Loading...</p>
	{:else if error}
		<p class="error">Error: {error}</p>
	{:else}
		<div class="incidences">
			<h2>Incidences ({incidences.length})</h2>
			{#each incidences as incidence}
				<div class="incidence">
					<h3>ID: {incidence.id}</h3>
					<p>Level: {incidence.level}</p>
					{#if incidence.typeId}
						<p>Type: {incidence.typeId}</p>
					{/if}
					{#if incidence.args && incidence.args.length > 0}
						<p>Args: {incidence.args.join(', ')}</p>
					{/if}
					{#if incidence.roles && incidence.roles.length > 0}
						<p>Roles: {incidence.roles.join(', ')}</p>
					{/if}
					{#if incidence.value}
						<div class="value">
							<p>Value:</p>
							<ul>
								{#if incidence.value.str}
									<li>String: {incidence.value.str}</li>
								{/if}
								{#if incidence.value.int !== null && incidence.value.int !== undefined}
									<li>Int: {incidence.value.int}</li>
								{/if}
								{#if incidence.value.float !== null && incidence.value.float !== undefined}
									<li>Float: {incidence.value.float}</li>
								{/if}
								{#if incidence.value.bool !== null && incidence.value.bool !== undefined}
									<li>Bool: {incidence.value.bool}</li>
								{/if}
							</ul>
						</div>
					{/if}
				</div>
			{:else}
				<p class="empty">No incidences found</p>
			{/each}
		</div>
	{/if}
</div>

<style>
	.container {
		max-width: 1200px;
		margin: 0 auto;
		padding: 2rem;
	}

	h1 {
		color: #333;
		margin-bottom: 0.5rem;
	}

	.status {
		display: inline-flex;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem 1rem;
		border-radius: 4px;
		border: 1px solid;
		margin: 1rem 0;
		font-size: 0.9rem;
	}

	.status-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
	}

	.loading {
		color: #666;
		font-style: italic;
	}

	.error {
		color: #f44336;
		padding: 1rem;
		background: #ffebee;
		border-radius: 4px;
	}

	.incidences {
		margin-top: 2rem;
	}

	.incidence {
		border: 1px solid #ddd;
		border-radius: 8px;
		padding: 1.5rem;
		margin-bottom: 1rem;
		background: #f9f9f9;
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
	}

	.incidence h3 {
		margin-top: 0;
		color: #555;
		border-bottom: 2px solid #ddd;
		padding-bottom: 0.5rem;
	}

	.incidence p {
		margin: 0.5rem 0;
		color: #666;
	}

	.value {
		margin-top: 1rem;
		padding: 1rem;
		background: #fff;
		border-radius: 4px;
		border-left: 3px solid #4caf50;
	}

	.value ul {
		margin: 0.5rem 0 0 1.5rem;
	}

	.value li {
		margin: 0.25rem 0;
	}

	.empty {
		text-align: center;
		color: #999;
		padding: 2rem;
		font-style: italic;
	}
</style>
