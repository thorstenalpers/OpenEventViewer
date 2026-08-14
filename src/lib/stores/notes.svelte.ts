import { call } from '$lib/bridge/client';
import type { Note } from '$lib/bridge/contract';

/**
 * The binder's notes, kept in one place so a note written in Review and a note saved from the
 * assistant land in the same list without either view knowing about the other.
 */
class NotesStore {
	notes = $state<Note[]>([]);
	binderId = $state<number | null>(null);
	error = $state<string | null>(null);

	forQuestion(questionId: number): Note[] {
		return this.notes.filter((note) => note.questionId === questionId);
	}

	async load(binderId: number): Promise<void> {
		this.binderId = binderId;
		this.error = null;
		try {
			this.notes = await call('list_notes', { binderId });
		} catch (caught) {
			this.error = caught instanceof Error ? caught.message : String(caught);
		}
	}

	async save(binderId: number, questionId: number | null, bodyMd: string): Promise<void> {
		if (!bodyMd.trim()) return;
		this.error = null;
		try {
			this.notes = await call('save_note', {
				binderId,
				note: { questionId, bodyMd: bodyMd.trim() }
			});
			this.binderId = binderId;
		} catch (caught) {
			this.error = caught instanceof Error ? caught.message : String(caught);
		}
	}
}

export const notes = new NotesStore();
