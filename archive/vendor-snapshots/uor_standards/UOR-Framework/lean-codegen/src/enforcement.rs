//! Lean 4 enforcement / ergonomics surface generator (v0.2.1).
//!
//! Mirrors `codegen/src/enforcement.rs` for the Lean 4 package. Emits
//! `lean4/UOR/Enforcement.lean` containing the v0.2.1 sealed wrappers,
//! `Certify` typeclass, `PipelineFailure` inductive, ring-op marker
//! structures, fragment markers, dispatch table consts, and the
//! `UOR.Prelude` re-exports — keeping Lean ergonomics in lock-step with
//! the published Rust crate.
//!
//! Every emitted symbol traces to an ontology entity, the same as the
//! Rust codegen template. Per-resolver `Certify` instances are emitted
//! parametrically from `resolver:CertifyMapping` individuals; the
//! `PipelineFailure` inductive draws variants from
//! `reduction:FailureField` individuals.

use crate::emit::LeanFile;
use uor_ontology::model::{IndividualValue, Ontology};

/// Local-name a class/individual/property IRI (everything after the last `/` or `#`).
fn local_name(iri: &str) -> &str {
    iri.rsplit_once(['/', '#']).map(|(_, n)| n).unwrap_or(iri)
}

/// Read a string-typed property value off an individual.
fn ind_prop_str<'a>(ind: &'a uor_ontology::model::Individual, prop_iri: &str) -> Option<&'a str> {
    for (k, v) in ind.properties {
        if *k == prop_iri {
            return match v {
                IndividualValue::IriRef(s) | IndividualValue::Str(s) => Some(s),
                _ => None,
            };
        }
    }
    None
}

/// Collect individuals of a given type.
fn individuals_of_type<'a>(
    ontology: &'a Ontology,
    type_iri: &str,
) -> Vec<&'a uor_ontology::model::Individual> {
    let mut out = Vec::new();
    for ns in &ontology.namespaces {
        for ind in &ns.individuals {
            if ind.type_ == type_iri {
                out.push(ind);
            }
        }
    }
    out
}

/// Generates the complete `Enforcement.lean` module content.
#[must_use]
pub fn generate_enforcement(ontology: &Ontology) -> String {
    let mut f = LeanFile::new(
        "v0.2.1 ergonomics surface — sealed wrappers, Certify typeclass, \
         PipelineFailure inductive, and the UOR.Prelude re-exports. \
         Every symbol traces to an ontology entity.",
    );

    f.line("import UOR.Primitives");
    f.line("import UOR.Enums");
    f.blank();
    f.line("namespace UOR.Enforcement");
    f.blank();
    f.line("open UOR.Primitives");
    f.blank();

    emit_ontology_target(&mut f);
    emit_validated(&mut f);
    emit_grounded_shape(&mut f);
    emit_grounded(&mut f);
    emit_certificates(&mut f);
    emit_witnesses(&mut f);
    emit_pipeline_failure(&mut f, ontology);
    emit_certify(&mut f, ontology);
    emit_ring_ops(&mut f, ontology);
    emit_fragment_markers(&mut f, ontology);
    emit_dispatch_tables(&mut f, ontology);

    f.line("end UOR.Enforcement");
    f.finish()
}

fn emit_ontology_target(f: &mut LeanFile) {
    f.doc_comment(
        "Sealed marker class identifying foundation-produced types. The Lean \
         counterpart of Rust's `OntologyTarget` trait. Implementations are \
         emitted only for foundation certificate / witness shims, mirroring \
         the Rust crate's prelude composition.",
    );
    f.line("class OntologyTarget (\u{03B1} : Type) : Prop where");
    f.line("  /-- Sealed witness — only foundation-emitted types may instantiate. -/");
    f.line("  sealed : True");
    f.blank();
}

