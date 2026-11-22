// The app notation: a few plain lines a model writes for an app that keeps state, and the canonical LexLean
// module they stand for. The module is the app: it is what Forge stores, runs, checks and has Lean prove.
// The notation is only how a model says it briefly; `expand` is a pure, total function of its text, so the
// same lines are always the same module, byte for byte.
//
//   app Guest list
//   text name "Name" max 40
//   choice meal "Meal": "Meat", "Fish", "Vegetarian"
//   table guests "Guests": name, meal
//   count guests "Coming"
//   action "Add": add to guests
//   action "Clear": clear guests
//
// Each table holds at most 16 rows, newest first. An add is refused, and changes nothing, unless every text
// in its row is 1 to max bytes, every choice is made, and the table has room. Fields empty after a press.

const M = "Foundation.View.Browser.V1.Model";
const RESERVED = new Set(["id", "decide", "take", "map", "length", "some", "none", "max", "min", "not", "and", "or", "state", "intent", "revision", "catalogue", "initial", "view", "step", "promises", "textOf", "choiceOf", "guarded", "seen", "pressed", "fields", "node", "head", "rest", "words", "chosen", "app", "text", "choice", "table", "count", "action", "form", "add", "clear", "to"]);
const ROWS = 16;

const str = (s) => '"' + s + '"';
const list = (items, type) => items.reduceRight((tail, item) => `(cons ${item} ${tail})`, `(nil ${type})`);
const cap = (s) => s[0].toUpperCase() + s.slice(1);
const bytes = (name) => `(prim length (prim utf8_encode ${name} : Bytes) : Nat)`;
const all = (terms) => (terms.length ? terms.reduce((a, b) => `(${a} && ${b})`) : "true");
const definition = (name, parameters, result, body, extra = "") => `definition{\n  name: ${name},\n  parameters: [${parameters.join(", ")}],\n  result: ${result},\n  body: ${body}${extra}\n}`;
const sorted = (names) => [...new Set(names)].sort((a, b) => (a < b ? -1 : a > b ? 1 : 0));

/** Whether a turn's block is written in the notation rather than as a module. */
export function isNotation(text) {
  const first = String(text || "").split("\n").map((l) => l.trim()).find((l) => l);
  return !!first && /^app\s+\S/.test(first) && !String(text).includes("\\begin{");
}

