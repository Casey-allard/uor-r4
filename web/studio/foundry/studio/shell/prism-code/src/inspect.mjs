/**
 * `inspect(bytes)`: what a `.holo` says about itself, read without running it.
 *
 * The result is one plain object, `prismpm/holo-inspection/1`, with the keys
 * `prismpm inspect` will print (snake_case, like every PrismPM document). It
 * reads the section table and the decoded sections. It never compiles or
 * instantiates the guest.
 *
 * Digests follow ADR-015 (`docs/adr/015-dual-digests.md`): every digest set
 * holds `sha256` and `blake3` over one byte string. A section's `digest`
 * covers exactly its table bytes, `offset` to `offset + length`. A content
 * blob also reports `content.digest` over the bytes after its label, whose
 * `blake3` equals the label: these are the values an attestation subject
 * names. The footer is BLAKE3 of the body, not of the file, so it is reported
 * as `footer`, never under a `blake3` key.
 */
import {
  parseArchive,
  resolveArchive,
  asBytes,
  KIND,
  DIRECTORY_EXTENSION,
  MODEL_EXTENSION,
  APP_MANIFEST_URI,
} from './archive.mjs';
import { blake3Hex } from './blake3.mjs';
import { sha256Hex } from './sha256.mjs';
import { fail } from './errors.mjs';
import { worldOf } from './vendor/verify.mjs';

export const INSPECTION_SCHEMA = 'prismpm/holo-inspection/1';

/**
 * The format revisions this reader recognizes, mirrored from
 * `@prism-code/spec` `FORMAT_REVISIONS` (prism-code has no dependency on it
 * yet). An archive does not carry its revision date, so a revision is
 * recognized by these identifiers. `test/inspect.test.mjs` checks that the
 * two tables agree.
 */
export const FORMAT_REVISIONS = Object.freeze({
  '2026-10-23': Object.freeze({
    physicalVersion: 4,
    flags: 0,
    sectionOrder: Object.freeze([15, 8, 14, 14, 16, 16, 16, 16]),
    appManifestUri: APP_MANIFEST_URI,
    extensions: Object.freeze([DIRECTORY_EXTENSION, MODEL_EXTENSION]),
    coreWasmContract: 'hologram:guest/core-wasm@1',
    holoviewVersion: 1,
    sourceManifestVersion: 4,
    provenanceSchema: 'prismpm/model-provenance/1',
    modelDocumentSchemas: Object.freeze([
      'prismpm/model-document/1',
      'prismpm/model-document/2',
      'prismpm/model-document/5',
      'prismpm/model-document/6',
    ]),
  }),
});

/**
 * The 86-byte manifest suffix is the `Foundation.Holo.V1.Wire` constant,
 * which `parseArchive` checks byte for byte. These are its fields.
 */
const MANIFEST_SUFFIX_FIELDS = Object.freeze({
  entry: 'holo_run',
  contract: 'hologram:guest/core-wasm@1',
  view_entry: 'index.html',
  surface: 'portable',
});

/** The capability set of an application that requests nothing (`Foundation.Holo.V1.Wire.emptyCapabilities`). */
const EMPTY_CAPABILITIES_HEX =
  '68747470733a2f2f686f6c6f6772616d2e666f756e646174696f6e2f7265616c697a6174696f6e2f6361706162696c6974792d736574' +
  '00' +
  '0000000029000000' +
  '0000000000000000000000000000000000000000000000000000000000000000000000000000000000';

const KIND_NAMES = Object.freeze({
  [KIND.APP_MANIFEST]: 'app-manifest',
  [KIND.METADATA]: 'metadata',
  [KIND.EXTENSION]: 'extension',
  [KIND.CONTENT_BLOB]: 'content-blob',
});

const decoder = new TextDecoder('utf-8', { fatal: true });

function hex(bytes) {
  let s = '';
  for (const b of bytes) s += b.toString(16).padStart(2, '0');
  return s;
}

async function digestSet(bytes) {
  return { sha256: await sha256Hex(bytes), blake3: blake3Hex(bytes) };
}

function sameList(a, b) {
  return a.length === b.length && a.every((x, i) => x === b[i]);
}

function holoviewVersion(content) {
  if (content.length < 10) return null;
  if (hex(content.subarray(0, 8)) !== hex(new TextEncoder().encode('HOLOVIEW'))) return null;
  return (content[8] << 8) | content[9];
}

function sourceManifestVersion(parsed) {
  if (!parsed.metadata) return null;
  try {
    return JSON.parse(parsed.metadata.text).version ?? null;
  } catch {
    return null;
  }
}

