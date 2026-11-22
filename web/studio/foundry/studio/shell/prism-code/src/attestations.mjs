/**
 * The attestation bundle section of a `.holo` (issue #267, design
 * `docs/proposals/267-attestation-bundle.md`).
 *
 * The section is an ordinary Extension row (kind 14), the last row of the
 * table, named `https://uor.foundation/extension/attestation-bundle/v1`. Its
 * payload is JSON Lines: one Sigstore bundle v0.3 per line, each line ended by
 * LF. Each bundle holds one DSSE envelope whose payload is an in-toto
 * Statement v1. Statement subjects name content blobs of the archive with both
 * `sha256` and `blake3` (ADR-015), never the archive or its footer, because the
 * footer covers the bundle.
 *
 * This module reads and writes the section. It does not verify signatures;
 * that is `prism verify` (#272) and `@prism-code/verify` (#273).
 */
import { fail } from './errors.mjs';
import { asBytes, parseArchive, KIND, MAGIC, VERSION, HEADER_LENGTH, TABLE_ENTRY_LENGTH, FOOTER_LENGTH } from './archive.mjs';
import { blake3 } from './blake3.mjs';

export const ATTESTATION_EXTENSION = 'https://uor.foundation/extension/attestation-bundle/v1';
export const BUNDLE_MEDIA_TYPE = 'application/vnd.dev.sigstore.bundle.v0.3+json';
export const STATEMENT_TYPE = 'https://in-toto.io/Statement/v1';
export const DSSE_PAYLOAD_TYPE = 'application/vnd.in-toto+json';
export const OCI_MANIFEST_SUBJECT = 'oci-manifest';

const encoder = new TextEncoder();
const decoder = new TextDecoder('utf-8', { fatal: true });
const LF = 0x0a;
const HEX64 = /^[0-9a-f]{64}$/;
const BASE64 = /^[A-Za-z0-9+/_-]*={0,2}$/;

function isObject(value) {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function parsedOf(source) {
  if (source && typeof source === 'object' && Array.isArray(source.sections) && source.bytes instanceof Uint8Array) return source;
  return parseArchive(asBytes(source));
}

/** Standard or URL-safe base64, padding optional, to bytes. Works in Node and browsers. */
function base64Bytes(text, what) {
  if (typeof text !== 'string' || !BASE64.test(text)) fail('attestation.envelope', `${what} is not base64`);
  let normal = text.replaceAll('-', '+').replaceAll('_', '/').replace(/=+$/, '');
  if (normal.length % 4 === 1) fail('attestation.envelope', `${what} has an impossible base64 length`);
  normal += '='.repeat((4 - (normal.length % 4)) % 4);
  const binary = atob(normal);
  const out = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) out[i] = binary.charCodeAt(i);
  return out;
}

/**
 * The attestation row of a parsed archive, or null. Throws when the row is
 * repeated or is not the last row of the table.
 */
export function attestationSection(source) {
  const parsed = parsedOf(source);
  const rows = parsed.extensions.filter((e) => e.name === ATTESTATION_EXTENSION);
  if (rows.length === 0) return null;
  if (rows.length > 1) fail('attestation.duplicate', `${rows.length} attestation sections, at most one is allowed`);
  const [row] = rows;
  if (row.section !== parsed.sections.length - 1) {
    fail('attestation.position', `attestation section is row ${row.section}, it must be the last row (${parsed.sections.length - 1})`);
  }
  return { section: row.section, payload: row.payload };
}

/** Split a JSON Lines payload into line byte views, without their LF. */
function splitLines(payload) {
  if (payload.length === 0) fail('attestation.lines', 'attestation section holds no bundle');
  if (payload[payload.length - 1] !== LF) fail('attestation.lines', 'the last bundle line does not end with LF');
  const lines = [];
  let start = 0;
  for (let i = 0; i < payload.length; i++) {
    if (payload[i] !== LF) continue;
    if (i === start) fail('attestation.lines', `line ${lines.length} is empty`);
    lines.push(payload.subarray(start, i));
    start = i + 1;
  }
  return lines;
}

function digestSet(digest, where) {
  if (!isObject(digest)) fail('attestation.subject', `${where} has no digest set`);
  const keys = Object.keys(digest).sort();
  if (keys.length !== 2 || keys[0] !== 'blake3' || keys[1] !== 'sha256') {
    fail('attestation.subject', `${where} must carry exactly blake3 and sha256 (ADR-015), found ${keys.join(', ') || 'none'}`);
  }
  for (const key of keys) {
    if (typeof digest[key] !== 'string' || !HEX64.test(digest[key])) fail('attestation.subject', `${where} ${key} is not 64 lowercase hex characters`);
  }
  return { blake3: digest.blake3, sha256: digest.sha256 };
}

