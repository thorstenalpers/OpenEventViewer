import { describe, expect, it } from 'vitest';
import {
	bucketSizeFor,
	choicesOf,
	errorTally,
	histogram,
	inTimeRange,
	isEmptyNumberQuery,
	matchesNumberQuery,
	parseNumberQuery
} from './events';

function matches(text: string, value: number): boolean {
	return matchesNumberQuery(value, parseNumberQuery(text));
}

describe('number query', () => {
	it('takes a single id', () => {
		expect(matches('41', 41)).toBe(true);
		expect(matches('41', 42)).toBe(false);
	});

	it('treats a list as alternatives, however it was separated', () => {
		expect(matches('41, 6008 7031', 6008)).toBe(true);
		expect(matches('41, 6008 7031', 7031)).toBe(true);
		expect(matches('41, 6008 7031', 1000)).toBe(false);
	});

	it('compares above and below, with and without the edge', () => {
		expect(matches('>7000', 7001)).toBe(true);
		expect(matches('>7000', 7000)).toBe(false);
		expect(matches('>=7000', 7000)).toBe(true);
		expect(matches('<100', 99)).toBe(true);
		expect(matches('<=100', 100)).toBe(true);
		expect(matches('<=100', 101)).toBe(false);
	});

	it('takes a range either way round, with both ends included', () => {
		expect(matches('7000-7040', 7000)).toBe(true);
		expect(matches('7000-7040', 7040)).toBe(true);
		expect(matches('7000-7040', 7041)).toBe(false);
		expect(matches('7040..7000', 7020)).toBe(true);
	});

	/// "Everything but the DCOM noise" is the one thing a list of alternatives cannot say, so an
	/// exclusion applies across all of them rather than being another alternative.
	it('excludes across every alternative, and excluding alone still matches the rest', () => {
		expect(matches('!10016', 10016)).toBe(false);
		expect(matches('!10016', 41)).toBe(true);
		expect(matches('>1000 !10016', 10016)).toBe(false);
		expect(matches('>1000 !10016', 7031)).toBe(true);
	});

	it('reports what it could not read rather than dropping it', () => {
		const query = parseNumberQuery('41, abc, >x');

		expect(query.invalid).toEqual(['abc', '>x']);
		expect(query.terms).toEqual([{ kind: 'eq', value: 41 }]);
	});

	it('is empty when nothing was asked, and an empty query matches anything', () => {
		expect(isEmptyNumberQuery(parseNumberQuery(''))).toBe(true);
		expect(matches('', 4711)).toBe(true);
	});
});

describe('time range', () => {
	it('leaves an end open when that half is blank', () => {
		const iso = new Date(2026, 7, 20, 12, 0).toISOString();

		expect(inTimeRange(iso, { from: '2026-08-20T11:00', to: '' })).toBe(true);
		expect(inTimeRange(iso, { from: '2026-08-20T13:00', to: '' })).toBe(false);
		expect(inTimeRange(iso, { from: '', to: '2026-08-20T13:00' })).toBe(true);
		expect(inTimeRange(iso, { from: '', to: '2026-08-20T11:00' })).toBe(false);
	});

	/// `datetime-local` carries no zone, so it has to be read as wall clock — the same clock the
	/// table prints, or the filter would exclude the rows it appears to name.
	it('reads both ends as local time, like the column it filters', () => {
		const noon = new Date(2026, 7, 20, 12, 0).toISOString();

		expect(inTimeRange(noon, { from: '2026-08-20T11:59', to: '2026-08-20T12:01' })).toBe(true);
	});
});

describe('choices', () => {
	it('counts the values and puts the commonest first', () => {
		expect(choicesOf(['b', 'a', 'b', 'c', 'b', 'a'])).toEqual([
			{ value: 'b', count: 3 },
			{ value: 'a', count: 2 },
			{ value: 'c', count: 1 }
		]);
	});
});

