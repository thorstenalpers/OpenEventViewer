import type { Translations } from './en';

export const hi: Translations = {
	sidebar: {
		tagline: 'Windows इवेंट लॉग',
		events: 'इवेंट',
		diagnose: 'निदान',
		log: 'लॉग',
		settings: 'सेटिंग्स',
		info: 'जानकारी',
		toLight: 'हल्की थीम पर जाएँ',
		toDark: 'गहरी थीम पर जाएँ',
		sections: 'अनुभाग',
		collapse: 'साइडबार समेटें',
		expand: 'साइडबार खोलें'
	},
	common: {
		loading: 'लोड हो रहा है…',
		mockHost: 'Mock होस्ट — कोई Tauri बैकएंड नहीं। इस पेज का डेटा नमूना डेटा है।'
	},
	events: {
		title: 'इवेंट',
		subtitle: 'Windows ने जो दर्ज किया, सबसे नया पहले।',
		channel: 'चैनल',
		allChannels: 'System और Application',
		from: 'से',
		to: 'तक',
		load: 'लोड करें',
		keyword: 'हर कॉलम में खोजें…',
		columnFilter: 'कॉलम फ़िल्टर',
		clearColumnFilters: 'कॉलम फ़िल्टर हटाएँ',
		loaded: (shown: number, total: number) =>
			shown === total ? `${total} इवेंट` : `${total} में से ${shown} इवेंट`,
		elapsed: (ms: number) => `${ms} ms में पढ़ा गया`,
		truncated: 'पंक्ति सीमा से अधिक — फ़िल्टर को सीमित करें या सेटिंग्स में सीमा बढ़ाएँ',
		securityHint:
			'OpenEventViewer बंद करके उसे व्यवस्थापक के रूप में फिर से शुरू करें, या कोई ऐसा चैनल चुनें जिसे इसकी ज़रूरत नहीं।',
		empty: 'कुछ भी मेल नहीं खाता।',
		search: 'इस इवेंट को वेब पर खोजें',
		resize: (column: string) => `${column} कॉलम की चौड़ाई बदलें`,
		filters: {
			search: 'खोजें…',
			noMatch: 'कुछ भी मेल नहीं खाता।',
			clear: 'यह फ़िल्टर हटाएँ',
			chosen: (count: number) => `${count} चुने गए`,
			after: (time: string) => `${time} के बाद`,
			before: (time: string) => `${time} से पहले`,
			timeHint: 'स्थानीय समय — वही घड़ी जो तालिका दिखाती है।',
			numberHint: 'उसमें कोई संख्या नहीं थी।',
			notUnderstood: (parts: string) => `समझ नहीं आया: ${parts}`,
			helpAny: 'इनमें से कोई भी',
			helpCompare: 'से अधिक, से कम',
			helpRange: 'एक सीमा, दोनों सिरे शामिल',
			helpNot: 'इसके सिवा सब कुछ'
		},
		overTime: 'समय के साथ',
		andMore: (kinds: number, count: number) => `${kinds} और प्रकार, कुल ${count}`,
		bucketSize: (minutes: number) =>
			minutes >= 1440
				? `हर ${minutes / 1440} ${minutes === 1440 ? 'दिन' : 'दिनों'} पर एक बार`
				: minutes >= 60
					? `हर ${minutes / 60} ${minutes === 60 ? 'घंटे' : 'घंटों'} पर एक बार`
					: `हर ${minutes} ${minutes === 1 ? 'मिनट' : 'मिनटों'} पर एक बार`,
		bucketCount: (total: number, errors: number, warnings: number) => {
			const events = `${total} इवेंट`;
			const parts = [
				errors > 0 && `${errors} ${errors === 1 ? 'त्रुटि' : 'त्रुटियाँ'}`,
				warnings > 0 && `${warnings} ${warnings === 1 ? 'चेतावनी' : 'चेतावनियाँ'}`
			].filter(Boolean);
			return parts.length === 0 ? events : `${events}, उनमें से ${parts.join(' और ')}`;
		},
		columns: {
			level: 'स्तर',
			time: 'समय',
			provider: 'स्रोत',
			eventId: 'ID',
			task: 'कार्य',
			channel: 'चैनल',
			computer: 'कंप्यूटर',
			message: 'संदेश'
		}
	},
	diagnose: {
		title: 'निदान',
		subtitle:
			'लॉग में उन इवेंट को खोजता है जो मशीन कुछ गड़बड़ होने पर लिखती है, फिर उनमें से किसी एक के आसपास का पौना घंटा निकालता है।',
		days: (count: number) => (count === 1 ? 'पिछला दिन' : `पिछले ${count} दिन`),
		scan: 'स्कैन करें',
		scanning: 'स्कैन हो रहा है…',
		intro:
			'अभी तक कुछ स्कैन नहीं हुआ। ऊपर एक अवधि चुनें और स्कैन दबाएँ; हर खोज — कोई क्रैश, कोई फ़्रीज़, कोई डिस्क त्रुटि, कोई धीमा किया गया प्रोसेसर — यहाँ एक घटना के रूप में दिखेगी जिसे आप खोल सकते हैं।',
		pick: 'किसी घटना को खोलें और देखें कि मशीन ने उसके आसपास के पंद्रह मिनटों में क्या-क्या लिखा।',
		nothing: 'कुछ नहीं मिला। लंबी अवधि स्कैन करें, या इसे अच्छी खबर मानें।',
		window: (from: string, to: string) => `${from} — ${to}`,
		inWindow: (count: number) => `इस अवधि में ${count} इवेंट`,
		kinds: {
			unexpectedShutdown: 'अनपेक्षित शटडाउन',
			bugCheck: 'बग चेक',
			hardwareError: 'हार्डवेयर त्रुटि',
			appHang: 'एप्लिकेशन अटका',
			appCrash: 'एप्लिकेशन क्रैश',
			serviceFailure: 'सेवा विफलता',
			diskError: 'डिस्क त्रुटि',
			ntfs: 'फ़ाइल सिस्टम',
			displayTdr: 'डिस्प्ले ड्राइवर रीसेट',
			processorPower: 'प्रोसेसर धीमा किया गया'
		}
	},
	log: {
		title: 'लॉग',
		subtitle: 'ऐप ने जो किया, सबसे नया अंत में। यहाँ कुछ भी डिस्क पर नहीं लिखा जाता।',
		filter: 'संदेश फ़िल्टर करें…',
		level: 'स्तर',
		levels: {
			all: 'सभी स्तर',
			error: 'त्रुटियाँ',
			warning: 'चेतावनियाँ',
			info: 'जानकारी',
			debug: 'डीबग'
		},
		clear: 'लॉग खाली करें',
		empty: 'अभी तक कुछ लॉग नहीं हुआ।',
		count: (shown: number, total: number) => `${total} में से ${shown} प्रविष्टियाँ`
	},
	info: {
		title: 'जानकारी',
		subtitle: 'यह ऐप क्या है, और किस पर बना है।',
		appBody:
			'Windows इवेंट लॉग पढ़ें और उन्हें छानकर वही देखें जो मायने रखता है — कोई खाता नहीं, कोई अपलोड नहीं, कोई टेलीमेट्री नहीं।',
		offline:
			'सब कुछ इसी मशीन पर चलता है। कुछ भी अपलोड नहीं होता, और कोई टेलीमेट्री एकत्र नहीं की जाती।',
		appLicense: 'OpenEventViewer MIT लाइसेंस के अंतर्गत है।',
		thirdParty: 'तृतीय-पक्ष घटक',
		thirdPartyBody: (total: number, vendored: number, crates: number, npm: number) =>
			`इस ऐप के साथ ${total} घटक आते हैं: ${vendored} साथ दी गई बाइनरी, ${crates} Rust crates, ${npm} npm पैकेज।`,
		shipped:
			'पूरे लाइसेंस पाठ इंस्टॉलर के अंदर THIRD_PARTY_LICENSES.txt के रूप में शामिल हैं। MIT, BSD और ISC सभी माँग करते हैं कि सूचना बाइनरी के साथ रहे, इसलिए केवल एक लिंक पर्याप्त नहीं होता।',
		filter: 'घटक फ़िल्टर करें…',
		showTexts: 'लाइसेंस पाठ दिखाएँ',
		hideTexts: 'लाइसेंस पाठ छिपाएँ',
		noMatch: 'कोई घटक मेल नहीं खाता।',
		redistributed: 'बाइनरी के रूप में शामिल',
		noOwnText: 'अपना कोई पाठ नहीं',
		withoutText: (count: number) =>
			`${count} घटकों ने अपनी कोई लाइसेंस फ़ाइल प्रकाशित नहीं की; नामित लाइसेंस का मानक पाठ लागू होता है।`,
		material: 'आपके लॉग',
		materialBody:
			'इवेंट लॉग वहीं रहते हैं जहाँ Windows उन्हें रखता है। यह ऐप उन्हें केवल पढ़ता है और उनमें कभी नहीं लिखता।'
	},
	detail: {
		general: 'सामान्य',
		data: 'इवेंट डेटा',
		xml: 'XML',
		search: 'वेब पर खोजें',
		copy: 'कॉपी करें',
		copied: 'कॉपी हो गया',
		close: 'विवरण फलक बंद करें',
		recordId: 'रिकॉर्ड',
		keywords: 'कीवर्ड',
		noData: 'इस इवेंट का अपना कोई डेटा नहीं है।'
	},
	updater: {
		title: 'अपडेट',
		body: (version: string) => `संस्करण ${version}। शुरुआत में एक बार जाँचा गया।`,
		check: 'अभी जाँचें',
		checking: 'जाँच हो रही है…',
		upToDate: 'नवीनतम है',
		available: (version: string) => `${version} उपलब्ध है`,
		downloading: (percent: number | null) =>
			percent === null ? 'डाउनलोड हो रहा है…' : `डाउनलोड हो रहा है — ${percent}%`,
		ready: 'इंस्टॉल हो गया — पुनः शुरू हो रहा है',
		install: 'इंस्टॉल करें और पुनः शुरू करें',
		failed: 'अपडेट की जाँच विफल रही।'
	},
	settings: {
		title: 'सेटिंग्स',
		appearance: 'रूप-रंग',
		appearanceBody: 'ऐप विंडो की थीम।',
		system: 'सिस्टम',
		light: 'हल्की',
		dark: 'गहरी',
		colours: 'रंग',
		coloursBody: 'वह पैलेट जिससे हर दृश्य बनता है।',
		presets: {
			default: 'डिफ़ॉल्ट',
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
		language: 'भाषा',
		languageBody:
			'ऐप का इंटरफ़ेस। इवेंट का पाठ उसी भाषा में रहता है जिसमें Windows ने उसे दर्ज किया।',
		eventsRows: 'इवेंट: लोड होने वाली पंक्तियाँ',
		eventsRowsBody:
			'हर इवेंट के लिए प्रकाशक से एक संदेश खोजना पड़ता है, इसलिए बड़ी संख्या का मतलब लंबी सूची नहीं बल्कि लंबा इंतज़ार है।',
		eventsRowsValue: (rows: number) => `${rows.toLocaleString('hi')} पंक्तियाँ`,
		showLogs: 'साइडबार में लॉग दिखाएँ',
		showLogsBody: 'नेविगेशन में एक लॉग प्रविष्टि जोड़ता है।',
		debugLogging: 'डीबग प्रविष्टियाँ दर्ज करें',
		debugLoggingBody:
			'विस्तृत। डिफ़ॉल्ट रूप से बंद, क्योंकि डीबग प्रविष्टियाँ ठीक उन्हीं को दबा देती हैं जिन्हें आप खोजने आए थे।'
	}
};
