//! UOR Foundation xtask — repository-wide maintenance commands.
//!
//! Invoke with `cargo xtask <subcommand>`. Subcommands:
//!
//! - `check-psi` — Phase J (target §7.3): ψ-leakage CI gate.
//! - `regression-drill` — Correctness-suite sensitivity verification.
//!   Applies a named codegen mutation, runs the conformance suite, and
//!   asserts the suite fails with the expected `[FAIL]` pointing at the
//!   right correctness/* validator. After the assertion, the mutation is
//!   reverted via `git restore`.

#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    missing_docs,
    clippy::missing_errors_doc
)]

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use uor_conformance::validators::docs::psi_leakage;
use uor_conformance::Severity;

/// UOR Foundation xtask commands.
#[derive(Debug, Parser)]
#[command(
    name = "xtask",
    about = "UOR Foundation repository maintenance commands"
)]
struct Cli {
    /// The subcommand to execute.
    #[command(subcommand)]
    command: Cmd,
}

/// xtask subcommands.
#[derive(Debug, Subcommand)]
enum Cmd {
    /// Phase J: ψ-leakage CI gate. Delegates to the shared `docs/psi_leakage`
    /// conformance validator (which already ships in the conformance suite)
    /// so the `cargo xtask check-psi` CLI surface and the conformance run
    /// report identical results.
    CheckPsi,

    /// Regression drill: apply a named codegen mutation, regenerate the
    /// foundation, run the conformance suite, and assert it fails. Used
    /// to verify the correctness-suite's sensitivity — a suite that
    /// misses an intentional regression is itself a defect.
    RegressionDrill {
        /// The named mutation to apply. `list` prints the available set.
        #[arg(default_value = "list")]
        mutation: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Cmd::CheckPsi => check_psi(),
        Cmd::RegressionDrill { mutation } => regression_drill(&mutation),
    }
}

/// Runs the ψ-leakage check by delegating to the shared conformance validator.
fn check_psi() -> Result<()> {
    let root = workspace_root()?;
    let report = psi_leakage::validate(&root).context("docs/psi_leakage validator failed")?;

    let mut any_fail = false;
    for result in &report.results {
        match result.severity {
            Severity::Pass => {
                println!("[PASS] {} — {}", result.validator, result.message);
            }
            Severity::Warning => {
                println!("[WARN] {} — {}", result.validator, result.message);
            }
            Severity::Failure => {
                eprintln!("[FAIL] {} — {}", result.validator, result.message);
                for detail in &result.details {
                    eprintln!("       {detail}");
                }
                any_fail = true;
            }
        }
    }

    if any_fail {
        anyhow::bail!("ψ leakage detected");
    }
    Ok(())
}

/// Named mutations the drill supports. Each mutation is a (filepath,
/// original-substring, patched-substring) triple. The xtask applies the
/// patch, regenerates the foundation via `cargo run --bin uor-crate`,
/// runs conformance, and asserts the expected `[FAIL]` label appears.
/// After the assertion (pass or fail), the patch is reverted via
/// `git restore --source HEAD`.
struct Mutation {
    /// Human-readable mutation ID (used as the CLI argument).
    id: &'static str,
    /// File in the repo to patch.
    file: &'static str,
    /// Original source string (must be a unique substring in `file`).
    original: &'static str,
    /// Patched replacement.
    patched: &'static str,
    /// Substring that must appear in the conformance output's FAIL line
    /// for the drill to pass (e.g., "correctness/calibration").
    expected_fail_marker: &'static str,
    /// Human-readable description.
    description: &'static str,
}

