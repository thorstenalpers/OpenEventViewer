import { SvelteSet } from 'svelte/reactivity';
import { call } from '$lib/bridge/client';
import type { EventFilter, EventRecord } from '$lib/bridge/contract';
import {
	ALL_CHANNELS,
	DEFAULT_CHANNELS,
	boundsOf,
	keyOf,
	listIn,
	numbersIn,
	type Range
} from '$lib/events';
import { settings } from '$lib/stores/settings.svelte';

/**
 * What the Events page is looking at.
 *
 * A singleton rather than per-view state: routing unmounts the page, and a trip to the assistant
 * and back should not throw away a query that took three seconds to answer.
 */
class EventsStore {
	channels = $state<string[]>([]);
	channel = $state<string>(ALL_CHANNELS);
	levels = new SvelteSet<number>([1, 2, 3]);
	range = $state<Range>('day');
	from = $state('');
	to = $state('');
	eventIdText = $state('');
	providerText = $state('');

	events = $state<EventRecord[]>([]);
	truncated = $state(false);
	elapsedMs = $state(0);
	loading = $state(false);
	error = $state<string | null>(null);
	selectedId = $state<string | null>(null);

	/** Whether the last failure was the Security channel refusing a non-elevated process. */
	get accessDenied(): boolean {
		return this.error?.includes('administrator rights') ?? false;
	}

	get selected(): EventRecord | null {
		return this.events.find((event) => keyOf(event) === this.selectedId) ?? null;
	}

	toggleLevel(level: number): void {
		if (!this.levels.delete(level)) this.levels.add(level);
	}

	select(event: EventRecord | null): void {
		this.selectedId = event ? keyOf(event) : null;
	}

	async loadChannels(): Promise<void> {
		try {
			this.channels = await call('events_channels', {});
		} catch {
			// A machine that will not enumerate its channels still reads the four everyone knows
			// about, so this must not stop the page from working.
			this.channels = [];
		}
	}

	toFilter(): EventFilter {
		const span = boundsOf(this.range, this.from, this.to);
		return {
			channels: this.channel === ALL_CHANNELS ? DEFAULT_CHANNELS : [this.channel],
			levels: [...this.levels].sort((left, right) => left - right),
			from: span.from,
			to: span.to,
			eventIds: numbersIn(this.eventIdText),
			providers: listIn(this.providerText),
			max: settings.eventsMaxRows
		};
	}

	async load(): Promise<void> {
		this.loading = true;
		this.error = null;
		try {
			const result = await call('events_query', { filter: this.toFilter() });
			this.events = result.events;
			this.truncated = result.truncated;
			this.elapsedMs = result.elapsedMs;
			this.selectedId = null;
		} catch (error) {
			this.error = error instanceof Error ? error.message : String(error);
			this.events = [];
			this.truncated = false;
		} finally {
			this.loading = false;
		}
	}
}

export const events = new EventsStore();
