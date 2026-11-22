/**
 * Reader for the Hologram v4 `.holo` application archive that PrismPM
 * composes (`SPEC.md` §3, `crates/prismpm/src/holo/archive.rs`, encoded through
 * `Foundation.Holo.V1.Wire`).
 *
 * Layout, little-endian:
 *
 *   0      "HOLO"
 *   4      u16 physical version, 4
 *   6      u16 flags, 0
 *   8      u16 section count
 *   10     section table: per section u64 kind, u64 offset, u64 length
 *   ...    section bodies, contiguous, in table order
 *   end-32 BLAKE3 of everything before it (the archive fingerprint)
 *
 * Section kinds: 15 application manifest, 8 metadata, 14 extension, 16 content
 * blob. A content blob starts with its 71-byte `blake3:<hex>` label.
 */
import { DigestMismatchError, fail } from './errors.mjs';
import { blake3Hex, kappa } from './blake3.mjs';

export const MAGIC = 'HOLO';
export const VERSION = 4;
export const KIND = Object.freeze({ APP_MANIFEST: 15, METADATA: 8, EXTENSION: 14, CONTENT_BLOB: 16 });
export const LABEL_LENGTH = 71;
export const FOOTER_LENGTH = 32;
export const HEADER_LENGTH = 10;
export const TABLE_ENTRY_LENGTH = 24;

export const DIRECTORY_EXTENSION = 'https://hologram.foundation/extension/application-directory/v1';
export const MODEL_EXTENSION = 'https://uor.foundation/extension/prismpm-model/v1';
export const APP_MANIFEST_URI = 'https://hologram.foundation/realization/app-manifest';
/** `Foundation.Holo.V1.Wire`: `wireManifestPrefix` is 57 bytes, the manifest 356, `wireManifestSuffix` the 86 bytes below. */
export const MANIFEST_LENGTH = 356;
export const MANIFEST_PREFIX_LENGTH = 57;
export const MANIFEST_SUFFIX_HEX =
  '52000000' + '00000000020000000008000000' + '686f6c6f5f72756e' + '1a000000' + '686f6c6f6772616d3a67756573742f636f72652d7761736d4031' +
  '030a000000' + '696e6465782e68746d6c' + '08000000' + '706f727461626c65' + '00000000';

const decoder = new TextDecoder('utf-8', { fatal: true });

export function asBytes(source) {
  if (source instanceof Uint8Array) return source;
  if (source instanceof ArrayBuffer) return new Uint8Array(source);
  if (ArrayBuffer.isView(source)) return new Uint8Array(source.buffer, source.byteOffset, source.byteLength);
  fail('archive.input', 'expected bytes (Uint8Array or ArrayBuffer)');
}

function u64(view, offset) {
  const value = view.getBigUint64(offset, true);
  if (value > BigInt(Number.MAX_SAFE_INTEGER)) fail('archive.bounds', `field at ${offset} exceeds 2^53`);
  return Number(value);
}

function isLabel(bytes) {
  if (bytes.length !== LABEL_LENGTH) return false;
  for (let i = 0; i < 7; i++) if (bytes[i] !== 'blake3:'.charCodeAt(i)) return false;
  for (let i = 7; i < LABEL_LENGTH; i++) {
    const c = bytes[i];
    if (!((c >= 48 && c <= 57) || (c >= 97 && c <= 102))) return false;
  }
  return true;
}

/**
 * Parse an archive. Structural errors throw; integrity results are returned in
 * `verification` so a caller can decide what an unverified archive is worth.
 */
