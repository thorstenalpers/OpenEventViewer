<script lang="ts">
	import FilterPopover from './filter-popover.svelte';
	import { isEmptyNumberQuery, parseNumberQuery } from '$lib/events';
	import { i18n } from '$lib/i18n/index.svelte';

	interface Props {
		title: string;
		text: string;
		onChange: (text: string | undefined) => void;
	}

	let { title, text, onChange }: Props = $props();

	const t = $derived(i18n.t);

	const query = $derived(parseNumberQuery(text));
	const active = $derived(text.trim().length > 0);

	function set(value: string) {
		onChange(value.trim() ? value : undefined);
	}
</script>

<FilterPopover label={active ? text : title} {title} {active} width={280}>
	<div class="flex flex-col gap-1.5">
		<input
			value={text}
			oninput={(event) => set(event.currentTarget.value)}
			placeholder="41, 6008, >7000, 7000-7040, !10016"
			aria-label={title}
			class="h-7 rounded border border-input bg-background px-1.5 text-xs focus-visible:ring-1 focus-visible:ring-ring focus-visible:outline-none"
		/>
		{#if query.invalid.length}
			<p class="text-[11px] text-destructive">
				{t.events.filters.notUnderstood(query.invalid.join(', '))}
			</p>
		{:else if active && isEmptyNumberQuery(query)}
			<p class="text-[11px] text-muted-foreground">{t.events.filters.numberHint}</p>
		{/if}
		<dl class="grid grid-cols-[auto_1fr] gap-x-2 text-[11px] text-muted-foreground">
			<dt class="font-mono">41, 6008</dt>
			<dd>{t.events.filters.helpAny}</dd>
			<dt class="font-mono">&gt;7000 &lt;=100</dt>
			<dd>{t.events.filters.helpCompare}</dd>
			<dt class="font-mono">7000-7040</dt>
			<dd>{t.events.filters.helpRange}</dd>
			<dt class="font-mono">!10016</dt>
			<dd>{t.events.filters.helpNot}</dd>
		</dl>
		{#if active}
			<button
				type="button"
				class="cursor-pointer rounded px-1 py-0.5 text-start text-xs text-muted-foreground hover:bg-muted/60"
				onclick={() => onChange(undefined)}
			>
				{t.events.filters.clear}
			</button>
		{/if}
	</div>
</FilterPopover>
