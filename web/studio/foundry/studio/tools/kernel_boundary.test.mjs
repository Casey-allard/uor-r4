import assert from "node:assert/strict";
import fs from "node:fs";
import vm from "node:vm";
import { createHash, webcrypto } from "node:crypto";
import { test } from "node:test";
import { executeWorkWithGuest } from "../shell/work-codec.mjs";
import { admitWithGuest } from "../shell/composition-codec.mjs";
const source = fs.readFileSync(new URL("../shell/kernel.js", import.meta.url), "utf8");
const originalMeta = JSON.parse(fs.readFileSync(new URL("../shell/kernel/kernel.json", import.meta.url)));
const originalBytes = fs.readFileSync(new URL("../shell/kernel/guest.wasm", import.meta.url));
const self = {};
vm.runInNewContext(source, { self, crypto: webcrypto, WebAssembly, TextEncoder, TextDecoder, Uint8Array, atob });
const sha = bytes => createHash("sha256").update(bytes).digest("hex");
async function load({ meta = structuredClone(originalMeta), bytes = originalBytes, get } = {}) {
  return self.ForgeKernel.load(get ?? (async path => path === "kernel/kernel.json" ? new Response(JSON.stringify(meta)) : new Response(bytes)));
}
const code = expected => error => error.code === expected;
test("the actual pinned guest replays its corpus and preserves phase behavior", async () => {
  const guest = await load(); assert.ok(guest.vectors > 0);
  assert.equal(guest.digest, originalMeta.digest);
  assert.equal(await guest.ask("drafting check"), "checked");
  assert.equal(await guest.ask("proved publish"), "published");
});
test("metadata refuses vacuity, incoherent bounds, wrong paths and malformed encodings", async () => {
  for (const change of [m => m.vectors = [], m => m.limits.request = m.limits.guestAllocation + 1, m => m.limits.response = 0, m => m.guest.path = "https://example.test/guest", m => m.guest.bytes = 16 * 1024 * 1024 + 1, m => m.digest = [m.digest], m => m.vectors[0] = ["bad base64", ""], m => m.vectors[0] = ["", "", ""]]) {
    const meta = structuredClone(originalMeta); change(meta);
    await assert.rejects(load({ meta }), code("invalidKernelMetadata"));
  }
});
test("missing resources, corrupt bytes and incorrect byte length refuse", async () => {
  await assert.rejects(load({ get: async () => new Response("missing", { status: 404 }) }), code("kernelLoadFailed"));
  const bytes = Uint8Array.from(originalBytes); bytes[20] ^= 1;
  await assert.rejects(load({ bytes }), code("kernelIntegrityMismatch"));
  const meta = structuredClone(originalMeta); meta.guest.bytes++;
  await assert.rejects(load({ meta }), code("kernelIntegrityMismatch"));
});
test("a correctly hashed module still needs the pure core ABI and matching vectors", async () => {
  const bytes = new Uint8Array([0, 97, 115, 109, 1, 0, 0, 0]);
  let meta = structuredClone(originalMeta); meta.guest.bytes = bytes.length; meta.guest.sha256 = sha(bytes);
  await assert.rejects(load({ meta, bytes }), code("kernelExportsMismatch"));
  meta = structuredClone(originalMeta); meta.vectors[0][1] = Buffer.from("forged").toString("base64");
  await assert.rejects(load({ meta }), code("kernelAcceptanceFailed"));
});
test("public requests are typed, bounded and isolated from caller mutation", async () => {
  const guest = await load();
  await assert.rejects(guest.dispatch("drafting check"), code("invalidKernelRequest"));
  await assert.rejects(guest.dispatch(Object.create(Uint8Array.prototype)), code("invalidKernelRequest"));
  await assert.rejects(guest.dispatch(new Proxy(new Uint8Array(), {})), code("invalidKernelRequest"));
  await assert.rejects(guest.ask({ toString: () => "drafting check" }), code("invalidKernelRequest"));
  await assert.rejects(guest.dispatch(new Uint8Array(originalMeta.limits.request + 1)), code("kernelRequestTooLarge"));
  const bytes = new TextEncoder().encode("drafting check");
  bytes.slice = () => { throw Error("caller-owned method must not run"); };
  const pending = guest.dispatch(bytes); bytes.fill(0);
  assert.equal(new TextDecoder().decode(await pending), "checked");
});
test("execution and output limits remain enforced after successful loading", async () => {
  const meta = structuredClone(originalMeta);
  meta.vectors = [[Buffer.from("drafting check").toString("base64"), Buffer.from("checked").toString("base64")]];
  meta.limits.response = 7;
  const short = await load({ meta });
  await assert.rejects(short.ask("proved publish"), code("kernelResponseTooLarge"));
  meta.limits.request = meta.limits.guestAllocation = 1024 * 1024;
  const oversized = await load({ meta });
  await assert.rejects(oversized.dispatch(new Uint8Array(1024 * 1024)), code("kernelExecutionFailed"));
});
test("the actual browser host delegates every structured composition case to the guest", async () => {
  const guest = await load(), handlers = new Map();
  const host = fs.readFileSync(new URL("../shell/host.js", import.meta.url), "utf8");
  const start = host.indexOf('handlers.set("forge:compose"'), end = host.indexOf("const PHASES", start);
  vm.runInNewContext(host.slice(start, end), { handlers, kernelReady: Promise.resolve(guest), admitWithGuest, executeWorkWithGuest });
  const handler = handlers.get("forge:compose"); assert.equal(typeof handler, "function");
  for (const input of [undefined, null, {}, { policy: {}, features: [], extra: true }]) assert.equal((await handler(input)).code, "malformedComposition");
  const vectors = JSON.parse(fs.readFileSync(new URL("../../verification/composition-wire-vectors.json", import.meta.url)));
  assert.ok(vectors.filter(v => v.policy).length >= 15);
  for (const vector of vectors.filter(v => v.policy)) {
    const result = await handler({ policy: vector.policy, features: vector.features });
    assert.equal(result.ok, vector.response === "accepted", vector.name);
    if (!result.ok) assert.equal(result.code, vector.response, vector.name);
  }
});

