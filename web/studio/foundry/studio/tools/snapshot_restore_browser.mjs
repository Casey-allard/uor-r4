// Exercise the shipped adapter against Chromium's real IndexedDB transactions.
import assert from "node:assert/strict";
import fs from "node:fs";
import http from "node:http";
import { chromium } from "../vendor/prism/node_modules/playwright-core/index.mjs";
const sources = new Map([
  ["/snapshot.mjs", fs.readFileSync(new URL("../shell/snapshot-restore.mjs", import.meta.url))],
  ["/blake3.mjs", fs.readFileSync(new URL("../../try/vendor/prism-code/src/blake3.mjs", import.meta.url))],
]);
const server = http.createServer((request, response) => {
  if (request.url === "/") { response.setHeader("content-type", "text/html"); response.end("<!doctype html><title>Snapshot transaction acceptance</title>"); return; }
  const source = sources.get(request.url);
  response.statusCode = source ? 200 : 404;
  response.setHeader("content-type", "text/javascript"); response.end(source ?? "");
});
await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
const executablePath = process.env.CHROME_BIN || ["/usr/bin/chromium", "/usr/bin/google-chrome"].find(path => fs.existsSync(path));
let browser;
try {
  browser = await chromium.launch({ executablePath, headless: true, args: ["--no-sandbox"] });
  const page = await browser.newPage();
  await page.goto(`http://127.0.0.1:${server.address().port}/`);
  const report = await page.evaluate(async () => {
    const { restoreSnapshot } = await import("/snapshot.mjs");
    const { kappa } = await import("/blake3.mjs");
    const check = (condition, message) => { if (!condition) throw Error(message); };
    const db = await new Promise((resolve, reject) => {
      const request = indexedDB.open("restore-test", 1);
      request.onupgradeneeded = () => { request.result.createObjectStore("files"); request.result.createObjectStore("refs"); };
      request.onsuccess = () => resolve(request.result); request.onerror = () => reject(request.error);
    });
    const seed = () => new Promise((resolve, reject) => {
      const t = db.transaction(["files", "refs"], "readwrite");
      t.objectStore("files").clear();
      t.objectStore("files").put("original", "1:old.txt");
      t.objectStore("files").put("other", "2:keep.txt");
      t.objectStore("refs").put("original-ref", "1:main");
      t.objectStore("refs").put({ phase: "proved" }, "prism:1");
      t.objectStore("refs").put({ phase: "proved" }, "prism:2");
      t.oncomplete = resolve; t.onabort = () => reject(t.error);
    });
    const read = (store, key) => new Promise((resolve, reject) => {
      const request = db.transaction(store).objectStore(store).get(key);
      request.onsuccess = () => resolve(request.result); request.onerror = () => reject(request.error);
    });
    const enc = new TextEncoder(), file = enc.encode("\uFEFFexact 😀\r\n"), fileId = kappa(file);
    const snapshot = enc.encode(JSON.stringify({ entries: [["src/new.txt", fileId, file.length]], name: "project", parent: "" }));
    const oid = kappa(snapshot), objects = new Map([[oid, snapshot], [fileId, file]]);
    const boundary = { readObject: async key => objects.get(key), derive: async bytes => kappa(bytes), admit: async (a, b) => a === b };
    const unchanged = async () => {
      check(await read("files", "1:old.txt") === "original", "original files lost");
      check(await read("files", "1:src/new.txt") === undefined, "partial insertion survived");
      check(await read("refs", "1:main") === "original-ref", "reference advanced on failure");
      check((await read("refs", "prism:1"))?.phase === "proved", "proof cache lost on failure");
      check(await read("files", "2:keep.txt") === "other", "other project changed");
    };
    await seed(); objects.delete(fileId);
    try { await restoreSnapshot(db, 1, oid, boundary); throw Error("accepted missing object"); }
    catch (error) { check(error.code === "missingFileObject", "wrong missing-object failure"); }
    await unchanged(); objects.set(fileId, file);
    // Fail after cursor deletions and file inserts have been queued. The real
    // IndexedDB transaction, not a mock database, must roll those writes back.
    const failingDb = { transaction(stores, mode) {
      const transaction = db.transaction(stores, mode);
      return new Proxy(transaction, { get(target, name) {
        if (name === "objectStore") return store => {
          const result = target.objectStore(store);
          if (store !== "refs") return result;
          return { put() { throw Error("injected reference write failure"); } };
        };
        const value = target[name]; return typeof value === "function" ? value.bind(target) : value;
      }, set(target, name, value) { target[name] = value; return true; } });
    } };
    try { await restoreSnapshot(failingDb, 1, oid, boundary); throw Error("accepted failed transaction"); }
    catch (error) { check(error.code === "snapshotStorageFailed", "wrong transaction failure"); }
    await unchanged();
    await restoreSnapshot(db, 1, oid, boundary);
    check(await read("files", "1:old.txt") === undefined, "old file retained");
    check(await read("files", "1:src/new.txt") === "\uFEFFexact 😀\r\n", "text bytes changed");
    check(await read("refs", "1:main") === oid, "reference not committed");
    check(await read("refs", "prism:1") === undefined, "stale proof cache survived restore");
    check((await read("refs", "prism:2"))?.phase === "proved", "other project proof changed");
    check(await read("files", "2:keep.txt") === "other", "other project changed");
    const empty = enc.encode(JSON.stringify({ entries: [], name: "empty", parent: oid }));
    const emptyId = kappa(empty); objects.set(emptyId, empty);
    await restoreSnapshot(db, 1, emptyId, boundary);
    check(await read("files", "1:src/new.txt") === undefined, "empty snapshot did not clear files");
    check(await read("refs", "1:main") === emptyId, "empty snapshot reference incorrect");
    db.close();
    return { missingBasis: "unchanged", transactionFailure: "rolledBack", successfulRestore: "exact", otherProject: "unchanged", emptyProject: "restored" };
  });
  assert.deepEqual(report, { missingBasis: "unchanged", transactionFailure: "rolledBack", successfulRestore: "exact", otherProject: "unchanged", emptyProject: "restored" });
  console.log(JSON.stringify(report));
} finally {
  if (browser) await browser.close();
  await new Promise(resolve => server.close(resolve));
}
