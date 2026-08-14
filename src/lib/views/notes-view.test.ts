import { fireEvent, render, screen, within } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('$app/paths', () => ({ resolve: (path: string) => path }));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));

import NotesView from './notes-view.svelte';
import { library } from '$lib/stores/library.svelte';

describe('notes view', () => {
	beforeEach(async () => {
		await library.refresh();
	});

	/// A summary is what the PDF and the episode are both made from, so neither is offered until
	/// there is one — an export button over an empty folder is a button that can only fail.
	it('offers nothing to export before a summary exists', async () => {
		const view = render(NotesView);

		expect(await view.findByText('Nothing made yet.')).toBeInTheDocument();
		expect(view.queryByRole('button', { name: 'As PDF' })).not.toBeInTheDocument();
		view.unmount();
	});

	it('sets a summary as a PDF and lists it beside the Markdown', async () => {
		const view = render(NotesView);

		// A summary is written from notes, so there has to be one before anything can be exported.
		const draft = await view.findByPlaceholderText('What you want to remember…');
		await fireEvent.input(draft, { target: { value: 'Blob Storage holds unstructured data.' } });
		await fireEvent.click(view.getByRole('button', { name: 'Save note' }));

		await fireEvent.click(await view.findByRole('button', { name: 'Write a summary' }));
		const summary = await view.findByText(/\.md$/);
		const row = summary.closest('li');
		expect(row).not.toBeNull();

		await fireEvent.click(within(row as HTMLElement).getByRole('button', { name: 'As PDF' }));

		const paper = await view.findByText(/\.pdf$/);
		expect(paper).toBeInTheDocument();
		// The Markdown stays: the PDF is made from it, not instead of it.
		expect(screen.getByText(/\.md$/)).toBeInTheDocument();
		view.unmount();
	});

	/// An episode is made from a summary too, so a PDF must not offer to be read out loud.
	it('leaves the PDF without the buttons that only a summary answers', async () => {
		const view = render(NotesView);

		const paper = (await view.findByText(/\.pdf$/)).closest('li');
		expect(paper).not.toBeNull();
		expect(within(paper as HTMLElement).queryByRole('button', { name: 'As PDF' })).toBeNull();
		expect(within(paper as HTMLElement).queryByRole('button', { name: 'As podcast' })).toBeNull();
		view.unmount();
	});
});