describe('histogram', () => {
	function event(minutesAgo: number, level: number) {
		return {
			timeCreated: new Date(Date.UTC(2026, 7, 20, 12, 0) - minutesAgo * 60_000).toISOString(),
			level
		};
	}

	it('has nothing to show for no events', () => {
		expect(histogram([]).buckets).toHaveLength(0);
	});

	it('counts every event exactly once', () => {
		const chart = histogram([event(0, 4), event(30, 2), event(90, 1), event(200, 4)]);

		expect(chart.buckets.reduce((sum, bucket) => sum + bucket.total, 0)).toBe(4);
		expect(chart.buckets.reduce((sum, bucket) => sum + bucket.errors, 0)).toBe(2);
	});

	/// Warning is not a failure. Colouring it red would make the chart say something the level
	/// column does not — it gets its own amber slice instead.
	it('counts critical and error as the red part and warning as the amber one', () => {
		const chart = histogram([event(0, 1), event(1, 2), event(2, 3), event(3, 4), event(4, 5)]);

		expect(chart.buckets.reduce((sum, bucket) => sum + bucket.errors, 0)).toBe(2);
		expect(chart.buckets.reduce((sum, bucket) => sum + bucket.warnings, 0)).toBe(1);
		expect(chart.buckets.reduce((sum, bucket) => sum + bucket.total, 0)).toBe(5);
	});

	it('picks a bucket size a reader can hold in their head', () => {
		expect(bucketSizeFor(60 * 60_000, 60)).toBe(60_000);
		expect(bucketSizeFor(24 * 60 * 60_000, 60)).toBe(30 * 60_000);
		expect(bucketSizeFor(7 * 24 * 60 * 60_000, 60)).toBe(3 * 60 * 60_000);
	});

	it('aligns the first bar to the bucket size rather than to the oldest event', () => {
		const chart = histogram([event(0, 4), event(59, 4)], 60);

		expect(chart.from % chart.sizeMs).toBe(0);
		expect(chart.to).toBeGreaterThan(chart.from);
	});

	/// Filtering to a quiet day must show a quiet day, not zoom into its one busy minute.
	it('pins the axis to a given span instead of the events that happen to match', () => {
		const at = Date.UTC(2026, 7, 20, 12, 0);
		const chart = histogram([event(0, 4)], 60, {
			from: at - 12 * 60 * 60_000,
			to: at
		});

		expect(chart.to - chart.from).toBeGreaterThanOrEqual(12 * 60 * 60_000);
		expect(chart.buckets.reduce((sum, bucket) => sum + bucket.total, 0)).toBe(1);
	});
});

describe('error tally', () => {
	function event(minute: number, level: number, provider: string, eventId: number) {
		return {
			timeCreated: new Date(Date.UTC(2026, 7, 20, 12, minute)).toISOString(),
			level,
			provider,
			eventId
		};
	}

	const from = Date.UTC(2026, 7, 20, 12, 0);
	const to = Date.UTC(2026, 7, 20, 12, 15);

	it('counts only what went wrong, commonest first', () => {
		const tally = errorTally(
			[
				event(1, 2, 'Application Error', 1000),
				event(2, 3, 'DCOM', 10016),
				event(3, 2, 'Application Error', 1000),
				event(4, 1, 'BugCheck', 1001),
				event(5, 4, 'ESENT', 326)
			],
			from,
			to
		);

		expect(tally).toEqual([
			{ label: 'Application Error · 1000', count: 2 },
			{ label: 'BugCheck · 1001', count: 1 }
		]);
	});

	/// The bucket is half-open, or an event on a boundary is counted into both bars beside it.
	it('takes the start of the window and leaves its end to the next bucket', () => {
		const at = (minute: number) => errorTally([event(minute, 2, 'A', 1)], from, to);

		expect(at(0)).toHaveLength(1);
		expect(at(14)).toHaveLength(1);
		expect(at(15)).toHaveLength(0);
	});

	/// The same fault writes the same provider and id every time; grouping by message would split
	/// one problem into forty because the text carries a path or a process id.
	it('groups by provider and id rather than by the text', () => {
		const tally = errorTally(
			[
				{ ...event(1, 2, 'Application Error', 1000) },
				{ ...event(2, 2, 'Application Error', 1000) }
			],
			from,
			to
		);

		expect(tally).toEqual([{ label: 'Application Error · 1000', count: 2 }]);
	});
});
