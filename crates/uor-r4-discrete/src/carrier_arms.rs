//! The eight arms of `carrier-credit-hard-selection` (#2819; prep sheet §2, §5, §7).
//!
//! An arm is the registered base config with one schedule, one seed and a
//! seeded-device codebook, trained for the released dose and scored on DEV
//! and on the sealed fresh slice. It runs only under a D67 release record: the
//! lab writes it after its own FIT and pre-run review, and it pins the sheet's
//! bytes, the fresh slice and every arm's lineage. Grading is exact integer
//! arithmetic over stored counts.

use crate::corpus::{self, PreparedCorpus, Split};
use crate::training::{CodebookInitialization, DriverConfig, SchedulePolicy};
use crate::w2_capacity::{build_tables, predict, TrainRow};
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}

pub const RELEASE_SCHEMA: &str = "uor-r4.carrier-arms-release/1";
pub const RECORD_SCHEMA: &str = "uor-r4.carrier-arm-record/1";
pub const ITEM: &str = "carrier-credit-hard-selection";
pub const ISSUE: u64 = 2819;
pub const DOSE_BATCHES: u64 = 16_404;
pub const SEEDS: [u32; 2] = [1001, 2001];
pub const SCHEDULES: [SchedulePolicy; 4] = [
    SchedulePolicy::CarrierCreditV1,
    SchedulePolicy::CarrierUniformTargetV1,
    SchedulePolicy::CarrierUniformRowV1,
    SchedulePolicy::InterleavedReadEvery8V1,
];
const COMMENT_PREFIX: &str = "https://github.com/UOR-Foundation/uor-r4/issues/2819#issuecomment-";

