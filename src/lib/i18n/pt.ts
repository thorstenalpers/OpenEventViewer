import type { Translations } from './en';

export const pt: Translations = {
	sidebar: {
		tagline: 'Logs de eventos do Windows',
		events: 'Eventos',
		diagnose: 'Diagnóstico',
		log: 'Log',
		settings: 'Configurações',
		info: 'Informações',
		toLight: 'Mudar para o tema claro',
		toDark: 'Mudar para o tema escuro',
		sections: 'Seções',
		collapse: 'Recolher a barra lateral',
		expand: 'Expandir a barra lateral'
	},
	common: {
		loading: 'Carregando…',
		mockHost: 'Host simulado — sem backend Tauri. Os dados nesta página são dados de teste.'
	},
	events: {
		title: 'Eventos',
		subtitle: 'O que o Windows registrou, mais recentes primeiro.',
		channel: 'Canal',
		allChannels: 'Sistema e Aplicativo',
		from: 'De',
		to: 'Até',
		load: 'Carregar',
		keyword: 'Pesquisar em todas as colunas…',
		columnFilter: 'filtro de coluna',
		clearColumnFilters: 'Limpar filtros de coluna',
		loaded: (shown: number, total: number) =>
			shown === total ? `${total} eventos` : `${shown} de ${total} eventos`,
		elapsed: (ms: number) => `lido em ${ms} ms`,
		truncated:
			'mais do que o limite de linhas — restrinja o filtro ou aumente o limite nas Configurações',
		securityHint:
			'Feche o OpenEventViewer e inicie-o novamente como administrador, ou escolha um canal que não exija isso.',
		empty: 'Nada corresponde.',
		search: 'Pesquisar este evento na web',
		resize: (column: string) => `Redimensionar a coluna ${column}`,
		filters: {
			search: 'Pesquisar…',
			noMatch: 'Nada corresponde.',
			clear: 'Limpar este filtro',
			chosen: (count: number) => (count === 1 ? `${count} escolhido` : `${count} escolhidos`),
			after: (time: string) => `depois de ${time}`,
			before: (time: string) => `antes de ${time}`,
			timeHint: 'Hora local, o mesmo relógio que a tabela mostra.',
			numberHint: 'Nada ali era um número.',
			notUnderstood: (parts: string) => `Não entendido: ${parts}`,
			helpAny: 'qualquer um deles',
			helpCompare: 'acima, abaixo',
			helpRange: 'um intervalo, extremos incluídos',
			helpNot: 'tudo menos'
		},
		overTime: 'Ao longo do tempo',
		andMore: (kinds: number, count: number) =>
			`mais ${kinds} tipo${kinds === 1 ? '' : 's'}, ${count} no total`,
		bucketSize: (minutes: number) =>
			minutes >= 1440
				? `uma barra por ${minutes / 1440} dia${minutes === 1440 ? '' : 's'}`
				: minutes >= 60
					? `uma barra por ${minutes / 60} hora${minutes === 60 ? '' : 's'}`
					: `uma barra por ${minutes} minuto${minutes === 1 ? '' : 's'}`,
		bucketCount: (total: number, errors: number, warnings: number) => {
			const events = `${total} evento${total === 1 ? '' : 's'}`;
			const parts = [
				errors > 0 && `${errors} erro${errors === 1 ? '' : 's'}`,
				warnings > 0 && `${warnings} aviso${warnings === 1 ? '' : 's'}`
			].filter(Boolean);
			return parts.length === 0 ? events : `${events}, dos quais ${parts.join(' e ')}`;
		},
		columns: {
			level: 'Nível',
			time: 'Hora',
			provider: 'Provedor',
			eventId: 'ID',
			task: 'Tarefa',
			channel: 'Canal',
			computer: 'Computador',
			message: 'Mensagem'
		}
	},
	diagnose: {
		title: 'Diagnóstico',
		subtitle:
			'Examina o log em busca dos eventos que uma máquina escreve quando algo deu errado e depois traz o quarto de hora em torno de um deles.',
		days: (count: number) => (count === 1 ? 'Último dia' : `Últimos ${count} dias`),
		scan: 'Examinar',
		scanning: 'Examinando…',
		intro:
			'Nada foi examinado ainda. Escolha um período acima e pressione Examinar; cada achado — uma falha, um congelamento, um erro de disco, um processador limitado — aparece aqui como um incidente que você pode abrir.',
		pick: 'Abra um incidente para ver tudo o que a máquina escreveu no quarto de hora em torno dele.',
		nothing: 'Nada encontrado. Examine um período mais longo, ou tome isso como boa notícia.',
		window: (from: string, to: string) => `${from} — ${to}`,
		inWindow: (count: number) => `${count} evento${count === 1 ? '' : 's'} na janela`,
		kinds: {
			unexpectedShutdown: 'Desligamento inesperado',
			bugCheck: 'Tela azul',
			hardwareError: 'Erro de hardware',
			appHang: 'Aplicativo travado',
			appCrash: 'Falha de aplicativo',
			serviceFailure: 'Falha de serviço',
			diskError: 'Erro de disco',
			ntfs: 'Sistema de arquivos',
			displayTdr: 'Driver de vídeo reiniciado',
			processorPower: 'Processador limitado'
		}
	},
	log: {
		title: 'Log',
		subtitle: 'O que o app fez, mais recentes por último. Nada daqui é gravado em disco.',
		filter: 'Filtrar mensagens…',
		level: 'Nível',
		levels: {
			all: 'Todos os níveis',
			error: 'Erros',
			warning: 'Avisos',
			info: 'Informações',
			debug: 'Depuração'
		},
		clear: 'Limpar o log',
		empty: 'Nada registrado ainda.',
		count: (shown: number, total: number) => `${shown} de ${total} entradas`
	},
	info: {
		title: 'Informações',
		subtitle: 'O que este app é e sobre o que ele foi construído.',
		appBody:
			'Leia os logs de eventos do Windows e filtre-os até o que importa — sem conta, sem upload, sem telemetria.',
		offline: 'Tudo roda nesta máquina. Nada é enviado, e nenhuma telemetria é coletada.',
		appLicense: 'O OpenEventViewer é licenciado sob a licença MIT.',
		thirdParty: 'Componentes de terceiros',
		thirdPartyBody: (total: number, vendored: number, crates: number, npm: number) =>
			`${total} componentes acompanham este app: ${vendored} binários incluídos, ${crates} crates Rust, ${npm} pacotes npm.`,
		shipped:
			'Os textos completos das licenças acompanham o instalador como THIRD_PARTY_LICENSES.txt. MIT, BSD e ISC exigem que o aviso acompanhe o binário, então um link não bastaria.',
		filter: 'Filtrar componentes…',
		showTexts: 'Mostrar textos das licenças',
		hideTexts: 'Ocultar textos das licenças',
		noMatch: 'Nenhum componente corresponde.',
		redistributed: 'distribuído como binário',
		noOwnText: 'sem texto próprio',
		withoutText: (count: number) =>
			`${count} componentes não publicaram arquivo de licença próprio; vale o texto canônico da licença indicada.`,
		material: 'Seus logs',
		materialBody:
			'Os logs de eventos ficam onde o Windows os mantém. Este app os lê e nunca escreve neles.'
	},
	detail: {
		general: 'Geral',
		data: 'Dados do evento',
		xml: 'XML',
		search: 'Pesquisar na web',
		copy: 'Copiar',
		copied: 'Copiado',
		close: 'Fechar o painel de detalhes',
		recordId: 'Registro',
		keywords: 'Palavras-chave',
		noData: 'Este evento não traz dados próprios.'
	},
	updater: {
		title: 'Atualizações',
		body: (version: string) => `Versão ${version}. Verificada uma vez ao iniciar.`,
		check: 'Verificar agora',
		checking: 'Verificando…',
		upToDate: 'atualizado',
		available: (version: string) => `${version} está disponível`,
		downloading: (percent: number | null) =>
			percent === null ? 'Baixando…' : `Baixando — ${percent}%`,
		ready: 'Instalado — reiniciando',
		install: 'Instalar e reiniciar',
		failed: 'A verificação de atualização falhou.'
	},
	settings: {
		title: 'Configurações',
		appearance: 'Aparência',
		appearanceBody: 'Tema da janela do app.',
		system: 'Sistema',
		light: 'Claro',
		dark: 'Escuro',
		colours: 'Cores',
		coloursBody: 'A paleta da qual cada tela é desenhada.',
		presets: {
			default: 'Padrão',
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
			'A interface do app. O texto dos eventos mantém o idioma em que o Windows o registrou.',
		eventsRows: 'Eventos: linhas por consulta',
		eventsRowsBody:
			'Cada evento custa uma consulta de mensagem ao publicador, então um número maior significa uma espera mais longa, não só uma lista mais longa.',
		eventsRowsValue: (rows: number) => `${rows.toLocaleString('pt')} linhas`,
		showLogs: 'Mostrar o log na barra lateral',
		showLogsBody: 'Adiciona uma entrada Log à navegação.',
		debugLogging: 'Registrar entradas de depuração',
		debugLoggingBody:
			'Detalhado. Desativado por padrão, porque as entradas de depuração abafam justamente as que você procurava.'
	}
};
