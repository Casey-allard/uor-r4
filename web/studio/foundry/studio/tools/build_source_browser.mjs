// Real IndexedDB source verification with independent preimage/identity fixtures.
import assert from "node:assert/strict";
import fs from "node:fs";
import http from "node:http";
import { chromium } from "../vendor/prism/node_modules/playwright-core/index.mjs";
const sources = new Map([
  ["/browser-chat-modes.mjs", fs.readFileSync(new URL("../shell/browser-chat-modes.mjs", import.meta.url))],
  ["/chat-mode-catalog.mjs", fs.readFileSync(new URL("../shell/chat-mode-catalog.mjs", import.meta.url))],
  ["/chat-contracts.mjs", fs.readFileSync(new URL("../shell/chat-contracts.mjs", import.meta.url))],

  ["/theme-library.mjs", fs.readFileSync(new URL("../shell/theme-library.mjs", import.meta.url))],
  ["/version-preview-codec.mjs", fs.readFileSync(new URL("../shell/version-preview-codec.mjs", import.meta.url))],["/blake3.mjs", fs.readFileSync(new URL("../../try/vendor/prism-code/src/blake3.mjs", import.meta.url))]]);
for (const name of ["build-source", "project-lifecycle", "project-edit", "project-creation", "generation-patch", "snapshot-restore", "storage-request"]) sources.set(`/${name}.mjs`, fs.readFileSync(new URL(`../shell/${name}.mjs`, import.meta.url)));
for (const name of ["application-shape.mjs", "approval-queue.mjs", "live/app.mjs", "live/notation.mjs", "live/v2/interp.mjs", "live/v2/render.mjs", "canonical-build.mjs", "prism.js", "prism/spec.mjs", "prism/spec-file.mjs", "prism/render.mjs", "in-tab-verdict-codec.mjs", "prism/builder/route.mjs", "prism/builder/prompt.mjs", "prism/template.mjs", "prism/sha256.mjs", "live/lexlean.mjs", "live/obligations.mjs", "live/table.mjs", "prism-code/src/blake3.mjs"]) sources.set("/" + name, fs.readFileSync(new URL("../shell/" + name, import.meta.url)));
const server = http.createServer((req, res) => {
  const bytes = sources.get(req.url);
  if (!bytes && req.url !== "/") { res.writeHead(404, {"content-type":"text/plain"}); res.end("Fixture resource is not declared"); return; }
  res.setHeader("content-type", bytes ? "text/javascript" : "text/html"); res.end(bytes || "<!doctype html><title>Build source acceptance</title>");
});
await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
let browser;
try {
  browser = await chromium.launch({ executablePath: process.env.CHROME_BIN || ["/usr/bin/chromium", "/usr/bin/google-chrome"].find(fs.existsSync), headless: true, args: ["--no-sandbox"] });
  const page = await browser.newPage(); await page.goto(`http://127.0.0.1:${server.address().port}/`);
  const report = await page.evaluate(async () => {
    const { captureBuildSource, assertBuildSourceCurrent } = await import("/build-source.mjs");
    const { prepareNewProject, createStoredProject } = await import("/project-creation.mjs");
    const { editStoredProject } = await import("/project-edit.mjs"); const { deleteStoredProject } = await import("/project-lifecycle.mjs");
    const { storageRequest } = await import("/storage-request.mjs"); const { kappa } = await import("/blake3.mjs");
    const check = (ok, why) => { if (!ok) throw Error(why); };
    const stores = ["apps", "chats", "files", "objects", "refs", "versions"];
    const db = await new Promise((resolve, reject) => { const q = indexedDB.open("build-source", 1); q.onupgradeneeded = () => stores.forEach(s => q.result.createObjectStore(s)); q.onsuccess = () => resolve(q.result); q.onerror = () => reject(q.error); });
    const get = (s, k) => storageRequest(db, s, "readonly", o => o.get(k)), put = (s, k, v) => storageRequest(db, s, "readwrite", o => o.put(v, k));
    const core = { run: ({ project, parent }) => ({ bytes: JSON.stringify({ entries: project.entries.map(e => [e.path, e.kappa, e.bytes]), name: project.label, parent }) }) };
    const boundary = { core, derive: kappa, readObject: k => get("objects", k), admit: async (a, b) => a === b };
    const prepared = await prepareNewProject({ name: "Source", files: { "a.txt": "original" } }, boundary);
    const project = await createStoredProject(db, prepared), id = project.app.id;
    const expect = async (operation, code) => { let actual; try { await operation(); } catch (e) { actual = e.code; } check(actual === code, `${code} expected, got ${actual}`); };
    const captured = await captureBuildSource(db, id, boundary); await assertBuildSourceCurrent(db, id, captured);
    check(captured.reference === prepared.oid && new Map(captured.files).get("a.txt") === "original", "capture changed exact source");
    await put("files", `${id}:a.txt`, "unsealed");
    await expect(() => captureBuildSource(db, id, boundary), "buildSourceMismatch"); await expect(() => assertBuildSourceCurrent(db, id, captured), "buildSourceChanged");
    await put("files", `${id}:a.txt`, "original");
    const fileKey = kappa(new TextEncoder().encode("original")); await put("objects", fileKey, new TextEncoder().encode("tampered"));
    await expect(() => captureBuildSource(db, id, boundary), "fileIdentityMismatch"); await put("objects", fileKey, new TextEncoder().encode("original"));
    await put("apps", id, { ...(await get("apps", id)), extraDeps: ["react@19.1.0"] });
    await expect(() => captureBuildSource(db, id, boundary), "buildSourceMismatch"); await expect(() => assertBuildSourceCurrent(db, id, captured), "buildSourceChanged");
    await put("apps", id, { ...(await get("apps", id)), extraDeps: [] });
    const oid = await editStoredProject(db, { appId: id, filePath: "a.txt", content: "new" }, boundary);
    await expect(() => assertBuildSourceCurrent(db, id, captured), "buildSourceChanged");
    const updated = await captureBuildSource(db, id, boundary); check(updated.reference === oid && new Map(updated.files).get("a.txt") === "new", "new capture not exact");
    await expect(() => captureBuildSource(db, id, { ...boundary, admit: async () => false }), "snapshotCoreRefused");
    await deleteStoredProject(db, id); await expect(() => assertBuildSourceCurrent(db, id, updated), "buildSourceChanged");
    db.close(); return { exactCapture: true, unsealedSource: "refused", changedSource: "refused", tamperedBasis: "refused", dependencies: "bound", coreRefusal: "preserved", deletion: "refused" };
  });
  assert.deepEqual(report, { exactCapture: true, unsealedSource: "refused", changedSource: "refused", tamperedBasis: "refused", dependencies: "bound", coreRefusal: "preserved", deletion: "refused" }); console.log(JSON.stringify(report));
} finally { await browser?.close(); await new Promise(r => server.close(r)); }
