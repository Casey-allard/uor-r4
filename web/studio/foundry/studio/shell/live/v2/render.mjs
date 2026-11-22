// Draws a PrismPM browser presentation (Foundation.View.Browser.V1.Model::Presentation, as the
// interpreter holds it) into the DOM, and turns a pressed action into the field values it binds.
// It knows the protocol, not the app.
//
// Covers the content kinds listed in KINDS. Any other kind is refused by name, never skipped.
import { toArray } from "./interp.mjs";

export class RenderError extends Error {
  constructor(message) { super(message); this.code = "render.unsupported"; }
}

const MODEL = "Foundation.View.Browser.V1.Model::";
const PREFIX = MODEL + "Content.";
const LIVE = ["off", "polite", "assertive"];
const field = (label, enabled, required) => ({ label, disabled: !enabled, required });

// kind -> what to draw, from the constructor's arguments in the order Wire.cddl gives them.
const KINDS = {
  Section: ([label], words) => ({ tag: "section", aria: words(label) }),
  Navigation: ([label], words) => ({ tag: "nav", aria: words(label) }),
  Form: ([label], words) => ({ tag: "form", aria: words(label) }),
  Heading: ([level, label], words) => ({ tag: "h" + level, text: words(label) }),
  Text: ([text]) => ({ tag: "p", text }),
  Action: ([label, id, enabled, isDefault, fields], words) => ({ tag: "button", text: words(label), action: String(id), disabled: !enabled, type: isDefault ? "submit" : "button", binds: toArray(fields).map(Number) }),
  Input: ([label, enabled, required, maximum, value, epoch], words) => ({ tag: "label", control: "input", ...field(words(label), enabled, required), maximum: Number(maximum), value, epoch: String(epoch) }),
  Multiline: ([label, enabled, required, maximum, value, epoch], words) => ({ tag: "label", control: "textarea", ...field(words(label), enabled, required), maximum: Number(maximum), value, epoch: String(epoch) }),
  Select: ([label, enabled, required, selected, choices, epoch], words) => ({ tag: "label", control: "select", ...field(words(label), enabled, required), selected: String(selected), epoch: String(epoch), options: toArray(choices).map((c) => [String(c.id), words(c.label)]) }),
  TextSelect: ([label, enabled, required, selected, choices, epoch], words) => ({ tag: "label", control: "select", ...field(words(label), enabled, required), selected: String(selected), epoch: String(epoch), options: toArray(choices).map((c) => [String(c.id), c.text]) }),
  Table: ([caption, columns, chunks], words) => ({ tag: "table", caption: words(caption), columns: toArray(columns).map(words), rows: toArray(chunks).flatMap((chunk) => toArray(chunk.rows).map((row) => toArray(row.cells))) }),
};

export function describe(presentation, catalogue) {
  const words = (index) => { const w = catalogue[Number(index)]; if (w === undefined) throw new RenderError("label outside the catalogue: " + index); return w; };
  return toArray(presentation.nodes).map((node) => {
    if (!node.content.c.startsWith(PREFIX)) throw new RenderError("not a content value: " + node.content.c);
    const kind = node.content.c.slice(PREFIX.length), make = KINDS[kind];
    if (!make) throw new RenderError("content kind not drawn yet: " + kind);
    return { parent: Number(node.parent), kind, ...make(node.content.a, words) };
  });
}

const el = (tag, parent) => { const e = document.createElement(tag); parent.append(e); return e; };

function build(n, parent) {
  const e = el(n.tag, parent);
  if (n.control) { el("span", e); const control = el(n.control, e); if (n.control === "input") control.type = "text"; }
  if (n.tag === "table") { el("caption", e); el("tr", el("thead", e)); el("tbody", e); }
  return e;
}

// A field keeps what the person typed until the model changes the value or moves the draft epoch.
function setValue(control, value, epoch) { const key = epoch + ":" + value; if (control.dataset.model !== key) { control.value = value; control.dataset.model = key; } }

function update(e, n) {
  if (n.text !== undefined && e.textContent !== n.text) e.textContent = n.text;
  if (n.aria !== undefined) e.setAttribute("aria-label", n.aria);
  if (n.action !== undefined) { e.dataset.action = n.action; e.type = n.type; e.disabled = n.disabled; }
  if (n.control) {
    const [span, control] = e.children;
    if (span.textContent !== n.label) span.textContent = n.label;
    control.disabled = n.disabled; control.required = n.required;
    if (n.options) {
      const key = JSON.stringify(n.options);
      if (control.dataset.options !== key) {
        control.textContent = "";
        const none = el("option", control); none.value = "0"; none.textContent = "";
        for (const [id, text] of n.options) { const o = el("option", control); o.value = id; o.textContent = text; }
        control.dataset.options = key; delete control.dataset.model;
      }
      setValue(control, n.selected, n.epoch);
    } else { control.maxLength = n.maximum; setValue(control, n.value, n.epoch); }
  }
  if (n.tag === "table") {
    const [caption, head, body] = e.children;
    if (caption.textContent !== n.caption) caption.textContent = n.caption;
    const key = JSON.stringify([n.columns, n.rows]);
    if (e.dataset.cells !== key) {
      head.firstChild.textContent = ""; for (const c of n.columns) { const th = el("th", head.firstChild); th.scope = "col"; th.textContent = c; }
      body.textContent = ""; for (const row of n.rows) { const tr = el("tr", body); for (const cell of row) el("td", tr).textContent = cell; }
      e.dataset.cells = key;
    }
  }
}

// `onAction(action, fields)`: fields are the values the action binds, as [{ node, text } | { node, choice }].
// Same shape as the last frame: change only words and flags, so focus, drafts and scroll survive.
// A different shape: rebuild.
export function createView(root, onAction) {
  let shape = "", elements = [], nodes = [];
  const press = (button) => {
    if (!button || button.disabled) return;
    const node = nodes[elements.indexOf(button)];
    // A button this view did not draw (the root was handed to a newer view) is not this view's to answer.
    if (!node) return;
    const fields = node.binds.map((id) => {
      const target = nodes[id - 1], control = elements[id - 1].children[1];
      return target.control === "select" ? { node: BigInt(id), choice: BigInt(control.value) } : { node: BigInt(id), text: control.value };
    });
    onAction(BigInt(node.action), fields);
  };
  root.addEventListener("submit", (e) => { e.preventDefault(); press(e.submitter ?? e.target.querySelector("button[type=submit]")); });
  root.addEventListener("click", (e) => press(e.target.closest("button[type=button][data-action]")));
  return function draw(presentation, catalogue) {
    nodes = describe(presentation, catalogue);
    const next = nodes.map((n) => n.parent + n.tag + (n.control ?? "")).join();
    if (next !== shape) {
      root.textContent = ""; elements = [];
      for (const n of nodes) elements.push(build(n, n.parent === 0 ? root : elements[n.parent - 1]));
      shape = next;
    }
    nodes.forEach((n, i) => update(elements[i], n));
    root.setAttribute("aria-live", LIVE[Number(presentation.live)] ?? "off");
  };
}

// The field values of an intent, as the model's own constructors.
export const fieldValue = (f) => ({ node: f.node, value: f.text !== undefined ? { c: MODEL + "Value.Text", a: [f.text] } : { c: MODEL + "Value.Choice", a: [f.choice] } });