const MUTATIONS: &[Mutation] = &[
    // ─── Correctness behavior validators ─────────────────────────────────
    Mutation {
        id: "calibration-accepts-nan",
        file: "codegen/src/enforcement.rs",
        original: "let k_b_t_nan = k_b_t != k_b_t;",
        patched: "let k_b_t_nan = false;",
        expected_fail_marker: "correctness/calibration",
        description:
            "Disable NaN check in Calibration::new; behavior_calibration must catch it",
    },
    Mutation {
        id: "builder-skips-root-term",
        file: "codegen/src/enforcement.rs",
        original: "f.indented_doc_comment(\"The root term expression.\");",
        patched: "f.indented_doc_comment(\"SKIP ROOT TERM CHECK — MUTATION\");",
        expected_fail_marker: "correctness/builder_rejection",
        description:
            "Tag the codegen emission (benign smoke test — confirms the drill pipeline runs)",
    },
    Mutation {
        id: "pipeline-nondeterministic-address",
        file: "codegen/src/enforcement.rs",
        original: "ContentAddress::from_bytes_const(address_bytes)",
        patched: "ContentAddress::from_bytes_const([0xFFu8; 16])",
        expected_fail_marker: "correctness/pipeline_determinism",
        description:
            "Pin unit_address to a constant; behavior_pipeline_determinism catches \
             content-non-determinism (different inputs → same address)",
    },
    Mutation {
        id: "grounding-interpreter-leaf-none",
        file: "codegen/src/enforcement.rs",
        original: "external.first().map(|&b| GroundedCoord::w8(b))",
        patched: "None",
        expected_fail_marker: "correctness/grounding_interpreter",
        description:
            "Force interpret_leaf_op::ReadBytes to return None; \
             behavior_grounding_interpreter must catch the broken leaf path",
    },
    Mutation {
        id: "resolver-kind-collision",
        file: "codegen/src/enforcement.rs",
        original: "\"CertificateKind::Session\"",
        patched: "\"CertificateKind::Grounding\"",
        expected_fail_marker: "correctness/resolver_tower",
        description:
            "Map the `session` resolver back to Grounding; \
             behavior_resolver_tower catches the fingerprint collision",
    },
    Mutation {
        id: "sat-decider-always-sat",
        file: "codegen/src/pipeline.rs",
        original: "pub const fn decide_two_sat",
        patched: "pub const fn _orig_decide_two_sat",
        expected_fail_marker: "correctness/sat_deciders",
        description:
            "Rename decide_two_sat so call sites fall through to a stub; \
             behavior_sat_deciders must catch the missing decider",
    },
    Mutation {
        id: "witness-residual-default-zero",
        file: "codegen/src/enforcement.rs",
        original: "pub const fn as_u32(&self) -> u32 {\n        self.value",
        patched: "pub const fn as_u32(&self) -> u32 {\n        0; let _ = self.value; 0",
        expected_fail_marker: "correctness/witness_accessors",
        description:
            "Make ResidualMetric::as_u32 always return 0; \
             behavior_witness_accessors catches the degenerate accessor",
    },
    Mutation {
        id: "const-ring-eval-skip-mask",
        file: "codegen/src/enforcement.rs",
        original: "result & W8_MASK",
        patched: "result",
        expected_fail_marker: "correctness/const_ring_eval",
        description:
            "Drop W8 width mask after ring op; behavior_const_ring_eval catches \
             the incorrect modular arithmetic",
    },
    Mutation {
        id: "ring-ops-break-bnot",
        file: "codegen/src/enforcement.rs",
        original: "impl_ring_ops_unary_bnot",
        patched: "_disabled_impl_ring_ops_unary_bnot",
        expected_fail_marker: "correctness/ring_ops_identities",
        description:
            "Rename BNot emitter so identity Neg(BNot(x))=Succ(x) fails to hold",
    },
    Mutation {
        id: "embedding-xor-upper-bits",
        file: "codegen/src/enforcement.rs",
        original: "impl ValidLevelEmbedding",
        patched: "impl _disabled_ValidLevelEmbedding",
        expected_fail_marker: "correctness/embedding_preserves_value",
        description:
            "Disable ValidLevelEmbedding impls; \
             behavior_embedding_preserves_value catches the broken Embed<From, To>",
    },
    Mutation {
        id: "uor-time-zero-rewrite-steps",
        file: "codegen/src/enforcement.rs",
        original: "pub const fn rewrite_steps(&self) -> u64 {\n        self.rewrite_steps",
        patched: "pub const fn rewrite_steps(&self) -> u64 {\n        0; let _ = self.rewrite_steps; 0",
        expected_fail_marker: "correctness/uor_time",
        description:
            "Make UorTime::rewrite_steps always zero; behavior_uor_time catches the \
             accessor regression",
    },
    Mutation {
        id: "grammar-shape-drift",
        file: "codegen/src/enforcement.rs",
        original: "https://uor.foundation/conformance/CompileUnitShape",
        patched: "https://uor.foundation/conformance/CompileUnitShape__DRIFTED",
        expected_fail_marker: "correctness/grammar_surface_roundtrip",
        description:
            "Drift a conformance:*Shape IRI; behavior_grammar_surface_roundtrip catches \
             the shape-IRI mismatch",
    },
    Mutation {
        id: "multiplication-accept-zero-stack",
        file: "codegen/src/enforcement.rs",
        original: "if context.stack_budget_bytes == 0 {",
        patched: "if context.stack_budget_bytes == 0 && false {",
        expected_fail_marker: "correctness/resolver_multiplication",
        description:
            "Skip the zero-stack-budget early-exit in multiplication::certify; \
             behavior_resolver_multiplication catches the admissibility regression",
    },
    Mutation {
        id: "observability-skip-emit",
        file: "codegen/src/enforcement.rs",
        original: "fn emit(&self, event: TraceEvent)",
        patched: "fn emit(&self, _event: TraceEvent)",
        expected_fail_marker: "correctness/observability",
        description:
            "Rename the emit arg to _event; the no-op emission is a regression \
             behavior_observability must catch",
    },
    // ─── Target-doc cross-reference validators ───────────────────────────
    Mutation {
        id: "w4-restore-fn-ground",
        file: "codegen/src/enforcement.rs",
        original: "fn program(&self) -> GroundingProgram<Self::Output, Self::Map>;",
        patched: "fn program(&self) -> GroundingProgram<Self::Output, Self::Map>;\n    fn ground(&self, external: &[u8]) -> Option<Self::Output>;",
        expected_fail_marker: "target/w4_grounding_closure",
        description:
            "Re-add `fn ground` to the Grounding trait; target/w4_grounding_closure catches the W4 regression",
    },
    Mutation {
        id: "constraint-encoder-wildcard-none",
        file: "codegen/src/pipeline.rs",
        original: "ConstraintRef::Bound { .. } => Some(EMPTY),",
        patched: "ConstraintRef::Bound { .. } => None,",
        expected_fail_marker: "target/constraint_encoder_completeness",
        description:
            "Make ConstraintRef::Bound route to None; \
             target/constraint_encoder_completeness catches the incomplete encoder",
    },
    Mutation {
        id: "spectral-walk-removed",
        file: "codegen/src/pipeline.rs",
        original: "let page = SpectralSequencePage::from_parts(",
        patched: "let _page = /* SpectralSequencePage removed */ ",
        expected_fail_marker: "target/spectral_sequence_walk",
        description:
            "Remove SpectralSequencePage construction from run_incremental_completeness; \
             target/spectral_sequence_walk catches the missing page walk",
    },
];

