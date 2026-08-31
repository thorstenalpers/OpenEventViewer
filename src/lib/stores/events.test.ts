import { describe, expect, it } from 'vitest';
import { keyOf } from '$lib/events';
import { events } from './events.svelte';

describe('events store', () => {
	/// The query decides which log and how much of it; everything about *which rows* is a column
	/// filter. Asking the same question in both places is what made the toolbar look duplicated.
	it('asks the host for a channel and a row count, and nothing else', () => {
		events.channel = 'System';

		const filter = events.toFilter();

		expect(filter.channels).toEqual(['System']);
		expect(filter.max).toBeGreaterThan(0);
		expect(filter).toMatchObject({ levels: [], eventIds: [], providers: [], from: null, to: null });
	});

	/// Reading everything is what the page opens on, and "everything" without administrator rights
	/// is the two channels a normal account can always read.
	it('asks for System and Application when no channel is chosen', () => {
		events.channel = '__all__';

		expect(events.toFilter().channels).toEqual(['System', 'Application']);
	});

	/// A column filter can only narrow what was loaded. Without the span on screen, an empty result
	/// for last Tuesday looks exactly like a quiet Tuesday.
	it('reports how far back what it holds actually reaches', async () => {
		events.channel = '__all__';
		await events.load();

		expect(events.events.length).toBeGreaterThan(0);
		expect(events.oldest).not.toBeNull();
		expect(events.newest).not.toBeNull();
		expect(events.oldest! <= events.newest!).toBe(true);

		const times = events.events.map((event) => event.timeCreated);
		expect(events.oldest).toBe([...times].sort()[0]);
		expect(events.newest).toBe([...times].sort().at(-1));
	});

	it('has no span to report before anything is loaded', () => {
		events.events = [];

		expect(events.oldest).toBeNull();
		expect(events.newest).toBeNull();
	});

	/// A record id is unique within its channel, not across the machine — two channels can both
	/// hold a record 7, and keying on the number alone would select both.
	it('keys a row by channel and record id together', () => {
		expect(keyOf({ channel: 'System', recordId: 7 } as never)).toBe('System:7');
	});
});
