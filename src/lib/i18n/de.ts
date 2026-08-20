import type { Translations } from './en';

export const de: Translations = {
	sidebar: {
		overview: 'Übersicht',
		log: 'Protokoll',
		projects: 'Meine Examen',
		review: 'Prüfen',
		study: 'Lernen',
		notes: 'Notizen',
		train: 'Üben',
		browse: 'Browser',
		media: 'Medien',
		catalog: 'Katalog',
		stats: 'Statistik',
		settings: 'Einstellungen',
		info: 'Info',
		toLight: 'Zum hellen Design wechseln',
		toDark: 'Zum dunklen Design wechseln',
		sections: 'Bereiche',
		noProject: 'Kein Projekt geladen',
		collapse: 'Seitenleiste einklappen',
		expand: 'Seitenleiste ausklappen'
	},
	common: {
		back: 'Zurück',
		loading: 'Wird geladen…',
		noBinder: 'Kein Exam ausgewählt.',
		noBinderBody:
			'Diese Seite dreht sich um ein Exam. Leg eins an oder wähl eins aus der Liste, dann füllt sie sich.',
		toProjects: 'Zu Meine Examen',
		mockHost: 'Mock-Host — kein Tauri-Backend. Die Daten auf dieser Seite sind Testdaten.'
	},
	dashboard: {
		title: 'Übersicht',
		subtitle: 'Wo du über alle Projekte hinweg stehst.',
		projects: 'Meine Examen',
		questions: 'Fragen',
		dueToday: 'Heute fällig',
		weak: 'Schwach',
		progress: 'Fortschritt',
		accuracy: 'Trefferquote',
		accuracyValue: (percent: number) => `${percent}% aller Antworten waren richtig.`,
		answeredOf: (answered: number, total: number) =>
			`${answered} von ${total} Fragen mindestens einmal beantwortet.`,
		nothingAnswered: 'Noch nichts beantwortet — die Zahlen erscheinen nach der ersten Session.',
		startDue: (count: number) => `${count} fällige lernen`,
		startWeak: (count: number) => `${count} schwache üben`,
		createFirst: 'Erstes Projekt anlegen',
		recent: 'Letzte Sessions',
		noSessions: 'Noch keine abgeschlossene Session.',
		modes: {
			practice: 'Üben',
			focus: 'Fokus',
			due: 'Fällig',
			weak: 'Schwach',
			exam: 'Prüfung',
			challenge: 'Challenge'
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
	projects: {
		title: 'Meine Examen',
		subtitle: (count: number) => `${count} Exam${count === 1 ? '' : 'en'} auf diesem Rechner.`,
		create: 'Neues Exam',
		template: 'Exam',
		ownExam: 'Nicht in der Liste',
		docUrl: 'Dokumentationsseite',
		templateHint:
			'Code und Dokumentationsseite zusammen identifizieren ein Exam. Ein hier getippter Code wird als Vorlage gemerkt.',
		code: 'Zertifizierung',
		name: 'Name',
		namePlaceholder: 'Optional — sonst der Code',
		save: 'Anlegen',
		created: (code: string) => `${code} angelegt. Jetzt eine Datei hineinimportieren.`,
		importDeck: 'Projektdatei importieren',
		filter: 'Projekte filtern…',
		multiSortHint: 'Zweite Spalte mit Umschalt-Klick dazunehmen.',
		empty: 'Noch keine Projekte.',
		noFile: 'noch keine Datei',
		addFile: 'Datei hinzufügen',
		train: 'Lernen',
		exportAria: (title: string) => `${title} exportieren`,
		deleteAria: (title: string) => `${title} löschen`,
		exported: (count: number, path: string) => `${count} Fragen nach ${path} exportiert`,
		imported: (title: string, count: number) => `${title} importiert — ${count} Fragen`,
		columns: {
			project: 'Projekt',
			certification: 'Zertifizierung',
			questions: 'Fragen',
			review: 'Prüfen',
			attempts: 'Versuche',
			accuracy: 'Trefferquote',
			created: 'Angelegt',
			lastStudied: 'Zuletzt gelernt',
			actions: 'Aktionen'
		}
	},
	info: {
		title: 'Info',
		subtitle: 'Was diese App ist und worauf sie aufbaut.',
		appBody:
			'Zertifizierungsmaterial importieren, abfragen und jede falsche Antwort in die nächste Session überführen.',
		offline:
			'Alles läuft auf diesem Rechner. Nichts wird hochgeladen, und es werden keine Nutzungsdaten erhoben.',
		appLicense: 'OpenExamTrainer steht unter der MIT-Lizenz.',
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
		material: 'Dein Material',
		materialBody:
			'Importiertes Prüfungsmaterial bleibt deins und bleibt auf diesem Rechner. Sein Urheberrecht wird von dieser App nicht berührt, und nichts hier erlaubt eine Weitergabe.'
	},
	import: {
		choose: 'Datei wählen',
		extracting: 'Wird extrahiert…',
		hint: 'Die Extraktion meldet, was sie tatsächlich gefunden hat. Ein Dump, der mit 250 Fragen wirbt und 11 enthält, wird als 11 gemeldet.',
		profile: 'Profil',
		meta: (pages: number, furniture: number) =>
			`${pages} Seiten · ${furniture} Kopf- und Fußzeilen entfernt`,
		questions: 'Fragen',
		needReview: 'Zu prüfen',
		missingFigure: 'Ohne Abbildung',
		figuresRecovered: 'Gerettete Abbildungen',
		excerpt: (markers: number[]) =>
			`Diese Datei ist ein Auszug. Marker ${markers.join(', ')} enthielt Werbung statt einer Frage.`,
		skips: (numbers: number[]) =>
			`Die Quelle überspringt die Fragennummer(n) ${numbers.join(', ')}.`,
		startTraining: 'Lernen starten',
		review: (count: number) => `${count} prüfen`
	},
	exam: {
		subtitle: (started: string) => `Begonnen am ${started}.`,
		studyGuide: 'Study Guide',
		checklist: 'Checkliste',
		derived: 'Das weiß die App selbst — der Haken setzt sich von allein.',
		steps: {
			create: 'Exam anlegen',
			intro: 'Einführung ansehen',
			study: 'Material durcharbeiten',
			notes: 'Eigene Notizen schreiben',
			train: 'Fragen üben',
			pass: 'Exam bestehen'
		},
		certifications: 'Bestanden',
		certificationsBody:
			'Jedes Mal, wenn dieses Exam bestanden wurde — eine Zertifizierung läuft ab und wird erneuert.',
		noCertifications: 'Noch nicht bestanden.',
		passedCount: 'Bestanden',
		passedOn: 'Bestanden am',
		note: 'Notiz',
		notePlaceholder: 'Optional — Punkte, Versuch, was erinnernswert ist',
		addDate: 'Hinzufügen',
		removeDate: (date: string) => `${date} entfernen`
	},
	study: {
		title: 'Lernen',
		subtitle: (binder: string) => `Kurse, Videos und Dokumentation für ${binder}.`,
		all: 'Alles',
		kinds: {
			course: 'Kurs',
			video: 'Video',
			docs: 'Dokumentation',
			other: 'Sonstiges'
		},
		empty: 'Noch nichts da — unten einen Kurs, ein Video oder eine Seite hinzufügen.',
		open: 'Öffnen',
		play: 'Abspielen',
		close: 'Schließen',
		removeAria: (title: string) => `${title} entfernen`,
		minutes: (count: number) => `${count} Min.`,
		hours: (hours: number, minutes: number) => `${hours} Std. ${minutes} Min.`,
		totalTime: (span: string) => `${span} insgesamt`,
		addTitle: 'Material hinzufügen',
		addBody: 'Ein Link bleibt ein Link; ein Video von diesem Rechner wird hier abgespielt.',
		url: 'Adresse',
		linkTitle: 'Titel',
		kind: 'Art',
		minutesLabel: 'Minuten',
		description: 'Beschreibung',
		descriptionPlaceholder: 'Worum es geht, in deinen Worten',
		add: 'Hinzufügen',
		addVideo: 'Videodatei hinzufügen',
		noPlayback: 'Abspielen geht nur in der App — ein Browser öffnet keine Datei von deiner Platte.'
	},
	notes: {
		title: 'Notizen',
		subtitle: (binder: string) => `Was du dir zu ${binder} aufgeschrieben hast.`,
		ownTitle: 'Deine Notizen',
		ownBody: 'Gelten für das ganze Exam. Notizen zu einer einzelnen Frage stehen bei dieser Frage.',
		empty: 'Noch nichts geschrieben.',
		placeholder: 'Was du dir merken willst …',
		save: 'Notiz speichern',
		none: 'Zu dieser Frage gibt es noch keine Notiz.',
		saveAnswer: 'Als Notiz speichern',
		saved: 'In der Mappe gespeichert.',
		workshopTitle: 'Aus deinen Notizen gemacht',
		workshopBody:
			'Der Assistent schreibt deine Notizen als Zusammenfassung; die Zusammenfassung kann danach als Folge vorgelesen werden. Beides sind Dateien, die du löschen kannst.',
		summarise: 'Zusammenfassung schreiben',
		summarising: 'Wird zusammengefasst …',
		asPodcast: 'Als Podcast',
		recording: 'Wird aufgenommen …',
		asPdf: 'Als PDF',
		typesetting: 'Wird gesetzt …',
		noArtefacts: 'Noch nichts erzeugt.',
		removeAria: (name: string) => `${name} löschen`,
		assistantNote:
			'Die Zusammenfassung schreibt der Assistent, der in den Einstellungen gewählt ist — die lokale Binary schickt nichts an Dritte.'
	},
	timeline: {
		title: 'Zeitachse',
		subtitle: 'Wann jedes Exam begonnen hat und wann es bestanden wurde.',
		started: 'Begonnen',
		passed: 'Bestanden',
		empty: 'Noch keine Examen — die Achse füllt sich, sobald du welche anlegst.'
	},
	question: {
		figureAlt: (index: number) => `Aus der Quellseite gerettete Abbildung ${index}`,
		figureUnavailable: 'Die Abbildung dieser Frage liegt nicht mehr auf der Festplatte.'
	},
	review: {
		title: 'Prüfen',
		subtitle: (count: number, binder: string) =>
			`${count} Frage${count === 1 ? '' : 'n'} in ${binder}, bei denen die Extraktion unsicher war.`,
		clean: 'Nichts zu prüfen — jede Frage liegt über der Konfidenzschwelle.',
		page: (page: number) => `Seite ${page}`,
		confidence: (value: string) => `Konfidenz ${value}`,
		needsSource:
			'Die eigentliche Frage ist eine Abbildung, die die Extraktion nicht aus der Quellseite retten konnte. Nur der Antwortschlüssel ist übrig, deshalb ist diese Frage von gewerteten Sessions ausgeschlossen.',
		matrixKey: 'Aus der Erklärung geretteter Antwortschlüssel',
		backToTraining: 'Zurück zum Lernen',
		warnings: {
			number_out_of_sequence: 'Fragennummer außer der Reihe',
			option_letters_not_sequential: 'Antwortbuchstaben nicht fortlaufend',
			answer_without_option: 'Antwortbuchstabe ohne passende Option',
			missing_answer: 'Keine Antwort gefunden',
			stem_too_short: 'Fragetext verdächtig kurz',
			figure_missing: 'Abbildung fehlt in der Quelle'
		}
	},
	train: {
		noBinderTitle: 'Keine Mappe ausgewählt',
		noBinderBody: 'Importiere zuerst eine Datei oder wähle eine Mappe in der Übersicht.',
		score: (correct: number, total: number) => `${correct} von ${total} richtig`,
		summaryMeta: (seconds: number, wrong: number) => `${seconds} Sekunden · ${wrong} zu üben`,
		sessionScore: 'Ergebnis der Session',
		sessionProgress: 'Fortschritt der Session',
		startFocus: (count: number) => `Fokus-Session starten (${count})`,
		nothingMissed: 'Nichts verpasst — es gibt keinen Fokus-Satz.',
		binderMeta: (questions: number, excluded: number) =>
			`${questions} Fragen · ${excluded} ausgeschlossen, weil die Abbildung fehlt`,
		practice: 'Üben',
		dueToday: 'Heute fällig',
		weak: 'Schwache Fragen',
		exam: 'Prüfung',
		challengeTitle: 'Challenge',
		challengeBody:
			'Ein Seed legt Fragen und Reihenfolge fest — zwei Läufe sind dieselbe Prüfung, und die Zeiten sind vergleichbar.',
		seed: 'Seed',
		questions: 'Fragen',
		minutes: 'Minuten',
		takeChallenge: 'Challenge starten',
		noRuns: (seed: number) => `Noch keine Läufe auf Seed ${seed}.`,
		postResult: 'In die Katalog-Bestenliste eintragen',
		postedAt: (place: number, total: number) =>
			`Eingetragen — Platz ${place} von ${total} auf dieser Liste.`,
		publishToPost:
			'Veröffentliche diese Mappe im Katalog, dann kann ein Lauf darauf in eine Bestenliste, an der sich andere messen lassen.',
		chooseN: (count: number) => `Wähle ${count}`,
		noFeedback: 'Rückmeldung erst am Ende',
		correct: 'Richtig.',
		notCorrect: 'Nicht richtig.',
		next: 'Nächste Frage',
		finish: 'Abschließen',
		check: 'Prüfen',
		answerAndContinue: 'Antworten und weiter'
	},
	browse: {
		back: 'Zurück',
		forward: 'Vor',
		reload: 'Neu laden',
		address: 'Adresse',
		go: 'Los',
		portals: {
			learn: 'Microsoft Learn',
			azure: 'Azure-Dokumentation',
			credentials: 'Zertifizierungskatalog',
			youtube: 'YouTube'
		},
		mockNote:
			'Der eingebettete Browser ist ein zweites WebView2-Kind des App-Fensters. In einem normalen Browser gibt es das nicht, deshalb bleibt dieser Bereich unter dem Mock-Host leer — das gemessene Rechteck wird trotzdem an den Host gemeldet.'
	},
	media: {
		title: 'Medien',
		subtitle: (binder: string) => `Videos und Audio für ${binder}.`,
		videosTitle: 'Videos',
		videosBody:
			'Ein YouTube-Link oder eine beliebige URL, wahlweise mit Startzeit an eine Frage gehängt.',
		colTitle: 'Titel',
		colUrl: 'URL',
		colStart: 'Beginnt bei',
		add: 'Hinzufügen',
		anchorTo: 'An Frage hängen',
		wholeBinder: 'die ganze Mappe',
		noVideos: 'Noch keine Videos.',
		open: 'Öffnen',
		removeAria: (title: string) => `${title} entfernen`,
		from: (time: string) => `ab ${time}`,
		podcastTitle: 'Podcast',
		podcastBody:
			'Vorgelesen auf diesem Rechner, offline. Kein Key, kein Konto, nichts verlässt ihn. Welche Stimme liest, steht in den Einstellungen.',
		readAnswer: 'Antwort vorlesen',
		readExplanation: 'Erklärung vorlesen',
		pause: 'Pause',
		format: 'Format',
		formatMp3: 'MP3 — klein genug zum Mitnehmen',
		formatWav: 'WAV — was die Stimme erzeugt hat',
		spokenLanguage: 'Gesprochene Sprache',
		spokenEn: 'Englisch',
		spokenDe: 'Deutsch',
		spokenLanguageBody:
			'Wählt die Worte rund um die Frage und die Windows-Stimme, solange in den Einstellungen kein Stimmpaket gewählt ist. Die Frage selbst wird in der Sprache gelesen, in der du sie importiert hast — übersetzt wird sie nie.',
		seconds: 'Sekunden',
		recallOnly: 'Nur Fragen und Stille — reines Abrufen.',
		record: 'Folge aufnehmen',
		recording: 'Wird aufgenommen…',
		episodeMeta: (duration: string, chapters: number) => `${duration} · ${chapters} Kapitel`
	},
	stats: {
		title: 'Statistik',
		subtitle: (binder: string) => `Jede Zahl unten stammt aus dem Versuchsprotokoll von ${binder}.`,
		empty: 'Noch nichts beantwortet — die Zahlen erscheinen nach der ersten Session.',
		byTopic: 'Nach Thema',
		byQuestion: 'Nach Frage',
		topic: 'Thema',
		noTopic: 'Ohne Thema',
		question: 'Frage',
		questionCount: (count: number) => `${count} Frage${count === 1 ? '' : 'n'}`,
		attempts: 'Versuche',
		accuracy: 'Trefferquote',
		averageTime: 'Ø Zeit',
		lapses: 'Rückfälle',
		due: 'Fällig',
		neverAnswered: 'nie beantwortet',
		excluded: 'ausgeschlossen'
	},
	assistant: {
		title: 'Assistent',
		explain: 'Warum diese Antwort?',
		variants: 'Varianten erzeugen',
		note: 'In eine Notiz verwandeln',
		thinking: 'Denkt nach…',
		sourceCli: 'lokales claude',
		sourceAnthropic: 'anthropic',
		noCli:
			'Die lokale claude-Binary liegt nicht im PATH. Installiere Claude Code oder wähle in den Einstellungen einen gehosteten Anbieter.',
		noKey: 'Kein API-Key hinterlegt. Trag in den Einstellungen einen ein.'
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
			'Die Oberfläche der App. Der Fragetext behält die Sprache der importierten Quelle.',
		voice: 'Stimme',
		voiceBody: 'Wer die Podcast-Folgen vorliest.',
		voiceWindows: 'Windows-Stimme',
		voicePreview: 'Probehören',
		voiceStopPreview: 'Stopp',
		voiceSample: 'Franz jagt im komplett verwahrlosten Taxi quer durch Bayern.',
		voicePacks: 'Stimmpakete',
		voicePacksBody:
			'Einmal geladen, danach offline — anders als die Windows-Stimmen, die pro Sprache installiert werden und selten eine deutsche dabeihaben.',
		voiceSize: (megabytes: number) => `${megabytes} MB Download`,
		voiceInstalled: (count: number) => `installiert · ${count} Stimme${count === 1 ? '' : 'n'}`,
		voiceDownload: 'Herunterladen',
		voiceDownloading: 'Lädt …',
		voiceUnpacking: 'Wird entpackt — das dauert einige Minuten',
		voiceCancel: 'Abbrechen',
		voiceRemove: 'Entfernen',
		showLogs: 'Protokoll in der Seitenleiste zeigen',
		showLogsBody: 'Fügt der Navigation einen Eintrag Protokoll hinzu.',
		debugLogging: 'Debug-Einträge aufzeichnen',
		debugLoggingBody:
			'Ausführlich. Standardmäßig aus, weil Debug-Einträge genau die verdrängen, die man gesucht hat.',
		assistant: 'Assistent',
		assistantBody: 'Wohin die Erklär- und Varianten-Knöpfe ihre Frage schicken.',
		sourceCliLabel: 'Lokale claude-Binary',
		sourceCliDetail: 'Läuft auf diesem Rechner. Diese App schickt nichts an Dritte.',
		sourceAnthropicLabel: 'Anthropic-API',
		sourceAnthropicDetail:
			'Die Frage und ihre Erklärung gehen an api.anthropic.com, sobald du einen Knopf drückst.',
		found: 'gefunden',
		keyStored: 'Key hinterlegt',
		apiKey: 'API-Key',
		store: 'Speichern',
		keyNote:
			'Der Key landet im Windows-Anmeldeinformationsmanager, nie in einer Datei dieser App, und kann in dieses Fenster nicht zurückgelesen werden.',
		stored: 'Im Windows-Anmeldeinformationsmanager hinterlegt.'
	},
	catalog: {
		title: 'Katalog',
		subtitle: 'Was von diesem Rechner veröffentlicht wurde — und was sich davon zurückholen lässt.',
		localNote:
			'Der Katalog ist eine Datei auf dieser Platte, kein Server. Veröffentlichen kopiert das Deck in den eigenen Ordner der App und schreibt eine Zeile in catalog.sqlite3 — nichts verlässt diesen Rechner, und noch sieht es niemand sonst.',
		publishedAs: 'Veröffentlicht als',
		rename: 'Umbenennen',
		namePlaceholder: 'Name, unter dem veröffentlicht wird',
		search: 'Suche',
		searchPlaceholder: 'Titel oder Zertifizierung …',
		sort: 'Sortierung',
		sortRecent: 'Neueste zuerst',
		sortRating: 'Beste Bewertung',
		sortQuestions: 'Meiste Fragen',
		sortTitle: 'Nach Titel',
		empty: 'Noch nichts veröffentlicht.',
		noMatch: 'Dazu passt nichts.',
		mine: 'von dir',
		by: (owner: string) => `von ${owner}`,
		questions: (count: number) => `${count} Frage${count === 1 ? '' : 'n'}`,
		needsSource: (count: number) => `${count} ohne Quelle`,
		noRating: 'nicht bewertet',
		ratingOf: (average: number, count: number) =>
			`${average.toFixed(1)} aus ${count} Bewertung${count === 1 ? '' : 'en'}`,
		publishTitle: 'Projekt veröffentlichen',
		publishBody:
			'Erneutes Veröffentlichen ersetzt den vorhandenen Eintrag, statt eine Kopie danebenzulegen.',
		project: 'Projekt',
		review: 'Zeigen, was veröffentlicht würde',
		previewTitle: 'Was veröffentlicht würde',
		previewBody:
			'Am tatsächlich geschriebenen Deck gemessen, nicht aus den Tabellen zusammengezählt — eine anders zusammengestellte Vorschau ist die Vorschau von etwas anderem.',
		links: 'Links',
		videos: 'Videos',
		notes: 'Notizen',
		figures: 'Abbildungen',
		sourceExcluded: 'Das importierte PDF bleibt hier: ein Deck trägt keinen sources-Ordner.',
		confirm: 'Veröffentlichen',
		cancel: 'Abbrechen',
		import: 'Importieren',
		importing: 'Wird importiert …',
		imported: (title: string) => `${title} liegt in der Bibliothek.`,
		published: (title: string) => `${title} steht im Katalog.`,
		withdraw: 'Zurückziehen',
		rate: 'Bewerten',
		rating: 'Deine Bewertung',
		comment: 'Kommentar (optional)',
		ratings: 'Bewertungen',
		noRatings: 'Bisher hat das niemand bewertet.',
		board: 'Bestenliste',
		noBoard: 'Für diese Mappe wurde noch kein Lauf eingetragen.',
		seed: 'Seed',
		boardRow: (correct: number, total: number) => `${correct} von ${total}`,
		syncTitle: 'Fortschritt',
		syncBody:
			'Eine Frage wird über Rechner hinweg durch einen Hash aus Fragetext und Antwortschlüssel zugeordnet, nie über ihre Zeilen-ID — IDs sind lokal, und derselbe Dump zweimal importiert nummeriert seine Zeilen anders. Nur der Zeitplan wandert: das Antwortprotokoll ist reines Anhängen, es aufzufüllen, damit zwei Zähler übereinstimmen, wäre eine gefälschte Historie.',
		push: 'Hochschieben',
		pull: 'Herunterholen',
		syncResult: (pushed: number, pulled: number, skipped: number) =>
			`${pushed} hoch, ${pulled} herunter, ${skipped} unangetastet.`,
		oneMachine:
			'Mit einem Rechner ist das ein Rundlauf zu sich selbst. Belegt wird die Zuordnung, nicht das Netz.'
	}
};
