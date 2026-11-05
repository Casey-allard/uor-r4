#!/usr/bin/env bash
set -euo pipefail

list_files() {
    printf '%s\n' \
        'index.html' \
        'index.css' \
        'geometric_prime_router_webapp.html' \
        'coi-serviceworker.js' \
        'r4_worker.js' \
        'models.json' \
        'assets/css/router_demo_offline.css' \
        'assets/css/uor_offline_fonts.css' \
        'assets/js/composer_input.js' \
        'assets/js/desktop_actions.js' \
        'assets/js/desktop_openai.js' \
        'assets/js/desktop_origin.js' \
        'assets/js/image_input.js' \
        'assets/js/mcp_client.js' \
        'assets/js/model_sources.js' \
        'assets/js/stream_metrics.js' \
        'assets/js/studio_api.js' \
        'assets/js/studio_files.js' \
        'assets/js/studio_ui.js' \
        'assets/js/uor_model_worker.js' \
        'assets/js/uor_offline_vendor.js' \
        'assets/js/uor_tokenizer_artifact.js' \
        'pkg/uor_r4_integer.js' \
        'pkg/uor_r4_integer_bg.wasm' \
        'pkg/synthetic-stack.lut' \
        'pkg/tokenizer.json'
}

case "${1:-}" in
    --list)
        list_files
        ;;
    copy)
        [ "$#" -ge 3 ] || { printf 'usage: runtime-files.sh copy <source> <dest> [--allow-generated-missing]\n' >&2; exit 2; }
        src=$2
        dest=$3
        allow_missing=${4:-}
        [ -d "$src" ] || { printf 'Studio source is unavailable: %s\n' "$src" >&2; exit 1; }
        case "$dest" in ''|'/'|'.') printf 'Refusing unsafe destination: %s\n' "$dest" >&2; exit 1 ;; esac
        rm -rf "$dest"
        while IFS= read -r rel; do
            if [ -f "$src/$rel" ]; then
                mkdir -p "$(dirname "$dest/$rel")"
                cp "$src/$rel" "$dest/$rel"
            elif [ "$allow_missing" != '--allow-generated-missing' ]; then
                printf 'Required runtime file is missing: %s\n' "$src/$rel" >&2
                exit 1
            fi
        done < <(list_files)
        ;;
    --verify)
        [ "$#" -eq 2 ] || { printf 'usage: runtime-files.sh --verify <dest>\n' >&2; exit 2; }
        dest=$2
        symlink=$(find "$dest" -type l -print -quit)
        if [ -n "$symlink" ]; then
            printf 'Runtime symlinks are refused: %s\n' "$symlink" >&2
            exit 1
        fi
        while IFS= read -r rel; do
            [ -f "$dest/$rel" ] || { printf 'Runtime file is missing: %s\n' "$dest/$rel" >&2; exit 1; }
        done < <(list_files)
        while IFS= read -r -d '' path; do
            rel=${path#"$dest"/}
            [ "$rel" = 'SHA256SUMS' ] && continue
            case "$rel" in
                models/*.uorlut|models/*.tokenizer.json|models/*.uorlut.audit.json)
                    continue
                    ;;
            esac
            if ! list_files | grep -Fxq "$rel"; then
                printf 'Unexpected runtime file: %s\n' "$path" >&2
                exit 1
            fi
        done < <(find "$dest" -type f -print0)
        node - "$dest" <<'NODE'
const { createHash } = require('node:crypto');
const { existsSync, lstatSync, readFileSync, readdirSync } = require('node:fs');
const { resolve, sep } = require('node:path');

const dest = resolve(process.argv[2]);
const modelsRoot = resolve(dest, 'models');
const pkgRoot = resolve(dest, 'pkg');
const manifest = JSON.parse(readFileSync(resolve(dest, 'models.json'), 'utf8'));
const declared = new Map();

function declare(url, sha) {
  if (typeof url !== 'string') return;
  const rel = url.replace(/^\.\//, '');
  const root = rel.startsWith('models/') ? modelsRoot : (rel.startsWith('pkg/') ? pkgRoot : null);
  if (!root) return;
  const file = resolve(dest, rel);
  if (file === root || !file.startsWith(root + sep)) {
    throw new Error('Model catalog path escapes its artifact root: ' + url);
  }
  if (!/^[0-9a-f]{64}$/.test(String(sha || ''))) {
    throw new Error('Model catalog hash is missing or invalid: ' + url);
  }
  if (declared.has(file) && declared.get(file) !== sha) {
    throw new Error('Conflicting model catalog hashes: ' + url);
  }
  declared.set(file, sha);
}

for (const model of manifest.models || []) {
  declare(model.url, model.sha256);
  declare(model.tokenizer_url, model.tokenizer_sha256);
  declare(model.audit_url, model.served_forward_audit_sha256);
}

function actualFiles(directory) {
  const files = [];
  if (!existsSync(directory)) return files;
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = resolve(directory, entry.name);
    if (entry.isSymbolicLink()) throw new Error('Model artifact symlinks are refused: ' + path);
    if (entry.isDirectory()) files.push(...actualFiles(path));
    else if (entry.isFile()) files.push(path);
  }
  return files;
}

if (existsSync(modelsRoot) && lstatSync(modelsRoot).isSymbolicLink()) {
  throw new Error('Model artifact root symlinks are refused: ' + modelsRoot);
}
for (const file of actualFiles(modelsRoot)) {
  if (!declared.has(file)) throw new Error('Unexpected model artifact: ' + file);
}
for (const [file, expected] of declared) {
  if (!existsSync(file) || !lstatSync(file).isFile()) {
    throw new Error('Declared model artifact is missing: ' + file);
  }
  const actual = createHash('sha256').update(readFileSync(file)).digest('hex');
  if (actual !== expected) {
    throw new Error('Model artifact hash mismatch: ' + file);
  }
}
console.log('Catalog-bound model artifacts verified: ' + declared.size);
NODE
        printf 'Runtime allowlist verified: %s\n' "$dest"
        ;;
    *)
        printf 'usage: runtime-files.sh --list | copy <source> <dest> [--allow-generated-missing] | --verify <dest>\n' >&2
        exit 2
        ;;
esac
