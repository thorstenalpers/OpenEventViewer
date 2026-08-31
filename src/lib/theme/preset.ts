export const THEME_PRESETS = [
	'default',
	'caffeine',
	'catppuccin',
	'claude',
	'modern-minimal',
	'mono',
	'northern-lights',
	'supabase',
	'tangerine',
	'twitter',
	'vercel'
] as const;

export type ThemePreset = (typeof THEME_PRESETS)[number];

export function isThemePreset(value: string): value is ThemePreset {
	return (THEME_PRESETS as readonly string[]).includes(value);
}

/**
 * Applies a colour preset to `<html>`.
 *
 * Transitions are suppressed across the swap. Chromium keeps the *old* colour indefinitely on any
 * element with a `transition` covering `background-color` when that colour comes from a custom
 * property that changes on an ancestor — and the buttons here all carry `transition-colors`, so
 * without this guard they stay painted in the previous preset permanently, not just for the
 * transition duration.
 */
export function applyPreset(preset: ThemePreset): void {
	if (typeof document === 'undefined') return;

	const suppressor = document.createElement('style');
	suppressor.textContent =
		'*,*::before,*::after{transition:none!important;animation:none!important}';
	document.head.appendChild(suppressor);

	const root = document.documentElement;
	for (const candidate of THEME_PRESETS) {
		root.classList.toggle(`theme-${candidate}`, candidate === preset && candidate !== 'default');
	}

	// Force the new values to be computed while transitions are still suppressed.
	void document.body.offsetHeight;

	// The timeout is not belt and braces: a webview that is not compositing never fires rAF, and a
	// suppressor left in the document would kill every transition in the app for good.
	let removed = false;
	const remove = () => {
		if (removed) return;
		removed = true;
		suppressor.remove();
	};
	requestAnimationFrame(() => requestAnimationFrame(remove));
	setTimeout(remove, 100);
}
