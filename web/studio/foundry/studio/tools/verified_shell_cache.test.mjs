import assert from "node:assert/strict";
import { test } from "node:test";
import { createHash } from "node:crypto";
import fs from "node:fs";
import vm from "node:vm";
import os from "node:os";
import path from "node:path";
import { checkStagedShell } from "./check_shell_closure.mjs";
await import("../shell/shell-cache.js");
const { prepare } = globalThis.ShellClosure;
const sha = bytes => createHash("sha256").update(bytes).digest("hex");
const base = "https://example.test/foundry/";
const fixture = (extra = {}) => {
  const files = { "index.html": "<h1>Foundry</h1>", "host.js": "// host", "core.wasm": "wasm", "assets/index.js": "// renderer", "assets/index.css": "body{}", "scaffold/files.json": "[]", "editor/vs/loader.js": "// editor", "editor/package.json": "{}", "compiler/package.json": "{}", "compiler/esm/browser.min.js": "// compiler", "compiler/esbuild.wasm": "fixture", ...extra };
  const rows = Object.entries(files).sort(([a], [b]) => a < b ? -1 : 1).map(([path, body]) => ({ bytes: Buffer.byteLength(body), path, sha256: sha(body) }));
  const closure = sha(JSON.stringify(rows));
  const manifest = { spec: "hologram-forge/shell/1", closure, files: rows };
  return { files, manifest, closure, fetchFn: async (url, options) => {
    assert.equal(options.method, "GET"); assert.equal(options.redirect, "error"); assert.equal(options.credentials, "omit"); assert.equal(options.cache, "no-store"); assert.equal(url.origin, new URL(base).origin);
    const path = decodeURIComponent(url.pathname.slice("/foundry/".length));
    return new Response(path === "manifest.json" ? JSON.stringify(manifest) : path === "provenance.json" ? JSON.stringify({ browser_projection_sha256: closure }) : files[path], { headers: { "content-type": "text/plain" } });
  } };
};
const refuse = (promise, code) => assert.rejects(promise, error => error.code === code && error.message === `Shell installation refused: ${code}`);
test("complete manifest and independent per-file bytes precede response publication", async () => {
  const f = fixture({ "fonts/a b.woff2": "\uFEFF😀\r\n" });
  const entries = await prepare(base, f.closure, f); assert.equal(entries.length, Object.keys(f.files).length + 2);
  for (const [url, response] of entries.slice(0, -2)) { const path = decodeURIComponent(new URL(url).pathname.slice(9)); assert.deepEqual(Buffer.from(await response.arrayBuffer()), Buffer.from(f.files[path])); }
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "shell-inventory-"));
  try {
    for (const [name, body] of Object.entries(f.files)) { fs.mkdirSync(path.dirname(path.join(dir, name)), { recursive: true }); fs.writeFileSync(path.join(dir, name), body); }
    fs.writeFileSync(path.join(dir, "manifest.json"), JSON.stringify(f.manifest)); fs.writeFileSync(path.join(dir, "provenance.json"), JSON.stringify({ browser_projection_sha256: f.closure }));
    assert.equal((await checkStagedShell(dir)).files, Object.keys(f.files).length);
    fs.writeFileSync(path.join(dir, "assets", "undeclared.js"), "secret"); await assert.rejects(checkStagedShell(dir), /complete staged inventory/);
  } finally { fs.rmSync(dir, { recursive: true, force: true }); }

});
test("changed closure file bytes and provenance refuse installation", async () => {
  let f = fixture(); f.manifest.files[0].sha256 = "0".repeat(64); await refuse(prepare(base, f.closure, f), "shellClosureMismatch");
  f = fixture(); f.files["host.js"] = "// evil"; await refuse(prepare(base, f.closure, f), "shellFileMismatch");
  f = fixture(); const fetchFn = f.fetchFn; await refuse(prepare(base, f.closure, { fetchFn: (url, options) => url.pathname.endsWith("provenance.json") ? new Response('{"browser_projection_sha256":"wrong"}') : fetchFn(url, options) }), "shellClosureMismatch");
  f = fixture(); f.files["host.js"] = "x"; await refuse(prepare(base, f.closure, f), "shellFileMismatch");
});
test("strict schema portable unique ordered paths required assets and byte limits", async () => {
  for (const path of ["../escape", "/root", "a?query", "a#hash", "a%2fb", "a\\b", "warmup.json"]) {
    const f = fixture(); f.manifest.files[0].path = path; await refuse(prepare(base, f.closure, f), "invalidShellManifest");
  }
  let f = fixture(); f.manifest.files.reverse(); await refuse(prepare(base, f.closure, f), "invalidShellManifest");
  f = fixture(); f.manifest.files[1].path = f.manifest.files[0].path; await refuse(prepare(base, f.closure, f), "invalidShellManifest");
  f = fixture(); f.manifest.files = f.manifest.files.filter(x => x.path !== "assets/index.js"); await refuse(prepare(base, f.closure, f), "invalidShellManifest");
  f = fixture(); f.manifest.extra = true; await refuse(prepare(base, f.closure, f), "invalidShellManifest");
  f = fixture(); f.manifest.files[0].bytes = 32 * 1024 * 1024 + 1; await refuse(prepare(base, f.closure, f), "shellClosureLimitExceeded");
  f = fixture(); f.manifest.files.forEach(row => { row.bytes = 32 * 1024 * 1024; }); await refuse(prepare(base, f.closure, f), "shellClosureLimitExceeded");
  f = fixture(); f.manifest.files = Array(8193).fill(f.manifest.files[0]); await refuse(prepare(base, f.closure, f), "shellClosureLimitExceeded");
  await refuse(prepare(base, "0".repeat(64), { fetchFn: () => new Response(" ".repeat(2 * 1024 * 1024 + 1)) }), "shellClosureLimitExceeded");
});
test("response failures and stalled headers bodies identities cancellation are typed", async () => {
  const f = fixture();
  await refuse(prepare(base, f.closure, { fetchFn: () => new Response("secret", { status: 404 }) }), "shellResponseInvalid");
  await refuse(prepare(base, f.closure, { fetchFn: () => { throw Error("secret"); } }), "shellClosureUnavailable");
  await refuse(prepare(base, f.closure, { fetchFn: () => new Response("bad") }), "invalidShellManifest");
  await refuse(prepare(base, f.closure, { fetchFn: () => new Promise(() => {}), timeoutMs: 5 }), "shellClosureTimedOut");
  await refuse(prepare(base, f.closure, { fetchFn: () => new Response(new ReadableStream({ start() {} })), timeoutMs: 5 }), "shellClosureTimedOut");
  await refuse(prepare(base, f.closure, { ...f, digest: () => new Promise(() => {}), timeoutMs: 5 }), "shellClosureTimedOut");
  await refuse(prepare(base, f.closure, { ...f, digest: () => new ArrayBuffer(0) }), "shellClosureUnavailable");
  const c = new AbortController(); c.abort(); await refuse(prepare(base, f.closure, { ...f, signal: c.signal }), "shellClosureCancelled");
  const active = new AbortController(), pending = prepare(base, f.closure, { fetchFn: () => new Promise(() => {}), signal: active.signal }); active.abort(); await refuse(pending, "shellClosureCancelled");
});
test("parallel file retrieval stays within eight requests and rejects declared length overruns", async () => {
  const f = fixture(Object.fromEntries(Array.from({ length: 20 }, (_, i) => [`extra/${i}.js`, "x"]))); let active = 0, maximum = 0;
  const fetchFn = async (url, options) => { active++; maximum = Math.max(maximum, active); await new Promise(r => setTimeout(r, 1)); active--; return f.fetchFn(url, options); };
  await prepare(base, f.closure, { fetchFn }); assert.equal(maximum, 8);
  f.files["host.js"] += "too much"; await refuse(prepare(base, f.closure, f), "shellClosureLimitExceeded");
});
test("actual worker verifies before caching and activation and projector includes renderer closure", async () => {
  const source = fs.readFileSync(new URL("../shell/sw.template.js", import.meta.url), "utf8");
  const start = source.indexOf('self.addEventListener("install"'), end = source.indexOf('self.addEventListener("activate"', start);
  const events = [], calls = [];
  const run = async (prepare, failPut = false) => { let completion; vm.runInNewContext(source.slice(start, end), {
    CLOSURE: "pin", ShellClosure: { prepare }, self: { registration: { scope: base }, addEventListener: (_, fn) => fn({ waitUntil: p => { completion = p; } }), skipWaiting: () => { events.push("activate"); } },
    caches: { open: async name => { calls.push(name); return { put: async (...args) => { events.push("put"); if (failPut) throw Error("private quota detail"); } }; } },
  }); return completion; };
  await assert.rejects(run(async () => { throw Error("refused"); })); assert.equal(calls.length, 0); assert.deepEqual(events, []);
  await run(async () => { events.push("verify"); return [[base, new Response("verified")]]; }); assert.deepEqual(events, ["verify", "put", "activate"]);
  events.length = 0; await assert.rejects(run(async () => [[base, new Response("verified")]], true), error => error.code === "shellClosureUnavailable" && !error.message.includes("private")); assert.deepEqual(events, ["put"]);
  assert.match(source, /importScripts\("kernel.js", "shell-cache.js"\)/); assert.match(source, /path.startsWith\("scaffold\/"\)/);
  assert.doesNotMatch(source, /cache\.addAll/);
  const projector = fs.readFileSync(new URL("../core/src/bin/project_shell.rs", import.meta.url), "utf8");
  assert.doesNotMatch(projector, /name == "assets"|name == "scaffold"/); assert.match(projector, /files.sort_by_key\(\|path\| path.to_string_lossy\(\).replace/);
  const parser = fs.readFileSync(new URL("./shell_parse.mjs", import.meta.url), "utf8"); assert.match(parser, /"shell-cache.js": "cjs"/);
  for (const workflow of ["studio.yml", "pages.yml"]) { const text = fs.readFileSync(new URL("../../.github/workflows/" + workflow, import.meta.url), "utf8"); assert.ok(text.indexOf("stamp_shell.py") < text.indexOf("run: ./scripts/lane.sh"), `${workflow} stages assets after closure projection`); }
});

test("controlled renderer navigations preserve active HTML and special route ownership", async () => {
  const source = fs.readFileSync(new URL("../shell/sw.template.js", import.meta.url), "utf8");
  let handler, offline = false;
  const rows = new Map([[base + "index.html", "HTML A"], [base + "assets/index.js", "JS A"], [base + "holo.html", "RUNNER A"], [base + "live/frame.html", "FRAME A"]]);
  const match = async (request, options = {}) => {
    const url = new URL(typeof request === "string" ? request : request.url, base);
    if (options.ignoreSearch) url.search = "";
    return rows.has(url.href) ? new Response(rows.get(url.href), { headers: { "content-type": "text/html", "cache-control": "no-store" } }) : undefined;
  };
  const network = [];
  vm.runInNewContext(source, {
    URL, Response, Request, importScripts() {},
    self: { registration: { scope: base }, location: { origin: new URL(base).origin }, addEventListener: (type, fn) => { if (type === "fetch") handler = fn; } },
    caches: { open: async name => ({ match: name === "previews" ? async () => undefined : match }) },
    fetch: async request => { network.push(typeof request === "string" ? request : request.url); if (offline) throw Error("offline"); return new Response("NETWORK B"); },
  });
  const read = async (path, mode = "navigate", method = "GET") => {
    let result;
    handler({ request: { url: new URL(path, base).href, method, mode }, respondWith: promise => { result = promise; } });
    return result === undefined ? null : await result;
  };
  const routes = ["", "?refresh=1", "index.html?refresh=1", "chat", "chat/?id=1", "apps", "app-details?id=1", "templates", "plugins", "plugins/server-1", "library", "library/media", "library/themes", "library/prompts", "settings", "settings/providers/openai", "unknown", "new/version/route", "build", "plugins/server.with.dots"];
  for (offline of [false, true]) {
    for (const route of routes) {
      const response = await read(route);
      assert.equal(await response.text(), "HTML A", route);
      assert.equal(response.headers.get("cache-control"), "no-store");
    }
    assert.equal(await (await read("assets/index.js", "cors")).text(), "JS A");
    assert.equal(await (await read("holo.html")).text(), "RUNNER A");
    assert.equal(await (await read("live/frame.html")).text(), "FRAME A");
    assert.equal((await read("p/project/frame")).status, 404);
    assert.equal((await read("holo/")).status, 302);
    for (const route of ["app", "app/", "app/page"]) assert.equal(await read(route), null);
  }
  assert.deepEqual(network, []);
  rows.delete(base + "index.html"); offline = false;
  assert.equal((await read("chat")).status, 404);
  for (const route of ["engine", "engine/v1/responses", "v1/unknown", "q/core/new-module", "live/new-frame", "prism/new", "prism-code/new", "kernel/new", "fonts/new", "wallpapers/new", "ui", "p", "holo"]) assert.equal(await (await read(route)).text(), "NETWORK B", route);
  assert.equal(await (await read("chat", "cors")).text(), "NETWORK B");
  assert.equal(await read("chat", "navigate", "POST"), null);
  for (const route of ["../outside", "../outside/chat"]) assert.equal(await read(route), null);
});
