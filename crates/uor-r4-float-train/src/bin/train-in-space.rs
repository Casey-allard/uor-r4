//! `train-in-space --data-root DIR --out DIR [--objective sheet|voronoi|nearest|cell] [--steps N]
//! [--train-split DIR --train-sha256 HEX --train-mask-sha256 HEX]
//! [--batch B] [--eval-every K] [--seed S] [--lr X] [--lr-end X] [--decay-mult X]
//! [--retain-rates 0|1] [--fold-scores 0|1] [--reflect-floor 0|1] [--fix-input-table 0|1] [--flock-read K] [--content-read K] [--hamming-read H] [--hamming-copy-key 1|2] [--opening-read K] [--opening-anchor turn|last] [--prompt-register Q] [--register-picks K] [--register-select learned|all] [--width N] [--type-spin 0|1] [--samples N] [--reply-tokens N] [--profile K] [--turn-memory K] [--turn-memory-empty 0|1] [--turn-memory-reply 0|1] [--turn-memory-dropout P] [--fbs-arm one-atom|identity|frozen] [--train-as-served 0|1] [--served-forward 0|1] [--context N]`
//!
//! `--context N` trains at an N-token window, 2 to 4096 (claude-273): TRAIN is cut into N-token
//! windows and the model is built to read N positions. DEV's evaluation keeps the sheet's
//! 192-token windows. 192, the default, writes the same `config.json` as before. `--grad-norms 1`
//! reads TRAIN at the window too. `--score-rows` cuts each `--panel` at N, at most the
//! checkpoint's own window, while DEV and Fresh V2 keep 192, and every decode slides over the
//! checkpoint's own window (claude-274); `--opening-keys`, `--served-key`, `--keyed-decode`,
//! `--decision-trace` and `--decision-force` read 192-token windows and are refused at another
//! N. Every other mode scores 192-token windows and refuses another.
//!
//! `--score-rows` and `--census` also accept `[--panel-queries NAME=FILE ...]` (claude-275): a
//! chained `--panel NAME`'s `queries.json` from `heldout-panel chain=N`. `--score-rows` decodes
//! each query whose reply fits the window, from the window ending at it, with the memory on and
//! off, and writes `samples-NAME-queries-*.json` and the recall grade `recall-NAME-*.json`;
//! `--census` counts the answers found in TRAIN's replies. `--census` also takes `--train-split
//! DIR --train-sha256 HEX --train-mask-sha256 HEX`, and counts against that TRAIN. `--score-rows
//! --context N` is refused from 193 to 1,536, where the row stride is untested.
//!
//! `--score-rows` also accepts `[--sample-opening first|last]` (claude-274): `first`, the
//! default, decodes each sampled window from its first scored target, a reply opening; `last`
//! from its last reply opening with at least 16 tokens of prompt, so a long window gives a long
//! prompt. A `last` reference is the reply's part inside the window, so a reply that runs past
//! the window's end is cut there: 7 of 16 on claude-274's chained panel at 192, none at 2,048.
//!
//! `--train-as-served 1` trains through the graph the integer artifact serves
//! (one-model-end-to-end, claude-258 and deepseek-4): the trunk's tables, Hamming reads and turn
//! memory are rounded as the export rounds them; D11 gates/transition, Q16 residuals, dynamic
//! i16 activations and the Q12 scored query are active; the gradient moves the float values the
//! rounded forward came from; and every checkpoint is scored and saved through that same served
//! forward. It refuses reads or training-only cuts that the export does not carry.
//!
//! `--served-forward 0|1` overrides the saved run mode only for `--inspect`, `--decode-speed` and
//! `--score-rows`, so different training modes can be scored through the same served graph.
//!
//! `--hamming-copy-key 2` (needs `--hamming-read`) keys the read's copy heads on the two tokens
//! before a position, in order, in place of one (`HammingReads::with_copy_key`, claude-261). The
//! checkpoint's `hamming_read.safetensors` records it, so scoring and the export read it there.
//! `--hamming-copy-keys K0,K1` gives each copy head its own key, 1 or 2, one per copy head
//! (`HammingReads::with_copy_keys`, claude-268); the checkpoint records the list when they differ.
//! `--score-rows` additionally accepts `[--batched-decode 0|1]`.
//!
//! `--score-rows` decodes equal-length prompts together on the full-scope nearest arms, except
//! their forced-opening and served-key decodes, and everything else one prompt at a time
//! (`BATCHED_DECODE_DEFAULT`, claude-270). `--batched-decode 0`
//! decodes every prompt one at a time: the serial reference, which gave byte-identical files on
//! an RTX 5090 at two arms and two seeds.
//!
//! `--score-rows` also accepts `[--decode-cache 0|1]`, default 1 (claude-279). The batched decode
//! then fills the exact step cache from each group's prompt forward and computes one position per
//! step from it while the window fits; from the first step that would slide the window, or that
//! the cache refuses, it re-runs the whole window as before. `--decode-cache 0` re-runs the whole
//! window at every step. Its forward counts go to stderr, so stdout stays comparable.
//!
//! `--profile K` times each stage of K training steps on the GPU after 8 of warm-up, each against
//! the unprofiled step after it, into `DIR/profile.jsonl` and `DIR/profile.json`
//! (`in_space_long::profile_window`; fbs-cost-profile, claude-157).
//!
//! `--turn-memory-dropout P` (thousandths, 0 to 1000; needs `--turn-memory`) drops the memory from
//! that share of training windows, which then name no memory row (one-model-end-to-end protocol
//! I-B, claude-207); scoring keeps it everywhere. `--fbs-arm` picks the SpiralCore/FBS transport
//! arm for training and for `--score-rows`, `--inspect` and `--decode-speed` (I-C): `one-atom`, the
//! trained transport (default), `identity` or `frozen` (routing queries kept at their seed).
//! Scoring refuses an arm other than the run's own config.json names.
//!
//! `--turn-memory K` keys each reply opening on the first K ids of its turn and moves its readout
//! point toward the token TRAIN's table stores for that key, through a weight that starts at 0
//! (`flock_read_memory`; route-plus-memory, claude-161). `--turn-memory-empty 1` runs it with an
//! empty table, the no-memory control. `--turn-memory-reply 1` (with `--turn-memory 8`) adds its
//! reply table (turn-memory-reply, claude-193): the rows past each opening, keyed as the
//! `--reply-bound` probe's `turn-8 + position + reply-last-1` at stored share 0.5, with their own
//! weight; with `--turn-memory-empty 1` both tables are empty.
//!
//! `train-in-space --data-root DIR --inspect RUN [--retain-rates 0|1] [--fold-scores 0|1] [--reflect-floor 0|1] [--seed S]`
//!
//! `train-in-space --data-root DIR --reach CHECKPOINT [--retain-rates 0|1] [--fold-scores 0|1] [--reflect-floor 0|1] [--seed S]`
//!
//! `train-in-space --data-root DIR --grad-norms 1 [--objective ...] [--retain-rates 0|1] [--fold-scores 0|1] [--reflect-floor 0|1] [--batch B] [--seed S]`
//!
//! `train-in-space --data-root DIR --decode-speed CHECKPOINT [--objective nearest] [--retain-rates 0|1] [--reply-tokens N]`
//!
//! `train-in-space --recall PAIRS --out DIR [--steps N] [--batch B] [--eval-every K] [--seed S] [--lr X] [--lr-end X] [--retain-rates 0|1] [--flock-read K] [--content-read K] [--hamming-read H] [--opening-read K] [--opening-anchor turn|last] [--prompt-register Q] [--register-picks K] [--register-select learned|all]`:
//! the multi-query associative recall task (`in_space_long::run_recall`), `PAIRS` key-value pairs
//! per distance bucket in each 192-token window, fresh every step; held-out recall per bucket goes
//! to `DIR/recall.jsonl`, and the final model to `DIR/models/`. Needs no `--data-root`, and
//! ignores it if given.
//!
//! `train-in-space --data-root DIR --opening-bound 1 [--panel NAME=DIR]...`: how well the prompt
//! words a window shows could name each reply opening, as TRAIN's commonest opening per key
//! scored on DEV and on each panel, with how often each key fires and its precision when it does,
//! and the oracle `reply-template` key (claude-218). A count over the corpus that needs no GPU;
//! one JSON line per split and key, then one line per split (`dev`, `train`) with how many
//! openings the opening read's turn anchor keeps the first-8 key at.
//!
//! `train-in-space --data-root DIR --reply-bound 1`: what an exact memory of TRAIN could name
//! among DEV's reply rows, for each candidate key and stored-share threshold, by kind (openings,
//! copies, the rest): turn-memory-reply's probe. A count that needs no GPU; one JSON line per
//! key and threshold.
//!
//! `train-in-space --data-root DIR --fresh-v2 DIR [--panel NAME=DIR ...] --census 1`: one-model-end-to-end's data census
//! (claude-202). TRAIN's and the panels' sizes, the probe's pinned counts reproduced, step 2's
//! decision rows, and each panel's exact, n-gram and value-masked template overlap with TRAIN. A
//! count that needs no GPU; one JSON line.
//!
//! `train-in-space --data-root DIR --score-rows CHECKPOINT --out DIR [--fresh-v2 DIR] [--panel NAME=DIR ...] [--samples N] [--reply-tokens N] [--retain-rates 0|1] [--fold-scores 0|1] [--reflect-floor 0|1] [--seed S]`:
//! one-model-end-to-end's per-row scorer (claude-205), on the GPU. Every scored DEV row (and
//! Fresh V2 row) of the step-N checkpoint, with its memory on and taken out, and greedy samples,
//! as JSON files in `--out` (`in_space_long::score_rows`). Prints one JSON line per file. The rate,
//! fold and floor flags must be the checkpoint's run's, as for `--inspect`. Each `--panel` (an unpinned UORT
//! split, its sha recorded) is scored and sampled like DEV; the samples come greedy, and under the
//! Flock rank rule with the memory on, each with its reply-table hits (protocol I-A, I-E, I-F,
//! claude-208). `--flock-cell M,...` (thousandths of a score) adds the cell rule's samples at each
//! margin, with the memory on (claude-213).
//!
//! `train-in-space --data-root DIR --cell-grade 1 --runs DIR --panel NAME=DIR --flock-cell M,...`:
//! grades the cell rule on the laptop (claude-213, `in_space_long::cell_grade`) from `--runs`'
//! `r-drop-s{1,2,3}` score folders: DEV selects the margin, the panel decides. One JSON line.
//!
//! `--reply-read copy,all` adds greedy samples to `--score-rows` with the Hamming read held to the
//! prompt, for the copy heads or for every head (claude-215). `train-in-space --data-root DIR
//! --read-grade 1 --runs DIR --panel NAME=DIR` grades them on the laptop (`in_space_long::read_grade`).
//! One JSON line.
//!
//! `--reply-read-train all|copy` trains with every reply row's Hamming read held to before its
//! reply, from the response mask (claude-216); the run's `config.json` records it, and
//! `--score-rows` reads it back. Since claude-216, `--reply-read`'s decodes hold every reply in
//! the window the same way, an earlier reply in the prompt included. `train-in-space
//! --data-root DIR --cut-grade 1 --runs DIR --panel NAME=DIR` grades (r-cut) against (r-drop) on the
//! laptop (`in_space_long::cut_grade`). One JSON line.
//!
//! `--cycle-step 1` adds `--score-rows` samples under the cycle step (claude-217): the nearest
//! token that closes no 4-gram the reply already holds, alone and under each `--reply-read`
//! scope. `train-in-space --data-root DIR --cycle-grade 1 --runs DIR --panel NAME=DIR` grades
//! them, and the decode-only cut, against greedy on the laptop (`in_space_long::cycle_grade`).
//! One JSON line.
//!
//! `--opening-keys 1` adds to `--score-rows` the opening keys (claude-218): TRAIN's opening points
//! under the trunk, with the memory off, and each split's openings named by the nearest of them
//! under L1 and by sign pattern (`keys-{split}.json`), and a forced-opening greedy arm, each
//! reply's first token given as its reference's (`samples-{split}-forced-memory.json`).
//!
//! `--keyed-decode sign=T,hsign=T,point=T`, with `--opening-keys 1`, adds a keyed arm per trunk key
//! (claude-219): greedy with the memory on, the reply's first token the key's name where the
//! memory's exact key misses the opening and the key's distance is at most τ (`inf`: wherever it
//! misses), each forced token kept as the sample's `keyed` (`samples-{split}-keyed-{key}-memory.json`).
//! `train-in-space --data-root DIR --keyed-grade 1 --runs DIR [--keys-runs DIR] --panel p-v=DIR`
//! grades them against greedy on the laptop (`in_space_long::keyed_grade`); with `--panel p-u=DIR`
//! on claude-218's run, it predicts their exact replies, the sheet's power check. One JSON line.
//!
//! `--served-key 1`, with `--opening-keys 1`, adds the served arm (claude-220): greedy with the
//! memory on, and the point key named in the decode itself from the opening row's own point
//! against TRAIN's opening table (`samples-{split}-served-memory.json`). The table is built,
//! saved to `--out`'s `opening-table.safetensors`, read back to the same bits, and served.
//! `train-in-space --data-root DIR --served-grade 1 --runs DIR --panel p-w=DIR --panel p-v=DIR`
//! grades the served arm against greedy on the laptop (`in_space_long::served_grade`). One JSON line.
//!
//! `--decision-trace SPLIT=POSITIONS_JSON`, once per split, adds to `--score-rows` Q5's decision
//! trace on the float side (integer-nearest-readout, claude-251): for the windows of the scored
//! split `SPLIT` whose first scored position the JSON array names, every decision of claude-247's
//! five kinds in the memory pass's own forward, written new as `decision-trace-{SPLIT}.json`
//! (`uor-r4.decision-trace/1`, side `F`, the f32 values as `f32::to_bits`). It changes no rows
//! file, and refuses a position that names no window and a checkpoint with a read R does not serve.
//! On an evaluation copy rounded as exported (`fbs-snapshot --round-as-exported`, claude-252) the
//! trace is side `T`, and a copy whose files are not the ones its marker names is refused.
//!
//! `--decision-force SPLIT=FORCE_JSON` (claude-253, Q5's forced replay), beside the same split's
//! `--decision-trace`, forces the windows the `uor-r4.decision-force/1` file names, in the forward
//! that traces them: each forced rate bucket replaces the entry its lane's score chooses, and each
//! forced code the computed one, as `fbs-toks --forced --force` forces R's. The trace records them
//! as used, and refuses a run in which one was not taken; that split's memory-pass rows are the
//! forced forward's. Refused, before any forward: a position that is not traced and R's refusals.
//!
//! `train-in-space --data-root DIR --panel-prompts 1 --out DIR [--panel NAME=DIR ...] [--runs DIR] --samples 1536 --reply-tokens 64`
//! (point-key-integer, claude-224) writes, with no model, the prompts a `--score-rows` run decodes on
//! DEV and each panel (`prompts-{split}.json`), for `fbs-toks --replies`. With `--runs` (claude-220's
//! `out`), it also writes each seed's prompts with its float served names, refusing a served sample
//! that is not its prompt's (`in_space_long::panel_prompts`). The `--samples` and `--reply-tokens`
//! must be the scored run's. `train-in-space --data-root DIR --integer-grade 1 --runs DIR
//! --keys-runs DIR [--panel NAME=DIR ...]` grades the integer served arm against integer greedy,
//! with parity against the float key, from each `r-drop-s{n}`'s
//! `integer-{served,greedy}-{split}.json` (`in_space_long::integer_grade`). `--keys-runs` is the
//! float served run, claude-220's `out`, whose names each integer served file must carry for its
//! seed. One JSON line.
//!
//! `train-in-space --data-root DIR --controls 1 --out DIR [--fresh-v2 DIR] [--panel NAME=DIR ...]`: the
//! protocol's recall controls (I-D, claude-211) on the laptop: (T) the memory's tables and (S) the
//! longest suffix over TRAIN, as rows files the grader pairs with the arms.
//!
//! `train-in-space --data-root DIR --grade 1 --controls-dir DIR --runs DIR --panel p-dev=DIR
//! [--frozen-probe RUN]`: grades the protocol's training run on the laptop (I-G, claude-211,
//! `in_space_long::grade`): `--runs` holds one score folder per `ARM-sSEED`, `--controls-dir` the
//! controls' files, `--frozen-probe` (r-frozen)'s 256-step probe run for I-C. One JSON line.
//!
//! `train-in-space --data-root DIR --overturn 1 --scored ARM-sSEED=DIR ...`: grades the overturn
//! check from those files, on the laptop (`in_space_long::overturn`). Needs `r-s1`, `r0-s1`,
//! `r-s2` and `r0-s2`; counts only; one JSON line.
//!
//! Long training of the SpiralCore/FBS geometric trunk in its own space, with text at every
//! checkpoint (`uor_r4_float_train::in_space_long`). GPU only. `--data-root` holds `tokenizer.json`,
//! `mw-balp/train/` and `ft/dev/`, each split with `tokens.u16` and `response_mask.u8`. `--out`
//! must not exist; it receives `config.json`, `progress.jsonl`, `samples.md`, `samples.jsonl` and
//! `models/step-*/`. `--inspect` instead reads every checkpoint a run saved and prints one JSON
//! line each, after one line with the address roots' separation.
//!
//! `train-in-space --data-root DIR --copy-key-oracle TOKENS MASK`: measures copy keys on the data
//! without training (claude-267). `TOKENS` and `MASK` are an unpinned UORT pair (`tokens.u16`,
//! `response_mask.u8`) cut into DEV scoring's windows; TRAIN and `tokenizer.json` come from
//! `--data-root`. At every copyable reply position (the next token appears earlier in its window)
//! it takes the latest earlier position whose key equals the query exactly, under copy key 1
//! (`k1`), copy key 2 (`k2`) and the slot key (`slot`: the token and its clause's rarest token,
//! `flock_read_hamming::slot_atoms`), and counts how often that position's token is the next one.
//! Two diagnostic rows follow, outside claude-267's rule: `atom2` keys on the token and its
//! clause's two rarest tokens, unordered (`slot_atom_pairs`), and `win4` on the token and the
//! rarest of the four tokens ending at it (`window_atoms`). A third, `k2>k1`, takes copy key 2's
//! candidate where it has one and copy key 1's elsewhere. Needs no GPU; one header line, then one
//! `copy-key-oracle` line per rule.
use std::env;
use std::path::{Path, PathBuf};