/**
 * Decode one bundle line. Structural problems throw `attestation.*`; what a
 * signature proves is left to a verifier.
 */
export function decodeBundle(line, labels = new Set()) {
  const bytes = typeof line === 'string' ? encoder.encode(line) : asBytes(line);
  if (bytes.includes(LF)) fail('attestation.lines', 'a bundle line contains LF; serialize each bundle on one line');
  let bundle;
  try {
    bundle = JSON.parse(decoder.decode(bytes));
  } catch (error) {
    fail('attestation.bundle', `bundle line is not UTF-8 JSON: ${error.message}`);
  }
  if (!isObject(bundle)) fail('attestation.bundle', 'bundle is not a JSON object');
  if (bundle.mediaType !== BUNDLE_MEDIA_TYPE) fail('attestation.bundle', `bundle mediaType is ${JSON.stringify(bundle.mediaType)}, expected ${BUNDLE_MEDIA_TYPE}`);
  if (!isObject(bundle.verificationMaterial)) fail('attestation.bundle', 'bundle has no verificationMaterial');
  if ('messageSignature' in bundle) fail('attestation.bundle', 'bundle carries a messageSignature; Prism bundles carry a DSSE envelope');
  const envelope = bundle.dsseEnvelope;
  if (!isObject(envelope)) fail('attestation.envelope', 'bundle has no dsseEnvelope');
  if (envelope.payloadType !== DSSE_PAYLOAD_TYPE) fail('attestation.envelope', `DSSE payloadType is ${JSON.stringify(envelope.payloadType)}, expected ${DSSE_PAYLOAD_TYPE}`);
  if (!Array.isArray(envelope.signatures) || envelope.signatures.length === 0) fail('attestation.envelope', 'DSSE envelope has no signature');
  for (const [i, signature] of envelope.signatures.entries()) {
    if (!isObject(signature)) fail('attestation.envelope', `DSSE signature ${i} is not an object`);
    base64Bytes(signature.sig, `DSSE signature ${i}`);
  }
  const payload = base64Bytes(envelope.payload, 'DSSE payload');
  let statement;
  try {
    statement = JSON.parse(decoder.decode(payload));
  } catch (error) {
    fail('attestation.statement', `DSSE payload is not UTF-8 JSON: ${error.message}`);
  }
  if (!isObject(statement) || statement._type !== STATEMENT_TYPE) fail('attestation.statement', `statement _type is not ${STATEMENT_TYPE}`);
  if (typeof statement.predicateType !== 'string' || statement.predicateType.length === 0) fail('attestation.statement', 'statement has no predicateType');
  if (!Array.isArray(statement.subject) || statement.subject.length === 0) fail('attestation.statement', 'statement has no subject');
  const subjects = statement.subject.map((subject, i) => {
    const where = `subject ${i}`;
    if (!isObject(subject) || typeof subject.name !== 'string' || subject.name.length === 0) fail('attestation.subject', `${where} has no name`);
    if (subject.name === OCI_MANIFEST_SUBJECT) {
      fail('attestation.subject', 'a statement inside the holo cannot name the OCI manifest; that statement is signed at push');
    }
    const digest = digestSet(subject.digest, where);
    const label = `blake3:${digest.blake3}`;
    return { name: subject.name, digest, blob: labels.has(label) ? label : null };
  });
  return {
    bytes,
    bundle,
    mediaType: bundle.mediaType,
    verificationMaterial: bundle.verificationMaterial,
    envelope,
    /** At least one signature is non-empty. An unsigned placeholder (#268) has one empty signature; no verifier accepts it. */
    signed: envelope.signatures.some((signature) => signature.sig !== ''),
    statement,
    predicateType: statement.predicateType,
    subjects,
    /** Every subject's blake3 equals the label of a content blob in this archive. sha256 is the verifier's check. */
    resolved: subjects.every((s) => s.blob !== null),
  };
}

/**
 * Every bundle in the archive's attestation section, in section order. An
 * archive without the section returns an empty array.
 */
export function readAttestations(source) {
  const parsed = parsedOf(source);
  const row = attestationSection(parsed);
  if (!row) return [];
  const labels = new Set(parsed.blobs.map((b) => b.label));
  return splitLines(row.payload).map((line, index) => ({ index, ...decodeBundle(line, labels) }));
}

