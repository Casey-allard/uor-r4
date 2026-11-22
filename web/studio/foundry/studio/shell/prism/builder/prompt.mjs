/**
 * `prism/prompt/1`: the one prompt from which the builder writes a whole Prism
 * application, every facet of it.
 *
 * The prompt is shaped as an MCP tool call (`tools/call` with `name` and
 * `arguments`), and its `app` member is exactly the `prism/spec/1` spec that
 * prism-code's own MCP tool `prism_spec` takes: the request fields, the
 * answers, the rules, the properties Lean proves, and the examples. A prompt
 * drafted for `prism_spec` is therefore already a prompt here. What the spec
 * does not say (who the app is for, what they care about, what can go wrong,
 * how good is good enough) is the rest of the prompt, and every part of it is
 * optional: what is left out is derived from the spec, never invented.
 */
import { SpecError } from '../spec.mjs';

export const PROMPT_SCHEMA = 'prism/prompt/1';

/** The STRIDE categories `Production.ThreatModel::Stride` declares. */
export const STRIDE = ['spoofing', 'tampering', 'repudiation', 'informationDisclosure', 'denialOfService', 'elevationOfPrivilege'];
/** `Foundation.Sec::Likelihood` and `Foundation.Sec::Impact`. */
export const LIKELIHOOD = ['low', 'medium', 'high'];
export const IMPACT = ['minor', 'major', 'critical'];
/** `Foundation.Arch::ComponentKind`. */
export const COMPONENT_KINDS = ['service', 'database', 'externalSystem', 'cliAdapter', 'staticSite', 'releaseBinary'];

/**
 * STRIDE per element (Shostack, Threat Modeling: Designing for Security, 2014,
 * table 3-9; Microsoft SDL Threat Modeling Tool): the categories each kind of
 * data flow diagram element is open to. Repudiation of a data store applies
 * when it holds a record someone could deny; it is kept, so it is enumerated
 * and answered rather than dropped.
 */
export const STRIDE_PER_ELEMENT = {
  externalEntity: ['spoofing', 'repudiation'],
  process: ['spoofing', 'tampering', 'repudiation', 'informationDisclosure', 'denialOfService', 'elevationOfPrivilege'],
  dataStore: ['tampering', 'repudiation', 'informationDisclosure', 'denialOfService'],
  dataFlow: ['tampering', 'informationDisclosure', 'denialOfService'],
};
/** The data flow diagram element kind of each component kind. */
export const DFD_OF = { service: 'process', cliAdapter: 'process', releaseBinary: 'process', database: 'dataStore', staticSite: 'dataStore', externalSystem: 'externalEntity' };
export const DFD_KINDS = Object.keys(STRIDE_PER_ELEMENT);
export const STRIDE_TEXT = { spoofing: 'Spoofing', tampering: 'Tampering', repudiation: 'Repudiation', informationDisclosure: 'Information disclosure', denialOfService: 'Denial of service', elevationOfPrivilege: 'Elevation of privilege' };
/** The model a concern is viewed in: `Foundation.Arch::ModelKind`, in the order its index counts. */
export const MODEL_KINDS = ['component', 'quality', 'security'];
/** `Production.ThreatModel::EvidenceKind`. */
export const EVIDENCE_KINDS = ['gate', 'proof', 'review'];
/** What a threat in the prompt can be against: the assets every app has. */
export const ASSET_KEYS = ['request', 'answer', 'holo', 'source'];
/** The axioms a Lean kernel check can observe in a generated declaration, as LexLean's exact axiom policy names them. */
export const AXIOMS = ['Classical.choice', 'Quot.sound', 'propext'];

/**
 * The established pattern an app's behavior follows. All three are one
 * function from a request to one answer, which is what a `prism/spec/1` guest
 * computes; they differ in what the answer means and so in what the builder
 * checks and proves about it.
 *
 * - `decisionTable`: an OMG DMN 1.5 decision table with hit policy FIRST
 *   (section 8.2.11): the rules are tried in order and the first that matches
 *   gives the output.
 * - `policyDecision`: an OASIS XACML 3.0 policy decision point (section 7.11,
 *   7.14): the answer is exactly one of the four decisions.
 * - `stateMachine`: an OMG UML 2.5.1 behavioral state machine (clause 14) as a
 *   pure transition function, the next state of the current state and an
 *   event. The state travels in the request, so the answer is fed back as the
 *   next request's state, and the invariants the builder derives hold over
 *   every run, by induction on its length.
 */