/// The base config with this arm's schedule and seed, on a seeded-device codebook.
pub fn arm_config(
    base: &DriverConfig,
    schedule: SchedulePolicy,
    seed: u32,
) -> Result<DriverConfig> {
    if !SCHEDULES.contains(&schedule) || !SEEDS.contains(&seed) {
        return Err(invalid("arm schedule or seed is not registered"));
    }
    let mut config = base.clone();
    config.schedule = schedule;
    config.seed = seed;
    config.codebook_initialization = CodebookInitialization::SeededDeviceV1;
    config.validate()?;
    Ok(config)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SheetPin {
    pub path: String,
    pub sha256: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FreshPin {
    pub tokens_sha256: String,
    pub mask_sha256: String,
    pub visited_targets: u64,
    pub kept_targets: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseArm {
    pub schedule: SchedulePolicy,
    pub seed: u32,
    pub lineage_sha256: String,
}
/// The D67 release: the lab's own FIT and pre-run review, pinned to exact bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CarrierRelease {
    pub schema: String,
    pub item: String,
    pub issue: u64,
    pub sheet: SheetPin,
    pub fit_url: String,
    pub prerun_url: String,
    pub max_completed_batches: u64,
    pub fresh: FreshPin,
    pub arms: Vec<ReleaseArm>,
}

impl CarrierRelease {
    /// Refuse unless the record releases exactly this arm, at this lineage,
    /// under these sheet bytes, with the dose ending before the fresh slice.
    pub fn validate_for(
        &self,
        sheet_bytes: &[u8],
        lineage_sha256: &str,
        schedule: SchedulePolicy,
        seed: u32,
        batch_rows: u32,
    ) -> Result<()> {
        if self.schema != RELEASE_SCHEMA || self.item != ITEM || self.issue != ISSUE {
            return Err(invalid("release is not this item's D67 record"));
        }
        if corpus::sha256(sheet_bytes) != self.sheet.sha256 {
            return Err(invalid("prep sheet bytes differ from the released sheet"));
        }
        for url in [&self.fit_url, &self.prerun_url] {
            let id = url.strip_prefix(COMMENT_PREFIX).unwrap_or("");
            if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
                return Err(invalid(
                    "FIT and pre-run reviews must be #2819 comment links",
                ));
            }
        }
        if self.max_completed_batches != DOSE_BATCHES
            || self.fresh.visited_targets != DOSE_BATCHES * u64::from(batch_rows)
            || self.fresh.kept_targets < 200
        {
            return Err(invalid(
                "release dose must be the registered 16,404 batches ending at the fresh slice",
            ));
        }
        let mut pairs: Vec<(u32, String)> = self
            .arms
            .iter()
            .map(|a| (a.seed, format!("{:?}", a.schedule)))
            .collect();
        pairs.sort();
        pairs.dedup();
        let registered = self
            .arms
            .iter()
            .all(|a| SCHEDULES.contains(&a.schedule) && SEEDS.contains(&a.seed));
        // Registered, distinct and eight: exactly the registered arms, no others.
        if !registered
            || self.arms.len() != SCHEDULES.len() * SEEDS.len()
            || pairs.len() != self.arms.len()
        {
            return Err(invalid(
                "release must list each registered arm exactly once",
            ));
        }
        let arm = self
            .arms
            .iter()
            .find(|a| a.schedule == schedule && a.seed == seed)
            .ok_or_else(|| invalid("this arm is not released"))?;
        if arm.lineage_sha256 != lineage_sha256 {
            return Err(invalid("arm lineage differs from the released lineage"));
        }
        Ok(())
    }

    /// The fresh corpus must be exactly the sealed slice.
    pub fn validate_fresh(&self, fresh: &PreparedCorpus) -> Result<()> {
        let binding = fresh.binding();
        if binding.source.split != Split::Fresh
            || binding.source.tokens.sha256 != self.fresh.tokens_sha256
            || binding.source.mask.sha256 != self.fresh.mask_sha256
            || binding.eligible_targets != self.fresh.kept_targets
        {
            return Err(invalid("fresh corpus differs from the sealed slice"));
        }
        Ok(())
    }
}

/// The §4.4 count tables, built from TRAIN's first `visited` eligible targets
/// only: the unigram, and the previous-token next-word table with the unigram
/// as its fallback (W2's tables and tie rule). Host counting over corpus data.
pub fn count_tables(
    train: &PreparedCorpus,
    visited: usize,
    scored: &PreparedCorpus,
) -> Result<Value> {
    let row = |tokens: &[u32], p: usize| -> Result<TrainRow> {
        let narrow = |id: u32| u16::try_from(id).map_err(|_| invalid("token ID exceeds u16"));
        Ok(TrainRow {
            code: 0,
            query_token: narrow(tokens[p - 1])?,
            target_token: narrow(tokens[p])?,
        })
    };
    let positions = train.eligible_positions();
    if visited == 0 || visited > positions.len() {
        return Err(invalid("count tables need visited TRAIN targets"));
    }
    let rows = positions[..visited]
        .iter()
        .map(|&p| row(train.tokens(), p))
        .collect::<Result<Vec<_>>>()?;
    let tables = build_tables(&rows).map_err(|e| invalid(format!("count tables: {e:?}")))?;
    let unigram = tables.unigram.ranked();
    let (mut u1, mut u8, mut g1, mut g8, mut fallback) = (0u64, 0u64, 0u64, 0u64, 0u64);
    for &p in scored.eligible_positions() {
        let r = row(scored.tokens(), p)?;
        u1 += u64::from(unigram.first() == Some(&r.target_token));
        u8 += u64::from(unigram.contains(&r.target_token));
        let g = predict(&tables.bigram, r.query_token, &tables.unigram);
        g1 += u64::from(g.hit1(r.target_token));
        g8 += u64::from(g.hit8(r.target_token));
        fallback += u64::from(g.fallback_used);
    }
    Ok(json!({
        "train_rows": visited,
        "scored_rows": scored.eligible_len(),
        "unigram_top1": u1,
        "unigram_top8": u8,
        "next_word_top1": g1,
        "next_word_top8": g8,
        "next_word_fallback_rows": fallback,
    }))
}

/// Sheet §5: `a` beats `b` on the same `n` rows when `a − b` exceeds the
/// registered seed noise `margin` and `(a − b)²·n > 8·h·(n − h)` with
/// `h = ⌊(a + b)/2⌋`: two sigma of the difference of two binomial counts at the
/// pooled rate. Treating the arms as independent is conservative for arms
/// scored on the same rows.
pub fn beats(a: u64, b: u64, n: u64, margin: u64) -> bool {
    if a <= b || n == 0 || a - b <= margin {
        return false;
    }
    let (a, b, n) = (u128::from(a), u128::from(b), u128::from(n));
    let h = (a + b) / 2;
    (a - b) * (a - b) * n > 8 * h * n.saturating_sub(h)
}

/// Sheet §6 check 6: whether ON's selections are informative. Read from the
/// receipts of one seed's ON and control arms, which see the same batch at
/// each ordinal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionAudit {
    pub seed: u32,
    pub vector_batches: u64,
    /// ON batches whose credit was zero, so the lowest-target fallback chose.
    pub on_zero_credit: u64,
    pub on_equals_control_u: u64,
    pub on_equals_control_f: u64,
    /// Descriptive: how many distinct tokens ON chose, and how often its most
    /// chosen token won (an ON stuck on one token passes the bounds above).
    pub on_distinct_tokens: u64,
    pub on_top_token_batches: u64,
}

