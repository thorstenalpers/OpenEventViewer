import { beforeEach, describe, expect, it } from 'vitest';
import { notes } from './notes.svelte';

describe('notes', () => {
	beforeEach(async () => {
		await notes.load(1);
	});

	it('keeps a note against the question it was written for', async () => {
		await notes.save(1, 42, 'Anomaly detection covers fraud.');

		expect(notes.forQuestion(42).map((n) => n.bodyMd)).toEqual(['Anomaly detection covers fraud.']);
		expect(notes.forQuestion(7)).toEqual([]);
	});

	it('refuses to store an empty note rather than creating a blank one', async () => {
		const before = notes.notes.length;

		await notes.save(1, 42, '   ');

		expect(notes.notes.length).toBe(before);
	});

	it('trims what it stores', async () => {
		await notes.save(1, 99, '  the rule to remember  ');

		expect(notes.forQuestion(99)[0]?.bodyMd).toBe('the rule to remember');
	});

	/// The host returns a fresh array on every call. A mock that handed back its own live array
	/// would leave the view showing stale content after the first save — which it did.
	it('replaces the list rather than mutating the one already on screen', async () => {
		const before = notes.notes;

		await notes.save(1, 1, 'first');

		expect(notes.notes).not.toBe(before);
	});
});
