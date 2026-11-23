#!/usr/bin/env node
// Build offline searchable UOR-R4 / Foundry / LexLean / PrismPM / Stitch standards index
// written to <dest> (default: web/studio/foundry/studio/shell/standards-index.json).

import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(__dirname, '../..');
const defaultOut = resolve(repoRoot, 'web/studio/foundry/studio/shell/standards-index.json');
const MAX_TOTAL_BYTES = 110 * 1024;

function readTrimmed(relPath, maxChars = 6000) {
  const full = resolve(repoRoot, relPath);
  if (!existsSync(full)) return '';
  const text = readFileSync(full, 'utf8').replace(/\r\n/g, '\n');
  return text.length > maxChars ? text.slice(0, maxChars).trimEnd() + '\n...[truncated]' : text.trim();
}

function splitMarkdownSections(relPath, topic, defaultTags, maxSectionChars = 2200) {
  const raw = readTrimmed(relPath, 24000);
  if (!raw) return [];
  const lines = raw.split('\n');
  const sections = [];
  let title = relPath;
  let buf = [];
  const flush = () => {
    const content = buf.join('\n').trim();
    if (!content) return;
    const clipped = content.length > maxSectionChars ? content.slice(0, maxSectionChars).trimEnd() + '\n...' : content;
    const slug = `${topic}-${title}`.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '').slice(0, 64);
    const firstPara = clipped.split('\n').find((l) => l.trim() && !l.startsWith('#')) || clipped.slice(0, 180);
    sections.push({
      id: `${slug}-${sections.length}`,
      title,
      topic,
      tags: [...new Set([topic, ...defaultTags, ...title.toLowerCase().split(/\W+/).filter((w) => w.length > 2)])].slice(0, 14),
      summary: firstPara.slice(0, 220),
      content: clipped,
      source: relPath,
    });
  };
  for (const line of lines) {
    const heading = line.match(/^#{1,3}\s+(.+)$/);
    if (heading) {
      flush();
      title = heading[1].trim();
      buf = [line];
    } else {
      buf.push(line);
    }
  }
  flush();
  return sections;
}

