import type { Episode, ImportReport } from '$lib/bridge/contract';

/**
 * What a view had on screen, kept beyond its own lifetime.
 *
 * Routing unmounts a page, so anything held in a `let … = $state()` inside a view is gone the
 * moment you click elsewhere. For most of it that is correct — a list refetches, an error should
 * not outlive the thing that caused it. These are the ones where it is not: work the user did by
 * hand, and results that took real time to produce.
 *
 * Module-level runes rather than a snapshot per route: the state belongs to the app, not to a
 * history entry, so it survives a click as well as a back button.
 */
class ViewState {
	/** The import summary. Losing it means importing, glancing at Train, and finding no report. */
	importReport = $state<ImportReport | null>(null);

	/** The video form, half-typed. */
	video = $state({
		title: '',
		url: '',
		startAt: '',
		anchoredTo: null as number | null
	});

	/** Podcast settings and the last episode, which costs a synthesis run to reproduce. */
	podcast = $state({
		includeAnswer: true,
		includeExplanation: true,
		pauseSeconds: 4,
		format: 'mp3',
		language: 'en',
		episode: null as Episode | null
	});

	/** Challenge rules — a seed typed in and then lost is a challenge you cannot repeat. */
	challenge = $state({
		seed: 4711,
		questionCount: 10,
		minutes: 10
	});

	infoFilter = $state('');

	/** The address bar. Leaving Browse and returning should not send you back to the start page. */
	browseAddress = $state('https://learn.microsoft.com/en-us/training/');
}

export const viewState = new ViewState();
