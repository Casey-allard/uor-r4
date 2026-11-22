// App covers in a real browser: a preview served under the worker's own sandbox policy (an opaque origin,
// the policy text read from sw.template.js), carrying the responder and the locked html-to-image, is
// captured from the page that frames it. Proves the picture crosses the sandbox, the passing toast is
// hidden, the cover is a 1600x800 WebP, a blank preview yields no cover, and a frame that is not the one
// asked cannot answer for it.
import assert from "node:assert/strict";
import fs from "node:fs";
import http from "node:http";
import { chromium } from "../vendor/prism/node_modules/playwright-core/index.mjs";
import { withResponder } from "../shell/covers.mjs";

const read = (p) => fs.readFileSync(new URL(p, import.meta.url), "utf8");
const sw = read("../shell/sw.template.js");
const block = (start) => { const i = sw.indexOf(start); assert.ok(i >= 0, "sw.template.js lost " + start); let depth = 0, j = sw.indexOf("{", i); for (; j < sw.length; j++) { if (sw[j] === "{") depth++; else if (sw[j] === "}" && --depth === 0) break; } return sw.slice(i, j + 1); };
const sandbox = sw.match(/const SANDBOX = "[^"]+";/)[0];
const policyOf = new Function(`${sandbox}\n${block("function policyOf(endpoints)")}\nreturn policyOf;`)();
const library = read("../vendor/prism/node_modules/html-to-image/dist/html-to-image.js");
const covers = read("../shell/covers.mjs");

const app = `<!doctype html><html><head><meta charset="utf-8"><title>Pantry</title><style>
body{margin:0;font:16px system-ui;background:#f6f4ef;color:#1b1b1f} header{display:flex;justify-content:space-between;padding:20px 40px;background:#1b1b1f;color:#fff}
main{padding:40px;display:grid;grid-template-columns:repeat(3,1fr);gap:24px} .card{background:#fff;border-radius:16px;padding:24px;height:180px;box-shadow:0 1px 3px #0002}
.card b{display:block;font-size:28px;color:#c2410c} .toast{position:fixed;right:24px;bottom:24px;width:280px;padding:16px;background:#e11d48;color:#fff;border-radius:10px}
</style></head><body><header><strong>Pantry planner</strong><span>Week 46</span></header><main>
<div class="card"><b>12</b>meals planned from what you already have</div><div class="card"><b>4</b>items running low</div><div class="card"><b>£38</b>saved this month</div>
</main><div class="toast" role="status">Saved offline</div></body></html>`;
const blank = `<!doctype html><html><body style="margin:0;background:#fafafa"></body></html>`;

const pages = {
  "/": [`<!doctype html><title>Covers acceptance</title>`, "text/html", null],
  "/covers.mjs": [covers, "text/javascript", null],
  "/p/app/": [withResponder(app, library), "text/html", policyOf([])],
  "/p/blank/": [withResponder(blank, library), "text/html", policyOf([])],
  "/p/bare/": [app, "text/html", policyOf([])],
};
const server = http.createServer((request, response) => {
  const hit = pages[request.url];
  if (!hit) { response.statusCode = 404; return response.end(); }
  response.setHeader("content-type", hit[1]);
  if (hit[2]) response.setHeader("content-security-policy", hit[2]);
  response.end(hit[0]);
});
await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
const executablePath = process.env.CHROME_BIN || ["/usr/bin/chromium", "/usr/bin/google-chrome"].find((p) => fs.existsSync(p));
let browser;
try {
  browser = await chromium.launch({ executablePath, headless: true, args: ["--no-sandbox"] });
  const page = await browser.newPage();
  await page.goto(`http://127.0.0.1:${server.address().port}/`);
  const report = await page.evaluate(async () => {
    const covers = await import("/covers.mjs");
    const fast = { settle: 300, retry: 600, timeout: 8000 };
    const shot = await covers.capture("/p/app/", fast);
    if (!shot) return { app: "no cover" };
    const blob = await covers.encodeCover(shot.png);
    const bmp = await createImageBitmap(blob);
    // The toast sat in the bottom right; its pixels must be the page's ground, not its red.
    const c = new OffscreenCanvas(bmp.width, bmp.height); const g = c.getContext("2d"); g.drawImage(bmp, 0, 0);
    const [r, gr, b] = g.getImageData(bmp.width - 120, bmp.height - 60, 1, 1).data;
    const toastHidden = !(r > 180 && gr < 80 && b < 110);
    const blankShot = await covers.capture("/p/blank/", fast);
    const bareShot = await covers.capture("/p/bare/", { settle: 200, retry: 200, timeout: 1500 });
    // Another frame answers every request id first, with a valid picture of its own; the capture must
    // take only the asked frame's answer.
    const red = (() => { const c = document.createElement("canvas"); c.width = 64; c.height = 32; const g = c.getContext("2d"); for (let x = 0; x < 64; x += 8) { g.fillStyle = x % 16 ? "#f00" : "#00f"; g.fillRect(x, 0, 8, 32); } return c.toDataURL("image/png"); })();
    const decoy = document.createElement("iframe");
    decoy.srcdoc = `<script>setInterval(() => { for (let id = 1; id < 40; id++) parent.postMessage({ type: "hologram/cover", id, png: ${JSON.stringify(red)}, text: "forged" }, "*"); }, 20)<\/script>`;
    document.body.appendChild(decoy);
    const real = await covers.capture("/p/app/", fast);
    decoy.remove();
    const forged = real && /Pantry planner/.test(real.text) ? "ignored" : "accepted";
    return { app: { type: blob.type, width: bmp.width, height: bmp.height, toastHidden, textSeen: /Pantry planner/.test(shot.text) }, blank: blankShot === null ? "no cover" : "cover", bare: bareShot === null ? "no cover" : "cover", forged };
  });
  assert.deepEqual(report, { app: { type: "image/webp", width: 1600, height: 800, toastHidden: true, textSeen: true }, blank: "no cover", bare: "no cover", forged: "ignored" });
  await page.evaluate(async () => {
    const covers = await import("/covers.mjs");
    const shot = await covers.capture("/p/app/", { settle: 300, retry: 600, timeout: 8000 });
    const blob = await covers.encodeCover(shot.png);
    window.__cover = await new Promise((r) => { const f = new FileReader(); f.onload = () => r(f.result); f.readAsDataURL(blob); });
  });
  if (process.env.COVER_OUT) fs.writeFileSync(process.env.COVER_OUT, Buffer.from((await page.evaluate(() => window.__cover)).split(",")[1], "base64"));
  console.log(JSON.stringify(report));
} finally {
  if (browser) await browser.close();
  await new Promise((resolve) => server.close(resolve));
}
