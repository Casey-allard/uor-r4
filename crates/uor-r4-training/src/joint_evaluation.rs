//! Population scoring and actual incremental generation for the D8 learner.
//!
//! The caller owns input/model identities, deadlines, report claiming and
//! sealing. Evaluation resets the entire model at each 256-input block.
//! Generation always constructs a new session, including for NoRead and edited
//! prompts; disabling only the last read would leave earlier read information
//! in recurrent state. Story probes are frozen exploratory task transfer, not
//! natural-language qualification or supervision for the read selector.

use std::time::Instant;

use serde::Serialize;
use sha2::{Digest, Sha256};
use uor_r4_core::answer_oracle::{self, Intent};
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

use crate::joint_model::{JointModel, JointStep, ReadMode};
use crate::reference_eval::short_cycle_period;
use crate::{invalid, Result};

pub const EVALUATION_CONTEXT: usize = 256;
pub const CALIBRATION_BLOCKS: usize = 64;
pub const MAX_EVALUATION_BATCH: usize = 32;
pub const MAX_NEW_TOKENS: usize = 128;
pub const STORY_PROBE_MAX_NEW_TOKENS: usize = 32;
pub const PROBABILITY_SUM_TOLERANCE: f64 = 0.0002;
pub const GREEDY_POLICY: &str = "joint-probability-greedy-token-asc/1";
pub const SEEDED_POLICY: &str = "joint-probability-top-k-q32-splitmix64/1;score=ln(f64(probability));temperature=0.8;top-k=40;rank=probability-desc-token-asc";
pub const STORY_PROBE_SCOPE: &str = "EXPLORATORY_TASK_TRANSFER: fixed natural-story noun completions; TinyStories-only training does not establish this task format. Failure is not by itself a read implementation defect or a general-language verdict.";
pub const STORY_STOP_POLICY: &str = "Stop at the first generated ASCII period, retaining the complete final token, or EOS, a terminal short cycle, or 32 new tokens. Complete correctness requires an exact accepted noun-plus-period response and a period/EOS stop; no response truncation or answer repair.";

#[derive(Clone, Debug, Serialize)]
pub struct ProbabilityScore {
    pub target_token: u32,
    pub predicted_token: u32,
    pub target_probability: f64,
    pub probability_sum: f64,
    pub nll_nats: f64,
    pub correct: bool,
}

fn row_summary(probabilities: &[f32]) -> Result<(u32, f64)> {
    if probabilities.is_empty() {
        return Err(invalid("joint probability vocabulary is empty"));
    }
    let mut sum = CompensatedSum::default();
    let mut best = 0usize;
    for (token, &probability) in probabilities.iter().enumerate() {
        if !probability.is_finite() || probability < 0.0 {
            return Err(invalid("joint probability row contains an invalid value"));
        }
        sum.add(f64::from(probability));
        if probability > probabilities[best] {
            best = token;
        }
    }
    if (sum.total() - 1.0).abs() > PROBABILITY_SUM_TOLERANCE {
        return Err(invalid(format!(
            "joint probability row is not normalized: {}",
            sum.total()
        )));
    }
    Ok((
        u32::try_from(best).map_err(|_| invalid("joint vocabulary exceeds u32"))?,
        sum.total(),
    ))
}

/// Score the model's declared distribution without renormalizing it or adding
/// a target-dependent floor. Zero target probability is a numerical failure.
pub fn score_probabilities(probabilities: &[f32], target: u32) -> Result<ProbabilityScore> {
    let (predicted_token, probability_sum) = row_summary(probabilities)?;
    let probability = f64::from(
        *probabilities
            .get(target as usize)
            .ok_or_else(|| invalid("joint evaluation target lies outside vocabulary"))?,
    );
    if probability <= 0.0 || probability > 1.0 + PROBABILITY_SUM_TOLERANCE {
        return Err(invalid(
            "joint target probability is outside its valid range",
        ));
    }
    Ok(ProbabilityScore {
        target_token: target,
        predicted_token,
        target_probability: probability,
        probability_sum,
        nll_nats: -probability.ln(),
        correct: target == predicted_token,
    })
}

