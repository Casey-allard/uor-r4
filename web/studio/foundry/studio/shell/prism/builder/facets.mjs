/**
 * The LexLean facet modules of one application, written from its model
 * (model.mjs): architecture (ISO/IEC/IEEE 42010), design views (OPM and UML),
 * features and journeys, security and the threat model (ISO/IEC 27034 and
 * 27005, STRIDE), quality (ISO/IEC 25010:2023) and compliance.
 *
 * Each module carries a validity predicate over its own records and a theorem,
 * proved by `decide`, that the predicate holds: the Lean kernel checks that
 * every index the records cross-reference is declared. The SDK checks the rest
 * (identifiers, numbering, cross-module references, the threat model and the
 * design views) when it verifies the project. The helper definitions are the
 * ones Prism-DevRel's own model uses (app/src/Architecture.lex.tex,
 * app/src/Quality.lex.tex and app/src/Compliance.lex.tex), so a generated
 * application is checked exactly as this repository's model is.
 */
import { cons, ctor, definition, listDefinitions, moduleText, rec, str, structure, theorem } from './lean.mjs';
import { DFD_KINDS, STRIDE } from './prompt.mjs';

const BOOL = 'lexlean.std.bool@1.2.0';
const NAT = 'lexlean.std.nat@1.2.0';
const ARCH = 'prism.arch@1.0.0';
const SEC = 'prism.sec@1.0.0';
const QUAL = 'prism.qual@1.0.0';

const A = 'Foundation.Arch';
const listDef = (name, type, items) => listDefinitions(name, type, items.map((r) => `(${r.decl})`));
const conj = (terms) => terms.reduceRight((rest, t) => (rest ? `(${t} && ${rest})` : t), '');
const nats = (name, values) => listDefinitions(name, 'Nat', values.map(String));
/** Exactly one of two Booleans, without `!=`, which the lexicon does not carry. */
const exactlyOne = definition('exactlyOne', 'Bool', '(if left then (if right then false else true) else right)', { parameters: ['left : Bool', 'right : Bool'] });
const listLength = (name, type) =>
  definition(name, 'Nat', `(match values | List.nil => 0 | List.cons value rest => ((${name} rest) + 1))`, { parameters: [`values : (List ${type})`], recursive: 'values' });

/** The ISO standards profile, in the order and at the positions the SDK's profile numbers them. */
const STANDARDS = [
  ['standardIso25010', 0, 'ISO-25010-2023', '2023'],
  ['standardIso27005', 1, 'ISO-27005-2022', '2022'],
  ['standardIso27034', 2, 'ISO-27034-1-2011', '2011'],
  ['standardIso42010', 4, 'ISO-42010-2022', '2022'],
];

