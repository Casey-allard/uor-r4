//! Host grader for the prime-zeta zero-update pre-flight (#2881, claude-112;
//! sheet `docs/harness/prep/r-prime-zeta-vs-nulls-2026-10-25.md`, Bar; decision
//! card `docs/loop/tasks/r-prime-zeta-vs-nulls/claude-112/DECISION.md`, B and C).
//!
//! It counts stored GPU outputs only (D24): each arm's `attention-dev-b3` rows,
//! its `eval-corpus` totals and its `coverage-census --token-cells` report. It
//! constructs and evaluates no model. The paired floor is the merged
//! `address_key_grade::clears`: `b - c > max(24, 2·√(b + c))`, exactly, in
//! integers.
use serde_json::{json, Value};
use uor_r4_discrete::address_key_grade::{clears, paired_psucc, Paired, DEV_ROWS, PSUCC_ROWS};
use uor_r4_discrete::controller::attention_learning::evidence::B3Summary;

/// The seven arms, P8 first, then Q0..Q2 and R0..R2, each with its pinned
/// config file sha256. Each `attention-dev-b3` report records the bytes it read
/// (`config_file.sha256`), so a manifest that swaps two arms is refused.
pub const ARMS: [(&str, &str); 7] = [
    ("p8", "7927b1ddde3025d947ce56166c0a30e3ad8d310211aaab1df80851a4ae611bda"),
    ("q0", "51986b8c0773a1a003c6a8cde2d63ea7068968beeff19f44e689f7fce66ec6d7"),
    ("q1", "94c5634d71a25484c682cfe8606b976eaa33c22577432ecd2fc9e5160431973b"),
    ("q2", "6f8b70dff8e692de6cdff9133de01f26c8897bd9d5b8b0034400cbd35512f945"),
    ("r0", "6a3a457aae0e8f94b7a095f8e9a9e9081bd91a22f77b37e5ae67d8e99093ed1b"),
    ("r1", "dc188404fd1eb2d5e94266dbdee5e5478deeef2a1be39d8826f8d85800fcdb94"),
    ("r2", "9c78bfa93d60412f9d86d5f4c19830be8c79797e824458ae00a2c249ab04b144"),
];

/// P8's 19 October pre-flight numbers (G-Exact): B3_full, B3_init(P_succ), top-1.
pub const P8_EXACT: (u64, u64, u64) = (427, 127, 38);
/// 256 cells of 16 tokens each: 256 · C(16, 2).
pub const QUOTA_PAIRS: u64 = 30_720;

/// One arm's stored outputs.
pub struct Arm {
    pub b3: B3Summary,
    pub top1: u64,
    pub top8: u64,
    pub census: Value,
}