impl SelectionAudit {
    /// The pre-stated instrument bound: ON is vacuous when more than half its
    /// selections fell back, or more than 90% equal either control's.
    pub fn is_instrument_fault(&self) -> bool {
        let v = u128::from(self.vector_batches);
        v == 0
            || u128::from(self.on_zero_credit) * 2 > v
            || u128::from(self.on_equals_control_u) * 10 > v * 9
            || u128::from(self.on_equals_control_f) * 10 > v * 9
    }
}

/// Compare one seed's ON, CONTROL-U and CONTROL-F receipts batch by batch.
pub fn audit_selections(
    seed: u32,
    on: &[crate::training::StepReceipt],
    control_u: &[crate::training::StepReceipt],
    control_f: &[crate::training::StepReceipt],
) -> Result<SelectionAudit> {
    if on.len() != control_u.len() || on.len() != control_f.len() || on.is_empty() {
        return Err(invalid("the audited arms ran different doses"));
    }
    let mut audit = SelectionAudit {
        seed,
        vector_batches: 0,
        on_zero_credit: 0,
        on_equals_control_u: 0,
        on_equals_control_f: 0,
        on_distinct_tokens: 0,
        on_top_token_batches: 0,
    };
    let mut chosen = std::collections::BTreeMap::<u32, u64>::new();
    for ((a, u), f) in on.iter().zip(control_u).zip(control_f) {
        let n = a.cursor_before.completed_batches;
        if u.cursor_before.completed_batches != n
            || f.cursor_before.completed_batches != n
            || u.target_positions != a.target_positions
            || f.target_positions != a.target_positions
        {
            return Err(invalid("audited receipts are not aligned batch by batch"));
        }
        match (a.carrier, u.carrier, f.carrier) {
            (Some(a), Some(u), Some(f)) => {
                use crate::carrier::CarrierMode::*;
                if (a.mode, u.mode, f.mode) != (CreditV1, UniformTargetV1, UniformRowV1) {
                    return Err(invalid("audited receipts come from the wrong arms"));
                }
                audit.vector_batches += 1;
                audit.on_zero_credit += u64::from(a.value == 0);
                audit.on_equals_control_u += u64::from(a.token == u.token);
                audit.on_equals_control_f += u64::from(a.token == f.token);
                *chosen.entry(a.token).or_default() += 1;
            }
            (None, None, None) => {}
            _ => return Err(invalid("a batch is a vector batch in one arm only")),
        }
    }
    audit.on_distinct_tokens = chosen.len() as u64;
    audit.on_top_token_batches = chosen.values().copied().max().unwrap_or(0);
    Ok(audit)
}

