//! Query-time beam-merge tests (issue #244).

#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;
use uor_r4_graph_certify::certify::merged_beam_distribution;

#[allow(clippy::type_complexity)] // test-fixture literal shape
fn level(entries: &[(&[u8], &[(u32, u32)])]) -> BTreeMap<Vec<u8>, BTreeMap<u32, u32>> {
    let mut level = BTreeMap::new();
    for (key, dist) in entries {
        let mut d = BTreeMap::new();
        for &(tok, count) in *dist {
            d.insert(tok, count);
        }
        level.insert(key.to_vec(), d);
    }
    level
}

#[test]
fn merges_counts_across_beam_keys() {
    let level = level(&[
        (&[3, 1], &[(10, 5), (11, 2)]),
        (&[3, 2], &[(10, 1), (12, 7)]),
        (&[9, 9], &[(99, 100)]),
    ]);
    let keys = vec![vec![3, 1], vec![3, 2]];
    let merged = merged_beam_distribution(&level, &keys);
    assert_eq!(merged.get(&10), Some(&6)); // 5 + 1 across the beam
    assert_eq!(merged.get(&11), Some(&2));
    assert_eq!(merged.get(&12), Some(&7));
    assert_eq!(merged.get(&99), None); // key outside the beam ignored
}

#[test]
fn missing_keys_contribute_nothing() {
    let level = level(&[(&[1], &[(7, 3)])]);
    let keys = vec![vec![1], vec![2], vec![3]];
    let merged = merged_beam_distribution(&level, &keys);
    assert_eq!(merged.len(), 1);
    assert_eq!(merged.get(&7), Some(&3));
}

#[test]
fn empty_beam_is_empty() {
    let level = level(&[(&[1], &[(7, 3)])]);
    let merged = merged_beam_distribution(&level, &[]);
    assert!(merged.is_empty());
}
