//! Lean 4 inductive type generator for enum classes.
//!
//! Generates `UOR/Enums.lean` containing all 18 enum classes as inductives
//! (or structures for open-world types like WittLevel), plus hardcoded
//! codegen enums (Space, PrimitiveOp, SiteState, ProofModality).

use std::fmt::Write as FmtWrite;

use uor_ontology::model::{IndividualValue, Ontology};

use crate::emit::{normalize_lean_comment, LeanFile};
use crate::mapping::local_name;

/// A detected enum with its variants.
struct DetectedEnum {
    /// Lean inductive name.
    name: String,
    /// Doc comment.
    comment: String,
    /// (variant_name, variant_comment) pairs.
    variants: Vec<(String, String)>,
}

/// Generates the content of `UOR/Enums.lean`.
pub fn generate_enums(ontology: &Ontology) -> String {
    let mut f = LeanFile::new("Controlled vocabulary types (enum classes).");
    f.line("import UOR.Primitives");
    f.blank();
    f.line("open UOR.Primitives");
    f.blank();

    let enums = detect_enums(ontology);

    for e in &enums {
        f.doc_comment(&e.comment);
        let _ = writeln!(f.buf, "inductive {} where", e.name);
        for (variant, comment) in &e.variants {
            f.indented_doc_comment(comment);
            let _ = writeln!(f.buf, "  | {variant} : {}", e.name);
        }
        f.line("  deriving DecidableEq, Repr, BEq, Hashable, Inhabited");
        f.blank();
    }

    // WittLevel — open-world structure (not inductive)
    generate_witt_level(&mut f, ontology);

    f.finish()
}

/// Detects all enum classes from the ontology and hardcoded codegen enums.
fn detect_enums(ontology: &Ontology) -> Vec<DetectedEnum> {
    let mut enums = Vec::new();

    // Hardcoded: Space
    enums.push(DetectedEnum {
        name: "Space".into(),
        comment: "Ontology space classification.".into(),
        variants: vec![
            ("kernel".into(), "Immutable foundation layer.".into()),
            ("user".into(), "Runtime-parameterizable layer.".into()),
            (
                "bridge".into(),
                "Kernel-computed, user-consumed layer.".into(),
            ),
        ],
    });

    // Hardcoded: SiteState
    enums.push(DetectedEnum {
        name: "SiteState".into(),
        comment: "Site occupancy state within a partition.".into(),
        variants: vec![
            ("pinned".into(), "Site is occupied and immutable.".into()),
            ("free".into(), "Site is available for allocation.".into()),
        ],
    });

    // PrimitiveOp — from individuals
    detect_primitive_op(ontology, &mut enums);

    // Vocabulary enums — doc comments sourced from the ontology's own class
    // definitions via `Ontology::enum_class_comment`, mirroring the Rust
    // codegen so the two generators cannot drift.
    detect_vocabulary_enum(ontology, "type", "MetricAxis", &mut enums);
    detect_vocabulary_enum(ontology, "op", "GeometricCharacter", &mut enums);
    detect_vocabulary_enum(ontology, "op", "VerificationDomain", &mut enums);
    detect_vocabulary_enum(ontology, "op", "ValidityScopeKind", &mut enums);
    detect_vocabulary_enum(ontology, "resolver", "ExecutionPolicyKind", &mut enums);
    detect_vocabulary_enum(ontology, "resolver", "ComplexityClass", &mut enums);
    detect_vocabulary_enum(ontology, "derivation", "RewriteRule", &mut enums);
    detect_vocabulary_enum(ontology, "type", "VarianceAnnotation", &mut enums);
    detect_vocabulary_enum(ontology, "observable", "MeasurementUnit", &mut enums);
    detect_vocabulary_enum(ontology, "query", "TriadProjection", &mut enums);
    detect_vocabulary_enum(ontology, "observable", "PhaseBoundaryType", &mut enums);
    detect_vocabulary_enum(ontology, "state", "GroundingPhase", &mut enums);
    detect_vocabulary_enum(ontology, "observable", "AchievabilityStatus", &mut enums);
    detect_vocabulary_enum(ontology, "state", "SessionBoundaryType", &mut enums);
    detect_vocabulary_enum(ontology, "proof", "ProofStrategy", &mut enums);
    detect_vocabulary_enum(ontology, "schema", "QuantifierKind", &mut enums);
    detect_vocabulary_enum(ontology, "conformance", "ViolationKind", &mut enums);
    // v0.2.2 Phase E — PartitionComponent enum class (Irreducible /
    // Reducible / Units / Exterior).
    detect_vocabulary_enum(ontology, "partition", "PartitionComponent", &mut enums);

    // Hardcoded: ProofModality (Amendment 86: Empirical variant removed — EmpiricalVerification eliminated)
    enums.push(DetectedEnum {
        name: "ProofModality".into(),
        comment: "Proof modality classification.".into(),
        variants: vec![
            (
                "computation".into(),
                "Exhaustive computation at a quantum level.".into(),
            ),
            (
                "axiomatic".into(),
                "Derivation from axioms and definitions.".into(),
            ),
            (
                "inductive".into(),
                "Structural induction on quantum level.".into(),
            ),
        ],
    });

    enums
}