/** The first revision whose every identifier this archive carries, or null. */
function recognizeRevision(parsed, resolved) {
  const wasmLayer = (resolved.directory.layers ?? []).find((l) => l.kind === 'wasm');
  const observed = {
    sectionOrder: parsed.sections.map((s) => s.kind),
    extensions: parsed.extensions.map((e) => e.name),
    coreWasmContract: wasmLayer?.contract ?? null,
    holoviewVersion: holoviewVersion(resolved.view.content),
    sourceManifestVersion: sourceManifestVersion(parsed),
    provenanceSchema: resolved.provenance.schema ?? null,
    modelDocumentSchema: resolved.modelDocument.schema ?? null,
  };
  for (const [revision, id] of Object.entries(FORMAT_REVISIONS)) {
    if (
      parsed.version === id.physicalVersion &&
      parsed.flags === id.flags &&
      sameList(observed.sectionOrder, id.sectionOrder) &&
      parsed.manifest.uri === id.appManifestUri &&
      sameList(observed.extensions, id.extensions) &&
      observed.coreWasmContract === id.coreWasmContract &&
      observed.holoviewVersion === id.holoviewVersion &&
      observed.sourceManifestVersion === id.sourceManifestVersion &&
      observed.provenanceSchema === id.provenanceSchema &&
      id.modelDocumentSchemas.includes(observed.modelDocumentSchema)
    ) {
      return revision;
    }
  }
  return null;
}

function roleOf(section, parsed, resolved) {
  switch (section.kind) {
    case KIND.APP_MANIFEST:
      return 'app-manifest';
    case KIND.METADATA:
      return 'source-manifest';
    case KIND.EXTENSION: {
      const name = parsed.extensions.find((e) => e.section === section.index)?.name;
      if (name === DIRECTORY_EXTENSION) return 'application-directory';
      if (name === MODEL_EXTENSION) return 'model-provenance';
      return 'extension';
    }
    case KIND.CONTENT_BLOB: {
      const label = parsed.blobs.find((b) => b.section === section.index)?.label;
      if (label === resolved.capabilities.label) return 'capability-set';
      if (label === resolved.guest.label) return 'guest';
      if (label === resolved.view.label) return 'view';
      if (label === resolved.model.label) return 'model-document';
      return 'content-blob';
    }
    default:
      return 'unknown';
  }
}

/**
 * Inspect a `.holo` from bytes or a Blob. Structural errors throw
 * `PrismCodeError` (`archive.*`). Integrity results are reported in
 * `verification`, not thrown, so a damaged archive can still be looked at.
 */
export async function inspect(source) {
  if (typeof Blob !== 'undefined' && source instanceof Blob) source = new Uint8Array(await source.arrayBuffer());
  const bytes = asBytes(source);
  const parsed = parseArchive(bytes);
  const resolved = resolveArchive(parsed);
  const { modelDocument, provenance, directory } = resolved;
  const application = modelDocument.application ?? {};

  const sections = [];
  for (const section of parsed.sections) {
    const entry = {
      index: section.index,
      kind: section.kind,
      kind_name: KIND_NAMES[section.kind] ?? 'unknown',
      role: roleOf(section, parsed, resolved),
      offset: section.offset,
      length: section.length,
      digest: await digestSet(section.body),
    };
    if (section.kind === KIND.EXTENSION) {
      entry.name = parsed.extensions.find((e) => e.section === section.index).name;
    }
    if (section.kind === KIND.CONTENT_BLOB) {
      const blob = parsed.blobs.find((b) => b.section === section.index);
      const digest = await digestSet(blob.content);
      entry.label = blob.label;
      entry.content = { length: blob.content.length, digest, label_matches: `blake3:${digest.blake3}` === blob.label };
    }
    sections.push(entry);
  }

  const [capabilitySet, guest, view] = parsed.manifest.labels;
  const wasmLayer = (directory.layers ?? []).find((l) => l.kind === 'wasm');
  const contract = wasmLayer?.contract ?? provenance.core_wasm_contract ?? null;

  return {
    schema: INSPECTION_SCHEMA,
    name: application.name ?? provenance.cargo_name ?? null,
    revision: recognizeRevision(parsed, resolved),
    physical_version: parsed.version,
    flags: parsed.flags,
    size: bytes.length,
    digest: await digestSet(bytes),
    footer: parsed.footer,
    sections,
    manifest: {
      uri: parsed.manifest.uri,
      labels: { capability_set: capabilitySet, guest, view },
      ...MANIFEST_SUFFIX_FIELDS,
    },
    model: {
      schema: modelDocument.schema ?? null,
      name: application.name ?? null,
      cargo_name: application.cargo_name ?? null,
      cargo_version: application.cargo_version ?? null,
      description: application.cargo_description ?? null,
      profile: application.profile ?? null,
      function: application.function_name ?? null,
      vectors: Array.isArray(application.acceptance_vectors) ? application.acceptance_vectors.length : 0,
      limits: {
        request: application.request_maximum ?? null,
        response: application.response_maximum ?? null,
        guest_allocation: application.guest_allocation_maximum ?? null,
      },
      digest: await digestSet(resolved.model.content),
    },
    provenance: {
      schema: provenance.schema ?? null,
      application_kappa: provenance.application_kappa ?? null,
      model_id: provenance.model_id ?? null,
      compiler_semantics_id: provenance.compiler_semantics_id ?? null,
      stdlib_semantics_id: provenance.stdlib_semantics_id ?? null,
      lexlean_commit: provenance.lexlean_commit ?? null,
    },
    // The world comes from @prism-code/verify's one function, vendored, so the two inspections cannot disagree (issue #278).
    world: contract
      ? {
          ...worldOf(contract, resolved.guest.content),
          capability_set: {
            label: resolved.capabilities.label,
            empty: hex(resolved.capabilities.content) === EMPTY_CAPABILITIES_HEX,
          },
        }
      : null,
    // Statements inside the archive arrive with the attestation section
    // (#267). Until then no holo carries any, and the list is empty.
    attestations: [],
    verification: {
      footer: parsed.verification.footer,
      labels: parsed.verification.labels,
      cross_references: resolved.identities.consistent,
    },
  };
}

