//! Enum detection and generation.
//!
//! Identifies closed sets of named individuals that should be represented
//! as Rust enums, and generates the enum definitions with derives.

use std::fmt::Write as FmtWrite;

use uor_ontology::Ontology;

use crate::emit::{normalize_comment, RustFile};
use crate::mapping::local_name;

/// Detected enum type to generate.
pub struct DetectedEnum {
    /// Rust enum name.
    pub name: &'static str,
    /// Doc comment.
    pub comment: &'static str,
    /// Variants: (variant_name, doc_comment).
    pub variants: Vec<(String, String)>,
    /// Whether to emit `#[non_exhaustive]` on the generated enum.
    pub non_exhaustive: bool,
}

/// Detects all enums from the ontology.
pub fn detect_enums(ontology: &Ontology) -> Vec<DetectedEnum> {
    let mut enums = Vec::new();

    // 1. Space enum (already exists in the ontology model)
    enums.push(DetectedEnum {
        name: "Space",
        comment: "Kernel/user/bridge classification for each namespace module.",
        variants: vec![
            (
                "Kernel".to_string(),
                "Immutable kernel-space: compiled into ROM.".to_string(),
            ),
            (
                "User".to_string(),
                "Parameterizable user-space: runtime declarations.".to_string(),
            ),
            (
                "Bridge".to_string(),
                "Bridge: kernel-computed, user-consumed.".to_string(),
            ),
        ],
        non_exhaustive: false,
    });

    // 2. PrimitiveOp enum from the 10 operation individuals
    let op_ns = ontology.find_namespace("op");
    if let Some(op_module) = op_ns {
        let op_variants: Vec<(String, String)> = op_module
            .individuals
            .iter()
            .filter(|ind| {
                let t = local_name(ind.type_);
                t == "UnaryOp" || t == "BinaryOp" || t == "Involution"
            })
            .map(|ind| {
                let name = capitalize(local_name(ind.id));
                let comment = normalize_comment(ind.comment);
                (name, comment)
            })
            .collect();

        if !op_variants.is_empty() {
            enums.push(DetectedEnum {
                name: "PrimitiveOp",
                comment: "The 18 primitive operations defined in the UOR Foundation. \
                    10 original (Neg/Bnot/Succ/Pred/Add/Sub/Mul/Xor/And/Or), \
                    5 ADR-013/TR-08 substrate amendments (Le/Lt/Ge/Gt/Concat), \
                    3 ADR-053 ring-axis completion (Div/Mod/Pow).",
                variants: op_variants,
                non_exhaustive: false,
            });
        }
    }

    // 3. MetricAxis enum from the 3 metric axis individuals
    let type_ns = ontology.find_namespace("type");
    if let Some(type_module) = type_ns {
        let axis_variants: Vec<(String, String)> = type_module
            .individuals
            .iter()
            .filter(|ind| local_name(ind.type_) == "MetricAxis")
            .map(|ind| {
                let mut name = capitalize(local_name(ind.id));
                // Strip "Axis" suffix to avoid clippy::enum_variant_names
                // (enum is already called MetricAxis)
                if name.ends_with("Axis") {
                    name.truncate(name.len() - 4);
                }
                let comment = normalize_comment(ind.comment);
                (name, comment)
            })
            .collect();

        if !axis_variants.is_empty() {
            enums.push(DetectedEnum {
                name: "MetricAxis",
                comment: "The three metric axes in the UOR tri-metric classification.",
                variants: axis_variants,
                non_exhaustive: false,
            });
        }
    }

    // 4. SiteState enum (from ontology comments)
    enums.push(DetectedEnum {
        name: "SiteState",
        comment: "The state of a site: pinned or free.",
        variants: vec![
            (
                "Pinned".to_string(),
                "Site is determined by a constraint.".to_string(),
            ),
            (
                "Free".to_string(),
                "Site is still available for refinement.".to_string(),
            ),
        ],
        non_exhaustive: false,
    });

    // 5–20. Vocabulary enums — doc comments sourced from the ontology's own
    // class definitions via `Ontology::enum_class_comment`, so Rust and Lean
    // generators cannot drift.
    detect_vocabulary_enum(ontology, "op", "GeometricCharacter", &mut enums);
    detect_vocabulary_enum(ontology, "op", "VerificationDomain", &mut enums);
    detect_vocabulary_enum(ontology, "resolver", "ComplexityClass", &mut enums);
    detect_vocabulary_enum(ontology, "derivation", "RewriteRule", &mut enums);
    detect_vocabulary_enum(ontology, "observable", "MeasurementUnit", &mut enums);
    detect_vocabulary_enum(ontology, "query", "TriadProjection", &mut enums);
    detect_vocabulary_enum(ontology, "state", "SessionBoundaryType", &mut enums);
    detect_vocabulary_enum(ontology, "observable", "PhaseBoundaryType", &mut enums);
    detect_vocabulary_enum(ontology, "state", "GroundingPhase", &mut enums);
    detect_vocabulary_enum(ontology, "observable", "AchievabilityStatus", &mut enums);
    detect_vocabulary_enum(ontology, "op", "ValidityScopeKind", &mut enums);
    detect_vocabulary_enum(ontology, "resolver", "ExecutionPolicyKind", &mut enums);
    detect_vocabulary_enum(ontology, "type", "VarianceAnnotation", &mut enums);
    detect_vocabulary_enum(ontology, "schema", "QuantifierKind", &mut enums);
    detect_vocabulary_enum(ontology, "proof", "ProofStrategy", &mut enums);
    detect_vocabulary_enum(ontology, "conformance", "ViolationKind", &mut enums);
    // v0.2.2 Phase E — PartitionComponent enum class.
    detect_vocabulary_enum(ontology, "partition", "PartitionComponent", &mut enums);

    // 21. ProofModality enum (hardcoded — codegen enum, not an OWL class)
    enums.push(DetectedEnum {
        name: "ProofModality",
        comment: "The modality of a proof: computation (exhaustive verification at a \
                  specific quantum level) or axiomatic (derivation from ring axioms).",
        variants: vec![
            (
                "Computation".to_string(),
                "A proof confirmed by exhaustive execution over R_n at a specific \
                 quantum level."
                    .to_string(),
            ),
            (
                "Axiomatic".to_string(),
                "A proof derived from ring axioms that holds at all quantum levels.".to_string(),
            ),
            // Amendment 86: Empirical variant removed (EmpiricalVerification eliminated)
            (
                "Inductive".to_string(),
                "A proof by structural induction on the quantum level parameter k.".to_string(),
            ),
        ],
        non_exhaustive: false,
    });

    enums
}

