//! Population evaluation and bounded generation for the retained dense reference.
//!
//! Evaluation follows `TokenStore::sequential_batches` in the retained
//! `tools/r4-softmax-trainer/src/r4_softmax_trainer/train.py`: context and
//! stride are 256, targets are shifted by one, and incomplete tails are not
//! scored. The caller owns artifact identities, output claims and deadlines.
//! Callback errors stop execution, allowing the caller to preserve partial
//! prediction records without reporting an incomplete population as complete.

use std::collections::BTreeSet;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

use crate::{invalid, ReferenceModel, Result};

pub const EVALUATION_CONTEXT: usize = 256;
pub const MAX_EVALUATION_BATCH: usize = 16;
pub const MAX_GENERATION_PROMPTS: usize = 5;
pub const MAX_NEW_TOKENS: usize = 128;
pub const BOS_TOKEN_ID: u32 = 0;
pub const EOS_TOKEN_ID: u32 = 1;
pub const SEEDED_SAMPLER_POLICY: &str =
    "r4-local-top-k-q32-splitmix64/1;temperature=0.8;top-k=40;rank=logit-desc-token-asc";
pub const GREEDY_SAMPLER_POLICY: &str = "r4-local-greedy-argmax-token-asc/1";

/// Scalar scoring of an actual complete vocabulary row. Reductions use stable
/// f64 logsumexp over the returned F32 logits, never a sampled vocabulary.
#[derive(Clone, Debug, Serialize)]
pub struct LogitScore {
    pub target_token: u32,
    pub predicted_token: u32,
    pub target_logit: f32,
    pub maximum_logit: f32,
    pub log_normalizer: f64,
    pub nll_nats: f64,
    pub correct: bool,
}

/// Score a vocabulary row; exact logit ties select the lower token ID.
pub fn score_logits(logits: &[f32], target: u32) -> Result<LogitScore> {
    let predicted = greedy_token(logits)?;
    let target_logit = *logits
        .get(target as usize)
        .ok_or_else(|| invalid("evaluation target lies outside the logit vocabulary"))?;
    let maximum_logit = logits[predicted as usize];
    let maximum = f64::from(maximum_logit);
    let mut denominator = CompensatedSum::default();
    for &logit in logits {
        denominator.add((f64::from(logit) - maximum).exp());
    }
    let shifted_log_normalizer = denominator.total().ln();
    let log_normalizer = maximum + shifted_log_normalizer;
    // Keep the subtraction close to the logits' scale to avoid cancellation.
    let nll_nats = maximum - f64::from(target_logit) + shifted_log_normalizer;
    if !nll_nats.is_finite() || nll_nats < 0.0 || !log_normalizer.is_finite() {
        return Err(invalid(
            "nonfinite or negative vocabulary negative log likelihood",
        ));
    }
    Ok(LogitScore {
        target_token: target,
        predicted_token: predicted,
        target_logit,
        maximum_logit,
        log_normalizer,
        nll_nats,
        correct: predicted == target,
    })
}

#[derive(Clone, Debug, Serialize)]
pub struct TokenPrediction {
    pub block_index: usize,
    pub position_in_block: usize,
    pub input_offset: usize,
    pub target_offset: usize,
    pub input_token: u32,
    #[serde(flatten)]
    pub score: LogitScore,
}

