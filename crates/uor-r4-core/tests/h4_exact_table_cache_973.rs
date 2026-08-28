//! Cold-start concurrency regression for #973's immutable exact-H4 caches.

use std::sync::{Arc, Barrier};

use uor_r4_core::canonical_lexical_ingestion::validate_h4_binary_icosahedral_closure;

const WORKERS: usize = 4;
const ROOT_TABLE_KAPPA: &str =
    "blake3:8d33d62a239fb8001fea2bd14a9a5ec7321d0f07d81c74a5715eaeb3df53aa76";
const MULTIPLICATION_TABLE_KAPPA: &str =
    "blake3:90ee73a27ee2e8ba5bccd1507d7fb37ed1f044b1640772c86752bc0bb2111759";

#[test]
fn concurrent_cold_h4_table_initialization_is_byte_identical() {
    let barrier = Arc::new(Barrier::new(WORKERS));
    let handles = (0..WORKERS)
        .map(|_| {
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                let table = validate_h4_binary_icosahedral_closure().unwrap();
                assert_eq!(table.root_count, 120);
                assert_eq!(table.product_count, 14_400);
                assert_eq!(table.h4_root_table_kappa, ROOT_TABLE_KAPPA);
                assert_eq!(table.multiplication_table_kappa, MULTIPLICATION_TABLE_KAPPA);
                assert!(table.unique_closure_exact);
                assert!(table.identity_exact);
                assert!(table.inverses_exact);
                assert!(table.associativity_exact);
                assert!(table.integer_only_no_rounding);
                serde_json::to_vec(&table).unwrap()
            })
        })
        .collect::<Vec<_>>();
    let results = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect::<Vec<_>>();
    assert!(results.windows(2).all(|pair| pair[0] == pair[1]));
    assert_eq!(
        serde_json::to_vec(&validate_h4_binary_icosahedral_closure().unwrap()).unwrap(),
        results[0]
    );
}
