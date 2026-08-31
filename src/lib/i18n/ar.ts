import type { Translations } from './en';

export const ar: Translations = {
	sidebar: {
		tagline: 'سجلات أحداث Windows',
		events: 'الأحداث',
		diagnose: 'التشخيص',
		log: 'السجل',
		settings: 'الإعدادات',
		info: 'معلومات',
		toLight: 'التبديل إلى السمة الفاتحة',
		toDark: 'التبديل إلى السمة الداكنة',
		sections: 'الأقسام',
		collapse: 'طي الشريط الجانبي',
		expand: 'توسيع الشريط الجانبي'
	},
	common: {
		loading: 'جارٍ التحميل…',
		mockHost: 'مضيف تجريبي — لا توجد واجهة خلفية Tauri. البيانات في هذه الصفحة بيانات وهمية.'
	},
	events: {
		title: 'الأحداث',
		subtitle: 'ما سجّله Windows، الأحدث أولاً.',
		channel: 'القناة',
		allChannels: 'النظام والتطبيق',
		from: 'من',
		to: 'إلى',
		load: 'تحميل',
		keyword: 'البحث في كل الأعمدة…',
		columnFilter: 'عامل تصفية العمود',
		clearColumnFilters: 'مسح عوامل تصفية الأعمدة',
		loaded: (shown: number, total: number) =>
			shown === total ? `${total} من الأحداث` : `${shown} من أصل ${total} من الأحداث`,
		elapsed: (ms: number) => `تمت القراءة في ${ms} مللي ثانية`,
		truncated: 'أكثر من حد الصفوف — ضيّق عامل التصفية أو ارفع الحد في الإعدادات',
		securityHint: 'أغلق OpenEventViewer وأعد تشغيله كمسؤول، أو اختر قناة لا تتطلب ذلك.',
		empty: 'لا شيء يطابق.',
		search: 'البحث في الويب عن هذا الحدث',
		resize: (column: string) => `تغيير عرض العمود ${column}`,
		filters: {
			search: 'بحث…',
			noMatch: 'لا شيء يطابق.',
			clear: 'مسح عامل التصفية هذا',
			chosen: (count: number) => `تم اختيار ${count}`,
			after: (time: string) => `بعد ${time}`,
			before: (time: string) => `قبل ${time}`,
			timeHint: 'التوقيت المحلي، نفس الساعة التي يعرضها الجدول.',
			numberHint: 'لم يكن هناك أي رقم.',
			notUnderstood: (parts: string) => `غير مفهوم: ${parts}`,
			helpAny: 'أي منها',
			helpCompare: 'أكبر من، أصغر من',
			helpRange: 'نطاق يشمل الطرفين',
			helpNot: 'كل شيء ما عدا'
		},
		overTime: 'على مدار الوقت',
		andMore: (kinds: number, count: number) =>
			kinds === 1 ? `نوع إضافي واحد، ${count} إجمالاً` : `${kinds} أنواع إضافية، ${count} إجمالاً`,
		bucketSize: (minutes: number) =>
			minutes >= 1440
				? minutes === 1440
					? 'عمود واحد لكل يوم'
					: `عمود واحد لكل ${minutes / 1440} أيام`
				: minutes >= 60
					? minutes === 60
						? 'عمود واحد لكل ساعة'
						: `عمود واحد لكل ${minutes / 60} ساعات`
					: minutes === 1
						? 'عمود واحد لكل دقيقة'
						: `عمود واحد لكل ${minutes} دقائق`,
		bucketCount: (total: number, errors: number, warnings: number) => {
			const events = total === 1 ? 'حدث واحد' : `${total} من الأحداث`;
			const parts = [
				errors > 0 && (errors === 1 ? 'خطأ واحد' : `${errors} أخطاء`),
				warnings > 0 && (warnings === 1 ? 'تحذير واحد' : `${warnings} تحذيرات`)
			].filter(Boolean);
			return parts.length === 0 ? events : `${events}، منها ${parts.join(' و')}`;
		},
		columns: {
			level: 'المستوى',
			time: 'الوقت',
			provider: 'الموفر',
			eventId: 'المعرّف',
			task: 'المهمة',
			channel: 'القناة',
			computer: 'الكمبيوتر',
			message: 'الرسالة'
		}
	},
	diagnose: {
		title: 'التشخيص',
		subtitle:
			'يفحص السجل بحثًا عن الأحداث التي يكتبها الجهاز عندما يحدث خطأ ما، ثم يجلب ربع الساعة المحيط بأحدها.',
		days: (count: number) => (count === 1 ? 'آخر يوم' : `آخر ${count} أيام`),
		scan: 'فحص',
		scanning: 'جارٍ الفحص…',
		intro:
			'لم يُفحص شيء بعد. اختر فترة أعلاه واضغط على فحص؛ كل ما يُعثر عليه — انهيار، تجمّد، خطأ في القرص، معالج مخفَّضة سرعته — يظهر هنا كواقعة يمكنك فتحها.',
		pick: 'افتح واقعة لترى كل ما كتبه الجهاز في ربع الساعة المحيط بها.',
		nothing: 'لم يُعثر على شيء. افحص فترة أطول، أو اعتبر ذلك خبرًا سارًا.',
		window: (from: string, to: string) => `${from} — ${to}`,
		inWindow: (count: number) =>
			count === 1 ? 'حدث واحد في النافذة الزمنية' : `${count} من الأحداث في النافذة الزمنية`,
		kinds: {
			unexpectedShutdown: 'إيقاف تشغيل غير متوقع',
			bugCheck: 'شاشة زرقاء',
			hardwareError: 'خطأ في الأجهزة',
			appHang: 'توقف تطبيق عن الاستجابة',
			appCrash: 'انهيار تطبيق',
			serviceFailure: 'فشل خدمة',
			diskError: 'خطأ في القرص',
			ntfs: 'نظام الملفات',
			displayTdr: 'إعادة تعيين برنامج تشغيل العرض',
			processorPower: 'خفض سرعة المعالج'
		}
	},
	log: {
		title: 'السجل',
		subtitle: 'ما فعله التطبيق، الأحدث أخيرًا. لا شيء هنا يُكتب على القرص.',
		filter: 'تصفية الرسائل…',
		level: 'المستوى',
		levels: {
			all: 'كل المستويات',
			error: 'أخطاء',
			warning: 'تحذيرات',
			info: 'معلومات',
			debug: 'تصحيح'
		},
		clear: 'مسح السجل',
		empty: 'لم يُسجَّل شيء بعد.',
		count: (shown: number, total: number) => `${shown} من أصل ${total} من الإدخالات`
	},
	info: {
		title: 'معلومات',
		subtitle: 'ما هذا التطبيق، وعلامَ بُني.',
		appBody:
			'اقرأ سجلات أحداث Windows وصفِّها إلى ما يهم — بلا حساب، بلا رفع، بلا بيانات قياس عن بُعد.',
		offline: 'كل شيء يعمل على هذا الجهاز. لا يُرفع شيء، ولا تُجمع أي بيانات قياس عن بُعد.',
		appLicense: 'OpenEventViewer مرخَّص بترخيص MIT.',
		thirdParty: 'مكونات الجهات الخارجية',
		thirdPartyBody: (total: number, vendored: number, crates: number, npm: number) =>
			`يُشحن مع هذا التطبيق ${total} من المكونات: ${vendored} من الملفات الثنائية المضمّنة، و${crates} من حزم Rust، و${npm} من حزم npm.`,
		shipped:
			'نصوص التراخيص الكاملة تُشحن داخل المثبِّت باسم THIRD_PARTY_LICENSES.txt. تشترط تراخيص MIT وBSD وISC أن يرافق الإشعارُ الملفَ الثنائي، لذا لا يكفي وضع رابط.',
		filter: 'تصفية المكونات…',
		showTexts: 'إظهار نصوص التراخيص',
		hideTexts: 'إخفاء نصوص التراخيص',
		noMatch: 'لا يوجد مكوّن مطابق.',
		redistributed: 'يُشحن كملف ثنائي',
		noOwnText: 'بلا نص خاص به',
		withoutText: (count: number) =>
			`${count} من المكونات لم تنشر ملف ترخيص خاصًا بها؛ يسري النص المعياري للترخيص المذكور.`,
		material: 'سجلاتك',
		materialBody:
			'تبقى سجلات الأحداث حيث يحتفظ بها Windows. هذا التطبيق يقرؤها ولا يكتب فيها أبدًا.'
	},
	detail: {
		general: 'عام',
		data: 'بيانات الحدث',
		xml: 'XML',
		search: 'البحث في الويب',
		copy: 'نسخ',
		copied: 'تم النسخ',
		close: 'إغلاق جزء التفاصيل',
		recordId: 'السجل',
		keywords: 'الكلمات الأساسية',
		noData: 'هذا الحدث لا يحمل بيانات خاصة به.'
	},
	updater: {
		title: 'التحديثات',
		body: (version: string) => `الإصدار ${version}. يتم التحقق مرة واحدة عند بدء التشغيل.`,
		check: 'التحقق الآن',
		checking: 'جارٍ التحقق…',
		upToDate: 'محدَّث',
		available: (version: string) => `الإصدار ${version} متوفر`,
		downloading: (percent: number | null) =>
			percent === null ? 'جارٍ التنزيل…' : `جارٍ التنزيل — ${percent}%`,
		ready: 'تم التثبيت — تتم إعادة التشغيل',
		install: 'التثبيت وإعادة التشغيل',
		failed: 'فشل التحقق من التحديثات.'
	},
	settings: {
		title: 'الإعدادات',
		appearance: 'المظهر',
		appearanceBody: 'سمة نافذة التطبيق.',
		system: 'النظام',
		light: 'فاتح',
		dark: 'داكن',
		colours: 'الألوان',
		coloursBody: 'اللوحة التي تُرسم منها كل طريقة عرض.',
		presets: {
			default: 'افتراضي',
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
		language: 'اللغة',
		languageBody: 'واجهة التطبيق. يحتفظ نص الحدث باللغة التي سجّله بها Windows.',
		eventsRows: 'الأحداث: عدد الصفوف المحمَّلة',
		eventsRowsBody:
			'كل حدث يكلّف الناشر عملية بحث عن رسالة، لذا فالرقم الأكبر يعني انتظارًا أطول لا قائمة أطول فحسب.',
		eventsRowsValue: (rows: number) => `${rows.toLocaleString('ar')} من الصفوف`,
		showLogs: 'إظهار السجل في الشريط الجانبي',
		showLogsBody: 'يضيف إدخال السجل إلى شريط التنقل.',
		debugLogging: 'تسجيل إدخالات التصحيح',
		debugLoggingBody:
			'مُسهب. متوقف افتراضيًا، لأن إدخالات التصحيح تزاحم الإدخالات التي كنت تبحث عنها.'
	}
};
