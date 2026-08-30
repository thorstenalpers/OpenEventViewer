import { z } from 'zod';

export const dataItem = z.object({
	name: z.string(),
	value: z.string()
});
export type DataItem = z.infer<typeof dataItem>;

export const eventRecord = z.object({
	recordId: z.number(),
	channel: z.string(),
	provider: z.string(),
	eventId: z.number(),
	level: z.number(),
	levelName: z.string(),
	task: z.string(),
	keywords: z.array(z.string()),
	/** RFC 3339, UTC, milliseconds. */
	timeCreated: z.string(),
	computer: z.string(),
	message: z.string(),
	eventData: z.array(dataItem)
});
export type EventRecord = z.infer<typeof eventRecord>;

export const eventFilter = z.object({
	/** Empty means System and Application, which is what the host reads when asked for nothing. */
	channels: z.array(z.string()),
	levels: z.array(z.number()),
	from: z.string().nullable(),
	to: z.string().nullable(),
	eventIds: z.array(z.number()),
	providers: z.array(z.string()),
	max: z.number()
});
export type EventFilter = z.infer<typeof eventFilter>;

export const queryResult = z.object({
	events: z.array(eventRecord),
	/** The log held more than the cap allowed. */
	truncated: z.boolean(),
	elapsedMs: z.number()
});
export type QueryResult = z.infer<typeof queryResult>;

export const incidentKind = z.enum([
	'unexpectedShutdown',
	'bugCheck',
	'hardwareError',
	'appHang',
	'appCrash',
	'serviceFailure',
	'diskError',
	'ntfs',
	'displayTdr',
	'processorPower'
]);
export type IncidentKind = z.infer<typeof incidentKind>;

export const incident = z.object({
	/** `{channel}:{recordId}` — a record id is unique within its channel, not across the machine. */
	id: z.string(),
	time: z.string(),
	kind: incidentKind,
	headline: z.string(),
	event: eventRecord
});
export type Incident = z.infer<typeof incident>;

export const bundle = z.object({
	incident,
	from: z.string(),
	to: z.string(),
	events: z.array(eventRecord)
});
export type Bundle = z.infer<typeof bundle>;

export const settings = z.object({
	theme: z.enum(['system', 'light', 'dark']),
	showLogs: z.boolean().default(false),
	debugLogging: z.boolean().default(false)
});
export type Settings = z.infer<typeof settings>;

export const logLevel = z.enum(['debug', 'info', 'warning', 'error']);
export type LogLevel = z.infer<typeof logLevel>;

export const logEntry = z.object({
	timestamp: z.string(),
	level: logLevel,
	source: z.string(),
	message: z.string()
});
export type LogEntry = z.infer<typeof logEntry>;

/**
 * The command surface. Keys are the Tauri command names; each entry pairs the argument shape with
 * the response schema, so the client validates every reply against one declaration rather than
 * trusting the host.
 */
export const commands = {
	events_channels: { response: z.array(z.string()) },
	events_query: { response: queryResult },
	events_xml: { response: z.string() },
	diagnose_incidents: { response: z.array(incident) },
	diagnose_bundle: { response: bundle },
	get_settings: { response: settings },
	set_settings: { response: settings },
	log_entries: { response: z.array(logEntry) },
	log_clear: { response: z.null() },
	log_write: { response: z.null() },
	third_party_licenses: { response: z.string() },
	devtools_open: { response: z.null() },
	open_url: { response: z.null() },
	app_exit: { response: z.null() }
} as const;

export type CommandName = keyof typeof commands;
export type CommandResponse<T extends CommandName> = z.infer<(typeof commands)[T]['response']>;

export interface CommandArgs {
	events_channels: Record<string, never>;
	events_query: { filter: EventFilter };
	events_xml: { channel: string; recordId: number };
	diagnose_incidents: { days: number };
	diagnose_bundle: { channel: string; recordId: number };
	get_settings: Record<string, never>;
	set_settings: { settings: Settings };
	log_entries: Record<string, never>;
	log_clear: Record<string, never>;
	log_write: { level: LogLevel; source: string; message: string };
	third_party_licenses: Record<string, never>;
	devtools_open: Record<string, never>;
	open_url: { url: string };
	app_exit: Record<string, never>;
}
