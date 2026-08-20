<script lang="ts">
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { call } from '$lib/bridge/client';
	import { viewState } from '$lib/stores/view-state.svelte';
	import { i18n } from '$lib/i18n/index.svelte';
	import components from '$lib/third-party.json';

	const t = $derived(i18n.t);
	const APP_VERSION = '0.1.0';

	let notices = $state<string | null>(null);
	let noticesError = $state<string | null>(null);
	let loading = $state(false);

	const shown = $derived(
		viewState.infoFilter.trim()
			? components.filter((entry) =>
					`${entry.name} ${entry.license}`
						.toLowerCase()
						.includes(viewState.infoFilter.trim().toLowerCase())
				)
			: components
	);

	const counts = $derived({
		total: components.length,
		vendored: components.filter((entry) => entry.kind === 'vendored').length,
		crate: components.filter((entry) => entry.kind === 'crate').length,
		npm: components.filter((entry) => entry.kind === 'npm').length,
		withoutText: components.filter((entry) => !entry.hasText).length
	});

	async function loadNotices() {
		if (notices) {
			notices = null;
			return;
		}
		loading = true;
		noticesError = null;
		try {
			notices = await call('third_party_licenses', {});
		} catch (caught) {
			noticesError = caught instanceof Error ? caught.message : String(caught);
		} finally {
			loading = false;
		}
	}
</script>

<div class="flex flex-col gap-3 p-4 sm:p-6">
	<header>
		<h1 class="text-xl font-semibold">{t.info.title}</h1>
		<p class="text-sm text-muted-foreground">{t.info.subtitle}</p>
	</header>

	<Card>
		<CardHeader>
			<CardTitle>OpenExamTrainer {APP_VERSION}</CardTitle>
			<CardDescription>{t.info.appBody}</CardDescription>
		</CardHeader>
		<CardContent class="flex flex-col gap-2 text-sm">
			<p>{t.info.offline}</p>
			<a
				href="https://github.com/thorstenalpers/OpenExamTrainer"
				target="_blank"
				rel="external noreferrer"
				class="underline underline-offset-2"
			>
				github.com/thorstenalpers/OpenExamTrainer
			</a>
			<p class="text-xs text-muted-foreground">{t.info.appLicense}</p>
		</CardContent>
	</Card>

	<Card>
		<CardHeader>
			<CardTitle>{t.info.thirdParty}</CardTitle>
			<CardDescription>
				{t.info.thirdPartyBody(counts.total, counts.vendored, counts.crate, counts.npm)}
			</CardDescription>
		</CardHeader>
		<CardContent class="flex flex-col gap-3">
			<p class="text-xs text-muted-foreground">{t.info.shipped}</p>

			<div class="flex flex-wrap items-center gap-3">
				<Input placeholder={t.info.filter} bind:value={viewState.infoFilter} class="max-w-sm" />
				<Button variant="outline" onclick={loadNotices} disabled={loading}>
					{loading ? t.common.loading : notices ? t.info.hideTexts : t.info.showTexts}
				</Button>
			</div>

			{#if noticesError}
				<p class="text-sm text-destructive">{noticesError}</p>
			{/if}

			{#if notices}
				<pre
					class="max-h-96 overflow-auto rounded-md border bg-muted/30 p-3 text-xs whitespace-pre-wrap">{notices}</pre>
			{/if}

			<ul class="flex max-h-96 flex-col divide-y overflow-y-auto text-sm">
				{#each shown as entry (entry.kind + entry.name + entry.version)}
					<li class="flex flex-wrap items-center gap-2 py-1.5">
						<span class="flex-1 truncate">
							{entry.name}
							<span class="text-xs text-muted-foreground">{entry.version}</span>
						</span>
						{#if entry.kind === 'vendored'}
							<Badge variant="accent">{t.info.redistributed}</Badge>
						{/if}
						{#if !entry.hasText}
							<Badge variant="neutral">{t.info.noOwnText}</Badge>
						{/if}
						<code class="text-xs text-muted-foreground">{entry.license}</code>
					</li>
				{:else}
					<li class="py-3 text-muted-foreground">{t.info.noMatch}</li>
				{/each}
			</ul>

			{#if counts.withoutText > 0}
				<p class="text-xs text-muted-foreground">{t.info.withoutText(counts.withoutText)}</p>
			{/if}
		</CardContent>
	</Card>

	<Card>
		<CardHeader>
			<CardTitle>{t.info.material}</CardTitle>
		</CardHeader>
		<CardContent class="text-sm text-muted-foreground">
			<p>{t.info.materialBody}</p>
		</CardContent>
	</Card>
</div>
