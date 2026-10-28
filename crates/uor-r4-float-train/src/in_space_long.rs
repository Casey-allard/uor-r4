//! Long training of the SpiralCore/FBS geometric trunk in its own space, with text at every
//! checkpoint.
//!
//! The model is the FBS sheet's trunk, `spiralcore_fbs_sheet::build_fbs_model(OneAtomActive)`.
//! - **Space:** 6 recurrence layers with the Hamilton gate and SpiralCore/FBS projections.
//! - **One table:** the native vector writer's table encodes the input token and holds every
//!   output token's vector.
//! - **Loss:** the writer's `MarginHinge` on the L1 displacement between the predicted point and
//!   the target's own vector, so training happens in the same space as transport and prediction.
//! - **Tokens:** they appear only at the writer's final nearest pick.
//!
//! The sheet trained this trunk for 1,024 steps of 4 windows: about 4,096 of TRAIN's ~39k usable
//! windows, seen once. This runner instead trains for `steps` steps, over epochs of every usable
//! window, each epoch in its own seeded order. Every `eval_every` steps it does three things:
//! 1. scores the whole DEV panel;
//! 2. writes greedy replies to fixed DEV prompts as text, beside the true replies;
//! 3. saves the model.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::{DType, Device, Tensor};
use serde::Serialize;
use uor_r4_tokenizer::ByteBpeTokenizer;

use crate::geometric_stack::{StackAdamW, StackModel};
use crate::hamilton_sheet::{self as hs, CausalWindow, HamiltonArm, SHEET_CONTEXT};
use crate::spiralcore_fbs_arms::{self as arms, FbsArm};
use crate::spiralcore_fbs_sheet::{build_fbs_model_at, panel_loss};
use crate::{invalid, sha256_bytes, Result};

/// What the trunk's predicted point is trained against.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PointObjective {
    /// The writer's own margin: the token hinge plus the address hinge at margin 1. The 16 roots
    /// cannot meet that margin: by the triangle inequality a cell beats the cell sharing its other
    /// port by at most the nearest root separation, 0.5 (`root_separation`).
    Sheet,
    /// The same token hinge, with the address hinge's margin at half the nearest root separation,
    /// which a query port meets inside its target root's Voronoi cell.
    Voronoi,
    /// No admission: the target must be nearer than the `SHEET_RIVALS` nearest wrong tokens of the
    /// whole table, and the reply is the whole table's nearest token.
    Nearest,
    /// The target token's cell: the predicted point must be a margin nearer the target than the
    /// whole table's single nearest wrong token, `relu(1 + max_r s(r) − s(target))`, so only the
    /// rival that would win is pushed. Decoded as [`PointObjective::Nearest`] is (claude-152).
    Cell,
}

impl PointObjective {
    /// Whether replies decode as the whole table's nearest token, not the writer's pick.
    pub fn decodes_nearest(self) -> bool {
        matches!(self, Self::Nearest | Self::Cell)
    }
}

/// One TRAIN split other than the pinned one, named by its directory and checked by the sha256s
/// of its `tokens.u16` and `response_mask.u8` (claude-152).
#[derive(Clone, Debug, Serialize)]
pub struct TrainSplit {
    pub dir: PathBuf,
    pub tokens_sha256: String,
    pub mask_sha256: String,
}

/// One run's settings.
#[derive(Clone, Debug, Serialize)]
pub struct LongConfig {
    pub objective: PointObjective,
    pub steps: usize,
    pub batch: usize,
    pub eval_every: usize,
    pub seed: u64,
    pub lr: f64,
    /// The learning rate falls linearly from `lr` at step 0 to `lr_end` at the last step.
    pub lr_end: f64,
    /// The `rec.decay` parameters' rate as a multiple of the main rate (the sheet's is 20).
    pub decay_mult: f64,
    /// Read the recurrence's rate table as retention (`geometric_stack::set_retention_rates`).
    pub retain_rates: bool,
    /// Fold the recurrence's rate score into [0, 1] (`geometric_stack::set_folded_scores`).
    pub fold_scores: bool,
    /// A reflecting floor and clamped ceiling for the rate score (`geometric_stack::set_reflected_floor`).
    pub reflect_floor: bool,
    /// Keep this many earlier positions in a Flock read on the trunk's points; 0 is no read
    /// (`flock_read`).
    pub flock_read: usize,
    /// Keep this many earlier positions in a content-keyed read inside the trunk; 0 is no read
    /// (`flock_read_content`).
    pub content_read: usize,
    /// Hamming flock attention with this many heads after layer 2 and on the readout; 0 is no read
    /// (`flock_read_hamming`).
    pub hamming_read: usize,
    /// The opening read (claude-154) after layer 2: the mean of up to this many opening token
    /// points; 0 is no read (`flock_read_opening`). Left out of `config.json` when 0.
    #[serde(skip_serializing_if = "is_zero")]
    pub opening_read: usize,
    /// Where the opening read's slots start; set exactly when `opening_read` is, and left out of
    /// `config.json` when none, so a run without the read writes the same file as before.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opening_anchor: Option<crate::flock_read_opening::OpeningAnchor>,
    /// The trunk's width: the shared table's columns and every read's width (default 256,
    /// `hamilton_sheet::SHEET_WIDTH`); a multiple of 8.
    pub width: usize,
    /// The Hamming read's type heads compare positions through SpiralCore's path-dependent
    /// icosian transport (`HammingReads::with_type_spin`, claude-151); off by default.
    pub type_spin: bool,
    /// Pass no gradient from the input encoding into the shared table
    /// (`geometric_stack::set_input_table_fixed`); it changes training only, never a forward.
    pub fix_input_table: bool,
    /// Train on this split in place of the pinned TRAIN; DEV stays the pinned one.
    pub train_split: Option<TrainSplit>,
    pub samples: usize,
    pub reply_tokens: usize,
}

/// One checkpoint's reading, one line of `progress.jsonl`.
#[derive(Clone, Debug, Serialize)]
pub struct Checkpoint {
    pub step: usize,
    pub epoch: f64,
    pub train_loss_mean: Option<f64>,
    /// The writer's own DEV panel (`hamilton_sheet::evaluate_dev_panel`): its top-1, top-8 and
    /// admitted counts, and `dev_loss` with the token and address margins below, read the trunk
    /// without the Flock read (a content read, being inside the trunk, is in them). `dev_nearest`
    /// goes through [`crate::flock_read::points`].
    pub dev_top1_hits: usize,
    pub dev_top8_hits: usize,
    pub dev_admitted_hits: usize,
    /// DEV decoded as the whole table's nearest token to the predicted point.
    pub dev_nearest: NearestReading,
    /// DEV decoded by the writer, the served decoder, on the same points and rows as
    /// `dev_nearest`; its `hits` must equal `dev_top1_hits`.
    pub dev_writer: WriterReading,
    pub dev_scored_rows: usize,
    pub dev_loss: f64,
    /// DEV's token hinge: the target's displacement score against the best admitted rivals.
    pub dev_token_margin: f64,
    /// DEV's address hinge: the target's cell against the best other admitted cell.
    pub dev_address_margin: f64,
    pub train_seconds: f64,
    /// Every context position of every trained window per second (not scored targets).
    pub train_context_tokens_per_second: f64,
}

/// One saved checkpoint's DEV reading, with the loss in its two parts and how varied the
/// predicted points are across DEV rows (`inspect_run`).
#[derive(Clone, Debug, Serialize)]
pub struct Inspection {
    pub checkpoint: String,
    pub scored_rows: usize,
    pub top1_hits: usize,
    pub nearest: NearestReading,
    pub admitted_hits: usize,
    pub token_unviolated_hits: usize,
    pub token_margin: f64,
    pub address_margin: f64,
    /// Over the first DEV windows' scored rows: how many distinct cells are admitted first.
    pub probe_rows: usize,
    pub distinct_first_cells: usize,
    /// Each query dimension's standard deviation across those rows, averaged over dimensions.
    pub query_spread: f64,
    pub query_mean_abs: f64,
    pub table_mean_abs: f64,
    /// The Flock read's per-coordinate weight, mean of its absolute values, when the checkpoint
    /// has a read.
    pub flock_weight_mean_abs: Option<f64>,
    /// The same for the content read, when the checkpoint has one.
    pub content_weight_mean_abs: Option<f64>,
    /// The same for the Hamming reads, over both depths, when the checkpoint has them.
    pub hamming_weight_mean_abs: Option<f64>,
    /// The same for the opening read (claude-154), when the checkpoint has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opening_weight_mean_abs: Option<f64>,
}

/// One fixed DEV prompt with the model's greedy reply and the true reply, as text.
#[derive(Clone, Debug, Serialize)]
pub struct Sample {
    pub step: usize,
    pub prompt: String,
    pub model: String,
    pub reference: String,
}

/// Clears the arm hook and the retention reading when the run ends, by any path.
struct ArmGuard;

impl Drop for ArmGuard {
    fn drop(&mut self) {
        arms::set_fbs_arm(None);
        crate::geometric_stack::set_retention_rates(false);
        crate::geometric_stack::set_folded_scores(false);
        crate::geometric_stack::set_reflected_floor(false);
        crate::geometric_stack::set_input_table_fixed(false);
        crate::flock_read::set_flock_read(None);
        crate::flock_read_content::set_content_read(None);
        crate::flock_read_hamming::set_hamming_reads(None);
        crate::flock_read_opening::set_opening_read(None);
    }
}

/// Set this thread's Flock read, content read, Hamming reads and opening read from the checkpoint
/// `directory`, each to none when the checkpoint has none.
fn load_reads(directory: &Path, device: &Device) -> Result<()> {
    crate::flock_read::set_flock_read(crate::flock_read::FlockRead::load(directory, device)?);
    crate::flock_read_content::set_content_read(crate::flock_read_content::ContentRead::load(
        directory, device,
    )?);
    crate::flock_read_hamming::set_hamming_reads(crate::flock_read_hamming::HammingReads::load(
        directory, device,
    )?);
    crate::flock_read_opening::set_opening_read(crate::flock_read_opening::OpeningRead::load(
        directory, device,
    )?);
    Ok(())
}

/// The Hamming reads `config` asks for, set on this thread, with their optimizer; none when off.
fn start_hamming(
    config: &LongConfig,
    device: &Device,
) -> Result<
    Option<(
        crate::flock_read_hamming::HammingReads,
        crate::flock_read::FlockAdamW,
    )>,
> {
    if config.hamming_read == 0 {
        return Ok(None);
    }
    let mut reads =
        crate::flock_read_hamming::HammingReads::new(config.hamming_read, config.width, device)?;
    if config.type_spin {
        reads = reads.with_type_spin()?;
    }
    crate::flock_read_hamming::set_hamming_reads(Some(reads.clone()));
    let opt = crate::flock_read::FlockAdamW::for_vars(&reads.variables())?;
    Ok(Some((reads, opt)))
}

/// The opening read `config` asks for (claude-154), set on this thread, with its own optimizer;
/// none when off.
fn start_opening(
    config: &LongConfig,
    device: &Device,
) -> Result<
    Option<(
        crate::flock_read_opening::OpeningRead,
        crate::flock_read::FlockAdamW,
    )>,
> {
    if config.opening_read == 0 {
        return Ok(None);
    }
    let read = crate::flock_read_opening::OpeningRead::new(
        config.opening_read,
        config.width,
        config.opening_anchor.unwrap_or_default(),
        device,
    )?;
    crate::flock_read_opening::set_opening_read(Some(read.clone()));
    let opt = crate::flock_read::FlockAdamW::for_vars(&read.variables())?;
    Ok(Some((read, opt)))
}

fn is_zero(n: &usize) -> bool {
    *n == 0
}

const ABSENT: f32 = -1.0e30;
/// At most this many F32 elements in one detached whole-table score chunk (256 MiB).
const NEAREST_CHUNK_ELEMENTS: usize = 64 * 1024 * 1024;

fn table(model: &StackModel) -> Result<Tensor> {
    let var = model
        .variables()
        .get("embedding.weight")
        .ok_or_else(|| invalid("the model has no shared vector table"))?;
    Ok(var.as_tensor().clone())
}

/// `relu(margin + rival − target)`: `rival` is (rows, k), `target` is (rows, 1).
fn hinge(rival: &Tensor, target: &Tensor, margin: f64) -> Result<Tensor> {
    Ok(rival.broadcast_sub(target)?.affine(1.0, margin)?.relu()?)
}

