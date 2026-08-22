import type { CommandArgs, CommandName, LogEntry, Settings } from './contract';

/**
 * The host stand-in for `npm run dev`, so every view can be built and tested in a browser without
 * Tauri or WebView2. It is deliberately thin: enough state to exercise the flows, no attempt to
 * mirror the real event log or the real assistant.
 */

let settings: Settings = { theme: 'system', showLogs: true, debugLogging: false };

const logEntries: LogEntry[] = [
	{
		timestamp: new Date().toISOString(),
		level: 'info',
		source: 'host',
		message: 'Mock host ready.'
	}
];

let storedKey = false;

export function mockHost<T extends CommandName>(name: T, args: CommandArgs[T]): unknown {
	switch (name) {
		case 'get_settings':
			return settings;

		case 'set_settings':
			settings = (args as CommandArgs['set_settings']).settings;
			return settings;

		case 'log_entries':
			return [...logEntries];

		case 'log_clear':
			logEntries.length = 0;
			return null;

		case 'log_write': {
			const { level, source, message } = args as CommandArgs['log_write'];
			logEntries.push({ timestamp: new Date().toISOString(), level, source, message });
			return null;
		}

		case 'third_party_licenses':
			return 'Third-party licence texts ship with the installer, not with the mock host.';

		case 'devtools_open':
			return null;

		case 'assistant_status': {
			const { source } = args as CommandArgs['assistant_status'];
			return { source, cliAvailable: true, hasKey: storedKey };
		}

		case 'assistant_set_key':
			storedKey = (args as CommandArgs['assistant_set_key']).key.trim().length > 0;
			return null;

		default:
			throw new Error(`mock host has no command ${name}`);
	}
}
