//! v0.2.1 example: walk the InhabitanceDispatchTable rules.
//!
//! Demonstrates the parametric dispatch table emission. The table's content
//! comes directly from the ontology's `predicate:InhabitanceDispatchTable`
//! individual and its three `predicate:DispatchRule` members.
//!
//! Run with: `cargo run --example dispatch_table_walk -p uor-foundation`

use uor_foundation::enforcement::INHABITANCE_DISPATCH_TABLE;

fn main() {
    println!(
        "InhabitanceDispatchTable rules ({} total):",
        INHABITANCE_DISPATCH_TABLE.len()
    );
    for (i, rule) in INHABITANCE_DISPATCH_TABLE.iter().enumerate() {
        println!(
            "  [{i}] priority={} predicate={} target={}",
            rule.priority, rule.predicate_iri, rule.target_resolver_iri
        );
    }
}