/// The `k` best positions of each row of `scores` among those `allowed` (u8, same shape) keeps,
/// as (rows, k), by repeated argmax on a detached copy.
fn best_positions(scores: &Tensor, allowed: &Tensor, k: usize) -> Result<Tensor> {
    let absent = Tensor::full(ABSENT, scores.shape(), scores.device())?;
    let mut left = allowed.where_cond(&scores.detach(), &absent)?;
    let columns = Tensor::arange(0u32, scores.dim(1)? as u32, scores.device())?
        .unsqueeze(0)?
        .broadcast_as(scores.shape())?;
    let mut picks = Vec::with_capacity(k);
    for _ in 0..k {
        let at = left.argmax_keepdim(1)?;
        left = columns.broadcast_eq(&at)?.where_cond(&absent, &left)?;
        picks.push(at);
    }
    Ok(Tensor::cat(&picks, 1)?)
}

/// The negative mean L1 displacement of each `query` row to every table row, without gradient,
/// in row chunks: (rows, vocabulary).
fn nearest_scores(query: &Tensor, table: &Tensor) -> Result<Tensor> {
    let (rows, width) = query.dims2()?;
    let chunk = (NEAREST_CHUNK_ELEMENTS / (table.dim(0)? * width)).max(1);
    let table = table.detach().unsqueeze(0)?;
    let mut parts = Vec::with_capacity(rows.div_ceil(chunk));
    let mut start = 0;
    while start < rows {
        let n = chunk.min(rows - start);
        let q = query.narrow(0, start, n)?.detach().unsqueeze(1)?;
        parts.push(q.broadcast_sub(&table)?.abs()?.mean(2)?.neg()?);
        start += n;
    }
    Ok(Tensor::cat(&parts, 0)?)
}

/// [`PointObjective::Voronoi`]: the writer's token hinge, and its address hinge at `address_margin`.
fn voronoi_loss(
    model: &StackModel,
    query: &Tensor,
    targets: &[u32],
    address_margin: f64,
) -> Result<Tensor> {
    let writer = model.native_vector_writer()?;
    let c = writer.candidates(query)?;
    let rows = targets.len();
    let device = query.device();
    let target_score = query
        .sub(&writer.encode(targets)?)?
        .abs()?
        .mean_keepdim(1)?
        .neg()?;
    let target_ids = Tensor::from_vec(targets.to_vec(), (rows, 1), device)?;
    // Every one of the 256 cells holds 16 real tokens at this vocabulary, so 8 kept cells always
    // leave at least `SHEET_RIVALS` non-target candidates.
    let available = c.token_ids().broadcast_ne(&target_ids)?.mul(c.valid())?;
    let at = best_positions(c.raw_scores(), &available, hs::SHEET_RIVALS)?;
    let token = hinge(&c.raw_scores().gather(&at, 1)?, &target_score, 1.0)?.mean(1)?;
    let cells = crate::native_vector_writer::CELLS as u32;
    let target_cells = Tensor::from_vec(
        targets.iter().map(|t| t % cells).collect::<Vec<_>>(),
        (rows, 1),
        device,
    )?;
    let target_address = c.address_scores().gather(&target_cells, 1)?;
    let admitted = c.address_scores().gather(c.cell_ids(), 1)?;
    let rival_address = c
        .cell_ids()
        .broadcast_ne(&target_cells)?
        .where_cond(&admitted, &Tensor::full(ABSENT, admitted.shape(), device)?)?
        .max_keepdim(1)?;
    let address = hinge(&rival_address, &target_address, address_margin)?;
    Ok(token.mean_all()?.add(&address.mean_all()?)?)
}

/// [`PointObjective::Nearest`] (each of the `rival` scores' hinges, averaged) or
/// [`PointObjective::Cell`] (only the hinge of the best-scoring rival), against `target`.
fn rival_hinge(rival: &Tensor, target: &Tensor, cell: bool) -> Result<Tensor> {
    if cell {
        Ok(hinge(&rival.max_keepdim(1)?, target, 1.0)?.mean_all()?)
    } else {
        Ok(hinge(rival, target, 1.0)?.mean_all()?)
    }
}

/// [`PointObjective::Nearest`] or [`PointObjective::Cell`]: the target against the whole table's
/// `SHEET_RIVALS` nearest wrong tokens. Their choice carries no gradient; their displacement and
/// the target's do.
fn nearest_loss(model: &StackModel, query: &Tensor, targets: &[u32], cell: bool) -> Result<Tensor> {
    let table = table(model)?;
    let (rows, width) = query.dims2()?;
    let device = query.device();
    let scores = nearest_scores(query, &table)?;
    let target_ids = Tensor::from_vec(targets.to_vec(), (rows, 1), device)?;
    let allowed = Tensor::arange(0u32, table.dim(0)? as u32, device)?
        .unsqueeze(0)?
        .broadcast_as(scores.shape())?
        .broadcast_ne(&target_ids)?;
    let rivals = best_positions(&scores, &allowed, hs::SHEET_RIVALS)?;
    let rival_vectors =
        table
            .index_select(&rivals.flatten_all()?, 0)?
            .reshape((rows, hs::SHEET_RIVALS, width))?;
    let rival = query
        .unsqueeze(1)?
        .broadcast_sub(&rival_vectors)?
        .abs()?
        .mean(2)?
        .neg()?;
    let target_score = query
        .sub(&table.index_select(&target_ids.flatten_all()?, 0)?)?
        .abs()?
        .mean_keepdim(1)?
        .neg()?;
    rival_hinge(&rival, &target_score, cell)
}

fn batch_loss(
    model: &StackModel,
    objective: PointObjective,
    ids: &[u32],
    scored: &[u32],
    targets: &[u32],
    batch: usize,
) -> Result<Tensor> {
    let hidden = crate::flock_read::points(model, ids, batch, SHEET_CONTEXT)?;
    let idx = Tensor::from_vec(scored.to_vec(), scored.len(), model.device())?;
    query_loss(model, objective, &hidden.index_select(&idx, 0)?, targets)
}

/// `objective`'s loss for the predicted points `query`, one row per target.
fn query_loss(
    model: &StackModel,
    objective: PointObjective,
    query: &Tensor,
    targets: &[u32],
) -> Result<Tensor> {
    match objective {
        PointObjective::Sheet => model.native_head_loss(query, targets, hs::SHEET_RIVALS),
        PointObjective::Voronoi => voronoi_loss(model, query, targets, root_separation()?.0 / 2.0),
        PointObjective::Nearest => nearest_loss(model, query, targets, false),
        PointObjective::Cell => nearest_loss(model, query, targets, true),
    }
}

/// Whole-table nearest decoding over DEV. Rows that open a reply are counted apart: their target
/// follows the prompt, not the reply's own earlier words, so only context can name it.
#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct NearestReading {
    pub hits: usize,
    pub opening_rows: usize,
    pub opening_hits: usize,
    /// Openings whose target is DEV's most common opening token: what always naming it scores.
    pub commonest_opening_rows: usize,
    /// How many distinct tokens the openings are decoded as.
    pub opening_distinct_picks: usize,
    /// Each query dimension's standard deviation across the opening rows, and across all rows,
    /// averaged over dimensions: whether the prompt moves the opening point at all.
    pub opening_query_spread: f64,
    pub query_spread: f64,
    /// Reply rows that copy the prompt ([`copies_prompt`]): the reply's previous token and its
    /// target stand side by side in the window's prompt, as in a code word or a number the user
    /// gave. Only a model that reads the prompt names these above chance (claude-152).
    pub copied_rows: usize,
    pub copied_hits: usize,
    /// Reply rows that neither open a reply nor copy the prompt.
    pub rest_rows: usize,
    pub rest_hits: usize,
}

/// Whether the reply row whose input sits at window `offset` (target at corpus `position`) copies
/// the window's prompt: the row's previous token and its target stand at two adjacent earlier
/// positions of the window that are both outside any reply. `mask` is the corpus response mask.
fn copies_prompt(
    window: &CausalWindow,
    offset: usize,
    position: usize,
    target: u32,
    mask: &[u8],
) -> bool {
    // The window's offset 0 sits at corpus position `position − 1 − offset`.
    let start = position - 1 - offset;
    let previous = window.input_ids[offset];
    (0..offset.saturating_sub(1)).any(|j| {
        window.input_ids[j] == previous
            && window.input_ids[j + 1] == target
            && mask[start + j] == 0
            && mask[start + j + 1] == 0
    })
}

/// The writer's decoding of the same DEV rows as [`NearestReading`]: the token the served
/// decoder (`native_vector_writer`, the one `fbs-toks` decodes with) picks for each scored row.
/// Its `hits` must equal the writer panel's own top-1 (`dev_top1_hits`) on the same checkpoint.
#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct WriterReading {
    pub hits: usize,
    pub opening_hits: usize,
    /// How many distinct tokens the openings are decoded as.
    pub opening_distinct_picks: usize,
}

/// Each column's standard deviation across `rows`' rows, averaged over columns.
fn spread(rows: &Tensor) -> Result<f64> {
    let centred = rows.broadcast_sub(&rows.mean_keepdim(0)?)?;
    Ok(f64::from(
        centred
            .sqr()?
            .mean(0)?
            .sqrt()?
            .mean_all()?
            .to_scalar::<f32>()?,
    ))
}

/// [`NearestReading`] over the DEV windows, and the writer's [`WriterReading`] of the same rows
/// from the same points; `mask` is DEV's response mask, by corpus position.
fn nearest_reading(
    model: &StackModel,
    dev: &[CausalWindow],
    mask: &[u8],
    seed: u64,
    step: usize,
) -> Result<(NearestReading, WriterReading)> {
    let table = table(model)?;
    let writer = model.native_vector_writer()?;
    hs::with_gate_mode(HamiltonArm::HamiltonActive, seed, step, false, || {
        let mut reading = NearestReading::default();
        let mut written = WriterReading::default();
        let mut openings = std::collections::BTreeMap::<u32, usize>::new();
        let mut picks = std::collections::BTreeSet::<u32>::new();
        let mut written_picks = std::collections::BTreeSet::<u32>::new();
        let (mut all_queries, mut opening_queries) = (Vec::new(), Vec::new());
        for chunk in dev.chunks(hs::SHEET_EVAL_BATCH) {
            let windows: Vec<&CausalWindow> = chunk.iter().collect();
            let (ids, scored, targets) = assemble(&windows);
            if scored.is_empty() {
                continue;
            }
            let opens: Vec<bool> = windows
                .iter()
                .flat_map(|w| w.scored_corpus_positions.iter())
                .map(|&p| opens_reply(mask, p))
                .collect();
            let copies: Vec<bool> = windows
                .iter()
                .flat_map(|w| {
                    w.scored_offsets
                        .iter()
                        .zip(&w.scored_corpus_positions)
                        .zip(&w.scored_targets)
                        .map(move |((&o, &p), &t)| copies_prompt(w, o, p, t, mask))
                })
                .collect();
            let hidden = crate::flock_read::points(model, &ids, windows.len(), SHEET_CONTEXT)?;
            let idx = Tensor::from_vec(scored.clone(), scored.len(), model.device())?;
            let query = hidden.index_select(&idx, 0)?.detach();
            let best = nearest_scores(&query, &table)?
                .argmax(1)?
                .to_vec1::<u32>()?;
            // The served decoder on the same points: the writer's pick among its kept cells.
            let served = writer
                .write(&query)?
                .flatten_all()?
                .to_dtype(DType::U32)?
                .to_vec1::<u32>()?;
            let rows = best
                .iter()
                .zip(&served)
                .zip(&targets)
                .zip(&opens)
                .zip(&copies);
            for ((((pick, served), target), open), copy) in rows {
                reading.hits += usize::from(pick == target);
                written.hits += usize::from(served == target);
                if *copy {
                    reading.copied_rows += 1;
                    reading.copied_hits += usize::from(pick == target);
                } else if !*open {
                    reading.rest_rows += 1;
                    reading.rest_hits += usize::from(pick == target);
                }
                if *open {
                    reading.opening_rows += 1;
                    reading.opening_hits += usize::from(pick == target);
                    written.opening_hits += usize::from(served == target);
                    *openings.entry(*target).or_default() += 1;
                    picks.insert(*pick);
                    written_picks.insert(*served);
                }
            }
            let at: Vec<u32> = (0..opens.len() as u32)
                .filter(|&i| opens[i as usize])
                .collect();
            if !at.is_empty() {
                let at = Tensor::from_vec(at.clone(), at.len(), model.device())?;
                opening_queries.push(query.index_select(&at, 0)?);
            }
            all_queries.push(query);
        }
        reading.commonest_opening_rows = openings.values().copied().max().unwrap_or(0);
        reading.opening_distinct_picks = picks.len();
        written.opening_distinct_picks = written_picks.len();
        if !opening_queries.is_empty() {
            reading.opening_query_spread = spread(&Tensor::cat(&opening_queries, 0)?)?;
        }
        if !all_queries.is_empty() {
            reading.query_spread = spread(&Tensor::cat(&all_queries, 0)?)?;
        }
        Ok((reading, written))
    })
}

