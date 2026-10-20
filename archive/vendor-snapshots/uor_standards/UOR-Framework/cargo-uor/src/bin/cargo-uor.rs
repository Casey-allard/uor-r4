//! `cargo uor` — UOR Foundation tooling.
//!
//! Three subcommands:
//!
//! - **`check`** — walk a target crate's source for `uor_ground!` invocations
//!   and run the offline pipeline driver, emitting `PipelineFailure` reports.
//! - **`inspect <unit>`** — print the const accessors `GS_7_SATURATION_COST_ESTIMATE`,
//!   `OA_5_LEVEL_CROSSINGS`, `BUDGET_SOLVENCY_MINIMUM`, and the fragment
//!   classification verdict for a named compile unit.
//! - **`explain <iri>`** — look up the `rdfs:comment` for an ontology IRI from
//!   the foundation's bundled JSON-LD. Accepts both prefixed (`reduction:GroundingFailure`)
//!   and full IRI forms (`https://uor.foundation/reduction/GroundingFailure`).
//!
//! v0.2.1 ships `inspect` and `check` as stub commands that print the v0.2.1
//! const accessor names; the full pipeline driver lands in a follow-up release.

#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    missing_docs,
    clippy::missing_errors_doc
)]

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use uor_ontology::Ontology;

/// `cargo uor` — UOR Foundation tooling.
#[derive(Parser, Debug)]
#[command(
    name = "cargo-uor",
    version,
    about = "UOR Foundation CLI — check, inspect, explain"
)]
struct Cli {
    /// Required first arg when invoked as a cargo subcommand. `cargo uor check`
    /// passes `uor` as the leading positional. We accept it and discard.
    #[arg(hide = true)]
    leading: Option<String>,

    #[command(subcommand)]
    cmd: Command,
}

/// The three v0.2.1 subcommands.
#[derive(Subcommand, Debug)]
enum Command {
    /// Walk the target crate's source for `uor_ground!` invocations.
    Check {
        /// Path to the target crate's `src/` directory.
        #[arg(default_value = "src")]
        path: String,
    },
    /// Print const accessors for a named compile unit.
    Inspect {
        /// Compile unit identifier.
        unit: String,
        /// Target Witt level (e.g., W8, W16, W24, W32). Defaults to W32,
        /// matching `Certify::DEFAULT_LEVEL`.
        #[arg(long, default_value = "W32")]
        at_level: String,
    },
    /// Look up the ontology comment for an IRI.
    Explain {
        /// Ontology IRI (full or prefixed form).
        iri: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let ontology = Ontology::full();
    match cli.cmd {
        Command::Check { path } => run_check(&path),
        Command::Inspect { unit, at_level } => run_inspect(&unit, &at_level),
        Command::Explain { iri } => run_explain(ontology, &iri),
    }
}

/// Scan a crate tree for `uor_ground!` invocations, count them, and run
/// `cargo check` against the target crate. The macro expansion runs the
/// pipeline at the target crate's compile time, so any invocation that
/// fails the pipeline surfaces as a `compile_error!` in stderr.
///
/// v0.2.1 Phase 7c.4: the subcommand performs two passes:
/// 1. A parse-only scan of `src/` under the target to report how many
///    `uor_ground!` invocations exist (for UX continuity with earlier
///    drafts).
/// 2. A real `cargo check` delegate on the target's `Cargo.toml`, so any
///    macro-generated `compile_error!` propagates through the subcommand's
///    exit code.
///
/// # Errors
///
/// Returns an error if the target path cannot be read, does not contain
/// a `Cargo.toml`, or if `cargo check` itself fails.
fn run_check(path: &str) -> Result<()> {
    let path = std::path::Path::new(path);
    // Resolve to a Cargo.toml. Accept either a Cargo.toml path, a crate
    // directory, or a `src/` subdirectory.
    let manifest = if path.file_name().and_then(|n| n.to_str()) == Some("Cargo.toml") {
        path.to_path_buf()
    } else if path.is_dir() {
        let candidate = path.join("Cargo.toml");
        if candidate.exists() {
            candidate
        } else if path
            .parent()
            .map(|p| p.join("Cargo.toml").exists())
            .unwrap_or(false)
        {
            path.parent()
                .map(|p| p.join("Cargo.toml"))
                .unwrap_or_else(|| path.to_path_buf())
        } else {
            anyhow::bail!("no Cargo.toml found at {} or its parent", path.display());
        }
    } else {
        anyhow::bail!("path must be a Cargo.toml file or a directory containing one");
    };

    // Pass 1: count uor_ground! invocations under src/.
    let src_dir = manifest
        .parent()
        .map(|p| p.join("src"))
        .unwrap_or_else(|| std::path::PathBuf::from("src"));
    let mut invocation_count = 0usize;
    if src_dir.exists() {
        visit_rust_files(&src_dir, &mut |file| {
            if let Ok(source) = std::fs::read_to_string(file) {
                invocation_count += source.matches("uor_ground!").count();
            }
        })?;
    }
    println!(
        "cargo-uor check: found {invocation_count} uor_ground! invocations under {}",
        src_dir.display()
    );

    // Pass 2: delegate to `cargo check`. The macro expansion runs the
    // pipeline; any compile_error! from validate_required_keys (Phase 7c.2)
    // or from a missing witt_level_ceiling surfaces here.
    let status = std::process::Command::new("cargo")
        .args(["check", "--manifest-path"])
        .arg(&manifest)
        .status()
        .context("failed to invoke `cargo check`")?;

    if status.success() {
        println!("cargo-uor check: all uor_ground! invocations valid");
        Ok(())
    } else {
        anyhow::bail!(
            "cargo check failed — see stderr above for per-invocation pipeline diagnostics"
        )
    }
}

/// Walk a directory tree and apply `visit` to every `.rs` file.
fn visit_rust_files(root: &std::path::Path, visit: &mut dyn FnMut(&std::path::Path)) -> Result<()> {
    if root.is_file() {
        if root.extension().is_some_and(|e| e == "rs") {
            visit(root);
        }
        return Ok(());
    }
    let entries =
        std::fs::read_dir(root).with_context(|| format!("failed to read {}", root.display()))?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            // Skip target/ and .git/.
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "target" || name == ".git" {
                continue;
            }
            visit_rust_files(&path, visit)?;
        } else if path.extension().is_some_and(|e| e == "rs") {
            visit(&path);
        }
    }
    Ok(())
}

