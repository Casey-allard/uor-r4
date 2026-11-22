// What Foundry keeps about your apps, as one dialog over Forge: each app and the sites it reached or was
// stopped from, then, on demand, how fast builds are. It is a native dialog built by the shell, so the one
// rule in appearance.css gives it the app's look: centred, the golden ratio's measures, brand tokens. It
// writes no colour, type face or control style of its own; the few rules below only lay rows out, in the
// same tokens. The wording is recorded in studio/experience (FoundryExperience.appsOnDevice): it leads with
// one sentence that sums every app up, uses plain words, and goes why, then how, then what.
// Opened by the host (#telemetry, or Ctrl+Alt+T); it reads only through `telemetry`. Every standing it
// shows is the Telemetry model's (TL-22 to TL-24), never decided here.
import { RULES } from "./prometheus.mjs";
import { LEVER_WORDS, SWITCH_WORDS, APP_WORDS, SITE_WORDS, APPS_SUMMARY } from "./words.mjs";

const ID = "foundry-build-timings";
const STYLE = `
#${ID} .ap-list { display: flex; flex-direction: column; gap: 8px; }
#${ID} details > summary { display: flex; align-items: center; gap: 8px; list-style: none; }
#${ID} details > summary::-webkit-details-marker { display: none; }
#${ID} details > summary::after { content: ""; margin-left: auto; flex: none; width: 8px; height: 8px; border: solid var(--muted-foreground); border-width: 0 1.5px 1.5px 0; transform: rotate(45deg); transition: transform 140ms; }
#${ID} details[open] > summary::after { transform: rotate(-135deg); }
#${ID} details[open] > summary { margin-bottom: 8px; }
#${ID} .ap-dot { flex: none; width: 8px; height: 8px; border-radius: 50%; background: color-mix(in srgb, var(--muted-foreground) 55%, transparent); }
#${ID} .ap-dot[data-standing="reaches"] { background: var(--success); }
#${ID} .ap-dot[data-standing="blocked"] { background: var(--destructive); }
#${ID} .ap-name { color: var(--popover-foreground); font-weight: 600; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
#${ID} .ap-line { color: var(--muted-foreground); font-weight: 400; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
#${ID} section > div { display: flex; flex-direction: column; min-width: 0; }
#${ID} section b { color: var(--popover-foreground); font: 500 13px/1.5 var(--font-mono); overflow-wrap: anywhere; }
#${ID} section small { font-size: 12px; }
#${ID} section span { flex: none; text-align: right; }
#${ID} section span[data-standing="blocked"] { color: var(--destructive); font-weight: 500; }
#${ID} .bt-table { overflow-x: auto; }
#${ID} table { width: 100%; min-width: 420px; border-collapse: collapse; }
#${ID} th, #${ID} td { padding: 5px 13px 5px 0; text-align: left; font-variant-numeric: tabular-nums; white-space: nowrap; }
#${ID} th { font-weight: 500; color: var(--muted-foreground); }
#${ID} td.bt-over { color: var(--destructive); font-weight: 600; }
#${ID} .bt-card { border: 1px solid color-mix(in srgb, var(--popover-foreground) 10%, transparent); border-radius: 8px; padding: 8px 13px; display: flex; flex-direction: column; gap: 8px; }
#${ID} .bt-card b { color: var(--popover-foreground); font-weight: 600; }
#${ID} .bt-body { display: flex; flex-direction: column; gap: 13px; }
@media (prefers-reduced-motion: reduce) { #${ID} details > summary::after { transition: none; } }
`;

const fmt = (k, v) => (v == null ? "–" : k.time ? (v >= 1e6 ? (v / 1e6).toFixed(1) + " s" : (v / 1e3).toFixed(v < 1e4 ? 1 : 0) + " ms") : String(v));
const el = (parent, tag, text, attrs) => { const e = parent.appendChild(document.createElement(tag)); if (text != null) e.textContent = text; if (attrs) for (const [k, v] of Object.entries(attrs)) e.setAttribute(k, v); return e; };
const plural = (n, one, many = one + "s") => `${n} ${n === 1 ? one : many}`;
const timesWord = (n) => (n === 1 ? "once" : n === 2 ? "twice" : `${n} times`);
// A site as people read it: its host, without the scheme.
const siteName = (site) => site.replace(/^[a-z][a-z0-9+.-]*:\/\//i, "").replace(/\/$/, "");
const size = (bytes) => (bytes >= 1e6 ? (bytes / 1e6).toFixed(1) + " MB" : bytes >= 1e3 ? Math.round(bytes / 1e3) + " KB" : bytes + " bytes");
// When, in the reader's own words: today and yesterday by the clock, otherwise the date.
export function when(micros, now = Date.now()) {
  const at = new Date(micros / 1000), day = (d) => new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();
  const clock = at.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });
  const days = Math.round((day(new Date(now)) - day(at)) / 864e5);
  if (days === 0) return "today at " + clock;
  if (days === 1) return "yesterday at " + clock;
  return at.toLocaleDateString(undefined, { day: "numeric", month: "short", year: at.getFullYear() === new Date(now).getFullYear() ? undefined : "numeric" });
}

