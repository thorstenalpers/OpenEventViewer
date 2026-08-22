import { isMockHost } from '$lib/bridge/client';

export type UpdaterState =
	| { kind: 'idle' }
	| { kind: 'checking' }
	| { kind: 'upToDate' }
	| { kind: 'available'; version: string }
	| { kind: 'downloading'; version: string; percent: number | null }
	| { kind: 'ready'; version: string }
	| { kind: 'error'; message: string };

/**
 * The signed auto-updater.
 *
 * Minisign, not Authenticode: the `.sig` beside the installer is what makes an update safe to take,
 * and it costs nothing. SmartScreen still warns on the first install — that is a different problem
 * and a paid one.
 */
class UpdaterStore {
	state = $state<UpdaterState>({ kind: 'idle' });

	get busy(): boolean {
		return this.state.kind === 'checking' || this.state.kind === 'downloading';
	}

	/**
	 * Called once at start and from the Settings row.
	 *
	 * `silent` is the start-up case: no release published yet means the endpoint answers with
	 * nothing, and a first run must not open with an error about an update that was never there.
	 */
	async check(silent = false): Promise<void> {
		if (this.busy) return;
		// Nothing to update in a browser, and the plugin is not there to say so.
		if (isMockHost()) {
			this.state = silent ? { kind: 'idle' } : { kind: 'upToDate' };
			return;
		}

		this.state = { kind: 'checking' };
		try {
			const { check } = await import('@tauri-apps/plugin-updater');
			const update = await check();
			this.state = update ? { kind: 'available', version: update.version } : { kind: 'upToDate' };
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			this.state = silent ? { kind: 'idle' } : { kind: 'error', message };
		}
	}

	async install(): Promise<void> {
		if (this.state.kind !== 'available') return;
		const version = this.state.version;
		this.state = { kind: 'downloading', version, percent: null };

		try {
			const { check } = await import('@tauri-apps/plugin-updater');
			const update = await check();
			if (!update) {
				this.state = { kind: 'upToDate' };
				return;
			}

			let total: number | null = null;
			let received = 0;
			await update.downloadAndInstall((progress) => {
				if (progress.event === 'Started') {
					total = progress.data.contentLength ?? null;
				} else if (progress.event === 'Progress') {
					received += progress.data.chunkLength;
					this.state = {
						kind: 'downloading',
						version,
						percent: total ? Math.round((received / total) * 100) : null
					};
				}
			});

			this.state = { kind: 'ready', version };
			const { relaunch } = await import('@tauri-apps/plugin-process');
			await relaunch();
		} catch (error) {
			this.state = {
				kind: 'error',
				message: error instanceof Error ? error.message : String(error)
			};
		}
	}
}

export const updater = new UpdaterStore();
