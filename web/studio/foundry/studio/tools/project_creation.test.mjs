import {browserChatMode} from "../shell/browser-chat-modes.mjs";
import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import vm from "node:vm";
import { prepareNewProject, loadProjectScaffold } from "../shell/project-creation.mjs";
import { kappa } from "../../try/vendor/prism-code/src/blake3.mjs";
const core = { run: ({ project, parent }) => ({ bytes: JSON.stringify({ entries: project.entries.map(e => [e.path, e.kappa, e.bytes]), name: project.label, parent }) }) };
const prepare = (files, extra = {}) => prepareNewProject({ name: "Example", files, initialChatMode: "build" }, { core, derive: kappa, ...extra });
const fail = (p, code) => assert.rejects(p, e => e.code === code && !e.message.includes("secret"));
test("preparation binds exact sorted files and generated snapshot", async () => {
  const p = await prepare({ "z.txt": "😀\r\n", "a.txt": "\uFEFFx" });
  assert.deepEqual(p.files.map(x => x[0]), ["a.txt", "z.txt"]);
  const snapshot = p.objects.find(([key]) => key === p.oid)[1];
  assert.equal(kappa(snapshot), p.oid); const doc = JSON.parse(new TextDecoder().decode(snapshot));
  assert.equal(doc.name, "Example"); assert.equal(doc.parent, "");
  for (const [path, text] of p.files) { const row = doc.entries.find(x => x[0] === path); const bytes = new TextEncoder().encode(text); assert.deepEqual(row, [path, kappa(bytes), bytes.length]); }
});
test("preparation snapshots caller input before asynchronous hashing", async () => {
  const files = { "a.txt": "original" }; let unblock; const gate = new Promise(r => { unblock = r; });
  const p = prepare(files, { derive: async b => { await gate; return kappa(b); } }); files["a.txt"] = "mutated"; unblock(); assert.equal((await p).files[0][1], "original");
});
test("invalid paths and text refuse before hashing", async () => {
  for (const files of [{ "../x": "x" }, { "/x": "x" }, { "x\\y": "x" }, { "x": 3 }, { "x": "\ud800" }]) await fail(prepare(files), "projectPreparationFailed");
});
test("bounds and broken identity or model refuse preparation", async () => {
  await fail(prepare(Object.fromEntries(Array.from({ length: 1001 }, (_, i) => [String(i), ""]))), "projectPreparationFailed");
  await fail(prepare({ x: "x".repeat(16 * 1024 * 1024 + 1) }), "projectPreparationFailed");
  await fail(prepare({ x: "x" }, { derive: () => "wrong" }), "projectPreparationFailed");
  await fail(prepare({ x: "x" }, { core: { run() { throw Error("secret"); } } }), "projectPreparationFailed");
});
test("scaffold loading requires every named file and rules", async () => {
  const seen = []; const fetchFn = async url => { seen.push(url.pathname); return new Response(url.pathname.endsWith("files.json") ? '["src/a.txt"]' : "complete"); };
  assert.deepEqual(await loadProjectScaffold(new URL("https://example.test/scaffold/"), { fetchFn }), { "src/a.txt": "complete", "AI_RULES.md": "complete" });
  assert.equal(seen.length, 3);
  await fail(loadProjectScaffold(new URL("https://example.test/scaffold/"), { fetchFn: async url => new Response(url.pathname.endsWith("files.json") ? '["a.txt"]' : "secret", { status: url.pathname.endsWith("files.json") ? 200 : 404 }) }), "projectScaffoldFailed");
});
test("malformed duplicate and escaping scaffold manifests refuse", async () => {
  await fail(loadProjectScaffold(new URL("https://example.test/scaffold/"), { fetchFn: async () => new Response("[]" + " ".repeat(65535)) }), "projectScaffoldFailed");
  for (const manifest of ['{}', '["../escape"]', '["a","a"]', '[3]', 'bad']) await fail(loadProjectScaffold(new URL("https://example.test/scaffold/"), { fetchFn: async () => new Response(manifest) }), "projectScaffoldFailed");
});
test("scaffold deadlines bound stalled bodies and invalid encoding", async () => {
  await fail(loadProjectScaffold(new URL("https://example.test/scaffold/"), { timeoutMs: 5, fetchFn: async () => new Promise(() => {}) }), "projectScaffoldFailed");
  await fail(loadProjectScaffold(new URL("https://example.test/scaffold/"), { fetchFn: async url => new Response(url.pathname.endsWith("files.json") ? '["a"]' : "x".repeat(16 * 1024 * 1024 + 1)) }), "projectScaffoldFailed");
  await fail(loadProjectScaffold(new URL("https://example.test/scaffold/"), { timeoutMs: 5, fetchFn: async () => new Response(new ReadableStream({ start() {} })) }), "projectScaffoldFailed");
  await fail(loadProjectScaffold(new URL("https://example.test/scaffold/"), { fetchFn: async () => new Response(new Uint8Array([255])) }), "projectScaffoldFailed");
});
test("actual host creation delegates atomically and import cannot overwrite IDs", async () => {
  const source = fs.readFileSync(new URL("../shell/host.js", import.meta.url), "utf8");
  const start = source.indexOf('handlers.set("create-app",'); const end = source.indexOf('handlers.set("read-app-file",', start);
  const handlers = new Map(), calls = [];
  vm.runInNewContext(source.slice(start, end), { handlers, browserChatMode, get: async()=>({defaultChatMode:"ask"}), coreReady: Promise.resolve(), core, kappa, loadScaffold: async () => ({ x: "y" }), dbReady: Promise.resolve({}), prepareNewProject: async input => { calls.push(input); return {}; }, createStoredProject: async () => ({ app: { id: 7 }, chatId: 8, oid: "test" }), nulls: {}, themesData: [], withDates: x => x, emit: (...a) => calls.push(a) });
  assert.deepEqual(JSON.parse(JSON.stringify(await handlers.get("create-app")({ name: "Example" }))), { app: { id: 7 }, chatId: 8 });
  assert.equal(calls[0].name, "Example"); assert.equal(calls[0].initialChatMode, "ask"); assert.ok(source.includes('store === "apps" || store === "chats"'));
  assert.ok(source.includes('.add(value, key)')); assert.ok(!source.includes('const rawCreate ='));
  assert.ok(source.includes('createStoredChat(await dbReady, appId'));
});
