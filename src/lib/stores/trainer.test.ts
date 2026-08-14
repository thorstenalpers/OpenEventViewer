import { beforeEach, describe, expect, it } from 'vitest';
import { trainer } from './trainer.svelte';

describe('trainer', () => {
	beforeEach(() => trainer.reset());

	// Question 3 had its figure recovered and is drillable; question 4's could not be, so only
	// question 4 is held back.
	it('excludes questions whose figure is missing from a scored session', async () => {
		await trainer.start(1, 'practice');

		expect(trainer.session?.questions.map((q) => q.id)).toEqual([1, 2, 3]);
	});

	it('turns the wrong answers of a session into the next session', async () => {
		await trainer.start(1, 'practice');

		trainer.toggle('A');
		await trainer.submit();
		expect(trainer.lastResult?.correct).toBe(false);
		expect(trainer.lastResult?.answerLetters).toEqual(['B']);
		await trainer.next();

		trainer.toggle('D');
		trainer.toggle('A');
		await trainer.submit();
		expect(trainer.lastResult?.correct).toBe(true);
		await trainer.next();

		trainer.toggle('A');
		await trainer.submit();
		await trainer.next();

		const summary = trainer.summary;
		expect(summary?.correct).toBe(2);
		expect(summary?.wrongQuestionIds).toEqual([1]);

		await trainer.start(1, 'focus', summary!.sessionId);
		expect(trainer.session?.questions.map((q) => q.id)).toEqual([1]);
	});

	it('replaces the selection on a single-answer question and accumulates on a multi-answer one', async () => {
		await trainer.start(1, 'practice');

		trainer.toggle('A');
		trainer.toggle('C');
		expect(trainer.selected).toEqual(['C']);

		await trainer.submit();
		await trainer.next();

		trainer.toggle('A');
		trainer.toggle('B');
		expect(trainer.selected).toEqual(['A', 'B']);
	});

	it('withholds feedback in an exam and moves straight to the next question', async () => {
		await trainer.start(1, 'exam', undefined, {
			seed: null,
			questionCount: 2,
			timeLimitSeconds: 600
		});

		expect(trainer.revealsAnswer).toBe(false);
		expect(trainer.deadlineAt).not.toBeNull();

		trainer.toggle('A');
		await trainer.submit();

		expect(trainer.lastResult).toBeNull();
		expect(trainer.index).toBe(1);
		expect(trainer.selected).toEqual([]);
	});

	it('records a session summary when the clock runs out mid-question', async () => {
		await trainer.start(1, 'challenge', undefined, {
			seed: 7,
			questionCount: 2,
			timeLimitSeconds: 600
		});

		await trainer.finish();

		expect(trainer.session).toBeNull();
		expect(trainer.deadlineAt).toBeNull();
		expect(trainer.summary).not.toBeNull();
	});

	it('ignores a selection made after the answer was checked', async () => {
		await trainer.start(1, 'practice');

		trainer.toggle('B');
		await trainer.submit();
		trainer.toggle('A');

		expect(trainer.selected).toEqual(['B']);
	});
});
