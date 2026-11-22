// Foundry's own facets, written by the builder's projector from what Foundry already declares.
//
// Foundry is one Prism model. Its behavior is the kernel and the protocols in app/src; the other
// facets (Architecture, Design, Features, Users, Security and its threat model, Quality, Compliance)
// are projected here by shell/prism/builder/facets.mjs, the same projector that writes the facets of
// every app the studio builds, so Foundry is checked exactly as its apps are.
//
// The front end drives the back end. A screen (product.json `surfaces`) names the back-end features
// it requires; its renderer code is walked (foundry_surfaces.mjs) and every IPC call it makes must
// reach a command-registry feature, or a host channel, that one of those back-end features owns. Each
// register ID is carried by exactly one back-end feature, each back-end feature is required by a
// screen, each screen is a step of a journey and each stakeholder has a journey, so nothing in the
// register, the registry or the architecture is unreached and nothing a screen does is unowned.
//
// Nothing is restated. Each record comes from one source:
//
//   app/product.json                     stakeholders, components, screens, journeys, threats, and
//                                         which kernel declarations realize each back-end feature
//   model/ids.toml                        the register IDs each feature carries
//   crates/conformance/tests/*.rs         the test that accepts each register ID
//   shell/command-registry-data.mjs       the registry features and the declaration owning each
//   vendor/prism/src                       what each screen calls (foundry_surfaces.mjs)
//   app/src/ForgeKernel.lex.tex           the build phases and the rules between them (the design's
//                                         state machine), and every budget and experience rule
//   app/src/FoundrySpecification.lex.tex  the product obligations (compliance)
//   tools/foundry-model/                  modules vendored byte for byte: Prism R4 (afflom/me) and
//                                         the context-of-use model (prism-devrel)
//   shell/prism/builder/assets.mjs        the shared Foundation and Production modules and the
//                                         prism.arch, prism.qual and prism.sec language packages
//
//   node tools/foundry_model.mjs           write the facet modules and lexlean.toml's entrypoints
//   node tools/foundry_model.mjs --check   the Model gate: refuse any byte that differs
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { facetModules } from "../shell/prism/builder/facets.mjs";
import { ASSETS } from "../shell/prism/builder/assets.mjs";
import { cons, ctor, definition, listDefinitions, moduleText, rec, str, theorem } from "../shell/prism/builder/lean.mjs";
import { ISO25010, MODEL_KINDS, STRIDE, STRIDE_PER_ELEMENT, STRIDE_TEXT, DFD_OF, LIKELIHOOD, IMPACT, COMPONENT_KINDS, EVIDENCE_KINDS } from "../shell/prism/builder/prompt.mjs";
import { commandRegistryData } from "../shell/command-registry-data.mjs";
import { surfaceCommands, SurfaceError } from "./foundry_surfaces.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.join(here, "..", "..");
const app = path.join(root, "app");
const read = (p) => fs.readFileSync(path.join(root, p), "utf8");

class ModelError extends Error {}
const refuse = (message) => {
  throw new ModelError(message);
};

const pascal = (s) =>
  String(s)
    .split(/[^A-Za-z0-9]+/)
    .filter(Boolean)
    .map((w) => w[0].toUpperCase() + w.slice(1))
    .join("") || "X";

/** Declaration names, unique within one module (the builder's rule). */
function namer() {
  const used = new Set();
  return (prefix, name) => {
    const base = `${prefix}${pascal(name)}`;
    let decl = base;
    for (let i = 2; used.has(decl); i++) decl = `${base}${i}`;
    used.add(decl);
    return decl;
  };
}

/** Index records by their position when sorted by declaration name, as the SDK numbers facet records. */
function indexed(records) {
  const sorted = [...records].sort((a, b) => (a.decl < b.decl ? -1 : a.decl > b.decl ? 1 : 0));
  sorted.forEach((r, i) => (r.index = i));
  return sorted;
}

const find = (records, key, what, field = "name") => records.find((r) => r[field] === key) ?? refuse(`\`${key}\` is not a ${what}`);
const unique = (xs) => [...new Set(xs)];

/** Modules copied byte for byte from the repository that owns them, by path under app/. */
const VENDORED = {
  "src/PrismR4.lex.tex": "r4/PrismR4.lex.tex",
  "src/PrismR4Proofs.lex.tex": "r4/PrismR4Proofs.lex.tex",
  "src/Foundation/Experience/V1/Model.lex.tex": "Foundation/Experience/V1/Model.lex.tex",
};
/** The label word the context-of-use model is titled by, as prism-devrel's prism.arch declares it. */
const EXPERIENCE_ENTRY = `spec = "lexlean/entry/1"
id = "brand-design-and-context-of-use"
category = "label-word"
surface_arity = 0
frame = "atom"

[denotation]
kind = "document"
module = "Foundation.Experience.V1.Model"
component = "BrandElement"

[[form]]
id = "brand-design-and-context-of-use"
channel = "text"
surface = "Brand, design and context of use"
canonical_source = true
features = ["sentence-case", "singular"]
`;
const vendored = () => Object.fromEntries(Object.entries(VENDORED).map(([to, from]) => [to, fs.readFileSync(path.join(here, "foundry-model", from), "utf8")]));

