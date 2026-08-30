import type {
	Bundle,
	CommandArgs,
	CommandName,
	EventFilter,
	EventRecord,
	Incident,
	IncidentKind,
	LogEntry,
	Settings
} from './contract';

/**
 * The host stand-in for `npm run dev`, so every view can be built and tested in a browser without
 * Tauri, WebView2 or a Windows event log. It is deliberately thin: enough events to exercise the
 * table, the filters and the diagnosis, no attempt to mirror what a real machine records.
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

const CHANNELS = [
	'Application',
	'HardwareEvents',
	'Microsoft-Windows-Diagnostics-Performance/Operational',
	'Microsoft-Windows-Kernel-Power/Thermal-Operational',
	'Security',
	'Setup',
	'System',
	'Windows PowerShell'
];

/** What a normal account is told when it asks for the Security channel. */
const ACCESS_DENIED =
	'reading Security needs administrator rights — the Security channel is readable only by an ' +
	'elevated process. Start OpenEventViewer as administrator to read it.';

interface Shape {
	channel: string;
	provider: string;
	eventId: number;
	level: number;
	task: string;
	message: string;
	data: [string, string][];
	/** Roughly how many of the 8 000 events are this one. */
	weight: number;
}

// Ordinary traffic, plus the signatures the Diagnose page goes looking for. The rare ones carry a
// weight of 1 so a week of fixture data holds a handful rather than a wall of them.
const SHAPES: [Shape, ...Shape[]] = [
	{
		channel: 'System',
		provider: 'Service Control Manager',
		eventId: 7036,
		level: 4,
		task: 'None',
		message: 'The Windows Update service entered the running state.',
		data: [['param1', 'Windows Update']],
		weight: 240
	},
	{
		channel: 'System',
		provider: 'Microsoft-Windows-Kernel-General',
		eventId: 16,
		level: 4,
		task: 'None',
		message: 'The access history in hive \\SystemRoot\\System32\\config\\SOFTWARE was cleared.',
		data: [['HiveName', '\\SystemRoot\\System32\\config\\SOFTWARE']],
		weight: 120
	},
	{
		channel: 'Application',
		provider: 'ESENT',
		eventId: 326,
		level: 4,
		task: 'General',
		message: 'svchost (2764) The database engine attached a database.',
		data: [['Process', 'svchost']],
		weight: 140
	},
	{
		channel: 'Application',
		provider: 'Windows Error Reporting',
		eventId: 1001,
		level: 4,
		task: 'None',
		message: 'Fault bucket 1234567890, type 5. Event Name: APPCRASH.',
		data: [['Bucket', '1234567890']],
		weight: 40
	},
	{
		channel: 'System',
		provider: 'DCOM',
		eventId: 10016,
		level: 3,
		task: 'None',
		message:
			'The application-specific permission settings do not grant Local Activation permission ' +
			'for the COM Server application with CLSID {2593F8B9}.',
		data: [['CLSID', '{2593F8B9}']],
		weight: 60
	},
	{
		channel: 'System',
		provider: 'Microsoft-Windows-DNS-Client',
		eventId: 1014,
		level: 3,
		task: 'None',
		message: 'Name resolution for the name wpad timed out after none of the servers responded.',
		data: [['QueryName', 'wpad']],
		weight: 45
	},
	{
		channel: 'System',
		provider: 'Service Control Manager',
		eventId: 7031,
		level: 2,
		task: 'None',
		message:
			'The Print Spooler service terminated unexpectedly. It has done this 1 time(s). ' +
			'The following corrective action will be taken in 60000 milliseconds: Restart the service.',
		data: [
			['param1', 'Print Spooler'],
			['param2', '1']
		],
		weight: 6
	},
	{
		channel: 'Application',
		provider: 'Application Error',
		eventId: 1000,
		level: 2,
		task: 'Application Crashing Events',
		message:
			'Faulting application name: contoso.exe, version 3.1.0.0, faulting module name: ntdll.dll.',
		data: [
			['AppName', 'contoso.exe'],
			['ModuleName', 'ntdll.dll'],
			['ExceptionCode', '0xc0000005']
		],
		weight: 5
	},
	{
		channel: 'Application',
		provider: 'Application Hang',
		eventId: 1002,
		level: 2,
		task: 'Application Hanging Events',
		message: 'The program contoso.exe version 3.1.0.0 stopped interacting with Windows.',
		data: [
			['AppName', 'contoso.exe'],
			['HangType', 'Top level window is idle']
		],
		weight: 4
	},
	{
		channel: 'System',
		provider: 'disk',
		eventId: 153,
		level: 3,
		task: 'None',
		message: 'The IO operation at logical block address 0x1f4a20 for Disk 0 was retried.',
		data: [['Disk', '0']],
		weight: 4
	},
	{
		channel: 'System',
		provider: 'Microsoft-Windows-Kernel-Power',
		eventId: 41,
		level: 1,
		task: 'None',
		message:
			'The system has rebooted without cleanly shutting down first. This error could be caused ' +
			'if the system stopped responding, crashed, or lost power unexpectedly.',
		data: [
			['BugcheckCode', '0'],
			['PowerButtonTimestamp', '0']
		],
		weight: 2
	},
	{
		channel: 'System',
		provider: 'EventLog',
		eventId: 6008,
		level: 2,
		task: 'None',
		message: 'The previous system shutdown at 03:14:07 was unexpected.',
		data: [
			['Data1', '03:14:07'],
			['Data2', '18.08.2026']
		],
		weight: 2
	},
	{
		channel: 'System',
		provider: 'Microsoft-Windows-WHEA-Logger',
		eventId: 18,
		level: 2,
		task: 'None',
		message: 'A fatal hardware error has occurred. Reported by component: Processor Core.',
		data: [['ErrorSource', 'Machine Check Exception']],
		weight: 1
	},
	{
		channel: 'System',
		provider: 'Display',
		eventId: 4101,
		level: 3,
		task: 'None',
		message: 'Display driver nvlddmkm stopped responding and has successfully recovered.',
		data: [['Driver', 'nvlddmkm']],
		weight: 2
	},
	{
		channel: 'Setup',
		provider: 'Microsoft-Windows-WUSA',
		eventId: 2,
		level: 4,
		task: 'None',
		message: 'Windows update "Security Update for Windows (KB5031234)" was installed.',
		data: [['UpdateTitle', 'KB5031234']],
		weight: 20
	}
];

