<script lang="ts">
	import { onMount } from 'svelte';
	import { paperContent, paperMetadata } from '$lib/paper/content';

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

	function formatContent(content: string): string {
		// Convert markdown-style formatting to HTML
		let formatted = content;
		
		// Code blocks (preserve line breaks)
		formatted = formatted.replace(/```(\w+)?\n([\s\S]*?)```/g, (_match, lang, code) => {
			return `<pre class="code-block"><code class="language-${lang || 'text'}">${escapeHtml(code.trim())}</code></pre>`;
		});
		
		// Inline code
		formatted = formatted.replace(/`([^`\n]+)`/g, '<code class="inline-code">$1</code>');
		
		// Bold text
		formatted = formatted.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');
		
		// Italic text
		formatted = formatted.replace(/\*([^*]+)\*/g, '<em>$1</em>');
		
		// Links
		formatted = formatted.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" rel="noopener">$1</a>');
		
		// Numbered lists
		formatted = formatted.replace(/^\d+\.\s+(.+)$/gm, '<li>$1</li>');
		formatted = formatted.replace(/(<li>.*<\/li>\n?)+/g, (listMatch) => {
			return `<ol>${listMatch}</ol>`;
		});
		
		// Bullet lists
		formatted = formatted.replace(/^[-*]\s+(.+)$/gm, '<li>$1</li>');
		formatted = formatted.replace(/(<li>.*<\/li>\n?)+/g, (match) => {
			if (!match.includes('<ol>')) {
				return `<ul>${match}</ul>`;
			}
			return match;
		});
		
		// Paragraphs (split by double newlines)
		const paragraphs = formatted.split(/\n\n+/);
		formatted = paragraphs.map(p => {
			const trimmed = p.trim();
			if (!trimmed || trimmed.startsWith('<')) {
				return trimmed;
			}
			return `<p>${trimmed}</p>`;
		}).join('\n\n');
		
		return formatted;
	}

	function escapeHtml(text: string): string {
		// SSR-safe HTML escaping without using document
		return text
			.replace(/&/g, '&amp;')
			.replace(/</g, '&lt;')
			.replace(/>/g, '&gt;')
			.replace(/"/g, '&quot;')
			.replace(/'/g, '&#39;');
	}

	function getSectionList(sections: typeof paperContent): typeof paperContent {
		const result: typeof paperContent = [];
		for (const section of sections) {
			result.push(section);
			if (section.subsections) {
				result.push(...getSectionList(section.subsections));
			}
		}
		return result;
	}

	const allSections = getSectionList(paperContent);
</script>

<div class="paper-page">
	<div class="paper-header">
		<h1 class="paper-title">{paperMetadata.title}</h1>
		<div class="paper-meta">
			<div class="authors">
				{paperMetadata.authors.join(', ')}
			</div>
			<div class="date">{paperMetadata.date}</div>
			<div class="keywords">
				{paperMetadata.keywords.join(' • ')}
			</div>
		</div>
	</div>

	<div class="paper-container">
		<aside class="paper-sidebar">
			<nav class="section-nav">
				<h3>Table of Contents</h3>
				<ul>
					{#each allSections as section}
						<li>
							<a href="#{section.id}">
								{section.title}
							</a>
						</li>
					{/each}
				</ul>
			</nav>
		</aside>

		<main class="paper-content">
			{#each paperContent as section}
				<div id={section.id} class="section">
					<h1 class="section-title">{section.title}</h1>
					<div class="section-content">
						{@html formatContent(section.content)}
					</div>

					{#if section.subsections}
						{#each section.subsections as subsection}
							<div id={subsection.id} class="subsection">
								<h2 class="subsection-title">{subsection.title}</h2>
								<div class="subsection-content">
									{@html formatContent(subsection.content)}
								</div>
							</div>
						{/each}
					{/if}
				</div>
			{/each}

			{#if leanProofResults}
				<div id="lean-proofs" class="section">
					<h1 class="section-title">Lean Proof Verification Results</h1>
					<div class="section-content">
						<pre class="lean-results"><code>{leanProofResults}</code></pre>
					</div>
				</div>
			{/if}
		</main>
	</div>
</div>

<style>
	.paper-page {
		max-width: 1400px;
		margin: 0 auto;
		padding: 2rem;
		background: white;
	}

	.paper-header {
		border-bottom: 2px solid #333;
		padding-bottom: 1.5rem;
		margin-bottom: 2rem;
	}

	.paper-title {
		font-size: 2rem;
		font-weight: bold;
		margin-bottom: 1rem;
		color: #1a1a1a;
	}

	.paper-meta {
		font-size: 0.9rem;
		color: #666;
	}

	.authors {
		font-weight: 500;
		margin-bottom: 0.5rem;
	}

	.date {
		margin-bottom: 0.5rem;
	}

	.keywords {
		font-style: italic;
	}

	.paper-container {
		display: flex;
		gap: 2rem;
	}

	.paper-sidebar {
		flex: 0 0 250px;
		position: sticky;
		top: 2rem;
		height: fit-content;
		max-height: calc(100vh - 4rem);
		overflow-y: auto;
	}

	.section-nav {
		background: #f5f5f5;
		padding: 1rem;
		border-radius: 4px;
	}

	.section-nav h3 {
		font-size: 1rem;
		margin-bottom: 0.5rem;
		color: #333;
	}

	.section-nav ul {
		list-style: none;
		padding: 0;
		margin: 0;
	}

	.section-nav li {
		margin-bottom: 0.25rem;
	}

	.section-nav a {
		display: block;
		padding: 0.25rem 0.5rem;
		color: #555;
		text-decoration: none;
		border-radius: 3px;
		font-size: 0.9rem;
		transition: background 0.2s;
	}

	.section-nav a:hover {
		background: #e0e0e0;
	}

	.section-nav a.active {
		background: #4a90e2;
		color: white;
	}

	.paper-content {
		flex: 1;
		min-width: 0;
	}

	.section {
		margin-bottom: 3rem;
		scroll-margin-top: 2rem;
	}

	.section-title {
		font-size: 1.8rem;
		font-weight: bold;
		margin-bottom: 1rem;
		color: #1a1a1a;
		border-bottom: 2px solid #4a90e2;
		padding-bottom: 0.5rem;
	}

	.subsection {
		margin-top: 2rem;
		margin-left: 1.5rem;
	}

	.subsection-title {
		font-size: 1.4rem;
		font-weight: 600;
		margin-bottom: 0.75rem;
		color: #333;
		border-bottom: 1px solid #ccc;
		padding-bottom: 0.25rem;
	}

	.section-content,
	.subsection-content {
		line-height: 1.8;
		color: #333;
		font-size: 1rem;
	}

	.section-content p,
	.subsection-content p {
		margin-bottom: 1rem;
	}

	.section-content code.inline-code,
	.subsection-content code.inline-code {
		background: #f0f0f0;
		padding: 0.2em 0.4em;
		border-radius: 3px;
		font-family: 'Monaco', 'Courier New', monospace;
		font-size: 0.9em;
		color: #d73a49;
	}

	.section-content code,
	.subsection-content code {
		font-family: 'Monaco', 'Courier New', monospace;
	}

	.section-content pre.code-block,
	.subsection-content pre.code-block {
		background: #f5f5f5;
		border: 1px solid #ddd;
		border-radius: 4px;
		padding: 1rem;
		overflow-x: auto;
		margin: 1.5rem 0;
		font-size: 0.9em;
		line-height: 1.6;
	}

	.section-content pre.code-block code,
	.subsection-content pre.code-block code {
		background: none;
		padding: 0;
		font-size: inherit;
		line-height: inherit;
		color: #333;
	}

	.section-content ul,
	.section-content ol,
	.subsection-content ul,
	.subsection-content ol {
		margin: 1rem 0;
		padding-left: 2rem;
	}

	.section-content li,
	.subsection-content li {
		margin: 0.5rem 0;
		line-height: 1.6;
	}

	.section-content a,
	.subsection-content a {
		color: #4a90e2;
		text-decoration: none;
		border-bottom: 1px solid transparent;
		transition: border-color 0.2s;
	}

	.section-content a:hover,
	.subsection-content a:hover {
		border-bottom-color: #4a90e2;
	}

	.section-content strong,
	.subsection-content strong {
		font-weight: 600;
		color: #1a1a1a;
	}

	.section-content em,
	.subsection-content em {
		font-style: italic;
		color: #555;
	}

	.lean-results {
		background: #1e1e1e;
		color: #d4d4d4;
		font-family: 'Monaco', 'Courier New', monospace;
		font-size: 0.85em;
		line-height: 1.6;
		max-height: 600px;
		overflow-y: auto;
	}

	.lean-results code {
		color: #d4d4d4;
		background: none;
	}

	/* Print styles */
	@media print {
		.paper-page {
			padding: 0;
			max-width: 100%;
		}

		.paper-sidebar {
			display: none;
		}

		.paper-container {
			display: block;
		}

		.paper-header {
			page-break-after: always;
		}

		.section {
			page-break-inside: avoid;
			margin-bottom: 2rem;
		}

		.section-title,
		.subsection-title {
			page-break-after: avoid;
		}

		.section-content pre.code-block,
		.subsection-content pre.code-block {
			page-break-inside: avoid;
			background: #f9f9f9;
			border: 1px solid #ccc;
		}

		.lean-results {
			page-break-inside: avoid;
			max-height: none;
		}

		a {
			color: #000;
			text-decoration: underline;
		}

		a[href^="http"]:after {
			content: " (" attr(href) ")";
			font-size: 0.8em;
			color: #666;
		}
	}

	/* Responsive */
	@media (max-width: 768px) {
		.paper-container {
			flex-direction: column;
		}

		.paper-sidebar {
			position: static;
			max-height: none;
		}
	}
</style>