/// Detects a vocabulary enum from named individuals of a given class type.
///
/// Scans the specified namespace for individuals whose `type_` matches the
/// class IRI `https://uor.foundation/{ns_prefix}/{class_name}`. Each individual
/// becomes a variant, with the variant name taken from the IRI local name.
/// The doc comment is looked up from the ontology's own class definition
/// via [`Ontology::enum_class_comment`], so Rust and Lean generators cannot
/// drift.
fn detect_vocabulary_enum(
    ontology: &Ontology,
    ns_prefix: &str,
    class_name: &'static str,
    enums: &mut Vec<DetectedEnum>,
) {
    let comment = ontology.enum_class_comment(class_name).unwrap_or("");
    if let Some(module) = ontology.find_namespace(ns_prefix) {
        let class_iri_suffix = format!("/{class_name}");
        let mut variants: Vec<(String, String)> = module
            .individuals
            .iter()
            .filter(|ind| ind.type_.ends_with(&class_iri_suffix))
            .map(|ind| {
                let name = local_name(ind.id).to_string();
                let doc = crate::emit::normalize_comment(ind.comment);
                (name, doc)
            })
            .collect();

        if !variants.is_empty() {
            // Strip common suffix to avoid clippy::enum_variant_names
            if let Some(suffix) = common_variant_suffix(&variants) {
                for (name, _) in &mut variants {
                    name.truncate(name.len() - suffix.len());
                }
            }
            enums.push(DetectedEnum {
                name: class_name,
                comment,
                variants,
                non_exhaustive: false,
            });
        }
    }
}

/// Returns the longest common PascalCase-word suffix shared by all variant names,
/// if all variants share it and removing it leaves a non-empty name.
/// E.g., ["ConstantTime", "LinearTime", "ExponentialTime"] → Some("Time").
fn common_variant_suffix(variants: &[(String, String)]) -> Option<String> {
    if variants.len() < 2 {
        return None;
    }
    let first = &variants[0].0;
    // Find the last uppercase boundary in the first name
    let mut boundary = first.len();
    for (i, ch) in first.char_indices().rev() {
        if ch.is_uppercase() && i > 0 {
            boundary = i;
            break;
        }
    }
    if boundary == 0 || boundary >= first.len() {
        return None;
    }
    let suffix = &first[boundary..];
    // Check if all variants share this suffix and stripping it leaves a non-empty name
    let all_share = variants
        .iter()
        .all(|(name, _)| name.ends_with(suffix) && name.len() > suffix.len());
    if all_share {
        Some(suffix.to_string())
    } else {
        None
    }
}

