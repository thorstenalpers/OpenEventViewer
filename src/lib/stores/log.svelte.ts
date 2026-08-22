import { call } from '$lib/bridge/client';
import type { LogEntry, LogLevel } from '$lib/bridge/contract';

/**
 * The host's log buffer, mirrored for display.
 *
 * There is no push channel yet, so this pulls: on opening the view, and on demand. That is honest
 * for a diagnostic — a log you have to refresh is still a log, whereas a push channel built only
 * for this would be infrastructure nobody else uses.
 */
class LogStore {
	entries = $state<LogEntry[]>([]);
	loading = $state(false);
	error = $state<string | null>(null);

	// The filters live here rather than in the view: routing unmounts the view, and a trip to
	// Settings and back would otherwise silently reset what the reader was looking at.
	messageFilter = $state('');
	levelFilter = $state<LogLevel | 'all'>('all');

	async refresh(): Promise<void> {
		this.loading = true;
		this.error = null;
		try {
			this.entries = await call('log_entries', {});
		} catch (error) {
			this.error = error instanceof Error ? error.message : String(error);
		} finally {
			this.loading = false;
		}
	}

	async clear(): Promise<void> {
		await call('log_clear', {});
		this.entries = [];
	}

	/** Writes into the host's buffer, so interface and host share one timeline. */
	async write(level: LogLevel, source: string, message: string): Promise<void> {
		try {
			await call('log_write', { level, source, message });
		} catch {
			// Logging must never be the thing that breaks. If the host will not take the entry,
			// the console still has it.
			console.error(`[${source}] ${message}`);
		}
	}
}

export const log = new LogStore();

/**
 * Sends anything the interface throws to the host's log.
 *
 * This exists because of a bug that could not be reproduced in a browser: buttons in a view did
 * nothing, with no visible error. An exception thrown inside a Svelte effect leaves the component
 * half-initialised and its handlers inert, and without this it is invisible unless someone has
 * devtools open — which nobody does in a packaged app.
 */
export function captureErrors(): void {
	if (typeof window === 'undefined') return;

	window.addEventListener('error', (event) => {
		void log.write(
			'error',
			'ui',
			`${event.message} (${event.filename ?? '?'}:${event.lineno ?? 0})`
		);
	});

	window.addEventListener('unhandledrejection', (event: PromiseRejectionEvent) => {
		const reason: unknown = event.reason;
		void log.write(
			'error',
			'ui',
			reason instanceof Error ? `${reason.message}\n${reason.stack ?? ''}` : String(reason)
		);
	});

	captureConsole();
}

/**
 * Copies the webview's console into the host log, so one timeline holds both sides.
 *
 * The originals are always called: this observes the console, it does not replace it. A guard stops
 * the recursion that would otherwise follow from `log.write` falling back to `console.error` when
 * the host refuses an entry — one failed write would then log itself forever.
 */
function captureConsole(): void {
	// Errors only. A framework reporting a broken hydration through `console.error` is the one line
	// that explains a window nobody can click; everything else the console says is chatter that
	// would evict it from a buffer of two thousand.
	const original = console.error.bind(console);
	let forwarding = false;

	console.error = (...args: unknown[]) => {
		original(...args);
		if (forwarding) return;
		forwarding = true;
		try {
			void log.write('error', 'web', args.map(describe).join(' '));
		} finally {
			forwarding = false;
		}
	};
}

function describe(value: unknown): string {
	if (typeof value === 'string') return value;
	if (value instanceof Error) return `${value.message}\n${value.stack ?? ''}`;
	try {
		return JSON.stringify(value);
	} catch {
		return String(value);
	}
}
