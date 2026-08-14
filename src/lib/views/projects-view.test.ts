import { fireEvent, render, screen } from '@testing-library/svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('$app/paths', () => ({ resolve: (path: string) => path }));
vi.mock('$app/navigation', () => ({ goto: vi.fn() }));

import ProjectsView from './projects-view.svelte';
import { library } from '$lib/stores/library.svelte';

describe('projects view', () => {
	beforeEach(async () => {
		await library.refresh();
	});

	it('builds the table and renders a row per project', async () => {
		render(ProjectsView);

		expect(await screen.findByText('AI-900 (mock)')).toBeInTheDocument();
		expect(screen.getByText('3 exams on this machine.')).toBeInTheDocument();
	});

	it('offers every column as a sort control', () => {
		render(ProjectsView);

		for (const header of ['Project', 'Certification', 'Questions', 'Created', 'Accuracy']) {
			expect(screen.getByRole('button', { name: new RegExp(`^${header}`) })).toBeInTheDocument();
		}
	});

	/// Newest first is the point of the default sort: the project you just made is the one you want.
	it('sorts by creation date, newest first', () => {
		render(ProjectsView);

		const rows = screen.getAllByRole('row').slice(1);
		const titles = rows.map((row) => row.querySelector('td')?.textContent?.trim());

		expect(titles).toEqual(['AI-900 (mock)', 'Azure AI Engineer', 'Security Fundamentals']);
	});

	/// Routing unmounts the page, so without the kept state a trip to Train and back would drop the
	/// filter the user just typed and silently return to the default sort.
	it('keeps sort and filter across an unmount', async () => {
		const first = render(ProjectsView);
		const filter = first.getByPlaceholderText('Filter projects…') as HTMLInputElement;
		await fireEvent.input(filter, { target: { value: 'SC-900' } });
		expect(first.getAllByRole('row')).toHaveLength(2); // header plus one match
		first.unmount();

		const second = render(ProjectsView);
		expect((second.getByPlaceholderText('Filter projects…') as HTMLInputElement).value).toBe(
			'SC-900'
		);
		expect(second.getAllByRole('row')).toHaveLength(2);
		second.unmount();

		// Left as it was found, or the next test inherits a filter it never set.
		const third = render(ProjectsView);
		await fireEvent.input(third.getByPlaceholderText('Filter projects…'), {
			target: { value: '' }
		});
		third.unmount();
	});

	/// A project with no file yet offers the way to give it one, not a training button that would
	/// open an empty session.
	it('offers a file to a project that has none', () => {
		render(ProjectsView);

		expect(screen.getByText('no file yet')).toBeInTheDocument();
		expect(screen.getByRole('button', { name: 'Add file' })).toBeInTheDocument();
	});
});