#[derive(Clone, Debug, Serialize)]
pub struct BlockEvaluation {
    pub block_index: usize,
    pub input_start: usize,
    pub first_target_offset: usize,
    pub target_end_exclusive: usize,
    pub scored_targets: usize,
    pub nll_sum_nats: f64,
    pub mean_nll_nats: f64,
    pub correct: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct PopulationEvaluation {
    pub schema: &'static str,
    pub input_token_count: usize,
    pub context_length: usize,
    pub stride: usize,
    pub requested_batch_size: usize,
    pub completed_batches: usize,
    pub completed_blocks: usize,
    pub available_next_targets: usize,
    pub scored_targets: usize,
    pub unscored_tail_targets: usize,
    pub nll_sum_nats: f64,
    pub mean_nll_nats: f64,
    pub perplexity: Option<f64>,
    pub correct: usize,
    pub top1_accuracy: f64,
    pub forward_and_transfer_seconds: f64,
    pub elapsed_seconds: f64,
    pub gradients_tracked: bool,
    pub optimizer_steps: u64,
    pub complete: bool,
}

/// Evaluate every full 256-input block, including a final partial batch.
///
/// Each callback contains exactly one complete block in corpus order and its
/// 256 predictions. A callback can stream CSV/JSONL, enforce a shared deadline,
/// or abort on an I/O error. No population summary is returned on partial work.
pub fn evaluate_tokens<F>(
    model: &ReferenceModel,
    tokens: &[u16],
    batch_size: usize,
    mut on_block: F,
) -> Result<PopulationEvaluation>
where
    F: FnMut(&BlockEvaluation, &[TokenPrediction]) -> Result<()>,
{
    if !(1..=MAX_EVALUATION_BATCH).contains(&batch_size) {
        return Err(invalid("evaluation batch size must be 1..=16"));
    }
    if model.config.max_position_embeddings != EVALUATION_CONTEXT {
        return Err(invalid(
            "reference evaluation requires the declared 256-token context",
        ));
    }
    let blocks = complete_blocks(tokens.len())?;
    if tokens
        .iter()
        .any(|&token| usize::from(token) >= model.config.vocab_size)
    {
        return Err(invalid(
            "evaluation token store contains an out-of-vocabulary token",
        ));
    }
    let started = Instant::now();
    let mut forward_seconds = 0.0;
    let mut total_nll = CompensatedSum::default();
    let mut total_correct = 0usize;
    let mut completed_batches = 0usize;
    for first_block in (0..blocks).step_by(batch_size) {
        let lanes = batch_size.min(blocks - first_block);
        let mut inputs = Vec::with_capacity(lanes * EVALUATION_CONTEXT);
        for block in first_block..first_block + lanes {
            let start = block * EVALUATION_CONTEXT;
            inputs.extend(
                tokens[start..start + EVALUATION_CONTEXT]
                    .iter()
                    .map(|&id| u32::from(id)),
            );
        }
        let forward_started = Instant::now();
        let logits = model.forward_eval_batch(&inputs, lanes, EVALUATION_CONTEXT)?;
        if logits.track_op() {
            return Err(invalid(
                "reference evaluation unexpectedly retained a gradient graph",
            ));
        }
        if logits.dims() != [lanes, EVALUATION_CONTEXT, model.config.vocab_size] {
            return Err(invalid(
                "reference batched logit shape differs from [batch,256,vocab]",
            ));
        }
        let values = logits.flatten_all()?.to_vec1::<f32>()?;
        drop(logits);
        forward_seconds += forward_started.elapsed().as_secs_f64();
        for lane in 0..lanes {
            let block_index = first_block + lane;
            let start = block_index * EVALUATION_CONTEXT;
            let mut nll = CompensatedSum::default();
            let mut correct = 0usize;
            let mut predictions = Vec::with_capacity(EVALUATION_CONTEXT);
            for position in 0..EVALUATION_CONTEXT {
                let row_start = (lane * EVALUATION_CONTEXT + position) * model.config.vocab_size;
                let row = &values[row_start..row_start + model.config.vocab_size];
                let score = score_logits(row, u32::from(tokens[start + position + 1]))?;
                nll.add(score.nll_nats);
                total_nll.add(score.nll_nats);
                correct += usize::from(score.correct);
                predictions.push(TokenPrediction {
                    block_index,
                    position_in_block: position,
                    input_offset: start + position,
                    target_offset: start + position + 1,
                    input_token: u32::from(tokens[start + position]),
                    score,
                });
            }
            total_correct += correct;
            let block = BlockEvaluation {
                block_index,
                input_start: start,
                first_target_offset: start + 1,
                target_end_exclusive: start + EVALUATION_CONTEXT + 1,
                scored_targets: EVALUATION_CONTEXT,
                nll_sum_nats: nll.total(),
                mean_nll_nats: nll.total() / EVALUATION_CONTEXT as f64,
                correct,
            };
            on_block(&block, &predictions)?;
        }
        completed_batches += 1;
    }
    let scored_targets = blocks * EVALUATION_CONTEXT;
    let mean_nll_nats = total_nll.total() / scored_targets as f64;
    let perplexity = mean_nll_nats.exp();
    Ok(PopulationEvaluation {
        schema: "uor-r4.reference-population-evaluation/1",
        input_token_count: tokens.len(),
        context_length: EVALUATION_CONTEXT,
        stride: EVALUATION_CONTEXT,
        requested_batch_size: batch_size,
        completed_batches,
        completed_blocks: blocks,
        available_next_targets: tokens.len() - 1,
        scored_targets,
        unscored_tail_targets: tokens.len() - 1 - scored_targets,
        nll_sum_nats: total_nll.total(),
        mean_nll_nats,
        perplexity: perplexity.is_finite().then_some(perplexity),
        correct: total_correct,
        top1_accuracy: total_correct as f64 / scored_targets as f64,
        forward_and_transfer_seconds: forward_seconds,
        elapsed_seconds: started.elapsed().as_secs_f64(),
        gradients_tracked: false,
        optimizer_steps: 0,
        complete: true,
    })
}

fn complete_blocks(token_count: usize) -> Result<usize> {
    let blocks = token_count.saturating_sub(1) / EVALUATION_CONTEXT;
    if blocks == 0 {
        return Err(invalid(
            "population evaluation needs at least 257 retained tokens",
        ));
    }
    Ok(blocks)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerationPrompt {
    pub id: String,
    pub text: String,
    /// None selects greedy; Some selects the unchanged historical Q32 sampler.
    pub seed: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct GenerationPrediction {
    pub prompt_id: String,
    pub decision: usize,
    pub input_positions: usize,
    pub selected_token: u32,
    pub greedy_token: u32,
    pub selected_logit: f32,
    /// Probability under the model's complete untempered vocabulary, not under
    /// the top-k sampling distribution.
    pub selected_model_nll_nats: f64,
    pub logits_sha256_le_f32: String,
    pub sampler_state_before: Option<u64>,
    pub sampler_state_after: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum GenerationStop {
    Eos,
    ShortCycle { period: usize },
    MaximumNewTokens,
}

#[derive(Clone, Debug, Serialize)]
pub struct GenerationResult {
    pub schema: &'static str,
    pub prompt: GenerationPrompt,
    pub tokenizer_cid: String,
    pub sampler_policy: &'static str,
    pub bos_policy: &'static str,
    pub prompt_token_ids: Vec<u32>,
    pub generated_token_ids: Vec<u32>,
    pub raw_decoded_bytes: Vec<u8>,
    pub raw_decoded: String,
    pub response_text: String,
    pub utf8_decodable: bool,
    pub short_cycle_period: Option<usize>,
    pub stop: GenerationStop,
    pub max_new_tokens: usize,
    pub context_capacity: usize,
    pub full_prefix_forward_calls: usize,
    pub total_input_positions_recomputed: usize,
    pub incremental_kv_cache: bool,
    pub gradients_tracked: bool,
    pub elapsed_seconds: f64,
}

/// Generate actual continuations from the same retained tokenizer and model.
/// Full-prefix reevaluation is explicit; no incremental-cache performance is
/// claimed. Prompt plus BOS plus the requested horizon must fit the context.
pub fn generate_prompts<F>(
    model: &ReferenceModel,
    tokenizer: &HfBpeTokenizer,
    prompts: &[GenerationPrompt],
    max_new_tokens: usize,
    mut on_prediction: F,
) -> Result<Vec<GenerationResult>>
where
    F: FnMut(&GenerationPrediction) -> Result<()>,
{
    if prompts.is_empty() || prompts.len() > MAX_GENERATION_PROMPTS {
        return Err(invalid("generation requires 1..=5 prompts per request"));
    }
    if !(1..=MAX_NEW_TOKENS).contains(&max_new_tokens) {
        return Err(invalid("generation horizon must be 1..=128 new tokens"));
    }
    if tokenizer.vocab_size() != model.config.vocab_size
        || model.config.max_position_embeddings != EVALUATION_CONTEXT
    {
        return Err(invalid(
            "generation tokenizer vocabulary or reference context differs",
        ));
    }
    let mut ids = BTreeSet::new();
    let mut prepared = Vec::with_capacity(prompts.len());
    for prompt in prompts {
        if prompt.id.is_empty() || !ids.insert(prompt.id.as_str()) {
            return Err(invalid("generation prompt IDs must be nonempty and unique"));
        }
        let content = tokenizer.encode(&prompt.text);
        if content.is_empty()
            || content
                .iter()
                .any(|&id| id as usize >= model.config.vocab_size)
        {
            return Err(invalid(format!("invalid tokenized prompt {}", prompt.id)));
        }
        if content
            .len()
            .checked_add(1)
            .and_then(|n| n.checked_add(max_new_tokens))
            .is_none_or(|n| n > EVALUATION_CONTEXT)
        {
            return Err(invalid(format!(
                "prompt {} plus BOS and horizon exceeds 256",
                prompt.id
            )));
        }
        let mut prefix = Vec::with_capacity(content.len() + 1 + max_new_tokens);
        prefix.push(BOS_TOKEN_ID);
        prefix.extend(content);
        prepared.push(prefix);
    }
    let mut results = Vec::with_capacity(prompts.len());
    for (prompt, mut prefix) in prompts.iter().zip(prepared) {
        let started = Instant::now();
        let prompt_token_ids = prefix.clone();
        let mut generated = Vec::with_capacity(max_new_tokens);
        let mut sampler = prompt.seed.map(SplitMix64::new);
        let mut stop = GenerationStop::MaximumNewTokens;
        let mut input_positions_recomputed = 0usize;
        for decision in 0..max_new_tokens {
            let logits = model.forward_eval_batch(&prefix, 1, prefix.len())?;
            if logits.track_op() {
                return Err(invalid(
                    "reference generation unexpectedly retained a gradient graph",
                ));
            }
            if logits.dims() != [1, prefix.len(), model.config.vocab_size] {
                return Err(invalid(
                    "generation logit shape differs from [1,time,vocab]",
                ));
            }
            let row = logits
                .narrow(1, prefix.len() - 1, 1)?
                .flatten_all()?
                .to_vec1::<f32>()?;
            drop(logits);
            let sampler_state_before = sampler.as_ref().map(|rng| rng.state);
            let selected = match sampler.as_mut() {
                Some(rng) => sample_top_k_q32(&row, rng)?,
                None => greedy_token(&row)?,
            };
            let score = score_logits(&row, selected)?;
            let mut digest = Sha256::new();
            for &value in &row {
                digest.update(value.to_le_bytes());
            }
            let prediction = GenerationPrediction {
                prompt_id: prompt.id.clone(),
                decision,
                input_positions: prefix.len(),
                selected_token: selected,
                greedy_token: score.predicted_token,
                selected_logit: score.target_logit,
                selected_model_nll_nats: score.nll_nats,
                logits_sha256_le_f32: hex::encode(digest.finalize()),
                sampler_state_before,
                sampler_state_after: sampler.as_ref().map(|rng| rng.state),
            };
            on_prediction(&prediction)?;
            input_positions_recomputed += prefix.len();
            generated.push(selected);
            if selected == EOS_TOKEN_ID {
                stop = GenerationStop::Eos;
                break;
            }
            if let Some(period) = short_cycle_period(&generated) {
                stop = GenerationStop::ShortCycle { period };
                break;
            }
            prefix.push(selected);
        }
        let end = generated
            .iter()
            .position(|&id| id == EOS_TOKEN_ID)
            .unwrap_or(generated.len());
        let raw_bytes = tokenizer.decode_bytes(&generated);
        let response_bytes = tokenizer.decode_bytes(&generated[..end]);
        results.push(GenerationResult {
            schema: "uor-r4.reference-generation/1",
            prompt: prompt.clone(),
            tokenizer_cid: tokenizer.address(),
            sampler_policy: if prompt.seed.is_some() { SEEDED_SAMPLER_POLICY } else { GREEDY_SAMPLER_POLICY },
            bos_policy: "prepend checkpoint BOS 0 exactly once before raw prompt encoding; EOS 1 ends output",
            prompt_token_ids,
            full_prefix_forward_calls: generated.len(),
            total_input_positions_recomputed: input_positions_recomputed,
            incremental_kv_cache: false,
            gradients_tracked: false,
            raw_decoded: String::from_utf8_lossy(&raw_bytes).into_owned(),
            response_text: String::from_utf8_lossy(&response_bytes).trim().to_owned(),
            utf8_decodable: std::str::from_utf8(&raw_bytes).is_ok() && std::str::from_utf8(&response_bytes).is_ok(),
            raw_decoded_bytes: raw_bytes,
            short_cycle_period: short_cycle_period(&generated),
            generated_token_ids: generated,
            stop,
            max_new_tokens,
            context_capacity: EVALUATION_CONTEXT,
            elapsed_seconds: started.elapsed().as_secs_f64(),
        });
    }
    Ok(results)
}

/// Arithmetic copied without a protocol change from
/// `src/r4_softmax_local_generation.rs::{SplitMix64,sample_top_k_q32}`.
struct SplitMix64 {
    state: u64,
}
impl SplitMix64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        value ^ (value >> 31)
    }
}

fn sample_top_k_q32(logits: &[f32], sampler: &mut SplitMix64) -> Result<u32> {
    if logits.is_empty() || logits.iter().any(|value| !value.is_finite()) {
        return Err(invalid("seeded sampler requires nonempty finite logits"));
    }
    let mut ranked = logits.iter().copied().enumerate().collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    ranked.truncate(40.min(ranked.len()));
    let maximum = f64::from(ranked[0].1);
    let mut weighted = Vec::with_capacity(ranked.len());
    let mut total = 0u64;
    for (token, logit) in ranked {
        let probability_ratio = ((f64::from(logit) - maximum) / 0.8).exp();
        let weight = (probability_ratio * 4_294_967_296.0)
            .round()
            .clamp(1.0, u64::MAX as f64) as u64;
        total = total
            .checked_add(weight)
            .ok_or_else(|| invalid("seeded sampler Q32 total overflow"))?;
        weighted.push((token, weight));
    }
    let threshold = ((u128::from(sampler.next_u64()) * u128::from(total)) >> 64) as u64;
    let mut cumulative = 0u64;
    for (token, weight) in &weighted {
        cumulative = cumulative
            .checked_add(*weight)
            .ok_or_else(|| invalid("seeded sampler Q32 cumulative overflow"))?;
        if threshold < cumulative {
            return u32::try_from(*token).map_err(|_| invalid("sampled token exceeds u32"));
        }
    }
    let token = weighted
        .last()
        .map(|(token, _)| *token)
        .ok_or_else(|| invalid("seeded sampler top-k empty"))?;
    u32::try_from(token).map_err(|_| invalid("sampled token exceeds u32"))
}

fn greedy_token(logits: &[f32]) -> Result<u32> {
    if logits.is_empty() || logits.iter().any(|value| !value.is_finite()) {
        return Err(invalid("greedy selection requires nonempty finite logits"));
    }
    let mut best = 0usize;
    for (token, &value) in logits.iter().enumerate().skip(1) {
        if value > logits[best] {
            best = token;
        }
    }
    u32::try_from(best).map_err(|_| invalid("greedy token exceeds u32"))
}

/// The historical stop rule: a terminal cycle of length 1..4 repeated 3 times.
pub fn short_cycle_period(tokens: &[u32]) -> Option<usize> {
    for period in 1..=4 {
        let span = period * 3;
        if tokens.len() < span {
            continue;
        }
        let tail = &tokens[tokens.len() - span..];
        if tail[..period] == tail[period..period * 2] && tail[..period] == tail[period * 2..] {
            return Some(period);
        }
    }
    None
}

#[derive(Default)]
struct CompensatedSum {
    sum: f64,
    correction: f64,
}
impl CompensatedSum {
    fn add(&mut self, value: f64) {
        let adjusted = value - self.correction;
        let next = self.sum + adjusted;
        self.correction = (next - self.sum) - adjusted;
        self.sum = next;
    }
    fn total(&self) -> f64 {
        self.sum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_population_preserves_shift_and_omits_only_incomplete_tail() -> Result<()> {
        assert!(complete_blocks(256).is_err());
        assert_eq!(complete_blocks(257)?, 1);
        assert_eq!(complete_blocks(512)?, 1);
        assert_eq!(complete_blocks(513)?, 2);
        assert_eq!(complete_blocks(250_000)?, 976);
        assert_eq!(complete_blocks(249_880)?, 976);
        Ok(())
    }

    #[test]
    fn full_vocabulary_nll_is_stable_under_large_offsets_and_ties() -> Result<()> {
        let uniform = score_logits(&[1000.0, 1000.0, 1000.0], 1)?;
        assert!((uniform.nll_nats - 3f64.ln()).abs() < 1e-12);
        assert_eq!(uniform.predicted_token, 0);
        let shifted = score_logits(&[-1001.0, -1000.0, -999.0], 2)?;
        let original = score_logits(&[-1.0, 0.0, 1.0], 2)?;
        assert!((shifted.nll_nats - original.nll_nats).abs() < 1e-12);
        assert!(score_logits(&[0.0, f32::NAN], 0).is_err());
        assert!(score_logits(&[0.0], 1).is_err());
        Ok(())
    }

    #[test]
    fn historical_sampler_rng_and_cycle_boundaries_are_fixed() -> Result<()> {
        let mut rng = SplitMix64::new(0);
        assert_eq!(rng.next_u64(), 0xe220_a839_7b1d_cdaf);
        assert_eq!(rng.next_u64(), 0x6e78_9e6a_a1b9_65f4);
        assert_eq!(sample_top_k_q32(&[9.0], &mut rng)?, 0);
        assert!(sample_top_k_q32(&[f32::INFINITY], &mut rng).is_err());
        assert_eq!(short_cycle_period(&[1, 2, 1, 2]), None);
        assert_eq!(short_cycle_period(&[1, 2, 1, 2, 1, 2]), Some(2));
        Ok(())
    }
}
