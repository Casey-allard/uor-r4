#!/usr/bin/env bash
# Stage the Studio web client (web/studio/) into a published-site directory.
#
# Usage:
#   scripts/stage-studio-site.sh <studio-src> <dest-dir> [public_model_url] [public_model_label] \
#       [--trained-manifest FILE]
#
# <studio-src> is the built web/studio tree; it must already contain the wasm-pack output and the
# generated demo artifacts (none of these are committed; see web/studio/README.md):
#   pkg/uor_r4_integer.js, pkg/uor_r4_integer_bg.wasm, pkg/synthetic-stack.lut, pkg/tokenizer.json
#
# The helper:
#   1. replaces <dest-dir> wholesale (no stale files from an older Studio can survive a publish;
#      the live gh-pages /studio was the retired 13 Oct app before this change);
#   2. copies only the runtime files, not the dev-only verification harness, tools or README;
#   3. writes <dest-dir>/models.json, schema "uor-r4.studio-models/1": the bundled synthetic demo
#      plus, when given, one optional public-model URL slot (http/https only), plus any trained
#      models named by --trained-manifest;
#   4. refuses a file over GitHub Pages' 100 MB per-file limit, and prints file count and size.
#
# --trained-manifest FILE names a JSON array of one to four trained model entries:
#   [{ "id": "fbs-dual-c131-seed1", "label": "...", "file": "/path/seed-1.uorlut",
#      "hf_url": "https://huggingface.co/<org>/<repo>/resolve/main/<file>.uorlut",
#      "tokenizer": "/path/seed-1.tokenizer.json",
#      "audit": "/path/final.audit.json", "audit_sha256": "<64 lowercase hex>",
#      "note": "..." }]
# Each entry is copied to <dest>/models/<id>.uorlut with <id>.tokenizer.json beside it, and TWO
# published cards are written: the same-origin copy (url "./models/<id>.uorlut") and the Hugging
# Face URL (cross-origin, user-initiated, CORS required). The tokenizer is validated against the
# artifact (stack_sha256 + vocab) and the served-forward audit receipt is validated against the
# artifact before anything is copied. Remote-only entries must pin both artifact and receipt.
# Paths are local files; nothing is fetched from the network.
#
# It reads nothing from the network and writes only under <dest-dir>.
set -euo pipefail
repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

usage() {
    echo "usage: $0 <studio-src> <dest-dir> [public_model_url] [public_model_label] [--trained-manifest FILE]" >&2
    exit 2
}

[ "$#" -ge 2 ] || usage
src=$1
dest=$2
shift 2
public_url=''
public_label='Public model'
trained_manifest=''
positional=0
while [ "$#" -gt 0 ]; do
    case "$1" in
        --trained-manifest)
            [ "$#" -ge 2 ] || usage
            trained_manifest=$2
            shift 2
            ;;
        --public-model-url)
            [ "$#" -ge 2 ] || usage
            public_url=$2
            shift 2
            ;;
        --public-model-label)
            [ "$#" -ge 2 ] || usage
            public_label=$2
            shift 2
            ;;
        --*)
            echo "stage-studio-site: unknown option $1" >&2
            usage
            ;;
        *)
            positional=$((positional + 1))
            if [ "$positional" -eq 1 ]; then
                public_url=$1
            elif [ "$positional" -eq 2 ]; then
                public_label=$1
            else
                usage
            fi
            shift
            ;;
    esac
done
if [ -n "$trained_manifest" ] && [ ! -f "$trained_manifest" ]; then
    echo "stage-studio-site: --trained-manifest file not found: $trained_manifest" >&2
    exit 1
fi

[ -f "$src/index.html" ] || { echo "stage-studio-site: $src/index.html is missing" >&2; exit 1; }
for required in \
    pkg/uor_r4_integer.js \
    pkg/uor_r4_integer_bg.wasm \
    pkg/synthetic-stack.lut \
    pkg/tokenizer.json; do
    if [ ! -f "$src/$required" ]; then
        echo "stage-studio-site: $src/$required is missing; build the wasm bundle and the demo artifacts first (web/studio/README.md)" >&2
        exit 1
    fi
done
case "$public_url" in
    '') ;;
    http://*|https://*) ;;
    *) echo "stage-studio-site: public_model_url must be an http(s) URL, got '$public_url'" >&2; exit 1 ;;
