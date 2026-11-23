#!/usr/bin/env node
import { PUBLIC_M1_ID, publicM1Card } from './public-m1.mjs';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync, copyFileSync, mkdirSync, unlinkSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

export function writeStudioCatalog({ dest, publicUrl = '', publicLabel = 'Public model', trainedManifestPath = '', receiptGate = '' }) {
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
    note: 'Real UOR-R4 Foundry model catalog. Built-in model is pure-integer UORR4V13 Qwen3-0.6B Foundry with OPFS cache and Stitch 2-of-3 parity recovery, alongside user-configured custom OpenAI-compatible AI API endpoints.',
    sources: {
      local_file: false,
      url: publicUrl
        ? { available: true, default_url: publicUrl, label: publicLabel }
        : { available: false, note: 'set public_model_url at workflow dispatch to publish a public model URL slot' },
      openai_compatible_api: { configured_in_browser: true, encrypted_vault: true },
    },
    models: [
      {
        id: 'qwen3-0.6b-foundry',
        kind: 'stitch-packs',
        primary: true,
        label: 'UOR-R4 Qwen3-0.6B Foundry',
        format: 'UORR4V13',
        manifest_url: 'https://raw.githubusercontent.com/Casey-allard/lexlean-r4/3b04cd460735af679d912ce749e6e4bb2bccc0b0/manifest.json',
        stitch_manifest_url: 'https://raw.githubusercontent.com/Casey-allard/lexlean-r4/3b04cd460735af679d912ce749e6e4bb2bccc0b0/manifest.stitch.json',
        manifest_sha256: '9c71182fe5462b55990b7f139635d0183ce73b15210c9169d68326f65f704a27',
        stitch_manifest_sha256: '42154f44c064d364303b1695b6c941228719be953b0aa8f8b4254c2b3761cd61',
        container_sha256: '789333898aed75c3be0f1db85a4da5b50bb1067481c37a79af38c605b4488d61',
        size_bytes: 500760064,
        context_window: 4096,
        note: 'Pure-integer UORR4V13 Qwen3-0.6B fine-tuned for Foundry 16-tool webharness (workspace, Linux VM, GitHub, PrismPM, LexLean, Stitch). Cached in OPFS via Stitch 2-of-3 packs.',
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
      if (entry && Object.hasOwn(entry, 'public_release')) {
        manifest.models.push(Object.assign(publicM1Card(entry, dest), { primary: false }));
        continue;
      }
      const id = String((entry && entry.id) || '').trim();
      if (id === PUBLIC_M1_ID) throw new Error('Reserved public M1 model id: ' + id);
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
      const auditSha = requireServedForwardAudit(entry, id, artifactSha, artifactBytes.byteLength, entry.file);
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
  const ids = new Set();
  for (const card of manifest.models) {
    if (ids.has(card.id)) throw new Error('Duplicate Studio model id: ' + card.id);
    ids.add(card.id);
  }
  writeFileSync(join(dest, 'models.json'), JSON.stringify(manifest, null, 2) + '\n');
  console.log('models.json: ' + manifest.models.map((m) => m.id).join(', '));
  return manifest;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [dest, publicUrl, publicLabel, trainedManifestPath, receiptGate] = process.argv.slice(2);
  writeStudioCatalog({ dest, publicUrl, publicLabel, trainedManifestPath, receiptGate });
}
