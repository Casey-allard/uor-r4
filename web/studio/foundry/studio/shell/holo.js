// Running a published Hologram application in this browser, on any device. The .holo v4 container
// is read here (header, section table, extensions, content blobs; the HOLOVIEW v1 bundle inside),
// but nothing is trusted from this reading: the verified core runs PrismPM's own validator on the
// bytes first (validate-application, vendored verbatim from the pinned PrismPM commit), every blob is
// re-derived through the page's own BLAKE3, and only then are the View's files placed in the
// worker's holo cache under the model's holoPath of the archive address, where the worker serves
// them and refuses every cross origin request, the empty baseline a published application carries.
// Adapters only: the layout is read here, the decisions are the core's.
import { coreReady, kappa, db } from "./inference.js";

const BASE = new URL("./", import.meta.url);
const enc = new TextEncoder(), dec = new TextDecoder();
export const DIRECTORY_EXTENSION = "https://hologram.foundation/extension/application-directory/v1";
export const PRISM_EXTENSION = "https://uor.foundation/extension/prismpm-model/v1";
const KINDS = { 8: "Metadata", 14: "Extension", 15: "AppManifest", 16: "ContentBlob" };

// ---- the container: header || section table || payloads || 32 byte footer (uor-hologram writer.rs)
export function sections(bytes) {
  const v = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  if (bytes.length < 42 || dec.decode(bytes.subarray(0, 4)) !== "HOLO") throw new Error("not a .holo file");
  const version = v.getUint16(4, true);
  if (version !== 4) throw new Error("only .holo version 4 is loadable; this file is version " + version);
  const count = v.getUint16(8, true);
  const out = [];
  for (let i = 0; i < count; i++) {
    const at = 10 + i * 24;
    const kind = bytes[at], offset = Number(v.getBigUint64(at + 8, true)), length = Number(v.getBigUint64(at + 16, true));
    out.push({ kind, name: KINDS[kind] || String(kind), bytes: bytes.subarray(offset, offset + length) });
  }
  return out;
}
export const extensions = (secs) => secs.filter((s) => s.kind === 14).map((s) => {
  const n = new DataView(s.bytes.buffer, s.bytes.byteOffset, s.bytes.byteLength).getUint16(0, true);
  return { key: dec.decode(s.bytes.subarray(2, 2 + n)), bytes: s.bytes.subarray(2 + n) };
});
export const blobs = (secs) => secs.filter((s) => s.kind === 16).map((s) => ({ kappa: dec.decode(s.bytes.subarray(0, 71)), bytes: s.bytes.subarray(71) }));

// ---- HOLOVIEW v1 (hologram-live holo_view.rs): big endian lengths, strictly ordered portable paths
export function decodeView(bytes) {
  const v = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  if (dec.decode(bytes.subarray(0, 8)) !== "HOLOVIEW" || v.getUint16(8) !== 1) throw new Error("not a HOLOVIEW v1 bundle");
  let at = 10;
  const el = v.getUint32(at); at += 4;
  const entry = dec.decode(bytes.subarray(at, at + el)); at += el;
  const count = v.getUint32(at); at += 4;
  const files = [];
  for (let i = 0; i < count; i++) {
    const pl = v.getUint32(at); at += 4;
    const path = dec.decode(bytes.subarray(at, at + pl)); at += pl;
    const len = Number(v.getBigUint64(at)); at += 8;
    files.push({ path, bytes: bytes.subarray(at, at + len) }); at += len;
  }
  if (at !== bytes.length) throw new Error("View bundle has trailing bytes");
  return { entry, files };
}
export function encodeView(files) {
  const ordered = [...files].sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
  const u32 = (n) => { const b = new Uint8Array(4); new DataView(b.buffer).setUint32(0, n); return b; };
  const u64 = (n) => { const b = new Uint8Array(8); new DataView(b.buffer).setBigUint64(0, BigInt(n)); return b; };
  const parts = [enc.encode("HOLOVIEW"), new Uint8Array([0, 1]), u32(10), enc.encode("index.html"), u32(ordered.length)];
  for (const f of ordered) { const p = enc.encode(f.path); parts.push(u32(p.length), p, u64(f.bytes.length), f.bytes); }
  const out = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  let at = 0;
  for (const p of parts) { out.set(p, at); at += p.length; }
  return out;
}

// ---- bytes across the core's JSON boundary
export const b64 = (bytes) => { let s = ""; for (let i = 0; i < bytes.length; i += 0x8000) s += String.fromCharCode.apply(null, bytes.subarray(i, i + 0x8000)); return btoa(s); };
export const unb64 = (text) => Uint8Array.from(atob(text), (c) => c.charCodeAt(0));
export const hex = (bytes) => [...bytes].map((b) => b.toString(16).padStart(2, "0")).join("");
export async function sha256(bytes) { return hex(new Uint8Array(await crypto.subtle.digest("SHA-256", bytes))); }

