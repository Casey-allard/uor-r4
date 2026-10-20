//! Lean 4 `Primitives` class generator.
//!
//! Generates `UOR/Primitives.lean` containing the `Primitives` typeclass.

use crate::emit::LeanFile;

/// Generates the content of `UOR/Primitives.lean`.
pub fn generate_primitives() -> String {
    let mut f = LeanFile::new("Primitives typeclass \u{2014} XSD primitive type family.");
    f.line("namespace UOR.Primitives");
    f.blank();
    f.doc_comment(
        "XSD primitive type family. Implementations choose concrete representations \
         for each XSD type. All generated structures are parametric over this class.",
    );
    f.line("class Primitives where");
    f.indented_doc_comment("String type (xsd:string).");
    f.line("  String : Type");
    f.indented_doc_comment("Integer type (xsd:integer).");
    f.line("  Integer : Type");
    f.indented_doc_comment("Non-negative integer type (xsd:nonNegativeInteger).");
    f.line("  NonNegativeInteger : Type");
    f.indented_doc_comment("Positive integer type (xsd:positiveInteger).");
    f.line("  PositiveInteger : Type");
    f.indented_doc_comment("Decimal type (xsd:decimal).");
    f.line("  Decimal : Type");
    f.indented_doc_comment("Boolean type (xsd:boolean).");
    f.line("  Boolean : Type");
    f.blank();
    f.line("end UOR.Primitives");
    f.blank();
    // Canonical `Primitives` instance using Lean core types. Every
    // generated named-individual `def` is parameterized over this
    // instance, making the package immediately usable after `import UOR`.
    f.line("namespace UOR.Prims");
    f.blank();
    f.doc_comment(
        "Canonical `Primitives` instance using Lean core types. \
         Used by all generated named-individual constants so that \
         `import UOR` yields directly constructible values.",
    );
    f.line("def Standard : UOR.Primitives.Primitives where");
    f.line("  String := String");
    f.line("  Integer := Int");
    f.line("  NonNegativeInteger := Nat");
    f.line("  PositiveInteger := Nat");
    f.line("  Decimal := Float");
    f.line("  Boolean := Bool");
    f.blank();
    f.line("end UOR.Prims");
    f.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn namespace_wrapper() {
        let s = generate_primitives();
        assert!(s.contains("namespace UOR.Primitives"));
        assert!(s.contains("class Primitives where"));
        assert!(s.contains("end UOR.Primitives"));
    }

    #[test]
    fn standard_instance_present() {
        let s = generate_primitives();
        assert!(s.contains("namespace UOR.Prims"));
        assert!(s.contains("def Standard : UOR.Primitives.Primitives"));
        assert!(s.contains("end UOR.Prims"));
        // All six primitive fields must be present.
        assert!(s.contains("String := String"));
        assert!(s.contains("Integer := Int"));
        assert!(s.contains("NonNegativeInteger := Nat"));
        assert!(s.contains("PositiveInteger := Nat"));
        assert!(s.contains("Decimal := Float"));
        assert!(s.contains("Boolean := Bool"));
    }
}