/// Generates the `enums.rs` file content.
pub fn generate_enums_file(ontology: &Ontology) -> String {
    let enums = detect_enums(ontology);
    let mut f = RustFile::new("Shared enumerations derived from the UOR Foundation ontology.");

    f.line("use core::fmt;");
    f.blank();

    for e in &enums {
        f.doc_comment(e.comment);
        if e.non_exhaustive {
            f.line("#[non_exhaustive]");
        }
        f.line("#[repr(u8)]");
        f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]");
        let _ = writeln!(f.buf, "pub enum {} {{", e.name);
        for (i, (variant, comment)) in e.variants.iter().enumerate() {
            f.indented_doc_comment(comment);
            if i == 0 {
                f.line("    #[default]");
            }
            let _ = writeln!(f.buf, "    {variant},");
        }
        f.line("}");
        f.blank();

        // Display impl
        let _ = writeln!(f.buf, "impl fmt::Display for {} {{", e.name);
        f.line("    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {");
        f.line("        match self {");
        for (variant, _) in &e.variants {
            let display = to_display_str(variant);
            let _ = writeln!(
                f.buf,
                "            Self::{variant} => f.write_str(\"{display}\"),"
            );
        }
        f.line("        }");
        f.line("    }");
        f.line("}");
        f.blank();
    }

    // WittLevel newtype struct — open-world representation of schema:WittLevel.
    // Not an enum: any positive multiple of 8 identifies a valid Witt length.
    f.doc_comment("A Witt level W_n at which the UOR ring R_n = Z/2^n Z operates.");
    f.doc_comment("");
    f.doc_comment("Corresponds to `schema:WittLevel` in the uor.foundation ontology.");
    f.doc_comment("The class is open: any positive multiple of 8 identifies a valid level.");
    f.doc_comment("Named levels W8 through W32 are provided as associated constants.");
    f.doc_comment("Arbitrary levels can be constructed with `WittLevel::new(n)`.");
    f.doc_example(
        "use uor_foundation::WittLevel;\n\
         \n\
         // Named reference levels (W8-W32 are spec-defined):\n\
         let w8 = WittLevel::W8;\n\
         assert_eq!(w8.witt_length(), 8);\n\
         assert_eq!(w8.bits_width(), 8);    // Witt length IS bit width\n\
         assert_eq!(w8.cycle_size(), Some(256)); // 2^8 = 256 ring elements\n\
         \n\
         let w32 = WittLevel::W32;\n\
         assert_eq!(w32.bits_width(), 32);  // 32 bits\n\
         \n\
         // Arbitrary levels beyond W32 (Prism-declared):\n\
         let w64 = WittLevel::new(64);\n\
         assert_eq!(w64.bits_width(), 64);  // 64 bits — native u64\n\
         \n\
         // The chain is unbounded:\n\
         let w88 = WittLevel::new(88);\n\
         assert_eq!(w88.bits_width(), 88);\n\
         assert_eq!(w88.next_witt_level().witt_length(), 96);\n\
         \n\
         // Ordering follows the Witt length:\n\
         assert!(WittLevel::W8 < WittLevel::W32);",
        "rust",
    );
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]");
    f.line("pub struct WittLevel {");
    f.indented_doc_comment("The Witt length n in W_n. Maps to `schema:wittLength`.");
    f.line("    witt_length: u32,");
    f.line("}");
    f.blank();
    f.line("impl WittLevel {");
    f.indented_doc_comment(
        "Witt level 8: 8-bit ring Z/256Z, 256 states. \
         The reference level for all ComputationCertificate proofs in the spec.",
    );
    f.line("    pub const W8: Self = Self { witt_length: 8 };");
    f.indented_doc_comment("Witt level 16: 16-bit ring Z/65536Z, 65,536 states.");
    f.line("    pub const W16: Self = Self { witt_length: 16 };");
    f.indented_doc_comment("Witt level 24: 24-bit ring Z/16777216Z, 16,777,216 states.");
    f.line("    pub const W24: Self = Self { witt_length: 24 };");
    f.indented_doc_comment(
        "Witt level 32: 32-bit ring Z/4294967296Z, 4,294,967,296 states. \
         The highest named level in the spec.",
    );
    f.line("    pub const W32: Self = Self { witt_length: 32 };");
    f.blank();
    f.indented_doc_comment(
        "Construct an arbitrary Witt level W_n. \
         `n` need not be one of the spec-named individuals; \
         Prism implementations may use any level.",
    );
    f.line("    #[inline]");
    f.line("    pub const fn new(witt_length: u32) -> Self {");
    f.line("        Self { witt_length }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("The Witt length n. Maps to `schema:wittLength`.");
    f.line("    #[inline]");
    f.line("    pub const fn witt_length(self) -> u32 {");
    f.line("        self.witt_length");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "Bit width of the ring at this level: equal to the Witt length. \
         Maps to `schema:bitsWidth`. This is an identity — \
         the Witt length IS the bit width.",
    );
    f.line("    #[inline]");
    f.line("    pub const fn bits_width(self) -> u32 {");
    f.line("        self.witt_length");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "Number of distinct ring states at this level: 2^n. \
         Maps to `schema:cycleSize`. Returns `None` if the result \
         exceeds `u128` (i.e. for n > 128).",
    );
    f.line("    #[inline]");
    f.line("    pub const fn cycle_size(self) -> Option<u128> {");
    f.line("        1u128.checked_shl(self.bits_width())");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "The next Witt level in the chain: W_n -> W_{n+8}. \
         Maps to `schema:nextWittLevel`. Always well-defined; the chain is unbounded.",
    );
    f.line("    #[inline]");
    f.line("    pub const fn next_witt_level(self) -> Self {");
    f.line("        Self {");
    f.line("            witt_length: self.witt_length + 8,");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();
    f.line("impl Default for WittLevel {");
    f.indented_doc_comment(
        "`W8` is the spec-defined minimum Witt level and the canonical base \
         referenced by `schema:WittLevel` individuals.",
    );
    f.line("    #[inline]");
    f.line("    fn default() -> Self {");
    f.line("        Self::W8");
    f.line("    }");
    f.line("}");
    f.blank();
    f.line("impl fmt::Display for WittLevel {");
    f.line("    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {");
    f.line("        write!(f, \"w{}\", self.witt_length)");
    f.line("    }");
    f.line("}");
    f.blank();

    f.finish()
}

