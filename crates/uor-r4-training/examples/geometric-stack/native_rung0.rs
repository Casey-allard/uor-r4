//! Native rung-0 development metrics. All score/count arithmetic stays on CUDA.
//! Host code prepares token/mask indices and serializes final scalar receipts.
use super::invalid;
use candle_core::{DType, Tensor};
use serde_json::{json, Value};
use uor_r4_training::geometric_stack::{Objective, StackModel};
use uor_r4_training::Result;

pub fn window_starts(tokens: usize, context: usize, windows: usize) -> Result<Vec<usize>> {
    if context == 0 || windows == 0 || tokens <= context || windows > (tokens - 1) / context {
        return Err(invalid(
            "native dev windows need complete non-overlapping input/shifted-target blocks",
        ));
    }
    Ok((0..windows).map(|i| i * context).collect())
}

pub fn validate_mask(tokens: usize, mask: &[u8]) -> Result<()> {
    if mask.len() != tokens || mask.iter().any(|&v| v > 1) {
        return Err(invalid(
            "valid_mask must contain one binary response-mask byte per valid token",
        ));
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct Window {
    start: usize,
    time: usize,
    score_from: usize,
}
#[derive(Clone, Debug)]
pub struct PanelPlan {
    windows: Vec<Window>,
    pub scored_targets: usize,
    pub total_mask_targets: usize,
    full_blocks: usize,
    tail_tokens: usize,
    bos_id: u32,
    expected_targets: usize,
}
impl PanelPlan {
    /// Token/mask partitioning is metadata preparation, before any model call.
    pub fn new(
        valid: &[u32],
        mask: &[u8],
        context: usize,
        full_blocks: usize,
        bos_id: u32,
        expected_targets: usize,
    ) -> Result<Self> {
        validate_mask(valid.len(), mask)?;
        if context == 0 || valid.first() != Some(&bos_id) || full_blocks != valid.len() / context {
            return Err(invalid("native dev panel must begin at verified BOS and pin all complete global context blocks"));
        }
        let documents: Vec<usize> = valid
            .iter()
            .enumerate()
            .filter_map(|(i, &id)| (id == bos_id).then_some(i))
            .collect();
        let mut windows = Vec::new();
        let mut scored_targets = 0;
        for (i, &begin) in documents.iter().enumerate() {
            let end = documents.get(i + 1).copied().unwrap_or(valid.len());
            let mut first = begin + 1;
            while first < end {
                let target_end = (first + context).min(end);
                let start = (target_end - 1).saturating_sub(context).max(begin);
                scored_targets += mask[first..target_end].iter().filter(|&&m| m != 0).count();
                windows.push(Window {
                    start,
                    time: target_end - 1 - start,
                    score_from: first,
                });
                first = target_end;
            }
        }
        if scored_targets == 0 {
            return Err(invalid("native development panel has no eligible masked targets with within-document predecessors"));
        }
        Ok(Self {
            windows,
            scored_targets,
            total_mask_targets: mask.iter().filter(|&&m| m != 0).count(),
            full_blocks,
            tail_tokens: valid.len() % context,
            bos_id,
            expected_targets,
        })
    }
}

/// Metadata selection only: no frequency, rank or score is computed on host.
#[derive(Clone, Debug)]
pub struct MaskedTargets {
    ids: Vec<u32>,
    payload_tokens: usize,
    bos_id: u32,
}
impl MaskedTargets {
    pub fn new(tokens: &[u32], mask: &[u8], bos_id: u32) -> Result<Self> {
        validate_mask(tokens.len(), mask)?;
        if tokens.first() != Some(&bos_id) {
            return Err(invalid("count target inventory must begin at verified BOS"));
        }
        let ids = tokens
            .iter()
            .zip(mask)
            .enumerate()
            .filter_map(|(position, (&id, &included))| {
                (position > 0 && id != bos_id && included == 1).then_some(id)
            })
            .collect::<Vec<_>>();
        if ids.is_empty() {
            return Err(invalid(
                "count target inventory has no response-masked within-document targets",
            ));
        }
        Ok(Self {
            ids,
            payload_tokens: tokens.len(),
            bos_id,
        })
    }
    pub fn len(&self) -> usize {
        self.ids.len()
    }
    pub fn payload_tokens(&self) -> usize {
        self.payload_tokens
    }
}

/// Frequency/rank arithmetic is entirely CUDA. Only supplied masked targets
/// enter this table; input/prompt/header/padding identities cannot enter it.
struct CountPredictor {
    count_table: Tensor,
    top_count_ids: Tensor,
    top_count_scores: Tensor,
    reference_targets: usize,
    payload_tokens: usize,
}
impl CountPredictor {
    fn new(model: &StackModel, targets: &MaskedTargets) -> Result<Self> {
        if !model.device().is_cuda()
            || model.config.vocab_size < 9
            || targets
                .ids
                .iter()
                .any(|&id| id as usize >= model.config.vocab_size)
        {
            return Err(invalid(
                "count table needs CUDA and in-vocabulary masked targets",
            ));
        }
        let reference_targets = targets.len();
        let index = Tensor::from_vec(targets.ids.clone(), reference_targets, model.device())?;
        let count_table = Tensor::zeros(model.config.vocab_size, DType::F32, model.device())?
            .scatter_add(
                &index,
                &Tensor::ones(reference_targets, DType::F32, model.device())?,
                0,
            )?;
        let identities = Tensor::from_vec(
            (0..model.config.vocab_size as u32).collect::<Vec<_>>(),
            model.config.vocab_size,
            model.device(),
        )?;
        let mut available = Tensor::ones(model.config.vocab_size, DType::U8, model.device())?;
        let mut top_ids = Vec::new();
        let mut top_scores = Vec::new();
        for _ in 0..9 {
            let scores = available.where_cond(
                &count_table,
                &Tensor::full(-1.0f32, model.config.vocab_size, model.device())?,
            )?;
            let best = scores.max_all()?;
            let tied = scores.broadcast_eq(&best)?;
            let id = tied
                .where_cond(
                    &identities,
                    &Tensor::full(u32::MAX, model.config.vocab_size, model.device())?,
                )?
                .min_all()?;
            available = available.mul(&identities.broadcast_ne(&id)?)?;
            top_ids.push(id.reshape(1)?);
            top_scores.push(best.reshape(1)?);
        }
        Ok(Self {
            count_table,
            top_count_ids: Tensor::cat(&top_ids, 0)?,
            top_count_scores: Tensor::cat(&top_scores, 0)?,
            reference_targets,
            payload_tokens: targets.payload_tokens(),
        })
    }

    /// Return loss sum and integer counts, not finalized host probabilities.
    fn score_rows(&self, target_ids: &Tensor, rivals: usize) -> Result<(Tensor, Tensor)> {
        let n = target_ids.elem_count();
        if !(1..=8).contains(&rivals)
            || n == 0
            || target_ids.dtype() != DType::U32
            || !target_ids.device().same_device(self.count_table.device())
        {
            return Err(invalid(
                "count metric target/rival/device inventory differs",
            ));
        }
        let device = target_ids.device();
        let target_score = self.count_table.index_select(target_ids, 0)?.unsqueeze(1)?;
        let candidates = self.top_count_ids.unsqueeze(0)?.broadcast_as((n, 9))?;
        let mut available = candidates.broadcast_ne(&target_ids.unsqueeze(1)?)?;
        let rival_scores = self.top_count_scores.unsqueeze(0)?.broadcast_as((n, 9))?;
        let mut row_loss = Tensor::zeros((n, 1), DType::F32, device)?;
        for _ in 0..rivals {
            let scores =
                available.where_cond(&rival_scores, &Tensor::full(-1.0f32, (n, 9), device)?)?;
            let best = scores.max_keepdim(1)?;
            row_loss = row_loss.add(
                &best
                    .sub(&target_score)?
                    .affine(1.0, 1.0)?
                    .maximum(&Tensor::zeros((n, 1), DType::F32, device)?)?,
            )?;
            let tied = scores.broadcast_eq(&best)?.mul(&available)?;
            let id = tied
                .where_cond(&candidates, &Tensor::full(u32::MAX, (n, 9), device)?)?
                .min_keepdim(1)?;
            available = available.mul(&candidates.broadcast_ne(&id)?)?;
        }
        row_loss = row_loss.affine(1.0 / rivals as f64, 0.0)?;
        let top1 = target_ids
            .broadcast_eq(&self.top_count_ids.narrow(0, 0, 1)?.reshape(())?)?
            .to_dtype(DType::U32)?
            .sum_all()?;
        let recall = target_ids
            .unsqueeze(1)?
            .broadcast_eq(&self.top_count_ids.narrow(0, 0, 8)?.unsqueeze(0)?)?
            .to_dtype(DType::U32)?
            .sum_all()?;
        let unviolated = row_loss.eq(0.0f32)?.to_dtype(DType::U32)?.sum_all()?;
        Ok((
            row_loss.sum_all()?,
            Tensor::cat(
                &[top1.reshape(1)?, recall.reshape(1)?, unviolated.reshape(1)?],
                0,
            )?,
        ))
    }

    fn report(
        &self,
        loss: &Tensor,
        counts: &Tensor,
        rows: usize,
        diagnostic_only: bool,
    ) -> Result<Value> {
        let rates = counts
            .to_dtype(DType::F32)?
            .affine(1.0 / rows as f64, 0.0)?
            .to_vec1::<f32>()?;
        let mean_loss = loss.affine(1.0 / rows as f64, 0.0)?.to_scalar::<f32>()?;
        Ok(
            json!({"source_split":if diagnostic_only {"dev"} else {"train"},
            "diagnostic_only":diagnostic_only, "reference_targets":self.reference_targets,
            "payload_tokens":self.payload_tokens,
            "frequency_policy":"response-masked targets with within-document predecessor",
            "score_units":"raw occurrence counts", "top1":rates[0], "recall_at_8":rates[1],
            "token_margin":mean_loss, "unviolated_target_rate":rates[2]}),
        )
    }
}

pub struct Evaluator {
    plan: PanelPlan,
    training_reference: CountPredictor,
    dev_response_oracle: CountPredictor,
}
impl Evaluator {
    pub fn new(
        model: &StackModel,
        training_reference: &MaskedTargets,
        valid: &[u32],
        mask: &[u8],
        plan: PanelPlan,
    ) -> Result<Self> {
        if !model.device().is_cuda() || model.native_vector_head_spec().is_none() {
            return Err(invalid(
                "native development metrics require a CUDA native vector head",
            ));
        }
        if training_reference.bos_id != plan.bos_id {
            return Err(invalid(
                "training reference BOS differs from development tokenizer identity",
            ));
        }
        let dev_targets = MaskedTargets::new(valid, mask, plan.bos_id)?;
        if dev_targets.len() != plan.scored_targets {
            return Err(invalid(
                "development oracle inventory differs from scored target set",
            ));
        }
        Ok(Self {
            training_reference: CountPredictor::new(model, training_reference)?,
            dev_response_oracle: CountPredictor::new(model, &dev_targets)?,
            plan,
        })
    }

    pub fn evaluate(&self, model: &StackModel, valid: &[u32], mask: &[u8]) -> Result<Value> {
        validate_mask(valid.len(), mask)?;
        let Objective::MarginHinge { rivals } = model.config.objective else {
            return Err(invalid(
                "native evaluation needs its declared margin-rival count",
            ));
        };
        let device = model.device();
        let writer = model.native_vector_writer()?;
        let mut sums = Tensor::zeros(3, DType::F32, device)?;
        let mut counts = Tensor::zeros(5, DType::U32, device)?;
        let mut reference_loss = Tensor::zeros((), DType::F32, device)?;
        let mut reference_counts = Tensor::zeros(3, DType::U32, device)?;
        let mut oracle_loss = Tensor::zeros((), DType::F32, device)?;
        let mut oracle_counts = Tensor::zeros(3, DType::U32, device)?;
        let mut targets_seen = 0usize;
        for group in self.plan.windows.chunks(16) {
            let time = group
                .iter()
                .map(|w| w.time)
                .max()
                .ok_or_else(|| invalid("empty native window group"))?;
            let mut ids = Vec::with_capacity(group.len() * time);
            let mut targets = Vec::new();
            let mut scored = Vec::new();
            for (row, window) in group.iter().enumerate() {
                ids.extend_from_slice(&valid[window.start..window.start + window.time]);
                ids.extend(std::iter::repeat_n(self.plan.bos_id, time - window.time));
                for position in 0..window.time {
                    let target_position = window.start + position + 1;
                    if target_position >= window.score_from && mask[target_position] != 0 {
                        scored.push((row * time + position) as u32);
                        targets.push(valid[target_position]);
                    }
                }
            }
            if targets.is_empty() {
                continue;
            }
            let n = targets.len();
            let hidden = model.hidden(&ids, group.len(), time)?.detach();
            let selected = hidden.index_select(&Tensor::from_vec(scored, n, device)?, 0)?;
            let candidates = writer.candidates(&selected)?;
            let metrics = writer.metrics(&candidates, &targets, rivals, 8)?;
            let as_row = |t: &Tensor| -> Result<Tensor> { Ok(t.reshape(1)?) };
            let losses = Tensor::cat(
                &[
                    as_row(&metrics.token_margin)?,
                    as_row(&metrics.address_margin)?,
                    as_row(&metrics.total_margin)?,
                ],
                0,
            )?
            .affine(n as f64, 0.0)?;
            sums = sums.add(&losses)?;
            counts = counts.add(&Tensor::cat(
                &[
                    as_row(&metrics.top1_count)?,
                    as_row(&metrics.recall_at_k_count)?,
                    as_row(&metrics.token_unviolated_count)?,
                    as_row(&metrics.total_unviolated_count)?,
                    as_row(&metrics.admitted_target_count)?,
                ],
                0,
            )?)?;
            let target_ids = Tensor::from_vec(targets, n, device)?;
            let (loss, counts) = self.training_reference.score_rows(&target_ids, rivals)?;
            reference_loss = reference_loss.add(&loss)?;
            reference_counts = reference_counts.add(&counts)?;
            let (loss, counts) = self.dev_response_oracle.score_rows(&target_ids, rivals)?;
            oracle_loss = oracle_loss.add(&loss)?;
            oracle_counts = oracle_counts.add(&counts)?;
            targets_seen += n;
        }
        if targets_seen != self.plan.scored_targets {
            return Err(invalid(
                "native evaluator lost or duplicated planned masked target positions",
            ));
        }
        if targets_seen == 0 {
            return Err(invalid(
                "native development panel has no scored shifted-response targets",
            ));
        }
        // Normalize on device; only finalized scalar reports cross to the host.
        let means = sums
            .affine(1.0 / targets_seen as f64, 0.0)?
            .to_vec1::<f32>()?;
        let rates = counts
            .to_dtype(DType::F32)?
            .affine(1.0 / targets_seen as f64, 0.0)?
            .to_vec1::<f32>()?;
        let reference = self.training_reference.report(
            &reference_loss,
            &reference_counts,
            targets_seen,
            false,
        )?;
        let oracle =
            self.dev_response_oracle
                .report(&oracle_loss, &oracle_counts, targets_seen, true)?;
        let norm = writer
            .table()
            .abs()?
            .sum(1)?
            .mean_all()?
            .to_scalar::<f32>()?;
        Ok(
            json!({ "schema":"uor-r4.native-rung0-development/2", "payload_tokens":valid.len(), "response_mask_bytes":mask.len(), "document_windows":self.plan.windows.len(), "full_global_blocks":self.plan.full_blocks, "global_tail_tokens":self.plan.tail_tokens, "expected_targets":self.plan.expected_targets, "denominator_mismatch":targets_seen!=self.plan.expected_targets, "total_mask_targets":self.plan.total_mask_targets, "bos_id":self.plan.bos_id, "targets":targets_seen, "mask_policy":"all masked targets with within-document predecessor, reset each BOS, context-bounded windows, overlapping tail context scored once", "token_margin":means[0], "address_margin":means[1], "total_margin":means[2], "unviolated_target_rate":rates[2], "total_unviolated_target_rate":rates[3], "top1":rates[0], "recall_at_8":rates[1], "admitted_target_rate":rates[4], "token_table_mean_l1_norm":norm, "table_norm_units":"mean per-token sum of absolute coordinates", "score_units":"mean absolute coordinate displacement; fixed scale1", "rivals":rivals, "kept_token_k":8, "count_predictor":reference, "dev_response_oracle":oracle }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_window_and_shifted_mask_contract_is_metadata_only() -> Result<()> {
        assert_eq!(window_starts(42856, 384, 111)?.len(), 111);
        assert_eq!(window_starts(42856, 384, 111)?[110], 42240);
        assert!(window_starts(42856, 384, 112).is_err());
        assert!(validate_mask(3, &[0, 1, 1]).is_ok());
        assert!(validate_mask(3, &[0, 1, 2]).is_err());
        assert!(validate_mask(3, &[1, 1]).is_err());
        Ok(())
    }
    #[test]
    fn uort_header_inventory_is_not_token_payload_or_tail() -> Result<()> {
        use uor_r4_core::native_geometric::mmap_corpus::{CorpusHeader, CORPUS_HEADER_SIZE};
        let bytes = CorpusHeader::new(42856, 4096).to_bytes();
        let decoded = CorpusHeader::from_bytes(&bytes).map_err(|e| invalid(e.to_string()))?;
        assert_eq!(CORPUS_HEADER_SIZE, 64);
        assert_eq!(decoded.total_tokens, 42856);
        assert_eq!(
            CORPUS_HEADER_SIZE + decoded.total_tokens as usize * 2,
            85776
        );
        assert_ne!(85776 / 2, decoded.total_tokens as usize);
        assert_eq!(decoded.total_tokens as usize / 384, 111);
        assert_eq!(decoded.total_tokens as usize % 384, 232);
        Ok(())
    }

    #[test]
    fn document_boundaries_and_overlap_tail_score_every_mask_once() -> Result<()> {
        let valid = [7, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mask = [0, 1, 1, 1, 1, 1, 1, 0, 1, 1];
        let plan = PanelPlan::new(&valid, &mask, 4, 2, 7, 8)?;
        assert_eq!(plan.scored_targets, 8);
        assert_eq!(plan.tail_tokens, 2);
        let mut seen = Vec::new();
        for window in &plan.windows {
            for p in window.start + 1..window.start + window.time + 1 {
                if p >= window.score_from && mask[p] != 0 {
                    seen.push(p);
                }
            }
        }
        assert_eq!(seen, vec![1, 2, 3, 4, 5, 6, 8, 9]);
        assert!(PanelPlan::new(&valid[1..], &mask[1..], 4, 2, 7, 8).is_err());
        Ok(())
    }

    #[test]
    fn masked_count_inventory_is_validated_metadata_only() -> Result<()> {
        let targets = MaskedTargets::new(&[0, 99, 3, 0, 4], &[1, 0, 1, 1, 1], 0)?;
        assert_eq!(targets.ids, vec![3, 4]); // BOS never owns a within-document predecessor.
        assert_eq!(targets.len(), 2);
        assert_eq!(targets.payload_tokens(), 5);
        assert!(MaskedTargets::new(&[0, 3], &[0], 0).is_err());
        assert!(MaskedTargets::new(&[0, 3], &[0, 2], 0).is_err());
        assert!(MaskedTargets::new(&[3, 0], &[1, 0], 0).is_err());
        assert!(MaskedTargets::new(&[], &[], 0).is_err());
        assert!(MaskedTargets::new(&[0, 3], &[0, 0], 0).is_err());
        assert!(MaskedTargets::new(&[0, 0], &[1, 1], 0).is_err());
        Ok(())
    }
}

#[cfg(all(test, feature = "cuda"))]
mod cuda_fixed_tests {
    use super::*;
    use std::path::PathBuf;
    use uor_r4_training::geometric_stack::{
        Gate, QuaternionWeights, RecGates, StackArch, StackConfig,
    };

    #[test]
    fn cuda_masked_train_reference_excludes_unmasked_ids_and_separates_dev_oracle() -> Result<()> {
        let device = candle_core::Device::new_cuda(0)?;
        let mut config = StackConfig::transformer(32, 4, 6, 64, 4, 11)?;
        config.arch = StackArch::Geometric;
        config.pattern = "rrrrrr".into();
        config.vocab_size = 512;
        config.gate = Gate::Hamilton;
        config.rec_gates = RecGates::Geometric;
        let mut model = StackModel::new(config, &device)?; // Parameter preparation only.
        model.set_quaternion_weights(QuaternionWeights::SpiralcoreFbs)?;
        model.set_native_vector_head(8)?;
        let train = [0, 99, 99, 99, 99, 99, 99, 3, 3, 4];
        let train_mask = [0, 0, 0, 0, 0, 0, 0, 1, 1, 1];
        let changed_unmasked = [0, 100, 100, 100, 100, 100, 100, 3, 3, 4];
        let inventory = MaskedTargets::new(&train, &train_mask, 0)?;
        let changed = MaskedTargets::new(&changed_unmasked, &train_mask, 0)?;
        let reference = CountPredictor::new(&model, &inventory)?;
        let unchanged = CountPredictor::new(&model, &changed)?;
        assert_eq!(
            reference
                .count_table
                .eq(&unchanged.count_table)?
                .min_all()?
                .to_scalar::<u8>()?,
            1
        );
        assert_eq!(
            reference
                .top_count_ids
                .eq(&unchanged.top_count_ids)?
                .min_all()?
                .to_scalar::<u8>()?,
            1
        );
        assert_eq!(
            reference
                .top_count_scores
                .eq(&unchanged.top_count_scores)?
                .min_all()?
                .to_scalar::<u8>()?,
            1
        );
        assert_eq!(reference.count_table.get(99)?.to_scalar::<f32>()?, 0.0);
        assert_eq!(reference.count_table.get(100)?.to_scalar::<f32>()?, 0.0);
        assert_eq!(reference.top_count_ids.get(0)?.to_scalar::<u32>()?, 3);
        let target_ids = Tensor::from_vec(vec![7u32, 7, 8], 3, &device)?;
        let dev = [0, 7, 7, 8];
        let dev_mask = [0, 1, 1, 1];
        let plan = PanelPlan::new(&dev, &dev_mask, 4, 1, 0, 3)?;
        let evaluator = Evaluator::new(&model, &inventory, &dev, &dev_mask, plan)?;
        assert_eq!(
            evaluator
                .training_reference
                .top_count_ids
                .get(0)?
                .to_scalar::<u32>()?,
            3
        );
        assert_eq!(
            evaluator
                .dev_response_oracle
                .top_count_ids
                .get(0)?
                .to_scalar::<u32>()?,
            7
        );
        for rivals in [1, 8] {
            let (loss, counts) = evaluator
                .training_reference
                .score_rows(&target_ids, rivals)?;
            let primary = evaluator
                .training_reference
                .report(&loss, &counts, 3, false)?;
            let (loss, counts) = evaluator
                .dev_response_oracle
                .score_rows(&target_ids, rivals)?;
            let oracle = evaluator
                .dev_response_oracle
                .report(&loss, &counts, 3, true)?;
            assert_eq!(primary["source_split"], "train");
            assert_eq!(primary["diagnostic_only"], false);
            assert_eq!(primary["reference_targets"], 3);
            assert_eq!(oracle["source_split"], "dev");
            assert_eq!(oracle["diagnostic_only"], true);
            assert_eq!(primary["top1"], 0.0);
            assert!(
                (oracle["top1"]
                    .as_f64()
                    .ok_or_else(|| invalid("oracle top1 absent"))?
                    - 2.0 / 3.0)
                    .abs()
                    < 1e-6
            );
            assert_eq!(oracle["recall_at_8"], 1.0);
            let stored_primary_loss = if rivals == 1 { 3.0 } else { 11.0 / 8.0 };
            let stored_oracle_loss = if rivals == 1 { 2.0 / 3.0 } else { 1.0 / 12.0 };
            assert!(
                (primary["token_margin"]
                    .as_f64()
                    .ok_or_else(|| invalid("primary margin absent"))?
                    - stored_primary_loss)
                    .abs()
                    < 1e-6
            );
            assert!(
                (oracle["token_margin"]
                    .as_f64()
                    .ok_or_else(|| invalid("oracle margin absent"))?
                    - stored_oracle_loss)
                    .abs()
                    < 1e-6
            );
            assert_eq!(primary["unviolated_target_rate"], 0.0);
            assert!(
                (oracle["unviolated_target_rate"]
                    .as_f64()
                    .ok_or_else(|| invalid("oracle unviolated absent"))?
                    - 2.0 / 3.0)
                    .abs()
                    < 1e-6
            );
            assert!(
                (primary["recall_at_8"]
                    .as_f64()
                    .ok_or_else(|| invalid("primary recall absent"))?
                    - 2.0 / 3.0)
                    .abs()
                    < 1e-6
            );
        }
        Ok(())
    }

    #[test]
    fn cuda_document_evaluator_known_answers_and_native_artifact_roundtrip() -> Result<()> {
        let parent = std::env::var_os("UOR_FINITE_REPORT_ROOT")
            .map(PathBuf::from)
            .ok_or_else(||invalid("CUDA finite artifact test needs UOR_FINITE_REPORT_ROOT under /workspace; no system temporary directory"))?;
        if !parent.is_absolute() || !parent.starts_with("/workspace") {
            return Err(invalid(
                "finite CUDA report root must be absolute and under /workspace",
            ));
        }
        let report = parent.join("native-document-evaluator");
        uor_r4_core::report_output::claim(&report)?;
        let result = (|| -> Result<()> {
            let device = candle_core::Device::new_cuda(0)?; // Missing CUDA is failure.
            let valid = [0, 1, 2, 1, 0, 2, 1, 2, 1];
            let mask = [0, 1, 1, 1, 0, 1, 1, 1, 1];
            let train_reference =
                MaskedTargets::new(&[0, 1, 1, 1, 1, 2, 2, 2], &[0, 1, 1, 1, 1, 1, 1, 1], 0)?;
            let plan = PanelPlan::new(&valid, &mask, 4, 2, 0, 7)?;
            let close = |record: &Value, key: &str, want: f64| -> Result<()> {
                let got = record[key]
                    .as_f64()
                    .ok_or_else(|| invalid(format!("missing metric {key}")))?;
                assert!(
                    (got - want).abs() < 1e-6,
                    "{key}: {got} vs stored analytic {want}"
                );
                Ok(())
            };
            for rivals in [1, 8] {
                let mut config = StackConfig::transformer(32, 4, 6, 64, 4, 11)?;
                config.arch = StackArch::Geometric;
                config.pattern = "rrrrrr".into();
                config.vocab_size = 512; // 16 candidates leaves enough non-target rivals for M8.
                config.mlp_hidden = 64;
                config.gate = Gate::Hamilton;
                config.rec_gates = RecGates::Geometric;
                config.objective = Objective::MarginHinge { rivals };
                let mut model = StackModel::new(config, &device)?;
                model.set_quaternion_weights(QuaternionWeights::SpiralcoreFbs)?;
                model.set_native_vector_head(8)?;
                for var in model.variables().values() {
                    var.set(&Tensor::zeros(var.shape(), DType::F32, &device)?)?;
                }
                // Stored analytic fixture: zero carriers/table, tied atlas cells.
                // No CPU forward/reference and no optimizer/update is executed.
                let metrics =
                    Evaluator::new(&model, &train_reference, &valid, &mask, plan.clone())?
                        .evaluate(&model, &valid, &mask)?;
                assert_eq!(metrics["targets"], 7);
                assert_eq!(metrics["denominator_mismatch"], false);
                close(&metrics, "token_margin", 1.)?;
                close(&metrics, "address_margin", 1.)?;
                close(&metrics, "total_margin", 2.)?;
                close(&metrics, "top1", 0.)?;
                close(&metrics, "recall_at_8", 1.)?;
                close(&metrics, "unviolated_target_rate", 0.)?;
                close(&metrics["count_predictor"], "top1", 4. / 7.)?;
                close(&metrics["count_predictor"], "recall_at_8", 1.)?;
                assert_eq!(metrics["count_predictor"]["source_split"], "train");
                assert_eq!(metrics["count_predictor"]["reference_targets"], 7);
                assert_eq!(metrics["dev_response_oracle"]["source_split"], "dev");
                assert_eq!(metrics["dev_response_oracle"]["diagnostic_only"], true);
                close(
                    &metrics["count_predictor"],
                    "token_margin",
                    if rivals == 1 { 6. / 7. } else { 3. / 28. },
                )?;
                let directory = report.join(format!("arm-{rivals}"));
                model.save(&directory)?;
                let restored = StackModel::load(&directory, &device)?;
                let restored_metrics =
                    Evaluator::new(&restored, &train_reference, &valid, &mask, plan.clone())?
                        .evaluate(&restored, &valid, &mask)?;
                assert_eq!(restored_metrics, metrics);
                let query = Tensor::zeros((1, 32), DType::F32, &device)?;
                assert_eq!(
                    model
                        .native_vector_channel()?
                        .emit(&query)?
                        .to_vec1::<u32>()?,
                    restored
                        .native_vector_channel()?
                        .emit(&query)?
                        .to_vec1::<u32>()?
                );
                let config_path = directory.join("config.json");
                let original = std::fs::read(&config_path)?;
                let mut missing: Value = serde_json::from_slice(&original)?;
                missing
                    .as_object_mut()
                    .ok_or_else(|| invalid("artifact config object"))?
                    .remove("native_vector_head");
                std::fs::write(&config_path, serde_json::to_vec_pretty(&missing)?)?;
                assert!(StackModel::load(&directory, &device).is_err());
                let mut stale: Value = serde_json::from_slice(&original)?;
                stale["native_vector_head"]["source_sha256"] = json!("stale");
                std::fs::write(&config_path, serde_json::to_vec_pretty(&stale)?)?;
                assert!(StackModel::load(&directory, &device).is_err());
                std::fs::write(&config_path, original)?;
                std::fs::write(
                    report.join(format!("arm-{rivals}-metrics.json")),
                    serde_json::to_vec_pretty(&metrics)?,
                )?;
            }
            Ok(())
        })();
        super::super::finish(&report, result)
    }
}
