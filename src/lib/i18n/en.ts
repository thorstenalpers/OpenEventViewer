export const en = {
	sidebar: {
		tagline: 'Windows event logs',
		events: 'Events',
		diagnose: 'Diagnose',
		log: 'Log',
		settings: 'Settings',
		info: 'Info',
		toLight: 'Switch to light theme',
		toDark: 'Switch to dark theme',
		sections: 'Sections',
		collapse: 'Collapse the sidebar',
		expand: 'Expand the sidebar'
	},
	common: {
		loading: 'Loading…',
		mockHost: 'Mock host — no Tauri backend. Data on this page is fixture data.'
	},
	events: {
		title: 'Events',
		subtitle: 'What Windows recorded, newest first.',
		channel: 'Channel',
		allChannels: 'System and Application',
		from: 'From',
		to: 'To',
		load: 'Load',
		keyword: 'Search every column…',
		columnFilter: 'column filter',
		clearColumnFilters: 'Clear column filters',
		loaded: (shown: number, total: number) =>
			shown === total ? `${total} events` : `${shown} of ${total} events`,
		elapsed: (ms: number) => `read in ${ms} ms`,
		truncated: 'more than the row limit — narrow the filter or raise it in Settings',
		securityHint:
			'Close OpenEventViewer and start it again as administrator, or pick a channel that does not need it.',
		empty: 'Nothing matches.',
		search: 'Search the web for this event',
		resize: (column: string) => `Resize the ${column} column`,
		filters: {
			search: 'Search…',
			noMatch: 'Nothing matches.',
			clear: 'Clear this filter',
			chosen: (count: number) => `${count} chosen`,
			after: (time: string) => `after ${time}`,
			before: (time: string) => `before ${time}`,
			timeHint: 'Local time, the same clock the table shows.',
			numberHint: 'Nothing in there was a number.',
			notUnderstood: (parts: string) => `Not understood: ${parts}`,
			helpAny: 'any of them',
			helpCompare: 'above, below',
			helpRange: 'a range, ends included',
			helpNot: 'everything but'
		},
		overTime: 'Over time',
		andMore: (kinds: number, count: number) =>
			`${kinds} more kind${kinds === 1 ? '' : 's'}, ${count} in total`,
		bucketSize: (minutes: number) =>
			minutes >= 1440
				? `one bar per ${minutes / 1440} day${minutes === 1440 ? '' : 's'}`
				: minutes >= 60
					? `one bar per ${minutes / 60} hour${minutes === 60 ? '' : 's'}`
					: `one bar per ${minutes} minute${minutes === 1 ? '' : 's'}`,
		bucketCount: (total: number, errors: number, warnings: number) => {
			const events = `${total} event${total === 1 ? '' : 's'}`;
			const parts = [
				errors > 0 && `${errors} ${errors === 1 ? 'error' : 'errors'}`,
				warnings > 0 && `${warnings} ${warnings === 1 ? 'warning' : 'warnings'}`
			].filter(Boolean);
			return parts.length === 0 ? events : `${events}, of them ${parts.join(' and ')}`;
		},
		columns: {
			level: 'Level',
			time: 'Time',
			provider: 'Provider',
			eventId: 'ID',
			task: 'Task',
			channel: 'Channel',
			computer: 'Computer',
			message: 'Message'
		}
	},
	diagnose: {
		title: 'Diagnose',
		subtitle:
			'Scans the log for the events a machine writes when something went wrong, then pulls the quarter of an hour around one of them.',
		days: (count: number) => (count === 1 ? 'Last day' : `Last ${count} days`),
		scan: 'Scan',
		scanning: 'Scanning…',
		intro:
			'Nothing has been scanned yet. Pick a stretch above and press Scan; every find — a crash, a freeze, a disk error, a throttled processor — appears here as an incident you can open.',
		pick: 'Open an incident to see everything the machine wrote in the quarter of an hour around it.',
		nothing: 'Nothing found. Scan a longer stretch, or take it as good news.',
		window: (from: string, to: string) => `${from} — ${to}`,
		inWindow: (count: number) => `${count} event${count === 1 ? '' : 's'} in the window`,
		kinds: {
			unexpectedShutdown: 'Unexpected shutdown',
			bugCheck: 'Bug check',
			hardwareError: 'Hardware error',
			appHang: 'Application hang',
			appCrash: 'Application crash',
			serviceFailure: 'Service failure',
			diskError: 'Disk error',
			ntfs: 'File system',
			displayTdr: 'Display driver reset',
			processorPower: 'Processor throttled'
		}
	},
	log: {
		title: 'Log',
		subtitle: 'What the app did, newest last. Nothing here is written to disk.',
		filter: 'Filter messages…',
		level: 'Level',
		levels: {
			all: 'All levels',
			error: 'Errors',
			warning: 'Warnings',
			info: 'Info',
			debug: 'Debug'
		},
		clear: 'Clear the log',
		empty: 'Nothing logged yet.',
		count: (shown: number, total: number) => `${shown} of ${total} entries`
	},
	info: {
		title: 'Info',
		subtitle: 'What this app is, and what it is built on.',
		appBody:
			'Read the Windows event logs and filter them down to what matters — no account, no upload, no telemetry.',
		offline: 'Everything runs on this machine. Nothing is uploaded, and no telemetry is collected.',
		appLicense: 'OpenEventViewer is MIT licensed.',
		thirdParty: 'Third-party components',
		thirdPartyBody: (total: number, vendored: number, crates: number, npm: number) =>
			`${total} components ship with this app: ${vendored} bundled binaries, ${crates} Rust crates, ${npm} npm packages.`,
		shipped:
			'The full licence texts ship inside the installer as THIRD_PARTY_LICENSES.txt. MIT, BSD and ISC all require the notice to accompany the binary, so a link would not be enough.',
		filter: 'Filter components…',
		showTexts: 'Show licence texts',
		hideTexts: 'Hide licence texts',
		noMatch: 'No component matches.',
		redistributed: 'shipped as a binary',
		noOwnText: 'no own text',
		withoutText: (count: number) =>
			`${count} components published no licence file of their own; the canonical text of the licence named applies.`,
		material: 'Your logs',
		materialBody:
			'The event logs stay where Windows keeps them. This app reads them and never writes to them.'
	},
	detail: {
		general: 'General',
		data: 'Event data',
		xml: 'XML',
		search: 'Search the web',
		copy: 'Copy',
		copied: 'Copied',
		close: 'Close the detail pane',
		recordId: 'Record',
		keywords: 'Keywords',
		noData: 'This event carries no data of its own.'
	},
	updater: {
		title: 'Updates',
		body: (version: string) => `Version ${version}. Checked once at start.`,
		check: 'Check now',
		checking: 'Checking…',
		upToDate: 'up to date',
		available: (version: string) => `${version} is available`,
		downloading: (percent: number | null) =>
			percent === null ? 'Downloading…' : `Downloading — ${percent}%`,
		ready: 'Installed — restarting',
		install: 'Install and restart',
		failed: 'The update check failed.'
	},
	settings: {
		title: 'Settings',
		appearance: 'Appearance',
		appearanceBody: 'Theme of the app window.',
		system: 'System',
		light: 'Light',
		dark: 'Dark',
		colours: 'Colours',
		coloursBody: 'The palette every view is drawn from.',
		presets: {
			default: 'Default',
			caffeine: 'Caffeine',
			catppuccin: 'Catppuccin',
			claude: 'Claude',
			'modern-minimal': 'Modern Minimal',
			mono: 'Mono',
			'northern-lights': 'Northern Lights',
			supabase: 'Supabase',
			tangerine: 'Tangerine',
			twitter: 'Twitter',
			vercel: 'Vercel'
		} as Record<string, string>,
		language: 'Language',
		languageBody: 'The app interface. Event text keeps the language Windows recorded it in.',
		eventsRows: 'Events: rows to load',
		eventsRowsBody:
			'Every event costs the publisher a message lookup, so a bigger number is a longer wait rather than a longer list.',
		eventsRowsValue: (rows: number) => `${rows.toLocaleString('en')} rows`,
		showLogs: 'Show the log in the sidebar',
		showLogsBody: 'Adds a Log entry to the navigation.',
		debugLogging: 'Record debug entries',
		debugLoggingBody:
			'Verbose. Off by default, because debug entries crowd out the ones you went looking for.'
	}
};

export type Translations = typeof en;