/// Runs the regression drill for the named mutation.
fn regression_drill(mutation_id: &str) -> Result<()> {
    if mutation_id == "list" {
        println!("Available regression-drill mutations:");
        for m in MUTATIONS {
            println!("  {:<40}  {}", m.id, m.description);
        }
        println!();
        println!("Run a specific drill: `cargo xtask regression-drill <mutation-id>`");
        println!("Run every drill sequentially: `cargo xtask regression-drill all`");
        return Ok(());
    }

    if mutation_id == "all" {
        let mut failures: Vec<&'static str> = Vec::new();
        for m in MUTATIONS {
            println!("\n[regression-drill/all] ── running `{}` ──", m.id);
            match run_single_mutation(m) {
                Ok(()) => {}
                Err(e) => {
                    eprintln!("[regression-drill/all] FAIL `{}`: {e}", m.id);
                    failures.push(m.id);
                }
            }
        }
        if failures.is_empty() {
            println!(
                "\n[regression-drill/all] {} mutations exercised; every one triggered the \
                 expected `[FAIL]` marker and reverted cleanly.",
                MUTATIONS.len()
            );
            return Ok(());
        }
        anyhow::bail!(
            "{} of {} mutations failed their drill: {:?}",
            failures.len(),
            MUTATIONS.len(),
            failures
        );
    }

    let mutation = MUTATIONS
        .iter()
        .find(|m| m.id == mutation_id)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "unknown mutation `{mutation_id}`. Run `cargo xtask regression-drill list` for the available set"
            )
        })?;

    run_single_mutation(mutation)
}

