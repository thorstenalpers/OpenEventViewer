import { call } from '$lib/bridge/client';
import type { PodcastVoice, VoicePack } from '$lib/bridge/contract';
import { subscribe } from '$lib/bridge/events';

const VOICE_KEY = 'oet.voice';

/** A pack speaker as one string, so it can be a `<select>` value and a stored preference at once. */
export function voiceChoice(packId: string, speaker: number): string {
	return `${packId}#${speaker}`;
}

/** `#` rather than `:`: a pack from the hub carries its repository in the id, colon and slash and all. */
export function parseChoice(value: string): PodcastVoice | null {
	const at = value.lastIndexOf('#');
	if (at < 1) return null;
	const speaker = Number(value.slice(at + 1));
	return Number.isInteger(speaker) ? { packId: value.slice(0, at), speaker } : null;
}

/**
 * The downloaded voices and the one that reads.
 *
 * The list is read from the host rather than remembered: a pack is a folder on disk, and a folder
 * deleted between two starts has to stop being offered. That is why it is loaded once at start and
 * again after every download.
 */
class VoiceStore {
	packs = $state<VoicePack[]>([]);
	/** Empty means the Windows voice for the episode's language. */
	choice = $state('');
	/**
	 * The downloads on their way, by pack id.
	 *
	 * A map rather than one slot: both packs can be fetched at once, and each carries its own
	 * progress and its own cancel.
	 */
	running = $state<Record<string, { received: number; total: number | null; unpacking: boolean }>>(
		{}
	);
	speaking = $state(false);
	error = $state<string | null>(null);

	/** Called once from the layout, before the settings page paints. */
	restore(): void {
		if (typeof localStorage !== 'undefined') this.choice = localStorage.getItem(VOICE_KEY) ?? '';
		void this.watch();
		void this.load();
	}

	/** Re-reads the folder: a pack put there by hand counts as installed, one deleted stops counting. */
	async load(): Promise<void> {
		try {
			this.packs = await call('voice_packs', {});
			// A voice whose pack is gone would otherwise stay chosen and fail at the next episode.
			if (this.chosen && !this.installed(this.chosen.packId)) this.setChoice('');
		} catch (caught) {
			this.error = message(caught);
		}
	}

	private async watch(): Promise<void> {
		await subscribe('voice:progress', (progress) => {
			if (!this.running[progress.id]) return;
			this.running = {
				...this.running,
				[progress.id]: {
					received: progress.received,
					total: progress.total,
					unpacking: progress.unpacking
				}
			};
		});
	}

	async install(id: string): Promise<void> {
		if (this.running[id]) return;
		this.running = { ...this.running, [id]: { received: 0, total: null, unpacking: false } };
		this.error = null;
		try {
			await call('voice_install', { id });
		} catch (caught) {
			// A cancel is the user's own doing, not a failure to report as one.
			const failure = message(caught);
			if (!failure.includes('cancelled')) this.error = failure;
		} finally {
			this.running = Object.fromEntries(Object.entries(this.running).filter(([key]) => key !== id));
			// Read back however the attempt ended: a download that failed half way through still
			// changed the folder, and the list has to say what is actually in it.
			await this.load();
		}
	}

	async cancel(id: string): Promise<void> {
		if (!this.running[id]) return;
		await call('voice_cancel', { id }).catch(() => {});
	}

	async remove(id: string): Promise<void> {
		this.error = null;
		try {
			await call('voice_remove', { id });
			await this.load();
		} catch (caught) {
			this.error = message(caught);
		}
	}

	/**
	 * Reads one sentence out loud with the chosen voice, so it can be heard before it is used.
	 *
	 * The sentence is the pangram every platform reaches for, one per language. A friendlier line
	 * would leave whole groups of sounds unheard, and a voice is judged on the ones it gets wrong.
	 *
	 * The language reaches the host for the Windows voice, which is picked by language rather than
	 * by name; a pack ignores it and reads in whatever it speaks.
	 */
	async preview(text: string, language: 'en' | 'de'): Promise<void> {
		const voice = this.chosen;
		this.error = null;
		this.speaking = true;
		try {
			await call('voice_preview', {
				id: voice?.packId ?? '',
				speaker: voice?.speaker ?? 0,
				text,
				language
			});
		} catch (caught) {
			this.error = message(caught);
		} finally {
			this.speaking = false;
		}
	}

	stop(): void {
		void call('voice_stop', {}).catch(() => {});
	}

	setChoice(value: string): void {
		this.choice = value;
		// Two seconds of the wait before a preview is the model coming off the disk. Started here,
		// while the reader is still looking at the dropdown, rather than when they press play.
		const picked = parseChoice(value);
		if (picked) void call('voice_warm', { id: picked.packId }).catch(() => {});

		if (typeof localStorage === 'undefined') return;
		if (value === '') localStorage.removeItem(VOICE_KEY);
		else localStorage.setItem(VOICE_KEY, value);
	}

	/** The chosen voice as the podcast takes it, or null for the Windows voice. */
	get chosen(): PodcastVoice | null {
		return parseChoice(this.choice);
	}

	/** Every speaker of every installed pack, as one flat list. */
	get speakers(): { value: string; label: string; language: string }[] {
		return this.packs
			.filter((pack) => pack.installed)
			.flatMap((pack) =>
				Array.from({ length: Math.max(pack.voices, 1) }, (_, index) => ({
					value: voiceChoice(pack.id, index),
					label: speakerLabel(pack, index),
					language: pack.language
				}))
			);
	}

	installed(id: string): boolean {
		return this.packs.some((pack) => pack.id === id && pack.installed);
	}

	isInstalling(id: string): boolean {
		return this.running[id] !== undefined;
	}

	/** The download is done and the pack is being written out, which cannot be cancelled. */
	isUnpacking(id: string): boolean {
		return this.running[id]?.unpacking ?? false;
	}

	/** How far one download has come, 0–100, or null while the size is unknown. */
	percentOf(id: string): number | null {
		const entry = this.running[id];
		if (!entry?.total) return null;
		return Math.min(100, Math.round((entry.received / entry.total) * 100));
	}
}

/**
 * What one speaker is called in the dropdown.
 *
 * The pack's own names where it has them — "Kokoro English · af_bella" says which of eleven voices
 * is being picked, where "Kokoro English 2" says only that it is the second one. A pack with a
 * single voice is named after itself, because the speaker and the pack are the same thing.
 */
function speakerLabel(pack: VoicePack, index: number): string {
	if (pack.voices <= 1) return pack.label;
	const name = pack.speakers[index];
	return `${pack.label} · ${name ?? index + 1}`;
}

function message(caught: unknown): string {
	return caught instanceof Error ? caught.message : String(caught);
}

export const voice = new VoiceStore();