/// A seeded order of `0..n` (xorshift64*, Fisher-Yates), one per epoch.
fn epoch_order(n: usize, seed: u64, epoch: u64) -> Vec<usize> {
    let mut state = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(epoch.wrapping_mul(0xD1B5_4A32_D192_ED03))
        | 1;
    let mut next = || {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        state.wrapping_mul(0x2545_F491_4F6C_DD1D)
    };
    let mut order: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let j = (next() % (i as u64 + 1)) as usize;
        order.swap(i, j);
    }
    order
}

/// The flat ids, scored indices and targets of `windows`, as the sheet's batches lay them out.
fn assemble(windows: &[&CausalWindow]) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
    let mut ids = Vec::with_capacity(windows.len() * SHEET_CONTEXT);
    let (mut scored, mut targets) = (Vec::new(), Vec::new());
    for (b, window) in windows.iter().enumerate() {
        ids.extend_from_slice(&window.input_ids);
        for (&off, &target) in window.scored_offsets.iter().zip(&window.scored_targets) {
            scored.push((b * SHEET_CONTEXT + off) as u32);
            targets.push(target);
        }
    }
    (ids, scored, targets)
}

/// Whether the target at corpus `position` opens a reply: the token before it is not a response
/// token, so the target follows the prompt and not the reply's own earlier words.
fn opens_reply(mask: &[u8], position: usize) -> bool {
    position > 0 && mask[position - 1] == 0
}

/// How well the prompt words a window shows could name its reply's first token: TRAIN's
/// commonest opening for each key, scored on DEV's openings ([`opening_bound`]). A count over the
/// corpus, not a model: it bounds what a read that finds those words could add to the openings.
#[derive(Clone, Debug, Serialize)]
pub struct OpeningBound {
    /// `none`, `first-k` (the first `k` prompt tokens the window shows) or `last-k` (the `k`
    /// tokens before the opening).
    pub key: String,
    pub train_rows: usize,
    pub train_keys: usize,
    pub dev_rows: usize,
    /// DEV openings whose key TRAIN never saw; TRAIN's commonest opening names them.
    pub dev_unseen: usize,
    pub dev_hits: usize,
    /// TRAIN's most frequent keys, decoded, with their commonest opening and its share.
    pub top: Vec<OpeningKey>,
}

#[derive(Clone, Debug, Serialize)]
pub struct OpeningKey {
    pub key: String,
    pub opening: String,
    pub train_rows: usize,
    pub opening_rows: usize,
}

/// Each opening in `windows`: the prompt tokens its window shows before it, in order, and its
/// target. The prompt runs back from the opening to the last response token or BOS.
fn openings(windows: &[CausalWindow], mask: &[u8]) -> Vec<(Vec<u32>, u32)> {
    opening_rows(windows, mask)
        .into_iter()
        .map(|row| (row.prompt, row.target))
        .collect()
}

/// One opening [`openings`] yields, with where it stands: its window's index and the offset of
/// the input token it is predicted from.
struct OpeningRow {
    window: usize,
    offset: usize,
    prompt: Vec<u32>,
    target: u32,
}

fn opening_rows(windows: &[CausalWindow], mask: &[u8]) -> Vec<OpeningRow> {
    let mut rows = Vec::new();
    for (index, w) in windows.iter().enumerate() {
        let scored = w
            .scored_offsets
            .iter()
            .zip(&w.scored_targets)
            .zip(&w.scored_corpus_positions);
        for ((&offset, &target), &position) in scored {
            if !opens_reply(mask, position) {
                continue;
            }
            // The corpus position of `input_ids[0]`.
            let base = position - 1 - offset;
            let mut first = offset + 1;
            while first > 0 {
                let i = first - 1;
                if w.input_ids[i] == hs::SHEET_BOS_ID || mask[base + i] == 1 {
                    break;
                }
                first = i;
            }
            rows.push(OpeningRow {
                window: index,
                offset,
                prompt: w.input_ids[first..=offset].to_vec(),
                target,
            });
        }
    }
    rows
}

/// How often the opening read's turn anchor (claude-154) keeps, at an opening, the same tokens
/// as the probe's first-8 key: `opening_slots(input_ids, 8, Turn)[offset]` as tokens against
/// `prompt[..8]` ([`opening_bound`]).
#[derive(Clone, Debug, Serialize)]
pub struct AnchorCheck {
    /// The split, `dev` or `train`.
    pub anchor_check: String,
    pub openings: usize,
    pub agree: usize,
}

fn anchor_check(split: &str, windows: &[CausalWindow], mask: &[u8]) -> AnchorCheck {
    use crate::flock_read_opening::{opening_slots, OpeningAnchor};
    let key = PromptKey::First(8);
    let rows = opening_rows(windows, mask);
    let mut agree = 0;
    let mut slots = (usize::MAX, Vec::new());
    for row in &rows {
        // Rows come window by window, so each window's slots are worked out once.
        let w = &windows[row.window];
        if slots.0 != row.window {
            slots = (
                row.window,
                opening_slots(&w.input_ids, 8, OpeningAnchor::Turn),
            );
        }
        let kept: Vec<u32> = slots.1[row.offset]
            .iter()
            .map(|&s| w.input_ids[s])
            .collect();
        agree += usize::from(kept == key.of(&row.prompt));
    }
    AnchorCheck {
        anchor_check: split.to_owned(),
        openings: rows.len(),
        agree,
    }
}

/// The commonest target in `counts` and its count, ties to the smaller token id.
fn commonest(counts: &std::collections::BTreeMap<u32, usize>) -> Option<(u32, usize)> {
    counts
        .iter()
        .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
        .map(|(&t, &c)| (t, c))
}

/// Which prompt tokens an [`OpeningBound`] keys on.
#[derive(Clone, Copy, Debug)]
enum PromptKey {
    None,
    First(usize),
    Last(usize),
}

impl PromptKey {
    fn of(self, prompt: &[u32]) -> &[u32] {
        match self {
            PromptKey::None => &[],
            PromptKey::First(k) => &prompt[..k.min(prompt.len())],
            PromptKey::Last(k) => &prompt[prompt.len() - k.min(prompt.len())..],
        }
    }

    fn name(self) -> String {
        match self {
            PromptKey::None => "none".to_owned(),
            PromptKey::First(k) => format!("first-{k}"),
            PromptKey::Last(k) => format!("last-{k}"),
        }
    }
}

fn bound(
    key: PromptKey,
    train: &[(Vec<u32>, u32)],
    dev: &[(Vec<u32>, u32)],
    decode: &dyn Fn(&[u32]) -> String,
) -> OpeningBound {
    let pick = |prompt| key.of(prompt);
    let mut table =
        std::collections::BTreeMap::<Vec<u32>, std::collections::BTreeMap<u32, usize>>::new();
    let mut overall = std::collections::BTreeMap::<u32, usize>::new();
    for (prompt, target) in train {
        *table
            .entry(pick(prompt).to_vec())
            .or_default()
            .entry(*target)
            .or_default() += 1;
        *overall.entry(*target).or_default() += 1;
    }
    let fallback = commonest(&overall).map(|(t, _)| t);
    let (mut dev_unseen, mut dev_hits) = (0, 0);
    for (prompt, target) in dev {
        let named = match table.get(pick(prompt)) {
            Some(counts) => commonest(counts).map(|(t, _)| t),
            None => {
                dev_unseen += 1;
                fallback
            }
        };
        dev_hits += usize::from(named == Some(*target));
    }
    let mut keys: Vec<(&Vec<u32>, usize)> = table
        .iter()
        .map(|(k, counts)| (k, counts.values().sum()))
        .collect();
    keys.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    let top = keys
        .into_iter()
        .take(8)
        .filter_map(|(k, rows)| {
            let (opening, opening_rows) = commonest(&table[k])?;
            Some(OpeningKey {
                key: decode(k),
                opening: decode(&[opening]),
                train_rows: rows,
                opening_rows,
            })
        })
        .collect();
    OpeningBound {
        key: key.name(),
        train_rows: train.len(),
        train_keys: table.len(),
        dev_rows: dev.len(),
        dev_unseen,
        dev_hits,
        top,
    }
}

/// [`OpeningBound`] for no key, the first 1–8 prompt tokens and the last 1–4, and the opening
/// read's [`AnchorCheck`] on DEV and on TRAIN.
pub fn opening_bound(data_root: &Path) -> Result<(Vec<OpeningBound>, Vec<AnchorCheck>)> {
    let tokenizer = pinned_tokenizer(data_root)?;
    let decode = |ids: &[u32]| tail(&tokenizer.decode(ids), 80);
    let (train_tokens, train_mask) = load_train_split(data_root)?;
    let (train_windows, _) =
        hs::build_document_windows(&train_tokens, &train_mask, SHEET_CONTEXT, hs::SHEET_BOS_ID)?;
    let (dev_windows, dev_mask) = load_dev(data_root)?;
    let train = openings(&train_windows, &train_mask);
    let dev = openings(&dev_windows, &dev_mask);
    let keys = std::iter::once(PromptKey::None)
        .chain((1..=8).map(PromptKey::First))
        .chain((1..=4).map(PromptKey::Last));
    let checks = vec![
        anchor_check("dev", &dev_windows, &dev_mask),
        anchor_check("train", &train_windows, &train_mask),
    ];
    Ok((
        keys.map(|key| bound(key, &train, &dev, &decode)).collect(),
        checks,
    ))
}

/// `samples` DEV windows spread over the panel whose first scored target opens a reply, with at
/// least 16 tokens of prompt before it. `mask` is DEV's response mask.
fn sample_windows<'a>(
    dev: &'a [CausalWindow],
    mask: &[u8],
    samples: usize,
) -> Vec<&'a CausalWindow> {
    let eligible: Vec<&CausalWindow> = dev
        .iter()
        .filter(|w| w.scored_offsets.first().is_some_and(|&off| off >= 16))
        .filter(|w| {
            w.scored_corpus_positions
                .first()
                .is_some_and(|&p| opens_reply(mask, p))
        })
        .collect();
    if eligible.is_empty() || samples == 0 {
        return Vec::new();
    }
    let stride = (eligible.len() / samples).max(1);
    eligible.into_iter().step_by(stride).take(samples).collect()
}

/// The model's greedy reply to `prompt`: `reply_tokens` tokens on the last `SHEET_CONTEXT`
/// tokens, stopping early at BOS. Each is the whole table's nearest token under
/// [`PointObjective::Nearest`] or [`PointObjective::Cell`], else the writer's pick among its
/// admitted cells.
fn greedy_reply(
    model: &StackModel,
    objective: PointObjective,
    prompt: &[u32],
    reply_tokens: usize,
    seed: u64,
) -> Result<Vec<u32>> {
    let table = table(model)?;
    hs::with_gate_mode(HamiltonArm::HamiltonActive, seed, 0, false, || {
        let mut ids = prompt.to_vec();
        let mut reply = Vec::with_capacity(reply_tokens);
        for _ in 0..reply_tokens {
            let window = &ids[ids.len().saturating_sub(SHEET_CONTEXT)..];
            let next = if objective.decodes_nearest() {
                let hidden = crate::flock_read::points(model, window, 1, window.len())?;
                nearest_scores(&hidden.narrow(0, window.len() - 1, 1)?, &table)?
                    .argmax(1)?
                    .to_vec1::<u32>()?
            } else {
                model
                    .native_next_token_ids(window, 1, window.len())?
                    .flatten_all()?
                    .to_dtype(DType::U32)?
                    .to_vec1::<u32>()?
            };
            let token = *next
                .last()
                .ok_or_else(|| invalid("the writer returned no token"))?;
            if token == hs::SHEET_BOS_ID {
                break;
            }
            ids.push(token);
            reply.push(token);
        }
        Ok(reply)
    })
}

fn append_line(path: &Path, line: &str) -> Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{line}")?;
    Ok(())
}

/// The last `chars` characters of `text`, on one line.
fn tail(text: &str, chars: usize) -> String {
    let flat: String = text
        .chars()
        .map(|c| if c == '\n' { ' ' } else { c })
        .collect();
    let n = flat.chars().count();
    flat.chars().skip(n.saturating_sub(chars)).collect()
}

/// The nearest and farthest pair of the writer's 16 address roots, as the mean L1 distance over
/// a port's 8 coordinates, the unit the address score uses. A query port sitting exactly on its
/// root beats the next cell sharing the other port by at most the nearest separation.
pub fn root_separation() -> Result<(f64, f64)> {
    let roots = crate::spiralcore_fbs::ProjectionPlan::new(8, 8)?
        .geometry
        .roots;
    let (mut nearest, mut farthest) = (f64::INFINITY, 0.0f64);
    for (i, a) in roots.iter().enumerate() {
        for b in &roots[i + 1..] {
            let l1: i32 = a
                .iter()
                .zip(b)
                .map(|(x, y)| (i32::from(*x) - i32::from(*y)).abs())
                .sum();
            let mean = f64::from(l1) / a.len() as f64;
            nearest = nearest.min(mean);
            farthest = farthest.max(mean);
        }
    }
    Ok((nearest, farthest))
}

