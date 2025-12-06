<script lang="ts">
	import { onMount } from 'svelte';
	import Paper from '$lib/paper/paper.mdx';
	import Layout from '$lib/paper/layout.svelte';

	let leanProofResults: string = '';

	onMount(async () => {
		// Load Lean proof results
		try {
			const response = await fetch('/proofs/results.md');
			if (response.ok) {
				leanProofResults = await response.text();
			} else {
				leanProofResults = 'Lean proof results file not found.';
			}
		} catch (e) {
			console.error('Failed to load Lean proof results:', e);
			leanProofResults = 'Lean proof results not available.';
		}
	});
</script>

<div class="paper-page">
	<Layout>
		<Paper />
	</Layout>

	{#if leanProofResults}
		<div id="lean-proofs" class="lean-proofs-section">
			<h1>Lean Proof Verification Results</h1>
			<pre class="lean-results"><code>{leanProofResults}</code></pre>
		</div>
	{/if}
</div>

<style>
	.paper-page {
		background: white;
		min-height: 100vh;
	}

	.lean-proofs-section {
		max-width: 1400px;
		margin: 0 auto;
		padding: 2rem;
	}

	.lean-proofs-section h1 {
		font-size: 1.8rem;
		font-weight: bold;
		margin-bottom: 1rem;
		color: #1a1a1a;
		border-bottom: 2px solid #4a90e2;
		padding-bottom: 0.5rem;
	}

	.lean-results {
		background: #1e1e1e;
		color: #d4d4d4;
		font-family: 'Monaco', 'Courier New', monospace;
		font-size: 0.85em;
		line-height: 1.6;
		max-height: 600px;
		overflow-y: auto;
		padding: 1rem;
		border-radius: 4px;
	}

	.lean-results code {
		color: #d4d4d4;
		background: none;
	}

	@media print {
		.lean-results {
			max-height: none;
			page-break-inside: avoid;
		}
	}
</style>