export function architecture(m) {
  const { stakeholders, concerns, components, edges, viewpoints, views } = m.architecture;
  const decls = [
    ...STANDARDS.map(([name, i, id, edition]) => definition(name, `${A}::StandardReference`, rec(`${A}::StandardReference`, { standardIndex: i, editionIndex: 0, standardId: str(id), edition: str(edition) }))),
    ...stakeholders.map((s) => definition(s.decl, `${A}::Stakeholder`, rec(`${A}::Stakeholder`, { index: s.index, name: str(s.name), text: str(s.text) }))),
    ...concerns.map((c) => definition(c.decl, `${A}::Concern`, rec(`${A}::Concern`, { index: c.index, name: str(c.name), text: str(c.text) }))),
    ...components.map((c) => definition(c.decl, `${A}::Component`, rec(`${A}::Component`, { index: c.index, kind: ctor(`${A}::ComponentKind`, c.kind), name: str(c.name), text: str(c.text) }))),
    ...edges.map((e) => definition(e.decl, `${A}::Edge`, rec(`${A}::Edge`, { index: e.index, fromIndex: e.from.index, toIndex: e.to.index, kind: ctor(`${A}::EdgeKind`, e.kind) }))),
    ...viewpoints.map((v) => definition(v.decl, `${A}::Viewpoint`, rec(`${A}::Viewpoint`, { index: v.index, stakeholderIndex: v.stakeholder.index, concernIndex: v.concern.index, modelKindIndex: v.modelKind }))),
    ...views.map((v) => definition(v.decl, `${A}::View`, rec(`${A}::View`, { index: v.index, viewpointIndex: v.viewpoint.index, modelKindIndex: v.modelKind }))),
    ...listDef('componentRecords', `${A}::Component`, components),
    ...listDef('edgeRecords', `${A}::Edge`, edges),
    ...listDef('viewpointRecords', `${A}::Viewpoint`, viewpoints),
    ...listDef('viewRecords', `${A}::View`, views),
    definition('edgesKnown', 'Bool', `(match records
    | List.nil => true
    | List.cons edge rest => (((project edge fromIndex) <? count) && (((project edge toIndex) <? count) && (edgesKnown count rest))))`, { parameters: ['count : Nat', `records : (List ${A}::Edge)`], recursive: 'records' }),
    definition('viewpointsKnown', 'Bool', `(match records
    | List.nil => true
    | List.cons viewpoint rest => (((project viewpoint stakeholderIndex) <? stakeholderCount) && (((project viewpoint concernIndex) <? concernCount) && (((project viewpoint modelKindIndex) <? 3) && (viewpointsKnown stakeholderCount concernCount rest)))))`, { parameters: ['stakeholderCount : Nat', 'concernCount : Nat', `records : (List ${A}::Viewpoint)`], recursive: 'records' }),
    definition('viewMatches', 'Bool', `(match viewpoints
    | List.nil => false
    | List.cons viewpoint rest => ((((project viewpoint index) =? (project view viewpointIndex)) && ((project viewpoint modelKindIndex) =? (project view modelKindIndex))) || (viewMatches view rest)))`, { parameters: [`view : ${A}::View`, `viewpoints : (List ${A}::Viewpoint)`], recursive: 'viewpoints' }),
    definition('viewsConform', 'Bool', `(match records
    | List.nil => true
    | List.cons view rest => ((viewMatches view viewpoints) && (viewsConform viewpoints rest)))`, { parameters: [`viewpoints : (List ${A}::Viewpoint)`, `records : (List ${A}::View)`], recursive: 'records' }),
    definition('architectureValid', 'Bool', conj([`(edgesKnown ${components.length} (edgeRecords))`, `(viewpointsKnown ${stakeholders.length} ${concerns.length} (viewpointRecords))`, '(viewsConform (viewpointRecords) (viewRecords))'])),
    theorem('architectureDescriptionHolds', '((architectureValid) = true)'),
  ];
  return moduleText({ name: 'Architecture', glossaries: [BOOL, NAT, ARCH], imports: ['Foundation.Arch'], title: 'architectural component', declarations: decls });
}

export function design(m) {
  const D = m.design;
  const ref = (decl) => `Design::${decl}`;
  const t = (type) => `${A}::${type}`;
  const def = (decl, type, fields) => definition(decl, t(type), rec(t(type), fields));
  const decls = [
    ...D.processes.map((p) => def(p.decl, 'SystemProcess', { owner: str(`Architecture::${p.owner.decl}`) })),
    ...D.states.map((s) => def(s.decl, 'SystemState', { owner: str(`Architecture::${s.owner.decl}`) })),
    ...D.views.map((v) => def(v.decl, 'DesignView', { title: str(v.title), viewpoint: str(`Architecture::${v.viewpoint.decl}`), diagram: ctor(t('DesignDiagram'), v.diagram) })),
    ...D.opmThings.map((x) => def(x.decl, 'OpmThing', { view: str(ref(x.view.decl)), kind: ctor(t('OpmThingKind'), x.kind), subject: str(x.subject) })),
    ...D.opmStates.map((x) => def(x.decl, 'OpmState', { view: str(ref(x.view.decl)), thing: str(x.thing), state: str(x.state) })),
    ...D.opmLinks.map((x) => def(x.decl, 'OpmLink', { view: str(ref(x.view.decl)), kind: ctor(t('OpmLinkKind'), x.kind), source: str(x.source), target: str(x.target) })),
    ...D.opmZooms.map((x) => def(x.decl, 'OpmZoom', { view: str(ref(x.view.decl)), process: str(x.process), subprocess: str(x.subprocess), order: x.order })),
    ...D.umlComponents.map((x) => def(x.decl, 'UmlComponent', { view: str(ref(x.view.decl)), component: str(x.component) })),
    ...D.umlConnectors.map((x) => def(x.decl, 'UmlConnector', { view: str(ref(x.view.decl)), edge: str(x.edge) })),
    ...D.lifelines.map((x) => def(x.decl, 'UmlLifeline', { view: str(ref(x.view.decl)), component: str(x.component), order: x.order })),
    ...D.messages.map((x) => def(x.decl, 'UmlMessage', { view: str(ref(x.view.decl)), kind: ctor(t('UmlMessageKind'), x.kind), order: x.order, source: str(x.source), target: str(x.target), process: str(x.process) })),
    ...D.umlStates.map((x) => def(x.decl, 'UmlState', { view: str(ref(x.view.decl)), state: str(x.state), initial: x.initial ? 'true' : 'false' })),
    ...D.transitions.map((x) => def(x.decl, 'UmlTransition', { view: str(ref(x.view.decl)), order: x.order, source: str(ref(x.source.decl)), target: str(ref(x.target.decl)), trigger: str(ref(x.trigger.decl)) })),
  ];
  return moduleText({ name: 'Design', glossaries: [BOOL, NAT, ARCH], imports: ['Architecture', 'Foundation.Arch'], title: 'design view', declarations: decls });
}