/// Detects PrimitiveOp variants from op namespace individuals.
fn detect_primitive_op(ontology: &Ontology, enums: &mut Vec<DetectedEnum>) {
    let op_module = match ontology.find_namespace("op") {
        Some(m) => m,
        None => return,
    };

    let mut variants = Vec::new();
    for ind in &op_module.individuals {
        let type_local = local_name(ind.type_);
        if type_local != "UnaryOp" && type_local != "BinaryOp" && type_local != "Involution" {
            continue;
        }
        let name = to_camel_case_variant(local_name(ind.id));
        let comment = normalize_lean_comment(ind.comment);
        variants.push((name, comment));
    }

    if !variants.is_empty() {
        enums.push(DetectedEnum {
            name: "PrimitiveOp".into(),
            comment: "Primitive algebraic operations.".into(),
            variants,
        });
    }
}

/// Detects a vocabulary enum from individuals of a specific class in a namespace.
///
/// The doc comment is looked up from the ontology's own class definition via
/// [`Ontology::enum_class_comment`], so the Rust and Lean generators cannot
/// drift on enum-class documentation.
fn detect_vocabulary_enum(
    ontology: &Ontology,
    ns_prefix: &str,
    class_name: &str,
    enums: &mut Vec<DetectedEnum>,
) {
    let comment = ontology.enum_class_comment(class_name).unwrap_or("");
    let module = match ontology.find_namespace(ns_prefix) {
        Some(m) => m,
        None => return,
    };

    let suffix = format!("/{class_name}");
    let mut variants: Vec<(String, String)> = module
        .individuals
        .iter()
        .filter(|ind| ind.type_.ends_with(&suffix))
        .map(|ind| {
            let name = to_camel_case_variant(local_name(ind.id));
            let c = normalize_lean_comment(ind.comment);
            (name, c)
        })
        .collect();

    // Strip common PascalCase suffix to avoid redundancy
    if let Some(sfx) = common_variant_suffix(&variants) {
        for (name, _) in &mut variants {
            if name.len() > sfx.len() && name.ends_with(&sfx) {
                name.truncate(name.len() - sfx.len());
            }
        }
    }

    if !variants.is_empty() {
        enums.push(DetectedEnum {
            name: class_name.to_string(),
            comment: comment.to_string(),
            variants,
        });
    }
}

/// Finds the common PascalCase-word suffix shared by all variant names.
fn common_variant_suffix(variants: &[(String, String)]) -> Option<String> {
    if variants.len() < 2 {
        return None;
    }
    let first = &variants[0].0;
    // Find the last uppercase boundary in the first variant
    let boundary = first
        .char_indices()
        .rev()
        .find(|(i, c)| *i > 0 && c.is_uppercase())
        .map(|(i, _)| i)?;
    let candidate = &first[boundary..];
    // Check all variants share this suffix and stripping leaves non-empty
    for (name, _) in variants {
        if !name.ends_with(candidate) || name.len() <= candidate.len() {
            return None;
        }
    }
    Some(candidate.to_string())
}