/// Builds an arm from its three reports, refusing a report that is not this
/// arm's or whose model carries an update. The update state is read from the
/// checkpoint's own counters (`model_epoch`, `completed_batches`,
/// `accepted_updates_in_model`), never from a field the writer fixes at 0. The
/// three reports are bound to one model by its storage hash, which is the
/// checkpoint's `model_sha256` in all three.
pub fn arm_from_reports(name: &str, dev_b3: &Value, eval: &Value, census: Value) -> Result<Arm, String> {
    let pinned = ARMS
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, sha)| *sha)
        .ok_or_else(|| format!("{name}: not a registered arm"))?;
    if dev_b3["schema"] != "uor-r4.attention-preflight-dev-b3/1"
        || dev_b3["model_epoch"] != 0
        || dev_b3["completed_batches"] != 0
    {
        return Err(format!("{name}: not a checkpoint-0 attention-dev-b3 report"));
    }
    if dev_b3["config_file"]["sha256"] != pinned {
        return Err(format!("{name}: its dev-b3 report read another config"));
    }
    let b3 = serde_json::from_value(dev_b3["b3"].clone()).map_err(|e| format!("{name} b3: {e}"))?;
    if eval["schema"] != "uor-r4.discrete-corpus-driver/1"
        || eval["model_epoch"] != 0
        || eval["accepted_updates_in_model"] != 0
    {
        return Err(format!("{name}: not a checkpoint-0 eval-corpus report"));
    }
    // A missing hash is refused, so two absent fields never compare equal.
    let hash = |report: &Value, key: &str, what: &str| {
        report[key]
            .as_str()
            .filter(|h| h.len() == 64)
            .map(str::to_owned)
            .ok_or_else(|| format!("{name}: its {what} report lacks {key}"))
    };
    let storage = hash(dev_b3, "model_storage_sha256", "dev-b3")?;
    if hash(eval, "model_storage_sha256", "eval")? != storage
        || hash(eval, "model_snapshot_sha256", "eval")? != hash(dev_b3, "model_snapshot_sha256", "dev-b3")?
    {
        return Err(format!("{name}: its eval report scored another model"));
    }
    if hash(&census, "model_sha256", "census")? != storage {
        return Err(format!("{name}: its census read another model"));
    }
    let total = |key: &str| {
        eval["aggregate"][key]
            .as_u64()
            .ok_or_else(|| format!("{name}: eval report lacks aggregate.{key}"))
    };
    Ok(Arm { b3, top1: total("top1_correct")?, top8: total("top8_correct")?, census })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Win,
    Tie,
    Loss,
}

impl Outcome {
    fn word(self) -> &'static str {
        match self {
            Outcome::Win => "win",
            Outcome::Tie => "tie",
            Outcome::Loss => "loss",
        }
    }
}

/// P8 against one null: a win clears the floor, a loss clears it the other way.
/// `clears` is false whenever its `b <= c`, so the two never both hold.
pub fn outcome(p: Paired) -> Outcome {
    if clears(p) {
        Outcome::Win
    } else if clears(Paired { b: p.c, c: p.b }) {
        Outcome::Loss
    } else {
        Outcome::Tie
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Positive,
    NegativeTie,
    NegativeLoss,
    Residual,
}

impl Verdict {
    pub fn word(self) -> &'static str {
        match self {
            Verdict::Positive => "positive",
            Verdict::NegativeTie => "negative (tie)",
            Verdict::NegativeLoss => "negative (loss)",
            Verdict::Residual => "residual",
        }
    }
}

/// The registered partition (card B), total by the count of Q wins: 0 is
/// negative, 1 or 2 is residual, 3 is positive only with three R wins.
pub fn decide(q: [Outcome; 3], r: [Outcome; 3]) -> Verdict {
    match q.iter().filter(|o| **o == Outcome::Win).count() {
        0 if q.contains(&Outcome::Loss) => Verdict::NegativeLoss,
        0 => Verdict::NegativeTie,
        3 if r.iter().all(|o| *o == Outcome::Win) => Verdict::Positive,
        _ => Verdict::Residual,
    }
}

/// The registered panel, with summary counts that equal the rows they
/// summarize and no repeated row ordinal: reach reads the counts, the paired
/// bar reads the rows, so the two must agree.
pub fn check_panel(name: &str, b3: &B3Summary) -> Result<(), String> {
    if b3.full.rows != DEV_ROWS || b3.psucc.rows != PSUCC_ROWS {
        return Err(format!("{name} does not score the registered 14,287-row DEV panel with 3,847 P_succ rows"));
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
        return Err(format!("{name}'s B3 summary counts differ from its rows, or a row ordinal repeats"));
    }
    Ok(())
}

/// A table's capacity words, from its `coverage-census/1` report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capacity {
    pub distinct: u64,
    pub ceiling: u64,
    pub alias_tokens: u64,
    pub non_representative: u64,
    pub occupied: u64,
    pub min_cell: u64,
    pub max_cell: u64,
    pub collisions: u64,
}