use candle_core::Device;
use uor_r4_float_train::flock_read_opening::OpeningAnchor;
use uor_r4_float_train::flock_read_register::RegisterSelect;
use uor_r4_float_train::in_space_long::{
    cell_grade, cells_ok, census, context_reach, controls, cut_grade, cycle_grade, decode_speed, key_grade, keyed_grade, served_grade, integer_grade, panel_prompts, fbs_arm_named, grade,
    gradient_norms, inspect_run, opening_bound, overturn, read_grade, reply_bound, root_separation,
    run_long, run_recall, score_rows, scopes_ok, set_decode_cache, take_decode_forwards,
    DecodeRule, LongConfig, PointObjective, ReadScope, TrainSplit, DEFAULT_REGISTER_PICKS,
    MEMORY_DROPOUT_MAX,
};
use uor_r4_float_train::{FloatTrainError, Result};
use uor_r4_integer::fbs_snapshot::DecisionForce;

/// Whether `--score-rows` batches equal-length prompts when `--batched-decode` is not given.
const BATCHED_DECODE_DEFAULT: bool = true;

/// The `--batched-decode` switch as given, or [`BATCHED_DECODE_DEFAULT`] when it is not.
fn batched_decode_or_default(given: Option<bool>) -> bool {
    given.unwrap_or(BATCHED_DECODE_DEFAULT)
}