/** The lines of a notation, read. `lenient` drops a last line that does not read yet (a turn still streaming). */
export function parse(text, { lenient = false } = {}) {
  const issues = [], app = { title: "", form: "", fields: [], tables: [], counts: [], actions: [] };
  const lines = String(text || "").split("\n");
  const names = new Set();
  const quoted = '"([^"\\\\]{1,60})"';
  const name = (n, at) => {
    if (!/^[a-z][A-Za-z0-9]{0,23}$/.test(n)) return issues.push({ line: at, message: `"${n}" is not a name: use a lowercase letter, then letters and digits` }), false;
    if (RESERVED.has(n)) return issues.push({ line: at, message: `"${n}" is a reserved word: choose another name` }), false;
    if (names.has(n)) return issues.push({ line: at, message: `"${n}" is already used: every name is used once` }), false;
    return names.add(n), true;
  };
  lines.forEach((raw, i) => {
    const line = raw.trim(), at = i + 1, before = issues.length;
    if (!line) return;
    let m;
    if ((m = /^app\s+(.{1,60})$/.exec(line))) {
      if (app.title) issues.push({ line: at, message: "the app is named twice" });
      else if (/["\\]/.test(m[1])) issues.push({ line: at, message: "the app's name holds no quote and no backslash" });
      else app.title = m[1].trim();
    } else if ((m = new RegExp(`^form\\s+${quoted}$`).exec(line))) app.form = m[1];
    else if ((m = new RegExp(`^text\\s+(\\S+)\\s+${quoted}(?:\\s+max\\s+(\\d{1,3}))?$`).exec(line))) {
      const max = m[3] ? Number(m[3]) : 40;
      if (max < 1 || max > 200) issues.push({ line: at, message: "max is between 1 and 200" });
      else if (name(m[1], at)) app.fields.push({ kind: "text", name: m[1], label: m[2], max });
    } else if ((m = new RegExp(`^choice\\s+(\\S+)\\s+${quoted}\\s*:\\s*(.+)$`).exec(line))) {
      const options = [...m[3].matchAll(new RegExp(quoted, "g"))].map((o) => o[1]);
      if (m[3].replace(new RegExp(quoted, "g"), "").replace(/[\s,]/g, "")) issues.push({ line: at, message: 'the options are quoted and separated by commas, as in "Meat", "Fish"' });
      else if (options.length < 2 || options.length > 8) issues.push({ line: at, message: "a choice has between 2 and 8 options" });
      else if (name(m[1], at)) app.fields.push({ kind: "choice", name: m[1], label: m[2], options });
    } else if ((m = new RegExp(`^table\\s+(\\S+)\\s+${quoted}\\s*:\\s*(.+)$`).exec(line))) {
      const columns = m[3].split(",").map((c) => c.trim());
      const unknown = columns.find((c) => !app.fields.some((f) => f.name === c));
      if (unknown) issues.push({ line: at, message: `"${unknown}" is not a text or a choice declared above this table` });
      else if (columns.some(c => !c) || new Set(columns).size !== columns.length || columns.length > 6) issues.push({ line: at, message: "a table has between 1 and 6 columns, each field once" });
      else if (app.tables.some(t => t.name + "Count" === m[1] || m[1] + "Count" === t.name)) issues.push({ line: at, message: "the table name collides with a generated table count: choose another name" });
      else if (name(m[1], at)) app.tables.push({ name: m[1], caption: m[2], columns });
    } else if ((m = new RegExp(`^count\\s+(\\S+)\\s+${quoted}$`).exec(line))) {
      if (!app.tables.some((t) => t.name === m[1])) issues.push({ line: at, message: `"${m[1]}" is not a table declared above this count` });
      else app.counts.push({ table: m[1], label: m[2] });
    } else if ((m = new RegExp(`^action\\s+${quoted}\\s*:\\s*(add\\s+to|clear)\\s+(\\S+)$`).exec(line))) {
      if (!app.tables.some((t) => t.name === m[3])) issues.push({ line: at, message: `"${m[3]}" is not a table declared above this action` });
      else if (app.actions.length >= 6) issues.push({ line: at, message: "an app has up to 6 actions" });
      else app.actions.push({ label: m[1], kind: m[2] === "clear" ? "clear" : "add", table: m[3] });
    } else issues.push({ line: at, message: 'this line is not one of: app NAME | form "Label" | text name "Label" max N | choice name "Label": "A", "B" | table name "Caption": field, field | count table "Label" | action "Label": add to table | action "Label": clear table' });
    if (lenient && i === lines.length - 1 && issues.length > before) issues.length = before;
  });
  if (!app.title) issues.unshift({ line: 1, message: "the first line names the app: app NAME" });
  if (!lenient) {
    if (!app.tables.length) issues.push({ line: lines.length, message: "the app has no table: it keeps nothing" });
    for (const t of app.tables) if (!app.actions.some((a) => a.kind === "add" && a.table === t.name)) issues.push({ line: lines.length, message: `nothing adds to the table "${t.name}": give it an action "Label": add to ${t.name}` });
  }
  return { app, issues };
}

/** The canonical LexLean module a notation stands for, and the promises it makes with their theorems. */
export function expand(text, options = {}) {
  const { app, issues } = parse(text, options);
  if (issues.length || !app.title) return { ok: false, issues, module: "" };
  const labels = [];
  const label = (s) => labels.push(s) - 1;
  const titleLabel = label(app.title), formLabel = label(app.form || "New entry");
  const fields = app.fields.map((f) => ({ ...f, at: label(f.label), options: f.options && f.options.map((o) => ({ text: o, at: label(o) })), param: f.name + (f.kind === "text" ? "Text" : "Choice") }));
  const actions = app.actions.map((a, i) => ({ ...a, id: i + 1, at: label(a.label), fn: (a.kind === "add" ? "added" : "cleared") + (i + 1) }));
  const counts = app.counts.map((c) => ({ ...c, at: label(c.label) }));
  const tables = app.tables.map((t) => ({ ...t, at: label(t.caption), count: t.name + "Count", columns: t.columns.map((c) => fields.find((f) => f.name === c)) }));

  // the screen: node numbers are positions in this list, from 1
  const nodes = [];
  const node = (parent, content) => nodes.push(`(record ${M}::Node {parent := ${parent}, content := (ctor ${M}::Content.${content})})`);
  node(0, `Section ${titleLabel}`); node(1, `Heading 1 ${titleLabel}`);
  const form = fields.length || actions.length ? node(1, `Form ${formLabel}`) : 0;
  for (const f of fields) {
    f.node = f.kind === "text"
      ? node(form, `Input ${f.at} true true ${f.max} "" (project state revision)`)
      : node(form, `Select ${f.at} true true 0 ${list(f.options.map((o, i) => `(record ${M}::Choice {id := ${i + 1}, label := ${o.at}})`), `${M}::Choice`)} (project state revision)`);
  }
  let hasDefault = false;
  for (const a of actions) {
    const table = tables.find((t) => t.name === a.table);
    a.sends = a.kind === "add" ? table.columns.map((f) => f.node).sort((x, y) => x - y) : [];
    const isDefault = a.kind === "add" && !hasDefault; hasDefault = hasDefault || isDefault;
    node(form, `Action ${a.at} ${a.id} true ${isDefault} ${list(a.sends.map(String), "Nat")}`);
  }
  for (const c of counts) { node(1, `Heading 2 ${c.at}`); node(1, `Text (countText (project state ${tables.find((t) => t.name === c.table).count}))`); }
  for (const t of tables) node(1, `Table ${t.at} ${list(t.columns.map((f) => String(f.at)), "Nat")} (match (project state ${t.name})\n    | List.nil => (nil ${M}::TableChunk)\n    | List.cons head rest => (cons (record ${M}::TableChunk {rows := (cons head rest)}) (nil ${M}::TableChunk)))`);

  const stateFields = tables.flatMap((t) => [[t.name, `(List ${M}::TableRow)`], [t.count, "Nat"]]);
  const record = (changes) => `(record State {revision := ((project state revision) + 1)${stateFields.map(([n]) => `, ${n} := ${changes[n] || `(project state ${n})`}`).join("")}})`;
  const out = [];
  out.push(`structure{\n  name: State,\n  type_parameters: [],\n  parameters: [],\n  fields: [\n    {name: revision, type: Nat}${stateFields.map(([n, t]) => `,\n    {name: ${n}, type: ${t}}`).join("")}\n  ]\n}`);
  out.push(definition("catalogue", [], "(List String)", list(labels.map(str), "String")));
  out.push(definition("initial", [], "State", `(record State {revision := 0${stateFields.map(([n, t]) => `, ${n} := ${t === "Nat" ? "0" : `(nil ${M}::TableRow)`}`).join("")}})`));
  out.push(definition("view", ["state : State"], `${M}::Presentation`, `(record ${M}::Presentation {revision := (project state revision), phase := 0, status := 0, live := 1, focus := 0, nodes := ${list(nodes, `${M}::Node`)}})`));

  // A count is at most 16, so its text is one of seventeen literals: the language formats no numbers.
  if (counts.length) out.splice(3, 0, definition("countText", ["rows : Nat"], "String", Array.from({ length: ROWS }, (_, i) => i).reduceRight((rest, i) => `(if (rows =? ${i}) then "${i}" else ${rest})`, `"${ROWS}"`)));
  const texts = fields.filter((f) => f.kind === "text"), choices = fields.filter((f) => f.kind === "choice");
  if (texts.length) out.push(definition("textOf", [`fields : (List ${M}::FieldValue)`, "node : Nat"], "String", `(match fields\n    | List.nil => ""\n    | List.cons head rest => (if ((project head node) =? node) then (match (project head value)\n      | ${M}::Value.Text words => words\n      | ${M}::Value.Choice chosen => "") else (textOf rest node)))`, ",\n  recursive_argument: fields"));
  if (choices.length) out.push(definition("choiceOf", [`fields : (List ${M}::FieldValue)`, "node : Nat"], "Nat", `(match fields\n    | List.nil => 0\n    | List.cons head rest => (if ((project head node) =? node) then (match (project head value)\n      | ${M}::Value.Text words => 0\n      | ${M}::Value.Choice chosen => chosen) else (choiceOf rest node)))`, ",\n  recursive_argument: fields"));
  for (const f of texts) out.push(definition(`${f.name}Fits`, ["words : String"], "Bool", `((1 <=? ${bytes("words")}) && (${bytes("words")} <=? ${f.max}))`));
  for (const f of choices) {
    out.push(definition(`${f.name}Made`, ["chosen : Nat"], "Bool", `((1 <=? chosen) && (chosen <=? ${f.options.length}))`));
    out.push(definition(`${f.name}Label`, ["chosen : Nat"], "String", f.options.slice(0, -1).reduceRight((rest, o, i) => `(if (chosen =? ${i + 1}) then ${str(o.text)} else ${rest})`, str(f.options[f.options.length - 1].text))));
  }
  const params = fields.map((f) => `${f.param} : ${f.kind === "text" ? "String" : "Nat"}`), args = fields.map((f) => f.param).join(" ");
  const call = (fn, a, state = "state") => `(${fn} ${state}${a.kind === "add" && args ? " " + args : ""})`;
  for (const a of actions) {
    const t = tables.find((x) => x.name === a.table);
    if (a.kind === "clear") { out.push(definition(a.fn, ["state : State"], "State", record({ [t.name]: `(nil ${M}::TableRow)`, [t.count]: "0" }))); continue; }
    a.guards = [...t.columns.map((f) => (f.kind === "text" ? `(${f.name}Fits ${f.param})` : `(${f.name}Made ${f.param})`)), `((project state ${t.count}) <? ${ROWS})`];
    const row = `(record ${M}::TableRow {cells := ${list(t.columns.map((f) => (f.kind === "text" ? f.param : `(${f.name}Label ${f.param})`)), "String")}})`;
    out.push(definition(a.fn, ["state : State", ...params], "State", `(if ${all(a.guards)} then ${record({ [t.name]: `(cons ${row} (project state ${t.name}))`, [t.count]: `((project state ${t.count}) + 1)` })} else state)`));
  }
  const guardedParams = ["state : State", "seen : Nat", "pressed : Nat", ...params];
  out.push(definition("guarded", guardedParams, "State", `(if (seen =? (project state revision)) then ${actions.reduceRight((rest, a) => `(if (pressed =? ${a.id}) then ${call(a.fn, a)} else ${rest})`, "state")} else state)`));
  const guardedCall = `(guarded state seen pressed${args ? " " + args : ""})`;

  // the promises, each with the theorem that states it
  const promises = [], theorems = [];
  const promise = (sentence, theorem) => { promises.push(sentence); theorems.push(theorem); };
  const theorem = (name, parameters, statement, proof, axioms) => `theorem{\n  name: ${name},\n  parameters: [${parameters.join(", ")}],\n  statement: ${statement},\n  proof: ${proof},\n  axioms: [${axioms}]\n}`;
  const arithmetic = (names) => `linear_arithmetic{definitions: [${sorted(names).join(", ")}]}`;
  const helpers = [...texts.map((f) => `${f.name}Fits`), ...choices.map((f) => `${f.name}Made`)];
  for (const t of tables) promise(`${t.caption} starts empty.`, theorem(`${t.name}StartsEmpty`, [], `((project (initial) ${t.name}) = (nil ${M}::TableRow))`, "reflexivity{}", ""));
  for (const a of actions) {
    const t = tables.find((x) => x.name === a.table);
    if (a.kind === "clear") { promise(`Pressing ${a.label} always leaves ${t.caption} empty.`, theorem(`${a.fn}Empties`, ["state : State"], `((project (${a.fn} state) ${t.name}) = (nil ${M}::TableRow))`, "reflexivity{}", "")); continue; }
    const moved = `((project state revision) <? (project ${call(a.fn, a)} revision))`;
    for (const f of t.columns) {
      const breaks = `breaks${cap(a.fn)}${cap(f.name)}`;
      const bad = f.kind === "text" ? `((${bytes(f.param)} =? 0) || (${f.max + 1} <=? ${bytes(f.param)}))` : `((${f.param} =? 0) || (${f.options.length + 1} <=? ${f.param}))`;
      out.push(definition(breaks, ["state : State", ...params], "Bool", `(${bad} && ${moved})`));
      promise(f.kind === "text" ? `Pressing ${a.label} with ${f.label} empty or longer than ${f.max} bytes changes nothing.` : `Pressing ${a.label} with no ${f.label} chosen changes nothing.`,
        theorem(`${a.fn}Needs${cap(f.name)}`, ["state : State", ...params], `((${breaks} state${args ? " " + args : ""}) = false)`, arithmetic([...helpers, a.fn, breaks]), "Quot.sound, propext"));
    }
    const full = `breaks${cap(a.fn)}Full`;
    out.push(definition(full, ["state : State", ...params], "Bool", `((${ROWS} <=? (project state ${t.count})) && ${moved})`));
    promise(`Pressing ${a.label} when ${t.caption} already holds ${ROWS} rows changes nothing.`, theorem(`${a.fn}StopsWhenFull`, ["state : State", ...params], `((${full} state${args ? " " + args : ""}) = false)`, arithmetic([...helpers, a.fn, full]), "Quot.sound, propext"));
  }
  out.push(definition("breaksStale", guardedParams, "Bool", `(((seen <? (project state revision)) || ((project state revision) <? seen)) && (((project ${guardedCall} revision) <? (project state revision)) || ((project state revision) <? (project ${guardedCall} revision))))`));
  promise("A press from an out-of-date screen never changes anything.", theorem("staleIgnored", guardedParams, `((breaksStale state seen pressed${args ? " " + args : ""}) = false)`, arithmetic([...helpers, ...actions.map((a) => a.fn), "breaksStale", "guarded"]), "Quot.sound, propext"));

  const stepArgs = fields.map((f) => `(${f.kind === "text" ? "textOf" : "choiceOf"} (project intent fields) ${f.node})`).join(" ");
  out.push(definition("step", ["state : State", `intent : ${M}::Intent`], "State", `(guarded state (project intent revision) (project intent action)${stepArgs ? " " + stepArgs : ""})`));
  out.push(definition("promises", [], "(List String)", list(promises.map(str), "String")));
  const module = `\\begin{lexlean}{App}\n\\useglossary{lexlean.std.bool@1.2.0}\n\\useglossary{lexlean.std.nat@1.2.0}\n\\useglossary{prism.arch@1.0.0}\n\\importmodule{${M}}\n\\title{Browser view model}\n\\begin{semanticmodule}\n${[...out, ...theorems].join("\n\n")}\n\\end{semanticmodule}\n\\end{lexlean}\n`;
  return { ok: true, issues: [], module, promises };
}

/** What a model is told about the notation. */
export const PROMPT = `Write the app in the app notation, inside the <prism-lexlean> block and nothing else in it. Forge turns these lines into the app's LexLean module, runs it in the preview as you write, and has Lean prove what it promises. Write the lines at once: no plan, no explanation before them.

The lines, in this order:
app NAME
form "Label"                                  (optional: the form's heading)
text name "Label" max N                       (a text box; N is its longest length in bytes, 1 to 200)
choice name "Label": "Option", "Option"       (a pick list of 2 to 8 options)
table name "Caption": field, field            (rows the app keeps; each column is a text or a choice above)
count table "Label"                           (optional: shows how many rows the table holds)
action "Label": add to table                  (adds the form's values as a new first row)
action "Label": clear table                   (empties the table)

A name is a lowercase letter, then letters and digits, used once. Labels hold no quote and no backslash. A table holds at most 16 rows; an add with an empty or too long text, no choice made, or a full table changes nothing. The fields empty after each press.

Example: <prism-lexlean>
app Guest list
form "New guest"
text name "Name" max 40
choice meal "Meal": "Meat", "Fish", "Vegetarian"
table guests "Guests": name, meal
count guests "Coming"
action "Add": add to guests
action "Clear": clear guests
</prism-lexlean>

If what is asked for needs something these lines cannot say, write the nearest app they can say, and say in one sentence after the block what is left out.`;
