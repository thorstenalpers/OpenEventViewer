import type {
	Binder,
	CatalogEntry,
	CatalogRating,
	CommandArgs,
	CommandName,
	Identity,
	LeaderboardRow,
	Link,
	Note,
	LogEntry,
	Question,
	Settings,
	Video,
	VoicePack,
	Template,
	Certification,
	Artefact
} from './contract';

/**
 * The host stand-in for `npm run dev`, so every view can be built and tested in a browser without
 * Tauri, WebView2 or a database. It is deliberately thin: enough state to exercise the flows, no
 * attempt to mirror the real scheduler, the real synthesiser or the real assistant.
 */

const SAMPLE_QUESTIONS: Question[] = [
	{
		id: 1,
		number: 1,
		topic: 1,
		kind: 'single_choice',
		stem: 'You are designing an AI system that empowers everyone, including people who have hearing, visual, and other impairments.\nThis is an example of which Microsoft guiding principle for responsible AI?',
		options: [
			{ letter: 'A', text: 'fairness', isCorrect: false },
			{ letter: 'B', text: 'inclusiveness', isCorrect: true },
			{ letter: 'C', text: 'reliability and safety', isCorrect: false },
			{ letter: 'D', text: 'accountability', isCorrect: false }
		],
		answerLetters: ['B'],
		matrix: [],
		explanation:
			'Inclusiveness: intelligent technology must incorporate and address a broad range of human needs and experiences.',
		references: [
			'https://docs.microsoft.com/en-us/learn/modules/responsible-ai-principles/4-guiding-principles'
		],
		sourcePage: 2,
		confidence: 1,
		needsSource: false,
		warnings: [],
		figures: []
	},
	{
		id: 2,
		number: 2,
		topic: 1,
		kind: 'multiple_choice',
		stem: 'Which two components can you drag onto a canvas in Azure Machine Learning designer? Each correct answer presents a complete solution.',
		options: [
			{ letter: 'A', text: 'dataset', isCorrect: true },
			{ letter: 'B', text: 'compute', isCorrect: false },
			{ letter: 'C', text: 'pipeline', isCorrect: false },
			{ letter: 'D', text: 'module', isCorrect: true }
		],
		answerLetters: ['A', 'D'],
		matrix: [],
		explanation: 'You can drag datasets and modules onto the designer canvas.',
		references: [],
		sourcePage: 4,
		confidence: 1,
		needsSource: false,
		warnings: [],
		figures: []
	},
	{
		id: 3,
		number: 3,
		topic: 1,
		kind: 'matrix',
		stem: 'For each of the following statements, select Yes if the statement is true. Otherwise, select No.',
		options: [
			{ letter: 'A', text: 'Mastered', isCorrect: true },
			{ letter: 'B', text: 'Not Mastered', isCorrect: false }
		],
		answerLetters: ['A'],
		matrix: [
			{ index: 1, value: 'No' },
			{ index: 2, value: 'Yes' },
			{ index: 3, value: 'Yes' }
		],
		explanation: 'Anomaly detection covers fraud detection and intrusion patterns.',
		references: [],
		sourcePage: 2,
		confidence: 1,
		needsSource: false,
		warnings: [],
		figures: ['mock']
	},
	{
		id: 4,
		number: 4,
		topic: 2,
		kind: 'matrix',
		stem: 'To complete the sentence, select the appropriate option in the answer area.',
		options: [
			{ letter: 'A', text: 'Mastered', isCorrect: true },
			{ letter: 'B', text: 'Not Mastered', isCorrect: false }
		],
		answerLetters: ['A'],
		matrix: [],
		explanation: '',
		references: [],
		sourcePage: 5,
		confidence: 0.5,
		needsSource: true,
		warnings: [{ code: 'figure_missing' }],
		figures: []
	}
];

