import { fireEvent, render, screen } from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('$app/paths', () => ({ resolve: (path: string) => path }));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));

import TrainView from './train-view.svelte';
import { call } from '$lib/bridge/client';
import { library } from '$lib/stores/library.svelte';
import { trainer } from '$lib/stores/trainer.svelte';

/** Runs a whole challenge on the selected binder and leaves the summary on screen. */
async function runChallenge(binderId: number) {
	await trainer.start(binderId, 'challenge', undefined, {
		seed: 7,
		questionCount: null,
		timeLimitSeconds: null
	});
	while (trainer.session) {
		trainer.toggle(trainer.current?.answerLetters[0] ?? 'A');
		await trainer.submit();
	}
}

describe('train view — posting a challenge result', () => {
	beforeEach(async () => {
		trainer.reset();
		await library.refresh();
	});

	afterEach(() => trainer.reset());

	/// Without a published entry there is no board, so the offer is not made — and the reason it is
	/// not made is on screen rather than left to be guessed at.
	it('explains why an unpublished binder has no board to post to', async () => {
		await runChallenge(1);
		render(TrainView);

		expect(await screen.findByText(/Publish this binder to the catalog/)).toBeInTheDocument();
		expect(screen.queryByRole('button', { name: 'Post to the catalog board' })).toBeNull();
	});

	it('posts a finished challenge to the board of the entry the binder was published as', async () => {
		await call('catalog_publish', { binderId: 1 });
		await library.refresh();
		await runChallenge(1);

		const view = render(TrainView);
		const post = await view.findByRole('button', { name: 'Post to the catalog board' });
		await fireEvent.click(post);

		expect(await view.findByText(/Posted — 1 of 1 on that board/)).toBeInTheDocument();
		// One post per press: the button goes once the run is on the board.
		expect(view.queryByRole('button', { name: 'Post to the catalog board' })).toBeNull();
		view.unmount();
	});

	/// A practice run shares its questions with nobody, so it is not offered a board at all.
	it('offers no board after a practice run', async () => {
		await trainer.start(1, 'practice');
		while (trainer.session) {
			trainer.toggle(trainer.current?.answerLetters[0] ?? 'A');
			await trainer.submit();
			if (trainer.lastResult) await trainer.next();
		}

		const view = render(TrainView);

		expect(view.queryByRole('button', { name: 'Post to the catalog board' })).toBeNull();
		expect(view.queryByText(/Publish this binder to the catalog/)).toBeNull();
		view.unmount();
	});
});
