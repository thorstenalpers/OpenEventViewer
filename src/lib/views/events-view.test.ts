import { afterEach, describe, expect, it, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import EventsView from './events-view.svelte';
import { events } from '$lib/stores/events.svelte';

vi.mock('$app/navigation', () => ({ goto: vi.fn(), preloadCode: vi.fn() }));

describe('events view', () => {
	afterEach(() => {
		events.channel = '__all__';
		events.error = null;
	});

	it('loads the fixture log and offers the channels the host lists', async () => {
		render(EventsView);

		await waitFor(() => expect(events.events.length).toBeGreaterThan(0));
		expect(screen.getByLabelText('Channel')).toBeInTheDocument();
		expect(screen.getByRole('option', { name: 'Security' })).toBeInTheDocument();
	});

	/// Access denied on Security is the operating system doing its job, not a bug in the app, and
	/// the page has to say which of the two it is.
	it('shows the elevation hint when a channel refuses a normal account', async () => {
		render(EventsView);
		await waitFor(() => expect(events.events.length).toBeGreaterThan(0));

		events.channel = 'Security';
		await events.load();

		expect(events.accessDenied).toBe(true);
		await waitFor(() =>
			expect(screen.getByText(/pick a channel that does not need it/)).toBeInTheDocument()
		);
	});
});
