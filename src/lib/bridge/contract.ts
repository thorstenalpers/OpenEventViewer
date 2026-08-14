import { z } from 'zod';

export const questionKind = z.enum(['single_choice', 'multiple_choice', 'matrix', 'image_based']);
export type QuestionKind = z.infer<typeof questionKind>;

export const warning = z.object({
	code: z.string(),
	detail: z.unknown().optional()
});

export const answerOption = z.object({
	letter: z.string(),
	text: z.string(),
	isCorrect: z.boolean()
});
export type AnswerOption = z.infer<typeof answerOption>;

export const matrixBox = z.object({
	index: z.number(),
	value: z.string()
});

export const question = z.object({
	id: z.number(),
	number: z.number(),
	topic: z.number().nullable(),
	kind: questionKind,
	stem: z.string(),
	options: z.array(answerOption),
	answerLetters: z.array(z.string()),
	matrix: z.array(matrixBox),
	explanation: z.string(),
	references: z.array(z.string()),
	sourcePage: z.number(),
	confidence: z.number(),
	needsSource: z.boolean(),
	warnings: z.array(warning),
	figures: z.array(z.string()).default([])
});
export type Question = z.infer<typeof question>;

export const binder = z.object({
	id: z.number(),
	title: z.string(),
	certification: z.string(),
	/** Where the vendor documents this exam; empty for a project created before templates existed. */
	docUrl: z.string().default(''),
	sourceFile: z.string(),
	profile: z.string(),
	questionCount: z.number(),
	needsReviewCount: z.number(),
	needsSourceCount: z.number(),
	importedAt: z.string(),
	lastStudiedAt: z.string().nullable(),
	attemptCount: z.number(),
	accuracy: z.number().nullable(),
	/** The catalog entry this binder was published as; `null` until it is published. */
	remoteId: z.string().nullable().default(null)
});
export type Binder = z.infer<typeof binder>;

export const recentSession = z.object({
	sessionId: z.number(),
	binderId: z.number(),
	binderTitle: z.string(),
	mode: z.string(),
	finishedAt: z.string(),
	total: z.number(),
	correct: z.number()
});
export type RecentSession = z.infer<typeof recentSession>;

export const dashboardSummary = z.object({
	projectCount: z.number(),
	questionCount: z.number(),
	answeredCount: z.number(),
	dueToday: z.number(),
	weakCount: z.number(),
	/** `null` until something has been answered — nothing attempted is not 0 % correct. */
	accuracy: z.number().nullable(),
	recentSessions: z.array(recentSession)
});
export type DashboardSummary = z.infer<typeof dashboardSummary>;

export const importReport = z.object({
	binder,
	profile: z.string(),
	pages: z.number(),
	furnitureDropped: z.number(),
	missingNumbers: z.array(z.number()),
	stubMarkers: z.array(z.number()),
	figuresRecovered: z.number().default(0)
});
export type ImportReport = z.infer<typeof importReport>;

export const manifest = z.object({
	format: z.string(),
	title: z.string(),
	certification: z.string(),
	/** Where the vendor documents this exam; empty for a project created before templates existed. */
	docUrl: z.string().default(''),
	sourceFile: z.string(),
	profile: z.string(),
	questionCount: z.number(),
	exportedAt: z.string()
});
export type Manifest = z.infer<typeof manifest>;

export const linkKind = z.enum(['course', 'video', 'docs', 'other']);
export type LinkKind = z.infer<typeof linkKind>;

export const link = z.object({
	id: z.number(),
	questionId: z.number().nullable(),
	url: z.string(),
	title: z.string(),
	description: z.string().default(''),
	kind: linkKind.catch('other'),
	/** How long it takes, where that is known. */
	minutes: z.number().nullable().default(null)
});
export type Link = z.infer<typeof link>;

/** An exam as it exists before anyone studies for it: what it is called and where it is documented. */
export const template = z.object({
	id: z.number(),
	name: z.string(),
	docUrl: z.string()
});
export type Template = z.infer<typeof template>;

