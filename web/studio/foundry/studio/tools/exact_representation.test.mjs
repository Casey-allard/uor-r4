import assert from "node:assert/strict";
import { test } from "node:test";
import { inflateSync, deflateSync } from "node:zlib";
import { randomBytes } from "node:crypto";
import fs from "node:fs";
import vm from "node:vm";
import { packBytes, unpackBytes as unpackArchive, byteIdentity, DECODER_SPEC, MAX_BYTES, RepresentationError } from "../shell/exact-representation.mjs";
const enc = new TextEncoder();
// Inner corruption controls supply a recomputed outer identity so they exercise
// their intended layer; the separate outer mutation retains the original identity.
const unpackBytes = async (bytes, basis = null, options = {}) => unpackArchive(bytes, await byteIdentity("representation", bytes), basis, options);
const serialized = value => enc.encode(JSON.stringify(value));
const parsed = value => JSON.parse(new TextDecoder().decode(value));
const refuses = (promise, code) => assert.rejects(promise, error => error instanceof RepresentationError && error.code === code);
async function replacePayload(manifest, bytes) { manifest.data = Buffer.from(bytes).toString("base64"); manifest.payload = await byteIdentity("payload", bytes); return serialized(manifest); }
test("self-contained binary representations reconstruct independently", async () => {
  for (const original of [new Uint8Array(), Uint8Array.from({ length: 256 }, (_, i) => i), new Uint8Array(50000).fill(37), new Uint8Array(randomBytes(10000))]) {
    const result = await packBytes(original); const manifest = parsed(result.bytes);
    assert.deepEqual(new Uint8Array(inflateSync(Buffer.from(manifest.data, "base64"))), original);
    assert.deepEqual(await unpackBytes(result.bytes), original);
    assert.deepEqual(result.identity, await byteIdentity("representation", result.bytes));
    assert.deepEqual(manifest.original, await byteIdentity("original", original));
    assert.equal(result.accounting.reconstructionBytes, result.bytes.length);
    assert.equal(result.accounting.mode, "self-contained");
  }
});
test("conditional basis reconstruction preserves distinct identities and accounting", async () => {
  const basis = Uint8Array.from({ length: 4096 }, (_, i) => i % 256); const original = basis.slice(); original[90] ^= 11;
  const result = await packBytes(original, basis); const manifest = parsed(result.bytes);
  const independent = new Uint8Array(inflateSync(Buffer.from(manifest.data, "base64"))); independent.forEach((value, i) => { independent[i] = value ^ basis[i]; });
  assert.deepEqual(independent, original); assert.deepEqual(await unpackBytes(result.bytes, basis), original);
  assert.equal(result.accounting.reconstructionBytes, result.bytes.length + basis.length);
  assert.equal(result.accounting.mode, "conditional-basis");
  assert.deepEqual(manifest.basis, await byteIdentity("basis", basis));
  await refuses(unpackBytes(result.bytes), "missingBasis");
  basis[0] ^= 1; await refuses(unpackBytes(result.bytes, basis), "basisMismatch");
  await refuses(packBytes(original, new Uint8Array(1)), "basisMismatch");
  await refuses(unpackBytes((await packBytes(original)).bytes, basis), "basisMismatch");
});
test("payload, original and decoder mutations fail separately", async () => {
  const packed = await packBytes(enc.encode("original bytes"));
  for (const [field, code] of [["original", "identityMismatch"], ["payload", "identityMismatch"], ["decoder", "unsupportedDecoder"]]) {
    const manifest = parsed(packed.bytes); manifest[field].digest = "0".repeat(64);
    await refuses(unpackBytes(serialized(manifest)), code);
  }
  const changed = parsed(packed.bytes);
  const alternate = await replacePayload(changed, new Uint8Array(deflateSync(enc.encode("original bytes"), { level: 0 })));
  assert.notDeepEqual(alternate, packed.bytes);
  await refuses(unpackArchive(alternate, packed.identity), "identityMismatch");
  await refuses(unpackArchive(packed.bytes, null), "invalidRepresentation");
  const manifest = parsed(packed.bytes); manifest.original.kind = "payload";
  await refuses(unpackBytes(serialized(manifest)), "invalidRepresentation");
});
test("malformed archives and noncanonical base64 are refused", async () => {
  for (const data of [enc.encode("{no"), new Uint8Array([255]), enc.encode("null"), enc.encode("{}")]) await refuses(unpackBytes(data), "invalidRepresentation");
  const result = await packBytes(new Uint8Array()); const manifest = parsed(result.bytes);
  manifest.data = "AB=="; await refuses(unpackBytes(serialized(manifest)), "invalidRepresentation");
  await refuses(unpackBytes(enc.encode(" " + new TextDecoder().decode(result.bytes))), "invalidRepresentation");
  const extra = parsed(result.bytes); extra.other = true; await refuses(unpackBytes(serialized(extra)), "invalidRepresentation");
});
test("bounded decompression rejects corrupt, oversized and truncated streams", async () => {
  const manifest = parsed((await packBytes(enc.encode("abc"))).bytes);
  await refuses(unpackBytes(await replacePayload(manifest, new Uint8Array([1, 2, 3]))), "decodeFailed");
  await refuses(unpackBytes(await replacePayload(manifest, new Uint8Array(deflateSync(new Uint8Array(1000000))))), "representationTooLarge");
  const payload = new Uint8Array(deflateSync(enc.encode("abc")));
  await refuses(unpackBytes(await replacePayload(manifest, payload.subarray(0, payload.length - 1))), "decodeFailed");
  await refuses(packBytes(new Uint8Array(MAX_BYTES + 1)), "representationTooLarge");
  await refuses(unpackBytes(new Uint8Array(4 * 1024 * 1024 + 1)), "representationTooLarge");
});
test("caller buffers are copied before asynchronous work", async () => {
  const original = enc.encode("immutable"); const expected = original.slice(); const pending = packBytes(original); original.fill(0);
  const packed = await pending; const next = unpackArchive(packed.bytes, packed.identity); packed.bytes.fill(0);
  assert.deepEqual(await next, expected);
  await refuses(packBytes({}), "invalidRepresentation");
  await refuses(packBytes(Object.create(Uint8Array.prototype)), "invalidRepresentation");
});
test("cancellation and deadlines are typed and stop native streams", async () => {
  const controller = new AbortController(); controller.abort(); await refuses(packBytes(new Uint8Array(20), null, { signal: controller.signal }), "representationCancelled");
  const stalled = globalThis.CompressionStream;
  try {
    globalThis.CompressionStream = class { constructor() { return new TransformStream({ async transform() { await new Promise(() => {}); } }); } };
    await refuses(packBytes(new Uint8Array(20), null, { deadlineMs: 5 }), "representationTimedOut");
  } finally { globalThis.CompressionStream = stalled; }
  await refuses(packBytes(new Uint8Array(20), null, { deadlineMs: 0 }), "invalidRepresentation");
  await refuses(packBytes(new Uint8Array(20), null, { signal: {} }), "invalidRepresentation");
  await refuses(packBytes(new Uint8Array(20), null, { get deadlineMs() { throw Error("private"); } }), "invalidRepresentation");
});
test("decoder descriptor and typed errors remain bound to the LexLean specification", () => {
  assert.match(DECODER_SPEC, /RFC1950/);
  const source = fs.readFileSync(new URL("../../app/src/FoundrySpecification.lex.tex", import.meta.url), "utf8");
  assert.ok(source.includes("name: representationBoundary,"));
  assert.ok(source.includes(`body: ${JSON.stringify(DECODER_SPEC)}`), "decoder specification drift");
  for (const code of ["invalidRepresentation", "identityMismatch", "missingBasis", "basisMismatch", "unsupportedDecoder", "representationTooLarge", "decodeFailed", "representationCancelled", "representationTimedOut"]) assert.ok(source.includes(`name: ${code}, fields: []`));
});
test("actual host commands delegate byte operations and preserve typed refusals", async () => {
  const source = fs.readFileSync(new URL("../shell/host.js", import.meta.url), "utf8");
  const start = source.indexOf("async function representationCommand"); const end = source.indexOf('handlers.set("forge:compose"', start);
  assert.ok(start > 0 && end > start); const handlers = new Map();
  vm.runInNewContext(source.slice(start, end), { handlers, packBytes, unpackBytes: unpackArchive, RepresentationError });
  const packed = await handlers.get("forge:pack-bytes")({ bytes: enc.encode("host"), basis: null }); assert.equal(packed.ok, true);
  const restored = await handlers.get("forge:unpack-bytes")({ bytes: packed.value.bytes, identity: packed.value.identity, basis: null }); assert.equal(new TextDecoder().decode(restored.value), "host");
  assert.equal((await handlers.get("forge:unpack-bytes")({ bytes: enc.encode("bad"), identity: null, basis: null })).code, "invalidRepresentation");
  assert.equal((await handlers.get("forge:pack-bytes")({ bytes: enc.encode("bad") })).code, "invalidRepresentation");
});
