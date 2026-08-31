import { ar } from './ar';
import { de } from './de';
import { en, type Translations } from './en';
import { es } from './es';
import { fr } from './fr';
import { hi } from './hi';
import { it } from './it';
import { ja } from './ja';
import { pt } from './pt';
import { ru } from './ru';
import { zh } from './zh';

/** English first, then by endonym — each language names itself, so nothing here is translated. */
export const LOCALES = [
	{ id: 'en', label: 'English' },
	{ id: 'ar', label: 'العربية' },
	{ id: 'de', label: 'Deutsch' },
	{ id: 'es', label: 'Español' },
	{ id: 'fr', label: 'Français' },
	{ id: 'hi', label: 'हिन्दी' },
	{ id: 'it', label: 'Italiano' },
	{ id: 'ja', label: '日本語' },
	{ id: 'pt', label: 'Português' },
	{ id: 'ru', label: 'Русский' },
	{ id: 'zh', label: '中文' }
] as const;

export type Locale = (typeof LOCALES)[number]['id'];

const DICTIONARIES: Record<Locale, Translations> = {
	en,
	ar,
	de,
	es,
	fr,
	hi,
	it,
	ja,
	pt,
	ru,
	zh
};

/** Scripts that run right to left. The whole shell mirrors for these. */
const RTL: ReadonlySet<Locale> = new Set<Locale>(['ar']);

export function isLocale(value: string): value is Locale {
	return value in DICTIONARIES;
}

class I18n {
	locale = $state<Locale>('en');

	get t(): Translations {
		return DICTIONARIES[this.locale];
	}

	/** `lang` for screen readers and hyphenation, `dir` so Arabic mirrors the whole shell. */
	applyToDocument(): void {
		if (typeof document === 'undefined') return;
		document.documentElement.lang = this.locale;
		document.documentElement.dir = RTL.has(this.locale) ? 'rtl' : 'ltr';
	}
}

export const i18n = new I18n();
