<script lang="ts">
	import { i18n } from '$lib/i18n/index.svelte';

	const t = $derived(i18n.t);
</script>

<div class="flex flex-col gap-3 p-4 sm:p-6">
	<h1 class="text-xl font-semibold">{t.events.title}</h1>
	<p class="text-sm text-muted-foreground">{t.events.subtitle}</p>
</div>
