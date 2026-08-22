import type { ColumnDef } from '@tanstack/table-core';
import type { EventRecord } from '$lib/bridge/contract';
import { createDataTable, type DataTable } from '$lib/table.svelte';

/** The column ids double as i18n keys, so a column added here needs one label, not two. */
export const COLUMNS: ColumnDef<EventRecord, never>[] = [
	{ id: 'level', accessorKey: 'levelName', header: 'level', filterFn: 'includesString' },
	{ id: 'time', accessorKey: 'timeCreated', header: 'time', filterFn: 'includesString' },
	{ id: 'provider', accessorKey: 'provider', header: 'provider', filterFn: 'includesString' },
	{ id: 'eventId', accessorKey: 'eventId', header: 'eventId', filterFn: 'includesString' },
	{ id: 'task', accessorKey: 'task', header: 'task', filterFn: 'includesString' },
	{ id: 'channel', accessorKey: 'channel', header: 'channel', filterFn: 'includesString' },
	{ id: 'computer', accessorKey: 'computer', header: 'computer', filterFn: 'includesString' },
	{ id: 'message', accessorKey: 'message', header: 'message', filterFn: 'includesString' }
];

/** Fixed, in the same order as COLUMNS, plus the actions column the table adds itself. */
export const WIDTHS = ['6.5rem', '11rem', '15rem', '5rem', '9rem', '9rem', '8rem', 'auto', '3rem'];

export type EventsTableData = DataTable<EventRecord>;

export function createEventsTable(getRows: () => EventRecord[], key?: string): EventsTableData {
	return createDataTable<EventRecord>(getRows, COLUMNS, [{ id: 'time', desc: true }], key);
}