const MIME = { html: "text/html; charset=utf-8", htm: "text/html; charset=utf-8", js: "text/javascript; charset=utf-8", mjs: "text/javascript; charset=utf-8", css: "text/css; charset=utf-8", json: "application/json", map: "application/json", svg: "image/svg+xml", png: "image/png", jpg: "image/jpeg", jpeg: "image/jpeg", webp: "image/webp", gif: "image/gif", ico: "image/x-icon", woff2: "font/woff2", woff: "font/woff", ttf: "font/ttf", txt: "text/plain; charset=utf-8", wasm: "application/wasm" };

// ---- inspect: PrismPM's validator decides; this reads what it accepted and re-derives every blob
export async function inspect(bytes) {
  const core = await coreReady();
  const verdict = core.run({ op: "validate-application", bytes: b64(bytes) }); // throws PrismPM's code on refusal
  const secs = sections(bytes);
  const address = await kappa(bytes);
  const fingerprint = hex(bytes.subarray(bytes.length - 32));
  const ext = extensions(secs);
  const directory = JSON.parse(dec.decode(ext.find((e) => e.key === DIRECTORY_EXTENSION).bytes));
  const prov = ext.find((e) => e.key === PRISM_EXTENSION);
  const provenance = prov ? JSON.parse(dec.decode(prov.bytes)) : null;
  const store = new Map(blobs(secs).map((b) => [b.kappa, b.bytes]));
  for (const [k, b] of store) if ((await kappa(b)) !== k) throw new Error("content " + k + " does not re-derive to its address");
  const viewLayer = directory.layers.find((l) => l.kind === "view");
  const view = decodeView(store.get(viewLayer.content_kappa));
  const model = provenance && store.has(provenance.model_content_kappa) ? JSON.parse(dec.decode(store.get(provenance.model_content_kappa))) : null;
  return { kappa: address, fingerprint, directory, provenance, view, model, byteLength: bytes.length, valid: verdict.valid === true, sections: secs.map((s) => ({ kind: s.name, length: s.bytes.length })) };
}

// ---- install: verify, keep the bytes at their address, unpack the View under holoPath(κ), audit
export async function install(bytes, extra = {}) {
  const t0 = performance.now();
  const info = await inspect(bytes);
  const core = await coreReady();
  const { path } = core.run({ op: "holo-path", kappa: info.kappa });
  const base = new URL(path.slice(1), BASE).pathname;
  const d = await db();
  const ref = { kappa: info.kappa, application: info.model && info.model.application, version: info.model && info.model.version, installed: Date.now(), byteLength: bytes.length, fingerprint: info.fingerprint, ...extra };
  await new Promise((res, rej) => {
    const t = d.transaction(["objects", "refs", "audit"], "readwrite");
    t.objectStore("objects").put(bytes, info.kappa);
    t.objectStore("refs").put(ref, "holo:" + info.kappa);
    // The grant the kernel's gate reads before the worker serves this application: it may be served,
    // and it names no host, so its policy admits no network (the empty capability baseline).
    t.objectStore("refs").put({ serve: true, network: { endpoints: [], denied: [] } }, "capabilities:" + base.split("/").filter(Boolean).pop());
    t.objectStore("audit").put({ timestamp_millis: Date.now(), principal: "holo:" + info.kappa, operation: "holo.application.verify", resource: info.kappa, outcome: "granted" }, Date.now() + ":" + info.kappa);
    t.oncomplete = res; t.onerror = () => rej(t.error);
  });
  const cache = await caches.open("holo");
  for (const f of info.view.files) {
    const ext = f.path.split(".").pop().toLowerCase();
    await cache.put(base + f.path, new Response(f.bytes, { headers: { "content-type": MIME[ext] || "application/octet-stream" } }));
  }
  return { ...info, ref, path: base, url: new URL(base, location.origin).href, ms: Math.round(performance.now() - t0) };
}
export async function stored(address) {
  const d = await db();
  return new Promise((res, rej) => { const r = d.transaction("objects").objectStore("objects").get(address); r.onsuccess = () => res(r.result || null); r.onerror = () => rej(r.error); });
}
export async function installed() {
  const d = await db();
  const all = await new Promise((res, rej) => { const r = d.transaction("refs").objectStore("refs").getAll(); r.onsuccess = () => res(r.result || []); r.onerror = () => rej(r.error); });
  return all.filter((r) => r && r.kappa && r.fingerprint).sort((a, b) => b.installed - a.installed);
}