const LEVEL_NAMES: Record<number, string> = {
	0: 'Information',
	1: 'Critical',
	2: 'Error',
	3: 'Warning',
	4: 'Information',
	5: 'Verbose'
};

const TOTAL = 8000;
const WINDOW_MS = 7 * 24 * 60 * 60 * 1000;

/**
 * Deterministic, so two runs of the same test look at the same log.
 *
 * A seeded generator rather than a captured export: an export of a real machine's log is somebody's
 * private history, and this only has to be shaped like one.
 */
function mulberry(seed: number): () => number {
	let state = seed >>> 0;
	return () => {
		state = (state + 0x6d2b79f5) >>> 0;
		let value = Math.imul(state ^ (state >>> 15), 1 | state);
		value = (value + Math.imul(value ^ (value >>> 7), 61 | value)) ^ value;
		return ((value ^ (value >>> 14)) >>> 0) / 4294967296;
	};
}

function generate(): EventRecord[] {
	const random = mulberry(20260822);
	const pool: Shape[] = SHAPES.flatMap((shape) =>
		Array.from({ length: shape.weight }, () => shape)
	);
	// Anchored to the current hour rather than to a date written into the file: the toolbar opens on
	// "last 24 hours", and a fixture pinned to the day it was written shows an empty table for ever
	// after. Rounding to the hour keeps two runs in the same hour identical.
	const newest = Math.floor(Date.now() / 3_600_000) * 3_600_000;
	const events: EventRecord[] = [];

	for (let index = 0; index < TOTAL; index += 1) {
		const shape = pool[Math.floor(random() * pool.length)] ?? SHAPES[0];
		const at = newest - Math.floor(random() * WINDOW_MS);
		events.push({
			recordId: 100000 - index,
			channel: shape.channel,
			provider: shape.provider,
			eventId: shape.eventId,
			level: shape.level,
			levelName: LEVEL_NAMES[shape.level] ?? 'Information',
			task: shape.task,
			keywords: shape.level === 4 ? [] : ['Classic'],
			timeCreated: new Date(at).toISOString().replace(/(\.\d{3})\d*Z$/, '$1Z'),
			computer: 'WORKBENCH',
			message: shape.message,
			eventData: shape.data.map(([name, value]) => ({ name, value }))
		});
	}

	return events.sort((left, right) => right.timeCreated.localeCompare(left.timeCreated));
}

