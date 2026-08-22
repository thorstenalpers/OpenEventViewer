import type { EventRecord } from '$lib/bridge/contract';

/** What the channel dropdown means by "everything worth reading without being an administrator". */
export const ALL_CHANNELS = '__all__';

export const DEFAULT_CHANNELS = ['System', 'Application'];

/** The four everyone reaches for, pinned above whatever else the machine publishes. */
export const PINNED_CHANNELS = ['System', 'Application', 'Security', 'Setup'];

export type Range = 'hour' | 'day' | 'week' | 'custom';

export const RANGES: Range[] = ['hour', 'day', 'week', 'custom'];

/** Every level a Windows event can carry, with 0 folded into Information by the host. */
export const LEVELS = [1, 2, 3, 4, 5] as const;

export type LevelKey = 'critical' | 'error' | 'warning' | 'information' | 'verbose';

const SPANS: Record<Exclude<Range, 'custom'>, number> = {
	hour: 60 * 60 * 1000,
	day: 24 * 60 * 60 * 1000,
	week: 7 * 24 * 60 * 60 * 1000
};

/** The i18n key for a level, so the label lives with the other translations rather than here. */
export function levelKey(level: number): LevelKey {
	switch (level) {
		case 1:
			return 'critical';
		case 2:
			return 'error';
		case 3:
			return 'warning';
		case 5:
			return 'verbose';
		default:
			return 'information';
	}
}

/** A record id is unique per channel, not per machine, so the channel has to be part of the key. */
export function keyOf(event: EventRecord): string {
	return `${event.channel}:${event.recordId}`;
}

/** The window the range names, as the host wants it: RFC 3339 in UTC, or null for open-ended. */
export function boundsOf(
	range: Range,
	from: string,
	to: string,
	now: number = Date.now()
): { from: string | null; to: string | null } {
	if (range === 'custom') {
		return { from: localToUtc(from), to: localToUtc(to) };
	}
	return { from: new Date(now - SPANS[range]).toISOString(), to: null };
}

/** `datetime-local` gives back wall-clock text with no zone; the host only speaks UTC. */
function localToUtc(value: string): string | null {
	if (!value) return null;
	const parsed = Date.parse(value);
	return Number.isNaN(parsed) ? null : new Date(parsed).toISOString();
}

/** `41, 6008 41` — separators are whatever the reader typed, and duplicates cost nothing. */
export function numbersIn(text: string): number[] {
	return [
		...new Set(
			text
				.split(/[^0-9]+/)
				.filter(Boolean)
				.map(Number)
		)
	];
}

export function listIn(text: string): string[] {
	return [
		...new Set(
			text
				.split(',')
				.map((part) => part.trim())
				.filter(Boolean)
		)
	];
}

/* -------------------------------------------------------------------------------------------- */
/* Column filters                                                                                  */
/* -------------------------------------------------------------------------------------------- */

/** One condition on a numeric column. */
export type NumberTerm =
	| { kind: 'eq'; value: number }
	| { kind: 'range'; from: number; to: number }
	| { kind: 'gt'; value: number; orEqual: boolean }
	| { kind: 'lt'; value: number; orEqual: boolean };

export interface NumberQuery {
	/** Alternatives. Empty means the query places no lower or upper bound at all. */
	terms: NumberTerm[];
	excluded: number[];
	/** Fragments that were not understood, so the box can say so rather than quietly dropping them. */
	invalid: string[];
}

export const EMPTY_NUMBER_QUERY: NumberQuery = { terms: [], excluded: [], invalid: [] };

/**
 * A numeric column filter in the shape people already type into a log search.
 *
 * `41` · `41, 6008` · `>7000` · `<=100` · `7000-7040` · `7000..7040` · `!10016`. Terms are
 * alternatives; an exclusion applies to all of them, because "everything but the DCOM noise" is
 * the one thing a single OR list cannot say.
 */
export function parseNumberQuery(text: string): NumberQuery {
	const query: NumberQuery = { terms: [], excluded: [], invalid: [] };

	for (const raw of text.split(/[,\s]+/).filter(Boolean)) {
		const part = raw.trim();
		let match: RegExpMatchArray | null;

		if ((match = /^!\s*(\d+)$/.exec(part))) {
			query.excluded.push(Number(match[1]));
		} else if ((match = /^(>=|>)\s*(\d+)$/.exec(part))) {
			query.terms.push({ kind: 'gt', value: Number(match[2]), orEqual: match[1] === '>=' });
		} else if ((match = /^(<=|<)\s*(\d+)$/.exec(part))) {
			query.terms.push({ kind: 'lt', value: Number(match[2]), orEqual: match[1] === '<=' });
		} else if ((match = /^(\d+)\s*(?:-|\.\.)\s*(\d+)$/.exec(part))) {
			const from = Number(match[1]);
			const to = Number(match[2]);
			query.terms.push({ kind: 'range', from: Math.min(from, to), to: Math.max(from, to) });
		} else if (/^\d+$/.test(part)) {
			query.terms.push({ kind: 'eq', value: Number(part) });
		} else {
			query.invalid.push(part);
		}
	}

	return query;
}

