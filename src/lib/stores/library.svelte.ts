import { call } from '$lib/bridge/client';
import type { Binder } from '$lib/bridge/contract';

/**
 * The binder list and the selection that Review and Train work against. A class rather than a plain
 * object because it owns the load state alongside the data.
 */
class LibraryStore {
	binders = $state<Binder[]>([]);
	selectedId = $state<number | null>(null);
	loading = $state(false);
	error = $state<string | null>(null);

	get selected(): Binder | null {
		return this.binders.find((b) => b.id === this.selectedId) ?? null;
	}

	async refresh(): Promise<void> {
		this.loading = true;
		this.error = null;
		try {
			this.binders = await call('list_binders', {});
			if (this.selectedId === null || !this.binders.some((b) => b.id === this.selectedId)) {
				this.selectedId = this.binders[0]?.id ?? null;
			}
		} catch (error) {
			this.error = error instanceof Error ? error.message : String(error);
		} finally {
			this.loading = false;
		}
	}

	async remove(binderId: number): Promise<void> {
		await call('delete_binder', { binderId });
		await this.refresh();
	}
}

export const library = new LibraryStore();
