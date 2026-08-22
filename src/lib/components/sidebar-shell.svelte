<script lang="ts" module>
	/**
	 * Every route the sidebar can reach, in one list so the layout can preload exactly what the
	 * navigation offers — a route added here is preloaded without anyone remembering to say so.
	 */
	export const ROUTES = ['/', '/assistant', '/diagnose', '/log', '/info', '/settings'] as const;

	export type Route = (typeof ROUTES)[number];
</script>

<script lang="ts">
	import { page } from '$app/state';
	import { resolve } from '$app/paths';
	import type { Component } from 'svelte';
	import EventsIcon from '@lucide/svelte/icons/list';
	import AssistantIcon from '@lucide/svelte/icons/sparkles';
	import DiagnoseIcon from '@lucide/svelte/icons/activity';
	import SettingsIcon from '@lucide/svelte/icons/settings';
	import InfoIcon from '@lucide/svelte/icons/info';
	import LogIcon from '@lucide/svelte/icons/scroll-text';
	import PanelLeftIcon from '@lucide/svelte/icons/panel-left';
	import { cn } from '$lib/utils';
	import { i18n } from '$lib/i18n/index.svelte';
	import { settings } from '$lib/stores/settings.svelte';

	interface Entry {
		route: Route;
		label: string;
		icon: Component;
	}

	const t = $derived(i18n.t);
	const expanded = $derived(settings.sidebarExpanded);

	const entries = $derived<Entry[]>([
		{ route: '/', label: t.sidebar.events, icon: EventsIcon },
		{ route: '/assistant', label: t.sidebar.assistant, icon: AssistantIcon },
		{ route: '/diagnose', label: t.sidebar.diagnose, icon: DiagnoseIcon }
	]);

	// The diagnostics live at the foot, with the things you reach for last. The log is off by
	// default and switched on in Settings: one nobody is reading is a row that answers nothing.
	const footer = $derived<Entry[]>([
		...(settings.showLogs ? [{ route: '/log' as const, label: t.sidebar.log, icon: LogIcon }] : []),
		{ route: '/info', label: t.sidebar.info, icon: InfoIcon },
		{ route: '/settings', label: t.sidebar.settings, icon: SettingsIcon }
	]);

	function isActive(route: string): boolean {
		const href = resolve(route as Entry['route']);
		return href === '/' ? page.url.pathname === '/' : page.url.pathname.startsWith(href);
	}

	// Below `md` the rail is forced regardless of the preference: 224 px of sidebar on a narrow
	// window leaves nothing for a question. The breakpoint is CSS, not a resize listener, so there
	// is no frame where the width and the labels disagree.
	const railOnly = 'w-14 md:w-56';
	const label = $derived(expanded ? 'hidden md:inline' : 'hidden');
</script>

{#snippet navLink(entry: Entry)}
	{@const Icon = entry.icon}
	{@const active = isActive(entry.route)}
	<a
		href={resolve(entry.route)}
		aria-current={active ? 'page' : undefined}
		title={expanded ? undefined : entry.label}
		class={cn(
			'group relative flex h-9 w-full items-center gap-2.5 overflow-hidden rounded-md text-sm',
			'transition-colors duration-150',
			'hover:bg-sidebar-accent hover:text-sidebar-accent-foreground',
			'focus-visible:ring-2 focus-visible:ring-sidebar-ring focus-visible:outline-none',
			expanded ? 'justify-center px-0 md:justify-start md:px-2.5' : 'justify-center px-0',
			active
				? 'bg-sidebar-primary/15 font-semibold text-sidebar-primary hover:bg-sidebar-primary/20'
				: 'text-sidebar-foreground'
		)}
	>
		<span
			aria-hidden="true"
			class={cn(
				'absolute start-0 top-1.5 bottom-1.5 w-[3px] rounded-e-full bg-sidebar-primary transition-transform duration-200',
				active ? 'translate-x-0' : '-translate-x-1.5 rtl:translate-x-1.5'
			)}
		></span>
		<Icon class="size-4 shrink-0" />
		<span class={cn('flex-1 truncate text-start', label)}>{entry.label}</span>
	</a>
{/snippet}

<nav
	class={cn(
		'flex shrink-0 flex-col overflow-hidden border-e border-sidebar-border bg-sidebar text-sidebar-foreground',
		'transition-[width] duration-150',
		expanded ? railOnly : 'w-14'
	)}
	aria-label={t.sidebar.sections}
>
	<div class={cn('flex h-12 shrink-0 items-center gap-1 px-2', !expanded && 'justify-center')}>
		<span class={cn('min-w-0 flex-col leading-none', expanded ? 'hidden md:flex' : 'hidden')}>
			<span class="truncate text-[13px] font-semibold tracking-tight">OpenEventViewer</span>
			<span class="truncate pt-0.5 text-[10px] text-sidebar-foreground/60">{t.sidebar.tagline}</span
			>
		</span>
		<!-- After the title, so the control that hides the title sits on the edge it collapses to.
		     With the rail there is no title left, and `justify-center` centres it on its own. -->
		<button
			type="button"
			onclick={() => settings.toggleSidebar()}
			aria-label={expanded ? t.sidebar.collapse : t.sidebar.expand}
			aria-expanded={expanded}
			class="flex size-8 shrink-0 cursor-pointer items-center justify-center rounded-md text-sidebar-foreground/70 transition-colors duration-150 hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 focus-visible:ring-sidebar-ring focus-visible:outline-none md:ms-auto"
		>
			<PanelLeftIcon class="size-4" />
		</button>
	</div>

	<ul class="flex min-h-0 flex-1 flex-col gap-0.5 overflow-y-auto px-1.5 pt-2">
		{#each entries as entry (entry.route)}
			<li>{@render navLink(entry)}</li>
		{/each}
	</ul>

	<ul class="flex shrink-0 flex-col gap-0.5 border-t border-sidebar-border px-1.5 py-1.5">
		{#each footer as entry (entry.route)}
			<li>{@render navLink(entry)}</li>
		{/each}
	</ul>
</nav>
