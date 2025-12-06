// Monaco Editor configuration

import * as monaco from 'monaco-editor';

// Configure Monaco Editor
monaco.editor.defineTheme('incdb-theme', {
	base: 'vs',
	inherit: true,
	rules: [],
	colors: {
		'editor.background': '#ffffff',
	},
});

export { monaco };