esac
if [ -n "$public_url" ] && [ -z "$public_label" ]; then
    echo "stage-studio-site: public_model_label must not be empty when public_model_url is set" >&2
    exit 1
fi

rm -rf "$dest"
mkdir -p "$dest"
cp -R "$src"/. "$dest"/
# Dev-only files stay out of the published page.
rm -rf "$dest/tools" "$dest/README.md"
find "$dest" -maxdepth 1 -type f \( -name 'verify_*.mjs' -o -name '*.sh' \) -delete
rm -f "$dest/pkg/.gitignore"
rm -f "$dest/models.json"

node - "$dest" "$public_url" "$public_label" "$trained_manifest" \
    "$repo_root/scripts/validate_served_forward_receipt.py" <<'NODE'
const { createHash } = require('node:crypto');
const { execFileSync } = require('node:child_process');
const { readFileSync, writeFileSync, copyFileSync, mkdirSync, unlinkSync } = require('node:fs');
const { join } = require('node:path');

const [dest, publicUrl, publicLabel, trainedManifestPath, receiptGate] = process.argv.slice(2);
const MAX_FILE_BYTES = 100 * 1024 * 1024;
const sha256 = (buffer) => createHash('sha256').update(buffer).digest('hex');
const sha256File = (path) => sha256(readFileSync(path));
const stack = join(dest, 'pkg/synthetic-stack.lut');
const tokenizer = join(dest, 'pkg/tokenizer.json');

function readUorLutHeader(path) {
  const bytes = readFileSync(path);
  if (bytes.byteLength < 16 || bytes.subarray(0, 8).toString('ascii') !== 'UORLUT01') {
    throw new Error(path + ': not a UORLUT01 stack artifact');
  }
  const jsonLen = Number(bytes.readBigUInt64LE(8));
  if (!Number.isSafeInteger(jsonLen) || jsonLen <= 0 || 16 + jsonLen > bytes.byteLength) {
    throw new Error(path + ': invalid UORLUT01 header length');
  }
  return JSON.parse(bytes.subarray(16, 16 + jsonLen).toString('utf8'));
}

function requireServedForwardAudit(entry, id, artifactSha, artifactBytes, artifactPath) {
  const receiptPath = typeof entry.audit === 'string' ? entry.audit.trim() : '';
  const receiptSha = typeof entry.audit_sha256 === 'string' ? entry.audit_sha256.trim().toLowerCase() : '';
  if (!receiptPath) throw new Error(id + ': audit receipt is required');
  if (!/^[0-9a-f]{64}$/.test(receiptSha)) throw new Error(id + ': audit_sha256 must be 64 lowercase hex characters');
  const args = [
    receiptGate,
    '--receipt', receiptPath,
    '--expected-receipt-sha256', receiptSha,
    '--expected-artifact-sha256', artifactSha,
  ];
  if (artifactPath) {
    args.push('--artifact', artifactPath);
  } else {
    args.push('--artifact-sha256', artifactSha, '--artifact-bytes', String(artifactBytes));
  }
  try {
    execFileSync(process.env.PYTHON || 'python3', args, { stdio: ['ignore', 'pipe', 'pipe'] });
  } catch (error) {
    const detail = error.stderr ? String(error.stderr).trim() : error.message;
    throw new Error(id + ': served-forward audit receipt rejected: ' + detail);
  }
  return receiptSha;
}

const manifest = {
  schema: 'uor-r4.studio-models/1',
  generated_by: '.github/workflows/docs-site.yml via scripts/stage-studio-site.sh',
  note: 'Model sources the Studio may offer. The bundled demo ships with the page; anything else is user-initiated (local file, URL fetch, or an OpenAI-compatible API the user configures). No private artifact is published here and no prompt leaves the device on the local path.',
  sources: {
    bundled: true,
    local_file: true,
    url: publicUrl
      ? { available: true, default_url: publicUrl, label: publicLabel }
      : { available: false, note: 'set public_model_url at workflow dispatch to publish a public model URL slot' },
    openai_compatible_api: { configured_in_browser: true, storage_key: 'uor.studio.modelSources.v1' },
  },
  models: [
    {
      id: 'synthetic-seed-5',
      kind: 'bundled',
      label: 'Synthetic demo stack (seed 5, untrained)',
      url: 'pkg/synthetic-stack.lut',
      tokenizer_url: 'pkg/tokenizer.json',
      format: 'UORLUT01',
      sha256: sha256File(stack),
      size_bytes: readFileSync(stack).length,
      tokenizer_sha256: sha256File(tokenizer),
      note: 'Synthetic stand-in generated by mk-synthetic-stack seed 5; it exercises the integer serving engine and is not a trained model.',
    },
  ],
};