fn main() -> Result<()> {
    let mut data_root = None;
    let mut key_oracle: Option<(PathBuf, PathBuf)> = None;
    let mut out = None;
    let mut inspect = None;
    let mut reach = None;
    let mut grad_norms = false;
    let mut decode = None;
    let mut bound = false;
    let mut reply = false;
    let mut census_on = false;
    let mut fresh_v2 = None;
    let mut extra_panels: Vec<(String, PathBuf)> = Vec::new();
    let mut panel_queries: Vec<(String, PathBuf)> = Vec::new();
    let mut traces: Vec<(String, Vec<usize>)> = Vec::new();
    let mut forces: Vec<(String, DecisionForce)> = Vec::new();
    let mut score = None;
    let mut overturn_on = false;
    let mut controls_on = false;
    let mut grade_on = false;
    let mut flock_end = false;
    let mut cell_grade_on = false;
    let mut cells: Vec<u32> = Vec::new();
    let mut read_grade_on = false;
    let mut cut_grade_on = false;
    let mut cycle_on = false;
    let mut keys_on = false;
    let mut keyed: Vec<(String, f32)> = Vec::new();
    let mut served_on = false;
    let mut cycle_grade_on = false;
    let mut key_grade_on = false;
    let mut keyed_grade_on = false;
    let mut served_grade_on = false;
    let mut panel_prompts_on = false;
    let mut integer_grade_on = false;
    let mut served_forward = None;
    let mut batched_decode = None;
    let mut decode_cache = true;
    let mut last_opening = None;
    let mut key_levels: Vec<(String, u8)> = Vec::new();
    let mut keys_runs: Option<PathBuf> = None;
    let mut scopes: Vec<ReadScope> = Vec::new();
    let (mut controls_dir, mut runs, mut frozen_probe) = (None, None, None);
    let mut scored: Vec<(String, PathBuf)> = Vec::new();
    let (mut split_dir, mut split_tokens, mut split_mask) = (None, None, None);
    let mut recall = 0usize;
    let (mut opening_read, mut opening_anchor) = (None, None);
    let (mut prompt_register, mut register_picks, mut register_select) = (None, None, None);
    let mut config = LongConfig {
        objective: PointObjective::Sheet,
        steps: 32_768,
        batch: 16,
        eval_every: 2_048,
        seed: 1,
        lr: 0.002,
        lr_end: f64::NAN,
        decay_mult: 20.0,
        retain_rates: false,
        fold_scores: false,
        reflect_floor: false,
        fix_input_table: false,
        train_split: None,
        flock_read: 0,
        content_read: 0,
        hamming_read: 0,
        opening_read: 0,
        opening_anchor: None,
        prompt_register: 0,
        register_picks: None,
        register_select: None,
        width: uor_r4_float_train::hamilton_sheet::SHEET_WIDTH,
        type_spin: false,
        hamming_copy_key: 1,
        hamming_copy_keys: None,
        samples: 6,
        reply_tokens: 48,
        profile: 0,
        turn_memory: 0,
        turn_memory_empty: false,
        turn_memory_reply: false,
        turn_memory_dropout: 0,
        fbs_arm: uor_r4_float_train::spiralcore_fbs_arms::FbsArm::OneAtomActive,
        reply_read_train: None,
        train_as_served: false,
        context: uor_r4_float_train::hamilton_sheet::SHEET_CONTEXT,
    };
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| FloatTrainError::Invalid(format!("missing {arg} value")))?;
        let number = |v: &str| {
            v.parse::<usize>()
                .map_err(|_| FloatTrainError::Invalid(format!("{arg} wants a whole number")))
        };
        let switch = |v: &str| match v {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err(FloatTrainError::Invalid(format!("{arg} wants 0 or 1"))),
        };
        match arg.as_str() {
            "--data-root" => data_root = Some(PathBuf::from(&value)),
            "--copy-key-oracle" => {
                let mask = args.next().ok_or_else(|| {
                    FloatTrainError::Invalid(
                        "--copy-key-oracle wants TOKENS and MASK paths".to_owned(),
                    )
                })?;
                key_oracle = Some((PathBuf::from(&value), PathBuf::from(mask)));
            }
            "--out" => out = Some(PathBuf::from(&value)),
            "--inspect" => inspect = Some(PathBuf::from(&value)),
            "--reach" => reach = Some(PathBuf::from(&value)),
            "--grad-norms" => grad_norms = switch(&value)?,
            "--decode-speed" => decode = Some(PathBuf::from(&value)),
            "--opening-bound" => bound = switch(&value)?,
            "--reply-bound" => reply = switch(&value)?,
            "--census" => census_on = switch(&value)?,
            "--score-rows" => score = Some(PathBuf::from(&value)),
            "--overturn" => overturn_on = switch(&value)?,
            "--controls" => controls_on = switch(&value)?,
            "--grade" => grade_on = switch(&value)?,
            "--flock-end" => flock_end = switch(&value)?,
            "--cell-grade" => cell_grade_on = switch(&value)?,
            "--read-grade" => read_grade_on = switch(&value)?,
            "--cut-grade" => cut_grade_on = switch(&value)?,
            "--cycle-step" => cycle_on = switch(&value)?,
            "--opening-keys" => keys_on = switch(&value)?,
            "--keyed-decode" => keyed = keyed_decode_arg(&value)?,
            "--served-key" => served_on = switch(&value)?,
            "--cycle-grade" => cycle_grade_on = switch(&value)?,
            "--key-grade" => key_grade_on = switch(&value)?,
            "--keyed-grade" => keyed_grade_on = switch(&value)?,
            "--served-grade" => served_grade_on = switch(&value)?,
            "--panel-prompts" => panel_prompts_on = switch(&value)?,
            "--integer-grade" => integer_grade_on = switch(&value)?,
            "--key-levels" => key_levels = key_levels_arg(&value)?,
            "--keys-runs" => keys_runs = Some(PathBuf::from(&value)),
            "--reply-read-train" => {
                config.reply_read_train = Some(match value.as_str() {
                    "copy" => ReadScope::Copy,
                    "all" => ReadScope::All,
                    _ => {
                        return Err(FloatTrainError::Invalid(
                            "--reply-read-train wants copy or all".to_owned(),
                        ))
                    }
                });
            }
            "--reply-read" => {
                scopes = value
                    .split(',')
                    .map(|s| match s {
                        "copy" => Ok(ReadScope::Copy),
                        "all" => Ok(ReadScope::All),
                        _ => Err(FloatTrainError::Invalid(
                            "--reply-read wants copy and/or all, comma-separated".to_owned(),
                        )),
                    })
                    .collect::<Result<_>>()?;
                if scopes.is_empty() || !scopes_ok(&scopes) {
                    return Err(FloatTrainError::Invalid(
                        "--reply-read wants distinct scopes from copy and all".to_owned(),
                    ));
                }
            }
            "--flock-cell" => {
                cells = value
                    .split(',')
                    .map(|m| m.parse::<u32>())
                    .collect::<std::result::Result<_, _>>()
                    .map_err(|_| {
                        FloatTrainError::Invalid(
                            "--flock-cell wants margins in thousandths, comma-separated".to_owned(),
                        )
                    })?;
                if cells.is_empty() || !cells_ok(&cells) {
                    return Err(FloatTrainError::Invalid(
                        "--flock-cell wants distinct margins of at most 4,000 thousandths".to_owned(),
                    ));
                }
            }
            "--controls-dir" => controls_dir = Some(PathBuf::from(&value)),
            "--runs" => runs = Some(PathBuf::from(&value)),
            "--frozen-probe" => frozen_probe = Some(PathBuf::from(&value)),
            "--scored" => {
                let arm = panel_arg(&value)?;
                if scored.iter().any(|(name, _)| *name == arm.0) {
                    return Err(FloatTrainError::Invalid(format!(
                        "--scored {} is given twice",
                        arm.0
                    )));
                }
                scored.push(arm);
            }
            "--fresh-v2" => fresh_v2 = Some(PathBuf::from(&value)),
            "--decision-trace" => traces.push(decision_trace_arg(&value)?),
            "--decision-force" => forces.push(decision_force_arg(&value)?),
            "--panel" => {
                let panel = panel_arg(&value)?;
                if extra_panels.iter().any(|(name, _)| *name == panel.0) {
                    return Err(FloatTrainError::Invalid(format!(
                        "--panel {} is given twice",
                        panel.0
                    )));
                }
                extra_panels.push(panel);
            }
            "--panel-queries" => {
                let queries = panel_arg(&value)?;
                if panel_queries.iter().any(|(name, _)| *name == queries.0) {
                    return Err(FloatTrainError::Invalid(format!(
                        "--panel-queries {} is given twice",
                        queries.0
                    )));
                }
                panel_queries.push(queries);
            }
            "--recall" => recall = number(&value)?,
            "--objective" => {
                config.objective = match value.as_str() {
                    "sheet" => PointObjective::Sheet,
                    "voronoi" => PointObjective::Voronoi,
                    "nearest" => PointObjective::Nearest,
                    "cell" => PointObjective::Cell,
                    other => {
                        return Err(FloatTrainError::Invalid(format!(
                            "--objective wants sheet, voronoi, nearest or cell, not {other}"
                        )))
                    }
                }
            }
            "--steps" => config.steps = number(&value)?,
            "--profile" => config.profile = number(&value)?,
            "--turn-memory" => config.turn_memory = number(&value)?,
            "--turn-memory-empty" => config.turn_memory_empty = switch(&value)?,
            "--turn-memory-reply" => config.turn_memory_reply = switch(&value)?,
            "--turn-memory-dropout" => {
                config.turn_memory_dropout = u32::try_from(number(&value)?)
                    .ok()
                    .filter(|&p| p <= MEMORY_DROPOUT_MAX)
                    .ok_or_else(|| {
                        FloatTrainError::Invalid(
                            "--turn-memory-dropout wants thousandths, 0 to 1000".to_owned(),
                        )
                    })?
            }
            "--fbs-arm" => {
                config.fbs_arm = fbs_arm_named(&value).ok_or_else(|| {
                    FloatTrainError::Invalid(format!(
                        "--fbs-arm wants one-atom, identity or frozen, not {value}"
                    ))
                })?
            }
            "--train-as-served" => config.train_as_served = switch(&value)?,
            "--served-forward" => served_forward = Some(switch(&value)?),
            "--batched-decode" => batched_decode = Some(switch(&value)?),
            "--decode-cache" => decode_cache = switch(&value)?,
            "--sample-opening" => {
                last_opening = Some(match value.as_str() {
                    "first" => false,
                    "last" => true,
                    other => {
                        return Err(FloatTrainError::Invalid(format!(
                            "--sample-opening wants first or last, not {other}"
                        )))
                    }
                })
            }
            "--batch" => config.batch = number(&value)?,
            "--eval-every" => config.eval_every = number(&value)?,
            "--seed" => config.seed = number(&value)? as u64,
            "--samples" => config.samples = number(&value)?,
            "--reply-tokens" => config.reply_tokens = number(&value)?,
            "--retain-rates" => config.retain_rates = switch(&value)?,
            "--fold-scores" => config.fold_scores = switch(&value)?,
            "--reflect-floor" => config.reflect_floor = switch(&value)?,
            "--fix-input-table" => config.fix_input_table = switch(&value)?,
            "--flock-read" => config.flock_read = number(&value)?,
            "--content-read" => config.content_read = number(&value)?,
            "--hamming-read" => config.hamming_read = number(&value)?,
            "--hamming-copy-key" => config.hamming_copy_key = number(&value)?,
            "--hamming-copy-keys" => {
                config.hamming_copy_keys = Some(
                    value
                        .split(',')
                        .map(number)
                        .collect::<Result<_>>()?,
                )
            }
            "--opening-read" => match number(&value)? {
                k @ 1..=16 => opening_read = Some(k),
                _ => {
                    return Err(FloatTrainError::Invalid(
                        "--opening-read wants 1 to 16".to_owned(),
                    ))
                }
            },
            "--opening-anchor" => {
                opening_anchor = Some(match value.as_str() {
                    "turn" => OpeningAnchor::Turn,
                    "last" => OpeningAnchor::Last,
                    other => {
                        return Err(FloatTrainError::Invalid(format!(
                            "--opening-anchor wants turn or last, not {other}"
                        )))
                    }
                })
            }
            "--prompt-register" => match number(&value)? {
                q @ 1..=16 => prompt_register = Some(q),
                _ => {
                    return Err(FloatTrainError::Invalid(
                        "--prompt-register wants 1 to 16".to_owned(),
                    ))
                }
            },
            "--register-picks" => match number(&value)? {
                k @ 1..=32 => register_picks = Some(k),
                _ => {
                    return Err(FloatTrainError::Invalid(
                        "--register-picks wants 1 to 32".to_owned(),
                    ))
                }
            },
            "--register-select" => {
                register_select = Some(match value.as_str() {
                    "learned" => RegisterSelect::Learned,
                    "all" => RegisterSelect::All,
                    other => {
                        return Err(FloatTrainError::Invalid(format!(
                            "--register-select wants learned or all, not {other}"
                        )))
                    }
                })
            }
            "--width" => config.width = number(&value)?,
            "--context" => {
                config.context = uor_r4_float_train::in_space_long::training_context(number(&value)?)?
            }
            "--type-spin" => config.type_spin = switch(&value)?,
            "--train-split" => split_dir = Some(PathBuf::from(&value)),
            "--train-sha256" => split_tokens = Some(value.clone()),
            "--train-mask-sha256" => split_mask = Some(value.clone()),
            "--lr" | "--lr-end" | "--decay-mult" => {
                let v: f64 = value
                    .parse()
                    .map_err(|_| FloatTrainError::Invalid(format!("{arg} wants a number")))?;
                match arg.as_str() {
                    "--lr" => config.lr = v,
                    "--lr-end" => config.lr_end = v,
                    _ => config.decay_mult = v,
                }
            }
            other => {
                return Err(FloatTrainError::Invalid(format!(
                    "unrecognized argument: {other}"
                )))
            }
        }
    }
    // The opening read (claude-154): its anchor is written exactly when the read is on.
    match (opening_read, opening_anchor) {
        (Some(k), anchor) => {
            config.opening_read = k;
            config.opening_anchor = Some(anchor.unwrap_or_default());
        }
        (None, Some(_)) => {
            return Err(FloatTrainError::Invalid(
                "--opening-anchor needs --opening-read".to_owned(),
            ))
        }
        (None, None) => {}
    }
    // The prompt register (claude-155): its picks and select are written exactly when it is on.
    match (prompt_register, register_picks, register_select) {
        (Some(q), _, Some(RegisterSelect::All)) if q != 1 => {
            return Err(FloatTrainError::Invalid(
                "--register-select all wants --prompt-register 1".to_owned(),
            ))
        }
        (Some(q), picks, select) => {
            config.prompt_register = q;
            config.register_picks = Some(picks.unwrap_or(DEFAULT_REGISTER_PICKS));
            config.register_select = Some(select.unwrap_or_default());
        }
        (None, None, None) => {}
        (None, _, _) => {
            return Err(FloatTrainError::Invalid(
                "--register-picks and --register-select need --prompt-register".to_owned(),
            ))
        }
    }
    if config.lr_end.is_nan() {
        config.lr_end = config.lr;
    }
    config.train_split = match (split_dir, split_tokens, split_mask) {
        (None, None, None) => None,
        (Some(dir), Some(tokens_sha256), Some(mask_sha256)) => Some(TrainSplit {
            dir,
            tokens_sha256,
            mask_sha256,
        }),
        _ => {
            return Err(FloatTrainError::Invalid(
                "--train-split needs --train-sha256 and --train-mask-sha256, and they need it"
                    .to_owned(),
            ))
        }
    };
    let modes = [
        inspect.is_some(),
        reach.is_some(),
        grad_norms,
        decode.is_some(),
        bound,
        reply,
        census_on,
        // `--score-rows`, `--controls` and `--panel-prompts` write to `--out` without training.
        out.is_some() && score.is_none() && !controls_on && !panel_prompts_on,
        score.is_some(),
        overturn_on,
        controls_on,
        grade_on,
        cell_grade_on,
        read_grade_on,
        cut_grade_on,
        cycle_grade_on,
        key_grade_on,
        keyed_grade_on,
        served_grade_on,
        panel_prompts_on,
        integer_grade_on,
        key_oracle.is_some(),
    ];
    if modes.iter().filter(|&&m| m).count() > 1 {
        return Err(FloatTrainError::Invalid(
            "give one of --out, --inspect, --reach, --grad-norms 1, --decode-speed, --opening-bound 1, --reply-bound 1, --census 1, --score-rows, --overturn 1, --controls 1, --grade 1, --cell-grade 1, --read-grade 1, --cut-grade 1, --cycle-grade 1, --key-grade 1, --keyed-grade 1, --served-grade 1, --panel-prompts 1, --integer-grade 1 or --copy-key-oracle"
                .to_owned(),
        ));
    }
    // Training as served is a training switch; no other mode would read it.
    if config.train_as_served && (!modes[7] || recall > 0) {
        return Err(FloatTrainError::Invalid(
            "--train-as-served applies to training with --out only".to_owned(),
        ));
    }
    served_forward_applies(
        served_forward,
        inspect.is_some(),
        decode.is_some(),
        score.is_some(),
    )?;
    context_applies(config.context, modes[7] && recall == 0, grad_norms, score.is_some())?;
    if last_opening.is_some() && score.is_none() {
        return Err(FloatTrainError::Invalid(
            "--sample-opening applies to --score-rows only".to_owned(),
        ));
    }
    if !panel_queries.is_empty() && score.is_none() && !census_on {
        return Err(FloatTrainError::Invalid(
            "--panel-queries applies to --score-rows and --census only".to_owned(),
        ));
    }
    if overturn_on && fresh_v2.is_some() {
        return Err(FloatTrainError::Invalid(
            "--overturn reads Fresh V2 from the scored files; it takes no --fresh-v2".to_owned(),
        ));
    }
    if !scored.is_empty() && !overturn_on {
        return Err(FloatTrainError::Invalid(
            "--scored needs --overturn 1".to_owned(),
        ));
    }
    // These modes run the default transport only; another arm would be scored or trained wrong.
    if (recall > 0 || grad_norms || reach.is_some())
        && config.fbs_arm != uor_r4_float_train::spiralcore_fbs_arms::FbsArm::OneAtomActive
    {
        return Err(FloatTrainError::Invalid(
            "--fbs-arm applies to training, --score-rows, --inspect and --decode-speed; --recall, --grad-norms and --reach run one-atom only".to_owned(),
        ));
    }
    if recall > 0 {
        let out = out.ok_or_else(|| FloatTrainError::Invalid("--recall needs --out".to_owned()))?;
        if modes[..5].iter().any(|&m| m)
            || score.is_some()
            || overturn_on
            || controls_on
            || grade_on
            || flock_end
            || cell_grade_on
            || !cells.is_empty()
            || read_grade_on
            || cut_grade_on
            || cycle_on
            || keys_on
            || cycle_grade_on
            || key_grade_on
            || keyed_grade_on
            || served_grade_on
            || panel_prompts_on
            || integer_grade_on
            || config.reply_read_train.is_some()
            || !scopes.is_empty()
            || key_oracle.is_some()
        {
            return Err(FloatTrainError::Invalid(
                "--recall trains; it takes --out and no other mode".to_owned(),
            ));
        }
        let device = Device::new_cuda(0)?;
        return run_recall(&out, &config, recall, &device);
    }
    let data_root =
        data_root.ok_or_else(|| FloatTrainError::Invalid("--data-root is required".to_owned()))?;
    if let Some((tokens, mask)) = &key_oracle {
        return copy_key_oracle(&data_root, tokens, mask);
    }
    if flock_end && score.is_none() && !grade_on {
        return Err(FloatTrainError::Invalid(
            "--flock-end applies to --score-rows and --grade 1 only".to_owned(),
        ));
    }
    if !cells.is_empty() && score.is_none() && !cell_grade_on {
        return Err(FloatTrainError::Invalid(
            "--flock-cell applies to --score-rows and --cell-grade 1 only".to_owned(),
        ));
    }
    if !scopes.is_empty() && score.is_none() {
        return Err(FloatTrainError::Invalid(
            "--reply-read applies to --score-rows only".to_owned(),
        ));
    }
    if cycle_on && score.is_none() {
        return Err(FloatTrainError::Invalid(
            "--cycle-step applies to --score-rows only".to_owned(),
        ));
    }
    if keys_on && score.is_none() {
        return Err(FloatTrainError::Invalid(
            "--opening-keys applies to --score-rows only".to_owned(),
        ));
    }
    if !keyed.is_empty() && !keys_on {
        return Err(FloatTrainError::Invalid(
            "--keyed-decode applies to --score-rows with --opening-keys 1 only".to_owned(),
        ));
    }
    if served_on && !keys_on {
        return Err(FloatTrainError::Invalid(
            "--served-key applies to --score-rows with --opening-keys 1 only".to_owned(),
        ));
    }
    if !traces.is_empty() && score.is_none() {
        return Err(FloatTrainError::Invalid(
            "--decision-trace applies to --score-rows only".to_owned(),
        ));
    }
    if !forces.is_empty() && score.is_none() {
        return Err(FloatTrainError::Invalid(
            "--decision-force applies to --score-rows only".to_owned(),
        ));
    }
    if cycle_grade_on {
        let runs = runs
            .ok_or_else(|| FloatTrainError::Invalid("--cycle-grade needs --runs DIR".to_owned()))?;
        let [(panel, _)] = extra_panels.as_slice() else {
            return Err(FloatTrainError::Invalid(
                "--cycle-grade needs one --panel NAME=DIR, the deciding panel".to_owned(),
            ));
        };
        let graded = cycle_grade(&data_root, &runs, panel)?;
        println!("{}", serde_json::to_string(&graded)?);
        return Ok(());
    }
    if key_grade_on {
        let runs = runs
            .ok_or_else(|| FloatTrainError::Invalid("--key-grade needs --runs DIR".to_owned()))?;
        let graded = key_grade(&data_root, &runs, keys_runs.as_deref(), &extra_panels, &key_levels)?;
        println!("{}", serde_json::to_string(&graded)?);
        return Ok(());
    }
    if panel_prompts_on {
        let out = out
            .ok_or_else(|| FloatTrainError::Invalid("--panel-prompts needs --out DIR".to_owned()))?;
        for (file, rows) in panel_prompts(&data_root, &extra_panels, config.samples, config.reply_tokens, runs.as_deref(), &out)? {
            println!("{}", serde_json::json!({ "file": file, "rows": rows }));
        }
        return Ok(());
    }
    if integer_grade_on {
        let runs = runs
            .ok_or_else(|| FloatTrainError::Invalid("--integer-grade needs --runs DIR".to_owned()))?;
        let names: Vec<String> = extra_panels.iter().map(|(name, _)| name.clone()).collect();
        let float_runs = keys_runs
            .as_deref()
            .ok_or_else(|| FloatTrainError::Invalid("--integer-grade needs --keys-runs DIR, the float served run whose names parity reads".to_owned()))?;
        let graded = integer_grade(&data_root, &runs, &names, float_runs)?;
        println!("{}", serde_json::to_string(&graded)?);
        return Ok(());
    }
    if served_grade_on {
        if !key_levels.is_empty() || keys_runs.is_some() {
            return Err(FloatTrainError::Invalid(
                "--served-grade reads one run's keys at +∞; it takes no --key-levels or --keys-runs".to_owned(),
            ));
        }
        let runs = runs
            .ok_or_else(|| FloatTrainError::Invalid("--served-grade needs --runs DIR".to_owned()))?;
        let graded = served_grade(&data_root, &runs, &extra_panels)?;
        println!("{}", serde_json::to_string(&graded)?);
        return Ok(());
    }
    if keyed_grade_on {
        if !key_levels.is_empty() {
            return Err(FloatTrainError::Invalid(
                "--keyed-grade reads its τ from the sheet; it takes no --key-levels".to_owned(),
            ));
        }
        let runs = runs
            .ok_or_else(|| FloatTrainError::Invalid("--keyed-grade needs --runs DIR".to_owned()))?;
        let graded = keyed_grade(&data_root, &runs, keys_runs.as_deref(), &extra_panels)?;
        println!("{}", serde_json::to_string(&graded)?);
        return Ok(());
    }
    if !key_levels.is_empty() || keys_runs.is_some() {
        return Err(FloatTrainError::Invalid(
            "--key-levels and --keys-runs apply to --key-grade (and --keys-runs to --keyed-grade) only".to_owned(),
        ));
    }
    // A run trains with the cut; scoring reads it back from the run's own config.
    if config.reply_read_train.is_some() && !(out.is_some() && score.is_none() && !controls_on) {
        return Err(FloatTrainError::Invalid(
            "--reply-read-train applies to training only; scoring reads it from the run".to_owned(),
        ));
    }
    if cut_grade_on {
        let runs =
            runs.ok_or_else(|| FloatTrainError::Invalid("--cut-grade needs --runs DIR".to_owned()))?;
        let [(panel, _)] = extra_panels.as_slice() else {
            return Err(FloatTrainError::Invalid(
                "--cut-grade needs one --panel NAME=DIR, the deciding panel".to_owned(),
            ));
        };
        let graded = cut_grade(&data_root, &runs, panel)?;
        println!("{}", serde_json::to_string(&graded)?);
        return Ok(());
    }
    if read_grade_on {
        let runs =
            runs.ok_or_else(|| FloatTrainError::Invalid("--read-grade needs --runs DIR".to_owned()))?;
        let [(panel, _)] = extra_panels.as_slice() else {
            return Err(FloatTrainError::Invalid(
                "--read-grade needs one --panel NAME=DIR, the deciding panel".to_owned(),
            ));
        };
        let graded = read_grade(&data_root, &runs, panel)?;
        println!("{}", serde_json::to_string(&graded)?);
        return Ok(());
    }
    if cell_grade_on {
        let runs =
            runs.ok_or_else(|| FloatTrainError::Invalid("--cell-grade needs --runs DIR".to_owned()))?;
        let [(panel, _)] = extra_panels.as_slice() else {
            return Err(FloatTrainError::Invalid(
                "--cell-grade needs one --panel NAME=DIR, the confirming panel".to_owned(),
            ));
        };
        if cells.is_empty() {
            return Err(FloatTrainError::Invalid(
                "--cell-grade needs --flock-cell's margins".to_owned(),
            ));
        }
        let graded = cell_grade(&data_root, &runs, panel, &cells)?;
        println!("{}", serde_json::to_string(&graded)?);
        return Ok(());
    }
    if grade_on {
        let need = |v: Option<PathBuf>, flag: &str| {
            v.ok_or_else(|| FloatTrainError::Invalid(format!("--grade needs {flag} DIR")))
        };
        let p_dev = extra_panels
            .iter()
            .find(|(name, _)| name == "p-dev")
            .map(|(_, dir)| dir.clone());
        let graded = grade(
            &data_root,
            &need(controls_dir, "--controls-dir")?,
            &need(runs, "--runs")?,
            &need(p_dev, "--panel p-dev=")?,
            frozen_probe.as_deref(),
            flock_end,
        )?;
        println!("{}", serde_json::to_string(&graded)?);
        return Ok(());
    }
    if controls_on {
        let out =
            out.ok_or_else(|| FloatTrainError::Invalid("--controls needs --out DIR".to_owned()))?;
        for file in controls(&data_root, &out, fresh_v2.as_deref(), &extra_panels)? {
            println!("{}", serde_json::to_string(&file)?);
        }
        return Ok(());
    }
    if overturn_on {
        println!("{}", serde_json::to_string(&overturn(&data_root, &scored)?)?);
        return Ok(());
    }
    if census_on {
        let fresh_v2 = fresh_v2
            .ok_or_else(|| FloatTrainError::Invalid("--census needs --fresh-v2 DIR".to_owned()))?;
        println!(
            "{}",
            serde_json::to_string(&census(
                &data_root,
                &fresh_v2,
                &extra_panels,
                config.train_split.as_ref(),
                &panel_queries,
            )?)?
        );
        return Ok(());
    }
    if reply {
        for row in reply_bound(&data_root)? {
            println!("{}", serde_json::to_string(&row)?);
        }
        return Ok(());
    }
    if bound {
        let (rows, checks) = opening_bound(&data_root, &extra_panels)?;
        for row in rows {
            println!("{}", serde_json::to_string(&row)?);
        }
        for check in checks {
            println!("{}", serde_json::to_string(&check)?);
        }
        return Ok(());
    }
    let device = Device::new_cuda(0)?;
    if let Some(checkpoint) = score {
        let out = out
            .ok_or_else(|| FloatTrainError::Invalid("--score-rows needs --out DIR".to_owned()))?;
        set_decode_cache(decode_cache);
        take_decode_forwards();
        for file in score_rows(
            &data_root,
            &checkpoint,
            &out,
            fresh_v2.as_deref(),
            &extra_panels,
            config.samples,
            config.reply_tokens,
            config.retain_rates,
            config.fold_scores,
            config.reflect_floor,
            config.seed,
            config.fbs_arm,
            if flock_end {
                DecodeRule::FLOCK_END
            } else {
                DecodeRule::FLOCK
            },
            &cells,
            &scopes,
            cycle_on,
            keys_on,
            &keyed,
            served_on,
            &traces,
            forces,
            &device,
            served_forward,
            batched_decode_or_default(batched_decode),
            config.context,
            last_opening.unwrap_or(false),
            &panel_queries,
        )? {
            println!("{}", serde_json::to_string(&file)?);
        }
        eprintln!(
            "{}",
            serde_json::json!({
                "decode_cache": decode_cache,
                "decode_forwards": take_decode_forwards(),
            })
        );
        return Ok(());
    }
    if let Some(checkpoint) = decode {
        let nearest = config.objective.decodes_nearest();
        for speed in decode_speed(
            &data_root,
            &checkpoint,
            config.retain_rates,
            config.fold_scores,
            config.reflect_floor,
            nearest,
            config.reply_tokens,
            config.seed,
            config.fbs_arm,
            &device,
            served_forward,
        )? {
            println!("{}", serde_json::to_string(&speed)?);
        }
        return Ok(());
    }
    if grad_norms {
        for norm in gradient_norms(&data_root, &config, &device)? {
            println!("{}", serde_json::to_string(&norm)?);
        }
        return Ok(());
    }
    if let Some(checkpoint) = reach {
        let (reach, bins) = context_reach(
            &data_root,
            &checkpoint,
            config.retain_rates,
            config.fold_scores,
            config.reflect_floor,
            config.seed,
            &device,
        )?;
        for point in reach {
            println!("{}", serde_json::to_string(&point)?);
        }
        for layer in bins {
            println!("{}", serde_json::to_string(&layer)?);
        }
        return Ok(());
    }
    if let Some(run) = inspect {
        let (nearest, farthest) = root_separation()?;
        println!(
            "{}",
            serde_json::json!({"address_root_separation": {"nearest": nearest, "farthest": farthest}})
        );
        for reading in inspect_run(
            &data_root,
            &run,
            config.retain_rates,
            config.fold_scores,
            config.reflect_floor,
            config.seed,
            config.fbs_arm,
            &device,
            served_forward,
        )? {
            println!("{}", serde_json::to_string(&reading)?);
        }
        return Ok(());
    }
    let out = out.ok_or_else(|| FloatTrainError::Invalid("--out is required".to_owned()))?;
    run_long(&data_root, &out, &config, &device)
}

