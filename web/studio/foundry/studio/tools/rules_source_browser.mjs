// Exercise the shipped adapter against Chromium's real IndexedDB transactions.
import assert from "node:assert/strict";
import fs from "node:fs";
import http from "node:http";
import { chromium } from "../vendor/prism/node_modules/playwright-core/index.mjs";
const sources = new Map([
  ["/application-shape.mjs", fs.readFileSync(new URL("../shell/application-shape.mjs", import.meta.url))],
  ["/rules.mjs", fs.readFileSync(new URL("../shell/rules-source.mjs", import.meta.url))],
  ["/blake3.mjs", fs.readFileSync(new URL("../../try/vendor/prism-code/src/blake3.mjs", import.meta.url))],
]);
const server = http.createServer((request, response) => {
  if (request.url === "/") { response.setHeader("content-type", "text/html"); response.end("<!doctype html><title>Rules source binding acceptance</title>"); return; }
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
    const { readBoundRules, saveBoundRules } = await import("/rules.mjs");
    const { kappa } = await import("/blake3.mjs");
    const check = (condition, message) => { if (!condition) throw Error(message); };
    const db = await new Promise((resolve, reject) => {
      const request = indexedDB.open("rules-test", 1);
      request.onupgradeneeded = () => { request.result.createObjectStore("files"); request.result.createObjectStore("refs"); };
      request.onsuccess = () => resolve(request.result); request.onerror = () => reject(request.error);
    });
    const write = (store, key, value) => new Promise((resolve, reject) => {
      const t = db.transaction(store, "readwrite");
      if (value === undefined) t.objectStore(store).delete(key); else t.objectStore(store).put(value, key);
      t.oncomplete = resolve; t.onabort = () => reject(t.error);
    });
    const read = (store, key) => new Promise((resolve, reject) => {
      const request = db.transaction(store).objectStore(store).get(key);
      request.onsuccess = () => resolve(request.result); request.onerror = () => reject(request.error);
    });
    const source = '{"name":"current"}', changed = '{"name":"changed"}';
    const current = { phase: "proving", specKappa: kappa(new TextEncoder().encode(source)), spec: { name: "current" } };
    const next = { ...current, phase: "proved", proof: { id: "captured-proof" } };
    const derive = async bytes => kappa(bytes);
    const reset = async () => { await write("files", "1:prism/spec.json", source); await write("refs", "prism:1", current); };
    await reset();
    check((await readBoundRules(db, 1, derive)).phase === "proving", "valid current state not visible");
    await write("files", "1:prism/spec.json", changed);
    check(await readBoundRules(db, 1, derive) === null, "stale status visible");
    check(await saveBoundRules(db, 1, next, derive, current) === false, "stale source accepted");
    await reset();
    check(await saveBoundRules(db, 1, next, async bytes => { await write("files", "1:prism/spec.json", changed); return kappa(bytes); }, current) === false, "source change during hashing accepted");
    check((await read("refs", "prism:1")).phase === "proving", "source race wrote proof status");
    await reset();
    const competing = { ...current, phase: "checked" };
    check(await saveBoundRules(db, 1, next, async bytes => { await write("refs", "prism:1", competing); return kappa(bytes); }, current) === false, "competing completion overwritten");
    check((await read("refs", "prism:1")).phase === "checked", "prior-state race wrote proof status");
    await reset();
    check(await saveBoundRules(db, 1, next, async bytes => { await write("refs", "prism:1", undefined); return kappa(bytes); }, current) === false, "restore invalidation overwritten");
    check(await read("refs", "prism:1") === undefined, "invalidated status recreated");
    await reset();
    check(await saveBoundRules(db, 1, next, derive, current) === true, "valid transition refused");
    check((await readBoundRules(db, 1, derive)).proof.id === "captured-proof", "valid transition not saved");
    await reset();
    const callerOwned = structuredClone(next);
    check(await saveBoundRules(db, 1, callerOwned, async bytes => { callerOwned.proof.id = "changed-by-caller"; return kappa(bytes); }, current) === true, "frozen input refused");
    check((await read("refs", "prism:1")).proof.id === "captured-proof", "caller mutation changed pending state");
    try { await readBoundRules(db, 1, () => { throw Error("private"); }); throw Error("identity engine failure accepted"); }
    catch (error) { check(error.code === "rulesIdentityFailed" && !error.message.includes("private"), "identity error leaked or untyped"); }
    db.close();
    return { staleRead: "hidden", sourceRace: "refused", stateRace: "refused", restoreRace: "refused", currentTransition: "accepted", callerMutation: "isolated" };
  });
  assert.deepEqual(report, { staleRead: "hidden", sourceRace: "refused", stateRace: "refused", restoreRace: "refused", currentTransition: "accepted", callerMutation: "isolated" });
  console.log(JSON.stringify(report));
} finally {
  if (browser) await browser.close();
  await new Promise(resolve => server.close(resolve));
}