let allEvents: EventRecord[] | null = null;

function events(): EventRecord[] {
	allEvents ??= generate();
	return allEvents;
}

/** What wevtapi accepts in one query. Over it, the host refuses rather than answering with none. */
const MAX_EXPRESSIONS = 20;

function expressionCount(filter: EventFilter): number {
	const levels = filter.levels.length
		? filter.levels.length + (filter.levels.includes(4) && !filter.levels.includes(0) ? 1 : 0)
		: 0;
	return (
		levels +
		(filter.from ? 1 : 0) +
		(filter.to ? 1 : 0) +
		filter.eventIds.length +
		filter.providers.length
	);
}

/** The same narrowing the host's XPath does, so a filter behaves the same in both. */
function matches(event: EventRecord, filter: EventFilter): boolean {
	const channels = filter.channels.length ? filter.channels : ['System', 'Application'];
	if (!channels.includes(event.channel)) return false;
	// Level 0 is "log always", which the host folds into Information.
	if (filter.levels.length) {
		const wanted = filter.levels.includes(4) ? [...filter.levels, 0] : filter.levels;
		if (!wanted.includes(event.level)) return false;
	}
	if (filter.from && event.timeCreated < filter.from) return false;
	if (filter.to && event.timeCreated > filter.to) return false;
	if (filter.eventIds.length && !filter.eventIds.includes(event.eventId)) return false;
	if (filter.providers.length && !filter.providers.includes(event.provider)) return false;
	return true;
}

function xmlOf(event: EventRecord): string {
	const data = event.eventData
		.map((item) => `    <Data Name="${item.name}">${item.value}</Data>`)
		.join('\n');
	return `<Event xmlns="http://schemas.microsoft.com/win/2004/08/events/event">
  <System>
    <Provider Name="${event.provider}" />
    <EventID>${event.eventId}</EventID>
    <Level>${event.level}</Level>
    <TimeCreated SystemTime="${event.timeCreated}" />
    <EventRecordID>${event.recordId}</EventRecordID>
    <Channel>${event.channel}</Channel>
    <Computer>${event.computer}</Computer>
  </System>
  <EventData>
${data}
  </EventData>
</Event>`;
}

/** The same signatures the host carries, so the fixture finds what a real machine would. */
const SIGNATURES: { provider: string; ids: number[]; kind: IncidentKind }[] = [
	{ provider: 'Microsoft-Windows-Kernel-Power', ids: [41, 137], kind: 'unexpectedShutdown' },
	{ provider: 'EventLog', ids: [6008], kind: 'unexpectedShutdown' },
	{ provider: 'BugCheck', ids: [1001], kind: 'bugCheck' },
	{ provider: 'Microsoft-Windows-WHEA-Logger', ids: [17, 18, 19, 47], kind: 'hardwareError' },
	{ provider: 'Application Hang', ids: [1002], kind: 'appHang' },
	{ provider: 'Application Error', ids: [1000], kind: 'appCrash' },
	{
		provider: 'Service Control Manager',
		ids: [7000, 7001, 7011, 7031, 7034],
		kind: 'serviceFailure'
	},
	{ provider: 'disk', ids: [7, 11, 51, 153], kind: 'diskError' },
	{ provider: 'Ntfs', ids: [55, 98, 140], kind: 'ntfs' },
	{ provider: 'Display', ids: [4101], kind: 'displayTdr' },
	{ provider: 'nvlddmkm', ids: [13, 14], kind: 'displayTdr' },
	{ provider: 'Microsoft-Windows-Kernel-Processor-Power', ids: [37], kind: 'processorPower' }
];

const NOISE = ['DCOM', 'Microsoft-Windows-DistributedCOM'];
const BUNDLE_BEFORE_MS = 15 * 60 * 1000;
const BUNDLE_AFTER_MS = 2 * 60 * 1000;
const COLLAPSE_MS = 60 * 1000;

function classify(event: EventRecord): IncidentKind | null {
	return (
		SIGNATURES.find(
			(signature) =>
				signature.provider.toLowerCase() === event.provider.toLowerCase() &&
				signature.ids.includes(event.eventId)
		)?.kind ?? null
	);
}

