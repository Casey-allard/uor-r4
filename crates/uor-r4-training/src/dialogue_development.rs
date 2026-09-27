//! Fixed source-stratified open-development episodes, independent of training
//! sampling. Selection uses only the bound inventory and declared seed.
//! Evaluation observes original full prefixes without gradients or updates.

use std::collections::BTreeSet;
use std::time::Instant;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::dialogue_episodes::{EpisodeBatch, EpisodeIndex, PrefixPolicy};
use crate::joint_model::{JointModel, ReadMode};
use crate::{invalid, Result};

pub const SELECTION_ID: &str = "uor-r4.dialogue-development/source-sha256-rank-v1";
const MAX_PER_SOURCE: usize = 32;

/// Select up to `per_source` eligible responses from every source, without
/// replacement. Short sources contribute every eligible response. Output is
/// source-index order, then ascending (SHA256 rank, response ID).
///
/// Rank bytes are SELECTION_ID || seed:u64-LE || source-index:u64-LE ||
/// eligible-response-ID:u64-LE. The bound inventory supplies the source order
/// and IDs; source labels never enter the model input or loss.
pub fn select(index: &EpisodeIndex<'_>, seed: u64, per_source: usize) -> Result<Vec<usize>> {
    if !(1..=MAX_PER_SOURCE).contains(&per_source) {
        return Err(invalid("development responses per source must be 1..32"));
    }
    let mut ranks = vec![Vec::new(); index.sources().len()];
    for span in index.episodes() {
        let mut hash = Sha256::new();
        hash.update(SELECTION_ID.as_bytes());
        hash.update(seed.to_le_bytes());
        hash.update((span.source_index as u64).to_le_bytes());
        hash.update((span.response_id as u64).to_le_bytes());
        let rank: [u8; 32] = hash.finalize().into();
        ranks[span.source_index].push((rank, span.response_id));
    }
    let mut selected = Vec::new();
    for source in &mut ranks {
        source.sort_unstable();
        selected.extend(source.iter().take(per_source).map(|(_, id)| *id));
    }
    validate(index, &selected, per_source)?;
    Ok(selected)
}

/// Verify IDs and the exact min(per_source, eligible) count for every source.
/// This validates a supplied inventory, not its seed/rank derivation; callers
/// retain the seed and the exact IDs returned by `select` in their receipts.
pub fn validate(index: &EpisodeIndex<'_>, ids: &[usize], per_source: usize) -> Result<()> {
    if !(1..=MAX_PER_SOURCE).contains(&per_source) || ids.is_empty() {
        return Err(invalid("empty or unsupported development selection"));
    }
    let mut seen = BTreeSet::new();
    let mut counts = vec![0usize; index.sources().len()];
    for &id in ids {
        let span = index
            .episodes()
            .get(id)
            .ok_or_else(|| invalid("development response ID outside bound inventory"))?;
        if !seen.insert(id) {
            return Err(invalid("duplicate development response ID"));
        }
        counts[span.source_index] += 1;
    }
    for (source_index, &count) in counts.iter().enumerate() {
        let eligible = index.population().sources[source_index].eligible_responses;
        if count != per_source.min(eligible) {
            return Err(invalid(
                "development source count differs from declared quota",
            ));
        }
    }
    Ok(())
}

#[derive(Default, Clone)]
struct Totals {
    responses: usize,
    targets: usize,
    nll_sum: f64,
    first_targets: usize,
    first_nll_sum: f64,
    eos_targets: usize,
}

impl Totals {
    fn add(&mut self, masks: &SourceMasks, mean: f32, first_mean: f32) -> Result<()> {
        if masks.targets == 0
            || masks.first_targets == 0
            || !mean.is_finite()
            || !first_mean.is_finite()
        {
            return Err(invalid("empty or nonfinite source development loss"));
        }
        self.responses += masks.responses;
        self.targets += masks.targets;
        self.nll_sum += f64::from(mean) * masks.targets as f64;
        self.first_targets += masks.first_targets;
        self.first_nll_sum += f64::from(first_mean) * masks.first_targets as f64;
        self.eos_targets += masks.eos_targets;
        Ok(())
    }

    fn merge(&mut self, other: &Self) {
        self.responses += other.responses;
        self.targets += other.targets;
        self.nll_sum += other.nll_sum;
        self.first_targets += other.first_targets;
        self.first_nll_sum += other.first_nll_sum;
        self.eos_targets += other.eos_targets;
    }

    fn report(&self) -> Value {
        json!({
            "selected_responses":self.responses,
            "supervised_targets":self.targets,
            "eos_targets":self.eos_targets,
            "response_mean_nll":(self.targets != 0).then(||self.nll_sum/self.targets as f64),
            "first_four_targets":self.first_targets,
            "first_four_response_targets_mean_nll":(self.first_targets != 0)
                .then(||self.first_nll_sum/self.first_targets as f64),
        })
    }
}

