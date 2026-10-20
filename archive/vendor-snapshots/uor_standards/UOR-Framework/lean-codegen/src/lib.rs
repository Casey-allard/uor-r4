//! Lean 4 code generator for the UOR Foundation ontology.
//!
//! Generates `.lean` files from `Ontology::full()`, producing structures for
//! OWL classes, inductives for enum classes, and constant namespaces for
//! named individuals.

#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    missing_docs,
    clippy::missing_errors_doc
)]

pub mod emit;
pub mod enforcement;
pub mod enums;
pub mod individuals;
pub mod mapping;
pub mod pipeline;
pub mod primitives;
pub mod rigor_patterns;
pub mod structures;

use std::collections::{HashMap, HashSet};
use std::fmt::Write as FmtWrite;
use std::path::Path;

use anyhow::{Context, Result};
use uor_ontology::model::{Class, Ontology, Property, PropertyKind};

use crate::emit::write_file;
use crate::mapping::lean_namespace_mappings;

/// Report of what the generator produced.
pub struct LeanGenerationReport {
    /// Number of `structure` declarations generated.
    pub structure_count: usize,
    /// Number of structure fields generated.
    pub field_count: usize,
    /// Number of `inductive` + struct enum types generated.
    pub enum_count: usize,
    /// Number of individual typed `def`s generated.
    pub def_count: usize,
    /// Number of individuals that could not be fully proven from their
    /// ontology assertions (non-Option/non-Array fields missing an
    /// assertion, IRI resolution failures, type mismatches, cyclic
    /// self-references, or blocked-type fields without assertion).
    /// Mirrored into `lean4/.uor-unproven.json` for conformance.
    pub unproven_count: usize,
    /// Absolute paths of files written.
    pub files: Vec<String>,
}

