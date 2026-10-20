//! Demonstrates loading the full UOR ontology and serializing it.
//!
//! Run with: `cargo run --example dump_ontology -p uor-ontology`

fn main() {
    let ontology = uor_ontology::Ontology::full();

    println!("UOR Foundation Ontology v{}", ontology.version);
    println!("  Namespaces:   {}", ontology.namespaces.len());
    println!("  Classes:      {}", ontology.class_count());
    println!("  Properties:   {}", ontology.property_count());
    println!("  Individuals:  {}", ontology.individual_count());
    println!();

    // List all namespaces with their space classification.
    for module in &ontology.namespaces {
        let ns = &module.namespace;
        println!(
            "  {:12} {:50} {:>2} classes, {:>2} properties, {:>2} individuals  [{}]",
            ns.prefix,
            ns.iri,
            module.classes.len(),
            module.properties.len(),
            module.individuals.len(),
            ns.space.as_str(),
        );
    }

    println!();

    // Serialize to JSON-LD (show first 200 chars).
    let json_ld = uor_ontology::serializer::jsonld::to_json_ld(ontology);
    let json_str =
        serde_json::to_string_pretty(&json_ld).unwrap_or_else(|e| format!("JSON error: {e}"));
    println!("JSON-LD output ({} bytes):", json_str.len());
    let preview_end = json_str
        .char_indices()
        .nth(200)
        .map_or(json_str.len(), |(i, _)| i);
    println!("{}...", &json_str[..preview_end]);
}
