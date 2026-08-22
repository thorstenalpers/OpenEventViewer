import type { ColumnDef, FilterFn, RowData } from '@tanstack/table-core';
import type { EventRecord } from '$lib/bridge/contract';
import { inTimeRange, matchesNumberQuery, parseNumberQuery, type TimeRange } from '$lib/events';
import { createDataTable, type DataTable } from '$lib/table.svelte';

/** Which control the header row puts under a column. */
export type FilterKind = 'choice' | 'time' | 'number' | 'text';

declare module '@tanstack/table-core' {
	/* eslint-disable @typescript-eslint/no-unused-vars */
	interface ColumnMeta<TData extends RowData, TValue> {
		filter: FilterKind;
	}
	/* eslint-enable @typescript-eslint/no-unused-vars */
}

/** One of the values the column actually holds. An empty selection filters nothing. */
const oneOf: FilterFn<EventRecord> = (row, columnId, filterValue) => {
	const wanted = filterValue as string[] | undefined;
	if (!Array.isArray(wanted) || wanted.length === 0) return true;
	return wanted.includes(String(row.getValue<unknown>(columnId)));
};

const withinTime: FilterFn<EventRecord> = (row, columnId, filterValue) => {
	const range = filterValue as TimeRange | undefined;
	if (!range) return true;
	return inTimeRange(row.getValue<string>(columnId), range);
};

// Parsed per row, which is wasteful and invisible: the query is a dozen characters and the row
// count is capped at fifty thousand. Caching it would mean a second place the text can go stale.
const matchesNumber: FilterFn<EventRecord> = (row, columnId, filterValue) => {
	const text = filterValue as string | undefined;
	if (!text?.trim()) return true;
	return matchesNumberQuery(row.getValue<number>(columnId), parseNumberQuery(text));
};

/** The column ids double as i18n keys, so a column added here needs one label, not two. */
export const COLUMNS: ColumnDef<EventRecord, never>[] = [
	{
		id: 'level',
		accessorKey: 'levelName',
		header: 'level',
		filterFn: oneOf,
		meta: { filter: 'choice' }
	},
	{
		id: 'time',
		accessorKey: 'timeCreated',
		header: 'time',
		filterFn: withinTime,
		meta: { filter: 'time' }
	},
	{
		id: 'provider',
		accessorKey: 'provider',
		header: 'provider',
		filterFn: oneOf,
		meta: { filter: 'choice' }
	},
	{
		id: 'eventId',
		accessorKey: 'eventId',
		header: 'eventId',
		filterFn: matchesNumber,
		meta: { filter: 'number' }
	},
	{
		id: 'task',
		accessorKey: 'task',
		header: 'task',
		filterFn: oneOf,
		meta: { filter: 'choice' }
	},
	{
		id: 'channel',
		accessorKey: 'channel',
		header: 'channel',
		filterFn: oneOf,
		meta: { filter: 'choice' }
	},
	{
		id: 'computer',
		accessorKey: 'computer',
		header: 'computer',
		filterFn: oneOf,
		meta: { filter: 'choice' }
	},
	// The one column with no bounded set of values and no order worth comparing against.
	{
		id: 'message',
		accessorKey: 'message',
		header: 'message',
		filterFn: 'includesString',
		meta: { filter: 'text' }
	}
];

/** Fixed, in the same order as COLUMNS, plus the actions column the table adds itself. */
export const WIDTHS = ['7rem', '11rem', '15rem', '5.5rem', '9rem', '9rem', '8rem', 'auto', '3rem'];

export type EventsTableData = DataTable<EventRecord>;

export function createEventsTable(getRows: () => EventRecord[], key?: string): EventsTableData {
	return createDataTable<EventRecord>(getRows, COLUMNS, [{ id: 'time', desc: true }], key);
}