struct SourceMasks {
    all: Vec<f32>,
    first: Vec<f32>,
    responses: usize,
    targets: usize,
    first_targets: usize,
    eos_targets: usize,
}

fn source_masks(batch: &EpisodeBatch, source_index: usize) -> SourceMasks {
    let mut masks = SourceMasks {
        all: vec![0.0; batch.weights.len()],
        first: vec![0.0; batch.weights.len()],
        responses: 0,
        targets: 0,
        first_targets: 0,
        eos_targets: 0,
    };
    for (lane, row) in batch.rows.iter().enumerate() {
        if row.source_index != source_index {
            continue;
        }
        masks.responses += 1;
        masks.eos_targets += row.counts.eos_targets;
        let start = lane * batch.time;
        let mut first_in_row = 0;
        for (offset, &weight) in batch.weights[start..start + batch.time].iter().enumerate() {
            if weight == 1.0 {
                masks.all[start + offset] = 1.0;
                masks.targets += 1;
                if first_in_row < 4 {
                    masks.first[start + offset] = 1.0;
                    masks.first_targets += 1;
                    first_in_row += 1;
                }
            }
        }
    }
    masks
}

/// Score the fixed supplied IDs under full original prefixes. One forward is
/// shared across source-specific masks in each chunk. Every reported aggregate
/// is a token mean over this selected panel, not a corpus estimate or an equal
/// mean of source means. First-four targets include EOS for short responses.
pub fn evaluate(
    model: &JointModel,
    index: &EpisodeIndex<'_>,
    ids: &[usize],
    batch_size: usize,
) -> Result<Value> {
    if !(1..=16).contains(&batch_size)
        || model.config.context != index.contract().context
        || model.config.vocab_size != index.contract().vocab_size
    {
        return Err(invalid(
            "development model/context/vocabulary/batch mismatch",
        ));
    }
    // Infer the effective quota when every source is shorter than the declared
    // cap. A caller with that cap available should also call validate explicitly.
    let mut selected_ids = vec![Vec::new(); index.sources().len()];
    for &id in ids {
        let span = index
            .episodes()
            .get(id)
            .ok_or_else(|| invalid("development response ID outside bound inventory"))?;
        selected_ids[span.source_index].push(id);
    }
    let quota = selected_ids.iter().map(Vec::len).max().unwrap_or(0);
    validate(index, ids, quota)?;
    let started = Instant::now();
    let mut totals = vec![Totals::default(); index.sources().len()];
    for chunk in ids.chunks(batch_size) {
        let batch = index.materialize(chunk, PrefixPolicy::FullPrefix)?;
        let output = model.forward(
            &batch.inputs,
            batch.batch,
            batch.time,
            ReadMode::Enabled,
            false,
        )?;
        let active: BTreeSet<_> = batch.rows.iter().map(|row| row.source_index).collect();
        for source_index in active {
            let masks = source_masks(&batch, source_index);
            let mean = output
                .weighted_loss(&batch.targets, &masks.all)?
                .to_scalar::<f32>()?;
            let first = output
                .weighted_loss(&batch.targets, &masks.first)?
                .to_scalar::<f32>()?;
            totals[source_index].add(&masks, mean, first)?;
        }
    }
    let mut pooled = Totals::default();
    let mut sources = Vec::with_capacity(totals.len());
    for (source_index, total) in totals.iter().enumerate() {
        pooled.merge(total);
        let eligible = index.population().sources[source_index].eligible_responses;
        let mut report = total.report();
        report["source_index"] = json!(source_index);
        report["label"] = json!(index.sources()[source_index].label);
        report["eligible_responses"] = json!(eligible);
        report["response_ids"] = json!(selected_ids[source_index]);
        report["status"] = json!(if eligible == 0 {
            "UNAVAILABLE_NO_ELIGIBLE_RESPONSES"
        } else {
            "MEASURED"
        });
        sources.push(report);
    }
    let mut report = pooled.report();
    report["schema"] = json!("uor-r4.dialogue-source-development/1");
    report["response_ids"] = json!(ids);
    report["per_source"] = json!(sources);
    report["conditioning"] = json!("full_original_prefix");
    report["elapsed_seconds"] = json!(started.elapsed().as_secs_f64());
    report["scope"] = json!("Fixed source-stratified exposed development responses. Pooled NLL is the selected panel's token mean, not the full corpus or an equal-source mean; no fresh held-out capability claim.");
    report["arithmetic"] = json!("Per-source/chunk weighted_loss uses F32 reductions; means are multiplied by actual target counts and accumulated in F64. No equal-source or equal-chunk averaging; bitwise equivalence to a single F32 reduction is not claimed.");
    report["first_four_scope"] = json!("First min(4,response_length) supervised response tokens including genuine EOS when it occurs within those positions.");
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialogue_episodes::{EpisodeContract, SourceSpan};

    fn fixture() -> (Vec<u16>, Vec<u8>, EpisodeContract, Vec<SourceSpan>) {
        let mut tokens = Vec::new();
        let mut masks = Vec::new();
        let mut sources = Vec::new();
        // More than32 eligible rows, a short source, then no eligible rows.
        for (source, count) in [33, 2, 0].into_iter().enumerate() {
            let start = tokens.len();
            for _ in 0..count {
                if source == 0 {
                    tokens.extend([0, 7, 8, 20, 1]);
                    masks.extend([0, 0, 0, 1, 1]);
                } else {
                    tokens.extend([0, 10, 7, 8, 21, 22, 23, 24, 1]);
                    masks.extend([0, 0, 0, 0, 1, 1, 1, 1, 1]);
                }
            }
            if source == 2 {
                let mut long = vec![10u16; 257];
                long[0] = 0;
                long[253..].copy_from_slice(&[7, 8, 25, 1]);
                let mut long_mask = vec![0u8; 257];
                long_mask[255..].fill(1);
                tokens.extend(long);
                masks.extend(long_mask);
            }
            sources.push(SourceSpan {
                label: format!("source-{source}"),
                start,
                end: tokens.len(),
            });
        }
        let contract = EpisodeContract {
            context: 256,
            vocab_size: 64,
            bos_id: 0,
            eos_id: 1,
            unk_id: 2,
            padding_id: 2,
            assistant_marker_ids: vec![7, 8],
        };
        (tokens, masks, contract, sources)
    }

    #[test]
    fn dialogue_development_selection_is_unique_deterministic_and_keeps_short_sources() -> Result<()>
    {
        let (tokens, masks, contract, sources) = fixture();
        let index = EpisodeIndex::new(&tokens, &masks, contract, &sources)?;
        let ids = select(&index, 240927, 32)?;
        assert_eq!(ids, select(&index, 240927, 32)?);
        assert_eq!(ids.len(), 34);
        assert_eq!(ids.iter().copied().collect::<BTreeSet<_>>().len(), 34);
        assert!(ids[..32]
            .iter()
            .all(|&id| index.episodes()[id].source_index == 0));
        assert_eq!(
            ids[32..].iter().copied().collect::<BTreeSet<_>>(),
            BTreeSet::from([33, 34])
        );
        assert_eq!(index.population().sources[2].eligible_responses, 0);
        validate(&index, &ids, 32)?;
        Ok(())
    }

    #[test]
    fn dialogue_development_rejects_duplicate_outside_and_unbalanced_ids() -> Result<()> {
        let (tokens, masks, contract, sources) = fixture();
        let index = EpisodeIndex::new(&tokens, &masks, contract, &sources)?;
        let ids = select(&index, 0, 32)?;
        let mut duplicate = ids.clone();
        duplicate[1] = duplicate[0];
        assert!(validate(&index, &duplicate, 32).is_err());
        let mut outside = ids.clone();
        outside[0] = index.episodes().len();
        assert!(validate(&index, &outside, 32).is_err());
        assert!(validate(&index, &ids[..33], 32).is_err());
        assert!(select(&index, 0, 0).is_err());
        assert!(select(&index, 0, 33).is_err());
        assert!(validate(&index, &[], 32).is_err());
        Ok(())
    }

    #[test]
    fn dialogue_development_masks_and_reduction_use_response_target_counts() -> Result<()> {
        let (tokens, masks, contract, sources) = fixture();
        let index = EpisodeIndex::new(&tokens, &masks, contract, &sources)?;
        let batch = index.materialize(&[0, 33, 34], PrefixPolicy::FullPrefix)?;
        let small = source_masks(&batch, 0);
        let large = source_masks(&batch, 1);
        let empty = source_masks(&batch, 2);
        assert_eq!(
            (small.responses, small.targets, small.first_targets),
            (1, 2, 2)
        );
        assert_eq!(
            (large.responses, large.targets, large.first_targets),
            (2, 10, 8)
        );
        assert_eq!(
            (small.eos_targets, large.eos_targets, empty.targets),
            (1, 2, 0)
        );
        for position in 0..batch.weights.len() {
            assert_eq!(
                small.all[position] + large.all[position],
                batch.weights[position]
            );
            assert!(small.first[position] <= small.all[position]);
            assert!(large.first[position] <= large.all[position]);
        }
        let mut a = Totals::default();
        let mut b = Totals::default();
        a.add(&small, 2.0, 3.0)?;
        b.add(&large, 5.0, 7.0)?;
        a.merge(&b);
        let result = a.report();
        assert_eq!(result["supervised_targets"], 12);
        assert_eq!(result["response_mean_nll"], 4.5);
        assert_eq!(result["first_four_targets"], 10);
        assert_eq!(result["first_four_response_targets_mean_nll"], 6.2);
        assert!(Totals::default().report()["response_mean_nll"].is_null());
        assert!(a.add(&empty, 1.0, 1.0).is_err());
        assert!(a.add(&small, f32::NAN, 1.0).is_err());
        Ok(())
    }
}
