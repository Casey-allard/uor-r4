#!/usr/bin/env node
// Prism ids on one machine: two demo apps, each its own origin
// (http://127.0.0.1:8791 and http://127.0.0.1:8792), both mounting the frame
// as frame/publish.mjs bakes it, so each keeps its own device of a visitor's
// Prism id. Devices meet on stitch's relay (the frame's PRISM_RELAY), which
// the apps' policy admits; identity/test.mjs stands a relay in for it in the
// browser, and run by hand the real one is used. Used by identity/test.mjs;
// run it by hand to try the flow:
//   node identity/dev.mjs
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { createServer } from "node:http";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));

// The frame as publish.mjs bakes it, without minifying: its panels module
// and Prism id beside it, each named by its SHA-256, on the demo app's own
// origin.
const PUBLISHED_HOST = "https://uor-foundation.github.io/template/frame/";
async function bakedFrame(host) {
  const at = (text) => text.split(JSON.stringify(PUBLISHED_HOST)).join(JSON.stringify(host));
  const prismId = await readFile(join(root, "identity/prism-id.mjs"), "utf8");
  const prismSha = createHash("sha256").update(prismId).digest("hex");
  const panels = at(await readFile(join(root, "frame/panels.mjs"), "utf8"))
    .replace('"/*PANELS_CSS*/"', JSON.stringify(await readFile(join(root, "frame/panels.css"), "utf8")))
    .replace('"/*PRISM_ID*/"', JSON.stringify(prismSha));
  const sha = createHash("sha256").update(panels).digest("hex");
  let module = at(await readFile(join(root, "frame/frame.mjs"), "utf8"));
  module = module.replace('"/*CSS*/"', JSON.stringify(await readFile(join(root, "frame/frame.css"), "utf8")));
  module = module.replace('"/*BRAND*/"', JSON.stringify(await readFile(join(root, "site/brand/hologram-warm.css"), "utf8")));
  return { frame: module.replace('"/*PANELS*/"', JSON.stringify(sha)), panels, sha, prismId, prismSha };
}

function app(port, name, relay) {
  const origin = `http://127.0.0.1:${port}`;
  return createServer(async (request, response) => {
    const path = new URL(request.url, origin).pathname;
    if (path === "/frame.mjs" || path.endsWith("/prism-panels.mjs") || path.endsWith("/prism-id.mjs")) {
      const baked = await bakedFrame(`${origin}/frame/`);
      const served = { "/frame.mjs": baked.frame, [`/frame/${baked.sha}/prism-panels.mjs`]: baked.panels, [`/frame/${baked.prismSha}/prism-id.mjs`]: baked.prismId }[path];
      if (served === undefined) {
        response.writeHead(404);
        return response.end();
      }
      response.writeHead(200, { "content-type": "text/javascript", "cache-control": "no-store" });
      return response.end(served);
    }
    response.writeHead(200, {
      "content-type": "text/html; charset=utf-8",
      // As a built page's policy: its own origin and the relay, nothing framed.
      "content-security-policy": `default-src 'self'; script-src 'self' blob:; style-src 'unsafe-inline'; connect-src 'self' ${relay} ${relay.replace(/^https:/, "wss:")}; frame-src 'none'`,
    });
    response.end(`<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>${name}</title><script type="module" src="/frame.mjs"></script></head>
<body><prism-frame name="${name}" live="${origin}/"></prism-frame></body></html>`);
  });
}

export async function start({ ports = [8791, 8792] } = {}) {
  const PRISM_RELAY = /export const PRISM_RELAY = "([^"]+)"/.exec(await readFile(join(root, "frame/frame.mjs"), "utf8"))[1];
  const servers = ports.map((port, i) => app(port, i ? "Second demo" : "Account demo", PRISM_RELAY));
  await Promise.all(servers.map((server, i) => new Promise((done) => server.listen(ports[i], "127.0.0.1", done))));
  return {
    apps: ports.map((port) => `http://127.0.0.1:${port}/`),
    relay: PRISM_RELAY,
    close: () => Promise.all(servers.map((server) => new Promise((done) => server.close(done)))),
  };
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const running = await start();
  console.log(`demo apps ${running.apps.join(" and ")}, devices meet on ${running.relay}`);
}