/// `--copy-key-oracle` (claude-267): exact-match copy-key purity on an unpinned UORT pair, with
/// slot ranks from the pinned TRAIN split and boundaries from the pinned tokenizer.
fn copy_key_oracle(data_root: &Path, tokens_path: &Path, mask_path: &Path) -> Result<()> {
    use sha2::{Digest, Sha256};
    use uor_r4_float_train::flock_read_hamming::{
        slot_atom_pairs, slot_atoms, slot_boundary_flags, slot_rank_table, window_atoms,
    };
    use uor_r4_float_train::hamilton_sheet as hs;
    let tokenizer = std::fs::read(data_root.join("tokenizer.json"))?;
    if hex::encode(Sha256::digest(&tokenizer)) != hs::PINNED_TOKENIZER_SHA256 {
        return Err(FloatTrainError::Invalid(
            "tokenizer.json differs from the pinned tokenizer".to_owned(),
        ));
    }
    let mut boundary = slot_boundary_flags(&tokenizer)?;
    if boundary.len() > hs::SHEET_VOCAB {
        return Err(FloatTrainError::Invalid(format!(
            "tokenizer.json names {} ids, past the sheet's vocabulary of {}",
            boundary.len(),
            hs::SHEET_VOCAB
        )));
    }
    boundary.resize(hs::SHEET_VOCAB, true);
    let (train, _) = hs::load_pinned_uort_split(
        &data_root.join("mw-balp/train/tokens.u16"),
        &data_root.join("mw-balp/train/response_mask.u8"),
        hs::PINNED_TRAIN_TOKENS_SHA256,
        hs::PINNED_TRAIN_MASK_SHA256,
        hs::EXPECTED_TRAIN_TOKENS,
        hs::EXPECTED_TRAIN_ELIGIBLE_TARGETS,
        hs::SHEET_VOCAB,
        hs::SHEET_BOS_ID,
    )?;
    let train: Vec<u16> = train.iter().map(|&t| t as u16).collect();
    let table = slot_rank_table(&train, hs::SHEET_VOCAB, &boundary)?;
    let (tokens, mask) = read_uort_pair(tokens_path, mask_path)?;
    let (windows, _) =
        hs::build_document_windows(&tokens, &mask, hs::SHEET_CONTEXT, hs::SHEET_BOS_ID)?;
    let boundary_tokens = boundary.iter().filter(|&&b| b).count();
    println!(
        "copy-key-oracle windows={} boundary_tokens={boundary_tokens} vocab={}",
        windows.len(),
        hs::SHEET_VOCAB
    );
    // Per rule: copyable, with an exact candidate, right.
    let mut counts = [[0usize; 3]; 6];
    for w in &windows {
        let (Some(&p_last), Some(&o0), Some(&p0)) = (
            w.scored_corpus_positions.last(),
            w.scored_offsets.first(),
            w.scored_corpus_positions.first(),
        ) else {
            continue;
        };
        let x = &tokens[p0 - 1 - o0..=p_last];
        let a = slot_atoms(x, &table);
        let a2 = slot_atom_pairs(x, &table);
        let w4 = window_atoms(x, &table, 4);
        let k2 = |i: usize| (x[i], if i == 0 { x[i] } else { x[i - 1] });
        for &t in &w.scored_offsets {
            let next = x[t + 1];
            if !x[..=t].contains(&next) {
                continue;
            }
            let found = [
                (1..t).rev().find(|&s| x[s - 1] == x[t]),
                (1..t).rev().find(|&s| k2(s - 1) == k2(t)),
                (1..t).rev().find(|&s| (x[s - 1], a[s - 1]) == (x[t], a[t])),
                (1..t).rev().find(|&s| (x[s - 1], a2[s - 1]) == (x[t], a2[t])),
                (1..t).rev().find(|&s| (x[s - 1], w4[s - 1]) == (x[t], w4[t])),
            ];
            let backoff = found[1].or(found[0]);
            for (count, found) in counts.iter_mut().zip(found.into_iter().chain([backoff])) {
                count[0] += 1;
                if let Some(s) = found {
                    count[1] += 1;
                    if x[s] == next {
                        count[2] += 1;
                    }
                }
            }
        }
    }
    let rules = ["k1", "k2", "slot", "atom2", "win4", "k2>k1"];
    for (rule, [copyable, with_exact, right]) in rules.into_iter().zip(counts) {
        let purity = if with_exact == 0 { 0.0 } else { right as f64 / with_exact as f64 };
        println!(
            "copy-key-oracle {rule} copyable={copyable} with_exact={with_exact} right={right} purity={purity:.4}"
        );
    }
    Ok(())
}

