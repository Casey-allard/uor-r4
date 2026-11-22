/**
 * Every facet of one application, derived from its prompt.
 *
 * The behavior is prism-code's: its spec checker decides whether the spec is
 * well formed, whether every example is answered as written and whether every
 * property survives the counterexample search, and its renderer writes the
 * model the guest is built from. Everything else is derived here, from the
 * spec and the optional parts of the prompt, by fixed rules: a rule becomes a
 * feature, a design process and a state-machine transition; a property becomes
 * a feature, a threat its proof controls, and a quality measure; the request
 * bound the spec implies becomes a quality threshold and a control. Nothing is
 * a constant chosen here: every number in the model is a count, an index or a
 * bound read from the spec or the prompt.
 *
 * The result is plain data. The LexLean facet modules (facets.mjs) and the
 * pages that show an application (the DevRel site) are both written from it,
 * so a page cannot show a record the model does not declare.
 */
import { SpecError, counterexample, describeConditions, encodeRequest, judge, requestLimits, respond, sampleRequests, validateSpec } from '../spec.mjs';
import { answersFromSpec, compile } from '../session.mjs';
import { acceptanceVectors, declarations, names, proofDeclarations } from '../render.mjs';
import { claims, idPrefix, suiteOf } from '../template.mjs';
import { DFD_OF, ISO25010, MODEL_KINDS, PATTERNS, STRIDE, STRIDE_PER_ELEMENT, STRIDE_TEXT, readPrompt } from './prompt.mjs';

const pascal = (s) =>
  String(s)
    .split(/[^A-Za-z0-9]+/)
    .filter(Boolean)
    .map((w) => w[0].toUpperCase() + w.slice(1))
    .join('') || 'X';

/**
 * Declaration names, unique within one module. A scope (an app's module name,
 * when several apps share one set of facet modules) is put after the prefix,
 * so the records of two apps never share a name.
 */
function namer(scope = '') {
  const used = new Set();
  return (prefix, name) => {
    const base = `${prefix}${scope}${pascal(name)}`;
    let decl = base;
    for (let i = 2; used.has(decl); i++) decl = `${base}${i}`;
    used.add(decl);
    return decl;
  };
}

/**
 * Give every record of one type its index: its position when the records are
 * sorted by identifier, which is how the SDK numbers facet records.
 */
function indexed(records) {
  const sorted = [...records].sort((a, b) => (a.decl < b.decl ? -1 : a.decl > b.decl ? 1 : 0));
  sorted.forEach((r, i) => (r.index = i));
  return sorted;
}

const byName = (records, name, what) => {
  const found = records.find((r) => r.name === name);
  if (!found) throw new SpecError(`\`${name}\` is not a ${what}.`, what);
  return found;
};

/**
 * Check the spec exactly as prism-code's journey checks a drafted spec before
 * it asks for approval: well formed, every example answered as written, every
 * property without a counterexample. The first issue is a refusal.
 */
export function checkedSpec(app) {
  const spec = validateSpec(app);
  const answers = { ...answersFromSpec(app), review: 'yes' };
  const { step, issue } = compile(answers);
  if (issue) throw new SpecError(issue, step);
  for (const p of spec.properties) {
    const found = counterexample(spec, p);
    if (found.request !== null) throw new SpecError(`${p.id} does not hold: ${found.request} is answered ${found.outcome}.`, 'properties');
  }
  for (const e of spec.examples) {
    const got = respond(spec, new TextEncoder().encode(e.request));
    if (got !== e.response) throw new SpecError(`The example \`${e.request}\` expects ${e.response}; the rules answer ${got}.`, 'examples');
  }
  return spec;
}

/** How many requests the machine's successor relation is read from; past it the sample is not every request. */
const SAMPLE_LIMIT = 200000;

/**
 * A stateMachine's spec: the prompt's app, with the properties that make the
 * machine's invariants theorems appended to the ones the prompt states.
 *
 * Each derived property is about one step, and a run is a sequence of steps
 * from the initial state, so a property that holds for every step holds over
 * every run, by induction on its length:
 *
 * - a final state f is never left: `always f when state is f`;
 * - the states reachable from the initial state are closed under a step: for
 *   every reachable r and every other state u, `never u when state is r`.
 *
 * The reachable states are read off the successor relation over every request
 * the spec's sample covers. When the sample does not cover every request, no
 * state is claimed unreachable, so a sample can never make a property false;
 * and whatever is claimed, the Lean kernel proves for every request.
 */
