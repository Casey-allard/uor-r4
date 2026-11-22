// What the screen shows for what the user entered: shared by the live frame (bundled into
// shell/live/components.js), the card renderer (shell/live/card-render.mjs) and the published page (copied
// verbatim into the app's site/card.mjs). One copy, so the preview and the page cannot decide a screen
// differently. No imports: the page's card.mjs is this file, the interpreter and the card's data.
//
// The card is the app's screen rendered at publish from the same view as the frame, one variant per state
// a node can be in. The page picks a variant per node (keysFor), fills the two values no variant can hold
// in advance, a number typed and a guest's answer the rules disagree with (compose), and morphs the page
// to it, so focus and typed text stay where they are.

export const NUMBER = "@@V@@";
export const ANSWER = "@@A@@";
export const MISMATCH = "The verified app answered this; the rules drawn here say otherwise";
export const UNVERIFIED = "The app did not verify in this browser, so nothing is answered";

export function defaults(logic, old) {
  const values = {};
  for (const f of (logic && logic.fields) || []) {
    const v = old[f.name];
    if (f.kind === "choice") values[f.name] = f.values.includes(v) ? v : f.values[0];
    else { const n = Number(v); values[f.name] = v !== undefined && /^(0|[1-9][0-9]*)$/.test(v) && n >= f.min && (f.max === undefined || n <= f.max) ? v : String(f.min); }
  }
  return values;
}

export const requestOf = (logic, values) => logic.fields.map((f) => String(values[f.name])).join(" ");

/** The answer, the rule that decides it, and which examples pass, by the interpreter every app ships. */
export function evaluate(logic, values, { prismAnswer, prismDecide }) {
  const ready = !!(logic && logic.otherwise && logic.fields && logic.fields.length);
  if (!ready) return { ready, answer: null, fires: -1, request: "" };
  const spec = { fields: logic.fields, rules: logic.rules, otherwise: logic.otherwise };
  const request = requestOf(logic, values);
  const answer = prismDecide(spec, values);
  let fires = -1;
  if (answer !== "invalid") {
    fires = spec.rules.length;
    for (let i = 0; i < spec.rules.length; i++) if (prismAnswer({ ...spec, rules: spec.rules.slice(0, i + 1), otherwise: "\u0000" }, request) !== "\u0000") { fires = i; break; }
  }
  const passes = (req, res) => prismAnswer(spec, req) === res;
  return { ready, answer, fires, request, passes, rules: spec.rules.length };
}

export function stepValue(field, v, d) {
  const n = (parseInt(v, 10) || 0) + d;
  return String(Math.max(field.min, field.max === undefined ? n : Math.min(field.max, n)));
}

/** An example's request, filled into the fields it names, in order. */
export function fillValues(logic, request, values) {
  const parts = request.split(" "), patch = { ...values };
  (logic.fields || []).forEach((f, i) => { if (parts[i] !== undefined) patch[f.name] = parts[i]; });
  return patch;
}

// The variant of each node. `guest` is what the page's verified app said: {verified, answer, passes}, where
// passes maps each example's request to whether the guest answered it as the example says.
export function keysFor(card, values, guest, interp) {
  const ev = evaluate(card.logic, values, interp);
  let answer, lit;
  if (!guest.verified) { answer = "unverified"; lit = -1; }
  else if (guest.answer !== ev.answer) { answer = "mismatch"; lit = -1; }
  else { answer = ev.answer === "invalid" ? "invalid" : "f" + ev.fires; lit = ev.fires; }
  const keys = {};
  for (const n of card.nodes) {
    if (n.kind === "answer") keys[n.key] = answer;
    else if (n.kind === "rule" || n.kind === "otherwise") keys[n.key] = n.index === lit ? "on" : "off";
    else if (n.kind === "example") { const p = guest.passes.get(n.request); keys[n.key] = (p === true ? "1" : p === false ? "0" : "n") + (ev.request === n.request ? "1" : "0"); }
    else if (n.kind === "input") keys[n.key] = n.number ? "" : String(values[n.field]);
    else keys[n.key] = "";
  }
  return keys;
}