test("the actual guest preserves work through suspension, transfer and completion", async () => {
  const guest = await load();
  const vectors = JSON.parse(fs.readFileSync(new URL("../../verification/work-wire-vectors.json", import.meta.url))).vectors;
  assert.equal(vectors.length, 30);
  for (const vector of vectors) assert.equal(await guest.ask(vector.request), vector.response, vector.name);
  const host = fs.readFileSync(new URL("../shell/host.js", import.meta.url), "utf8");
  const line = host.split("\n").find(value => value.startsWith('handlers.set("forge:work",'));
  assert.ok(line); const handlers = new Map();
  vm.runInNewContext(line, { handlers, kernelReady: Promise.resolve(guest), executeWorkWithGuest });
  const run = handlers.get("forge:work");
  let result = await run({ kind: "initialize", owner: 7, capacity: 20, remaining: 2 });
  assert.equal(result.ok, true);
  for (const action of [{ kind: "reserve", amount: 10 }, { kind: "start" }, { kind: "progress", cost: 3, units: 1, checkpoint: 7 }, { kind: "pause" }, { kind: "transfer", owner: 9 }, { kind: "resume" }, { kind: "progress", cost: 7, units: 1, checkpoint: 8 }, { kind: "finish" }]) {
    const state = result.state;
    result = await run({ kind: "transition", state, owner: state.owner, revision: state.revision, action });
    assert.equal(result.ok, true, action.kind);
  }
  assert.deepEqual(result.state, { owner: 9, revision: 8, capacity: 20, spent: 10, reserved: 0, checkpoint: 8, remaining: 0, phase: 3 });
});

