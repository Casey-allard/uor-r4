// The Published gate: a published app draws what its live preview draws. The preview is the frame, React
// over shadcn/ui (shell/live/components.js); the published page is the template's page with the app's own
// card (site/card.html, card.css, card.mjs), which Forge writes beside templateCommit's files. Both come
// from one view (live-src/src/view.jsx): the card's markup is that view rendered at publish, one variant
// per state a node can be in, and card.mjs picks a variant per node from what the verified guest answers.
//
//   node tools/published_preview.mjs --check
//
// Claims (level build: validated here against an oracle, evidence not proof). On the RSVP fixture:
//   same screen   for every state checked (the defaults, every example, a grid over the fields, refused
//                 requests), the screen card.mjs composes from the card's variants is byte for byte the
//                 view rendered for that state, which is what the frame renders
//   guest decides the answer shown is the guest's: a guest that disagrees with the rules is shown as such,
//                 and no rule is lit; a page that did not verify shows no answer
//   card files    card.html is the default state in the surface slot, with no script and no style
//                 attribute (the page's policy hashes its styles); card.css keeps Tailwind's base layer
//                 inside the card and resets the page's own rules there; card.mjs is a module whose
//                 default export starts the card; the three fit a budget of 24 KB gzipped
//   one view      the frame and the card renderer are built from the same view.jsx (live_components --check)
import fs from "node:fs";
import path from "node:path";
import zlib from "node:zlib";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const at = (p) => pathToFileURL(path.join(root, p)).href;
const failures = [];
const fail = (claim, message) => failures.push(`${claim}: ${message}`);

const spec = JSON.parse(fs.readFileSync(path.join(root, "tools/fixtures/live/rsvp.json"), "utf8"));
const { project } = await import(at("shell/live/project.mjs"));
const prism = await import(at("shell/prism.js"));
const { cardFiles } = await import(at("shell/live/card-files.mjs"));
const runtime = await import(at("shell/live/card-runtime.mjs"));
const { renderScreen } = await import(at("shell/live/card-render.mjs"));
const interp = { prismAnswer: prism.prismAnswer, prismDecide: prism.prismDecide };
const load = async (p) => fs.readFileSync(path.join(root, "shell", p), "utf8");

const projection = project(spec);
const { files, card } = await cardFiles(spec, { load });
const file = (name) => files.find((f) => f.path === "site/" + name)?.content ?? "";

// A guest that is the rules: what the page shows when the verified app and the rules agree, as they do on
// every acceptance vector (the Rules and Live gates).
const agreeing = { verified: true, answer: (request) => prism.prismAnswer(spec, request) };

// ---- same screen
const logic = projection.logic;
const states = [runtime.defaults(logic, {})];
for (const e of spec.examples) states.push(runtime.fillValues(logic, e.request, runtime.defaults(logic, {})));
for (const attending of ["yes", "no", "maybe"]) for (const party of ["0", "1", "3", "6"]) for (const seats of ["0", "2", "5", "500"]) states.push({ attending, party, seats });
states.push({ attending: "yes", party: "7", seats: "2" }, { attending: "yes", party: "", seats: "2" }, { attending: "yes", party: "01", seats: "2" }, { attending: "yes", party: "2", seats: "<b>" });
let same = 0;
for (const values of states) {
  const want = renderScreen(projection, values, interp);
  const got = runtime.compose(card, values, await guestOf(agreeing, values), interp);
  if (got === want) same++;
  else fail("same screen", `state ${JSON.stringify(values)}: the card composes ${firstDiff(got, want)}`);
}
if (states.length < 50) fail("same screen", `only ${states.length} states checked`);

function unlayered(css) {
  const out = [];
  let depth = 0, from = 0;
  for (let i = 0; i < css.length; i++) {
    if (css[i] === "/" && css[i + 1] === "*") { i = css.indexOf("*/", i) + 1; from = i + 1; continue; }
    if (css[i] === "{") { if (depth === 0) { const sel = css.slice(from, i).trim(); if (!sel.startsWith("@")) out.push(sel); } depth++; from = i + 1; }
    else if (css[i] === "}") { depth--; from = i + 1; }
    else if (css[i] === ";" && depth === 0) from = i + 1;
  }
  return out;
}
// What the page knows of the guest, found as the page finds it (card-runtime.mjs examplesPass), from the
// card's own data: a card that lost what an example expects shows it failing here as it would on the page.
async function guestOf(guest, values) {
  const request = runtime.requestOf(logic, values);
  return { verified: guest.verified, answer: guest.verified ? guest.answer(request) : null, passes: guest.verified ? await runtime.examplesPass(card, guest.answer) : new Map() };
}
function firstDiff(a, b) {
  let i = 0;
  while (i < a.length && a[i] === b[i]) i++;
  return `${JSON.stringify(a.slice(Math.max(0, i - 60), i + 60))} where the view draws ${JSON.stringify(b.slice(Math.max(0, i - 60), i + 60))}`;
}