const NAME = '[A-Za-z_][A-Za-z0-9_-]*';
const SELECTOR = new RegExp(`^\\.?(?:${NAME}|\\[\\d+\\])(?:\\.${NAME}|\\.\\d+|\\[\\d+\\])*$`);
const STEP = new RegExp(`(${NAME})|\\[(\\d+)\\]|\\.(\\d+)`, 'g');

/**
 * Read one value out of an inspection by a dotted path, such as
 * `sections[0].digest.sha256`, `sections.0.role` or `world.exports`. A
 * leading `.` is allowed, as in a Go template. Returns `{ found, value }`. A
 * malformed path throws `PrismCodeError` `inspect.selector`.
 */
export function select(value, selector) {
  if (typeof selector !== 'string' || !SELECTOR.test(selector)) {
    fail('inspect.selector', `malformed selector ${JSON.stringify(selector)}; use a dotted path such as sections[0].digest.sha256`);
  }
  let current = value;
  for (const match of selector.matchAll(STEP)) {
    if (match[1] !== undefined) {
      if (current === null || typeof current !== 'object' || Array.isArray(current) || !Object.hasOwn(current, match[1])) {
        return { found: false, value: undefined };
      }
      current = current[match[1]];
    } else {
      const index = Number(match[2] ?? match[3]);
      if (!Array.isArray(current) || index >= current.length) return { found: false, value: undefined };
      current = current[index];
    }
  }
  return { found: true, value: current };
}

function table(rows, indent = '  ') {
  const widths = rows[0].map((_, i) => Math.max(...rows.map((r) => String(r[i]).length)));
  return rows
    .map((r) => indent + r.map((c, i) => (i === r.length - 1 ? String(c) : String(c).padEnd(widths[i]))).join('  '))
    .join('\n');
}

function fields(pairs, indent = '') {
  const width = Math.max(...pairs.map(([k]) => k.length)) + 1;
  const lines = [];
  for (const [key, value] of pairs) {
    const values = Array.isArray(value) ? value : [value];
    values.forEach((v, i) => lines.push(`${indent}${(i === 0 ? `${key}:` : '').padEnd(width)}  ${v}`));
  }
  return lines.join('\n');
}

const ok = (b) => (b ? 'ok' : 'FAIL');
const list = (items) => (items === null ? 'unknown contract' : items.length === 0 ? 'none' : items.join(', '));
const orNone = (v) => (v === null || v === undefined ? 'none' : String(v));

/**
 * The human view of an inspection: fixed blocks, aligned columns, full
 * digests and one fact per line, so two outputs diff cleanly.
 */
