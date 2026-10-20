//! `uor-lean` — Generates the Lean 4 formalization from the ontology.
//!
//! Reads `uor_ontology::Ontology::full()` and writes generated Lean 4 source
//! files to the `lean4/` directory.
//!
//! **Usage:**
//! ```text
//! uor-lean [--out <path>]
//! ```

#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    missing_docs,
    clippy::missing_errors_doc
)]

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

/// Generate the Lean 4 formalization from the UOR ontology.
#[derive(Parser)]
#[command(
    name = "uor-lean",
    about = "Generate the UOR Foundation Lean 4 formalization"
)]
struct Args {
    /// Output directory for generated Lean 4 files.
    #[arg(long, default_value = "lean4")]
    out: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let ontology = uor_ontology::Ontology::full();

    println!(
        "Generating Lean 4 from ontology v{}: {} namespaces, {} classes, {} properties, {} individuals",
        ontology.version,
        ontology.namespaces.len(),
        ontology.class_count(),
        ontology.property_count(),
        ontology.individual_count()
    );

    let report = uor_lean_codegen::generate(ontology, &args.out)?;

    println!(
        "Generated {} structures, {} fields, {} enums, {} individuals ({} unproven)",
        report.structure_count,
        report.field_count,
        report.enum_count,
        report.def_count,
        report.unproven_count
    );
    println!("Files written ({}):", report.files.len());
    for file in &report.files {
        println!("  {file}");
    }

    println!("Generation complete.");
    Ok(())
}
