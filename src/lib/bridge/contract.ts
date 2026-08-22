import { z } from 'zod';

export const assistantSource = z.enum(['cli', 'anthropic']);
export type AssistantSource = z.infer<typeof assistantSource>;

export const assistantStatus = z.object({
	source: assistantSource,
	cliAvailable: z.boolean(),
	hasKey: z.boolean()
});
export type AssistantStatus = z.infer<typeof assistantStatus>;

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
	get_settings: { response: settings },
	set_settings: { response: settings },
	log_entries: { response: z.array(logEntry) },
	log_clear: { response: z.null() },
	log_write: { response: z.null() },
	third_party_licenses: { response: z.string() },
	devtools_open: { response: z.null() },
	assistant_status: { response: assistantStatus },
	assistant_set_key: { response: z.null() }
} as const;

export type CommandName = keyof typeof commands;
export type CommandResponse<T extends CommandName> = z.infer<(typeof commands)[T]['response']>;

export interface CommandArgs {
	get_settings: Record<string, never>;
	set_settings: { settings: Settings };
	log_entries: Record<string, never>;
	log_clear: Record<string, never>;
	log_write: { level: LogLevel; source: string; message: string };
	third_party_licenses: Record<string, never>;
	devtools_open: Record<string, never>;
	assistant_status: { source: AssistantSource };
	assistant_set_key: { key: string };
}
