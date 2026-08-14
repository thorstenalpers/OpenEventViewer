<script lang="ts">
	import ArrowLeftIcon from '@lucide/svelte/icons/arrow-left';
	import ArrowRightIcon from '@lucide/svelte/icons/arrow-right';
	import RefreshIcon from '@lucide/svelte/icons/rotate-cw';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Badge } from '$lib/components/ui/badge';
	import { call, isMockHost } from '$lib/bridge/client';
	import type { Link } from '$lib/bridge/contract';
	import { library } from '$lib/stores/library.svelte';
	import { viewState } from '$lib/stores/view-state.svelte';
	import { i18n } from '$lib/i18n/index.svelte';

	const t = $derived(i18n.t);

	const PORTALS = $derived([
		{ title: t.browse.portals.learn, url: 'https://learn.microsoft.com/en-us/training/' },
		{ title: t.browse.portals.azure, url: 'https://learn.microsoft.com/en-us/azure/' },
		{ title: t.browse.portals.credentials, url: 'https://learn.microsoft.com/en-us/credentials/' },
		{ title: t.browse.portals.youtube, url: 'https://www.youtube.com/' }
	]);

	let links = $state<Link[]>([]);
	let error = $state<string | null>(null);
	let surface = $state<HTMLDivElement | null>(null);

	const binder = $derived(library.selected);

	$effect(() => {
		const id = binder?.id;
		if (id === undefined) {
			links = [];
			return;
		}
		call('list_links', { binderId: id })
			.then((result) => (links = result))
			.catch(() => (links = []));
	});

	function bounds(element: HTMLElement) {
		const box = element.getBoundingClientRect();
		return { x: box.left, y: box.top, width: box.width, height: box.height };
	}

	// The site webview is a sibling of this page, not an element in it. The placeholder below is
	// the only thing that knows where it belongs, so its box is what the host is told — no sidebar
	// width duplicated in Rust.
	$effect(() => {
		const element = surface;
		if (!element) return;

		let disposed = false;
		const place = () => {
			if (!disposed) void call('site_place', { rect: bounds(element) }).catch(() => {});
		};

		void call('site_open', { url: viewState.browseAddress, rect: bounds(element) }).catch(
			(caught: unknown) => {
				error = caught instanceof Error ? caught.message : String(caught);
			}
		);

		const observer = new ResizeObserver(place);
		observer.observe(element);
		window.addEventListener('resize', place);

		return () => {
			disposed = true;
			observer.disconnect();
			window.removeEventListener('resize', place);
			void call('site_hide', {}).catch(() => {});
		};
	});

	function go(url: string) {
		viewState.browseAddress = url;
	}

	function submit(event: SubmitEvent) {
		event.preventDefault();
		const value = viewState.browseAddress.trim();
		viewState.browseAddress = value.startsWith('http') ? value : `https://${value}`;
	}
</script>

<!--
	Two webviews share this window and only the focused one receives clicks. The portal takes the
	focus when it opens or when it is clicked into, which leaves the toolbar below dead until
	something gives it back — so the pointer crossing the boundary is what moves the focus.
-->
<div class="flex h-full flex-col">
	<header
		role="toolbar"
		tabindex="-1"
		aria-label={t.browse.address}
		class="flex flex-col gap-2 border-b px-4 py-2.5"
		onmouseenter={() => call('site_focus', { target: 'chrome' })}
	>
		<div class="flex items-center gap-2">
			<Button
				variant="ghost"
				size="icon"
				aria-label={t.browse.back}
				onclick={() => call('site_history', { step: -1 })}
			>
				<ArrowLeftIcon class="size-4" />
			</Button>
			<Button
				variant="ghost"
				size="icon"
				aria-label={t.browse.forward}
				onclick={() => call('site_history', { step: 1 })}
			>
				<ArrowRightIcon class="size-4" />
			</Button>
			<Button
				variant="ghost"
				size="icon"
				aria-label={t.browse.reload}
				onclick={() => call('site_history', { step: 0 })}
			>
				<RefreshIcon class="size-4" />
			</Button>
			<form class="flex flex-1 gap-2" onsubmit={submit}>
				<Input bind:value={viewState.browseAddress} aria-label={t.browse.address} class="flex-1" />
				<Button type="submit" variant="outline">{t.browse.go}</Button>
			</form>
		</div>

		<div class="flex flex-wrap items-center gap-2 text-xs">
			{#each PORTALS as portal (portal.url)}
				<button
					type="button"
					class="rounded-md border px-2 py-1 hover:bg-muted"
					onclick={() => go(portal.url)}
				>
					{portal.title}
				</button>
			{/each}
			{#if links.length}
				<span class="text-muted-foreground">·</span>
				<Badge variant="accent">{binder?.certification}</Badge>
				{#each links.slice(0, 8) as bookmark (bookmark.id)}
					<button
						type="button"
						title={bookmark.url}
						class="max-w-56 truncate rounded-md border px-2 py-1 hover:bg-muted"
						onclick={() => go(bookmark.url)}
					>
						{bookmark.title}
					</button>
				{/each}
			{/if}
		</div>
	</header>

	{#if error}
		<p class="px-6 py-2 text-sm text-destructive">{error}</p>
	{/if}

	<!-- No role: the box is empty by design — the page it stands for is a sibling webview, not
	     content in this document, so any role here would describe something that is not there. -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		bind:this={surface}
		class="flex-1 bg-muted/30"
		onmouseenter={() => call('site_focus', { target: 'site' })}
	>
		{#if isMockHost()}
			<div
				class="flex h-full items-center justify-center p-6 text-center text-sm text-muted-foreground"
			>
				{t.browse.mockNote}
			</div>
		{/if}
	</div>
</div>