/** One time an exam was passed. A project can hold several — a certification expires and is retaken. */
export const certification = z.object({
	id: z.number(),
	passedAt: z.string(),
	note: z.string().default('')
});
export type Certification = z.infer<typeof certification>;

export const examTimeline = z.object({
	binderId: z.number(),
	title: z.string(),
	certification: z.string(),
	startedAt: z.string(),
	lastStudiedAt: z.string().nullable(),
	questionCount: z.number(),
	passed: z.array(z.string())
});
export type ExamTimeline = z.infer<typeof examTimeline>;

export const video = z.object({
	id: z.number(),
	questionId: z.number().nullable(),
	url: z.string(),
	title: z.string(),
	startSeconds: z.number()
});
export type Video = z.infer<typeof video>;

export const note = z.object({
	id: z.number(),
	questionId: z.number().nullable(),
	bodyMd: z.string(),
	updatedAt: z.string()
});
export type Note = z.infer<typeof note>;

export const sessionMode = z.enum(['practice', 'focus', 'due', 'weak', 'exam', 'challenge']);
export type SessionMode = z.infer<typeof sessionMode>;

export const questionStat = z.object({
	questionId: z.number(),
	number: z.number(),
	topic: z.number().nullable(),
	stem: z.string(),
	attempts: z.number(),
	correct: z.number(),
	accuracy: z.number().nullable(),
	averageMs: z.number().nullable(),
	lapses: z.number(),
	dueAt: z.string().nullable(),
	needsSource: z.boolean()
});
export type QuestionStat = z.infer<typeof questionStat>;

export const ruleSet = z.object({
	seed: z.number().nullable(),
	questionCount: z.number().nullable(),
	timeLimitSeconds: z.number().nullable()
});
export type RuleSet = z.infer<typeof ruleSet>;

export const session = z.object({
	id: z.number(),
	binderId: z.number(),
	binderTitle: z.string(),
	mode: sessionMode,
	rules: ruleSet,
	questions: z.array(question)
});
export type Session = z.infer<typeof session>;

export const sessionSummary = z.object({
	sessionId: z.number(),
	binderId: z.number(),
	mode: sessionMode,
	total: z.number(),
	correct: z.number(),
	elapsedMs: z.number(),
	wrongQuestionIds: z.array(z.number())
});
export type SessionSummary = z.infer<typeof sessionSummary>;

export const challengeResult = z.object({
	sessionId: z.number(),
	seed: z.number(),
	finishedAt: z.string(),
	total: z.number(),
	correct: z.number(),
	elapsedMs: z.number()
});
export type ChallengeResult = z.infer<typeof challengeResult>;

export const attemptResult = z.object({
	correct: z.boolean(),
	answerLetters: z.array(z.string())
});
export type AttemptResult = z.infer<typeof attemptResult>;

export const assistantSource = z.enum(['cli', 'anthropic']);
export type AssistantSource = z.infer<typeof assistantSource>;

export const assistantTask = z.enum(['explain', 'variants', 'note']);
export type AssistantTask = z.infer<typeof assistantTask>;

export const assistantStatus = z.object({
	source: assistantSource,
	cliAvailable: z.boolean(),
	hasKey: z.boolean()
});
export type AssistantStatus = z.infer<typeof assistantStatus>;

/** A downloadable voice, and what is on disk for it. */
export const voicePack = z.object({
	id: z.string(),
	language: z.string(),
	label: z.string(),
	megabytes: z.number(),
	installed: z.boolean(),
	/** How many speakers the installed pack offers; zero until it is there. */
	voices: z.number(),
	/** Their names where they are known; empty means they are only numbered. */
	speakers: z.array(z.string())
});
export type VoicePack = z.infer<typeof voicePack>;

