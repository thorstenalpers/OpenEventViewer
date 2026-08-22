import { execFileSync } from 'node:child_process';
import { existsSync, readdirSync, readFileSync, mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { homedir } from 'node:os';

/**
 * Writes the third-party notices this app has to ship.
 *
 * The customer gets a binary, not source. MIT, BSD and ISC all require the copyright notice *and*
 * the licence text to travel with a binary distribution, and Apache-2.0 additionally requires the
 * licence itself — so a list of SPDX identifiers is not compliance, it is a summary. This produces
 * both: the full text file that ships beside the executable, and a metadata index the Info page
 * renders as a searchable table.
 *
 *   node scripts/create-licenses.mjs
 */

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const rule = '='.repeat(96);

// `shell` because npm and cargo are .cmd shims on Windows, which Node refuses to spawn otherwise.
// Every argument here is a literal, so the usual escaping hazard does not apply.
function run(command, args, cwd) {
	return execFileSync(command, args, {
		cwd,
		encoding: 'utf8',
		maxBuffer: 128 * 1024 * 1024,
		shell: true
	});
}

const LICENSE_FILE = /^(LICEN[CS]E|COPYING|NOTICE|UNLICENSE)([-._].*)?$/i;

/** The licence text as published, or `null` when the package shipped none. */
function licenseText(directory) {
	if (!directory || !existsSync(directory)) return null;
	let entries;
	try {
		entries = readdirSync(directory, { withFileTypes: true });
	} catch {
		return null;
	}

	const texts = entries
		.filter((entry) => entry.isFile() && LICENSE_FILE.test(entry.name))
		.sort((a, b) => a.name.localeCompare(b.name))
		.map((entry) => {
			try {
				return readFileSync(join(directory, entry.name), 'utf8').trim();
			} catch {
				return '';
			}
		})
		.filter(Boolean);

	return texts.length ? texts.join('\n\n') : null;
}

/** Where cargo unpacked a crate's source, which is where its licence file lives. */
function crateDirectory(name, version) {
	const home = process.env.CARGO_HOME ?? join(homedir(), '.cargo');
	const registry = join(home, 'registry', 'src');
	if (!existsSync(registry)) return null;
	for (const host of readdirSync(registry)) {
		const candidate = join(registry, host, `${name}-${version}`);
		if (existsSync(candidate)) return candidate;
	}
	return null;
}

function crates() {
	// Only what actually ships: the Windows target, no build- or dev-dependencies.
	const json = run(
		'cargo',
		[
			'license',
			'--json',
			'--avoid-dev-deps',
			'--avoid-build-deps',
			'--filter-platform',
			'x86_64-pc-windows-msvc'
		],
		join(root, 'src-tauri')
	);

	return JSON.parse(json)
		.filter((crate) => !crate.name.startsWith('openexamtrainer'))
		.map((crate) => ({
			kind: 'crate',
			name: crate.name,
			version: crate.version,
			url: crate.repository ?? '',
			license: crate.license ?? 'UNKNOWN',
			text: licenseText(crateDirectory(crate.name, crate.version))
		}));
}

/**
 * Every package in the tree, not only `dependencies`.
 *
 * The frontend is bundled at build time by adapter-static, so a package under devDependencies can
 * still end up inside the shipped assets. The split carries no information about what ships.
 */
function packages() {
	const tree = JSON.parse(run('npm', ['ls', '--all', '--long', '--json'], root));
	const found = new Map();

	(function walk(node) {
		for (const [name, dep] of Object.entries(node.dependencies ?? {})) {
			if (!dep.version) continue;
			const key = `${name}@${dep.version}`;
			if (!found.has(key)) {
				found.set(key, {
					kind: 'npm',
					name,
					version: dep.version,
					license: dep.license ?? 'UNKNOWN',
					url: repository(dep.path),
					text: licenseText(dep.path)
				});
			}
			walk(dep);
		}
	})(tree);

	return [...found.values()];
}

function repository(path) {
	if (!path) return '';
	try {
		const pkg = JSON.parse(readFileSync(join(path, 'package.json'), 'utf8'));
		const url = typeof pkg.repository === 'string' ? pkg.repository : pkg.repository?.url;
		return (url ?? pkg.homepage ?? '').replace(/^git\+/, '').replace(/\.git$/, '');
	} catch {
		return '';
	}
}

/** Every licence a text announces, not the first one matched. */
function spdxOf(text) {
	if (!text) return 'UNKNOWN';

	const found = [];
	const note = (id) => found.includes(id) || found.push(id);

	if (/Redistribution and use in source and binary forms/i.test(text)) {
		note(
			/neither the name|nor the names of (its|their) contributors/i.test(text)
				? 'BSD-3-Clause'
				: 'BSD-2-Clause'
		);
	}
	if (/Apache License\s*\n?\s*Version 2\.0/i.test(text)) note('Apache-2.0');
	if (/UNICODE LICENSE/i.test(text)) note('Unicode-3.0');
	if (/Permission is hereby granted, free of charge/i.test(text)) note('MIT');
	if (/FreeType Project LICENSE/i.test(text)) note('FTL');
	if (/Independent JPEG Group/i.test(text)) note('IJG');
	if (/Mozilla Public License/i.test(text)) note('MPL-2.0');
	if (/altered source versions must be plainly marked/i.test(text)) note('Zlib');

	return found.length ? found.join(' AND ') : 'see text';
}

const entries = [...crates(), ...packages()]
	.map((entry) => ({ ...entry, license: entry.license || spdxOf(entry.text) }))
	.sort((a, b) => a.kind.localeCompare(b.kind) || a.name.localeCompare(b.name));

const missing = entries.filter((entry) => !entry.text);

const header = [
	'OpenExamTrainer — third-party notices',
	'',
	`${entries.length} components ship with this application.`,
	'',
	'This file is generated by scripts/create-licenses.mjs. Rust crates are resolved for the',
	'x86_64-pc-windows-msvc target with build- and dev-dependencies excluded; npm packages cover',
	'the whole tree, because the frontend is bundled at build time and the dependency split says',
	'nothing about what ends up in the assets.',
	'',
	missing.length
		? `${missing.length} components published no licence file of their own; their SPDX identifier is recorded and the canonical text of that licence applies.`
		: 'Every component below carries its own licence text.',
	''
].join('\n');

const body = entries
	.map((entry) =>
		[
			rule,
			`${entry.name} ${entry.version}`,
			`License: ${entry.license}`,
			entry.url ? `Source: ${entry.url}` : null,
			'',
			entry.text ?? '(no licence file published with this component)',
			''
		]
			.filter((line) => line !== null)
			.join('\n')
	)
	.join('\n');

const resources = join(root, 'src-tauri', 'resources');
mkdirSync(resources, { recursive: true });
writeFileSync(join(resources, 'THIRD_PARTY_LICENSES.txt'), `${header}\n${body}`, 'utf8');

// The index the Info page renders. Texts are deliberately left out: the table is for finding a
// component, and 583 licence texts inside the prerendered bundle would cost a megabyte to no end.
writeFileSync(
	join(root, 'src', 'lib', 'third-party.json'),
	`${JSON.stringify(
		entries.map(({ kind, name, version, license, url, text }) => ({
			kind,
			name,
			version,
			license,
			url,
			hasText: Boolean(text)
		})),
		null,
		'\t'
	)}\n`,
	'utf8'
);

console.log(
	`${entries.length} components — ${entries.length - missing.length} with their own licence text, ${missing.length} without.`
);