export function machineSpec(prompt) {
  if (prompt.pattern !== 'stateMachine') return { app: prompt.app, machine: null };
  const base = validateSpec(prompt.app);
  const { state, initial, final } = prompt.machine;
  const field = base.fields.find((f) => f.name === state);
  if (!field || field.kind !== 'choice') throw new SpecError(`machine.state \`${state}\` is not a choice field; the state is one.`, 'machine');
  const states = field.values;
  if (!states.includes(initial)) throw new SpecError(`machine.initial \`${initial}\` is not a state; the states are ${states.join(', ')}.`, 'machine');
  for (const f of final) if (!states.includes(f)) throw new SpecError(`machine.final names \`${f}\`, which is not a state.`, 'machine');
  if (base.outcomes.length !== states.length || !states.every((s) => base.outcomes.includes(s))) throw new SpecError(`A state machine answers the next state: its answers are exactly the states ${states.join(', ')}.`, 'machine');

  const step = new Map(states.map((s) => [s, new Map()]));
  let sampled = 0;
  for (const request of sampleRequests(base, SAMPLE_LIMIT)) {
    sampled++;
    const { outcome, rule } = judge(base, request);
    const from = request[state];
    if (final.includes(from) && outcome !== from) throw new SpecError(`${from} is final, but \`${encodeRequest(base, request)}\` leaves it for ${outcome}.`, 'machine');
    const out = step.get(from);
    if (!out.has(outcome)) out.set(outcome, new Set());
    out.get(outcome).add(rule ?? 'otherwise');
  }
  const exhaustive = sampled < SAMPLE_LIMIT;
  const reachable = new Set([initial]);
  for (const queue = [initial]; queue.length; ) for (const next of step.get(queue.shift()).keys()) if (!reachable.has(next) && reachable.add(next)) queue.push(next);
  const closed = exhaustive ? states.filter((s) => reachable.has(s)) : states;
  const outside = states.filter((s) => !closed.includes(s));
  const is = (value) => [{ field: state, op: 'is', value }];
  const derived = [
    ...final.map((f) => ({ kind: 'always', when: is(f), then: f, why: `${f} is a final state (UML 2.5.1 FinalState): a run that reaches it stays there` })),
    ...closed.flatMap((r) => outside.map((u) => ({ kind: 'never', when: is(r), then: u, why: `no step from a state reachable from ${initial} reaches ${u}, so no run from ${initial} does` }))),
  ];
  const given = base.properties.length;
  const transitions = closed.flatMap((from) => [...step.get(from)].flatMap(([to, rules]) => [...rules].sort().map((rule) => ({ from, to, rule }))));
  return {
    app: { ...prompt.app, properties: [...(prompt.app.properties ?? []), ...derived] },
    machine: { state, initial, final, states, reachable: closed, unreachable: outside, exhaustive, sampled, transitions, derived: derived.map((_, i) => `INV-${String(given + i + 1).padStart(3, '0')}`) },
  };
}

