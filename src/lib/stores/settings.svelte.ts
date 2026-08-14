import { call } from '$lib/bridge/client';
import type { AssistantSource } from '$lib/bridge/contract';
import { i18n, isLocale, type Locale } from '$lib/i18n/index.svelte';
import { applyPreset, isThemePreset, type ThemePreset } from '$lib/theme/preset';

const LOCALE_KEY = 'oet.locale';
const PRESET_KEY = 'oet.preset';
const SIDEBAR_KEY = 'oet.sidebar';

/**
 * Preferences the UI owns. They live in `localStorage` rather than in the host's settings file:
 * they only affect what this webview paints, and reading them before the first frame is what keeps
 * the app from flashing the wrong palette on every start.
 */
class SettingsStore {
	assistantSource = $state<AssistantSource>('cli');
	locale = $state<Locale>('en');
	preset = $state<ThemePreset>('default');
	sidebarExpanded = $state(true);

	// These two belong to the host, not the webview: the log buffer lives in Rust and has to know
	// about `debugLogging` before the first command runs, which `localStorage` cannot tell it.
	showLogs = $state(false);
	debugLogging = $state(false);

	/** Called once from the layout, before anything reads a translation. */
	restore(): void {
		void this.loadHostSettings();
		if (typeof localStorage === 'undefined') return;

		const locale = localStorage.getItem(LOCALE_KEY);
		if (locale && isLocale(locale)) this.locale = locale;
		i18n.locale = this.locale;

		const preset = localStorage.getItem(PRESET_KEY);
		if (preset && isThemePreset(preset)) this.preset = preset;
		applyPreset(this.preset);

		this.sidebarExpanded = localStorage.getItem(SIDEBAR_KEY) !== 'collapsed';
	}

	toggleSidebar(): void {
		this.sidebarExpanded = !this.sidebarExpanded;
		localStorage?.setItem(SIDEBAR_KEY, this.sidebarExpanded ? 'expanded' : 'collapsed');
	}

	setLocale(locale: Locale): void {
		this.locale = locale;
		i18n.locale = locale;
		localStorage?.setItem(LOCALE_KEY, locale);
	}

	setPreset(preset: ThemePreset): void {
		this.preset = preset;
		applyPreset(preset);
		localStorage?.setItem(PRESET_KEY, preset);
	}

	private async loadHostSettings(): Promise<void> {
		try {
			const stored = await call('get_settings', {});
			this.showLogs = stored.showLogs;
			this.debugLogging = stored.debugLogging;
		} catch {
			// A host that cannot answer leaves the defaults standing; the log is a diagnostic, and
			// failing to read its switches must not take the app down with it.
		}
	}

	async setLogging(changes: { showLogs?: boolean; debugLogging?: boolean }): Promise<void> {
		const next = {
			theme: 'system' as const,
			showLogs: changes.showLogs ?? this.showLogs,
			debugLogging: changes.debugLogging ?? this.debugLogging
		};
		const saved = await call('set_settings', { settings: next });
		this.showLogs = saved.showLogs;
		this.debugLogging = saved.debugLogging;
	}
}

export const settings = new SettingsStore();
