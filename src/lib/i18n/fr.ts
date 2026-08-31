import type { Translations } from './en';

export const fr: Translations = {
	sidebar: {
		tagline: 'Journaux d’événements Windows',
		events: 'Événements',
		diagnose: 'Diagnostic',
		log: 'Journal',
		settings: 'Paramètres',
		info: 'Infos',
		toLight: 'Passer au thème clair',
		toDark: 'Passer au thème sombre',
		sections: 'Sections',
		collapse: 'Replier la barre latérale',
		expand: 'Déplier la barre latérale'
	},
	common: {
		loading: 'Chargement…',
		mockHost:
			'Hôte simulé — pas de backend Tauri. Les données de cette page sont des données de test.'
	},
	events: {
		title: 'Événements',
		subtitle: 'Ce que Windows a enregistré, du plus récent au plus ancien.',
		channel: 'Canal',
		allChannels: 'Système et Application',
		from: 'Du',
		to: 'Au',
		load: 'Charger',
		keyword: 'Rechercher dans toutes les colonnes…',
		columnFilter: 'filtre de colonne',
		clearColumnFilters: 'Effacer les filtres de colonnes',
		loaded: (shown: number, total: number) =>
			shown === total ? `${total} événements` : `${shown} sur ${total} événements`,
		elapsed: (ms: number) => `lus en ${ms} ms`,
		truncated:
			'plus que la limite de lignes — affinez le filtre ou augmentez-la dans les Paramètres',
		securityHint:
			'Fermez OpenEventViewer et relancez-le en tant qu’administrateur, ou choisissez un canal qui ne l’exige pas.',
		empty: 'Aucun résultat.',
		search: 'Rechercher cet événement sur le web',
		resize: (column: string) => `Redimensionner la colonne ${column}`,
		filters: {
			search: 'Rechercher…',
			noMatch: 'Aucun résultat.',
			clear: 'Effacer ce filtre',
			chosen: (count: number) => (count === 1 ? `${count} choisi` : `${count} choisis`),
			after: (time: string) => `après ${time}`,
			before: (time: string) => `avant ${time}`,
			timeHint: 'Heure locale, la même horloge que celle du tableau.',
			numberHint: 'Rien là-dedans n’était un nombre.',
			notUnderstood: (parts: string) => `Non compris : ${parts}`,
			helpAny: 'l’un d’entre eux',
			helpCompare: 'au-dessus, en dessous',
			helpRange: 'une plage, bornes incluses',
			helpNot: 'tout sauf'
		},
		overTime: 'Au fil du temps',
		andMore: (kinds: number, count: number) =>
			`${kinds} autre${kinds === 1 ? '' : 's'} type${kinds === 1 ? '' : 's'}, ${count} au total`,
		bucketSize: (minutes: number) =>
			minutes >= 1440
				? `une barre par ${minutes / 1440} jour${minutes === 1440 ? '' : 's'}`
				: minutes >= 60
					? `une barre par ${minutes / 60} heure${minutes === 60 ? '' : 's'}`
					: `une barre par ${minutes} minute${minutes === 1 ? '' : 's'}`,
		bucketCount: (total: number, errors: number, warnings: number) => {
			const events = `${total} événement${total === 1 ? '' : 's'}`;
			const parts = [
				errors > 0 && `${errors} erreur${errors === 1 ? '' : 's'}`,
				warnings > 0 && `${warnings} avertissement${warnings === 1 ? '' : 's'}`
			].filter(Boolean);
			return parts.length === 0 ? events : `${events}, dont ${parts.join(' et ')}`;
		},
		columns: {
			level: 'Niveau',
			time: 'Heure',
			provider: 'Fournisseur',
			eventId: 'ID',
			task: 'Tâche',
			channel: 'Canal',
			computer: 'Ordinateur',
			message: 'Message'
		}
	},
	diagnose: {
		title: 'Diagnostic',
		subtitle:
			'Parcourt le journal à la recherche des événements qu’une machine écrit quand quelque chose a mal tourné, puis extrait le quart d’heure autour de l’un d’eux.',
		days: (count: number) => (count === 1 ? 'Dernier jour' : `${count} derniers jours`),
		scan: 'Analyser',
		scanning: 'Analyse…',
		intro:
			'Rien n’a encore été analysé. Choisissez une période ci-dessus et appuyez sur Analyser ; chaque trouvaille — un plantage, un gel, une erreur de disque, un processeur bridé — apparaît ici comme un incident à ouvrir.',
		pick: 'Ouvrez un incident pour voir tout ce que la machine a écrit dans le quart d’heure qui l’entoure.',
		nothing:
			'Rien trouvé. Analysez une période plus longue, ou prenez-le comme une bonne nouvelle.',
		window: (from: string, to: string) => `${from} — ${to}`,
		inWindow: (count: number) => `${count} événement${count === 1 ? '' : 's'} dans la fenêtre`,
		kinds: {
			unexpectedShutdown: 'Arrêt inattendu',
			bugCheck: 'Écran bleu',
			hardwareError: 'Erreur matérielle',
			appHang: 'Application bloquée',
			appCrash: 'Plantage d’application',
			serviceFailure: 'Défaillance de service',
			diskError: 'Erreur de disque',
			ntfs: 'Système de fichiers',
			displayTdr: 'Pilote graphique réinitialisé',
			processorPower: 'Processeur bridé'
		}
	},
	log: {
		title: 'Journal',
		subtitle:
			'Ce que l’application a fait, du plus ancien au plus récent. Rien ici n’est écrit sur le disque.',
		filter: 'Filtrer les messages…',
		level: 'Niveau',
		levels: {
			all: 'Tous les niveaux',
			error: 'Erreurs',
			warning: 'Avertissements',
			info: 'Infos',
			debug: 'Débogage'
		},
		clear: 'Vider le journal',
		empty: 'Rien de consigné pour l’instant.',
		count: (shown: number, total: number) => `${shown} sur ${total} entrée${total === 1 ? '' : 's'}`
	},
	info: {
		title: 'Infos',
		subtitle: 'Ce qu’est cette application, et sur quoi elle repose.',
		appBody:
			'Lire les journaux d’événements Windows et les filtrer pour n’en garder que l’essentiel — sans compte, sans envoi de données, sans télémétrie.',
		offline:
			'Tout s’exécute sur cette machine. Rien n’est envoyé, et aucune télémétrie n’est collectée.',
		appLicense: 'OpenEventViewer est sous licence MIT.',
		thirdParty: 'Composants tiers',
		thirdPartyBody: (total: number, vendored: number, crates: number, npm: number) =>
			`${total} composants sont livrés avec cette application : ${vendored} binaires embarqués, ${crates} crates Rust, ${npm} paquets npm.`,
		shipped:
			'Les textes de licence complets sont livrés dans l’installateur sous THIRD_PARTY_LICENSES.txt. MIT, BSD et ISC exigent tous que la notice accompagne le binaire ; un lien ne suffirait donc pas.',
		filter: 'Filtrer les composants…',
		showTexts: 'Afficher les textes de licence',
		hideTexts: 'Masquer les textes de licence',
		noMatch: 'Aucun composant ne correspond.',
		redistributed: 'livré sous forme de binaire',
		noOwnText: 'sans texte propre',
		withoutText: (count: number) =>
			`${count} composants n’ont publié aucun fichier de licence propre ; le texte canonique de la licence nommée s’applique.`,
		material: 'Vos journaux',
		materialBody:
			'Les journaux d’événements restent là où Windows les conserve. Cette application les lit et n’y écrit jamais.'
	},
	detail: {
		general: 'Général',
		data: 'Données de l’événement',
		xml: 'XML',
		search: 'Rechercher sur le web',
		copy: 'Copier',
		copied: 'Copié',
		close: 'Fermer le volet de détails',
		recordId: 'Enregistrement',
		keywords: 'Mots clés',
		noData: 'Cet événement ne porte aucune donnée propre.'
	},
	updater: {
		title: 'Mises à jour',
		body: (version: string) => `Version ${version}. Vérifiée une fois au démarrage.`,
		check: 'Vérifier maintenant',
		checking: 'Vérification…',
		upToDate: 'à jour',
		available: (version: string) => `${version} est disponible`,
		downloading: (percent: number | null) =>
			percent === null ? 'Téléchargement…' : `Téléchargement — ${percent} %`,
		ready: 'Installée — redémarrage',
		install: 'Installer et redémarrer',
		failed: 'La vérification des mises à jour a échoué.'
	},
	settings: {
		title: 'Paramètres',
		appearance: 'Apparence',
		appearanceBody: 'Le thème de la fenêtre de l’application.',
		system: 'Système',
		light: 'Clair',
		dark: 'Sombre',
		colours: 'Couleurs',
		coloursBody: 'La palette dont chaque vue est tirée.',
		presets: {
			default: 'Par défaut',
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
		language: 'Langue',
		languageBody:
			'L’interface de l’application. Le texte des événements garde la langue dans laquelle Windows l’a enregistré.',
		eventsRows: 'Événements : lignes à charger',
		eventsRowsBody:
			'Chaque événement coûte une consultation de message auprès du fournisseur — un nombre plus grand signifie donc une attente plus longue, pas seulement une liste plus longue.',
		eventsRowsValue: (rows: number) => `${rows.toLocaleString('fr')} lignes`,
		showLogs: 'Afficher le journal dans la barre latérale',
		showLogsBody: 'Ajoute une entrée Journal à la navigation.',
		debugLogging: 'Consigner les entrées de débogage',
		debugLoggingBody:
			'Verbeux. Désactivé par défaut, parce que les entrées de débogage noient justement celles que vous cherchiez.'
	}
};
