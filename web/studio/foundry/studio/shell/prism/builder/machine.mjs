/**
 * A state machine's decision, owned by its states.
 *
 * In an OMG UML 2.5.1 behavioral state machine every transition has a source
 * vertex (section 14.2.3.8): the machine first finds the transitions of the
 * state it is in, then fires the first whose trigger and guard hold. A flat
 * first-match chain over every rule computes the same answer, but it asks the
 * prover to split one `if` per rule of the whole machine. Factored by source
 * state, `judge` tests the state once and hands the request to that state's own
 * chain, which holds only the rules that can fire from it, in their order.
 *
 * The answer is unchanged for every request: a rule left out of a state's chain
 * has a condition `state is t` with t another state, or `state is not s`, so it
 * cannot hold there; and the chain past the last state keeps exactly the rules
 * that name no state with `is`, for a state value no state has (the decoder
 * never yields one). The other declarations are prism-code's own, so the guest
 * and the proofs still come from one model.
 */
import { declarations, declarationText, names, proofDeclarations } from '../render.mjs';

const pascal = (s) => s.split(/[^A-Za-z0-9]+/).filter(Boolean).map((w) => w[0].toUpperCase() + w.slice(1)).join('');

/** Whether rule `r` can fire when the state field holds `value` (null: a value no state has). */
function canFire(r, field, value) {
  return r.when.every((c) => c.field !== field || (c.op === 'is' ? c.value === value : c.op === 'is not' ? c.value !== value : true));
}

/** The state-owned decision: one chain per state and the dispatch on the state, replacing the flat `judge`. */
export function machineDeclarations(spec, machine, n = names(spec)) {
  const decls = declarations(spec, n);
  const at = decls.findIndex((d) => d.name === 'judge');
  const judge = decls[at];
  const taken = new Set(decls.map((d) => d.name));
  const claim = (base) => {
    let name = base;
    for (let i = 2; taken.has(name); i++) name = `${base}${i}`;
    taken.add(name);
    return name;
  };
  const field = spec.fields.find((f) => f.name === machine.state);
  const chain = (rules) => rules.reduceRight((rest, r) => `(if (${n.rules[r.id]} request) then (${n.answers[r.then]}) else ${rest})`, `(${n.answers[spec.otherwise]})`);
  const steps = field.values.map((value) => ({ value, name: claim(`step${pascal(value)}`), rules: spec.rules.filter((r) => canFire(r, field.name, value)) }));
  const stateOf = `(project request ${n.fields[field.name].id})`;
  const rest = spec.rules.filter((r) => canFire(r, field.name, null));
  const body = steps.reduceRight((tail, s) => `(if (${stateOf} =? (${n.fields[field.name].choices[s.value]})) then (${s.name} request) else ${tail})`, chain(rest));
  const stepDecls = steps.map((s) => ({ ...judge, name: s.name, body: chain(s.rules) }));
  return {
    decls: [...decls.slice(0, at), ...stepDecls, { ...judge, body }, ...decls.slice(at + 1)],
    steps: steps.map((s) => ({ value: s.value, name: s.name, rules: s.rules.map((r) => r.id) })),
    rest: rest.map((r) => r.id),
    n,
  };
}

/** The proofs module's declarations: each theorem also unfolds the states' chains. */
export function machineProofDeclarations(spec, machine, n = names(spec)) {
  const { steps } = machineDeclarations(spec, machine, n);
  const q = (name) => `${n.module}::${name}`;
  return proofDeclarations(spec, n).map((d) => (d.kind === 'theorem' ? { ...d, unfold: [...new Set([...d.unfold, ...steps.map((s) => q(s.name))])].sort() } : d));
}

// The module text exactly as prism-code's renderer lays it out.
const moduleText = (name, imports, decls) =>
  [
    `\\begin{lexlean}{${name}}`,
    '\\useglossary{lexlean.std.bool@1.2.0}',
    '\\useglossary{lexlean.std.nat@1.2.0}',
    '\\useglossary{prism.arch@1.0.0}',
    ...imports.map((m) => `\\importmodule{${m}}`),
    '\\title{Text application model}',
    '\\begin{semanticmodule}',
    decls.map(declarationText).join('\n\n'),
    '\\end{semanticmodule}',
    '\\end{lexlean}',
    '',
  ].join('\n');

const withAxioms = (decls, axioms) => decls.map((d) => (Object.hasOwn(axioms, d.name) ? { ...d, axioms: axioms[d.name] } : d));

export function renderMachineModel(spec, machine, { axioms = {} } = {}) {
  const n = names(spec);
  return moduleText(n.module, ['Foundation.Bytes', 'Foundation.View.Text.V1.Model'], withAxioms(machineDeclarations(spec, machine, n).decls, axioms));
}

export function renderMachineProofs(spec, machine, { axioms = {} } = {}) {
  const n = names(spec);
  if (spec.properties.length === 0) return null;
  return moduleText(n.proofs, [n.module], withAxioms(machineProofDeclarations(spec, machine, n), axioms));
}