/// A missing or renamed field is refused, never read as a gate miss.
pub fn capacity(name: &str, census: &Value) -> Result<Capacity, String> {
    if census["schema"] != "uor-r4.coverage-census/1" {
        return Err(format!("{name}: not a uor-r4.coverage-census/1 report"));
    }
    let word = |key: &str| {
        census
            .pointer(key)
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("{name}: census lacks {key}"))
    };
    let histogram = census
        .pointer("/cells/cell_occupancy_histogram")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("{name}: census lacks /cells/cell_occupancy_histogram"))?;
    let min_cell = histogram
        .keys()
        .map(|k| k.parse::<u64>().map_err(|_| format!("{name}: occupancy key {k} is not a count")))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .min()
        .ok_or_else(|| format!("{name}: empty occupancy histogram"))?;
    Ok(Capacity {
        distinct: word("/distinct_vectors")?,
        ceiling: word("/dev_alias_ceiling_rows")?,
        alias_tokens: word("/within_cell_tie_mass/tokens_in_alias_groups")?,
        non_representative: word("/within_cell_tie_mass/dev_rows_targeting_non_representative")?,
        occupied: word("/cells/occupied_cells")?,
        min_cell,
        max_cell: word("/cells/max_cell_occupancy")?,
        collisions: word("/cells/cell_collision_pairs")?,
    })
}

/// G-Occ (card C). The quota arms are P8 and Q0..Q2.
pub fn g_occ(quota: bool, c: &Capacity) -> bool {
    let common = c.distinct == 4_096
        && c.ceiling == DEV_ROWS
        && c.alias_tokens == 0
        && c.non_representative == 0
        && c.occupied == 256;
    common
        && if quota {
            c.max_cell == 16 && c.collisions == QUOTA_PAIRS
        } else {
            c.collisions > QUOTA_PAIRS
        }
}

fn paired_json(p: Paired, o: Outcome) -> Value {
    json!({"b": p.b, "c": p.c, "outcome": o.word()})
}