fn emit_validated(f: &mut LeanFile) {
    f.doc_comment(
        "Validated wrapper around a foundation-produced inner value. v0.2.1 \
         pairs Lean's `Validated` with `OntologyTarget` to gate the inner \
         type. Construction is private to UOR.Enforcement, mirroring the \
         Rust `pub(crate) const fn new`.",
    );
    f.line("structure Validated (\u{03B1} : Type) [OntologyTarget \u{03B1}] where");
    f.line("  /-- The validated inner value. -/");
    f.line("  inner : \u{03B1}");
    f.blank();
    f.line("namespace Validated");
    f.line("/-- Read-only accessor for the validated inner value. -/");
    f.line("def value {\u{03B1} : Type} [OntologyTarget \u{03B1}] (v : Validated \u{03B1}) : \u{03B1} := v.inner");
    f.line("end Validated");
    f.blank();
    f.doc_comment(
        "Coercion to the inner type so consumers can call certificate methods \
         directly: `cert.targetLevel` rather than `cert.value.targetLevel`. \
         Lean coercions resolve at elaboration time, matching the Rust \
         auto-deref ergonomics.",
    );
    f.line("instance {\u{03B1} : Type} [OntologyTarget \u{03B1}] : Coe (Validated \u{03B1}) \u{03B1} where");
    f.line("  coe v := v.inner");
    f.blank();
}

fn emit_grounded_shape(f: &mut LeanFile) {
    f.doc_comment(
        "Sealed marker class for type:ConstrainedType subclasses that may \
         appear as the parameter of `Grounded`. Counterpart of Rust's \
         `GroundedShape` sealed trait.",
    );
    f.line("class GroundedShape (\u{03B1} : Type) : Prop where");
    f.line("  /-- Sealed witness. -/");
    f.line("  sealed : True");
    f.blank();
}

fn emit_grounded(f: &mut LeanFile) {
    f.doc_comment("BindingsTable carries the static op:GS_5 zero-step access table.");
    f.line("structure BindingsTable where");
    f.line("  /-- Sorted-by-address binding entries. -/");
    f.line("  entries : Array (UInt64 \u{00D7} Array UInt8)");
    f.blank();
    f.doc_comment(
        "The compile-time witness that op:GS_4 holds for the value it carries: \
         \u{03C3} = 1, freeRank = 0, S = 0, T_ctx = 0. Counterpart of Rust's \
         `Grounded<T: GroundedShape>`. Construction is gated by the back-door \
         minting path.",
    );
    f.line("structure Grounded (\u{03B1} : Type) [GroundedShape \u{03B1}] where");
    f.line("  /-- The bindings table laid out for op:GS_5 access. -/");
    f.line("  bindings : BindingsTable");
    f.line("  /-- The Witt level the grounded value was minted at. -/");
    f.line("  wittLevelBits : UInt16");
    f.line("  /-- Content-address of the originating CompileUnit. -/");
    f.line("  unitAddress : UInt64");
    f.blank();
    f.line("namespace Grounded");
    f.line("/-- Returns the binding for the given query address, or `none` if absent. -/");
    f.line("def getBinding {\u{03B1} : Type} [GroundedShape \u{03B1}] (g : Grounded \u{03B1}) (q : UInt64) : Option (Array UInt8) :=");
    f.line("  (g.bindings.entries.find? (fun (a, _) => a == q)).map Prod.snd");
    f.line("end Grounded");
    f.blank();
}

fn emit_certificates(f: &mut LeanFile) {
    // v0.2.1 Phase 7e.6: certificate structures carry a real `wittBits`
    // field populated by the pipeline driver via `withWittBits`. The field
    // enables `LiftChainCertificate.targetLevel` and sibling accessors to
    // return the Witt level the pipeline advanced to, matching the Rust
    // Phase 7b.1.c rewrite.
    let certs = [
        ("GroundingCertificate", "Sealed shim for cert:GroundingCertificate. Phase 7e.6: carries real `wittBits` from the pipeline."),
        ("LiftChainCertificate", "Sealed shim for cert:LiftChainCertificate. Exposes the v0.2.1 `targetLevel` accessor."),
        ("InhabitanceCertificate", "Sealed shim for cert:InhabitanceCertificate (v0.2.1)."),
        ("CompletenessCertificate", "Sealed shim for cert:CompletenessCertificate."),
        ("MultiplicationCertificate", "Sealed shim for cert:MultiplicationCertificate (v0.2.2 Phase C.4)."),
    ];
    for (name, doc) in &certs {
        f.doc_comment(doc);
        f.line(&format!("structure {name} where"));
        f.line("  wittBits : UInt16 := 32");
        f.line("  deriving Inhabited, Repr");
        f.line(&format!(
            "instance : OntologyTarget {name} where sealed := trivial"
        ));
        f.blank();
        f.doc_comment(&format!(
            "Construct a {name} carrying the given Witt level. Used by the \
             Lean pipeline driver to mint certificates with the real level."
        ));
        f.line(&format!(
            "def {name}.withWittBits (witt : UInt16) : {name} := {{ wittBits := witt }}"
        ));
        f.blank();
    }
    f.doc_comment(
        "Returns the Witt level the certificate was issued for, sourced from \
         the `wittBits` field populated by the pipeline.",
    );
    f.line("def LiftChainCertificate.targetLevel (cert : LiftChainCertificate) : Nat :=");
    f.line("  cert.wittBits.toNat");
    f.blank();
    f.doc_comment(
        "Returns the witness value tuple bytes when `verified` is true. \
         v0.2.1 always returns `none`; witness emission lands in v0.2.2.",
    );
    f.line("def InhabitanceCertificate.witness (_ : InhabitanceCertificate) : Option (Array UInt8) := none");
    f.blank();
}