const BINDER: Binder = {
	id: 1,
	title: 'AI-900 (mock)',
	certification: 'AI-900',
	docUrl:
		'https://learn.microsoft.com/en-us/credentials/certifications/resources/study-guides/ai-900',
	sourceFile: 'certleader-ai900.pdf',
	profile: 'certleader',
	questionCount: SAMPLE_QUESTIONS.length,
	needsReviewCount: 1,
	needsSourceCount: 1,
	importedAt: '2026-08-12T09:00:00Z',
	lastStudiedAt: null,
	attemptCount: 0,
	accuracy: null,
	remoteId: null
};

/** Stands in for a rasterised answer area; the real host returns a PNG data URL of the same shape. */
const MOCK_FIGURE =
	'data:image/svg+xml;utf8,' +
	encodeURIComponent(
		`<svg xmlns="http://www.w3.org/2000/svg" width="520" height="150" font-family="sans-serif">
			<rect width="520" height="150" fill="#fff"/>
			<text x="12" y="26" font-size="15" font-weight="700">Answer Area</text>
			<text x="12" y="62" font-size="13">Statements</text>
			<text x="360" y="62" font-size="13">Yes</text>
			<text x="440" y="62" font-size="13">No</text>
			<text x="12" y="96" font-size="13">Anomaly detection can find unusual credit card use.</text>
			<circle cx="372" cy="91" r="7" fill="none" stroke="#555"/>
			<circle cx="450" cy="91" r="7" fill="none" stroke="#555"/>
			<text x="12" y="128" font-size="13">Anomaly detection needs labelled training data.</text>
			<circle cx="372" cy="123" r="7" fill="none" stroke="#555"/>
			<circle cx="450" cy="123" r="7" fill="none" stroke="#555"/>
		</svg>`
	);

let links: Link[] = [
	{
		id: 1,
		questionId: 1,
		url: 'https://docs.microsoft.com/en-us/learn/modules/responsible-ai-principles/4-guiding-principles',
		title: '4 guiding principles',
		description: 'The module the exam draws its responsible-AI question from.',
		kind: 'docs',
		minutes: 25
	},
	{
		id: 2,
		questionId: null,
		url: 'https://www.youtube.com/watch?v=-pX5PjIYTJs',
		title: 'AZ-900 Azure Fundamentals Full Course (2025)',
		description: 'Complete tutorial for beginners.',
		kind: 'course',
		minutes: 150
	}
];

let templates: Template[] = [
	'AZ-900',
	'AI-900',
	'DP-900',
	'MS-900',
	'SC-900',
	'PL-900',
	'GH-900',
	'AZ-104',
	'AZ-204',
	'AZ-305',
	'AZ-500',
	'AI-102',
	'DP-203',
	'SC-200',
	'SC-300'
].map((name, index) => ({
	id: index + 1,
	name,
	docUrl: `https://learn.microsoft.com/en-us/credentials/certifications/resources/study-guides/${name.toLowerCase()}`
}));

const certifications: Record<number, Certification[]> = {
	1: [{ id: 1, passedAt: '2026-03-14', note: 'first try' }],
	3: [
		{ id: 2, passedAt: '2024-06-02', note: '' },
		{ id: 3, passedAt: '2026-05-28', note: 'renewed' }
	]
};

const progress: Record<number, string[]> = {
	1: ['create', 'intro'],
	3: ['create', 'intro', 'study', 'train', 'pass']
};

let artefacts: Artefact[] = [];

let settings: Settings = { theme: 'system', showLogs: true, debugLogging: false };

let voicePacks: VoicePack[] = [
	{
		id: 'hub:Godelaune/Kokoro-82M-ONNX-German-Martin',
		language: 'de',
		label: 'Kokoro German (Martin)',
		megabytes: 330,
		installed: false,
		voices: 0,
		speakers: []
	},
	{
		id: 'hub:crane-local-ai/Kokoro-82M-v1.0-German-ONNX',
		language: 'de',
		label: 'Kokoro German (Kerstin)',
		megabytes: 330,
		installed: false,
		voices: 0,
		speakers: ['df_kerstin']
	},
	{
		id: 'kokoro-en-v0_19',
		language: 'en',
		label: 'Kokoro English',
		megabytes: 330,
		installed: false,
		voices: 0,
		speakers: [
			'af',
			'af_bella',
			'af_nicole',
			'af_sarah',
			'af_sky',
			'am_adam',
			'am_michael',
			'bf_emma',
			'bf_isabella',
			'bm_george',
			'bm_lewis'
		]
	}
];