/// An unpinned UORT pair for `--copy-key-oracle`, refused unless the header is UORT, the payload
/// and mask lengths match its count, the first token is BOS and every token is in the vocabulary.
fn read_uort_pair(tokens_path: &Path, mask_path: &Path) -> Result<(Vec<u32>, Vec<u8>)> {
    use uor_r4_float_train::hamilton_sheet as hs;
    let bytes = std::fs::read(tokens_path)?;
    let mask = std::fs::read(mask_path)?;
    let bad = |what: &str| {
        FloatTrainError::Invalid(format!("--copy-key-oracle {}: {what}", tokens_path.display()))
    };
    if bytes.len() < hs::UORT_HEADER_BYTES || &bytes[..4] != b"UORT" {
        return Err(bad("no 64-byte UORT header"));
    }
    let count = u64::from_le_bytes(bytes[8..16].try_into().expect("8 header bytes")) as usize;
    let payload = &bytes[hs::UORT_HEADER_BYTES..];
    if payload.len() != count * 2 || mask.len() != count {
        return Err(bad("the payload or mask does not match the header count"));
    }
    let tokens: Vec<u32> = payload
        .chunks_exact(2)
        .map(|w| u32::from(u16::from_le_bytes([w[0], w[1]])))
        .collect();
    if tokens.first() != Some(&hs::SHEET_BOS_ID)
        || tokens.iter().any(|&t| t as usize >= hs::SHEET_VOCAB)
    {
        return Err(bad("tokens out of range or no opening BOS"));
    }
    Ok((tokens, mask))
}