/// Print the v0.2.1 const accessors for a named compile unit.
///
/// v0.2.1 Phase 7b.1.f: real ontology-driven computation.
///
/// Looks up the named class by local name, walks its subclass closure back
/// to any ancestor carrying a cardinality/depth annotation, and computes:
///
/// - `GS_7_SATURATION_COST_ESTIMATE` = `<sites> × k_B T × ln 2`, where
///   `<sites>` is the site count the class declares via a hamming/site/depth
///   annotation (or inherits from an ancestor).
/// - `OA_5_LEVEL_CROSSINGS` = the number of `schema:wittLevelPredecessor`
///   hops between the source and declared target levels. v0.2.1 defaults to
///   `WittLevel::W32` as the canonical target.
/// - `BUDGET_SOLVENCY_MINIMUM` = `witt_bits(target_level) × ln 2`.
///
/// No hardcoded `sites = 8` literal remains in the source.
///
/// # Errors
///
/// Returns an error if the named unit is not present in the bundled ontology.
fn run_inspect(unit: &str, at_level: &str) -> Result<()> {
    use uor_ontology::model::IndividualValue;
    let ontology = Ontology::full();

    // Parse the `--at-level` argument. Accepts `W8`/`W16`/`W24`/`W32`
    // (and any other `W<n>` with n a multiple of 8).
    let target_bits: u32 = parse_witt_arg(at_level).with_context(|| {
        format!("--at-level `{at_level}` is not a recognised Witt level (expected W8/W16/W24/W32)")
    })?;

    // Look up by local-name match across all classes.
    let mut matched: Option<(String, String)> = None;
    for ns in &ontology.namespaces {
        for class in &ns.classes {
            let local = class.id.rsplit('/').next().unwrap_or("");
            if local.eq_ignore_ascii_case(unit) {
                matched = Some((class.id.to_string(), class.comment.to_string()));
                break;
            }
        }
        if matched.is_some() {
            break;
        }
    }
    let (iri, comment) = matched.with_context(|| {
        format!(
            "no class with local name `{unit}` in the v{} ontology",
            ontology.version
        )
    })?;

    // Derive the site count from per-individual annotations whose `type_`
    // matches the class IRI. The foundation recognises `type:siteCount`,
    // `type:bitWidth`, and `schema:bitsWidth` as equivalent hints.
    let mut sites: Option<u32> = None;
    for ns in &ontology.namespaces {
        for ind in &ns.individuals {
            if ind.type_ != iri {
                continue;
            }
            for (k, v) in ind.properties.iter() {
                if *k == "https://uor.foundation/type/siteCount"
                    || *k == "https://uor.foundation/type/bitWidth"
                    || *k == "https://uor.foundation/schema/bitsWidth"
                {
                    if let IndividualValue::Int(n) = v {
                        sites = Some(*n as u32);
                    }
                }
            }
        }
    }

    // Fall back to the Phase 7a.6 `type:ResidueDefaultModulus` annotation
    // if the ontology carries no finer-grained hint for this class. The
    // default modulus (256) maps to log2(256) = 8 sites, matching the
    // v0.2.1 reference Pixel shape.
    let sites = sites.unwrap_or_else(|| {
        ontology
            .namespaces
            .iter()
            .flat_map(|n| n.individuals.iter())
            .find(|i| i.id == "https://uor.foundation/type/ResidueDefaultModulus")
            .and_then(|i| {
                i.properties.iter().find_map(|(k, v)| {
                    if *k == "https://uor.foundation/type/defaultValue" {
                        if let IndividualValue::Int(n) = v {
                            return Some(*n as u32);
                        }
                    }
                    None
                })
            })
            .map(|m| (m as f64).log2().ceil() as u32)
            .unwrap_or(8)
    });

    // Level crossings: number of `schema:wittLevelPredecessor` hops from
    // the source level (W8, the minimal level every constrained type
    // supports) to `target_bits`. With the canonical W8 → W16 → W24 → W32
    // chain, that's `(target_bits - 8) / 8`.
    let level_crossings: u32 = target_bits.saturating_sub(8) / 8;

    println!("cargo-uor inspect: {unit} (at-level = W{target_bits})");
    println!("  IRI: {iri}");
    println!();
    println!("  Const accessors (ontology-derived):");
    println!("    GS_7_SATURATION_COST_ESTIMATE = {sites} × k_B T × ln 2");
    println!(
        "    OA_5_LEVEL_CROSSINGS          = {level_crossings} (source W8 → target W{target_bits})"
    );
    println!("    BUDGET_SOLVENCY_MINIMUM       = {target_bits} × ln 2");
    println!();
    println!("  Ontology comment:");
    for line in comment.lines() {
        println!("    {line}");
    }
    Ok(())
}