/// The registered grade over the seven arms in `ARMS` order. A refused input is
/// an error; a failed instrument gate (G-Exact, G-Occ) is a record whose
/// verdict is "instrument fault", with no paired reading (D31).
pub fn grade(arms: &[Arm; 7]) -> Result<Value, String> {
    let mut reach = serde_json::Map::new();
    let mut caps = serde_json::Map::new();
    let mut faults = Vec::new();
    for (i, (name, _)) in ARMS.iter().enumerate() {
        let arm = &arms[i];
        check_panel(name, &arm.b3)?;
        let c = capacity(name, &arm.census)?;
        if !g_occ(i < 4, &c) {
            faults.push(format!("G-Occ: {name} gives {c:?}"));
        }
        reach.insert(
            name.to_string(),
            json!({"b3_full": arm.b3.full.target_admitted, "b3_psucc": arm.b3.psucc.target_admitted,
                   "top1": arm.top1, "top8": arm.top8}),
        );
        caps.insert(
            name.to_string(),
            json!({"distinct_vectors": c.distinct, "dev_alias_ceiling_rows": c.ceiling,
                   "occupied_cells": c.occupied, "min_cell": c.min_cell, "max_cell": c.max_cell,
                   "cell_collision_pairs": c.collisions}),
        );
    }
    let p8 = &arms[0];
    let exact = (p8.b3.full.target_admitted, p8.b3.psucc.target_admitted, p8.top1);
    if exact != P8_EXACT {
        faults.insert(0, format!("G-Exact: P8 gives {exact:?}, not (427, 127, 38)"));
    }
    let mut record = json!({
        "schema": "uor-r4.pz-nulls-grade/1",
        "rule": "win: clears(Paired{b,c}); loss: clears(Paired{b:c,c:b}); verdict by Q wins (card B)",
        "reach": reach,
        "capacity": caps,
    });
    if !faults.is_empty() {
        record["verdict"] = json!("instrument fault");
        record["faults"] = json!(faults);
        return Ok(record);
    }
    let mut outcomes = [Outcome::Tie; 6];
    let mut pairs = serde_json::Map::new();
    for i in 1..7 {
        let (name, _) = ARMS[i];
        let p = paired_psucc(&p8.b3, &arms[i].b3).map_err(|e| format!("p8 vs {name}: {e}"))?;
        outcomes[i - 1] = outcome(p);
        pairs.insert(format!("p8_vs_{name}"), paired_json(p, outcomes[i - 1]));
    }
    // Descriptive only: matched-seed quota against crowding.
    let mut crowding = serde_json::Map::new();
    for s in 0..3 {
        let p = paired_psucc(&arms[1 + s].b3, &arms[4 + s].b3).map_err(|e| format!("q{s} vs r{s}: {e}"))?;
        crowding.insert(format!("q{s}_vs_r{s}"), paired_json(p, outcome(p)));
    }
    let verdict = decide([outcomes[0], outcomes[1], outcomes[2]], [outcomes[3], outcomes[4], outcomes[5]]);
    record["pairs"] = json!(pairs);
    record["crowding_descriptive"] = json!(crowding);
    record["verdict"] = json!(verdict.word());
    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Range;
    use uor_r4_discrete::controller::attention_learning::evidence::{AdmissionRow, B3Count};
    use Outcome::{Loss, Tie, Win};

    /// A synthetic arm on the registered panel: P_succ rows are ordinals
    /// 0..3,847; `psucc` and the first `complement` complement rows are admitted.
    fn arm(psucc: Range<u64>, complement: u64, top1: u64, collisions: u64) -> Arm {
        let rows: Vec<AdmissionRow> = (0..DEV_ROWS)
            .map(|o| AdmissionRow {
                row_ordinal: o,
                target_position: o,
                target_token: (o % 4_096) as u32,
                is_psucc: o < PSUCC_ROWS,
                target_admitted: psucc.contains(&o) || (o >= PSUCC_ROWS && o < PSUCC_ROWS + complement),
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
        let max_cell = if collisions == QUOTA_PAIRS { 16 } else { 27 };
        let census = json!({"schema": "uor-r4.coverage-census/1", "distinct_vectors": 4096,
            "dev_alias_ceiling_rows": 14287,
            "within_cell_tie_mass": {"tokens_in_alias_groups": 0, "dev_rows_targeting_non_representative": 0},
            "cells": {"occupied_cells": 256, "max_cell_occupancy": max_cell, "cell_collision_pairs": collisions,
                      "cell_occupancy_histogram": {"16": 256}}});
        Arm { b3, top1, top8: 0, census }
    }

    const R_PAIRS: u64 = 32_493;

    fn p8() -> Arm {
        arm(0..127, 300, 38, QUOTA_PAIRS)
    }

    /// P8, then three Q arms and three R arms with the given P_succ admissions.
    fn arms(q: [Range<u64>; 3], r: [Range<u64>; 3]) -> [Arm; 7] {
        let [q0, q1, q2] = q;
        let [r0, r1, r2] = r;
        [
            p8(),
            arm(q0, 300, 38, QUOTA_PAIRS),
            arm(q1, 300, 38, QUOTA_PAIRS),
            arm(q2, 300, 38, QUOTA_PAIRS),
            arm(r0, 300, 38, R_PAIRS),
            arm(r1, 300, 38, R_PAIRS),
            arm(r2, 300, 38, R_PAIRS),
        ]
    }

    #[test]
    fn outcome_is_the_exact_floor_both_ways() {
        assert_eq!(outcome(Paired { b: 25, c: 0 }), Win);
        assert_eq!(outcome(Paired { b: 24, c: 0 }), Tie);
        assert_eq!(outcome(Paired { b: 0, c: 25 }), Loss);
        // d = 40: 1,600 > 4 · 360; d = 30: 900 <= 4 · 370.
        assert_eq!(outcome(Paired { b: 200, c: 160 }), Win);
        assert_eq!(outcome(Paired { b: 200, c: 170 }), Tie);
        assert_eq!(outcome(Paired { b: 160, c: 200 }), Loss);
    }

    #[test]
    fn decide_is_the_registered_partition_over_every_pattern() {
        let all = [Win, Tie, Loss];
        let mut seen = 0;
        for code in 0..729u32 {
            let pick = |i: u32| all[((code / 3u32.pow(i)) % 3) as usize];
            let q = [pick(0), pick(1), pick(2)];
            let r = [pick(3), pick(4), pick(5)];
            let q_wins = q.iter().filter(|o| **o == Win).count();
            let expected = if q_wins == 0 && q.contains(&Loss) {
                Verdict::NegativeLoss
            } else if q_wins == 0 {
                Verdict::NegativeTie
            } else if q_wins == 3 && r == [Win; 3] {
                Verdict::Positive
            } else {
                Verdict::Residual
            };
            assert_eq!(decide(q, r), expected, "q {q:?} r {r:?}");
            seen += 1;
        }
        assert_eq!(seen, 729);
        // The draft's contradictory example: tied with every Q, clearing every R.
        assert_eq!(decide([Tie; 3], [Win; 3]), Verdict::NegativeTie);
        assert_eq!(decide([Tie, Loss, Tie], [Win; 3]), Verdict::NegativeLoss);
        assert_eq!(decide([Win; 3], [Win, Win, Tie]), Verdict::Residual);
    }

    #[test]
    fn six_wins_grade_positive() {
        // Each null admits P_succ rows 0..60: b = 67, c = 0 against P8.
        let g = grade(&arms([0..60, 0..60, 0..60], [0..60, 0..60, 0..60])).unwrap();
        assert_eq!(g["verdict"], "positive");
        assert_eq!(g["pairs"]["p8_vs_q0"], json!({"b": 67, "c": 0, "outcome": "win"}));
        assert_eq!(g["capacity"]["r0"]["cell_collision_pairs"], R_PAIRS);
    }

    #[test]
    fn quota_ties_grade_negative_whatever_the_r_arms_give() {
        // Q admits 10..137: b = 10, c = 10, a tie. R admits 0..60: a win.
        let g = grade(&arms([10..137, 10..137, 10..137], [0..60, 0..60, 0..60])).unwrap();
        assert_eq!(g["verdict"], "negative (tie)");
        assert_eq!(g["pairs"]["p8_vs_r2"]["outcome"], "win");
    }

    #[test]
    fn a_quota_loss_grades_negative_loss() {
        // Q0 admits 0..200: b = 0, c = 73.
        let g = grade(&arms([0..200, 10..137, 10..137], [0..60, 0..60, 0..60])).unwrap();
        assert_eq!(g["verdict"], "negative (loss)");
        assert_eq!(g["pairs"]["p8_vs_q0"], json!({"b": 0, "c": 73, "outcome": "loss"}));
    }

    #[test]
    fn p8_off_its_record_is_an_instrument_fault_with_no_reading() {
        let mut a = arms([0..60, 0..60, 0..60], [0..60, 0..60, 0..60]);
        a[0] = arm(0..128, 300, 38, QUOTA_PAIRS);
        let g = grade(&a).unwrap();
        assert_eq!(g["verdict"], "instrument fault");
        assert!(g["faults"][0].as_str().unwrap().starts_with("G-Exact"));
        assert!(g.get("pairs").is_none());
    }

    #[test]
    fn quota_crowding_on_an_r_arm_is_an_instrument_fault() {
        let mut a = arms([0..60, 0..60, 0..60], [0..60, 0..60, 0..60]);
        a[5] = arm(0..60, 300, 38, QUOTA_PAIRS);
        let g = grade(&a).unwrap();
        assert_eq!(g["verdict"], "instrument fault");
        assert!(g["faults"][0].as_str().unwrap().starts_with("G-Occ: r1"));
        // Every arm, R included, must occupy all 256 cells.
        let mut a = arms([0..60, 0..60, 0..60], [0..60, 0..60, 0..60]);
        a[6].census["cells"]["occupied_cells"] = json!(255);
        let g = grade(&a).unwrap();
        assert!(g["faults"][0].as_str().unwrap().starts_with("G-Occ: r2"));
    }

    #[test]
    fn a_summary_that_disagrees_with_its_rows_is_refused() {
        let mut a = arms([0..60, 0..60, 0..60], [0..60, 0..60, 0..60]);
        a[2].b3.psucc.target_admitted += 1;
        assert!(grade(&a).unwrap_err().contains("q1's B3 summary counts differ"));
    }

    #[test]
    fn a_missing_census_field_is_refused() {
        let mut a = arms([0..60, 0..60, 0..60], [0..60, 0..60, 0..60]);
        a[4].census["cells"].as_object_mut().unwrap().remove("cell_collision_pairs");
        assert!(grade(&a).unwrap_err().contains("r0: census lacks /cells/cell_collision_pairs"));
    }

    #[test]
    fn a_report_for_another_arm_model_or_a_trained_checkpoint_is_refused() {
        let storage = "a".repeat(64);
        let snapshot = "b".repeat(64);
        let [(_, p8_config), (_, q0_config), ..] = ARMS;
        // The fields as the crate writes them: `updates_applied` and
        // `updates_applied_by_evaluation` are literal zeros, so only the
        // checkpoint counters can carry an update.
        let dev = json!({"schema": "uor-r4.attention-preflight-dev-b3/1", "updates_applied": 0,
            "model_epoch": 0, "completed_batches": 0, "config_file": {"sha256": p8_config},
            "model_storage_sha256": storage, "model_snapshot_sha256": snapshot,
            "b3": serde_json::to_value(&p8().b3).unwrap()});
        let eval = json!({"schema": "uor-r4.discrete-corpus-driver/1", "updates_applied_by_evaluation": 0,
            "model_epoch": 0, "accepted_updates_in_model": 0,
            "model_storage_sha256": storage, "model_snapshot_sha256": snapshot,
            "aggregate": {"top1_correct": 38, "top8_correct": 120}});
        let mut census = p8().census;
        census["model_sha256"] = json!(storage);
        let refused = |d: &Value, e: &Value, c: &Value| arm_from_reports("p8", d, e, c.clone()).err().unwrap();
        assert!(arm_from_reports("p8", &dev, &eval, census.clone()).is_ok());

        let mut swapped = dev.clone();
        swapped["config_file"]["sha256"] = json!(q0_config);
        assert!(refused(&swapped, &eval, &census).contains("read another config"));
        for key in ["model_epoch", "completed_batches"] {
            let mut trained = dev.clone();
            trained[key] = json!(16_404);
            assert!(refused(&trained, &eval, &census).contains("not a checkpoint-0 attention-dev-b3"), "{key}");
        }
        for key in ["model_epoch", "accepted_updates_in_model"] {
            let mut trained = eval.clone();
            trained[key] = json!(1_410);
            assert!(refused(&dev, &trained, &census).contains("not a checkpoint-0 eval-corpus"), "{key}");
        }
        let mut other = eval.clone();
        other["model_storage_sha256"] = json!("c".repeat(64));
        assert!(refused(&dev, &other, &census).contains("eval report scored another model"));
        let mut other = census.clone();
        other["model_sha256"] = json!("c".repeat(64));
        assert!(refused(&dev, &eval, &other).contains("census read another model"));
        // Absent hashes on both sides are refused, never read as equal.
        let mut bare_dev = dev.clone();
        bare_dev.as_object_mut().unwrap().remove("model_storage_sha256");
        let mut bare_eval = eval.clone();
        bare_eval.as_object_mut().unwrap().remove("model_storage_sha256");
        assert!(refused(&bare_dev, &bare_eval, &census).contains("lacks model_storage_sha256"));
    }
}
