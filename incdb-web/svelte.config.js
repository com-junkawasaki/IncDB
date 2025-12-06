import adapter from '@sveltejs/adapter-node';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import { mdsvex } from 'mdsvex';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	extensions: ['.svelte', '.md', '.svx', '.mdx'],
	preprocess: [
		vitePreprocess(),
		mdsvex({
			extensions: ['.md', '.svx', '.mdx'],
			layout: {
				paper: './src/lib/paper/layout.svelte'
			}
		})
	],
	kit: {
		adapter: adapter({
			out: 'build'
		})
	}
};

export default config;