/** Every top-level declaration of the behavior modules (app/src and the vendored ones), as Module::name. */
function behaviorDeclarations() {
  const decls = new Set();
  const theorems = new Set();
  const texts = [];
  for (const file of fs.readdirSync(path.join(app, "src"))) if (file.endsWith(".lex.tex")) texts.push([file.slice(0, -".lex.tex".length), fs.readFileSync(path.join(app, "src", file), "utf8")]);
  for (const [p, text] of Object.entries(vendored())) texts.push([p.slice("src/".length, -".lex.tex".length).replaceAll("/", "."), text]);
  for (const [mod, text] of texts) {
    for (const m of text.matchAll(/^(?:  name: |\w+\{name: )(\w+),/gm)) decls.add(`${mod}::${m[1]}`);
    for (const m of text.matchAll(/^theorem\{\n  name: (\w+),/gm)) theorems.add(`${mod}::${m[1]}`);
  }
  return { decls, theorems };
}

/** The kernel's named constants: `definition{name: x, parameters: [], result: Nat, body: n}`. */
function kernelConstants(kernel) {
  return Object.fromEntries([...kernel.matchAll(/^definition\{name: (\w+), parameters: \[\], result: Nat, body: (\d+)\}$/gm)].map((m) => [m[1], Number(m[2])]));
}

/** A one-parameter Nat function of the kernel as nested `if (journey =? k) then v else ...`, evaluated. */
function kernelTable(kernel, name) {
  const m = kernel.match(new RegExp(`name: ${name},\\n  parameters: \\[journey : Nat\\],\\n  result: Nat,\\n  body: (.*)\\n`));
  if (!m) refuse(`ForgeKernel declares no ${name}(journey)`);
  return Object.fromEntries([...m[1].matchAll(/\(journey =\? (\d+)\) then (\d+)/g)].map((x) => [Number(x[1]), Number(x[2])]));
}

/**
 * The build-phase rules: ruleN's body names an event and, except for the rule every phase obeys, a
 * phase; judge maps ruleN to its answer. Each becomes a transition of the design's state machine.
 */
function phaseRules(kernel, constants) {
  const body = (name) => (kernel.match(new RegExp(`name: ${name},\\n  parameters: \\[request : Request\\],\\n  result: Bool,\\n  body: (.*)\\n`)) ?? [])[1];
  const judge = (kernel.match(/name: judge,\n  parameters: \[request : Request\],\n  result: Nat,\n  body: (.*)\n/) ?? refuse("ForgeKernel declares no judge"))[1];
  const answers = Object.fromEntries([...judge.matchAll(/\(rule(\d+) request\) then \((answer\w+)\)/g)].map((m) => [Number(m[1]), m[2]]));
  const rules = [];
  for (let n = 1; body(`rule${n}`); n++) {
    const b = body(`rule${n}`);
    const event = (b.match(/\((event\w+)\)/) ?? [])[1];
    const phase = (b.match(/\((phase\w+)\)/) ?? [])[1];
    if (!event || !answers[n]) continue; // a rule that names no event keeps the phase: it is no transition
    const to = `phase${answers[n].slice("answer".length)}`;
    if (!(to in constants)) refuse(`judge answers ${answers[n]}, which names no phase`);
    rules.push({ rule: `rule${n}`, event, from: phase ?? null, to });
  }
  return rules;
}

/** The obligations FoundrySpecification declares, in constructor order, with their requirement text. */
function specificationObligations(spec) {
  const ctors = (spec.match(/name: Obligation,[\s\S]*?constructors: \[([\s\S]*?)\n  \]/) ?? refuse("FoundrySpecification declares no Obligation"))[1];
  const names = [...ctors.matchAll(/\{name: (\w+), fields: \[\]\}/g)].map((m) => m[1]);
  return names.map((name) => {
    const text = (spec.match(new RegExp(`\\| Obligation\\.${name} => "([^"]*)"`)) ?? refuse(`requirement says nothing for ${name}`))[1];
    return { name, text };
  });
}

function registerIds() {
  const ids = new Map();
  let current = null;
  for (const line of read("model/ids.toml").split("\n")) {
    const id = line.match(/^id = "([^"]+)"$/);
    if (id) ids.set((current = id[1]), {});
    const field = line.match(/^(statement|suite) = "(.*)"$/);
    if (field && current) ids.get(current)[field[1]] = field[2];
  }
  return ids;
}

/** The Rust test that accepts each register ID: `fn <suite>_<id>()` in crates/conformance/tests. */
function registerTests() {
  const dir = path.join(root, "crates/conformance/tests");
  const tests = new Map();
  for (const file of fs.readdirSync(dir).filter((f) => f.endsWith(".rs")).sort()) {
    for (const m of fs.readFileSync(path.join(dir, file), "utf8").matchAll(/\bfn (\w+?_([a-z]+)_(\d+))\s*\(/g)) {
      const id = `${m[2].toUpperCase()}-${m[3]}`;
      if (!tests.has(id)) tests.set(id, `crates/conformance/tests/${file}::${m[1]}`);
    }
  }
  return tests;
}

/** The path a message takes from one component to another along the architecture's data flows. */
function route(edges, from, to) {
  const prev = new Map([[from, null]]);
  const queue = [from];
  while (queue.length) {
    const at = queue.shift();
    if (at === to) break;
    for (const e of edges) if (e.from === at && !prev.has(e.to)) prev.set(e.to, e), queue.push(e.to);
  }
  if (!prev.has(to)) return null;
  const hops = [];
  for (let e = prev.get(to); e; e = prev.get(e.from)) hops.unshift(e);
  return hops;
}

/** The model app/product.json and the repository project; `product` stands in for app/product.json. */
export function deriveFoundry({ product = JSON.parse(read("app/product.json")) } = {}) {
  if (product.schema !== "prism/product/1") refuse("app/product.json is a prism/product/1");
  if (!/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/.test(product.lastModified ?? "")) refuse("lastModified is an instant, YYYY-MM-DDThh:mm:ssZ");
  const kernel = read("app/src/ForgeKernel.lex.tex");
  const constants = kernelConstants(kernel);
  const { decls: declared, theorems } = behaviorDeclarations();
  const ids = registerIds();
  const tests = registerTests();
  const registry = commandRegistryData;
  const known = (ref, what) => (declared.has(ref) ? ref : refuse(`${what} \`${ref}\` is not declared in app/src`));
  const oneOf = (v, set, what) => (set.includes(v) ? v : refuse(`${what} \`${v}\` is not one of ${set.join(", ")}`));

  // ---- Architecture -------------------------------------------------------
  const arch = namer();
  const concerns = indexed(product.concerns.map((c) => ({ ...c, model: oneOf(c.model, MODEL_KINDS, "model"), decl: arch("concern", c.name) })));
  const stakeholders = indexed(product.stakeholders.map((s) => ({ ...s, decl: arch("stakeholder", s.name) })));
  for (const s of stakeholders) for (const c of s.concerns) find(concerns, c, "concern");
  const components = indexed(product.components.map((c) => ({ ...c, kind: oneOf(c.kind, COMPONENT_KINDS, "component kind"), decl: arch("component", c.name) })));
  const comp = (key) => find(components, key, "component", "key");
  const edges = indexed(product.edges.map(([from, to]) => ({ from: comp(from), to: comp(to), kind: "dataFlow", decl: arch("edge", `${comp(from).name} to ${comp(to).name}`) })));
  const viewpoints = indexed(
    stakeholders.flatMap((s) =>
      s.concerns.map((cn) => {
        const c = find(concerns, cn, "concern");
        return { stakeholder: s, concern: c, modelKind: MODEL_KINDS.indexOf(c.model), decl: arch("viewpoint", `${s.name} ${c.name}`) };
      }),
    ),
  );
  const views = indexed(viewpoints.map((v) => ({ viewpoint: v, modelKind: v.modelKind, decl: arch("view", v.decl.slice("viewpoint".length)) })));
  const viewpointOf = (stakeholder, concern) => viewpoints.find((v) => v.stakeholder.name === stakeholder && v.concern.name === concern) ?? refuse(`${stakeholder} holds no concern ${concern}`);
  const viewpointFor = (concern) => viewpoints.find((v) => v.concern.name === concern) ?? refuse(`no stakeholder holds the concern ${concern}`);
  const architecture = { stakeholders, concerns, components, edges, viewpoints, views };

  // ---- Back-end features --------------------------------------------------
  // A back-end feature carries register IDs, realizes them by kernel and protocol declarations, owns
  // command-registry features (whose owner declarations it then calls) and host channels, and is
  // accepted by its theorems and by the test of every ID it carries.
  const feat = namer();
  const modules = new Set();
  const owners = new Map(registry.features.map((x) => [x.id, x.owner.replace(".", "::")]));
  const carried = new Map();
  const ownedBy = new Map();
  const channelOwner = new Map();
  const backend = product.features.map((f) => {
    const decl = feat("feature", f.name);
    for (const id of f.ids) {
      if (!ids.has(id)) refuse(`feature ${f.name} carries ${id}, which model/ids.toml does not register`);
      if (carried.has(id)) refuse(`${id} is carried by ${carried.get(id)} and ${f.name}; one feature carries an ID`);
      carried.set(id, f.name);
    }
    for (const r of f.registry) {
      if (!owners.has(r)) refuse(`feature ${f.name} owns the registry feature ${r}, which the command registry does not declare`);
      if (ownedBy.has(r)) refuse(`the registry feature ${r} is owned by ${ownedBy.get(r)} and ${f.name}`);
      ownedBy.set(r, f.name);
    }
    for (const ch of f.channels ?? []) channelOwner.set(ch, f.name);
    for (const c of f.components) comp(c);
    const calls = unique([...f.calls, ...f.registry.map((r) => owners.get(r))]).map((c) => known(c, `call of ${f.name}`));
    const accepted = f.acceptance.map((c) => known(c, "acceptance row"));
    for (const ref of [...calls, ...accepted]) modules.add(ref.split("::")[0]);
    // A theorem or a test accepts through a definition that names it, as the builder writes it for an app.
    const accepts = [
      ...accepted.map((ref) => (theorems.has(ref) ? { decl: feat("accept", ref.replace("::", " ")), target: ref } : { ref })),
      ...f.ids.map((id) => ({ decl: feat("test", id), target: tests.get(id) ?? refuse(`${id} has no test fn <suite>_${id.toLowerCase().replace("-", "_")} in crates/conformance/tests`) })),
    ];
    const acceptance = accepts.map((x) => x.ref ?? `Features::${x.decl}`);
    const statement = `${f.name} (${f.ids.join(", ")}): ${f.text}`;
    return { ...f, decl, statement, calls, acceptance, accepts: accepts.filter((x) => x.decl), componentRecords: f.components.map(comp) };
  });
  for (const id of ids.keys()) if (!carried.has(id)) refuse(`${id} is carried by no feature; every register ID belongs to one`);
  for (const r of owners.keys()) if (!ownedBy.has(r)) refuse(`the registry feature ${r} is owned by no feature`);
  const backendOf = (name) => find(backend, name, "back-end feature");

  // ---- Screens ------------------------------------------------------------
  // Each screen requires back-end features; what its renderer calls proves the requirement.
  const shell = product.surfaces.filter((s) => s.shell).flatMap((s) => s.renderer ?? []);
  const unowned = [];
  const surfaces = product.surfaces.map((s) => {
    find(stakeholders, s.stakeholder, "stakeholder");
    const requires = s.requires.map(backendOf);
    const on = comp(s.on);
    for (const p of s.code ?? []) if (!fs.existsSync(path.join(root, p))) refuse(`screen ${s.name} names ${p}, which does not exist`);
    if (s.budget && !(s.budget in constants)) refuse(`screen ${s.name} has the budget ${s.budget}, which ForgeKernel does not declare`);
    if (s.renderer) {
      let traced;
      try {
        traced = surfaceCommands(s.renderer, registry, { exclude: s.shell ? [] : shell });
      } catch (e) {
        if (e instanceof SurfaceError) refuse(`screen ${s.name}: ${e.message}`);
        throw e;
      }
      if (traced.unresolved.length) refuse(`screen ${s.name} calls ${traced.unresolved.join(", ")}, which no IPC contract declares`);
      const owns = new Set(requires.flatMap((f) => f.registry));
      const hosts = new Set(requires.flatMap((f) => f.channels ?? []));
      for (const [r, chs] of traced.reached) if (!owns.has(r)) unowned.push(`${s.name} reaches ${r} (${chs.join(", ")}), owned by ${ownedBy.get(r)}, which it does not require`);
      for (const ch of traced.direct) if (!hosts.has(ch)) unowned.push(`${s.name} calls the host channel ${ch}, ${channelOwner.has(ch) ? `owned by ${channelOwner.get(ch)}, which it does not require` : "which no feature owns"}`);
    }
    return { ...s, requires, onRecord: on, decl: feat("screen", s.name) };
  });
  if (unowned.length) refuse(`a screen calls the back end beyond what it requires:\n  ${unowned.join("\n  ")}`);
  const surfaceOf = (name) => find(surfaces, name, "screen");

  // ---- Journeys -----------------------------------------------------------
  const journeysIn = product.journeys.map((j) => ({ ...j, actor: find(stakeholders, j.stakeholder, "stakeholder"), stepRecords: j.steps.map(surfaceOf), decl: feat("journey", j.name) }));
  for (const s of stakeholders) if (!journeysIn.some((j) => j.actor === s)) refuse(`${s.name} has no journey`);
  for (const s of surfaces) if (!journeysIn.some((j) => j.stepRecords.includes(s))) refuse(`screen ${s.name} is in no journey`);
  for (const f of backend) if (!surfaces.some((s) => s.requires.includes(f))) refuse(`back-end feature ${f.name} is required by no screen`);
  const usedComponents = new Set([...backend.flatMap((f) => f.componentRecords), ...surfaces.map((s) => s.onRecord), comp(product.opm.owner), comp(product.opm.agent), ...product.opm.steps.map((x) => comp(x.owner))]);
  for (const c of components) if (!usedComponents.has(c) && !edges.some((e) => e.from === c && usedComponents.has(e.to))) refuse(`component ${c.name} is used by no feature or screen`);

  // ---- Quality (ISO/IEC 25010:2023) --------------------------------------
  // The budgets and experience rules are the kernel's own numbers: each threshold is the kernel term,
  // and its value is pinned by a theorem the Lean kernel checks.
  const budgets = { ...kernelTable(kernel, "siteBudget"), ...kernelTable(kernel, "buildBudget") };
  const fn = product.budgets.function;
  known(fn, "budget function");
  const qualityIn = [
    ...product.quality,
    ...Object.entries(product.budgets.journeys).map(([journey, measure]) => {
      if (!(journey in constants)) refuse(`ForgeKernel declares no ${journey}`);
      const value = budgets[constants[journey]] ?? refuse(`${fn} has no budget for ${journey}`);
      return { characteristic: product.budgets.characteristic, subcharacteristic: product.budgets.subcharacteristic, requirement: `Budget ${journey.slice("journey".length)}`, text: `${measure.split(",")[0]} stays within the kernel's budget.`, measure, measureText: `${measure}, measured by the gate that asks the kernel.`, threshold: value, thresholdTerm: `(${fn} (ForgeKernel::${journey}))`, journey };
    }),
    ...Object.entries(product.rules.constants).map(([name, [sub, requirement, measure]]) => {
      if (!(name in constants)) refuse(`ForgeKernel declares no ${name}`);
      return { characteristic: product.rules.characteristic, subcharacteristic: sub, requirement, text: `${measure.split(",")[0]}, as the kernel decides it.`, measure, measureText: `${measure}.`, threshold: constants[name], thresholdTerm: `(ForgeKernel::${name})` };
    }),
  ];
  const qual = namer();
  for (const q of qualityIn) oneOf(q.subcharacteristic, ISO25010[oneOf(q.characteristic, Object.keys(ISO25010), "characteristic")], `subcharacteristic of ${q.characteristic}`);
  // The whole product quality model: every characteristic and subcharacteristic is declared, and one
  // no requirement states is listed open, so the open ones are a number a change moves.
  const characteristics = indexed(Object.keys(ISO25010).map((c) => ({ name: c, decl: qual("characteristic", c), text: `${c}, as ISO/IEC 25010:2023 defines it, for ${product.title}.` })));
  const subcharacteristics = indexed(Object.entries(ISO25010).flatMap(([c, subs]) => subs.map((s) => ({ name: s, characteristic: find(characteristics, c, "characteristic"), decl: qual("subcharacteristic", s), text: `${c}: ${s}.` }))));
  const requirements = indexed(qualityIn.map((q) => ({ name: q.requirement, text: q.text, subcharacteristic: find(subcharacteristics, q.subcharacteristic, "subcharacteristic"), decl: qual("requirement", q.requirement), row: q })));
  const measures = indexed(requirements.map((r) => ({ name: r.row.measure, text: r.row.measureText, threshold: r.row.threshold, thresholdTerm: r.row.thresholdTerm, requirement: r, decl: qual("measure", r.row.measure) })));
  const openSubcharacteristics = subcharacteristics.filter((x) => !requirements.some((r) => r.subcharacteristic === x));
  const budgetMeasure = (journey) => measures.find((x) => x.requirement.row.journey === journey);

  // ---- Features facet: back-end features, screens and journeys -----------
  // A screen calls the back-end features it requires and is accepted by them, and by the theorem that
  // pins its budget when it has one.
  const features = [
    ...backend.map((f) => ({ name: f.name, decl: f.decl, statement: f.statement, calls: f.calls, acceptance: f.acceptance, accepts: f.accepts })),
    ...surfaces.map((s) => {
      const budget = s.budget ? budgetMeasure(s.budget) : null;
      const accepts = budget ? [{ decl: feat("accept", `${s.name} budget`), target: `Quality::${budget.decl}Threshold` }] : [];
      const required = s.requires.map((f) => `Features::${f.decl}`);
      return { name: s.name, decl: s.decl, statement: `${s.name} screen, for the ${s.stakeholder.toLowerCase()}: ${s.text}`, calls: required, acceptance: [...accepts.map((x) => `Features::${x.decl}`), ...required], accepts };
    }),
  ];
  const journeys = journeysIn.map((j) => ({ decl: j.decl, actor: j.actor, steps: j.stepRecords }));

  // ---- Design -------------------------------------------------------------
  const design = namer();
  const a = (c) => `Architecture::${c.decl}`;
  const d = (decl) => `Design::${decl}`;
  const processes = [];
  const processOf = (name, owner, text) => {
    const found = processes.find((p) => p.name === name && p.owner === owner);
    if (found) return found;
    const p = { name, owner, text, decl: design("process", name) };
    processes.push(p);
    return p;
  };
  // Each event the kernel judges is a process of the guest: the trigger of the transitions it takes.
  for (const e of Object.keys(constants).filter((k) => k.startsWith("event"))) processOf(`On ${e.slice("event".length).toLowerCase()}`, comp("kernel"), `The kernel judges the ${e.slice("event".length).toLowerCase()} event and answers the next phase.`).key = e;
  const eventProcess = (key) => find(processes, key, "kernel event", "key");
  const guest = comp("kernel");
  const phases = Object.keys(constants).filter((k) => k.startsWith("phase"));
  const states = phases.map((p) => ({ key: p, name: p.slice("phase".length), decl: design("state", p.slice("phase".length)), owner: guest }));
  const state = (key) => find(states, key, "phase", "key");
  const rules = phaseRules(kernel, constants);

  // OPM (ISO/PAS 19450): building an app from an idea, zoomed into its steps, with the kernel's phases.
  const O = product.opm;
  const vOpm = { decl: design("view", "Opm build"), title: O.name, viewpoint: viewpointOf(O.stakeholder, O.concern), diagram: "opmObjectProcess" };
  const opmMain = processOf(O.name, comp(O.owner), O.text);
  const opmSteps = O.steps.map((x) => processOf(x.name, comp(x.owner), x.text));
  const opmObjects = unique([comp(O.agent), comp(O.owner), ...opmSteps.map((p) => p.owner), comp(O.effect), guest]);
  const opmThings = [
    ...opmObjects.map((c) => ({ decl: design("opm", c.name), view: vOpm, kind: "object", subject: a(c) })),
    ...[opmMain, ...opmSteps].map((p) => ({ decl: design("opm", p.name), view: vOpm, kind: "process", subject: d(p.decl) })),
  ];
  const opmStates = states.map((s) => ({ decl: design("opm", `${guest.name} ${s.name}`), view: vOpm, thing: a(guest), state: d(s.decl) }));
  const opmLinks = [
    { decl: design("opm", `${comp(O.agent).name} agent`), view: vOpm, kind: "agent", source: a(comp(O.agent)), target: d(opmMain.decl) },
    ...opmSteps.filter((p) => p.owner !== comp(O.owner)).map((p) => ({ decl: design("opm", `${p.owner.name} instrument of ${p.name}`), view: vOpm, kind: "instrument", source: a(p.owner), target: d(p.decl) })),
    // A step invokes the next only where its component hands to the next one's; otherwise the
    // studio carries the idea on, which the zoom's order shows.
    ...opmSteps
      .slice(1)
      .filter((p, i) => p.owner === opmSteps[i].owner || edges.some((e) => e.from === opmSteps[i].owner && e.to === p.owner))
      .map((p) => {
        const before = opmSteps[opmSteps.indexOf(p) - 1];
        return { decl: design("opm", `${before.name} invokes ${p.name}`), view: vOpm, kind: "invocation", source: d(before.decl), target: d(p.decl) };
      }),
    { decl: design("opm", `${opmMain.name} affects ${comp(O.effect).name}`), view: vOpm, kind: "effect", source: a(comp(O.effect)), target: d(opmMain.decl) },
  ];
  const opmZooms = opmSteps.map((p, order) => ({ decl: design("opm", `Zoom ${p.name}`), view: vOpm, process: d(opmMain.decl), subprocess: d(p.decl), order }));

  // UML components: every component and data flow.
  const vComponents = { decl: design("view", "Uml components"), title: `Components of ${product.title}`, viewpoint: viewpointFor("Integrity of published apps"), diagram: "umlComponent" };
  const umlComponents = components.map((c) => ({ decl: design("uml", c.name), view: vComponents, component: a(c) }));
  const umlConnectors = edges.map((e) => ({ decl: design("uml", `${e.from.name} to ${e.to.name}`), view: vComponents, edge: a(e) }));

  // UML sequences: one per journey. The actor opens each screen; the screen's component reaches each
  // component of every feature the screen requires along the data flows, first time in the journey.
  const sequenceViews = [];
  const lifelines = [];
  const messages = [];
  const browser = comp(O.agent);
  for (const j of journeysIn) {
    const view = { decl: design("view", `Uml ${j.name}`), title: j.name, viewpoint: viewpointOf(j.stakeholder, j.actor.concerns[0]), diagram: "umlSequence" };
    sequenceViews.push(view);
    const seq = [];
    const send = (e, process) => {
      seq.push({ e, kind: "synchronous", process });
      if (edges.some((r) => r.from === e.to && r.to === e.from)) seq.push({ e: { from: e.to, to: e.from }, kind: "reply", process });
    };
    const shown = new Set();
    for (const s of j.stepRecords) {
      const open = edges.find((e) => e.from === browser && e.to === s.onRecord);
      if (open) send(open, processOf(`Show ${s.name}`, s.onRecord, `${s.onRecord.name} shows the ${s.name} screen.`));
      for (const f of s.requires) {
        if (shown.has(f)) continue;
        shown.add(f);
        const targets = f.componentRecords.filter((c) => c !== s.onRecord);
        const routes = targets.map((c) => route(edges, s.onRecord, c)).filter(Boolean);
        if (!f.componentRecords.includes(s.onRecord) && !routes.length) refuse(`screen ${s.name} requires ${f.name}, whose components ${f.componentRecords.map((c) => c.name).join(", ")} no data flow from ${s.onRecord.name} reaches`);
        for (const hops of routes) for (const e of hops) send(e, processOf(`${f.name} in ${e.to.name}`, e.to, `${e.to.name} does its part of ${f.name.toLowerCase()}.`));
      }
    }
    const order = unique(seq.flatMap((x) => [x.e.from, x.e.to]));
    order.forEach((c, i) => lifelines.push({ decl: design("seq", `${j.name} ${c.name}`), view, component: a(c), order: i }));
    seq.forEach((x, i) => messages.push({ decl: design("msg", `${j.name} ${i}`), view, kind: x.kind, order: i, source: a(x.e.from), target: a(x.e.to), process: d(x.process.decl) }));
  }

  // UML state machine: the build phase, from the kernel's own rules.
  const vPhases = { decl: design("view", "Uml phases"), title: "The build phase of an app", viewpoint: viewpointFor("Honest proof status"), diagram: "umlStateMachine" };
  const transitionsIn = rules.flatMap((r) => {
    const trigger = eventProcess(r.event);
    const from = r.from ? [r.from] : phases; // a rule that names no phase applies in every phase
    return from.map((f) => ({ source: state(f), target: state(r.to), trigger, name: `${r.rule} from ${f.slice("phase".length)}` }));
  });
  const umlStates = states.map((s) => ({ decl: design("st", s.key), view: vPhases, state: d(s.decl), initial: constants[s.key] === 0 }));
  const transitions = transitionsIn.map((t, order) => ({ decl: design("tr", t.name), view: vPhases, order, source: t.source, target: t.target, trigger: t.trigger }));
  const designViews = [vOpm, vComponents, ...sequenceViews, vPhases];

  // ---- Security and threat model -----------------------------------------
  // STRIDE per element: every component and data flow is an element; each threat names the element
  // it acts on, and every category the method opens on an element no threat answers is stated open.
  const sec = namer();
  const assets = indexed(product.assets.map((x) => ({ ...x, component: comp(x.component), decl: sec("asset", x.name) })));
  const elements = [...components.map((c) => ({ name: c.name, dfd: DFD_OF[c.kind] })), ...edges.map((e) => ({ name: `${e.from.name} to ${e.to.name}`, dfd: "dataFlow" }))].map((x, index) => ({ ...x, index }));
  const threatsIn = product.threats.map((t) => {
    const target = find(elements, t.element ?? refuse(`threat ${t.name} names no element`), "component or data flow");
    if (!STRIDE_PER_ELEMENT[target.dfd].includes(t.stride)) refuse(`threat ${t.name} is ${t.stride} of ${target.name}, a ${target.dfd}, which STRIDE per element does not open`);
    if ([t.control, t.accepted, t.reuse].filter(Boolean).length !== 1) refuse(`threat ${t.name} has a control with evidence, a reused control or an accepted risk, exactly one`);
    for (const e of t.control?.evidence ?? []) {
      oneOf(e.kind, EVIDENCE_KINDS, "evidence kind");
      if (e.kind === "proof" && !theorems.has(known(e.reference, "proof evidence"))) refuse(`proof evidence ${e.reference} is not a theorem`);
    }
    return { ...t, stride: oneOf(t.stride, STRIDE, "stride"), likelihood: oneOf(t.likelihood, LIKELIHOOD, "likelihood"), impact: oneOf(t.impact, IMPACT, "impact"), assetRecord: find(assets, t.asset, "asset", "key"), target, decl: sec("threat", t.name) };
  });
  const threats = indexed(threatsIn);
  const risks = indexed(threats.map((t) => ({ threat: t, decl: sec("risk", t.name) })));
  const controls = indexed(risks.filter((r) => r.threat.control).map((r) => ({ risk: r, ...r.threat.control, decl: sec("control", r.threat.control.name) })));
  const mappings = threats.filter((t) => t.control || t.reuse).map((t) => ({ threat: t, controls: t.control ? controls.filter((c) => c.risk.threat === t) : [find(controls, t.reuse, "control")] }));
  const strideOpen = elements.flatMap((x) => STRIDE.filter((c) => STRIDE_PER_ELEMENT[x.dfd].includes(c) && !threats.some((t) => t.target === x && t.stride === c)).map((category) => ({ element: x, category, name: `${STRIDE_TEXT[category]} of ${x.name}` })));

  // ---- Users (ISO 9241-11:2018 context of use) ---------------------------
  // Each stakeholder is a user group, each journey a goal, each step a task done through a screen.
  const users = {
    groups: stakeholders.map((s) => ({ stakeholder: s, characteristics: s.text })),
    goals: journeysIn.map((j) => ({ user: product.stakeholders.findIndex((s) => s.name === j.stakeholder), journey: j, outcome: j.outcome })),
    environments: components.filter((c) => usedComponents.has(c) && (c.kind !== "externalSystem" || c === browser)).map((c) => ({ kind: "technical", component: c })),
  };
  users.groups = product.stakeholders.map((s) => users.groups.find((g) => g.stakeholder.name === s.name));
  users.tasks = users.goals.flatMap((g, goal) => g.journey.stepRecords.map((s, order) => ({ goal, order, feature: s })));
  users.measures = users.goals.flatMap((g, goal) => {
    const steps = g.journey.stepRecords;
    const accepted = steps.reduce((n, s) => n + features.find((f) => f.decl === s.decl).acceptance.length, 0);
    return [
      { goal, outcome: "effectiveness", observed: accepted, method: "acceptance rows of the goal's tasks: each is a feature or a theorem the screen is accepted by" },
      { goal, outcome: "efficiency", observed: steps.length, method: "screens the goal takes, from its journey's steps" },
      { goal, outcome: "satisfaction", observed: null, method: "open: no instrument in the model measures it" },
    ];
  });

  // ---- Compliance --------------------------------------------------------
  const obligations = specificationObligations(read("app/src/FoundrySpecification.lex.tex"));
  const r4Proofs = [...theorems].filter((t) => t.startsWith("PrismR4Proofs::")).sort();
  const compliance = [
    { standard: "ISO/IEC/IEEE 42010:2022", subject: "Architecture description", text: "Stakeholders, concerns, viewpoints and views are declared and their references resolve.", evidence: ["Architecture::architectureDescriptionHolds"] },
    { standard: "ISO/PAS 19450:2015 and OMG UML 2.5.1", subject: "Design views", text: `Building an app from an idea is an OPM view; every journey is a UML sequence along the data flows, and the build-phase machine is the kernel's own rules (${designViews.length} views).`, evidence: designViews.map((v) => d(v.decl)) },
    { standard: "ISO 9241-11:2018", subject: "Context of use", text: `Every stakeholder is a user group with a goal; every goal is a journey whose tasks are screens, each accepted (${users.goals.length} goals, ${users.tasks.length} tasks).`, evidence: ["Users::everyGoalHasItsUser", "Users::everyTaskServesAGoal", "Users::everyTaskIsAccepted", "Users::everyMeasureNamesAGoal"] },
    { standard: "ISO/IEC 25010:2023", subject: "Product quality", text: `All ${characteristics.length} characteristics and ${subcharacteristics.length} subcharacteristics are declared; each is stated by a requirement with a measure or listed open (${openSubcharacteristics.length} open); every budget and experience rule of the kernel is a measure whose threshold is the kernel's term.`, evidence: ["Quality::qualityModelHolds", "Quality::openSubcharacteristicCount"] },
    { standard: "ISO/IEC 27034-1:2011, ISO/IEC 27005:2022 and STRIDE per element", subject: "Application security and risk", text: `Every threat has a STRIDE category and a mapped control with evidence, or an accepted risk with its rationale; every category STRIDE per element opens on the ${elements.length} components and data flows is answered by a threat or stated open (${strideOpen.length} open).`, evidence: ["Security::strideEnumerated", "Security::strideOpenCount", "Security::controlSet"] },
    { standard: "Prism R4 (afflom/me app/specs/r4)", subject: "Own language and reasoning", text: "Every chat turn is decided by R4 before its request is made and before its answer is shown; Foundry grants no route, so only its own model answers.", evidence: r4Proofs },
    ...obligations.map((o) => ({ standard: "Prism Foundry specification", subject: o.name, text: o.text, evidence: ["FoundrySpecification::requirement"] })),
    { standard: `SPDX ${product.license}`, subject: "Licensing", text: `Foundry and its model are offered under ${product.license}.`, evidence: ["ForgeKernel::application"] },
  ];
  for (const ref of r4Proofs) modules.add(ref.split("::")[0]);
  modules.add("FoundrySpecification");
  modules.add("ForgeKernel");

  return {
    title: product.title,
    lastModified: product.lastModified,
    modules: [...modules].sort(),
    controlText: `${product.title} asks Prism R4 before every chat turn and its kernel guest before every phase change and every effect. ${product.goal}`,
    architecture,
    design: { processes, states, views: designViews, opmThings, opmStates, opmLinks, opmZooms, umlComponents, umlConnectors, lifelines, messages, umlStates, transitions },
    features: { features, journeys },
    users,
    security: { assets, threats, risks, controls, mappings, elements, strideOpen },
    quality: { characteristics, subcharacteristics, requirements, measures, openSubcharacteristics, imports: ["ForgeKernel"] },
    compliance,
  };
}

/** The Users facet: the context of use, in prism-devrel's Foundation.Experience.V1.Model. */
function usersModule(m) {
  const E = "Foundation.Experience.V1.Model";
  const U = m.users;
  const some = (n) => (n === null ? `(ctor Option.none [Nat])` : `(ctor Option.some [Nat] ${n})`);
  const decls = [
    definition("authority", `${E}::Authority`, rec(`${E}::Authority`, { framework: str("ISO 9241-11:2018 context of use and usability; ISO 9241-210:2019 human-centred design"), repository: str("https://github.com/UOR-Foundation/foundry"), commit: str("the revision of this model") })),
    ...listDefinitions("groups", `${E}::UserGroup`, U.groups.map((g) => rec(`${E}::UserGroup`, { stakeholder: `(Architecture::${g.stakeholder.decl})`, characteristics: str(g.characteristics) }))),
    ...listDefinitions("goals", `${E}::Goal`, U.goals.map((g) => rec(`${E}::Goal`, { user: g.user, journey: `(Features::${g.journey.decl})`, outcome: str(g.outcome) }))),
    ...listDefinitions("tasks", `${E}::Task`, U.tasks.map((t) => rec(`${E}::Task`, { goal: t.goal, order: t.order, feature: `(Features::${t.feature.decl})` }))),
    ...listDefinitions("environments", `${E}::Environment`, U.environments.map((x) => rec(`${E}::Environment`, { kind: ctor(`${E}::EnvironmentKind`, x.kind), component: `(Architecture::${x.component.decl})` }))),
    ...listDefinitions("measures", `${E}::UsabilityMeasure`, U.measures.map((x) => rec(`${E}::UsabilityMeasure`, { goal: x.goal, outcome: ctor(`${E}::UsabilityOutcome`, x.outcome), observed: some(x.observed), method: str(x.method) }))),
    theorem("everyGoalHasItsUser", `((${E}::goalsOfGroups (goals) ${U.groups.length}) = true)`),
    theorem("everyTaskServesAGoal", `((${E}::tasksOfGoals (tasks) ${U.goals.length}) = true)`),
    theorem("everyTaskIsAccepted", `((${E}::tasksAccepted (tasks)) = true)`),
    theorem("everyMeasureNamesAGoal", `((${E}::measuresOfGoals (measures) ${U.goals.length}) = true)`),
  ];
  return moduleText({ name: "Users", glossaries: ["lexlean.std.bool@1.2.0", "lexlean.std.nat@1.2.0", "prism.arch@1.0.0"], imports: [E, "Architecture", "Features"], title: "Brand, design and context of use", declarations: decls });
}

/** Every file the model owns under app/, by path from app/. */
export function foundryFiles() {
  const m = deriveFoundry();
  // The Foundation and Production modules and language packages every Prism app carries are the
  // builder's, so Foundry and the apps it builds are checked against the same ones.
  const files = { ...ASSETS, ...vendored(), "language/prism.arch/entries/brand-design-and-context-of-use.toml": EXPERIENCE_ENTRY };
  const modules = { ...facetModules(m), "src/Users.lex.tex": usersModule(m) };
  Object.assign(files, modules);
  const toml = fs.readFileSync(path.join(app, "lexlean.toml"), "utf8");
  const behavior = ["src/ForgeKernel.lex.tex", "src/ForgeKernelProofs.lex.tex", "src/InTabProver.lex.tex", "src/PrismR4Proofs.lex.tex"];
  const entry = `entrypoints = [${unique([...behavior, ...Object.keys(modules)]).sort().map((e) => `"${e}"`).join(", ")}]`;
  files["lexlean.toml"] = toml.replace(/^entrypoints = \[.*\]$/m, entry);
  return { files, model: m };
}

function main() {
  const check = process.argv.includes("--check");
  let files, model;
  try {
    ({ files, model } = foundryFiles());
  } catch (e) {
    if (e instanceof ModelError) {
      console.error(`foundry model: ${e.message}`);
      process.exit(2);
    }
    throw e;
  }
  const stale = [];
  for (const [p, text] of Object.entries(files)) {
    const target = path.join(app, p);
    const current = fs.existsSync(target) ? fs.readFileSync(target, "utf8") : null;
    if (current === text) continue;
    if (check) stale.push(`app/${p}`);
    else fs.mkdirSync(path.dirname(target), { recursive: true }), fs.writeFileSync(target, text);
  }
  if (check && stale.length) {
    console.error(`foundry model: ${stale.join(", ")} differ from what app/product.json and app/src project; run node studio/tools/foundry_model.mjs`);
    process.exit(1);
  }
  const f = model.features.features.length;
  const s = model.security;
  console.log(
    `${check ? "foundry model: every facet is what the model projects" : `foundry model: wrote ${Object.keys(files).length} files`} ` +
      `(${f} features, ${model.features.journeys.length} journeys, ${model.design.views.length} design views, ${s.threats.length} threats, ${s.strideOpen.length} STRIDE pairs open on ${s.elements.length} elements, ${model.quality.openSubcharacteristics.length} of ${model.quality.subcharacteristics.length} subcharacteristics open)`,
  );
}

export { ModelError };

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