/// `--panel NAME=DIR` (claude-204): an extra UORT split for the census, counted unpinned.
fn panel_arg(value: &str) -> Result<(String, PathBuf)> {
    match value.split_once('=') {
        Some((name, dir)) if !name.is_empty() && !dir.is_empty() => {
            Ok((name.to_owned(), PathBuf::from(dir)))
        }
        _ => Err(FloatTrainError::Invalid(
            "--panel takes NAME=DIR".to_owned(),
        )),
    }
}

/// `--decision-trace SPLIT=POSITIONS_JSON` (claude-251): the split and the positions, a JSON array
/// of the windows' first scored positions, read from the file.
fn decision_trace_arg(value: &str) -> Result<(String, Vec<usize>)> {
    let Some((split, path)) = value.split_once('=').filter(|(s, p)| !s.is_empty() && !p.is_empty())
    else {
        return Err(FloatTrainError::Invalid(
            "--decision-trace takes SPLIT=POSITIONS_JSON".to_owned(),
        ));
    };
    let positions: Vec<usize> = serde_json::from_slice(&std::fs::read(path)?).map_err(|e| {
        FloatTrainError::Invalid(format!("--decision-trace {split}: {path} is not a JSON array of positions: {e}"))
    })?;
    if positions.is_empty() {
        return Err(FloatTrainError::Invalid(format!(
            "--decision-trace {split}: {path} names no position"
        )));
    }
    Ok((split.to_owned(), positions))
}

