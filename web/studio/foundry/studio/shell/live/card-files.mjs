// The app's own card for its published page: what Forge writes beside templateCommit's files, so the page
// draws what the live preview draws. The template's page (bootstrap/page.mjs) takes three optional files
// from an app and from nowhere else: site/card.html, whose surface slot it moves into the workspace;
// site/card.css, inlined and hashed into its policy; and site/card.mjs, which its app.mjs starts with the
// verified app. They are the app's, not the template's, so no file the template pins changes.
//
// card.html is the view (live-src/src/view.jsx) rendered for the app's default state; card.mjs is the card
// runtime (card-runtime.mjs, less its comment lines), the interpreter every app ships (for which rule
// decides, never for the answer) and the card's variants; card.css is the frame's sheet kept inside the
// card (tools/live_components.mjs), less the rules this card never uses. tools/published_preview.mjs holds
// the three to the view.
import { project } from "./project.mjs";
import { INTERPRETER, prismAnswer, prismDecide } from "../prism.js";

const interp = { prismAnswer, prismDecide };

// The template's page has 120 KB for everything it loads (its lighthouserc.json), so the card ships only
// the rules of the components' sheet its own markup can match: a style rule stays when one of its
// selectors starts with a class the card's markup has, or starts with no class at all.
const firstClass = (selector) => {
  const m = /^\s*\.((?:\\.|[\w-])+)/.exec(selector);
  return m ? m[1].replace(/\\(.)/g, "$1") : null;
};
export function purgeSheet(css, used) {
  const keep = (selector) => selector.split(",").some((part) => { const c = firstClass(part); return c === null || used.has(c); });
  let i = 0;
  // The text up to the brace that closes the current block, with every style rule no selector of which can match dropped.
  const block = () => {
    let body = "";
    while (i < css.length) {
      if (css.startsWith("/*", i)) { const j = css.indexOf("*/", i) + 2; body += css.slice(i, j); i = j; continue; }
      const open = css.indexOf("{", i), semi = css.indexOf(";", i), close = css.indexOf("}", i);
      const next = Math.min(...[open, semi, close].map((n) => (n < 0 ? Infinity : n)));
      if (next === Infinity) { body += css.slice(i); i = css.length; break; }
      if (next === close) { body += css.slice(i, close); i = close + 1; return body; }
      if (next === semi) { body += css.slice(i, semi + 1); i = semi + 1; continue; }
      const head = css.slice(i, open);
      i = open + 1;
      const inner = block();
      if (head.trim().startsWith("@")) { if (inner.trim() || !/^@(media|supports|layer|container)/.test(head.trim())) body += `${head}{${inner}}`; }
      else if (keep(head)) body += `${head}{${inner}}`;
    }
    return body;
  };
  return block();
}

const unescape = (s) => s.replace(/&amp;/g, "&").replace(/&gt;/g, ">").replace(/&lt;/g, "<").replace(/&quot;/g, '"').replace(/&#x27;/g, "'");
/** Every class the card's markup can carry: its variants' and its main's, and the card's own and the page's dark. */
export function classesOf(card) {
  const used = new Set(["prism-card", "dark"]);
  const markup = [card.open, card.close, ...card.nodes.flatMap((n) => Object.values(n.variants))];
  for (const html of markup) for (const m of html.matchAll(/class="([^"]*)"/g)) for (const c of unescape(m[1]).split(/\s+/)) if (c) used.add(c);
  return used;
}

/** `load(path)` gives the text of a file under shell/. */
export async function cardFiles(spec, { load }) {
  const [{ renderCard }, runtime, runtimeText, css] = await Promise.all([import("./card-render.mjs"), import("./card-runtime.mjs"), load("live/card-runtime.mjs"), load("live/card.css")]);
  const projection = project(spec);
  const card = renderCard(projection, interp);
  const values = runtime.defaults(card.logic, {});
  // The default state as the page first draws it, before the verified app has answered: as the rules say,
  // which is how the app answers every acceptance vector. card.mjs then asks the app and redraws.
  const guest = { verified: true, answer: prismDecide(card.logic, values), passes: await runtime.examplesPass(card, (request) => prismAnswer(card.logic, request)) };
  const html = `<div data-slot="surface" class="prism-card">${runtime.compose(card, values, guest, interp)}</div>\n`;
  const code = runtimeText.replace(/\r\n/g, "\n").replace(/^[ \t]*\/\/.*\n/gm, "");
  const mjs = `// ${projection.title || spec.name}: this app's card, written by Forge from its spec (shell/live/card-runtime.mjs). The answer shown is the verified app's.\n` +
    `${INTERPRETER}const CARD = ${JSON.stringify(card)};\n${code}\nexport default (context) => start(CARD, { prismAnswer, prismDecide }, context);\n`;
  const sheet = purgeSheet(css.replace(/\r\n/g, "\n"), classesOf(card));
  return { card, files: [{ path: "site/card.html", content: html }, { path: "site/card.css", content: sheet }, { path: "site/card.mjs", content: mjs }] };
}
