import { fireEvent, render, screen, within } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';

vi.mock('$app/paths', () => ({ resolve: (path: string) => path }));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));

import CatalogView from './catalog-view.svelte';

describe('catalog view', () => {
	it('lists what the catalog holds, with an unrated binder saying so', async () => {
		render(CatalogView);

		expect(await screen.findByText('AZ-900 Fundamentals')).toBeInTheDocument();
		expect(screen.getByText('SC-900 Security Fundamentals')).toBeInTheDocument();
		expect(screen.getByText('4.5 from 2 ratings')).toBeInTheDocument();
		// The difference between 'nobody has rated it' and 'everybody rated it nought'.
		expect(screen.getByText('not rated')).toBeInTheDocument();
	});

	/// Hard rule 2: what would leave the machine is shown before anything does, and the publish
	/// button only exists behind that preview.
	it('shows the upload preview before it offers to publish', async () => {
		const view = render(CatalogView);

		expect(view.queryByRole('button', { name: 'Publish' })).not.toBeInTheDocument();
		await fireEvent.click(
			await view.findByRole('button', { name: /Show what would be published/ })
		);

		expect(await view.findByText('What would be published')).toBeInTheDocument();
		expect(
			view.getByText('The imported PDF stays here: a deck carries no sources folder.')
		).toBeInTheDocument();
		expect(view.getByRole('button', { name: 'Publish' })).toBeInTheDocument();
		view.unmount();
	});

	it('publishes the chosen project and marks the entry as its own', async () => {
		const view = render(CatalogView);

		await fireEvent.click(
			await view.findByRole('button', { name: /Show what would be published/ })
		);
		await fireEvent.click(await view.findByRole('button', { name: 'Publish' }));

		const published = await view.findByText('AI-900 (mock)');
		const row = published.closest('li');
		expect(row).not.toBeNull();
		expect(within(row as HTMLElement).getByText('yours')).toBeInTheDocument();
		// Only an owner is offered the withdrawal, which is the policy a server would enforce.
		expect(
			within(row as HTMLElement).getByRole('button', { name: 'Withdraw' })
		).toBeInTheDocument();
		view.unmount();
	});

	it('leaves a foreign entry without a withdrawal', async () => {
		const view = render(CatalogView);

		const foreign = (await view.findByText('SC-900 Security Fundamentals')).closest('li');
		expect(foreign).not.toBeNull();
		expect(within(foreign as HTMLElement).queryByRole('button', { name: 'Withdraw' })).toBeNull();
		expect(
			within(foreign as HTMLElement).getByRole('button', { name: /Import/ })
		).toBeInTheDocument();
		view.unmount();
	});

	it('opens ratings and the board for one entry', async () => {
		const view = render(CatalogView);

		await fireEvent.click(await view.findByText('AZ-900 Fundamentals'));

		expect(await view.findByText('mira')).toBeInTheDocument();
		expect(view.getByText('Ratings')).toBeInTheDocument();
		expect(view.getByText('Leaderboard')).toBeInTheDocument();
		expect(await view.findByText('18 of 20')).toBeInTheDocument();
		view.unmount();
	});

	it('reports what a sync moved and what it left alone', async () => {
		const view = render(CatalogView);

		await fireEvent.click(await view.findByRole('button', { name: 'Push' }));

		expect(await view.findByText('3 pushed, 0 pulled, 1 left alone.')).toBeInTheDocument();
		view.unmount();
	});
});