/// `--decision-force SPLIT=FORCE_JSON` (claude-253): the split and the forced sets, a
/// `uor-r4.decision-force/1` file, read from the file; its schema is checked by `score_rows`.
fn decision_force_arg(value: &str) -> Result<(String, DecisionForce)> {
    let Some((split, path)) = value
        .split_once('=')
        .filter(|(s, p)| !s.is_empty() && !p.is_empty())
    else {
        return Err(FloatTrainError::Invalid(
            "--decision-force takes SPLIT=FORCE_JSON".to_owned(),
        ));
    };
    let force: DecisionForce = serde_json::from_slice(&std::fs::read(path)?).map_err(|e| {
        FloatTrainError::Invalid(format!(
            "--decision-force {split}: {path} is not a forced set: {e}"
        ))
    })?;
    Ok((split.to_owned(), force))
}

/// `--key-levels sign=L,hsign=L,point=L` (claude-218): each trunk key's level, 1–10 (q10 to q100).
fn key_levels_arg(value: &str) -> Result<Vec<(String, u8)>> {
    value
        .split(',')
        .map(|part| match part.split_once('=') {
            Some((key, level)) if ["sign", "hsign", "point"].contains(&key) => match level.parse::<u8>() {
                Ok(l) if (1..=10).contains(&l) => Ok((key.to_owned(), l)),
                _ => Err(FloatTrainError::Invalid(format!("--key-levels: {part} wants a level 1–10"))),
            },
            _ => Err(FloatTrainError::Invalid("--key-levels takes sign=L,hsign=L,point=L".to_owned())),
        })
        .collect()
}