export function isEmptyNumberQuery(query: NumberQuery): boolean {
	return query.terms.length === 0 && query.excluded.length === 0;
}

export function matchesNumberQuery(value: number, query: NumberQuery): boolean {
	if (query.excluded.includes(value)) return false;
	if (query.terms.length === 0) return true;
	return query.terms.some((term) => {
		switch (term.kind) {
			case 'eq':
				return value === term.value;
			case 'range':
				return value >= term.from && value <= term.to;
			case 'gt':
				return term.orEqual ? value >= term.value : value > term.value;
			case 'lt':
				return term.orEqual ? value <= term.value : value < term.value;
		}
	});
}

/** Two `datetime-local` strings. Either half may be empty, which leaves that end open. */
export interface TimeRange {
	from: string;
	to: string;
}

export function isEmptyTimeRange(range: TimeRange): boolean {
	return !range.from && !range.to;
}

export function inTimeRange(iso: string, range: TimeRange): boolean {
	const at = Date.parse(iso);
	if (Number.isNaN(at)) return true;
	// `datetime-local` has no zone, so it parses as wall-clock on this machine — which is also how
	// the table shows the timestamp, so the two agree by construction.
	const from = range.from ? Date.parse(range.from) : Number.NaN;
	const to = range.to ? Date.parse(range.to) : Number.NaN;
	if (!Number.isNaN(from) && at < from) return false;
	if (!Number.isNaN(to) && at > to) return false;
	return true;
}

export interface Choice {
	value: string;
	count: number;
}

/** What a column actually holds, commonest first, so the useful values are not below the fold. */
export function choicesOf(values: string[]): Choice[] {
	const counts = new Map<string, number>();
	for (const value of values) counts.set(value, (counts.get(value) ?? 0) + 1);
	return [...counts]
		.map(([value, count]) => ({ value, count }))
		.sort((left, right) => right.count - left.count || left.value.localeCompare(right.value));
}

/* -------------------------------------------------------------------------------------------- */
/* Histogram                                                                                       */
/* -------------------------------------------------------------------------------------------- */

export interface Bucket {
	start: number;
	total: number;
	/** Critical and Error together: the part of the bar that is drawn in red. */
	errors: number;
}

export interface Histogram {
	buckets: Bucket[];
	sizeMs: number;
	from: number;
	to: number;
	peak: number;
}

export const EMPTY_HISTOGRAM: Histogram = {
	buckets: [],
	sizeMs: 0,
	from: 0,
	to: 0,
	peak: 0
};

const MINUTE = 60_000;

/**
 * Bucket widths a reader can hold in their head, rather than "3 minutes 47 seconds".
 *
 * The bars are also aligned to these, so a bar starts on the hour rather than on whenever the
 * oldest event in the window happened to be.
 */
const NICE_SIZES = [
	MINUTE,
	5 * MINUTE,
	15 * MINUTE,
	30 * MINUTE,
	60 * MINUTE,
	3 * 60 * MINUTE,
	6 * 60 * MINUTE,
	12 * 60 * MINUTE,
	24 * 60 * MINUTE,
	7 * 24 * 60 * MINUTE,
	30 * 24 * 60 * MINUTE
];

export function bucketSizeFor(spanMs: number, columns: number): number {
	return NICE_SIZES.find((size) => spanMs / size <= columns) ?? NICE_SIZES[NICE_SIZES.length - 1]!;
}

/**
 * The loaded events counted into equal buckets over the span they cover.
 *
 * Built from whatever the table is currently showing rather than from everything loaded: a chart
 * that ignores the filters would be describing a different question than the one on screen.
 */
export function histogram(
	events: { timeCreated: string; level: number }[],
	columns = 60
): Histogram {
	const times = events
		.map((event) => Date.parse(event.timeCreated))
		.filter((at) => !Number.isNaN(at));
	if (times.length === 0) return EMPTY_HISTOGRAM;

	const oldest = Math.min(...times);
	const newest = Math.max(...times);
	const sizeMs = bucketSizeFor(Math.max(newest - oldest, 1), columns);
	const from = Math.floor(oldest / sizeMs) * sizeMs;
	const to = Math.floor(newest / sizeMs) * sizeMs + sizeMs;

	const buckets: Bucket[] = [];
	for (let start = from; start < to; start += sizeMs) {
		buckets.push({ start, total: 0, errors: 0 });
	}

	for (const event of events) {
		const at = Date.parse(event.timeCreated);
		if (Number.isNaN(at)) continue;
		const bucket = buckets[Math.floor((at - from) / sizeMs)];
		if (!bucket) continue;
		bucket.total += 1;
		// Critical and Error together. Warning is not a failure and would make the red misleading.
		if (event.level === 1 || event.level === 2) bucket.errors += 1;
	}

	return {
		buckets,
		sizeMs,
		from,
		to,
		peak: buckets.reduce((most, bucket) => Math.max(most, bucket.total), 0)
	};
}
