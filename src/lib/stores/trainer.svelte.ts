import { call } from '$lib/bridge/client';
import type {
	AttemptResult,
	Question,
	RuleSet,
	Session,
	SessionMode,
	SessionSummary
} from '$lib/bridge/contract';

class TrainerStore {
	session = $state<Session | null>(null);
	index = $state(0);
	selected = $state<string[]>([]);
	lastResult = $state<AttemptResult | null>(null);
	summary = $state<SessionSummary | null>(null);
	busy = $state(false);
	error = $state<string | null>(null);
	/** Epoch milliseconds, or null when the session is untimed. */
	deadlineAt = $state<number | null>(null);

	private questionStartedAt = 0;

	get current(): Question | null {
		return this.session?.questions[this.index] ?? null;
	}

	get remaining(): number {
		return this.session ? this.session.questions.length - this.index : 0;
	}

	get required(): number {
		return this.current?.answerLetters.length ?? 1;
	}

	/** Exam and challenge runs withhold feedback until the end, exactly as the real exam does. */
	get revealsAnswer(): boolean {
		const mode = this.session?.mode;
		return mode !== 'exam' && mode !== 'challenge';
	}

	async start(
		binderId: number,
		mode: SessionMode,
		sourceSessionId?: number,
		rules?: RuleSet
	): Promise<void> {
		this.busy = true;
		this.error = null;
		try {
			const started = await call('start_session', { binderId, mode, sourceSessionId, rules });
			this.session = started;
			this.index = 0;
			this.selected = [];
			this.lastResult = null;
			this.summary = null;
			this.questionStartedAt = Date.now();
			this.deadlineAt = started.rules.timeLimitSeconds
				? Date.now() + started.rules.timeLimitSeconds * 1000
				: null;
		} catch (caught) {
			this.error = caught instanceof Error ? caught.message : String(caught);
		} finally {
			this.busy = false;
		}
	}

	toggle(letter: string): void {
		if (this.lastResult) return;
		if (this.required === 1) {
			this.selected = [letter];
			return;
		}
		this.selected = this.selected.includes(letter)
			? this.selected.filter((l) => l !== letter)
			: [...this.selected, letter];
	}

	async submit(): Promise<void> {
		const question = this.current;
		if (!this.session || !question || this.lastResult) return;
		this.busy = true;
		try {
			const result = await call('record_attempt', {
				sessionId: this.session.id,
				questionId: question.id,
				given: this.selected,
				elapsedMs: Date.now() - this.questionStartedAt
			});
			if (this.revealsAnswer) {
				this.lastResult = result;
			}
		} catch (caught) {
			this.error = caught instanceof Error ? caught.message : String(caught);
		} finally {
			this.busy = false;
		}
		if (!this.revealsAnswer) await this.next();
	}

	async next(): Promise<void> {
		if (!this.session) return;
		if (this.index + 1 < this.session.questions.length) {
			this.index += 1;
			this.selected = [];
			this.lastResult = null;
			this.questionStartedAt = Date.now();
			return;
		}
		await this.finish();
	}

	/** Also the path the clock takes when a timed run expires mid-question. */
	async finish(): Promise<void> {
		if (!this.session) return;
		this.busy = true;
		try {
			this.summary = await call('finish_session', { sessionId: this.session.id });
			this.session = null;
			this.deadlineAt = null;
		} catch (caught) {
			this.error = caught instanceof Error ? caught.message : String(caught);
		} finally {
			this.busy = false;
		}
	}

	reset(): void {
		this.session = null;
		this.summary = null;
		this.lastResult = null;
		this.selected = [];
		this.index = 0;
		this.error = null;
		this.deadlineAt = null;
	}
}

export const trainer = new TrainerStore();