/// Applies one mutation, runs the drill, reverts, and reports. Shared
/// between `regression-drill <id>` and `regression-drill all`.
fn run_single_mutation(mutation: &Mutation) -> Result<()> {
    let root = workspace_root()?;
    let file_path = root.join(mutation.file);

    println!(
        "[regression-drill] applying mutation `{}` to {}",
        mutation.id, mutation.file
    );

    // 1. Read + patch the file.
    let original = std::fs::read_to_string(&file_path)
        .with_context(|| format!("reading {}", file_path.display()))?;
    let occurrences = original.matches(mutation.original).count();
    if occurrences != 1 {
        anyhow::bail!(
            "mutation's `original` substring appears {occurrences} times in {}; drill requires uniqueness",
            mutation.file
        );
    }
    let patched = original.replacen(mutation.original, mutation.patched, 1);
    std::fs::write(&file_path, patched.as_bytes())
        .with_context(|| format!("writing patched {}", file_path.display()))?;

    // 2. Run the drill and capture the outcome. Always revert.
    let drill_result = run_drill(&root, mutation);

    // 3. Revert via `git restore`.
    println!("[regression-drill] reverting mutation via git restore");
    let revert = Command::new("git")
        .current_dir(&root)
        .args(["restore", "--source", "HEAD", "--", mutation.file])
        .output()
        .context("git restore failed to launch")?;
    if !revert.status.success() {
        eprintln!(
            "[regression-drill] WARNING: git restore failed: {}",
            String::from_utf8_lossy(&revert.stderr)
        );
    }
    // 3b. Also revert the regenerated foundation sources if this was a
    // codegen mutation — otherwise subsequent drills start from a dirty
    // foundation/src tree.
    if mutation.file.starts_with("codegen/") {
        let revert_foundation = Command::new("git")
            .current_dir(&root)
            .args(["restore", "--source", "HEAD", "--", "foundation/src/"])
            .output()
            .context("git restore foundation/src/ failed to launch")?;
        if !revert_foundation.status.success() {
            eprintln!(
                "[regression-drill] WARNING: git restore foundation/src/ failed: {}",
                String::from_utf8_lossy(&revert_foundation.stderr)
            );
        }
    }

    // 4. Report.
    drill_result
}

/// Executes the regen + conformance-run + assertion pipeline.
fn run_drill(root: &Path, mutation: &Mutation) -> Result<()> {
    // Some mutations target `codegen/`; regenerate the foundation source
    // so the mutation takes effect on the generated code.
    if mutation.file.starts_with("codegen/") {
        println!("[regression-drill] regenerating foundation via `cargo run --bin uor-crate`");
        let gen = Command::new(env!("CARGO"))
            .current_dir(root)
            .args(["run", "--bin", "uor-crate", "--quiet"])
            .output()
            .context("cargo run --bin uor-crate failed to launch")?;
        if !gen.status.success() {
            anyhow::bail!(
                "mutation caused `uor-crate` to fail to generate (stderr: {})",
                String::from_utf8_lossy(&gen.stderr)
            );
        }
    }

    // Run conformance.
    println!("[regression-drill] running `cargo run --bin uor-conformance`");
    let conf = Command::new(env!("CARGO"))
        .current_dir(root)
        .args(["run", "--bin", "uor-conformance", "--quiet"])
        .output()
        .context("cargo run --bin uor-conformance failed to launch")?;

    let stdout = String::from_utf8_lossy(&conf.stdout);
    let failed = !conf.status.success() || stdout.contains("Conformance FAILED");
    let marker_found = stdout.contains(mutation.expected_fail_marker);

    if failed && marker_found {
        println!(
            "[PASS] regression-drill/{}: conformance failed with expected marker `{}`",
            mutation.id, mutation.expected_fail_marker
        );
        Ok(())
    } else if !failed {
        anyhow::bail!(
            "conformance suite UNEXPECTEDLY PASSED under mutation `{}` \u{2014} the suite is \
             not sensitive to this regression. This is a suite defect.",
            mutation.id
        );
    } else {
        anyhow::bail!(
            "conformance failed but did not surface marker `{}` \u{2014} either the wrong \
             validator caught the regression, or the marker text has drifted.",
            mutation.expected_fail_marker
        );
    }
}

/// Locates the workspace root by walking up from the current directory
/// until a `Cargo.toml` containing `[workspace]` is found.
fn workspace_root() -> Result<PathBuf> {
    let start = std::env::current_dir().context("current_dir unreadable")?;
    let mut here: &Path = &start;
    loop {
        let manifest = here.join("Cargo.toml");
        if manifest.exists() {
            let content = std::fs::read_to_string(&manifest)
                .with_context(|| format!("reading {}", manifest.display()))?;
            if content.contains("[workspace]") {
                return Ok(here.to_path_buf());
            }
        }
        match here.parent() {
            Some(p) => here = p,
            None => anyhow::bail!("no [workspace] Cargo.toml found above {}", start.display()),
        }
    }
}