/// The pinned DEV panel's windows and its response mask.
fn load_dev(data_root: &Path) -> Result<(Vec<CausalWindow>, Vec<u8>)> {
    let (dev_tokens, dev_mask) = hs::load_pinned_uort_split(
        &data_root.join("ft/dev/tokens.u16"),
        &data_root.join("ft/dev/response_mask.u8"),
        hs::PINNED_DEV_TOKENS_SHA256,
        hs::PINNED_DEV_MASK_SHA256,
        hs::EXPECTED_DEV_TOKENS,
        hs::EXPECTED_DEV_ELIGIBLE_TARGETS,
        hs::SHEET_VOCAB,
        hs::SHEET_BOS_ID,
    )?;
    let (dev_windows, _) =
        hs::build_document_windows(&dev_tokens, &dev_mask, SHEET_CONTEXT, hs::SHEET_BOS_ID)?;
    Ok((dev_windows, dev_mask))
}

/// How far back a checkpoint reads from where a reply opens ([`context_reach`]): the opening
/// point's mean absolute shift per dimension when the token `distance` places before it is
/// replaced by BOS, over each DEV window's first opening with that many tokens before it.
#[derive(Clone, Debug, Serialize)]
pub struct Reach {
    pub distance: usize,
    pub rows: usize,
    pub mean_shift: f64,
}

pub const REACH_DISTANCES: [usize; 13] = [0, 1, 2, 3, 4, 5, 6, 8, 10, 12, 16, 24, 32];

/// One recurrence layer's selected rate-table entries over the first 16 DEV windows that open a
/// reply, at their openings and at every position. Entry `k` read as retention holds about
/// `2^(k+1)` tokens; read as the default rate, it holds at most a few.
#[derive(Clone, Debug, Serialize)]
pub struct RateBins {
    pub layer: usize,
    pub openings: [u64; 16],
    pub all_positions: [u64; 16],
}

/// [`Reach`] at each of [`REACH_DISTANCES`] for the checkpoint saved in `checkpoint`, and each
/// layer's [`RateBins`]. Distance 0 replaces the opening's own input token, the end of the prompt.
pub fn context_reach(
    data_root: &Path,
    checkpoint: &Path,
    retain_rates: bool,
    fold_scores: bool,
    reflect_floor: bool,
    seed: u64,
    device: &Device,
) -> Result<(Vec<Reach>, Vec<RateBins>)> {
    let (dev_windows, dev_mask) = load_dev(data_root)?;
    arms::set_fbs_arm(Some(FbsArm::OneAtomActive));
    crate::geometric_stack::set_retention_rates(retain_rates);
    crate::geometric_stack::set_folded_scores(fold_scores);
    crate::geometric_stack::set_reflected_floor(reflect_floor);
    let _guard = ArmGuard;
    let model = StackModel::load(checkpoint, device)?;
    load_reads(checkpoint, device)?;
    let firsts: Vec<(&CausalWindow, usize)> = dev_windows
        .iter()
        .filter_map(|w| {
            w.scored_offsets
                .iter()
                .zip(&w.scored_corpus_positions)
                .find(|&(_, &p)| opens_reply(&dev_mask, p))
                .map(|(&off, _)| (w, off))
        })
        .collect();
    hs::with_gate_mode(HamiltonArm::HamiltonActive, seed, 0, false, || {
        let points = |ids: &[u32], offs: &[usize]| -> Result<Tensor> {
            let hidden = crate::flock_read::points(&model, ids, offs.len(), SHEET_CONTEXT)?;
            let idx: Vec<u32> = offs
                .iter()
                .enumerate()
                .map(|(b, &off)| (b * SHEET_CONTEXT + off) as u32)
                .collect();
            Ok(hidden
                .index_select(&Tensor::from_vec(idx, offs.len(), device)?, 0)?
                .detach())
        };
        let mut reach = Vec::with_capacity(REACH_DISTANCES.len());
        for &distance in &REACH_DISTANCES {
            let eligible: Vec<&(&CausalWindow, usize)> = firsts
                .iter()
                // A row whose replaced token is already BOS would add a zero shift.
                .filter(|(w, off)| {
                    *off >= distance && w.input_ids[*off - distance] != hs::SHEET_BOS_ID
                })
                .collect();
            let (mut total, mut rows) = (0.0f64, 0usize);
            for chunk in eligible.chunks(hs::SHEET_EVAL_BATCH) {
                let offs: Vec<usize> = chunk.iter().map(|(_, off)| *off).collect();
                let mut ids: Vec<u32> = chunk
                    .iter()
                    .flat_map(|(w, _)| w.input_ids.iter().copied())
                    .collect();
                let base = points(&ids, &offs)?;
                for (b, off) in offs.iter().enumerate() {
                    ids[b * SHEET_CONTEXT + off - distance] = hs::SHEET_BOS_ID;
                }
                let moved = points(&ids, &offs)?;
                total += f64::from(
                    moved
                        .sub(&base)?
                        .abs()?
                        .mean(1)?
                        .sum_all()?
                        .to_scalar::<f32>()?,
                );
                rows += chunk.len();
            }
            reach.push(Reach {
                distance,
                rows,
                mean_shift: total / rows.max(1) as f64,
            });
        }
        let probe = &firsts[..firsts.len().min(hs::SHEET_EVAL_BATCH)];
        let ids: Vec<u32> = probe
            .iter()
            .flat_map(|(w, _)| w.input_ids.iter().copied())
            .collect();
        let (_, captured) = crate::secant_descent::with_rate_indices(|| {
            crate::flock_read::points(&model, &ids, probe.len(), SHEET_CONTEXT)
        })?;
        let lanes = table(&model)?.dim(1)? / 4;
        let per_layer = probe.len() * SHEET_CONTEXT * lanes;
        if per_layer == 0 || captured.is_empty() || captured.len() % per_layer != 0 {
            return Err(invalid("rate capture does not split into whole layers"));
        }
        let bins = captured
            .chunks(per_layer)
            .enumerate()
            .map(|(layer, values)| {
                let mut bins = RateBins {
                    layer,
                    openings: [0; 16],
                    all_positions: [0; 16],
                };
                for (at, &k) in values.iter().enumerate() {
                    let position = at / lanes;
                    let (b, t) = (position / SHEET_CONTEXT, position % SHEET_CONTEXT);
                    // BOS padding after a window's last scored target is not text.
                    if probe[b]
                        .0
                        .scored_offsets
                        .last()
                        .is_some_and(|&last| t <= last)
                    {
                        bins.all_positions[usize::from(k)] += 1;
                    }
                    if t == probe[b].1 {
                        bins.openings[usize::from(k)] += 1;
                    }
                }
                bins
            })
            .collect();
        Ok((reach, bins))
    })
}

/// Greedy decoding speed at one context length ([`decode_speed`]).
#[derive(Clone, Debug, Serialize)]
pub struct DecodeSpeed {
    pub context_tokens: usize,
    pub decoded_tokens: usize,
    pub seconds: f64,
    pub tokens_per_second: f64,
}

/// Context lengths for [`decode_speed`].
pub const DECODE_CONTEXTS: [usize; 3] = [16, 64, SHEET_CONTEXT];

/// Greedy decoding speed of the checkpoint in `checkpoint` on one RTX-class GPU: `tokens` tokens
/// after a prompt of each of [`DECODE_CONTEXTS`] tokens taken from the first context-long DEV
/// window. Each token re-runs the trunk over the whole window (no carried state), so this times
/// this runner's decoding, not a served recurrent decode. Nearest decoding under `nearest`, else
/// the writer's.
pub fn decode_speed(
    data_root: &Path,
    checkpoint: &Path,
    retain_rates: bool,
    fold_scores: bool,
    reflect_floor: bool,
    nearest: bool,
    tokens: usize,
    seed: u64,
    device: &Device,
) -> Result<Vec<DecodeSpeed>> {
    let (dev_windows, _) = load_dev(data_root)?;
    arms::set_fbs_arm(Some(FbsArm::OneAtomActive));
    crate::geometric_stack::set_retention_rates(retain_rates);
    crate::geometric_stack::set_folded_scores(fold_scores);
    crate::geometric_stack::set_reflected_floor(reflect_floor);
    let _guard = ArmGuard;
    let model = StackModel::load(checkpoint, device)?;
    load_reads(checkpoint, device)?;
    let source = dev_windows
        .iter()
        .find(|w| w.input_ids.iter().all(|&id| id != hs::SHEET_BOS_ID))
        .unwrap_or(&dev_windows[0]);
    let objective = if nearest {
        PointObjective::Nearest
    } else {
        PointObjective::Sheet
    };
    // One untimed reply first, so no context is charged for kernel loading.
    greedy_reply(&model, objective, &source.input_ids[..16], 4, seed)?;
    let mut speeds = Vec::with_capacity(DECODE_CONTEXTS.len());
    for &context in &DECODE_CONTEXTS {
        let prompt = &source.input_ids[..context.min(source.input_ids.len())];
        let started = Instant::now();
        // BOS stops a reply early; count what was decoded.
        let reply = greedy_reply(&model, objective, prompt, tokens, seed)?;
        device.synchronize()?;
        let seconds = started.elapsed().as_secs_f64();
        // A reply cut short by BOS ran one more forward pass, the one that produced BOS.
        let passes = (reply.len() + 1).min(tokens);
        speeds.push(DecodeSpeed {
            context_tokens: prompt.len(),
            decoded_tokens: passes,
            seconds,
            tokens_per_second: passes as f64 / seconds.max(1e-9),
        });
    }
    Ok(speeds)
}

/// Read every checkpoint a `run_long` run saved under `run/models`, in step order, as an
/// [`Inspection`]. `retain_rates`, `fold_scores` and `reflect_floor` must match the run's.
pub fn inspect_run(
    data_root: &Path,
    run: &Path,
    retain_rates: bool,
    fold_scores: bool,
    reflect_floor: bool,
    seed: u64,
    device: &Device,
) -> Result<Vec<Inspection>> {
    let (dev_windows, dev_mask) = load_dev(data_root)?;
    arms::set_fbs_arm(Some(FbsArm::OneAtomActive));
    crate::geometric_stack::set_retention_rates(retain_rates);
    crate::geometric_stack::set_folded_scores(fold_scores);
    crate::geometric_stack::set_reflected_floor(reflect_floor);
    let _guard = ArmGuard;
    let mut dirs: Vec<PathBuf> = fs::read_dir(run.join("models"))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.join("model.safetensors").exists())
        .collect();
    dirs.sort();
    let probe: Vec<&CausalWindow> = dev_windows.iter().take(hs::SHEET_EVAL_BATCH).collect();
    let (ids, scored, _) = assemble(&probe);
    let mut readings = Vec::with_capacity(dirs.len());
    for dir in dirs {
        let name = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let step: usize = name.trim_start_matches("step-").parse().unwrap_or(0);
        let model = StackModel::load(&dir, device)?;
        load_reads(&dir, device)?;
        let (dev, _) = hs::evaluate_dev_panel(
            &model,
            &dev_windows,
            HamiltonArm::HamiltonActive,
            seed,
            step,
            false,
            false,
        )?;
        let query = hs::with_gate_mode(HamiltonArm::HamiltonActive, seed, step, false, || {
            let hidden = crate::flock_read::points(&model, &ids, probe.len(), SHEET_CONTEXT)?;
            let idx = Tensor::from_vec(scored.clone(), scored.len(), device)?;
            Ok(hidden.index_select(&idx, 0)?)
        })?;
        let (nearest, _) = nearest_reading(&model, &dev_windows, &dev_mask, seed, step)?;
        let first_cells = model
            .native_head_candidates(&query)?
            .cell_ids()
            .narrow(1, 0, 1)?
            .flatten_all()?
            .to_vec1::<u32>()?;
        let centred = query.broadcast_sub(&query.mean_keepdim(0)?)?;
        let scalar = |t: Tensor| -> Result<f64> { Ok(f64::from(t.to_scalar::<f32>()?)) };
        readings.push(Inspection {
            checkpoint: name,
            scored_rows: dev.scored_rows,
            top1_hits: dev.top1_hits,
            nearest,
            admitted_hits: dev.admitted_target_hits,
            token_unviolated_hits: dev.token_unviolated_hits,
            token_margin: dev.token_margin,
            address_margin: dev.address_margin,
            probe_rows: first_cells.len(),
            distinct_first_cells: first_cells
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            query_spread: scalar(centred.sqr()?.mean(0)?.sqrt()?.mean_all()?)?,
            query_mean_abs: scalar(query.abs()?.mean_all()?)?,
            table_mean_abs: scalar(
                model.variables()["embedding.weight"]
                    .as_tensor()
                    .abs()?
                    .mean_all()?,
            )?,
            flock_weight_mean_abs: match crate::flock_read::flock_read() {
                Some(read) => Some(scalar(read.weight_mean_abs()?)?),
                None => None,
            },
            content_weight_mean_abs: match crate::flock_read_content::content_read() {
                Some(content) => Some(scalar(content.read.weight_mean_abs()?)?),
                None => None,
            },
            hamming_weight_mean_abs: match crate::flock_read_hamming::hamming_reads() {
                Some(reads) => Some(scalar(reads.weight_mean_abs()?)?),
                None => None,
            },
            opening_weight_mean_abs: match crate::flock_read_opening::opening_read() {
                Some(read) => Some(scalar(read.weight_mean_abs()?)?),
                None => None,
            },
        });
    }
    Ok(readings)
}