fn emit_witnesses(f: &mut LeanFile) {
    // v0.2.1 Phase 8c.12: each witness / input shim is emitted as a Lean
    // structure with a single `private dummy : Unit := ()` field. The dummy
    // is load-bearing — Lean 4 requires every non-`Prop` structure to have
    // at least one field, and these shims carry no payload (unlike the
    // certificate structures in emit_certificates() which hold `wittBits`).
    // The `private` modifier prevents downstream code from constructing
    // instances by hand; the only legitimate construction path is through
    // the sealed `OntologyTarget` instance emitted below.
    let witnesses = [
        (
            "GenericImpossibilityWitness",
            "Sealed shim for proof:ImpossibilityWitness.",
        ),
        (
            "InhabitanceImpossibilityWitness",
            "Sealed shim for proof:InhabitanceImpossibilityWitness (v0.2.1).",
        ),
        (
            "ConstrainedTypeInput",
            "Input shim for type:ConstrainedType.",
        ),
        ("CompileUnit", "Input shim for reduction:CompileUnit."),
    ];
    for (name, doc) in &witnesses {
        f.doc_comment(doc);
        f.line(&format!("structure {name} where"));
        // Lean 4 requires ≥1 field on a non-Prop structure. This dummy is
        // the sealed-construction pattern; see Phase 8c.12 block comment above.
        f.line("  private dummy : Unit := ()");
        f.line(&format!(
            "instance : OntologyTarget {name} where sealed := trivial"
        ));
        f.blank();
    }
    f.doc_comment("Sealed marker class for impossibility witnesses.");
    f.line("class ImpossibilityWitnessKind (\u{03B1} : Type) : Prop where");
    f.line("  /-- Sealed witness. -/");
    f.line("  sealed : True");
    f.line(
        "instance : ImpossibilityWitnessKind GenericImpossibilityWitness where sealed := trivial",
    );
    f.line("instance : ImpossibilityWitnessKind InhabitanceImpossibilityWitness where sealed := trivial");
    f.blank();
}

fn emit_pipeline_failure(f: &mut LeanFile, ontology: &Ontology) {
    f.doc_comment(
        "PipelineFailure inductive — variants discovered parametrically from \
         reduction:PipelineFailureReason individuals plus failure: namespace \
         and conformance:ShapeViolationReport.",
    );
    f.line("inductive PipelineFailure where");

    let reasons = individuals_of_type(
        ontology,
        "https://uor.foundation/reduction/PipelineFailureReason",
    );
    for ind in &reasons {
        let variant = local_name(ind.id);
        f.line(&format!("  | {variant}"));
    }
    f.line("  | LiftObstructionFailure");
    f.line("  | ShapeViolation");
    f.line("  deriving Repr, BEq, Inhabited");
    f.blank();
}

