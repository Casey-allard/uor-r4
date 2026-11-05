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

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::{DType, Device, Tensor};
use serde::{Deserialize, Serialize};
use uor_r4_integer::fbs_snapshot::{
    CodeRole, DecisionForce, DecisionForceWindow, DecisionHead, DecisionLayer, DecisionMemory,
    DecisionNearest, DecisionRead, DecisionStep, DecisionTrace, DecisionWindow, HammingTraceRead,
    HammingTraceWindow, DECISION_FORCE_SCHEMA, DECISION_TRACE_SCHEMA,
};
use uor_r4_tokenizer::ByteBpeTokenizer;

pub use crate::flock_read_hamming::ReadScope;
use crate::flock_read_memory::MemoryLookup;
use crate::geometric_stack::stage_clock::{self, StageTime};
use crate::geometric_stack::{RateScores, StackAdamW, StackModel};
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
    /// The prompt register (claude-155) after layer 2 and on the readout: this many slots over
    /// the current turn's span; 0 is no register (`flock_read_register`). Left out of
    /// `config.json` when 0.
    #[serde(skip_serializing_if = "is_zero")]
    pub prompt_register: usize,
    /// How many span positions each register slot weighs; set exactly when `prompt_register` is,
    /// and left out of `config.json` when none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub register_picks: Option<usize>,
    /// How the register's slots choose their positions; set exactly when `prompt_register` is,
    /// and left out of `config.json` when none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub register_select: Option<crate::flock_read_register::RegisterSelect>,
    /// The trunk's width: the shared table's columns and every read's width (default 256,
    /// `hamilton_sheet::SHEET_WIDTH`); a multiple of 8.
    pub width: usize,
    /// The Hamming read's type heads compare positions through SpiralCore's path-dependent
    /// icosian transport (`HammingReads::with_type_spin`, claude-151); off by default.
    pub type_spin: bool,
    /// How many tokens before a position the Hamming read's copy heads key on: 1, the default,
    /// or 2, the ordered two-token key (`HammingReads::with_copy_key`, claude-261). Left out of
    /// `config.json` at 1, so a one-token run writes the same file as before.
    #[serde(skip_serializing_if = "is_one")]
    pub hamming_copy_key: usize,
    /// Each copy head's own key, 1 or 2, one per copy head, in place of `hamming_copy_key`
    /// (`HammingReads::with_copy_keys`, claude-268). Left out of `config.json` when none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hamming_copy_keys: Option<Vec<usize>>,
    /// Pass no gradient from the input encoding into the shared table
    /// (`geometric_stack::set_input_table_fixed`); it changes training only, never a forward.
    pub fix_input_table: bool,
    /// Train on this split in place of the pinned TRAIN; DEV stays the pinned one.
    pub train_split: Option<TrainSplit>,
    pub samples: usize,
    pub reply_tokens: usize,
    /// Profile this many training steps by stage (fbs-cost-profile, claude-157): see
    /// [`profile_window`]. 0 is off, and left out of `config.json`.
    #[serde(skip_serializing_if = "is_zero")]
    pub profile: usize,
    /// The turn memory (route-plus-memory, claude-161): key each reply opening on this many ids
    /// of its turn; 0 is no memory, and left out of `config.json`.
    #[serde(skip_serializing_if = "is_zero")]
    pub turn_memory: usize,
    /// The turn memory with an empty table, the no-memory control (L-010); left out when false.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub turn_memory_empty: bool,
    /// The turn memory's reply table (turn-memory-reply, claude-193): the rows past each reply's
    /// opening keyed by `turn-8 + position + reply-last-1` at stored share at least
    /// [`REPLY_MEMORY_SHARE`]; needs `turn_memory`, and is emptied by `turn_memory_empty`. Left
    /// out when false.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub turn_memory_reply: bool,
    /// Memory dropout (one-model-end-to-end protocol I-B, claude-207), in thousandths: each
    /// training window is dropped with this chance ([`memory_dropped`]), and in a dropped window
    /// the turn memory names no row, so the trunk also learns those windows without it. Training
    /// only: scoring and serving use the memory at every row. 0 is off, and left out of
    /// `config.json`.
    #[serde(skip_serializing_if = "is_zero_u32")]
    pub turn_memory_dropout: u32,
    /// The SpiralCore/FBS transport arm (protocol I-C, claude-207): `OneAtomActive`, the trained
    /// transport, by default; `IdentityTransport` or `FrozenQueries` for the transport controls.
    /// Left out of `config.json` when the default.
    #[serde(skip_serializing_if = "is_one_atom")]
    pub fbs_arm: FbsArm,
    /// Train with the reply's Hamming read held to the prompt (claude-216): every scored row
    /// whose input is a reply token reads, for the heads the scope limits, only positions before
    /// its reply run's first token ([`reply_row_limits`]). Scoring such a checkpoint applies the
    /// same cut to its rows. Left out of `config.json` when none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_read_train: Option<ReadScope>,
    /// Train as served (one-model-end-to-end, claude-258 and deepseek-4): before each step's
    /// forward, the trunk's tables, the Hamming reads and the turn memory's weights are rounded
    /// as the integer export rounds them; the D11 gates/transition, Q16 residual and dynamic i16
    /// activations are on; and the scored query is Q12. The gradient moves the float values the
    /// rounded forward came from ([`ServedValues`]). Every checkpoint is scored in the same
    /// served forward and saved at the rounded values, so rounding a saved step as exported
    /// changes nothing. Left out of `config.json` when false.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub train_as_served: bool,
    /// The training window (claude-273): TRAIN is cut into windows of this many tokens, and the
    /// model is built to read this many positions ([`StackModel::set_context`]); 2 to 4,096.
    /// DEV's evaluation keeps [`SHEET_CONTEXT`]'s windows. Left out of `config.json` at
    /// [`SHEET_CONTEXT`], so a run at the default writes the same file as before.
    #[serde(skip_serializing_if = "is_sheet_context")]
    pub context: usize,
}

/// The float values one step rounded as served ([`LongConfig::train_as_served`], claude-258):
/// every variable of the trunk, the Hamming reads and the turn memory, each with a copy of its
/// value before the rounding. [`ServedValues::restore`] puts the copies back, so the optimizer
/// moves the float values, and an update finer than the export's step accumulates instead of
/// being rounded away.
struct ServedValues {
    float: Vec<(candle_core::Var, Tensor)>,
    restored: bool,
}

impl ServedValues {
    /// Copy each variable's value, then round them in place with the export's own roundings
    /// (`fbs_export::round_tables_as_exported`, `round_hamming_as_exported`,
    /// `round_memory_as_exported`). A rounding the export refuses leaves every value as it was.
    fn round(
        model: &StackModel,
        reads: Option<&crate::flock_read_hamming::HammingReads>,
        memory: Option<&crate::flock_read_memory::TurnMemory>,
    ) -> Result<Self> {
        let mut vars: Vec<&candle_core::Var> = model.variables().values().collect();
        if let Some(reads) = reads {
            vars.extend(reads.variables());
        }
        if let Some(memory) = memory {
            vars.extend(memory.variables());
        }
        let float = vars
            .into_iter()
            .map(|var| Ok((var.clone(), var.as_tensor().copy()?)))
            .collect::<Result<Vec<_>>>()?;
        let served = Self {
            float,
            restored: false,
        };
        let rounded = (|| -> Result<()> {
            crate::fbs_export::round_tables_as_exported(model)?;
            if let Some(reads) = reads {
                crate::fbs_export::round_hamming_as_exported(reads)?;
            }
            if let Some(memory) = memory {
                crate::fbs_export::round_memory_as_exported(memory)?;
            }
            Ok(())
        })();
        match rounded {
            Ok(()) => Ok(served),
            Err(refused) => {
                served.restore()?;
                Err(refused)
            }
        }
    }

    /// Put each variable's float value back.
    fn restore_values(&self) -> Result<()> {
        for (var, value) in &self.float {
            var.set(value)?;
        }
        Ok(())
    }

    /// Put each variable's float value back and mark the guard restored.
    fn restore(mut self) -> Result<()> {
        self.restore_values()?;
        self.restored = true;
        Ok(())
    }
}

impl Drop for ServedValues {
    fn drop(&mut self) {
        if !self.restored {
            // Every `?` between rounding and the explicit restore takes this path. A failed
            // restoration cannot be reported from `Drop`, but it is no worse than leaving the
            // rounded values in place; the explicit success/error path still returns any error.
            let _ = self.restore_values();
            self.restored = true;
        }
    }
}

/// [`ServedValues::round`] when the run trains as served; none, with nothing changed, when not.
fn round_if_served(
    on: bool,
    model: &StackModel,
    reads: Option<&crate::flock_read_hamming::HammingReads>,
    memory: Option<&crate::flock_read_memory::TurnMemory>,
) -> Result<Option<ServedValues>> {
    on.then(|| ServedValues::round(model, reads, memory))
        .transpose()
}

/// Run a training step under the graph it will export, or retain the existing plain path.
fn with_training_forward<T>(
    served: bool,
    seed: u64,
    step: usize,
    f: impl FnOnce() -> Result<T>,
) -> Result<T> {
    if served {
        hs::with_served_surrogate_training(seed, step + 1, f)
    } else {
        hs::with_gate_mode(HamiltonArm::HamiltonActive, seed, step + 1, false, f)
    }
}

fn with_eval_forward<T>(
    served: bool,
    seed: u64,
    step: usize,
    f: impl FnOnce() -> Result<T>,
) -> Result<T> {
    if served {
        hs::with_served_surrogate_training(seed, step, f)
    } else {
        hs::with_gate_mode(HamiltonArm::HamiltonActive, seed, step, false, f)
    }
}

fn is_zero_u32(n: &u32) -> bool {
    *n == 0
}

fn is_one_atom(arm: &FbsArm) -> bool {
    *arm == FbsArm::OneAtomActive
}

/// The most [`LongConfig::turn_memory_dropout`] can be: every window dropped.
pub const MEMORY_DROPOUT_MAX: u32 = 1_000;

/// Whether the turn memory is dropped from training window `b` of step `step` at `permille`
/// thousandths (protocol I-B): one xorshift64* draw from its own stream, seeded by the run's
/// seed and the window's place in the run, so the batch order is untouched. 0 drops none and
/// [`MEMORY_DROPOUT_MAX`] drops all.
pub fn memory_dropped(seed: u64, step: usize, b: usize, batch: usize, permille: u32) -> bool {
    if permille == 0 {
        return false;
    }
    if permille >= MEMORY_DROPOUT_MAX {
        return true;
    }
    let index = (step as u64).wrapping_mul(batch as u64).wrapping_add(b as u64);
    let mut state = seed
        .wrapping_mul(0xD6E8_FEB8_6659_FD93)
        .wrapping_add(index.wrapping_mul(0xA076_1D64_78BD_642F))
        | 1;
    state ^= state >> 12;
    state ^= state << 25;
    state ^= state >> 27;
    state.wrapping_mul(0x2545_F491_4F6C_DD1D) % u64::from(MEMORY_DROPOUT_MAX) < u64::from(permille)
}

/// Whether the main optimizer updates the variable `name` under `arm`: never the decay rates,
/// which have their own optimizer, and never the SpiralCore/FBS routing queries when the arm
/// keeps them frozen at their seed (`FbsArm::trains_queries`; the registered sheet's runner keeps
/// them out the same way).
pub fn main_optimizer_updates(name: &str, arm: FbsArm) -> bool {
    !name.ends_with("rec.decay") && (arm.trains_queries() || !name.ends_with(".spiralcore_fbs"))
}

/// The transport arm's command-line name, as `--fbs-arm` takes it.
pub fn fbs_arm_named(name: &str) -> Option<FbsArm> {
    match name {
        "one-atom" => Some(FbsArm::OneAtomActive),
        "identity" => Some(FbsArm::IdentityTransport),
        "frozen" => Some(FbsArm::FrozenQueries),
        _ => None,
    }
}

/// Refuse to score a checkpoint under another arm than its run trained with, when the run's
/// `config.json` is found beside it: at `checkpoint` itself (a run folder) or two levels up (a
/// `models/step-N` folder). A checkpoint copied without its run's config is scored as asked.
pub fn check_run_arm(checkpoint: &Path, arm: FbsArm) -> Result<()> {
    // Absolute, so a relative `step-N` still finds the run two levels up.
    let checkpoint = checkpoint
        .canonicalize()
        .unwrap_or_else(|_| checkpoint.to_path_buf());
    let candidates = [
        Some(checkpoint.join("config.json")),
        checkpoint.parent().and_then(Path::parent).map(|run| run.join("config.json")),
    ];
    for path in candidates.into_iter().flatten() {
        if !path.exists() {
            continue;
        }
        // A config that is there but cannot be read is refused, never skipped.
        let value: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)
            .map_err(|e| invalid(format!("{}: {e}", path.display())))?;
        // A run's config has `steps`; a model's own `config.json` does not.
        if value.get("steps").is_none() {
            continue;
        }
        let trained = match value.get("fbs_arm") {
            None => FbsArm::OneAtomActive,
            Some(v) => serde_json::from_value(v.clone())
                .map_err(|e| invalid(format!("{}: fbs_arm: {e}", path.display())))?,
        };
        if trained != arm {
            return Err(invalid(format!(
                "{} trained with {}, not {}; pass --fbs-arm to match",
                path.display(),
                trained.name(),
                arm.name()
            )));
        }
        return Ok(());
    }
    Ok(())
}

/// The stored share the reply table keeps its keys at: the `--reply-bound` probe's choice
/// (claude-192).
pub const REPLY_MEMORY_SHARE: f64 = 0.5;

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
    /// The same for the prompt register (claude-155), when the checkpoint has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub register_weight_mean_abs: Option<f64>,
    /// The last checkpoint's nearest-token DEV reading with each register slot dropped in turn,
    /// then all of them, when it has a register (claude-155).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub register_drop: Option<Vec<RegisterDrop>>,
    /// The turn memory's weight (claude-161), when the checkpoint has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_weight_mean_abs: Option<f64>,
    /// The turn memory's reply weight (claude-193), when its memory has a reply table.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_reply_weight_mean_abs: Option<f64>,
    /// The last checkpoint's nearest reading with the turn memory taken out (claude-161).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_drop: Option<RegisterDrop>,
}

/// The nearest-token DEV reading with one prompt-register slot, or every slot, dropped
/// ([`PromptRegister::without_slot`](crate::flock_read_register::PromptRegister::without_slot)).
#[derive(Clone, Debug, Serialize)]
pub struct RegisterDrop {
    /// The dropped slot's index, or `"all"`.
    pub slot: serde_json::Value,
    pub hits: usize,
    pub opening_hits: usize,
    pub copied_hits: usize,
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
        crate::flock_read_register::set_prompt_register(None);
        crate::flock_read_register::set_register_slots(None);
        crate::flock_read_memory::set_turn_memory(None);
        crate::flock_read_memory::set_memory_rows(None);
        crate::flock_read_memory::set_reply_rows(None);
        set_trained_cut(None);
        crate::flock_read_hamming::set_row_cut(None);
    }
}

/// The read scope `checkpoint`'s run trained its replies with ([`LongConfig::reply_read_train`],
/// claude-216), from the run's `config.json` where [`check_run_arm`] finds it; none when the run
/// trained without the cut, or no run config is found.
fn run_trained_cut(checkpoint: &Path) -> Result<Option<ReadScope>> {
    let checkpoint = checkpoint
        .canonicalize()
        .unwrap_or_else(|_| checkpoint.to_path_buf());
    let candidates = [
        Some(checkpoint.join("config.json")),
        checkpoint.parent().and_then(Path::parent).map(|run| run.join("config.json")),
    ];
    for path in candidates.into_iter().flatten() {
        if !path.exists() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)
            .map_err(|e| invalid(format!("{}: {e}", path.display())))?;
        if value.get("steps").is_none() {
            continue;
        }
        return match value.get("reply_read_train") {
            None => Ok(None),
            Some(v) => serde_json::from_value(v.clone())
                .map_err(|e| invalid(format!("{}: reply_read_train: {e}", path.display()))),
        };
    }
    Ok(None)
}

/// Whether the run saved beside `checkpoint` trained as served. A checkpoint copied without its
/// run config is scored through the plain path, as before.
fn run_train_as_served(checkpoint: &Path, served_forward: Option<bool>) -> Result<bool> {
    if let Some(served) = served_forward {
        return Ok(served);
    }
    let checkpoint = checkpoint
        .canonicalize()
        .unwrap_or_else(|_| checkpoint.to_path_buf());
    let candidates = [
        Some(checkpoint.join("config.json")),
        checkpoint
            .parent()
            .and_then(Path::parent)
            .map(|run| run.join("config.json")),
    ];
    for path in candidates.into_iter().flatten() {
        if !path.exists() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)
            .map_err(|e| invalid(format!("{}: {e}", path.display())))?;
        if value.get("steps").is_none() {
            continue;
        }
        return value
            .get("train_as_served")
            .map(|v| {
                v.as_bool()
                    .ok_or_else(|| invalid(format!("{}: train_as_served is not a bool", path.display())))
            })
            .unwrap_or(Ok(false));
    }
    Ok(false)
}

/// Set this thread's Flock read, content read, Hamming reads, opening read and prompt register
/// from the checkpoint `directory`, each to none when the checkpoint has none.
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
    crate::flock_read_register::set_prompt_register(
        crate::flock_read_register::PromptRegister::load(directory, device)?,
    );
    crate::flock_read_memory::set_turn_memory(crate::flock_read_memory::TurnMemory::load(
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
        if config.hamming_copy_key != 1 || config.hamming_copy_keys.is_some() {
            return Err(invalid("--hamming-copy-key and --hamming-copy-keys key the Hamming read's copy heads; turn the read on with --hamming-read"));
        }
        return Ok(None);
    }
    if let Some(keys) = &config.hamming_copy_keys {
        if config.hamming_copy_key != 1 {
            return Err(invalid("--hamming-copy-keys names every copy head's key; leave --hamming-copy-key out"));
        }
        // One key for every head is `--hamming-copy-key`, so `config.json` and the checkpoint,
        // which records a uniform key as `copy_key`, name it the same way.
        if keys.windows(2).all(|w| w[0] == w[1]) {
            return Err(invalid("--hamming-copy-keys gives the copy heads different keys; when every head takes the same key, use --hamming-copy-key"));
        }
    }
    let mut reads =
        crate::flock_read_hamming::HammingReads::new(config.hamming_read, config.width, device)?
            .with_copy_key(config.hamming_copy_key)?;
    if let Some(keys) = &config.hamming_copy_keys {
        reads = reads.with_copy_keys(keys)?;
    }
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

/// The prompt register `config` asks for (claude-155), set on this thread, with its own
/// optimizer; none when off.
fn start_register(
    config: &LongConfig,
    device: &Device,
) -> Result<
    Option<(
        crate::flock_read_register::PromptRegister,
        crate::flock_read::FlockAdamW,
    )>,
> {
    if config.prompt_register == 0 {
        return Ok(None);
    }
    let reg = crate::flock_read_register::PromptRegister::new(
        config.prompt_register,
        config.register_picks.unwrap_or(DEFAULT_REGISTER_PICKS),
        config.register_select.unwrap_or_default(),
        config.width,
        device,
    )?;
    crate::flock_read_register::set_prompt_register(Some(reg.clone()));
    let opt = crate::flock_read::FlockAdamW::for_vars(&reg.variables())?;
    Ok(Some((reg, opt)))
}

/// How many span positions a register slot weighs when the run names none (claude-155).
pub const DEFAULT_REGISTER_PICKS: usize = 8;

/// How many of the opening rows `rows` (each a window's index into `windows` and its opening
/// offsets) the memory's own key finds in its table: `(rows, hits)`.
fn memory_hits(
    memory: &crate::flock_read_memory::TurnMemory,
    windows: &[CausalWindow],
    rows: &[(usize, Vec<usize>)],
) -> Result<(usize, usize)> {
    let (mut total, mut hits) = (0, 0);
    for (index, offsets) in rows {
        if offsets.is_empty() {
            continue;
        }
        let ids = &windows[*index].input_ids;
        let (_, hit) = memory.stored(ids, ids.len(), offsets)?;
        total += offsets.len();
        hits += hit.iter().filter(|&&h| h > 0.0).count();
    }
    Ok((total, hits))
}

/// A turn memory as a run starts it, with its own optimizer.
type MemoryStart = (
    crate::flock_read_memory::TurnMemory,
    crate::flock_read::FlockAdamW,
);

/// The turn memory `config` asks for (claude-161), set on this thread, with its own optimizer,
/// and each of `usable`'s windows' opening offsets (the input offsets that predict a reply's
/// first token, by the split's response mask); none and no offsets when off. The table is built
/// from every TRAIN window, as the `--opening-bound` probe builds it, or left empty for the
/// no-memory control. With `turn_memory_reply` it also has the reply table (claude-193), built
/// from TRAIN's rows past their openings as the `--reply-bound` probe counted them, and returns
/// each usable window's rows past their openings with their keys.
fn start_memory(
    data_root: &Path,
    config: &LongConfig,
    usable: &[CausalWindow],
    device: &Device,
) -> Result<(Option<MemoryStart>, Vec<Vec<usize>>, ReplyOffsets)> {
    if config.turn_memory == 0 {
        if config.turn_memory_empty {
            return Err(invalid("--turn-memory-empty needs --turn-memory K"));
        }
        if config.turn_memory_reply {
            return Err(invalid("--turn-memory-reply needs --turn-memory K"));
        }
        if config.turn_memory_dropout > 0 {
            return Err(invalid("--turn-memory-dropout needs --turn-memory K"));
        }
        return Ok((None, Vec::new(), Vec::new()));
    }
    if config.turn_memory_reply && config.turn_memory != TURN_KEY {
        return Err(invalid(format!(
            "--turn-memory-reply keys on the turn's first {TURN_KEY} ids, as the probe did: it needs --turn-memory {TURN_KEY}"
        )));
    }
    let (tokens, mask) = match config.train_split.as_ref() {
        None => load_train_split(data_root)?,
        Some(split) => load_named_split(split)?,
    };
    let (windows, _) = hs::build_document_windows(&tokens, &mask, config.context, hs::SHEET_BOS_ID)?;
    let train = openings(&windows, &mask);
    let table = if config.turn_memory_empty {
        std::collections::HashMap::new()
    } else {
        crate::flock_read_memory::build_table(&train, config.turn_memory)
    };
    println!(
        "[memory] key {} ids; TRAIN openings {}; table keys {}{}",
        config.turn_memory,
        train.len(),
        table.len(),
        if config.turn_memory_empty {
            " (the empty control)"
        } else {
            ""
        }
    );
    let mut memory =
        crate::flock_read_memory::TurnMemory::new(config.turn_memory, table, config.width, device)?;
    // The reply table (claude-193): TRAIN's rows past their openings, at the probe's key and share.
    let mut replies = Vec::new();
    if config.turn_memory_reply {
        let rows: Vec<(Vec<u32>, u32)> = reply_rows(&windows, &mask)
            .into_iter()
            .filter(|row| !row.opening)
            .map(|row| (reply_key(&row), row.target))
            .collect();
        let table = if config.turn_memory_empty {
            std::collections::HashMap::new()
        } else {
            crate::flock_read_memory::build_reply_table(&rows, REPLY_MEMORY_SHARE)
        };
        println!(
            "[memory] reply table: TRAIN rows past their openings {}; keys at share {REPLY_MEMORY_SHARE} {}",
            rows.len(),
            table.len()
        );
        memory = memory.with_reply(table)?;
        replies = reply_offsets(usable, &mask);
    }
    crate::flock_read_memory::set_turn_memory(Some(memory.clone()));
    let opt = crate::flock_read::FlockAdamW::for_vars(&memory.variables())?;
    let offsets = usable
        .iter()
        .map(|w| {
            w.scored_offsets
                .iter()
                .zip(&w.scored_corpus_positions)
                .filter(|(_, &position)| opens_reply(&mask, position))
                .map(|(&offset, _)| offset)
                .collect()
        })
        .collect();
    Ok((Some((memory, opt)), offsets, replies))
}

fn is_zero(n: &usize) -> bool {
    *n == 0
}

fn is_one(n: &usize) -> bool {
    *n == 1
}

fn is_sheet_context(n: &usize) -> bool {
    *n == SHEET_CONTEXT
}

/// `context` as a training window ([`LongConfig::context`], claude-273): 2 to 4,096 tokens, the
/// stack's own range, or an error.
pub fn training_context(context: usize) -> Result<usize> {
    if (2..=4096).contains(&context) {
        Ok(context)
    } else {
        Err(invalid(format!("--context wants 2 to 4096 tokens, not {context}")))
    }
}

/// The trunk a run trains: [`build_fbs_model_at`] at `arm` and `config`'s seed and width, built
/// to read `config.context` positions (claude-273). The window sizes no weight, so the trunk's
/// weights are the ones it has at [`SHEET_CONTEXT`].
fn build_long_model(arm: FbsArm, config: &LongConfig, device: &Device) -> Result<StackModel> {
    let mut model = build_fbs_model_at(arm, config.seed, config.width, device)?;
    model.set_context(training_context(config.context)?)?;
    Ok(model)
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
    stage_clock::mark("objective.table_search");
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
    let loss = rival_hinge(&rival, &target_score, cell)?;
    stage_clock::mark("objective.hinge");
    Ok(loss)
}

thread_local! {
    /// The read scope a model trains and scores its rows with (claude-216's
    /// [`LongConfig::reply_read_train`]); none, the default, reads as before.
    static TRAINED_CUT: std::cell::Cell<Option<ReadScope>> = const { std::cell::Cell::new(None) };
    /// The next forward's inputs' response-mask flags ([`window_input_mask`]), which
    /// [`trained_points`] takes under a trained cut.
    static REPLY_MASK: std::cell::RefCell<Option<Vec<u8>>> = const { std::cell::RefCell::new(None) };
}

/// Train and score rows on this thread with the reply's read held to the prompt under `scope`
/// ([`trained_points`]); `None` reads as before. Any reply mask left unread is cleared.
pub fn set_trained_cut(scope: Option<ReadScope>) {
    TRAINED_CUT.with(|c| c.set(scope));
    set_reply_mask(None);
}

/// Give the next [`trained_points`] on this thread its inputs' response-mask flags.
fn set_reply_mask(mask: Option<Vec<u8>>) {
    REPLY_MASK.with(|m| *m.borrow_mut() = mask);
}

/// Whether this thread trains or scores under a cut, so each forward needs its inputs' mask.
fn trained_cut_on() -> bool {
    TRAINED_CUT.with(|c| c.get()).is_some()
}

/// Each input position's response-mask flag in `windows`, flat as [`assemble`] lays them out
/// (claude-216): `mask` at the position's corpus index, 0 for the BOS that opens a document or
/// pads a short window. A window's corpus start is its first scored target, less one, less
/// that row's offset.
fn window_input_mask(windows: &[&CausalWindow], mask: &[u8]) -> Result<Vec<u8>> {
    let mut flat = Vec::with_capacity(windows.iter().map(|w| w.input_ids.len()).sum());
    for window in windows {
        let start = match (window.scored_corpus_positions.first(), window.scored_offsets.first()) {
            (Some(&p), Some(&o)) if p > o => p - 1 - o,
            _ => return Err(invalid("a window with no scored row has no corpus start")),
        };
        for (offset, &id) in window.input_ids.iter().enumerate() {
            flat.push(if id == hs::SHEET_BOS_ID {
                0
            } else {
                *mask
                    .get(start + offset)
                    .ok_or_else(|| invalid("a window runs past its split's mask"))?
            });
        }
    }
    Ok(flat)
}

/// Each row's read limit for windows of `context` positions whose inputs carry the
/// response-mask flags `input_mask` ([`window_input_mask`]) (claude-216): a row whose input is a
/// reply token reads only before its reply's first token in the window, or nothing when the
/// reply began before the window; every other row keeps every earlier position (`context`). So
/// a reply's first scored row, whose input is the prompt's last token, keeps them, as at decode.
fn reply_row_limits(input_mask: &[u8], context: usize) -> Vec<usize> {
    let mut limits = vec![context; input_mask.len()];
    let mut first = 0;
    for (row, &flag) in input_mask.iter().enumerate() {
        let offset = row % context;
        if flag == 1 {
            if offset == 0 || input_mask[row - 1] == 0 {
                first = offset;
            }
            limits[row] = first;
        }
    }
    limits
}

/// Over `windows` of `context` positions and their split's `mask`: the rows, the rows the
/// trained cut limits, and the limited rows that read nothing in the limited heads, their reply
/// having begun before the window (claude-216).
fn cut_census(
    windows: &[CausalWindow],
    mask: &[u8],
    context: usize,
) -> Result<(usize, usize, usize)> {
    let refs: Vec<&CausalWindow> = windows.iter().collect();
    let limits = reply_row_limits(&window_input_mask(&refs, mask)?, context);
    let cut = limits.iter().filter(|&&l| l < context).count();
    let none = limits.iter().filter(|&&l| l == 0).count();
    Ok((limits.len(), cut, none))
}

/// The window length of a forward of `batch` windows laid out flat in `ids_len` ids
/// (claude-273): the windows are equal, so it is `ids_len / batch`, and a layout that does not
/// divide is refused.
fn window_time(ids_len: usize, batch: usize) -> Result<usize> {
    if batch == 0 || ids_len == 0 || !ids_len.is_multiple_of(batch) {
        return Err(invalid("a forward's ids are not batch windows of one length"));
    }
    Ok(ids_len / batch)
}

/// The trunk's points for `batch` windows of `ids`, with [`set_trained_cut`]'s scope applied to
/// every reply row ([`reply_row_limits`], on the mask [`set_reply_mask`] gave this forward,
/// which it takes), cleared before any error returns. Under a cut, a forward without its mask,
/// or with one of another length, is refused. The windows' length is `ids.len() / batch`
/// ([`window_time`]).
fn trained_points(model: &StackModel, ids: &[u32], batch: usize) -> Result<Tensor> {
    let mask = REPLY_MASK.with(|m| m.borrow_mut().take());
    let time = window_time(ids.len(), batch)?;
    if let Some(scope) = TRAINED_CUT.with(|c| c.get()) {
        let mask = mask.ok_or_else(|| invalid("a trained cut's forward has no reply mask"))?;
        if mask.len() != ids.len() {
            return Err(invalid("a trained cut's reply mask is not its forward's length"));
        }
        crate::flock_read_hamming::set_row_cut(Some((scope, reply_row_limits(&mask, time))));
    }
    let hidden = crate::flock_read::points(model, ids, batch, time);
    crate::flock_read_hamming::set_row_cut(None);
    hidden
}

fn batch_loss(
    model: &StackModel,
    objective: PointObjective,
    ids: &[u32],
    scored: &[u32],
    targets: &[u32],
    batch: usize,
    q12: bool,
) -> Result<Tensor> {
    let hidden = trained_points(model, ids, batch)?;
    let idx = Tensor::from_vec(scored.to_vec(), scored.len(), model.device())?;
    let query = hidden.index_select(&idx, 0)?;
    let query = if q12 {
        hs::q12_query(&query)?
    } else {
        query
    };
    stage_clock::mark("points.select");
    query_loss(model, objective, &query, targets)
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

/// One scored row as a checkpoint decodes it ([`row_picks`]): its corpus position and target,
/// the whole table's nearest token, the served writer's pick, and its kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RowPick {
    position: usize,
    target: u32,
    nearest: u32,
    writer: u32,
    opens: bool,
    copies: bool,
}

/// [`NearestReading`] over the DEV windows, and the writer's [`WriterReading`] of the same rows
/// from the same points; `mask` is DEV's response mask, by corpus position.
fn nearest_reading(
    model: &StackModel,
    dev: &[CausalWindow],
    mask: &[u8],
    seed: u64,
    step: usize,
    served: bool,
) -> Result<(NearestReading, WriterReading)> {
    let (mut all_queries, mut opening_queries) = (Vec::new(), Vec::new());
    let picks = row_picks(
        model,
        dev,
        mask,
        seed,
        step,
        served,
        None,
        |query, opens| {
            let at: Vec<u32> = (0..opens.len() as u32)
                .filter(|&i| opens[i as usize])
                .collect();
            if !at.is_empty() {
                let at = Tensor::from_vec(at.clone(), at.len(), model.device())?;
                opening_queries.push(query.index_select(&at, 0)?);
            }
            all_queries.push(query.clone());
            Ok(())
        },
    )?;
    let (mut reading, written) = fold_picks(&picks);
    if !opening_queries.is_empty() {
        reading.opening_query_spread = spread(&Tensor::cat(&opening_queries, 0)?)?;
    }
    if !all_queries.is_empty() {
        reading.query_spread = spread(&Tensor::cat(&all_queries, 0)?)?;
    }
    Ok((reading, written))
}

/// [`nearest_reading`]'s counts from its rows' picks (all but the two spreads).
fn fold_picks(picks: &[RowPick]) -> (NearestReading, WriterReading) {
    let mut reading = NearestReading::default();
    let mut written = WriterReading::default();
    let mut openings = std::collections::BTreeMap::<u32, usize>::new();
    let mut nearest_picks = std::collections::BTreeSet::<u32>::new();
    let mut written_picks = std::collections::BTreeSet::<u32>::new();
    for row in picks {
        let hit = row.nearest == row.target;
        reading.hits += usize::from(hit);
        written.hits += usize::from(row.writer == row.target);
        if row.copies {
            reading.copied_rows += 1;
            reading.copied_hits += usize::from(hit);
        } else if !row.opens {
            reading.rest_rows += 1;
            reading.rest_hits += usize::from(hit);
        }
        if row.opens {
            reading.opening_rows += 1;
            reading.opening_hits += usize::from(hit);
            written.opening_hits += usize::from(row.writer == row.target);
            *openings.entry(row.target).or_default() += 1;
            nearest_picks.insert(row.nearest);
            written_picks.insert(row.writer);
        }
    }
    reading.commonest_opening_rows = openings.values().copied().max().unwrap_or(0);
    reading.opening_distinct_picks = nearest_picks.len();
    written.opening_distinct_picks = written_picks.len();
    (reading, written)
}

/// How many windows of `window` positions one scoring forward holds (claude-274): at most the
/// evaluation's established positions, `SHEET_EVAL_BATCH` windows of [`SHEET_CONTEXT`]. A loaded
/// checkpoint's weights are variables, so a forward keeps its intermediates until its picks are
/// read, and its memory grows with its positions. So 16 windows at 192, and one at 2,048.
fn windows_per_forward(window: usize) -> usize {
    (hs::SHEET_EVAL_BATCH * SHEET_CONTEXT / window.max(1)).clamp(1, hs::SHEET_EVAL_BATCH)
}

/// Every scored row of `dev` decoded by `model`, in panel order: the nearest token and the
/// writer's pick from the same points, with the turn memory named at the rows training names
/// (its openings, and its reply table's later rows). `on_query` sees each batch's query rows
/// and which of them open a reply. A `trace` (claude-251) records the decisions of the windows it
/// names, in this same forward.
#[allow(clippy::too_many_arguments)]
fn row_picks(
    model: &StackModel,
    dev: &[CausalWindow],
    mask: &[u8],
    seed: u64,
    step: usize,
    served: bool,
    mut trace: Option<&mut FTrace>,
    mut on_query: impl FnMut(&Tensor, &[bool]) -> Result<()>,
) -> Result<Vec<RowPick>> {
    let table = table(model)?;
    let writer = model.native_vector_writer()?;
    let per_forward = dev.first().map_or(hs::SHEET_EVAL_BATCH, |w| windows_per_forward(w.input_ids.len()));
    let forward = || {
        let mut out = Vec::new();
        for chunk in dev.chunks(per_forward) {
            let windows: Vec<&CausalWindow> = chunk.iter().collect();
            let (ids, scored, targets) = assemble(&windows);
            if scored.is_empty() {
                continue;
            }
            let traced: Vec<usize> = match trace.as_deref() {
                Some(t) => (0..windows.len()).filter(|&b| t.wants(windows[b])).collect(),
                None => Vec::new(),
            };
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
            // The turn memory (claude-161) moves the opening rows only, as training names them.
            if crate::flock_read_memory::turn_memory().is_some() {
                crate::flock_read_memory::set_memory_rows(Some(
                    scored
                        .iter()
                        .zip(&opens)
                        .filter(|(_, &open)| open)
                        .map(|(&row, _)| row as usize)
                        .collect(),
                ));
                // Its reply table (claude-193) moves the later reply rows, as training names them, at
                // `assemble`'s row for each, at this forward's own window (claude-274).
                if crate::flock_read_memory::turn_memory().is_some_and(|m| m.reply_len().is_some())
                {
                    crate::flock_read_memory::set_reply_rows(Some(reply_table_rows(
                        chunk,
                        mask,
                        window_time(ids.len(), windows.len())?,
                    )));
                }
            }
            if trained_cut_on() {
                set_reply_mask(Some(window_input_mask(&windows, mask)?));
            }
            // The rows the memory reads in this forward, read back as it will read them.
            let named = (!traced.is_empty()).then(|| {
                (
                    crate::flock_read_memory::memory_rows(),
                    crate::flock_read_memory::reply_rows(),
                )
            });
            if named.is_some() {
                set_decision_capture(true);
            }
            // claude-253: the traced windows' forced rows, for this forward only.
            if let Some(sink) = trace.as_deref() {
                let (rates, codes) = sink.batch_force(&windows, &traced);
                crate::geometric_stack::set_rate_force(rates);
                crate::flock_read_hamming::set_code_force(codes);
            }
            let hidden = trained_points(model, &ids, windows.len());
            crate::geometric_stack::set_rate_force(Vec::new());
            crate::flock_read_hamming::set_code_force(Vec::new());
            crate::flock_read_memory::set_memory_rows(None);
            crate::flock_read_memory::set_reply_rows(None);
            let captured = named.is_some().then(|| {
                let captured = (
                    crate::flock_read_hamming::take_hamming_trace().unwrap_or_default(),
                    crate::geometric_stack::take_rate_trace().unwrap_or_default(),
                );
                set_decision_capture(false);
                captured
            });
            let hidden = hidden?;
            let idx = Tensor::from_vec(scored.clone(), scored.len(), model.device())?;
            let query = hidden.index_select(&idx, 0)?;
            let query = if served {
                hs::q12_query(&query)?.detach()
            } else {
                query.detach()
            };
            let scores = nearest_scores(&query, &table)?;
            let best = scores.argmax(1)?.to_vec1::<u32>()?;
            if let (Some(sink), Some((rows, reply)), Some((hamming, rates))) =
                (trace.as_deref_mut(), named, captured)
            {
                // The memory's lookups, where the forward ran it (`StackModel::hidden`'s rule).
                let lookups = match crate::flock_read_memory::turn_memory() {
                    Some(memory) if rows.is_some() || reply.is_some() => Some(memory.lookups(
                        &ids,
                        SHEET_CONTEXT,
                        &rows.unwrap_or_default(),
                        &reply.unwrap_or_default(),
                    )?),
                    _ => None,
                };
                let recorded = Recorded { hamming, rates, lookups };
                let at = traced_rows(&scored, &traced);
                let index = Tensor::from_vec(at.clone(), at.len(), query.device())?;
                let queries = query.index_select(&index, 0)?.to_vec2::<f32>()?;
                let row_scores = scores.index_select(&index, 0)?.to_vec2::<f32>()?;
                sink.hold_table(&table)?;
                sink.record(&windows, &traced, &ids, &scored, &best, &queries, &row_scores, recorded)?;
            }
            drop(scores);
            // The served decoder on the same points: the writer's pick among its kept cells.
            let served = writer
                .write(&query)?
                .flatten_all()?
                .to_dtype(DType::U32)?
                .to_vec1::<u32>()?;
            let positions: Vec<usize> = windows
                .iter()
                .flat_map(|w| w.scored_corpus_positions.iter().copied())
                .collect();
            if positions.len() != targets.len() || best.len() != targets.len() {
                return Err(invalid(format!(
                    "a batch scores {} rows but has {} positions and {} picks",
                    targets.len(),
                    positions.len(),
                    best.len()
                )));
            }
            let rows = positions
                .into_iter()
                .zip(&best)
                .zip(&served)
                .zip(&targets)
                .zip(&opens)
                .zip(&copies);
            for (((((position, &nearest), &served), &target), &open), &copy) in rows {
                out.push(RowPick {
                    position,
                    target,
                    nearest,
                    writer: served,
                    opens: open,
                    copies: copy,
                });
            }
            on_query(&query, &opens)?;
        }
        Ok(out)
    };
    with_eval_forward(served, seed, step, forward)
}

/// claude-251, Q5's decision trace (side F, `uor-r4.decision-trace/1`): one split's windows to
/// trace, each named by its `position` (its first scored position, as `split_prompts` names a
/// panel row), and what [`row_picks`] recorded for them in its own forward, in panel order. Row
/// `t` of a window is R's step `t`. Recording copies to the host and changes nothing computed.
/// On an evaluation copy rounded as exported, the same trace is side T (claude-252,
/// [`FTrace::on_side`]).
pub struct FTrace {
    positions: std::collections::BTreeSet<usize>,
    trace: DecisionTrace,
    /// The shared table on the host, row-major, for the nearest's scores in R's units.
    table: Option<Vec<f32>>,
    /// claude-253: each traced window's forced set by its position, applied in the forward that
    /// traces it ([`FTrace::with_force`]).
    force: BTreeMap<usize, DecisionForceWindow>,
}

/// What one traced forward recorded beside its points.
struct Recorded {
    hamming: Vec<HammingTraceWindow>,
    rates: Vec<RateScores>,
    /// Each `batch · time` row's turn-memory lookup; `None` where the forward ran no memory.
    lookups: Option<Vec<Option<MemoryLookup>>>,
}

impl FTrace {
    /// A trace of the windows `positions` name, each named once.
    pub fn new(positions: &[usize]) -> Result<Self> {
        let set: std::collections::BTreeSet<usize> = positions.iter().copied().collect();
        if set.len() != positions.len() {
            return Err(invalid("a decision trace names each position once"));
        }
        Ok(Self {
            positions: set,
            trace: DecisionTrace {
                schema: DECISION_TRACE_SCHEMA.to_owned(),
                side: "F".to_owned(),
                layers: 0,
                lanes: 0,
                heads: 0,
                kept: crate::flock_read_hamming::HAMMING_KEPT,
                windows: Vec::new(),
            },
            table: None,
            force: BTreeMap::new(),
        })
    }

    /// This trace under `force`, a `uor-r4.decision-force/1` file (claude-253, Q5's forced
    /// replay), checked against the model's shape (`layers` layers of `lanes` lanes, and each
    /// read's depth, heads and head width in `reads`). Each forced bucket replaces the entry its
    /// lane's score chooses, and each forced code the computed one, in the forward that traces
    /// its window; the trace records them as used. R's refusals (`set_decision_trace`): a file of
    /// another schema, a position named twice or not traced, a step past the context, a layer,
    /// lane or entry outside the model, a depth with no read, a head its read does not serve, a
    /// key at step 0 and a decision forced twice; and a code with a bit past its head's width.
    pub fn with_force(
        mut self,
        force: DecisionForce,
        layers: usize,
        lanes: usize,
        reads: &[(usize, usize, usize)],
    ) -> Result<Self> {
        if force.schema != DECISION_FORCE_SCHEMA {
            return Err(invalid(format!(
                "--decision-force: the schema is {}, not {DECISION_FORCE_SCHEMA}",
                force.schema
            )));
        }
        for window in force.windows {
            let position = window.position;
            if !self.positions.contains(&position) {
                return Err(invalid(format!(
                    "--decision-force: position {position} is not traced"
                )));
            }
            check_forced(&window, layers, lanes, reads)
                .map_err(|e| invalid(format!("--decision-force: position {position}: {e}")))?;
            if self.force.insert(position, window).is_some() {
                return Err(invalid(format!(
                    "--decision-force: position {position} is named twice"
                )));
            }
        }
        Ok(self)
    }

    /// The forced rows of one batch's `traced` windows: each forced bucket and code at its
    /// window's row `b · SHEET_CONTEXT + step`.
    fn batch_force(
        &self,
        windows: &[&CausalWindow],
        traced: &[usize],
    ) -> (
        Vec<crate::geometric_stack::ForcedRate>,
        Vec<crate::flock_read_hamming::ForcedRowCode>,
    ) {
        let (mut rates, mut codes) = (Vec::new(), Vec::new());
        for &b in traced {
            let Some(force) = windows[b]
                .scored_corpus_positions
                .first()
                .and_then(|p| self.force.get(p))
            else {
                continue;
            };
            let row = |step: usize| b * SHEET_CONTEXT + step;
            rates.extend(
                force
                    .buckets
                    .iter()
                    .map(|f| crate::geometric_stack::ForcedRate {
                        layer: f.layer,
                        row: row(f.step),
                        lane: f.lane,
                        entry: f.entry,
                    }),
            );
            codes.extend(
                force
                    .codes
                    .iter()
                    .map(|f| crate::flock_read_hamming::ForcedRowCode {
                        depth: f.depth,
                        row: row(f.step),
                        head: f.head,
                        key: f.role == CodeRole::Key,
                        code: f.code,
                    }),
            );
        }
        (rates, codes)
    }

    /// This trace on a float side: `F`, or `T` on an evaluation copy rounded as exported
    /// (claude-252, [`crate::fbs_export::checkpoint_trace_side`]).
    pub fn on_side(mut self, side: &str) -> Result<Self> {
        if side != "F" && side != "T" {
            return Err(invalid(format!("a float trace is side F or T, not {side}")));
        }
        self.trace.side = side.to_owned();
        Ok(self)
    }

    fn wants(&self, window: &CausalWindow) -> bool {
        window
            .scored_corpus_positions
            .first()
            .is_some_and(|p| self.positions.contains(p))
    }

    /// The trace, once every position has named exactly one window.
    pub fn finish(self) -> Result<DecisionTrace> {
        let found: Vec<usize> = self.trace.windows.iter().map(|w| w.position).collect();
        let distinct: std::collections::BTreeSet<usize> = found.iter().copied().collect();
        if distinct.len() != found.len() {
            return Err(invalid("--decision-trace: a position names two windows"));
        }
        if let Some(missing) = self.positions.difference(&distinct).next() {
            return Err(invalid(format!(
                "--decision-trace: position {missing} names no window of its split"
            )));
        }
        Ok(self.trace)
    }

    /// The shared table on the host, copied once per trace: the points the nearest scores against.
    fn hold_table(&mut self, points: &Tensor) -> Result<()> {
        if self.table.is_none() {
            self.table = Some(points.flatten_all()?.to_vec1::<f32>()?);
        }
        Ok(())
    }

    /// The decisions of the `traced` windows of one batch, from what its forward `recorded`:
    /// at each window's rows 0 to its last scored offset, each layer's rate buckets and the stream
    /// read after its layer, the readout read, the memory's lookup, and at each scored offset the
    /// nearest. `scored` and `best` are the batch's scored rows and picks, as decoded; `queries`
    /// and `row_scores` the [`traced_rows`] of its points and `nearest_scores`, on the host.
    #[allow(clippy::too_many_arguments)]
    fn record(
        &mut self,
        windows: &[&CausalWindow],
        traced: &[usize],
        ids: &[u32],
        scored: &[u32],
        best: &[u32],
        queries: &[Vec<f32>],
        row_scores: &[Vec<f32>],
        recorded: Recorded,
    ) -> Result<()> {
        let time = SHEET_CONTEXT;
        let Recorded { hamming, rates, lookups } = recorded;
        let lanes = rates.first().map_or(0, |r| r.dims.2);
        if rates.iter().enumerate().any(|(l, r)| {
            r.layer != l || r.dims != (windows.len(), time, lanes) || r.entry.len() != r.score.len()
        }) {
            return Err(invalid(
                "the decision trace's rate scores are not one per layer in order (an instrument fault)",
            ));
        }
        if !hamming.is_empty() && hamming.len() != windows.len() {
            return Err(invalid(
                "the decision trace's Hamming record is not one per window (an instrument fault)",
            ));
        }
        let heads = hamming
            .first()
            .and_then(|w| w.reads.first())
            .map_or(0, |r| r.heads.len());
        let shape = (rates.len(), lanes, heads);
        if self.trace.windows.is_empty() {
            (self.trace.layers, self.trace.lanes, self.trace.heads) = shape;
        } else if (self.trace.layers, self.trace.lanes, self.trace.heads) != shape {
            return Err(invalid("the decision trace's shape changed between batches"));
        }
        let host_table = self
            .table
            .as_deref()
            .ok_or_else(|| invalid("the decision trace holds no table (an instrument fault)"))?;
        let at = traced_rows(scored, traced);
        if queries.len() != at.len() || row_scores.len() != at.len() {
            return Err(invalid(
                "the decision trace's host rows are not the traced windows' scored rows (an instrument fault)",
            ));
        }
        let mut next = 0;
        for &b in traced {
            let window = windows[b];
            let (&first, &last) = window
                .scored_offsets
                .first()
                .zip(window.scored_offsets.last())
                .ok_or_else(|| invalid("a traced window has no scored row"))?;
            let reads = match hamming.get(b) {
                None => &[][..],
                Some(w) if w.ids[..] == ids[b * time..(b + 1) * time] => &w.reads[..],
                Some(_) => {
                    return Err(invalid(
                        "the decision trace's Hamming record names another window (an instrument fault)",
                    ))
                }
            };
            // The stream read opens a window's Hamming record and the readout read joins it.
            let (stream, readout) = match reads {
                [] => (None, None),
                [s] => (Some(s), None),
                [s, r] => (Some(s), Some(r)),
                _ => return Err(invalid("a window's Hamming record holds more than two reads")),
            };
            let nearest_rows: BTreeMap<usize, usize> = window
                .scored_offsets
                .iter()
                .enumerate()
                .map(|(j, &off)| (off, next + j))
                .collect();
            next += window.scored_offsets.len();
            let mut steps = Vec::with_capacity(last + 1);
            for t in 0..=last {
                let row = b * time + t;
                let layers = rates
                    .iter()
                    .enumerate()
                    .map(|(l, r)| {
                        let score = &r.score[row * lanes..(row + 1) * lanes];
                        Ok(DecisionLayer {
                            score_q16: Vec::new(),
                            score_f32: score.iter().map(|s| s.to_bits()).collect(),
                            // The entry rate selection read: the score's, or a forced one.
                            entry: r.entry[row * lanes..(row + 1) * lanes].to_vec(),
                            read: match stream {
                                Some(s) if s.depth == l + 1 => Some(decision_read(s, t)?),
                                _ => None,
                            },
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                let nearest = match nearest_rows.get(&t) {
                    None => None,
                    Some(&i) => {
                        let pick = best[at[i] as usize];
                        let runner = runner_up(&row_scores[i], pick)?;
                        let width = queries[i].len();
                        let score = |u: u32| -> Result<Option<u64>> {
                            let point = host_table
                                .get(u as usize * width..(u as usize + 1) * width)
                                .ok_or_else(|| invalid("a pick names no row of the table"))?;
                            Ok(Some(l1_score_r(&queries[i], point).to_bits()))
                        };
                        Some(DecisionNearest {
                            pick,
                            score_q12: None,
                            score_f64: score(pick)?,
                            runner_up: runner,
                            runner_score_q12: None,
                            runner_score_f64: score(runner)?,
                            query_q12: Vec::new(),
                            query_f32: queries[i].iter().map(|v| v.to_bits()).collect(),
                        })
                    }
                };
                steps.push(DecisionStep {
                    layers,
                    readout: readout.map(|r| decision_read(r, t)).transpose()?,
                    memory: lookups.as_ref().and_then(|l| l[row].as_ref()).map(|m| {
                        DecisionMemory {
                            key: m.key.clone(),
                            stored: m.stored,
                            opening: m.opening,
                        }
                    }),
                    nearest,
                });
            }
            if let Some(force) = self.force.get(&window.scored_corpus_positions[0]) {
                forced_taken(force, &steps)?;
            }
            self.trace.windows.push(DecisionWindow {
                position: window.scored_corpus_positions[0],
                ids: window.input_ids[..=last].to_vec(),
                reply_start: first,
                steps,
            });
        }
        Ok(())
    }
}

/// The traced windows' scored rows, in panel order: their indices into a batch's `scored` rows,
/// each a `window · SHEET_CONTEXT + offset` row of the batch.
fn traced_rows(scored: &[u32], traced: &[usize]) -> Vec<u32> {
    (0..scored.len() as u32)
        .filter(|&i| traced.contains(&(scored[i as usize] as usize / SHEET_CONTEXT)))
        .collect()
}

/// One window's forced set against the model's shape (claude-253): R's refusals, as
/// `set_decision_trace` and the reads' `forcing` make them, and a code with a bit past its head's
/// width. `reads` holds each read's depth, heads and head width.
fn check_forced(
    window: &DecisionForceWindow,
    layers: usize,
    lanes: usize,
    reads: &[(usize, usize, usize)],
) -> std::result::Result<(), String> {
    let mut buckets = std::collections::BTreeSet::new();
    for f in &window.buckets {
        if f.step >= SHEET_CONTEXT || f.layer >= layers || f.lane >= lanes || f.entry > 15 {
            return Err(
                "a forced bucket names a step, layer, lane or entry outside the model".to_owned(),
            );
        }
        if !buckets.insert((f.step, f.layer, f.lane)) {
            return Err("a bucket is forced twice".to_owned());
        }
    }
    let mut codes = std::collections::BTreeSet::new();
    for f in &window.codes {
        let &(_, heads, width) = reads
            .iter()
            .find(|r| r.0 == f.depth)
            .ok_or("a forced code names a depth no read runs at")?;
        if f.head >= heads {
            return Err("a forced code names a head its read does not serve".to_owned());
        }
        if f.step >= SHEET_CONTEXT {
            return Err("a forced code names a step past the context".to_owned());
        }
        if f.role == CodeRole::Key && f.step == 0 {
            return Err("a forced key at step 0 names no candidate".to_owned());
        }
        if width < 64 && f.code >> width != 0 {
            return Err("a forced code sets a bit past its head's width".to_owned());
        }
        if !codes.insert((f.step, f.depth, f.head, f.role == CodeRole::Key)) {
            return Err("a code is forced twice".to_owned());
        }
    }
    Ok(())
}

/// Refuses, before any forward, a forced decision at a step past its window's traced steps:
/// the trace records a window from step 0 to its last scored offset, and no further.
fn forced_within(force: &DecisionForce, windows: &[CausalWindow]) -> Result<()> {
    for w in &force.windows {
        let last = windows
            .iter()
            .find(|c| c.scored_corpus_positions.first() == Some(&w.position))
            .and_then(|c| c.scored_offsets.last())
            .ok_or_else(|| {
                invalid(format!(
                    "--decision-force: position {} names no window of its split",
                    w.position
                ))
            })?;
        let steps = w
            .buckets
            .iter()
            .map(|b| b.step)
            .chain(w.codes.iter().map(|c| c.step));
        if let Some(step) = steps.filter(|s| s > last).max() {
            return Err(invalid(format!(
                "--decision-force: position {}: step {step} is past the window's traced steps (0 to {last})",
                w.position
            )));
        }
    }
    Ok(())
}

/// Each decision of a window's forced set as its traced `steps` recorded it (claude-253, the
/// card's I-Force): every forced bucket and code took its value in the forward. Refused, as an
/// instrument fault: one that did not. A step past the window's traced steps is refused here
/// too, as a backstop: `forced_within` refuses it before any forward.
fn forced_taken(force: &DecisionForceWindow, steps: &[DecisionStep]) -> Result<()> {
    let at = |step: usize| {
        steps.get(step).ok_or_else(|| {
            invalid(format!(
                "a forced decision at step {step} is past the window's traced steps"
            ))
        })
    };
    for f in &force.buckets {
        let used = at(f.step)?
            .layers
            .get(f.layer)
            .and_then(|l| l.entry.get(f.lane));
        if used != Some(&f.entry) {
            return Err(invalid(format!(
                "the bucket forced at step {}, layer {}, lane {} was not taken (an instrument fault)",
                f.step, f.layer, f.lane
            )));
        }
    }
    for f in &force.codes {
        let step = at(f.step)?;
        let head = step
            .layers
            .iter()
            .filter_map(|l| l.read.as_ref())
            .chain(&step.readout)
            .find(|r| r.depth == f.depth)
            .and_then(|r| r.heads.get(f.head));
        let used = head.map(|h| match f.role {
            CodeRole::Query => h.query,
            CodeRole::Key => h.key,
        });
        if used != Some(f.code) {
            return Err(invalid(format!(
                "the code forced at step {}, depth {}, head {} was not taken (an instrument fault)",
                f.step, f.depth, f.head
            )));
        }
    }
    Ok(())
}

/// One read's choices at row `t`, from its Hamming record: each head's codes, its coordinates,
/// and its kept set in R's order ([`kept_in_order`]).
fn decision_read(read: &HammingTraceRead, t: usize) -> Result<DecisionRead> {
    let heads = read
        .heads
        .iter()
        .map(|h| {
            let row = |rows: &[Vec<u32>]| -> Result<Vec<u32>> {
                rows.get(t)
                    .cloned()
                    .ok_or_else(|| invalid("a traced head has no coordinates (an instrument fault)"))
            };
            Ok(DecisionHead {
                query: h.query[t],
                key: h.key[t],
                kept: kept_in_order(h.query[t], &h.key, &h.kept[t]),
                query_f32: row(&h.query_f32)?,
                key_f32: row(&h.key_f32)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(DecisionRead { depth: read.depth, heads })
}

/// The kept positions in R's order: nearest first by the Hamming distance from the query code
/// to each position's key code, ties to the earlier position.
fn kept_in_order(query: u64, keys: &[u64], kept: &[u32]) -> Vec<u32> {
    let mut kept = kept.to_vec();
    kept.sort_by_key(|&s| ((query ^ keys[s as usize]).count_ones(), s));
    kept
}

/// The best-scoring token other than `pick` in one row of `nearest_scores`, ties to the smaller
/// id: the nearest's runner-up as the GPU scored it.
fn runner_up(scores: &[f32], pick: u32) -> Result<u32> {
    let mut best: Option<(u32, f32)> = None;
    for (u, &s) in scores.iter().enumerate() {
        if u as u32 != pick && best.is_none_or(|(_, b)| s > b) {
            best = Some((u as u32, s));
        }
    }
    best.map(|(u, _)| u)
        .ok_or_else(|| invalid("the nearest needs two tokens to name a runner-up"))
}

/// A token's nearest score in R's units, `−2^12 · Σ_j |q_j − e_j|` in f64 over the f32 query and
/// table point: R scores its Q12 query against the Q12 table, so one Q12 unit is one here.
fn l1_score_r(query: &[f32], point: &[f32]) -> f64 {
    let sum: f64 = query
        .iter()
        .zip(point)
        .map(|(&q, &e)| (f64::from(q) - f64::from(e)).abs())
        .sum();
    -4096.0 * sum
}

/// claude-251: turn on (or off) every capture a traced forward makes: the Hamming reads' codes
/// with their coordinates, and each layer's rate scores.
fn set_decision_capture(on: bool) {
    crate::flock_read_hamming::set_hamming_trace(on);
    crate::flock_read_hamming::set_hamming_trace_coords(on);
    crate::geometric_stack::set_rate_trace(on);
}

/// claude-251: the reads R does not serve, any of which would make a decision the trace leaves
/// out; the trace refuses a checkpoint that sets one.
fn unserved_reads() -> Vec<&'static str> {
    let mut set = Vec::new();
    if crate::flock_read::flock_read().is_some() {
        set.push("the Flock read");
    }
    if crate::flock_read_content::content_read().is_some() {
        set.push("the content read");
    }
    if crate::flock_read_opening::opening_read().is_some() {
        set.push("the opening read");
    }
    if crate::flock_read_register::prompt_register().is_some() {
        set.push("the prompt register");
    }
    set
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

/// The flat ids, scored indices and targets of `windows`, as the sheet's batches lay them out:
/// each scored index is its window's start in the flat ids plus its offset, so windows of any
/// one length land at `b · length + offset` (claude-273; before, `b · SHEET_CONTEXT`).
fn assemble(windows: &[&CausalWindow]) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
    let mut ids = Vec::with_capacity(windows.iter().map(|w| w.input_ids.len()).sum());
    let (mut scored, mut targets) = (Vec::new(), Vec::new());
    for window in windows {
        let start = ids.len();
        ids.extend_from_slice(&window.input_ids);
        for (&off, &target) in window.scored_offsets.iter().zip(&window.scored_targets) {
            scored.push((start + off) as u32);
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
/// commonest opening for each key, scored on a split's openings ([`opening_bound`]): DEV, or a
/// `--panel` (claude-218). A count over the corpus, not a model: it bounds what a read that finds
/// those words could add to the openings.
#[derive(Clone, Debug, Serialize)]
pub struct OpeningBound {
    /// The scored split: `dev`, or the panel's name.
    pub split: String,
    /// The panel's token file sha256; DEV is pinned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    /// `none`, `first-k` (the first `k` prompt tokens the window shows), `last-k` (the `k`
    /// tokens before the opening), `set` and `set-first-k` (those tokens' distinct ids in id
    /// order, an order-free key, claude-218), or `reply-template`: the oracle U, keyed by the
    /// reference reply itself with its values masked as the census masks them
    /// ([`OPENING_TEMPLATE_K`]), the most any key that knew the question could name.
    pub key: String,
    pub train_rows: usize,
    pub train_keys: usize,
    pub rows: usize,
    /// Openings whose key TRAIN never saw; TRAIN's commonest opening names them.
    pub unseen: usize,
    pub hits: usize,
    /// The hits among the openings whose key TRAIN saw, where the key fires: its precision is
    /// `fired_hits / (rows − unseen)`.
    pub fired_hits: usize,
    /// TRAIN's most frequent keys, decoded, with their commonest opening and its share.
    pub top: Vec<OpeningKey>,
}

/// The census's value mask for [`OpeningBound`]'s `reply-template` key: TRAIN's 1,024 most
/// frequent tokens are kept, the K the panel census reads its templates at.
const OPENING_TEMPLATE_K: usize = 1024;

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

/// One opening [`openings`] yields, with where it stands: its window's index, the offset of the
/// input token it is predicted from, and its target's corpus position.
struct OpeningRow {
    window: usize,
    offset: usize,
    position: usize,
    prompt: Vec<u32>,
    target: u32,
}

/// The reply that starts at corpus `position`: its tokens while the mask holds, up to BOS.
fn reply_at<'a>(tokens: &'a [u32], mask: &[u8], position: usize) -> &'a [u32] {
    let end = (position..tokens.len())
        .find(|&i| mask[i] == 0 || tokens[i] == hs::SHEET_BOS_ID)
        .unwrap_or(tokens.len());
    &tokens[position..end]
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
                position,
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
    /// The split: `dev`, a panel's name, or `train`.
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
    /// The prompt's distinct ids, in id order: the same key for any order of the same words.
    Set,
    /// The first `k` prompt tokens' distinct ids, in id order.
    SetFirst(usize),
}

impl PromptKey {
    fn of(self, prompt: &[u32]) -> Vec<u32> {
        let set = |ids: &[u32]| {
            let mut ids = ids.to_vec();
            ids.sort_unstable();
            ids.dedup();
            ids
        };
        match self {
            PromptKey::None => Vec::new(),
            PromptKey::First(k) => prompt[..k.min(prompt.len())].to_vec(),
            PromptKey::Last(k) => prompt[prompt.len() - k.min(prompt.len())..].to_vec(),
            PromptKey::Set => set(prompt),
            PromptKey::SetFirst(k) => set(&prompt[..k.min(prompt.len())]),
        }
    }

    fn name(self) -> String {
        match self {
            PromptKey::None => "none".to_owned(),
            PromptKey::First(k) => format!("first-{k}"),
            PromptKey::Last(k) => format!("last-{k}"),
            PromptKey::Set => "set".to_owned(),
            PromptKey::SetFirst(k) => format!("set-first-{k}"),
        }
    }
}

/// [`OpeningBound`] for one key: `train` and `split` are the openings already keyed, each as its
/// key and its target.
fn bound(
    split: &str,
    key: String,
    train: &[(Vec<u32>, u32)],
    rows: &[(Vec<u32>, u32)],
    decode: &dyn Fn(&[u32]) -> String,
) -> OpeningBound {
    let mut table =
        std::collections::BTreeMap::<Vec<u32>, std::collections::BTreeMap<u32, usize>>::new();
    let mut overall = std::collections::BTreeMap::<u32, usize>::new();
    for (k, target) in train {
        *table.entry(k.clone()).or_default().entry(*target).or_default() += 1;
        *overall.entry(*target).or_default() += 1;
    }
    let fallback = commonest(&overall).map(|(t, _)| t);
    let (mut unseen, mut hits, mut fired_hits) = (0, 0, 0);
    for (k, target) in rows {
        let (named, fired) = match table.get(k) {
            Some(counts) => (commonest(counts).map(|(t, _)| t), true),
            None => (fallback, false),
        };
        let hit = named == Some(*target);
        unseen += usize::from(!fired);
        hits += usize::from(hit);
        fired_hits += usize::from(hit && fired);
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
        split: split.to_owned(),
        sha256: None,
        key,
        train_rows: train.len(),
        train_keys: table.len(),
        rows: rows.len(),
        unseen,
        hits,
        fired_hits,
        top,
    }
}

/// Each opening of `rows` keyed by its reply's template: the reply at its position with its
/// values masked ([`value_mask`]), and its target.
fn template_keyed(
    rows: &[OpeningRow],
    tokens: &[u32],
    mask: &[u8],
    keep: &std::collections::HashSet<u32>,
    digits: &std::collections::HashSet<u32>,
) -> Vec<(Vec<u32>, u32)> {
    rows.iter()
        .map(|r| (value_mask(reply_at(tokens, mask, r.position), keep, digits), r.target))
        .collect()
}

/// [`OpeningBound`] on DEV and on each of `panels` (claude-218), for no key, the first 1–8 prompt
/// tokens, the last 1–4, the order-free `set` and `set-first-8`, and the oracle
/// `reply-template`; and the opening read's [`AnchorCheck`] on every split and on TRAIN.
pub fn opening_bound(
    data_root: &Path,
    panels: &[(String, PathBuf)],
) -> Result<(Vec<OpeningBound>, Vec<AnchorCheck>)> {
    let bos = hs::SHEET_BOS_ID;
    let tokenizer = pinned_tokenizer(data_root)?;
    let shown = |ids: &[u32]| -> Vec<u32> { ids.iter().copied().filter(|&t| t != CENSUS_MASK).collect() };
    let decode = |ids: &[u32]| tail(&tokenizer.decode(&shown(ids)), 80);
    let (train_tokens, train_mask) = load_train_split(data_root)?;
    let (train_windows, _) =
        hs::build_document_windows(&train_tokens, &train_mask, SHEET_CONTEXT, bos)?;
    let (dev_tokens, dev_mask) = hs::load_pinned_uort_split(
        &data_root.join("ft/dev/tokens.u16"),
        &data_root.join("ft/dev/response_mask.u8"),
        hs::PINNED_DEV_TOKENS_SHA256,
        hs::PINNED_DEV_MASK_SHA256,
        hs::EXPECTED_DEV_TOKENS,
        hs::EXPECTED_DEV_ELIGIBLE_TARGETS,
        hs::SHEET_VOCAB,
        bos,
    )?;
    let mut splits = vec![("dev".to_owned(), dev_tokens, dev_mask, None)];
    for (name, dir) in panels {
        if !panel_name_ok(name) {
            return Err(invalid(format!("--panel {name}: not a panel name")));
        }
        let (tokens, mask, sha) = load_unpinned_uort(dir)?;
        splits.push((name.clone(), tokens, mask, Some(sha)));
    }
    let keep = top_tokens(&train_tokens, OPENING_TEMPLATE_K, bos);
    let digits = digit_tokens(&fs::read_to_string(data_root.join("tokenizer.json"))?)?;
    let train_rows = opening_rows(&train_windows, &train_mask);
    let train_templates = template_keyed(&train_rows, &train_tokens, &train_mask, &keep, &digits);
    let keys: Vec<PromptKey> = std::iter::once(PromptKey::None)
        .chain((1..=8).map(PromptKey::First))
        .chain((1..=4).map(PromptKey::Last))
        .chain([PromptKey::Set, PromptKey::SetFirst(8)])
        .collect();
    let keyed = |rows: &[OpeningRow], key: PromptKey| -> Vec<(Vec<u32>, u32)> {
        rows.iter().map(|r| (key.of(&r.prompt), r.target)).collect()
    };
    let mut bounds = Vec::new();
    let mut checks = Vec::new();
    for (name, tokens, mask, sha) in &splits {
        let (windows, _) = hs::build_document_windows(tokens, mask, SHEET_CONTEXT, bos)?;
        let rows = opening_rows(&windows, mask);
        // Every split's anchor check (claude-218, FIT round 1): the memory's turn slots keep the
        // first-8 key at its openings, so the laptop's exact key is the memory's served one there.
        checks.push(anchor_check(name, &windows, mask));
        let mut split_bounds: Vec<OpeningBound> = keys
            .iter()
            .map(|&key| bound(name, key.name(), &keyed(&train_rows, key), &keyed(&rows, key), &decode))
            .collect();
        let templates = template_keyed(&rows, tokens, mask, &keep, &digits);
        split_bounds.push(bound(name, "reply-template".to_owned(), &train_templates, &templates, &decode));
        for b in &mut split_bounds {
            b.sha256 = sha.clone();
        }
        bounds.extend(split_bounds);
    }
    checks.push(anchor_check("train", &train_windows, &train_mask));
    Ok((bounds, checks))
}

/// The trunk's own point at every opening of `windows` (claude-218): the readout point that
/// predicts a reply's first token, with no turn memory moving it (the caller has it off), in
/// panel order, with each opening's target and corpus position. A forward of the windows that
/// hold an opening, under the gate mode the rows are scored in ([`row_picks`]).
fn opening_points(
    model: &StackModel,
    windows: &[CausalWindow],
    mask: &[u8],
    seed: u64,
    step: usize,
    served: bool,
) -> Result<(Tensor, Vec<u32>, Vec<usize>)> {
    if crate::flock_read_memory::turn_memory().is_some() {
        return Err(invalid("the opening keys read the trunk's own points, with the memory off"));
    }
    with_eval_forward(served, seed, step, || {
        let (mut points, mut targets, mut positions) = (Vec::new(), Vec::new(), Vec::new());
        for chunk in windows.chunks(hs::SHEET_EVAL_BATCH) {
            let windows: Vec<&CausalWindow> = chunk.iter().collect();
            let (ids, scored, batch_targets) = assemble(&windows);
            let batch_positions: Vec<usize> = windows
                .iter()
                .flat_map(|w| w.scored_corpus_positions.iter().copied())
                .collect();
            let open: Vec<usize> =
                (0..scored.len()).filter(|&i| opens_reply(mask, batch_positions[i])).collect();
            if open.is_empty() {
                continue;
            }
            if trained_cut_on() {
                set_reply_mask(Some(window_input_mask(&windows, mask)?));
            }
            let hidden = trained_points(model, &ids, windows.len())?;
            let rows: Vec<u32> = open.iter().map(|&i| scored[i]).collect();
            let idx = Tensor::from_vec(rows, open.len(), model.device())?;
            let point = hidden.index_select(&idx, 0)?;
            points.push(if served {
                hs::q12_query(&point)?.detach()
            } else {
                point.detach()
            });
            targets.extend(open.iter().map(|&i| batch_targets[i]));
            positions.extend(open.iter().map(|&i| batch_positions[i]));
        }
        if points.is_empty() {
            return Err(invalid("the split has no opening rows"));
        }
        Ok((Tensor::cat(&points, 0)?, targets, positions))
    })
}

/// One opening as a trunk key names it ([`l1_named`]).
#[derive(Clone, Copy, Debug, PartialEq)]
struct Named {
    /// The commonest target among TRAIN's openings at the least distance, ties to the smaller id.
    target: u32,
    distance: f32,
    /// How many TRAIN openings are at that distance.
    ties: u32,
}

/// For each row of `query`, the rows of `table` at the least L1 distance (the mean `|q − t|` over
/// the coordinates, as the cell objective scores), named by the commonest of their `targets`,
/// ties to the smaller id, as the exact key names its table (FIT round 1): on the device in
/// blocks, with no dot product. A first pass finds each query's least distance and a second the
/// rows at it; both compute the same distances by the same kernels, so they agree exactly.
fn l1_named(query: &Tensor, table: &Tensor, targets: &[u32]) -> Result<Vec<Named>> {
    const QUERIES: usize = 64;
    const ROWS: usize = 4096;
    let (nq, _) = query.dims2()?;
    let (nt, _) = table.dims2()?;
    if targets.len() != nt {
        return Err(invalid("the trunk keys want one target per TRAIN opening"));
    }
    let distances = |q: &Tensor, t0: usize| -> Result<Tensor> {
        let t = table.narrow(0, t0, ROWS.min(nt - t0))?.unsqueeze(0)?;
        Ok(q.broadcast_sub(&t)?.abs()?.mean(2)?)
    };
    let mut named = Vec::with_capacity(nq);
    for q0 in (0..nq).step_by(QUERIES) {
        let n = QUERIES.min(nq - q0);
        let q = query.narrow(0, q0, n)?.unsqueeze(1)?;
        let mut least = vec![f32::INFINITY; n];
        for t0 in (0..nt).step_by(ROWS) {
            let low = distances(&q, t0)?.min(1)?.to_vec1::<f32>()?;
            for (l, m) in least.iter_mut().zip(low) {
                if m < *l {
                    *l = m;
                }
            }
        }
        let at = Tensor::from_vec(least.clone(), (n, 1), query.device())?;
        let mut counts = vec![std::collections::BTreeMap::<u32, usize>::new(); n];
        for t0 in (0..nt).step_by(ROWS) {
            let tied = distances(&q, t0)?.broadcast_eq(&at)?.to_vec2::<u8>()?;
            for (count, row) in counts.iter_mut().zip(&tied) {
                for (j, _) in row.iter().enumerate().filter(|(_, &f)| f == 1) {
                    *count.entry(targets[t0 + j]).or_default() += 1;
                }
            }
        }
        for (count, distance) in counts.iter().zip(least) {
            let (target, _) =
                commonest(count).ok_or_else(|| invalid("a trunk key found no TRAIN opening at its least distance"))?;
            named.push(Named { target, distance, ties: count.values().sum::<usize>() as u32 });
        }
    }
    Ok(named)
}

/// Each point's sign pattern as 0/1 coordinates: L1 between two of them is their Hamming
/// distance over the width, so [`l1_named`] on them is the Hamming-nearest sign cell.
fn sign_code(points: &Tensor) -> Result<Tensor> {
    Ok(points.ge(&points.zeros_like()?)?.to_dtype(DType::F32)?)
}

/// The fixed ±1 signs S of the Hadamard rotation (claude-218; Casey, 30 Oct: "ok add that
/// trick", the rotation in PrismML's Bonsai 2): coordinate `i` is negated when its splitmix64
/// hash is odd, so S is exact, fixed and reproducible.
fn rotation_signs(width: usize) -> Vec<f32> {
    (0..width as u64)
        .map(|i| {
            let mut z = i.wrapping_add(0x9E37_79B9_7F4A_7C15);
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^= z >> 31;
            if z & 1 == 1 { -1.0 } else { 1.0 }
        })
        .collect()
}

/// The fast Walsh–Hadamard transform of each row of `x`, whose width is a power of two,
/// unnormalised: log2(width) stages of adds and subtracts on the device, with no dot product.
fn walsh_hadamard(x: &Tensor) -> Result<Tensor> {
    let (n, w) = x.dims2()?;
    if !w.is_power_of_two() {
        return Err(invalid(format!("the Walsh–Hadamard transform wants a power-of-two width, not {w}")));
    }
    let mut y = x.contiguous()?;
    let mut h = 1;
    while h < w {
        let v = y.reshape((n, w / (2 * h), 2, h))?;
        let (a, b) = (v.narrow(2, 0, 1)?, v.narrow(2, 1, 1)?);
        y = Tensor::cat(&[(&a + &b)?, (&a - &b)?], 2)?.reshape((n, w))?;
        h *= 2;
    }
    Ok(y)
}

/// The Hadamard sign code: the sign pattern of `H·S·x` ([`sign_code`] after the fixed signs and
/// the transform). The rotation spreads each point's magnitude across the coordinates, so its
/// Hamming distance compares points along spread-out directions rather than the raw axes.
fn hadamard_sign_code(points: &Tensor) -> Result<Tensor> {
    let (_, w) = points.dims2()?;
    let signs = Tensor::from_vec(rotation_signs(w), (1, w), points.device())?;
    sign_code(&walsh_hadamard(&points.broadcast_mul(&signs)?)?)
}

/// One split's openings named by the two trunk keys (claude-218), from TRAIN's opening points
/// under the same trunk: by point under L1, and by sign pattern (Hamming), each as the commonest
/// target among the TRAIN openings at the least distance ([`l1_named`]), with the distance and
/// the tie count. Written by `--opening-keys 1` as `keys-{split}.json`; the laptop grade reads it
/// with the rows.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpeningKeys {
    pub split: String,
    /// TRAIN's openings, the table both keys search, and its bytes as f32 points.
    pub train_openings: usize,
    pub table_bytes: usize,
    pub positions: Vec<usize>,
    pub targets: Vec<u32>,
    pub point_target: Vec<u32>,
    pub point_distance: Vec<f32>,
    pub point_ties: Vec<u32>,
    pub sign_target: Vec<u32>,
    pub sign_distance: Vec<f32>,
    pub sign_ties: Vec<u32>,
    /// Each key's search over all the split's openings, in milliseconds: the point key's, and
    /// the sign key's with the queries' sign codes taken (TRAIN's are taken before it starts, as
    /// a served table would hold them).
    pub point_ms: u64,
    pub sign_ms: u64,
    /// The Hadamard sign key ([`hadamard_sign_code`], Casey's amendment of 30 Oct), as the sign
    /// key; empty in a keys file written before it.
    #[serde(default)]
    pub hsign_target: Vec<u32>,
    #[serde(default)]
    pub hsign_distance: Vec<f32>,
    #[serde(default)]
    pub hsign_ties: Vec<u32>,
    #[serde(default)]
    pub hsign_ms: u64,
}

impl OpeningKeys {
    /// One trunk key's names and distances: `point`, `hsign`, or else `sign`.
    fn by_key(&self, key: &str) -> (&[u32], &[f32]) {
        match key {
            "point" => (&self.point_target, &self.point_distance),
            "hsign" => (&self.hsign_target, &self.hsign_distance),
            _ => (&self.sign_target, &self.sign_distance),
        }
    }
}

/// The keyed opening (claude-219): a trunk key's `named` target as the reply's first token,
/// where the memory's exact key misses the opening (`fires` false) and the key's `distance` is at
/// most `tau`; elsewhere none, and the decode is greedy with the memory, as served. A τ of +∞
/// fires at every opening the exact key misses.
fn keyed_opening(fires: bool, named: u32, distance: f32, tau: f32) -> Option<u32> {
    (!fires && distance <= tau).then_some(named)
}

/// Whether `--keyed-decode`'s arms are distinct trunk keys, each with a τ of zero or more.
fn keyed_ok(keyed: &[(String, f32)]) -> bool {
    keyed.iter().enumerate().all(|(i, (key, tau))| {
        TRUNK_KEYS.contains(&key.as_str()) && *tau >= 0.0 && !keyed[..i].iter().any(|(k, _)| k == key)
    })
}

/// The keyed openings ([`keyed_opening`]) of a split's sampled `prompts` under the trunk key
/// `key` at `tau`, from the split's `keys`: whether the exact key fires is the served `memory`'s
/// own first-8 lookup on the decode's first window. A sampled opening the keys do not name, or
/// name with another target, is an instrument fault.
fn keyed_openings(
    prompts: &[&CausalWindow],
    keys: &OpeningKeys,
    key: &str,
    tau: f32,
    memory: Option<&crate::flock_read_memory::TurnMemory>,
) -> Result<Vec<Option<u32>>> {
    let (names, distances) = keys.by_key(key);
    let at: std::collections::HashMap<usize, usize> =
        keys.positions.iter().enumerate().map(|(i, &p)| (p, i)).collect();
    prompts
        .iter()
        .map(|window| {
            let position = window.scored_corpus_positions[0];
            let i = *at.get(&position).ok_or_else(|| {
                invalid(format!("{}: no key names the sampled opening at {position} (an instrument fault)", keys.split))
            })?;
            if keys.targets[i] != window.scored_targets[0] {
                return Err(invalid(format!(
                    "{}: the keys' target at {position} is not the window's (an instrument fault)",
                    keys.split
                )));
            }
            let prompt = &window.input_ids[..=window.scored_offsets[0]];
            let first = &prompt[prompt.len().saturating_sub(SHEET_CONTEXT)..];
            let fires = match memory {
                Some(m) => m.stored(first, first.len(), &[first.len() - 1])?.1[first.len() - 1] == 1.0,
                None => false,
            };
            let token = keyed_opening(fires, names[i], distances[i], tau);
            // A key's name is a TRAIN reply's first token, never BOS (the code review).
            if token == Some(hs::SHEET_BOS_ID) {
                return Err(invalid(format!("{}: a key names BOS at {position} (an instrument fault)", keys.split)));
            }
            Ok(token)
        })
        .collect()
}

/// [`OpeningKeys`] for one split's opening points against TRAIN's.
fn opening_keys(
    split: &str,
    query: &Tensor,
    targets: Vec<u32>,
    positions: Vec<usize>,
    train: &Tensor,
    train_targets: &[u32],
) -> Result<OpeningKeys> {
    let started = std::time::Instant::now();
    let by_point = l1_named(query, train, train_targets)?;
    let point_ms = started.elapsed().as_millis() as u64;
    let train_sign = sign_code(train)?;
    let started = std::time::Instant::now();
    let by_sign = l1_named(&sign_code(query)?, &train_sign, train_targets)?;
    let sign_ms = started.elapsed().as_millis() as u64;
    drop(train_sign);
    let train_hsign = hadamard_sign_code(train)?;
    let started = std::time::Instant::now();
    let by_hsign = l1_named(&hadamard_sign_code(query)?, &train_hsign, train_targets)?;
    let hsign_ms = started.elapsed().as_millis() as u64;
    let (rows, width) = train.dims2()?;
    Ok(OpeningKeys {
        split: split.to_owned(),
        train_openings: rows,
        table_bytes: rows * width * 4,
        positions,
        targets,
        point_target: by_point.iter().map(|n| n.target).collect(),
        point_distance: by_point.iter().map(|n| n.distance).collect(),
        point_ties: by_point.iter().map(|n| n.ties).collect(),
        sign_target: by_sign.iter().map(|n| n.target).collect(),
        sign_distance: by_sign.iter().map(|n| n.distance).collect(),
        sign_ties: by_sign.iter().map(|n| n.ties).collect(),
        point_ms,
        sign_ms,
        hsign_target: by_hsign.iter().map(|n| n.target).collect(),
        hsign_distance: by_hsign.iter().map(|n| n.distance).collect(),
        hsign_ties: by_hsign.iter().map(|n| n.ties).collect(),
        hsign_ms,
    })
}

/// The development and deciding panels' token and mask sha256s (claude-218's sheet), checked
/// before the key grade reads them.
const KEY_PANELS: [(&str, &str, &str); 2] = [
    (
        "p-s",
        "62099a4800f6ae34a467762a42fcc79535836f222f64a1613d57aea9176610f1",
        "9053e117eab299b48623f00f293aa7dd5275cae3804a6ce55f01b1303f05f11a",
    ),
    (
        "p-u",
        "d8e80f38950769650a0f5786a42fbfff3332c9d3cf49a3e42d2cfe428502fbd0",
        "c883aa4a14c3cf79460c8706de66c3eb5d8166ee89844484c9bb523c45206865",
    ),
];

/// The forced arm's and greedy's replies per seed and split, as the sheet registers them.
const KEY_FORCED_REPLIES: usize = 256;

/// The trunk keys the key grade reads, the cheaper first: if more than one passes, the sign key
/// ships, then the Hadamard sign key (the same 32 bytes, plus the rotation), then the point key.
const TRUNK_KEYS: [&str; 3] = ["sign", "hsign", "point"];

/// An exact table over keyed openings: each key's commonest target, ties to the smaller id, and
/// the overall commonest for an unseen key, as [`bound`] names them.
struct KeyTable {
    named: std::collections::HashMap<Vec<u32>, u32>,
    fallback: u32,
}

impl KeyTable {
    fn new(rows: impl Iterator<Item = (Vec<u32>, u32)>) -> Self {
        let mut table =
            std::collections::HashMap::<Vec<u32>, std::collections::BTreeMap<u32, usize>>::new();
        let mut overall = std::collections::BTreeMap::<u32, usize>::new();
        for (key, target) in rows {
            *table.entry(key).or_default().entry(target).or_default() += 1;
            *overall.entry(target).or_default() += 1;
        }
        KeyTable {
            named: table.into_iter().filter_map(|(k, c)| commonest(&c).map(|(t, _)| (k, t))).collect(),
            fallback: commonest(&overall).map_or(u32::MAX, |(t, _)| t),
        }
    }

    /// The target `key` names, and whether it fires (TRAIN holds it).
    fn name(&self, key: &[u32]) -> (u32, bool) {
        self.named.get(key).map_or((self.fallback, false), |&t| (t, true))
    }
}

/// One split's openings as the key grade reads them, the same at every seed: each opening's
/// position and target; the exact key's name and whether it fires (the memory's `first-8`); the
/// oracle U's name (the reference reply's value-masked template); and whether the reference is a
/// verbatim TRAIN reply (the known stratum).
struct GradeOpenings {
    name: String,
    positions: Vec<usize>,
    targets: Vec<u32>,
    exact: Vec<(u32, bool)>,
    oracle: Vec<u32>,
    known: Vec<bool>,
}

#[allow(clippy::too_many_arguments)]
fn grade_openings(
    name: &str,
    tokens: &[u32],
    mask: &[u8],
    exact: &KeyTable,
    oracle: &KeyTable,
    keep: &std::collections::HashSet<u32>,
    digits: &std::collections::HashSet<u32>,
    train_replies: &std::collections::HashSet<u128>,
) -> Result<GradeOpenings> {
    let (windows, _) = hs::build_document_windows(tokens, mask, SHEET_CONTEXT, hs::SHEET_BOS_ID)?;
    // The memory's turn slots must keep the first-8 key at every opening, so the laptop's exact
    // key is the served one (FIT round 3).
    let anchor = anchor_check(name, &windows, mask);
    if anchor.agree != anchor.openings {
        return Err(invalid(format!(
            "{name}: the turn anchor keeps the first-8 key at {} of {} openings (an instrument fault)",
            anchor.agree, anchor.openings
        )));
    }
    let rows = opening_rows(&windows, mask);
    let reply = |r: &OpeningRow| reply_at(tokens, mask, r.position);
    Ok(GradeOpenings {
        name: name.to_owned(),
        positions: rows.iter().map(|r| r.position).collect(),
        targets: rows.iter().map(|r| r.target).collect(),
        exact: rows.iter().map(|r| exact.name(&PromptKey::First(8).of(&r.prompt))).collect(),
        oracle: rows.iter().map(|r| oracle.name(&value_mask(reply(r), keep, digits)).0).collect(),
        known: rows.iter().map(|r| train_replies.contains(&census_hash(reply(r)))).collect(),
    })
}

/// One seed's picks at a split's openings: the trunk keys' names and distances, B (the trunk alone,
/// memory off) and the served model (memory on), each checked against the split's openings.
struct SeedPicks {
    keys: OpeningKeys,
    base: Vec<u32>,
    served: Vec<u32>,
}

impl SeedPicks {
    fn key(&self, key: &str) -> (&[u32], &[f32]) {
        self.keys.by_key(key)
    }
}

/// Whether two keys files name the same openings by the same point and sign keys: positions,
/// targets, and every point and sign field. The follow-up pass's files must equal the main run's
/// so (Casey's amendment, FIT round 5): its second binary reproduced the first.
fn same_trunk_keys(a: &OpeningKeys, b: &OpeningKeys) -> bool {
    a.positions == b.positions
        && a.targets == b.targets
        && a.point_target == b.point_target
        && a.point_distance == b.point_distance
        && a.point_ties == b.point_ties
        && a.sign_target == b.sign_target
        && a.sign_distance == b.sign_distance
        && a.sign_ties == b.sign_ties
}

/// One seed's picks on a split: the rows from the main run in `runs`, and the keys from
/// `keys_runs` (the follow-up pass, `--keys-runs`) when given, refused unless their point and
/// sign keys equal the main run's ([`same_trunk_keys`]).
fn seed_picks(runs: &Path, keys_runs: Option<&Path>, seed: u8, openings: &GradeOpenings) -> Result<SeedPicks> {
    let dir = runs.join(format!("r-drop-s{seed}"));
    let split = &openings.name;
    let file = format!("keys-{split}.json");
    let keys: OpeningKeys = match keys_runs {
        Some(follow) => {
            let keys: OpeningKeys = read_scored(&follow.join(format!("r-drop-s{seed}")).join(&file))?;
            let main: OpeningKeys = read_scored(&dir.join(&file))?;
            if !same_trunk_keys(&keys, &main) {
                return Err(invalid(format!(
                    "{split} s{seed}: the follow-up pass's point or sign keys differ from the main run's (an instrument fault)"
                )));
            }
            keys
        }
        None => read_scored(&dir.join(&file))?,
    };
    if keys.positions != openings.positions || keys.targets != openings.targets {
        return Err(invalid(format!("{split} s{seed}: the keys' openings are not the split's (an instrument fault)")));
    }
    if keys.hsign_target.len() != keys.positions.len() || keys.hsign_distance.len() != keys.positions.len() {
        return Err(invalid(format!(
            "{split} s{seed}: the keys file has no Hadamard sign key (grade the follow-up pass's files)"
        )));
    }
    let at = |label: &str| -> Result<Vec<u32>> {
        let rows: ScoredRows = read_scored(&dir.join(format!("rows-{split}-{label}.json")))?;
        let by: std::collections::HashMap<usize, u32> =
            rows.positions.iter().copied().zip(rows.nearest.iter().copied()).collect();
        openings
            .positions
            .iter()
            .map(|p| by.get(p).copied().ok_or_else(|| invalid(format!("{split} s{seed}: no {label} row at opening {p}"))))
            .collect()
    };
    let (base, served) = (at("no-memory")?, at("memory")?);
    // The memory moves nothing where its key misses, so B is what the served model sees there.
    if openings.exact.iter().zip(base.iter().zip(&served)).any(|(&(_, fired), (b, s))| !fired && b != s) {
        return Err(invalid(format!(
            "{split} s{seed}: the memory moved an opening its key misses (an instrument fault)"
        )));
    }
    Ok(SeedPicks { keys, base, served })
}

/// Level ℓ's threshold over `distances` (ℓ = 1–10, q10 to q100): the ⌈ℓ·n/10⌉-th smallest; q100
/// is +∞, so it always fires.
fn level_tau(distances: &[f32], level: u8) -> f32 {
    if level >= 10 || distances.is_empty() {
        return f32::INFINITY;
    }
    let mut d = distances.to_vec();
    d.sort_by(|a, b| a.total_cmp(b));
    d[(usize::from(level) * d.len()).div_ceil(10).max(1) - 1]
}

/// One trunk key's reading on some of a split's openings at one seed (claude-218): where its
/// distance is at most τ it names the opening, elsewhere `base` does. `b` counts rows it gets
/// right and the base wrong, `c` the reverse; the hits where it fires are kept for it and the base.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct KeyReading {
    pub split: String,
    pub key: String,
    /// `missed` (the exact key does not fire), `new` (missed, reference not verbatim TRAIN),
    /// `q100` (missed, every row firing) or `dev-backoff` (all DEV openings, the served model).
    pub stratum: String,
    pub seed: u8,
    pub level: u8,
    pub tau: f32,
    pub rows: usize,
    pub fired: usize,
    pub b: usize,
    pub c: usize,
    pub fired_hits: usize,
    pub base_fired_hits: usize,
}

impl KeyReading {
    fn net(&self) -> i64 {
        self.b as i64 - self.c as i64
    }

    fn band(&self) -> f64 {
        2.0 * ((self.b + self.c) as f64).sqrt()
    }
}

/// [`KeyReading`] on the openings `rows` indexes, against `base`; a row `fires_ok` excludes never
/// fires (the backoff: the key acts only where the exact key misses).
#[allow(clippy::too_many_arguments)]
fn key_reading(
    head: (&str, &str, &str, u8, u8, f32),
    rows: &[usize],
    targets: &[u32],
    named: &[u32],
    distance: &[f32],
    base: &[u32],
    fires_ok: &dyn Fn(usize) -> bool,
) -> KeyReading {
    let (split, key, stratum, seed, level, tau) = head;
    let mut r = KeyReading {
        split: split.to_owned(),
        key: key.to_owned(),
        stratum: stratum.to_owned(),
        seed,
        level,
        tau,
        rows: 0,
        fired: 0,
        b: 0,
        c: 0,
        fired_hits: 0,
        base_fired_hits: 0,
    };
    for &i in rows {
        let fired = fires_ok(i) && distance[i] <= tau;
        let arm = if fired { named[i] } else { base[i] };
        let (hit, base_hit) = (arm == targets[i], base[i] == targets[i]);
        r.rows += 1;
        r.fired += usize::from(fired);
        r.b += usize::from(hit && !base_hit);
        r.c += usize::from(!hit && base_hit);
        if fired {
            r.fired_hits += usize::from(hit);
            r.base_fired_hits += usize::from(base_hit);
        }
    }
    r
}

/// The level with the largest net gain summed over seeds, `net[seed][level − 1]`; a tie keeps
/// the lower level.
fn choose_level(net: &[[i64; 10]]) -> u8 {
    let total = |l: usize| net.iter().map(|s| s[l]).sum::<i64>();
    let mut best = 0;
    for l in 1..10 {
        if total(l) > total(best) {
            best = l;
        }
    }
    best as u8 + 1
}

/// The forced-opening arm against greedy at one seed (claude-218's row 1), on the same prompts:
/// paired counts (b: forced right and greedy wrong; c: the reverse) for replies equal to their
/// reference, and for replies whose value-masked template equals the reference's.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ForcedReading {
    pub split: String,
    pub seed: u8,
    pub replies: usize,
    pub exact: (usize, usize),
    pub template: (usize, usize),
}

impl ForcedReading {
    /// Row 1 at this seed: the lift is inside its band on both measures, and the reading is not
    /// underpowered (b + c reaches 10 on at least one).
    fn no_lift(&self) -> bool {
        let inside = |(b, c): (usize, usize)| (b as f64 - c as f64) <= 2.0 * ((b + c) as f64).sqrt();
        let powered = self.exact.0 + self.exact.1 >= 10 || self.template.0 + self.template.1 >= 10;
        powered && inside(self.exact) && inside(self.template)
    }
}

#[allow(clippy::too_many_arguments)]
fn forced_reading(
    split: &str,
    seed: u8,
    greedy: &[IdSample],
    forced: &[IdSample],
    eos: u32,
    keep: &std::collections::HashSet<u32>,
    digits: &std::collections::HashSet<u32>,
) -> Result<ForcedReading> {
    if greedy.len() != forced.len()
        || greedy.iter().zip(forced).any(|(g, f)| g.position != f.position || g.reference != f.reference)
    {
        return Err(invalid(format!("{split} s{seed}: the forced arm's prompts are not greedy's")));
    }
    if forced.iter().any(|f| f.reply.first() != f.reference.first()) {
        return Err(invalid(format!(
            "{split} s{seed}: a forced reply does not open with its reference's first token (an instrument fault)"
        )));
    }
    let exact = |s: &IdSample| cut_at_eos(&s.reply, eos) == cut_at_eos(&s.reference, eos);
    let template = |s: &IdSample| {
        value_mask(cut_at_eos(&s.reply, eos), keep, digits) == value_mask(cut_at_eos(&s.reference, eos), keep, digits)
    };
    let paired = |right: &dyn Fn(&IdSample) -> bool| {
        greedy.iter().zip(forced).fold((0, 0), |(b, c), (g, f)| {
            (b + usize::from(right(f) && !right(g)), c + usize::from(!right(f) && right(g)))
        })
    };
    Ok(ForcedReading {
        split: split.to_owned(),
        seed,
        replies: forced.len(),
        exact: paired(&exact),
        template: paired(&template),
    })
}

/// The decision table of claude-218's sheet, read in order on the deciding panel: the row that
/// holds (1–5) and the next unit it names, with the key that ships on row 3.
fn key_outcome(
    forced: &[ForcedReading],
    oracle: &[(usize, usize)],
    missed: &[(&str, Vec<KeyReading>)],
    new: &[(&str, Vec<KeyReading>)],
    dev: &[(&str, Vec<KeyReading>)],
    q100: &[(&str, Vec<KeyReading>)],
) -> (u8, &'static str, Option<String>) {
    let all3 = |r: &[KeyReading], f: &dyn Fn(&KeyReading) -> bool| r.len() == 3 && r.iter().all(f);
    if forced.len() == 3 && forced.iter().all(ForcedReading::no_lift) {
        return (1, "the opening is not enough: the rollout hinge, or a combined unit", None);
    }
    // A reading with b + c under 10 holds no row (FIT round 3): too thin to tell.
    let powered = |b: usize, c: usize| b + c >= 10;
    if oracle.len() == 3
        && oracle
            .iter()
            .all(|&(b, c)| powered(b, c) && (b as f64 - c as f64).abs() <= 2.0 * ((b + c) as f64).sqrt())
    {
        return (2, "no prompt key has headroom: the rollout hinge and the later rows", None);
    }
    let of = |set: &[(&str, Vec<KeyReading>)], key: &str| -> Vec<KeyReading> {
        set.iter().find(|(k, _)| *k == key).map(|(_, r)| r.clone()).unwrap_or_default()
    };
    for key in TRUNK_KEYS {
        let passes = all3(&of(missed, key), &|r| {
            powered(r.b, r.c) && r.net() as f64 > r.band() && r.fired_hits >= r.base_fired_hits
        })
            && all3(&of(new, key), &|r| r.net() as f64 >= -r.band())
            && all3(&of(dev, key), &|r| r.net() as f64 >= -r.band());
        if passes {
            return (3, "train and serve the key, with an opening bar on another fresh panel", Some(key.to_owned()));
        }
    }
    if TRUNK_KEYS
        .iter()
        .any(|&key| of(q100, key).iter().any(|r| powered(r.b, r.c) && (r.net() as f64) < -r.band()))
    {
        return (4, "an abstention gate", None);
    }
    (5, "train the trunk's opening point to gather paraphrases, with a geometric hinge", None)
}

/// A deciding grade's given level for `key` must be p-s's own `chosen` one (the code review), so
/// p-u is never graded at a level the rule did not pick.
fn level_given(levels: &[(String, u8)], key: &str, chosen: u8) -> Result<()> {
    match levels.iter().find(|(n, _)| n == key) {
        Some(&(_, l)) if l == chosen => Ok(()),
        _ => Err(invalid(format!("--key-levels: {key} is not at p-s's chosen level {chosen}"))),
    }
}

/// A split's missed rows (the exact key does not fire), the new stratum among them (the
/// reference not a verbatim TRAIN reply) and the known stratum, as indices into its openings.
fn key_strata(o: &GradeOpenings) -> (Vec<usize>, Vec<usize>, Vec<usize>) {
    let missed: Vec<usize> = (0..o.positions.len()).filter(|&i| !o.exact[i].1).collect();
    let (known, new): (Vec<usize>, Vec<usize>) = missed.iter().partition(|&&i| o.known[i]);
    (missed, new, known)
}

/// Row 2's paired counts at one seed: U against the larger, in hits on all openings, of the
/// exact key (with its fallback) and B; a tie takes the exact key. b: U right and that arm
/// wrong; c: the reverse.
fn oracle_reading(o: &GradeOpenings, base: &[u32]) -> (usize, usize) {
    let hits = |named: &dyn Fn(usize) -> u32| (0..o.targets.len()).filter(|&i| named(i) == o.targets[i]).count();
    let exact = |i: usize| o.exact[i].0;
    let by_base = |i: usize| base[i];
    let arm: &dyn Fn(usize) -> u32 = if hits(&exact) >= hits(&by_base) { &exact } else { &by_base };
    (0..o.targets.len()).fold((0, 0), |(b, c), i| {
        let (u, a) = (o.oracle[i] == o.targets[i], arm(i) == o.targets[i]);
        (b + usize::from(u && !a), c + usize::from(!u && a))
    })
}

/// The key grade (claude-218): on the development panel p-s, each trunk key's level and every
/// seed's τ, with row 1's power there; and, given the levels, the decision on the deciding panel
/// p-u, with DEV's backoff. One JSON value.
#[derive(Clone, Debug, Serialize)]
pub struct KeyGrade {
    pub levels: Vec<(String, u8)>,
    pub taus: Vec<(String, u8, f32)>,
    /// Each key's net gain on p-s's missed rows at every level, summed over the seeds.
    pub net_by_level: Vec<(String, Vec<i64>)>,
    pub development_forced: Vec<ForcedReading>,
    pub readings: Vec<KeyReading>,
    pub forced: Vec<ForcedReading>,
    /// Per seed on p-u, U against the larger of the exact key and B: (b, c).
    pub oracle: Vec<(usize, usize)>,
    pub row: Option<u8>,
    pub next: Option<String>,
    pub ships: Option<String>,
}

/// [`KeyGrade`] from the sealed run in `runs` (`r-drop-s{1,2,3}`), on the laptop. With `levels`
/// empty it reads p-s only and chooses them; with them given it also grades p-u, which it never
/// reads otherwise.
pub fn key_grade(
    data_root: &Path,
    runs: &Path,
    keys_runs: Option<&Path>,
    panels: &[(String, PathBuf)],
    levels: &[(String, u8)],
) -> Result<KeyGrade> {
    let bos = hs::SHEET_BOS_ID;
    let deciding = !levels.is_empty();
    let panel = |name: &str| -> Result<(Vec<u32>, Vec<u8>)> {
        let (_, want, want_mask) =
            KEY_PANELS.iter().find(|(n, _, _)| *n == name).ok_or_else(|| invalid("not a key panel"))?;
        let dir = &panels
            .iter()
            .find(|(n, _)| n == name)
            .ok_or_else(|| invalid(format!("--key-grade needs --panel {name}=DIR")))?
            .1;
        let (tokens, mask, sha) = load_unpinned_uort(dir)?;
        let mask_sha = crate::sha256_bytes(&mask);
        if sha != *want || mask_sha != *want_mask {
            return Err(invalid(format!(
                "--panel {name}: tokens sha256 {sha} and mask {mask_sha}, not the sheet's {want} and {want_mask}"
            )));
        }
        Ok((tokens, mask))
    };
    if deciding && (levels.len() != TRUNK_KEYS.len() || TRUNK_KEYS.iter().any(|k| !levels.iter().any(|(n, l)| n == k && (1..=10).contains(l)))) {
        return Err(invalid("--key-levels wants sign=L,hsign=L,point=L with each L in 1–10"));
    }
    let tokenizer = pinned_tokenizer(data_root)?;
    let eos = tokenizer.token_id("<|eos|>").ok_or_else(|| invalid("the pinned tokenizer has no <|eos|>"))?;
    let digits = digit_tokens(&fs::read_to_string(data_root.join("tokenizer.json"))?)?;
    let (train_tokens, train_mask) = load_train_split(data_root)?;
    let keep = top_tokens(&train_tokens, OPENING_TEMPLATE_K, bos);
    let (train_windows, _) = hs::build_document_windows(&train_tokens, &train_mask, SHEET_CONTEXT, bos)?;
    let train_rows = opening_rows(&train_windows, &train_mask);
    let exact = KeyTable::new(train_rows.iter().map(|r| (PromptKey::First(8).of(&r.prompt), r.target)));
    let oracle = KeyTable::new(
        train_rows
            .iter()
            .map(|r| (value_mask(reply_at(&train_tokens, &train_mask, r.position), &keep, &digits), r.target)),
    );
    let train_replies: std::collections::HashSet<u128> =
        split_turns(&train_tokens, &train_mask, bos).replies.iter().map(|r| census_hash(r)).collect();
    drop((train_rows, train_windows));
    let openings_of = |name: &str, tokens: &[u32], mask: &[u8]| {
        grade_openings(name, tokens, mask, &exact, &oracle, &keep, &digits, &train_replies)
    };
    let samples = |seed: u8, split: &str, arm: &str| -> Result<Vec<IdSample>> {
        let file = match (split, arm) {
            ("dev", "greedy") => "samples-memory.json".to_owned(),
            (split, arm) => format!("samples-{split}-{arm}-memory.json"),
        };
        read_scored(&runs.join(format!("r-drop-s{seed}")).join(file))
    };
    // The development panel: each key's net gain at every level on its missed rows, and τ.
    let (ps_tokens, ps_mask) = panel("p-s")?;
    let ps = openings_of("p-s", &ps_tokens, &ps_mask)?;
    let (ps_missed, _, _) = key_strata(&ps);
    let ps_picks: Vec<SeedPicks> = (1..=3u8).map(|s| seed_picks(runs, keys_runs, s, &ps)).collect::<Result<_>>()?;
    let mut net_by_level = Vec::new();
    let mut taus = Vec::new();
    let mut chosen = Vec::new();
    for key in TRUNK_KEYS {
        let mut net = Vec::new();
        for (s, picks) in ps_picks.iter().enumerate() {
            let (named, distance) = picks.key(key);
            let missed_d: Vec<f32> = ps_missed.iter().map(|&i| distance[i]).collect();
            let mut by_level = [0i64; 10];
            for level in 1..=10u8 {
                let tau = level_tau(&missed_d, level);
                let head = ("p-s", key, "missed", s as u8 + 1, level, tau);
                by_level[usize::from(level) - 1] =
                    key_reading(head, &ps_missed, &ps.targets, named, distance, &picks.base, &|_| true).net();
            }
            net.push(by_level);
        }
        let level = choose_level(&net);
        if deciding {
            level_given(levels, key, level)?;
        }
        for (s, picks) in ps_picks.iter().enumerate() {
            let missed_d: Vec<f32> = ps_missed.iter().map(|&i| picks.key(key).1[i]).collect();
            taus.push((key.to_owned(), s as u8 + 1, level_tau(&missed_d, level)));
        }
        net_by_level.push((key.to_owned(), (0..10).map(|l| net.iter().map(|n| n[l]).sum()).collect()));
        chosen.push((key.to_owned(), level));
    }
    let forced_at = |split: &str, s: u8| -> Result<ForcedReading> {
        let f = forced_reading(split, s, &samples(s, split, "greedy")?, &samples(s, split, "forced")?, eos, &keep, &digits)?;
        if f.replies != KEY_FORCED_REPLIES {
            return Err(invalid(format!("{split} s{s}: {} forced replies, not {KEY_FORCED_REPLIES}", f.replies)));
        }
        Ok(f)
    };
    let development_forced = (1..=3u8).map(|s| forced_at("p-s", s)).collect::<Result<Vec<_>>>()?;
    let mut grade = KeyGrade {
        levels: chosen.clone(),
        taus: taus.clone(),
        net_by_level,
        development_forced,
        readings: Vec::new(),
        forced: Vec::new(),
        oracle: Vec::new(),
        row: None,
        next: None,
        ships: None,
    };
    if !deciding {
        return Ok(grade);
    }
    // The deciding panel and DEV, at each seed's τ.
    let (pu_tokens, pu_mask) = panel("p-u")?;
    let pu = openings_of("p-u", &pu_tokens, &pu_mask)?;
    let (dev_tokens, dev_mask) = hs::load_pinned_uort_split(
        &data_root.join("ft/dev/tokens.u16"),
        &data_root.join("ft/dev/response_mask.u8"),
        hs::PINNED_DEV_TOKENS_SHA256,
        hs::PINNED_DEV_MASK_SHA256,
        hs::EXPECTED_DEV_TOKENS,
        hs::EXPECTED_DEV_ELIGIBLE_TARGETS,
        hs::SHEET_VOCAB,
        bos,
    )?;
    let dev = openings_of("dev", &dev_tokens, &dev_mask)?;
    let (missed, new, known) = key_strata(&pu);
    let all_dev: Vec<usize> = (0..dev.positions.len()).collect();
    let pu_picks: Vec<SeedPicks> = (1..=3u8).map(|s| seed_picks(runs, keys_runs, s, &pu)).collect::<Result<_>>()?;
    let dev_picks: Vec<SeedPicks> = (1..=3u8).map(|s| seed_picks(runs, keys_runs, s, &dev)).collect::<Result<_>>()?;
    // Each seed's τ at a level, from that seed's own p-s missed rows.
    let tau_at = |key: &str, s: usize, level: u8| {
        let missed_d: Vec<f32> = ps_missed.iter().map(|&i| ps_picks[s].key(key).1[i]).collect();
        level_tau(&missed_d, level)
    };
    let dev_missed = |i: usize| !dev.exact[i].1;
    let (mut by_missed, mut by_new, mut by_dev, mut by_q100) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for key in TRUNK_KEYS {
        let level = chosen.iter().find(|(n, _)| n == key).map(|&(_, l)| l).unwrap_or(10);
        let (mut m, mut n, mut d, mut q) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for (s, (picks, dp)) in pu_picks.iter().zip(&dev_picks).enumerate() {
            let (seed, tau) = (s as u8 + 1, tau_at(key, s, level));
            let (named, distance) = picks.key(key);
            let (dev_named, dev_distance) = dp.key(key);
            let on_pu = |stratum: &str, rows: &[usize], level: u8, tau: f32| {
                key_reading(("p-u", key, stratum, seed, level, tau), rows, &pu.targets, named, distance, &picks.base, &|_| true)
            };
            let on_dev = |stratum: &str, level: u8, tau: f32| {
                key_reading(("dev", key, stratum, seed, level, tau), &all_dev, &dev.targets, dev_named, dev_distance, &dp.served, &dev_missed)
            };
            m.push(on_pu("missed", &missed, level, tau));
            n.push(on_pu("new", &new, level, tau));
            q.push(on_pu("q100", &missed, 10, f32::INFINITY));
            d.push(on_dev("dev-backoff", level, tau));
            // Reported, deciding nothing: the known stratum, and every decile on p-u and DEV.
            grade.readings.push(on_pu("known", &known, level, tau));
            for l in 1..=10u8 {
                grade.readings.push(on_pu("missed-decile", &missed, l, tau_at(key, s, l)));
                grade.readings.push(on_dev("dev-decile", l, tau_at(key, s, l)));
            }
        }
        grade.readings.extend(m.iter().chain(&n).chain(&q).chain(&d).cloned());
        by_missed.push((key, m));
        by_new.push((key, n));
        by_dev.push((key, d));
        by_q100.push((key, q));
    }
    for (s, picks) in pu_picks.iter().enumerate() {
        grade.forced.push(forced_at("p-u", s as u8 + 1)?);
        grade.oracle.push(oracle_reading(&pu, &picks.base));
    }
    let (row, next, ships) = key_outcome(&grade.forced, &grade.oracle, &by_missed, &by_new, &by_dev, &by_q100);
    grade.row = Some(row);
    grade.next = Some(next.to_owned());
    grade.ships = ships;
    Ok(grade)
}

/// The panels the keyed grade reads (claude-219's sheet), with their token and mask sha256s:
/// p-u, claude-218's deciding panel, for the power check from its sealed files, and p-v, the
/// fresh panel (seed 2111) that decides.
const KEYED_PANELS: [(&str, &str, &str); 2] = [
    KEY_PANELS[1],
    (
        "p-v",
        "3488eef5bba82c3d3b97633fbd92f568c237636e6bb44d9decbdb8886d4196f0",
        "c170fd0ac9568f98fa3e2ab5f22af0fe31b94ea14bbbf802daf446e20d5ca023",
    ),
];

/// Each trunk key's τ at seeds 1–3: claude-218's levels (sign 2, hsign 1, point 10), chosen on
/// p-s and committed in its `grade-p-s-levels.json`. The keyed arms decode at these, and the
/// keyed grade checks every keyed token against them.
const KEYED_TAUS: [(&str, [f32; 3]); 3] = [
    ("sign", [0.0078125, 0.01953125, 0.0078125]),
    ("hsign", [0.0, 0.0, 0.0]),
    ("point", [f32::INFINITY; 3]),
];

/// What a keyed reply must be, from the greedy and forced replies to the same prompt: with no
/// keyed token, greedy's; with the reference's first token, the forced arm's (the same decode);
/// with greedy's own first token, greedy's (at a missed opening the memory moves nothing, so
/// greedy's first step is that token); with any other, only that it opens with it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyedReply {
    Greedy,
    Forced,
    Novel(u32),
}

fn keyed_reply(keyed: Option<u32>, greedy: &IdSample, forced: &IdSample) -> KeyedReply {
    match keyed {
        None => KeyedReply::Greedy,
        Some(t) if forced.reference.first() == Some(&t) => KeyedReply::Forced,
        Some(t) if greedy.reply.first() == Some(&t) => KeyedReply::Greedy,
        Some(t) => KeyedReply::Novel(t),
    }
}

/// One keyed arm against greedy (memory on) at one seed on one split (claude-219), on the same
/// prompts. Paired counts, b the arm right and greedy wrong and c the reverse: exact replies,
/// value-masked templates, and loops (b: the arm's reply loops and greedy's does not). `keyed` is
/// how many replies the key opened and `right` how many of those with the reference's first
/// token. `predicted_exact` is the exact count [`keyed_reply`] gives from the greedy and forced
/// replies alone, which an observed arm must equal.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct KeyedReading {
    pub split: String,
    pub key: String,
    pub seed: u8,
    pub tau: f32,
    pub replies: usize,
    pub keyed: usize,
    pub right: usize,
    pub predicted_exact: (usize, usize),
    /// None where the arm's file is absent: the power check on p-u, from claude-218's files.
    pub exact: Option<(usize, usize)>,
    pub template: Option<(usize, usize)>,
    pub loops: Option<(usize, usize)>,
}

impl KeyedReading {
    /// The sheet's bar at this seed, with `dev` the same key's reading on DEV: templates beyond
    /// their band (b − c > 2·√(b + c), with b + c ≥ 10), and exact replies, loops and DEV's
    /// templates not worse beyond theirs.
    fn carries(&self, dev: &KeyedReading) -> bool {
        let band = |(b, c): (usize, usize)| 2.0 * ((b + c) as f64).sqrt();
        let net = |(b, c): (usize, usize)| b as f64 - c as f64;
        match self.template {
            Some(t) => t.0 + t.1 >= 10 && net(t) > band(t) && self.harmless(dev),
            None => false,
        }
    }

    /// Bars (b)–(d) alone: exact replies and DEV's templates not lower beyond their bands, and
    /// loops not higher, so a seed that fails to carry fails on (a)'s templates alone.
    fn harmless(&self, dev: &KeyedReading) -> bool {
        let band = |(b, c): (usize, usize)| 2.0 * ((b + c) as f64).sqrt();
        let net = |(b, c): (usize, usize)| b as f64 - c as f64;
        match (self.exact, self.loops, dev.template) {
            (Some(e), Some(l), Some(d)) => net(e) >= -band(e) && net(l) <= band(l) && net(d) >= -band(d),
            _ => false,
        }
    }

    /// Whether this seed's template reading has b + c under 10: too thin to tell.
    fn thin(&self) -> bool {
        self.template.is_none_or(|(b, c)| b + c < 10)
    }
}

/// [`KeyedReading`] for the keyed tokens `keyed_opening` gives at `tau` on `openings` (with the
/// laptop's exact key, which the turn anchor makes the served one) from `picks`' keys; `arm` is
/// the keyed arm's samples, when its file exists. A keyed token, or a reply, that is not what
/// [`keyed_reply`] says is an instrument fault.
#[allow(clippy::too_many_arguments)]
fn keyed_reading(
    head: (&str, &str, u8, f32),
    openings: &GradeOpenings,
    picks: &SeedPicks,
    greedy: &[IdSample],
    forced: &[IdSample],
    arm: Option<&[IdSample]>,
    eos: u32,
    keep: &std::collections::HashSet<u32>,
    digits: &std::collections::HashSet<u32>,
) -> Result<KeyedReading> {
    let (split, key, seed, tau) = head;
    let fault = |what: &str| invalid(format!("{split} s{seed} keyed-{key}: {what} (an instrument fault)"));
    let same = |a: &[IdSample]| {
        a.len() == greedy.len() && a.iter().zip(greedy).all(|(x, g)| x.position == g.position && x.reference == g.reference)
    };
    if !same(forced) || arm.is_some_and(|a| !same(a)) {
        return Err(fault("an arm's prompts are not greedy's"));
    }
    let at: std::collections::HashMap<usize, usize> =
        openings.positions.iter().enumerate().map(|(i, &p)| (p, i)).collect();
    let (names, distances) = picks.key(key);
    let exact = |reply: &[u32], s: &IdSample| cut_at_eos(reply, eos) == cut_at_eos(&s.reference, eos);
    let template = |reply: &[u32], s: &IdSample| {
        value_mask(cut_at_eos(reply, eos), keep, digits) == value_mask(cut_at_eos(&s.reference, eos), keep, digits)
    };
    let pair = |(b, c): (usize, usize), arm: bool, base: bool| (b + usize::from(arm && !base), c + usize::from(!arm && base));
    let mut r = KeyedReading {
        split: split.to_owned(),
        key: key.to_owned(),
        seed,
        tau,
        replies: greedy.len(),
        keyed: 0,
        right: 0,
        predicted_exact: (0, 0),
        exact: arm.map(|_| (0, 0)),
        template: arm.map(|_| (0, 0)),
        loops: arm.map(|_| (0, 0)),
    };
    for (j, (g, f)) in greedy.iter().zip(forced).enumerate() {
        let i = *at.get(&g.position).ok_or_else(|| fault(&format!("no opening at the sampled position {}", g.position)))?;
        let token = keyed_opening(openings.exact[i].1, names[i], distances[i], tau);
        let want = keyed_reply(token, g, f);
        r.keyed += usize::from(token.is_some());
        r.right += usize::from(want == KeyedReply::Forced);
        let predicted = match want {
            KeyedReply::Greedy => exact(&g.reply, g),
            KeyedReply::Forced => exact(&f.reply, f),
            KeyedReply::Novel(_) => false,
        };
        r.predicted_exact = pair(r.predicted_exact, predicted, exact(&g.reply, g));
        let Some(k) = arm.map(|a| &a[j]) else { continue };
        if k.keyed != token {
            return Err(fault(&format!("the keyed token at {} is not the registered τ's", g.position)));
        }
        let ok = match want {
            KeyedReply::Greedy => k.reply == g.reply,
            // The forced reply opens with the keyed token too, on DEV as on the panel (the code
            // review), so a forcing both arms ignore is refused.
            KeyedReply::Forced => k.reply == f.reply && f.reply.first() == token.as_ref(),
            // A key's name is a TRAIN reply's first token, never BOS (FIT round 2).
            KeyedReply::Novel(t) => t != hs::SHEET_BOS_ID && k.reply.first() == Some(&t),
        };
        if !ok {
            return Err(fault(&format!("the keyed reply at {} is not the one its opening fixes", g.position)));
        }
        r.exact = r.exact.map(|e| pair(e, exact(&k.reply, k), exact(&g.reply, g)));
        r.template = r.template.map(|t| pair(t, template(&k.reply, k), template(&g.reply, g)));
        r.loops = r.loops.map(|l| pair(l, loops(&k.reply, eos), loops(&g.reply, eos)));
    }
    if r.exact.is_some_and(|e| e != r.predicted_exact) {
        return Err(fault("the observed exact replies are not the predicted ones"));
    }
    Ok(r)
}

/// The decision table of claude-219's sheet, read in order on the deciding panel from the
/// forced readings there and each key's `(panel, dev)` readings at seeds 1–3: the row (1–4), the
/// next unit, and the key that ships on row 2.
fn keyed_outcome(
    forced: &[ForcedReading],
    readings: &[(&str, Vec<(KeyedReading, KeyedReading)>)],
) -> (u8, &'static str, Option<String>) {
    // A right first token lifted exact replies by 49–59 of 256 at every seed on p-s and p-u, so a
    // seed whose forced arm does not lift is an instrument signal (FIT round 1), whatever the rest.
    let lifts = |f: &ForcedReading| {
        let (b, c) = f.exact;
        b + c >= 10 && b as f64 - c as f64 > 2.0 * ((b + c) as f64).sqrt()
    };
    if forced.len() != 3 || !forced.iter().all(lifts) {
        return (1, "an instrument check: claude-218's forced arm lifted 49–59 of 256 at every seed on p-s and p-u", None);
    }
    for (key, by_seed) in readings {
        if by_seed.len() == 3 && by_seed.iter().all(|(p, d)| p.carries(d)) {
            return (2, "train and serve the key's opening, and score the whole model end to end", Some((*key).to_owned()));
        }
    }
    if readings.iter().all(|(_, by_seed)| by_seed.iter().any(|(p, _)| p.thin())) {
        return (3, "underpowered: a larger panel, or the seeds pooled under a fresh sheet", None);
    }
    (4, "the key's opening does not carry: train the trunk's opening point to gather paraphrases", None)
}

/// The keyed grade (claude-219), on the laptop: on the panel (p-v decides; p-u gives the power
/// check from claude-218's files) and DEV, at seeds 1–3, the forced arm against greedy and each
/// keyed arm against greedy, with the sheet's decision when every keyed arm's file is there.
#[derive(Clone, Debug, Serialize)]
pub struct KeyedGrade {
    pub panel: String,
    pub taus: Vec<(String, [f32; 3])>,
    pub forced: Vec<ForcedReading>,
    pub readings: Vec<KeyedReading>,
    pub row: Option<u8>,
    pub next: Option<String>,
    pub ships: Option<String>,
}

/// [`KeyedGrade`] from the run in `runs` (`r-drop-s{1,2,3}`), with the keys from `keys_runs`
/// when given ([`seed_picks`]), on the one `panels` entry, p-u or p-v, checked by its sha256s.
pub fn keyed_grade(
    data_root: &Path,
    runs: &Path,
    keys_runs: Option<&Path>,
    panels: &[(String, PathBuf)],
) -> Result<KeyedGrade> {
    let bos = hs::SHEET_BOS_ID;
    let [(name, dir)] = panels else {
        return Err(invalid("--keyed-grade needs one --panel NAME=DIR, p-u or p-v"));
    };
    let (_, want, want_mask) = KEYED_PANELS
        .iter()
        .find(|(n, _, _)| n == name)
        .ok_or_else(|| invalid(format!("--keyed-grade reads p-u or p-v, not {name}")))?;
    let (tokens, mask, sha) = load_unpinned_uort(dir)?;
    let mask_sha = crate::sha256_bytes(&mask);
    if sha != *want || mask_sha != *want_mask {
        return Err(invalid(format!(
            "--panel {name}: tokens sha256 {sha} and mask {mask_sha}, not the sheet's {want} and {want_mask}"
        )));
    }
    let tokenizer = pinned_tokenizer(data_root)?;
    let eos = tokenizer.token_id("<|eos|>").ok_or_else(|| invalid("the pinned tokenizer has no <|eos|>"))?;
    let digits = digit_tokens(&fs::read_to_string(data_root.join("tokenizer.json"))?)?;
    let (train_tokens, train_mask) = load_train_split(data_root)?;
    let keep = top_tokens(&train_tokens, OPENING_TEMPLATE_K, bos);
    let (train_windows, _) = hs::build_document_windows(&train_tokens, &train_mask, SHEET_CONTEXT, bos)?;
    let train_rows = opening_rows(&train_windows, &train_mask);
    let exact = KeyTable::new(train_rows.iter().map(|r| (PromptKey::First(8).of(&r.prompt), r.target)));
    let oracle = KeyTable::new(
        train_rows
            .iter()
            .map(|r| (value_mask(reply_at(&train_tokens, &train_mask, r.position), &keep, &digits), r.target)),
    );
    let train_replies: std::collections::HashSet<u128> =
        split_turns(&train_tokens, &train_mask, bos).replies.iter().map(|r| census_hash(r)).collect();
    drop((train_rows, train_windows));
    let (dev_tokens, dev_mask) = hs::load_pinned_uort_split(
        &data_root.join("ft/dev/tokens.u16"),
        &data_root.join("ft/dev/response_mask.u8"),
        hs::PINNED_DEV_TOKENS_SHA256,
        hs::PINNED_DEV_MASK_SHA256,
        hs::EXPECTED_DEV_TOKENS,
        hs::EXPECTED_DEV_ELIGIBLE_TARGETS,
        hs::SHEET_VOCAB,
        bos,
    )?;
    let splits = [
        grade_openings(name, &tokens, &mask, &exact, &oracle, &keep, &digits, &train_replies)?,
        grade_openings("dev", &dev_tokens, &dev_mask, &exact, &oracle, &keep, &digits, &train_replies)?,
    ];
    let samples = |seed: u8, split: &str, arm: &str| -> Result<Option<Vec<IdSample>>> {
        let file = match (split, arm) {
            ("dev", "greedy") => "samples-memory.json".to_owned(),
            (split, arm) => format!("samples-{split}-{arm}-memory.json"),
        };
        let path = runs.join(format!("r-drop-s{seed}")).join(file);
        if path.exists() { read_scored(&path).map(Some) } else { Ok(None) }
    };
    let mut grade = KeyedGrade {
        panel: name.clone(),
        taus: KEYED_TAUS.iter().map(|(k, t)| ((*k).to_owned(), *t)).collect(),
        forced: Vec::new(),
        readings: Vec::new(),
        row: None,
        next: None,
        ships: None,
    };
    let mut by_key: Vec<(&str, Vec<(KeyedReading, KeyedReading)>)> =
        KEYED_TAUS.iter().map(|(k, _)| (*k, Vec::new())).collect();
    for seed in 1..=3u8 {
        let mut pair: Vec<Vec<KeyedReading>> = vec![Vec::new(); KEYED_TAUS.len()];
        for openings in &splits {
            let split = openings.name.as_str();
            let picks = seed_picks(runs, keys_runs, seed, openings)?;
            let need = |arm: &str| -> Result<Vec<IdSample>> {
                samples(seed, split, arm)?.ok_or_else(|| invalid(format!("{split} s{seed}: no {arm} samples")))
            };
            let (greedy, forced) = (need("greedy")?, need("forced")?);
            if split == name {
                grade.forced.push(forced_reading(split, seed, &greedy, &forced, eos, &keep, &digits)?);
            }
            for (k, (key, taus)) in KEYED_TAUS.iter().enumerate() {
                let arm = samples(seed, split, &format!("keyed-{key}"))?;
                let head = (split, *key, seed, taus[usize::from(seed) - 1]);
                let reading = keyed_reading(head, openings, &picks, &greedy, &forced, arm.as_deref(), eos, &keep, &digits)?;
                pair[k].push(reading.clone());
                grade.readings.push(reading);
            }
        }
        for (k, mut readings) in pair.into_iter().enumerate() {
            let dev = readings.pop().ok_or_else(|| invalid("no DEV reading"))?;
            let panel = readings.pop().ok_or_else(|| invalid("no panel reading"))?;
            by_key[k].1.push((panel, dev));
        }
    }
    // The decision only on p-v, the deciding panel, and only when every keyed arm was decoded;
    // p-u's power check has none.
    if name == "p-v" && grade.readings.iter().all(|r| r.exact.is_some()) {
        let (row, next, ships) = keyed_outcome(&grade.forced, &by_key);
        grade.row = Some(row);
        grade.next = Some(next.to_owned());
        grade.ships = ships;
    }
    Ok(grade)
}

/// The panels the served grade reads (claude-220's sheet), with their token and mask sha256s:
/// p-v, claude-219's deciding panel, where the served key must give claude-219's keyed-point
/// replies, and p-w, the fresh panel (seed 2121, 1,200 conversations) that decides.
const SERVED_PANELS: [(&str, &str, &str); 2] = [
    KEYED_PANELS[1],
    (
        "p-w",
        "038b0085e6888d988d4b0b6ceb7846015f21f1c40f26ec3286ddcc049c17d217",
        "797fc23474231595ae42b9cfbf7930ae2021b13d23d8b8b3d4fb3c7dda06ec16",
    ),
];

/// The parity count of a served arm (claude-220's sheet, FIT round 1): of the replies the served
/// key opened, how many it opened with the run's keys-file name for the same opening; and, where
/// they agree, the largest gap between the served search's least distance and the keys file's,
/// 0 when the decode's point and the batched forward's are equal.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ServedParity {
    pub split: String,
    pub seed: u8,
    pub served: usize,
    pub agree: usize,
    pub max_drift: f32,
}

impl ServedParity {
    /// The sheet's rule: parity holds when at most 1% of the served openings disagree.
    fn holds(&self) -> bool {
        self.served > 0 && (self.served - self.agree) * 100 <= self.served
    }
}

/// [`ServedParity`] for `served` at `openings`, and `picks` with the point key's names and
/// distances replaced at each served opening by the served key's own, so that [`keyed_reading`]
/// reads the served arm by its own tokens (the sheet's rule (a)).
fn served_parity(
    split: &str,
    seed: u8,
    openings: &GradeOpenings,
    picks: &SeedPicks,
    served: &[IdSample],
) -> Result<(ServedParity, SeedPicks)> {
    let at: std::collections::HashMap<usize, usize> =
        openings.positions.iter().enumerate().map(|(i, &p)| (p, i)).collect();
    let mut keys = picks.keys.clone();
    let mut parity = ServedParity { split: split.to_owned(), seed, served: 0, agree: 0, max_drift: 0.0 };
    for s in served {
        let Some(token) = s.keyed else { continue };
        let i = *at.get(&s.position).ok_or_else(|| {
            invalid(format!("{split} s{seed}: a served opening at {} is not the split's (an instrument fault)", s.position))
        })?;
        let distance = s
            .served_distance
            .filter(|d| d.is_finite())
            .ok_or_else(|| invalid(format!("{split} s{seed}: a served token at {} has no finite distance", s.position)))?;
        parity.served += 1;
        // Against the keys file as read, never the copy this loop rewrites.
        if picks.keys.point_target[i] == token {
            parity.agree += 1;
            parity.max_drift = parity.max_drift.max((distance - picks.keys.point_distance[i]).abs());
        }
        keys.point_target[i] = token;
        keys.point_distance[i] = distance;
    }
    Ok((parity, SeedPicks { keys, base: picks.base.clone(), served: picks.served.clone() }))
}

/// The served sheet's row (claude-220): 1 where any split's parity fails ([`ServedParity::holds`]),
/// else [`keyed_outcome`]'s row for the served arm's `(p-w, DEV)` readings at seeds 1–3, except
/// that row 3 is read only where every seed is harmless on bars (b)–(d) ([`KeyedReading::harmless`]):
/// then a row 4 whose seeds pooled gain beyond their band on p-w's templates is row 3 (FIT rounds 1
/// and 2), and wherever a seed has a harm, a row 3 is row 4 (round 3).
fn served_outcome(
    forced: &[ForcedReading],
    by_seed: Vec<(KeyedReading, KeyedReading)>,
    parity: &[ServedParity],
) -> u8 {
    let (b, c) = by_seed.iter().fold((0, 0), |(b, c), (p, _)| {
        let (pb, pc) = p.template.unwrap_or((0, 0));
        (b + pb, c + pc)
    });
    let harmless = by_seed.iter().all(|(p, d)| p.harmless(d));
    let pooled_gain = b as f64 - c as f64 > 2.0 * ((b + c) as f64).sqrt() && harmless;
    match keyed_outcome(forced, &[("point", by_seed)]).0 {
        _ if !parity.iter().all(ServedParity::holds) => 1,
        4 if pooled_gain => 3,
        // A thin seed beside a harmed one is a harm, not a power shortfall (FIT round 3).
        3 if !harmless => 4,
        row => row,
    }
}

/// The served grade (claude-220), on the laptop: on p-w (deciding), p-v and DEV at seeds 1–3,
/// the forced arm against greedy on p-w; the served key's parity with the run's keys file
/// ([`served_parity`]); the served arm against greedy, read by [`keyed_reading`] by its own
/// tokens, which refuses any reply its opening does not fix; and the protocol's free-generation
/// measures ([`generation`]) for greedy and the served arm. With the sheet's decision
/// ([`served_outcome`]): parity or the forced arm failing is row 1, then [`keyed_outcome`]'s rows
/// on p-w, with row 3 only where every seed is harmless.
#[derive(Clone, Debug, Serialize)]
pub struct ServedGrade {
    pub parity: Vec<ServedParity>,
    pub forced: Vec<ForcedReading>,
    pub readings: Vec<KeyedReading>,
    /// `(split, seed, arm, measures)`, the arm `greedy` or `served`.
    pub generation: Vec<(String, u8, String, Generation)>,
    pub row: Option<u8>,
    pub next: Option<String>,
}

/// [`ServedGrade`] from the run in `runs` (`r-drop-s{1,2,3}`), on `panels`' p-w and p-v, each
/// checked by its sha256s.
pub fn served_grade(data_root: &Path, runs: &Path, panels: &[(String, PathBuf)]) -> Result<ServedGrade> {
    if let Some((name, _)) = panels.iter().find(|(n, _)| !SERVED_PANELS.iter().any(|(s, _, _)| *s == n.as_str())) {
        return Err(invalid(format!("--served-grade reads p-w and p-v only, not --panel {name}")));
    }
    let bos = hs::SHEET_BOS_ID;
    let panel = |name: &str| -> Result<(Vec<u32>, Vec<u8>)> {
        let (_, want, want_mask) =
            SERVED_PANELS.iter().find(|(n, _, _)| *n == name).ok_or_else(|| invalid("not a served panel"))?;
        let dir = &panels
            .iter()
            .find(|(n, _)| n == name)
            .ok_or_else(|| invalid(format!("--served-grade needs --panel {name}=DIR")))?
            .1;
        let (tokens, mask, sha) = load_unpinned_uort(dir)?;
        let mask_sha = crate::sha256_bytes(&mask);
        if sha != *want || mask_sha != *want_mask {
            return Err(invalid(format!(
                "--panel {name}: tokens sha256 {sha} and mask {mask_sha}, not the sheet's {want} and {want_mask}"
            )));
        }
        Ok((tokens, mask))
    };
    let tokenizer = pinned_tokenizer(data_root)?;
    let eos = tokenizer.token_id("<|eos|>").ok_or_else(|| invalid("the pinned tokenizer has no <|eos|>"))?;
    let digits = digit_tokens(&fs::read_to_string(data_root.join("tokenizer.json"))?)?;
    let (train_tokens, train_mask) = load_train_split(data_root)?;
    let keep = top_tokens(&train_tokens, OPENING_TEMPLATE_K, bos);
    let (train_windows, _) = hs::build_document_windows(&train_tokens, &train_mask, SHEET_CONTEXT, bos)?;
    let train_rows = opening_rows(&train_windows, &train_mask);
    let exact = KeyTable::new(train_rows.iter().map(|r| (PromptKey::First(8).of(&r.prompt), r.target)));
    let oracle = KeyTable::new(
        train_rows
            .iter()
            .map(|r| (value_mask(reply_at(&train_tokens, &train_mask, r.position), &keep, &digits), r.target)),
    );
    let train_turns = split_turns(&train_tokens, &train_mask, bos).replies;
    let train_replies: std::collections::HashSet<u128> = train_turns.iter().map(|r| census_hash(r)).collect();
    drop((train_rows, train_windows));
    let (dev_tokens, dev_mask) = hs::load_pinned_uort_split(
        &data_root.join("ft/dev/tokens.u16"),
        &data_root.join("ft/dev/response_mask.u8"),
        hs::PINNED_DEV_TOKENS_SHA256,
        hs::PINNED_DEV_MASK_SHA256,
        hs::EXPECTED_DEV_TOKENS,
        hs::EXPECTED_DEV_ELIGIBLE_TARGETS,
        hs::SHEET_VOCAB,
        bos,
    )?;
    let (pw_tokens, pw_mask) = panel("p-w")?;
    let (pv_tokens, pv_mask) = panel("p-v")?;
    let openings_of = |name: &str, tokens: &[u32], mask: &[u8]| {
        grade_openings(name, tokens, mask, &exact, &oracle, &keep, &digits, &train_replies)
    };
    let splits = [
        openings_of("p-w", &pw_tokens, &pw_mask)?,
        openings_of("p-v", &pv_tokens, &pv_mask)?,
        openings_of("dev", &dev_tokens, &dev_mask)?,
    ];
    let samples = |seed: u8, split: &str, arm: &str| -> Result<Vec<IdSample>> {
        let file = match (split, arm) {
            ("dev", "greedy") => "samples-memory.json".to_owned(),
            (split, arm) => format!("samples-{split}-{arm}-memory.json"),
        };
        read_scored(&runs.join(format!("r-drop-s{seed}")).join(file))
    };
    let mut grade = ServedGrade {
        parity: Vec::new(),
        forced: Vec::new(),
        readings: Vec::new(),
        generation: Vec::new(),
        row: None,
        next: None,
    };
    let mut by_seed = Vec::new();
    for seed in 1..=3u8 {
        let mut pw_dev: (Option<KeyedReading>, Option<KeyedReading>) = (None, None);
        for openings in &splits {
            let split = openings.name.as_str();
            let picks = seed_picks(runs, None, seed, openings)?;
            let (greedy, forced, served) = (samples(seed, split, "greedy")?, samples(seed, split, "forced")?, samples(seed, split, "served")?);
            if split == "p-w" {
                grade.forced.push(forced_reading(split, seed, &greedy, &forced, eos, &keep, &digits)?);
            }
            let (parity, own) = served_parity(split, seed, openings, &picks, &served)?;
            grade.parity.push(parity);
            let reading = keyed_reading(
                (split, "point", seed, f32::INFINITY),
                openings,
                &own,
                &greedy,
                &forced,
                Some(&served),
                eos,
                &keep,
                &digits,
            )?;
            for (arm, replies) in [("greedy", &greedy), ("served", &served)] {
                let replies: Vec<Vec<u32>> = replies.iter().map(|s| s.reply.clone()).collect();
                grade.generation.push((split.to_owned(), seed, arm.to_owned(), generation(&replies, &train_turns, eos)));
            }
            match split {
                "p-w" => pw_dev.0 = Some(reading.clone()),
                "dev" => pw_dev.1 = Some(reading.clone()),
                _ => {}
            }
            grade.readings.push(reading);
        }
        if let (Some(p), Some(d)) = pw_dev {
            by_seed.push((p, d));
        }
    }
    let row = served_outcome(&grade.forced, by_seed, &grade.parity);
    grade.row = Some(row);
    grade.next = Some(
        match row {
            1 => "an instrument check: the forced arm did not lift on p-w at some seed, or the served key's parity with the keys file failed",
            2 => "the one float model serves the point key; next, its integer export under its own item, then the integer model end to end",
            3 => "underpowered at some seed: a larger panel, or the seeds pooled under a fresh sheet",
            _ => "the served key does not carry on a second fresh panel; next, train the trunk's opening point to gather paraphrases",
        }
        .to_owned(),
    );
    Ok(grade)
}

/// One panel prompt for the integer decode (point-key-integer unit 2, claude-224).
/// - **Its fields:** the reply's corpus position, the prompt's ids, and the reference reply.
/// - **Its origin:** these are exactly what [`score_rows`] decodes and writes, and `fbs-toks --replies` reads them.
/// - **`float_keyed`** is the float side's served name at the opening, where a seed's served file gives one.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PanelPrompt {
    pub position: usize,
    pub prompt: Vec<u32>,
    pub reference: Vec<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub float_keyed: Option<u32>,
}

/// The prompts [`score_rows`] decodes on a split (claude-224):
/// - the windows are [`sample_windows`]';
/// - each prompt is its window's input through its first scored offset;
/// - each reference is its [`reference_reply`].
///
/// No model is involved.
fn split_prompts(windows: &[CausalWindow], mask: &[u8], samples: usize, reply_tokens: usize) -> Vec<PanelPrompt> {
    sample_windows(windows, mask, samples)
        .into_iter()
        .map(|window| PanelPrompt {
            position: window.scored_corpus_positions[0],
            prompt: window.input_ids[..=window.scored_offsets[0]].to_vec(),
            reference: reference_reply(window, reply_tokens),
            float_keyed: None,
        })
        .collect()
}

/// `--panel-prompts` (claude-224): DEV's and each panel's prompts, as [`score_rows`] decodes them,
/// written new to `out` as `prompts-{split}.json`.
///
/// With `served_runs` (claude-220's `out`), each `r-drop-s{seed}` there whose served file holds a
/// split also gets `prompts-{split}-s{seed}.json`, each prompt carrying that seed's float served
/// name. A served sample whose position, prompt or reference is not its prompt's is refused: the
/// float arms decoded these prompts, so the two lists are one.
///
/// It returns each written file's name and rows.
pub fn panel_prompts(
    data_root: &Path,
    panels: &[(String, PathBuf)],
    samples: usize,
    reply_tokens: usize,
    served_runs: Option<&Path>,
    out: &Path,
) -> Result<Vec<(String, usize)>> {
    let windows_of = |tokens: &[u32], mask: &[u8]| -> Result<Vec<CausalWindow>> {
        Ok(hs::build_document_windows(tokens, mask, SHEET_CONTEXT, hs::SHEET_BOS_ID)?.0)
    };
    let (dev_windows, dev_mask) = load_dev(data_root)?;
    let mut splits = vec![("dev".to_owned(), split_prompts(&dev_windows, &dev_mask, samples, reply_tokens))];
    for (name, dir) in panels {
        if !panel_name_ok(name) || splits.iter().any(|(n, _)| n == name) {
            return Err(invalid(format!("--panel {name}: a panel name is one word, not dev")));
        }
        let (tokens, mask, _) = load_unpinned_uort(dir)?;
        splits.push((name.clone(), split_prompts(&windows_of(&tokens, &mask)?, &mask, samples, reply_tokens)));
    }
    fs::create_dir_all(out)?;
    let mut written = Vec::new();
    for (split, prompts) in &splits {
        let file = format!("prompts-{split}.json");
        write_new(&out.join(&file), &serde_json::to_vec(prompts)?)?;
        written.push((file, prompts.len()));
        let Some(runs) = served_runs else { continue };
        let mut seeds = 0;
        for seed in 1..=3u8 {
            let path = runs.join(format!("r-drop-s{seed}")).join(format!("samples-{split}-served-memory.json"));
            if !path.exists() {
                continue;
            }
            let keyed = attach_float_names(prompts, &read_scored::<Vec<IdSample>>(&path)?)
                .map_err(|e| invalid(format!("{}: {e}", path.display())))?;
            let file = format!("prompts-{split}-s{seed}.json");
            write_new(&out.join(&file), &serde_json::to_vec(&keyed)?)?;
            written.push((file, keyed.len()));
            seeds += 1;
        }
        // A served run holds DEV at every seed: without it, `--runs` is not that run (a fresh
        // panel, which no float arm served, is written plain only).
        if split == "dev" && seeds != 3 {
            return Err(invalid(format!("--runs {}: DEV's served files are not there at seeds 1–3", runs.display())));
        }
    }
    Ok(written)
}

/// `prompts` with each one's float served name from `served`, a scored run's served arm on the
/// same split (claude-224). Refused unless it holds exactly these prompts, sample by sample: the
/// float arm decoded them.
fn attach_float_names(prompts: &[PanelPrompt], served: &[IdSample]) -> Result<Vec<PanelPrompt>> {
    if served.len() != prompts.len() {
        return Err(invalid(format!("{} samples, not the split's {} prompts", served.len(), prompts.len())));
    }
    prompts
        .iter()
        .zip(served)
        .map(|(p, s)| {
            if (p.position, &p.prompt, &p.reference) != (s.position, &s.prompt, &s.reference) {
                return Err(invalid(format!("the sample at {} is not the prompt's (an instrument fault)", s.position)));
            }
            Ok(PanelPrompt { float_keyed: s.keyed, ..p.clone() })
        })
        .collect()
}

/// One reply as `fbs-toks --replies` writes it (claude-224).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
struct IntegerSample {
    position: usize,
    prompt: Vec<u32>,
    reply: Vec<u32>,
    reference: Vec<u32>,
    keyed: Option<u32>,
    served_distance_q12: Option<i64>,
    float_keyed: Option<u32>,
    float_distance_q12: Option<i64>,
    end: String,
}

/// The item's rounding bound, at Q12 (done-when 3). Half a unit on each of 256 values of the query
/// and of a row moves their L1 sum by at most 256, and two rows can move apart, so an order that
/// rounding alone can flip lies within 512.
pub const POINT_KEY_ROUNDING_BOUND: i64 = 512;

/// Parity with the float key on one split and seed (done-when 3, reported, deciding nothing).
/// - **`served`:** the integer served openings.
/// - **`both`:** those the float side served too.
/// - **The split of `both`:**
///   - `agree`, the same name;
///   - `within`, a different name within [`POINT_KEY_ROUNDING_BOUND`] of the integer least distance;
///   - `drift`, every other one, a float name no row holds included.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct IntegerParity {
    pub split: String,
    pub seed: u8,
    pub served: usize,
    pub both: usize,
    pub agree: usize,
    pub within: usize,
    pub drift: usize,
}

/// [`IntegerParity`] of a served arm's samples.
fn integer_parity(split: &str, seed: u8, served: &[IntegerSample]) -> IntegerParity {
    let mut p = IntegerParity { split: split.to_owned(), seed, served: 0, both: 0, agree: 0, within: 0, drift: 0 };
    for s in served {
        let Some(name) = s.keyed else { continue };
        p.served += 1;
        let Some(float) = s.float_keyed else { continue };
        p.both += 1;
        if name == float {
            p.agree += 1;
        } else if s
            .float_distance_q12
            .zip(s.served_distance_q12)
            .is_some_and(|(f, i)| f - i <= POINT_KEY_ROUNDING_BOUND)
        {
            p.within += 1;
        } else {
            p.drift += 1;
        }
    }
    p
}

/// The integer served arm against integer greedy on one split and seed (claude-224): a
/// [`KeyedReading`] by their own tokens, so [`KeyedReading::carries`] and
/// [`KeyedReading::harmless`] read it as they read claude-220's served arm. The arms must hold the
/// same prompts.
///
/// Instrument faults, refused:
/// - a served reply that does not open with its keyed token;
/// - a reply the key did not open that differs from greedy's. The two artifacts decode alike there (claude-222).
fn integer_reading(
    split: &str,
    seed: u8,
    served: &[IntegerSample],
    greedy: &[IntegerSample],
    eos: u32,
    keep: &std::collections::HashSet<u32>,
    digits: &std::collections::HashSet<u32>,
) -> Result<KeyedReading> {
    let fault = |what: String| invalid(format!("{split} s{seed} integer: {what} (an instrument fault)"));
    if served.len() != greedy.len()
        || served.iter().zip(greedy).any(|(s, g)| (s.position, &s.prompt, &s.reference) != (g.position, &g.prompt, &g.reference))
    {
        return Err(fault("the served and greedy arms do not hold the same prompts".to_owned()));
    }
    let exact = |s: &IntegerSample| cut_at_eos(&s.reply, eos) == cut_at_eos(&s.reference, eos);
    let template = |s: &IntegerSample| {
        value_mask(cut_at_eos(&s.reply, eos), keep, digits) == value_mask(cut_at_eos(&s.reference, eos), keep, digits)
    };
    let pair = |(b, c): (usize, usize), arm: bool, base: bool| (b + usize::from(arm && !base), c + usize::from(!arm && base));
    let mut r = KeyedReading {
        split: split.to_owned(),
        key: "integer-point".to_owned(),
        seed,
        tau: f32::INFINITY,
        replies: served.len(),
        keyed: 0,
        right: 0,
        predicted_exact: (0, 0),
        exact: Some((0, 0)),
        template: Some((0, 0)),
        loops: Some((0, 0)),
    };
    for (s, g) in served.iter().zip(greedy) {
        match s.keyed {
            Some(token) if s.reply.first() != Some(&token) => {
                return Err(fault(format!("the served reply at {} does not open with its keyed token", s.position)));
            }
            Some(token) => {
                r.keyed += 1;
                r.right += usize::from(s.reference.first() == Some(&token));
            }
            None if s.reply != g.reply => {
                return Err(fault(format!("the reply at {} differs from greedy's though the key did not open it", s.position)));
            }
            None => {}
        }
        r.exact = r.exact.map(|e| pair(e, exact(s), exact(g)));
        r.template = r.template.map(|t| pair(t, template(s), template(g)));
        r.loops = r.loops.map(|l| pair(l, loops(&s.reply, eos), loops(&g.reply, eos)));
    }
    r.predicted_exact = r.exact.unwrap_or((0, 0));
    Ok(r)
}

/// The float names an integer served arm carries must be this seed's (claude-224). With the float
/// side's served samples of the split, each sample's `float_keyed` is that file's `keyed` at the
/// same position. Without them, which is a split no float arm served, it carries none. Anything
/// else is an instrument fault: parity would read another seed's names, or none.
fn check_float_names(split: &str, seed: u8, served: &[IntegerSample], float: Option<&[IdSample]>) -> Result<()> {
    let fault = |what: &str| invalid(format!("{split} s{seed} integer: {what} (an instrument fault)"));
    match float {
        Some(float) => {
            if float.len() != served.len()
                || served.iter().zip(float).any(|(s, f)| s.position != f.position || s.float_keyed != f.keyed)
            {
                return Err(fault("its float names are not this seed's served file's"));
            }
        }
        None if served.iter().any(|s| s.float_keyed.is_some()) => {
            return Err(fault("it carries float names, but no float arm served this split"));
        }
        None => {}
    }
    Ok(())
}

/// The integer grade (point-key-integer unit 2, claude-224), on the laptop.
/// - **What it reads:** the integer served arm against integer greedy on DEV and each panel, at seeds 1–3, with their parity against the float key and their free-generation measures.
/// - **`carries`:** each panel's per-seed reading of done-when 4's bar against that seed's DEV (claude-220's `carries`). The rows are the run's sheet's.
#[derive(Clone, Debug, Serialize)]
pub struct IntegerGrade {
    pub readings: Vec<KeyedReading>,
    pub parity: Vec<IntegerParity>,
    /// Each `(split, seed)`'s capped replies, served and greedy: those the context cut.
    pub capped: Vec<(String, u8, usize, usize)>,
    pub generation: Vec<(String, u8, String, Generation)>,
    pub carries: Vec<(String, u8, bool)>,
}

/// [`IntegerGrade`] of `runs`.
/// - **The files:** each `r-drop-s{seed}` there holds `integer-greedy-{split}.json` (from `/7`) and `integer-served-{split}.json` (from `/8`), as `fbs-toks --replies` writes them, for DEV and each name of `panels`.
/// - **TRAIN's templates and replies** come from `data_root`, as in [`served_grade`].
pub fn integer_grade(data_root: &Path, runs: &Path, panels: &[String], float_runs: &Path) -> Result<IntegerGrade> {
    let bos = hs::SHEET_BOS_ID;
    let tokenizer = pinned_tokenizer(data_root)?;
    let eos = tokenizer.token_id("<|eos|>").ok_or_else(|| invalid("the pinned tokenizer has no <|eos|>"))?;
    let digits = digit_tokens(&fs::read_to_string(data_root.join("tokenizer.json"))?)?;
    let (train_tokens, train_mask) = load_train_split(data_root)?;
    let keep = top_tokens(&train_tokens, OPENING_TEMPLATE_K, bos);
    let train_turns = split_turns(&train_tokens, &train_mask, bos).replies;
    drop((train_tokens, train_mask));
    let mut grade = IntegerGrade { readings: Vec::new(), parity: Vec::new(), capped: Vec::new(), generation: Vec::new(), carries: Vec::new() };
    let splits: Vec<&str> = std::iter::once("dev").chain(panels.iter().map(String::as_str)).collect();
    for seed in 1..=3u8 {
        let dir = runs.join(format!("r-drop-s{seed}"));
        let mut dev = None;
        for &split in &splits {
            let read = |arm: &str| -> Result<Vec<IntegerSample>> { read_scored(&dir.join(format!("integer-{arm}-{split}.json"))) };
            let (served, greedy) = (read("served")?, read("greedy")?);
            let float = float_runs.join(format!("r-drop-s{seed}")).join(format!("samples-{split}-served-memory.json"));
            let names = if float.exists() { Some(read_scored::<Vec<IdSample>>(&float)?) } else { None };
            check_float_names(split, seed, &served, names.as_deref())?;
            let reading = integer_reading(split, seed, &served, &greedy, eos, &keep, &digits)?;
            grade.parity.push(integer_parity(split, seed, &served));
            let capped = |arm: &[IntegerSample]| arm.iter().filter(|s| s.end == "context").count();
            grade.capped.push((split.to_owned(), seed, capped(&served), capped(&greedy)));
            for (arm, samples) in [("greedy", &greedy), ("served", &served)] {
                let replies: Vec<Vec<u32>> = samples.iter().map(|s| s.reply.clone()).collect();
                grade.generation.push((split.to_owned(), seed, arm.to_owned(), generation(&replies, &train_turns, eos)));
            }
            if split == "dev" {
                dev = Some(reading.clone());
            } else if let Some(dev) = &dev {
                grade.carries.push((split.to_owned(), seed, reading.carries(dev)));
            }
            grade.readings.push(reading);
        }
    }
    Ok(grade)
}

/// One reply row (turn-memory-reply, claude-192): the prompt its window shows for its turn, back
/// from the reply's start to the last response token or BOS; the reply's own earlier tokens; its
/// target; and whether it opens the reply or copies the prompt ([`copies_prompt`]).
#[derive(Clone, Debug, PartialEq, Eq)]
struct ReplyRow {
    /// The row's window, an index into the windows it was read from, and the input offset it is
    /// predicted from.
    window: usize,
    offset: usize,
    prompt: Vec<u32>,
    reply: Vec<u32>,
    target: u32,
    opening: bool,
    copy: bool,
}

/// The reply table's key for `row` (claude-193): the probe's `turn-8 + position + reply-last-1`,
/// so the read keys exactly as the probe counted.
fn reply_key(row: &ReplyRow) -> Vec<u32> {
    ReplyKey::TurnPosition(1).of(row)
}

/// The reply table's rows in one forward of `windows`, each `time` positions long: every row
/// past its reply's opening, with its [`reply_key`], at [`assemble`]'s row for it, the window's
/// index times `time` plus the offset (claude-274; before, the index times [`SHEET_CONTEXT`]).
fn reply_table_rows(windows: &[CausalWindow], mask: &[u8], time: usize) -> Vec<(usize, Vec<u32>)> {
    reply_rows(windows, mask)
        .iter()
        .filter(|row| !row.opening)
        .map(|row| (row.window * time + row.offset, reply_key(row)))
        .collect()
}

/// For each of `windows`, its rows past their reply's opening, each as its input offset and its
/// [`reply_key`]: the rows the reply table is named at (claude-193).
/// Each window's rows past their openings, as [`reply_offsets`] gives them.
type ReplyOffsets = Vec<Vec<(usize, Vec<u32>)>>;

fn reply_offsets(windows: &[CausalWindow], mask: &[u8]) -> ReplyOffsets {
    let mut out = vec![Vec::new(); windows.len()];
    for row in reply_rows(windows, mask) {
        if !row.opening {
            out[row.window].push((row.offset, reply_key(&row)));
        }
    }
    out
}

/// The reply key a greedy reply's next step reads (claude-193): the turn's first
/// [`TURN_KEY`] ids, by the turn anchor at the prompt's last token, which equals the probe's
/// first-8 key at every opening (claude-154), then the reply's length and its last id. `window`
/// is the step's window and `opening` the window index of the prompt's last token.
fn greedy_reply_key(window: &[u32], opening: usize, reply: &[u32]) -> Option<Vec<u32>> {
    use crate::flock_read_opening::{opening_slots, OpeningAnchor};
    let last = *reply.last()?;
    let slots = opening_slots(&window[..=opening], TURN_KEY, OpeningAnchor::Turn);
    let turn = slots[opening].iter().map(|&s| window[s]);
    Some(
        turn.chain([u32::MAX, reply.len() as u32, u32::MAX, last])
            .collect(),
    )
}

fn reply_rows(windows: &[CausalWindow], mask: &[u8]) -> Vec<ReplyRow> {
    let mut rows = Vec::new();
    for (index, w) in windows.iter().enumerate() {
        let scored = w
            .scored_offsets
            .iter()
            .zip(&w.scored_targets)
            .zip(&w.scored_corpus_positions);
        for ((&offset, &target), &position) in scored {
            if mask[position] == 0 {
                continue;
            }
            // The corpus position of `input_ids[0]`.
            let base = position - 1 - offset;
            let part = |i: usize, reply: bool| {
                w.input_ids[i] != hs::SHEET_BOS_ID && (mask[base + i] == 1) == reply
            };
            let mut start = offset + 1;
            while start > 0 && part(start - 1, true) {
                start -= 1;
            }
            let mut first = start;
            while first > 0 && part(first - 1, false) {
                first -= 1;
            }
            rows.push(ReplyRow {
                window: index,
                offset,
                prompt: w.input_ids[first..start].to_vec(),
                reply: w.input_ids[start..=offset].to_vec(),
                target,
                opening: opens_reply(mask, position),
                copy: copies_prompt(w, offset, position, target, mask),
            });
        }
    }
    rows
}

/// How many of the turn's first prompt tokens a turn key holds, as claude-161's memory. The reply
/// table's keys hold this many, whatever the opening table's `k`; the export carries it
/// (claude-198).
pub(crate) const TURN_KEY: usize = 8;

/// Which part of a [`ReplyRow`] a [`ReplyBound`] keys on.
#[derive(Clone, Copy, Debug)]
enum ReplyKey {
    None,
    /// The reply's last `j` tokens alone: the control with no turn.
    Reply(usize),
    /// The turn's first 8 tokens and the reply's last `j`.
    Turn(usize),
    /// The turn's first 8 tokens, the reply position, and the reply's last `j`. At position 0
    /// this is claude-161's opening key.
    TurnPosition(usize),
}

impl ReplyKey {
    fn of(self, row: &ReplyRow) -> Vec<u32> {
        let last = |j: usize| &row.reply[row.reply.len() - j.min(row.reply.len())..];
        let turn = &row.prompt[..TURN_KEY.min(row.prompt.len())];
        // The parts are joined by `u32::MAX`, which no token id takes.
        match self {
            ReplyKey::None => Vec::new(),
            ReplyKey::Reply(j) => last(j).to_vec(),
            ReplyKey::Turn(j) => [turn, &[u32::MAX], last(j)].concat(),
            ReplyKey::TurnPosition(j) => {
                [turn, &[u32::MAX, row.reply.len() as u32, u32::MAX], last(j)].concat()
            }
        }
    }

    fn name(self) -> String {
        match self {
            ReplyKey::None => "none".to_owned(),
            ReplyKey::Reply(j) => format!("reply-last-{j}"),
            ReplyKey::Turn(j) => format!("turn-{TURN_KEY}+reply-last-{j}"),
            ReplyKey::TurnPosition(j) => format!("turn-{TURN_KEY}+position+reply-last-{j}"),
        }
    }
}

/// DEV rows of one kind: how many there are, how many a key the table keeps names, and how many
/// of those it names right.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct KindCount {
    pub rows: usize,
    pub named: usize,
    pub hits: usize,
}

/// What an exact memory of TRAIN, keyed by one [`ReplyKey`], could name among DEV's reply rows
/// (`--reply-bound`). The table keeps, for each key, TRAIN's commonest target (ties to the smaller
/// id), and only where that target's share of the key's rows is at least `min_share`. A DEV row
/// whose key is not kept is named by nothing: the trunk alone would decide it. A count over the
/// corpus, not a model.
#[derive(Clone, Debug, Serialize)]
pub struct ReplyBound {
    pub key: String,
    pub min_share: f64,
    pub train_rows: usize,
    /// Keys the table keeps at `min_share`.
    pub train_keys: usize,
    pub all: KindCount,
    pub openings: KindCount,
    pub copies: KindCount,
    /// Rows that neither open a reply nor copy the prompt.
    pub rest: KindCount,
    /// Openings named right when an opening whose key is not kept takes TRAIN's commonest
    /// opening, as [`opening_bound`] counts: claude-161's 607 for the opening key.
    pub opening_hits_with_fallback: usize,
}

/// TRAIN's table for `key`: each key's commonest target (ties to the smaller id), that target's
/// count, and the key's rows ([`reply_bound_rows`], and the census's row selection).
type ReplyTableCounts = std::collections::HashMap<Vec<u32>, (u32, usize, usize)>;

fn reply_table(key: ReplyKey, train: &[ReplyRow]) -> ReplyTableCounts {
    let mut counts = std::collections::HashMap::<Vec<u32>, Vec<(u32, usize)>>::new();
    for row in train {
        let targets = counts.entry(key.of(row)).or_default();
        match targets.iter_mut().find(|(t, _)| *t == row.target) {
            Some((_, n)) => *n += 1,
            None => targets.push((row.target, 1)),
        }
    }
    counts
        .into_iter()
        .map(|(k, targets)| {
            let rows = targets.iter().map(|(_, n)| n).sum();
            let (target, n) = targets
                .into_iter()
                .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0)))
                .unwrap_or((0, 0));
            (k, (target, n, rows))
        })
        .collect()
}

/// Whether a table entry `(target, count, rows)` is kept at `min_share`.
fn table_keeps(stored: &(u32, usize, usize), min_share: f64) -> bool {
    stored.1 as f64 >= min_share * stored.2 as f64
}

/// [`ReplyBound`] for `key` at each of `shares`, the table built from `train`.
fn reply_bound_rows(
    key: ReplyKey,
    train: &[ReplyRow],
    dev: &[ReplyRow],
    shares: &[f64],
) -> Vec<ReplyBound> {
    let mut openings = std::collections::BTreeMap::<u32, usize>::new();
    for row in train.iter().filter(|row| row.opening) {
        *openings.entry(row.target).or_default() += 1;
    }
    let table = reply_table(key, train);
    let fallback = commonest(&openings).map(|(t, _)| t);
    shares
        .iter()
        .map(|&min_share| {
            let kept = |stored: &(u32, usize, usize)| table_keeps(stored, min_share);
            let mut out = ReplyBound {
                key: key.name(),
                min_share,
                train_rows: train.len(),
                train_keys: table.values().filter(|s| kept(s)).count(),
                all: KindCount::default(),
                openings: KindCount::default(),
                copies: KindCount::default(),
                rest: KindCount::default(),
                opening_hits_with_fallback: 0,
            };
            for row in dev {
                let named = table.get(&key.of(row)).filter(|s| kept(s)).map(|s| s.0);
                let mut kinds = vec![&mut out.all];
                if row.opening {
                    kinds.push(&mut out.openings);
                }
                if row.copy {
                    kinds.push(&mut out.copies);
                }
                if !row.opening && !row.copy {
                    kinds.push(&mut out.rest);
                }
                for kind in kinds {
                    kind.rows += 1;
                    kind.named += usize::from(named.is_some());
                    kind.hits += usize::from(named == Some(row.target));
                }
                if row.opening {
                    out.opening_hits_with_fallback +=
                        usize::from(named.or(fallback) == Some(row.target));
                }
            }
            out
        })
        .collect()
}

/// The stored-share thresholds `--reply-bound` reports.
const REPLY_BOUND_SHARES: [f64; 4] = [0.0, 0.5, 0.8, 0.95];

/// [`ReplyBound`] over DEV's reply rows, with TRAIN's table, for each [`ReplyKey`]: none, the
/// reply's last 1–4 tokens, the turn with the reply's last 0–4, and the turn with the position
/// and the reply's last 0–1 (turn-memory-reply's probe, claude-192).
pub fn reply_bound(data_root: &Path) -> Result<Vec<ReplyBound>> {
    let (train_tokens, train_mask) = load_train_split(data_root)?;
    let (train_windows, _) =
        hs::build_document_windows(&train_tokens, &train_mask, SHEET_CONTEXT, hs::SHEET_BOS_ID)?;
    let (dev_windows, dev_mask) = load_dev(data_root)?;
    let train = reply_rows(&train_windows, &train_mask);
    let dev = reply_rows(&dev_windows, &dev_mask);
    let keys = std::iter::once(ReplyKey::None)
        .chain((1..=4).map(ReplyKey::Reply))
        .chain((0..=4).map(ReplyKey::Turn))
        .chain((0..=1).map(ReplyKey::TurnPosition));
    Ok(keys
        .flat_map(|key| reply_bound_rows(key, &train, &dev, &REPLY_BOUND_SHARES))
        .collect())
}

/// Pinned SHA-256 of the sealed Fresh V2 panel's `tokens.u16`, opened once for M1 bar (e)
/// (`docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md:69`). The census only counts it.
pub const PINNED_FRESH_V2_TOKENS_SHA256: &str =
    "0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8";
/// Pinned SHA-256 of Fresh V2's `response_mask.u8`.
pub const PINNED_FRESH_V2_MASK_SHA256: &str =
    "a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e";
/// Fresh V2's payload tokens and eligible response targets.
pub const EXPECTED_FRESH_V2_TOKENS: usize = 44_678;
pub const EXPECTED_FRESH_V2_ELIGIBLE_TARGETS: usize = 14_479;

/// The census's value masks keep TRAIN's K most frequent tokens (one-model-end-to-end, claude-202).
pub const CENSUS_MASK_KS: [usize; 2] = [256, 1024];
/// The n-gram lengths the census compares.
pub const CENSUS_NGRAMS: [usize; 2] = [8, 13];
/// A panel holds new templates when its value-masked prompt share at K = 1,024 is at most this,
/// fixed in claude-202's ORDER before any count.
pub const CENSUS_NEW_TEMPLATE_SHARE: f64 = 0.5;
/// The precision a no-turn key needs to count as the reply-only control (claude-192's ORDER).
const CENSUS_CONTROL_PRECISION: f64 = 0.83;
/// Stands for a masked run in a value-masked sequence; never a token id.
const CENSUS_MASK: u32 = u32::MAX;

/// One split's size (census M0).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SplitSize {
    pub file_bytes: u64,
    pub payload_tokens: usize,
    pub eligible_targets: usize,
    pub documents: usize,
    pub prompts: usize,
    pub replies: usize,
}

/// A split's prompts and replies, each a maximal run of mask-0 or mask-1 tokens inside one
/// document; `<|bos|>` starts a document and belongs to neither.
#[derive(Debug, Default, PartialEq, Eq)]
struct Turns {
    documents: usize,
    prompts: Vec<Vec<u32>>,
    replies: Vec<Vec<u32>>,
}

fn split_turns(tokens: &[u32], mask: &[u8], bos: u32) -> Turns {
    let mut turns = Turns::default();
    let mut run: Vec<u32> = Vec::new();
    let mut run_mask = 0u8;
    let close = |run: &mut Vec<u32>, run_mask: u8, turns: &mut Turns| {
        if !run.is_empty() {
            let done = std::mem::take(run);
            if run_mask == 1 {
                turns.replies.push(done);
            } else {
                turns.prompts.push(done);
            }
        }
    };
    for (&t, &m) in tokens.iter().zip(mask) {
        if t == bos {
            close(&mut run, run_mask, &mut turns);
            turns.documents += 1;
            continue;
        }
        if !run.is_empty() && m != run_mask {
            close(&mut run, run_mask, &mut turns);
        }
        run_mask = m;
        run.push(t);
    }
    close(&mut run, run_mask, &mut turns);
    turns
}

/// A 128-bit hash of `ids`, two FNV-1a passes with different offsets: the census's set key, so its
/// n-gram sets fit in memory. Deterministic.
fn census_hash(ids: &[u32]) -> u128 {
    let pass = |mut h: u64| {
        for &id in ids {
            for b in id.to_le_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        }
        h ^ (ids.len() as u64)
    };
    (u128::from(pass(0xcbf2_9ce4_8422_2325)) << 64) | u128::from(pass(0x84222325_cbf29ce4))
}

/// The hashes of every `n`-token window inside each run (no window crosses a run).
fn ngram_set(runs: &[Vec<u32>], n: usize) -> std::collections::HashSet<u128> {
    runs.iter()
        .flat_map(|run| run.windows(n).map(census_hash))
        .collect()
}

/// The share of `runs`' `n`-token windows that `train` holds, and the window count.
fn ngram_share(runs: &[Vec<u32>], train: &std::collections::HashSet<u128>, n: usize) -> (f64, usize) {
    let (mut found, mut total) = (0usize, 0usize);
    for window in runs.iter().flat_map(|run| run.windows(n)) {
        total += 1;
        found += usize::from(train.contains(&census_hash(window)));
    }
    (if total == 0 { 0.0 } else { found as f64 / total as f64 }, total)
}

/// The share of `runs` equal to a run of `train` (as hashed by [`census_hash`]).
fn exact_share(runs: &[Vec<u32>], train: &std::collections::HashSet<u128>) -> f64 {
    let found = runs.iter().filter(|run| train.contains(&census_hash(run))).count();
    if runs.is_empty() { 0.0 } else { found as f64 / runs.len() as f64 }
}

/// `run` with every token outside `keep` (or in `digits`) replaced by one [`CENSUS_MASK`], and
/// each run of masks collapsed to one.
fn value_mask(
    run: &[u32],
    keep: &std::collections::HashSet<u32>,
    digits: &std::collections::HashSet<u32>,
) -> Vec<u32> {
    let mut out: Vec<u32> = Vec::with_capacity(run.len());
    for &t in run {
        let id = if keep.contains(&t) && !digits.contains(&t) { t } else { CENSUS_MASK };
        if !(id == CENSUS_MASK && out.last() == Some(&CENSUS_MASK)) {
            out.push(id);
        }
    }
    out
}

/// TRAIN's `k` most frequent tokens, ties to the smaller id.
fn top_tokens(tokens: &[u32], k: usize, bos: u32) -> std::collections::HashSet<u32> {
    let mut counts = std::collections::HashMap::<u32, usize>::new();
    for &t in tokens.iter().filter(|&&t| t != bos) {
        *counts.entry(t).or_default() += 1;
    }
    let mut ranked: Vec<(u32, usize)> = counts.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    ranked.into_iter().take(k).map(|(t, _)| t).collect()
}

/// The ids of the tokenizer's single-digit pieces (a digit, with or without the byte-level space
/// mark `Ġ` before it): the census masks them as values, since numbers are split into digits.
fn digit_tokens(tokenizer_json: &str) -> Result<std::collections::HashSet<u32>> {
    let value: serde_json::Value = serde_json::from_str(tokenizer_json)?;
    let vocab = value["model"]["vocab"]
        .as_object()
        .ok_or_else(|| invalid("tokenizer.json has no model.vocab"))?;
    Ok(vocab
        .iter()
        .filter(|(piece, _)| {
            let bare = piece.strip_prefix('\u{120}').unwrap_or(piece);
            bare.len() == 1 && bare.as_bytes()[0].is_ascii_digit()
        })
        .filter_map(|(_, id)| id.as_u64().map(|id| id as u32))
        .collect())
}

/// One panel's overlap with TRAIN (census M3–M6).
#[derive(Clone, Debug, Serialize)]
pub struct PanelOverlap {
    pub panel: String,
    pub prompts: usize,
    pub replies: usize,
    pub exact_prompt_share: f64,
    pub exact_reply_share: f64,
    /// `(n, share, windows)` for each n in [`CENSUS_NGRAMS`].
    pub prompt_ngram_share: Vec<(usize, f64, usize)>,
    pub reply_ngram_share: Vec<(usize, f64, usize)>,
    /// `(k, share)` for each k in [`CENSUS_MASK_KS`].
    pub prompt_template_share: Vec<(usize, f64)>,
    pub reply_template_share: Vec<(usize, f64)>,
    /// Openings whose turn's first-8 key TRAIN's opening table holds, and all openings (M6).
    pub keyed_openings: usize,
    pub openings: usize,
    /// The ORDER's rule: the prompt template share at K = 1,024 is at most
    /// [`CENSUS_NEW_TEMPLATE_SHARE`].
    pub new_templates: bool,
}

/// One no-turn key's later-row count, for the reply-only control (census M1).
#[derive(Clone, Debug, Serialize)]
pub struct ControlCell {
    pub key: String,
    pub min_share: f64,
    pub named: usize,
    pub hits: usize,
}

/// The pinned numbers the census must reproduce (M1), and step 2's rows (M2).
#[derive(Clone, Debug, Serialize)]
pub struct CensusRows {
    pub dev_openings: usize,
    pub dev_keyed_openings: usize,
    /// DEV's later rows (past their openings) the turn-8 + position + reply-last-1 table names at
    /// share 0.5, and how many it names right.
    pub later_rows: usize,
    pub later_named: usize,
    pub later_hits: usize,
    /// The most later-row hits among no-turn keys with precision at least 0.83, and the grid.
    pub reply_only_control: usize,
    pub reply_only_grid: Vec<ControlCell>,
    /// DEV corpus positions of the later rows the table does not name and the openings no key
    /// names: step 2's decision rows, pooled.
    pub unnamed_later: usize,
    pub unkeyed_openings: usize,
    pub decision_positions: Vec<usize>,
}

/// The data census (one-model-end-to-end, claude-202): every count the ORDER fixes.
#[derive(Clone, Debug, Serialize)]
pub struct Census {
    pub train: SplitSize,
    pub dev: SplitSize,
    pub fresh_v2: SplitSize,
    pub rows: CensusRows,
    pub panels: Vec<PanelOverlap>,
    /// The `--panel` splits (claude-204): each one's name, its `tokens.u16` sha256 and its size.
    /// They are counted, not pinned.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub extra: Vec<(String, String, SplitSize)>,
    /// The TRAIN counted against when it is not the pinned one (`--train-split`, claude-275).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub train_split: Option<TrainSplit>,
    /// Each `--panel-queries` panel's answers against TRAIN's replies (claude-275).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub answers: Vec<AnswerOverlap>,
}

/// An unpinned UORT split (census `--panel`, claude-204): `DIR/tokens.u16` and
/// `DIR/response_mask.u8`, refused unless the header is UORT, the payload and mask lengths match
/// its count, the first token is BOS and every token is in the sheet's vocabulary. Returns the
/// tokens, the mask and the token file's sha256.
fn load_unpinned_uort(dir: &Path) -> Result<(Vec<u32>, Vec<u8>, String)> {
    let token_bytes = fs::read(dir.join("tokens.u16"))?;
    let mask = fs::read(dir.join("response_mask.u8"))?;
    if token_bytes.len() < hs::UORT_HEADER_BYTES || &token_bytes[..4] != b"UORT" {
        return Err(invalid(format!(
            "{}: tokens.u16 has no 64-byte UORT header",
            dir.display()
        )));
    }
    let count = u64::from_le_bytes(token_bytes[8..16].try_into().unwrap()) as usize;
    let payload = &token_bytes[hs::UORT_HEADER_BYTES..];
    if payload.len() != count * 2 || mask.len() != count {
        return Err(invalid(format!(
            "{}: the UORT payload or mask does not match its count {count}",
            dir.display()
        )));
    }
    let tokens: Vec<u32> = payload
        .chunks_exact(2)
        .map(|w| u32::from(u16::from_le_bytes([w[0], w[1]])))
        .collect();
    if tokens.first() != Some(&hs::SHEET_BOS_ID)
        || tokens.iter().any(|&t| t as usize >= hs::SHEET_VOCAB)
    {
        return Err(invalid(format!(
            "{}: the tokens are out of range or do not start with BOS",
            dir.display()
        )));
    }
    Ok((tokens, mask, crate::sha256_bytes(&token_bytes)))
}

/// The corpus position of `row`, from its window.
fn row_position(row: &ReplyRow, windows: &[CausalWindow]) -> Option<usize> {
    let w = windows.get(row.window)?;
    let i = w.scored_offsets.iter().position(|&off| off == row.offset)?;
    w.scored_corpus_positions.get(i).copied()
}

/// Step 2's rows: DEV's later rows the reply table does not name at share 0.5, and its openings
/// no first-8 key names, as `(later, openings)` index lists into `dev`.
fn census_decision_rows(
    reply: &ReplyTableCounts,
    opening: &ReplyTableCounts,
    dev: &[ReplyRow],
) -> (Vec<usize>, Vec<usize>) {
    let mut later = Vec::new();
    let mut openings = Vec::new();
    for (i, row) in dev.iter().enumerate() {
        if row.opening {
            if !opening.contains_key(&ReplyKey::TurnPosition(0).of(row)) {
                openings.push(i);
            }
        } else if !reply
            .get(&ReplyKey::TurnPosition(1).of(row))
            .is_some_and(|s| table_keeps(s, REPLY_MEMORY_SHARE))
        {
            later.push(i);
        }
    }
    (later, openings)
}

/// The split's size from its file and turns.
fn split_size(path: &Path, tokens: &[u32], mask: &[u8], turns: &Turns) -> Result<SplitSize> {
    Ok(SplitSize {
        file_bytes: fs::metadata(path)?.len(),
        payload_tokens: tokens.len(),
        eligible_targets: tokens
            .iter()
            .zip(mask)
            .enumerate()
            .filter(|&(p, (&t, &m))| p > 0 && t != hs::SHEET_BOS_ID && m == 1)
            .count(),
        documents: turns.documents,
        prompts: turns.prompts.len(),
        replies: turns.replies.len(),
    })
}

/// The data census of claude-202's ORDER over TRAIN, DEV and the sealed Fresh V2 panel in
/// `fresh_v2` (its `tokens.u16` and `response_mask.u8`, pinned), and over each `(name, dir)` of
/// `extra`, unpinned (claude-204). Counts only; no model, no GPU. TRAIN is the pinned split, or
/// `train` when named (`--train-split`, claude-275). For each `(name, file)` of `panel_queries`,
/// a chained `extra` panel and its `queries.json`, it counts the answers found as words of
/// TRAIN's replies ([`AnswerOverlap`]).
pub fn census(
    data_root: &Path,
    fresh_v2: &Path,
    extra: &[(String, PathBuf)],
    train: Option<&TrainSplit>,
    panel_queries: &[(String, PathBuf)],
) -> Result<Census> {
    let bos = hs::SHEET_BOS_ID;
    for (name, _) in panel_queries {
        if !extra.iter().any(|(panel, _)| panel == name) {
            return Err(invalid(format!("--panel-queries {name} needs --panel {name}=DIR")));
        }
    }
    let (train_tokens, train_mask, train_path) = match train {
        None => {
            let (tokens, mask) = load_train_split(data_root)?;
            (tokens, mask, data_root.join("mw-balp/train/tokens.u16"))
        }
        Some(split) => {
            let (tokens, mask) = load_named_split(split)?;
            (tokens, mask, split.dir.join("tokens.u16"))
        }
    };
    let (dev_tokens, dev_mask) = hs::load_pinned_uort_split(
        &data_root.join("ft/dev/tokens.u16"),
        &data_root.join("ft/dev/response_mask.u8"),
        hs::PINNED_DEV_TOKENS_SHA256,
        hs::PINNED_DEV_MASK_SHA256,
        hs::EXPECTED_DEV_TOKENS,
        hs::EXPECTED_DEV_ELIGIBLE_TARGETS,
        hs::SHEET_VOCAB,
        bos,
    )?;
    let (fresh_tokens, fresh_mask) = load_fresh_v2(fresh_v2)?;
    let windows = |tokens: &[u32], mask: &[u8]| -> Result<Vec<CausalWindow>> {
        Ok(hs::build_document_windows(tokens, mask, SHEET_CONTEXT, bos)?.0)
    };
    let (train_w, dev_w, fresh_w) = (
        windows(&train_tokens, &train_mask)?,
        windows(&dev_tokens, &dev_mask)?,
        windows(&fresh_tokens, &fresh_mask)?,
    );
    let train_rows = reply_rows(&train_w, &train_mask);
    let dev_rows = reply_rows(&dev_w, &dev_mask);
    let fresh_rows = reply_rows(&fresh_w, &fresh_mask);

    // M1 and M2, with the probe's own tables.
    let opening_table = reply_table(ReplyKey::TurnPosition(0), &train_rows);
    let reply = reply_table(ReplyKey::TurnPosition(1), &train_rows);
    let keyed = |rows: &[ReplyRow]| {
        rows.iter()
            .filter(|r| r.opening && opening_table.contains_key(&ReplyKey::TurnPosition(0).of(r)))
            .count()
    };
    let later: Vec<&ReplyRow> = dev_rows.iter().filter(|r| !r.opening).collect();
    let named_later = |table: &ReplyTableCounts, key: ReplyKey, share: f64| {
        let mut named = 0;
        let mut hits = 0;
        for row in &later {
            if let Some(stored) = table.get(&key.of(row)).filter(|s| table_keeps(s, share)) {
                named += 1;
                hits += usize::from(stored.0 == row.target);
            }
        }
        (named, hits)
    };
    let (later_named, later_hits) = named_later(&reply, ReplyKey::TurnPosition(1), REPLY_MEMORY_SHARE);
    let mut grid = Vec::new();
    for key in std::iter::once(ReplyKey::None).chain((1..=4).map(ReplyKey::Reply)) {
        let table = reply_table(key, &train_rows);
        for &share in &REPLY_BOUND_SHARES {
            let (named, hits) = named_later(&table, key, share);
            grid.push(ControlCell { key: key.name(), min_share: share, named, hits });
        }
    }
    let reply_only_control = grid
        .iter()
        .filter(|c| c.named > 0 && c.hits as f64 >= CENSUS_CONTROL_PRECISION * c.named as f64)
        .map(|c| c.hits)
        .max()
        .unwrap_or(0);
    let (unnamed, unkeyed) = decision_positions(&reply, &opening_table, &dev_rows, &dev_w)?;
    let mut pooled: Vec<usize> = unnamed.iter().chain(&unkeyed).copied().collect();
    pooled.sort_unstable();
    let rows = CensusRows {
        dev_openings: dev_rows.iter().filter(|r| r.opening).count(),
        dev_keyed_openings: keyed(&dev_rows),
        later_rows: later.len(),
        later_named,
        later_hits,
        reply_only_control,
        reply_only_grid: grid,
        unnamed_later: unnamed.len(),
        unkeyed_openings: unkeyed.len(),
        decision_positions: pooled,
    };

    // M0 and M3–M6.
    let train_turns = split_turns(&train_tokens, &train_mask, bos);
    let dev_turns = split_turns(&dev_tokens, &dev_mask, bos);
    let fresh_turns = split_turns(&fresh_tokens, &fresh_mask, bos);
    let digits = digit_tokens(&fs::read_to_string(data_root.join("tokenizer.json"))?)?;
    let exact = |runs: &[Vec<u32>]| runs.iter().map(|r| census_hash(r)).collect();
    let (train_prompts, train_replies): (std::collections::HashSet<u128>, std::collections::HashSet<u128>) =
        (exact(&train_turns.prompts), exact(&train_turns.replies));
    let ngram_sets: Vec<(usize, std::collections::HashSet<u128>, std::collections::HashSet<u128>)> =
        CENSUS_NGRAMS
            .iter()
            .map(|&n| (n, ngram_set(&train_turns.prompts, n), ngram_set(&train_turns.replies, n)))
            .collect();
    let masks: Vec<(usize, std::collections::HashSet<u32>)> = CENSUS_MASK_KS
        .iter()
        .map(|&k| (k, top_tokens(&train_tokens, k, bos)))
        .collect();
    let template_sets: Vec<(usize, std::collections::HashSet<u128>, std::collections::HashSet<u128>)> = masks
        .iter()
        .map(|(k, keep)| {
            let set = |runs: &[Vec<u32>]| {
                runs.iter().map(|r| census_hash(&value_mask(r, keep, &digits))).collect()
            };
            (*k, set(&train_turns.prompts), set(&train_turns.replies))
        })
        .collect();
    let overlap = |panel: &str, turns: &Turns, rows: &[ReplyRow]| {
        let template = |runs: &[Vec<u32>], prompts: bool| -> Vec<(usize, f64)> {
            template_sets
                .iter()
                .zip(&masks)
                .map(|((k, p, r), (_, keep))| {
                    let set = if prompts { p } else { r };
                    let masked: Vec<Vec<u32>> =
                        runs.iter().map(|run| value_mask(run, keep, &digits)).collect();
                    (*k, exact_share(&masked, set))
                })
                .collect()
        };
        let prompt_template_share = template(&turns.prompts, true);
        let new_templates = prompt_template_share
            .iter()
            .find(|(k, _)| *k == 1024)
            .is_some_and(|(_, share)| *share <= CENSUS_NEW_TEMPLATE_SHARE);
        PanelOverlap {
            panel: panel.to_owned(),
            prompts: turns.prompts.len(),
            replies: turns.replies.len(),
            exact_prompt_share: exact_share(&turns.prompts, &train_prompts),
            exact_reply_share: exact_share(&turns.replies, &train_replies),
            prompt_ngram_share: ngram_sets
                .iter()
                .map(|(n, p, _)| {
                    let (share, windows) = ngram_share(&turns.prompts, p, *n);
                    (*n, share, windows)
                })
                .collect(),
            reply_ngram_share: ngram_sets
                .iter()
                .map(|(n, _, r)| {
                    let (share, windows) = ngram_share(&turns.replies, r, *n);
                    (*n, share, windows)
                })
                .collect(),
            prompt_template_share,
            reply_template_share: template(&turns.replies, false),
            keyed_openings: keyed(rows),
            openings: rows.iter().filter(|r| r.opening).count(),
            new_templates,
        }
    };
    let mut panels = vec![
        overlap("ft/dev", &dev_turns, &dev_rows),
        overlap("fresh_v2", &fresh_turns, &fresh_rows),
    ];
    let mut extra_sizes = Vec::with_capacity(extra.len());
    let mut answers = Vec::new();
    // TRAIN's reply words, read only when a queries panel needs them (claude-275).
    let train_words: std::collections::HashSet<String> = if panel_queries.is_empty() {
        std::collections::HashSet::new()
    } else {
        let tokenizer = pinned_tokenizer(data_root)?;
        train_turns
            .replies
            .iter()
            .flat_map(|reply| {
                words_of(&tokenizer.decode(reply)).map(str::to_owned).collect::<Vec<_>>()
            })
            .collect()
    };
    for (name, dir) in extra {
        let (tokens, mask, sha) = load_unpinned_uort(dir)?;
        let turns = split_turns(&tokens, &mask, bos);
        let rows = reply_rows(&windows(&tokens, &mask)?, &mask);
        panels.push(overlap(name, &turns, &rows));
        if let Some((_, file)) = panel_queries.iter().find(|(panel, _)| panel == name) {
            let queries = load_panel_queries(file, &sha, &sha256_bytes(&mask))?;
            let values: Vec<String> = queries.into_iter().map(|q| q.answer).collect();
            answers.push(answer_overlap(name, &values, &train_words));
        }
        extra_sizes.push((
            name.clone(),
            sha,
            split_size(&dir.join("tokens.u16"), &tokens, &mask, &turns)?,
        ));
    }
    Ok(Census {
        extra: extra_sizes,
        train_split: train.cloned(),
        answers,
        train: split_size(&train_path, &train_tokens, &train_mask, &train_turns)?,
        dev: split_size(&data_root.join("ft/dev/tokens.u16"), &dev_tokens, &dev_mask, &dev_turns)?,
        fresh_v2: split_size(&fresh_v2.join("tokens.u16"), &fresh_tokens, &fresh_mask, &fresh_turns)?,
        rows,
        panels,
    })
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

/// [`sample_windows`]' pick from each window's last reply opening (`--sample-opening last`,
/// claude-274): every window with a scored target that opens a reply at least 16 tokens in is
/// cut to a clone whose scored lists start at the last such target, and `samples` of those are
/// taken as [`sample_windows`] takes them. So each prompt is its window's input through that
/// opening, and its reference, corpus position and input flags follow it. In a chained
/// conversation that is the latest turn in the window, with the earlier turns before it. Its
/// reference is the scored targets left in the window, so a reply that runs past the window's
/// end is cut there: 7 of 16 on claude-274's chained panel at 192, none at 2,048.
fn last_openings(windows: &[CausalWindow], mask: &[u8], samples: usize) -> Vec<CausalWindow> {
    let cut: Vec<CausalWindow> = windows
        .iter()
        .filter_map(|w| {
            let at = w
                .scored_offsets
                .iter()
                .zip(&w.scored_corpus_positions)
                .rposition(|(&off, &p)| off >= 16 && opens_reply(mask, p))?;
            Some(CausalWindow {
                doc_id: w.doc_id,
                input_ids: w.input_ids.clone(),
                scored_offsets: w.scored_offsets[at..].to_vec(),
                scored_targets: w.scored_targets[at..].to_vec(),
                scored_corpus_positions: w.scored_corpus_positions[at..].to_vec(),
            })
        })
        .collect();
    sample_windows(&cut, mask, samples).into_iter().cloned().collect()
}

/// The scoring flags whose TRAIN table or trace read 192-token windows (claude-274): at any other
/// `context`, each one asked for is refused by name, before the checkpoint loads.
fn window_bound_flags(
    context: usize,
    keys: bool,
    served: bool,
    keyed: bool,
    traces: bool,
    forces: bool,
) -> Result<()> {
    let named: Vec<&str> = [
        (keys, "--opening-keys"),
        (served, "--served-key"),
        (keyed, "--keyed-decode"),
        (traces, "--decision-trace"),
        (forces, "--decision-force"),
    ]
    .into_iter()
    .filter_map(|(on, name)| on.then_some(name))
    .collect();
    if context == SHEET_CONTEXT || named.is_empty() {
        return Ok(());
    }
    Err(invalid(format!(
        "--context {context}: {} read 192-token windows; score them at 192",
        named.join(", ")
    )))
}

/// The row stride (claude-275): from 193 to 1,536 positions several windows share one scoring
/// forward ([`windows_per_forward`]), and [`row_picks`]' stride over them has no test there, so
/// `--score-rows --context N` refuses that range by name. At 192, and from 1,537 up, it is
/// tested: 16 windows of 192 a forward, or one window.
fn row_stride_window(context: usize) -> Result<()> {
    let per_forward = windows_per_forward(context);
    if context > SHEET_CONTEXT && per_forward > 1 {
        return Err(invalid(format!(
            "--context {context}: {per_forward} windows share a scoring forward there, and the row stride over them has no test; score at 192, or past 1,536"
        )));
    }
    Ok(())
}

/// One recall query of a chained panel, a row of `heldout-panel chain=N`'s `queries.json`
/// (claude-275): where its document and its reply start, the reply's length (its mask run, end
/// marker included), the note's value it must state, and how far back the note is.
#[derive(Clone, Debug, Deserialize)]
pub struct ChainQuery {
    pub document_start: usize,
    pub reply_start: usize,
    pub reply_tokens: usize,
    pub answer: String,
    pub chain: QueryChain,
}

/// The part of a query's `Chain` record the grade reads: tokens from the end of the note's user
/// text to the start of the query's.
#[derive(Clone, Debug, Deserialize)]
pub struct QueryChain {
    pub distance: usize,
}

/// `file`'s queries, refused unless it is a `uor-r4-chain-queries/v1` record of the split whose
/// `tokens.u16` and `response_mask.u8` hash to `tokens_sha256` and `mask_sha256`.
fn load_panel_queries(file: &Path, tokens_sha256: &str, mask_sha256: &str) -> Result<Vec<ChainQuery>> {
    let record: serde_json::Value = serde_json::from_slice(&fs::read(file)?)?;
    if record["schema"] != "uor-r4-chain-queries/v1" {
        return Err(invalid(format!("{} is not a chain-queries record", file.display())));
    }
    for (key, want) in [("tokens_sha256", tokens_sha256), ("mask_sha256", mask_sha256)] {
        if record[key].as_str() != Some(want) {
            return Err(invalid(format!(
                "{}: its {key} is not its panel's {want}",
                file.display()
            )));
        }
    }
    let queries: Vec<ChainQuery> = serde_json::from_value(record["queries"].clone())?;
    if queries.is_empty() {
        return Err(invalid(format!("{} holds no query", file.display())));
    }
    Ok(queries)
}

/// The prompts a chained panel's queries decode from at a `context`-token window (claude-275).
#[derive(Clone, Debug)]
struct QueryPrompts {
    /// Each decoded query's index and its window: the `context` positions ending at the reply's
    /// last target, or from the document's BOS when it is nearer, scoring the reply alone. So its
    /// prompt is the window's input through the reply's opening, as a served decode slides, and
    /// its reference is the whole reply.
    decoded: Vec<(usize, CausalWindow)>,
    /// The queries whose reply is longer than the window: counted, not decoded.
    past_window: Vec<usize>,
}

/// [`QueryPrompts`] for `queries` over a panel's `tokens` (BOS-opened documents) and response
/// `mask`, at windows of `context` positions. Refused when a reply or an answer is empty, when a
/// reply runs past the panel, when its document does not open with BOS before it in one
/// document, and when no mask run of the reply's length opens at its first position: queries of
/// another panel, or a writer fault.
fn query_prompts(
    tokens: &[u32],
    mask: &[u8],
    context: usize,
    queries: &[ChainQuery],
) -> Result<QueryPrompts> {
    let bos = hs::SHEET_BOS_ID;
    let mut prompts = QueryPrompts { decoded: Vec::new(), past_window: Vec::new() };
    for (index, query) in queries.iter().enumerate() {
        let (doc, start, len) = (query.document_start, query.reply_start, query.reply_tokens);
        let end = start + len;
        let fault = if len == 0 || query.answer.is_empty() {
            Some("its reply or its answer is empty")
        } else if end > tokens.len() || end > mask.len() {
            Some("its reply runs past the panel")
        } else if doc >= start || tokens[doc] != bos || tokens[doc + 1..end].contains(&bos) {
            Some("its document does not open with BOS before its reply")
        } else if mask[start - 1] != 0
            || mask[start..end].iter().any(|&m| m != 1)
            || mask.get(end).is_some_and(|&m| m == 1)
        {
            Some("no mask run of its reply's length opens at its reply")
        } else {
            None
        };
        if let Some(fault) = fault {
            return Err(invalid(format!("query {index} at {start}: {fault}")));
        }
        if len > context {
            prompts.past_window.push(index);
            continue;
        }
        // Inputs `from..end - 1`: the reply's last target is read at offset `context - 1`.
        let from = (end - 1).saturating_sub(context).max(doc);
        prompts.decoded.push((
            index,
            CausalWindow {
                doc_id: index,
                input_ids: tokens[from..end - 1].to_vec(),
                scored_offsets: (start - 1 - from..end - 1 - from).collect(),
                scored_targets: tokens[start..end].to_vec(),
                scored_corpus_positions: (start..end).collect(),
            },
        ));
    }
    Ok(prompts)
}

/// Whether `text` states `answer`: as a whole word, not inside a longer word or number.
fn states_answer(text: &str, answer: &str) -> bool {
    !answer.is_empty()
        && text.match_indices(answer).any(|(at, _)| {
            let before = text[..at].chars().next_back();
            let after = text[at + answer.len()..].chars().next();
            !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
        })
}

/// The distance bins' lower edges (claude-275): inside a 192-token window, out of it, and the
/// far bins P2b's sheet reads.
const RECALL_BIN_EDGES: [usize; 4] = [0, 192, 512, 1024];

/// One distance bin's queries, or all of them (claude-275).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct RecallBin {
    /// The bin's distances, `from` up to `to` (exclusive), or every distance past `from`.
    pub from: usize,
    pub to: Option<usize>,
    pub queries: usize,
    /// Decoded: the reply fits the window.
    pub decoded: usize,
    /// Decoded from the document's BOS: the whole conversation is in the prompt.
    pub whole_document: usize,
    /// Decoded, and the reference as decoded (`reply_tokens`, cut at its end marker) states the
    /// answer: the grade can be met.
    pub reference_states: usize,
    /// Decoded, and the reply, cut at its end marker, states the answer.
    pub stated: usize,
}

/// One query's outcome (claude-275), in `queries.json` order, so two windows' or two memory
/// settings' outcomes pair query by query.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct QueryOutcome {
    pub query: usize,
    pub position: usize,
    pub distance: usize,
    pub decoded: bool,
    pub whole_document: bool,
    pub reference_states: bool,
    pub stated: bool,
}

/// A chained panel's recall at one window and memory setting (claude-275):
/// `recall-{panel}-{memory|no-memory}.json`.
#[derive(Clone, Debug, Serialize)]
pub struct QueryRecall {
    pub split: String,
    pub memory: bool,
    pub context: usize,
    pub reply_tokens: usize,
    pub total: RecallBin,
    pub bins: Vec<RecallBin>,
    pub outcomes: Vec<QueryOutcome>,
}

/// The recall grade (claude-275): each query's outcome, from `texts`, the decoded queries'
/// (reply, reference) texts each cut at its end marker, in `prompts.decoded`'s order. A query
/// states its answer when its reply does as a whole word ([`states_answer`]). Refused when the
/// texts do not match the decoded queries.
fn recall_grade(
    queries: &[ChainQuery],
    prompts: &QueryPrompts,
    texts: &[(String, String)],
) -> Result<(RecallBin, Vec<RecallBin>, Vec<QueryOutcome>)> {
    if texts.len() != prompts.decoded.len() {
        return Err(invalid(format!(
            "{} replies for {} decoded queries",
            texts.len(),
            prompts.decoded.len()
        )));
    }
    let mut outcomes: Vec<QueryOutcome> = queries
        .iter()
        .enumerate()
        .map(|(query, q)| QueryOutcome {
            query,
            position: q.reply_start,
            distance: q.chain.distance,
            decoded: false,
            whole_document: false,
            reference_states: false,
            stated: false,
        })
        .collect();
    for ((index, window), (reply, reference)) in prompts.decoded.iter().zip(texts) {
        let (query, outcome) = (&queries[*index], &mut outcomes[*index]);
        outcome.decoded = true;
        outcome.whole_document = window.input_ids.first() == Some(&hs::SHEET_BOS_ID)
            && window.scored_corpus_positions[0] - 1 - window.scored_offsets[0] == query.document_start;
        outcome.reference_states = states_answer(reference, &query.answer);
        outcome.stated = states_answer(reply, &query.answer);
    }
    let count = |from: usize, to: Option<usize>| {
        let mut bin = RecallBin { from, to, ..RecallBin::default() };
        for o in outcomes.iter().filter(|o| o.distance >= from && to.is_none_or(|t| o.distance < t)) {
            bin.queries += 1;
            bin.decoded += usize::from(o.decoded);
            bin.whole_document += usize::from(o.whole_document);
            bin.reference_states += usize::from(o.reference_states);
            bin.stated += usize::from(o.stated);
        }
        bin
    };
    let bins = RECALL_BIN_EDGES
        .iter()
        .enumerate()
        .map(|(i, &from)| count(from, RECALL_BIN_EDGES.get(i + 1).copied()))
        .collect();
    Ok((count(0, None), bins, outcomes))
}

/// A queries panel's answers against TRAIN's replies (claude-275): how many TRAIN states as
/// words of a reply. An answer is a number (all digits) or a word; it is found when each of its
/// alphanumeric pieces is a word of some TRAIN reply. A small number is expected in TRAIN by
/// chance, so numbers and words are counted apart.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AnswerOverlap {
    pub panel: String,
    pub answers: usize,
    pub distinct: usize,
    pub words: usize,
    pub words_in_train: usize,
    pub numbers: usize,
    pub numbers_in_train: usize,
    /// Up to 16 of the distinct answers found, in order.
    pub found: Vec<String>,
    /// Every query whose answer is found, by its index in the panel's `queries.json`, in order
    /// (claude-281): the queries a reading sets aside.
    pub found_queries: Vec<usize>,
}

/// The alphanumeric pieces of `text`.
fn words_of(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty())
}

fn answer_overlap(
    panel: &str,
    answers: &[String],
    train_words: &std::collections::HashSet<String>,
) -> AnswerOverlap {
    let distinct: std::collections::BTreeSet<&str> = answers.iter().map(String::as_str).collect();
    let mut overlap = AnswerOverlap {
        panel: panel.to_owned(),
        answers: answers.len(),
        distinct: distinct.len(),
        words: 0,
        words_in_train: 0,
        numbers: 0,
        numbers_in_train: 0,
        found: Vec::new(),
        found_queries: Vec::new(),
    };
    for (query, answer) in answers.iter().enumerate() {
        let number = answer.chars().all(|c| c.is_ascii_digit());
        let mut pieces = words_of(answer).peekable();
        let found = pieces.peek().is_some() && pieces.all(|w| train_words.contains(w));
        if number {
            overlap.numbers += 1;
            overlap.numbers_in_train += usize::from(found);
        } else {
            overlap.words += 1;
            overlap.words_in_train += usize::from(found);
        }
        if found {
            overlap.found_queries.push(query);
        }
        if found && overlap.found.len() < 16 && !overlap.found.contains(answer) {
            overlap.found.push(answer.clone());
        }
    }
    overlap
}

fn greedy_reply_with_mode(
    model: &StackModel,
    objective: PointObjective,
    prompt: &[u32],
    reply_tokens: usize,
    seed: u64,
    served: bool,
) -> Result<Vec<u32>> {
    Ok(decode_reply_with_mode(
        model,
        objective,
        prompt,
        reply_tokens,
        seed,
        DecodeRule::Greedy,
        0,
        None,
        ReadScope::Full,
        &[],
        served,
    )?
    .tokens)
}

/// The native writer's next token from the last row of `hidden`, rounded to the served Q12 grid
/// when `served`.
fn native_writer_next_token(model: &StackModel, hidden: &Tensor, served: bool) -> Result<u32> {
    let (rows, _) = hidden.dims2()?;
    let point = hidden.narrow(0, rows.saturating_sub(1), 1)?;
    let point = if served {
        hs::q12_query(&point)?
    } else {
        point
    };
    model
        .native_vector_writer()?
        .write(&point)?
        .flatten_all()?
        .to_dtype(DType::U32)?
        .to_vec1::<u32>()?
        .last()
        .copied()
        .ok_or_else(|| invalid("the writer returned no token"))
}

/// How a reply picks each next token (one-model-end-to-end protocol, "The decode rules").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeRule {
    /// The whole table's nearest token, as [`greedy_reply`] decodes.
    Greedy,
    /// The Flock rank rule (protocol I-E): the `k` nearest table tokens (1 to
    /// [`FLOCK_DECODE_K`]), ranked, and a rank drawn with the harmonic weights of
    /// `stack/flock.rs` ([`flock_rank_draw`]). The served rule is [`DecodeRule::FLOCK`]; `k = 1`
    /// is I-E's check that it decodes as greedy. With `keep_end`, a nearest `<|eos|>` is taken
    /// ([`flock_pick`]).
    FlockRank { k: usize, keep_end: bool },
    /// The Flock rank rule inside the nearest token's cell (claude-213): of the `k` nearest,
    /// only those whose score is within `margin_milli / 1000` of the nearest's are drawn among
    /// ([`cell_admitted`]), with the harmonic weights over that count. A nearest `<|eos|>` is
    /// kept, as under [`DecodeRule::FLOCK_END`]. Margin 0 admits only exact ties, so it
    /// decodes as greedy.
    FlockCell { k: usize, margin_milli: u32 },
    /// The cycle step (claude-217, Casey 30 Oct: "steps off a reply's own cycle"): of the `k`
    /// nearest, in rank order, the first whose append would not close an `n`-gram the reply has
    /// already produced ([`cycle_step_pick`]); the nearest when every one would. No draw: it is
    /// exact and deterministic. It stops after the end of turn, as the Flock rules do. Casey's
    /// rule is `n = 4`; `n = 2`, the echo step, acts on a repeated token's second repeat.
    CycleStep { k: usize, n: usize },
}

impl DecodeRule {
    /// The served Flock rank rule, over [`FLOCK_DECODE_K`] candidates, as the protocol registers it.
    pub const FLOCK: Self = Self::FlockRank { k: FLOCK_DECODE_K, keep_end: false };
    /// The Flock rank rule that keeps the end of turn (claude-211, Casey 30 Oct: "do not rule
    /// flock out, improve it instead"): the draw varies the reply's tokens, not where its turn
    /// ends. A nearest end of turn is always taken; a farther one can still be drawn, as
    /// any other candidate is.
    pub const FLOCK_END: Self = Self::FlockRank { k: FLOCK_DECODE_K, keep_end: true };

    /// [`DecodeRule::FlockCell`] over [`FLOCK_DECODE_K`] candidates at `margin_milli`.
    pub const fn flock_cell(margin_milli: u32) -> Self {
        Self::FlockCell { k: FLOCK_DECODE_K, margin_milli }
    }

    /// The cycle step over [`FLOCK_DECODE_K`] candidates and 4-grams, as Casey directed and
    /// claude-217 registers it.
    pub const CYCLE_STEP: Self = Self::CycleStep { k: FLOCK_DECODE_K, n: 4 };
    /// The echo step: the same rule on bigrams, so a token repeats at most twice in a row
    /// (claude-217, reported: 77–89% of greedy's loops repeat one token).
    pub const ECHO_STEP: Self = Self::CycleStep { k: FLOCK_DECODE_K, n: 2 };
}

/// [`DecodeRule::CycleStep`]'s token from its `ranked` candidates (the nearest first), after
/// `reply` (claude-217): the first whose append would not close an `n`-gram `reply` already
/// holds (`n ≥ 2`), and its mark: [`STEP_NEAREST`] when that is the nearest, [`STEP_OFF`] when
/// the rule stepped off the nearest, [`STEP_EXHAUSTED`] when every candidate would close one and
/// the nearest is taken. `None` with no candidate.
fn cycle_step_pick(reply: &[u32], ranked: &[u32], n: usize) -> Option<(u32, u8)> {
    let closes = |token: u32| {
        let keep = n.saturating_sub(1);
        n >= 2 && reply.len() >= keep && {
            let tail = &reply[reply.len() - keep..];
            reply.windows(n).any(|w| w[..keep] == *tail && w[keep] == token)
        }
    };
    let nearest = *ranked.first()?;
    Some(match ranked.iter().position(|&t| !closes(t)) {
        Some(0) => (nearest, STEP_NEAREST),
        Some(i) => (ranked[i], STEP_OFF),
        None => (nearest, STEP_EXHAUSTED),
    })
}

/// The name a [`DecodeRule::CycleStep`] file carries for its gram size `n`: `cycle-step` for
/// Casey's 4-grams, `echo-step` for bigrams, else `cycle-step-{n}`.
fn step_name(n: usize) -> String {
    match n {
        4 => "cycle-step".to_owned(),
        2 => "echo-step".to_owned(),
        n => format!("cycle-step-{n}"),
    }
}

/// [`cycle_step_pick`]'s marks, one a decoded step ([`IdSample::stepped`]).
pub const STEP_NEAREST: u8 = 0;
pub const STEP_OFF: u8 = 1;
pub const STEP_EXHAUSTED: u8 = 2;

/// The cell objective's margin in thousandths of a score: `rival_hinge`'s
/// `relu(1 + max_r s(r) − s(target))` asks the target to keep this gap over its nearest rival,
/// so a rival nearer than it is one the model has not separated (claude-213).
pub const CELL_MARGIN_MILLI: u32 = 1_000;

/// The margins claude-213 decodes under [`DecodeRule::FlockCell`], in thousandths: the cell
/// objective's own, then halving toward greedy.
pub const FLOCK_CELL_LADDER_MILLI: [u32; 6] = [CELL_MARGIN_MILLI, 500, 250, 125, 62, 31];

/// Whether `scopes` can name `--reply-read`'s scopes: distinct (each names its own file), and
/// never [`ReadScope::Full`], which is greedy's own file (claude-215).
pub fn scopes_ok(scopes: &[ReadScope]) -> bool {
    let distinct: std::collections::BTreeSet<&str> = scopes.iter().map(|s| s.name()).collect();
    distinct.len() == scopes.len() && !scopes.contains(&ReadScope::Full)
}

/// Whether `cells` can name `--flock-cell`'s margins: distinct (each names its own file), and
/// none past four of the cell objective's margins.
pub fn cells_ok(cells: &[u32]) -> bool {
    let distinct: std::collections::BTreeSet<&u32> = cells.iter().collect();
    distinct.len() == cells.len() && cells.iter().all(|&m| m <= 4 * CELL_MARGIN_MILLI)
}

/// How many of `ranked` (in rank order, the nearest first) [`DecodeRule::FlockCell`] admits:
/// those whose score is within `margin_milli / 1000` of the nearest's, never fewer than one.
fn cell_admitted(ranked: &[(u32, f32)], margin_milli: u32) -> usize {
    let Some(&(_, nearest)) = ranked.first() else {
        return 0;
    };
    let margin = margin_milli as f32 / 1000.0;
    ranked
        .iter()
        .take_while(|&&(_, s)| nearest - s <= margin)
        .count()
        .max(1)
}

/// The Flock rank rule's token from its `ranked` candidates and the drawn rank `draw`: the
/// drawn candidate, except that with `keep_end` a nearest `eos` is taken. The registered rule
/// takes the nearest `<|eos|>` only when its draw lands on rank 0, `W_0 / H` (0.30) of the
/// time, so a reply can run past its end (claude-211: 56 of seed 1's 256 DEV replies ran to 64
/// tokens).
fn flock_pick(ranked: &[u32], draw: usize, keep_end: bool, eos: Option<u32>) -> u32 {
    if keep_end && ranked.first().copied() == eos && eos.is_some() {
        ranked[0]
    } else {
        ranked[draw.min(ranked.len() - 1)]
    }
}

/// The Flock rank rule's candidates: the Hamming read's kept count, and one of the Flock
/// selector's top-k values.
pub const FLOCK_DECODE_K: usize = 16;

/// The first [`FLOCK_DECODE_K`] of `uor_r4_integer::stack::flock::RAW_RANK_WEIGHTS_Q16`, the
/// integer Flock selector's harmonic rank weights `1/(i+1)` in Q16 (`stack/flock.rs:382`). The
/// integer crate's native root does not export `stack`, so they are copied here, and a test
/// reads that file and fails if the two ever differ.
pub const FLOCK_RANK_WEIGHTS_Q16: [u16; FLOCK_DECODE_K] = [
    65535, 32768, 21845, 16384, 13107, 10922, 9362, 8192, 7281, 6553, 5957, 5461, 5041, 4681, 4369,
    4096,
];

/// The rank the Flock rank rule draws among `k` candidates (protocol I-E): one xorshift64* step,
/// seeded as [`epoch_order`] seeds (`state = seed·φ + index·κ`, then `| 1`, never zero); `r =
/// output mod H`, with `H` the sum of the first `k` harmonic weights
/// [`FLOCK_RANK_WEIGHTS_Q16`] (`1/(i+1)` in Q16); and the first rank
/// whose running weight exceeds `r`. No exponential; `k = 1` always draws rank 0.
pub fn flock_rank_draw(seed: u64, index: u64, k: usize) -> usize {
    let weights = &FLOCK_RANK_WEIGHTS_Q16[..k.clamp(1, FLOCK_RANK_WEIGHTS_Q16.len())];
    let total: u64 = weights.iter().map(|&w| u64::from(w)).sum();
    let mut state = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(index.wrapping_mul(0xD1B5_4A32_D192_ED03))
        | 1;
    state ^= state >> 12;
    state ^= state << 25;
    state ^= state >> 27;
    harmonic_rank(state.wrapping_mul(0x2545_F491_4F6C_DD1D) % total, weights)
}

/// The rank a draw `r` (below the weights' sum) lands on: the first whose running weight
/// exceeds `r`, so rank `i` takes exactly `W_i` of the draws.
fn harmonic_rank(r: u64, weights: &[u16]) -> usize {
    let mut running = 0u64;
    for (i, &w) in weights.iter().enumerate() {
        running += u64::from(w);
        if running > r {
            return i;
        }
    }
    weights.len() - 1
}

/// The longest reply the Flock rank rule draws for: its draw index is `sample · 64 + step`, so a
/// longer reply would reuse the next sample's draws.
pub const FLOCK_MAX_REPLY: usize = 64;

/// `reply` up to and including its first `eos`, the reply as the protocol counts it.
fn cut_at_eos(reply: &[u32], eos: u32) -> &[u32] {
    match reply.iter().position(|&t| t == eos) {
        Some(at) => &reply[..=at],
        None => reply,
    }
}

/// `(id, score)` candidates in rank order: the higher score first, ties to the smaller id.
fn rank_order(mut candidates: Vec<(u32, f32)>) -> Vec<(u32, f32)> {
    candidates.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    candidates
}

/// Where a prompt of `prompt_len` tokens ends in a decode window that starts at sequence
/// position `start` (claude-215): window positions below it are the prompt's. The window holds
/// the last of the model's own window of tokens (claude-274; before, the last
/// [`SHEET_CONTEXT`]), so once the reply has pushed `start` past the prompt's
/// start the cut moves down with it, and once the prompt has scrolled out it is 0.
fn prompt_cut(prompt_len: usize, start: usize) -> usize {
    prompt_len.saturating_sub(start)
}

/// The response-mask flags of a decode window that starts at sequence position `start`, over a
/// prompt flagged `prompt_mask` and the `reply_len` tokens decoded so far, each a reply token
/// (claude-216): [`reply_row_limits`] then limits the window as it would a training window, the
/// reply's rows to [`prompt_cut`].
fn decode_window_mask(prompt_mask: &[u8], reply_len: usize, start: usize) -> Vec<u8> {
    prompt_mask
        .iter()
        .copied()
        .chain(std::iter::repeat_n(1, reply_len))
        .skip(start)
        .collect()
}

/// One decoded reply ([`decode_reply`]).
struct DecodedReply {
    tokens: Vec<u32>,
    /// Under [`DecodeRule::FlockCell`], each drawn step's admitted count, a final step that
    /// drew BOS included; empty under any other rule.
    admitted: Vec<u8>,
    /// Under [`DecodeRule::CycleStep`], each step's mark ([`cycle_step_pick`]), a final step
    /// that picked BOS included; empty under any other rule.
    stepped: Vec<u8>,
    /// The served point key's first token and its least L1 distance ([`served_opening`], claude-220), where it gave one.
    served: Option<(u32, f32)>,
}

/// The served point key's table (claude-220): TRAIN's opening points under the trunk, on the
/// model's device, and their targets, as [`opening_points`] builds them and `opening-table.safetensors`
/// holds them.
#[derive(Clone)]
struct ServedKey {
    points: Tensor,
    targets: std::sync::Arc<Vec<u32>>,
}

thread_local! {
    static FORCED_OPENING: std::cell::Cell<Option<u32>> = const { std::cell::Cell::new(None) };
    static SERVED_KEY: std::cell::RefCell<Option<ServedKey>> = const { std::cell::RefCell::new(None) };
    /// The served key's searches on this thread since the last [`take_served_searches`]: how
    /// many, and their wall time in microseconds.
    static SERVED_SEARCHES: std::cell::Cell<(u64, u64)> = const { std::cell::Cell::new((0, 0)) };
}

/// The served point key for the next [`decode_reply`] calls on this thread, or none.
fn set_served_key(key: Option<ServedKey>) {
    SERVED_KEY.with(|k| *k.borrow_mut() = key);
}

/// The served key's searches since the last call, `(count, microseconds)`, and a reset.
fn take_served_searches() -> (u64, u64) {
    SERVED_SEARCHES.with(|c| c.replace((0, 0)))
}

/// The batched decode's forwards (claude-279): the step cache's prefills, its steps, and the
/// whole-window forwards that every other step re-runs. `first_refusal` names the kind of the
/// first refusal ([`crate::geometric_stack::CacheRefusal::kind`]) that kept a group from
/// prefilling or dropped its cache,
/// a window slide included (claude-281).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct DecodeForwards {
    pub prefills: u64,
    pub steps: u64,
    pub whole: u64,
    pub first_refusal: Option<&'static str>,
}

thread_local! {
    /// Whether the batched decode uses the exact step cache on this thread (claude-279).
    static DECODE_CACHE: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
    static DECODE_FORWARDS: std::cell::Cell<DecodeForwards> = const {
        std::cell::Cell::new(DecodeForwards { prefills: 0, steps: 0, whole: 0, first_refusal: None })
    };
}

/// Whether [`decode_replies_with_mode`]'s batched path steps with the exact step cache
/// (`StackModel::step_cached`, claude-278) on this thread, where each step's window fits; on by
/// default (claude-279). Off, every step re-runs the whole window, as before.
pub fn set_decode_cache(on: bool) {
    DECODE_CACHE.with(|c| c.set(on));
}

/// The batched decode's forwards on this thread since the last call, and a reset.
pub fn take_decode_forwards() -> DecodeForwards {
    DECODE_FORWARDS.with(|c| c.replace(DecodeForwards::default()))
}

/// Record `refusal` as the pass's first, unless one is recorded already.
fn note_refusal(refusal: &crate::geometric_stack::CacheRefusal) {
    count_forward(|counts| {
        counts.first_refusal.get_or_insert(refusal.kind());
    });
}

fn count_forward(add: impl FnOnce(&mut DecodeForwards)) {
    DECODE_FORWARDS.with(|c| {
        let mut counts = c.get();
        add(&mut counts);
        c.set(counts);
    });
}

/// The served point key's first token (claude-220) for the opening that ends `window`, from the
/// opening row's own `point` (`(1, width)`, from the decode's forward). Where the turn memory's
/// exact key misses the opening (the memory moves nothing there, so `point` is the trunk's), it
/// is the commonest target among the served table's L1-nearest TRAIN openings, ties to the
/// smaller id, as [`l1_named`] names them. None where no key is served, the memory is off, or
/// the exact key fires. A key naming BOS is an instrument fault.
fn served_opening(window: &[u32], point: &Tensor) -> Result<Option<(u32, f32)>> {
    let Some(key) = SERVED_KEY.with(|k| k.borrow().clone()) else {
        return Ok(None);
    };
    let Some(memory) = crate::flock_read_memory::turn_memory() else {
        return Ok(None);
    };
    let last = window.len() - 1;
    if memory.stored(window, window.len(), &[last])?.1[last] == 1.0 {
        return Ok(None);
    }
    // The forward's kernels finish first, so the timer holds the search alone.
    point.device().synchronize()?;
    let started = Instant::now();
    let named = l1_named(point, &key.points, &key.targets)?;
    let micros = started.elapsed().as_micros() as u64;
    SERVED_SEARCHES.with(|c| {
        let (n, us) = c.get();
        c.set((n + 1, us + micros));
    });
    let first = named.first().ok_or_else(|| invalid("the served key named nothing"))?;
    if first.target == hs::SHEET_BOS_ID {
        return Err(invalid("the served key names BOS (an instrument fault)"));
    }
    if !first.distance.is_finite() {
        return Err(invalid("the served key's distance is not finite (an instrument fault)"));
    }
    Ok(Some((first.target, first.distance)))
}

/// Whether the served point key may name the next token of `reply` under `rule` (claude-220):
/// at the reply's first step, under the greedy rule only.
fn serves_at(reply: &[u32], rule: DecodeRule) -> bool {
    reply.is_empty() && rule == DecodeRule::Greedy
}

/// The reply's first token for the next [`decode_reply`] calls on this thread, or none: the
/// forced-opening arm (claude-218) gives each prompt its reference's first token.
fn set_forced_opening(token: Option<u32>) {
    FORCED_OPENING.with(|c| c.set(token));
}

fn forced_opening() -> Option<u32> {
    FORCED_OPENING.with(|c| c.get())
}

/// The model's reply to `prompt` under `rule`: as [`greedy_reply`], with the next token picked
/// by `rule`. The Flock rank rule also stops after `eos` (the protocol); greedy decodes to
/// BOS or its length, as before. `sample` is the reply's index, which with `seed` seeds the Flock rank draws: draw
/// `index = sample · 64 + step`. The Flock rank rule needs a nearest objective. `scope` limits
/// which positions the Hamming read may choose at each step (claude-215): under any scope but
/// [`ReadScope::Full`], the limited heads of every reply row in the window read only before
/// their reply's first token, training's rule ([`reply_row_limits`], claude-216), on the
/// prompt's response-mask flags `prompt_mask` and the reply's tokens; so the reply decoded reads
/// only the prompt's positions, and an earlier reply in the prompt reads only before itself.
#[allow(clippy::too_many_arguments)]
fn decode_reply_with_mode(
    model: &StackModel,
    objective: PointObjective,
    prompt: &[u32],
    reply_tokens: usize,
    seed: u64,
    rule: DecodeRule,
    sample: usize,
    eos: Option<u32>,
    scope: ReadScope,
    prompt_mask: &[u8],
    served_mode: bool,
) -> Result<DecodedReply> {
    let flock = rule != DecodeRule::Greedy;
    if flock && !objective.decodes_nearest() {
        return Err(invalid("the Flock rank rule decodes the nearest table tokens only"));
    }
    // The read's scope is set on the nearest decode's forward only; on the writer's it would
    // limit nothing, and its files would be plain greedy under another name.
    if scope != ReadScope::Full && !objective.decodes_nearest() {
        return Err(invalid("a read scope decodes the nearest table tokens only"));
    }
    if scope != ReadScope::Full && prompt_mask.len() != prompt.len() {
        return Err(invalid("a read scope needs the prompt's response mask, one flag a token"));
    }
    // The reply opens after the prompt's last token, so its rows read only before
    // [`prompt_cut`], claude-215's cut; a prompt that ended inside a reply would merge the two.
    if scope != ReadScope::Full && prompt_mask.last() != Some(&0) {
        return Err(invalid("a scoped decode's prompt ends inside a reply"));
    }
    if flock && reply_tokens > FLOCK_MAX_REPLY {
        return Err(invalid(format!(
            "the Flock rank rule draws for replies of at most {FLOCK_MAX_REPLY} tokens"
        )));
    }
    let table = table(model)?;
    // The decode slides over the model's own window (claude-274; before, always the sheet's).
    let context = model.config.context;
    with_eval_forward(served_mode, seed, 0, || {
        let mut ids = prompt.to_vec();
        let mut reply = Vec::with_capacity(reply_tokens);
        let mut admitted = Vec::new();
        let mut stepped = Vec::new();
        let mut served = None;
        for step in 0..reply_tokens {
            // A forced opening (claude-218) is the reply's first token, taken without a forward,
            // so no opening row is read or moved for it.
            if let Some(token) = forced_opening().filter(|_| reply.is_empty()) {
                // As a decoded token would: BOS ends the reply unkept, and an end of turn is kept
                // and ends it under the Flock rules only.
                if token == hs::SHEET_BOS_ID {
                    break;
                }
                reply.push(token);
                ids.push(token);
                if flock && Some(token) == eos {
                    break;
                }
                continue;
            }
            let window = &ids[ids.len().saturating_sub(context)..];
            let token = if objective.decodes_nearest() {
                // The reply's first step is its opening row, the one the turn memory moves.
                if reply.is_empty()
                    && !window.is_empty()
                    && crate::flock_read_memory::turn_memory().is_some()
                {
                    crate::flock_read_memory::set_memory_rows(Some(vec![window.len() - 1]));
                }
                // A later step reads the reply table (claude-193), keyed by the turn at the prompt's
                // last token, the reply's length and its last id.
                let start = ids.len().saturating_sub(context);
                let cut = prompt_cut(prompt.len(), start);
                if !reply.is_empty()
                    && cut > 0
                    && crate::flock_read_memory::turn_memory()
                        .is_some_and(|m| m.reply_len().is_some())
                {
                    if let Some(key) = greedy_reply_key(window, cut - 1, &reply) {
                        crate::flock_read_memory::set_reply_rows(Some(vec![(
                            window.len() - 1,
                            key,
                        )]));
                    }
                }
                // Under a scope, every reply row's limited heads choose only before its reply, so
                // the reply decoded never reads its own tokens, as in training.
                if scope != ReadScope::Full {
                    let flags = decode_window_mask(prompt_mask, reply.len(), start);
                    crate::flock_read_hamming::set_row_cut(Some((
                        scope,
                        reply_row_limits(&flags, flags.len()),
                    )));
                }
                let hidden = crate::flock_read::points(model, window, 1, window.len());
                crate::flock_read_hamming::set_row_cut(None);
                crate::flock_read_memory::set_memory_rows(None);
                crate::flock_read_memory::set_reply_rows(None);
                let hidden = hidden?;
                let point = hidden.narrow(0, window.len() - 1, 1)?;
                let point = if served_mode {
                    hs::q12_query(&point)?
                } else {
                    point
                };
                // The served point key (claude-220) names the opening where the exact key misses,
                // from this forward's own point, under the greedy rule only.
                if serves_at(&reply, rule) {
                    served = served_opening(window, &point)?;
                }
                let scores = nearest_scores(&point, &table)?;
                match rule {
                    DecodeRule::Greedy if reply.is_empty() && served.is_some() => {
                        served.map(|(token, _)| token).ok_or_else(|| invalid("the served key gave no token"))?
                    }
                    DecodeRule::Greedy => *scores
                        .argmax(1)?
                        .to_vec1::<u32>()?
                        .last()
                        .ok_or_else(|| invalid("the readout returned no token"))?,
                    DecodeRule::FlockRank { k, .. }
                    | DecodeRule::FlockCell { k, .. }
                    | DecodeRule::CycleStep { k, .. } => {
                        // The K nearest on the GPU; their order, ties to the smaller id, here.
                        let k = k.clamp(1, FLOCK_DECODE_K).min(scores.dim(1)?);
                        let top = scores
                            .arg_sort_last_dim(false)?
                            .narrow(1, 0, k)?
                            .contiguous()?;
                        let picked = scores.gather(&top, 1)?.flatten_all()?.to_vec1::<f32>()?;
                        let ids_k = top.flatten_all()?.to_vec1::<u32>()?;
                        let scored = rank_order(ids_k.into_iter().zip(picked).collect());
                        if let DecodeRule::CycleStep { n, .. } = rule {
                            // The cycle step takes the nearest that closes no repeated n-gram.
                            let ranked: Vec<u32> = scored.iter().map(|&(id, _)| id).collect();
                            let (token, mark) = cycle_step_pick(&reply, &ranked, n)
                                .ok_or_else(|| invalid("the cycle step had no candidate"))?;
                            stepped.push(mark);
                            token
                        } else {
                            // The rank rule draws among all K; the cell rule among those it
                            // admits.
                            let (drawn, keep_end) = match rule {
                                DecodeRule::FlockCell { margin_milli, .. } => {
                                    let n = cell_admitted(&scored, margin_milli);
                                    admitted.push(n as u8);
                                    (n, true)
                                }
                                DecodeRule::FlockRank { keep_end, .. } => (scored.len(), keep_end),
                                DecodeRule::Greedy | DecodeRule::CycleStep { .. } => (1, false),
                            };
                            let ranked: Vec<u32> =
                                scored[..drawn].iter().map(|&(id, _)| id).collect();
                            let index = (sample as u64) * 64 + step as u64;
                            let draw = flock_rank_draw(seed, index, ranked.len());
                            flock_pick(&ranked, draw, keep_end, eos)
                        }
                    }
                }
            } else {
                let hidden = model.hidden(window, 1, window.len())?;
                native_writer_next_token(model, &hidden, served_mode)?
            };
            if token == hs::SHEET_BOS_ID {
                break;
            }
            ids.push(token);
            reply.push(token);
            // The Flock rank rule stops at its reply's end of turn (the protocol); greedy keeps
            // decoding to BOS or its length, as it always has.
            if flock && Some(token) == eos {
                break;
            }
        }
        Ok(DecodedReply {
            tokens: reply,
            admitted,
            stepped,
            served,
        })
    })
}

/// The same prompts in rectangular batches of exactly equal prompt length.
///
/// A first scorer unit batches only whole-table nearest decoding with the full read, no forced
/// opening and no served point key. Everything else keeps [`decode_reply_with_mode`]'s
/// one-at-a-time path. Equal lengths avoid padding, which would change the recurrent state; each
/// group is split by the evaluation's established row/token budget, `SHEET_EVAL_BATCH` windows
/// of [`SHEET_CONTEXT`], at the widest window a member reaches, which the model's own `context`
/// caps (claude-274). The budget stays in positions because a forward's memory grows with them.
fn exact_length_batches(prompts: &[Vec<u32>], reply_tokens: usize, context: usize) -> Vec<Vec<usize>> {
    let budget = hs::SHEET_EVAL_BATCH * SHEET_CONTEXT;
    let mut by_length: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (index, prompt) in prompts.iter().enumerate() {
        by_length.entry(prompt.len()).or_default().push(index);
    }
    let mut groups = Vec::new();
    for indices in by_length.into_values() {
        let width = prompts[indices[0]].len();
        // A group is split by the widest window any member can reach, not only by its prompt.
        let final_width = width
            .saturating_add(reply_tokens)
            .clamp(1, context);
        let per_group = budget
            .checked_div(final_width)
            .map_or(indices.len(), |count| count.max(1));
        for chunk in indices.chunks(per_group) {
            groups.push(chunk.to_vec());
        }
    }
    groups
}

/// Whether a decode may take the batched path. The caller supplies the two per-thread controls,
/// so a forced opening or served key always keeps the serial path.
fn can_batch_decode(
    enabled: bool,
    objective: PointObjective,
    scope: ReadScope,
    forced: bool,
    served_key: bool,
) -> bool {
    enabled && objective.decodes_nearest() && scope == ReadScope::Full && !forced && !served_key
}

struct BatchReply {
    index: usize,
    ids: Vec<u32>,
    tokens: Vec<u32>,
    admitted: Vec<u8>,
    stepped: Vec<u8>,
    active: bool,
}

/// Decode `prompts` with [`decode_reply_with_mode`], batching only same-length, unscoped,
/// unforced nearest decodes. Results stay in prompt order and Flock draws keep each prompt's
/// original sample index.
#[allow(clippy::too_many_arguments)]
fn decode_replies_with_mode(
    model: &StackModel,
    objective: PointObjective,
    prompts: &[Vec<u32>],
    prompt_masks: &[Vec<u8>],
    reply_tokens: usize,
    seed: u64,
    rule: DecodeRule,
    eos: Option<u32>,
    scope: ReadScope,
    served_mode: bool,
    batched_decode: bool,
) -> Result<Vec<DecodedReply>> {
    if prompts.len() != prompt_masks.len() {
        return Err(invalid("the decode prompts and masks are not one per sample"));
    }
    let flock = rule != DecodeRule::Greedy;
    if flock && !objective.decodes_nearest() {
        return Err(invalid("the Flock rank rule decodes the nearest table tokens only"));
    }
    if scope != ReadScope::Full && !objective.decodes_nearest() {
        return Err(invalid("a read scope decodes the nearest table tokens only"));
    }
    if flock && reply_tokens > FLOCK_MAX_REPLY {
        return Err(invalid(format!(
            "the Flock rank rule draws for replies of at most {FLOCK_MAX_REPLY} tokens"
        )));
    }
    let can_batch = can_batch_decode(
        batched_decode,
        objective,
        scope,
        forced_opening().is_some(),
        SERVED_KEY.with(|key| key.borrow().is_some()),
    );
    if !can_batch {
        return prompts
            .iter()
            .zip(prompt_masks)
            .enumerate()
            .map(|(sample, (prompt, mask))| {
                decode_reply_with_mode(
                    model,
                    objective,
                    prompt,
                    reply_tokens,
                    seed,
                    rule,
                    sample,
                    eos,
                    scope,
                    mask,
                    served_mode,
                )
            })
            .collect();
    }

    let table = table(model)?;
    let memory = crate::flock_read_memory::turn_memory();
    let mut replies: Vec<Option<DecodedReply>> = (0..prompts.len()).map(|_| None).collect();
    // The model's own window, as the serial decode slides over it (claude-274).
    let context = model.config.context;
    for group in exact_length_batches(prompts, reply_tokens, context) {
        let width = prompts[group[0]].len();
        if width == 0 {
            return Err(invalid("a batched decode needs a non-empty prompt"));
        }
        let batch = group.len();
        let mut state: Vec<BatchReply> = group
            .iter()
            .map(|&index| BatchReply {
                index,
                ids: prompts[index].clone(),
                tokens: Vec::with_capacity(reply_tokens),
                admitted: Vec::new(),
                stepped: Vec::new(),
                active: true,
            })
            .collect();
        // The exact step cache (claude-279): the prompt's forward fills it, and each later step
        // whose window fits computes its one position from it. The first step it refuses (the
        // window would slide, or a read or instrument it does not carry) drops it, and every
        // step after re-runs the whole window, as before.
        let mut cache: Option<crate::geometric_stack::DecodeCache> = None;
        let caching = DECODE_CACHE.with(|c| c.get());
        with_eval_forward(served_mode, seed, 0, || {
            for step in 0..reply_tokens {
                if state.iter().all(|row| !row.active) {
                    break;
                }
                let time = state[0].ids.len();
                if state.iter().any(|row| row.ids.len() != time) {
                    return Err(invalid("a batched decode lost its rectangular history"));
                }
                let window_len = time.min(context);
                let start = time.saturating_sub(context);
                let flat: Vec<u32> = state
                    .iter()
                    .flat_map(|row| row.ids[start..].iter().copied())
                    .collect();
                // Each row's newest id, the one position a cached step computes.
                let step_ids: Vec<u32> = state.iter().map(|row| row.ids[time - 1]).collect();
                // The decode's forwards run without a row cut, so a cut left on the thread must
                // not refuse the cache before it is cleared (claude-281).
                crate::flock_read_hamming::set_row_cut(None);
                if let Some(refusal) = cache
                    .as_ref()
                    .and_then(|c| model.step_refusal(c, &step_ids))
                {
                    note_refusal(&refusal);
                    cache = None;
                }
                let stepping = step > 0 && cache.is_some();
                if cache.as_ref().is_some_and(|c| c.position() + 1 != time) {
                    return Err(invalid("the decode cache fell out of step with its rows"));
                }
                // A prefill pays only where a later step can fit.
                let prefilling = caching
                    && step == 0
                    && time < context
                    && reply_tokens > 1
                    && match model.prefill_refusal(&flat, batch, window_len) {
                        Some(refusal) => {
                            note_refusal(&refusal);
                            false
                        }
                        None => true,
                    };

                let mut openings = Vec::new();
                let mut reply_rows = Vec::new();
                for (b, row) in state.iter().enumerate() {
                    if !row.active {
                        continue;
                    }
                    let global_row = b * window_len + window_len - 1;
                    // A step counts its rows as its own: row `b` (claude-278).
                    let named_row = if stepping { b } else { global_row };
                    if step == 0 && memory.is_some() {
                        openings.push(global_row);
                    } else if let Some(memory) = &memory {
                        let cut = prompt_cut(width, start);
                        if cut > 0 && memory.reply_len().is_some() {
                            if let Some(key) =
                                greedy_reply_key(&row.ids[start..], cut - 1, &row.tokens)
                            {
                                reply_rows.push((named_row, key));
                            }
                        }
                    }
                }
                crate::flock_read_memory::set_memory_rows(
                    (!openings.is_empty()).then_some(openings),
                );
                crate::flock_read_memory::set_reply_rows(
                    (!reply_rows.is_empty()).then_some(reply_rows),
                );
                let forward =
                    match cache.as_mut() {
                        Some(filled) if stepping => model.step_cached(filled, &step_ids).map(Some),
                        _ if prefilling => model.prefill_cached(&flat, batch, window_len).map(
                            |(hidden, filled)| {
                                cache = Some(filled);
                                Some(hidden)
                            },
                        ),
                        _ => crate::flock_read::points(model, &flat, batch, window_len).map(Some),
                    };
                crate::flock_read_memory::set_memory_rows(None);
                crate::flock_read_memory::set_reply_rows(None);
                crate::flock_read_hamming::set_row_cut(None);
                let hidden =
                    forward?.ok_or_else(|| invalid("a decode forward returned nothing"))?;
                count_forward(|counts| match (stepping, prefilling) {
                    (true, _) => counts.steps += 1,
                    (false, true) => counts.prefills += 1,
                    (false, false) => counts.whole += 1,
                });
                let point = if stepping {
                    // The step's points are already one row per batch row.
                    hidden.contiguous()?
                } else {
                    let last_rows: Vec<u32> = (0..batch)
                        .map(|b| (b * window_len + window_len - 1) as u32)
                        .collect();
                    hidden.index_select(&Tensor::from_vec(last_rows, batch, model.device())?, 0)?
                };
                let point = if served_mode {
                    hs::q12_query(&point)?
                } else {
                    point
                };
                let scores = nearest_scores(&point, &table)?;
                let greedy = if rule == DecodeRule::Greedy {
                    Some(scores.argmax(1)?.to_vec1::<u32>()?)
                } else {
                    None
                };
                let flock_rows = match rule {
                    DecodeRule::Greedy => None,
                    DecodeRule::FlockRank { k, .. }
                    | DecodeRule::FlockCell { k, .. }
                    | DecodeRule::CycleStep { k, .. } => {
                        let k = k.clamp(1, FLOCK_DECODE_K).min(scores.dim(1)?);
                        let top = scores
                            .arg_sort_last_dim(false)?
                            .narrow(1, 0, k)?
                            .contiguous()?;
                        let picked = scores.gather(&top, 1)?.to_vec2::<f32>()?;
                        Some((top.to_vec2::<u32>()?, picked))
                    }
                };
                for (b, row) in state.iter_mut().enumerate() {
                    if !row.active {
                        continue;
                    }
                    let token = match &flock_rows {
                        None => greedy.as_ref().and_then(|tokens| tokens.get(b)).copied()
                            .ok_or_else(|| invalid("the batched greedy readout returned no token"))?,
                        Some((ids_k, picked)) => {
                            let scored = rank_order(
                                ids_k[b]
                                    .iter()
                                    .copied()
                                    .zip(picked[b].iter().copied())
                                    .collect(),
                            );
                            if let DecodeRule::CycleStep { n, .. } = rule {
                                let ranked: Vec<u32> =
                                    scored.iter().map(|&(id, _)| id).collect();
                                let (token, mark) = cycle_step_pick(&row.tokens, &ranked, n)
                                    .ok_or_else(|| {
                                        invalid("the batched cycle step had no candidate")
                                    })?;
                                row.stepped.push(mark);
                                token
                            } else {
                                let (drawn, keep_end) = match rule {
                                    DecodeRule::FlockCell { margin_milli, .. } => {
                                        let n = cell_admitted(&scored, margin_milli);
                                        row.admitted.push(n as u8);
                                        (n, true)
                                    }
                                    DecodeRule::FlockRank { keep_end, .. } => {
                                        (scored.len(), keep_end)
                                    }
                                    DecodeRule::Greedy | DecodeRule::CycleStep { .. } => {
                                        (1, false)
                                    }
                                };
                                let ranked: Vec<u32> =
                                    scored[..drawn].iter().map(|&(id, _)| id).collect();
                                let index = (row.index as u64) * 64 + step as u64;
                                let draw = flock_rank_draw(seed, index, ranked.len());
                                flock_pick(&ranked, draw, keep_end, eos)
                            }
                        }
                    };
                    let stopped =
                        token == hs::SHEET_BOS_ID || (flock && Some(token) == eos);
                    if token != hs::SHEET_BOS_ID {
                        row.tokens.push(token);
                    }
                    row.ids.push(token);
                    row.active = !stopped;
                }
                // Every row advances one position, including terminated rows, so the next
                // forward is rectangular. A terminated row's suffix is never read.
                for row in state.iter_mut().filter(|row| !row.active) {
                    if row.ids.len() == time {
                        row.ids.push(hs::SHEET_BOS_ID);
                    }
                }
            }
            for row in state {
                replies[row.index] = Some(DecodedReply {
                    tokens: row.tokens,
                    admitted: row.admitted,
                    stepped: row.stepped,
                    served: None,
                });
            }
            Ok(())
        })?;
    }
    replies
        .into_iter()
        .enumerate()
        .map(|(index, reply)| reply.ok_or_else(|| invalid(format!("prompt {index} was not decoded"))))
        .collect()
}

/// The reply-table keys a reply's later steps read, and how many the table names (protocol
/// I-F): at each step `j ≥ 1`, the key [`greedy_reply_key`] forms from `reply[..j]` in the
/// window the decoder would see, exactly as [`decode_reply`] forms it. For a generated reply
/// this is the table hit rate under generation; for the true reply after the same prompt it is
/// the same rate under teacher forcing. `named` asks the reply table. `context` is the window
/// the decoder slides over, the model's own (claude-274).
fn reply_key_hits(
    prompt: &[u32],
    reply: &[u32],
    context: usize,
    named: impl Fn(&[u32]) -> bool,
) -> (usize, usize) {
    let (mut keys, mut hits) = (0, 0);
    for j in 1..reply.len() {
        let mut ids = prompt.to_vec();
        ids.extend_from_slice(&reply[..j]);
        let start = ids.len().saturating_sub(context);
        if prompt.len() <= start {
            continue;
        }
        let window = &ids[start..];
        if let Some(key) = greedy_reply_key(window, prompt.len() - 1 - start, &reply[..j]) {
            keys += 1;
            hits += usize::from(named(&key));
        }
    }
    (keys, hits)
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
    arm: FbsArm,
    device: &Device,
    served_forward: Option<bool>,
) -> Result<Vec<DecodeSpeed>> {
    check_run_arm(checkpoint, arm)?;
    let served = run_train_as_served(checkpoint, served_forward)?;
    let (dev_windows, _) = load_dev(data_root)?;
    arms::set_fbs_arm(Some(arm));
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
    greedy_reply_with_mode(&model, objective, &source.input_ids[..16], 4, seed, served)?;
    let mut speeds = Vec::with_capacity(DECODE_CONTEXTS.len());
    for &context in &DECODE_CONTEXTS {
        let prompt = &source.input_ids[..context.min(source.input_ids.len())];
        let started = Instant::now();
        // BOS stops a reply early; count what was decoded.
        let reply = greedy_reply_with_mode(&model, objective, prompt, tokens, seed, served)?;
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
    arm: FbsArm,
    device: &Device,
    served_forward: Option<bool>,
) -> Result<Vec<Inspection>> {
    check_run_arm(run, arm)?;
    let served = run_train_as_served(run, served_forward)?;
    let (dev_windows, dev_mask) = load_dev(data_root)?;
    arms::set_fbs_arm(Some(arm));
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
    let last = dirs.len().saturating_sub(1);
    for (index, dir) in dirs.into_iter().enumerate() {
        let name = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let step: usize = name.trim_start_matches("step-").parse().unwrap_or(0);
        let model = StackModel::load(&dir, device)?;
        load_reads(&dir, device)?;
        let (dev, _) = if served {
            hs::with_served_surrogate_training(seed, step, || {
                hs::evaluate_dev_panel_d11(&model, &dev_windows)
            })?
        } else {
            hs::evaluate_dev_panel(
                &model,
                &dev_windows,
                HamiltonArm::HamiltonActive,
                seed,
                step,
                false,
                false,
            )?
        };
        let query = with_eval_forward(served, seed, step, || {
            let hidden = crate::flock_read::points(&model, &ids, probe.len(), SHEET_CONTEXT)?;
            let idx = Tensor::from_vec(scored.clone(), scored.len(), device)?;
            let query = hidden.index_select(&idx, 0)?;
            if served {
                hs::q12_query(&query)
            } else {
                Ok(query)
            }
        })?;
        let (nearest, _) =
            nearest_reading(&model, &dev_windows, &dev_mask, seed, step, served)?;
        // The last checkpoint's register, one slot dropped at a time and then all of them
        // (claude-155): the same nearest-token reading, with the register restored after each.
        let register_drop = match crate::flock_read_register::prompt_register() {
            Some(reg) if index == last => {
                let mut rows = Vec::with_capacity(reg.q() + 1);
                for slot in (0..reg.q()).map(Some).chain([None]) {
                    crate::flock_read_register::set_prompt_register(Some(reg.without_slot(slot)?));
                    let dropped =
                        nearest_reading(&model, &dev_windows, &dev_mask, seed, step, served);
                    crate::flock_read_register::set_prompt_register(Some(reg.clone()));
                    let (dropped, _) = dropped?;
                    rows.push(RegisterDrop {
                        slot: slot.map_or_else(|| "all".into(), serde_json::Value::from),
                        hits: dropped.hits,
                        opening_hits: dropped.opening_hits,
                        copied_hits: dropped.copied_hits,
                    });
                }
                Some(rows)
            }
            _ => None,
        };
        // The last checkpoint's reading with the turn memory taken out (claude-161), restored after.
        let memory_drop = match crate::flock_read_memory::turn_memory() {
            Some(memory) if index == last => {
                crate::flock_read_memory::set_turn_memory(None);
                let dropped =
                    nearest_reading(&model, &dev_windows, &dev_mask, seed, step, served);
                crate::flock_read_memory::set_turn_memory(Some(memory));
                let (dropped, _) = dropped?;
                Some(RegisterDrop {
                    slot: "memory".into(),
                    hits: dropped.hits,
                    opening_hits: dropped.opening_hits,
                    copied_hits: dropped.copied_hits,
                })
            }
            _ => None,
        };
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
            register_weight_mean_abs: match crate::flock_read_register::prompt_register() {
                Some(reg) => Some(scalar(reg.weight_mean_abs()?)?),
                None => None,
            },
            register_drop,
            memory_weight_mean_abs: match crate::flock_read_memory::turn_memory() {
                Some(memory) => Some(scalar(memory.weight_mean_abs()?)?),
                None => None,
            },
            memory_reply_weight_mean_abs: match crate::flock_read_memory::turn_memory() {
                Some(memory) => match memory.reply_weight_mean_abs()? {
                    Some(mean) => Some(scalar(mean)?),
                    None => None,
                },
                None => None,
            },
            memory_drop,
        });
    }
    Ok(readings)
}

/// One split's scored rows as one checkpoint decodes them ([`score_rows`], one-model-end-to-end
/// claude-205): parallel lists, one entry per scored row in panel order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoredRows {
    pub split: String,
    /// Whether the checkpoint's turn memory was on.
    pub memory: bool,
    pub positions: Vec<usize>,
    pub targets: Vec<u32>,
    /// The whole table's nearest token: the reading the bars count.
    pub nearest: Vec<u32>,
    /// The served writer's pick from the same points: the float reference for the integer
    /// artifact's top-1.
    pub writer: Vec<u32>,
}

impl ScoredRows {
    fn hits(&self) -> usize {
        self.nearest.iter().zip(&self.targets).filter(|(p, t)| p == t).count()
    }
}

/// One greedy reply ([`score_rows`]): ids, for the grader, and text, to read.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IdSample {
    /// The corpus position of the reply's first target.
    pub position: usize,
    pub prompt: Vec<u32>,
    pub reply: Vec<u32>,
    pub reference: Vec<u32>,
    pub reply_text: String,
    pub reference_text: String,
    /// The reply-table keys the reply's later steps read, and how many the table names
    /// ([`reply_key_hits`], protocol I-F): the hit rate under generation. 0 for files written
    /// before claude-208.
    #[serde(default)]
    pub reply_keys: usize,
    #[serde(default)]
    pub reply_named: usize,
    /// The same for the true reply after the same prompt: the hit rate under teacher forcing.
    #[serde(default)]
    pub reference_keys: usize,
    #[serde(default)]
    pub reference_named: usize,
    /// Under [`DecodeRule::FlockCell`], each drawn step's admitted count ([`cell_admitted`],
    /// claude-213), a step that drew BOS and ended the reply included, so it can hold one more
    /// count than `reply` has tokens. Empty, and not written, under every other rule, so their
    /// files are as before.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub admitted: Vec<u8>,
    /// Under [`DecodeRule::CycleStep`], each step's mark (claude-217): [`STEP_NEAREST`],
    /// [`STEP_OFF`] or [`STEP_EXHAUSTED`]. Empty, and not written, under every other rule.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stepped: Vec<u8>,
    /// Under a keyed arm (claude-219), the token a trunk key gave the reply as its first, where
    /// the memory's exact key missed the opening and the key fired within its τ
    /// ([`keyed_opening`]). None, and not written, where it did not and under every other arm.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keyed: Option<u32>,
    /// Under the served arm (claude-220), the served key's least L1 distance where it gave the
    /// first token, beside the keys file's for the same opening (the parity count). None, and
    /// not written, elsewhere.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub served_distance: Option<f32>,
}

/// One file [`score_rows`] wrote, with its nearest and writer hits when it holds rows.
#[derive(Clone, Debug, Serialize)]
pub struct ScoredFile {
    pub file: String,
    pub rows: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nearest_hits: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub writer_hits: Option<usize>,
    /// An unpinned panel's `tokens.u16` sha256 (`--panel`, protocol I-A), when the file is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub panel_sha256: Option<String>,
    /// The tables control's rows, openings apart from later rows ([`controls`]' (T) only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tables: Option<TableNamed>,
    /// A samples file's decode: the reply tokens it decoded and the wall seconds it took, so
    /// each rule's decode tok/s (claude-213).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decode: Option<(usize, f64)>,
    /// The served point key's searches in a samples file's decode (claude-220): how many, and
    /// their wall seconds, inside `decode`'s.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<(u64, f64)>,
    /// The window the file's split was cut at (claude-274): a `--panel`'s is `--context`'s. Left
    /// out at [`SHEET_CONTEXT`], so a 192-token file's line is as before.
    #[serde(skip_serializing_if = "is_sheet_context")]
    pub context: usize,
}

/// The rows the tables control (T) names, and its hits among them, at the openings and at the
/// later rows apart: protocol I-D's check reads them against the census.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct TableNamed {
    pub openings_named: usize,
    pub openings_hit: usize,
    pub later_named: usize,
    pub later_hit: usize,
}

/// Whether `name` may name a `--panel` split: one file-name part, and never DEV or the sealed Fresh
/// V2, so no panel is ever read as either.
fn panel_name_ok(name: &str) -> bool {
    !name.is_empty()
        && !["dev", "fresh-v2"].contains(&name)
        && !name.contains(['/', '\\'])
        && !name.contains("..")
}

/// One split [`score_rows`] scores: its name, windows and mask, and an unpinned panel's sha.
struct ScoreSplit {
    name: String,
    windows: Vec<CausalWindow>,
    mask: Vec<u8>,
    sha256: Option<String>,
    /// Whether its prompts are sampled: DEV and every `--panel`, never Fresh V2.
    samples: bool,
    /// The window it is cut at: [`SHEET_CONTEXT`] for DEV and Fresh V2, `--context`'s for a
    /// `--panel` (claude-274).
    context: usize,
    /// A chained panel's `--panel-queries` and their prompts at `context` (claude-275).
    queries: Option<(Vec<ChainQuery>, QueryPrompts)>,
}

/// Write `bytes` to a new file at `path`; an existing file is refused, never replaced.
fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| invalid(format!("{}: {e}", path.display())))?;
    file.write_all(bytes)?;
    Ok(())
}

/// The sealed Fresh V2 panel in `dir`, pinned: its tokens and response mask.
fn load_fresh_v2(dir: &Path) -> Result<(Vec<u32>, Vec<u8>)> {
    hs::load_pinned_uort_split(
        &dir.join("tokens.u16"),
        &dir.join("response_mask.u8"),
        PINNED_FRESH_V2_TOKENS_SHA256,
        PINNED_FRESH_V2_MASK_SHA256,
        EXPECTED_FRESH_V2_TOKENS,
        EXPECTED_FRESH_V2_ELIGIBLE_TARGETS,
        hs::SHEET_VOCAB,
        hs::SHEET_BOS_ID,
    )
}

/// The true reply after `window`'s prompt, which ends at its first scored offset: the scored
/// targets while their offsets stay contiguous from there, so a mask gap or the window's end
/// never splices text, at most `tokens` of them.
fn reference_reply(window: &CausalWindow, tokens: usize) -> Vec<u32> {
    let off = window.scored_offsets.first().copied().unwrap_or(0);
    window
        .scored_offsets
        .iter()
        .zip(&window.scored_targets)
        .enumerate()
        .take_while(|&(i, (&o, _))| o == off + i)
        .take(tokens)
        .map(|(_, (_, &t))| t)
        .collect()
}

/// Score one saved checkpoint row by row (one-model-end-to-end done-when 2, claude-205), on the
/// GPU, with the protocol's scoring instruments (claude-208).
/// - **Rows.** For DEV, for the sealed Fresh V2 panel in `fresh_v2` when given, and for each
///   unpinned `(name, dir)` of `panels` (I-A; P-dev), it writes every scored row's nearest and
///   writer picks ([`ScoredRows`]) to `out/rows-{split}-memory.json` with the checkpoint's turn
///   memory as loaded, and to `rows-{split}-no-memory.json` with it taken out. The rows come from
///   [`nearest_reading`]'s own loop, so their sums are its readings.
/// - **Samples.** For DEV and each panel (never Fresh V2), `samples` replies of `reply_tokens`
///   tokens ([`IdSample`]) after [`sample_windows`]' prompts over that split:
///   - greedy, with the memory on and taken out: DEV's to `samples-memory.json` and
///     `samples-no-memory.json` as before, a panel's to `samples-{name}-greedy-{memory|no-memory}.json`;
///   - the Flock rank rule (I-E), with the memory on, as it would be served:
///     `samples-{split}-flock-memory.json`;
///   - the cell rule at each margin of `cells` (claude-213), with the memory on:
///     `samples-{split}-flock-cell-{margin}-memory.json`;
///   - greedy under each read scope of `scopes` (claude-215), with the memory on:
///     `samples-{split}-greedy-read-{scope}-memory.json`;
///   - with `cycle`, the cycle step (claude-217), with the memory on, alone and under each read
///     scope: `samples-{split}-cycle-step-memory.json` and
///     `samples-{split}-cycle-step-read-{scope}-memory.json`;
///   - with `keys`, greedy with each reply's first token given as its reference's (claude-218),
///     with the memory on: `samples-{split}-forced-memory.json`;
///   - for each `(key, τ)` of `keyed` (claude-219, which needs `keys`), greedy with the memory on
///     and the reply's first token the key's name where [`keyed_opening`] fires:
///     `samples-{split}-keyed-{key}-memory.json`;
///   - with `served_arm` (claude-220, which needs `keys`), greedy with the memory on and the point
///     key named in the decode itself from the opening row's own point ([`served_opening`]):
///     `samples-{split}-served-memory.json`, with the search's count and seconds.
///
///   Each records its reply-table hits under generation and under teacher forcing (I-F).
///   With `batched_decode`, full-scope nearest arms decode prompts of equal length together; all
///   other arms retain the one-at-a-time path.
/// - **Keys.** With `keys`, every split's openings named by the trunk keys ([`OpeningKeys`],
///   claude-218), taken with the memory off before the decodes: `keys-{split}.json`. Their
///   TRAIN table is built from TRAIN; with `served_arm` it is saved to
///   `out/opening-table.safetensors`, read back to the same bits, and the copy read back is served.
///
/// `checkpoint` is a `step-N` directory; the flags and `arm` must match its run's.
///
/// **The window (claude-274).** Every `panels` split is cut into `context`-token windows, at most
/// the checkpoint's own; DEV and Fresh V2 keep [`SHEET_CONTEXT`]'s. Every decode slides over the
/// model's own window. Off [`SHEET_CONTEXT`], `keys`, `served_arm`, `keyed`, `traces` and
/// `forces`, whose TRAIN table and trace read 192-token windows, are refused by name
/// ([`window_bound_flags`]). With `last_opening`, each sample decodes from its window's
/// last reply opening ([`last_openings`]) in place of its first. From 193 to 1,536 positions
/// the row stride is untested, and is refused ([`row_stride_window`], claude-275).
///
/// **The queries (claude-275).** For each `(name, file)` of `panel_queries`, a chained `--panel`
/// and its `queries.json`, every query whose reply fits the window decodes greedy at full scope
/// with the memory on and off, from the window ending at its reply ([`query_prompts`]):
/// `samples-{name}-queries-{memory|no-memory}.json`. Each is graded by whether its reply states
/// its note's value ([`recall_grade`]), binned by distance, in `recall-{name}-{label}.json`.
#[allow(clippy::too_many_arguments)]
pub fn score_rows(
    data_root: &Path,
    checkpoint: &Path,
    out: &Path,
    fresh_v2: Option<&Path>,
    panels: &[(String, PathBuf)],
    samples: usize,
    reply_tokens: usize,
    retain_rates: bool,
    fold_scores: bool,
    reflect_floor: bool,
    seed: u64,
    arm: FbsArm,
    flock: DecodeRule,
    cells: &[u32],
    scopes: &[ReadScope],
    cycle: bool,
    keys: bool,
    keyed: &[(String, f32)],
    served_arm: bool,
    traces: &[(String, Vec<usize>)],
    forces: Vec<(String, DecisionForce)>,
    device: &Device,
    served_forward: Option<bool>,
    batched_decode: bool,
    context: usize,
    last_opening: bool,
    panel_queries: &[(String, PathBuf)],
) -> Result<Vec<ScoredFile>> {
    check_run_arm(checkpoint, arm)?;
    training_context(context)?;
    window_bound_flags(
        context,
        keys,
        served_arm,
        !keyed.is_empty(),
        !traces.is_empty(),
        !forces.is_empty(),
    )?;
    row_stride_window(context)?;
    if served_arm && !keys {
        return Err(invalid("--served-key reads the opening keys, so it needs --opening-keys 1"));
    }
    if served_arm && samples == 0 {
        return Err(invalid("--served-key decodes samples, so it needs --samples of 1 or more"));
    }
    if !keyed_ok(keyed) {
        return Err(invalid("--keyed-decode wants distinct trunk keys (sign, hsign, point), each with a τ of 0 or more"));
    }
    if !keyed.is_empty() && !keys {
        return Err(invalid("--keyed-decode reads the opening keys, so it needs --opening-keys 1"));
    }
    if !keyed.is_empty() && samples == 0 {
        return Err(invalid("--keyed-decode decodes samples, so it needs --samples of 1 or more"));
    }
    if !cells_ok(cells) {
        return Err(invalid(
            "--flock-cell wants distinct margins of at most 4,000 thousandths",
        ));
    }
    if !scopes_ok(scopes) {
        return Err(invalid("--reply-read wants distinct scopes from copy and all"));
    }
    let step: usize = checkpoint
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| n.strip_prefix("step-"))
        .and_then(|n| n.parse().ok())
        .ok_or_else(|| invalid("--score-rows wants a run's step-N checkpoint directory"))?;
    let tokenizer = pinned_tokenizer(data_root)?;
    let eos = tokenizer
        .token_id("<|eos|>")
        .ok_or_else(|| invalid("the pinned tokenizer has no <|eos|>"))?;
    let windows_of = |tokens: &[u32], mask: &[u8], window: usize| -> Result<Vec<CausalWindow>> {
        Ok(hs::build_document_windows(tokens, mask, window, hs::SHEET_BOS_ID)?.0)
    };
    let (dev_windows, dev_mask) = load_dev(data_root)?;
    let mut splits = vec![ScoreSplit {
        name: "dev".to_owned(),
        windows: dev_windows,
        mask: dev_mask,
        sha256: None,
        samples: true,
        context: SHEET_CONTEXT,
        queries: None,
    }];
    if let Some(dir) = fresh_v2 {
        let (tokens, mask) = load_fresh_v2(dir)?;
        splits.push(ScoreSplit {
            name: "fresh-v2".to_owned(),
            windows: windows_of(&tokens, &mask, SHEET_CONTEXT)?,
            mask,
            sha256: None,
            samples: false,
            context: SHEET_CONTEXT,
            queries: None,
        });
    }
    for (name, _) in panel_queries {
        if !panels.iter().any(|(panel, _)| panel == name) {
            return Err(invalid(format!("--panel-queries {name} needs --panel {name}=DIR")));
        }
    }
    for (name, dir) in panels {
        if !panel_name_ok(name) || splits.iter().any(|s| s.name == *name) {
            return Err(invalid(format!(
                "--panel {name}: a panel name is one word, not dev, fresh-v2 or a scored split"
            )));
        }
        let (tokens, mask, sha) = load_unpinned_uort(dir)?;
        let queries = match panel_queries.iter().find(|(panel, _)| panel == name) {
            Some((_, file)) => {
                let queries = load_panel_queries(file, &sha, &sha256_bytes(&mask))?;
                let prompts = query_prompts(&tokens, &mask, context, &queries)?;
                println!(
                    "[queries] {name}: {} queries, {} decoded at {context}, {} past the window",
                    queries.len(),
                    prompts.decoded.len(),
                    prompts.past_window.len()
                );
                Some((queries, prompts))
            }
            None => None,
        };
        splits.push(ScoreSplit {
            name: name.clone(),
            windows: windows_of(&tokens, &mask, context)?,
            mask,
            sha256: Some(sha),
            samples: true,
            context,
            queries,
        });
    }
    for (i, (name, _)) in traces.iter().enumerate() {
        if !splits.iter().any(|s| s.name == *name) || traces[..i].iter().any(|(n, _)| n == name) {
            return Err(invalid(format!(
                "--decision-trace {name}: a trace names a scored split (dev, fresh-v2 or a panel), once"
            )));
        }
    }
    fs::create_dir_all(out)?;
    arms::set_fbs_arm(Some(arm));
    crate::geometric_stack::set_retention_rates(retain_rates);
    crate::geometric_stack::set_folded_scores(fold_scores);
    crate::geometric_stack::set_reflected_floor(reflect_floor);
    let _guard = ArmGuard;
    let model = StackModel::load(checkpoint, device)?;
    if context > model.config.context {
        return Err(invalid(format!(
            "--context {context} is past the checkpoint's own {}-token window",
            model.config.context
        )));
    }
    load_reads(checkpoint, device)?;
    let unserved = unserved_reads();
    if !traces.is_empty() && !unserved.is_empty() {
        return Err(invalid(format!(
            "--decision-trace records the decisions R serves, and this checkpoint also sets {}",
            unserved.join(", ")
        )));
    }
    // The trace's side (claude-252): T on an evaluation copy rounded as exported, F otherwise.
    let side = if traces.is_empty() {
        "F"
    } else {
        crate::fbs_export::checkpoint_trace_side(checkpoint)?
    };
    // Q5's forced replay (claude-253): each forced split's set, on that split's trace, checked
    // against the model's shape here, before any forward.
    let (layers, lanes) = (model.config.layers(), model.config.width / 4);
    let reads: Vec<(usize, usize, usize)> = crate::flock_read_hamming::hamming_reads()
        .map(|r| {
            crate::flock_read_hamming::HAMMING_DEPTHS
                .iter()
                .zip(&r.reads)
                .map(|(&depth, read)| {
                    let width = model.config.width.checked_div(read.heads()).unwrap_or(0);
                    (depth, read.heads(), width)
                })
                .collect()
        })
        .unwrap_or_default();
    let mut forced: BTreeMap<String, DecisionForce> = BTreeMap::new();
    for (split, force) in forces {
        let Some((_, positions)) = traces.iter().find(|(name, _)| *name == split) else {
            return Err(invalid(format!(
                "--decision-force {split} needs --decision-trace {split}"
            )));
        };
        FTrace::new(positions)?.with_force(force.clone(), layers, lanes, &reads)?;
        match splits.iter().find(|s| s.name == split) {
            Some(s) => forced_within(&force, &s.windows)?,
            None => return Err(invalid(format!("--decision-force {split}: no such split"))),
        }
        if forced.insert(split.clone(), force).is_some() {
            return Err(invalid(format!("--decision-force {split} is given twice")));
        }
    }
    // A run trained with the reply's read held to the prompt scores its rows the same way
    // (claude-216); its decodes take the cut through `--reply-read`, as served.
    let trained = run_trained_cut(checkpoint)?;
    let trained_as_served = run_train_as_served(checkpoint, served_forward)?;
    set_trained_cut(trained);
    println!("[cut] scoring with the trained cut: {}", trained.map_or("none", |s| s.name()));
    let memory = crate::flock_read_memory::turn_memory();
    if served_arm && memory.is_none() {
        return Err(invalid("--served-key serves where the turn memory's exact key misses, so it needs a checkpoint with a turn memory"));
    }
    // The opening keys (claude-218), with the memory off: TRAIN's opening points under this
    // trunk, and each split's openings named by the nearest of them. Taken before the decodes,
    // so the keyed arms (claude-219) can read them, and written after them, as before.
    let mut served_key = None;
    let mut table_rows = None;
    let named_openings: Vec<OpeningKeys> = if keys {
        crate::flock_read_memory::set_turn_memory(None);
        // The served table (claude-220), built from TRAIN; with `served_arm` it is saved to `out`
        // and read back, which must give the same bits, and the copy read back is the one served.
        let (train_tokens, train_mask) = load_train_split(data_root)?;
        let train_windows = windows_of(&train_tokens, &train_mask, SHEET_CONTEXT)?;
        let (built, targets, _) = opening_points(
            &model,
            &train_windows,
            &train_mask,
            seed,
            step,
            trained_as_served,
        )?;
        drop((train_windows, train_tokens, train_mask));
        let (train, train_targets) = if served_arm {
            let path = out.join(OPENING_TABLE_FILE);
            save_opening_table(&path, &built, &targets)?;
            let (points, back) = load_opening_table(&path, device)?;
            if back != targets || !same_bits(&points, &built)? {
                return Err(invalid("the opening table does not read back as saved (an instrument fault)"));
            }
            table_rows = Some(back.len());
            (points, back)
        } else {
            (built, targets)
        };
        let named = splits
            .iter()
            .map(|split| {
                let (query, targets, positions) = opening_points(
                    &model,
                    &split.windows,
                    &split.mask,
                    seed,
                    step,
                    trained_as_served,
                )?;
                opening_keys(&split.name, &query, targets, positions, &train, &train_targets)
            })
            .collect::<Result<_>>()?;
        if served_arm {
            served_key = Some(ServedKey { points: train, targets: std::sync::Arc::new(train_targets) });
        }
        named
    } else {
        Vec::new()
    };
    let mut written = Vec::new();
    for on in [true, false] {
        crate::flock_read_memory::set_turn_memory(if on { memory.clone() } else { None });
        let label = if on { "memory" } else { "no-memory" };
        for split in &splits {
            // Q5's decision trace (claude-251), in the memory pass only, as F′'s rows are read.
            let mut trace = match traces.iter().find(|(name, _)| on && *name == split.name) {
                Some((_, positions)) => {
                    let trace = FTrace::new(positions)?.on_side(side)?;
                    Some(match forced.get(&split.name) {
                        Some(force) => trace.with_force(force.clone(), layers, lanes, &reads)?,
                        None => trace,
                    })
                }
                None => None,
            };
            let picks = row_picks(
                &model,
                &split.windows,
                &split.mask,
                seed,
                step,
                trained_as_served,
                trace.as_mut(),
                |_, _| Ok(()),
            )?;
            if let Some(trace) = trace {
                let file = format!("decision-trace-{}.json", split.name);
                let trace = trace.finish()?;
                write_new(&out.join(&file), &serde_json::to_vec(&trace)?)?;
                println!(
                    "[trace] {file}: {} windows, {} steps",
                    trace.windows.len(),
                    trace.windows.iter().map(|w| w.steps.len()).sum::<usize>()
                );
            }
            let rows = ScoredRows {
                split: split.name.clone(),
                memory: on,
                positions: picks.iter().map(|p| p.position).collect(),
                targets: picks.iter().map(|p| p.target).collect(),
                nearest: picks.iter().map(|p| p.nearest).collect(),
                writer: picks.iter().map(|p| p.writer).collect(),
            };
            let file = format!("rows-{}-{label}.json", split.name);
            write_new(&out.join(&file), &serde_json::to_vec(&rows)?)?;
            written.push(ScoredFile {
                file,
                rows: picks.len(),
                nearest_hits: Some(rows.hits()),
                writer_hits: Some(picks.iter().filter(|p| p.writer == p.target).count()),
                panel_sha256: split.sha256.clone(),
                tables: None,
                decode: None,
                search: None,
                context: split.context,
            });
        }
        // Greedy with the memory on and off; the Flock rules, and greedy under each read scope
        // (claude-215), with it on only, as served.
        let mut served = vec![(DecodeRule::Greedy, ReadScope::Full), (flock, ReadScope::Full)];
        served.extend(cells.iter().map(|&m| (DecodeRule::flock_cell(m), ReadScope::Full)));
        served.extend(scopes.iter().map(|&s| (DecodeRule::Greedy, s)));
        if cycle {
            served.push((DecodeRule::CYCLE_STEP, ReadScope::Full));
            served.extend(scopes.iter().map(|&s| (DecodeRule::CYCLE_STEP, s)));
            served.push((DecodeRule::ECHO_STEP, ReadScope::Full));
        }
        let rules: &[(DecodeRule, ReadScope)] = if on { &served } else { &served[..1] };
        let active = crate::flock_read_memory::turn_memory();
        let named = |key: &[u32]| active.as_ref().is_some_and(|m| m.reply_stored(key).is_some());
        for (at, split) in splits.iter().enumerate().filter(|(_, s)| s.samples) {
            let last;
            let prompts: Vec<&CausalWindow> = if last_opening {
                last = last_openings(&split.windows, &split.mask, samples);
                last.iter().collect()
            } else {
                sample_windows(&split.windows, &split.mask, samples)
            };
            // Each prompt's reply under one rule and scope, the replies' tokens and the wall time.
            let decode = |prompts: &[&CausalWindow],
                          rule: DecodeRule,
                          scope: ReadScope|
             -> Result<(Vec<IdSample>, usize, f64)> {
                let started = std::time::Instant::now();
                let prompt_ids: Vec<Vec<u32>> = prompts
                    .iter()
                    .map(|window| window.input_ids[..=window.scored_offsets[0]].to_vec())
                    .collect();
                // The prompt's own flags: an earlier reply in it is limited as in training.
                let prompt_masks = prompts
                    .iter()
                    .map(|window| {
                        let off = window.scored_offsets[0];
                        let flags = window_input_mask(std::slice::from_ref(window), &split.mask)?;
                        Ok(flags[..=off].to_vec())
                    })
                    .collect::<Result<Vec<_>>>()?;
                let decoded = decode_replies_with_mode(
                    &model,
                    PointObjective::Cell,
                    &prompt_ids,
                    &prompt_masks,
                    reply_tokens,
                    seed,
                    rule,
                    Some(eos),
                    scope,
                    trained_as_served,
                    batched_decode,
                )?;
                let mut replies = Vec::with_capacity(prompts.len());
                for (window, (prompt, decoded)) in
                    prompts.iter().zip(prompt_ids.iter().zip(decoded))
                {
                    let DecodedReply {
                        tokens: reply,
                        admitted,
                        stepped,
                        ..
                    } = decoded;
                    let reference = reference_reply(window, reply_tokens);
                    let (reply_keys, reply_named) = reply_key_hits(prompt, cut_at_eos(&reply, eos), model.config.context, named);
                    let (reference_keys, reference_named) =
                        reply_key_hits(prompt, cut_at_eos(&reference, eos), model.config.context, named);
                    replies.push(IdSample {
                        position: window.scored_corpus_positions[0],
                        prompt: prompt.to_vec(),
                        reply_text: tokenizer.decode(&reply),
                        reference_text: tokenizer.decode(&reference),
                        reply,
                        reference,
                        reply_keys,
                        reply_named,
                        reference_keys,
                        reference_named,
                        admitted,
                        stepped,
                        keyed: None,
                        served_distance: None,
                    });
                }
                // Every reply's tokens over the loop's wall time; the reference work in the loop
                // is host-side and small beside the decode.
                let decoded = replies.iter().map(|r| r.reply.len()).sum();
                Ok((replies, decoded, started.elapsed().as_secs_f64()))
            };
            for &(rule, scope) in rules {
                let (replies, decoded, seconds) = decode(&prompts, rule, scope)?;
                let file = match (split.name.as_str(), rule) {
                    (name, DecodeRule::CycleStep { n, .. }) if scope != ReadScope::Full => {
                        format!("samples-{name}-{}-read-{}-{label}.json", step_name(n), scope.name())
                    }
                    (name, DecodeRule::CycleStep { n, .. }) => {
                        format!("samples-{name}-{}-{label}.json", step_name(n))
                    }
                    (name, DecodeRule::Greedy) if scope != ReadScope::Full => {
                        format!("samples-{name}-greedy-read-{}-{label}.json", scope.name())
                    }
                    ("dev", DecodeRule::Greedy) => format!("samples-{label}.json"),
                    (name, DecodeRule::Greedy) => format!("samples-{name}-greedy-{label}.json"),
                    (name, DecodeRule::FlockRank { keep_end: false, .. }) => {
                        format!("samples-{name}-flock-{label}.json")
                    }
                    (name, DecodeRule::FlockRank { keep_end: true, .. }) => {
                        format!("samples-{name}-flock-end-{label}.json")
                    }
                    (name, DecodeRule::FlockCell { margin_milli, .. }) => {
                        format!("samples-{name}-flock-cell-{margin_milli}-{label}.json")
                    }
                };
                write_new(&out.join(&file), &serde_json::to_vec(&replies)?)?;
                written.push(ScoredFile {
                    file,
                    rows: replies.len(),
                    nearest_hits: None,
                    writer_hits: None,
                    panel_sha256: split.sha256.clone(),
                    tables: None,
                    decode: Some((decoded, seconds)),
                    search: None,
                    context: split.context,
                });
            }
            // A chained panel's queries (claude-275), greedy at full scope with the memory on and
            // off: each from a prompt ending at its opening, graded by whether the reply states
            // its note's value.
            if let Some((queries, query_windows)) = &split.queries {
                let windows: Vec<&CausalWindow> = query_windows.decoded.iter().map(|(_, w)| w).collect();
                let (replies, decoded, seconds) = decode(&windows, DecodeRule::Greedy, ReadScope::Full)?;
                let texts: Vec<(String, String)> = replies
                    .iter()
                    .map(|r| {
                        (
                            tokenizer.decode(cut_at_eos(&r.reply, eos)),
                            tokenizer.decode(cut_at_eos(&r.reference, eos)),
                        )
                    })
                    .collect();
                let (total, bins, outcomes) = recall_grade(queries, query_windows, &texts)?;
                println!(
                    "[recall] {} {label}: {} of {} decoded state the answer; {} references do; {} from the document's start",
                    split.name, total.stated, total.decoded, total.reference_states, total.whole_document
                );
                let file = format!("samples-{}-queries-{label}.json", split.name);
                write_new(&out.join(&file), &serde_json::to_vec(&replies)?)?;
                written.push(ScoredFile {
                    file,
                    rows: replies.len(),
                    nearest_hits: None,
                    writer_hits: None,
                    panel_sha256: split.sha256.clone(),
                    tables: None,
                    decode: Some((decoded, seconds)),
                    search: None,
                    context: split.context,
                });
                let recall = QueryRecall {
                    split: split.name.clone(),
                    memory: on,
                    context: split.context,
                    reply_tokens,
                    total,
                    bins,
                    outcomes,
                };
                let file = format!("recall-{}-{label}.json", split.name);
                write_new(&out.join(&file), &serde_json::to_vec(&recall)?)?;
                written.push(ScoredFile {
                    file,
                    rows: queries.len(),
                    nearest_hits: None,
                    writer_hits: None,
                    panel_sha256: split.sha256.clone(),
                    tables: None,
                    decode: None,
                    search: None,
                    context: split.context,
                });
            }
            // Greedy with each reply's first token given, where `openings` gives one (the arms
            // below): the token is taken without a forward, and the rest decodes from the prompt
            // and it, as served. With `record`, each reply keeps the token it was given. With
            // `serve`, the served point key names the opening in the decode itself (claude-220),
            // and each reply keeps the token it gave.
            let opened = |openings: &[Option<u32>], record: bool, serve: bool| -> Result<(Vec<IdSample>, usize, f64)> {
                let mut replies = Vec::with_capacity(prompts.len());
                let started = std::time::Instant::now();
                for (index, (window, &opening)) in prompts.iter().zip(openings).enumerate() {
                    let off = window.scored_offsets[0];
                    let prompt = &window.input_ids[..=off];
                    let reference = reference_reply(window, reply_tokens);
                    set_forced_opening(opening);
                    set_served_key(if serve { served_key.clone() } else { None });
                    let decoded = decode_reply_with_mode(
                        &model,
                        PointObjective::Cell,
                        prompt,
                        reply_tokens,
                        seed,
                        DecodeRule::Greedy,
                        index,
                        Some(eos),
                        ReadScope::Full,
                        &[],
                        trained_as_served,
                    );
                    set_forced_opening(None);
                    set_served_key(None);
                    let decoded = decoded?;
                    let (given, distance) = match (serve, decoded.served) {
                        (true, Some((token, distance))) => (Some(token), Some(distance)),
                        (true, None) => (None, None),
                        (false, _) => (opening.filter(|_| record), None),
                    };
                    let reply = decoded.tokens;
                    let (reply_keys, reply_named) = reply_key_hits(prompt, cut_at_eos(&reply, eos), model.config.context, named);
                    let (reference_keys, reference_named) =
                        reply_key_hits(prompt, cut_at_eos(&reference, eos), model.config.context, named);
                    replies.push(IdSample {
                        position: window.scored_corpus_positions[0],
                        prompt: prompt.to_vec(),
                        reply_text: tokenizer.decode(&reply),
                        reference_text: tokenizer.decode(&reference),
                        reply,
                        reference,
                        reply_keys,
                        reply_named,
                        reference_keys,
                        reference_named,
                        admitted: Vec::new(),
                        stepped: Vec::new(),
                        keyed: given,
                        served_distance: distance,
                    });
                }
                // A given token had no forward, so the speed counts only the decoded ones.
                let decoded = replies
                    .iter()
                    .zip(openings)
                    .map(|(r, o)| r.reply.len().saturating_sub(usize::from(o.is_some())))
                    .sum();
                Ok((replies, decoded, started.elapsed().as_secs_f64()))
            };
            // The forced-opening arm (claude-218), each reply's first token its reference's, so
            // what a right opening alone lifts is read beside greedy; and the keyed arms
            // (claude-219), the first token a trunk key's name where the exact key misses.
            // And the served arm (claude-220): the point key named in the decode itself, from the
            // table read back from disk.
            let mut arms: Vec<(String, Vec<Option<u32>>, bool, bool)> = Vec::new();
            if keys && on {
                let openings = prompts.iter().map(|w| reference_reply(w, reply_tokens).first().copied()).collect();
                arms.push((format!("samples-{}-forced-{label}.json", split.name), openings, false, false));
                for (key, tau) in keyed {
                    let openings = keyed_openings(&prompts, &named_openings[at], key, *tau, active.as_ref())?;
                    arms.push((format!("samples-{}-keyed-{key}-{label}.json", split.name), openings, true, false));
                }
                if served_arm {
                    arms.push((format!("samples-{}-served-{label}.json", split.name), vec![None; prompts.len()], false, true));
                }
            }
            for (file, openings, record, serve) in arms {
                take_served_searches();
                let (replies, decoded, seconds) = opened(&openings, record, serve)?;
                let (searches, micros) = take_served_searches();
                write_new(&out.join(&file), &serde_json::to_vec(&replies)?)?;
                written.push(ScoredFile {
                    file,
                    rows: replies.len(),
                    nearest_hits: None,
                    writer_hits: None,
                    panel_sha256: split.sha256.clone(),
                    tables: None,
                    decode: Some((decoded, seconds)),
                    search: serve.then_some((searches, micros as f64 / 1e6)),
                    context: split.context,
                });
            }
        }
    }
    for (split, keys) in splits.iter().zip(&named_openings) {
        let file = format!("keys-{}.json", split.name);
        write_new(&out.join(&file), &serde_json::to_vec(keys)?)?;
        // The keys' hits are in the file itself; the nearest and writer fields name the
        // trunk's and the writer's picks, so they stay empty here.
        written.push(ScoredFile {
            file,
            rows: keys.positions.len(),
            nearest_hits: None,
            writer_hits: None,
            panel_sha256: split.sha256.clone(),
            tables: None,
            decode: None,
            search: None,
            context: split.context,
        });
    }
    if let Some(rows) = table_rows {
        written.push(ScoredFile {
            file: OPENING_TABLE_FILE.to_owned(),
            rows,
            nearest_hits: None,
            writer_hits: None,
            panel_sha256: None,
            tables: None,
            decode: None,
            search: None,
            context: SHEET_CONTEXT,
        });
    }
    Ok(written)
}

/// The served point key's table file (claude-220), in a scoring run's `out`.
pub const OPENING_TABLE_FILE: &str = "opening-table.safetensors";

/// Saves the served table (claude-220): TRAIN's opening `points` (`(n, width)`, f32) and their
/// `targets` (u32), as `points` and `targets`. It never replaces a file: the path is created
/// new before the table is written into it.
fn save_opening_table(path: &Path, points: &Tensor, targets: &[u32]) -> Result<()> {
    let (n, _) = points.dims2()?;
    if n != targets.len() {
        return Err(invalid("the opening table wants one target per point"));
    }
    write_new(path, &[]).map_err(|e| invalid(format!("{e}; a table is never replaced")))?;
    let targets = Tensor::from_vec(targets.to_vec(), n, points.device())?;
    let tensors = std::collections::HashMap::from([("points".to_owned(), points.clone()), ("targets".to_owned(), targets)]);
    candle_core::safetensors::save(&tensors, path)?;
    Ok(())
}

/// Whether two f32 tensors hold the same shape and the same bits: a +0.0 against a −0.0, or
/// a NaN, differs, where a difference of values would not see it.
fn same_bits(a: &Tensor, b: &Tensor) -> Result<bool> {
    let bits = |t: &Tensor| -> Result<Vec<u32>> {
        Ok(t.flatten_all()?.to_vec1::<f32>()?.into_iter().map(f32::to_bits).collect())
    };
    Ok(a.dims() == b.dims() && bits(a)? == bits(b)?)
}

/// The served table [`save_opening_table`] wrote, on `device`: its points and targets.
fn load_opening_table(path: &Path, device: &Device) -> Result<(Tensor, Vec<u32>)> {
    let tensors = candle_core::safetensors::load(path, device)?;
    let get = |name: &str| {
        tensors.get(name).cloned().ok_or_else(|| invalid(format!("{}: no {name} tensor", path.display())))
    };
    let (points, targets) = (get("points")?, get("targets")?.to_vec1::<u32>()?);
    let (n, _) = points.dims2()?;
    if n != targets.len() || points.dtype() != DType::F32 {
        return Err(invalid(format!("{}: not an opening table of f32 points with one target each", path.display())));
    }
    Ok((points, targets))
}

/// The longest suffix (S) reads: the longest context suffix it looks for in TRAIN.
pub const SUFFIX_CONTROL_MAX: usize = 64;

/// A prediction no token can equal: a control's miss.
const CONTROL_MISS: u32 = u32::MAX;

/// One step of the suffix hash, extending `h` (the hash of a suffix of length `n − 1`) by
/// `token` on its left: order-dependent, deterministic, and the same for TRAIN and the rows.
fn suffix_hash_step(h: u64, token: u32, n: usize) -> u64 {
    let mut x = h ^ (u64::from(token).wrapping_add(1)).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x = x.wrapping_add((n as u64).wrapping_mul(0xD1B5_4A32_D192_ED03));
    x ^= x >> 31;
    x.wrapping_mul(0xBF58_476D_1CE4_E5B9)
}

/// The longest-suffix control (S, protocol I-D) for each `(context, _)` of `rows`: TRAIN's
/// commonest next token, ties to the smaller id, after the longest suffix of the context, up
/// to `max` tokens, that `train` holds with a token after it. A BOS can be a suffix's first
/// token (a document's start), and no suffix reaches back past it. A context whose last token
/// TRAIN never shows gets [`CONTROL_MISS`].
fn longest_suffix_predictions(train: &[u32], contexts: &[&[u32]], max: usize) -> Vec<u32> {
    use std::collections::{HashMap, HashSet};
    // Every suffix every row asks for. A suffix no row asks for cannot end a longer one a row
    // asks for (the longer one's last tokens are that suffix), so TRAIN's scan stops there.
    let mut wanted: HashSet<(usize, u64)> = HashSet::new();
    for ctx in contexts {
        let mut h = 0u64;
        for n in 1..=max.min(ctx.len()) {
            h = suffix_hash_step(h, ctx[ctx.len() - n], n);
            wanted.insert((n, h));
        }
    }
    let mut next: HashMap<(usize, u64), HashMap<u32, u32>> = HashMap::new();
    for i in 1..train.len() {
        let target = train[i];
        if target == hs::SHEET_BOS_ID {
            continue;
        }
        let mut h = 0u64;
        for n in 1..=max.min(i) {
            let token = train[i - n];
            h = suffix_hash_step(h, token, n);
            if !wanted.contains(&(n, h)) {
                break;
            }
            *next.entry((n, h)).or_default().entry(target).or_default() += 1;
            if token == hs::SHEET_BOS_ID {
                break;
            }
        }
    }
    contexts
        .iter()
        .map(|ctx| {
            let mut h = 0u64;
            let mut best = CONTROL_MISS;
            for n in 1..=max.min(ctx.len()) {
                h = suffix_hash_step(h, ctx[ctx.len() - n], n);
                let Some(counts) = next.get(&(n, h)) else {
                    break;
                };
                best = counts
                    .iter()
                    .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
                    .map(|(&t, _)| t)
                    .unwrap_or(CONTROL_MISS);
            }
            best
        })
        .collect()
}

/// One split the controls count: its name, tokens, reply mask, and an unpinned panel's sha256.
type ControlSplit = (String, Vec<u32>, Vec<u8>, Option<String>);

/// The protocol's recall controls (I-D, one-model-end-to-end claude-211), counted on the
/// laptop with no model, for DEV, the sealed Fresh V2 panel in `fresh_v2` when given, and each
/// unpinned `--panel`.
/// - **(T), the tables:** at an opening its first-8 key names, TRAIN's commonest opening; at a
///   later row the reply table keeps at share 0.5, its stored token; elsewhere a miss. The
///   memory's own tables, built from TRAIN as `census` builds them.
/// - **(S), the longest suffix** over TRAIN ([`longest_suffix_predictions`], at most
///   [`SUFFIX_CONTROL_MAX`] tokens, in the row's own window).
///
/// Each is written as [`ScoredRows`] (`rows-{split}-T.json`, `rows-{split}-S.json`, with the
/// rows in [`score_rows`]' order), so the grader pairs them with the arms row by row.
pub fn controls(
    data_root: &Path,
    out: &Path,
    fresh_v2: Option<&Path>,
    panels: &[(String, PathBuf)],
) -> Result<Vec<ScoredFile>> {
    let bos = hs::SHEET_BOS_ID;
    let (train_tokens, train_mask) = load_train_split(data_root)?;
    let (train_w, _) = hs::build_document_windows(&train_tokens, &train_mask, SHEET_CONTEXT, bos)?;
    let train_rows = reply_rows(&train_w, &train_mask);
    let opening = reply_table(ReplyKey::TurnPosition(0), &train_rows);
    let reply = reply_table(ReplyKey::TurnPosition(1), &train_rows);
    drop((train_w, train_rows));
    let (dev_tokens, dev_mask) = hs::load_pinned_uort_split(
        &data_root.join("ft/dev/tokens.u16"),
        &data_root.join("ft/dev/response_mask.u8"),
        hs::PINNED_DEV_TOKENS_SHA256,
        hs::PINNED_DEV_MASK_SHA256,
        hs::EXPECTED_DEV_TOKENS,
        hs::EXPECTED_DEV_ELIGIBLE_TARGETS,
        hs::SHEET_VOCAB,
        bos,
    )?;
    let mut splits: Vec<ControlSplit> = vec![("dev".to_owned(), dev_tokens, dev_mask, None)];
    if let Some(dir) = fresh_v2 {
        let (t, m) = load_fresh_v2(dir)?;
        splits.push(("fresh-v2".to_owned(), t, m, None));
    }
    for (name, dir) in panels {
        if !panel_name_ok(name) || splits.iter().any(|s| s.0 == *name) {
            return Err(invalid(format!("--panel {name}: not a usable panel name")));
        }
        let (t, m, sha) = load_unpinned_uort(dir)?;
        splits.push((name.clone(), t, m, Some(sha)));
    }
    fs::create_dir_all(out)?;
    let mut written = Vec::new();
    for (name, tokens, mask, sha) in &splits {
        let (windows, _) = hs::build_document_windows(tokens, mask, SHEET_CONTEXT, bos)?;
        let rows = reply_rows(&windows, mask);
        let mut by_place = std::collections::HashMap::new();
        for row in &rows {
            by_place.insert((row.window, row.offset), row);
        }
        let (mut positions, mut targets, mut tables) = (Vec::new(), Vec::new(), Vec::new());
        let mut named = TableNamed::default();
        let mut contexts: Vec<&[u32]> = Vec::new();
        for (wi, w) in windows.iter().enumerate() {
            for ((&off, &target), &position) in w
                .scored_offsets
                .iter()
                .zip(&w.scored_targets)
                .zip(&w.scored_corpus_positions)
            {
                positions.push(position);
                targets.push(target);
                contexts.push(&w.input_ids[..=off]);
                let row = by_place.get(&(wi, off));
                let stored = row.and_then(|row| {
                    if row.opening {
                        opening.get(&ReplyKey::TurnPosition(0).of(row)).map(|s| s.0)
                    } else {
                        reply
                            .get(&ReplyKey::TurnPosition(1).of(row))
                            .filter(|s| table_keeps(s, REPLY_MEMORY_SHARE))
                            .map(|s| s.0)
                    }
                });
                if let (Some(row), Some(token)) = (row, stored) {
                    let (seen, hit) = if row.opening {
                        (&mut named.openings_named, &mut named.openings_hit)
                    } else {
                        (&mut named.later_named, &mut named.later_hit)
                    };
                    *seen += 1;
                    *hit += usize::from(token == target);
                }
                tables.push(stored.unwrap_or(CONTROL_MISS));
            }
        }
        let suffix = longest_suffix_predictions(&train_tokens, &contexts, SUFFIX_CONTROL_MAX);
        for (label, picks) in [("T", tables), ("S", suffix)] {
            let scored = ScoredRows {
                split: name.clone(),
                memory: false,
                positions: positions.clone(),
                targets: targets.clone(),
                nearest: picks.clone(),
                writer: picks,
            };
            let file = format!("rows-{name}-{label}.json");
            write_new(&out.join(&file), &serde_json::to_vec(&scored)?)?;
            written.push(ScoredFile {
                file,
                rows: scored.positions.len(),
                nearest_hits: Some(scored.hits()),
                writer_hits: None,
                panel_sha256: sha.clone(),
                tables: (label == "T").then_some(named),
                decode: None,
                search: None,
                context: SHEET_CONTEXT,
            });
        }
    }
    Ok(written)
}

/// Where one arm stands against another at one seed of the overturn check.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OverturnCell {
    /// Ahead by more than the band.
    Ahead,
    /// Behind by more than the band.
    Behind,
    /// Within the band.
    Inside,
}

/// `⌈2·√n⌉`, exactly, in integers: the paired band's floor for `n = b + c` discordant rows.
fn two_root_ceil(n: usize) -> usize {
    let four = 4 * n;
    let mut k = (four as f64).sqrt() as usize;
    while k * k < four {
        k += 1;
    }
    while k > 0 && (k - 1) * (k - 1) >= four {
        k -= 1;
    }
    k
}

/// One seed's cell: `b` rows only the first arm gets, `c` rows only the second gets, the band
/// `max(⌈2·√(b+c)⌉, spread)`, and ahead or behind only beyond it.
fn seed_cell(b: usize, c: usize, spread: usize) -> (usize, OverturnCell) {
    let band = two_root_ceil(b + c).max(spread);
    let difference = b as i64 - c as i64;
    let cell = if difference > band as i64 {
        OverturnCell::Ahead
    } else if difference < -(band as i64) {
        OverturnCell::Behind
    } else {
        OverturnCell::Inside
    };
    (band, cell)
}

/// The verdict over the seeds' cells: `ahead` or `behind` only when every seed says so, else
/// `mixed` (the seeds disagree, or one is within its band).
fn overturn_verdict(cells: &[OverturnCell]) -> &'static str {
    if !cells.is_empty() && cells.iter().all(|&c| c == OverturnCell::Ahead) {
        "ahead"
    } else if !cells.is_empty() && cells.iter().all(|&c| c == OverturnCell::Behind) {
        "behind"
    } else {
        "mixed"
    }
}

/// On the rows at `positions`, the hits of `a` and of `b` and the paired discordant counts: rows
/// only `a` gets, and rows only `b` gets. Both must list the same rows in the same order, and
/// hold every one of `positions`.
fn paired(
    a: &ScoredRows,
    b: &ScoredRows,
    positions: &std::collections::BTreeSet<usize>,
) -> Result<(usize, usize, usize, usize)> {
    let whole = |s: &ScoredRows| {
        s.targets.len() == s.positions.len() && s.nearest.len() == s.positions.len()
    };
    if !whole(a) || !whole(b) {
        return Err(invalid("a scored file's lists differ in length"));
    }
    if a.positions != b.positions || a.targets != b.targets {
        return Err(invalid(format!(
            "{} and {} do not score the same rows",
            a.split, b.split
        )));
    }
    let (mut a_hits, mut b_hits, mut only_a, mut only_b, mut seen) = (0, 0, 0, 0, 0);
    for (i, p) in a.positions.iter().enumerate() {
        if !positions.contains(p) {
            continue;
        }
        seen += 1;
        let (x, y) = (a.nearest[i] == a.targets[i], b.nearest[i] == b.targets[i]);
        a_hits += usize::from(x);
        b_hits += usize::from(y);
        only_a += usize::from(x && !y);
        only_b += usize::from(y && !x);
    }
    if seen != positions.len() {
        return Err(invalid(format!(
            "the scored rows hold {seen} of the {} decision rows",
            positions.len()
        )));
    }
    Ok((a_hits, b_hits, only_a, only_b))
}

/// How many rows at `positions` `a` and `b` pick differently: the memory taken out against left
/// in, where the memory names none of them, must be 0.
fn pick_differences(
    a: &ScoredRows,
    b: &ScoredRows,
    positions: &std::collections::BTreeSet<usize>,
) -> usize {
    a.positions
        .iter()
        .zip(a.nearest.iter().zip(&b.nearest))
        .filter(|(p, (x, y))| positions.contains(p) && x != y)
        .count()
}

/// Free-generation measures of a set of replies against TRAIN's (one-model-end-to-end ASSESSMENT
/// §5). A reply is cut after its first end-of-turn id; a cut reply is complete. A share or rate
/// with nothing to count (no replies, or none long enough) is 0.0; read it beside `replies`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Generation {
    pub replies: usize,
    pub complete: usize,
    /// Complete replies equal to a TRAIN reply.
    pub verbatim_train: usize,
    /// For n = 4 to 8: the share of the replies' n-grams that no TRAIN reply holds.
    pub new_ngram_share: Vec<f64>,
    pub distinct_1: f64,
    pub distinct_2: f64,
    /// The share of each reply's 4-grams that repeat an earlier 4-gram of the same reply.
    pub repeated_4gram_rate: f64,
}

/// The n-gram lengths [`Generation::new_ngram_share`] counts.
pub const GENERATION_NGRAMS: std::ops::RangeInclusive<usize> = 4..=8;

/// [`Generation`] for `replies` against `train` (TRAIN's replies, each ending at its `eos`).
fn generation(replies: &[Vec<u32>], train: &[Vec<u32>], eos: u32) -> Generation {
    use std::collections::HashSet;
    let cut: Vec<&[u32]> = replies
        .iter()
        .map(|r| match r.iter().position(|&t| t == eos) {
            Some(at) => &r[..=at],
            None => &r[..],
        })
        .collect();
    let complete: Vec<&[u32]> = cut.iter().copied().filter(|r| r.last() == Some(&eos)).collect();
    let exact: HashSet<u128> = complete.iter().map(|r| census_hash(r)).collect();
    let mut found_exact = HashSet::new();
    for reply in train {
        let h = census_hash(reply);
        if exact.contains(&h) {
            found_exact.insert(h);
        }
    }
    let verbatim_train = complete
        .iter()
        .filter(|r| found_exact.contains(&census_hash(r)))
        .count();
    let mut new_ngram_share = Vec::new();
    for n in GENERATION_NGRAMS {
        let wanted: HashSet<u128> = cut.iter().flat_map(|r| r.windows(n).map(census_hash)).collect();
        let mut seen = HashSet::new();
        for window in train.iter().flat_map(|r| r.windows(n)) {
            let h = census_hash(window);
            if wanted.contains(&h) {
                seen.insert(h);
            }
        }
        let (mut new, mut total) = (0usize, 0usize);
        for window in cut.iter().flat_map(|r| r.windows(n)) {
            total += 1;
            new += usize::from(!seen.contains(&census_hash(window)));
        }
        new_ngram_share.push(if total == 0 { 0.0 } else { new as f64 / total as f64 });
    }
    let share = |n: usize| {
        let total: usize = cut.iter().map(|r| r.len().saturating_sub(n - 1)).sum();
        let distinct: HashSet<&[u32]> = cut.iter().flat_map(|r| r.windows(n)).collect();
        if total == 0 { 0.0 } else { distinct.len() as f64 / total as f64 }
    };
    let (mut repeated, mut grams) = (0usize, 0usize);
    for reply in &cut {
        let mut earlier = HashSet::new();
        for window in reply.windows(4) {
            grams += 1;
            repeated += usize::from(!earlier.insert(window));
        }
    }
    Generation {
        replies: replies.len(),
        complete: complete.len(),
        verbatim_train,
        new_ngram_share,
        distinct_1: share(1),
        distinct_2: share(2),
        repeated_4gram_rate: if grams == 0 { 0.0 } else { repeated as f64 / grams as f64 },
    }
}

/// One scored arm's readings in the overturn check ([`overturn`]).
#[derive(Clone, Debug, Serialize)]
pub struct OverturnArm {
    /// `r-s1`, `r0-s2` and the like: the arm and its seed.
    pub name: String,
    pub dev_hits: usize,
    pub dev_hits_no_memory: usize,
    pub decision_hits: usize,
    pub unkeyed_opening_hits: usize,
    /// Decision rows picked differently with the memory taken out: 0, since the memory names
    /// none of them.
    pub decision_pick_differences: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fresh_v2_hits: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fresh_v2_hits_no_memory: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub samples: Option<Generation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub samples_no_memory: Option<Generation>,
}

/// One seed's comparison of (r) against (r0) on the decision rows.
#[derive(Clone, Debug, Serialize)]
pub struct OverturnSeed {
    pub seed: u8,
    pub r_hits: usize,
    pub r0_hits: usize,
    /// Rows only (r) gets, and rows only (r0) gets.
    pub b: usize,
    pub c: usize,
    pub band: usize,
    pub cell: OverturnCell,
}

/// The overturn check (one-model-end-to-end done-when 2), graded from stored scores.
#[derive(Clone, Debug, Serialize)]
pub struct Overturn {
    pub decision_rows: usize,
    pub unnamed_later: usize,
    pub unkeyed_openings: usize,
    /// `|(b − c) at seed 1 − (b − c) at seed 2|`.
    pub two_seed_spread: usize,
    pub seeds: Vec<OverturnSeed>,
    pub verdict: &'static str,
    pub arms: Vec<OverturnArm>,
    /// DEV's own replies after the same prompts, cut at the reply length the samples had.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<Generation>,
}

/// The decision rows' DEV positions, as the census counts them from the reply and opening
/// tables: `(later, openings)`, the later rows the reply table does not name and the openings no
/// first-8 key names, each sorted.
fn decision_positions(
    reply: &ReplyTableCounts,
    opening: &ReplyTableCounts,
    dev_rows: &[ReplyRow],
    dev_windows: &[CausalWindow],
) -> Result<(Vec<usize>, Vec<usize>)> {
    let (unnamed, unkeyed) = census_decision_rows(reply, opening, dev_rows);
    let positions = |rows: &[usize]| -> Result<Vec<usize>> {
        let mut out = rows
            .iter()
            .map(|&i| {
                row_position(&dev_rows[i], dev_windows)
                    .ok_or_else(|| invalid("a DEV reply row has no corpus position"))
            })
            .collect::<Result<Vec<usize>>>()?;
        out.sort_unstable();
        Ok(out)
    };
    Ok((positions(&unnamed)?, positions(&unkeyed)?))
}

/// Read a JSON file [`score_rows`] wrote.
fn read_scored<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    serde_json::from_slice(&fs::read(path)?)
        .map_err(|e| invalid(format!("{}: {e}", path.display())))
}

/// Grade the overturn check (one-model-end-to-end done-when 2) on the laptop, from the stored
/// outputs of [`score_rows`]: `scored` names each arm's output directory as `ARM-sSEED`, and
/// must hold `r-s1`, `r0-s1`, `r-s2` and `r0-s2`. It recomputes the census's decision rows from
/// TRAIN and DEV, compares (r) with (r0) on them at each seed with the band
/// `max(⌈2·√(b+c)⌉, two-seed spread)`, and scores every arm's samples, and DEV's own replies
/// after the same prompts, against TRAIN's replies. Counts only; no model.
pub fn overturn(data_root: &Path, scored: &[(String, PathBuf)]) -> Result<Overturn> {
    use std::collections::BTreeSet;
    let bos = hs::SHEET_BOS_ID;
    for (name, _) in scored {
        let ok = name
            .split_once("-s")
            .is_some_and(|(arm, seed)| !arm.is_empty() && matches!(seed, "1" | "2"));
        if !ok {
            return Err(invalid(format!("--scored {name}: names are ARM-s1 or ARM-s2")));
        }
    }
    let (train_tokens, train_mask) = load_train_split(data_root)?;
    let (dev_tokens, dev_mask) = hs::load_pinned_uort_split(
        &data_root.join("ft/dev/tokens.u16"),
        &data_root.join("ft/dev/response_mask.u8"),
        hs::PINNED_DEV_TOKENS_SHA256,
        hs::PINNED_DEV_MASK_SHA256,
        hs::EXPECTED_DEV_TOKENS,
        hs::EXPECTED_DEV_ELIGIBLE_TARGETS,
        hs::SHEET_VOCAB,
        bos,
    )?;
    let (train_w, _) = hs::build_document_windows(&train_tokens, &train_mask, SHEET_CONTEXT, bos)?;
    let (dev_w, _) = hs::build_document_windows(&dev_tokens, &dev_mask, SHEET_CONTEXT, bos)?;
    let train_rows = reply_rows(&train_w, &train_mask);
    let dev_rows = reply_rows(&dev_w, &dev_mask);
    let (later, openings) = decision_positions(
        &reply_table(ReplyKey::TurnPosition(1), &train_rows),
        &reply_table(ReplyKey::TurnPosition(0), &train_rows),
        &dev_rows,
        &dev_w,
    )?;
    drop((train_w, train_rows, dev_w, dev_rows));
    let decision: BTreeSet<usize> = later.iter().chain(&openings).copied().collect();
    let unkeyed: BTreeSet<usize> = openings.iter().copied().collect();
    let train_replies = split_turns(&train_tokens, &train_mask, bos).replies;
    let eos = pinned_tokenizer(data_root)?
        .token_id("<|eos|>")
        .ok_or_else(|| invalid("the pinned tokenizer has no <|eos|>"))?;

    let mut arms = Vec::with_capacity(scored.len());
    let mut references = None;
    let mut dev_of = std::collections::BTreeMap::new();
    for (name, dir) in scored {
        let on: ScoredRows = read_scored(&dir.join("rows-dev-memory.json"))?;
        let off: ScoredRows = read_scored(&dir.join("rows-dev-no-memory.json"))?;
        if !on.memory || off.memory || on.split != "dev" || off.split != "dev" {
            return Err(invalid(format!(
                "{}: its DEV files do not hold the memory states their names say",
                dir.display()
            )));
        }
        let (decision_hits, _, _, _) = paired(&on, &off, &decision)?;
        let (unkeyed_opening_hits, _, _, _) = paired(&on, &off, &unkeyed)?;
        let fresh = |label: &str| -> Result<Option<usize>> {
            let path = dir.join(format!("rows-fresh-v2-{label}.json"));
            Ok(match path.exists() {
                true => Some(read_scored::<ScoredRows>(&path)?.hits()),
                false => None,
            })
        };
        let samples = |label: &str| -> Result<Option<Vec<IdSample>>> {
            let path = dir.join(format!("samples-{label}.json"));
            Ok(match path.exists() {
                true => Some(read_scored(&path)?),
                false => None,
            })
        };
        let (with, without) = (samples("memory")?, samples("no-memory")?);
        if references.is_none() {
            if let Some(list) = &with {
                let refs: Vec<Vec<u32>> = list.iter().map(|s| s.reference.clone()).collect();
                references = Some(generation(&refs, &train_replies, eos));
            }
        }
        let grade = |list: Option<Vec<IdSample>>| {
            list.map(|l| {
                let replies: Vec<Vec<u32>> = l.into_iter().map(|s| s.reply).collect();
                generation(&replies, &train_replies, eos)
            })
        };
        arms.push(OverturnArm {
            name: name.clone(),
            dev_hits: on.hits(),
            dev_hits_no_memory: off.hits(),
            decision_hits,
            unkeyed_opening_hits,
            decision_pick_differences: pick_differences(&on, &off, &decision),
            fresh_v2_hits: fresh("memory")?,
            fresh_v2_hits_no_memory: fresh("no-memory")?,
            samples: grade(with),
            samples_no_memory: grade(without),
        });
        dev_of.insert(name.clone(), on);
    }
    let arm = |name: String| {
        dev_of
            .get(&name)
            .ok_or_else(|| invalid(format!("--overturn needs --scored {name}=DIR")))
    };
    let mut counts = Vec::new();
    for seed in [1u8, 2] {
        let (r, r0) = (arm(format!("r-s{seed}"))?, arm(format!("r0-s{seed}"))?);
        counts.push((seed, paired(r, r0, &decision)?));
    }
    let difference = |(_, (_, _, b, c)): &(u8, (usize, usize, usize, usize))| *b as i64 - *c as i64;
    let two_seed_spread = (difference(&counts[0]) - difference(&counts[1])).unsigned_abs() as usize;
    let seeds: Vec<OverturnSeed> = counts
        .iter()
        .map(|&(seed, (r_hits, r0_hits, b, c))| {
            let (band, cell) = seed_cell(b, c, two_seed_spread);
            OverturnSeed { seed, r_hits, r0_hits, b, c, band, cell }
        })
        .collect();
    let cells: Vec<OverturnCell> = seeds.iter().map(|s| s.cell).collect();
    Ok(Overturn {
        decision_rows: decision.len(),
        unnamed_later: later.len(),
        unkeyed_openings: openings.len(),
        two_seed_spread,
        verdict: overturn_verdict(&cells),
        seeds,
        arms,
        references,
    })
}

/// The protocol's trained arms (one-model-end-to-end protocol, "Base artifact: the arms").
pub const PROTOCOL_ARMS: [&str; 7] = [
    "r", "r-drop", "r0", "r-flock", "r-spin", "r-ident", "r-frozen",
];

/// The prompts each split's free generation is sampled at, per rule (the protocol's 256).
pub const PROTOCOL_SAMPLES: usize = 256;

/// The strata's names, each with the split it is drawn from, in [`STRATA_COUNTS`]' order.
pub const STRATA: [(&str, &str); 4] = [
    ("K", "dev"),
    ("U", "dev"),
    ("N-u", "p-dev"),
    ("N-k", "p-dev"),
];

/// The counts the protocol and the training sheet fix for K, U, N-u and N-k.
pub const STRATA_COUNTS: [usize; 4] = [10_849, 3_438, 15_143, 1_274];

/// The protocol's strata (I-G, claude-211) by corpus position: K and U on DEV, N-u and N-k on
/// P-dev. U's rule is "the rows TRAIN's tables name none of", which is whether the tables
/// control (T) of [`controls`] names the row: a named row is keyed, a miss is not.
#[derive(Clone, Debug, Default)]
pub struct Strata {
    pub k: std::collections::BTreeSet<usize>,
    pub u: std::collections::BTreeSet<usize>,
    pub n_u: std::collections::BTreeSet<usize>,
    pub n_k: std::collections::BTreeSet<usize>,
}

impl Strata {
    /// The strata from the (T) files on DEV and on P-dev.
    fn from_tables(dev: &ScoredRows, p_dev: &ScoredRows) -> Result<Self> {
        use std::collections::BTreeSet;
        let split = |rows: &ScoredRows| -> Result<(BTreeSet<usize>, BTreeSet<usize>)> {
            if rows.memory || rows.nearest.len() != rows.positions.len() {
                return Err(invalid(format!(
                    "{}: not a tables control file",
                    rows.split
                )));
            }
            let (mut named, mut not, mut seen) =
                (BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
            for (&p, &pick) in rows.positions.iter().zip(&rows.nearest) {
                if !seen.insert(p) {
                    return Err(invalid(format!(
                        "{}: position {p} is scored twice",
                        rows.split
                    )));
                }
                if pick == CONTROL_MISS {
                    not.insert(p);
                } else {
                    named.insert(p);
                }
            }
            Ok((named, not))
        };
        let (k, u) = split(dev)?;
        let (n_k, n_u) = split(p_dev)?;
        Ok(Self { k, u, n_u, n_k })
    }

    fn get(&self, stratum: &str) -> Option<&std::collections::BTreeSet<usize>> {
        match stratum {
            "K" => Some(&self.k),
            "U" => Some(&self.u),
            "N-u" => Some(&self.n_u),
            "N-k" => Some(&self.n_k),
            _ => None,
        }
    }

    fn counts(&self) -> [usize; 4] {
        [self.k.len(), self.u.len(), self.n_u.len(), self.n_k.len()]
    }
}

/// One seed of a comparison of X with Y on a stratum: the hits of each, the rows only X gets
/// (`b`) and only Y gets (`c`), `d = b − c`, the band and the cell.
#[derive(Clone, Debug, Serialize)]
pub struct GradeSeed {
    pub seed: u8,
    pub x_hits: usize,
    pub y_hits: usize,
    pub b: usize,
    pub c: usize,
    pub d: i64,
    pub band: usize,
    pub cell: OverturnCell,
}

/// X against Y on one stratum over the seeds (the protocol's "Bar: the band and the cell").
#[derive(Clone, Debug, Serialize)]
pub struct GradeCell {
    pub x: String,
    pub y: String,
    pub stratum: &'static str,
    /// `max d − min d` over the seeds.
    pub spread: usize,
    pub seeds: Vec<GradeSeed>,
    /// Ahead or behind only when all three seeds say so; else inside, which is underpowered,
    /// never a negative (L-007).
    pub cell: OverturnCell,
}

/// One seed's paired counts, `(seed, (x_hits, y_hits, b, c))`, as [`paired`] returns them.
type SeedCounts = (u8, (usize, usize, usize, usize));

/// The cell over `counts`, one per seed: each seed's band is `max(⌈2·√(b+c)⌉, max d − min d)`,
/// and the cell is ahead or behind only when all three seeds are.
fn grade_cell(x: &str, y: &str, stratum: &'static str, counts: &[SeedCounts]) -> GradeCell {
    let ds: Vec<i64> = counts
        .iter()
        .map(|(_, (_, _, b, c))| *b as i64 - *c as i64)
        .collect();
    let spread = match (ds.iter().max(), ds.iter().min()) {
        (Some(hi), Some(lo)) => (hi - lo) as usize,
        _ => 0,
    };
    let seeds: Vec<GradeSeed> = counts
        .iter()
        .map(|&(seed, (x_hits, y_hits, b, c))| {
            let (band, cell) = seed_cell(b, c, spread);
            GradeSeed {
                seed,
                x_hits,
                y_hits,
                b,
                c,
                d: b as i64 - c as i64,
                band,
                cell,
            }
        })
        .collect();
    let all = |want: OverturnCell| seeds.len() == 3 && seeds.iter().all(|s| s.cell == want);
    let cell = if all(OverturnCell::Ahead) {
        OverturnCell::Ahead
    } else if all(OverturnCell::Behind) {
        OverturnCell::Behind
    } else {
        OverturnCell::Inside
    };
    GradeCell {
        x: x.to_owned(),
        y: y.to_owned(),
        stratum,
        spread,
        seeds,
        cell,
    }
}

/// Decision 1, the base recipe: `drop_u` and `drop_nu` are (r-drop)'s cells against (r) on U
/// and on N-u (a cell against (r) mirrors (r)'s against it). The arm that ships, and the rule
/// that chose it.
fn base_recipe(drop_u: OverturnCell, drop_nu: OverturnCell) -> (&'static str, &'static str) {
    use OverturnCell::{Ahead, Behind};
    if drop_nu == Ahead && drop_u != Behind {
        ("r-drop", "ahead of (r) on N-u and not behind on U")
    } else if drop_nu == Behind && drop_u != Ahead {
        ("r", "ahead of (r-drop) on N-u and not behind on U")
    } else if drop_u == Ahead && drop_nu != Behind {
        ("r-drop", "ahead of (r) on U and not behind on N-u")
    } else if drop_u == Behind && drop_nu != Ahead {
        ("r", "ahead of (r-drop) on U and not behind on N-u")
    } else {
        (
            "r",
            "neither leads, so (r), the recipe already served in part",
        )
    }
}

/// Decision 2, one add-on against (r): a candidate if ahead on N-u and not behind on U; a
/// negative for this configuration if behind on N-u; else no decision.
fn addon_reading(u: OverturnCell, n_u: OverturnCell) -> &'static str {
    match n_u {
        OverturnCell::Ahead if u != OverturnCell::Behind => "a candidate",
        OverturnCell::Behind => "a negative for this configuration",
        _ => "no decision (underpowered)",
    }
}

/// Decision 3, from (r-ident)'s and (r-frozen)'s cells against (r) on N-u: learned transport
/// earns its place if both are behind; a negative if either is ahead; else no decision.
fn transport_reading(ident_nu: OverturnCell, frozen_nu: OverturnCell) -> &'static str {
    use OverturnCell::{Ahead, Behind};
    if ident_nu == Behind && frozen_nu == Behind {
        "learned transport earns its place"
    } else if ident_nu == Ahead || frozen_nu == Ahead {
        "a negative for learned transport in this configuration"
    } else {
        "no decision (underpowered)"
    }
}

/// `⌈2·√(n·p(1−p))⌉` for `p = g/n`, exactly in integers: the smallest `k` with
/// `n·k² ≥ 4·g·(n − g)`.
fn count_band(g: usize, n: usize) -> usize {
    let want = 4 * g as u128 * n.saturating_sub(g) as u128;
    let n = n as u128;
    let mut k: u128 = 0;
    while n * k * k < want {
        k += 1;
    }
    k as usize
}

/// Decision 4 at one seed: the Flock rank rule holds here if its repeated-4-gram rate is below
/// greedy's and `f ≥ g − ⌈2·√(n·p(1−p))⌉`, with `p = g/n`.
fn decode_holds(g: usize, f: usize, n: usize, g_repeat: f64, f_repeat: f64) -> bool {
    f_repeat < g_repeat && f + count_band(g, n) >= g
}

/// Decision 4 at one seed: complete replies under greedy (`g`) and the Flock rank rule (`f`)
/// out of `n`, the count band, and the repeated-4-gram rates.
#[derive(Clone, Debug, Serialize)]
pub struct DecodeSeed {
    pub seed: u8,
    pub n: usize,
    pub g: usize,
    pub f: usize,
    pub band: usize,
    pub g_repeat: f64,
    pub f_repeat: f64,
    pub holds: bool,
}

/// Decision 4 on the shipped recipe: F ships only if it holds at all three seeds.
#[derive(Clone, Debug, Serialize)]
pub struct DecodeGrade {
    pub recipe: &'static str,
    /// Which Flock rank rule F is: `registered` (the protocol's) or `keeps the end of turn`
    /// (`DecodeRule::FLOCK_END`, added after the run's first grade; Casey, 30 Oct).
    pub flock_rule: &'static str,
    pub seeds: Vec<DecodeSeed>,
    pub ships: &'static str,
}

/// Decision 5's free-generation readings at one seed, on P-dev's replies under the shipped rule
/// against P-dev's own references after the same prompts.
#[derive(Clone, Debug, Serialize)]
pub struct VerdictSeed {
    pub seed: u8,
    pub complete: usize,
    pub verbatim: usize,
    pub reference_complete: usize,
    pub reference_verbatim: usize,
    /// `v`, `v_ref` and `tol`; none when there is nothing complete to share over.
    pub v: Option<f64>,
    pub v_ref: Option<f64>,
    pub tol: Option<f64>,
    pub repeat: f64,
    pub reference_repeat: f64,
}

impl VerdictSeed {
    fn new(seed: u8, replies: &Generation, references: &Generation) -> Self {
        let share = |part: usize, whole: usize| (whole > 0).then(|| part as f64 / whole as f64);
        let v = share(replies.verbatim_train, replies.complete);
        let v_ref = share(references.verbatim_train, references.complete);
        let tol = match (v, v_ref) {
            (Some(_), Some(r)) => {
                Some(0.10f64.max(3.0 * (r * (1.0 - r) / replies.complete as f64).sqrt()))
            }
            _ => None,
        };
        Self {
            seed,
            complete: replies.complete,
            verbatim: replies.verbatim_train,
            reference_complete: references.complete,
            reference_verbatim: references.verbatim_train,
            v,
            v_ref,
            tol,
            repeat: replies.repeated_4gram_rate,
            reference_repeat: references.repeated_4gram_rate,
        }
    }

    /// "generalizes"' free-generation condition: `n_c ≥ 64`, `|v − v_ref| ≤ tol`, and a
    /// repeated-4-gram rate at most the references' + 0.10.
    fn generalizes(&self) -> bool {
        let close = matches!((self.v, self.v_ref, self.tol), (Some(v), Some(r), Some(t)) if (v - r).abs() <= t);
        self.complete >= 64 && close && self.repeat <= self.reference_repeat + 0.10
    }

    /// "memorized"' free-generation condition: `v − v_ref > tol`.
    fn memorized(&self) -> bool {
        matches!((self.v, self.v_ref, self.tol), (Some(v), Some(r), Some(t)) if v - r > t)
    }
}

/// Decision 5 on the shipped recipe with the shipped decode rule.
#[derive(Clone, Debug, Serialize)]
pub struct VerdictGrade {
    pub recipe: &'static str,
    pub rule: &'static str,
    pub flock_rule: &'static str,
    pub seeds: Vec<VerdictSeed>,
    pub verdict: &'static str,
    /// For "mixed", each condition that failed.
    pub failed: Vec<String>,
}

/// The verdict: `beats` are the recipe's cells against (r0) on U, (r0) on N-u, (S) on U and
/// (S) on N-u; `keyed` its cell against (r0) on K. "generalizes" needs all four ahead and every
/// seed's generalizing condition; "memorized" needs K ahead, none of the four ahead, and every
/// seed's memorized condition; else "mixed", with what failed.
fn verdict(
    beats: [OverturnCell; 4],
    keyed: OverturnCell,
    seeds: &[VerdictSeed],
) -> (&'static str, Vec<String>) {
    let names = ["(r0) on U", "(r0) on N-u", "(S) on U", "(S) on N-u"];
    let three = seeds.len() == 3;
    let ahead = beats.map(|c| c == OverturnCell::Ahead);
    if ahead.iter().all(|&a| a) && three && seeds.iter().all(VerdictSeed::generalizes) {
        return ("generalizes", Vec::new());
    }
    if keyed == OverturnCell::Ahead
        && !ahead.iter().any(|&a| a)
        && three
        && seeds.iter().all(VerdictSeed::memorized)
    {
        return ("memorized", Vec::new());
    }
    let mut failed: Vec<String> = names
        .iter()
        .zip(ahead)
        .filter(|(_, a)| !a)
        .map(|(n, _)| format!("generalizes: not ahead of {n}"))
        .collect();
    if !three {
        failed.push(format!("{} of 3 seeds graded", seeds.len()));
    }
    for s in seeds.iter().filter(|s| !s.generalizes()) {
        failed.push(format!(
            "generalizes: seed {} fails n_c ≥ 64, |v − v_ref| ≤ tol or the repeat rate",
            s.seed
        ));
    }
    if keyed != OverturnCell::Ahead {
        failed.push("memorized: not ahead of (r0) on K".to_owned());
    }
    for s in seeds.iter().filter(|s| !s.memorized()) {
        failed.push(format!(
            "memorized: seed {} has v − v_ref ≤ tol, or nothing complete",
            s.seed
        ));
    }
    ("mixed", failed)
}

/// I-C under `frozen` (claude-211): in the run at `run`, every `.spiralcore_fbs` routing
/// tensor of the step-0 checkpoint equals its step-256 one, bit for bit, while some other tensor
/// moved, so the run trained. `(routing tensors compared, other tensors that moved)`.
///
/// It reads each `model.safetensors` as bytes, never as tensors (the CPU device refuses model
/// compute, D24/D29): an 8-byte little-endian header length, a JSON index of each tensor's
/// dtype, shape and `data_offsets`, then the raw data. Two tensors are the same when their
/// dtype, shape and bytes are.
pub fn frozen_check(run: &Path) -> Result<(usize, usize)> {
    use serde_json::{Map, Value};
    let read = |step: &str| -> Result<(Vec<u8>, Map<String, Value>)> {
        let mut bytes = fs::read(run.join("models").join(step).join("model.safetensors"))?;
        let head: [u8; 8] = bytes
            .get(..8)
            .and_then(|h| h.try_into().ok())
            .ok_or_else(|| invalid(format!("{step}: no safetensors header")))?;
        let end = usize::try_from(u64::from_le_bytes(head))
            .ok()
            .and_then(|n| n.checked_add(8))
            .ok_or_else(|| invalid(format!("{step}: the header length overflows")))?;
        let n = end - 8;
        let index: Value = serde_json::from_slice(
            bytes
                .get(8..end)
                .ok_or_else(|| invalid(format!("{step}: the header runs past the file")))?,
        )?;
        let index = index
            .as_object()
            .cloned()
            .ok_or_else(|| invalid(format!("{step}: the header is not an object")))?;
        Ok((bytes.split_off(8 + n), index))
    };
    let ((start_data, start), (end_data, end)) = (read("step-000000")?, read("step-000256")?);
    let tensor = |data: &[u8], info: &Value| -> Option<(Value, Value, Vec<u8>)> {
        let offsets = info["data_offsets"].as_array()?;
        let (from, to) = (offsets.first()?.as_u64()? as usize, offsets.get(1)?.as_u64()? as usize);
        Some((info["dtype"].clone(), info["shape"].clone(), data.get(from..to)?.to_vec()))
    };
    let (mut routing, mut moved) = (0, 0);
    for (name, a) in start.iter().filter(|(name, _)| name.as_str() != "__metadata__") {
        let b = end
            .get(name)
            .ok_or_else(|| invalid(format!("step 256 has no {name}")))?;
        let (a, b) = (tensor(&start_data, a), tensor(&end_data, b));
        // A tensor that cannot be read is refused, never counted as moved.
        let (Some(a), Some(b)) = (a, b) else {
            return Err(invalid(format!("{name}: its data cannot be read")));
        };
        let same = a == b;
        if name.ends_with(".spiralcore_fbs") {
            if !same {
                return Err(invalid(format!("{name} moved under frozen")));
            }
            routing += 1;
        } else {
            moved += usize::from(!same);
        }
    }
    if routing == 0 || moved == 0 {
        return Err(invalid(format!(
            "{routing} routing tensors and {moved} moved tensors: the check proves nothing"
        )));
    }
    Ok((routing, moved))
}

/// The grade of the protocol's training run (I-G, one-model-end-to-end claude-211), from stored
/// outputs only, on the laptop.
#[derive(Clone, Debug, Serialize)]
pub struct Grade {
    /// K, U, N-u and N-k, as [`STRATA_COUNTS`] fixes them.
    pub strata: [usize; 4],
    /// Each stratum's rows that copy the prompt (C, reported), in the same order.
    pub copied: [usize; 4],
    /// I-F: DEV's later rows, those whose decoder key is the census's `reply_key`, and those the
    /// reply table names (13,061, 13,061 and 9,655).
    pub hit_keys: [usize; 3],
    /// I-C under `frozen`, when a probe is given: routing tensors compared, other tensors moved.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frozen: Option<[usize; 2]>,
    /// Arm-seeds with no rows: an arm its probe skipped, or a failed step.
    pub missing: Vec<String>,
    pub cells: Vec<GradeCell>,
    pub recipe: &'static str,
    pub recipe_rule: &'static str,
    pub addons: Vec<(&'static str, &'static str)>,
    pub transport: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decode: Option<DecodeGrade>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verdict: Option<VerdictGrade>,
    /// Hits on each stratum's copied rows, per arm-seed (C).
    pub copied_hits: Vec<(String, [usize; 4])>,
    /// Why a decision could not be graded.
    pub notes: Vec<String>,
}

/// The rows [`grade`] compares for `name` (an arm, or the control `T` or `S`) at `seed` on
/// `split`; a control has no seed.
fn rows_in<'m>(
    controls: &'m std::collections::BTreeMap<(&'static str, &'static str), ScoredRows>,
    arms: &'m std::collections::BTreeMap<(&'static str, u8, &'static str), ScoredRows>,
    name: &str,
    seed: u8,
    split: &str,
) -> Option<&'m ScoredRows> {
    let split = *["dev", "p-dev"].iter().find(|&&s| s == split)?;
    if let Some(&control) = ["T", "S"].iter().find(|&&c| c == name) {
        return controls.get(&(control, split));
    }
    let arm = *PROTOCOL_ARMS.iter().find(|&&a| a == name)?;
    arms.get(&(arm, seed, split))
}

/// Grade the training run (I-G): `controls` holds [`controls`]' files, `runs` one score folder
/// per arm-seed named `ARM-sSEED`, `p_dev` the panel, and `frozen_probe` (r-frozen)'s probe run.
/// It refuses strata other than [`STRATA_COUNTS`], and a U that is not the census's decision
/// rows; then applies the protocol's five decisions in order. Counts only; no model. With
/// `flock_end`, decisions 4 and 5 read the end-of-turn Flock rule's samples
/// (`samples-*-flock-end-memory.json`) in place of the registered rule's, and say so.
/// This mode was added after the run's first grade, at Casey's direction (30 Oct).
pub fn grade(
    data_root: &Path,
    controls_dir: &Path,
    runs: &Path,
    p_dev: &Path,
    frozen_probe: Option<&Path>,
    flock_end: bool,
) -> Result<Grade> {
    use std::collections::{BTreeMap, BTreeSet};
    let (flock_rule, flock_label) = if flock_end {
        ("keeps the end of turn", "flock-end")
    } else {
        ("registered", "flock")
    };
    let bos = hs::SHEET_BOS_ID;
    let control = |label: &str, split: &str| -> Result<ScoredRows> {
        read_scored(&controls_dir.join(format!("rows-{split}-{label}.json")))
    };
    let mut controls_of: BTreeMap<(&'static str, &'static str), ScoredRows> = BTreeMap::new();
    for label in ["T", "S"] {
        for split in ["dev", "p-dev"] {
            controls_of.insert((label, split), control(label, split)?);
        }
    }
    let strata = Strata::from_tables(&controls_of[&("T", "dev")], &controls_of[&("T", "p-dev")])?;
    if strata.counts() != STRATA_COUNTS {
        return Err(invalid(format!(
            "the strata are {:?}, not the sheet's {STRATA_COUNTS:?}",
            strata.counts()
        )));
    }

    // U against the census, I-F's full count, and C, from TRAIN, DEV and P-dev.
    let (train_tokens, train_mask) = load_train_split(data_root)?;
    let (dev_tokens, dev_mask) = hs::load_pinned_uort_split(
        &data_root.join("ft/dev/tokens.u16"),
        &data_root.join("ft/dev/response_mask.u8"),
        hs::PINNED_DEV_TOKENS_SHA256,
        hs::PINNED_DEV_MASK_SHA256,
        hs::EXPECTED_DEV_TOKENS,
        hs::EXPECTED_DEV_ELIGIBLE_TARGETS,
        hs::SHEET_VOCAB,
        bos,
    )?;
    let (train_w, _) = hs::build_document_windows(&train_tokens, &train_mask, SHEET_CONTEXT, bos)?;
    let train_rows = reply_rows(&train_w, &train_mask);
    let reply = reply_table(ReplyKey::TurnPosition(1), &train_rows);
    let opening = reply_table(ReplyKey::TurnPosition(0), &train_rows);
    drop((train_w, train_rows));
    let (dev_w, _) = hs::build_document_windows(&dev_tokens, &dev_mask, SHEET_CONTEXT, bos)?;
    let dev_rows = reply_rows(&dev_w, &dev_mask);
    let (later, openings) = decision_positions(&reply, &opening, &dev_rows, &dev_w)?;
    let census_u: BTreeSet<usize> = later.iter().chain(&openings).copied().collect();
    if census_u != strata.u {
        return Err(invalid(format!(
            "U from the tables control ({}) is not the census's {} decision rows",
            strata.u.len(),
            census_u.len()
        )));
    }
    let mut hit_keys = [0usize; 3];
    for row in dev_rows.iter().filter(|r| !r.opening) {
        let ids = &dev_w[row.window].input_ids;
        let key = reply_key(row);
        let decoded = row
            .offset
            .checked_sub(row.reply.len())
            .and_then(|at| greedy_reply_key(&ids[..=row.offset], at, &row.reply));
        hit_keys[0] += 1;
        hit_keys[1] += usize::from(decoded.as_ref() == Some(&key));
        hit_keys[2] += usize::from(
            reply
                .get(&key)
                .is_some_and(|s| table_keeps(s, REPLY_MEMORY_SHARE)),
        );
    }
    let copied_at = |rows: &[ReplyRow], windows: &[CausalWindow]| -> Result<BTreeSet<usize>> {
        rows.iter()
            .filter(|r| r.copy)
            .map(|r| {
                row_position(r, windows)
                    .ok_or_else(|| invalid("a reply row has no corpus position"))
            })
            .collect()
    };
    let dev_copied = copied_at(&dev_rows, &dev_w)?;
    drop((dev_w, dev_rows));
    let (p_tokens, p_mask, _) = load_unpinned_uort(p_dev)?;
    let (p_w, _) = hs::build_document_windows(&p_tokens, &p_mask, SHEET_CONTEXT, bos)?;
    let p_copied = copied_at(&reply_rows(&p_w, &p_mask), &p_w)?;
    drop(p_w);
    let copied_in = |stratum: &str| -> BTreeSet<usize> {
        let rows = strata.get(stratum).expect("a named stratum");
        let copied = if stratum.starts_with('N') {
            &p_copied
        } else {
            &dev_copied
        };
        rows.intersection(copied).copied().collect()
    };
    let copied_sets: Vec<BTreeSet<usize>> = STRATA.iter().map(|(s, _)| copied_in(s)).collect();
    let train_replies = split_turns(&train_tokens, &train_mask, bos).replies;
    let eos = pinned_tokenizer(data_root)?
        .token_id("<|eos|>")
        .ok_or_else(|| invalid("the pinned tokenizer has no <|eos|>"))?;

    // Every arm-seed's rows, memory on, as served.
    let mut arms: BTreeMap<(&'static str, u8, &'static str), ScoredRows> = BTreeMap::new();
    let mut missing = Vec::new();
    let mut copied_hits = Vec::new();
    for arm in PROTOCOL_ARMS {
        for seed in 1..=3u8 {
            let dir = runs.join(format!("{arm}-s{seed}"));
            // An arm-seed is graded only with both its splits: one with a file missing is
            // listed as missing and enters no cell (claude-211 review).
            let paths = ["dev", "p-dev"].map(|split| (split, dir.join(format!("rows-{split}-memory.json"))));
            if paths.iter().any(|(_, path)| !path.exists()) {
                missing.push(format!("{arm}-s{seed}"));
                continue;
            }
            for (split, path) in paths {
                let rows: ScoredRows = read_scored(&path)?;
                if !rows.memory || rows.split != split {
                    return Err(invalid(format!(
                        "{}: not {split}'s rows with the memory on",
                        path.display()
                    )));
                }
                arms.insert((arm, seed, split), rows);
            }
            let mut hits = [0usize; 4];
            for (i, (_, split)) in STRATA.iter().enumerate() {
                let rows = &arms[&(arm, seed, *split)];
                hits[i] = rows
                    .positions
                    .iter()
                    .zip(rows.nearest.iter().zip(&rows.targets))
                    .filter(|(p, (n, t))| copied_sets[i].contains(p) && n == t)
                    .count();
            }
            copied_hits.push((format!("{arm}-s{seed}"), hits));
        }
    }
    let rows_of =
        |name: &str, seed: u8, split: &str| rows_in(&controls_of, &arms, name, seed, split);
    let mut cells = Vec::new();
    let mut compare = |x: &str, y: &str, stratum: &'static str| -> Result<OverturnCell> {
        let split = STRATA
            .iter()
            .find(|(s, _)| *s == stratum)
            .map(|(_, split)| *split)
            .ok_or_else(|| invalid(format!("no stratum {stratum}")))?;
        let positions = strata.get(stratum).expect("a named stratum");
        let mut counts = Vec::new();
        for seed in 1..=3u8 {
            if let (Some(a), Some(b)) = (rows_of(x, seed, split), rows_of(y, seed, split)) {
                counts.push((seed, paired(a, b, positions)?));
            }
        }
        let cell = grade_cell(x, y, stratum, &counts);
        let out = cell.cell;
        cells.push(cell);
        Ok(out)
    };

    // 1. The base recipe; 2. the add-ons; 3. the transport (U reported beside N-u).
    let (recipe, recipe_rule) =
        base_recipe(compare("r-drop", "r", "U")?, compare("r-drop", "r", "N-u")?);
    let mut addons = Vec::new();
    for addon in ["r-flock", "r-spin"] {
        let u = compare(addon, "r", "U")?;
        addons.push((addon, addon_reading(u, compare(addon, "r", "N-u")?)));
    }
    compare("r-ident", "r", "U")?;
    compare("r-frozen", "r", "U")?;
    let transport = transport_reading(
        compare("r-ident", "r", "N-u")?,
        compare("r-frozen", "r", "N-u")?,
    );
    // The shipped recipe against the controls, for decision 5 (N-k and K against (T) reported).
    let keyed = compare(recipe, "r0", "K")?;
    compare(recipe, "T", "K")?;
    let beats = [
        compare(recipe, "r0", "U")?,
        compare(recipe, "r0", "N-u")?,
        compare(recipe, "S", "U")?,
        compare(recipe, "S", "N-u")?,
    ];
    compare(recipe, "r0", "N-k")?;

    // 4. The decode rule, and 5. the verdict, from the shipped recipe's stored samples.
    let mut notes = Vec::new();
    let samples = |seed: u8, file: &str| -> Result<Vec<IdSample>> {
        let path = runs.join(format!("{recipe}-s{seed}")).join(file);
        let list: Vec<IdSample> = read_scored(&path)?;
        if list.len() != PROTOCOL_SAMPLES || list.iter().any(|s| s.reference.is_empty()) {
            return Err(invalid(format!(
                "{}: {} samples, or a prompt that opens no reply (I-A)",
                path.display(),
                list.len()
            )));
        }
        Ok(list)
    };
    let replies = |lists: Vec<Vec<IdSample>>| -> Vec<Vec<u32>> {
        lists.into_iter().flatten().map(|s| s.reply).collect()
    };
    let mut decode_seeds = Vec::new();
    let mut verdict_lists = Vec::new();
    for seed in 1..=3u8 {
        let read = || -> Result<(DecodeSeed, Vec<IdSample>, Vec<IdSample>)> {
            let p_greedy = samples(seed, "samples-p-dev-greedy-memory.json")?;
            let p_flock = samples(seed, &format!("samples-p-dev-{flock_label}-memory.json"))?;
            let g = replies(vec![
                samples(seed, "samples-memory.json")?,
                p_greedy.clone(),
            ]);
            let f = replies(vec![
                samples(seed, &format!("samples-dev-{flock_label}-memory.json"))?,
                p_flock.clone(),
            ]);
            let (gg, fg) = (
                generation(&g, &train_replies, eos),
                generation(&f, &train_replies, eos),
            );
            let n = g.len();
            Ok((
                DecodeSeed {
                    seed,
                    n,
                    g: gg.complete,
                    f: fg.complete,
                    band: count_band(gg.complete, n),
                    g_repeat: gg.repeated_4gram_rate,
                    f_repeat: fg.repeated_4gram_rate,
                    holds: decode_holds(
                        gg.complete,
                        fg.complete,
                        n,
                        gg.repeated_4gram_rate,
                        fg.repeated_4gram_rate,
                    ),
                },
                p_greedy,
                p_flock,
            ))
        };
        match read() {
            Ok((d, p_greedy, p_flock)) => {
                decode_seeds.push(d);
                verdict_lists.push((seed, p_greedy, p_flock));
            }
            Err(e) => notes.push(format!(
                "{recipe}-s{seed}: not graded for decisions 4 and 5: {e}"
            )),
        }
    }
    let flock = decode_seeds.len() == 3 && decode_seeds.iter().all(|d| d.holds);
    let rule = if flock { "F" } else { "G" };
    let verdict_seeds: Vec<VerdictSeed> = verdict_lists
        .into_iter()
        .map(|(seed, p_greedy, p_flock)| {
            let list = if flock { p_flock } else { p_greedy };
            let references: Vec<Vec<u32>> = list.iter().map(|s| s.reference.clone()).collect();
            let reply: Vec<Vec<u32>> = list.into_iter().map(|s| s.reply).collect();
            VerdictSeed::new(
                seed,
                &generation(&reply, &train_replies, eos),
                &generation(&references, &train_replies, eos),
            )
        })
        .collect();
    let (name, failed) = verdict(beats, keyed, &verdict_seeds);
    let frozen = match frozen_probe {
        Some(run) => {
            let (routing, moved) = frozen_check(run)?;
            Some([routing, moved])
        }
        None => None,
    };
    Ok(Grade {
        strata: strata.counts(),
        copied: [0, 1, 2, 3].map(|i| copied_sets[i].len()),
        hit_keys,
        frozen,
        missing,
        cells,
        recipe,
        recipe_rule,
        addons,
        transport,
        decode: (!decode_seeds.is_empty()).then_some(DecodeGrade {
            recipe,
            flock_rule,
            seeds: decode_seeds,
            ships: rule,
        }),
        verdict: Some(VerdictGrade {
            recipe,
            rule,
            flock_rule,
            seeds: verdict_seeds,
            verdict: name,
            failed,
        }),
        copied_hits,
        notes,
    })
}

/// One decode rule's free-generation reading on one split at one seed (claude-213's
/// `--cell-grade`), against that split's own references after the same prompts.
#[derive(Clone, Debug, Serialize)]
pub struct CellReading {
    pub split: String,
    /// `greedy`, `flock-end`, or `flock-cell-{margin}`.
    pub rule: String,
    pub seed: u8,
    pub reading: VerdictSeed,
    /// Decision 4 against greedy on the same prompts ([`decode_holds`]); none for greedy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub holds: Option<bool>,
    /// The free-generation condition of "generalizes" ([`VerdictSeed::generalizes`]).
    pub close: bool,
    /// The cell rule's admitted count per step, averaged over every step of every reply.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean_admitted: Option<f64>,
    /// The replies that repeat a 4-gram of their own ([`looping_replies`]), when counted
    /// (claude-215's `read_grade`; `cell_grade` writes none, so its files are as before).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loops: Option<usize>,
    /// Whether this rule loops in fewer replies than greedy on the same prompts, by more than
    /// decision 4's band `⌈2·√(n·p(1−p))⌉` at greedy's count; none for greedy, or uncounted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fewer_loops: Option<bool>,
    /// The replies' mean length in tokens, each cut after its first end of turn, when loops are
    /// counted: a rule that ends replies sooner has fewer 4-grams to repeat.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean_length: Option<f64>,
    /// The prompts every compared rule completes, and this rule's verbatim TRAIN replies among
    /// them, when matched (claude-216's `cut_grade`): a content share no rule's completions select.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_n: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_verbatim: Option<usize>,
}

impl CellReading {
    /// The verbatim TRAIN share on the matched prompts, when matched and any.
    fn matched_v(&self) -> Option<f64> {
        match (self.matched_verbatim, self.matched_n) {
            (Some(v), Some(n)) if n > 0 => Some(v as f64 / n as f64),
            _ => None,
        }
    }
}

impl CellReading {
    /// `|v − v_ref|`, or none when nothing is complete to share over.
    fn gap(&self) -> Option<f64> {
        match (self.reading.v, self.reading.v_ref) {
            (Some(v), Some(r)) => Some((v - r).abs()),
            _ => None,
        }
    }
}

/// claude-213's grade of the cell rule: every reading, DEV's selection and the panel's outcome.
#[derive(Clone, Debug, Serialize)]
pub struct CellGrade {
    pub panel: String,
    pub margins: Vec<u32>,
    pub readings: Vec<CellReading>,
    pub selected: Option<u32>,
    /// `qualified`, `closest, not qualified`, or `none holds decision 4`.
    pub selection: &'static str,
    /// The selected margin's outcome on the panel: `improves`, `narrows`, `no gain`, or
    /// `no selection`.
    pub outcome: &'static str,
}

/// DEV's selection, as the claude-213 sheet registers it, from each margin's three DEV seeds:
/// - the widest margin that holds decision 4 against greedy and is close (free generation,
///   [`VerdictSeed::generalizes`]) at all three seeds: `qualified`;
/// - else, of those that hold decision 4 at all three seeds, the one nearest the references by
///   the mean `|v − v_ref|` over its seeds, ties to the wider: `closest, not qualified`;
/// - else none.
fn select_margin(dev: &[(u32, Vec<&CellReading>)]) -> (Option<u32>, &'static str) {
    let three = |r: &[&CellReading], f: &dyn Fn(&CellReading) -> bool| r.len() == 3 && r.iter().all(|x| f(x));
    let mut by_width: Vec<&(u32, Vec<&CellReading>)> = dev.iter().collect();
    by_width.sort_by_key(|r| std::cmp::Reverse(r.0));
    if let Some((m, _)) = by_width
        .iter()
        .find(|(_, r)| three(r, &|x| x.holds == Some(true) && x.close))
    {
        return (Some(*m), "qualified");
    }
    let mut best: Option<(u32, f64)> = None;
    for (m, r) in by_width {
        if !three(r, &|x| x.holds == Some(true)) {
            continue;
        }
        let gaps: Option<Vec<f64>> = r.iter().map(|x| x.gap()).collect();
        let Some(gaps) = gaps else { continue };
        let mean = gaps.iter().sum::<f64>() / gaps.len() as f64;
        // Widest first, so a later margin replaces it only when strictly nearer.
        if best.is_none_or(|(_, b)| mean < b) {
            best = Some((*m, mean));
        }
    }
    match best {
        Some((m, _)) => (Some(m), "closest, not qualified"),
        None => (None, "none holds decision 4"),
    }
}

/// The panel's outcome for the selected margin's three seeds `cell`, against `FLOCK_END`'s
/// three seeds `end` there, as the sheet registers it: `improves` when it holds decision 4 and
/// is close at all three; `narrows` when it holds decision 4 at all three and is nearer the
/// references than `FLOCK_END` at all three; else `no gain`.
fn panel_outcome(cell: &[&CellReading], end: &[&CellReading]) -> &'static str {
    let holds = cell.len() == 3 && cell.iter().all(|x| x.holds == Some(true));
    if holds && cell.iter().all(|x| x.close) {
        return "improves";
    }
    let nearer = end.len() == 3
        && cell.iter().zip(end).all(|(c, e)| match (c.gap(), e.gap()) {
            (Some(c), Some(e)) => c < e,
            (Some(_), None) => true,
            _ => false,
        });
    if holds && nearer { "narrows" } else { "no gain" }
}

/// Grade the cell rule (claude-213) from stored samples only, on the laptop: `runs` holds one
/// `--score-rows` folder per seed, `r-drop-s{1,2,3}`, each with DEV's and `panel`'s samples
/// under greedy, `FLOCK_END` and the cell rule at each of `margins`. Every reading is against
/// the split's own references after the same prompts; DEV selects the margin
/// ([`select_margin`]) and the panel decides ([`panel_outcome`]). Counts only; no model.
pub fn cell_grade(data_root: &Path, runs: &Path, panel: &str, margins: &[u32]) -> Result<CellGrade> {
    if margins.is_empty() || !cells_ok(margins) {
        return Err(invalid("--cell-grade wants --flock-cell's distinct margins"));
    }
    if panel == "dev" {
        return Err(invalid("--cell-grade's confirming panel cannot be DEV, which selects"));
    }
    let mut rules = vec!["greedy".to_owned(), "flock-end".to_owned()];
    rules.extend(margins.iter().map(|m| format!("flock-cell-{m}")));
    let readings = rule_readings(data_root, runs, panel, &rules, false, 0)?;
    let of = |split: &str, rule: &str| -> Vec<&CellReading> {
        readings.iter().filter(|r| r.split == split && r.rule == rule).collect()
    };
    let dev: Vec<(u32, Vec<&CellReading>)> = margins
        .iter()
        .map(|&m| (m, of("dev", &format!("flock-cell-{m}"))))
        .collect();
    let (selected, selection) = select_margin(&dev);
    let outcome = match selected {
        Some(m) => panel_outcome(&of(panel, &format!("flock-cell-{m}")), &of(panel, "flock-end")),
        None => "no selection",
    };
    Ok(CellGrade {
        panel: panel.to_owned(),
        margins: margins.to_vec(),
        readings,
        selected,
        selection,
        outcome,
    })
}

/// How many of `replies`, each cut after its first `eos`, repeat a 4-gram of their own: the
/// replies that loop (claude-215).
fn looping_replies(replies: &[Vec<u32>], eos: u32) -> usize {
    replies.iter().filter(|r| loops(r, eos)).count()
}

/// Whether `reply`, cut after its first `eos`, repeats a 4-gram of its own ([`looping_replies`]).
fn loops(reply: &[u32], eos: u32) -> bool {
    let cut = cut_at_eos(reply, eos);
    let mut seen = std::collections::HashSet::new();
    cut.windows(4).any(|w| !seen.insert(w))
}

/// Whether `loops` looping replies of `n` are fewer than greedy's `greedy_loops` by more than
/// decision 4's band at greedy's count, `⌈2·√(n·p(1−p))⌉` (claude-215).
fn fewer_loops(loops: usize, greedy_loops: usize, n: usize) -> bool {
    loops + count_band(greedy_loops, n) < greedy_loops
}

/// From each rule's `(complete, complete and verbatim)` flags over the same prompts: how many
/// prompts the first `by` rules all complete, and each rule's verbatim replies among them
/// (claude-216). A rule past the first `by` is counted on their prompts and selects none.
fn matched_counts(flags: &[Vec<(bool, bool)>], by: usize) -> (usize, Vec<usize>) {
    let n = flags.first().map_or(0, Vec::len);
    let both: Vec<usize> = (0..n).filter(|&j| flags.iter().take(by).all(|f| f[j].0)).collect();
    let verbatim = flags
        .iter()
        .map(|f| both.iter().filter(|&&j| f[j].1).count())
        .collect();
    (both.len(), verbatim)
}

/// Every reading [`cell_grade`], [`read_grade`] and [`cut_grade`] compare: for DEV and `panel`,
/// at seeds 1–3, each of `rules`' samples files against the split's own references, greedy
/// (`rules[0]`) first. A rule is `name` (in `runs/r-drop-s{seed}/`) or `run/name` (in
/// `runs/{run}-s{seed}/`, claude-216). Every rule must answer greedy's prompts. With
/// `count_loops`, each reading also counts its looping replies, and every rule after greedy
/// whether it has fewer than greedy's beyond decision 4's band. With `matched` above 0, each
/// reading also counts its verbatim TRAIN replies on the prompts the first `matched` rules all
/// complete ([`CellReading::matched_n`], [`matched_counts`]).
fn rule_readings(
    data_root: &Path,
    runs: &Path,
    panel: &str,
    rules: &[String],
    count_loops: bool,
    matched: usize,
) -> Result<Vec<CellReading>> {
    let matched = matched.min(rules.len());
    if rules.first().map(String::as_str) != Some("greedy") {
        return Err(invalid("the readings compare against greedy, which comes first"));
    }
    let (train_tokens, train_mask) = load_train_split(data_root)?;
    let train_replies = split_turns(&train_tokens, &train_mask, hs::SHEET_BOS_ID).replies;
    drop((train_tokens, train_mask));
    let eos = pinned_tokenizer(data_root)?
        .token_id("<|eos|>")
        .ok_or_else(|| invalid("the pinned tokenizer has no <|eos|>"))?;
    let train_set: std::collections::HashSet<u128> = if matched > 0 {
        train_replies.iter().map(|r| census_hash(r)).collect()
    } else {
        std::collections::HashSet::new()
    };
    let samples = |seed: u8, run: &str, file: &str| -> Result<Vec<IdSample>> {
        let path = runs.join(format!("{run}-s{seed}")).join(file);
        let list: Vec<IdSample> = read_scored(&path)?;
        if list.len() != PROTOCOL_SAMPLES || list.iter().any(|s| s.reference.is_empty()) {
            return Err(invalid(format!(
                "{}: {} samples, or a prompt that opens no reply",
                path.display(),
                list.len()
            )));
        }
        Ok(list)
    };
    let mut readings = Vec::new();
    for split in ["dev", panel] {
        for seed in 1..=3u8 {
            let mut greedy: Option<Generation> = None;
            let mut greedy_loops = 0usize;
            let mut prompts: Option<Vec<(usize, Vec<u32>)>> = None;
            let first_reading = readings.len();
            // Each rule's replies: complete, and complete and verbatim in TRAIN (`matched`).
            let mut flags: Vec<Vec<(bool, bool)>> = Vec::new();
            for rule in rules {
                let (run, name) = rule.split_once('/').unwrap_or(("r-drop", rule.as_str()));
                let file = match (split, name) {
                    ("dev", "greedy") => "samples-memory.json".to_owned(),
                    (_, "greedy") => format!("samples-{split}-greedy-memory.json"),
                    _ => format!("samples-{split}-{name}-memory.json"),
                };
                let list = samples(seed, run, &file)?;
                // Every rule answers greedy's prompts, sample by sample, or nothing compares.
                let these: Vec<(usize, Vec<u32>)> =
                    list.iter().map(|s| (s.position, s.prompt.clone())).collect();
                match &prompts {
                    None => prompts = Some(these),
                    Some(first) if *first != these => {
                        return Err(invalid(format!(
                            "{run}-s{seed}/{file}: its prompts are not greedy's on {split}"
                        )));
                    }
                    Some(_) => {}
                }
                let replies: Vec<Vec<u32>> = list.iter().map(|s| s.reply.clone()).collect();
                let references: Vec<Vec<u32>> = list.iter().map(|s| s.reference.clone()).collect();
                if matched > 0 {
                    flags.push(
                        replies
                            .iter()
                            .map(|r| {
                                let cut = cut_at_eos(r, eos);
                                let complete = cut.last() == Some(&eos);
                                (complete, complete && train_set.contains(&census_hash(cut)))
                            })
                            .collect(),
                    );
                }
                let made = generation(&replies, &train_replies, eos);
                let reading = VerdictSeed::new(seed, &made, &generation(&references, &train_replies, eos));
                let mean_admitted = if name.starts_with("flock-cell-") {
                    let steps: Vec<u8> = list.iter().flat_map(|s| s.admitted.iter().copied()).collect();
                    if steps.is_empty() {
                        return Err(invalid(format!("{file}: the cell rule recorded no admitted counts")));
                    }
                    Some(steps.iter().map(|&n| f64::from(n)).sum::<f64>() / steps.len() as f64)
                } else {
                    None
                };
                let holds = match &greedy {
                    None => None,
                    Some(g) => Some(decode_holds(
                        g.complete,
                        made.complete,
                        replies.len(),
                        g.repeated_4gram_rate,
                        made.repeated_4gram_rate,
                    )),
                };
                let loops = count_loops.then(|| looping_replies(&replies, eos));
                let mean_length = count_loops.then(|| {
                    let total: usize = replies.iter().map(|r| cut_at_eos(r, eos).len()).sum();
                    total as f64 / replies.len().max(1) as f64
                });
                let fewer_loops = match (&greedy, loops) {
                    (Some(_), Some(n)) => Some(fewer_loops(n, greedy_loops, replies.len())),
                    _ => None,
                };
                if greedy.is_none() {
                    greedy = Some(made);
                    greedy_loops = loops.unwrap_or(0);
                }
                readings.push(CellReading {
                    split: split.to_owned(),
                    rule: rule.clone(),
                    seed,
                    close: reading.generalizes(),
                    reading,
                    holds,
                    mean_admitted,
                    loops,
                    fewer_loops,
                    mean_length,
                    matched_n: None,
                    matched_verbatim: None,
                });
            }
            // The prompts every rule completes, and each rule's verbatim replies among them.
            if matched > 0 {
                let (n, verbatim) = matched_counts(&flags, matched);
                for (i, v) in verbatim.into_iter().enumerate() {
                    let reading = &mut readings[first_reading + i];
                    reading.matched_n = Some(n);
                    reading.matched_verbatim = Some(v);
                }
            }
        }
    }
    Ok(readings)
}

/// claude-215's grade of the read's decode scopes: every reading, and the panel's outcomes.
#[derive(Clone, Debug, Serialize)]
pub struct ReadGrade {
    pub panel: String,
    pub readings: Vec<CellReading>,
    /// The registered decision, from [`ReadScope::All`] on the panel ([`read_outcome`]).
    pub outcome: &'static str,
    /// The same reading for [`ReadScope::Copy`], the copy heads alone. It decides nothing.
    pub copy_outcome: &'static str,
}

/// The fewest of 256 panel replies greedy must loop in, at every seed, for the read's outcome
/// to be decided (claude-215's FIT): below it, a reduction is too poorly powered to read, so the
/// outcome is `underpowered`. It is a power choice, not an impossibility: at 40, the band is 12.
pub const READ_LOOP_FLOOR: usize = 40;

/// Whether a scope's replies keep greedy's content: its `|v − v_ref|` is at most greedy's gap
/// plus the verdict's noise tolerance `tol` (a 3σ band on a share; claude-215's FIT).
fn content_kept(scope: &CellReading, greedy: &CellReading) -> bool {
    match (scope.gap(), greedy.gap(), scope.reading.tol) {
        (Some(s), Some(g), Some(t)) => s <= g + t,
        _ => false,
    }
}

/// The sheet's decision for one read scope's three panel seeds against greedy's three
/// (`greedy`, seed for seed), in this order:
/// - `underpowered` when greedy loops in fewer than [`READ_LOOP_FLOOR`] replies at any seed;
/// - `cures` when, at all three, it has fewer looping replies than greedy beyond the band,
///   holds decision 4, and is close;
/// - `reduces` when it has fewer loops and holds at all three, keeping greedy's content;
/// - `reduces, content lost` when it has fewer loops and holds at all three, but not that;
/// - else `no gain`.
///
/// An arm with fewer than three seeds reads `no gain`; it cannot occur from [`read_grade`],
/// whose [`rule_readings`] refuses any missing file.
fn read_outcome(scope: &[&CellReading], greedy: &[&CellReading]) -> &'static str {
    if greedy.len() != 3 || greedy.iter().any(|g| g.loops.is_none_or(|n| n < READ_LOOP_FLOOR)) {
        return "underpowered";
    }
    let fewer = scope.len() == 3
        && scope
            .iter()
            .all(|x| x.fewer_loops == Some(true) && x.holds == Some(true));
    if !fewer {
        return "no gain";
    }
    if scope.iter().all(|x| x.close) {
        "cures"
    } else if scope.iter().zip(greedy).all(|(s, g)| content_kept(s, g)) {
        "reduces"
    } else {
        "reduces, content lost"
    }
}

/// Grade the read's decode scopes (claude-215) from stored samples only, on the laptop: `runs`
/// holds `r-drop-s{1,2,3}`, each with DEV's and `panel`'s greedy samples as trained and under
/// [`ReadScope::Copy`] and [`ReadScope::All`]. The panel decides ([`read_outcome`]); DEV is
/// reported. Counts only; no model.
pub fn read_grade(data_root: &Path, runs: &Path, panel: &str) -> Result<ReadGrade> {
    if panel == "dev" {
        return Err(invalid("--read-grade's deciding panel cannot be DEV"));
    }
    let rules: Vec<String> = ["greedy", "greedy-read-copy", "greedy-read-all"]
        .map(String::from)
        .to_vec();
    let readings = rule_readings(data_root, runs, panel, &rules, true, 0)?;
    let of = |rule: &str| -> Vec<&CellReading> {
        readings.iter().filter(|r| r.split == panel && r.rule == rule).collect()
    };
    let greedy = of("greedy");
    let (outcome, copy_outcome) = (
        read_outcome(&of("greedy-read-all"), &greedy),
        read_outcome(&of("greedy-read-copy"), &greedy),
    );
    Ok(ReadGrade {
        panel: panel.to_owned(),
        readings,
        outcome,
        copy_outcome,
    })
}

/// claude-216's grade of the cut trained in: every reading (with its matched content), the
/// panel rows' teacher-forced cell, and the outcomes.
#[derive(Clone, Debug, Serialize)]
pub struct CutGrade {
    pub panel: String,
    pub readings: Vec<CellReading>,
    /// (r-cut)'s nearest hits against (r-drop)'s on every panel row, teacher-forced: the
    /// protocol's paired cell, ahead or behind only at all three seeds past the band.
    pub rows: GradeCell,
    /// The registered decision, from (r-cut) decoded with its cut, as served ([`cut_outcome`]).
    pub outcome: &'static str,
    /// The same rule for (r-drop) decoded with the cut only (claude-215's arm). It decides nothing.
    /// It has no teacher-forced rows of its own, so its rows' step is read as inside: it can never
    /// read "costs accuracy".
    pub decode_only_outcome: &'static str,
}

/// Whether `arm` keeps `base`'s content on the prompts both complete: its matched verbatim share
/// is below the base's by at most `max(0.10, 3·√(p(1−p)/m))`, `p` the base's share and `m` the
/// matched prompts (claude-216).
fn matched_kept(arm: &CellReading, base: &CellReading) -> bool {
    match (arm.matched_v(), base.matched_v(), base.matched_n) {
        (Some(a), Some(p), Some(m)) if m > 0 => {
            a >= p - 0.10f64.max(3.0 * (p * (1.0 - p) / m as f64).sqrt())
        }
        _ => false,
    }
}

/// The rules [`cut_grade`]'s matched content is taken over: the base and the served arm, the two
/// the decision compares (FIT round 2); the decode-only and uncut arms, which decide nothing, are
/// counted on their prompts and cannot thin them.
const MATCHED_BY: usize = 2;

/// [`cut_grade`]'s rules, in [`rule_readings`]'s order: the base and the served arm first, so the
/// first [`MATCHED_BY`] are the two the decision compares; then the decode-only and uncut arms.
const CUT_RULES: [&str; 4] = ["greedy", "r-cut/greedy-read-all", "greedy-read-all", "r-cut/greedy"];

/// The fewest matched prompts ([`matched_counts`]) the base must have at every seed for the
/// matched content to decide anything (claude-216's FIT): below it, `underpowered (content)`.
const MATCHED_FLOOR: usize = 64;

/// The sheet's decision (claude-216) for an arm's three panel seeds against (r-drop) greedy's
/// three (`base`, seed for seed), with the teacher-forced rows' cell `rows`, in this order:
/// - `underpowered` when greedy loops in fewer than [`READ_LOOP_FLOOR`] replies at any seed;
/// - `no gain` unless the arm has fewer loops beyond the band and holds decision 4 at all three;
/// - `costs accuracy` when the rows' cell is behind;
/// - `underpowered (content)` when the base has fewer than [`MATCHED_FLOOR`] matched prompts
///   at any seed: an instrument outcome, never a content negative;
/// - `cures` when it is also close and keeps the matched content ([`matched_kept`]) at all three;
/// - `improves` when it keeps the matched content at all three;
/// - else `reduces, content lost`.
fn cut_outcome(arm: &[&CellReading], base: &[&CellReading], rows: OverturnCell) -> &'static str {
    if base.len() != 3 || base.iter().any(|g| g.loops.is_none_or(|n| n < READ_LOOP_FLOOR)) {
        return "underpowered";
    }
    let fewer = arm.len() == 3
        && arm
            .iter()
            .all(|x| x.fewer_loops == Some(true) && x.holds == Some(true));
    if !fewer {
        return "no gain";
    }
    if rows == OverturnCell::Behind {
        return "costs accuracy";
    }
    if base.iter().any(|g| g.matched_n.is_none_or(|m| m < MATCHED_FLOOR)) {
        return "underpowered (content)";
    }
    let kept = arm.iter().zip(base).all(|(a, b)| matched_kept(a, b));
    if kept && arm.iter().all(|x| x.close) {
        "cures"
    } else if kept {
        "improves"
    } else {
        "reduces, content lost"
    }
}

/// Grade the cut trained in (claude-216) from stored outputs only, on the laptop: `runs` holds
/// `r-drop-s{1,2,3}` (the sealed (r-drop), scored on `panel` with `--reply-read all`) and
/// `r-cut-s{1,2,3}` (the run trained with the cut, scored the same way). Every reading is
/// counted on the prompts the base and the served arm complete ([`MATCHED_BY`]); the panel decides
/// ([`cut_outcome`]); DEV is reported.
pub fn cut_grade(data_root: &Path, runs: &Path, panel: &str) -> Result<CutGrade> {
    if panel == "dev" {
        return Err(invalid("--cut-grade's deciding panel cannot be DEV"));
    }
    let rules: Vec<String> = CUT_RULES.map(String::from).to_vec();
    let readings = rule_readings(data_root, runs, panel, &rules, true, MATCHED_BY)?;
    let mut counts = Vec::new();
    for seed in 1..=3u8 {
        let rows = |run: &str| -> Result<ScoredRows> {
            let path = runs.join(format!("{run}-s{seed}")).join(format!("rows-{panel}-memory.json"));
            let rows: ScoredRows = read_scored(&path)?;
            if !rows.memory || rows.split != panel {
                return Err(invalid(format!("{}: not {panel}'s rows with the memory on", path.display())));
            }
            Ok(rows)
        };
        let (cut, drop) = (rows("r-cut")?, rows("r-drop")?);
        let every: std::collections::BTreeSet<usize> = cut.positions.iter().copied().collect();
        counts.push((seed, paired(&cut, &drop, &every)?));
    }
    let rows = grade_cell("r-cut", "r-drop", "the panel's rows", &counts);
    let of = |rule: &str| -> Vec<&CellReading> {
        readings.iter().filter(|r| r.split == panel && r.rule == rule).collect()
    };
    let base = of("greedy");
    let (outcome, decode_only_outcome) = (
        cut_outcome(&of("r-cut/greedy-read-all"), &base, rows.cell),
        cut_outcome(&of("greedy-read-all"), &base, OverturnCell::Inside),
    );
    Ok(CutGrade {
        panel: panel.to_owned(),
        readings,
        rows,
        outcome,
        decode_only_outcome,
    })
}

/// [`cycle_grade`]'s rules for its first decision, in [`rule_readings`]'s order: the base and
/// the cycle step first, so the first [`MATCHED_BY`] are the two it compares; then the
/// decode-only cut, the cycle step under it, and the echo step, reported on the same prompts.
const CYCLE_RULES: [&str; 5] =
    ["greedy", "cycle-step", "greedy-read-all", "cycle-step-read-all", "echo-step"];

/// [`cycle_grade`]'s rules for its second decision: the base and the decode-only cut, matched
/// on the prompts those two complete.
const CONFIRM_RULES: [&str; 2] = ["greedy", "greedy-read-all"];

/// How often one arm's cycle step left the nearest token, at one split and seed (claude-217).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct StepCount {
    pub split: String,
    pub rule: String,
    pub seed: u8,
    /// Decoded steps, a final step that picked BOS included.
    pub steps: usize,
    /// Steps that took a farther candidate ([`STEP_OFF`]).
    pub stepped_off: usize,
    /// Steps where every candidate closed a repeated 4-gram, so the nearest was taken
    /// ([`STEP_EXHAUSTED`]).
    pub exhausted: usize,
    /// Replies with at least one step off.
    pub replies_stepped: usize,
}

/// One arm's [`StepCount`] from its samples' marks.
fn step_count(split: &str, rule: &str, seed: u8, samples: &[IdSample]) -> StepCount {
    let count = |mark: u8| samples.iter().flat_map(|s| &s.stepped).filter(|&&m| m == mark).count();
    StepCount {
        split: split.to_owned(),
        rule: rule.to_owned(),
        seed,
        steps: samples.iter().map(|s| s.stepped.len()).sum(),
        stepped_off: count(STEP_OFF),
        exhausted: count(STEP_EXHAUSTED),
        replies_stepped: samples.iter().filter(|s| s.stepped.contains(&STEP_OFF)).count(),
    }
}

/// claude-217's grade of the cycle step: both decisions' readings, the step counts, and the
/// outcomes.
#[derive(Clone, Debug, Serialize)]
pub struct CycleGrade {
    pub panel: String,
    /// The first decision's readings ([`CYCLE_RULES`]), matched on the base's and the cycle
    /// step's completions.
    pub readings: Vec<CellReading>,
    /// The second decision's readings ([`CONFIRM_RULES`]), matched on the base's and the
    /// decode-only cut's completions.
    pub confirm_readings: Vec<CellReading>,
    /// Every cycle-step arm's step counts, on DEV and the panel, at seeds 1–3.
    pub steps: Vec<StepCount>,
    /// The first registered decision: the cycle step against greedy ([`cycle_outcome`]: its
    /// content is read on the base's looping prompts as well as on the matched prompts).
    pub outcome: &'static str,
    /// The second registered decision: the decode-only cut against greedy, claude-216's
    /// reported "improves" on a fresh panel, by the same rule.
    pub confirm_outcome: &'static str,
    /// The cycle step under the cut, by [`cycle_outcome`] on the first decision's matched
    /// prompts. It decides what ships only if both decisions pass.
    pub cut_cycle_outcome: &'static str,
    /// The echo step ([`DecodeRule::ECHO_STEP`]), by [`cycle_outcome`] on the first decision's
    /// matched prompts. It is reported and decides nothing.
    pub echo_outcome: &'static str,
    /// Every arm's content on the base's looping prompts ([`LoopContent`]), on DEV and the
    /// panel, at seeds 1–3.
    pub loop_content: Vec<LoopContent>,
    /// The served decode the sheet's table gives from the three outcomes ([`ships`]).
    pub ships: &'static str,
}

/// The sheet's "what ships" table (claude-217): the cycle step when only it passes ("cures" or
/// "improves"); the decode-only cut when only it does; both when both do and "both" passes on
/// the first decision's prompts, else the cycle step alone; greedy when neither does.
fn ships(cycle: &str, cut: &str, both: &str) -> &'static str {
    let pass = |o: &str| o == "cures" || o == "improves";
    match (pass(cycle), pass(cut)) {
        (true, false) => "cycle-step",
        (false, true) => "decode-only cut",
        (true, true) if pass(both) => "both",
        (true, true) => "cycle-step",
        (false, false) => "greedy",
    }
}

/// The fewest looping prompts the base must have at every seed for the content there to decide
/// anything (claude-217's FIT): the loop floor's count.
const LOOP_FLOOR: usize = READ_LOOP_FLOOR;

/// An arm's content on the prompts where the base, greedy, loops (claude-217's FIT, rounds 1 and
/// 3). A loop fix changes what is written exactly there, and greedy rarely completes there, so
/// the matched content (on the prompts both complete) cannot see it; this reading can. For the
/// cycle step these are the prompts it steps off on, since it writes greedy's reply until a cycle
/// would close. It counts the complete replies that are verbatim TRAIN replies, the arm's against
/// the references' on the same prompts, and, reported only, the replies equal to their reference.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LoopContent {
    pub split: String,
    pub rule: String,
    pub seed: u8,
    /// The prompts where the base loops ([`loops`]).
    pub prompts: usize,
    /// Of them, the arm's replies that are complete and verbatim in TRAIN.
    pub arm_verbatim: usize,
    /// Of them, the references that are complete and verbatim in TRAIN.
    pub reference_verbatim: usize,
    /// Of them, the arm's replies equal to their reference, each cut after its first end of turn:
    /// a fallback reading, reported. It decides nothing.
    pub arm_exact: usize,
}

impl LoopContent {
    /// Whether the arm keeps the references' content on the base's looping prompts: its share is
    /// below theirs by at most `max(0.10, 3·√(p(1−p)/m))`, `p` the references' share and `m` the
    /// looping prompts; `None` with none.
    pub fn kept(&self) -> Option<bool> {
        let m = self.prompts as f64;
        (self.prompts > 0).then(|| {
            let (a, p) = (self.arm_verbatim as f64 / m, self.reference_verbatim as f64 / m);
            a >= p - 0.10f64.max(3.0 * (p * (1.0 - p) / m).sqrt())
        })
    }

    /// Whether the reading can tell salad from content (claude-217's FIT, rounds 2 and 4): the
    /// lowest share [`LoopContent::kept`] accepts is above [`LOOP_POWER_SHARE`] and above half the
    /// references' share, so an arm at or below 0.10 never reads kept, and nor does one that
    /// keeps less than half of what the references hold. The looping prompts are not a random
    /// sample, so their references may be rarely verbatim; then the reading is underpowered.
    pub fn powered(&self) -> bool {
        let m = self.prompts as f64;
        if self.prompts == 0 {
            return false;
        }
        let p = self.reference_verbatim as f64 / m;
        p - 0.10f64.max(3.0 * (p * (1.0 - p) / m).sqrt()) > LOOP_POWER_SHARE.max(p / 2.0)
    }
}

/// The verbatim share a step into salad is taken to stay at or below: a loop-prompt reading
/// decides only where an arm this low would read lost ([`LoopContent::powered`]).
pub const LOOP_POWER_SHARE: f64 = 0.10;

/// One arm's [`LoopContent`] from its samples and the base's on the same prompts, in the same
/// order: the prompts are the base's looping ones; `verbatim` asks whether a reply is complete
/// and verbatim in TRAIN. Refused when the two do not answer the same prompts.
fn loop_content(
    split: &str,
    rule: &str,
    seed: u8,
    arm: &[IdSample],
    base: &[IdSample],
    eos: u32,
    verbatim: impl Fn(&[u32]) -> bool,
) -> Result<LoopContent> {
    if arm.len() != base.len()
        || arm.iter().zip(base).any(|(a, b)| a.position != b.position || a.reference != b.reference)
    {
        return Err(invalid(format!("{split} s{seed} {rule}: its prompts are not the base's")));
    }
    let on: Vec<&IdSample> =
        arm.iter().zip(base).filter(|(_, b)| loops(&b.reply, eos)).map(|(a, _)| a).collect();
    Ok(LoopContent {
        split: split.to_owned(),
        rule: rule.to_owned(),
        seed,
        prompts: on.len(),
        arm_verbatim: on.iter().filter(|s| verbatim(&s.reply)).count(),
        reference_verbatim: on.iter().filter(|s| verbatim(&s.reference)).count(),
        arm_exact: on
            .iter()
            .filter(|s| cut_at_eos(&s.reply, eos) == cut_at_eos(&s.reference, eos))
            .count(),
    })
}

/// The cycle step's instrument check (claude-217's FIT, round 3): each of `arm`'s replies, cut
/// after its first end of turn, must be its greedy `base`'s token for token until its first step
/// off ([`STEP_OFF`]). A reply that left greedy with no mark is a fault, never a reading.
fn follows_greedy(arm: &[IdSample], base: &[IdSample], eos: u32) -> bool {
    arm.len() == base.len()
        && arm.iter().zip(base).all(|(a, b)| {
            let off = a.stepped.iter().position(|&m| m == STEP_OFF).unwrap_or(a.reply.len());
            let k = off.min(cut_at_eos(&a.reply, eos).len());
            b.reply.len() >= k && a.reply[..k] == b.reply[..k]
        })
}

/// The sheet's decision for a decode arm against greedy (claude-217): [`cut_outcome`]'s order with
/// no rows' step, and its content condition taken on the base's looping prompts as well as on the
/// matched prompts, seed for seed:
/// - `underpowered` when greedy loops in fewer than [`READ_LOOP_FLOOR`] replies at any seed;
/// - `no gain` unless the arm has fewer loops beyond the band and holds decision 4 at all three;
/// - `underpowered (content)` when the base has fewer than [`MATCHED_FLOOR`] matched prompts or
///   [`LOOP_FLOOR`] looping ones, or the loop-prompt reading cannot tell salad from content
///   ([`LoopContent::powered`]), at any seed;
/// - `cures` when, at all three, it is close, keeps the matched content ([`matched_kept`]) and
///   keeps the references' content on the looping prompts ([`LoopContent::kept`]);
/// - `improves` when it keeps both at all three;
/// - else `reduces, content lost`: short of the references where greedy looped, or of greedy's
///   content where it did not.
fn cycle_outcome(
    arm: &[&CellReading],
    base: &[&CellReading],
    changed: &[&LoopContent],
) -> &'static str {
    if base.len() != 3 || base.iter().any(|g| g.loops.is_none_or(|n| n < READ_LOOP_FLOOR)) {
        return "underpowered";
    }
    let fewer = arm.len() == 3
        && arm
            .iter()
            .all(|x| x.fewer_loops == Some(true) && x.holds == Some(true));
    if !fewer {
        return "no gain";
    }
    if changed.len() != 3
        || base.iter().any(|g| g.matched_n.is_none_or(|m| m < MATCHED_FLOOR))
        || changed.iter().any(|c| c.prompts < LOOP_FLOOR || !c.powered())
    {
        return "underpowered (content)";
    }
    let kept = arm
        .iter()
        .zip(base)
        .zip(changed)
        .all(|((a, b), c)| matched_kept(a, b) && c.kept() == Some(true));
    if kept && arm.iter().all(|x| x.close) {
        "cures"
    } else if kept {
        "improves"
    } else {
        "reduces, content lost"
    }
}

/// Grade the cycle step (claude-217) from stored outputs only, on the laptop: `runs` holds
/// `r-drop-s{1,2,3}`, the sealed (r-drop) scored on `panel` with `--reply-read all
/// --cycle-step 1`. The panel decides both decisions by [`cycle_outcome`]; DEV is reported. It
/// refuses to grade a cycle-step reply that left greedy's before its first step off
/// ([`follows_greedy`]).
pub fn cycle_grade(data_root: &Path, runs: &Path, panel: &str) -> Result<CycleGrade> {
    if panel == "dev" {
        return Err(invalid("--cycle-grade's deciding panel cannot be DEV"));
    }
    let named = |rules: &[&str]| -> Vec<String> { rules.iter().map(|&r| r.to_owned()).collect() };
    let readings = rule_readings(data_root, runs, panel, &named(&CYCLE_RULES), true, MATCHED_BY)?;
    let confirm_readings =
        rule_readings(data_root, runs, panel, &named(&CONFIRM_RULES), true, MATCHED_BY)?;
    // Complete and verbatim in TRAIN, as `rule_readings` counts its matched content.
    let (train_tokens, train_mask) = load_train_split(data_root)?;
    let train: std::collections::HashSet<u128> =
        split_turns(&train_tokens, &train_mask, hs::SHEET_BOS_ID).replies.iter().map(|r| census_hash(r)).collect();
    drop((train_tokens, train_mask));
    let eos = pinned_tokenizer(data_root)?
        .token_id("<|eos|>")
        .ok_or_else(|| invalid("the pinned tokenizer has no <|eos|>"))?;
    let verbatim = |reply: &[u32]| {
        let cut = cut_at_eos(reply, eos);
        cut.last() == Some(&eos) && train.contains(&census_hash(cut))
    };
    let (mut steps, mut loop_content_all) = (Vec::new(), Vec::new());
    for split in ["dev", panel] {
        for seed in 1..=3u8 {
            let samples = |name: &str| -> Result<Vec<IdSample>> {
                let file = match (split, name) {
                    ("dev", "greedy") => "samples-memory.json".to_owned(),
                    (_, name) => format!("samples-{split}-{name}-memory.json"),
                };
                read_scored(&runs.join(format!("r-drop-s{seed}")).join(file))
            };
            let base = samples("greedy")?;
            for rule in ["cycle-step", "greedy-read-all", "cycle-step-read-all", "echo-step"] {
                let arm = samples(rule)?;
                if rule != "greedy-read-all" {
                    if arm.iter().any(|s| s.stepped.is_empty()) {
                        return Err(invalid(format!("{split} s{seed} {rule}: a reply records no cycle-step marks")));
                    }
                    // Its own greedy path: greedy for the cycle step, the cut for it under the cut.
                    let path = if rule == "cycle-step-read-all" { samples("greedy-read-all")? } else { base.clone() };
                    if !follows_greedy(&arm, &path, eos) {
                        return Err(invalid(format!(
                            "{split} s{seed} {rule}: a reply left greedy's before its first step off (an instrument fault)"
                        )));
                    }
                    steps.push(step_count(split, rule, seed, &arm));
                }
                loop_content_all.push(loop_content(split, rule, seed, &arm, &base, eos, verbatim)?);
            }
        }
    }
    let of = |readings: &[CellReading], rule: &str| -> Vec<CellReading> {
        readings.iter().filter(|r| r.split == panel && r.rule == rule).cloned().collect()
    };
    // Each decision reads its arm on its own matched prompts and on the base's looping prompts.
    let decide = |readings: &[CellReading], rule: &str| -> Result<&'static str> {
        let (arm, base) = (of(readings, rule), of(readings, "greedy"));
        let on: Vec<&LoopContent> =
            loop_content_all.iter().filter(|c| c.split == panel && c.rule == rule).collect();
        // `cycle_outcome` pairs the three by position, so they must pair seed with seed.
        if arm.iter().zip(&base).zip(&on).any(|((a, b), c)| a.seed != b.seed || a.seed != c.seed) {
            return Err(invalid(format!("{panel} {rule}: its readings do not pair seed with seed")));
        }
        Ok(cycle_outcome(&arm.iter().collect::<Vec<_>>(), &base.iter().collect::<Vec<_>>(), &on))
    };
    let outcome = decide(&readings, "cycle-step")?;
    let confirm_outcome = decide(&confirm_readings, "greedy-read-all")?;
    let cut_cycle_outcome = decide(&readings, "cycle-step-read-all")?;
    let echo_outcome = decide(&readings, "echo-step")?;
    Ok(CycleGrade {
        panel: panel.to_owned(),
        outcome,
        confirm_outcome,
        cut_cycle_outcome,
        echo_outcome,
        ships: ships(outcome, confirm_outcome, cut_cycle_outcome),
        readings,
        confirm_readings,
        steps,
        loop_content: loop_content_all,
    })
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub register_weight_mean_abs: Option<f64>,
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
    // The prompt register (claude-155), when on: its own optimizer, applied inside the trunk.
    let mut register = start_register(config, device)?;
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
            register_weight_mean_abs: match crate::flock_read_register::prompt_register() {
                Some(reg) => Some(scalar(reg.weight_mean_abs()?)?),
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
                    false,
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
        if let Some((reg, opt)) = register.as_mut() {
            opt.step_vars(&reg.variables(), &grads, lr)?;
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
    if let Some(reg) = crate::flock_read_register::prompt_register() {
        reg.save(&directory)?;
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

/// TRAIN's windows of `context` tokens with at least 4 scored targets, each context-long; at
/// least `batch` of them; and TRAIN's response mask. TRAIN is the pinned split, or `split` when
/// a run names one.
fn load_train(
    data_root: &Path,
    batch: usize,
    split: Option<&TrainSplit>,
    context: usize,
) -> Result<(Vec<CausalWindow>, Vec<u8>)> {
    let (train_tokens, train_mask) = match split {
        None => load_train_split(data_root)?,
        Some(split) => load_named_split(split)?,
    };
    let (train_windows, _) =
        hs::build_document_windows(&train_tokens, &train_mask, context, hs::SHEET_BOS_ID)?;
    let usable: Vec<CausalWindow> = train_windows
        .into_iter()
        .filter(|w| w.scored_offsets.len() >= 4)
        .collect();
    if usable.len() < batch || usable.iter().any(|w| w.input_ids.len() != context) {
        return Err(invalid("TRAIN windows are too few or not context-long"));
    }
    Ok((usable, train_mask))
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
    let (usable, _) =
        load_train(data_root, config.batch, config.train_split.as_ref(), config.context)?;
    arms::set_fbs_arm(Some(FbsArm::OneAtomActive));
    crate::geometric_stack::set_retention_rates(config.retain_rates);
    crate::geometric_stack::set_folded_scores(config.fold_scores);
    crate::geometric_stack::set_reflected_floor(config.reflect_floor);
    crate::geometric_stack::set_input_table_fixed(config.fix_input_table);
    let _guard = ArmGuard;
    let model = build_long_model(FbsArm::OneAtomActive, config, device)?;
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
    let _register = start_register(config, device)?;
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
            false,
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
            let hidden = crate::flock_read::points(&model, &ids, config.batch, config.context)?;
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
/// Training steps run before the profile's window opens (fbs-cost-profile, claude-157): past
/// step 0's diagnostics and the first steps' allocations.
pub const PROFILE_WARMUP: usize = 8;

/// Where `step` sits in a `profile`-step window: `Some(true)` for a profiled step, `Some(false)`
/// for the unprofiled step after it, whose time it is compared with, and `None` outside the
/// window or when `profile` is 0. The window is the `2 * profile` steps after
/// [`PROFILE_WARMUP`], profiled and unprofiled in turn.
pub fn profile_window(step: usize, profile: usize) -> Option<bool> {
    if profile == 0
        || step < PROFILE_WARMUP
        || step >= PROFILE_WARMUP.saturating_add(profile.saturating_mul(2))
    {
        return None;
    }
    Some((step - PROFILE_WARMUP).is_multiple_of(2))
}

/// One profiled training step, one line of `profile.jsonl`.
#[derive(Clone, Debug, Serialize)]
pub struct ProfileRow {
    pub step: usize,
    pub loss: f64,
    /// The step's host time through its closing wait for the GPU, as every step is timed.
    pub step_ms: f64,
    /// Each stage from the mark before it, in step order; the last, `end`, is the closing mark,
    /// with nothing queued before it since `optimizer.decay`, so about zero GPU time.
    pub stages: Vec<StageTime>,
    /// The stages' GPU times summed: the GPU's span from the step's first mark to its last.
    pub gpu_span_ms: f64,
    /// The unprofiled step after this one, timed as every training step is.
    pub next_unprofiled_ms: Option<f64>,
}

/// One stage's medians over the profiled steps.
#[derive(Clone, Debug, Serialize)]
pub struct StageSummary {
    pub name: String,
    pub gpu_ms: f64,
    pub host_ms: f64,
}

/// `profile.json`: the profiled steps' stages against the unprofiled steps' time.
#[derive(Clone, Debug, Serialize)]
pub struct ProfileSummary {
    pub profiled_steps: usize,
    /// Each stage's median, in step order.
    pub stages: Vec<StageSummary>,
    /// The stage medians' GPU times summed by group: `forward`, `objective`, `backward`,
    /// `optimizer` and `end`.
    pub groups: Vec<(String, f64)>,
    /// The stage medians' GPU times summed.
    pub stage_sum_ms: f64,
    /// The median of the unprofiled steps' times.
    pub median_unprofiled_ms: f64,
    /// `stage_sum_ms / median_unprofiled_ms`.
    pub ratio: f64,
    /// The item's bar: the stages sum to the measured step time within 10%.
    pub within_10pct: bool,
    /// The median of the profiled steps' host times, timed as the unprofiled ones are.
    pub median_profiled_ms: f64,
    /// `median_profiled_ms / median_unprofiled_ms - 1`: what the clock itself adds to a step. The
    /// stage sum cannot show it, since both spans start and end with the GPU idle.
    pub clock_overhead: f64,
    /// The three stages with the largest median GPU time, largest first.
    pub top3: Vec<String>,
    /// The same, among the forward and objective stages only: the backward holds every stage's
    /// gradient in one time.
    pub top3_forward: Vec<String>,
}

/// The median of `values` (the mean of the middle two for an even count); NaN when empty.
fn median(values: &mut [f64]) -> f64 {
    if values.is_empty() {
        return f64::NAN;
    }
    values.sort_by(f64::total_cmp);
    let mid = values.len() / 2;
    if values.len() % 2 == 1 {
        values[mid]
    } else {
        (values[mid - 1] + values[mid]) / 2.0
    }
}

/// The group a stage's time counts in.
fn stage_group(name: &str) -> &'static str {
    if name == "points.select" || name.starts_with("objective.") {
        "objective"
    } else if name == "backward" {
        "backward"
    } else if name.starts_with("optimizer.") {
        "optimizer"
    } else if name == "end" {
        "end"
    } else {
        "forward"
    }
}

/// The profile's summary: each stage's medians over `rows` in the first row's stage order, the
/// group sums, and the sum against the unprofiled steps' median.
pub fn summarize_profile(rows: &[ProfileRow]) -> ProfileSummary {
    let names: Vec<String> = rows
        .first()
        .map(|r| r.stages.iter().map(|s| s.name.clone()).collect())
        .unwrap_or_default();
    let stages: Vec<StageSummary> = names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let mut gpu: Vec<f64> = rows
                .iter()
                .filter_map(|r| r.stages.get(i).and_then(|s| s.gpu_ms))
                .collect();
            let mut host: Vec<f64> = rows
                .iter()
                .filter_map(|r| r.stages.get(i).map(|s| s.host_ms))
                .collect();
            StageSummary {
                name: name.clone(),
                gpu_ms: median(&mut gpu),
                host_ms: median(&mut host),
            }
        })
        .collect();
    let mut groups: Vec<(String, f64)> = ["forward", "objective", "backward", "optimizer", "end"]
        .iter()
        .map(|g| (g.to_string(), 0.0))
        .collect();
    for s in &stages {
        if let Some(g) = groups.iter_mut().find(|(g, _)| g == stage_group(&s.name)) {
            g.1 += s.gpu_ms;
        }
    }
    let stage_sum_ms: f64 = stages.iter().map(|s| s.gpu_ms).sum();
    let mut unprofiled: Vec<f64> = rows.iter().filter_map(|r| r.next_unprofiled_ms).collect();
    let median_unprofiled_ms = median(&mut unprofiled);
    let ratio = stage_sum_ms / median_unprofiled_ms;
    let mut profiled: Vec<f64> = rows.iter().map(|r| r.step_ms).collect();
    let median_profiled_ms = median(&mut profiled);
    let mut ranked: Vec<&StageSummary> = stages.iter().collect();
    ranked.sort_by(|a, b| b.gpu_ms.total_cmp(&a.gpu_ms));
    let names_of = |keep: &dyn Fn(&str) -> bool| -> Vec<String> {
        ranked
            .iter()
            .filter(|s| keep(&s.name))
            .take(3)
            .map(|s| s.name.clone())
            .collect()
    };
    let top3 = names_of(&|_| true);
    let top3_forward = names_of(&|n| matches!(stage_group(n), "forward" | "objective"));
    ProfileSummary {
        profiled_steps: rows.len(),
        top3,
        top3_forward,
        stages,
        groups,
        stage_sum_ms,
        median_unprofiled_ms,
        ratio,
        within_10pct: (ratio - 1.0).abs() <= 0.10,
        median_profiled_ms,
        clock_overhead: median_profiled_ms / median_unprofiled_ms - 1.0,
    }
}

pub fn run_long(data_root: &Path, out: &Path, config: &LongConfig, device: &Device) -> Result<()> {
    if config.steps == 0 || config.batch == 0 || config.eval_every == 0 {
        return Err(invalid("steps, batch and eval-every must be positive"));
    }
    training_context(config.context)?;
    // A clock left on by a run that failed mid-step on this thread starts no stage of this one.
    stage_clock::reset();
    if config.profile > 0 {
        let window_end = config
            .profile
            .checked_mul(2)
            .and_then(|w| w.checked_add(PROFILE_WARMUP))
            .ok_or_else(|| invalid(format!("--profile {} is too large", config.profile)))?;
        if config.steps < window_end {
            return Err(invalid(format!(
                "--profile {} needs at least {window_end} steps: {PROFILE_WARMUP} of warm-up, \
                 then each profiled step and the unprofiled step after it",
                config.profile
            )));
        }
        if !device.is_cuda() {
            return Err(invalid(
                "--profile times the GPU's stages and needs a CUDA device",
            ));
        }
    }
    // Training as served (claude-258) rounds what the export carries: the trunk, the Hamming
    // read and the turn memory. A read it does not carry would train at values no artifact holds,
    // and a run with folded or reflected rate scores has no export at all (`run_rate_reading`).
    if config.train_as_served
        && (config.flock_read > 0
            || config.content_read > 0
            || config.opening_read > 0
            || config.prompt_register > 0
            || config.type_spin
            || config.reply_read_train.is_some()
            || config.fold_scores
            || config.reflect_floor)
    {
        return Err(invalid(
            "--train-as-served rounds the trunk, the Hamming read and the turn memory as the export \
             does; the Flock, content and opening reads, the prompt register and the type spin \
             have no exported form, the reply-read training cut is not served, and the export \
             refuses folded or reflected rate scores",
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

    let tokenizer = pinned_tokenizer(data_root)?;

    let (usable, train_mask) =
        load_train(data_root, config.batch, config.train_split.as_ref(), config.context)?;
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

    arms::set_fbs_arm(Some(config.fbs_arm));
    crate::geometric_stack::set_retention_rates(config.retain_rates);
    crate::geometric_stack::set_folded_scores(config.fold_scores);
    crate::geometric_stack::set_reflected_floor(config.reflect_floor);
    crate::geometric_stack::set_input_table_fixed(config.fix_input_table);
    let _guard = ArmGuard;
    // The reply's read held to the prompt in training (claude-216); it needs the Hamming read.
    if config.reply_read_train.is_some() && config.hamming_read == 0 {
        return Err(invalid("--reply-read-train needs --hamming-read"));
    }
    set_trained_cut(config.reply_read_train);
    if config.reply_read_train.is_some() {
        let (rows, cut, none) = cut_census(&usable, &train_mask, config.context)?;
        println!(
            "[cut] TRAIN rows {rows}: {cut} reply rows read only before their reply, {none} of \
             them nothing (their reply began before the window)"
        );
    }
    let model = build_long_model(config.fbs_arm, config, device)?;
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
    // The prompt register (claude-155), when on: its own optimizer, applied inside the trunk.
    let mut register = start_register(config, device)?;
    // The turn memory (claude-161), when on: its table from TRAIN, its own optimizer, and each
    // usable window's opening offsets, which name the rows it moves.
    let (mut memory, train_openings, train_replies) =
        start_memory(data_root, config, &usable, device)?;
    // How often the read's own key finds the table at an opening row, on TRAIN's usable windows
    // and on DEV: the table is keyed from the probe's prompts and read through the turn anchor,
    // so this is the measured agreement, not one assumed.
    if let Some((mem, _)) = memory.as_ref() {
        let train_rows: Vec<(usize, Vec<usize>)> = train_openings
            .iter()
            .enumerate()
            .map(|(i, offsets)| (i, offsets.clone()))
            .collect();
        let (rows, hits) = memory_hits(mem, &usable, &train_rows)?;
        println!("[memory] TRAIN opening rows {rows}, hits {hits}");
        let dev_rows: Vec<(usize, Vec<usize>)> = dev_windows
            .iter()
            .enumerate()
            .map(|(i, w)| {
                let offsets = w
                    .scored_offsets
                    .iter()
                    .zip(&w.scored_corpus_positions)
                    .filter(|(_, &position)| opens_reply(&dev_mask, position))
                    .map(|(&offset, _)| offset)
                    .collect();
                (i, offsets)
            })
            .collect();
        let (rows, hits) = memory_hits(mem, &dev_windows, &dev_rows)?;
        println!("[memory] DEV opening rows {rows}, hits {hits}");
        // The same for the reply table (claude-193), keyed by the probe's own key.
        if mem.reply_len().is_some() {
            let count = |rows: &[Vec<(usize, Vec<u32>)>]| {
                let named: Vec<&Vec<u32>> = rows.iter().flatten().map(|(_, key)| key).collect();
                let hits = named
                    .iter()
                    .filter(|key| mem.reply_stored(key).is_some())
                    .count();
                (named.len(), hits)
            };
            // "Named": the rows whose key the reply table holds, not the rows it names right.
            let (rows, held) = count(&train_replies);
            println!("[memory] TRAIN reply rows past their openings {rows}, named {held}");
            let (rows, held) = count(&reply_offsets(&dev_windows, &dev_mask));
            println!("[memory] DEV reply rows past their openings {rows}, named {held}");
        }
    }
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
        // Trained as served (claude-258), the checkpoint is scored and saved at the exported
        // values, through the reads and memory set on this thread, which share their variables.
        let served = round_if_served(
            config.train_as_served,
            model,
            crate::flock_read_hamming::hamming_reads().as_ref(),
            crate::flock_read_memory::turn_memory().as_ref(),
        )?;
        let (dev, dev_loss, nearest, written) = if config.train_as_served {
            hs::with_served_surrogate_training(config.seed, step, || {
                let (dev, _) = hs::evaluate_dev_panel_d11(model, &dev_windows)?;
                let dev_loss =
                    crate::spiralcore_fbs_sheet::panel_loss_d11(model, &dev_windows)?;
                let (nearest, written) =
                    nearest_reading(model, &dev_windows, &dev_mask, config.seed, step, true)?;
                Ok((dev, dev_loss, nearest, written))
            })?
        } else {
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
                nearest_reading(model, &dev_windows, &dev_mask, config.seed, step, false)?;
            (dev, dev_loss, nearest, written)
        };
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
            train_context_tokens_per_second: (step * config.batch * config.context) as f64
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
            let reply = greedy_reply_with_mode(
                model,
                config.objective,
                prompt,
                config.reply_tokens,
                config.seed,
                config.train_as_served,
            )?;
            let reference = reference_reply(window, config.reply_tokens);
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
        if let Some(reg) = crate::flock_read_register::prompt_register() {
            reg.save(&directory)?;
        }
        if let Some(memory) = crate::flock_read_memory::turn_memory() {
            memory.save(&directory)?;
        }
        if let Some(served) = served {
            served.restore()?;
        }
        Ok(())
    };

    checkpoint(&model, 0, None, 0.0)?;
    let mut order = epoch_order(usable.len(), config.seed, 0);
    let mut cursor = 0usize;
    let mut epoch = 0u64;
    // The profile (fbs-cost-profile, claude-157): each profiled step waits for the unprofiled
    // step after it, whose time it is compared with.
    let mut profiled_row: Option<ProfileRow> = None;
    let mut profile_rows: Vec<ProfileRow> = Vec::new();
    for step in 0..config.steps {
        if cursor + config.batch > order.len() {
            epoch += 1;
            order = epoch_order(usable.len(), config.seed, epoch);
            cursor = 0;
        }
        let picked = &order[cursor..cursor + config.batch];
        let windows: Vec<&CausalWindow> = picked.iter().map(|&i| &usable[i]).collect();
        // A window the memory is dropped from (protocol I-B) names no memory row below.
        let kept = |b: usize| {
            !memory_dropped(config.seed, step, b, config.batch, config.turn_memory_dropout)
        };
        // The batch's opening rows, which the turn memory moves; none when it is off.
        crate::flock_read_memory::set_memory_rows(memory.as_ref().map(|_| {
            picked
                .iter()
                .enumerate()
                .filter(|&(b, _)| kept(b))
                .flat_map(|(b, &i)| {
                    train_openings[i]
                        .iter()
                        .map(move |&offset| b * config.context + offset)
                })
                .collect()
        }));
        // Its reply rows past their openings, with their keys (claude-193); none when it has no
        // reply table.
        crate::flock_read_memory::set_reply_rows((!train_replies.is_empty()).then(|| {
            picked
                .iter()
                .enumerate()
                .filter(|&(b, _)| kept(b))
                .flat_map(|(b, &i)| {
                    train_replies[i]
                        .iter()
                        .map(move |(offset, key)| (b * config.context + offset, key.clone()))
                })
                .collect()
        }));
        cursor += config.batch;
        let (ids, scored, targets) = assemble(&windows);
        // The memory's rows are `b · config.context + offset`, so every window must be whole.
        if ids.len() != config.batch * config.context {
            return Err(invalid("a training batch's windows are not context-long"));
        }
        // Under the cut (claude-216), every reply row of the batch reads only before its reply.
        if trained_cut_on() {
            set_reply_mask(Some(window_input_mask(&windows, &train_mask)?));
        }
        let profiled = profile_window(step, config.profile);
        let started = Instant::now();
        if profiled == Some(true) {
            stage_clock::start(device);
        }
        // Trained as served (claude-258): the forward and its gradient at the exported values.
        let served = round_if_served(
            config.train_as_served,
            &model,
            hamming.as_ref().map(|(reads, _)| reads),
            memory.as_ref().map(|(mem, _)| mem),
        )?;
        if served.is_some() {
            stage_clock::mark("served.round");
        }
        // The gate hook carries the real step: at step 0 its backward runs diagnostics with host
        // syncs, and a checkpoint's replies leave it at 0. Training as served adds the complete
        // quantization bridge and Q12 query used by the exported graph.
        let (value, grads) = with_training_forward(
            config.train_as_served,
            config.seed,
            step,
            || {
                let loss = batch_loss(
                    &model,
                    config.objective,
                    &ids,
                    &scored,
                    &targets,
                    config.batch,
                    config.train_as_served,
                )?;
                // The copy to the host waits for the GPU to finish the forward. Only the nearest
                // and cell objectives mark their own stages; the sheet's and Voronoi objectives
                // count in this one.
                let value = f64::from(loss.to_scalar::<f32>()?);
                stage_clock::mark("objective.readback");
                if !value.is_finite() {
                    return Err(invalid(format!("non-finite training loss at step {step}")));
                }
                let grads = loss.backward()?;
                stage_clock::mark("backward");
                Ok((value, grads))
            },
        )?;
        // The gradient, taken at the exported values, moves the float values they came from.
        if let Some(served) = served {
            served.restore()?;
            stage_clock::mark("served.restore");
        }
        // Linear from `lr` at the first step to `lr_end` at the last.
        let progress = if config.steps == 1 {
            1.0
        } else {
            step as f64 / (config.steps - 1) as f64
        };
        let lr = config.lr + (config.lr_end - config.lr) * progress;
        opt_main.update_filtered(&model, &grads, lr, |name| {
            main_optimizer_updates(name, config.fbs_arm)
        })?;
        stage_clock::mark("optimizer.main");
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
        if let Some((reg, opt)) = register.as_mut() {
            opt.step_vars(&reg.variables(), &grads, lr)?;
        }
        if let Some((mem, opt)) = memory.as_mut() {
            opt.step_vars(&mem.variables(), &grads, lr)?;
        }
        crate::flock_read_memory::set_memory_rows(None);
        crate::flock_read_memory::set_reply_rows(None);
        stage_clock::mark("optimizer.reads");
        opt_decay.update_filtered(&model, &grads, lr * config.decay_mult, |name| {
            name.ends_with("rec.decay")
        })?;
        stage_clock::mark("optimizer.decay");
        // Reading the events waits for the GPU, as the synchronization below would.
        let stages = stage_clock::finish()?;
        device.synchronize()?;
        let step_seconds = started.elapsed().as_secs_f64();
        train_seconds += step_seconds;
        match (profiled, stages) {
            (Some(true), Some(stages)) => {
                let gpu_span_ms = stages.iter().filter_map(|s| s.gpu_ms).sum();
                profiled_row = Some(ProfileRow {
                    step,
                    loss: value,
                    step_ms: step_seconds * 1e3,
                    stages,
                    gpu_span_ms,
                    next_unprofiled_ms: None,
                });
            }
            (Some(false), _) => {
                if let Some(mut row) = profiled_row.take() {
                    row.next_unprofiled_ms = Some(step_seconds * 1e3);
                    append_line(&out.join("profile.jsonl"), &serde_json::to_string(&row)?)?;
                    profile_rows.push(row);
                }
                if Some(step + 1) == config.profile.checked_mul(2).map(|w| w + PROFILE_WARMUP) {
                    let summary = summarize_profile(&profile_rows);
                    fs::write(
                        out.join("profile.json"),
                        serde_json::to_vec_pretty(&summary)?,
                    )?;
                    println!(
                        "[profile] {} steps: stages sum to {:.3} ms against {:.3} ms unprofiled \
                         (ratio {:.3}); the clock adds {:.3}; top three {:?}",
                        summary.profiled_steps,
                        summary.stage_sum_ms,
                        summary.median_unprofiled_ms,
                        summary.ratio,
                        summary.clock_overhead,
                        summary.top3
                    );
                }
            }
            _ => {}
        }
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
    fn exact_length_batches_cover_each_prompt_once_without_padding() {
        let prompts: Vec<Vec<u32>> = [17usize, 19, 17, 32, 18, 19, 17]
            .into_iter()
            .enumerate()
            .map(|(seed, len)| (0..len).map(|i| (i + seed) as u32).collect())
            .collect();
        let groups = exact_length_batches(&prompts, 16, SHEET_CONTEXT);
        let mut seen: Vec<usize> = groups.iter().flatten().copied().collect();
        seen.sort_unstable();
        assert_eq!(seen, (0..prompts.len()).collect::<Vec<_>>());
        for group in &groups {
            let width = prompts[group[0]].len();
            assert!(group.iter().all(|&index| prompts[index].len() == width));
            assert!(group.len() * width <= hs::SHEET_EVAL_BATCH * SHEET_CONTEXT);
        }
        assert_eq!(
            groups,
            exact_length_batches(&prompts, 16, SHEET_CONTEXT),
            "the grouping is deterministic"
        );
    }

    #[test]
    fn exact_length_batches_split_a_wide_group_at_the_evaluation_budget() {
        let prompts: Vec<Vec<u32>> = (0..40).map(|seed| vec![seed; SHEET_CONTEXT]).collect();
        let groups = exact_length_batches(&prompts, 64, SHEET_CONTEXT);
        assert!(groups.len() > 1);
        assert!(groups.iter().all(|group| group.len() <= hs::SHEET_EVAL_BATCH));
        let near_context: Vec<Vec<u32>> =
            (0..40).map(|seed| vec![seed; SHEET_CONTEXT - 12]).collect();
        let groups = exact_length_batches(&near_context, 64, SHEET_CONTEXT);
        assert!(groups.iter().all(|group| {
            group.len() * SHEET_CONTEXT <= hs::SHEET_EVAL_BATCH * SHEET_CONTEXT
        }));
    }

    #[test]
    fn batched_decode_is_opt_in_and_falls_back_for_every_unsupported_case() {
        assert!(can_batch_decode(
            true,
            PointObjective::Cell,
            ReadScope::Full,
            false,
            false
        ));
        assert!(!can_batch_decode(
            false,
            PointObjective::Cell,
            ReadScope::Full,
            false,
            false
        ));
        assert!(!can_batch_decode(
            true,
            PointObjective::Sheet,
            ReadScope::Full,
            false,
            false
        ));
        assert!(!can_batch_decode(
            true,
            PointObjective::Cell,
            ReadScope::Copy,
            false,
            false
        ));
        assert!(!can_batch_decode(
            true,
            PointObjective::Cell,
            ReadScope::Full,
            true,
            false
        ));
        assert!(!can_batch_decode(
            true,
            PointObjective::Cell,
            ReadScope::Full,
            false,
            true
        ));
    }

    #[test]
    fn root_separation_reads_the_sixteen_roots() {
        let (nearest, farthest) = root_separation().unwrap();
        println!("address roots: nearest {nearest}, farthest {farthest}");
        assert!(nearest > 0.0 && nearest <= farthest);
    }

    /// claude-252: a float trace is side F unless put on side T, and no other side is taken.
    #[test]
    fn a_float_trace_is_side_f_or_t() {
        let side = |trace: FTrace| trace.trace.side;
        assert_eq!(side(FTrace::new(&[500]).unwrap()), "F");
        assert_eq!(
            side(FTrace::new(&[500]).unwrap().on_side("T").unwrap()),
            "T"
        );
        assert_eq!(
            side(FTrace::new(&[500]).unwrap().on_side("F").unwrap()),
            "F"
        );
        assert!(FTrace::new(&[500]).unwrap().on_side("R").is_err());
    }

    /// claude-251: a traced window's steps hold, in R's order, each layer's buckets and the stream
    /// read at its depth, the readout read, the memory's lookup and, at each scored offset only,
    /// the nearest: the decoded pick, the scores' runner-up (ties to the smaller id), both scores
    /// in R's units. Kept sets come nearest first, ties to the earlier position.
    #[test]
    fn a_traced_window_records_the_five_kinds_in_rs_order() {
        use uor_r4_integer::fbs_snapshot::HammingTraceHead;
        let time = SHEET_CONTEXT;
        let window = |offsets: Vec<usize>, first: usize, seed: u32| CausalWindow {
            doc_id: 0,
            input_ids: (0..time as u32).map(|i| (i * 7 + seed) % 3).collect(),
            scored_targets: vec![0; offsets.len()],
            scored_corpus_positions: offsets.iter().map(|o| first + o).collect(),
            scored_offsets: offsets,
        };
        let windows = [window(vec![3, 4], 97, 1), window(vec![2, 3, 5], 498, 2)];
        let refs: Vec<&CausalWindow> = windows.iter().collect();
        let ids: Vec<u32> = windows.iter().flat_map(|w| w.input_ids.clone()).collect();
        let scored: Vec<u32> = [3, 4, time + 2, time + 3, time + 5].map(|r| r as u32).to_vec();
        let best = [9, 9, 1, 2, 1];
        let points = vec![0f32, 0.0, 1.0, 0.0, 0.0, 2.0];
        let queries = [[0f32, 0.0], [0.0, 0.0], [0.9, 0.1], [0.1, 1.9], [0.5, -0.25]];
        let rows = [[0f32; 3], [0.0; 3], [-0.5, -0.1, -1.4], [-1.0, -1.0, -0.1], [-0.5, -0.5, -1.0]];
        // Two layers of two lanes: layer l's lane j at row r scores ((r + 5j + l) mod 17) / 16, at
        // most 1, and uses the entry the kernel takes from it, min(⌊s·16⌋, 15), written out here.
        let rates: Vec<RateScores> = (0..2)
            .map(|layer| {
                let score: Vec<f32> = (0..2 * time * 2)
                    .map(|i| (((i / 2 + 5 * (i % 2) + layer) % 17) as f32 / 16.0).min(1.0))
                    .collect();
                let entry = score
                    .iter()
                    .map(|&s| ((s * 16.0).floor() as u8).min(15))
                    .collect();
                RateScores {
                    layer,
                    dims: (2, time, 2),
                    score,
                    entry,
                }
            })
            .collect();
        // One head; at row 5 it keeps 4, 2, 1 and 3, at distances 1, 1, 0 and 2.
        let head = |b: u64| {
            let mut key = vec![0u64; time];
            (key[1], key[2], key[3], key[4]) = (0b1010, 0b1011, 0b0000, 0b1000);
            let mut kept = vec![Vec::new(); time];
            kept[5] = vec![4, 2, 1, 3];
            HammingTraceHead {
                query: (0..time as u64).map(|t| if t == 5 { 0b1010 } else { t + b }).collect(),
                key,
                kept,
                query_f32: (0..time).map(|t| vec![(t as f32).to_bits()]).collect(),
                key_f32: (0..time).map(|t| vec![(-(t as f32)).to_bits()]).collect(),
            }
        };
        let hamming = |ids: &[u32], b: u64| HammingTraceWindow {
            ids: ids.to_vec(),
            reads: vec![
                HammingTraceRead { depth: 1, heads: vec![head(b)] },
                HammingTraceRead { depth: 2, heads: vec![head(b)] },
            ],
        };
        let recorded = |hamming: Vec<HammingTraceWindow>| {
            let mut lookups = vec![None; 2 * time];
            lookups[time + 2] = Some(MemoryLookup { key: vec![1, 2], stored: Some(4), opening: true });
            lookups[time + 3] = Some(MemoryLookup { key: vec![1, 9], stored: None, opening: false });
            Recorded { hamming, rates: rates.clone(), lookups: Some(lookups) }
        };
        let both = || vec![hamming(&ids[..time], 0), hamming(&ids[time..], 1)];
        // What `row_picks` copies to the host: the traced rows of the points and of the scores.
        let run_with = |trace: &mut FTrace, recorded: Recorded| {
            let traced: Vec<usize> = (0..2).filter(|&b| trace.wants(refs[b])).collect();
            let at = traced_rows(&scored, &traced);
            let q: Vec<Vec<f32>> = at.iter().map(|&i| queries[i as usize].to_vec()).collect();
            let s: Vec<Vec<f32>> = at.iter().map(|&i| rows[i as usize].to_vec()).collect();
            trace.table = Some(points.clone());
            trace.record(&refs, &traced, &ids, &scored, &best, &q, &s, recorded)
        };
        let run = |trace: &mut FTrace, hamming: Vec<HammingTraceWindow>| {
            run_with(trace, recorded(hamming))
        };
        let mut trace = FTrace::new(&[500]).unwrap();
        run(&mut trace, both()).unwrap();
        let trace = trace.finish().unwrap();
        assert_eq!((trace.side.as_str(), trace.layers, trace.lanes, trace.heads), ("F", 2, 2, 1));
        let [w] = trace.windows.as_slice() else { panic!("one window is traced") };
        assert_eq!((w.position, w.reply_start), (500, 2));
        assert_eq!((w.ids.as_slice(), w.steps.len()), (&windows[1].input_ids[..6], 6));
        let mut clamped = false;
        for (t, step) in w.steps.iter().enumerate() {
            let row = time + t;
            for (l, layer) in step.layers.iter().enumerate() {
                let score = &rates[l].score[row * 2..row * 2 + 2];
                assert_eq!(layer.score_f32, score.iter().map(|s| s.to_bits()).collect::<Vec<_>>());
                // The kernel's entry, min(⌊s·16⌋, 15), written out here rather than read from
                // `rate_index`; a score of 1 takes the last entry.
                let entry: Vec<u8> = score.iter().map(|&s| ((s * 16.0).floor() as u8).min(15)).collect();
                clamped |= score.iter().zip(&layer.entry).any(|(&s, &e)| s == 1.0 && e == 15);
                assert_eq!((layer.entry.clone(), layer.score_q16.len()), (entry, 0));
                assert_eq!(layer.read.as_ref().map(|r| r.depth), (l == 0).then_some(1));
            }
            let readout = step.readout.as_ref().unwrap();
            assert_eq!((readout.depth, readout.heads[0].query_f32.clone()), (2, vec![(t as f32).to_bits()]));
            assert_eq!(step.nearest.is_some(), [2, 3, 5].contains(&t));
            let opening = match t {
                2 => Some(true),
                3 => Some(false),
                _ => None,
            };
            assert_eq!(step.memory.as_ref().map(|m| m.opening), opening);
        }
        assert!(clamped, "the window holds a score of 1, at the last entry");
        // Nearest first; 2 and 4 tie at distance 1, and the earlier comes first.
        let read = &w.steps[5].layers[0].read.as_ref().unwrap().heads[0];
        assert_eq!((read.query, read.kept.clone()), (0b1010, vec![1, 2, 4, 3]));
        assert_eq!((read.key, read.key_f32.clone()), (0, vec![(-5.0f32).to_bits()]));
        let nearest = |t: usize| w.steps[t].nearest.clone().unwrap();
        let (n2, n3, n5) = (nearest(2), nearest(3), nearest(5));
        assert_eq!((n2.pick, n2.runner_up, n3.pick, n3.runner_up, n5.runner_up), (1, 0, 2, 0, 0));
        // −2^12 · (|0.9f32 − 1| + |0.1f32|), summed in f64; an f32 sum rounds it.
        assert_eq!(n2.score_f64.map(f64::from_bits), Some(-819.200_103_759_765_6));
        assert_ne!((-4096.0 * ((0.9f32 - 1.0).abs() + 0.1f32)) as f64, -819.200_103_759_765_6);
        assert_eq!(n5.score_f64.map(f64::from_bits), Some(-3072.0));
        assert_eq!(n5.runner_score_f64.map(f64::from_bits), Some(-3072.0));
        assert_eq!(n3.query_f32, queries[3].map(f32::to_bits).to_vec());
        assert!(n2.score_q12.is_none() && n2.runner_score_q12.is_none() && n2.query_q12.is_empty());
        // A position no window holds, a position named twice, a Hamming record of another
        // window, a head without coordinates and a one-token table are refused.
        let mut missing = FTrace::new(&[500, 777]).unwrap();
        run(&mut missing, both()).unwrap();
        assert!(missing.finish().is_err());
        assert!(FTrace::new(&[500, 500]).is_err());
        let swapped = vec![hamming(&ids[time..], 1), hamming(&ids[..time], 0)];
        assert!(run(&mut FTrace::new(&[500]).unwrap(), swapped).is_err());
        let mut bare = head(0);
        bare.query_f32.clear();
        assert!(decision_read(&HammingTraceRead { depth: 1, heads: vec![bare] }, 5).is_err());
        assert!(runner_up(&[1.0], 0).is_err());

        // claude-253: under a forced set, each forced decision is recorded as the forward used
        // it, its score as computed; a run in which one was not taken, or one past the window's
        // traced steps, is refused. The fixture's reads sit at depths 1 and 2, one head of 4 bits.
        use crate::flock_read_hamming::ForcedRowCode;
        use crate::geometric_stack::ForcedRate;
        use uor_r4_integer::fbs_snapshot::{ForcedBucket, ForcedCode};
        let shape = [(1, 1, 4), (2, 1, 4)];
        let forced = |buckets: Vec<ForcedBucket>, codes: Vec<ForcedCode>| DecisionForce {
            schema: DECISION_FORCE_SCHEMA.to_owned(),
            windows: vec![DecisionForceWindow {
                position: 500,
                buckets,
                codes,
            }],
        };
        let under = |force: DecisionForce| {
            FTrace::new(&[500])
                .unwrap()
                .with_force(force, 2, 2, &shape)
                .unwrap()
        };
        let row = time + 3;
        let other = if rates[1].entry[row * 2] == 15 { 0 } else { 15 };
        let bucket = ForcedBucket {
            step: 3,
            layer: 1,
            lane: 0,
            entry: other,
        };
        let query = ForcedCode {
            step: 4,
            depth: 1,
            head: 0,
            role: CodeRole::Query,
            code: 0b1111,
        };
        // The fixture's key at step 2, forced to itself.
        let key = ForcedCode {
            step: 2,
            depth: 2,
            head: 0,
            role: CodeRole::Key,
            code: 0b1011,
        };
        // What a forward that took them recorded.
        let mut taken_rates = rates.clone();
        taken_rates[1].entry[row * 2] = other;
        let mut taken_hamming = both();
        taken_hamming[1].reads[0].heads[0].query[4] = 0b1111;
        let taken = |rates: Vec<RateScores>, hamming| Recorded {
            hamming,
            rates,
            lookups: None,
        };
        let mut trace = under(forced(
            vec![bucket.clone()],
            vec![query.clone(), key.clone()],
        ));
        run_with(
            &mut trace,
            taken(taken_rates.clone(), taken_hamming.clone()),
        )
        .unwrap();
        let trace = trace.finish().unwrap();
        let steps = &trace.windows[0].steps;
        assert_eq!(steps[3].layers[1].entry[0], other, "the entry used");
        assert_eq!(
            steps[3].layers[1].score_f32[0],
            rates[1].score[row * 2].to_bits(),
            "the score as computed"
        );
        assert_eq!(
            steps[4].layers[0].read.as_ref().unwrap().heads[0].query,
            0b1111
        );
        assert_eq!(steps[2].readout.as_ref().unwrap().heads[0].key, 0b1011);
        // Not taken: the bucket, then the query's code; and a step past the six traced.
        assert!(run_with(
            &mut under(forced(vec![bucket.clone()], vec![])),
            taken(rates.clone(), taken_hamming.clone())
        )
        .is_err());
        assert!(run_with(
            &mut under(forced(vec![], vec![query.clone()])),
            taken(taken_rates.clone(), both())
        )
        .is_err());
        let late = ForcedBucket {
            step: 6,
            ..bucket.clone()
        };
        assert!(run_with(
            &mut under(forced(vec![late], vec![])),
            taken(rates.clone(), both())
        )
        .is_err());
        // The key forced to another code: taken where the key reads it (the query left as
        // computed), refused where it does not.
        let other_key = ForcedCode {
            code: 0b0100,
            ..key.clone()
        };
        let mut key_taken = both();
        key_taken[1].reads[1].heads[0].key[2] = 0b0100;
        assert_ne!(key_taken[1].reads[1].heads[0].query[2], 0b0100);
        assert!(run_with(
            &mut under(forced(vec![], vec![other_key.clone()])),
            taken(rates.clone(), key_taken)
        )
        .is_ok());
        assert!(run_with(
            &mut under(forced(vec![], vec![other_key])),
            taken(rates.clone(), both())
        )
        .is_err());
        // Each forced decision at its window's row of the batch, `b · SHEET_CONTEXT + step`.
        let trace = under(forced(vec![bucket.clone()], vec![key.clone()]));
        assert_eq!(
            trace.batch_force(&refs, &[1]),
            (
                vec![ForcedRate {
                    layer: 1,
                    row: SHEET_CONTEXT + 3,
                    lane: 0,
                    entry: other
                }],
                vec![ForcedRowCode {
                    depth: 2,
                    row: SHEET_CONTEXT + 2,
                    head: 0,
                    key: true,
                    code: 0b1011
                }]
            )
        );
        assert_eq!(trace.batch_force(&refs, &[]), (vec![], vec![]));
        assert_eq!(
            FTrace::new(&[500]).unwrap().batch_force(&refs, &[1]),
            (vec![], vec![])
        );
        // R's refusals, and a code with a bit past its head's 4.
        let refused = |buckets: Vec<ForcedBucket>, codes: Vec<ForcedCode>| {
            FTrace::new(&[500])
                .unwrap()
                .with_force(forced(buckets, codes), 2, 2, &shape)
                .is_err()
        };
        let b = |step, layer, lane, entry| ForcedBucket {
            step,
            layer,
            lane,
            entry,
        };
        let c = |step, depth, head, role, code| ForcedCode {
            step,
            depth,
            head,
            role,
            code,
        };
        let (q, k, last) = (CodeRole::Query, CodeRole::Key, SHEET_CONTEXT - 1);
        assert!(!refused(
            vec![b(last, 1, 1, 15)],
            vec![c(last, 2, 0, k, 0b1111), c(last, 2, 0, q, 0)]
        ));
        for bad in [
            b(SHEET_CONTEXT, 0, 0, 0),
            b(0, 2, 0, 0),
            b(0, 0, 2, 0),
            b(0, 0, 0, 16),
        ] {
            assert!(refused(vec![bad], vec![]));
        }
        assert!(refused(vec![b(1, 0, 0, 3), b(1, 0, 0, 4)], vec![]));
        for bad in [
            c(1, 3, 0, q, 0),
            c(1, 1, 1, q, 0),
            c(SHEET_CONTEXT, 1, 0, q, 0),
            c(0, 1, 0, k, 0),
            c(1, 1, 0, q, 0b1_0000),
        ] {
            assert!(refused(vec![], vec![bad]));
        }
        assert!(refused(vec![], vec![c(1, 1, 0, q, 1), c(1, 1, 0, q, 2)]));
        // Another schema, an untraced position and a position named twice.
        let file = |schema: &str, positions: &[usize]| DecisionForce {
            schema: schema.to_owned(),
            windows: positions
                .iter()
                .map(|&position| DecisionForceWindow {
                    position,
                    ..Default::default()
                })
                .collect(),
        };
        let with = |force| {
            FTrace::new(&[500])
                .unwrap()
                .with_force(force, 2, 2, &shape)
                .is_err()
        };
        assert!(!with(file(DECISION_FORCE_SCHEMA, &[500])));
        assert!(with(file("uor-r4.decision-force/0", &[500])));
        assert!(with(file(DECISION_FORCE_SCHEMA, &[777])));
        assert!(with(file(DECISION_FORCE_SCHEMA, &[500, 500])));
        // A step past its window's traced steps (0 to 5 at position 500), and a position with no
        // window, refused before any forward.
        let at = |step| {
            forced(
                vec![ForcedBucket {
                    step,
                    ..bucket.clone()
                }],
                vec![],
            )
        };
        assert!(forced_within(&at(5), &windows).is_ok());
        assert!(forced_within(&at(6), &windows).is_err());
        assert!(forced_within(
            &forced(
                vec![],
                vec![ForcedCode {
                    step: 6,
                    ..query.clone()
                }]
            ),
            &windows
        )
        .is_err());
        assert!(forced_within(&file(DECISION_FORCE_SCHEMA, &[777]), &windows).is_err());
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

    /// claude-218: the probe counts where a key fires (TRAIN saw it) and its hits there, keys
    /// order-free by the distinct ids, and keys the oracle by each opening's own reply template.
    #[test]
    fn the_opening_bound_counts_where_a_key_fires_and_its_precision() {
        // Any order of the same words is one order-free key.
        assert_eq!(PromptKey::Set.of(&[5, 3, 5, 1]), [1, 3, 5]);
        assert_eq!(PromptKey::Set.of(&[3, 5]), PromptKey::Set.of(&[5, 3]));
        assert_eq!(PromptKey::SetFirst(2).of(&[5, 3, 1]), [3, 5]);
        assert_eq!(PromptKey::First(2).of(&[5, 3, 1]), [5, 3]);
        // TRAIN: key [1] opens with 7 twice and 8 once, key [2] with 9; 7 is the commonest.
        let train = [(vec![1], 7), (vec![1], 7), (vec![1], 8), (vec![2], 9)];
        // [1] fires and is right, [2] fires and is wrong, [3] is unseen and the fallback 7 is
        // right, [4] is unseen and wrong: 2 hits, 1 of them where the key fired.
        let rows = [(vec![1], 7), (vec![2], 7), (vec![3], 7), (vec![4], 5)];
        let b = bound("p", "k".to_owned(), &train, &rows, &|_| String::new());
        assert_eq!((b.rows, b.unseen, b.hits, b.fired_hits, b.train_keys), (4, 2, 2, 1, 2));
        // Two documents: BOS, a prompt (10 11), a reply (20 21), a prompt (12), a reply (22);
        // then BOS, a prompt (13) and a reply (23). Each reply runs while the mask holds, to BOS.
        let tokens = [0u32, 10, 11, 20, 21, 12, 22, 0, 13, 23];
        let mask = [0u8, 0, 0, 1, 1, 0, 1, 0, 0, 1];
        assert_eq!(reply_at(&tokens, &mask, 3), [20, 21]);
        assert_eq!(reply_at(&tokens, &mask, 6), [22]);
        assert_eq!(reply_at(&tokens, &mask, 9), [23]);
        let (windows, _) = hs::build_document_windows(&tokens, &mask, 16, 0).unwrap();
        let opened = opening_rows(&windows, &mask);
        assert_eq!(opened.iter().map(|r| r.position).collect::<Vec<_>>(), [3, 6, 9]);
        // The oracle's key is the reply with its values masked: 23 is not kept, so it is a value.
        let keep: std::collections::HashSet<u32> = [20, 21, 22].into();
        let templates = template_keyed(&opened, &tokens, &mask, &keep, &Default::default());
        assert_eq!(templates, [(vec![20, 21], 20), (vec![22], 22), (vec![CENSUS_MASK], 23)]);
    }

    /// claude-218: the key grade's parts, and the sheet's decision table read in order. τ is level
    /// ℓ's ⌈ℓ·n/10⌉-th smallest distance, +∞ at q100; a key names a row where it fires and the
    /// base elsewhere; the level choice keeps the lower on a tie; the forced arm is paired with
    /// greedy on both measures and refused when it does not open with the reference's token.
    #[test]
    fn the_key_grade_reads_the_sheets_table_in_order() {
        let d = [5.0f32, 1.0, 3.0, 2.0, 4.0];
        assert_eq!([level_tau(&d, 1), level_tau(&d, 5), level_tau(&d, 9)], [1.0, 3.0, 5.0]);
        assert_eq!(level_tau(&d, 10), f32::INFINITY);
        // Four rows: the key names 7 at distances 1–4, the base 9; the targets are 7 7 9 9.
        let (targets, named, base) = ([7u32, 7, 9, 9], [7u32; 4], [9u32; 4]);
        let (distance, rows) = ([1.0f32, 2.0, 3.0, 4.0], [0usize, 1, 2, 3]);
        let at = |tau, fires: &dyn Fn(usize) -> bool| {
            key_reading(("p", "point", "missed", 1, 5, tau), &rows, &targets, &named, &distance, &base, fires)
        };
        let r = at(2.0, &|_| true);
        assert_eq!((r.fired, r.b, r.c, r.fired_hits, r.base_fired_hits), (2, 2, 0, 2, 0));
        let r = at(f32::INFINITY, &|_| true);
        assert_eq!((r.fired, r.b, r.c), (4, 2, 2));
        // A row the backoff excludes never fires: row 0 keeps the base's wrong 9.
        let r = at(f32::INFINITY, &|i| i != 0);
        assert_eq!((r.fired, r.b, r.c), (3, 1, 2));
        // The level choice, summed over seeds: a tie keeps the lower level.
        let (mut a, mut b) = ([0i64; 10], [0i64; 10]);
        (a[2], a[6], b[6]) = (5, 5, 1);
        assert_eq!((choose_level(&[a]), choose_level(&[a, b])), (3, 7));
        // The forced arm. The end of turn is 1; 9 is a value (not kept). Prompt 0: greedy writes
        // 7 7 7, forced the reference 5 6 1. Prompt 1: greedy 5 9 1 has the reference's template
        // 5 · 1 but not its words 5 8 1, which forced writes.
        let keep: std::collections::HashSet<u32> = [1, 5, 6, 7].into();
        let sample = |position: usize, reply: Vec<u32>, reference: Vec<u32>| IdSample {
            position,
            prompt: vec![2],
            reply,
            reference,
            reply_text: String::new(),
            reference_text: String::new(),
            reply_keys: 0,
            reply_named: 0,
            reference_keys: 0,
            reference_named: 0,
            admitted: Vec::new(),
            stepped: Vec::new(),
            keyed: None,
            served_distance: None,
        };
        let greedy = [sample(0, vec![7, 7, 7], vec![5, 6, 1]), sample(1, vec![5, 9, 1], vec![5, 8, 1])];
        let forced = [sample(0, vec![5, 6, 1], vec![5, 6, 1]), sample(1, vec![5, 8, 1], vec![5, 8, 1])];
        let none = std::collections::HashSet::new();
        let f = forced_reading("p", 1, &greedy, &forced, 1, &keep, &none).unwrap();
        assert_eq!((f.exact, f.template), ((2, 0), (1, 0)));
        let wrong = [sample(0, vec![6, 6, 1], vec![5, 6, 1]), forced[1].clone()];
        assert!(forced_reading("p", 1, &greedy, &wrong, 1, &keep, &none).is_err());
        // The table, in order.
        let read = |b: usize, c: usize, hits: usize, base: usize| KeyReading {
            split: "p-u".to_owned(),
            key: String::new(),
            stratum: String::new(),
            seed: 1,
            level: 5,
            tau: 1.0,
            rows: 1218,
            fired: 600,
            b,
            c,
            fired_hits: hits,
            base_fired_hits: base,
        };
        let seeds = |r: KeyReading| vec![r.clone(), r.clone(), r];
        let lift = |exact, template| ForcedReading { split: "p-u".to_owned(), seed: 1, replies: 256, exact, template };
        let flat = vec![lift((0, 0), (6, 5)); 3];
        let lifted = vec![lift((3, 0), (30, 0)); 3];
        let thin = vec![lift((2, 0), (3, 1)); 3];
        let gain = seeds(read(200, 10, 250, 60));
        let neutral = seeds(read(50, 45, 0, 0));
        let lost = seeds(read(10, 60, 0, 0));
        let even = seeds(read(10, 10, 100, 100));
        let set = |sign: &Vec<KeyReading>, point: &Vec<KeyReading>| vec![("sign", sign.clone()), ("point", point.clone())];
        let headroom = vec![(300, 0); 3];
        // Row 1: a forced opening lifts nothing beyond its band, and the reading has power.
        assert_eq!(key_outcome(&flat, &headroom, &set(&gain, &gain), &set(&neutral, &neutral), &set(&neutral, &neutral), &set(&even, &even)).0, 1);
        // An underpowered forced reading cannot hold row 1.
        assert_ne!(key_outcome(&thin, &headroom, &set(&gain, &gain), &set(&neutral, &neutral), &set(&neutral, &neutral), &set(&even, &even)).0, 1);
        // Row 2: U has no headroom over the larger arm.
        assert_eq!(key_outcome(&lifted, &[(5, 5); 3], &set(&gain, &gain), &set(&neutral, &neutral), &set(&neutral, &neutral), &set(&even, &even)).0, 2);
        // Row 3: the sign key passes all four and ships before the point key.
        let r = key_outcome(&lifted, &headroom, &set(&gain, &gain), &set(&neutral, &neutral), &set(&neutral, &neutral), &set(&even, &even));
        assert_eq!((r.0, r.2.as_deref()), (3, Some("sign")));
        let r = key_outcome(&lifted, &headroom, &set(&even, &gain), &set(&neutral, &neutral), &set(&neutral, &neutral), &set(&even, &even));
        assert_eq!((r.0, r.2.as_deref()), (3, Some("point")));
        // Casey's amendment: the Hadamard sign key ships when it passes and the sign key does not,
        // before the point key; the sign key, cheaper still, ships first when both pass.
        let set3 = |sign: &Vec<KeyReading>, hsign: &Vec<KeyReading>, point: &Vec<KeyReading>| {
            vec![("sign", sign.clone()), ("hsign", hsign.clone()), ("point", point.clone())]
        };
        let (n3, e3) = (set3(&neutral, &neutral, &neutral), set3(&even, &even, &even));
        let r = key_outcome(&lifted, &headroom, &set3(&even, &gain, &gain), &n3, &n3, &e3);
        assert_eq!((r.0, r.2.as_deref()), (3, Some("hsign")));
        let r = key_outcome(&lifted, &headroom, &set3(&gain, &gain, &gain), &n3, &n3, &e3);
        assert_eq!((r.0, r.2.as_deref()), (3, Some("sign")));
        // A loss beyond its band on the new stratum, or on DEV's backoff, fails row 3.
        assert_eq!(key_outcome(&lifted, &headroom, &set(&gain, &gain), &set(&lost, &lost), &set(&neutral, &neutral), &set(&even, &even)).0, 5);
        assert_eq!(key_outcome(&lifted, &headroom, &set(&gain, &gain), &set(&neutral, &neutral), &set(&lost, &lost), &set(&even, &even)).0, 5);
        // FIT round 2: a small loss inside its band on the new stratum still passes row 3.
        let mild = seeds(read(45, 50, 0, 0));
        assert_eq!(key_outcome(&lifted, &headroom, &set(&gain, &gain), &set(&mild, &mild), &set(&neutral, &neutral), &set(&even, &even)).0, 3);
        // Row 4: no key passes, and one loses beyond its band at q100. A loss inside the band, or
        // none, is row 5.
        assert_eq!(key_outcome(&lifted, &headroom, &set(&even, &even), &set(&neutral, &neutral), &set(&neutral, &neutral), &set(&lost, &even)).0, 4);
        assert_eq!(key_outcome(&lifted, &headroom, &set(&even, &even), &set(&neutral, &neutral), &set(&neutral, &neutral), &set(&mild, &mild)).0, 5);
        assert_eq!(key_outcome(&lifted, &headroom, &set(&even, &even), &set(&neutral, &neutral), &set(&neutral, &neutral), &set(&even, &even)).0, 5);
        // The code review: row 1 is one-sided (a forced arm that loses still holds it); row 2 is
        // two-sided (U far behind does not hold it); row 3 wants a gain strictly beyond its band
        // (12 − 4 = 8 = 2·√16 does not pass); and a reading with b + c under 10 holds neither
        // row 2 nor row 4 (FIT round 3).
        let behind = vec![lift((0, 30), (0, 30)); 3];
        assert_eq!(key_outcome(&behind, &headroom, &set(&gain, &gain), &set(&neutral, &neutral), &set(&neutral, &neutral), &set(&even, &even)).0, 1);
        assert_ne!(key_outcome(&lifted, &[(0, 300); 3], &set(&even, &even), &set(&neutral, &neutral), &set(&neutral, &neutral), &set(&even, &even)).0, 2);
        let edge = seeds(read(12, 4, 12, 4));
        assert_eq!(key_outcome(&lifted, &headroom, &set(&edge, &edge), &set(&neutral, &neutral), &set(&neutral, &neutral), &set(&even, &even)).0, 5);
        assert_ne!(key_outcome(&lifted, &[(3, 3); 3], &set(&even, &even), &set(&neutral, &neutral), &set(&neutral, &neutral), &set(&even, &even)).0, 2);
        // FIT round 4: a gain on too few rows (5 − 0 > 2·√5, but b + c is 5) does not pass row 3.
        let thin_gain = seeds(read(5, 0, 5, 0));
        assert_eq!(key_outcome(&lifted, &headroom, &set(&thin_gain, &thin_gain), &set(&neutral, &neutral), &set(&neutral, &neutral), &set(&even, &even)).0, 5);
        let thin_loss = seeds(read(0, 9, 0, 0));
        assert_eq!(key_outcome(&lifted, &headroom, &set(&even, &even), &set(&neutral, &neutral), &set(&neutral, &neutral), &set(&thin_loss, &thin_loss)).0, 5);
        // The wiring: the strata, and row 2's arm. Openings 0–3: the exact key fires at 0 only;
        // references 1 is a verbatim TRAIN reply, 2 and 3 are not.
        let opened = GradeOpenings {
            name: "p".to_owned(),
            positions: vec![10, 20, 30, 40],
            targets: vec![1, 2, 3, 4],
            exact: vec![(1, true), (2, false), (0, false), (0, false)],
            oracle: vec![1, 2, 3, 0],
            known: vec![false, true, false, false],
        };
        assert_eq!(key_strata(&opened), (vec![1, 2, 3], vec![2, 3], vec![1]));
        // The exact key and B tie at 2 hits, so U is read against the exact key: right where it
        // is wrong at opening 2 only. With B ahead (3 hits), U is read against B.
        assert_eq!(oracle_reading(&opened, &[1, 0, 0, 4]), (1, 0));
        assert_eq!(oracle_reading(&opened, &[1, 2, 3, 0]), (0, 0));
        // The grade's refusals (the code review, round 2): a given level other than p-s's choice,
        // or none; and a split where the turn anchor does not keep the first-8 key at every
        // opening (the anchor check's own case: 2 of 3).
        let given = [("sign".to_owned(), 4), ("point".to_owned(), 7)];
        assert!(level_given(&given, "sign", 4).is_ok());
        assert!(level_given(&given, "sign", 3).is_err());
        assert!(level_given(&given[..1], "point", 7).is_err());
        let tokens = [0u32, 10, 11, 12, 20, 21, 1, 13, 14, 22, 15, 23];
        let mask = [0u8, 0, 0, 0, 1, 1, 1, 0, 0, 1, 0, 1];
        let empty = KeyTable::new(std::iter::empty());
        let none = std::collections::HashSet::new();
        let opened = grade_openings("p", &tokens, &mask, &empty, &empty, &none, &none, &std::collections::HashSet::new());
        assert!(opened.is_err(), "an opening the anchor misses refuses the grade");
        let agreeing = grade_openings("p", &tokens[..9], &mask[..9], &empty, &empty, &none, &none, &std::collections::HashSet::new());
        // Its first turn alone, whose one opening the anchor keeps, grades.
        assert_eq!(agreeing.unwrap().positions, [4]);
        // FIT round 5: the follow-up pass's keys must carry the main run's point and sign keys;
        // its Hadamard sign fields may differ, any point or sign field may not.
        let keys = |point: f32, hsign: u32| OpeningKeys {
            split: "p".to_owned(),
            train_openings: 2,
            table_bytes: 16,
            positions: vec![4],
            targets: vec![7],
            point_target: vec![7],
            point_distance: vec![point],
            point_ties: vec![1],
            sign_target: vec![7],
            sign_distance: vec![0.0],
            sign_ties: vec![2],
            point_ms: 1,
            sign_ms: 1,
            hsign_target: vec![hsign],
            hsign_distance: vec![0.0],
            hsign_ties: vec![1],
            hsign_ms: 1,
        };
        assert!(same_trunk_keys(&keys(0.5, 7), &keys(0.5, 3)));
        assert!(!same_trunk_keys(&keys(0.5, 7), &keys(0.25, 7)));
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
            prompt_register: 0,
            register_picks: None,
            register_select: None,
            width: hs::SHEET_WIDTH,
            type_spin: false,
            hamming_copy_key: 1,
            hamming_copy_keys: None,
            fix_input_table: false,
            train_split: None,
            samples: 6,
            reply_tokens: 48,
            profile: 0,
            turn_memory: 0,
            turn_memory_empty: false,
            turn_memory_reply: false,
            turn_memory_dropout: 0,
            fbs_arm: FbsArm::OneAtomActive,
            reply_read_train: None,
            train_as_served: false,
            context: hs::SHEET_CONTEXT,
        }
    }

    use crate::flock_read_opening::OpeningAnchor;

    /// claude-273: a run at the sheet's window writes no `context` key, so its `config.json` is
    /// the file it wrote before the setting; any other window is written.
    #[test]
    fn a_config_writes_its_window_only_off_the_sheets() {
        let sheet = String::from_utf8(serde_json::to_vec_pretty(&config(0, None)).unwrap()).unwrap();
        assert!(!sheet.contains("\"context\""), "{sheet}");
        let long = LongConfig { context: 2_048, ..config(0, None) };
        let long = String::from_utf8(serde_json::to_vec_pretty(&long).unwrap()).unwrap();
        assert!(long.contains("\"context\": 2048"), "{long}");
    }

    /// claude-273: a training window is 2 to 4,096 tokens, the stack's own range.
    #[test]
    fn a_training_window_is_two_to_four_thousand_ninety_six_tokens() {
        for good in [2, SHEET_CONTEXT, 2_048, 4_096] {
            assert_eq!(training_context(good).unwrap(), good);
        }
        for bad in [0, 1, 4_097] {
            assert!(training_context(bad).is_err(), "{bad}");
        }
    }

    /// claude-273: a forward's window is its ids over its batch; a layout that does not divide
    /// is refused.
    #[test]
    fn a_forward_window_is_its_ids_over_its_batch() {
        assert_eq!(window_time(4 * 2_048, 4).unwrap(), 2_048);
        assert_eq!(window_time(16 * SHEET_CONTEXT, 16).unwrap(), SHEET_CONTEXT);
        assert!(window_time(4 * 2_048 + 1, 4).is_err());
        assert!(window_time(0, 4).is_err());
        assert!(window_time(8, 0).is_err());
    }

    /// claude-273: `assemble` lays each scored row at its window's start in the flat ids. At a
    /// 2,048 window the rows are `b · 2,048 + offset`; at the sheet's they are the old
    /// `b · 192 + offset`. Each row's input is the corpus token before its target.
    #[test]
    fn assembled_rows_follow_the_window() {
        // Three documents, 300, 40 and 2,100 tokens after their BOS: at 192 they cut into 2, 1
        // and 11 windows, at 2,048 into 1, 1 and 2.
        let mut tokens = Vec::new();
        for len in [300u32, 40, 2_100] {
            tokens.push(hs::SHEET_BOS_ID);
            tokens.extend((0..len).map(|i| 1 + i % 900));
        }
        let mask = vec![1u8; tokens.len()];
        for (context, count) in [(SHEET_CONTEXT, 14), (2_048, 4)] {
            let (windows, _) =
                hs::build_document_windows(&tokens, &mask, context, hs::SHEET_BOS_ID).unwrap();
            assert_eq!(windows.len(), count, "context {context}");
            let refs: Vec<&CausalWindow> = windows.iter().collect();
            let (ids, scored, targets) = assemble(&refs);
            assert_eq!(ids.len(), windows.len() * context);
            let expected: Vec<u32> = windows
                .iter()
                .enumerate()
                .flat_map(|(b, w)| w.scored_offsets.iter().map(move |&o| (b * context + o) as u32))
                .collect();
            assert_eq!(scored, expected, "context {context}");
            let positions = windows.iter().flat_map(|w| w.scored_corpus_positions.iter());
            for ((&row, &target), &p) in scored.iter().zip(&targets).zip(positions) {
                assert_eq!((ids[row as usize], target), (tokens[p - 1], tokens[p]));
            }
        }
    }

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
    fn a_config_writes_the_copy_key_only_when_two() {
        // claude-261: a one-token run writes the same `config.json` as before the key.
        let one = String::from_utf8(serde_json::to_vec_pretty(&config(0, None)).unwrap()).unwrap();
        assert!(!one.contains("hamming_copy_key"), "{one}");
        assert!(
            one.contains("\"type_spin\": false,\n  \"fix_input_table\": false,"),
            "{one}"
        );
        let two = LongConfig {
            hamming_copy_key: 2,
            ..config(0, None)
        };
        let two = String::from_utf8(serde_json::to_vec_pretty(&two).unwrap()).unwrap();
        assert!(
            two.contains(
                "\"type_spin\": false,\n  \"hamming_copy_key\": 2,\n  \"fix_input_table\": false,"
            ),
            "{two}"
        );
        // claude-268: per-head keys are written only when set.
        assert!(!one.contains("hamming_copy_keys"), "{one}");
        let mixed = LongConfig {
            hamming_copy_keys: Some(vec![2, 1]),
            ..config(0, None)
        };
        let mixed = String::from_utf8(serde_json::to_vec_pretty(&mixed).unwrap()).unwrap();
        assert!(
            mixed.contains("\"type_spin\": false,\n  \"hamming_copy_keys\": [\n    2,\n    1\n  ],\n  \"fix_input_table\": false,"),
            "{mixed}"
        );
    }

    /// claude-268: the trainer refuses per-head keys that name one key for every head, keys
    /// without the read, and keys together with `--hamming-copy-key`, before it builds the read.
    #[test]
    fn the_trainer_refuses_copy_keys_it_would_record_two_ways() {
        let refused = |config: LongConfig, needle: &str| {
            let err = start_hamming(&config, &Device::Cpu).err().map(|e| e.to_string());
            assert!(err.as_deref().is_some_and(|e| e.contains(needle)), "{needle}: {err:?}");
        };
        for keys in [vec![2, 2], vec![1, 1], vec![2]] {
            let config = LongConfig { hamming_read: 4, hamming_copy_keys: Some(keys), ..config(0, None) };
            refused(config, "when every head takes the same key");
        }
        let both = LongConfig {
            hamming_read: 4,
            hamming_copy_key: 2,
            hamming_copy_keys: Some(vec![2, 1]),
            ..config(0, None)
        };
        refused(both, "leave --hamming-copy-key out");
        refused(LongConfig { hamming_copy_keys: Some(vec![2, 1]), ..config(0, None) }, "--hamming-read");
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

    use crate::flock_read_register::{PromptRegister, RegisterSelect};

    #[test]
    fn a_config_without_the_register_writes_none_of_its_keys() {
        for json in [config(0, None), config(8, Some(OpeningAnchor::Turn))]
            .map(|c| String::from_utf8(serde_json::to_vec_pretty(&c).unwrap()).unwrap())
        {
            for key in ["prompt_register", "register_picks", "register_select"] {
                assert!(!json.contains(key), "{json}");
            }
        }
    }

    #[test]
    fn a_config_with_the_register_writes_its_three_keys_after_the_opening_reads() {
        for (q, select, name) in [
            (8, RegisterSelect::Learned, "learned"),
            (1, RegisterSelect::All, "all"),
        ] {
            let mut with = config(8, Some(OpeningAnchor::Turn));
            (with.prompt_register, with.register_picks) = (q, Some(8));
            with.register_select = Some(select);
            let json = String::from_utf8(serde_json::to_vec_pretty(&with).unwrap()).unwrap();
            let keys = format!(
                "\"opening_anchor\": \"turn\",\n  \"prompt_register\": {q},\n  \"register_picks\": 8,\n  \"register_select\": \"{name}\",\n  \"width\": 256,"
            );
            assert!(json.contains(&keys), "{json}");
        }
    }

    #[test]
    fn a_checkpoint_with_a_register_sets_it_on_the_thread_until_the_guard_drops() {
        // Save and load only: the vendored candle computes nothing on the CPU, but files move.
        let device = Device::Cpu;
        let directory =
            std::env::temp_dir().join(format!("claude-155-load-reads-{}", std::process::id()));
        PromptRegister::new(3, 5, RegisterSelect::Learned, 8, &device)
            .unwrap()
            .save(&directory)
            .unwrap();
        {
            let _guard = ArmGuard;
            load_reads(&directory, &device).unwrap();
            let reg = crate::flock_read_register::prompt_register().expect("the register is set");
            assert_eq!(
                (reg.q(), reg.picks(), reg.select()),
                (3, 5, RegisterSelect::Learned)
            );
            crate::flock_read_register::set_register_slots(Some(
                Tensor::zeros(1, candle_core::DType::F32, &device).unwrap(),
            ));
        }
        assert!(crate::flock_read_register::prompt_register().is_none());
        assert!(crate::flock_read_register::take_register_slots().is_none());
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
        let by = |key: PromptKey, rows: &[(Vec<u32>, u32)]| -> Vec<(Vec<u32>, u32)> {
            rows.iter().map(|(p, t)| (key.of(p), *t)).collect()
        };
        let key = PromptKey::First(1);
        let first = bound("dev", key.name(), &by(key, &train), &by(key, &dev), &decode);
        // Key 10 opens with 20 twice and 23 once; key 13 with 22; key 50 is unseen and takes
        // TRAIN's commonest opening, 20.
        assert_eq!(
            (first.hits, first.unseen, first.fired_hits, first.train_keys),
            (3, 1, 2, 2)
        );
        assert_eq!(first.top[0].key, "[10]");
        assert_eq!((first.top[0].train_rows, first.top[0].opening_rows), (3, 2));
        let key = PromptKey::None;
        let none = bound("dev", key.name(), &by(key, &train), &by(key, &dev), &decode);
        assert_eq!((none.hits, none.unseen, none.fired_hits), (2, 0, 2));
    }

    #[test]
    fn reply_rows_carry_the_turn_the_reply_so_far_and_their_kinds() {
        // BOS, a prompt (10 11 12), a reply (11 12 30), a prompt (13), a reply (31).
        let tokens = [0u32, 10, 11, 12, 11, 12, 30, 13, 31];
        let mask = [0u8, 0, 0, 0, 1, 1, 1, 0, 1];
        let (windows, _) = hs::build_document_windows(&tokens, &mask, 16, 0).unwrap();
        let rows = reply_rows(&windows, &mask);
        let row = |offset, prompt: &[u32], reply: &[u32], target, opening, copy| ReplyRow {
            window: 0,
            offset,
            prompt: prompt.to_vec(),
            reply: reply.to_vec(),
            target,
            opening,
            copy,
        };
        assert_eq!(
            rows,
            [
                row(3, &[10, 11, 12], &[], 11, true, false),
                // The reply's 11 then 12 stand side by side in the prompt: a copy.
                row(4, &[10, 11, 12], &[11], 12, false, true),
                row(5, &[10, 11, 12], &[11, 12], 30, false, false),
                row(7, &[13], &[], 31, true, false),
            ]
        );
        // claude-161's opening key is the turn at position 0; later rows key on their position.
        assert_eq!(
            ReplyKey::TurnPosition(0).of(&rows[0]),
            [10, 11, 12, u32::MAX, 0, u32::MAX]
        );
        assert_eq!(
            ReplyKey::TurnPosition(1).of(&rows[2]),
            [10, 11, 12, u32::MAX, 2, u32::MAX, 12]
        );
        assert_eq!(ReplyKey::Reply(4).of(&rows[2]), [11, 12]);
    }

    /// claude-202: a document starts at BOS, which belongs to neither turn; prompts and replies
    /// are maximal runs of one mask value inside a document.
    #[test]
    fn census_turns_split_at_bos_and_at_each_mask_change() {
        let tokens = [0, 5, 6, 7, 8, 9, 0, 5, 7, 3];
        let mask = [0, 0, 0, 1, 1, 0, 0, 0, 1, 1];
        let turns = split_turns(&tokens, &mask, 0);
        assert_eq!(turns.documents, 2);
        assert_eq!(turns.prompts, vec![vec![5, 6], vec![9], vec![5]]);
        assert_eq!(turns.replies, vec![vec![7, 8], vec![7, 3]]);
    }

    /// claude-202: n-gram windows never cross a run, and exact matches compare whole runs.
    #[test]
    fn census_ngrams_stay_inside_their_run_and_exact_matches_are_whole_runs() {
        let train = vec![vec![1, 2], vec![3, 4, 5]];
        let set = ngram_set(&train, 2);
        // TRAIN's 2-grams: [1, 2], [3, 4], [4, 5]; [2, 3] crosses runs and is absent.
        assert_eq!(set.len(), 3);
        let (share, windows) = ngram_share(&[vec![1, 2, 3]], &set, 2);
        assert_eq!((share, windows), (0.5, 2));
        let exact: std::collections::HashSet<u128> = train.iter().map(|r| census_hash(r)).collect();
        assert_eq!(exact_share(&[vec![1, 2], vec![1, 2, 3], vec![3, 4, 5], vec![9]], &exact), 0.5);
        // The length is part of the hash: [3, 4] and [3, 4, 5] differ.
        assert_ne!(census_hash(&[3, 4]), census_hash(&[3, 4, 5]));
    }

    /// claude-202: a value mask keeps the frequent tokens, masks the rest and every digit, and
    /// collapses each run of masks to one.
    #[test]
    fn census_value_masks_keep_frequent_tokens_and_collapse_masks() {
        let tokens = [0, 1, 1, 1, 2, 2, 4, 5, 3, 3, 3, 3];
        let keep = top_tokens(&tokens, 2, 0);
        // 3 is the commonest (4), then 1 (3); 2 (2) is third.
        assert_eq!(keep, [1u32, 3].into_iter().collect());
        // BOS is never counted, even when it is the commonest token; ties go to the smaller id.
        let frequent_bos = [0, 0, 0, 0, 0, 7, 7, 6, 6, 9];
        assert_eq!(top_tokens(&frequent_bos, 2, 0), [6u32, 7].into_iter().collect());
        let digits: std::collections::HashSet<u32> = [3u32].into_iter().collect();
        assert_eq!(
            value_mask(&[1, 4, 5, 3, 1, 2], &keep, &digits),
            vec![1, CENSUS_MASK, 1, CENSUS_MASK]
        );
        let tokenizer = r#"{"model": {"vocab": {"a": 4, "7": 5, "Ġ9": 6, "12": 7, "Ġx": 8}}}"#;
        assert_eq!(
            digit_tokens(tokenizer).unwrap(),
            [5u32, 6].into_iter().collect()
        );
    }

    /// claude-204: an unpinned `--panel` split loads only with a UORT header whose count matches
    /// its payload and mask, a first BOS, and ids inside the vocabulary.
    #[test]
    fn an_unpinned_panel_split_is_checked_before_it_is_counted() {
        let dir = std::env::temp_dir().join(format!("claude-204-panel-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let write = |tokens: &[u16], mask: &[u8], magic: &[u8; 4], count: u64| {
            let mut bytes = vec![0u8; hs::UORT_HEADER_BYTES];
            bytes[..4].copy_from_slice(magic);
            bytes[8..16].copy_from_slice(&count.to_le_bytes());
            bytes.extend(tokens.iter().flat_map(|t| t.to_le_bytes()));
            std::fs::write(dir.join("tokens.u16"), bytes).unwrap();
            std::fs::write(dir.join("response_mask.u8"), mask).unwrap();
        };
        write(&[0, 5, 6, 7], &[0, 0, 1, 1], b"UORT", 4);
        let (tokens, mask, sha) = load_unpinned_uort(&dir).unwrap();
        assert_eq!((tokens, mask, sha.len()), (vec![0, 5, 6, 7], vec![0, 0, 1, 1], 64));
        write(&[0, 5, 6, 7], &[0, 0, 1, 1], b"XXXX", 4);
        assert!(load_unpinned_uort(&dir).is_err(), "no UORT header");
        write(&[0, 5, 6, 7], &[0, 0, 1, 1], b"UORT", 3);
        assert!(load_unpinned_uort(&dir).is_err(), "a count that does not match");
        write(&[0, 5, 6, 7], &[0, 0, 1], b"UORT", 4);
        assert!(load_unpinned_uort(&dir).is_err(), "a mask shorter than the count");
        write(&[5, 0, 6, 7], &[0, 0, 1, 1], b"UORT", 4);
        assert!(load_unpinned_uort(&dir).is_err(), "no first BOS");
        write(&[0, 5, 9_000, 7], &[0, 0, 1, 1], b"UORT", 4);
        assert!(load_unpinned_uort(&dir).is_err(), "an id past the vocabulary");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// claude-202: step 2's rows are the later rows the turn-8 + position + reply-last-1 table
    /// does not keep at share 0.5, and the openings no first-8 key names.
    #[test]
    fn census_decision_rows_are_the_unnamed_later_rows_and_unkeyed_openings() {
        let row = |prompt: &[u32], reply: &[u32], target, opening| ReplyRow {
            window: 0,
            offset: 0,
            prompt: prompt.to_vec(),
            reply: reply.to_vec(),
            target,
            opening,
            copy: false,
        };
        let train = [
            // An opening after the prompt [1, 2], and two later rows after reply [10].
            row(&[1, 2], &[], 10, true),
            row(&[1, 2], &[10], 11, false),
            row(&[1, 2], &[10], 11, false),
            // After reply [20]: 21 once and 22 once, a 1/2 share, kept at 0.5.
            row(&[1, 2], &[20], 21, false),
            row(&[1, 2], &[20], 22, false),
            // After reply [30]: 31 once, 32 once, 33 once, a 1/3 share, not kept at 0.5.
            row(&[1, 2], &[30], 31, false),
            row(&[1, 2], &[30], 32, false),
            row(&[1, 2], &[30], 33, false),
        ];
        let reply = reply_table(ReplyKey::TurnPosition(1), &train);
        let opening = reply_table(ReplyKey::TurnPosition(0), &train);
        let dev = [
            row(&[1, 2], &[], 10, true),
            row(&[5, 6], &[], 10, true),
            row(&[1, 2], &[10], 11, false),
            row(&[1, 2], &[20], 21, false),
            row(&[1, 2], &[30], 31, false),
            row(&[1, 2], &[40], 41, false),
        ];
        let (later, openings) = census_decision_rows(&reply, &opening, &dev);
        // Rows 4 ([30], share 1/3) and 5 ([40], unseen) are unnamed; row 1's key is unseen.
        assert_eq!((later, openings), (vec![4, 5], vec![1]));
    }

    #[test]
    fn the_reply_bound_keeps_each_keys_commonest_target_at_its_share() {
        let row = |reply: &[u32], target, opening, copy| ReplyRow {
            window: 0,
            offset: 0,
            prompt: vec![1],
            reply: reply.to_vec(),
            target,
            opening,
            copy,
        };
        let train = [
            // After 5: 7 twice and 8 once, a 2/3 share.
            row(&[5], 7, false, false),
            row(&[5], 7, false, false),
            row(&[5], 8, false, false),
            // After 6: always 9.
            row(&[6], 9, false, true),
            // After 3: 4 and 2 once each; the tie goes to the smaller id.
            row(&[3], 4, false, false),
            row(&[3], 2, false, false),
            // Openings: 40 twice, 41 once.
            row(&[], 40, true, false),
            row(&[], 40, true, false),
            row(&[], 41, true, false),
        ];
        let dev = [
            row(&[5], 7, false, false),
            row(&[5], 8, false, false),
            row(&[6], 9, false, true),
            row(&[3], 2, false, false),
            row(&[4], 1, false, false),
            row(&[], 41, true, false),
        ];
        let counts = reply_bound_rows(ReplyKey::Reply(1), &train, &dev, &[0.0, 0.8]);
        let all = &counts[0];
        assert_eq!(all.train_keys, 4);
        // Named: after 5 (twice), 6, 3 and the opening; after 4 is unseen. Right: 7, 9, 2.
        assert_eq!(
            (all.all, all.copies, all.openings),
            (
                KindCount {
                    rows: 6,
                    named: 5,
                    hits: 3
                },
                KindCount {
                    rows: 1,
                    named: 1,
                    hits: 1
                },
                KindCount {
                    rows: 1,
                    named: 1,
                    hits: 0
                },
            )
        );
        assert_eq!(
            all.rest,
            KindCount {
                rows: 4,
                named: 3,
                hits: 2
            }
        );
        // At 0.8 only the key after 6 (share 1) is kept; 2/3 and 1/2 shares are dropped.
        let strict = &counts[1];
        assert_eq!(strict.train_keys, 1);
        assert_eq!(
            strict.all,
            KindCount {
                rows: 6,
                named: 1,
                hits: 1
            }
        );
        // An opening whose key is not kept takes TRAIN's commonest opening, 40: wrong for 41.
        assert_eq!(
            (
                all.opening_hits_with_fallback,
                strict.opening_hits_with_fallback
            ),
            (0, 0)
        );
    }

    #[test]
    fn the_reply_bound_falls_back_to_the_commonest_opening_only_for_openings() {
        let row = |prompt: &[u32], target, opening| ReplyRow {
            window: 0,
            offset: 0,
            prompt: prompt.to_vec(),
            reply: Vec::new(),
            target,
            opening,
            copy: false,
        };
        let train = [
            row(&[1], 40, true),
            row(&[2], 40, true),
            row(&[3], 41, true),
        ];
        // Turn 9 is unseen: the fallback names 40, which is right; turn 3 names 41, right.
        let dev = [
            row(&[9], 40, true),
            row(&[3], 41, true),
            row(&[9], 40, false),
        ];
        let counts = reply_bound_rows(ReplyKey::TurnPosition(0), &train, &dev, &[0.0]);
        assert_eq!(
            counts[0].openings,
            KindCount {
                rows: 2,
                named: 1,
                hits: 1
            }
        );
        assert_eq!(counts[0].opening_hits_with_fallback, 2);
        assert_eq!(
            counts[0].rest,
            KindCount {
                rows: 1,
                named: 0,
                hits: 0
            }
        );
    }

    #[test]
    fn a_row_that_opens_and_copies_counts_in_both_kinds_and_not_the_rest() {
        let row = ReplyRow {
            window: 0,
            offset: 0,
            prompt: vec![1],
            reply: Vec::new(),
            target: 40,
            opening: true,
            copy: true,
        };
        let counts = reply_bound_rows(
            ReplyKey::None,
            std::slice::from_ref(&row),
            std::slice::from_ref(&row),
            &[0.0],
        );
        let one = KindCount {
            rows: 1,
            named: 1,
            hits: 1,
        };
        assert_eq!((counts[0].openings, counts[0].copies), (one, one));
        assert_eq!(counts[0].rest, KindCount::default());
        assert_eq!(counts[0].all, one);
    }

    #[test]
    fn reply_offsets_name_the_rows_past_each_opening_with_the_probes_key() {
        // A prompt and a 5-token reply in windows of 4, so the reply runs into a second window.
        let tokens = [0u32, 10, 11, 20, 21, 22, 23, 24];
        let mask = [0u8, 0, 0, 1, 1, 1, 1, 1];
        let (windows, _) = hs::build_document_windows(&tokens, &mask, 4, 0).unwrap();
        assert!(windows.len() >= 2, "the reply spans two windows");
        let offsets = reply_offsets(&windows, &mask);
        let want: Vec<Vec<(usize, Vec<u32>)>> = {
            let mut want = vec![Vec::new(); windows.len()];
            for row in reply_rows(&windows, &mask) {
                if !row.opening {
                    want[row.window].push((row.offset, reply_key(&row)));
                }
            }
            want
        };
        assert_eq!(offsets, want);
        // The first window sees the turn: the rows after the opening key on [10, 11], their
        // position and the last reply id.
        assert_eq!(offsets[0][0].1, [10, 11, u32::MAX, 1, u32::MAX, 20]);
        // A later window cannot see the turn: its key holds the window's own reply tokens only,
        // the probe's scope (claude-192's REPORT, "the keys see the window").
        let later = &offsets[windows.len() - 1];
        assert!(later.iter().all(|(_, key)| key[0] == u32::MAX));
    }

    #[test]
    fn reply_rows_land_on_the_rows_assemble_scores() {
        // Two documents, so two windows: the DEV reading names a reply row at its window's index
        // times the context plus its offset, which must be the row `assemble` scores.
        let tokens = [0u32, 10, 11, 20, 21, 22, 0, 12, 13, 23, 24];
        let mask = [0u8, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1];
        let (windows, _) =
            hs::build_document_windows(&tokens, &mask, SHEET_CONTEXT, hs::SHEET_BOS_ID).unwrap();
        assert_eq!(windows.len(), 2);
        let refs: Vec<&CausalWindow> = windows.iter().collect();
        let (_, scored, targets) = assemble(&refs);
        let rows = reply_rows(&windows, &mask);
        let named: Vec<u32> = rows
            .iter()
            .map(|row| (row.window * SHEET_CONTEXT + row.offset) as u32)
            .collect();
        assert_eq!(named, scored);
        assert_eq!(
            rows.iter().map(|row| row.target).collect::<Vec<_>>(),
            targets
        );
    }

    #[test]
    fn the_greedy_key_equals_the_corpus_key_at_the_same_row() {
        // BOS, a turn of 3, then the reply 20 21 22.
        let tokens = [0u32, 10, 11, 12, 20, 21, 22];
        let mask = [0u8, 0, 0, 0, 1, 1, 1];
        let (windows, _) = hs::build_document_windows(&tokens, &mask, 16, 0).unwrap();
        let rows: Vec<ReplyRow> = reply_rows(&windows, &mask)
            .into_iter()
            .filter(|row| !row.opening)
            .collect();
        assert_eq!(rows.len(), 2);
        for row in &rows {
            // The greedy step that predicts this row's target sees the prompt and the reply so
            // far; the prompt's last token is window index 3.
            let window = &tokens[..=row.offset];
            assert_eq!(
                greedy_reply_key(window, 3, &row.reply),
                Some(reply_key(row)),
                "offset {}",
                row.offset
            );
        }
        // The opening step has no reply yet, so no reply key.
        assert_eq!(greedy_reply_key(&tokens[..4], 3, &[]), None);
    }

    #[test]
    fn the_reply_table_names_what_the_probe_counts() {
        // BOS, a turn, a reply; another turn with the same start, a reply that differs at its
        // second token; a third.
        let tokens = [
            0u32, 10, 11, 20, 21, 22, 13, 10, 11, 20, 25, 22, 13, 10, 11, 20, 21, 22,
        ];
        let mask = [0u8, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1];
        let (windows, _) = hs::build_document_windows(&tokens, &mask, 32, 0).unwrap();
        let rows = reply_rows(&windows, &mask);
        let probe = &reply_bound_rows(ReplyKey::TurnPosition(1), &rows, &rows, &[0.5])[0];
        let later: Vec<(Vec<u32>, u32)> = rows
            .iter()
            .filter(|row| !row.opening)
            .map(|row| (reply_key(row), row.target))
            .collect();
        let table = crate::flock_read_memory::build_reply_table(&later, 0.5);
        let named = later
            .iter()
            .filter(|(key, _)| table.contains_key(key))
            .count();
        let hits = later
            .iter()
            .filter(|(key, target)| table.get(key) == Some(target))
            .count();
        assert_eq!(
            (named, hits),
            (
                probe.all.named - probe.openings.named,
                probe.all.hits - probe.openings.hits
            )
        );
        assert!(named > 0 && hits < named);
    }

    #[test]
    fn a_config_writes_the_reply_table_key_only_when_on_and_refuses_it_without_the_memory() {
        let mut config = config(0, None);
        let json = String::from_utf8(serde_json::to_vec_pretty(&config).unwrap()).unwrap();
        assert!(!json.contains("turn_memory_reply"), "{json}");
        config.turn_memory_reply = true;
        let json = String::from_utf8(serde_json::to_vec_pretty(&config).unwrap()).unwrap();
        assert!(json.contains("\"turn_memory_reply\": true"), "{json}");
        let missing = std::env::temp_dir().join(format!("claude-193-none-{}", std::process::id()));
        let refused = |config: &LongConfig| {
            start_memory(&missing, config, &[], &Device::Cpu)
                .err()
                .map(|e| e.to_string())
                .unwrap_or_default()
        };
        assert!(refused(&config).contains("--turn-memory-reply needs"));
        config.turn_memory = 6;
        assert!(refused(&config).contains("needs --turn-memory 8"));
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
    fn the_stage_clock_times_a_step_without_changing_its_loss() {
        let device = Device::new_cuda(0).unwrap();
        let model = build_fbs_model(FbsArm::OneAtomActive, 1, &device).unwrap();
        let ids: Vec<u32> = (0..SHEET_CONTEXT as u32).map(|i| (i * 37) % 4096).collect();
        let (scored, targets) = ([10u32, 50, 100, 191], [11u32, 2000, 7, 300]);
        let loss =
            || batch_loss(&model, PointObjective::Cell, &ids, &scored, &targets, 1, false).unwrap();
        let off = loss().to_scalar::<f32>().unwrap();
        stage_clock::start(&device);
        let on = loss();
        on.backward().unwrap();
        stage_clock::mark("backward");
        let stages = stage_clock::finish().unwrap().expect("the clock was on");
        // Events mark the stream and change no kernel: the loss is the same, bit for bit.
        assert_eq!(off.to_bits(), on.to_scalar::<f32>().unwrap().to_bits());
        let names: Vec<&str> = stages.iter().map(|s| s.name.as_str()).collect();
        for want in [
            "embed",
            "layer0.recurrence",
            "layer0.mlp",
            "final_norm",
            "points.select",
            "objective.table_search",
            "objective.hinge",
            "backward",
            "end",
        ] {
            assert!(names.contains(&want), "{names:?} lacks {want}");
        }
        assert!(
            stages
                .iter()
                .all(|s| s.gpu_ms.is_some_and(|ms| ms >= 0.0) && s.host_ms >= 0.0),
            "{stages:?}"
        );
        // Off again: a mark does nothing, and there is nothing to finish.
        stage_clock::mark("after");
        assert!(stage_clock::finish().unwrap().is_none());
    }

    #[test]
    fn the_profile_window_alternates_after_the_warm_up() {
        assert_eq!(profile_window(PROFILE_WARMUP, 0), None);
        assert_eq!(profile_window(PROFILE_WARMUP - 1, 3), None);
        let window: Vec<Option<bool>> = (PROFILE_WARMUP..PROFILE_WARMUP + 7)
            .map(|step| profile_window(step, 3))
            .collect();
        assert_eq!(
            window,
            [
                Some(true),
                Some(false),
                Some(true),
                Some(false),
                Some(true),
                Some(false),
                None
            ]
        );
    }

    fn profile_row(scale: f64, unprofiled_ms: f64) -> ProfileRow {
        let stage = |name: &str, gpu_ms: f64| StageTime {
            name: name.to_string(),
            gpu_ms: Some(gpu_ms * scale),
            host_ms: gpu_ms * scale / 2.0,
        };
        let stages = vec![
            stage("embed", 1.0),
            stage("layer0.recurrence", 6.0),
            stage("points.select", 0.5),
            stage("objective.table_search", 3.0),
            stage("backward", 8.0),
            stage("optimizer.main", 1.5),
            stage("end", 0.0),
        ];
        ProfileRow {
            step: PROFILE_WARMUP,
            loss: 1.0,
            step_ms: unprofiled_ms,
            gpu_span_ms: stages.iter().filter_map(|s| s.gpu_ms).sum(),
            stages,
            next_unprofiled_ms: Some(unprofiled_ms),
        }
    }

    #[test]
    fn the_profile_summary_takes_medians_sums_groups_and_reads_the_bar() {
        // Three steps: each stage's median is the middle step's, at scale 1.
        let summary = summarize_profile(&[
            profile_row(0.9, 21.0),
            profile_row(1.0, 20.0),
            profile_row(1.3, 19.0),
        ]);
        assert_eq!(summary.profiled_steps, 3);
        let gpu: Vec<f64> = summary.stages.iter().map(|s| s.gpu_ms).collect();
        assert_eq!(gpu, [1.0, 6.0, 0.5, 3.0, 8.0, 1.5, 0.0]);
        assert_eq!(summary.stages[1].host_ms, 3.0);
        let group = |g: &str| summary.groups.iter().find(|(n, _)| n == g).unwrap().1;
        assert_eq!(
            [
                group("forward"),
                group("objective"),
                group("backward"),
                group("optimizer"),
                group("end")
            ],
            [7.0, 3.5, 8.0, 1.5, 0.0]
        );
        assert_eq!(
            (summary.stage_sum_ms, summary.median_unprofiled_ms),
            (20.0, 20.0)
        );
        assert!(summary.within_10pct && summary.ratio == 1.0);
        assert_eq!(
            summary.top3,
            ["backward", "layer0.recurrence", "objective.table_search"]
        );
        assert_eq!(
            summary.top3_forward,
            ["layer0.recurrence", "objective.table_search", "embed"]
        );
        assert_eq!(
            (summary.median_profiled_ms, summary.clock_overhead),
            (20.0, 0.0)
        );
        // A clock that slows the profiled steps shows in its overhead, which the sum cannot see.
        let mut slowed = profile_row(1.0, 20.0);
        slowed.step_ms = 22.0;
        let slowed = summarize_profile(&[slowed]);
        assert!(slowed.within_10pct && (slowed.clock_overhead - 0.1).abs() < 1e-12);
        // The stages summing to far more than an unprofiled step, or far less, misses the bar.
        let over = summarize_profile(&[profile_row(1.0, 15.0), profile_row(1.0, 15.0)]);
        assert!(!over.within_10pct && over.ratio > 1.3);
        let under = summarize_profile(&[profile_row(1.0, 30.0)]);
        assert!(!under.within_10pct && under.ratio < 0.7);
    }

    #[test]
    fn a_profiled_run_refuses_a_window_it_cannot_hold_and_a_device_without_cuda() {
        let missing = std::env::temp_dir().join(format!("claude-157-none-{}", std::process::id()));
        let refusal = |profile: usize, steps: usize| -> String {
            let mut config = config(0, None);
            (config.profile, config.steps) = (profile, steps);
            match run_long(&missing, &missing.join("out"), &config, &Device::Cpu) {
                Err(e) => e.to_string(),
                Ok(()) => "ran".to_string(),
            }
        };
        assert!(refusal(usize::MAX, 72).contains("too large"));
        assert!(refusal(3, 13).contains("needs at least 14 steps"));
        assert!(refusal(3, 14).contains("needs a CUDA device"));
        assert!(!missing.exists());
    }

    #[test]
    fn a_config_without_the_profile_writes_no_profile_key() {
        let mut config = config(0, None);
        let json = String::from_utf8(serde_json::to_vec_pretty(&config).unwrap()).unwrap();
        assert!(!json.contains("profile"), "{json}");
        config.profile = 3;
        let json = String::from_utf8(serde_json::to_vec_pretty(&config).unwrap()).unwrap();
        assert!(json.contains("\"profile\": 3"), "{json}");
    }

    #[test]
    fn a_config_without_the_turn_memory_writes_neither_of_its_keys() {
        let mut config = config(0, None);
        let json = String::from_utf8(serde_json::to_vec_pretty(&config).unwrap()).unwrap();
        assert!(!json.contains("turn_memory"), "{json}");
        config.turn_memory = 8;
        let json = String::from_utf8(serde_json::to_vec_pretty(&config).unwrap()).unwrap();
        assert!(
            json.contains("\"turn_memory\": 8") && !json.contains("turn_memory_empty"),
            "{json}"
        );
        config.turn_memory_empty = true;
        let json = String::from_utf8(serde_json::to_vec_pretty(&config).unwrap()).unwrap();
        assert!(json.contains("\"turn_memory_empty\": true"), "{json}");
    }

    #[test]
    fn the_empty_memory_control_refuses_a_run_with_no_memory() {
        let mut config = config(0, None);
        config.turn_memory_empty = true;
        let missing = std::env::temp_dir().join(format!("claude-161-none-{}", std::process::id()));
        let refused = start_memory(&missing, &config, &[], &Device::Cpu)
            .err()
            .map(|e| e.to_string());
        assert!(refused.is_some_and(|e| e.contains("--turn-memory-empty needs")));
        config.turn_memory_empty = false;
        let (memory, offsets, replies) =
            start_memory(&missing, &config, &[], &Device::Cpu).unwrap();
        assert!(memory.is_none() && offsets.is_empty() && replies.is_empty());
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

    fn pick(position: usize, target: u32, nearest: u32, writer: u32, opens: bool, copies: bool) -> RowPick {
        RowPick { position, target, nearest, writer, opens, copies }
    }

    /// claude-205: the per-row picks fold into `nearest_reading`'s counts exactly as its loop
    /// counted them, kind by kind.
    #[test]
    fn row_picks_fold_into_the_nearest_reading() {
        let picks = [
            pick(3, 7, 7, 7, true, false),  // an opening hit, the writer too
            pick(4, 7, 9, 7, true, true),   // an opening that copies, missed by nearest
            pick(5, 8, 8, 1, true, false),  // an opening hit, the writer missing
            pick(6, 2, 2, 2, false, true),  // a copied hit
            pick(7, 5, 5, 4, false, false), // a rest hit
            pick(8, 6, 1, 6, false, false), // a rest miss the writer gets
        ];
        let (reading, written) = fold_picks(&picks);
        assert_eq!(reading.hits, 4);
        assert_eq!((reading.opening_rows, reading.opening_hits), (3, 2));
        assert_eq!((reading.copied_rows, reading.copied_hits), (2, 1));
        assert_eq!((reading.rest_rows, reading.rest_hits), (2, 1));
        assert_eq!(reading.commonest_opening_rows, 2, "target 7 opens twice");
        assert_eq!(reading.opening_distinct_picks, 3, "7, 9 and 8");
        assert_eq!((written.hits, written.opening_hits), (4, 2));
        assert_eq!(written.opening_distinct_picks, 2, "7 and 1");
    }

    /// claude-205: the band's floor is `⌈2·√n⌉` exactly, at and between squares.
    #[test]
    fn the_band_floor_is_twice_the_root_rounded_up() {
        for (n, want) in [(0, 0), (1, 2), (4, 4), (5, 5), (6, 5), (150, 25), (3_438, 118)] {
            assert_eq!(two_root_ceil(n), want, "n = {n}");
            let k = two_root_ceil(n);
            assert!(k * k >= 4 * n && (k == 0 || (k - 1) * (k - 1) < 4 * n));
        }
    }

    /// claude-205: a seed is ahead or behind only beyond `max(⌈2·√(b+c)⌉, spread)`, and at the
    /// band itself it is inside.
    #[test]
    fn a_seed_is_ahead_or_behind_only_beyond_the_band() {
        // b + c = 150, floor 25.
        assert_eq!(seed_cell(100, 50, 10), (25, OverturnCell::Ahead));
        assert_eq!(seed_cell(50, 100, 10), (25, OverturnCell::Behind));
        assert_eq!(seed_cell(85, 65, 10), (25, OverturnCell::Inside));
        // The spread, when wider, is the band.
        assert_eq!(seed_cell(100, 50, 60), (60, OverturnCell::Inside));
        // b + c = 135, floor 24: b − c = 25 is beyond it, but not beyond a spread of 25.
        assert_eq!(seed_cell(80, 55, 0), (24, OverturnCell::Ahead));
        assert_eq!(seed_cell(80, 55, 25), (25, OverturnCell::Inside));
        assert_eq!(seed_cell(81, 55, 25), (25, OverturnCell::Ahead));
        // The same edges on the behind side: −25 is inside a band of 25, −26 is behind it.
        assert_eq!(seed_cell(55, 80, 25), (25, OverturnCell::Inside));
        assert_eq!(seed_cell(55, 81, 25), (25, OverturnCell::Behind));
    }

    /// claude-205: the verdict is ahead or behind only when both seeds agree beyond their bands.
    #[test]
    fn the_verdict_needs_every_seed() {
        use OverturnCell::{Ahead, Behind, Inside};
        assert_eq!(overturn_verdict(&[Ahead, Ahead]), "ahead");
        assert_eq!(overturn_verdict(&[Behind, Behind]), "behind");
        assert_eq!(overturn_verdict(&[Ahead, Behind]), "mixed");
        assert_eq!(overturn_verdict(&[Ahead, Inside]), "mixed");
        assert_eq!(overturn_verdict(&[Inside, Inside]), "mixed");
        assert_eq!(overturn_verdict(&[]), "mixed");
    }

    fn scored(nearest: &[u32]) -> ScoredRows {
        ScoredRows {
            split: "dev".into(),
            memory: true,
            positions: vec![10, 11, 12, 13, 14],
            targets: vec![1, 2, 3, 4, 5],
            nearest: nearest.to_vec(),
            writer: nearest.to_vec(),
        }
    }

    /// claude-205: the pairing counts hits and discordant rows on the decision rows only, and
    /// refuses arms that score other rows, or rows that miss a decision row.
    #[test]
    fn pairing_counts_discordant_rows_on_the_decision_rows_only() {
        let rows: std::collections::BTreeSet<usize> = [10, 11, 12, 13].into_iter().collect();
        // a gets rows 10, 11, 13, 14; b gets 10, 12, 14. Row 14 is not a decision row. So on the
        // decision rows a has 3 hits and b 2, with 2 rows only a gets (11, 13) and 1 only b gets
        // (12): an ordered result, so a swap of the arms or of the counts fails.
        let (a, b) = (scored(&[1, 2, 0, 4, 5]), scored(&[1, 0, 3, 0, 5]));
        assert_eq!(paired(&a, &b, &rows).unwrap(), (3, 2, 2, 1));
        assert_eq!(paired(&b, &a, &rows).unwrap(), (2, 3, 1, 2));
        assert_eq!(pick_differences(&a, &b, &rows), 3, "rows 11, 12 and 13");
        let mut short = a.clone();
        short.nearest.pop();
        assert!(paired(&short, &b, &rows).is_err(), "a short list is refused, not read past");
        assert_eq!(pick_differences(&a, &a, &rows), 0);
        let mut other = b.clone();
        other.positions[4] = 99;
        assert!(paired(&a, &other, &rows).is_err(), "different rows");
        let missing: std::collections::BTreeSet<usize> = [10, 50].into_iter().collect();
        assert!(paired(&a, &b, &missing).is_err(), "a decision row neither scores");
    }

    /// claude-205: replies are cut after their first end-of-turn id; only complete ones can be
    /// verbatim; new n-grams are those no TRAIN reply holds; distinct and repeat rates count
    /// within the cut replies.
    #[test]
    fn generation_measures_count_against_trains_replies() {
        const EOS: u32 = 2;
        let train = vec![vec![10, 11, 12, 13, EOS], vec![20, 21, 22, 23, 24, EOS]];
        let replies = vec![
            vec![10, 11, 12, 13, EOS, 99, 99], // verbatim, cut after EOS
            vec![20, 21, 22, 23, 30, EOS],     // complete, not verbatim
            vec![5, 5, 5, 5, 5, 5],            // never ends
        ];
        let g = generation(&replies, &train, EOS);
        assert_eq!((g.replies, g.complete, g.verbatim_train), (3, 2, 1));
        assert_eq!(g.new_ngram_share.len(), 5);
        // 4-grams: reply 0 has 2, both in TRAIN; reply 1 has 3, the first in TRAIN; reply 2 has
        // 3, none in TRAIN. New: 2 + 3 of 8.
        assert!((g.new_ngram_share[0] - 5.0 / 8.0).abs() < 1e-12);
        // Unigrams: 5 + 6 + 6 = 17 tokens; distinct {10..13, 2, 20..23, 30, 5} = 11.
        assert!((g.distinct_1 - 11.0 / 17.0).abs() < 1e-12);
        // Bigrams: 4 + 5 + 5 = 14; distinct: 4 + 5 + 1 ((5, 5)) = 10.
        assert!((g.distinct_2 - 10.0 / 14.0).abs() < 1e-12);
        // Repeated 4-grams: only reply 2's second and third repeat its first, of 8.
        assert!((g.repeated_4gram_rate - 2.0 / 8.0).abs() < 1e-12);
        let none = generation(&[], &train, EOS);
        assert_eq!((none.replies, none.distinct_1, none.repeated_4gram_rate), (0, 0.0, 0.0));
    }

    /// claude-205: a scorer's output is never replaced.
    #[test]
    fn a_scored_file_is_never_replaced() {
        let dir = std::env::temp_dir().join(format!("claude-205-new-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("rows-dev-memory.json");
        write_new(&path, b"first").unwrap();
        assert!(write_new(&path, b"second").is_err());
        assert_eq!(std::fs::read(&path).unwrap(), b"first");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// claude-205: the reference reply runs from the first scored offset, and stops at the first
    /// gap in the offsets and at its length.
    #[test]
    fn the_reference_reply_stops_at_a_gap() {
        let window = CausalWindow {
            doc_id: 0,
            input_ids: vec![0; 12],
            scored_offsets: vec![3, 4, 5, 8, 9],
            scored_targets: vec![31, 41, 51, 81, 91],
            scored_corpus_positions: vec![4, 5, 6, 9, 10],
        };
        assert_eq!(reference_reply(&window, 10), vec![31, 41, 51]);
        assert_eq!(reference_reply(&window, 2), vec![31, 41]);
    }

    /// claude-207 (protocol I-B): no window is dropped at 0, every window at 1,000, and about
    /// half at 500; the draw is a pure function of the seed and the window's place, so it never
    /// touches the batch order, and seeds differ.
    #[test]
    fn memory_dropout_drops_the_share_it_is_given() {
        let count = |seed: u64, permille: u32| {
            (0..100)
                .flat_map(|step| (0..16).map(move |b| (step, b)))
                .filter(|&(step, b)| memory_dropped(seed, step, b, 16, permille))
                .count()
        };
        assert_eq!(count(1, 0), 0);
        assert_eq!(count(1, MEMORY_DROPOUT_MAX), 1_600);
        let half = count(1, 500);
        // 1,600 windows at p = 0.5: σ = 20, so 4σ either side.
        assert!((720..=880).contains(&half), "{half}");
        assert_eq!(count(1, 500), half, "the same draw every time");
        assert_ne!(
            (0..64).map(|b| memory_dropped(1, 3, b, 64, 500)).collect::<Vec<_>>(),
            (0..64).map(|b| memory_dropped(2, 3, b, 64, 500)).collect::<Vec<_>>(),
            "another seed draws another pattern"
        );
        let low = count(1, 100);
        assert!((100..=220).contains(&low), "{low}");
    }

    /// claude-207 (protocol I-C): the frozen arm keeps the routing queries out of the main
    /// optimizer, the other arms train them, and no arm trains the decay rates there.
    #[test]
    fn only_the_frozen_arm_keeps_the_routing_queries_out_of_the_optimizer() {
        let query = "layers.0.attn.q.spiralcore_fbs";
        for arm in [FbsArm::OneAtomActive, FbsArm::IdentityTransport] {
            assert!(main_optimizer_updates(query, arm), "{arm:?}");
            assert!(main_optimizer_updates("embedding.weight", arm));
            assert!(!main_optimizer_updates("layers.0.rec.decay", arm));
        }
        assert!(!main_optimizer_updates(query, FbsArm::FrozenQueries));
        assert!(main_optimizer_updates("embedding.weight", FbsArm::FrozenQueries));
        assert!(!main_optimizer_updates("layers.0.rec.decay", FbsArm::FrozenQueries));
    }

    /// claude-207: `--fbs-arm` takes the three protocol arms by name, and nothing else.
    #[test]
    fn the_transport_arm_is_named_on_the_command_line() {
        assert_eq!(fbs_arm_named("one-atom"), Some(FbsArm::OneAtomActive));
        assert_eq!(fbs_arm_named("identity"), Some(FbsArm::IdentityTransport));
        assert_eq!(fbs_arm_named("frozen"), Some(FbsArm::FrozenQueries));
        assert_eq!(fbs_arm_named("two-atom"), None);
        assert_eq!(fbs_arm_named("Identity"), None);
    }

    /// claude-207: a config at the defaults writes neither new key, so a run without them writes
    /// the same `config.json` as before; set, both are written.
    #[test]
    fn a_config_writes_dropout_and_arm_only_when_set() {
        let json = String::from_utf8(serde_json::to_vec_pretty(&config(0, None)).unwrap()).unwrap();
        assert!(!json.contains("turn_memory_dropout"), "{json}");
        assert!(!json.contains("fbs_arm"), "{json}");
        let mut set = config(0, None);
        set.turn_memory_dropout = 500;
        set.fbs_arm = FbsArm::FrozenQueries;
        let json = serde_json::to_string(&set).unwrap();
        assert!(json.contains("\"turn_memory_dropout\":500"), "{json}");
        assert!(json.contains("\"fbs_arm\":\"FrozenQueries\""), "{json}");
    }

    /// claude-207: scoring refuses an arm other than the run's, read from the run's
    /// `config.json` beside a run folder or two levels above a `models/step-N` folder; a model's
    /// own `config.json` (no `steps`) and a missing run config are no refusal.
    #[test]
    fn scoring_refuses_another_arm_than_the_runs() {
        let dir = std::env::temp_dir().join(format!("claude-207-arm-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let step = dir.join("run").join("models").join("step-000256");
        std::fs::create_dir_all(&step).unwrap();
        std::fs::write(step.join("config.json"), br#"{"width":256}"#).unwrap();
        let run = dir.join("run");
        // No run config yet: scored as asked.
        assert!(check_run_arm(&step, FbsArm::IdentityTransport).is_ok());
        std::fs::write(run.join("config.json"), br#"{"steps":256,"fbs_arm":"IdentityTransport"}"#)
            .unwrap();
        assert!(check_run_arm(&step, FbsArm::IdentityTransport).is_ok());
        assert!(check_run_arm(&run, FbsArm::IdentityTransport).is_ok());
        assert!(check_run_arm(&step, FbsArm::OneAtomActive).is_err());
        assert!(check_run_arm(&run, FbsArm::FrozenQueries).is_err());
        // A run config without the key trained the default arm.
        std::fs::write(run.join("config.json"), br#"{"steps":256}"#).unwrap();
        assert!(check_run_arm(&step, FbsArm::OneAtomActive).is_ok());
        assert!(check_run_arm(&step, FbsArm::FrozenQueries).is_err());
        // A run config cut short is refused, not skipped.
        std::fs::write(run.join("config.json"), br#"{"steps":256,"fbs_arm":"Ident"#).unwrap();
        assert!(check_run_arm(&step, FbsArm::OneAtomActive).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// claude-208 (protocol I-E): the copied weights are the integer Flock selector's harmonic
    /// table, read from its source, so a change there fails here.
    #[test]
    fn the_flock_rank_weights_are_the_integer_selectors_table() {
        let source = include_str!("../../uor-r4-integer/src/stack/flock.rs");
        let start = source
            .find("pub const RAW_RANK_WEIGHTS_Q16: [u16; 129] = [")
            .expect("the table is declared");
        let body = &source[start..];
        let body = &body[body.find('[').unwrap() + 1..];
        let body = &body[body.find('[').unwrap() + 1..body.find("];").unwrap()];
        let table: Vec<u16> = body
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.parse().unwrap())
            .collect();
        assert_eq!(table.len(), 129);
        assert_eq!(&table[..FLOCK_DECODE_K], &FLOCK_RANK_WEIGHTS_Q16[..]);
        // And they are 1/(i+1) in Q16: the first is saturated at u16::MAX.
        for (i, &w) in FLOCK_RANK_WEIGHTS_Q16.iter().enumerate().skip(1) {
            assert_eq!(u32::from(w), 65_536 / (i as u32 + 1), "rank {i}");
        }
    }

    /// claude-208 (protocol I-E): at k = 1 the Flock rank rule is greedy, and over 10^6 seeded
    /// draws each rank's frequency is within 0.002 of `W_i / H`.
    #[test]
    fn the_flock_rank_draw_follows_the_harmonic_weights() {
        for index in 0..1_000 {
            assert_eq!(flock_rank_draw(7, index, 1), 0);
        }
        let draws = 1_000_000u64;
        let mut counts = [0u64; FLOCK_DECODE_K];
        for index in 0..draws {
            counts[flock_rank_draw(1, index, FLOCK_DECODE_K)] += 1;
        }
        let total: f64 = FLOCK_RANK_WEIGHTS_Q16.iter().map(|&w| f64::from(w)).sum();
        for (i, &count) in counts.iter().enumerate() {
            let want = f64::from(FLOCK_RANK_WEIGHTS_Q16[i]) / total;
            let got = count as f64 / draws as f64;
            assert!((got - want).abs() < 0.002, "rank {i}: {got} against {want}");
        }
        // Deterministic for a seed and index, and seeds differ.
        assert_eq!(flock_rank_draw(3, 42, 16), flock_rank_draw(3, 42, 16));
        assert_ne!(
            (0..64).map(|i| flock_rank_draw(1, i, 16)).collect::<Vec<_>>(),
            (0..64).map(|i| flock_rank_draw(2, i, 16)).collect::<Vec<_>>()
        );
    }
    /// claude-211 (protocol I-D): the longest-suffix control predicts TRAIN's commonest next
    /// token after the longest context suffix TRAIN holds, ties to the smaller id; a BOS anchors
    /// a suffix at a document's start, and no suffix reaches back past one; and a token TRAIN
    /// never shows is a miss.
    #[test]
    fn the_longest_suffix_control_reads_the_longest_match() {
        // BOS 5 6 7 8 | BOS 9 6 7 3 | BOS 6 7 4 | BOS 6 7 4
        let train = [0u32, 5, 6, 7, 8, 0, 9, 6, 7, 3, 0, 6, 7, 4, 0, 6, 7, 4];
        let contexts: Vec<&[u32]> = vec![
            &[1, 5, 6, 7],  // "5 6 7" occurs once, followed by 8 ("6 7" alone gives 4)
            &[1, 9, 6, 7],  // "9 6 7" once, followed by 3
            &[2, 2, 6, 7],  // only "6 7": 8, 3, 4, 4, so 4
            &[2, 2, 2, 7],  // only "7": the same as "6 7": 4
            &[2, 2, 2, 42], // 42 never occurs: a miss
        ];
        assert_eq!(
            longest_suffix_predictions(&train, &contexts, 8),
            vec![8, 3, 4, 4, CONTROL_MISS]
        );
        // A tie goes to the smaller id: "1" is followed once by 2 and once by 3.
        let tied = [0u32, 1, 2, 0, 1, 3];
        assert_eq!(longest_suffix_predictions(&tied, &[&[1]], 4), vec![2]);
        // BOS anchors: "6 7" is followed by 4, 3, 3, but "BOS 6 7" only by 4.
        let anchored = [0u32, 6, 7, 4, 0, 5, 6, 7, 3, 0, 9, 6, 7, 3];
        assert_eq!(
            longest_suffix_predictions(&anchored, &[&[0, 6, 7], &[1, 6, 7]], 8),
            vec![4, 3]
        );
        // Never past a BOS: "8 BOS 9" would give 1, but "BOS 9" is followed by 1, 2, 2.
        let crossed = [0u32, 8, 0, 9, 1, 0, 9, 2, 0, 9, 2];
        assert_eq!(
            longest_suffix_predictions(&crossed, &[&[8, 0, 9]], 8),
            vec![2]
        );
    }

    /// claude-211 (protocol I-E, its GPU half): on a model, the Flock rank rule at K = 1
    /// decodes exactly as greedy, through the same top-K sort and gather the served rule runs;
    /// and the served K = 16 leaves greedy's path, so its draws do run.
    #[cfg(feature = "cuda")]
    #[test]
    fn the_flock_rank_rule_at_k1_decodes_as_greedy_on_a_model() {
        let device = Device::new_cuda(0).unwrap();
        let model = build_fbs_model(FbsArm::OneAtomActive, 1, &device).unwrap();
        let (mut decoded, mut left) = (0, false);
        for sample in 0..4usize {
            let shift = sample as u32 * 101;
            let prompt: Vec<u32> = (1..=24u32).map(|i| (i * 37 + shift) % 4096).collect();
            let decode = |rule| {
                decode_reply_with_mode(
                    &model,
                    PointObjective::Cell,
                    &prompt,
                    16,
                    1,
                    rule,
                    sample,
                    None,
                    ReadScope::Full,
                    &[],
                    false,
                )
                .unwrap()
                .tokens
            };
            let greedy = decode(DecodeRule::Greedy);
            assert_eq!(
                decode(DecodeRule::FlockRank {
                    k: 1,
                    keep_end: false
                }),
                greedy,
                "sample {sample}"
            );
            left |= decode(DecodeRule::FLOCK) != greedy;
            decoded += greedy.len();
        }
        assert!(decoded > 0, "every reply was empty, so the check held vacuously");
        assert!(left, "K = 16 never left greedy's path: its draws did not run");
    }

    /// claude-217 (its GPU half): on a model, the cycle step at K = 1 decodes exactly as greedy;
    /// at K = 16 it follows greedy until its first step off, where greedy's token would close a
    /// 4-gram the reply already holds; and its reply repeats no 4-gram unless a step was
    /// exhausted. It records one mark per step.
    #[cfg(feature = "cuda")]
    #[test]
    fn the_cycle_step_follows_greedy_until_it_steps_off_a_cycle_on_a_model() {
        let device = Device::new_cuda(0).unwrap();
        let model = build_fbs_model(FbsArm::OneAtomActive, 1, &device).unwrap();
        let mut decoded = 0;
        for sample in 0..4usize {
            let shift = sample as u32 * 101;
            let prompt: Vec<u32> = (1..=24u32).map(|i| (i * 37 + shift) % 4096).collect();
            let decode = |rule| {
                decode_reply_with_mode(
                    &model,
                    PointObjective::Cell,
                    &prompt,
                    32,
                    1,
                    rule,
                    sample,
                    None,
                    ReadScope::Full,
                    &[],
                    false,
                )
                .unwrap()
            };
            let greedy = decode(DecodeRule::Greedy).tokens;
            assert_eq!(decode(DecodeRule::CycleStep { k: 1, n: 4 }).tokens, greedy, "sample {sample}");
            assert_eq!(decode(DecodeRule::CycleStep { k: 1, n: 2 }).tokens, greedy, "sample {sample}, n = 2");
            let step = decode(DecodeRule::CYCLE_STEP);
            // One mark per step, a final step that picked BOS (and ended the reply) included.
            assert!(step.stepped.len() - step.tokens.len() <= 1, "one mark per step");
            let off = step.stepped.iter().position(|&m| m == STEP_OFF).unwrap_or(step.tokens.len());
            assert_eq!(step.tokens[..off], greedy[..off.min(greedy.len())], "sample {sample}");
            if off < step.tokens.len() && off < greedy.len() {
                let (_, mark) = cycle_step_pick(&greedy[..off], &[greedy[off]], 4).unwrap();
                assert_eq!(mark, STEP_EXHAUSTED, "greedy's token at {off} closes no cycle");
            }
            if !step.stepped.contains(&STEP_EXHAUSTED) {
                let mut seen = std::collections::HashSet::new();
                assert!(step.tokens.windows(4).all(|w| seen.insert(w)), "a 4-gram repeats");
            }
            // Wherever greedy repeats a 4-gram, the rule must have left the nearest token at
            // least once (FIT round 2): the step-off path ran on the model.
            let mut seen = std::collections::HashSet::new();
            if !greedy.windows(4).all(|w| seen.insert(w)) {
                assert!(step.stepped.iter().any(|&m| m != STEP_NEAREST), "greedy loops, the rule never stepped");
            }
            decoded += greedy.len();
        }
        assert!(decoded > 0, "every reply was empty, so the check held vacuously");
    }

    /// claude-218: a forced opening is the reply's first token, and the rest is greedy from the
    /// prompt and that token; and the opening points are exactly the trunk's points at the
    /// scored rows that open a reply, as `row_picks` sees them.
    /// claude-220: at an opening the memory's exact key misses, the served key names the reply's
    /// first token from the decode's own point by the table's nearest TRAIN opening, and the
    /// reply continues as a forced one with that token; where the exact key fires, the decode
    /// is greedy's; and only the missed opening counts a search.
    #[cfg(feature = "cuda")]
    #[test]
    fn the_served_key_names_a_missed_opening_from_the_decodes_own_point() {
        use crate::flock_read_opening::{opening_slots, OpeningAnchor};
        let device = Device::new_cuda(0).unwrap();
        let model = build_fbs_model(FbsArm::OneAtomActive, 1, &device).unwrap();
        let a: Vec<u32> = std::iter::once(hs::SHEET_BOS_ID).chain((1..=24u32).map(|i| (i * 37) % 4096 + 2)).collect();
        let b: Vec<u32> = std::iter::once(hs::SHEET_BOS_ID).chain((1..=24u32).map(|i| (i * 53) % 4096 + 2)).collect();
        let point_of = |p: &[u32]| {
            crate::flock_read::points(&model, p, 1, p.len()).unwrap().narrow(0, p.len() - 1, 1).unwrap()
        };
        let decode = |p: &[u32], n: usize| {
            decode_reply_with_mode(
                &model,
                PointObjective::Cell,
                p,
                n,
                1,
                DecodeRule::Greedy,
                0,
                None,
                ReadScope::Full,
                &[],
                false,
            )
            .unwrap()
        };
        // A memory whose table holds a's own turn key, with a zero weight, so it moves nothing.
        let slots = opening_slots(&a, 8, OpeningAnchor::Turn);
        let key: Vec<u32> = slots[a.len() - 1].iter().map(|&s| a[s]).collect();
        let pb = point_of(&b);
        let width = pb.dim(1).unwrap();
        let memory = crate::flock_read_memory::TurnMemory::new(8, std::collections::HashMap::from([(key, 5u32)]), width, &device).unwrap();
        crate::flock_read_memory::set_turn_memory(Some(memory));
        let greedy_a = decode(&a, 12).tokens;
        // The table: b's own opening point, target 77, and a far point, target 88.
        let points = Tensor::cat(&[pb.clone(), (&pb + 100.0).unwrap()], 0).unwrap();
        set_served_key(Some(ServedKey { points, targets: std::sync::Arc::new(vec![77, 88]) }));
        take_served_searches();
        let served_b = decode(&b, 12);
        assert_eq!((served_b.tokens[0], served_b.served.map(|s| s.0)), (77, Some(77)), "the missed opening takes the nearest's target");
        assert_eq!(served_b.served.map(|s| s.1), Some(0.0), "the decode's own point is the table's row, bit for bit");
        let served_a = decode(&a, 12);
        assert_eq!((served_a.tokens.as_slice(), served_a.served.is_none()), (greedy_a.as_slice(), true), "the exact key fires: greedy");
        assert_eq!(take_served_searches().0, 1, "only the missed opening is searched");
        set_served_key(None);
        set_forced_opening(Some(77));
        let forced_b = decode(&b, 12).tokens;
        set_forced_opening(None);
        assert_eq!(served_b.tokens, forced_b, "the rest decodes as the forced reply with that token");
        crate::flock_read_memory::set_turn_memory(None);
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn a_forced_opening_continues_greedy_and_the_opening_points_are_the_rows() {
        let device = Device::new_cuda(0).unwrap();
        let model = build_fbs_model(FbsArm::OneAtomActive, 1, &device).unwrap();
        let prompt: Vec<u32> = (1..=24u32).map(|i| (i * 37) % 4096).collect();
        let decode = |prompt: &[u32], n| {
            decode_reply_with_mode(
                &model,
                PointObjective::Cell,
                prompt,
                n,
                1,
                DecodeRule::Greedy,
                0,
                None,
                ReadScope::Full,
                &[],
                false,
            )
            .unwrap()
            .tokens
        };
        set_forced_opening(Some(77));
        let forced = decode(&prompt, 16);
        set_forced_opening(None);
        assert_eq!(forced[0], 77, "the forced token opens the reply");
        let extended: Vec<u32> = prompt.iter().copied().chain([77]).collect();
        assert_eq!(forced[1..], decode(&extended, 15)[..], "the rest is greedy from the prompt and it");
        assert_ne!(decode(&prompt, 16).first(), None, "the unforced reply decodes");
        // Two turns: BOS, a prompt, a reply, a prompt, a reply.
        let mut tokens = vec![hs::SHEET_BOS_ID];
        let mut mask = vec![0u8];
        for (ids, flag) in [(10..20u32, 0u8), (20..26, 1), (26..30, 0), (30..34, 1)] {
            for t in ids {
                tokens.push(t);
                mask.push(flag);
            }
        }
        let (windows, _) =
            hs::build_document_windows(&tokens, &mask, SHEET_CONTEXT, hs::SHEET_BOS_ID).unwrap();
        let (points, targets, positions) =
            opening_points(&model, &windows, &mask, 1, 0, false).unwrap();
        assert_eq!(positions, [11, 21]);
        assert_eq!(targets, [20, 30]);
        let mut seen = Vec::new();
        row_picks(&model, &windows, &mask, 1, 0, false, None, |query, opens| {
            let rows: Vec<u32> = (0..opens.len() as u32).filter(|&i| opens[i as usize]).collect();
            let idx = Tensor::from_vec(rows.clone(), rows.len(), query.device())?;
            seen.push(query.index_select(&idx, 0)?);
            Ok(())
        })
        .unwrap();
        let seen = Tensor::cat(&seen, 0).unwrap();
        let gap = (points - seen).unwrap().abs().unwrap().max_all().unwrap().to_scalar::<f32>().unwrap();
        assert_eq!(gap, 0.0, "the opening points are the rows' own points");
    }

    /// claude-251 (Done-when 3, on the GPU): side F's trace of a small trunk with both Hamming
    /// reads. Tracing changes no pick, and the trace holds the one window named, in R's order and
    /// lengths: six layers, the stream read after layer 2 only, the readout read, no memory, and a
    /// nearest at each scored offset. Each recorded entry is the kernel's bucket of its recorded
    /// score (`rate_select` returns that entry's λ); each code is `pack_signs` of its recorded
    /// coordinates; each kept set is in R's order; each pick is the kernel's argmax for its
    /// recorded query.
    #[cfg(feature = "cuda")]
    #[test]
    fn a_traced_forward_records_the_kernels_choices_in_rs_order() {
        use crate::flock_read_hamming::{pack_signs, set_hamming_reads, HammingReads, HAMMING_KEPT};
        use crate::geometric_stack::{rate_index, rate_select, RATE_TABLE};
        let device = Device::new_cuda(0).unwrap();
        let model = build_fbs_model(FbsArm::OneAtomActive, 1, &device).unwrap();
        set_hamming_reads(Some(HammingReads::new(4, hs::SHEET_WIDTH, &device).unwrap()));
        // Two documents, each BOS, a prompt, a reply, a prompt, a reply.
        let (mut tokens, mut mask) = (Vec::new(), Vec::new());
        for doc in 0..2u32 {
            tokens.push(hs::SHEET_BOS_ID);
            mask.push(0u8);
            for (ids, flag) in [(10..20u32, 0u8), (20..26, 1), (26..30, 0), (30..34, 1)] {
                for t in ids {
                    tokens.push(t + 40 * doc);
                    mask.push(flag);
                }
            }
        }
        let (windows, _) =
            hs::build_document_windows(&tokens, &mask, SHEET_CONTEXT, hs::SHEET_BOS_ID).unwrap();
        assert_eq!(windows.len(), 2);
        let picks = |trace: Option<&mut FTrace>| {
            row_picks(&model, &windows, &mask, 1, 0, false, trace, |_, _| Ok(())).unwrap()
        };
        let plain = picks(None);
        let named = &windows[1];
        let mut trace = FTrace::new(&[named.scored_corpus_positions[0]]).unwrap();
        assert_eq!(picks(Some(&mut trace)), plain, "tracing changes no pick");
        let host_table = trace.table.clone().unwrap();
        let trace = trace.finish().unwrap();
        set_hamming_reads(None);
        assert_eq!((trace.layers, trace.heads), (6, 4));
        let [w] = trace.windows.as_slice() else { panic!("one window is traced") };
        let last = *named.scored_offsets.last().unwrap();
        assert_eq!((w.position, w.reply_start), (named.scored_corpus_positions[0], named.scored_offsets[0]));
        assert_eq!((w.ids.as_slice(), w.steps.len()), (&named.input_ids[..=last], last + 1));
        let (mut scores, mut entries) = (Vec::new(), Vec::new());
        let (mut queries, mut picked) = (Vec::new(), Vec::new());
        let mut unsigned = false;
        for (t, step) in w.steps.iter().enumerate() {
            assert_eq!(step.layers.len(), 6);
            for (l, layer) in step.layers.iter().enumerate() {
                assert_eq!((layer.score_f32.len(), layer.entry.len()), (trace.lanes, trace.lanes));
                for (&bits, &entry) in layer.score_f32.iter().zip(&layer.entry) {
                    assert_eq!(entry as usize, rate_index(f32::from_bits(bits)));
                    scores.push(f32::from_bits(bits));
                    entries.push(entry as usize);
                }
                assert_eq!(layer.read.as_ref().map(|r| r.depth), (l == 1).then_some(2));
            }
            let readout = step.readout.as_ref().map(|r| r.depth);
            assert_eq!((readout, step.memory.is_none()), (Some(6), true));
            let reads = step.layers[1].read.iter().chain(&step.readout);
            for head in reads.flat_map(|r| &r.heads) {
                let signs = |bits: &[u32]| pack_signs(&bits.iter().map(|&b| f32::from_bits(b)).collect::<Vec<_>>());
                assert_eq!((signs(&head.query_f32), signs(&head.key_f32)), (vec![head.query], vec![head.key]));
                assert_eq!(head.kept.len(), t.saturating_sub(1).min(HAMMING_KEPT), "row {t}");
                // The coordinates the sign read, not its ±1 codes.
                unsigned |= head.query_f32.iter().chain(&head.key_f32).any(|&b| f32::from_bits(b).abs() != 1.0);
            }
            assert_eq!(step.nearest.is_some(), named.scored_offsets.contains(&t));
            if let Some(nearest) = &step.nearest {
                assert_ne!(nearest.runner_up, nearest.pick);
                queries.extend(nearest.query_f32.iter().map(|&b| f32::from_bits(b)));
                picked.push(nearest.pick);
            }
        }
        assert!(unsigned, "the trace records the coordinates, not the codes");
        let mut distinct = entries.clone();
        distinct.sort();
        distinct.dedup();
        assert!(distinct.len() > 1, "the window's scores fall in more than one bucket");
        // Each kept set, as the kernel chose it, is the 16 positions 1 ≤ s < t nearest the row's
        // query word by Hamming distance to their key words, nearest first, ties to the earlier
        // position: so the recorded words are the ones the kernel chose by. Every head of both reads.
        let read = |s: &DecisionStep, r: usize| if r == 0 { s.layers[1].read.clone() } else { s.readout.clone() };
        for r in 0..2 {
            for h in 0..4 {
                let keys: Vec<u64> = w.steps.iter().map(|s| read(s, r).unwrap().heads[h].key).collect();
                for (t, step) in w.steps.iter().enumerate() {
                    let head = &read(step, r).unwrap().heads[h];
                    let mut nearest: Vec<u32> = (1..t as u32).collect();
                    nearest.sort_by_key(|&s| ((head.query ^ keys[s as usize]).count_ones(), s));
                    nearest.truncate(HAMMING_KEPT);
                    assert_eq!(head.kept, nearest, "read {r}, head {h}, row {t}");
                }
            }
        }
        let lambdas = rate_select(&Tensor::from_vec(scores.clone(), scores.len(), &device).unwrap())
            .unwrap()
            .to_vec2::<f32>()
            .unwrap();
        for (lambda, &entry) in lambdas.iter().zip(&entries) {
            assert_eq!(lambda[0].to_bits(), RATE_TABLE[entry].to_bits(), "an entry the kernel did not choose");
        }
        let width = queries.len() / picked.len();
        let query = Tensor::from_vec(queries, (picked.len(), width), &device).unwrap();
        let points = Tensor::from_vec(host_table.clone(), (host_table.len() / width, width), &device).unwrap();
        let best = nearest_scores(&query, &points).unwrap().argmax(1).unwrap().to_vec1::<u32>().unwrap();
        assert_eq!(best, picked, "each pick is the kernel's argmax for its recorded query");
    }

    /// claude-253, the card's I-Force on the kernels. A window forced to every one of its own
    /// decisions gives its rows, and its trace byte for byte. One forced bucket takes its entry,
    /// keeps its score as computed, and moves the forward after it. A forced query code in the
    /// stream read and a forced key code in the readout read each take their code, and every
    /// kept set of both reads is the one the codes as forced choose.
    #[cfg(feature = "cuda")]
    #[test]
    fn a_forced_forward_takes_each_forced_decision() {
        use crate::flock_read_hamming::{set_hamming_reads, HammingReads, HAMMING_KEPT};
        use uor_r4_integer::fbs_snapshot::{ForcedBucket, ForcedCode};
        let device = Device::new_cuda(0).unwrap();
        let model = build_fbs_model(FbsArm::OneAtomActive, 1, &device).unwrap();
        set_hamming_reads(Some(
            HammingReads::new(4, hs::SHEET_WIDTH, &device).unwrap(),
        ));
        let (mut tokens, mut mask) = (Vec::new(), Vec::new());
        for doc in 0..2u32 {
            tokens.push(hs::SHEET_BOS_ID);
            mask.push(0u8);
            for (ids, flag) in [(10..20u32, 0u8), (20..26, 1), (26..30, 0), (30..34, 1)] {
                for t in ids {
                    tokens.push(t + 40 * doc);
                    mask.push(flag);
                }
            }
        }
        let (windows, _) =
            hs::build_document_windows(&tokens, &mask, SHEET_CONTEXT, hs::SHEET_BOS_ID).unwrap();
        let position = windows[1].scored_corpus_positions[0];
        let (lanes, head_width) = (hs::SHEET_WIDTH / 4, hs::SHEET_WIDTH / 4);
        let run = |force: Option<DecisionForceWindow>| {
            let mut trace = FTrace::new(&[position]).unwrap();
            if let Some(window) = force {
                let file = DecisionForce {
                    schema: DECISION_FORCE_SCHEMA.to_owned(),
                    windows: vec![window],
                };
                trace = trace
                    .with_force(file, 6, lanes, &[(2, 4, head_width), (6, 4, head_width)])
                    .unwrap();
            }
            let picks = row_picks(
                &model,
                &windows,
                &mask,
                1,
                0,
                false,
                Some(&mut trace),
                |_, _| Ok(()),
            )
            .unwrap();
            (picks, trace.finish().unwrap())
        };
        let (plain, base) = run(None);
        let w = base.windows[0].clone();
        let reads = |s: &DecisionStep| {
            s.layers
                .iter()
                .filter_map(|l| l.read.clone())
                .chain(s.readout.clone())
                .collect::<Vec<_>>()
        };
        // Every one of the window's own decisions, forced.
        let mut own = DecisionForceWindow {
            position,
            ..Default::default()
        };
        for (step, s) in w.steps.iter().enumerate() {
            for (layer, l) in s.layers.iter().enumerate() {
                own.buckets.extend(
                    l.entry
                        .iter()
                        .enumerate()
                        .map(|(lane, &entry)| ForcedBucket {
                            step,
                            layer,
                            lane,
                            entry,
                        }),
                );
            }
            for read in reads(s) {
                for (head, h) in read.heads.iter().enumerate() {
                    let code = |role, code| ForcedCode {
                        step,
                        depth: read.depth,
                        head,
                        role,
                        code,
                    };
                    own.codes.push(code(CodeRole::Query, h.query));
                    if step > 0 {
                        own.codes.push(code(CodeRole::Key, h.key));
                    }
                }
            }
        }
        let (picks, again) = run(Some(own));
        assert_eq!(
            picks, plain,
            "forced to its own decisions, the rows are the same"
        );
        assert_eq!(
            serde_json::to_vec(&again).unwrap(),
            serde_json::to_vec(&base).unwrap(),
            "and the trace, byte for byte"
        );
        // One bucket, at layer 0 so the layers after it read it.
        let step = w.steps.len() / 2;
        let entry = w.steps[step].layers[0].entry[0];
        let other = (entry + 8) % 16;
        let bucket = ForcedBucket {
            step,
            layer: 0,
            lane: 0,
            entry: other,
        };
        let (_, bucketed) = run(Some(DecisionForceWindow {
            position,
            buckets: vec![bucket],
            codes: vec![],
        }));
        let at = &bucketed.windows[0].steps[step].layers[0];
        assert_eq!(
            (at.entry[0], at.score_f32[0]),
            (other, w.steps[step].layers[0].score_f32[0])
        );
        let mut unmoved = base.clone();
        unmoved.windows[0].steps[step].layers[0].entry[0] = other;
        assert_ne!(
            bucketed, unmoved,
            "the forward after the forced bucket moves"
        );
        // A query code at the last step in the stream read (depth 2), and a key code at an earlier
        // step in the readout read (depth 6), each with its first bit flipped.
        let (t, s) = (w.steps.len() - 1, w.steps.len() / 2);
        let query = reads(&w.steps[t])[0].heads[0].query ^ 1;
        let key = reads(&w.steps[s])[1].heads[1].key ^ 1;
        let codes = vec![
            ForcedCode {
                step: t,
                depth: 2,
                head: 0,
                role: CodeRole::Query,
                code: query,
            },
            ForcedCode {
                step: s,
                depth: 6,
                head: 1,
                role: CodeRole::Key,
                code: key,
            },
        ];
        let (_, coded) = run(Some(DecisionForceWindow {
            position,
            buckets: vec![],
            codes,
        }));
        let cw = &coded.windows[0];
        assert_eq!(
            (
                reads(&cw.steps[t])[0].heads[0].query,
                reads(&cw.steps[s])[1].heads[1].key
            ),
            (query, key)
        );
        for r in 0..2 {
            for h in 0..4 {
                let keys: Vec<u64> = cw
                    .steps
                    .iter()
                    .map(|st| reads(st)[r].heads[h].key)
                    .collect();
                for (row, st) in cw.steps.iter().enumerate() {
                    let head = &reads(st)[r].heads[h];
                    let mut nearest: Vec<u32> = (1..row as u32).collect();
                    nearest.sort_by_key(|&c| ((head.query ^ keys[c as usize]).count_ones(), c));
                    nearest.truncate(HAMMING_KEPT);
                    assert_eq!(head.kept, nearest, "read {r}, head {h}, row {row}");
                }
            }
        }
        set_hamming_reads(None);
    }

    /// claude-218 (FIT round 1): each query is named by the commonest target among the TRAIN
    /// openings at its least L1 distance, ties to the smaller id, across the search's blocks and
    /// inside one; and by sign pattern, whose L1 is the Hamming distance over the width. On the
    /// device: CPU tensor compute is refused (D24/D29).
    #[cfg(feature = "cuda")]
    #[test]
    fn the_opening_keys_name_each_opening_by_its_nearest_train_opening() {
        let device = Device::new_cuda(0).unwrap();
        // 5,000 TRAIN rows, past one 4,096-row block: row r is (r, 0), target r % 7.
        let rows: Vec<f32> = (0..5000).flat_map(|r| [r as f32, 0.0]).collect();
        let train = Tensor::from_vec(rows, (5000, 2), &device).unwrap();
        let train_targets: Vec<u32> = (0..5000u32).map(|r| r % 7).collect();
        // 4,500.2 is nearest row 4,500 alone (second block); −3 is nearest row 0; 4,095.5 ties
        // rows 4,095 (target 0) and 4,096 (target 1) across the blocks, and names the smaller.
        let query = Tensor::from_vec(vec![4500.2f32, 0.0, -3.0, 0.0, 4095.5, 0.0], (3, 2), &device).unwrap();
        let near = l1_named(&query, &train, &train_targets).unwrap();
        assert_eq!(near.iter().map(|n| (n.target, n.ties)).collect::<Vec<_>>(), [(4500 % 7, 1), (0, 1), (0, 2)]);
        assert!((near[0].distance - 0.1).abs() < 1e-3 && (near[1].distance - 1.5).abs() < 1e-6, "{near:?}");
        // Ties inside one block: every row is (100, 0) with target 0, but rows 1 and 1,024 are
        // (5, 0), with targets 9 and 3: a tie of one each names the smaller id, 3. With row 2,000
        // also at (5, 0) and targets 3, 9, 9, the commonest, 9, names it, not the first row's 3.
        let block = |at: &[(usize, u32)]| {
            let rows: Vec<f32> = (0..4096)
                .flat_map(|r| [if at.iter().any(|&(i, _)| i == r) { 5.0 } else { 100.0 }, 0.0])
                .collect();
            let targets: Vec<u32> = (0..4096).map(|r| at.iter().find(|&&(i, _)| i == r).map_or(0, |&(_, t)| t)).collect();
            (Tensor::from_vec(rows, (4096, 2), &device).unwrap(), targets)
        };
        let five = Tensor::from_vec(vec![5.0f32, 0.0], (1, 2), &device).unwrap();
        let (table, targets) = block(&[(1, 9), (1024, 3)]);
        assert_eq!(l1_named(&five, &table, &targets).unwrap()[0], Named { target: 3, distance: 0.0, ties: 2 });
        let (table, targets) = block(&[(1, 3), (1024, 9), (2000, 9)]);
        assert_eq!(l1_named(&five, &table, &targets).unwrap()[0], Named { target: 9, distance: 0.0, ties: 3 });
        // The sign code: 0/1 per coordinate, zero counting as positive.
        let points = Tensor::from_vec(vec![0.5f32, -0.2, -1.0, 0.0], (2, 2), &device).unwrap();
        let code = sign_code(&points).unwrap().to_vec2::<f32>().unwrap();
        assert_eq!(code, [[1.0, 0.0], [0.0, 1.0]]);
        let keyed = opening_keys("p", &query, vec![1, 2, 3], vec![10, 20, 30], &train, &train_targets).unwrap();
        assert_eq!(keyed.point_target, [4500 % 7, 0, 0]);
        assert_eq!(keyed.point_ties, [1, 1, 2]);
        assert_eq!((keyed.train_openings, keyed.table_bytes), (5000, 5000 * 2 * 4));
        // Every TRAIN row's sign code is (1, 1), so all 5,000 tie for every query; targets 0 and
        // 1 are the commonest (715 each), so 0 names them. Hamming distance 0 from the queries
        // with both coordinates at or above zero, and 1/2 from (−3, 0).
        assert_eq!(keyed.sign_target, [0, 0, 0]);
        assert_eq!(keyed.sign_ties, [5000, 5000, 5000]);
        assert_eq!(keyed.sign_distance, [0.0, 0.5, 0.0]);
        // The Hadamard sign key (Casey's amendment) is written beside them, for every opening.
        assert_eq!((keyed.hsign_target.len(), keyed.hsign_distance.len(), keyed.hsign_ties.len()), (3, 3, 3));
        assert!(keyed.hsign_ties.iter().all(|&t| t >= 1));
        // The transform: H4 (1, 2, 3, 4) = (10, −2, −4, 0); twice is 4 times the input; a width
        // that is not a power of two is refused.
        let x = Tensor::from_vec(vec![1.0f32, 2.0, 3.0, 4.0], (1, 4), &device).unwrap();
        let hx = walsh_hadamard(&x).unwrap();
        assert_eq!(hx.to_vec2::<f32>().unwrap(), [[10.0, -2.0, -4.0, 0.0]]);
        assert_eq!(walsh_hadamard(&hx).unwrap().to_vec2::<f32>().unwrap(), [[4.0, 8.0, 12.0, 16.0]]);
        assert!(walsh_hadamard(&Tensor::zeros((1, 3), DType::F32, &device).unwrap()).is_err());
        // The code is the sign of H·S·x, by hand (the code review): S = (−, −, +, −), so
        // S·x = (−1, −2, 3, −4), H4 of it = (−4, 8, −2, −6), and the code is (0, 1, 0, 0). The
        // unrotated code, of H·x = (10, −2, −4, 0), would be (1, 0, 0, 1).
        assert_eq!(hadamard_sign_code(&x).unwrap().to_vec2::<f32>().unwrap(), [[0.0, 1.0, 0.0, 0.0]]);
    }

    /// claude-218 (Casey's amendment): the Hadamard rotation's fixed signs are exact, the same at
    /// every call, and mixed.
    #[test]
    fn the_hadamard_rotations_signs_are_fixed_and_mixed() {
        let s = rotation_signs(256);
        assert_eq!(s, rotation_signs(256));
        assert!(s.iter().all(|&v| v == 1.0 || v == -1.0));
        let negative = s.iter().filter(|&&v| v < 0.0).count();
        assert!((96..=160).contains(&negative), "{negative} of 256 negated");
        assert_eq!(rotation_signs(4), rotation_signs(256)[..4]);
        // The first eight, as literals (the code review): a changed hash or rule fails here.
        assert_eq!(rotation_signs(8), [-1.0, -1.0, 1.0, -1.0, 1.0, 1.0, 1.0, -1.0]);
    }

    /// claude-219: a keyed opening gives the key's name only where the served memory's exact key
    /// misses the opening and the key's distance is within τ; the key read is the one named; and
    /// a sampled opening the keys do not name, or name with another target, is refused.
    #[test]
    fn a_keyed_opening_fires_only_where_the_exact_key_misses_within_tau() {
        use crate::flock_read_opening::{opening_slots, OpeningAnchor};
        assert_eq!(keyed_opening(false, 7, 0.25, 0.5), Some(7));
        assert_eq!(keyed_opening(false, 7, 0.5, 0.5), Some(7), "at τ itself");
        assert_eq!(keyed_opening(false, 7, 0.75, 0.5), None);
        assert_eq!(keyed_opening(true, 7, 0.0, f32::INFINITY), None, "the exact key fires");
        assert_eq!(keyed_opening(false, 7, 9.0, f32::INFINITY), Some(7), "+∞ fires wherever it misses");
        assert_eq!(keyed_opening(false, 7, f32::NAN, f32::INFINITY), None);
        let arm = |k: &str, t: f32| (k.to_owned(), t);
        assert!(keyed_ok(&[arm("sign", 0.0), arm("hsign", 0.5), arm("point", f32::INFINITY)]));
        assert!(!keyed_ok(&[arm("sign", 0.0), arm("sign", 1.0)]));
        assert!(!keyed_ok(&[arm("exact", 0.0)]));
        assert!(!keyed_ok(&[arm("point", -1.0)]));
        assert!(!keyed_ok(&[arm("point", f32::NAN)]));
        // Two prompts after BOS; the memory's table holds the first's own turn key, read by the
        // served slots, so its opening fires and the second's misses.
        let window = |ids: Vec<u32>, target: u32, position: usize| CausalWindow {
            doc_id: 0,
            scored_offsets: vec![ids.len() - 1],
            scored_targets: vec![target],
            scored_corpus_positions: vec![position],
            input_ids: ids,
        };
        let a: Vec<u32> = std::iter::once(hs::SHEET_BOS_ID).chain(10..30).collect();
        let b: Vec<u32> = std::iter::once(hs::SHEET_BOS_ID).chain(40..60).collect();
        let slots = opening_slots(&a, 8, OpeningAnchor::Turn);
        let key: Vec<u32> = slots[a.len() - 1].iter().map(|&s| a[s]).collect();
        assert!(!key.is_empty());
        let table = std::collections::HashMap::from([(key, 5u32)]);
        let memory = crate::flock_read_memory::TurnMemory::new(8, table, 4, &Device::Cpu).unwrap();
        let keys = OpeningKeys {
            split: "p".to_owned(),
            train_openings: 3,
            table_bytes: 48,
            positions: vec![100, 200],
            targets: vec![1, 2],
            point_target: vec![11, 12],
            point_distance: vec![0.0, 0.25],
            point_ties: vec![1, 1],
            sign_target: vec![21, 22],
            sign_distance: vec![0.0, 0.75],
            sign_ties: vec![1, 1],
            point_ms: 0,
            sign_ms: 0,
            hsign_target: vec![31, 32],
            hsign_distance: vec![0.0, 0.0],
            hsign_ties: vec![1, 1],
            hsign_ms: 0,
        };
        let (wa, wb) = (window(a.clone(), 1, 100), window(b.clone(), 2, 200));
        let prompts = [&wa, &wb];
        let keyed = |key: &str, tau: f32| keyed_openings(&prompts, &keys, key, tau, Some(&memory)).unwrap();
        assert_eq!(keyed("point", 0.5), [None, Some(12)]);
        assert_eq!(keyed("point", 0.125), [None, None]);
        assert_eq!(keyed("sign", 0.5), [None, None]);
        assert_eq!(keyed("sign", f32::INFINITY), [None, Some(22)]);
        assert_eq!(keyed("hsign", 0.0), [None, Some(32)]);
        // With no memory, no exact key fires.
        assert_eq!(keyed_openings(&prompts, &keys, "hsign", 0.0, None).unwrap(), [Some(31), Some(32)]);
        let (unnamed, moved) = (window(b.clone(), 2, 300), window(b, 3, 200));
        assert!(keyed_openings(&[&unnamed], &keys, "point", 1.0, Some(&memory)).is_err());
        assert!(keyed_openings(&[&moved], &keys, "point", 1.0, Some(&memory)).is_err());
        // A key that would give BOS is refused (the code review); where it does not fire, it is not.
        let bos = OpeningKeys { point_target: vec![11, hs::SHEET_BOS_ID], ..keys.clone() };
        assert!(keyed_openings(&prompts, &bos, "point", 0.5, Some(&memory)).is_err());
        assert_eq!(keyed_openings(&prompts, &bos, "point", 0.125, Some(&memory)).unwrap(), [None, None]);
    }

    /// claude-220: the served table reads back exactly as it was saved, and is never replaced.
    #[test]
    fn the_opening_table_reads_back_as_saved_and_is_never_replaced() {
        /// Removes the test's folder however the test ends.
        struct Gone(PathBuf);
        impl Drop for Gone {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let dir = std::env::temp_dir().join(format!("claude-220-table-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let _gone = Gone(dir.clone());
        let path = dir.join(OPENING_TABLE_FILE);
        let values: Vec<f32> = vec![0.5, -1.25, 3.0, 0.0, -0.0, 7.75];
        let points = Tensor::from_vec(values.clone(), (3, 2), &Device::Cpu).unwrap();
        save_opening_table(&path, &points, &[9, 4, 9]).unwrap();
        let (back, targets) = load_opening_table(&path, &Device::Cpu).unwrap();
        assert_eq!(targets, [9, 4, 9]);
        assert_eq!(back.dims(), [3, 2]);
        let bits = |v: Vec<f32>| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
        assert_eq!(bits(back.flatten_all().unwrap().to_vec1::<f32>().unwrap()), bits(values.clone()));
        assert!(same_bits(&back, &points).unwrap(), "the read-back's bits are the saved ones");
        assert!(save_opening_table(&path, &points, &[9, 4, 9]).is_err(), "a table is never replaced");
        assert_eq!(load_opening_table(&path, &Device::Cpu).unwrap().1, [9, 4, 9], "the refused save left the table whole");
        assert!(save_opening_table(&dir.join("other.safetensors"), &points, &[9, 4]).is_err(), "one target per point");
        let wide = dir.join("f64.safetensors");
        let points_f64 = Tensor::from_vec(values.iter().map(|&x| f64::from(x)).collect::<Vec<_>>(), (3, 2), &Device::Cpu).unwrap();
        save_opening_table(&wide, &points_f64, &[9, 4, 9]).unwrap();
        assert!(load_opening_table(&wide, &Device::Cpu).is_err(), "a table of f64 points is not served");
        // The read-back check is by bits: −0.0 against +0.0 differs, though their values are equal.
        let signed = Tensor::from_vec(vec![0.5f32, -1.25, 3.0, -0.0, -0.0, 7.75], (3, 2), &Device::Cpu).unwrap();
        assert!(!same_bits(&signed, &points).unwrap());
        assert!(!same_bits(&points.reshape((2, 3)).unwrap(), &points).unwrap(), "the shape is part of it");
    }

    /// claude-220: the served key names only a reply's first token, and only under greedy.
    #[test]
    fn the_served_key_serves_greedys_first_step_only() {
        assert!(serves_at(&[], DecodeRule::Greedy));
        assert!(!serves_at(&[5], DecodeRule::Greedy), "past the first step");
        for rule in [DecodeRule::FLOCK, DecodeRule::FlockCell { k: 4, margin_milli: 0 }, DecodeRule::CycleStep { k: 4, n: 4 }] {
            assert!(!serves_at(&[], rule), "{rule:?} decodes as it did");
        }
    }

    /// claude-224: the panel prompts are the scored run's own. Each prompt is its window's input
    /// through the first scored offset, which ends at the row that predicts the reply's first
    /// token. Its reference is the true reply that follows, and its position is that reply's
    /// first target.
    #[test]
    fn the_panel_prompts_are_the_scored_runs_own() {
        let bos = hs::SHEET_BOS_ID;
        let (mut tokens, mut mask) = (Vec::new(), Vec::new());
        for doc in 0..2u32 {
            tokens.push(bos);
            mask.push(0u8);
            tokens.extend((0..20).map(|i| 10 + i + doc));
            mask.extend([0u8; 20]);
            tokens.extend([5, 6, 7]);
            mask.extend([1u8; 3]);
        }
        let windows = hs::build_document_windows(&tokens, &mask, SHEET_CONTEXT, bos).unwrap().0;
        let prompt = |doc: u32| std::iter::once(bos).chain((0..20).map(|i| 10 + i + doc)).collect::<Vec<u32>>();
        let row = |position: usize, doc: u32| PanelPrompt { position, prompt: prompt(doc), reference: vec![5, 6, 7], float_keyed: None };
        assert_eq!(split_prompts(&windows, &mask, 2, 64), [row(21, 0), row(45, 1)]);
        assert_eq!(split_prompts(&windows, &mask, 2, 2)[0].reference, [5, 6], "at most the reply tokens");
        assert_eq!(split_prompts(&windows, &mask, 1, 64), [row(21, 0)], "the stride takes the first");
    }

    fn integer_sample(keyed: Option<u32>, reply: Vec<u32>, reference: Vec<u32>) -> IntegerSample {
        IntegerSample {
            position: 0,
            prompt: vec![2],
            reply,
            reference,
            keyed,
            served_distance_q12: keyed.map(|_| 10),
            float_keyed: None,
            float_distance_q12: None,
            end: "length".to_owned(),
        }
    }

    /// claude-224: a seed's float names attach only to the prompts its served arm decoded, and an
    /// integer served file must carry this seed's names, or none where no float arm served.
    #[test]
    fn the_float_names_are_the_seeds_own() {
        let prompt = |position| PanelPrompt { position, prompt: vec![2, 3], reference: vec![4], float_keyed: None };
        let sample = |position, keyed| IdSample {
            position,
            prompt: vec![2, 3],
            reply: vec![4],
            reference: vec![4],
            reply_text: String::new(),
            reference_text: String::new(),
            reply_keys: 0,
            reply_named: 0,
            reference_keys: 0,
            reference_named: 0,
            admitted: Vec::new(),
            stepped: Vec::new(),
            keyed,
            served_distance: None,
        };
        let prompts = [prompt(10), prompt(20)];
        let served = [sample(10, Some(7)), sample(20, None)];
        let named = attach_float_names(&prompts, &served).unwrap();
        assert_eq!(named.iter().map(|p| p.float_keyed).collect::<Vec<_>>(), [Some(7), None]);
        assert!(attach_float_names(&prompts, &served[..1]).is_err(), "another count");
        let mut moved = served.clone();
        moved[1].position = 21;
        assert!(attach_float_names(&prompts, &moved).is_err(), "another prompt");
        let int = |position, float| IntegerSample { position, float_keyed: float, ..integer_sample(None, vec![4], vec![4]) };
        assert!(check_float_names("p-w", 1, &[int(10, Some(7)), int(20, None)], Some(&served)).is_ok());
        assert!(check_float_names("p-w", 1, &[int(10, Some(8)), int(20, None)], Some(&served)).is_err(), "another seed's name");
        assert!(check_float_names("p-x", 1, &[int(10, None), int(20, None)], None).is_ok());
        assert!(check_float_names("p-x", 1, &[int(10, Some(7)), int(20, None)], None).is_err(), "names where no float arm served");
    }

    /// claude-224: parity's classes are the item's, fixed before any panel is read. An integer
    /// name that differs from the float's is within the rounding bound when the float name's least
    /// distance is at most 512 past the integer's, and drift otherwise, a float name no row holds
    /// included. Openings only one side served are counted apart.
    #[test]
    fn the_integer_parity_classes_are_the_items() {
        let s = |keyed: Option<u32>, float: Option<u32>, float_d: Option<i64>| IntegerSample {
            float_keyed: float,
            float_distance_q12: float_d,
            ..integer_sample(keyed, vec![1], vec![1])
        };
        let served = [
            s(Some(5), Some(5), Some(10)),
            s(Some(5), Some(6), Some(522)),
            s(Some(5), Some(6), Some(523)),
            s(Some(5), Some(6), None),
            s(Some(5), None, None),
            s(None, Some(6), Some(10)),
        ];
        let p = integer_parity("p-x", 1, &served);
        assert_eq!((p.served, p.both, p.agree, p.within, p.drift), (5, 4, 1, 1, 2));
    }

    /// claude-224: the integer reading pairs served against greedy by their own tokens. A served
    /// reply must open with its keyed token, and a reply the key did not open must be greedy's;
    /// either is an instrument fault, as is a different prompt.
    #[test]
    fn the_integer_reading_pairs_the_arms_and_refuses_an_instrument_fault() {
        let empty = std::collections::HashSet::new();
        let served = [
            integer_sample(Some(7), vec![7, 8], vec![7, 8]),
            integer_sample(None, vec![4], vec![4]),
            integer_sample(Some(3), vec![3, 9], vec![2, 9]),
        ];
        let greedy = [
            integer_sample(None, vec![9, 8], vec![7, 8]),
            integer_sample(None, vec![4], vec![4]),
            integer_sample(None, vec![2, 9], vec![2, 9]),
        ];
        let r = integer_reading("p-x", 1, &served, &greedy, 1, &empty, &empty).unwrap();
        assert_eq!((r.keyed, r.right, r.exact), (2, 1, Some((1, 1))));
        let mut unopened = served.clone();
        unopened[0].reply = vec![9, 8];
        assert!(integer_reading("p-x", 1, &unopened, &greedy, 1, &empty, &empty).is_err(), "a served reply not opening with its key");
        let mut drifted = served.clone();
        drifted[1].reply = vec![5];
        assert!(integer_reading("p-x", 1, &drifted, &greedy, 1, &empty, &empty).is_err(), "an unkeyed reply that is not greedy's");
        let mut other = greedy.clone();
        other[2].prompt = vec![3];
        assert!(integer_reading("p-x", 1, &served, &other, 1, &empty, &empty).is_err(), "another prompt");
    }

    /// claude-220 (FIT round 1): the parity count compares each served token with the keys
    /// file's name and measures the distance drift where they agree; the served arm is read by its
    /// own tokens; parity failing is row 1; and row 4 turns to row 3 when the seeds pooled show
    /// a gain beyond their band.
    #[test]
    fn the_served_parity_counts_disagreements_and_the_pooled_seeds_guard_row_4() {
        let n4 = |v: [u32; 4]| v.to_vec();
        let keys = OpeningKeys {
            split: "p".to_owned(),
            train_openings: 4,
            table_bytes: 0,
            positions: vec![10, 20, 30, 40],
            targets: n4([3, 7, 6, 6]),
            point_target: n4([5, 7, 8, 9]),
            point_distance: vec![0.0, 0.25, 0.5, 0.75],
            point_ties: n4([1; 4]),
            sign_target: n4([5; 4]),
            sign_distance: vec![1.0; 4],
            sign_ties: n4([1; 4]),
            point_ms: 0,
            sign_ms: 0,
            hsign_target: n4([5; 4]),
            hsign_distance: vec![1.0; 4],
            hsign_ties: n4([1; 4]),
            hsign_ms: 0,
        };
        let openings = GradeOpenings {
            name: "p".to_owned(),
            positions: vec![10, 20, 30, 40],
            targets: n4([3, 7, 6, 6]),
            exact: vec![(3, true), (0, false), (0, false), (0, false)],
            oracle: vec![0; 4],
            known: vec![false; 4],
        };
        let picks = SeedPicks { keys, base: n4([3, 4, 8, 5]), served: n4([3, 4, 8, 5]) };
        let served = |position: usize, keyed: Option<u32>, distance: Option<f32>| IdSample {
            position,
            prompt: vec![2],
            reply: vec![1],
            reference: vec![1],
            reply_text: String::new(),
            reference_text: String::new(),
            reply_keys: 0,
            reply_named: 0,
            reference_keys: 0,
            reference_named: 0,
            admitted: Vec::new(),
            stepped: Vec::new(),
            keyed,
            served_distance: distance,
        };
        // 10 fires (no served token); 20 and 30 agree with the keys file (drift 0 and 0.125); 40
        // disagrees (9 against 2).
        let arm = [served(10, None, None), served(20, Some(7), Some(0.25)), served(30, Some(8), Some(0.625)), served(40, Some(2), Some(0.7))];
        let (parity, own) = served_parity("p", 1, &openings, &picks, &arm).unwrap();
        assert_eq!((parity.served, parity.agree, parity.max_drift), (3, 2, 0.125));
        assert!(!parity.holds(), "1 of 3 disagreeing is past 1%");
        assert_eq!((own.keys.point_target[3], own.keys.point_distance[3]), (2, 0.7), "read by its own token");
        assert!(served_parity("p", 1, &openings, &picks, &[served(40, Some(2), None)]).is_err(), "a served token needs its distance");
        assert!(served_parity("p", 1, &openings, &picks, &[served(40, Some(2), Some(f32::NAN))]).is_err(), "a finite one");
        // Each served token is compared with the keys file as read: a second token at 40 does
        // not agree with the first.
        let twice = [served(40, Some(2), Some(0.7)), served(40, Some(2), Some(0.7))];
        let (parity, _) = served_parity("p", 1, &openings, &picks, &twice).unwrap();
        assert_eq!((parity.served, parity.agree), (2, 0));
        // The grade reads p-w and p-v, and refuses any other panel before it reads anything.
        let other = [("p-u".to_owned(), PathBuf::from("/nowhere"))];
        let refused = served_grade(Path::new("/nowhere"), Path::new("/nowhere"), &other).err().map(|e| e.to_string());
        assert!(refused.is_some_and(|e| e.contains("p-w and p-v only")), "another panel is refused");
        assert!(served_parity("p", 1, &openings, &picks, &[served(50, Some(2), Some(0.1))]).is_err(), "an opening the split lacks");
        // The rows.
        let at = |t: (usize, usize)| KeyedReading {
            split: "p-w".to_owned(),
            key: "point".to_owned(),
            seed: 1,
            tau: f32::INFINITY,
            replies: 1200,
            keyed: 0,
            right: 0,
            predicted_exact: (0, 0),
            exact: Some((0, 0)),
            template: Some(t),
            loops: Some((0, 0)),
        };
        let lift = ForcedReading { split: "p-w".to_owned(), seed: 1, replies: 1200, exact: (50, 0), template: (50, 0) };
        let forced = [lift.clone(), lift.clone(), lift];
        let seeds = |t: [(usize, usize); 3]| t.iter().map(|&t| (at(t), at((0, 0)))).collect::<Vec<_>>();
        let held = |served: usize, agree: usize| ServedParity { split: "p-w".to_owned(), seed: 1, served, agree, max_drift: 0.0 };
        assert_eq!(served_outcome(&forced, seeds([(20, 0); 3]), &[held(1000, 1000)]), 2);
        assert_eq!(served_outcome(&forced, seeds([(20, 0); 3]), &[held(1000, 989)]), 1, "past 1% disagreeing");
        assert_eq!(served_outcome(&forced, seeds([(20, 0); 3]), &[held(1000, 990)]), 2, "1% is within");
        assert_eq!(served_outcome(&forced, seeds([(20, 0); 3]), &[held(10, 10), held(1000, 989)]), 1, "every split and seed");
        // A seed below the bar but not thin: row 4 only when the pooled seeds show no gain.
        assert_eq!(served_outcome(&forced, seeds([(20, 0), (20, 0), (10, 8)]), &[held(10, 10)]), 3);
        assert_eq!(served_outcome(&forced, seeds([(10, 8); 3]), &[held(10, 10)]), 4);
        // A seed whose templates gain but whose loops rise beyond their band keeps row 4: a harm
        // is not a power shortfall (FIT round 2).
        let mut harmed = seeds([(20, 0), (20, 0), (25, 3)]);
        harmed[2].0.loops = Some((12, 2));
        assert_eq!(served_outcome(&forced, harmed, &[held(10, 10)]), 4);
        // Nor does a thin seed make a harm underpowered (FIT round 3): one thin seed, one seed
        // with a loop harm, one that carries, is row 4.
        let mut thin_and_harmed = seeds([(4, 2), (30, 0), (20, 0)]);
        thin_and_harmed[1].0.loops = Some((30, 2));
        assert_eq!(served_outcome(&forced, thin_and_harmed, &[held(10, 10)]), 4);
        assert_eq!(served_outcome(&forced, seeds([(4, 2), (20, 0), (20, 0)]), &[held(10, 10)]), 3, "thin alone is row 3");
        // Exact replies and DEV's templates are harms too (round 4's note), beside a thin seed.
        let mut exact_harm = seeds([(4, 2), (30, 0), (20, 0)]);
        exact_harm[1].0.exact = Some((2, 30));
        assert_eq!(served_outcome(&forced, exact_harm, &[held(10, 10)]), 4);
        let mut dev_harm = seeds([(4, 2), (30, 0), (20, 0)]);
        dev_harm[1].1.template = Some((2, 30));
        assert_eq!(served_outcome(&forced, dev_harm, &[held(10, 10)]), 4);
        assert!(!held(0, 0).holds(), "no served opening is no parity");
    }

    /// claude-219: the keyed grade predicts each keyed reply from the greedy and forced ones,
    /// refuses an arm whose tokens or replies are not those, counts exact, template and loop pairs
    /// against greedy, and reads the sheet's table in order.
    #[test]
    fn the_keyed_grade_predicts_each_keyed_reply_and_reads_the_table_in_order() {
        let sample = |position: usize, reply: Vec<u32>, reference: Vec<u32>, keyed: Option<u32>| IdSample {
            position,
            prompt: vec![2],
            reply,
            reference,
            reply_text: String::new(),
            reference_text: String::new(),
            reply_keys: 0,
            reply_named: 0,
            reference_keys: 0,
            reference_named: 0,
            admitted: Vec::new(),
            stepped: Vec::new(),
            keyed,
            served_distance: None,
        };
        // Four openings; the exact key fires on the first. The point key names 5, 7, 8 and 9 at
        // distances 0, 0.1, 0.2 and 0.9. The end of turn is 1.
        let openings = GradeOpenings {
            name: "p".to_owned(),
            positions: vec![10, 20, 30, 40],
            targets: vec![3, 7, 6, 6],
            exact: vec![(3, true), (0, false), (0, false), (0, false)],
            oracle: vec![0; 4],
            known: vec![false; 4],
        };
        let n = |v: [u32; 4]| v.to_vec();
        let keys = OpeningKeys {
            split: "p".to_owned(),
            train_openings: 4,
            table_bytes: 0,
            positions: vec![10, 20, 30, 40],
            targets: n([3, 7, 6, 6]),
            point_target: n([5, 7, 8, 9]),
            point_distance: vec![0.0, 0.1, 0.2, 0.9],
            point_ties: n([1; 4]),
            sign_target: n([5; 4]),
            sign_distance: vec![1.0; 4],
            sign_ties: n([1; 4]),
            point_ms: 0,
            sign_ms: 0,
            hsign_target: n([5; 4]),
            hsign_distance: vec![1.0; 4],
            hsign_ties: n([1; 4]),
            hsign_ms: 0,
        };
        let picks = SeedPicks { keys, base: n([3, 4, 8, 5]), served: n([3, 4, 8, 5]) };
        // Greedy: right at 10 only. Forced: every reference.
        let greedy = [
            sample(10, vec![3, 1], vec![3, 1], None),
            sample(20, vec![4, 1], vec![7, 6, 1], None),
            sample(30, vec![8, 2, 1], vec![6, 2, 1], None),
            sample(40, vec![5, 1], vec![6, 1], None),
        ];
        let forced: Vec<IdSample> = greedy.iter().map(|g| IdSample { reply: g.reference.clone(), ..g.clone() }).collect();
        // The keyed arm at +∞: 20 takes the reference's 7 (the forced reply), 30 greedy's own 8
        // (greedy's reply), and 40 a new 9, whose reply loops.
        let keyed = [
            greedy[0].clone(),
            sample(20, vec![7, 6, 1], vec![7, 6, 1], Some(7)),
            sample(30, vec![8, 2, 1], vec![6, 2, 1], Some(8)),
            sample(40, vec![9, 2, 9, 2, 9, 2, 1], vec![6, 1], Some(9)),
        ];
        let keep: std::collections::HashSet<u32> = (0..16).collect();
        let none = std::collections::HashSet::new();
        let read = |tau: f32, arm: Option<&[IdSample]>| {
            keyed_reading(("p", "point", 1, tau), &openings, &picks, &greedy, &forced, arm, 1, &keep, &none)
        };
        let r = read(f32::INFINITY, Some(&keyed)).unwrap();
        assert_eq!((r.replies, r.keyed, r.right), (4, 3, 1));
        assert_eq!(r.predicted_exact, (1, 0));
        assert_eq!((r.exact, r.template, r.loops), (Some((1, 0)), Some((1, 0)), Some((1, 0))));
        // Without the arm's file, only the prediction; at τ = 0.15 only 20 fires.
        let p = read(0.15, None).unwrap();
        assert_eq!((p.keyed, p.right, p.predicted_exact, p.exact), (1, 1, (1, 0), None));
        // A keyed token not the registered τ's, a reply not the one its opening fixes, and a new
        // opening the reply does not start with are each refused.
        let with = |j: usize, s: IdSample| {
            let mut a = keyed.to_vec();
            a[j] = s;
            a
        };
        assert!(read(f32::INFINITY, Some(&with(1, sample(20, vec![7, 6, 1], vec![7, 6, 1], None)))).is_err());
        assert!(read(f32::INFINITY, Some(&with(2, sample(30, vec![8, 3, 1], vec![6, 2, 1], Some(8))))).is_err());
        assert!(read(f32::INFINITY, Some(&with(3, sample(40, vec![5, 1], vec![6, 1], Some(9))))).is_err());
        assert!(read(f32::INFINITY, Some(&keyed[..3])).is_err(), "an arm's prompts are greedy's");
        // A forcing that both the forced and the keyed arm ignore is refused (the code review).
        let mut unforced = forced.clone();
        unforced[1].reply = vec![4, 1];
        let ignored = with(1, sample(20, vec![4, 1], vec![7, 6, 1], Some(7)));
        assert!(keyed_reading(("p", "point", 1, f32::INFINITY), &openings, &picks, &greedy, &unforced, Some(&ignored), 1, &keep, &none).is_err());
        // The table, in order: the forced arm without a lift at any seed is row 1; a key that
        // carries at all three is row 2, the cheapest first; every key thin at some seed is row 3;
        // else row 4.
        let at = |t: (usize, usize)| KeyedReading {
            split: "p-v".to_owned(),
            key: "k".to_owned(),
            seed: 1,
            tau: 0.0,
            replies: 256,
            keyed: 0,
            right: 0,
            predicted_exact: (0, 0),
            exact: Some((0, 0)),
            template: Some(t),
            loops: Some((0, 0)),
        };
        let seeds = |t: (usize, usize)| vec![(at(t), at((0, 0))); 3];
        let lift = ForcedReading { split: "p-v".to_owned(), seed: 1, replies: 256, exact: (50, 0), template: (50, 0) };
        let flat = ForcedReading { exact: (5, 5), template: (5, 5), ..lift.clone() };
        let carries = (12, 0);
        let table = |sign, hsign, point| vec![("sign", seeds(sign)), ("hsign", seeds(hsign)), ("point", seeds(point))];
        assert_eq!(keyed_outcome(&[flat.clone(), flat.clone(), flat.clone()], &table(carries, carries, carries)).0, 1);
        // One seed's forced arm without a lift is row 1 even where every key carries (FIT round 1),
        // and so is one too thin to show its lift.
        assert_eq!(keyed_outcome(&[lift.clone(), flat, lift.clone()], &table(carries, carries, carries)).0, 1);
        let thin = ForcedReading { exact: (6, 0), template: (6, 0), ..lift.clone() };
        assert_eq!(keyed_outcome(&[lift.clone(), lift.clone(), thin], &table(carries, carries, carries)).0, 1);
        let forced3 = [lift.clone(), lift.clone(), lift];
        assert_eq!(keyed_outcome(&forced3, &table((3, 1), (3, 1), carries)).2.as_deref(), Some("point"));
        assert_eq!(keyed_outcome(&forced3, &table(carries, (3, 1), carries)).2.as_deref(), Some("sign"));
        assert_eq!(keyed_outcome(&forced3, &table((3, 1), (3, 1), (3, 1))).0, 3);
        assert_eq!(keyed_outcome(&forced3, &table((10, 8), (3, 1), (10, 8))).0, 4);
        // b + c under 10 never carries, however large the net (the code review).
        assert_eq!(keyed_outcome(&forced3, &table((9, 0), (9, 0), (9, 0))).0, 3);
        // Thin at one seed of three, and carrying at none, is row 3, not row 4 (the code review).
        let mixed = vec![(at((3, 1)), at((0, 0))), (at((10, 8)), at((0, 0))), (at((10, 8)), at((0, 0)))];
        let by_seed = vec![("sign", mixed.clone()), ("hsign", mixed.clone()), ("point", mixed)];
        assert_eq!(keyed_outcome(&forced3, &by_seed).0, 3);
        // Worse exact replies, more loops or DEV's templates lower beyond their bands stop a key.
        let worse = |e: (usize, usize), l: (usize, usize), d: (usize, usize)| {
            let p = KeyedReading { exact: Some(e), loops: Some(l), ..at(carries) };
            p.carries(&KeyedReading { template: Some(d), ..at((0, 0)) })
        };
        assert!(worse((0, 0), (0, 0), (0, 0)));
        assert!(!worse((0, 9), (0, 0), (0, 0)));
        assert!(!worse((0, 0), (9, 0), (0, 0)));
        assert!(!worse((0, 0), (0, 0), (0, 9)));
    }

    /// claude-211 (Casey 30 Oct: "do not rule flock out, improve it instead"): with `keep_end`,
    /// a nearest end of turn is taken whatever the draw; otherwise, and for any other nearest
    /// token, the drawn rank is.
    #[test]
    fn the_flock_rule_keeps_a_nearest_end_of_turn() {
        let eos = Some(1u32);
        assert_eq!(flock_pick(&[1, 7, 9], 2, true, eos), 1, "the end of turn is kept");
        assert_eq!(flock_pick(&[1, 7, 9], 2, false, eos), 9, "the registered rule draws");
        assert_eq!(flock_pick(&[7, 1, 9], 2, true, eos), 9, "a farther end of turn is drawn");
        assert_eq!(flock_pick(&[7, 1, 9], 1, true, eos), 1);
        assert_eq!(flock_pick(&[1, 7, 9], 2, true, None), 9, "no end of turn to keep");
    }

    /// claude-213: the cell rule admits the nearest and every candidate within the margin of
    /// its score, the margin's edge included, and never fewer than one.
    #[test]
    fn the_cell_rule_admits_within_the_margin() {
        // Gaps from the nearest: 0, 0.375, 1.0, 1.125 (exact in binary).
        let ranked = [(5u32, -1.0f32), (3, -1.375), (9, -2.0), (2, -2.125)];
        assert_eq!(cell_admitted(&ranked, 0), 1);
        assert_eq!(cell_admitted(&ranked, 374), 1);
        assert_eq!(cell_admitted(&ranked, 500), 2);
        assert_eq!(cell_admitted(&ranked, CELL_MARGIN_MILLI), 3, "the edge is inside");
        assert_eq!(cell_admitted(&ranked, 4_000), 4);
        assert_eq!(cell_admitted(&[(1, -0.5), (2, -0.5), (3, -0.75)], 0), 2, "an exact tie");
        assert_eq!(cell_admitted(&[(1, -0.5)], 0), 1);
        assert_eq!(cell_admitted(&[], 1_000), 0);
        // The ladder is the objective's margin, then halving, and every margin names its own file.
        assert_eq!(FLOCK_CELL_LADDER_MILLI[0], CELL_MARGIN_MILLI);
        assert!(cells_ok(&FLOCK_CELL_LADDER_MILLI));
        assert!(!cells_ok(&[500, 500]), "a margin twice would write one file twice");
        assert!(!cells_ok(&[4_001]));
    }

    /// claude-213: a sample decoded under any rule but the cell rule serializes exactly as
    /// before, with no `admitted` key, so the registered files reproduce byte for byte; and a
    /// file written before claude-213 still reads.
    #[test]
    fn the_cell_rules_record_leaves_the_other_rules_files_alone() {
        let sample = |admitted: Vec<u8>| IdSample {
            position: 7,
            prompt: vec![1, 2],
            reply: vec![3],
            reference: vec![4],
            reply_text: "a".to_owned(),
            reference_text: "b".to_owned(),
            reply_keys: 0,
            reply_named: 0,
            reference_keys: 0,
            reference_named: 0,
            admitted,
            stepped: Vec::new(),
            keyed: None,
            served_distance: None,
        };
        let before = r#"{"position":7,"prompt":[1,2],"reply":[3],"reference":[4],"reply_text":"a","reference_text":"b","reply_keys":0,"reply_named":0,"reference_keys":0,"reference_named":0}"#;
        assert_eq!(serde_json::to_string(&sample(Vec::new())).unwrap(), before);
        assert_eq!(serde_json::from_str::<IdSample>(before).unwrap(), sample(Vec::new()));
        let cell = serde_json::to_string(&sample(vec![2, 1])).unwrap();
        assert!(cell.ends_with(r#""admitted":[2,1]}"#), "{cell}");
        // claude-217: the cycle step's marks are written only under that rule, the same way.
        let step = IdSample { stepped: vec![STEP_NEAREST, STEP_OFF, STEP_EXHAUSTED], ..sample(Vec::new()) };
        let step = serde_json::to_string(&step).unwrap();
        assert!(step.ends_with(r#""reference_named":0,"stepped":[0,1,2]}"#), "{step}");
    }

    /// claude-213: the sheet's selection on DEV (the widest qualified margin; else the nearest
    /// of those holding decision 4; else none) and its decision on the panel.
    #[test]
    fn the_cell_selection_and_outcome_follow_the_sheet() {
        let read = |holds: bool, v: f64, repeat: f64| {
            let made = |complete: usize, verbatim: usize, repeat: f64| Generation {
                replies: 256,
                complete,
                verbatim_train: verbatim,
                new_ngram_share: Vec::new(),
                distinct_1: 0.0,
                distinct_2: 0.0,
                repeated_4gram_rate: repeat,
            };
            // References: v_ref = 0.5; 200 complete, so tol = 0.10.
            let reading = VerdictSeed::new(1, &made(200, (v * 200.0) as usize, repeat), &made(200, 100, 0.0));
            CellReading {
                split: "dev".to_owned(),
                rule: "flock-cell".to_owned(),
                seed: 1,
                close: reading.generalizes(),
                reading,
                holds: Some(holds),
                mean_admitted: Some(1.0),
                loops: None,
                fewer_loops: None,
                mean_length: None,
                matched_n: None,
                matched_verbatim: None,
            }
        };
        let (good, salad, loops, off) = (
            read(true, 0.5, 0.0),
            read(true, 0.02, 0.0),
            read(false, 0.5, 0.6),
            read(true, 0.3, 0.0),
        );
        assert!(good.close && !salad.close && !off.close);
        fn three(r: &CellReading) -> Vec<&CellReading> {
            vec![r; 3]
        }
        fn margins(rows: [(u32, &CellReading); 3]) -> Vec<(u32, Vec<&CellReading>)> {
            rows.iter().map(|&(m, r)| (m, three(r))).collect()
        }
        // The widest qualified margin, whatever the order given.
        assert_eq!(
            select_margin(&margins([(125, &good), (1_000, &salad), (500, &good)])),
            (Some(500), "qualified")
        );
        // None qualifies: the nearest of those holding decision 4 (0.2 beats 0.48).
        assert_eq!(
            select_margin(&margins([(1_000, &salad), (250, &off), (31, &loops)])),
            (Some(250), "closest, not qualified")
        );
        // Equal mean gaps: the tie goes to the wider margin, whatever the order given.
        assert_eq!(
            select_margin(&margins([(125, &off), (250, &off), (31, &loops)])),
            (Some(250), "closest, not qualified")
        );
        // One seed of three failing decision 4 rules a margin out.
        let mixed = vec![(500u32, vec![&good, &good, &loops])];
        assert_eq!(select_margin(&mixed), (None, "none holds decision 4"));
        // The panel.
        assert_eq!(panel_outcome(&three(&good), &three(&salad)), "improves");
        assert_eq!(panel_outcome(&three(&off), &three(&salad)), "narrows");
        assert_eq!(panel_outcome(&three(&salad), &three(&off)), "no gain", "farther than FLOCK_END");
        assert_eq!(panel_outcome(&three(&loops), &three(&salad)), "no gain", "decision 4 fails");
        assert_eq!(panel_outcome(&three(&good)[..2], &three(&salad)), "no gain", "two seeds");
        assert_eq!(
            panel_outcome(&three(&off), &three(&salad)[..2]),
            "no gain",
            "FLOCK_END at two seeds is no comparison"
        );
        // FLOCK_END with nothing complete has no share at all, so any share is nearer.
        let silent = CellReading {
            reading: VerdictSeed {
                complete: 0,
                v: None,
                tol: None,
                ..salad.reading.clone()
            },
            ..salad.clone()
        };
        assert!(silent.gap().is_none());
        assert_eq!(panel_outcome(&three(&off), &three(&silent)), "narrows");
    }

    /// claude-215: a looping reply repeats a 4-gram of its own before its end of turn; the read's
    /// outcome needs fewer loops beyond the band, decision 4, and (for "cures") close, at all
    /// three seeds; and `--reply-read`'s scopes are distinct and never `Full`.
    #[test]
    fn the_read_grade_counts_loops_and_follows_the_sheet() {
        let eos = 9;
        let replies = vec![
            vec![1, 2, 3, 4, 1, 2, 3, 4, eos],    // repeats 1 2 3 4: loops
            vec![1, 2, 3, 4, eos, 1, 2, 3, 4],    // the repeat is after its end of turn
            vec![5, 5, 5, 5, 5],                  // 5 5 5 5 twice: loops, never ends
            vec![1, 2, 3, eos],                   // too short to loop
        ];
        assert_eq!(looping_replies(&replies, eos), 2);
        // Greedy loops in 64 of 256: the band is ⌈2·√(256·0.25·0.75)⌉ = 14, so 49 is fewer
        // beyond it and 50 is not.
        assert_eq!(count_band(64, 256), 14);
        assert!(fewer_loops(49, 64, 256));
        assert!(!fewer_loops(50, 64, 256), "inside the band");
        assert!(!fewer_loops(70, 64, 256));
        // References: 120 of 200 complete replies verbatim, so v_ref = 0.6 and tol = 0.104.
        let reading = |fewer: bool, holds: bool, close: bool, verbatim: usize, loops: usize| {
            let made = |verbatim_train: usize| Generation {
                replies: 256,
                complete: 200,
                verbatim_train,
                new_ngram_share: Vec::new(),
                distinct_1: 0.0,
                distinct_2: 0.0,
                repeated_4gram_rate: 0.0,
            };
            CellReading {
                split: "p-r".to_owned(),
                rule: "greedy-read-all".to_owned(),
                seed: 1,
                reading: VerdictSeed::new(1, &made(verbatim), &made(120)),
                holds: Some(holds),
                close,
                mean_admitted: None,
                loops: Some(loops),
                fewer_loops: Some(fewer),
                mean_length: None,
                matched_n: None,
                matched_verbatim: None,
            }
        };
        // Greedy loops in 60 of 256 with v = 0.5, a gap of 0.1.
        let g = reading(false, false, false, 100, 60);
        let greedy = [&g, &g, &g];
        let (cured, kept, lost, same, worse) = (
            reading(true, true, true, 120, 10),
            reading(true, true, false, 100, 10), // gap 0.1: within greedy's 0.1 + tol
            reading(true, true, false, 40, 10),  // v = 0.2, gap 0.4: past 0.1 + 0.104
            reading(false, true, true, 120, 10),
            reading(true, false, true, 120, 10),
        );
        assert_eq!(read_outcome(&[&cured, &cured, &cured], &greedy), "cures");
        assert_eq!(read_outcome(&[&cured, &kept, &cured], &greedy), "reduces");
        assert_eq!(read_outcome(&[&cured, &lost, &cured], &greedy), "reduces, content lost");
        assert_eq!(read_outcome(&[&cured, &same, &cured], &greedy), "no gain", "one seed's loops not fewer");
        assert_eq!(read_outcome(&[&cured, &worse, &cured], &greedy), "no gain", "decision 4 fails");
        assert_eq!(read_outcome(&[&cured, &cured], &greedy), "no gain", "two seeds");
        // Greedy too quiet at one seed to show fewer loops: underpowered, whatever the scope did.
        let quiet = reading(false, false, false, 100, READ_LOOP_FLOOR - 1);
        assert_eq!(read_outcome(&[&cured, &cured, &cured], &[&g, &quiet, &g]), "underpowered");
        let floor = reading(false, false, false, 100, READ_LOOP_FLOOR);
        assert_eq!(read_outcome(&[&cured, &cured, &cured], &[&g, &floor, &g]), "cures");
        // Seed by seed: greedy's seed 2 is far from the references (v = 0.3, a gap of 0.3), so
        // the arm's seed-2 gap of 0.3 keeps content against it, and only against it.
        let (g_far, far) = (reading(false, false, false, 60, 60), reading(true, true, false, 60, 10));
        assert_eq!(
            read_outcome(&[&kept, &far, &kept], &[&g, &g_far, &g]),
            "reduces",
            "each arm seed is held to its own greedy seed"
        );
        assert!(scopes_ok(&[ReadScope::Copy, ReadScope::All]));
        assert!(!scopes_ok(&[ReadScope::All, ReadScope::All]));
        assert!(!scopes_ok(&[ReadScope::Full]), "Full is greedy's own file");
    }

    /// claude-215: the cut is where the prompt ends in the window. Before the window slides it
    /// is the prompt's length; once the reply pushes the window's start past the prompt's start
    /// it moves down by as much; once the prompt has scrolled out it is 0.
    #[test]
    fn the_read_cut_is_where_the_prompt_ends_in_the_window() {
        assert_eq!(prompt_cut(40, 0), 40, "a short prompt: the window starts at 0");
        assert_eq!(prompt_cut(180, 10), 170, "slid by 10");
        assert_eq!(prompt_cut(180, 180), 0, "the prompt has scrolled out");
        assert_eq!(prompt_cut(180, 200), 0);
        // The decode's window is the last SHEET_CONTEXT ids: with a 180-token prompt and 30 reply
        // tokens, it starts at 18, so the prompt ends at window position 162.
        let ids = 180 + 30;
        assert_eq!(prompt_cut(180, ids - SHEET_CONTEXT), 162);
    }

    /// claude-217: the cycle step refuses exactly the token whose append closes an n-gram the
    /// reply already holds, and marks each step.
    #[test]
    fn the_cycle_step_refuses_exactly_the_token_that_closes_a_repeated_4_gram() {
        // The reply holds 5 6 7 8, and ends 5 6 7: an 8 would close 5 6 7 8 again.
        let reply = [5, 6, 7, 8, 5, 6, 7];
        assert_eq!(cycle_step_pick(&reply, &[8, 9, 10], 4), Some((9, STEP_OFF)));
        assert_eq!(cycle_step_pick(&reply, &[9, 8], 4), Some((9, STEP_NEAREST)));
        // Every candidate closes one: the nearest, marked exhausted.
        assert_eq!(cycle_step_pick(&reply, &[8], 4), Some((8, STEP_EXHAUSTED)));
        // One candidate, K = 1: always the nearest, so the rule decodes as greedy.
        for nearest in [8, 9] {
            assert_eq!(cycle_step_pick(&reply, &[nearest], 4).map(|(t, _)| t), Some(nearest));
        }
        // A reply of fewer than 3 tokens closes nothing; an overlapping cycle is caught.
        assert_eq!(cycle_step_pick(&[5, 6], &[5], 4), Some((5, STEP_NEAREST)));
        assert_eq!(cycle_step_pick(&[1, 1, 1], &[1], 4), Some((1, STEP_NEAREST)));
        assert_eq!(cycle_step_pick(&[1, 1, 1, 1], &[1, 2], 4), Some((2, STEP_OFF)));
        // The rule looks at the reply's own 4-grams only, never a 3-gram or the prompt.
        assert_eq!(cycle_step_pick(&[5, 6, 7, 9, 6, 7], &[9], 4), Some((9, STEP_NEAREST)));
        assert_eq!(cycle_step_pick(&reply, &[], 4), None);
        // The echo step, on bigrams: a token may repeat once ("very very"), not twice.
        assert_eq!(cycle_step_pick(&[3, 4], &[4], 2), Some((4, STEP_NEAREST)));
        assert_eq!(cycle_step_pick(&[3, 4, 4], &[4, 5], 2), Some((5, STEP_OFF)));
        // A bigram met earlier is refused anywhere: 5 6 then 6 … 5 cannot be followed by 6.
        assert_eq!(cycle_step_pick(&[5, 6, 9, 5], &[6, 7], 2), Some((7, STEP_OFF)));
        // On 4-grams the same replies pass: Casey's rule lets a token run four times.
        assert_eq!(cycle_step_pick(&[3, 4, 4], &[4], 4), Some((4, STEP_NEAREST)));
        assert_eq!(step_name(4), "cycle-step");
        assert_eq!(step_name(2), "echo-step");
    }

    /// claude-217: the step counts read the marks; the grade's first decision is matched on the
    /// base's and the cycle step's prompts, its second on the base's and the decode-only cut's.
    #[test]
    fn the_cycle_grade_counts_steps_and_matches_each_decision() {
        let sample = |stepped: Vec<u8>| IdSample {
            position: 0,
            prompt: vec![1],
            reply: vec![2],
            reference: vec![3],
            reply_text: String::new(),
            reference_text: String::new(),
            reply_keys: 0,
            reply_named: 0,
            reference_keys: 0,
            reference_named: 0,
            admitted: Vec::new(),
            stepped,
            keyed: None,
            served_distance: None,
        };
        let samples = [
            sample(vec![STEP_NEAREST, STEP_OFF, STEP_NEAREST, STEP_OFF]),
            sample(vec![STEP_NEAREST, STEP_EXHAUSTED]),
            sample(vec![STEP_NEAREST]),
        ];
        assert_eq!(
            step_count("p", "cycle-step", 2, &samples),
            StepCount {
                split: "p".to_owned(),
                rule: "cycle-step".to_owned(),
                seed: 2,
                steps: 7,
                stepped_off: 2,
                exhausted: 1,
                replies_stepped: 1,
            }
        );
        assert_eq!(CYCLE_RULES[..MATCHED_BY], ["greedy", "cycle-step"]);
        assert_eq!(CONFIRM_RULES[..MATCHED_BY], ["greedy", "greedy-read-all"]);
    }

    /// claude-217 (the FIT, rounds 1 and 3): an arm's content is also read on the prompts where
    /// the base, greedy, loops, against the references there. Salad where greedy looped reads as
    /// content lost, though the matched prompts, where the arm wrote greedy's replies, are kept;
    /// and a cycle-step reply must follow greedy until its first step off.
    #[test]
    fn the_cycle_step_is_read_on_the_prompts_it_changed() {
        let sample = |position: usize, reply: Vec<u32>, reference: u32, marks: Vec<u8>| IdSample {
            position,
            prompt: vec![1],
            reply,
            reference: vec![reference, 1],
            reply_text: String::new(),
            reference_text: String::new(),
            reply_keys: 0,
            reply_named: 0,
            reference_keys: 0,
            reference_named: 0,
            admitted: Vec::new(),
            stepped: marks,
            keyed: None,
            served_distance: None,
        };
        // Token 7 opens a TRAIN reply; 9 does not. The end of turn is 1. Greedy loops on prompts
        // 0 and 1 (5 6 7 8 twice) and completes prompt 2.
        let (eos, verbatim) = (1, |r: &[u32]| r.first() == Some(&7) && r.last() == Some(&1));
        let looping = vec![5, 6, 7, 8, 5, 6, 7, 8];
        let base = [
            sample(0, looping.clone(), 7, Vec::new()),
            sample(1, looping.clone(), 7, Vec::new()),
            sample(2, vec![7, 1], 9, Vec::new()),
        ];
        // The arm: salad after a step off on prompt 0, the reference on prompt 1, greedy's on 2.
        let arm = [
            sample(0, vec![5, 6, 7, 8, 5, 6, 7, 9, 1], 7, vec![0, 0, 0, 0, 0, 0, 0, STEP_OFF, 0]),
            sample(1, vec![7, 1], 7, vec![STEP_OFF, 0]),
            sample(2, vec![7, 1], 9, vec![0, 0]),
        ];
        let got = loop_content("p", "cycle-step", 1, &arm, &base, eos, verbatim).unwrap();
        assert_eq!((got.prompts, got.arm_verbatim, got.reference_verbatim, got.arm_exact), (2, 1, 2, 1));
        // Another arm's prompts are refused.
        let moved = [sample(9, vec![7, 1], 7, Vec::new()), arm[1].clone(), arm[2].clone()];
        assert!(loop_content("p", "x", 1, &moved, &base, eos, verbatim).is_err());
        // The arm follows greedy until its first step off; prompt 1 left greedy at its first step
        // with a mark, so it follows too; a reply that left with no mark does not.
        assert!(follows_greedy(&arm[..1], &base[..1], eos));
        assert!(follows_greedy(&arm, &[base[0].clone(), base[1].clone(), sample(2, vec![7, 1], 9, Vec::new())], eos));
        let unmarked = [sample(0, vec![5, 6, 9, 1], 7, vec![0, 0, 0, 0])];
        assert!(!follows_greedy(&unmarked, &base[..1], eos));
        let content = |prompts: usize, arm: usize, reference: usize| LoopContent {
            split: "p".to_owned(),
            rule: "cycle-step".to_owned(),
            seed: 1,
            prompts,
            arm_verbatim: arm,
            reference_verbatim: reference,
            arm_exact: 0,
        };
        // References 43 of 70 (p = 0.614, tolerance 0.175): 31 is kept, 30 is not, salad is not.
        assert_eq!(content(70, 31, 43).kept(), Some(true));
        assert_eq!(content(70, 30, 43).kept(), Some(false));
        assert_eq!(content(70, 5, 43).kept(), Some(false));
        assert_eq!(content(0, 0, 0).kept(), None);
        // The outcome: an arm that keeps the matched content (it wrote greedy's replies there).
        let reading = |close: bool| {
            let made = Generation {
                replies: 256,
                complete: 200,
                verbatim_train: 100,
                new_ngram_share: Vec::new(),
                distinct_1: 0.0,
                distinct_2: 0.0,
                repeated_4gram_rate: 0.0,
            };
            CellReading {
                split: "p".to_owned(),
                rule: "cycle-step".to_owned(),
                seed: 1,
                reading: VerdictSeed::new(1, &made, &made),
                holds: Some(true),
                close,
                mean_admitted: None,
                loops: Some(70),
                fewer_loops: Some(true),
                mean_length: None,
                matched_n: Some(186),
                matched_verbatim: Some(93),
            }
        };
        let (arm, closed) = (reading(false), reading(true));
        let (arms, closed) = ([&arm, &arm, &arm], [&closed, &closed, &closed]);
        let base = [&arm, &arm, &arm];
        let (salad, good, thin) = (content(70, 5, 43), content(70, 40, 43), content(LOOP_FLOOR - 1, 30, 30));
        // The FIT's scenario: matched kept, completion held, salad where it stepped.
        assert_eq!(cycle_outcome(&arms, &base, &[&salad, &good, &good]), "reduces, content lost");
        assert_eq!(cycle_outcome(&arms, &base, &[&good, &good, &good]), "improves");
        assert_eq!(cycle_outcome(&closed, &base, &[&good, &good, &good]), "cures");
        assert_eq!(cycle_outcome(&closed, &base, &[&good, &salad, &good]), "reduces, content lost");
        // Too few changed prompts, or a seed missing, is an instrument outcome, never a negative.
        assert_eq!(cycle_outcome(&arms, &base, &[&good, &thin, &good]), "underpowered (content)");
        assert_eq!(cycle_outcome(&arms, &base, &[&good, &good]), "underpowered (content)");
        // FIT round 2: where the changed prompts' references are rarely verbatim, salad would read
        // kept, so the reading cannot decide: underpowered, not improves. At 7 of 70 (p = 0.10)
        // the bound is below 0; at 14 of 70 (p = 0.20) it is 0.057, so 4 of 70 would pass.
        for (low, arm) in [(7, 0), (14, 4)] {
            let low = content(70, arm, low);
            assert_eq!(low.kept(), Some(true), "the tolerance alone would pass salad here");
            assert!(!low.powered());
            assert_eq!(cycle_outcome(&arms, &base, &[&low, &good, &good]), "underpowered (content)");
        }
        assert!(good.powered() && salad.powered() && !content(0, 0, 0).powered());
        // FIT round 4: at 18 of 70 (p = 0.257, tolerance 0.157) "kept" would accept 0.10, an arm
        // keeping 39% of the references' share; the reading needs kept to mean at least half.
        let partial = content(70, 8, 18);
        assert_eq!(partial.kept(), Some(true), "partial salad would read kept here");
        assert!(!partial.powered());
        assert_eq!(cycle_outcome(&arms, &base, &[&partial, &good, &good]), "underpowered (content)");
        // The review: each step of the order, at one seed. A base under the loop floor is
        // underpowered; an arm that does not loop less, or does not hold, is no gain; a base under
        // the matched floor is underpowered (content); and an arm that loses greedy's matched
        // content reads lost, though the looping prompts keep the references'.
        let thin_base = CellReading { loops: Some(READ_LOOP_FLOOR - 1), ..reading(false) };
        assert_eq!(cycle_outcome(&arms, &[&arm, &thin_base, &arm], &[&good, &good, &good]), "underpowered");
        let unheld = CellReading { holds: Some(false), ..reading(false) };
        assert_eq!(cycle_outcome(&[&arm, &unheld, &arm], &base, &[&good, &good, &good]), "no gain");
        let more = CellReading { fewer_loops: Some(false), ..reading(false) };
        assert_eq!(cycle_outcome(&[&arm, &arm, &more], &base, &[&good, &good, &good]), "no gain");
        let unmatched = CellReading { matched_n: Some(MATCHED_FLOOR - 1), ..reading(false) };
        assert_eq!(cycle_outcome(&arms, &[&arm, &arm, &unmatched], &[&good, &good, &good]), "underpowered (content)");
        let lost = CellReading { matched_verbatim: Some(40), ..reading(true) };
        assert_eq!(cycle_outcome(&[closed[0], &lost, closed[0]], &base, &[&good, &good, &good]), "reduces, content lost");
    }

    /// claude-217: the sheet's "what ships" table, row by row.
    #[test]
    fn the_grade_ships_what_the_table_says() {
        assert_eq!(ships("improves", "no gain", "improves"), "cycle-step");
        assert_eq!(ships("reduces, content lost", "cures", "cures"), "decode-only cut");
        assert_eq!(ships("cures", "improves", "improves"), "both");
        assert_eq!(ships("improves", "cures", "reduces, content lost"), "cycle-step");
        assert_eq!(ships("underpowered", "underpowered (content)", "cures"), "greedy");
    }

    /// claude-216: the matched content counts only the prompts every rule completes, so one
    /// rule's completions select nothing; an arm keeps the base's matched content within
    /// `max(0.10, 3·√(p(1−p)/m))`; and the cut's outcome follows the sheet's order.
    #[test]
    fn the_cut_grade_matches_content_and_follows_the_sheet() {
        // Three rules over four prompts: prompt 1 is incomplete under rule 2, prompt 3 under 0.
        let flags = vec![
            vec![(true, true), (true, true), (true, false), (false, false)],
            vec![(true, false), (true, true), (true, true), (true, true)],
            vec![(true, true), (false, false), (true, true), (true, true)],
        ];
        assert_eq!(matched_counts(&flags, 3), (2, vec![1, 1, 2]));
        // Taken over the first two rules only, rule 2's incomplete prompt 1 selects nothing.
        assert_eq!(matched_counts(&flags, 2), (3, vec![2, 2, 2]));
        assert_eq!(matched_counts(&[], 3), (0, vec![]));
        let reading = |fewer: bool, holds: bool, close: bool, loops: usize, verbatim: usize| {
            let made = Generation {
                replies: 256,
                complete: 200,
                verbatim_train: 100,
                new_ngram_share: Vec::new(),
                distinct_1: 0.0,
                distinct_2: 0.0,
                repeated_4gram_rate: 0.0,
            };
            CellReading {
                split: "p-t".to_owned(),
                rule: "r-cut/greedy-read-all".to_owned(),
                seed: 1,
                reading: VerdictSeed::new(1, &made, &made),
                holds: Some(holds),
                close,
                mean_admitted: None,
                loops: Some(loops),
                fewer_loops: Some(fewer),
                mean_length: None,
                matched_n: Some(200),
                matched_verbatim: Some(verbatim),
            }
        };
        // The base: 120 of 200 matched prompts verbatim, p = 0.6, so the tolerance is 0.104.
        let g = reading(false, false, false, 60, 120);
        let base = [&g, &g, &g];
        let (kept, lost) = (reading(true, true, false, 10, 100), reading(true, true, false, 10, 98));
        assert!(matched_kept(&kept, &g), "0.50 is within 0.104 of 0.60");
        assert!(!matched_kept(&lost, &g), "0.49 is not");
        let cured = reading(true, true, true, 10, 120);
        let same = reading(false, true, true, 60, 120);
        use OverturnCell::{Behind, Inside};
        assert_eq!(cut_outcome(&[&cured, &cured, &cured], &base, Inside), "cures");
        assert_eq!(cut_outcome(&[&kept, &kept, &kept], &base, Inside), "improves");
        assert_eq!(cut_outcome(&[&kept, &lost, &kept], &base, Inside), "reduces, content lost");
        assert_eq!(cut_outcome(&[&cured, &cured, &cured], &base, Behind), "costs accuracy");
        assert_eq!(cut_outcome(&[&cured, &same, &cured], &base, Inside), "no gain");
        let quiet = reading(false, false, false, READ_LOOP_FLOOR - 1, 120);
        assert_eq!(cut_outcome(&[&cured, &cured, &cured], &[&g, &quiet, &g], Inside), "underpowered");
        // Close on its own completions, but the matched content lost: not cured (the FIT).
        let close_lost = reading(true, true, true, 10, 98);
        assert_eq!(
            cut_outcome(&[&close_lost, &close_lost, &close_lost], &base, Inside),
            "reduces, content lost"
        );
        // Too few matched prompts, or none counted, is an instrument outcome, never a negative.
        let thin = CellReading { matched_n: Some(MATCHED_FLOOR - 1), ..g.clone() };
        let unmatched = CellReading { matched_n: None, ..g.clone() };
        for weak in [&thin, &unmatched] {
            assert_eq!(
                cut_outcome(&[&lost, &lost, &lost], &[&g, weak, &g], Inside),
                "underpowered (content)"
            );
        }
        // At the floor, at every seed, the content decides: 38 of 64 (p = 0.59, tolerance 0.18)
        // against 0.30 is lost, and against 0.50 kept.
        let at_floor = CellReading { matched_n: Some(MATCHED_FLOOR), matched_verbatim: Some(38), ..g.clone() };
        let floor_base = [&at_floor, &at_floor, &at_floor];
        let far = reading(true, true, false, 10, 60);
        assert_eq!(cut_outcome(&[&far, &far, &far], &floor_base, Inside), "reduces, content lost");
        assert_eq!(cut_outcome(&[&kept, &kept, &kept], &floor_base, Inside), "improves");
        // The matched set is the base's and the served arm's: the first two rules the grade reads.
        assert_eq!(CUT_RULES[..MATCHED_BY], ["greedy", "r-cut/greedy-read-all"]);
    }

    /// claude-216: in training and scoring, a row whose input is a reply token reads only before
    /// its reply's first token in the window, or nothing when the reply began before the window;
    /// every other row, the reply's first scored row (the prompt's last token) among them, keeps
    /// every earlier position; a reply never crosses a window.
    #[test]
    fn every_reply_row_reads_only_before_its_reply() {
        let context = 10;
        // Window 0: prompt 0–2, reply 3–5, prompt 6–7, reply 8–9. Window 1: a reply carried in
        // at 0–1, prompt 2–4, reply 5–6, prompt 7–9.
        let flags = [0, 0, 0, 1, 1, 1, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0];
        let want = [10, 10, 10, 3, 3, 3, 10, 10, 8, 8, 0, 0, 10, 10, 10, 5, 5, 10, 10, 10];
        assert_eq!(reply_row_limits(&flags, context), want);
        assert_eq!(reply_row_limits(&[], context), Vec::<usize>::new());
    }

    /// claude-216 (the FIT's first point): a window that ends a long conversation carries up to
    /// a context of earlier tokens, earlier replies among them, so a decode prompt can hold an
    /// earlier reply, which training limits. Each window's input flags come from the split's
    /// mask ([`window_input_mask`]), and a decode window ([`decode_window_mask`]) is limited
    /// exactly as the training window holding the same tokens: its earlier replies to before
    /// themselves, the reply decoded to [`prompt_cut`]. claude-215's uniform cut, `s < cut` on
    /// every row, would leave the earlier reply reading itself.
    #[test]
    fn a_decode_window_is_limited_as_its_training_window() {
        // Two conversations of five turns, each longer than a window. The first's second window
        // opens inside turn 3's reply; the second's, at 108, inside turn 2's.
        let (mut tokens, mut mask) = (Vec::new(), Vec::new());
        for (prompt, reply) in [(30, 45), (40, 20)] {
            tokens.push(hs::SHEET_BOS_ID);
            mask.push(0u8);
            for _ in 0..5 {
                tokens.extend(std::iter::repeat_n(7u32, prompt + reply));
                mask.extend(std::iter::repeat_n(0u8, prompt));
                mask.extend(std::iter::repeat_n(1u8, reply));
            }
        }
        let (windows, _) =
            hs::build_document_windows(&tokens, &mask, SHEET_CONTEXT, hs::SHEET_BOS_ID).unwrap();
        assert_eq!(windows.len(), 4);
        let flags_of = |w: &CausalWindow| window_input_mask(std::slice::from_ref(&w), &mask).unwrap();
        // The second conversation's second window: turn 2's reply carried in at 0–12, turn 3's
        // prompt, its reply at 53–72, turn 4's prompt, and the reply the sample opens at 113.
        let flags = flags_of(&windows[3]);
        assert!(flags[..13].iter().all(|&f| f == 1) && flags[13] == 0);
        assert!(flags[52] == 0 && flags[53] == 1 && flags[72] == 1 && flags[73] == 0);
        assert!(flags[112] == 0 && flags[113] == 1);
        let limits = reply_row_limits(&flags, SHEET_CONTEXT);
        assert_eq!((limits[5], limits[60], limits[52], limits[112]), (0, 53, SHEET_CONTEXT, SHEET_CONTEXT));
        // Every sampled prompt opens a reply; its decode windows are limited as training's.
        // What each row may read: `first ≤ s < min(t, limit)` (`flock_read_hamming::is_candidate`).
        let reach = |limits: &[usize]| -> Vec<usize> {
            limits.iter().enumerate().map(|(t, &l)| t.min(l)).collect()
        };
        let sampled = sample_windows(&windows, &mask, 16);
        assert_eq!(sampled.len(), 3, "both first windows, and the second one's second");
        let mut held_earlier = false;
        for &window in &sampled {
            let off = window.scored_offsets[0];
            let flags = flags_of(window);
            let train = reply_row_limits(&flags, SHEET_CONTEXT);
            // Within the reference reply, which is 20 tokens or more, the windows match.
            for reply_len in [0, 1, 2, 20] {
                let decode = decode_window_mask(&flags[..=off], reply_len, 0);
                let width = off + 1 + reply_len;
                assert_eq!(decode, flags[..width], "offset {off}, {reply_len} tokens");
                assert_eq!(
                    reach(&reply_row_limits(&decode, decode.len())),
                    reach(&train[..width]),
                    "offset {off}, {reply_len} tokens"
                );
            }
            // The earlier reply in this prompt reads only before itself; a uniform cut would
            // leave it every earlier position.
            held_earlier |= (0..=off).any(|t| train[t] < t);
        }
        assert!(held_earlier, "no sampled prompt held an earlier reply, so the check was vacuous");
        // Once the window slides, a reply carried in reads nothing and the reply decoded reads
        // the prompt's positions, claude-215's cut.
        let prompt: Vec<u8> = [vec![0; 50], vec![1; 30], vec![0; 100]].concat();
        let (reply_len, start) = (64, 180 + 64 - SHEET_CONTEXT);
        let slid = decode_window_mask(&prompt, reply_len, start);
        let limits = reply_row_limits(&slid, slid.len());
        assert_eq!(slid.len(), SHEET_CONTEXT);
        assert!(limits[..28].iter().all(|&l| l == 0));
        assert!(limits[28..128].iter().all(|&l| l == SHEET_CONTEXT));
        assert!(limits[128..].iter().all(|&l| l == prompt_cut(prompt.len(), start)));
        // The census of the four windows: 101 + 132 + 60 + 72 reply rows, of which the 43 and
        // 13 carried in read nothing.
        assert_eq!(cut_census(&windows, &mask, SHEET_CONTEXT).unwrap(), (4 * SHEET_CONTEXT, 365, 56));
    }

    /// claude-216: a run that trains without the cut writes no `reply_read_train` key, so its
    /// `config.json` is as before; one that trains with it records the scope, and scoring reads
    /// it back from the run beside a `step-N` checkpoint.
    #[test]
    fn the_trained_cut_is_recorded_and_read_back() {
        let mut config = super::tests::config(0, None);
        let plain = serde_json::to_string(&config).unwrap();
        assert!(!plain.contains("reply_read_train"), "{plain}");
        config.reply_read_train = Some(ReadScope::All);
        let cut = serde_json::to_vec_pretty(&config).unwrap();
        assert!(String::from_utf8_lossy(&cut).contains("\"reply_read_train\": \"all\""));
        let run = std::env::temp_dir().join(format!("claude-216-cut-{}", std::process::id()));
        let step = run.join("models").join("step-000256");
        fs::create_dir_all(&step).unwrap();
        fs::write(run.join("config.json"), &cut).unwrap();
        assert_eq!(run_trained_cut(&step).unwrap(), Some(ReadScope::All));
        assert!(!run_train_as_served(&step, None).unwrap());
        config.train_as_served = true;
        let served = serde_json::to_vec_pretty(&config).unwrap();
        fs::write(run.join("config.json"), &served).unwrap();
        assert!(run_train_as_served(&step, None).unwrap());
        assert!(run_train_as_served(&step, Some(true)).unwrap());
        assert_eq!(run_trained_cut(&step).unwrap(), Some(ReadScope::All));
        fs::write(run.join("config.json"), plain.as_bytes()).unwrap();
        assert_eq!(run_trained_cut(&step).unwrap(), None);
        assert!(!run_train_as_served(&step, None).unwrap());
        assert!(run_train_as_served(&step, Some(true)).unwrap());
        assert!(!run_train_as_served(&step, Some(false)).unwrap());
        fs::remove_dir_all(&run).unwrap();
    }

    /// claude-215 (its GPU half): on a model with the Hamming reads set, a reply's first token is
    /// the same under every read scope (its window is all prompt, so the cut limits nothing),
    /// and the cut (a row cut since claude-216) is cleared after every decode. New reads start
    /// with zero weights, so the scopes cannot change a token here: `a_cut_keeps_every_pick_before_it` pins the cut itself,
    /// and the pod's job checks that the scoped decodes of the trained checkpoints differ.
    #[cfg(feature = "cuda")]
    #[test]
    fn the_read_scope_starts_as_greedy_and_clears_its_cut() {
        let device = Device::new_cuda(0).unwrap();
        let model = build_fbs_model(FbsArm::OneAtomActive, 1, &device).unwrap();
        crate::flock_read_hamming::set_hamming_reads(Some(
            crate::flock_read_hamming::HammingReads::new(4, hs::SHEET_WIDTH, &device).unwrap(),
        ));
        let mut decoded = 0;
        for sample in 0..4usize {
            let shift = sample as u32 * 101;
            let prompt: Vec<u32> = (1..=24u32).map(|i| (i * 37 + shift) % 4096).collect();
            let decode = |scope| {
                let reply = decode_reply_with_mode(
                    &model,
                    PointObjective::Cell,
                    &prompt,
                    16,
                    1,
                    DecodeRule::Greedy,
                    sample,
                    None,
                    scope,
                    &vec![0; prompt.len()],
                    false,
                )
                .unwrap()
                .tokens;
                assert_eq!(crate::flock_read_hamming::read_cut(), None, "the cut outlived its decode");
                assert!(!crate::flock_read_hamming::row_cut_set(), "the row cut outlived its decode");
                reply
            };
            let full = decode(ReadScope::Full);
            for scope in [ReadScope::Copy, ReadScope::All] {
                let scoped = decode(scope);
                assert_eq!(scoped.first(), full.first(), "sample {sample}, {scope:?}");
            }
            decoded += full.len();
        }
        crate::flock_read_hamming::set_hamming_reads(None);
        assert!(decoded > 0, "every reply was empty, so the check held vacuously");
    }

    /// Exact-length batching keeps complete decode results equal to one-at-a-time decoding,
    /// including the Flock draw's original sample index and early termination.
    #[cfg(feature = "cuda")]
    #[test]
    fn exact_length_batches_match_one_at_a_time_decodes() -> Result<()> {
        let device = Device::new_cuda(0)?;
        let model = build_fbs_model(FbsArm::OneAtomActive, 1, &device)?;
        crate::flock_read_hamming::set_hamming_reads(Some(
            crate::flock_read_hamming::HammingReads::new(4, hs::SHEET_WIDTH, &device)?,
        ));
        let lengths = [17usize, 18, 18, 19, 32, 32];
        let prompts: Vec<Vec<u32>> = lengths
            .iter()
            .enumerate()
            .map(|(seed, &len)| {
                std::iter::once(hs::SHEET_BOS_ID)
                    .chain(
                        (1..len)
                            .map(|i| ((i * 37 + seed * 101) as u32) % hs::SHEET_VOCAB as u32),
                    )
                    .collect()
            })
            .collect();
        let masks: Vec<Vec<u8>> = prompts.iter().map(|prompt| vec![0; prompt.len()]).collect();
        let openings: std::collections::HashMap<Vec<u32>, u32> = prompts
            .iter()
            .enumerate()
            .map(|(seed, prompt)| (prompt[1..3].to_vec(), 100 + seed as u32))
            .collect();
        crate::flock_read_memory::set_turn_memory(Some(
            crate::flock_read_memory::TurnMemory::new(2, openings, hs::SHEET_WIDTH, &device)?
                .with_reply(std::collections::HashMap::new())?,
        ));
        let rules = [
            DecodeRule::Greedy,
            DecodeRule::FlockRank { k: 1, keep_end: false },
            DecodeRule::FlockRank { k: 16, keep_end: true },
            DecodeRule::flock_cell(CELL_MARGIN_MILLI),
            DecodeRule::CYCLE_STEP,
        ];
        let mut decoded = 0usize;
        for rule in rules {
            let serial: Vec<DecodedReply> = prompts
                .iter()
                .zip(&masks)
                .enumerate()
                .map(|(sample, (prompt, mask))| {
                    decode_reply_with_mode(
                        &model,
                        PointObjective::Cell,
                        prompt,
                        16,
                        17,
                        rule,
                        sample,
                        None,
                        ReadScope::Full,
                        mask,
                        false,
                    )
                })
                .collect::<Result<_>>()?;
            let batched = decode_replies_with_mode(
                &model,
                PointObjective::Cell,
                &prompts,
                &masks,
                16,
                17,
                rule,
                None,
                ReadScope::Full,
                false,
                true,
            )?;
            assert_eq!(serial.len(), batched.len());
            for (sample, (serial, batched)) in serial.iter().zip(&batched).enumerate() {
                assert_eq!(serial.tokens, batched.tokens, "{rule:?}, sample {sample}");
                assert_eq!(serial.admitted, batched.admitted, "{rule:?}, sample {sample}");
                assert_eq!(serial.stepped, batched.stepped, "{rule:?}, sample {sample}");
                assert_eq!(serial.served, batched.served, "{rule:?}, sample {sample}");
                decoded += batched.tokens.len();
            }
        }
        let firsts: Vec<u32> = prompts
            .iter()
            .zip(&masks)
            .enumerate()
            .map(|(sample, (prompt, mask))| {
                decode_reply_with_mode(
                    &model,
                    PointObjective::Cell,
                    prompt,
                    1,
                    17,
                    DecodeRule::Greedy,
                    sample,
                    None,
                    ReadScope::Full,
                    mask,
                    false,
                )
                .and_then(|reply| {
                    reply.tokens.first().copied().ok_or_else(|| {
                        invalid("an early-termination control decoded no first token")
                    })
                })
            })
            .collect::<Result<_>>()?;
        let pair = (0..prompts.len())
            .find_map(|first| {
                (first + 1..prompts.len())
                    .find(|&other| {
                        prompts[other].len() == prompts[first].len()
                            && firsts[other] != firsts[first]
                    })
                    .map(|other| (first, other))
            })
            .ok_or_else(|| invalid("no equal-length pair has different first tokens"))?;
        let selected = vec![prompts[pair.0].clone(), prompts[pair.1].clone()];
        let selected_masks = vec![masks[pair.0].clone(), masks[pair.1].clone()];
        let early = DecodeRule::FlockRank { k: 1, keep_end: true };
        let serial: Vec<DecodedReply> = selected
            .iter()
            .zip(&selected_masks)
            .enumerate()
            .map(|(sample, (prompt, mask))| {
                decode_reply_with_mode(
                    &model,
                    PointObjective::Cell,
                    prompt,
                    16,
                    17,
                    early,
                    sample,
                    Some(firsts[pair.0]),
                    ReadScope::Full,
                    mask,
                    false,
                )
            })
            .collect::<Result<_>>()?;
        let batched = decode_replies_with_mode(
            &model,
            PointObjective::Cell,
            &selected,
            &selected_masks,
            16,
            17,
            early,
            Some(firsts[pair.0]),
            ReadScope::Full,
            false,
            true,
        )?;
        assert_eq!(serial[0].tokens, batched[0].tokens);
        assert_eq!(serial[1].tokens, batched[1].tokens);
        assert_eq!(serial[0].tokens.first(), Some(&firsts[pair.0]));
        assert!(serial[0].tokens.len() < 16, "the first row must stop early");
        assert!(
            serial[1].tokens.len() > 1,
            "the second row must continue after the first stops"
        );
        crate::flock_read_hamming::set_hamming_reads(None);
        crate::flock_read_memory::set_turn_memory(None);
        assert!(decoded > 0, "every reply was empty, so the parity check was vacuous");
        Ok(())
    }

    /// claude-279's decode switch: a random model with P2b's two Hamming reads (random
    /// parameters) and the turn memory (openings, and a reply table the decodes hit) decodes the
    /// same replies with the step cache on and off, at a 16-position window, plain and served.
    /// The groups of 5, 9 and 12 prompt positions prefill, step, meet the slide and re-run the
    /// whole window from there; the 3-position prompt (shorter than the convolution) and the
    /// 20-position one (past the window) re-run it at every step.
    #[cfg(feature = "cuda")]
    #[test]
    fn the_decode_cache_decodes_the_same_replies_through_a_window_slide() -> Result<()> {
        let device = Device::new_cuda(0)?;
        let mut model = build_fbs_model(FbsArm::OneAtomActive, 1, &device)?;
        model.set_context(16)?;
        // The arm the scorer sets on every pass (`score_rows`); the guard clears it and the reads.
        arms::set_fbs_arm(Some(FbsArm::OneAtomActive));
        let _guard = ArmGuard;
        let width = hs::SHEET_WIDTH;
        crate::flock_read_hamming::set_hamming_reads(Some(
            crate::flock_read_hamming::HammingReads::new(4, width, &device)?.randomized(2793)?,
        ));
        let prompts: Vec<Vec<u32>> = [3usize, 5, 5, 9, 12, 20]
            .iter()
            .enumerate()
            .map(|(seed, &len)| {
                std::iter::once(hs::SHEET_BOS_ID)
                    .chain(
                        (1..len)
                            .map(|i| ((i * 37 + seed * 101) as u32) % hs::SHEET_VOCAB as u32),
                    )
                    .collect()
            })
            .collect();
        let masks: Vec<Vec<u8>> = prompts.iter().map(|prompt| vec![0; prompt.len()]).collect();
        let openings: std::collections::HashMap<Vec<u32>, u32> = prompts
            .iter()
            .enumerate()
            .map(|(seed, prompt)| (prompt[1..3].to_vec(), 100 + seed as u32))
            .collect();
        let remember = |replies: std::collections::HashMap<Vec<u32>, u32>| -> Result<()> {
            let memory =
                crate::flock_read_memory::TurnMemory::new(2, openings.clone(), width, &device)?
                    .with_reply(replies)?;
            memory.weight.set(&Tensor::full(0.25f32, width, &device)?)?;
            if let Some(reply) = memory.reply.as_ref() {
                reply.weight.set(&Tensor::full(0.5f32, width, &device)?)?;
            }
            crate::flock_read_memory::set_turn_memory(Some(memory));
            Ok(())
        };
        let decode = |rule: DecodeRule, served: bool, cached: bool| {
            set_decode_cache(cached);
            let replies = decode_replies_with_mode(
                &model,
                PointObjective::Cell,
                &prompts,
                &masks,
                14,
                17,
                rule,
                None,
                ReadScope::Full,
                served,
                true,
            );
            set_decode_cache(true);
            (replies, take_decode_forwards())
        };
        // The reply table holds each row's key at its first reply step, read off a decode with an
        // empty table, so the decodes below hit it there.
        remember(std::collections::HashMap::new())?;
        take_decode_forwards();
        let (first, _) = decode(DecodeRule::Greedy, false, false);
        let keys: std::collections::HashMap<Vec<u32>, u32> = prompts
            .iter()
            .zip(first?)
            .filter_map(|(prompt, reply)| {
                let token = *reply.tokens.first()?;
                let ids: Vec<u32> = prompt.iter().copied().chain([token]).collect();
                let start = ids.len().saturating_sub(16);
                let cut = prompt_cut(prompt.len(), start);
                (cut > 0).then(|| greedy_reply_key(&ids[start..], cut - 1, &[token]))?
            })
            .map(|key| (key, 7))
            .collect();
        assert!(!keys.is_empty(), "no reply key for the decodes to hit");
        remember(keys)?;
        let rules = [
            DecodeRule::Greedy,
            DecodeRule::FlockRank { k: 16, keep_end: true },
            DecodeRule::flock_cell(CELL_MARGIN_MILLI),
            DecodeRule::CYCLE_STEP,
        ];
        let (mut decoded, mut stepped, mut prefilled) = (0usize, 0u64, 0u64);
        for rule in rules {
            for served in [false, true] {
                let (whole, off) = decode(rule, served, false);
                let (cached, on) = decode(rule, served, true);
                let (whole, cached) = (whole?, cached?);
                let case = format!("{rule:?}, served {served}");
                assert_eq!((off.prefills, off.steps), (0, 0), "{case}: the cache ran while off");
                assert_eq!(
                    on.prefills + on.steps + on.whole,
                    off.whole,
                    "{case}: one forward per decode step either way"
                );
                assert!(on.whole > 0 && on.whole < off.whole, "{case}: {on:?} against {off:?}");
                // claude-281: the 3-position prompt's prefill and the slides are refusals, named.
                assert_eq!(
                    off.first_refusal, None,
                    "{case}: a refusal with the cache off"
                );
                assert!(
                    matches!(on.first_refusal, Some("short prompt" | "window full")),
                    "{case}: {on:?}"
                );
                for (sample, (whole, cached)) in whole.iter().zip(&cached).enumerate() {
                    assert_eq!(whole.tokens, cached.tokens, "{case}, sample {sample}");
                    assert_eq!(whole.admitted, cached.admitted, "{case}, sample {sample}");
                    assert_eq!(whole.stepped, cached.stepped, "{case}, sample {sample}");
                    decoded += cached.tokens.len();
                }
                (stepped, prefilled) = (stepped + on.steps, prefilled + on.prefills);
            }
        }
        // claude-281: a row cut left on the thread neither refuses the prefill nor changes a
        // reply; the decode clears it before it asks. Every forward clears the cut after it, so
        // the cut meets only the first group's first step: the 3-position prompt is left out, as
        // its prefill is refused as a short prompt before the cut is asked about.
        let decode_cut = || {
            set_decode_cache(true);
            let replies = decode_replies_with_mode(
                &model,
                PointObjective::Cell,
                &prompts[1..],
                &masks[1..],
                14,
                17,
                DecodeRule::Greedy,
                None,
                ReadScope::Full,
                false,
                true,
            );
            (replies, take_decode_forwards())
        };
        let (uncut, clear) = decode_cut();
        crate::flock_read_hamming::set_row_cut(Some((
            crate::flock_read_hamming::ReadScope::Copy,
            vec![2; 4],
        )));
        let (cut, left) = decode_cut();
        assert!(!crate::flock_read_hamming::row_cut_set(), "the row cut outlived its decode");
        assert_eq!(left, clear, "a row cut left on the thread changed the forwards");
        assert!(left.prefills > 0, "a row cut left on the thread refused the prefill: {left:?}");
        for (sample, (uncut, cut)) in uncut?.iter().zip(&cut?).enumerate() {
            assert_eq!(uncut.tokens, cut.tokens, "row cut, sample {sample}");
        }
        crate::flock_read_hamming::set_hamming_reads(None);
        crate::flock_read_memory::set_turn_memory(None);
        assert!(decoded > 0, "every reply was empty, so the parity check was vacuous");
        assert!(stepped > 0 && prefilled > 0, "the cache never prefilled or stepped");
        Ok(())
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn served_writer_decode_rounds_the_hidden_point_to_q12() -> Result<()> {
        let device = Device::new_cuda(0).unwrap();
        let model = build_fbs_model(FbsArm::OneAtomActive, 1, &device)?;
        let prompt: Vec<u32> = (1..=24u32).map(|i| (i * 37) % 4096).collect();
        let hidden =
            hs::with_served_surrogate_training(1, 0, || model.hidden(&prompt, 1, prompt.len()))?;
        let expected = native_writer_next_token(&model, &hidden, true)?;
        let decoded = decode_reply_with_mode(
            &model,
            PointObjective::Sheet,
            &prompt,
            1,
            1,
            DecodeRule::Greedy,
            0,
            None,
            ReadScope::Full,
            &[],
            true,
        )?;
        if expected == hs::SHEET_BOS_ID {
            assert!(decoded.tokens.is_empty(), "BOS ends the reply");
        } else {
            assert_eq!(
                decoded.tokens.first(),
                Some(&expected),
                "the served writer takes the Q12 point"
            );
        }
        Ok(())
    }

    /// claude-213 (its GPU half): on a model, the cell rule at margin 0 decodes exactly as
    /// greedy, admitting one candidate at every step, and records a count at each step; the
    /// objective's own margin admits more than one somewhere, so its draws do run.
    #[cfg(feature = "cuda")]
    #[test]
    fn the_cell_rule_at_margin_0_decodes_as_greedy_on_a_model() {
        let device = Device::new_cuda(0).unwrap();
        let model = build_fbs_model(FbsArm::OneAtomActive, 1, &device).unwrap();
        let (mut decoded, mut wider) = (0, false);
        for sample in 0..4usize {
            let shift = sample as u32 * 101;
            let prompt: Vec<u32> = (1..=24u32).map(|i| (i * 37 + shift) % 4096).collect();
            let decode = |rule| {
                decode_reply_with_mode(
                    &model,
                    PointObjective::Cell,
                    &prompt,
                    16,
                    1,
                    rule,
                    sample,
                    None,
                    ReadScope::Full,
                    &[],
                    false,
                )
                .unwrap()
            };
            let greedy = decode(DecodeRule::Greedy);
            assert!(greedy.admitted.is_empty(), "greedy records no counts");
            let cell = decode(DecodeRule::flock_cell(0));
            assert_eq!(cell.tokens, greedy.tokens, "sample {sample}");
            assert!(cell.admitted.len() >= cell.tokens.len());
            assert!(cell.admitted.iter().all(|&n| n == 1), "sample {sample}: {:?}", cell.admitted);
            wider |= decode(DecodeRule::flock_cell(CELL_MARGIN_MILLI)).admitted.iter().any(|&n| n > 1);
            decoded += greedy.tokens.len();
        }
        assert!(decoded > 0, "every reply was empty, so the check held vacuously");
        assert!(wider, "the objective's margin never admitted a second candidate");
    }

    /// claude-211 (I-C): the frozen check reads safetensors bytes, passes a run whose routing
    /// tensor stayed while another moved, and refuses a moved routing tensor or a run that did
    /// not move at all.
    #[test]
    fn the_frozen_check_compares_routing_tensor_bytes() {
        let file = |route: f32, other: f32| -> Vec<u8> {
            let header = r#"{"a.spiralcore_fbs":{"dtype":"F32","shape":[1],"data_offsets":[0,4]},"b.weight":{"dtype":"F32","shape":[1],"data_offsets":[4,8]}}"#;
            let mut bytes = (header.len() as u64).to_le_bytes().to_vec();
            bytes.extend_from_slice(header.as_bytes());
            bytes.extend_from_slice(&route.to_le_bytes());
            bytes.extend_from_slice(&other.to_le_bytes());
            bytes
        };
        let run = |name: &str, end: (f32, f32)| {
            let dir = std::env::temp_dir().join(format!("claude-211-frozen-{name}-{}", std::process::id()));
            for (step, (r, o)) in [("step-000000", (1.0, 1.0)), ("step-000256", end)] {
                let at = dir.join("models").join(step);
                fs::create_dir_all(&at).unwrap();
                fs::write(at.join("model.safetensors"), file(r, o)).unwrap();
            }
            dir
        };
        let (kept, moved, still) = (run("kept", (1.0, 2.0)), run("moved", (3.0, 2.0)), run("still", (1.0, 1.0)));
        assert_eq!(frozen_check(&kept).unwrap(), (1, 1));
        assert!(frozen_check(&moved).is_err(), "a routing tensor moved under frozen");
        assert!(frozen_check(&still).is_err(), "nothing trained, so the check proves nothing");
        for dir in [kept, moved, still] {
            fs::remove_dir_all(dir).unwrap();
        }
    }

    /// claude-211 (I-G): the strata split by whether (T) names a row; a position scored twice,
    /// or a file with the memory on, is refused.
    #[test]
    fn the_strata_split_by_the_tables_control() {
        let rows = |split: &str, positions: Vec<usize>, picks: Vec<u32>| ScoredRows {
            split: split.to_owned(),
            memory: false,
            targets: vec![1; positions.len()],
            writer: picks.clone(),
            nearest: picks,
            positions,
        };
        let dev = rows("dev", vec![5, 6, 7], vec![1, CONTROL_MISS, 2]);
        let p_dev = rows("p-dev", vec![1, 2], vec![CONTROL_MISS, CONTROL_MISS]);
        let strata = Strata::from_tables(&dev, &p_dev).unwrap();
        assert_eq!(strata.counts(), [2, 1, 2, 0]);
        assert!(strata.u.contains(&6) && strata.k.contains(&5) && strata.k.contains(&7));
        let twice = rows("dev", vec![5, 5], vec![1, CONTROL_MISS]);
        assert!(Strata::from_tables(&twice, &p_dev).is_err());
        let memory_on = ScoredRows {
            memory: true,
            ..dev.clone()
        };
        assert!(Strata::from_tables(&memory_on, &p_dev).is_err());
    }

    /// claude-211 (I-G): a cell is ahead or behind only when all three seeds are, each past
    /// `max(⌈2·√(b+c)⌉, max d − min d)`; two seeds, or one inside, leave it inside.
    #[test]
    fn a_grade_cell_needs_all_three_seeds_past_the_band() {
        // b + c = 100, so each floor is 20; d = 40, 42, 44, a spread of 4.
        let ahead = [
            (1u8, (0, 0, 70, 30)),
            (2, (0, 0, 71, 29)),
            (3, (0, 0, 72, 28)),
        ];
        assert_eq!(grade_cell("x", "y", "U", &ahead).cell, OverturnCell::Ahead);
        let behind = ahead.map(|(s, (a, b, x, y))| (s, (b, a, y, x)));
        assert_eq!(
            grade_cell("y", "x", "U", &behind).cell,
            OverturnCell::Behind
        );
        assert_eq!(
            grade_cell("x", "y", "U", &ahead[..2]).cell,
            OverturnCell::Inside
        );
        // d = 60, 62, 30: each past its floor of 20, but the spread of 32 is the band.
        let spread = [
            (1u8, (0, 0, 80, 20)),
            (2, (0, 0, 81, 19)),
            (3, (0, 0, 65, 35)),
        ];
        let cell = grade_cell("x", "y", "U", &spread);
        assert_eq!((cell.spread, cell.seeds[0].band), (32, 32));
        assert_eq!(cell.cell, OverturnCell::Inside);
    }

    /// claude-211 (I-G): decision 1's four rules in the protocol's order, and its default.
    #[test]
    fn the_base_recipe_follows_the_protocols_order() {
        use OverturnCell::{Ahead, Behind, Inside};
        // (r-drop)'s cells against (r), on U then on N-u.
        assert_eq!(base_recipe(Inside, Ahead).0, "r-drop");
        assert_eq!(base_recipe(Inside, Behind).0, "r");
        assert_eq!(
            base_recipe(Ahead, Inside).0,
            "r-drop",
            "U decides when N-u is inside"
        );
        assert_eq!(base_recipe(Behind, Inside).0, "r");
        // Opposite leads on the two strata: no rule holds, so the default.
        assert_eq!(base_recipe(Behind, Ahead).1, base_recipe(Inside, Inside).1);
        assert_eq!(base_recipe(Ahead, Behind).1, base_recipe(Inside, Inside).1);
        assert_eq!(base_recipe(Inside, Inside).0, "r");
    }

    /// claude-211 (I-G): decisions 2 and 3's readings.
    #[test]
    fn the_add_on_and_transport_readings_follow_the_protocol() {
        use OverturnCell::{Ahead, Behind, Inside};
        assert_eq!(addon_reading(Inside, Ahead), "a candidate");
        assert_eq!(addon_reading(Behind, Ahead), "no decision (underpowered)");
        assert_eq!(
            addon_reading(Ahead, Behind),
            "a negative for this configuration"
        );
        assert_eq!(addon_reading(Ahead, Inside), "no decision (underpowered)");
        assert_eq!(
            transport_reading(Behind, Behind),
            "learned transport earns its place"
        );
        assert_eq!(
            transport_reading(Behind, Inside),
            "no decision (underpowered)"
        );
        let negative = "a negative for learned transport in this configuration";
        assert_eq!(transport_reading(Ahead, Behind), negative);
        assert_eq!(transport_reading(Inside, Ahead), negative);
    }

    /// claude-211 (I-G): decision 4's count band is `⌈2·√(n·p(1−p))⌉` exactly, and the Flock
    /// rank rule holds only with a lower repeat rate and within the band.
    #[test]
    fn the_decode_rule_band_is_exact() {
        // p = 1/2 at n = 512: 2·√128 = 22.6, so 23. p = 1/4: 2·√96 = 19.6, so 20.
        assert_eq!(count_band(256, 512), 23);
        assert_eq!(count_band(128, 512), 20);
        assert_eq!((count_band(0, 512), count_band(512, 512)), (0, 0));
        assert!(decode_holds(256, 233, 512, 0.5, 0.4));
        assert!(!decode_holds(256, 232, 512, 0.5, 0.4), "one past the band");
        assert!(
            !decode_holds(256, 300, 512, 0.4, 0.4),
            "the repeat rate must be lower"
        );
    }

    /// claude-211 (I-G): decision 5's tolerance and its three verdicts.
    #[test]
    fn the_verdict_follows_the_protocol() {
        use OverturnCell::{Ahead, Inside};
        let made = |complete: usize, verbatim: usize, repeat: f64| Generation {
            replies: 256,
            complete,
            verbatim_train: verbatim,
            new_ngram_share: Vec::new(),
            distinct_1: 0.0,
            distinct_2: 0.0,
            repeated_4gram_rate: repeat,
        };
        let references = made(200, 128, 0.05);
        // v = v_ref = 0.64: tol = max(0.10, 3·√(0.64·0.36/100)) = 0.144.
        let close = VerdictSeed::new(1, &made(100, 64, 0.1), &references);
        assert!((close.tol.unwrap() - 0.144).abs() < 1e-12);
        assert!(close.generalizes() && !close.memorized());
        // v = 0.80: past the tolerance.
        let copied = VerdictSeed::new(1, &made(100, 80, 0.1), &references);
        assert!(copied.memorized() && !copied.generalizes());
        assert!(
            !VerdictSeed::new(1, &made(63, 40, 0.1), &references).generalizes(),
            "n_c < 64"
        );
        assert!(
            !VerdictSeed::new(1, &made(100, 64, 0.2), &references).generalizes(),
            "a loop"
        );
        // The repeat bound is inclusive: exactly the references' rate + 0.10 passes.
        assert!(VerdictSeed::new(1, &made(100, 64, 0.1), &made(200, 128, 0.0)).generalizes());
        let none = VerdictSeed::new(1, &made(0, 0, 0.0), &references);
        assert!(none.v.is_none() && !none.generalizes() && !none.memorized());
        let three = |s: &VerdictSeed| [1u8, 2, 3].map(|seed| VerdictSeed { seed, ..s.clone() });
        assert_eq!(verdict([Ahead; 4], Ahead, &three(&close)).0, "generalizes");
        assert_eq!(
            verdict([Ahead, Ahead, Ahead, Inside], Ahead, &three(&close)).0,
            "mixed"
        );
        assert_eq!(verdict([Inside; 4], Ahead, &three(&copied)).0, "memorized");
        let (name, failed) = verdict([Ahead, Inside, Inside, Inside], Ahead, &three(&copied));
        assert_eq!(name, "mixed", "memorized needs it ahead of none of them");
        assert!(!failed.is_empty());
        assert_eq!(
            verdict([Ahead; 4], Ahead, &three(&close)[..2]).0,
            "mixed",
            "three seeds"
        );
    }
    /// claude-208: candidates rank by the higher score, ties to the smaller id.
    #[test]
    fn candidates_rank_by_score_then_smaller_id() {
        let ranked = rank_order(vec![(9, -1.0), (4, -0.5), (2, -1.0), (7, -0.25)]);
        assert_eq!(ranked.iter().map(|&(id, _)| id).collect::<Vec<_>>(), vec![7, 4, 2, 9]);
        assert_eq!(ranked[0], (7, -0.25), "each keeps its score");
    }

    /// claude-208 (protocol I-F): the key each later step of a reply reads, as the decoder forms
    /// it, is the census's and the memory's `reply_key` for that row, so counting it on the true
    /// reply is teacher forcing; and `reply_key_hits` counts every later step.
    #[test]
    fn the_hit_rate_keys_are_the_teacher_forced_reply_keys() {
        // BOS, a prompt (10 11 12), a reply (11 12 30), a prompt (13), a reply (31).
        let tokens = [0u32, 10, 11, 12, 11, 12, 30, 13, 31];
        let mask = [0u8, 0, 0, 0, 1, 1, 1, 0, 1];
        let (windows, _) = hs::build_document_windows(&tokens, &mask, 16, 0).unwrap();
        let rows = reply_rows(&windows, &mask);
        let later: Vec<&ReplyRow> = rows.iter().filter(|r| !r.opening).collect();
        assert_eq!(later.len(), 2);
        for row in &later {
            let ids = &windows[row.window].input_ids;
            let opening = row.offset - row.reply.len();
            let key = greedy_reply_key(&ids[..=row.offset], opening, &row.reply).unwrap();
            assert_eq!(key, reply_key(row), "row at offset {}", row.offset);
        }
        // The first reply after its prompt: two later steps, both keyed as their rows.
        let prompt = &windows[0].input_ids[..=3];
        let reply = [11u32, 12, 30];
        let keys: std::collections::HashSet<Vec<u32>> = later.iter().map(|r| reply_key(r)).collect();
        assert_eq!(reply_key_hits(prompt, &reply, SHEET_CONTEXT, |k| keys.contains(k)), (2, 2));
        assert_eq!(reply_key_hits(prompt, &reply, SHEET_CONTEXT, |_| false), (2, 0));
        assert_eq!(reply_key_hits(prompt, &reply[..1], SHEET_CONTEXT, |_| true), (0, 0), "an opening only");
    }

    /// claude-208 (review): rank `i` takes exactly `W_i` draws: a draw equal to a running sum
    /// belongs to the next rank, so `>` (not `>=`) is pinned at each boundary.
    #[test]
    fn a_draw_on_a_boundary_takes_the_next_rank() {
        let w = &FLOCK_RANK_WEIGHTS_Q16;
        assert_eq!(harmonic_rank(0, w), 0);
        assert_eq!(harmonic_rank(u64::from(w[0]) - 1, w), 0);
        assert_eq!(harmonic_rank(u64::from(w[0]), w), 1);
        let first_two = u64::from(w[0]) + u64::from(w[1]);
        assert_eq!(harmonic_rank(first_two - 1, w), 1);
        assert_eq!(harmonic_rank(first_two, w), 2);
        let total: u64 = w.iter().map(|&x| u64::from(x)).sum();
        assert_eq!(harmonic_rank(total - 1, w), FLOCK_DECODE_K - 1);
    }

    /// claude-208 (review): a generated reply is counted up to its first end of turn, so the next
    /// turn's tokens it runs on into add no keys.
    #[test]
    fn the_hit_rate_counts_a_reply_up_to_its_end_of_turn() {
        const EOS: u32 = 2;
        assert_eq!(cut_at_eos(&[5, 6, EOS, 7, 8], EOS), &[5, 6, EOS]);
        assert_eq!(cut_at_eos(&[5, 6], EOS), &[5, 6]);
        let prompt = [0u32, 10, 11, 12];
        let all = reply_key_hits(&prompt, &[11, 12, EOS, 40, 41, 42], SHEET_CONTEXT, |_| true).0;
        let cut = reply_key_hits(&prompt, cut_at_eos(&[11, 12, EOS, 40, 41, 42], EOS), SHEET_CONTEXT, |_| true).0;
        assert_eq!((all, cut), (5, 2), "the 3 steps after the end of turn are not counted");
    }

    /// claude-208 (review): a reply whose window no longer reaches its prompt forms no key, as the
    /// decoder forms none there.
    #[test]
    fn the_hit_rate_stops_where_the_window_loses_the_prompt() {
        let prompt: Vec<u32> = (10..20).collect();
        let reply: Vec<u32> = (100..400).collect();
        // Step j's window is the last SHEET_CONTEXT of prompt + reply[..j]; it holds the prompt's
        // last token while 10 + j − SHEET_CONTEXT < 10, that is for j < SHEET_CONTEXT.
        let (keys, _) = reply_key_hits(&prompt, &reply, SHEET_CONTEXT, |_| true);
        assert_eq!(keys, SHEET_CONTEXT - 1);
    }

    /// claude-274: the hit rate slides over the window the decoder sees, the model's own. At
    /// 2,048 the window keeps the prompt through every step; at 8, shorter than the prompt, it
    /// loses it after 7.
    #[test]
    fn the_hit_rate_reads_the_window_the_decoder_sees() {
        let prompt: Vec<u32> = (10..20).collect();
        let reply: Vec<u32> = (100..400).collect();
        assert_eq!(reply_key_hits(&prompt, &reply, 2048, |_| true).0, reply.len() - 1);
        assert_eq!(reply_key_hits(&prompt, &reply, 8, |_| true).0, 7);
    }

    /// claude-274: the reply table's rows sit at the forward's own row for them, the window's
    /// index times its length plus the offset, so at 2,048 the second window's rows start at
    /// 2,048 and not at 192.
    #[test]
    fn the_reply_table_rows_follow_the_forward_s_window() {
        // Two documents, each BOS, a prompt (3), a reply (3): the later rows are offsets 4 and 5.
        let tokens = [0u32, 10, 11, 12, 11, 12, 30, 0, 20, 21, 22, 21, 22, 40];
        let mask = [0u8, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1];
        let (windows, _) = hs::build_document_windows(&tokens, &mask, 2048, 0).unwrap();
        assert_eq!(windows.len(), 2);
        let rows: Vec<usize> = reply_table_rows(&windows, &mask, 2048).into_iter().map(|(r, _)| r).collect();
        assert_eq!(rows, [4, 5, 2048 + 4, 2048 + 5]);
        let keys: Vec<Vec<u32>> = reply_table_rows(&windows, &mask, 2048).into_iter().map(|(_, k)| k).collect();
        let later: Vec<Vec<u32>> =
            reply_rows(&windows, &mask).iter().filter(|r| !r.opening).map(reply_key).collect();
        assert_eq!(keys, later, "each row keeps its own key");
    }

    /// claude-274: a scoring forward holds the positions of 16 windows of 192, so 16 windows at
    /// 192 as before, and fewer at longer windows.
    #[test]
    fn a_scoring_forward_holds_sixteen_windows_of_192_positions() {
        assert_eq!(windows_per_forward(SHEET_CONTEXT), hs::SHEET_EVAL_BATCH);
        assert_eq!(windows_per_forward(96), hs::SHEET_EVAL_BATCH, "never more than 16");
        assert_eq!(windows_per_forward(1024), 3);
        assert_eq!(windows_per_forward(2048), 1);
        assert_eq!(windows_per_forward(4096), 1, "never fewer than 1");
    }

    /// claude-274: the batched decode's budget stays `SHEET_EVAL_BATCH` windows of 192 positions,
    /// and a group's reach is capped by the model's own window. At 192 a 1,000-token prompt group
    /// reaches the 192 window, 16 a batch, as before; at 2,048 its 1,064-token reach takes 2.
    #[test]
    fn exact_length_batches_cap_the_reach_at_the_model_s_window() {
        let prompts: Vec<Vec<u32>> = (0..40).map(|seed| vec![seed; 1000]).collect();
        let sizes = |context| -> Vec<usize> {
            exact_length_batches(&prompts, 64, context).iter().map(Vec::len).collect()
        };
        assert_eq!(sizes(SHEET_CONTEXT), [16, 16, 8]);
        assert_eq!(sizes(2048), [2; 20]);
    }

    /// claude-274: `--sample-opening last` decodes from each window's last reply opening with at
    /// least 16 tokens of prompt. Its prompt runs through the earlier turns, and its reference,
    /// position and input flags follow that opening; a window with no such opening is left out.
    #[test]
    fn the_last_opening_starts_each_window_at_its_latest_reply() {
        // BOS, a 20-token prompt, a reply (3), a 5-token prompt, a reply (3); then a document
        // whose only opening is 5 tokens in.
        let mut tokens = vec![0u32];
        tokens.extend(100..120);
        tokens.extend([200, 201, 202]);
        tokens.extend(300..305);
        tokens.extend([400, 401, 402]);
        tokens.extend([0, 500, 501, 502, 503, 504, 600, 601]);
        let mask: Vec<u8> = tokens
            .iter()
            .map(|&t| u8::from((200..203).contains(&t) || (400..403).contains(&t) || t >= 600))
            .collect();
        let (windows, _) = hs::build_document_windows(&tokens, &mask, 2048, 0).unwrap();
        assert_eq!(windows.len(), 2);
        let first = sample_windows(&windows, &mask, 4);
        assert_eq!(first.len(), 1, "the second document's opening is 5 tokens in");
        assert_eq!(first[0].scored_offsets[0], 20);
        let last = last_openings(&windows, &mask, 4);
        assert_eq!(last.len(), 1);
        let cut = &last[0];
        assert_eq!((cut.doc_id, &cut.input_ids), (windows[0].doc_id, &windows[0].input_ids));
        assert_eq!(cut.scored_offsets, [28, 29, 30]);
        assert_eq!(cut.scored_corpus_positions, [29, 30, 31]);
        assert_eq!(&cut.input_ids[..=cut.scored_offsets[0]], &tokens[..29], "the prompt holds both turns");
        assert_eq!(reference_reply(cut, 64), [400, 401, 402]);
        assert_eq!(
            window_input_mask(&[cut], &mask).unwrap(),
            window_input_mask(&[&windows[0]], &mask).unwrap(),
            "the input flags are the window's own"
        );
    }

    /// claude-274: a scored file at 192 writes no `context`, so a 192 manifest is as before; at
    /// another window it names it.
    #[test]
    fn a_scored_file_names_its_window_only_off_192() {
        let file = |context| ScoredFile {
            file: "rows-p-dev-on.json".to_owned(),
            rows: 3,
            nearest_hits: Some(1),
            writer_hits: None,
            panel_sha256: None,
            tables: None,
            decode: None,
            search: None,
            context,
        };
        assert_eq!(
            serde_json::to_string(&file(SHEET_CONTEXT)).unwrap(),
            r#"{"file":"rows-p-dev-on.json","rows":3,"nearest_hits":1}"#
        );
        assert!(serde_json::to_string(&file(2048)).unwrap().ends_with(r#","context":2048}"#));
    }

    /// claude-274: off 192, each flag that reads 192-token windows is refused by its own name; at
    /// 192, or with none of them, scoring goes ahead.
    #[test]
    fn each_192_bound_flag_is_refused_by_name_off_192() {
        assert!(window_bound_flags(SHEET_CONTEXT, true, true, true, true, true).is_ok());
        assert!(window_bound_flags(2048, false, false, false, false, false).is_ok());
        let names = [
            "--opening-keys",
            "--served-key",
            "--keyed-decode",
            "--decision-trace",
            "--decision-force",
        ];
        for (i, name) in names.iter().enumerate() {
            let on = |j: usize| i == j;
            let err = window_bound_flags(2048, on(0), on(1), on(2), on(3), on(4))
                .expect_err(name)
                .to_string();
            assert!(err.contains(name) && err.contains("--context 2048"), "{err}");
            for other in names.iter().filter(|o| *o != name) {
                assert!(!err.contains(other), "{err} names {other}");
            }
        }
    }

    /// claude-208 (review): a panel name is one file-name part, and never DEV or Fresh V2.
    #[test]
    fn a_panel_name_is_never_dev_fresh_v2_or_a_path() {
        assert!(panel_name_ok("p-dev"));
        for bad in ["", "dev", "fresh-v2", "a/b", "..", "x\\y"] {
            assert!(!panel_name_ok(bad), "{bad:?}");
        }
    }

    /// claude-208: a sample file written before the hit counts still loads, with them at 0.
    #[test]
    fn an_older_sample_file_still_loads() {
        let old = r#"{"position":4,"prompt":[1],"reply":[2],"reference":[3],"reply_text":"a","reference_text":"b"}"#;
        let sample: IdSample = serde_json::from_str(old).unwrap();
        assert_eq!((sample.reply_keys, sample.reply_named, sample.reference_keys), (0, 0, 0));
    }

    /// claude-258: the trunk, Hamming reads and turn memory a training run holds, with the reads'
    /// and the memory's weights moved off the export's grid (they start at 0, which it holds), so
    /// rounding each part as exported moves it.
    fn served_parts(
        device: &Device,
    ) -> Result<(
        StackModel,
        crate::flock_read_hamming::HammingReads,
        crate::flock_read_memory::TurnMemory,
    )> {
        let model = build_fbs_model_at(FbsArm::OneAtomActive, 1, hs::SHEET_WIDTH, device)?;
        let reads = crate::flock_read_hamming::HammingReads::new(4, hs::SHEET_WIDTH, device)?;
        let memory = crate::flock_read_memory::TurnMemory::new(
            8,
            std::collections::HashMap::new(),
            hs::SHEET_WIDTH,
            device,
        )?;
        let off_grid = |var: &candle_core::Var| -> Result<()> {
            let values: Vec<f32> = (0..var.elem_count())
                .map(|i| 0.123_456_7 + i as f32 * 1e-7)
                .collect();
            var.set(&Tensor::from_vec(values, var.shape(), device)?)?;
            Ok(())
        };
        for read in &reads.reads {
            off_grid(&read.weight)?;
        }
        off_grid(&memory.weight)?;
        Ok((model, reads, memory))
    }

    fn served_vars<'a>(
        model: &'a StackModel,
        reads: &'a crate::flock_read_hamming::HammingReads,
        memory: &'a crate::flock_read_memory::TurnMemory,
    ) -> Vec<&'a candle_core::Var> {
        let mut vars: Vec<&candle_core::Var> = model.variables().values().collect();
        vars.extend(reads.variables());
        vars.extend(memory.variables());
        vars
    }

    fn served_bits(vars: &[&candle_core::Var]) -> Result<Vec<Vec<u32>>> {
        vars.iter()
            .map(|var| {
                let values = var.as_tensor().flatten_all()?.to_vec1::<f32>()?;
                Ok(values.into_iter().map(f32::to_bits).collect())
            })
            .collect()
    }

    /// A checkpoint's save as [`run_long`] makes it: rounded as served when `on`, then put back.
    fn save_step(
        dir: &Path,
        on: bool,
        model: &StackModel,
        reads: &crate::flock_read_hamming::HammingReads,
        memory: &crate::flock_read_memory::TurnMemory,
    ) -> Result<()> {
        fs::create_dir_all(dir)?;
        let served = round_if_served(on, model, Some(reads), Some(memory))?;
        model.save(dir)?;
        reads.save(dir)?;
        memory.save(dir)?;
        if let Some(served) = served {
            served.restore()?;
        }
        Ok(())
    }

    /// Which of the trunk, the Hamming reads and the turn memory saved in `dir` the export's own
    /// rounding changes, on the CPU, which stores and moves the values (D24/D29).
    fn export_changes(dir: &Path) -> Result<[bool; 3]> {
        let device = Device::Cpu;
        let model = StackModel::load(dir, &device)?;
        let reads = crate::flock_read_hamming::HammingReads::load(dir, &device)?
            .ok_or_else(|| invalid("no saved Hamming reads"))?;
        let memory = crate::flock_read_memory::TurnMemory::load(dir, &device)?
            .ok_or_else(|| invalid("no saved turn memory"))?;
        let trunk: Vec<&candle_core::Var> = model.variables().values().collect();
        let held = [
            served_bits(&trunk)?,
            served_bits(&reads.variables())?,
            served_bits(&memory.variables())?,
        ];
        crate::fbs_export::round_tables_as_exported(&model)?;
        crate::fbs_export::round_hamming_as_exported(&reads)?;
        crate::fbs_export::round_memory_as_exported(&memory)?;
        Ok([
            held[0] != served_bits(&trunk)?,
            held[1] != served_bits(&reads.variables())?,
            held[2] != served_bits(&memory.variables())?,
        ])
    }

    /// claude-258: a checkpoint saved as served is one the export's rounding leaves unchanged in
    /// every part, and the run's own values are the float ones again after it. Saved without the
    /// rounding, the same values are ones the export changes in every part. The training steps
    /// themselves run on the GPU only (D24/D29), in the test after this one.
    #[test]
    fn a_step_saved_as_served_is_one_its_export_leaves_unchanged() -> Result<()> {
        let root = std::env::temp_dir().join(format!("claude-258-saved-{}", std::process::id()));
        let (model, reads, memory) = served_parts(&Device::Cpu)?;
        let vars = served_vars(&model, &reads, &memory);
        let float = served_bits(&vars)?;
        {
            let guard = round_if_served(true, &model, Some(&reads), Some(&memory))?
                .expect("the rounding guard is on");
            assert!(crate::fbs_export::check_tables_rounded_as_exported(&model).is_ok());
            drop(guard);
        }
        assert_eq!(
            served_bits(&vars)?,
            float,
            "an early return or panic drops the guard back to the float masters"
        );
        save_step(&root.join("on"), true, &model, &reads, &memory)?;
        assert_eq!(served_bits(&vars)?, float, "the float values are back");
        assert_eq!(export_changes(&root.join("on"))?, [false; 3]);
        save_step(&root.join("off"), false, &model, &reads, &memory)?;
        assert_eq!(export_changes(&root.join("off"))?, [true; 3]);
        fs::remove_dir_all(&root)?;
        Ok(())
    }

    /// claude-258: three training steps on the GPU, through the trunk's own loss and the run's
    /// own optimizers in [`run_long`]'s order. As served, each forward reads the tables on the
    /// export's grid, the run's values stay float, and the saved step is one the export leaves
    /// unchanged; without the switch, the export changes every part of the saved step.
    #[cfg(feature = "cuda")]
    #[test]
    #[ignore = "needs a CUDA device; run on the pod"]
    fn three_steps_trained_as_served_save_a_step_the_export_leaves_unchanged() -> Result<()> {
        let device = Device::new_cuda(0)?;
        let root = std::env::temp_dir().join(format!("claude-258-gpu-{}", std::process::id()));
        let batch = 2;
        let ids: Vec<u32> = (0..batch * SHEET_CONTEXT)
            .map(|i| ((i * 37 + 5) % hs::SHEET_VOCAB) as u32)
            .collect();
        let scored: Vec<u32> = (0..batch * SHEET_CONTEXT - 1)
            .step_by(7)
            .map(|i| i as u32)
            .collect();
        let targets: Vec<u32> = scored.iter().map(|&i| ids[i as usize + 1]).collect();
        for on in [true, false] {
            let (model, reads, memory) = served_parts(&device)?;
            crate::flock_read_hamming::set_hamming_reads(Some(reads.clone()));
            crate::flock_read_memory::set_turn_memory(Some(memory.clone()));
            let mut opt_main = StackAdamW::new(&model, hs::SHEET_WEIGHT_DECAY, hs::SHEET_CLIP)?;
            let mut opt_decay = StackAdamW::new(&model, 0.0, hs::SHEET_CLIP)?;
            let mut opt_reads = crate::flock_read::FlockAdamW::for_vars(&reads.variables())?;
            let mut opt_memory = crate::flock_read::FlockAdamW::for_vars(&memory.variables())?;
            for step in 0..3 {
                let served = round_if_served(on, &model, Some(&reads), Some(&memory))?;
                assert_eq!(
                    crate::fbs_export::check_tables_rounded_as_exported(&model).is_ok(),
                    on,
                    "step {step}"
                );
                let grads = with_training_forward(on, 1, step, || {
                    let loss = batch_loss(
                        &model,
                        PointObjective::Cell,
                        &ids,
                        &scored,
                        &targets,
                        batch,
                        on,
                    )?;
                    Ok(loss.backward()?)
                })?;
                if let Some(served) = served {
                    served.restore()?;
                }
                opt_main.update_filtered(&model, &grads, 1e-3, |name| {
                    main_optimizer_updates(name, FbsArm::OneAtomActive)
                })?;
                opt_reads.step_vars(&reads.variables(), &grads, 1e-3)?;
                opt_memory.step_vars(&memory.variables(), &grads, 1e-3)?;
                opt_decay.update_filtered(&model, &grads, 1e-3, |name| {
                    name.ends_with("rec.decay")
                })?;
            }
            // The run's own values stay float: the optimizer moved them, not their rounding.
            assert!(crate::fbs_export::check_tables_rounded_as_exported(&model).is_err());
            let dir = root.join(if on { "on" } else { "off" });
            save_step(&dir, on, &model, &reads, &memory)?;
            let changes = export_changes(&dir)?;
            assert_eq!(changes, if on { [false; 3] } else { [true; 3] }, "on {on}");
            crate::flock_read_hamming::set_hamming_reads(None);
            crate::flock_read_memory::set_turn_memory(None);
        }
        fs::remove_dir_all(&root)?;
        Ok(())
    }

    /// claude-285 (G2): one training step as served, through the trunk's own loss, at (S)
    /// 192 × 2 and (L) 2048 × 4. The fused quantizers give the chains' loss and every gradient bit
    /// for bit, after an A/A pair (the chains twice) shows that the step repeats itself; a fused
    /// node can change the order in which candle sums a gradient, which this would catch.
    #[cfg(feature = "cuda")]
    #[test]
    #[ignore = "needs a CUDA device; run on the pod"]
    fn one_step_as_served_is_bit_equal_with_the_fused_quantizers() -> Result<()> {
        let device = Device::new_cuda(0)?;
        for (context, batch) in [(SHEET_CONTEXT, 2), (2048, 4)] {
            let ids: Vec<u32> = (0..batch * context)
                .map(|i| ((i * 37 + 5) % hs::SHEET_VOCAB) as u32)
                .collect();
            let scored: Vec<u32> = (0..batch * context - 1)
                .step_by(7)
                .map(|i| i as u32)
                .collect();
            let targets: Vec<u32> = scored.iter().map(|&i| ids[i as usize + 1]).collect();
            let step = |fused: bool| -> Result<(Vec<u32>, Vec<Vec<u32>>, hs::FusedQuantizeCounts)> {
                let (mut model, reads, memory) = served_parts(&device)?;
                model.set_context(context)?;
                crate::flock_read_hamming::set_hamming_reads(Some(reads.clone()));
                crate::flock_read_memory::set_turn_memory(Some(memory.clone()));
                let served = round_if_served(true, &model, Some(&reads), Some(&memory))?;
                let ((loss, grads), counts) = hs::with_fused_quantize(fused, || {
                    with_training_forward(true, 1, 0, || {
                        let loss = batch_loss(
                            &model,
                            PointObjective::Cell,
                            &ids,
                            &scored,
                            &targets,
                            batch,
                            true,
                        )?;
                        let grads = loss.backward()?;
                        Ok((loss, grads))
                    })
                })?;
                if let Some(served) = served {
                    served.restore()?;
                }
                let bits = |t: &Tensor| -> Result<Vec<u32>> {
                    Ok(t.flatten_all()?
                        .to_vec1::<f32>()?
                        .into_iter()
                        .map(f32::to_bits)
                        .collect())
                };
                let mut grad_bits = Vec::new();
                for var in served_vars(&model, &reads, &memory) {
                    grad_bits.push(match grads.get(var.as_tensor()) {
                        Some(grad) => bits(grad)?,
                        None => Vec::new(),
                    });
                }
                crate::flock_read_hamming::set_hamming_reads(None);
                crate::flock_read_memory::set_turn_memory(None);
                Ok((bits(&loss)?, grad_bits, counts))
            };
            let chain = step(false)?;
            let again = step(false)?;
            assert_eq!((&again.0, &again.1), (&chain.0, &chain.1), "{context}: A/A");
            let fused = step(true)?;
            eprintln!(
                "claude-285 G2 {context}x{batch}: chains {:?}, fused {:?}",
                chain.2, fused.2
            );
            assert!(
                fused.2.activation_fused > 0 && fused.2.residual_fused > 0,
                "{context}"
            );
            assert_eq!(
                fused.2.activation_fused + fused.2.activation_chain,
                chain.2.activation_chain,
                "{context}: the same activation calls"
            );
            assert_eq!(
                fused.2.residual_fused + fused.2.residual_chain,
                chain.2.residual_chain,
                "{context}: the same residual calls"
            );
            assert_eq!(fused.0, chain.0, "{context}: loss");
            assert!(
                chain.1.iter().any(|grad| !grad.is_empty()),
                "{context}: gradients"
            );
            assert_eq!(fused.1, chain.1, "{context}: every gradient");
        }
        Ok(())
    }

    /// claude-262: one training step as served reaches every projection that reads a requantized
    /// activation. Each layer's `mlp.down` and `rec.out` table gets a gradient whose norm
    /// is within 4x of the plain step's, from the same initial values, and never 0. Before, the
    /// served stand-in rounded those activations to whole numbers, so most of them were 0 and so
    /// was the gradient of the weight that reads them.
    #[cfg(feature = "cuda")]
    #[test]
    fn a_step_as_served_reaches_every_projection_after_a_requantized_activation() -> Result<()> {
        let device = Device::new_cuda(0)?;
        let batch = 2;
        let ids: Vec<u32> = (0..batch * SHEET_CONTEXT)
            .map(|i| ((i * 37 + 5) % hs::SHEET_VOCAB) as u32)
            .collect();
        let scored: Vec<u32> = (0..batch * SHEET_CONTEXT - 1)
            .step_by(7)
            .map(|i| i as u32)
            .collect();
        let targets: Vec<u32> = scored.iter().map(|&i| ids[i as usize + 1]).collect();
        let mut norms: Vec<Vec<(String, f32)>> = Vec::new();
        for on in [false, true] {
            let (model, reads, memory) = served_parts(&device)?;
            crate::flock_read_hamming::set_hamming_reads(Some(reads.clone()));
            crate::flock_read_memory::set_turn_memory(Some(memory.clone()));
            let served = round_if_served(on, &model, Some(&reads), Some(&memory))?;
            let grads = with_training_forward(on, 1, 0, || {
                let loss = batch_loss(
                    &model,
                    PointObjective::Cell,
                    &ids,
                    &scored,
                    &targets,
                    batch,
                    on,
                )?;
                Ok(loss.backward()?)
            })?;
            if let Some(served) = served {
                served.restore()?;
            }
            crate::flock_read_hamming::set_hamming_reads(None);
            crate::flock_read_memory::set_turn_memory(None);
            let mut arm = Vec::new();
            // The FBS trunk holds each projection as its SpiralCore table, `….spiralcore_fbs`,
            // in place of the dense `….weight`.
            for layer in 0..model.config.layers() {
                for table in ["mlp.down.spiralcore_fbs", "rec.out.spiralcore_fbs"] {
                    let name = format!("layers.{layer:02}.{table}");
                    let var = model
                        .variables()
                        .get(&name)
                        .ok_or_else(|| invalid(format!("the trunk lacks {name}")))?;
                    let norm = match grads.get(var.as_tensor()) {
                        Some(g) => g.sqr()?.sum_all()?.sqrt()?.to_scalar::<f32>()?,
                        None => 0.0,
                    };
                    arm.push((name, norm));
                }
            }
            norms.push(arm);
        }
        assert!(
            !norms[0].is_empty(),
            "the trunk has no layers, so the check is vacuous"
        );
        for ((name, plain), (_, served)) in norms[0].iter().zip(&norms[1]) {
            assert!(*plain > 0.0, "{name}: the plain step gives it no gradient");
            let ratio = served / plain;
            assert!(
                (0.25..=4.0).contains(&ratio),
                "{name}: gradient norm {served} as served against {plain} plain"
            );
        }
        Ok(())
    }

    /// claude-258: without the switch nothing is rounded, so a run without it trains as before,
    /// and its `config.json` has no new key.
    #[test]
    fn training_without_the_switch_rounds_nothing() -> Result<()> {
        let (model, reads, memory) = served_parts(&Device::Cpu)?;
        let vars = served_vars(&model, &reads, &memory);
        let before = served_bits(&vars)?;
        assert!(round_if_served(false, &model, Some(&reads), Some(&memory))?.is_none());
        assert_eq!(served_bits(&vars)?, before);
        let mut set = config(0, None);
        let json = serde_json::to_string(&set)?;
        assert!(!json.contains("train_as_served"), "{json}");
        set.train_as_served = true;
        let json = serde_json::to_string(&set)?;
        assert!(json.contains("\"train_as_served\":true"), "{json}");
        Ok(())
    }

    /// claude-258: a rounding the export refuses, here a margin that rounds to 0, leaves every
    /// value as it was, the trunk's tables rounded before it included.
    #[test]
    fn a_refused_rounding_leaves_every_value_as_it_was() -> Result<()> {
        let (model, reads, memory) = served_parts(&Device::Cpu)?;
        reads.reads[0]
            .margin
            .set(&Tensor::from_vec(vec![1e-7f32; 4], 4, &Device::Cpu)?)?;
        let vars = served_vars(&model, &reads, &memory);
        let before = served_bits(&vars)?;
        let refused = round_if_served(true, &model, Some(&reads), Some(&memory));
        assert!(
            matches!(&refused, Err(e) if e.to_string().contains("not positive")),
            "{:?}",
            refused.err()
        );
        assert_eq!(served_bits(&vars)?, before);
        Ok(())
    }

    #[test]
    fn the_training_forward_selects_and_restores_the_served_bridges() -> Result<()> {
        let flags = || {
            [
                hs::is_rounding_residual_q16(),
                hs::is_bridging_q16_residual(),
                hs::is_bridging_activation_i16(),
                hs::is_bridging_d11(),
                hs::is_snapping_transition(),
            ]
        };
        assert_eq!(with_training_forward(false, 1, 3, || Ok(flags()))?, [false; 5]);
        // claude-265: the served contract is the unsnapped one, so the transition is not snapped.
        assert_eq!(
            with_training_forward(true, 1, 3, || Ok(flags()))?,
            [true, true, true, true, false]
        );
        assert_eq!(flags(), [false; 5]);
        Ok(())
    }

    /// claude-258: training as served refuses the reads the export does not carry, the type spin,
    /// and folded or reflected rate scores, before the run writes anything.
    #[test]
    fn training_as_served_refuses_what_the_export_does_not_carry() {
        let missing = std::env::temp_dir().join(format!("claude-258-none-{}", std::process::id()));
        let refusal = |set: fn(&mut LongConfig)| -> String {
            let mut config = config(0, None);
            config.train_as_served = true;
            set(&mut config);
            match run_long(&missing, &missing.join("out"), &config, &Device::Cpu) {
                Err(e) => e.to_string(),
                Ok(()) => "ran".to_string(),
            }
        };
        for set in [
            (|c: &mut LongConfig| c.flock_read = 1) as fn(&mut LongConfig),
            |c| c.content_read = 1,
            |c| c.opening_read = 1,
            |c| c.prompt_register = 1,
            |c| c.type_spin = true,
            |c| {
                c.hamming_read = 4;
                c.reply_read_train = Some(ReadScope::Copy);
            },
            |c| c.fold_scores = true,
            |c| c.reflect_floor = true,
        ] {
            assert!(refusal(set).contains("--train-as-served"));
        }
        assert!(!missing.exists());
    }

    /// claude-275: from 193 to 1,536 positions several windows share a scoring forward and the
    /// row stride over them has no test, so `--score-rows --context N` refuses them by name; 192
    /// and 1,537 up go ahead.
    #[test]
    fn the_row_stride_is_refused_where_windows_share_a_forward() {
        for ok in [SHEET_CONTEXT, 1537, 2048, 4096] {
            assert!(row_stride_window(ok).is_ok(), "{ok}");
        }
        for refused in [193, 384, 1024, 1536] {
            let err = row_stride_window(refused).expect_err("refused").to_string();
            assert!(err.contains(&format!("--context {refused}")), "{err}");
        }
    }

    fn chain_query(document_start: usize, reply_start: usize, reply_tokens: usize, answer: &str, distance: usize) -> ChainQuery {
        ChainQuery {
            document_start,
            reply_start,
            reply_tokens,
            answer: answer.to_owned(),
            chain: QueryChain { distance },
        }
    }

    /// claude-275: a query decodes from the `context` positions ending at its reply's last
    /// target, or from its document's BOS when that is nearer, as a served decode slides: its
    /// prompt ends at the reply's opening, its reference is the whole reply, its input flags read
    /// the panel's mask. A reply longer than the window is counted past it, and a query that does
    /// not sit on a reply of its length in one BOS-opened document is refused.
    #[test]
    fn a_query_decodes_from_the_window_that_ends_at_its_reply() {
        // BOS, a 20-token prompt, a reply (3), a 5-token prompt, the query's reply (3); then a
        // document with a 5-token prompt and a 2-token reply.
        let mut tokens = vec![0u32];
        tokens.extend(100..120);
        tokens.extend([200, 201, 202]);
        tokens.extend(300..305);
        tokens.extend([400, 401, 402]);
        tokens.extend([0, 500, 501, 502, 503, 504, 600, 601]);
        let mask: Vec<u8> = tokens
            .iter()
            .map(|&t| u8::from((200..203).contains(&t) || (400..403).contains(&t) || t >= 600))
            .collect();
        let query = chain_query(0, 29, 3, "x", 10);
        let whole = query_prompts(&tokens, &mask, 2048, std::slice::from_ref(&query)).unwrap();
        assert!(whole.past_window.is_empty());
        let (index, window) = &whole.decoded[0];
        assert_eq!(*index, 0);
        assert_eq!(window.input_ids, &tokens[..31]);
        assert_eq!(window.scored_offsets, [28, 29, 30]);
        assert_eq!(window.scored_corpus_positions, [29, 30, 31]);
        assert_eq!(&window.input_ids[..=window.scored_offsets[0]], &tokens[..29]);
        assert_eq!(reference_reply(window, 64), [400, 401, 402]);
        assert_eq!(window_input_mask(&[window], &mask).unwrap(), {
            let mut flags = mask[..31].to_vec();
            flags[0] = 0;
            flags
        });
        // At 8 positions the window ends at the reply and starts 6 tokens before its opening.
        let cut = query_prompts(&tokens, &mask, 8, std::slice::from_ref(&query)).unwrap();
        let (_, window) = &cut.decoded[0];
        assert_eq!(window.input_ids, &tokens[23..31]);
        assert_eq!(window.scored_offsets, [5, 6, 7]);
        assert_eq!(&window.input_ids[..=5], &tokens[23..29]);
        assert_eq!(window_input_mask(&[window], &mask).unwrap(), &mask[23..31]);
        // A reply longer than the window is counted past it, not decoded.
        let past = query_prompts(&tokens, &mask, 2, std::slice::from_ref(&query)).unwrap();
        assert!(past.decoded.is_empty());
        assert_eq!(past.past_window, [0]);
        // The second document's reply, from its own BOS.
        let second = query_prompts(&tokens, &mask, 2048, &[chain_query(32, 38, 2, "y", 3)]).unwrap();
        assert_eq!(second.decoded[0].1.input_ids, &tokens[32..39]);
        for bad in [
            chain_query(0, 30, 2, "x", 10),
            chain_query(0, 29, 2, "x", 10),
            chain_query(0, 29, 4, "x", 10),
            chain_query(1, 29, 3, "x", 10),
            chain_query(0, 38, 2, "y", 3),
            chain_query(0, 29, 3, "", 10),
            chain_query(32, 38, 3, "y", 3),
        ] {
            assert!(
                query_prompts(&tokens, &mask, 2048, std::slice::from_ref(&bad)).is_err(),
                "{bad:?}"
            );
        }
    }

    /// claude-275: a reply states an answer only as a whole word, and the grade bins each query
    /// by distance, counting the decoded, the references that state the answer, the replies that
    /// do, and the prompts that start at the document's BOS.
    #[test]
    fn the_recall_grade_counts_whole_word_answers_by_distance() {
        assert!(states_answer(" That's 5468.", "5468"));
        assert!(states_answer("likperkmist", "likperkmist"));
        assert!(!states_answer(" That's 15468.", "5468"));
        assert!(!states_answer(" likperkmists", "likperkmist"));
        assert!(!states_answer(" That's 5468.", ""));
        let queries = [
            chain_query(0, 10, 2, "97", 100),
            chain_query(0, 20, 2, "97", 600),
            chain_query(40, 50, 2, "staiduk", 1500),
        ];
        // The first query's window starts at its document's BOS; the second's does not.
        let window = |from: usize, start: usize, bos: bool| CausalWindow {
            doc_id: 0,
            input_ids: std::iter::once(if bos { 0 } else { 7 }).chain(from + 1..start + 1).map(|t| t as u32).collect(),
            scored_offsets: vec![start - 1 - from, start - from],
            scored_targets: vec![1, 2],
            scored_corpus_positions: vec![start, start + 1],
        };
        let prompts = QueryPrompts {
            decoded: vec![(0, window(0, 10, true)), (1, window(5, 20, false))],
            past_window: vec![2],
        };
        let texts = [
            (" That's 97.".to_owned(), " That's 97.".to_owned()),
            (" That's 1979.".to_owned(), " That's 97.".to_owned()),
        ];
        let (total, bins, outcomes) = recall_grade(&queries, &prompts, &texts).unwrap();
        let bin = |from, to, queries, decoded, whole_document, reference_states, stated| RecallBin {
            from,
            to,
            queries,
            decoded,
            whole_document,
            reference_states,
            stated,
        };
        assert_eq!(total, bin(0, None, 3, 2, 1, 2, 1));
        assert_eq!(
            bins,
            [
                bin(0, Some(192), 1, 1, 1, 1, 1),
                bin(192, Some(512), 0, 0, 0, 0, 0),
                bin(512, Some(1024), 1, 1, 0, 1, 0),
                bin(1024, None, 1, 0, 0, 0, 0),
            ]
        );
        assert_eq!(outcomes.iter().map(|o| o.stated).collect::<Vec<_>>(), [true, false, false]);
        assert_eq!(outcomes[2].position, 50);
        assert!(recall_grade(&queries, &prompts, &texts[..1]).is_err());
    }

    /// claude-275: the census counts a queries panel's answers found as words of TRAIN's replies,
    /// numbers apart from words.
    #[test]
    fn the_answer_census_counts_numbers_and_words_apart() {
        let train: std::collections::HashSet<String> =
            ["That", "s", "97", "likperkmist"].into_iter().map(str::to_owned).collect();
        let answers: Vec<String> = ["97", "likperkmist", "likperkmist", "zorb", "5468"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        let overlap = answer_overlap("p-long", &answers, &train);
        assert_eq!(
            (overlap.answers, overlap.distinct, overlap.words, overlap.words_in_train),
            (5, 4, 3, 2)
        );
        assert_eq!((overlap.numbers, overlap.numbers_in_train), (2, 1));
        assert_eq!(overlap.found, ["97", "likperkmist"]);
        assert_eq!(overlap.found_queries, [0, 1, 2]);
    }

    /// claude-281: the census names every query it finds, past the 16 values `found` lists.
    #[test]
    fn the_answer_census_names_every_query_it_finds() {
        let train: std::collections::HashSet<String> =
            (0..40).map(|n| n.to_string()).collect();
        let answers: Vec<String> = (0..30).map(|n| (n * 2).to_string()).collect();
        let overlap = answer_overlap("q-tt", &answers, &train);
        assert_eq!(overlap.numbers_in_train, 20);
        assert_eq!(overlap.found.len(), 16);
        assert_eq!(overlap.found_queries, (0..20).collect::<Vec<_>>());
    }

    /// claude-275: `--panel-queries` reads the panel writer's record, and refuses one of another
    /// split or schema.
    #[test]
    fn panel_queries_are_read_only_for_their_own_split() {
        let dir = std::env::temp_dir().join(format!("uor-r4-panel-queries-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("queries.json");
        let record = |schema: &str| {
            serde_json::json!({
                "schema": schema,
                "tokens_sha256": "aa",
                "mask_sha256": "bb",
                "queries": [{
                    "conversation": 0, "turn": 7, "document_start": 0, "reply_start": 12,
                    "reply_tokens": 3, "answer": "staiduk",
                    "chain": {"notes": 2, "queried": 1, "episodes": 3, "episode": 1, "distance": 489, "tokens": 1900}
                }]
            })
        };
        fs::write(&file, serde_json::to_vec(&record("uor-r4-chain-queries/v1")).unwrap()).unwrap();
        let queries = load_panel_queries(&file, "aa", "bb").unwrap();
        assert_eq!((queries[0].reply_start, queries[0].chain.distance), (12, 489));
        assert!(load_panel_queries(&file, "aa", "cc").is_err());
        assert!(load_panel_queries(&file, "ab", "bb").is_err());
        fs::write(&file, serde_json::to_vec(&record("uor-r4-chat-corpus/v1")).unwrap()).unwrap();
        assert!(load_panel_queries(&file, "aa", "bb").is_err());
        fs::remove_dir_all(&dir).unwrap();
    }
}