export function features(m) {
  const { features: fs, journeys } = m.features;
  const strings = (items) => cons('String', items.map(str));
  const decls = [
    // An acceptance row is a definition (PP2018): one that accepts by a theorem or a test names it.
    ...fs.flatMap((f) => [f.accept, ...(f.accepts ?? [])].filter(Boolean)).map((x) => definition(x.decl, 'String', str(x.target))),
    ...fs.map((f) => definition(f.decl, `${A}::Feature`, rec(`${A}::Feature`, { statement: str(f.statement), calls: strings(f.calls), acceptance: strings(f.acceptance) }))),
    ...journeys.map((j) => definition(j.decl, `${A}::Journey`, rec(`${A}::Journey`, { actor: str(`Architecture::${j.actor.decl}`), steps: strings(j.steps.map((f) => `Features::${f.decl}`)) }))),
  ];
  const imports = [...(m.modules ?? [m.module]), 'Architecture', 'Foundation.Arch'];
  return moduleText({ name: 'Features', glossaries: [BOOL, NAT, ARCH], imports, title: 'feature', declarations: decls });
}

export function security(m) {
  const { assets, threats, risks, controls, mappings, elements, strideOpen } = m.security;
  const S = 'Foundation.Sec';
  const T = 'Production.ThreatModel';
  const evidence = (items) => cons(`${T}::Evidence`, items.map((e) => rec(`${T}::Evidence`, { kind: ctor(`${T}::EvidenceKind`, e.kind), reference: str(e.reference) })));
  const ref = (decl) => str(`Security::${decl}`);
  const byElement = (a, b) => a.element - b.element || a.category - b.category;
  const threatPairs = threats.map((t) => ({ element: t.target.index, category: STRIDE.indexOf(t.stride), name: ref(t.decl) })).sort(byElement);
  const openPairs = strideOpen.map((p) => ({ element: p.element.index, category: STRIDE.indexOf(p.category), name: str(p.name) })).sort(byElement);
  const decls = [
    ...assets.map((x) => definition(x.decl, `${S}::Asset`, rec(`${S}::Asset`, { index: x.index, componentIndex: x.component.index }))),
    ...threats.map((x) => definition(x.decl, `${S}::Threat`, rec(`${S}::Threat`, { index: x.index }))),
    ...risks.map((r) => definition(r.decl, `${S}::Risk`, rec(`${S}::Risk`, { index: r.index, assetIndex: r.threat.assetRecord.index, threatIndex: r.threat.index, likelihood: ctor(`${S}::Likelihood`, r.threat.likelihood), impact: ctor(`${S}::Impact`, r.threat.impact) }))),
    ...controls.map((c) => definition(c.decl, `${S}::ApplicationSecurityControl`, rec(`${S}::ApplicationSecurityControl`, { index: c.index, riskIndex: c.risk.index }))),
    definition('controlSet', `${T}::ControlSet`, rec(`${T}::ControlSet`, { title: str(`${m.title} security controls`), text: str(m.controlText ?? `${m.title} answers one request with one answer in a Core-Wasm guest granted no capability. ${m.spec.purpose}`), version: str('0.1.0'), lastModified: str(m.lastModified) })),
    ...assets.map((x) => definition(`modeled${x.decl[0].toUpperCase()}${x.decl.slice(1)}`, `${T}::ModeledAsset`, rec(`${T}::ModeledAsset`, { asset: ref(x.decl), name: str(x.name), text: str(x.text) }))),
    ...threats.map((x) => definition(`modeled${x.decl[0].toUpperCase()}${x.decl.slice(1)}`, `${T}::ModeledThreat`, rec(`${T}::ModeledThreat`, { threat: ref(x.decl), name: str(x.name), text: str(x.text), category: ctor(`${T}::Stride`, x.stride) }))),
    ...controls.map((c) => definition(`modeled${c.decl[0].toUpperCase()}${c.decl.slice(1)}`, `${T}::ModeledControl`, rec(`${T}::ModeledControl`, { control: ref(c.decl), name: str(c.name), text: str(c.text), evidence: evidence(c.evidence) }))),
    ...mappings.map((x) => definition(`mapping${x.threat.decl.slice('threat'.length)}`, `${T}::ThreatMapping`, rec(`${T}::ThreatMapping`, { threat: ref(x.threat.decl), controls: cons('String', x.controls.map((c) => ref(c.decl))) }))),
    ...threats.filter((x) => x.accepted).map((x) => definition(`accepted${x.decl.slice('threat'.length)}`, `${T}::AcceptedRisk`, rec(`${T}::AcceptedRisk`, { threat: ref(x.decl), rationale: str(x.accepted) }))),
    // STRIDE per element. An element is a component, then a data flow, by
    // index; its kind is a data flow diagram kind, by position in
    // externalEntity, process, dataStore, dataFlow; a category is a position in
    // spoofing, tampering, repudiation, informationDisclosure, denialOfService,
    // elevationOfPrivilege. Every pair the method opens is answered by a
    // threat or stated open, never both, and no other pair is stated open.
    // threatPairs and openPairs are written in element order, so an element's
    // pairs are the run at the head of each list when the walk reaches it: the
    // check reads both lists once, and its cost grows with the pairs, not with
    // elements times pairs. A pair out of order is left over, and refused.
    structure('StrideElement', ['{name: index, type: Nat}', '{name: kind, type: Nat}', '{name: name, type: String}']),
    structure('StridePair', ['{name: element, type: Nat}', '{name: category, type: Nat}', '{name: name, type: String}']),
    ...listDefinitions('strideElements', 'StrideElement', elements.map((x) => rec('StrideElement', { index: x.index, kind: DFD_KINDS.indexOf(x.dfd), name: str(x.name) }))),
    ...listDefinitions('threatPairs', 'StridePair', threatPairs.map((p) => rec('StridePair', { element: p.element, category: p.category, name: p.name }))),
    ...listDefinitions('openPairs', 'StridePair', openPairs.map((p) => rec('StridePair', { element: p.element, category: p.category, name: p.name }))),
    ...nats('strideCategories', STRIDE.map((_, i) => i)),
    definition('strideApplies', 'Bool', '(if (kind =? 0) then ((category =? 0) || (category =? 2)) else (if (kind =? 1) then (category <? 6) else (if (kind =? 2) then ((0 <? category) && (category <? 5)) else ((category =? 1) || ((category =? 3) || (category =? 4))))))', { parameters: ['kind : Nat', 'category : Nat'] }),
    definition('runListed', 'Bool', `(match pairs
    | List.nil => false
    | List.cons pair rest => (if ((project pair element) =? element) then (((project pair category) =? category) || (runListed element category rest)) else false))`, { parameters: ['element : Nat', 'category : Nat', 'pairs : (List StridePair)'], recursive: 'pairs' }),
    definition('afterRun', '(List StridePair)', `(match pairs
    | List.nil => (nil StridePair)
    | List.cons pair rest => (if ((project pair element) =? element) then (afterRun element rest) else (cons pair rest)))`, { parameters: ['element : Nat', 'pairs : (List StridePair)'], recursive: 'pairs' }),
    // One line: a match that is not recursive is written on one line by lexlean fmt.
    definition('pairsEmpty', 'Bool', '(match pairs | List.nil => true | List.cons pair rest => false)', { parameters: ['pairs : (List StridePair)'] }),
    exactlyOne,
    definition('categoriesStated', 'Bool', `(match categories
    | List.nil => true
    | List.cons category rest => ((if (strideApplies kind category) then (exactlyOne (runListed element category answered) (runListed element category stated)) else (if (runListed element category stated) then false else true)) && (categoriesStated element kind answered stated rest)))`, { parameters: ['element : Nat', 'kind : Nat', 'answered : (List StridePair)', 'stated : (List StridePair)', 'categories : (List Nat)'], recursive: 'categories' }),
    definition('elementsStated', 'Bool', `(match records
    | List.nil => ((pairsEmpty answered) && (pairsEmpty stated))
    | List.cons item rest => (((project item index) =? position) && ((categoriesStated position (project item kind) answered stated (strideCategories)) && (elementsStated (position + 1) rest (afterRun position answered) (afterRun position stated)))))`, { parameters: ['position : Nat', 'records : (List StrideElement)', 'answered : (List StridePair)', 'stated : (List StridePair)'], recursive: 'records' }),
    definition('pairsKnown', 'Bool', `(match pairs
    | List.nil => true
    | List.cons pair rest => (((project pair element) <? count) && (((project pair category) <? 6) && (pairsKnown count rest))))`, { parameters: ['count : Nat', 'pairs : (List StridePair)'], recursive: 'pairs' }),
    listLength('pairCount', 'StridePair'),
    definition('securityValid', 'Bool', conj(['(elementsStated 0 (strideElements) (threatPairs) (openPairs))', `(pairsKnown ${elements.length} (threatPairs))`, `(pairsKnown ${elements.length} (openPairs))`])),
    theorem('strideEnumerated', '((securityValid) = true)'),
    // The open pairs are a measure: a change that answers or opens one moves this number.
    theorem('strideOpenCount', `((pairCount (openPairs)) = ${strideOpen.length})`),
  ];
  return moduleText({ name: 'Security', glossaries: [BOOL, NAT, ARCH, SEC], imports: ['Foundation.Sec', 'Production.ThreatModel', 'Architecture'], title: 'security threat', declarations: decls });
}

