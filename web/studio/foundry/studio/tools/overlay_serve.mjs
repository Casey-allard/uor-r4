// Development only: this branch's shell sources over the deployed site, served under /foundry/ as
// Pages serves it. A file that exists in studio/shell is served from here; anything else (the built
// renderer, the core, the worker, the models' manifests) is fetched once from the deployed site and
// kept in a cache folder. It lets a change to the shell be tried in the real app without the lane.
//
//   node tools/overlay_serve.mjs [port] [cache dir]        PORT in the environment also sets the port
//
// What it cannot show: anything the lane derives from the shell's bytes (the worker's closure hashes,
// the page projected by the core). The deployed copies of those describe the deployed shell.
import fs from "node:fs";
import http from "node:http";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const shell = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../shell");
const port = Number(process.env.PORT || process.argv[2] || 4190);
const cache = path.resolve(process.argv[3] || path.join(os.tmpdir(), "foundry-overlay-cache"));
const SITE = "https://uor-foundation.github.io/foundry/";
const TYPES = { ".html": "text/html; charset=utf-8", ".js": "text/javascript", ".mjs": "text/javascript", ".css": "text/css", ".json": "application/json", ".wasm": "application/wasm", ".svg": "image/svg+xml", ".png": "image/png", ".jpg": "image/jpeg", ".webp": "image/webp", ".woff2": "font/woff2", ".webmanifest": "application/manifest+json", ".txt": "text/plain; charset=utf-8" };
const type = (file) => TYPES[path.extname(file)] || "application/octet-stream";
const inside = (root, file) => file === root || file.startsWith(root + path.sep);

async function deployed(rel) {
  const file = path.join(cache, rel || "index.html");
  if (!inside(cache, file)) return null;
  if (fs.existsSync(file) && fs.statSync(file).isFile()) return { body: fs.readFileSync(file), status: 200 };
  const res = await fetch(SITE + rel);
  const body = Buffer.from(await res.arrayBuffer());
  if (res.status === 200) { fs.mkdirSync(path.dirname(file), { recursive: true }); fs.writeFileSync(file, body); }
  return { body, status: res.status };
}

http.createServer(async (req, res) => {
  try {
    const url = new URL(req.url, "http://x");
    if (!url.pathname.startsWith("/foundry/")) { res.writeHead(302, { location: "/foundry/" }); return res.end(); }
    let rel = decodeURIComponent(url.pathname.slice("/foundry/".length));
    const local = path.join(shell, rel);
    const headers = { "cache-control": "no-store", "service-worker-allowed": "/foundry/" };
    if (rel && inside(shell, local) && fs.existsSync(local) && fs.statSync(local).isFile()) { res.writeHead(200, { ...headers, "content-type": type(local), "x-overlay": "branch" }); return res.end(fs.readFileSync(local)); }
    // Pages answers a path it does not have with 404.html, which is the app's one page.
    const page = !path.extname(rel);
    const got = await deployed(page ? (rel === "" ? "index.html" : "404.html") : rel);
    if (!got) { res.writeHead(403); return res.end(); }
    res.writeHead(page && rel !== "" ? 404 : got.status, { ...headers, "content-type": page ? TYPES[".html"] : type(rel), "x-overlay": "deployed" });
    res.end(got.body);
  } catch (e) { res.writeHead(502, { "content-type": "text/plain" }); res.end(String(e && e.message || e)); }
}).listen(port, "127.0.0.1", () => console.log(`http://127.0.0.1:${port}/foundry/  (shell: ${shell}; deployed files cached in ${cache})`));
