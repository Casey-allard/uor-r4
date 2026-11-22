import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { blake3 } from "../../try/vendor/prism-code/src/blake3.mjs";
import { prepareSnapshot, restoreSnapshot } from "../shell/snapshot-restore.mjs";
const enc = new TextEncoder();
const derive = async bytes => "blake3:" + Buffer.from(blake3(bytes)).toString("hex");
async function fixture(entries, bytes = enc.encode("hello")) {
  const key = await derive(bytes), state = { entries: entries ?? [["src/a.txt", key, bytes.length]], name: "app", parent: "" };
  const body = enc.encode(JSON.stringify(state)), oid = await derive(body), objects = new Map([[oid, body], [key, bytes]]);
  return { oid, key, body, objects, boundary: { readObject: async key => objects.get(key), derive, admit: async () => true } };
}
const refusal = (promise, code) => assert.rejects(promise, error => error.code === code && !error.message.includes("hello"));
test("reconstruction preserves exact Unicode and BOM bytes", async () => {
  for (const content of ["hello", "\uFEFFhello", "😀\r\n\u0000", ""]) {
    const f = await fixture(undefined, enc.encode(content));
    assert.deepEqual((await prepareSnapshot(f.oid, f.boundary)).files, [["src/a.txt", content]]);
  }
});
test("missing snapshot and file bases refuse before any write", async () => {
  const f = await fixture(); f.objects.delete(f.key);
  let touched = false;
  await refusal(restoreSnapshot({ transaction() { touched = true; } }, 1, f.oid, f.boundary), "missingFileObject");
  assert.equal(touched, false);
  f.objects.delete(f.oid);
  await refusal(prepareSnapshot(f.oid, f.boundary), "missingSnapshotObject");
});
test("snapshot identity, file identity and lengths are independently checked", async () => {
  let f = await fixture(); f.objects.set(f.oid, enc.encode("{}"));
  await refusal(prepareSnapshot(f.oid, f.boundary), "snapshotIdentityMismatch");
  f = await fixture(); f.objects.set(f.key, enc.encode("other"));
  await refusal(prepareSnapshot(f.oid, f.boundary), "fileIdentityMismatch");
  f = await fixture(); f.objects.set(f.key, enc.encode("shorter!"));
  await refusal(prepareSnapshot(f.oid, f.boundary), "fileIdentityMismatch");
});
test("malformed snapshot shape, duplicate and nonportable paths refuse", async () => {
  const base = await fixture();
  for (const path of ["", "/root", "../x", "a/../x", "a//b", "a\\b", "x:y", "a\u0000b"]) {
    const f = await fixture([[path, base.key, 5]]);
    await refusal(prepareSnapshot(f.oid, f.boundary), "invalidSnapshot");
  }
  for (const entries of [[["a", base.key, 5], ["a", base.key, 5]], [["a", base.key]], [["a", base.key, -1]], [["a", "prefix" + base.key, 5]], {}]) {
    const f = await fixture(entries); await refusal(prepareSnapshot(f.oid, f.boundary), "invalidSnapshot");
  }
  const f = await fixture(); f.boundary.readObject = async () => enc.encode("not json"); f.boundary.derive = async () => f.oid;
  await refusal(prepareSnapshot(f.oid, f.boundary), "invalidSnapshot");
});
test("fixed representation limits refuse before loading oversized dependencies", async () => {
  const base = await fixture();
  for (const entries of [Array.from({ length: 4097 }, (_, i) => ["f" + i, base.key, 5]), [["a", base.key, 16 * 1024 * 1024 + 1]], Array.from({ length: 5 }, (_, i) => ["f" + i, base.key, 16 * 1024 * 1024])]) {
    const f = await fixture(entries); await refusal(prepareSnapshot(f.oid, f.boundary), "snapshotLimitExceeded");
  }
  const f = await fixture(); f.objects.set(f.oid, new Uint8Array(2 * 1024 * 1024 + 1));
  await refusal(prepareSnapshot(f.oid, f.boundary), "snapshotLimitExceeded");
});
test("invalid UTF-8 refuses instead of replacing bytes", async () => {
  const f = await fixture(undefined, new Uint8Array([0xc0, 0xaf]));
  await refusal(prepareSnapshot(f.oid, f.boundary), "invalidFileEncoding");
});
test("an empty project reconstructs and core refusal remains authoritative", async () => {
  const f = await fixture([]); assert.deepEqual((await prepareSnapshot(f.oid, f.boundary)).files, []);
  f.boundary.admit = () => false;
  await refusal(prepareSnapshot(f.oid, f.boundary), "snapshotCoreRefused");
  f.boundary.admit = () => { throw Error("private"); };
  await refusal(prepareSnapshot(f.oid, f.boundary), "snapshotCoreRefused");
});
test("storage failure and invalid app identity expose typed refusals", async () => {
  const f = await fixture();
  await refusal(restoreSnapshot({}, 0, f.oid, f.boundary), "invalidSnapshot");
  await refusal(restoreSnapshot({ transaction() { throw Error("private"); } }, 1, f.oid, f.boundary), "snapshotStorageFailed");
  f.boundary.readObject = () => { throw Error("private"); };
  await refusal(prepareSnapshot(f.oid, f.boundary), "snapshotStorageFailed");
});
test("the real host delegates restoration to the checked atomic adapter", () => {
  const host = fs.readFileSync(new URL("../shell/host.js", import.meta.url), "utf8");
  const restore = host.slice(host.indexOf("async function restore(appId, oid, revert = false)"), host.indexOf('handlers.set("checkout-version"'));
  assert.match(restore, /return restoreStoredProject\(await dbReady, appId, oid/);
  assert.match(restore, /readObject: key => get\("objects", key\)/);
  assert.match(restore, /derive: kappa/);
  assert.match(restore, /core.run\(\{ op: "restore", derived, expected \}\)/);
  assert.doesNotMatch(restore, /(?:del|put)\("(?:files|refs)"/);
});
