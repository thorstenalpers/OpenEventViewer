import { describe, expect, it } from 'vitest';
import { call, isMockHost } from './client';
import { commands } from './contract';

describe('bridge client', () => {
	it('runs against the mock host when there is no Tauri backend', () => {
		expect(isMockHost()).toBe(true);
	});

	it('returns validated settings', async () => {
		const stored = await call('get_settings', {});

		expect(stored.theme).toBe('system');
		expect(stored.debugLogging).toBe(false);
	});

	it('writes an entry into the log the host hands back', async () => {
		await call('log_write', { level: 'warning', source: 'test', message: 'written by the test' });
		const entries = await call('log_entries', {});

		expect(entries.at(-1)?.message).toBe('written by the test');
	});

	it('rejects a reply that does not match the contract', () => {
		expect(() => commands.assistant_status.response.parse({ source: 'cli' })).toThrow();
	});
});
