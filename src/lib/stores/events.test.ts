import { describe, expect, it } from 'vitest';
import { keyOf } from '$lib/events';
import { events } from './events.svelte';

describe('events store', () => {
	it('turns the toolbar into the filter the host is given', () => {
		events.channel = 'System';

		expect(events.toFilter().channels).toEqual(['System']);
	});

	/// Everything but the channel is a column filter in the table, so the host query leaves the
	/// rest open and the row cap alone decides how far back the answer reaches.
	it('asks for everything the channel holds, newest first', () => {
		const filter = events.toFilter();

		expect(filter.levels).toEqual([]);
		expect(filter.from).toBeNull();
		expect(filter.to).toBeNull();
		expect(filter.eventIds).toEqual([]);
		expect(filter.providers).toEqual([]);
	});

	/// Reading everything is what the page opens on, and "everything" without administrator rights
	/// is the two channels a normal account can always read.
	it('asks for System and Application when no channel is chosen', () => {
		events.channel = '__all__';

		expect(events.toFilter().channels).toEqual(['System', 'Application']);
	});

	/// A record id is unique within its channel, not across the machine — two channels can both
	/// hold a record 7, and keying on the number alone would select both.
	it('keys a row by channel and record id together', () => {
		expect(keyOf({ channel: 'System', recordId: 7 } as never)).toBe('System:7');
	});
});