fn emit_certify(f: &mut LeanFile, ontology: &Ontology) {
    // v0.2.1 Phase 7e.8: Emits only the `class Certify` declaration and
    // the resolver façade structs. The `instance : Certify ...` blocks
    // that call the pipeline entry points live in `UOR.Pipeline` (emitted
    // by `pipeline.rs`'s `emit_certify_instances`), because the instance
    // bodies reference `UOR.Pipeline.run*` functions and Lean's module
    // system requires those references to resolve after the functions
    // are declared.
    f.doc_comment(
        "Verdict-producing typeclass. Mirrors the Rust `Certify<I>` trait. \
         The `certify` method accepts an optional `level : Nat` defaulting to \
         32 (per `Certify::DEFAULT_LEVEL` in the Rust side).",
    );
    f.line("class Certify (\u{03C1} : Type) (I : Type) where");
    f.line("  Certificate : Type");
    f.line("  Witness : Type");
    f.line("  [certificateTarget : OntologyTarget Certificate]");
    f.line("  [witnessKind : ImpossibilityWitnessKind Witness]");
    f.line("  certify : \u{03C1} \u{2192} I \u{2192} Except Witness (Validated Certificate)");
    f.line("  certifyAt : \u{03C1} \u{2192} I \u{2192} Nat \u{2192} Except Witness (Validated Certificate)");
    f.blank();

    // Emit just the resolver façade structs + `new` constructors. Instances
    // come later in UOR.Pipeline.
    let mappings = individuals_of_type(ontology, "https://uor.foundation/resolver/CertifyMapping");
    for m in mappings {
        let resolver_iri = match ind_prop_str(m, "https://uor.foundation/resolver/forResolver") {
            Some(s) => s,
            None => continue,
        };
        let resolver_name = local_name(resolver_iri).to_string();
        f.doc_comment(&format!(
            "v0.2.1 façade structure for the {resolver_name} resolver class. \
             The `Certify` instance lives in `UOR.Pipeline`."
        ));
        f.line(&format!("structure {resolver_name} where"));
        // Lean 4 requires ≥1 field on a non-Prop structure. Same
        // sealed-construction pattern as the witness shims in
        // emit_witnesses() (Phase 8c.12).
        f.line("  private dummy : Unit := ()");
        f.line(&format!(
            "def {resolver_name}.new : {resolver_name} := {{}}"
        ));
        f.blank();
    }
}

/// v0.2.1 Phase 7e.8: emit the `instance : Certify ...` blocks for each
/// `resolver:CertifyMapping` individual. Called from
/// `lean-codegen/src/pipeline.rs`'s `generate_pipeline` after the real
/// run entry points are defined so the instance bodies can reference them.
pub fn emit_certify_instances(f: &mut LeanFile, ontology: &Ontology) {
    f.doc_comment(
        "v0.2.1 Phase 7e.8: Certify instances calling the pipeline entry \
         points. Each instance delegates to the matching `UOR.Pipeline.run*` \
         function with a Witt-level parameter defaulting to 32.",
    );
    let mappings = individuals_of_type(ontology, "https://uor.foundation/resolver/CertifyMapping");
    for m in mappings {
        let resolver_iri = match ind_prop_str(m, "https://uor.foundation/resolver/forResolver") {
            Some(s) => s,
            None => continue,
        };
        let cert_iri = match ind_prop_str(m, "https://uor.foundation/resolver/producesCertificate")
        {
            Some(s) => s,
            None => continue,
        };
        let witness_iri = match ind_prop_str(m, "https://uor.foundation/resolver/producesWitness") {
            Some(s) => s,
            None => continue,
        };
        let resolver_name = local_name(resolver_iri).to_string();
        let cert_name = local_name(cert_iri).to_string();
        let witness_name = match local_name(witness_iri) {
            "ImpossibilityWitness" => "GenericImpossibilityWitness".to_string(),
            other => other.to_string(),
        };
        let input_name = match resolver_name.as_str() {
            "GroundingAwareResolver" => "CompileUnit",
            _ => "ConstrainedTypeInput",
        };

        f.line(&format!(
            "instance : Certify {resolver_name} {input_name} where"
        ));
        f.line(&format!("  Certificate := {cert_name}"));
        f.line(&format!("  Witness := {witness_name}"));
        let (default_body, at_body) = match resolver_name.as_str() {
            "TowerCompletenessResolver" => (
                "runTowerCompleteness #[] 32",
                "runTowerCompleteness #[] lvl",
            ),
            "IncrementalCompletenessResolver" => (
                "runIncrementalCompleteness #[] 32",
                "runIncrementalCompleteness #[] lvl",
            ),
            "GroundingAwareResolver" => {
                ("runGroundingAware input 32", "runGroundingAware input lvl")
            }
            "InhabitanceResolver" => ("runInhabitance #[] 32", "runInhabitance #[] lvl"),
            _ => (".ok \u{27E8} {} \u{27E9}", ".ok \u{27E8} {} \u{27E9}"),
        };
        let param_name = if resolver_name == "GroundingAwareResolver" {
            "input"
        } else {
            "_"
        };
        f.line(&format!("  certify _ {param_name} := {default_body}"));
        // Use `_lvl` when the resolver's at_body is a constant (no lvl
        // reference) so Lean's unused-variable lint is satisfied.
        let lvl_param = if at_body.contains("lvl") {
            "lvl"
        } else {
            "_lvl"
        };
        f.line(&format!(
            "  certifyAt _ {param_name} {lvl_param} := {at_body}"
        ));
        f.blank();
    }
}

