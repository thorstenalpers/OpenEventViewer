import { call } from '$lib/bridge/client';
import type { Bundle, Incident } from '$lib/bridge/contract';

export const DAY_CHOICES = [1, 7, 30] as const;

/**
 * The guided walk: scan a stretch of log for incidents, then pull the events around one of them.
 *
 * A singleton for the same reason the events store is one — the bundle costs a second query and
 * should survive a trip to the assistant and back.
 */
class DiagnoseStore {
	days = $state<number>(7);
	incidents = $state<Incident[]>([]);
	selectedId = $state<string | null>(null);
	bundle = $state<Bundle | null>(null);
	scanning = $state(false);
	opening = $state(false);
	error = $state<string | null>(null);

	get scanned(): boolean {
		return this.incidents.length > 0 || (!this.scanning && this.error === null && this.touched);
	}

	private touched = $state(false);

	async load(): Promise<void> {
		this.scanning = true;
		this.error = null;
		this.bundle = null;
		this.selectedId = null;
		try {
			this.incidents = await call('diagnose_incidents', { days: this.days });
			this.touched = true;
		} catch (error) {
			this.error = error instanceof Error ? error.message : String(error);
			this.incidents = [];
		} finally {
			this.scanning = false;
		}
	}

	async open(incident: Incident): Promise<void> {
		if (this.selectedId === incident.id) {
			this.selectedId = null;
			this.bundle = null;
			return;
		}
		this.selectedId = incident.id;
		this.bundle = null;
		this.opening = true;
		this.error = null;
		try {
			this.bundle = await call('diagnose_bundle', {
				channel: incident.event.channel,
				recordId: incident.event.recordId
			});
		} catch (error) {
			this.error = error instanceof Error ? error.message : String(error);
		} finally {
			this.opening = false;
		}
	}
}

export const diagnose = new DiagnoseStore();
