import assert from "node:assert/strict";
import fs from "node:fs";
import http from "node:http";
import { inflateSync } from "node:zlib";
import { chromium } from "../vendor/prism/node_modules/playwright-core/index.mjs";
const source = fs.readFileSync(new URL("../shell/exact-representation.mjs", import.meta.url));
const server = http.createServer((request, response) => {
  response.setHeader("content-type", request.url === "/representation.mjs" ? "text/javascript" : "text/html");
  response.end(request.url === "/representation.mjs" ? source : "<!doctype html><title>Byte representation acceptance</title>");
});
await new Promise(resolve => server.listen(0, "127.0.0.1", resolve));
const executablePath = process.env.CHROME_BIN || ["/usr/bin/chromium", "/usr/bin/google-chrome"].find(path => fs.existsSync(path));
let browser;
try {
  browser = await chromium.launch({ executablePath, headless: true, args: ["--no-sandbox"] });
  const page = await browser.newPage(); await page.goto(`http://127.0.0.1:${server.address().port}/`);
  const report = await page.evaluate(async () => {
    const { packBytes, unpackBytes, MAX_BYTES } = await import("/representation.mjs");
    const original = Uint8Array.from({ length: MAX_BYTES }, (_, i) => i % 256); const basis = original.slice(); basis[0] = 9;
    const packages = [];
    for (const input of [null, basis]) {
      const packed = await packBytes(original, input); const restored = await unpackBytes(packed.bytes, packed.identity, input);
      if (restored.length !== original.length || !restored.every((value, index) => value === original[index])) throw Error("browser reconstruction differs");
      packages.push({ manifest: JSON.parse(new TextDecoder().decode(packed.bytes)), accounting: packed.accounting });
    }
    const conditional = await packBytes(original, basis);
    const refused = async (operation, code) => { try { await operation(); throw Error("unexpected acceptance"); } catch (error) { if (error.code !== code) throw error; } };
    await refused(() => unpackBytes(conditional.bytes, conditional.identity), "missingBasis");
    basis[1] ^= 1; await refused(() => unpackBytes(conditional.bytes, conditional.identity, basis), "basisMismatch");
    const controller = new AbortController(); controller.abort();
    await refused(() => packBytes(original, null, { signal: controller.signal }), "representationCancelled");
    return { packages, originalLength: original.length, missingBasis: "refused", wrongBasis: "refused", cancellation: "refused" };
  });
  assert.equal(report.originalLength, 2097152);
  for (const { manifest, accounting } of report.packages) {
    const restored = new Uint8Array(inflateSync(Buffer.from(manifest.data, "base64")));
    if (manifest.basis !== null) restored.forEach((value, index) => { restored[index] = value ^ (index === 0 ? 9 : index % 256); });
    assert.equal(restored.length, report.originalLength); assert.ok(restored.every((value, index) => value === index % 256));
    assert.equal(accounting.reconstructionBytes, accounting.representationBytes + accounting.basisBytes);
  }
  console.log(JSON.stringify({ bytesPerCase: report.originalLength, independentlyReconstructed: report.packages.length, missingBasis: report.missingBasis, wrongBasis: report.wrongBasis, cancellation: report.cancellation }));
} finally { if (browser) await browser.close(); server.closeAllConnections(); await new Promise(resolve => server.close(resolve)); }