function save(name, text, type) {
  const url = URL.createObjectURL(new Blob([text], { type }));
  const a = document.createElement("a");
  a.href = url; a.download = name; a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

// One app: a line that says what it reached, which opens to each site, how often and when.
function drawApp(list, a) {
  const box = el(list, "details", null, { "data-standing": a.standing });
  const line = el(box, "summary");
  el(line, "i", null, { class: "ap-dot", "data-standing": a.standing, "aria-hidden": "true" });
  el(line, "span", a.name || (a.installed ? "An installed app" : "An app you built"), { class: "ap-name" });
  el(line, "span", APP_WORDS[a.standing](a), { class: "ap-line" });
  const facts = [];
  if (a.opened) facts.push(`Opened ${timesWord(a.opened)}, last ${when(a.last)}.`);
  if (a.size) facts.push(`It takes ${size(a.size)} on this device.`);
  if (a.notOpened) facts.push(`It was stopped from opening ${timesWord(a.notOpened)}, because it had no permission to open.`);
  if (facts.length) el(box, "p", facts.join(" "));
  if (!a.sites.length) { el(box, "p", "It asks for no site, so it works without the internet."); return; }
  for (const s of a.sites) {
    const row = el(box, "section"), left = el(row, "div");
    el(left, "b", siteName(s.site));
    el(left, "small", "last " + when(s.last));
    el(row, "span", SITE_WORDS[s.standing](s), { "data-standing": s.standing });
  }
}

export function openPanel({ telemetry, model }) {
  if (!document.getElementById(ID + "-style")) el(document.head, "style", STYLE, { id: ID + "-style" });
  let dialog = document.getElementById(ID);
  if (!dialog) dialog = el(document.body, "dialog", null, { id: ID });
  dialog.setAttribute("aria-label", "Your apps on this device");
  const draw = async () => {
    dialog.textContent = "";
    el(dialog, "h2", "Your apps on this device");
    let apps = null;
    try { apps = await telemetry.apps(); } catch (e) { el(dialog, "p", "What your apps reached cannot be shown right now: " + e.message); }
    if (apps) {
      // The sentence that sums it up comes first; the explanation that follows says why, how, then what.
      el(dialog, "p", APPS_SUMMARY(apps));
      el(dialog, "p", "To keep what you build in your hands, Foundry checks every site an app asks to reach, each time it opens. Only sites you allowed get through, and this record stays in this browser. Open an app to see each site and when.");
      if (apps.length) { const list = el(dialog, "div", null, { class: "ap-list" }); for (const a of apps) drawApp(list, a); }
    }
    await drawTimings();
    el(dialog, "h3", "Keep or clear");
    el(dialog, "p", "To use this record elsewhere, save it as a file; to start again, erase it. Nothing is saved or erased until you choose.");
    const files = el(dialog, "div");
    el(files, "button", "Save for Prometheus").onclick = async () => save("prism.prom", await telemetry.exportSnapshot({ userAct: true }), "text/plain; version=0.0.4");
    el(files, "button", "Save history").onclick = async () => save("prism-history.om.txt", await telemetry.exportHistory({ userAct: true }), "application/openmetrics-text; version=1.0.0");
    el(files, "button", "Save alert rules").onclick = () => save("prism.rules.yml", RULES, "text/yaml");
    el(files, "button", "Save everything").onclick = async () => save("prism-turns.json", await telemetry.exportTurns({ userAct: true }), "application/json");
    finish();
  };

  // How fast builds are, one line that opens on demand; a suggestion is counted on the line itself.
  async function drawTimings() {
    let r;
    try { r = await telemetry.report(); } catch (e) { el(dialog, "p", "Build times cannot be shown right now: " + e.message); return; }
    const tips = r.suggestions.length + (r.switchTo ? 1 : 0);
    const box = el(dialog, "details");
    const line = el(box, "summary");
    el(line, "span", "How fast your builds are", { class: "ap-name" });
    el(line, "span", r.turns.length ? `${plural(r.turns.length, "build")} timed` + (tips ? `, ${plural(tips, "suggestion")}` : "") : "No build timed yet", { class: "ap-line" });
    const body = el(box, "div", null, { class: "bt-body" });
    el(body, "p", `To show where a build is slow, Foundry times each one on this device: when you press Enter, when the model writes, when the preview appears and when the check ends. The latest ${model.keepLimit} are kept here and sent nowhere.`);
    if (r.switchTo) el(body, "p", SWITCH_WORDS(r.switchTo.from, r.switchTo.to, r.switchTo.fromRate, r.switchTo.toRate));
    for (const s of r.suggestions) {
      const k = r.kpis.find((x) => x.name === s.kpi), card = el(body, "div", null, { class: "bt-card" });
      el(card, "p", null).append(Object.assign(document.createElement("b"), { textContent: k.words.label }), ` was slower than its limit in ${k.observation.over} of ${k.observation.turns} builds, on average ${fmt(k, k.mean)}.`);
      el(card, "p", LEVER_WORDS[s.lever]);
      if (k.loop.applied) el(card, "p", `You made this change. Since then: ${({ improved: "faster", worse: "slower", unchanged: "about the same" })[k.assessment] ?? "no build yet"}.`);
      const row = el(card, "div");
      const adopt = el(row, "button", k.loop.applied ? "Change made" : "I made this change");
      adopt.disabled = !!k.loop.applied;
      adopt.onclick = async () => { await telemetry.adopt(s.kpi); draw(); };
      el(row, "button", "Not now").onclick = async () => { await telemetry.dismiss(s.kpi); draw(); };
    }
    const shown = r.kpis.filter((k) => k.observation.turns > 0 && (k.budget != null || ["think", "lead", "write"].includes(k.name)));
    if (shown.length) {
      const table = el(el(body, "div", null, { class: "bt-table" }), "table");
      const head = el(el(table, "thead"), "tr");
      for (const h of ["", "Latest", "Average", "Slowest", "Limit"]) el(head, "th", h);
      const rows = el(table, "tbody");
      for (const k of shown) {
        const tr = el(rows, "tr"), limit = k.budget == null ? "–" : k.time ? "under " + fmt(k, k.threshold) : `at most ${k.budget}`;
        [k.words.label, fmt(k, k.last), fmt(k, k.mean), fmt(k, k.observation.worst), limit].forEach((c, i) => el(tr, "td", c, i === 1 && k.overNow ? { class: "bt-over" } : null));
      }
      el(body, "p", "A time in red is over the limit the Foundry kernel sets.");
      const models = r.stats.filter((s) => s.reads > 0);
      if (models.length) {
        const mt = el(el(body, "div", null, { class: "bt-table" }), "table");
        const mh = el(el(mt, "thead"), "tr");
        for (const h of ["Model", "Apps", "At the first try", "After a repair", "Did not read"]) el(mh, "th", h);
        const mb = el(mt, "tbody");
        for (const s of models) {
          const mine = r.turns.filter((t) => t.modelIndex === s.model), sum = (key) => mine.reduce((a, t) => a + (t.kpis[key] ?? 0), 0);
          const tr = el(mb, "tr");
          [r.models[s.model], s.reads, `${sum("firstTry")} (${(model.firstTryPermille(s) / 10).toFixed(0)}%)`, sum("repaired"), sum("refused")].forEach((c) => el(tr, "td", String(c)));
        }
      }
    }
  }

  // The answers sit together at the right, the refusal first and the last button the quiet close.
  function finish() {
    const bar = el(dialog, "div");
    // Erasing asks twice inside this dialog, so the question is centred and in the app's look like the rest.
    const erase = el(bar, "button", "Erase everything");
    erase.onclick = async () => {
      if (erase.dataset.sure !== "yes") { erase.dataset.sure = "yes"; erase.textContent = "Erase for good?"; return; }
      await telemetry.erase(); draw();
    };
    const close = el(bar, "button", "Close");
    close.onclick = () => dialog.close();
    close.autofocus = true;
  }
  draw();
  if (!dialog.open) dialog.showModal();
  return { refresh: draw, close: () => dialog.close() };
}
