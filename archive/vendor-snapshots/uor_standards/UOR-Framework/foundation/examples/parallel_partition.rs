//! v0.2.2 Phase Q.3 example: run a parallel product over a 3-component site partition.
//!
//! `ParallelDeclaration::new_with_partition(partition, witness_iri)` declares
//! that `partition.len()` sites split into disjoint components (given by
//! `partition[i]` = component-id-for-site-i). `run_parallel` walks the
//! partition, folds per-component signatures into the fingerprint, and emits
//! a `Grounded<'static, T>` whose unit_address content-addresses the full partition.
//!
//! Run with: `cargo run --example parallel_partition -p uor-foundation`

use uor_foundation::enforcement::{ConstrainedTypeInput, Grounded, Validated};
use uor_foundation::pipeline::{run_parallel, ParallelDeclaration};
use uor_foundation_test_helpers::{validated_runtime, Fnv1aHasher16, REFERENCE_INLINE_BYTES as N};

const DISJOINTNESS_WITNESS: &str = "https://uor.foundation/parallel/ParallelDisjointnessWitness";

fn main() {
    // 9-site partition: 3 components of 3 sites each.
    static PARTITION_ABC: &[u32] = &[0, 0, 0, 1, 1, 1, 2, 2, 2];
    // Alternate partition: the same 9 sites split 4-3-2.
    static PARTITION_432: &[u32] = &[0, 0, 0, 0, 1, 1, 1, 2, 2];

    let unit_abc: Validated<ParallelDeclaration> =
        validated_runtime(ParallelDeclaration::new_with_partition::<
            ConstrainedTypeInput,
        >(PARTITION_ABC, DISJOINTNESS_WITNESS));
    let unit_432: Validated<ParallelDeclaration> =
        validated_runtime(ParallelDeclaration::new_with_partition::<
            ConstrainedTypeInput,
        >(PARTITION_432, DISJOINTNESS_WITNESS));

    let g_abc: Grounded<'static, ConstrainedTypeInput, N> =
        run_parallel::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(unit_abc)
            .expect("3-3-3 admits");
    let g_432: Grounded<'static, ConstrainedTypeInput, N> =
        run_parallel::<ConstrainedTypeInput, _, Fnv1aHasher16, N, 32>(unit_432)
            .expect("4-3-2 admits");

    // Both partitions have site_count = 9 but differ in component structure —
    // the per-component fold produces distinct fingerprints.
    assert_ne!(
        g_abc.content_fingerprint(),
        g_432.content_fingerprint(),
        "distinct partitions must yield distinct fingerprints"
    );

    println!("3-3-3 partition: unit_address={:?}", g_abc.unit_address());
    println!("4-3-2 partition: unit_address={:?}", g_432.unit_address());
    println!("Fingerprints differ → per-component fold is live.");
}
