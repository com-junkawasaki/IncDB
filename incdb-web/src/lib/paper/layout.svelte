<script lang="ts">
	export let title = '';
	export let authors = '';
	export let date = '';
	export let keywords: string[] = [];
</script>

<div class="paper-layout">
	<slot />
</div>

<style>
	.paper-layout {
		max-width: 1400px;
		margin: 0 auto;
		padding: 2rem;
		background: white;
		line-height: 1.8;
		color: #333;
		font-size: 1rem;
	}

	:global(.paper-layout h1) {
		font-size: 1.8rem;
		font-weight: bold;
		margin: 2rem 0 1rem;
		color: #1a1a1a;
		border-bottom: 2px solid #4a90e2;
		padding-bottom: 0.5rem;
	}

	:global(.paper-layout h2) {
		font-size: 1.4rem;
		font-weight: 600;
		margin: 1.5rem 0 0.75rem;
		color: #333;
		border-bottom: 1px solid #ccc;
		padding-bottom: 0.25rem;
	}

	:global(.paper-layout h3) {
		font-size: 1.2rem;
		font-weight: 600;
		margin: 1rem 0 0.5rem;
		color: #444;
	}

	:global(.paper-layout p) {
		margin-bottom: 1rem;
	}

	:global(.paper-layout code) {
		background: #f0f0f0;
		padding: 0.2em 0.4em;
		border-radius: 3px;
		font-family: 'Monaco', 'Courier New', monospace;
		font-size: 0.9em;
		color: #d73a49;
	}

	:global(.paper-layout pre) {
		background: #f5f5f5;
		border: 1px solid #ddd;
		border-radius: 4px;
		padding: 1rem;
		overflow-x: auto;
		margin: 1.5rem 0;
		font-size: 0.9em;
		line-height: 1.6;
	}

	:global(.paper-layout pre code) {
		background: none;
		padding: 0;
		color: #333;
	}

	:global(.paper-layout ul),
	:global(.paper-layout ol) {
		margin: 1rem 0;
		padding-left: 2rem;
	}

	:global(.paper-layout li) {
		margin: 0.5rem 0;
	}

	:global(.paper-layout strong) {
		font-weight: 600;
		color: #1a1a1a;
	}

	:global(.paper-layout em) {
		font-style: italic;
		color: #555;
	}

	:global(.paper-layout a) {
		color: #4a90e2;
		text-decoration: none;
		border-bottom: 1px solid transparent;
		transition: border-color 0.2s;
	}

	:global(.paper-layout a:hover) {
		border-bottom-color: #4a90e2;
	}

	@media print {
		.paper-layout {
			padding: 0;
			max-width: 100%;
		}

		:global(.paper-layout h1),
		:global(.paper-layout h2) {
			page-break-after: avoid;
		}

		:global(.paper-layout pre) {
			page-break-inside: avoid;
		}
	}
</style>
