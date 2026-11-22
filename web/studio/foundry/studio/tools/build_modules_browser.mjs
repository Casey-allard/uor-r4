// Browser transport controls use a local origin while preserving production request options.
import assert from "node:assert/strict";
import fs from "node:fs";
import http from "node:http";
import { createHash } from "node:crypto";
import { chromium } from "../vendor/prism/node_modules/playwright-core/index.mjs";
const moduleSource = fs.readFileSync(new URL("../shell/build-modules.mjs", import.meta.url));
const moduleText = "\ufeff// café\r\nexport default 7";
let leaks = 0, aborted = 0, requests = 0, cookies = 0;
const server = http.createServer((request, response) => {
  if (request.url === "/") { response.setHeader("set-cookie", "private=do-not-forward; Path=/"); response.end("<!doctype html><title>Build module transport</title>"); return; }
  if (request.url === "/module.mjs") { response.setHeader("content-type", "text/javascript"); response.end(moduleSource); return; }
  if (!["/redirect", "/leak", "/stall", "/invalid", "/success"].includes(request.url)) { response.writeHead(404); response.end(); return; }
  requests++; if (request.headers.cookie) cookies++;
  if (request.url === "/redirect") { response.writeHead(302, { location: "/leak" }); response.end(); return; }
  if (request.url === "/leak") { leaks++; response.end("secret"); return; }
  response.setHeader("content-type", "application/javascript; charset=utf-8");
  if (request.url === "/stall") { response.flushHeaders(); response.on("close", () => aborted++); return; }
  if (request.url === "/invalid") { response.end(Buffer.from([255])); return; }
  response.end(moduleText);
});
await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
let browser;
try {
  browser = await chromium.launch({ executablePath: process.env.CHROME_BIN || ["/usr/bin/chromium", "/usr/bin/google-chrome"].find(fs.existsSync), headless: true, args: ["--no-sandbox"] });
  const page = await browser.newPage(); await page.goto(`http://127.0.0.1:${server.address().port}/`);
  const result = await page.evaluate(async () => {
    const { withBuildModules } = await import("/module.mjs");
    const transport = (url, init) => fetch(new URL(url).pathname, init);
    const expect = async (operation, code) => { let actual; try { await operation(); } catch (e) { actual = e.code; } if (actual !== code) throw Error(`${code} expected, got ${actual}`); };
    const closure = await withBuildModules(async remote => ({ text: await remote.fetchText("https://esm.sh/success") }), { fetch: transport });
    await expect(() => withBuildModules(remote => remote.fetchText("https://evil.example/secret"), { fetch: () => { throw Error("unapproved origin fetched"); } }), "buildModuleOriginRefused");
    await expect(() => withBuildModules(remote => remote.fetchText("https://esm.sh/redirect"), { fetch: transport }), "buildModuleNetworkFailed");
    await expect(() => withBuildModules(remote => remote.fetchText("https://esm.sh/invalid"), { fetch: transport }), "buildModuleEncodingInvalid");
    await expect(() => withBuildModules(remote => remote.fetchText("https://esm.sh/stall"), { fetch: transport, timeoutMs: 100 }), "buildModuleTimedOut");
    const controller = new AbortController();
    await expect(() => withBuildModules(remote => remote.fetchText("https://esm.sh/stall"), { signal: controller.signal, timeoutMs: 1000, fetch: async (url, init) => { const response = await transport(url, init); controller.abort(); return response; } }), "buildModuleCancelled");
    return { text: closure.text, receipt: closure.moduleClosure.modules[0], bytes: Array.from(closure.moduleFiles[0].bytes) };
  });
  assert.equal(result.text, moduleText); assert.deepEqual(result.bytes, Array.from(Buffer.from(moduleText)));
  assert.equal(result.receipt.sha256, createHash("sha256").update(moduleText).digest("hex"));
  assert.equal(result.receipt.bytes, Buffer.byteLength(moduleText));
  const until = Date.now() + 1000; while (aborted < 2 && Date.now() < until) await new Promise(resolve => setTimeout(resolve, 10));
  assert.equal(leaks, 0); assert.equal(cookies, 0); assert.equal(requests, 5); assert.equal(aborted, 2);
  console.log(JSON.stringify({ exactModule: true, independentHash: true, redirectRequests: leaks, forwardedCookies: cookies, requests, abortedBodies: aborted, failures: "typed" }));
} finally { await browser?.close(); server.closeAllConnections(); await new Promise(resolve => server.close(resolve)); }
