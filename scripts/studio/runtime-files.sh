#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
typed=0
source_root=''

resolve_source_root() {
    local candidate="${1:-}"
    if [ -z "$candidate" ]; then
        candidate="$repo_root/web/studio/dist"
    fi
    if [ ! -f "$candidate/host.bundle.js" ] && [ -f "$candidate/dist/host.bundle.js" ]; then
        candidate="$candidate/dist"
    fi
    printf '%s' "$candidate"
}

list_files() {
    local root="${source_root:-$repo_root/web/studio/dist}"
    if [ ! -d "$root" ]; then
        return 1
    fi
    (
        cd "$root"
        find . -type f ! -path '*/node_modules/*' ! -name 'sw.template.js' ! -name '*.test.*' ! -name '*.map' ! -name 'warmup.json' ! -name 'README.md' ! -name '*.d.ts' ! -path './pkg/package.json' ! -path './pkg-mvm/package.json' | sed 's|^\./||' | sort
    )
}

verify_required_files() {
    local root="$1"
    for req in \
        'index.html' \
        '404.html' \
        'holo.html' \
        'manifest.json' \
        'sw.js' \
        'models.json' \
        'host.bundle.js' \
        'core.wasm' \
        'assets/index.js' \
        'assets/index.css' \
        'kernel/kernel.holo' \
        'pkg/uor_r4_integer.js' \
        'pkg/uor_r4_integer_bg.wasm' \
        'pkg/synthetic-stack.lut' \
        'pkg/tokenizer.json' \
        'pkg-mvm/uor_r4_mvm.js' \
        'pkg-mvm/uor_r4_mvm_bg.wasm' \
        'uor/js/uor_model_worker.js' \
        'uor/js/uor_tokenizer_artifact.js'; do
        if [ ! -f "$root/$req" ]; then
            printf 'Required Foundry runtime file is missing: %s\n' "$root/$req" >&2
            exit 1
        fi
    done
}

case "${1:-}" in
    --list)
        if [ "${2:-}" = '--typed' ]; then
            typed=1
            source_root=$(resolve_source_root "${3:-}")
        else
            source_root=$(resolve_source_root "${2:-}")
        fi
        list_files
        ;;
    copy)
        [ "$#" -ge 3 ] || { printf 'usage: runtime-files.sh copy <source> <dest> [--allow-generated-missing|--typed]\n' >&2; exit 2; }
        src=$(resolve_source_root "$2")
        dest=$3
        allow_missing=${4:-}
        if [ "$allow_missing" = '--typed' ]; then
            typed=1
        fi
        source_root=$src
        [ -d "$src" ] || { printf 'Studio source is unavailable: %s\n' "$src" >&2; exit 1; }
        case "$dest" in ''|'/'|'.') printf 'Refusing unsafe destination: %s\n' "$dest" >&2; exit 1 ;; esac
        if [ "$allow_missing" != '--allow-generated-missing' ]; then
            verify_required_files "$src"
        fi
        rm -rf "$dest"
        mkdir -p "$dest"
        while IFS= read -r rel; do
            mkdir -p "$(dirname "$dest/$rel")"
            cp "$src/$rel" "$dest/$rel"
        done < <(list_files)
        ;;
    --verify)
        [ "$#" -ge 2 ] || { printf 'usage: runtime-files.sh --verify <dest> [--typed]\n' >&2; exit 2; }
        dest=$2
        if [ "${3:-}" = '--typed' ]; then
            typed=1
        fi
        source_root=$dest
        symlink=$(find "$dest" -type l -print -quit)
        if [ -n "$symlink" ]; then
            printf 'Runtime symlinks are refused: %s\n' "$symlink" >&2
            exit 1
        fi
        for forbidden_name in warmup.json sw.template.js; do
            if find "$dest" -name "$forbidden_name" -print -quit | grep -q .; then
                printf 'Forbidden staged file in %s: %s\n' "$dest" "$forbidden_name" >&2
                exit 1
            fi
        done
        if find "$dest" -name '*.map' -o -name '*.test.*' -print -quit | grep -q .; then
            printf 'Dev/test artifact leaked into %s\n' "$dest" >&2
            exit 1
        fi
        verify_required_files "$dest"
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

// Verify Foundry shell closure manifest matches staged files.
const shellManifest = JSON.parse(readFileSync(resolve(dest, 'manifest.json'), 'utf8'));
if (shellManifest.spec !== 'hologram-forge/shell/1' || !Array.isArray(shellManifest.files)) {
  throw new Error('Invalid Foundry shell manifest.json');
}
for (const row of shellManifest.files) {
  const p = resolve(dest, row.path);
  if (!existsSync(p) || !lstatSync(p).isFile()) {
    throw new Error('Shell manifest file missing from staged dir: ' + row.path);
  }
  const actual = createHash('sha256').update(readFileSync(p)).digest('hex');
  if (actual !== row.sha256) {
    throw new Error('Shell manifest digest mismatch for ' + row.path);
  }
}
console.log('Foundry runtime + ' + declared.size + ' model artifacts + ' + shellManifest.files.length + ' closure files verified');
NODE
        printf 'Runtime allowlist verified: %s\n' "$dest"
        ;;
    *)
        printf 'usage: runtime-files.sh --list [--typed] [dir] | copy <source> <dest> [--allow-generated-missing|--typed] | --verify <dest> [--typed]\n' >&2
        exit 2
        ;;
esac
