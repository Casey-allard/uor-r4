//! Host grader for the address-metric pre-flight (#2834, claude-110; sheet
//! `docs/harness/prep/m1-address-key-full-vector-2026-10-25.md`, Bar).
//!
//! It counts stored GPU outputs only (D24): each arm's `attention-dev-b3`
//! rows, its `eval-corpus` top-1 total, and the FV table's coverage census.
//! It constructs and evaluates no model. Integer arithmetic only: the paired
//! floor `b - c > max(24, 2·√(b + c))` is decided as `d > 24 && d² > 4(b + c)`.
use crate::controller::attention_learning::evidence::B3Summary;
use crate::{Error, Result};
use serde_json::{json, Value};

pub const DEV_ROWS: u64 = 14_287;
pub const PSUCC_ROWS: u64 = 3_847;
pub const A8_FULL_ADMITTED: u64 = 4_244;
pub const A8_PSUCC_ADMITTED: u64 = 1_351;
pub const A8_TOP1: u64 = 133;
pub const RUN_BAR: u64 = 1_941;
pub const FLOOR: u64 = 24;

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}

/// One arm's stored outputs: its DEV B3 rows and its full-DEV top-1 total.
pub struct Arm {
    pub b3: B3Summary,
    pub top1: u64,
}

/// On P_succ rows: `b` rows the treatment admits and the control does not,
/// `c` the reverse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Paired {
    pub b: u64,
    pub c: u64,
}

/// Rows are joined by `row_ordinal`, never by file order, and each pair must
/// carry the same target position, token and P_succ flag.
pub fn paired_psucc(treatment: &B3Summary, control: &B3Summary) -> Result<Paired> {
    if treatment.rows.len() != control.rows.len() {
        return Err(invalid("paired arms score different DEV row counts"));
    }
    let by_ordinal: std::collections::BTreeMap<
        u64,
        &crate::controller::attention_learning::evidence::AdmissionRow,
    > = control.rows.iter().map(|r| (r.row_ordinal, r)).collect();
    if by_ordinal.len() != control.rows.len() {
        return Err(invalid("a DEV row ordinal repeats"));
    }
    let mut paired = Paired { b: 0, c: 0 };
    for t in &treatment.rows {
        let c = by_ordinal
            .get(&t.row_ordinal)
            .ok_or_else(|| invalid("a DEV row is missing from the paired arm"))?;
        if (t.target_position, t.target_token, t.is_psucc)
            != (c.target_position, c.target_token, c.is_psucc)
        {
            return Err(invalid("paired arms score different DEV row identities"));
        }
        if t.is_psucc {
            match (t.target_admitted, c.target_admitted) {
                (true, false) => paired.b += 1,
                (false, true) => paired.c += 1,
                _ => {}
            }
        }
    }
    Ok(paired)
}

/// `b - c > max(FLOOR, 2·√(b + c))`, exactly, in integers.
pub fn clears(p: Paired) -> bool {
    if p.b <= p.c {
        return false;
    }
    let d = u128::from(p.b - p.c);
    d > u128::from(FLOOR) && d * d > 4 * u128::from(p.b + p.c)
}

/// The registered panel, with summary counts that equal the rows they summarize
/// and no repeated row ordinal: the reach bar reads the counts, the paired bar
/// reads the rows, so the two must agree.
fn check_panel(name: &str, arm: &Arm) -> Result<()> {
    let b3 = &arm.b3;
    if b3.full.rows != DEV_ROWS || b3.psucc.rows != PSUCC_ROWS {
        return Err(invalid(format!(
            "{name} does not score the registered 14,287-row DEV panel with 3,847 P_succ rows"
        )));
    }
    let ordinals: std::collections::BTreeSet<u64> = b3.rows.iter().map(|r| r.row_ordinal).collect();
    let count = |psucc: Option<bool>, admitted: bool| {
        b3.rows
            .iter()
            .filter(|r| psucc.is_none_or(|p| r.is_psucc == p) && (!admitted || r.target_admitted))
            .count() as u64
    };
    if ordinals.len() as u64 != DEV_ROWS
        || b3.rows.len() as u64 != DEV_ROWS
        || count(Some(true), false) != b3.psucc.rows
        || count(None, true) != b3.full.target_admitted
        || count(Some(true), true) != b3.psucc.target_admitted
    {
        return Err(invalid(format!(
            "{name}'s B3 summary counts differ from its rows, or a row ordinal repeats"
        )));
    }
    Ok(())
}

