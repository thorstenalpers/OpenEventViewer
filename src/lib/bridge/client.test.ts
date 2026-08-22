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

	it('narrows the fixture log by channel, level and time', async () => {
		const all = await call('events_query', {
			filter: {
				channels: ['System'],
				levels: [],
				from: null,
				to: null,
				eventIds: [],
				providers: [],
				max: 50000
			}
		});
		const critical = await call('events_query', {
			filter: {
				channels: ['System'],
				levels: [1],
				from: null,
				to: null,
				eventIds: [],
				providers: [],
				max: 50000
			}
		});

		expect(all.events.length).toBeGreaterThan(critical.events.length);
		expect(critical.events.every((event) => event.level === 1)).toBe(true);
		expect(all.events.every((event) => event.channel === 'System')).toBe(true);
	});

	it('says plainly that Security needs an elevated process', async () => {
		await expect(
			call('events_query', {
				filter: {
					channels: ['Security'],
					levels: [],
					from: null,
					to: null,
					eventIds: [],
					providers: [],
					max: 100
				}
			})
		).rejects.toThrow(/administrator rights/);
	});

	it('reports truncation rather than quietly cutting the list short', async () => {
		const result = await call('events_query', {
			filter: {
				channels: ['System'],
				levels: [],
				from: null,
				to: null,
				eventIds: [],
				providers: [],
				max: 10
			}
		});

		expect(result.events).toHaveLength(10);
		expect(result.truncated).toBe(true);
	});

	it('rejects a reply that does not match the contract', () => {
		expect(() => commands.assistant_status.response.parse({ source: 'cli' })).toThrow();
	});
});