/// Generates the complete Lean 4 formalization from the ontology.
///
/// Writes all `.lean` files to `out_dir/` and returns a generation report.
///
/// # Errors
///
/// Returns an error if any file cannot be written.
pub fn generate(ontology: &Ontology, out_dir: &Path) -> Result<LeanGenerationReport> {
    clean_out_dir(out_dir)?;

    let ns_map = lean_namespace_mappings();
    let mut files = Vec::new();

    // Build cross-namespace maps
    let all_props_by_domain = build_all_props_by_domain(ontology);
    let all_classes_by_iri = build_all_classes_by_iri(ontology);
    let all_individuals_by_iri = build_all_individuals_by_iri(ontology);
    let inhabited_blocked = structures::compute_inhabited_blocked_for_ontology(
        ontology,
        &ns_map,
        &all_props_by_domain,
        &all_classes_by_iri,
    );

    // 1. Generate Primitives
    let primitives_content = primitives::generate_primitives();
    let primitives_path = out_dir.join("UOR").join("Primitives.lean");
    write_file(&primitives_path, &primitives_content)?;
    files.push(primitives_path.display().to_string());

    // 2. Generate Enums
    let mut enums_content = enums::generate_enums(ontology);
    let op_methods = enums::generate_primitive_op_methods(ontology);
    if !op_methods.is_empty() {
        enums_content.push('\n');
        enums_content.push_str(&op_methods);
        enums_content.push('\n');
    }
    let enums_path = out_dir.join("UOR").join("Enums.lean");
    write_file(&enums_path, &enums_content)?;
    files.push(enums_path.display().to_string());
    let enum_count = enums::count_enums(ontology);

    // 3. Generate the combined structures file (single compilation unit).
    let (structures_content, total_structures, total_fields) = structures::generate_all_structures(
        ontology,
        &ns_map,
        &all_props_by_domain,
        &all_classes_by_iri,
    );
    let structures_path = out_dir.join("UOR").join("Structures.lean");
    write_file(&structures_path, &structures_content)?;
    files.push(structures_path.display().to_string());

    // 4. Generate the individuals files. Per-module split is required
    //    because Lean's kernel hits a stack overflow if all ~3000 nested
    //    individual namespaces are placed in a single file. An
    //    aggregator `UOR/Individuals.lean` re-imports every per-module
    //    file so downstream consumers only need `import UOR`.
    let skip_types: HashSet<&str> = enums::enum_individual_types().into_iter().collect();
    let (individuals_agg_content, per_module_files, total_defs, unproven_manifest) =
        individuals::generate_all_individuals(
            ontology,
            &ns_map,
            &all_individuals_by_iri,
            &all_classes_by_iri,
            &all_props_by_domain,
            &inhabited_blocked,
            &skip_types,
        );
    for entry in &per_module_files {
        let file_path = out_dir.join(&entry.rel_path);
        write_file(&file_path, &entry.content)?;
        files.push(file_path.display().to_string());
    }
    let individuals_path = out_dir.join("UOR").join("Individuals.lean");
    write_file(&individuals_path, &individuals_agg_content)?;
    files.push(individuals_path.display().to_string());

    // 4a. Write the unproven-individuals manifest. Always emitted
    //     (even if empty) so the `lean4/individual_proof` conformance
    //     check has a predictable file to read.
    let manifest_path = out_dir.join(".uor-unproven.json");
    write_file(&manifest_path, &unproven_manifest.to_pretty_json())?;
    files.push(manifest_path.display().to_string());
    let unproven_count = unproven_manifest.unproven_individual_count();

    // 4b. Generate Enforcement.lean (v0.2.1 ergonomics surface)
    let enforcement_content = enforcement::generate_enforcement(ontology);
    let enforcement_path = out_dir.join("UOR").join("Enforcement.lean");
    write_file(&enforcement_path, &enforcement_content)?;
    files.push(enforcement_path.display().to_string());

    // 4c. Generate Pipeline.lean (v0.2.1 reduction pipeline driver)
    let pipeline_content = pipeline::generate_pipeline(ontology);
    let pipeline_path = out_dir.join("UOR").join("Pipeline.lean");
    write_file(&pipeline_path, &pipeline_content)?;
    files.push(pipeline_path.display().to_string());

    // 4d. Generate Examples.lean (v0.2.1 worked examples)
    let examples_content = enforcement::generate_examples(ontology);
    let examples_path = out_dir.join("UOR").join("Examples.lean");
    write_file(&examples_path, &examples_content)?;
    files.push(examples_path.display().to_string());

    // 4e. Generate Test.lean (v0.2.1 #guard assertions)
    let test_content = enforcement::generate_test(ontology);
    let test_path = out_dir.join("UOR").join("Test.lean");
    write_file(&test_path, &test_content)?;
    files.push(test_path.display().to_string());

    // 4f. Generate Prelude.lean (v0.2.1 re-exports)
    let prelude_content = enforcement::generate_prelude(ontology);
    let prelude_path = out_dir.join("UOR").join("Prelude.lean");
    write_file(&prelude_path, &prelude_content)?;
    files.push(prelude_path.display().to_string());

    // 5. Generate root UOR.lean
    let root_content = generate_root_import();
    let root_path = out_dir.join("UOR.lean");
    write_file(&root_path, &root_content)?;
    files.push(root_path.display().to_string());

    // 6. Generate LICENSE (required for Lean Reservoir indexing)
    let license_content = include_str!("../../LICENSE");
    let license_path = out_dir.join("LICENSE");
    write_file(&license_path, license_content)?;
    files.push(license_path.display().to_string());

    // 7. Generate README.md
    let readme = format!(
        "# UOR Foundation \u{2014} Lean 4 Formalization\n\n\
         Machine-generated Lean 4 structures, enums, and constants for the\n\
         [UOR Foundation](https://uor.foundation/) ontology (v{}).\n\n\
         **Do not edit manually** \u{2014} regenerated by \
         [UOR-Framework](https://github.com/UOR-Foundation/UOR-Framework).\n\n\
         ## Usage\n\n\
         Add to your `lakefile.lean`:\n\n\
         ```lean\n\
         require uor from git\n\
         \x20 \"https://github.com/UOR-Foundation/UOR-Framework\"\n\
         ```\n\n\
         Then `import UOR` in your Lean files.\n",
        ontology.version
    );
    let readme_path = out_dir.join("README.md");
    write_file(&readme_path, &readme)?;
    files.push(readme_path.display().to_string());

    Ok(LeanGenerationReport {
        structure_count: total_structures,
        field_count: total_fields,
        enum_count,
        def_count: total_defs,
        unproven_count,
        files,
    })
}

