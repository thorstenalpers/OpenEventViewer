<script lang="ts">
	import '../app.css';
	import { onMount, type Snippet } from 'svelte';
	import { afterNavigate, beforeNavigate, preloadCode } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { ModeWatcher } from 'mode-watcher';
	import { Toaster } from '$lib/components/ui/sonner';
	import SidebarShell, { ROUTES } from '$lib/components/sidebar-shell.svelte';
	import { library } from '$lib/stores/library.svelte';
	import { settings } from '$lib/stores/settings.svelte';
	import { voice } from '$lib/stores/voice.svelte';
	import { captureErrors } from '$lib/stores/log.svelte';
	import { i18n } from '$lib/i18n/index.svelte';
	import { call, isMockHost } from '$lib/bridge/client';

	let { children }: { children?: Snippet } = $props();

	// First, before anything that could throw: a restore that fails must land in the log rather
	// than take the page down silently with it.
	captureErrors();
	settings.restore();
	// At start rather than when the settings page opens: the chosen voice reads episodes, and a
	// pack deleted from the folder between two runs has to stop being the one that reads.
	voice.restore();

	$effect(() => {
		void library.refresh();
	});

	// No focus-reclaiming here, deliberately. An earlier version took the focus back for the app on
	// every mouseenter and pointerdown — which is the trap site::focus_chrome's own doc warns about:
	// a native select's popup is a window of its own, and so is the title bar, and stealing the
	// activation on those events closes the popup mid-click and swallows the close button. The app
	// loses focus only to things that legitimately take it — the portal (handed back by site_hide
	// and the Browse view's own handlers) and, before CREATE_NO_WINDOW, spawned console windows.

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

		// F12 by hand: the UI is a child webview, and a child does not reliably get WebView2's own
		// accelerators — without this there is no way to read a console error out of the app.
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
