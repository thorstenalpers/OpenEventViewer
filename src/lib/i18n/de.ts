import type { Translations } from './en';

export const de: Translations = {
	sidebar: {
		tagline: 'Windows-Ereignisprotokolle',
		events: 'Ereignisse',
		diagnose: 'Diagnose',
		log: 'Protokoll',
		settings: 'Einstellungen',
		info: 'Info',
		toLight: 'Zum hellen Design wechseln',
		toDark: 'Zum dunklen Design wechseln',
		sections: 'Bereiche',
		collapse: 'Seitenleiste einklappen',
		expand: 'Seitenleiste ausklappen'
	},
	menu: {
		label: 'Menüleiste',
		titles: {
			file: 'Datei',
			view: 'Ansicht',
			help: 'Hilfe'
		},
		items: {
			exit: 'Beenden',
			settings: 'Einstellungen',
			about: 'Über OpenEventViewer'
		}
	},
	common: {
		loading: 'Wird geladen…',
		mockHost: 'Mock-Host — kein Tauri-Backend. Die Daten auf dieser Seite sind Testdaten.'
	},
	events: {
		title: 'Ereignisse',
		subtitle: 'Was Windows aufgezeichnet hat, neueste zuerst.',
		channel: 'Kanal',
		allChannels: 'System und Anwendung',
		from: 'Von',
		to: 'Bis',
		load: 'Laden',
		span: (from: string, to: string) => `${from} bis ${to}`,
		keyword: 'Alle Spalten durchsuchen…',
		columnFilter: 'Spaltenfilter',
		clearColumnFilters: 'Spaltenfilter zurücksetzen',
		loaded: (shown: number, total: number) =>
			shown === total ? `${total} Ereignisse` : `${shown} von ${total} Ereignissen`,
		elapsed: (ms: number) => `in ${ms} ms gelesen`,
		truncated:
			'mehr als die Zeilengrenze — grenz den Filter ein oder erhöh sie in den Einstellungen',
		securityHint:
			'Schließ OpenEventViewer und starte es als Administrator neu, oder wähl einen Kanal, der das nicht braucht.',
		empty: 'Nichts passt.',
		search: 'Im Web nach diesem Ereignis suchen',
		resize: (column: string) => `Spalte ${column} in der Breite ändern`,
		filters: {
			search: 'Suchen…',
			noMatch: 'Nichts passt.',
			clear: 'Diesen Filter zurücksetzen',
			chosen: (count: number) => `${count} ausgewählt`,
			after: (time: string) => `nach ${time}`,
			before: (time: string) => `vor ${time}`,
			timeHint: 'Ortszeit — dieselbe Uhr, die die Tabelle zeigt.',
			numberHint: 'Da stand keine Zahl drin.',
			notUnderstood: (parts: string) => `Nicht verstanden: ${parts}`,
			helpAny: 'eines davon',
			helpCompare: 'größer, kleiner',
			helpRange: 'ein Bereich, Grenzen inklusive',
			helpNot: 'alles außer'
		},
		overTime: 'Zeitlicher Verlauf',
		andMore: (kinds: number, count: number) =>
			`${kinds} weitere Art${kinds === 1 ? '' : 'en'}, zusammen ${count}`,
		bucketSize: (minutes: number) =>
			minutes >= 1440
				? `ein Balken je ${minutes / 1440} Tag${minutes === 1440 ? '' : 'e'}`
				: minutes >= 60
					? `ein Balken je ${minutes / 60} Stunde${minutes === 60 ? '' : 'n'}`
					: `ein Balken je ${minutes} Minute${minutes === 1 ? '' : 'n'}`,
		bucketCount: (total: number, errors: number) => {
			const events = `${total} Ereignis${total === 1 ? '' : 'se'}`;
			return errors === 0 ? events : `${events}, davon ${errors} Fehler`;
		},
		columns: {
			level: 'Stufe',
			time: 'Zeit',
			provider: 'Quelle',
			eventId: 'ID',
			task: 'Aufgabe',
			channel: 'Kanal',
			computer: 'Computer',
			message: 'Meldung'
		}
	},
	diagnose: {
		title: 'Diagnose',
		subtitle:
			'Durchsucht das Protokoll nach den Ereignissen, die ein Rechner schreibt, wenn etwas schiefging, und holt dann die Viertelstunde rund um eines davon.',
		days: (count: number) => (count === 1 ? 'Letzter Tag' : `Letzte ${count} Tage`),
		scan: 'Suchen',
		scanning: 'Sucht…',
		nothing:
			'Nichts gefunden. Such über einen längeren Zeitraum — oder nimm es als gute Nachricht.',
		window: (from: string, to: string) => `${from} — ${to}`,
		inWindow: (count: number) => `${count} Ereignis${count === 1 ? '' : 'se'} im Zeitfenster`,
		kinds: {
			unexpectedShutdown: 'Unerwartetes Herunterfahren',
			bugCheck: 'Bluescreen',
			hardwareError: 'Hardwarefehler',
			appHang: 'Programm hängt',
			appCrash: 'Programmabsturz',
			serviceFailure: 'Dienstfehler',
			diskError: 'Datenträgerfehler',
			ntfs: 'Dateisystem',
			displayTdr: 'Grafiktreiber zurückgesetzt',
			processorPower: 'Prozessor gedrosselt'
		}
	},
	log: {
		title: 'Protokoll',
		subtitle:
			'Was die App getan hat, neueste zuletzt. Nichts davon wird auf die Festplatte geschrieben.',
		filter: 'Meldungen filtern…',
		level: 'Stufe',
		levels: {
			all: 'Alle Stufen',
			error: 'Fehler',
			warning: 'Warnungen',
			info: 'Info',
			debug: 'Debug'
		},
		clear: 'Protokoll leeren',
		empty: 'Noch nichts protokolliert.',
		count: (shown: number, total: number) => `${shown} von ${total} Einträgen`
	},
	info: {
		title: 'Info',
		subtitle: 'Was diese App ist und worauf sie aufbaut.',
		appBody:
			'Die Windows-Ereignisprotokolle lesen und auf das Wesentliche filtern — ohne Konto, ohne Upload, ohne Nutzungsdaten.',
		offline:
			'Alles läuft auf diesem Rechner. Nichts wird hochgeladen, und es werden keine Nutzungsdaten erhoben.',
		appLicense: 'OpenEventViewer steht unter der MIT-Lizenz.',
		thirdParty: 'Komponenten Dritter',
		thirdPartyBody: (total: number, vendored: number, crates: number, npm: number) =>
			`${total} Komponenten werden mit dieser App ausgeliefert: ${vendored} mitgelieferte Binaries, ${crates} Rust-Crates, ${npm} npm-Pakete.`,
		shipped:
			'Die vollständigen Lizenztexte liegen als THIRD_PARTY_LICENSES.txt im Installer. MIT, BSD und ISC verlangen, dass der Hinweis der Binärverteilung beiliegt — ein Link genügt dafür nicht.',
		filter: 'Komponenten filtern…',
		showTexts: 'Lizenztexte anzeigen',
		hideTexts: 'Lizenztexte ausblenden',
		noMatch: 'Keine Komponente passt.',
		redistributed: 'wird mitgeliefert',
		noOwnText: 'ohne eigenen Text',
		withoutText: (count: number) =>
			`${count} Komponenten haben keine eigene Lizenzdatei veröffentlicht; es gilt der kanonische Text der genannten Lizenz.`,
		material: 'Deine Protokolle',
		materialBody:
			'Die Ereignisprotokolle bleiben dort, wo Windows sie führt. Diese App liest sie und schreibt nie hinein.'
	},
	detail: {
		general: 'Allgemein',
		data: 'Ereignisdaten',
		xml: 'XML',
		search: 'Im Web suchen',
		copy: 'Kopieren',
		copied: 'Kopiert',
		close: 'Detailbereich schließen',
		recordId: 'Datensatz',
		keywords: 'Schlüsselwörter',
		noData: 'Dieses Ereignis führt keine eigenen Daten.'
	},
	updater: {
		title: 'Aktualisierungen',
		body: (version: string) => `Version ${version}. Wird einmal beim Start geprüft.`,
		check: 'Jetzt prüfen',
		checking: 'Prüft…',
		upToDate: 'aktuell',
		available: (version: string) => `${version} ist verfügbar`,
		downloading: (percent: number | null) => (percent === null ? 'Lädt…' : `Lädt — ${percent}%`),
		ready: 'Installiert — startet neu',
		install: 'Installieren und neu starten',
		failed: 'Die Aktualisierungsprüfung ist fehlgeschlagen.'
	},
	settings: {
		title: 'Einstellungen',
		appearance: 'Darstellung',
		appearanceBody: 'Das Design des App-Fensters.',
		system: 'System',
		light: 'Hell',
		dark: 'Dunkel',
		colours: 'Farben',
		coloursBody: 'Die Palette, aus der jede Ansicht gezeichnet wird.',
		presets: {
			default: 'Standard',
			caffeine: 'Caffeine',
			'modern-minimal': 'Modern Minimal',
			mono: 'Mono',
			'northern-lights': 'Northern Lights',
			vercel: 'Vercel'
		},
		language: 'Sprache',
		languageBody:
			'Die Oberfläche der App. Der Ereignistext behält die Sprache, in der Windows ihn aufgezeichnet hat.',
		eventsRows: 'Ereignisse: Zeilen pro Abfrage',
		eventsRowsBody:
			'Jedes Ereignis kostet einen Nachschlag beim Herausgeber — eine größere Zahl bedeutet also längeres Warten, nicht nur eine längere Liste.',
		eventsRowsValue: (rows: number) => `${rows.toLocaleString('de')} Zeilen`,
		showLogs: 'Protokoll in der Seitenleiste zeigen',
		showLogsBody: 'Fügt der Navigation einen Eintrag Protokoll hinzu.',
		debugLogging: 'Debug-Einträge aufzeichnen',
		debugLoggingBody:
			'Ausführlich. Standardmäßig aus, weil Debug-Einträge genau die verdrängen, die man gesucht hat.'
	}
};