/// Key, value and filler token ranges for the recall task: disjoint, and clear of BOS (0).
pub const RECALL_KEYS: std::ops::Range<u32> = 16..528;
pub const RECALL_VALUES: std::ops::Range<u32> = 528..1_040;
pub const RECALL_FILLER: std::ops::Range<u32> = 1_040..4_096;
/// The recall task's query distance buckets, `d = q − p`, each drawn uniformly inside.
pub const RECALL_BUCKETS: [(usize, usize); 4] = [(2, 8), (9, 32), (33, 96), (97, 180)];
/// Held-out recall windows scored at each checkpoint.
pub const RECALL_EVAL_WINDOWS: u64 = 128;
/// The held-out windows' seed offset. A held-out window equals a training window only if their
/// generator states collide, which for these seeds needs a training index far past any run.
const RECALL_EVAL_SEED: u64 = 0x5EED_F00D;

/// One window of the multi-query associative recall task, after
/// `uor-r4-training/examples/mqar-bench.rs`, and each scored row's distance bucket.
///
/// The window is `context` tokens, BOS first. Each pair writes its key at `p` and its value at
/// `p + 1`. Its query writes the key again at `q = p + d`, with `d` drawn inside the pair's
/// bucket, and the only scored target, at offset `q`, is the value. Keys, values and filler come
/// from disjoint ranges. Keys and values are distinct within a window, and pairings are fresh
/// for every `(seed, index)`, so a value can only be recalled from the context.
///
/// The longest bucket is placed first, while the window is empty, so its distances stay uniform.
/// A window that cannot hold every pair is an error, never a thinner bucket.
pub fn recall_window(
    seed: u64,
    index: u64,
    context: usize,
    pairs_per_bucket: usize,
) -> Result<(CausalWindow, Vec<usize>)> {
    let mut state = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(index.wrapping_mul(0xD1B5_4A32_D192_ED03))
        | 1;
    let mut next = move || {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        state.wrapping_mul(0x2545_F491_4F6C_DD1D)
    };
    let draw = |range: &std::ops::Range<u32>, x: u64| {
        range.start + (x % u64::from(range.end - range.start)) as u32
    };
    let mut ids: Vec<u32> = (0..context).map(|_| draw(&RECALL_FILLER, next())).collect();
    ids[0] = hs::SHEET_BOS_ID;
    let mut taken = vec![false; context];
    taken[0] = true;
    let (mut keys, mut values) = (Vec::new(), Vec::new());
    let mut rows: Vec<(usize, u32, usize)> = Vec::new();
    for (bucket, &(low, high)) in RECALL_BUCKETS.iter().enumerate().rev() {
        let (mut placed, mut tries) = (0, 0);
        while placed < pairs_per_bucket && tries < 10_000 {
            tries += 1;
            let d = low + (next() % (high - low + 1) as u64) as usize;
            if d + 3 >= context {
                continue;
            }
            let p = 1 + (next() % (context - d - 2) as u64) as usize;
            let q = p + d;
            if [p, p + 1, q, q + 1]
                .iter()
                .any(|&i| i >= context || taken[i])
            {
                continue;
            }
            let (key, value) = (draw(&RECALL_KEYS, next()), draw(&RECALL_VALUES, next()));
            if keys.contains(&key) || values.contains(&value) {
                continue;
            }
            for i in [p, p + 1, q, q + 1] {
                taken[i] = true;
            }
            (ids[p], ids[p + 1], ids[q], ids[q + 1]) = (key, value, key, value);
            keys.push(key);
            values.push(value);
            rows.push((q, value, bucket));
            placed += 1;
        }
        if placed < pairs_per_bucket {
            return Err(invalid(format!(
                "a {context}-token recall window holds only {placed} of {pairs_per_bucket} pairs in bucket {low}-{high}"
            )));
        }
    }
    rows.sort_unstable();
    let base = index as usize * context;
    let window = CausalWindow {
        doc_id: index as usize,
        input_ids: ids,
        scored_offsets: rows.iter().map(|r| r.0).collect(),
        scored_targets: rows.iter().map(|r| r.1).collect(),
        scored_corpus_positions: rows.iter().map(|r| base + r.0 + 1).collect(),
    };
    Ok((window, rows.iter().map(|r| r.2).collect()))
}

/// One recall checkpoint's reading, one line of `recall.jsonl` ([`run_recall`]).
#[derive(Clone, Debug, Serialize)]
pub struct RecallReading {
    pub step: usize,
    pub train_loss_mean: Option<f64>,
    /// Held-out queries per distance bucket ([`RECALL_BUCKETS`]), and how many of them the whole
    /// table's nearest token to the predicted point names right.
    pub rows: [usize; 4],
    pub hits: [usize; 4],
    pub train_seconds: f64,
    pub train_context_tokens_per_second: f64,
    /// Each read's weight, mean of its absolute values, when the run has that read.
    pub content_weight_mean_abs: Option<f64>,
    pub hamming_weight_mean_abs: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opening_weight_mean_abs: Option<f64>,
    /// The Hamming reads' margins in bits, read by read and head by head: a margin below every
    /// distance weighs nothing and gets no gradient back.
    pub hamming_margins: Option<Vec<f64>>,
}

/// Recall hits per bucket on the held-out `windows`, decoded as the whole table's nearest token.
fn recall_hits(
    model: &StackModel,
    windows: &[(CausalWindow, Vec<usize>)],
    seed: u64,
) -> Result<([usize; 4], [usize; 4])> {
    let table = table(model)?;
    let (mut rows, mut hits) = ([0usize; 4], [0usize; 4]);
    hs::with_gate_mode(HamiltonArm::HamiltonActive, seed, 0, false, || {
        for chunk in windows.chunks(16) {
            let refs: Vec<&CausalWindow> = chunk.iter().map(|(w, _)| w).collect();
            let (ids, scored, targets) = assemble(&refs);
            let hidden = crate::flock_read::points(model, &ids, refs.len(), SHEET_CONTEXT)?;
            let idx = Tensor::from_vec(scored, targets.len(), model.device())?;
            let picks = nearest_scores(&hidden.index_select(&idx, 0)?, &table)?
                .argmax(1)?
                .to_vec1::<u32>()?;
            let buckets = chunk.iter().flat_map(|(_, b)| b.iter().copied());
            for ((pick, target), bucket) in picks.iter().zip(&targets).zip(buckets) {
                rows[bucket] += 1;
                hits[bucket] += usize::from(pick == target);
            }
        }
        Ok(())
    })?;
    Ok((rows, hits))
}

/// Train the trunk on the recall task, fresh windows every step, with `config`'s reads and
/// optimizer settings as [`run_long`] uses them, scoring held-out recall per distance bucket
/// every `eval_every` steps into `out/recall.jsonl`.
pub fn run_recall(
    out: &Path,
    config: &LongConfig,
    pairs_per_bucket: usize,
    device: &Device,
) -> Result<()> {
    if config.steps == 0 || config.batch == 0 || config.eval_every == 0 || pairs_per_bucket == 0 {
        return Err(invalid(
            "steps, batch, eval-every and pairs must be positive",
        ));
    }
    if out.exists() {
        return Err(invalid(format!(
            "{} exists; a run never writes over one",
            out.display()
        )));
    }
    fs::create_dir_all(out)?;
    fs::write(out.join("config.json"), serde_json::to_vec_pretty(config)?)?;
    // The task itself, for the record: `config.json` holds only the runner's settings.
    let task =
        serde_json::json!({ "pairs_per_bucket": pairs_per_bucket, "context": SHEET_CONTEXT });
    fs::write(out.join("recall.json"), serde_json::to_vec_pretty(&task)?)?;
    arms::set_fbs_arm(Some(FbsArm::OneAtomActive));
    crate::geometric_stack::set_retention_rates(config.retain_rates);
    crate::geometric_stack::set_folded_scores(config.fold_scores);
    crate::geometric_stack::set_reflected_floor(config.reflect_floor);
    crate::geometric_stack::set_input_table_fixed(config.fix_input_table);
    let _guard = ArmGuard;
    let model = build_fbs_model_at(FbsArm::OneAtomActive, config.seed, config.width, device)?;
    let mut flock = if config.flock_read > 0 {
        let read = crate::flock_read::FlockRead::new(config.flock_read, config.width, device)?;
        crate::flock_read::set_flock_read(Some(read.clone()));
        let opt = crate::flock_read::FlockAdamW::new(&read)?;
        Some((read, opt))
    } else {
        None
    };
    let mut content = if config.content_read > 0 {
        let read =
            crate::flock_read_content::ContentRead::new(config.content_read, config.width, device)?;
        crate::flock_read_content::set_content_read(Some(read.clone()));
        let opt = crate::flock_read::FlockAdamW::new(&read.read)?;
        Some((read, opt))
    } else {
        None
    };
    // The Hamming reads, when on: their own optimizer, applied inside the trunk.
    let mut hamming = start_hamming(config, device)?;
    // The opening read (claude-154), when on: its own optimizer, applied inside the trunk.
    let mut opening = start_opening(config, device)?;
    let mut opt_main = StackAdamW::new(&model, hs::SHEET_WEIGHT_DECAY, hs::SHEET_CLIP)?;
    let mut opt_decay = StackAdamW::new(&model, 0.0, hs::SHEET_CLIP)?;
    let held_out: Vec<(CausalWindow, Vec<usize>)> = (0..RECALL_EVAL_WINDOWS)
        .map(|i| {
            recall_window(
                config.seed ^ RECALL_EVAL_SEED,
                i,
                SHEET_CONTEXT,
                pairs_per_bucket,
            )
        })
        .collect::<Result<_>>()?;
    let log = out.join("recall.jsonl");
    let (mut train_seconds, mut loss_sum, mut loss_steps) = (0.0f64, 0.0f64, 0usize);
    let reading = |step: usize, loss: Option<f64>, seconds: f64| -> Result<()> {
        let (rows, hits) = recall_hits(&model, &held_out, config.seed)?;
        let context_tokens = (step * config.batch * SHEET_CONTEXT) as f64;
        let scalar = |t: Tensor| -> Result<f64> { Ok(f64::from(t.to_scalar::<f32>()?)) };
        let line = RecallReading {
            step,
            train_loss_mean: loss,
            rows,
            hits,
            train_seconds: seconds,
            train_context_tokens_per_second: if seconds > 0.0 {
                context_tokens / seconds
            } else {
                0.0
            },
            content_weight_mean_abs: match crate::flock_read_content::content_read() {
                Some(read) => Some(scalar(read.read.weight_mean_abs()?)?),
                None => None,
            },
            hamming_weight_mean_abs: match crate::flock_read_hamming::hamming_reads() {
                Some(reads) => Some(scalar(reads.weight_mean_abs()?)?),
                None => None,
            },
            opening_weight_mean_abs: match crate::flock_read_opening::opening_read() {
                Some(read) => Some(scalar(read.weight_mean_abs()?)?),
                None => None,
            },
            hamming_margins: match crate::flock_read_hamming::hamming_reads() {
                Some(reads) => Some(reads.margins()?),
                None => None,
            },
        };
        println!(
            "[recall] step {step} hits {:?} of {:?} train {seconds:.0}s",
            line.hits, line.rows
        );
        append_line(&log, &serde_json::to_string(&line)?)
    };
    reading(0, None, 0.0)?;
    for step in 0..config.steps {
        let windows: Vec<CausalWindow> = (0..config.batch)
            .map(|b| {
                recall_window(
                    config.seed,
                    (step * config.batch + b) as u64,
                    SHEET_CONTEXT,
                    pairs_per_bucket,
                )
                .map(|(window, _)| window)
            })
            .collect::<Result<_>>()?;
        let refs: Vec<&CausalWindow> = windows.iter().collect();
        let (ids, scored, targets) = assemble(&refs);
        let started = Instant::now();
        let (value, grads) = hs::with_gate_mode(
            HamiltonArm::HamiltonActive,
            config.seed,
            step + 1,
            false,
            || {
                let loss = batch_loss(
                    &model,
                    config.objective,
                    &ids,
                    &scored,
                    &targets,
                    config.batch,
                )?;
                let value = f64::from(loss.to_scalar::<f32>()?);
                if !value.is_finite() {
                    return Err(invalid(format!("non-finite recall loss at step {step}")));
                }
                Ok((value, loss.backward()?))
            },
        )?;
        let progress = if config.steps == 1 {
            1.0
        } else {
            step as f64 / (config.steps - 1) as f64
        };
        let lr = config.lr + (config.lr_end - config.lr) * progress;
        opt_main.update_filtered(&model, &grads, lr, |name| !name.ends_with("rec.decay"))?;
        if let Some((read, opt)) = flock.as_mut() {
            opt.step(read, &grads, lr)?;
        }
        if let Some((read, opt)) = content.as_mut() {
            opt.step(&read.read, &grads, lr)?;
        }
        if let Some((reads, opt)) = hamming.as_mut() {
            opt.step_vars(&reads.variables(), &grads, lr)?;
        }
        if let Some((read, opt)) = opening.as_mut() {
            opt.step_vars(&read.variables(), &grads, lr)?;
        }
        opt_decay.update_filtered(&model, &grads, lr * config.decay_mult, |name| {
            name.ends_with("rec.decay")
        })?;
        device.synchronize()?;
        train_seconds += started.elapsed().as_secs_f64();
        loss_sum += value;
        loss_steps += 1;
        if (step + 1) % config.eval_every == 0 || step + 1 == config.steps {
            reading(step + 1, Some(loss_sum / loss_steps as f64), train_seconds)?;
            (loss_sum, loss_steps) = (0.0, 0);
        }
    }
    // The final model and its reads, for the record and for any later reading.
    let directory = out.join(format!("models/step-{:06}", config.steps));
    model.save(&directory)?;
    if let Some(read) = crate::flock_read::flock_read() {
        read.save(&directory)?;
    }
    if let Some(read) = crate::flock_read_content::content_read() {
        read.save(&directory)?;
    }
    if let Some(reads) = crate::flock_read_hamming::hamming_reads() {
        reads.save(&directory)?;
    }
    if let Some(read) = crate::flock_read_opening::opening_read() {
        read.save(&directory)?;
    }
    Ok(())
}