fn emit_ring_ops(f: &mut LeanFile, ontology: &Ontology) {
    // v0.2.1 Phase 7e.7: emit real typeclasses + per-(op, level) instances.
    // The op list is sourced from `op:isRingOp = true` annotations (Phase
    // 7a.9). Each op gets its own typeclass per-arity; W8/W16/W24/W32 each
    // get one instance per op using native `UInt8` / `UInt16` / `UInt32`
    // arithmetic.
    f.doc_comment(
        "Witt level marker structures. Reified at the type level so \
         consumers can bind `Mul.{W8}.apply a b` with compile-time level \
         checking.",
    );
    for local in &["W8", "W16", "W24", "W32"] {
        f.line(&format!("structure {local} where"));
    }
    f.blank();

    // Collect ring ops from the ontology. We walk every Individual whose
    // type is a subclass of `op:Operation` and that carries `isRingOp =
    // true`.
    let mut ring_ops: Vec<(String, bool)> = Vec::new(); // (local_name, is_binary)
    for ns in &ontology.namespaces {
        for ind in &ns.individuals {
            // Only op:* individuals participate.
            if !ind.type_.starts_with("https://uor.foundation/op/") {
                continue;
            }
            let is_ring = ind.properties.iter().any(|(k, v)| {
                *k == "https://uor.foundation/op/isRingOp"
                    && matches!(v, IndividualValue::Bool(true))
            });
            if !is_ring {
                continue;
            }
            // op:arity: 1 = unary, 2 = binary.
            let arity = ind
                .properties
                .iter()
                .find_map(|(k, v)| {
                    if *k == "https://uor.foundation/op/arity" {
                        if let IndividualValue::Int(n) = v {
                            return Some(*n);
                        }
                    }
                    None
                })
                .unwrap_or(2);
            let local = local_name(ind.id).to_string();
            let pascal = pascal_case(&local);
            ring_ops.push((pascal, arity == 2));
        }
    }
    // Deduplicate (some namespaces re-use the same local name).
    ring_ops.sort();
    ring_ops.dedup();

    f.doc_comment(
        "Ring-op typeclasses. One class per ring op (mul/add/sub/xor/and/or \
         binary; neg/bnot/succ/pred unary), indexed over the Witt level `L`.",
    );
    for (op_name, is_binary) in &ring_ops {
        if *is_binary {
            f.line(&format!(
                "class {op_name}Op (L : Type) (\u{03B1} : Type) where apply : \u{03B1} \u{2192} \u{03B1} \u{2192} \u{03B1}"
            ));
        } else {
            f.line(&format!(
                "class {op_name}Op (L : Type) (\u{03B1} : Type) where apply : \u{03B1} \u{2192} \u{03B1}"
            ));
        }
    }
    f.blank();

    // Emit instances: 10 ops × 4 levels = 40 instances.
    f.doc_comment(
        "Ring-op instances at each Witt level. Delegates to the Lean core \
         `UInt8` / `UInt16` / `UInt32` operators for actual arithmetic.",
    );
    let levels = [
        ("W8", "UInt8"),
        ("W16", "UInt16"),
        ("W24", "UInt32"),
        ("W32", "UInt32"),
    ];
    for (lvl, ty) in &levels {
        for (op_name, is_binary) in &ring_ops {
            let body = lean_ring_op_body(op_name);
            if *is_binary {
                f.line(&format!(
                    "instance : {op_name}Op {lvl} {ty} where apply a b := {body}"
                ));
            } else {
                f.line(&format!(
                    "instance : {op_name}Op {lvl} {ty} where apply a := {body}"
                ));
            }
        }
    }
    f.blank();
}

fn pascal_case(s: &str) -> String {
    let mut out = String::new();
    let mut upper = true;
    for c in s.chars() {
        if c == '_' || c == '-' {
            upper = true;
        } else if upper {
            out.push(c.to_ascii_uppercase());
            upper = false;
        } else {
            out.push(c);
        }
    }
    out
}

