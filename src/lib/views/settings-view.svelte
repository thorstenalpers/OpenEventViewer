<script lang="ts">
	import { setMode, userPrefersMode } from 'mode-watcher';
	import DownloadIcon from '@lucide/svelte/icons/download';
	import PlayIcon from '@lucide/svelte/icons/play';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Badge } from '$lib/components/ui/badge';
	import { Select } from '$lib/components/ui/select';
	import { call } from '$lib/bridge/client';
	import type { AssistantSource, AssistantStatus } from '$lib/bridge/contract';
	import { i18n, LOCALES, isLocale } from '$lib/i18n/index.svelte';
	import { THEME_PRESETS, isThemePreset } from '$lib/theme/preset';
	import { settings } from '$lib/stores/settings.svelte';
	import { library } from '$lib/stores/library.svelte';
	import { voice, type VoiceSample } from '$lib/stores/voice.svelte';

	const t = $derived(i18n.t);

	let sampleKind = $state('all');
	let ownQuestion = $state<string | null>(null);

	// The one sample that is not written here: what the voice will actually be asked to read. Only
	// the first line of it, because a stem can run to a paragraph and this is a preview.
	$effect(() => {
		const binder = library.selected;
		if (!binder || binder.questionCount === 0) {
			ownQuestion = null;
			return;
		}
		void call('list_questions', { binderId: binder.id })
			.then((questions) => {
				const first = questions[0];
				const stem = (first?.stem.split('\n')[0] ?? '').trim();
				ownQuestion = stem ? stem.slice(0, 200) : null;
			})
			.catch(() => (ownQuestion = null));
	});

	const spokenTexts = $derived<Record<string, string>>({
		pangram: t.settings.voiceSpoken.pangram,
		balanced: t.settings.voiceSpoken.balanced,
		passage: t.settings.voiceSpoken.passage,
		named: t.settings.voiceSpoken.named(voice.chosenLabel ?? t.settings.voiceSystem),
		...(ownQuestion ? { question: ownQuestion } : {})
	});

	const sampleOptions = $derived([
		{ value: 'all', label: t.settings.voiceSamples.all },
		...Object.keys(spokenTexts).map((key) => ({
			value: key,
			label: t.settings.voiceSamples[key as keyof typeof t.settings.voiceSamples]
		}))
	]);

	// The question sample disappears with the project it came from; a stale choice would leave the
	// dropdown showing nothing.
	$effect(() => {
		if (!sampleOptions.some((option) => option.value === sampleKind)) sampleKind = 'all';
	});

	const samples = $derived<VoiceSample[]>(
		sampleKind === 'all'
			? Object.entries(spokenTexts).map(([key, text]) => ({
					label: `${t.settings.voiceSamples[key as keyof typeof t.settings.voiceSamples]}.`,
					text
				}))
			: [{ text: spokenTexts[sampleKind] ?? t.settings.voiceSpoken.pangram }]
	);

	const ROW = 'flex flex-wrap items-center justify-between gap-x-4 gap-y-2 px-3 py-2.5';

	const modes = $derived([
		{ id: 'system', label: t.settings.system },
		{ id: 'light', label: t.settings.light },
		{ id: 'dark', label: t.settings.dark }
	] as const);

	const sources = $derived([
		{
			id: 'cli' as AssistantSource,
			label: t.settings.sourceCliLabel,
			detail: t.settings.sourceCliDetail
		},
		{
			id: 'anthropic' as AssistantSource,
			label: t.settings.sourceAnthropicLabel,
			detail: t.settings.sourceAnthropicDetail
		}
	]);

	const localeOptions = LOCALES.map((locale) => ({ value: locale.id, label: locale.label }));
	// The Windows voice first, because it is what reads before anything has been downloaded.
	const voiceOptions = $derived([
		{ value: '', label: t.settings.voiceWindows },
		...voice.speakers.map((speaker) => ({ value: speaker.value, label: speaker.label }))
	]);
	const presetOptions = $derived(
		THEME_PRESETS.map((preset) => ({ value: preset, label: t.settings.presets[preset] ?? preset }))
	);

	let status = $state<AssistantStatus | null>(null);
	let key = $state('');
	let notice = $state<string | null>(null);
	let error = $state<string | null>(null);

	let locale = $state(settings.locale);
	let preset = $state(settings.preset);

	$effect(() => {
		if (isLocale(locale) && locale !== settings.locale) settings.setLocale(locale);
	});

	$effect(() => {
		if (isThemePreset(preset) && preset !== settings.preset) settings.setPreset(preset);
	});

	$effect(() => {
		const source = settings.assistantSource;
		call('assistant_status', { source })
			.then((result) => (status = result))
			.catch(() => (status = null));
	});

	let loggingError = $state<string | null>(null);

	async function logging(changes: { showLogs?: boolean; debugLogging?: boolean }) {
		loggingError = null;
		try {
			await settings.setLogging(changes);
		} catch (caught) {
			loggingError = caught instanceof Error ? caught.message : String(caught);
		}
	}

	async function storeKey() {
		error = null;
		notice = null;
		try {
			await call('assistant_set_key', { key });
			// Cleared immediately: the field is a one-way door, and the key cannot be read back.
			key = '';
			notice = t.settings.stored;
			status = await call('assistant_status', { source: settings.assistantSource });
		} catch (caught) {
			error = caught instanceof Error ? caught.message : String(caught);
		}
	}
