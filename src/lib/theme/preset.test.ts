import { beforeEach, describe, expect, it } from 'vitest';
import { applyPreset, isThemePreset, THEME_PRESETS } from './preset';

describe('theme presets', () => {
	beforeEach(() => {
		document.documentElement.className = '';
	});

	it('puts exactly one preset class on the root element', () => {
		applyPreset('caffeine');
		expect(document.documentElement.classList.contains('theme-caffeine')).toBe(true);

		applyPreset('vercel');
		expect(document.documentElement.classList.contains('theme-caffeine')).toBe(false);
		expect(document.documentElement.classList.contains('theme-vercel')).toBe(true);
	});

	it('carries no class of its own for the default palette', () => {
		applyPreset('mono');
		applyPreset('default');

		const classes = [...document.documentElement.classList];
		expect(classes.filter((name) => name.startsWith('theme-'))).toEqual([]);
	});

	it('leaves the light/dark class alone — they are separate axes', () => {
		document.documentElement.classList.add('dark');

		applyPreset('northern-lights');

		expect(document.documentElement.classList.contains('dark')).toBe(true);
		expect(document.documentElement.classList.contains('theme-northern-lights')).toBe(true);
	});

	it('accepts only the presets themes.css actually defines', () => {
		expect(THEME_PRESETS).toContain('default');
		expect(isThemePreset('vercel')).toBe(true);
		expect(isThemePreset('twitter')).toBe(false);
	});
});
