//! Named individual → constant generation.
//!
//! Generates `PrimitiveOp` trait implementations from the operation
//! individual property assertions.

use std::fmt::Write as FmtWrite;

use uor_ontology::{IndividualValue, Ontology};

use crate::emit::RustFile;
use crate::mapping::local_name;

/// Generates `PrimitiveOp` impl block in the `op` module.
///
/// Returns the Rust source code for the impl block to be appended to `op.rs`.
pub fn generate_primitive_op_impls(ontology: &Ontology) -> String {
    // Use a plain string — not RustFile::new() — since this is appended mid-file
    let mut f = RustFile {
        buf: String::with_capacity(2048),
    };

    let op_module = match ontology.find_namespace("op") {
        Some(m) => m,
        None => return String::new(),
    };

    // Import the PrimitiveOp enum (it lives in crate::enums, re-exported at crate root)
    f.line("use crate::enums::PrimitiveOp;");
    f.blank();

    // Collect operation data
    let mut ops: Vec<OpData> = Vec::new();
    for ind in &op_module.individuals {
        let type_local = local_name(ind.type_);
        if type_local != "UnaryOp" && type_local != "BinaryOp" && type_local != "Involution" {
            continue;
        }
        let variant = capitalize(local_name(ind.id));
        let mut data = OpData {
            variant,
            type_local: type_local.to_string(),
            arity: None,
            is_commutative: None,
            is_involution: None,
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
                "isInvolution" => {
                    if let IndividualValue::Bool(b) = value {
                        data.is_involution = Some(*b);
                    }
                }
                "hasGeometricCharacter" => {
                    if let IndividualValue::IriRef(iri) = value {
                        data.geometric_character = Some(local_name(iri).to_string());
                    }
                }
                _ => {}
            }
        }
        ops.push(data);
    }

    // Generate arity method
    f.line("impl PrimitiveOp {");
    f.indented_doc_comment("Returns the arity of this operation (1 for unary, 2 for binary).");
    f.line("    #[must_use]");
    f.line("    pub const fn arity(self) -> i64 {");
    f.line("        match self {");
    for op in &ops {
        if let Some(a) = op.arity {
            let _ = writeln!(f.buf, "            Self::{} => {a},", op.variant);
        }
    }
    f.line("        }");
    f.line("    }");
    f.blank();

    // Generate is_commutative method
    f.indented_doc_comment("Returns whether this operation is commutative.");
    f.line("    #[must_use]");
    f.line("    pub const fn is_commutative(self) -> bool {");
    let has_any_commutative = ops.iter().any(|op| op.is_commutative.is_some());
    if has_any_commutative {
        f.line("        match self {");
        for op in &ops {
            if let Some(b) = op.is_commutative {
                let _ = writeln!(f.buf, "            Self::{} => {b},", op.variant);
            }
        }
        f.line("            _ => false,");
        f.line("        }");
    } else {
        f.line("        false");
    }
    f.line("    }");
    f.blank();

    // Generate is_involution method
    f.indented_doc_comment("Returns whether this operation is an involution (self-inverse).");
    f.line("    #[must_use]");
    f.line("    pub const fn is_involution(self) -> bool {");
    let has_any_involution = ops.iter().any(|op| op.is_involution.is_some());
    if has_any_involution {
        f.line("        match self {");
        for op in &ops {
            if let Some(b) = op.is_involution {
                let _ = writeln!(f.buf, "            Self::{} => {b},", op.variant);
            }
        }
        f.line("            _ => false,");
        f.line("        }");
    } else {
        f.line("        false");
    }
    f.line("    }");
    f.blank();

    // Generate has_geometric_character method
    f.indented_doc_comment("Returns the geometric character of this operation.");
    f.line("    #[must_use]");
    f.line("    pub const fn has_geometric_character(self) -> crate::enums::GeometricCharacter {");
    f.line("        match self {");
    let all_have_gc = ops.iter().all(|op| op.geometric_character.is_some());
    for op in &ops {
        if let Some(ref gc) = op.geometric_character {
            // gc is already PascalCase (IRI local name), no conversion needed
            let _ = writeln!(
                f.buf,
                "            Self::{} => crate::enums::GeometricCharacter::{gc},",
                op.variant
            );
        }
    }
    // Only add a default arm if not all variants have a geometric character
    if !all_have_gc {
        f.line("            _ => crate::enums::GeometricCharacter::RingReflection,");
    }
    f.line("        }");
    f.line("    }");

    // Generate is_unary/is_binary convenience methods
    f.blank();
    f.indented_doc_comment("Returns true if this is a unary operation.");
    f.line("    #[must_use]");
    f.line("    pub const fn is_unary(self) -> bool {");
    f.line("        self.arity() == 1");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Returns true if this is a binary operation.");
    f.line("    #[must_use]");
    f.line("    pub const fn is_binary(self) -> bool {");
    f.line("        self.arity() == 2");
    f.line("    }");

    f.line("}");
    f.blank();

    f.finish()
}

struct OpData {
    variant: String,
    #[allow(dead_code)]
    type_local: String,
    arity: Option<i64>,
    is_commutative: Option<bool>,
    is_involution: Option<bool>,
    geometric_character: Option<String>,
}

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