// The composition boundary vectors: an accepted request of 30,008 bytes and a malformed one at the 32,776-byte
// request maximum. They are the kernel's own (compositionBoundaryVectors in
// app/src/ForgeKernel.lex.tex, built from CompositionFixtures), kept out of the application's acceptance
// list because every acceptance vector ships inside app.holo, which the Prism page verifies on its main
// thread: these two were 190 KB of a 385 KB archive. The request one past the maximum stays in the shipped
// list: PrismPM requires an exact allocation-boundary vector there (PP2009). Here they are read from the model itself, each request
// assembled as the model writes it, and replayed through the pinned guest.
test("the kernel's composition boundary vectors replay through the pinned guest", async () => {
  const kernel = fs.readFileSync(new URL("../../app/src/ForgeKernel.lex.tex", import.meta.url), "utf8");
  const fixtures = fs.readFileSync(new URL("../../app/src/CompositionFixtures.lex.tex", import.meta.url), "utf8");
  const wires = new Map([...fixtures.matchAll(/name: (wire[0-9a-f]{16}),\s*parameters: \[\],\s*result: Bytes,\s*body: 0x([0-9a-f]*)\s*\}/g)].map(m => [m[1], Buffer.from(m[2], "hex")]));
  const at = kernel.indexOf("  name: compositionBoundaryVectors,");
  assert.ok(at > 0, "compositionBoundaryVectors is declared");
  const body = kernel.slice(at, kernel.indexOf("\n}\n", at));
  // The expression forms the model uses for a request: a hex literal, a fixture, or two of them appended.
  function evaluate(text) {
    let i = 0;
    const space = () => { while (/\s/.test(text[i])) i++; };
    const expect = (word) => { space(); assert.equal(text.slice(i, i + word.length), word, `expected ${word} at ${i}`); i += word.length; };
    function value() {
      space();
      if (text.startsWith("0x", i)) { const m = /^0x([0-9a-f]*)/.exec(text.slice(i)); i += m[0].length; return Buffer.from(m[1], "hex"); }
      expect("(");
      space();
      if (text.startsWith("CompositionFixtures::", i)) {
        const m = /^CompositionFixtures::(wire[0-9a-f]{16})\)/.exec(text.slice(i)); assert.ok(m && wires.has(m[1]), "a known fixture"); i += m[0].length; return wires.get(m[1]);
      }
      expect("prim append"); const a = value(); const b = value(); expect(": Bytes"); expect(")");
      return Buffer.concat([a, b]);
    }
    const out = value(); space(); assert.equal(i, text.length, "nothing follows the request"); return out;
  }
  const vectors = [];
  const marker = "(record Foundation.View.Text.V1.Model::AcceptanceVector {request := ";
  for (let from = body.indexOf(marker); from >= 0; from = body.indexOf(marker, from + 1)) {
    const start = from + marker.length, end = body.indexOf(", response := 0x", start);
    const response = /^, response := 0x([0-9a-f]*)\}/.exec(body.slice(end));
    vectors.push([evaluate(body.slice(start, end)), Buffer.from(response[1], "hex").toString()]);
  }
  assert.deepEqual(vectors.map(([r, a]) => [r.length, a]), [[30008, "accepted"], [32776, "malformedComposition"]]);
  // The guest's own answer, through its core-wasm ABI, as kernel.js replays a vector; the page's kernel.js
  // refuses the oversized request before the guest is asked, and that refusal is checked too.
  const module = await WebAssembly.compile(originalBytes);
  const run = async (request) => {
    const { exports } = await WebAssembly.instantiate(module, {});
    const ptr = exports.holo_alloc(request.length);
    new Uint8Array(exports.memory.buffer, ptr, request.length).set(request);
    const packed = BigInt.asUintN(64, BigInt(exports.holo_run(ptr, request.length)));
    return Buffer.from(new Uint8Array(exports.memory.buffer, Number(packed >> 32n), Number(packed & 0xffffffffn))).toString();
  };
  for (const [request, response] of vectors) assert.equal(await run(request), response, `${request.length} bytes`);
  const guest = await load();
  assert.equal(await guest.ask(vectors[0][0].toString("latin1")), "accepted");
  await assert.rejects(guest.ask(vectors[1][0].toString("latin1") + "x".repeat(8)), code("kernelRequestTooLarge"));
  // The compact composition list excludes these two large cases, while the allocation boundary
  // and the independent interaction oversize refusal remain embedded.
  const large = originalMeta.vectors.map(([request,response]) => [Buffer.from(request, "base64"),Buffer.from(response,"base64").toString()]).filter(([request]) => request.length > 8192).map(([request,response])=>[request.length,response]).sort((a,b)=>a[0]-b[0]);
  assert.deepEqual(large, [[8202,"interactionTooLarge"],[originalMeta.limits.guestAllocation,"compositionTooLarge"]], "only explicit allocation and interaction oversize refusals remain large");
});