/** Which examples the app answers as they say: `ask(request)` is the app's answer, or throws. */
export async function examplesPass(card, ask) {
  const passes = new Map();
  for (const n of card.nodes) if (n.kind === "example") passes.set(n.request, (await Promise.resolve().then(() => ask(n.request)).catch(() => null)) === n.response);
  return passes;
}

// React's escaping, so a filled value is the bytes React would have written.
const escape = (s) => String(s).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#x27;" })[c]);

/** One node's markup for these keys. */
export function nodeHtml(card, node, key, values, guest) {
  const html = node.variants[key];
  if (html === undefined) throw new Error(`card: node ${node.key} has no variant ${key}`);
  return html.split(NUMBER).join(escape(node.number ? values[node.field] ?? "" : "")).split(ANSWER).join(escape(guest.answer ?? ""));
}

/** The whole screen's markup: what the page shows for these values and this guest. */
export function compose(card, values, guest, interp) {
  const keys = keysFor(card, values, guest, interp);
  return card.open + card.nodes.map((n) => nodeHtml(card, n, keys[n.key], values, guest)).join("") + card.close;
}

// ---- the page

// Makes `from` look like `to` in place: same elements kept, so focus, selection and typed text survive.
function morph(from, to) {
  // A new element where React would make one: another tag, or a keyed element whose key changed.
  if (from.nodeType !== to.nodeType || from.nodeName !== to.nodeName || (to.nodeType === 1 && from.getAttribute("data-remount") !== to.getAttribute("data-remount"))) { from.replaceWith(to.cloneNode(true)); return; }
  if (from.nodeType === 3) { if (from.data !== to.data) from.data = to.data; return; }
  if (from.nodeType !== 1) return;
  // `style` is the page's to keep: it is set from data-style, never written as markup.
  for (const a of [...from.attributes]) if (a.name !== "style" && !to.hasAttribute(a.name)) from.removeAttribute(a.name);
  for (const a of [...to.attributes]) if (from.getAttribute(a.name) !== a.value) from.setAttribute(a.name, a.value);
  if (from.tagName === "INPUT" && from.value !== (to.getAttribute("value") ?? "")) from.value = to.getAttribute("value") ?? "";
  if (from.tagName === "SELECT") from.value = to.querySelector("option[selected]")?.value ?? from.value;
  const a = [...from.childNodes], b = [...to.childNodes];
  for (let i = 0; i < b.length; i++) { if (a[i]) morph(a[i], b[i]); else from.appendChild(b[i].cloneNode(true)); }
  for (let i = b.length; i < a.length; i++) a[i].remove();
}

