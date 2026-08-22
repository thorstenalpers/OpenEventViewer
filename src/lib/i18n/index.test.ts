import { describe, expect, it } from 'vitest';
import { de } from './de';
import { en } from './en';
import { i18n, isLocale, LOCALES } from './index.svelte';

/** Walks both dictionaries in step so a key added to one and forgotten in the other is a failure. */
function compare(left: unknown, right: unknown, path: string, problems: string[]): void {
	if (typeof left === 'function') {
		if (typeof right !== 'function') problems.push(`${path}: not a function in de`);
		return;
	}
	if (typeof left === 'string') {
		if (typeof right !== 'string') problems.push(`${path}: not a string in de`);
		else if (right.trim() === '') problems.push(`${path}: empty in de`);
		return;
	}
	if (left && typeof left === 'object') {
		const a = left as Record<string, unknown>;
		const b = (right ?? {}) as Record<string, unknown>;
		for (const key of Object.keys(a)) {
			if (!(key in b)) problems.push(`${path}.${key}: missing in de`);
			else compare(a[key], b[key], `${path}.${key}`, problems);
		}
		for (const key of Object.keys(b)) {
			if (!(key in a)) problems.push(`${path}.${key}: extra in de`);
		}
	}
}

describe('i18n', () => {
	it('has a German entry for every English one and nothing extra', () => {
		const problems: string[] = [];
		compare(en, de, 'root', problems);

		expect(problems).toEqual([]);
	});

	it('switches the interface language', () => {
		i18n.locale = 'en';
		expect(i18n.t.sidebar.settings).toBe('Settings');

		i18n.locale = 'de';
		expect(i18n.t.sidebar.settings).toBe('Einstellungen');

		i18n.locale = 'en';
	});

	it('rejects a locale it cannot serve', () => {
		expect(LOCALES.map((l) => l.id)).toEqual(['en', 'de']);
		expect(isLocale('de')).toBe(true);
		expect(isLocale('fr')).toBe(false);
	});

	it('formats counted strings in both languages', () => {
		expect(en.log.count(2, 7)).toBe('2 of 7 entries');
		expect(de.log.count(2, 7)).toBe('2 von 7 Einträgen');
	});
});
