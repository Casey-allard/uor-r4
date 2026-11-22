import assert from "node:assert/strict";
import fs from "node:fs";
import http from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { build } from "../vendor/prism/node_modules/esbuild/lib/main.js";
import ts from "../vendor/prism/node_modules/typescript/lib/typescript.js";
import { chromium } from "../vendor/prism/node_modules/playwright-core/index.mjs";
import { privacySource } from "../vendor/prism/hologram.privacy.mjs";
const root = fileURLToPath(new URL("../vendor/prism/", import.meta.url));
const text = privacySource("src/renderer.tsx", fs.readFileSync(path.join(root, "src/renderer.tsx"), "utf8"));
const parsed = ts.createSourceFile("renderer.tsx", text, ts.ScriptTarget.Latest, true, ts.ScriptKind.TSX);
const declarations = []; const visit = node => { if (ts.isVariableStatement(node) && node.declarationList.declarations.some(d => d.name.getText(parsed) === "posthogClient")) declarations.push(node.getText(parsed)); ts.forEachChild(node, visit); }; visit(parsed); assert.equal(declarations.length, 1);
const bundle = async contents => (await build({ stdin: { contents, resolveDir: root, sourcefile: "privacy-fixture.js" }, bundle: true, write: false, format: "iife", platform: "browser", logLevel: "silent" })).outputFiles[0].text;
const positive = await bundle(`import posthog from "posthog-js";
${declarations[0]}
posthogClient.capture("fixture_event", { public: true });
posthogClient.captureException(new Error("public fixture"));
posthogClient.identify("public-fixture");
posthogClient.register({ public: true });
posthogClient.getFeatureFlag("fixture_flag");
window.privacyResult = { token: posthogClient.config?.token || null, identity: posthogClient.get_distinct_id() || null };`);
// The negative control uses the real installed SDK, never a mock analytics client.
const negative = await bundle('import posthog from "posthog-js"; posthog.init("phc_public_negative_control", { api_host: "https://telemetry.invalid", autocapture: false, capture_pageview: false }); window.privacyResult = true;');
const server = http.createServer((request, response) => {
  response.setHeader("content-type", request.url.endsWith(".js") ? "text/javascript" : "text/html");
  response.end(request.url === "/positive.js" ? positive : request.url === "/negative.js" ? negative : '<!doctype html><title>Privacy acceptance</title>');
});
await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
const base = `http://127.0.0.1:${server.address().port}`;
const executablePath = process.env.CHROME_BIN || ["/usr/bin/chromium", "/usr/bin/google-chrome"].find(p => fs.existsSync(p));
let browser;
try {
  browser = await chromium.launch({ executablePath, headless: true, args: ["--no-sandbox"] });
  for (const legacy of [false, true]) {
    const context = await browser.newContext(), page = await context.newPage(), attempts = [];
    page.on("pageerror", error => console.error("Privacy SDK fixture:", error.message));
    await page.route("**/*", route => { if (new URL(route.request().url()).origin === base) return route.continue(); attempts.push(route.request().url()); return route.abort(); });
    await page.goto(base);
    if (legacy) await page.evaluate(() => localStorage.setItem("prismTelemetryConsent", "opted_in"));
    await page.addScriptTag({ url: base + "/positive.js" }); await page.waitForFunction(() => !!window.privacyResult); await page.waitForTimeout(750);
    assert.deepEqual(await page.evaluate(() => window.privacyResult), { token: null, identity: null }); assert.equal(attempts.length, 0);
    assert.equal((await context.cookies()).length, 0); assert.deepEqual(await page.evaluate(() => Object.keys(localStorage).filter(key => key !== "prismTelemetryConsent")), []);
    await context.close();
  }
  const context = await browser.newContext(), page = await context.newPage(), attempts = [];
  await page.route("**/*", route => { if (new URL(route.request().url()).origin === base) return route.continue(); attempts.push(route.request().url()); return route.abort(); });
  await page.goto(base); await page.addScriptTag({ url: base + "/negative.js" }); await page.waitForTimeout(1500);
  assert.ok(attempts.some(url => new URL(url).hostname === "telemetry.invalid"), "observer failed to detect SDK initialization transport");
  console.log(JSON.stringify({ fresh: "no-telemetry", legacyOptIn: "no-telemetry", persistentIdentifiers: 0, sdk: "actual-installed-package", negativeControl: "detected-before-transmission" }));
} finally { if (browser) await browser.close(); await new Promise(resolve => server.close(resolve)); }