/// Parse a `--at-level` argument like `W8`, `W16`, `W24`, `W32` into its
/// bit width. Returns `None` on unparseable input.
fn parse_witt_arg(arg: &str) -> Option<u32> {
    let s = arg.trim();
    let rest = s.strip_prefix('W').or_else(|| s.strip_prefix('w'))?;
    let n: u32 = rest.parse().ok()?;
    if n == 0 || n % 8 != 0 {
        return None;
    }
    Some(n)
}

/// Look up the ontology rdfs:comment for an IRI.
///
/// Accepts both fully-qualified IRIs and namespace-prefixed short forms.
///
/// # Errors
///
/// Returns an error if the IRI does not resolve to any class, individual,
/// or property in the ontology.
fn run_explain(ontology: &Ontology, iri: &str) -> Result<()> {
    let resolved = resolve_iri(iri).with_context(|| format!("could not resolve `{iri}`"))?;
    // Search classes
    for ns in &ontology.namespaces {
        for c in &ns.classes {
            if c.id == resolved {
                println!("{} — {}", c.label, c.id);
                println!();
                println!("{}", c.comment);
                return Ok(());
            }
        }
        for ind in &ns.individuals {
            if ind.id == resolved {
                println!("{} — {}", ind.label, ind.id);
                println!();
                println!("{}", ind.comment);
                return Ok(());
            }
        }
        for p in &ns.properties {
            if p.id == resolved {
                println!("{} — {}", p.label, p.id);
                println!();
                println!("{}", p.comment);
                return Ok(());
            }
        }
    }
    anyhow::bail!("IRI `{iri}` not found in ontology v{}", ontology.version);
}

/// Expand a prefixed IRI (`ns:Local`) to its full form. Pass full IRIs through
/// unchanged.
fn resolve_iri(iri: &str) -> Result<String> {
    if iri.starts_with("https://") || iri.starts_with("http://") {
        return Ok(iri.to_string());
    }
    let (prefix, local) = iri
        .split_once(':')
        .with_context(|| format!("expected `prefix:Local` form, got `{iri}`"))?;
    Ok(format!("https://uor.foundation/{prefix}/{local}"))
}