const logEntries: LogEntry[] = [
	{ timestamp: '21:03:58.114', level: 'info', source: 'app', message: 'started' },
	{
		timestamp: '21:04:12.907',
		level: 'info',
		source: 'import',
		message: 'reading C:\\fixtures\\certleader-ai900.pdf'
	},
	{
		timestamp: '21:04:14.220',
		level: 'info',
		source: 'import',
		message: '11 questions, 5 figures, profile certleader'
	},
	{
		timestamp: '21:05:01.556',
		level: 'warning',
		source: 'assistant',
		message: 'no claude binary on PATH'
	}
];
let nextSessionId = 1;
let nextId = 100;
const videos: Video[] = [];
const notes: Note[] = [];
const wrongPerSession = new Map<number, number[]>();
const startedAt = new Map<number, number>();
/** What each session was run as, so a finished challenge can be told apart from a practice run. */
const ranAs = new Map<number, { mode: string; seed: number | null }>();

const projects: Binder[] = [
	BINDER,
	{
		...BINDER,
		id: 2,
		title: 'Azure AI Engineer',
		certification: 'AI-102',
		sourceFile: '',
		profile: '',
		questionCount: 0,
		needsReviewCount: 0,
		needsSourceCount: 0,
		importedAt: '2026-08-11T14:00:00Z',
		attemptCount: 0,
		accuracy: null
	},
	{
		...BINDER,
		id: 3,
		title: 'Security Fundamentals',
		certification: 'SC-900',
		sourceFile: 'sc900.pdf',
		questionCount: 42,
		needsReviewCount: 3,
		needsSourceCount: 0,
		importedAt: '2026-07-29T08:30:00Z',
		attemptCount: 61,
		accuracy: 0.72
	}
];

let publisher: Identity = { id: 'this-machine', name: 'local' };

/** Two entries nobody here published, so `mine` has something to be false about. */
let catalogEntries: CatalogEntry[] = [
	{
		id: 'entry-az900',
		ownerId: 'someone-else',
		ownerName: 'mira',
		mine: false,
		title: 'AZ-900 Fundamentals',
		certification: 'AZ-900',
		profile: 'certshared',
		questionCount: 120,
		needsSourceCount: 2,
		bytes: 486_000,
		publishedAt: '2026-07-02T10:00:00Z',
		updatedAt: '2026-07-02T10:00:00Z',
		ratingCount: 2,
		rating: 4.5
	},
	{
		id: 'entry-sc900',
		ownerId: 'someone-else',
		ownerName: 'mira',
		mine: false,
		title: 'SC-900 Security Fundamentals',
		certification: 'SC-900',
		profile: 'generic',
		questionCount: 42,
		needsSourceCount: 0,
		bytes: 130_500,
		publishedAt: '2026-08-01T08:00:00Z',
		updatedAt: '2026-08-01T08:00:00Z',
		ratingCount: 0,
		rating: null
	}
];

const catalogRatings: Record<string, CatalogRating[]> = {
	'entry-az900': [
		{
			raterId: 'someone-else',
			raterName: 'mira',
			mine: false,
			stars: 5,
			comment: 'Answer keys match the real exam objectives.',
			ratedAt: '2026-07-04T12:00:00Z'
		},
		{
			raterId: 'a-third-party',
			raterName: 'jo',
			mine: false,
			stars: 4,
			comment: '',
			ratedAt: '2026-07-11T09:30:00Z'
		}
	]
};

const boards: Record<string, LeaderboardRow[]> = {
	'entry-az900': [
		{
			runnerId: 'someone-else',
			runnerName: 'mira',
			mine: false,
			seed: 42,
			questionCount: 20,
			correct: 18,
			elapsedMs: 640_000,
			finishedAt: '2026-07-20T18:12:00Z'
		}
	]
};

function rank(entry: CatalogEntry): number {
	return entry.rating ?? -1;
}