/// The pinned tokenizer.
fn pinned_tokenizer(data_root: &Path) -> Result<ByteBpeTokenizer> {
    let tokenizer_bytes = fs::read(data_root.join("tokenizer.json"))?;
    if sha256_bytes(&tokenizer_bytes) != hs::PINNED_TOKENIZER_SHA256 {
        return Err(invalid(
            "tokenizer.json differs from the pinned d36d3e87 tokenizer",
        ));
    }
    ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or_else(|| invalid("tokenizer.json is not a byte-level BPE tokenizer"))
}

/// The pinned TRAIN split's tokens and response mask.
fn load_train_split(data_root: &Path) -> Result<(Vec<u32>, Vec<u8>)> {
    hs::load_pinned_uort_split(
        &data_root.join("mw-balp/train/tokens.u16"),
        &data_root.join("mw-balp/train/response_mask.u8"),
        hs::PINNED_TRAIN_TOKENS_SHA256,
        hs::PINNED_TRAIN_MASK_SHA256,
        hs::EXPECTED_TRAIN_TOKENS,
        hs::EXPECTED_TRAIN_ELIGIBLE_TARGETS,
        hs::SHEET_VOCAB,
        hs::SHEET_BOS_ID,
    )
}

/// `split`'s tokens and response mask, refused unless both files hash to the sha256s it names,
/// the tokens carry a `UORT` header whose count matches both payloads, and every token is a
/// vocabulary id after an opening BOS (claude-152).
fn load_named_split(split: &TrainSplit) -> Result<(Vec<u32>, Vec<u8>)> {
    let token_bytes = fs::read(split.dir.join("tokens.u16"))?;
    let mask = fs::read(split.dir.join("response_mask.u8"))?;
    for (name, bytes, want) in [
        ("tokens.u16", &token_bytes, &split.tokens_sha256),
        ("response_mask.u8", &mask, &split.mask_sha256),
    ] {
        let got = sha256_bytes(bytes);
        if &got != want {
            return Err(invalid(format!(
                "{name} in {} hashes to {got}, not {want}",
                split.dir.display()
            )));
        }
    }
    const HEADER: usize = 64;
    if token_bytes.len() < HEADER || &token_bytes[..4] != b"UORT" {
        return Err(invalid("tokens.u16 lacks its 64-byte UORT header"));
    }
    let count = u64::from_le_bytes(
        token_bytes[8..16]
            .try_into()
            .map_err(|_| invalid("a short UORT header"))?,
    ) as usize;
    if token_bytes.len() != HEADER + 2 * count || mask.len() != count {
        return Err(invalid(format!(
            "the UORT header says {count} tokens; the payload or the mask disagrees"
        )));
    }
    let tokens: Vec<u32> = token_bytes[HEADER..]
        .chunks_exact(2)
        .map(|w| u32::from(u16::from_le_bytes([w[0], w[1]])))
        .collect();
    if tokens.first() != Some(&hs::SHEET_BOS_ID)
        || tokens.iter().any(|&t| t as usize >= hs::SHEET_VOCAB)
    {
        return Err(invalid(
            "the split does not open with BOS, or has an id past the vocabulary",
        ));
    }
    Ok((tokens, mask))
}

/// TRAIN's windows with at least 4 scored targets, each context-long; at least `batch` of them.
/// TRAIN is the pinned split, or `split` when a run names one.
fn load_train(
    data_root: &Path,
    batch: usize,
    split: Option<&TrainSplit>,
) -> Result<Vec<CausalWindow>> {
    let (train_tokens, train_mask) = match split {
        None => load_train_split(data_root)?,
        Some(split) => load_named_split(split)?,
    };
    let (train_windows, _) =
        hs::build_document_windows(&train_tokens, &train_mask, SHEET_CONTEXT, hs::SHEET_BOS_ID)?;
    let usable: Vec<CausalWindow> = train_windows
        .into_iter()
        .filter(|w| w.scored_offsets.len() >= 4)
        .collect();
    if usable.len() < batch || usable.iter().any(|w| w.input_ids.len() != SHEET_CONTEXT) {
        return Err(invalid("TRAIN windows are too few or not context-long"));
    }
    Ok(usable)
}

/// One parameter's gradient norm on the first training batch ([`gradient_norms`]).
#[derive(Clone, Debug, Serialize)]
pub struct GradientNorm {
    pub name: String,
    pub norm: f64,
}

/// The L2 norm of every parameter's gradient on the first `config.batch` TRAIN windows of a
/// freshly built model under `config`'s objective and switches, largest first, after one line
/// named `total` and one with the norm the main optimizer clips. No update is made.
pub fn gradient_norms(
    data_root: &Path,
    config: &LongConfig,
    device: &Device,
) -> Result<Vec<GradientNorm>> {
    let usable = load_train(data_root, config.batch, config.train_split.as_ref())?;
    arms::set_fbs_arm(Some(FbsArm::OneAtomActive));
    crate::geometric_stack::set_retention_rates(config.retain_rates);
    crate::geometric_stack::set_folded_scores(config.fold_scores);
    crate::geometric_stack::set_reflected_floor(config.reflect_floor);
    crate::geometric_stack::set_input_table_fixed(config.fix_input_table);
    let _guard = ArmGuard;
    let model = build_fbs_model_at(FbsArm::OneAtomActive, config.seed, config.width, device)?;
    if config.flock_read > 0 {
        crate::flock_read::set_flock_read(Some(crate::flock_read::FlockRead::new(
            config.flock_read,
            config.width,
            device,
        )?));
    }
    if config.content_read > 0 {
        crate::flock_read_content::set_content_read(Some(
            crate::flock_read_content::ContentRead::new(config.content_read, config.width, device)?,
        ));
    }
    let _hamming = start_hamming(config, device)?;
    let _opening = start_opening(config, device)?;
    let windows: Vec<&CausalWindow> = usable.iter().take(config.batch).collect();
    let (ids, scored, targets) = assemble(&windows);
    let grads = hs::with_gate_mode(HamiltonArm::HamiltonActive, config.seed, 0, false, || {
        Ok(batch_loss(
            &model,
            config.objective,
            &ids,
            &scored,
            &targets,
            config.batch,
        )?
        .backward()?)
    })?;
    let mut norms = Vec::new();
    let (mut total, mut main) = (0.0f64, 0.0f64);
    for (name, var) in model.variables() {
        if let Some(grad) = grads.get(var.as_tensor()) {
            let squared = f64::from(grad.sqr()?.sum_all()?.to_scalar::<f32>()?);
            total += squared;
            if !name.ends_with("rec.decay") {
                main += squared;
            }
            norms.push(GradientNorm {
                name: name.clone(),
                norm: squared.sqrt(),
            });
        }
    }
    // The shared table's gradient split by role: the output side alone, from the loss on the
    // predicted points with the trunk cut off, and the input side, the rest.
    let table_var = model
        .variables()
        .get("embedding.weight")
        .ok_or_else(|| invalid("the model has no shared vector table"))?;
    let output_side =
        hs::with_gate_mode(HamiltonArm::HamiltonActive, config.seed, 0, false, || {
            let hidden = crate::flock_read::points(&model, &ids, config.batch, SHEET_CONTEXT)?;
            let idx = Tensor::from_vec(scored.clone(), scored.len(), device)?;
            let query = hidden.index_select(&idx, 0)?.detach();
            Ok(query_loss(&model, config.objective, &query, &targets)?.backward()?)
        })?;
    if let (Some(full), Some(output)) = (
        grads.get(table_var.as_tensor()),
        output_side.get(table_var.as_tensor()),
    ) {
        let norm = |t: &Tensor| -> Result<f64> {
            Ok(f64::from(t.sqr()?.sum_all()?.to_scalar::<f32>()?).sqrt())
        };
        norms.push(GradientNorm {
            name: "embedding.weight, output side".to_owned(),
            norm: norm(output)?,
        });
        norms.push(GradientNorm {
            name: "embedding.weight, input side".to_owned(),
            norm: norm(&full.sub(output)?)?,
        });
    }
    norms.sort_by(|a, b| b.norm.total_cmp(&a.norm));
    norms.insert(
        0,
        GradientNorm {
            name: "total".to_owned(),
            norm: total.sqrt(),
        },
    );
    // The main optimizer clips everything but `rec.decay` together (`run_long`).
    norms.insert(
        1,
        GradientNorm {
            name: "total_without_rec.decay".to_owned(),
            norm: main.sqrt(),
        },
    );
    Ok(norms)
}

