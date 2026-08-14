import { render, fireEvent, screen } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('$app/paths', () => ({ resolve: (path: string) => path }));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));

import MediaView from '$lib/views/media-view.svelte';
import ProjectsView from '$lib/views/projects-view.svelte';
import { viewState } from './view-state.svelte';
import { library } from '$lib/stores/library.svelte';

/**
 * Routing unmounts a page. These check the state that must not go with it — the things the user
 * typed, and the results that cost real time to produce.
 */
describe('state kept across a view being unmounted', () => {
	beforeEach(async () => {
		viewState.importReport = null;
		viewState.video.title = '';
		viewState.video.url = '';
		viewState.podcast.pauseSeconds = 4;
		await library.refresh();
	});

	it('keeps a half-typed video form', async () => {
		const first = render(MediaView);
		await fireEvent.input(first.getByPlaceholderText('https://www.youtube.com/watch?v=…'), {
			target: { value: 'https://youtu.be/abc' }
		});
		first.unmount();

		const second = render(MediaView);
		expect(
			(second.getByPlaceholderText('https://www.youtube.com/watch?v=…') as HTMLInputElement).value
		).toBe('https://youtu.be/abc');
		second.unmount();
	});

	/// The report is the whole point of an import. Clicking to Train to look something up and
	/// coming back to an empty page reads as though the import never happened.
	it('keeps the import report', async () => {
		const first = render(ProjectsView);
		await fireEvent.click(first.getByRole('button', { name: 'Choose a file' }));
		// Anchored on the report card's own button, not on the project name: the name is in the
		// table below as well, and matching it would pass whether the card is there or not.
		expect(await first.findByText('Start training')).toBeInTheDocument();
		const recovered = viewState.importReport;
		expect(recovered).not.toBeNull();
		first.unmount();

		const second = render(ProjectsView);
		expect(await second.findByText('Start training')).toBeInTheDocument();
		expect(viewState.importReport).toBe(recovered);
		second.unmount();
	});

	/// The card belongs to an import that happened, not to the page — nothing imported, nothing
	/// reported.
	it('shows no report before anything was imported', () => {
		render(ProjectsView);
		expect(screen.queryByText('Start training')).toBeNull();
	});
});
