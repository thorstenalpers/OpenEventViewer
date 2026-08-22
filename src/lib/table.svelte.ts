import {
	createTable,
	getCoreRowModel,
	getFilteredRowModel,
	getSortedRowModel,
	type ColumnDef,
	type ColumnFiltersState,
	type ColumnSizingState,
	type RowData,
	type SortingState,
	type Table,
	type TableOptionsResolved,
	type TableState,
	type Updater
} from '@tanstack/table-core';

interface Kept {
	sorting: SortingState;
	globalFilter: string;
	columnFilters: ColumnFiltersState;
	columnSizing: ColumnSizingState;
}

const EMPTY: Kept = { sorting: [], globalFilter: '', columnFilters: [], columnSizing: {} };

/**
 * Sort, filters and column widths that outlive the view — and the run.
 *
 * Routing unmounts a page, so a table's state would otherwise reset on every trip to Settings and
 * back. It is written to `localStorage` as well, because a column dragged to the width its content
 * needs is work, and work should not have to be redone after every restart. Keyed by table so two
 * tables never share a filter.
 */
const KEPT: Record<string, Kept> = {};

function storageKey(key: string): string {
	return `oev.table.${key}`;
}

function restore(key: string): Kept {
	const held = KEPT[key];
	if (held) return held;
	try {
		const raw = localStorage?.getItem(storageKey(key));
		// Shallow-merged onto the defaults: a state written by an older build is missing whatever
		// was added since, and the table must not read `undefined` for it.
		const kept = raw ? { ...EMPTY, ...(JSON.parse(raw) as Partial<Kept>) } : { ...EMPTY };
		KEPT[key] = kept;
		return kept;
	} catch {
		// Unreadable or unparseable is the same answer: start from the defaults.
		return { ...EMPTY };
	}
}

/** What a view is handed: the table itself plus the two filters it does not drive through columns. */
export interface DataTable<TData extends RowData> {
	readonly table: Table<TData>;
	globalFilter: string;
	readonly columnFilters: ColumnFiltersState;
	clearColumnFilters(): void;
	/** Back to the widths the column definitions ask for. */
	resetSizing(): void;
}

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
	/** Set to keep this table's sort, filters and widths across navigation and across runs. */
	key?: string
): DataTable<TData> {
	const kept = key ? restore(key) : undefined;
	let sorting = $state<SortingState>(kept?.sorting.length ? kept.sorting : initialSorting);
	let globalFilter = $state(kept?.globalFilter ?? '');
	let columnFilters = $state<ColumnFiltersState>(kept?.columnFilters ?? []);
	let columnSizing = $state<ColumnSizingState>(kept?.columnSizing ?? {});

	function remember() {
		if (!key) return;
		const state: Kept = { sorting, globalFilter, columnFilters, columnSizing };
		KEPT[key] = state;
		try {
			localStorage?.setItem(storageKey(key), JSON.stringify(state));
		} catch {
			// A full or unavailable store costs the memory of a width, not the table.
		}
	}

	const common = {
		// TanStack types the resolved option as `ColumnDef<TData, any>[]`, which nothing outside
		// the library can satisfy without an `any` of its own.
		columns: columns as TableOptionsResolved<TData>['columns'],
		onStateChange: () => {},
		renderFallbackValue: null,
		enableColumnResizing: true,
		// The width follows the pointer rather than landing when it is released: a drag you cannot
		// see the result of is a drag you do twice.
		columnResizeMode: 'onChange' as const,
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

	// Not kept: it lives for the length of one drag, and rebuilding the instance on every mouse
	// move is what a `$state` here would cost.
	let sizingInfo = $state(defaults.columnSizingInfo);

	const table = $derived.by<Table<TData>>(() =>
		createTable<TData>({
			...common,
			data: getRows(),
			state: {
				...defaults,
				sorting,
				globalFilter,
				columnFilters,
				columnSizing,
				columnSizingInfo: sizingInfo
			},
			onSortingChange: (updater: Updater<SortingState>) => {
				sorting = typeof updater === 'function' ? updater(sorting) : updater;
				remember();
			},
			onGlobalFilterChange: (updater: Updater<string>) => {
				globalFilter = typeof updater === 'function' ? updater(globalFilter) : updater;
				remember();
			},
			onColumnFiltersChange: (updater: Updater<ColumnFiltersState>) => {
				columnFilters = typeof updater === 'function' ? updater(columnFilters) : updater;
				remember();
			},
			onColumnSizingChange: (updater: Updater<ColumnSizingState>) => {
				columnSizing = typeof updater === 'function' ? updater(columnSizing) : updater;
				remember();
			},
			onColumnSizingInfoChange: (updater) => {
				sizingInfo = typeof updater === 'function' ? updater(sizingInfo!) : updater;
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
		},
		get columnFilters() {
			return columnFilters;
		},
		clearColumnFilters() {
			columnFilters = [];
			remember();
		},
		resetSizing() {
			columnSizing = {};
			remember();
		}
	};
}