export const voiceProgress = z.object({
	id: z.string(),
	received: z.number(),
	/** Null while the server has not said how big the download is. */
	total: z.number().nullable(),
	/** The bytes are all here and are being written out, which takes minutes for a whole pack. */
	unpacking: z.boolean()
});
export type VoiceProgress = z.infer<typeof voiceProgress>;

/** Which downloaded voice reads, and which of its speakers. */
export const podcastVoice = z.object({
	packId: z.string(),
	speaker: z.number()
});
export type PodcastVoice = z.infer<typeof podcastVoice>;

export const podcastOptions = z.object({
	includeAnswer: z.boolean(),
	includeExplanation: z.boolean(),
	pauseSeconds: z.number(),
	format: z.enum(['mp3', 'wav']),
	language: z.enum(['en', 'de']),
	/** Null reads with the Windows voice for the language. */
	voice: podcastVoice.nullable()
});
export type PodcastOptions = z.infer<typeof podcastOptions>;

export const episode = z.object({
	path: z.string(),
	durationMs: z.number(),
	chapters: z.array(
		z.object({
			questionNumber: z.number(),
			offsetMs: z.number(),
			title: z.string()
		})
	)
});
export type Episode = z.infer<typeof episode>;

/** A file this app made out of the user's own notes. */
export const artefact = z.object({
	name: z.string(),
	kind: z.string(),
	bytes: z.number(),
	path: z.string()
});
export type Artefact = z.infer<typeof artefact>;

/** Who this machine publishes as. The stand-in for an account: no password, no server, one row. */
export const identity = z.object({
	id: z.string(),
	name: z.string()
});
export type Identity = z.infer<typeof identity>;

export const catalogEntry = z.object({
	id: z.string(),
	ownerId: z.string(),
	ownerName: z.string(),
	/** Published from this machine — what a row-level policy would decide server-side. */
	mine: z.boolean(),
	title: z.string(),
	certification: z.string(),
	profile: z.string(),
	questionCount: z.number(),
	needsSourceCount: z.number(),
	bytes: z.number(),
	publishedAt: z.string(),
	updatedAt: z.string(),
	ratingCount: z.number(),
	/** `null` until somebody rates it — no rating is not nought stars. */
	rating: z.number().nullable()
});
export type CatalogEntry = z.infer<typeof catalogEntry>;

/** Exactly what publishing would put in the catalog, measured off the deck the publish writes. */
export const uploadPreview = z.object({
	title: z.string(),
	certification: z.string(),
	questionCount: z.number(),
	linkCount: z.number(),
	videoCount: z.number(),
	noteCount: z.number(),
	figureCount: z.number(),
	bytes: z.number(),
	includesSource: z.boolean()
});
export type UploadPreview = z.infer<typeof uploadPreview>;

export const catalogRating = z.object({
	raterId: z.string(),
	raterName: z.string(),
	mine: z.boolean(),
	stars: z.number(),
	comment: z.string(),
	ratedAt: z.string()
});
export type CatalogRating = z.infer<typeof catalogRating>;

export const leaderboardRow = z.object({
	runnerId: z.string(),
	runnerName: z.string(),
	mine: z.boolean(),
	seed: z.number(),
	questionCount: z.number(),
	correct: z.number(),
	elapsedMs: z.number(),
	finishedAt: z.string()
});
export type LeaderboardRow = z.infer<typeof leaderboardRow>;

export const syncReport = z.object({
	pushed: z.number(),
	pulled: z.number(),
	/** Rows left alone because the other side already held a longer history of them. */
	skipped: z.number()
});
export type SyncReport = z.infer<typeof syncReport>;

export const catalogSort = z.enum(['recent', 'rating', 'questions', 'title']);
export type CatalogSort = z.infer<typeof catalogSort>;

export interface CatalogFilter {
	certification?: string;
	search?: string;
	sort?: CatalogSort;
}

export const rect = z.object({
	x: z.number(),
	y: z.number(),
	width: z.number(),
	height: z.number()
});
export type Rect = z.infer<typeof rect>;

