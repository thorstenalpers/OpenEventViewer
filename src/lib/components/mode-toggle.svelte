<script lang="ts">
	import { mode, toggleMode } from 'mode-watcher';
	import SunIcon from '@lucide/svelte/icons/sun';
	import MoonIcon from '@lucide/svelte/icons/moon';
	import { i18n } from '$lib/i18n/index.svelte';

	const isDark = $derived(mode.current === 'dark');
</script>

<button
	type="button"
	aria-label={isDark ? i18n.t.sidebar.toLight : i18n.t.sidebar.toDark}
	onclick={() => toggleMode()}
	class="flex size-8 cursor-pointer items-center justify-center rounded-md text-muted-foreground transition-colors duration-150 hover:bg-muted hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
>
	{#if isDark}
		<SunIcon class="size-4" />
	{:else}
		<MoonIcon class="size-4" />
	{/if}
</button>