// ---- guest decides
{
  const values = runtime.defaults(logic, {});
  const liar = { verified: true, answer: () => "<none>" };
  const shown = runtime.compose(card, values, await guestOf(liar, values), interp);
  if (!shown.includes("&lt;none&gt;")) fail("guest decides", "an answer from the guest that the rules disagree with is not the one shown");
  if (!shown.includes(runtime.MISMATCH)) fail("guest decides", "a guest that disagrees with the rules is not said to");
  const keys = runtime.keysFor(card, values, await guestOf(liar, values), interp);
  if (Object.entries(keys).some(([k, v]) => (k.startsWith("rule:") || k === "otherwise") && v === "on")) fail("guest decides", "a rule is lit although the guest disagrees with the rules");
  const none = runtime.compose(card, values, await guestOf({ verified: false }, values), interp);
  if (!none.includes(runtime.UNVERIFIED)) fail("guest decides", "a page that did not verify does not say so");
  const answerNode = card.nodes.find((n) => n.kind === "answer");
  const answerShown = runtime.nodeHtml(card, answerNode, "unverified", values, await guestOf({ verified: false }, values));
  if (spec.outcomes.some((o) => answerShown.includes(`>${o}<`))) fail("guest decides", "a page that did not verify still shows an answer");
}

// ---- card files
{
  const html = file("card.html"), css = file("card.css"), mjs = file("card.mjs");
  if (!html || !css || !mjs) fail("card files", `Forge writes ${files.map((f) => f.path).join(", ") || "nothing"}`);
  const surface = /^<div data-slot="surface" class="prism-card[^"]*">([\s\S]*)<\/div>$/.exec(html.trim());
  if (!surface) fail("card files", "card.html is not one surface slot holding the card");
  else if (surface[1] !== runtime.compose(card, states[0], await guestOf(agreeing, states[0]), interp)) fail("card files", "card.html is not the default state");
  if (/<script/i.test(html)) fail("card files", "card.html has a script");
  if (/\sstyle=/i.test(html)) fail("card files", "card.html has a style attribute, which the page's policy refuses");
  const base = /@layer base\{([\s\S]*?)\}@layer components/.exec(css);
  if (!base) fail("card files", "card.css has no base layer");
  else for (const sel of base[1].match(/(^|[{}])[^{}@]+\{/g) || []) {
    const s = sel.replace(/^[{}]/, "").slice(0, -1);
    if (s.split(",").some((part) => !/^(\.prism-card|:where\(\.prism-card\))/.test(part.trim()))) { fail("card files", `card.css lets Tailwind's base rule \`${s}\` out of the card`); break; }
  }
  if (!/#stage \.prism-card \*[^{]*\{all:revert-layer\}/.test(css)) fail("card files", "card.css does not reset the page's own rules inside the card");
  if (!css.includes("#stage .prism-card:not([data-started]) .animate-in{animation:none}") || !mjs.includes('setAttribute("data-started"')) fail("card files", "the card fades in at load, so the page's first paint is not its largest (the template's LCP gate)");
  // Outside every layer only the card's own rules may stand: any other unlayered rule is undone inside the
  // card by its reset, so the card would not draw what the frame draws.
  for (const sel of unlayered(css)) if (!sel.startsWith("#stage ")) { fail("card files", `card.css has an unlayered rule \`${sel}\` the card's reset would undo`); break; }
  const mod = await import("data:text/javascript;base64," + Buffer.from(mjs).toString("base64")).catch((e) => fail("card files", `card.mjs does not load: ${e.message}`));
  if (mod && typeof mod.default !== "function") fail("card files", "card.mjs has no default export to start the card");
  const gz = [html, css, mjs].reduce((n, t) => n + zlib.gzipSync(t, { level: 9 }).length, 0);
  if (gz > 24 * 1024) fail("card files", `${gz} bytes gzipped, over the 24 KB budget`);
  console.log(`card files: card.html ${html.length}, card.css ${css.length}, card.mjs ${mjs.length} bytes; ${gz} gzipped`);
}

// ---- one view
for (const name of ["components.js", "card-render.mjs"]) {
  const body = fs.readFileSync(path.join(root, "shell/live", name), "utf8");
  if (!body.includes("data-fill")) fail("one view", `shell/live/${name} is not built from view.jsx`);
}

if (failures.length) {
  console.log(`published refused:\n${failures.join("\n")}`);
  process.exit(1);
}
console.log(`published: ${same} states, the card composes the view's screen byte for byte; the guest decides the answer shown; card files within budget`);
// React's server renderer keeps a message channel open; the gate is done.
process.exit(0);
