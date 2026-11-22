/**
 * HOLOVIEW v1, the portable view bundle (hologram-live ADR 018). Big-endian:
 *
 *   "HOLOVIEW"  u16 version 1
 *   u32 entry path length, entry path
 *   u32 file count
 *   per file, in strict byte-lexical path order:
 *     u32 path length, path, u64 content length, content
 */
import { fail } from './errors.mjs';

const MAGIC = 'HOLOVIEW';
const VERSION = 1;
const decoder = new TextDecoder('utf-8', { fatal: true });
const encoder = new TextEncoder();

function compareBytes(a, b) {
  const n = Math.min(a.length, b.length);
  for (let i = 0; i < n; i++) if (a[i] !== b[i]) return a[i] - b[i];
  return a.length - b.length;
}

export function decodeHoloview(bytes) {
  if (bytes.length < 8 + 2 + 4 + 4) fail('view.bounds', 'HOLOVIEW shorter than its header');
  for (let i = 0; i < 8; i++) if (bytes[i] !== MAGIC.charCodeAt(i)) fail('view.magic', 'not a HOLOVIEW bundle');
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const version = view.getUint16(8, false);
  if (version !== VERSION) fail('view.version', `expected HOLOVIEW version ${VERSION}, found ${version}`);
  let at = 10;
  const entryLength = view.getUint32(at, false);
  at += 4;
  if (at + entryLength > bytes.length) fail('view.bounds', 'entry path overruns the bundle');
  const entry = decoder.decode(bytes.subarray(at, at + entryLength));
  at += entryLength;
  const count = view.getUint32(at, false);
  at += 4;
  const files = new Map();
  let previous = null;
  for (let i = 0; i < count; i++) {
    if (at + 4 > bytes.length) fail('view.bounds', `file ${i} overruns the bundle`);
    const pathLength = view.getUint32(at, false);
    at += 4;
    if (at + pathLength + 8 > bytes.length) fail('view.bounds', `file ${i} path overruns the bundle`);
    const pathBytes = bytes.subarray(at, at + pathLength);
    const path = decoder.decode(pathBytes);
    at += pathLength;
    const length = view.getBigUint64(at, false);
    at += 8;
    if (length > BigInt(bytes.length - at)) fail('view.bounds', `file ${path} content overruns the bundle`);
    const size = Number(length);
    if (previous && compareBytes(previous, pathBytes) >= 0) fail('view.order', `file ${path} is not in strict lexical order`);
    if (files.has(path)) fail('view.order', `file ${path} appears twice`);
    files.set(path, bytes.subarray(at, at + size));
    at += size;
    previous = pathBytes;
  }
  if (at !== bytes.length) fail('view.bounds', `${bytes.length - at} trailing bytes after the last file`);
  if (!files.has(entry)) fail('view.entry', `entry ${entry} is not one of the bundled files`);
  return { entry, files };
}

export function encodeHoloview({ entry, files }) {
  const entries = [...files.entries()]
    .map(([path, content]) => ({ path: encoder.encode(path), content }))
    .sort((a, b) => compareBytes(a.path, b.path));
  const entryBytes = encoder.encode(entry);
  let total = 8 + 2 + 4 + entryBytes.length + 4;
  for (const e of entries) total += 4 + e.path.length + 8 + e.content.length;
  const out = new Uint8Array(total);
  const view = new DataView(out.buffer);
  out.set(encoder.encode(MAGIC), 0);
  view.setUint16(8, VERSION, false);
  let at = 10;
  view.setUint32(at, entryBytes.length, false);
  at += 4;
  out.set(entryBytes, at);
  at += entryBytes.length;
  view.setUint32(at, entries.length, false);
  at += 4;
  for (const e of entries) {
    view.setUint32(at, e.path.length, false);
    at += 4;
    out.set(e.path, at);
    at += e.path.length;
    view.setBigUint64(at, BigInt(e.content.length), false);
    at += 8;
    out.set(e.content, at);
    at += e.content.length;
  }
  return out;
}