export const PATTERNS = {
  decisionTable: { name: 'Decision table', standard: 'OMG DMN 1.5, hit policy FIRST', url: 'https://www.omg.org/spec/DMN/1.5/' },
  policyDecision: { name: 'Policy decision', standard: 'OASIS XACML 3.0 policy decision point', url: 'https://docs.oasis-open.org/xacml/3.0/xacml-3.0-core-spec-os-en.html' },
  stateMachine: { name: 'State machine', standard: 'OMG UML 2.5.1 behavioral state machine', url: 'https://www.omg.org/spec/UML/2.5.1/' },
};
/** The four decisions of XACML 3.0 section 7.11, as answer words. */
export const XACML_DECISIONS = ['permit', 'deny', 'not-applicable', 'indeterminate'];

/**
 * ISO/IEC 25010:2023 product quality: each characteristic with its
 * subcharacteristics. A quality row in the prompt names one pair.
 */
export const ISO25010 = {
  'Functional suitability': ['Functional completeness', 'Functional correctness', 'Functional appropriateness'],
  'Performance efficiency': ['Time behaviour', 'Resource utilization', 'Capacity'],
  Compatibility: ['Co-existence', 'Interoperability'],
  'Interaction capability': ['Appropriateness recognizability', 'Learnability', 'Operability', 'User error protection', 'User engagement', 'Inclusivity', 'User assistance', 'Self-descriptiveness'],
  Reliability: ['Faultlessness', 'Availability', 'Fault tolerance', 'Recoverability'],
  Security: ['Confidentiality', 'Integrity', 'Non-repudiation', 'Accountability', 'Authenticity', 'Resistance'],
  Maintainability: ['Modularity', 'Reusability', 'Analysability', 'Modifiability', 'Testability'],
  Flexibility: ['Adaptability', 'Scalability', 'Installability', 'Replaceability'],
  Safety: ['Operational constraint', 'Risk identification', 'Fail safe', 'Hazard warning', 'Safe integration'],
};

const text = { type: 'string' };
const named = (description, extra = {}, required = ['name']) => ({ type: 'object', description, properties: { name: text, text, ...extra }, required });

/**
 * The `arguments` of the MCP tool `prism_prompt`. `app` is the input of
 * prism-code's `prism_spec` (`prism/spec/1`), referenced by the schema it
 * carries in prism-code's MCP server and checked against it in the tests.
 */
export function inputSchema(specInput) {
  return {
    type: 'object',
    description: 'One prism/prompt/1 prompt: the app as a prism/spec/1 spec, and, optionally, everything else every facet needs.',
    properties: {
      schema: { const: PROMPT_SCHEMA },
      goal: { type: 'string', description: 'The goal the app is built toward, in one sentence a developer would say. It becomes the spec purpose when app.purpose is left out.' },
      app: specInput,
      stakeholders: { type: 'array', description: 'Who the app is for. A Verifier, anyone who replays the holo before trusting it, is always added.', items: named('A stakeholder.', { concerns: { type: 'array', items: text, description: 'Names of concerns, from concerns or the derived ones.' } }) },
      concerns: { type: 'array', description: 'What the stakeholders care about, each viewed in one model.', items: named('A concern.', { model: { enum: MODEL_KINDS } }) },
      components: { type: 'array', description: 'Systems outside the app that send it requests or read its answers.', items: named('A component.', { kind: { enum: COMPONENT_KINDS } }) },
      threats: {
        type: 'array',
        description: 'What can go wrong, beyond the threats every app has (an oversized or malformed request, a substituted holo or source, a granted capability, an answer a property rules out).',
        items: {
          type: 'object',
          properties: {
            name: text,
            text,
            stride: { enum: STRIDE },
            asset: { enum: ASSET_KEYS },
            element: { type: 'string', description: 'The architecture element the threat acts on, for STRIDE per element: a component name, or a data flow written `From to To`. Left out, it is the component that holds the asset.' },
            likelihood: { enum: LIKELIHOOD },
            impact: { enum: IMPACT },
            control: named('The control that answers it.', { evidence: { type: 'array', items: { type: 'object', properties: { kind: { enum: EVIDENCE_KINDS }, reference: { type: 'string', description: 'For proof evidence, a property id such as INV-001 (its theorem) or Module::theorem; otherwise the gate or file that is the evidence.' } }, required: ['kind', 'reference'] } } }, ['name', 'text', 'evidence']),
            accepted: { type: 'string', description: 'Instead of a control: why the risk is accepted.' },
          },
          required: ['name', 'text', 'stride', 'asset'],
        },
      },
      quality: {
        type: 'array',
        description: 'How good is good enough, as ISO/IEC 25010:2023 requirements with a measure and its threshold.',
        items: {
          type: 'object',
          properties: {
            characteristic: { enum: Object.keys(ISO25010) },
            subcharacteristic: text,
            requirement: text,
            text,
            measure: text,
            measureText: text,
            threshold: { type: 'integer', minimum: 0 },
          },
          required: ['characteristic', 'subcharacteristic', 'requirement', 'text', 'measure', 'measureText', 'threshold'],
        },
      },
      journeys: { type: 'array', description: 'Ordered steps a stakeholder takes. A step is a rule ID (REQ-001), a property ID (INV-001), otherwise or refuse.', items: { type: 'object', properties: { stakeholder: text, steps: { type: 'array', items: text } }, required: ['stakeholder', 'steps'] } },
      pattern: { enum: Object.keys(PATTERNS), description: 'The pattern the behavior follows: decisionTable (OMG DMN 1.5, hit policy FIRST, the default), policyDecision (OASIS XACML 3.0: the answers are exactly permit, deny, not-applicable and indeterminate) or stateMachine (OMG UML 2.5.1: the answer is the next state, and machine is required).' },
      machine: {
        type: 'object',
        description: 'For a stateMachine: which choice field carries the current state, the initial state, and the final states. The answers are exactly the state values. The builder proves that a final state is never left and that no run from the initial state reaches a state outside the reachable set.',
        properties: {
          state: { type: 'string', description: 'The choice field whose values are the states.' },
          initial: { type: 'string', description: 'The state every run starts in.' },
          final: { type: 'array', items: text, uniqueItems: true, description: 'States a run never leaves once it reaches them (UML FinalState).' },
        },
        required: ['state', 'initial'],
      },
      license: { type: 'string', description: 'The SPDX license expression; MIT OR Apache-2.0 when left out.' },
      repository: { type: 'string', description: 'owner/name of the repository the app will live in, for its page and links.' },
      axioms: {
        type: 'object',
        description: 'The exact axiom set a verify observed for a generated declaration, by declaration name, as prism-code records it when LexLean reports LLV7005. `node builder/cli.mjs axioms` writes it from a verify log.',
        additionalProperties: { type: 'array', items: { enum: AXIOMS }, uniqueItems: true },
      },
    },
    required: ['app'],
  };
}