export const settings = z.object({
	theme: z.enum(['system', 'light', 'dark']),
	showLogs: z.boolean().default(false),
	debugLogging: z.boolean().default(false)
});
export type Settings = z.infer<typeof settings>;

export const logLevel = z.enum(['debug', 'info', 'warning', 'error']);
export type LogLevel = z.infer<typeof logLevel>;

export const logEntry = z.object({
	timestamp: z.string(),
	level: logLevel,
	source: z.string(),
	message: z.string()
});
export type LogEntry = z.infer<typeof logEntry>;

/**
 * The command surface. Keys are the Tauri command names; each entry pairs the argument shape with
 * the response schema, so the client validates every reply against one declaration rather than
 * trusting the host.
 */
export const commands = {
	list_binders: { response: z.array(binder) },
	create_project: { response: binder },
	dashboard: { response: dashboardSummary },
	import_file: { response: importReport },
	delete_binder: { response: z.null() },
	list_questions: { response: z.array(question) },
	start_session: { response: session },
	record_attempt: { response: attemptResult },
	finish_session: { response: sessionSummary },
	challenge_results: { response: z.array(challengeResult) },
	question_stats: { response: z.array(questionStat) },
	question_figure: { response: z.string() },
	third_party_licenses: { response: z.string() },
	log_entries: { response: z.array(logEntry) },
	log_clear: { response: z.null() },
	log_write: { response: z.null() },
	export_deck: { response: manifest },
	import_deck: { response: binder },
	peek_deck: { response: manifest },
	list_links: { response: z.array(link) },
	save_link: { response: z.array(link) },
	delete_link: { response: z.array(link) },
	list_templates: { response: z.array(template) },
	save_template: { response: z.array(template) },
	delete_template: { response: z.array(template) },
	list_certifications: { response: z.array(certification) },
	add_certification: { response: z.array(certification) },
	delete_certification: { response: z.array(certification) },
	list_progress: { response: z.array(z.string()) },
	set_progress: { response: z.array(z.string()) },
	timeline: { response: z.array(examTimeline) },
	list_videos: { response: z.array(video) },
	add_video: { response: z.array(video) },
	delete_video: { response: z.array(video) },
	list_notes: { response: z.array(note) },
	save_note: { response: z.array(note) },
	site_open: { response: z.null() },
	site_place: { response: z.null() },
	site_hide: { response: z.null() },
	site_history: { response: z.null() },
	site_url: { response: z.string().nullable() },
	site_focus: { response: z.null() },
	devtools_open: { response: z.null() },
	assistant_status: { response: assistantStatus },
	assistant_set_key: { response: z.null() },
	assistant_ask: { response: z.string() },
	podcast_build: { response: episode },
	notes_summarise: { response: z.array(artefact) },
	notes_podcast: { response: z.array(artefact) },
	list_artefacts: { response: z.array(artefact) },
	delete_artefact: { response: z.array(artefact) },
	notes_pdf: { response: z.array(artefact) },
	voice_packs: { response: z.array(voicePack) },
	voice_install: { response: voicePack },
	voice_cancel: { response: z.null() },
	voice_remove: { response: z.null() },
	voice_preview: { response: z.null() },
	voice_stop: { response: z.null() },
	voice_warm: { response: z.null() },
	catalog_identity: { response: identity },
	catalog_rename: { response: identity },
	catalog_list: { response: z.array(catalogEntry) },
	catalog_preview: { response: uploadPreview },
	catalog_publish: { response: catalogEntry },
	catalog_withdraw: { response: z.array(catalogEntry) },
	catalog_import: { response: binder },
	catalog_rate: { response: z.array(catalogRating) },
	catalog_ratings: { response: z.array(catalogRating) },
	catalog_post_result: { response: z.array(leaderboardRow) },
	catalog_leaderboard: { response: z.array(leaderboardRow) },
	catalog_seeds: { response: z.array(z.number()) },
	progress_push: { response: syncReport },
	progress_pull: { response: syncReport },
	get_settings: { response: settings },
	set_settings: { response: settings }
} as const;