/**
 * Encode a physical v4 archive from its rows: header, table, contiguous bodies,
 * then the BLAKE3 footer over every earlier byte.
 */
export function encodeArchive(sections, { version = VERSION, flags = 0 } = {}) {
  if (sections.length > 0xffff) fail('archive.bounds', 'more than 65535 sections');
  const tableEnd = HEADER_LENGTH + sections.length * TABLE_ENTRY_LENGTH;
  const bodyLength = sections.reduce((n, s) => n + s.body.length, tableEnd);
  const out = new Uint8Array(bodyLength + FOOTER_LENGTH);
  const view = new DataView(out.buffer);
  for (let i = 0; i < 4; i++) out[i] = MAGIC.charCodeAt(i);
  view.setUint16(4, version, true);
  view.setUint16(6, flags, true);
  view.setUint16(8, sections.length, true);
  let offset = tableEnd;
  sections.forEach((section, i) => {
    const at = HEADER_LENGTH + i * TABLE_ENTRY_LENGTH;
    view.setBigUint64(at, BigInt(section.kind), true);
    view.setBigUint64(at + 8, BigInt(offset), true);
    view.setBigUint64(at + 16, BigInt(section.body.length), true);
    out.set(section.body, offset);
    offset += section.body.length;
  });
  out.set(blake3(out.subarray(0, bodyLength)), bodyLength);
  return out;
}

function extensionBody(name, payload) {
  const nameBytes = encoder.encode(name);
  const body = new Uint8Array(2 + nameBytes.length + payload.length);
  body[0] = nameBytes.length & 0xff;
  body[1] = nameBytes.length >> 8;
  body.set(nameBytes, 2);
  body.set(payload, 2 + nameBytes.length);
  return body;
}

function lineOf(bundle) {
  if (typeof bundle === 'string') return encoder.encode(bundle);
  if (bundle instanceof Uint8Array || bundle instanceof ArrayBuffer || ArrayBuffer.isView(bundle)) return asBytes(bundle);
  if (isObject(bundle)) return encoder.encode(JSON.stringify(bundle));
  fail('attestation.input', 'a bundle is an object, a JSON string or UTF-8 bytes');
}

function hex(bytes) {
  let s = '';
  for (const b of bytes) s += b.toString(16).padStart(2, '0');
  return s;
}

/**
 * Return a new archive with `bundles` added to its attestation section,
 * creating the section as the last row when absent. Every other row's body is
 * copied byte for byte; offsets shift by one table row and the footer is
 * recomputed. The input must already verify, so nothing is signed over a
 * corrupted archive. Pure: the input is not modified.
 */
export function appendAttestations(source, bundles) {
  const parsed = parsedOf(source);
  if (!parsed.verification.footer) fail('attestation.footer', 'the archive footer does not verify; refusing to append');
  if (!Array.isArray(bundles) || bundles.length === 0) fail('attestation.input', 'appendAttestations takes a non-empty array of bundles');
  const existing = attestationSection(parsed);
  const labels = new Set(parsed.blobs.map((b) => b.label));
  const lines = existing ? splitLines(existing.payload) : [];
  const seen = new Set(lines.map(hex));
  for (const bundle of bundles) {
    const line = lineOf(bundle);
    decodeBundle(line, labels);
    const key = hex(line);
    if (seen.has(key)) fail('attestation.duplicate', 'the same bundle bytes are already in the archive');
    seen.add(key);
    lines.push(line);
  }
  const payload = new Uint8Array(lines.reduce((n, l) => n + l.length + 1, 0));
  let at = 0;
  for (const line of lines) {
    payload.set(line, at);
    payload[line.length + at] = LF;
    at += line.length + 1;
  }
  const kept = parsed.sections.filter((s) => !existing || s.index !== existing.section).map(({ kind, body }) => ({ kind, body }));
  kept.push({ kind: KIND.EXTENSION, body: extensionBody(ATTESTATION_EXTENSION, payload) });
  return encodeArchive(kept, { version: parsed.version, flags: parsed.flags });
}

/**
 * Return the archive without its attestation section. For an archive produced
 * by `appendAttestations` from a verified archive this is byte-identical to the
 * original: the unsigned archive is recoverable, so reproducibility checks
 * compare it rather than the signed file.
 */
export function stripAttestations(source) {
  const parsed = parsedOf(source);
  const existing = attestationSection(parsed);
  if (!existing) return parsed.bytes.slice();
  const kept = parsed.sections.filter((s) => s.index !== existing.section).map(({ kind, body }) => ({ kind, body }));
  return encodeArchive(kept, { version: parsed.version, flags: parsed.flags });
}
