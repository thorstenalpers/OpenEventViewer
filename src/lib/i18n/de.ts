import type { Translations } from './en';

export const de: Translations = {
	sidebar: {
		tagline: 'Windows-Ereignisprotokolle',
		events: 'Ereignisse',
		assistant: 'Assistent',
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
	common: {
		loading: 'Wird geladen…',
		mockHost: 'Mock-Host — kein Tauri-Backend. Die Daten auf dieser Seite sind Testdaten.'
	},
	events: {
		title: 'Ereignisse',
		subtitle: 'Was Windows aufgezeichnet hat, neueste zuerst.',
		channel: 'Kanal',
		allChannels: 'System und Anwendung',
		level: 'Stufe',
		levels: {
			critical: 'Kritisch',
			error: 'Fehler',
			warning: 'Warnung',
			information: 'Information',
			verbose: 'Ausführlich'
		},
		range: 'Zeitraum',
		ranges: {
			hour: 'Letzte Stunde',
			day: 'Letzte 24 Stunden',
			week: 'Letzte 7 Tage',
			custom: 'Eigener Zeitraum'
		},
		from: 'Von',
		to: 'Bis',
		eventIds: 'Ereignis-IDs',
		providers: 'Quellen',
		providersHint: 'Exakte Namen, mit Komma getrennt',
		load: 'Laden',
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
		ask: 'Den Assistenten zu diesem Ereignis fragen',
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
		previewBundle: 'Was der Assistent bekommen würde',
		send: 'An den Assistenten schicken',
		question: 'Was ist hier passiert, und was sollte ich als Nächstes prüfen?',
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
		refresh: 'Aktualisieren',
		includeWeb: 'Web-Konsole einbeziehen',
		includeWebBody:
			'Kopiert die Konsole des Webviews in dieses Protokoll, damit Oberfläche und Host eine gemeinsame Zeitleiste haben.',
		clear: 'Protokoll leeren',
		empty: 'Noch nichts protokolliert.',
		count: (shown: number, total: number) => `${shown} von ${total} Einträgen`
	},
	info: {
		title: 'Info',
		subtitle: 'Was diese App ist und worauf sie aufbaut.',
		appBody:
			'Die Windows-Ereignisprotokolle lesen, auf das Wesentliche filtern und einen Assistenten fragen, was eine Folge davon bedeutet.',
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
	assistant: {
		title: 'Assistent',
		thinking: 'Denkt nach…',
		sourceCli: 'lokales claude',
		sourceAnthropic: 'anthropic',
		noCli:
			'Die lokale claude-Binary liegt nicht im PATH. Installiere Claude Code oder wähle in den Einstellungen einen gehosteten Anbieter.',
		noKey: 'Kein API-Key hinterlegt. Trag in den Einstellungen einen ein.',
		empty: 'Häng ein Ereignis aus der Ereignisliste an oder frag einfach etwas.',
		placeholder: 'Frag nach den angehängten Ereignissen…',
		send: 'Senden',
		newConversation: 'Neues Gespräch',
		preview: 'Was gesendet wird',
		previewBody:
			'Genau dieser Text verlässt den Rechner, wenn du auf Senden drückst — danach kommt nichts mehr dazu.',
		systemPrompt: 'Feste Anweisungen',
		nextMessage: 'Deine nächste Nachricht',
		nothingYet: 'Noch nichts zu senden.',
		characters: (count: number) => `${count.toLocaleString('de')} Zeichen`,
		attachedCount: (count: number) => `${count} Ereignis${count === 1 ? '' : 'se'}`,
		removeAttachment: (title: string) => `${title} entfernen`
	},
	detail: {
		general: 'Allgemein',
		data: 'Ereignisdaten',
		xml: 'XML',
		ask: 'Fragen',
		copy: 'Kopieren',
		copied: 'Kopiert',
		close: 'Detailbereich schließen',
		recordId: 'Datensatz',
		keywords: 'Schlüsselwörter',
		noData: 'Dieses Ereignis führt keine eigenen Daten.'
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
			'Ausführlich. Standardmäßig aus, weil Debug-Einträge genau die verdrängen, die man gesucht hat.',
		assistant: 'Assistent',
		assistantBody: 'Wohin der Assistent schickt, was du ihn fragst.',
		sourceCliLabel: 'Lokale claude-Binary',
		sourceCliDetail: 'Läuft auf diesem Rechner. Diese App schickt nichts an Dritte.',
		sourceAnthropicLabel: 'Anthropic-API',
		sourceAnthropicDetail:
			'Was die Vorschau zeigt, geht an api.anthropic.com, sobald du auf Senden drückst.',
		found: 'gefunden',
		keyStored: 'Key hinterlegt',
		apiKey: 'API-Key',
		store: 'Speichern',
		keyNote:
			'Der Key landet im Windows-Anmeldeinformationsmanager, nie in einer Datei dieser App, und kann in dieses Fenster nicht zurückgelesen werden.',
		stored: 'Im Windows-Anmeldeinformationsmanager hinterlegt.'
	}
};