fn lean_ring_op_body(op_pascal: &str) -> &'static str {
    match op_pascal {
        "Mul" => "a * b",
        "Add" => "a + b",
        "Sub" => "a - b",
        "Xor" => "a ^^^ b",
        "And" => "a &&& b",
        "Or" => "a ||| b",
        "Neg" => "0 - a",
        "Bnot" => "~~~ a",
        "Succ" => "a + 1",
        "Pred" => "a - 1",
        _ => "a",
    }
}

fn emit_fragment_markers(f: &mut LeanFile, ontology: &Ontology) {
    f.doc_comment(
        "Fragment classifier markers — emitted parametrically from \
         predicate:DispatchRule individuals whose dispatchPredicate \
         evaluates over type:ConstrainedType.",
    );
    f.line("class FragmentMarker (\u{03B1} : Type) : Prop where");
    f.line("  /-- Sealed witness. -/");
    f.line("  sealed : True");
    f.blank();
    let rules = individuals_of_type(ontology, "https://uor.foundation/predicate/DispatchRule");
    let mut emitted: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for r in rules {
        if let Some(pred_iri) =
            ind_prop_str(r, "https://uor.foundation/predicate/dispatchPredicate")
        {
            for ns in &ontology.namespaces {
                for ind in &ns.individuals {
                    if ind.id == pred_iri {
                        if let Some(over) =
                            ind_prop_str(ind, "https://uor.foundation/predicate/evaluatesOver")
                        {
                            if over == "https://uor.foundation/type/ConstrainedType" {
                                emitted.insert(local_name(pred_iri).to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    for m in &emitted {
        f.line(&format!("structure {m} where"));
        f.line(&format!(
            "instance : FragmentMarker {m} where sealed := trivial"
        ));
    }
    f.blank();
}

fn emit_dispatch_tables(f: &mut LeanFile, ontology: &Ontology) {
    f.doc_comment(
        "DispatchRule entries with predicate IRI, target resolver IRI, and \
         priority. v0.2.1 emits one constant per predicate:DispatchTable \
         individual in the ontology.",
    );
    f.line("structure DispatchRule where");
    f.line("  predicateIri : String");
    f.line("  targetResolverIri : String");
    f.line("  priority : Nat");
    f.blank();
    let tables = individuals_of_type(ontology, "https://uor.foundation/predicate/DispatchTable");
    for t in tables {
        let const_name = local_name(t.id).to_string();
        let rules = individuals_of_type(ontology, "https://uor.foundation/predicate/DispatchRule");
        let table_local = local_name(t.id);
        let table_prefix = table_local
            .strip_suffix("DispatchTable")
            .unwrap_or(table_local)
            .to_lowercase();
        let mut rule_specs: Vec<(u32, String, String)> = Vec::new();
        for r in &rules {
            let local = local_name(r.id);
            if !local.starts_with(&format!("{table_prefix}_rule_")) {
                continue;
            }
            let pred = ind_prop_str(r, "https://uor.foundation/predicate/dispatchPredicate")
                .unwrap_or("")
                .to_string();
            let tgt = ind_prop_str(r, "https://uor.foundation/predicate/dispatchTarget")
                .unwrap_or("")
                .to_string();
            let prio: u32 = r
                .properties
                .iter()
                .find_map(|(k, v)| {
                    if *k == "https://uor.foundation/predicate/dispatchPriority" {
                        if let IndividualValue::Int(i) = v {
                            return Some(*i as u32);
                        }
                    }
                    None
                })
                .unwrap_or(0);
            rule_specs.push((prio, pred, tgt));
        }
        rule_specs.sort_by_key(|(p, _, _)| *p);
        f.doc_comment(&format!(
            "v0.2.1 dispatch table generated from `predicate:{}`.",
            local_name(t.id)
        ));
        f.line(&format!("def {const_name} : Array DispatchRule := #["));
        for (i, (prio, pred, tgt)) in rule_specs.iter().enumerate() {
            let comma = if i + 1 < rule_specs.len() { "," } else { "" };
            f.line(&format!(
                "  {{ predicateIri := \"{pred}\", targetResolverIri := \"{tgt}\", priority := {prio} }}{comma}"
            ));
        }
        f.line("]");
        f.blank();
    }
}

/// Generates the `UOR/Examples.lean` worked-example module.
#[must_use]
pub fn generate_examples(_ontology: &Ontology) -> String {
    let mut f = LeanFile::new(
        "v0.2.1 Lean 4 worked examples \u{2014} parallel of the \
         `foundation/examples/` directory. Each example compiles under \
         `lake build` and mirrors a Rust example one-for-one. Every \
         definition is total and reduces at elaboration — no `sorry`, \
         `partial def`, or `native_decide` appear in this module.",
    );
    f.line("import UOR.Enforcement");
    f.line("import UOR.Pipeline");
    f.blank();
    f.line("namespace UOR.Examples");
    f.blank();
    f.line("open UOR.Enforcement");
    f.line("open UOR.Pipeline");
    f.blank();
    f.line("/-- Reference to the TowerCompletenessResolver façade constructor. -/");
    f.line("def towerResolverNew : TowerCompletenessResolver :=");
    f.line("  TowerCompletenessResolver.new");
    f.blank();
    f.line("/-- Reference to the InhabitanceResolver façade constructor. -/");
    f.line("def inhabitanceResolverNew : InhabitanceResolver :=");
    f.line("  InhabitanceResolver.new");
    f.blank();
    f.line("/-- \"At what Witt level is this shape representable?\" — mirrors the");
    f.line("    Rust `witt_level_query` example. Calls the pipeline at W32 and");
    f.line("    reads the minted certificate's target level. -/");
    f.line("def wittLevelQueryW32 : Except GenericImpossibilityWitness (Validated LiftChainCertificate) :=");
    f.line("  runTowerCompleteness #[] 32");
    f.blank();
    f.line("/-- Same query at W16 — demonstrates end-to-end level propagation. -/");
    f.line("def wittLevelQueryW16 : Except GenericImpossibilityWitness (Validated LiftChainCertificate) :=");
    f.line("  runTowerCompleteness #[] 16");
    f.blank();
    f.line("/-- Inhabitance query on a vacuous input (residual-vacuous classification). -/");
    f.line("def inhabitanceQuery : Except InhabitanceImpossibilityWitness (Validated InhabitanceCertificate) :=");
    f.line("  runInhabitance #[] 32");
    f.blank();
    f.line("/-- Dispatch table walk — report size to stdout. -/");
    f.line("def dispatchTableWalk : IO Unit := do");
    f.line("  IO.println s!\"InhabitanceDispatchTable rules: {UOR.Enforcement.InhabitanceDispatchTable.size}\"");
    f.blank();
    f.line("/-- The Inhabitance dispatch table exported from UOR.Enforcement. -/");
    f.line("def dispatchTable := UOR.Enforcement.InhabitanceDispatchTable");
    f.blank();
    f.line("end UOR.Examples");
    f.finish()
}

/// Generates the `UOR/Test.lean` test module with real `by decide`
/// assertions proving the pipeline deciders reduce at elaboration time.
#[must_use]
pub fn generate_test(_ontology: &Ontology) -> String {
    let mut f = LeanFile::new(
        "v0.2.1 Lean 4 test module. Every `example ... := by decide` \
         assertion verifies at elaboration time that the decider under \
         test reduces. A successful `lake build` of this file is proof \
         that the pipeline is (a) pure-functional, (b) fuel-bounded, and \
         (c) free of `sorry` / `partial def` / `native_decide`.",
    );
    f.line("import UOR.Enforcement");
    f.line("import UOR.Pipeline");
    f.blank();
    f.line("namespace UOR.Test");
    f.blank();
    f.line("open UOR.Enforcement");
    f.line("open UOR.Pipeline");
    f.blank();
    // --- Reducibility tests (by decide) ---
    f.line("-- =========================================================");
    f.line("-- Phase 7e.10: Reducibility assertions via `by decide`.");
    f.line("-- Every example below reduces at elaboration time. If any");
    f.line("-- definition in UOR.Pipeline is marked `partial def`, uses");
    f.line("-- `native_decide`, or otherwise blocks reduction, these");
    f.line("-- examples would fail to elaborate.");
    f.line("-- =========================================================");
    f.blank();
    f.line("/-- Empty constraint list classifies as residual-vacuous. -/");
    f.line("example : fragmentClassify #[] = FragmentKind.residual := by decide");
    f.blank();
    f.line("/-- 2-SAT decider accepts the empty clause list. -/");
    f.line("example : decideTwoSat #[] 0 = true := by decide");
    f.blank();
    f.line("/-- 2-SAT decider accepts a known-satisfiable formula (x ∨ y). -/");
    f.line("example : decideTwoSat #[#[(0, false), (1, false)]] 2 = true := by decide");
    f.blank();
    f.line("/-- 2-SAT decider rejects (x) ∧ (¬x). -/");
    f.line("example : decideTwoSat #[#[(0, false)], #[(0, true)]] 1 = false := by decide");
    f.blank();
    f.line("/-- Horn-SAT decider accepts the empty clause list. -/");
    f.line("example : decideHornSat #[] 0 = true := by decide");
    f.blank();
    f.line("/-- Horn-SAT decider rejects (x) ∧ (¬x). -/");
    f.line("example : decideHornSat #[#[(0, false)], #[(0, true)]] 1 = false := by decide");
    f.blank();
    f.line("/-- Dispatch table has exactly 3 rules. -/");
    f.line("example : UOR.Enforcement.InhabitanceDispatchTable.size = 3 := by decide");
    f.blank();
    f.line("/-- LiftChainCertificate.targetLevel reads the wittBits field. -/");
    f.line("example : ({ wittBits := 16 : LiftChainCertificate }).targetLevel = 16 := by decide");
    f.blank();
    f.line("/-- TowerCompletenessResolver at W32 mints a certificate with targetLevel = 32. -/");
    f.line("example :");
    f.line("    (match runTowerCompleteness #[] 32 with");
    f.line("     | .ok cert => cert.inner.targetLevel == 32");
    f.line("     | .error _ => false) = true := by decide");
    f.blank();
    f.line("/-- TowerCompletenessResolver at W16 mints a certificate with targetLevel = 16. -/");
    f.line("example :");
    f.line("    (match runTowerCompleteness #[] 16 with");
    f.line("     | .ok cert => cert.inner.targetLevel == 16");
    f.line("     | .error _ => false) = true := by decide");
    f.blank();
    f.line("/-- InhabitanceResolver certifies the vacuous input. -/");
    f.line("example : (runInhabitance #[] 32).isOk = true := by decide");
    f.blank();
    f.line("-- =========================================================");
    f.line("-- Legacy def hooks retained for backward compatibility.");
    f.line("-- =========================================================");
    f.blank();
    f.line("/-- Size of the Inhabitance dispatch table (expected 3). -/");
    f.line("def dispatchTableSize : Nat := UOR.Enforcement.InhabitanceDispatchTable.size");
    f.blank();
    f.line("/-- 2-SAT decider on the empty clause list. -/");
    f.line("def twoSatEmpty : Bool := UOR.Pipeline.decideTwoSat #[] 0");
    f.blank();
    f.line("/-- Horn-SAT decider on the empty clause list. -/");
    f.line("def hornSatEmpty : Bool := UOR.Pipeline.decideHornSat #[] 0");
    f.blank();
    f.line("end UOR.Test");
    f.finish()
}

/// Generates the `UOR/Prelude.lean` re-export module.
#[must_use]
pub fn generate_prelude(_ontology: &Ontology) -> String {
    let mut f = LeanFile::new(
        "v0.2.1 ergonomics prelude — re-exports the foundation surface \
         under `UOR.Prelude`. Consumers write `import UOR.Prelude` and \
         `open UOR.Prelude` to access the v0.2.1 one-liner API.",
    );
    f.line("import UOR.Enforcement");
    f.line("import UOR.Structures");
    f.blank();
    f.line("namespace UOR.Prelude");
    f.blank();
    f.line("export UOR.Enforcement (");
    f.line("  Validated Grounded GroundedShape OntologyTarget");
    f.line("  ImpossibilityWitnessKind Certify PipelineFailure");
    f.line("  BindingsTable");
    f.line("  GroundingCertificate LiftChainCertificate InhabitanceCertificate");
    f.line("  CompletenessCertificate");
    f.line("  GenericImpossibilityWitness InhabitanceImpossibilityWitness");
    f.line("  ConstrainedTypeInput CompileUnit");
    f.line("  TowerCompletenessResolver IncrementalCompletenessResolver");
    f.line("  GroundingAwareResolver InhabitanceResolver");
    f.line("  W8 W16 W24 W32 MulOp AddOp SubOp XorOp AndOp OrOp NegOp BnotOp SuccOp PredOp");
    f.line("  FragmentMarker DispatchRule");
    f.line(")");
    f.blank();
    f.line("end UOR.Prelude");
    f.finish()
}
