<script lang="ts">
	import { onMount } from 'svelte';

	let incidences: any[] = [];
	let loading = false;
	let error: string | null = null;

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
							}
						}
					`,
				}),
			});
			const data = await response.json();
			if (data.errors) {
				error = data.errors[0].message;
			} else {
				incidences = data.data.incidences || [];
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Unknown error';
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		fetchIncidences();
	});
</script>

<div class="container">
	<h1>IncDB</h1>
	<p>Incidence-only Foundation Database</p>

	{#if loading}
		<p>Loading...</p>
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
					{#if incidence.args.length > 0}
						<p>Args: {incidence.args.join(', ')}</p>
					{/if}
				</div>
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
	}

	.error {
		color: red;
	}

	.incidences {
		margin-top: 2rem;
	}

	.incidence {
		border: 1px solid #ddd;
		border-radius: 4px;
		padding: 1rem;
		margin-bottom: 1rem;
		background: #f9f9f9;
	}

	.incidence h3 {
		margin-top: 0;
		color: #555;
	}
</style>

