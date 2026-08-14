import { de } from './de';
import { en, type Translations } from './en';

export const LOCALES = [
	{ id: 'en', label: 'English' },
	{ id: 'de', label: 'Deutsch' }
] as const;

export type Locale = (typeof LOCALES)[number]['id'];

const DICTIONARIES: Record<Locale, Translations> = { en, de };

export function isLocale(value: string): value is Locale {
	return value in DICTIONARIES;
}

/**
 * The interface language. Question text is never translated — it belongs to the source the user
 * imported, and a translated exam question would be a different question.
 */
class I18n {
	locale = $state<Locale>('en');

	get t(): Translations {
		return DICTIONARIES[this.locale];
	}
}

export const i18n = new I18n();
