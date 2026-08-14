import { invoke } from '@tauri-apps/api/core';
import { commands, type CommandArgs, type CommandName, type CommandResponse } from './contract';
import { mockHost } from './mock';

function hasHost(): boolean {
	return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

/**
 * The single door to the host. Every reply is parsed against the contract, so a host that drifts
 * fails here with the offending field rather than three components later with `undefined`.
 */
export async function call<T extends CommandName>(
	name: T,
	args: CommandArgs[T]
): Promise<CommandResponse<T>> {
	const raw = hasHost()
		? await invoke(name, args as Record<string, unknown>)
		: mockHost(name, args);
	return commands[name].response.parse(raw) as CommandResponse<T>;
}

export function isMockHost(): boolean {
	return !hasHost();
}
