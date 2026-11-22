import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import vm from "node:vm";
import { storageRequest } from "../shell/storage-request.mjs";
function fixture() {
  const q = { onsuccess: null, onerror: null, result: "value" };
  const t = { objectStore: () => ({ get: () => q }), aborts: 0, abort() { this.aborts++; this.onabort?.(); } };
  const db = { transaction: () => t };
  return { q, t, db, run: () => storageRequest(db, "store", "readonly", s => s.get(1)) };
}
const refuses = p => assert.rejects(p, e => e.code === "storageTransactionFailed" && e.message === "Storage transaction failed");
test("request success waits for transaction completion", async () => {
  const f = fixture(); let done = false; const p = f.run().then(v => { done = true; return v; });
  f.q.onsuccess(); await Promise.resolve(); assert.equal(done, false);
  f.t.oncomplete(); assert.equal(await p, "value");
});
test("abort after request success cannot become success", async () => {
  const f = fixture(); const p = f.run(); f.q.onsuccess(); f.t.onabort(); await refuses(p);
});
test("request error aborts and redacts details", async () => {
  const f = fixture(); const p = f.run(); f.q.error = Error("secret"); f.q.onerror(); await refuses(p); assert.equal(f.t.aborts, 1);
});
test("transaction error aborts even after request success", async () => {
  const f = fixture(); const p = f.run(); f.q.onsuccess(); f.t.onerror(); await refuses(p); assert.equal(f.t.aborts, 1);
});
test("synchronous operation failure aborts queued writes", async () => {
  const f = fixture(); await refuses(storageRequest(f.db, "store", "readwrite", () => { throw Error("secret"); })); assert.equal(f.t.aborts, 1);
});
test("opening and reading results fail closed", async () => {
  await refuses(storageRequest({ transaction() { throw Error("secret"); } }, "store", "readonly", () => {}));
  const f = fixture(); Object.defineProperty(f.q, "result", { get() { throw Error("secret"); } }); const p = f.run(); f.q.onsuccess(); await refuses(p); assert.equal(f.t.aborts, 1);
});
test("missing request and completion without result fail closed", async () => {
  const f = fixture(); await refuses(storageRequest(f.db, "store", "readwrite", () => undefined)); assert.equal(f.t.aborts, 1);
  const other = fixture(); const p = other.run(); other.t.oncomplete(); await refuses(p);
});
test("undefined deletion results remain successful after commit", async () => {
  const f = fixture(); f.q.result = undefined; const p = f.run(); f.q.onsuccess(); f.t.oncomplete(); assert.equal(await p, undefined);
});
test("the actual host transaction helper delegates without early success", async () => {
  const source = fs.readFileSync(new URL("../shell/host.js", import.meta.url), "utf8");
  const line = source.split("\n").find(x => x.startsWith("async function tx(")); assert.ok(line);
  const calls = []; const db = {}; const fn = () => {};
  const tx = vm.runInNewContext(line + "\ntx", { dbReady: Promise.resolve(db), storageRequest: (...args) => { calls.push(args); return "committed"; } });
  assert.equal(await tx("files", "readwrite", fn), "committed"); assert.deepEqual(calls, [[db, "files", "readwrite", fn]]);
});
test("the canonical boundary and parser inventory cover the adapter", () => {
  const spec = fs.readFileSync(new URL("../../app/src/FoundrySpecification.lex.tex", import.meta.url), "utf8");
  assert.ok(spec.includes("name: StorageRequestError,")); assert.ok(spec.includes("name: storageTransactionFailed,")); assert.ok(spec.includes("name: storageCommitBoundary,"));
  const parser = fs.readFileSync(new URL("./shell_parse.mjs", import.meta.url), "utf8"); assert.ok(parser.includes('"storage-request.mjs": "mjs"'));
});