export function formatInspection(inspection) {
  const i = inspection;
  const head = fields([
    ['Name', i.name ?? 'unnamed'],
    ['Revision', i.revision ?? 'unknown'],
    ['Format', `HOLO physical version ${i.physical_version}, flags ${i.flags}`],
    ['Size', `${i.size} B`],
    ['Digest', [`sha256:${i.digest.sha256}`, `blake3:${i.digest.blake3}`]],
    ['Footer', `${i.footer} (BLAKE3 of the body)`],
  ]);
  const sections = table([
    ['INDEX', 'KIND', 'ROLE', 'OFFSET', 'LENGTH', 'SHA256', 'BLAKE3'],
    ...i.sections.map((s) => [s.index, s.kind, s.role, s.offset, s.length, s.digest.sha256, s.digest.blake3]),
  ]);
  const contents = table([
    ['INDEX', 'ROLE', 'LENGTH', 'SHA256', 'LABEL'],
    ...i.sections
      .filter((s) => s.content)
      .map((s) => [s.index, s.role, s.content.length, s.content.digest.sha256, `${s.label}${s.content.label_matches ? '' : ' (MISMATCH)'}`]),
  ]);
  const m = i.manifest;
  const manifest = fields(
    [
      ['URI', m.uri],
      ['Capability set', m.labels.capability_set],
      ['Guest', m.labels.guest],
      ['View', m.labels.view],
      ['Entry', m.entry],
      ['Contract', m.contract],
      ['View entry', m.view_entry],
      ['Surface', m.surface],
    ],
    '  ',
  );
  const d = i.model;
  const model = fields(
    [
      ['Schema', orNone(d.schema)],
      ['Name', d.name ?? 'unnamed'],
      ['Crate', d.cargo_name ? `${d.cargo_name} ${d.cargo_version ?? ''}`.trim() : 'none'],
      ['Description', orNone(d.description)],
      ['Profile', orNone(d.profile)],
      ['Function', orNone(d.function)],
      ['Vectors', String(d.vectors)],
      ['Limits', `request ${orNone(d.limits.request)} B, response ${orNone(d.limits.response)} B, guest allocation ${orNone(d.limits.guest_allocation)} B`],
      ['Digest', [`sha256:${d.digest.sha256}`, `blake3:${d.digest.blake3}`]],
    ],
    '  ',
  );
  const p = i.provenance;
  const provenance = fields(
    [
      ['Schema', orNone(p.schema)],
      ['Application', orNone(p.application_kappa)],
      ['Model ID', orNone(p.model_id)],
      ['Compiler', orNone(p.compiler_semantics_id)],
      ['Stdlib', orNone(p.stdlib_semantics_id)],
      ['LexLean', orNone(p.lexlean_commit)],
    ],
    '  ',
  );
  const w = i.world;
  const world = w
    ? fields(
        [
          ['Contract', w.contract],
          ['WIT', w.package ? `${w.package} world ${w.name}, ${w.file}` : 'unknown contract'],
          ['WIT digest', w.digest ? [`sha256:${w.digest.sha256}`, `blake3:${w.digest.blake3}`] : 'unknown contract'],
          ['Imports', list(w.imports)],
          ['Exports', list(w.exports)],
          ['Guest', w.conforms === null ? 'not checked' : w.conforms ? 'conforms to the world' : 'DOES NOT conform to the world'],
          ['Purity', w.pure === null || w.pure === undefined ? 'unknown contract' : w.pure ? 'pure: no clock, random, socket or filesystem import' : `impure: ${w.effects.join(', ')}`],
          ['Capability set', `${w.capability_set.label}${w.capability_set.empty ? ' (requests nothing)' : ''}`],
        ],
        '  ',
      )
    : '  not declared';
  const attestations =
    i.attestations.length === 0
      ? '  none'
      : table([
          ['PREDICATE', 'SUBJECTS'],
          ...i.attestations.map((a) => [a.predicate_type, (a.subjects ?? []).map((s) => s.name).join(', ')]),
        ]);
  const v = i.verification;
  const checks = fields(
    [
      ['Footer', ok(v.footer)],
      ['Labels', ok(v.labels)],
      ['Cross-references', ok(v.cross_references)],
    ],
    '  ',
  );
  return [
    head,
    '',
    'Sections:',
    sections,
    '',
    'Content:',
    contents,
    '',
    'Manifest:',
    manifest,
    '',
    'Model:',
    model,
    '',
    'Provenance:',
    provenance,
    '',
    'World:',
    world,
    '',
    'Attestations:',
    attestations,
    '',
    'Verification:',
    checks,
    '',
  ].join('\n');
}