/// Converts a local name to a camelCase Lean variant name.
fn to_camel_case_variant(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => {
            let mut result: String = c.to_lowercase().collect();
            result.extend(chars);
            result
        }
    }
}

/// Generates the WittLevel structure (open-world, not inductive) and one
/// `def Wn : WittLevel := ⟨n⟩` per `schema:WittLevel` individual in the
/// ontology. v0.2.2 Phase C lifted the W8/W16/W24/W32 hardcoded set to a
/// parametric walk so that adding a Witt level is a pure ontology edit.
fn generate_witt_level(f: &mut LeanFile, ontology: &Ontology) {
    f.doc_comment("Witt vector length (multiples of 8). Open-world: any W_n is valid.");
    f.line("structure WittLevel where");
    f.indented_doc_comment("The Witt vector length in bits.");
    f.line("  wittLength : Nat");
    f.line("  deriving DecidableEq, Repr, BEq, Hashable, Inhabited");
    f.blank();
    f.line("namespace WittLevel");
    f.blank();

    // Walk schema:WittLevel individuals; emit one `def` per individual,
    // sorted by bit width so the file is deterministic.
    let witt_iri = "https://uor.foundation/schema/WittLevel";
    let mut levels: Vec<(String, u64)> = Vec::new();
    if let Some(schema_module) = ontology.find_namespace("schema") {
        for ind in &schema_module.individuals {
            if ind.type_ != witt_iri {
                continue;
            }
            // Extract bitsWidth via the schema:bitsWidth property.
            let bits = ind
                .properties
                .iter()
                .find_map(|(k, v)| {
                    if *k == "https://uor.foundation/schema/bitsWidth" {
                        if let uor_ontology::model::IndividualValue::Int(n) = v {
                            return Some(*n as u64);
                        }
                    }
                    None
                })
                .unwrap_or(0);
            if bits == 0 {
                continue;
            }
            levels.push((ind.label.to_string(), bits));
        }
    }
    levels.sort_by_key(|(_, b)| *b);
    for (label, bits) in &levels {
        f.doc_comment(&format!("Standard Witt level {label} ({bits}-bit ring)."));
        let _ = writeln!(f.buf, "def {label} : WittLevel := \u{27e8}{bits}\u{27e9}");
    }
    f.blank();
    f.doc_comment("Construct an arbitrary Witt level.");
    f.line("def new (n : Nat) : WittLevel := \u{27e8}n\u{27e9}");
    f.blank();
    f.doc_comment("The bit width (identity with wittLength).");
    f.line("def bitsWidth (w : WittLevel) : Nat := w.wittLength");
    f.blank();
    f.line("end WittLevel");
}

/// Returns the number of enums that will be generated (for reporting).
pub fn count_enums(ontology: &Ontology) -> usize {
    // detect_enums length + 1 for WittLevel
    detect_enums(ontology).len() + 1
}

/// Returns the set of enum class names from the ontology (used for filtering).
///
/// This delegates to `Ontology::enum_class_names()` to maintain single source
/// of truth.
pub fn enum_class_names() -> &'static [&'static str] {
    Ontology::enum_class_names()
}

/// Returns the set of individual types that map ONLY to enum variants
/// (not typed-def individuals). `UnaryOp`/`BinaryOp`/`Involution` are
/// deliberately excluded: those are lifted into the synthetic
/// `PrimitiveOp` enum, but they *also* have their individuals emitted
/// as typed `def`s by `individuals::generate_all_individuals` so that
/// structure-typed references to them (`op:D2n.generatedBy → neg`,
/// `morphism:criticalComposition.lawComponents → neg`, etc.) can
/// resolve through the subclass coercion chain. Only ontology enum
/// classes — whose individuals have no structure declaration — are
/// skipped by the individual emitter.
pub fn enum_individual_types() -> Vec<&'static str> {
    Ontology::enum_class_names().to_vec()
}