export function quality(m) {
  const { characteristics, subcharacteristics, requirements, measures, openSubcharacteristics } = m.quality;
  const Q = 'Foundation.Qual';
  const named = (name, items) => listDefinitions(name, 'Named', items.map((x) => rec('Named', { index: x.index, name: str(x.name), text: str(x.text) })));
  const known = (name, type, field) =>
    definition(name, 'Bool', `(match records
    | List.nil => true
    | List.cons item rest => (((project item ${field}) <? count) && (${name} count rest)))`, { parameters: ['count : Nat', `records : (List ${Q}::${type})`], recursive: 'records' });
  const decls = [
    structure('Named', ['{name: index, type: Nat}', '{name: name, type: String}', '{name: text, type: String}']),
    ...characteristics.map((x) => definition(x.decl, `${Q}::ProductQualityCharacteristic`, rec(`${Q}::ProductQualityCharacteristic`, { index: x.index }))),
    ...subcharacteristics.map((x) => definition(x.decl, `${Q}::QualitySubcharacteristic`, rec(`${Q}::QualitySubcharacteristic`, { index: x.index, characteristicIndex: x.characteristic.index }))),
    ...requirements.map((x) => definition(x.decl, `${Q}::QualityRequirement`, rec(`${Q}::QualityRequirement`, { index: x.index, subcharacteristicIndex: x.subcharacteristic.index }))),
    ...measures.map((x) => definition(x.decl, `${Q}::QualityMeasure`, rec(`${Q}::QualityMeasure`, { index: x.index, requirementIndex: x.requirement.index, threshold: x.threshold }))),
    ...named('characteristicNames', characteristics),
    ...named('subcharacteristicNames', subcharacteristics),
    ...named('requirementNames', requirements),
    ...named('measureNames', measures),
    ...listDef('subcharacteristicRecords', `${Q}::QualitySubcharacteristic`, subcharacteristics),
    ...listDef('requirementRecords', `${Q}::QualityRequirement`, requirements),
    ...listDef('measureRecords', `${Q}::QualityMeasure`, measures),
    definition('namesNumbered', 'Bool', `(match names
    | List.nil => true
    | List.cons item rest => (((project item index) =? position) && (namesNumbered (position + 1) rest)))`, { parameters: ['position : Nat', 'names : (List Named)'], recursive: 'names' }),
    known('parentsKnown', 'QualitySubcharacteristic', 'characteristicIndex'),
    known('requirementsKnown', 'QualityRequirement', 'subcharacteristicIndex'),
    known('measuresKnown', 'QualityMeasure', 'requirementIndex'),
    definition('thresholdOf', 'Nat', `(match records
    | List.nil => 0
    | List.cons head rest => (if ((project head index) =? wanted) then (project head threshold) else (thresholdOf wanted rest)))`, { parameters: ['wanted : Nat', `records : (List ${Q}::QualityMeasure)`], recursive: 'records' }),
    // The whole ISO/IEC 25010:2023 product quality model, and its coverage:
    // a subcharacteristic is stated by a requirement or listed open, never both.
    ...nats('openSubcharacteristics', openSubcharacteristics.map((x) => x.index)),
    listLength('namesLength', 'Named'),
    listLength('natsLength', 'Nat'),
    definition('hasRequirement', 'Bool', `(match records
    | List.nil => false
    | List.cons item rest => (((project item subcharacteristicIndex) =? wanted) || (hasRequirement wanted rest)))`, { parameters: ['wanted : Nat', `records : (List ${Q}::QualityRequirement)`], recursive: 'records' }),
    definition('natListed', 'Bool', `(match values
    | List.nil => false
    | List.cons value rest => ((value =? wanted) || (natListed wanted rest)))`, { parameters: ['wanted : Nat', 'values : (List Nat)'], recursive: 'values' }),
    exactlyOne,
    definition('coverageStated', 'Bool', `(match names
    | List.nil => true
    | List.cons item rest => ((exactlyOne (hasRequirement position (requirementRecords)) (natListed position (openSubcharacteristics))) && (coverageStated (position + 1) rest)))`, { parameters: ['position : Nat', 'names : (List Named)'], recursive: 'names' }),
    definition(
      'qualityValid',
      'Bool',
      conj([
        `((namesLength (characteristicNames)) =? ${characteristics.length})`,
        `((namesLength (subcharacteristicNames)) =? ${subcharacteristics.length})`,
        '(coverageStated 0 (subcharacteristicNames))',
        '(namesNumbered 0 (characteristicNames))',
        '(namesNumbered 0 (subcharacteristicNames))',
        '(namesNumbered 0 (requirementNames))',
        '(namesNumbered 0 (measureNames))',
        `(parentsKnown ${characteristics.length} (subcharacteristicRecords))`,
        `(requirementsKnown ${subcharacteristics.length} (requirementRecords))`,
        `(measuresKnown ${requirements.length} (measureRecords))`,
      ]),
    ),
    theorem('qualityModelHolds', '((qualityValid) = true)'),
    theorem('openSubcharacteristicCount', `((natsLength (openSubcharacteristics)) = ${openSubcharacteristics.length})`),
    // Each threshold, pinned by a theorem, so a weakened measure fails the kernel.
    // The SDK reads a threshold as a literal; one that is the behavior's own number (thresholdTerm) is
    // proved equal to that term, so the behavior stays its only source.
    ...measures.map((x) => theorem(`${x.decl}Threshold`, `((thresholdOf ${x.index} (measureRecords)) = ${x.thresholdTerm ?? x.threshold})`)),
  ];
  return moduleText({ name: 'Quality', glossaries: [BOOL, NAT, ARCH, QUAL], imports: ['Foundation.Qual', ...(m.quality.imports ?? [])], title: 'quality measure', declarations: decls });
}