/// Train, score, write text and save, as the module documentation says. `out` must not exist.
pub fn run_long(data_root: &Path, out: &Path, config: &LongConfig, device: &Device) -> Result<()> {
    if config.steps == 0 || config.batch == 0 || config.eval_every == 0 {
        return Err(invalid("steps, batch and eval-every must be positive"));
    }
    if out.exists() {
        return Err(invalid(format!(
            "{} exists; a run never writes over one",
            out.display()
        )));
    }
    fs::create_dir_all(out)?;
    fs::write(out.join("config.json"), serde_json::to_vec_pretty(config)?)?;

    let tokenizer = pinned_tokenizer(data_root)?;

    let usable = load_train(data_root, config.batch, config.train_split.as_ref())?;
    let (dev_windows, dev_mask) = load_dev(data_root)?;
    let prompts = sample_windows(&dev_windows, &dev_mask, config.samples);
    println!(
        "[in-space] usable TRAIN windows {} (batch {}, {} steps = {:.2} epochs), DEV windows {}",
        usable.len(),
        config.batch,
        config.steps,
        (config.steps * config.batch) as f64 / usable.len() as f64,
        dev_windows.len()
    );

    arms::set_fbs_arm(Some(FbsArm::OneAtomActive));
    crate::geometric_stack::set_retention_rates(config.retain_rates);
    crate::geometric_stack::set_folded_scores(config.fold_scores);
    crate::geometric_stack::set_reflected_floor(config.reflect_floor);
    crate::geometric_stack::set_input_table_fixed(config.fix_input_table);
    let _guard = ArmGuard;
    let model = build_fbs_model_at(FbsArm::OneAtomActive, config.seed, config.width, device)?;
    // The Flock read, when on: its own parameters and optimizer, applied by `points`.
    let flock = if config.flock_read > 0 {
        let read = crate::flock_read::FlockRead::new(config.flock_read, config.width, device)?;
        crate::flock_read::set_flock_read(Some(read.clone()));
        Some((read.clone(), crate::flock_read::FlockAdamW::new(&read)?))
    } else {
        None
    };
    let mut flock = flock;
    // The content read, when on: claude-144's parameters and optimizer, applied inside the trunk.
    let mut content = if config.content_read > 0 {
        let read =
            crate::flock_read_content::ContentRead::new(config.content_read, config.width, device)?;
        crate::flock_read_content::set_content_read(Some(read.clone()));
        let opt = crate::flock_read::FlockAdamW::new(&read.read)?;
        Some((read, opt))
    } else {
        None
    };
    // The Hamming reads, when on: their own optimizer, applied inside the trunk.
    let mut hamming = start_hamming(config, device)?;
    // The opening read (claude-154), when on: its own optimizer, applied inside the trunk.
    let mut opening = start_opening(config, device)?;
    let mut opt_main = StackAdamW::new(&model, hs::SHEET_WEIGHT_DECAY, hs::SHEET_CLIP)?;
    let mut opt_decay = StackAdamW::new(&model, 0.0, hs::SHEET_CLIP)?;

    let progress = out.join("progress.jsonl");
    let samples_md = out.join("samples.md");
    let save_root: PathBuf = out.join("models");
    let mut train_seconds = 0.0f64;
    let (mut loss_sum, mut loss_steps) = (0.0f64, 0usize);
    let checkpoint = |model: &StackModel,
                      step: usize,
                      train_loss_mean: Option<f64>,
                      train_seconds: f64|
     -> Result<()> {
        let (dev, _) = hs::evaluate_dev_panel(
            model,
            &dev_windows,
            HamiltonArm::HamiltonActive,
            config.seed,
            step,
            false,
            false,
        )?;
        let dev_loss = panel_loss(model, &dev_windows, config.seed, step, false)?;
        let (nearest, written) =
            nearest_reading(model, &dev_windows, &dev_mask, config.seed, step)?;
        let point = Checkpoint {
            step,
            epoch: (step * config.batch) as f64 / usable.len() as f64,
            train_loss_mean,
            dev_top1_hits: dev.top1_hits,
            dev_top8_hits: dev.top8_hits,
            dev_admitted_hits: dev.admitted_target_hits,
            dev_nearest: nearest,
            dev_writer: written,
            dev_scored_rows: dev.scored_rows,
            dev_loss,
            dev_token_margin: dev.token_margin,
            dev_address_margin: dev.address_margin,
            train_seconds,
            train_context_tokens_per_second: (step * config.batch * SHEET_CONTEXT) as f64
                / train_seconds.max(1e-9),
        };
        append_line(&progress, &serde_json::to_string(&point)?)?;
        println!(
            "[in-space] step {} epoch {:.2} train_loss {} dev top-1 {}/{} nearest {} (openings {}/{}, commonest {}) top-8 {} admitted {} dev_loss {:.5} (token {:.5} address {:.5}) train {:.0}s",
            step,
            point.epoch,
            train_loss_mean.map_or("-".to_owned(), |v| format!("{v:.5}")),
            dev.top1_hits,
            dev.scored_rows,
            nearest.hits,
            nearest.opening_hits,
            nearest.opening_rows,
            nearest.commonest_opening_rows,
            dev.top8_hits,
            dev.admitted_target_hits,
            dev_loss,
            dev.token_margin,
            dev.address_margin,
            train_seconds
        );
        append_line(
            &samples_md,
            &format!(
                "\n## Step {step} (epoch {:.2}): DEV top-1 {}/{}\n",
                point.epoch, dev.top1_hits, dev.scored_rows
            ),
        )?;
        for window in &prompts {
            let off = window.scored_offsets[0];
            let prompt = &window.input_ids[..=off];
            let reply = greedy_reply(
                model,
                config.objective,
                prompt,
                config.reply_tokens,
                config.seed,
            )?;
            // The true reply: the scored targets while their offsets stay contiguous from the
            // prompt's end, so a mask gap or the window's end never splices text together.
            let reference: Vec<u32> = window
                .scored_offsets
                .iter()
                .zip(&window.scored_targets)
                .enumerate()
                .take_while(|&(i, (&o, _))| o == off + i)
                .take(config.reply_tokens)
                .map(|(_, (_, &t))| t)
                .collect();
            let sample = Sample {
                step,
                prompt: tokenizer.decode(prompt),
                model: {
                    // A reply ends at its first end-of-turn marker; what follows is not shown.
                    let text = tokenizer.decode(&reply);
                    match text.find("<|eos|>") {
                        Some(at) => text[..at + "<|eos|>".len()].to_owned(),
                        None => text,
                    }
                },
                reference: tokenizer.decode(&reference),
            };
            append_line(&out.join("samples.jsonl"), &serde_json::to_string(&sample)?)?;
            append_line(
                &samples_md,
                &format!(
                    "- **prompt:** …{}\n  - **model:** {}\n  - **true:** {}",
                    tail(&sample.prompt, 240),
                    tail(&sample.model, 400),
                    tail(&sample.reference, 400)
                ),
            )?;
        }
        let directory = save_root.join(format!("step-{step:06}"));
        model.save(&directory)?;
        if let Some(read) = crate::flock_read::flock_read() {
            read.save(&directory)?;
        }
        if let Some(read) = crate::flock_read_content::content_read() {
            read.save(&directory)?;
        }
        if let Some(reads) = crate::flock_read_hamming::hamming_reads() {
            reads.save(&directory)?;
        }
        if let Some(read) = crate::flock_read_opening::opening_read() {
            read.save(&directory)?;
        }
        Ok(())
    };

    checkpoint(&model, 0, None, 0.0)?;
    let mut order = epoch_order(usable.len(), config.seed, 0);
    let mut cursor = 0usize;
    let mut epoch = 0u64;
    for step in 0..config.steps {
        if cursor + config.batch > order.len() {
            epoch += 1;
            order = epoch_order(usable.len(), config.seed, epoch);
            cursor = 0;
        }
        let windows: Vec<&CausalWindow> = order[cursor..cursor + config.batch]
            .iter()
            .map(|&i| &usable[i])
            .collect();
        cursor += config.batch;
        let (ids, scored, targets) = assemble(&windows);
        let started = Instant::now();
        // The gate hook carries the real step: at step 0 its backward runs diagnostics with host
        // syncs, and a checkpoint's replies leave it at 0.
        let (value, grads) = hs::with_gate_mode(
            HamiltonArm::HamiltonActive,
            config.seed,
            step + 1,
            false,
            || {
                let loss = batch_loss(
                    &model,
                    config.objective,
                    &ids,
                    &scored,
                    &targets,
                    config.batch,
                )?;
                let value = f64::from(loss.to_scalar::<f32>()?);
                if !value.is_finite() {
                    return Err(invalid(format!("non-finite training loss at step {step}")));
                }
                Ok((value, loss.backward()?))
            },
        )?;
        // Linear from `lr` at the first step to `lr_end` at the last.
        let progress = if config.steps == 1 {
            1.0
        } else {
            step as f64 / (config.steps - 1) as f64
        };
        let lr = config.lr + (config.lr_end - config.lr) * progress;
        opt_main.update_filtered(&model, &grads, lr, |name| !name.ends_with("rec.decay"))?;
        if let Some((read, opt)) = flock.as_mut() {
            opt.step(read, &grads, lr)?;
        }
        if let Some((content, opt)) = content.as_mut() {
            opt.step(&content.read, &grads, lr)?;
        }
        if let Some((reads, opt)) = hamming.as_mut() {
            opt.step_vars(&reads.variables(), &grads, lr)?;
        }
        if let Some((read, opt)) = opening.as_mut() {
            opt.step_vars(&read.variables(), &grads, lr)?;
        }
        opt_decay.update_filtered(&model, &grads, lr * config.decay_mult, |name| {
            name.ends_with("rec.decay")
        })?;
        device.synchronize()?;
        train_seconds += started.elapsed().as_secs_f64();
        loss_sum += value;
        loss_steps += 1;
        if (step + 1) % config.eval_every == 0 || step + 1 == config.steps {
            checkpoint(
                &model,
                step + 1,
                Some(loss_sum / loss_steps as f64),
                train_seconds,
            )?;
            (loss_sum, loss_steps) = (0.0, 0);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "cuda")]
    use crate::spiralcore_fbs_sheet::build_fbs_model;

    #[test]
    fn each_epoch_order_is_a_seeded_permutation() {
        let a = epoch_order(1_000, 1, 0);
        let mut sorted = a.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..1_000).collect::<Vec<_>>());
        assert_eq!(a, epoch_order(1_000, 1, 0));
        assert_ne!(a, epoch_order(1_000, 1, 1));
        assert_ne!(a, epoch_order(1_000, 2, 0));
    }

    #[test]
    fn root_separation_reads_the_sixteen_roots() {
        let (nearest, farthest) = root_separation().unwrap();
        println!("address roots: nearest {nearest}, farthest {farthest}");
        assert!(nearest > 0.0 && nearest <= farthest);
    }

    #[test]
    fn a_reply_opens_where_the_token_before_is_not_a_response_token() {
        // prompt, prompt, reply, reply, prompt, reply: the scored targets are the reply tokens
        // 2, 3 and 5, and the first of each reply opens it.
        let mask = [0u8, 0, 1, 1, 0, 1];
        let opens: Vec<bool> = [2, 3, 5].iter().map(|&p| opens_reply(&mask, p)).collect();
        assert_eq!(opens, [true, false, true]);
    }

    #[test]
    fn a_copied_row_repeats_a_pair_the_prompt_holds_and_not_one_the_reply_made() {
        // BOS, a prompt "u q r a", then the reply "k z k z q r". The reply's own "k z" repeats
        // inside the reply, but only "q r" stands in the prompt.
        let tokens = [0u32, 5, 9, 10, 6, 7, 8, 7, 8, 9, 10];
        let mask = [0u8, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1];
        let (windows, _) = hs::build_document_windows(&tokens, &mask, 16, 0).unwrap();
        let copied: Vec<usize> = windows
            .iter()
            .flat_map(|w| {
                w.scored_offsets
                    .iter()
                    .zip(&w.scored_corpus_positions)
                    .zip(&w.scored_targets)
                    .filter(|((&o, &p), &t)| copies_prompt(w, o, p, t, &mask))
                    .map(|((_, &p), _)| p)
            })
            .collect();
        assert_eq!(copied, [10], "only the reply's \"q r\" copies the prompt");
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_cell_objective_pushes_only_the_nearest_rival() {
        let device = Device::new_cuda(0).unwrap();
        // Scores are negative distances: the first rival is the nearest wrong token.
        let rival = Tensor::new(&[[-1.5f32, -1.8, -4.0]], &device).unwrap();
        let loss = |target: f32, cell: bool| -> f32 {
            let target = Tensor::new(&[[target]], &device).unwrap();
            rival_hinge(&rival, &target, cell)
                .unwrap()
                .to_scalar::<f32>()
                .unwrap()
        };
        // The nearest rival's hinge alone: 1 + (−1.5) − (−1.0).
        assert!((loss(-1.0, true) - 0.5).abs() < 1e-6);
        // The nearest objective averages all three hinges instead: (0.5 + 0.2 + 0) / 3.
        assert!((loss(-1.0, false) - 0.7 / 3.0).abs() < 1e-6);
        // A margin nearer than the nearest rival: inside the target's cell, no loss.
        assert_eq!(loss(-0.4, true), 0.0);
    }

    #[test]
    fn a_named_split_loads_only_under_the_sha256s_it_names() {
        let dir = std::env::temp_dir().join(format!("claude-152-split-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let mut header = vec![0u8; 64];
        header[..4].copy_from_slice(b"UORT");
        header[8..16].copy_from_slice(&3u64.to_le_bytes());
        let tokens: Vec<u8> = header.into_iter().chain([0u8, 0, 5, 0, 6, 0]).collect();
        let mask = vec![0u8, 0, 1];
        fs::write(dir.join("tokens.u16"), &tokens).unwrap();
        fs::write(dir.join("response_mask.u8"), &mask).unwrap();
        let split = |tokens_sha256: String| TrainSplit {
            dir: dir.clone(),
            tokens_sha256,
            mask_sha256: sha256_bytes(&mask),
        };
        let (ids, m) = load_named_split(&split(sha256_bytes(&tokens))).unwrap();
        assert_eq!((ids, m), (vec![0, 5, 6], mask.clone()));
        assert!(load_named_split(&split("0".repeat(64))).is_err());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_recall_query_names_the_value_after_its_keys_only_earlier_copy() {
        for index in 0..64 {
            let (w, buckets) = recall_window(7, index, SHEET_CONTEXT, 4).unwrap();
            assert_eq!(w.input_ids.len(), SHEET_CONTEXT);
            assert_eq!(w.input_ids[0], hs::SHEET_BOS_ID);
            assert_eq!(w.scored_offsets.len(), 16, "window {index}");
            let rows = w.scored_offsets.iter().zip(&w.scored_targets).zip(&buckets);
            for ((&q, &value), &bucket) in rows {
                let key = w.input_ids[q];
                assert!(RECALL_KEYS.contains(&key) && RECALL_VALUES.contains(&value));
                let earlier: Vec<usize> = (0..q).filter(|&i| w.input_ids[i] == key).collect();
                assert_eq!(earlier.len(), 1, "window {index}, query {q}");
                let p = earlier[0];
                assert_eq!((w.input_ids[p + 1], w.input_ids[q + 1]), (value, value));
                let (low, high) = RECALL_BUCKETS[bucket];
                assert!((low..=high).contains(&(q - p)), "d = {} in {bucket}", q - p);
            }
            // No two pairs share a value, so a value names one key only.
            let mut values = w.scored_targets.clone();
            values.sort_unstable();
            values.dedup();
            assert_eq!(values.len(), w.scored_targets.len(), "window {index}");
            let pair_positions = 4 * w.scored_offsets.len();
            let filler = w
                .input_ids
                .iter()
                .filter(|t| RECALL_FILLER.contains(t))
                .count();
            assert_eq!(filler + pair_positions + 1, SHEET_CONTEXT);
        }
    }

    #[test]
    fn a_window_too_small_for_its_pairs_is_refused_and_far_distances_stay_uniform() {
        // 12 pairs per bucket need 192 positions of a 192-token window that also holds BOS.
        assert!(recall_window(1, 0, SHEET_CONTEXT, 12).is_err());
        // The longest bucket is placed first, so even a crowded window (8 pairs per bucket, 128 of
        // 192 positions) fills it, with distances uniform over 97..=180. Placed last, it underfills.
        let mut far = Vec::new();
        for index in 0..256 {
            let (w, buckets) = recall_window(3, index, SHEET_CONTEXT, 8).unwrap();
            for (&q, &bucket) in w.scored_offsets.iter().zip(&buckets) {
                if bucket == 3 {
                    let key = w.input_ids[q];
                    let p = (0..q).find(|&i| w.input_ids[i] == key).unwrap();
                    far.push((q - p) as f64);
                }
            }
        }
        let mean = far.iter().sum::<f64>() / far.len() as f64;
        assert!((mean - 138.5).abs() < 6.0, "mean far distance {mean}");
    }

    #[test]
    fn recall_windows_are_seeded_and_fresh_per_index() {
        let ids = |seed, index| {
            recall_window(seed, index, SHEET_CONTEXT, 4)
                .unwrap()
                .0
                .input_ids
        };
        assert_eq!(ids(1, 3), ids(1, 3));
        assert_ne!(ids(1, 3), ids(1, 4));
        assert_ne!(ids(1, 3), ids(2, 3));
    }

    #[test]
    fn openings_carry_the_prompt_their_window_shows_back_to_the_last_reply() {
        // BOS, a prompt (10 11 12), a reply (20 21), a prompt (13 14), a reply (22).
        let tokens = [0u32, 10, 11, 12, 20, 21, 13, 14, 22];
        let mask = [0u8, 0, 0, 0, 1, 1, 0, 0, 1];
        let (windows, _) = hs::build_document_windows(&tokens, &mask, 16, 0).unwrap();
        assert_eq!(
            openings(&windows, &mask),
            [(vec![10, 11, 12], 20), (vec![13, 14], 22)]
        );
    }

    #[test]
    fn the_anchor_check_counts_openings_whose_turn_slots_are_the_first_8_key() {
        // BOS, a prompt (10 11 12), a reply (20 21) closed by EOS, a prompt (13 14), a reply (22)
        // with no EOS, then a prompt (15) and a reply (23). The turn anchor starts after BOS and
        // after EOS, so it keeps the probe's key at the first two openings; at the third it still
        // runs from the EOS, through the reply 22.
        let tokens = [0u32, 10, 11, 12, 20, 21, 1, 13, 14, 22, 15, 23];
        let mask = [0u8, 0, 0, 0, 1, 1, 1, 0, 0, 1, 0, 1];
        let (windows, _) = hs::build_document_windows(&tokens, &mask, 16, 0).unwrap();
        assert_eq!(
            openings(&windows, &mask),
            [(vec![10, 11, 12], 20), (vec![13, 14], 22), (vec![15], 23)]
        );
        let check = anchor_check("dev", &windows, &mask);
        assert_eq!((check.openings, check.agree), (3, 2));
        assert_eq!(
            serde_json::to_string(&check).unwrap(),
            r#"{"anchor_check":"dev","openings":3,"agree":2}"#
        );
    }

    fn config(opening_read: usize, opening_anchor: Option<OpeningAnchor>) -> LongConfig {
        LongConfig {
            objective: PointObjective::Sheet,
            steps: 1,
            batch: 1,
            eval_every: 1,
            seed: 1,
            lr: 0.002,
            lr_end: 0.002,
            decay_mult: 20.0,
            retain_rates: false,
            fold_scores: false,
            reflect_floor: false,
            flock_read: 0,
            content_read: 0,
            hamming_read: 0,
            opening_read,
            opening_anchor,
            width: hs::SHEET_WIDTH,
            type_spin: false,
            fix_input_table: false,
            train_split: None,
            samples: 6,
            reply_tokens: 48,
        }
    }

    use crate::flock_read_opening::OpeningAnchor;

    #[test]
    fn a_config_without_the_opening_read_writes_neither_of_its_keys() {
        let json = String::from_utf8(serde_json::to_vec_pretty(&config(0, None)).unwrap()).unwrap();
        assert!(!json.contains("opening_read"), "{json}");
        assert!(!json.contains("opening_anchor"), "{json}");
        // The keys around them stand as they did before the read.
        assert!(
            json.contains("\"hamming_read\": 0,\n  \"width\": 256,"),
            "{json}"
        );
    }

    #[test]
    fn a_config_with_the_opening_read_writes_both_of_its_keys() {
        for (anchor, name) in [(OpeningAnchor::Turn, "turn"), (OpeningAnchor::Last, "last")] {
            let json =
                String::from_utf8(serde_json::to_vec_pretty(&config(8, Some(anchor))).unwrap())
                    .unwrap();
            let keys = format!(
                "\"hamming_read\": 0,\n  \"opening_read\": 8,\n  \"opening_anchor\": \"{name}\",\n  \"width\": 256,"
            );
            assert!(json.contains(&keys), "{json}");
        }
    }

    #[test]
    fn a_checkpoint_with_an_opening_read_sets_it_on_the_thread_until_the_guard_drops() {
        // Save and load only: the vendored candle computes nothing on the CPU, but files move.
        let device = Device::Cpu;
        let directory =
            std::env::temp_dir().join(format!("claude-154-load-reads-{}", std::process::id()));
        crate::flock_read_opening::OpeningRead::new(5, 8, OpeningAnchor::Last, &device)
            .unwrap()
            .save(&directory)
            .unwrap();
        {
            let _guard = ArmGuard;
            load_reads(&directory, &device).unwrap();
            let read = crate::flock_read_opening::opening_read().expect("the opening read is set");
            assert_eq!((read.k(), read.anchor()), (5, OpeningAnchor::Last));
        }
        assert!(crate::flock_read_opening::opening_read().is_none());
        fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn the_bound_names_each_key_by_its_commonest_train_opening() {
        let train = [
            (vec![10, 11, 12], 20),
            (vec![13, 14], 22),
            (vec![10, 30], 20),
            (vec![10, 31], 23),
        ];
        let dev = [(vec![10, 99], 20), (vec![13, 98], 22), (vec![50], 20)];
        let decode = |ids: &[u32]| format!("{ids:?}");
        let first = bound(PromptKey::First(1), &train, &dev, &decode);
        // Key 10 opens with 20 twice and 23 once; key 13 with 22; key 50 is unseen and takes
        // TRAIN's commonest opening, 20.
        assert_eq!(
            (first.dev_hits, first.dev_unseen, first.train_keys),
            (3, 1, 2)
        );
        assert_eq!(first.top[0].key, "[10]");
        assert_eq!((first.top[0].train_rows, first.top[0].opening_rows), (3, 2));
        let none = bound(PromptKey::None, &train, &dev, &decode);
        assert_eq!((none.dev_hits, none.dev_unseen), (2, 0));
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn best_positions_skip_what_is_not_allowed_and_rank_the_rest() {
        let device = Device::new_cuda(0).unwrap();
        let scores = Tensor::new(&[[0.9f32, 0.1, 0.8, 0.3, 0.5]], &device).unwrap();
        let allowed = Tensor::new(&[[0u8, 1, 1, 1, 1]], &device).unwrap();
        let at = best_positions(&scores, &allowed, 3).unwrap();
        assert_eq!(at.to_vec2::<u32>().unwrap(), vec![vec![2, 4, 3]]);
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn whole_table_scores_do_not_depend_on_the_chunk() {
        let device = Device::new_cuda(0).unwrap();
        let table = Tensor::new(
            &[[0.0f32, 1.0], [1.0, 0.0], [2.0, 2.0], [-1.0, 0.5]],
            &device,
        )
        .unwrap();
        let query = Tensor::new(&[[0.0f32, 0.0], [1.0, 1.0], [2.0, -1.0]], &device).unwrap();
        let whole = nearest_scores(&query, &table)
            .unwrap()
            .to_vec2::<f32>()
            .unwrap();
        // Row by row, as a chunk of one row gives.
        for (r, row) in whole.iter().enumerate() {
            let one = nearest_scores(&query.narrow(0, r, 1).unwrap(), &table)
                .unwrap()
                .to_vec2::<f32>()
                .unwrap();
            assert_eq!(&one[0], row);
        }
        // Negative mean L1: query [0, 0] against [0, 1] is -(0 + 1) / 2.
        assert_eq!(whole[0][0], -0.5);
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_voronoi_loss_at_margin_one_is_the_writers_margin() {
        let device = Device::new_cuda(0).unwrap();
        let model = build_fbs_model(FbsArm::OneAtomActive, 1, &device).unwrap();
        let ids: Vec<u32> = (0..SHEET_CONTEXT as u32).map(|i| (i * 37) % 4096).collect();
        let hidden = model.hidden(&ids, 1, SHEET_CONTEXT).unwrap();
        let idx = Tensor::new(&[10u32, 50, 100, 191], &device).unwrap();
        let query = hidden.index_select(&idx, 0).unwrap();
        let targets = [11u32, 2000, 7, 300];
        let writer = model
            .native_head_loss(&query, &targets, hs::SHEET_RIVALS)
            .unwrap()
            .to_scalar::<f32>()
            .unwrap();
        let ours = voronoi_loss(&model, &query, &targets, 1.0)
            .unwrap()
            .to_scalar::<f32>()
            .unwrap();
        assert!((writer - ours).abs() < 1e-6, "{writer} != {ours}");
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn the_fold_reflects_into_the_unit_interval_and_keeps_its_gradient() {
        let device = Device::new_cuda(0).unwrap();
        let at = [-2.5f32, -0.3, 0.0, 0.4, 1.0, 1.2, 2.7, 3.0];
        let x = candle_core::Var::new(&at[..], &device).unwrap();
        let folded = crate::geometric_stack::fold_unit(x.as_tensor()).unwrap();
        let got = folded.to_vec1::<f32>().unwrap();
        let want = [0.5f32, 0.3, 0.0, 0.4, 1.0, 0.8, 0.7, 1.0];
        for (g, w) in got.iter().zip(&want) {
            assert!((g - w).abs() < 1e-6, "{got:?} != {want:?}");
        }
        let grads = folded.sum_all().unwrap().backward().unwrap();
        let slope = grads.get(x.as_tensor()).unwrap().to_vec1::<f32>().unwrap();
        // Away from the fold points the slope is +1 or -1: never zero, unlike a clamp's.
        for (i, &s) in slope.iter().enumerate() {
            if [1, 3, 5, 6].contains(&i) {
                assert_eq!(s.abs(), 1.0, "slope {slope:?}");
            }
        }
        assert_eq!(slope[1], -1.0);
        assert_eq!(slope[3], 1.0);
        assert_eq!(slope[5], -1.0);
    }

    #[test]
    fn tail_keeps_the_last_characters_on_one_line() {
        assert_eq!(tail("ab\ncdé", 3), "cdé");
        assert_eq!(tail("ab", 10), "ab");
    }
}
