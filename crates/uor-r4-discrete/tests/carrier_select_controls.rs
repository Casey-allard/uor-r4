//! Finite CUDA controls for `discrete_carrier_select` (`carrier-credit-hard-selection`,
//! #2819, claude-103; prep sheet §6 check 1). Each case hands the selector
//! literal rows, laid out as `discrete_evaluate` writes them, and states the
//! expected two words. They run on a GPU (`--features cuda`).
#![cfg(feature = "cuda")]

use uor_r4_discrete::carrier::{schedule_key, splitmix64, CarrierMode, CarrierSelection};
use uor_r4_discrete::device::Device;
use uor_r4_discrete::objective::ROW_WORDS;

const VOCABULARY: u32 = 16;
const ABSENT: u32 = u32::MAX;

/// One row: target, token hinge, address hinge, top-1 correct, top-ranked ID.
struct Row(u32, u64, u64, bool, u32);

fn select(rows: &[Row], mode: CarrierMode, key: u64) -> uor_r4_discrete::Result<CarrierSelection> {
    select_in(VOCABULARY, rows, mode, key)
}

fn select_in(
    vocabulary: u32,
    rows: &[Row],
    mode: CarrierMode,
    key: u64,
) -> uor_r4_discrete::Result<CarrierSelection> {
    let targets: Vec<u32> = rows.iter().map(|r| r.0).collect();
    let mut top8 = vec![ABSENT; rows.len() * 8];
    let mut metrics = vec![0u64; rows.len() * ROW_WORDS];
    for (i, r) in rows.iter().enumerate() {
        top8[i * 8] = r.4;
        metrics[i * ROW_WORDS] = r.1;
        metrics[i * ROW_WORDS + 1] = r.2;
        metrics[i * ROW_WORDS + 5] = u64::from(r.3);
    }
    Device::carrier_select_rows(vocabulary, &targets, &top8, &metrics, mode, key, 0)
}

fn credit(rows: &[Row]) -> (u32, u64) {
    let s = select(rows, CarrierMode::CreditV1, 0).expect("credit selection");
    assert_eq!(s.mode, CarrierMode::CreditV1);
    (s.token, s.value)
}

#[test]
fn c1_the_largest_credit_wins() {
    // Row 0 (wrong, hinge 7) credits target 3 and rival 4; row 1 (right,
    // hinge 1) credits target 3 only: 3 -> 8, 4 -> 7.
    let rows = [Row(3, 5, 2, false, 4), Row(3, 1, 0, true, 3)];
    assert_eq!(credit(&rows), (3, 8));
}

#[test]
fn c2_a_tie_goes_to_the_lowest_id() {
    let rows = [Row(9, 4, 0, true, 9), Row(2, 0, 4, true, 2)];
    assert_eq!(credit(&rows), (2, 4));
}

#[test]
fn c3_zero_credit_falls_back_to_the_lowest_target() {
    let rows = [
        Row(7, 0, 0, true, 7),
        Row(5, 0, 0, true, 5),
        Row(6, 0, 0, false, 1),
    ];
    assert_eq!(credit(&rows), (5, 0));
}

#[test]
fn c4_a_rival_is_credited_only_when_top1_is_wrong() {
    // Row 0 is right, so its top-ranked 7 earns nothing (else 7 -> 11 would win);
    // row 1 is wrong and credits target 5 and rival 7 with 5 each.
    let rows = [Row(4, 6, 0, true, 7), Row(5, 3, 2, false, 7)];
    assert_eq!(credit(&rows), (4, 6));
}

#[test]
fn c5_an_out_of_range_rival_is_skipped() {
    let rows = [Row(2, 3, 0, false, ABSENT), Row(1, 1, 0, false, VOCABULARY)];
    assert_eq!(credit(&rows), (2, 3));
}

#[test]
fn c6_uniform_target_takes_the_kth_distinct_target() {
    let rows = [
        Row(9, 9, 9, false, 1),
        Row(3, 0, 0, true, 3),
        Row(9, 0, 0, true, 9),
        Row(5, 0, 0, true, 5),
    ];
    let key = schedule_key(1001, 6);
    let distinct = [3u32, 5, 9];
    let expected = distinct[(splitmix64(key) % 3) as usize];
    let s = select(&rows, CarrierMode::UniformTargetV1, key).expect("uniform target");
    assert_eq!((s.token, s.value), (expected, 3));
}

#[test]
fn c7_uniform_row_takes_the_drawn_row() {
    let rows = [
        Row(9, 9, 9, false, 1),
        Row(3, 0, 0, true, 3),
        Row(9, 0, 0, true, 9),
        Row(5, 0, 0, true, 5),
    ];
    let key = schedule_key(2001, 10);
    let row = splitmix64(key) % 4;
    let Row(expected, ..) = rows[row as usize];
    let s = select(&rows, CarrierMode::UniformRowV1, key).expect("uniform row");
    assert_eq!((s.token, s.value), (expected, row));
}

#[test]
fn c8_two_launches_are_identical() {
    let rows = [
        Row(3, 5, 2, false, 4),
        Row(11, 2, 2, false, 3),
        Row(3, 1, 0, true, 3),
    ];
    for mode in [
        CarrierMode::CreditV1,
        CarrierMode::UniformTargetV1,
        CarrierMode::UniformRowV1,
    ] {
        let key = schedule_key(1001, 41);
        assert_eq!(select(&rows, mode, key).ok(), select(&rows, mode, key).ok());
    }
}

#[test]
fn c9_a_target_outside_the_vocabulary_is_refused() {
    let rows = [Row(VOCABULARY, 1, 0, true, 1)];
    assert!(select(&rows, CarrierMode::CreditV1, 0).is_err());
}

#[test]
fn c10_ties_and_walks_inside_a_multi_id_chunk() {
    // V = 1000 gives 4-id chunks: [4, 8) holds ids 4 and 5, and [996, 1000) is
    // the last partial chunk. A within-chunk `>=` would hand the tie to 5.
    let rows = [
        Row(5, 3, 0, true, 5),
        Row(4, 2, 1, true, 4),
        Row(999, 1, 0, true, 999),
        Row(997, 0, 0, true, 997),
    ];
    let credit = select_in(1000, &rows, CarrierMode::CreditV1, 0).expect("credit");
    assert_eq!((credit.token, credit.value), (4, 3));
    // Distinct targets ascending: 4, 5, 997, 999; the walk crosses into chunks.
    let key = schedule_key(2001, 77);
    let distinct = [4u32, 5, 997, 999];
    let expected = distinct[(splitmix64(key) % 4) as usize];
    let uniform = select_in(1000, &rows, CarrierMode::UniformTargetV1, key).expect("uniform");
    assert_eq!((uniform.token, uniform.value), (expected, 4));
}