function headlineOf(event: EventRecord): string {
	const first = event.message.split('\n').find((line) => line.trim().length > 0) ?? '';
	return first.trim().length <= 160 ? first.trim() : `${first.trim().slice(0, 159)}…`;
}

function findIncidents(since: string): Incident[] {
	const found: Incident[] = [];
	for (const event of events().filter((candidate) => candidate.timeCreated >= since)) {
		const kind = classify(event);
		if (!kind) continue;
		const collapsed = found.some(
			(held) =>
				held.kind === kind &&
				Math.abs(Date.parse(held.time) - Date.parse(event.timeCreated)) <= COLLAPSE_MS
		);
		if (collapsed) continue;
		found.push({
			id: `${event.channel}:${event.recordId}`,
			time: event.timeCreated,
			kind,
			headline: headlineOf(event),
			event
		});
	}
	return found;
}

function bundleFor(channel: string, recordId: number): Bundle {
	const found = events().find((event) => event.channel === channel && event.recordId === recordId);
	if (!found) throw new Error(`${channel} no longer holds event ${recordId}`);
	const kind = classify(found);
	if (!kind) {
		throw new Error(
			`${found.provider} event ${found.eventId} is not one of the incidents this page knows about`
		);
	}

	const at = Date.parse(found.timeCreated);
	const from = new Date(at - BUNDLE_BEFORE_MS).toISOString();
	const to = new Date(at + BUNDLE_AFTER_MS).toISOString();
	const inWindow = events()
		.filter((event) => event.timeCreated >= from && event.timeCreated <= to)
		.filter((event) => event.level <= 3 && event.level >= 1)
		.filter((event) => !NOISE.some((name) => name.toLowerCase() === event.provider.toLowerCase()))
		.slice(0, 500);

	return {
		incident: {
			id: `${channel}:${recordId}`,
			time: found.timeCreated,
			kind,
			headline: headlineOf(found),
			event: found
		},
		from,
		to,
		events: inWindow
	};
}

export function mockHost<T extends CommandName>(name: T, args: CommandArgs[T]): unknown {
	switch (name) {
		case 'events_channels':
			return [...CHANNELS];

		case 'events_query': {
			const { filter } = args as CommandArgs['events_query'];
			if (filter.channels.includes('Security')) throw new Error(ACCESS_DENIED);
			const conditions = expressionCount(filter);
			if (conditions > MAX_EXPRESSIONS) {
				throw new Error(
					`this filter asks ${conditions} separate conditions and the event log accepts at most ` +
						`${MAX_EXPRESSIONS} — narrow the levels, the event ids or the providers`
				);
			}

			const started = performance.now();
			const matching = events().filter((event) => matches(event, filter));
			const max = Math.min(Math.max(filter.max, 1), 50000);
			return {
				events: matching.slice(0, max),
				truncated: matching.length > max,
				elapsedMs: Math.round(performance.now() - started)
			};
		}

		case 'events_xml': {
			const { channel, recordId } = args as CommandArgs['events_xml'];
			const event = events().find(
				(candidate) => candidate.channel === channel && candidate.recordId === recordId
			);
			if (!event) throw new Error(`${channel} no longer holds event ${recordId}`);
			return xmlOf(event);
		}

		case 'diagnose_incidents': {
			const { days } = args as CommandArgs['diagnose_incidents'];
			const newest = Date.parse(events()[0]?.timeCreated ?? new Date().toISOString());
			return findIncidents(new Date(newest - days * 24 * 60 * 60 * 1000).toISOString());
		}

		case 'diagnose_bundle': {
			const { channel, recordId } = args as CommandArgs['diagnose_bundle'];
			return bundleFor(channel, recordId);
		}

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

		case 'app_exit':
			// A browser tab cannot close itself unless a script opened it, so this only says so.
			logEntries.push({
				timestamp: new Date().toISOString(),
				level: 'info',
				source: 'host',
				message: 'Exit — the mock host has no window to close.'
			});
			return null;

		case 'open_url': {
			const { url } = args as CommandArgs['open_url'];
			window.open(url, '_blank', 'noopener');
			return null;
		}

		default:
			throw new Error(`mock host has no command ${name}`);
	}
}
