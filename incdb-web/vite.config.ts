import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
	server: {
		host: '0.0.0.0',
		port: 5173,
		allowedHosts: true
	},
	preview: {
		host: '0.0.0.0',
		port: 5173
	},
	ssr: {
		external: ['plotly.js-dist-min', 'monaco-editor', 'vis-network']
	},
	optimizeDeps: {
		include: ['monaco-editor', 'plotly.js-dist-min', 'vis-network']
	},
	build: {
		rollupOptions: {
			external: ['plotly.js-dist-min', 'monaco-editor', 'vis-network']
		}
	}
});