/** Starts the card on the page: `app` is the verified app (null when it did not verify). */
export async function start(card, interp, { app, show, $ }) {
  const main = document.querySelector(".prism-card > main");
  if (!main) return;
  const encoder = new TextEncoder();
  const ask = async (request) => show(await app.dispatch(encoder.encode(request)));
  let values = defaults(card.logic, {});
  const guest = { verified: !!app, answer: null, passes: new Map() };
  const out = $("out"), hist = $("hist");
  let seq = 0, last = "", settle = 0;
  async function draw() {
    const n = ++seq, request = requestOf(card.logic, values);
    const answer = app ? await ask(request).catch(() => null) : null;
    if (n !== seq) return;
    guest.answer = answer;
    const keys = keysFor(card, values, guest, interp);
    const nodes = [...main.children];
    card.nodes.forEach((node, i) => {
      const t = document.createElement("template");
      t.innerHTML = nodeHtml(card, node, keys[node.key], values, guest);
      if (nodes[i]) morph(nodes[i], t.content.firstChild);
    });
    styles();
    if (answer !== null) rail(request, answer);
  }
  // The page's own rail keeps what it keeps for every app (site/app.mjs, workspace): the verified app's
  // answer, styled by what app.json says it means, and the requests made, newest first.
  const data = (() => { try { return JSON.parse($("app-data").textContent); } catch { return {}; } })();
  const matches = (text, list) => (list ?? []).some((e) => (e.endsWith("*") ? text.startsWith(e.slice(0, -1)) : text === e));
  const state = (text) => matches(text, data.answers?.positive) ? "good" : text === data.answers?.refusal ? "refuse" : matches(text, data.answers?.negative) ? "bad" : "";
  function rail(request, answer) {
    const kind = state(answer);
    if (out) { out.textContent = kind === "refuse" ? `Not a valid ${(data.inputLabel || "request").toLowerCase()}` : answer.charAt(0).toUpperCase() + answer.slice(1); out.className = kind; }
    // Kept once the visitor pauses, as the page keeps a typed request.
    clearTimeout(settle);
    if (hist && request !== last) settle = setTimeout(() => remember(request, answer, kind), 900);
  }
  function remember(request, answer, kind) {
    last = request;
    const li = document.createElement("li"), b = document.createElement("button"), q = document.createElement("span"), tag = document.createElement("span");
    b.type = "button"; b.className = "again";
    q.textContent = request.length > 22 ? `${request.slice(0, 20)}…` : request;
    tag.className = `tag ${kind}`; tag.textContent = answer.length > 18 ? `${answer.length} bytes` : answer;
    b.append(q, tag); li.append(b); hist.prepend(li);
    b.addEventListener("click", () => set(fillValues(card.logic, request, values)));
    while (hist.children.length > 30) hist.lastChild.remove();
  }
  const set = (patch) => { values = { ...values, ...patch }; draw(); };
  const fieldOf = (el) => { const key = el.closest("[data-key]")?.dataset.key || ""; return key.startsWith("in:") ? card.logic.fields.find((f) => f.name === key.slice(3)) : null; };
  main.addEventListener("click", (e) => {
    const fill = e.target.closest("[data-fill]");
    if (fill) return set(fillValues(card.logic, fill.dataset.fill, values));
    const f = fieldOf(e.target);
    if (!f) return;
    const item = e.target.closest("[data-value]"), step = e.target.closest("[data-step]");
    if (item) set({ [f.name]: item.dataset.value });
    else if (step) set({ [f.name]: stepValue(f, values[f.name], Number(step.dataset.step)) });
  });
  // A choice's toggles are one radio group: arrows move along it and choose, as Radix does in the preview.
  main.addEventListener("keydown", (e) => {
    const item = e.target.closest("[data-value]"), f = item && fieldOf(item);
    if (!f || !/^Arrow(Left|Right|Up|Down)$/.test(e.key)) return;
    e.preventDefault();
    const i = f.values.indexOf(item.dataset.value), d = /Left|Up/.test(e.key) ? -1 : 1;
    const next = f.values[(i + d + f.values.length) % f.values.length];
    set({ [f.name]: next });
    queueMicrotask(() => item.parentElement.querySelector(`[data-value="${CSS.escape(next)}"]`)?.focus());
  });
  main.addEventListener("input", (e) => { const f = fieldOf(e.target); if (f && e.target.tagName === "INPUT") set({ [f.name]: e.target.value }); });
  main.addEventListener("change", (e) => { const f = fieldOf(e.target); if (f && e.target.tagName === "SELECT") set({ [f.name]: e.target.value }); });
  // Inline styles the view sets travel as data-style, since the page's policy refuses style attributes.
  const styles = () => { for (const el of main.querySelectorAll("[data-style]")) if (el.style.cssText !== el.dataset.style) el.style.cssText = el.dataset.style; };
  styles();
  if (app) guest.passes = await examplesPass(card, ask);
  // What loaded at rest stays at rest (card.css): pinned here before the card is marked started, so only
  // what the page mounts from now on enters as it does in the preview.
  for (const el of main.querySelectorAll(".animate-in")) el.style.setProperty("animation", "none");
  main.parentElement.setAttribute("data-started", "");
  await draw();
}
