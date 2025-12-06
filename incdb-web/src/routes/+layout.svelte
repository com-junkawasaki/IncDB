<script lang="ts">
	import '../app.css';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';

	const navItems = [
		{ path: '/', label: 'Home', icon: '🏠' },
		{ path: '/schema', label: 'Schema', icon: '📋' },
		{ path: '/query', label: 'Query', icon: '🔍' },
		{ path: '/view', label: 'View', icon: '👁️' },
		{ path: '/graph', label: 'Graph', icon: '🕸️' },
		{ path: '/vector', label: 'Vector', icon: '📊' },
		{ path: '/benchmark', label: 'Benchmark', icon: '⚡' },
		{ path: '/paper', label: 'Paper', icon: '📄' },
	];

	function isActive(path: string): boolean {
		return $page.url.pathname === path;
	}
</script>

<div class="layout">
	<aside class="sidebar">
		<div class="sidebar-header">
			<h1>IncDB</h1>
			<p class="subtitle">Incidence Database</p>
		</div>
		<nav class="nav">
			{#each navItems as item}
				<a
					href={item.path}
					class:active={isActive(item.path)}
					class="nav-item"
				>
					<span class="nav-icon">{item.icon}</span>
					<span class="nav-label">{item.label}</span>
				</a>
			{/each}
		</nav>
	</aside>
	<main class="main-content">
		<slot />
	</main>
</div>

<style>
	.layout {
		display: flex;
		min-height: 100vh;
		background: #f5f5f7;
	}

	.sidebar {
		width: 280px;
		background: #ffffff;
		border-right: 1px solid #e5e5e7;
		display: flex;
		flex-direction: column;
		position: sticky;
		top: 0;
		height: 100vh;
		overflow-y: auto;
	}

	.sidebar-header {
		padding: 2rem 1.5rem;
		border-bottom: 1px solid #e5e5e7;
	}

	.sidebar-header h1 {
		font-size: 1.5rem;
		font-weight: 600;
		color: #1d1d1f;
		margin-bottom: 0.25rem;
	}

	.subtitle {
		font-size: 0.875rem;
		color: #86868b;
	}

	.nav {
		flex: 1;
		padding: 1rem 0;
	}

	.nav-item {
		display: flex;
		align-items: center;
		padding: 0.75rem 1.5rem;
		color: #1d1d1f;
		text-decoration: none;
		transition: background-color 0.2s;
		cursor: pointer;
	}

	.nav-item:hover {
		background-color: #f5f5f7;
	}

	.nav-item.active {
		background-color: #007aff;
		color: #ffffff;
	}

	.nav-icon {
		font-size: 1.25rem;
		margin-right: 0.75rem;
		width: 1.5rem;
		text-align: center;
	}

	.nav-label {
		font-size: 0.9375rem;
		font-weight: 500;
	}

	.main-content {
		flex: 1;
		padding: 2rem;
		overflow-y: auto;
		max-width: 1400px;
		margin: 0 auto;
		width: 100%;
	}

	/* iPad 対応 */
	@media (max-width: 1024px) {
		.sidebar {
			width: 240px;
		}

		.main-content {
			padding: 1.5rem;
		}
	}

	/* モバイル対応 */
	@media (max-width: 768px) {
		.layout {
			flex-direction: column;
		}

		.sidebar {
			width: 100%;
			height: auto;
			position: relative;
			border-right: none;
			border-bottom: 1px solid #e5e5e7;
		}

		.sidebar-header {
			padding: 1rem;
		}

		.nav {
			display: flex;
			overflow-x: auto;
			padding: 0.5rem 0;
		}

		.nav-item {
			padding: 0.5rem 1rem;
			white-space: nowrap;
		}

		.main-content {
			padding: 1rem;
		}
	}
</style>
