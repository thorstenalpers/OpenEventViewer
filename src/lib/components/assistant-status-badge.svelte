<script lang="ts">
	import SparklesIcon from '@lucide/svelte/icons/sparkles';
	import { Badge } from '$lib/components/ui/badge';
	import { i18n } from '$lib/i18n/index.svelte';
	import { settings } from '$lib/stores/settings.svelte';
	import { assistant } from '$lib/stores/assistant.svelte';

	const t = $derived(i18n.t);

	$effect(() => {
		// Reading the source rather than closing over it: switching provider in Settings has to
		// re-probe, and the probe answers differently for each.
		void settings.assistantSource;
		void assistant.refreshStatus();
	});
</script>

<span class="flex items-center gap-2 text-sm">
	<SparklesIcon class="size-4" />
	{t.assistant.title}
	<Badge variant={assistant.ready ? 'accent' : 'neutral'}>
		{settings.assistantSource === 'cli' ? t.assistant.sourceCli : t.assistant.sourceAnthropic}
	</Badge>
	{#if !assistant.ready}
		<span class="text-xs text-muted-foreground">
			{settings.assistantSource === 'cli' ? t.assistant.noCli : t.assistant.noKey}
		</span>
	{/if}
</span>