/// One arm's stored fresh and DEV top-1 counts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArmScore {
    pub schedule: SchedulePolicy,
    pub seed: u32,
    pub fresh_top1: u64,
    pub fresh_rows: u64,
    pub dev_top1: u64,
    pub dev_rows: u64,
}

fn outcome(per_seed: &[bool]) -> &'static str {
    match per_seed.iter().filter(|&&c| c).count() {
        n if n == per_seed.len() => "win",
        0 => "tie_or_loss",
        _ => "split",
    }
}

/// The two pre-named comparisons of sheet §5, per seed on fresh rows, with the
/// same comparisons on DEV reported for the panel-fit label. The seed noise is
/// OFF's spread across the two seeds, on each scored set. A selection audit
/// that finds ON vacuous at either seed makes the credit outcome an instrument
/// fault (§4.2), with no verdict.
pub fn grade(scores: &[ArmScore], audits: &[SelectionAudit]) -> Result<Value> {
    let find = |schedule: SchedulePolicy, seed: u32| {
        scores
            .iter()
            .find(|s| s.schedule == schedule && s.seed == seed)
            .ok_or_else(|| invalid(format!("missing arm {schedule:?} seed {seed}")))
    };
    if scores.len() != SCHEDULES.len() * SEEDS.len() {
        return Err(invalid("grading needs exactly the eight registered arms"));
    }
    if audits.len() != SEEDS.len() || SEEDS.iter().any(|s| !audits.iter().any(|a| a.seed == *s)) {
        return Err(invalid("grading needs one selection audit per seed"));
    }
    let off_a = find(SchedulePolicy::InterleavedReadEvery8V1, SEEDS[0])?;
    let off_b = find(SchedulePolicy::InterleavedReadEvery8V1, SEEDS[1])?;
    let noise = off_a.fresh_top1.abs_diff(off_b.fresh_top1);
    let noise_dev = off_a.dev_top1.abs_diff(off_b.dev_top1);
    let (mut credit, mut selection, mut credit_dev, mut selection_dev) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut seeds = Vec::new();
    for seed in SEEDS {
        let on = find(SchedulePolicy::CarrierCreditV1, seed)?;
        let cu = find(SchedulePolicy::CarrierUniformTargetV1, seed)?;
        let cf = find(SchedulePolicy::CarrierUniformRowV1, seed)?;
        let off = find(SchedulePolicy::InterleavedReadEvery8V1, seed)?;
        let rows: Vec<u64> = [on, cu, cf, off].iter().map(|s| s.fresh_rows).collect();
        let dev_rows: Vec<u64> = [on, cu, cf, off].iter().map(|s| s.dev_rows).collect();
        if rows.iter().any(|&n| n != rows[0]) || dev_rows.iter().any(|&n| n != dev_rows[0]) {
            return Err(invalid("arms of one seed were scored on different rows"));
        }
        let (n, d) = (rows[0], dev_rows[0]);
        let c = beats(on.fresh_top1, cu.fresh_top1, n, noise)
            && beats(on.fresh_top1, cf.fresh_top1, n, noise);
        let s = beats(cf.fresh_top1, off.fresh_top1, n, noise);
        let cd = beats(on.dev_top1, cu.dev_top1, d, noise_dev)
            && beats(on.dev_top1, cf.dev_top1, d, noise_dev);
        let sd = beats(cf.dev_top1, off.dev_top1, d, noise_dev);
        credit.push(c);
        selection.push(s);
        credit_dev.push(cd);
        selection_dev.push(sd);
        seeds.push(json!({
            "seed": seed,
            "fresh_rows": n,
            "fresh_top1": {"on": on.fresh_top1, "control_u": cu.fresh_top1, "control_f": cf.fresh_top1, "off": off.fresh_top1},
            "fresh_gaps": {
                "on_minus_control_u": i128::from(on.fresh_top1) - i128::from(cu.fresh_top1),
                "on_minus_control_f": i128::from(on.fresh_top1) - i128::from(cf.fresh_top1),
                "control_f_minus_off": i128::from(cf.fresh_top1) - i128::from(off.fresh_top1),
                "on_minus_off": i128::from(on.fresh_top1) - i128::from(off.fresh_top1),
                "binomial_only": {
                    "on_beats_control_u": beats(on.fresh_top1, cu.fresh_top1, n, 0),
                    "on_beats_control_f": beats(on.fresh_top1, cf.fresh_top1, n, 0),
                    "control_f_beats_off": beats(cf.fresh_top1, off.fresh_top1, n, 0),
                },
            },
            "credit_counts": c,
            "selection_counts": s,
            "on_beats_off": beats(on.fresh_top1, off.fresh_top1, n, noise),
            "dev_credit_counts": cd,
            "dev_selection_counts": sd,
        }));
    }
    let label = |fresh: &[bool], dev: &[bool]| {
        if outcome(dev) == "win" && outcome(fresh) != "win" {
            "panel_fit"
        } else {
            "none"
        }
    };
    let vacuous = audits.iter().any(SelectionAudit::is_instrument_fault);
    Ok(json!({
        "schema": "uor-r4.carrier-arms-grade/1",
        "rule": "a beats b when a - b > seed noise and (a-b)^2*n > 8*h*(n-h), h = floor((a+b)/2); seed noise = |OFF(1001) - OFF(2001)| on the same set; fresh rows decide",
        "seed_noise": {"fresh": noise, "dev": noise_dev},
        "selection_audits": audits,
        "seeds": seeds,
        "credit": if vacuous { "instrument" } else { outcome(&credit) },
        "selection": outcome(&selection),
        "credit_label": if vacuous { "none" } else { label(&credit, &credit_dev) },
        "selection_label": label(&selection, &selection_dev),
    }))
}

