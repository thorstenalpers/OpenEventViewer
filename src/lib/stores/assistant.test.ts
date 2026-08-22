import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { EventRecord } from '$lib/bridge/contract';
import { assistant } from './assistant.svelte';
import * as client from '$lib/bridge/client';

function event(recordId: number, message: string): EventRecord {
	return {
		recordId,
		channel: 'System',
		provider: 'Microsoft-Windows-Kernel-Power',
		eventId: 41,
		level: 1,
		levelName: 'Critical',
		task: 'None',
		keywords: [],
		timeCreated: '2026-08-20T09:00:00.000Z',
		computer: 'WORKBENCH',
		message,
		eventData: []
	};
}

describe('assistant store', () => {
	beforeEach(async () => {
		assistant.reset();
		await assistant.refreshStatus();
	});

	it('composes the draft alone when nothing is attached', () => {
		assistant.draft = '  why did it reboot?  ';

		expect(assistant.composeNext()).toBe('why did it reboot?');
	});

	it('files the attached text under one heading below the draft', async () => {
		assistant.draft = 'why did it reboot?';
		await assistant.attachEvent(event(1, 'the system has rebooted'));

		const composed = assistant.composeNext();

		expect(composed.startsWith('why did it reboot?')).toBe(true);
		expect(composed).toContain('--- Attached events ---');
		expect(composed).toContain('the system has rebooted');
	});

	/// The hard rule: the preview and the send are the same string, built once. A `send` that
	/// appended anything of its own would make the preview a promise the app does not keep.
	it('posts exactly the string the preview showed', async () => {
		assistant.draft = 'what happened here?';
		await assistant.attachEvent(event(2, 'unexpected shutdown'));
		const previewed = assistant.composeNext();

		const spy = vi.spyOn(client, 'call');
		await assistant.send();

		const posted = spy.mock.calls.find(([name]) => name === 'assistant_chat')?.[1] as {
			messages: { role: string; content: string }[];
		};
		expect(posted.messages.at(-1)?.content).toBe(previewed);
		spy.mockRestore();
	});

	it('clears the attachments once they are part of the transcript', async () => {
		assistant.draft = 'and now?';
		await assistant.attachEvent(event(3, 'a third one'));
		await assistant.send();

		expect(assistant.attachments).toHaveLength(0);
		expect(assistant.draft).toBe('');
		expect(assistant.messages).toHaveLength(2);
	});

	it('attaches an event once however often the button is pressed', async () => {
		await assistant.attachEvent(event(4, 'same event'));
		await assistant.attachEvent(event(4, 'same event'));

		expect(assistant.attachments).toHaveLength(1);
	});

	it('knows the standing instructions the host would prepend', () => {
		expect(assistant.systemPrompt).toContain('never invent an event');
	});
});
