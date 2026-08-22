import { describe, expect, it, vi } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import DiagnoseView from './diagnose-view.svelte';
import { diagnose } from '$lib/stores/diagnose.svelte';
import { assistant } from '$lib/stores/assistant.svelte';

const goto = vi.fn();
vi.mock('$app/navigation', () => ({
	goto: (url: string) => {
		goto(url);
	},
	preloadCode: vi.fn()
}));

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

	/// The bundle is the whole point: an incident on its own says a machine went down, and the
	/// quarter of an hour before it is what says why.
	it('pulls the window around an incident and previews exactly what would be sent', async () => {
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
			expect(screen.getByTestId('bundle-preview').textContent).toBe(bundle!.prompt)
		);
	});

	it('hands the bundle to the assistant unchanged', async () => {
		assistant.reset();
		render(DiagnoseView);
		diagnose.days = 30;
		await diagnose.load();
		await diagnose.open(diagnose.incidents[0]!);

		const bundle = diagnose.bundle!;
		screen.getByRole('button', { name: /Send to the assistant/ }).click();

		await waitFor(() => expect(assistant.attachments).toHaveLength(1));
		expect(assistant.attachments[0]?.text).toBe(bundle.prompt);
		expect(assistant.composeNext()).toContain(bundle.prompt.trim());
		expect(goto).toHaveBeenCalled();
	});
});