export function deriveModel(input, { scope = '' } = {}) {
  const prompt = readPrompt(input);
  const { app, machine } = machineSpec(prompt);
  const spec = checkedSpec(app);
  if (new Set(spec.properties.map((p) => p.id)).size !== spec.properties.length) throw new SpecError('Two properties share an id; leave the ids to the builder.', 'properties');
  const n = names(spec);
  const declared = new Set([...declarations(spec, n), ...(spec.properties.length ? proofDeclarations(spec, n) : [])].map((d) => d.name));
  for (const name of Object.keys(prompt.axioms)) if (!declared.has(name)) throw new SpecError(`axioms names \`${name}\`, which the app does not declare.`, 'axioms');
  const limits = requestLimits(spec);
  const vectors = acceptanceVectors(spec);
  const title = spec.title;
  const M = n.module;
  const P = n.proofs;
  const hasProofs = spec.properties.length > 0;

  // ---- Architecture -------------------------------------------------------
  const arch = namer(scope);
  const concernsIn = [
    ...prompt.concerns,
    { name: 'Correct answers', text: `Every well-formed request is answered by the first rule that matches it, or ${spec.otherwise} when none does, as the spec states.`, model: 'quality', derived: true },
    { name: 'Refused malformed requests', text: `A request that is not the ${spec.fields.length} field${spec.fields.length === 1 ? '' : 's'} in order, or is longer than ${limits.requestMaximum} bytes, is answered invalid and never guessed.`, model: 'security', derived: true },
    { name: 'Integrity of the published app', text: 'What a visitor runs is the holo the build verified, with its fingerprint and signed provenance beside it.', model: 'security', derived: true },
    ...(hasProofs ? [{ name: 'Proved guarantees', text: `Each of the ${spec.properties.length} propert${spec.properties.length === 1 ? 'y' : 'ies'} of the spec holds for every request, proved by the Lean kernel.`, model: 'quality', derived: true }] : []),
  ].filter((c, i, all) => all.findIndex((d) => d.name === c.name) === i);
  const concerns = indexed(concernsIn.map((c) => ({ ...c, decl: arch('concern', c.name) })));

  const userConcerns = concerns.filter((c) => !c.derived).map((c) => c.name);
  const stakeholdersIn = [
    ...(prompt.stakeholders.length
      ? prompt.stakeholders
      : (spec.actors?.length ? spec.actors : ['User']).map((a) => ({ name: a, text: `Sends ${spec.fields.map((f) => f.name).join(', ')} and acts on the answer: ${spec.purpose}`, concerns: [] }))
    ).map((s) => ({ ...s, concerns: s.concerns.length ? s.concerns : [...userConcerns, 'Correct answers'] })),
    { name: 'Verifier', text: 'Anyone who loads the holo, checks its fingerprint and replays its acceptance vectors before trusting an answer.', concerns: ['Integrity of the published app', 'Refused malformed requests', ...(hasProofs ? ['Proved guarantees'] : [])], derived: true },
  ];
  const stakeholders = indexed(stakeholdersIn.map((s) => ({ ...s, decl: arch('stakeholder', s.name) })));
  for (const s of stakeholders) for (const c of s.concerns) byName(concerns, c, 'concern');

  const componentsIn = [
    { key: 'browser', name: 'Browser', kind: 'externalSystem', text: `The visitor's web browser, which loads the page and runs the ${title} guest.` },
    { key: 'guest', name: `${title} guest`, kind: 'service', text: `The Core-Wasm guest built from the model: it answers one request of at most ${limits.requestMaximum} bytes with one answer of at most ${limits.responseMaximum}.` },
    { key: 'page', name: `${title} page`, kind: 'staticSite', text: 'The page the template publishes: the holo, its fingerprint, its provenance and its acceptance vectors.' },
    { key: 'build', name: 'Prism build', kind: 'cliAdapter', text: 'prismpm verify in the SDK image prismpm.lock pins: it checks every theorem, builds the guest and replays every vector.' },
    { key: 'host', name: 'GitHub Pages', kind: 'externalSystem', text: 'Serves the page and the holo from the repository Pages workflow.' },
    { key: 'source', name: 'Source repository', kind: 'externalSystem', text: 'The repository that holds the prompt, the model and its gate.' },
    ...prompt.components.map((c) => ({ ...c, key: `extra:${c.name}` })),
  ];
  const components = indexed(componentsIn.map((c) => ({ ...c, decl: arch('component', c.name) })));
  const comp = (key) => components.find((c) => c.key === key);
  const edgesIn = [
    ['source', 'build', 'dataFlow'],
    ['build', 'guest', 'dataFlow'],
    ['build', 'page', 'dataFlow'],
    ['page', 'host', 'dataFlow'],
    ['host', 'browser', 'dataFlow'],
    ['browser', 'guest', 'dataFlow'],
    ['guest', 'browser', 'dataFlow'],
    ...prompt.components.flatMap((c) => [[`extra:${c.name}`, 'guest', 'dataFlow'], ['guest', `extra:${c.name}`, 'dataFlow']]),
  ];
  const edges = indexed(edgesIn.map(([from, to, kind]) => ({ from: comp(from), to: comp(to), kind, decl: arch('edge', `${comp(from).name} to ${comp(to).name}`) })));

  const viewpoints = indexed(
    stakeholders.flatMap((s) =>
      s.concerns.map((cn) => {
        const c = byName(concerns, cn, 'concern');
        return { stakeholder: s, concern: c, modelKind: MODEL_KINDS.indexOf(c.model), decl: arch('viewpoint', `${s.name} ${c.name}`) };
      }),
    ),
  );
  const views = indexed(viewpoints.map((v) => ({ viewpoint: v, modelKind: v.modelKind, decl: arch('view', v.decl.slice('viewpoint'.length)) })));
  const viewpointFor = (concernName) => viewpoints.find((v) => v.concern.name === concernName) ?? viewpoints[0];

  // ---- Design -------------------------------------------------------------
  const design = namer(scope);
  const a = (c) => `Architecture::${c.decl}`;
  const d = (decl) => `Design::${decl}`;
  const process = (name, owner, text) => ({ decl: design('process', name), name, owner, text });
  const pRequest = process('Send request', comp('browser'), 'The browser sends the request bytes to the guest.');
  const pDecide = process('Decide', comp('guest'), 'The guest answers one request.');
  const pParse = process('Parse request', comp('guest'), `Read the ${spec.fields.length} field${spec.fields.length === 1 ? '' : 's'}, refusing anything else.`);
  const pJudge = process('Judge', comp('guest'), 'Apply the rules in order; the first that matches decides.');
  const pEncode = process('Encode answer', comp('guest'), 'Write the answer word.');
  const pBuild = process('Build and verify', comp('build'), 'Check every theorem, build the guest, replay every vector.');
  const pRules = spec.rules.map((r, i) => ({ ...process(`Rule ${i + 1}`, comp('guest'), r.says), rule: r }));
  const pOtherwise = process('Otherwise', comp('guest'), `No rule matches: the answer is ${spec.otherwise}.`);
  const processes = [pRequest, pDecide, pParse, pJudge, pEncode, pBuild, ...pRules, pOtherwise];

  const state = (name, owner, text) => ({ decl: design('state', name), name, owner, text });
  const sIdle = state('Idle', comp('guest'), 'Waiting for a request.');
  const sDeciding = state('Deciding', comp('guest'), 'A well-formed request is being judged.');
  const sRefused = state('Refused', comp('guest'), 'The request was malformed and is answered invalid.');
  const sAnswered = Object.fromEntries(spec.outcomes.map((o) => [o, state(`Answered ${o}`, comp('guest'), spec.meanings?.[o] ? `Answered ${o}: ${spec.meanings[o]}` : `Answered ${o}.`)]));
  const states = [sIdle, sDeciding, sRefused, ...Object.values(sAnswered)];

  const vOpm = { decl: design('view', 'Opm decide'), title: `Deciding a ${title} request`, viewpoint: viewpointFor('Correct answers'), diagram: 'opmObjectProcess' };
  const vComponents = { decl: design('view', 'Uml components'), title: `Components of ${title}`, viewpoint: viewpointFor('Integrity of the published app'), diagram: 'umlComponent' };
  const vSequence = { decl: design('view', 'Uml request'), title: `Answering a ${title} request`, viewpoint: viewpointFor('Correct answers'), diagram: 'umlSequence' };
  const vStates = { decl: design('view', 'Uml states'), title: `States of the ${title} guest`, viewpoint: viewpointFor('Refused malformed requests'), diagram: 'umlStateMachine' };
  // A state machine's own states, beside the guest's: one per state, each step a transition triggered by the rule that takes it.
  const vMachine = machine && { decl: design('view', 'Uml machine'), title: `The ${title} state machine`, viewpoint: viewpointFor('Correct answers'), diagram: 'umlStateMachine' };
  const machineStates = machine ? Object.fromEntries(machine.states.map((s) => [s, state(`Machine ${s}`, comp('guest'), `${machine.final.includes(s) ? 'Final state' : s === machine.initial ? 'Initial state' : 'State'} ${s}${spec.meanings?.[s] ? `: ${spec.meanings[s]}` : '.'}`)])) : {};
  if (machine) states.push(...Object.values(machineStates));
  const designViews = [vOpm, vComponents, vSequence, vStates, ...(machine ? [vMachine] : [])];

  const opmThings = [
    { decl: design('opm', 'Browser'), view: vOpm, kind: 'object', subject: a(comp('browser')) },
    { decl: design('opm', 'Guest'), view: vOpm, kind: 'object', subject: a(comp('guest')) },
    ...[pDecide, pParse, pJudge, pEncode].map((p) => ({ decl: design('opm', p.name), view: vOpm, kind: 'process', subject: d(p.decl) })),
  ];
  const opmStates = [sIdle, sDeciding].map((s) => ({ decl: design('opm', `Guest ${s.name}`), view: vOpm, thing: a(comp('guest')), state: d(s.decl) }));
  const opmLinks = [
    { decl: design('opm', 'Guest agent'), view: vOpm, kind: 'agent', source: a(comp('guest')), target: d(pDecide.decl) },
    { decl: design('opm', 'Browser instrument'), view: vOpm, kind: 'instrument', source: a(comp('browser')), target: d(pParse.decl) },
    { decl: design('opm', 'Parse invokes judge'), view: vOpm, kind: 'invocation', source: d(pParse.decl), target: d(pJudge.decl) },
    { decl: design('opm', 'Judge invokes encode'), view: vOpm, kind: 'invocation', source: d(pJudge.decl), target: d(pEncode.decl) },
    { decl: design('opm', 'Encode affects browser'), view: vOpm, kind: 'effect', source: a(comp('browser')), target: d(pEncode.decl) },
  ];
  const opmZooms = [pParse, pJudge, pEncode].map((p, order) => ({ decl: design('opm', `Zoom ${p.name}`), view: vOpm, process: d(pDecide.decl), subprocess: d(p.decl), order }));
  const umlComponents = components.map((c) => ({ decl: design('uml', c.name), view: vComponents, component: a(c) }));
  const umlConnectors = edges.map((e) => ({ decl: design('uml', `${e.from.name} to ${e.to.name}`), view: vComponents, edge: a(e) }));
  const lifelines = [comp('browser'), comp('guest')].map((c, order) => ({ decl: design('seq', c.name), view: vSequence, component: a(c), order }));
  const messages = [
    { decl: design('seq', 'Request'), view: vSequence, kind: 'synchronous', order: 0, source: a(comp('browser')), target: a(comp('guest')), process: d(pDecide.decl) },
    { decl: design('seq', 'Reply'), view: vSequence, kind: 'reply', order: 1, source: a(comp('guest')), target: a(comp('browser')), process: d(pDecide.decl) },
  ];
  const umlStates = [
    ...[sIdle, sDeciding, sRefused, ...Object.values(sAnswered)].map((s) => ({ decl: design('st', s.name), view: vStates, state: d(s.decl), initial: s === sIdle })),
    ...Object.entries(machineStates).map(([v, s]) => ({ decl: design('st', s.name), view: vMachine, state: d(s.decl), initial: v === machine.initial })),
  ];
  let order = 0;
  const transitions = [
    { decl: design('tr', 'Accept'), source: sIdle, target: sDeciding, trigger: pParse },
    { decl: design('tr', 'Refuse'), source: sIdle, target: sRefused, trigger: pParse },
    ...pRules.map((p, i) => ({ decl: design('tr', `Rule ${i + 1}`), source: sDeciding, target: sAnswered[p.rule.then], trigger: p })),
    { decl: design('tr', 'Otherwise'), source: sDeciding, target: sAnswered[spec.otherwise], trigger: pOtherwise },
    ...spec.outcomes.map((o) => ({ decl: design('tr', `Reset ${o}`), source: sAnswered[o], target: sIdle, trigger: pEncode })),
    { decl: design('tr', 'Reset refused'), source: sRefused, target: sIdle, trigger: pEncode },
  ].map((t) => ({ ...t, view: vStates, order: order++ }));
  if (machine) {
    order = 0;
    const trigger = (rule) => (rule === 'otherwise' ? pOtherwise : pRules.find((p) => p.rule.id === rule));
    for (const t of machine.transitions) transitions.push({ decl: design('tr', `Machine ${t.from} to ${t.to} by ${t.rule}`), source: machineStates[t.from], target: machineStates[t.to], trigger: trigger(t.rule), view: vMachine, order: order++ });
  }

  // ---- Features and journeys ---------------------------------------------
  const feat = namer(scope);
  const proofOf = Object.fromEntries(spec.properties.map((p) => [p.id, n.properties[p.id]]));
  const features = [
    ...spec.rules.map((r, i) => ({ key: r.id, decl: feat('feature', `Rule ${i + 1}`), statement: r.says, calls: [`${M}::${n.rules[r.id]}`, `${M}::judge`], acceptance: [`${M}::application`], why: r.why })),
    { key: 'otherwise', decl: feat('feature', 'Otherwise'), statement: `When no rule matches, the answer is ${spec.otherwise}.`, calls: [`${M}::judge`], acceptance: [`${M}::application`], why: spec.otherwiseWhy },
    { key: 'refuse', decl: feat('feature', 'Refuse malformed'), statement: `A request that is not ${spec.fields.map((f) => f.name).join(', ')} in order, separated by single spaces, or is over ${limits.requestMaximum} bytes, is answered invalid.`, calls: [`${M}::dispatch`, `${M}::decodeField1`], acceptance: [`${M}::application`] },
    ...spec.properties.map((p) => {
      // An acceptance row is a definition (PP2018): this one names the theorem that accepts the feature.
      const accept = { decl: feat('accept', p.id), target: `${P}::${proofOf[p.id].theorem}` };
      return { key: p.id, decl: feat('feature', p.id), statement: p.says, calls: [`${M}::judge`], acceptance: [`Features::${accept.decl}`], accept, why: p.why, property: p };
    }),
  ];
  const featureBy = (key) => {
    const f = features.find((x) => x.key === key);
    if (!f) throw new SpecError(`The journey step \`${key}\` names no rule, property, otherwise or refuse.`, 'journeys');
    return f;
  };
  const journeySteps = (s) => {
    const given = prompt.journeys.find((j) => j.stakeholder === s.name);
    if (given) return given.steps.map(featureBy);
    if (s.derived) return [featureBy('refuse'), ...spec.properties.map((p) => featureBy(p.id))];
    return [...spec.rules.map((r) => featureBy(r.id)), featureBy('otherwise')];
  };
  for (const j of prompt.journeys) byName(stakeholders, j.stakeholder, 'stakeholder');
  const journeys = stakeholders.map((s) => ({ decl: feat('journey', s.name), actor: s, steps: journeySteps(s) }));

  // ---- Security and threat model -----------------------------------------
  const sec = namer(scope);
  const assetsIn = [
    { key: 'request', name: 'Request bytes', text: `The bytes a caller sends, at most ${limits.requestMaximum} of them.`, component: comp('guest') },
    { key: 'answer', name: 'Answer', text: `The answer the guest returns: one of ${spec.outcomes.join(', ')}, or invalid.`, component: comp('guest') },
    { key: 'holo', name: 'Published holo', text: 'The holo and page the Pages workflow publishes, with its fingerprint and signed provenance.', component: comp('page') },
    { key: 'source', name: 'Model source', text: 'The prompt, the spec and the LexLean model in the repository, from which the holo is built.', component: comp('source') },
  ];
  const assets = indexed(assetsIn.map((x) => ({ ...x, decl: sec('asset', x.name) })));
  const asset = (key) => assets.find((x) => x.key === key);
  const gate = (reference) => ({ kind: 'gate', reference });
  // Proof evidence names a property (INV-001), which becomes its theorem, or a theorem as Module::name; a theorem of this app's proofs module must be one it declares.
  const theorems = new Set(spec.properties.map((p) => `${P}::${proofOf[p.id].theorem}`));
  const promptThreats = prompt.threats.map((t) => {
    if (!t.control) return t;
    const evidence = t.control.evidence.map((e) => {
      if (e.kind !== 'proof') return e;
      if (/^INV-\d{3}$/.test(e.reference)) {
        if (!proofOf[e.reference]) throw new SpecError(`Proof evidence \`${e.reference}\` names no property; the properties are ${spec.properties.map((p) => p.id).join(', ') || 'none'}.`, 'threat');
        return { ...e, reference: `${P}::${proofOf[e.reference].theorem}` };
      }
      if (!/^[A-Za-z][A-Za-z0-9.]*::[A-Za-z][A-Za-z0-9]*$/.test(e.reference)) throw new SpecError(`Proof evidence \`${e.reference}\` is not a property id or Module::theorem.`, 'threat');
      if (e.reference.startsWith(`${P}::`) && !theorems.has(e.reference)) throw new SpecError(`Proof evidence \`${e.reference}\` is not a theorem of ${P}.`, 'threat');
      return e;
    });
    return { ...t, control: { ...t.control, evidence } };
  });
  const threatsIn = [
    {
      name: 'Oversized request',
      text: `A request longer than ${limits.requestMaximum} bytes makes the guest parse or allocate without bound.`,
      stride: 'denialOfService',
      asset: 'request',
      likelihood: 'medium',
      impact: 'minor',
      control: { name: 'Bounded request', text: `dispatch answers invalid for a request over ${limits.requestMaximum} bytes before reading a field, and the guest allocates at most ${limits.guestAllocationMaximum}.`, evidence: [gate('acceptance vectors at the request bound, one byte over it, and at the guest allocation bound, replayed by prismpm verify')] },
    },
    {
      name: 'Malformed request',
      text: 'A request with a missing, extra, unknown or badly written field is answered as if it were well formed.',
      stride: 'tampering',
      asset: 'request',
      likelihood: 'high',
      impact: 'major',
      control: { name: 'Strict decoder', text: 'Every field is decoded exactly: choices from their list, numbers as plain decimal digits within their bounds, single spaces between fields; anything else is invalid.', evidence: [gate('malformed-request acceptance vectors, replayed in the compiled guest by prismpm verify')] },
    },
    {
      name: 'Substituted holo',
      text: 'A holo other than the one the build verified is served under the app address.',
      stride: 'spoofing',
      asset: 'holo',
      element: comp('guest').name,
      likelihood: 'low',
      impact: 'critical',
      control: { name: 'Fingerprint and provenance', text: 'The page verifies the holo digests before it runs, and the build signs its provenance as the repository workflow.', evidence: [gate('actions/attest-build-provenance in the pages workflow'), gate('prism-code check of the holo identities in the page')] },
    },
    {
      name: 'Tampered model',
      text: 'The model the holo is built from differs from the reviewed prompt and spec.',
      stride: 'tampering',
      asset: 'source',
      likelihood: 'low',
      impact: 'major',
      control: { name: 'Regenerated from the prompt', text: 'The model is written from the prompt by the builder, and the build checks every theorem and replays every vector in the pinned SDK.', evidence: [gate('prismpm verify in the SDK image prismpm.lock pins'), { kind: 'review', reference: 'app/prompt.json' }] },
    },
    {
      name: 'Granted capability',
      text: 'The guest reads, stores or sends data beyond the request it answers.',
      stride: 'informationDisclosure',
      asset: 'answer',
      likelihood: 'low',
      impact: 'major',
      control: { name: 'No capability granted', text: 'prism.toml grants the holo nothing and the model declares capabilitiesEmpty, so a guest import of any host capability is refused.', evidence: [gate('prism-code check refuses an ungranted guest import')] },
    },
    ...spec.properties.map((p) => ({
      name: `Answer that breaks ${p.id}`,
      text: `The guest answers ${p.then} ${p.kind === 'never' ? 'although' : 'not when'} ${describeConditions(p.when)}${p.why ? `, which matters because ${p.why}` : ''}.`,
      stride: 'repudiation',
      asset: 'answer',
      likelihood: 'medium',
      impact: 'critical',
      control: { name: `Proof of ${p.id}`, text: `${p.says} The Lean kernel proves it for every request.`, evidence: [{ kind: 'proof', reference: `${P}::${proofOf[p.id].theorem}` }] },
    })),
    { name: 'Substituted host', text: 'Something other than the repository Pages workflow serves pages under its address.', stride: 'spoofing', asset: 'holo', element: comp('host').name, likelihood: 'low', impact: 'critical', accepted: 'A page cannot defend against its own host. The holo carries signed build provenance a visitor can check, and the app can be rebuilt from its prompt.' },
    ...promptThreats,
  ];
  // STRIDE per element: every component and every data flow is an element of
  // the data flow diagram, and each element kind is open to the categories the
  // method assigns it (STRIDE_PER_ELEMENT). The holo crosses three flows on
  // its way to a visitor; its fingerprint answers tampering on each, and it
  // carries nothing secret to disclose.
  const elements = [
    ...components.map((c) => ({ name: c.name, dfd: DFD_OF[c.kind], decl: c.decl })),
    ...edges.map((e) => ({ name: `${e.from.name} to ${e.to.name}`, dfd: 'dataFlow', decl: e.decl })),
  ].map((x, index) => ({ ...x, index }));
  const element = (name, threat) => {
    const found = elements.find((x) => x.name === name);
    if (!found) throw new SpecError(`The threat \`${threat}\` acts on \`${name}\`, which is no component or data flow; the elements are ${elements.map((x) => x.name).join(', ')}.`, 'threat');
    return found;
  };
  const holoFlows = [['build', 'page'], ['page', 'host'], ['host', 'browser']].map(([from, to]) => `${comp(from).name} to ${comp(to).name}`);
  const flowThreats = holoFlows.flatMap((flow) => [
    { name: `Holo altered from ${flow}`, text: `The holo or its page is changed on the flow from ${flow}.`, stride: 'tampering', asset: 'holo', element: flow, likelihood: 'low', impact: 'critical', reuse: 'Fingerprint and provenance' },
    { name: `Holo read from ${flow}`, text: `Someone reads the holo or its page on the flow from ${flow}.`, stride: 'informationDisclosure', asset: 'holo', element: flow, likelihood: 'high', impact: 'minor', accepted: 'The holo, its page and its acceptance vectors are published openly and carry no secret.' },
  ]);
  // The build and the page it publishes answer with the controls above: the
  // signed provenance names the workflow that built the holo, and the pinned
  // SDK rechecks every theorem and vector.
  const buildName = comp('build').name;
  const pageName = comp('page').name;
  const elementThreats = [
    { name: 'Impersonated build', text: `Something other than ${buildName} in the repository workflow claims to have built the holo.`, stride: 'spoofing', asset: 'holo', element: buildName, likelihood: 'low', impact: 'critical', reuse: 'Fingerprint and provenance' },
    { name: 'Altered build', text: `${buildName} runs other code than the SDK image the lock pins, or skips a check.`, stride: 'tampering', asset: 'source', element: buildName, likelihood: 'low', impact: 'critical', reuse: 'Regenerated from the prompt' },
    { name: 'Denied build', text: `Nobody can show which run of ${buildName} produced a published holo.`, stride: 'repudiation', asset: 'holo', element: buildName, likelihood: 'low', impact: 'major', reuse: 'Fingerprint and provenance' },
    { name: 'Altered page', text: `${pageName} is changed after the build published it.`, stride: 'tampering', asset: 'holo', element: pageName, likelihood: 'low', impact: 'critical', reuse: 'Fingerprint and provenance' },
    { name: 'Denied page', text: `Nobody can show which build published ${pageName}.`, stride: 'repudiation', asset: 'holo', element: pageName, likelihood: 'low', impact: 'major', reuse: 'Fingerprint and provenance' },
    { name: 'Page read', text: `Someone reads ${pageName}.`, stride: 'informationDisclosure', asset: 'holo', element: pageName, likelihood: 'high', impact: 'minor', accepted: 'The holo, its page and its acceptance vectors are published openly and carry no secret.' },
  ];
  const threats = indexed(
    [...threatsIn, ...flowThreats, ...elementThreats].map((t) => ({ ...t, decl: sec('threat', t.name), assetRecord: asset(t.asset), target: element(t.element ?? asset(t.asset).component.name, t.name) })),
  );
  const risks = indexed(threats.map((t) => ({ threat: t, decl: sec('risk', t.name) })));
  const controls = indexed(risks.filter((r) => r.threat.control).map((r) => ({ risk: r, ...r.threat.control, decl: sec('control', r.threat.control.name) })));
  const mappings = threats
    .filter((t) => t.control || t.reuse)
    .map((t) => ({ threat: t, controls: t.control ? controls.filter((c) => c.risk.threat === t) : [controls.find((c) => c.name === t.reuse)] }));
  const strideOpen = elements.flatMap((x) =>
    STRIDE.filter((category) => STRIDE_PER_ELEMENT[x.dfd].includes(category) && !threats.some((t) => t.target === x && t.stride === category)).map((category) => ({
      element: x,
      category,
      name: `${STRIDE_TEXT[category]} of ${x.name}`,
    })),
  );

  // ---- Quality (ISO/IEC 25010:2023) --------------------------------------
  const qual = namer(scope);
  const shortest = vectors.reduce((m, v) => Math.min(m, v.bytes.length), Infinity);
  const qualityIn = [
    { characteristic: 'Functional suitability', subcharacteristic: 'Functional correctness', requirement: 'Examples answered as stated', text: `Every one of the ${spec.examples.length} example${spec.examples.length === 1 ? '' : 's'} of the spec is answered as it states.`, measure: 'Example mismatches', measureText: 'Examples the guest answers otherwise than they state.', threshold: 0 },
    { characteristic: 'Functional suitability', subcharacteristic: 'Functional completeness', requirement: 'Every request answered', text: `Every request gets an answer: a rule, ${spec.otherwise} otherwise, or invalid.`, measure: 'Unanswered requests', measureText: 'Acceptance vectors the guest does not answer.', threshold: 0 },
    { characteristic: 'Performance efficiency', subcharacteristic: 'Time behaviour', requirement: 'Bounded request', text: `A request over ${limits.requestMaximum} bytes is refused before any field is read.`, measure: 'Request bytes read', measureText: 'Longest request the guest decodes, in bytes.', threshold: limits.requestMaximum },
    { characteristic: 'Performance efficiency', subcharacteristic: 'Resource utilization', requirement: 'Bounded allocation', text: `The guest allocates at most ${limits.guestAllocationMaximum} bytes for one request.`, measure: 'Guest allocation bytes', measureText: 'Most bytes the guest allocates for one request.', threshold: limits.guestAllocationMaximum },
    { characteristic: 'Reliability', subcharacteristic: 'Faultlessness', requirement: 'Every vector replays', text: `All ${vectors.length} acceptance vectors replay in the compiled guest, the shortest ${shortest} bytes long.`, measure: 'Failed vectors', measureText: 'Acceptance vectors whose replay differs from the model.', threshold: 0 },
    ...(hasProofs ? [{ characteristic: 'Security', subcharacteristic: 'Integrity', requirement: 'Every property proved', text: `All ${spec.properties.length} properties are theorems the Lean kernel checks.`, measure: 'Unproved properties', measureText: 'Properties of the spec without a checked theorem.', threshold: 0 }] : []),
    ...prompt.quality,
  ];
  // The whole product quality model: every characteristic and subcharacteristic
  // of ISO/IEC 25010:2023 is declared, and one no requirement states is open.
  const characteristics = indexed(Object.keys(ISO25010).map((c) => ({ name: c, decl: qual('characteristic', c), text: `${c}, as ISO/IEC 25010:2023 defines it, for ${title}.` })));
  const subcharacteristics = indexed(
    Object.entries(ISO25010).flatMap(([c, subs]) => subs.map((s) => ({ name: s, characteristic: characteristics.find((x) => x.name === c), decl: qual('subcharacteristic', s), text: `${c}: ${s}.` }))),
  );
  const requirements = indexed(qualityIn.map((q) => ({ name: q.requirement, text: q.text, subcharacteristic: subcharacteristics.find((x) => x.name === q.subcharacteristic), decl: qual('requirement', q.requirement), row: q })));
  const measures = indexed(requirements.map((r) => ({ name: r.row.measure, text: r.row.measureText, threshold: r.row.threshold, requirement: r, decl: qual('measure', r.row.measure) })));
  const openSubcharacteristics = subcharacteristics.filter((x) => !requirements.some((r) => r.subcharacteristic === x));

  // ---- Compliance --------------------------------------------------------
  const obligations = [
    { standard: 'ISO/IEC/IEEE 42010:2022', subject: 'Architecture description', text: 'Stakeholders, concerns, viewpoints and views are declared and their references resolve.', evidence: ['Architecture::architectureDescriptionHolds'], holds: 'Architecture::architectureValid' },
    { standard: 'ISO/PAS 19450:2015 and OMG UML 2.5.1', subject: 'Design views', text: 'Every design view is checked by the SDK against the system model before it is rendered.', evidence: designViews.map((v) => d(v.decl)) },
    { standard: 'ISO/IEC 25010:2023', subject: 'Product quality', text: `All ${characteristics.length} characteristics and ${subcharacteristics.length} subcharacteristics are declared; each subcharacteristic is stated by a requirement with a measure or listed open (${openSubcharacteristics.length} open); every parent resolves.`, evidence: ['Quality::qualityModelHolds', 'Quality::openSubcharacteristicCount'], holds: 'Quality::qualityValid' },
    { standard: 'ISO/IEC 27034-1:2011, ISO/IEC 27005:2022 and STRIDE per element', subject: 'Application security and risk', text: `Every threat has a STRIDE category and a mapped control with evidence, or an accepted risk with its rationale; every category STRIDE per element opens on the ${elements.length} components and data flows is answered by a threat or stated open (${strideOpen.length} open).`, evidence: ['Security::strideEnumerated', 'Security::strideOpenCount', 'Security::controlSet'], holds: 'Security::securityValid' },
    { standard: 'NIST OSCAL 1.1.0', subject: 'Control set', text: 'The controls, their evidence and the threats each answers are emitted by the build as an OSCAL 1.1.0 component definition, oscal/component-definition.json, which the oscal-1.1.0-structural oracle accepts; accepted risks are stated beside them.', evidence: ['Security::strideEnumerated', 'Security::controlSet', 'oscal-validator oscal/component-definition.json'], holds: 'Security::securityValid' },
    patternObligation(prompt, spec, machine, M, P, n),
    { standard: 'OpenSpec', subject: 'Requirements', text: 'Every rule and property of the spec is a requirement with a scenario in openspec/specs.', evidence: [`${M}::application`] },
    { standard: `SPDX ${prompt.license}`, subject: 'Licensing', text: `The application and its model are offered under ${prompt.license}.`, evidence: [`${M}::application`] },
  ];

  return {
    prompt,
    spec,
    pattern: { key: prompt.pattern, ...PATTERNS[prompt.pattern] },
    machine,
    names: n,
    limits,
    vectors,
    title,
    module: M,
    proofs: P,
    hasProofs,
    claims: claims(spec),
    idPrefix: idPrefix(spec),
    suite: suiteOf(spec),
    architecture: { stakeholders, concerns, components, edges, viewpoints, views },
    design: { processes, states, views: designViews, opmThings, opmStates, opmLinks, opmZooms, umlComponents, umlConnectors, lifelines, messages, umlStates, transitions },
    features: { features, journeys },
    security: { assets, threats, risks, controls, mappings, elements, strideOpen },
    quality: { characteristics, subcharacteristics, requirements, measures, openSubcharacteristics },
    compliance: obligations,
    judge: (request) => judge(spec, request),
  };
}

