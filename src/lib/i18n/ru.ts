import type { Translations } from './en';

export const ru: Translations = {
	sidebar: {
		tagline: 'Журналы событий Windows',
		events: 'События',
		diagnose: 'Диагностика',
		log: 'Журнал',
		settings: 'Настройки',
		info: 'О программе',
		toLight: 'Переключиться на светлую тему',
		toDark: 'Переключиться на тёмную тему',
		sections: 'Разделы',
		collapse: 'Свернуть боковую панель',
		expand: 'Развернуть боковую панель'
	},
	common: {
		loading: 'Загрузка…',
		mockHost: 'Mock-хост — без бэкенда Tauri. Данные на этой странице — тестовые.'
	},
	events: {
		title: 'События',
		subtitle: 'Что записала Windows, сначала новые.',
		channel: 'Канал',
		allChannels: 'Система и Приложение',
		from: 'С',
		to: 'По',
		load: 'Загрузить',
		keyword: 'Искать по всем столбцам…',
		columnFilter: 'фильтр столбца',
		clearColumnFilters: 'Сбросить фильтры столбцов',
		loaded: (shown: number, total: number) => {
			const plural = (n: number, one: string, few: string, many: string) =>
				n % 10 === 1 && n % 100 !== 11
					? one
					: n % 10 >= 2 && n % 10 <= 4 && (n % 100 < 12 || n % 100 > 14)
						? few
						: many;
			return shown === total
				? `${total} ${plural(total, 'событие', 'события', 'событий')}`
				: `${shown} из ${total} ${plural(total, 'события', 'событий', 'событий')}`;
		},
		elapsed: (ms: number) => `прочитано за ${ms} мс`,
		truncated: 'больше лимита строк — сузьте фильтр или поднимите лимит в настройках',
		securityHint:
			'Закройте OpenEventViewer и запустите его заново от имени администратора — или выберите канал, которому это не нужно.',
		empty: 'Совпадений нет.',
		search: 'Найти это событие в интернете',
		resize: (column: string) => `Изменить ширину столбца «${column}»`,
		filters: {
			search: 'Поиск…',
			noMatch: 'Совпадений нет.',
			clear: 'Сбросить этот фильтр',
			chosen: (count: number) => `Выбрано: ${count}`,
			after: (time: string) => `после ${time}`,
			before: (time: string) => `до ${time}`,
			timeHint: 'Местное время — те же часы, что показывает таблица.',
			numberHint: 'Числа там не нашлось.',
			notUnderstood: (parts: string) => `Не удалось разобрать: ${parts}`,
			helpAny: 'любое из них',
			helpCompare: 'больше, меньше',
			helpRange: 'диапазон, границы включаются',
			helpNot: 'всё, кроме'
		},
		overTime: 'Динамика',
		andMore: (kinds: number, count: number) => {
			const plural = (n: number, one: string, few: string, many: string) =>
				n % 10 === 1 && n % 100 !== 11
					? one
					: n % 10 >= 2 && n % 10 <= 4 && (n % 100 < 12 || n % 100 > 14)
						? few
						: many;
			return `ещё ${kinds} ${plural(kinds, 'вид', 'вида', 'видов')}, всего ${count}`;
		},
		bucketSize: (minutes: number) => {
			const plural = (n: number, one: string, few: string, many: string) =>
				n % 10 === 1 && n % 100 !== 11
					? one
					: n % 10 >= 2 && n % 10 <= 4 && (n % 100 < 12 || n % 100 > 14)
						? few
						: many;
			if (minutes >= 1440) {
				const days = minutes / 1440;
				return `один столбик на ${days} ${plural(days, 'день', 'дня', 'дней')}`;
			}
			if (minutes >= 60) {
				const hours = minutes / 60;
				return `один столбик на ${hours} ${plural(hours, 'час', 'часа', 'часов')}`;
			}
			return `один столбик на ${minutes} ${plural(minutes, 'минуту', 'минуты', 'минут')}`;
		},
		bucketCount: (total: number, errors: number, warnings: number) => {
			const plural = (n: number, one: string, few: string, many: string) =>
				n % 10 === 1 && n % 100 !== 11
					? one
					: n % 10 >= 2 && n % 10 <= 4 && (n % 100 < 12 || n % 100 > 14)
						? few
						: many;
			const events = `${total} ${plural(total, 'событие', 'события', 'событий')}`;
			const parts = [
				errors > 0 && `${errors} ${plural(errors, 'ошибка', 'ошибки', 'ошибок')}`,
				warnings > 0 &&
					`${warnings} ${plural(warnings, 'предупреждение', 'предупреждения', 'предупреждений')}`
			].filter(Boolean);
			return parts.length === 0 ? events : `${events}, из них ${parts.join(' и ')}`;
		},
		columns: {
			level: 'Уровень',
			time: 'Время',
			provider: 'Поставщик',
			eventId: 'ID',
			task: 'Задача',
			channel: 'Канал',
			computer: 'Компьютер',
			message: 'Сообщение'
		}
	},
	diagnose: {
		title: 'Диагностика',
		subtitle:
			'Просматривает журнал в поисках событий, которые машина записывает, когда что-то пошло не так, а затем достаёт четверть часа вокруг одного из них.',
		days: (count: number) => {
			const plural = (n: number, one: string, few: string, many: string) =>
				n % 10 === 1 && n % 100 !== 11
					? one
					: n % 10 >= 2 && n % 10 <= 4 && (n % 100 < 12 || n % 100 > 14)
						? few
						: many;
			return count === 1
				? 'Последний день'
				: `Последние ${count} ${plural(count, 'день', 'дня', 'дней')}`;
		},
		scan: 'Сканировать',
		scanning: 'Сканирование…',
		intro:
			'Пока ничего не сканировалось. Выберите отрезок времени выше и нажмите «Сканировать»; каждая находка — сбой, зависание, ошибка диска, замедленный процессор — появится здесь как инцидент, который можно открыть.',
		pick: 'Откройте инцидент, чтобы увидеть всё, что машина записала за четверть часа вокруг него.',
		nothing:
			'Ничего не найдено. Просканируйте более длинный отрезок — или считайте это хорошей новостью.',
		window: (from: string, to: string) => `${from} — ${to}`,
		inWindow: (count: number) => {
			const plural = (n: number, one: string, few: string, many: string) =>
				n % 10 === 1 && n % 100 !== 11
					? one
					: n % 10 >= 2 && n % 10 <= 4 && (n % 100 < 12 || n % 100 > 14)
						? few
						: many;
			return `${count} ${plural(count, 'событие', 'события', 'событий')} в этом окне`;
		},
		kinds: {
			unexpectedShutdown: 'Неожиданное завершение работы',
			bugCheck: 'Синий экран',
			hardwareError: 'Аппаратная ошибка',
			appHang: 'Зависание программы',
			appCrash: 'Сбой программы',
			serviceFailure: 'Сбой службы',
			diskError: 'Ошибка диска',
			ntfs: 'Файловая система',
			displayTdr: 'Сброс видеодрайвера',
			processorPower: 'Процессор замедлен'
		}
	},
	log: {
		title: 'Журнал',
		subtitle: 'Что делала программа, новые записи внизу. Ничего из этого не пишется на диск.',
		filter: 'Фильтровать сообщения…',
		level: 'Уровень',
		levels: {
			all: 'Все уровни',
			error: 'Ошибки',
			warning: 'Предупреждения',
			info: 'Инфо',
			debug: 'Отладка'
		},
		clear: 'Очистить журнал',
		empty: 'Пока ничего не записано.',
		count: (shown: number, total: number) => {
			const plural = (n: number, one: string, few: string, many: string) =>
				n % 10 === 1 && n % 100 !== 11
					? one
					: n % 10 >= 2 && n % 10 <= 4 && (n % 100 < 12 || n % 100 > 14)
						? few
						: many;
			return `${shown} из ${total} ${plural(total, 'записи', 'записей', 'записей')}`;
		}
	},
	info: {
		title: 'О программе',
		subtitle: 'Что это за программа и на чём она построена.',
		appBody:
			'Читает журналы событий Windows и отфильтровывает их до главного — без учётной записи, без выгрузки, без телеметрии.',
		offline: 'Всё работает на этом компьютере. Ничего не выгружается, телеметрия не собирается.',
		appLicense: 'OpenEventViewer распространяется по лицензии MIT.',
		thirdParty: 'Сторонние компоненты',
		thirdPartyBody: (total: number, vendored: number, crates: number, npm: number) => {
			const plural = (n: number, one: string, few: string, many: string) =>
				n % 10 === 1 && n % 100 !== 11
					? one
					: n % 10 >= 2 && n % 10 <= 4 && (n % 100 < 12 || n % 100 > 14)
						? few
						: many;
			return `С этой программой поставляется ${total} ${plural(total, 'компонент', 'компонента', 'компонентов')}: ${vendored} ${plural(vendored, 'встроенный бинарный файл', 'встроенных бинарных файла', 'встроенных бинарных файлов')}, ${crates} Rust-${plural(crates, 'крейт', 'крейта', 'крейтов')}, ${npm} npm-${plural(npm, 'пакет', 'пакета', 'пакетов')}.`;
		},
		shipped:
			'Полные тексты лицензий входят в установщик как THIRD_PARTY_LICENSES.txt. MIT, BSD и ISC требуют, чтобы уведомление сопровождало двоичную сборку — одной ссылки для этого недостаточно.',
		filter: 'Фильтровать компоненты…',
		showTexts: 'Показать тексты лицензий',
		hideTexts: 'Скрыть тексты лицензий',
		noMatch: 'Ни один компонент не подходит.',
		redistributed: 'поставляется в виде бинарного файла',
		noOwnText: 'без собственного текста',
		withoutText: (count: number) => {
			const plural = (n: number, one: string, few: string, many: string) =>
				n % 10 === 1 && n % 100 !== 11
					? one
					: n % 10 >= 2 && n % 10 <= 4 && (n % 100 < 12 || n % 100 > 14)
						? few
						: many;
			return `${count} ${plural(count, 'компонент не опубликовал', 'компонента не опубликовали', 'компонентов не опубликовали')} собственного файла лицензии; действует канонический текст указанной лицензии.`;
		},
		material: 'Ваши журналы',
		materialBody:
			'Журналы событий остаются там, где их хранит Windows. Программа их читает и никогда в них не пишет.'
	},
	detail: {
		general: 'Общие',
		data: 'Данные события',
		xml: 'XML',
		search: 'Искать в интернете',
		copy: 'Копировать',
		copied: 'Скопировано',
		close: 'Закрыть панель сведений',
		recordId: 'Запись',
		keywords: 'Ключевые слова',
		noData: 'У этого события нет собственных данных.'
	},
	updater: {
		title: 'Обновления',
		body: (version: string) => `Версия ${version}. Проверяется один раз при запуске.`,
		check: 'Проверить сейчас',
		checking: 'Проверка…',
		upToDate: 'актуальная версия',
		available: (version: string) => `Доступна версия ${version}`,
		downloading: (percent: number | null) =>
			percent === null ? 'Загрузка…' : `Загрузка — ${percent}%`,
		ready: 'Установлено — перезапуск',
		install: 'Установить и перезапустить',
		failed: 'Не удалось проверить обновления.'
	},
	settings: {
		title: 'Настройки',
		appearance: 'Оформление',
		appearanceBody: 'Тема окна программы.',
		system: 'Системная',
		light: 'Светлая',
		dark: 'Тёмная',
		colours: 'Цвета',
		coloursBody: 'Палитра, из которой рисуется каждый экран.',
		presets: {
			default: 'Стандартная',
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
		language: 'Язык',
		languageBody:
			'Интерфейс программы. Текст событий остаётся на том языке, на котором его записала Windows.',
		eventsRows: 'События: строк за один запрос',
		eventsRowsBody:
			'Каждое событие требует запроса текста сообщения у поставщика, поэтому большее число — это дольше ждать, а не только длиннее список.',
		eventsRowsValue: (rows: number) => {
			const plural = (n: number, one: string, few: string, many: string) =>
				n % 10 === 1 && n % 100 !== 11
					? one
					: n % 10 >= 2 && n % 10 <= 4 && (n % 100 < 12 || n % 100 > 14)
						? few
						: many;
			return `${rows.toLocaleString('ru')} ${plural(rows, 'строка', 'строки', 'строк')}`;
		},
		showLogs: 'Показывать журнал в боковой панели',
		showLogsBody: 'Добавляет в навигацию пункт «Журнал».',
		debugLogging: 'Записывать отладочные сообщения',
		debugLoggingBody:
			'Многословно. По умолчанию выключено, потому что отладочные записи вытесняют именно те, которые вы искали.'
	}
};