fn paired_json(p: Paired) -> Value {
    json!({"b": p.b, "c": p.c, "clears": clears(p)})
}

/// A pinned table's census meets G-Occ: 4,096 distinct vectors in 256 cells,
/// no alias group, and every DEV row's target its cell's representative.
/// Returns that, and the alias ceiling.
fn census_ok(census: &Value) -> Result<(bool, u64)> {
    if census["schema"] != "uor-r4.coverage-census/1" {
        return Err(invalid("not a uor-r4.coverage-census/1 report"));
    }
    // A missing or renamed field is refused, never read as a gate miss.
    let word = |key: &str| {
        census
            .pointer(key)
            .and_then(Value::as_u64)
            .ok_or_else(|| invalid(format!("census lacks {key}")))
    };
    let ceiling = word("/dev_alias_ceiling_rows")?;
    let ok = word("/distinct_vectors")? == 4_096
        && word("/cells/occupied_cells")? == 256
        && word("/within_cell_tie_mass/tokens_in_alias_groups")? == 0
        && word("/within_cell_tie_mass/dev_rows_targeting_non_representative")? == 0
        && ceiling == DEV_ROWS;
    Ok((ok, ceiling))
}

/// The sheet's pinned config file sha256s, by manifest key. Each arm's
/// `attention-dev-b3` report records the bytes it read (`config_file.sha256`),
/// so a manifest that swaps two arms is refused.
pub const ARM_CONFIGS: [(&str, &str); 5] = [
    (
        "a8",
        "469a2ff0fe17a6a80f5875a077b3e6a9436553efdf1c6e7884541220553e9d5e",
    ),
    (
        "fv8_env",
        "56c3fafe0b8cdb7800f57d0a6ba577b2964356de5765b350a701546cace6e534",
    ),
    (
        "fv8_two_scale",
        "1cdb80615d1467bbe062bc1a70804630b3f69af5c580ce3b0d2b5d7407a89a28",
    ),
    (
        "fvs8_env",
        "b3b5fb60a904eaecb17d36f3ff5263ff4f83d6846612380df5ce8a5aef20830e",
    ),
    (
        "fvs8_two_scale",
        "05d907d0d6e2d3526642b32ead7edf7a9db422c02334ff3ed9aa03939950c834",
    ),
];

