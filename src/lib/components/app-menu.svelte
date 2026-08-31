<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { call } from '$lib/bridge/client';
	import { i18n } from '$lib/i18n/index.svelte';
	import { cn } from '$lib/utils';

	const t = $derived(i18n.t);

	type ItemId = keyof typeof t.menu.items;

	interface Item {
		id: ItemId;
		run: () => unknown;
		/** Only where the key really does it — an invented shortcut is worse than none. */
		shortcut?: string;
	}

	interface Menu {
		id: keyof typeof t.menu.titles;
		groups: Item[][];
	}

	// Every entry here does what it says. An entry that only apologises when pressed is worse than
	// no entry, so the sketches this bar started as are gone rather than greyed out.
	const MENUS = $derived<Menu[]>([
		{
			id: 'file',
			groups: [[{ id: 'exit', run: () => call('app_exit', {}), shortcut: 'Alt+F4' }]]
		},
		{
			id: 'view',
			groups: [[{ id: 'settings', run: () => goto(resolve('/settings')) }]]
		},
		{
			id: 'help',
			groups: [[{ id: 'about', run: () => goto(resolve('/info')) }]]
		}
	]);

	let open = $state<string | null>(null);

	function pick(item: Item) {
		open = null;
		void item.run();
	}
</script>

<svelte:window
	onmousedown={(event: MouseEvent) => {
		const target = event.target as HTMLElement | null;
		if (open && !target?.closest('[data-menu]')) open = null;
	}}
	onkeydown={(event: KeyboardEvent) => {
		if (event.key === 'Escape') open = null;
	}}
/>

<div
	data-menu
	role="menubar"
	tabindex="-1"
	aria-label={t.menu.label}
	class="relative z-40 flex h-7 shrink-0 items-center border-b border-sidebar-border bg-sidebar px-1"
	onmouseleave={() => {}}
>
	{#each MENUS as menu (menu.id)}
		<div class="relative">
			<button
				type="button"
				role="menuitem"
				aria-haspopup="true"
				aria-expanded={open === menu.id}
				onclick={() => (open = open === menu.id ? null : menu.id)}
				onmouseenter={() => {
					// Once one is open, sliding across the bar walks the menus — the way every menu bar
					// on this platform has behaved for thirty years.
					if (open) open = menu.id;
				}}
				class={cn(
					'h-6 cursor-pointer rounded-sm px-2 text-xs text-sidebar-foreground/90',
					'hover:bg-sidebar-accent hover:text-sidebar-accent-foreground',
					'focus-visible:ring-2 focus-visible:ring-sidebar-ring focus-visible:outline-none',
					open === menu.id && 'bg-sidebar-accent text-sidebar-accent-foreground'
				)}
			>
				{t.menu.titles[menu.id]}
			</button>

			{#if open === menu.id}
				<div
					role="menu"
					aria-label={t.menu.titles[menu.id]}
					tabindex="-1"
					class="absolute start-0 top-full z-50 min-w-56 rounded-b-md border border-t-0 bg-background py-1 shadow-lg"
				>
					{#each menu.groups as group, index (index)}
						{#if index > 0}
							<div class="my-1 h-px bg-border" role="separator"></div>
						{/if}
						{#each group as item (item.id)}
							<button
								type="button"
								role="menuitem"
								onclick={() => pick(item)}
								class="flex w-full cursor-pointer items-center gap-6 px-3 py-1 text-start text-xs hover:bg-muted"
							>
								<span class="flex-1">{t.menu.items[item.id]}</span>
								{#if item.shortcut}
									<span class="text-muted-foreground">{item.shortcut}</span>
								{/if}
							</button>
						{/each}
					{/each}
				</div>
			{/if}
		</div>
	{/each}
</div>