/// Generates PrimitiveOp method definitions from individual property data.
///
/// This produces `def arity`, `def isCommutative`, etc. in the PrimitiveOp
/// namespace, with match arms generated from individual properties.
pub fn generate_primitive_op_methods(ontology: &Ontology) -> String {
    let op_module = match ontology.find_namespace("op") {
        Some(m) => m,
        None => return String::new(),
    };

    struct OpData {
        variant: String,
        arity: Option<i64>,
        is_commutative: Option<bool>,
        geometric_character: Option<String>,
    }

    let mut ops: Vec<OpData> = Vec::new();
    for ind in &op_module.individuals {
        let type_local = local_name(ind.type_);
        if type_local != "UnaryOp" && type_local != "BinaryOp" && type_local != "Involution" {
            continue;
        }
        let variant = to_camel_case_variant(local_name(ind.id));
        let mut data = OpData {
            variant,
            arity: None,
            is_commutative: None,
            geometric_character: None,
        };
        for (prop_iri, value) in ind.properties {
            let prop = local_name(prop_iri);
            match prop {
                "arity" => {
                    if let IndividualValue::Int(n) = value {
                        data.arity = Some(*n);
                    }
                }
                "isCommutative" => {
                    if let IndividualValue::Bool(b) = value {
                        data.is_commutative = Some(*b);
                    }
                }
                "hasGeometricCharacter" => {
                    if let IndividualValue::IriRef(iri) = value {
                        data.geometric_character = Some(to_camel_case_variant(local_name(iri)));
                    }
                }
                _ => {}
            }
        }
        ops.push(data);
    }

    if ops.is_empty() {
        return String::new();
    }

    let mut buf = String::new();
    buf.push_str("namespace PrimitiveOp\n\n");

    // arity
    buf.push_str("/-- The arity of this operation. -/\n");
    buf.push_str("def arity : PrimitiveOp \u{2192} Int\n");
    for op in &ops {
        let a = op.arity.unwrap_or(0);
        let _ = writeln!(buf, "  | .{} => {a}", op.variant);
    }
    buf.push('\n');

    // isCommutative
    buf.push_str("/-- Whether this operation is commutative. -/\n");
    buf.push_str("def isCommutative : PrimitiveOp \u{2192} Bool\n");
    for op in &ops {
        let c = op.is_commutative.unwrap_or(false);
        let _ = writeln!(buf, "  | .{} => {c}", op.variant);
    }
    buf.push('\n');

    // hasGeometricCharacter
    buf.push_str("/-- The geometric character of this operation. -/\n");
    buf.push_str("def hasGeometricCharacter : PrimitiveOp \u{2192} GeometricCharacter\n");
    for op in &ops {
        let gc = op
            .geometric_character
            .as_deref()
            .unwrap_or("ringReflection");
        let _ = writeln!(buf, "  | .{} => .{gc}", op.variant);
    }
    buf.push('\n');

    buf.push_str("end PrimitiveOp\n");
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_all_enum_classes() {
        let ontology = uor_ontology::Ontology::full();
        let enums = detect_enums(ontology);
        // 18 vocabulary (17 v0.2.1 + 1 v0.2.2 PartitionComponent) +
        // 4 hardcoded (Space, SiteState, PrimitiveOp, ProofModality)
        assert_eq!(enums.len(), 22);
    }

    #[test]
    fn primitive_op_methods_include_geometric_character() {
        let ontology = uor_ontology::Ontology::full();
        let methods = generate_primitive_op_methods(ontology);
        assert!(methods.contains("def hasGeometricCharacter"));
        assert!(methods.contains("def arity"));
        assert!(methods.contains("def isCommutative"));
    }

    #[test]
    fn witt_level_counted_separately() {
        let ontology = uor_ontology::Ontology::full();
        // count_enums = detect_enums().len() + 1 for WittLevel
        assert_eq!(count_enums(ontology), 23);
    }
}
