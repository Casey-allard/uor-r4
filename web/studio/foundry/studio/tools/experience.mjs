// The Experience gate: what the page does about motion, tap targets, layout and "this week" agrees with the
// Forge kernel, which decides each (FK-16 to FK-21, proved by Lean in app/src).
//
// The renderer asks the kernel at run time where it can (the chat layout, the Apps page's "this week"); a
// stylesheet cannot ask anything, so its values are adapters and this gate holds them to the kernel's
// answers. The thresholds are read from the kernel, never typed here: the guest KERNEL_REV pins is loaded
// through shell/kernel.js exactly as the page loads it, and asked until each boundary is found.
//
//   node tools/experience.mjs --check
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const studio = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const shell = path.join(studio, "shell");
globalThis.self = globalThis;
await import(pathToFileURL(path.join(shell, "kernel.js")).href);
const kernel = await self.ForgeKernel.load(async (p) => new Response(fs.readFileSync(path.join(shell, p))));
const failures = [];

// The first value in [lo, hi] at which test(value) turns true; test must be false below it and true from it on.
async function boundary(lo, hi, test) {
  if (!(await test(hi)) || (await test(lo))) return null;
  while (hi - lo > 1) { const mid = (lo + hi) >> 1; if (await test(mid)) hi = mid; else lo = mid; }
  return hi;
}
const tap = await boundary(0, 4096, async (px) => (await kernel.ask(`target ${px}`)) === "ok");
const longest = (await boundary(0, 60000, async (ms) => (await kernel.ask(`motion ${ms} no`)) === "deny")) - 1;
const reducedDenies = (await kernel.ask("motion 1 yes")) === "deny" && (await kernel.ask("motion 0 yes")) === "deny";
const split = await boundary(0, 16384, async (w) => (await kernel.ask(`layout ${w}`)) === "split");
const week = (await boundary(0, 1 << 30, async (s) => (await kernel.ask(`recent ${s}`)) === "no")) - 1;
console.log(`kernel: tap targets from ${tap} px, motion up to ${longest} ms and none when reduced (${reducedDenies}), split from ${split} px, this week up to ${week} s`);
if (tap == null || split == null || !Number.isFinite(longest) || !Number.isFinite(week)) failures.push("the kernel did not answer every experience request: is the guest older than FK-16 to FK-21?");
if (!reducedDenies) failures.push("the kernel allows motion when reduced motion is asked");

// appearance.css, the shell's own stylesheet: the touch hit area, every duration, the reduced-motion rule.
const cssPath = path.join(shell, "appearance.css");
const css = fs.readFileSync(cssPath, "utf8");
const line = (at) => css.slice(0, at).split("\n").length;
const hit = [...css.matchAll(/(width|height): max\(100%, (\d+)px\)/g)];
if (hit.length < 2) failures.push("appearance.css: the touch hit area (width and height max(100%, Npx)) is gone");
for (const m of hit) if (+m[2] < tap) failures.push(`appearance.css:${line(m.index)}: a ${m[2]} px hit area is under the kernel's ${tap} px tap target`);
// One kind of motion answers no press: the composer's ambient colour drift, an infinite loop. It is the one
// exception to the kernel's duration, named here, and it stands only while the reduced-motion rule below
// stops it with everything else.
const AMBIENT = /^animation: spectrum-drift \d+s linear infinite$/;
for (const m of css.matchAll(/(?:transition|animation)(?:-duration)?:[^;}]*?(\d*\.?\d+)(ms|s)\b[^;}]*/g)) {
  if (AMBIENT.test(m[0].trim())) continue;
  const ms = m[2] === "s" ? +m[1] * 1000 : +m[1];
  if (ms > longest) failures.push(`appearance.css:${line(m.index)}: ${m[0].trim()} runs ${ms} ms, longer than the kernel allows (${longest} ms)`);
}
const reduced = css.match(/@media \(prefers-reduced-motion: reduce\) \{\s*\*, ::before, ::after \{[^}]*animation-duration: 0\.01ms !important;[^}]*transition-duration: 0\.01ms !important;/);
if (!reduced) failures.push("appearance.css: no rule stops every animation and transition under prefers-reduced-motion, which the kernel requires");

// The renderer's two run-time questions, asked of the kernel and decided by nothing else.
const lean = fs.readFileSync(path.join(studio, "vendor", "prism", "hologram.lean.mjs"), "utf8");
const apps = fs.readFileSync(path.join(studio, "vendor", "prism", "hologram.screens", "AppsPage.tsx"), "utf8");
if (!/kernel\.ask\("layout " \+/.test(lean)) failures.push("hologram.lean.mjs: the chat layout no longer asks the kernel");
if (/max-width: \d+px|innerWidth\s*<\s*\d+/.test(lean)) failures.push("hologram.lean.mjs: the chat layout carries a breakpoint of its own");
if (!/ask\("recent " \+/.test(apps)) failures.push("AppsPage.tsx: this week no longer asks the kernel");
if (/\b7 \* 24\b|604800|WEEK\b/.test(apps)) failures.push("AppsPage.tsx: this week carries a length of its own");

if (failures.length) { console.log("Experience gate refused:\n  " + failures.join("\n  ")); process.exit(1); }
console.log("Experience gate: the stylesheet's hit area, durations and reduced-motion rule, the chat layout and this week agree with the kernel");
