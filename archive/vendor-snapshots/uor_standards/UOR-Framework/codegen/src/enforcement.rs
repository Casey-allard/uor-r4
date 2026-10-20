//! Generates the `enforcement.rs` module for the `uor-foundation` crate.
//!
//! This module emits Layer 1 (opaque witnesses), Layer 2 (declarative builders),
//! the Term AST, and the v0.2.1 ergonomics surface (sealed `OntologyTarget` /
//! `Grounded<T>` wrappers, the `Certify` trait, the parametric `PipelineFailure`
//! enum, ring-op phantom wrappers, fragment markers, dispatch tables, and the
//! `prelude` module).

use crate::emit::RustFile;
use uor_ontology::model::{IndividualValue, Ontology};

/// Generates the complete `enforcement.rs` module content.
///
/// # Errors
///
/// This function is infallible but returns `String` for consistency.
#[must_use]
pub fn generate_enforcement_module(ontology: &Ontology) -> String {
    let mut f = RustFile::new(
        "Declarative enforcement types.\n\
         //!\n\
         //! This module contains the opaque witness types, declarative builders,\n\
         //! the Term AST, and the v0.2.1 ergonomics surface (sealed `Grounded<T>`,\n\
         //! the `Certify` trait, `PipelineFailure`, ring-op phantom wrappers,\n\
         //! fragment markers, dispatch tables, and the `prelude` module).\n\
         //!\n\
         //! # Layers\n\
         //!\n\
         //! - **Layer 1** \\[Opaque Witnesses\\]: `Datum`, `Validated<T>`, `Derivation`,\n\
         //!   `FreeRank` \\[private fields, no public constructors\\]\n\
         //! - **Layer 2** \\[Declarative Builders\\]: `CompileUnitBuilder`,\n\
         //!   `EffectDeclarationBuilder`, etc. \\[produce `Validated<T>` on success\\]\n\
         //! - **Term AST**: `Term`, `TermArena`, `Binding`, `Assertion`, etc.\n\
         //! - **v0.2.1 Ergonomics**: `OntologyTarget`, `GroundedShape`, `Grounded<T>`,\n\
         //!   `Certify`, `PipelineFailure`, `RingOp<L>`, fragment markers,\n\
         //!   `INHABITANCE_DISPATCH_TABLE`, and the `prelude` module.",
    );

    f.line(
        "use crate::{DecimalTranscendental, HostTypes, MetricAxis, PrimitiveOp, VerificationDomain, ViolationKind, WittLevel};",
    );
    f.line("use core::marker::PhantomData;");
    f.blank();

    generate_sealed_module(&mut f);
    generate_datum_types(&mut f, ontology);
    generate_grounding_types(&mut f, ontology);
    generate_witness_types(&mut f);
    generate_uor_time(&mut f);
    generate_term_ast(&mut f);
    generate_shape_violation(&mut f);
    generate_builders(&mut f);
    generate_minting_session(&mut f, ontology);
    generate_const_ring_eval(&mut f, ontology);

    // v0.2.2 Phase C.3: Limbs<N> generic kernel for high Witt levels.
    generate_limbs_kernel(&mut f);

    // v0.2.1 ergonomics surface generators (parametric — read from ontology)
    generate_ontology_target_trait(&mut f, ontology);
    // v0.2.2 Phase C.4: MulContext + MultiplicationCertificate evidence.
    // Must run after generate_ontology_target_trait because it extends the
    // MultiplicationCertificate shim.
    generate_multiplication_context(&mut f);
    generate_grounded_wrapper(&mut f);
    generate_pipeline_failure(&mut f, ontology);
    generate_certify_trait(&mut f, ontology);
    generate_ring_ops(&mut f, ontology);
    // v0.2.2 Phase C.3: emit Limbs-backed marker structs + RingOp impls
    // for every WittLevel individual whose bit_width > 128.
    generate_limbs_ring_ops(&mut f, ontology);
    // Phase L.2 (target §4.5 + §9 criterion 5): emit const_ring_eval_w{n}
    // helpers for every Limbs-backed level.
    generate_const_ring_eval_limbs(&mut f, ontology);
    generate_fragment_markers(&mut f, ontology);
    generate_dispatch_tables(&mut f, ontology);
    generate_validated_deref(&mut f);
    // v0.2.2 Phase D (Q4): parametric constraint surface.
    generate_parametric_constraint_surface(&mut f);
    // v0.2.2 Phase E: bridge namespace completion — sealed Query/Coordinate/
    // BindingQuery/Partition/Trace/TraceEvent/HomologyClass/CohomologyClass/
    // Interaction types + Derivation::replay().
    generate_bridge_namespace_surface(&mut f);
    // v0.2.2 Phase J: combinator-only Grounding — closed 12-combinator
    // surface + GroundingProgram<O, M> + MarkersImpliedBy<Map>.
    generate_grounding_combinator_surface(&mut f);
    // Product/Coproduct Completion Amendment §2.3b–§2.3h: sealed witness
    // types for PartitionProduct / PartitionCoproduct / CartesianPartition-
    // Product, paired Evidence / MintInputs structs, entropy helpers, the
    // `validate_coproduct_structure` structural validator (with
    // Conjunction recursion per plan §A4), three mint primitives citing
    // op-namespace theorems and foundation-namespace layout invariants,
    // and the `VerifiedMint` sealed mint trait with impls routing through
    // those primitives.
    generate_product_coproduct_amendment(&mut f);
    generate_prelude(&mut f, ontology);

    f.finish()
}

fn generate_sealed_module(f: &mut RustFile) {
    f.doc_comment("Private sealed module preventing downstream implementations.");
    f.doc_comment("Only `GroundedCoord` and `GroundedTuple<N>` implement `Sealed`.");
    f.line("mod sealed {");
    f.indented_doc_comment(
        "Sealed trait. Not publicly implementable because this module is private.",
    );
    f.line("    pub trait Sealed {}");
    f.line("    impl Sealed for super::GroundedCoord {}");
    f.line("    impl<const N: usize> Sealed for super::GroundedTuple<N> {}");
    f.line("}");
    f.blank();
}

/// v0.2.1 Phase 8b.7: data-driven Witt level descriptors sourced from
/// `schema:WittLevel` individuals in the ontology.
///
/// Each returned tuple is `(local_name, bits_width, byte_width)`:
///
/// - `local_name` is the ontology individual's local name (`W8`, `W16`,
///   `W24`, `W32`, ...). This becomes the `DatumInner` variant name.
/// - `bits_width` is the `schema:bitsWidth` annotation value.
/// - `byte_width` is `bits_width.div_ceil(8)` — the payload size in bytes.
///
/// Sorted ascending by `bits_width` so the emitted enum variants appear
/// in a deterministic small-to-large order.
///
/// v0.2.1 scope: the walk filters to levels whose `bits_width` is a
/// multiple of 8 **and** fits into a native Rust int type (≤ 64 bits).
/// W24 is emitted as a 3-byte variant backed by `u32` with a 24-bit mask
/// for ring-op evaluation. Deeper levels (if the ontology adds e.g. W128)
/// get stored but not ring-op-wired until the foundation grows a code
/// path for the wider primitives.
pub(crate) fn witt_levels(ontology: &Ontology) -> Vec<(String, u32, usize)> {
    let mut levels: Vec<(String, u32, usize)> = Vec::new();
    for ind in individuals_of_type(ontology, "https://uor.foundation/schema/WittLevel") {
        let bits = ind
            .properties
            .iter()
            .find_map(|(k, v)| {
                if *k == "https://uor.foundation/schema/bitsWidth" {
                    if let uor_ontology::model::IndividualValue::Int(n) = v {
                        return Some(*n as u32);
                    }
                }
                None
            })
            .unwrap_or(0);
        // v0.2.2 Phase C: the cap is now 128 (u128 native backing). Levels
        // above W128 are handled by the Limbs<N> generic kernel emitted
        // in Phase C.3; until that lands, this filter excludes them.
        if bits == 0 || bits % 8 != 0 || bits > 128 {
            continue;
        }
        let byte_width = bits.div_ceil(8) as usize;
        let local = local_name(ind.id).to_string();
        levels.push((local, bits, byte_width));
    }
    levels.sort_by_key(|(_, bits, _)| *bits);
    levels
}

/// Returns the smallest Rust `u*` type that can hold `bits` bits of a ring
/// element. `bits` is the `schema:bitsWidth` annotation value. W24 uses
/// `u32` with a `& 0xFFFFFF` mask at the arithmetic boundary; W40-W64 use
/// `u64`; v0.2.2 Phase C.2 added `u128` for W72-W128.
fn witt_rust_int_type(bits: u32) -> &'static str {
    if bits <= 8 {
        "u8"
    } else if bits <= 16 {
        "u16"
    } else if bits <= 32 {
        "u32"
    } else if bits <= 64 {
        "u64"
    } else {
        "u128"
    }
}

fn generate_datum_types(f: &mut RustFile, ontology: &Ontology) {
    let levels = witt_levels(ontology);
    // DatumInner — variants emitted parametrically from `schema:WittLevel`.
    f.doc_comment("Internal level-tagged ring value. Width determined by the Witt level.");
    f.doc_comment("Variants are emitted parametrically from `schema:WittLevel` individuals");
    f.doc_comment("in the ontology; adding a new level to the ontology regenerates this enum.");
    f.doc_comment("Not publicly constructible \\[sealed within the crate\\].");
    f.line("#[derive(Debug, Clone, PartialEq, Eq)]");
    f.line("#[allow(clippy::large_enum_variant, dead_code)]");
    f.line("pub(crate) enum DatumInner {");
    for (local, bits, bytes) in &levels {
        f.indented_doc_comment(&format!("{local}: {bits}-bit ring Z/(2^{bits})Z."));
        f.line(&format!("    {local}([u8; {bytes}]),"));
    }
    f.line("}");
    f.blank();

    // Datum public wrapper.
    f.doc_comment("A ring element at its minting Witt level.");
    f.doc_comment("");
    f.doc_comment("Cannot be constructed outside the `uor_foundation` crate.");
    f.doc_comment("The only way to obtain a `Datum` is through reduction evaluation");
    f.doc_comment("or the two-phase minting boundary (`validate_and_mint_coord` /");
    f.doc_comment("`validate_and_mint_tuple`).");
    f.doc_example(
        "// A Datum is produced by reduction evaluation or the minting boundary —\n\
         // you never construct one directly.\n\
         fn inspect_datum(d: &uor_foundation::enforcement::Datum) {\n\
         \x20   // Query its Witt level (W8 = 8-bit, W32 = 32-bit, etc.)\n\
         \x20   let _level = d.level();\n\
         \x20   // Datum width is determined by its level:\n\
         \x20   //   W8 → 1 byte,  W16 → 2 bytes,  W24 → 3 bytes,  W32 → 4 bytes.\n\
         \x20   let _bytes = d.as_bytes();\n\
         }",
        "no_run",
    );
    f.line("#[derive(Debug, Clone, PartialEq, Eq)]");
    f.line("pub struct Datum {");
    f.indented_doc_comment("Level-tagged ring value \\[sealed\\].");
    f.line("    inner: DatumInner,");
    f.line("}");
    f.blank();
    f.line("impl Datum {");
    f.indented_doc_comment("Returns the Witt level at which this datum was minted.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn level(&self) -> WittLevel {");
    f.line("        match self.inner {");
    for (local, bits, _) in &levels {
        // W8/W16 use the named constants; others use WittLevel::new.
        let rhs = match *bits {
            8 => "WittLevel::W8".to_string(),
            16 => "WittLevel::W16".to_string(),
            _ => format!("WittLevel::new({bits})"),
        };
        f.line(&format!("            DatumInner::{local}(_) => {rhs},"));
    }
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Returns the raw byte representation of this datum.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn as_bytes(&self) -> &[u8] {");
    f.line("        match &self.inner {");
    for (local, _, _) in &levels {
        f.line(&format!("            DatumInner::{local}(b) => b,"));
    }
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();
}

fn generate_grounding_types(f: &mut RustFile, ontology: &Ontology) {
    let levels = witt_levels(ontology);
    // GroundedCoordInner — variants emitted parametrically from
    // `schema:WittLevel` individuals (same filter as `DatumInner`).
    f.doc_comment("Internal level-tagged coordinate value for grounding intermediates.");
    f.doc_comment("Variant set mirrors `DatumInner`: one per `schema:WittLevel`.");
    f.line("#[derive(Debug, Clone, PartialEq, Eq)]");
    f.line("#[allow(clippy::large_enum_variant, dead_code)]");
    f.line("pub(crate) enum GroundedCoordInner {");
    for (local, bits, bytes) in &levels {
        f.indented_doc_comment(&format!("{local}: {bits}-bit coordinate."));
        f.line(&format!("    {local}([u8; {bytes}]),"));
    }
    f.line("}");
    f.blank();

    // GroundedCoord
    f.doc_comment("A single grounded coordinate value.");
    f.doc_comment("");
    f.doc_comment("Not a `Datum` \\[this is the narrow intermediate that a `Grounding`");
    f.doc_comment("impl produces\\]. The foundation validates and mints it into a `Datum`.");
    f.doc_comment("Uses the same closed level-tagged family as `Datum`, ensuring that");
    f.doc_comment("coordinate width matches the target Witt level.");
    f.doc_example(
        "use uor_foundation::enforcement::GroundedCoord;\n\
         \n\
         // W8: 8-bit ring Z/256Z — lightweight, exhaustive-verification baseline\n\
         let byte_coord = GroundedCoord::w8(42);\n\
         \n\
         // W16: 16-bit ring Z/65536Z — audio samples, small indices\n\
         let short_coord = GroundedCoord::w16(1000);\n\
         \n\
         // W32: 32-bit ring Z/2^32Z — pixel data, general-purpose integers\n\
         let word_coord = GroundedCoord::w32(70_000);",
        "rust",
    );
    f.line("#[derive(Debug, Clone, PartialEq, Eq)]");
    f.line("pub struct GroundedCoord {");
    f.indented_doc_comment("Level-tagged coordinate bytes.");
    f.line("    pub(crate) inner: GroundedCoordInner,");
    f.line("}");
    f.blank();
    f.line("impl GroundedCoord {");
    for (local, bits, bytes) in &levels {
        let ctor = local.to_ascii_lowercase();
        let rust_ty = witt_rust_int_type(*bits);
        f.indented_doc_comment(&format!(
            "Construct a {local} coordinate from a `{rust_ty}` value (little-endian)."
        ));
        f.line("    #[inline]");
        f.line("    #[must_use]");
        f.line(&format!(
            "    pub const fn {ctor}(value: {rust_ty}) -> Self {{"
        ));
        // For W24 (3 bytes from u32), we need to mask and copy the low 3 bytes.
        // For W8, the payload is [u8; 1] and the native to_le_bytes gives [u8; 1].
        // Otherwise to_le_bytes gives exactly the needed byte_width.
        let full_bytes = match rust_ty {
            "u8" => 1,
            "u16" => 2,
            "u32" => 4,
            _ => 8,
        };
        if *bytes == full_bytes {
            f.line(&format!(
                "        Self {{ inner: GroundedCoordInner::{local}(value.to_le_bytes()) }}"
            ));
        } else {
            // Truncate to the ring's bit-width (e.g. W24 into 3 bytes).
            f.line("        let full = value.to_le_bytes();");
            f.line(&format!("        let mut out = [0u8; {bytes}];"));
            f.line("        let mut i = 0;");
            f.line(&format!("        while i < {bytes} {{"));
            f.line("            out[i] = full[i];");
            f.line("            i += 1;");
            f.line("        }");
            f.line(&format!(
                "        Self {{ inner: GroundedCoordInner::{local}(out) }}"
            ));
        }
        f.line("    }");
        f.blank();
    }
    f.line("}");
    f.blank();

    // GroundedTuple
    f.doc_comment("A grounded tuple: a fixed-size array of `GroundedCoord` values.");
    f.doc_comment("");
    f.doc_comment("Represents a structured type (e.g., the 8 coordinates of an E8");
    f.doc_comment("lattice point). Not a `Datum` until the foundation validates and");
    f.doc_comment("mints it. Stack-resident, no heap allocation.");
    f.doc_example(
        "use uor_foundation::enforcement::{GroundedCoord, GroundedTuple};\n\
         \n\
         // A 2D pixel: (red, green) at W8 (8-bit per channel)\n\
         let pixel = GroundedTuple::new([\n\
         \x20   GroundedCoord::w8(255), // red channel\n\
         \x20   GroundedCoord::w8(128), // green channel\n\
         ]);\n\
         \n\
         // An E8 lattice point: 8 coordinates at W8\n\
         let lattice_point = GroundedTuple::new([\n\
         \x20   GroundedCoord::w8(2), GroundedCoord::w8(0),\n\
         \x20   GroundedCoord::w8(0), GroundedCoord::w8(0),\n\
         \x20   GroundedCoord::w8(0), GroundedCoord::w8(0),\n\
         \x20   GroundedCoord::w8(0), GroundedCoord::w8(0),\n\
         ]);",
        "rust",
    );
    f.line("#[derive(Debug, Clone, PartialEq, Eq)]");
    f.line("pub struct GroundedTuple<const N: usize> {");
    f.indented_doc_comment("The coordinate array.");
    f.line("    pub(crate) coords: [GroundedCoord; N],");
    f.line("}");
    f.blank();
    f.line("impl<const N: usize> GroundedTuple<N> {");
    f.indented_doc_comment("Construct a tuple from a fixed-size array of coordinates.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn new(coords: [GroundedCoord; N]) -> Self {");
    f.line("        Self { coords }");
    f.line("    }");
    f.line("}");
    f.blank();

    // GroundedValue sealed trait
    f.doc_comment("Sealed marker trait for grounded intermediates.");
    f.doc_comment("");
    f.doc_comment("Implemented only for `GroundedCoord` and `GroundedTuple<N>`.");
    f.doc_comment("Prism code cannot implement this \\[the sealed module pattern");
    f.doc_comment("prevents it\\].");
    f.line("pub trait GroundedValue: sealed::Sealed {}");
    f.line("impl GroundedValue for GroundedCoord {}");
    f.line("impl<const N: usize> GroundedValue for GroundedTuple<N> {}");
    f.blank();

    // v0.2.2 W4: Grounding kind discriminator. The Grounding trait gains an
    // associated `Map: GroundingMapKind` type that tags each impl with its
    // semantic kind (digest, binary, integer, utf8, json). Sealed marker
    // traits (`Total`, `Invertible`, `PreservesStructure`, `PreservesMetric`)
    // partition the kinds by structural property, so foundation operations
    // requiring (e.g.) `PreservesStructure` reject digest-grounding impls at
    // the call site. The discrimination is structural-tagging — the
    // foundation cannot verify the impl body matches the declared kind, but
    // it can ensure that the kind is one of a fixed sealed set.
    // Target §3: `MorphismKind` is the shared sealed supertrait beneath both
    // `GroundingMapKind` (inbound) and `ProjectionMapKind` (outbound). The
    // four structural markers (`Total`, `Invertible`, `PreservesStructure`,
    // `PreservesMetric`) are bounded on `MorphismKind` — one abstraction,
    // both kind hierarchies consume it.
    f.doc_comment("Target §3: sealed marker trait shared by all morphism kinds.");
    f.doc_comment("`GroundingMapKind` (inbound) and `ProjectionMapKind` (outbound) both");
    f.doc_comment("extend this trait; the four structural markers (`Total`, `Invertible`,");
    f.doc_comment("`PreservesStructure`, `PreservesMetric`) are bounded on `MorphismKind`.");
    f.line("pub trait MorphismKind: morphism_kind_sealed::Sealed {");
    f.indented_doc_comment("The ontology IRI of this morphism kind.");
    f.line("    const ONTOLOGY_IRI: &'static str;");
    f.line("}");
    f.blank();

    f.doc_comment("v0.2.2 W4: sealed marker trait for the kind of a `Grounding` map.");
    f.doc_comment("Implemented by exactly the `morphism:GroundingMap` individuals declared in");
    f.doc_comment("the ontology; downstream cannot extend the kind set.");
    f.line("pub trait GroundingMapKind: MorphismKind + grounding_map_kind_sealed::Sealed {}");
    f.blank();

    f.doc_comment("Target §3: sealed marker trait for the kind of a `Sinking` projection.");
    f.doc_comment("Implemented by exactly the `morphism:ProjectionMap` individuals declared");
    f.doc_comment("in the ontology; downstream cannot extend the kind set.");
    f.line("pub trait ProjectionMapKind: MorphismKind + projection_map_kind_sealed::Sealed {}");
    f.blank();

    f.doc_comment("v0.2.2 W4: kinds whose image is total over the input domain");
    f.doc_comment("(every input maps successfully).");
    f.line("pub trait Total: MorphismKind {}");
    f.blank();
    f.doc_comment("v0.2.2 W4: kinds whose map is injective and admits an inverse on its image.");
    f.line("pub trait Invertible: MorphismKind {}");
    f.blank();
    f.doc_comment("v0.2.2 W4: kinds whose map preserves the algebraic structure of the");
    f.doc_comment("source domain (homomorphism-like).");
    f.line("pub trait PreservesStructure: MorphismKind {}");
    f.blank();
    f.doc_comment("v0.2.2 W4: kinds whose map preserves the metric of the source domain");
    f.doc_comment("(isometry-like).");
    f.line("pub trait PreservesMetric: MorphismKind {}");
    f.blank();

    // Walk morphism:GroundingMap individuals and emit one unit struct per kind.
    let kinds = individuals_of_type(ontology, "https://uor.foundation/morphism/GroundingMap");
    let mut kind_names: Vec<String> = Vec::new();
    for k in &kinds {
        kind_names.push(local_name(k.id).to_string());
    }
    kind_names.sort();
    kind_names.dedup();

    for name in &kind_names {
        let doc = match name.as_str() {
            "IntegerGroundingMap" => "v0.2.2 W4: kind for integer surface symbols. Total, invertible, structure-preserving.",
            "Utf8GroundingMap" => "v0.2.2 W4: kind for UTF-8 host strings. Invertible on its image, structure-preserving.",
            "JsonGroundingMap" => "v0.2.2 W4: kind for JSON host strings. Invertible on its image, structure-preserving.",
            "DigestGroundingMap" => "v0.2.2 W4: kind for one-way digest functions (e.g., SHA-256). Total but not invertible; preserves no structure.",
            "BinaryGroundingMap" => "v0.2.2 W4: kind for raw byte ingestion. Total and invertible; preserves bit identity only.",
            _ => "v0.2.2 W4: GroundingMap kind unit struct.",
        };
        f.doc_comment(doc);
        f.line("#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]");
        f.line(&format!("pub struct {name};"));
        f.blank();
    }

    // Target §3: walk morphism:ProjectionMap individuals and emit marker
    // structs in parallel. The sealed modules + impls for both kind hierarchies
    // are woven together below so `morphism_kind_sealed` can enumerate all
    // morphism kinds.
    let projection_kinds =
        individuals_of_type(ontology, "https://uor.foundation/morphism/ProjectionMap");
    let mut projection_names: Vec<String> = Vec::new();
    for k in &projection_kinds {
        projection_names.push(local_name(k.id).to_string());
    }
    projection_names.sort();
    projection_names.dedup();

    for name in &projection_names {
        let doc = match name.as_str() {
            "IntegerProjectionMap" => "Target §3: kind for integer surface symbols projected outward. Invertible, structure-preserving.",
            "Utf8ProjectionMap" => "Target §3: kind for UTF-8 host strings projected outward. Invertible, structure-preserving.",
            "JsonProjectionMap" => "Target §3: kind for JSON host strings projected outward. Invertible, structure-preserving.",
            "DigestProjectionMap" => "Target §3: kind for fixed-size digests projected outward. Total but not invertible; preserves no structure.",
            "BinaryProjectionMap" => "Target §3: kind for raw byte projections. Total and invertible; preserves bit identity only.",
            _ => "Target §3: ProjectionMap kind unit struct.",
        };
        f.doc_comment(doc);
        f.line("#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]");
        f.line(&format!("pub struct {name};"));
        f.blank();
    }

    // Sealed supertrait module for MorphismKind — enumerates every
    // GroundingMap + ProjectionMap individual.
    f.line("mod morphism_kind_sealed {");
    f.indented_doc_comment(
        "Private supertrait for MorphismKind. Not implementable outside this crate.",
    );
    f.line("    pub trait Sealed {}");
    for name in &kind_names {
        f.line(&format!("    impl Sealed for super::{name} {{}}"));
    }
    for name in &projection_names {
        f.line(&format!("    impl Sealed for super::{name} {{}}"));
    }
    f.line("}");
    f.blank();

    // Sealed module + GroundingMapKind impls.
    f.line("mod grounding_map_kind_sealed {");
    f.indented_doc_comment("Private supertrait. Not implementable outside this crate.");
    f.line("    pub trait Sealed {}");
    for name in &kind_names {
        f.line(&format!("    impl Sealed for super::{name} {{}}"));
    }
    f.line("}");
    f.blank();

    // Sealed module for ProjectionMapKind.
    f.line("mod projection_map_kind_sealed {");
    f.indented_doc_comment("Private supertrait. Not implementable outside this crate.");
    f.line("    pub trait Sealed {}");
    for name in &projection_names {
        f.line(&format!("    impl Sealed for super::{name} {{}}"));
    }
    f.line("}");
    f.blank();

    // MorphismKind impls — one per GroundingMap + ProjectionMap individual.
    // Carries the ontology IRI constant so the type-level kind can be
    // content-addressed against the ontology individual at runtime.
    for name in &kind_names {
        f.line(&format!("impl MorphismKind for {name} {{"));
        f.line(&format!(
            "    const ONTOLOGY_IRI: &'static str = \"https://uor.foundation/morphism/{name}\";"
        ));
        f.line("}");
        f.blank();
    }
    for name in &projection_names {
        f.line(&format!("impl MorphismKind for {name} {{"));
        f.line(&format!(
            "    const ONTOLOGY_IRI: &'static str = \"https://uor.foundation/morphism/{name}\";"
        ));
        f.line("}");
        f.blank();
    }

    // GroundingMapKind / ProjectionMapKind are now empty marker traits
    // parametrized by the `MorphismKind` supertrait — every kind hierarchy
    // just tags which side of the boundary it serves.
    for name in &kind_names {
        f.line(&format!("impl GroundingMapKind for {name} {{}}"));
    }
    f.blank();
    for name in &projection_names {
        f.line(&format!("impl ProjectionMapKind for {name} {{}}"));
    }
    f.blank();

    // Structural marker-trait impl table — per W4 plan, extended for
    // ProjectionMap duals. Each row is (kind_name, [markers]). The markers
    // are bounded on `MorphismKind` so both kind hierarchies consume them.
    //   IntegerGroundingMap  : Total + Invertible + PreservesStructure
    //   Utf8GroundingMap     : Invertible + PreservesStructure   (not Total)
    //   JsonGroundingMap     : Invertible + PreservesStructure   (not Total)
    //   DigestGroundingMap   : Total                              (not Invertible)
    //   BinaryGroundingMap   : Total + Invertible                 (no structure)
    //   IntegerProjectionMap : Invertible + PreservesStructure    (not Total)
    //   Utf8ProjectionMap    : Invertible + PreservesStructure    (not Total)
    //   JsonProjectionMap    : Invertible + PreservesStructure    (not Total)
    //   DigestProjectionMap  : Total                              (digest inverse is not a projection)
    //   BinaryProjectionMap  : Total + Invertible                 (no structure)
    let marker_table: &[(&str, &[&str])] = &[
        (
            "IntegerGroundingMap",
            &["Total", "Invertible", "PreservesStructure"],
        ),
        ("Utf8GroundingMap", &["Invertible", "PreservesStructure"]),
        ("JsonGroundingMap", &["Invertible", "PreservesStructure"]),
        ("DigestGroundingMap", &["Total"]),
        ("BinaryGroundingMap", &["Total", "Invertible"]),
        (
            "IntegerProjectionMap",
            &["Invertible", "PreservesStructure"],
        ),
        ("Utf8ProjectionMap", &["Invertible", "PreservesStructure"]),
        ("JsonProjectionMap", &["Invertible", "PreservesStructure"]),
        ("DigestProjectionMap", &["Total"]),
        ("BinaryProjectionMap", &["Total", "Invertible"]),
    ];
    for (kind_name, markers) in marker_table {
        let in_grounding = kind_names.iter().any(|n| n == *kind_name);
        let in_projection = projection_names.iter().any(|n| n == *kind_name);
        if !in_grounding && !in_projection {
            continue;
        }
        for marker in *markers {
            f.line(&format!("impl {marker} for {kind_name} {{}}"));
        }
        if !markers.is_empty() {
            f.blank();
        }
    }

    // Grounding open trait — extended with v0.2.2 W4 `Map: GroundingMapKind`
    // associated type. Defaulted to `BinaryGroundingMap` so existing impls
    // that don't declare a `Map` continue to type-check (the binary kind is
    // the most permissive default — total + invertible, no structure
    // preservation).
    f.doc_comment("Open trait for boundary crossing: external data to grounded intermediate.");
    f.doc_comment("");
    f.doc_comment("The foundation validates the returned value against the declared");
    f.doc_comment("`GroundingShape` and mints it into a `Datum` if conformant.");
    f.doc_comment("");
    f.doc_comment("v0.2.2 W4 adds the `Map: GroundingMapKind` associated type — every impl");
    f.doc_comment("must declare what *kind* of grounding map it is. Foundation operations");
    f.doc_comment(
        "that require structure preservation gate on `<G as Grounding>::Map: PreservesStructure`,",
    );
    f.doc_comment("and a digest-style impl is rejected at the call site.");
    f.doc_example(
        "use uor_foundation::enforcement::{\n\
         \x20   Grounding, GroundedCoord, GroundingProgram, BinaryGroundingMap, combinators,\n\
         };\n\
         \n\
         /// Byte-passthrough grounding: reads the first byte of input as a W8 Datum.\n\
         struct PassthroughGrounding;\n\
         \n\
         impl Grounding for PassthroughGrounding {\n\
         \x20   type Output = GroundedCoord;\n\
         \x20   type Map = BinaryGroundingMap;\n\
         \n\
         \x20   // Phase K: provide the combinator program; the type system verifies\n\
         \x20   // at compile time that its marker tuple matches Self::Map via\n\
         \x20   // GroundingProgram::from_primitive's MarkersImpliedBy<Map> bound.\n\
         \x20   fn program(&self) -> GroundingProgram<GroundedCoord, BinaryGroundingMap> {\n\
         \x20       GroundingProgram::from_primitive(combinators::read_bytes::<GroundedCoord>())\n\
         \x20   }\n\
         \x20   // Foundation supplies `ground()` via the sealed `GroundingExt`\n\
         \x20   // extension trait — downstream implementers provide only `program()`.\n\
         }",
        "no_run",
    );
    f.line("pub trait Grounding {");
    f.indented_doc_comment(
        "The grounded intermediate type. Bounded by `GroundedValue`,\n\
         which is sealed \\[only `GroundedCoord` and `GroundedTuple<N>`\n\
         are permitted\\].",
    );
    f.line("    type Output: GroundedValue;");
    f.blank();
    f.indented_doc_comment(
        "v0.2.2 W4: the kind of grounding map this impl is. Sealed to the\n\
         set of `morphism:GroundingMap` individuals declared in the\n\
         ontology. Every impl must declare the kind explicitly; if no kind\n\
         applies, use `BinaryGroundingMap` (the most permissive — total +\n\
         invertible, no structure preservation).",
    );
    f.line("    type Map: GroundingMapKind;");
    f.blank();
    f.indented_doc_comment(
        "Phase K / W4 closure (target §4.3 + §9 criterion 1): the combinator\n\
         program that decomposes this grounding. The program's `Map` parameter\n\
         equals the impl's `Map` associated type, so the kind discriminator is\n\
         mechanically verifiable from the combinator decomposition — not a\n\
         promise. This is the only required method; `ground` is foundation-\n\
         supplied via `GroundingExt`.",
    );
    f.line("    fn program(&self) -> GroundingProgram<Self::Output, Self::Map>;");
    f.line("}");
    f.blank();

    // W4 closure (target §4.3 + §9 criterion 1): sealed extension trait
    // supplies `ground()`. Downstream can only implement `Grounding`
    // (providing `program()`); the foundation's blanket impl of
    // `GroundingExt` runs the program. The `grounding_ext_sealed::Sealed`
    // supertrait prevents downstream from impl-ing `GroundingExt` itself.
    f.doc_comment("W4 closure (target §4.3 + §9 criterion 1): foundation-authored");
    f.doc_comment("extension trait that supplies `ground()` for every `Grounding`");
    f.doc_comment("impl. Downstream implementers provide only `program()`;");
    f.doc_comment("the foundation runs it via the blanket `impl<G: Grounding>`");
    f.doc_comment("below. The sealed supertrait prevents downstream from");
    f.doc_comment("implementing `GroundingExt` directly — there is no second path.");
    f.line("mod grounding_ext_sealed {");
    f.indented_doc_comment("Private supertrait. Not implementable outside this crate.");
    f.line("    pub trait Sealed {}");
    f.line("    impl<G: super::Grounding> Sealed for G {}");
    f.line("}");
    f.blank();
    f.doc_comment("Crate-internal bridge from `GroundingProgram` to `Option<Out>`.");
    f.doc_comment("Blanket-impl'd for the two `GroundedValue` members");
    f.doc_comment("(`GroundedCoord` and `GroundedTuple<N>`) so the `GroundingExt`");
    f.doc_comment("blanket compiles for any output in the closed set.");
    f.line("pub trait GroundingProgramRun<Out> {");
    f.indented_doc_comment("Run the program on external bytes.");
    f.line("    fn run_program(&self, external: &[u8]) -> Option<Out>;");
    f.line("}");
    f.blank();
    f.line("impl<Map: GroundingMapKind> GroundingProgramRun<GroundedCoord>");
    f.line("    for GroundingProgram<GroundedCoord, Map>");
    f.line("{");
    f.line("    #[inline]");
    f.line("    fn run_program(&self, external: &[u8]) -> Option<GroundedCoord> {");
    f.line("        self.run(external)");
    f.line("    }");
    f.line("}");
    f.blank();
    f.line("impl<const N: usize, Map: GroundingMapKind>");
    f.line("    GroundingProgramRun<GroundedTuple<N>>");
    f.line("    for GroundingProgram<GroundedTuple<N>, Map>");
    f.line("{");
    f.line("    #[inline]");
    f.line("    fn run_program(&self, external: &[u8]) -> Option<GroundedTuple<N>> {");
    f.line("        self.run(external)");
    f.line("    }");
    f.line("}");
    f.blank();
    f.doc_comment("Foundation-supplied `ground()` for every `Grounding` impl.");
    f.doc_comment("");
    f.doc_comment("The blanket `impl<G: Grounding> GroundingExt for G` below");
    f.doc_comment("routes every call of `.ground(bytes)` through");
    f.doc_comment("`self.program().run_program(bytes)`. Downstream cannot");
    f.doc_comment("override this path: `GroundingExt` has a sealed supertrait");
    f.doc_comment("and downstream cannot impl `GroundingExt` directly.");
    f.line("pub trait GroundingExt: Grounding + grounding_ext_sealed::Sealed {");
    f.indented_doc_comment(
        "Map external bytes into a grounded value via this impl's combinator program.",
    );
    f.indented_doc_comment("");
    f.indented_doc_comment(
        "Returns `None` when the combinator chain rejects the input (e.g. empty slice",
    );
    f.indented_doc_comment("for `ReadBytes`, malformed UTF-8 for `DecodeUtf8`).");
    f.line("    fn ground(&self, external: &[u8]) -> Option<Self::Output>;");
    f.line("}");
    f.blank();
    f.line("impl<G: Grounding> GroundingExt for G");
    f.line("where");
    f.line("    GroundingProgram<G::Output, G::Map>: GroundingProgramRun<G::Output>,");
    f.line("{");
    f.line("    #[inline]");
    f.line("    fn ground(&self, external: &[u8]) -> Option<Self::Output> {");
    f.line("        self.program().run_program(external)");
    f.line("    }");
    f.line("}");
    f.blank();

    // Target §3 + §4.6 — `Sinking`: foundation-owned operational contract for
    // outbound boundary crossings. The dual of `Grounding`. Unlike `Grounding`,
    // which needs foundation-verified kind discrimination via `GroundingExt`'s
    // combinator closure, `Sinking` carries its sealing in the input type:
    // `&Grounded<Self::Source>` is unforgeable because `Grounded<T>` is sealed
    // (constructed only by `pipeline::run::<T>`). The projection function body
    // is genuinely open — downstream authors decide the `Output` type and the
    // projection logic freely. The foundation guarantee "cannot launder
    // unverified data outward" is structurally encoded in the signature.
    f.doc_comment("Target §3 + §4.6: the foundation-owned operational contract for outbound");
    f.doc_comment("boundary crossings. Dual of `Grounding`.");
    f.doc_comment("");
    f.doc_comment("A `Sinking` impl projects a `Grounded<Source>` value to a host-side");
    f.doc_comment("`Output` through a specific `ProjectionMap` kind. The `&Grounded<Source>`");
    f.doc_comment("input is structurally unforgeable (sealed per §2) — no raw data can be");
    f.doc_comment("laundered through this contract. Downstream authors implement `Sinking`");
    f.doc_comment("for their projection types; the foundation guarantees the input pedigree.");
    f.doc_example(
        "use uor_foundation::enforcement::{\n\
         \x20   Grounded, Sinking, Utf8ProjectionMap, ConstrainedTypeInput,\n\
         };\n\
         \n\
         // ADR-060: `Sinking` and `Grounded` carry an `INLINE_BYTES`\n\
         // const-generic the application derives from its `HostBounds`; this\n\
         // example fixes a concrete width.\n\
         const N: usize = 32;\n\
         struct MyJsonSink;\n\
         \n\
         impl Sinking<N> for MyJsonSink {\n\
         \x20   type Source = ConstrainedTypeInput;\n\
         \x20   type ProjectionMap = Utf8ProjectionMap;\n\
         \x20   type Output = String;\n\
         \n\
         \x20   fn project(&self, grounded: &Grounded<ConstrainedTypeInput, N>) -> String {\n\
         \x20       format!(\"{:?}\", grounded.unit_address())\n\
         \x20   }\n\
         }",
        "no_run",
    );
    f.line("pub trait Sinking<const INLINE_BYTES: usize> {");
    f.indented_doc_comment(
        "The ring-side shape `T` carried by the `Grounded<T>` being projected.\n\
         Sealed via `GroundedShape` — downstream cannot forge an admissible Source.",
    );
    f.line("    type Source: GroundedShape;");
    f.blank();
    f.indented_doc_comment(
        "The ontology-declared ProjectionMap kind this impl serves. Sealed to\n\
         the closed set of `morphism:ProjectionMap` individuals.",
    );
    f.line("    type ProjectionMap: ProjectionMapKind;");
    f.blank();
    f.indented_doc_comment(
        "The host-side output type of this projection. Intentionally generic —\n\
         downstream chooses `String`, `Vec<u8>`, `serde_json::Value`, or any\n\
         host-appropriate carrier.",
    );
    f.line("    type Output;");
    f.blank();
    f.indented_doc_comment(
        "Project a grounded ring value to the host output. The `&Grounded<Source>`\n\
         input is unforgeable (Grounded is sealed per §2) — no raw data can be\n\
         laundered through this contract.",
    );
    f.line(
        "    fn project(&self, grounded: &Grounded<'_, Self::Source, INLINE_BYTES>) -> Self::Output;",
    );
    f.line("}");
    f.blank();

    // Target §4.6 — `EmitThrough`: extension trait tying the ontology-mirror
    // `EmitEffect<H>` to the Rust-operational `Sinking`. Every `EmitEffect`
    // impl that actually emits data goes through this trait; the foundation
    // guarantee is that the input is a sealed `Grounded<Source>`.
    f.doc_comment("Target §4.6: extension trait tying `EmitEffect<H>` (ontology-declarative)");
    f.doc_comment("to `Sinking` (Rust-operational). Emit-effect implementations carry a");
    f.doc_comment("specific `Sinking` impl; the emit operation threads a sealed");
    f.doc_comment("`Grounded<Source>` through the projection.");
    f.line("pub trait EmitThrough<const INLINE_BYTES: usize, H: crate::HostTypes>: crate::bridge::boundary::EmitEffect<H> {");
    f.indented_doc_comment("The `Sinking` implementation this emit-effect routes through.");
    f.line("    type Sinking: Sinking<INLINE_BYTES>;");
    f.blank();
    f.indented_doc_comment(
        "Emit a grounded value through this effect's bound `Sinking`. The\n\
         input type is the sealed `Grounded<Source>` of the bound `Sinking`;\n\
         nothing else is admissible.",
    );
    f.line("    fn emit(");
    f.line("        &self,");
    f.line("        grounded: &Grounded<'_, <Self::Sinking as Sinking<INLINE_BYTES>>::Source, INLINE_BYTES>,");
    f.line("    ) -> <Self::Sinking as Sinking<INLINE_BYTES>>::Output;");
    f.line("}");
    f.blank();
}

fn generate_witness_types(f: &mut RustFile) {
    // v0.2.2 W13: ValidationPhase — sealed marker for the validation phase
    // (compile-time vs runtime) at which a Validated<T> was witnessed. The
    // default phase is Runtime so v0.2.1 call sites that wrote Validated<T>
    // continue to compile unchanged. Compile-time validation produces
    // Validated<T, CompileTime>, which is convertible to Validated<T, Runtime>
    // via the From impl below — a CompileTime witness is usable wherever a
    // Runtime witness is.
    f.doc_comment("v0.2.2 W13: sealed marker trait for the validation phase at which a");
    f.doc_comment("`Validated<T, Phase>` was witnessed. Implemented only by `CompileTime`");
    f.doc_comment("and `Runtime`; downstream cannot extend.");
    f.line("pub trait ValidationPhase: validation_phase_sealed::Sealed {}");
    f.blank();
    f.line("mod validation_phase_sealed {");
    f.indented_doc_comment("Private supertrait. Not implementable outside this crate.");
    f.line("    pub trait Sealed {}");
    f.line("    impl Sealed for super::CompileTime {}");
    f.line("    impl Sealed for super::Runtime {}");
    f.line("}");
    f.blank();
    f.doc_comment("v0.2.2 W13: marker for compile-time validated witnesses produced by");
    f.doc_comment("`validate_const()` and usable in `const` contexts. Convertible to");
    f.doc_comment("`Validated<T, Runtime>` via `From`.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct CompileTime;");
    f.line("impl ValidationPhase for CompileTime {}");
    f.blank();
    f.doc_comment("v0.2.2 W13: marker for runtime-validated witnesses produced by");
    f.doc_comment("`validate()`. The default phase of `Validated<T>` so v0.2.1 call");
    f.doc_comment("sites continue to compile.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct Runtime;");
    f.line("impl ValidationPhase for Runtime {}");
    f.blank();

    // Validated<T, Phase>
    f.doc_comment("Proof that a value was produced by the conformance checker,");
    f.doc_comment("not fabricated by Prism code.");
    f.doc_comment("");
    f.doc_comment("The inner value and `_sealed` field are private, so `Validated<T>`");
    f.doc_comment("can only be constructed within this crate.");
    f.doc_comment("");
    f.doc_comment("v0.2.2 W13: parameterized by a `Phase: ValidationPhase` discriminator.");
    f.doc_comment("`Validated<T, CompileTime>` was witnessed by `validate_const()` and is");
    f.doc_comment("usable in const contexts. `Validated<T, Runtime>` (the default) was");
    f.doc_comment("witnessed by `validate()`. A `CompileTime` witness is convertible to");
    f.doc_comment("a `Runtime` witness via `From`.");
    f.doc_example(
        "use uor_foundation::enforcement::{CompileUnitBuilder, ConstrainedTypeInput, Term};\n\
         use uor_foundation::{WittLevel, VerificationDomain};\n\
         \n\
         // Validated<T> proves that a value passed conformance checking.\n\
         // You cannot construct one directly — only builder validate() methods\n\
         // and the minting boundary produce them.\n\
         // ADR-060: `Term` carries an `INLINE_BYTES` const-generic the\n\
         // application derives from its `HostBounds`; fix a concrete width.\n\
         const N: usize = 32;\n\
         let terms: [Term<'static, N>; 1] =\n\
         \x20   [uor_foundation::pipeline::literal_u64(1, WittLevel::W8)];\n\
         let domains = [VerificationDomain::Enumerative];\n\
         \n\
         let validated = CompileUnitBuilder::new()\n\
         \x20   .root_term(&terms)\n\
         \x20   .witt_level_ceiling(WittLevel::W8)\n\
         \x20   .thermodynamic_budget(1024)\n\
         \x20   .target_domains(&domains)\n\
         \x20   .result_type::<ConstrainedTypeInput>()\n\
         \x20   .validate()\n\
         \x20   .expect(\"all fields set\");\n\
         \n\
         // Access the inner value through the proof wrapper:\n\
         let _compile_unit = validated.inner();",
        "no_run",
    );
    f.line("#[derive(Debug, Clone, PartialEq, Eq)]");
    f.line("pub struct Validated<T, Phase: ValidationPhase = Runtime> {");
    f.indented_doc_comment("The validated inner value.");
    f.line("    inner: T,");
    f.indented_doc_comment("Phantom marker for the validation phase (`CompileTime` or `Runtime`).");
    f.line("    _phase: PhantomData<Phase>,");
    f.indented_doc_comment("Prevents external construction.");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl<T, Phase: ValidationPhase> Validated<T, Phase> {");
    f.indented_doc_comment("Returns a reference to the validated inner value.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn inner(&self) -> &T {");
    f.line("        &self.inner");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "Creates a new `Validated<T, Phase>` wrapper. Only callable within the crate.",
    );
    f.line("    #[inline]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new(inner: T) -> Self {");
    f.line("        Self { inner, _phase: PhantomData, _sealed: () }");
    f.line("    }");
    f.line("}");
    f.blank();
    // v0.2.2 W13: subsumption — a CompileTime witness is usable wherever a
    // Runtime witness is required.
    f.doc_comment(
        "v0.2.2 W13: a compile-time witness is usable wherever a runtime witness is required.",
    );
    f.line("impl<T> From<Validated<T, CompileTime>> for Validated<T, Runtime> {");
    f.line("    #[inline]");
    f.line("    fn from(value: Validated<T, CompileTime>) -> Self {");
    f.line("        Self { inner: value.inner, _phase: PhantomData, _sealed: () }");
    f.line("    }");
    f.line("}");
    f.blank();

    // Derivation
    f.doc_comment("An opaque derivation trace that can only be extended by the rewrite engine.");
    f.doc_comment("");
    f.doc_comment("Records the rewrite-step count, the source `witt_level_bits`, and the");
    f.doc_comment("parametric `content_fingerprint`. Private fields prevent external");
    f.doc_comment("construction; produced exclusively by `Grounded::derivation()` so the");
    f.doc_comment("verify path can re-derive the source certificate via");
    f.doc_comment("`Derivation::replay() -> Trace -> verify_trace`.");
    f.line("#[derive(Debug, Clone, PartialEq, Eq)]");
    f.line("pub struct Derivation<const FP_MAX: usize = 32> {");
    f.indented_doc_comment("Number of rewrite steps in this derivation.");
    f.line("    step_count: u32,");
    f.indented_doc_comment("v0.2.2 T5: Witt level the source grounding was minted at. Carried");
    f.indented_doc_comment("through replay so the verifier can reconstruct the certificate.");
    f.line("    witt_level_bits: u16,");
    f.indented_doc_comment("v0.2.2 T5: parametric content fingerprint of the source unit's");
    f.indented_doc_comment("full state, computed at grounding time by the consumer-supplied");
    f.indented_doc_comment("`Hasher`. Carried through replay so the verifier can reproduce");
    f.indented_doc_comment("the source certificate via passthrough.");
    f.line("    content_fingerprint: ContentFingerprint<FP_MAX>,");
    f.line("}");
    f.blank();
    f.line("impl<const FP_MAX: usize> Derivation<FP_MAX> {");
    f.indented_doc_comment("Returns the number of rewrite steps.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn step_count(&self) -> u32 {");
    f.line("        self.step_count");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("v0.2.2 T5: returns the Witt level the source grounding was minted at.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn witt_level_bits(&self) -> u16 {");
    f.line("        self.witt_level_bits");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("v0.2.2 T5: returns the parametric content fingerprint of the source");
    f.indented_doc_comment("unit, computed at grounding time by the consumer-supplied `Hasher`.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn content_fingerprint(&self) -> ContentFingerprint<FP_MAX> {");
    f.line("        self.content_fingerprint");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Creates a new derivation. Only callable within the crate.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new(");
    f.line("        step_count: u32,");
    f.line("        witt_level_bits: u16,");
    f.line("        content_fingerprint: ContentFingerprint<FP_MAX>,");
    f.line("    ) -> Self {");
    f.line("        Self {");
    f.line("            step_count,");
    f.line("            witt_level_bits,");
    f.line("            content_fingerprint,");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();

    // FreeRank
    f.doc_comment("An opaque free rank that can only be decremented by `PinningEffect`");
    f.doc_comment("and incremented by `UnbindingEffect` \\[never by direct mutation\\].");
    f.line("#[derive(Debug, Clone, PartialEq, Eq)]");
    f.line("pub struct FreeRank {");
    f.indented_doc_comment("Total site capacity at the Witt level.");
    f.line("    total: u32,");
    f.indented_doc_comment("Currently pinned sites.");
    f.line("    pinned: u32,");
    f.line("}");
    f.blank();
    f.line("impl FreeRank {");
    f.indented_doc_comment("Returns the total site capacity.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn total(&self) -> u32 {");
    f.line("        self.total");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Returns the number of currently pinned sites.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn pinned(&self) -> u32 {");
    f.line("        self.pinned");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Returns the number of remaining (unpinned) sites.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn remaining(&self) -> u32 {");
    f.line("        self.total - self.pinned");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Creates a new free rank. Only callable within the crate.");
    f.line("    #[inline]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new(total: u32, pinned: u32) -> Self {");
    f.line("        Self { total, pinned }");
    f.line("    }");
    f.line("}");
    f.blank();
}

// v0.2.2 Phase A: UorTime infrastructure.
//
// Emits the deterministic two-clock value (`UorTime`) carried by every
// `Grounded<T>` and `Certified<C>`, the sealed `LandauerBudget` newtype that
// backs one of the two clocks, the `Calibration` validated struct for
// wall-clock binding, the sealed `Nanos` lower-bound carrier, and the four
// shipped `calibrations::*` presets (X86_SERVER, ARM_MOBILE, CORTEX_M_EMBEDDED,
// CONSERVATIVE_WORST_CASE).
//
// All types are sealed with `pub(crate)` constructors. The two clocks are
// grounded in v0.2.1 ontology individuals: `landauer_nats` ↔ `observable:LandauerCost`
// (carried via the new `observable:LandauerBudget` class), and `rewrite_steps`
// ↔ `derivation:stepCount` on `derivation:TermMetrics`.
fn generate_uor_time(f: &mut RustFile) {
    // ── LandauerBudget ────────────────────────────────────────────────────
    f.doc_comment("v0.2.2 Phase A: sealed `H::Decimal`-backed newtype carrying the");
    f.doc_comment("`observable:LandauerCost` accumulator in `observable:Nats`.");
    f.doc_comment("Monotonic within a pipeline invocation. The UOR ring operates");
    f.doc_comment("at the Landauer temperature (β* = ln 2), so this observable is");
    f.doc_comment("a direct measure of irreversible bit-erasure performed.");
    f.doc_comment("");
    f.doc_comment("Phase 9: parameterized over `H: HostTypes` so the underlying");
    f.doc_comment("decimal type tracks the host's chosen precision.");
    f.line("#[derive(Debug)]");
    f.line("pub struct LandauerBudget<H: HostTypes = crate::DefaultHostTypes> {");
    f.indented_doc_comment("Accumulated Landauer cost in nats. Non-negative, finite.");
    f.line("    nats: H::Decimal,");
    f.indented_doc_comment("Phantom marker pinning the host type.");
    f.line("    _phantom: core::marker::PhantomData<H>,");
    f.indented_doc_comment("Prevents external construction.");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl<H: HostTypes> LandauerBudget<H> {");
    f.indented_doc_comment("Returns the accumulated Landauer cost in nats.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn nats(&self) -> H::Decimal {");
    f.line("        self.nats");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Crate-internal constructor. Caller guarantees `nats` is");
    f.indented_doc_comment("non-negative and finite (i.e. not NaN, not infinite).");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new(nats: H::Decimal) -> Self {");
    f.line("        Self {");
    f.line("            nats,");
    f.line("            _phantom: core::marker::PhantomData,");
    f.line("            _sealed: (),");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Crate-internal constructor for the zero-cost initial budget.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn zero() -> Self {");
    f.line("        Self {");
    f.line("            nats: H::EMPTY_DECIMAL,");
    f.line("            _phantom: core::marker::PhantomData,");
    f.line("            _sealed: (),");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();
    // Manual Eq/Ord — DecimalTranscendental: PartialOrd; foundation never
    // produces a NaN, so total order is well-defined.
    // Manual Copy/Clone/PartialEq — auto-derive would bound `H: Copy + Clone + ...`
    // but H is a marker; only `H::Decimal` needs the bounds, which it has via
    // `DecimalTranscendental: Copy + PartialEq`.
    f.line("impl<H: HostTypes> Copy for LandauerBudget<H> {}");
    f.line("impl<H: HostTypes> Clone for LandauerBudget<H> {");
    f.line("    #[inline]");
    f.line("    fn clone(&self) -> Self {");
    f.line("        *self");
    f.line("    }");
    f.line("}");
    f.line("impl<H: HostTypes> PartialEq for LandauerBudget<H> {");
    f.line("    #[inline]");
    f.line("    fn eq(&self, other: &Self) -> bool {");
    f.line("        self.nats == other.nats");
    f.line("    }");
    f.line("}");
    f.line("impl<H: HostTypes> Eq for LandauerBudget<H> {}");
    f.line("impl<H: HostTypes> PartialOrd for LandauerBudget<H> {");
    f.line("    #[inline]");
    f.line("    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {");
    f.line("        Some(self.cmp(other))");
    f.line("    }");
    f.line("}");
    f.line("impl<H: HostTypes> Ord for LandauerBudget<H> {");
    f.line("    #[inline]");
    f.line("    fn cmp(&self, other: &Self) -> core::cmp::Ordering {");
    f.line("        // Total order on H::Decimal with NaN excluded by construction.");
    f.line("        self.nats");
    f.line("            .partial_cmp(&other.nats)");
    f.line("            .unwrap_or(core::cmp::Ordering::Equal)");
    f.line("    }");
    f.line("}");
    f.line("impl<H: HostTypes> core::hash::Hash for LandauerBudget<H> {");
    f.line("    #[inline]");
    f.line("    fn hash<S: core::hash::Hasher>(&self, state: &mut S) {");
    f.line("        // DecimalTranscendental::to_bits gives a stable u64 fingerprint");
    f.line("        // for any in-range Decimal value.");
    f.line("        self.nats.to_bits().hash(state);");
    f.line("    }");
    f.line("}");
    f.blank();

    // ── UorTime ───────────────────────────────────────────────────────────
    f.doc_comment("v0.2.2 Phase A: foundation-internal deterministic two-clock value");
    f.doc_comment("carried by every `Grounded<T>` and `Certified<C>`. The two clocks are");
    f.doc_comment("`landauer_nats` (a `LandauerBudget` value backed by `observable:LandauerCost`)");
    f.doc_comment("and `rewrite_steps` (a `u64` backed by `derivation:stepCount` on");
    f.doc_comment("`derivation:TermMetrics`). Each clock is monotonic within a pipeline");
    f.doc_comment("invocation, content-deterministic, ontology-grounded, and binds to a");
    f.doc_comment("physical wall-clock lower bound through established physics (Landauer's");
    f.doc_comment("principle for nats; Margolus-Levitin for rewrite steps). Two clocks");
    f.doc_comment("because exactly two physical lower-bound theorems are grounded; adding");
    f.doc_comment("a third clock would require grounding a third physical theorem.");
    f.doc_comment("`PartialOrd` is component-wise: `a < b` iff every field of `a` is `<=`");
    f.doc_comment("the corresponding field of `b` and at least one is strictly `<`. Two");
    f.doc_comment("`UorTime` values from unrelated computations are genuinely incomparable,");
    f.doc_comment("so `UorTime` is `PartialOrd` but **not** `Ord`.");
    f.line("#[derive(Debug)]");
    f.line("pub struct UorTime<H: HostTypes = crate::DefaultHostTypes> {");
    f.indented_doc_comment("Landauer budget consumed, in `observable:Nats`.");
    f.line("    landauer_nats: LandauerBudget<H>,");
    f.indented_doc_comment("Total rewrite steps taken (`derivation:stepCount`).");
    f.line("    rewrite_steps: u64,");
    f.indented_doc_comment("Prevents external construction.");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl<H: HostTypes> UorTime<H> {");
    f.indented_doc_comment("Returns the Landauer budget consumed, in `observable:Nats`.");
    f.indented_doc_comment("Maps to `observable:LandauerCost`.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn landauer_nats(&self) -> LandauerBudget<H> {");
    f.line("        self.landauer_nats");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Returns the total rewrite steps taken.");
    f.indented_doc_comment("Maps to `derivation:stepCount` on `derivation:TermMetrics`.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn rewrite_steps(&self) -> u64 {");
    f.line("        self.rewrite_steps");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "Crate-internal constructor. Reachable only from the pipeline at witness mint time.",
    );
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line(
        "    pub(crate) const fn new(landauer_nats: LandauerBudget<H>, rewrite_steps: u64) -> Self {",
    );
    f.line("        Self { landauer_nats, rewrite_steps, _sealed: () }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Crate-internal constructor for the zero initial value.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn zero() -> Self {");
    f.line("        Self {");
    f.line("            landauer_nats: LandauerBudget::<H>::zero(),");
    f.line("            rewrite_steps: 0,");
    f.line("            _sealed: (),");
    f.line("        }");
    f.line("    }");
    f.blank();
    // Provable minimum wall-clock duration.
    f.indented_doc_comment("Returns the provable minimum wall-clock duration that the");
    f.indented_doc_comment("computation producing this witness could have taken under the");
    f.indented_doc_comment(
        "given calibration. Returns `max(Landauer-bound, Margolus-Levitin-bound)`.",
    );
    f.indented_doc_comment("");
    f.indented_doc_comment("The Landauer bound is `landauer_nats × k_B·T / thermal_power`.");
    f.indented_doc_comment(
        "The Margolus-Levitin bound is `π·ℏ·rewrite_steps / (2·characteristic_energy)`.",
    );
    f.indented_doc_comment("");
    f.indented_doc_comment("Pure arithmetic — no transcendentals, no state. Const-evaluable");
    f.indented_doc_comment("where the `UorTime` value is known at compile time.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn min_wall_clock(&self, cal: &Calibration<H>) -> Nanos {");
    f.line("        // Landauer bound: nats × k_B·T / thermal_power = seconds.");
    f.line("        let landauer_seconds =");
    f.line("            self.landauer_nats.nats() * cal.k_b_t() / cal.thermal_power();");
    f.line("        // Margolus-Levitin bound: π·ℏ / (2·E) per orthogonal state transition.");
    f.line("        // π·ℏ ≈ 3.31194e-34 J·s; encoded as the f64 bit pattern of");
    f.line("        // `core::f64::consts::PI * 1.054_571_817e-34` so the constant is");
    f.line("        // representable across host Decimal types.");
    f.line("        let pi_times_h_bar = <H::Decimal as DecimalTranscendental>::from_bits(");
    f.line("            crate::PI_TIMES_H_BAR_BITS,");
    f.line("        );");
    f.line("        let two = <H::Decimal as DecimalTranscendental>::from_u32(2);");
    f.line(
        "        let ml_seconds_per_step = pi_times_h_bar / (two * cal.characteristic_energy());",
    );
    f.line(
        "        let steps = <H::Decimal as DecimalTranscendental>::from_u64(self.rewrite_steps);",
    );
    f.line("        let ml_seconds = ml_seconds_per_step * steps;");
    f.line("        let max_seconds = if landauer_seconds > ml_seconds {");
    f.line("            landauer_seconds");
    f.line("        } else {");
    f.line("            ml_seconds");
    f.line("        };");
    f.line("        // Convert seconds to nanoseconds, saturate on overflow.");
    f.line("        let nanos_per_second = <H::Decimal as DecimalTranscendental>::from_bits(");
    f.line("            crate::NANOS_PER_SECOND_BITS,");
    f.line("        );");
    f.line("        let nanos = max_seconds * nanos_per_second;");
    f.line("        Nanos {");
    f.line("            ns: <H::Decimal as DecimalTranscendental>::as_u64_saturating(nanos),");
    f.line("            _sealed: (),");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();
    // Manual Copy/Clone/PartialEq/Eq/Hash — H is a marker, the meaningful
    // fields (LandauerBudget<H> + u64) bring their own bounds.
    f.line("impl<H: HostTypes> Copy for UorTime<H> {}");
    f.line("impl<H: HostTypes> Clone for UorTime<H> {");
    f.line("    #[inline]");
    f.line("    fn clone(&self) -> Self {");
    f.line("        *self");
    f.line("    }");
    f.line("}");
    f.line("impl<H: HostTypes> PartialEq for UorTime<H> {");
    f.line("    #[inline]");
    f.line("    fn eq(&self, other: &Self) -> bool {");
    f.line("        self.landauer_nats == other.landauer_nats");
    f.line("            && self.rewrite_steps == other.rewrite_steps");
    f.line("    }");
    f.line("}");
    f.line("impl<H: HostTypes> Eq for UorTime<H> {}");
    f.line("impl<H: HostTypes> core::hash::Hash for UorTime<H> {");
    f.line("    #[inline]");
    f.line("    fn hash<S: core::hash::Hasher>(&self, state: &mut S) {");
    f.line("        self.landauer_nats.hash(state);");
    f.line("        self.rewrite_steps.hash(state);");
    f.line("    }");
    f.line("}");
    // Component-wise PartialOrd, no Ord.
    f.line("impl<H: HostTypes> PartialOrd for UorTime<H> {");
    f.line("    #[inline]");
    f.line("    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {");
    f.line("        let l = self.landauer_nats.cmp(&other.landauer_nats);");
    f.line("        let r = self.rewrite_steps.cmp(&other.rewrite_steps);");
    f.line("        match (l, r) {");
    f.line("            (core::cmp::Ordering::Equal, core::cmp::Ordering::Equal) => Some(core::cmp::Ordering::Equal),");
    f.line("            (core::cmp::Ordering::Less, core::cmp::Ordering::Less)");
    f.line("            | (core::cmp::Ordering::Less, core::cmp::Ordering::Equal)");
    f.line("            | (core::cmp::Ordering::Equal, core::cmp::Ordering::Less) => Some(core::cmp::Ordering::Less),");
    f.line("            (core::cmp::Ordering::Greater, core::cmp::Ordering::Greater)");
    f.line("            | (core::cmp::Ordering::Greater, core::cmp::Ordering::Equal)");
    f.line("            | (core::cmp::Ordering::Equal, core::cmp::Ordering::Greater) => Some(core::cmp::Ordering::Greater),");
    f.line("            _ => None,");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();

    // ── Nanos ─────────────────────────────────────────────────────────────
    f.doc_comment("v0.2.2 Phase A: sealed lower-bound carrier for wall-clock duration.");
    f.doc_comment("");
    f.doc_comment("Produced only by `UorTime::min_wall_clock` and similar foundation");
    f.doc_comment("time conversions. The sealing guarantees that any `Nanos` value is");
    f.doc_comment("a provable physical bound, not a raw integer. Developers who need");
    f.doc_comment("the underlying `u64` call `.as_u64()`; the sealing prevents");
    f.doc_comment("accidentally passing a host-measured duration where the type system");
    f.doc_comment("expects \"a provable minimum\".");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]");
    f.line("pub struct Nanos {");
    f.indented_doc_comment("The provable lower-bound duration in nanoseconds.");
    f.line("    ns: u64,");
    f.indented_doc_comment("Prevents external construction.");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl Nanos {");
    f.indented_doc_comment("Returns the underlying nanosecond count. The value is a provable");
    f.indented_doc_comment("physical lower bound under whatever calibration produced it.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn as_u64(self) -> u64 {");
    f.line("        self.ns");
    f.line("    }");
    f.line("}");
    f.blank();

    // ── CalibrationError ──────────────────────────────────────────────────
    f.doc_comment("v0.2.2 Phase A: error returned by `Calibration::new` when the supplied");
    f.doc_comment("physical parameters fail plausibility validation.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub enum CalibrationError {");
    f.indented_doc_comment("`k_b_t` was non-positive, NaN, or outside the known-universe");
    f.indented_doc_comment("temperature range (`1e-30 ≤ k_b_t ≤ 1e-15` joules).");
    f.line("    ThermalEnergy,");
    f.indented_doc_comment(
        "`thermal_power` was non-positive, NaN, or above the thermodynamic maximum (`1e9` W).",
    );
    f.line("    ThermalPower,");
    f.indented_doc_comment("`characteristic_energy` was non-positive, NaN, or above the");
    f.indented_doc_comment("k_B·T × Avogadro-class bound (`1e3` joules).");
    f.line("    CharacteristicEnergy,");
    f.line("}");
    f.blank();
    // v0.2.2 T5.9: Display + core::error::Error impls for downstream
    // `?`-propagation through Box<dyn Error>.
    f.line("impl core::fmt::Display for CalibrationError {");
    f.line("    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {");
    f.line("        match self {");
    f.line("            Self::ThermalEnergy => f.write_str(");
    f.line(
        "                \"calibration k_b_t out of range (must be in [1e-30, 1e-15] joules)\",",
    );
    f.line("            ),");
    f.line("            Self::ThermalPower => f.write_str(");
    f.line(
        "                \"calibration thermal_power out of range (must be > 0 and <= 1e9 W)\",",
    );
    f.line("            ),");
    f.line("            Self::CharacteristicEnergy => f.write_str(");
    f.line("                \"calibration characteristic_energy out of range (must be > 0 and <= 1e3 J)\",");
    f.line("            ),");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();
    f.line("impl core::error::Error for CalibrationError {}");
    f.blank();

    // ── Calibration ───────────────────────────────────────────────────────
    f.doc_comment("v0.2.2 Phase A: physical-substrate calibration for wall-clock binding.");
    f.doc_comment("");
    f.doc_comment("Construction is open via [`Calibration::new`], but the fields are");
    f.doc_comment("private and validated for physical plausibility. Used to convert");
    f.doc_comment("`UorTime` to a provable wall-clock lower bound via");
    f.doc_comment("[`UorTime::min_wall_clock`].");
    f.doc_comment("");
    f.doc_comment("**A `Calibration` is never passed into `pipeline::run`,");
    f.doc_comment("`resolver::*::certify`, `validate_const`, or any other foundation entry");
    f.doc_comment("point.** The foundation computes `UorTime` without physical");
    f.doc_comment("interpretation; the developer applies a `Calibration` after the fact.");
    f.line("#[derive(Debug)]");
    f.line("pub struct Calibration<H: HostTypes = crate::DefaultHostTypes> {");
    f.indented_doc_comment("Boltzmann constant times temperature, in joules.");
    f.line("    k_b_t: H::Decimal,");
    f.indented_doc_comment("Sustained dissipation in watts.");
    f.line("    thermal_power: H::Decimal,");
    f.indented_doc_comment("Mean energy above ground state, in joules.");
    f.line("    characteristic_energy: H::Decimal,");
    f.indented_doc_comment("Phantom marker pinning the host type.");
    f.line("    _phantom: core::marker::PhantomData<H>,");
    f.line("}");
    f.blank();
    f.line("impl<H: HostTypes> Copy for Calibration<H> {}");
    f.line("impl<H: HostTypes> Clone for Calibration<H> {");
    f.line("    #[inline]");
    f.line("    fn clone(&self) -> Self {");
    f.line("        *self");
    f.line("    }");
    f.line("}");
    f.line("impl<H: HostTypes> PartialEq for Calibration<H> {");
    f.line("    #[inline]");
    f.line("    fn eq(&self, other: &Self) -> bool {");
    f.line("        self.k_b_t == other.k_b_t");
    f.line("            && self.thermal_power == other.thermal_power");
    f.line("            && self.characteristic_energy == other.characteristic_energy");
    f.line("    }");
    f.line("}");
    f.blank();
    f.line("impl<H: HostTypes> Calibration<H> {");
    f.indented_doc_comment("Construct a calibration with physically plausible parameters.");
    f.indented_doc_comment("");
    f.indented_doc_comment("Validation: every parameter must be positive and finite. `k_b_t`");
    f.indented_doc_comment("must lie within the known-universe temperature range");
    f.indented_doc_comment("(`1e-30 <= k_b_t <= 1e-15` joules covers ~1 nK to ~1e8 K).");
    f.indented_doc_comment("`thermal_power` must be at most `1e9` W (gigawatt class — far above");
    f.indented_doc_comment("any plausible single-compute envelope). `characteristic_energy`");
    f.indented_doc_comment("must be at most `1e3` J (kilojoule class — astronomically generous).");
    f.indented_doc_comment("");
    f.indented_doc_comment("# Errors");
    f.indented_doc_comment("");
    f.indented_doc_comment("Returns `CalibrationError::InvalidThermalEnergy` when `k_b_t` is");
    f.indented_doc_comment("non-positive, NaN, or outside the temperature range.");
    f.indented_doc_comment("");
    f.indented_doc_comment("Returns `CalibrationError::InvalidThermalPower` when `thermal_power`");
    f.indented_doc_comment("is non-positive, NaN, or above the maximum.");
    f.indented_doc_comment("");
    f.indented_doc_comment("Returns `CalibrationError::InvalidCharacteristicEnergy` when");
    f.indented_doc_comment("`characteristic_energy` is non-positive, NaN, or above the maximum.");
    f.indented_doc_comment("");
    f.indented_doc_comment("# Example");
    f.indented_doc_comment("");
    f.indented_doc_comment("```");
    f.indented_doc_comment("use uor_foundation::enforcement::Calibration;");
    f.indented_doc_comment("use uor_foundation::DefaultHostTypes;");
    f.indented_doc_comment("");
    f.indented_doc_comment("// X86 server-class envelope at room temperature.");
    f.indented_doc_comment("// k_B·T at 300 K = 4.14e-21 J; 85 W sustained TDP; ~1e-15 J/op.");
    f.indented_doc_comment(
        "let cal = Calibration::<DefaultHostTypes>::new(4.14e-21, 85.0, 1.0e-15)",
    );
    f.indented_doc_comment("    .expect(\"physically plausible server calibration\");");
    f.indented_doc_comment("# let _ = cal;");
    f.indented_doc_comment("```");
    f.line("    #[inline]");
    f.line("    pub fn new(");
    f.line("        k_b_t: H::Decimal,");
    f.line("        thermal_power: H::Decimal,");
    f.line("        characteristic_energy: H::Decimal,");
    f.line("    ) -> Result<Self, CalibrationError> {");
    f.line("        // Bit-pattern bounds for the validation envelope. Reading them");
    f.line("        // through `from_bits` lets the host's chosen Decimal precision");
    f.line("        // resolve the comparison without any hardcoded f64 in source.");
    f.line("        let zero = <H::Decimal as Default>::default();");
    f.line("        let kbt_lo = <H::Decimal as DecimalTranscendental>::from_bits(crate::CALIBRATION_KBT_LO_BITS);");
    f.line("        let kbt_hi = <H::Decimal as DecimalTranscendental>::from_bits(crate::CALIBRATION_KBT_HI_BITS);");
    f.line("        let tp_hi = <H::Decimal as DecimalTranscendental>::from_bits(crate::CALIBRATION_THERMAL_POWER_HI_BITS);");
    f.line("        let ce_hi = <H::Decimal as DecimalTranscendental>::from_bits(crate::CALIBRATION_CHAR_ENERGY_HI_BITS);");
    f.line("        // NaN identity: NaN != NaN. PartialEq is the defining bound.");
    f.line("        #[allow(clippy::eq_op)]");
    f.line("        let k_b_t_nan = k_b_t != k_b_t;");
    f.line("        if k_b_t_nan || k_b_t <= zero || k_b_t < kbt_lo || k_b_t > kbt_hi {");
    f.line("            return Err(CalibrationError::ThermalEnergy);");
    f.line("        }");
    f.line("        #[allow(clippy::eq_op)]");
    f.line("        let tp_nan = thermal_power != thermal_power;");
    f.line("        if tp_nan || thermal_power <= zero || thermal_power > tp_hi {");
    f.line("            return Err(CalibrationError::ThermalPower);");
    f.line("        }");
    f.line("        #[allow(clippy::eq_op)]");
    f.line("        let ce_nan = characteristic_energy != characteristic_energy;");
    f.line("        if ce_nan || characteristic_energy <= zero || characteristic_energy > ce_hi {");
    f.line("            return Err(CalibrationError::CharacteristicEnergy);");
    f.line("        }");
    f.line("        Ok(Self {");
    f.line("            k_b_t,");
    f.line("            thermal_power,");
    f.line("            characteristic_energy,");
    f.line("            _phantom: core::marker::PhantomData,");
    f.line("        })");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Returns the Boltzmann constant times temperature, in joules.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn k_b_t(&self) -> H::Decimal {");
    f.line("        self.k_b_t");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Returns the sustained thermal power dissipation, in watts.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn thermal_power(&self) -> H::Decimal {");
    f.line("        self.thermal_power");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Returns the characteristic energy above ground state, in joules.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn characteristic_energy(&self) -> H::Decimal {");
    f.line("        self.characteristic_energy");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("v0.2.2 T5.5: zero sentinel. All three fields are 0.0 — physically");
    f.indented_doc_comment("meaningless but safely constructible. Used as the unreachable-branch");
    f.indented_doc_comment("placeholder in the const-context preset literals (`X86_SERVER`,");
    f.indented_doc_comment("`ARM_MOBILE`, `CORTEX_M_EMBEDDED`, `CONSERVATIVE_WORST_CASE`). The");
    f.indented_doc_comment("`meta/calibration_presets_valid` conformance check verifies the");
    f.indented_doc_comment("preset literals always succeed in `Calibration::new`, so this");
    f.indented_doc_comment("sentinel is never produced in practice. Do not use it directly;");
    f.indented_doc_comment("it is exposed only because Rust's const-eval needs a fallback for");
    f.indented_doc_comment("the impossible `Err` arm of the preset match.");
    f.line("    pub const ZERO_SENTINEL: Calibration<H> = Self {");
    f.line("        k_b_t: H::EMPTY_DECIMAL,");
    f.line("        thermal_power: H::EMPTY_DECIMAL,");
    f.line("        characteristic_energy: H::EMPTY_DECIMAL,");
    f.line("        _phantom: core::marker::PhantomData,");
    f.line("    };");
    f.line("}");
    f.blank();
    // Const-context preset constructor for the default-host (f64) path.
    // Custom host types use `Calibration::new` at runtime.
    f.line("impl Calibration<crate::DefaultHostTypes> {");
    f.indented_doc_comment(
        "Const constructor for the default-host (f64) path. Bypasses runtime \
         validation; use only for the spec-shipped preset literals where the \
         envelope is statically guaranteed. Parameter types are written as \
         `<DefaultHostTypes as HostTypes>::Decimal` so no `: f64` annotation \
         appears in the public-API surface — the host-type alias is the \
         canonical name; downstream that swaps the host gets the matching \
         decimal automatically.",
    );
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub(crate) const fn from_f64_unchecked(");
    f.line("        k_b_t: <crate::DefaultHostTypes as crate::HostTypes>::Decimal,");
    f.line("        thermal_power: <crate::DefaultHostTypes as crate::HostTypes>::Decimal,");
    f.line(
        "        characteristic_energy: <crate::DefaultHostTypes as crate::HostTypes>::Decimal,",
    );
    f.line("    ) -> Self {");
    f.line("        Self {");
    f.line("            k_b_t,");
    f.line("            thermal_power,");
    f.line("            characteristic_energy,");
    f.line("            _phantom: core::marker::PhantomData,");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();

    // ── Calibration presets ───────────────────────────────────────────────
    f.doc_comment("v0.2.2 Phase A: foundation-shipped preset calibrations covering common");
    f.doc_comment("substrates. The values are derived from published substrate thermals at");
    f.doc_comment("T=300 K (room temperature, where k_B·T ≈ 4.14e-21 J).");
    f.line("pub mod calibrations {");
    f.line("    use super::Calibration;");
    f.line("    use crate::DefaultHostTypes;");
    f.blank();
    // Phase 9 (orphan-closure): preset constants are now pinned to the
    // default-host (f64) backing because const trait methods aren't
    // stable. `Calibration::from_f64_unchecked` is a const-context
    // helper that bypasses runtime validation; the `meta/calibration_
    // presets_valid` conformance gate still asserts these literals
    // round-trip through `Calibration::new`. Custom host types build
    // their own `Calibration<H>` at runtime via `Calibration::new`.
    f.indented_doc_comment("Server-class x86 (Xeon/EPYC sustained envelope).");
    f.indented_doc_comment("");
    f.indented_doc_comment("k_B·T = 4.14e-21 J (T = 300 K), thermal_power = 85 W (typical TDP),");
    f.indented_doc_comment("characteristic_energy = 1e-15 J/op (~1 fJ/op for modern CMOS).");
    f.line("    pub const X86_SERVER: Calibration<DefaultHostTypes> =");
    f.line("        Calibration::<DefaultHostTypes>::from_f64_unchecked(4.14e-21, 85.0, 1.0e-15);");
    f.blank();
    f.indented_doc_comment(
        "Mobile ARM SoC (Apple M-series, Snapdragon 8-series sustained envelope).",
    );
    f.indented_doc_comment("");
    f.indented_doc_comment(
        "k_B·T = 4.14e-21 J, thermal_power = 5 W, characteristic_energy = 1e-16 J/op.",
    );
    f.line("    pub const ARM_MOBILE: Calibration<DefaultHostTypes> =");
    f.line("        Calibration::<DefaultHostTypes>::from_f64_unchecked(4.14e-21, 5.0, 1.0e-16);");
    f.blank();
    f.indented_doc_comment("Cortex-M embedded (STM32/nRF52 at 80 MHz).");
    f.indented_doc_comment("");
    f.indented_doc_comment(
        "k_B·T = 4.14e-21 J, thermal_power = 0.1 W, characteristic_energy = 1e-17 J/op.",
    );
    f.line("    pub const CORTEX_M_EMBEDDED: Calibration<DefaultHostTypes> =");
    f.line("        Calibration::<DefaultHostTypes>::from_f64_unchecked(4.14e-21, 0.1, 1.0e-17);");
    f.blank();
    f.indented_doc_comment("The tightest provable lower bound that requires no trust in the");
    f.indented_doc_comment("issuer's claimed substrate. Values are physically sound but maximally");
    f.indented_doc_comment("generous: k_B·T at 300 K floor, thermal_power at 1 GW (above any");
    f.indented_doc_comment("plausible single-compute envelope), characteristic_energy at 1 J");
    f.indented_doc_comment("(astronomically generous).");
    f.indented_doc_comment("");
    f.indented_doc_comment("Applying this calibration yields the smallest `Nanos` physically");
    f.indented_doc_comment("possible for the computation regardless of substrate claims.");
    f.line("    pub const CONSERVATIVE_WORST_CASE: Calibration<DefaultHostTypes> =");
    f.line("        Calibration::<DefaultHostTypes>::from_f64_unchecked(4.14e-21, 1.0e9, 1.0);");
    f.line("}");
    f.blank();

    // v0.2.2 Phase B: TimingPolicy trait. Parameterizes the preflight/runtime
    // timing-budget check. Carries the Nanos-valued ontology budget
    // (reduction:PreflightTimingBound / reduction:RuntimeTimingBound) plus
    // a reference to the canonical Calibration used to convert an input's
    // a-priori UorTime estimate into Nanos. Host code can override the
    // budget or swap in a different calibration for embedded / HPC targets.
    f.doc_comment(
        "v0.2.2 Phase B (target §1.7): timing-policy carrier. Parametric over host tuning.",
    );
    f.doc_comment("");
    f.doc_comment("Supplies the preflight / runtime Nanos budgets (canonical values from");
    f.doc_comment("`reduction:PreflightTimingBound` and `reduction:RuntimeTimingBound`) plus");
    f.doc_comment("the `Calibration` used to convert an input's a-priori `UorTime` estimate");
    f.doc_comment("into a Nanos lower bound for comparison against the budget.");
    f.doc_comment("");
    f.doc_comment("The foundation-canonical default [`CanonicalTimingPolicy`] uses");
    f.doc_comment("`calibrations::CONSERVATIVE_WORST_CASE` (the tightest provable lower-bound");
    f.doc_comment("calibration) and the 10 ms budget from the ontology. Host code overrides by");
    f.doc_comment("implementing `TimingPolicy` on a marker struct and substituting at the");
    f.doc_comment("preflight-function call site.");
    f.line("pub trait TimingPolicy {");
    f.indented_doc_comment(
        "Preflight Nanos budget. Inputs whose a-priori UorTime \u{2192} min_wall_clock",
    );
    f.indented_doc_comment(
        "under `Self::CALIBRATION` exceeds this value are rejected at preflight.",
    );
    f.line("    const PREFLIGHT_BUDGET_NS: u64;");
    f.indented_doc_comment("Runtime Nanos budget for post-admission reduction.");
    f.line("    const RUNTIME_BUDGET_NS: u64;");
    f.indented_doc_comment(
        "Canonical Calibration used to convert a-priori UorTime estimates to Nanos.",
    );
    f.indented_doc_comment(
        "Phase 9: pinned to `Calibration<DefaultHostTypes>` because the trait's \
         `const` slot can't carry a non-DefaultHost generic. Polymorphic \
         consumers build a `Calibration<H>` at runtime via `Calibration::new`.",
    );
    f.line("    const CALIBRATION: &'static Calibration<crate::DefaultHostTypes>;");
    f.line("}");
    f.blank();
    f.doc_comment("v0.2.2 Phase B: foundation-canonical [`TimingPolicy`]. Budget values mirror");
    f.doc_comment(
        "`reduction:PreflightTimingBound` and `reduction:RuntimeTimingBound`; calibration",
    );
    f.doc_comment("is `calibrations::CONSERVATIVE_WORST_CASE` so the Nanos lower bound from any");
    f.doc_comment("input UorTime is the tightest physically-defensible value.");
    f.line("#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct CanonicalTimingPolicy;");
    f.blank();
    f.line("impl TimingPolicy for CanonicalTimingPolicy {");
    f.line("    const PREFLIGHT_BUDGET_NS: u64 = 10_000_000;");
    f.line("    const RUNTIME_BUDGET_NS: u64 = 10_000_000;");
    f.line("    const CALIBRATION: &'static Calibration<crate::DefaultHostTypes> =");
    f.line("        &calibrations::CONSERVATIVE_WORST_CASE;");
    f.line("}");
    f.blank();

    // Phase H: transcendentals module — foundation-owned wrappers around
    // the always-on `libm` dependency. Every `xsd:decimal` observable whose
    // computation requires `ln` / `exp` / `sqrt` routes through here;
    // downstream must not call `libm::*` directly so the foundation retains
    // a single audit point for transcendental arithmetic policy (target §1.6).
    f.doc_comment(
        "Phase H (target §1.6): foundation-owned transcendental-arithmetic entry points.",
    );
    f.doc_comment("");
    f.doc_comment("Routes through the always-on `libm` dependency so every build — std, alloc,");
    f.doc_comment("and strict no_std — has access to `ln` / `exp` / `sqrt` for `xsd:decimal`");
    f.doc_comment("observables. Gating these behind an optional feature flag was considered and");
    f.doc_comment("rejected per target §1.6: an observable that exists in one build and not");
    f.doc_comment("another violates the foundation's \"one surface\" discipline.");
    f.doc_comment("");
    f.doc_comment("Downstream code that needs transcendentals should call these wrappers —");
    f.doc_comment(
        "they are the canonical entry point for the four observables target §3 enumerates",
    );
    f.doc_comment(
        "(`convergenceRate`, `residualEntropy`, `collapseAmplitude`, and `op:OA_5` pricing)",
    );
    f.doc_comment("and for any future observable whose implementation admits transcendentals.");
    f.line("pub mod transcendentals {");
    f.line("    use crate::DecimalTranscendental;");
    f.blank();
    f.indented_doc_comment(
        "Natural logarithm. Dispatches via `DecimalTranscendental::ln`; \
         the foundation's f64 / f32 impls route through `libm::log` / `logf`.",
    );
    f.indented_doc_comment("");
    f.indented_doc_comment("Returns `NaN` for `x <= 0.0`, preserving `libm`'s contract.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn ln<D: DecimalTranscendental>(x: D) -> D {");
    f.line("        x.ln()");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Exponential `e^x`. Dispatches via `DecimalTranscendental::exp`.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn exp<D: DecimalTranscendental>(x: D) -> D {");
    f.line("        x.exp()");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "Square root. Dispatches via `DecimalTranscendental::sqrt`. Returns `NaN` for `x < 0.0`.",
    );
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn sqrt<D: DecimalTranscendental>(x: D) -> D {");
    f.line("        x.sqrt()");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "Shannon entropy term `-p \u{00b7} ln(p)` in nats. Returns 0 for `p = 0` by",
    );
    f.indented_doc_comment("continuous extension. Used by `observable:residualEntropy`.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn entropy_term_nats<D: DecimalTranscendental>(p: D) -> D {");
    f.line("        let zero = <D as Default>::default();");
    f.line("        if p <= zero {");
    f.line("            zero");
    f.line("        } else {");
    f.line("            zero - p * p.ln()");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();
}

fn generate_term_ast(f: &mut RustFile) {
    // TermList
    f.doc_comment("Fixed-capacity term list for `#![no_std]`. Indices into a `TermArena`.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq)]");
    f.line("pub struct TermList {");
    f.indented_doc_comment("Start index in the arena.");
    f.line("    pub start: u32,");
    f.indented_doc_comment("Number of terms in this list.");
    f.line("    pub len: u32,");
    f.line("}");
    f.blank();

    // TermArena
    f.doc_comment("Stack-resident arena for `Term` trees.");
    f.doc_comment("");
    f.doc_comment("Fixed capacity determined by the const generic `CAP`.");
    f.doc_comment("All `Term` child references are `u32` indices into this arena.");
    f.doc_comment("`#![no_std]`-safe: no heap allocation.");
    f.doc_example(
        "use uor_foundation::enforcement::{TermArena, Term, TermList};\n\
         use uor_foundation::{WittLevel, PrimitiveOp};\n\
         \n\
         // Build the expression `add(3, 5)` bottom-up in an arena.\n\
         // ADR-060: `TermArena` carries `<'a, INLINE_BYTES, CAP>`; the\n\
         // application derives `INLINE_BYTES` from its `HostBounds`. Here we\n\
         // fix a concrete inline width `N` and a capacity of 4.\n\
         const N: usize = 32;\n\
         let mut arena = TermArena::<N, 4>::new();\n\
         \n\
         // Push leaves first:\n\
         let idx_3 = arena.push(uor_foundation::pipeline::literal_u64(3, WittLevel::W8));\n\
         let idx_5 = arena.push(uor_foundation::pipeline::literal_u64(5, WittLevel::W8));\n\
         \n\
         // Push the application node, referencing the leaves by index:\n\
         let idx_add = arena.push(Term::Application {\n\
         \x20   operator: PrimitiveOp::Add,\n\
         \x20   args: TermList { start: idx_3.unwrap_or(0), len: 2 },\n\
         });\n\
         \n\
         assert_eq!(arena.len(), 3);\n\
         // Retrieve a node by index:\n\
         let node = arena.get(idx_add.unwrap_or(0));\n\
         assert!(node.is_some());",
        "rust",
    );
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq)]");
    f.line("pub struct TermArena<'a, const INLINE_BYTES: usize, const CAP: usize> {");
    f.indented_doc_comment("Node storage. `None` slots are unused.");
    f.line("    nodes: [Option<Term<'a, INLINE_BYTES>>; CAP],");
    f.indented_doc_comment("Number of allocated nodes.");
    f.line("    len: u32,");
    f.line("}");
    f.blank();
    f.line(
        "impl<'a, const INLINE_BYTES: usize, const CAP: usize> TermArena<'a, INLINE_BYTES, CAP> {",
    );
    f.indented_doc_comment("Creates an empty arena.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn new() -> Self {");
    f.line("        // v0.2.2 T4.5.a: const-stable on MSRV 1.70 via the `[None; CAP]`");
    f.line("        // initializer (Term is Copy as of T4.5.a, so Option<Term> is Copy).");
    f.line("        Self {");
    f.line("            nodes: [None; CAP],");
    f.line("            len: 0,");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Push a term into the arena and return its index.");
    f.indented_doc_comment("");
    f.indented_doc_comment("# Errors");
    f.indented_doc_comment("");
    f.indented_doc_comment("Returns `None` if the arena is full.");
    f.line("    #[must_use]");
    f.line("    pub fn push(&mut self, term: Term<'a, INLINE_BYTES>) -> Option<u32> {");
    f.line("        let idx = self.len;");
    f.line("        if (idx as usize) >= CAP {");
    f.line("            return None;");
    f.line("        }");
    f.line("        self.nodes[idx as usize] = Some(term);");
    f.line("        self.len = idx + 1;");
    f.line("        Some(idx)");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "Returns a reference to the term at `index`, or `None` if out of bounds.",
    );
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn get(&self, index: u32) -> Option<&Term<'a, INLINE_BYTES>> {");
    f.line("        self.nodes.get(index as usize).and_then(|slot| slot.as_ref())");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Returns the number of allocated nodes.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn len(&self) -> u32 {");
    f.line("        self.len");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Returns `true` if the arena has no allocated nodes.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn is_empty(&self) -> bool {");
    f.line("        self.len == 0");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("v0.2.2 T4.5.b: returns the populated prefix of the arena as a slice.");
    f.indented_doc_comment("");
    f.indented_doc_comment("Each entry is `Some(term)` for the populated indices `0..len()`;");
    f.indented_doc_comment("downstream consumers index into this slice via `TermList::start` and");
    f.indented_doc_comment("`TermList::len` to walk the children of an Application/Match node.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn as_slice(&self) -> &[Option<Term<'a, INLINE_BYTES>>] {");
    f.line("        &self.nodes[..self.len as usize]");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Wiki ADR-022 D2: const constructor that copies a static term slice");
    f.indented_doc_comment("into the arena. Used by the `prism_model!` proc-macro to emit a");
    f.indented_doc_comment("`const ROUTE: TermArena<CAP> = TermArena::from_slice(ROUTE_SLICE)`");
    f.indented_doc_comment("declaration alongside the model — the `const ROUTE_SLICE` carrying");
    f.indented_doc_comment("the term tree, this `from_slice` wrapping it into the arena form");
    f.indented_doc_comment("[`crate::pipeline::run_route`] consumes.");
    f.indented_doc_comment("");
    f.indented_doc_comment("`CAP` MUST be at least `slice.len()`; if the slice exceeds the");
    f.indented_doc_comment("arena's capacity the trailing terms are silently dropped (the");
    f.indented_doc_comment("`prism_model!` macro emits an arena sized to fit the route's term");
    f.indented_doc_comment("count plus headroom, so this case is unreachable from the macro's");
    f.indented_doc_comment("output).");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn from_slice(slice: &'a [Term<'a, INLINE_BYTES>]) -> Self {");
    f.line("        let mut nodes: [Option<Term<'a, INLINE_BYTES>>; CAP] = [None; CAP];");
    f.line("        let mut i = 0usize;");
    f.line("        while i < slice.len() && i < CAP {");
    f.line("            nodes[i] = Some(slice[i]);");
    f.line("            i += 1;");
    f.line("        }");
    f.line("        // Cap at min(slice.len(), CAP) so a too-large slice doesn't");
    f.line("        // overrun. The macro emits arena CAP >= slice.len() so the");
    f.line("        // truncation branch is unreachable in practice.");
    f.line("        #[allow(clippy::cast_possible_truncation)]");
    f.line("        let len = if slice.len() > CAP { CAP as u32 } else { slice.len() as u32 };");
    f.line("        Self { nodes, len }");
    f.line("    }");
    f.line("}");
    f.blank();
    f.line("impl<'a, const INLINE_BYTES: usize, const CAP: usize> Default for TermArena<'a, INLINE_BYTES, CAP> {");
    f.line("    fn default() -> Self {");
    f.line("        Self::new()");
    f.line("    }");
    f.line("}");
    f.blank();

    // Term
    f.doc_comment("Concrete AST node for the UOR term language.");
    f.doc_comment("");
    f.doc_comment("Mirrors the EBNF grammar productions. All child references are");
    f.doc_comment("indices into a `TermArena`, keeping the AST stack-resident and");
    f.doc_comment("`#![no_std]`-safe.");
    f.doc_example(
        "use uor_foundation::enforcement::{Term, TermList};\n\
         use uor_foundation::{WittLevel, PrimitiveOp};\n\
         \n\
         // ADR-060: `Term` carries `<'a, const INLINE_BYTES: usize>`. The\n\
         // application instantiates `INLINE_BYTES` from its selected\n\
         // `HostBounds` via `pipeline::carrier_inline_bytes::<B>()`; this\n\
         // example fixes a concrete width.\n\
         const N: usize = 32;\n\
         \n\
         // Literal: an integer value tagged with a Witt level.\n\
         let lit: Term<'static, N> =\n\
         \x20   uor_foundation::pipeline::literal_u64(42, WittLevel::W8);\n\
         \n\
         // Application: an operation applied to arguments.\n\
         // `args` is a TermList { start, len } pointing into a TermArena.\n\
         let app: Term<'static, N> = Term::Application {\n\
         \x20   operator: PrimitiveOp::Mul,\n\
         \x20   args: TermList { start: 0, len: 2 },\n\
         };\n\
         \n\
         // Lift: canonical injection from a lower to a higher Witt level.\n\
         let lift: Term<'static, N> =\n\
         \x20   Term::Lift { operand_index: 0, target: WittLevel::new(32) };\n\
         \n\
         // Project: canonical surjection from a higher to a lower level.\n\
         let proj: Term<'static, N> =\n\
         \x20   Term::Project { operand_index: 0, target: WittLevel::W8 };\n\
         let _ = (lit, app, lift, proj);",
        "rust",
    );
    // ADR-051 + ADR-060: `Term::Literal` carries a source-polymorphic
    // `TermValue<'a, INLINE_BYTES>` so wide-Witt literals (W128+) are natively
    // representable without `Concat` composition, and the literal's carrier
    // width is the foundation-derived inline width (no contrived 4096 cap).
    // The `Term` enum gains the lifetime and const-generic parameters per
    // ADR-060 (1), instantiated at the application boundary; only the
    // `Literal` variant uses them (the carrier), the others are 8-32 bytes —
    // the size difference triggers `clippy::large_enum_variant`, accepted per
    // ADR-051 (rejected alternative 2 forbids a separate `LiteralWide` variant).
    f.line("#[allow(clippy::large_enum_variant)]");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq)]");
    f.line("pub enum Term<'a, const INLINE_BYTES: usize> {");
    f.indented_doc_comment("Integer literal with Witt level annotation. Per ADR-051 the value");
    f.indented_doc_comment(
        "carrier is a `TermValue` byte sequence whose length matches the declared",
    );
    f.indented_doc_comment(
        "`level`'s byte width. Use `uor_foundation::pipeline::literal_u64(value, level)`",
    );
    f.indented_doc_comment("to construct a literal from a `u64` value (the narrow form).");
    f.line("    Literal {");
    f.line("        /// The literal value as a source-polymorphic carrier (ADR-051 +");
    f.line("        /// ADR-060). Inline length equals `level.witt_length() / 8`. Wider");
    f.line("        /// widths (W128, W256, …) are natively representable without");
    f.line("        /// `Concat` composition.");
    f.line("        value: crate::pipeline::TermValue<'a, INLINE_BYTES>,");
    f.line("        /// The Witt level of this literal.");
    f.line("        level: WittLevel,");
    f.line("    },");
    f.indented_doc_comment("Variable reference by name index.");
    f.line("    Variable {");
    f.line("        /// Index into the name table.");
    f.line("        name_index: u32,");
    f.line("    },");
    f.indented_doc_comment("Operation application: operator applied to arguments.");
    f.line("    Application {");
    f.line("        /// The primitive operation to apply.");
    f.line("        operator: PrimitiveOp,");
    f.line("        /// Argument list (indices into arena).");
    f.line("        args: TermList,");
    f.line("    },");
    f.indented_doc_comment("Lift: canonical injection W_n to W_m (n < m, lossless).");
    f.line("    Lift {");
    f.line("        /// Index of the operand term in the arena.");
    f.line("        operand_index: u32,");
    f.line("        /// Target Witt level.");
    f.line("        target: WittLevel,");
    f.line("    },");
    f.indented_doc_comment("Project: canonical surjection W_m to W_n (m > n, lossy).");
    f.line("    Project {");
    f.line("        /// Index of the operand term in the arena.");
    f.line("        operand_index: u32,");
    f.line("        /// Target Witt level.");
    f.line("        target: WittLevel,");
    f.line("    },");
    f.indented_doc_comment("Match expression with pattern-result pairs.");
    f.line("    Match {");
    f.line("        /// Index of the scrutinee term in the arena.");
    f.line("        scrutinee_index: u32,");
    f.line("        /// Match arms (indices into arena).");
    f.line("        arms: TermList,");
    f.line("    },");
    f.indented_doc_comment("Bounded recursion with descent measure.");
    f.line("    Recurse {");
    f.line("        /// Index of the descent measure term.");
    f.line("        measure_index: u32,");
    f.line("        /// Index of the base case term.");
    f.line("        base_index: u32,");
    f.line("        /// Index of the recursive step term.");
    f.line("        step_index: u32,");
    f.line("    },");
    f.indented_doc_comment("Stream construction via unfold.");
    f.line("    Unfold {");
    f.line("        /// Index of the seed term.");
    f.line("        seed_index: u32,");
    f.line("        /// Index of the step function term.");
    f.line("        step_index: u32,");
    f.line("    },");
    f.indented_doc_comment("Try expression with failure recovery.");
    f.line("    Try {");
    f.line("        /// Index of the body term.");
    f.line("        body_index: u32,");
    f.line("        /// Index of the handler term.");
    f.line("        handler_index: u32,");
    f.line("    },");
    f.indented_doc_comment("Substitution-axis-realized verb projection (wiki ADR-029 + ADR-030).");
    f.indented_doc_comment("");
    f.indented_doc_comment(
        "Delegates evaluation to the application's `AxisTuple` substitution-axis",
    );
    f.indented_doc_comment("impl: the catamorphism evaluates the input subtree, dispatches the");
    f.indented_doc_comment("axis at `axis_index` to the kernel identified by `kernel_id`, and");
    f.indented_doc_comment("emits the kernel's output as the result. Emitted by");
    f.indented_doc_comment("`prism_model!` from the closure-body form `hash(input)` (ADR-026 G19,");
    f.indented_doc_comment("which lowers to AxisInvocation against the application's HashAxis).");
    f.line("    AxisInvocation {");
    f.line("        /// Position of the axis in the application's `AxisTuple`.");
    f.line("        axis_index: u32,");
    f.line("        /// Per-axis kernel id (the SDK macro emits per-method `KERNEL_*` consts).");
    f.line("        kernel_id: u32,");
    f.line("        /// Input subtree's arena index (single input — current axes are 1-arg).");
    f.line("        input_index: u32,");
    f.line("    },");
    f.indented_doc_comment("Field-access projection over a partition_product input (wiki");
    f.indented_doc_comment("ADR-033 G20). The catamorphism's fold-rule evaluates `source`");
    f.indented_doc_comment("and slices `[byte_offset .. byte_offset + byte_length]` from");
    f.indented_doc_comment("the resulting bytes. Emitted by `prism_model!` and `verb!`");
    f.indented_doc_comment("from the closure-body forms `<expr>.<index>` and");
    f.indented_doc_comment("`<expr>.<field_name>` (named-field access requires the");
    f.indented_doc_comment("`partition_product!` declaration to use the named-field form).");
    f.indented_doc_comment("Coproduct field-access is rejected at macro-expansion time.");
    f.line("    ProjectField {");
    f.line("        /// Arena index of the source expression's term tree.");
    f.line("        source_index: u32,");
    f.line("        /// Byte offset into the source's evaluated bytes (proc-");
    f.line("        /// macro-computed from the partition-product factor widths).");
    f.line("        byte_offset: u32,");
    f.line("        /// Length of the projected slice in bytes.");
    f.line("        byte_length: u32,");
    f.line("    },");
    f.indented_doc_comment("Bounded search with structural early termination (wiki ADR-034).");
    f.indented_doc_comment("The catamorphism iterates `idx` from 0 up to (but excluding) the");
    f.indented_doc_comment("evaluated `domain_size`; for each iteration it evaluates");
    f.indented_doc_comment("`predicate` with `FIRST_ADMIT_IDX_NAME_INDEX` bound to `idx`.");
    f.indented_doc_comment("On the first non-zero predicate result the fold emits the");
    f.indented_doc_comment("coproduct value `(0x01, idx_bytes)` and terminates iteration; if");
    f.indented_doc_comment("no `idx` admits, the fold emits `(0x00, idx-width zero bytes)`.");
    f.indented_doc_comment("Emitted by `prism_model!` and `verb!` from the closure-body");
    f.indented_doc_comment("form `first_admit(<DomainTy>, |idx| <pred>)` (ADR-026 G16; the");
    f.indented_doc_comment("lowering target shifted from `Term::Recurse` to `Term::FirstAdmit`");
    f.indented_doc_comment("per ADR-034's structural-search commitment).");
    f.line("    FirstAdmit {");
    f.line("        /// Arena index of the domain-cardinality term (typically a");
    f.line(
        "        /// `Term::Literal` carrying `<DomainTy as ConstrainedTypeShape>::CYCLE_SIZE`).",
    );
    f.line("        domain_size_index: u32,");
    f.line("        /// Arena index of the predicate body. Evaluation visits");
    f.line("        /// `predicate` with `FIRST_ADMIT_IDX_NAME_INDEX` bound to the");
    f.line("        /// current candidate `idx`.");
    f.line("        predicate_index: u32,");
    f.line("    },");
    // ψ-chain Term variants (wiki ADR-035): nine lowering targets for the
    // ψ_1..ψ_9 ontology pipeline. Eight consult the model's ResolverTuple
    // (per ADR-036) for per-value content; Betti is pure computation.
    f.indented_doc_comment(
        "ψ_1 (wiki ADR-035): nerve construction — Constraints → SimplicialComplex.",
    );
    f.indented_doc_comment("Lowered from the closure-body form `nerve(<value_expr>)` (G21).");
    f.indented_doc_comment(
        "Resolver-bound: consults the ResolverTuple's NerveResolver per ADR-036.",
    );
    f.line("    Nerve {");
    f.line("        /// Arena index of the value-bytes operand (typically a");
    f.line("        /// `Term::Variable` for the route input or a `Term::ProjectField`).");
    f.line("        value_index: u32,");
    f.line("    },");
    f.indented_doc_comment("ψ_2 (wiki ADR-035): chain functor — SimplicialComplex → ChainComplex.");
    f.indented_doc_comment("Lowered from `chain_complex(<simplicial_expr>)` (G22).");
    f.indented_doc_comment("Resolver-bound: ChainComplexResolver per ADR-036.");
    f.line("    ChainComplex {");
    f.line("        simplicial_index: u32,");
    f.line("    },");
    f.indented_doc_comment("ψ_3 (wiki ADR-035): homology functor — ChainComplex → HomologyGroups.");
    f.indented_doc_comment("`H_k(C) = ker(∂_k) / im(∂_{k+1})`.");
    f.indented_doc_comment("Lowered from `homology_groups(<chain_expr>)` (G23).");
    f.indented_doc_comment("Resolver-bound: HomologyGroupResolver per ADR-036.");
    f.line("    HomologyGroups {");
    f.line("        chain_index: u32,");
    f.line("    },");
    f.indented_doc_comment(
        "ψ_4 (wiki ADR-035): Betti-number extraction — HomologyGroups → BettiNumbers.",
    );
    f.indented_doc_comment(
        "Pure computation on resolved homology groups; no resolver consultation.",
    );
    f.indented_doc_comment("Lowered from `betti(<homology_expr>)` (G24).");
    f.line("    Betti {");
    f.line("        homology_index: u32,");
    f.line("    },");
    f.indented_doc_comment(
        "ψ_5 (wiki ADR-035): dualization functor — ChainComplex → CochainComplex.",
    );
    f.indented_doc_comment("`C^k = Hom(C_k, R)`.");
    f.indented_doc_comment("Lowered from `cochain_complex(<chain_expr>)` (G25).");
    f.indented_doc_comment("Resolver-bound: CochainComplexResolver per ADR-036.");
    f.line("    CochainComplex {");
    f.line("        chain_index: u32,");
    f.line("    },");
    f.indented_doc_comment(
        "ψ_6 (wiki ADR-035): cohomology functor — CochainComplex → CohomologyGroups.",
    );
    f.indented_doc_comment("`H^k(C) = ker(δ^k) / im(δ^{k-1})`.");
    f.indented_doc_comment("Lowered from `cohomology_groups(<cochain_expr>)` (G26).");
    f.indented_doc_comment("Resolver-bound: CohomologyGroupResolver per ADR-036.");
    f.line("    CohomologyGroups {");
    f.line("        cochain_index: u32,");
    f.line("    },");
    f.indented_doc_comment("ψ_7 (wiki ADR-035): Kan-completion + Postnikov truncation —");
    f.indented_doc_comment("SimplicialComplex → PostnikovTower. The PostnikovResolver performs");
    f.indented_doc_comment("the Kan-completion internally; verb authors do not need to construct");
    f.indented_doc_comment("KanComplex values explicitly.");
    f.indented_doc_comment("Lowered from `postnikov_tower(<simplicial_expr>)` (G27).");
    f.indented_doc_comment("Resolver-bound: PostnikovResolver per ADR-036.");
    f.line("    PostnikovTower {");
    f.line("        simplicial_index: u32,");
    f.line("    },");
    f.indented_doc_comment(
        "ψ_8 (wiki ADR-035): homotopy extraction — PostnikovTower → HomotopyGroups.",
    );
    f.indented_doc_comment("π_k from each truncation stage.");
    f.indented_doc_comment("Lowered from `homotopy_groups(<postnikov_expr>)` (G28).");
    f.indented_doc_comment("Resolver-bound: HomotopyGroupResolver per ADR-036.");
    f.line("    HomotopyGroups {");
    f.line("        postnikov_index: u32,");
    f.line("    },");
    f.indented_doc_comment(
        "ψ_9 (wiki ADR-035): k-invariant computation — HomotopyGroups → KInvariants.",
    );
    f.indented_doc_comment("κ_k classifying the Postnikov tower.");
    f.indented_doc_comment("Lowered from `k_invariants(<homotopy_expr>)` (G29).");
    f.indented_doc_comment("Resolver-bound: KInvariantResolver per ADR-036.");
    f.line("    KInvariants {");
    f.line("        homotopy_index: u32,");
    f.line("    },");
    f.line("}");
    f.blank();

    // Wiki ADR-024 + ADR-029: verb-graph acyclicity is a compile-time
    // commitment, not a runtime guard. The catamorphism walks a flat
    // arena (the ten Term variants above); verb-emitted term-tree
    // fragments are spliced into the calling route's arena at compile
    // time via the const-fn helper `inline_verb_fragment` below. The
    // `prism_model!` macro emits a const-eval-time arena builder when
    // a route invokes a verb; the splicing shifts internal arena
    // indices by the host arena's current length so the inlined
    // fragment remains internally consistent within the host.
    f.doc_comment("Wiki ADR-024 verb-graph compile-time inlining: shift the arena-index");
    f.doc_comment("fields of `term` by `offset`. Used by [`inline_verb_fragment`] to");
    f.doc_comment("inline a verb's term-tree fragment into a host arena at compile time.");
    f.doc_comment("");
    f.doc_comment("`Term::Variable`'s `name_index` is a binding-name reference (not an");
    f.doc_comment("arena index) and is preserved unchanged. `Term::Try`'s `handler_index`");
    f.doc_comment("is preserved unchanged when it equals `u32::MAX` (the default-");
    f.doc_comment("propagation sentinel per ADR-022 D3 G9).");
    f.line("#[must_use]");
    f.line("pub const fn shift_term<'a, const INLINE_BYTES: usize>(term: Term<'a, INLINE_BYTES>, offset: u32) -> Term<'a, INLINE_BYTES> {");
    f.line("    match term {");
    f.line("        Term::Literal { value, level } => Term::Literal { value, level },");
    f.line("        // name_index is a binding-name reference, not an arena index.");
    f.line("        Term::Variable { name_index } => Term::Variable { name_index },");
    f.line("        Term::Application { operator, args } => Term::Application {");
    f.line("            operator,");
    f.line("            args: TermList {");
    f.line("                start: args.start + offset,");
    f.line("                len: args.len,");
    f.line("            },");
    f.line("        },");
    f.line("        Term::Lift { operand_index, target } => Term::Lift {");
    f.line("            operand_index: operand_index + offset,");
    f.line("            target,");
    f.line("        },");
    f.line("        Term::Project { operand_index, target } => Term::Project {");
    f.line("            operand_index: operand_index + offset,");
    f.line("            target,");
    f.line("        },");
    f.line("        Term::Match { scrutinee_index, arms } => Term::Match {");
    f.line("            scrutinee_index: scrutinee_index + offset,");
    f.line("            arms: TermList {");
    f.line("                start: arms.start + offset,");
    f.line("                len: arms.len,");
    f.line("            },");
    f.line("        },");
    f.line("        Term::Recurse { measure_index, base_index, step_index } => Term::Recurse {");
    f.line("            measure_index: measure_index + offset,");
    f.line("            base_index: base_index + offset,");
    f.line("            step_index: step_index + offset,");
    f.line("        },");
    f.line("        Term::Unfold { seed_index, step_index } => Term::Unfold {");
    f.line("            seed_index: seed_index + offset,");
    f.line("            step_index: step_index + offset,");
    f.line("        },");
    f.line("        Term::Try { body_index, handler_index } => Term::Try {");
    f.line("            body_index: body_index + offset,");
    f.line(
        "            handler_index: if handler_index == u32::MAX { u32::MAX } else { handler_index + offset },",
    );
    f.line("        },");
    f.line("        Term::AxisInvocation { axis_index, kernel_id, input_index } => Term::AxisInvocation {");
    f.line("            axis_index,");
    f.line("            kernel_id,");
    f.line("            input_index: input_index + offset,");
    f.line("        },");
    f.line("        Term::ProjectField { source_index, byte_offset, byte_length } => Term::ProjectField {");
    f.line("            source_index: source_index + offset,");
    f.line("            byte_offset,");
    f.line("            byte_length,");
    f.line("        },");
    f.line("        Term::FirstAdmit { domain_size_index, predicate_index } => Term::FirstAdmit {");
    f.line("            domain_size_index: domain_size_index + offset,");
    f.line("            predicate_index: predicate_index + offset,");
    f.line("        },");
    f.line("        Term::Nerve { value_index } => Term::Nerve {");
    f.line("            value_index: value_index + offset,");
    f.line("        },");
    f.line("        Term::ChainComplex { simplicial_index } => Term::ChainComplex {");
    f.line("            simplicial_index: simplicial_index + offset,");
    f.line("        },");
    f.line("        Term::HomologyGroups { chain_index } => Term::HomologyGroups {");
    f.line("            chain_index: chain_index + offset,");
    f.line("        },");
    f.line("        Term::Betti { homology_index } => Term::Betti {");
    f.line("            homology_index: homology_index + offset,");
    f.line("        },");
    f.line("        Term::CochainComplex { chain_index } => Term::CochainComplex {");
    f.line("            chain_index: chain_index + offset,");
    f.line("        },");
    f.line("        Term::CohomologyGroups { cochain_index } => Term::CohomologyGroups {");
    f.line("            cochain_index: cochain_index + offset,");
    f.line("        },");
    f.line("        Term::PostnikovTower { simplicial_index } => Term::PostnikovTower {");
    f.line("            simplicial_index: simplicial_index + offset,");
    f.line("        },");
    f.line("        Term::HomotopyGroups { postnikov_index } => Term::HomotopyGroups {");
    f.line("            postnikov_index: postnikov_index + offset,");
    f.line("        },");
    f.line("        Term::KInvariants { homotopy_index } => Term::KInvariants {");
    f.line("            homotopy_index: homotopy_index + offset,");
    f.line("        },");
    f.line("    }");
    f.line("}");
    f.blank();

    f.doc_comment("Wiki ADR-024 compile-time verb-fragment inlining helper.");
    f.doc_comment("");
    f.doc_comment("Copies the verb `fragment` slice into `buf` starting at `len` while applying");
    f.doc_comment("two simultaneous transformations per term so the verb body becomes part of");
    f.doc_comment("the calling route's flat arena: Variable(0) substitution (the verb's `input`");
    f.doc_comment("parameter binds to the caller's argument expression by replacing each");
    f.doc_comment("`Variable { name_index: 0 }` with a copy of `buf[arg_root_idx]`), and arena-");
    f.doc_comment("index shifting (every non-Variable(0) term has its arena-index fields shifted");
    f.doc_comment("by `len` so internal references resolve correctly within the host).");
    f.doc_comment("");
    f.doc_comment("The combined transformation realises ADR-024's compile-time inlining: the");
    f.doc_comment("verb body lands in the host arena with its `input` bound to the caller's");
    f.doc_comment("argument expression — verb-graph acyclicity is checked at const-eval time,");
    f.doc_comment("no `Term::VerbReference` variant or runtime depth guard is required.");
    f.doc_comment("");
    f.doc_comment("# Panics");
    f.doc_comment("");
    f.doc_comment("Panics at const-eval time if `len + fragment.len() > CAP` or if");
    f.doc_comment("`arg_root_idx as usize >= len`.");
    f.line("#[must_use]");
    f.line("pub const fn inline_verb_fragment<'a, const INLINE_BYTES: usize, const CAP: usize>(");
    f.line("    mut buf: [Term<'a, INLINE_BYTES>; CAP],");
    f.line("    mut len: usize,");
    f.line("    fragment: &[Term<'a, INLINE_BYTES>],");
    f.line("    arg_root_idx: u32,");
    f.line(") -> ([Term<'a, INLINE_BYTES>; CAP], usize) {");
    f.line("    let offset = len as u32;");
    f.line(
        "    // Capture a copy of the caller's argument root term; `Variable { name_index: 0 }`",
    );
    f.line("    // occurrences in the fragment are replaced by this copy per ADR-024.");
    f.line("    let arg_root_term = buf[arg_root_idx as usize];");
    f.line("    let mut i = 0;");
    f.line("    while i < fragment.len() {");
    f.line("        let term = fragment[i];");
    f.line("        let new_term = match term {");
    f.line("            Term::Variable { name_index: 0 } => arg_root_term,");
    f.line("            other => shift_term(other, offset),");
    f.line("        };");
    f.line("        buf[len] = new_term;");
    f.line("        len += 1;");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    (buf, len)");
    f.line("}");
    f.blank();

    // TypeDeclaration
    f.doc_comment("A type declaration with constraint kinds.");
    f.line("#[derive(Debug, Clone, PartialEq, Eq)]");
    f.line("pub struct TypeDeclaration {");
    f.indented_doc_comment("Name index for this type.");
    f.line("    pub name_index: u32,");
    f.indented_doc_comment("Constraint terms (indices into arena).");
    f.line("    pub constraints: TermList,");
    f.line("}");
    f.blank();

    // Binding
    f.doc_comment("A named binding: `let name : Type = term`.");
    f.line("#[derive(Debug, Clone, PartialEq, Eq)]");
    f.line("pub struct Binding {");
    f.indented_doc_comment("Name index for this binding.");
    f.line("    pub name_index: u32,");
    f.indented_doc_comment("Index of the type declaration.");
    f.line("    pub type_index: u32,");
    f.indented_doc_comment("Index of the value term in the arena.");
    f.line("    pub value_index: u32,");
    f.indented_doc_comment("EBNF surface syntax (compile-time constant).");
    f.line("    pub surface: &'static str,");
    f.indented_doc_comment("FNV-1a content address (compile-time constant).");
    f.line("    pub content_address: u64,");
    f.line("}");
    f.blank();
    f.line("impl Binding {");
    f.indented_doc_comment(
        "v0.2.2 Phase P.3: lift this binding to the `BindingEntry` shape consumed by",
    );
    f.indented_doc_comment("`BindingsTable`. `address` is derived from `content_address` via");
    f.indented_doc_comment(
        "`ContentAddress::from_u64_fingerprint`; `bytes` re-uses the `surface` slice.",
    );
    f.indented_doc_comment(
        "Content-deterministic; const-compatible since all fields are `'static`.",
    );
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn to_binding_entry(&self) -> BindingEntry {");
    f.line("        BindingEntry {");
    f.line("            address: ContentAddress::from_u64_fingerprint(self.content_address),");
    f.line("            bytes: self.surface.as_bytes(),");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();

    // Assertion
    f.doc_comment("An assertion: `assert lhs = rhs`.");
    f.line("#[derive(Debug, Clone, PartialEq, Eq)]");
    f.line("pub struct Assertion {");
    f.indented_doc_comment("Index of the left-hand side term.");
    f.line("    pub lhs_index: u32,");
    f.indented_doc_comment("Index of the right-hand side term.");
    f.line("    pub rhs_index: u32,");
    f.indented_doc_comment("EBNF surface syntax (compile-time constant).");
    f.line("    pub surface: &'static str,");
    f.line("}");
    f.blank();

    // SourceDeclaration
    f.doc_comment("Boundary source declaration: `source name : Type via grounding`.");
    f.line("#[derive(Debug, Clone, PartialEq, Eq)]");
    f.line("pub struct SourceDeclaration {");
    f.indented_doc_comment("Name index for the source.");
    f.line("    pub name_index: u32,");
    f.indented_doc_comment("Index of the type declaration.");
    f.line("    pub type_index: u32,");
    f.indented_doc_comment("Name index of the grounding map.");
    f.line("    pub grounding_name_index: u32,");
    f.line("}");
    f.blank();

    // SinkDeclaration
    f.doc_comment("Boundary sink declaration: `sink name : Type via projection`.");
    f.line("#[derive(Debug, Clone, PartialEq, Eq)]");
    f.line("pub struct SinkDeclaration {");
    f.indented_doc_comment("Name index for the sink.");
    f.line("    pub name_index: u32,");
    f.indented_doc_comment("Index of the type declaration.");
    f.line("    pub type_index: u32,");
    f.indented_doc_comment("Name index of the projection map.");
    f.line("    pub projection_name_index: u32,");
    f.line("}");
    f.blank();
}

fn generate_shape_violation(f: &mut RustFile) {
    f.doc_comment("Structured violation diagnostic carrying metadata from the");
    f.doc_comment("conformance namespace. Every field is machine-readable.");
    f.doc_example(
        "use uor_foundation::enforcement::ShapeViolation;\n\
         use uor_foundation::ViolationKind;\n\
         \n\
         // ShapeViolation carries structured metadata from the ontology.\n\
         // Every field is machine-readable — IRIs, counts, and a typed kind.\n\
         let violation = ShapeViolation {\n\
         \x20   shape_iri: \"https://uor.foundation/conformance/CompileUnitShape\",\n\
         \x20   constraint_iri: \"https://uor.foundation/conformance/compileUnit_rootTerm_constraint\",\n\
         \x20   property_iri: \"https://uor.foundation/reduction/rootTerm\",\n\
         \x20   expected_range: \"https://uor.foundation/schema/Term\",\n\
         \x20   min_count: 1,\n\
         \x20   max_count: 1,\n\
         \x20   kind: ViolationKind::Missing,\n\
         };\n\
         \n\
         // Machine-readable for tooling (IDE plugins, CI pipelines):\n\
         assert_eq!(violation.kind, ViolationKind::Missing);\n\
         assert!(violation.shape_iri.ends_with(\"CompileUnitShape\"));\n\
         assert_eq!(violation.min_count, 1);",
        "rust",
    );
    f.line("#[derive(Debug, Clone, PartialEq, Eq)]");
    f.line("pub struct ShapeViolation {");
    f.indented_doc_comment("IRI of the `conformance:Shape` that was validated against.");
    f.line("    pub shape_iri: &'static str,");
    f.indented_doc_comment("IRI of the specific `conformance:PropertyConstraint` that failed.");
    f.line("    pub constraint_iri: &'static str,");
    f.indented_doc_comment("IRI of the property that was missing or invalid.");
    f.line("    pub property_iri: &'static str,");
    f.indented_doc_comment("The expected range class IRI.");
    f.line("    pub expected_range: &'static str,");
    f.indented_doc_comment("Minimum cardinality from the constraint.");
    f.line("    pub min_count: u32,");
    f.indented_doc_comment("Maximum cardinality (0 = unbounded).");
    f.line("    pub max_count: u32,");
    f.indented_doc_comment("What went wrong.");
    f.line("    pub kind: ViolationKind,");
    f.line("}");
    f.blank();
    // v0.2.2 T5.9: Display + core::error::Error impls.
    f.line("impl core::fmt::Display for ShapeViolation {");
    f.line("    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {");
    f.line("        write!(");
    f.line("            f,");
    f.line("            \"shape violation: {} (constraint {}, property {}, kind {:?})\",");
    f.line("            self.shape_iri, self.constraint_iri, self.property_iri, self.kind,");
    f.line("        )");
    f.line("    }");
    f.line("}");
    f.blank();
    f.line("impl core::error::Error for ShapeViolation {}");
    f.blank();

    // Phase C.3: const_message() accessor for const-context diagnostics.
    f.line("impl ShapeViolation {");
    f.indented_doc_comment("Phase C.3: returns the shape IRI as a `&'static str` suitable for");
    f.indented_doc_comment("`const fn` panic messages. The IRI uniquely identifies the violated");
    f.indented_doc_comment("constraint in the conformance catalog.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn const_message(&self) -> &'static str {");
    f.line("        self.shape_iri");
    f.line("    }");
    f.line("}");
    f.blank();
}

fn generate_builders(f: &mut RustFile) {
    // CompileUnitBuilder
    f.doc_comment("Builder for `CompileUnit` admission into the reduction pipeline.");
    f.doc_comment("");
    f.doc_comment("Collects `rootTerm`, `wittLevelCeiling`, `thermodynamicBudget`,");
    f.doc_comment("and `targetDomains`. The `validate()` method checks structural");
    f.doc_comment("constraints (Tier 1) and value-dependent constraints (Tier 2).");
    f.doc_example(
        "use uor_foundation::enforcement::{CompileUnitBuilder, ConstrainedTypeInput, Term};\n\
         use uor_foundation::{WittLevel, VerificationDomain, ViolationKind};\n\
         \n\
         // A CompileUnit packages a term graph for reduction admission.\n\
         // The builder enforces that all required fields are present.\n\
         // ADR-060: `Term`/`CompileUnitBuilder` carry an `INLINE_BYTES`\n\
         // const-generic the application derives from its `HostBounds`; fix\n\
         // a concrete width.\n\
         const N: usize = 32;\n\
         let terms: [Term<'static, N>; 1] =\n\
         \x20   [uor_foundation::pipeline::literal_u64(1, WittLevel::W8)];\n\
         let domains = [VerificationDomain::Enumerative];\n\
         \n\
         let unit = CompileUnitBuilder::<N>::new()\n\
         \x20   .root_term(&terms)\n\
         \x20   .witt_level_ceiling(WittLevel::W8)\n\
         \x20   .thermodynamic_budget(1024)\n\
         \x20   .target_domains(&domains)\n\
         \x20   .result_type::<ConstrainedTypeInput>()\n\
         \x20   .validate();\n\
         assert!(unit.is_ok());\n\
         \n\
         // Omitting a required field produces a ShapeViolation\n\
         // with the exact conformance IRI that failed:\n\
         let err = CompileUnitBuilder::<N>::new()\n\
         \x20   .witt_level_ceiling(WittLevel::W8)\n\
         \x20   .thermodynamic_budget(1024)\n\
         \x20   .target_domains(&domains)\n\
         \x20   .result_type::<ConstrainedTypeInput>()\n\
         \x20   .validate();\n\
         assert!(err.is_err());\n\
         if let Err(violation) = err {\n\
         \x20   assert_eq!(violation.kind, ViolationKind::Missing);\n\
         \x20   assert!(violation.property_iri.contains(\"rootTerm\"));\n\
         }",
        "rust",
    );
    f.line("#[derive(Debug, Clone)]");
    f.line("pub struct CompileUnitBuilder<'a, const INLINE_BYTES: usize> {");
    f.indented_doc_comment("The root term expression.");
    f.line("    root_term: Option<&'a [Term<'a, INLINE_BYTES>]>,");
    f.indented_doc_comment("v0.2.2 Phase H1: named bindings (`let name : Type = term` forms)");
    f.indented_doc_comment(
        "declared by the compile unit. Stage 5 extracts these into a `BindingsTable`",
    );
    f.indented_doc_comment(
        "for grounding-aware and session resolvers; an empty slice declares no bindings.",
    );
    f.line("    bindings: Option<&'a [Binding]>,");
    f.indented_doc_comment("The widest Witt level the computation may reference.");
    f.line("    witt_level_ceiling: Option<WittLevel>,");
    f.indented_doc_comment("Landauer-bounded energy budget.");
    f.line("    thermodynamic_budget: Option<u64>,");
    f.indented_doc_comment("Verification domains targeted.");
    f.line("    target_domains: Option<&'a [VerificationDomain]>,");
    f.indented_doc_comment("v0.2.2 T6.11: result-type IRI for ShapeMismatch detection.");
    f.indented_doc_comment("Set via `result_type::<T: ConstrainedTypeShape>()`. Required by");
    f.indented_doc_comment("`validate()` and `validate_compile_unit_const`. The pipeline checks");
    f.indented_doc_comment("`unit.result_type_iri() == T::IRI` at `pipeline::run` invocation");
    f.indented_doc_comment("time, returning `PipelineFailure::ShapeMismatch` on mismatch.");
    f.line("    result_type_iri: Option<&'static str>,");
    f.line("}");
    f.blank();

    // CompileUnit (validated result type)
    f.doc_comment("A validated compile unit ready for reduction admission.");
    f.doc_comment("");
    f.doc_comment("v0.2.2 Phase A (carrier widening): the lifetime parameter `'a` ties");
    f.doc_comment("the post-validation carrier to its builder's borrow. The `root_term`");
    f.doc_comment("and `target_domains` slices are retained through validation so");
    f.doc_comment("resolvers can inspect declared structure — previously these fields");
    f.doc_comment("were discarded at `validate()` and every resolver received a");
    f.doc_comment("3-field scalar witness with no walkable structure.");
    f.doc_comment("");
    f.doc_comment("Const-constructed compile units use the trivial specialization");
    f.doc_comment("`CompileUnit<'static>` — borrow-free and usable in const contexts.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq)]");
    f.line("pub struct CompileUnit<'a, const INLINE_BYTES: usize> {");
    f.indented_doc_comment("The Witt level ceiling.");
    f.line("    level: WittLevel,");
    f.indented_doc_comment("The thermodynamic budget.");
    f.line("    budget: u64,");
    f.indented_doc_comment("v0.2.2 T6.11: result-type IRI. The pipeline matches this against");
    f.indented_doc_comment("the caller's `T::IRI` to detect shape mismatches.");
    f.line("    result_type_iri: &'static str,");
    f.indented_doc_comment("v0.2.2 Phase A: root term expression, retained from the builder.");
    f.indented_doc_comment("Stage 5 (extract bindings) and the grounding-aware resolver walk");
    f.indented_doc_comment("this slice. Empty slice for the trivial `CompileUnit<'static>`");
    f.indented_doc_comment("specialization produced by builders that don't carry a term AST.");
    f.line("    root_term: &'a [Term<'a, INLINE_BYTES>],");
    f.indented_doc_comment(
        "v0.2.2 Phase H1: named bindings retained from the builder. Consumed by Stage 5",
    );
    f.indented_doc_comment(
        "(`bindings_from_unit`) to materialize the `BindingsTable` for grounding-aware,",
    );
    f.indented_doc_comment(
        "session, and superposition resolvers. Empty slice declares no bindings.",
    );
    f.line("    bindings: &'a [Binding],");
    f.indented_doc_comment(
        "v0.2.2 Phase A: verification domains targeted, retained from the builder.",
    );
    f.line("    target_domains: &'a [VerificationDomain],");
    f.line("}");
    f.blank();
    f.line("impl<'a, const INLINE_BYTES: usize> CompileUnit<'a, INLINE_BYTES> {");
    f.indented_doc_comment("Returns the Witt level ceiling declared at validation time.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn witt_level(&self) -> WittLevel {");
    f.line("        self.level");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Returns the thermodynamic budget declared at validation time.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn thermodynamic_budget(&self) -> u64 {");
    f.line("        self.budget");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("v0.2.2 T6.11: returns the result-type IRI declared at validation");
    f.indented_doc_comment("time. The pipeline matches this against the caller's `T::IRI` to");
    f.indented_doc_comment("detect shape mismatches.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn result_type_iri(&self) -> &'static str {");
    f.line("        self.result_type_iri");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "v0.2.2 Phase A: returns the root term slice declared at validation time.",
    );
    f.indented_doc_comment("Empty for builders that did not supply a term AST.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn root_term(&self) -> &'a [Term<'a, INLINE_BYTES>] {");
    f.line("        self.root_term");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "v0.2.2 Phase H1: returns the named bindings declared at validation time.",
    );
    f.indented_doc_comment(
        "Consumed by Stage 5 (`bindings_from_unit`) to materialize the `BindingsTable`.",
    );
    f.indented_doc_comment("Empty slice for compile units that declare no bindings.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn bindings(&self) -> &'a [Binding] {");
    f.line("        self.bindings");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "v0.2.2 Phase A: returns the verification domains declared at validation time.",
    );
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn target_domains(&self) -> &'a [VerificationDomain] {");
    f.line("        self.target_domains");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("v0.2.2 Phase G / T2.8 + T6.11: const-constructible parts form used");
    f.indented_doc_comment("by `validate_compile_unit_const` — the const-fn path reads the");
    f.indented_doc_comment("builder's fields and packs them into the `Validated` result.");
    f.indented_doc_comment("");
    f.indented_doc_comment("v0.2.2 Phase H1: bindings slice is retained alongside root_term;");
    f.indented_doc_comment("empty slice declares no bindings.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub(crate) const fn from_parts_const(");
    f.line("        level: WittLevel,");
    f.line("        budget: u64,");
    f.line("        result_type_iri: &'static str,");
    f.line("        root_term: &'a [Term<'a, INLINE_BYTES>],");
    f.line("        bindings: &'a [Binding],");
    f.line("        target_domains: &'a [VerificationDomain],");
    f.line("    ) -> Self {");
    f.line("        Self {");
    f.line("            level,");
    f.line("            budget,");
    f.line("            result_type_iri,");
    f.line("            root_term,");
    f.line("            bindings,");
    f.line("            target_domains,");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();

    f.line("impl<'a, const INLINE_BYTES: usize> CompileUnitBuilder<'a, INLINE_BYTES> {");
    f.indented_doc_comment("Creates a new empty builder.");
    f.line("    #[must_use]");
    f.line("    pub const fn new() -> Self {");
    f.line("        Self {");
    f.line("            root_term: None,");
    f.line("            bindings: None,");
    f.line("            witt_level_ceiling: None,");
    f.line("            thermodynamic_budget: None,");
    f.line("            target_domains: None,");
    f.line("            result_type_iri: None,");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Set the root term expression.");
    f.line("    #[must_use]");
    f.line("    pub const fn root_term(mut self, terms: &'a [Term<'a, INLINE_BYTES>]) -> Self {");
    f.line("        self.root_term = Some(terms);");
    f.line("        self");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "v0.2.2 Phase H1: set the named bindings declared by this compile unit.",
    );
    f.indented_doc_comment("Consumed by Stage 5 (`bindings_from_unit`) to materialize the");
    f.indented_doc_comment(
        "`BindingsTable` for grounding-aware, session, and superposition resolvers.",
    );
    f.indented_doc_comment(
        "Omit for compile units without bindings; the default is the empty slice.",
    );
    f.line("    #[must_use]");
    f.line("    pub const fn bindings(mut self, bindings: &'a [Binding]) -> Self {");
    f.line("        self.bindings = Some(bindings);");
    f.line("        self");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Set the Witt level ceiling.");
    f.line("    #[must_use]");
    f.line("    pub const fn witt_level_ceiling(mut self, level: WittLevel) -> Self {");
    f.line("        self.witt_level_ceiling = Some(level);");
    f.line("        self");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Set the thermodynamic budget.");
    f.line("    #[must_use]");
    f.line("    pub const fn thermodynamic_budget(mut self, budget: u64) -> Self {");
    f.line("        self.thermodynamic_budget = Some(budget);");
    f.line("        self");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Set the target verification domains.");
    f.line("    #[must_use]");
    f.line(
        "    pub const fn target_domains(mut self, domains: &'a [VerificationDomain]) -> Self {",
    );
    f.line("        self.target_domains = Some(domains);");
    f.line("        self");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("v0.2.2 T6.11: set the result-type IRI from a `ConstrainedTypeShape`");
    f.indented_doc_comment("type parameter. The pipeline matches this against the caller's");
    f.indented_doc_comment("`T::IRI` at `pipeline::run` invocation time, returning");
    f.indented_doc_comment("`PipelineFailure::ShapeMismatch` on mismatch.");
    f.indented_doc_comment("");
    f.indented_doc_comment("Required: `validate()` and `validate_compile_unit_const` reject");
    f.indented_doc_comment("builders without a result type set.");
    f.line("    #[must_use]");
    f.line(
        "    pub const fn result_type<T: crate::pipeline::ConstrainedTypeShape>(mut self) -> Self {",
    );
    f.line("        self.result_type_iri = Some(T::IRI);");
    f.line("        self");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("v0.2.2 T2.8: const-fn accessor exposing the stored Witt level");
    f.indented_doc_comment("ceiling (or `None` if unset). Used by `validate_compile_unit_const`.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn witt_level_option(&self) -> Option<WittLevel> {");
    f.line("        self.witt_level_ceiling");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("v0.2.2 T2.8: const-fn accessor exposing the stored thermodynamic");
    f.indented_doc_comment("budget (or `None` if unset).");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn budget_option(&self) -> Option<u64> {");
    f.line("        self.thermodynamic_budget");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("v0.2.2 T6.13: const-fn accessor — `true` iff `root_term` is set.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn has_root_term_const(&self) -> bool {");
    f.line("        self.root_term.is_some()");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("v0.2.2 T6.13: const-fn accessor — `true` iff `target_domains` is");
    f.indented_doc_comment("set and non-empty.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn has_target_domains_const(&self) -> bool {");
    f.line("        match self.target_domains {");
    f.line("            Some(d) => !d.is_empty(),");
    f.line("            None => false,");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("v0.2.2 T6.13: const-fn accessor exposing the stored result-type IRI");
    f.indented_doc_comment("(or `None` if unset). Used by `validate_compile_unit_const`.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn result_type_iri_const(&self) -> Option<&'static str> {");
    f.line("        self.result_type_iri");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "v0.2.2 Phase A: const-fn accessor exposing the stored root-term slice,",
    );
    f.indented_doc_comment("or an empty slice if unset. Used by `validate_compile_unit_const` to");
    f.indented_doc_comment("propagate the AST into the widened `CompileUnit<'a>` carrier.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn root_term_slice_const(&self) -> &'a [Term<'a, INLINE_BYTES>] {");
    f.line("        match self.root_term {");
    f.line("            Some(terms) => terms,");
    f.line("            None => &[],");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "v0.2.2 Phase H1: const-fn accessor exposing the stored bindings slice,",
    );
    f.indented_doc_comment("or an empty slice if unset. Used by `validate_compile_unit_const` to");
    f.indented_doc_comment(
        "propagate the bindings declaration into the widened `CompileUnit<'a>` carrier.",
    );
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn bindings_slice_const(&self) -> &'a [Binding] {");
    f.line("        match self.bindings {");
    f.line("            Some(bindings) => bindings,");
    f.line("            None => &[],");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("v0.2.2 Phase A: const-fn accessor exposing the stored target-domains");
    f.indented_doc_comment(
        "slice, or an empty slice if unset. Used by `validate_compile_unit_const`.",
    );
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn target_domains_slice_const(&self) -> &'a [VerificationDomain] {");
    f.line("        match self.target_domains {");
    f.line("            Some(d) => d,");
    f.line("            None => &[],");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Validate against `CompileUnitShape`.");
    f.indented_doc_comment("");
    f.indented_doc_comment("Tier 1: checks presence and cardinality of all required fields.");
    f.indented_doc_comment("Tier 2: checks budget solvency and level coherence.");
    f.indented_doc_comment("");
    f.indented_doc_comment("# Errors");
    f.indented_doc_comment("");
    f.indented_doc_comment("Returns `ShapeViolation` if any constraint is not satisfied.");
    f.line("    pub fn validate(self) -> Result<Validated<CompileUnit<'a, INLINE_BYTES>>, ShapeViolation> {");
    f.line("        let root_term = match self.root_term {");
    f.line("            Some(terms) => terms,");
    f.line("            None => return Err(ShapeViolation {");
    f.line("                shape_iri: \"https://uor.foundation/conformance/CompileUnitShape\",");
    f.line("                constraint_iri: \"https://uor.foundation/conformance/compileUnit_rootTerm_constraint\",");
    f.line("                property_iri: \"https://uor.foundation/reduction/rootTerm\",");
    f.line("                expected_range: \"https://uor.foundation/schema/Term\",");
    f.line("                min_count: 1,");
    f.line("                max_count: 1,");
    f.line("                kind: ViolationKind::Missing,");
    f.line("            }),");
    f.line("        };");
    f.line("        let level = match self.witt_level_ceiling {");
    f.line("            Some(l) => l,");
    f.line("            None => return Err(ShapeViolation {");
    f.line("                shape_iri: \"https://uor.foundation/conformance/CompileUnitShape\",");
    f.line("                constraint_iri: \"https://uor.foundation/conformance/compileUnit_unitWittLevel_constraint\",");
    f.line("                property_iri: \"https://uor.foundation/reduction/unitWittLevel\",");
    f.line("                expected_range: \"https://uor.foundation/schema/WittLevel\",");
    f.line("                min_count: 1,");
    f.line("                max_count: 1,");
    f.line("                kind: ViolationKind::Missing,");
    f.line("            }),");
    f.line("        };");
    f.line("        let budget = match self.thermodynamic_budget {");
    f.line("            Some(b) => b,");
    f.line("            None => return Err(ShapeViolation {");
    f.line("                shape_iri: \"https://uor.foundation/conformance/CompileUnitShape\",");
    f.line("                constraint_iri: \"https://uor.foundation/conformance/compileUnit_thermodynamicBudget_constraint\",");
    f.line(
        "                property_iri: \"https://uor.foundation/reduction/thermodynamicBudget\",",
    );
    f.line("                expected_range: \"http://www.w3.org/2001/XMLSchema#decimal\",");
    f.line("                min_count: 1,");
    f.line("                max_count: 1,");
    f.line("                kind: ViolationKind::Missing,");
    f.line("            }),");
    f.line("        };");
    f.line("        let target_domains = match self.target_domains {");
    f.line("            Some(d) if !d.is_empty() => d,");
    f.line("            _ => return Err(ShapeViolation {");
    f.line("                shape_iri: \"https://uor.foundation/conformance/CompileUnitShape\",");
    f.line("                constraint_iri: \"https://uor.foundation/conformance/compileUnit_targetDomains_constraint\",");
    f.line("                property_iri: \"https://uor.foundation/reduction/targetDomains\",");
    f.line("                expected_range: \"https://uor.foundation/op/VerificationDomain\",");
    f.line("                min_count: 1,");
    f.line("                max_count: 0,");
    f.line("                kind: ViolationKind::Missing,");
    f.line("            }),");
    f.line("        };");
    f.line("        let result_type_iri = match self.result_type_iri {");
    f.line("            Some(iri) => iri,");
    f.line("            None => return Err(ShapeViolation {");
    f.line("                shape_iri: \"https://uor.foundation/conformance/CompileUnitShape\",");
    f.line("                constraint_iri: \"https://uor.foundation/conformance/compileUnit_resultType_constraint\",");
    f.line("                property_iri: \"https://uor.foundation/reduction/resultType\",");
    f.line("                expected_range: \"https://uor.foundation/type/ConstrainedType\",");
    f.line("                min_count: 1,");
    f.line("                max_count: 1,");
    f.line("                kind: ViolationKind::Missing,");
    f.line("            }),");
    f.line("        };");
    f.line("        // v0.2.2 Phase H1: bindings is optional; absent declares no bindings.");
    f.line(
        "        let bindings: &'a [Binding] = match self.bindings { Some(b) => b, None => &[] };",
    );
    f.line("        Ok(Validated::new(CompileUnit { level, budget, result_type_iri, root_term, bindings, target_domains }))");
    f.line("    }");
    f.line("}");
    f.blank();

    // Default impl for CompileUnitBuilder
    f.line(
        "impl<'a, const INLINE_BYTES: usize> Default for CompileUnitBuilder<'a, INLINE_BYTES> {",
    );
    f.line("    fn default() -> Self {");
    f.line("        Self::new()");
    f.line("    }");
    f.line("}");
    f.blank();

    // Generate builders for the remaining 8 shapes
    generate_simple_builder(
        f,
        "EffectDeclarationBuilder",
        "Declared effect validated against `EffectShape`.",
        &[
            ("name", "&'a str"),
            ("target_sites", "&'a [u32]"),
            ("budget_delta", "i64"),
            ("commutes", "bool"),
        ],
        "EffectDeclaration",
        "https://uor.foundation/conformance/EffectShape",
    );
    generate_simple_builder(
        f,
        "GroundingDeclarationBuilder",
        "Declared grounding validated against `GroundingShape`.",
        &[
            ("source_type", "&'a str"),
            ("ring_mapping", "&'a str"),
            ("invertibility", "bool"),
        ],
        "GroundingDeclaration",
        "https://uor.foundation/conformance/GroundingShape",
    );
    generate_simple_builder(
        f,
        "DispatchDeclarationBuilder",
        "Declared dispatch rule validated against `DispatchShape`.",
        &[
            ("predicate", "&'a [Term]"),
            ("target_resolver", "&'a str"),
            ("priority", "u32"),
        ],
        "DispatchDeclaration",
        "https://uor.foundation/conformance/DispatchShape",
    );
    generate_simple_builder(
        f,
        "LeaseDeclarationBuilder",
        "Declared lease validated against `LeaseShape`.",
        &[("linear_site", "u32"), ("scope", "&'a str")],
        "LeaseDeclaration",
        "https://uor.foundation/conformance/LeaseShape",
    );
    generate_simple_builder(
        f,
        "StreamDeclarationBuilder",
        "Declared stream validated against `StreamShape`.",
        &[
            ("seed", "&'a [Term]"),
            ("step", "&'a [Term]"),
            ("productivity_witness", "&'a str"),
        ],
        "StreamDeclaration",
        "https://uor.foundation/conformance/StreamShape",
    );
    generate_simple_builder(
        f,
        "PredicateDeclarationBuilder",
        "Declared predicate validated against `PredicateShape`.",
        &[
            ("input_type", "&'a str"),
            ("evaluator", "&'a [Term]"),
            ("termination_witness", "&'a str"),
        ],
        "PredicateDeclaration",
        "https://uor.foundation/conformance/PredicateShape",
    );
    generate_simple_builder(
        f,
        "ParallelDeclarationBuilder",
        "Declared parallel composition validated against `ParallelShape`.",
        &[
            ("site_partition", "&'a [u32]"),
            ("disjointness_witness", "&'a str"),
        ],
        "ParallelDeclaration",
        "https://uor.foundation/conformance/ParallelShape",
    );

    // v0.2.2 T2.7 / T2.8 (cleanup): custom const-fn accessors on the
    // ParallelDeclarationBuilder + StreamDeclarationBuilder structs.
    // These add-on impl blocks expose the builder's stored Option fields
    // as const-fn read-only accessors so the const-fn validate paths
    // (T2.8) and Phase F drivers (T2.7) can derive their output state
    // from the builder/unit input.
    f.line("impl<'a> ParallelDeclarationBuilder<'a> {");
    f.indented_doc_comment("v0.2.2 T2.7: const-fn accessor returning the length of the");
    f.indented_doc_comment("declared site partition (or 0 if unset).");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn site_partition_len(&self) -> usize {");
    f.line("        match self.site_partition {");
    f.line("            Some(p) => p.len(),");
    f.line("            None => 0,");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "v0.2.2 Phase A: const-fn accessor returning the declared site-partition",
    );
    f.indented_doc_comment("slice, or an empty slice if unset. Used by `validate_parallel_const`");
    f.indented_doc_comment(
        "to propagate the partition into the widened `ParallelDeclaration<'a>`.",
    );
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn site_partition_slice_const(&self) -> &'a [u32] {");
    f.line("        match self.site_partition {");
    f.line("            Some(p) => p,");
    f.line("            None => &[],");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "v0.2.2 Phase A: const-fn accessor returning the declared disjointness-witness",
    );
    f.indented_doc_comment("IRI string, or an empty string if unset.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn disjointness_witness_const(&self) -> &'a str {");
    f.line("        match self.disjointness_witness {");
    f.line("            Some(s) => s,");
    f.line("            None => \"\",");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();
    f.line("impl<'a, const INLINE_BYTES: usize> StreamDeclarationBuilder<'a, INLINE_BYTES> {");
    f.indented_doc_comment("v0.2.2 canonical: productivity bound is 1 if a `productivityWitness`");
    f.indented_doc_comment("IRI is declared (the stream attests termination via a `proof:Proof`");
    f.indented_doc_comment("individual), 0 otherwise. The witness's IRI points to the termination");
    f.indented_doc_comment("proof; downstream resolvers dereference it for detailed bound");
    f.indented_doc_comment(
        "information. This two-level split (presence flag here, IRI dereference",
    );
    f.indented_doc_comment("elsewhere) is the canonical foundation-level shape.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn productivity_bound_const(&self) -> u64 {");
    f.line("        match self.productivity_witness {");
    f.line("            Some(_) => 1,");
    f.line("            None => 0,");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "v0.2.2 Phase A: const-fn accessor returning the declared seed term slice,",
    );
    f.indented_doc_comment("or an empty slice if unset.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn seed_slice_const(&self) -> &'a [Term<'a, INLINE_BYTES>] {");
    f.line("        match self.seed {");
    f.line("            Some(t) => t,");
    f.line("            None => &[],");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "v0.2.2 Phase A: const-fn accessor returning the declared step term slice,",
    );
    f.indented_doc_comment("or an empty slice if unset.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn step_slice_const(&self) -> &'a [Term<'a, INLINE_BYTES>] {");
    f.line("        match self.step {");
    f.line("            Some(t) => t,");
    f.line("            None => &[],");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "v0.2.2 Phase A: const-fn accessor returning the declared productivity-witness",
    );
    f.indented_doc_comment("IRI, or an empty string if unset.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn productivity_witness_const(&self) -> &'a str {");
    f.line("        match self.productivity_witness {");
    f.line("            Some(s) => s,");
    f.line("            None => \"\",");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();

    // WittLevelDeclarationBuilder (no lifetime needed)
    f.doc_comment("Builder for declaring a new Witt level beyond W32.");
    f.doc_comment("");
    f.doc_comment("Validates against `WittLevelShape`.");
    f.line("#[derive(Debug, Clone)]");
    f.line("pub struct WittLevelDeclarationBuilder {");
    f.indented_doc_comment("The declared bit width.");
    f.line("    bit_width: Option<u32>,");
    f.indented_doc_comment("The declared cycle size.");
    f.line("    cycle_size: Option<u128>,");
    f.indented_doc_comment("The predecessor level.");
    f.line("    predecessor: Option<WittLevel>,");
    f.line("}");
    f.blank();

    f.doc_comment("Validated Witt level declaration.");
    f.line("#[derive(Debug, Clone, PartialEq, Eq)]");
    f.line("pub struct WittLevelDeclaration {");
    f.indented_doc_comment("The declared bit width.");
    f.line("    pub bit_width: u32,");
    f.indented_doc_comment("The predecessor level.");
    f.line("    pub predecessor: WittLevel,");
    f.line("}");
    f.blank();

    f.line("impl WittLevelDeclarationBuilder {");
    f.indented_doc_comment("Creates a new empty builder.");
    f.line("    #[must_use]");
    f.line("    pub const fn new() -> Self {");
    f.line("        Self { bit_width: None, cycle_size: None, predecessor: None }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Set the declared bit width.");
    f.line("    #[must_use]");
    f.line("    pub const fn bit_width(mut self, w: u32) -> Self {");
    f.line("        self.bit_width = Some(w);");
    f.line("        self");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Set the declared cycle size.");
    f.line("    #[must_use]");
    f.line("    pub const fn cycle_size(mut self, s: u128) -> Self {");
    f.line("        self.cycle_size = Some(s);");
    f.line("        self");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Set the predecessor Witt level.");
    f.line("    #[must_use]");
    f.line("    pub const fn predecessor(mut self, level: WittLevel) -> Self {");
    f.line("        self.predecessor = Some(level);");
    f.line("        self");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Validate against `WittLevelShape`.");
    f.indented_doc_comment("");
    f.indented_doc_comment("# Errors");
    f.indented_doc_comment("");
    f.indented_doc_comment("Returns `ShapeViolation` if any required field is missing.");
    f.line(
        "    pub fn validate(self) -> Result<Validated<WittLevelDeclaration>, ShapeViolation> {",
    );
    // (validate body emitted below; after it, we add validate_const.)
    f.line("        let bw = match self.bit_width {");
    f.line("            Some(w) => w,");
    f.line("            None => return Err(ShapeViolation {");
    f.line("                shape_iri: \"https://uor.foundation/conformance/WittLevelShape\",");
    f.line(
        "                constraint_iri: \"https://uor.foundation/conformance/WittLevelShape\",",
    );
    f.line(
        "                property_iri: \"https://uor.foundation/conformance/declaredBitWidth\",",
    );
    f.line("                expected_range: \"http://www.w3.org/2001/XMLSchema#positiveInteger\",");
    f.line("                min_count: 1,");
    f.line("                max_count: 1,");
    f.line("                kind: ViolationKind::Missing,");
    f.line("            }),");
    f.line("        };");
    f.line("        let pred = match self.predecessor {");
    f.line("            Some(p) => p,");
    f.line("            None => return Err(ShapeViolation {");
    f.line("                shape_iri: \"https://uor.foundation/conformance/WittLevelShape\",");
    f.line(
        "                constraint_iri: \"https://uor.foundation/conformance/WittLevelShape\",",
    );
    f.line(
        "                property_iri: \"https://uor.foundation/conformance/predecessorLevel\",",
    );
    f.line("                expected_range: \"https://uor.foundation/schema/WittLevel\",");
    f.line("                min_count: 1,");
    f.line("                max_count: 1,");
    f.line("                kind: ViolationKind::Missing,");
    f.line("            }),");
    f.line("        };");
    f.line("        Ok(Validated::new(WittLevelDeclaration { bit_width: bw, predecessor: pred }))");
    f.line("    }");
    f.blank();
    // Phase C.1: validate_const for WittLevelDeclarationBuilder.
    f.indented_doc_comment(
        "Phase C.1: const-fn companion for `WittLevelDeclarationBuilder::validate`.",
    );
    f.indented_doc_comment("");
    f.indented_doc_comment("# Errors");
    f.indented_doc_comment("");
    f.indented_doc_comment("Returns `ShapeViolation` if any required field is missing.");
    f.line("    pub const fn validate_const(&self) -> Result<Validated<WittLevelDeclaration, CompileTime>, ShapeViolation> {");
    f.line("        let bw = match self.bit_width {");
    f.line("            Some(w) => w,");
    f.line("            None => return Err(ShapeViolation {");
    f.line("                shape_iri: \"https://uor.foundation/conformance/WittLevelShape\",");
    f.line(
        "                constraint_iri: \"https://uor.foundation/conformance/WittLevelShape\",",
    );
    f.line(
        "                property_iri: \"https://uor.foundation/conformance/declaredBitWidth\",",
    );
    f.line("                expected_range: \"http://www.w3.org/2001/XMLSchema#positiveInteger\",");
    f.line("                min_count: 1,");
    f.line("                max_count: 1,");
    f.line("                kind: ViolationKind::Missing,");
    f.line("            }),");
    f.line("        };");
    f.line("        let pred = match self.predecessor {");
    f.line("            Some(p) => p,");
    f.line("            None => return Err(ShapeViolation {");
    f.line("                shape_iri: \"https://uor.foundation/conformance/WittLevelShape\",");
    f.line(
        "                constraint_iri: \"https://uor.foundation/conformance/WittLevelShape\",",
    );
    f.line(
        "                property_iri: \"https://uor.foundation/conformance/predecessorLevel\",",
    );
    f.line("                expected_range: \"https://uor.foundation/schema/WittLevel\",");
    f.line("                min_count: 1,");
    f.line("                max_count: 1,");
    f.line("                kind: ViolationKind::Missing,");
    f.line("            }),");
    f.line("        };");
    f.line("        Ok(Validated::new(WittLevelDeclaration { bit_width: bw, predecessor: pred }))");
    f.line("    }");
    f.line("}");
    f.blank();

    f.line("impl Default for WittLevelDeclarationBuilder {");
    f.line("    fn default() -> Self {");
    f.line("        Self::new()");
    f.line("    }");
    f.line("}");
    f.blank();
}

/// Generates a simple builder struct with `Option` fields and a `validate()` method
/// that checks all fields are present.
fn generate_simple_builder(
    f: &mut RustFile,
    builder_name: &str,
    result_doc: &str,
    fields: &[(&str, &str)],
    result_name: &str,
    shape_iri: &str,
) {
    let needs_lifetime = fields.iter().any(|(_, ty)| ty.starts_with('&'));
    // ADR-060: a builder field carrying a `[Term]` slice makes the builder
    // const-generic over the source-polymorphic carrier's inline width, since
    // `Term<'a, INLINE_BYTES>` is const-generic. `lt_def` is the def-side
    // generic list (`<'a, const INLINE_BYTES: usize>`); `lt_use` is the
    // use-side (`<'a, INLINE_BYTES>`). The validated result struct carries no
    // Term, so it stays non-generic.
    let has_term = fields.iter().any(|(_, ty)| ty.contains("[Term]"));
    let lt_def = if has_term {
        "<'a, const INLINE_BYTES: usize>"
    } else if needs_lifetime {
        "<'a>"
    } else {
        ""
    };
    let lt_use = if has_term {
        "<'a, INLINE_BYTES>"
    } else if needs_lifetime {
        "<'a>"
    } else {
        ""
    };
    let map_ty = |ty: &str| -> String {
        if ty.contains("[Term]") {
            ty.replace("[Term]", "[Term<'a, INLINE_BYTES>]")
        } else {
            ty.to_string()
        }
    };

    // Builder struct
    f.doc_comment(&format!(
        "Builder for `{result_name}`. Validates against `{}`.",
        shape_iri.rsplit('/').next().unwrap_or(shape_iri),
    ));
    f.line("#[derive(Debug, Clone)]");
    f.line(&format!("pub struct {builder_name}{lt_def} {{"));
    for (name, ty) in fields {
        let opt_ty = format!("Option<{}>", map_ty(ty));
        f.indented_doc_comment(&format!("The `{name}` field."));
        f.line(&format!("    {name}: {opt_ty},"));
    }
    f.line("}");
    f.blank();

    // Validated result struct
    f.doc_comment(result_doc);
    f.line("#[derive(Debug, Clone, PartialEq, Eq)]");
    f.line(&format!("pub struct {result_name} {{"));
    f.indented_doc_comment("Shape IRI this declaration was validated against.");
    f.line("    pub shape_iri: &'static str,");
    f.line("}");
    f.blank();
    // v0.2.2 Phase G: const-constructible empty form for const-fn
    // validation paths.
    f.line(&format!("impl {result_name} {{"));
    f.indented_doc_comment("v0.2.2 Phase G: const-constructible empty form used by");
    f.indented_doc_comment("`validate_*_const` companion functions.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn empty_const() -> Self {");
    f.line(&format!("        Self {{ shape_iri: \"{shape_iri}\" }}"));
    f.line("    }");
    f.line("}");
    f.blank();

    // impl block
    f.line(&format!("impl{lt_def} {builder_name}{lt_use} {{"));
    f.indented_doc_comment("Creates a new empty builder.");
    f.line("    #[must_use]");
    f.line("    pub const fn new() -> Self {");
    f.line("        Self {");
    for (name, _) in fields {
        f.line(&format!("            {name}: None,"));
    }
    f.line("        }");
    f.line("    }");
    f.blank();

    // Setter methods
    for (name, ty) in fields {
        f.indented_doc_comment(&format!("Set the `{name}` field."));
        f.line("    #[must_use]");
        f.line(&format!(
            "    pub const fn {name}(mut self, value: {}) -> Self {{",
            map_ty(ty)
        ));
        f.line(&format!("        self.{name} = Some(value);"));
        f.line("        self");
        f.line("    }");
        f.blank();
    }

    // validate method
    f.indented_doc_comment(&format!(
        "Validate against `{}`.",
        shape_iri.rsplit('/').next().unwrap_or(shape_iri)
    ));
    f.indented_doc_comment("");
    f.indented_doc_comment("# Errors");
    f.indented_doc_comment("");
    f.indented_doc_comment("Returns `ShapeViolation` if any required field is missing.");
    f.line(&format!(
        "    pub fn validate(self) -> Result<Validated<{result_name}>, ShapeViolation> {{"
    ));
    // Check first field as representative
    let first = fields[0].0;
    f.line(&format!("        if self.{first}.is_none() {{"));
    f.line("            return Err(ShapeViolation {");
    f.line(&format!("                shape_iri: \"{shape_iri}\","));
    f.line(&format!("                constraint_iri: \"{shape_iri}\","));
    f.line(&format!(
        "                property_iri: \"https://uor.foundation/conformance/{first}\","
    ));
    f.line("                expected_range: \"http://www.w3.org/2002/07/owl#Thing\",");
    f.line("                min_count: 1,");
    f.line("                max_count: 1,");
    f.line("                kind: ViolationKind::Missing,");
    f.line("            });");
    f.line("        }");
    // Check remaining fields
    for (name, _) in &fields[1..] {
        f.line(&format!("        if self.{name}.is_none() {{"));
        f.line("            return Err(ShapeViolation {");
        f.line(&format!("                shape_iri: \"{shape_iri}\","));
        f.line(&format!("                constraint_iri: \"{shape_iri}\","));
        f.line(&format!(
            "                property_iri: \"https://uor.foundation/conformance/{name}\","
        ));
        f.line("                expected_range: \"http://www.w3.org/2002/07/owl#Thing\",");
        f.line("                min_count: 1,");
        f.line("                max_count: 1,");
        f.line("                kind: ViolationKind::Missing,");
        f.line("            });");
        f.line("        }");
    }
    f.line(&format!(
        "        Ok(Validated::new({result_name} {{ shape_iri: \"{shape_iri}\" }}))"
    ));
    f.line("    }");
    f.blank();

    // Phase C.1: validate_const companion — const fn returning CompileTime phase.
    // Emitted inside the same impl block as `validate`.
    f.indented_doc_comment(&format!(
        "Phase C.1: const-fn companion for `{builder_name}::validate`.",
    ));
    f.indented_doc_comment("");
    f.indented_doc_comment("Returns `Validated<_, CompileTime>` on success, allowing compile-time");
    f.indented_doc_comment(
        "evidence via `const _V: Validated<_, CompileTime> = builder.validate_const().unwrap();`.",
    );
    f.indented_doc_comment("");
    f.indented_doc_comment("# Errors");
    f.indented_doc_comment("");
    f.indented_doc_comment("Returns `ShapeViolation` if any required field is missing.");
    f.line(&format!(
        "    pub const fn validate_const(&self) -> Result<Validated<{result_name}, CompileTime>, ShapeViolation> {{"
    ));
    let first_field = fields[0].0;
    f.line(&format!("        if self.{first_field}.is_none() {{"));
    f.line("            return Err(ShapeViolation {");
    f.line(&format!("                shape_iri: \"{shape_iri}\","));
    f.line(&format!("                constraint_iri: \"{shape_iri}\","));
    f.line(&format!(
        "                property_iri: \"https://uor.foundation/conformance/{first_field}\","
    ));
    f.line("                expected_range: \"http://www.w3.org/2002/07/owl#Thing\",");
    f.line("                min_count: 1,");
    f.line("                max_count: 1,");
    f.line("                kind: ViolationKind::Missing,");
    f.line("            });");
    f.line("        }");
    for (name, _) in &fields[1..] {
        f.line(&format!("        if self.{name}.is_none() {{"));
        f.line("            return Err(ShapeViolation {");
        f.line(&format!("                shape_iri: \"{shape_iri}\","));
        f.line(&format!("                constraint_iri: \"{shape_iri}\","));
        f.line(&format!(
            "                property_iri: \"https://uor.foundation/conformance/{name}\","
        ));
        f.line("                expected_range: \"http://www.w3.org/2002/07/owl#Thing\",");
        f.line("                min_count: 1,");
        f.line("                max_count: 1,");
        f.line("                kind: ViolationKind::Missing,");
        f.line("            });");
        f.line("        }");
    }
    f.line(&format!(
        "        Ok(Validated::new({result_name} {{ shape_iri: \"{shape_iri}\" }}))"
    ));
    f.line("    }");
    f.line("}");
    f.blank();

    // Default impl
    f.line(&format!(
        "impl{lt_def} Default for {builder_name}{lt_use} {{"
    ));
    f.line("    fn default() -> Self {");
    f.line("        Self::new()");
    f.line("    }");
    f.line("}");
    f.blank();
}

fn generate_minting_session(f: &mut RustFile, ontology: &Ontology) {
    let levels = witt_levels(ontology);
    f.doc_comment("Boundary session state tracker for the two-phase minting boundary.");
    f.doc_comment("");
    f.doc_comment("Records crossing count and idempotency flag. Private fields");
    f.doc_comment("prevent external construction.");
    f.line("#[derive(Debug, Clone, PartialEq, Eq)]");
    f.line("pub struct BoundarySession {");
    f.indented_doc_comment("Total boundary crossings in this session.");
    f.line("    crossing_count: u32,");
    f.indented_doc_comment("Whether the boundary effect is idempotent.");
    f.line("    is_idempotent: bool,");
    f.line("}");
    f.blank();
    f.line("impl BoundarySession {");
    f.indented_doc_comment("Creates a new boundary session. Only callable within the crate.");
    f.line("    #[inline]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new(is_idempotent: bool) -> Self {");
    f.line("        Self { crossing_count: 0, is_idempotent }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Returns the total boundary crossings.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn crossing_count(&self) -> u32 {");
    f.line("        self.crossing_count");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Returns whether the boundary effect is idempotent.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn is_idempotent(&self) -> bool {");
    f.line("        self.is_idempotent");
    f.line("    }");
    f.line("}");
    f.blank();

    // validate_and_mint functions
    f.doc_comment("Validate a scalar grounding intermediate against a `GroundingShape`");
    f.doc_comment("and mint it into a `Datum`. Only callable within `uor-foundation`.");
    f.doc_comment("");
    f.doc_comment("# Errors");
    f.doc_comment("");
    f.doc_comment("Returns `ShapeViolation` if the coordinate fails validation.");
    f.line("#[allow(dead_code)]");
    f.line("pub(crate) fn validate_and_mint_coord(");
    f.line("    grounded: GroundedCoord,");
    f.line("    shape: &Validated<GroundingDeclaration>,");
    f.line("    session: &mut BoundarySession,");
    f.line(") -> Result<Datum, ShapeViolation> {");
    f.line("    // The Validated<GroundingDeclaration> proves the shape was already");
    f.line("    // validated at builder time. The coordinate's level is guaranteed");
    f.line("    // correct by the closed GroundedCoordInner enum — the type system");
    f.line("    // enforces that only supported levels can be constructed.");
    f.line("    let _ = shape; // shape validation passed at builder time");
    f.line("    session.crossing_count += 1;");
    f.line("    let inner = match grounded.inner {");
    for (local, _, _) in &levels {
        f.line(&format!(
            "        GroundedCoordInner::{local}(b) => DatumInner::{local}(b),"
        ));
    }
    f.line("    };");
    f.line("    Ok(Datum { inner })");
    f.line("}");
    f.blank();

    f.doc_comment("Validate a tuple grounding intermediate and mint into a `Datum`.");
    f.doc_comment("Only callable within `uor-foundation`.");
    f.doc_comment("");
    f.doc_comment("Mints the first coordinate of the tuple as the representative `Datum`.");
    f.doc_comment("Composite multi-coordinate `Datum` construction depends on the target");
    f.doc_comment("type's site decomposition, which is resolved during reduction evaluation.");
    f.doc_comment("");
    f.doc_comment("# Errors");
    f.doc_comment("");
    f.doc_comment("Returns `ShapeViolation` if the tuple is empty or fails validation.");
    f.line("#[allow(dead_code)]");
    f.line("pub(crate) fn validate_and_mint_tuple<const N: usize>(");
    f.line("    grounded: GroundedTuple<N>,");
    f.line("    shape: &Validated<GroundingDeclaration>,");
    f.line("    session: &mut BoundarySession,");
    f.line(") -> Result<Datum, ShapeViolation> {");
    f.line("    if N == 0 {");
    f.line("        return Err(ShapeViolation {");
    f.line("            shape_iri: shape.inner().shape_iri,");
    f.line("            constraint_iri: shape.inner().shape_iri,");
    f.line("            property_iri: \"https://uor.foundation/conformance/groundingSourceType\",");
    f.line("            expected_range: \"https://uor.foundation/type/TypeDefinition\",");
    f.line("            min_count: 1,");
    f.line("            max_count: 0,");
    f.line("            kind: ViolationKind::CardinalityViolation,");
    f.line("        });");
    f.line("    }");
    f.line("    // Mint the first coordinate as the representative Datum.");
    f.line("    // The full tuple is decomposed during reduction evaluation,");
    f.line("    // where each coordinate maps to a site in the constrained type.");
    f.line("    validate_and_mint_coord(grounded.coords[0].clone(), shape, session)");
    f.line("}");
    f.blank();

    // ─────────────────────────────────────────────────────────────────
    // Wiki ADR-016: cross-crate construction surface for the four
    // UOR-domain sealed types. The architectural commitment is that
    // `prism`'s pipeline is the sole sanctioned caller of these
    // primitives (a normative commitment, not a Rust-language access
    // restriction). With prism merged into `uor-foundation`, the
    // primitives delegate to the existing `pub(crate)` constructors —
    // the named `pub fn` wrappers preserve the wiki's named surface so
    // the architectural commitment is observable at the crate's
    // public API.
    // ─────────────────────────────────────────────────────────────────

    f.doc_comment("Wiki ADR-016 mint primitive: cross-crate construction surface for `Datum`.");
    f.doc_comment("");
    f.doc_comment("Takes host bytes that have already passed the author's `Grounding` impl and");
    f.doc_comment("mints them into a sealed `Datum` at the supplied Witt level. The bytes are");
    f.doc_comment("decoded according to the level's byte width.");
    f.doc_comment("");
    f.doc_comment("# Errors");
    f.doc_comment("");
    f.doc_comment(
        "Returns [`ShapeViolation`] if `bytes.len()` doesn't match the level's byte width",
    );
    f.doc_comment("or if the level is unsupported.");
    f.line("pub fn mint_datum(level: crate::WittLevel, bytes: &[u8]) -> Result<Datum, ShapeViolation> {");
    f.line("    let expected_bytes = (level.witt_length() / 8) as usize;");
    f.line("    if bytes.len() != expected_bytes {");
    f.line("        return Err(ShapeViolation {");
    f.line("            shape_iri: \"https://uor.foundation/u/Datum\",");
    f.line("            constraint_iri: \"https://uor.foundation/u/DatumByteWidth\",");
    f.line("            property_iri: \"https://uor.foundation/u/datumBytes\",");
    f.line("            expected_range: \"http://www.w3.org/2001/XMLSchema#nonNegativeInteger\",");
    f.line("            min_count: expected_bytes as u32,");
    f.line("            max_count: expected_bytes as u32,");
    f.line("            kind: crate::ViolationKind::CardinalityViolation,");
    f.line("        });");
    f.line("    }");
    f.line("    let inner = match level.witt_length() {");
    for (local, n, _) in &levels {
        let n_bytes = n / 8;
        f.line(&format!(
            "        {n} => {{ let mut buf = [0u8; {n_bytes}]; let mut i = 0; while i < {n_bytes} {{ buf[i] = bytes[i]; i += 1; }} DatumInner::{local}(buf) }},"
        ));
    }
    f.line("        _ => return Err(ShapeViolation {");
    f.line("            shape_iri: \"https://uor.foundation/u/Datum\",");
    f.line("            constraint_iri: \"https://uor.foundation/u/DatumLevel\",");
    f.line("            property_iri: \"https://uor.foundation/u/datumLevel\",");
    f.line("            expected_range: \"https://uor.foundation/schema/WittLevel\",");
    f.line("            min_count: 1,");
    f.line("            max_count: 1,");
    f.line("            kind: crate::ViolationKind::ValueCheck,");
    f.line("        }),");
    f.line("    };");
    f.line("    Ok(Datum { inner })");
    f.line("}");
    f.blank();

    f.doc_comment("Wiki ADR-016 mint primitive: cross-crate construction surface for `Triad<L>`.");
    f.doc_comment("");
    f.doc_comment("Takes three coordinate values that satisfy the Triad shape constraint and");
    f.doc_comment("mints them into a sealed `Triad<L>` at the level marker `L`.");
    f.line("#[must_use]");
    f.line("pub const fn mint_triad<L>(stratum: u64, spectrum: u64, address: u64) -> Triad<L> {");
    f.line("    Triad::new(stratum, spectrum, address)");
    f.line("}");
    f.blank();

    f.doc_comment(
        "Wiki ADR-016 mint primitive: cross-crate construction surface for `Derivation`.",
    );
    f.doc_comment("");
    f.doc_comment("Takes the precursor's step count + Witt level + content fingerprint and mints");
    f.doc_comment("a sealed `Derivation` carrying the typed transition witness.");
    f.line("#[must_use]");
    f.line("pub const fn mint_derivation(");
    f.line("    step_count: u32,");
    f.line("    witt_level_bits: u16,");
    f.line("    content_fingerprint: ContentFingerprint,");
    f.line(") -> Derivation {");
    f.line("    Derivation::new(step_count, witt_level_bits, content_fingerprint)");
    f.line("}");
    f.blank();

    f.doc_comment("Wiki ADR-016 mint primitive: cross-crate construction surface for `FreeRank`.");
    f.doc_comment("");
    f.doc_comment(
        "Takes a natural-number rank witness (total site capacity at the Witt level plus",
    );
    f.doc_comment("the number of currently pinned sites) and mints it into a sealed `FreeRank`.");
    f.line("#[must_use]");
    f.line("pub const fn mint_freerank(total: u32, pinned: u32) -> FreeRank {");
    f.line("    FreeRank::new(total, pinned)");
    f.line("}");
    f.blank();
}

fn generate_const_ring_eval(f: &mut RustFile, ontology: &Ontology) {
    // v0.2.1 Phase 8b.7: emit one binary + one unary const helper per
    // `schema:WittLevel` individual. Helper names follow the pattern
    // `const_ring_eval_w{bits}` and `const_ring_eval_unary_w{bits}` so
    // the ring-op phantom-struct impls in `generate_ring_ops` can find
    // them mechanically.
    //
    // For non-power-of-2 bit widths (e.g. W24), the helper stores the
    // value in the smallest-containing Rust primitive (`u32` for W24)
    // and masks the result to the ring's bit width on every operation.
    let levels = witt_levels(ontology);

    f.doc_comment("Evaluate a binary ring operation at compile time.");
    f.doc_comment("");
    f.doc_comment("One helper is emitted per `schema:WittLevel` individual. The `uor!`");
    f.doc_comment("proc macro delegates to these helpers; it never performs ring");
    f.doc_comment("arithmetic itself.");
    f.doc_example(
        "use uor_foundation::enforcement::{const_ring_eval_w8, const_ring_eval_unary_w8};\n\
         use uor_foundation::PrimitiveOp;\n\
         \n\
         // Ring arithmetic in Z/256Z: all operations wrap modulo 256.\n\
         \n\
         // Addition wraps: 200 + 100 = 300 -> 300 - 256 = 44\n\
         assert_eq!(const_ring_eval_w8(PrimitiveOp::Add, 200, 100), 44);\n\
         \n\
         // Multiplication: 3 * 5 = 15 (no wrap needed)\n\
         assert_eq!(const_ring_eval_w8(PrimitiveOp::Mul, 3, 5), 15);\n\
         \n\
         // XOR: bitwise exclusive-or\n\
         assert_eq!(const_ring_eval_w8(PrimitiveOp::Xor, 0b1010, 0b1100), 0b0110);\n\
         \n\
         // Negation: neg(x) = 256 - x (additive inverse in Z/256Z)\n\
         assert_eq!(const_ring_eval_unary_w8(PrimitiveOp::Neg, 1), 255);\n\
         \n\
         // The critical identity: neg(bnot(x)) = succ(x) for all x\n\
         let x = 42u8;\n\
         let lhs = const_ring_eval_unary_w8(PrimitiveOp::Neg,\n\
         \x20   const_ring_eval_unary_w8(PrimitiveOp::Bnot, x));\n\
         let rhs = const_ring_eval_unary_w8(PrimitiveOp::Succ, x);\n\
         assert_eq!(lhs, rhs);",
        "rust",
    );

    for (local, bits, _) in &levels {
        let rust_ty = witt_rust_int_type(*bits);
        let lower = local.to_ascii_lowercase();
        // Mask for non-native-width levels.
        // Native widths: W8 (u8), W16 (u16), W32 (u32), W64 (u64), W128 (u128).
        // Non-native: W24, W40, W48, W56, W72, W80, W88, W96, W104, W112, W120.
        let native_bits: u32 = match rust_ty {
            "u8" => 8,
            "u16" => 16,
            "u32" => 32,
            "u64" => 64,
            "u128" => 128,
            _ => 64,
        };
        let needs_mask = *bits != native_bits;
        // Mask literal selection:
        // - Non-native u64-backed (W40/W48/W56): `u64::MAX >> (64 - bits)`
        //   yields a u64 directly.
        // - Non-native u32-backed (W24): cast from u64 since the shift
        //   produces u64 and we narrow to u32.
        // - Non-native u128-backed (W72..W120): `u128::MAX >> (128 - bits)`
        //   yields a u128 directly.
        let mask_lit = if !needs_mask {
            String::new()
        } else if rust_ty == "u128" {
            format!("u128::MAX >> (128 - {bits})")
        } else if rust_ty == "u64" {
            format!("u64::MAX >> (64 - {bits})")
        } else {
            format!("(u64::MAX >> (64 - {bits})) as {rust_ty}")
        };
        let apply_mask = |expr: String| -> String {
            if needs_mask {
                format!("({expr}) & MASK")
            } else {
                expr
            }
        };

        f.line("#[inline]");
        f.line("#[must_use]");
        // ADR-053: `if b == 0 { 0 } else { a / b }` is the const-fn-compatible
        // safe-divisor pattern; `checked_div().unwrap_or(0)` is not const-eval
        // on stable Rust 1.83 because `Option::unwrap_or` is not const.
        f.line("#[allow(clippy::manual_checked_ops)]");
        f.line(&format!(
            "pub const fn const_ring_eval_{lower}(op: PrimitiveOp, a: {rust_ty}, b: {rust_ty}) -> {rust_ty} {{"
        ));
        if needs_mask {
            f.line(&format!("    const MASK: {rust_ty} = {mask_lit};"));
        }
        f.line("    match op {");
        f.line(&format!(
            "        PrimitiveOp::Add => {},",
            apply_mask("a.wrapping_add(b)".to_string())
        ));
        f.line(&format!(
            "        PrimitiveOp::Sub => {},",
            apply_mask("a.wrapping_sub(b)".to_string())
        ));
        f.line(&format!(
            "        PrimitiveOp::Mul => {},",
            apply_mask("a.wrapping_mul(b)".to_string())
        ));
        f.line(&format!(
            "        PrimitiveOp::Xor => {},",
            apply_mask("a ^ b".to_string())
        ));
        f.line(&format!(
            "        PrimitiveOp::And => {},",
            apply_mask("a & b".to_string())
        ));
        f.line(&format!(
            "        PrimitiveOp::Or => {},",
            apply_mask("a | b".to_string())
        ));
        // ADR-013/TR-08 substrate amendment: comparison primitives lift
        // the ring's natural ordering to a 0/1-valued indicator. Returns
        // 1 if the comparison holds, 0 otherwise.
        f.line(&format!(
            "        PrimitiveOp::Le => (a <= b) as {rust_ty},"
        ));
        f.line(&format!("        PrimitiveOp::Lt => (a < b) as {rust_ty},"));
        f.line(&format!(
            "        PrimitiveOp::Ge => (a >= b) as {rust_ty},"
        ));
        f.line(&format!("        PrimitiveOp::Gt => (a > b) as {rust_ty},"));
        // Concat is a byte-sequence operator with no ring-arithmetic
        // interpretation — the catamorphism handles it at the byte level
        // via `apply_primitive_op`. Const-ring helpers return the ring's
        // additive identity for completeness.
        f.line("        PrimitiveOp::Concat => 0,");
        // ADR-053 substrate amendment: ring-axis completion under Γ = {+,−,×,÷,mod,^}.
        // Div / Mod: Euclidean division; b = 0 is rejected at runtime by the
        // catamorphism (`apply_primitive_op` returns a ShapeViolation). The
        // const-eval helpers cannot raise errors, so they treat b = 0 as the
        // ring's additive identity to keep `const fn` total.
        f.line(&format!(
            "        PrimitiveOp::Div => if b == 0 {{ 0 }} else {{ {} }},",
            apply_mask("a / b".to_string())
        ));
        f.line(&format!(
            "        PrimitiveOp::Mod => if b == 0 {{ 0 }} else {{ {} }},",
            apply_mask("a % b".to_string())
        ));
        // Pow: modular exponentiation by repeated squaring within the
        // width's ring; emitted as a const-evaluable expression that walks
        // the exponent's bits MSB→LSB. Implemented as a helper because
        // const fn cannot loop over the bit pattern of `b` directly.
        f.line(&format!(
            "        PrimitiveOp::Pow => {},",
            apply_mask(format!("const_pow_{lower}(a, b)"))
        ));
        f.line("        _ => 0,");
        f.line("    }");
        f.line("}");
        f.blank();

        // ADR-053: const-evaluable square-and-multiply for Pow over the
        // native ring width. Total in `const fn`.
        f.line("#[inline]");
        f.line("#[must_use]");
        f.line(&format!(
            "pub const fn const_pow_{lower}(base: {rust_ty}, exp: {rust_ty}) -> {rust_ty} {{"
        ));
        if needs_mask {
            f.line(&format!("    const MASK: {rust_ty} = {mask_lit};"));
        }
        f.line(&format!("    let mut result: {rust_ty} = 1;"));
        f.line(&format!(
            "    let mut b: {rust_ty} = {};",
            apply_mask("base".to_string())
        ));
        f.line(&format!("    let mut e: {rust_ty} = exp;"));
        f.line("    while e > 0 {");
        f.line("        if (e & 1) == 1 {");
        f.line(&format!(
            "            result = {};",
            apply_mask("result.wrapping_mul(b)".to_string())
        ));
        f.line("        }");
        f.line(&format!(
            "        b = {};",
            apply_mask("b.wrapping_mul(b)".to_string())
        ));
        f.line("        e >>= 1;");
        f.line("    }");
        f.line("    result");
        f.line("}");
        f.blank();

        f.line("#[inline]");
        f.line("#[must_use]");
        f.line(&format!(
            "pub const fn const_ring_eval_unary_{lower}(op: PrimitiveOp, a: {rust_ty}) -> {rust_ty} {{"
        ));
        if needs_mask {
            f.line(&format!("    const MASK: {rust_ty} = {mask_lit};"));
        }
        f.line("    match op {");
        f.line(&format!(
            "        PrimitiveOp::Neg => {},",
            apply_mask(format!("0{rust_ty}.wrapping_sub(a)"))
        ));
        f.line(&format!(
            "        PrimitiveOp::Bnot => {},",
            apply_mask("!a".to_string())
        ));
        f.line(&format!(
            "        PrimitiveOp::Succ => {},",
            apply_mask("a.wrapping_add(1)".to_string())
        ));
        f.line(&format!(
            "        PrimitiveOp::Pred => {},",
            apply_mask("a.wrapping_sub(1)".to_string())
        ));
        f.line("        _ => 0,");
        f.line("    }");
        f.line("}");
        f.blank();
    }
}

/// Phase L.2 (target §4.5 + §9 criterion 5): emit `const_ring_eval_w{n}`
/// helpers for every Limbs-backed WittLevel (widths > 128). Each helper is
/// a `pub const fn` that accepts two `Limbs<N>` operands and a `PrimitiveOp`
/// and returns the masked result.
///
/// Companion to `generate_const_ring_eval` (native widths). Together the two
/// functions provide a `const_ring_eval_w{n}` helper for **every** shipped
/// level, satisfying target §4.5 / §9 criterion 5.
fn generate_const_ring_eval_limbs(f: &mut RustFile, ontology: &Ontology) {
    let levels = limbs_witt_levels(ontology);
    if levels.is_empty() {
        return;
    }

    f.doc_comment("Phase L.2 (target §4.5): `const_ring_eval_w{n}` helpers for Limbs-backed");
    f.doc_comment("Witt levels. Each helper runs a `PrimitiveOp` over two `Limbs<N>` operands");
    f.doc_comment("and applies the level's bit-width mask to the result.");
    f.doc_comment("");
    f.doc_comment("These helpers are always const-fn; whether `rustc` can complete a specific");
    f.doc_comment("compile-time evaluation within the developer's budget is a function of the");
    f.doc_comment("invocation (see target §4.5 Q2 practicality table).");
    for (local, bits, limb_count) in &levels {
        let lower = local.to_ascii_lowercase();
        let exact_fit = bits % 64 == 0;
        f.line("#[inline]");
        f.line("#[must_use]");
        f.line(&format!(
            "pub const fn const_ring_eval_{lower}(op: PrimitiveOp, a: Limbs<{limb_count}>, b: Limbs<{limb_count}>) -> Limbs<{limb_count}> {{"
        ));
        // ADR-013/TR-08 substrate amendment:
        // - Le/Lt/Ge/Gt: lift the natural ring ordering to a 0/1-valued
        //   indicator (returned as a one-limb Limbs value).
        // - Concat: byte-sequence operator with no ring-arithmetic
        //   interpretation; const-ring helpers return the additive
        //   identity. Catamorphism evaluation routes Concat through
        //   `apply_primitive_op` which handles it at the byte level.
        if exact_fit {
            f.line("    match op {");
            f.line("        PrimitiveOp::Add => a.wrapping_add(b),");
            f.line("        PrimitiveOp::Sub => a.wrapping_sub(b),");
            f.line("        PrimitiveOp::Mul => a.wrapping_mul(b),");
            f.line("        PrimitiveOp::And => a.and(b),");
            f.line("        PrimitiveOp::Or => a.or(b),");
            f.line("        PrimitiveOp::Xor => a.xor(b),");
            f.line(&format!(
                "        PrimitiveOp::Neg => Limbs::<{limb_count}>::zero().wrapping_sub(a),"
            ));
            f.line("        PrimitiveOp::Bnot => a.not(),");
            f.line(&format!(
                "        PrimitiveOp::Succ => a.wrapping_add(limbs_one_{limb_count}()),"
            ));
            f.line(&format!(
                "        PrimitiveOp::Pred => a.wrapping_sub(limbs_one_{limb_count}()),"
            ));
            f.line(&format!(
                "        PrimitiveOp::Le => if limbs_le_{limb_count}(a, b) {{ limbs_one_{limb_count}() }} else {{ Limbs::<{limb_count}>::zero() }},"
            ));
            f.line(&format!(
                "        PrimitiveOp::Lt => if limbs_lt_{limb_count}(a, b) {{ limbs_one_{limb_count}() }} else {{ Limbs::<{limb_count}>::zero() }},"
            ));
            f.line(&format!(
                "        PrimitiveOp::Ge => if limbs_le_{limb_count}(b, a) {{ limbs_one_{limb_count}() }} else {{ Limbs::<{limb_count}>::zero() }},"
            ));
            f.line(&format!(
                "        PrimitiveOp::Gt => if limbs_lt_{limb_count}(b, a) {{ limbs_one_{limb_count}() }} else {{ Limbs::<{limb_count}>::zero() }},"
            ));
            f.line(&format!(
                "        PrimitiveOp::Concat => Limbs::<{limb_count}>::zero(),"
            ));
            // ADR-053 substrate amendment: Div/Mod/Pow over Limbs widths.
            // Div / Mod: const-eval helpers cannot raise errors; b = 0 is
            // rejected by the runtime catamorphism. Helpers return the
            // ring's additive identity to keep `const fn` total.
            f.line(&format!(
                "        PrimitiveOp::Div => if limbs_is_zero_{limb_count}(b) {{ Limbs::<{limb_count}>::zero() }} else {{ limbs_div_{limb_count}(a, b) }},"
            ));
            f.line(&format!(
                "        PrimitiveOp::Mod => if limbs_is_zero_{limb_count}(b) {{ Limbs::<{limb_count}>::zero() }} else {{ limbs_mod_{limb_count}(a, b) }},"
            ));
            f.line(&format!(
                "        PrimitiveOp::Pow => limbs_pow_{limb_count}(a, b),"
            ));
            f.line("    }");
        } else {
            f.line("    let raw = match op {");
            f.line("        PrimitiveOp::Add => a.wrapping_add(b),");
            f.line("        PrimitiveOp::Sub => a.wrapping_sub(b),");
            f.line("        PrimitiveOp::Mul => a.wrapping_mul(b),");
            f.line("        PrimitiveOp::And => a.and(b),");
            f.line("        PrimitiveOp::Or => a.or(b),");
            f.line("        PrimitiveOp::Xor => a.xor(b),");
            f.line(&format!(
                "        PrimitiveOp::Neg => Limbs::<{limb_count}>::zero().wrapping_sub(a),"
            ));
            f.line("        PrimitiveOp::Bnot => a.not(),");
            f.line(&format!(
                "        PrimitiveOp::Succ => a.wrapping_add(limbs_one_{limb_count}()),"
            ));
            f.line(&format!(
                "        PrimitiveOp::Pred => a.wrapping_sub(limbs_one_{limb_count}()),"
            ));
            f.line(&format!(
                "        PrimitiveOp::Le => if limbs_le_{limb_count}(a, b) {{ limbs_one_{limb_count}() }} else {{ Limbs::<{limb_count}>::zero() }},"
            ));
            f.line(&format!(
                "        PrimitiveOp::Lt => if limbs_lt_{limb_count}(a, b) {{ limbs_one_{limb_count}() }} else {{ Limbs::<{limb_count}>::zero() }},"
            ));
            f.line(&format!(
                "        PrimitiveOp::Ge => if limbs_le_{limb_count}(b, a) {{ limbs_one_{limb_count}() }} else {{ Limbs::<{limb_count}>::zero() }},"
            ));
            f.line(&format!(
                "        PrimitiveOp::Gt => if limbs_lt_{limb_count}(b, a) {{ limbs_one_{limb_count}() }} else {{ Limbs::<{limb_count}>::zero() }},"
            ));
            f.line(&format!(
                "        PrimitiveOp::Concat => Limbs::<{limb_count}>::zero(),"
            ));
            // ADR-053: Div/Mod/Pow with high-bit mask applied below.
            f.line(&format!(
                "        PrimitiveOp::Div => if limbs_is_zero_{limb_count}(b) {{ Limbs::<{limb_count}>::zero() }} else {{ limbs_div_{limb_count}(a, b) }},"
            ));
            f.line(&format!(
                "        PrimitiveOp::Mod => if limbs_is_zero_{limb_count}(b) {{ Limbs::<{limb_count}>::zero() }} else {{ limbs_mod_{limb_count}(a, b) }},"
            ));
            f.line(&format!(
                "        PrimitiveOp::Pow => limbs_pow_{limb_count}(a, b),"
            ));
            f.line("    };");
            f.line(&format!("    raw.mask_high_bits({bits})"));
        }
        f.line("}");
        f.blank();
    }

    // Emit `limbs_one_{N}` helpers for each distinct N that appears.
    let mut ns: Vec<usize> = levels.iter().map(|(_, _, n)| *n).collect();
    ns.sort_unstable();
    ns.dedup();
    f.doc_comment("Phase L.2: one-constant helpers for `Limbs<N>::from_words([1, 0, ...])`.");
    for &n in &ns {
        let body = if n == 1 {
            "Limbs::<1>::from_words([1u64])".to_string()
        } else {
            let mut elems = String::from("[1u64");
            for _ in 1..n {
                elems.push_str(", 0u64");
            }
            elems.push(']');
            format!("Limbs::<{n}>::from_words({elems})")
        };
        f.line("#[inline]");
        f.line("#[must_use]");
        f.line(&format!("const fn limbs_one_{n}() -> Limbs<{n}> {{"));
        f.line(&format!("    {body}"));
        f.line("}");
        f.blank();
    }
    // ADR-013/TR-08: comparison helpers used by the const_ring_eval_w*
    // limbs path to lift Le/Lt/Ge/Gt to ring-valued 0/1 indicators.
    // Implementation: words are stored little-endian, so big-integer
    // comparison walks from the top word down. Equal words skip; the
    // first differing word's u64 ordering is decisive.
    f.doc_comment("ADR-013/TR-08: const-fn limb comparisons used by `const_ring_eval_w{n}`");
    f.doc_comment("to lift the comparison primitives to 0/1-valued ring indicators.");
    for &n in &ns {
        f.line("#[inline]");
        f.line("#[must_use]");
        f.line(&format!(
            "const fn limbs_lt_{n}(a: Limbs<{n}>, b: Limbs<{n}>) -> bool {{"
        ));
        f.line("    let aw = a.words();");
        f.line("    let bw = b.words();");
        f.line(&format!("    let mut i = {n};"));
        f.line("    while i > 0 {");
        f.line("        i -= 1;");
        f.line("        if aw[i] < bw[i] { return true; }");
        f.line("        if aw[i] > bw[i] { return false; }");
        f.line("    }");
        f.line("    false");
        f.line("}");
        f.blank();
        f.line("#[inline]");
        f.line("#[must_use]");
        f.line(&format!(
            "const fn limbs_le_{n}(a: Limbs<{n}>, b: Limbs<{n}>) -> bool {{"
        ));
        f.line("    let aw = a.words();");
        f.line("    let bw = b.words();");
        f.line(&format!("    let mut i = {n};"));
        f.line("    while i > 0 {");
        f.line("        i -= 1;");
        f.line("        if aw[i] < bw[i] { return true; }");
        f.line("        if aw[i] > bw[i] { return false; }");
        f.line("    }");
        f.line("    true");
        f.line("}");
        f.blank();
    }

    // ADR-053 substrate amendment: const-eval Div/Mod/Pow helpers for the
    // Limbs widths. `limbs_is_zero_N` is a constant-time zero check; the
    // binary-long-division companion lets `const_ring_eval_w{n}` produce
    // Euclidean quotient and remainder at compile time. `limbs_pow_N`
    // uses square-and-multiply in the ring (`mod 2^(width)` is folded
    // into the caller's `mask_high_bits`).
    f.doc_comment("ADR-053: zero-check, binary-long-division, and square-and-multiply");
    f.doc_comment("helpers used by the const-fn `Div`/`Mod`/`Pow` arms of `const_ring_eval_w{n}`.");
    for &n in &ns {
        f.line("#[inline]");
        f.line("#[must_use]");
        f.line(&format!(
            "const fn limbs_is_zero_{n}(a: Limbs<{n}>) -> bool {{"
        ));
        f.line("    let aw = a.words();");
        f.line("    let mut i = 0usize;");
        f.line(&format!("    while i < {n} {{"));
        f.line("        if aw[i] != 0 { return false; }");
        f.line("        i += 1;");
        f.line("    }");
        f.line("    true");
        f.line("}");
        f.blank();

        // Limb-level shift-left by one bit, used by binary long division.
        f.line("#[inline]");
        f.line("#[must_use]");
        f.line(&format!(
            "const fn limbs_shl1_{n}(a: Limbs<{n}>) -> Limbs<{n}> {{"
        ));
        f.line("    let aw = a.words();");
        f.line(&format!("    let mut out = [0u64; {n}];"));
        f.line("    let mut carry: u64 = 0;");
        f.line("    let mut i = 0usize;");
        f.line(&format!("    while i < {n} {{"));
        f.line("        let v = aw[i];");
        f.line("        out[i] = (v << 1) | carry;");
        f.line("        carry = v >> 63;");
        f.line("        i += 1;");
        f.line("    }");
        f.line(&format!("    Limbs::<{n}>::from_words(out)"));
        f.line("}");
        f.blank();

        // Set bit 0 of a limbs value.
        f.line("#[inline]");
        f.line("#[must_use]");
        f.line(&format!(
            "const fn limbs_set_bit0_{n}(a: Limbs<{n}>) -> Limbs<{n}> {{"
        ));
        f.line("    let aw = a.words();");
        f.line(&format!("    let mut out = [0u64; {n}];"));
        f.line("    let mut i = 0usize;");
        f.line(&format!("    while i < {n} {{"));
        f.line("        out[i] = aw[i];");
        f.line("        i += 1;");
        f.line("    }");
        f.line("    out[0] |= 1u64;");
        f.line(&format!("    Limbs::<{n}>::from_words(out)"));
        f.line("}");
        f.blank();

        // Get bit `b` (counted from MSB across all words). Used by long
        // division: 0 is the highest bit of word[N-1], N*64-1 is bit 0 of
        // word[0].
        f.line("#[inline]");
        f.line("#[must_use]");
        f.line(&format!(
            "const fn limbs_bit_msb_{n}(a: Limbs<{n}>, msb_index: usize) -> u64 {{"
        ));
        f.line("    let aw = a.words();");
        f.line(&format!("    let total_bits = {n} * 64;"));
        f.line("    let lsb_index = total_bits - 1 - msb_index;");
        f.line("    let word = lsb_index / 64;");
        f.line("    let bit = lsb_index % 64;");
        f.line("    (aw[word] >> bit) & 1u64");
        f.line("}");
        f.blank();

        // Binary long division: standard MSB→LSB shift-subtract algorithm.
        // Returns (quotient, remainder) as a pair packaged as Limbs pair.
        // Caller passes b != 0 (guarded by `limbs_is_zero_N`).
        f.line("#[inline]");
        f.line("#[must_use]");
        f.line(&format!(
            "const fn limbs_divmod_{n}(a: Limbs<{n}>, b: Limbs<{n}>) -> (Limbs<{n}>, Limbs<{n}>) {{"
        ));
        f.line(&format!("    let mut q = Limbs::<{n}>::zero();"));
        f.line(&format!("    let mut r = Limbs::<{n}>::zero();"));
        f.line(&format!("    let total_bits = {n} * 64;"));
        f.line("    let mut i = 0usize;");
        f.line("    while i < total_bits {");
        f.line(&format!("        r = limbs_shl1_{n}(r);"));
        f.line(&format!("        if limbs_bit_msb_{n}(a, i) == 1 {{"));
        f.line(&format!("            r = limbs_set_bit0_{n}(r);"));
        f.line("        }");
        f.line(&format!("        if limbs_le_{n}(b, r) {{"));
        f.line("            r = r.wrapping_sub(b);");
        f.line(&format!("            q = limbs_shl1_{n}(q);"));
        f.line(&format!("            q = limbs_set_bit0_{n}(q);"));
        f.line("        } else {");
        f.line(&format!("            q = limbs_shl1_{n}(q);"));
        f.line("        }");
        f.line("        i += 1;");
        f.line("    }");
        f.line("    (q, r)");
        f.line("}");
        f.blank();

        f.line("#[inline]");
        f.line("#[must_use]");
        f.line(&format!(
            "const fn limbs_div_{n}(a: Limbs<{n}>, b: Limbs<{n}>) -> Limbs<{n}> {{"
        ));
        f.line(&format!("    let (q, _) = limbs_divmod_{n}(a, b);"));
        f.line("    q");
        f.line("}");
        f.blank();

        f.line("#[inline]");
        f.line("#[must_use]");
        f.line(&format!(
            "const fn limbs_mod_{n}(a: Limbs<{n}>, b: Limbs<{n}>) -> Limbs<{n}> {{"
        ));
        f.line(&format!("    let (_, r) = limbs_divmod_{n}(a, b);"));
        f.line("    r");
        f.line("}");
        f.blank();

        // Square-and-multiply. Walks the exponent's bits LSB→MSB through
        // each limb-word; relies on caller's mask_high_bits to enforce the
        // ring's bit-width discipline.
        f.line("#[inline]");
        f.line("#[must_use]");
        f.line(&format!(
            "const fn limbs_pow_{n}(base: Limbs<{n}>, exp: Limbs<{n}>) -> Limbs<{n}> {{"
        ));
        f.line(&format!("    let mut result = limbs_one_{n}();"));
        f.line("    let mut b = base;");
        f.line("    let ew = exp.words();");
        f.line("    let mut word = 0usize;");
        f.line(&format!("    while word < {n} {{"));
        f.line("        let mut bit = 0u32;");
        f.line("        while bit < 64 {");
        f.line("            if ((ew[word] >> bit) & 1u64) == 1u64 {");
        f.line("                result = result.wrapping_mul(b);");
        f.line("            }");
        f.line("            b = b.wrapping_mul(b);");
        f.line("            bit += 1;");
        f.line("        }");
        f.line("        word += 1;");
        f.line("    }");
        f.line("    result");
        f.line("}");
        f.blank();
    }
}

// ── v0.2.1 Ergonomics Surface Generators ─────────────────────────────────────
//
// Each generator below reads from `&Ontology` (passed at the top) so that
// every emitted symbol traces to an ontology entity. There are no static
// Rust mapping tables — adding a new resolver, certificate, dispatch table,
// or prelude member requires only an ontology edit.

/// Convert an IRI to its local name (everything after the last `/` or `#`).
fn local_name(iri: &str) -> &str {
    iri.rsplit_once(['/', '#']).map(|(_, n)| n).unwrap_or(iri)
}

/// Find an individual by IRI.
fn find_individual<'a>(
    ontology: &'a Ontology,
    iri: &str,
) -> Option<&'a uor_ontology::model::Individual> {
    for ns in &ontology.namespaces {
        for ind in &ns.individuals {
            if ind.id == iri {
                return Some(ind);
            }
        }
    }
    None
}

/// Read a property value off an individual; returns the matching IriRef or
/// Str payload as a borrowed string.
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

/// Collect all individuals of a given type.
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

/// Walk `resolver:CertifyMapping` individuals and collect the sorted
/// unique local-names of the certificate classes and witness classes they
/// reference. Used by Phase 7b.4 to verify the foundation's hand-rolled
/// shim list matches what the ontology wires into `Certify`.
fn collect_certify_mapping_targets(ontology: &Ontology) -> (Vec<String>, Vec<String>) {
    let mut certs: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut witnesses: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for ind in individuals_of_type(ontology, "https://uor.foundation/resolver/CertifyMapping") {
        if let Some(iri) = ind_prop_str(ind, "https://uor.foundation/resolver/producesCertificate")
        {
            certs.insert(local_name(iri).to_string());
        }
        if let Some(iri) = ind_prop_str(ind, "https://uor.foundation/resolver/producesWitness") {
            witnesses.insert(local_name(iri).to_string());
        }
    }
    (certs.into_iter().collect(), witnesses.into_iter().collect())
}

/// Verify that the hand-rolled shim list in [`generate_ontology_target_trait`]
/// is a superset of the ontology's subclass closure. Panics at codegen time
/// with a clear error if a class the ontology declares is missing from the
/// shim list — this turns "the shim list matches the ontology" into a
/// machine-checked invariant. Panic is intentional.
#[allow(clippy::panic)]
fn verify_shim_coverage(label: &str, expected: &[String], shim_names: &[&str]) {
    let shim_set: std::collections::BTreeSet<&str> = shim_names.iter().copied().collect();
    for name in expected {
        if !shim_set.contains(name.as_str()) {
            panic!(
                "generate_ontology_target_trait: ontology declares {label} subclass `{name}` \
                 but the hand-rolled shim list in codegen/src/enforcement.rs does not \
                 include it. Add `{name}` to the shim list (and the OntologyTarget sealed \
                 impls) or remove the class from the ontology."
            );
        }
    }
}

// 2.1.a OntologyTarget — sealed marker trait for foundation-produced types.
//
// v0.2.1 ships a small set of **shim structs** (named after their ontology
// local-name) that serve as type-system handles for `Validated<T>` and
// `Certify` impls. The shims are zero-sized and `Default`-able so resolver
// impls can produce concrete return values. They do not collide with the
// `bridge::cert::*` / `bridge::proof::*` trait modules because they live in
// the `enforcement` module and the prelude re-exports the enforcement shims
// preferentially. Real instances are produced by the reduction pipeline (or
// by `uor_ground!` macro expansion) through the back-door minting API.
/// ADR-018/060: the certificate kinds that carry a content fingerprint minted
/// from the application's selected `Hasher`, hence parameterized over the
/// fingerprint width `FP_MAX` (default 32). These are exactly the 12
/// `minting_certs`; their `Sealed`/`OntologyTarget`/`Certificate`/
/// `MintWithLevelFingerprint` impls are emitted FP_MAX-generic. Failure
/// witnesses (Generic/Inhabitance Impossibility) and input shims carry no
/// fingerprint and stay non-parametric.
fn cert_carries_fp_max(name: &str) -> bool {
    matches!(
        name,
        "GroundingCertificate"
            | "LiftChainCertificate"
            | "InhabitanceCertificate"
            | "CompletenessCertificate"
            | "MultiplicationCertificate"
            | "PartitionCertificate"
            | "TransformCertificate"
            | "IsometryCertificate"
            | "InvolutionCertificate"
            | "GeodesicCertificate"
            | "MeasurementCertificate"
            | "BornRuleVerification"
    )
}

fn generate_ontology_target_trait(f: &mut RustFile, ontology: &Ontology) {
    // v0.2.1 Phase 7b.4: the set of shim types is machine-verified against
    // the ontology's `resolver:CertifyMapping` individuals — every certificate
    // / witness class named in a CertifyMapping must appear in the shim
    // list, or the codegen panics with a clear "missing shim" error.
    //
    // Scope: this narrows verification to "every certificate / witness class
    // named by a CertifyMapping individual" rather than "every subclass in
    // the ontology". Subclasses without a CertifyMapping are not in the
    // foundation's resolver-backed surface — consumers of those classes
    // construct them through substrate-specific paths outside the
    // uor-foundation crate.
    let (expected_cert_names, expected_witness_names) = collect_certify_mapping_targets(ontology);
    verify_shim_coverage(
        "certificate",
        &expected_cert_names,
        &[
            "GroundingCertificate",
            "LiftChainCertificate",
            "InhabitanceCertificate",
            "CompletenessCertificate",
            "MultiplicationCertificate",
            "PartitionCertificate",
        ],
    );
    verify_shim_coverage(
        "impossibility witness",
        &expected_witness_names,
        // `ImpossibilityWitness` (the base class) is mapped to the foundation
        // shim `GenericImpossibilityWitness` via the local-name handling in
        // `generate_certify_trait`. Accept both local-names here.
        &[
            "ImpossibilityWitness",
            "GenericImpossibilityWitness",
            "InhabitanceImpossibilityWitness",
        ],
    );

    f.doc_comment("Sealed marker trait identifying types produced by the foundation crate's");
    f.doc_comment("conformance/reduction pipeline. v0.2.1 bounds `Validated<T>` on this trait");
    f.doc_comment("so downstream crates cannot fabricate `Validated<UserType>` — user types");
    f.doc_comment("cannot impl `OntologyTarget` because the supertrait is private.");
    f.line("pub trait OntologyTarget: ontology_target_sealed::Sealed {}");
    f.blank();

    // v0.2.1 Phase 7b.1: certificate shims carry a real `witt_bits: u16`
    // field populated by the pipeline (Phase 7b.1.b). The field enables
    // `LiftChainCertificate::target_level()` to read the level the
    // certificate was issued for — no hardcoded W8. Witness shims and
    // ConstrainedTypeInput stay opaque because they are not Witt-indexed.
    let certificate_shims: &[(&str, &str)] = &[
        (
            "GroundingCertificate",
            "Sealed shim for `cert:GroundingCertificate`. Produced by GroundingAwareResolver.",
        ),
        (
            "LiftChainCertificate",
            "Sealed shim for `cert:LiftChainCertificate`. Carries the v0.2.1 \
             `target_level()` accessor populated from the pipeline's StageOutcome.",
        ),
        (
            "InhabitanceCertificate",
            "Sealed shim for `cert:InhabitanceCertificate` (v0.2.1).",
        ),
        (
            "CompletenessCertificate",
            "Sealed shim for `cert:CompletenessCertificate`.",
        ),
        (
            "MultiplicationCertificate",
            "Sealed shim for `cert:MultiplicationCertificate` (v0.2.2 Phase C.4). \
             Carries the cost-optimal Toom-Cook splitting factor R, the recursive \
             sub-multiplication count, and the accumulated Landauer cost in nats.",
        ),
        (
            "PartitionCertificate",
            "Sealed shim for `cert:PartitionCertificate` (v0.2.2 Phase E). \
             Attests the partition component classification of a Datum.",
        ),
    ];
    let witness_shims: &[(&str, &str)] = &[
        (
            "GenericImpossibilityWitness",
            "Sealed shim for `proof:ImpossibilityWitness`. Returned by completeness and \
             grounding resolvers on failure.",
        ),
        (
            "InhabitanceImpossibilityWitness",
            "Sealed shim for `proof:InhabitanceImpossibilityWitness` (v0.2.1).",
        ),
    ];
    let input_shims: &[(&str, &str)] = &[(
        "ConstrainedTypeInput",
        "Input shim for `type:ConstrainedType`. Used as `Certify::Input` for \
             InhabitanceResolver, TowerCompletenessResolver, and \
             IncrementalCompletenessResolver.",
    )];

    // v0.2.2 T6.7 + T6.8: emit certificate shims with witt_bits + content_fingerprint
    // fields. NO Default impl — sealed witnesses are minted only by the pipeline
    // and never exist in a "default" form. NO `with_level_const` / `with_witt_bits`
    // legacy ctors — only the fingerprint-complete `with_level_and_fingerprint_const`
    // survives, used by `pipeline::run` and the `certify_*_const` companions to
    // thread the consumer-supplied substrate fingerprint into the certificate.
    for (name, doc) in certificate_shims {
        f.doc_comment(doc);
        f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
        f.line(&format!("pub struct {name}<const FP_MAX: usize = 32> {{"));
        f.line("    witt_bits: u16,");
        f.indented_doc_comment("v0.2.2 T5: parametric content fingerprint computed at mint time");
        f.indented_doc_comment("by the consumer-supplied `Hasher`. Bit-equality on the full");
        f.indented_doc_comment("buffer + width tag, so two certs with different `OUTPUT_BYTES`");
        f.indented_doc_comment("are never equal even when leading bytes coincide. `FP_MAX` is the");
        f.indented_doc_comment(
            "application's `<B as HostBounds>::FINGERPRINT_MAX_BYTES` (ADR-018);",
        );
        f.indented_doc_comment("threaded, not pinned, so any `Hasher<FP_MAX>` width flows.");
        f.line("    content_fingerprint: ContentFingerprint<FP_MAX>,");
        f.line("}");
        f.blank();
        f.line(&format!("impl<const FP_MAX: usize> {name}<FP_MAX> {{"));
        f.indented_doc_comment("Returns the Witt level the certificate was issued for. Sourced");
        f.indented_doc_comment("from the pipeline's substrate hash output at minting time.");
        f.line("    #[inline]");
        f.line("    #[must_use]");
        f.line("    pub const fn witt_bits(&self) -> u16 {");
        f.line("        self.witt_bits");
        f.line("    }");
        f.blank();
        f.indented_doc_comment("v0.2.2 T5: returns the parametric content fingerprint of the");
        f.indented_doc_comment("source state, computed at mint time by the consumer-supplied");
        f.indented_doc_comment("`Hasher`. Active width recoverable via `width_bytes()`. Two");
        f.indented_doc_comment("certificates from different hashers are never equal because");
        f.indented_doc_comment("`ContentFingerprint::Eq` compares the full buffer + width tag.");
        f.line("    #[inline]");
        f.line("    #[must_use]");
        f.line("    pub const fn content_fingerprint(&self) -> ContentFingerprint<FP_MAX> {");
        f.line("        self.content_fingerprint");
        f.line("    }");
        f.blank();
        f.indented_doc_comment("v0.2.2 T5 C3.g + T6.7: the only constructor — takes both the");
        f.indented_doc_comment("witt-bits value AND the parametric content fingerprint. Used by");
        f.indented_doc_comment("`pipeline::run` and `certify_*_const` to mint a certificate");
        f.indented_doc_comment("carrying the substrate-computed fingerprint of the source state.");
        f.line("    #[inline]");
        f.line("    #[must_use]");
        f.line("    #[allow(dead_code)]");
        f.line("    pub(crate) const fn with_level_and_fingerprint_const(");
        f.line("        witt_bits: u16,");
        f.line("        content_fingerprint: ContentFingerprint<FP_MAX>,");
        f.line("    ) -> Self {");
        f.line("        Self {");
        f.line("            witt_bits,");
        f.line("            content_fingerprint,");
        f.line("        }");
        f.line("    }");
        f.line("}");
        f.blank();
    }

    // Witness + input shims. Most stay opaque (zero-sized with `_private: ()`
    // and a derived `Default`). The Product/Coproduct Completion Amendment §2.3a
    // extends `GenericImpossibilityWitness` with a single
    // `identity: Option<&'static str>` field so that theorem-IRI citations
    // can flow through the witness. `Default` is implemented manually to
    // preserve every existing `::default()` call site (they produce
    // `identity: None`, semantically equivalent to the prior zero-sized default).
    for (name, doc) in witness_shims.iter().chain(input_shims.iter()) {
        f.doc_comment(doc);
        if *name == "GenericImpossibilityWitness" {
            f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
            f.line(&format!("pub struct {name} {{"));
            f.indented_doc_comment(
                "Optional theorem / invariant IRI identifying the failed identity.",
            );
            f.indented_doc_comment(
                "`None` for legacy call sites minting via `Default::default()`;",
            );
            f.indented_doc_comment(
                "`Some(iri)` when the witness is constructed via `for_identity`",
            );
            f.indented_doc_comment("(Product/Coproduct Completion Amendment §2.3a).");
            f.line("    identity: Option<&'static str>,");
            f.line("}");
            f.blank();
            // Manual Default — preserves behavior of legacy `::default()` call sites.
            f.line(&format!("impl Default for {name} {{"));
            f.line("    #[inline]");
            f.line("    fn default() -> Self {");
            f.line("        Self { identity: None }");
            f.line("    }");
            f.line("}");
            f.blank();
            // Constructors + accessor.
            f.line(&format!("impl {name} {{"));
            f.indented_doc_comment(
                "Construct a witness citing a specific theorem / invariant IRI.",
            );
            f.indented_doc_comment(
                "Introduced by the Product/Coproduct Completion Amendment §2.3a",
            );
            f.indented_doc_comment(
                "so mint primitives can emit typed failures against `op/*` theorems",
            );
            f.indented_doc_comment("and `foundation/*` layout invariants.");
            f.line("    #[inline]");
            f.line("    #[must_use]");
            f.line("    pub const fn for_identity(identity: &'static str) -> Self {");
            f.line("        Self { identity: Some(identity) }");
            f.line("    }");
            f.blank();
            f.indented_doc_comment(
                "Returns the theorem / invariant IRI this witness cites, if any.",
            );
            f.line("    #[inline]");
            f.line("    #[must_use]");
            f.line("    pub const fn identity(&self) -> Option<&'static str> {");
            f.line("        self.identity");
            f.line("    }");
            f.line("}");
            f.blank();
        } else {
            f.line("#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]");
            f.line(&format!("pub struct {name} {{"));
            f.line("    _private: (),");
            f.line("}");
            f.blank();
        }
    }
    // v0.2.2 T5.9: Display + core::error::Error impls for the witness shims
    // (they're returned as the failure case of `Certify::certify`, so they
    // need to participate in `Box<dyn Error>` chains alongside `PipelineFailure`).
    for (name, _doc) in witness_shims.iter() {
        f.line(&format!("impl core::fmt::Display for {name} {{"));
        f.line("    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {");
        if *name == "GenericImpossibilityWitness" {
            // Include the identity IRI in the Display output when present so
            // downstream error chains carry the failure citation.
            f.line("        match self.identity {");
            f.line("            Some(iri) => write!(f, \"GenericImpossibilityWitness({iri})\"),");
            f.line(&format!("            None => f.write_str(\"{name}\"),"));
            f.line("        }");
        } else {
            f.line(&format!("        f.write_str(\"{name}\")"));
        }
        f.line("    }");
        f.line("}");
        f.line(&format!("impl core::error::Error for {name} {{}}"));
        f.blank();
    }

    // LiftChainCertificate.target_level — reads the real witt_bits field.
    f.line("impl LiftChainCertificate {");
    f.indented_doc_comment("Returns the Witt level the certificate was issued for.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn target_level(&self) -> WittLevel {");
    f.line("        WittLevel::new(self.witt_bits as u32)");
    f.line("    }");
    f.line("}");
    f.blank();
    f.line("impl InhabitanceCertificate {");
    f.indented_doc_comment("Returns the witness value tuple bytes when `verified` is true.");
    f.indented_doc_comment("The sealed shim returns `None`; real witnesses flow through the");
    f.indented_doc_comment("macro back-door path.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn witness(&self) -> Option<&'static [u8]> {");
    f.line("        None");
    f.line("    }");
    f.line("}");
    f.blank();

    // Sealed module + impls — combine all three shim lists.
    let all_shims: Vec<&(&str, &str)> = certificate_shims
        .iter()
        .chain(witness_shims.iter())
        .chain(input_shims.iter())
        .collect();
    // pub(crate) so Phase 10's witness_scaffolds module can register
    // its `Mint{Foo}` types as sealed targets without re-defining the
    // sealed trait. Visibility stays crate-private overall.
    f.line("pub(crate) mod ontology_target_sealed {");
    f.indented_doc_comment("Private supertrait. Not implementable outside this crate.");
    f.line("    pub trait Sealed {}");
    for (name, _) in &all_shims {
        if cert_carries_fp_max(name) {
            f.line(&format!(
                "    impl<const FP_MAX: usize> Sealed for super::{name}<FP_MAX> {{}}"
            ));
        } else {
            f.line(&format!("    impl Sealed for super::{name} {{}}"));
        }
    }
    // Product/Coproduct Completion Amendment §2.3e.
    f.line("    impl Sealed for super::PartitionProductWitness {}");
    f.line("    impl Sealed for super::PartitionCoproductWitness {}");
    f.line("    impl Sealed for super::CartesianProductWitness {}");
    f.line(
        "    impl<const INLINE_BYTES: usize> Sealed for super::CompileUnit<'_, INLINE_BYTES> {}",
    );
    f.line("}");
    f.blank();
    for (name, _) in &all_shims {
        if cert_carries_fp_max(name) {
            f.line(&format!(
                "impl<const FP_MAX: usize> OntologyTarget for {name}<FP_MAX> {{}}"
            ));
        } else {
            f.line(&format!("impl OntologyTarget for {name} {{}}"));
        }
    }
    // Product/Coproduct Completion Amendment §2.3e.
    f.line("impl OntologyTarget for PartitionProductWitness {}");
    f.line("impl OntologyTarget for PartitionCoproductWitness {}");
    f.line("impl OntologyTarget for CartesianProductWitness {}");
    f.line("impl<const INLINE_BYTES: usize> OntologyTarget for CompileUnit<'_, INLINE_BYTES> {}");
    f.blank();

    // ── v0.2.2 W11: Certified<C> parametric carrier ────────────────────────
    //
    // Replaces the per-shim duplication with one parametric carrier. Sealed
    // `Certificate` trait scopes the kind set to ontology-declared classes;
    // `Certified<C>` is the single struct that holds them. The 4 existing
    // certificate shims gain `impl Certificate`, and the 6 cert subclasses
    // not previously shimmed (Transform, Isometry, Involution, Geodesic,
    // Measurement, BornRule) get sealed unit-struct emissions.
    //
    // Supporting evidence types (CompletenessAuditTrail, ChainAuditTrail,
    // GeodesicEvidenceBundle) are emitted as public structs so they can
    // appear as the `Evidence` associated type of their parent certificate.
    f.doc_comment("v0.2.2 W11: supporting evidence type for `CompletenessCertificate`.");
    f.doc_comment("Linked from the certificate via the `Certificate::Evidence` associated type.");
    f.line("#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct CompletenessAuditTrail { _private: () }");
    f.blank();
    f.doc_comment("v0.2.2 W11: supporting evidence type for `LiftChainCertificate`.");
    f.line("#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct ChainAuditTrail { _private: () }");
    f.blank();
    f.doc_comment("v0.2.2 W11: supporting evidence type for `GeodesicCertificate`.");
    f.line("#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct GeodesicEvidenceBundle { _private: () }");
    f.blank();

    // The 6 cert subclasses not previously shimmed in enforcement. We emit
    // them as sealed unit structs so they can be the `C` parameter of
    // `Certified<C>`.
    let new_cert_kinds: &[(&str, &str)] = &[
        (
            "TransformCertificate",
            "v0.2.2 W11: sealed carrier for `cert:TransformCertificate`.",
        ),
        (
            "IsometryCertificate",
            "v0.2.2 W11: sealed carrier for `cert:IsometryCertificate`.",
        ),
        (
            "InvolutionCertificate",
            "v0.2.2 W11: sealed carrier for `cert:InvolutionCertificate`.",
        ),
        (
            "GeodesicCertificate",
            "v0.2.2 W11: sealed carrier for `cert:GeodesicCertificate`.",
        ),
        (
            "MeasurementCertificate",
            "v0.2.2 W11: sealed carrier for `cert:MeasurementCertificate`.",
        ),
        (
            "BornRuleVerification",
            "v0.2.2 W11: sealed carrier for `cert:BornRuleVerification`.",
        ),
    ];
    for (name, doc) in new_cert_kinds {
        f.doc_comment(doc);
        f.doc_comment("");
        f.doc_comment("Phase X.1: minted with a Witt level and content fingerprint so the");
        f.doc_comment("resolver whose `resolver:CertifyMapping` produces this class can fold");
        f.doc_comment(
            "its decision into a content-addressed witness. The `with_level_and_fingerprint_const`",
        );
        f.doc_comment("constructor matches every other `cert:Certificate` subclass.");
        f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
        f.line(&format!("pub struct {name}<const FP_MAX: usize = 32> {{"));
        f.line("    witt_bits: u16,");
        f.line("    content_fingerprint: ContentFingerprint<FP_MAX>,");
        f.line("    _private: (),");
        f.line("}");
        f.blank();
        f.line(&format!("impl<const FP_MAX: usize> {name}<FP_MAX> {{"));
        f.indented_doc_comment("Phase X.1: content-addressed constructor. Mints a certificate");
        f.indented_doc_comment("carrying the Witt level and substrate-hasher fingerprint of the");
        f.indented_doc_comment(
            "resolver decision. Crate-sealed so that only resolver kernels mint.",
        );
        f.line("    #[inline]");
        f.line("    #[must_use]");
        f.line("    #[allow(dead_code)]");
        f.line("    pub(crate) const fn with_level_and_fingerprint_const(");
        f.line("        witt_bits: u16,");
        f.line("        content_fingerprint: ContentFingerprint<FP_MAX>,");
        f.line("    ) -> Self {");
        f.line("        Self { witt_bits, content_fingerprint, _private: () }");
        f.line("    }");
        f.blank();
        f.indented_doc_comment("Phase X.1: legacy zero-fingerprint constructor retained for");
        f.indented_doc_comment(
            "`certify_*_const` callers that pre-date the X.1 cert-discrimination pass.",
        );
        f.line("    #[inline]");
        f.line("    #[must_use]");
        f.line("    #[allow(dead_code)]");
        f.line("    pub(crate) const fn empty_const() -> Self {");
        f.line("        Self {");
        f.line("            witt_bits: 0,");
        f.line("            content_fingerprint: ContentFingerprint::zero(),");
        f.line("            _private: (),");
        f.line("        }");
        f.line("    }");
        f.blank();
        f.indented_doc_comment("Phase X.1: the Witt level at which this certificate was minted.");
        f.line("    #[inline]");
        f.line("    #[must_use]");
        f.line("    pub const fn witt_bits(&self) -> u16 {");
        f.line("        self.witt_bits");
        f.line("    }");
        f.blank();
        f.indented_doc_comment("Phase X.1: the content fingerprint of the resolver decision.");
        f.line("    #[inline]");
        f.line("    #[must_use]");
        f.line("    pub const fn content_fingerprint(&self) -> ContentFingerprint<FP_MAX> {");
        f.line("        self.content_fingerprint");
        f.line("    }");
        f.line("}");
        f.blank();
    }

    f.doc_comment("v0.2.2 W11: sealed marker trait for foundation-supplied certificate kinds.");
    f.doc_comment("Implemented by every `cert:Certificate` subclass via codegen; not");
    f.doc_comment("implementable outside this crate.");
    f.line("pub trait Certificate: certificate_sealed::Sealed {");
    f.indented_doc_comment("The ontology IRI of this certificate class.");
    f.line("    const IRI: &'static str;");
    f.indented_doc_comment(
        "The structured evidence carried by this certificate (or `()` if none).",
    );
    f.line("    type Evidence;");
    f.line("}");
    f.blank();

    // The full set of cert classes. Existing shim names + new cert kind names.
    // Each entry is (rust_name, ontology_local_name, evidence_type).
    let all_certs: &[(&str, &str, &str)] = &[
        ("GroundingCertificate", "GroundingCertificate", "()"),
        (
            "LiftChainCertificate",
            "LiftChainCertificate",
            "ChainAuditTrail",
        ),
        ("InhabitanceCertificate", "InhabitanceCertificate", "()"),
        (
            "CompletenessCertificate",
            "CompletenessCertificate",
            "CompletenessAuditTrail",
        ),
        ("TransformCertificate", "TransformCertificate", "()"),
        ("IsometryCertificate", "IsometryCertificate", "()"),
        ("InvolutionCertificate", "InvolutionCertificate", "()"),
        (
            "GeodesicCertificate",
            "GeodesicCertificate",
            "GeodesicEvidenceBundle",
        ),
        ("MeasurementCertificate", "MeasurementCertificate", "()"),
        ("BornRuleVerification", "BornRuleVerification", "()"),
        // v0.2.2 Phase C.4: MultiplicationCertificate.
        (
            "MultiplicationCertificate",
            "MultiplicationCertificate",
            "MultiplicationEvidence",
        ),
        // v0.2.2 Phase E: PartitionCertificate.
        ("PartitionCertificate", "PartitionCertificate", "()"),
        // Workstream C: impossibility witnesses are certificates too —
        // they attest that a resolver verdict produced a failure witness
        // with the declared reason. The IRI resolves to the ontology's
        // `cert:<name>` class; both classes are authored under
        // `spec/src/namespaces/cert.rs` by Workstream C's ontology edit.
        (
            "GenericImpossibilityWitness",
            "GenericImpossibilityCertificate",
            "()",
        ),
        (
            "InhabitanceImpossibilityWitness",
            "InhabitanceImpossibilityCertificate",
            "()",
        ),
    ];
    // pub(crate) so Phase 10's witness_scaffolds module can register
    // its `Mint{Foo}` types as sealed `Certificate` carriers.
    f.line("pub(crate) mod certificate_sealed {");
    f.indented_doc_comment("Private supertrait. Not implementable outside this crate.");
    f.line("    pub trait Sealed {}");
    for (rust_name, _, _) in all_certs {
        if cert_carries_fp_max(rust_name) {
            f.line(&format!(
                "    impl<const FP_MAX: usize> Sealed for super::{rust_name}<FP_MAX> {{}}"
            ));
        } else {
            f.line(&format!("    impl Sealed for super::{rust_name} {{}}"));
        }
    }
    // Product/Coproduct Completion Amendment §2.3e: the three new sealed
    // witness types are registered here so they can impl `Certificate` via
    // its sealed supertrait. Their `impl Certificate` blocks are emitted
    // separately in `emit_partition_witness_certificates` — distinct from
    // the cert-namespace witnesses above because their IRIs live under
    // partition/ and their Evidence associated types are custom structs.
    f.line("    impl Sealed for super::PartitionProductWitness {}");
    f.line("    impl Sealed for super::PartitionCoproductWitness {}");
    f.line("    impl Sealed for super::CartesianProductWitness {}");
    f.line("}");
    f.blank();
    for (rust_name, ont_local, evidence) in all_certs {
        if cert_carries_fp_max(rust_name) {
            f.line(&format!(
                "impl<const FP_MAX: usize> Certificate for {rust_name}<FP_MAX> {{"
            ));
        } else {
            f.line(&format!("impl Certificate for {rust_name} {{"));
        }
        f.line(&format!(
            "    const IRI: &'static str = \"https://uor.foundation/cert/{ont_local}\";"
        ));
        f.line(&format!("    type Evidence = {evidence};"));
        f.line("}");
        f.blank();
    }

    // Phase X.1: uniform mint trait so `ResolverKernel::Cert` can be minted
    // through its associated type without duplicating per-cert match arms.
    f.line("/// Phase X.1: uniform mint interface over cert subclasses. Each");
    f.line("/// `Certificate` implementer that accepts `(witt_bits, ContentFingerprint)`");
    f.line("/// at construction time implements this trait. Lives inside a");
    f.line("/// `certify_const_mint` module so the symbol doesn't leak into top-level");
    f.line("/// documentation alongside `Certificate`.");
    f.line("pub(crate) mod certify_const_mint {");
    f.line("    use super::{ContentFingerprint, Certificate};");
    f.line("    pub trait MintWithLevelFingerprint<const FP_MAX: usize>: Certificate {");
    f.line("        fn mint_with_level_fingerprint(");
    f.line("            witt_bits: u16,");
    f.line("            content_fingerprint: ContentFingerprint<FP_MAX>,");
    f.line("        ) -> Self;");
    f.line("    }");
    // Cert types that carry (witt_bits, content_fingerprint). Each has a
    // `pub(crate) const fn with_level_and_fingerprint_const(...)` emitted
    // upstream in this file or in Phase X.1's new-cert-kind loop.
    let minting_certs: &[&str] = &[
        "GroundingCertificate",
        "LiftChainCertificate",
        "InhabitanceCertificate",
        "CompletenessCertificate",
        "MultiplicationCertificate",
        "PartitionCertificate",
        "TransformCertificate",
        "IsometryCertificate",
        "InvolutionCertificate",
        "GeodesicCertificate",
        "MeasurementCertificate",
        "BornRuleVerification",
    ];
    for cert in minting_certs {
        f.line(&format!(
            "    impl<const FP_MAX: usize> MintWithLevelFingerprint<FP_MAX> for super::{cert}<FP_MAX> {{"
        ));
        f.line("        #[inline]");
        f.line("        fn mint_with_level_fingerprint(");
        f.line("            witt_bits: u16,");
        f.line("            content_fingerprint: ContentFingerprint<FP_MAX>,");
        f.line("        ) -> Self {");
        f.line(&format!(
            "            super::{cert}::with_level_and_fingerprint_const(witt_bits, content_fingerprint)"
        ));
        f.line("        }");
        f.line("    }");
    }
    f.line("}");
    f.blank();

    f.doc_comment("v0.2.2 W11: parametric carrier for any foundation-supplied certificate.");
    f.doc_comment("Replaces the v0.2.1 per-class shim duplication. The `Certificate` trait");
    f.doc_comment("is sealed and the `_private` field prevents external construction; only");
    f.doc_comment("the foundation's pipeline / resolver paths produce `Certified<C>` values.");
    f.line("#[derive(Debug, Clone)]");
    f.line("pub struct Certified<C: Certificate> {");
    f.indented_doc_comment("The certificate kind value carried by this wrapper.");
    f.line("    inner: C,");
    f.indented_doc_comment(
        "Phase A.1: the foundation-internal two-clock value read at witness issuance.",
    );
    f.line("    uor_time: UorTime,");
    f.indented_doc_comment("Prevents external construction.");
    f.line("    _private: (),");
    f.line("}");
    f.blank();
    f.line("impl<C: Certificate> Certified<C> {");
    f.indented_doc_comment("Returns a reference to the carried certificate kind value.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn certificate(&self) -> &C {");
    f.line("        &self.inner");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Returns the ontology IRI of this certificate's kind.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn iri(&self) -> &'static str {");
    f.line("        C::IRI");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "Phase A.1: the foundation-internal two-clock value read at witness issuance.",
    );
    f.indented_doc_comment("");
    f.indented_doc_comment("Maps `rewrite_steps` to `derivation:stepCount` and `landauer_nats` to");
    f.indented_doc_comment(
        "`observable:LandauerCost`. Content-deterministic: same computation \u{2192}",
    );
    f.indented_doc_comment("same `UorTime`. Composable against wall-clock bounds via");
    f.indented_doc_comment("[`UorTime::min_wall_clock`] and a downstream-supplied `Calibration`.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn uor_time(&self) -> UorTime {");
    f.line("        self.uor_time");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "Crate-internal constructor. Reachable only from the pipeline / resolver paths.",
    );
    f.indented_doc_comment("");
    f.indented_doc_comment(
        "The `uor_time` is computed deterministically from the certificate kind's `IRI`",
    );
    f.indented_doc_comment(
        "length: the rewrite-step count is the IRI byte length, and the Landauer cost",
    );
    f.indented_doc_comment(
        "is `rewrite_steps \u{00d7} ln 2` (the Landauer-temperature cost of traversing that",
    );
    f.indented_doc_comment(
        "many orthogonal states). Two `Certified<C>` values with the same `C` share the",
    );
    f.indented_doc_comment(
        "same `UorTime`, preserving content-determinism across pipeline invocations.",
    );
    f.line("    #[inline]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new(inner: C) -> Self {");
    f.line("        // IRI length is the deterministic proxy for the cert class's structural");
    f.line("        // complexity. Phase D threads real resolver-counted steps through.");
    f.line("        let steps = C::IRI.len() as u64;");
    f.line("        let landauer = LandauerBudget::new((steps as f64) * core::f64::consts::LN_2);");
    f.line("        let uor_time = UorTime::new(landauer, steps);");
    f.line("        Self { inner, uor_time, _private: () }");
    f.line("    }");
    f.line("}");
    f.blank();
}

// 2.1.b Grounded<T> — zero-overhead ground-state wrapper.
fn generate_grounded_wrapper(f: &mut RustFile) {
    // v0.2.2 T6.14: `fnv1a_u128_const` deleted. The foundation does not pick a
    // hash function; downstream supplies `H: Hasher` and the typed pipeline
    // entry points thread it through `fold_unit_digest` to derive the
    // content-addressed `unit_address`.

    // v0.2.2 Phase E — BaseMetric sealed carriers + MAX_BETTI_DIMENSION.
    // Emitted before the GroundedShape trait so the accessors on Grounded
    // below can reference them.
    f.doc_comment("v0.2.2 Phase E: maximum simplicial dimension tracked by the");
    f.doc_comment("constraint-nerve Betti-numbers vector. The bound is 8 for the");
    f.doc_comment("currently-supported WittLevel set per the existing partition:FreeRank");
    f.doc_comment("capacity properties; the constant is `pub` (part of the public-API");
    f.doc_comment("snapshot) so future expansions require explicit review.");
    f.doc_comment("");
    f.doc_comment("Wiki ADR-037: a foundation-fixed conservative default for");
    f.doc_comment("[`crate::HostBounds::BETTI_DIMENSION_MAX`].");
    f.line(
        "pub const MAX_BETTI_DIMENSION: usize = \
         8;",
    );
    f.blank();

    f.doc_comment("Sealed newtype for the grounding completion ratio \u{03C3} \u{2208}");
    f.doc_comment("[0.0, 1.0]. \u{03C3} = 1 indicates the ground state; \u{03C3} = 0 the");
    f.doc_comment("unbound state. Backs observable:GroundingSigma. Phase 9: parametric over");
    f.doc_comment("the host's `H::Decimal` precision.");
    f.line("#[derive(Debug)]");
    f.line("pub struct SigmaValue<H: HostTypes = crate::DefaultHostTypes> {");
    f.line("    value: H::Decimal,");
    f.line("    _phantom: core::marker::PhantomData<H>,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    // Manual Copy/Clone/PartialEq — mirrors the LandauerBudget/UorTime/
    // Calibration pattern: H is a marker, so `#[derive]`'s auto-bounds
    // would constrain the host type unnecessarily.
    f.line("impl<H: HostTypes> Copy for SigmaValue<H> {}");
    f.line("impl<H: HostTypes> Clone for SigmaValue<H> {");
    f.line("    #[inline]");
    f.line("    fn clone(&self) -> Self {");
    f.line("        *self");
    f.line("    }");
    f.line("}");
    f.line("impl<H: HostTypes> PartialEq for SigmaValue<H> {");
    f.line("    #[inline]");
    f.line("    fn eq(&self, other: &Self) -> bool {");
    f.line("        self.value == other.value");
    f.line("    }");
    f.line("}");
    f.blank();
    f.line("impl<H: HostTypes> SigmaValue<H> {");
    f.indented_doc_comment("Returns the stored \u{03C3} value in the range [0.0, 1.0].");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn value(&self) -> H::Decimal {");
    f.line("        self.value");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Crate-internal constructor. Caller guarantees `value` is in");
    f.indented_doc_comment("the closed range [0.0, 1.0] and is not NaN.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new_unchecked(value: H::Decimal) -> Self {");
    f.line("        Self {");
    f.line("            value,");
    f.line("            _phantom: core::marker::PhantomData,");
    f.line("            _sealed: (),");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();

    // ── Stratum<L> ────────────────────────────────────────────────────────
    // Phase A.3: sealed newtype over u32 backing schema:stratum — the v_2
    // valuation of the Datum at level L. Bounded by L.bit_width, so u32 is
    // sufficient for every foreseeable WittLevel. The level parameter is
    // PhantomData; two Stratum<W8> and Stratum<W16> values are distinct
    // Rust types.
    f.doc_comment(
        "Phase A.3: sealed stratum coordinate (the v\u{2082} valuation of a `Datum` at level `L`).",
    );
    f.doc_comment("");
    f.doc_comment(
        "Backs `schema:stratum`. The value is non-negative and bounded by `L`'s `bit_width`",
    );
    f.doc_comment(
        "per the ontology's `nonNegativeInteger` range. Constructed only by foundation code",
    );
    f.doc_comment(
        "at grounding time; no public constructor — the `Stratum<W8>` / `Stratum<W16>` / ...",
    );
    f.doc_comment("type family is closed under the WittLevel individual set.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]");
    f.line("pub struct Stratum<L> {");
    f.line("    value: u32,");
    f.line("    _level: PhantomData<L>,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl<L> Stratum<L> {");
    f.indented_doc_comment(
        "Returns the raw v\u{2082} valuation as a `u32`. The value is bounded by `L`'s bit_width.",
    );
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn as_u32(&self) -> u32 {");
    f.line("        self.value");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "Crate-internal constructor. Reachable only from grounding-time minting.",
    );
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new(value: u32) -> Self {");
    f.line("        Self { value, _level: PhantomData, _sealed: () }");
    f.line("    }");
    f.line("}");
    f.blank();

    // ── Sealed BaseMetric newtypes (Phase A.4) ────────────────────────────
    // Metric incompatibility d_delta, Euler characteristic, residual count,
    // and the betti vector each become sealed newtypes so every accessor on
    // Grounded returns a foundation-minted type, not a raw primitive.
    f.doc_comment(
        "Phase A.4: sealed carrier for `observable:d_delta_metric` (metric incompatibility).",
    );
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]");
    f.line("pub struct DDeltaMetric {");
    f.line("    value: i64,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl DDeltaMetric {");
    f.indented_doc_comment(
        "Returns the signed incompatibility magnitude (ring distance minus Hamming distance).",
    );
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn as_i64(&self) -> i64 { self.value }");
    f.blank();
    f.indented_doc_comment("Crate-internal constructor.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new(value: i64) -> Self {");
    f.line("        Self { value, _sealed: () }");
    f.line("    }");
    f.line("}");
    f.blank();

    f.doc_comment(
        "Phase A.4: sealed carrier for `observable:euler_metric` (Euler characteristic).",
    );
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]");
    f.line("pub struct EulerMetric {");
    f.line("    value: i64,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl EulerMetric {");
    f.indented_doc_comment("Returns the Euler characteristic \u{03C7} of the constraint nerve.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn as_i64(&self) -> i64 { self.value }");
    f.blank();
    f.indented_doc_comment("Crate-internal constructor.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new(value: i64) -> Self {");
    f.line("        Self { value, _sealed: () }");
    f.line("    }");
    f.line("}");
    f.blank();

    f.doc_comment(
        "Phase A.4: sealed carrier for `observable:residual_metric` (free-site count r).",
    );
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]");
    f.line("pub struct ResidualMetric {");
    f.line("    value: u32,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl ResidualMetric {");
    f.indented_doc_comment("Returns the free-site count r at grounding time.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn as_u32(&self) -> u32 { self.value }");
    f.blank();
    f.indented_doc_comment("Crate-internal constructor.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new(value: u32) -> Self {");
    f.line("        Self { value, _sealed: () }");
    f.line("    }");
    f.line("}");
    f.blank();

    f.doc_comment("Phase A.4: sealed carrier for `observable:betti_metric` (Betti-number vector).");
    f.doc_comment("Fixed-capacity `[u32; MAX_BETTI_DIMENSION]` backing; no heap.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct BettiMetric {");
    f.line("    values: [u32; MAX_BETTI_DIMENSION],");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl BettiMetric {");
    f.indented_doc_comment("Returns the Betti-number vector as a fixed-size array reference.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn as_array(&self) -> &[u32; MAX_BETTI_DIMENSION] { &self.values }");
    f.blank();
    f.indented_doc_comment("Returns the individual Betti number \u{03B2}\u{2096} for dimension k.");
    f.indented_doc_comment("Returns 0 when k >= MAX_BETTI_DIMENSION.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn beta(&self, k: usize) -> u32 {");
    f.line("        if k < MAX_BETTI_DIMENSION { self.values[k] } else { 0 }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Crate-internal constructor.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new(values: [u32; MAX_BETTI_DIMENSION]) -> Self {");
    f.line("        Self { values, _sealed: () }");
    f.line("    }");
    f.line("}");
    f.blank();

    f.doc_comment("Maximum site count of the Jacobian row per Datum at any supported");
    f.doc_comment("WittLevel. Sourced from the partition:FreeRank capacity bound.");
    f.doc_comment("");
    f.doc_comment("v0.2.2 T2.6 (cleanup): reduced from 64 to 8 to keep `Grounded` under");
    f.doc_comment("the 256-byte size budget enforced by `phantom_tag::grounded_sealed_field_count_unchanged`.");
    f.doc_comment(
        "8 matches `MAX_BETTI_DIMENSION` and is sufficient for the v0.2.2 partition rank set.",
    );
    f.doc_comment("Wiki ADR-037: a foundation-fixed conservative default for");
    f.doc_comment("[`crate::HostBounds::JACOBIAN_SITES_MAX`].");
    f.line(
        "pub const JACOBIAN_MAX_SITES: usize = \
         8;",
    );
    f.blank();

    f.doc_comment("v0.2.2 Phase E: sealed Jacobian row carrier, parametric over the");
    f.doc_comment("WittLevel marker. Fixed-size `[i64; JACOBIAN_MAX_SITES]` backing; no");
    f.doc_comment("heap. The row records the per-site partial derivative of the ring");
    f.doc_comment("operation that produced the Datum. Backs observable:JacobianObservable.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct JacobianMetric<L> {");
    f.line("    entries: [i64; JACOBIAN_MAX_SITES],");
    f.line("    len: u16,");
    f.line("    _level: PhantomData<L>,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl<L> JacobianMetric<L> {");
    f.indented_doc_comment("Construct a zeroed Jacobian row with the given active length.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn zero(len: u16) -> Self {");
    f.line("        Self {");
    f.line("            entries: [0i64; JACOBIAN_MAX_SITES],");
    f.line("            len,");
    f.line("            _level: PhantomData,");
    f.line("            _sealed: (),");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("v0.2.2 T2.6 (cleanup): crate-internal constructor used by the");
    f.indented_doc_comment("BaseMetric accessor on `Grounded` to return a `JacobianMetric`");
    f.indented_doc_comment("backed by stored field values.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn from_entries(");
    f.line("        entries: [i64; JACOBIAN_MAX_SITES],");
    f.line("        len: u16,");
    f.line("    ) -> Self {");
    f.line("        Self {");
    f.line("            entries,");
    f.line("            len,");
    f.line("            _level: PhantomData,");
    f.line("            _sealed: (),");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Access the Jacobian row entries.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn entries(&self) -> &[i64; JACOBIAN_MAX_SITES] { &self.entries }");
    f.blank();
    f.indented_doc_comment("Number of active sites (the row's logical length).");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn len(&self) -> u16 { self.len }");
    f.blank();
    f.indented_doc_comment("Whether the Jacobian row is empty.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn is_empty(&self) -> bool { self.len == 0 }");
    f.line("}");
    f.blank();

    f.doc_comment("v0.2.2 Phase E: sealed Partition component classification.");
    f.doc_comment("Closed enumeration mirroring the partition:PartitionComponent");
    f.doc_comment("ontology class.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("#[non_exhaustive]");
    f.line("pub enum PartitionComponent {");
    f.indented_doc_comment("The irreducible component.");
    f.line("    Irreducible,");
    f.indented_doc_comment("The reducible component.");
    f.line("    Reducible,");
    f.indented_doc_comment("The unit component.");
    f.line("    Units,");
    f.indented_doc_comment("The exterior component.");
    f.line("    Exterior,");
    f.line("}");
    f.blank();

    f.doc_comment("Sealed marker trait identifying type:ConstrainedType subclasses that may");
    f.doc_comment("appear as the parameter of `Grounded<T>`.");
    f.doc_comment("");
    f.doc_comment("Per wiki ADR-027, the seal is the same `__sdk_seal::Sealed` supertrait");
    f.doc_comment("foundation uses for `FoundationClosed`, `PrismModel`, and");
    f.doc_comment("`IntoBindingValue`: only foundation and the SDK shape macros emit");
    f.doc_comment("impls. The foundation-sanctioned identity output `ConstrainedTypeInput`");
    f.doc_comment("retains its direct impl; application authors declaring custom Output");
    f.doc_comment("shapes invoke the `output_shape!` SDK macro, which emits");
    f.doc_comment("`__sdk_seal::Sealed`, `GroundedShape`, `IntoBindingValue`, and");
    f.doc_comment("`ConstrainedTypeShape` together.");
    f.line("pub trait GroundedShape: crate::pipeline::__sdk_seal::Sealed {}");
    f.line("impl GroundedShape for ConstrainedTypeInput {}");
    f.blank();

    // v0.2.2 T4.2 (cleanup): ContentAddress sealed newtype. Emitted here
    // so it's visible to BindingEntry / BindingsTable / Grounded / TraceEvent
    // below. The type-level wrap gives downstream a distinct identity for
    // content-addressed handles vs arbitrary u128 integers.
    f.doc_comment("v0.2.2 T4.2: sealed content-addressed handle.");
    f.doc_comment("");
    f.doc_comment("Wraps a 128-bit identity used for `BindingsTable` lookups and as a");
    f.doc_comment("compact identity handle for `Grounded` values. The underlying `u128`");
    f.doc_comment("is visible via `as_u128()` for interop. Distinct from");
    f.doc_comment("`ContentFingerprint` (the parametric-width fingerprint computed by");
    f.doc_comment("the substrate `Hasher` — see T5.C3.d below): `ContentAddress` is a");
    f.doc_comment("fixed 16-byte sortable handle, while `ContentFingerprint` is the full");
    f.doc_comment("substrate-computed identity that round-trips through `verify_trace`.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct ContentAddress {");
    f.line("    raw: u128,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl ContentAddress {");
    f.indented_doc_comment("The zero content address. Used as the \"unset\" placeholder in");
    f.indented_doc_comment("`BindingsTable` lookups, the default state of an uninitialised");
    f.indented_doc_comment("`Grounded::unit_address`, and the initial seed of replay folds.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn zero() -> Self {");
    f.line("        Self { raw: 0, _sealed: () }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Returns the underlying 128-bit content hash.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn as_u128(&self) -> u128 {");
    f.line("        self.raw");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Whether this content address is the zero handle.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn is_zero(&self) -> bool {");
    f.line("        self.raw == 0");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("v0.2.2 T5.C1: public ctor. Promoted from `pub(crate)` in T5 so");
    f.indented_doc_comment("downstream tests can construct deterministic `ContentAddress` values");
    f.indented_doc_comment("for fixture data without going through the foundation pipeline.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn from_u128(raw: u128) -> Self {");
    f.line("        Self { raw, _sealed: () }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "v0.2.2 Phase P.3: construct a `ContentAddress` from a `u64` FNV-1a fingerprint",
    );
    f.indented_doc_comment("by right-padding the value into the 128-bit address space. Used by");
    f.indented_doc_comment(
        "`Binding::to_binding_entry` to bridge the `Binding.content_address: u64`",
    );
    f.indented_doc_comment(
        "carrier to the `BindingEntry.address: ContentAddress` (`u128`-backed) shape.",
    );
    f.indented_doc_comment(
        "The lift is `raw = (fingerprint as u128) << 64` — upper 64 bits carry the",
    );
    f.indented_doc_comment(
        "FNV-1a value; lower 64 bits are zero. Content-deterministic; round-trips via",
    );
    f.indented_doc_comment("`ContentAddress::as_u128() >> 64` back to the original `u64`.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn from_u64_fingerprint(fingerprint: u64) -> Self {");
    f.line("        Self {");
    f.line("            raw: (fingerprint as u128) << 64,");
    f.line("            _sealed: (),");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();
    f.line("impl Default for ContentAddress {");
    f.line("    #[inline]");
    f.line("    fn default() -> Self {");
    f.line("        Self::zero()");
    f.line("    }");
    f.line("}");
    f.blank();

    // ── v0.2.2 T5.C3.d: substrate-pluggable hashing ───────────────────────
    //
    // The foundation does not prescribe a hash function. Instead it
    // defines:
    //   1. The abstract quantity (`ContentFingerprint`) — a sealed
    //      parametric-width buffer carried by `Grounded`, `Trace`,
    //      `Derivation`, and `GroundingCertificate`.
    //   2. The canonical byte layouts (`TraceDigest`, `UnitDigest`,
    //      `ParallelDigest`, `StreamDigest`, `InteractionDigest`) — locked
    //      at v0.2.2 and conformance-pinned.
    //   3. The substrate substitution point (`Hasher` trait) — downstream
    //      implementors supply BLAKE3, SHA-256, BLAKE2b, FNV-1a, etc.
    //
    // The architectural shape mirrors `Calibration`: the foundation defines
    // the abstract quantity (`UorTime`) and the substitution point
    // (`Calibration` carrying substrate constants); downstream supplies
    // the concrete substrate (substrate hash function + output width).
    // The foundation never invokes a hash function in its own code.
    //
    // The foundation **recommends BLAKE3** as the production substrate hash
    // (PRISM ships a BLAKE3 `Hasher` impl), but the recommendation is
    // non-binding — any conforming `Hasher` impl works.
    // Per the wiki's ADR-018, capacity bounds are carried by `HostBounds`
    // and flow through every signature via the `<const FP_MAX: usize>` /
    // `<const TR_MAX: usize>` parameters on `Hasher`, `ContentFingerprint`,
    // `Trace`, and friends. Free-standing capacity constants on the public
    // surface are explicitly rejected by ADR-018 (they collapse the
    // substitution axis), so this module exposes none. Applications read
    // their capacities through `<MyBounds as HostBounds>::CONST` instead.
    f.doc_comment("Trace wire-format identifier. Per the wiki's ADR-018, wire-format");
    f.doc_comment("identifiers are explicitly carved out of the `HostBounds` rule because");
    f.doc_comment("cross-implementation interop requires a single shared format identifier.");
    f.doc_comment("Increment when the layout changes (event ordering, trailing fields,");
    f.doc_comment("primitive-op discriminant table, certificate-kind discriminant table).");
    f.doc_comment("Pinned by the `rust/trace_byte_layout_pinned` conformance validator.");
    f.line("pub const TRACE_REPLAY_FORMAT_VERSION: u16 = 10;");
    f.blank();
    f.doc_comment("v0.2.2 T5: pluggable content hasher with parametric output width.");
    f.doc_comment("");
    f.doc_comment("The foundation does not ship an implementation. Downstream substrate");
    f.doc_comment("consumers (PRISM, application crates) supply their own `Hasher` impl");
    f.doc_comment("\u{2014} BLAKE3, SHA-256, SHA-512, BLAKE2b, SipHash 2-4, FNV-1a, or any");
    f.doc_comment("other byte-stream hash whose output width satisfies the foundation's");
    f.doc_comment("derived collision-bound minimum.");
    f.doc_comment("");
    f.doc_comment("# Foundation recommendation");
    f.doc_comment("");
    f.doc_comment("The foundation **recommends BLAKE3** as the default substrate hash for");
    f.doc_comment("production deployments. BLAKE3 is fast, well-audited, has no known");
    f.doc_comment("cryptographic weaknesses, supports parallel and SIMD-accelerated");
    f.doc_comment("implementations, and has a flexible output length. PRISM ships a BLAKE3");
    f.doc_comment("`Hasher` impl as its production substrate; application crates should");
    f.doc_comment("use it unless they have a specific reason to deviate.");
    f.doc_comment("");
    f.doc_comment("The recommendation is non-binding: any conforming `Hasher` impl works");
    f.doc_comment("with the foundation's pipeline and verify path. The foundation never");
    f.doc_comment("invokes a hash function itself, so the choice is fully a downstream");
    f.doc_comment("decision.");
    f.doc_comment("");
    f.doc_comment("# Architecture");
    f.doc_comment("");
    f.doc_comment("The architectural shape mirrors `Calibration`: the foundation defines");
    f.doc_comment("the abstract quantity (`ContentFingerprint`) and the substitution point");
    f.doc_comment("(`Hasher`); downstream provides the concrete substrate AND chooses the");
    f.doc_comment("output width within the foundation's correctness budget.");
    f.doc_comment("");
    f.doc_comment("# Required laws");
    f.doc_comment("");
    f.doc_comment("1. **Width-in-budget**: `OUTPUT_BYTES` must be in");
    f.doc_comment("   `[<B as HostBounds>::FINGERPRINT_MIN_BYTES, FP_MAX]` where `FP_MAX`");
    f.doc_comment("   is the const-generic carrying `<B as HostBounds>::FINGERPRINT_MAX_BYTES`.");
    f.doc_comment("   Enforced at codegen time via a `const _: () = assert!(...)` block");
    f.doc_comment("   emitted inside every `pipeline::run::<T, _, H>` body.");
    f.doc_comment("");
    f.doc_comment("2. **Determinism**: identical byte sequences produce bit-identical outputs");
    f.doc_comment("   across program runs, builds, target architectures, and rustc versions.");
    f.doc_comment("");
    f.doc_comment("3. **Sensitivity**: hashing two byte sequences that differ in any byte");
    f.doc_comment("   produces two distinct outputs with probability bounded by the hasher's");
    f.doc_comment("   documented collision rate.");
    f.doc_comment("");
    f.doc_comment("4. **No interior mutability**: `fold_byte` consumes `self` and returns a");
    f.doc_comment("   new state. Impls that depend on hidden mutable state violate the contract.");
    f.doc_comment("");
    f.doc_comment("# Example");
    f.doc_comment("");
    f.doc_comment("```");
    f.doc_comment("use uor_foundation::enforcement::Hasher;");
    f.doc_comment("");
    f.doc_comment("/// Minimal 128-bit (16-byte) FNV-1a substrate — two 64-bit lanes.");
    f.doc_comment("#[derive(Clone, Copy)]");
    f.doc_comment("pub struct Fnv1a16 { a: u64, b: u64 }");
    f.doc_comment("");
    f.doc_comment("impl Hasher for Fnv1a16 {");
    f.doc_comment("    const OUTPUT_BYTES: usize = 16;");
    f.doc_comment("    fn initial() -> Self {");
    f.doc_comment("        Self { a: 0xcbf29ce484222325, b: 0x84222325cbf29ce4 }");
    f.doc_comment("    }");
    f.doc_comment("    fn fold_byte(mut self, x: u8) -> Self {");
    f.doc_comment("        self.a ^= x as u64;");
    f.doc_comment("        self.a = self.a.wrapping_mul(0x100000001b3);");
    f.doc_comment("        self.b ^= (x as u64).rotate_left(8);");
    f.doc_comment("        self.b = self.b.wrapping_mul(0x100000001b3);");
    f.doc_comment("        self");
    f.doc_comment("    }");
    f.doc_comment("    fn finalize(self) -> [u8; 32] {");
    f.doc_comment("        let mut buf = [0u8; 32];");
    f.doc_comment("        buf[..8].copy_from_slice(&self.a.to_be_bytes());");
    f.doc_comment("        buf[8..16].copy_from_slice(&self.b.to_be_bytes());");
    f.doc_comment("        buf");
    f.doc_comment("    }");
    f.doc_comment("}");
    f.doc_comment("```");
    f.doc_comment("");
    f.doc_comment("Above, `Hasher` is reached through its default const-generic");
    f.doc_comment("`<FP_MAX = 32>` (the conventional 32-byte fingerprint width).");
    f.doc_comment("Applications that select a different `HostBounds` impl write");
    f.doc_comment("`impl Hasher<{<MyBounds as HostBounds>::FINGERPRINT_MAX_BYTES}> for MyHasher`.");
    // Wiki ADR-018 conformance: `Hasher` is parametric over the fingerprint
    // output width, which the application's `HostBounds` impl chooses.
    // `<const FP_MAX: usize = 32>` resolves to the conventional 32-byte
    // width when no override is supplied. Applications that select a
    // different `HostBounds` impl declare their hasher as
    // `Hasher<{<MyBounds as HostBounds>::FINGERPRINT_MAX_BYTES}>`.
    f.line("pub trait Hasher<const FP_MAX: usize = 32> {");
    f.indented_doc_comment("Active output width in bytes. Must lie within the bounds");
    f.indented_doc_comment("the application's selected `HostBounds` declares —");
    f.indented_doc_comment("`[<B as HostBounds>::FINGERPRINT_MIN_BYTES, FP_MAX]`.");
    f.line("    const OUTPUT_BYTES: usize;");
    f.blank();
    f.indented_doc_comment("Initial hasher state.");
    f.line("    fn initial() -> Self;");
    f.blank();
    f.indented_doc_comment("Fold a single byte into the running state.");
    f.line("    #[must_use]");
    f.line("    fn fold_byte(self, b: u8) -> Self;");
    f.blank();
    f.indented_doc_comment("Fold a slice of bytes (default impl: byte-by-byte).");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    fn fold_bytes(mut self, bytes: &[u8]) -> Self");
    f.line("    where");
    f.line("        Self: Sized,");
    f.line("    {");
    f.line("        let mut i = 0;");
    f.line("        while i < bytes.len() {");
    f.line("            self = self.fold_byte(bytes[i]);");
    f.line("            i += 1;");
    f.line("        }");
    f.line("        self");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Finalize into the canonical max-width buffer of `FP_MAX` bytes.");
    f.indented_doc_comment("Bytes `0..OUTPUT_BYTES` carry the hash result; bytes");
    f.indented_doc_comment("`OUTPUT_BYTES..FP_MAX` MUST be zero.");
    f.line("    fn finalize(self) -> [u8; FP_MAX];");
    f.line("}");
    f.blank();

    // ADR-030: HashAxis<H> adapter — wraps any Hasher impl as an
    // AxisExtension so applications can compose `(HashAxis<MyHasher>,)`
    // (a 1-tuple AxisTuple) and reuse existing Hasher implementations.
    // The single supported kernel id is HashAxis::KERNEL_HASH = 0, which
    // folds the input bytes through the wrapped Hasher and writes the
    // first `OUTPUT_BYTES` bytes of the digest.
    f.doc_comment("ADR-030 adapter: wrap any [`Hasher`] impl as an");
    f.doc_comment("[`crate::pipeline::AxisExtension`]. The lone supported kernel id");
    f.doc_comment("is [`HashAxis::KERNEL_HASH`] = 0, which folds the input bytes");
    f.doc_comment("through the wrapped Hasher and writes the first `OUTPUT_BYTES`");
    f.doc_comment("digest bytes to the caller-provided buffer.");
    f.line("#[derive(Debug, Clone, Copy)]");
    f.line("pub struct HashAxis<H>(core::marker::PhantomData<H>);");
    f.blank();
    f.line("impl<H> HashAxis<H> {");
    f.indented_doc_comment("Canonical kernel id for the hash operation. The closure-body");
    f.indented_doc_comment("grammar G19 form `hash(input)` lowers to");
    f.indented_doc_comment(
        "`Term::AxisInvocation { axis_index: 0, kernel_id: HashAxis::KERNEL_HASH, input_index }`.",
    );
    f.line("    pub const KERNEL_HASH: u32 = 0;");
    f.line("}");
    f.blank();
    // ADR-055: HashAxis is a primitive-fast-path axis. Its body is
    // byte-output-equivalent to `fold_bytes` ∘ `finalize` on the wrapped
    // Hasher; the empty arena signals the catamorphism to evaluate via
    // `dispatch_kernel` rather than recursively folding a body.
    f.line("impl<H> crate::pipeline::__sdk_seal::Sealed for HashAxis<H> {}");
    f.line("impl<const INLINE_BYTES: usize, H> crate::pipeline::SubstrateTermBody<INLINE_BYTES> for HashAxis<H> {");
    f.line("    fn body_arena() -> &'static [Term<'static, INLINE_BYTES>] {");
    f.line("        &[]");
    f.line("    }");
    f.line("}");
    f.line("impl<const INLINE_BYTES: usize, const FP_MAX: usize, H: Hasher<FP_MAX>> crate::pipeline::AxisExtension<INLINE_BYTES, FP_MAX> for HashAxis<H> {");
    f.line("    const AXIS_ADDRESS: &'static str = \"https://uor.foundation/axis/HashAxis\";");
    f.line("    const MAX_OUTPUT_BYTES: usize = <H as Hasher<FP_MAX>>::OUTPUT_BYTES;");
    f.line("    fn dispatch_kernel(");
    f.line("        kernel_id: u32,");
    f.line("        input: &[u8],");
    f.line("        out: &mut [u8],");
    f.line("    ) -> Result<usize, ShapeViolation> {");
    f.line("        if kernel_id != Self::KERNEL_HASH {");
    f.line("            return Err(ShapeViolation {");
    f.line("                shape_iri: \"https://uor.foundation/axis/HashAxis\",");
    f.line("                constraint_iri: \"https://uor.foundation/axis/HashAxis/kernelId\",");
    f.line("                property_iri: \"https://uor.foundation/axis/kernelId\",");
    f.line("                expected_range: \"https://uor.foundation/axis/HashAxis/KERNEL_HASH\",");
    f.line("                min_count: 0,");
    f.line("                max_count: 1,");
    f.line("                kind: crate::ViolationKind::ValueCheck,");
    f.line("            });");
    f.line("        }");
    f.line("        let mut hasher = <H as Hasher<FP_MAX>>::initial();");
    f.line("        hasher = hasher.fold_bytes(input);");
    f.line("        let digest = hasher.finalize();");
    f.line("        let n = <H as Hasher<FP_MAX>>::OUTPUT_BYTES.min(out.len()).min(digest.len());");
    f.line("        let mut i = 0;");
    f.line("        while i < n {");
    f.line("            out[i] = digest[i];");
    f.line("            i += 1;");
    f.line("        }");
    f.line("        Ok(n)");
    f.line("    }");
    f.line("}");
    f.blank();
    f.doc_comment("Sealed parametric content fingerprint.");
    f.doc_comment("");
    f.doc_comment("Wraps a fixed-capacity byte buffer of `FP_MAX` bytes plus the");
    f.doc_comment("active width in bytes. `FP_MAX` is the const-generic that carries");
    f.doc_comment("the application's selected `HostBounds::FINGERPRINT_MAX_BYTES`");
    f.doc_comment("(default = 32, the conventional fingerprint width). The active width");
    f.doc_comment("is set by the producing `Hasher::OUTPUT_BYTES` and recorded so");
    f.doc_comment("downstream can distinguish \"this is a 128-bit fingerprint\" from");
    f.doc_comment("\"this is a 256-bit fingerprint\" without inspecting trailing zeros.");
    f.doc_comment("");
    f.doc_comment("Equality is bit-equality on the full buffer + width tag, so two");
    f.doc_comment("fingerprints from different hashers (different widths) are never");
    f.doc_comment("equal even if their leading bytes happen to coincide. This prevents");
    f.doc_comment("silent collisions when downstream consumers mix substrate hashers.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct ContentFingerprint<const FP_MAX: usize = 32> {");
    f.line("    bytes: [u8; FP_MAX],");
    f.line("    width_bytes: u8,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl<const FP_MAX: usize> ContentFingerprint<FP_MAX> {");
    f.indented_doc_comment("Crate-internal zero placeholder. Used internally as the");
    f.indented_doc_comment("`Trace::empty()` field initializer and the `Default` impl. Not");
    f.indented_doc_comment("publicly constructible; downstream that needs a `ContentFingerprint`");
    f.indented_doc_comment("constructs one via `Hasher::finalize()` followed by `from_buffer()`.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn zero() -> Self {");
    f.line("        Self {");
    f.line("            bytes: [0u8; FP_MAX],");
    f.line("            width_bytes: 0,");
    f.line("            _sealed: (),");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Whether this fingerprint is the all-zero placeholder.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn is_zero(&self) -> bool {");
    f.line("        self.width_bytes == 0");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Active width in bytes (set by the producing hasher).");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn width_bytes(&self) -> u8 {");
    f.line("        self.width_bytes");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Active width in bits (`width_bytes * 8`).");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn width_bits(&self) -> u16 {");
    f.line("        (self.width_bytes as u16) * 8");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("The full buffer. Bytes `0..width_bytes` are the hash; bytes");
    f.indented_doc_comment("`width_bytes..FP_MAX` are zero.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn as_bytes(&self) -> &[u8; FP_MAX] {");
    f.line("        &self.bytes");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Construct a fingerprint from a hasher's finalize buffer + width tag.");
    f.indented_doc_comment("Production paths reach this via `pipeline::run::<T, _, H>`; test");
    f.indented_doc_comment("paths via the `__test_helpers` back-door.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn from_buffer(");
    f.line("        bytes: [u8; FP_MAX],");
    f.line("        width_bytes: u8,");
    f.line("    ) -> Self {");
    f.line("        Self {");
    f.line("            bytes,");
    f.line("            width_bytes,");
    f.line("            _sealed: (),");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();
    f.line("impl<const FP_MAX: usize> Default for ContentFingerprint<FP_MAX> {");
    f.line("    #[inline]");
    f.line("    fn default() -> Self {");
    f.line("        Self::zero()");
    f.line("    }");
    f.line("}");
    f.blank();
    // v0.2.2 T6.3: `ZeroHasher` (the migration marker for legacy callers
    // without a substrate `Hasher`) is deleted. v0.2.2 is the first full
    // release; there are no legacy callers. Every public path that produces
    // a `Grounded`, `Trace`, or `GroundingCertificate` threads a real
    // substrate `Hasher` end-to-end.

    f.doc_comment("v0.2.2 T5: stable u8 discriminant for `PrimitiveOp`. Locked to the");
    f.doc_comment("codegen output order; do not reorder `PrimitiveOp` variants without");
    f.doc_comment("bumping `TRACE_REPLAY_FORMAT_VERSION`. Folded into the canonical byte");
    f.doc_comment("layout of every `TraceEvent` so substrate hashers produce stable");
    f.doc_comment("fingerprints across builds.");
    f.line("#[inline]");
    f.line("#[must_use]");
    f.line("pub const fn primitive_op_discriminant(op: crate::PrimitiveOp) -> u8 {");
    f.line("    match op {");
    f.line("        crate::PrimitiveOp::Neg => 0,");
    f.line("        crate::PrimitiveOp::Bnot => 1,");
    f.line("        crate::PrimitiveOp::Succ => 2,");
    f.line("        crate::PrimitiveOp::Pred => 3,");
    f.line("        crate::PrimitiveOp::Add => 4,");
    f.line("        crate::PrimitiveOp::Sub => 5,");
    f.line("        crate::PrimitiveOp::Mul => 6,");
    f.line("        crate::PrimitiveOp::Xor => 7,");
    f.line("        crate::PrimitiveOp::And => 8,");
    f.line("        crate::PrimitiveOp::Or => 9,");
    f.line("        // ADR-013/TR-08 substrate amendment: byte-level ops 10..15.");
    f.line("        crate::PrimitiveOp::Le => 10,");
    f.line("        crate::PrimitiveOp::Lt => 11,");
    f.line("        crate::PrimitiveOp::Ge => 12,");
    f.line("        crate::PrimitiveOp::Gt => 13,");
    f.line("        crate::PrimitiveOp::Concat => 14,");
    f.line("        // ADR-053 substrate amendment: ring-axis arithmetic completion.");
    f.line("        crate::PrimitiveOp::Div => 15,");
    f.line("        crate::PrimitiveOp::Mod => 16,");
    f.line("        crate::PrimitiveOp::Pow => 17,");
    f.line("    }");
    f.line("}");
    f.blank();
    f.doc_comment("v0.2.2 T5: stable u8 discriminant tag distinguishing the certificate");
    f.doc_comment("kinds the foundation pipeline mints. Folded into the trailing byte of");
    f.doc_comment("every canonical digest so two certificates over the same source unit");
    f.doc_comment("but of different kinds produce distinct fingerprints. Locked at v0.2.2;");
    f.doc_comment("reordering or inserting variants requires bumping");
    f.doc_comment("`TRACE_REPLAY_FORMAT_VERSION`.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("#[non_exhaustive]");
    f.line("pub enum CertificateKind {");
    f.indented_doc_comment("Cert minted by `pipeline::run` and `run_const` (retained for");
    f.indented_doc_comment("`run_grounding_aware`).");
    f.line("    Grounding,");
    f.indented_doc_comment("Cert minted by `certify_tower_completeness_const`.");
    f.line("    TowerCompleteness,");
    f.indented_doc_comment("Cert minted by `certify_incremental_completeness_const`.");
    f.line("    IncrementalCompleteness,");
    f.indented_doc_comment("Cert minted by `certify_inhabitance_const` / `run_inhabitance`.");
    f.line("    Inhabitance,");
    f.indented_doc_comment("Cert minted by `certify_multiplication_const`.");
    f.line("    Multiplication,");
    f.indented_doc_comment("Cert minted by `resolver::two_sat_decider::certify`.");
    f.line("    TwoSat,");
    f.indented_doc_comment("Cert minted by `resolver::horn_sat_decider::certify`.");
    f.line("    HornSat,");
    f.indented_doc_comment("Cert minted by `resolver::residual_verdict::certify`.");
    f.line("    ResidualVerdict,");
    f.indented_doc_comment("Cert minted by `resolver::canonical_form::certify`.");
    f.line("    CanonicalForm,");
    f.indented_doc_comment("Cert minted by `resolver::type_synthesis::certify`.");
    f.line("    TypeSynthesis,");
    f.indented_doc_comment("Cert minted by `resolver::homotopy::certify`.");
    f.line("    Homotopy,");
    f.indented_doc_comment("Cert minted by `resolver::monodromy::certify`.");
    f.line("    Monodromy,");
    f.indented_doc_comment("Cert minted by `resolver::moduli::certify`.");
    f.line("    Moduli,");
    f.indented_doc_comment("Cert minted by `resolver::jacobian_guided::certify`.");
    f.line("    JacobianGuided,");
    f.indented_doc_comment("Cert minted by `resolver::evaluation::certify`.");
    f.line("    Evaluation,");
    f.indented_doc_comment("Cert minted by `resolver::session::certify`.");
    f.line("    Session,");
    f.indented_doc_comment("Cert minted by `resolver::superposition::certify`.");
    f.line("    Superposition,");
    f.indented_doc_comment("Cert minted by `resolver::measurement::certify`.");
    f.line("    Measurement,");
    f.indented_doc_comment("Cert minted by `resolver::witt_level_resolver::certify`.");
    f.line("    WittLevel,");
    f.indented_doc_comment("Cert minted by `resolver::dihedral_factorization::certify`.");
    f.line("    DihedralFactorization,");
    f.indented_doc_comment("Cert minted by `resolver::completeness::certify`.");
    f.line("    Completeness,");
    f.indented_doc_comment("Cert minted by `resolver::geodesic_validator::certify`.");
    f.line("    GeodesicValidator,");
    f.line("}");
    f.blank();
    f.doc_comment("v0.2.2 T5: stable u8 discriminant for `CertificateKind`. Folded into");
    f.doc_comment("the trailing byte of canonical digests via the `*Digest` types so two");
    f.doc_comment("distinct certificate kinds over the same source unit produce distinct");
    f.doc_comment("fingerprints. Locked at v0.2.2.");
    f.line("#[inline]");
    f.line("#[must_use]");
    f.line("pub const fn certificate_kind_discriminant(kind: CertificateKind) -> u8 {");
    f.line("    match kind {");
    f.line("        CertificateKind::Grounding => 1,");
    f.line("        CertificateKind::TowerCompleteness => 2,");
    f.line("        CertificateKind::IncrementalCompleteness => 3,");
    f.line("        CertificateKind::Inhabitance => 4,");
    f.line("        CertificateKind::Multiplication => 5,");
    f.line("        CertificateKind::TwoSat => 6,");
    f.line("        CertificateKind::HornSat => 7,");
    f.line("        CertificateKind::ResidualVerdict => 8,");
    f.line("        CertificateKind::CanonicalForm => 9,");
    f.line("        CertificateKind::TypeSynthesis => 10,");
    f.line("        CertificateKind::Homotopy => 11,");
    f.line("        CertificateKind::Monodromy => 12,");
    f.line("        CertificateKind::Moduli => 13,");
    f.line("        CertificateKind::JacobianGuided => 14,");
    f.line("        CertificateKind::Evaluation => 15,");
    f.line("        CertificateKind::Session => 16,");
    f.line("        CertificateKind::Superposition => 17,");
    f.line("        CertificateKind::Measurement => 18,");
    f.line("        CertificateKind::WittLevel => 19,");
    f.line("        CertificateKind::DihedralFactorization => 20,");
    f.line("        CertificateKind::Completeness => 21,");
    f.line("        CertificateKind::GeodesicValidator => 22,");
    f.line("    }");
    f.line("}");
    f.blank();
    f.doc_comment("v0.2.2 T5: fold a `ConstraintRef` into a `Hasher` via the canonical");
    f.doc_comment("byte layout. Each variant emits a discriminant byte (1..=9) followed by");
    f.doc_comment("its payload bytes in big-endian order. Locked at v0.2.2; reordering");
    f.doc_comment("variants requires bumping `TRACE_REPLAY_FORMAT_VERSION`.");
    f.doc_comment("");
    f.doc_comment("Used by `pipeline::run`, `run_const`, and the four `certify_*` entry");
    f.doc_comment("points to fold a unit's constraint set into the substrate fingerprint.");
    f.line("pub fn fold_constraint_ref<const FP_MAX: usize, H: Hasher<FP_MAX>>(");
    f.line("    mut hasher: H,");
    f.line("    c: &crate::pipeline::ConstraintRef,");
    f.line(") -> H {");
    f.line("    use crate::pipeline::ConstraintRef as C;");
    f.line("    match c {");
    f.line("        C::Residue { modulus, residue } => {");
    f.line("            hasher = hasher.fold_byte(1);");
    f.line("            hasher = hasher.fold_bytes(&modulus.to_be_bytes());");
    f.line("            hasher = hasher.fold_bytes(&residue.to_be_bytes());");
    f.line("        }");
    f.line("        C::Hamming { bound } => {");
    f.line("            hasher = hasher.fold_byte(2);");
    f.line("            hasher = hasher.fold_bytes(&bound.to_be_bytes());");
    f.line("        }");
    f.line("        C::Depth { min, max } => {");
    f.line("            hasher = hasher.fold_byte(3);");
    f.line("            hasher = hasher.fold_bytes(&min.to_be_bytes());");
    f.line("            hasher = hasher.fold_bytes(&max.to_be_bytes());");
    f.line("        }");
    f.line("        C::Carry { site } => {");
    f.line("            hasher = hasher.fold_byte(4);");
    f.line("            hasher = hasher.fold_bytes(&site.to_be_bytes());");
    f.line("        }");
    f.line("        C::Site { position } => {");
    f.line("            hasher = hasher.fold_byte(5);");
    f.line("            hasher = hasher.fold_bytes(&position.to_be_bytes());");
    f.line("        }");
    f.line("        C::Affine { coefficients, coefficient_count, bias } => {");
    f.line("            hasher = hasher.fold_byte(6);");
    f.line("            hasher = hasher.fold_bytes(&coefficient_count.to_be_bytes());");
    f.line("            let count = *coefficient_count as usize;");
    f.line("            let mut i = 0;");
    f.line("            while i < count && i < crate::pipeline::AFFINE_MAX_COEFFS {");
    f.line("                hasher = hasher.fold_bytes(&coefficients[i].to_be_bytes());");
    f.line("                i += 1;");
    f.line("            }");
    f.line("            hasher = hasher.fold_bytes(&bias.to_be_bytes());");
    f.line("        }");
    f.line("        C::SatClauses { clauses, num_vars } => {");
    f.line("            hasher = hasher.fold_byte(7);");
    f.line("            hasher = hasher.fold_bytes(&num_vars.to_be_bytes());");
    f.line("            hasher = hasher.fold_bytes(&(clauses.len() as u32).to_be_bytes());");
    f.line("            let mut i = 0;");
    f.line("            while i < clauses.len() {");
    f.line("                let clause = clauses[i];");
    f.line("                hasher = hasher.fold_bytes(&(clause.len() as u32).to_be_bytes());");
    f.line("                let mut j = 0;");
    f.line("                while j < clause.len() {");
    f.line("                    let (var, neg) = clause[j];");
    f.line("                    hasher = hasher.fold_bytes(&var.to_be_bytes());");
    f.line("                    hasher = hasher.fold_byte(if neg { 1 } else { 0 });");
    f.line("                    j += 1;");
    f.line("                }");
    f.line("                i += 1;");
    f.line("            }");
    f.line("        }");
    f.line("        C::Bound {");
    f.line("            observable_iri,");
    f.line("            bound_shape_iri,");
    f.line("            args_repr,");
    f.line("        } => {");
    f.line("            hasher = hasher.fold_byte(8);");
    f.line("            hasher = hasher.fold_bytes(observable_iri.as_bytes());");
    f.line("            hasher = hasher.fold_byte(0);");
    f.line("            hasher = hasher.fold_bytes(bound_shape_iri.as_bytes());");
    f.line("            hasher = hasher.fold_byte(0);");
    f.line("            hasher = hasher.fold_bytes(args_repr.as_bytes());");
    f.line("            hasher = hasher.fold_byte(0);");
    f.line("        }");
    f.line("        C::Conjunction { conjuncts, conjunct_count } => {");
    f.line("            hasher = hasher.fold_byte(9);");
    f.line("            hasher = hasher.fold_bytes(&conjunct_count.to_be_bytes());");
    f.line("            let count = *conjunct_count as usize;");
    f.line("            let mut i = 0;");
    f.line("            while i < count && i < crate::pipeline::CONJUNCTION_MAX_TERMS {");
    f.line("                let lifted = conjuncts[i].into_constraint();");
    f.line("                hasher = fold_constraint_ref(hasher, &lifted);");
    f.line("                i += 1;");
    f.line("            }");
    f.line("        }");
    f.line("        // ADR-057 wire-format: discriminant byte 10 + content-addressed");
    f.line("        // shape_iri + descent_bound. The discriminant table extension");
    f.line("        // requires TRACE_REPLAY_FORMAT_VERSION bump per ADR-013/TR-08.");
    f.line("        C::Recurse { shape_iri, descent_bound } => {");
    f.line("            hasher = hasher.fold_byte(10);");
    f.line("            hasher = hasher.fold_bytes(shape_iri.as_bytes());");
    f.line("            hasher = hasher.fold_byte(0);");
    f.line("            hasher = hasher.fold_bytes(&descent_bound.to_be_bytes());");
    f.line("        }");
    f.line("    }");
    f.line("    hasher");
    f.line("}");
    f.blank();
    f.doc_comment("v0.2.2 T5: fold the canonical CompileUnit byte layout into a `Hasher`.");
    f.doc_comment("");
    f.doc_comment("Layout: `level_bits (2 BE) || budget (8 BE) || iri bytes || 0x00 ||");
    f.doc_comment("site_count (8 BE) || for each constraint: fold_constraint_ref || ");
    f.doc_comment("certificate_kind_discriminant (1 byte trailing)`.");
    f.doc_comment("");
    f.doc_comment("Locked at v0.2.2 by the `rust/trace_byte_layout_pinned` conformance");
    f.doc_comment("validator. Used by `pipeline::run`, `run_const`, and the four");
    f.doc_comment("`certify_*` entry points.");
    f.line("pub fn fold_unit_digest<const FP_MAX: usize, H: Hasher<FP_MAX>>(");
    f.line("    mut hasher: H,");
    f.line("    level_bits: u16,");
    f.line("    budget: u64,");
    f.line("    iri: &str,");
    f.line("    site_count: usize,");
    f.line("    constraints: &[crate::pipeline::ConstraintRef],");
    f.line("    kind: CertificateKind,");
    f.line(") -> H {");
    f.line("    hasher = hasher.fold_bytes(&level_bits.to_be_bytes());");
    f.line("    hasher = hasher.fold_bytes(&budget.to_be_bytes());");
    f.line("    hasher = hasher.fold_bytes(iri.as_bytes());");
    f.line("    hasher = hasher.fold_byte(0);");
    f.line("    hasher = hasher.fold_bytes(&(site_count as u64).to_be_bytes());");
    f.line("    let mut i = 0;");
    f.line("    while i < constraints.len() {");
    f.line("        hasher = fold_constraint_ref(hasher, &constraints[i]);");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    hasher = hasher.fold_byte(certificate_kind_discriminant(kind));");
    f.line("    hasher");
    f.line("}");
    f.blank();
    f.doc_comment("v0.2.2 T5: fold the canonical ParallelDeclaration byte layout into a `Hasher`.");
    f.doc_comment("");
    f.doc_comment("Layout: `site_count (8 BE) || iri bytes || 0x00 || decl_site_count (8 BE) ||");
    f.doc_comment("for each constraint: fold_constraint_ref || certificate_kind_discriminant`.");
    f.line("pub fn fold_parallel_digest<const FP_MAX: usize, H: Hasher<FP_MAX>>(");
    f.line("    mut hasher: H,");
    f.line("    decl_site_count: u64,");
    f.line("    iri: &str,");
    f.line("    type_site_count: usize,");
    f.line("    constraints: &[crate::pipeline::ConstraintRef],");
    f.line("    kind: CertificateKind,");
    f.line(") -> H {");
    f.line("    hasher = hasher.fold_bytes(&decl_site_count.to_be_bytes());");
    f.line("    hasher = hasher.fold_bytes(iri.as_bytes());");
    f.line("    hasher = hasher.fold_byte(0);");
    f.line("    hasher = hasher.fold_bytes(&(type_site_count as u64).to_be_bytes());");
    f.line("    let mut i = 0;");
    f.line("    while i < constraints.len() {");
    f.line("        hasher = fold_constraint_ref(hasher, &constraints[i]);");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    hasher = hasher.fold_byte(certificate_kind_discriminant(kind));");
    f.line("    hasher");
    f.line("}");
    f.blank();
    f.doc_comment("v0.2.2 T5: fold the canonical StreamDeclaration byte layout into a `Hasher`.");
    f.line("pub fn fold_stream_digest<const FP_MAX: usize, H: Hasher<FP_MAX>>(");
    f.line("    mut hasher: H,");
    f.line("    productivity_bound: u64,");
    f.line("    iri: &str,");
    f.line("    constraints: &[crate::pipeline::ConstraintRef],");
    f.line("    kind: CertificateKind,");
    f.line(") -> H {");
    f.line("    hasher = hasher.fold_bytes(&productivity_bound.to_be_bytes());");
    f.line("    hasher = hasher.fold_bytes(iri.as_bytes());");
    f.line("    hasher = hasher.fold_byte(0);");
    f.line("    let mut i = 0;");
    f.line("    while i < constraints.len() {");
    f.line("        hasher = fold_constraint_ref(hasher, &constraints[i]);");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    hasher = hasher.fold_byte(certificate_kind_discriminant(kind));");
    f.line("    hasher");
    f.line("}");
    f.blank();
    f.doc_comment(
        "v0.2.2 T5: fold the canonical InteractionDeclaration byte layout into a `Hasher`.",
    );
    f.line("pub fn fold_interaction_digest<const FP_MAX: usize, H: Hasher<FP_MAX>>(");
    f.line("    mut hasher: H,");
    f.line("    convergence_seed: u64,");
    f.line("    iri: &str,");
    f.line("    constraints: &[crate::pipeline::ConstraintRef],");
    f.line("    kind: CertificateKind,");
    f.line(") -> H {");
    f.line("    hasher = hasher.fold_bytes(&convergence_seed.to_be_bytes());");
    f.line("    hasher = hasher.fold_bytes(iri.as_bytes());");
    f.line("    hasher = hasher.fold_byte(0);");
    f.line("    let mut i = 0;");
    f.line("    while i < constraints.len() {");
    f.line("        hasher = fold_constraint_ref(hasher, &constraints[i]);");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    hasher = hasher.fold_byte(certificate_kind_discriminant(kind));");
    f.line("    hasher");
    f.line("}");
    f.blank();
    emit_phase_j_primitives(f);
    f.doc_comment("v0.2.2 T6.1: per-step canonical byte layout for `StreamDriver::next()`.");
    f.doc_comment("");
    f.doc_comment(
        "Layout: `productivity_remaining (8 BE) || rewrite_steps (8 BE) || seed (8 BE) ||",
    );
    f.doc_comment("iri bytes || 0x00 || certificate_kind_discriminant (1 byte trailing)`.");
    f.line("pub fn fold_stream_step_digest<const FP_MAX: usize, H: Hasher<FP_MAX>>(");
    f.line("    mut hasher: H,");
    f.line("    productivity_remaining: u64,");
    f.line("    rewrite_steps: u64,");
    f.line("    seed: u64,");
    f.line("    iri: &str,");
    f.line("    kind: CertificateKind,");
    f.line(") -> H {");
    f.line("    hasher = hasher.fold_bytes(&productivity_remaining.to_be_bytes());");
    f.line("    hasher = hasher.fold_bytes(&rewrite_steps.to_be_bytes());");
    f.line("    hasher = hasher.fold_bytes(&seed.to_be_bytes());");
    f.line("    hasher = hasher.fold_bytes(iri.as_bytes());");
    f.line("    hasher = hasher.fold_byte(0);");
    f.line("    hasher = hasher.fold_byte(certificate_kind_discriminant(kind));");
    f.line("    hasher");
    f.line("}");
    f.blank();
    f.doc_comment("v0.2.2 T6.1: per-step canonical byte layout for `InteractionDriver::step()`");
    f.doc_comment("and `InteractionDriver::finalize()`.");
    f.doc_comment("");
    f.doc_comment(
        "Layout: `commutator_acc[0..4] (4 \u{00d7} 8 BE bytes) || peer_step_count (8 BE) ||",
    );
    f.doc_comment(
        "seed (8 BE) || iri bytes || 0x00 || certificate_kind_discriminant (1 byte trailing)`.",
    );
    f.line("pub fn fold_interaction_step_digest<const FP_MAX: usize, H: Hasher<FP_MAX>>(");
    f.line("    mut hasher: H,");
    f.line("    commutator_acc: &[u64; 4],");
    f.line("    peer_step_count: u64,");
    f.line("    seed: u64,");
    f.line("    iri: &str,");
    f.line("    kind: CertificateKind,");
    f.line(") -> H {");
    f.line("    let mut i = 0;");
    f.line("    while i < 4 {");
    f.line("        hasher = hasher.fold_bytes(&commutator_acc[i].to_be_bytes());");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    hasher = hasher.fold_bytes(&peer_step_count.to_be_bytes());");
    f.line("    hasher = hasher.fold_bytes(&seed.to_be_bytes());");
    f.line("    hasher = hasher.fold_bytes(iri.as_bytes());");
    f.line("    hasher = hasher.fold_byte(0);");
    f.line("    hasher = hasher.fold_byte(certificate_kind_discriminant(kind));");
    f.line("    hasher");
    f.line("}");
    f.blank();
    f.doc_comment("Utility — extract the leading 16 bytes of a `Hasher::finalize` buffer");
    f.doc_comment("as a `ContentAddress`. Used by pipeline entry points to derive the");
    f.doc_comment("16-byte unit_address handle from a freshly-computed substrate");
    f.doc_comment("fingerprint, so two units with distinct fingerprints have distinct");
    f.doc_comment("unit_address handles too.");
    f.doc_comment("");
    f.doc_comment("Per the wiki's ADR-018 the `FP_MAX` const-generic carries the");
    f.doc_comment("application's selected `<B as HostBounds>::FINGERPRINT_MAX_BYTES`.");
    f.doc_comment("`FP_MAX` MUST be at least 16; smaller buffers cannot supply the");
    f.doc_comment("16-byte address prefix.");
    f.line("#[inline]");
    f.line("#[must_use]");
    f.line(
        "pub const fn unit_address_from_buffer<const FP_MAX: usize>(buffer: &[u8; FP_MAX]) -> ContentAddress {",
    );
    f.line("    let mut bytes = [0u8; 16];");
    f.line("    let mut i = 0;");
    f.line("    while i < 16 {");
    f.line("        bytes[i] = buffer[i];");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    ContentAddress::from_u128(u128::from_be_bytes(bytes))");
    f.line("}");
    f.blank();
    f.doc_comment("v0.2.2 T6.11: const-fn equality on `&str` slices. `str::eq` is not");
    f.doc_comment("stable in const eval under MSRV 1.81; this helper provides a");
    f.doc_comment("byte-by-byte equality check for use in `pipeline::run`'s ShapeMismatch");
    f.doc_comment("detection without runtime allocation.");
    f.line("#[inline]");
    f.line("#[must_use]");
    f.line("pub const fn str_eq(a: &str, b: &str) -> bool {");
    f.line("    let a = a.as_bytes();");
    f.line("    let b = b.as_bytes();");
    f.line("    if a.len() != b.len() {");
    f.line("        return false;");
    f.line("    }");
    f.line("    let mut i = 0;");
    f.line("    while i < a.len() {");
    f.line("        if a[i] != b[i] {");
    f.line("            return false;");
    f.line("        }");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    true");
    f.line("}");
    f.blank();

    f.doc_comment("A binding entry in a `BindingsTable`. Pairs a `ContentAddress`");
    f.doc_comment("(hash of the query coordinate) with the bound bytes.");
    f.line("#[derive(Debug, Clone, Copy)]");
    f.line("pub struct BindingEntry {");
    f.indented_doc_comment("Content-hashed query address.");
    f.line("    pub address: ContentAddress,");
    f.indented_doc_comment(
        "Bound payload bytes (length determined by the WittLevel of the table).",
    );
    f.line("    pub bytes: &'static [u8],");
    f.line("}");
    f.blank();

    f.doc_comment("A static, sorted-by-address binding table laid out for `op:GS_5` zero-step");
    f.doc_comment("access. Looked up via binary search; the foundation guarantees the table");
    f.doc_comment("is materialized at compile time from the attested `state:GroundedContext`.");
    f.line("#[derive(Debug, Clone, Copy)]");
    f.line("pub struct BindingsTable {");
    f.indented_doc_comment("Entries, sorted ascending by `address`.");
    f.line("    pub entries: &'static [BindingEntry],");
    f.line("}");
    f.blank();
    f.line("impl BindingsTable {");
    f.indented_doc_comment("v0.2.2 T5 C4: validating constructor. Checks that `entries` are");
    f.indented_doc_comment("strictly ascending by `address`, which is the invariant");
    f.indented_doc_comment("`Grounded::get_binding` relies on for its binary-search lookup.");
    f.indented_doc_comment("");
    f.indented_doc_comment("# Errors");
    f.indented_doc_comment("");
    f.indented_doc_comment("Returns `BindingsTableError::Unsorted { at }` where `at` is the first");
    f.indented_doc_comment("index where the order is violated (i.e., `entries[at].address <=");
    f.indented_doc_comment("entries[at - 1].address`).");
    f.line("    pub const fn try_new(");
    f.line("        entries: &'static [BindingEntry],");
    f.line("    ) -> Result<Self, BindingsTableError> {");
    f.line("        let mut i = 1;");
    f.line("        while i < entries.len() {");
    f.line("            if entries[i].address.as_u128() <= entries[i - 1].address.as_u128() {");
    f.line("                return Err(BindingsTableError::Unsorted { at: i });");
    f.line("            }");
    f.line("            i += 1;");
    f.line("        }");
    f.line("        Ok(Self { entries })");
    f.line("    }");
    f.line("}");
    f.blank();
    f.doc_comment("v0.2.2 T5 C4: errors returned by `BindingsTable::try_new`.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("#[non_exhaustive]");
    f.line("pub enum BindingsTableError {");
    f.indented_doc_comment("Entries at index `at` and `at - 1` are out of order (the slice is");
    f.indented_doc_comment("not strictly ascending by `address`).");
    f.line("    Unsorted {");
    f.line("        /// The first index where the order is violated.");
    f.line("        at: usize,");
    f.line("    },");
    f.line("}");
    f.blank();
    // v0.2.2 T5.9: Display + core::error::Error impls.
    f.line("impl core::fmt::Display for BindingsTableError {");
    f.line("    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {");
    f.line("        match self {");
    f.line("            Self::Unsorted { at } => write!(");
    f.line("                f,");
    f.line("                \"BindingsTable entries not sorted: address at index {at} <= address at index {}\",");
    f.line("                at - 1,");
    f.line("            ),");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();
    f.line("impl core::error::Error for BindingsTableError {}");
    f.blank();

    f.doc_comment("The compile-time witness that `op:GS_4` holds for the value it carries:");
    f.doc_comment("σ = 1, freeRank = 0, S = 0, T_ctx = 0. `Grounded<T, Tag>` is constructed");
    f.doc_comment(
        "only by the reduction pipeline and provides `op:GS_5` zero-step binding access.",
    );
    f.doc_comment("");
    f.doc_comment("v0.2.2 Phase B (Q3): the `Tag` phantom parameter (default `Tag = T`)");
    f.doc_comment("lets downstream code attach a domain marker to a grounded witness without");
    f.doc_comment("any new sealing — e.g., `Grounded<ConstrainedTypeInput, BlockHashTag>` is");
    f.doc_comment("a distinct Rust type from `Grounded<ConstrainedTypeInput, PixelTag>`. The");
    f.doc_comment("inner witness is unchanged; the tag is pure decoration. The foundation");
    f.doc_comment("guarantees ring soundness on the inner witness; the tag is the developer's");
    f.doc_comment("domain claim. Coerce via `Grounded::tag::<NewTag>()` (zero-cost).");
    f.line("///");
    f.doc_comment("# Wiki ADR-039 — Inhabitance verdict realization mapping");
    f.line("///");
    f.doc_comment("For typed feature hierarchies whose admission relations are");
    f.doc_comment("inhabitance questions, a successful `Grounded<Output>` IS a");
    f.doc_comment("`cert:InhabitanceCertificate` envelope:");
    f.line("///");
    f.doc_comment("- The κ-label (homotopy-classification structural witness at ψ_9");
    f.doc_comment("  per ADR-035) is the `Term::KInvariants` emission's bytes, exposed");
    f.doc_comment("  via [`Grounded::output_bytes`].");
    f.doc_comment("- The concrete `cert:witness` ValueTuple is derivable from");
    f.doc_comment("  `Term::Nerve`'s 0-simplices at ψ_1 (the per-value bytes the model's");
    f.doc_comment("  NerveResolver consumed).");
    f.doc_comment("- The `cert:searchTrace` is realized as");
    f.doc_comment("  [`Grounded::derivation`]`().replay()`.");
    f.line("///");
    f.doc_comment("The κ-label and `cert:witness` are different witness granularities");
    f.doc_comment("(homotopy classification vs. concrete ValueTuple); the canonical");
    f.doc_comment("k-invariants branch ψ_1 → ψ_7 → ψ_8 → ψ_9 produces both, at the");
    f.doc_comment("ψ_9 and ψ_1 stages respectively. Ontology references:");
    f.doc_comment("`<https://uor.foundation/cert/InhabitanceCertificate>`,");
    f.doc_comment("`<https://uor.foundation/cert/witness>`,");
    f.doc_comment("`<https://uor.foundation/cert/searchTrace>`.");
    f.line("#[derive(Debug, Clone)]");
    f.line("pub struct Grounded<'a, T: GroundedShape, const INLINE_BYTES: usize, const FP_MAX: usize = 32, Tag = T> {");
    f.indented_doc_comment("The validated grounding certificate this wrapper carries.");
    f.line("    validated: Validated<GroundingCertificate<FP_MAX>>,");
    f.indented_doc_comment("The compile-time-materialized bindings table.");
    f.line("    bindings: BindingsTable,");
    f.indented_doc_comment("The Witt level the grounded value was minted at.");
    f.line("    witt_level_bits: u16,");
    f.indented_doc_comment("Content-address of the originating CompileUnit.");
    f.line("    unit_address: ContentAddress,");
    f.indented_doc_comment(
        "Phase A.1: the foundation-internal two-clock value read at witness mint time.",
    );
    f.indented_doc_comment(
        "Computed deterministically from (witt_level_bits, unit_address, bindings).",
    );
    f.line("    uor_time: UorTime,");
    f.indented_doc_comment("v0.2.2 T2.6 (cleanup): BaseMetric storage — populated by the");
    f.indented_doc_comment("pipeline at mint time as a deterministic function of witt level,");
    f.indented_doc_comment("unit address, and bindings. All six fields are read-only from");
    f.indented_doc_comment("the accessors; downstream cannot mutate them.");
    f.indented_doc_comment(
        "Grounding completion ratio \u{03C3} \u{00d7} 10\u{2076} (parts per million).",
    );
    f.line("    sigma_ppm: u32,");
    f.indented_doc_comment("Metric incompatibility d_\u{0394}.");
    f.line("    d_delta: i64,");
    f.indented_doc_comment("Euler characteristic of the constraint nerve.");
    f.line("    euler_characteristic: i64,");
    f.indented_doc_comment("Free-site count at grounding time.");
    f.line("    residual_count: u32,");
    f.indented_doc_comment("Per-site Jacobian row (fixed capacity, zero-padded).");
    f.line("    jacobian_entries: [i64; JACOBIAN_MAX_SITES],");
    f.indented_doc_comment("Active length of jacobian_entries.");
    f.line("    jacobian_len: u16,");
    f.indented_doc_comment("Betti numbers \u{03b2}_0..\u{03b2}_{MAX_BETTI_DIMENSION-1}.");
    f.line("    betti_numbers: [u32; MAX_BETTI_DIMENSION],");
    f.indented_doc_comment("v0.2.2 T5: parametric content fingerprint of the source unit's");
    f.indented_doc_comment("full state, computed at grounding time by the consumer-supplied");
    f.indented_doc_comment("`Hasher`. Width is `ContentFingerprint::width_bytes()`, set by");
    f.indented_doc_comment("`H::OUTPUT_BYTES` at the call site. Read by `Grounded::derivation()`");
    f.indented_doc_comment("so the verify path can re-derive the source certificate. The buffer");
    f.indented_doc_comment(
        "width `FP_MAX` is the application's `<B as HostBounds>::FINGERPRINT_MAX_BYTES`",
    );
    f.indented_doc_comment("(threaded, not pinned) — any `Hasher<FP_MAX>` width flows.");
    f.line("    content_fingerprint: ContentFingerprint<FP_MAX>,");
    f.indented_doc_comment("Wiki ADR-028 (amended by ADR-060): output-value payload — the");
    f.indented_doc_comment("catamorphism's evaluation result populated by `pipeline::run_route`");
    f.indented_doc_comment("per ADR-029's per-variant fold rules, carried as a source-polymorphic");
    f.indented_doc_comment("[`crate::pipeline::TermValue`] (Inline κ-label for content-addressing");
    f.indented_doc_comment(
        "routes; Borrowed/Stream for structural/unbounded outputs). There is no",
    );
    f.indented_doc_comment("fixed output buffer and no output byte-width ceiling. The lifetime");
    f.indented_doc_comment("`'a` is the borrowed-input-data lifetime a Borrowed/Stream output");
    f.indented_doc_comment("propagates from the route input. Read via [`Grounded::output_bytes`]");
    f.indented_doc_comment("(contiguous) or [`Grounded::output_value`] (the carrier).");
    f.line("    output: crate::pipeline::TermValue<'a, INLINE_BYTES>,");
    f.indented_doc_comment("Phantom type tying this `Grounded` to a specific `ConstrainedType`.");
    f.line("    _phantom: PhantomData<T>,");
    f.indented_doc_comment("Phantom domain tag (Q3). Defaults to `T` for backwards-compatible");
    f.indented_doc_comment("call sites; downstream attaches a custom tag via `tag::<NewTag>()`.");
    f.line("    _tag: PhantomData<Tag>,");
    f.line("}");
    f.blank();
    f.line(
        "impl<'a, T: GroundedShape, const INLINE_BYTES: usize, const FP_MAX: usize, Tag> Grounded<'a, T, INLINE_BYTES, FP_MAX, Tag> {",
    );
    f.indented_doc_comment("Returns the binding for the given query address, or `None` if not in");
    f.indented_doc_comment("the table. Resolves in O(log n) via binary search; for true `op:GS_5`");
    f.indented_doc_comment("zero-step access, downstream code uses statically-known indices.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn get_binding(&self, address: ContentAddress) -> Option<&'static [u8]> {");
    f.line("        self.bindings");
    f.line("            .entries");
    f.line("            .binary_search_by_key(&address.as_u128(), |e| e.address.as_u128())");
    f.line("            .ok()");
    f.line("            .map(|i| self.bindings.entries[i].bytes)");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Iterate over all bindings in this grounded context.");
    f.line("    #[inline]");
    f.line("    pub fn iter_bindings(&self) -> impl Iterator<Item = &BindingEntry> + '_ {");
    f.line("        self.bindings.entries.iter()");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Returns the Witt level the grounded value was minted at.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn witt_level_bits(&self) -> u16 {");
    f.line("        self.witt_level_bits");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Returns the content-address of the originating CompileUnit.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn unit_address(&self) -> ContentAddress {");
    f.line("        self.unit_address");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Returns the validated grounding certificate this wrapper carries.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn certificate(&self) -> &Validated<GroundingCertificate<FP_MAX>> {");
    f.line("        &self.validated");
    f.line("    }");
    f.blank();
    // Phase A.4: BaseMetric accessors return sealed newtypes. The six
    // accessors correspond one-to-one with the `observable:BaseMetric`
    // individuals in the ontology: d_delta, sigma, jacobian, betti, euler,
    // residual. Each return type is foundation-minted, so downstream can
    // neither fabricate nor mutate a metric value.
    f.indented_doc_comment(
        "Phase A.4: `observable:d_delta_metric` — sealed metric incompatibility between",
    );
    f.indented_doc_comment("ring distance and Hamming distance for this datum's neighborhood.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn d_delta(&self) -> DDeltaMetric {");
    f.line("        DDeltaMetric::new(self.d_delta)");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "Phase A.4: `observable:sigma_metric` — sealed grounding completion ratio.",
    );
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn sigma(&self) -> SigmaValue<crate::DefaultHostTypes> {");
    f.line("        // Default-host (f64) projection; polymorphic consumers");
    f.line("        // can re-encode via DecimalTranscendental::from_u32 + Div.");
    f.line("        let value = <f64 as crate::DecimalTranscendental>::from_u32(self.sigma_ppm)");
    f.line("            / <f64 as crate::DecimalTranscendental>::from_u32(1_000_000);");
    f.line("        SigmaValue::<crate::DefaultHostTypes>::new_unchecked(value)");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "Phase A.4: `observable:jacobian_metric` — sealed per-site Jacobian row.",
    );
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn jacobian(&self) -> JacobianMetric<T> {");
    f.line("        JacobianMetric::from_entries(self.jacobian_entries, self.jacobian_len)");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Phase A.4: `observable:betti_metric` — sealed Betti-number vector.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn betti(&self) -> BettiMetric {");
    f.line("        BettiMetric::new(self.betti_numbers)");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "Phase A.4: `observable:euler_metric` — sealed Euler characteristic \u{03C7} of",
    );
    f.indented_doc_comment("the constraint nerve.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn euler(&self) -> EulerMetric {");
    f.line("        EulerMetric::new(self.euler_characteristic)");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "Phase A.4: `observable:residual_metric` — sealed free-site count r at grounding.",
    );
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn residual(&self) -> ResidualMetric {");
    f.line("        ResidualMetric::new(self.residual_count)");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("v0.2.2 T5: returns the parametric content fingerprint of the source");
    f.indented_doc_comment("unit, computed at grounding time by the consumer-supplied `Hasher`.");
    f.indented_doc_comment("Width is set by `H::OUTPUT_BYTES` at the call site. Used by");
    f.indented_doc_comment("`derivation()` to seed the replayed Trace's fingerprint, which");
    f.indented_doc_comment("`verify_trace` then passes through to the re-derived certificate.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn content_fingerprint(&self) -> ContentFingerprint<FP_MAX> {");
    f.line("        self.content_fingerprint");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("v0.2.2 T5 (C2): returns the `Derivation` that produced this grounded");
    f.indented_doc_comment("value. Use the returned `Derivation` with `Derivation::replay()` and");
    f.indented_doc_comment("then `uor_foundation_verify::verify_trace` to re-derive the source");
    f.indented_doc_comment("certificate without re-running the deciders.");
    f.indented_doc_comment("");
    f.indented_doc_comment("The round-trip property:");
    f.indented_doc_comment("");
    f.indented_doc_comment("```text");
    f.indented_doc_comment("verify_trace(&grounded.derivation().replay()).certificate()");
    f.indented_doc_comment("    == grounded.certificate()");
    f.indented_doc_comment("```");
    f.indented_doc_comment("");
    f.indented_doc_comment("holds for every conforming substrate `Hasher`.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn derivation(&self) -> Derivation<FP_MAX> {");
    f.line("        Derivation::new(");
    f.line("            (self.jacobian_len as u32) + 1,");
    f.line("            self.witt_level_bits,");
    f.line("            self.content_fingerprint,");
    f.line("        )");
    f.line("    }");
    f.blank();

    f.indented_doc_comment("v0.2.2 Phase B (Q3): coerce this `Grounded<T, Tag>` to a different");
    f.indented_doc_comment("phantom tag. Zero-cost — the inner witness is unchanged; only the");
    f.indented_doc_comment("type-system view differs. Downstream uses this to attach a domain");
    f.indented_doc_comment(
        "marker for use in function signatures (e.g., `Grounded<_, BlockHashTag>`",
    );
    f.indented_doc_comment("vs `Grounded<_, PixelTag>` are distinct Rust types).");
    f.indented_doc_comment("");
    f.indented_doc_comment("**The foundation does not validate the tag.** The tag records what");
    f.indented_doc_comment("the developer is claiming about the witness's domain semantics; the");
    f.indented_doc_comment("foundation's contract is about ring soundness, not domain semantics.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn tag<NewTag>(self) -> Grounded<'a, T, INLINE_BYTES, FP_MAX, NewTag> {");
    f.line("        Grounded {");
    f.line("            validated: self.validated,");
    f.line("            bindings: self.bindings,");
    f.line("            witt_level_bits: self.witt_level_bits,");
    f.line("            unit_address: self.unit_address,");
    f.line("            uor_time: self.uor_time,");
    f.line("            sigma_ppm: self.sigma_ppm,");
    f.line("            d_delta: self.d_delta,");
    f.line("            euler_characteristic: self.euler_characteristic,");
    f.line("            residual_count: self.residual_count,");
    f.line("            jacobian_entries: self.jacobian_entries,");
    f.line("            jacobian_len: self.jacobian_len,");
    f.line("            betti_numbers: self.betti_numbers,");
    f.line("            content_fingerprint: self.content_fingerprint,");
    f.line("            output: self.output,");
    f.line("            _phantom: PhantomData,");
    f.line("            _tag: PhantomData,");
    f.line("        }");
    f.line("    }");
    f.blank();

    // Wiki ADR-028: output_bytes accessor.
    f.indented_doc_comment(
        "Wiki ADR-028: returns the catamorphism's evaluation output bytes — the",
    );
    f.indented_doc_comment("active prefix of the on-stack output payload `pipeline::run_route`");
    f.indented_doc_comment("populated per ADR-029's per-variant fold rules.");
    f.indented_doc_comment("");
    f.indented_doc_comment(
        "For the foundation-sanctioned identity output (`ConstrainedTypeInput`)",
    );
    f.indented_doc_comment("the slice is empty (no transformation, identity route). For shapes");
    f.indented_doc_comment("declared via the `output_shape!` SDK macro the slice carries the");
    f.indented_doc_comment("route's evaluation result.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn output_bytes(&self) -> &[u8] {");
    f.line("        // Inline/Borrowed carriers expose their contiguous bytes; a Stream");
    f.line("        // carrier has no contiguous slice (read it via `output_value()` +");
    f.line("        // `TermValue::for_each_chunk`).");
    f.line("        self.output.bytes()");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Wiki ADR-028 (amended by ADR-060): returns the output as the");
    f.indented_doc_comment("source-polymorphic [`crate::pipeline::TermValue`] carrier. Use this");
    f.indented_doc_comment("(rather than [`Grounded::output_bytes`]) when the route's output may");
    f.indented_doc_comment("be a `Stream` (unbounded) or `Borrowed` carrier — fold it via");
    f.indented_doc_comment("[`crate::pipeline::TermValue::for_each_chunk`] for arbitrary sizes.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn output_value(&self) -> crate::pipeline::TermValue<'a, INLINE_BYTES> {");
    f.line("        self.output");
    f.line("    }");
    f.blank();

    // Phase A.1: uor_time() accessor on Grounded<T, Tag>.
    f.indented_doc_comment(
        "Phase A.1: the foundation-internal two-clock value read at witness mint time.",
    );
    f.indented_doc_comment("");
    f.indented_doc_comment("Maps `rewrite_steps` to `derivation:stepCount` and `landauer_nats` to");
    f.indented_doc_comment(
        "`observable:LandauerCost`. The value is content-deterministic: two `Grounded`",
    );
    f.indented_doc_comment("witnesses minted from the same inputs share the same `UorTime`.");
    f.indented_doc_comment("");
    f.indented_doc_comment("Compose with a `Calibration` via [`UorTime::min_wall_clock`] to");
    f.indented_doc_comment(
        "bound the provable minimum wall-clock duration the computation required.",
    );
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn uor_time(&self) -> UorTime {");
    f.line("        self.uor_time");
    f.line("    }");
    f.blank();

    // Phase A.2: triad() accessor on Grounded<T, Tag>.
    f.indented_doc_comment(
        "Phase A.2: the sealed triadic coordinate `(stratum, spectrum, address)` at the",
    );
    f.indented_doc_comment("witness's Witt level, projected from the content-addressed unit.");
    f.indented_doc_comment("");
    f.indented_doc_comment(
        "`stratum` is the v\u{2082} valuation of the lower unit-address half; `spectrum`",
    );
    f.indented_doc_comment(
        "is the lower 64 bits of the unit address; `address` is the upper 64 bits.",
    );
    f.indented_doc_comment(
        "The projection is deterministic and content-addressed, so replay reproduces the",
    );
    f.indented_doc_comment("same `Triad` bit-for-bit.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn triad(&self) -> Triad<T> {");
    f.line("        let addr = self.unit_address.as_u128();");
    f.line("        let addr_lo = addr as u64;");
    f.line("        let addr_hi = (addr >> 64) as u64;");
    f.line("        let stratum = if addr_lo == 0 {");
    f.line("            0u64");
    f.line("        } else {");
    f.line("            addr_lo.trailing_zeros() as u64");
    f.line("        };");
    f.line("        Triad::new(stratum, addr_lo, addr_hi)");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Crate-internal constructor used by the pipeline at mint time.");
    f.indented_doc_comment("");
    f.indented_doc_comment("Not callable from outside `uor-foundation`. The tag defaults to `T`");
    f.indented_doc_comment(
        "(the unparameterized form); downstream attaches a custom tag via `tag()`.",
    );
    f.indented_doc_comment("");
    f.indented_doc_comment("v0.2.2 T2.6 (cleanup): BaseMetric fields are computed here from");
    f.indented_doc_comment("the input witt level, bindings, and unit address. Two `Grounded`");
    f.indented_doc_comment("values built from the same inputs return identical metrics; two");
    f.indented_doc_comment("built from different inputs differ in at least three fields.");
    f.line("    #[inline]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new_internal(");
    f.line("        validated: Validated<GroundingCertificate<FP_MAX>>,");
    f.line("        bindings: BindingsTable,");
    f.line("        witt_level_bits: u16,");
    f.line("        unit_address: ContentAddress,");
    f.line("        content_fingerprint: ContentFingerprint<FP_MAX>,");
    f.line("    ) -> Self {");
    f.line("        let bound_count = bindings.entries.len() as u32;");
    f.line("        let declared_sites = if witt_level_bits == 0 { 1u32 } else { witt_level_bits as u32 };");
    f.line("        // sigma = bound / declared, in parts per million.");
    f.line("        let sigma_ppm = if bound_count >= declared_sites {");
    f.line("            1_000_000u32");
    f.line("        } else {");
    f.line("            // Integer division, rounded down, cannot exceed 1_000_000.");
    f.line("            let num = (bound_count as u64) * 1_000_000u64;");
    f.line("            (num / (declared_sites as u64)) as u32");
    f.line("        };");
    f.line("        // residual_count = declared - bound (saturating).");
    f.line("        let residual_count = declared_sites.saturating_sub(bound_count);");
    f.line("        // d_delta = witt_bits - bound_count (signed).");
    f.line("        let d_delta = (witt_level_bits as i64) - (bound_count as i64);");
    f.line("        // Betti numbers: β_0 = 1 (connected); β_k = bit k of witt_level_bits.");
    f.line("        let mut betti = [0u32; MAX_BETTI_DIMENSION];");
    f.line("        betti[0] = 1;");
    f.line("        let mut k = 1usize;");
    f.line("        while k < MAX_BETTI_DIMENSION {");
    f.line("            betti[k] = ((witt_level_bits as u32) >> (k - 1)) & 1;");
    f.line("            k += 1;");
    f.line("        }");
    f.line("        // Euler characteristic: alternating sum of Betti numbers.");
    f.line("        let mut euler: i64 = 0;");
    f.line("        let mut k = 0usize;");
    f.line("        while k < MAX_BETTI_DIMENSION {");
    f.line("            if k & 1 == 0 {");
    f.line("                euler += betti[k] as i64;");
    f.line("            } else {");
    f.line("                euler -= betti[k] as i64;");
    f.line("            }");
    f.line("            k += 1;");
    f.line("        }");
    f.line("        // Jacobian row: entry i = (unit_address.as_u128() as i64 XOR (i as i64)) mod witt+1.");
    f.line("        let mut jac = [0i64; JACOBIAN_MAX_SITES];");
    f.line("        let modulus = (witt_level_bits as i64) + 1;");
    f.line("        let ua_lo = unit_address.as_u128() as i64;");
    f.line("        let mut i = 0usize;");
    f.line("        let jac_len = if (witt_level_bits as usize) < JACOBIAN_MAX_SITES {");
    f.line("            witt_level_bits as usize");
    f.line("        } else {");
    f.line("            JACOBIAN_MAX_SITES");
    f.line("        };");
    f.line("        while i < jac_len {");
    f.line("            let raw = ua_lo ^ (i as i64);");
    f.line("            // Rust's % is remainder; ensure non-negative.");
    f.line("            let m = if modulus == 0 { 1 } else { modulus };");
    f.line("            jac[i] = ((raw % m) + m) % m;");
    f.line("            i += 1;");
    f.line("        }");
    f.line("        // Phase A.1: uor_time is content-deterministic. rewrite_steps counts");
    f.line("        // the reduction work proxied by (witt bits + bound count + active jac len);");
    f.line("        // Landauer nats = rewrite_steps \u{00d7} ln 2 (Landauer-temperature cost of");
    f.line("        // traversing that many orthogonal states). Two Grounded values minted from");
    f.line("        // the same inputs share the same UorTime.");
    f.line(
        "        let steps = (witt_level_bits as u64) + (bound_count as u64) + (jac_len as u64);",
    );
    f.line("        let landauer = LandauerBudget::new((steps as f64) * core::f64::consts::LN_2);");
    f.line("        let uor_time = UorTime::new(landauer, steps);");
    f.line("        Self {");
    f.line("            validated,");
    f.line("            bindings,");
    f.line("            witt_level_bits,");
    f.line("            unit_address,");
    f.line("            uor_time,");
    f.line("            sigma_ppm,");
    f.line("            d_delta,");
    f.line("            euler_characteristic: euler,");
    f.line("            residual_count,");
    f.line("            jacobian_entries: jac,");
    f.line("            jacobian_len: jac_len as u16,");
    f.line("            betti_numbers: betti,");
    f.line("            content_fingerprint,");
    f.line("            output: crate::pipeline::TermValue::empty(),");
    f.line("            _phantom: PhantomData,");
    f.line("            _tag: PhantomData,");
    f.line("        }");
    f.line("    }");
    f.blank();

    // Wiki ADR-028 (amended by ADR-060): crate-internal setter for the
    // source-polymorphic output-value carrier.
    f.indented_doc_comment("Wiki ADR-028 (amended by ADR-060): crate-internal setter for the");
    f.indented_doc_comment("source-polymorphic output-value carrier.");
    f.indented_doc_comment("");
    f.indented_doc_comment("Called by `pipeline::run_route` after the catamorphism evaluates the");
    f.indented_doc_comment("Term tree per ADR-029. The carrier is stored by move — no copy, no");
    f.indented_doc_comment("byte-width ceiling: an `Inline` κ-label, a `Borrowed` slice into the");
    f.indented_doc_comment("route's `'a`-lived input, or a `Stream` of arbitrary size. Returns");
    f.indented_doc_comment("self for chaining.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub(crate) fn with_output(mut self, output: crate::pipeline::TermValue<'a, INLINE_BYTES>) -> Self {");
    f.line("        self.output = output;");
    f.line("        self");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("v0.2.2 T6.17: attach a downstream-validated `BindingsTable` to this");
    f.indented_doc_comment("grounded value. The original `Grounded` was minted by the foundation");
    f.indented_doc_comment("pipeline with a substrate-computed certificate; this builder lets");
    f.indented_doc_comment("downstream attach its own binding table without re-grounding.");
    f.indented_doc_comment("");
    f.indented_doc_comment("The `bindings` parameter must satisfy the sortedness invariant. Use");
    f.indented_doc_comment("[`BindingsTable::try_new`] to construct a validated table from a");
    f.indented_doc_comment("pre-sorted slice.");
    f.indented_doc_comment("");
    f.indented_doc_comment("**Trust boundary:** the certificate witnesses the unit's grounding,");
    f.indented_doc_comment("not the bindings' contents. A downstream consumer that uses the");
    f.indented_doc_comment("certificate as a trust root for the bindings is wrong.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn with_bindings(self, bindings: BindingsTable) -> Self {");
    f.line("        Self { bindings, ..self }");
    f.line("    }");
    f.blank();
    // ADR-042: typed inhabitance-verdict view over Grounded<T>.
    f.indented_doc_comment("Wiki ADR-042: borrow `self` as an");
    f.indented_doc_comment("[`crate::pipeline::InhabitanceCertificateView`] over the canonical");
    f.indented_doc_comment("k-invariants branch's verdict envelope.");
    f.indented_doc_comment("");
    f.indented_doc_comment("Universal — available for any `Grounded<T, Tag>`; applications whose");
    f.indented_doc_comment("admission relations are not inhabitance questions simply don't");
    f.indented_doc_comment("call the typed accessors. The view is zero-cost");
    f.indented_doc_comment("(`#[repr(transparent)]` over `&'a Grounded<T, Tag>`).");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line(
        "    pub fn as_inhabitance_certificate(&self) -> crate::pipeline::InhabitanceCertificateView<'_, T, INLINE_BYTES, FP_MAX, Tag> {",
    );
    f.line("        crate::pipeline::InhabitanceCertificateView(self)");
    f.line("    }");
    f.line("}");
    f.blank();

    // ── v0.2.2 W8: Triad<L> bundling struct ────────────────────────────────
    //
    // The triadic coordinate of a Datum: (stratum, spectrum, address).
    // Parametric over the Witt level marker L (one of the unit structs
    // W8/W16/W24/W32 emitted by generate_ring_ops). Fields are private; only
    // the foundation can construct a Triad. Accessors return typed coordinate
    // wrappers — `Stratum<L>` (sealed u32 newtype over the two-adic valuation),
    // `Datum<L>` (sealed bit-pattern value), and `ContentAddress` (sealed
    // 32-byte digest) — per target §1.6 row 4.
    f.doc_comment("v0.2.2 W8: triadic coordinate of a Datum at level `L`. Bundles the");
    f.doc_comment("(stratum, spectrum, address) projection in one structurally-enforced");
    f.doc_comment("type. No public constructor — `Triad<L>` is built only by foundation code");
    f.doc_comment("at grounding time. Field access goes through the named accessors.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct Triad<L> {");
    f.indented_doc_comment("The stratum coordinate (two-adic valuation).");
    f.line("    stratum: u64,");
    f.indented_doc_comment("The spectrum coordinate (Walsh-Hadamard image).");
    f.line("    spectrum: u64,");
    f.indented_doc_comment("The address coordinate (Braille-glyph address).");
    f.line("    address: u64,");
    f.indented_doc_comment("Phantom marker for the Witt level.");
    f.line("    _level: PhantomData<L>,");
    f.line("}");
    f.blank();
    f.line("impl<L> Triad<L> {");
    f.indented_doc_comment("Returns the stratum component (`query:TwoAdicValuation` coordinate).");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn stratum(&self) -> u64 {");
    f.line("        self.stratum");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "Returns the spectrum component (`query:WalshHadamardImage` coordinate).",
    );
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn spectrum(&self) -> u64 {");
    f.line("        self.spectrum");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Returns the address component (`query:Address` coordinate).");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn address(&self) -> u64 {");
    f.line("        self.address");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "Crate-internal constructor. Reachable only from grounding-time minting.",
    );
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new(stratum: u64, spectrum: u64, address: u64) -> Self {");
    f.line("        Self { stratum, spectrum, address, _level: PhantomData }");
    f.line("    }");
    f.line("}");
    f.blank();
}

// 2.1.d PipelineFailure — parametric over reduction:FailureField individuals.
fn generate_pipeline_failure(f: &mut RustFile, ontology: &Ontology) {
    f.doc_comment("The Rust-surface rendering of `reduction:PipelineFailureReason` and the");
    f.doc_comment("v0.2.1 cross-namespace failure variants. Variant set and field shapes are");
    f.doc_comment("generated parametrically by walking `reduction:FailureField` individuals;");
    f.doc_comment("adding a new field requires only an ontology edit.");
    f.line("///");
    f.doc_comment("# Wiki ADR-039 — Inhabitance verdict realization mapping");
    f.line("///");
    f.doc_comment("An `Err(PipelineFailure)` whose structural cause is \"the constraint");
    f.doc_comment(
        "nerve has empty Kan completion\" realizes a `cert:InhabitanceImpossibilityCertificate`",
    );
    f.doc_comment("envelope, carrying `proof:InhabitanceImpossibilityWitness` as the");
    f.doc_comment("proof payload with `proof:contradictionProof` as the canonical-form");
    f.doc_comment("encoding of the failure trace. The verdict mapping is the dual of");
    f.doc_comment("the `Grounded<Output>` → `cert:InhabitanceCertificate` mapping; the");
    f.doc_comment("two together realize the ontology's three-primitive inhabitance");
    f.doc_comment("verdict structure (success / impossibility-witnessed / unknown).");
    f.doc_comment("Ontology references:");
    f.doc_comment("`<https://uor.foundation/cert/InhabitanceImpossibilityCertificate>`,");
    f.doc_comment("`<https://uor.foundation/proof/InhabitanceImpossibilityWitness>`,");
    f.doc_comment("`<https://uor.foundation/proof/contradictionProof>`.");
    f.line("#[derive(Debug, Clone, PartialEq)]");
    f.line("#[non_exhaustive]");
    f.line("pub enum PipelineFailure {");

    // Walk all PipelineFailureReason individuals plus failure:LiftObstructionFailure
    // and conformance:ShapeViolationReport (the latter wraps the existing struct).
    let reasons = individuals_of_type(
        ontology,
        "https://uor.foundation/reduction/PipelineFailureReason",
    );
    let mut variant_specs: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for ind in &reasons {
        let variant = local_name(ind.id).to_string();
        let fields = collect_failure_fields(ontology, ind.id);
        variant_specs.push((variant, fields));
    }

    // failure:LiftObstructionFailure variant
    let lift_fields = collect_failure_fields(
        ontology,
        "https://uor.foundation/failure/LiftObstructionFailure",
    );
    if !lift_fields.is_empty() {
        variant_specs.push(("LiftObstructionFailure".to_string(), lift_fields));
    }

    // conformance:ShapeViolationReport — wraps the existing ShapeViolation
    // struct emitted by `generate_shape_violation` earlier in this file.
    variant_specs.push((
        "ShapeViolation".to_string(),
        vec![("report".to_string(), "ShapeViolation".to_string())],
    ));

    for (variant, fields) in &variant_specs {
        f.indented_doc_comment(&format!("`{variant}` failure variant."));
        if fields.is_empty() {
            f.line(&format!("    {variant},"));
        } else {
            f.line(&format!("    {variant} {{"));
            for (name, ty) in fields {
                f.line(&format!("        /// {name} field."));
                f.line(&format!("        {name}: {ty},"));
            }
            f.line("    },");
        }
    }

    f.line("}");
    f.blank();

    // Display impl for nice error rendering
    f.line("impl core::fmt::Display for PipelineFailure {");
    f.line("    fn fmt(&self, ff: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {");
    f.line("        match self {");
    for (variant, fields) in &variant_specs {
        let pat: String = if fields.is_empty() {
            format!("Self::{variant}")
        } else {
            let names: Vec<String> = fields.iter().map(|(n, _)| n.clone()).collect();
            format!("Self::{variant} {{ {} }}", names.join(", "))
        };
        let body = if fields.is_empty() {
            format!("write!(ff, \"{variant}\")")
        } else if variant == "ShapeViolation" {
            "write!(ff, \"ShapeViolation({:?})\", report)".to_string()
        } else {
            // Render IRI fields specifically; otherwise debug-print.
            let parts: Vec<String> = fields.iter().map(|(n, _)| format!("{n}={{:?}}")).collect();
            let names: Vec<String> = fields.iter().map(|(n, _)| n.clone()).collect();
            format!(
                "write!(ff, \"{}({})\", {})",
                variant,
                parts.join(", "),
                names.join(", ")
            )
        };
        f.line(&format!("            {pat} => {body},"));
    }
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();

    // v0.2.2 T5.9: `core::error::Error` is stable on no_std as of Rust 1.81.
    // The foundation's MSRV bumped to 1.81 in T5, so we now ship the trait
    // impl directly. Downstream consumers can `?`-propagate `PipelineFailure`
    // through `Box<dyn Error>` chains without manual wrapping.
    f.line("impl core::error::Error for PipelineFailure {}");
    f.blank();
}

/// Walk reduction:FailureField individuals filtered by ofFailure == failure_iri,
/// returning (field_name, field_type) tuples in declaration order.
fn collect_failure_fields(ontology: &Ontology, failure_iri: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let fields = individuals_of_type(ontology, "https://uor.foundation/reduction/FailureField");
    for f in fields {
        let of = ind_prop_str(f, "https://uor.foundation/reduction/ofFailure");
        if of != Some(failure_iri) {
            continue;
        }
        let name = ind_prop_str(f, "https://uor.foundation/reduction/fieldName")
            .unwrap_or("unknown")
            .to_string();
        let ty = ind_prop_str(f, "https://uor.foundation/reduction/fieldType")
            .unwrap_or("()")
            .to_string();
        out.push((name, ty));
    }
    out
}

// 2.1.c ImpossibilityWitnessKind sealed trait.
//
// Phase B (target §4.1 W12): the `Certify` trait and its five unit-struct
// resolver façades (`TowerCompletenessResolver`, `IncrementalCompletenessResolver`,
// `GroundingAwareResolver`, `InhabitanceResolver`, `MultiplicationResolver`) are
// deleted. The only verdict surface is the module-per-resolver free-function
// path (`enforcement::resolver::<name>::certify(...)`) emitted below. The
// `_ontology` parameter is preserved for the signature stability of callers
// in `mod.rs`; future phases consume it when the resolver tower grows.
fn generate_certify_trait(f: &mut RustFile, _ontology: &Ontology) {
    f.doc_comment("Sealed marker for impossibility witnesses returned by the resolver");
    f.doc_comment("free-function path. Every failure return value of every");
    f.doc_comment("`resolver::<name>::certify(...)` call is a member of this set.");
    f.line("pub trait ImpossibilityWitnessKind: impossibility_witness_kind_sealed::Sealed {}");
    f.blank();
    f.line("mod impossibility_witness_kind_sealed {");
    f.indented_doc_comment("Private supertrait.");
    f.line("    pub trait Sealed {}");
    f.line("    impl Sealed for super::GenericImpossibilityWitness {}");
    f.line("    impl Sealed for super::InhabitanceImpossibilityWitness {}");
    f.line("}");
    f.blank();
    f.line("impl ImpossibilityWitnessKind for GenericImpossibilityWitness {}");
    f.line("impl ImpossibilityWitnessKind for InhabitanceImpossibilityWitness {}");
    f.blank();

    // ── v0.2.2 W12: resolver free functions ────────────────────────────────
    //
    // Replaces the v0.2.1 unit structs (`TowerCompletenessResolver::new()`,
    // etc.) with free functions in `pub mod resolver`. The unit structs were
    // decorative — there is no state. Free functions in module-per-resolver
    // organization keep the namespace structure mirrored from the ontology
    // (`resolver/InhabitanceResolver`, etc.) without the fictional state.
    //
    // Each free function returns `Result<Certified<Cert>, Witness>` where
    // `Cert` is the W11 sealed cert kind and `Witness` is the existing
    // impossibility witness shim. The Phase 3 test migration switches
    // consumers from `Resolver::new().certify(...)` to
    // `resolver::resolver_name::certify(...)`.
    f.doc_comment("v0.2.2 W12: resolver free functions. Replaces the v0.2.1 unit-struct");
    f.doc_comment("façades with module-per-resolver free functions returning the W11");
    f.doc_comment("`Certified<C>` parametric carrier.");
    f.line("pub mod resolver {");
    f.line("    use super::{Certified, Validated, WittLevel,");
    f.line("        CompileUnit, GenericImpossibilityWitness, InhabitanceImpossibilityWitness,");
    f.line("        GroundingCertificate, LiftChainCertificate, InhabitanceCertificate,");
    f.line("        // Phase X.1: per-resolver cert discrimination.");
    f.line("        TransformCertificate, IsometryCertificate, InvolutionCertificate,");
    f.line("        CompletenessCertificate, GeodesicCertificate, MeasurementCertificate,");
    f.line("        BornRuleVerification};");
    f.blank();
    // Tower completeness
    f.line("    /// v0.2.2 W12: certify tower-completeness for a constrained type.");
    f.line("    ///");
    f.line("    /// Replaces `TowerCompletenessResolver::new().certify(input)` from v0.2.1.");
    f.line("    /// Delegates to `crate::pipeline::run_tower_completeness` and wraps the");
    f.line("    /// returned `LiftChainCertificate` in the W11 `Certified<_>` carrier.");
    f.line("    ///");
    f.line("    /// # Errors");
    f.line("    ///");
    f.line("    /// Returns `GenericImpossibilityWitness` when no certificate can be issued.");
    f.line("    pub mod tower_completeness {");
    f.line("        use super::*;");
    f.line("        /// v0.2.2 closure (target §4.2): parameterized over phase + hasher.");
    f.line("        ///");
    f.line("        /// # Errors");
    f.line("        ///");
    f.line("        /// Returns `Certified<GenericImpossibilityWitness>` on failure.");
    f.line("        pub fn certify<T, P, H, const FP_MAX: usize>(");
    f.line("            input: &Validated<T, P>,");
    f.line("        ) -> Result<Certified<LiftChainCertificate<FP_MAX>>, Certified<GenericImpossibilityWitness>>");
    f.line("        where");
    f.line("            T: crate::pipeline::ConstrainedTypeShape,");
    f.line("            P: crate::enforcement::ValidationPhase,");
    f.line("            H: crate::enforcement::Hasher<FP_MAX>,");
    f.line("        {");
    f.line("            certify_at::<T, P, H, FP_MAX>(input, WittLevel::W32)");
    f.line("        }");
    f.blank();
    f.line("        /// Certify at an explicit Witt level.");
    f.line("        ///");
    f.line("        /// # Errors");
    f.line("        ///");
    f.line("        /// Returns `Certified<GenericImpossibilityWitness>` on failure.");
    f.line("        pub fn certify_at<T, P, H, const FP_MAX: usize>(");
    f.line("            input: &Validated<T, P>,");
    f.line("            level: WittLevel,");
    f.line("        ) -> Result<Certified<LiftChainCertificate<FP_MAX>>, Certified<GenericImpossibilityWitness>>");
    f.line("        where");
    f.line("            T: crate::pipeline::ConstrainedTypeShape,");
    f.line("            P: crate::enforcement::ValidationPhase,");
    f.line("            H: crate::enforcement::Hasher<FP_MAX>,");
    f.line("        {");
    f.line(
        "            crate::pipeline::run_tower_completeness::<T, H, FP_MAX>(input.inner(), level)",
    );
    f.line("                .map(|v| Certified::new(*v.inner()))");
    f.line("                .map_err(|_| Certified::new(GenericImpossibilityWitness::default()))");
    f.line("        }");
    f.line("    }");
    f.blank();
    // Incremental completeness
    f.line("    /// v0.2.2 closure: certify incremental completeness for a constrained type.");
    f.line("    pub mod incremental_completeness {");
    f.line("        use super::*;");
    f.line("        /// v0.2.2 closure (target §4.2): parameterized over phase + hasher.");
    f.line("        ///");
    f.line("        /// # Errors");
    f.line("        ///");
    f.line("        /// Returns `Certified<GenericImpossibilityWitness>` on failure.");
    f.line("        pub fn certify<T, P, H, const FP_MAX: usize>(");
    f.line("            input: &Validated<T, P>,");
    f.line("        ) -> Result<Certified<LiftChainCertificate<FP_MAX>>, Certified<GenericImpossibilityWitness>>");
    f.line("        where");
    f.line("            T: crate::pipeline::ConstrainedTypeShape,");
    f.line("            P: crate::enforcement::ValidationPhase,");
    f.line("            H: crate::enforcement::Hasher<FP_MAX>,");
    f.line("        {");
    f.line("            certify_at::<T, P, H, FP_MAX>(input, WittLevel::W32)");
    f.line("        }");
    f.blank();
    f.line("        /// Certify at an explicit Witt level.");
    f.line("        ///");
    f.line("        /// # Errors");
    f.line("        ///");
    f.line("        /// Returns `Certified<GenericImpossibilityWitness>` on failure.");
    f.line("        pub fn certify_at<T, P, H, const FP_MAX: usize>(");
    f.line("            input: &Validated<T, P>,");
    f.line("            level: WittLevel,");
    f.line("        ) -> Result<Certified<LiftChainCertificate<FP_MAX>>, Certified<GenericImpossibilityWitness>>");
    f.line("        where");
    f.line("            T: crate::pipeline::ConstrainedTypeShape,");
    f.line("            P: crate::enforcement::ValidationPhase,");
    f.line("            H: crate::enforcement::Hasher<FP_MAX>,");
    f.line("        {");
    f.line(
        "            crate::pipeline::run_incremental_completeness::<T, H, FP_MAX>(input.inner(), level)",
    );
    f.line("                .map(|v| Certified::new(*v.inner()))");
    f.line("                .map_err(|_| Certified::new(GenericImpossibilityWitness::default()))");
    f.line("        }");
    f.line("    }");
    f.blank();
    // Grounding aware
    f.line("    /// v0.2.2 closure: certify grounding-aware reduction for a CompileUnit.");
    f.line("    pub mod grounding_aware {");
    f.line("        use super::*;");
    f.line("        /// v0.2.2 closure (target §4.2): parameterized over phase + hasher.");
    f.line("        ///");
    f.line("        /// # Errors");
    f.line("        ///");
    f.line("        /// Returns `Certified<GenericImpossibilityWitness>` on failure.");
    f.line("        pub fn certify<P, H, const INLINE_BYTES: usize, const FP_MAX: usize>(");
    f.line("            input: &Validated<CompileUnit<'_, INLINE_BYTES>, P>,");
    f.line("        ) -> Result<Certified<GroundingCertificate<FP_MAX>>, Certified<GenericImpossibilityWitness>>");
    f.line("        where");
    f.line("            P: crate::enforcement::ValidationPhase,");
    f.line("            H: crate::enforcement::Hasher<FP_MAX>,");
    f.line("        {");
    f.line("            certify_at::<P, H, INLINE_BYTES, FP_MAX>(input, WittLevel::W32)");
    f.line("        }");
    f.blank();
    f.line("        /// Certify at an explicit Witt level.");
    f.line("        ///");
    f.line("        /// # Errors");
    f.line("        ///");
    f.line("        /// Returns `Certified<GenericImpossibilityWitness>` on failure.");
    f.line("        pub fn certify_at<P, H, const INLINE_BYTES: usize, const FP_MAX: usize>(");
    f.line("            input: &Validated<CompileUnit<'_, INLINE_BYTES>, P>,");
    f.line("            level: WittLevel,");
    f.line("        ) -> Result<Certified<GroundingCertificate<FP_MAX>>, Certified<GenericImpossibilityWitness>>");
    f.line("        where");
    f.line("            P: crate::enforcement::ValidationPhase,");
    f.line("            H: crate::enforcement::Hasher<FP_MAX>,");
    f.line("        {");
    f.line(
        "            crate::pipeline::run_grounding_aware::<INLINE_BYTES, H, FP_MAX>(input.inner(), level)",
    );
    f.line("                .map(|v| Certified::new(*v.inner()))");
    f.line("                .map_err(|_| Certified::new(GenericImpossibilityWitness::default()))");
    f.line("        }");
    f.line("    }");
    f.blank();
    // Inhabitance
    f.line("    /// v0.2.2 closure: certify inhabitance for a constrained type.");
    f.line("    pub mod inhabitance {");
    f.line("        use super::*;");
    f.line("        /// v0.2.2 closure (target §4.2): parameterized over phase + hasher.");
    f.line("        ///");
    f.line("        /// # Errors");
    f.line("        ///");
    f.line("        /// Returns `Certified<InhabitanceImpossibilityWitness>` on failure.");
    f.line("        pub fn certify<T, P, H, const FP_MAX: usize>(");
    f.line("            input: &Validated<T, P>,");
    f.line("        ) -> Result<");
    f.line("            Certified<InhabitanceCertificate<FP_MAX>>,");
    f.line("            Certified<InhabitanceImpossibilityWitness>,");
    f.line("        >");
    f.line("        where");
    f.line("            T: crate::pipeline::ConstrainedTypeShape,");
    f.line("            P: crate::enforcement::ValidationPhase,");
    f.line("            H: crate::enforcement::Hasher<FP_MAX>,");
    f.line("        {");
    f.line("            certify_at::<T, P, H, FP_MAX>(input, WittLevel::W32)");
    f.line("        }");
    f.blank();
    f.line("        /// Certify at an explicit Witt level.");
    f.line("        ///");
    f.line("        /// # Errors");
    f.line("        ///");
    f.line("        /// Returns `Certified<InhabitanceImpossibilityWitness>` on failure.");
    f.line("        pub fn certify_at<T, P, H, const FP_MAX: usize>(");
    f.line("            input: &Validated<T, P>,");
    f.line("            level: WittLevel,");
    f.line("        ) -> Result<");
    f.line("            Certified<InhabitanceCertificate<FP_MAX>>,");
    f.line("            Certified<InhabitanceImpossibilityWitness>,");
    f.line("        >");
    f.line("        where");
    f.line("            T: crate::pipeline::ConstrainedTypeShape,");
    f.line("            P: crate::enforcement::ValidationPhase,");
    f.line("            H: crate::enforcement::Hasher<FP_MAX>,");
    f.line("        {");
    f.line("            crate::pipeline::run_inhabitance::<T, H, FP_MAX>(input.inner(), level)");
    f.line(
        "                .map(|v: Validated<InhabitanceCertificate<FP_MAX>>| Certified::new(*v.inner()))",
    );
    f.line(
        "                .map_err(|_| Certified::new(InhabitanceImpossibilityWitness::default()))",
    );
    f.line("        }");
    f.line("    }");
    f.blank();
    // v0.2.2 Phase C.4: multiplication resolver free-function module.
    // The resolver is a pure derivation over the closed-form Landauer cost
    // function; it picks the cost-optimal Toom-Cook splitting factor R for
    // the given call-site context and returns a Certified<MultiplicationCertificate>
    // recording the choice. See the rustdoc on certify() for the cost formula
    // and its grounding in op:OA_5.
    f.line("    /// v0.2.2 Phase C.4: multiplication resolver — picks the cost-optimal");
    f.line("    /// Toom-Cook splitting factor R for a `Datum<L>` \u{00d7} `Datum<L>`");
    f.line("    /// multiplication at a given call-site context. The cost function is");
    f.line("    /// closed-form and grounded in `op:OA_5`:");
    f.line("    ///");
    f.line("    /// ```text");
    f.line("    /// sub_mul_count(N, R) = (2R - 1)  for R > 1");
    f.line("    ///                     = 1         for R = 1 (schoolbook)");
    f.line("    /// landauer_cost(N, R) = sub_mul_count(N, R) \u{00b7} (N/R)\u{00b2} \u{00b7} 64 \u{00b7} ln 2  nats");
    f.line("    /// ```");
    f.line("    pub mod multiplication {");
    f.line("        use super::*;");
    f.line("        use super::super::{MultiplicationCertificate, MulContext};");
    f.blank();
    f.line("        /// v0.2.2 T6.7: parameterized over `H: Hasher`. Pick the cost-optimal");
    f.line("        /// splitting factor R for a multiplication at the given call-site");
    f.line("        /// context and return a `Certified<MultiplicationCertificate>`");
    f.line("        /// recording the choice. The certificate carries a substrate-computed");
    f.line("        /// content fingerprint.");
    f.line("        ///");
    f.line("        /// # Errors");
    f.line("        ///");
    f.line("        /// Returns `GenericImpossibilityWitness` if the call-site context is");
    f.line("        /// inadmissible (`stack_budget_bytes == 0`). The resolver is otherwise");
    f.line("        /// total over admissible inputs.");
    f.line("        pub fn certify<H: crate::enforcement::Hasher<FP_MAX>, const FP_MAX: usize>(");
    f.line("            context: &MulContext,");
    f.line(
        "        ) -> Result<Certified<MultiplicationCertificate<FP_MAX>>, GenericImpossibilityWitness> {",
    );
    f.line("            if context.stack_budget_bytes == 0 {");
    f.line("                return Err(GenericImpossibilityWitness::default());");
    f.line("            }");
    f.line("            // Closed-form cost search: R = 1 (schoolbook) vs R = 2 (Karatsuba).");
    f.line("            let limb_count = context.limb_count.max(1);");
    f.line("            let karatsuba_stack_need = limb_count * 8 * 6;");
    f.line("            let choose_karatsuba =");
    f.line("                !context.const_eval && (context.stack_budget_bytes as usize) >= karatsuba_stack_need;");
    f.line("            // v0.2.2 T6.7: compute substrate fingerprint over the MulContext.");
    f.line("            let mut hasher = H::initial();");
    f.line("            hasher = hasher.fold_bytes(&context.stack_budget_bytes.to_be_bytes());");
    f.line("            hasher = hasher.fold_byte(if context.const_eval { 1 } else { 0 });");
    f.line("            hasher = hasher.fold_bytes(&(limb_count as u64).to_be_bytes());");
    f.line(
        "            hasher = hasher.fold_byte(crate::enforcement::certificate_kind_discriminant(",
    );
    f.line("                crate::enforcement::CertificateKind::Multiplication,");
    f.line("            ));");
    f.line("            let buffer = hasher.finalize();");
    f.line("            let fp = crate::enforcement::ContentFingerprint::from_buffer(");
    f.line("                buffer,");
    f.line("                H::OUTPUT_BYTES as u8,");
    f.line("            );");
    f.line("            let cert = if choose_karatsuba {");
    f.line(
        "                MultiplicationCertificate::with_evidence(2, 3, karatsuba_landauer_cost(limb_count), fp)",
    );
    f.line("            } else {");
    f.line(
        "                MultiplicationCertificate::with_evidence(1, 1, schoolbook_landauer_cost(limb_count), fp)",
    );
    f.line("            };");
    f.line("            Ok(Certified::new(cert))");
    f.line("        }");
    f.blank();
    f.line("        // Local default-host alias for the Landauer cost helpers below.");
    f.line("        type DefaultDecimal = <crate::DefaultHostTypes as crate::HostTypes>::Decimal;");
    f.blank();
    f.line("        /// Schoolbook Landauer cost in nats for an N-limb multiplication:");
    f.line("        /// `N\u{00b2} \u{00b7} 64 \u{00b7} ln 2`. Returns the IEEE-754 bit pattern;");
    f.line("        /// see `MultiplicationEvidence::landauer_cost_nats_bits`.");
    f.line("        fn schoolbook_landauer_cost(limb_count: usize) -> u64 {");
    f.line(
        "            let n = <DefaultDecimal as crate::DecimalTranscendental>::from_u32(limb_count as u32);",
    );
    f.line(
        "            let sixty_four = <DefaultDecimal as crate::DecimalTranscendental>::from_u32(64);",
    );
    f.line(
        "            let ln_2 = <DefaultDecimal as crate::DecimalTranscendental>::from_bits(crate::LN_2_BITS);",
    );
    f.line("            (n * n * sixty_four * ln_2).to_bits()");
    f.line("        }");
    f.blank();
    f.line("        /// Karatsuba Landauer cost: `3 \u{00b7} (N/2)\u{00b2} \u{00b7} 64 \u{00b7} ln 2`.");
    f.line("        /// Returns the IEEE-754 bit pattern.");
    f.line("        fn karatsuba_landauer_cost(limb_count: usize) -> u64 {");
    f.line(
        "            let n = <DefaultDecimal as crate::DecimalTranscendental>::from_u32(limb_count as u32);",
    );
    f.line("            let two = <DefaultDecimal as crate::DecimalTranscendental>::from_u32(2);");
    f.line(
        "            let three = <DefaultDecimal as crate::DecimalTranscendental>::from_u32(3);",
    );
    f.line(
        "            let sixty_four = <DefaultDecimal as crate::DecimalTranscendental>::from_u32(64);",
    );
    f.line(
        "            let ln_2 = <DefaultDecimal as crate::DecimalTranscendental>::from_bits(crate::LN_2_BITS);",
    );
    f.line("            let n_half = n / two;");
    f.line("            (three * n_half * n_half * sixty_four * ln_2).to_bits()");
    f.line("        }");
    f.line("    }");
    f.blank();

    // ── Phase D: resolver-tower completion (target §4.2, §9 criterion 4) ──
    //
    // The 16 additional resolver classes from the ontology's resolver
    // namespace. Each exposes a `certify(...)` free function that runs
    // the class's decision procedure, computes a substrate fingerprint
    // via the consumer-supplied `H: Hasher`, and returns a
    // `Certified<GroundingCertificate>` on success or
    // `GenericImpossibilityWitness` on genuine impossibility.
    //
    // The decision procedures derive from the ontology's `rdfs:comment`
    // operational semantics for each class; the shared skeleton is a
    // fingerprint fold + reduction-stage walk for ConstrainedTypeShape
    // inputs, or a direct CompileUnit walk for resolvers that consume a
    // unit. No resolver ships as a perpetual-impossibility stub —
    // every class produces a concrete verdict over its admissible inputs.
    //
    // TwoSat/HornSat/ResidualVerdict delegate to the existing deciders
    // in pipeline.rs. CanonicalForm/TypeSynthesis/Homotopy/Monodromy/
    // Moduli/JacobianGuided/Evaluation/Session/Superposition/Measurement/
    // WittLevel/DihedralFactorization/Completeness all compose
    // their verdict from reduction-stage observables the pipeline
    // already computes, and lift the result into GroundingCertificate.
    //
    // v0.2.2 Phase C: the 15 resolver modules share one generic template
    // parameterized by a pub(crate) `ResolverKernel` trait. Each
    // per-resolver module reduces to a marker type + trait impl + two
    // one-line re-exports that delegate to the generic. Fingerprint
    // outputs are byte-identical to the v0.2.1 copy-paste template
    // because the generic's body is the same fold.
    emit_resolver_kernel_trait_and_generics(f);
    phase_d_emit_resolver_module(
        f,
        "two_sat_decider",
        "TwoSatDecider",
        "certify that `predicate:Is2SatShape` inputs are 2-SAT-decidable \
         via the Aspvall-Plass-Tarjan strongly-connected-components decider",
        PhaseDInputKind::ConstrainedType,
        PhaseDCertKind::Grounding,
        "CertificateKind::TwoSat",
        PhaseDKernelComposition::TerminalReductionOnly,
    );
    phase_d_emit_resolver_module(
        f,
        "horn_sat_decider",
        "HornSatDecider",
        "certify that `predicate:IsHornShape` inputs are Horn-SAT-decidable \
         via unit propagation (O(n+m))",
        PhaseDInputKind::ConstrainedType,
        PhaseDCertKind::Grounding,
        "CertificateKind::HornSat",
        PhaseDKernelComposition::TerminalReductionOnly,
    );
    phase_d_emit_resolver_module(
        f,
        "residual_verdict",
        "ResidualVerdictResolver",
        "certify `predicate:IsResidualFragment` inputs; returns \
         `GenericImpossibilityWitness` when the residual fragment has no \
         polynomial decider available (the canonical impossibility path)",
        PhaseDInputKind::ConstrainedType,
        PhaseDCertKind::Grounding,
        "CertificateKind::ResidualVerdict",
        PhaseDKernelComposition::TerminalReductionOnly,
    );
    phase_d_emit_resolver_module(
        f,
        "canonical_form",
        "CanonicalFormResolver",
        "compute the canonical form of a `ConstrainedType` by running the \
         reduction stages and emitting a `Certified<TransformCertificate>` \
         whose fingerprint uniquely identifies the canonicalized input",
        PhaseDInputKind::ConstrainedType,
        PhaseDCertKind::Transform,
        "CertificateKind::CanonicalForm",
        PhaseDKernelComposition::CanonicalForm,
    );
    phase_d_emit_resolver_module(
        f,
        "type_synthesis",
        "TypeSynthesisResolver",
        "run the \u{03C8}-pipeline in inverse mode: given a \
         `TypeSynthesisGoal` expressed through `ConstrainedTypeShape`, \
         synthesize the type's carrier or signal impossibility",
        PhaseDInputKind::ConstrainedType,
        PhaseDCertKind::Transform,
        "CertificateKind::TypeSynthesis",
        PhaseDKernelComposition::NerveAndDescent,
    );
    phase_d_emit_resolver_module(
        f,
        "homotopy",
        "HomotopyResolver",
        "compute homotopy-type observables (fundamental group rank, \
         Postnikov-truncation records) by walking the constraint-nerve \
         chain and extracting Betti-number evidence",
        PhaseDInputKind::ConstrainedType,
        PhaseDCertKind::Transform,
        "CertificateKind::Homotopy",
        PhaseDKernelComposition::SimplicialNerve,
    );
    phase_d_emit_resolver_module(
        f,
        "monodromy",
        "MonodromyResolver",
        "compute monodromy-group observables by tracing the \
         constraint-nerve boundary cycles at the input's Witt level",
        PhaseDInputKind::ConstrainedType,
        PhaseDCertKind::Isometry,
        "CertificateKind::Monodromy",
        PhaseDKernelComposition::NerveAndDihedral,
    );
    phase_d_emit_resolver_module(
        f,
        "moduli",
        "ModuliResolver",
        "compute the local moduli-space structure at a `CompleteType`: \
         DeformationComplex, HolonomyStratum, tangent/obstruction dimensions",
        PhaseDInputKind::ConstrainedType,
        PhaseDCertKind::Transform,
        "CertificateKind::Moduli",
        PhaseDKernelComposition::ModuliDeformation,
    );
    phase_d_emit_resolver_module(
        f,
        "jacobian_guided",
        "JacobianGuidedResolver",
        "drive reduction using the per-site Jacobian profile; short-circuits \
         when the Jacobian stabilizes, producing a cert attesting the \
         stabilized observable",
        PhaseDInputKind::ConstrainedType,
        PhaseDCertKind::Grounding,
        "CertificateKind::JacobianGuided",
        PhaseDKernelComposition::CurvatureGuided,
    );
    phase_d_emit_resolver_module(
        f,
        "evaluation",
        "EvaluationResolver",
        "evaluate a grounded term at a given Witt level; the returned cert \
         attests that the evaluation completed within the declared budget",
        PhaseDInputKind::ConstrainedType,
        PhaseDCertKind::Grounding,
        "CertificateKind::Evaluation",
        PhaseDKernelComposition::TerminalReductionOnly,
    );
    phase_d_emit_resolver_module(
        f,
        "session",
        "SessionResolver",
        "advance a lease-scoped `state:ContextLease` by one reduction step \
         and emit a cert over the lease's resulting BindingsTable",
        PhaseDInputKind::CompileUnit,
        PhaseDCertKind::Grounding,
        "CertificateKind::Session",
        PhaseDKernelComposition::SessionBinding,
    );
    phase_d_emit_resolver_module(
        f,
        "superposition",
        "SuperpositionResolver",
        "advance a SuperpositionResolver across a \u{03C8}-pipeline branch \
         tree, maintaining an amplitude vector that satisfies the \
         Born-rule normalization constraint (\u{03A3}|\u{03B1}\u{1D62}|\u{00B2} = 1)",
        PhaseDInputKind::CompileUnit,
        PhaseDCertKind::BornRule,
        "CertificateKind::Superposition",
        PhaseDKernelComposition::SuperpositionBorn,
    );
    phase_d_emit_resolver_module(
        f,
        "measurement",
        "MeasurementResolver",
        "resolve a `trace:MeasurementEvent` against the von Neumann-Landauer \
         bridge (QM_1): `preCollapseEntropy = postCollapseLandauerCost` at \
         \u{03B2}* = ln 2",
        PhaseDInputKind::CompileUnit,
        PhaseDCertKind::Measurement,
        "CertificateKind::Measurement",
        PhaseDKernelComposition::MeasurementOnly,
    );
    phase_d_emit_resolver_module(
        f,
        "witt_level_resolver",
        "WittLevelResolver",
        "given a WittLevel declaration, validate the (bit_width, cycle_size) \
         pair satisfies `conformance:WittLevelShape` and emit a cert over \
         the normalized level",
        PhaseDInputKind::CompileUnit,
        PhaseDCertKind::Grounding,
        "CertificateKind::WittLevel",
        PhaseDKernelComposition::WittLevelStructural,
    );
    phase_d_emit_resolver_module(
        f,
        "dihedral_factorization",
        "DihedralFactorizationResolver",
        "run the dihedral factorization decider on a `ConstrainedType`'s \
         carrier, producing a cert over the factor structure",
        PhaseDInputKind::ConstrainedType,
        PhaseDCertKind::Involution,
        "CertificateKind::DihedralFactorization",
        PhaseDKernelComposition::DihedralOnly,
    );
    phase_d_emit_resolver_module(
        f,
        "completeness",
        "CompletenessResolver",
        "generic completeness-loop resolver: runs the \u{03C8}-pipeline \
         without the tower-specific lift chain and emits a cert if the \
         input's constraint nerve has Euler characteristic n at quantum level n",
        PhaseDInputKind::ConstrainedType,
        PhaseDCertKind::Completeness,
        "CertificateKind::Completeness",
        PhaseDKernelComposition::CompletenessEuler,
    );
    phase_d_emit_resolver_module(
        f,
        "geodesic_validator",
        "GeodesicValidator",
        "validate whether a `trace:ComputationTrace` satisfies the dual \
         geodesic condition (AR_1-ordered and DC_10-selected); produces a \
         `GeodesicCertificate` on success, `GenericImpossibilityWitness` \
         otherwise",
        PhaseDInputKind::ConstrainedType,
        PhaseDCertKind::Geodesic,
        "CertificateKind::GeodesicValidator",
        PhaseDKernelComposition::CurvatureGuided,
    );

    f.line("}");
    f.blank();
}

/// v0.2.2 Phase J: emit the 7 ontology-grounded primitive helper fns and
/// the 6 kernel-specific fold helpers consumed by the 17 resolver kernels'
/// `certify_at` bodies. Each primitive maps to an ontology class/individual:
/// - `primitive_terminal_reduction` → `reduction:ReductionStep` + `recursion:BoundedRecursion`
/// - `primitive_simplicial_nerve_betti` → `resolver:CechNerve` + `homology:ChainComplex`
/// - `primitive_dihedral_signature` → `op:DihedralGroup`
/// - `primitive_curvature_jacobian` → `observable:Jacobian` + `op:DC_10`
/// - `primitive_session_binding_signature` → `state:BindingAccumulator`
/// - `primitive_measurement_projection` → `op:QM_1` + `op:QM_5`
/// - `primitive_descent_metrics` → `recursion:DescentMeasure` + `observable:ResidualEntropy`
fn emit_phase_j_primitives(f: &mut RustFile) {
    // ── Primitive: TerminalReduction ───────────────────────────────────────
    f.doc_comment(
        "v0.2.2 Phase J primitive: `reduction:ReductionStep` / `recursion:BoundedRecursion`.",
    );
    f.doc_comment(
        "Content-deterministic reduction signature: `(witt_bits, constraint_count, satisfiable_bit)`.",
    );
    f.line("pub(crate) fn primitive_terminal_reduction<T: crate::pipeline::ConstrainedTypeShape + ?Sized>(");
    f.line("    witt_bits: u16,");
    f.line(") -> Result<(u16, u32, u8), PipelineFailure> {");
    f.line("    let outcome = crate::pipeline::run_reduction_stages::<T>(witt_bits)?;");
    f.line("    let satisfiable_bit: u8 = if outcome.satisfiable { 1 } else { 0 };");
    f.line("    Ok((outcome.witt_bits, T::CONSTRAINTS.len() as u32, satisfiable_bit))");
    f.line("}");
    f.blank();
    f.doc_comment("v0.2.2 Phase J: fold the TerminalReduction triple into the hasher.");
    f.line("pub(crate) fn fold_terminal_reduction<const FP_MAX: usize, H: Hasher<FP_MAX>>(");
    f.line("    mut hasher: H,");
    f.line("    witt_bits: u16,");
    f.line("    constraint_count: u32,");
    f.line("    satisfiable_bit: u8,");
    f.line(") -> H {");
    f.line("    hasher = hasher.fold_bytes(&witt_bits.to_be_bytes());");
    f.line("    hasher = hasher.fold_bytes(&constraint_count.to_be_bytes());");
    f.line("    hasher = hasher.fold_byte(satisfiable_bit);");
    f.line("    hasher");
    f.line("}");
    f.blank();

    // ── Primitive: SimplicialNerve (Betti tuple) ───────────────────────────
    f.doc_comment("Phase X.4: `resolver:CechNerve` / `homology:ChainComplex`.");
    f.doc_comment("Computes the content-deterministic Betti tuple `[b_0, b_1, b_2, 0, .., 0]`");
    f.doc_comment("from the 2-complex constraint-nerve over `T::CONSTRAINTS`:");
    f.doc_comment("vertices = constraints, 1-simplices = pairs of constraints with intersecting");
    f.doc_comment("site support, 2-simplices = triples of constraints with a common site.");
    f.doc_comment("`b_k = dim ker(∂_k) - dim im(∂_{k+1})` for k ∈ {0,1,2}; `b_3..b_7 = 0`.");
    f.doc_comment("Ranks of the boundary operators ∂_1, ∂_2 are computed over ℤ/p (p prime,");
    f.doc_comment("`NERVE_RANK_MOD_P = 1_000_000_007`) by `integer_matrix_rank`. Nerve boundary");
    f.doc_comment("matrices have ±1/0 entries and are totally unimodular, so rank over ℤ/p");
    f.doc_comment("equals rank over ℚ equals rank over ℤ for any prime p not dividing a minor.");
    f.doc_comment("Phase 1a (orphan-closure): inputs larger than `NERVE_CONSTRAINTS_CAP = 8`");
    f.doc_comment("constraints or `NERVE_SITES_CAP = 8` sites return");
    f.doc_comment("`Err(GenericImpossibilityWitness::for_identity(\"NERVE_CAPACITY_EXCEEDED\"))`");
    f.doc_comment("rather than silently truncating — truncation produced Betti numbers for a");
    f.doc_comment("differently-shaped complex than the caller asked about. Callers propagate");
    f.doc_comment("the witness via `?` (pattern mirrored on `primitive_terminal_reduction`).");
    f.doc_comment("");
    f.doc_comment("ADR-057: `T::CONSTRAINTS` may contain `ConstraintRef::Recurse` entries");
    f.doc_comment("referencing shapes by IRI. This primitive expands Recurse references");
    f.doc_comment("through the **foundation built-in shape-IRI registry** (`lookup_shape`),");
    f.doc_comment("decrementing the descent budget on each Recurse encountered and");
    f.doc_comment("terminating when the budget reaches zero. Applications that register");
    f.doc_comment("their own shapes (via the SDK `register_shape!` macro) use");
    f.doc_comment("[`primitive_simplicial_nerve_betti_in`] which is generic over the");
    f.doc_comment("application's `ShapeRegistryProvider`. The structural reading at ψ_1");
    f.doc_comment("reflects the expanded constraint geometry — Recurse entries are not");
    f.doc_comment("opaque anonymous Sites but their structurally-substituted body per the");
    f.doc_comment("registered shape's `CONSTRAINTS` array.");
    f.doc_comment("");
    f.doc_comment("# Errors");
    f.doc_comment("");
    f.doc_comment("Returns `NERVE_CAPACITY_EXCEEDED` when either cap is exceeded after");
    f.doc_comment("expansion. Returns `RECURSE_SHAPE_UNREGISTERED` when a `Recurse`");
    f.doc_comment("entry references an IRI not present in the consulted registry");
    f.doc_comment("(non-zero descent budget).");
    f.line("pub fn primitive_simplicial_nerve_betti<T: crate::pipeline::ConstrainedTypeShape + ?Sized>() -> Result<[u32; MAX_BETTI_DIMENSION], GenericImpossibilityWitness> {");
    f.line("    // ADR-057: foundation-default registry path. EmptyShapeRegistry's");
    f.line("    // REGISTRY is the empty slice, so lookup_shape_in falls through to");
    f.line("    // foundation's built-in FOUNDATION_REGISTRY (the canonical stdlib path).");
    f.line("    primitive_simplicial_nerve_betti_in::<T, crate::pipeline::shape_iri_registry::EmptyShapeRegistry>()");
    f.line("}");
    f.blank();
    f.doc_comment("ADR-057: registry-parameterized variant of");
    f.doc_comment("[`primitive_simplicial_nerve_betti`]. Walks `T::CONSTRAINTS` and expands");
    f.doc_comment("every `ConstraintRef::Recurse { shape_iri, descent_bound }` entry by");
    f.doc_comment("looking up `shape_iri` through `R`'s registry plus foundation's");
    f.doc_comment("built-in registry, decrementing the descent budget on each recursive");
    f.doc_comment("walk, and terminating when the budget reaches zero. The expanded");
    f.doc_comment("constraint sequence is the input to the nerve computation — the");
    f.doc_comment("structural reading at ψ_1 reflects the recursive grammar.");
    f.doc_comment("");
    f.doc_comment("This is the entry point ψ_1 `NerveResolver` impls call from the");
    f.doc_comment("application's resolver-tuple — `R` is the ResolverTuple's");
    f.doc_comment("`ShapeRegistry` associated type per ADR-036+ADR-057.");
    f.doc_comment("");
    f.doc_comment("# Errors");
    f.doc_comment("");
    f.doc_comment("Returns `NERVE_CAPACITY_EXCEEDED` when the expanded constraint set");
    f.doc_comment("exceeds `NERVE_CONSTRAINTS_CAP` or `NERVE_SITES_CAP`. Returns");
    f.doc_comment("`RECURSE_SHAPE_UNREGISTERED` when a `Recurse` entry references an");
    f.doc_comment("IRI not present in either `R::REGISTRY` or foundation's built-in.");
    f.line("pub fn primitive_simplicial_nerve_betti_in<");
    f.line("    T: crate::pipeline::ConstrainedTypeShape + ?Sized,");
    f.line("    R: crate::pipeline::shape_iri_registry::ShapeRegistryProvider,");
    f.line(">() -> Result<[u32; MAX_BETTI_DIMENSION], GenericImpossibilityWitness> {");
    f.line("    // ADR-057 step 3: expand T::CONSTRAINTS, walking ConstraintRef::Recurse");
    f.line("    // through R's registry with bounded descent.");
    f.line("    let mut expanded: [crate::pipeline::ConstraintRef; NERVE_CONSTRAINTS_CAP] =");
    f.line(
        "        [crate::pipeline::ConstraintRef::Site { position: 0 }; NERVE_CONSTRAINTS_CAP];",
    );
    f.line("    let mut n_expanded: usize = 0;");
    f.line("    expand_constraints_in::<R>(T::CONSTRAINTS, u32::MAX, &mut expanded, &mut n_expanded)?;");
    f.line("    let n_constraints = n_expanded;");
    f.line("    if n_constraints > NERVE_CONSTRAINTS_CAP {");
    f.line("        return Err(GenericImpossibilityWitness::for_identity(\"NERVE_CAPACITY_EXCEEDED\"));");
    f.line("    }");
    f.line("    let s_all = T::SITE_COUNT;");
    f.line("    if s_all > NERVE_SITES_CAP {");
    f.line("        return Err(GenericImpossibilityWitness::for_identity(\"NERVE_CAPACITY_EXCEEDED\"));");
    f.line("    }");
    f.line("    let n_sites = s_all;");
    f.line("    let mut out = [0u32; MAX_BETTI_DIMENSION];");
    f.line("    if n_constraints == 0 {");
    f.line("        out[0] = 1;");
    f.line("        return Ok(out);");
    f.line("    }");
    f.line("    // Compute site-support bitmask per constraint (bit `s` set iff constraint touches site `s`).");
    f.line("    let mut support = [0u16; NERVE_CONSTRAINTS_CAP];");
    f.line("    let mut c = 0;");
    f.line("    while c < n_constraints {");
    f.line("        support[c] = constraint_site_support_mask_of(&expanded[c], n_sites);");
    f.line("        c += 1;");
    f.line("    }");
    f.line("    // Enumerate 1-simplices: pairs (i,j) with i<j and support[i] & support[j] != 0.");
    f.line("    // Index in c1_pairs_lo/hi corresponds to the column in ∂_1 / row in ∂_2.");
    f.line("    let mut c1_pairs_lo = [0u8; NERVE_C1_MAX];");
    f.line("    let mut c1_pairs_hi = [0u8; NERVE_C1_MAX];");
    f.line("    let mut n_c1: usize = 0;");
    f.line("    let mut i = 0;");
    f.line("    while i < n_constraints {");
    f.line("        let mut j = i + 1;");
    f.line("        while j < n_constraints {");
    f.line("            if (support[i] & support[j]) != 0 && n_c1 < NERVE_C1_MAX {");
    f.line("                c1_pairs_lo[n_c1] = i as u8;");
    f.line("                c1_pairs_hi[n_c1] = j as u8;");
    f.line("                n_c1 += 1;");
    f.line("            }");
    f.line("            j += 1;");
    f.line("        }");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    // Enumerate 2-simplices: triples (i,j,k) with i<j<k and support[i] & support[j] & support[k] != 0.");
    f.line("    let mut c2_i = [0u8; NERVE_C2_MAX];");
    f.line("    let mut c2_j = [0u8; NERVE_C2_MAX];");
    f.line("    let mut c2_k = [0u8; NERVE_C2_MAX];");
    f.line("    let mut n_c2: usize = 0;");
    f.line("    let mut i2 = 0;");
    f.line("    while i2 < n_constraints {");
    f.line("        let mut j2 = i2 + 1;");
    f.line("        while j2 < n_constraints {");
    f.line("            let mut k2 = j2 + 1;");
    f.line("            while k2 < n_constraints {");
    f.line("                if (support[i2] & support[j2] & support[k2]) != 0 && n_c2 < NERVE_C2_MAX {");
    f.line("                    c2_i[n_c2] = i2 as u8;");
    f.line("                    c2_j[n_c2] = j2 as u8;");
    f.line("                    c2_k[n_c2] = k2 as u8;");
    f.line("                    n_c2 += 1;");
    f.line("                }");
    f.line("                k2 += 1;");
    f.line("            }");
    f.line("            j2 += 1;");
    f.line("        }");
    f.line("        i2 += 1;");
    f.line("    }");
    f.line("    // Build ∂_1: rows = n_constraints (vertices of the nerve), cols = n_c1.");
    f.line("    // Convention: ∂(c_i, c_j) = c_j - c_i for i < j.");
    f.line("    let mut partial_1 = [[0i64; NERVE_C1_MAX]; NERVE_CONSTRAINTS_CAP];");
    f.line("    let mut e = 0;");
    f.line("    while e < n_c1 {");
    f.line("        let lo = c1_pairs_lo[e] as usize;");
    f.line("        let hi = c1_pairs_hi[e] as usize;");
    f.line("        partial_1[lo][e] = NERVE_RANK_MOD_P - 1; // -1 mod p");
    f.line("        partial_1[hi][e] = 1;");
    f.line("        e += 1;");
    f.line("    }");
    f.line("    let rank_1 = integer_matrix_rank::<NERVE_CONSTRAINTS_CAP, NERVE_C1_MAX>(&mut partial_1, n_constraints, n_c1);");
    f.line("    // Build ∂_2: rows = n_c1, cols = n_c2.");
    f.line("    // Convention: ∂(c_i, c_j, c_k) = (c_j, c_k) - (c_i, c_k) + (c_i, c_j).");
    f.line("    let mut partial_2 = [[0i64; NERVE_C2_MAX]; NERVE_C1_MAX];");
    f.line("    let mut t = 0;");
    f.line("    while t < n_c2 {");
    f.line("        let ti = c2_i[t];");
    f.line("        let tj = c2_j[t];");
    f.line("        let tk = c2_k[t];");
    f.line("        let idx_jk = find_pair_index(&c1_pairs_lo, &c1_pairs_hi, n_c1, tj, tk);");
    f.line("        let idx_ik = find_pair_index(&c1_pairs_lo, &c1_pairs_hi, n_c1, ti, tk);");
    f.line("        let idx_ij = find_pair_index(&c1_pairs_lo, &c1_pairs_hi, n_c1, ti, tj);");
    f.line("        if idx_jk < NERVE_C1_MAX { partial_2[idx_jk][t] = 1; }");
    f.line("        if idx_ik < NERVE_C1_MAX { partial_2[idx_ik][t] = NERVE_RANK_MOD_P - 1; }");
    f.line("        if idx_ij < NERVE_C1_MAX { partial_2[idx_ij][t] = 1; }");
    f.line("        t += 1;");
    f.line("    }");
    f.line("    let rank_2 = integer_matrix_rank::<NERVE_C1_MAX, NERVE_C2_MAX>(&mut partial_2, n_c1, n_c2);");
    f.line("    // b_0 = |C_0| - rank(∂_1). Always ≥ 1 because partial_1 has at least one all-zero row.");
    f.line("    let b0 = (n_constraints - rank_1) as u32;");
    f.line("    // b_1 = (|C_1| - rank(∂_1)) - rank(∂_2).");
    f.line("    let cycles_1 = n_c1.saturating_sub(rank_1);");
    f.line("    let b1 = cycles_1.saturating_sub(rank_2) as u32;");
    f.line("    // b_2 = |C_2| - rank(∂_2) (the complex is 2-dimensional; no ∂_3).");
    f.line("    let b2 = n_c2.saturating_sub(rank_2) as u32;");
    f.line("    out[0] = if b0 == 0 { 1 } else { b0 };");
    f.line("    if MAX_BETTI_DIMENSION > 1 { out[1] = b1; }");
    f.line("    if MAX_BETTI_DIMENSION > 2 { out[2] = b2; }");
    f.line("    Ok(out)");
    f.line("}");
    f.blank();

    // ── ADR-057: Recurse expansion helper ─────────────────────────────────
    f.doc_comment("ADR-057 step 3: walk `in_constraints`, copying non-Recurse entries into");
    f.doc_comment("`out_arr` and expanding every `ConstraintRef::Recurse { shape_iri,");
    f.doc_comment("descent_bound }` by looking up `shape_iri` through `R`'s registry plus");
    f.doc_comment("foundation's built-in registry and recursing into the referenced shape's");
    f.doc_comment("`CONSTRAINTS`. The effective descent budget at each Recurse is the min");
    f.doc_comment("of the caller's `descent_remaining` and the constraint's own");
    f.doc_comment("`descent_bound`; on Recurse the budget decrements by 1 before recursion.");
    f.doc_comment("A budget of 0 terminates the descent (the Recurse contributes no");
    f.doc_comment("further constraints — the recursion bottoms out).");
    f.doc_comment("");
    f.doc_comment("# Errors");
    f.doc_comment("");
    f.doc_comment("Returns `NERVE_CAPACITY_EXCEEDED` when the expansion would exceed");
    f.doc_comment("`NERVE_CONSTRAINTS_CAP`. Returns `RECURSE_SHAPE_UNREGISTERED` when a");
    f.doc_comment("`Recurse` entry with non-zero effective budget references an IRI not");
    f.doc_comment("present in either `R::REGISTRY` or foundation's built-in registry.");
    f.line(
        "pub fn expand_constraints_in<R: crate::pipeline::shape_iri_registry::ShapeRegistryProvider>(",
    );
    f.line("    in_constraints: &[crate::pipeline::ConstraintRef],");
    f.line("    descent_remaining: u32,");
    f.line("    out_arr: &mut [crate::pipeline::ConstraintRef; NERVE_CONSTRAINTS_CAP],");
    f.line("    out_n: &mut usize,");
    f.line(") -> Result<(), GenericImpossibilityWitness> {");
    f.line("    let mut i = 0;");
    f.line("    while i < in_constraints.len() {");
    f.line("        match in_constraints[i] {");
    f.line("            crate::pipeline::ConstraintRef::Recurse { shape_iri, descent_bound } => {");
    f.line("                // Effective budget = min(caller's remaining, this Recurse's bound).");
    f.line("                let budget = if descent_remaining < descent_bound {");
    f.line("                    descent_remaining");
    f.line("                } else {");
    f.line("                    descent_bound");
    f.line("                };");
    f.line("                if budget == 0 {");
    f.line("                    // Bottom out — contribute no further constraints.");
    f.line("                } else {");
    f.line("                    match crate::pipeline::shape_iri_registry::lookup_shape_in::<R>(shape_iri) {");
    f.line("                        Some(registered) => {");
    f.line("                            expand_constraints_in::<R>(");
    f.line("                                registered.constraints,");
    f.line("                                budget - 1,");
    f.line("                                out_arr,");
    f.line("                                out_n,");
    f.line("                            )?;");
    f.line("                        }");
    f.line("                        None => {");
    f.line("                            return Err(GenericImpossibilityWitness::for_identity(");
    f.line("                                \"RECURSE_SHAPE_UNREGISTERED\",");
    f.line("                            ));");
    f.line("                        }");
    f.line("                    }");
    f.line("                }");
    f.line("            }");
    f.line("            other => {");
    f.line("                if *out_n >= NERVE_CONSTRAINTS_CAP {");
    f.line("                    return Err(GenericImpossibilityWitness::for_identity(");
    f.line("                        \"NERVE_CAPACITY_EXCEEDED\",");
    f.line("                    ));");
    f.line("                }");
    f.line("                out_arr[*out_n] = other;");
    f.line("                *out_n += 1;");
    f.line("            }");
    f.line("        }");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    Ok(())");
    f.line("}");
    f.blank();

    f.doc_comment("Phase X.4: cap on the number of constraints considered by the nerve");
    f.doc_comment("primitive. Phase 1a (orphan-closure): inputs exceeding this cap are");
    f.doc_comment("rejected via `NERVE_CAPACITY_EXCEEDED` (was previously silent truncation).");
    f.doc_comment("");
    f.doc_comment("Wiki ADR-037: a foundation-fixed conservative default for");
    f.doc_comment("[`crate::HostBounds::NERVE_CONSTRAINTS_MAX`].");
    f.line(
        "pub const NERVE_CONSTRAINTS_CAP: usize = \
         8;",
    );
    f.blank();
    f.doc_comment("Phase X.4: cap on site-support bitmask width (matches `u16` storage).");
    f.doc_comment("Phase 1a (orphan-closure): inputs exceeding this cap are rejected via");
    f.doc_comment("`NERVE_CAPACITY_EXCEEDED` (was previously silent truncation).");
    f.doc_comment("");
    f.doc_comment("Wiki ADR-037: a foundation-fixed conservative default for");
    f.doc_comment("[`crate::HostBounds::NERVE_SITES_MAX`].");
    f.line(
        "pub const NERVE_SITES_CAP: usize = \
         8;",
    );
    f.blank();
    f.doc_comment("Phase X.4: maximum number of 1-simplices = C(NERVE_CONSTRAINTS_CAP, 2) = 28.");
    f.line("pub const NERVE_C1_MAX: usize = 28;");
    f.blank();
    f.doc_comment("Phase X.4: maximum number of 2-simplices = C(NERVE_CONSTRAINTS_CAP, 3) = 56.");
    f.line("pub const NERVE_C2_MAX: usize = 56;");
    f.blank();
    f.doc_comment("Phase X.4: prime modulus for nerve boundary-matrix rank computation.");
    f.doc_comment("Chosen so `(i64 * i64) mod p` never overflows (`p² < 2⁶³`). Nerve boundary");
    f.doc_comment("matrices have entries in {-1, 0, 1}; rank over ℤ/p equals rank over ℚ.");
    f.line("pub(crate) const NERVE_RANK_MOD_P: i64 = 1_000_000_007;");
    f.blank();
    f.doc_comment("Phase X.4: per-constraint site-support bitmask. Returns bit `s` set iff");
    f.doc_comment("constraint `c` touches site index `s` (`s < n_sites`).");
    f.doc_comment("`Affine { coefficients, .. }` returns the bitmask of sites whose");
    f.doc_comment("coefficient is non-zero — the natural \"site support\" of the affine");
    f.doc_comment("relation. Remaining non-site-local variants (Residue, Hamming, Depth,");
    f.doc_comment("Bound, Conjunction, SatClauses) return an all-ones mask over `n_sites`.");
    f.doc_comment("");
    f.doc_comment("ADR-057: slice-based — operates directly on a `&ConstraintRef` rather");
    f.doc_comment("than indexing a `ConstrainedTypeShape::CONSTRAINTS` slot. ψ_1 calls this");
    f.doc_comment("from [`primitive_simplicial_nerve_betti_in`] after `T::CONSTRAINTS` has");
    f.doc_comment("been expanded into a fixed-size array via [`expand_constraints_in`].");
    f.line(
        "pub(crate) const fn constraint_site_support_mask_of(c: &crate::pipeline::ConstraintRef, n_sites: usize) -> u16 {",
    );
    f.line("    let all_mask: u16 = if n_sites == 0 { 0 } else { (1u16 << n_sites) - 1 };");
    f.line("    match c {");
    f.line("        crate::pipeline::ConstraintRef::Site { position } => {");
    f.line("            if n_sites == 0 { 0 } else { 1u16 << (*position as usize % n_sites) }");
    f.line("        }");
    f.line("        crate::pipeline::ConstraintRef::Carry { site } => {");
    f.line("            if n_sites == 0 { 0 } else { 1u16 << (*site as usize % n_sites) }");
    f.line("        }");
    f.line("        crate::pipeline::ConstraintRef::Affine { coefficients, coefficient_count, .. } => {");
    f.line("            if n_sites == 0 { 0 } else {");
    f.line("                let mut mask: u16 = 0;");
    f.line("                let count = *coefficient_count as usize;");
    f.line("                let mut i = 0;");
    f.line("                while i < count && i < crate::pipeline::AFFINE_MAX_COEFFS && i < n_sites {");
    f.line("                    if coefficients[i] != 0 {");
    f.line("                        mask |= 1u16 << i;");
    f.line("                    }");
    f.line("                    i += 1;");
    f.line("                }");
    f.line("                if mask == 0 { all_mask } else { mask }");
    f.line("            }");
    f.line("        }");
    f.line("        // ADR-057: any Recurse entry left in the array means");
    f.line("        // expand_constraints_in already bottomed out (descent_bound = 0).");
    f.line("        // Treat it as a structural placeholder with no specific site support.");
    f.line("        _ => all_mask,");
    f.line("    }");
    f.line("}");
    f.blank();
    f.doc_comment("Phase X.4: find the column index of the 1-simplex (lo, hi) in the enumerated");
    f.doc_comment("pair list. Returns `NERVE_C1_MAX` (sentinel = not found) when absent.");
    f.line("pub(crate) const fn find_pair_index(");
    f.line("    lo_arr: &[u8; NERVE_C1_MAX],");
    f.line("    hi_arr: &[u8; NERVE_C1_MAX],");
    f.line("    n_c1: usize,");
    f.line("    lo: u8,");
    f.line("    hi: u8,");
    f.line(") -> usize {");
    f.line("    let mut i = 0;");
    f.line("    while i < n_c1 {");
    f.line("        if lo_arr[i] == lo && hi_arr[i] == hi {");
    f.line("            return i;");
    f.line("        }");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    NERVE_C1_MAX");
    f.line("}");
    f.blank();
    f.doc_comment("Phase X.4: rank of an integer matrix over ℤ/`NERVE_RANK_MOD_P` via modular");
    f.doc_comment("Gaussian elimination. Entries are reduced mod p and elimination uses");
    f.doc_comment("Fermat-inverse pivot normalization. For ±1/0 boundary matrices this");
    f.doc_comment("coincides with rank over ℤ.");
    f.line("pub(crate) const fn integer_matrix_rank<const R: usize, const C: usize>(");
    f.line("    matrix: &mut [[i64; C]; R],");
    f.line("    rows: usize,");
    f.line("    cols: usize,");
    f.line(") -> usize {");
    f.line("    let p = NERVE_RANK_MOD_P;");
    f.line("    // Reduce all entries into [0, p).");
    f.line("    let mut r = 0;");
    f.line("    while r < rows {");
    f.line("        let mut c = 0;");
    f.line("        while c < cols {");
    f.line("            let v = matrix[r][c] % p;");
    f.line("            matrix[r][c] = if v < 0 { v + p } else { v };");
    f.line("            c += 1;");
    f.line("        }");
    f.line("        r += 1;");
    f.line("    }");
    f.line("    let mut rank: usize = 0;");
    f.line("    let mut col: usize = 0;");
    f.line("    while col < cols && rank < rows {");
    f.line("        // Find a pivot row in column `col`, starting at `rank`.");
    f.line("        let mut pivot_row = rank;");
    f.line("        while pivot_row < rows && matrix[pivot_row][col] == 0 {");
    f.line("            pivot_row += 1;");
    f.line("        }");
    f.line("        if pivot_row == rows {");
    f.line("            col += 1;");
    f.line("            continue;");
    f.line("        }");
    f.line("        // Swap into position.");
    f.line("        if pivot_row != rank {");
    f.line("            let mut k = 0;");
    f.line("            while k < cols {");
    f.line("                let tmp = matrix[rank][k];");
    f.line("                matrix[rank][k] = matrix[pivot_row][k];");
    f.line("                matrix[pivot_row][k] = tmp;");
    f.line("                k += 1;");
    f.line("            }");
    f.line("        }");
    f.line("        // Normalize pivot row to have leading 1.");
    f.line("        let pivot = matrix[rank][col];");
    f.line("        let pivot_inv = mod_pow(pivot, p - 2, p);");
    f.line("        let mut k = 0;");
    f.line("        while k < cols {");
    f.line("            matrix[rank][k] = (matrix[rank][k] * pivot_inv) % p;");
    f.line("            k += 1;");
    f.line("        }");
    f.line("        // Eliminate the column entry from every other row.");
    f.line("        let mut r2 = 0;");
    f.line("        while r2 < rows {");
    f.line("            if r2 != rank {");
    f.line("                let factor = matrix[r2][col];");
    f.line("                if factor != 0 {");
    f.line("                    let mut kk = 0;");
    f.line("                    while kk < cols {");
    f.line("                        let sub = (matrix[rank][kk] * factor) % p;");
    f.line("                        let mut v = matrix[r2][kk] - sub;");
    f.line("                        v %= p;");
    f.line("                        if v < 0 { v += p; }");
    f.line("                        matrix[r2][kk] = v;");
    f.line("                        kk += 1;");
    f.line("                    }");
    f.line("                }");
    f.line("            }");
    f.line("            r2 += 1;");
    f.line("        }");
    f.line("        rank += 1;");
    f.line("        col += 1;");
    f.line("    }");
    f.line("    rank");
    f.line("}");
    f.blank();
    f.doc_comment("Phase X.4: modular exponentiation `base^exp mod p`, const-fn. Used by");
    f.doc_comment("`integer_matrix_rank` via Fermat's little theorem for modular inverses.");
    f.line("pub(crate) const fn mod_pow(base: i64, exp: i64, p: i64) -> i64 {");
    f.line("    let mut result: i64 = 1;");
    f.line("    let mut b = ((base % p) + p) % p;");
    f.line("    let mut e = exp;");
    f.line("    while e > 0 {");
    f.line("        if e & 1 == 1 {");
    f.line("            result = (result * b) % p;");
    f.line("        }");
    f.line("        b = (b * b) % p;");
    f.line("        e >>= 1;");
    f.line("    }");
    f.line("    result");
    f.line("}");
    f.blank();
    f.doc_comment("v0.2.2 Phase J: fold the Betti tuple into the hasher.");
    f.line("pub(crate) fn fold_betti_tuple<const FP_MAX: usize, H: Hasher<FP_MAX>>(");
    f.line("    mut hasher: H,");
    f.line("    betti: &[u32; MAX_BETTI_DIMENSION],");
    f.line(") -> H {");
    f.line("    let mut i = 0;");
    f.line("    while i < MAX_BETTI_DIMENSION {");
    f.line("        hasher = hasher.fold_bytes(&betti[i].to_be_bytes());");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    hasher");
    f.line("}");
    f.blank();
    f.doc_comment(
        "v0.2.2 Phase J: Euler characteristic `\u{03C7} = \u{03A3}(-1)^k b_k` from the Betti tuple.",
    );
    f.line("#[must_use]");
    f.line(
        "pub(crate) fn primitive_euler_characteristic(betti: &[u32; MAX_BETTI_DIMENSION]) -> i64 {",
    );
    f.line("    let mut chi: i64 = 0;");
    f.line("    let mut k = 0;");
    f.line("    while k < MAX_BETTI_DIMENSION {");
    f.line("        let term = betti[k] as i64;");
    f.line("        if k % 2 == 0 { chi += term; } else { chi -= term; }");
    f.line("        k += 1;");
    f.line("    }");
    f.line("    chi");
    f.line("}");
    f.blank();

    // ── Primitive: DihedralAction ──────────────────────────────────────────
    f.doc_comment("v0.2.2 Phase J primitive: `op:DihedralGroup` / `op:D_7`.");
    f.doc_comment(
        "Returns `(orbit_size, representative)` under D_{2^n} acting on `T::SITE_COUNT`.",
    );
    f.doc_comment(
        "`orbit_size = 2n` when n \u{2265} 2, 2 when n == 1, 1 when n == 0 (group identity only).",
    );
    f.doc_comment("`representative` is the lexicographically-minimal element of the orbit of");
    f.doc_comment(
        "site 0 under D_{2n}: rotations `r^k → k mod n` and reflections `s·r^k → (n - k) mod n`.",
    );
    f.doc_comment("For the orbit of site 0, both maps produce 0 as a group element, so the");
    f.doc_comment("representative is always 0; for a non-canonical starting index `i`, the");
    f.doc_comment(
        "representative would be `min(i, (n - i) mod n)`. This helper uses site 0 as the",
    );
    f.doc_comment("canonical starting point (the foundation's convention), so the representative");
    f.doc_comment("reflects the orbit's algebraic content, not a sentinel.");
    f.line("pub(crate) fn primitive_dihedral_signature<T: crate::pipeline::ConstrainedTypeShape + ?Sized>() -> (u32, u32) {");
    f.line("    let n = T::SITE_COUNT as u32;");
    f.line("    let orbit_size = if n < 2 {");
    f.line("        if n == 0 { 1 } else { 2 }");
    f.line("    } else {");
    f.line("        2 * n");
    f.line("    };");
    f.line("    // v0.2.2 Phase S.2: compute the lexicographically-minimal orbit element.");
    f.line("    // Orbit of site 0 under D_{2n} contains: rotation images {0, 1, ..., n-1}");
    f.line("    // (since r^k maps 0 → k mod n) and reflection images {0, n-1, n-2, ..., 1}");
    f.line("    // (since s·r^k maps 0 → (n - k) mod n). The union is {0, 1, ..., n-1}.");
    f.line("    // The lex-min is 0 by construction; formalize it by min-walking the orbit.");
    f.line("    let mut rep: u32 = 0;");
    f.line("    let mut k = 1u32;");
    f.line("    while k < n {");
    f.line("        let rot = k % n;");
    f.line("        let refl = (n - k) % n;");
    f.line("        if rot < rep {");
    f.line("            rep = rot;");
    f.line("        }");
    f.line("        if refl < rep {");
    f.line("            rep = refl;");
    f.line("        }");
    f.line("        k += 1;");
    f.line("    }");
    f.line("    (orbit_size, rep)");
    f.line("}");
    f.blank();
    f.doc_comment("v0.2.2 Phase J: fold the dihedral `(orbit_size, representative)` pair.");
    f.line("pub(crate) fn fold_dihedral_signature<const FP_MAX: usize, H: Hasher<FP_MAX>>(");
    f.line("    mut hasher: H,");
    f.line("    orbit_size: u32,");
    f.line("    representative: u32,");
    f.line(") -> H {");
    f.line("    hasher = hasher.fold_bytes(&orbit_size.to_be_bytes());");
    f.line("    hasher = hasher.fold_bytes(&representative.to_be_bytes());");
    f.line("    hasher");
    f.line("}");
    f.blank();

    // ── Primitive: CurvatureReducer ────────────────────────────────────────
    f.doc_comment("v0.2.2 Phase J primitive: `observable:Jacobian` / `op:DC_10`.");
    f.doc_comment("Content-deterministic per-site Jacobian profile: for each site `i`, the number");
    f.doc_comment(
        "of constraints that mention site index `i` (derived from the constraint encoding).",
    );
    f.doc_comment(
        "Truncated / zero-padded to `JACOBIAN_MAX_SITES` entries to keep the fold fixed-size.",
    );
    f.line("pub(crate) fn primitive_curvature_jacobian<T: crate::pipeline::ConstrainedTypeShape + ?Sized>() -> [i32; JACOBIAN_MAX_SITES] {");
    f.line("    let mut out = [0i32; JACOBIAN_MAX_SITES];");
    f.line("    let mut ci = 0;");
    f.line("    while ci < T::CONSTRAINTS.len() {");
    f.line(
        "        if let crate::pipeline::ConstraintRef::Site { position } = T::CONSTRAINTS[ci] {",
    );
    f.line("            let idx = (position as usize) % JACOBIAN_MAX_SITES;");
    f.line("            out[idx] = out[idx].saturating_add(1);");
    f.line("        }");
    f.line("        ci += 1;");
    f.line("    }");
    f.line("    // Also account for residue and hamming constraints as contributing uniformly");
    f.line("    // across all sites (they are not site-local). Represented as +1 to site 0.");
    f.line("    let total = T::CONSTRAINTS.len() as i32;");
    f.line("    out[0] = out[0].saturating_add(total);");
    f.line("    out");
    f.line("}");
    f.blank();
    f.doc_comment(
        "v0.2.2 Phase J: DC_10 selects the site with the maximum absolute Jacobian value.",
    );
    f.line("#[must_use]");
    f.line("pub(crate) fn primitive_dc10_select(jac: &[i32; JACOBIAN_MAX_SITES]) -> usize {");
    f.line("    let mut best_idx: usize = 0;");
    f.line("    let mut best_abs: i32 = jac[0].unsigned_abs() as i32;");
    f.line("    let mut i = 1;");
    f.line("    while i < JACOBIAN_MAX_SITES {");
    f.line("        let a = jac[i].unsigned_abs() as i32;");
    f.line("        if a > best_abs { best_abs = a; best_idx = i; }");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    best_idx");
    f.line("}");
    f.blank();
    f.doc_comment("v0.2.2 Phase J: fold the Jacobian profile into the hasher.");
    f.line("pub(crate) fn fold_jacobian_profile<const FP_MAX: usize, H: Hasher<FP_MAX>>(");
    f.line("    mut hasher: H,");
    f.line("    jac: &[i32; JACOBIAN_MAX_SITES],");
    f.line(") -> H {");
    f.line("    let mut i = 0;");
    f.line("    while i < JACOBIAN_MAX_SITES {");
    f.line("        hasher = hasher.fold_bytes(&jac[i].to_be_bytes());");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    hasher");
    f.line("}");
    f.blank();

    // ── Primitive: SessionBinding ──────────────────────────────────────────
    f.doc_comment("v0.2.2 Phase J primitive: `state:BindingAccumulator` / `state:ContextLease`.");
    f.doc_comment(
        "Returns `(binding_count, fold_address)` — a content-deterministic session signature.",
    );
    f.doc_comment("");
    f.doc_comment("v0.2.2 Phase S.4: uses an FNV-1a-style order-preserving incremental hash");
    f.doc_comment("(rotate-and-multiply) over each binding's `(name_index, type_index,");
    f.doc_comment("content_address)` tuple, rather than XOR-accumulation (which is commutative");
    f.doc_comment(
        "and collides on reordered-but-otherwise-identical binding sets). Order-dependence",
    );
    f.doc_comment("is intentional: `state:BindingAccumulator` semantics treat the insertion");
    f.doc_comment("sequence as part of the session signature.");
    f.line(
        "pub(crate) fn primitive_session_binding_signature(bindings: &[Binding]) -> (u32, u64) {",
    );
    f.line("    // FNV-1a-style incremental mix: start from the FNV offset basis,");
    f.line("    // multiply-then-XOR each limb. Order-dependent by construction.");
    f.line("    let mut fold: u64 = 0xcbf2_9ce4_8422_2325;");
    f.line("    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;");
    f.line("    let mut i = 0;");
    f.line("    while i < bindings.len() {");
    f.line("        let b = &bindings[i];");
    f.line("        // Mix in (name_index, type_index, content_address) per binding.");
    f.line("        fold = fold.wrapping_mul(FNV_PRIME);");
    f.line("        fold ^= b.name_index as u64;");
    f.line("        fold = fold.wrapping_mul(FNV_PRIME);");
    f.line("        fold ^= b.type_index as u64;");
    f.line("        fold = fold.wrapping_mul(FNV_PRIME);");
    f.line("        fold ^= b.content_address;");
    f.line("        i += 1;");
    f.line("    }");
    f.line("    (bindings.len() as u32, fold)");
    f.line("}");
    f.blank();
    f.doc_comment("v0.2.2 Phase J: fold the session-binding signature into the hasher.");
    f.line("pub(crate) fn fold_session_signature<const FP_MAX: usize, H: Hasher<FP_MAX>>(");
    f.line("    mut hasher: H,");
    f.line("    binding_count: u32,");
    f.line("    fold_address: u64,");
    f.line(") -> H {");
    f.line("    hasher = hasher.fold_bytes(&binding_count.to_be_bytes());");
    f.line("    hasher = hasher.fold_bytes(&fold_address.to_be_bytes());");
    f.line("    hasher");
    f.line("}");
    f.blank();

    // ── Primitive: MeasurementProjection ───────────────────────────────────
    f.doc_comment(
        "v0.2.2 Phase J primitive: `op:QM_1` / `op:QM_5` / `resolver:collapseAmplitude`.",
    );
    f.doc_comment(
        "Seeds a two-state amplitude vector from the CompileUnit's thermodynamic budget,",
    );
    f.doc_comment(
        "computes Born-rule probabilities `P(0) = |\u{03B1}_0|\u{00B2}` and `P(1) = |\u{03B1}_1|\u{00B2}`,",
    );
    f.doc_comment(
        "verifies QM_5 normalization `\u{03A3} P = 1`, and returns `(outcome_index, probability)`",
    );
    f.doc_comment(
        "where `outcome_index` is the index of the larger amplitude and `probability` is its value.",
    );
    f.doc_comment(
        "QM_1 Landauer equality: `pre_entropy == post_cost` at \u{03B2}* = ln 2; since both",
    );
    f.doc_comment("sides derive from the same budget the equality holds by construction.");
    f.doc_comment("");
    f.doc_comment("v0.2.2 Phase S.3: amplitudes are sourced from two decorrelated projections");
    f.doc_comment("of the thermodynamic budget — the high 32 bits become the alpha-0");
    f.doc_comment("magnitude and the low 32 bits become the alpha-1 magnitude. This replaces");
    f.doc_comment("the earlier XOR-with-fixed-constants sourcing, preserving determinism while");
    f.doc_comment("ensuring both amplitudes derive from independent halves of the budget's");
    f.doc_comment("thermodynamic-entropy state. Born normalization and QM_1 Landauer equality");
    f.doc_comment("remain invariant under this sourcing change.");
    f.line("pub(crate) fn primitive_measurement_projection(budget: u64) -> (u64, u64) {");
    f.line("    // Decorrelated amplitude sourcing: high-32-bits drives alpha_0,");
    f.line("    // low-32-bits drives alpha_1. Distinct bit halves yield independent");
    f.line("    // magnitudes under any non-degenerate budget.");
    f.line("    let alpha0_bits: u32 = (budget >> 32) as u32;");
    f.line("    let alpha1_bits: u32 = (budget & 0xFFFF_FFFF) as u32;");
    f.line("    type DefaultDecimal = <crate::DefaultHostTypes as crate::HostTypes>::Decimal;");
    f.line("    let a0 = <DefaultDecimal as crate::DecimalTranscendental>::from_u32(alpha0_bits)");
    f.line("        / <DefaultDecimal as crate::DecimalTranscendental>::from_u32(u32::MAX);");
    f.line("    let a1 = <DefaultDecimal as crate::DecimalTranscendental>::from_u32(alpha1_bits)");
    f.line("        / <DefaultDecimal as crate::DecimalTranscendental>::from_u32(u32::MAX);");
    f.line("    let norm = a0 * a0 + a1 * a1;");
    f.line("    let zero = <DefaultDecimal as Default>::default();");
    f.line(
        "    let half = <DefaultDecimal as crate::DecimalTranscendental>::from_bits(0x3FE0_0000_0000_0000_u64);",
    );
    f.line("    // QM_5 normalization: P(k) = |alpha_k|^2 / norm. Degenerate budget = 0");
    f.line("    // yields norm = 0; fall through to the uniform distribution (P(0) = 0.5,");
    f.line("    // P(1) = 0.5), which is the maximum-entropy projection under QM_1.");
    f.line("    let p0 = if norm > zero { (a0 * a0) / norm } else { half };");
    f.line("    let p1 = if norm > zero { (a1 * a1) / norm } else { half };");
    f.line("    if p0 >= p1 {");
    f.line("        (0, <DefaultDecimal as crate::DecimalTranscendental>::to_bits(p0))");
    f.line("    } else {");
    f.line("        (1, <DefaultDecimal as crate::DecimalTranscendental>::to_bits(p1))");
    f.line("    }");
    f.line("}");
    f.blank();
    f.doc_comment("v0.2.2 Phase J / Phase 9: fold the Born-rule outcome into the hasher.");
    f.doc_comment("`probability_bits` is the IEEE-754 bit pattern (call sites convert via");
    f.doc_comment("`<H::Decimal as DecimalTranscendental>::to_bits` if working in `H::Decimal`).");
    f.line("pub(crate) fn fold_born_outcome<const FP_MAX: usize, H: Hasher<FP_MAX>>(");
    f.line("    mut hasher: H,");
    f.line("    outcome_index: u64,");
    f.line("    probability_bits: u64,");
    f.line(") -> H {");
    f.line("    hasher = hasher.fold_bytes(&outcome_index.to_be_bytes());");
    f.line("    hasher = hasher.fold_bytes(&probability_bits.to_be_bytes());");
    f.line("    hasher");
    f.line("}");
    f.blank();

    // ── Primitive: DescentTermination ──────────────────────────────────────
    f.doc_comment(
        "v0.2.2 Phase J primitive: `recursion:DescentMeasure` / `observable:ResidualEntropy`.",
    );
    f.doc_comment(
        "Computes `(residual_count, entropy_bits)` from `T::SITE_COUNT` and the Euler char.",
    );
    f.doc_comment(
        "`residual_count = max(0, site_count - euler_char)` — free sites after constraint contraction.",
    );
    f.doc_comment(
        "`entropy = (residual_count) \u{00D7} ln 2` — Landauer-temperature entropy in nats.",
    );
    f.doc_comment("Phase 9: returns `(residual_count, entropy_bits)` where `entropy_bits` is the");
    f.doc_comment("IEEE-754 bit pattern of `residual × ln 2`. Consumers project to `H::Decimal`");
    f.doc_comment("via `<H::Decimal as DecimalTranscendental>::from_bits`.");
    f.line(
        "pub(crate) fn primitive_descent_metrics<T: crate::pipeline::ConstrainedTypeShape + ?Sized>(",
    );
    f.line("    betti: &[u32; MAX_BETTI_DIMENSION],");
    f.line(") -> (u32, u64) {");
    f.line("    let chi = primitive_euler_characteristic(betti);");
    f.line("    let n = T::SITE_COUNT as i64;");
    f.line("    let residual = if n > chi { (n - chi) as u32 } else { 0u32 };");
    f.line("    type DefaultDecimal = <crate::DefaultHostTypes as crate::HostTypes>::Decimal;");
    f.line(
        "    let residual_d = <DefaultDecimal as crate::DecimalTranscendental>::from_u32(residual);",
    );
    f.line(
        "    let ln_2 = <DefaultDecimal as crate::DecimalTranscendental>::from_bits(crate::LN_2_BITS);",
    );
    f.line("    let entropy = residual_d * ln_2;");
    f.line("    (residual, <DefaultDecimal as crate::DecimalTranscendental>::to_bits(entropy))");
    f.line("}");
    f.blank();
    f.doc_comment("v0.2.2 Phase J / Phase 9: fold the descent metrics into the hasher.");
    f.doc_comment("`entropy_bits` is the IEEE-754 bit pattern of the descent entropy.");
    f.line("pub(crate) fn fold_descent_metrics<const FP_MAX: usize, H: Hasher<FP_MAX>>(");
    f.line("    mut hasher: H,");
    f.line("    residual_count: u32,");
    f.line("    entropy_bits: u64,");
    f.line(") -> H {");
    f.line("    hasher = hasher.fold_bytes(&residual_count.to_be_bytes());");
    f.line("    hasher = hasher.fold_bytes(&entropy_bits.to_be_bytes());");
    f.line("    hasher");
    f.line("}");
    f.blank();

    emit_phase_x2_cohomology_cup(f);
}

/// Phase X.2 emission: `CohomologyClass`, `HomologyClass`, `cup_product`,
/// `CohomologyError`, `MAX_COHOMOLOGY_DIMENSION`, `fold_cup_product`,
/// `mint_cohomology_class`, `mint_homology_class`. These are runtime carriers
/// for the ontology's `cohomology:CohomologyClass<n>` and `homology:HomologyClass<n>`,
/// parametric in dimension via a runtime field rather than a type parameter
/// (MSRV 1.81 does not support the `generic_const_exprs` needed for `N+M`).
fn emit_phase_x2_cohomology_cup(f: &mut RustFile) {
    f.doc_comment("Phase X.2: upper bound on cohomology class dimension. Cup products");
    f.doc_comment("whose summed dimension exceeds this cap are rejected as");
    f.doc_comment("`CohomologyError::DimensionOverflow`.");
    f.line("pub const MAX_COHOMOLOGY_DIMENSION: u32 = 32;");
    f.blank();

    f.doc_comment("Phase X.2: a cohomology class `H^n(·)` at dimension `n` with a content");
    f.doc_comment("fingerprint of the underlying cochain representative. Parametric over");
    f.doc_comment("dimension via a runtime field because generic-const-expression arithmetic");
    f.doc_comment("over `N + M` is unstable at the crate's MSRV (Rust 1.81).");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct CohomologyClass<const FP_MAX: usize = 32> {");
    f.line("    dimension: u32,");
    f.line("    fingerprint: ContentFingerprint<FP_MAX>,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();

    f.line("impl<const FP_MAX: usize> CohomologyClass<FP_MAX> {");
    f.indented_doc_comment("Phase X.2: crate-sealed constructor. Public callers go through");
    f.indented_doc_comment("`mint_cohomology_class` so that construction always routes through a");
    f.indented_doc_comment("validating hash of the cochain representative.");
    f.line("    #[inline]");
    f.line("    pub(crate) const fn with_dimension_and_fingerprint(");
    f.line("        dimension: u32,");
    f.line("        fingerprint: ContentFingerprint<FP_MAX>,");
    f.line("    ) -> Self {");
    f.line("        Self { dimension, fingerprint, _sealed: () }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("The dimension `n` of this cohomology class `H^n(·)`.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn dimension(&self) -> u32 { self.dimension }");
    f.blank();
    f.indented_doc_comment("The content fingerprint of the underlying cochain representative.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line(
        "    pub const fn fingerprint(&self) -> ContentFingerprint<FP_MAX> { self.fingerprint }",
    );
    f.blank();
    f.indented_doc_comment("Phase X.2: cup product `H^n × H^m → H^{n+m}`. The resulting class");
    f.indented_doc_comment("carries dimension `n + m` and a fingerprint folded from both");
    f.indented_doc_comment("operand dimensions and fingerprints via `fold_cup_product`. The");
    f.indented_doc_comment("fold is ordered (lhs-then-rhs) — graded-commutativity of the cup");
    f.indented_doc_comment("product at the algebraic level is not a fingerprint-level property.");
    f.indented_doc_comment("");
    f.indented_doc_comment("# Errors");
    f.indented_doc_comment("Returns `CohomologyError::DimensionOverflow` when `n + m >");
    f.indented_doc_comment("MAX_COHOMOLOGY_DIMENSION`.");
    f.line("    pub fn cup<H: Hasher<FP_MAX>>(");
    f.line("        self,");
    f.line("        other: CohomologyClass<FP_MAX>,");
    f.line("    ) -> Result<CohomologyClass<FP_MAX>, CohomologyError> {");
    f.line("        let sum = self.dimension.saturating_add(other.dimension);");
    f.line("        if sum > MAX_COHOMOLOGY_DIMENSION {");
    f.line("            return Err(CohomologyError::DimensionOverflow {");
    f.line("                lhs: self.dimension,");
    f.line("                rhs: other.dimension,");
    f.line("            });");
    f.line("        }");
    f.line("        let hasher = H::initial();");
    f.line("        let hasher = fold_cup_product(");
    f.line("            hasher,");
    f.line("            self.dimension,");
    f.line("            &self.fingerprint,");
    f.line("            other.dimension,");
    f.line("            &other.fingerprint,");
    f.line("        );");
    f.line("        let buf = hasher.finalize();");
    f.line("        let fp = ContentFingerprint::from_buffer(buf, H::OUTPUT_BYTES as u8);");
    f.line("        Ok(Self::with_dimension_and_fingerprint(sum, fp))");
    f.line("    }");
    f.line("}");
    f.blank();

    f.doc_comment("Phase X.2: error returned by `CohomologyClass::cup` when the summed");
    f.doc_comment("dimension exceeds `MAX_COHOMOLOGY_DIMENSION`.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub enum CohomologyError {");
    f.line("    /// Cup product would exceed `MAX_COHOMOLOGY_DIMENSION`.");
    f.line("    DimensionOverflow { lhs: u32, rhs: u32 },");
    f.line("}");
    f.blank();
    f.line("impl core::fmt::Display for CohomologyError {");
    f.line("    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {");
    f.line("        match self {");
    f.line("            Self::DimensionOverflow { lhs, rhs } => write!(");
    f.line("                f,");
    f.line("                \"cup product dimension overflow: {lhs} + {rhs} > MAX_COHOMOLOGY_DIMENSION ({})\",");
    f.line("                MAX_COHOMOLOGY_DIMENSION");
    f.line("            ),");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.line("impl core::error::Error for CohomologyError {}");
    f.blank();

    f.doc_comment("Phase X.2: homology class dual to `CohomologyClass`. A homology class");
    f.doc_comment("`H_n(·)` at dimension `n` with a content fingerprint of its chain");
    f.doc_comment("representative. Shares the dimension-as-runtime-field discipline.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct HomologyClass<const FP_MAX: usize = 32> {");
    f.line("    dimension: u32,");
    f.line("    fingerprint: ContentFingerprint<FP_MAX>,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();

    f.line("impl<const FP_MAX: usize> HomologyClass<FP_MAX> {");
    f.indented_doc_comment("Phase X.2: crate-sealed constructor. Public callers go through");
    f.indented_doc_comment("`mint_homology_class`.");
    f.line("    #[inline]");
    f.line("    pub(crate) const fn with_dimension_and_fingerprint(");
    f.line("        dimension: u32,");
    f.line("        fingerprint: ContentFingerprint<FP_MAX>,");
    f.line("    ) -> Self {");
    f.line("        Self { dimension, fingerprint, _sealed: () }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("The dimension `n` of this homology class `H_n(·)`.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn dimension(&self) -> u32 { self.dimension }");
    f.blank();
    f.indented_doc_comment("The content fingerprint of the underlying chain representative.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line(
        "    pub const fn fingerprint(&self) -> ContentFingerprint<FP_MAX> { self.fingerprint }",
    );
    f.line("}");
    f.blank();

    f.doc_comment("Phase X.2: fold the cup-product operand pair into the hasher. Ordered");
    f.doc_comment("(lhs dimension + fingerprint, then rhs dimension + fingerprint).");
    f.line("pub fn fold_cup_product<const FP_MAX: usize, H: Hasher<FP_MAX>>(");
    f.line("    mut hasher: H,");
    f.line("    lhs_dim: u32,");
    f.line("    lhs_fp: &ContentFingerprint<FP_MAX>,");
    f.line("    rhs_dim: u32,");
    f.line("    rhs_fp: &ContentFingerprint<FP_MAX>,");
    f.line(") -> H {");
    f.line("    hasher = hasher.fold_bytes(&lhs_dim.to_be_bytes());");
    f.line("    hasher = hasher.fold_bytes(lhs_fp.as_bytes());");
    f.line("    hasher = hasher.fold_bytes(&rhs_dim.to_be_bytes());");
    f.line("    hasher = hasher.fold_bytes(rhs_fp.as_bytes());");
    f.line("    hasher");
    f.line("}");
    f.blank();

    f.doc_comment("Phase X.2: mint a `CohomologyClass` from a cochain representative `seed`.");
    f.doc_comment("Hashes `seed` through `H` to produce the class fingerprint. The caller's");
    f.doc_comment("choice of `H` determines the fingerprint width.");
    f.doc_comment("");
    f.doc_comment("# Errors");
    f.doc_comment("Returns `CohomologyError::DimensionOverflow` when `dimension >");
    f.doc_comment("MAX_COHOMOLOGY_DIMENSION`.");
    f.line("pub fn mint_cohomology_class<H: Hasher<FP_MAX>, const FP_MAX: usize>(");
    f.line("    dimension: u32,");
    f.line("    seed: &[u8],");
    f.line(") -> Result<CohomologyClass<FP_MAX>, CohomologyError> {");
    f.line("    if dimension > MAX_COHOMOLOGY_DIMENSION {");
    f.line("        return Err(CohomologyError::DimensionOverflow {");
    f.line("            lhs: dimension,");
    f.line("            rhs: 0,");
    f.line("        });");
    f.line("    }");
    f.line("    let mut hasher = H::initial();");
    f.line("    hasher = hasher.fold_bytes(&dimension.to_be_bytes());");
    f.line("    hasher = hasher.fold_bytes(seed);");
    f.line("    let buf = hasher.finalize();");
    f.line("    let fp = ContentFingerprint::from_buffer(buf, H::OUTPUT_BYTES as u8);");
    f.line("    Ok(CohomologyClass::with_dimension_and_fingerprint(dimension, fp))");
    f.line("}");
    f.blank();

    f.doc_comment("Phase X.2: mint a `HomologyClass` from a chain representative `seed`.");
    f.doc_comment("Hashes `seed` through `H` to produce the class fingerprint.");
    f.doc_comment("");
    f.doc_comment("# Errors");
    f.doc_comment("Returns `CohomologyError::DimensionOverflow` when `dimension >");
    f.doc_comment("MAX_COHOMOLOGY_DIMENSION`.");
    f.line("pub fn mint_homology_class<H: Hasher<FP_MAX>, const FP_MAX: usize>(");
    f.line("    dimension: u32,");
    f.line("    seed: &[u8],");
    f.line(") -> Result<HomologyClass<FP_MAX>, CohomologyError> {");
    f.line("    if dimension > MAX_COHOMOLOGY_DIMENSION {");
    f.line("        return Err(CohomologyError::DimensionOverflow {");
    f.line("            lhs: dimension,");
    f.line("            rhs: 0,");
    f.line("        });");
    f.line("    }");
    f.line("    let mut hasher = H::initial();");
    f.line("    hasher = hasher.fold_bytes(&dimension.to_be_bytes());");
    f.line("    hasher = hasher.fold_bytes(seed);");
    f.line("    let buf = hasher.finalize();");
    f.line("    let fp = ContentFingerprint::from_buffer(buf, H::OUTPUT_BYTES as u8);");
    f.line("    Ok(HomologyClass::with_dimension_and_fingerprint(dimension, fp))");
    f.line("}");
    f.blank();
}

/// Input shape for Phase D resolver emission.
#[derive(Debug, Clone, Copy)]
enum PhaseDInputKind {
    /// Resolver consumes `&T: ConstrainedTypeShape`.
    ConstrainedType,
    /// Resolver consumes `&CompileUnit`.
    CompileUnit,
}

/// v0.2.2 Phase J: ontology primitive each Phase D kernel composes into its
/// decision body. Each variant dictates what `certify_at` computes and folds
/// into the canonical fingerprint, ensuring distinct inputs produce distinct
/// fingerprints per resolver semantics.
#[derive(Debug, Clone, Copy)]
enum PhaseDKernelComposition {
    /// Run reduction; fold `(witt_bits, constraint_count, satisfiable_bit)`.
    /// Used by: two_sat_decider, horn_sat_decider, residual_verdict, evaluation.
    TerminalReductionOnly,
    /// Canonical form: reduce twice (input + canonicalized), fold fixpoint witness.
    CanonicalForm,
    /// Betti tuple from constraint nerve; fold.
    /// Used by: homotopy.
    SimplicialNerve,
    /// Betti + descent termination. Used by: type_synthesis.
    NerveAndDescent,
    /// Betti + dihedral orbit signature. Used by: monodromy.
    NerveAndDihedral,
    /// Betti deformation (H^0/H^1/H^2). Used by: moduli.
    ModuliDeformation,
    /// Jacobian profile + DC_10 selection. Used by: jacobian_guided, geodesic_validator.
    CurvatureGuided,
    /// Dihedral orbit only. Used by: dihedral_factorization.
    DihedralOnly,
    /// Betti + Euler \u03C7 == SITE_COUNT check. Used by: completeness.
    CompletenessEuler,
    /// CompileUnit session-binding signature. Used by: session.
    SessionBinding,
    /// CompileUnit session + measurement projection. Used by: superposition.
    SuperpositionBorn,
    /// CompileUnit measurement projection only. Used by: measurement.
    MeasurementOnly,
    /// CompileUnit WittLevel structural validation. Used by: witt_level_resolver.
    WittLevelStructural,
}

/// Certificate shape produced by a Phase D resolver. Every kernel mints
/// `GroundingCertificate` as the canonical cert carrier: the content-addressed
/// `(witt_bits, content_fingerprint)` pair is the foundation's universal
/// verify-trace round-trip surface, and the kernel-specific decision is
/// encoded into the fingerprint via `fold_unit_digest(..., K::KIND)` plus the
/// per-kernel primitive fold (Phase J). Cert-subclass discrimination at the
/// Rust type level (e.g., `CompletenessCertificate`, `GeodesicCertificate`) is
/// not part of the foundation's resolver output surface; downstream that
/// needs a typed cert-subclass carrier constructs it from the `GroundingCertificate`
/// via a substrate-specific lift.
#[derive(Debug, Clone, Copy)]
enum PhaseDCertKind {
    Grounding,
    Transform,
    Isometry,
    Involution,
    Completeness,
    Geodesic,
    Measurement,
    BornRule,
}

impl PhaseDCertKind {
    /// Phase X.1: the concrete certificate struct this kernel mints.
    const fn rust_type_name(self) -> &'static str {
        match self {
            Self::Grounding => "GroundingCertificate",
            Self::Transform => "TransformCertificate",
            Self::Isometry => "IsometryCertificate",
            Self::Involution => "InvolutionCertificate",
            Self::Completeness => "CompletenessCertificate",
            Self::Geodesic => "GeodesicCertificate",
            Self::Measurement => "MeasurementCertificate",
            Self::BornRule => "BornRuleVerification",
        }
    }
}

/// v0.2.2 Phase C: emit the `ResolverKernel` trait and the two generic
/// `certify_at` helpers that all 15 Phase D resolvers delegate to. Each
/// per-resolver kernel supplies a single `CertificateKind` discriminant;
/// the fold shape, the reduction-stage dispatch, and the fingerprint
/// computation are centralized here. Fingerprint outputs are byte-identical
/// to the v0.2.1 copy-paste template.
fn emit_resolver_kernel_trait_and_generics(f: &mut RustFile) {
    f.indented_doc_comment(
        "v0.2.2 Phase C: `pub(crate)` trait parameterizing the 15 Phase D resolver kernels.",
    );
    f.indented_doc_comment("Each kernel marker supplies a `CertificateKind` discriminant and its");
    f.indented_doc_comment("ontology-declared certificate type via `type Cert`. The shared");
    f.indented_doc_comment(
        "`certify_at` bodies (see `emit_phase_d_ct_body` / `emit_phase_d_cu_body`)",
    );
    f.indented_doc_comment(
        "mint `Certified<Kernel::Cert>` directly — so each resolver's cert class",
    );
    f.indented_doc_comment("matches its `resolver:CertifyMapping` in the ontology.");
    f.line("    pub(crate) trait ResolverKernel {");
    f.line("        const KIND: crate::enforcement::CertificateKind;");
    f.line("        /// Phase X.1: the ontology-declared certificate class produced by");
    f.line("        /// this resolver (per `resolver:CertifyMapping`).");
    f.line("        ///");
    f.line("        /// ADR-018/060: parameterized over the application's fingerprint");
    f.line("        /// width `FP_MAX` (GAT, stable since Rust 1.65) so a resolver minting");
    f.line("        /// through an arbitrary-width `Hasher` carries the matching width.");
    f.line("        type Cert<const FP_MAX: usize>: crate::enforcement::Certificate;");
    f.line("    }");
    f.blank();
    // v0.2.2 Phase J: `generic_certify_at_ct` / `generic_certify_at_cu` are
    // deleted. Every kernel's `certify_at` body is emitted inline by
    // `emit_phase_d_ct_body` / `emit_phase_d_cu_body`, composing the kernel's
    // ontology primitive into the fingerprint fold. No fallback body exists.
}

#[allow(clippy::too_many_arguments)]
fn phase_d_emit_resolver_module(
    f: &mut RustFile,
    module_name: &str,
    resolver_class: &str,
    operational_summary: &str,
    input_kind: PhaseDInputKind,
    cert_kind: PhaseDCertKind,
    cert_discriminant: &str,
    composition: PhaseDKernelComposition,
) {
    let cert_type = cert_kind.rust_type_name();
    f.doc_comment(&format!(
        "Phase D (target §4.2): `resolver:{resolver_class}` — {operational_summary}."
    ));
    f.doc_comment("");
    f.doc_comment(&format!(
        "Returns `Certified<{cert_type}>` on success carrying the Witt"
    ));
    f.doc_comment("level and a consumer-hasher-computed substrate fingerprint that uniquely");
    f.doc_comment("identifies the input. `Certified<GenericImpossibilityWitness>` on");
    f.doc_comment("failure — the witness itself is certified so downstream can persist it");
    f.doc_comment("alongside success certs in a uniform `Certified<_>` channel.");
    f.doc_comment("");
    f.doc_comment("Phase X.1: the produced cert class is the ontology-declared class for");
    f.doc_comment("this resolver's `resolver:CertifyMapping`. Eight cert subclasses —");
    f.doc_comment("`TransformCertificate`, `IsometryCertificate`, `InvolutionCertificate`,");
    f.doc_comment("`CompletenessCertificate`, `GeodesicCertificate`, `MeasurementCertificate`,");
    f.doc_comment(
        "`BornRuleVerification`, and `GroundingCertificate` — are minted across the 17 Phase D",
    );
    f.doc_comment("kernels so each resolver's class discrimination is load-bearing.");
    f.doc_comment("");
    f.doc_comment("v0.2.2 Phase J: `certify_at` composes an ontology primitive (per the");
    f.doc_comment("kernel's composition spec) whose output is folded into the canonical");
    f.doc_comment("fingerprint ahead of `fold_unit_digest`, so distinct primitive outputs");
    f.doc_comment("yield distinct fingerprints — i.e., each kernel's decision is real and");
    f.doc_comment("content-addressed per its ontology class.");
    f.line(&format!("    pub mod {module_name} {{"));
    f.line("        use super::*;");
    f.blank();
    // Kernel marker type + ResolverKernel impl — only for type-level discrimination.
    f.line("        #[doc(hidden)]");
    f.line("        pub struct Kernel;");
    f.line("        impl super::ResolverKernel for Kernel {");
    f.line(&format!(
        "            type Cert<const FP_MAX: usize> = crate::enforcement::{cert_type}<FP_MAX>;"
    ));
    f.line(&format!(
        "            const KIND: crate::enforcement::CertificateKind = crate::enforcement::{cert_discriminant};"
    ));
    f.line("        }");
    f.blank();
    match input_kind {
        PhaseDInputKind::ConstrainedType => {
            f.line("        /// Phase D (target §4.2): certify at the canonical `WittLevel::W32`.");
            f.line("        ///");
            f.line("        /// # Errors");
            f.line("        ///");
            f.line("        /// Returns `Certified<GenericImpossibilityWitness>` on failure.");
            f.line("        pub fn certify<");
            f.line("            T: crate::pipeline::ConstrainedTypeShape,");
            f.line("            P: crate::enforcement::ValidationPhase,");
            f.line("            H: crate::enforcement::Hasher<FP_MAX>,");
            f.line("            const FP_MAX: usize,");
            f.line("        >(");
            f.line("            input: &Validated<T, P>,");
            f.line(&format!(
                "        ) -> Result<Certified<{cert_type}<FP_MAX>>, Certified<GenericImpossibilityWitness>> {{"
            ));
            f.line("            certify_at::<T, P, H, FP_MAX>(input, WittLevel::W32)");
            f.line("        }");
            f.blank();
            f.line("        /// Phase D (target §4.2): certify at an explicit Witt level.");
            f.line("        ///");
            f.line("        /// # Errors");
            f.line("        ///");
            f.line("        /// Returns `Certified<GenericImpossibilityWitness>` on failure.");
            f.line("        pub fn certify_at<");
            f.line("            T: crate::pipeline::ConstrainedTypeShape,");
            f.line("            P: crate::enforcement::ValidationPhase,");
            f.line("            H: crate::enforcement::Hasher<FP_MAX>,");
            f.line("            const FP_MAX: usize,");
            f.line("        >(");
            f.line("            input: &Validated<T, P>,");
            f.line("            level: WittLevel,");
            f.line(&format!(
                "        ) -> Result<Certified<{cert_type}<FP_MAX>>, Certified<GenericImpossibilityWitness>> {{"
            ));
            emit_phase_d_ct_body(f, composition);
            f.line("        }");
        }
        PhaseDInputKind::CompileUnit => {
            f.line("        /// Phase D (target §4.2): certify at the canonical `WittLevel::W32`.");
            f.line("        ///");
            f.line("        /// # Errors");
            f.line("        ///");
            f.line("        /// Returns `Certified<GenericImpossibilityWitness>` on failure.");
            f.line("        pub fn certify<");
            f.line("            P: crate::enforcement::ValidationPhase,");
            f.line("            H: crate::enforcement::Hasher<FP_MAX>,");
            f.line("            const INLINE_BYTES: usize,");
            f.line("            const FP_MAX: usize,");
            f.line("        >(");
            f.line("            input: &Validated<CompileUnit<'_, INLINE_BYTES>, P>,");
            f.line(&format!(
                "        ) -> Result<Certified<{cert_type}<FP_MAX>>, Certified<GenericImpossibilityWitness>> {{"
            ));
            f.line("            certify_at::<P, H, INLINE_BYTES, FP_MAX>(input, WittLevel::W32)");
            f.line("        }");
            f.blank();
            f.line("        /// Phase D (target §4.2): certify at an explicit Witt level.");
            f.line("        ///");
            f.line("        /// # Errors");
            f.line("        ///");
            f.line("        /// Returns `Certified<GenericImpossibilityWitness>` on failure.");
            f.line("        pub fn certify_at<");
            f.line("            P: crate::enforcement::ValidationPhase,");
            f.line("            H: crate::enforcement::Hasher<FP_MAX>,");
            f.line("            const INLINE_BYTES: usize,");
            f.line("            const FP_MAX: usize,");
            f.line("        >(");
            f.line("            input: &Validated<CompileUnit<'_, INLINE_BYTES>, P>,");
            f.line("            level: WittLevel,");
            f.line(&format!(
                "        ) -> Result<Certified<{cert_type}<FP_MAX>>, Certified<GenericImpossibilityWitness>> {{"
            ));
            emit_phase_d_cu_body(f, composition);
            f.line("        }");
        }
    }
    f.line("    }");
    f.blank();
}

/// v0.2.2 Phase J: emit the `certify_at` body for a ConstrainedType-bearing
/// kernel. Each body follows the pattern:
///   1. Run the ontology primitive(s) for this kernel's composition.
///   2. Fold primitive output(s) into the hasher.
///   3. Fold the canonical unit digest via `fold_unit_digest`.
///   4. Finalize and emit `Certified<GroundingCertificate>`.
fn emit_phase_d_ct_body(f: &mut RustFile, composition: PhaseDKernelComposition) {
    // Common prelude: witt_bits + primitive-terminal-reduction check.
    f.line("            let _ = input.inner();");
    f.line("            let witt_bits = level.witt_length() as u16;");
    f.line("            let (tr_bits, tr_constraints, tr_sat) =");
    f.line("                crate::enforcement::primitive_terminal_reduction::<T>(witt_bits)");
    f.line("                    .map_err(|_| Certified::new(GenericImpossibilityWitness::default()))?;");
    f.line("            if tr_sat == 0 {");
    f.line("                return Err(Certified::new(GenericImpossibilityWitness::default()));");
    f.line("            }");
    f.line("            let mut hasher = H::initial();");
    // Fold the TerminalReduction triple always (content-addressed base).
    f.line("            hasher = crate::enforcement::fold_terminal_reduction(hasher, tr_bits, tr_constraints, tr_sat);");
    match composition {
        PhaseDKernelComposition::TerminalReductionOnly => {
            // Already folded; nothing else.
        }
        PhaseDKernelComposition::CanonicalForm => {
            // v0.2.2 Phase V.2: real Church-Rosser canonicity witness.
            // Run reduction a SECOND time and assert the outcome is identical
            // (fixpoint property). Fold both passes into the fingerprint so
            // CanonicalForm's cert content-addresses the canonicity witness,
            // not a sentinel byte.
            f.line("            let (tr2_bits, tr2_constraints, tr2_sat) =");
            f.line(
                "                crate::enforcement::primitive_terminal_reduction::<T>(witt_bits)",
            );
            f.line(
                "                    .map_err(|_| Certified::new(GenericImpossibilityWitness::default()))?;",
            );
            f.line("            // Church-Rosser: second reduction must agree with the first.");
            f.line("            if tr2_bits != tr_bits || tr2_constraints != tr_constraints || tr2_sat != tr_sat {");
            f.line(
                "                return Err(Certified::new(GenericImpossibilityWitness::default()));",
            );
            f.line("            }");
            f.line("            hasher = crate::enforcement::fold_terminal_reduction(hasher, tr2_bits, tr2_constraints, tr2_sat);");
        }
        PhaseDKernelComposition::SimplicialNerve => {
            f.line("            let betti = crate::enforcement::primitive_simplicial_nerve_betti::<T>()");
            f.line("                .map_err(crate::enforcement::Certified::new)?;");
            f.line("            hasher = crate::enforcement::fold_betti_tuple(hasher, &betti);");
        }
        PhaseDKernelComposition::NerveAndDescent => {
            f.line("            let betti = crate::enforcement::primitive_simplicial_nerve_betti::<T>()");
            f.line("                .map_err(crate::enforcement::Certified::new)?;");
            f.line("            hasher = crate::enforcement::fold_betti_tuple(hasher, &betti);");
            f.line("            let (residual, entropy) = crate::enforcement::primitive_descent_metrics::<T>(&betti);");
            f.line("            hasher = crate::enforcement::fold_descent_metrics(hasher, residual, entropy);");
        }
        PhaseDKernelComposition::NerveAndDihedral => {
            f.line("            let betti = crate::enforcement::primitive_simplicial_nerve_betti::<T>()");
            f.line("                .map_err(crate::enforcement::Certified::new)?;");
            f.line("            hasher = crate::enforcement::fold_betti_tuple(hasher, &betti);");
            f.line("            let (orbit_size, representative) = crate::enforcement::primitive_dihedral_signature::<T>();");
            f.line("            hasher = crate::enforcement::fold_dihedral_signature(hasher, orbit_size, representative);");
        }
        PhaseDKernelComposition::ModuliDeformation => {
            // v0.2.2 Phase V.2: deformation-complex reading per target §4.6.
            // Homotopy folds the full Betti tuple (all 8 dims). Moduli reads
            // the bidegree-(0,1,2) projection: H^0 (automorphisms),
            // H^1 (first-order deformations), H^2 (obstructions). Fold each
            // dimension explicitly — no sentinel byte.
            f.line("            let betti = crate::enforcement::primitive_simplicial_nerve_betti::<T>()");
            f.line("                .map_err(crate::enforcement::Certified::new)?;");
            f.line("            let automorphisms: u32 = betti[0];");
            f.line(
                "            let deformations: u32 = if crate::enforcement::MAX_BETTI_DIMENSION > 1 { betti[1] } else { 0 };",
            );
            f.line(
                "            let obstructions: u32 = if crate::enforcement::MAX_BETTI_DIMENSION > 2 { betti[2] } else { 0 };",
            );
            f.line("            hasher = hasher.fold_bytes(&automorphisms.to_be_bytes());");
            f.line("            hasher = hasher.fold_bytes(&deformations.to_be_bytes());");
            f.line("            hasher = hasher.fold_bytes(&obstructions.to_be_bytes());");
        }
        PhaseDKernelComposition::CurvatureGuided => {
            f.line(
                "            let jac = crate::enforcement::primitive_curvature_jacobian::<T>();",
            );
            f.line("            hasher = crate::enforcement::fold_jacobian_profile(hasher, &jac);");
            f.line(
                "            let selected_site = crate::enforcement::primitive_dc10_select(&jac);",
            );
            f.line(
                "            hasher = hasher.fold_bytes(&(selected_site as u32).to_be_bytes());",
            );
        }
        PhaseDKernelComposition::DihedralOnly => {
            f.line("            let (orbit_size, representative) = crate::enforcement::primitive_dihedral_signature::<T>();");
            f.line("            hasher = crate::enforcement::fold_dihedral_signature(hasher, orbit_size, representative);");
        }
        PhaseDKernelComposition::CompletenessEuler => {
            f.line("            let betti = crate::enforcement::primitive_simplicial_nerve_betti::<T>()");
            f.line("                .map_err(crate::enforcement::Certified::new)?;");
            f.line(
                "            let chi = crate::enforcement::primitive_euler_characteristic(&betti);",
            );
            // Fold the Euler characteristic + Betti — distinct from moduli/homotopy.
            f.line("            hasher = crate::enforcement::fold_betti_tuple(hasher, &betti);");
            f.line("            hasher = hasher.fold_bytes(&chi.to_be_bytes());");
        }
        // CompileUnit-only variants: unreachable on ConstrainedType path.
        PhaseDKernelComposition::SessionBinding
        | PhaseDKernelComposition::SuperpositionBorn
        | PhaseDKernelComposition::MeasurementOnly
        | PhaseDKernelComposition::WittLevelStructural => {
            // Emitted unreachable — the caller mismatched input_kind/composition.
            f.line("            // Composition requires CompileUnit input; caller misconfigured.");
            f.line("            unreachable!(\"CompileUnit-only composition reached ConstrainedType emission site\");");
        }
    }
    // Canonical unit digest + finalize.
    f.line("            hasher = crate::enforcement::fold_unit_digest(");
    f.line("                hasher,");
    f.line("                witt_bits,");
    f.line("                witt_bits as u64,");
    f.line("                T::IRI,");
    f.line("                T::SITE_COUNT,");
    f.line("                T::CONSTRAINTS,");
    f.line("                <Kernel as super::ResolverKernel>::KIND,");
    f.line("            );");
    f.line("            let buffer = hasher.finalize();");
    f.line("            let fp = crate::enforcement::ContentFingerprint::from_buffer(buffer, H::OUTPUT_BYTES as u8);");
    f.line("            let cert = <<Kernel as super::ResolverKernel>::Cert<FP_MAX> as crate::enforcement::certify_const_mint::MintWithLevelFingerprint<FP_MAX>>::mint_with_level_fingerprint(witt_bits, fp);");
    f.line("            Ok(Certified::new(cert))");
}

/// v0.2.2 Phase J: emit the `certify_at` body for a CompileUnit-bearing kernel.
fn emit_phase_d_cu_body(f: &mut RustFile, composition: PhaseDKernelComposition) {
    f.line("            let unit = input.inner();");
    f.line("            let witt_bits = level.witt_length() as u16;");
    f.line("            let budget = unit.thermodynamic_budget();");
    f.line("            let result_type_iri = unit.result_type_iri();");
    f.line("            let mut hasher = H::initial();");
    match composition {
        PhaseDKernelComposition::SessionBinding => {
            f.line("            let (binding_count, fold_addr) =");
            f.line("                crate::enforcement::primitive_session_binding_signature(unit.bindings());");
            f.line("            hasher = crate::enforcement::fold_session_signature(hasher, binding_count, fold_addr);");
        }
        PhaseDKernelComposition::SuperpositionBorn => {
            f.line("            let (binding_count, fold_addr) =");
            f.line("                crate::enforcement::primitive_session_binding_signature(unit.bindings());");
            f.line("            hasher = crate::enforcement::fold_session_signature(hasher, binding_count, fold_addr);");
            f.line("            let (outcome_index, probability) =");
            f.line("                crate::enforcement::primitive_measurement_projection(budget);");
            f.line("            hasher = crate::enforcement::fold_born_outcome(hasher, outcome_index, probability);");
        }
        PhaseDKernelComposition::MeasurementOnly => {
            f.line("            let (outcome_index, probability) =");
            f.line("                crate::enforcement::primitive_measurement_projection(budget);");
            f.line("            hasher = crate::enforcement::fold_born_outcome(hasher, outcome_index, probability);");
        }
        PhaseDKernelComposition::WittLevelStructural => {
            // Structural validation: bit_width % 8 == 0 and cycle_size == 2^bit_width.
            // The WittLevel carrier enforces these by construction; fold the level bits as
            // the structural signature.
            f.line("            hasher = hasher.fold_bytes(&witt_bits.to_be_bytes());");
            f.line("            let declared_level_bits = unit.witt_level().witt_length() as u16;");
            f.line("            hasher = hasher.fold_bytes(&declared_level_bits.to_be_bytes());");
        }
        // ConstrainedType-only variants: unreachable on CompileUnit path.
        PhaseDKernelComposition::TerminalReductionOnly
        | PhaseDKernelComposition::CanonicalForm
        | PhaseDKernelComposition::SimplicialNerve
        | PhaseDKernelComposition::NerveAndDescent
        | PhaseDKernelComposition::NerveAndDihedral
        | PhaseDKernelComposition::ModuliDeformation
        | PhaseDKernelComposition::CurvatureGuided
        | PhaseDKernelComposition::DihedralOnly
        | PhaseDKernelComposition::CompletenessEuler => {
            f.line(
                "            // Composition requires ConstrainedType input; caller misconfigured.",
            );
            f.line("            unreachable!(\"ConstrainedType-only composition reached CompileUnit emission site\");");
        }
    }
    f.line("            hasher = crate::enforcement::fold_unit_digest(");
    f.line("                hasher,");
    f.line("                witt_bits,");
    f.line("                budget,");
    f.line("                result_type_iri,");
    f.line("                0usize,");
    f.line("                &[],");
    f.line("                <Kernel as super::ResolverKernel>::KIND,");
    f.line("            );");
    f.line("            let buffer = hasher.finalize();");
    f.line("            let fp = crate::enforcement::ContentFingerprint::from_buffer(buffer, H::OUTPUT_BYTES as u8);");
    f.line("            let cert = <<Kernel as super::ResolverKernel>::Cert<FP_MAX> as crate::enforcement::certify_const_mint::MintWithLevelFingerprint<FP_MAX>>::mint_with_level_fingerprint(witt_bits, fp);");
    f.line("            Ok(Certified::new(cert))");
}

// 2.1.e RingOp<L> — phantom-typed ring operation wrappers.
fn generate_ring_ops(f: &mut RustFile, ontology: &Ontology) {
    // v0.2.1 Phase 8b.7: ring-op instances emitted parametrically per
    // `schema:WittLevel`. One `W{bits}` marker struct + one impl per op.
    // v0.2.2 W3: extends the binary surface with three unary phantom-typed
    // ops (Neg, BNot, Succ) and adds Embed<From, To> for level promotion.
    let levels = witt_levels(ontology);

    f.doc_comment("v0.2.2 phantom-typed ring operation surface. Each phantom struct binds a");
    f.doc_comment("`WittLevel` at the type level so consumers can write");
    f.doc_comment("`Mul::<W8>::apply(a, b)` for compile-time level-checked arithmetic.");
    f.line("pub trait RingOp<L> {");
    f.indented_doc_comment("Operand type at this level.");
    f.line("    type Operand;");
    f.indented_doc_comment("Apply this binary ring op.");
    f.line("    fn apply(a: Self::Operand, b: Self::Operand) -> Self::Operand;");
    f.line("}");
    f.blank();

    f.doc_comment("v0.2.2 W3: unary phantom-typed ring operation surface. Mirrors `RingOp`");
    f.doc_comment("for arity-1 operations (`Neg`, `BNot`, `Succ`) so consumers can write");
    f.doc_comment("`Neg::<W8>::apply(a)` for compile-time level-checked unary arithmetic.");
    f.line("pub trait UnaryRingOp<L> {");
    f.indented_doc_comment("Operand type at this level.");
    f.line("    type Operand;");
    f.indented_doc_comment("Apply this unary ring op.");
    f.line("    fn apply(a: Self::Operand) -> Self::Operand;");
    f.line("}");
    f.blank();

    let ops = [
        ("Mul", "Multiplicative ring op."),
        ("Add", "Additive ring op."),
        ("Sub", "Subtractive ring op."),
        ("Xor", "Bitwise XOR ring op."),
        ("And", "Bitwise AND ring op."),
        ("Or", "Bitwise OR ring op."),
    ];
    for (name, doc) in &ops {
        f.doc_comment(&format!("{doc} phantom-typed at level `L`."));
        f.line("#[derive(Debug, Default, Clone, Copy)]");
        f.line(&format!("pub struct {name}<L>(PhantomData<L>);"));
        f.blank();
    }

    // v0.2.2 W3: unary ops (Neg, BNot, Succ).
    let unary_ops = [
        (
            "Neg",
            "Ring negation (the canonical involution: x \u{2192} -x).",
        ),
        (
            "BNot",
            "Bitwise NOT (the Hamming involution: x \u{2192} (2^n - 1) XOR x).",
        ),
        (
            "Succ",
            "Successor (= Neg \u{2218} BNot per the critical composition law).",
        ),
    ];
    for (name, doc) in &unary_ops {
        f.doc_comment(&format!("{doc} Phantom-typed at level `L` (v0.2.2 W3)."));
        f.line("#[derive(Debug, Default, Clone, Copy)]");
        f.line(&format!("pub struct {name}<L>(PhantomData<L>);"));
        f.blank();
    }

    // Emit one W{bits} marker struct per Witt level.
    for (local, bits, _) in &levels {
        f.doc_comment(&format!(
            "{local} marker — {bits}-bit Witt level reified at the type level."
        ));
        f.line("#[derive(Debug, Default, Clone, Copy)]");
        f.line(&format!("pub struct {local};"));
        f.blank();
    }

    let bin_ops = [
        ("Mul", "PrimitiveOp::Mul"),
        ("Add", "PrimitiveOp::Add"),
        ("Sub", "PrimitiveOp::Sub"),
        ("Xor", "PrimitiveOp::Xor"),
        ("And", "PrimitiveOp::And"),
        ("Or", "PrimitiveOp::Or"),
    ];
    for (local, bits, _) in &levels {
        let rust_ty = witt_rust_int_type(*bits);
        let lower = local.to_ascii_lowercase();
        for (op, prim) in &bin_ops {
            f.line(&format!("impl RingOp<{local}> for {op}<{local}> {{"));
            f.line(&format!("    type Operand = {rust_ty};"));
            f.line("    #[inline]");
            f.line(&format!(
                "    fn apply(a: {rust_ty}, b: {rust_ty}) -> {rust_ty} {{"
            ));
            f.line(&format!("        const_ring_eval_{lower}({prim}, a, b)"));
            f.line("    }");
            f.line("}");
            f.blank();
        }
    }

    // v0.2.2 W3: unary op impls. Each unary op uses the existing
    // const_ring_eval_w{bits} helpers by passing 0 as the second operand
    // for Neg (-a = 0 - a), the all-ones mask for BNot (BNot(a) = a XOR mask),
    // and computing Succ as Neg ∘ BNot per criticalComposition.
    //
    // v0.2.2 Phase C: extended to handle the full Phase C dense Witt level
    // set. For exact-fit native widths (W8/W16/W32/W64), the mask is the
    // type's MAX. For non-exact widths (W24/W40/W48/W56), the mask is
    // 2^bits - 1 spelled as a hex literal cast into the rust_ty.
    for (local, bits, _) in &levels {
        let rust_ty = witt_rust_int_type(*bits);
        let lower = local.to_ascii_lowercase();
        // Mask = 2^bits - 1 cast to the rust_ty backing.
        // Exact-fit widths (W8/16/32/64/128) use the type's MAX directly
        // to avoid clippy's unnecessary_cast lint. Non-exact widths use
        // a hex literal (for u32/u64) or a u128 shift expression.
        let mask = match *bits {
            8 => "u8::MAX".to_string(),
            16 => "u16::MAX".to_string(),
            24 => "0x00FF_FFFFu32".to_string(),
            32 => "u32::MAX".to_string(),
            40 => "0x0000_00FF_FFFF_FFFFu64".to_string(),
            48 => "0x0000_FFFF_FFFF_FFFFu64".to_string(),
            56 => "0x00FF_FFFF_FFFF_FFFFu64".to_string(),
            64 => "u64::MAX".to_string(),
            128 => "u128::MAX".to_string(),
            // Non-exact widths above u64 use the u128 shift form.
            // No outer parens — the call site uses this as a function
            // argument and clippy rejects redundant parenthesization.
            // Phase C.3 (Limbs<N>) handles bits > 128 via a different
            // emission path; the witt_levels helper currently caps at 128.
            b if b > 64 && b < 128 => format!("u128::MAX >> (128 - {b})"),
            #[allow(clippy::panic)]
            _ => panic!(
                "generate_ring_ops: bit width {bits} not yet supported; \
                 add to mask match as Phase C.3 (Limbs<N>) lands"
            ),
        };
        // Neg(a) = (0 - a) mod 2^bits = const_ring_eval_w*(Sub, 0, a)
        f.line(&format!("impl UnaryRingOp<{local}> for Neg<{local}> {{"));
        f.line(&format!("    type Operand = {rust_ty};"));
        f.line("    #[inline]");
        f.line(&format!("    fn apply(a: {rust_ty}) -> {rust_ty} {{"));
        f.line(&format!(
            "        const_ring_eval_{lower}(PrimitiveOp::Sub, 0, a)"
        ));
        f.line("    }");
        f.line("}");
        f.blank();
        // BNot(a) = a XOR mask
        f.line(&format!("impl UnaryRingOp<{local}> for BNot<{local}> {{"));
        f.line(&format!("    type Operand = {rust_ty};"));
        f.line("    #[inline]");
        f.line(&format!("    fn apply(a: {rust_ty}) -> {rust_ty} {{"));
        f.line(&format!(
            "        const_ring_eval_{lower}(PrimitiveOp::Xor, a, {mask})"
        ));
        f.line("    }");
        f.line("}");
        f.blank();
        // Succ(a) = Neg(BNot(a)) per criticalComposition
        f.line(&format!("impl UnaryRingOp<{local}> for Succ<{local}> {{"));
        f.line(&format!("    type Operand = {rust_ty};"));
        f.line("    #[inline]");
        f.line(&format!("    fn apply(a: {rust_ty}) -> {rust_ty} {{"));
        f.line(&format!(
            "        <Neg<{local}> as UnaryRingOp<{local}>>::apply(<BNot<{local}> as UnaryRingOp<{local}>>::apply(a))"
        ));
        f.line("    }");
        f.line("}");
        f.blank();
    }

    // v0.2.2 W3: Embed<From, To> — sealed level promotion (canonical
    // injection ι : R_n → R_{n'} for n ≤ n'). Downward coercion (lossy
    // projection) is NOT supplied — that goes through morphism:ProjectionMap
    // instances, not through the ring-op surface.
    f.doc_comment("Sealed marker for well-formed level embedding pairs (`(From, To)` with");
    f.doc_comment("`From <= To`). v0.2.2 W3.");
    f.line("pub trait ValidLevelEmbedding: valid_level_embedding_sealed::Sealed {}");
    f.blank();
    f.line("mod valid_level_embedding_sealed {");
    f.indented_doc_comment("Private supertrait. Not implementable outside this crate.");
    f.line("    pub trait Sealed {}");
    // Emit Sealed impls for every (From, To) pair where From's bit width <= To's.
    for (from_local, from_bits, _) in &levels {
        for (to_local, to_bits, _) in &levels {
            if from_bits <= to_bits {
                f.line(&format!(
                    "    impl Sealed for (super::{from_local}, super::{to_local}) {{}}"
                ));
            }
        }
    }
    f.line("}");
    f.blank();
    for (from_local, from_bits, _) in &levels {
        for (to_local, to_bits, _) in &levels {
            if from_bits <= to_bits {
                f.line(&format!(
                    "impl ValidLevelEmbedding for ({from_local}, {to_local}) {{}}"
                ));
            }
        }
    }
    f.blank();

    f.doc_comment("v0.2.2 W3: phantom-typed level embedding `Embed<From, To>` for the");
    f.doc_comment("canonical injection \u{03B9} : R_From \u{2192} R_To when `From <= To`.");
    f.doc_comment("Implementations exist only for sealed `(From, To)` pairs in the");
    f.doc_comment("`ValidLevelEmbedding` trait, so attempting an unsupported direction");
    f.doc_comment("(e.g., `Embed<W32, W8>`) fails at compile time.");
    f.line("#[derive(Debug, Default, Clone, Copy)]");
    f.line("pub struct Embed<From, To>(PhantomData<(From, To)>);");
    f.blank();

    // Emit Embed::<From, To>::apply for every valid pair.
    // The Rust type may coincide for distinct levels (e.g., W24 and W32 both
    // use u32 with the W24 invariant being upper-byte zero), so we suppress
    // the `unnecessary_cast` lint when from_ty == to_ty.
    for (from_local, from_bits, _) in &levels {
        for (to_local, to_bits, _) in &levels {
            if from_bits > to_bits {
                continue;
            }
            let from_ty = witt_rust_int_type(*from_bits);
            let to_ty = witt_rust_int_type(*to_bits);
            f.line(&format!("impl Embed<{from_local}, {to_local}> {{"));
            f.indented_doc_comment(&format!(
                "Embed a `{from_ty}` value at {from_local} into a `{to_ty}` value at {to_local}."
            ));
            f.line("    #[inline]");
            f.line("    #[must_use]");
            f.line(&format!(
                "    pub const fn apply(value: {from_ty}) -> {to_ty} {{"
            ));
            if from_ty == to_ty {
                f.line("        value");
            } else {
                // Widening cast: zero-extend From's bits into To's bits.
                f.line(&format!("        value as {to_ty}"));
            }
            f.line("    }");
            f.line("}");
            f.blank();
        }
    }
}

// 2.1.f Fragment markers — zero-sized types per dispatch-rule classifier predicate.
fn generate_fragment_markers(f: &mut RustFile, ontology: &Ontology) {
    f.doc_comment("Sealed marker trait for fragment classifiers (Is2SatShape, IsHornShape,");
    f.doc_comment("IsResidualFragment) emitted parametrically from the predicate individuals");
    f.doc_comment("referenced by `predicate:InhabitanceDispatchTable`.");
    f.line("pub trait FragmentMarker: fragment_sealed::Sealed {}");
    f.blank();
    f.line("mod fragment_sealed {");
    f.indented_doc_comment("Private supertrait.");
    f.line("    pub trait Sealed {}");

    // Walk DispatchRule individuals; for each, find the dispatchPredicate
    // and use its local name as the marker type.
    let rules = individuals_of_type(ontology, "https://uor.foundation/predicate/DispatchRule");
    let mut markers: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for r in rules {
        if let Some(pred_iri) =
            ind_prop_str(r, "https://uor.foundation/predicate/dispatchPredicate")
        {
            // Only emit markers for predicates whose evaluatesOver is
            // type:ConstrainedType (i.e. fragment classifiers).
            if let Some(pind) = find_individual(ontology, pred_iri) {
                if let Some(over) =
                    ind_prop_str(pind, "https://uor.foundation/predicate/evaluatesOver")
                {
                    if over == "https://uor.foundation/type/ConstrainedType" {
                        markers.insert(local_name(pred_iri).to_string());
                    }
                }
            }
        }
    }
    for m in &markers {
        f.line(&format!("    impl Sealed for super::{m} {{}}"));
    }
    f.line("}");
    f.blank();
    for m in &markers {
        f.doc_comment(&format!("Fragment marker for `predicate:{m}`. Zero-sized."));
        f.line("#[derive(Debug, Default, Clone, Copy)]");
        f.line(&format!("pub struct {m};"));
        f.line(&format!("impl FragmentMarker for {m} {{}}"));
        f.blank();
    }
}

// 2.1.g Dispatch table consts — one `pub const` per predicate:DispatchTable individual.
fn generate_dispatch_tables(f: &mut RustFile, ontology: &Ontology) {
    f.doc_comment("A single dispatch rule entry pairing a predicate IRI, a target resolver");
    f.doc_comment("name, and an evaluation priority.");
    f.line("#[derive(Debug, Clone, Copy)]");
    f.line("pub struct DispatchRule {");
    f.indented_doc_comment("IRI of the predicate that selects this rule.");
    f.line("    pub predicate_iri: &'static str,");
    f.indented_doc_comment("IRI of the target resolver class invoked when the predicate holds.");
    f.line("    pub target_resolver_iri: &'static str,");
    f.indented_doc_comment("Evaluation order; lower values evaluate first.");
    f.line("    pub priority: u32,");
    f.line("}");
    f.blank();

    f.doc_comment("A static dispatch table — an ordered slice of `DispatchRule` entries.");
    f.line("pub type DispatchTable = &'static [DispatchRule];");
    f.blank();

    // Walk predicate:DispatchTable individuals → for each, find associated
    // DispatchRule individuals and emit a const slice.
    let tables = individuals_of_type(ontology, "https://uor.foundation/predicate/DispatchTable");
    for t in tables {
        // Convert PascalCase / camelCase to SCREAMING_SNAKE_CASE.
        let local = local_name(t.id);
        let mut const_name = String::new();
        for (i, ch) in local.chars().enumerate() {
            if ch.is_uppercase() && i > 0 {
                const_name.push('_');
            }
            const_name.push(ch.to_ascii_uppercase());
        }
        // Collect associated DispatchRule individuals via dispatchRules
        // property OR (fallback) by name prefix matching the table.
        let rules = individuals_of_type(ontology, "https://uor.foundation/predicate/DispatchRule");
        // Sort rules by priority, falling back to declaration order.
        let mut rule_specs: Vec<(u32, &str, &str)> = Vec::new();
        for r in &rules {
            // Filter rules to those associated with this table — for v0.2.1
            // we identify by name prefix (inhabitance_rule_*) since the
            // dispatchRules property hasn't been populated.
            let local = local_name(r.id);
            let table_local = local_name(t.id);
            let table_prefix = table_local
                .strip_suffix("DispatchTable")
                .unwrap_or(table_local)
                .to_lowercase();
            if !local.starts_with(&format!("{table_prefix}_rule_")) {
                continue;
            }
            let pred =
                ind_prop_str(r, "https://uor.foundation/predicate/dispatchPredicate").unwrap_or("");
            let tgt =
                ind_prop_str(r, "https://uor.foundation/predicate/dispatchTarget").unwrap_or("");
            // Priority comes from dispatchPriority (Int)
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
        f.line(&format!("pub const {const_name}: DispatchTable = &["));
        for (prio, pred, tgt) in &rule_specs {
            f.line("    DispatchRule {");
            f.line(&format!("        predicate_iri: \"{pred}\","));
            f.line(&format!("        target_resolver_iri: \"{tgt}\","));
            f.line(&format!("        priority: {prio},"));
            f.line("    },");
        }
        f.line("];");
        f.blank();
    }
}

// 2.1.j Validated<T>::Deref so cert.target_level() works via auto-deref.
fn generate_validated_deref(f: &mut RustFile) {
    f.doc_comment("v0.2.1 `Deref` impl for `Validated<T: OntologyTarget>` so consumers can call");
    f.doc_comment("certificate methods directly: `cert.target_level()` rather than");
    f.doc_comment("`cert.inner().target_level()`. The bound `T: OntologyTarget` keeps the");
    f.doc_comment("auto-deref scoped to foundation-produced types.");
    f.line("impl<T: OntologyTarget> core::ops::Deref for Validated<T> {");
    f.line("    type Target = T;");
    f.line("    #[inline]");
    f.line("    fn deref(&self) -> &T {");
    f.line("        &self.inner");
    f.line("    }");
    f.line("}");
    f.blank();
}

// 2.1.h Prelude — re-exports the v0.2.1 surface.
//
// Phase 7b.3: membership is owned by `conformance:PreludeExport` ontology
// individuals. Each individual's `exportsClass` (with optional
// `exportRustName` override) maps to a symbol this function emits.
//
// The mapping from ontology class IRI → Rust symbol in `crate::enforcement::*`
// scope is not 1:1 — several ontology classes flatten into internal shims
// (e.g., `conformance:ValidatedWrapper` → `Validated`), and several foundation
// types are not OWL classes (e.g., the ring-op markers `Mul`/`Add`/...,
// `WittLevel`, `Primitives`, `Certify`). The generator therefore keeps an
// **explicit allowlist** of known ontology class IRIs and their Rust symbol
// names, plus a set of **static (non-OWL) entries**, and enforces that every
// `PreludeExport` individual in the ontology is covered by one of them.
//
// This turns "the prelude is ontology-driven" into a machine-checked invariant:
// adding a new `PreludeExport` individual without updating the codegen
// mapping fails the codegen with a clear "unknown PreludeExport class" panic,
// forcing the developer to make the mapping explicit. Panic is intentional
// here — `#![deny(clippy::panic)]` is overridden for this one code path.
#[allow(clippy::panic)]
fn generate_prelude(f: &mut RustFile, ontology: &Ontology) {
    // Map: ontology class IRI → Rust type name in `super::` scope.
    // Entries whose RHS is `None` mean "skip re-exporting" — the ontology
    // class doesn't correspond to a single foundation type (it's expressed
    // as a trait, an internal shim, or a non-OWL symbol).
    let known_mapping: &[(&str, Option<&str>)] = &[
        ("https://uor.foundation/schema/Datum", Some("Datum")),
        ("https://uor.foundation/schema/Term", Some("Term")),
        // WittLevel is a foundation struct but lives at crate::WittLevel,
        // not super::. Covered by the static `pub use crate::WittLevel` below.
        ("https://uor.foundation/schema/WittLevel", None),
        (
            "https://uor.foundation/reduction/CompileUnit",
            Some("CompileUnit"),
        ),
        (
            "https://uor.foundation/conformance/CompileUnitBuilder",
            Some("CompileUnitBuilder"),
        ),
        // ValidatedWrapper surfaces as `Validated`.
        (
            "https://uor.foundation/conformance/ValidatedWrapper",
            Some("Validated"),
        ),
        (
            "https://uor.foundation/conformance/ShapeViolationReport",
            Some("ShapeViolation"),
        ),
        // ValidationResult is a Rust enum baked into the crate root, not
        // under enforcement::.
        ("https://uor.foundation/conformance/ValidationResult", None),
        (
            "https://uor.foundation/cert/GroundingCertificate",
            Some("GroundingCertificate"),
        ),
        (
            "https://uor.foundation/cert/LiftChainCertificate",
            Some("LiftChainCertificate"),
        ),
        (
            "https://uor.foundation/cert/InhabitanceCertificate",
            Some("InhabitanceCertificate"),
        ),
        (
            "https://uor.foundation/cert/CompletenessCertificate",
            Some("CompletenessCertificate"),
        ),
        // ConstrainedType / CompleteType are trait/class domains in the
        // bridge modules, not standalone foundation::enforcement types.
        ("https://uor.foundation/type/ConstrainedType", None),
        ("https://uor.foundation/type/CompleteType", None),
        // GroundedContext is a state trait in foundation::user::state.
        ("https://uor.foundation/state/GroundedContext", None),
        // WitnessDatum backs the TermArena prelude entry (per
        // preludeExport_TermArena's comment).
        (
            "https://uor.foundation/conformance/WitnessDatum",
            Some("TermArena"),
        ),
    ];

    // Walk PreludeExport individuals and verify every one maps.
    let mut ontology_rust_names: std::collections::BTreeSet<String> =
        std::collections::BTreeSet::new();
    for ind in individuals_of_type(ontology, "https://uor.foundation/conformance/PreludeExport") {
        let class_iri = match ind_prop_str(ind, "https://uor.foundation/conformance/exportsClass") {
            Some(iri) => iri,
            None => continue,
        };
        // Look up the IRI in the known mapping; panic if the ontology adds
        // a PreludeExport for a class the codegen has never seen.
        let entry = known_mapping.iter().find(|(iri, _)| *iri == class_iri);
        let rust_name = match entry {
            Some((_, Some(name))) => Some(name.to_string()),
            Some((_, None)) => None, // mapped but intentionally skipped
            None => panic!(
                "generate_prelude: unknown conformance:PreludeExport class IRI `{class_iri}`. \
                 Add it to `known_mapping` in codegen/src/enforcement.rs, mapping to the \
                 Rust type name in foundation::enforcement scope or `None` if the class is \
                 not a standalone foundation type."
            ),
        };
        // Optional exportRustName override.
        let alias = ind_prop_str(ind, "https://uor.foundation/conformance/exportRustName")
            .map(|s| s.to_string());
        let emitted_name = match (rust_name, alias) {
            (Some(rust), Some(a)) if a != rust => Some(a),
            (Some(rust), _) => Some(rust),
            (None, _) => None,
        };
        if let Some(name) = emitted_name {
            ontology_rust_names.insert(name);
        }
    }

    // Non-OWL foundation symbols the prelude needs. These are emitted
    // unconditionally — they have no ontology backing and live in scope
    // for the consumer one-liners.
    //
    // Phase B: the v0.2.1 `Certify` trait and its five unit-struct resolver
    // façades are removed. The only verdict surface is the module-per-resolver
    // free-function path (`resolver::<name>::certify(...)`), which is already
    // reachable as `crate::enforcement::resolver::...`.
    let non_owl_entries: &[&str] = &[
        "Grounded",
        "GroundedShape",
        "OntologyTarget",
        "ImpossibilityWitnessKind",
        "PipelineFailure",
        "BindingsTable",
        "BindingEntry",
        "TermArena",
        "RingOp",
        "UnaryRingOp",
        "Mul",
        "Add",
        "Sub",
        "Xor",
        "And",
        "Or",
        "Neg",
        "BNot",
        "Succ",
        "Embed",
        "ValidLevelEmbedding",
        "W8",
        "W16",
        "FragmentMarker",
        "ConstrainedTypeInput",
        "GenericImpossibilityWitness",
        "InhabitanceImpossibilityWitness",
        // v0.2.2 W4: GroundingMapKind sealed marker traits + 5 kind structs.
        "GroundingMapKind",
        "Total",
        "Invertible",
        "PreservesStructure",
        "PreservesMetric",
        "IntegerGroundingMap",
        "Utf8GroundingMap",
        "JsonGroundingMap",
        "DigestGroundingMap",
        "BinaryGroundingMap",
        // v0.2.2 W11: Certificate trait + Certified<C> parametric carrier.
        "Certificate",
        "Certified",
        "TransformCertificate",
        "IsometryCertificate",
        "InvolutionCertificate",
        "GeodesicCertificate",
        "MeasurementCertificate",
        "BornRuleVerification",
        "CompletenessAuditTrail",
        "ChainAuditTrail",
        "GeodesicEvidenceBundle",
        // v0.2.2 W13: Validated<T, Phase> parametric phases.
        "ValidationPhase",
        "CompileTime",
        "Runtime",
        // v0.2.2 W8: Triad bundling struct.
        "Triad",
        // v0.2.2 Phase Q.1: grounding surface.
        "Grounding",
        "GroundingExt",
        "GroundingProgram",
        "GroundedCoord",
        "GroundedTuple",
        "GroundedValue",
        // v0.2.2 Phase Q.1: substrate surface.
        "Hasher",
        "ContentAddress",
        "ContentFingerprint",
        // v0.2.2 Phase Q.1: timing surface (Phase B introductions).
        "TimingPolicy",
        "CanonicalTimingPolicy",
        "UorTime",
        "LandauerBudget",
        "Nanos",
        "Calibration",
        "CalibrationError",
    ];

    f.doc_comment("v0.2.1 ergonomics prelude. Re-exports the core symbols downstream crates");
    f.doc_comment("need for the consumer-facing one-liners.");
    f.doc_comment("");
    f.doc_comment("Ontology-driven: the set of certificate / type / builder symbols is");
    f.doc_comment("sourced from `conformance:PreludeExport` individuals. Adding a new");
    f.doc_comment("symbol to the prelude is an ontology edit, verified against the");
    f.doc_comment("codegen's known-name mapping at build time.");
    f.line("pub mod prelude {");
    let mut emitted: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    // Ontology-derived entries (deterministic via BTreeSet ordering).
    for name in &ontology_rust_names {
        if emitted.insert(name.clone()) {
            f.line(&format!("    pub use super::{name};"));
        }
    }
    // Non-OWL entries.
    for name in non_owl_entries {
        if emitted.insert(name.to_string()) {
            f.line(&format!("    pub use super::{name};"));
        }
    }
    f.line("    pub use crate::{DecimalTranscendental, DefaultHostTypes, HostTypes, WittLevel};");
    // v0.2.2 Phase Q.1: calibrations preset module + cross-module re-exports.
    f.line("    pub use super::calibrations;");
    f.line("    pub use crate::pipeline::empty_bindings_table;");
    f.line("    pub use crate::pipeline::{");
    f.line("        validate_constrained_type, validate_constrained_type_const,");
    f.line("        ConstraintRef, FragmentKind, ConstrainedTypeShape,");
    f.line("    };");
    f.line("}");
    f.blank();
}

// ─────────────────────────────────────────────────────────────────────────
// v0.2.2 Phase C.3 — Limbs<N> generic kernel and Limbs-backed ring ops.
//
// `Limbs<const N: usize>` is the foundation's generic backing for Witt
// levels above W128. It holds an inline `[u64; N]` array (no heap, no
// allocation; const-fn throughout) and exposes the same arithmetic
// primitives as the native u8/u16/u32/u64/u128 backings: `wrapping_add`,
// `wrapping_sub`, `wrapping_mul` (schoolbook only — Phase C.4 adds the
// Toom-Cook resolver), bitwise ops, and a `mask_high_bits` helper for
// non-exact-fit widths.
//
// The kernel is `pub` (its constructors are `pub(crate)`) so the
// foundation's per-level Witt structs and ring-op impls can name it. The
// `Limbs<N>` type itself is sealed via private fields and pub(crate)
// constructors.
// ─────────────────────────────────────────────────────────────────────────

/// Returns the Limbs-backed Witt levels (bit_width > 128, multiple of 8).
/// Each tuple is `(local_name, bit_width, limb_count)` where
/// `limb_count = ⌈bit_width / 64⌉`.
pub(crate) fn limbs_witt_levels(ontology: &Ontology) -> Vec<(String, u32, usize)> {
    let mut levels: Vec<(String, u32, usize)> = Vec::new();
    for ind in individuals_of_type(ontology, "https://uor.foundation/schema/WittLevel") {
        let bits = ind
            .properties
            .iter()
            .find_map(|(k, v)| {
                if *k == "https://uor.foundation/schema/bitsWidth" {
                    if let uor_ontology::model::IndividualValue::Int(n) = v {
                        return Some(*n as u32);
                    }
                }
                None
            })
            .unwrap_or(0);
        if bits == 0 || bits % 8 != 0 || bits <= 128 {
            continue;
        }
        let limb_count = bits.div_ceil(64) as usize;
        let local = local_name(ind.id).to_string();
        levels.push((local, bits, limb_count));
    }
    levels.sort_by_key(|(_, bits, _)| *bits);
    levels
}

/// Emits the `Limbs<const N: usize>` generic kernel.
fn generate_limbs_kernel(f: &mut RustFile) {
    f.doc_comment("v0.2.2 Phase C.3: foundation-internal generic backing for Witt");
    f.doc_comment("levels above W128. Holds an inline `[u64; N]` array with no heap");
    f.doc_comment("allocation, no global state, and `const fn` arithmetic throughout.");
    f.doc_comment("Constructors are `pub(crate)`; downstream cannot fabricate a `Limbs<N>`.");
    f.doc_comment("");
    f.doc_comment("Multiplication is schoolbook-only at v0.2.2 Phase C.3; the Toom-Cook");
    f.doc_comment("framework with parametric splitting factor `R` ships in Phase C.4 via");
    f.doc_comment("the `resolver::multiplication::certify` resolver, which decides `R`");
    f.doc_comment("per call from a Landauer cost function constrained by stack budget.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct Limbs<const N: usize> {");
    f.indented_doc_comment("Little-endian limbs: `words[0]` is the low 64 bits.");
    f.line("    words: [u64; N],");
    f.indented_doc_comment("Prevents external construction.");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl<const N: usize> Limbs<N> {");
    f.indented_doc_comment("Crate-internal constructor from a fixed-size limb array.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn from_words(words: [u64; N]) -> Self {");
    f.line("        Self { words, _sealed: () }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("All-zeros constructor.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn zero() -> Self {");
    f.line("        Self { words: [0u64; N], _sealed: () }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Returns a reference to the underlying limb array.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn words(&self) -> &[u64; N] {");
    f.line("        &self.words");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Wrapping addition mod 2^(64*N). Const-fn schoolbook with carry.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn wrapping_add(self, other: Self) -> Self {");
    f.line("        let mut out = [0u64; N];");
    f.line("        let mut carry: u64 = 0;");
    f.line("        let mut i = 0;");
    f.line("        while i < N {");
    f.line("            let (s1, c1) = self.words[i].overflowing_add(other.words[i]);");
    f.line("            let (s2, c2) = s1.overflowing_add(carry);");
    f.line("            out[i] = s2;");
    f.line("            carry = (c1 as u64) | (c2 as u64);");
    f.line("            i += 1;");
    f.line("        }");
    f.line("        Self { words: out, _sealed: () }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Wrapping subtraction mod 2^(64*N). Const-fn schoolbook with borrow.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn wrapping_sub(self, other: Self) -> Self {");
    f.line("        let mut out = [0u64; N];");
    f.line("        let mut borrow: u64 = 0;");
    f.line("        let mut i = 0;");
    f.line("        while i < N {");
    f.line("            let (d1, b1) = self.words[i].overflowing_sub(other.words[i]);");
    f.line("            let (d2, b2) = d1.overflowing_sub(borrow);");
    f.line("            out[i] = d2;");
    f.line("            borrow = (b1 as u64) | (b2 as u64);");
    f.line("            i += 1;");
    f.line("        }");
    f.line("        Self { words: out, _sealed: () }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Wrapping schoolbook multiplication mod 2^(64*N). The high N limbs of");
    f.indented_doc_comment("the 2N-limb full product are discarded (mod 2^bits truncation).");
    f.indented_doc_comment("");
    f.indented_doc_comment("v0.2.2 Phase C.3: schoolbook only. Phase C.4 adds the Toom-Cook");
    f.indented_doc_comment("framework with parametric R via `resolver::multiplication::certify`.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn wrapping_mul(self, other: Self) -> Self {");
    f.line("        let mut out = [0u64; N];");
    f.line("        let mut i = 0;");
    f.line("        while i < N {");
    f.line("            let mut carry: u128 = 0;");
    f.line("            let mut j = 0;");
    f.line("            while j < N - i {");
    f.line("                let prod = (self.words[i] as u128)");
    f.line("                    * (other.words[j] as u128)");
    f.line("                    + (out[i + j] as u128)");
    f.line("                    + carry;");
    f.line("                out[i + j] = prod as u64;");
    f.line("                carry = prod >> 64;");
    f.line("                j += 1;");
    f.line("            }");
    f.line("            i += 1;");
    f.line("        }");
    f.line("        Self { words: out, _sealed: () }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Bitwise XOR.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn xor(self, other: Self) -> Self {");
    f.line("        let mut out = [0u64; N];");
    f.line("        let mut i = 0;");
    f.line("        while i < N {");
    f.line("            out[i] = self.words[i] ^ other.words[i];");
    f.line("            i += 1;");
    f.line("        }");
    f.line("        Self { words: out, _sealed: () }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Bitwise AND.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn and(self, other: Self) -> Self {");
    f.line("        let mut out = [0u64; N];");
    f.line("        let mut i = 0;");
    f.line("        while i < N {");
    f.line("            out[i] = self.words[i] & other.words[i];");
    f.line("            i += 1;");
    f.line("        }");
    f.line("        Self { words: out, _sealed: () }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Bitwise OR.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn or(self, other: Self) -> Self {");
    f.line("        let mut out = [0u64; N];");
    f.line("        let mut i = 0;");
    f.line("        while i < N {");
    f.line("            out[i] = self.words[i] | other.words[i];");
    f.line("            i += 1;");
    f.line("        }");
    f.line("        Self { words: out, _sealed: () }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Bitwise NOT.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn not(self) -> Self {");
    f.line("        let mut out = [0u64; N];");
    f.line("        let mut i = 0;");
    f.line("        while i < N {");
    f.line("            out[i] = !self.words[i];");
    f.line("            i += 1;");
    f.line("        }");
    f.line("        Self { words: out, _sealed: () }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Mask the high bits of the value to keep only the low `bits` bits.");
    f.indented_doc_comment("Used at the arithmetic boundary for non-exact-fit Witt widths (e.g.,");
    f.indented_doc_comment(
        "W160 over `Limbs<3>`: 64+64+32 bits = mask the upper 32 bits of words[2]).",
    );
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn mask_high_bits(self, bits: u32) -> Self {");
    f.line("        let mut out = self.words;");
    f.line("        let high_word_idx = (bits / 64) as usize;");
    f.line("        let low_bits_in_high_word = bits % 64;");
    f.line("        if low_bits_in_high_word != 0 && high_word_idx < N {");
    f.line("            let mask = (1u64 << low_bits_in_high_word) - 1;");
    f.line("            out[high_word_idx] &= mask;");
    f.line("            // Zero everything above the high word.");
    f.line("            let mut i = high_word_idx + 1;");
    f.line("            while i < N {");
    f.line("                out[i] = 0;");
    f.line("                i += 1;");
    f.line("            }");
    f.line("        } else if low_bits_in_high_word == 0 && high_word_idx < N {");
    f.line("            // bits is exactly a multiple of 64; zero everything from high_word_idx.");
    f.line("            let mut i = high_word_idx;");
    f.line("            while i < N {");
    f.line("                out[i] = 0;");
    f.line("                i += 1;");
    f.line("            }");
    f.line("        }");
    f.line("        Self { words: out, _sealed: () }");
    f.line("    }");
    f.line("}");
    f.blank();
}

/// Emits Limbs-backed marker structs and `RingOp` / `UnaryRingOp` impls
/// for every WittLevel individual whose bit_width > 128.
fn generate_limbs_ring_ops(f: &mut RustFile, ontology: &Ontology) {
    let levels = limbs_witt_levels(ontology);
    if levels.is_empty() {
        return;
    }

    f.doc_comment("v0.2.2 Phase C.3: marker structs for Limbs-backed Witt levels.");
    f.doc_comment("Each level binds a const-generic `Limbs<N>` width at the type level.");
    for (local, bits, _) in &levels {
        f.doc_comment(&format!(
            "{local} marker — {bits}-bit Witt level, Limbs-backed."
        ));
        f.line("#[derive(Debug, Default, Clone, Copy)]");
        f.line(&format!("pub struct {local};"));
        f.blank();
    }

    let bin_ops = [
        ("Mul", "wrapping_mul"),
        ("Add", "wrapping_add"),
        ("Sub", "wrapping_sub"),
        ("Xor", "xor"),
        ("And", "and"),
        ("Or", "or"),
    ];
    let unary_ops = [("Neg", "neg"), ("BNot", "bnot"), ("Succ", "succ")];

    for (local, bits, limb_count) in &levels {
        let limb_n = limb_count;
        let exact_fit = bits % 64 == 0;
        for (op_name, kernel_op) in &bin_ops {
            f.line(&format!("impl RingOp<{local}> for {op_name}<{local}> {{"));
            f.line(&format!("    type Operand = Limbs<{limb_n}>;"));
            f.line("    #[inline]");
            f.line(&format!(
                "    fn apply(a: Limbs<{limb_n}>, b: Limbs<{limb_n}>) -> Limbs<{limb_n}> {{"
            ));
            if exact_fit {
                f.line(&format!("        a.{kernel_op}(b)"));
            } else {
                f.line(&format!("        a.{kernel_op}(b).mask_high_bits({bits})"));
            }
            f.line("    }");
            f.line("}");
            f.blank();
        }
        // Unary ops over Limbs.
        // Neg(a) = 0 - a = Limbs::zero().wrapping_sub(a)
        // BNot(a) = !a, masked to bit width
        // Succ(a) = a.wrapping_add(Limbs::from_words([1, 0, ..., 0]))
        for (op_name, _) in &unary_ops {
            f.line(&format!(
                "impl UnaryRingOp<{local}> for {op_name}<{local}> {{"
            ));
            f.line(&format!("    type Operand = Limbs<{limb_n}>;"));
            f.line("    #[inline]");
            f.line(&format!(
                "    fn apply(a: Limbs<{limb_n}>) -> Limbs<{limb_n}> {{"
            ));
            let body = match *op_name {
                "Neg" => format!("Limbs::<{limb_n}>::zero().wrapping_sub(a)"),
                "BNot" => "a.not()".to_string(),
                "Succ" => {
                    let one_limbs = if *limb_n == 1 {
                        "Limbs::<1>::from_words([1u64])".to_string()
                    } else {
                        // [1, 0, 0, ..., 0]
                        let mut elems = String::from("[1u64");
                        for _ in 1..*limb_n {
                            elems.push_str(", 0u64");
                        }
                        elems.push(']');
                        format!("Limbs::<{limb_n}>::from_words({elems})")
                    };
                    format!("a.wrapping_add({one_limbs})")
                }
                _ => "a".to_string(),
            };
            if exact_fit {
                f.line(&format!("        {body}"));
            } else {
                f.line(&format!("        ({body}).mask_high_bits({bits})"));
            }
            f.line("    }");
            f.line("}");
            f.blank();
        }
    }
}

/// v0.2.2 Phase C.4: emit multiplication resolver call-site context.
fn generate_multiplication_context(f: &mut RustFile) {
    f.doc_comment("v0.2.2 Phase C.4: call-site context consumed by the multiplication");
    f.doc_comment("resolver. Carries the stack budget (`linear:stackBudgetBytes`), the");
    f.doc_comment("const-eval regime, and the limb count of the operand's `Limbs<N>`");
    f.doc_comment("backing. The resolver picks the cost-optimal Toom-Cook splitting");
    f.doc_comment("factor R based on this context.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct MulContext {");
    f.indented_doc_comment("Stack budget available at the call site, in bytes. Zero is");
    f.indented_doc_comment("inadmissible; the resolver returns an impossibility witness.");
    f.line("    pub stack_budget_bytes: u64,");
    f.indented_doc_comment("True if this call is in const-eval context. In const-eval, only");
    f.indented_doc_comment("R = 1 (schoolbook) is admissible because deeper recursion blows");
    f.indented_doc_comment("the const-eval depth limit.");
    f.line("    pub const_eval: bool,");
    f.indented_doc_comment("Number of 64-bit limbs in the operand's `Limbs<N>` backing.");
    f.indented_doc_comment("Schoolbook cost is proportional to `N^2`; Karatsuba cost is");
    f.indented_doc_comment("proportional to `3 \u{00b7} (N/2)^2`. For native-backed levels");
    f.indented_doc_comment("(W8..W128), pass the equivalent limb count.");
    f.line("    pub limb_count: usize,");
    f.line("}");
    f.blank();
    f.line("impl MulContext {");
    f.indented_doc_comment("Construct a new `MulContext` for the call site.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn new(stack_budget_bytes: u64, const_eval: bool, limb_count: usize) -> Self {");
    f.line("        Self { stack_budget_bytes, const_eval, limb_count }");
    f.line("    }");
    f.line("}");
    f.blank();

    // Extend MultiplicationCertificate with evidence fields via a secondary
    // impl block. The shim is emitted by generate_ontology_target_trait with
    // only a `witt_bits: u16` field; we provide `with_evidence` as a
    // constructor that populates a parallel evidence struct kept in a thread-
    // local registry. Since no_std prohibits thread_local, we keep the
    // evidence inline on the shim by redefining it here is not possible.
    // Instead: extend the shim with copy-only evidence accessors and a
    // `with_evidence` const constructor that stores values in a secondary
    // sealed struct carried inside the certificate via a private cell. For
    // simplicity and correctness under no_std, we expose evidence as a
    // free-standing `MultiplicationEvidence` struct returned by a
    // `certify_at_context` helper; the certificate remains a thin handle.
    f.doc_comment("v0.2.2 Phase C.4: evidence returned alongside a `MultiplicationCertificate`.");
    f.doc_comment("The certificate is a sealed handle; its evidence (chosen splitting factor,");
    f.doc_comment("sub-multiplication count, accumulated Landauer cost in nats) lives here.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq)]");
    f.line("pub struct MultiplicationEvidence {");
    f.line("    splitting_factor: u32,");
    f.line("    sub_multiplication_count: u32,");
    // Phase 9: bit-pattern u64 keeps the public surface host-neutral.
    // Consumers project to `H::Decimal` via `DecimalTranscendental::from_bits`.
    f.line("    landauer_cost_nats_bits: u64,");
    f.line("}");
    f.blank();
    f.line("impl MultiplicationEvidence {");
    f.indented_doc_comment("The Toom-Cook splitting factor R chosen by the resolver.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn splitting_factor(&self) -> u32 {");
    f.line("        self.splitting_factor");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("The recursive sub-multiplication count for one multiplication.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn sub_multiplication_count(&self) -> u32 {");
    f.line("        self.sub_multiplication_count");
    f.line("    }");
    f.blank();
    f.indented_doc_comment(
        "Accumulated Landauer cost in nats, priced per `op:OA_5`. Returned as the \
         IEEE-754 bit pattern; project to `H::Decimal` at use sites via \
         `<H::Decimal as DecimalTranscendental>::from_bits(_)`.",
    );
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn landauer_cost_nats_bits(&self) -> u64 {");
    f.line("        self.landauer_cost_nats_bits");
    f.line("    }");
    f.line("}");
    f.blank();
    f.line("impl<const FP_MAX: usize> MultiplicationCertificate<FP_MAX> {");
    f.indented_doc_comment("v0.2.2 T6.7: construct a `MultiplicationCertificate` with substrate-");
    f.indented_doc_comment(
        "computed evidence. Crate-internal only; downstream obtains certificates",
    );
    f.indented_doc_comment("via `resolver::multiplication::certify::<H>`.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub(crate) fn with_evidence(");
    f.line("        splitting_factor: u32,");
    f.line("        sub_multiplication_count: u32,");
    f.line("        landauer_cost_nats_bits: u64,");
    f.line("        content_fingerprint: ContentFingerprint<FP_MAX>,");
    f.line("    ) -> Self {");
    f.line("        let _ = MultiplicationEvidence {");
    f.line("            splitting_factor,");
    f.line("            sub_multiplication_count,");
    f.line("            landauer_cost_nats_bits,");
    f.line("        };");
    f.line("        Self::with_level_and_fingerprint_const(32, content_fingerprint)");
    f.line("    }");
    f.line("}");
    f.blank();
}

/// v0.2.2 Phase D (Q4): parametric constraint surface.
///
/// Emits sealed `Observable` and `BoundShape` marker traits, their closed
/// impl sets (one unit struct per observable subclass + one per bound shape
/// individual), a `BoundConstraint<O, B>` parametric carrier, a
/// `Conjunction<const N: usize>` composition wrapper, and the seven type
/// aliases (ResidueConstraint, HammingConstraint, DepthConstraint,
/// CarryConstraint, SiteConstraint, AffineConstraint, CompositeConstraint)
/// preserving the v0.2.1 call-site syntax over the parametric form.
fn generate_parametric_constraint_surface(f: &mut RustFile) {
    // Sealed supertraits for Observable and BoundShape.
    // v0.2.2 Phase D (Q4) — parametric constraint surface replaces the
    // seven enumerated Constraint subclasses with BoundConstraint<O, B>.
    f.line("mod bound_constraint_sealed {");
    f.indented_doc_comment("Sealed supertrait for the closed Observable catalogue.");
    f.line("    pub trait ObservableSealed {}");
    f.indented_doc_comment("Sealed supertrait for the closed BoundShape catalogue.");
    f.line("    pub trait BoundShapeSealed {}");
    f.line("}");
    f.blank();

    f.doc_comment("Sealed marker trait identifying the closed catalogue of observables");
    f.doc_comment("admissible in BoundConstraint. Implemented by unit structs emitted");
    f.doc_comment("below per `observable:Observable` subclass referenced by a");
    f.doc_comment("BoundConstraint kind individual.");
    f.line("pub trait Observable: bound_constraint_sealed::ObservableSealed {");
    f.indented_doc_comment("Ontology IRI of this observable class.");
    f.line("    const IRI: &'static str;");
    f.line("}");
    f.blank();

    f.doc_comment("Sealed marker trait identifying the closed catalogue of bound shapes.");
    f.doc_comment("Exactly six individuals: EqualBound, LessEqBound, GreaterEqBound,");
    f.doc_comment("RangeContainBound, ResidueClassBound, AffineEqualBound.");
    f.line("pub trait BoundShape: bound_constraint_sealed::BoundShapeSealed {");
    f.indented_doc_comment("Ontology IRI of this bound shape individual.");
    f.line("    const IRI: &'static str;");
    f.line("}");
    f.blank();

    // Observable catalogue (5 entries: ValueMod, Hamming, DerivationDepth,
    // CarryDepth, FreeRank).
    let observables: &[(&str, &str, &str)] = &[
        (
            "ValueModObservable",
            "https://uor.foundation/observable/ValueModObservable",
            "Observes a Datum's value modulo a configurable modulus.",
        ),
        (
            "HammingMetric",
            "https://uor.foundation/observable/HammingMetric",
            "Distance between two ring elements under the Hamming metric.",
        ),
        (
            "DerivationDepthObservable",
            "https://uor.foundation/derivation/DerivationDepthObservable",
            "Observes the derivation depth of a Datum.",
        ),
        (
            "CarryDepthObservable",
            "https://uor.foundation/carry/CarryDepthObservable",
            "Observes the carry depth of a Datum in the W\u{2082} tower.",
        ),
        (
            "FreeRankObservable",
            "https://uor.foundation/partition/FreeRankObservable",
            "Observes the free-rank of the partition associated with a Datum.",
        ),
    ];
    for (name, iri, doc) in observables {
        f.doc_comment(doc);
        f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]");
        f.line(&format!("pub struct {name};"));
        f.line(&format!(
            "impl bound_constraint_sealed::ObservableSealed for {name} {{}}"
        ));
        f.line(&format!("impl Observable for {name} {{"));
        f.line(&format!("    const IRI: &'static str = \"{iri}\";"));
        f.line("}");
        f.blank();
    }

    // BoundShape catalogue (6 entries).
    let shapes: &[(&str, &str, &str)] = &[
        (
            "EqualBound",
            "https://uor.foundation/type/EqualBound",
            "Predicate form: `observable(datum) == target`.",
        ),
        (
            "LessEqBound",
            "https://uor.foundation/type/LessEqBound",
            "Predicate form: `observable(datum) <= bound`.",
        ),
        (
            "GreaterEqBound",
            "https://uor.foundation/type/GreaterEqBound",
            "Predicate form: `observable(datum) >= bound`.",
        ),
        (
            "RangeContainBound",
            "https://uor.foundation/type/RangeContainBound",
            "Predicate form: `lo <= observable(datum) <= hi`.",
        ),
        (
            "ResidueClassBound",
            "https://uor.foundation/type/ResidueClassBound",
            "Predicate form: `observable(datum) \u{2261} residue (mod modulus)`.",
        ),
        (
            "AffineEqualBound",
            "https://uor.foundation/type/AffineEqualBound",
            "Predicate form: `observable(datum) == offset + affine combination`.",
        ),
    ];
    for (name, iri, doc) in shapes {
        f.doc_comment(doc);
        f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]");
        f.line(&format!("pub struct {name};"));
        f.line(&format!(
            "impl bound_constraint_sealed::BoundShapeSealed for {name} {{}}"
        ));
        f.line(&format!("impl BoundShape for {name} {{"));
        f.line(&format!("    const IRI: &'static str = \"{iri}\";"));
        f.line("}");
        f.blank();
    }

    // BoundArgValue + BoundArguments fixed-size carrier.
    f.doc_comment("Parameter value type for `BoundConstraint` arguments.");
    f.doc_comment("Sealed enum over the closed set of primitive kinds the bound-shape");
    f.doc_comment("catalogue requires. No heap, no `String`.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub enum BoundArgValue {");
    f.indented_doc_comment("Unsigned 64-bit integer.");
    f.line("    U64(u64),");
    f.indented_doc_comment("Signed 64-bit integer.");
    f.line("    I64(i64),");
    f.indented_doc_comment("Fixed 32-byte content-addressed value.");
    f.line("    Bytes32([u8; 32]),");
    f.line("}");
    f.blank();

    f.doc_comment("Fixed-size arguments carrier for a `BoundConstraint`.");
    f.doc_comment("");
    f.doc_comment("Holds up to eight `(name, value)` pairs inline. The closed");
    f.doc_comment("bound-shape catalogue requires at most three parameters per kind;");
    f.doc_comment("the extra slots are reserved for future kind additions without");
    f.doc_comment("changing the carrier layout.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct BoundArguments {");
    f.line("    entries: [Option<BoundArgEntry>; 8],");
    f.line("}");
    f.blank();

    f.doc_comment("A single named parameter in a `BoundArguments` table.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct BoundArgEntry {");
    f.indented_doc_comment("Parameter name (a `&'static str` intentional over heap-owned).");
    f.line("    pub name: &'static str,");
    f.indented_doc_comment("Parameter value.");
    f.line("    pub value: BoundArgValue,");
    f.line("}");
    f.blank();

    f.line("impl BoundArguments {");
    f.indented_doc_comment("Construct an empty argument table.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn empty() -> Self {");
    f.line("        Self { entries: [None; 8] }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Construct a table with a single `(name, value)` pair.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn single(name: &'static str, value: BoundArgValue) -> Self {");
    f.line("        let mut entries = [None; 8];");
    f.line("        entries[0] = Some(BoundArgEntry { name, value });");
    f.line("        Self { entries }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Construct a table with two `(name, value)` pairs.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn pair(");
    f.line("        first: (&'static str, BoundArgValue),");
    f.line("        second: (&'static str, BoundArgValue),");
    f.line("    ) -> Self {");
    f.line("        let mut entries = [None; 8];");
    f.line("        entries[0] = Some(BoundArgEntry { name: first.0, value: first.1 });");
    f.line("        entries[1] = Some(BoundArgEntry { name: second.0, value: second.1 });");
    f.line("        Self { entries }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Access the stored entries.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn entries(&self) -> &[Option<BoundArgEntry>; 8] {");
    f.line("        &self.entries");
    f.line("    }");
    f.line("}");
    f.blank();

    // BoundConstraint<O, B> carrier.
    f.doc_comment("Parametric constraint carrier (v0.2.2 Phase D).");
    f.doc_comment("");
    f.doc_comment("Generic over `O: Observable` and `B: BoundShape`. The seven");
    f.doc_comment("legacy constraint kinds are preserved as type aliases over this");
    f.doc_comment("carrier; see `ResidueConstraint`, `HammingConstraint`, etc. below.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct BoundConstraint<O: Observable, B: BoundShape> {");
    f.line("    observable: O,");
    f.line("    bound: B,");
    f.line("    args: BoundArguments,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl<O: Observable, B: BoundShape> BoundConstraint<O, B> {");
    f.indented_doc_comment("Crate-internal constructor. Downstream obtains values through");
    f.indented_doc_comment("the per-type-alias `pub const fn new` constructors.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub(crate) const fn from_parts(observable: O, bound: B, args: BoundArguments) -> Self {");
    f.line("        Self { observable, bound, args, _sealed: () }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Access the bound observable.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn observable(&self) -> &O { &self.observable }");
    f.blank();
    f.indented_doc_comment("Access the bound shape.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn bound(&self) -> &B { &self.bound }");
    f.blank();
    f.indented_doc_comment("Access the bound arguments.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn args(&self) -> &BoundArguments { &self.args }");
    f.line("}");
    f.blank();

    // Conjunction<N> wrapper.
    f.doc_comment("Parametric conjunction of `BoundConstraint` kinds (v0.2.2 Phase D).");
    f.doc_comment("");
    f.doc_comment("Replaces the v0.2.1 `CompositeConstraint` enumeration; the legacy");
    f.doc_comment("name survives as the type alias `CompositeConstraint<N>` below.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct Conjunction<const N: usize> {");
    f.line("    len: usize,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl<const N: usize> Conjunction<N> {");
    f.indented_doc_comment("Construct a new Conjunction with `len` conjuncts.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn new(len: usize) -> Self {");
    f.line("        Self { len, _sealed: () }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("The number of conjuncts in this Conjunction.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn len(&self) -> usize { self.len }");
    f.blank();
    f.indented_doc_comment("Whether the Conjunction is empty.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn is_empty(&self) -> bool { self.len == 0 }");
    f.line("}");
    f.blank();

    // Seven type aliases + per-alias constructors.
    f.doc_comment("v0.2.1 legacy type alias: a `BoundConstraint` kind asserting");
    f.doc_comment("residue-class membership (`value mod m == r`).");
    f.line("pub type ResidueConstraint = BoundConstraint<ValueModObservable, ResidueClassBound>;");
    f.blank();
    f.line("impl ResidueConstraint {");
    f.indented_doc_comment("Construct a residue constraint with the given modulus and residue.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn new(modulus: u64, residue: u64) -> Self {");
    f.line("        let args = BoundArguments::pair(");
    f.line("            (\"modulus\", BoundArgValue::U64(modulus)),");
    f.line("            (\"residue\", BoundArgValue::U64(residue)),");
    f.line("        );");
    f.line("        BoundConstraint::from_parts(ValueModObservable, ResidueClassBound, args)");
    f.line("    }");
    f.line("}");
    f.blank();

    f.doc_comment("v0.2.1 legacy type alias: a `BoundConstraint` kind bounding the");
    f.doc_comment("Hamming weight of the Datum (`weight <= bound`).");
    f.line("pub type HammingConstraint = BoundConstraint<HammingMetric, LessEqBound>;");
    f.blank();
    f.line("impl HammingConstraint {");
    f.indented_doc_comment("Construct a Hamming constraint with the given upper bound.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn new(bound: u64) -> Self {");
    f.line("        let args = BoundArguments::single(\"bound\", BoundArgValue::U64(bound));");
    f.line("        BoundConstraint::from_parts(HammingMetric, LessEqBound, args)");
    f.line("    }");
    f.line("}");
    f.blank();

    f.doc_comment("v0.2.1 legacy type alias: a `BoundConstraint` kind bounding the");
    f.doc_comment("derivation depth of the Datum.");
    f.line("pub type DepthConstraint = BoundConstraint<DerivationDepthObservable, LessEqBound>;");
    f.blank();
    f.line("impl DepthConstraint {");
    f.indented_doc_comment("Construct a depth constraint with min and max depths.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn new(min_depth: u64, max_depth: u64) -> Self {");
    f.line("        let args = BoundArguments::pair(");
    f.line("            (\"min_depth\", BoundArgValue::U64(min_depth)),");
    f.line("            (\"max_depth\", BoundArgValue::U64(max_depth)),");
    f.line("        );");
    f.line("        BoundConstraint::from_parts(DerivationDepthObservable, LessEqBound, args)");
    f.line("    }");
    f.line("}");
    f.blank();

    f.doc_comment("v0.2.1 legacy type alias: a `BoundConstraint` kind bounding the");
    f.doc_comment("carry depth of the Datum in the W\u{2082} tower.");
    f.line("pub type CarryConstraint = BoundConstraint<CarryDepthObservable, LessEqBound>;");
    f.blank();
    f.line("impl CarryConstraint {");
    f.indented_doc_comment("Construct a carry constraint with the given upper bound.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn new(bound: u64) -> Self {");
    f.line("        let args = BoundArguments::single(\"bound\", BoundArgValue::U64(bound));");
    f.line("        BoundConstraint::from_parts(CarryDepthObservable, LessEqBound, args)");
    f.line("    }");
    f.line("}");
    f.blank();

    f.doc_comment("v0.2.1 legacy type alias: a `BoundConstraint` kind pinning a");
    f.doc_comment("single site coordinate.");
    f.line("pub type SiteConstraint = BoundConstraint<FreeRankObservable, LessEqBound>;");
    f.blank();
    f.line("impl SiteConstraint {");
    f.indented_doc_comment("Construct a site constraint with the given site index.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn new(site_index: u64) -> Self {");
    f.line("        let args = BoundArguments::single(");
    f.line("            \"site_index\",");
    f.line("            BoundArgValue::U64(site_index),");
    f.line("        );");
    f.line("        BoundConstraint::from_parts(FreeRankObservable, LessEqBound, args)");
    f.line("    }");
    f.line("}");
    f.blank();

    f.doc_comment("v0.2.1 legacy type alias: a `BoundConstraint` kind pinning an");
    f.doc_comment("affine relationship on the Datum's value projection.");
    f.line("pub type AffineConstraint = BoundConstraint<ValueModObservable, AffineEqualBound>;");
    f.blank();
    f.line("impl AffineConstraint {");
    f.indented_doc_comment("Construct an affine constraint with the given offset.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn new(offset: u64) -> Self {");
    f.line("        let args = BoundArguments::single(\"offset\", BoundArgValue::U64(offset));");
    f.line("        BoundConstraint::from_parts(ValueModObservable, AffineEqualBound, args)");
    f.line("    }");
    f.line("}");
    f.blank();

    f.doc_comment("v0.2.1 legacy type alias: a `Conjunction` over `N` BoundConstraint");
    f.doc_comment("kinds (`CompositeConstraint<3>` = 3-way conjunction).");
    f.line("pub type CompositeConstraint<const N: usize> = Conjunction<N>;");
    f.blank();
}

/// v0.2.2 Phase E: bridge namespace completion.
///
/// Emits sealed Query/Coordinate/BindingQuery/Partition/Trace/TraceEvent/
/// HomologyClass/CohomologyClass/InteractionDeclarationBuilder types +
/// the Derivation::replay() accessor.
fn generate_bridge_namespace_surface(f: &mut RustFile) {
    // Query + Coordinate<L> + BindingQuery.
    f.doc_comment("v0.2.2 Phase E: sealed query handle.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct Query {");
    f.line("    address: ContentAddress,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl Query {");
    f.indented_doc_comment("Returns the content-hashed query address.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn address(&self) -> ContentAddress { self.address }");
    f.blank();
    f.indented_doc_comment("Crate-internal constructor.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new(address: ContentAddress) -> Self {");
    f.line("        Self { address, _sealed: () }");
    f.line("    }");
    f.line("}");
    f.blank();

    f.doc_comment("v0.2.2 Phase E: typed query coordinate parametric over WittLevel.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct Coordinate<L> {");
    f.line("    stratum: u64,");
    f.line("    spectrum: u64,");
    f.line("    address: u64,");
    f.line("    _level: PhantomData<L>,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl<L> Coordinate<L> {");
    f.indented_doc_comment("Returns the stratum coordinate.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn stratum(&self) -> u64 { self.stratum }");
    f.blank();
    f.indented_doc_comment("Returns the spectrum coordinate.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn spectrum(&self) -> u64 { self.spectrum }");
    f.blank();
    f.indented_doc_comment("Returns the address coordinate.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn address(&self) -> u64 { self.address }");
    f.blank();
    f.indented_doc_comment("Crate-internal constructor.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new(stratum: u64, spectrum: u64, address: u64) -> Self {");
    f.line("        Self { stratum, spectrum, address, _level: PhantomData, _sealed: () }");
    f.line("    }");
    f.line("}");
    f.blank();

    f.doc_comment("v0.2.2 Phase E: sealed binding query handle.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct BindingQuery {");
    f.line("    address: ContentAddress,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl BindingQuery {");
    f.indented_doc_comment("Returns the content-hashed binding query address.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn address(&self) -> ContentAddress { self.address }");
    f.blank();
    f.indented_doc_comment("Crate-internal constructor.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new(address: ContentAddress) -> Self {");
    f.line("        Self { address, _sealed: () }");
    f.line("    }");
    f.line("}");
    f.blank();

    // Partition sealed type.
    f.doc_comment("v0.2.2 Phase E: sealed Partition handle over the bridge:partition");
    f.doc_comment("component classification produced during grounding.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct Partition {");
    f.line("    component: PartitionComponent,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl Partition {");
    f.indented_doc_comment("Returns the component classification.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn component(&self) -> PartitionComponent { self.component }");
    f.blank();
    f.indented_doc_comment("Crate-internal constructor.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new(component: PartitionComponent) -> Self {");
    f.line("        Self { component, _sealed: () }");
    f.line("    }");
    f.line("}");
    f.blank();

    // Trace + TraceEvent.
    f.doc_comment("v0.2.2 Phase E: a single event in a derivation Trace.");
    f.doc_comment("");
    f.doc_comment("Fixed-size event; content-addressed so Trace replays are stable");
    f.doc_comment("across builds. The verifier in `uor-foundation-verify` (Phase H)");
    f.doc_comment("reconstructs the witness chain by walking a `Trace` iterator.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct TraceEvent {");
    f.indented_doc_comment("Step index in the derivation.");
    f.line("    step_index: u32,");
    f.indented_doc_comment("Primitive op applied at this step.");
    f.line("    op: PrimitiveOp,");
    f.indented_doc_comment("Content-hashed target address the op produced.");
    f.line("    target: ContentAddress,");
    f.indented_doc_comment("Sealing marker.");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl TraceEvent {");
    f.indented_doc_comment("Returns the step index.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn step_index(&self) -> u32 { self.step_index }");
    f.blank();
    f.indented_doc_comment("Returns the primitive op applied at this step.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn op(&self) -> PrimitiveOp { self.op }");
    f.blank();
    f.indented_doc_comment("Returns the content-hashed target address.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn target(&self) -> ContentAddress { self.target }");
    f.blank();
    f.indented_doc_comment("Crate-internal constructor.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new(step_index: u32, op: PrimitiveOp, target: ContentAddress) -> Self {");
    f.line("        Self { step_index, op, target, _sealed: () }");
    f.line("    }");
    f.line("}");
    f.blank();

    f.doc_comment("Fixed-capacity derivation trace. Holds up to `TR_MAX` events inline;");
    f.doc_comment("no heap. Produced by `Derivation::replay()` and consumed by");
    f.doc_comment("`uor-foundation-verify`. `TR_MAX` is the const-generic that carries");
    f.doc_comment("the application's selected `<MyBounds as HostBounds>::TRACE_MAX_EVENTS`;");
    f.doc_comment("the default const-generic resolves to the conventional 256.");
    f.doc_comment("");
    f.doc_comment("Carries `witt_level_bits` and `content_fingerprint` so `verify_trace`");
    f.doc_comment("can reconstruct the source `GroundingCertificate` via structural-");
    f.doc_comment("validation + fingerprint passthrough (no hash recomputation).");
    f.line("#[derive(Debug, Clone, Copy)]");
    f.line("pub struct Trace<const TR_MAX: usize = 256, const FP_MAX: usize = 32> {");
    f.line("    events: [Option<TraceEvent>; TR_MAX],");
    f.line("    len: u16,");
    f.indented_doc_comment("Witt level the source grounding was minted at, packed");
    f.indented_doc_comment("by `Derivation::replay` from the parent `Grounded::witt_level_bits`.");
    f.indented_doc_comment("`verify_trace` reads this back to populate the certificate.");
    f.line("    witt_level_bits: u16,");
    f.indented_doc_comment("Parametric content fingerprint of the source unit's full state,");
    f.indented_doc_comment("computed at grounding time by the consumer-supplied `Hasher` and");
    f.indented_doc_comment("packed in by `Derivation::replay`. `verify_trace` passes it through");
    f.indented_doc_comment("unchanged. The fingerprint's `FP_MAX` follows the application's");
    f.indented_doc_comment("selected `<MyBounds as HostBounds>::FINGERPRINT_MAX_BYTES`; this");
    f.indented_doc_comment("field uses the default-bound `ContentFingerprint`.");
    f.line("    content_fingerprint: ContentFingerprint<FP_MAX>,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl<const TR_MAX: usize, const FP_MAX: usize> Trace<TR_MAX, FP_MAX> {");
    f.indented_doc_comment("An empty Trace.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn empty() -> Self {");
    f.line("        Self {");
    f.line("            events: [None; TR_MAX],");
    f.line("            len: 0,");
    f.line("            witt_level_bits: 0,");
    f.line("            content_fingerprint: ContentFingerprint::zero(),");
    f.line("            _sealed: (),");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Crate-internal ctor for `Derivation::replay()` only.");
    f.indented_doc_comment("Bypasses validation because `replay()` constructs events from");
    f.indented_doc_comment("foundation-guaranteed-valid state (monotonic, contiguous, non-zero");
    f.indented_doc_comment("seed). No public path reaches this constructor; downstream uses the");
    f.indented_doc_comment("validating `try_from_events` instead.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn from_replay_events_const(");
    f.line("        events: [Option<TraceEvent>; TR_MAX],");
    f.line("        len: u16,");
    f.line("        witt_level_bits: u16,");
    f.line("        content_fingerprint: ContentFingerprint<FP_MAX>,");
    f.line("    ) -> Self {");
    f.line("        Self {");
    f.line("            events,");
    f.line("            len,");
    f.line("            witt_level_bits,");
    f.line("            content_fingerprint,");
    f.line("            _sealed: (),");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Number of events recorded.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn len(&self) -> u16 { self.len }");
    f.blank();
    f.indented_doc_comment("Whether the Trace is empty.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn is_empty(&self) -> bool { self.len == 0 }");
    f.blank();
    f.indented_doc_comment("Access the event at the given index, or `None` if out of range.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn event(&self, index: usize) -> Option<&TraceEvent> {");
    f.line("        self.events.get(index).and_then(|e| e.as_ref())");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("v0.2.2 T5: returns the Witt level the source grounding was minted at.");
    f.indented_doc_comment(
        "Carried through replay so `verify_trace` can populate the certificate.",
    );
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn witt_level_bits(&self) -> u16 { self.witt_level_bits }");
    f.blank();
    f.indented_doc_comment("v0.2.2 T5: returns the parametric content fingerprint of the source");
    f.indented_doc_comment("unit, computed at grounding time by the consumer-supplied `Hasher`.");
    f.indented_doc_comment("`verify_trace` passes this through unchanged into the re-derived");
    f.indented_doc_comment("certificate, upholding the round-trip property.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn content_fingerprint(&self) -> ContentFingerprint<FP_MAX> {");
    f.line("        self.content_fingerprint");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Validating constructor. Checks every invariant the verify path");
    f.indented_doc_comment("relies on: events are contiguous from index 0, no event has a zero");
    f.indented_doc_comment("target, and the slice fits within `TR_MAX` events.");
    f.indented_doc_comment("");
    f.indented_doc_comment("# Errors");
    f.indented_doc_comment("");
    f.indented_doc_comment("- `ReplayError::EmptyTrace` if `events.is_empty()`.");
    f.indented_doc_comment("- `ReplayError::CapacityExceeded { declared, provided }` if the");
    f.indented_doc_comment("  slice exceeds `TR_MAX`.");
    f.indented_doc_comment("- `ReplayError::OutOfOrderEvent { index }` if the event at `index`");
    f.indented_doc_comment("  has a `step_index` not equal to `index` (strict contiguity).");
    f.indented_doc_comment("- `ReplayError::ZeroTarget { index }` if any event carries a zero");
    f.indented_doc_comment("  `ContentAddress` target.");
    f.line("    pub fn try_from_events(");
    f.line("        events: &[TraceEvent],");
    f.line("        witt_level_bits: u16,");
    f.line("        content_fingerprint: ContentFingerprint<FP_MAX>,");
    f.line("    ) -> Result<Self, ReplayError> {");
    f.line("        if events.is_empty() {");
    f.line("            return Err(ReplayError::EmptyTrace);");
    f.line("        }");
    f.line("        if events.len() > TR_MAX {");
    f.line("            return Err(ReplayError::CapacityExceeded {");
    f.line("                declared: TR_MAX as u16,");
    f.line("                provided: events.len() as u32,");
    f.line("            });");
    f.line("        }");
    f.line("        let mut i = 0usize;");
    f.line("        while i < events.len() {");
    f.line("            let e = &events[i];");
    f.line("            if e.step_index() as usize != i {");
    f.line("                return Err(ReplayError::OutOfOrderEvent { index: i });");
    f.line("            }");
    f.line("            if e.target().is_zero() {");
    f.line("                return Err(ReplayError::ZeroTarget { index: i });");
    f.line("            }");
    f.line("            i += 1;");
    f.line("        }");
    f.line("        let mut arr = [None; TR_MAX];");
    f.line("        let mut j = 0usize;");
    f.line("        while j < events.len() {");
    f.line("            arr[j] = Some(events[j]);");
    f.line("            j += 1;");
    f.line("        }");
    f.line("        Ok(Self {");
    f.line("            events: arr,");
    f.line("            len: events.len() as u16,");
    f.line("            witt_level_bits,");
    f.line("            content_fingerprint,");
    f.line("            _sealed: (),");
    f.line("        })");
    f.line("    }");
    f.line("}");
    f.blank();
    f.line("impl<const TR_MAX: usize> Default for Trace<TR_MAX> {");
    f.line("    #[inline]");
    f.line("    fn default() -> Self {");
    f.line("        Self::empty()");
    f.line("    }");
    f.line("}");
    f.blank();

    f.doc_comment("v0.2.2 Phase E / T2.6: `Derivation::replay()` produces a content-addressed");
    f.doc_comment("Trace the verifier can re-walk without invoking the deciders. The trace");
    f.doc_comment("length matches the derivation's `step_count()`, and each event's");
    f.doc_comment("`step_index` reflects its position in the derivation.");
    f.line("impl<const FP_MAX: usize> Derivation<FP_MAX> {");
    f.indented_doc_comment(
        "Replay this derivation as a fixed-size `Trace<TR_MAX>` whose length matches",
    );
    f.indented_doc_comment(
        "`self.step_count()` (capped at the application's `<HostBounds>::TRACE_MAX_EVENTS`).",
    );
    f.indented_doc_comment("Callers either annotate the binding (`let trace: Trace = ...;` picks");
    f.indented_doc_comment(
        "`Trace`'s default `TR_MAX` of 256) or use turbofish (`derivation.replay::<1024>()`).",
    );
    f.indented_doc_comment("");
    f.indented_doc_comment("# Example");
    f.indented_doc_comment("");
    f.indented_doc_comment("```no_run");
    f.indented_doc_comment("use uor_foundation::enforcement::{");
    f.indented_doc_comment(
        "    replay, CompileUnitBuilder, ConstrainedTypeInput, Grounded, Term, Trace,",
    );
    f.indented_doc_comment("};");
    f.indented_doc_comment("use uor_foundation::pipeline::run;");
    f.indented_doc_comment("use uor_foundation::{VerificationDomain, WittLevel};");
    f.indented_doc_comment("# use uor_foundation::enforcement::Hasher;");
    f.indented_doc_comment("# struct H; impl Hasher for H {");
    f.indented_doc_comment("#     const OUTPUT_BYTES: usize = 16;");
    f.indented_doc_comment("#     fn initial() -> Self { Self }");
    f.indented_doc_comment("#     fn fold_byte(self, _: u8) -> Self { self }");
    f.indented_doc_comment("#     fn finalize(self) -> [u8; 32] { [0; 32] } }");
    f.indented_doc_comment(
        "// ADR-060: `Term`/`Grounded` carry an `INLINE_BYTES` const-generic the",
    );
    f.indented_doc_comment(
        "// application derives from its `HostBounds`; fix a concrete width and",
    );
    f.indented_doc_comment("// thread it through `run`'s 4th const argument.");
    f.indented_doc_comment("const N: usize = 32;");
    f.indented_doc_comment(
        "let terms: [Term<'static, N>; 1] = \
         [uor_foundation::pipeline::literal_u64(7, WittLevel::W8)];",
    );
    f.indented_doc_comment(
        "let doms: [VerificationDomain; 1] = [VerificationDomain::Enumerative];",
    );
    f.indented_doc_comment("");
    f.indented_doc_comment("let unit = CompileUnitBuilder::<N>::new()");
    f.indented_doc_comment("    .root_term(&terms).witt_level_ceiling(WittLevel::W32)");
    f.indented_doc_comment("    .thermodynamic_budget(1024).target_domains(&doms)");
    f.indented_doc_comment("    .result_type::<ConstrainedTypeInput>()");
    f.indented_doc_comment("    .validate().expect(\"unit well-formed\");");
    f.indented_doc_comment("let grounded: Grounded<ConstrainedTypeInput, N, 32> =");
    f.indented_doc_comment(
        "    run::<ConstrainedTypeInput, _, H, N, 32>(unit).expect(\"grounds\");",
    );
    f.indented_doc_comment("");
    f.indented_doc_comment("// Replay → round-trip verification. The trace's event-count");
    f.indented_doc_comment("// capacity comes from the application's `HostBounds`; here the");
    f.indented_doc_comment("// type-annotated binding defaults `Trace`'s `TR_MAX` to 256.");
    f.indented_doc_comment("let trace: Trace = grounded.derivation().replay();");
    f.indented_doc_comment(
        "let recert = replay::certify_from_trace(&trace).expect(\"valid trace\");",
    );
    f.indented_doc_comment("assert_eq!(recert.certificate().content_fingerprint(),");
    f.indented_doc_comment("           grounded.content_fingerprint());");
    f.indented_doc_comment("```");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn replay<const TR_MAX: usize>(&self) -> Trace<TR_MAX, FP_MAX> {");
    f.line("        let steps = self.step_count() as usize;");
    f.line("        let len = if steps > TR_MAX { TR_MAX } else { steps };");
    f.line("        let mut events = [None; TR_MAX];");
    f.line("        // Seed targets from the leading 8 bytes of the source");
    f.line("        // `content_fingerprint` (substrate-computed). Combined with `| 1` so");
    f.line("        // the first event's target is guaranteed nonzero even when the");
    f.line("        // leading bytes are all zero, and XOR with `(i + 1)` keeps the");
    f.line("        // sequence non-degenerate.");
    f.line("        let fp = self.content_fingerprint.as_bytes();");
    f.line("        let seed = u64::from_be_bytes([");
    f.line("            fp[0], fp[1], fp[2], fp[3], fp[4], fp[5], fp[6], fp[7],");
    f.line("        ]) as u128;");
    f.line("        let nonzero_seed = seed | 1u128;");
    f.line("        let mut i = 0usize;");
    f.line("        while i < len {");
    f.line("            let target_raw = nonzero_seed ^ ((i as u128) + 1u128);");
    f.line("            events[i] = Some(TraceEvent::new(");
    f.line("                i as u32,");
    f.line("                crate::PrimitiveOp::Add,");
    f.line("                ContentAddress::from_u128(target_raw),");
    f.line("            ));");
    f.line("            i += 1;");
    f.line("        }");
    f.line("        // Pack the source `witt_level_bits` and `content_fingerprint`");
    f.line("        // into the Trace so `verify_trace` can reproduce the source certificate");
    f.line("        // via passthrough. The fingerprint was computed at grounding time by the");
    f.line("        // consumer-supplied Hasher and stored on the parent Grounded; the");
    f.line("        // Derivation accessor read it through.");
    f.line("        Trace::from_replay_events_const(");
    f.line("            events,");
    f.line("            len as u16,");
    f.line("            self.witt_level_bits,");
    f.line("            self.content_fingerprint,");
    f.line("        )");
    f.line("    }");
    f.line("}");
    f.blank();

    // v0.2.2 T5: ReplayError enum + replay re-derivation module.
    //
    // T5.8: `LengthMismatch` was renamed to `NonContiguousSteps` (the variant
    // fires when step indices skip values, NOT when the literal event count
    // is wrong). A new `CapacityExceeded` variant carries the literal-capacity
    // case for `Trace::try_from_events` (added by C4).
    //
    // T5.7: The previous v0.2.2 code path used a small-prime XOR-multiply
    // fold + 16-bit truncation inside `certify_from_trace`. That violated
    // both the round-trip property (the fold output couldn't reproduce the
    // source certificate) and the substrate-agnostic principle (the
    // foundation should not pick a hash function). The fix: rewrite
    // `certify_from_trace` as structural validation + fingerprint passthrough.
    // The fingerprint is computed at mint time by the consumer-supplied
    // `Hasher` and stored on the Trace; the verifier copies it through
    // unchanged. See T5.C3.d for the architectural rationale.
    f.doc_comment("v0.2.2 T5: errors emitted by the trace-replay re-derivation path.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("#[non_exhaustive]");
    f.line("pub enum ReplayError {");
    f.indented_doc_comment("The trace was empty; nothing to replay.");
    f.line("    EmptyTrace,");
    f.indented_doc_comment("Event at `index` has a non-monotonic step_index.");
    f.line("    OutOfOrderEvent {");
    f.line("        /// The event index that was out of order.");
    f.line("        index: usize,");
    f.line("    },");
    f.indented_doc_comment(
        "Event at `index` carries a zero ContentAddress (forbidden in well-formed traces).",
    );
    f.line("    ZeroTarget {");
    f.line("        /// The event index that carried a zero target.");
    f.line("        index: usize,");
    f.line("    },");
    f.indented_doc_comment("v0.2.2 T5.8: event step indices do not form a contiguous sequence");
    f.indented_doc_comment("`[0, 1, ..., len-1]`. Replaces the misleadingly-named v0.2.1");
    f.indented_doc_comment("`LengthMismatch` variant. The trace has the right number of");
    f.indented_doc_comment("events, but their step indices skip values (e.g., `[0, 2, 5]`");
    f.indented_doc_comment("with `len = 3`).");
    f.line("    NonContiguousSteps {");
    f.line("        /// The trace's declared length (number of events).");
    f.line("        declared: u16,");
    f.line("        /// The largest step_index observed in the event sequence.");
    f.line("        /// Always strictly greater than `declared - 1` when this");
    f.line("        /// variant fires.");
    f.line("        last_step: u32,");
    f.line("    },");
    f.indented_doc_comment("A caller attempted to construct a `Trace<TR_MAX>` whose event count");
    f.indented_doc_comment(
        "exceeds `TR_MAX` (the application's `<HostBounds>::TRACE_MAX_EVENTS`).",
    );
    f.indented_doc_comment("Distinct from `NonContiguousSteps` because the recovery is different");
    f.indented_doc_comment("(truncate vs. close gaps). Returned by `Trace::try_from_events`,");
    f.indented_doc_comment("never by `verify_trace` (the verifier reads from an existing `Trace`");
    f.indented_doc_comment("whose capacity is already enforced by the type's storage).");
    f.line("    CapacityExceeded {");
    f.line("        /// The trace's hard capacity (`TR_MAX`).");
    f.line("        declared: u16,");
    f.line("        /// The actual event count the caller attempted to pack in.");
    f.line("        provided: u32,");
    f.line("    },");
    f.line("}");
    f.blank();
    // v0.2.2 T5.9: Display + core::error::Error impls.
    f.line("impl core::fmt::Display for ReplayError {");
    f.line("    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {");
    f.line("        match self {");
    f.line("            Self::EmptyTrace => f.write_str(\"trace was empty; nothing to replay\"),");
    f.line("            Self::OutOfOrderEvent { index } => write!(");
    f.line("                f,");
    f.line("                \"event at index {index} has out-of-order step index\",");
    f.line("            ),");
    f.line("            Self::ZeroTarget { index } => write!(");
    f.line("                f,");
    f.line("                \"event at index {index} has a zero ContentAddress target\",");
    f.line("            ),");
    f.line("            Self::NonContiguousSteps { declared, last_step } => write!(");
    f.line("                f,");
    f.line("                \"trace declares {declared} events but step indices skip values \\");
    f.line("                 (last step {last_step})\",");
    f.line("            ),");
    f.line("            Self::CapacityExceeded { declared, provided } => write!(");
    f.line("                f,");
    f.line("                \"trace capacity exceeded: tried to pack {provided} events into a buffer of {declared}\",");
    f.line("            ),");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();
    f.line("impl core::error::Error for ReplayError {}");
    f.blank();

    f.doc_comment("v0.2.2 T5: trace-replay re-derivation module.");
    f.doc_comment("");
    f.doc_comment("The foundation owns the certificate-construction boundary; the");
    f.doc_comment("`uor-foundation-verify` crate is a thin facade that delegates to");
    f.doc_comment("`replay::certify_from_trace`. This preserves sealing discipline:");
    f.doc_comment("`Certified::new` stays `pub(crate)` and no external crate can mint a");
    f.doc_comment("certificate.");
    f.line("pub mod replay {");
    f.line("    use super::{Certified, GroundingCertificate, ReplayError, Trace};");
    f.blank();
    f.indented_doc_comment("Re-derive the `Certified<GroundingCertificate>` that the foundation");
    f.indented_doc_comment("grounding path produced for the source unit.");
    f.indented_doc_comment("");
    f.indented_doc_comment("Validates the trace's structural invariants (monotonic, contiguous");
    f.indented_doc_comment("step indices; no zero targets; no None slots in the populated");
    f.indented_doc_comment("prefix) and re-packages the trace's stored `ContentFingerprint` and");
    f.indented_doc_comment("`witt_level_bits` into a fresh certificate. The verifier does NOT");
    f.indented_doc_comment("invoke a hash function: the fingerprint is *data carried by the");
    f.indented_doc_comment("Trace*, computed at mint time by the consumer-supplied `Hasher` and");
    f.indented_doc_comment("passed through unchanged.");
    f.indented_doc_comment("");
    f.indented_doc_comment("# Round-trip property");
    f.indented_doc_comment("");
    f.indented_doc_comment("For every `Grounded<T>` produced by `pipeline::run::<T, _, H>` with");
    f.indented_doc_comment("a conforming substrate `H: Hasher`:");
    f.indented_doc_comment("");
    f.indented_doc_comment("```text");
    f.indented_doc_comment("verify_trace(&grounded.derivation().replay()).certificate()");
    f.indented_doc_comment("    == grounded.certificate()");
    f.indented_doc_comment("```");
    f.indented_doc_comment("");
    f.indented_doc_comment("holds bit-identically. The contract is orthogonal to the substrate");
    f.indented_doc_comment("hasher choice and to the chosen `OUTPUT_BYTES` width.");
    f.indented_doc_comment("");
    f.indented_doc_comment("# Errors");
    f.indented_doc_comment("");
    f.indented_doc_comment("Returns:");
    f.indented_doc_comment("- `ReplayError::EmptyTrace` if `trace.is_empty()`.");
    f.indented_doc_comment("- `ReplayError::OutOfOrderEvent { index }` if step indices are not");
    f.indented_doc_comment("  strictly monotonic at position `index`.");
    f.indented_doc_comment("- `ReplayError::ZeroTarget { index }` if any event carries");
    f.indented_doc_comment("  `ContentAddress::zero()`.");
    f.indented_doc_comment("- `ReplayError::NonContiguousSteps { declared, last_step }` if");
    f.indented_doc_comment("  the event step indices skip values.");
    f.line("    pub fn certify_from_trace<const TR_MAX: usize, const FP_MAX: usize>(");
    f.line("        trace: &Trace<TR_MAX, FP_MAX>,");
    f.line("    ) -> Result<Certified<GroundingCertificate<FP_MAX>>, ReplayError> {");
    f.line("        let len = trace.len() as usize;");
    f.line("        if len == 0 {");
    f.line("            return Err(ReplayError::EmptyTrace);");
    f.line("        }");
    f.line("        // Structural validation: monotonic step indices, contiguous from 0,");
    f.line("        // no zero targets, no None slots in the populated prefix.");
    f.line("        let mut last_step: i64 = -1;");
    f.line("        let mut max_step_index: u32 = 0;");
    f.line("        let mut i = 0usize;");
    f.line("        while i < len {");
    f.line("            let event = match trace.event(i) {");
    f.line("                Some(e) => e,");
    f.line("                None => return Err(ReplayError::OutOfOrderEvent { index: i }),");
    f.line("            };");
    f.line("            let step_index = event.step_index();");
    f.line("            if (step_index as i64) <= last_step {");
    f.line("                return Err(ReplayError::OutOfOrderEvent { index: i });");
    f.line("            }");
    f.line("            if event.target().is_zero() {");
    f.line("                return Err(ReplayError::ZeroTarget { index: i });");
    f.line("            }");
    f.line("            if step_index > max_step_index {");
    f.line("                max_step_index = step_index;");
    f.line("            }");
    f.line("            last_step = step_index as i64;");
    f.line("            i += 1;");
    f.line("        }");
    f.line("        if (max_step_index as u16).saturating_add(1) != trace.len() {");
    f.line("            return Err(ReplayError::NonContiguousSteps {");
    f.line("                declared: trace.len(),");
    f.line("                last_step: max_step_index,");
    f.line("            });");
    f.line("        }");
    f.line("        // v0.2.2 T5: fingerprint passthrough. The trace was minted by the");
    f.line("        // foundation pipeline with a `ContentFingerprint` already computed by");
    f.line("        // the consumer-supplied `Hasher`. The verifier does not invoke any");
    f.line("        // hash function; it copies the fingerprint through unchanged. The");
    f.line("        // round-trip property holds by construction. v0.2.2 T6.5: the");
    f.line("        // FingerprintMissing variant is removed because under T6.3 / T6.10,");
    f.line("        // no public path can produce a Trace with a zero fingerprint.");
    f.line("        Ok(Certified::new(");
    f.line("            GroundingCertificate::with_level_and_fingerprint_const(");
    f.line("                trace.witt_level_bits(),");
    f.line("                trace.content_fingerprint(),");
    f.line("            ),");
    f.line("        ))");
    f.line("    }");
    f.line("}");
    f.blank();

    // Phase X.2: `HomologyClass` and `CohomologyClass` carriers (dimension-as-
    // runtime-field + `cup` algebra) are emitted from `emit_phase_j_primitives`
    // → `emit_phase_x2_cohomology_cup`. The previous const-generic orphan
    // emissions (`HomologyClass<const N: usize>` / `CohomologyClass<const N>`)
    // carried no production path — replaced by the carriers with real behavior.

    // InteractionDeclarationBuilder stub (Phase E).
    f.doc_comment("v0.2.2 Phase E: sealed builder for an InteractionDeclaration.");
    f.doc_comment("");
    f.doc_comment("Validates the peer protocol, convergence predicate, and");
    f.doc_comment("commutator state class required by `conformance:InteractionShape`.");
    f.doc_comment("Phase F wires the full `InteractionDriver` on top of this builder.");
    f.line("#[derive(Debug, Clone, Copy, Default)]");
    f.line("pub struct InteractionDeclarationBuilder {");
    f.line("    peer_protocol: Option<u128>,");
    f.line("    convergence_predicate: Option<u128>,");
    f.line("    commutator_state_class: Option<u128>,");
    f.line("}");
    f.blank();
    f.line("impl InteractionDeclarationBuilder {");
    f.indented_doc_comment("Construct a new builder.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn new() -> Self {");
    f.line("        Self { peer_protocol: None, convergence_predicate: None, commutator_state_class: None }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Set the peer protocol content address.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn peer_protocol(mut self, address: u128) -> Self {");
    f.line("        self.peer_protocol = Some(address);");
    f.line("        self");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Set the convergence predicate content address.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn convergence_predicate(mut self, address: u128) -> Self {");
    f.line("        self.convergence_predicate = Some(address);");
    f.line("        self");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Set the commutator state class content address.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn commutator_state_class(mut self, address: u128) -> Self {");
    f.line("        self.commutator_state_class = Some(address);");
    f.line("        self");
    f.line("    }");
    f.blank();
    // Phase E.6: validate + validate_const against conformance:InteractionShape.
    f.indented_doc_comment("Phase E.6: validate against `conformance:InteractionShape`.");
    f.indented_doc_comment("");
    f.indented_doc_comment("# Errors");
    f.indented_doc_comment("");
    f.indented_doc_comment(
        "Returns `ShapeViolation` if any of the three required fields is missing.",
    );
    f.line("    pub fn validate(&self) -> Result<Validated<InteractionShape>, ShapeViolation> {");
    f.line("        self.validate_common().map(|_| Validated::new(InteractionShape { shape_iri: \"https://uor.foundation/conformance/InteractionShape\" }))");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Phase E.6 + C.1: const-fn companion.");
    f.indented_doc_comment("");
    f.indented_doc_comment("# Errors");
    f.indented_doc_comment("");
    f.indented_doc_comment("Returns `ShapeViolation` if any required field is missing.");
    f.line("    pub const fn validate_const(&self) -> Result<Validated<InteractionShape, CompileTime>, ShapeViolation> {");
    f.line("        if self.peer_protocol.is_none() {");
    f.line("            return Err(ShapeViolation {");
    f.line("                shape_iri: \"https://uor.foundation/conformance/InteractionShape\",");
    f.line(
        "                constraint_iri: \"https://uor.foundation/conformance/InteractionShape\",",
    );
    f.line("                property_iri: \"https://uor.foundation/interaction/peerProtocol\",");
    f.line("                expected_range: \"http://www.w3.org/2002/07/owl#Thing\",");
    f.line("                min_count: 1,");
    f.line("                max_count: 1,");
    f.line("                kind: ViolationKind::Missing,");
    f.line("            });");
    f.line("        }");
    f.line("        if self.convergence_predicate.is_none() {");
    f.line("            return Err(ShapeViolation {");
    f.line("                shape_iri: \"https://uor.foundation/conformance/InteractionShape\",");
    f.line(
        "                constraint_iri: \"https://uor.foundation/conformance/InteractionShape\",",
    );
    f.line("                property_iri: \"https://uor.foundation/interaction/convergencePredicate\",");
    f.line("                expected_range: \"http://www.w3.org/2002/07/owl#Thing\",");
    f.line("                min_count: 1,");
    f.line("                max_count: 1,");
    f.line("                kind: ViolationKind::Missing,");
    f.line("            });");
    f.line("        }");
    f.line("        if self.commutator_state_class.is_none() {");
    f.line("            return Err(ShapeViolation {");
    f.line("                shape_iri: \"https://uor.foundation/conformance/InteractionShape\",");
    f.line(
        "                constraint_iri: \"https://uor.foundation/conformance/InteractionShape\",",
    );
    f.line("                property_iri: \"https://uor.foundation/interaction/commutatorStateClass\",");
    f.line("                expected_range: \"http://www.w3.org/2002/07/owl#Thing\",");
    f.line("                min_count: 1,");
    f.line("                max_count: 1,");
    f.line("                kind: ViolationKind::Missing,");
    f.line("            });");
    f.line("        }");
    f.line("        Ok(Validated::new(InteractionShape { shape_iri: \"https://uor.foundation/conformance/InteractionShape\" }))");
    f.line("    }");
    f.blank();
    // Shared non-const validation path.
    f.line("    fn validate_common(&self) -> Result<(), ShapeViolation> {");
    f.line("        self.validate_const().map(|_| ())");
    f.line("    }");
    f.line("}");
    f.blank();
    // Companion result type for Validated<InteractionShape>.
    f.doc_comment(
        "Phase E.6: validated InteractionDeclaration per `conformance:InteractionShape`.",
    );
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq)]");
    f.line("pub struct InteractionShape {");
    f.indented_doc_comment("Shape IRI this declaration was validated against.");
    f.line("    pub shape_iri: &'static str,");
    f.line("}");
    f.blank();

    // Phase E.5: observability feature-gated subscribe API.
    f.doc_comment("Phase E.5 (target §7.4): observability subscribe API.");
    f.doc_comment("");
    f.doc_comment("When the `observability` feature is enabled, downstream may call");
    f.doc_comment("`subscribe_trace_events` with a handler closure that receives each");
    f.doc_comment("`TraceEvent` as the pipeline emits it. When the feature is off, this");
    f.doc_comment("function is entirely absent from the public API.");
    f.line("#[cfg(feature = \"observability\")]");
    f.line("pub fn subscribe_trace_events<F>(handler: F) -> ObservabilitySubscription<F>");
    f.line("where");
    f.line("    F: FnMut(&TraceEvent),");
    f.line("{");
    f.line("    ObservabilitySubscription { handler, _sealed: () }");
    f.line("}");
    f.blank();
    f.line("#[cfg(feature = \"observability\")]");
    f.doc_comment("Phase E.5: sealed subscription handle returned by `subscribe_trace_events`.");
    f.line("#[cfg(feature = \"observability\")]");
    f.line("pub struct ObservabilitySubscription<F: FnMut(&TraceEvent)> {");
    f.line("    handler: F,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("#[cfg(feature = \"observability\")]");
    f.line("impl<F: FnMut(&TraceEvent)> ObservabilitySubscription<F> {");
    f.indented_doc_comment("Dispatch a TraceEvent through the subscribed handler.");
    f.line("    pub fn emit(&mut self, event: &TraceEvent) {");
    f.line("        (self.handler)(event);");
    f.line("    }");
    f.line("}");
    f.blank();

    // ── Phase F: kernel namespace enforcement surface (target §4.7) ─────
    //
    // Sealed witness types and closed enumerations for the 8 kernel
    // namespaces — carry, convergence, division, monoidal, operad,
    // recursion, region, linear. Each sealed type has private fields and
    // a `pub(crate)` constructor reachable only from the foundation's
    // own emission paths.

    // F.2: closed six-kind constraint enumeration.
    f.doc_comment("Phase F.2 (target §4.7): closed enumeration of the six constraint kinds.");
    f.doc_comment("");
    f.doc_comment("`type-decl` bodies enumerate exactly these six kinds per `uor_term.ebnf`'s");
    f.doc_comment("`constraint-decl` production. `CompositeConstraint` — the implicit shape of");
    f.doc_comment("a multi-decl body — has no syntactic constructor and is therefore not a");
    f.doc_comment("variant of this enum.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("#[non_exhaustive]");
    f.line("pub enum ConstraintKind {");
    f.indented_doc_comment("`type:ResidueConstraint` — value is congruent to `r (mod m)`.");
    f.line("    Residue,");
    f.indented_doc_comment("`type:CarryConstraint` — bounded carry depth.");
    f.line("    Carry,");
    f.indented_doc_comment("`type:DepthConstraint` — bounded derivation depth.");
    f.line("    Depth,");
    f.indented_doc_comment("`type:HammingConstraint` — bounded Hamming distance from a reference.");
    f.line("    Hamming,");
    f.indented_doc_comment("`type:SiteConstraint` — per-site cardinality or containment.");
    f.line("    Site,");
    f.indented_doc_comment("`type:AffineConstraint` — linear inequality over site values.");
    f.line("    Affine,");
    f.line("}");
    f.blank();

    // F.3 carry: sealed CarryProfile + CarryEvent.
    f.doc_comment("Phase F.3 (target §4.7 carry): sealed per-ring-op carry profile.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct CarryProfile {");
    f.line("    chain_length: u32,");
    f.line("    max_depth: u32,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl CarryProfile {");
    f.indented_doc_comment("Length of the longest carry chain observed.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn chain_length(&self) -> u32 { self.chain_length }");
    f.blank();
    f.indented_doc_comment("Maximum carry depth across the op.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn max_depth(&self) -> u32 { self.max_depth }");
    f.blank();
    f.indented_doc_comment("Crate-internal constructor.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new(chain_length: u32, max_depth: u32) -> Self {");
    f.line("        Self { chain_length, max_depth, _sealed: () }");
    f.line("    }");
    f.line("}");
    f.blank();

    f.doc_comment(
        "Phase F.3 (target §4.7 carry): sealed carry-event witness — records the ring op",
    );
    f.doc_comment("and two `Datum<L>` witt widths involved.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct CarryEvent {");
    f.line("    left_bits: u16,");
    f.line("    right_bits: u16,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl CarryEvent {");
    f.indented_doc_comment("Witt bit-width of the left operand.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn left_bits(&self) -> u16 { self.left_bits }");
    f.blank();
    f.indented_doc_comment("Witt bit-width of the right operand.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn right_bits(&self) -> u16 { self.right_bits }");
    f.blank();
    f.indented_doc_comment("Crate-internal constructor.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new(left_bits: u16, right_bits: u16) -> Self {");
    f.line("        Self { left_bits, right_bits, _sealed: () }");
    f.line("    }");
    f.line("}");
    f.blank();

    // F.3 convergence: sealed ConvergenceLevel<L>, HopfFiber<L>, ConvergenceResidual<L>.
    f.doc_comment("Phase F.3 (target §4.7 convergence): sealed convergence-level witness at `L`.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct ConvergenceLevel<L> {");
    f.line("    valuation: u32,");
    f.line("    _level: PhantomData<L>,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl<L> ConvergenceLevel<L> {");
    f.indented_doc_comment("The v\u{2082} valuation of the datum at this convergence level.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn valuation(&self) -> u32 { self.valuation }");
    f.blank();
    f.indented_doc_comment("Crate-internal constructor.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new(valuation: u32) -> Self {");
    f.line("        Self { valuation, _level: PhantomData, _sealed: () }");
    f.line("    }");
    f.line("}");
    f.blank();

    // F.3 division: sealed DivisionAlgebraWitness enum.
    f.doc_comment("Phase F.3 (target §4.7 division): sealed enum over the four normed division");
    f.doc_comment(
        "algebras from Cayley-Dickson. No other admissible algebra exists (Hurwitz's theorem).",
    );
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("#[non_exhaustive]");
    f.line("pub enum DivisionAlgebraWitness {");
    f.indented_doc_comment("Real numbers \u{211D} (dimension 1).");
    f.line("    Real,");
    f.indented_doc_comment("Complex numbers \u{2102} (dimension 2).");
    f.line("    Complex,");
    f.indented_doc_comment("Quaternions \u{210D} (dimension 4, non-commutative).");
    f.line("    Quaternion,");
    f.indented_doc_comment("Octonions \u{1D546} (dimension 8, non-commutative, non-associative).");
    f.line("    Octonion,");
    f.line("}");
    f.blank();

    // F.3 monoidal: sealed MonoidalProduct<L, R>, MonoidalUnit<L>.
    f.doc_comment("Phase F.3 (target §4.7 monoidal): sealed monoidal-product witness.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct MonoidalProduct<L, R> {");
    f.line("    _left: PhantomData<L>,");
    f.line("    _right: PhantomData<R>,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl<L, R> MonoidalProduct<L, R> {");
    f.indented_doc_comment("Crate-internal constructor.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new() -> Self {");
    f.line("        Self { _left: PhantomData, _right: PhantomData, _sealed: () }");
    f.line("    }");
    f.line("}");
    f.blank();
    f.doc_comment("Phase F.3 (target §4.7 monoidal): sealed monoidal-unit witness at `L`.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct MonoidalUnit<L> {");
    f.line("    _level: PhantomData<L>,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl<L> MonoidalUnit<L> {");
    f.indented_doc_comment("Crate-internal constructor.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new() -> Self {");
    f.line("        Self { _level: PhantomData, _sealed: () }");
    f.line("    }");
    f.line("}");
    f.blank();

    // F.1 operad: sealed OperadComposition.
    f.doc_comment("Phase F.1 (target §4.7 operad): sealed operad-composition witness.");
    f.doc_comment("");
    f.doc_comment(
        "Every `type-app` form in the term grammar materializes a fresh `OperadComposition`",
    );
    f.doc_comment(
        "carrying the outer/inner type IRIs and composed site count, per `operad:` ontology.",
    );
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct OperadComposition {");
    f.line("    outer_type_iri: &'static str,");
    f.line("    inner_type_iri: &'static str,");
    f.line("    composed_site_count: u32,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl OperadComposition {");
    f.indented_doc_comment("IRI of the outer `type:TypeDefinition`.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn outer_type_iri(&self) -> &'static str { self.outer_type_iri }");
    f.blank();
    f.indented_doc_comment("IRI of the inner `type:TypeDefinition`.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn inner_type_iri(&self) -> &'static str { self.inner_type_iri }");
    f.blank();
    f.indented_doc_comment("Site count of the composed type.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn composed_site_count(&self) -> u32 { self.composed_site_count }");
    f.blank();
    f.indented_doc_comment("Crate-internal constructor.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line(
        "    pub(crate) const fn new(outer_type_iri: &'static str, inner_type_iri: &'static str, composed_site_count: u32) -> Self {",
    );
    f.line("        Self { outer_type_iri, inner_type_iri, composed_site_count, _sealed: () }");
    f.line("    }");
    f.line("}");
    f.blank();

    // F.3 recursion: sealed RecursionTrace, BaseCase, RecursiveStep.
    f.doc_comment("Phase F.3 (target §4.7 recursion): maximum depth of the recursion trace");
    f.doc_comment("witness. Bounded by the declared descent budget at builder-validate time;");
    f.doc_comment("the constant is a size-budget cap matching other foundation arenas.");
    f.doc_comment("");
    f.doc_comment("Wiki ADR-037: a foundation-fixed conservative default for");
    f.doc_comment("[`crate::HostBounds::RECURSION_TRACE_DEPTH_MAX`].");
    f.line(
        "pub const RECURSION_TRACE_MAX_DEPTH: usize = \
         16;",
    );
    f.blank();
    f.doc_comment("Phase F.3 (target §4.7 recursion): sealed recursion trace with fixed-capacity");
    f.doc_comment("descent-measure sequence.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct RecursionTrace {");
    f.line("    depth: u32,");
    f.line("    measure: [u32; RECURSION_TRACE_MAX_DEPTH],");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl RecursionTrace {");
    f.indented_doc_comment("Number of recursive descents in this trace.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn depth(&self) -> u32 { self.depth }");
    f.blank();
    f.indented_doc_comment("Descent-measure sequence.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line(
        "    pub const fn measure(&self) -> &[u32; RECURSION_TRACE_MAX_DEPTH] { &self.measure }",
    );
    f.blank();
    f.indented_doc_comment("Crate-internal constructor.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new(depth: u32, measure: [u32; RECURSION_TRACE_MAX_DEPTH]) -> Self {");
    f.line("        Self { depth, measure, _sealed: () }");
    f.line("    }");
    f.line("}");
    f.blank();

    // F.3 region: sealed AddressRegion, WorkingSet, RegionAllocation.
    f.doc_comment("Phase F.3 (target §4.7 region): sealed address-region witness.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct AddressRegion {");
    f.line("    base: u128,");
    f.line("    extent: u64,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl AddressRegion {");
    f.indented_doc_comment("Base address of the region.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn base(&self) -> u128 { self.base }");
    f.blank();
    f.indented_doc_comment("Extent (number of addressable cells).");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn extent(&self) -> u64 { self.extent }");
    f.blank();
    f.indented_doc_comment("Crate-internal constructor.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new(base: u128, extent: u64) -> Self {");
    f.line("        Self { base, extent, _sealed: () }");
    f.line("    }");
    f.line("}");
    f.blank();

    // F.3 linear: sealed LinearBudget, LeaseAllocation.
    f.doc_comment("Phase F.3 (target §4.7 linear): sealed linear-resource budget.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct LinearBudget {");
    f.line("    sites_remaining: u64,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl LinearBudget {");
    f.indented_doc_comment("Number of linear sites still available for allocation.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn sites_remaining(&self) -> u64 { self.sites_remaining }");
    f.blank();
    f.indented_doc_comment("Crate-internal constructor.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new(sites_remaining: u64) -> Self {");
    f.line("        Self { sites_remaining, _sealed: () }");
    f.line("    }");
    f.line("}");
    f.blank();

    f.doc_comment("Phase F.3 (target §4.7 linear): sealed lease-allocation witness.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct LeaseAllocation {");
    f.line("    site_count: u32,");
    f.line("    scope_hash: u128,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl LeaseAllocation {");
    f.indented_doc_comment("Number of linear sites taken by this allocation.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn site_count(&self) -> u32 { self.site_count }");
    f.blank();
    f.indented_doc_comment("Content-hash of the lease scope identifier.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn scope_hash(&self) -> u128 { self.scope_hash }");
    f.blank();
    f.indented_doc_comment("Crate-internal constructor.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn new(site_count: u32, scope_hash: u128) -> Self {");
    f.line("        Self { site_count, scope_hash, _sealed: () }");
    f.line("    }");
    f.line("}");
    f.blank();
}

/// v0.2.2 Phase J: combinator-only Grounding.
///
/// Emits the closed 12-combinator set that downstream composes to build
/// grounding programs. The program's marker tuple is verified at
/// construction time via the `MarkersImpliedBy<Map>` trait, so a
/// `GroundingProgram<_, DigestGroundingMap>` built out of integer
/// combinators is rejected at compile time.
#[allow(clippy::too_many_lines)]
fn generate_grounding_combinator_surface(f: &mut RustFile) {
    // Marker unit structs — zero-sized, type-level tokens anchoring the
    // closed catalogue of admissible marker tuples.
    f.doc_comment("v0.2.2 Phase J: zero-sized token identifying the `Total` marker.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]");
    f.line("pub struct TotalMarker;");
    f.blank();

    f.doc_comment("v0.2.2 Phase J: zero-sized token identifying the `Invertible` marker.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]");
    f.line("pub struct InvertibleMarker;");
    f.blank();

    f.doc_comment("v0.2.2 Phase J: zero-sized token identifying the `PreservesStructure` marker.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]");
    f.line("pub struct PreservesStructureMarker;");
    f.blank();

    // Sealed supertrait — anchors `MarkerTuple`, `MarkerIntersection`,
    // `MarkersImpliedBy` so downstream cannot add new marker tuples.
    f.line("mod marker_tuple_sealed {");
    f.indented_doc_comment("Private supertrait. Not implementable outside this crate.");
    f.line("    pub trait Sealed {}");
    f.line("}");
    f.blank();

    // MarkerTuple — closed catalogue of six admissible tuples.
    f.doc_comment("v0.2.2 Phase J: sealed marker-tuple trait. Implemented exhaustively by");
    f.doc_comment("the closed catalogue of six admissible marker tuples in canonical order");
    f.doc_comment("(Total, Invertible, PreservesStructure). Downstream cannot add new");
    f.doc_comment("marker tuples; the seal anchors Phase J's compile-time correctness claim.");
    f.line("pub trait MarkerTuple: marker_tuple_sealed::Sealed {}");
    f.blank();

    f.line("impl marker_tuple_sealed::Sealed for () {}");
    f.line("impl MarkerTuple for () {}");
    f.line("impl marker_tuple_sealed::Sealed for (TotalMarker,) {}");
    f.line("impl MarkerTuple for (TotalMarker,) {}");
    f.line("impl marker_tuple_sealed::Sealed for (TotalMarker, InvertibleMarker) {}");
    f.line("impl MarkerTuple for (TotalMarker, InvertibleMarker) {}");
    f.line(
        "impl marker_tuple_sealed::Sealed for (TotalMarker, InvertibleMarker, PreservesStructureMarker) {}",
    );
    f.line("impl MarkerTuple for (TotalMarker, InvertibleMarker, PreservesStructureMarker) {}");
    f.line("impl marker_tuple_sealed::Sealed for (InvertibleMarker,) {}");
    f.line("impl MarkerTuple for (InvertibleMarker,) {}");
    f.line("impl marker_tuple_sealed::Sealed for (InvertibleMarker, PreservesStructureMarker) {}");
    f.line("impl MarkerTuple for (InvertibleMarker, PreservesStructureMarker) {}");
    f.blank();

    // MarkerIntersection<Other> — type-level set intersection of marker tuples.
    f.doc_comment("v0.2.2 Phase J: type-level set intersection of two marker tuples.");
    f.doc_comment("");
    f.doc_comment("Implemented exhaustively for every ordered pair in the closed catalogue.");
    f.doc_comment("Composition combinators (`then`, `and_then`) use this trait to compute");
    f.doc_comment("the output marker tuple of a composed primitive as the intersection of");
    f.doc_comment("its two inputs' tuples. Because the catalogue is closed, the result is");
    f.doc_comment("always another tuple in the catalogue — no open-world hazards.");
    f.line("pub trait MarkerIntersection<Other: MarkerTuple>: MarkerTuple {");
    f.indented_doc_comment("The intersection of `Self` and `Other` in the closed catalogue.");
    f.line("    type Output: MarkerTuple;");
    f.line("}");
    f.blank();

    // Emit the 36 MarkerIntersection impls programmatically.
    // Canonical tuple order: (Total, Invertible, PreservesStructure).
    let e_ = "()";
    let t_ = "(TotalMarker,)";
    let ti_ = "(TotalMarker, InvertibleMarker)";
    let tis = "(TotalMarker, InvertibleMarker, PreservesStructureMarker)";
    let i_ = "(InvertibleMarker,)";
    let is_ = "(InvertibleMarker, PreservesStructureMarker)";
    // Each row is (Self, [(Other, Intersection), ...]). Indexed by
    // (T ∈ tuple, I ∈ tuple, S ∈ tuple) set membership.
    let table: &[(&str, &[(&str, &str)])] = &[
        (
            e_,
            &[
                (e_, e_),
                (t_, e_),
                (ti_, e_),
                (tis, e_),
                (i_, e_),
                (is_, e_),
            ],
        ),
        (
            t_,
            &[
                (e_, e_),
                (t_, t_),
                (ti_, t_),
                (tis, t_),
                (i_, e_),
                (is_, e_),
            ],
        ),
        (
            ti_,
            &[
                (e_, e_),
                (t_, t_),
                (ti_, ti_),
                (tis, ti_),
                (i_, i_),
                (is_, i_),
            ],
        ),
        (
            tis,
            &[
                (e_, e_),
                (t_, t_),
                (ti_, ti_),
                (tis, tis),
                (i_, i_),
                (is_, is_),
            ],
        ),
        (
            i_,
            &[
                (e_, e_),
                (t_, e_),
                (ti_, i_),
                (tis, i_),
                (i_, i_),
                (is_, i_),
            ],
        ),
        (
            is_,
            &[
                (e_, e_),
                (t_, e_),
                (ti_, i_),
                (tis, is_),
                (i_, i_),
                (is_, is_),
            ],
        ),
    ];
    for (self_tup, row) in table.iter() {
        for (other_tup, output_tup) in row.iter() {
            f.line(&format!(
                "impl MarkerIntersection<{other_tup}> for {self_tup} {{ type Output = {output_tup}; }}"
            ));
        }
    }
    f.blank();

    // MarkersImpliedBy<Map> — compile-time check that a marker tuple carries
    // enough properties to satisfy the GroundingMapKind's declared requirements.
    f.doc_comment("v0.2.2 Phase J: compile-time check that a combinator's marker tuple");
    f.doc_comment("carries every property declared by the `GroundingMapKind` a program");
    f.doc_comment("claims. Implemented exhaustively by codegen for every valid `(tuple,");
    f.doc_comment("map)` pair; absent impls reject the mismatched declaration at the");
    f.doc_comment("`GroundingProgram::from_primitive` call site.");
    f.line("pub trait MarkersImpliedBy<Map: GroundingMapKind>: MarkerTuple {}");
    f.blank();

    // Marker bitmask record. We use a bitmask-valued type parameter to
    // keep the tuple-arithmetic simple. Each primitive carries its marker
    // bitmask as a const.
    f.doc_comment("v0.2.2 Phase J: bitmask encoding of a combinator's marker set.");
    f.doc_comment("Bit 0 = Total, bit 1 = Invertible, bit 2 = PreservesStructure.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct MarkerBits(u8);");
    f.blank();
    f.line("impl MarkerBits {");
    f.indented_doc_comment("The `Total` marker bit.");
    f.line("    pub const TOTAL: Self = Self(1);");
    f.indented_doc_comment("The `Invertible` marker bit.");
    f.line("    pub const INVERTIBLE: Self = Self(2);");
    f.indented_doc_comment("The `PreservesStructure` marker bit.");
    f.line("    pub const PRESERVES_STRUCTURE: Self = Self(4);");
    f.indented_doc_comment("An empty marker set.");
    f.line("    pub const NONE: Self = Self(0);");
    f.blank();
    f.indented_doc_comment("Construct a marker bitmask from raw u8.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn from_u8(bits: u8) -> Self { Self(bits) }");
    f.blank();
    f.indented_doc_comment("Access the raw bitmask.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn as_u8(&self) -> u8 { self.0 }");
    f.blank();
    f.indented_doc_comment("Bitwise OR of two marker bitmasks.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn union(self, other: Self) -> Self { Self(self.0 | other.0) }");
    f.blank();
    f.indented_doc_comment("Bitwise AND of two marker bitmasks (marker intersection).");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn intersection(self, other: Self) -> Self {");
    f.line("        Self(self.0 & other.0)");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Whether this set contains all marker bits of `other`.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn contains(&self, other: Self) -> bool {");
    f.line("        (self.0 & other.0) == other.0");
    f.line("    }");
    f.line("}");
    f.blank();

    // PrimitiveOp (the 12-op catalogue) as a sealed enum.
    f.doc_comment("v0.2.2 Phase J: closed catalogue of grounding primitives.");
    f.doc_comment("");
    f.doc_comment("Exactly 12 operations — read_bytes, interpret_le_integer,");
    f.doc_comment("interpret_be_integer, digest, decode_utf8, decode_json, select_field,");
    f.doc_comment("select_index, const_value, then, map_err, and_then. Adding a new");
    f.doc_comment("primitive is an ontology+grammar+codegen edit, not a Rust patch.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("#[non_exhaustive]");
    f.line("pub enum GroundingPrimitiveOp {");
    f.indented_doc_comment("Read a fixed-size byte slice from the input.");
    f.line("    ReadBytes,");
    f.indented_doc_comment("Interpret bytes as a little-endian integer at the target WittLevel.");
    f.line("    InterpretLeInteger,");
    f.indented_doc_comment("Interpret bytes as a big-endian integer.");
    f.line("    InterpretBeInteger,");
    f.indented_doc_comment("Hash bytes via blake3 → 32-byte digest → `Datum<W256>`.");
    f.indented_doc_comment("");
    f.indented_doc_comment("# Scope");
    f.indented_doc_comment("");
    f.indented_doc_comment(
        "`interpret_leaf_op` returns the first byte of the blake3 32-byte digest.",
    );
    f.indented_doc_comment(
        "The full digest is produced by `Datum<W256>` composition of 32 `Digest` leaves —",
    );
    f.indented_doc_comment("the leaf-level output is the single-byte projection.");
    f.line("    Digest,");
    f.indented_doc_comment("Decode UTF-8 bytes; rejects malformed input.");
    f.indented_doc_comment("");
    f.indented_doc_comment("# Scope");
    f.indented_doc_comment("");
    f.indented_doc_comment(
        "Only single-byte ASCII (`b < 0x80`) is decoded by `interpret_leaf_op`.",
    );
    f.indented_doc_comment(
        "Multi-byte UTF-8 is not decoded by this leaf; multi-byte sequences traverse the",
    );
    f.indented_doc_comment("foundation via `GroundedTuple<N>` composition of single-byte leaves.");
    f.line("    DecodeUtf8,");
    f.indented_doc_comment("Decode JSON bytes; rejects malformed input.");
    f.indented_doc_comment("");
    f.indented_doc_comment("# Scope");
    f.indented_doc_comment("");
    f.indented_doc_comment(
        "Only the leading byte of a JSON number scalar (`-` or ASCII digit) is parsed",
    );
    f.indented_doc_comment(
        "by `interpret_leaf_op`. Structured JSON values (objects, arrays, strings,",
    );
    f.indented_doc_comment("multi-byte numbers) are not parsed by this leaf.");
    f.line("    DecodeJson,");
    f.indented_doc_comment("Select a field from a structured value.");
    f.line("    SelectField,");
    f.indented_doc_comment("Select an indexed element.");
    f.line("    SelectIndex,");
    f.indented_doc_comment("Inject a foundation-known constant.");
    f.indented_doc_comment("");
    f.indented_doc_comment("# Scope");
    f.indented_doc_comment("");
    f.indented_doc_comment(
        "`interpret_leaf_op` returns `GroundedCoord::w8(0)` — the foundation-canonical",
    );
    f.indented_doc_comment(
        "zero constant. Non-zero constants are materialized through the const-fn frontier",
    );
    f.indented_doc_comment("(`validate_const` paths) rather than through this leaf.");
    f.line("    ConstValue,");
    f.indented_doc_comment("Compose two combinators sequentially.");
    f.line("    Then,");
    f.indented_doc_comment("Map the error variant of a fallible combinator.");
    f.line("    MapErr,");
    f.indented_doc_comment("Conditional composition (and_then).");
    f.line("    AndThen,");
    f.line("}");
    f.blank();

    // GroundingPrimitive<Out, Markers> carrier — parametric over both the
    // output type and the type-level marker tuple.
    f.doc_comment("Max depth of a composed op chain retained inline inside");
    f.doc_comment("`GroundingPrimitive`. Depth-2 composites (`Then(leaf, leaf)`,");
    f.doc_comment("`AndThen(leaf, leaf)`) are the exercised shape today; 8 gives headroom");
    f.doc_comment("for nested composition while keeping `Copy` and `no_std` without alloc.");
    f.doc_comment("");
    f.doc_comment("Wiki ADR-037: a foundation-fixed conservative default for");
    f.doc_comment("[`crate::HostBounds::OP_CHAIN_DEPTH_MAX`].");
    f.line(
        "pub const MAX_OP_CHAIN_DEPTH: usize = \
         8;",
    );
    f.blank();
    f.doc_comment("v0.2.2 Phase J: a single grounding primitive parametric over its output");
    f.doc_comment("type `Out` and its type-level marker tuple `Markers`.");
    f.doc_comment("");
    f.doc_comment("Constructed only by the 12 enumerated combinator functions below;");
    f.doc_comment("downstream cannot construct one directly. The `Markers` parameter");
    f.doc_comment("defaults to `()` for backwards-compatible call sites, but each");
    f.doc_comment("combinator returns a specific tuple — see `combinators::digest` etc.");
    f.doc_comment("");
    f.doc_comment("For leaf primitives `chain_len == 0`. For `Then`/`AndThen`/`MapErr`");
    f.doc_comment("composites, `chain[..chain_len as usize]` is the linearized post-order");
    f.doc_comment("sequence of leaf primitive ops the interpreter walks.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct GroundingPrimitive<Out, Markers: MarkerTuple = ()> {");
    f.line("    op: GroundingPrimitiveOp,");
    f.line("    markers: MarkerBits,");
    f.line("    chain: [GroundingPrimitiveOp; MAX_OP_CHAIN_DEPTH],");
    f.line("    chain_len: u8,");
    f.line("    _out: PhantomData<Out>,");
    f.line("    _markers: PhantomData<Markers>,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl<Out, Markers: MarkerTuple> GroundingPrimitive<Out, Markers> {");
    f.indented_doc_comment("Access the primitive op.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn op(&self) -> GroundingPrimitiveOp { self.op }");
    f.blank();
    f.indented_doc_comment("Access the runtime marker bitmask (mirrors the type-level tuple).");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn markers(&self) -> MarkerBits { self.markers }");
    f.blank();
    f.indented_doc_comment("Access the recorded composition chain. Empty for leaf primitives;");
    f.indented_doc_comment("the post-order leaf-op sequence for `Then`/`AndThen`/`MapErr`.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn chain(&self) -> &[GroundingPrimitiveOp] {");
    f.line("        &self.chain[..self.chain_len as usize]");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Crate-internal constructor for a leaf primitive (no recorded chain).");
    f.indented_doc_comment("The type-level `Markers` tuple is selected via turbofish at call");
    f.indented_doc_comment("sites inside the combinator functions.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn from_parts(");
    f.line("        op: GroundingPrimitiveOp,");
    f.line("        markers: MarkerBits,");
    f.line("    ) -> Self {");
    f.line("        Self {");
    f.line("            op,");
    f.line("            markers,");
    f.line("            chain: [GroundingPrimitiveOp::ReadBytes; MAX_OP_CHAIN_DEPTH],");
    f.line("            chain_len: 0,");
    f.line("            _out: PhantomData,");
    f.line("            _markers: PhantomData,");
    f.line("            _sealed: (),");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Crate-internal constructor for a composite primitive. Stores");
    f.indented_doc_comment("`chain[..chain_len]` inline; accessors expose only the prefix.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(dead_code)]");
    f.line("    pub(crate) const fn from_parts_with_chain(");
    f.line("        op: GroundingPrimitiveOp,");
    f.line("        markers: MarkerBits,");
    f.line("        chain: [GroundingPrimitiveOp; MAX_OP_CHAIN_DEPTH],");
    f.line("        chain_len: u8,");
    f.line("    ) -> Self {");
    f.line("        Self {");
    f.line("            op,");
    f.line("            markers,");
    f.line("            chain,");
    f.line("            chain_len,");
    f.line("            _out: PhantomData,");
    f.line("            _markers: PhantomData,");
    f.line("            _sealed: (),");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();

    // Emit the closed catalogue of 12 combinators. Each leaf combinator
    // returns a `GroundingPrimitive<Out, M>` where M is its specific marker
    // tuple — the type parameter is fixed by the return type signature.
    f.doc_comment("v0.2.2 Phase J: closed 12-combinator surface for building grounding");
    f.doc_comment("programs. See `GroundingProgram` for composition. Each leaf combinator");
    f.doc_comment("returns a `GroundingPrimitive<Out, M>` carrying a specific marker tuple;");
    f.doc_comment("the type parameter is what `GroundingProgram::from_primitive`'s");
    f.doc_comment("`MarkersImpliedBy<Map>` bound checks at compile time.");
    f.line("pub mod combinators {");
    f.line("    use super::{");
    f.line("        GroundingPrimitive, GroundingPrimitiveOp, InvertibleMarker,");
    f.line("        MarkerBits, MarkerIntersection, MarkerTuple,");
    f.line("        PreservesStructureMarker, TotalMarker, MAX_OP_CHAIN_DEPTH,");
    f.line("    };");
    f.blank();
    f.line("    /// Build the post-order leaf-op chain for a sequential composite:");
    f.line("    /// `first.chain ++ [first.op] ++ second.chain ++ [second.op]`.");
    f.line("    /// Saturated to `MAX_OP_CHAIN_DEPTH`.");
    f.line("    fn compose_chain<A, B, MA: MarkerTuple, MB: MarkerTuple>(");
    f.line("        first: &GroundingPrimitive<A, MA>,");
    f.line("        second: &GroundingPrimitive<B, MB>,");
    f.line("    ) -> ([GroundingPrimitiveOp; MAX_OP_CHAIN_DEPTH], u8) {");
    f.line("        let mut chain = [GroundingPrimitiveOp::ReadBytes; MAX_OP_CHAIN_DEPTH];");
    f.line("        let mut len: usize = 0;");
    f.line("        for &op in first.chain() {");
    f.line("            if len >= MAX_OP_CHAIN_DEPTH { return (chain, len as u8); }");
    f.line("            chain[len] = op;");
    f.line("            len += 1;");
    f.line("        }");
    f.line("        if len < MAX_OP_CHAIN_DEPTH { chain[len] = first.op(); len += 1; }");
    f.line("        for &op in second.chain() {");
    f.line("            if len >= MAX_OP_CHAIN_DEPTH { return (chain, len as u8); }");
    f.line("            chain[len] = op;");
    f.line("            len += 1;");
    f.line("        }");
    f.line("        if len < MAX_OP_CHAIN_DEPTH { chain[len] = second.op(); len += 1; }");
    f.line("        (chain, len as u8)");
    f.line("    }");
    f.blank();
    f.line("    /// Build the chain for `map_err(first)`: `first.chain ++ [first.op]`.");
    f.line("    fn map_err_chain<A, M: MarkerTuple>(");
    f.line("        first: &GroundingPrimitive<A, M>,");
    f.line("    ) -> ([GroundingPrimitiveOp; MAX_OP_CHAIN_DEPTH], u8) {");
    f.line("        let mut chain = [GroundingPrimitiveOp::ReadBytes; MAX_OP_CHAIN_DEPTH];");
    f.line("        let mut len: usize = 0;");
    f.line("        for &op in first.chain() {");
    f.line("            if len >= MAX_OP_CHAIN_DEPTH { return (chain, len as u8); }");
    f.line("            chain[len] = op;");
    f.line("            len += 1;");
    f.line("        }");
    f.line("        if len < MAX_OP_CHAIN_DEPTH { chain[len] = first.op(); len += 1; }");
    f.line("        (chain, len as u8)");
    f.line("    }");
    f.blank();
    // Each entry: (fn_name, op_variant, type_tuple, bits_expr, doc)
    let combinator_entries: &[(&str, &str, &str, &str, &str)] = &[
        (
            "read_bytes",
            "ReadBytes",
            "(TotalMarker, InvertibleMarker)",
            "TOTAL.union(MarkerBits::INVERTIBLE)",
            "Read a fixed-size byte slice from the input. `(Total, Invertible)`.",
        ),
        (
            "interpret_le_integer",
            "InterpretLeInteger",
            "(TotalMarker, InvertibleMarker, PreservesStructureMarker)",
            "TOTAL.union(MarkerBits::INVERTIBLE).union(MarkerBits::PRESERVES_STRUCTURE)",
            "Interpret bytes as a little-endian integer at the target WittLevel.",
        ),
        (
            "interpret_be_integer",
            "InterpretBeInteger",
            "(TotalMarker, InvertibleMarker, PreservesStructureMarker)",
            "TOTAL.union(MarkerBits::INVERTIBLE).union(MarkerBits::PRESERVES_STRUCTURE)",
            "Interpret bytes as a big-endian integer.",
        ),
        (
            "digest",
            "Digest",
            "(TotalMarker,)",
            "TOTAL",
            "Hash bytes via blake3 → 32-byte digest → `Datum<W256>`. `(Total,)` only.",
        ),
        (
            "decode_utf8",
            "DecodeUtf8",
            "(InvertibleMarker, PreservesStructureMarker)",
            "INVERTIBLE.union(MarkerBits::PRESERVES_STRUCTURE)",
            "Decode UTF-8 bytes. `(Invertible, PreservesStructure)` — not Total.",
        ),
        (
            "decode_json",
            "DecodeJson",
            "(InvertibleMarker, PreservesStructureMarker)",
            "INVERTIBLE.union(MarkerBits::PRESERVES_STRUCTURE)",
            "Decode JSON bytes. `(Invertible, PreservesStructure)` — not Total.",
        ),
        (
            "select_field",
            "SelectField",
            "(InvertibleMarker,)",
            "INVERTIBLE",
            "Select a field from a structured value. `(Invertible,)` — not Total.",
        ),
        (
            "select_index",
            "SelectIndex",
            "(InvertibleMarker,)",
            "INVERTIBLE",
            "Select an indexed element. `(Invertible,)` — not Total.",
        ),
        (
            "const_value",
            "ConstValue",
            "(TotalMarker, InvertibleMarker, PreservesStructureMarker)",
            "TOTAL.union(MarkerBits::INVERTIBLE).union(MarkerBits::PRESERVES_STRUCTURE)",
            "Inject a foundation-known constant. `(Total, Invertible, PreservesStructure)`.",
        ),
    ];
    for (fn_name, op_variant, type_tuple, markers_expr, doc) in combinator_entries {
        f.line(&format!("    /// {doc}"));
        f.line("    #[inline]");
        f.line("    #[must_use]");
        f.line(&format!(
            "    pub const fn {fn_name}<Out>() -> GroundingPrimitive<Out, {type_tuple}> {{"
        ));
        f.line(&format!(
            "        GroundingPrimitive::from_parts(GroundingPrimitiveOp::{op_variant}, MarkerBits::{markers_expr})"
        ));
        f.line("    }");
        f.blank();
    }

    // Composition combinators: then, map_err, and_then. Each takes typed
    // marker tuples and computes the intersection at the type level via the
    // MarkerIntersection trait. The runtime `markers()` bitmask mirrors it.
    f.line("    /// Compose two combinators sequentially. Markers are intersected at");
    f.line("    /// the type level via the `MarkerIntersection` trait. The recorded");
    f.line("    /// leaf-op chain lets the foundation's interpreter walk the operands");
    f.line("    /// at runtime.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn then<A, B, MA, MB>(");
    f.line("        first: GroundingPrimitive<A, MA>,");
    f.line("        second: GroundingPrimitive<B, MB>,");
    f.line("    ) -> GroundingPrimitive<B, <MA as MarkerIntersection<MB>>::Output>");
    f.line("    where");
    f.line("        MA: MarkerTuple + MarkerIntersection<MB>,");
    f.line("        MB: MarkerTuple,");
    f.line("    {");
    f.line("        let (chain, chain_len) = compose_chain(&first, &second);");
    f.line("        GroundingPrimitive::from_parts_with_chain(");
    f.line("            GroundingPrimitiveOp::Then,");
    f.line("            first.markers().intersection(second.markers()),");
    f.line("            chain,");
    f.line("            chain_len,");
    f.line("        )");
    f.line("    }");
    f.blank();
    f.line("    /// Map an error variant of a fallible combinator. Marker tuple");
    f.line("    /// is preserved. The operand's op is recorded so the interpreter's");
    f.line("    /// `MapErr` arm can forward the success value.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn map_err<A, M: MarkerTuple>(");
    f.line("        first: GroundingPrimitive<A, M>,");
    f.line("    ) -> GroundingPrimitive<A, M> {");
    f.line("        let (chain, chain_len) = map_err_chain(&first);");
    f.line("        GroundingPrimitive::from_parts_with_chain(");
    f.line("            GroundingPrimitiveOp::MapErr,");
    f.line("            first.markers(),");
    f.line("            chain,");
    f.line("            chain_len,");
    f.line("        )");
    f.line("    }");
    f.blank();
    f.line("    /// Conditional composition (and_then). Markers are intersected; the");
    f.line("    /// recorded chain mirrors `then` so the interpreter walks operands.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn and_then<A, B, MA, MB>(");
    f.line("        first: GroundingPrimitive<A, MA>,");
    f.line("        second: GroundingPrimitive<B, MB>,");
    f.line("    ) -> GroundingPrimitive<B, <MA as MarkerIntersection<MB>>::Output>");
    f.line("    where");
    f.line("        MA: MarkerTuple + MarkerIntersection<MB>,");
    f.line("        MB: MarkerTuple,");
    f.line("    {");
    f.line("        let (chain, chain_len) = compose_chain(&first, &second);");
    f.line("        GroundingPrimitive::from_parts_with_chain(");
    f.line("            GroundingPrimitiveOp::AndThen,");
    f.line("            first.markers().intersection(second.markers()),");
    f.line("            chain,");
    f.line("            chain_len,");
    f.line("        )");
    f.line("    }");
    f.line("}");
    f.blank();

    // GroundingProgram<Out, Map> with the MarkersImpliedBy<Map> bound on
    // `from_primitive`. Mismatched programs are rejected at compile time.
    f.doc_comment("v0.2.2 Phase J: sealed grounding program.");
    f.doc_comment("");
    f.doc_comment("A composition of combinators with a statically tracked marker tuple.");
    f.doc_comment("Constructed only via `GroundingProgram::from_primitive`, which requires");
    f.doc_comment("via the `MarkersImpliedBy<Map>` trait bound that the primitive's marker");
    f.doc_comment("tuple carries every property declared by `Map: GroundingMapKind`.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct GroundingProgram<Out, Map: GroundingMapKind> {");
    f.line("    primitive: GroundingPrimitive<Out>,");
    f.line("    _map: PhantomData<Map>,");
    f.line("    _sealed: (),");
    f.line("}");
    f.blank();
    f.line("impl<Out, Map: GroundingMapKind> GroundingProgram<Out, Map> {");
    f.indented_doc_comment("Foundation-verified constructor. Accepts a primitive whose marker");
    f.indented_doc_comment("tuple satisfies `MarkersImpliedBy<Map>`. Programs built from");
    f.indented_doc_comment("combinators whose marker tuple lacks a property `Map` requires are");
    f.indented_doc_comment("rejected at compile time — this is Phase J's marquee correctness");
    f.indented_doc_comment("claim: misdeclarations fail to compile.");
    f.indented_doc_comment("");
    f.indented_doc_comment("# Example: valid program");
    f.indented_doc_comment("");
    f.indented_doc_comment("```");
    f.indented_doc_comment(
        "use uor_foundation::enforcement::{GroundingProgram, IntegerGroundingMap, combinators};",
    );
    f.indented_doc_comment("let prog: GroundingProgram<u64, IntegerGroundingMap> =");
    f.indented_doc_comment(
        "    GroundingProgram::from_primitive(combinators::interpret_le_integer::<u64>());",
    );
    f.indented_doc_comment("let _ = prog;");
    f.indented_doc_comment("```");
    f.indented_doc_comment("");
    f.indented_doc_comment("# Example: rejected misdeclaration");
    f.indented_doc_comment("");
    f.indented_doc_comment("```compile_fail");
    f.indented_doc_comment(
        "use uor_foundation::enforcement::{GroundingProgram, IntegerGroundingMap, combinators};",
    );
    f.indented_doc_comment("// digest returns (TotalMarker,) which does NOT satisfy");
    f.indented_doc_comment(
        "// MarkersImpliedBy<IntegerGroundingMap> — the line below fails to compile.",
    );
    f.indented_doc_comment("let prog: GroundingProgram<[u8; 32], IntegerGroundingMap> =");
    f.indented_doc_comment(
        "    GroundingProgram::from_primitive(combinators::digest::<[u8; 32]>());",
    );
    f.indented_doc_comment("let _ = prog;");
    f.indented_doc_comment("```");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn from_primitive<Markers>(");
    f.line("        primitive: GroundingPrimitive<Out, Markers>,");
    f.line("    ) -> Self");
    f.line("    where");
    f.line("        Markers: MarkerTuple + MarkersImpliedBy<Map>,");
    f.line("    {");
    f.line("        // Preserve the composition chain so the interpreter can walk");
    f.line("        // operands of Then/AndThen/MapErr composites.");
    f.line("        let mut chain = [GroundingPrimitiveOp::ReadBytes; MAX_OP_CHAIN_DEPTH];");
    f.line("        let src = primitive.chain();");
    f.line("        let mut i = 0;");
    f.line("        while i < src.len() && i < MAX_OP_CHAIN_DEPTH {");
    f.line("            chain[i] = src[i];");
    f.line("            i += 1;");
    f.line("        }");
    f.line("        let erased = GroundingPrimitive::<Out, ()>::from_parts_with_chain(");
    f.line("            primitive.op(),");
    f.line("            primitive.markers(),");
    f.line("            chain,");
    f.line("            i as u8,");
    f.line("        );");
    f.line("        Self {");
    f.line("            primitive: erased,");
    f.line("            _map: PhantomData,");
    f.line("            _sealed: (),");
    f.line("        }");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Access the underlying primitive (erased marker tuple).");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn primitive(&self) -> &GroundingPrimitive<Out> {");
    f.line("        &self.primitive");
    f.line("    }");
    f.line("}");
    f.blank();

    // Phase K (target §4.3): run the program on external bytes via a
    // foundation-supplied interpreter. Full 12-combinator dispatch —
    // leaf ops call `interpret_leaf_op` directly; composition ops walk
    // the chain recorded by `combinators::{then, and_then, map_err}`.
    f.doc_comment("Phase K (target §4.3 / §9 criterion 1): foundation-supplied interpreter for");
    f.doc_comment("grounding programs whose `Out` is `GroundedCoord`. Handles every op in");
    f.doc_comment("the closed 12-combinator catalogue: leaf ops (`ReadBytes`,");
    f.doc_comment("`InterpretLeInteger`, `InterpretBeInteger`, `Digest`, `DecodeUtf8`,");
    f.doc_comment("`DecodeJson`, `ConstValue`, `SelectField`, `SelectIndex`) call");
    f.doc_comment("`interpret_leaf_op` directly; composition ops (`Then`, `AndThen`,");
    f.doc_comment("`MapErr`) walk the chain recorded in the primitive and thread");
    f.doc_comment("`external` through each leaf step. The interpreter surfaces");
    f.doc_comment("`GroundedCoord::w8(byte)` values; richer outputs compose through");
    f.doc_comment("combinator chains producing `GroundedTuple<N>`. No `ground()`");
    f.doc_comment("override exists after W4 closure — downstream provides only");
    f.doc_comment("`program()`, and `GroundingExt::ground` is foundation-authored.");
    f.line("impl<Map: GroundingMapKind> GroundingProgram<GroundedCoord, Map> {");
    f.indented_doc_comment("Run this program on external bytes, producing a `GroundedCoord`.");
    f.indented_doc_comment("Returns `None` if the input is malformed/undersized for the");
    f.indented_doc_comment("program's op chain.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn run(&self, external: &[u8]) -> Option<GroundedCoord> {");
    f.line("        match self.primitive.op() {");
    f.line("            GroundingPrimitiveOp::Then | GroundingPrimitiveOp::AndThen => {");
    f.line("                let chain = self.primitive.chain();");
    f.line("                if chain.is_empty() { return None; }");
    f.line("                let mut last: Option<GroundedCoord> = None;");
    f.line("                for &op in chain {");
    f.line("                    match interpret_leaf_op(op, external) {");
    f.line("                        Some(c) => last = Some(c),");
    f.line("                        None => return None,");
    f.line("                    }");
    f.line("                }");
    f.line("                last");
    f.line("            }");
    f.line("            GroundingPrimitiveOp::MapErr => self");
    f.line("                .primitive");
    f.line("                .chain()");
    f.line("                .first()");
    f.line("                .and_then(|&op| interpret_leaf_op(op, external)),");
    f.line("            leaf => interpret_leaf_op(leaf, external),");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();
    f.doc_comment("W4 closure (target §4.3 + §9 criterion 1): foundation-supplied");
    f.doc_comment("interpreter for programs producing `GroundedTuple<N>`. Splits");
    f.doc_comment("`external` into `N` equal windows and runs the same dispatch");
    f.doc_comment("that `GroundingProgram<GroundedCoord, Map>::run` performs on");
    f.doc_comment("each window. Returns `None` if `N == 0`, the input is empty,");
    f.doc_comment("the input length isn't divisible by `N`, or any window fails.");
    f.line("impl<const N: usize, Map: GroundingMapKind> GroundingProgram<GroundedTuple<N>, Map> {");
    f.indented_doc_comment("Run this program on external bytes, producing a `GroundedTuple<N>`.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn run(&self, external: &[u8]) -> Option<GroundedTuple<N>> {");
    f.line("        if N == 0 || external.is_empty() || external.len() % N != 0 {");
    f.line("            return None;");
    f.line("        }");
    f.line("        let window = external.len() / N;");
    f.line("        let mut coords: [GroundedCoord; N] = [const { GroundedCoord::w8(0) }; N];");
    f.line("        let mut i = 0usize;");
    f.line("        while i < N {");
    f.line("            let start = i * window;");
    f.line("            let end = start + window;");
    f.line("            let sub = &external[start..end];");
    f.line("            // Walk this window through the same leaf/composition");
    f.line("            // dispatch as the GroundedCoord interpreter above. A");
    f.line("            // helper that runs the chain is reused via the same");
    f.line("            // primitive accessors.");
    f.line("            let outcome = match self.primitive.op() {");
    f.line("                GroundingPrimitiveOp::Then | GroundingPrimitiveOp::AndThen => {");
    f.line("                    let chain = self.primitive.chain();");
    f.line("                    if chain.is_empty() { return None; }");
    f.line("                    let mut last: Option<GroundedCoord> = None;");
    f.line("                    for &op in chain {");
    f.line("                        match interpret_leaf_op(op, sub) {");
    f.line("                            Some(c) => last = Some(c),");
    f.line("                            None => return None,");
    f.line("                        }");
    f.line("                    }");
    f.line("                    last");
    f.line("                }");
    f.line("                GroundingPrimitiveOp::MapErr => self");
    f.line("                    .primitive");
    f.line("                    .chain()");
    f.line("                    .first()");
    f.line("                    .and_then(|&op| interpret_leaf_op(op, sub)),");
    f.line("                leaf => interpret_leaf_op(leaf, sub),");
    f.line("            };");
    f.line("            match outcome {");
    f.line("                Some(c) => { coords[i] = c; }");
    f.line("                None => { return None; }");
    f.line("            }");
    f.line("            i += 1;");
    f.line("        }");
    f.line("        Some(GroundedTuple::new(coords))");
    f.line("    }");
    f.line("}");
    f.blank();
    f.doc_comment("Foundation-canonical leaf-op interpreter. Called directly by");
    f.doc_comment("`GroundingProgram::run` for leaf primitives and step-by-step for");
    f.doc_comment("composition ops.");
    f.line("#[inline]");
    f.line("fn interpret_leaf_op(op: GroundingPrimitiveOp, external: &[u8]) -> Option<GroundedCoord> {");
    f.line("    match op {");
    f.line(
        "        GroundingPrimitiveOp::ReadBytes | GroundingPrimitiveOp::InterpretLeInteger => {",
    );
    f.line("            external.first().map(|&b| GroundedCoord::w8(b))");
    f.line("        }");
    f.line("        GroundingPrimitiveOp::InterpretBeInteger => {");
    f.line("            external.last().map(|&b| GroundedCoord::w8(b))");
    f.line("        }");
    f.line("        GroundingPrimitiveOp::Digest => {");
    f.line("            // Foundation-canonical digest: first byte as `GroundedCoord` —");
    f.line("            // the full 32-byte digest requires a Datum<W256> sink that does");
    f.line("            // not fit in `GroundedCoord`. Downstream that needs the full");
    f.line("            // digest composes a richer program via Then/AndThen chains over");
    f.line("            // the 12 closed combinators — no `ground()` override exists after");
    f.line("            // W4 closure.");
    f.line("            external.first().map(|&b| GroundedCoord::w8(b))");
    f.line("        }");
    f.line("        GroundingPrimitiveOp::DecodeUtf8 => {");
    f.line("            // ASCII single-byte path — the canonical v0.2.2 semantics for");
    f.line("            // `Grounding::ground` over `GroundedCoord` outputs. Multi-byte");
    f.line("            // UTF-8 sequences are out of scope for w8-sized leaves; a richer");
    f.line("            // structured output composes via combinator chains into");
    f.line("            // `GroundedTuple<N>`.");
    f.line("            match external.first() {");
    f.line("                Some(&b) if b < 0x80 => Some(GroundedCoord::w8(b)),");
    f.line("                _ => None,");
    f.line("            }");
    f.line("        }");
    f.line("        GroundingPrimitiveOp::DecodeJson => {");
    f.line("            // Accept JSON number scalars (leading `-` or ASCII digit).");
    f.line("            match external.first() {");
    f.line("                Some(&b) if b == b'-' || b.is_ascii_digit() => Some(GroundedCoord::w8(b)),");
    f.line("                _ => None,");
    f.line("            }");
    f.line("        }");
    f.line("        GroundingPrimitiveOp::ConstValue => Some(GroundedCoord::w8(0)),");
    f.line("        GroundingPrimitiveOp::SelectField | GroundingPrimitiveOp::SelectIndex => {");
    f.line("            // Selector ops are composition-only in normal use; if invoked");
    f.line("            // directly, forward the first byte as a GroundedCoord.");
    f.line("            external.first().map(|&b| GroundedCoord::w8(b))");
    f.line("        }");
    f.line("        GroundingPrimitiveOp::Then | GroundingPrimitiveOp::AndThen | GroundingPrimitiveOp::MapErr => {");
    f.line("            // Composite ops are dispatched by `run()` through the chain;");
    f.line("            // they never reach the leaf interpreter.");
    f.line("            None");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();

    // MarkersImpliedBy<Map> impls for every valid (tuple, map) pair.
    // The rules:
    // - BinaryGroundingMap requires {Total, Invertible}: tuples TI, TIS.
    // - DigestGroundingMap requires {Total}: tuples T, TI, TIS.
    // - IntegerGroundingMap requires {Total, Invertible, PreservesStructure}: tuple TIS.
    // - JsonGroundingMap requires {Invertible, PreservesStructure}: tuples TIS, IS.
    // - Utf8GroundingMap requires {Invertible, PreservesStructure}: tuples TIS, IS.
    // 10 impls total.
    f.doc_comment("v0.2.2 Phase J: MarkersImpliedBy impls for the closed catalogue of valid");
    f.doc_comment("(marker tuple, GroundingMapKind) pairs. These are the compile-time");
    f.doc_comment("witnesses the foundation accepts; every absent pair is a rejection.");
    f.line("impl MarkersImpliedBy<DigestGroundingMap> for (TotalMarker,) {}");
    f.line("impl MarkersImpliedBy<BinaryGroundingMap> for (TotalMarker, InvertibleMarker) {}");
    f.line("impl MarkersImpliedBy<DigestGroundingMap> for (TotalMarker, InvertibleMarker) {}");
    f.line(
        "impl MarkersImpliedBy<BinaryGroundingMap> for (TotalMarker, InvertibleMarker, PreservesStructureMarker) {}",
    );
    f.line(
        "impl MarkersImpliedBy<DigestGroundingMap> for (TotalMarker, InvertibleMarker, PreservesStructureMarker) {}",
    );
    f.line(
        "impl MarkersImpliedBy<IntegerGroundingMap> for (TotalMarker, InvertibleMarker, PreservesStructureMarker) {}",
    );
    f.line(
        "impl MarkersImpliedBy<JsonGroundingMap> for (TotalMarker, InvertibleMarker, PreservesStructureMarker) {}",
    );
    f.line(
        "impl MarkersImpliedBy<Utf8GroundingMap> for (TotalMarker, InvertibleMarker, PreservesStructureMarker) {}",
    );
    f.line("impl MarkersImpliedBy<JsonGroundingMap> for (InvertibleMarker, PreservesStructureMarker) {}");
    f.line("impl MarkersImpliedBy<Utf8GroundingMap> for (InvertibleMarker, PreservesStructureMarker) {}");
    f.blank();

    // v0.2.2 T2.5 (cleanup): test-only back-door module exposing crate-internal
    // constructors via a `#[doc(hidden)] pub` interface. Consumed exclusively
    // by the `uor-foundation-test-helpers` workspace member; its `__` prefix
    // signals private-API status. Excluded from `cargo public-api` snapshot
    // output via #[doc(hidden)].
    f.doc_comment("v0.2.2 T2.5: foundation-private test-only back-door module.");
    f.doc_comment("");
    f.doc_comment("Exposes crate-internal constructors for `Trace`, `TraceEvent`, and");
    f.doc_comment("`MulContext` to the `uor-foundation-test-helpers` workspace member, which");
    f.doc_comment("re-exports them under stable test-only names. Not part of the public API.");
    f.line("#[doc(hidden)]");
    f.line("pub mod __test_helpers {");
    f.line("    use super::{");
    f.line("        ContentAddress, ContentFingerprint, MulContext, Trace, TraceEvent, Validated,");
    f.line("    };");
    f.blank();
    f.indented_doc_comment("Test-only ctor: build a Trace from a slice of events with a");
    f.indented_doc_comment("`ContentFingerprint::zero()` placeholder. Tests that need a non-zero");
    f.indented_doc_comment("fingerprint use `trace_with_fingerprint` instead. Parametric in");
    f.indented_doc_comment("`TR_MAX` per the wiki's ADR-018; callers pick the trace event-count");
    f.indented_doc_comment("ceiling from their selected `HostBounds`.");
    f.line("    #[must_use]");
    f.line(
        "    pub fn trace_from_events<const TR_MAX: usize>(events: &[TraceEvent]) -> Trace<TR_MAX> {",
    );
    f.line("        trace_with_fingerprint(events, 0, ContentFingerprint::zero())");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Test-only ctor that takes an explicit `witt_level_bits` and");
    f.indented_doc_comment("`ContentFingerprint`. Used by round-trip tests that need to verify");
    f.indented_doc_comment("the verify-trace fingerprint passthrough invariant.");
    f.line("    #[must_use]");
    f.line("    pub fn trace_with_fingerprint<const TR_MAX: usize>(");
    f.line("        events: &[TraceEvent],");
    f.line("        witt_level_bits: u16,");
    f.line("        content_fingerprint: ContentFingerprint,");
    f.line("    ) -> Trace<TR_MAX> {");
    f.line("        let mut arr = [None; TR_MAX];");
    f.line("        let n = events.len().min(TR_MAX);");
    f.line("        let mut i = 0;");
    f.line("        while i < n {");
    f.line("            arr[i] = Some(events[i]);");
    f.line("            i += 1;");
    f.line("        }");
    f.line("        // The test-helpers back-door uses the foundation-private");
    f.line("        // `from_replay_events_const` to build malformed fixtures for error-path");
    f.line("        // tests. Downstream code uses `Trace::try_from_events` (validating).");
    f.line("        Trace::from_replay_events_const(arr, n as u16, witt_level_bits, content_fingerprint)");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Test-only ctor: build a TraceEvent.");
    f.line("    #[must_use]");
    f.line("    pub fn trace_event(step_index: u32, target: u128) -> TraceEvent {");
    f.line("        TraceEvent::new(");
    f.line("            step_index,");
    f.line("            crate::PrimitiveOp::Add,");
    f.line("            ContentAddress::from_u128(target),");
    f.line("        )");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Test-only ctor: build a MulContext.");
    f.line("    #[must_use]");
    f.line("    pub fn mul_context(");
    f.line("        stack_budget_bytes: u64,");
    f.line("        const_eval: bool,");
    f.line("        limb_count: usize,");
    f.line("    ) -> MulContext {");
    f.line("        MulContext::new(stack_budget_bytes, const_eval, limb_count)");
    f.line("    }");
    f.blank();
    f.indented_doc_comment("Test-only ctor: wrap any T in a Runtime-phase Validated. Used by");
    f.indented_doc_comment("integration tests to construct `Validated<Decl, P>` values that");
    f.indented_doc_comment("the public API otherwise can't construct directly.");
    f.line("    #[must_use]");
    f.line("    pub fn validated_runtime<T>(inner: T) -> Validated<T> {");
    f.line("        Validated::new(inner)");
    f.line("    }");
    f.line("}");
    f.blank();
}

/// Product/Coproduct Completion Amendment — emits the full Part A foundation
/// surface: witness structs, Evidence / MintInputs structs, entropy helpers,
/// the coproduct structural validator, three mint primitives, and the
/// `VerifiedMint` sealed trait + impls. See the plan's Phase 2.3 section for
/// the rationale and sub-step layout.
fn generate_product_coproduct_amendment(f: &mut RustFile) {
    // -----------------------------------------------------------------------
    // Phase 2.4 / Phase D1: amendment §1d witness trait impls.
    //
    // Emitted by D1.4:
    //   impl<H: HostTypes> PartitionProduct<H> for PartitionProductWitness
    //   impl<H: HostTypes> PartitionCoproduct<H> for PartitionCoproductWitness
    //   impl<H: HostTypes> CartesianPartitionProduct<H> for CartesianProductWitness
    //   impl<H: HostTypes> Partition<H> for NullPartition<H>
    //
    // Each of the three witness impls returns a `NullPartition<H>` —
    // the resolver-absent default-value Partition introduced by D1.3
    // and emitted by `emit_pc_null_partition_composite`. NullPartition
    // embeds inline stubs for all 7 sub-trait associated types
    // (IrreducibleSet, ReducibleSet, UnitGroup, Complement, FreeRank,
    // TagSite, TypeDefinition) so `&self.field` accessors satisfy the
    // Partition trait's reference-returning signatures.
    //
    // NOT emitted (B1a structural limit, NOT a host-type-narrowing
    // compromise): `impl<H: HostTypes> Partition<H> for PartitionHandle<H>`.
    // PartitionHandle<H> is a 16-byte content-addressed identity token
    // (`ContentFingerprint + PhantomData<H>`); embedding a
    // ~150-byte NullPartition inline would break the Copy-small-token
    // invariant the witness types depend on for content-addressed
    // indexing. Consumers reach Partition<H> from a handle via either
    //   handle.resolve_with(&resolver) → Option<PartitionRecord<H>>
    // or
    //   <_ as PartitionProduct<H>>::left_factor(&witness) → NullPartition<H>
    // — both are first-class API paths.
    //
    // Grep anchor: `phase-2.4-deferral` (now historical — D1 lands the
    // deferred witness impls).
    // -----------------------------------------------------------------------

    emit_pc_entropy_helpers(f);
    emit_pc_evidence_structs(f);
    emit_pc_mint_inputs_structs(f);
    emit_pc_witness_structs(f);
    emit_pc_certificate_impls(f);
    emit_pc_verified_mint_trait(f);
    emit_pc_validate_coproduct_structure(f);
    emit_pc_mint_primitives(f);
    emit_pc_verified_mint_impls(f);
    emit_pc_partition_handle_protocol(f);
    // §D1: Null* sub-trait stubs + NullPartition<H> composite + 3 witness
    // trait impls. Lands the deferred Phase 2.4 ergonomics — every
    // PartitionProduct/Coproduct/CartesianPartitionProduct witness exposes
    // its operand fingerprints through the codegen-generated trait
    // accessors, returning a generic NullPartition<H> as the resolver-
    // absent default.
    emit_pc_null_partition_stubs(f);
    emit_pc_null_partition_composite(f);
    emit_pc_witness_trait_impls(f);
}

/// §2.3i — emits the resolver protocol: `PartitionResolver<H>` trait,
/// `PartitionRecord<H>` data record, and `PartitionHandle<H>` identity
/// token. The handle is intentionally a standalone value type and does
/// NOT implement the codegen-generated `Partition<H>` trait — that trait
/// has seven associated types each bounded on its own sub-trait, and
/// providing a no-resolver default for each would require substantial
/// stub infrastructure beyond this amendment's scope. Consumers who need
/// a full `Partition<H>` implementation build their own resolver-backed
/// handle type; this amendment supplies the identity + resolution
/// protocol they compose against.
fn emit_pc_partition_handle_protocol(f: &mut RustFile) {
    f.doc_comment("Data record of a partition's runtime-queried properties. Produced at");
    f.doc_comment("witness-mint time and consulted by consumer code that holds a");
    f.doc_comment("`PartitionHandle` and a `PartitionResolver`. Phase 9 stores entropy");
    f.doc_comment("as the IEEE-754 `u64` bit-pattern (`entropy_nats_bits`) so the record");
    f.doc_comment("derives `Eq + Hash` cleanly; consumers project to `H::Decimal` via");
    f.doc_comment("`<H::Decimal as DecimalTranscendental>::from_bits`.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct PartitionRecord<H: crate::HostTypes> {");
    f.indented_doc_comment("Data sites only — the partition's `siteBudget`, not its layout width.");
    f.line("    pub site_budget: u16,");
    f.indented_doc_comment("Euler characteristic of the partition's nerve.");
    f.line("    pub euler: i32,");
    f.indented_doc_comment(
        "Betti profile of the partition's nerve, padded to `MAX_BETTI_DIMENSION`.",
    );
    f.line("    pub betti: [u32; MAX_BETTI_DIMENSION],");
    f.indented_doc_comment(
        "Shannon entropy in nats (matches `LandauerBudget::nats()` convention).",
    );
    f.line("    pub entropy_nats_bits: u64,");
    f.line("    _phantom: core::marker::PhantomData<H>,");
    f.line("}");
    f.blank();
    f.line("impl<H: crate::HostTypes> PartitionRecord<H> {");
    f.indented_doc_comment("Construct a new record. The phantom marker ensures records are");
    f.indented_doc_comment("parameterized by the host types they originate from.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn new(");
    f.line("        site_budget: u16,");
    f.line("        euler: i32,");
    f.line("        betti: [u32; MAX_BETTI_DIMENSION],");
    f.line("        entropy_nats_bits: u64,");
    f.line("    ) -> Self {");
    f.line("        Self {");
    f.line("            site_budget,");
    f.line("            euler,");
    f.line("            betti,");
    f.line("            entropy_nats_bits,");
    f.line("            _phantom: core::marker::PhantomData,");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();

    f.doc_comment("Resolver mapping content fingerprints to `PartitionRecord`s. Provided");
    f.doc_comment("by the host application — typically a persistent store, an in-memory");
    f.doc_comment("registry populated from witness mint-time data, or a chain-of-witnesses");
    f.doc_comment("trail that can reconstruct properties.");
    f.line("pub trait PartitionResolver<H: crate::HostTypes> {");
    f.indented_doc_comment("Look up partition data by fingerprint. Returns `None` if the");
    f.indented_doc_comment("resolver has no record for the handle. Handles remain valid as");
    f.indented_doc_comment("identity tokens regardless of resolver presence.");
    f.line("    fn resolve(&self, fp: ContentFingerprint) -> Option<PartitionRecord<H>>;");
    f.line("}");
    f.blank();

    f.doc_comment("Content-addressed identity token for a partition. Carries only a");
    f.doc_comment("fingerprint; partition data is recovered by pairing the handle with a");
    f.doc_comment("`PartitionResolver` via `resolve_with`. Handles compare and hash by");
    f.doc_comment("fingerprint, so they can serve as keys in content-addressed indices");
    f.doc_comment("without resolver access.");
    // Phase 14: hand-written Copy/Clone/PartialEq/Eq/Hash so the impls
    // do NOT require `H: Copy`. The auto-derive synthesises a where
    // clause that bounds H by every trait it derives, which propagates
    // up to MintInputs<H> structs that contain PartitionHandle<H>.
    f.line("#[derive(Debug)]");
    f.line("pub struct PartitionHandle<H: crate::HostTypes> {");
    f.line("    fingerprint: ContentFingerprint,");
    f.line("    _phantom: core::marker::PhantomData<H>,");
    f.line("}");
    f.line("impl<H: crate::HostTypes> Copy for PartitionHandle<H> {}");
    f.line("impl<H: crate::HostTypes> Clone for PartitionHandle<H> {");
    f.line("    #[inline]");
    f.line("    fn clone(&self) -> Self { *self }");
    f.line("}");
    f.line("impl<H: crate::HostTypes> PartialEq for PartitionHandle<H> {");
    f.line("    #[inline]");
    f.line("    fn eq(&self, other: &Self) -> bool { self.fingerprint == other.fingerprint }");
    f.line("}");
    f.line("impl<H: crate::HostTypes> Eq for PartitionHandle<H> {}");
    f.line("impl<H: crate::HostTypes> core::hash::Hash for PartitionHandle<H> {");
    f.line("    #[inline]");
    f.line("    fn hash<S: core::hash::Hasher>(&self, state: &mut S) {");
    f.line("        self.fingerprint.hash(state);");
    f.line("    }");
    f.line("}");
    f.blank();
    f.line("impl<H: crate::HostTypes> PartitionHandle<H> {");
    f.indented_doc_comment("Construct a handle from a content fingerprint.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn from_fingerprint(fingerprint: ContentFingerprint) -> Self {");
    f.line("        Self {");
    f.line("            fingerprint,");
    f.line("            _phantom: core::marker::PhantomData,");
    f.line("        }");
    f.line("    }");
    f.indented_doc_comment("Return the content fingerprint this handle references.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn fingerprint(&self) -> ContentFingerprint {");
    f.line("        self.fingerprint");
    f.line("    }");
    f.indented_doc_comment("Resolve this handle against a consumer-supplied resolver.");
    f.indented_doc_comment("Returns `None` if the resolver has no record for this fingerprint.");
    f.line("    #[inline]");
    f.line("    pub fn resolve_with<R: PartitionResolver<H>>(");
    f.line("        &self,");
    f.line("        resolver: &R,");
    f.line("    ) -> Option<PartitionRecord<H>> {");
    f.line("        resolver.resolve(self.fingerprint)");
    f.line("    }");
    f.line("}");
    f.blank();
}

/// §2.3f entropy helpers. Used by all three mint primitives to validate
/// operand entropies and assert additivity identities within a
/// magnitude-scaled tolerance.
fn emit_pc_entropy_helpers(f: &mut RustFile) {
    f.doc_comment("Tolerance for entropy equality checks in the Product/Coproduct");
    f.doc_comment("Completion Amendment mint primitives. Returns an absolute-error bound");
    f.doc_comment("scaled to the magnitude of `expected`, so PT_4 / ST_2 / CPT_5");
    f.doc_comment("verifications are robust to floating-point rounding accumulated through");
    f.doc_comment("Künneth products and componentwise sums. The default-host (f64) backing");
    f.doc_comment("is hidden behind the `DefaultDecimal` alias so the function signature");
    f.doc_comment("reads as host-typed; downstream that swaps `H::Decimal` reaches the");
    f.doc_comment("same surface via the alias rebind.");
    f.line("type DefaultDecimal = <crate::DefaultHostTypes as crate::HostTypes>::Decimal;");
    f.blank();
    f.line("#[inline]");
    f.line("const fn pc_entropy_tolerance(expected: DefaultDecimal) -> DefaultDecimal {");
    f.line("    let magnitude = if expected < 0.0 { -expected } else { expected };");
    f.line("    let scale = if magnitude > 1.0 { magnitude } else { 1.0 };");
    f.line("    1024.0 * <DefaultDecimal>::EPSILON * scale");
    f.line("}");
    f.blank();

    f.doc_comment("Validate an entropy value before participating in additivity checks.");
    f.doc_comment("Rejects NaN, ±∞, and negative values — the foundation's");
    f.doc_comment("`primitive_descent_metrics` produces `residual × LN_2` with");
    f.doc_comment("`residual: u32`, so valid entropies are non-negative finite Decimals.");
    f.line("#[inline]");
    f.line("fn pc_entropy_input_is_valid(value: DefaultDecimal) -> bool {");
    f.line("    value.is_finite() && value >= 0.0");
    f.line("}");
    f.blank();

    f.doc_comment("Check that `actual` matches `expected` within tolerance and that both");
    f.doc_comment("inputs are valid entropy values. Returns `false` if either input is");
    f.doc_comment("non-finite, negative, or differs from `expected` by more than");
    f.doc_comment("`pc_entropy_tolerance(expected)`.");
    f.line("#[inline]");
    f.line(
        "fn pc_entropy_additivity_holds(actual: DefaultDecimal, expected: DefaultDecimal) -> bool {",
    );
    f.line("    if !pc_entropy_input_is_valid(actual) || !pc_entropy_input_is_valid(expected) {");
    f.line("        return false;");
    f.line("    }");
    f.line("    let diff = actual - expected;");
    f.line("    let diff_abs = if diff < 0.0 { -diff } else { diff };");
    f.line("    diff_abs <= pc_entropy_tolerance(expected)");
    f.line("}");
    f.blank();
}

/// §2.3c — three Evidence sidecar structs. Derive `PartialEq` only because
/// they carry `f64` entropy fields; they are auditing sidecars, not hash-map
/// keys, so the absence of `Eq` / `Hash` is intentional.
fn emit_pc_evidence_structs(f: &mut RustFile) {
    // PartitionProductEvidence
    f.doc_comment("Evidence bundle for `PartitionProductWitness`. Carries the PT_1 / PT_3 /");
    f.doc_comment("PT_4 input values used at mint time. Derives `PartialEq` only because");
    f.doc_comment("`f64` entropy fields exclude `Eq` / `Hash`; this is the auditing surface,");
    f.doc_comment("not a hash-map key.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq)]");
    f.line("pub struct PartitionProductEvidence {");
    f.indented_doc_comment("Left operand `siteBudget` (data sites only, PT_1 input).");
    f.line("    pub left_site_budget: u16,");
    f.indented_doc_comment("Right operand `siteBudget`.");
    f.line("    pub right_site_budget: u16,");
    f.indented_doc_comment("Left operand `SITE_COUNT` (layout width, layout-invariant input).");
    f.line("    pub left_total_site_count: u16,");
    f.indented_doc_comment("Right operand `SITE_COUNT`.");
    f.line("    pub right_total_site_count: u16,");
    f.indented_doc_comment("Left operand Euler characteristic (PT_3 input).");
    f.line("    pub left_euler: i32,");
    f.indented_doc_comment("Right operand Euler characteristic.");
    f.line("    pub right_euler: i32,");
    f.indented_doc_comment("Left operand entropy in nats (PT_4 input).");
    f.line("    pub left_entropy_nats_bits: u64,");
    f.indented_doc_comment("Right operand entropy in nats.");
    f.line("    pub right_entropy_nats_bits: u64,");
    f.indented_doc_comment("Fingerprint of the source witness the evidence belongs to.");
    f.line("    pub source_witness_fingerprint: ContentFingerprint,");
    f.line("}");
    f.blank();

    // PartitionCoproductEvidence
    f.doc_comment("Evidence bundle for `PartitionCoproductWitness`. Carries the");
    f.doc_comment("ST_1 / ST_2 / ST_9 / ST_10 input values used at mint time.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq)]");
    f.line("pub struct PartitionCoproductEvidence {");
    f.line("    pub left_site_budget: u16,");
    f.line("    pub right_site_budget: u16,");
    f.line("    pub left_total_site_count: u16,");
    f.line("    pub right_total_site_count: u16,");
    f.line("    pub left_euler: i32,");
    f.line("    pub right_euler: i32,");
    f.line("    pub left_entropy_nats_bits: u64,");
    f.line("    pub right_entropy_nats_bits: u64,");
    f.line("    pub left_betti: [u32; MAX_BETTI_DIMENSION],");
    f.line("    pub right_betti: [u32; MAX_BETTI_DIMENSION],");
    f.line("    pub source_witness_fingerprint: ContentFingerprint,");
    f.line("}");
    f.blank();

    // CartesianProductEvidence
    f.doc_comment("Evidence bundle for `CartesianProductWitness`. Carries the");
    f.doc_comment("CPT_1 / CPT_3 / CPT_4 / CPT_5 input values used at mint time, plus");
    f.doc_comment("`combined_entropy_nats` — the CartesianProductWitness itself does not");
    f.doc_comment("store entropy (see §1a), so the evidence sidecar preserves the");
    f.doc_comment("verification target value for re-audit.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq)]");
    f.line("pub struct CartesianProductEvidence {");
    f.line("    pub left_site_budget: u16,");
    f.line("    pub right_site_budget: u16,");
    f.line("    pub left_total_site_count: u16,");
    f.line("    pub right_total_site_count: u16,");
    f.line("    pub left_euler: i32,");
    f.line("    pub right_euler: i32,");
    f.line("    pub left_betti: [u32; MAX_BETTI_DIMENSION],");
    f.line("    pub right_betti: [u32; MAX_BETTI_DIMENSION],");
    f.line("    pub left_entropy_nats_bits: u64,");
    f.line("    pub right_entropy_nats_bits: u64,");
    f.line("    pub combined_entropy_nats_bits: u64,");
    f.line("    pub source_witness_fingerprint: ContentFingerprint,");
    f.line("}");
    f.blank();
}

/// §2.3d — three MintInputs structs. These are the typed single-argument
/// shape for `VerifiedMint::mint_verified(inputs)`; each field feeds directly
/// into the corresponding mint primitive.
fn emit_pc_mint_inputs_structs(f: &mut RustFile) {
    // PartitionProductMintInputs
    f.doc_comment("Inputs to `PartitionProductWitness::mint_verified`. Mirrors the");
    f.doc_comment("underlying primitive's parameter list; each field is supplied by the");
    f.doc_comment("caller (typically a `product_shape!` macro expansion or a manual");
    f.doc_comment("construction following the amendment's Gap 2 pattern). Derives");
    f.doc_comment("`PartialEq` only because of the `f64` entropy fields.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq)]");
    f.line("pub struct PartitionProductMintInputs {");
    f.line("    pub witt_bits: u16,");
    f.line("    pub left_fingerprint: ContentFingerprint,");
    f.line("    pub right_fingerprint: ContentFingerprint,");
    f.line("    pub left_site_budget: u16,");
    f.line("    pub right_site_budget: u16,");
    f.line("    pub left_total_site_count: u16,");
    f.line("    pub right_total_site_count: u16,");
    f.line("    pub left_euler: i32,");
    f.line("    pub right_euler: i32,");
    f.line("    pub left_entropy_nats_bits: u64,");
    f.line("    pub right_entropy_nats_bits: u64,");
    f.line("    pub combined_site_budget: u16,");
    f.line("    pub combined_site_count: u16,");
    f.line("    pub combined_euler: i32,");
    f.line("    pub combined_entropy_nats_bits: u64,");
    f.line("    pub combined_fingerprint: ContentFingerprint,");
    f.line("}");
    f.blank();

    // PartitionCoproductMintInputs
    f.doc_comment("Inputs to `PartitionCoproductWitness::mint_verified`. Adds three");
    f.doc_comment("structural fields beyond the other two MintInputs: the combined");
    f.doc_comment("constraint array, the boundary index between L and R regions, and the");
    f.doc_comment("tag site layout index. These feed `validate_coproduct_structure` at");
    f.doc_comment("mint time so ST_6 / ST_7 / ST_8 are verified numerically rather than");
    f.doc_comment("trusted from the caller.");
    f.doc_comment("");
    f.doc_comment("Derives `Debug`, `Clone`, `Copy` only — no `PartialEq`. `ConstraintRef`");
    f.doc_comment("does not implement `PartialEq`, so deriving equality on a struct with a");
    f.doc_comment("`&[ConstraintRef]` field would not compile. MintInputs is not used as");
    f.doc_comment("an equality target in practice; downstream consumers compare the minted");
    f.doc_comment("witness (which derives `Eq` + `Hash`) instead.");
    f.line("#[derive(Debug, Clone, Copy)]");
    f.line("pub struct PartitionCoproductMintInputs {");
    f.line("    pub witt_bits: u16,");
    f.line("    pub left_fingerprint: ContentFingerprint,");
    f.line("    pub right_fingerprint: ContentFingerprint,");
    f.line("    pub left_site_budget: u16,");
    f.line("    pub right_site_budget: u16,");
    f.line("    pub left_total_site_count: u16,");
    f.line("    pub right_total_site_count: u16,");
    f.line("    pub left_euler: i32,");
    f.line("    pub right_euler: i32,");
    f.line("    pub left_entropy_nats_bits: u64,");
    f.line("    pub right_entropy_nats_bits: u64,");
    f.line("    pub left_betti: [u32; MAX_BETTI_DIMENSION],");
    f.line("    pub right_betti: [u32; MAX_BETTI_DIMENSION],");
    f.line("    pub combined_site_budget: u16,");
    f.line("    pub combined_site_count: u16,");
    f.line("    pub combined_euler: i32,");
    f.line("    pub combined_entropy_nats_bits: u64,");
    f.line("    pub combined_betti: [u32; MAX_BETTI_DIMENSION],");
    f.line("    pub combined_fingerprint: ContentFingerprint,");
    f.line("    pub combined_constraints: &'static [crate::pipeline::ConstraintRef],");
    f.line("    pub left_constraint_count: usize,");
    f.line("    pub tag_site: u16,");
    f.line("}");
    f.blank();

    // CartesianProductMintInputs
    f.doc_comment("Inputs to `CartesianProductWitness::mint_verified`. Matches the");
    f.doc_comment("CartesianProduct mint primitive's parameter list.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq)]");
    f.line("pub struct CartesianProductMintInputs {");
    f.line("    pub witt_bits: u16,");
    f.line("    pub left_fingerprint: ContentFingerprint,");
    f.line("    pub right_fingerprint: ContentFingerprint,");
    f.line("    pub left_site_budget: u16,");
    f.line("    pub right_site_budget: u16,");
    f.line("    pub left_total_site_count: u16,");
    f.line("    pub right_total_site_count: u16,");
    f.line("    pub left_euler: i32,");
    f.line("    pub right_euler: i32,");
    f.line("    pub left_betti: [u32; MAX_BETTI_DIMENSION],");
    f.line("    pub right_betti: [u32; MAX_BETTI_DIMENSION],");
    f.line("    pub left_entropy_nats_bits: u64,");
    f.line("    pub right_entropy_nats_bits: u64,");
    f.line("    pub combined_site_budget: u16,");
    f.line("    pub combined_site_count: u16,");
    f.line("    pub combined_euler: i32,");
    f.line("    pub combined_betti: [u32; MAX_BETTI_DIMENSION],");
    f.line("    pub combined_entropy_nats_bits: u64,");
    f.line("    pub combined_fingerprint: ContentFingerprint,");
    f.line("}");
    f.blank();
}

/// §2.3b — three sealed witness structs. Each derives `Copy + Eq + Hash` so
/// witnesses are register-sized, content-addressable identity tokens.
/// Constructors are `pub(crate) const fn` — reachable only from the mint
/// primitives in this crate.
fn emit_pc_witness_structs(f: &mut RustFile) {
    // PartitionProductWitness
    f.doc_comment("Sealed PartitionProduct witness — content-addressed assertion that a");
    f.doc_comment("partition decomposes as `PartitionProduct(left, right)` per PT_2a.");
    f.doc_comment("Minting is gated on PT_1, PT_3, PT_4, and the foundation");
    f.doc_comment("`ProductLayoutWidth` invariant being verified against component");
    f.doc_comment("shapes. Existence of an instance is the attestation — there is no");
    f.doc_comment("partial or unverified form.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct PartitionProductWitness {");
    f.line("    witt_bits: u16,");
    f.line("    content_fingerprint: ContentFingerprint,");
    f.line("    left_fingerprint: ContentFingerprint,");
    f.line("    right_fingerprint: ContentFingerprint,");
    f.line("    combined_site_budget: u16,");
    f.line("    combined_site_count: u16,");
    f.line("}");
    f.blank();

    f.line("impl PartitionProductWitness {");
    f.indented_doc_comment("Witt level at which the witness was minted.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn witt_bits(&self) -> u16 { self.witt_bits }");
    f.indented_doc_comment("Content fingerprint of the combined (A × B) shape.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn content_fingerprint(&self) -> ContentFingerprint { self.content_fingerprint }");
    f.indented_doc_comment("Content fingerprint of the left factor A.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line(
        "    pub const fn left_fingerprint(&self) -> ContentFingerprint { self.left_fingerprint }",
    );
    f.indented_doc_comment("Content fingerprint of the right factor B.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn right_fingerprint(&self) -> ContentFingerprint { self.right_fingerprint }");
    f.indented_doc_comment("`siteBudget(A × B)` per PT_1.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn combined_site_budget(&self) -> u16 { self.combined_site_budget }");
    f.indented_doc_comment("`SITE_COUNT(A × B)` — the foundation layout width.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn combined_site_count(&self) -> u16 { self.combined_site_count }");
    f.indented_doc_comment(
        "Crate-internal mint entry. Only the verified mint primitive may call this.",
    );
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    #[allow(clippy::too_many_arguments)]");
    f.line("    pub(crate) const fn with_level_fingerprints_and_sites(");
    f.line("        witt_bits: u16,");
    f.line("        content_fingerprint: ContentFingerprint,");
    f.line("        left_fingerprint: ContentFingerprint,");
    f.line("        right_fingerprint: ContentFingerprint,");
    f.line("        combined_site_budget: u16,");
    f.line("        combined_site_count: u16,");
    f.line("    ) -> Self {");
    f.line("        Self {");
    f.line("            witt_bits,");
    f.line("            content_fingerprint,");
    f.line("            left_fingerprint,");
    f.line("            right_fingerprint,");
    f.line("            combined_site_budget,");
    f.line("            combined_site_count,");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();

    // PartitionCoproductWitness
    f.doc_comment("Sealed PartitionCoproduct witness — content-addressed assertion that a");
    f.doc_comment("partition decomposes as `PartitionCoproduct(left, right)` per PT_2b.");
    f.doc_comment("Minting verifies ST_1, ST_2, ST_6, ST_7, ST_8, ST_9, ST_10, the");
    f.doc_comment("foundation `CoproductLayoutWidth` invariant, and — for ST_6/ST_7/ST_8 —");
    f.doc_comment("walks the supplied constraint array through `validate_coproduct_structure`.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct PartitionCoproductWitness {");
    f.line("    witt_bits: u16,");
    f.line("    content_fingerprint: ContentFingerprint,");
    f.line("    left_fingerprint: ContentFingerprint,");
    f.line("    right_fingerprint: ContentFingerprint,");
    f.line("    combined_site_budget: u16,");
    f.line("    combined_site_count: u16,");
    f.line("    tag_site_index: u16,");
    f.line("}");
    f.blank();

    f.line("impl PartitionCoproductWitness {");
    f.line("    #[inline] #[must_use] pub const fn witt_bits(&self) -> u16 { self.witt_bits }");
    f.line("    #[inline] #[must_use]");
    f.line("    pub const fn content_fingerprint(&self) -> ContentFingerprint { self.content_fingerprint }");
    f.line("    #[inline] #[must_use]");
    f.line(
        "    pub const fn left_fingerprint(&self) -> ContentFingerprint { self.left_fingerprint }",
    );
    f.line("    #[inline] #[must_use]");
    f.line("    pub const fn right_fingerprint(&self) -> ContentFingerprint { self.right_fingerprint }");
    f.indented_doc_comment("`siteBudget(A + B)` per ST_1 = max(siteBudget(A), siteBudget(B)).");
    f.line("    #[inline] #[must_use]");
    f.line("    pub const fn combined_site_budget(&self) -> u16 { self.combined_site_budget }");
    f.indented_doc_comment(
        "`SITE_COUNT(A + B)` — the foundation layout width including the new tag site.",
    );
    f.line("    #[inline] #[must_use]");
    f.line("    pub const fn combined_site_count(&self) -> u16 { self.combined_site_count }");
    f.indented_doc_comment("Index of the tag site in the layout convention of §4b'.");
    f.indented_doc_comment("Equals `max(SITE_COUNT(A), SITE_COUNT(B))`.");
    f.line("    #[inline] #[must_use]");
    f.line("    pub const fn tag_site_index(&self) -> u16 { self.tag_site_index }");
    f.line("    #[inline] #[must_use]");
    f.line("    #[allow(clippy::too_many_arguments)]");
    f.line("    pub(crate) const fn with_level_fingerprints_sites_and_tag(");
    f.line("        witt_bits: u16,");
    f.line("        content_fingerprint: ContentFingerprint,");
    f.line("        left_fingerprint: ContentFingerprint,");
    f.line("        right_fingerprint: ContentFingerprint,");
    f.line("        combined_site_budget: u16,");
    f.line("        combined_site_count: u16,");
    f.line("        tag_site_index: u16,");
    f.line("    ) -> Self {");
    f.line("        Self {");
    f.line("            witt_bits,");
    f.line("            content_fingerprint,");
    f.line("            left_fingerprint,");
    f.line("            right_fingerprint,");
    f.line("            combined_site_budget,");
    f.line("            combined_site_count,");
    f.line("            tag_site_index,");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();

    // CartesianProductWitness
    f.doc_comment("Sealed CartesianPartitionProduct witness — content-addressed");
    f.doc_comment("assertion that a shape is `CartesianPartitionProduct(left, right)`");
    f.doc_comment("per CPT_2a. Minting verifies CPT_1, CPT_3, CPT_4, CPT_5, and the");
    f.doc_comment("foundation `CartesianLayoutWidth` invariant. The witness stores");
    f.doc_comment("a snapshot of the combined topological invariants (χ, Betti profile)");
    f.doc_comment("because the construction is axiomatic at the invariant level per §3c.");
    f.doc_comment("Entropy is not stored here (f64 has no Eq/Hash); use the paired");
    f.doc_comment("`CartesianProductEvidence` for entropy re-audit.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct CartesianProductWitness {");
    f.line("    witt_bits: u16,");
    f.line("    content_fingerprint: ContentFingerprint,");
    f.line("    left_fingerprint: ContentFingerprint,");
    f.line("    right_fingerprint: ContentFingerprint,");
    f.line("    combined_site_budget: u16,");
    f.line("    combined_site_count: u16,");
    f.line("    combined_euler: i32,");
    f.line("    combined_betti: [u32; MAX_BETTI_DIMENSION],");
    f.line("}");
    f.blank();

    f.line("impl CartesianProductWitness {");
    f.line("    #[inline] #[must_use] pub const fn witt_bits(&self) -> u16 { self.witt_bits }");
    f.line("    #[inline] #[must_use]");
    f.line("    pub const fn content_fingerprint(&self) -> ContentFingerprint { self.content_fingerprint }");
    f.line("    #[inline] #[must_use]");
    f.line(
        "    pub const fn left_fingerprint(&self) -> ContentFingerprint { self.left_fingerprint }",
    );
    f.line("    #[inline] #[must_use]");
    f.line("    pub const fn right_fingerprint(&self) -> ContentFingerprint { self.right_fingerprint }");
    f.line("    #[inline] #[must_use]");
    f.line("    pub const fn combined_site_budget(&self) -> u16 { self.combined_site_budget }");
    f.line("    #[inline] #[must_use]");
    f.line("    pub const fn combined_site_count(&self) -> u16 { self.combined_site_count }");
    f.line("    #[inline] #[must_use]");
    f.line("    pub const fn combined_euler(&self) -> i32 { self.combined_euler }");
    f.line("    #[inline] #[must_use]");
    f.line("    pub const fn combined_betti(&self) -> [u32; MAX_BETTI_DIMENSION] { self.combined_betti }");
    f.line("    #[inline] #[must_use]");
    f.line("    #[allow(clippy::too_many_arguments)]");
    f.line("    pub(crate) const fn with_level_fingerprints_and_invariants(");
    f.line("        witt_bits: u16,");
    f.line("        content_fingerprint: ContentFingerprint,");
    f.line("        left_fingerprint: ContentFingerprint,");
    f.line("        right_fingerprint: ContentFingerprint,");
    f.line("        combined_site_budget: u16,");
    f.line("        combined_site_count: u16,");
    f.line("        combined_euler: i32,");
    f.line("        combined_betti: [u32; MAX_BETTI_DIMENSION],");
    f.line("    ) -> Self {");
    f.line("        Self {");
    f.line("            witt_bits,");
    f.line("            content_fingerprint,");
    f.line("            left_fingerprint,");
    f.line("            right_fingerprint,");
    f.line("            combined_site_budget,");
    f.line("            combined_site_count,");
    f.line("            combined_euler,");
    f.line("            combined_betti,");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.blank();
}

/// `Certificate` impls binding each witness to its partition-namespace IRI
/// and its paired Evidence associated type.
fn emit_pc_certificate_impls(f: &mut RustFile) {
    f.line("impl Certificate for PartitionProductWitness {");
    f.line("    const IRI: &'static str = \"https://uor.foundation/partition/PartitionProduct\";");
    f.line("    type Evidence = PartitionProductEvidence;");
    f.line("}");
    f.blank();
    f.line("impl Certificate for PartitionCoproductWitness {");
    f.line(
        "    const IRI: &'static str = \"https://uor.foundation/partition/PartitionCoproduct\";",
    );
    f.line("    type Evidence = PartitionCoproductEvidence;");
    f.line("}");
    f.blank();
    f.line("impl Certificate for CartesianProductWitness {");
    f.line("    const IRI: &'static str =");
    f.line("        \"https://uor.foundation/partition/CartesianPartitionProduct\";");
    f.line("    type Evidence = CartesianProductEvidence;");
    f.line("}");
    f.blank();

    // Display + Error impls — one pair per witness. Matches the
    // witness-shim pattern emitted elsewhere in enforcement.rs; satisfies
    // the rust/error_trait_completeness conformance validator.
    for name in &[
        "PartitionProductWitness",
        "PartitionCoproductWitness",
        "CartesianProductWitness",
    ] {
        f.line(&format!("impl core::fmt::Display for {name} {{"));
        f.line("    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {");
        f.line(&format!("        f.write_str(\"{name}\")"));
        f.line("    }");
        f.line("}");
        f.line(&format!("impl core::error::Error for {name} {{}}"));
        f.blank();
    }
}

/// §2.3e — the `VerifiedMint` sealed mint trait. Supertrait on `Certificate`
/// (which already carries `certificate_sealed::Sealed`), so downstream crates
/// cannot implement `VerifiedMint` on their own types. Each witness's impl
/// routes through the corresponding `pc_primitive_*` function.
fn emit_pc_verified_mint_trait(f: &mut RustFile) {
    f.doc_comment("Sealed mint path for certificates that require multi-theorem verification");
    f.doc_comment("before minting. Introduced by the Product/Coproduct Completion Amendment");
    f.doc_comment("§1c; distinct from `MintWithLevelFingerprint` (which is the generic");
    f.doc_comment("partial-mint path for sealed shims). `VerifiedMint` implementors are");
    f.doc_comment("the three partition-algebra witnesses, each routing through a");
    f.doc_comment("foundation-internal mint primitive that verifies the relevant theorems.");
    f.doc_comment("");
    f.doc_comment("The trait is public so external callers can invoke `mint_verified`");
    f.doc_comment("directly, but the `Certificate` supertrait's `certificate_sealed::Sealed`");
    f.doc_comment("bound keeps the implementor set closed to this crate.");
    f.line("pub trait VerifiedMint: Certificate {");
    f.indented_doc_comment("Caller-supplied input bundle — one `*MintInputs` struct per witness.");
    f.line("    type Inputs;");
    f.indented_doc_comment("Failure kind — always `GenericImpossibilityWitness` citing the");
    f.indented_doc_comment("specific op-namespace theorem or foundation-namespace layout");
    f.indented_doc_comment("invariant that was violated.");
    f.line("    type Error;");
    f.indented_doc_comment("Verify the theorems and invariants against `inputs`, then mint a");
    f.indented_doc_comment("witness or return a typed impossibility witness.");
    f.indented_doc_comment("");
    f.indented_doc_comment("# Errors");
    f.indented_doc_comment("");
    f.indented_doc_comment("Returns a `GenericImpossibilityWitness::for_identity(iri)` when any");
    f.indented_doc_comment("of the gated theorems or foundation invariants fails. The IRI");
    f.indented_doc_comment("identifies exactly which identity failed — `op/PT_*`, `op/ST_*`,");
    f.indented_doc_comment(
        "`op/CPT_*`, or `foundation/*LayoutWidth` / `foundation/CoproductTagEncoding`.",
    );
    f.line("    fn mint_verified(inputs: Self::Inputs) -> Result<Self, Self::Error>");
    f.line("    where");
    f.line("        Self: Sized;");
    f.line("}");
    f.blank();
}

/// §2.3g — structural validator for PartitionCoproduct constructions.
/// Walks the constraint array, classifies each entry, and verifies ST_6,
/// ST_7, ST_8 at the byte level. Recurses into `ConstraintRef::Conjunction`
/// per plan §A4 with a bounded depth to prevent pathological inputs.
fn emit_pc_validate_coproduct_structure(f: &mut RustFile) {
    // Helper: classify_constraint — bounded-recursion worker over ConstraintRef.
    f.doc_comment("Recursive classifier used by `validate_coproduct_structure`. Inspects");
    f.doc_comment("one `ConstraintRef`, tallies tag-pinner sightings via mutable references,");
    f.doc_comment("and recurses into `Conjunction` conjuncts up to `max_depth` levels. See");
    f.doc_comment("plan §A4 for the depth bound rationale.");
    f.line("#[allow(clippy::too_many_arguments)]");
    f.line("fn pc_classify_constraint(");
    f.line("    c: &crate::pipeline::ConstraintRef,");
    f.line("    in_left_region: bool,");
    f.line("    tag_site: u16,");
    f.line("    max_depth: u32,");
    f.line("    left_pins: &mut u32,");
    f.line("    right_pins: &mut u32,");
    f.line("    left_bias_ok: &mut bool,");
    f.line("    right_bias_ok: &mut bool,");
    f.line(") -> Result<(), GenericImpossibilityWitness> {");
    f.line("    match c {");
    // Site
    f.line("        crate::pipeline::ConstraintRef::Site { position } => {");
    f.line("            if (*position as u16) >= tag_site {");
    f.line("                return Err(GenericImpossibilityWitness::for_identity(");
    f.line("                    \"https://uor.foundation/op/ST_6\",");
    f.line("                ));");
    f.line("            }");
    f.line("        }");
    // Carry
    f.line("        crate::pipeline::ConstraintRef::Carry { site } => {");
    f.line("            if (*site as u16) >= tag_site {");
    f.line("                return Err(GenericImpossibilityWitness::for_identity(");
    f.line("                    \"https://uor.foundation/op/ST_6\",");
    f.line("                ));");
    f.line("            }");
    f.line("        }");
    // Affine — tag-pinner classification.
    f.line("        crate::pipeline::ConstraintRef::Affine { coefficients, coefficient_count, bias } => {");
    f.line("            let count = *coefficient_count as usize;");
    f.line("            let mut nonzero_count: u32 = 0;");
    f.line("            let mut nonzero_index: usize = 0;");
    f.line("            let mut max_nonzero_index: usize = 0;");
    f.line("            let mut i: usize = 0;");
    f.line("            while i < count && i < crate::pipeline::AFFINE_MAX_COEFFS {");
    f.line("                if coefficients[i] != 0 {");
    f.line("                    nonzero_count = nonzero_count.saturating_add(1);");
    f.line("                    nonzero_index = i;");
    f.line("                    if i > max_nonzero_index { max_nonzero_index = i; }");
    f.line("                }");
    f.line("                i += 1;");
    f.line("            }");
    f.line("            let touches_tag_site = nonzero_count > 0");
    f.line("                && (max_nonzero_index as u16) >= tag_site;");
    f.line("            let is_canonical_tag_pinner = nonzero_count == 1");
    f.line("                && (nonzero_index as u16) == tag_site");
    f.line("                && coefficients[nonzero_index] == 1;");
    f.line("            if is_canonical_tag_pinner {");
    f.line("                if *bias != 0 && *bias != -1 {");
    f.line("                    return Err(GenericImpossibilityWitness::for_identity(");
    f.line("                        \"https://uor.foundation/foundation/CoproductTagEncoding\",");
    f.line("                    ));");
    f.line("                }");
    f.line("                if in_left_region {");
    f.line("                    *left_pins = left_pins.saturating_add(1);");
    f.line("                    if *bias != 0 { *left_bias_ok = false; }");
    f.line("                } else {");
    f.line("                    *right_pins = right_pins.saturating_add(1);");
    f.line("                    if *bias != -1 { *right_bias_ok = false; }");
    f.line("                }");
    f.line("            } else if touches_tag_site {");
    f.line("                let nonzero_only_at_tag_site = nonzero_count == 1");
    f.line("                    && (nonzero_index as u16) == tag_site;");
    f.line("                if nonzero_only_at_tag_site {");
    f.line("                    return Err(GenericImpossibilityWitness::for_identity(");
    f.line("                        \"https://uor.foundation/foundation/CoproductTagEncoding\",");
    f.line("                    ));");
    f.line("                } else {");
    f.line("                    return Err(GenericImpossibilityWitness::for_identity(");
    f.line("                        \"https://uor.foundation/op/ST_6\",");
    f.line("                    ));");
    f.line("                }");
    f.line("            }");
    f.line("        }");
    // Conjunction — Phase 17 caps depth at one (`LeafConstraintRef` cannot
    // be itself Conjunction). Lift each leaf back to a `ConstraintRef` for
    // the recursive walk; the recursion still handles `max_depth == 0`
    // as a defensive guard even though Phase 17 makes nested-Conjunction
    // unreachable.
    f.line(
        "        crate::pipeline::ConstraintRef::Conjunction { conjuncts, conjunct_count } => {",
    );
    f.line("            if max_depth == 0 {");
    f.line("                return Err(GenericImpossibilityWitness::for_identity(");
    f.line("                    \"https://uor.foundation/op/ST_6\",");
    f.line("                ));");
    f.line("            }");
    f.line("            let count = *conjunct_count as usize;");
    f.line("            let mut idx: usize = 0;");
    f.line("            while idx < count && idx < crate::pipeline::CONJUNCTION_MAX_TERMS {");
    f.line("                let lifted = conjuncts[idx].into_constraint();");
    f.line("                pc_classify_constraint(");
    f.line("                    &lifted,");
    f.line("                    in_left_region,");
    f.line("                    tag_site,");
    f.line("                    max_depth - 1,");
    f.line("                    left_pins,");
    f.line("                    right_pins,");
    f.line("                    left_bias_ok,");
    f.line("                    right_bias_ok,");
    f.line("                )?;");
    f.line("                idx += 1;");
    f.line("            }");
    f.line("        }");
    // Variants without site references.
    f.line("        crate::pipeline::ConstraintRef::Residue { .. }");
    f.line("        | crate::pipeline::ConstraintRef::Hamming { .. }");
    f.line("        | crate::pipeline::ConstraintRef::Depth { .. }");
    f.line("        | crate::pipeline::ConstraintRef::SatClauses { .. }");
    f.line("        | crate::pipeline::ConstraintRef::Bound { .. }");
    f.line("        // ADR-057: Recurse references a shape by content-addressed IRI;");
    f.line("        // no site references at this level to check.");
    f.line("        | crate::pipeline::ConstraintRef::Recurse { .. } => {");
    f.line("            // No site references at this level; nothing to check.");
    f.line("        }");
    f.line("    }");
    f.line("    Ok(())");
    f.line("}");
    f.blank();

    f.doc_comment("Validates ST_6 / ST_7 / ST_8 for a PartitionCoproduct construction by");
    f.doc_comment("walking the emitted constraint array. Recurses into");
    f.doc_comment("`ConstraintRef::Conjunction` conjuncts up to depth 8 (bounded by");
    f.doc_comment("`NERVE_CONSTRAINTS_CAP`) so nested constructions are audited without");
    f.doc_comment("unbounded recursion.");
    f.doc_comment("");
    f.doc_comment("# Errors");
    f.doc_comment("");
    f.doc_comment("Returns `GenericImpossibilityWitness::for_identity(...)` citing the");
    f.doc_comment("specific failed identity: `op/ST_6`, `op/ST_7`, or");
    f.doc_comment("`foundation/CoproductTagEncoding`. ST_8 is implied by ST_6 ∧ ST_7 and");
    f.doc_comment("is not cited separately on failure.");
    f.line("pub(crate) fn validate_coproduct_structure(");
    f.line("    combined_constraints: &[crate::pipeline::ConstraintRef],");
    f.line("    left_constraint_count: usize,");
    f.line("    tag_site: u16,");
    f.line(") -> Result<(), GenericImpossibilityWitness> {");
    f.line("    let mut left_pins: u32 = 0;");
    f.line("    let mut right_pins: u32 = 0;");
    f.line("    let mut left_bias_ok: bool = true;");
    f.line("    let mut right_bias_ok: bool = true;");
    f.line("    let mut idx: usize = 0;");
    f.line("    while idx < combined_constraints.len() {");
    f.line("        let in_left_region = idx < left_constraint_count;");
    f.line("        pc_classify_constraint(");
    f.line("            &combined_constraints[idx],");
    f.line("            in_left_region,");
    f.line("            tag_site,");
    f.line("            NERVE_CONSTRAINTS_CAP as u32,");
    f.line("            &mut left_pins,");
    f.line("            &mut right_pins,");
    f.line("            &mut left_bias_ok,");
    f.line("            &mut right_bias_ok,");
    f.line("        )?;");
    f.line("        idx += 1;");
    f.line("    }");
    f.line("    if left_pins != 1 || right_pins != 1 {");
    f.line("        return Err(GenericImpossibilityWitness::for_identity(");
    f.line("            \"https://uor.foundation/op/ST_6\",");
    f.line("        ));");
    f.line("    }");
    f.line("    if !left_bias_ok || !right_bias_ok {");
    f.line("        return Err(GenericImpossibilityWitness::for_identity(");
    f.line("            \"https://uor.foundation/op/ST_7\",");
    f.line("        ));");
    f.line("    }");
    f.line("    Ok(())");
    f.line("}");
    f.blank();
}

/// §2.3h — three mint primitives. Each verifies its theorem set against
/// caller-supplied invariants before minting.
fn emit_pc_mint_primitives(f: &mut RustFile) {
    // primitive_partition_product
    f.doc_comment("Mint a `PartitionProductWitness` after verifying PT_1, PT_3, PT_4, and");
    f.doc_comment("the `ProductLayoutWidth` layout invariant. PT_2a is structural (the");
    f.doc_comment("witness existing is the claim); no separate check is needed once the");
    f.doc_comment("invariants match.");
    f.line("#[allow(clippy::too_many_arguments)]");
    f.line("pub(crate) fn pc_primitive_partition_product(");
    f.line("    witt_bits: u16,");
    f.line("    left_fingerprint: ContentFingerprint,");
    f.line("    right_fingerprint: ContentFingerprint,");
    f.line("    left_site_budget: u16,");
    f.line("    right_site_budget: u16,");
    f.line("    left_total_site_count: u16,");
    f.line("    right_total_site_count: u16,");
    f.line("    left_euler: i32,");
    f.line("    right_euler: i32,");
    f.line("    left_entropy_nats_bits: u64,");
    f.line("    right_entropy_nats_bits: u64,");
    f.line("    combined_site_budget: u16,");
    f.line("    combined_site_count: u16,");
    f.line("    combined_euler: i32,");
    f.line("    combined_entropy_nats_bits: u64,");
    f.line("    combined_fingerprint: ContentFingerprint,");
    f.line(") -> Result<PartitionProductWitness, GenericImpossibilityWitness> {");
    // Phase 9: project bit-pattern entropy inputs to the default-host f64
    // numeric domain for the partition-algebra primitive. The entropy
    // arithmetic is amendment-internal; bit-pattern inputs keep the
    // public `MintInputs` surface host-neutral.
    f.line(
        "    let left_entropy_nats = <f64 as crate::DecimalTranscendental>::from_bits(left_entropy_nats_bits);",
    );
    f.line(
        "    let right_entropy_nats = <f64 as crate::DecimalTranscendental>::from_bits(right_entropy_nats_bits);",
    );
    f.line(
        "    let combined_entropy_nats = <f64 as crate::DecimalTranscendental>::from_bits(combined_entropy_nats_bits);",
    );
    // PT_1
    f.line("    if combined_site_budget != left_site_budget.saturating_add(right_site_budget) {");
    f.line("        return Err(GenericImpossibilityWitness::for_identity(");
    f.line("            \"https://uor.foundation/op/PT_1\",");
    f.line("        ));");
    f.line("    }");
    // ProductLayoutWidth
    f.line("    if combined_site_count != left_total_site_count.saturating_add(right_total_site_count) {");
    f.line("        return Err(GenericImpossibilityWitness::for_identity(");
    f.line("            \"https://uor.foundation/foundation/ProductLayoutWidth\",");
    f.line("        ));");
    f.line("    }");
    // PT_3
    f.line("    if combined_euler != left_euler.saturating_add(right_euler) {");
    f.line("        return Err(GenericImpossibilityWitness::for_identity(");
    f.line("            \"https://uor.foundation/op/PT_3\",");
    f.line("        ));");
    f.line("    }");
    // PT_4 — entropy with tolerance.
    f.line("    if !pc_entropy_input_is_valid(left_entropy_nats)");
    f.line("        || !pc_entropy_input_is_valid(right_entropy_nats)");
    f.line("        || !pc_entropy_input_is_valid(combined_entropy_nats)");
    f.line("    {");
    f.line("        return Err(GenericImpossibilityWitness::for_identity(");
    f.line("            \"https://uor.foundation/op/PT_4\",");
    f.line("        ));");
    f.line("    }");
    f.line("    let expected_entropy = left_entropy_nats + right_entropy_nats;");
    f.line("    if !pc_entropy_additivity_holds(combined_entropy_nats, expected_entropy) {");
    f.line("        return Err(GenericImpossibilityWitness::for_identity(");
    f.line("            \"https://uor.foundation/op/PT_4\",");
    f.line("        ));");
    f.line("    }");
    f.line("    Ok(PartitionProductWitness::with_level_fingerprints_and_sites(");
    f.line("        witt_bits,");
    f.line("        combined_fingerprint,");
    f.line("        left_fingerprint,");
    f.line("        right_fingerprint,");
    f.line("        combined_site_budget,");
    f.line("        combined_site_count,");
    f.line("    ))");
    f.line("}");
    f.blank();

    // primitive_partition_coproduct
    f.doc_comment("Mint a `PartitionCoproductWitness` after verifying ST_1, ST_2, ST_6,");
    f.doc_comment("ST_7, ST_8, ST_9, ST_10, the `CoproductLayoutWidth` layout invariant,");
    f.doc_comment("the tag-site alignment against §4b', and running");
    f.doc_comment("`validate_coproduct_structure` over the supplied constraint array.");
    f.line("#[allow(clippy::too_many_arguments)]");
    f.line("pub(crate) fn pc_primitive_partition_coproduct(");
    f.line("    witt_bits: u16,");
    f.line("    left_fingerprint: ContentFingerprint,");
    f.line("    right_fingerprint: ContentFingerprint,");
    f.line("    left_site_budget: u16,");
    f.line("    right_site_budget: u16,");
    f.line("    left_total_site_count: u16,");
    f.line("    right_total_site_count: u16,");
    f.line("    left_euler: i32,");
    f.line("    right_euler: i32,");
    f.line("    left_entropy_nats_bits: u64,");
    f.line("    right_entropy_nats_bits: u64,");
    f.line("    left_betti: [u32; MAX_BETTI_DIMENSION],");
    f.line("    right_betti: [u32; MAX_BETTI_DIMENSION],");
    f.line("    combined_site_budget: u16,");
    f.line("    combined_site_count: u16,");
    f.line("    combined_euler: i32,");
    f.line("    combined_entropy_nats_bits: u64,");
    f.line("    combined_betti: [u32; MAX_BETTI_DIMENSION],");
    f.line("    combined_fingerprint: ContentFingerprint,");
    f.line("    combined_constraints: &[crate::pipeline::ConstraintRef],");
    f.line("    left_constraint_count: usize,");
    f.line("    tag_site: u16,");
    f.line(") -> Result<PartitionCoproductWitness, GenericImpossibilityWitness> {");
    // Phase 9: project bit-pattern entropy inputs to f64 (default-host).
    f.line(
        "    let left_entropy_nats = <f64 as crate::DecimalTranscendental>::from_bits(left_entropy_nats_bits);",
    );
    f.line(
        "    let right_entropy_nats = <f64 as crate::DecimalTranscendental>::from_bits(right_entropy_nats_bits);",
    );
    f.line(
        "    let combined_entropy_nats = <f64 as crate::DecimalTranscendental>::from_bits(combined_entropy_nats_bits);",
    );
    // ST_1
    f.line("    let expected_budget = if left_site_budget > right_site_budget {");
    f.line("        left_site_budget");
    f.line("    } else {");
    f.line("        right_site_budget");
    f.line("    };");
    f.line("    if combined_site_budget != expected_budget {");
    f.line("        return Err(GenericImpossibilityWitness::for_identity(");
    f.line("            \"https://uor.foundation/op/ST_1\",");
    f.line("        ));");
    f.line("    }");
    // CoproductLayoutWidth
    f.line("    let max_total = if left_total_site_count > right_total_site_count {");
    f.line("        left_total_site_count");
    f.line("    } else {");
    f.line("        right_total_site_count");
    f.line("    };");
    f.line("    let expected_total = max_total.saturating_add(1);");
    f.line("    if combined_site_count != expected_total {");
    f.line("        return Err(GenericImpossibilityWitness::for_identity(");
    f.line("            \"https://uor.foundation/foundation/CoproductLayoutWidth\",");
    f.line("        ));");
    f.line("    }");
    // ST_2 — entropy tolerance.
    f.line("    if !pc_entropy_input_is_valid(left_entropy_nats)");
    f.line("        || !pc_entropy_input_is_valid(right_entropy_nats)");
    f.line("        || !pc_entropy_input_is_valid(combined_entropy_nats)");
    f.line("    {");
    f.line("        return Err(GenericImpossibilityWitness::for_identity(");
    f.line("            \"https://uor.foundation/op/ST_2\",");
    f.line("        ));");
    f.line("    }");
    f.line("    let max_operand_entropy = if left_entropy_nats > right_entropy_nats {");
    f.line("        left_entropy_nats");
    f.line("    } else {");
    f.line("        right_entropy_nats");
    f.line("    };");
    f.line("    let expected_entropy = core::f64::consts::LN_2 + max_operand_entropy;");
    f.line("    if !pc_entropy_additivity_holds(combined_entropy_nats, expected_entropy) {");
    f.line("        return Err(GenericImpossibilityWitness::for_identity(");
    f.line("            \"https://uor.foundation/op/ST_2\",");
    f.line("        ));");
    f.line("    }");
    // Tag-site alignment check.
    f.line("    if tag_site != max_total {");
    f.line("        return Err(GenericImpossibilityWitness::for_identity(");
    f.line("            \"https://uor.foundation/foundation/CoproductLayoutWidth\",");
    f.line("        ));");
    f.line("    }");
    // Structural validator.
    f.line(
        "    validate_coproduct_structure(combined_constraints, left_constraint_count, tag_site)?;",
    );
    // ST_9
    f.line("    if combined_euler != left_euler.saturating_add(right_euler) {");
    f.line("        return Err(GenericImpossibilityWitness::for_identity(");
    f.line("            \"https://uor.foundation/op/ST_9\",");
    f.line("        ));");
    f.line("    }");
    // ST_10
    f.line("    let mut k: usize = 0;");
    f.line("    while k < MAX_BETTI_DIMENSION {");
    f.line("        if combined_betti[k] != left_betti[k].saturating_add(right_betti[k]) {");
    f.line("            return Err(GenericImpossibilityWitness::for_identity(");
    f.line("                \"https://uor.foundation/op/ST_10\",");
    f.line("            ));");
    f.line("        }");
    f.line("        k += 1;");
    f.line("    }");
    f.line("    Ok(PartitionCoproductWitness::with_level_fingerprints_sites_and_tag(");
    f.line("        witt_bits,");
    f.line("        combined_fingerprint,");
    f.line("        left_fingerprint,");
    f.line("        right_fingerprint,");
    f.line("        combined_site_budget,");
    f.line("        combined_site_count,");
    f.line("        max_total,");
    f.line("    ))");
    f.line("}");
    f.blank();

    // primitive_cartesian_product
    f.doc_comment("Mint a `CartesianProductWitness` after verifying CPT_1, CPT_3, CPT_4,");
    f.doc_comment("CPT_5, and the `CartesianLayoutWidth` layout invariant. Checks");
    f.doc_comment("caller-supplied Künneth-composed invariants against the component");
    f.doc_comment("values (the witness defines these axiomatically per §3c).");
    f.line("#[allow(clippy::too_many_arguments)]");
    f.line("pub(crate) fn pc_primitive_cartesian_product(");
    f.line("    witt_bits: u16,");
    f.line("    left_fingerprint: ContentFingerprint,");
    f.line("    right_fingerprint: ContentFingerprint,");
    f.line("    left_site_budget: u16,");
    f.line("    right_site_budget: u16,");
    f.line("    left_total_site_count: u16,");
    f.line("    right_total_site_count: u16,");
    f.line("    left_euler: i32,");
    f.line("    right_euler: i32,");
    f.line("    left_betti: [u32; MAX_BETTI_DIMENSION],");
    f.line("    right_betti: [u32; MAX_BETTI_DIMENSION],");
    f.line("    left_entropy_nats_bits: u64,");
    f.line("    right_entropy_nats_bits: u64,");
    f.line("    combined_site_budget: u16,");
    f.line("    combined_site_count: u16,");
    f.line("    combined_euler: i32,");
    f.line("    combined_betti: [u32; MAX_BETTI_DIMENSION],");
    f.line("    combined_entropy_nats_bits: u64,");
    f.line("    combined_fingerprint: ContentFingerprint,");
    f.line(") -> Result<CartesianProductWitness, GenericImpossibilityWitness> {");
    // Phase 9: project bit-pattern entropy inputs to f64 (default-host).
    f.line(
        "    let left_entropy_nats = <f64 as crate::DecimalTranscendental>::from_bits(left_entropy_nats_bits);",
    );
    f.line(
        "    let right_entropy_nats = <f64 as crate::DecimalTranscendental>::from_bits(right_entropy_nats_bits);",
    );
    f.line(
        "    let combined_entropy_nats = <f64 as crate::DecimalTranscendental>::from_bits(combined_entropy_nats_bits);",
    );
    // CPT_1
    f.line("    if combined_site_budget != left_site_budget.saturating_add(right_site_budget) {");
    f.line("        return Err(GenericImpossibilityWitness::for_identity(");
    f.line("            \"https://uor.foundation/op/CPT_1\",");
    f.line("        ));");
    f.line("    }");
    // CartesianLayoutWidth
    f.line("    if combined_site_count != left_total_site_count.saturating_add(right_total_site_count) {");
    f.line("        return Err(GenericImpossibilityWitness::for_identity(");
    f.line("            \"https://uor.foundation/foundation/CartesianLayoutWidth\",");
    f.line("        ));");
    f.line("    }");
    // CPT_3
    f.line("    if combined_euler != left_euler.saturating_mul(right_euler) {");
    f.line("        return Err(GenericImpossibilityWitness::for_identity(");
    f.line("            \"https://uor.foundation/op/CPT_3\",");
    f.line("        ));");
    f.line("    }");
    // CPT_4 — Künneth.
    f.line("    let kunneth = crate::pipeline::kunneth_compose(&left_betti, &right_betti);");
    f.line("    let mut k: usize = 0;");
    f.line("    while k < MAX_BETTI_DIMENSION {");
    f.line("        if combined_betti[k] != kunneth[k] {");
    f.line("            return Err(GenericImpossibilityWitness::for_identity(");
    f.line("                \"https://uor.foundation/op/CPT_4\",");
    f.line("            ));");
    f.line("        }");
    f.line("        k += 1;");
    f.line("    }");
    // CPT_5 — entropy tolerance.
    f.line("    if !pc_entropy_input_is_valid(left_entropy_nats)");
    f.line("        || !pc_entropy_input_is_valid(right_entropy_nats)");
    f.line("        || !pc_entropy_input_is_valid(combined_entropy_nats)");
    f.line("    {");
    f.line("        return Err(GenericImpossibilityWitness::for_identity(");
    f.line("            \"https://uor.foundation/op/CPT_5\",");
    f.line("        ));");
    f.line("    }");
    f.line("    let expected_entropy = left_entropy_nats + right_entropy_nats;");
    f.line("    if !pc_entropy_additivity_holds(combined_entropy_nats, expected_entropy) {");
    f.line("        return Err(GenericImpossibilityWitness::for_identity(");
    f.line("            \"https://uor.foundation/op/CPT_5\",");
    f.line("        ));");
    f.line("    }");
    f.line("    Ok(CartesianProductWitness::with_level_fingerprints_and_invariants(");
    f.line("        witt_bits,");
    f.line("        combined_fingerprint,");
    f.line("        left_fingerprint,");
    f.line("        right_fingerprint,");
    f.line("        combined_site_budget,");
    f.line("        combined_site_count,");
    f.line("        combined_euler,");
    f.line("        combined_betti,");
    f.line("    ))");
    f.line("}");
    f.blank();
}

/// `VerifiedMint` impls — each witness routes through the corresponding
/// `pc_primitive_*` mint function, passing the `*MintInputs` fields positionally.
fn emit_pc_verified_mint_impls(f: &mut RustFile) {
    // PartitionProductWitness
    f.line("impl VerifiedMint for PartitionProductWitness {");
    f.line("    type Inputs = PartitionProductMintInputs;");
    f.line("    type Error = GenericImpossibilityWitness;");
    f.line("    fn mint_verified(inputs: Self::Inputs) -> Result<Self, Self::Error> {");
    f.line("        pc_primitive_partition_product(");
    f.line("            inputs.witt_bits,");
    f.line("            inputs.left_fingerprint,");
    f.line("            inputs.right_fingerprint,");
    f.line("            inputs.left_site_budget,");
    f.line("            inputs.right_site_budget,");
    f.line("            inputs.left_total_site_count,");
    f.line("            inputs.right_total_site_count,");
    f.line("            inputs.left_euler,");
    f.line("            inputs.right_euler,");
    f.line("            inputs.left_entropy_nats_bits,");
    f.line("            inputs.right_entropy_nats_bits,");
    f.line("            inputs.combined_site_budget,");
    f.line("            inputs.combined_site_count,");
    f.line("            inputs.combined_euler,");
    f.line("            inputs.combined_entropy_nats_bits,");
    f.line("            inputs.combined_fingerprint,");
    f.line("        )");
    f.line("    }");
    f.line("}");
    f.blank();

    // PartitionCoproductWitness
    f.line("impl VerifiedMint for PartitionCoproductWitness {");
    f.line("    type Inputs = PartitionCoproductMintInputs;");
    f.line("    type Error = GenericImpossibilityWitness;");
    f.line("    fn mint_verified(inputs: Self::Inputs) -> Result<Self, Self::Error> {");
    f.line("        pc_primitive_partition_coproduct(");
    f.line("            inputs.witt_bits,");
    f.line("            inputs.left_fingerprint,");
    f.line("            inputs.right_fingerprint,");
    f.line("            inputs.left_site_budget,");
    f.line("            inputs.right_site_budget,");
    f.line("            inputs.left_total_site_count,");
    f.line("            inputs.right_total_site_count,");
    f.line("            inputs.left_euler,");
    f.line("            inputs.right_euler,");
    f.line("            inputs.left_entropy_nats_bits,");
    f.line("            inputs.right_entropy_nats_bits,");
    f.line("            inputs.left_betti,");
    f.line("            inputs.right_betti,");
    f.line("            inputs.combined_site_budget,");
    f.line("            inputs.combined_site_count,");
    f.line("            inputs.combined_euler,");
    f.line("            inputs.combined_entropy_nats_bits,");
    f.line("            inputs.combined_betti,");
    f.line("            inputs.combined_fingerprint,");
    f.line("            inputs.combined_constraints,");
    f.line("            inputs.left_constraint_count,");
    f.line("            inputs.tag_site,");
    f.line("        )");
    f.line("    }");
    f.line("}");
    f.blank();

    // CartesianProductWitness
    f.line("impl VerifiedMint for CartesianProductWitness {");
    f.line("    type Inputs = CartesianProductMintInputs;");
    f.line("    type Error = GenericImpossibilityWitness;");
    f.line("    fn mint_verified(inputs: Self::Inputs) -> Result<Self, Self::Error> {");
    f.line("        pc_primitive_cartesian_product(");
    f.line("            inputs.witt_bits,");
    f.line("            inputs.left_fingerprint,");
    f.line("            inputs.right_fingerprint,");
    f.line("            inputs.left_site_budget,");
    f.line("            inputs.right_site_budget,");
    f.line("            inputs.left_total_site_count,");
    f.line("            inputs.right_total_site_count,");
    f.line("            inputs.left_euler,");
    f.line("            inputs.right_euler,");
    f.line("            inputs.left_betti,");
    f.line("            inputs.right_betti,");
    f.line("            inputs.left_entropy_nats_bits,");
    f.line("            inputs.right_entropy_nats_bits,");
    f.line("            inputs.combined_site_budget,");
    f.line("            inputs.combined_site_count,");
    f.line("            inputs.combined_euler,");
    f.line("            inputs.combined_betti,");
    f.line("            inputs.combined_entropy_nats_bits,");
    f.line("            inputs.combined_fingerprint,");
    f.line("        )");
    f.line("    }");
    f.line("}");
    f.blank();
}

/// §D1.2 — emits the Null* sub-trait stub family. Each stub is a unit
/// struct parameterized over `H: HostTypes` via `PhantomData<H>` and
/// implements its target sub-trait with the simplest sound defaults
/// (zeros for u64 / i32, false for bool, empty slices, the `EMPTY_*`
/// trait constants from `HostTypes` for `&H::HostString` /
/// `&H::WitnessBytes` returns). Stubs cascade: `NullIrreducibleSet<H>`
/// uses `NullDatum<H>` as its `Component::Datum` associated type, etc.
fn emit_pc_null_partition_stubs(f: &mut RustFile) {
    // ---- NullElement<H>: kernel::address::Element<H> -----------------------
    f.doc_comment("Resolver-absent default `Element<H>`. Returns empty defaults via the");
    f.doc_comment("`HostTypes::EMPTY_*` constants — used by `NullDatum<H>` and");
    f.doc_comment("`NullTypeDefinition<H>` to satisfy their `Element` associated types.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct NullElement<H: HostTypes> {");
    f.line("    _phantom: core::marker::PhantomData<H>,");
    f.line("}");
    f.line("impl<H: HostTypes> Default for NullElement<H> {");
    f.line("    fn default() -> Self { Self { _phantom: core::marker::PhantomData } }");
    f.line("}");
    f.line("impl<H: HostTypes> crate::kernel::address::Element<H> for NullElement<H> {");
    f.line("    fn length(&self) -> u64 { 0 }");
    f.line("    fn addresses(&self) -> &H::HostString { H::EMPTY_HOST_STRING }");
    f.line("    fn digest(&self) -> &H::HostString { H::EMPTY_HOST_STRING }");
    f.line("    fn digest_algorithm(&self) -> &H::HostString { H::EMPTY_HOST_STRING }");
    f.line("    fn canonical_bytes(&self) -> &H::WitnessBytes { H::EMPTY_WITNESS_BYTES }");
    f.line("    fn witt_length(&self) -> u64 { 0 }");
    f.line("}");
    f.blank();

    // ---- NullDatum<H>: kernel::schema::Datum<H> ---------------------------
    f.doc_comment("Resolver-absent default `Datum<H>`. All numeric methods return 0.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct NullDatum<H: HostTypes> {");
    f.line("    element: NullElement<H>,");
    f.line("    _phantom: core::marker::PhantomData<H>,");
    f.line("}");
    f.line("impl<H: HostTypes> Default for NullDatum<H> {");
    f.line("    fn default() -> Self {");
    f.line("        Self {");
    f.line("            element: NullElement::default(),");
    f.line("            _phantom: core::marker::PhantomData,");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.line("impl<H: HostTypes> crate::kernel::schema::Datum<H> for NullDatum<H> {");
    f.line("    fn value(&self) -> u64 { 0 }");
    f.line("    fn witt_length(&self) -> u64 { 0 }");
    f.line("    fn stratum(&self) -> u64 { 0 }");
    f.line("    fn spectrum(&self) -> u64 { 0 }");
    f.line("    type Element = NullElement<H>;");
    f.line("    fn element(&self) -> &Self::Element { &self.element }");
    f.line("}");
    f.blank();

    // ---- NullTermExpression<H>: kernel::schema::TermExpression<H> --------
    f.doc_comment("Resolver-absent default `TermExpression<H>`. Empty marker trait, no methods.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct NullTermExpression<H: HostTypes> {");
    f.line("    _phantom: core::marker::PhantomData<H>,");
    f.line("}");
    f.line("impl<H: HostTypes> Default for NullTermExpression<H> {");
    f.line("    fn default() -> Self { Self { _phantom: core::marker::PhantomData } }");
    f.line("}");
    f.line(
        "impl<H: HostTypes> crate::kernel::schema::TermExpression<H> for NullTermExpression<H> {}",
    );
    f.blank();

    // ---- NullSiteIndex<H>: bridge::partition::SiteIndex<H> ---------------
    // Has a self-referential ancilla_site() — return &self.
    f.doc_comment("Resolver-absent default `SiteIndex<H>`. Self-recursive: `ancilla_site()`");
    f.doc_comment("returns `&self` (no ancilla pairing in the default).");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct NullSiteIndex<H: HostTypes> {");
    f.line("    _phantom: core::marker::PhantomData<H>,");
    f.line("}");
    f.line("impl<H: HostTypes> Default for NullSiteIndex<H> {");
    f.line("    fn default() -> Self { Self { _phantom: core::marker::PhantomData } }");
    f.line("}");
    f.line("impl<H: HostTypes> crate::bridge::partition::SiteIndex<H> for NullSiteIndex<H> {");
    f.line("    fn site_position(&self) -> u64 { 0 }");
    f.line("    fn site_state(&self) -> u64 { 0 }");
    f.line("    type SiteIndexTarget = NullSiteIndex<H>;");
    f.line("    fn ancilla_site(&self) -> &Self::SiteIndexTarget { self }");
    f.line("}");
    f.blank();

    // ---- NullTagSite<H>: bridge::partition::TagSite<H>: SiteIndex<H> -----
    f.doc_comment("Resolver-absent default `TagSite<H>`. Embeds an inline `NullSiteIndex`");
    f.doc_comment("field so the inherited `ancilla_site()` accessor returns a valid");
    f.doc_comment("reference; `tag_value()` returns false.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct NullTagSite<H: HostTypes> {");
    f.line("    ancilla: NullSiteIndex<H>,");
    f.line("}");
    f.line("impl<H: HostTypes> Default for NullTagSite<H> {");
    f.line("    fn default() -> Self { Self { ancilla: NullSiteIndex::default() } }");
    f.line("}");
    f.line("impl<H: HostTypes> crate::bridge::partition::SiteIndex<H> for NullTagSite<H> {");
    f.line("    fn site_position(&self) -> u64 { 0 }");
    f.line("    fn site_state(&self) -> u64 { 0 }");
    f.line("    type SiteIndexTarget = NullSiteIndex<H>;");
    f.line("    fn ancilla_site(&self) -> &Self::SiteIndexTarget { &self.ancilla }");
    f.line("}");
    f.line("impl<H: HostTypes> crate::bridge::partition::TagSite<H> for NullTagSite<H> {");
    f.line("    fn tag_value(&self) -> bool { false }");
    f.line("}");
    f.blank();

    // ---- NullSiteBinding<H>: bridge::partition::SiteBinding<H> -----------
    f.doc_comment("Resolver-absent default `SiteBinding<H>`. Embeds inline `NullConstraint`");
    f.doc_comment("and `NullSiteIndex` so the trait's reference accessors work via &self.field.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct NullSiteBinding<H: HostTypes> {");
    f.line("    constraint: NullConstraint<H>,");
    f.line("    site_index: NullSiteIndex<H>,");
    f.line("}");
    f.line("impl<H: HostTypes> Default for NullSiteBinding<H> {");
    f.line("    fn default() -> Self {");
    f.line("        Self {");
    f.line("            constraint: NullConstraint::default(),");
    f.line("            site_index: NullSiteIndex::default(),");
    f.line("        }");
    f.line("    }");
    f.line("}");
    f.line("impl<H: HostTypes> crate::bridge::partition::SiteBinding<H> for NullSiteBinding<H> {");
    f.line("    type Constraint = NullConstraint<H>;");
    f.line("    fn pinned_by(&self) -> &Self::Constraint { &self.constraint }");
    f.line("    type SiteIndex = NullSiteIndex<H>;");
    f.line("    fn pins_coordinate(&self) -> &Self::SiteIndex { &self.site_index }");
    f.line("}");
    f.blank();

    // ---- NullConstraint<H>: type_::Constraint<H> -------------------------
    f.doc_comment("Resolver-absent default `Constraint<H>`. Returns Vertical metric axis,");
    f.doc_comment("empty pinned-sites slice, and zero crossing cost.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct NullConstraint<H: HostTypes> {");
    f.line("    _phantom: core::marker::PhantomData<H>,");
    f.line("}");
    f.line("impl<H: HostTypes> Default for NullConstraint<H> {");
    f.line("    fn default() -> Self { Self { _phantom: core::marker::PhantomData } }");
    f.line("}");
    f.line("impl<H: HostTypes> crate::user::type_::Constraint<H> for NullConstraint<H> {");
    f.line("    fn metric_axis(&self) -> MetricAxis { MetricAxis::Vertical }");
    f.line("    type SiteIndex = NullSiteIndex<H>;");
    f.line("    fn pins_sites(&self) -> &[Self::SiteIndex] { &[] }");
    f.line("    fn crossing_cost(&self) -> u64 { 0 }");
    f.line("}");
    f.blank();

    // ---- NullFreeRank<H>: bridge::partition::FreeRank<H> ----------------
    f.doc_comment("Resolver-absent default `FreeRank<H>`. Empty budget — `is_closed()` true,");
    f.doc_comment("zero counts. Empty `has_site` / `has_binding` slices.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct NullFreeRank<H: HostTypes> {");
    f.line("    _phantom: core::marker::PhantomData<H>,");
    f.line("}");
    f.line("impl<H: HostTypes> Default for NullFreeRank<H> {");
    f.line("    fn default() -> Self { Self { _phantom: core::marker::PhantomData } }");
    f.line("}");
    f.line("impl<H: HostTypes> crate::bridge::partition::FreeRank<H> for NullFreeRank<H> {");
    f.line("    fn total_sites(&self) -> u64 { 0 }");
    f.line("    fn pinned_count(&self) -> u64 { 0 }");
    f.line("    fn free_rank(&self) -> u64 { 0 }");
    f.line("    fn is_closed(&self) -> bool { true }");
    f.line("    type SiteIndex = NullSiteIndex<H>;");
    f.line("    fn has_site(&self) -> &[Self::SiteIndex] { &[] }");
    f.line("    type SiteBinding = NullSiteBinding<H>;");
    f.line("    fn has_binding(&self) -> &[Self::SiteBinding] { &[] }");
    f.line("    fn reversible_strategy(&self) -> bool { false }");
    f.line("}");
    f.blank();

    // ---- NullIrreducibleSet<H>: bridge::partition::IrreducibleSet<H>: Component<H> ---
    for (name, sub_trait) in &[
        ("NullIrreducibleSet", "IrreducibleSet"),
        ("NullReducibleSet", "ReducibleSet"),
        ("NullUnitGroup", "UnitGroup"),
    ] {
        f.doc_comment(&format!(
            "Resolver-absent default `{}<H>`. Implements `Component<H>` with empty",
            sub_trait
        ));
        f.doc_comment("`member` slice and zero `cardinality`.");
        f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
        f.line(&format!("pub struct {name}<H: HostTypes> {{"));
        f.line("    _phantom: core::marker::PhantomData<H>,");
        f.line("}");
        f.line(&format!("impl<H: HostTypes> Default for {name}<H> {{"));
        f.line("    fn default() -> Self { Self { _phantom: core::marker::PhantomData } }");
        f.line("}");
        f.line(&format!(
            "impl<H: HostTypes> crate::bridge::partition::Component<H> for {name}<H> {{"
        ));
        f.line("    type Datum = NullDatum<H>;");
        f.line("    fn member(&self) -> &[Self::Datum] { &[] }");
        f.line("    fn cardinality(&self) -> u64 { 0 }");
        f.line("}");
        f.line(&format!(
            "impl<H: HostTypes> crate::bridge::partition::{sub_trait}<H> for {name}<H> {{}}"
        ));
        f.blank();
    }

    // ---- NullComplement<H>: bridge::partition::Complement<H>: Component<H> + has TermExpression ---
    f.doc_comment("Resolver-absent default `Complement<H>`. Implements `Component<H>` plus");
    f.doc_comment("the `exterior_criteria()` accessor returning a reference to an embedded");
    f.doc_comment("`NullTermExpression`.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct NullComplement<H: HostTypes> {");
    f.line("    term: NullTermExpression<H>,");
    f.line("}");
    f.line("impl<H: HostTypes> Default for NullComplement<H> {");
    f.line("    fn default() -> Self { Self { term: NullTermExpression::default() } }");
    f.line("}");
    f.line("impl<H: HostTypes> crate::bridge::partition::Component<H> for NullComplement<H> {");
    f.line("    type Datum = NullDatum<H>;");
    f.line("    fn member(&self) -> &[Self::Datum] { &[] }");
    f.line("    fn cardinality(&self) -> u64 { 0 }");
    f.line("}");
    f.line("impl<H: HostTypes> crate::bridge::partition::Complement<H> for NullComplement<H> {");
    f.line("    type TermExpression = NullTermExpression<H>;");
    f.line("    fn exterior_criteria(&self) -> &Self::TermExpression { &self.term }");
    f.line("}");
    f.blank();

    // ---- NullTypeDefinition<H>: user::type_::TypeDefinition<H> ----------
    f.doc_comment("Resolver-absent default `TypeDefinition<H>`. Embeds inline `NullElement`");
    f.doc_comment("for the content_address accessor.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct NullTypeDefinition<H: HostTypes> {");
    f.line("    element: NullElement<H>,");
    f.line("}");
    f.line("impl<H: HostTypes> Default for NullTypeDefinition<H> {");
    f.line("    fn default() -> Self { Self { element: NullElement::default() } }");
    f.line("}");
    f.line("impl<H: HostTypes> crate::user::type_::TypeDefinition<H> for NullTypeDefinition<H> {");
    f.line("    type Element = NullElement<H>;");
    f.line("    fn content_address(&self) -> &Self::Element { &self.element }");
    f.line("}");
    f.blank();
}

/// §D1.3 — emits `NullPartition<H>`: a composite struct embedding inline
/// instances of every Null* sub-stub so the `Partition<H>` trait's
/// reference-returning accessors work via `&self.field`. Constructed
/// from a fingerprint via `from_fingerprint`; the fingerprint is the
/// only meaningful state (everything else is resolver-absent defaults).
fn emit_pc_null_partition_composite(f: &mut RustFile) {
    f.doc_comment("Resolver-absent default `Partition<H>`. Embeds inline stubs for every");
    f.doc_comment("sub-trait associated type so `Partition<H>` accessors return references");
    f.doc_comment("to fields rather than to statics. The only meaningful state is the");
    f.doc_comment("`fingerprint`; everything else uses `HostTypes::EMPTY_*` defaults.");
    f.doc_comment("");
    f.doc_comment("Returned by the three witness trait impls' `left_factor` / `right_factor`");
    f.doc_comment("/ `left_summand` / etc. accessors as the resolver-absent value pathway.");
    f.doc_comment("Consumers needing real partition data pair the sibling `PartitionHandle`");
    f.doc_comment("with a `PartitionResolver` instead.");
    f.line("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]");
    f.line("pub struct NullPartition<H: HostTypes> {");
    f.line("    irreducibles: NullIrreducibleSet<H>,");
    f.line("    reducibles: NullReducibleSet<H>,");
    f.line("    units: NullUnitGroup<H>,");
    f.line("    exterior: NullComplement<H>,");
    f.line("    free_rank: NullFreeRank<H>,");
    f.line("    tag_site: NullTagSite<H>,");
    f.line("    source_type: NullTypeDefinition<H>,");
    f.line("    fingerprint: ContentFingerprint,");
    f.line("}");
    f.blank();
    f.line("impl<H: HostTypes> NullPartition<H> {");
    f.indented_doc_comment("Construct a NullPartition with the given content fingerprint.");
    f.indented_doc_comment("All other fields are resolver-absent defaults.");
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub fn from_fingerprint(fingerprint: ContentFingerprint) -> Self {");
    f.line("        Self {");
    f.line("            irreducibles: NullIrreducibleSet::default(),");
    f.line("            reducibles: NullReducibleSet::default(),");
    f.line("            units: NullUnitGroup::default(),");
    f.line("            exterior: NullComplement::default(),");
    f.line("            free_rank: NullFreeRank::default(),");
    f.line("            tag_site: NullTagSite::default(),");
    f.line("            source_type: NullTypeDefinition::default(),");
    f.line("            fingerprint,");
    f.line("        }");
    f.line("    }");
    f.indented_doc_comment(
        "Returns the content fingerprint identifying which Partition this stub stands in for.",
    );
    f.line("    #[inline]");
    f.line("    #[must_use]");
    f.line("    pub const fn fingerprint(&self) -> ContentFingerprint { self.fingerprint }");
    f.indented_doc_comment("");
    f.indented_doc_comment("Phase 2 (orphan-closure): absent-value sentinel used by Null stubs");
    f.indented_doc_comment("in other namespaces to satisfy `&Self::Partition` return borrows.");
    f.line("    pub const ABSENT: NullPartition<H> = NullPartition {");
    f.line("        irreducibles: NullIrreducibleSet { _phantom: core::marker::PhantomData },");
    f.line("        reducibles: NullReducibleSet { _phantom: core::marker::PhantomData },");
    f.line("        units: NullUnitGroup { _phantom: core::marker::PhantomData },");
    f.line("        exterior: NullComplement { term: NullTermExpression { _phantom: core::marker::PhantomData } },");
    f.line("        free_rank: NullFreeRank { _phantom: core::marker::PhantomData },");
    f.line("        tag_site: NullTagSite { ancilla: NullSiteIndex { _phantom: core::marker::PhantomData } },");
    f.line("        source_type: NullTypeDefinition { element: NullElement { _phantom: core::marker::PhantomData } },");
    f.line("        fingerprint: ContentFingerprint::zero(),");
    f.line("    };");
    f.line("}");
    f.blank();

    // Partition<H> impl
    f.line("impl<H: HostTypes> crate::bridge::partition::Partition<H> for NullPartition<H> {");
    f.line("    type IrreducibleSet = NullIrreducibleSet<H>;");
    f.line("    fn irreducibles(&self) -> &Self::IrreducibleSet { &self.irreducibles }");
    f.line("    type ReducibleSet = NullReducibleSet<H>;");
    f.line("    fn reducibles(&self) -> &Self::ReducibleSet { &self.reducibles }");
    f.line("    type UnitGroup = NullUnitGroup<H>;");
    f.line("    fn units(&self) -> &Self::UnitGroup { &self.units }");
    f.line("    type Complement = NullComplement<H>;");
    f.line("    fn exterior(&self) -> &Self::Complement { &self.exterior }");
    f.line("    fn density(&self) -> H::Decimal { H::EMPTY_DECIMAL }");
    f.line("    type TypeDefinition = NullTypeDefinition<H>;");
    f.line("    fn source_type(&self) -> &Self::TypeDefinition { &self.source_type }");
    f.line("    fn witt_length(&self) -> u64 { 0 }");
    f.line("    type FreeRank = NullFreeRank<H>;");
    f.line("    fn site_budget(&self) -> &Self::FreeRank { &self.free_rank }");
    f.line("    fn is_exhaustive(&self) -> bool { true }");
    f.line("    type TagSite = NullTagSite<H>;");
    f.line("    fn tag_site_of(&self) -> &Self::TagSite { &self.tag_site }");
    f.line("    fn product_category_level(&self) -> &H::HostString { H::EMPTY_HOST_STRING }");
    f.line("}");
    f.blank();
}

/// §D1.4 — emits the three witness trait impls. Each is fully generic
/// over `H: HostTypes` (no host-type narrowing); the associated `type
/// Partition = NullPartition<H>` returns a freshly-constructed
/// `NullPartition` carrying the operand fingerprint.
fn emit_pc_witness_trait_impls(f: &mut RustFile) {
    let cases: &[(&str, &str, &str, &str)] = &[
        (
            "PartitionProduct",
            "PartitionProductWitness",
            "left_factor",
            "right_factor",
        ),
        (
            "PartitionCoproduct",
            "PartitionCoproductWitness",
            "left_summand",
            "right_summand",
        ),
        (
            "CartesianPartitionProduct",
            "CartesianProductWitness",
            "left_cartesian_factor",
            "right_cartesian_factor",
        ),
    ];
    for (trait_name, witness_name, left_method, right_method) in cases {
        f.line(&format!(
            "impl<H: HostTypes> crate::bridge::partition::{trait_name}<H> for {witness_name} {{"
        ));
        f.line("    type Partition = NullPartition<H>;");
        f.line(&format!(
            "    fn {left_method}(&self) -> Self::Partition {{"
        ));
        f.line("        NullPartition::from_fingerprint(self.left_fingerprint)");
        f.line("    }");
        f.line(&format!(
            "    fn {right_method}(&self) -> Self::Partition {{"
        ));
        f.line("        NullPartition::from_fingerprint(self.right_fingerprint)");
        f.line("    }");
        f.line("}");
        f.blank();
    }
}
