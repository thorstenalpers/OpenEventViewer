<script lang="ts">
	import { call } from '$lib/bridge/client';
	import { i18n } from '$lib/i18n/index.svelte';

	interface Props {
		hashes: string[];
	}

	const { hashes }: Props = $props();
	const t = $derived(i18n.t);

	let sources = $state<string[]>([]);
	let failed = $state(false);

	// The bytes are fetched rather than shipped inside the question: a binder holds dozens of
	// screenshots and only the card on screen needs one.
	$effect(() => {
		const wanted = hashes;
		if (!wanted.length) {
			sources = [];
			return;
		}

		let current = true;
		Promise.all(wanted.map((hash) => call('question_figure', { hash })))
			.then((urls) => {
				if (current) {
					sources = urls;
					failed = false;
				}
			})
			.catch(() => {
				if (current) {
					sources = [];
					failed = true;
				}
			});
		return () => {
			current = false;
		};
	});
</script>

{#if failed}
	<p class="rounded-md bg-warning/10 px-3 py-2 text-sm">{t.question.figureUnavailable}</p>
{:else}
	{#each sources as source, index (source)}
		<img
			src={source}
			alt={t.question.figureAlt(index + 1)}
			class="max-w-full rounded-md border bg-white"
		/>
	{/each}
{/if}
