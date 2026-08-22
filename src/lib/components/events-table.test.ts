import { describe, expect, it } from 'vitest';
import { render } from '@testing-library/svelte';
import { flushSync } from 'svelte';
import EventsTable from './events-table.svelte';
import { createEventsTable } from '$lib/stores/events-table.svelte';
import type { EventRecord } from '$lib/bridge/contract';

function rows(count: number): EventRecord[] {
	return Array.from({ length: count }, (_, index) => ({
		recordId: count - index,
		channel: index % 2 === 0 ? 'System' : 'Application',
		provider: index % 3 === 0 ? 'EventLog' : 'Service Control Manager',
		eventId: 1000 + (index % 7),
		level: (index % 4) + 1,
		levelName: ['Critical', 'Error', 'Warning', 'Information'][index % 4] ?? 'Information',
		task: 'None',
		keywords: [],
		timeCreated: new Date(Date.UTC(2026, 7, 20, 0, 0, index)).toISOString(),
		computer: 'WORKBENCH',
		message: `event number ${index}`,
		eventData: []
	}));
}

describe('events table', () => {
	/// The whole point of the hand-rolled windowing: eight thousand rows must not become eight
	/// thousand table rows, or the first keystroke in a filter costs a second.
	it('paints a window of the rows rather than all of them', () => {
		const data = createEventsTable(() => rows(8000));
		const { container } = render(EventsTable, { props: { data } });

		const painted = container.querySelectorAll('tbody tr[style*="height: 32px"]');
		expect(painted.length).toBeGreaterThan(0);
		expect(painted.length).toBeLessThan(100);
	});

	it('keeps the scroll height of every row that is not painted', () => {
		const data = createEventsTable(() => rows(8000));
		const { container } = render(EventsTable, { props: { data } });

		const spacers = [...container.querySelectorAll('tbody tr')].filter(
			(row) => row.children.length === 0
		);
		const held = spacers.reduce(
			(total, row) => total + Number.parseInt((row as HTMLElement).style.height, 10),
			0
		);
		const painted = container.querySelectorAll('tbody tr[style*="height: 32px"]').length;

		expect(held + painted * 32).toBe(8000 * 32);
	});

	it('narrows to what a column filter asks for', () => {
		const data = createEventsTable(() => rows(200));
		render(EventsTable, { props: { data } });

		data.table.getColumn('channel')?.setFilterValue('Application');
		flushSync();

		expect(data.table.getRowModel().rows.length).toBe(100);
		expect(
			data.table.getRowModel().rows.every((row) => row.original.channel === 'Application')
		).toBe(true);
	});

	it('narrows on a keyword from any column at all', () => {
		const data = createEventsTable(() => rows(200));
		render(EventsTable, { props: { data } });

		data.globalFilter = 'event number 42';
		flushSync();

		expect(data.table.getRowModel().rows).toHaveLength(1);
	});
});