/// Capitalizes the first character of a string.
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => {
            let mut result = c.to_uppercase().to_string();
            result.push_str(chars.as_str());
            result
        }
    }
}

/// Converts a PascalCase variant to a display string (e.g., "VerticalAxis" → "vertical_axis").
fn to_display_str(s: &str) -> String {
    crate::mapping::to_snake_case(s)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn enum_detection_finds_all() {
        let ontology = Ontology::full();
        let enums = detect_enums(ontology);
        assert!(
            enums.len() >= 12,
            "Expected at least 12 enums, got {}",
            enums.len()
        );

        let names: Vec<&str> = enums.iter().map(|e| e.name).collect();
        assert!(names.contains(&"Space"));
        assert!(names.contains(&"PrimitiveOp"));
        assert!(names.contains(&"MetricAxis"));
        assert!(names.contains(&"SiteState"));
        assert!(names.contains(&"GeometricCharacter"));
        assert!(names.contains(&"VerificationDomain"));
        assert!(names.contains(&"ComplexityClass"));
        assert!(names.contains(&"RewriteRule"));
        assert!(names.contains(&"MeasurementUnit"));
        assert!(names.contains(&"TriadProjection"));
        assert!(names.contains(&"SessionBoundaryType"));
        assert!(names.contains(&"ProofModality"));
    }

    #[test]
    fn primitive_op_has_18_variants() {
        let ontology = Ontology::full();
        let enums = detect_enums(ontology);
        let prim_op = enums.iter().find(|e| e.name == "PrimitiveOp").unwrap();
        // 10 original (Neg/Bnot/Succ/Pred/Add/Sub/Mul/Xor/And/Or)
        // + 4 comparison (Le/Lt/Ge/Gt) per ADR-013/TR-08 substrate amendment
        // + 1 byte-packing (Concat) per ADR-013/TR-08 substrate amendment
        // + 3 ring-axis completion (Div/Mod/Pow) per ADR-053
        assert_eq!(prim_op.variants.len(), 18);
    }
}