/** The obligation the app's pattern places on it, with the declarations that discharge it. */
function patternObligation(prompt, spec, machine, M, P, n) {
  const { standard } = PATTERNS[prompt.pattern];
  if (prompt.pattern === 'stateMachine') {
    const proofs = machine.derived.map((id) => `${P}::${n.properties[id].theorem}`);
    const parts = [
      `The answer to a state and an event is the next state, one of ${machine.states.join(', ')}; every run starts in ${machine.initial}.`,
      machine.final.length ? `The final state${machine.final.length === 1 ? '' : 's'} ${machine.final.join(', ')} ${machine.final.length === 1 ? 'is' : 'are'} never left.` : '',
      machine.unreachable.length ? `No run from ${machine.initial} reaches ${machine.unreachable.join(', ')}.` : '',
    ];
    return { standard, subject: 'State machine', text: parts.filter(Boolean).join(' '), evidence: [`${M}::judge`, ...proofs] };
  }
  if (prompt.pattern === 'policyDecision') return { standard, subject: 'Policy decision', text: `Every well-formed request is answered with exactly one decision: ${spec.outcomes.join(', ')}; a malformed one is invalid, never a decision.`, evidence: [`${M}::judge`, `${M}::encodeAnswer`] };
  return { standard, subject: 'Decision table', text: `The ${spec.rules.length} rules are tried in order and the first that matches gives the answer; otherwise ${spec.otherwise}.`, evidence: [`${M}::judge`] };
}

export { ISO25010 };