if (trainedManifestPath) {
  let entries;
  try {
    entries = JSON.parse(readFileSync(trainedManifestPath, 'utf8'));
  } catch (error) {
    throw new Error('--trained-manifest is not valid JSON: ' + error.message);
  }
  if (!Array.isArray(entries) || entries.length < 1 || entries.length > 4) {
    throw new Error('--trained-manifest must hold one to four model entries (the two M1 trunks and optionally two research trunks)');
  }
  mkdirSync(join(dest, 'models'), { recursive: true });
  for (const entry of entries) {
    const id = String((entry && entry.id) || '').trim();
    if (!/^[a-z0-9][a-z0-9._-]{2,63}$/.test(id)) throw new Error('trained entry id is invalid: ' + JSON.stringify(entry && entry.id));
    const label = typeof entry.label === 'string' ? entry.label.trim() : '';
    if (!label) throw new Error(id + ': label is required');
    const localFile = typeof entry.file === 'string' ? entry.file.trim() : '';
    const localTokenizer = typeof entry.tokenizer === 'string' ? entry.tokenizer.trim() : '';
    const hfUrl = typeof entry.hf_url === 'string' ? entry.hf_url.trim() : '';
    if (hfUrl && !/^https?:\/\//i.test(hfUrl)) throw new Error(id + ': hf_url must be an http(s) URL');
    if (!localFile && !hfUrl) throw new Error(id + ': file or hf_url is required');
    if (localFile && !localTokenizer) throw new Error(id + ': tokenizer is required for a bundled model');
    const subdir = typeof entry.subdir === 'string' ? entry.subdir.trim().replace(/^\/+|\/+$/g, '') : '';
    if (subdir && !/^[a-z0-9][a-z0-9._/-]*$/.test(subdir)) throw new Error(id + ': subdir is invalid: ' + JSON.stringify(entry.subdir));
    const primary = entry.primary === true;
    const hfTokenizerUrl = typeof entry.hf_tokenizer_url === 'string' ? entry.hf_tokenizer_url : '';
    if (hfTokenizerUrl && !/^https?:\/\//i.test(hfTokenizerUrl)) throw new Error(id + ': hf_tokenizer_url must be an http(s) URL');
    if (!localFile) {
      // Published (remote-only) entry: no artifact is bundled or copied; the card names the host.
      const sha = /^[0-9a-f]{64}$/i.test(String(entry.sha256 || '')) ? String(entry.sha256).toLowerCase() : '';
      if (!sha) throw new Error(id + ': sha256 is required for an HF-only entry');
      const size = Number(entry.size_bytes);
      if (!Number.isInteger(size) || size <= 0) throw new Error(id + ': size_bytes is required for an HF-only entry');
      if (size > MAX_FILE_BYTES) throw new Error(id + ': artifact is over the 100 MB per-file limit');
      if (!hfTokenizerUrl) throw new Error(id + ': hf_tokenizer_url is required for an HF-only entry');
      const auditSha = requireServedForwardAudit(entry, id, sha, size, null);
      const schema = typeof entry.schema === 'string' ? entry.schema : '';
      const note = typeof entry.note === 'string' && entry.note.trim() ? entry.note.trim() + ' ' : '';
      const remote = {
        id: id,
        kind: 'published',
        primary: primary,
        label: primary ? label : label + ' - Hugging Face',
        url: hfUrl,
        tokenizer_url: hfTokenizerUrl,
        format: 'UORLUT01',
        sha256: sha,
        size_bytes: size,
        served_forward_audit_sha256: auditSha,
        note: note + 'User-initiated cross-origin download from the host named on the card; the host must serve CORS headers.',
      };
      if (schema) remote.schema = schema;
      manifest.models.push(remote);
      continue;
    }
    const artifactBytes = readFileSync(entry.file);
    const tokenizerBytes = readFileSync(entry.tokenizer);
    if (artifactBytes.byteLength > MAX_FILE_BYTES) throw new Error(id + ': artifact is over the 100 MB per-file limit');
    if (tokenizerBytes.byteLength > MAX_FILE_BYTES) throw new Error(id + ': tokenizer is over the 100 MB per-file limit');
    const artifactSha = sha256(artifactBytes);
    const tokenizerSha = sha256(tokenizerBytes);
    const header = readUorLutHeader(entry.file);
    const schema = header && typeof header.schema === 'string' ? header.schema : '';
    const shape = header && header.shape && typeof header.shape === 'object' ? header.shape : {};
    const vocab = Number(shape.vocab || shape.vocabulary || 0);
    let tokenizerArtifact;
    try {
      tokenizerArtifact = JSON.parse(tokenizerBytes.toString('utf8'));
    } catch (error) {
      throw new Error(id + ': tokenizer is not valid JSON: ' + error.message);
    }
    if (!tokenizerArtifact || tokenizerArtifact.stack_sha256 !== artifactSha || tokenizerArtifact.vocab_size !== vocab) {
      throw new Error(id + ': tokenizer does not bind to the artifact (stack_sha256/vocab mismatch)');
    }
    const auditSha = requireServedForwardAudit(
      entry, id, artifactSha, artifactBytes.byteLength, entry.file,
    );
    const artifactName = id + '.uorlut';
    const tokenizerName = id + '.tokenizer.json';
    const auditName = id + '.uorlut.audit.json';
    const targetDir = subdir ? join(dest, 'models', subdir) : join(dest, 'models');
    mkdirSync(targetDir, { recursive: true });
    const artifactOut = join(targetDir, artifactName);
    const tokenizerOut = join(targetDir, tokenizerName);
    const auditOut = join(targetDir, auditName);
    copyFileSync(entry.file, artifactOut);
    copyFileSync(entry.tokenizer, tokenizerOut);
    copyFileSync(entry.audit, auditOut);
    if (sha256File(artifactOut) !== artifactSha || sha256File(tokenizerOut) !== tokenizerSha || sha256File(auditOut) !== auditSha) {
      for (const path of [artifactOut, tokenizerOut, auditOut]) {
        try { unlinkSync(path); } catch (_) {}
      }
      throw new Error(id + ': a source changed while release files were being copied');
    }
    const urlPrefix = './models/' + (subdir ? subdir + '/' : '');
    const note = typeof entry.note === 'string' && entry.note.trim() ? entry.note.trim() + ' ' : '';
    const shared = {
      kind: 'published',
      format: 'UORLUT01',
      schema,
      sha256: artifactSha,
      size_bytes: artifactBytes.byteLength,
      tokenizer_sha256: tokenizerSha,
      served_forward_audit_sha256: auditSha,
    };
    manifest.models.push(Object.assign({}, shared, {
      id: id,
      label: primary ? label : label + ' - this site',
      primary: primary,
      url: urlPrefix + artifactName,
      tokenizer_url: urlPrefix + tokenizerName,
      audit_url: urlPrefix + auditName,
      note: note + 'Same-origin copy served by this Studio site; the browser fetches it only after an explicit Load click.',
    }));
    if (hfUrl) {
      manifest.models.push(Object.assign({}, shared, {
        id: id + '-hf',
        label: label + ' - Hugging Face',
        url: hfUrl,
        tokenizer_url: hfTokenizerUrl || urlPrefix + tokenizerName,
        note: note + 'User-initiated cross-origin download from the host named on the card; the host must serve CORS headers.',
      }));
    }
  }
}

if (publicUrl) {
  manifest.models.push({
    id: 'public-model',
    kind: 'url',
    label: publicLabel,
    url: publicUrl,
    note: 'Optional public UORLUT01 model supplied at publish time. The browser fetches it once; the host must serve CORS headers. Its sha256 is not pinned here.',
  });
}
writeFileSync(join(dest, 'models.json'), JSON.stringify(manifest, null, 2) + '\n');
console.log('models.json: ' + manifest.models.map((m) => m.id).join(', '));
NODE

if find "$dest" -type f -size +100M | grep -q .; then
    echo "stage-studio-site: a file in $dest is over GitHub Pages' 100 MB per-file limit:" >&2
    find "$dest" -type f -size +100M >&2
    exit 1
fi

files=$(find "$dest" -type f | wc -l | tr -d ' ')
bytes=$(du -sk "$dest" | awk '{print $1 * 1024}')
echo "staged Studio: $files file(s), $bytes byte(s), manifest $(du -h "$dest/models.json" | cut -f1)"