export type CommandName = keyof typeof commands;
export type CommandResponse<T extends CommandName> = z.infer<(typeof commands)[T]['response']>;

export interface CommandArgs {
	list_binders: Record<string, never>;
	create_project: { title: string; certification: string; docUrl?: string };
	dashboard: Record<string, never>;
	import_file: { path: string; projectId?: number };
	delete_binder: { binderId: number };
	list_questions: { binderId: number; onlyReview?: boolean };
	start_session: {
		binderId: number;
		mode: SessionMode;
		sourceSessionId?: number;
		rules?: RuleSet;
	};
	record_attempt: {
		sessionId: number;
		questionId: number;
		given: string[];
		elapsedMs: number;
	};
	finish_session: { sessionId: number };
	challenge_results: { binderId: number; seed: number };
	question_stats: { binderId: number };
	question_figure: { hash: string };
	third_party_licenses: Record<string, never>;
	log_entries: Record<string, never>;
	log_clear: Record<string, never>;
	log_write: { level: LogLevel; source: string; message: string };
	export_deck: { binderId: number; path: string };
	import_deck: { path: string };
	peek_deck: { path: string };
	list_links: { binderId: number };
	save_link: { binderId: number; link: Omit<Link, 'id'> & { id?: number } };
	delete_link: { binderId: number; linkId: number };
	list_templates: Record<string, never>;
	save_template: { name: string; docUrl: string };
	delete_template: { templateId: number };
	list_certifications: { binderId: number };
	add_certification: { binderId: number; passedAt: string; note: string };
	delete_certification: { binderId: number; certificationId: number };
	list_progress: { binderId: number };
	set_progress: { binderId: number; step: string; done: boolean };
	timeline: Record<string, never>;
	list_videos: { binderId: number };
	add_video: { binderId: number; video: Omit<Video, 'id'> & { id?: number } };
	delete_video: { binderId: number; videoId: number };
	list_notes: { binderId: number };
	save_note: { binderId: number; note: Omit<Note, 'id' | 'updatedAt'> & { id?: number } };
	site_open: { url: string; rect: Rect };
	site_place: { rect: Rect };
	site_hide: Record<string, never>;
	site_history: { step: number };
	site_url: Record<string, never>;
	site_focus: { target: 'chrome' | 'site' };
	devtools_open: Record<string, never>;
	assistant_status: { source: AssistantSource };
	assistant_set_key: { key: string };
	assistant_ask: { source: AssistantSource; task: AssistantTask; questionId: number };
	podcast_build: { binderId: number; questionIds: number[]; options: PodcastOptions };
	notes_summarise: { binderId: number; source: AssistantSource };
	notes_podcast: { binderId: number; name: string; options: PodcastOptions };
	list_artefacts: { binderId: number };
	delete_artefact: { binderId: number; name: string };
	notes_pdf: { binderId: number; name: string };
	voice_packs: Record<string, never>;
	voice_install: { id: string };
	voice_cancel: { id: string };
	voice_remove: { id: string };
	voice_preview: { id: string; speaker: number; text: string; language: 'en' | 'de' };
	voice_stop: Record<string, never>;
	voice_warm: { id: string };
	catalog_identity: Record<string, never>;
	catalog_rename: { name: string };
	catalog_list: { filter?: CatalogFilter };
	catalog_preview: { binderId: number };
	catalog_publish: { binderId: number };
	catalog_withdraw: { entryId: string };
	catalog_import: { entryId: string };
	catalog_rate: { entryId: string; stars: number; comment: string };
	catalog_ratings: { entryId: string };
	catalog_post_result: { entryId: string; sessionId: number };
	catalog_leaderboard: { entryId: string; seed: number };
	catalog_seeds: { entryId: string };
	progress_push: Record<string, never>;
	progress_pull: Record<string, never>;
	get_settings: Record<string, never>;
	set_settings: { settings: Settings };
}
