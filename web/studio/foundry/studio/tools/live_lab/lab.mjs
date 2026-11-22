// Prism Live lab: the studio's own live frame, driven from a canonical LexLean module instead of a
// spec JSON. This page stands in for the host (host.js does the same with a chat turn): it reads the
// LexLean as it is written, projects it with live/project.mjs, checks it with the local obligations,
// and posts each projection to the frame exactly as the studio does.
import * as live from "../../shell/live/project.mjs";
import { frameHtml, FONTS } from "../../shell/live/frame-html.mjs";
import * as prism from "../../shell/prism.js";
import { readLexLean, readLexLeanPartial } from "../../shell/live/lexlean.mjs";
import { checkObligations } from "../../shell/live/obligations.mjs";

const $ = (id) => document.getElementById(id);
const BASE = new URL("../../shell/", import.meta.url);
const APP = "1";
const stats = { posts: 0, painted: [], firstControls: null, firstAnswer: null, streamStart: null, streamEnd: null, checks: [] };
window.__lab = { stats };

async function frameUrl() {
  const text = async (p) => (await fetch(new URL(p, BASE))).text();
  const fonts = {};
  for (const p of FONTS) {
    const r = await fetch(new URL(p, BASE));
    if (r.ok) { const b = new Uint8Array(await r.arrayBuffer()); let s = ""; for (let i = 0; i < b.length; i += 0x8000) s += String.fromCharCode(...b.subarray(i, i + 0x8000)); fonts[p] = btoa(s); }
  }
  const html = frameHtml({ brandCss: await text("brand.css"), componentsCss: await text("live/components.css"), componentsJs: await text("live/components.js"), interpreter: prism.INTERPRETER, fonts });
  return URL.createObjectURL(new Blob([html], { type: "text/html" })) + "#" + APP;
}

let frameWin = null, frameReady = null;
const ready = new Promise((r) => (frameReady = r));
addEventListener("message", (e) => {
  const m = e.data;
  if (!m || e.source !== frameWin) return;
  if (m.type === "live/hello") frameReady();
  if (m.type === "live/painted") {
    stats.painted.push({ at: performance.now(), ms: m.ms, changed: m.changed });
    if (stats.pendingStage) { const s = stats.pendingStage; stats.pendingStage = null; if (s === "fields" && !stats.firstControls && stats.streamStart) stats.firstControls = performance.now() - stats.streamStart; if (s !== "fields" && !stats.firstAnswer && stats.streamStart) stats.firstAnswer = performance.now() - stats.streamStart; }
  }
});

let last = { root: null };
function post(spec, trust, stage) {
  const p = live.project(spec);
  if (p.root === last.root && trust === last.trust) return;
  last = { root: p.root, trust };
  stats.posts++;
  stats.pendingStage = stage;
  frameWin.postMessage({ type: "live/projection", app: APP, root: p.root, logic: p.logic, logicKappa: p.logicKappa, nodes: p.nodes, trust, theme: matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light", edit: false }, "*");
}

// Project whatever is whole: a module still being written projects as a draft; a complete one is
// checked by its local obligations, and only then labelled checked.
function project(text, { streaming = false } = {}) {
  const t0 = performance.now();
  const part = readLexLeanPartial(text);
  if (!part.spec) { $("stage").textContent = streaming ? "writing…" : `draft: ${part.error || "not a complete module yet"}`; return; }
  if (!part.complete) { post(part.spec, "draft", part.stage); $("stage").textContent = `writing · ${part.stage}: ${part.spec.fields.length} fields, ${part.spec.rules.length} rules, ${part.spec.properties.length} guarantees`; return; }
  const ob = checkObligations(part.spec);
  const ms = performance.now() - t0;
  stats.checks.push({ ms, fresh: ob.fresh, hits: ob.hits });
  if (!ob.ok) { post(part.spec, "draft", "rules"); $("stage").textContent = "a rule uses != : outside the obligation fragment (kernel check needed)"; return; }
  post(part.spec, ob.holds ? "checked" : "draft", "rules");
  $("stage").innerHTML = ob.holds
    ? `<b>checked</b> · ${ob.total} obligations: ${ob.fresh} new, ${ob.hits} reused · read + check ${ms.toFixed(1)} ms · proof manifest <code>${ob.manifest.slice(7, 23)}</code>`
    : `<b class="bad">breaks ${ob.broken}</b> at <code>${ob.witness.map((v, i) => { const f = part.spec.fields[i]; return f.kind === "choice" ? f.values[v] : v; }).join(" ")}</code> · ${ms.toFixed(1)} ms`;
}

$("src").addEventListener("input", () => project($("src").value));

// A model streaming the module: chunks at a chosen rate, projected as each declaration completes.
$("stream").onclick = async () => {
  const full = await (await fetch("../fixtures/live/verdant.lex.tex")).text();
  const rate = Number($("rate").value); // characters per second
  $("src").value = ""; last = { root: null };
  stats.firstControls = stats.firstAnswer = null; stats.streamStart = performance.now(); stats.posts = 0; stats.painted = [];
  const step = Math.max(8, Math.round(rate / 60)); // about one chunk per frame
  for (let i = 0; i < full.length; i += step) {
    $("src").value = full.slice(0, i + step);
    project($("src").value, { streaming: true });
    await new Promise((r) => setTimeout(r, (step / rate) * 1000));
  }
  stats.streamEnd = performance.now() - stats.streamStart;
  project($("src").value);
  $("streamed").textContent = `streamed ${full.length.toLocaleString()} characters in ${(stats.streamEnd / 1000).toFixed(1)} s · controls painted at ${stats.firstControls?.toFixed(0)} ms · answers live at ${stats.firstAnswer?.toFixed(0)} ms · ${stats.posts} projections`;
};

frameWin = (() => { const f = $("frame"); return f.contentWindow; })();
frameUrl().then((u) => { $("frame").src = u; frameWin = $("frame").contentWindow; });
await ready;
$("src").value = await (await fetch("../fixtures/live/verdant.lex.tex")).text();
project($("src").value);
