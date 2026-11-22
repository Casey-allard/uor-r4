#!/usr/bin/env node
// Publishes the canonical frame. Bakes frame/frame.css and the brand's tokens
// (site/brand/hologram-warm.css, pinned by brand.lock) into frame/frame.mjs,
// minifies it with Terser (the exact version package-lock.json pins, so the
// same source always gives the same bytes), names the result by its SHA-256,
// and writes the pointer every app follows:
//
//   <out>/<sha256>/prism-frame.mjs   immutable: the frame, by its content
//   <out>/<sha256>/prism-panels.mjs  immutable: every panel's styles, the
//                                    theme panel and the photos
//                                    (frame/panels.mjs, frame/panels.css),
//                                    whose hash the frame carries
//   <out>/<sha256>/prism-id.mjs      immutable: Prism id (identity/prism-id.mjs,
//                                    pinned to stitch), whose hash the
//                                    panels module carries
//   <out>/current.json               the pointer: { spec, sha256, module, source }
//   <out>/backdrops/<id>.jpg         the immersive photos, and <id>-thumb.jpg
//
// try-page.yml runs it on every push to main into the template's Pages site,
// so the frame at https://uor-foundation.github.io/template/frame/ is always
// main's, and every Prism app picks it up on its next load.
//   node frame/publish.mjs <out-dir> [<source commit>]
// PRISM_FRAME_HOST=<url> publishes for a frame served there instead (a test).
import { createHash } from "node:crypto";
import { copyFile, mkdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { minify as terser } from "terser";

const root = new URL("../", import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1");
const [out = "frame-out", commit = process.env.GITHUB_SHA ?? null] = process.argv.slice(2);

const read = (path) => readFile(join(root, path), "utf8");
const minify = (text) => text.replace(/\/\*[\s\S]*?\*\//g, "").replace(/^[ \t]+/gm, "").replace(/\n{2,}/g, "\n").trim();
const named = (bytes) => createHash("sha256").update(bytes).digest("hex");
const PUBLISHED_HOST = "https://uor-foundation.github.io/template/frame/";
const host = process.env.PRISM_FRAME_HOST ?? PUBLISHED_HOST;
const at = (text) => text.split(JSON.stringify(PUBLISHED_HOST)).join(JSON.stringify(host));
const bake = (text, mark, value, file) => {
  if (!text.includes(mark)) throw new Error(`${file} has no ${mark}`);
  return text.replace(mark, JSON.stringify(value));
};
// Every Prism app runs these bytes, so they ship minified: comments and
// whitespace are Lighthouse's unminified-javascript failure, and the frame
// is the largest script an app page loads.
const minified = async (source, what) => {
  const result = await terser(source, { module: true, compress: { passes: 2 }, mangle: true, format: { comments: false } });
  if (!result.code) throw new Error(`terser produced no ${what}`);
  return Buffer.from(result.code, "utf8");
};

// Prism id first: the panels module carries its hash.
const prismBytes = await minified(await read("identity/prism-id.mjs"), "prism id");
const prismSha = named(prismBytes);
await mkdir(join(out, prismSha), { recursive: true });
await writeFile(join(out, prismSha, "prism-id.mjs"), prismBytes);

// The panels module next: the frame carries its hash.
const themeSource = await read("frame/panels.mjs");
let themeModule = bake(at(themeSource), '"/*PANELS_CSS*/"', minify(await read("frame/panels.css")), "frame/panels.mjs");
themeModule = bake(themeModule, '"/*PRISM_ID*/"', prismSha, "frame/panels.mjs");
const themeBytes = await minified(themeModule, "panels");
const themeSha = named(themeBytes);
await mkdir(join(out, themeSha), { recursive: true });
await writeFile(join(out, themeSha, "prism-panels.mjs"), themeBytes);

let module = at(await read("frame/frame.mjs"));
module = bake(module, '"/*CSS*/"', minify(await read("frame/frame.css")), "frame/frame.mjs");
module = bake(module, '"/*BRAND*/"', minify(await read("site/brand/hologram-warm.css")), "frame/frame.mjs");
module = bake(module, '"/*PANELS*/"', themeSha, "frame/frame.mjs");
const bytes = await minified(module, "frame");
const sha256 = named(bytes);
await mkdir(join(out, sha256), { recursive: true });
await writeFile(join(out, sha256, "prism-frame.mjs"), bytes);
// The immersive photos, beside the frame: every app page may fetch from the
// frame's host (its connect-src names it), so the frame can draw them on a
// page whose img-src allows only its own files and data:. A photo the panels
// module names and the repository lacks fails the publish.
await mkdir(join(out, "backdrops"), { recursive: true });
let photos = 0;
for (const [, id] of themeSource.matchAll(/\bid: "([\w-]+)",\s*\n\s*image: "https:\/\/images\.unsplash\.com\//g)) {
  for (const file of [`${id}.jpg`, `${id}-thumb.jpg`]) {
    await copyFile(join(root, "frame/backdrops", file), join(out, "backdrops", file));
  }
  photos++;
}
if (!photos) throw new Error("frame/panels.mjs names no built-in photo");
const pointer = { spec: "prism/frame/1", sha256, module: `${sha256}/prism-frame.mjs`, source: { repository: "UOR-Foundation/template", commit } };
await writeFile(join(out, "current.json"), JSON.stringify(pointer, null, 2) + "\n");
console.log(`frame: ${sha256} (${bytes.length} bytes), panels ${themeSha} (${themeBytes.length} bytes), prism id ${prismSha} (${prismBytes.length} bytes), ${photos} photos -> ${out}/current.json`);
