//! `uor-build` — Assembles the UOR Foundation ontology from the `uor-ontology` library
//! and writes the artifacts to the output directory.
//!
//! **Outputs:**
//! - `<out>/uor.foundation.jsonld` — JSON-LD 1.1
//! - `<out>/uor.foundation.ttl` — Turtle 1.1
//! - `<out>/uor.foundation.nt` — N-Triples
//! - `<out>/uor.foundation.owl` — OWL 2 RDF/XML
//! - `<out>/uor.foundation.schema.json` — JSON Schema (Draft 2020-12)
//! - `<out>/uor.shapes.ttl` — SHACL validation shapes
//! - `<out>/uor.term.ebnf` — EBNF grammar (Amendment 42)
//!
//! **Usage:**
//! ```
//! uor-build [--out <path>]
//! ```

#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    missing_docs,
    clippy::missing_errors_doc
)]

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use uor_ontology::serializer::{
    conformance_ebnf, ebnf, json_schema, jsonld, ntriples, owl_xml, shacl, turtle,
};
use uor_ontology::Ontology;

/// Build the UOR Foundation ontology artifacts.
#[derive(Parser)]
#[command(name = "uor-build", about = "Build UOR Foundation ontology artifacts")]
struct Args {
    /// Output directory for generated artifacts.
    #[arg(long, default_value = "public")]
    out: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let out = &args.out;

    fs::create_dir_all(out)
        .with_context(|| format!("Failed to create output directory: {}", out.display()))?;

    let ontology = Ontology::full();

    // Print summary
    println!(
        "UOR Foundation ontology v{}: {} namespaces, {} classes, {} properties, {} individuals",
        ontology.version,
        ontology.namespaces.len(),
        ontology.class_count(),
        ontology.property_count(),
        ontology.individual_count()
    );

    // JSON-LD
    let json_path = out.join("uor.foundation.jsonld");
    let json_value = jsonld::to_json_ld(ontology);
    let json_str = serde_json::to_string_pretty(&json_value)
        .context("Failed to serialize ontology to JSON-LD")?;
    fs::write(&json_path, &json_str)
        .with_context(|| format!("Failed to write {}", json_path.display()))?;
    println!("  Written: {}", json_path.display());

    // Turtle
    let ttl_path = out.join("uor.foundation.ttl");
    let ttl_str = turtle::to_turtle(ontology);
    fs::write(&ttl_path, &ttl_str)
        .with_context(|| format!("Failed to write {}", ttl_path.display()))?;
    println!("  Written: {}", ttl_path.display());

    // N-Triples
    let nt_path = out.join("uor.foundation.nt");
    let nt_str = ntriples::to_ntriples(ontology);
    fs::write(&nt_path, &nt_str)
        .with_context(|| format!("Failed to write {}", nt_path.display()))?;
    println!("  Written: {}", nt_path.display());

    // EBNF grammar (Amendment 42)
    let ebnf_path = out.join("uor.term.ebnf");
    let ebnf_str = ebnf::to_ebnf(ontology);
    fs::write(&ebnf_path, &ebnf_str)
        .with_context(|| format!("Failed to write {}", ebnf_path.display()))?;
    println!("  Written: {}", ebnf_path.display());

    // v0.2.1: Conformance declaration grammar (parametric from
    // conformance:Shape + PropertyConstraint surface metadata).
    let conformance_ebnf_path = out.join("uor.conformance.ebnf");
    let conformance_ebnf_str = conformance_ebnf::to_conformance_ebnf(ontology);
    fs::write(&conformance_ebnf_path, &conformance_ebnf_str)
        .with_context(|| format!("Failed to write {}", conformance_ebnf_path.display()))?;
    println!("  Written: {}", conformance_ebnf_path.display());

    // OWL RDF/XML
    let owl_path = out.join("uor.foundation.owl");
    let owl_str = owl_xml::to_owl_xml(ontology);
    fs::write(&owl_path, &owl_str)
        .with_context(|| format!("Failed to write {}", owl_path.display()))?;
    println!("  Written: {}", owl_path.display());

    // JSON Schema
    let schema_path = out.join("uor.foundation.schema.json");
    let schema_value = json_schema::to_json_schema(ontology);
    let schema_str = serde_json::to_string_pretty(&schema_value)
        .context("Failed to serialize ontology to JSON Schema")?;
    fs::write(&schema_path, &schema_str)
        .with_context(|| format!("Failed to write {}", schema_path.display()))?;
    println!("  Written: {}", schema_path.display());

    // SHACL Shapes
    let shacl_path = out.join("uor.shapes.ttl");
    let shacl_str = shacl::to_shacl(ontology);
    fs::write(&shacl_path, &shacl_str)
        .with_context(|| format!("Failed to write {}", shacl_path.display()))?;
    println!("  Written: {}", shacl_path.display());

    println!("Build complete.");
    Ok(())
}
