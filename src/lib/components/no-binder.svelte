<script lang="ts">
	import { resolve } from '$app/paths';
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent } from '$lib/components/ui/card';
	import { i18n } from '$lib/i18n/index.svelte';

	const t = $derived(i18n.t);
</script>

<!-- A page whose whole subject is one exam has nothing to say without one. Saying so, and pointing
     at the place that fixes it, beats a single grey line that reads as a broken page. -->
<Card>
	<CardContent class="flex flex-col items-start gap-3 py-8">
		<p class="text-sm font-medium">{t.common.noBinder}</p>
		<p class="max-w-md text-sm text-muted-foreground">{t.common.noBinderBody}</p>
		<Button href={resolve('/projects')}>{t.common.toProjects}</Button>
	</CardContent>
</Card>
