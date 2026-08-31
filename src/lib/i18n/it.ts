import type { Translations } from './en';

export const it: Translations = {
	sidebar: {
		tagline: 'Registri eventi di Windows',
		events: 'Eventi',
		diagnose: 'Diagnosi',
		log: 'Registro',
		settings: 'Impostazioni',
		info: 'Info',
		toLight: 'Passa al tema chiaro',
		toDark: 'Passa al tema scuro',
		sections: 'Sezioni',
		collapse: 'Comprimi la barra laterale',
		expand: 'Espandi la barra laterale'
	},
	common: {
		loading: 'Caricamento…',
		mockHost: 'Host fittizio — nessun backend Tauri. I dati in questa pagina sono dati di prova.'
	},
	events: {
		title: 'Eventi',
		subtitle: 'Ciò che Windows ha registrato, dal più recente.',
		channel: 'Canale',
		allChannels: 'Sistema e Applicazione',
		from: 'Da',
		to: 'A',
		load: 'Carica',
		keyword: 'Cerca in tutte le colonne…',
		columnFilter: 'filtro di colonna',
		clearColumnFilters: 'Azzera i filtri di colonna',
		loaded: (shown: number, total: number) =>
			shown === total ? `${total} eventi` : `${shown} di ${total} eventi`,
		elapsed: (ms: number) => `letti in ${ms} ms`,
		truncated: 'più del limite di righe — restringi il filtro o aumentalo nelle Impostazioni',
		securityHint:
			'Chiudi OpenEventViewer e riavvialo come amministratore, oppure scegli un canale che non lo richiede.',
		empty: 'Nessuna corrispondenza.',
		search: 'Cerca questo evento sul web',
		resize: (column: string) => `Ridimensiona la colonna ${column}`,
		filters: {
			search: 'Cerca…',
			noMatch: 'Nessuna corrispondenza.',
			clear: 'Azzera questo filtro',
			chosen: (count: number) => (count === 1 ? `${count} selezionato` : `${count} selezionati`),
			after: (time: string) => `dopo ${time}`,
			before: (time: string) => `prima di ${time}`,
			timeHint: 'Ora locale, lo stesso orologio che mostra la tabella.',
			numberHint: "Lì dentro non c'era nessun numero.",
			notUnderstood: (parts: string) => `Non compreso: ${parts}`,
			helpAny: 'uno qualsiasi',
			helpCompare: 'maggiore, minore',
			helpRange: 'un intervallo, estremi inclusi',
			helpNot: 'tutto tranne'
		},
		overTime: 'Nel tempo',
		andMore: (kinds: number, count: number) =>
			`${kinds} ${kinds === 1 ? 'altro tipo' : 'altri tipi'}, ${count} in totale`,
		bucketSize: (minutes: number) =>
			minutes >= 1440
				? `una barra ogni ${minutes / 1440} giorn${minutes === 1440 ? 'o' : 'i'}`
				: minutes >= 60
					? `una barra ogni ${minutes / 60} or${minutes === 60 ? 'a' : 'e'}`
					: `una barra ogni ${minutes} minut${minutes === 1 ? 'o' : 'i'}`,
		bucketCount: (total: number, errors: number, warnings: number) => {
			const events = `${total} event${total === 1 ? 'o' : 'i'}`;
			const parts = [
				errors > 0 && `${errors} error${errors === 1 ? 'e' : 'i'}`,
				warnings > 0 && `${warnings} avvis${warnings === 1 ? 'o' : 'i'}`
			].filter(Boolean);
			return parts.length === 0 ? events : `${events}, di cui ${parts.join(' e ')}`;
		},
		columns: {
			level: 'Livello',
			time: 'Ora',
			provider: 'Provider',
			eventId: 'ID',
			task: 'Attività',
			channel: 'Canale',
			computer: 'Computer',
			message: 'Messaggio'
		}
	},
	diagnose: {
		title: 'Diagnosi',
		subtitle:
			"Cerca nel registro gli eventi che una macchina scrive quando qualcosa è andato storto, poi estrae il quarto d'ora attorno a uno di essi.",
		days: (count: number) => (count === 1 ? 'Ultimo giorno' : `Ultimi ${count} giorni`),
		scan: 'Analizza',
		scanning: 'Analisi in corso…',
		intro:
			'Non è stato ancora analizzato nulla. Scegli un periodo qui sopra e premi Analizza; ogni risultato — un arresto anomalo, un blocco, un errore del disco, un processore rallentato — appare qui come un incidente che puoi aprire.',
		pick: "Apri un incidente per vedere tutto ciò che la macchina ha scritto nel quarto d'ora attorno ad esso.",
		nothing:
			'Nessun risultato. Analizza un periodo più lungo, oppure prendila come una buona notizia.',
		window: (from: string, to: string) => `${from} — ${to}`,
		inWindow: (count: number) => `${count} event${count === 1 ? 'o' : 'i'} nella finestra`,
		kinds: {
			unexpectedShutdown: 'Arresto imprevisto',
			bugCheck: 'Schermata blu',
			hardwareError: 'Errore hardware',
			appHang: 'Applicazione bloccata',
			appCrash: "Arresto anomalo dell'applicazione",
			serviceFailure: 'Errore del servizio',
			diskError: 'Errore del disco',
			ntfs: 'File system',
			displayTdr: 'Driver video reimpostato',
			processorPower: 'Processore rallentato'
		}
	},
	log: {
		title: 'Registro',
		subtitle:
			"Ciò che l'app ha fatto, dal più recente in fondo. Nulla di questo viene scritto su disco.",
		filter: 'Filtra i messaggi…',
		level: 'Livello',
		levels: {
			all: 'Tutti i livelli',
			error: 'Errori',
			warning: 'Avvisi',
			info: 'Info',
			debug: 'Debug'
		},
		clear: 'Svuota il registro',
		empty: 'Ancora nulla nel registro.',
		count: (shown: number, total: number) => `${shown} di ${total} voci`
	},
	info: {
		title: 'Info',
		subtitle: "Che cos'è questa app e su cosa è costruita.",
		appBody:
			'Leggi i registri eventi di Windows e filtrali fino a ciò che conta — senza account, senza upload, senza telemetria.',
		offline:
			'Tutto gira su questa macchina. Nulla viene caricato e non viene raccolta alcuna telemetria.',
		appLicense: 'OpenEventViewer è rilasciato con licenza MIT.',
		thirdParty: 'Componenti di terze parti',
		thirdPartyBody: (total: number, vendored: number, crates: number, npm: number) =>
			`${total} componenti vengono distribuiti con questa app: ${vendored} binari inclusi, ${crates} crate Rust, ${npm} pacchetti npm.`,
		shipped:
			"I testi completi delle licenze sono inclusi nell'installer come THIRD_PARTY_LICENSES.txt. MIT, BSD e ISC richiedono tutte che l'avviso accompagni il binario, quindi un link non basterebbe.",
		filter: 'Filtra i componenti…',
		showTexts: 'Mostra i testi delle licenze',
		hideTexts: 'Nascondi i testi delle licenze',
		noMatch: 'Nessun componente corrisponde.',
		redistributed: 'distribuito come binario',
		noOwnText: 'senza testo proprio',
		withoutText: (count: number) =>
			`${count} componenti non hanno pubblicato un proprio file di licenza; vale il testo canonico della licenza indicata.`,
		material: 'I tuoi registri',
		materialBody:
			'I registri eventi restano dove Windows li conserva. Questa app li legge e non vi scrive mai.'
	},
	detail: {
		general: 'Generale',
		data: "Dati dell'evento",
		xml: 'XML',
		search: 'Cerca sul web',
		copy: 'Copia',
		copied: 'Copiato',
		close: 'Chiudi il pannello dei dettagli',
		recordId: 'Record',
		keywords: 'Parole chiave',
		noData: 'Questo evento non contiene dati propri.'
	},
	updater: {
		title: 'Aggiornamenti',
		body: (version: string) => `Versione ${version}. Verificata una volta all'avvio.`,
		check: 'Verifica ora',
		checking: 'Verifica in corso…',
		upToDate: 'aggiornata',
		available: (version: string) => `${version} è disponibile`,
		downloading: (percent: number | null) =>
			percent === null ? 'Download in corso…' : `Download in corso — ${percent}%`,
		ready: 'Installato — riavvio in corso',
		install: 'Installa e riavvia',
		failed: 'La verifica degli aggiornamenti non è riuscita.'
	},
	settings: {
		title: 'Impostazioni',
		appearance: 'Aspetto',
		appearanceBody: "Il tema della finestra dell'app.",
		system: 'Sistema',
		light: 'Chiaro',
		dark: 'Scuro',
		colours: 'Colori',
		coloursBody: 'La palette da cui è disegnata ogni vista.',
		presets: {
			default: 'Predefinito',
			caffeine: 'Caffeine',
			catppuccin: 'Catppuccin',
			claude: 'Claude',
			'modern-minimal': 'Modern Minimal',
			mono: 'Mono',
			'northern-lights': 'Northern Lights',
			supabase: 'Supabase',
			tangerine: 'Tangerine',
			twitter: 'Twitter',
			vercel: 'Vercel'
		},
		language: 'Lingua',
		languageBody:
			"L'interfaccia dell'app. Il testo degli eventi mantiene la lingua in cui Windows lo ha registrato.",
		eventsRows: 'Eventi: righe da caricare',
		eventsRowsBody:
			"Ogni evento costa al provider una ricerca del messaggio, quindi un numero più grande significa un'attesa più lunga, non solo una lista più lunga.",
		eventsRowsValue: (rows: number) => `${rows.toLocaleString('it')} righe`,
		showLogs: 'Mostra il registro nella barra laterale',
		showLogsBody: 'Aggiunge una voce Registro alla navigazione.',
		debugLogging: 'Registra le voci di debug',
		debugLoggingBody:
			'Prolisso. Disattivato per impostazione predefinita, perché le voci di debug soffocano proprio quelle che stavi cercando.'
	}
};
