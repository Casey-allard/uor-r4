//! `uor-crate` — Generates the `uor-foundation` Rust library crate from the ontology.
//!
//! Reads `uor_ontology::Ontology::full()` and writes generated Rust source files
//! to the `foundation/src/` directory. Also emits the companion
//! `uor-foundation-sdk` proc-macro crate source to a sibling directory —
//! derived by default from `--out`, or overridden with `--sdk-out`.
//!
//! **Usage:**
//! ```
//! uor-crate [--out <path>] [--sdk-out <path>]
//! ```

#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    missing_docs,
    clippy::missing_errors_doc
)]

use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Parser;

/// Generate the uor-foundation Rust library crate from the ontology.
#[derive(Parser)]
#[command(
    name = "uor-crate",
    about = "Generate the uor-foundation Rust trait crate"
)]
struct Args {
    /// Output directory for generated foundation source files.
    #[arg(long, default_value = "foundation/src")]
    out: PathBuf,

    /// Output directory for the `uor-foundation-sdk` proc-macro crate
    /// source files. Defaults to `<out.parent()>/uor-foundation-sdk/src`
    /// so the two emitted crates sit as siblings under the workspace.
    #[arg(long)]
    sdk_out: Option<PathBuf>,
}

fn default_sdk_out(foundation_out: &Path) -> PathBuf {
    // If foundation_out is `foundation/src`, parent is `foundation/`, grandparent
    // is the workspace root. Prefer the workspace root's `uor-foundation-sdk/src`
    // so the SDK emits as a workspace-member sibling of foundation. Fall back
    // to the current directory only if the foundation_out has no grandparent
    // (a degenerate case, e.g. `--out /` — clippy's unwrap_used denial forbids
    // panicking here).
    let foundation_crate = foundation_out.parent().unwrap_or(Path::new("."));
    let workspace_root = foundation_crate.parent().unwrap_or(Path::new("."));
    workspace_root.join("uor-foundation-sdk").join("src")
}

fn main() -> Result<()> {
    let args = Args::parse();
    let ontology = uor_ontology::Ontology::full();

    println!(
        "Generating uor-foundation from ontology v{}: {} namespaces, {} classes, {} properties, {} individuals",
        ontology.version,
        ontology.namespaces.len(),
        ontology.class_count(),
        ontology.property_count(),
        ontology.individual_count()
    );

    let sdk_out = args.sdk_out.unwrap_or_else(|| default_sdk_out(&args.out));

    let report = uor_codegen::generate(ontology, &args.out, &sdk_out)?;

    println!(
        "Generated {} traits, {} methods, {} enums, {} constants",
        report.trait_count, report.method_count, report.enum_count, report.const_count
    );
    println!("Files written ({}):", report.files.len());
    for file in &report.files {
        println!("  {}", file);
    }

    println!("Generation complete.");
    Ok(())
}