const refuse = (message, rule) => {
  throw new SpecError(message, rule);
};

const words = (s) => String(s ?? '').replace(/[\p{Cc}"\\]/gu, '').replace(/\s+/g, ' ').trim();

/**
 * Accept a prompt as data: either the arguments alone, or a whole MCP
 * `tools/call` request whose `params.name` is `prism_prompt`. Every member is
 * checked against the closed sets above; free text is cut to what a LexLean
 * string can carry. Returns the arguments.
 */
export function readPrompt(input) {
  let args = input;
  if (input && input.jsonrpc === '2.0') {
    if (input.method !== 'tools/call' || input.params?.name !== 'prism_prompt') refuse('A prompt is the tools/call of prism_prompt.', 'prompt');
    args = input.params.arguments;
  }
  if (!args || typeof args !== 'object' || Array.isArray(args)) refuse('A prompt is an object.', 'prompt');
  if (args.schema !== undefined && args.schema !== PROMPT_SCHEMA) refuse(`schema must be ${PROMPT_SCHEMA}`, 'schema');
  if (!args.app || typeof args.app !== 'object') refuse('A prompt carries its app as a prism/spec/1 spec under app.', 'app');
  const prompt = structuredClone(args);
  prompt.schema = PROMPT_SCHEMA;
  if (prompt.goal !== undefined) prompt.goal = words(prompt.goal);
  if (!prompt.app.purpose && prompt.goal) prompt.app.purpose = prompt.goal;
  const list = (key) => {
    if (prompt[key] === undefined) return [];
    if (!Array.isArray(prompt[key])) refuse(`${key} is a list.`, key);
    return prompt[key];
  };
  const oneOf = (value, set, what) => {
    if (!set.includes(value)) refuse(`${what} \`${value}\` is not one of ${set.join(', ')}.`, what);
    return value;
  };
  const unique = (items, what) => {
    const seen = new Set();
    for (const item of items) {
      if (!item.name) refuse(`Every ${what} has a name.`, what);
      if (seen.has(item.name)) refuse(`The ${what} \`${item.name}\` is named twice.`, what);
      seen.add(item.name);
    }
    return items;
  };
  prompt.stakeholders = unique(list('stakeholders').map((s) => ({ name: words(s.name), text: words(s.text), concerns: (s.concerns ?? []).map(words) })), 'stakeholder');
  prompt.concerns = unique(list('concerns').map((c) => ({ name: words(c.name), text: words(c.text), model: oneOf(c.model ?? 'component', MODEL_KINDS, 'model') })), 'concern');
  prompt.components = unique(list('components').map((c) => ({ name: words(c.name), text: words(c.text), kind: oneOf(c.kind ?? 'externalSystem', COMPONENT_KINDS, 'component kind') })), 'component');
  prompt.threats = unique(
    list('threats').map((t) => {
      const threat = { name: words(t.name), text: words(t.text), stride: oneOf(t.stride, STRIDE, 'stride'), asset: oneOf(t.asset, ASSET_KEYS, 'asset'), likelihood: oneOf(t.likelihood ?? 'medium', LIKELIHOOD, 'likelihood'), impact: oneOf(t.impact ?? 'major', IMPACT, 'impact') };
      if (t.element !== undefined) threat.element = words(t.element);
      if (t.control && t.accepted) refuse(`The threat \`${threat.name}\` has both a control and an accepted risk; it has one or the other.`, 'threat');
      if (t.control) {
        const evidence = (t.control.evidence ?? []).map((e) => ({ kind: oneOf(e.kind, EVIDENCE_KINDS, 'evidence kind'), reference: words(e.reference) }));
        if (evidence.length === 0) refuse(`The control of \`${threat.name}\` names no evidence.`, 'threat');
        threat.control = { name: words(t.control.name), text: words(t.control.text), evidence };
      } else if (t.accepted) threat.accepted = words(t.accepted);
      else refuse(`The threat \`${threat.name}\` needs a control with evidence, or an accepted risk with its rationale.`, 'threat');
      return threat;
    }),
    'threat',
  );
  prompt.quality = list('quality').map((q) => {
    const characteristic = oneOf(q.characteristic, Object.keys(ISO25010), 'characteristic');
    oneOf(q.subcharacteristic, ISO25010[characteristic], `subcharacteristic of ${characteristic}`);
    if (!(Number.isSafeInteger(q.threshold) && q.threshold >= 0)) refuse(`The threshold of \`${q.measure}\` is a whole number.`, 'quality');
    return { characteristic, subcharacteristic: q.subcharacteristic, requirement: words(q.requirement), text: words(q.text), measure: words(q.measure), measureText: words(q.measureText), threshold: q.threshold };
  });
  prompt.journeys = list('journeys').map((j) => ({ stakeholder: words(j.stakeholder), steps: (j.steps ?? []).map(words) }));
  prompt.license = words(prompt.license ?? 'MIT OR Apache-2.0');
  if (prompt.axioms === undefined) prompt.axioms = {};
  if (typeof prompt.axioms !== 'object' || Array.isArray(prompt.axioms)) refuse('axioms maps a declaration name to its observed axiom set.', 'axioms');
  prompt.axioms = Object.fromEntries(
    Object.keys(prompt.axioms)
      .sort()
      .map((name) => {
        const set = prompt.axioms[name];
        if (!Array.isArray(set)) refuse(`The axioms of \`${name}\` are a list.`, 'axioms');
        for (const a of set) oneOf(a, AXIOMS, `axiom of ${name}`);
        if (new Set(set).size !== set.length) refuse(`The axioms of \`${name}\` name one axiom twice.`, 'axioms');
        return [name, [...set].sort()];
      }),
  );
  prompt.pattern = oneOf(prompt.pattern ?? (prompt.machine ? 'stateMachine' : 'decisionTable'), Object.keys(PATTERNS), 'pattern');
  if (prompt.pattern === 'stateMachine') {
    const m = prompt.machine;
    if (!m || typeof m !== 'object' || Array.isArray(m)) refuse('A stateMachine names its machine: the state field and the initial state.', 'machine');
    if (m.final !== undefined && !Array.isArray(m.final)) refuse('machine.final is a list of states.', 'machine');
    prompt.machine = { state: words(m.state), initial: words(m.initial), final: [...new Set((m.final ?? []).map(words))] };
  } else if (prompt.machine !== undefined) refuse(`machine belongs to a stateMachine; this prompt is a ${prompt.pattern}.`, 'machine');
  if (prompt.pattern === 'policyDecision') {
    const outcomes = prompt.app.outcomes ?? [];
    if (outcomes.length !== XACML_DECISIONS.length || !XACML_DECISIONS.every((d) => outcomes.includes(d))) refuse(`A policyDecision answers exactly the XACML 3.0 decisions ${XACML_DECISIONS.join(', ')}.`, 'pattern');
  }
  if (prompt.repository !== undefined && !/^[A-Za-z0-9-]+\/[A-Za-z0-9._-]+$/.test(prompt.repository)) refuse('repository is owner/name.', 'repository');
  return prompt;
}