export function compliance(m) {
  const decls = [
    structure('Obligation', ['{name: index, type: Nat}', '{name: standard, type: String}', '{name: subject, type: String}', '{name: text, type: String}', '{name: evidence, type: (List String)}']),
    ...listDefinitions('obligations', 'Obligation', m.compliance.map((o, index) => rec('Obligation', { index, standard: str(o.standard), subject: str(o.subject), text: str(o.text), evidence: cons('String', o.evidence.map(str)) }))),
    // A separate definition: a match nested in a term is rewritten by
    // lexlean fmt into a form its parser refuses.
    definition('hasEvidence', 'Bool', `(match evidence | List.nil => false | List.cons item others => true)`, { parameters: ['evidence : (List String)'] }),
    definition('obligationsEvidenced', 'Bool', `(match items
    | List.nil => true
    | List.cons item rest => (((project item index) =? position) && ((hasEvidence (project item evidence)) && (obligationsEvidenced (position + 1) rest))))`, { parameters: ['position : Nat', 'items : (List Obligation)'], recursive: 'items' }),
    theorem('everyObligationEvidenced', '((obligationsEvidenced 0 (obligations)) = true)'),
  ];
  return moduleText({ name: 'Compliance', glossaries: [BOOL, NAT, ARCH], imports: ['Architecture', 'Design', 'Quality', 'Security', ...(m.modules ?? [m.module])], title: 'standards reference', declarations: decls });
}

/** The facet modules, by path under app/. */
export function facetModules(m) {
  return {
    'src/Architecture.lex.tex': architecture(m),
    'src/Compliance.lex.tex': compliance(m),
    'src/Design.lex.tex': design(m),
    'src/Features.lex.tex': features(m),
    'src/Quality.lex.tex': quality(m),
    'src/Security.lex.tex': security(m),
  };
}