#[derive(Clone, Debug, Serialize)]
pub struct JointTokenPrediction {
    pub block_index: usize,
    pub position_in_block: usize,
    pub input_offset: usize,
    pub target_offset: usize,
    pub input_token: u32,
    #[serde(flatten)]
    pub score: ProbabilityScore,
    pub no_read_mass: f64,
    pub copy_gate: f64,
    pub effective_copy_mass: f64,
    pub read_mass_sum: f64,
    pub top_read_position: Option<usize>,
    pub top_read_token: Option<u32>,
    pub top_read_mass: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct JointBlockEvaluation {
    pub block_index: usize,
    pub input_start: usize,
    pub first_target_offset: usize,
    pub target_end_exclusive: usize,
    pub partition: &'static str,
    pub scored_targets: usize,
    pub nll_sum_nats: f64,
    pub mean_nll_nats: f64,
    pub correct: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct JointPartitionMetrics {
    pub blocks: usize,
    pub scored_targets: usize,
    pub nll_sum_nats: f64,
    pub mean_nll_nats: Option<f64>,
    pub perplexity: Option<f64>,
    pub correct: usize,
    pub top1_accuracy: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct JointEvaluation {
    pub schema: &'static str,
    pub mode: ReadMode,
    pub input_token_count: usize,
    pub context: usize,
    pub stride: usize,
    pub requested_batch_size: usize,
    pub completed_batches: usize,
    pub completed_blocks: usize,
    pub unscored_tail_targets: usize,
    pub full: JointPartitionMetrics,
    pub tune: JointPartitionMetrics,
    pub comparison: JointPartitionMetrics,
    pub maximum_probability_sum_error: f64,
    pub forward_and_transfer_seconds: f64,
    pub elapsed_seconds: f64,
    pub gradients_tracked: bool,
    pub optimizer_steps: u64,
    pub complete: bool,
}

/// Score every complete block supplied by the caller. With the pinned 250,000
/// IDs this is 64 calibration and 912 comparison blocks; a smaller development
/// prefix is reported honestly, with null metrics for an empty partition.
/// No completed summary is returned if the callback or model fails.
pub fn evaluate<F>(
    model: &JointModel,
    tokens: &[u16],
    mode: ReadMode,
    batch_size: usize,
    mut on_block: F,
) -> Result<JointEvaluation>
where
    F: FnMut(&JointBlockEvaluation, &[JointTokenPrediction]) -> Result<()>,
{
    let started = Instant::now();
    if !(1..=MAX_EVALUATION_BATCH).contains(&batch_size)
        || model.config.context != EVALUATION_CONTEXT
    {
        return Err(invalid(
            "joint evaluation requires context256 and batch1..=32",
        ));
    }
    let blocks = tokens.len().saturating_sub(1) / EVALUATION_CONTEXT;
    if blocks == 0
        || tokens
            .iter()
            .any(|&id| usize::from(id) >= model.config.vocab_size)
    {
        return Err(invalid(
            "joint evaluation needs valid tokens and at least one full block",
        ));
    }
    let mut full = PartitionAccumulator::default();
    let mut tune = PartitionAccumulator::default();
    let mut comparison = PartitionAccumulator::default();
    let mut maximum_probability_sum_error = 0.0f64;
    let mut forward_and_transfer_seconds = 0.0;
    let mut completed_batches = 0usize;
    for first_block in (0..blocks).step_by(batch_size) {
        let batch = batch_size.min(blocks - first_block);
        let mut inputs = Vec::with_capacity(batch * EVALUATION_CONTEXT);
        for block in first_block..first_block + batch {
            let start = block * EVALUATION_CONTEXT;
            inputs.extend(
                tokens[start..start + EVALUATION_CONTEXT]
                    .iter()
                    .map(|&x| u32::from(x)),
            );
        }
        let forward_started = Instant::now();
        let output = model.forward(&inputs, batch, EVALUATION_CONTEXT, mode, false)?;
        if output.probabilities.track_op()
            || output.probabilities.dims() != [batch, EVALUATION_CONTEXT, model.config.vocab_size]
            || output.no_read_mass.dims() != [batch, EVALUATION_CONTEXT]
            || output.copy_gate.dims() != [batch, EVALUATION_CONTEXT]
            || output.read_masses.dims() != [batch, EVALUATION_CONTEXT, EVALUATION_CONTEXT]
        {
            return Err(invalid(
                "joint evaluation output shape or gradient contract differs",
            ));
        }
        let probabilities = output.probabilities.flatten_all()?.to_vec1::<f32>()?;
        let no_read = output.no_read_mass.flatten_all()?.to_vec1::<f32>()?;
        let gates = output.copy_gate.flatten_all()?.to_vec1::<f32>()?;
        let reads = output.read_masses.flatten_all()?.to_vec1::<f32>()?;
        drop(output);
        forward_and_transfer_seconds += forward_started.elapsed().as_secs_f64();
        for lane in 0..batch {
            let block_index = first_block + lane;
            let start = block_index * EVALUATION_CONTEXT;
            let mut predictions = Vec::with_capacity(EVALUATION_CONTEXT);
            let mut nll = CompensatedSum::default();
            let mut correct = 0usize;
            for position in 0..EVALUATION_CONTEXT {
                let index = lane * EVALUATION_CONTEXT + position;
                let row_start = index * model.config.vocab_size;
                let row = &probabilities[row_start..row_start + model.config.vocab_size];
                let score = score_probabilities(row, u32::from(tokens[start + position + 1]))?;
                maximum_probability_sum_error =
                    maximum_probability_sum_error.max((score.probability_sum - 1.0).abs());
                let read_row = &reads[index * EVALUATION_CONTEXT..(index + 1) * EVALUATION_CONTEXT];
                if read_row[position..].iter().any(|&mass| mass != 0.0) {
                    return Err(invalid(
                        "joint evaluation contains a current/future memory read",
                    ));
                }
                let audit = read_audit(
                    &read_row[..position],
                    f64::from(no_read[index]),
                    f64::from(gates[index]),
                    mode,
                )?;
                nll.add(score.nll_nats);
                correct += usize::from(score.correct);
                predictions.push(JointTokenPrediction {
                    block_index,
                    position_in_block: position,
                    input_offset: start + position,
                    target_offset: start + position + 1,
                    input_token: u32::from(tokens[start + position]),
                    score,
                    no_read_mass: audit.no_read_mass,
                    copy_gate: audit.copy_gate,
                    effective_copy_mass: audit.effective_copy_mass,
                    read_mass_sum: audit.read_mass_sum,
                    top_read_position: audit.top_read_index,
                    top_read_token: audit.top_read_index.map(|p| u32::from(tokens[start + p])),
                    top_read_mass: audit.top_read_mass,
                });
            }
            let block = JointBlockEvaluation {
                block_index,
                input_start: start,
                first_target_offset: start + 1,
                target_end_exclusive: start + EVALUATION_CONTEXT + 1,
                partition: if block_index < CALIBRATION_BLOCKS {
                    "tune"
                } else {
                    "comparison"
                },
                scored_targets: EVALUATION_CONTEXT,
                nll_sum_nats: nll.total(),
                mean_nll_nats: nll.total() / EVALUATION_CONTEXT as f64,
                correct,
            };
            on_block(&block, &predictions)?;
            full.add(&block);
            if block_index < CALIBRATION_BLOCKS {
                tune.add(&block);
            } else {
                comparison.add(&block);
            }
        }
        completed_batches += 1;
    }
    Ok(JointEvaluation {
        schema: "uor-r4.joint-population-evaluation/1",
        mode,
        input_token_count: tokens.len(),
        context: EVALUATION_CONTEXT,
        stride: EVALUATION_CONTEXT,
        requested_batch_size: batch_size,
        completed_batches,
        completed_blocks: blocks,
        unscored_tail_targets: tokens.len() - 1 - blocks * EVALUATION_CONTEXT,
        full: full.finish(),
        tune: tune.finish(),
        comparison: comparison.finish(),
        maximum_probability_sum_error,
        forward_and_transfer_seconds,
        elapsed_seconds: started.elapsed().as_secs_f64(),
        gradients_tracked: false,
        optimizer_steps: 0,
        complete: true,
    })
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum JointGenerationStop {
    Eos,
    ShortCycle { period: usize },
    MaximumNewTokens,
    FirstSentenceBoundary,
}

#[derive(Clone, Debug, Serialize)]
pub struct JointGenerationDecision {
    pub decision: usize,
    pub input_positions: usize,
    pub selected_token: u32,
    pub greedy_token: u32,
    pub selected_probability: f64,
    pub selected_model_nll_nats: f64,
    pub probability_sum: f64,
    pub probabilities_sha256_le_f32: String,
    pub no_read_mass: f64,
    pub copy_gate: f64,
    pub effective_copy_mass: f64,
    pub top_read_occurrence: Option<usize>,
    pub top_read_token: Option<u32>,
    pub top_read_mass: f64,
    pub sampler_state_before: Option<u64>,
    pub sampler_state_after: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct JointGeneration {
    pub schema: &'static str,
    pub prompt: String,
    pub mode: ReadMode,
    pub seed: Option<u64>,
    pub tokenizer_cid: String,
    pub sampler_policy: &'static str,
    pub bos_policy: &'static str,
    pub prompt_token_ids: Vec<u32>,
    pub generated_token_ids: Vec<u32>,
    pub raw_decoded_bytes: Vec<u8>,
    pub raw_decoded: String,
    pub response_text: String,
    pub utf8_decodable: bool,
    pub stop: JointGenerationStop,
    pub max_new_tokens: usize,
    pub context_capacity: usize,
    pub incremental_step_calls: usize,
    pub fresh_session: bool,
    pub whole_prompt_read_mode: bool,
    pub gradients_tracked: bool,
    pub decisions: Vec<JointGenerationDecision>,
    pub elapsed_seconds: f64,
}

/// Actual incremental generation. `seed=None` is greedy; `Some(seed)` uses the
/// historical top-k/Q32/SplitMix arithmetic on log model probabilities. The
/// probability-based score input is explicit; historical F32-logit equality is
/// not claimed for a different model and output representation.
pub fn generate(
    model: &JointModel,
    tokenizer: &HfBpeTokenizer,
    prompt: &str,
    mode: ReadMode,
    seed: Option<u64>,
    max_new_tokens: usize,
) -> Result<JointGeneration> {
    generate_inner(model, tokenizer, prompt, mode, seed, max_new_tokens, false)
}

fn generate_inner(
    model: &JointModel,
    tokenizer: &HfBpeTokenizer,
    prompt: &str,
    mode: ReadMode,
    seed: Option<u64>,
    max_new_tokens: usize,
    first_sentence: bool,
) -> Result<JointGeneration> {
    let started = Instant::now();
    if !(1..=MAX_NEW_TOKENS).contains(&max_new_tokens)
        || model.config.vocab_size != tokenizer.vocab_size()
    {
        return Err(invalid(
            "joint generation horizon or tokenizer vocabulary differs",
        ));
    }
    let content = tokenizer.encode(prompt);
    if content.is_empty()
        || content
            .iter()
            .any(|&id| id as usize >= model.config.vocab_size)
        || content
            .len()
            .checked_add(1)
            .and_then(|n| n.checked_add(max_new_tokens))
            .is_none_or(|n| n > model.config.context)
    {
        return Err(invalid(
            "joint prompt plus BOS and generation horizon exceeds context",
        ));
    }
    let mut inputs = Vec::with_capacity(content.len() + 1 + max_new_tokens);
    inputs.push(0);
    inputs.extend(content);
    let prompt_token_ids = inputs.clone();
    let mut session = model.new_session(1)?;
    let mut current = None;
    for &token in &inputs {
        current = Some(model.step(&mut session, &[token], mode)?);
    }
    let mut calls = inputs.len();
    let mut generated = Vec::with_capacity(max_new_tokens);
    let mut decisions = Vec::with_capacity(max_new_tokens);
    let mut sampler = seed.map(SplitMix64::new);
    let mut stop = JointGenerationStop::MaximumNewTokens;
    for decision in 0..max_new_tokens {
        let step = current
            .take()
            .ok_or_else(|| invalid("joint generation has no current prediction"))?;
        let (row, audit) = generation_row(&step, &inputs, mode, model.config.vocab_size)?;
        let sampler_state_before = sampler.as_ref().map(|rng| rng.state);
        let selected = match sampler.as_mut() {
            Some(rng) => sample_top_k_q32(&row, rng)?,
            None => row_summary(&row)?.0,
        };
        let score = score_probabilities(&row, selected)?;
        let mut digest = Sha256::new();
        for &value in &row {
            digest.update(value.to_le_bytes());
        }
        let top_read_occurrence = audit
            .top_read_index
            .map(|index| step.read_occurrences[index]);
        decisions.push(JointGenerationDecision {
            decision,
            input_positions: inputs.len(),
            selected_token: selected,
            greedy_token: score.predicted_token,
            selected_probability: score.target_probability,
            selected_model_nll_nats: score.nll_nats,
            probability_sum: score.probability_sum,
            probabilities_sha256_le_f32: hex::encode(digest.finalize()),
            no_read_mass: audit.no_read_mass,
            copy_gate: audit.copy_gate,
            effective_copy_mass: audit.effective_copy_mass,
            top_read_occurrence,
            top_read_token: top_read_occurrence.map(|position| inputs[position]),
            top_read_mass: audit.top_read_mass,
            sampler_state_before,
            sampler_state_after: sampler.as_ref().map(|rng| rng.state),
        });
        generated.push(selected);
        if selected == 1 {
            stop = JointGenerationStop::Eos;
            break;
        }
        if first_sentence && tokenizer.decode_bytes(&generated).contains(&b'.') {
            stop = JointGenerationStop::FirstSentenceBoundary;
            break;
        }
        if let Some(period) = short_cycle_period(&generated) {
            stop = JointGenerationStop::ShortCycle { period };
            break;
        }
        if decision + 1 < max_new_tokens {
            inputs.push(selected);
            current = Some(model.step(&mut session, &[selected], mode)?);
            calls += 1;
        }
    }
    let end = generated
        .iter()
        .position(|&id| id == 1)
        .unwrap_or(generated.len());
    let raw_bytes = tokenizer.decode_bytes(&generated);
    let response_bytes = tokenizer.decode_bytes(&generated[..end]);
    Ok(JointGeneration {
        schema: "uor-r4.joint-incremental-generation/1",
        prompt: prompt.to_owned(),
        mode,
        seed,
        tokenizer_cid: tokenizer.address(),
        sampler_policy: if seed.is_some() {
            SEEDED_POLICY
        } else {
            GREEDY_POLICY
        },
        bos_policy: "prepend checkpoint BOS0 exactly once; EOS1 ends output",
        prompt_token_ids,
        generated_token_ids: generated,
        raw_decoded: String::from_utf8_lossy(&raw_bytes).into_owned(),
        response_text: String::from_utf8_lossy(&response_bytes).trim().to_owned(),
        utf8_decodable: std::str::from_utf8(&raw_bytes).is_ok()
            && std::str::from_utf8(&response_bytes).is_ok(),
        raw_decoded_bytes: raw_bytes,
        stop,
        max_new_tokens,
        context_capacity: model.config.context,
        incremental_step_calls: calls,
        fresh_session: true,
        whole_prompt_read_mode: true,
        gradients_tracked: false,
        decisions,
        elapsed_seconds: started.elapsed().as_secs_f64(),
    })
}

fn generation_row(
    step: &JointStep,
    inputs: &[u32],
    mode: ReadMode,
    vocab: usize,
) -> Result<(Vec<f32>, ReadAudit)> {
    if step.probabilities.track_op()
        || step.probabilities.dims() != [1, vocab]
        || step.no_read_mass.dims() != [1]
        || step.copy_gate.dims() != [1]
        || step.read_masses.dims() != [1, step.read_occurrences.len()]
        || step.written_occurrence != inputs.len() - 1
        || step
            .read_occurrences
            .iter()
            .any(|&position| position >= step.written_occurrence)
        || step
            .read_occurrences
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
    {
        return Err(invalid(
            "joint incremental generation shape, occurrence or gradient contract differs",
        ));
    }
    let probabilities = step.probabilities.flatten_all()?.to_vec1::<f32>()?;
    let masses = step.read_masses.flatten_all()?.to_vec1::<f32>()?;
    let no_read = step.no_read_mass.to_vec1::<f32>()?;
    let gates = step.copy_gate.to_vec1::<f32>()?;
    Ok((
        probabilities,
        read_audit(&masses, f64::from(no_read[0]), f64::from(gates[0]), mode)?,
    ))
}

struct ReadAudit {
    no_read_mass: f64,
    copy_gate: f64,
    effective_copy_mass: f64,
    read_mass_sum: f64,
    top_read_index: Option<usize>,
    top_read_mass: f64,
}

fn read_audit(masses: &[f32], no_read: f64, copy_gate: f64, mode: ReadMode) -> Result<ReadAudit> {
    let valid_mass =
        |value: f64| value.is_finite() && (0.0..=1.0 + PROBABILITY_SUM_TOLERANCE).contains(&value);
    if !valid_mass(no_read) || !valid_mass(copy_gate) {
        return Err(invalid("joint NoRead mass or copy gate is invalid"));
    }
    let mut sum = CompensatedSum::default();
    let mut top = None;
    let mut top_mass = 0.0f64;
    for (index, &mass) in masses.iter().enumerate() {
        let mass = f64::from(mass);
        if !valid_mass(mass) {
            return Err(invalid("joint causal read mass is invalid"));
        }
        sum.add(mass);
        if mass > top_mass {
            top = Some(index);
            top_mass = mass;
        }
    }
    if (sum.total() + no_read - 1.0).abs() > PROBABILITY_SUM_TOLERANCE
        || (mode == ReadMode::NoRead
            && ((no_read - 1.0).abs() > PROBABILITY_SUM_TOLERANCE
                || sum.total() > PROBABILITY_SUM_TOLERANCE))
    {
        return Err(invalid(
            "joint read normalization or full NoRead intervention failed",
        ));
    }
    Ok(ReadAudit {
        no_read_mass: no_read,
        copy_gate,
        effective_copy_mass: copy_gate * (1.0 - no_read),
        read_mass_sum: sum.total(),
        top_read_index: top,
        top_read_mass: top_mass,
    })
}

/// Typed lexical identity authors each accepted completion once. Judging below
/// performs membership only and never rewrites a model output.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StoryItem {
    Apple,
    Pear,
    Ball,
    Kite,
    Book,
    Toy,
    Bell,
    Drum,
    Doll,
    Bear,
    Cup,
    Bowl,
    Hat,
    Cap,
    Shoe,
    Boot,
    Brush,
    Comb,
    Spoon,
    Fork,
    Flower,
    Leaf,
    Key,
    Coin,
    Box,
    Bag,
    Stone,
    Shell,
    Pencil,
    Crayon,
    Rope,
    Ribbon,
}

impl StoryItem {
    pub const fn noun(self) -> &'static str {
        match self {
            Self::Apple => "apple",
            Self::Pear => "pear",
            Self::Ball => "ball",
            Self::Kite => "kite",
            Self::Book => "book",
            Self::Toy => "toy",
            Self::Bell => "bell",
            Self::Drum => "drum",
            Self::Doll => "doll",
            Self::Bear => "bear",
            Self::Cup => "cup",
            Self::Bowl => "bowl",
            Self::Hat => "hat",
            Self::Cap => "cap",
            Self::Shoe => "shoe",
            Self::Boot => "boot",
            Self::Brush => "brush",
            Self::Comb => "comb",
            Self::Spoon => "spoon",
            Self::Fork => "fork",
            Self::Flower => "flower",
            Self::Leaf => "leaf",
            Self::Key => "key",
            Self::Coin => "coin",
            Self::Box => "box",
            Self::Bag => "bag",
            Self::Stone => "stone",
            Self::Shell => "shell",
            Self::Pencil => "pencil",
            Self::Crayon => "crayon",
            Self::Rope => "rope",
            Self::Ribbon => "ribbon",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct StoryVariant {
    pub item: StoryItem,
    pub prompt: String,
    pub accepted_continuations: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct StoryProbe {
    pub id: String,
    pub template: usize,
    pub original: StoryVariant,
    pub edited: StoryVariant,
}

/// The exact 16 pairs are independent of trained weights and model output.
pub fn story_probes() -> Vec<StoryProbe> {
    use StoryItem::*;
    let pairs = [
        (Apple, Pear),
        (Ball, Kite),
        (Book, Toy),
        (Bell, Drum),
        (Doll, Bear),
        (Cup, Bowl),
        (Hat, Cap),
        (Shoe, Boot),
        (Brush, Comb),
        (Spoon, Fork),
        (Flower, Leaf),
        (Key, Coin),
        (Box, Bag),
        (Stone, Shell),
        (Pencil, Crayon),
        (Rope, Ribbon),
    ];
    pairs
        .into_iter()
        .enumerate()
        .map(|(index, (original, edited))| {
            let template = index % 4;
            StoryProbe {
                id: format!("story-source-edit-{index:02}"),
                template,
                original: story_variant(original, template),
                edited: story_variant(edited, template),
            }
        })
        .collect()
}

fn story_variant(item: StoryItem, template: usize) -> StoryVariant {
    let noun = item.noun();
    let prompt = match template {
        0 => format!("One day, Lily carried her {noun} into the garden. She put it on the little table beside the door. Then she ran to play with her friend. They laughed and watched the clouds move across the sky. When it was time to go home, Lily went back to the table and picked up the"),
        1 => format!("Tim wanted to keep his {noun} safe. He put it under the wooden bench before he went outside. He walked around the pond and waved to the ducks. Soon his mother called him home for lunch. Tim remembered what he had left behind. He looked under the bench and found his"),
        2 => format!("Anna brought her {noun} to the park. She left it beside a tall tree while she played with her brother. They ran along the path and listened to the birds. The sun began to set, and they were ready to leave. Anna returned to the tree to collect her"),
        _ => format!("Ben was very happy with his {noun}. Before dinner, he placed it on the kitchen chair. He washed his hands and helped his father with the plates. After they had finished eating, Ben remembered his special thing. He went to the chair and took his"),
    };
    StoryVariant {
        item,
        prompt,
        // The shared oracle's first plain/current form is the bare value.
        // Its location sentence is outside this story-completion protocol.
        accepted_continuations: answer_oracle::accepted(Intent::Current, true, "story-item", noun)
            .into_iter()
            .take(1)
            .map(|answer| answer.trim().to_owned())
            .collect(),
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct JudgedStoryGeneration {
    pub generation: JointGeneration,
    pub first_noun_correct: bool,
    pub complete_correct: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct StoryProbeResult {
    pub schema: &'static str,
    pub scope: &'static str,
    pub stop_policy: &'static str,
    pub probe: StoryProbe,
    pub mode: ReadMode,
    pub original: JudgedStoryGeneration,
    pub edited: JudgedStoryGeneration,
    pub pair_complete_correct: bool,
    pub outputs_differ: bool,
}

pub fn run_story_probes(
    model: &JointModel,
    tokenizer: &HfBpeTokenizer,
    mode: ReadMode,
) -> Result<Vec<StoryProbeResult>> {
    let mut results = Vec::with_capacity(16);
    for probe in story_probes() {
        let original = judge_story(model, tokenizer, &probe.original, mode)?;
        let edited = judge_story(model, tokenizer, &probe.edited, mode)?;
        results.push(StoryProbeResult {
            schema: "uor-r4.joint-story-source-edit/1",
            scope: STORY_PROBE_SCOPE,
            stop_policy: STORY_STOP_POLICY,
            mode,
            pair_complete_correct: original.complete_correct && edited.complete_correct,
            outputs_differ: original.generation.response_text != edited.generation.response_text,
            probe,
            original,
            edited,
        });
    }
    Ok(results)
}

fn judge_story(
    model: &JointModel,
    tokenizer: &HfBpeTokenizer,
    variant: &StoryVariant,
    mode: ReadMode,
) -> Result<JudgedStoryGeneration> {
    let generation = generate_inner(
        model,
        tokenizer,
        &variant.prompt,
        mode,
        None,
        STORY_PROBE_MAX_NEW_TOKENS,
        true,
    )?;
    let first_word = generation
        .response_text
        .split(|c: char| !c.is_alphabetic())
        .next()
        .unwrap_or("");
    let first_noun_correct = first_word == variant.item.noun();
    let complete_correct = generation.utf8_decodable
        && matches!(
            generation.stop,
            JointGenerationStop::Eos | JointGenerationStop::FirstSentenceBoundary
        )
        && answer_oracle::accepts(&variant.accepted_continuations, &generation.response_text);
    Ok(JudgedStoryGeneration {
        generation,
        first_noun_correct,
        complete_correct,
    })
}

/// Same RNG and Q32 draw arithmetic as reference_eval; only the explicit score
/// input changes from raw logits to ln(model probabilities).
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

fn sample_top_k_q32(probabilities: &[f32], sampler: &mut SplitMix64) -> Result<u32> {
    row_summary(probabilities)?;
    let mut ranked = probabilities
        .iter()
        .copied()
        .enumerate()
        .collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0.cmp(&right.0))
    });
    ranked.truncate(40.min(ranked.len()));
    let maximum = f64::from(ranked[0].1).ln();
    let mut weighted = Vec::with_capacity(ranked.len());
    let mut total = 0u64;
    for (token, probability) in ranked {
        let ratio = ((f64::from(probability).ln() - maximum) / 0.8).exp();
        let weight = (ratio * 4_294_967_296.0)
            .round()
            .clamp(1.0, u64::MAX as f64) as u64;
        total = total
            .checked_add(weight)
            .ok_or_else(|| invalid("joint Q32 sampler total overflow"))?;
        weighted.push((token, weight));
    }
    let threshold = ((u128::from(sampler.next_u64()) * u128::from(total)) >> 64) as u64;
    let mut cumulative = 0u64;
    for (token, weight) in &weighted {
        cumulative = cumulative
            .checked_add(*weight)
            .ok_or_else(|| invalid("joint Q32 sampler cumulative overflow"))?;
        if threshold < cumulative {
            return u32::try_from(*token).map_err(|_| invalid("joint sampled token exceeds u32"));
        }
    }
    Err(invalid(
        "joint Q32 draw fell outside normalized integer mass",
    ))
}

#[derive(Default)]
struct PartitionAccumulator {
    blocks: usize,
    targets: usize,
    nll: CompensatedSum,
    correct: usize,
}
impl PartitionAccumulator {
    fn add(&mut self, block: &JointBlockEvaluation) {
        self.blocks += 1;
        self.targets += block.scored_targets;
        self.nll.add(block.nll_sum_nats);
        self.correct += block.correct;
    }
    fn finish(self) -> JointPartitionMetrics {
        let mean = (self.targets > 0).then(|| self.nll.total() / self.targets as f64);
        JointPartitionMetrics {
            blocks: self.blocks,
            scored_targets: self.targets,
            nll_sum_nats: self.nll.total(),
            mean_nll_nats: mean,
            perplexity: mean.map(f64::exp).filter(|p| p.is_finite()),
            correct: self.correct,
            top1_accuracy: (self.targets > 0).then(|| self.correct as f64 / self.targets as f64),
        }
    }
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
