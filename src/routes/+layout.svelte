<script lang="ts">
	import '../app.css';
	import { onMount, type Snippet } from 'svelte';
	import { afterNavigate, beforeNavigate, preloadCode } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { ModeWatcher } from 'mode-watcher';
	import { Toaster } from '$lib/components/ui/sonner';
	import SidebarShell, { ROUTES } from '$lib/components/sidebar-shell.svelte';
	import { settings } from '$lib/stores/settings.svelte';
	import { captureErrors } from '$lib/stores/log.svelte';
	import { i18n } from '$lib/i18n/index.svelte';
	import { call, isMockHost } from '$lib/bridge/client';

	let { children }: { children?: Snippet } = $props();

	// First, before anything that could throw: a restore that fails must land in the log rather
	// than take the page down silently with it.
	captureErrors();
	settings.restore();

	// `main` is the scroll container, not the window, so the browser's own restoration never sees
	// it. Remembering the offset per route is what makes leaving a long statistics table and coming
	// back feel like returning rather than starting over.
	let scroller = $state<HTMLElement | null>(null);
	const offsets: Record<string, number> = {};

	beforeNavigate(({ from }) => {
		if (from && scroller) offsets[from.url.pathname] = scroller.scrollTop;
	});

	afterNavigate(({ to }) => {
		if (!to || !scroller) return;
		const target = offsets[to.url.pathname] ?? 0;

		// After the new page has painted, or the container is still the old height and the
		// assignment lands nowhere. The timeout is not belt and braces: a webview that is not
		// compositing never fires rAF, and the scroll would then never be restored at all.
		let done = false;
		const restore = () => {
			if (done || !scroller) return;
			done = true;
			scroller.scrollTop = target;
		};
		requestAnimationFrame(restore);
		setTimeout(restore, 50);
	});

	onMount(() => {
		// Every route's code, fetched in the background right after start. A first click then costs
		// a component swap rather than a network round trip and a module evaluation.
		for (const route of ROUTES) void preloadCode(resolve(route));

		// F12 by hand: WebView2's own accelerators are off in a packaged app, and without this
		// there is no way to read a console error out of it.
		const devtools = (event: KeyboardEvent) => {
			if (event.key === 'F12') {
				event.preventDefault();
				void call('devtools_open', {});
			}
		};
		window.addEventListener('keydown', devtools);
		return () => window.removeEventListener('keydown', devtools);
	});
</script>

<!-- Synchronous because the default defers the switch into a `requestAnimationFrame`, and a webview
     that is not compositing never fires one — minimised or occluded, the theme would move in
     `localStorage` and not on screen until the next load. The same trap `applyPreset` guards. -->
<ModeWatcher synchronousModeChanges />
<Toaster />

<div class="flex h-screen w-screen overflow-hidden">
	<SidebarShell />
	<main bind:this={scroller} class="flex-1 overflow-y-auto">
		{#if isMockHost()}
			<div class="border-b border-warning/40 bg-warning/10 px-6 py-2 text-xs text-muted-foreground">
				{i18n.t.common.mockHost}
			</div>
		{/if}
		{@render children?.()}
	</main>
</div>
