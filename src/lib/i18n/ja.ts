import type { Translations } from './en';

export const ja: Translations = {
	sidebar: {
		tagline: 'Windows イベントログ',
		events: 'イベント',
		diagnose: '診断',
		log: 'ログ',
		settings: '設定',
		info: '情報',
		toLight: 'ライトテーマに切り替え',
		toDark: 'ダークテーマに切り替え',
		sections: 'セクション',
		collapse: 'サイドバーを折りたたむ',
		expand: 'サイドバーを展開する'
	},
	common: {
		loading: '読み込み中…',
		mockHost:
			'モックホスト — Tauri バックエンドがありません。このページのデータはテスト用データです。'
	},
	events: {
		title: 'イベント',
		subtitle: 'Windows が記録した内容を、新しい順に表示します。',
		channel: 'チャネル',
		allChannels: 'システムとアプリケーション',
		from: '開始',
		to: '終了',
		load: '読み込む',
		span: (from: string, to: string) => `${from}〜${to}`,
		keyword: 'すべての列を検索…',
		columnFilter: '列フィルター',
		clearColumnFilters: '列フィルターをクリア',
		loaded: (shown: number, total: number) =>
			shown === total ? `${total} 件のイベント` : `${total} 件中 ${shown} 件のイベント`,
		elapsed: (ms: number) => `${ms} ms で読み取り`,
		truncated: '行数の上限を超えています — フィルターを絞り込むか、設定で上限を引き上げてください',
		securityHint:
			'OpenEventViewer を終了して管理者として起動し直すか、管理者権限の要らないチャネルを選んでください。',
		empty: '一致するものがありません。',
		search: 'このイベントを Web で検索',
		resize: (column: string) => `${column} 列の幅を変更`,
		filters: {
			search: '検索…',
			noMatch: '一致するものがありません。',
			clear: 'このフィルターをクリア',
			chosen: (count: number) => `${count} 件選択中`,
			after: (time: string) => `${time} より後`,
			before: (time: string) => `${time} より前`,
			timeHint: '現地時刻です。テーブルに表示されているのと同じ時計です。',
			numberHint: '数値が含まれていませんでした。',
			notUnderstood: (parts: string) => `解釈できませんでした: ${parts}`,
			helpAny: 'いずれかに一致',
			helpCompare: 'より大きい、より小さい',
			helpRange: '範囲（両端を含む）',
			helpNot: 'これ以外すべて'
		},
		overTime: '時間の推移',
		andMore: (kinds: number, count: number) => `ほかに ${kinds} 種類、合計 ${count} 件`,
		bucketSize: (minutes: number) =>
			minutes >= 1440
				? `1 本のバーは ${minutes / 1440} 日分`
				: minutes >= 60
					? `1 本のバーは ${minutes / 60} 時間分`
					: `1 本のバーは ${minutes} 分間分`,
		bucketCount: (total: number, errors: number, warnings: number) => {
			const events = `${total} 件のイベント`;
			const parts = [
				errors > 0 && `エラー ${errors} 件`,
				warnings > 0 && `警告 ${warnings} 件`
			].filter(Boolean);
			return parts.length === 0 ? events : `${events}、うち${parts.join('と')}`;
		},
		columns: {
			level: 'レベル',
			time: '時刻',
			provider: 'プロバイダー',
			eventId: 'ID',
			task: 'タスク',
			channel: 'チャネル',
			computer: 'コンピューター',
			message: 'メッセージ'
		}
	},
	diagnose: {
		title: '診断',
		subtitle:
			'問題が起きたときにマシンが書き込むイベントをログから探し出し、その前後 15 分間の記録を取り出します。',
		days: (count: number) => (count === 1 ? '過去 1 日' : `過去 ${count} 日`),
		scan: 'スキャン',
		scanning: 'スキャン中…',
		intro:
			'まだ何もスキャンしていません。上で期間を選んでスキャンを押してください。見つかったもの — クラッシュ、フリーズ、ディスクエラー、プロセッサのスロットリング — はすべて、開ける形のインシデントとしてここに表示されます。',
		pick: 'インシデントを開くと、その前後 15 分間にマシンが書き込んだすべての記録が見られます。',
		nothing:
			'何も見つかりませんでした。より長い期間をスキャンするか、良い知らせと受け取ってください。',
		window: (from: string, to: string) => `${from} — ${to}`,
		inWindow: (count: number) => `対象時間内に ${count} 件のイベント`,
		kinds: {
			unexpectedShutdown: '予期しないシャットダウン',
			bugCheck: 'バグチェック',
			hardwareError: 'ハードウェアエラー',
			appHang: 'アプリケーションのハング',
			appCrash: 'アプリケーションのクラッシュ',
			serviceFailure: 'サービスの障害',
			diskError: 'ディスクエラー',
			ntfs: 'ファイルシステム',
			displayTdr: 'ディスプレイドライバーのリセット',
			processorPower: 'プロセッサのスロットリング'
		}
	},
	log: {
		title: 'ログ',
		subtitle:
			'アプリの動作記録を、新しいものが下に来る順で表示します。ここの内容はディスクには書き込まれません。',
		filter: 'メッセージを絞り込む…',
		level: 'レベル',
		levels: {
			all: 'すべてのレベル',
			error: 'エラー',
			warning: '警告',
			info: '情報',
			debug: 'デバッグ'
		},
		clear: 'ログをクリア',
		empty: 'まだ何も記録されていません。',
		count: (shown: number, total: number) => `${total} 件中 ${shown} 件のエントリ`
	},
	info: {
		title: '情報',
		subtitle: 'このアプリが何であり、何の上に作られているか。',
		appBody:
			'Windows のイベントログを読み取り、重要なものだけに絞り込みます — アカウント不要、アップロードなし、テレメトリなし。',
		offline: 'すべてこのマシン上で動作します。何もアップロードされず、テレメトリも収集されません。',
		appLicense: 'OpenEventViewer は MIT ライセンスです。',
		thirdParty: 'サードパーティコンポーネント',
		thirdPartyBody: (total: number, vendored: number, crates: number, npm: number) =>
			`このアプリには ${total} 個のコンポーネントが同梱されています: バンドルされたバイナリ ${vendored} 個、Rust クレート ${crates} 個、npm パッケージ ${npm} 個。`,
		shipped:
			'ライセンス全文は THIRD_PARTY_LICENSES.txt としてインストーラーに同梱されています。MIT、BSD、ISC はいずれも通知文をバイナリに添付することを求めているため、リンクだけでは足りません。',
		filter: 'コンポーネントを絞り込む…',
		showTexts: 'ライセンス全文を表示',
		hideTexts: 'ライセンス全文を隠す',
		noMatch: '一致するコンポーネントがありません。',
		redistributed: 'バイナリとして同梱',
		noOwnText: '独自の全文なし',
		withoutText: (count: number) =>
			`${count} 個のコンポーネントは独自のライセンスファイルを公開していません。この場合は記載のライセンスの正規の全文が適用されます。`,
		material: 'あなたのログ',
		materialBody:
			'イベントログは Windows が保管している場所にそのまま残ります。このアプリは読み取るだけで、書き込むことはありません。'
	},
	detail: {
		general: '全般',
		data: 'イベントデータ',
		xml: 'XML',
		search: 'Web で検索',
		copy: 'コピー',
		copied: 'コピーしました',
		close: '詳細ペインを閉じる',
		recordId: 'レコード',
		keywords: 'キーワード',
		noData: 'このイベントには固有のデータがありません。'
	},
	updater: {
		title: '更新',
		body: (version: string) => `バージョン ${version}。起動時に一度確認します。`,
		check: '今すぐ確認',
		checking: '確認中…',
		upToDate: '最新です',
		available: (version: string) => `${version} が利用できます`,
		downloading: (percent: number | null) =>
			percent === null ? 'ダウンロード中…' : `ダウンロード中 — ${percent}%`,
		ready: 'インストール済み — 再起動します',
		install: 'インストールして再起動',
		failed: '更新の確認に失敗しました。'
	},
	settings: {
		title: '設定',
		appearance: '外観',
		appearanceBody: 'アプリウィンドウのテーマです。',
		system: 'システム',
		light: 'ライト',
		dark: 'ダーク',
		colours: '配色',
		coloursBody: 'すべての画面の描画に使われるパレットです。',
		presets: {
			default: 'デフォルト',
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
		language: '言語',
		languageBody: 'アプリの表示言語です。イベント本文は Windows が記録した言語のまま表示されます。',
		eventsRows: 'イベント: 読み込む行数',
		eventsRowsBody:
			'イベント 1 件ごとにプロバイダーへのメッセージ照会が発生するため、数を増やすとリストが長くなるというより待ち時間が長くなります。',
		eventsRowsValue: (rows: number) => `${rows.toLocaleString('ja')} 行`,
		showLogs: 'サイドバーにログを表示',
		showLogsBody: 'ナビゲーションにログの項目を追加します。',
		debugLogging: 'デバッグエントリを記録',
		debugLoggingBody:
			'出力が多くなります。デバッグエントリは探していたエントリを埋もれさせてしまうため、既定ではオフです。'
	}
};
