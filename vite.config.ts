import { readFileSync } from 'node:fs';
import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';

// One version, in package.json, which tauri.conf.json also points at. A number typed a second time
// into a view is a number that goes stale the first time anyone bumps the first one.
const { version } = JSON.parse(readFileSync('package.json', 'utf8')) as { version: string };

export default defineConfig({
	define: { __APP_VERSION__: JSON.stringify(version) },
	plugins: [tailwindcss(), sveltekit()],
	server: {
		// Pinned because tauri.conf.json waits for exactly this URL: on a taken port Vite would
		// silently pick the next one and `npm run start` would hang forever.
		port: 5176,
		strictPort: true,
		// Cargo rewrites the exe while the dev server runs, and the watcher dies on it with EBUSY.
		watch: { ignored: ['**/src-tauri/**', '**/target/**', '**/vendor/**'] }
	}
});