/// Train one released arm and score it on DEV and the sealed fresh slice.
/// Receipts stream to `receipts`, one JSON line per batch.
#[cfg(feature = "cuda")]
pub fn run_arm(
    driver: &crate::training::PreparedDriver,
    release: &CarrierRelease,
    sheet_bytes: &[u8],
    fresh: &PreparedCorpus,
    batches: u64,
    ordinal: usize,
    receipts: &mut dyn std::io::Write,
) -> Result<Value> {
    let config = driver.config();
    release.validate_for(
        sheet_bytes,
        &driver.lineage_sha256()?,
        config.schedule,
        config.seed,
        config.batch_rows,
    )?;
    release.validate_fresh(fresh)?;
    if batches == 0 || batches > release.max_completed_batches {
        return Err(invalid("batches must be 1..=the released dose"));
    }
    let admission =
        crate::training::issue_carrier_arm_admission(driver, release.max_completed_batches)?;
    let visited = release.fresh.visited_targets;
    let mut model = driver.new_model(ordinal)?;
    let mut cursor = driver.initial_cursor();
    let started = std::time::Instant::now();
    let (mut accepted, mut selected) = (0u64, 0u64);
    while cursor.completed_batches < batches {
        if cursor.next_target + u64::from(config.batch_rows) > visited {
            return Err(invalid("the dose would reach the fresh slice"));
        }
        let receipt = driver.transition(&mut model, &cursor, &admission)?;
        serde_json::to_writer(&mut *receipts, &receipt).map_err(|e| invalid(e.to_string()))?;
        receipts
            .write_all(b"\n")
            .map_err(|e| invalid(e.to_string()))?;
        accepted += u64::from(receipt.applied);
        selected += u64::from(receipt.carrier.is_some());
        cursor = receipt.cursor_after;
    }
    let train_ns = started.elapsed().as_nanos();
    let targets = cursor.next_target;
    let mut record = json!({
        "schema": RECORD_SCHEMA,
        "schedule": config.schedule,
        "seed": config.seed,
        "lineage_sha256": driver.lineage_sha256()?,
        "release_sha256": corpus::sha256(&serde_json::to_vec(release).map_err(|e| invalid(e.to_string()))?),
        "completed_batches": cursor.completed_batches,
        "trained_targets": targets,
        "accepted_updates": accepted,
        "carrier_selected_batches": selected,
        "train_wall_ns": train_ns.to_string(),
        "train_targets_per_second": (u128::from(targets) * 1_000_000_000 / train_ns.max(1)).to_string(),
        "scored": false,
    });
    // Sheet §4: the fresh slice is scored once, after the released dose. A
    // shorter run is a throughput probe and scores nothing.
    if cursor.completed_batches == release.max_completed_batches {
        let dev = driver.evaluate(&model)?;
        let fresh_eval = driver.evaluate_transfer(&model, fresh)?;
        let visited_now = usize::try_from(targets).map_err(|_| invalid("target count"))?;
        let tables_dev = count_tables(driver.train(), visited_now, driver.dev())?;
        let tables_fresh = count_tables(driver.train(), visited_now, fresh)?;
        record["scored"] = json!(true);
        record["dev"] = dev["aggregate"].clone();
        record["fresh"] = fresh_eval["aggregate"].clone();
        record["count_tables"] = json!({"dev": tables_dev, "fresh": tables_fresh});
    }
    Ok(record)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_win_rule_is_two_sigma_of_a_difference_above_the_seed_noise() {
        assert!(beats(1500, 1300, 65_536, 0));
        assert!(!beats(1340, 1300, 65_536, 0));
        assert!(!beats(1300, 1300, 65_536, 0));
        assert!(!beats(1300, 1500, 65_536, 0));
        assert!(!beats(5, 0, 0, 0));
        // At h = 1350, 8*h*(n-h) = 693,208,800: a gap of 103 gives 695,271,424
        // and clears it; a gap of 102 gives 681,836,544 and does not.
        assert!(beats(1350 + 52, 1350 - 51, 65_536, 0));
        assert!(!beats(1350 + 51, 1350 - 51, 65_536, 0));
        // The seed noise is a strict floor on the gap.
        assert!(!beats(1500, 1300, 65_536, 200));
        assert!(beats(1500, 1300, 65_536, 199));
    }

    fn audit(seed: u32, zero: u64, same_u: u64, same_f: u64) -> SelectionAudit {
        SelectionAudit {
            seed,
            vector_batches: 100,
            on_zero_credit: zero,
            on_equals_control_u: same_u,
            on_equals_control_f: same_f,
            on_distinct_tokens: 40,
            on_top_token_batches: 9,
        }
    }

    fn audits() -> Vec<SelectionAudit> {
        SEEDS.iter().map(|&s| audit(s, 10, 5, 5)).collect()
    }

    #[test]
    fn a_vacuous_on_arm_is_an_instrument_fault() {
        assert!(!audit(1001, 50, 90, 90).is_instrument_fault());
        assert!(audit(1001, 51, 0, 0).is_instrument_fault());
        assert!(audit(1001, 0, 91, 0).is_instrument_fault());
        assert!(audit(1001, 0, 0, 91).is_instrument_fault());
    }

    fn receipt(
        n: u64,
        selection: Option<(u32, u64)>,
        mode: crate::carrier::CarrierMode,
    ) -> crate::training::StepReceipt {
        use crate::training::{Coordinate, CoordinateKind, Cursor, StepReceipt};
        let mut cursor = Cursor::initial();
        cursor.completed_batches = n;
        StepReceipt {
            lineage_sha256: String::new(),
            cursor_before: cursor,
            cursor_after: cursor,
            coordinate: Coordinate {
                kind: CoordinateKind::VectorBlock,
                index: 0,
            },
            target_positions: vec![n],
            candidate: 0,
            loss_before: 0,
            loss_after: 0,
            model_epoch: 0,
            applied: false,
            apply_refused_by_policy: false,
            any_candidate_cost_differs_from_identity: false,
            carrier: selection.map(|(token, value)| crate::carrier::CarrierSelection {
                mode,
                token,
                value,
            }),
        }
    }

    #[test]
    fn the_audit_compares_selections_batch_by_batch() {
        use crate::carrier::CarrierMode::*;
        let arm = |picks: [Option<(u32, u64)>; 4], mode| -> Vec<_> {
            picks
                .iter()
                .enumerate()
                .map(|(n, p)| receipt(n as u64, *p, mode))
                .collect()
        };
        let on = arm([Some((5, 3)), Some((5, 0)), Some((7, 2)), None], CreditV1);
        let cu = arm(
            [Some((5, 3)), Some((9, 3)), Some((1, 3)), None],
            UniformTargetV1,
        );
        let cf = arm(
            [Some((2, 0)), Some((5, 1)), Some((7, 2)), None],
            UniformRowV1,
        );
        let a = audit_selections(1001, &on, &cu, &cf).expect("audit");
        assert_eq!(
            (
                a.vector_batches,
                a.on_zero_credit,
                a.on_equals_control_u,
                a.on_equals_control_f
            ),
            (3, 1, 1, 2)
        );
        assert_eq!((a.on_distinct_tokens, a.on_top_token_batches), (2, 2));
        assert!(!a.is_instrument_fault());
        // Receipts from the wrong arm are refused, not audited.
        assert!(audit_selections(1001, &on, &cf, &cu).is_err());
        let mut shifted = cf.clone();
        shifted[0].target_positions = vec![99];
        assert!(audit_selections(1001, &on, &cu, &shifted).is_err());
        let mut lopsided = cf;
        lopsided[3].carrier = lopsided[2].carrier;
        assert!(audit_selections(1001, &on, &cu, &lopsided).is_err());
    }

    fn arm(schedule: SchedulePolicy, seed: u32, fresh: u64, dev: u64) -> ArmScore {
        ArmScore {
            schedule,
            seed,
            fresh_top1: fresh,
            fresh_rows: 65_536,
            dev_top1: dev,
            dev_rows: 14_287,
        }
    }

    #[test]
    fn grading_reads_credit_and_selection_per_seed() {
        use SchedulePolicy::*;
        let mut scores = Vec::new();
        for seed in SEEDS {
            scores.push(arm(CarrierCreditV1, seed, 2000, 500));
            scores.push(arm(CarrierUniformTargetV1, seed, 1500, 400));
            scores.push(arm(
                CarrierUniformRowV1,
                seed,
                if seed == 1001 { 1700 } else { 1500 },
                400,
            ));
            scores.push(arm(InterleavedReadEvery8V1, seed, 1500, 400));
        }
        let g = grade(&scores, &audits()).expect("grade");
        assert_eq!(g["credit"], "win");
        // CONTROL-F beats OFF at seed 1001 only: a split.
        assert_eq!(g["selection"], "split");
        assert_eq!(g["credit_label"], "none");
        assert!(grade(&scores[..7], &audits()).is_err());
        assert!(grade(&scores, &audits()[..1]).is_err());
        // A vacuous ON at either seed turns the credit verdict into an instrument fault.
        let mut faulty = audits();
        faulty[1].on_zero_credit = 60;
        assert_eq!(
            grade(&scores, &faulty).expect("grade")["credit"],
            "instrument"
        );
        // OFF's seed spread is the noise floor: 500 here equals ON's 500-hit lead
        // over CONTROL-U at both seeds, so neither seed counts.
        let mut noisy = scores.clone();
        for s in noisy
            .iter_mut()
            .filter(|s| s.schedule == InterleavedReadEvery8V1 && s.seed == 2001)
        {
            s.fresh_top1 = 1000;
        }
        let g = grade(&noisy, &audits()).expect("grade");
        assert_eq!(g["seed_noise"]["fresh"], 500);
        assert_eq!(g["credit"], "tie_or_loss");
    }

    #[test]
    fn a_dev_only_win_is_labelled_panel_fit() {
        use SchedulePolicy::*;
        let mut scores = Vec::new();
        for seed in SEEDS {
            scores.push(arm(CarrierCreditV1, seed, 1500, 600));
            scores.push(arm(CarrierUniformTargetV1, seed, 1500, 400));
            scores.push(arm(CarrierUniformRowV1, seed, 1500, 400));
            scores.push(arm(InterleavedReadEvery8V1, seed, 1500, 400));
        }
        let g = grade(&scores, &audits()).expect("grade");
        assert_eq!(g["credit"], "tie_or_loss");
        assert_eq!(g["credit_label"], "panel_fit");
        // A vacuous ON arm voids the label too, not only the verdict.
        let mut faulty = audits();
        faulty[1].on_zero_credit = 60;
        let g = grade(&scores, &faulty).expect("grade");
        assert_eq!(g["credit"], "instrument");
        assert_eq!(g["credit_label"], "none");
    }

    fn release() -> CarrierRelease {
        let arms = SEEDS
            .iter()
            .flat_map(|&seed| {
                SCHEDULES.iter().map(move |&schedule| ReleaseArm {
                    schedule,
                    seed,
                    lineage_sha256: format!("{schedule:?}-{seed}"),
                })
            })
            .collect();
        CarrierRelease {
            schema: RELEASE_SCHEMA.into(),
            item: ITEM.into(),
            issue: ISSUE,
            sheet: SheetPin {
                path: "sheet.md".into(),
                sha256: corpus::sha256(b"sheet"),
            },
            fit_url: format!("{COMMENT_PREFIX}1"),
            prerun_url: format!("{COMMENT_PREFIX}2"),
            max_completed_batches: DOSE_BATCHES,
            fresh: FreshPin {
                tokens_sha256: "t".into(),
                mask_sha256: "m".into(),
                visited_targets: DOSE_BATCHES * 64,
                kept_targets: 65_536,
            },
            arms,
        }
    }

    #[test]
    fn the_release_admits_only_its_own_arms_at_their_lineage() {
        let r = release();
        let on = SchedulePolicy::CarrierCreditV1;
        let ok = |r: &CarrierRelease, sheet: &[u8], lineage: &str, rows: u32| {
            r.validate_for(sheet, lineage, on, 1001, rows).is_ok()
        };
        assert!(ok(&r, b"sheet", "CarrierCreditV1-1001", 64));
        assert!(!ok(&r, b"edited sheet", "CarrierCreditV1-1001", 64));
        assert!(!ok(&r, b"sheet", "another lineage", 64));
        assert!(!ok(&r, b"sheet", "CarrierCreditV1-1001", 32));
        let mut longer = r.clone();
        longer.max_completed_batches += 1;
        assert!(!ok(&longer, b"sheet", "CarrierCreditV1-1001", 64));
        let mut unlinked = r.clone();
        unlinked.fit_url = "https://example.com/fit".into();
        assert!(!ok(&unlinked, b"sheet", "CarrierCreditV1-1001", 64));
        let mut missing = r.clone();
        missing.arms.pop();
        assert!(!ok(&missing, b"sheet", "CarrierCreditV1-1001", 64));
        let mut doubled = r.clone();
        let first = doubled.arms[0].clone();
        doubled.arms[7] = first;
        assert!(!ok(&doubled, b"sheet", "CarrierCreditV1-1001", 64));
        // Eight distinct arms, one unregistered: refused (exact-head review, slice 4).
        let mut stranger = r;
        stranger.arms[7].seed = 3001;
        assert!(!ok(&stranger, b"sheet", "CarrierCreditV1-1001", 64));
    }
}