/// The sheet's decision table for one treatment, against A8-Outer and against
/// the shuffled arm with the same metric. G-Exact and G-Occ on either pinned
/// table are instrument checks: a failure there is never a mechanism result.
pub fn grade(
    a8: &Arm,
    treatment: &Arm,
    shuffled: &Arm,
    fv_census: &Value,
    fvs_census: &Value,
    name: &str,
) -> Result<Value> {
    for (label, arm) in [("A8-Outer", a8), (name, treatment), ("shuffled", shuffled)] {
        check_panel(label, arm)?;
    }
    let g_exact = a8.b3.full.target_admitted == A8_FULL_ADMITTED
        && a8.b3.psucc.target_admitted == A8_PSUCC_ADMITTED
        && a8.top1 == A8_TOP1;
    let (fv_ok, ceiling) = census_ok(fv_census)?;
    let (fvs_ok, _) = census_ok(fvs_census)?;
    let g_occ = fv_ok && fvs_ok;
    let vs_a8 = paired_psucc(&treatment.b3, &a8.b3)?;
    let vs_shuffled = paired_psucc(&treatment.b3, &shuffled.b3)?;
    let reach = treatment.b3.psucc.target_admitted > A8_PSUCC_ADMITTED && clears(vs_a8);
    // Near-vacuous with a 14,287-row alias ceiling: it reduces to the
    // treatment's full-DEV admission clearing 1,941. Recorded, not evidence.
    let capacity = ceiling.min(treatment.b3.full.target_admitted) > RUN_BAR;
    let outcome = if !g_exact || !g_occ {
        "INSTRUMENT"
    } else if reach && capacity && clears(vs_shuffled) {
        "PASS"
    } else if reach && capacity {
        "PARTIAL"
    } else {
        "FAIL"
    };
    Ok(json!({
        "schema": "uor-r4.address-key-grade/1",
        "arm": name,
        "rule": "reach: P_succ admitted > 1,351 and b - c > max(24, 2·sqrt(b + c)) on P_succ against A8-Outer; PASS also against the same-metric shuffled arm; capacity: min(alias ceiling, full admitted) > 1,941",
        "g_exact": g_exact,
        "g_occ": {"fv": fv_ok, "fvs": fvs_ok},
        "a8": {"psucc_admitted": a8.b3.psucc.target_admitted, "full_admitted": a8.b3.full.target_admitted, "top1": a8.top1},
        "treatment": {"psucc_admitted": treatment.b3.psucc.target_admitted, "full_admitted": treatment.b3.full.target_admitted, "top1": treatment.top1},
        "shuffled": {"psucc_admitted": shuffled.b3.psucc.target_admitted, "full_admitted": shuffled.b3.full.target_admitted, "top1": shuffled.top1},
        "vs_a8": paired_json(vs_a8),
        "vs_shuffled": paired_json(vs_shuffled),
        "reach": reach,
        "capacity": capacity,
        "outcome": outcome,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controller::attention_learning::evidence::{AdmissionRow, B3Count};

    /// A registered-size panel. P_succ rows are the first 3,847; `psucc` and
    /// `rest` give how many of each part admit their target.
    fn arm(psucc: u64, rest: u64, top1: u64) -> Arm {
        let rows: Vec<AdmissionRow> = (0..DEV_ROWS)
            .map(|i| {
                let is_psucc = i < PSUCC_ROWS;
                let admitted = if is_psucc {
                    i < psucc
                } else {
                    i - PSUCC_ROWS < rest
                };
                AdmissionRow {
                    row_ordinal: i,
                    target_position: i + 1,
                    target_token: (i % 4_096) as u32,
                    is_psucc,
                    target_admitted: admitted,
                }
            })
            .collect();
        let count = |f: &dyn Fn(&AdmissionRow) -> bool| B3Count {
            rows: rows.iter().filter(|r| f(r)).count() as u64,
            target_admitted: rows.iter().filter(|r| f(r) && r.target_admitted).count() as u64,
        };
        let b3 = B3Summary {
            full: count(&|_| true),
            psucc: count(&|r| r.is_psucc),
            complement: count(&|r| !r.is_psucc),
            rows,
        };
        Arm { b3, top1 }
    }

    fn census() -> Value {
        json!({"schema": "uor-r4.coverage-census/1", "distinct_vectors": 4096, "cells": {"occupied_cells": 256}, "dev_alias_ceiling_rows": 14287,
            "within_cell_tie_mass": {"tokens_in_alias_groups": 0, "dev_rows_targeting_non_representative": 0}})
    }

    #[test]
    fn the_paired_floor_is_exact_in_integers() {
        assert!(!clears(Paired { b: 10, c: 10 }));
        assert!(
            !clears(Paired { b: 24, c: 0 }),
            "d = 24 is not above the floor"
        );
        assert!(clears(Paired { b: 25, c: 0 }), "d = 25 > 24 and 625 > 100");
        assert!(clears(Paired { b: 200, c: 100 }), "10,000 > 1,200");
        assert!(!clears(Paired { b: 1_000, c: 950 }), "2,500 <= 7,800");
        assert!(!clears(Paired { b: 0, c: 30 }));
        // The square-root boundary itself: 28² = 784 = 4·196 does not clear.
        assert!(!clears(Paired { b: 112, c: 84 }));
        assert!(clears(Paired { b: 113, c: 84 }), "29² = 841 > 788");
    }

    #[test]
    fn rows_pair_only_on_identical_identities() {
        let a = arm(1_400, 2_893, 133);
        let b = arm(1_351, 2_893, 133);
        assert_eq!(paired_psucc(&a.b3, &b.b3).unwrap(), Paired { b: 49, c: 0 });
        let mut moved = arm(1_351, 2_893, 133);
        moved.b3.rows[5].target_token += 1;
        assert!(paired_psucc(&a.b3, &moved.b3).is_err());
        // File order does not matter: rows join by ordinal.
        let mut reversed = arm(1_351, 2_893, 133);
        reversed.b3.rows.reverse();
        assert_eq!(
            paired_psucc(&a.b3, &reversed.b3).unwrap(),
            Paired { b: 49, c: 0 }
        );
        let mut repeated = arm(1_351, 2_893, 133);
        repeated.b3.rows[1].row_ordinal = 0;
        assert!(paired_psucc(&a.b3, &repeated.b3).is_err());
    }

    #[test]
    fn the_decision_table() {
        let a8 = arm(
            A8_PSUCC_ADMITTED,
            A8_FULL_ADMITTED - A8_PSUCC_ADMITTED,
            A8_TOP1,
        );
        let strong = arm(1_800, 3_000, 400);
        let shuffled = arm(1_500, 3_000, 300);
        let c = census();
        let outcome = |a8: &Arm, t: &Arm, s: &Arm, fv: &Value, fvs: &Value| {
            grade(a8, t, s, fv, fvs, "FV8-TwoScale").unwrap()["outcome"].clone()
        };
        let pass = grade(&a8, &strong, &shuffled, &c, &c, "FV8-TwoScale").unwrap();
        assert_eq!(pass["outcome"], "PASS");
        assert_eq!(pass["vs_a8"]["b"], 449);
        assert_eq!(pass["vs_shuffled"]["b"], 300);
        // Beats A8 but not its shuffled packing: the gain is packing balance.
        assert_eq!(outcome(&a8, &shuffled, &shuffled, &c, &c), "PARTIAL");
        // No reach over A8 + F.
        let flat = arm(1_360, 3_000, 140);
        assert_eq!(outcome(&a8, &flat, &shuffled, &c, &c), "FAIL");
        // Reach and geometry without capacity: 1,800 full-DEV admitted <= 1,941.
        let narrow = arm(1_800, 0, 400);
        assert_eq!(outcome(&a8, &narrow, &shuffled, &c, &c), "FAIL");
        // A census missing a field, or of another schema, is refused, not graded.
        let mut renamed = census();
        renamed.as_object_mut().unwrap().remove("distinct_vectors");
        assert!(grade(&a8, &strong, &shuffled, &renamed, &c, "FV8-TwoScale").is_err());
        let mut foreign = census();
        foreign["schema"] = json!("uor-r4.something-else/1");
        assert!(grade(&a8, &strong, &shuffled, &c, &foreign, "FV8-TwoScale").is_err());
        // G-Occ on either pinned table is an instrument check, never a FAIL.
        let mut aliased = census();
        aliased["within_cell_tie_mass"]["tokens_in_alias_groups"] = json!(2);
        assert_eq!(outcome(&a8, &strong, &shuffled, &aliased, &c), "INSTRUMENT");
        assert_eq!(outcome(&a8, &strong, &shuffled, &c, &aliased), "INSTRUMENT");
        // A8 off by one row is an instrument fault, read before any treatment.
        let off = arm(
            A8_PSUCC_ADMITTED - 1,
            A8_FULL_ADMITTED - A8_PSUCC_ADMITTED,
            A8_TOP1,
        );
        assert_eq!(outcome(&off, &strong, &shuffled, &c, &c), "INSTRUMENT");
        let wrong_top1 = arm(
            A8_PSUCC_ADMITTED,
            A8_FULL_ADMITTED - A8_PSUCC_ADMITTED,
            A8_TOP1 + 1,
        );
        assert_eq!(
            outcome(&wrong_top1, &strong, &shuffled, &c, &c),
            "INSTRUMENT"
        );
        // A panel of another size is refused, not graded.
        let mut short = arm(1_800, 3_000, 400);
        short.b3.full.rows -= 1;
        assert!(grade(&a8, &short, &shuffled, &c, &c, "FV8-TwoScale").is_err());
        // A summary that disagrees with its rows, or a repeated treatment
        // ordinal, is refused: reach reads counts, the paired bar reads rows.
        let mut inflated = arm(1_800, 3_000, 400);
        inflated.b3.psucc.target_admitted += 100;
        assert!(grade(&a8, &inflated, &shuffled, &c, &c, "FV8-TwoScale").is_err());
        let mut repeated = arm(1_800, 3_000, 400);
        repeated.b3.rows[1].row_ordinal = 0;
        assert!(grade(&a8, &repeated, &shuffled, &c, &c, "FV8-TwoScale").is_err());
    }
}
