import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import AssistantView from './assistant-view.svelte';
import { assistant } from '$lib/stores/assistant.svelte';
import type { EventRecord } from '$lib/bridge/contract';

vi.mock('$app/navigation', () => ({ goto: vi.fn(), preloadCode: vi.fn() }));

const EVENT: EventRecord = {
	recordId: 41,
	channel: 'System',
	provider: 'Microsoft-Windows-Kernel-Power',
	eventId: 41,
	level: 1,
	levelName: 'Critical',
	task: 'None',
	keywords: [],
	timeCreated: '2026-08-20T09:00:00.000Z',
	computer: 'WORKBENCH',
	message: 'the system has rebooted without cleanly shutting down first',
	eventData: []
};

describe('assistant view', () => {
	beforeEach(() => {
		assistant.reset();
	});

	/// The preview is the app's whole promise about what leaves the machine, so it has to open
	/// itself the moment there is more in the message than the user typed.
	it('opens the preview on its own once an event is attached, and shows the text verbatim', async () => {
		render(AssistantView);
		await assistant.attachEvent(EVENT);

		await waitFor(() => expect(screen.getByTestId('preview').textContent).toContain('EventID 41'));
		expect(screen.getByTestId('preview').textContent).toBe(assistant.composeNext());
	});

	it('will not send until the host says the assistant is reachable', async () => {
		render(AssistantView);
		assistant.status = { source: 'cli', cliAvailable: false, hasKey: false, systemPrompt: '' };
		assistant.draft = 'anything';

		await waitFor(() => expect(screen.getByRole('button', { name: /Send/ })).toBeDisabled());

		assistant.status = { source: 'cli', cliAvailable: true, hasKey: false, systemPrompt: '' };
		await waitFor(() => expect(screen.getByRole('button', { name: /Send/ })).toBeEnabled());
	});

	it('will not send an empty message', async () => {
		render(AssistantView);
		await waitFor(() => expect(assistant.ready).toBe(true));

		expect(screen.getByRole('button', { name: /Send/ })).toBeDisabled();
	});
});
