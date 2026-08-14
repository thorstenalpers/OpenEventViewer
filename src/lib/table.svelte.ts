import {
	createTable,
	getCoreRowModel,
	getFilteredRowModel,
	getSortedRowModel,
	type ColumnDef,
	type RowData,
	type SortingState,
	type Table,
	type TableOptionsResolved,
	type TableState,
	type Updater
} from '@tanstack/table-core';

/**
 * Sort and filter that outlive the view.
 *
 * Routing unmounts a page, so a table's state would otherwise reset on every trip to Settings and
 * back. Keyed by table so two tables never share a filter.
 */
const KEPT: Record<string, { sorting: SortingState; globalFilter: string }> = {};

/**
 * TanStack Table driven by runes.
 *
 * The instance is rebuilt whenever data or state changes rather than mutated through
 * `table.setOptions`. Row models memoise per instance, so this trades a cache that the row counts
 * here never benefit from for a data flow with no hidden mutation.
 */
export function createDataTable<TData extends RowData>(
	getRows: () => TData[],
	columns: ColumnDef<TData, never>[],
	initialSorting: SortingState = [],
	/** Set to keep this table's sort and filter across navigation. */
	key?: string
) {
	const kept = key ? KEPT[key] : undefined;
	let sorting = $state<SortingState>(kept?.sorting ?? initialSorting);
	let globalFilter = $state(kept?.globalFilter ?? '');

	function remember() {
		if (key) KEPT[key] = { sorting, globalFilter };
	}

	const common = {
		// TanStack types the resolved option as `ColumnDef<TData, any>[]`, which nothing outside
		// the library can satisfy without an `any` of its own.
		columns: columns as TableOptionsResolved<TData>['columns'],
		onStateChange: () => {},
		renderFallbackValue: null,
		getCoreRowModel: getCoreRowModel<TData>(),
		getSortedRowModel: getSortedRowModel<TData>(),
		getFilteredRowModel: getFilteredRowModel<TData>()
	};

	// A table whose `state` carries only the keys we drive reads `undefined` for every other
	// feature — column pinning first, which the core row model dereferences on the first render.
	// Seeding from a throwaway instance keeps the full shape without duplicating TanStack's
	// defaults here.
	const defaults: Partial<TableState> = createTable<TData>({
		...common,
		data: [],
		state: {}
	}).initialState;

	const table = $derived.by<Table<TData>>(() =>
		createTable<TData>({
			...common,
			data: getRows(),
			state: { ...defaults, sorting, globalFilter },
			onSortingChange: (updater: Updater<SortingState>) => {
				sorting = typeof updater === 'function' ? updater(sorting) : updater;
				remember();
			},
			onGlobalFilterChange: (updater: Updater<string>) => {
				globalFilter = typeof updater === 'function' ? updater(globalFilter) : updater;
				remember();
			}
		})
	);

	return {
		get table() {
			return table;
		},
		get globalFilter() {
			return globalFilter;
		},
		set globalFilter(value: string) {
			globalFilter = value;
			remember();
		}
	};
}
