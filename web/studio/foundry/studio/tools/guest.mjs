// The wire vectors through the wasm guest, exactly as the page calls it: holo_alloc, holo_run,
// holo_free over the JSON ABI. Run from the repo root: node tools/guest.mjs
import { readFileSync } from "node:fs";
const wasm = readFileSync(new URL("../shell/core.wasm", import.meta.url));
const vectors = JSON.parse(readFileSync(new URL("../model/wire.json", import.meta.url), "utf8"));
const { instance } = await WebAssembly.instantiate(wasm, {});
const { memory, holo_alloc, holo_free, holo_run } = instance.exports;
const enc = new TextEncoder(), dec = new TextDecoder();
function run(request) {
  const bytes = enc.encode(JSON.stringify(request));
  const ptr = holo_alloc(bytes.length);
  new Uint8Array(memory.buffer, ptr, bytes.length).set(bytes);
  const packed = holo_run(ptr, bytes.length);
  const outPtr = Number(packed >> 32n), outLen = Number(packed & 0xffffffffn);
  const text = dec.decode(new Uint8Array(memory.buffer, outPtr, outLen));
  holo_free(outPtr, outLen); holo_free(ptr, bytes.length);
  const out = JSON.parse(text); if (out.error) throw new Error(out.error); return out;
}
let checked = 0;
for (const v of vectors) {
  const c = v.completion;
  const got = {
    completion_bytes: run({ op: "encode-completion", completion: c }).bytes,
    role_bytes: run({ op: "encode-role", completion: c }).bytes,
    delta_bytes: run({ op: "encode-delta", completion: c, delta: v.delta }).bytes,
    final_bytes: run({ op: "encode-final", completion: c }).bytes,
    error_bytes: run({ op: "encode-error", message: v.error.message, type: v.error.type }).bytes,
    models_bytes: run({ op: "encode-models", ids: v.ids }).bytes,
    done_bytes: run({ op: "done" }).bytes,
  };
  for (const [k, want] of Object.entries(got)) {
    if (v[k] !== want) { console.error(`guest: ${v.name}.${k} differs\n want ${v[k].slice(0, 200)}\n got  ${want.slice(0, 200)}`); process.exit(1); }
    checked++;
  }
}
const cases = JSON.parse(readFileSync(new URL("../model/inference.json", import.meta.url), "utf8"));
let orChecked = 0;
for (const c of cases) {
  // A JS number cannot carry a u64 seed above 2^53; the page cannot send one either. The crate test
  // covers that case; the guest skips it and says so.
  if (c.seed != null && !Number.isSafeInteger(c.seed)) { console.log(`guest: ${c.name} skipped for OpenRouter bytes (seed above 2^53 is not a JS number)`); continue; }
  const request = { model: "webgpu:Qwen", messages: c.messages, max_tokens: c.max_tokens, seed: c.seed, temperature: c.temperature == null ? "" : c.temperature };
  for (const stream of [true, false]) {
    const got = run({ op: "encode-openrouter-request", model: c.openrouter_model, request, stream }).bytes;
    const want = stream ? c.openrouter_stream : c.openrouter_plain;
    if (got !== want) { console.error(`guest: openrouter ${c.name} stream=${stream} differs\n want ${want.slice(0, 200)}\n got  ${got.slice(0, 200)}`); process.exit(1); }
    orChecked++;
  }
}
const rows = [[true, false, "Serve"], [false, true, "Local"], [false, false, "NoGpu"]];
for (const [hit, gpuReady, want] of rows) {
  const got = run({ op: "route", hit, gpuReady }).route;
  if (got !== want) { console.error(`guest: route(${hit},${gpuReady}) = ${got}, want ${want}`); process.exit(1); }
}
const readiness = [[true, false, false, true], [false, true, true, true], [false, true, false, false], [false, false, true, false], [false, false, false, false]];
for (const [resident, keyPresent, online, want] of readiness) {
  const got = run({ op: "endpoint-ready", resident, keyPresent, online }).ready;
  if (got !== want) { console.error(`guest: endpointReady(${resident},${keyPresent},${online}) = ${got}, want ${want}`); process.exit(1); }
}
// The κ object through the guest: the real edge0-8b manifest's root preimage, and the page rules.
const ladder = [
  ["pack-rank", { section: "header" }, "rank", 0], ["pack-rank", { section: "spine" }, "rank", 2], ["pack-rank", { section: "table" }, "packed", false], ["pack-rank", { section: "expert" }, "packed", true],
  ["first-token-ready", { spinePresent: true, promptPagesPresent: true }, "ready", true], ["first-token-ready", { spinePresent: true, promptPagesPresent: false }, "ready", false],
  ["promote", { current: "large", largeResident: false, largeFast: false }, "tier", "Large"], ["promote", { current: "small", largeResident: true, largeFast: true }, "tier", "Large"], ["promote", { current: "small", largeResident: true, largeFast: false }, "tier", "Small"],
  ["loader-start", { shellOnDevice: true, snapshotOnDevice: true }, "start", "Resume"], ["loader-start", { shellOnDevice: true, snapshotOnDevice: false }, "start", "Warm"], ["loader-start", { shellOnDevice: false, snapshotOnDevice: false }, "start", "Cold"],
];
for (const [op, input, key, want] of ladder) { const got = run({ op, ...input })[key]; if (got !== want) { console.error(`guest: ${op} ${JSON.stringify(input)} = ${got}, want ${want}`); process.exit(1); } }
console.log(`guest: ${ladder.length} pack, ladder and loader rows identical through core.wasm`);
const pool = [
  ["page-action", { resident: true, staging: "true-routing" }, "action", "Bind"], ["page-action", { resident: false, staging: "true-routing" }, "action", "Fetch"], ["page-action", { resident: false, staging: "staged-replace" }, "action", "Drop"],
  ["pool-admit", { present: true, spaceLeft: false }, "admission", "Touch"], ["pool-admit", { present: false, spaceLeft: true }, "admission", "Insert"], ["pool-admit", { present: false, spaceLeft: false }, "admission", "EvictThenInsert"],
  ["fetch-source", { onDevice: true, onMirror: true, peerFaster: true }, "source", "Device"], ["fetch-source", { onDevice: false, onMirror: true, peerFaster: true }, "source", "Peer"], ["fetch-source", { onDevice: false, onMirror: true, peerFaster: false }, "source", "Mirror"], ["fetch-source", { onDevice: false, onMirror: false, peerFaster: false }, "source", "Nowhere"],
  ["prefetch-order", { predicted: true, popular: false }, "priority", "First"], ["prefetch-order", { predicted: false, popular: true }, "priority", "Fill"], ["prefetch-order", { predicted: false, popular: false }, "priority", "Skip"],
];
for (const [op, input, key, want] of pool) { const got = run({ op, ...input })[key]; if (got !== want) { console.error(`guest: ${op} ${JSON.stringify(input)} = ${got}, want ${want}`); process.exit(1); } }
console.log(`guest: ${pool.length} pool and stage rows identical through core.wasm`);
for (const name of ["qwen38-flash-next"]) {
  const mf = JSON.parse(readFileSync(new URL(`../model/objects/${name}.manifest.json`, import.meta.url), "utf8"));
  const want = readFileSync(new URL(`../model/objects/${name}.preimage.json`, import.meta.url), "utf8");
  const got = run({ op: "root-preimage", manifest: mf }).bytes;
  if (got !== want) { console.error(`guest: ${name} root preimage differs (${got.length} vs ${want.length} bytes)`); process.exit(1); }
  console.log(`guest: kappa object ${name} preimage ${got.length} bytes identical through core.wasm (${mf.shards.length} shards)`);
}
const manifest = JSON.parse(readFileSync(new URL("../model/objects/edge0-8b.manifest.json", import.meta.url), "utf8"));
const preimage = readFileSync(new URL("../model/objects/edge0-8b.preimage.json", import.meta.url), "utf8");
const pre = run({ op: "root-preimage", manifest }).bytes;
if (pre !== preimage) { console.error(`guest: root preimage differs (${pre.length} vs ${preimage.length} bytes)`); process.exit(1); }
for (let e = 0; e < 4; e++) { const r = run({ op: "expert-page", start: 100, length: 80, experts: 4, expert: e }); if (r.start !== 100 + 20 * e || r.end !== 120 + 20 * e) { console.error("guest: expert page differs"); process.exit(1); } }
const tp = [0, 1, 2, 3].map((i) => { const r = run({ op: "table-page", start: 7, end: 27, rowBytes: 2, rows: 3, index: i }); return [r.start, r.end]; });
if (JSON.stringify(tp) !== JSON.stringify([[7, 13], [13, 19], [19, 25], [25, 27]])) { console.error("guest: table page differs " + JSON.stringify(tp)); process.exit(1); }
let lines = 0, first = null;
for (const sh of manifest.shards) {
  const body = readFileSync(new URL("../model/objects/edge0-8b/" + sh.name.replace(/\//g, "_") + ".objects.jsonl", import.meta.url), "utf8");
  for (const line of body.split("\n")) { const o = JSON.parse(line); const got = run({ op: "object-line", kind: o[0], name: o[1], kappa: o[2], bytes: o[3] }).bytes; if (got !== line) { console.error("guest: object line differs\n " + line + "\n " + got); process.exit(1); } if (first === null) first = o[2]; lines++; }
}
if (!run({ op: "admit", listed: [first], kappa: first, derived: first }).admit || run({ op: "admit", listed: [first], kappa: first, derived: "blake3:0" }).admit) { console.error("guest: admit differs"); process.exit(1); }
console.log(`guest: kappa object preimage ${pre.length} bytes and ${lines} object lines identical through core.wasm; page rules and admit rows hold`);
console.log(`guest: ${vectors.length} wire vectors, ${checked} encodings byte identical through core.wasm; ${orChecked} OpenRouter request encodings and ${rows.length} route rows and ${readiness.length} readiness rows identical`);
