import { describe, expect, it } from 'vitest';
import { call, isMockHost } from './client';
import { commands } from './contract';

describe('bridge client', () => {
	it('runs against the mock host when there is no Tauri backend', () => {
		expect(isMockHost()).toBe(true);
	});

	it('returns a validated project list', async () => {
		const projects = await call('list_binders', {});

		expect(projects).toHaveLength(3);
		expect(projects[0]?.certification).toBe('AI-900');
	});

	/// A project created without a file is a valid row, not a half-written one — the contract has
	/// to accept an empty source and a null accuracy rather than reject them.
	it('accepts a project that has no file yet', async () => {
		const created = await call('create_project', { title: 'Cosmos DB', certification: 'DP-420' });

		expect(created.certification).toBe('DP-420');
		expect(created.questionCount).toBe(0);
		expect(created.sourceFile).toBe('');
		expect(created.accuracy).toBeNull();
	});

	it('scores a multi-answer question regardless of the order the letters arrive in', async () => {
		const session = await call('start_session', { binderId: 1, mode: 'practice' });

		const forwards = await call('record_attempt', {
			sessionId: session.id,
			questionId: 2,
			given: ['A', 'D'],
			elapsedMs: 1000
		});
		const backwards = await call('record_attempt', {
			sessionId: session.id,
			questionId: 2,
			given: ['D', 'A'],
			elapsedMs: 1000
		});

		expect(forwards.correct).toBe(true);
		expect(backwards.correct).toBe(true);
	});

	it('rejects a reply that does not match the contract', () => {
		expect(() => commands.list_binders.response.parse([{ id: 1, title: 'incomplete' }])).toThrow();
	});
});
