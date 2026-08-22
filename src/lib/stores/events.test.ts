import { describe, expect, it } from 'vitest';
import { boundsOf, keyOf, levelKey, listIn, numbersIn } from '$lib/events';
import { events } from './events.svelte';

describe('events store', () => {
	it('turns the toolbar into the filter the host is given', () => {
		events.channel = 'System';
		events.levels.clear();
		events.levels.add(2);
		events.levels.add(1);
		events.range = 'hour';
		events.eventIdText = '41, 6008 41';
		events.providerText = ' EventLog , Microsoft-Windows-Kernel-Power ,';

		const filter = events.toFilter();

		expect(filter.channels).toEqual(['System']);
		expect(filter.levels).toEqual([1, 2]);
		expect(filter.eventIds).toEqual([41, 6008]);
		expect(filter.providers).toEqual(['EventLog', 'Microsoft-Windows-Kernel-Power']);
		expect(filter.to).toBeNull();
		expect(filter.from).not.toBeNull();
	});

	/// Reading everything is what the page opens on, and "everything" without administrator rights
	/// is the two channels a normal account can always read.
	it('asks for System and Application when no channel is chosen', () => {
		events.channel = '__all__';

		expect(events.toFilter().channels).toEqual(['System', 'Application']);
	});

	it('measures a named range back from now and leaves a custom one open at both ends', () => {
		const now = Date.parse('2026-08-22T12:00:00.000Z');

		expect(boundsOf('day', '', '', now)).toEqual({
			from: '2026-08-21T12:00:00.000Z',
			to: null
		});
		expect(boundsOf('custom', '', '', now)).toEqual({ from: null, to: null });
	});

	it('reads a wall-clock custom range back as UTC', () => {
		const window = boundsOf('custom', '2026-08-20T08:30', '2026-08-20T09:30');

		expect(window.from).toBe(new Date('2026-08-20T08:30').toISOString());
		expect(window.to).toBe(new Date('2026-08-20T09:30').toISOString());
	});

	it('accepts ids typed with whatever separator came to hand', () => {
		expect(numbersIn('41,6008  41; 137')).toEqual([41, 6008, 137]);
		expect(numbersIn('')).toEqual([]);
		expect(listIn('a, ,b,a')).toEqual(['a', 'b']);
	});

	/// A record id is unique within its channel, not across the machine — two channels can both
	/// hold a record 7, and keying on the number alone would select both.
	it('keys a row by channel and record id together', () => {
		expect(keyOf({ channel: 'System', recordId: 7 } as never)).toBe('System:7');
	});

	it('names every level, and treats log-always as information', () => {
		expect(levelKey(0)).toBe('information');
		expect(levelKey(1)).toBe('critical');
		expect(levelKey(4)).toBe('information');
		expect(levelKey(5)).toBe('verbose');
	});
});