export function buildStandardsIndex(outPath = defaultOut) {
  const sections = [
    ...splitMarkdownSections('CHARTER.md', 'charter', ['rules', 'geometry', 'native-space', 'milestones'], 2000),
    ...splitMarkdownSections('AGENTS.md', 'agents', ['onboarding', 'constraints', 'harness', 'delivery'], 1800),
  ];

  // Key Decisions D84, D85, D90 from docs/integration/DECISIONS.md
  const decRaw = readTrimmed('docs/integration/DECISIONS.md', 200000);
  if (decRaw) {
    for (const id of ['D80', 'D81', 'D82', 'D83', 'D84', 'D85', 'D90']) {
      const re = new RegExp(`^\\|\\s*${id}\\s*\\|.*$`, 'm');
      const m = decRaw.match(re);
      if (m) {
        const content = m[0].slice(0, 2200);
        sections.push({
          id: `decision-${id.toLowerCase()}`,
          title: `Decision ${id}`,
          topic: 'decisions',
          tags: ['decisions', id.toLowerCase(), id === 'D90' ? 'studio' : 'architecture', id === 'D85' ? 'native-space' : 'uor-r4'],
          summary: content.slice(0, 220),
          content,
          source: 'docs/integration/DECISIONS.md',
        });
      }
    }
  }

  // Real Foundry R4 Judge spec from web/studio/foundry/studio/shell/r4/spec.json
  const r4SpecRaw = readTrimmed('web/studio/foundry/studio/shell/r4/spec.json', 8000);
  if (r4SpecRaw) {
    sections.push({
      id: 'prism-r4-judge-spec',
      title: 'Prism R4 Judge Specification (r4/spec.json)',
      topic: 'r4-policy',
      tags: ['r4', 'prism', 'policy', 'weights', 'engine', 'boundary'],
      summary: 'Compiled Prism R4 admission rules gating in-browser integer WASM and Prism IdeaReader outputs.',
      content: r4SpecRaw.slice(0, 2400),
      source: 'web/studio/foundry/studio/shell/r4/spec.json',
    });
  }

  // Real Foundry specification & LexLean header
  const foundryLex = readTrimmed('web/studio/foundry/app/src/FoundrySpecification.lex.tex', 4500);
  if (foundryLex) {
    sections.push({
      id: 'foundry-specification-lexlean',
      title: 'FoundrySpecification LexLean 0.3.0 Kernel Module',
      topic: 'lexlean',
      tags: ['lexlean', 'foundry', 'specification', 'semantic-module', 'boundaries'],
      summary: 'Formal LexLean semantic module defining Foundry host, storage, preview, build, and chat queue boundaries.',
      content: foundryLex.slice(0, 2400),
      source: 'web/studio/foundry/app/src/FoundrySpecification.lex.tex',
    });
  }

  // Grounded technical standards for Foundry webharness subsystems
  sections.push(
    {
      id: 'lexlean-semantic-module-contract',
      title: 'LexLean 0.3.0 & Prism Spec Authoring Standard',
      topic: 'lexlean',
      tags: ['lexlean', 'prism', 'spec', 'theorems', 'verification', 'lean'],
      summary: 'How Foundry checks and proves decision (<prism-spec>) and stateful (<prism-lexlean>) modules in-browser.',
      content: [
        '# LexLean & Prism Verification in Foundry Studio',
        '- Decision apps: stored in `prism/spec.json` and rendered canonically via `lexlean.mjs` (`toLexLean`), verified in-browser against acceptance vectors and proved by Lean in `prism.js`.',
        '- Stateful apps: stored in `prism/App.lex.tex` with declarations `State`, `catalogue`, `initial`, `view`, `step`, and `theorem` obligations checked via `live/app.mjs` (`checkApp`).',
        '- Use `lexlean_check(path)` to parse, typecheck, and run semantic checks on a `.lex.tex` or `prism/spec.json` file, and `lexlean_verify(path, profile?)` to evaluate theorems / obligation checks.',
      ].join('\n'),
      source: 'web/studio/foundry/studio/shell/prism.js',
    },
    {
      id: 'prismpm-holo-v4-standard',
      title: 'PrismPM .holo v4 Capsule & Core Closure Standard',
      topic: 'prismpm',
      tags: ['prismpm', 'holo', 'closure', 'blake3', 'cha-cha20', 'core-wasm'],
      summary: 'Portable Hologram v4 archives compose view, model, source manifest, provenance, and guest.wasm with BLAKE3 addressing.',
      content: [
        '# PrismPM .holo v4 Verification Standard',
        '- Every Studio project version is addressed by BLAKE3 (`kappa`) over deterministic preimages verified by `core.wasm`.',
        '- Portable publication (`holo:publish`) requires the shell closure `manifest.json` SHA-256 to match `provenance.json` (`browser_projection_sha256`).',
        '- Secrets (such as GitHub PATs in `refs["vault:github:credentials"]`) are sealed synchronously in `core.wasm` using ChaCha20-Poly1305 (`holo_seal`/`holo_open`) under `frame.workspaceKey()`.',
      ].join('\n'),
      source: 'web/studio/foundry/studio/shell/holo.js',
    },
    {
      id: 'stitch-protocol-standard',
      title: 'Stitch Networking & 2-of-3 XOR Parity Standard',
      topic: 'stitch',
      tags: ['stitch', 'xor-2-of-3', 'webrtc', 'signal', 'afflom', 'networking', 'shards'],
      summary: 'Stitch provides `stitch/xor-2-of-3/1` bytewise shard recovery, `stitch.signal/1` WebRTC peering, and `uor-workspace-bridge/1` (`afflom/me`).',
      content: [
        '# Stitch Protocol & afflom/me Standard',
        '- `stitch/xor-2-of-3/1`: Pairs adjacent model pieces (`left`, `right`, `parity = left ^ right`). Loss or SHA-256 mismatch on either shard heals deterministically from the remaining 2 shards.',
        '- `stitch.signal/1`: Ed25519-authenticated WebRTC `RTCPeerConnection` signaling channel connecting browser VMs (`ne2k` `net0-send`/`net0-receive`) and `afflom/me` app launch (`app.json`, `embed=me`, `uor-workspace-bridge/1`).',
      ].join('\n'),
      source: 'web/studio/foundry/studio/shell/uor-v13-worker.mjs',
    },
    {
      id: 'foundry-v13-runtime-standard',
      title: 'UOR-R4 Foundry V13 Pure-Integer Serving & 16-Tool Webharness',
      topic: 'runtime',
      tags: ['uorr4v13', 'qwen3', 'tools', 'workspace', 'github', 'linux', 'v86', 'mvm'],
      summary: 'Default Studio runtime serves pure-integer Qwen3-0.6B (`UORR4V13`) with OPFS shard caching and 16 native browser tools.',
      content: [
        '# UOR-R4 V13 Webharness Standard',
        '- Engine: `uor_foundry_v13_wasm` (`compute_float_ops: 0`, `dense_matmul_ops: 0` across all 49 `HoloV13Model` functions).',
        '- Editing app files in Build mode uses `<prism-write path="..." description="...">...</prism-write>` or tool `workspace_write`.',
        '- Shell commands execute inside the in-browser Linux 6.18.7 VM (`linux_exec`, `/workspace` 9p synced to Studio IndexedDB) or the RV64IMAC MVM sandbox (`mvm_exec`).',
      ].join('\n'),
      source: 'web/studio/foundry/studio/shell/r4-agent-loop.mjs',
    },
  );

  // Enforce strict total size bound < 110 KB so shell closure stays lean.
  const index = {
    schema: 'uor-r4.standards-index/1',
    generated_at: new Date().toISOString(),
    section_count: sections.length,
    sections,
  };
  let encoded = JSON.stringify(index);
  while (Buffer.byteLength(encoded, 'utf8') > MAX_TOTAL_BYTES && sections.length > 6) {
    sections.pop();
    index.section_count = sections.length;
    encoded = JSON.stringify(index);
  }
  mkdirSync(dirname(outPath), { recursive: true });
  writeFileSync(outPath, encoded + '\n', 'utf8');
  return {
    path: outPath,
    bytes: Buffer.byteLength(encoded, 'utf8') + 1,
    sha256: createHash('sha256').update(encoded + '\n', 'utf8').digest('hex'),
    sections: sections.length,
  };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const out = process.argv[2] ? resolve(process.argv[2]) : defaultOut;
  const info = buildStandardsIndex(out);
  console.log(JSON.stringify(info));
}