/// `--keyed-decode sign=T,hsign=T,point=T` (claude-219): the keyed arms, each trunk key with its
/// τ, a distance of 0 or more, or `inf` to fire at every opening the exact key misses.
fn keyed_decode_arg(value: &str) -> Result<Vec<(String, f32)>> {
    let keyed: Vec<(String, f32)> = value
        .split(',')
        .map(|part| match part.split_once('=') {
            Some((key, tau)) if ["sign", "hsign", "point"].contains(&key) => match tau.parse::<f32>() {
                Ok(t) if t >= 0.0 => Ok((key.to_owned(), t)),
                _ => Err(FloatTrainError::Invalid(format!("--keyed-decode: {part} wants a τ of 0 or more, or inf"))),
            },
            _ => Err(FloatTrainError::Invalid("--keyed-decode takes sign=T,hsign=T,point=T".to_owned())),
        })
        .collect::<Result<_>>()?;
    if keyed.iter().enumerate().any(|(i, (k, _))| keyed[..i].iter().any(|(j, _)| j == k)) {
        return Err(FloatTrainError::Invalid("--keyed-decode names each trunk key once".to_owned()));
    }
    Ok(keyed)
}

fn served_forward_applies(
    served_forward: Option<bool>,
    inspect: bool,
    decode: bool,
    score: bool,
) -> Result<()> {
    if served_forward.is_some() && !(inspect || decode || score) {
        return Err(FloatTrainError::Invalid(
            "--served-forward applies to --inspect, --decode-speed and --score-rows only"
                .to_owned(),
        ));
    }
    Ok(())
}

/// `--context` sets the window training cuts TRAIN at and builds its model to read (claude-273),
/// `--grad-norms 1` reads TRAIN the same way, and `--score-rows` cuts its panels at it
/// (claude-274). Every other mode, `--recall` among them, reads 192-token windows, so another
/// window there would be ignored: it is refused.
fn context_applies(context: usize, training: bool, grad_norms: bool, scoring: bool) -> Result<()> {
    if context != uor_r4_float_train::hamilton_sheet::SHEET_CONTEXT
        && !(training || grad_norms || scoring)
    {
        return Err(FloatTrainError::Invalid(
            "--context applies to training with --out, --grad-norms 1 and --score-rows only"
                .to_owned(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyed_decode_names_each_trunk_key_once_with_a_tau_of_0_or_more() {
        assert_eq!(
            keyed_decode_arg("sign=0.0078125,hsign=0,point=inf").unwrap(),
            [("sign".to_owned(), 0.0078125), ("hsign".to_owned(), 0.0), ("point".to_owned(), f32::INFINITY)]
        );
        assert!(keyed_decode_arg("sign=-0.5").is_err());
        assert!(keyed_decode_arg("sign=NaN").is_err());
        assert!(keyed_decode_arg("sign=x").is_err());
        assert!(keyed_decode_arg("exact=0").is_err());
        assert!(keyed_decode_arg("sign=0,sign=1").is_err());
    }

    #[test]
    fn key_levels_name_each_trunk_key_once_with_a_level_from_1_to_10() {
        assert_eq!(
            key_levels_arg("sign=3,point=10").unwrap(),
            [("sign".to_owned(), 3), ("point".to_owned(), 10)]
        );
        assert_eq!(key_levels_arg("hsign=5").unwrap(), [("hsign".to_owned(), 5)]);
        assert!(key_levels_arg("sign=0").is_err());
        assert!(key_levels_arg("sign=11").is_err());
        assert!(key_levels_arg("exact=3").is_err());
    }

    #[test]
    fn a_panel_argument_is_a_name_and_a_directory() {
        let (name, dir) = panel_arg("p-dev=/data/p-dev").unwrap();
        assert_eq!((name.as_str(), dir), ("p-dev", PathBuf::from("/data/p-dev")));
        assert!(panel_arg("p-dev").is_err());
        assert!(panel_arg("=dir").is_err());
        assert!(panel_arg("name=").is_err());
    }

    #[test]
    fn score_rows_decodes_batched_unless_told_otherwise() {
        // claude-270: the batched path gave byte-identical files to the serial one on the GPU;
        // `--batched-decode 0` keeps the serial reference.
        assert!(batched_decode_or_default(None));
        assert!(!batched_decode_or_default(Some(false)));
        assert!(batched_decode_or_default(Some(true)));
    }

    #[test]
    fn served_forward_is_a_scoring_only_override() {
        assert!(served_forward_applies(None, false, false, false).is_ok());
        assert!(served_forward_applies(Some(true), true, false, false).is_ok());
        assert!(served_forward_applies(Some(false), false, true, false).is_ok());
        assert!(served_forward_applies(Some(true), false, false, true).is_ok());
        assert!(served_forward_applies(Some(true), false, false, false).is_err());
    }

    #[test]
    fn a_window_other_than_192_trains_or_scores() {
        let sheet = uor_r4_float_train::hamilton_sheet::SHEET_CONTEXT;
        assert!(context_applies(sheet, false, false, false).is_ok());
        assert!(context_applies(2048, true, false, false).is_ok());
        assert!(context_applies(2048, false, true, false).is_ok());
        assert!(context_applies(2048, false, false, true).is_ok(), "claude-274: scoring at a window");
        assert!(context_applies(2048, false, false, false).is_err());
        assert!(context_applies(sheet + 1, false, false, false).is_err());
    }
}