export function parseArchive(source) {
  const bytes = asBytes(source);
  if (bytes.length < HEADER_LENGTH + FOOTER_LENGTH) fail('archive.bounds', 'archive shorter than header and footer');
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const magic = String.fromCharCode(bytes[0], bytes[1], bytes[2], bytes[3]);
  if (magic !== MAGIC) fail('archive.magic', `expected magic ${MAGIC}, found ${JSON.stringify(magic)}`);
  const version = view.getUint16(4, true);
  if (version !== VERSION) fail('archive.version', `expected physical version ${VERSION}, found ${version}`);
  const flags = view.getUint16(6, true);
  if (flags !== 0) fail('archive.flags', `expected flags 0, found ${flags}`);
  const count = view.getUint16(8, true);
  const tableEnd = HEADER_LENGTH + count * TABLE_ENTRY_LENGTH;
  const bodyEnd = bytes.length - FOOTER_LENGTH;
  if (tableEnd > bodyEnd) fail('archive.bounds', `section table of ${count} entries overruns the archive`);

  const sections = [];
  let cursor = tableEnd;
  for (let i = 0; i < count; i++) {
    const at = HEADER_LENGTH + i * TABLE_ENTRY_LENGTH;
    const kind = u64(view, at);
    const offset = u64(view, at + 8);
    const length = u64(view, at + 16);
    if (offset !== cursor) fail('archive.bounds', `section ${i} starts at ${offset}, expected ${cursor}`);
    if (offset + length > bodyEnd) fail('archive.bounds', `section ${i} overruns the archive body`);
    sections.push({ index: i, kind, offset, length, body: bytes.subarray(offset, offset + length) });
    cursor = offset + length;
  }
  if (cursor !== bodyEnd) fail('archive.bounds', `sections end at ${cursor}, body ends at ${bodyEnd}`);

  const body = bytes.subarray(0, bodyEnd);
  const footer = blake3HexOf(bytes.subarray(bodyEnd));
  const fingerprint = blake3Hex(body);
  const labelMismatches = [];
  // Text a section holds, or a refusal: never a raw decoding error. When the footer
  // does not match either, the bytes were altered, and that is the reason to name.
  const text = (slice, check, what) => {
    try {
      return decoder.decode(slice);
    } catch {
      if (footer !== fingerprint) {
        throw new DigestMismatchError(`integrity failure: the holo was altered: the footer is not the BLAKE3 of the body, and ${what} is not UTF-8`, { check: 'holo.integrity' });
      }
      return fail(check, `${what} is not UTF-8`);
    }
  };

  let manifest = null;
  let metadata = null;
  const extensions = [];
  const blobs = [];
  try {
  for (const section of sections) {
    const b = section.body;
    switch (section.kind) {
      case KIND.APP_MANIFEST: {
        // `Foundation.Holo.V1.Wire.validAppManifest`: exactly 356 bytes, the
        // 57-byte prefix (URI, NUL, u32 LE 3), three 71-byte labels (requires,
        // guest, view), then the fixed 86-byte layer suffix.
        if (b.length !== MANIFEST_LENGTH) fail('archive.manifest', `application manifest is ${b.length} bytes, expected ${MANIFEST_LENGTH}`);
        const nul = b.indexOf(0);
        if (nul !== MANIFEST_PREFIX_LENGTH - 5) fail('archive.manifest', 'application manifest URI has the wrong length');
        const uri = text(b.subarray(0, nul), 'archive.manifest', 'the application manifest URI');
        if (uri !== APP_MANIFEST_URI) fail('archive.manifest', `application manifest URI is ${uri}`);
        const labelCount = new DataView(b.buffer, b.byteOffset + nul + 1, 4).getUint32(0, true);
        if (labelCount !== 3) fail('archive.manifest', `application manifest declares ${labelCount} labels, expected 3`);
        const labels = [];
        for (let i = 0; i < 3; i++) {
          const label = b.subarray(MANIFEST_PREFIX_LENGTH + i * LABEL_LENGTH, MANIFEST_PREFIX_LENGTH + (i + 1) * LABEL_LENGTH);
          if (!isLabel(label)) fail('archive.manifest', `application manifest label ${i} is not a blake3 label`);
          labels.push(decoder.decode(label));
        }
        const suffix = b.subarray(MANIFEST_PREFIX_LENGTH + 3 * LABEL_LENGTH);
        if (blake3HexOf(suffix) !== MANIFEST_SUFFIX_HEX) fail('archive.manifest', 'application manifest layer suffix is not the Wire constant');
        if (manifest) fail('archive.manifest', 'more than one application manifest');
        manifest = { uri, labels, suffix, section: section.index };
        break;
      }
      case KIND.METADATA: {
        if (metadata) fail('archive.metadata', 'more than one metadata section');
        metadata = { text: text(b, 'archive.metadata', `metadata section ${section.index}`), section: section.index };
        break;
      }
      case KIND.EXTENSION: {
        if (b.length < 2) fail('archive.extension', 'extension shorter than its name length');
        const nameLength = b[0] | (b[1] << 8);
        if (2 + nameLength > b.length) fail('archive.extension', 'extension name overruns the section');
        const name = text(b.subarray(2, 2 + nameLength), 'archive.extension', `the name of extension section ${section.index}`);
        const payload = b.subarray(2 + nameLength);
        extensions.push({ name, payload, text: text(payload, 'archive.extension', `extension ${name}`), section: section.index });
        break;
      }
      case KIND.CONTENT_BLOB: {
        if (b.length < LABEL_LENGTH) fail('archive.blob', 'content blob shorter than its label');
        const labelBytes = b.subarray(0, LABEL_LENGTH);
        if (!isLabel(labelBytes)) fail('archive.blob', `content blob ${blobs.length} has no blake3 label`);
        const label = decoder.decode(labelBytes);
        const content = b.subarray(LABEL_LENGTH);
        const actual = kappa(content);
        if (actual !== label) labelMismatches.push({ section: section.index, label, actual });
        blobs.push({ label, content, section: section.index });
      break;
      }
      default:
        fail('archive.kind', `unknown section kind ${section.kind} at section ${section.index}`);
    }
  }
  if (!manifest) fail('archive.manifest', 'no application manifest section');
  } catch (error) {
    // A section that does not parse in a holo whose footer does not match either was altered:
    // name the damage first, and keep what broke as the detail.
    if (footer !== fingerprint && error?.code !== DigestMismatchError.code && !(error instanceof DigestMismatchError)) {
      throw new DigestMismatchError(`integrity failure: the holo was altered: the footer is not the BLAKE3 of the body (${error.code ?? 'parse'}: ${error.message})`, { check: 'holo.integrity', cause: error });
    }
    throw error;
  }

  return {
    bytes,
    body,
    version,
    flags,
    sections,
    manifest,
    metadata,
    extensions,
    blobs,
    footer,
    fingerprint,
    verification: {
      footer: footer === fingerprint,
      labels: labelMismatches.length === 0,
      labelMismatches,
    },
  };
}