</script>

{#snippet describe(title: string, detail: string)}
	<span class="flex min-w-0 flex-1 flex-col">
		<span class="text-sm font-medium">{title}</span>
		<span class="text-xs text-muted-foreground">{detail}</span>
	</span>
{/snippet}

<div class="flex max-w-2xl flex-col gap-3 p-4 sm:p-6">
	<h1 class="text-xl font-semibold">{t.settings.title}</h1>

	<!-- One row per setting rather than one card per setting: five bordered boxes around five
	     single controls is mostly chrome, and the row labels say everything the headers did. -->
	<Card>
		<CardContent class="flex flex-col divide-y p-0">
			<div class={ROW}>
				{@render describe(t.settings.appearance, t.settings.appearanceBody)}
				<div class="flex flex-wrap gap-1.5">
					{#each modes as mode (mode.id)}
						<Button
							size="sm"
							variant={userPrefersMode.current === mode.id ? 'default' : 'outline'}
							onclick={() => setMode(mode.id)}
						>
							{mode.label}
						</Button>
					{/each}
				</div>
			</div>

			<div class={ROW}>
				{@render describe(t.settings.language, t.settings.languageBody)}
				<Select
					bind:value={locale}
					options={localeOptions}
					aria-label={t.settings.language}
					class="w-44"
				/>
			</div>

			<div class={ROW}>
				{@render describe(t.settings.voice, t.settings.voiceBody)}
				<div class="flex flex-wrap items-center gap-2">
					<Select
						value={voice.choice}
						options={voiceOptions}
						onchange={(event: Event) =>
							voice.setChoice((event.currentTarget as HTMLSelectElement).value)}
						aria-label={t.settings.voice}
						class="w-56"
					/>
					{#if voice.speaking}
						<Button size="sm" variant="outline" onclick={() => voice.stop()}>
							{t.settings.voiceStopPreview}
						</Button>
					{:else}
						<!-- A split button: the play half acts, the other half chooses what it plays. The
						     right half is a bare <select> rather than a menu built from divs, so it keeps
						     the platform's own popup, its keyboard handling and its screen reader role. -->
						<div class="inline-flex items-stretch">
							<Button
								size="sm"
								variant="outline"
								class="rounded-e-none"
								onclick={() => voice.preview(samples, settings.locale)}
							>
								<PlayIcon class="size-4" />
								{t.settings.voicePreview}
							</Button>
							<div class="relative inline-flex items-center">
								<select
									bind:value={sampleKind}
									aria-label={t.settings.voiceSampleLabel}
									class="h-8 max-w-48 cursor-pointer appearance-none truncate rounded-md rounded-s-none border border-s-0 border-input bg-background ps-2 pe-7 text-xs shadow-sm transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
								>
									{#each sampleOptions as option (option.value)}
										<option value={option.value}>{option.label}</option>
									{/each}
								</select>
								<ChevronDownIcon class="pointer-events-none absolute end-2 size-3.5 opacity-60" />
							</div>
						</div>
					{/if}
				</div>
			</div>

			<div class={ROW}>
				{@render describe(t.settings.voicePacks, t.settings.voicePacksBody)}
				<div class="flex flex-col items-end gap-1">
					{#each voice.packs as pack (pack.id)}
						<div class="flex items-center gap-2 text-xs">
							<span class="text-muted-foreground">{pack.label}</span>
							{#if pack.installed}
								<Badge variant="neutral">{t.settings.voiceInstalled(pack.voices)}</Badge>
								<Button
									size="sm"
									variant="ghost"
									class="h-7 px-2"
									onclick={() => voice.remove(pack.id)}
								>
									{t.settings.voiceRemove}
								</Button>
							{:else if voice.isInstalling(pack.id)}
								<div class="h-1.5 w-24 overflow-hidden rounded-full bg-muted">
									<div
										class="h-full bg-primary transition-[width]"
										style:width={voice.percentOf(pack.id) === null
											? '10%'
											: `${voice.percentOf(pack.id)}%`}
									></div>
								</div>
								{#if voice.isUnpacking(pack.id)}
									<span class="text-muted-foreground">{t.settings.voiceUnpacking}</span>
								{:else}
									<span class="w-8 text-muted-foreground">
										{voice.percentOf(pack.id) === null ? '' : `${voice.percentOf(pack.id)}%`}
									</span>
									<Button
										size="sm"
										variant="ghost"
										class="h-7 px-2"
										onclick={() => voice.cancel(pack.id)}
									>
										{t.settings.voiceCancel}
									</Button>
								{/if}
							{:else}
								<span class="text-muted-foreground">{t.settings.voiceSize(pack.megabytes)}</span>
								<Button
									size="sm"
									variant="outline"
									class="h-7 px-2"
									onclick={() => voice.install(pack.id)}
									aria-label={`${t.settings.voiceDownload} — ${pack.label}`}
								>
									<DownloadIcon class="size-3.5" />
								</Button>
							{/if}
						</div>
					{/each}
					{#if voice.error}
						<p class="text-xs text-destructive">{voice.error}</p>
					{/if}
				</div>
			</div>

			<div class={ROW}>
				{@render describe(t.settings.colours, t.settings.coloursBody)}
				<Select
					bind:value={preset}
					options={presetOptions}
					aria-label={t.settings.colours}
					class="w-44"
				/>
			</div>

			<label class="{ROW} cursor-pointer select-none">
				{@render describe(t.settings.showLogs, t.settings.showLogsBody)}
				<input
					type="checkbox"
					class="size-4"
					checked={settings.showLogs}
					onchange={(event) => logging({ showLogs: event.currentTarget.checked })}
				/>
			</label>

			<label class="{ROW} cursor-pointer select-none">
				{@render describe(t.settings.debugLogging, t.settings.debugLoggingBody)}
				<input
					type="checkbox"
					class="size-4"
					checked={settings.debugLogging}
					onchange={(event) => logging({ debugLogging: event.currentTarget.checked })}
				/>
			</label>

			{#if loggingError}
				<p class="px-3 py-2 text-sm text-destructive">{loggingError}</p>
			{/if}
		</CardContent>
	</Card>

	<Card>
		<CardHeader>
			<CardTitle>{t.settings.assistant}</CardTitle>
			<CardDescription>{t.settings.assistantBody}</CardDescription>
		</CardHeader>
		<CardContent class="flex flex-col gap-3">
			<div class="flex flex-col gap-1.5">
				{#each sources as source (source.id)}
					<button
						type="button"
						onclick={() => (settings.assistantSource = source.id)}
						class="flex flex-col items-start gap-0.5 rounded-md border px-3 py-2 text-start text-sm transition-colors {settings.assistantSource ===
						source.id
							? 'border-primary bg-primary/5'
							: 'hover:bg-muted/50'}"
					>
						<span class="flex items-center gap-2 font-medium">
							{source.label}
							{#if source.id === 'cli' && status?.cliAvailable}
								<Badge variant="accent">{t.settings.found}</Badge>
							{:else if source.id === 'anthropic' && status?.hasKey}
								<Badge variant="accent">{t.settings.keyStored}</Badge>
							{/if}
						</span>
						<span class="text-xs text-muted-foreground">{source.detail}</span>
					</button>
				{/each}
			</div>

			{#if settings.assistantSource === 'anthropic'}
				<div class="flex items-end gap-2">
					<label class="flex flex-1 flex-col gap-1 text-xs text-muted-foreground">
						{t.settings.apiKey}
						<Input type="password" bind:value={key} placeholder="sk-ant-…" />
					</label>
					<Button onclick={storeKey} disabled={!key.trim()}>{t.settings.store}</Button>
				</div>
				<p class="text-xs text-muted-foreground">{t.settings.keyNote}</p>
			{/if}

			{#if notice}
				<p class="text-sm text-muted-foreground">{notice}</p>
			{/if}
			{#if error}
				<p class="text-sm text-destructive">{error}</p>
			{/if}
		</CardContent>
	</Card>
</div>
