#!/usr/bin/env node
// Recompute Foundry's shell closure manifest.json, sw.js (from sw.template.js),
// and update provenance.json's browser_projection_sha256 to match the sealed closure.
//
// Usage:
//   node scripts/studio/seal-shell-closure.mjs <shell-dir> [--template <sw.template.js>]

import { createHash } from 'node:crypto';
import { existsSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { join, relative, resolve } from 'node:path';

const EXCLUDED = new Set(['manifest.json', 'provenance.json', 'warmup.json', 'sw.js', 'sw.template.js']);

function sha256Hex(buf) {
  return createHash('sha256').update(buf).digest('hex');
}

function walkFiles(dir, root, out) {
  const entries = readdirSync(dir, { withFileTypes: true }).sort((a, b) => (a.name < b.name ? -1 : a.name > b.name ? 1 : 0));
  for (const entry of entries) {
    const full = join(dir, entry.name);
    const rel = relative(root, full).replace(/\\/g, '/');
    if (entry.isSymbolicLink()) {
      throw new Error(`Refusing symlink in shell closure: ${rel}`);
    }
    if (entry.isDirectory()) {
      if (rel === 'app') continue;
      walkFiles(full, root, out);
    } else if (entry.isFile()) {
      out.push(rel);
    }
  }
}

function parseArgs(argv) {
  let shellDir = '';
  let templatePath = '';
  for (let i = 0; i < argv.length; i += 1) {
    const a = argv[i];
    if (a === '--template') {
      i += 1;
      templatePath = resolve(argv[i] || '');
    } else if (!shellDir) {
      shellDir = resolve(a);
    } else {
      throw new Error(`Unknown argument: ${a}`);
    }
  }
  if (!shellDir || !existsSync(shellDir) || !statSync(shellDir).isDirectory()) {
    throw new Error('Usage: node scripts/studio/seal-shell-closure.mjs <shell-dir> [--template <sw.template.js>]');
  }
  if (!templatePath) {
    templatePath = join(shellDir, 'sw.template.js');
  }
  return { shellDir, templatePath };
}

function main() {
  const { shellDir, templatePath } = parseArgs(process.argv.slice(2));
  const files = [];
  walkFiles(shellDir, shellDir, files);
  files.sort((a, b) => (a < b ? -1 : a > b ? 1 : 0));

  const rows = [];
  for (const rel of files) {
    if (rel === 'warmup.json' || rel.endsWith('/warmup.json')) {
      throw new Error(`Refusing credential file in shell closure: ${rel}`);
    }
    if (EXCLUDED.has(rel)) continue;
    const bytes = readFileSync(join(shellDir, rel));
    rows.push({
      path: rel,
      sha256: sha256Hex(bytes),
      bytes: bytes.byteLength,
    });
  }

  // Canonical field order inside serialized objects matches project_shell.rs & shell-cache.js.
  const canonicalRows = rows.map((r) => ({ bytes: r.bytes, path: r.path, sha256: r.sha256 }));
  const closure = sha256Hex(Buffer.from(JSON.stringify(canonicalRows), 'utf8'));
  const manifest = {
    spec: 'hologram-forge/shell/1',
    closure,
    files: rows,
  };
  writeFileSync(join(shellDir, 'manifest.json'), JSON.stringify(manifest) + '\n', 'utf8');

  const tmpl = readFileSync(templatePath, 'utf8');
  if (!tmpl.includes('__CLOSURE__')) {
    throw new Error(`Service worker template missing __CLOSURE__ placeholder: ${templatePath}`);
  }
  writeFileSync(join(shellDir, 'sw.js'), tmpl.replaceAll('__CLOSURE__', closure), 'utf8');

  const provPath = join(shellDir, 'provenance.json');
  let prov = {};
  if (existsSync(provPath)) {
    try {
      prov = JSON.parse(readFileSync(provPath, 'utf8'));
    } catch {
      prov = {};
    }
  }
  prov.schema = prov.schema || 'hologram-forge/provenance/1';
  prov.browser_projection_sha256 = closure;
  writeFileSync(provPath, JSON.stringify(prov, null, 2) + '\n', 'utf8');

  console.log(JSON.stringify({
    schema: 'hologram-forge/shell/1',
    files: rows.length,
    closure,
  }));
}

main();
