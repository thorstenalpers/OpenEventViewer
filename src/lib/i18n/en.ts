export const en = {
	sidebar: {
		tagline: 'Windows event logs',
		events: 'Events',
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
		subtitle: 'What Windows recorded, newest first.'
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
		refresh: 'Refresh',
		includeWeb: 'Include web console',
		includeWebBody:
			"Copies the webview's own console into this log, so interface and host share one timeline.",
		clear: 'Clear the log',
		empty: 'Nothing logged yet.',
		count: (shown: number, total: number) => `${shown} of ${total} entries`
	},
	info: {
		title: 'Info',
		subtitle: 'What this app is, and what it is built on.',
		appBody:
			'Read the Windows event logs, filter them down to what matters, and ask an assistant what a run of them means.',
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
	assistant: {
		title: 'Assistant',
		thinking: 'Thinking…',
		sourceCli: 'local claude',
		sourceAnthropic: 'anthropic',
		noCli:
			'The local claude binary is not on PATH. Install Claude Code, or pick a hosted provider in Settings.',
		noKey: 'No API key stored. Add one in Settings.'
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
			'modern-minimal': 'Modern Minimal',
			mono: 'Mono',
			'northern-lights': 'Northern Lights',
			vercel: 'Vercel'
		} as Record<string, string>,
		language: 'Language',
		languageBody: 'The app interface. Event text keeps the language Windows recorded it in.',
		showLogs: 'Show the log in the sidebar',
		showLogsBody: 'Adds a Log entry to the navigation.',
		debugLogging: 'Record debug entries',
		debugLoggingBody:
			'Verbose. Off by default, because debug entries crowd out the ones you went looking for.',
		assistant: 'Assistant',
		assistantBody: 'Where the assistant sends what you ask it.',
		sourceCliLabel: 'Local claude binary',
		sourceCliDetail: 'Runs on this machine. This app sends nothing to a third party.',
		sourceAnthropicLabel: 'Anthropic API',
		sourceAnthropicDetail: 'What the preview shows goes to api.anthropic.com when you press Send.',
		found: 'found',
		keyStored: 'key stored',
		apiKey: 'API key',
		store: 'Store',
		keyNote:
			'The key goes into the Windows Credential Manager, never into a file this app owns, and it cannot be read back into this window.',
		stored: 'Stored in the Windows Credential Manager.'
	}
};

export type Translations = typeof en;
