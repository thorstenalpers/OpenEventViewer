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