function blake3HexOf(bytes) {
  let s = '';
  for (const b of bytes) s += b.toString(16).padStart(2, '0');
  return s;
}

/** JSON with sorted keys and no whitespace, the SDK's canonical encoding (`holo/canonical.rs`). */
export function canonicalJson(value) {
  if (Array.isArray(value)) return `[${value.map(canonicalJson).join(',')}]`;
  if (value && typeof value === 'object') {
    return `{${Object.keys(value)
      .sort()
      .map((k) => `${JSON.stringify(k)}:${canonicalJson(value[k])}`)
      .join(',')}}`;
  }
  return JSON.stringify(value);
}

/**
 * Resolve the parsed archive into the application's parts, cross-checking the
 * directory and model extensions against the manifest and the blobs.
 */
export function resolveArchive(parsed) {
  const extension = (name) => parsed.extensions.find((e) => e.name === name);
  const directoryExtension = extension(DIRECTORY_EXTENSION);
  const modelExtension = extension(MODEL_EXTENSION);
  if (!directoryExtension) fail('archive.directory', `missing extension ${DIRECTORY_EXTENSION}`);
  if (!modelExtension) fail('archive.provenance', `missing extension ${MODEL_EXTENSION}`);
  const directory = JSON.parse(directoryExtension.text);
  const provenance = JSON.parse(modelExtension.text);
  const byLabel = new Map(parsed.blobs.map((b) => [b.label, b]));
  const blob = (label, what) => {
    const found = byLabel.get(label);
    if (!found) fail('archive.directory', `${what} ${label} is not a content blob of this archive`);
    return found;
  };

  const layers = Array.isArray(directory.layers) ? directory.layers : [];
  const wasmLayer = layers.find((l) => l.kind === 'wasm');
  const viewLayer = layers.find((l) => l.kind === 'view');
  if (!wasmLayer) fail('archive.directory', 'application directory has no wasm layer');
  if (!viewLayer) fail('archive.directory', 'application directory has no view layer');
  if (wasmLayer.entry !== 'holo_run') fail('archive.directory', `wasm layer entry is ${wasmLayer.entry}, expected holo_run`);
  if (typeof provenance.model_content_kappa !== 'string') fail('archive.provenance', 'model extension names no model_content_kappa');

  const guest = blob(wasmLayer.content_kappa, 'wasm layer');
  const view = blob(viewLayer.content_kappa, 'view layer');
  const model = blob(provenance.model_content_kappa, 'model document');
  const capabilities = blob(directory.requires_kappa, 'capability set');

  const consistent =
    provenance.guest_content_kappa === wasmLayer.content_kappa &&
    parsed.manifest.labels.includes(wasmLayer.content_kappa) &&
    parsed.manifest.labels.includes(viewLayer.content_kappa) &&
    parsed.manifest.labels.includes(directory.requires_kappa) &&
    parsed.manifest.uri === APP_MANIFEST_URI;

  let modelDocument;
  try {
    modelDocument = JSON.parse(decoder.decode(model.content));
  } catch (error) {
    fail('archive.model', `model document is not JSON: ${error.message}`);
  }

  return {
    directory,
    provenance,
    modelDocument,
    guest,
    view,
    model,
    capabilities,
    identities: {
      archiveFingerprint: parsed.fingerprint,
      footer: parsed.footer,
      guestKappa: guest.label,
      viewKappa: view.label,
      modelKappa: model.label,
      capabilitiesKappa: capabilities.label,
      applicationKappa: provenance.application_kappa ?? null,
      verified: parsed.verification.footer && parsed.verification.labels && consistent,
      consistent,
    },
  };
}
