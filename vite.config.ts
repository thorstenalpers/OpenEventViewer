import { defineConfig } from 'vite';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
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
