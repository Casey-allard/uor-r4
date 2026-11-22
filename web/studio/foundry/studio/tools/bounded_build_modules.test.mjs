import assert from "node:assert/strict";
import { createHash, webcrypto } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
import { approvedModuleURL, withBuildModules } from "../shell/build-modules.mjs";
const options = fetch => ({ fetch, digest: bytes => webcrypto.subtle.digest("SHA-256", bytes) });
const response = (body = "export default 1", init = {}) => new Response(body, { headers: { "content-type": "application/javascript" }, ...init });
const refuses = code => error => error.code === code && !error.message.includes("secret");
test("only the approved origin is requested with redirect credential and cache controls", async () => {
  for (const url of ["http://esm.sh/a", "https://evil.example/a", "https://esm.sh.evil.example/a", "https://user:secret@esm.sh/a", "https://esm.sh/a#fragment", "https://esm.sh/a#", "data:text/javascript,x", "https://esm.sh/" + "x".repeat(8192)]) assert.throws(() => approvedModuleURL(url), refuses("buildModuleOriginRefused"));
  let calls = 0;
  await withBuildModules(async m => { await m.fetchText("https://esm.sh/react@19.1.0"); }, options(async (url, init) => {
    calls++; assert.equal(url, "https://esm.sh/react@19.1.0"); assert.equal(init.method, "GET"); assert.equal(init.redirect, "error"); assert.equal(init.credentials, "omit"); assert.equal(init.cache, "no-store"); return response();
  })); assert.equal(calls, 1);
  await assert.rejects(withBuildModules(m => m.fetchText("https://evil.example/secret"), options(() => assert.fail("unapproved origin fetched"))), refuses("buildModuleOriginRefused"));
});
test("duplicate imports share work and the closure retains exact bytes and independently hashed identities", async () => {
  let calls = 0, held;
  const text = "\ufeff// café\r\nexport default 1";
  const result = await withBuildModules(async m => { held = m; const values = await Promise.all([m.fetchText("https://esm.sh/b"), m.fetchText("https://esm.sh/a"), m.fetchText("https://esm.sh/a")]); assert.deepEqual(values, [text, text, text]); return { count: m.count }; }, options(async () => { calls++; return response(new TextEncoder().encode(text)); }));
  assert.equal(calls, 2); assert.equal(result.count, 2); assert.equal(result.moduleFiles.length, 1);
  assert.deepEqual(result.moduleClosure.modules.map(r => r.url), ["https://esm.sh/a", "https://esm.sh/b"]);
  const hash = createHash("sha256").update(text).digest("hex");
  assert.equal(result.moduleClosure.modules[0].sha256, hash); assert.equal(result.moduleFiles[0].path, `build/modules/${hash}.txt`);
  assert.deepEqual(result.moduleFiles[0].bytes, new TextEncoder().encode(text));
  assert.throws(() => held.fetchText("https://esm.sh/late"), refuses("buildModuleCancelled")); assert.equal(calls, 2);
});
test("HTTP content type UTF8 and digest failures remain typed and private", async () => {
  for (const [fetch, code] of [[async () => response("secret", { status: 500 }), "buildModuleUnavailable"], [async () => response("secret", { headers: { "content-type": "text/html" } }), "buildModuleResponseInvalid"], [async () => response(new Uint8Array([255])), "buildModuleEncodingInvalid"], [async () => { throw Error("secret"); }, "buildModuleNetworkFailed"]]) await assert.rejects(withBuildModules(m => m.fetchText("https://esm.sh/a"), options(fetch)), refuses(code));
  await assert.rejects(withBuildModules(m => m.fetchText("https://esm.sh/a"), { fetch: async () => response(), digest: async () => new ArrayBuffer(1) }), refuses("buildModuleIdentityFailed"));
});
test("per-module aggregate and count limits refuse instead of returning a partial closure", async () => {
  await assert.rejects(withBuildModules(m => m.fetchText("https://esm.sh/large"), options(async () => response(new Uint8Array(8 * 1024 * 1024 + 1)))), refuses("buildModuleTooLarge"));
  await assert.rejects(withBuildModules(async m => { for (let i = 0; i < 9; i++) await m.fetchText("https://esm.sh/" + i); }, options(async () => response(new Uint8Array(8 * 1024 * 1024)))), refuses("buildModuleLimitExceeded"));
  let calls = 0;
  await assert.rejects(withBuildModules(m => { for (let i = 0; i < 513; i++) m.fetchText("https://esm.sh/" + i); }, options(async () => { calls++; return response(); })), refuses("buildModuleLimitExceeded")); assert.equal(calls, 512);
});
test("one deadline bounds headers bodies and callback completion and cancellation aborts fetch", async () => {
  let aborted = false;
  await assert.rejects(withBuildModules(m => m.fetchText("https://esm.sh/a"), { ...options((_, init) => { init.signal.addEventListener("abort", () => { aborted = true; }); return new Promise(() => {}); }), timeoutMs: 10 }), refuses("buildModuleTimedOut")); assert.ok(aborted);
  let cancelledBody = false;
  await assert.rejects(withBuildModules(m => m.fetchText("https://esm.sh/a"), { ...options(async () => response(new ReadableStream({ cancel() { cancelledBody = true; } }))), timeoutMs: 10 }), refuses("buildModuleTimedOut")); assert.ok(cancelledBody);
  await assert.rejects(withBuildModules(() => new Promise(() => {}), { timeoutMs: 10 }), refuses("buildModuleTimedOut"));
  let release, lateCancelled = false;
  await assert.rejects(withBuildModules(m => { void m.fetchText("https://esm.sh/late"); throw Error("callback stopped"); }, options(() => new Promise(resolve => { release = resolve; }))), /callback stopped/);
  release(response(new ReadableStream({ cancel() { lateCancelled = true; } })));
  await new Promise(resolve => setTimeout(resolve, 0)); assert.ok(lateCancelled);
  const controller = new AbortController(); controller.abort();
  await assert.rejects(withBuildModules(() => assert.fail("cancelled callback ran"), { signal: controller.signal }), refuses("buildModuleCancelled"));
});
test("portable build wiring uses the bounded resolver and archives its receipts and sources", () => {
  const host = fs.readFileSync(new URL("../shell/host.js", import.meta.url), "utf8");
  const start = host.indexOf("async function buildPortable("), end = host.indexOf("// Runs inside the compile frame", start), body = host.slice(start, end);
  assert.ok(body.includes("withBuildModules(async remote =>")); assert.ok(body.includes("await remote.fetchText(args.path)")); assert.ok(!body.includes("await fetch(args.path)"));
  assert.ok(host.includes('path: "build/modules.json"')); assert.ok(host.includes("...page.moduleFiles"));
  assert.ok(fs.readFileSync(new URL("./shell_parse.mjs", import.meta.url), "utf8").includes('"build-modules.mjs": "mjs"'));
});