/// Removes the `UOR/` subtree of `out_dir`. Refuses to run unless
/// a `lakefile.lean` is present in `out_dir` or its parent directory,
/// as a hard safety guard against mis-pointed `--out` arguments.
fn clean_out_dir(out_dir: &Path) -> Result<()> {
    let has_lakefile_here = out_dir.join("lakefile.lean").exists();
    let has_lakefile_parent = out_dir
        .parent()
        .map(|p| p.join("lakefile.lean").exists())
        .unwrap_or(false);
    if !has_lakefile_here && !has_lakefile_parent {
        anyhow::bail!(
            "Refusing to clean {}: no lakefile.lean found in this directory or its parent \
             (not a Lean project root).",
            out_dir.display()
        );
    }
    let uor_dir = out_dir.join("UOR");
    if uor_dir.exists() {
        std::fs::remove_dir_all(&uor_dir)
            .with_context(|| format!("Failed to remove {}", uor_dir.display()))?;
    }
    Ok(())
}

/// Builds the cross-namespace property-by-domain map.
pub fn build_all_props_by_domain(ontology: &Ontology) -> HashMap<&str, Vec<&Property>> {
    let mut map: HashMap<&str, Vec<&Property>> = HashMap::new();
    for module in &ontology.namespaces {
        for prop in &module.properties {
            if let Some(domain) = prop.domain {
                if prop.kind != PropertyKind::Annotation {
                    map.entry(domain).or_default().push(prop);
                }
            }
        }
    }
    map
}

/// Builds a map from class IRI to `Class` struct for transitive inheritance lookup.
pub fn build_all_classes_by_iri(ontology: &Ontology) -> HashMap<&str, &Class> {
    let mut map = HashMap::new();
    for module in &ontology.namespaces {
        for class in &module.classes {
            map.insert(class.id, class);
        }
    }
    map
}

/// Builds a map from individual IRI to the individual plus its owning
/// `NamespaceModule`. The owning module is required when resolving an
/// individual reference to its Lean path (`UOR.<Space>.<Module>.<name>`),
/// because the Lean module name comes from the containing namespace's
/// mapping entry.
pub fn build_all_individuals_by_iri(
    ontology: &Ontology,
) -> HashMap<
    &str,
    (
        &uor_ontology::model::Individual,
        &uor_ontology::model::NamespaceModule,
    ),
> {
    let mut map = HashMap::new();
    for module in &ontology.namespaces {
        for ind in &module.individuals {
            map.insert(ind.id, (ind, module));
        }
    }
    map
}

/// Generates the root `UOR.lean` import file.
fn generate_root_import() -> String {
    let mut buf = String::new();
    let _ = writeln!(
        buf,
        "-- @generated by uor-lean from uor-ontology \u{2014} do not edit manually"
    );
    let _ = writeln!(buf, "--");
    let _ = writeln!(buf, "-- UOR Foundation \u{2014} Lean 4 formalization root.");
    buf.push('\n');
    buf.push_str("import UOR.Primitives\n");
    buf.push_str("import UOR.Enums\n");
    buf.push_str("import UOR.Structures\n");
    buf.push_str("import UOR.Individuals\n");
    buf.push_str("import UOR.Enforcement\n");
    buf.push_str("import UOR.Pipeline\n");
    buf.push_str("import UOR.Examples\n");
    buf.push_str("import UOR.Test\n");
    buf.push_str("import UOR.Prelude\n");
    buf
}
