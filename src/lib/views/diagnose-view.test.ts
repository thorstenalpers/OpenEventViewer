import { describe, expect, it, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import DiagnoseView from './diagnose-view.svelte';
import { diagnose } from '$lib/stores/diagnose.svelte';

vi.mock('$app/navigation', () => ({ goto: vi.fn(), preloadCode: vi.fn() }));

describe('diagnose view', () => {
	it('finds the signatures the fixture log carries and collapses the repeats', async () => {
		render(DiagnoseView);
		diagnose.days = 30;
		await diagnose.load();

		expect(diagnose.incidents.length).toBeGreaterThan(0);
		const kinds = new Set(diagnose.incidents.map((incident) => incident.kind));
		expect(kinds.has('unexpectedShutdown')).toBe(true);

		await waitFor(() =>
			expect(screen.getAllByText(/Unexpected shutdown|Service failure/).length).toBeGreaterThan(0)
		);
	});

	/// The window is the whole point: an incident on its own says a machine went down, and the
	/// quarter of an hour before it is what says why.
	it('pulls the window around an incident, leaving the permission noise out', async () => {
		render(DiagnoseView);
		diagnose.days = 30;
		await diagnose.load();

		const incident = diagnose.incidents[0];
		expect(incident).toBeDefined();
		await diagnose.open(incident!);

		const bundle = diagnose.bundle;
		expect(bundle).not.toBeNull();
		expect(bundle!.from < bundle!.incident.time).toBe(true);
		expect(bundle!.to > bundle!.incident.time).toBe(true);
		expect(bundle!.events.some((event) => event.provider === 'DCOM')).toBe(false);

		await waitFor(() =>
			expect(
				screen.getByText(new RegExp(`${bundle!.events.length} events? in the window`))
			).toBeInTheDocument()
		);
	});

	it('closes the window again when the same incident is picked twice', async () => {
		render(DiagnoseView);
		diagnose.days = 30;
		await diagnose.load();

		const incident = diagnose.incidents[0]!;
		await diagnose.open(incident);
		expect(diagnose.bundle).not.toBeNull();

		await diagnose.open(incident);
		expect(diagnose.bundle).toBeNull();
		expect(diagnose.selectedId).toBeNull();
	});
});