export function mockHost<T extends CommandName>(name: T, args: CommandArgs[T]): unknown {
	switch (name) {
		case 'list_binders':
			return [...projects];

		case 'create_project': {
			const { title, certification } = args as CommandArgs['create_project'];
			const created: Binder = {
				...BINDER,
				id: ++nextId,
				title,
				certification,
				sourceFile: '',
				profile: '',
				questionCount: 0,
				needsReviewCount: 0,
				needsSourceCount: 0,
				importedAt: new Date().toISOString(),
				lastStudiedAt: null,
				attemptCount: 0,
				accuracy: null
			};
			projects.unshift(created);
			return created;
		}

		case 'dashboard':
			return {
				projectCount: projects.length,
				questionCount: projects.reduce((sum, p) => sum + p.questionCount, 0),
				answeredCount: 27,
				dueToday: 12,
				weakCount: 5,
				accuracy: 0.68,
				recentSessions: [
					{
						sessionId: 3,
						binderId: 1,
						binderTitle: BINDER.title,
						mode: 'practice',
						finishedAt: '2026-08-12 10:15:00',
						total: 11,
						correct: 8
					},
					{
						sessionId: 2,
						binderId: 3,
						binderTitle: 'Security Fundamentals',
						mode: 'challenge',
						finishedAt: '2026-08-11 19:02:00',
						total: 10,
						correct: 6
					}
				]
			};

		case 'import_file': {
			// A question bank has no pages and no figures; the report shape differs and the view
			// hides the figure columns for it, so the mock has to distinguish the two.
			const path = (args as CommandArgs['import_file']).path;
			if (/\.(md|markdown|json)$/i.test(path)) {
				return {
					binder: { ...BINDER, profile: 'bank-markdown', needsSourceCount: 0 },
					profile: 'bank-markdown',
					pages: 0,
					furnitureDropped: 0,
					missingNumbers: [],
					stubMarkers: [],
					figuresRecovered: 0
				};
			}
			return {
				binder: BINDER,
				profile: 'certleader',
				pages: 7,
				furnitureDropped: 30,
				missingNumbers: [11, 12, 13, 14],
				stubMarkers: [15],
				figuresRecovered: 5
			};
		}

		case 'delete_binder':
			return null;

		case 'list_questions': {
			const { onlyReview } = args as CommandArgs['list_questions'];
			return onlyReview ? SAMPLE_QUESTIONS.filter((q) => q.confidence < 0.75) : SAMPLE_QUESTIONS;
		}

		case 'start_session': {
			const { mode, sourceSessionId, rules } = args as CommandArgs['start_session'];
			const id = nextSessionId++;
			startedAt.set(id, Date.now());
			ranAs.set(id, { mode, seed: rules?.seed ?? null });
			const wrong = sourceSessionId ? (wrongPerSession.get(sourceSessionId) ?? []) : [];
			const scored = SAMPLE_QUESTIONS.filter((q) => !q.needsSource);
			const everMissed = [...new Set([...wrongPerSession.values()].flat())];
			let pool = scored;
			if (mode === 'focus' && wrong.length) pool = scored.filter((q) => wrong.includes(q.id));
			if (mode === 'weak') pool = scored.filter((q) => everMissed.includes(q.id));
			if (rules?.questionCount) pool = pool.slice(0, rules.questionCount);
			return {
				id,
				binderId: BINDER.id,
				binderTitle: BINDER.title,
				mode,
				rules: {
					seed: rules?.seed ?? null,
					questionCount: rules?.questionCount ?? null,
					timeLimitSeconds: rules?.timeLimitSeconds ?? null
				},
				questions: pool
			};
		}

		case 'record_attempt': {
			const { sessionId, questionId, given } = args as CommandArgs['record_attempt'];
			const target = SAMPLE_QUESTIONS.find((q) => q.id === questionId);
			const expected = target ? [...target.answerLetters].sort() : [];
			const actual = [...given].sort();
			const correct =
				expected.length === actual.length && expected.every((l, i) => l === actual[i]);
			if (!correct) {
				const list = wrongPerSession.get(sessionId) ?? [];
				list.push(questionId);
				wrongPerSession.set(sessionId, list);
			}
			return { correct, answerLetters: expected };
		}

		case 'finish_session': {
			const { sessionId } = args as CommandArgs['finish_session'];
			const wrong = wrongPerSession.get(sessionId) ?? [];
			const total = SAMPLE_QUESTIONS.filter((q) => !q.needsSource).length;
			return {
				sessionId,
				binderId: BINDER.id,
				mode: ranAs.get(sessionId)?.mode ?? 'practice',
				total,
				correct: total - wrong.length,
				elapsedMs: Date.now() - (startedAt.get(sessionId) ?? Date.now()),
				wrongQuestionIds: wrong
			};
		}

		case 'question_stats':
			return SAMPLE_QUESTIONS.map((question, index) => ({
				questionId: question.id,
				number: question.number,
				topic: question.topic,
				stem: question.stem,
				attempts: index === 0 ? 4 : index,
				correct: index === 0 ? 1 : index,
				accuracy: index === 0 ? 0.25 : index ? 1 : null,
				averageMs: index === 0 ? 21_500 : index ? 9_000 : null,
				lapses: index === 0 ? 3 : 0,
				dueAt: index === 0 ? '2026-08-13 09:00:00' : null,
				needsSource: question.needsSource
			}));

		case 'question_figure':
			return MOCK_FIGURE;

		case 'log_entries':
			return [...logEntries];

		case 'log_clear':
			logEntries.length = 0;
			return null;

		case 'log_write': {
			const { level, source, message } = args as CommandArgs['log_write'];
			logEntries.push({
				timestamp: new Date().toISOString().slice(11, 23),
				level,
				source,
				message
			});
			return null;
		}

		case 'third_party_licenses':
			// The real file is a bundled resource of the installer; the browser has no such thing.
			return 'THIRD_PARTY_LICENSES.txt ships inside the installer and is read from the app\nresources at runtime. Under the mock host there is no bundle to read it from.';

		case 'challenge_results':
			return [
				{
					sessionId: 1,
					seed: (args as CommandArgs['challenge_results']).seed,
					finishedAt: '2026-08-12 10:15:00',
					total: 2,
					correct: 2,
					elapsedMs: 41_000
				}
			];

		case 'export_deck':
		case 'peek_deck':
			return {
				format: 'examdeck/1',
				title: BINDER.title,
				certification: BINDER.certification,
				sourceFile: BINDER.sourceFile,
				profile: BINDER.profile,
				questionCount: BINDER.questionCount,
				exportedAt: '2026-08-12 10:00:00'
			};

		case 'import_deck':
			return BINDER;

		case 'list_links':
			return [...links];

		case 'save_link': {
			const { link } = args as CommandArgs['save_link'];
			// Keyed on the address, like the real host's unique index: pasting a link twice edits it.
			const stored = links.find((entry) => entry.url === link.url);
			if (stored) Object.assign(stored, link, { id: stored.id });
			else links.push({ ...link, id: ++nextId });
			return [...links];
		}

		case 'delete_link': {
			const { linkId } = args as CommandArgs['delete_link'];
			links = links.filter((entry) => entry.id !== linkId);
			return [...links];
		}

		case 'list_templates':
			return [...templates];

		case 'save_template': {
			const { name, docUrl } = args as CommandArgs['save_template'];
			if (!templates.some((entry) => entry.name === name && entry.docUrl === docUrl)) {
				templates = [...templates, { id: ++nextId, name, docUrl }];
			}
			return [...templates];
		}

		case 'delete_template': {
			const { templateId } = args as CommandArgs['delete_template'];
			templates = templates.filter((entry) => entry.id !== templateId);
			return [...templates];
		}

		case 'list_certifications': {
			const { binderId } = args as CommandArgs['list_certifications'];
			return [...(certifications[binderId] ?? [])];
		}

		case 'add_certification': {
			const { binderId, passedAt, note } = args as CommandArgs['add_certification'];
			const held = certifications[binderId] ?? (certifications[binderId] = []);
			held.push({ id: ++nextId, passedAt, note });
			held.sort((a, b) => a.passedAt.localeCompare(b.passedAt));
			return [...held];
		}

		case 'delete_certification': {
			const { binderId, certificationId } = args as CommandArgs['delete_certification'];
			certifications[binderId] = (certifications[binderId] ?? []).filter(
				(entry) => entry.id !== certificationId
			);
			return [...certifications[binderId]];
		}

		case 'list_progress': {
			const { binderId } = args as CommandArgs['list_progress'];
			return [...(progress[binderId] ?? [])];
		}

		case 'set_progress': {
			const { binderId, step, done } = args as CommandArgs['set_progress'];
			const steps = new Set(progress[binderId] ?? []);
			if (done) steps.add(step);
			else steps.delete(step);
			progress[binderId] = [...steps];
			return [...progress[binderId]];
		}

		case 'timeline':
			return projects.map((project) => ({
				binderId: project.id,
				title: project.title,
				certification: project.certification,
				startedAt: project.importedAt,
				lastStudiedAt: project.lastStudiedAt,
				questionCount: project.questionCount,
				passed: (certifications[project.id] ?? []).map((entry) => entry.passedAt)
			}));

		case 'list_videos':
			return [...videos];

		case 'add_video': {
			const { video } = args as CommandArgs['add_video'];
			videos.push({ ...video, id: nextId++ });
			return [...videos];
		}

		case 'delete_video': {
			const { videoId } = args as CommandArgs['delete_video'];
			const index = videos.findIndex((v) => v.id === videoId);
			if (index >= 0) videos.splice(index, 1);
			return [...videos];
		}

		case 'list_notes':
			return [...notes];

		case 'save_note': {
			const { note } = args as CommandArgs['save_note'];
			const existing = note.id ? notes.find((n) => n.id === note.id) : undefined;
			if (existing) {
				existing.bodyMd = note.bodyMd;
			} else {
				notes.unshift({
					id: nextId++,
					questionId: note.questionId,
					bodyMd: note.bodyMd,
					updatedAt: '2026-08-12 10:00:00'
				});
			}
			return [...notes];
		}

		case 'site_open':
		case 'site_place':
		case 'site_hide':
		case 'site_history':
		case 'site_focus':
			return null;

		case 'site_url':
			return null;

		// The browser has its own developer tools; the mock host has nothing to open.
		case 'devtools_open':
			return null;

		case 'assistant_status':
			return {
				source: (args as CommandArgs['assistant_status']).source,
				cliAvailable: false,
				hasKey: false
			};

		case 'assistant_set_key':
			return null;

		case 'assistant_ask': {
			const { task } = args as CommandArgs['assistant_ask'];
			return `[mock assistant] nothing was sent anywhere. Task requested: ${task}.`;
		}

		case 'podcast_build':
			return {
				path: `C:\\mock\\AI-900.${(args as CommandArgs['podcast_build']).options.format}`,
				durationMs: 92_000,
				chapters: SAMPLE_QUESTIONS.filter((q) => !q.needsSource).map((q, index) => ({
					questionNumber: q.number,
					offsetMs: index * 30_000,
					title: `Question ${q.number}`
				}))
			};

		case 'voice_packs':
			return [...voicePacks];

		// Installed at once and with no bytes moved: the browser mock exists to exercise the flow,
		// and a fake progress bar would only be a slower way to reach the same state.
		case 'voice_install': {
			const { id } = args as CommandArgs['voice_install'];
			voicePacks = voicePacks.map((pack) =>
				pack.id === id
					? { ...pack, installed: true, voices: Math.max(pack.speakers.length, 1) }
					: pack
			);
			return voicePacks.find((pack) => pack.id === id);
		}

		case 'voice_remove': {
			const { id } = args as CommandArgs['voice_remove'];
			voicePacks = voicePacks.map((pack) =>
				pack.id === id ? { ...pack, installed: false, voices: 0 } : pack
			);
			return null;
		}

		case 'voice_cancel':
		case 'voice_preview':
		case 'voice_stop':
		case 'voice_warm':
			return null;

		case 'list_artefacts':
			return [...artefacts];

		case 'notes_summarise': {
			const summary = `ai-900-mock-${artefacts.length + 1}.md`;
			artefacts = [
				...artefacts,
				{ name: summary, kind: 'md', bytes: 4096, path: `C:\\mock\\${summary}` }
			];
			return [...artefacts];
		}

		case 'notes_podcast': {
			const { name } = args as CommandArgs['notes_podcast'];
			const episode = name.replace(/\.md$/, '.mp3');
			artefacts = [
				...artefacts,
				{ name: episode, kind: 'mp3', bytes: 1_800_000, path: `C:\\mock\\${episode}` }
			];
			return [...artefacts];
		}

		case 'delete_artefact': {
			const { name } = args as CommandArgs['delete_artefact'];
			artefacts = artefacts.filter((entry) => entry.name !== name);
			return [...artefacts];
		}

		case 'catalog_identity':
			return publisher;

		case 'catalog_rename': {
			const { name: renamed } = args as CommandArgs['catalog_rename'];
			if (!renamed.trim()) throw new Error('a publisher needs a name');
			publisher = { ...publisher, name: renamed.trim() };
			catalogEntries = catalogEntries.map((entry) =>
				entry.mine ? { ...entry, ownerName: publisher.name } : entry
			);
			return publisher;
		}

		case 'catalog_list': {
			const { filter } = args as CommandArgs['catalog_list'];
			const search = filter?.search?.toLowerCase() ?? '';
			const found = catalogEntries.filter(
				(entry) =>
					(!filter?.certification || entry.certification === filter.certification) &&
					(!search ||
						entry.title.toLowerCase().includes(search) ||
						entry.certification.toLowerCase().includes(search))
			);
			// The same order the SQL produces, so a view built here is not rearranged by the host.
			switch (filter?.sort) {
				case 'rating':
					return found.sort((a, b) => rank(b) - rank(a) || a.title.localeCompare(b.title));
				case 'questions':
					return found.sort(
						(a, b) => b.questionCount - a.questionCount || a.title.localeCompare(b.title)
					);
				case 'title':
					return found.sort((a, b) => a.title.localeCompare(b.title));
				default:
					return found.sort((a, b) => b.publishedAt.localeCompare(a.publishedAt));
			}
		}

		case 'catalog_preview': {
			const { binderId } = args as CommandArgs['catalog_preview'];
			const project = projects.find((entry) => entry.id === binderId);
			if (!project) throw new Error(`no binder ${binderId}`);
			return {
				title: project.title,
				certification: project.certification,
				questionCount: project.questionCount,
				linkCount: links.length,
				videoCount: videos.length,
				noteCount: notes.length,
				figureCount: SAMPLE_QUESTIONS.filter((q) => q.figures.length > 0).length,
				bytes: 12_400 + project.questionCount * 900,
				includesSource: false
			};
		}

		case 'catalog_publish': {
			const { binderId } = args as CommandArgs['catalog_publish'];
			const project = projects.find((entry) => entry.id === binderId);
			if (!project) throw new Error(`no binder ${binderId}`);
			if (project.questionCount === 0) {
				throw new Error('an empty project has nothing to publish — import a file into it first');
			}
			const stamp = new Date().toISOString();
			const existing = catalogEntries.find((entry) => entry.mine && entry.title === project.title);
			const published: CatalogEntry = {
				id: existing?.id ?? `entry-${++nextId}`,
				ownerId: publisher.id,
				ownerName: publisher.name,
				mine: true,
				title: project.title,
				certification: project.certification,
				profile: project.profile,
				questionCount: project.questionCount,
				needsSourceCount: project.needsSourceCount,
				bytes: 12_400 + project.questionCount * 900,
				publishedAt: existing?.publishedAt ?? stamp,
				updatedAt: stamp,
				ratingCount: existing?.ratingCount ?? 0,
				rating: existing?.rating ?? null
			};
			catalogEntries = existing
				? catalogEntries.map((entry) => (entry.id === existing.id ? published : entry))
				: [published, ...catalogEntries];
			// The binder remembers which entry it is, which is how a finished challenge knows
			// there is a board to post to.
			project.remoteId = published.id;
			return published;
		}

		case 'catalog_withdraw': {
			const { entryId } = args as CommandArgs['catalog_withdraw'];
			const entry = catalogEntries.find((candidate) => candidate.id === entryId);
			if (entry && !entry.mine) throw new Error('only the publisher may withdraw a binder');
			catalogEntries = catalogEntries.filter((candidate) => candidate.id !== entryId);
			for (const project of projects) {
				if (project.remoteId === entryId) project.remoteId = null;
			}
			return [...catalogEntries];
		}

		case 'catalog_import': {
			const { entryId } = args as CommandArgs['catalog_import'];
			const entry = catalogEntries.find((candidate) => candidate.id === entryId);
			if (!entry) throw new Error(`no catalog entry ${entryId}`);
			const imported: Binder = {
				...BINDER,
				id: ++nextId,
				title: entry.title,
				certification: entry.certification,
				sourceFile: `${entry.id}.examdeck`,
				profile: entry.profile,
				questionCount: entry.questionCount,
				needsReviewCount: 0,
				needsSourceCount: entry.needsSourceCount,
				importedAt: new Date().toISOString(),
				lastStudiedAt: null,
				attemptCount: 0,
				accuracy: null
			};
			projects.unshift(imported);
			return imported;
		}

		case 'catalog_rate': {
			const { entryId, stars, comment } = args as CommandArgs['catalog_rate'];
			if (stars < 1 || stars > 5) throw new Error(`a rating is one to five stars, not ${stars}`);
			const existing = catalogRatings[entryId] ?? [];
			const mine: CatalogRating = {
				raterId: publisher.id,
				raterName: publisher.name,
				mine: true,
				stars,
				comment,
				ratedAt: new Date().toISOString()
			};
			catalogRatings[entryId] = [mine, ...existing.filter((rating) => !rating.mine)];
			const all = catalogRatings[entryId];
			catalogEntries = catalogEntries.map((entry) =>
				entry.id === entryId
					? {
							...entry,
							ratingCount: all.length,
							rating: all.reduce((sum, rating) => sum + rating.stars, 0) / all.length
						}
					: entry
			);
			return [...all];
		}

		case 'catalog_ratings':
			return [...(catalogRatings[(args as CommandArgs['catalog_ratings']).entryId] ?? [])];

		case 'catalog_post_result': {
			const { entryId, sessionId } = args as CommandArgs['catalog_post_result'];
			const ran = ranAs.get(sessionId);
			if (!ran || ran.seed === null) {
				throw new Error(
					`session ${sessionId} ran without a seed, so there is no board to post it to`
				);
			}
			const total = SAMPLE_QUESTIONS.filter((q) => !q.needsSource).length;
			boards[entryId] = [
				...(boards[entryId] ?? []),
				{
					runnerId: publisher.id,
					runnerName: publisher.name,
					mine: true,
					seed: ran.seed,
					questionCount: total,
					correct: total - (wrongPerSession.get(sessionId)?.length ?? 0),
					elapsedMs: Date.now() - (startedAt.get(sessionId) ?? Date.now()),
					finishedAt: new Date().toISOString()
				}
			].sort((a, b) => b.correct - a.correct || a.elapsedMs - b.elapsedMs);
			return [...boards[entryId]];
		}

		case 'catalog_leaderboard': {
			const { entryId, seed } = args as CommandArgs['catalog_leaderboard'];
			return (boards[entryId] ?? []).filter((row) => row.seed === seed);
		}

		case 'catalog_seeds': {
			const { entryId } = args as CommandArgs['catalog_seeds'];
			return [...new Set((boards[entryId] ?? []).map((row) => row.seed))].sort((a, b) => a - b);
		}

		case 'progress_push':
			return { pushed: 3, pulled: 0, skipped: 1 };

		case 'progress_pull':
			return { pushed: 0, pulled: 0, skipped: 3 };

		case 'get_settings':
			return settings;

		case 'set_settings':
			settings = (args as CommandArgs['set_settings']).settings;
			return settings;

		default:
			throw new Error(`mock host has no command ${name}`);
	}
}
