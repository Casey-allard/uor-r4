//! Phase 10 conformance gate: every Path-2 class has a `Mint{Foo}` +
//! `Mint{Foo}Inputs<H>` + `Certificate` + `OntologyVerifiedMint` block in
//! `foundation/src/witness_scaffolds.rs`, and every family-routed
//! primitive stub module exists at `foundation/src/primitives/{family}.rs`.

use std::path::Path;

use anyhow::Result;

use crate::report::{ConformanceReport, TestResult};

const VALIDATOR: &str = "rust/witness_scaffold_surface";

/// The Phase-10 expected emissions, hand-mirrored from
/// `uor_codegen::classification::PATH2_THEOREM_OVERRIDES`. Each tuple:
/// `(class_local, namespace, namespace_qualifier_needed, theorem_iri,
/// primitive_module, entropy_bearing)`.
const EXPECTED_PATH2: &[(&str, &str, bool, &str, &str, bool)] = &[
    (
        "BornRuleVerification",
        "cert",
        false,
        "https://uor.foundation/op/QM_5",
        "br",
        false,
    ),
    (
        "DisjointnessWitness",
        "effect",
        false,
        "https://uor.foundation/op/FX_4",
        "dp",
        false,
    ),
    (
        "GroundingWitness",
        "morphism",
        true,
        "https://uor.foundation/op/surfaceSymmetry",
        "oa",
        false,
    ),
    (
        "ProjectionWitness",
        "morphism",
        false,
        "https://uor.foundation/op/surfaceSymmetry",
        "oa",
        false,
    ),
    (
        "Witness",
        "morphism",
        false,
        "https://uor.foundation/op/surfaceSymmetry",
        "oa",
        false,
    ),
    (
        "ImpossibilityWitness",
        "proof",
        false,
        "https://uor.foundation/op/IH_1",
        "ih",
        false,
    ),
    (
        "InhabitanceImpossibilityWitness",
        "proof",
        false,
        "https://uor.foundation/op/IH_1",
        "ih",
        false,
    ),
    (
        "GroundingWitness",
        "state",
        true,
        "https://uor.foundation/op/surfaceSymmetry",
        "oa",
        false,
    ),
    (
        "CompletenessWitness",
        "type",
        false,
        "https://uor.foundation/op/CC_1",
        "cc",
        false,
    ),
    (
        "LiftObstruction",
        "type",
        false,
        "https://uor.foundation/op/WLS_2",
        "lo",
        false,
    ),
];

