import type { Translations } from './en';

export const zh: Translations = {
	sidebar: {
		tagline: 'Windows 事件日志',
		events: '事件',
		diagnose: '诊断',
		log: '日志',
		settings: '设置',
		info: '关于',
		toLight: '切换到浅色主题',
		toDark: '切换到深色主题',
		sections: '栏目',
		collapse: '收起侧边栏',
		expand: '展开侧边栏'
	},
	common: {
		loading: '正在加载…',
		mockHost: '模拟主机 — 没有 Tauri 后端。此页面上的数据是测试数据。'
	},
	events: {
		title: '事件',
		subtitle: 'Windows 记录的内容，最新的在前。',
		channel: '通道',
		allChannels: '系统和应用程序',
		from: '从',
		to: '到',
		load: '加载',
		span: (from: string, to: string) => `${from} 至 ${to}`,
		keyword: '搜索所有列…',
		columnFilter: '列筛选器',
		clearColumnFilters: '清除列筛选器',
		loaded: (shown: number, total: number) =>
			shown === total ? `${total} 个事件` : `${shown} / ${total} 个事件`,
		elapsed: (ms: number) => `读取用时 ${ms} 毫秒`,
		truncated: '超出行数上限 — 收窄筛选条件，或在设置中调高上限',
		securityHint: '关闭 OpenEventViewer 并以管理员身份重新启动，或选择一个不需要管理员权限的通道。',
		empty: '没有匹配项。',
		search: '在网上搜索此事件',
		resize: (column: string) => `调整“${column}”列的宽度`,
		filters: {
			search: '搜索…',
			noMatch: '没有匹配项。',
			clear: '清除此筛选器',
			chosen: (count: number) => `已选 ${count} 项`,
			after: (time: string) => `${time} 之后`,
			before: (time: string) => `${time} 之前`,
			timeHint: '本地时间，与表格显示的时钟相同。',
			numberHint: '其中没有数字。',
			notUnderstood: (parts: string) => `无法理解：${parts}`,
			helpAny: '其中任意一个',
			helpCompare: '大于、小于',
			helpRange: '一个范围，含两端',
			helpNot: '排除某项'
		},
		overTime: '时间分布',
		andMore: (kinds: number, count: number) => `另有 ${kinds} 种，共 ${count} 个`,
		bucketSize: (minutes: number) =>
			minutes >= 1440
				? `每 ${minutes / 1440} 天一根柱`
				: minutes >= 60
					? `每 ${minutes / 60} 小时一根柱`
					: `每 ${minutes} 分钟一根柱`,
		bucketCount: (total: number, errors: number, warnings: number) => {
			const events = `${total} 个事件`;
			const parts = [errors > 0 && `${errors} 个错误`, warnings > 0 && `${warnings} 个警告`].filter(
				Boolean
			);
			return parts.length === 0 ? events : `${events}，其中 ${parts.join('和')}`;
		},
		columns: {
			level: '级别',
			time: '时间',
			provider: '提供程序',
			eventId: 'ID',
			task: '任务',
			channel: '通道',
			computer: '计算机',
			message: '消息'
		}
	},
	diagnose: {
		title: '诊断',
		subtitle: '在日志中查找机器出问题时写下的事件，然后取出其中一个前后一刻钟内的全部记录。',
		days: (count: number) => (count === 1 ? '最近一天' : `最近 ${count} 天`),
		scan: '扫描',
		scanning: '正在扫描…',
		intro:
			'尚未扫描任何内容。在上方选择一段时间并点击“扫描”；每个发现 — 崩溃、死机、磁盘错误、处理器降频 — 都会作为可打开的事故显示在这里。',
		pick: '打开一个事故，查看机器在其前后一刻钟内写下的全部内容。',
		nothing: '未发现任何问题。扫描更长的时间段，或者把它当作好消息。',
		window: (from: string, to: string) => `${from} — ${to}`,
		inWindow: (count: number) => `窗口内 ${count} 个事件`,
		kinds: {
			unexpectedShutdown: '意外关机',
			bugCheck: '蓝屏错误',
			hardwareError: '硬件错误',
			appHang: '应用程序挂起',
			appCrash: '应用程序崩溃',
			serviceFailure: '服务故障',
			diskError: '磁盘错误',
			ntfs: '文件系统',
			displayTdr: '显示驱动程序重置',
			processorPower: '处理器降频'
		}
	},
	log: {
		title: '日志',
		subtitle: '应用做过的事，最新的在后。这里的内容不会写入磁盘。',
		filter: '筛选消息…',
		level: '级别',
		levels: {
			all: '所有级别',
			error: '错误',
			warning: '警告',
			info: '信息',
			debug: '调试'
		},
		clear: '清空日志',
		empty: '尚无日志记录。',
		count: (shown: number, total: number) => `${shown} / ${total} 条`
	},
	info: {
		title: '关于',
		subtitle: '这个应用是什么，以及它建立在什么之上。',
		appBody: '读取 Windows 事件日志并筛选出真正重要的内容 — 无需账户，不上传，不收集遥测数据。',
		offline: '一切都在这台机器上运行。不上传任何内容，也不收集任何遥测数据。',
		appLicense: 'OpenEventViewer 采用 MIT 许可证。',
		thirdParty: '第三方组件',
		thirdPartyBody: (total: number, vendored: number, crates: number, npm: number) =>
			`此应用附带 ${total} 个组件：${vendored} 个捆绑的二进制文件、${crates} 个 Rust crate、${npm} 个 npm 包。`,
		shipped:
			'完整的许可证文本作为 THIRD_PARTY_LICENSES.txt 随安装程序一起提供。MIT、BSD 和 ISC 都要求声明随二进制文件一同分发，仅提供链接是不够的。',
		filter: '筛选组件…',
		showTexts: '显示许可证文本',
		hideTexts: '隐藏许可证文本',
		noMatch: '没有匹配的组件。',
		redistributed: '以二进制文件形式提供',
		noOwnText: '无自带文本',
		withoutText: (count: number) =>
			`${count} 个组件未发布自己的许可证文件；适用所述许可证的标准文本。`,
		material: '你的日志',
		materialBody: '事件日志留在 Windows 保存它们的地方。此应用只读取它们，从不写入。'
	},
	detail: {
		general: '常规',
		data: '事件数据',
		xml: 'XML',
		search: '在网上搜索',
		copy: '复制',
		copied: '已复制',
		close: '关闭详细信息窗格',
		recordId: '记录',
		keywords: '关键字',
		noData: '此事件不带自己的数据。'
	},
	updater: {
		title: '更新',
		body: (version: string) => `版本 ${version}。启动时检查一次。`,
		check: '立即检查',
		checking: '正在检查…',
		upToDate: '已是最新',
		available: (version: string) => `${version} 可用`,
		downloading: (percent: number | null) =>
			percent === null ? '正在下载…' : `正在下载 — ${percent}%`,
		ready: '已安装 — 正在重启',
		install: '安装并重启',
		failed: '更新检查失败。'
	},
	settings: {
		title: '设置',
		appearance: '外观',
		appearanceBody: '应用窗口的主题。',
		system: '系统',
		light: '浅色',
		dark: '深色',
		colours: '颜色',
		coloursBody: '每个视图使用的调色板。',
		presets: {
			default: '默认',
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
		language: '语言',
		languageBody: '应用界面的语言。事件文本保持 Windows 记录时所用的语言。',
		eventsRows: '事件：每次加载的行数',
		eventsRowsBody:
			'每个事件都需要向提供程序查询一次消息，所以更大的数字意味着更长的等待，而不只是更长的列表。',
		eventsRowsValue: (rows: number) => `${rows.toLocaleString('zh')} 行`,
		showLogs: '在侧边栏显示日志',
		showLogsBody: '在导航中添加一个日志入口。',
		debugLogging: '记录调试条目',
		debugLoggingBody: '非常详细。默认关闭，因为调试条目会淹没你真正要找的那些。'
	}
};
