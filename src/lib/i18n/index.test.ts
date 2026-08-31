import { describe, expect, it } from 'vitest';
import { ar } from './ar';
import { de } from './de';
import { en } from './en';
import { es } from './es';
import { fr } from './fr';
import { hi } from './hi';
import { it as italian } from './it';
import { ja } from './ja';
import { pt } from './pt';
import { ru } from './ru';
import { zh } from './zh';
import { i18n, isLocale, LOCALES } from './index.svelte';

const TRANSLATED = { ar, de, es, fr, hi, it: italian, ja, pt, ru, zh } as const;

/** Walks both dictionaries in step so a key added to one and forgotten in the other is a failure. */
function compare(left: unknown, right: unknown, path: string, problems: string[]): void {
	if (typeof left === 'function') {
		if (typeof right !== 'function') problems.push(`${path}: not a function`);
		return;
	}
	if (typeof left === 'string') {
		if (typeof right !== 'string') problems.push(`${path}: not a string`);
		else if (right.trim() === '') problems.push(`${path}: empty`);
		return;
	}
	if (left && typeof left === 'object') {
		const a = left as Record<string, unknown>;
		const b = (right ?? {}) as Record<string, unknown>;
		for (const key of Object.keys(a)) {
			if (!(key in b)) problems.push(`${path}.${key}: missing`);
			else compare(a[key], b[key], `${path}.${key}`, problems);
		}
		for (const key of Object.keys(b)) {
			if (!(key in a)) problems.push(`${path}.${key}: extra`);
		}
	}
}

describe('i18n', () => {
	it('has an entry in every language for every English one and nothing extra', () => {
		const problems: string[] = [];
		for (const [name, dictionary] of Object.entries(TRANSLATED)) {
			compare(en, dictionary, name, problems);
		}

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
		expect(LOCALES.map((l) => l.id)).toEqual([
			'en',
			'ar',
			'de',
			'es',
			'fr',
			'hi',
			'it',
			'ja',
			'pt',
			'ru',
			'zh'
		]);
		expect(isLocale('de')).toBe(true);
		expect(isLocale('tlh')).toBe(false);
	});

	it('formats counted strings in both languages', () => {
		expect(en.log.count(2, 7)).toBe('2 of 7 entries');
		expect(de.log.count(2, 7)).toBe('2 von 7 Einträgen');
	});
});