/// Runs the Phase 10 witness-scaffold surface validator.
///
/// # Errors
///
/// Returns an error if a foundation source file cannot be read.
pub fn validate(workspace: &Path) -> Result<ConformanceReport> {
    let mut report = ConformanceReport::new();

    let scaffolds_path = workspace.join("foundation/src/witness_scaffolds.rs");
    let scaffolds = match std::fs::read_to_string(&scaffolds_path) {
        Ok(c) => c,
        Err(e) => {
            report.push(TestResult::fail(
                VALIDATOR,
                format!("cannot read {}: {e}", scaffolds_path.display()),
            ));
            return Ok(report);
        }
    };

    let mut failures: Vec<String> = Vec::new();

    if !scaffolds.contains("pub trait OntologyVerifiedMint:") {
        failures.push("OntologyVerifiedMint trait declaration missing".to_string());
    }
    // Phase 14 added the `+ 'static` bound on the GAT so MintInputs
    // structs containing `&'static [{Range}Handle<H>]` non-functional
    // fields satisfy `Handle<H>: 'static`.
    if !scaffolds.contains("type Inputs<H: HostTypes + 'static>") {
        failures
            .push("OntologyVerifiedMint::Inputs<H: HostTypes + 'static> GAT missing".to_string());
    }
    if !scaffolds.contains("const THEOREM_IDENTITY:") {
        failures.push("OntologyVerifiedMint::THEOREM_IDENTITY const missing".to_string());
    }

    for (class_local, namespace, qualified, theorem_iri, primitive_module, entropy_bearing) in
        EXPECTED_PATH2
    {
        // Mint type name: namespace-qualified when local name collides.
        let qualifier = if *qualified {
            let mut ns = (*namespace).to_string();
            if let Some(c) = ns.get_mut(0..1) {
                c.make_ascii_uppercase();
            }
            ns
        } else {
            String::new()
        };
        let mint = format!("Mint{qualifier}{class_local}");
        let inputs = format!("{mint}Inputs");

        if !scaffolds.contains(&format!("pub struct {mint} {{")) {
            failures.push(format!(
                "missing `pub struct {mint}` (Path-2 class `{namespace}::{class_local}`)"
            ));
        }
        // Phase 14 — MintInputs<H> may carry the `+ 'static` bound when
        // any field is `&'static [Handle<H>]`. Accept either bound shape;
        // only require the `pub struct {inputs}<H` prefix.
        let inputs_decl_a = format!("pub struct {inputs}<H: HostTypes>");
        let inputs_decl_b = format!("pub struct {inputs}<H: HostTypes + 'static>");
        if !scaffolds.contains(&inputs_decl_a) && !scaffolds.contains(&inputs_decl_b) {
            failures.push(format!(
                "missing `pub struct {inputs}<H: HostTypes[+'static]>`"
            ));
        }
        if !scaffolds.contains(&format!("impl Certificate for {mint}")) {
            failures.push(format!("missing `impl Certificate for {mint}`"));
        }
        if !scaffolds.contains(&format!("impl OntologyVerifiedMint for {mint}")) {
            failures.push(format!("missing `impl OntologyVerifiedMint for {mint}`"));
        }
        // Phase 14 — verify the impl uses the `+ 'static` GAT shape.
        let inputs_assoc = format!("type Inputs<H: HostTypes + 'static> = {inputs}<H>;");
        if !scaffolds.contains(&inputs_assoc) {
            failures.push(format!(
                "{mint}'s OntologyVerifiedMint impl missing `type Inputs<H: HostTypes + 'static> = {inputs}<H>;`"
            ));
        }
        if !scaffolds.contains(&format!("\"{theorem_iri}\"")) {
            failures.push(format!(
                "scaffold for `{namespace}::{class_local}` missing THEOREM_IDENTITY `{theorem_iri}`"
            ));
        }

        // R7 Hash discipline. We reach the derive line for `pub struct {mint}`.
        let pat = format!("pub struct {mint} {{");
        if let Some(idx) = scaffolds.find(&pat) {
            // Walk back to the derive line.
            let pre = &scaffolds[..idx];
            if let Some(derive_pos) = pre.rfind("#[derive") {
                let derive_line = pre[derive_pos..].lines().next().unwrap_or("");
                if *entropy_bearing {
                    if derive_line.contains("Hash") {
                        failures.push(format!(
                            "entropy_bearing class `{namespace}::{class_local}` must NOT derive `Hash`"
                        ));
                    }
                } else if !derive_line.contains("Hash") {
                    failures.push(format!(
                        "non-entropy-bearing class `{namespace}::{class_local}` must derive `Hash`"
                    ));
                }
            }
        }

        // Per-family primitive stub file exists.
        let prim_path = workspace.join(format!("foundation/src/primitives/{primitive_module}.rs"));
        if !prim_path.exists() {
            failures.push(format!(
                "missing primitive module file `foundation/src/primitives/{primitive_module}.rs`"
            ));
        }
    }

    if failures.is_empty() {
        report.push(TestResult::pass(
            VALIDATOR,
            format!(
                "Phase 10 witness scaffolds: all {} Path-2 classes have \
                 Mint{{Foo}} + Mint{{Foo}}Inputs<H> + Certificate + \
                 OntologyVerifiedMint impls; per-family primitive stubs \
                 emitted",
                EXPECTED_PATH2.len()
            ),
        ));
    } else {
        let mut summary = format!(
            "Phase 10 witness scaffold drift: {} issue(s):",
            failures.len()
        );
        for f in &failures {
            summary.push_str("\n       - ");
            summary.push_str(f);
        }
        report.push(TestResult::fail(VALIDATOR, summary));
    }

    Ok(report)
}
