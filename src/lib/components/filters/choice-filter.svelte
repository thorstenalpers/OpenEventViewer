<script lang="ts">
	import SearchIcon from '@lucide/svelte/icons/search';
	import FilterPopover from './filter-popover.svelte';
	import { choicesOf } from '$lib/events';
	import { i18n } from '$lib/i18n/index.svelte';

	interface Props {
		title: string;
		/** Every value the column holds before any filter narrowed it. */
		values: string[];
		selected: string[];
		onChange: (selected: string[]) => void;
	}

	let { title, values, selected, onChange }: Props = $props();

	const t = $derived(i18n.t);

	let search = $state('');

	// Counted over the unfiltered column, so the list does not shrink out from under a second tick:
	// narrowing to one provider must not hide the provider you were about to add.
	const choices = $derived(choicesOf(values));
	const shown = $derived(
		search.trim()
			? choices.filter((choice) => choice.value.toLowerCase().includes(search.trim().toLowerCase()))
			: choices
	);

	const label = $derived(
		selected.length === 0
			? title
			: selected.length === 1
				? (selected[0] ?? title)
				: t.events.filters.chosen(selected.length)
	);

	function toggle(value: string) {
		const next = selected.includes(value)
			? selected.filter((held) => held !== value)
			: [...selected, value];
		onChange(next);
	}
</script>

<FilterPopover {label} {title} active={selected.length > 0}>
	<div class="flex items-center gap-1.5 rounded border border-input px-1.5">
		<SearchIcon class="size-3 shrink-0 opacity-60" />
		<input
			bind:value={search}
			placeholder={t.events.filters.search}
			aria-label={`${title} — ${t.events.filters.search}`}
			class="h-6 w-full bg-transparent text-xs focus-visible:outline-none"
		/>
	</div>

	<div class="max-h-56 overflow-y-auto py-1">
		{#each shown as choice (choice.value)}
			<label
				class="flex cursor-pointer items-center gap-2 rounded px-1 py-0.5 text-xs hover:bg-muted/60"
			>
				<input
					type="checkbox"
					class="size-3"
					checked={selected.includes(choice.value)}
					onchange={() => toggle(choice.value)}
				/>
				<span class="min-w-0 flex-1 truncate" title={choice.value}>{choice.value}</span>
				<span class="shrink-0 text-muted-foreground tabular-nums">{choice.count}</span>
			</label>
		{:else}
			<p class="px-1 py-1 text-xs text-muted-foreground">{t.events.filters.noMatch}</p>
		{/each}
	</div>

	{#if selected.length}
		<button
			type="button"
			class="w-full cursor-pointer rounded px-1 py-0.5 text-start text-xs text-muted-foreground hover:bg-muted/60"
			onclick={() => onChange([])}
		>
			{t.events.filters.clear}
		</button>
	{/if}
</FilterPopover>
