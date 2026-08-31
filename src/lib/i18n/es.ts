import type { Translations } from './en';

export const es: Translations = {
	sidebar: {
		tagline: 'Registros de eventos de Windows',
		events: 'Eventos',
		diagnose: 'Diagnóstico',
		log: 'Registro',
		settings: 'Configuración',
		info: 'Información',
		toLight: 'Cambiar al tema claro',
		toDark: 'Cambiar al tema oscuro',
		sections: 'Secciones',
		collapse: 'Contraer la barra lateral',
		expand: 'Expandir la barra lateral'
	},
	common: {
		loading: 'Cargando…',
		mockHost: 'Host simulado — sin backend de Tauri. Los datos de esta página son datos de prueba.'
	},
	events: {
		title: 'Eventos',
		subtitle: 'Lo que Windows ha registrado, lo más reciente primero.',
		channel: 'Canal',
		allChannels: 'Sistema y Aplicación',
		from: 'Desde',
		to: 'Hasta',
		load: 'Cargar',
		span: (from: string, to: string) => `de ${from} a ${to}`,
		keyword: 'Buscar en todas las columnas…',
		columnFilter: 'filtro de columna',
		clearColumnFilters: 'Quitar los filtros de columna',
		loaded: (shown: number, total: number) =>
			shown === total ? `${total} eventos` : `${shown} de ${total} eventos`,
		elapsed: (ms: number) => `leído en ${ms} ms`,
		truncated: 'más que el límite de filas — acota el filtro o súbelo en Configuración',
		securityHint:
			'Cierra OpenEventViewer y vuelve a iniciarlo como administrador, o elige un canal que no lo necesite.',
		empty: 'Nada coincide.',
		search: 'Buscar este evento en la web',
		resize: (column: string) => `Cambiar el ancho de la columna ${column}`,
		filters: {
			search: 'Buscar…',
			noMatch: 'Nada coincide.',
			clear: 'Quitar este filtro',
			chosen: (count: number) => (count === 1 ? `${count} elegido` : `${count} elegidos`),
			after: (time: string) => `después de ${time}`,
			before: (time: string) => `antes de ${time}`,
			timeHint: 'Hora local, el mismo reloj que muestra la tabla.',
			numberHint: 'Ahí no había ningún número.',
			notUnderstood: (parts: string) => `No se entendió: ${parts}`,
			helpAny: 'cualquiera de ellos',
			helpCompare: 'mayor que, menor que',
			helpRange: 'un rango, extremos incluidos',
			helpNot: 'todo excepto'
		},
		overTime: 'Evolución temporal',
		andMore: (kinds: number, count: number) =>
			`${kinds} tipo${kinds === 1 ? '' : 's'} más, ${count} en total`,
		bucketSize: (minutes: number) =>
			minutes >= 1440
				? `una barra por ${minutes / 1440} día${minutes === 1440 ? '' : 's'}`
				: minutes >= 60
					? `una barra por ${minutes / 60} hora${minutes === 60 ? '' : 's'}`
					: `una barra por ${minutes} minuto${minutes === 1 ? '' : 's'}`,
		bucketCount: (total: number, errors: number, warnings: number) => {
			const events = `${total} evento${total === 1 ? '' : 's'}`;
			const parts = [
				errors > 0 && `${errors} error${errors === 1 ? '' : 'es'}`,
				warnings > 0 && `${warnings} advertencia${warnings === 1 ? '' : 's'}`
			].filter(Boolean);
			return parts.length === 0 ? events : `${events}, de ellos ${parts.join(' y ')}`;
		},
		columns: {
			level: 'Nivel',
			time: 'Hora',
			provider: 'Proveedor',
			eventId: 'ID',
			task: 'Tarea',
			channel: 'Canal',
			computer: 'Equipo',
			message: 'Mensaje'
		}
	},
	diagnose: {
		title: 'Diagnóstico',
		subtitle:
			'Recorre el registro en busca de los eventos que una máquina escribe cuando algo salió mal y luego extrae el cuarto de hora en torno a uno de ellos.',
		days: (count: number) => (count === 1 ? 'Último día' : `Últimos ${count} días`),
		scan: 'Buscar',
		scanning: 'Buscando…',
		intro:
			'Todavía no se ha buscado nada. Elige arriba un intervalo y pulsa Buscar; cada hallazgo — un bloqueo, un cuelgue, un error de disco, un procesador limitado — aparece aquí como un incidente que puedes abrir.',
		pick: 'Abre un incidente para ver todo lo que la máquina escribió en el cuarto de hora a su alrededor.',
		nothing:
			'No se encontró nada. Busca en un intervalo más largo, o tómalo como una buena noticia.',
		window: (from: string, to: string) => `${from} — ${to}`,
		inWindow: (count: number) => `${count} evento${count === 1 ? '' : 's'} en la ventana`,
		kinds: {
			unexpectedShutdown: 'Apagado inesperado',
			bugCheck: 'Pantalla azul',
			hardwareError: 'Error de hardware',
			appHang: 'Aplicación no responde',
			appCrash: 'Bloqueo de aplicación',
			serviceFailure: 'Error de servicio',
			diskError: 'Error de disco',
			ntfs: 'Sistema de archivos',
			displayTdr: 'Controlador de pantalla restablecido',
			processorPower: 'Procesador limitado'
		}
	},
	log: {
		title: 'Registro',
		subtitle: 'Lo que la app ha hecho, lo más reciente al final. Nada de esto se escribe en disco.',
		filter: 'Filtrar mensajes…',
		level: 'Nivel',
		levels: {
			all: 'Todos los niveles',
			error: 'Errores',
			warning: 'Advertencias',
			info: 'Información',
			debug: 'Depuración'
		},
		clear: 'Vaciar el registro',
		empty: 'Todavía no hay nada registrado.',
		count: (shown: number, total: number) => `${shown} de ${total} entradas`
	},
	info: {
		title: 'Información',
		subtitle: 'Qué es esta app y sobre qué está construida.',
		appBody:
			'Lee los registros de eventos de Windows y fíltralos hasta lo que importa — sin cuenta, sin subir nada, sin telemetría.',
		offline: 'Todo se ejecuta en esta máquina. No se sube nada y no se recopila telemetría.',
		appLicense: 'OpenEventViewer tiene licencia MIT.',
		thirdParty: 'Componentes de terceros',
		thirdPartyBody: (total: number, vendored: number, crates: number, npm: number) =>
			`Con esta app se distribuyen ${total} componentes: ${vendored} binarios incluidos, ${crates} crates de Rust, ${npm} paquetes de npm.`,
		shipped:
			'Los textos completos de las licencias van dentro del instalador como THIRD_PARTY_LICENSES.txt. MIT, BSD e ISC exigen que el aviso acompañe al binario, así que un enlace no bastaría.',
		filter: 'Filtrar componentes…',
		showTexts: 'Mostrar los textos de las licencias',
		hideTexts: 'Ocultar los textos de las licencias',
		noMatch: 'Ningún componente coincide.',
		redistributed: 'distribuido como binario',
		noOwnText: 'sin texto propio',
		withoutText: (count: number) =>
			`${count} componentes no publicaron un archivo de licencia propio; se aplica el texto canónico de la licencia indicada.`,
		material: 'Tus registros',
		materialBody:
			'Los registros de eventos se quedan donde Windows los guarda. Esta app los lee y nunca escribe en ellos.'
	},
	detail: {
		general: 'General',
		data: 'Datos del evento',
		xml: 'XML',
		search: 'Buscar en la web',
		copy: 'Copiar',
		copied: 'Copiado',
		close: 'Cerrar el panel de detalles',
		recordId: 'Registro',
		keywords: 'Palabras clave',
		noData: 'Este evento no lleva datos propios.'
	},
	updater: {
		title: 'Actualizaciones',
		body: (version: string) => `Versión ${version}. Se comprueba una vez al iniciar.`,
		check: 'Comprobar ahora',
		checking: 'Comprobando…',
		upToDate: 'actualizado',
		available: (version: string) => `${version} está disponible`,
		downloading: (percent: number | null) =>
			percent === null ? 'Descargando…' : `Descargando — ${percent}%`,
		ready: 'Instalada — reiniciando',
		install: 'Instalar y reiniciar',
		failed: 'La comprobación de actualizaciones falló.'
	},
	settings: {
		title: 'Configuración',
		appearance: 'Apariencia',
		appearanceBody: 'El tema de la ventana de la app.',
		system: 'Sistema',
		light: 'Claro',
		dark: 'Oscuro',
		colours: 'Colores',
		coloursBody: 'La paleta con la que se dibuja cada vista.',
		presets: {
			default: 'Predeterminado',
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
		language: 'Idioma',
		languageBody:
			'La interfaz de la app. El texto de los eventos conserva el idioma en el que Windows lo registró.',
		eventsRows: 'Eventos: filas por consulta',
		eventsRowsBody:
			'Cada evento cuesta una consulta de mensaje al publicador, así que un número mayor significa una espera más larga, no solo una lista más larga.',
		eventsRowsValue: (rows: number) => `${rows.toLocaleString('es')} filas`,
		showLogs: 'Mostrar el registro en la barra lateral',
		showLogsBody: 'Añade una entrada Registro a la navegación.',
		debugLogging: 'Grabar entradas de depuración',
		debugLoggingBody:
			'Detallado. Desactivado de forma predeterminada, porque las entradas de depuración desplazan justo las que buscabas.'
	}
};
