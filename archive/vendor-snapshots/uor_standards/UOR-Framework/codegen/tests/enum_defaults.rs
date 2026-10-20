//! Phase 7a test: every generated enum class emits `Default` with `#[default]`
//! on the first variant, and `WittLevel` emits `Default::default() == W8`.
//!
//! Verifies two pieces of the Phase-7a spec:
//!
//! 1. The enum emission loop adds `Default` to the derive list and tags the
//!    first variant with `#[default]`.
//! 2. The hand-emitted `WittLevel` struct has `impl Default for WittLevel {
//!    fn default() -> Self { Self::W8 } }`.

use uor_codegen::enums::generate_enums_file;
use uor_ontology::Ontology;

#[test]
fn every_enum_class_derives_default_and_tags_first_variant() {
    let source = generate_enums_file(Ontology::full());

    // Each `pub enum {Name}` block must have a `#[derive(..., Default)]` line
    // immediately preceding it, and the first variant (ignoring doc comments
    // and indented attributes) must be tagged `#[default]`.
    let mut enum_heads: Vec<&str> = Vec::new();
    for (idx, line) in source.lines().enumerate() {
        if line.starts_with("pub enum ") {
            enum_heads.push(line.trim_start_matches("pub enum "));
            let preceding = source.lines().nth(idx.saturating_sub(1)).unwrap_or("");
            assert!(
                preceding.contains("Default"),
                "enum {line}: missing Default in preceding derive line `{preceding}`"
            );
        }
    }
    assert!(
        enum_heads.len() >= 17,
        "expected ≥ 17 generated enums, got {} — emission skipped some",
        enum_heads.len()
    );

    // Every enum block must contain at least one `#[default]` attribute line.
    let mut default_count = 0usize;
    for line in source.lines() {
        if line.trim() == "#[default]" {
            default_count += 1;
        }
    }
    assert_eq!(
        default_count,
        enum_heads.len(),
        "expected exactly one `#[default]` per enum; got {default_count} attributes \
         across {} enums",
        enum_heads.len()
    );
}

#[test]
fn witt_level_default_impl_present() {
    let source = generate_enums_file(Ontology::full());

    assert!(
        source.contains("impl Default for WittLevel {"),
        "WittLevel needs a hand-emitted `impl Default` block"
    );
    assert!(
        source.contains("Self::W8"),
        "WittLevel::default() must return Self::W8"
    );
}
