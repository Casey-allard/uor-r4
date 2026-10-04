//! Train, evaluate and sample the geometric stack and its transformer control
//! (`uor_r4_training::geometric_stack`) on u16 token files, and prepare those
//! files with the lab's byte-level BPE.
//!
//! Modes:
//!
//! ```text
//! geometric-stack train train=TRAIN.u16[,MORE.u16] [train_weights=W1,W2] valid=VALID.u16 \
//!   out=NEW_REPORT_ROOT (init=ROOT/model | arch=transformer|geometric) [pattern=rrarra] \
//!   [read=lorentz|dot|l2] [rotation=true|false] [rotation_group=quaternion|u1] [qat=false|true] \
//!   [transport_snap=none|icosian] \
//!   [seed=1] [steps=7324] [batch=16] [lr=0.002] [warmup=200] [min_lr=0.1] [weight_decay=0.1] \
//!   [clip=1.0] [eval_every=250] [eval_windows=64] [final_windows=512] [lens=LENS.u16] \
//!   [merges=MERGES.txt] [checkpoint_every=250] [resume=OLD_ROOT/checkpoint] [max_seconds=inf] \
//!   [sample_tokens=128] [tokenizer=TOKENIZER.json] [width=288] [heads=6] [layers=6] [mlp=768] \
//!   [stack_mlp=MATCHED] [context=256] [memory_layers=L1,L2 [memory_sub_keys=256] [memory_top_k=32] \
//!   [memory_heads=4] [memory_key_dim=128] [memory_score=dot|lorentz] [memory_codebook=h4|e8]]
//! geometric-stack sample model=ROOT/model valid=VALID.u16 merges=MERGES.txt|tokenizer=TOKENIZER.json \
//!   out=NEW_REPORT_ROOT [prompts=3] [prompt_tokens=64] [sample_tokens=128] [temperature=0.8] [top_k=40] \
//!   [seed=1]
//! geometric-stack evaluate model=ROOT/model tokens=DEV.u16 out=NEW_REPORT_ROOT [tune_blocks=64] \
//!   [lens=LENS.u16]
//! geometric-stack encode merges=MERGES.txt input=TEXT out=TOKENS.u16
//! geometric-stack corpus registry=CARGO_REGISTRY_SRC_INDEX out=TEXT [max_file_bytes=200000]
//! geometric-stack export model=ROOT/model out=NEW_REPORT_ROOT [calibration=TRAIN.u16] \
//!   [calibration_windows=64] [calibration_time=CONTEXT] [damp=0.01]
//! geometric-stack lut-evaluate artifact=ROOT/model.lut valid=VALID.u16 out=NEW_REPORT_ROOT \
//!   [model=ROOT/model] [windows=64] [blocks=false] [tune_blocks=64] [threads=1] [lens=LENS.u16] \
//!   [reference=false]
//! geometric-stack d11-evaluate artifact=ROOT/model.lut valid=VALID.u16 out=NEW_REPORT_ROOT \
//!   [windows=8] [threads=1] [lens=LENS.u16]
//! geometric-stack snap-evaluate model=ROOT/model valid=VALID.u16 out=NEW_REPORT_ROOT [windows=512]
//!   (roadmap S1.0b: development NLL with every transport quaternion snapped to the nearest of the
//!   120 unit icosians, against the unsnapped model on the same windows; evaluation only)
//! geometric-stack rounding-attribution artifact=ROOT/model.lut model=ROOT/model valid=VALID.u16 \
//!   out=NEW_REPORT_ROOT [windows=512]
//!   (roadmap S1.0c: the float model with one tensor group at a time from the artifact; evaluation only)
//! geometric-stack lut-sample artifact=ROOT/model.lut out=NEW_REPORT_ROOT (valid=VALID.u16 | prompt=TEXT) \
//!   [merges=MERGES.txt | tokenizer=TOKENIZER.json] [prompts=3] [prompt_tokens=64] [sample_tokens=128] \
//!   [temperature=0.8] [top_k=40] [top_p=1] [seed=1] [threads=1]
//! geometric-stack dialogue-train out=NEW_REPORT_ROOT tokenizer=TOKENIZER.json \
//!   train_tokens=TRAIN.uort train_mask=TRAIN.mask train_manifest=TRAIN/manifest.json \
//!   dev_tokens=DEV.uort dev_mask=DEV.mask dev_manifest=DEV/manifest.json \
//!   (init=ROOT/model | arch=geometric|transformer [shape options as train]) [qat=false|true] \
//!   [transport_snap=none|icosian] [select=none|flock:WINDOW:K] [pointer=none|DIM] \
//!   [pointer_score=dot|lorentz] [pointer_select=none|flock:WINDOW:K|top:K] \
//!   [pointer_route=none|prime:WINDOW|prime-ranked:WINDOW|ngram:WINDOW|ngram-ranked:WINDOW] \
//!   [context=256] \
//!   [policy=full_prefix|role_only|truncated_prefix[:KEEP]] [data_seed=1] [steps=1024] [batch=16] [lr=0.001] [warmup=50] \
//!   [min_lr=0.1] [weight_decay=0.1] [clip=1.0] [eval_every=128] [dev_seed=1] [dev_per_source=32] \
//!   [checkpoint_every=128] [resume=OLD_ROOT/checkpoint] [max_seconds=inf] [requests=REQUESTS.json] \
//!   [max_new_tokens=96] [protocol=1|2]
//! geometric-stack lut-chat artifact=ROOT/model.lut tokenizer=TOKENIZER.json out=NEW_REPORT_ROOT \
//!   [requests=REQUESTS.json] [max_new_tokens=96] [temperature=0] [top_k=40] [top_p=1] [seed=1] \
//!   [threads=1] [protocol=1|2] [engine=d11|d10]
//! ```
//!
//! The shape options describe the transformer control (#1017's by default). A
//! geometric stack takes the same width, heads, depth and context, the
//! pattern `rra` repeated by default, and the MLP width that matches the
//! control's parameter count.
//!
//! `train` samples windows uniformly from TRAIN with a seeded generator, so two
//! arms with one seed see identical windows. Evaluation windows are evenly
//! spaced over VALID, the rule of `joint-read-geometry`, so `final_windows=512`
//! at context 256 scores the same 131,072 targets as that example. A stopped run
//! continues with `resume=OLD_ROOT/checkpoint` in a new root, with identical
//! settings and input contents (sizes and SHA-256 of every input file). Every
//! report root is claimed before loading and sealed at the end. Set
//! RAYON_NUM_THREADS to bound the threads.
//!
//! With `init=ROOT/model`, `train` starts from that model's weights with a
//! fresh optimizer and schedule and takes its configuration from
//! `ROOT/model/config.json`; architecture options given beside it must agree
//! with the saved model, and `seed=` (default: the saved one) seeds the window
//! sampler. It scores the model at step 0 before any update. `qat=true` trains
//! with the served representation in the forward pass (quantization-aware
//! training, `StackModel::set_served_representation` with the D11 interim
//! codec): the values `export` writes when rounding to nearest, with
//! straight-through gradients. A QAT run's evaluations score the served
//! representation (`dev`, `final`) and the float weights (`dev_float`,
//! `final_float`) on the same windows; its saved model is the float weights,
//! whose round-to-nearest export is the representation it trained, and its
//! `config.json` records that representation (`served_representation`).
//!
//! Matched ablation controls (float training only; `qat=true`, the snap and
//! every export refuse them): `read=l2` scores each source `-beta (|q - k| -
//! offset)`, the flat Euclidean distance in place of the Lorentz read's
//! hyperbolic one, with the same per-head `beta` and `offset`;
//! `rotation_group=u1` (with `rotation=true`) zeroes each raw transport
//! quaternion's `j` and `k` before normalization, so every lane transition
//! lies in the commutative subgroup `{a + b i}`. Both keep the parameter
//! count of the configuration they control (`read=lorentz`,
//! `rotation_group=quaternion`, the default).
//!
//! `transport_snap=icosian` (in `train` and `dialogue-train`, after `init=`
//! and on a resume alike) trains with every recurrence's unit transport
//! quaternion snapped to the nearest of the 120 unit icosians of 2I before
//! its scaling by lambda, with straight-through gradients
//! (`StackModel::set_transport_snap`), so the transport is an exact element
//! of 2I. It needs a geometric stack with `rotation=true`, and composes with
//! `qat=true`. The resume lineage records it. Each evaluation scores the
//! model's own forward, snapped (`dev`, `final`; in `dialogue-train`
//! `dev_response_nll`), and the same weights with the transport free
//! (`dev_unsnapped`, `final_unsnapped`; `dev_unsnapped_*`) on the same
//! windows or panel, and reports the icosian usage over the evaluated
//! positions (`transport_usage`: roots selected, entropy of the selection
//! distribution, the identity's share). The saved model is the float
//! weights with `transport.json` beside them recording the snap; loading it
//! does not apply the snap, and `export` refuses it, as the integer engines
//! serve the unsnapped transport.
//!
//! `evaluate` scores consecutive 256-input blocks with a fresh state per
//! block (257 stored ids, 256 targets), the retained evaluator's protocol
//! (`docs/integration/reference-evaluator-v2.json`): it reports the first
//! `tune_blocks` blocks, the remaining comparison blocks and all blocks
//! separately. On the retained TinyStories development store that is 976
//! blocks, 64 tune and 912 comparison (233,472 targets), where #1017 scores
//! 1.574024 nats. Samples decode with `merges=` (the lab BPE) or
//! `tokenizer=` (a `tokenizer.json`, such as the retained 4,096-token one).
//!
//! `export` writes the integer serving artifact of a trained model under owner
//! decision D10 (`uor_r4_training::stack_export`): a stack artifact for a
//! geometric stack, served by `uor_r4_lut::stack`, or a Llama artifact for the
//! transformer control, served by `uor_r4_lut::engine`. Weights round to
//! nearest, or with `calibration=` by GPTQ against the input moments of every
//! weight map over `calibration_windows` evenly spaced windows of that token
//! file, which should be training data. A model whose `config.json` records a
//! served representation (a `qat=true` run's) exports only as that
//! representation using its restored codec, with the record in `export.json`
//! and the artifact's source; `calibration=` is refused. `lut-evaluate` runs
//! either engine position by position over the evenly spaced windows of
//! `train`'s evaluation (`windows=512` is the final evaluation's 131,072
//! targets), with a fresh session per window, and reports its NLL beside the
//! float model's on the same windows when `model=` is given, their top-1
//! agreement and the engine's tokens per second; `blocks=true` uses
//! `evaluate`'s consecutive blocks and tune/comparison split instead.
//! `d11-evaluate` runs a stack artifact through the multiplier-free D11
//! engine (`uor_r4_integer::stack`) and the frozen D10 engine side by side
//! on the same evenly spaced windows, and reports both NLLs, the largest
//! absolute difference between their integer logits (zero for a bit-identical
//! port), each engine's tokens per second and the dense per-token weight
//! reads.
//! `lut-sample` writes greedy and sampled continuations from the integer
//! engine (the integer sampler of `uor_r4_lut::sampling`). Token files may
//! also be UORT stores, the prepared dialogue corpus's format.
//!
//! `dialogue-train` teaches a stack the responses of the prepared
//! literal-role dialogue corpus (`uor_r4_training::stack_dialogue`) with the
//! retained dialogue study's episodes: its contract, eligibility,
//! response-uniform sampler and source-stratified development panel
//! (`dev_seed`, `dev_per_source`). The loss covers response and EOS targets
//! only; each batch drops the padding columns after its longest episode. It
//! starts from a trained stack (`init=`) or a new one, scores the panel every
//! `eval_every` updates, and with `requests=` (the study's request format)
//! writes the float model's greedy replies. `qat=true` trains with the served
//! representation as `train` does (after `init=` and on a resume alike): each
//! evaluation scores the served representation (`dev_response_nll`) and the
//! float weights (`dev_float_response_nll`) on the same panel, the replies
//! come from the served representation, and the saved model's `config.json`
//! records the representation. `lut-chat` talks through either
//! integer engine under the same protocol: replies to `requests=`, or an
//! interactive conversation on standard input (`/reset` starts over; a full
//! 256-position context starts a new conversation). Replies stop as the
//! study's do: at EOS, at a terminal cycle of one to four ids repeated three
//! times, or at `max_new_tokens`. A plain artifact uses protocol 1 unless
//! `protocol=2` is given. A pointer artifact (`uor-r4.lut-stack/2`) is served
//! through its mixture: greedily (ties to the lower id), by the
//! multiplier-free D11 session (`engine=d11`, the default) or the D10
//! comparator (`engine=d10`), under protocol 2 by default, with prompts and
//! stops exactly as the float model's `reply_panel` (`chat-grade grade|reply`)
//! has them, so the integer and float replies compare id for id. Sampling
//! options (`temperature`, `top_k`, `top_p`, `seed`) are refused for a pointer
//! artifact, and `engine=` for a plain one. The D11 engine runs its weight
//! maps on `threads=` threads (default 1); the integers do not depend on it. Its
//! `chat.json` (`uor-r4.geometric-stack-lut-chat/2`) records the engine's
//! thread count, each reply's generated ids, seconds and ids per second, and
//! the panel totals.
//!
//! `policy=` (`dialogue-train`) sets the training episodes' prefix; the
//! development panel always keeps its full prefix. `full_prefix` (the default)
//! keeps the whole document before the response, and `role_only` only BOS and
//! the assistant marker, on the same responses. `truncated_prefix:KEEP` also
//! admits each response whose document is too long but which fits whole after
//! BOS and the marker, and keeps BOS and at most the last KEEP prefix IDs that
//! fit beside it (`truncated_prefix` alone: as many as fit). No response is
//! cut. Its eligible population is recorded as `train_population`.
//!
//! `context=N` (`dialogue-train`) sets the positions the model reads (default
//! 256, at least 256). For a new model it is a shape option; after `init=` it
//! may only grow the saved context (`StackModel::extend_context`: every
//! learned age is kept, and each read head continues along its initial slope,
//! so the model scores sequences up to the saved context exactly as before).
//! Training episodes fill the model's context; the development panel stays
//! the retained study's 256-ID panel.
//!
//! `select=flock:WINDOW:K`, `pointer=DIM`, `pointer_score=dot|lorentz` and
//! `pointer_select=none|flock:WINDOW:K|top:K` (`dialogue-train`, for fresh
//! shapes and after `init=` alike) are the A1 retrieval mechanisms of
//! `uor_r4_training::geometric_stack`, selecting with the shared
//! `uor_r4_training::flock` selector. A flock (`select=`, its sink at position
//! 0) makes every read row (the geometric reads and the control's attention)
//! softmax over the sink, the last WINDOW positions and the K best-scoring
//! other sources only. A pointer adds a copy head after the final norm: a
//! DIM-wide query and key attend the input tokens by the pointer's own score
//! (`pointer_score=`: `dot`, the default, or `lorentz`, the fused read's
//! hyperboloid form with a learned scale) and its own selection
//! (`pointer_select=`, default none, which keeps every source; `top:K` keeps
//! the K best alone, so `top:1` is the single-source pointer). `dialogue-train`
//! refuses `top:1`, given or carried by an `init=` head: its one kept source
//! gives the query, key and scale no gradient, so the single-source pointer is
//! soft-trained weights with `top:1` applied by `m-world evaluate`. The reads'
//! `select=` never applies to the pointer. A gate mixes the copied token's
//! distribution with the ordinary one; the loss, the development scores and
//! the greedy replies are the mixture's. All go into the saved `config.json`
//! and the report. After `init=`, `select=` replaces the saved flock and
//! `pointer_select=` the saved pointer's selection (the weights are unchanged
//! by both), `pointer=DIM` and `pointer_score=` must agree with a saved head,
//! and `pointer=DIM` on a model saved without a pointer adds one with weights
//! drawn fresh from `seed=` (default: the saved model's seed). That seed is
//! recorded in the head's `init_seed` and in the report (`flock_and_pointer`);
//! a resume verifies it (a different one is refused) and carries it forward.
//! `pointer_route=prime:WINDOW` (`dialogue-train`) replaces the pointer's
//! learned scores by the exact prime route of ADR-0003
//! (`uor_r4_training::geometric_stack::PrimeRoute`): a source is admitted when
//! the registered primes of the WINDOW tokens before it share a factor with
//! the query's last WINDOW, and the pointer copies the token that followed;
//! the gate still learns, the query and key get no gradient. `prime-ranked:WINDOW`
//! keeps that admission and ranks the admitted sources by the learned score
//! plus the route's, falling back to the learned pointer where nothing is
//! admitted (the query and key then train). `ngram:WINDOW` and
//! `ngram-ranked:WINDOW` admit by the longest ordered n-let match instead
//! (the n tokens before a source equal the query's last n, for the longest
//! n up to WINDOW with a match; ADR-0003's transition indexes). Each
//! `pointer_select=`, and `none` clears a saved route.
//! Each evaluation reports the pointer's mean gate and hit rate on the scored
//! targets (`dev_pointer_*` in the curve, `pointer` in the developments). The
//! pointer has no served representation for training (`qat=true` refuses
//! it). `export` writes a pointer that keeps every source, which both integer
//! engines serve as its mixture (`uor-r4-stack generate` decodes greedily over
//! it, `d11-evaluate` compares the engines' logits and mixtures and scores the
//! mixture); it refuses a pointer selection or route, and a flock, until a
//! D11 port exists. The evaluators that read raw logits (`snap-evaluate`,
//! `rounding-attribution`, and `lut-evaluate` and the artifact samplers, also
//! on a pointer artifact) refuse a pointer model, whose distribution is the
//! mixture. `m-world evaluate` overrides a saved model's selections without
//! training (`select=`, `pointer_select=`). `--help` prints the usage.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use candle_core::Device;
use serde_json::{json, Value};
use uor_r4_core::native_geometric::learner::embedding::canonical_h4_roots;
use uor_r4_core::report_output;
use uor_r4_tokenizer::dialogue::DialogueProtocol;
use uor_r4_training::dialogue_development;
use uor_r4_training::dialogue_episodes::{EpisodeIndex, PrefixPolicy, EPISODE_CONTEXT};
use uor_r4_training::flock::FlockSelect;
use uor_r4_training::geometric_stack::{
    average_replica_gradients, logits_cross_entropy, parse_flock_select, parse_pointer_route,
    parse_pointer_select, D11Interim, MapCodec, PointerConfig, PointerSelect, PrimeRoute,
    ReadScore, RotationGroup, ServedStatistics, StackAdamW, StackArch, StackConfig, StackModel,
    TransportSnap, TransportUsage,
};
use uor_r4_training::lut_export::export_llama;
use uor_r4_training::stack_dialogue::{
    annotate_turn_costs, check_panel, development, episode_contract_for, greedy_reply,
    load_requests, reply_panel, trim, DialogueSplit, Reply, TurnCost, MAX_NEW_TOKENS,
};
use uor_r4_training::stack_export::{
    check_export_config, check_export_representation, check_raw_logit_evaluation,
    control_checkpoint, control_grid_reference, export_quantizer_method, export_stack,
    stack_grid_reference, StackCalibration,
};
use uor_r4_training::stack_memory::{Codebook, MemoryConfig, MemoryScore};
use uor_r4_training::{codec_by_name, sha256_file, Result, TrainingError};

fn invalid(message: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(message.into())
}

fn lut(error: uor_r4_lut::LutError) -> TrainingError {
    invalid(error.to_string())
}

struct Args(BTreeMap<String, String>);

impl Args {
    fn parse(arguments: &[String], allowed: &[&str]) -> Result<Self> {
        let mut pairs = BTreeMap::new();
        for argument in arguments {
            let (key, value) = argument
                .split_once('=')
                .ok_or_else(|| invalid(format!("arguments are key=value, got {argument}")))?;
            if !allowed.contains(&key) {
                return Err(invalid(format!("unknown argument {key}=")));
            }
            pairs.insert(key.to_owned(), value.to_owned());
        }
        Ok(Self(pairs))
    }

    fn required(&self, key: &str) -> Result<String> {
        self.0
            .get(key)
            .cloned()
            .ok_or_else(|| invalid(format!("missing {key}=")))
    }

    fn optional(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }

    fn number<T: std::str::FromStr>(&self, key: &str, default: T) -> Result<T> {
        match self.0.get(key) {
            None => Ok(default),
            Some(value) => value
                .parse()
                .map_err(|_| invalid(format!("invalid {key}={value}"))),
        }
    }
}

/// SplitMix64: window sampling and categorical draws.
#[derive(Clone, Copy)]
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }

    fn uniform(&mut self) -> f64 {
        ((self.next() >> 11) as f64 + 0.5) / (1u64 << 53) as f64
    }
}

/// A u16 token file: raw little-endian ids, or a UORT store (the prepared
/// dialogue corpus's format, `uor_r4_core::native_geometric::mmap_corpus`).
fn read_tokens(path: &Path, vocabulary: usize) -> Result<Vec<u32>> {
    let bytes = fs::read(path)?;
    if bytes.starts_with(b"UORT") {
        let reader = uor_r4_core::native_geometric::mmap_corpus::MmapCorpusReader::open(path)
            .map_err(|e| invalid(format!("{}: {e}", path.display())))?;
        if reader.vocab_size() as usize > vocabulary {
            return Err(invalid(format!(
                "{} declares a larger vocabulary",
                path.display()
            )));
        }
        let tokens: Vec<u32> = reader.as_slice().iter().map(|&id| u32::from(id)).collect();
        if tokens.iter().any(|&id| id as usize >= vocabulary) {
            return Err(invalid(format!(
                "{} has ids outside the vocabulary",
                path.display()
            )));
        }
        return Ok(tokens);
    }
    if bytes.len() % 2 != 0 {
        return Err(invalid(format!(
            "{} is not a u16 token file",
            path.display()
        )));
    }
    let tokens: Vec<u32> = bytes
        .chunks_exact(2)
        .map(|pair| u32::from(u16::from_le_bytes([pair[0], pair[1]])))
        .collect();
    if tokens.iter().any(|&id| id as usize >= vocabulary) {
        return Err(invalid(format!(
            "{} has ids outside the vocabulary",
            path.display()
        )));
    }
    Ok(tokens)
}

fn identity(path: &Path) -> Result<Value> {
    Ok(json!({"path": path, "bytes": fs::metadata(path)?.len(), "sha256": sha256_file(path)?}))
}

// ---------------------------------------------------------------------------
// The lab's byte-level BPE (docs/evidence/native-lorentz-packet-2026-09-26/
// sources/c3-harness/src/bin/bpe.rs): identical pre-tokenization and merges.

const MAX_PUNCT: usize = 8;
const MAX_SPACE: usize = 32;

fn class(byte: u8) -> u8 {
    match byte {
        b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | 0x80..=0xff => 0,
        b' ' | b'\t' | b'\n' | b'\r' => 1,
        _ => 2,
    }
}

fn run_end(text: &[u8], from: usize, c: u8) -> usize {
    let cap = match c {
        0 => usize::MAX,
        1 => MAX_SPACE,
        _ => MAX_PUNCT,
    };
    let mut end = from;
    while end < text.len() && class(text[end]) == c && end - from < cap {
        end += 1;
    }
    end
}

fn chunks(text: &[u8]) -> Vec<&[u8]> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < text.len() {
        let c = class(text[i]);
        if c == 1 {
            if text[i] == b' ' && i + 1 < text.len() && class(text[i + 1]) != 1 {
                let end = run_end(text, i + 1, class(text[i + 1]));
                out.push(&text[i..end]);
                i = end;
                continue;
            }
            let mut end = run_end(text, i, 1);
            if end < text.len() && end - i > 1 && text[end - 1] == b' ' && class(text[end]) != 1 {
                end -= 1;
            }
            out.push(&text[i..end]);
            i = end;
            continue;
        }
        let end = run_end(text, i, c);
        out.push(&text[i..end]);
        i = end;
    }
    out
}

struct Bpe {
    rank: HashMap<(u32, u32), u32>,
    token_bytes: Vec<Vec<u8>>,
}

impl Bpe {
    fn load(path: &Path) -> Result<Self> {
        let text = fs::read_to_string(path)?;
        let mut rank = HashMap::new();
        let mut token_bytes: Vec<Vec<u8>> = (0..=255u8).map(|b| vec![b]).collect();
        for (r, line) in text.lines().enumerate() {
            let (a, b) = line
                .split_once(' ')
                .ok_or_else(|| invalid("merges lines are `a b`"))?;
            let (a, b): (u32, u32) = (
                a.parse().map_err(|_| invalid("merge id"))?,
                b.parse().map_err(|_| invalid("merge id"))?,
            );
            if a as usize >= token_bytes.len() || b as usize >= token_bytes.len() {
                return Err(invalid("merge refers to a later token"));
            }
            let joined = [
                token_bytes[a as usize].clone(),
                token_bytes[b as usize].clone(),
            ]
            .concat();
            token_bytes.push(joined);
            rank.insert((a, b), r as u32);
        }
        Ok(Self { rank, token_bytes })
    }

    fn encode_chunk(&self, chunk: &[u8]) -> Vec<u32> {
        let mut symbols: Vec<u32> = chunk.iter().map(|&b| u32::from(b)).collect();
        loop {
            let mut best: Option<(u32, (u32, u32))> = None;
            for pair in symbols.windows(2) {
                if let Some(&r) = self.rank.get(&(pair[0], pair[1])) {
                    if best.is_none_or(|(b, _)| r < b) {
                        best = Some((r, (pair[0], pair[1])));
                    }
                }
            }
            let Some((r, (a, b))) = best else { break };
            let merged = 256 + r;
            let mut next = Vec::with_capacity(symbols.len());
            let mut i = 0;
            while i < symbols.len() {
                if i + 1 < symbols.len() && symbols[i] == a && symbols[i + 1] == b {
                    next.push(merged);
                    i += 2;
                } else {
                    next.push(symbols[i]);
                    i += 1;
                }
            }
            symbols = next;
        }
        symbols
    }

    fn encode(&self, text: &[u8]) -> Vec<u16> {
        let mut cache: HashMap<&[u8], Vec<u32>> = HashMap::new();
        let mut out = Vec::with_capacity(text.len() / 3);
        for chunk in chunks(text) {
            let ids = cache
                .entry(chunk)
                .or_insert_with(|| self.encode_chunk(chunk));
            out.extend(ids.iter().map(|&id| id as u16));
        }
        out
    }

    fn decode(&self, ids: &[u32]) -> String {
        let bytes: Vec<u8> = ids
            .iter()
            .flat_map(|&id| {
                self.token_bytes
                    .get(id as usize)
                    .cloned()
                    .unwrap_or_default()
            })
            .collect();
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

/// Decodes sample ids for the report.
enum Decoder {
    Lab(Bpe),
    Retained(Box<uor_r4_tokenizer::ByteBpeTokenizer>),
}

impl Decoder {
    fn load(merges: Option<&Path>, tokenizer: Option<&Path>) -> Result<Option<Self>> {
        match (merges, tokenizer) {
            (Some(_), Some(_)) => Err(invalid("give merges= or tokenizer=, not both")),
            (Some(path), None) => Ok(Some(Self::Lab(Bpe::load(path)?))),
            (None, Some(path)) => Ok(Some(Self::Retained(Box::new(
                uor_r4_tokenizer::ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(path)?)
                    .ok_or_else(|| invalid("unreadable tokenizer.json"))?,
            )))),
            (None, None) => Ok(None),
        }
    }

    fn decode(&self, ids: &[u32]) -> String {
        match self {
            Self::Lab(bpe) => bpe.decode(ids),
            Self::Retained(tokenizer) => tokenizer.decode(ids),
        }
    }

    fn encode(&self, text: &str) -> Vec<u32> {
        match self {
            Self::Lab(bpe) => bpe
                .encode(text.as_bytes())
                .into_iter()
                .map(u32::from)
                .collect(),
            Self::Retained(tokenizer) => tokenizer.encode(text),
        }
    }
}

fn encode_mode(arguments: &[String]) -> Result<()> {
    let args = Args::parse(arguments, &["merges", "input", "out"])?;
    let out = PathBuf::from(args.required("out")?);
    if out.exists() {
        return Err(invalid("encode output exists"));
    }
    let bpe = Bpe::load(Path::new(&args.required("merges")?))?;
    let text = fs::read(args.required("input")?)?;
    let ids = bpe.encode(&text);
    let decoded: Vec<u8> = ids
        .iter()
        .flat_map(|&id| bpe.token_bytes[id as usize].iter().copied())
        .collect();
    if decoded != text {
        return Err(invalid("BPE round trip failed"));
    }
    let bytes: Vec<u8> = ids.iter().flat_map(|id| id.to_le_bytes()).collect();
    fs::write(&out, bytes)?;
    println!(
        "{}",
        json!({"input_bytes": text.len(), "tokens": ids.len(), "bytes_per_token": text.len() as f64 / ids.len() as f64, "round_trip": "exact", "sha256": sha256_file(&out)?})
    );
    Ok(())
}

/// Latest version of each crate in a Cargo registry source directory; every
/// `.rs` file up to `max_file_bytes`, in sorted path order, each followed by a
/// newline (the repository corpus's convention).
fn corpus_mode(arguments: &[String]) -> Result<()> {
    let args = Args::parse(arguments, &["registry", "out", "max_file_bytes"])?;
    let registry = PathBuf::from(args.required("registry")?);
    let out = PathBuf::from(args.required("out")?);
    let max_file_bytes: u64 = args.number("max_file_bytes", 200_000)?;
    if out.exists() {
        return Err(invalid("corpus output exists"));
    }
    let mut latest: BTreeMap<String, (Vec<u64>, PathBuf)> = BTreeMap::new();
    for entry in fs::read_dir(&registry)? {
        let path = entry?.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let Some(split) = name
            .char_indices()
            .find(|&(i, c)| c == '-' && name[i + 1..].starts_with(|d: char| d.is_ascii_digit()))
            .map(|(i, _)| i)
        else {
            continue;
        };
        let (crate_name, version) = (&name[..split], &name[split + 1..]);
        let key: Vec<u64> = version
            .split(|c: char| !c.is_ascii_digit())
            .filter(|part| !part.is_empty())
            .map(|part| part.parse().unwrap_or(0))
            .collect();
        let replace = latest
            .get(crate_name)
            .is_none_or(|(existing, _)| key > *existing);
        if replace {
            latest.insert(crate_name.to_owned(), (key, path));
        }
    }
    let mut files = Vec::new();
    for (_, root) in latest.values() {
        let mut stack = vec![root.clone()];
        while let Some(directory) = stack.pop() {
            for entry in fs::read_dir(&directory)? {
                let path = entry?.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().is_some_and(|e| e == "rs")
                    && fs::metadata(&path)?.len() <= max_file_bytes
                {
                    files.push(path);
                }
            }
        }
    }
    files.sort();
    let mut text = Vec::new();
    for path in &files {
        text.extend(fs::read(path)?);
        text.push(b'\n');
    }
    fs::write(&out, &text)?;
    println!(
        "{}",
        json!({"crates": latest.len(), "files": files.len(), "bytes": text.len(), "sha256": sha256_file(&out)?})
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Training.

struct Settings {
    /// One or more training streams; a window's stream is drawn with
    /// probability proportional to `train_weights`, then its start uniformly.
    train: Vec<PathBuf>,
    train_weights: Vec<f64>,
    valid: PathBuf,
    lens: Option<PathBuf>,
    merges: Option<PathBuf>,
    tokenizer: Option<PathBuf>,
    config: StackConfig,
    /// A trained model directory the run starts from (`init=`), with a fresh
    /// optimizer and schedule.
    init: Option<PathBuf>,
    /// Train with the served representation (`qat=true`).
    qat: bool,
    /// Train with the transport snapped (`transport_snap=`).
    transport_snap: Option<TransportSnap>,
    steps: usize,
    batch: usize,
    lr: f64,
    warmup: usize,
    min_lr: f64,
    weight_decay: f64,
    clip: f64,
    eval_every: usize,
    eval_windows: usize,
    final_windows: usize,
    checkpoint_every: usize,
    resume: Option<PathBuf>,
    max_seconds: f64,
    sample_tokens: usize,
    /// `device=cpu|metal|cuda` (default cpu). A Metal or CUDA run needs the
    /// `metal` or `cuda` feature; ops without a GPU kernel run on host copies.
    device: Device,
    /// `data_parallel=1|2` (default 1). 2: CUDA only; each step splits the
    /// batch into equal halves on GPUs 0 and 1, averages the replicas'
    /// gradients on GPU 0, updates there and copies the weights to GPU 1.
    data_parallel: usize,
}

/// `device=cpu|metal|cuda` (default cpu); no implicit fallback. Also applies
/// `tf32=true|false` (default false): CUDA f32 matmuls may then use TF32
/// tensor cores, which round inputs to a 10-bit mantissa. Training only; the
/// run's report records the setting.
fn device_arg(args: &Args) -> Result<Device> {
    let device = match args.optional("device") {
        None => Device::Cpu,
        Some(name) => uor_r4_training::baseline_protocol::device(&name)?,
    };
    let tf32 = match args.optional("tf32").as_deref() {
        None | Some("false") => false,
        Some("true") if device.is_cuda() => true,
        Some("true") => return Err(invalid("tf32=true needs device=cuda")),
        Some(other) => return Err(invalid(format!("invalid tf32={other}"))),
    };
    candle_core::cuda::set_gemm_reduced_precision_f32(tf32);
    Ok(device)
}

/// Whether CUDA f32 matmuls may use TF32 in this process (`tf32=`).
fn tf32_enabled() -> bool {
    candle_core::cuda::gemm_reduced_precision_f32()
}

/// The served representation of `qat=true`.
fn qat_codec() -> Arc<dyn MapCodec> {
    Arc::new(D11Interim)
}

/// `qat=true|false` (default false).
fn qat_flag(args: &Args) -> Result<bool> {
    match args.optional("qat").as_deref() {
        None | Some("false") => Ok(false),
        Some("true") => Ok(true),
        Some(other) => Err(invalid(format!("invalid qat={other}"))),
    }
}

/// `select=none|flock:<window>:<k>`: `Some(None)` clears a saved flock, `None`
/// is not given. The flock's sink is position 0.
fn select_arg(args: &Args) -> Result<Option<Option<FlockSelect>>> {
    args.optional("select")
        .map(|text| parse_flock_select(&text))
        .transpose()
}

/// The pointer options of a run as given: `pointer=none|<dim>`,
/// `pointer_score=dot|lorentz` and `pointer_select=none|flock:<window>:<k>|top:<k>`.
struct PointerArgs {
    /// `pointer=`: `Some(None)` is `none`, `None` is not given.
    head: Option<Option<usize>>,
    /// `pointer_score=`; `None` is not given.
    score: Option<ReadScore>,
    /// `pointer_select=`: `Some(None)` is `none` (it clears a saved selection),
    /// `None` is not given.
    select: Option<Option<PointerSelect>>,
    /// `pointer_route=`: `Some(None)` is `none` (it clears a saved route),
    /// `None` is not given.
    route: Option<Option<PrimeRoute>>,
}

impl PointerArgs {
    /// The options name a new head: `pointer=<dim>`, whose score defaults to
    /// Dot and whose selection to none. `seed` is the seed an added head's
    /// weights are drawn from (recorded in its `init_seed`), `None` for a head
    /// built with a fresh model.
    fn new_head(&self, seed: Option<u64>) -> Option<PointerConfig> {
        let dim = self.head.flatten()?;
        Some(PointerConfig {
            dim,
            score: self.score.unwrap_or(ReadScore::Dot),
            select: self.select.flatten(),
            init_seed: seed,
            route: self.route.flatten(),
        })
    }

    /// `pointer_score=`, `pointer_select=` and `pointer_route=` have nothing
    /// to configure without a head, given or saved.
    fn refuse_without_head(&self) -> Result<()> {
        if self.score.is_some() || self.select.is_some() || self.route.is_some() {
            return Err(invalid(
                "pointer_score=, pointer_select= and pointer_route= configure a pointer head: \
                 give pointer=<dim> (the model has none)",
            ));
        }
        Ok(())
    }
}

fn pointer_args(args: &Args) -> Result<PointerArgs> {
    let head = match args.optional("pointer").as_deref() {
        None => None,
        Some("none") => Some(None),
        Some(text) => match text.parse::<usize>() {
            Ok(dim) if dim > 0 => Some(Some(dim)),
            _ => {
                return Err(invalid(format!(
                    "invalid pointer={text} (none or a positive query width)"
                )))
            }
        },
    };
    let score = match args.optional("pointer_score").as_deref() {
        None => None,
        Some("dot") => Some(ReadScore::Dot),
        Some("lorentz") => Some(ReadScore::Lorentz),
        Some(other) => {
            return Err(invalid(format!(
                "invalid pointer_score={other} (dot or lorentz)"
            )))
        }
    };
    let select = args
        .optional("pointer_select")
        .map(|text| parse_pointer_select(&text))
        .transpose()?;
    let route = args
        .optional("pointer_route")
        .map(|text| parse_pointer_route(&text))
        .transpose()?;
    Ok(PointerArgs {
        head,
        score,
        select,
        route,
    })
}

/// A score's name as the command line and the reports give it.
fn score_name(score: ReadScore) -> &'static str {
    match score {
        ReadScore::Dot => "dot",
        ReadScore::Lorentz => "lorentz",
        ReadScore::L2 => "l2",
    }
}

/// A transport group's name as `rotation_group=` gives it.
fn rotation_group_name(group: RotationGroup) -> &'static str {
    match group {
        RotationGroup::Quaternion => "quaternion",
        RotationGroup::U1 => "u1",
    }
}

/// The refusal of a pointer head under `qat=true`.
fn pointer_qat_refusal() -> TrainingError {
    invalid(
        "qat=true with a pointer head: the pointer has no served representation for \
         quantization-aware training; train it without qat=true",
    )
}

/// `top:1` is not a training setting. Its one kept source has attention 1 and
/// `p_copy` is that source's match, so the pointer's query, key and scale get
/// no gradient (weight decay only shrinks them) and the gate alone learns.
/// The single-source pointer is soft-trained weights with `top:1` applied
/// afterwards (`m-world evaluate pointer_select=top:1`). A wider `top:K` and a
/// flock train the scores of the sources they keep.
/// A raw-logit evaluator's float comparator: the model at `model_dir`, refused
/// when it has a pointer head, whose distribution is the mixture and not the
/// raw logits `mode` scores.
fn load_raw_logit_comparator(model_dir: &Path, mode: &str) -> Result<StackModel> {
    let model = StackModel::load(model_dir, &Device::Cpu)?;
    check_raw_logit_evaluation(&model.config, mode)?;
    Ok(model)
}

fn refuse_trained_single_source(select: Option<PointerSelect>) -> Result<()> {
    if select == Some(PointerSelect::TopK(1)) {
        return Err(invalid(
            "pointer_select=top:1 is not a training setting: its one kept source has attention \
             1, so the pointer's query, key and scale get no gradient (weight decay only shrinks \
             them); train soft (pointer_select=none) or with a wider selection, and apply top:1 \
             to the trained weights with m-world evaluate pointer_select=top:1",
        ));
    }
    Ok(())
}

/// The stacks `qat=true` can train: those the geometric stack export writes.
fn check_qat_config(config: &StackConfig) -> Result<()> {
    if config.pointer.is_some() {
        return Err(pointer_qat_refusal());
    }
    if config.arch != StackArch::Geometric
        || config.memory.is_some()
        || !config.width.is_multiple_of(uor_r4_lut::GROUP)
    {
        return Err(invalid(format!(
            "qat=true trains the geometric stack export's representation: a geometric stack \
             without memory layers whose width is a multiple of {}",
            uor_r4_lut::GROUP
        )));
    }
    Ok(())
}

/// A file's path, size and SHA-256, taken once.
struct FileIdentity {
    path: PathBuf,
    bytes: u64,
    sha256: String,
}

impl FileIdentity {
    fn of(path: &Path) -> Result<Self> {
        Ok(Self {
            path: path.to_owned(),
            bytes: fs::metadata(path)?.len(),
            sha256: sha256_file(path)?,
        })
    }

    /// As [`identity`] records a file.
    fn identity(&self) -> Value {
        json!({"path": self.path, "bytes": self.bytes, "sha256": self.sha256})
    }

    /// Size and SHA-256 only: the resume lineage compares contents, not paths.
    fn content(&self) -> Value {
        json!({"bytes": self.bytes, "sha256": self.sha256})
    }
}

/// `init=`'s model and configuration, identified once when a run starts; the
/// lineage, the settings record and the inputs record all reuse these, so the
/// report binds the bytes the run loaded without hashing them again.
struct InitFiles {
    model: FileIdentity,
    config: FileIdentity,
}

impl InitFiles {
    fn identify(directory: &Path) -> Result<Self> {
        Ok(Self {
            model: FileIdentity::of(&directory.join("model.safetensors"))?,
            config: FileIdentity::of(&directory.join("config.json"))?,
        })
    }
}
/// `transport_snap=none|icosian` (default none).
fn transport_snap_arg(args: &Args) -> Result<Option<TransportSnap>> {
    match args.optional("transport_snap").as_deref() {
        None | Some("none") => Ok(None),
        Some("icosian") => Ok(Some(TransportSnap::Icosian)),
        Some(other) => Err(invalid(format!(
            "invalid transport_snap={other} (none or icosian)"
        ))),
    }
}

/// What a transport-snap run's report says its scores are.
const TRANSPORT_SNAP_SCOPE: &str =
    "every recurrence's unit transport quaternion was snapped, before its scaling by lambda, to \
     the nearest of the snap's roots (the 120 unit icosians of 2I), with straight-through \
     gradients to the rotation logits; the model's own forward (its `dev` scores, the samples and \
     replies) is snapped, and the `unsnapped` scores are the same weights with the transport free \
     on the same windows or panel (a served representation, if any, unchanged); \
     `transport_usage` counts the roots the snapped forward selected over every recurrence layer, \
     lane and evaluated position; the saved model is the float weights, with transport.json \
     recording the snap";

impl Settings {
    fn record(&self, init: Option<&InitFiles>) -> Value {
        let init = init.map_or(
            Value::Null,
            |files| json!({"model": files.model.identity(), "config": files.config.identity()}),
        );
        json!({
            "train": self.train, "train_weights": self.train_weights, "valid": self.valid,
            "lens": self.lens, "merges": self.merges, "tokenizer": self.tokenizer,
            "config": self.config, "init": init,
            "qat": self.qat, "qat_codec": self.qat.then(|| qat_codec().name().to_owned()),
            "transport_snap": self.transport_snap,
            "steps": self.steps, "batch": self.batch, "lr": self.lr,
            "warmup": self.warmup, "min_lr": self.min_lr, "weight_decay": self.weight_decay,
            "clip": self.clip, "eval_every": self.eval_every, "eval_windows": self.eval_windows,
            "final_windows": self.final_windows, "checkpoint_every": self.checkpoint_every,
            "sample_tokens": self.sample_tokens,
        })
    }

    /// Settings and input contents a resumed run must share with its parent.
    /// A run from `init=`, with `qat=true` or with a transport snap also
    /// shares those; other runs' lineage is unchanged, so their earlier
    /// checkpoints still resume.
    fn lineage(&self, inputs: &Value) -> Value {
        let mut lineage = json!({
            "config": self.config, "steps": self.steps, "batch": self.batch, "lr": self.lr,
            "warmup": self.warmup, "min_lr": self.min_lr, "weight_decay": self.weight_decay,
            "clip": self.clip, "eval_every": self.eval_every, "eval_windows": self.eval_windows,
            "train_weights": self.train_weights, "inputs": inputs,
        });
        if self.qat {
            lineage["qat"] = json!({"codec": qat_codec().name()});
        }
        if let Some(snap) = self.transport_snap {
            lineage["transport_snap"] = snap.record();
        }
        lineage
    }

    /// Content identities (size and SHA-256, not paths) of every file the run
    /// reads, so a resume must present the same bytes in the same order.
    /// `init` holds `init=`'s identities, taken when the run started.
    fn input_contents(&self, init: Option<&InitFiles>) -> Result<Value> {
        let content = |path: &Path| -> Result<Value> {
            Ok(json!({"bytes": fs::metadata(path)?.len(), "sha256": sha256_file(path)?}))
        };
        let mut inputs = json!({
            "train": self.train.iter().map(|p| content(p)).collect::<Result<Vec<_>>>()?,
            "valid": content(&self.valid)?,
            "lens": self.lens.as_deref().map(content).transpose()?,
            "merges": self.merges.as_deref().map(content).transpose()?,
            "tokenizer": self.tokenizer.as_deref().map(content).transpose()?,
        });
        if let Some(files) = init {
            inputs["init"] = json!({
                "model": files.model.content(),
                "config": files.config.content(),
            });
        }
        Ok(inputs)
    }
}

/// The configuration of `init=`'s model: its `config.json`, with `seed=`
/// (which seeds the window sampler) in place of the saved seed when given.
/// Architecture options given beside `init=` must agree with the saved model.
fn init_config(args: &Args, directory: &Path) -> Result<StackConfig> {
    let mut config: StackConfig =
        serde_json::from_slice(&fs::read(directory.join("config.json"))?)?;
    config.validate()?;
    let name = |arch: StackArch| match arch {
        StackArch::Geometric => "geometric",
        StackArch::Transformer => "transformer",
    };
    let mut saved: Vec<(&str, String)> = vec![
        ("arch", name(config.arch).to_owned()),
        ("pattern", config.pattern.clone()),
        ("read", score_name(config.read).to_owned()),
        ("rotation", config.rotation.to_string()),
        (
            "rotation_group",
            rotation_group_name(config.rotation_group).to_owned(),
        ),
        ("width", config.width.to_string()),
        ("heads", config.heads.to_string()),
        ("layers", config.layers().to_string()),
        ("context", config.context.to_string()),
        (
            match config.arch {
                StackArch::Transformer => "mlp",
                StackArch::Geometric => "stack_mlp",
            },
            config.mlp_hidden.to_string(),
        ),
    ];
    if let Some(memory) = &config.memory {
        let layers: Vec<String> = memory.layers.iter().map(ToString::to_string).collect();
        saved.extend([
            ("memory_layers", layers.join(",")),
            ("memory_sub_keys", memory.sub_keys.to_string()),
            ("memory_top_k", memory.top_k.to_string()),
            ("memory_heads", memory.heads.to_string()),
            ("memory_key_dim", memory.key_dim.to_string()),
            (
                "memory_score",
                match memory.score {
                    MemoryScore::Dot => "dot",
                    MemoryScore::Lorentz => "lorentz",
                }
                .to_owned(),
            ),
            (
                "memory_codebook",
                match memory.codebook {
                    Some(Codebook::H4) => "h4",
                    Some(Codebook::E8) => "e8",
                    None => "none",
                }
                .to_owned(),
            ),
        ]);
    }
    // Numbers compare as numbers, everything else as text.
    let agree = |given: &str, saved: &str| match (given.parse::<usize>(), saved.parse::<usize>()) {
        (Ok(a), Ok(b)) => a == b,
        _ => given == saved,
    };
    let mut conflicts: Vec<String> = saved
        .iter()
        .filter_map(|(key, value)| {
            let given = args.optional(key)?;
            (!agree(&given, value)).then(|| format!("{key}={given} (the model has {value})"))
        })
        .collect();
    let options = [
        "memory_layers",
        "memory_sub_keys",
        "memory_top_k",
        "memory_heads",
        "memory_key_dim",
        "memory_score",
        "memory_codebook",
    ];
    if config.memory.is_none() {
        conflicts.extend(
            options
                .iter()
                .filter(|key| args.optional(key).is_some())
                .map(|key| format!("{key}= (the model has no memory layers)")),
        );
    }
    // A geometric stack's `mlp=` is its control's; it must match the saved width.
    if let (StackArch::Geometric, Some(text)) = (config.arch, args.optional("mlp")) {
        let mlp: usize = text
            .parse()
            .map_err(|_| invalid(format!("invalid mlp={text}")))?;
        let mut control = StackConfig::transformer(
            config.width,
            config.heads,
            config.layers(),
            mlp,
            config.context,
            config.seed,
        )?;
        control.vocab_size = config.vocab_size;
        let matched = StackConfig::geometric_matched_to(
            &control,
            &config.pattern,
            config.read,
            config.rotation,
        )?
        .mlp_hidden;
        if matched != config.mlp_hidden {
            conflicts.push(format!(
                "mlp={mlp} matches a stack MLP of {matched} (the model has {})",
                config.mlp_hidden
            ));
        }
    }
    if !conflicts.is_empty() {
        return Err(invalid(format!(
            "init= takes the architecture of {}; these options conflict: {}",
            directory.display(),
            conflicts.join("; ")
        )));
    }
    if let Some(seed) = args.optional("seed") {
        config.seed = seed
            .parse()
            .map_err(|_| invalid(format!("invalid seed={seed}")))?;
    }
    Ok(config)
}

/// The model shape of `arch=` and the shape options: the control's shape,
/// #1017's by default, or a geometric stack with its width, heads, depth and
/// context whose MLP matches its parameter count. `stack_mlp` instead pins
/// the stack's MLP width, so two stacks can differ in one component at equal
/// width (and unequal parameter counts). `vocab` replaces the default
/// 4,096-token vocabulary first.
fn stack_config(args: &Args, vocab: Option<usize>) -> Result<StackConfig> {
    let seed: u64 = args.number("seed", 1)?;
    let read = match args.optional("read").as_deref() {
        None | Some("lorentz") => ReadScore::Lorentz,
        Some("dot") => ReadScore::Dot,
        Some("l2") => ReadScore::L2,
        Some(other) => return Err(invalid(format!("unknown read {other}"))),
    };
    let rotation = match args.optional("rotation").as_deref() {
        None | Some("true") => true,
        Some("false") => false,
        Some(other) => return Err(invalid(format!("invalid rotation={other}"))),
    };
    let rotation_group = match args.optional("rotation_group").as_deref() {
        None | Some("quaternion") => RotationGroup::Quaternion,
        Some("u1") => RotationGroup::U1,
        Some(other) => {
            return Err(invalid(format!(
                "invalid rotation_group={other} (quaternion or u1)"
            )))
        }
    };
    let mut control = StackConfig::transformer(
        args.number("width", 288)?,
        args.number("heads", 6)?,
        args.number("layers", 6)?,
        args.number("mlp", 768)?,
        args.number("context", 256)?,
        seed,
    )?;
    if let Some(vocab) = vocab {
        control.vocab_size = vocab;
    }
    let stack_mlp = args
        .optional("stack_mlp")
        .map(|text| {
            text.parse::<usize>()
                .map_err(|_| invalid(format!("invalid stack_mlp={text}")))
        })
        .transpose()?;
    let mut config = match args.required("arch")?.as_str() {
        "transformer" if stack_mlp.is_some() => {
            return Err(invalid(
                "stack_mlp= sets a geometric stack's MLP; the control's is mlp=",
            ))
        }
        "transformer" if rotation_group != RotationGroup::Quaternion => {
            return Err(invalid(
                "rotation_group= restricts a geometric stack's recurrence transport",
            ))
        }
        "transformer" => control,
        "geometric" => {
            let layers = control.layers();
            let default_pattern = "rra".repeat(layers / 3) + &"r".repeat(layers % 3);
            let mut config = StackConfig::geometric_matched_to(
                &control,
                &args.optional("pattern").unwrap_or(default_pattern),
                read,
                rotation,
            )?;
            // The U(1) control has the quaternion stack's gate shapes, so the
            // matched MLP width is unchanged.
            config.rotation_group = rotation_group;
            config.validate()?;
            if let Some(hidden) = stack_mlp {
                config.mlp_hidden = hidden;
                config.validate()?;
            }
            config
        }
        other => return Err(invalid(format!("unknown arch {other}"))),
    };
    // Product-key memories replace the listed layers' MLPs after the MLP width
    // is matched, so the other layers keep the matched width.
    if let Some(layers) = args.optional("memory_layers") {
        // A fixed codebook sets the sub-key count and the key width.
        let codebook = match args.optional("memory_codebook").as_deref() {
            None => None,
            Some("h4") => Some(Codebook::H4),
            Some("e8") => Some(Codebook::E8),
            Some(other) => return Err(invalid(format!("unknown memory_codebook {other}"))),
        };
        let (default_sub_keys, default_key_dim) =
            codebook.map_or((256, 128), |c| (c.size(), 2 * c.dim()));
        config.memory = Some(MemoryConfig {
            layers: layers
                .split(',')
                .map(|l| {
                    l.parse()
                        .map_err(|_| invalid(format!("invalid memory layer {l}")))
                })
                .collect::<Result<_>>()?,
            sub_keys: args.number("memory_sub_keys", default_sub_keys)?,
            top_k: args.number("memory_top_k", 32)?,
            heads: args.number("memory_heads", 4)?,
            key_dim: args.number("memory_key_dim", default_key_dim)?,
            score: match args.optional("memory_score").as_deref() {
                None | Some("dot") => MemoryScore::Dot,
                Some("lorentz") => MemoryScore::Lorentz,
                Some(other) => return Err(invalid(format!("unknown memory_score {other}"))),
            },
            codebook,
        });
        config.validate()?;
    }
    // The A1 read mechanisms, on any fresh shape (`dialogue-train` alone
    // accepts the options; the other modes never see them).
    if let Some(select) = select_arg(args)? {
        config.select = select;
    }
    // A fresh shape's head is built with its model: no `init_seed`.
    let pointer = pointer_args(args)?;
    match pointer.new_head(None) {
        Some(head) => config.pointer = Some(head),
        None => pointer.refuse_without_head()?,
    }
    config.validate()?;
    Ok(config)
}

fn train_settings(args: &Args) -> Result<Settings> {
    let init = args.optional("init").map(PathBuf::from);
    let config = match &init {
        Some(directory) => init_config(args, directory)?,
        None => stack_config(args, None)?,
    };
    // An `init=` model saved with a single-source pointer would train nothing
    // but its gate; refused here as `dialogue-train` refuses it.
    refuse_trained_single_source(config.pointer.and_then(|pointer| pointer.select))?;
    let qat = qat_flag(args)?;
    if qat {
        check_qat_config(&config)?;
    }
    let transport_snap = transport_snap_arg(args)?;
    if let Some(snap) = transport_snap {
        snap.check(&config)?;
    }
    let train: Vec<PathBuf> = args
        .required("train")?
        .split(',')
        .map(PathBuf::from)
        .collect();
    let train_weights: Vec<f64> = match args.optional("train_weights") {
        None => vec![1.0; train.len()],
        Some(text) => text
            .split(',')
            .map(|w| {
                w.parse()
                    .map_err(|_| invalid(format!("invalid train weight {w}")))
            })
            .collect::<Result<_>>()?,
    };
    if train_weights.len() != train.len()
        || train_weights
            .iter()
            .any(|w| w.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater))
    {
        return Err(invalid("one positive train weight per training stream"));
    }
    let settings = Settings {
        train,
        train_weights,
        valid: PathBuf::from(args.required("valid")?),
        lens: args.optional("lens").map(PathBuf::from),
        merges: args.optional("merges").map(PathBuf::from),
        tokenizer: args.optional("tokenizer").map(PathBuf::from),
        config,
        init,
        qat,
        transport_snap,
        steps: args.number("steps", 7324)?,
        batch: args.number("batch", 16)?,
        lr: args.number("lr", 0.002)?,
        warmup: args.number("warmup", 200)?,
        min_lr: args.number("min_lr", 0.1)?,
        weight_decay: args.number("weight_decay", 0.1)?,
        clip: args.number("clip", 1.0)?,
        eval_every: args.number("eval_every", 250)?,
        eval_windows: args.number("eval_windows", 64)?,
        final_windows: args.number("final_windows", 512)?,
        checkpoint_every: args.number("checkpoint_every", 250)?,
        resume: args.optional("resume").map(PathBuf::from),
        max_seconds: args.number("max_seconds", f64::INFINITY)?,
        sample_tokens: args.number("sample_tokens", 128)?,
        device: device_arg(args)?,
        data_parallel: args.number("data_parallel", 1)?,
    };
    if !(1..=2).contains(&settings.data_parallel) {
        return Err(invalid("data_parallel must be 1 or 2"));
    }
    if settings.data_parallel == 2
        && (!matches!(settings.device, Device::Cuda(_))
            || settings.batch % 2 != 0
            || settings.qat
            || settings.transport_snap.is_some())
    {
        return Err(invalid(
            "data_parallel=2 needs device=cuda, an even batch, and no qat or transport_snap",
        ));
    }
    if settings.steps == 0
        || settings.batch == 0
        || settings.eval_every == 0
        || settings.eval_windows == 0
        || settings.final_windows == 0
        || settings.lr.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)
        || !(0.0..=1.0).contains(&settings.min_lr)
    {
        return Err(invalid(
            "steps, batch, evaluation counts and lr must be positive; min_lr in [0, 1]",
        ));
    }
    Ok(settings)
}

/// `UOR_NAN_TRACE`: rerun the failing batch through the forward with every
/// weight map's input captured, and name the first site whose input holds a
/// nonfinite value and the final state's finiteness (diagnosis only).
fn first_nonfinite_site(
    model: &StackModel,
    ids: &[u32],
    batch: usize,
    time: usize,
) -> Result<String> {
    let mut first: Option<String> = None;
    let mut sites = 0usize;
    let hidden = model.hidden_with_capture(ids, batch, time, &mut |site, input| {
        sites += 1;
        if first.is_none() {
            let values = input.flatten_all()?.to_vec1::<f32>()?;
            let bad = values.iter().filter(|v| !v.is_finite()).count();
            if bad > 0 {
                let largest = values
                    .iter()
                    .filter(|v| v.is_finite())
                    .fold(0f32, |m, v| m.max(v.abs()));
                first = Some(format!(
                    "{site:?}: {bad}/{} nonfinite, largest finite {largest:e}",
                    values.len()
                ));
            }
        }
        Ok(())
    })?;
    let final_bad = hidden
        .flatten_all()?
        .to_vec1::<f32>()?
        .iter()
        .filter(|v| !v.is_finite())
        .count();
    Ok(format!(
        "; first nonfinite site of {sites}: {}; final state nonfinite values {final_bad}",
        first.unwrap_or_else(|| "none (the head or loss)".into())
    ))
}

/// `UOR_NAN_TRACE`: fail before the update when any gradient (or parameter)
/// is nonfinite, naming the variables, their largest finite magnitude and
/// the step, so a run that would poison every parameter shows where the
/// nonfinite value first appears.
fn check_finite_gradients(
    model: &StackModel,
    grads: &candle_core::backprop::GradStore,
    step: usize,
    loss: f64,
) -> Result<()> {
    let mut bad = Vec::new();
    for (name, var) in model.variables() {
        let parameter = var.as_tensor().flatten_all()?.to_vec1::<f32>()?;
        if parameter.iter().any(|v| !v.is_finite()) {
            bad.push(format!("{name}: parameter nonfinite"));
        }
        if let Some(grad) = grads.get(var.as_tensor()) {
            let values = grad.flatten_all()?.to_vec1::<f32>()?;
            let nonfinite = values.iter().filter(|v| !v.is_finite()).count();
            if nonfinite > 0 {
                let largest = values
                    .iter()
                    .filter(|v| v.is_finite())
                    .fold(0f32, |m, v| m.max(v.abs()));
                bad.push(format!(
                    "{name}: {nonfinite}/{} gradient values nonfinite, largest finite {largest:e}",
                    values.len()
                ));
            }
        }
    }
    if bad.is_empty() {
        return Ok(());
    }
    Err(invalid(format!(
        "nonfinite gradients at step {step} (loss {loss}): {}",
        bad.join("; ")
    )))
}

fn learning_rate(settings: &Settings, step: usize) -> f64 {
    cosine_rate(
        settings.lr,
        settings.warmup,
        settings.min_lr,
        settings.steps,
        step,
    )
}

/// Linear warmup to `lr`, then a cosine decay to `min_lr * lr` at `steps`.
fn cosine_rate(lr: f64, warmup: usize, min_lr: f64, steps: usize, step: usize) -> f64 {
    if step < warmup {
        return lr * (step + 1) as f64 / warmup as f64;
    }
    let span = (steps - warmup).max(1) as f64;
    let progress = ((step - warmup) as f64 / span).min(1.0);
    let floor = lr * min_lr;
    floor + (lr - floor) * 0.5 * (1.0 + (std::f64::consts::PI * progress).cos())
}

struct Evaluation {
    nll: f64,
    bits_per_byte: Option<f64>,
    targets: usize,
}

impl Evaluation {
    fn record(&self) -> Value {
        json!({"nll": self.nll, "bits_per_byte": self.bits_per_byte, "targets": self.targets})
    }
}

/// Evenly spaced windows over VALID (the `joint-read-geometry` rule).
fn evaluate(
    model: &StackModel,
    valid: &[u32],
    lens: Option<&[u32]>,
    windows: usize,
) -> Result<Evaluation> {
    let time = model.config.context;
    let stride = (valid.len() - time - 1) / windows;
    let starts: Vec<usize> = (0..windows).map(|window| window * stride).collect();
    let (mut nll, mut bytes, mut targets_seen) = (0f64, 0f64, 0usize);
    for group in starts.chunks(16) {
        let mut ids = Vec::with_capacity(group.len() * time);
        let mut targets = Vec::with_capacity(group.len() * time);
        for &start in group {
            ids.extend_from_slice(&valid[start..start + time]);
            targets.extend_from_slice(&valid[start + 1..start + time + 1]);
        }
        if let Some(lens) = lens {
            bytes += targets
                .iter()
                .map(|&id| f64::from(lens[id as usize]))
                .sum::<f64>();
        }
        nll += model
            .target_nll(&ids, &targets, group.len(), time)?
            .iter()
            .sum::<f64>();
        targets_seen += targets.len();
    }
    Ok(Evaluation {
        nll: nll / targets_seen as f64,
        bits_per_byte: lens.map(|_| nll / std::f64::consts::LN_2 / bytes),
        targets: targets_seen,
    })
}

/// `f` on the model in its mode and, in served mode (a QAT run), on its float
/// weights too.
fn in_both_modes<T>(
    model: &mut StackModel,
    f: impl Fn(&StackModel) -> Result<T>,
) -> Result<(T, Option<T>)> {
    let current = f(model)?;
    let float = match model.served_codec() {
        Some(_) => Some(model.with_float_forward(&f)?),
        None => None,
    };
    Ok((current, float))
}

/// [`evaluate`] in the model's mode and, in served mode (a QAT run), of its
/// float weights too, on the same windows.
fn evaluate_modes(
    model: &mut StackModel,
    valid: &[u32],
    lens: Option<&[u32]>,
    windows: usize,
) -> Result<(Evaluation, Option<Evaluation>)> {
    in_both_modes(model, |model| evaluate(model, valid, lens, windows))
}

/// In a transport-snap run, on the windows [`evaluate`] scores: the NLL of
/// the same weights with the transport free (a served representation
/// unchanged), and the roots the snapped forward selects there.
fn evaluate_transport(
    model: &mut StackModel,
    valid: &[u32],
    lens: Option<&[u32]>,
    windows: usize,
) -> Result<Option<(Evaluation, TransportUsage)>> {
    let Some(snap) = model.transport_snap() else {
        return Ok(None);
    };
    let unsnapped =
        model.with_unsnapped_transport(|model| evaluate(model, valid, lens, windows))?;
    let time = model.config.context;
    let stride = (valid.len() - time - 1) / windows;
    let starts: Vec<usize> = (0..windows).map(|window| window * stride).collect();
    let mut usage = TransportUsage::new(snap, &model.config);
    for group in starts.chunks(16) {
        let ids: Vec<u32> = group
            .iter()
            .flat_map(|&start| valid[start..start + time].iter().copied())
            .collect();
        usage.add(&model.transport_usage(&ids, group.len(), time, None)?)?;
    }
    Ok(Some((unsnapped, usage)))
}

/// A transport-snap evaluation into an evaluation point: the unsnapped
/// score under `key`, the usage under `transport_usage`.
fn record_transport(
    point: &mut Value,
    key: &str,
    transport: Option<&(Evaluation, TransportUsage)>,
) {
    if let Some((unsnapped, usage)) = transport {
        point[key] = unsnapped.record();
        point["transport_usage"] = usage.summary();
    }
}

/// Seconds of this process's updates: the first (which includes one-time
/// allocation) apart from the rest.
fn step_timing(seconds: &[f64]) -> Value {
    let rest = seconds.get(1..).unwrap_or(&[]);
    let mean = |values: &[f64]| {
        (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
    };
    json!({
        "steps": seconds.len(),
        "first": seconds.first(),
        "mean_after_first": mean(rest),
        "min_after_first": rest.iter().copied().reduce(f64::min),
        "max_after_first": rest.iter().copied().reduce(f64::max),
    })
}

/// Add the served representation's work between `before` and `after` to `total`.
fn add_served_work(
    total: &mut ServedStatistics,
    before: Option<ServedStatistics>,
    after: Option<ServedStatistics>,
) {
    if let (Some(before), Some(after)) = (before, after) {
        total.checks += after.checks - before.checks;
        total.refreshes += after.refreshes - before.refreshes;
        total.tensors += after.tensors - before.tensors;
        total.seconds += after.seconds - before.seconds;
    }
}

/// Greedy (`temperature = 0`) or top-k sampled continuation of `prompt`.
fn generate(
    model: &StackModel,
    prompt: &[u32],
    new_tokens: usize,
    temperature: f64,
    top_k: usize,
    rng: &mut Rng,
) -> Result<Vec<u32>> {
    let context = model.config.context;
    let mut tokens = prompt.to_vec();
    for _ in 0..new_tokens {
        let window = &tokens[tokens.len().saturating_sub(context)..];
        // The last position's logits, or a pointer model's mixture scores.
        let last = model.next_scores(window)?;
        let next = if temperature <= 0.0 {
            last.iter()
                .enumerate()
                .max_by(|a, b| a.1.total_cmp(b.1))
                .map(|(i, _)| i)
                .ok_or_else(|| invalid("empty logits"))?
        } else {
            let mut ranked: Vec<(usize, f64)> = last
                .iter()
                .enumerate()
                .map(|(i, &l)| (i, f64::from(l) / temperature))
                .collect();
            ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
            ranked.truncate(top_k.max(1));
            let maximum = ranked[0].1;
            let total: f64 = ranked.iter().map(|(_, l)| (l - maximum).exp()).sum();
            let mut draw = rng.uniform() * total;
            let mut chosen = ranked[ranked.len() - 1].0;
            for &(i, l) in &ranked {
                draw -= (l - maximum).exp();
                if draw <= 0.0 {
                    chosen = i;
                    break;
                }
            }
            chosen
        };
        tokens.push(next as u32);
    }
    Ok(tokens[prompt.len()..].to_vec())
}

#[allow(clippy::too_many_arguments)]
fn samples(
    model: &StackModel,
    valid: &[u32],
    decoder: Option<&Decoder>,
    prompts: usize,
    prompt_tokens: usize,
    new_tokens: usize,
    temperature: f64,
    top_k: usize,
    seed: u64,
) -> Result<Value> {
    let mut rng = Rng(seed ^ 0x5341_4D50);
    let stride = (valid.len() - prompt_tokens) / prompts.max(1);
    let mut out = Vec::new();
    for index in 0..prompts {
        let prompt = &valid[index * stride..index * stride + prompt_tokens];
        let greedy = generate(model, prompt, new_tokens, 0.0, 1, &mut rng)?;
        let sampled = generate(model, prompt, new_tokens, temperature, top_k, &mut rng)?;
        let text = |ids: &[u32]| decoder.map(|decoder| decoder.decode(ids));
        out.push(json!({
            "prompt_start": index * stride, "prompt": text(prompt),
            "greedy": text(&greedy), "sampled": text(&sampled),
            "greedy_ids": greedy, "sampled_ids": sampled,
        }));
    }
    Ok(json!({"temperature": temperature, "top_k": top_k, "seed": seed, "rows": out}))
}

struct Progress {
    step: usize,
    rng: Rng,
    curve: Vec<Value>,
    train_seconds: f64,
}

fn save_checkpoint(
    out: &Path,
    model: &StackModel,
    optimizer: &StackAdamW,
    progress: &Progress,
    lineage: &Value,
) -> Result<()> {
    let staging = out.join("checkpoint.partial");
    let _ = fs::remove_dir_all(&staging);
    model.save(&staging.join("model"))?;
    optimizer.save(&staging.join("optimizer"))?;
    fs::write(
        staging.join("state.json"),
        serde_json::to_vec_pretty(&json!({
            "step": progress.step, "rng": progress.rng.0.to_string(), "curve": progress.curve,
            "train_seconds": progress.train_seconds, "lineage": lineage,
        }))?,
    )?;
    let final_path = out.join("checkpoint");
    let _ = fs::remove_dir_all(&final_path);
    fs::rename(&staging, &final_path)?;
    Ok(())
}

/// The model, optimizer and progress a stopped run saved, if its lineage is
/// `lineage`; with the identity of its state file.
fn load_checkpoint(
    checkpoint: &Path,
    lineage: &Value,
    device: &Device,
) -> Result<(StackModel, StackAdamW, Progress, Value)> {
    let state: Value = serde_json::from_slice(&fs::read(checkpoint.join("state.json"))?)?;
    if &state["lineage"] != lineage {
        let keys = |value: &Value| -> Vec<String> {
            value
                .as_object()
                .map(|object| object.keys().cloned().collect())
                .unwrap_or_default()
        };
        let mut differing: Vec<String> = keys(lineage)
            .into_iter()
            .chain(keys(&state["lineage"]))
            .filter(|key| state["lineage"][key.as_str()] != lineage[key.as_str()])
            .collect();
        differing.sort();
        differing.dedup();
        return Err(invalid(format!(
            "the resume differs from the checkpoint's lineage in: {}",
            differing.join(", ")
        )));
    }
    let model = StackModel::load(&checkpoint.join("model"), device)?;
    let optimizer = StackAdamW::load(&checkpoint.join("optimizer"), &model)?;
    let progress = Progress {
        step: state["step"]
            .as_u64()
            .ok_or_else(|| invalid("checkpoint step"))? as usize,
        rng: Rng(state["rng"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| invalid("checkpoint rng"))?),
        curve: state["curve"].as_array().cloned().unwrap_or_default(),
        train_seconds: state["train_seconds"].as_f64().unwrap_or(0.0),
    };
    if optimizer.step != progress.step {
        return Err(invalid("checkpoint optimizer and progress steps differ"));
    }
    Ok((
        model,
        optimizer,
        progress,
        identity(&checkpoint.join("state.json"))?,
    ))
}

fn train(settings: &Settings, out: &Path) -> Result<()> {
    let device = settings.device.clone();
    // `init=`'s files are hashed once, here, before the model is loaded.
    let init_files = settings
        .init
        .as_deref()
        .map(InitFiles::identify)
        .transpose()?;
    let lineage = settings.lineage(&settings.input_contents(init_files.as_ref())?);
    let vocabulary = settings.config.vocab_size;
    let train: Vec<Vec<u32>> = settings
        .train
        .iter()
        .map(|path| read_tokens(path, vocabulary))
        .collect::<Result<_>>()?;
    let weight_total: f64 = settings.train_weights.iter().sum();
    let valid = read_tokens(&settings.valid, vocabulary)?;
    let lens = settings
        .lens
        .as_ref()
        .map(|path| read_tokens(path, u16::MAX as usize + 1))
        .transpose()?;
    if lens.as_ref().is_some_and(|lens| lens.len() != vocabulary) {
        return Err(invalid("lens must hold one byte length per token id"));
    }
    let decoder = Decoder::load(settings.merges.as_deref(), settings.tokenizer.as_deref())?;
    let time = settings.config.context;
    if train.iter().any(|stream| stream.len() <= time + 1)
        || valid.len() <= time + settings.final_windows
    {
        return Err(invalid(
            "token files are too short for the context and windows",
        ));
    }
    let (mut model, mut optimizer, mut progress, resumed_from) = match &settings.resume {
        None => {
            let model = match &settings.init {
                Some(directory) => {
                    let mut model = StackModel::load(directory, &device)?;
                    // `seed=` alone may differ from the saved configuration.
                    let mut saved = model.config.clone();
                    saved.seed = settings.config.seed;
                    if saved != settings.config {
                        return Err(invalid("init='s model differs from its config.json"));
                    }
                    model.config = saved;
                    model
                }
                None => StackModel::new(settings.config.clone(), &device)?,
            };
            let optimizer = StackAdamW::new(&model, settings.weight_decay, settings.clip)?;
            let progress = Progress {
                step: 0,
                rng: Rng(settings.config.seed ^ 0x5749_4E44_4F57),
                curve: Vec::new(),
                train_seconds: 0.0,
            };
            (model, optimizer, progress, None)
        }
        Some(checkpoint) => {
            let (model, optimizer, progress, state) =
                load_checkpoint(checkpoint, &lineage, &device)?;
            (model, optimizer, progress, Some(state))
        }
    };
    let parameters = model.parameter_count();
    let active_parameters = model.config.active_parameter_count()?;
    eprintln!(
        "{:?} pattern {} read {:?} rotation {}: {parameters} parameters ({active_parameters} read per token), mlp {}",
        model.config.arch,
        model.config.pattern,
        model.config.read,
        model.config.rotation,
        model.config.mlp_hidden
    );
    if settings.qat {
        model.set_served_representation(Some(qat_codec()))?;
    }
    // After `init=` or a resume alike: the requested setting replaces any snap the load restored.
    model.set_transport_snap(settings.transport_snap)?;
    // data_parallel=2: a replica on GPU 1 with the primary's current weights
    // (after init= or a resume alike).
    let replica = if settings.data_parallel == 2 {
        let replica = StackModel::new(model.config.clone(), &Device::new_cuda(1)?)?;
        replica.copy_variables_from(&model)?;
        eprintln!(
            "data parallel: replica on GPU 1, batch halves of {}",
            settings.batch / 2
        );
        Some(replica)
    } else {
        None
    };
    // A run from a trained model scores it before any update.
    if settings.init.is_some() && progress.step == 0 {
        let (evaluation, float) =
            evaluate_modes(&mut model, &valid, lens.as_deref(), settings.eval_windows)?;
        let transport =
            evaluate_transport(&mut model, &valid, lens.as_deref(), settings.eval_windows)?;
        let mut point = json!({"step": 0, "tokens": 0, "dev": evaluation.record()});
        if let Some(float) = &float {
            point["dev_float"] = float.record();
        }
        record_transport(&mut point, "dev_unsnapped", transport.as_ref());
        eprintln!("{point}");
        progress.curve.push(point);
    }
    let mut window_loss = (0f64, 0usize);
    let mut stopped_early = false;
    // This process's updates: their seconds, and the served representation's
    // work inside them.
    let mut step_seconds = Vec::new();
    let mut served_in_steps = ServedStatistics::default();
    let started = Instant::now();
    // UOR_NAN_TRACE=1: before every update, stop at the first nonfinite
    // gradient and name the variables that carry it (diagnosis only).
    let nan_trace = std::env::var("UOR_NAN_TRACE").is_ok_and(|v| v == "1");
    while progress.step < settings.steps {
        let lr = learning_rate(settings, progress.step);
        let clock = Instant::now();
        let mut ids = Vec::with_capacity(settings.batch * time);
        let mut targets = Vec::with_capacity(settings.batch * time);
        for _ in 0..settings.batch {
            let mut draw = progress.rng.uniform() * weight_total;
            let mut stream = &train[train.len() - 1];
            for (candidate, weight) in train.iter().zip(&settings.train_weights) {
                draw -= weight;
                if draw <= 0.0 {
                    stream = candidate;
                    break;
                }
            }
            let start = progress.rng.below(stream.len() - time - 1);
            ids.extend_from_slice(&stream[start..start + time]);
            targets.extend_from_slice(&stream[start + 1..start + time + 1]);
        }
        let served_before = model.served_statistics()?;
        let (value, grads) = match &replica {
            None => {
                let loss = model.loss(&ids, &targets, settings.batch, time)?;
                (f64::from(loss.to_scalar::<f32>()?), loss.backward()?)
            }
            Some(replica) => {
                // Equal halves of the same drawn batch: the mean of the two
                // half-batch losses and gradients is the full-batch mean.
                let half = settings.batch / 2;
                let split = half * time;
                let half_step =
                    |model: &StackModel,
                     ids: &[u32],
                     targets: &[u32]|
                     -> Result<(f64, candle_core::backprop::GradStore)> {
                        let loss = model.loss(ids, targets, half, time)?;
                        Ok((f64::from(loss.to_scalar::<f32>()?), loss.backward()?))
                    };
                let (local, remote) = std::thread::scope(|scope| {
                    let worker =
                        scope.spawn(|| half_step(replica, &ids[split..], &targets[split..]));
                    let local = half_step(&model, &ids[..split], &targets[..split]);
                    let remote = worker
                        .join()
                        .map_err(|_| invalid("the replica's thread panicked"));
                    (local, remote)
                });
                let (local, remote) = (local?, remote??);
                let mut grads = local.1;
                average_replica_gradients(&model, &mut grads, replica, &remote.1)?;
                ((local.0 + remote.0) / 2.0, grads)
            }
        };
        if !value.is_finite() {
            let site = if nan_trace {
                first_nonfinite_site(&model, &ids, settings.batch, time)?
            } else {
                String::new()
            };
            return Err(invalid(format!(
                "nonfinite training loss at step {}{site}",
                progress.step
            )));
        }
        if nan_trace {
            check_finite_gradients(&model, &grads, progress.step, value)?;
        }
        let grad_norm = optimizer.update(&model, &grads, lr)?;
        if let Some(replica) = &replica {
            replica.copy_variables_from(&model)?;
        }
        let seconds = clock.elapsed().as_secs_f64();
        add_served_work(
            &mut served_in_steps,
            served_before,
            model.served_statistics()?,
        );
        step_seconds.push(seconds);
        progress.train_seconds += seconds;
        progress.step += 1;
        window_loss.0 += value;
        window_loss.1 += 1;
        if progress.step % settings.eval_every == 0 || progress.step == settings.steps {
            let (evaluation, float) =
                evaluate_modes(&mut model, &valid, lens.as_deref(), settings.eval_windows)?;
            let transport =
                evaluate_transport(&mut model, &valid, lens.as_deref(), settings.eval_windows)?;
            let tokens = progress.step * settings.batch * time;
            let mut point = json!({
                "step": progress.step, "tokens": tokens, "lr": lr,
                "train_loss": window_loss.0 / window_loss.1.max(1) as f64,
                "dev": evaluation.record(), "grad_norm": grad_norm,
                "train_seconds": progress.train_seconds,
                "tokens_per_second": tokens as f64 / progress.train_seconds,
            });
            if let Some(float) = &float {
                point["dev_float"] = float.record();
            }
            record_transport(&mut point, "dev_unsnapped", transport.as_ref());
            eprintln!("{point}");
            progress.curve.push(point);
            window_loss = (0.0, 0);
        }
        if settings.checkpoint_every > 0
            && progress.step % settings.checkpoint_every == 0
            && progress.step < settings.steps
        {
            save_checkpoint(out, &model, &optimizer, &progress, &lineage)?;
        }
        if started.elapsed().as_secs_f64() > settings.max_seconds {
            stopped_early = true;
            save_checkpoint(out, &model, &optimizer, &progress, &lineage)?;
            break;
        }
    }
    let (final_evaluation, final_float) =
        evaluate_modes(&mut model, &valid, lens.as_deref(), settings.final_windows)?;
    let final_transport =
        evaluate_transport(&mut model, &valid, lens.as_deref(), settings.final_windows)?;
    eprintln!("final: {}", final_evaluation.record());
    if let Some(float) = &final_float {
        eprintln!("final float: {}", float.record());
    }
    if let Some((unsnapped, _)) = &final_transport {
        eprintln!("final unsnapped: {}", unsnapped.record());
    }
    model.save(&out.join("model"))?;
    if !stopped_early {
        // A completed run keeps only its final model; a stopped one keeps the
        // checkpoint that `resume=` continues from.
        let _ = fs::remove_dir_all(out.join("checkpoint"));
    }
    let sample_record = if settings.sample_tokens > 0 {
        samples(
            &model,
            &valid,
            decoder.as_ref(),
            3,
            64,
            settings.sample_tokens,
            0.8,
            40,
            settings.config.seed,
        )?
    } else {
        Value::Null
    };
    let executable = std::env::current_exe()?;
    let mut report = json!({
        "schema": "uor-r4.geometric-stack-run/1",
        "settings": settings.record(init_files.as_ref()),
        "parameters": parameters,
        "active_parameters": active_parameters,
        "completed_steps": progress.step,
        "stopped_early": stopped_early,
        "resumed_from": resumed_from,
        "target_visits": progress.step * settings.batch * time,
        "train_seconds": progress.train_seconds,
        "tokens_per_second": (progress.step * settings.batch * time) as f64 / progress.train_seconds.max(1e-9),
        "step_seconds": step_timing(&step_seconds),
        "data_parallel": settings.data_parallel,
        "tf32": tf32_enabled(),
        "threads": std::env::var("RAYON_NUM_THREADS").ok(),
        "curve": progress.curve,
        "final": final_evaluation.record(),
        "samples": sample_record,
        "inputs": {
            "train": settings.train.iter().map(|p| identity(p)).collect::<Result<Vec<_>>>()?,
            "valid": identity(&settings.valid)?,
            "lens": settings.lens.as_ref().map(|p| identity(p)).transpose()?,
            "merges": settings.merges.as_ref().map(|p| identity(p)).transpose()?,
            "tokenizer": settings.tokenizer.as_ref().map(|p| identity(p)).transpose()?,
            "init": init_files.as_ref().map(|files| files.model.identity()),
        },
        "executable": identity(&executable)?,
        "model_sha256": sha256_file(&out.join("model").join("model.safetensors"))?,
    });
    if let Some(codec) = model.served_codec() {
        report["qat"] = json!({
            "codec": codec.name(),
            "representation": "the forward pass read the round-to-nearest export's values (4-bit maps with the norm gains folded in, grid-code scalars, fixed-point biases, age tables and offsets) with straight-through gradients; `dev` and `final` score that representation, `dev_float` and `final_float` the float weights on the same windows; samples come from the served representation; the saved model's config.json records the codec (`served_representation`), and `export` writes it by rounding to nearest only",
            "final_float": final_float.as_ref().map(Evaluation::record),
            "served_work_in_updates": served_in_steps,
            "served_work_total": model.served_statistics()?,
        });
    }
    if let Some(snap) = model.transport_snap() {
        let mut section = snap.record();
        section["scope"] = json!(TRANSPORT_SNAP_SCOPE);
        section["saved_record"] = json!(StackModel::saved_transport_snap(&out.join("model"))?);
        record_transport(&mut section, "final_unsnapped", final_transport.as_ref());
        report["transport_snap"] = section;
    }
    fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    Ok(())
}

fn sample_mode(arguments: &[String]) -> Result<()> {
    let args = Args::parse(
        arguments,
        &[
            "model",
            "valid",
            "merges",
            "tokenizer",
            "out",
            "prompts",
            "prompt_tokens",
            "sample_tokens",
            "temperature",
            "top_k",
            "seed",
        ],
    )?;
    let model_dir = PathBuf::from(args.required("model")?);
    let valid_path = PathBuf::from(args.required("valid")?);
    let merges = args.optional("merges").map(PathBuf::from);
    let tokenizer = args.optional("tokenizer").map(PathBuf::from);
    if merges.is_none() && tokenizer.is_none() {
        return Err(invalid("sample needs merges= or tokenizer="));
    }
    let out = PathBuf::from(args.required("out")?);
    let prompts: usize = args.number("prompts", 3)?;
    let prompt_tokens: usize = args.number("prompt_tokens", 64)?;
    let sample_tokens: usize = args.number("sample_tokens", 128)?;
    let temperature: f64 = args.number("temperature", 0.8)?;
    let top_k: usize = args.number("top_k", 40)?;
    let seed: u64 = args.number("seed", 1)?;
    report_output::claim(&out)?;
    let result = (|| -> Result<()> {
        // `sample` runs the artifact as saved: the load restores its transport snap by design.
        let model = StackModel::load(&model_dir, &Device::Cpu)?;
        let valid = read_tokens(&valid_path, model.config.vocab_size)?;
        let decoder = Decoder::load(merges.as_deref(), tokenizer.as_deref())?;
        let record = samples(
            &model,
            &valid,
            decoder.as_ref(),
            prompts,
            prompt_tokens,
            sample_tokens,
            temperature,
            top_k,
            seed,
        )?;
        fs::write(
            out.join("samples.json"),
            serde_json::to_vec_pretty(&json!({
                "model": identity(&model_dir.join("model.safetensors"))?, "config": model.config,
                "samples": record,
            }))?,
        )?;
        Ok(())
    })();
    finish(&out, result)
}

/// Consecutive 256-input blocks, fresh state per block: the retained
/// evaluator's protocol. Reports tune, comparison and full means.
fn evaluate_mode(arguments: &[String]) -> Result<()> {
    let args = Args::parse(
        arguments,
        &["model", "tokens", "out", "tune_blocks", "lens"],
    )?;
    let model_dir = PathBuf::from(args.required("model")?);
    let tokens_path = PathBuf::from(args.required("tokens")?);
    let lens_path = args.optional("lens").map(PathBuf::from);
    let out = PathBuf::from(args.required("out")?);
    let tune_blocks: usize = args.number("tune_blocks", 64)?;
    report_output::claim(&out)?;
    let result = (|| -> Result<()> {
        // `evaluate` runs the artifact as saved: the load restores its transport snap by design.
        let model = StackModel::load(&model_dir, &Device::Cpu)?;
        let time = model.config.context;
        let tokens = read_tokens(&tokens_path, model.config.vocab_size)?;
        let lens = lens_path
            .as_ref()
            .map(|path| read_tokens(path, u16::MAX as usize + 1))
            .transpose()?;
        let blocks = (tokens.len() - 1) / time;
        if blocks <= tune_blocks {
            return Err(invalid("fewer blocks than tune_blocks"));
        }
        let mut block_nll = Vec::with_capacity(blocks);
        let mut block_bytes = Vec::with_capacity(blocks);
        for group in (0..blocks).collect::<Vec<_>>().chunks(16) {
            let mut ids = Vec::with_capacity(group.len() * time);
            let mut targets = Vec::with_capacity(group.len() * time);
            for &block in group {
                ids.extend_from_slice(&tokens[block * time..(block + 1) * time]);
                targets.extend_from_slice(&tokens[block * time + 1..(block + 1) * time + 1]);
            }
            let nll = model.target_nll(&ids, &targets, group.len(), time)?;
            for (index, _) in group.iter().enumerate() {
                block_nll.push(nll[index * time..(index + 1) * time].iter().sum::<f64>());
                block_bytes.push(lens.as_ref().map(|lens| {
                    targets[index * time..(index + 1) * time]
                        .iter()
                        .map(|&id| f64::from(lens[id as usize]))
                        .sum::<f64>()
                }));
            }
        }
        let summary = |range: std::ops::Range<usize>| {
            let targets = range.len() * time;
            let nll: f64 = block_nll[range.clone()].iter().sum();
            let bytes: Option<f64> = block_bytes[range].iter().copied().sum();
            json!({
                "blocks": targets / time, "targets": targets, "nll": nll / targets as f64,
                "bits_per_byte": bytes.map(|bytes| nll / std::f64::consts::LN_2 / bytes),
            })
        };
        fs::write(
            out.join("evaluation.json"),
            serde_json::to_vec_pretty(&json!({
                "schema": "uor-r4.geometric-stack-evaluation/1",
                "protocol": "consecutive blocks of 256 inputs and 256 shifted targets, fresh state per block",
                "model": identity(&model_dir.join("model.safetensors"))?,
                "config": model.config,
                "tokens": identity(&tokens_path)?,
                "lens": lens_path.as_ref().map(|p| identity(p)).transpose()?,
                "tune": summary(0..tune_blocks),
                "comparison": summary(tune_blocks..blocks),
                "full": summary(0..blocks),
                "block_nll": block_nll,
            }))?,
        )?;
        Ok(())
    })();
    finish(&out, result)
}

/// The integer serving artifact of a trained model (owner decision D10).
fn export_mode(arguments: &[String]) -> Result<()> {
    let args = Args::parse(
        arguments,
        &[
            "model",
            "out",
            "calibration",
            "calibration_windows",
            "calibration_time",
            "damp",
        ],
    )?;
    let model_dir = PathBuf::from(args.required("model")?);
    let out = PathBuf::from(args.required("out")?);
    let calibration_tokens = args.optional("calibration").map(PathBuf::from);
    let windows: usize = args.number("calibration_windows", 64)?;
    let damp: f64 = args.number("damp", 0.01)?;
    if windows == 0 || !damp.is_finite() || damp < 0.0 {
        return Err(invalid(
            "calibration_windows must be positive and damp >= 0",
        ));
    }
    // A model saved with a transport snap exports with the snap recorded;
    // a malformed record refuses the export here.
    let transport_snap = StackModel::saved_transport_snap(&model_dir)?;
    report_output::claim(&out)?;
    let result = (|| -> Result<()> {
        let started = Instant::now();
        // A model trained against a served representation (a `qat=true`
        // run's) exports only as that representation.
        let served = StackModel::saved_served_representation(&model_dir)?;
        check_export_representation(served.as_ref(), calibration_tokens.is_some())?;
        let mut model = StackModel::load(&model_dir, &Device::Cpu)?;
        // A selected or routed pointer head or a flock has no integer engine
        // yet: refuse before any calibration work.
        check_export_config(&model.config)?;
        if let Some(saved_served) = &served {
            if model.config.arch == StackArch::Geometric {
                let codec = codec_by_name(&saved_served.codec)?;
                model.set_served_representation(Some(codec))?;
            }
        }
        let time: usize = args.number("calibration_time", model.config.context)?;
        if time == 0 || time > model.config.context {
            return Err(invalid("calibration_time must be within the context"));
        }
        let weights = model_dir.join("model.safetensors");
        let calibration = match &calibration_tokens {
            Some(path) => {
                let tokens = read_tokens(path, model.config.vocab_size)?;
                Some(StackCalibration::collect(&model, &tokens, windows, time)?)
            }
            None => None,
        };
        let calibration_seconds = started.elapsed().as_secs_f64();
        let quantizer = match &calibration_tokens {
            Some(path) => json!({
                "method": "gptq",
                "calibration": identity(path)?,
                "calibration_windows": windows,
                "calibration_time": time,
                "damp": damp,
            }),
            None => {
                let method_name = export_quantizer_method(&model);
                json!({"method": method_name})
            }
        };
        let mut source = json!({
            "exporter": "geometric-stack export",
            "model": identity(&weights)?,
            "config": model.config,
            "executable": identity(&std::env::current_exe()?)?,
            "quantizer": quantizer,
        });
        if let Some(served) = &served {
            source["served_representation"] = json!({
                "codec": served.codec,
                "scope": format!(
                    "recorded in the model's config.json: the model was trained against this representation (quantization-aware training), which this export writes using codec '{}'",
                    served.codec
                ),
            });
        }
        let (bytes, report) = match model.config.arch {
            StackArch::Geometric => export_stack(
                &model,
                source.clone(),
                calibration.as_ref().map(|c| (c, damp)),
                transport_snap,
            )?,
            StackArch::Transformer => {
                let checkpoint = control_checkpoint(&model, sha256_file(&weights)?)?;
                let llama = calibration.as_ref().map(|c| c.llama()).transpose()?;
                export_llama(
                    &checkpoint,
                    model.config.context,
                    source.clone(),
                    llama.as_ref().map(|c| (c, damp)),
                    None,
                )?
            }
        };
        let artifact = out.join("model.lut");
        fs::write(&artifact, &bytes)?;
        fs::write(
            out.join("export.json"),
            serde_json::to_vec_pretty(&json!({
                "schema": "uor-r4.geometric-stack-export/1",
                "transport_snap": transport_snap.map(|s| s.record()),
                "source": source,
                "artifact": identity(&artifact)?,
                "quantization": report,
                "calibration_seconds": calibration_seconds,
                "seconds": started.elapsed().as_secs_f64(),
            }))?,
        )?;
        Ok(())
    })();
    finish(&out, result)
}

/// Either integer engine, behind one step function.
enum Engine {
    Stack(Box<uor_r4_lut::stack::StackModel>),
    Llama(Box<uor_r4_lut::engine::Model>),
}

/// An integer decoding session of either engine.
trait Stepper {
    /// Feed one id; the next-token logits at exponent -16.
    fn advance(&mut self, id: u32) -> Result<&[i32]>;
}

impl Stepper for uor_r4_lut::stack::StackSession<'_> {
    fn advance(&mut self, id: u32) -> Result<&[i32]> {
        self.step(id).map_err(lut)
    }
}

impl Stepper for uor_r4_lut::engine::Session<'_> {
    fn advance(&mut self, id: u32) -> Result<&[i32]> {
        self.step(id).map_err(lut)
    }
}

/// Feed `prompt`, then draw `new_tokens` ids with `sampler`.
fn continue_prompt(
    session: &mut impl Stepper,
    prompt: &[u32],
    new_tokens: usize,
    sampler: &mut uor_r4_lut::sampling::Sampler,
    exp: (&[u32], i32),
) -> Result<Vec<u32>> {
    let mut seen = prompt.to_vec();
    let mut logits = Vec::new();
    for &id in prompt {
        logits = session.advance(id)?.to_vec();
    }
    let mut generated = Vec::with_capacity(new_tokens);
    for step in 0..new_tokens {
        let next = sampler.sample(&logits, &seen, exp.0, exp.1).map_err(lut)?;
        generated.push(next);
        seen.push(next);
        if step + 1 < new_tokens {
            logits = session.advance(next)?.to_vec();
        }
    }
    Ok(generated)
}

impl Engine {
    fn load(bytes: Vec<u8>, threads: usize) -> Result<Self> {
        use uor_r4_lut::format::{is_stack_schema, schema_of, Artifact, StackArtifact, SCHEMA};
        let schema = schema_of(&bytes).map_err(lut)?;
        Ok(if is_stack_schema(&schema) {
            let mut model = uor_r4_lut::stack::StackModel::from_artifact(
                StackArtifact::parse(bytes).map_err(lut)?,
            )
            .map_err(lut)?;
            // Every user of this engine scores or samples the raw logits,
            // which are not a pointer model's distribution.
            if model.pointer().is_some() {
                return Err(invalid(
                    "the artifact has a pointer head, whose distribution is its mixture, not \
                     the raw logits this mode reads: decode it with uor-r4-stack generate or \
                     compare the engines with d11-evaluate",
                ));
            }
            model.set_threads(threads).map_err(lut)?;
            Self::Stack(Box::new(model))
        } else if schema == SCHEMA {
            let mut model =
                uor_r4_lut::engine::Model::from_artifact(Artifact::parse(bytes).map_err(lut)?)
                    .map_err(lut)?;
            model.set_threads(threads).map_err(lut)?;
            Self::Llama(Box::new(model))
        } else {
            return Err(invalid(format!("unknown artifact schema {schema}")));
        })
    }

    fn vocabulary(&self) -> usize {
        match self {
            Self::Stack(model) => model.shape().vocab,
            Self::Llama(model) => model.shape().vocab,
        }
    }

    fn context(&self) -> usize {
        match self {
            Self::Stack(model) => model.shape().context,
            Self::Llama(model) => model.shape().max_positions,
        }
    }

    fn sha256(&self) -> &str {
        match self {
            Self::Stack(model) => model.artifact_sha256(),
            Self::Llama(model) => model.artifact_sha256(),
        }
    }

    fn backend(&self) -> &'static str {
        match self {
            Self::Stack(model) => model.backend().name(),
            Self::Llama(model) => model.backend().name(),
        }
    }

    /// Logits (exponent -16) after each id of one window, from a fresh session.
    /// Returns the seconds spent inside the engine's steps, without `visit`.
    fn window(&self, ids: &[u32], mut visit: impl FnMut(usize, &[i32])) -> Result<f64> {
        fn run(
            session: &mut impl Stepper,
            ids: &[u32],
            visit: &mut impl FnMut(usize, &[i32]),
        ) -> Result<f64> {
            let mut seconds = 0f64;
            for (t, &id) in ids.iter().enumerate() {
                let clock = Instant::now();
                let logits = session.advance(id)?;
                seconds += clock.elapsed().as_secs_f64();
                visit(t, logits);
            }
            Ok(seconds)
        }
        match self {
            Self::Stack(model) => run(&mut model.session(), ids, &mut visit),
            Self::Llama(model) => run(&mut model.session(), ids, &mut visit),
        }
    }

    /// A continuation of `prompt` from a fresh session.
    fn generate(
        &self,
        prompt: &[u32],
        new_tokens: usize,
        sampler: &mut uor_r4_lut::sampling::Sampler,
    ) -> Result<Vec<u32>> {
        match self {
            Self::Stack(model) => continue_prompt(
                &mut model.session(),
                prompt,
                new_tokens,
                sampler,
                model.exp_table(),
            ),
            Self::Llama(model) => continue_prompt(
                &mut model.session(),
                prompt,
                new_tokens,
                sampler,
                model.exp_table(),
            ),
        }
    }
}

/// `-log softmax(logits)[target]` and the argmax, in f64 (evaluation only).
/// The NLL of `target` under a pointer mixture in Q30 (a zero probability is
/// floored at one quantum, `2^-30`, about 20.8 nats) and the greedy id (the
/// first maximum).
fn score_mixture(mixture: &[i32], target: usize) -> (f64, usize) {
    let mut best = 0usize;
    for (i, &v) in mixture.iter().enumerate() {
        if v > mixture[best] {
            best = i;
        }
    }
    let p = mixture.get(target).copied().unwrap_or(0).max(1);
    (30.0 * std::f64::consts::LN_2 - f64::from(p).ln(), best)
}

fn score_row(logits: impl Iterator<Item = f64> + Clone, target: usize) -> (f64, usize) {
    let (mut best, mut max) = (0usize, f64::NEG_INFINITY);
    for (i, v) in logits.clone().enumerate() {
        if v > max {
            (best, max) = (i, v);
        }
    }
    let total: f64 = logits.clone().map(|v| (v - max).exp()).sum();
    let at = logits.clone().nth(target).unwrap_or(f64::NEG_INFINITY);
    (max + total.ln() - at, best)
}

/// S1.0b: the development NLL cost of serving every transport quaternion as
/// the nearest of the 120 unit icosians (2I), on the windows `evaluate` uses.
/// Evaluation only: no training, no serving path.
fn snap_evaluate_mode(arguments: &[String]) -> Result<()> {
    let args = Args::parse(arguments, &["model", "valid", "windows", "out"])?;
    let model_dir = PathBuf::from(args.required("model")?);
    let valid_path = PathBuf::from(args.required("valid")?);
    let windows: usize = args.number("windows", 512)?;
    let out = PathBuf::from(args.required("out")?);
    if windows == 0 {
        return Err(invalid("windows must be positive"));
    }
    report_output::claim(&out)?;
    let result = (|| -> Result<()> {
        let started = Instant::now();
        // `load` restores a trained-in snap; this diagnostic's fused baseline
        // is the free transport, taken explicitly below.
        let mut model = StackModel::load(&model_dir, &Device::Cpu)?;
        // This mode scores the raw logits, which are not a pointer model's
        // distribution.
        check_raw_logit_evaluation(&model.config, "snap-evaluate")?;
        let valid = read_tokens(&valid_path, model.config.vocab_size)?;
        let time = model.config.context;
        if valid.len() <= time + windows {
            return Err(invalid("too few development tokens for the windows"));
        }
        let roots: Vec<[f32; 4]> = canonical_h4_roots()
            .iter()
            .map(|root| {
                let a = root.to_array();
                [a[0] as f32, a[1] as f32, a[2] as f32, a[3] as f32]
            })
            .collect();
        let stride = (valid.len() - time - 1) / windows;
        let starts: Vec<usize> = (0..windows).map(|window| window * stride).collect();
        let (mut fused_nll, mut snapped_nll, mut agree, mut seen) = (0f64, 0f64, 0usize, 0usize);
        let mut composed_max_abs = 0f64;
        for (index, group) in starts.chunks(16).enumerate() {
            let mut ids = Vec::with_capacity(group.len() * time);
            let mut targets = Vec::with_capacity(group.len() * time);
            for &start in group {
                ids.extend_from_slice(&valid[start..start + time]);
                targets.extend_from_slice(&valid[start + 1..start + time + 1]);
            }
            let rows = targets.len();
            let fused =
                model.with_unsnapped_transport(|free| free.forward(&ids, group.len(), time))?;
            let snapped = model.logits_with_transport(&ids, group.len(), time, Some(&roots))?;
            if index == 0 {
                // The composed reference must equal the fused forward, so the
                // snapped delta is due to snapping alone.
                let composed = model.logits_with_transport(&ids, group.len(), time, None)?;
                composed_max_abs = f64::from(
                    composed
                        .sub(&fused)?
                        .abs()?
                        .max_keepdim(1)?
                        .max(0)?
                        .to_vec1::<f32>()?[0],
                );
            }
            fused_nll +=
                f64::from(logits_cross_entropy(&fused, &targets, None)?.to_scalar::<f32>()?)
                    * rows as f64;
            snapped_nll +=
                f64::from(logits_cross_entropy(&snapped, &targets, None)?.to_scalar::<f32>()?)
                    * rows as f64;
            let fused_top = fused.argmax(1)?.to_vec1::<u32>()?;
            let snapped_top = snapped.argmax(1)?.to_vec1::<u32>()?;
            agree += fused_top
                .iter()
                .zip(&snapped_top)
                .filter(|(a, b)| a == b)
                .count();
            seen += rows;
        }
        let fused_mean = fused_nll / seen as f64;
        let snapped_mean = snapped_nll / seen as f64;
        let report = json!({
            "schema": "uor-r4.stack-transport-snap/1",
            "roadmap": "S1.0b (evaluation-only diagnostic)",
            "model": identity(&model_dir.join("model.safetensors"))?,
            "config": model.config,
            "valid": identity(&valid_path)?,
            "windows": windows,
            "targets": seen,
            "roots": "canonical_h4_roots: the 120 unit icosians of 2I",
            "trained_transport_snap": StackModel::saved_transport_snap(&model_dir)?,
            "fused_nll": fused_mean,
            "snapped_nll": snapped_mean,
            "snapped_minus_fused_nats": snapped_mean - fused_mean,
            "top1_agreement": agree as f64 / seen as f64,
            "composed_minus_fused_max_abs_logit_first_group": composed_max_abs,
            "executable": identity(&std::env::current_exe()?)?,
            "seconds": started.elapsed().as_secs_f64(),
        });
        println!("{}", serde_json::to_string_pretty(&report)?);
        fs::write(out.join("snap.json"), serde_json::to_vec_pretty(&report)?)?;
        Ok(())
    })();
    finish(&out, result)
}

/// S1.0c: where a stack artifact's representation gap comes from. The float
/// model with one group of tensors at a time replaced by the artifact's own
/// values (in f32, the float kernels), on the windows `evaluate` uses. Groups
/// keep a normalization with the matrices its gain is folded into. Evaluation
/// only.
fn rounding_attribution_mode(arguments: &[String]) -> Result<()> {
    let args = Args::parse(arguments, &["artifact", "model", "valid", "windows", "out"])?;
    let artifact_path = PathBuf::from(args.required("artifact")?);
    let model_dir = PathBuf::from(args.required("model")?);
    let valid_path = PathBuf::from(args.required("valid")?);
    let windows: usize = args.number("windows", 512)?;
    let out = PathBuf::from(args.required("out")?);
    if windows == 0 {
        return Err(invalid("windows must be positive"));
    }
    report_output::claim(&out)?;
    let result = (|| -> Result<()> {
        let started = Instant::now();
        let float = StackModel::load(&model_dir, &Device::Cpu)?;
        // This mode scores the raw logits, which are not a pointer model's
        // distribution.
        check_raw_logit_evaluation(&float.config, "rounding-attribution")?;
        // The D10 comparator computes the free transport; a snap-trained
        // model is served only by the multiplier-free engine.
        if let Some(snap) = float.transport_snap() {
            return Err(invalid(format!(
                "rounding attribution measures the D10 comparator, which computes the free \
                 transport; {} records the {} transport snap, which only the multiplier-free \
                 engine serves (use stack-snap-parity or d11-evaluate)",
                model_dir.display(),
                snap.name()
            )));
        }
        let artifact =
            uor_r4_lut::format::StackArtifact::parse(fs::read(&artifact_path)?).map_err(lut)?;
        let reference = stack_grid_reference(&float, &artifact)?;
        let valid = read_tokens(&valid_path, float.config.vocab_size)?;
        let time = float.config.context;
        if valid.len() <= time + windows {
            return Err(invalid("too few development tokens for the windows"));
        }
        // Every float tensor belongs to exactly one group.
        let group_of = |name: &str| -> Result<String> {
            if name == "embedding.weight" {
                return Ok("embed".into());
            }
            if name == "final_norm.weight" {
                return Ok("head".into());
            }
            let rest = name
                .strip_prefix("layers.")
                .ok_or_else(|| invalid(format!("no attribution group for {name}")))?;
            let (layer, part) = rest
                .split_once('.')
                .ok_or_else(|| invalid(format!("no attribution group for {name}")))?;
            let layer: usize = layer
                .parse()
                .map_err(|_| invalid(format!("no attribution group for {name}")))?;
            let kind = match part {
                "rec_norm.weight" | "rec.in.weight" | "rec.gate.weight" | "rec.out.weight"
                | "read_norm.weight" | "read.query.weight" | "read.key.weight"
                | "read.value.weight" | "read.null.weight" | "read.out.weight" => "mixer_maps",
                "rec.conv.weight" | "rec.conv.bias" | "rec.gate.bias" | "rec.decay"
                | "read.null.bias" | "read.age" | "read.log_beta" | "read.offset" => {
                    "mixer_scalars"
                }
                "mlp_norm.weight" | "mlp.gate.weight" | "mlp.up.weight" | "mlp.down.weight" => {
                    "mlp"
                }
                _ => return Err(invalid(format!("no attribution group for {name}"))),
            };
            Ok(format!("l{layer}.{kind}"))
        };
        let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for name in float.variables().keys() {
            groups
                .entry(group_of(name)?)
                .or_default()
                .push(name.clone());
        }
        let float_head = float.variables()["embedding.weight"].as_tensor().clone();
        let stride = (valid.len() - time - 1) / windows;
        let starts: Vec<usize> = (0..windows).map(|window| window * stride).collect();
        // The float model with the tensors of `selected` groups from the artifact.
        let score = |selected: &[&str]| -> Result<(f64, Vec<u32>)> {
            let hybrid = StackModel::load(&model_dir, &Device::Cpu)?;
            for group in selected {
                let names = groups
                    .get(*group)
                    .ok_or_else(|| invalid(format!("unknown group {group}")))?;
                for name in names {
                    hybrid.variables()[name].set(reference.model.variables()[name].as_tensor())?;
                }
            }
            let head = if selected.contains(&"head") {
                &reference.head
            } else {
                &float_head
            };
            let (mut nll, mut top) = (0f64, Vec::new());
            for group in starts.chunks(16) {
                let mut ids = Vec::with_capacity(group.len() * time);
                let mut targets = Vec::with_capacity(group.len() * time);
                for &start in group {
                    ids.extend_from_slice(&valid[start..start + time]);
                    targets.extend_from_slice(&valid[start + 1..start + time + 1]);
                }
                let logits = hybrid.hidden(&ids, group.len(), time)?.matmul(&head.t()?)?;
                nll +=
                    f64::from(logits_cross_entropy(&logits, &targets, None)?.to_scalar::<f32>()?)
                        * targets.len() as f64;
                top.extend(logits.argmax(1)?.to_vec1::<u32>()?);
            }
            Ok((nll / top.len() as f64, top))
        };
        let (float_nll, float_top) = score(&[])?;
        let names: Vec<String> = groups.keys().cloned().collect();
        let mut rows = Vec::new();
        let mut evaluate = |label: String, selected: Vec<&str>| -> Result<()> {
            let (nll, top) = score(&selected)?;
            let agree = top.iter().zip(&float_top).filter(|(a, b)| a == b).count();
            let row = json!({
                "group": label,
                "members": selected,
                "nll": nll,
                "minus_float_nats": nll - float_nll,
                "top1_agreement": agree as f64 / top.len() as f64,
            });
            eprintln!("{row}");
            rows.push(row);
            Ok(())
        };
        for name in &names {
            evaluate(name.clone(), vec![name.as_str()])?;
        }
        let of_kind = |suffix: &str| -> Vec<&str> {
            names
                .iter()
                .filter(|n| n.ends_with(suffix))
                .map(String::as_str)
                .collect()
        };
        let mut maps = of_kind(".mixer_maps");
        maps.extend(of_kind(".mlp"));
        maps.extend(["embed", "head"]);
        evaluate("all_maps".into(), maps)?;
        evaluate("all_scalars".into(), of_kind(".mixer_scalars"))?;
        evaluate("all".into(), names.iter().map(String::as_str).collect())?;
        let report = json!({
            "schema": "uor-r4.stack-rounding-attribution/1",
            "roadmap": "S1.0c (evaluation-only diagnostic)",
            "artifact": identity(&artifact_path)?,
            "model": identity(&model_dir.join("model.safetensors"))?,
            "config": float.config,
            "valid": identity(&valid_path)?,
            "windows": windows,
            "targets": float_top.len(),
            "float_nll": float_nll,
            "groups": groups,
            "rows": rows,
            "reading": "each row: the float model with only the listed groups from the artifact; maps are the 4-bit matrices (with the normalization whose gain they carry), scalars the grid-code and fixed-point per-channel values",
            "executable": identity(&std::env::current_exe()?)?,
            "seconds": started.elapsed().as_secs_f64(),
        });
        fs::write(
            out.join("attribution.json"),
            serde_json::to_vec_pretty(&report)?,
        )?;
        Ok(())
    })();
    finish(&out, result)
}

/// Integer serving on development windows, beside the float model on the same
/// windows when `model=` is given: evenly spaced windows (`train`'s rule), or
/// with `blocks=true` the retained evaluator's consecutive blocks with its
/// tune/comparison split. With `reference=true` (and `model=`), the artifact's
/// own values in float arithmetic are scored too, which splits the integer
/// gap into weight rounding and integer arithmetic.
fn lut_evaluate_mode(arguments: &[String]) -> Result<()> {
    let args = Args::parse(
        arguments,
        &[
            "artifact",
            "valid",
            "model",
            "windows",
            "blocks",
            "tune_blocks",
            "threads",
            "lens",
            "reference",
            "out",
        ],
    )?;
    let artifact_path = PathBuf::from(args.required("artifact")?);
    let valid_path = PathBuf::from(args.required("valid")?);
    let model_dir = args.optional("model").map(PathBuf::from);
    let lens_path = args.optional("lens").map(PathBuf::from);
    let windows: usize = args.number("windows", 64)?;
    let blocks: bool = args.number("blocks", false)?;
    let tune_blocks: usize = args.number("tune_blocks", 64)?;
    let threads: usize = args.number("threads", 1)?;
    let use_reference: bool = args.number("reference", false)?;
    if use_reference && model_dir.is_none() {
        return Err(invalid("reference=true needs model="));
    }
    let out = PathBuf::from(args.required("out")?);
    report_output::claim(&out)?;
    let result = (|| -> Result<()> {
        let bytes = fs::read(&artifact_path)?;
        let engine = Engine::load(bytes.clone(), threads)?;
        let time = engine.context();
        let valid = read_tokens(&valid_path, engine.vocabulary())?;
        let lens = lens_path
            .as_ref()
            .map(|path| read_tokens(path, u16::MAX as usize + 1))
            .transpose()?;
        let float = model_dir
            .as_ref()
            .map(|dir| StackModel::load(dir, &Device::Cpu))
            .transpose()?;
        if let Some(model) = &float {
            // The float side of this mode scores the raw logits, which are not
            // a pointer model's distribution.
            check_raw_logit_evaluation(&model.config, "lut-evaluate")?;
            if model.config.vocab_size != engine.vocabulary() || model.config.context != time {
                return Err(invalid("the float model and the artifact differ in shape"));
            }
            // The D10 comparator computes the free transport; a snap-trained
            // model is served only by the multiplier-free engine.
            if let Some(snap) = model.transport_snap() {
                return Err(invalid(format!(
                    "lut-evaluate measures the D10 comparator, which computes the free transport; \
                     the model records the {} transport snap, which only the multiplier-free \
                     engine serves (use stack-snap-parity or d11-evaluate)",
                    snap.name()
                )));
            }
        }
        let reference = match (&float, use_reference) {
            (Some(model), true) => Some(match &engine {
                Engine::Stack(_) => stack_grid_reference(
                    model,
                    &uor_r4_lut::format::StackArtifact::parse(bytes).map_err(lut)?,
                )?,
                Engine::Llama(_) => control_grid_reference(
                    model,
                    &uor_r4_lut::format::Artifact::parse(bytes).map_err(lut)?,
                )?,
            }),
            _ => None,
        };
        let starts: Vec<usize> = if blocks {
            let count = (valid.len() - 1) / time;
            if count <= tune_blocks {
                return Err(invalid("fewer blocks than tune_blocks"));
            }
            (0..count).map(|block| block * time).collect()
        } else {
            if windows == 0 || valid.len() <= time + windows {
                return Err(invalid("too few development tokens for the windows"));
            }
            let stride = (valid.len() - time - 1) / windows;
            (0..windows).map(|window| window * stride).collect()
        };
        // Per window: integer, float and reference NLL, and target bytes (sums
        // over targets).
        let mut sums: Vec<(f64, f64, f64, f64)> = Vec::with_capacity(starts.len());
        let (mut agree, mut step_seconds, mut loop_seconds) = (0usize, 0f64, 0f64);
        for &start in &starts {
            let ids = &valid[start..start + time];
            let next = &valid[start + 1..start + time + 1];
            let mut window_nll = 0f64;
            let mut integer_top = vec![0usize; time];
            let clock = Instant::now();
            step_seconds += engine.window(ids, |t, logits| {
                let (nll, top) = score_row(
                    logits.iter().map(|&v| f64::from(v) / 65536.0),
                    next[t] as usize,
                );
                window_nll += nll;
                integer_top[t] = top;
            })?;
            loop_seconds += clock.elapsed().as_secs_f64();
            let mut window_float = 0f64;
            if let Some(model) = &float {
                let logits = model.forward(ids, 1, time)?.to_vec2::<f32>()?;
                for (t, row) in logits.iter().enumerate() {
                    let (nll, top) = score_row(row.iter().map(|&v| f64::from(v)), next[t] as usize);
                    window_float += nll;
                    agree += usize::from(top == integer_top[t]);
                }
            }
            let mut window_reference = 0f64;
            if let Some(reference) = &reference {
                let logits = reference.logits(ids, 1, time)?.to_vec2::<f32>()?;
                for (t, row) in logits.iter().enumerate() {
                    window_reference +=
                        score_row(row.iter().map(|&v| f64::from(v)), next[t] as usize).0;
                }
            }
            let bytes = lens.as_ref().map_or(0.0, |lens| {
                next.iter()
                    .map(|&id| f64::from(lens[id as usize]))
                    .sum::<f64>()
            });
            sums.push((window_nll, window_float, bytes, window_reference));
        }
        let summary = |range: std::ops::Range<usize>| -> Value {
            let part = &sums[range];
            let targets = (part.len() * time) as f64;
            let (integer, float_sum, bytes, reference_sum) =
                part.iter().fold((0.0, 0.0, 0.0, 0.0), |a, s| {
                    (a.0 + s.0, a.1 + s.1, a.2 + s.2, a.3 + s.3)
                });
            let bits = |nll: f64| lens.as_ref().map(|_| nll / std::f64::consts::LN_2 / bytes);
            json!({
                "windows": part.len(), "targets": part.len() * time,
                "integer": {"nll": integer / targets, "bits_per_byte": bits(integer)},
                "float": float.as_ref().map(|_| json!({"nll": float_sum / targets, "bits_per_byte": bits(float_sum)})),
                "integer_minus_float_nll": float.as_ref().map(|_| (integer - float_sum) / targets),
                "reference": reference.as_ref().map(|_| json!({"nll": reference_sum / targets, "bits_per_byte": bits(reference_sum)})),
                "weight_rounding_nll": reference.as_ref().map(|_| (reference_sum - float_sum) / targets),
                "integer_arithmetic_nll": reference.as_ref().map(|_| (integer - reference_sum) / targets),
            })
        };
        let targets = starts.len() * time;
        let mut record = json!({
            "schema": "uor-r4.geometric-stack-lut-evaluation/4",
            "protocol": if blocks {
                "consecutive blocks of 256 inputs and 256 shifted targets (the retained evaluator's), one fresh integer session per block, position by position"
            } else {
                "evenly spaced windows of the development tokens (train's evaluation rule), one fresh integer session per window, position by position"
            },
            "artifact": identity(&artifact_path)?,
            "artifact_sha256": engine.sha256(),
            "float_model": model_dir.as_ref().map(|dir| identity(&dir.join("model.safetensors"))).transpose()?,
            "valid": identity(&valid_path)?,
            "windows": starts.len(),
            "targets": targets,
            "all": summary(0..starts.len()),
            "engine": {
                "step_seconds": step_seconds,
                "tokens_per_second": targets as f64 / step_seconds,
                "loop_seconds": loop_seconds,
                "loop_tokens_per_second": targets as f64 / loop_seconds,
                "scope": "step: the engine's steps only; loop: the steps plus scoring each position's logits in f64",
                "threads": threads,
                "backend": engine.backend(),
            },
            "top1_agreement": float.as_ref().map(|_| agree as f64 / targets as f64),
            "reference": reference.as_ref().map(|_| "the artifact's dequantized matrices, grid-code scalars and integer biases with unit norm gains and its own head, in f32 (the float model's kernels)"),
            "per_window": starts.iter().zip(&sums).map(|(start, s)| json!({
                "start": start, "integer_nll": s.0 / time as f64,
                "float_nll": float.as_ref().map(|_| s.1 / time as f64),
                "reference_nll": reference.as_ref().map(|_| s.3 / time as f64),
            })).collect::<Vec<_>>(),
        });
        if blocks {
            record["tune"] = summary(0..tune_blocks);
            record["comparison"] = summary(tune_blocks..starts.len());
        }
        fs::write(
            out.join("evaluation.json"),
            serde_json::to_vec_pretty(&record)?,
        )?;
        Ok(())
    })();
    finish(&out, result)
}

/// The multiplier-free (D11) stack engine, `uor_r4_integer::stack`, beside the
/// frozen D10 engine, `uor_r4_lut::stack`, on the same evenly spaced
/// development windows (`train`'s rule), each in a fresh session of both,
/// position by position: both engines' NLL, the largest absolute difference
/// between their logits (zero when the port is bit-identical), the positions
/// where any logit differs, their top-1 agreement and each engine's step time.
fn d11_evaluate_mode(arguments: &[String]) -> Result<()> {
    use uor_r4_integer::stack::IntegerStackModel;
    let args = Args::parse(
        arguments,
        &[
            "artifact", "valid", "out", "windows", "threads", "lens", "model",
        ],
    )?;
    let artifact_path = PathBuf::from(args.required("artifact")?);
    let valid_path = PathBuf::from(args.required("valid")?);
    let lens_path = args.optional("lens").map(PathBuf::from);
    let model_dir = args.optional("model").map(PathBuf::from);
    let windows: usize = args.number("windows", 8)?;
    let threads: usize = args.number("threads", 1)?;
    let out = PathBuf::from(args.required("out")?);
    report_output::claim(&out)?;
    let result = (|| -> Result<()> {
        let bytes = fs::read(&artifact_path)?;
        let clock = Instant::now();
        let mut d11 = IntegerStackModel::parse(&bytes).map_err(|e| invalid(e.to_string()))?;
        let d11_load_seconds = clock.elapsed().as_secs_f64();
        d11.set_threads(threads)
            .map_err(|e| invalid(e.to_string()))?;
        if let Some(snap) = d11.transport_snap() {
            // The D10 comparator serves the free transport and refuses this
            // artifact; compare against the snapped float forward instead.
            let snap = snap.clone();
            return d11_evaluate_snapped(
                &d11,
                &snap,
                model_dir
                    .as_ref()
                    .ok_or_else(|| invalid("a snapped artifact needs model= for its float side"))?,
                &valid_path,
                lens_path.as_ref(),
                windows,
                d11_load_seconds,
                &artifact_path,
                &out,
            );
        }
        let mut d10 = uor_r4_lut::stack::StackModel::from_artifact(
            uor_r4_lut::format::StackArtifact::parse(bytes).map_err(lut)?,
        )
        .map_err(lut)?;
        d10.set_threads(threads).map_err(lut)?;
        if d10.artifact_sha256() != d11.artifact_sha256() {
            return Err(invalid("the engines loaded different artifact bytes"));
        }
        let (vocab, time) = (d11.shape().vocab, d11.shape().context);
        let valid = read_tokens(&valid_path, vocab)?;
        let lens = lens_path
            .as_ref()
            .map(|path| read_tokens(path, u16::MAX as usize + 1))
            .transpose()?;
        if windows == 0 || valid.len() <= time + windows {
            return Err(invalid("too few development tokens for the windows"));
        }
        let stride = (valid.len() - time - 1) / windows;
        let starts: Vec<usize> = (0..windows).map(|window| window * stride).collect();
        let (mut s10, mut s11) = (d10.session(), d11.session());
        let (mut d10_seconds, mut d11_seconds) = (0f64, 0f64);
        let (mut max_difference, mut differing, mut agree) = (0i64, 0usize, 0usize);
        let mut first_difference: Option<Value> = None;
        // Per window: D11 NLL, D10 NLL and target bytes (sums over targets).
        let mut sums: Vec<(f64, f64, f64)> = Vec::with_capacity(starts.len());
        for &start in &starts {
            let ids = &valid[start..start + time];
            let next = &valid[start + 1..start + time + 1];
            s10.reset();
            s11.reset();
            let (mut nll11, mut nll10) = (0f64, 0f64);
            for (t, &id) in ids.iter().enumerate() {
                let clock = Instant::now();
                s11.step(id).map_err(|e| invalid(e.to_string()))?;
                d11_seconds += clock.elapsed().as_secs_f64();
                let clock = Instant::now();
                s10.step(id).map_err(lut)?;
                d10_seconds += clock.elapsed().as_secs_f64();
                let (logits11, logits10) = (s11.logits(), s10.logits());
                let mut position_max = 0i64;
                for (a, b) in logits11.iter().zip(logits10) {
                    position_max = position_max.max((i64::from(*a) - i64::from(*b)).abs());
                }
                // A pointer model's served distribution is its mixture (Q30):
                // compared as integers like the logits, and scored below.
                let (mixture11, mixture10) = (s11.mixture(), s10.mixture());
                if let (Some(m11), Some(m10)) = (mixture11, mixture10) {
                    for (a, b) in m11.iter().zip(m10) {
                        position_max = position_max.max((i64::from(*a) - i64::from(*b)).abs());
                    }
                }
                if position_max > 0 {
                    differing += 1;
                    if first_difference.is_none() {
                        first_difference = Some(json!({
                            "window_start": start, "position": t, "max_abs_difference": position_max,
                        }));
                    }
                }
                max_difference = max_difference.max(position_max);
                let target = next[t] as usize;
                let ((n11, top11), (n10, top10)) = match (mixture11, mixture10) {
                    (Some(m11), Some(m10)) => {
                        (score_mixture(m11, target), score_mixture(m10, target))
                    }
                    _ => (
                        score_row(logits11.iter().map(|&v| f64::from(v) / 65536.0), target),
                        score_row(logits10.iter().map(|&v| f64::from(v) / 65536.0), target),
                    ),
                };
                nll11 += n11;
                nll10 += n10;
                agree += usize::from(top11 == top10);
            }
            let bytes = lens.as_ref().map_or(0.0, |lens| {
                next.iter()
                    .map(|&id| f64::from(lens[id as usize]))
                    .sum::<f64>()
            });
            sums.push((nll11, nll10, bytes));
        }
        let targets = starts.len() * time;
        let (nll11, nll10, target_bytes) = sums
            .iter()
            .fold((0.0, 0.0, 0.0), |a, s| (a.0 + s.0, a.1 + s.1, a.2 + s.2));
        let bits = |nll: f64| {
            lens.as_ref()
                .map(|_| nll / std::f64::consts::LN_2 / target_bytes)
        };
        let record = json!({
            "schema": "uor-r4.geometric-stack-d11-evaluation/1",
            "protocol": "evenly spaced windows of the development tokens (train's evaluation rule), one fresh session of each engine per window, position by position; logits (and a pointer model's Q30 mixture) compared as integers",
            "scored_distribution": if d11.pointer().is_some() {
                "the pointer mixture (Q30, a zero probability floored at one quantum, 2^-30)"
            } else {
                "softmax of the logits"
            },
            "artifact": identity(&artifact_path)?,
            "artifact_sha256": d11.artifact_sha256(),
            "valid": identity(&valid_path)?,
            "windows": starts.len(),
            "targets": targets,
            "d11": {"nll": nll11 / targets as f64, "bits_per_byte": bits(nll11)},
            "d10": {"nll": nll10 / targets as f64, "bits_per_byte": bits(nll10)},
            "d11_minus_d10_nll": (nll11 - nll10) / targets as f64,
            "max_abs_logit_difference": max_difference,
            "positions_with_a_difference": differing,
            "first_difference": first_difference,
            "top1_agreement": agree as f64 / targets as f64,
            "weights_read_per_token": d11.weights_per_token(),
            "engine": {
                "d11_step_seconds": d11_seconds,
                "d11_tokens_per_second": targets as f64 / d11_seconds,
                "d11_threads": d11.threads(),
                "d11_load_seconds": d11_load_seconds,
                "d10_step_seconds": d10_seconds,
                "d10_tokens_per_second": targets as f64 / d10_seconds,
                "d10_threads": threads,
                "d10_backend": d10.backend().name(),
                "scope": "each engine's step calls only, timed separately and interleaved per position",
            },
            "per_window": starts.iter().zip(&sums).map(|(start, s)| json!({
                "start": start, "d11_nll": s.0 / time as f64, "d10_nll": s.1 / time as f64,
            })).collect::<Vec<_>>(),
        });
        fs::write(
            out.join("evaluation.json"),
            serde_json::to_vec_pretty(&record)?,
        )?;
        Ok(())
    })();
    finish(&out, result)
}

/// The snapped-artifact arm of [`d11_evaluate_mode`]: the D11 engine against
/// the float model's snapped forward (the artifact records the snap, so the
/// D10 comparator's free transport is not the model that was trained), on the
/// same evenly spaced windows, position by position: both sides' NLL, the
/// largest absolute logit gap (integer quanta and nats) and top-1 agreement.
#[allow(clippy::too_many_arguments)]
fn d11_evaluate_snapped(
    d11: &uor_r4_integer::stack::IntegerStackModel,
    snap: &uor_r4_integer::stack::StackTransportSnap,
    model_dir: &std::path::Path,
    valid_path: &std::path::Path,
    lens_path: Option<&PathBuf>,
    windows: usize,
    d11_load_seconds: f64,
    artifact_path: &std::path::Path,
    out: &std::path::Path,
) -> Result<()> {
    // The comparator's raw logits are scored: a pointer model is refused.
    let mut float = load_raw_logit_comparator(model_dir, "d11-evaluate")?;
    float.set_transport_snap(Some(match snap.name.as_str() {
        "icosian" => TransportSnap::Icosian,
        other => return Err(invalid(format!("unknown transport snap {other}"))),
    }))?;
    let (vocab, time) = (d11.shape().vocab, d11.shape().context);
    if float.config.vocab_size != vocab || float.config.context != time {
        return Err(invalid("the float model and the artifact differ in shape"));
    }
    let valid = read_tokens(valid_path, vocab)?;
    let lens = lens_path
        .map(|path| read_tokens(path, u16::MAX as usize + 1))
        .transpose()?;
    if windows == 0 || valid.len() <= time + windows {
        return Err(invalid("too few development tokens for the windows"));
    }
    let stride = (valid.len() - time - 1) / windows;
    let starts: Vec<usize> = (0..windows).map(|window| window * stride).collect();
    let mut session = d11.session();
    let (mut d11_seconds, mut float_seconds) = (0f64, 0f64);
    let (mut max_gap, mut agree) = (0i64, 0usize);
    let mut sums: Vec<(f64, f64, f64)> = Vec::with_capacity(starts.len());
    for &start in &starts {
        let ids = &valid[start..start + time];
        let next = &valid[start + 1..start + time + 1];
        session.reset();
        let clock = Instant::now();
        let float_logits = float.forward(ids, 1, time)?.to_vec2::<f32>()?;
        float_seconds += clock.elapsed().as_secs_f64();
        let (mut nll11, mut nllf) = (0f64, 0f64);
        for (t, &id) in ids.iter().enumerate() {
            let clock = Instant::now();
            let logits11 = session.step(id).map_err(|e| invalid(e.to_string()))?;
            d11_seconds += clock.elapsed().as_secs_f64();
            let (n11, top11) = score_row(
                logits11.iter().map(|&v| f64::from(v) / 65536.0),
                next[t] as usize,
            );
            let (nf, topf) = score_row(
                float_logits[t].iter().map(|&v| f64::from(v)),
                next[t] as usize,
            );
            for (&a, &b) in logits11.iter().zip(&float_logits[t]) {
                max_gap =
                    max_gap.max((i64::from(a) - (f64::from(b) * 65536.0).round() as i64).abs());
            }
            agree += usize::from(top11 == topf);
            nll11 += n11;
            nllf += nf;
        }
        let bytes = lens.as_ref().map_or(0.0, |lens| {
            next.iter().map(|&id| f64::from(lens[id as usize])).sum()
        });
        sums.push((nll11, nllf, bytes));
    }
    let targets = starts.len() * time;
    let (nll11, nllf, target_bytes) = sums
        .iter()
        .fold((0.0, 0.0, 0.0), |a, s| (a.0 + s.0, a.1 + s.1, a.2 + s.2));
    let bits = |nll: f64| {
        lens.as_ref()
            .map(|_| nll / std::f64::consts::LN_2 / target_bytes)
    };
    let record = json!({
        "schema": "uor-r4.geometric-stack-d11-evaluation/1",
        "protocol": "evenly spaced windows of the development tokens (train's evaluation rule), one fresh D11 session per window, position by position; the comparator is the float model's snapped forward, not the free-transport D10 engine",
        "comparator": "float-snapped",
        "transport_snap": {"name": snap.name, "roots": snap.roots, "roots_sha256": snap.roots_sha256},
        "artifact": identity(artifact_path)?,
        "artifact_sha256": d11.artifact_sha256(),
        "float_model": identity(&model_dir.join("model.safetensors"))?,
        "valid": identity(valid_path)?,
        "windows": starts.len(),
        "targets": targets,
        "d11": {"nll": nll11 / targets as f64, "bits_per_byte": bits(nll11)},
        "float": {"nll": nllf / targets as f64, "bits_per_byte": bits(nllf)},
        "d11_minus_float_nll": (nll11 - nllf) / targets as f64,
        "max_abs_logit_gap_quanta": max_gap,
        "max_abs_logit_gap_nats": max_gap as f64 / 65536.0,
        "top1_agreement": agree as f64 / targets as f64,
        "weights_read_per_token": d11.weights_per_token(),
        "engine": {
            "d11_step_seconds": d11_seconds,
            "d11_tokens_per_second": targets as f64 / d11_seconds,
            "d11_threads": d11.threads(),
            "d11_load_seconds": d11_load_seconds,
            "float_forward_seconds": float_seconds,
            "scope": "the D11 step calls only, timed separately from the float forward",
        },
        "per_window": starts.iter().zip(&sums).map(|(start, s)| json!({
            "start": start, "d11_nll": s.0 / time as f64, "float_nll": s.1 / time as f64,
        })).collect::<Vec<_>>(),
    });
    fs::write(
        out.join("evaluation.json"),
        serde_json::to_vec_pretty(&record)?,
    )?;
    Ok(())
}

/// Continuations generated by an integer engine, greedy and sampled, from
/// evenly spaced development prompts or from `prompt=` text.
fn lut_sample_mode(arguments: &[String]) -> Result<()> {
    let args = Args::parse(
        arguments,
        &[
            "artifact",
            "valid",
            "merges",
            "tokenizer",
            "prompt",
            "out",
            "prompts",
            "prompt_tokens",
            "sample_tokens",
            "temperature",
            "top_k",
            "top_p",
            "seed",
            "threads",
        ],
    )?;
    let artifact_path = PathBuf::from(args.required("artifact")?);
    let valid_path = args.optional("valid").map(PathBuf::from);
    let merges = args.optional("merges").map(PathBuf::from);
    let tokenizer = args.optional("tokenizer").map(PathBuf::from);
    let prompt_text = args.optional("prompt");
    let out = PathBuf::from(args.required("out")?);
    let prompts: usize = args.number("prompts", 3)?;
    let prompt_tokens: usize = args.number("prompt_tokens", 64)?;
    let sample_tokens: usize = args.number("sample_tokens", 128)?;
    let temperature: f64 = args.number("temperature", 0.8)?;
    let top_k: usize = args.number("top_k", 40)?;
    let top_p: f64 = args.number("top_p", 1.0)?;
    let seed: u64 = args.number("seed", 1)?;
    let threads: usize = args.number("threads", 1)?;
    report_output::claim(&out)?;
    let result = (|| -> Result<()> {
        use uor_r4_lut::sampling::{Sampler, SamplingSettings};
        let engine = Engine::load(fs::read(&artifact_path)?, threads)?;
        let decoder = Decoder::load(merges.as_deref(), tokenizer.as_deref())?;
        let prompt_ids: Vec<(Option<usize>, Vec<u32>)> = match (&prompt_text, &valid_path) {
            (Some(text), _) => {
                let decoder = decoder
                    .as_ref()
                    .ok_or_else(|| invalid("a text prompt needs merges= or tokenizer="))?;
                vec![(None, decoder.encode(text))]
            }
            (None, Some(path)) => {
                let valid = read_tokens(path, engine.vocabulary())?;
                if prompts == 0 || valid.len() < prompt_tokens + prompts {
                    return Err(invalid("too few development tokens for the prompts"));
                }
                let stride = (valid.len() - prompt_tokens) / prompts;
                (0..prompts)
                    .map(|index| {
                        let start = index * stride;
                        (Some(start), valid[start..start + prompt_tokens].to_vec())
                    })
                    .collect()
            }
            (None, None) => return Err(invalid("give prompt= text or valid= prompts")),
        };
        let greedy_settings = SamplingSettings::default();
        let sampled_settings =
            SamplingSettings::from_decimal(temperature, top_k, top_p, 0.0).map_err(lut)?;
        let text = |ids: &[u32]| decoder.as_ref().map(|decoder| decoder.decode(ids));
        let (mut rows, mut generated, mut seconds) = (Vec::new(), 0usize, 0f64);
        for (index, (start, prompt)) in prompt_ids.iter().enumerate() {
            if prompt.is_empty() || prompt.len() + sample_tokens > engine.context() {
                return Err(invalid("the prompt and continuation exceed the context"));
            }
            let clock = Instant::now();
            let greedy = engine.generate(
                prompt,
                sample_tokens,
                &mut Sampler::new(greedy_settings, seed),
            )?;
            let sampled = engine.generate(
                prompt,
                sample_tokens,
                &mut Sampler::new(sampled_settings, seed.wrapping_add(index as u64)),
            )?;
            seconds += clock.elapsed().as_secs_f64();
            generated += 2 * (prompt.len() + sample_tokens);
            rows.push(json!({
                "prompt_start": start, "prompt": text(prompt), "prompt_ids": prompt,
                "greedy": text(&greedy), "sampled": text(&sampled),
                "greedy_ids": greedy, "sampled_ids": sampled,
            }));
        }
        fs::write(
            out.join("samples.json"),
            serde_json::to_vec_pretty(&json!({
                "schema": "uor-r4.geometric-stack-lut-samples/1",
                "artifact": identity(&artifact_path)?,
                "artifact_sha256": engine.sha256(),
                "settings": {"temperature": temperature, "top_k": top_k, "top_p": top_p, "seed": seed},
                "engine": {"positions": generated, "seconds": seconds,
                    "positions_per_second": generated as f64 / seconds,
                    "scope": "prompt reading and generation: the engine's steps and integer sampling, greedy and sampled",
                    "threads": threads, "backend": engine.backend()},
                "rows": rows,
            }))?,
        )?;
        Ok(())
    })();
    finish(&out, result)
}

/// Settings of `dialogue-train`.
struct DialogueSettings {
    tokenizer: PathBuf,
    /// Token store, response mask and manifest of the training and
    /// development splits.
    train: [PathBuf; 3],
    dev: [PathBuf; 3],
    init: Option<PathBuf>,
    /// Train with the served representation (`qat=true`), as `train` does.
    qat: bool,
    /// Train with the transport snapped (`transport_snap=`), as `train` does.
    transport_snap: Option<TransportSnap>,
    /// `select=`, `pointer=`, `pointer_score=`, `pointer_select=` and
    /// `pointer_route=` as given (the A1 read mechanisms and the prime route).
    select: Option<String>,
    pointer: Option<String>,
    pointer_score: Option<String>,
    pointer_select: Option<String>,
    pointer_route: Option<String>,
    policy: PrefixPolicy,
    data_seed: u64,
    steps: usize,
    batch: usize,
    lr: f64,
    warmup: usize,
    min_lr: f64,
    weight_decay: f64,
    clip: f64,
    eval_every: usize,
    dev_seed: u64,
    dev_per_source: usize,
    checkpoint_every: usize,
    resume: Option<PathBuf>,
    max_seconds: f64,
    requests: Option<PathBuf>,
    max_new_tokens: usize,
    /// `protocol=1|2`: the literal-role dialogue version of both corpora
    /// (their assistant markers locate the scored responses).
    protocol: u8,
    /// `device=cpu|metal|cuda` (default cpu). A Metal or CUDA run needs the
    /// `metal` or `cuda` feature; ops without a GPU kernel run on host copies.
    device: Device,
}

impl DialogueSettings {
    fn record(&self) -> Value {
        let mut record = json!({
            "tokenizer": self.tokenizer, "train": self.train, "dev": self.dev,
            "init": self.init,
            "qat": self.qat, "qat_codec": self.qat.then(|| qat_codec().name().to_owned()),
            "transport_snap": self.transport_snap,
            "policy": self.policy, "data_seed": self.data_seed,
            "steps": self.steps, "batch": self.batch, "lr": self.lr, "warmup": self.warmup,
            "min_lr": self.min_lr, "weight_decay": self.weight_decay, "clip": self.clip,
            "eval_every": self.eval_every, "dev_seed": self.dev_seed,
            "dev_per_source": self.dev_per_source, "checkpoint_every": self.checkpoint_every,
            "requests": self.requests, "max_new_tokens": self.max_new_tokens,
        });
        if self.protocol != 1 {
            record["protocol"] = json!(self.protocol);
        }
        // As given on the command line; only when given, so other runs'
        // records are unchanged. The configuration (`config`) has the result.
        if let Some(select) = &self.select {
            record["select"] = json!(select);
        }
        if let Some(pointer) = &self.pointer {
            record["pointer"] = json!(pointer);
        }
        if let Some(score) = &self.pointer_score {
            record["pointer_score"] = json!(score);
        }
        if let Some(select) = &self.pointer_select {
            record["pointer_select"] = json!(select);
        }
        if let Some(route) = &self.pointer_route {
            record["pointer_route"] = json!(route);
        }
        record
    }

    /// Settings, input contents and executable a resumed run must share with
    /// its parent, so one run's updates all come from the same build. A run
    /// with `qat=true` or a transport snap also shares that, as in `train`;
    /// other runs' lineage is unchanged, so their earlier checkpoints still resume.
    fn lineage(&self, config: &StackConfig) -> Result<Value> {
        let content = |path: &Path| -> Result<Value> {
            Ok(json!({"bytes": fs::metadata(path)?.len(), "sha256": sha256_file(path)?}))
        };
        let split = |paths: &[PathBuf; 3]| -> Result<Vec<Value>> {
            paths.iter().map(|p| content(p)).collect()
        };
        let mut lineage = json!({
            "config": config, "policy": self.policy, "data_seed": self.data_seed,
            "steps": self.steps, "batch": self.batch, "lr": self.lr, "warmup": self.warmup,
            "min_lr": self.min_lr, "weight_decay": self.weight_decay, "clip": self.clip,
            "eval_every": self.eval_every, "dev_seed": self.dev_seed,
            "dev_per_source": self.dev_per_source,
            "executable": content(&std::env::current_exe()?)?,
            "inputs": {
                "tokenizer": content(&self.tokenizer)?, "train": split(&self.train)?,
                "dev": split(&self.dev)?,
                "init": self.init.as_deref().map(|p| content(&p.join("model.safetensors"))).transpose()?,
            },
        });
        if self.qat {
            lineage["qat"] = json!({"codec": qat_codec().name()});
        }
        if self.protocol != 1 {
            lineage["protocol"] = json!(self.protocol);
        }
        if let Some(snap) = self.transport_snap {
            lineage["transport_snap"] = snap.record();
        }
        Ok(lineage)
    }
}

/// A model to continue must have learned the same token ids. The run that
/// wrote it records its inputs: in `ROOT/report.json` beside `ROOT/model`, or,
/// for a checkpoint's model (`ROOT/checkpoint/model`), in the lineage of the
/// checkpoint's `state.json`. The model is refused unless one of them records
/// this tokenizer.json and no lab merges.
fn check_init_tokenizer(model: &Path, tokenizer: &Path) -> Result<()> {
    let root = model
        .parent()
        .ok_or_else(|| invalid("init= has no parent directory"))?;
    let inputs = if root.join("report.json").exists() {
        let report: Value = serde_json::from_slice(&fs::read(root.join("report.json"))?)?;
        report["inputs"].clone()
    } else if root.join("state.json").exists() {
        let state: Value = serde_json::from_slice(&fs::read(root.join("state.json"))?)?;
        state["lineage"]["inputs"].clone()
    } else {
        return Err(invalid(
            "init= needs the report.json or checkpoint state.json of the run that wrote it",
        ));
    };
    if !inputs["merges"].is_null() {
        return Err(invalid(
            "init= was trained on the lab BPE's ids, not this tokenizer's",
        ));
    }
    match inputs["tokenizer"]["sha256"].as_str() {
        Some(recorded) if recorded == sha256_file(tokenizer)? => Ok(()),
        Some(_) => Err(invalid("init= was trained with a different tokenizer.json")),
        None => Err(invalid(
            "init= does not record the tokenizer.json its ids came from",
        )),
    }
}

/// Response learning on the prepared literal-role dialogue corpus with the
/// retained study's episodes (`uor_r4_training::stack_dialogue`).
fn dialogue_train_mode(arguments: &[String]) -> Result<()> {
    let args = Args::parse(
        arguments,
        &[
            "out",
            "tokenizer",
            "train_tokens",
            "train_mask",
            "train_manifest",
            "dev_tokens",
            "dev_mask",
            "dev_manifest",
            "init",
            "arch",
            "pattern",
            "read",
            "rotation",
            "rotation_group",
            "width",
            "heads",
            "layers",
            "mlp",
            "stack_mlp",
            "context",
            "seed",
            "policy",
            "data_seed",
            "steps",
            "batch",
            "lr",
            "warmup",
            "min_lr",
            "weight_decay",
            "clip",
            "eval_every",
            "dev_seed",
            "dev_per_source",
            "checkpoint_every",
            "resume",
            "max_seconds",
            "requests",
            "max_new_tokens",
            "device",
            "qat",
            "transport_snap",
            "select",
            "pointer",
            "pointer_score",
            "pointer_select",
            "pointer_route",
            "protocol",
            "tf32",
        ],
    )?;
    // Validate the A1 options before anything is claimed or loaded.
    select_arg(&args)?;
    let pointer_requested = pointer_args(&args)?;
    let path = |key: &str| -> Result<PathBuf> { Ok(PathBuf::from(args.required(key)?)) };
    let settings = DialogueSettings {
        tokenizer: path("tokenizer")?,
        train: [
            path("train_tokens")?,
            path("train_mask")?,
            path("train_manifest")?,
        ],
        dev: [
            path("dev_tokens")?,
            path("dev_mask")?,
            path("dev_manifest")?,
        ],
        init: args.optional("init").map(PathBuf::from),
        qat: qat_flag(&args)?,
        transport_snap: transport_snap_arg(&args)?,
        select: args.optional("select"),
        pointer: args.optional("pointer"),
        pointer_score: args.optional("pointer_score"),
        pointer_select: args.optional("pointer_select"),
        pointer_route: args.optional("pointer_route"),
        policy: PrefixPolicy::parse(args.optional("policy").as_deref())?,
        data_seed: args.number("data_seed", 1)?,
        steps: args.number("steps", 1024)?,
        batch: args.number("batch", 16)?,
        lr: args.number("lr", 0.001)?,
        warmup: args.number("warmup", 50)?,
        min_lr: args.number("min_lr", 0.1)?,
        weight_decay: args.number("weight_decay", 0.1)?,
        clip: args.number("clip", 1.0)?,
        eval_every: args.number("eval_every", 128)?,
        dev_seed: args.number("dev_seed", 1)?,
        dev_per_source: args.number("dev_per_source", 32)?,
        checkpoint_every: args.number("checkpoint_every", 128)?,
        resume: args.optional("resume").map(PathBuf::from),
        max_seconds: args.number("max_seconds", f64::INFINITY)?,
        requests: args.optional("requests").map(PathBuf::from),
        max_new_tokens: args.number("max_new_tokens", 32)?,
        protocol: match args.optional("protocol").as_deref() {
            None | Some("1") => 1,
            Some("2") => 2,
            Some(other) => return Err(invalid(format!("invalid protocol={other} (1 or 2)"))),
        },
        device: device_arg(&args)?,
    };
    if settings.steps == 0
        || !(1..=64).contains(&settings.batch)
        || settings.eval_every == 0
        || !(1..=MAX_NEW_TOKENS).contains(&settings.max_new_tokens)
        || settings.lr.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater)
        || !(0.0..=1.0).contains(&settings.min_lr)
    {
        return Err(invalid(format!(
            "steps, eval_every and lr must be positive, batch 1..64, max_new_tokens \
             1..{MAX_NEW_TOKENS}, min_lr in [0, 1]"
        )));
    }
    // The snap needs a rotating geometric stack: `init=`'s, or the shape
    // options' (whose vocabulary the corpus sets later).
    if let Some(snap) = settings.transport_snap {
        let config = match &settings.init {
            Some(directory) => {
                serde_json::from_slice::<StackConfig>(&fs::read(directory.join("config.json"))?)?
            }
            None => stack_config(&args, None)?,
        };
        snap.check(&config)?;
    }
    // The pointer has no served representation: refuse it under qat before
    // anything is claimed, whether the run asks for one or `init=` has one.
    if settings.qat {
        let saved_pointer = match &settings.init {
            Some(directory) => {
                serde_json::from_slice::<StackConfig>(&fs::read(directory.join("config.json"))?)?
                    .pointer
            }
            None => None,
        };
        if saved_pointer.is_some() || matches!(pointer_requested.head, Some(Some(_))) {
            return Err(pointer_qat_refusal());
        }
    }
    // The selection the pointer trains with: the run's `pointer_select=`, or
    // else an `init=` head's own (a resume shares the run's configuration).
    let trained_select = match pointer_requested.select {
        Some(select) => select,
        None => match &settings.init {
            Some(directory) => {
                serde_json::from_slice::<StackConfig>(&fs::read(directory.join("config.json"))?)?
                    .pointer
                    .and_then(|pointer| pointer.select)
            }
            None => None,
        },
    };
    refuse_trained_single_source(trained_select)?;
    let out = PathBuf::from(args.required("out")?);
    report_output::claim(&out)?;
    let result = dialogue_train(&settings, &args, &out);
    finish(&out, result)
}

/// The configuration of a `dialogue-train` run from `init=`'s saved model with
/// the run's options applied. `select=` replaces the saved flock and
/// `pointer_select=` the saved pointer's own selection (no weights change by
/// either). `pointer=DIM` and `pointer_score=` must agree with a saved head,
/// which cannot be removed. On a model saved without one, `pointer=DIM` asks
/// for a new head (the returned flag), scoring by `pointer_score=` (default
/// dot), whose weights are drawn from `seed=` (default: the saved model's
/// seed); that seed is recorded in the head's `init_seed`, so it belongs to
/// the configuration a resume must share.
fn init_extended_config(args: &Args, saved: &StackConfig) -> Result<(StackConfig, bool)> {
    let mut config = saved.clone();
    if let Some(select) = select_arg(args)? {
        config.select = select;
    }
    // `context=` may only grow the saved context (`StackModel::extend_context`).
    let context: usize = args.number("context", saved.context)?;
    if context < saved.context {
        return Err(invalid(format!(
            "context={context} is shorter than the saved model's {}; a context can only grow",
            saved.context
        )));
    }
    config.context = context;
    let asked = pointer_args(args)?;
    let mut added = false;
    match saved.pointer {
        Some(existing) => {
            if asked.head == Some(None) {
                return Err(invalid(
                    "pointer=none cannot remove the saved model's pointer head",
                ));
            }
            if let Some(Some(dim)) = asked.head {
                if dim != existing.dim {
                    return Err(invalid(format!(
                        "pointer={dim} conflicts with the saved model's pointer head of width {}",
                        existing.dim
                    )));
                }
            }
            if let Some(score) = asked.score {
                if score != existing.score {
                    return Err(invalid(format!(
                        "pointer_score={} conflicts with the saved model's pointer head, which \
                         scores {}",
                        score_name(score),
                        score_name(existing.score)
                    )));
                }
            }
            if let Some(select) = asked.select {
                config.pointer = Some(PointerConfig { select, ..existing });
            }
            if let Some(route) = asked.route {
                config.pointer = config
                    .pointer
                    .map(|pointer| PointerConfig { route, ..pointer });
            }
        }
        None => match asked.new_head(Some(args.number("seed", saved.seed)?)) {
            Some(head) => {
                added = true;
                config.pointer = Some(head);
            }
            None => asked.refuse_without_head()?,
        },
    }
    config.validate()?;
    Ok((config, added))
}

/// A resume must carry the pointer head's init seed of the run it continues:
/// the seed is in the lineage's configuration, and a different one is refused
/// here with both seeds named (the general lineage check would name only the
/// configuration). A checkpoint whose head was built with its model, or that
/// has none, records no seed, and so must the resume.
fn check_resume_pointer_seed(checkpoint: &Path, config: &StackConfig) -> Result<()> {
    let state: Value = serde_json::from_slice(&fs::read(checkpoint.join("state.json"))?)?;
    let saved = &state["lineage"]["config"]["pointer"]["init_seed"];
    let asked = json!(config.pointer.and_then(|pointer| pointer.init_seed));
    if *saved != asked {
        return Err(invalid(format!(
            "the resume's pointer init seed ({asked}) differs from the checkpoint's ({saved}): \
             a head added to init= keeps the seed its weights were drawn from, so resume with the \
             same seed= (or none, when the run used the saved model's)"
        )));
    }
    Ok(())
}

/// The roots the snapped forward selects over the development panel, as
/// [`development`] batches it (full original prefixes, 16 responses at a
/// time), counted over each episode's real input positions.
fn panel_transport_usage(
    model: &StackModel,
    dev: &EpisodeIndex<'_>,
    panel: &[usize],
) -> Result<Option<TransportUsage>> {
    let Some(snap) = model.transport_snap() else {
        return Ok(None);
    };
    let mut usage = TransportUsage::new(snap, &model.config);
    for chunk in panel.chunks(16) {
        let episodes = dev.materialize(chunk, PrefixPolicy::FullPrefix)?;
        let trimmed = trim(&episodes);
        let lengths: Vec<usize> = episodes
            .rows
            .iter()
            .map(|row| row.counts.real_input_positions.min(trimmed.time))
            .collect();
        usage.add(&model.transport_usage(
            &trimmed.inputs,
            episodes.batch,
            trimmed.time,
            Some(&lengths),
        )?)?;
    }
    Ok(Some(usage))
}

/// In a transport-snap run: the development panel with the transport free
/// (the same weights), and the icosian usage over the panel.
fn panel_transport(
    model: &mut StackModel,
    dev: &EpisodeIndex<'_>,
    panel: &[usize],
) -> Result<Option<(Value, TransportUsage)>> {
    let Some(usage) = panel_transport_usage(model, dev, panel)? else {
        return Ok(None);
    };
    let unsnapped = model.with_unsnapped_transport(|model| development(model, dev, panel, 16))?;
    Ok(Some((unsnapped, usage)))
}

fn dialogue_train(s: &DialogueSettings, args: &Args, out: &Path) -> Result<()> {
    let device = s.device.clone();
    let tokenizer =
        uor_r4_tokenizer::ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(&s.tokenizer)?)
            .ok_or_else(|| invalid("unreadable tokenizer.json"))?;
    let train_split = DialogueSplit::load(&s.train[0], &s.train[1], &s.train[2])?;
    let dev_split = DialogueSplit::load(&s.dev[0], &s.dev[1], &s.dev[2])?;
    let vocab = train_split.vocab_size();
    if dev_split.vocab_size() != vocab {
        return Err(invalid("the splits declare different vocabularies"));
    }
    // After `init=`, `select=`, `context=` and the pointer options extend the
    // saved model; a pointer added to a model saved without one draws its
    // weights from `seed=` (the saved seed by default), which its `init_seed`
    // records.
    let (config, pointer_added) = match &s.init {
        Some(directory) => {
            check_init_tokenizer(directory, &s.tokenizer)?;
            let saved = StackModel::load(directory, &device)?.config;
            init_extended_config(args, &saved)?
        }
        None => (stack_config(args, Some(vocab))?, false),
    };
    let pointer_seed: u64 = if pointer_added {
        config
            .pointer
            .and_then(|pointer| pointer.init_seed)
            .ok_or_else(|| invalid("an added pointer head records its init seed"))?
    } else {
        config.seed
    };
    if config.context < EPISODE_CONTEXT || config.vocab_size != vocab {
        return Err(invalid(
            "the model's context must be at least the development panel's 256 and its \
             vocabulary the corpus's",
        ));
    }
    // Training episodes fill the model's context; the development panel stays
    // the retained study's 256-ID panel, so its scores stay comparable.
    let (protocol, contract) = episode_contract_for(&tokenizer, vocab, config.context, s.protocol)?;
    let train = train_split.index_for(contract, s.policy)?;
    let (_, dev_contract) = episode_contract_for(&tokenizer, vocab, EPISODE_CONTEXT, s.protocol)?;
    let dev = dev_split.index(dev_contract)?;
    let panel = dialogue_development::select(&dev, s.dev_seed, s.dev_per_source)?;
    let requests = s.requests.as_deref().map(load_requests).transpose()?;
    if s.qat {
        check_qat_config(&config)?;
    }
    let encoder = protocol
        .bind(&tokenizer)
        .map_err(|e| invalid(e.to_string()))?;
    // The panel is answered after training; check it before any update.
    if let Some(requests) = &requests {
        check_panel(&encoder, requests, config.context, s.max_new_tokens)?;
    }
    let lineage = s.lineage(&config)?;
    let (mut model, mut optimizer, mut progress, resumed_from) = match &s.resume {
        None => {
            let model = match &s.init {
                Some(directory) => {
                    let mut model = StackModel::load(directory, &device)?;
                    model.extend_context(config.context)?;
                    model.set_select(config.select)?;
                    if let Some(pointer) = config.pointer {
                        // A saved head keeps its weights, its recorded seed and
                        // (unless the run replaces them) its selection and route.
                        model.add_pointer(pointer, pointer_seed)?;
                        model.set_pointer_route(None)?;
                        model.set_pointer_select(pointer.select)?;
                        model.set_pointer_route(pointer.route)?;
                    }
                    if model.config != config {
                        return Err(invalid(
                            "the extended init= model differs from its configuration",
                        ));
                    }
                    model
                }
                None => StackModel::new(config.clone(), &device)?,
            };
            // The optimizer is built after any pointer head is added.
            let optimizer = StackAdamW::new(&model, s.weight_decay, s.clip)?;
            let progress = Progress {
                step: 0,
                rng: Rng(0),
                curve: Vec::new(),
                train_seconds: 0.0,
            };
            (model, optimizer, progress, None)
        }
        Some(checkpoint) => {
            // The pointer head's init seed is part of the lineage; a different
            // one is refused, naming both.
            check_resume_pointer_seed(checkpoint, &config)?;
            let (model, optimizer, progress, state) =
                load_checkpoint(checkpoint, &lineage, &device)?;
            // And the model it resumes carries it: the report gives the
            // resumed head's seed, not the one this invocation was given.
            if model.config.pointer.and_then(|pointer| pointer.init_seed)
                != config.pointer.and_then(|pointer| pointer.init_seed)
            {
                return Err(invalid(
                    "the checkpoint's model records a different pointer init seed than its lineage",
                ));
            }
            (model, optimizer, progress, Some(state))
        }
    };
    // After `init=` or a resume alike: the requested settings replace any mode the load restored.
    if s.qat {
        model.set_served_representation(Some(qat_codec()))?;
    }
    model.set_transport_snap(s.transport_snap)?;
    eprintln!(
        "{:?} pattern {} read {:?}: {} parameters; {} training and {} development responses",
        model.config.arch,
        model.config.pattern,
        model.config.read,
        model.parameter_count(),
        train.episodes().len(),
        panel.len(),
    );
    if model.config.select.is_some() || model.config.pointer.is_some() {
        eprintln!(
            "select {:?}, pointer {:?}{}",
            model.config.select,
            model.config.pointer,
            if pointer_added {
                format!(" (new weights from seed {pointer_seed})")
            } else {
                String::new()
            }
        );
    }
    // In a QAT run each development score is of the served representation,
    // with the float weights' on the same panel beside it; in a transport-snap
    // run the panel is also scored unsnapped, with the icosian usage.
    let score = |model: &StackModel| development(model, &dev, &panel, 16);
    let (initial, initial_transport) = if progress.step == 0 {
        (
            Some(in_both_modes(&mut model, &score)?),
            panel_transport(&mut model, &dev, &panel)?,
        )
    } else {
        (None, None)
    };
    if let Some((unsnapped, usage)) = &initial_transport {
        eprintln!(
            "step 0 unsnapped response NLL {} (first four {}); transport usage {}",
            unsnapped["response_mean_nll"],
            unsnapped["first_four_response_targets_mean_nll"],
            usage.summary()
        );
    }
    if let Some((served, _)) = &initial {
        if let Some(pointer) = served.get("pointer") {
            eprintln!(
                "step 0 dev response NLL {}; pointer mean gate {} hit rate {} reachable {}",
                served["response_mean_nll"],
                pointer["mean_gate"],
                pointer["pointer_hit_rate"],
                pointer["target_reachable_rate"]
            );
        }
    }
    // Visits of earlier steps follow from the stateless sampler.
    let mut visits = (0usize, 0usize);
    for step in 0..progress.step {
        for id in train.sample_ids(s.data_seed, step as u64, s.batch)? {
            let episode = &train.episodes()[id];
            visits.0 += episode.response_end - episode.response_start;
        }
    }
    let (mut window_loss, mut stopped_early) = ((0f64, 0usize), false);
    // The served representation's work inside this process's updates.
    let mut served_in_steps = ServedStatistics::default();
    // This process's updates' seconds.
    let mut step_seconds = Vec::new();
    let started = Instant::now();
    while progress.step < s.steps {
        let lr = cosine_rate(s.lr, s.warmup, s.min_lr, s.steps, progress.step);
        let clock = Instant::now();
        let ids = train.sample_ids(s.data_seed, progress.step as u64, s.batch)?;
        let batch = train.materialize(&ids, s.policy)?;
        let trimmed = trim(&batch);
        let served_before = model.served_statistics()?;
        let loss = model.weighted_loss(
            &trimmed.inputs,
            &trimmed.targets,
            &trimmed.weights,
            batch.batch,
            trimmed.time,
        )?;
        let value = f64::from(loss.to_scalar::<f32>()?);
        if !value.is_finite() {
            return Err(invalid(format!("nonfinite loss at step {}", progress.step)));
        }
        let grads = loss.backward()?;
        let grad_norm = optimizer.update(&model, &grads, lr)?;
        let seconds = clock.elapsed().as_secs_f64();
        step_seconds.push(seconds);
        progress.train_seconds += seconds;
        add_served_work(
            &mut served_in_steps,
            served_before,
            model.served_statistics()?,
        );
        progress.step += 1;
        visits.0 += batch.counts.supervised_target_count;
        visits.1 += batch.batch * trimmed.time;
        window_loss.0 += value;
        window_loss.1 += 1;
        if progress.step % s.eval_every == 0 || progress.step == s.steps {
            let (dev_report, dev_float) = in_both_modes(&mut model, &score)?;
            let transport = panel_transport(&mut model, &dev, &panel)?;
            let mut point = json!({
                "step": progress.step, "lr": lr,
                "train_response_nll": window_loss.0 / window_loss.1.max(1) as f64,
                "dev_response_nll": dev_report["response_mean_nll"],
                "dev_first_four_nll": dev_report["first_four_response_targets_mean_nll"],
                "grad_norm": grad_norm, "supervised_target_visits": visits.0,
                "train_seconds": progress.train_seconds,
            });
            if let Some(float) = &dev_float {
                point["dev_float_response_nll"] = float["response_mean_nll"].clone();
                point["dev_float_first_four_nll"] =
                    float["first_four_response_targets_mean_nll"].clone();
            }
            // A pointer head's gate and hit rate on the scored dev targets.
            if let Some(pointer) = dev_report.get("pointer") {
                point["dev_pointer_mean_gate"] = pointer["mean_gate"].clone();
                point["dev_pointer_hit_rate"] = pointer["pointer_hit_rate"].clone();
                point["dev_pointer_reachable_rate"] = pointer["target_reachable_rate"].clone();
            }
            if let Some((unsnapped, usage)) = &transport {
                point["dev_unsnapped_response_nll"] = unsnapped["response_mean_nll"].clone();
                point["dev_unsnapped_first_four_nll"] =
                    unsnapped["first_four_response_targets_mean_nll"].clone();
                point["transport_usage"] = usage.summary();
            }
            eprintln!("{point}");
            progress.curve.push(point);
            window_loss = (0.0, 0);
        }
        if s.checkpoint_every > 0
            && progress.step % s.checkpoint_every == 0
            && progress.step < s.steps
        {
            save_checkpoint(out, &model, &optimizer, &progress, &lineage)?;
        }
        if started.elapsed().as_secs_f64() > s.max_seconds {
            stopped_early = true;
            save_checkpoint(out, &model, &optimizer, &progress, &lineage)?;
            break;
        }
    }
    // In served mode the saved configuration records the codec.
    model.save(&out.join("model"))?;
    if !stopped_early {
        let _ = fs::remove_dir_all(out.join("checkpoint"));
    }
    let (final_development, final_float) = in_both_modes(&mut model, &score)?;
    let final_transport = panel_transport(&mut model, &dev, &panel)?;
    let (initial, initial_float) = match initial {
        Some((served, float)) => (Some(served), float),
        None => (None, None),
    };
    let replies_from = match (s.qat, s.transport_snap.is_some()) {
        (true, true) => {
            "the served representation (the round-to-nearest export's values in the float \
             forward) with its transport snapped"
        }
        (true, false) => {
            "the served representation (the round-to-nearest export's values in the float forward)"
        }
        (false, true) => "the float model with its transport snapped",
        (false, false) => "the float model",
    };
    let replies = match &requests {
        Some(requests) => {
            let clock = Instant::now();
            let mut panel = reply_panel(
                &encoder,
                &protocol,
                requests,
                model.config.context,
                s.max_new_tokens,
                &|ids| tokenizer.decode(ids),
                &mut |history, cap| greedy_reply(&model, history, cap, protocol.eos_id),
            )?;
            panel["seconds"] = json!(clock.elapsed().as_secs_f64());
            panel["decoding"] = json!(format!("{replies_from}, greedy"));
            Some(panel)
        }
        None => None,
    };
    let mut report = json!({
        "schema": "uor-r4.geometric-stack-dialogue-run/1",
        "settings": s.record(),
        "tf32": tf32_enabled(),
        "config": model.config,
        "parameters": model.parameter_count(),
        "protocol": protocol,
        "sampler": uor_r4_training::dialogue_episodes::SAMPLER_ID,
        "development_selection": uor_r4_training::dialogue_development::SELECTION_ID,
        "train_population": train.population(),
        "completed_steps": progress.step,
        "stopped_early": stopped_early,
        "resumed_from": resumed_from,
        "supervised_target_visits": visits.0,
        "tensor_positions_this_process": visits.1,
        "train_seconds": progress.train_seconds,
        "step_seconds": step_timing(&step_seconds),
        "threads": std::env::var("RAYON_NUM_THREADS").ok(),
        "curve": progress.curve,
        "initial_development": initial,
        "final_development": final_development,
        "replies": replies,
        "inputs": {
            "tokenizer": identity(&s.tokenizer)?,
            "train": s.train.iter().map(|p| identity(p)).collect::<Result<Vec<_>>>()?,
            "dev": s.dev.iter().map(|p| identity(p)).collect::<Result<Vec<_>>>()?,
            "init": s.init.as_ref().map(|p| identity(&p.join("model.safetensors"))).transpose()?,
            "requests": s.requests.as_ref().map(|p| identity(p)).transpose()?,
        },
        "executable": identity(&std::env::current_exe()?)?,
        "model_sha256": sha256_file(&out.join("model").join("model.safetensors"))?,
        "scope": format!("Offline response learning of the geometric stack on the prepared dialogue corpus; development NLL on the fixed source-stratified panel under full original prefixes; replies from {replies_from}. No integer serving or quality qualification."),
    });
    if let Some(codec) = model.served_codec() {
        report["qat"] = json!({
            "codec": codec.name(),
            "representation": "the forward pass read the round-to-nearest export's values (4-bit maps with the norm gains folded in, grid-code scalars, fixed-point biases, age tables and offsets) with straight-through gradients; `dev_response_nll`, `dev_first_four_nll`, `initial_development` and `final_development` score that representation, `dev_float_*`, `initial_development_float` and `final_development_float` the float weights on the same panel; replies come from the served representation; the saved model's config.json records the codec (`served_representation`), and `export` writes it by rounding to nearest only",
            "initial_development_float": initial_float,
            "final_development_float": final_float,
            "served_work_in_updates": served_in_steps,
            "served_work_total": model.served_statistics()?,
        });
    }
    if let Some(snap) = model.transport_snap() {
        let mut section = snap.record();
        section["scope"] = json!(TRANSPORT_SNAP_SCOPE);
        section["saved_record"] = json!(StackModel::saved_transport_snap(&out.join("model"))?);
        let (initial_unsnapped, initial_usage) = match &initial_transport {
            Some((unsnapped, usage)) => (Some(unsnapped), Some(usage.summary())),
            None => (None, None),
        };
        section["initial_development_unsnapped"] = json!(initial_unsnapped);
        section["initial_transport_usage"] = json!(initial_usage);
        if let Some((unsnapped, usage)) = &final_transport {
            section["final_development_unsnapped"] = unsnapped.clone();
            section["final_transport_usage"] = usage.summary();
        }
        report["transport_snap"] = section;
    }
    if model.config.select.is_some() || model.config.pointer.is_some() {
        let final_pointer = final_development.get("pointer").cloned();
        let seed_source = if args.optional("seed").is_some() {
            "seed="
        } else {
            "the saved model's seed"
        };
        let pointer_parameters: usize = model
            .variables()
            .iter()
            .filter(|(name, _)| name.starts_with("pointer."))
            .map(|(_, var)| var.elem_count())
            .sum();
        report["flock_and_pointer"] = json!({
            "select": model.config.select,
            "pointer": model.config.pointer,
            "pointer_score": model.config.pointer.map(|pointer| score_name(pointer.score)),
            "pointer_select": model.config.pointer.and_then(|pointer| pointer.select),
            "pointer_route": model.config.pointer.and_then(|pointer| pointer.route),
            "pointer_head_added_to_init": pointer_added,
            // The seed the model itself records: a resumed run reports the
            // head's seed, which the resume verified, not one it was given.
            "pointer_init_seed": model.config.pointer.and_then(|pointer| pointer.init_seed),
            "pointer_init_seed_source": pointer_added.then_some(seed_source),
            "pointer_parameters": pointer_parameters,
            "final_pointer_diagnostics": final_pointer,
            "scope": "Flock selection of the reads (sink at position 0, last WINDOW positions and the K best-scoring other sources per read row, by the shared crate::flock selector; unkept sources weigh and receive exactly 0; the NoRead slot stays outside the selection) applies to every read of the model and never to the pointer. The pointer head scores each source of the window with its own score (`pointer_score`: dot, or the fused read's Lorentz form with a learned scale) and softmaxes over the sources its own selection keeps (`pointer_select`: none keeps all, top:1 is the single-source pointer), copies the input tokens at the attended positions, and a gate g = sigmoid(w.h + b) (b starts at -2) mixes that with the ordinary distribution; when no kept source holds a target the mixture is (1 - g) softmax alone, with no floor. `response_mean_nll`, the losses and the greedy replies are the mixture's. `pointer` diagnostics are over the scored dev targets. Neither a flock nor a pointer selection or route has a D11 port; `export` writes a pointer that keeps every source (both integer engines serve its mixture), and `qat=true` refuses a pointer. Offline float training only; not a served or quality result.",
        });
    }
    fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    Ok(())
}

/// The next id from a step's scores and the ids so far: the sampler over a
/// plain artifact's logits, or the greedy argmax over a pointer artifact's
/// mixture.
type Pick<'a> = dyn FnMut(&[i32], &[u32]) -> Result<u32> + 'a;

/// Integer replies of either engine: feed what the session has not yet
/// consumed of the history (a history that does not extend it restarts the
/// session), then draw ids until EOS or the cap.
struct IntegerChat<S> {
    session: S,
    fed: Vec<u32>,
    logits: Vec<i32>,
}

impl<S: Stepper> IntegerChat<S> {
    fn reply(
        &mut self,
        new_session: &dyn Fn() -> S,
        history: &[u32],
        cap: usize,
        eos: u32,
        pick: &mut Pick<'_>,
    ) -> Result<Reply> {
        if !history.starts_with(&self.fed) || history.len() == self.fed.len() {
            self.session = new_session();
            self.fed.clear();
        }
        for &id in &history[self.fed.len()..] {
            self.logits = self.session.advance(id)?.to_vec();
            self.fed.push(id);
        }
        let mut seen = history.to_vec();
        let mut ids = Vec::with_capacity(cap);
        for step in 0..cap {
            let next = pick(&self.logits, &seen)?;
            ids.push(next);
            seen.push(next);
            if let Some(reply) = Reply::stop(&ids, eos) {
                return Ok(reply);
            }
            if step + 1 < cap {
                self.logits = self.session.advance(next)?.to_vec();
                self.fed.push(next);
            }
        }
        Ok(Reply {
            ids,
            eos: false,
            cycle: None,
        })
    }
}

/// Chat with an integer artifact under the literal-role protocol: replies to
/// a request panel (`requests=`), or turns read from standard input, one user
/// message per line (`/reset` starts a new conversation).
fn lut_chat_mode(arguments: &[String]) -> Result<()> {
    let args = Args::parse(
        arguments,
        &[
            "artifact",
            "tokenizer",
            "out",
            "requests",
            "max_new_tokens",
            "temperature",
            "top_k",
            "top_p",
            "seed",
            "threads",
            "oracle",
            "engine",
            "protocol",
        ],
    )?;
    let engine_choice = args
        .optional("engine")
        .map(|name| MixtureEngine::parse(&name))
        .transpose()?;
    let version: Option<u8> = match args.optional("protocol").as_deref() {
        None => None,
        Some("1") => Some(1),
        Some("2") => Some(2),
        Some(other) => return Err(invalid(format!("invalid protocol={other} (1 or 2)"))),
    };
    // A pointer artifact decodes greedily over its mixture; sampling options
    // name a distribution it does not serve.
    let sampling_given: Vec<&str> = ["temperature", "top_k", "top_p", "seed"]
        .into_iter()
        .filter(|key| args.optional(key).is_some())
        .collect();
    if engine_choice.is_some() {
        refuse_mixture_sampling(&sampling_given)?;
    }
    let artifact_path = PathBuf::from(args.required("artifact")?);
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let requests_path = args.optional("requests").map(PathBuf::from);
    let oracle_path = args.optional("oracle").map(PathBuf::from);
    let out = PathBuf::from(args.required("out")?);
    let max_new_tokens: usize = args.number("max_new_tokens", 32)?;
    let temperature: f64 = args.number("temperature", 0.0)?;
    let top_k: usize = args.number("top_k", 40)?;
    let top_p: f64 = args.number("top_p", 1.0)?;
    let seed: u64 = args.number("seed", 1)?;
    let threads: usize = args.number("threads", 1)?;
    if !(1..=MAX_NEW_TOKENS).contains(&max_new_tokens) {
        return Err(invalid(format!(
            "max_new_tokens must be 1..{MAX_NEW_TOKENS}"
        )));
    }
    let requests = requests_path
        .as_deref()
        .map(load_requests)
        .transpose()?
        .map(|requests| match oracle_path.as_deref() {
            None => Ok(requests),
            Some(path) => apply_oracle_turns(requests, path),
        })
        .transpose()?;
    report_output::claim(&out)?;
    let result = (|| -> Result<()> {
        use uor_r4_lut::sampling::{Sampler, SamplingSettings};
        let bytes = fs::read(&artifact_path)?;
        let tokenizer = uor_r4_tokenizer::ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(
            &tokenizer_path,
        )?)
        .ok_or_else(|| invalid("unreadable tokenizer.json"))?;
        if uor_r4_lut::format::schema_of(&bytes).map_err(lut)?
            == uor_r4_lut::format::STACK_POINTER_SCHEMA
        {
            refuse_mixture_sampling(&sampling_given)?;
            let engine = engine_choice.unwrap_or(MixtureEngine::D11);
            let protocol =
                DialogueProtocol::literal_roles_version(&tokenizer, version.unwrap_or(2))
                    .map_err(|e| invalid(e.to_string()))?;
            let chat = mixture_chat(
                bytes,
                engine,
                threads,
                &tokenizer,
                &protocol,
                requests.as_deref(),
                max_new_tokens,
            )?;
            fs::write(
                out.join("chat.json"),
                serde_json::to_vec_pretty(&json!({
                    "schema": "uor-r4.geometric-stack-lut-chat/2",
                    "artifact": identity(&artifact_path)?,
                    "artifact_sha256": chat.artifact_sha256,
                    "tokenizer": identity(&tokenizer_path)?,
                    "protocol": protocol,
                    "requests": requests_path.as_ref().map(|p| identity(p)).transpose()?,
                    "settings": {"decoding": MIXTURE_DECODING, "max_new_tokens": max_new_tokens},
                    "engine": {"name": engine.name(), "backend": chat.backend,
                        "threads": chat.threads, "generated_positions": chat.positions,
                        "seconds": chat.seconds,
                        "ids_per_second": if chat.seconds > 0.0 {
                            json!(chat.positions as f64 / chat.seconds) } else { Value::Null }},
                    "record": chat.record,
                }))?,
            )?;
            return chat.failure.map_or(Ok(()), Err);
        }
        if engine_choice.is_some() {
            return Err(invalid(
                "engine= chooses the server of a pointer artifact's mixture; this artifact has \
                 no pointer head and is served by the D10 engine",
            ));
        }
        let engine = Engine::load(bytes, threads)?;
        let (protocol, _) = episode_contract_for(
            &tokenizer,
            engine.vocabulary(),
            EPISODE_CONTEXT,
            version.unwrap_or(1),
        )?;
        let encoder = protocol
            .bind(&tokenizer)
            .map_err(|e| invalid(e.to_string()))?;
        let settings = if temperature == 0.0 {
            SamplingSettings::default()
        } else {
            SamplingSettings::from_decimal(temperature, top_k, top_p, 0.0).map_err(lut)?
        };
        let mut sampler = Sampler::new(settings, seed);
        let decode = |ids: &[u32]| tokenizer.decode(ids);
        let clock = Instant::now();
        let mut positions = 0usize;
        // The plain path records no per-reply costs (its report is unchanged).
        let mut costs = Vec::new();
        let (record, failure) = match &engine {
            Engine::Stack(model) => {
                let exp = model.exp_table();
                chat_with(
                    &|| model.session(),
                    engine.context(),
                    &encoder,
                    &protocol,
                    requests.as_deref(),
                    max_new_tokens,
                    &decode,
                    &mut |logits, seen| sampler.sample(logits, seen, exp.0, exp.1).map_err(lut),
                    &mut positions,
                    &mut costs,
                )?
            }
            Engine::Llama(model) => {
                let exp = model.exp_table();
                chat_with(
                    &|| model.session(),
                    engine.context(),
                    &encoder,
                    &protocol,
                    requests.as_deref(),
                    max_new_tokens,
                    &decode,
                    &mut |logits, seen| sampler.sample(logits, seen, exp.0, exp.1).map_err(lut),
                    &mut positions,
                    &mut costs,
                )?
            }
        };
        let seconds = clock.elapsed().as_secs_f64();
        fs::write(
            out.join("chat.json"),
            serde_json::to_vec_pretty(&json!({
                "schema": "uor-r4.geometric-stack-lut-chat/1",
                "artifact": identity(&artifact_path)?,
                "artifact_sha256": engine.sha256(),
                "tokenizer": identity(&tokenizer_path)?,
                "protocol": protocol,
                "requests": requests_path.as_ref().map(|p| identity(p)).transpose()?,
                "settings": {"temperature": temperature, "top_k": top_k, "top_p": top_p,
                    "seed": seed, "max_new_tokens": max_new_tokens},
                "engine": {"backend": engine.backend(), "threads": threads,
                    "generated_positions": positions, "seconds": seconds},
                "record": record,
            }))?,
        )?;
        failure.map_or(Ok(()), Err)
    })();
    finish(&out, result)
}

/// How `lut-chat` decodes a pointer artifact.
const MIXTURE_DECODING: &str =
    "greedy over the pointer mixture (Q30), ties to the lower id; no sampling";

/// The integer engine that serves a pointer artifact's mixture in `lut-chat`:
/// the multiplier-free D11 session (the served path, the default) or the D10
/// comparator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MixtureEngine {
    D11,
    D10,
}

impl MixtureEngine {
    fn parse(text: &str) -> Result<Self> {
        match text {
            "d11" => Ok(Self::D11),
            "d10" => Ok(Self::D10),
            other => Err(invalid(format!("invalid engine={other} (d11 or d10)"))),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::D11 => "d11",
            Self::D10 => "d10",
        }
    }
}

/// Refuse sampling options for a pointer artifact, which `lut-chat` decodes
/// greedily over its mixture.
fn refuse_mixture_sampling(given: &[&str]) -> Result<()> {
    if given.is_empty() {
        return Ok(());
    }
    Err(invalid(format!(
        "a pointer artifact chats greedily over its mixture: {} not supported (omit them)",
        given.join(", ")
    )))
}

/// A D11 session whose step returns the pointer mixture.
struct D11Mixture<'m>(uor_r4_integer::stack::IntegerStackSession<'m>);

impl Stepper for D11Mixture<'_> {
    fn advance(&mut self, id: u32) -> Result<&[i32]> {
        self.0.step(id).map_err(|e| invalid(e.to_string()))?;
        Ok(self.0.next_token_scores())
    }
}

/// A D10 session whose step returns the pointer mixture.
struct D10Mixture<'m>(uor_r4_lut::stack::StackSession<'m>);

impl Stepper for D10Mixture<'_> {
    fn advance(&mut self, id: u32) -> Result<&[i32]> {
        self.0.step(id).map_err(lut)?;
        Ok(self.0.next_token_scores())
    }
}

/// What a pointer artifact's conversation produced.
struct MixtureChat {
    record: Value,
    /// The error that ended an interactive conversation, if one did.
    failure: Option<TrainingError>,
    artifact_sha256: String,
    backend: String,
    threads: Option<usize>,
    positions: usize,
    seconds: f64,
}

/// Chat with a pointer artifact through `engine`: greedy over the mixture,
/// prompts and stops as [`reply_panel`] (the float model's `chat-grade`
/// path) has them. A panel's record carries each reply's cost.
fn mixture_chat(
    bytes: Vec<u8>,
    engine: MixtureEngine,
    threads: usize,
    tokenizer: &uor_r4_tokenizer::ByteBpeTokenizer,
    protocol: &DialogueProtocol,
    requests: Option<&[uor_r4_training::stack_dialogue::Request]>,
    max_new_tokens: usize,
) -> Result<MixtureChat> {
    if !(1..=MAX_NEW_TOKENS).contains(&max_new_tokens) {
        return Err(invalid(format!(
            "max_new_tokens must be 1..{MAX_NEW_TOKENS}"
        )));
    }
    let encoder = protocol
        .bind(tokenizer)
        .map_err(|e| invalid(e.to_string()))?;
    let decode = |ids: &[u32]| tokenizer.decode(ids);
    let mut pick =
        |scores: &[i32], _: &[u32]| Ok(uor_r4_integer::stack::stack_argmax(scores) as u32);
    let mut positions = 0usize;
    let mut costs = Vec::new();
    let no_head = || invalid("the artifact's schema names a pointer head the engine did not load");
    let (mut record, failure, artifact_sha256, backend, threads, seconds) = match engine {
        MixtureEngine::D11 => {
            let mut model = uor_r4_integer::stack::IntegerStackModel::parse(&bytes)
                .map_err(|e| invalid(e.to_string()))?;
            model.pointer().ok_or_else(no_head)?;
            model
                .set_threads(threads)
                .map_err(|e| invalid(e.to_string()))?;
            let clock = Instant::now();
            let (record, failure) = chat_with(
                &|| D11Mixture(model.session()),
                model.shape().context,
                &encoder,
                protocol,
                requests,
                max_new_tokens,
                &decode,
                &mut pick,
                &mut positions,
                &mut costs,
            )?;
            let seconds = clock.elapsed().as_secs_f64();
            (
                record,
                failure,
                model.artifact_sha256().to_owned(),
                "d11 multiplier-free scalar".to_owned(),
                Some(model.threads()),
                seconds,
            )
        }
        MixtureEngine::D10 => {
            let mut model = uor_r4_lut::stack::StackModel::from_artifact(
                uor_r4_lut::format::StackArtifact::parse(bytes).map_err(lut)?,
            )
            .map_err(lut)?;
            model.pointer().ok_or_else(no_head)?;
            model.set_threads(threads).map_err(lut)?;
            let clock = Instant::now();
            let (record, failure) = chat_with(
                &|| D10Mixture(model.session()),
                model.shape().context,
                &encoder,
                protocol,
                requests,
                max_new_tokens,
                &decode,
                &mut pick,
                &mut positions,
                &mut costs,
            )?;
            let seconds = clock.elapsed().as_secs_f64();
            (
                record,
                failure,
                model.artifact_sha256().to_owned(),
                model.backend().name().to_owned(),
                Some(threads),
                seconds,
            )
        }
    };
    if requests.is_some() {
        annotate_turn_costs(&mut record, &costs)?;
    }
    Ok(MixtureChat {
        record,
        failure,
        artifact_sha256,
        backend,
        threads,
        positions,
        seconds,
    })
}

/// Replies to a request panel, or one interactive conversation. The record
/// comes back with the error that ended an interactive conversation, if one
/// did, so its transcript is still written.
#[allow(clippy::too_many_arguments)]
/// Prepend one **oracle** turn per request, from a JSON object of
/// `request id -> text`, so the value a memory row stores is present in the
/// history as an explicit statement of the same relation.
///
/// This is the labelled upper bound for the value path: the model is given a
/// statement to read from, so a failure cannot be blamed on the store not
/// having the value. A turn is inserted rather than a token sequence so that
/// the armoury's own role markers, separators and masking are used unchanged,
/// and the row's real turns follow it in their original order.
fn apply_oracle_turns(
    mut requests: Vec<uor_r4_training::stack_dialogue::Request>,
    path: &Path,
) -> Result<Vec<uor_r4_training::stack_dialogue::Request>> {
    let text = fs::read_to_string(path)?;
    let mapping: Value = serde_json::from_str(&text)
        .map_err(|e| invalid(format!("oracle file is not JSON: {e}")))?;
    let map = mapping
        .as_object()
        .ok_or_else(|| invalid("the oracle file must be a JSON object of id -> statement"))?;
    for request in &mut requests {
        if let Some(statement) = map.get(&request.id) {
            let statement = statement
                .as_str()
                .ok_or_else(|| invalid(format!("oracle {} is not a string", request.id)))?;
            request.user_turns.insert(0, statement.to_string());
        }
    }
    Ok(requests)
}

fn chat_with<S: Stepper>(
    new_session: &dyn Fn() -> S,
    context: usize,
    encoder: &uor_r4_tokenizer::dialogue::DialogueEncoder<'_>,
    protocol: &uor_r4_tokenizer::dialogue::DialogueProtocol,
    requests: Option<&[uor_r4_training::stack_dialogue::Request]>,
    max_new_tokens: usize,
    decode: &dyn Fn(&[u32]) -> String,
    pick: &mut Pick<'_>,
    positions: &mut usize,
    costs: &mut Vec<TurnCost>,
) -> Result<(Value, Option<TrainingError>)> {
    let mut chat = IntegerChat {
        session: new_session(),
        fed: Vec::new(),
        logits: Vec::new(),
    };
    let eos = protocol.eos_id;
    if let Some(requests) = requests {
        let record = reply_panel(
            encoder,
            protocol,
            requests,
            context,
            max_new_tokens,
            decode,
            &mut |history, cap| {
                let clock = Instant::now();
                let reply = chat.reply(new_session, history, cap, eos, pick)?;
                costs.push(TurnCost {
                    ids: reply.ids.len(),
                    seconds: clock.elapsed().as_secs_f64(),
                });
                *positions += reply.ids.len();
                Ok(reply)
            },
        )?;
        return Ok((record, None));
    }
    let mut events = Vec::new();
    let outcome = (|| -> Result<()> {
        use std::io::{BufRead, Write};
        let mut history = vec![protocol.bos_id];
        let mut stdout = std::io::stdout();
        write!(stdout, "you> ")?;
        stdout.flush()?;
        for line in std::io::stdin().lock().lines() {
            let line = line?;
            let text = line.trim();
            if text == "/reset" {
                history = vec![protocol.bos_id];
                events.push(json!({"reset": "user"}));
                writeln!(stdout, "[new conversation]")?;
            } else if !text.is_empty() {
                let mut prefix = encoder.encode_user_prefix(text, history.len() > 1);
                let fresh = encoder.encode_user_prefix(text, false);
                if prefix.emitted_turns != 1 || prefix.special_token_occurrences != 0 {
                    events.push(json!({"user": text, "rejected": "not one plain user turn"}));
                    writeln!(stdout, "[that message has special tokens or role markers]")?;
                } else if 1 + fresh.tokens.len() + max_new_tokens > context {
                    events.push(json!({"user": text, "rejected": "too long for the context"}));
                    writeln!(stdout, "[that message is too long for the context]")?;
                } else {
                    if history.len() + prefix.tokens.len() + max_new_tokens > context {
                        history = vec![protocol.bos_id];
                        events.push(json!({"reset": "context_full"}));
                        writeln!(stdout, "[the context is full; starting a new conversation]")?;
                        prefix = fresh;
                    }
                    history.extend(&prefix.tokens);
                    let reply = chat.reply(new_session, &history, max_new_tokens, eos, pick)?;
                    *positions += reply.ids.len();
                    let words: Vec<u32> =
                        reply.ids.iter().copied().filter(|&id| id != eos).collect();
                    writeln!(stdout, "model> {}", decode(&words))?;
                    history.extend(&reply.ids);
                    if !reply.eos {
                        history.push(eos);
                    }
                    events.push(json!({"user": text, "reply": decode(&words),
                        "reply_ids": reply.ids, "model_eos": reply.eos,
                        "stop": reply.stop_record()}));
                }
            }
            write!(stdout, "you> ")?;
            stdout.flush()?;
        }
        writeln!(stdout)?;
        Ok(())
    })();
    let error = outcome.as_ref().err().map(|e| e.to_string());
    Ok((
        json!({"interactive": events, "error": error}),
        outcome.err(),
    ))
}

fn finish(out: &Path, result: Result<()>) -> Result<()> {
    if let Err(error) = &result {
        fs::write(
            out.join("error.json"),
            serde_json::to_vec_pretty(&json!({"error": error.to_string()}))?,
        )?;
    }
    report_output::seal(out)?;
    report_output::verify(out)?;
    result
}

/// `--help`: the modes, and the options of `dialogue-train` (the module
/// documentation has every mode's full usage).
const HELP: &str = "\
geometric-stack MODE key=value ...
modes: train|sample|evaluate|encode|corpus|export|lut-evaluate|d11-evaluate|lut-sample|\
snap-evaluate|rounding-attribution|dialogue-train|lut-chat
(every mode's full usage is in the header of examples/geometric-stack.rs)

geometric-stack dialogue-train out=NEW_REPORT_ROOT tokenizer=TOKENIZER.json \\
  train_tokens=TRAIN.uort train_mask=TRAIN.mask train_manifest=TRAIN/manifest.json \\
  dev_tokens=DEV.uort dev_mask=DEV.mask dev_manifest=DEV/manifest.json \\
  (init=ROOT/model | arch=geometric|transformer [width= heads= layers= pattern= read= rotation= \\
  stack_mlp= mlp=]) [qat=false|true] [transport_snap=none|icosian] \\
  [select=none|flock:WINDOW:K] [pointer=none|DIM] [pointer_score=dot|lorentz] \\
  [pointer_select=none|flock:WINDOW:K|top:K] \\
  [pointer_route=none|prime:WINDOW|prime-ranked:WINDOW|ngram:WINDOW|ngram-ranked:WINDOW] \\
  [seed=] [context=] [policy=] \\
  [data_seed=] \\
  [steps=] \\
  [batch=] [lr=] [warmup=] [min_lr=] [weight_decay=] [clip=] [eval_every=] [dev_seed=] \\
  [dev_per_source=] [checkpoint_every=] [resume=] [max_seconds=] [requests=] [max_new_tokens=] \\
  [protocol=1|2]

  select=flock:WINDOW:K  every read row (geometric reads and the control's attention) softmaxes
                         over the sink (position 0), the last WINDOW positions and the K
                         best-scoring other sources only (the shared crate::flock selector);
                         unkept sources weigh 0. It never applies to the pointer. With init=,
                         replaces the saved flock (select=none clears it); the weights do not
                         change.
  pointer=DIM            a pointer-copy head after the final norm (DIM-wide query and key over
                         the input tokens, and a gate that starts at sigmoid(-2)); the loss, dev
                         scores and greedy replies are the mixture's. With init= on a model saved
                         without one, adds the head with weights drawn fresh from seed= (default:
                         the saved seed); the seed is recorded in the head's init_seed and the
                         report, and a resume must carry the same one. Refused with qat=true and
                         by the evaluators that read raw logits (snap-evaluate,
                         rounding-attribution, lut-evaluate). export writes it when it keeps every
                         source (no pointer_select, no pointer_route); both integer engines serve
                         its mixture.
  pointer_score=...      the pointer's own score of a source: dot (default, q.k/sqrt(DIM)) or
                         lorentz (the fused read's hyperboloid form, with a learned scale
                         pointer.log_beta). With init=, must agree with a saved head.
  pointer_select=...     the sources the pointer softmaxes over (default none: all): the flock
                         WINDOW:K, or top:K (the K best alone; top:1 is the single-source
                         pointer). Its own selection, not select=. Training stays soft unless
                         given, and refuses top:1 (its one kept source gives the query, key and
                         scale no gradient); m-world evaluate applies any selection, top:1
                         included, to saved weights afterwards. With init=, replaces the saved
                         head's selection (the weights do not change).
  pointer_route=...      none (default) or prime:WINDOW, the exact prime route (ADR-0003): a
                         source is admitted when the registered primes of the WINDOW tokens
                         before it share a factor with the query's last WINDOW (1..6), scored by
                         ln gcd plus recency, and the pointer copies the token that followed. The
                         gate learns; the query and key get no gradient. prime-ranked:WINDOW
                         keeps the admission and ranks admitted sources by the learned score
                         plus the route's (learned pointer where none is admitted). ngram:WINDOW
                         and ngram-ranked:WINDOW admit by the longest ordered n-let match
                         (n up to WINDOW) instead of any shared atom. Excludes
                         pointer_select=. With init=, replaces the saved head's route.
  protocol=1|2           the literal-role dialogue version of both corpora (default 1); 2 puts
                         the space after a role marker into the message (m-world corpus
                         protocol=2), so a reply's first word can be copied from context.
  reports                each eval adds dev_pointer_mean_gate / dev_pointer_hit_rate /
                         dev_pointer_reachable_rate to the curve; all settings are in the saved
                         config.json and the report's config and flock_and_pointer.
";

fn main() -> Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments
        .iter()
        .any(|argument| matches!(argument.as_str(), "--help" | "-h" | "help"))
    {
        print!("{HELP}");
        return Ok(());
    }
    let Some((mode, rest)) = arguments.split_first() else {
        return Err(invalid(
            "usage: geometric-stack train|sample|evaluate|encode|corpus|export|lut-evaluate|d11-evaluate|lut-sample|dialogue-train|lut-chat key=value ...",
        ));
    };
    match mode.as_str() {
        "train" => {
            let args = Args::parse(
                rest,
                &[
                    "train",
                    "train_weights",
                    "width",
                    "heads",
                    "layers",
                    "mlp",
                    "stack_mlp",
                    "context",
                    "valid",
                    "out",
                    "arch",
                    "pattern",
                    "read",
                    "rotation",
                    "rotation_group",
                    "seed",
                    "steps",
                    "batch",
                    "lr",
                    "warmup",
                    "min_lr",
                    "weight_decay",
                    "clip",
                    "eval_every",
                    "eval_windows",
                    "final_windows",
                    "lens",
                    "merges",
                    "tokenizer",
                    "checkpoint_every",
                    "resume",
                    "max_seconds",
                    "sample_tokens",
                    "device",
                    "memory_layers",
                    "memory_sub_keys",
                    "memory_top_k",
                    "memory_heads",
                    "memory_key_dim",
                    "memory_score",
                    "memory_codebook",
                    "init",
                    "qat",
                    "transport_snap",
                    "data_parallel",
                    "tf32",
                ],
            )?;
            let settings = train_settings(&args)?;
            let out = PathBuf::from(args.required("out")?);
            report_output::claim(&out)?;
            let result = train(&settings, &out);
            finish(&out, result)
        }
        "sample" => sample_mode(rest),
        "evaluate" => evaluate_mode(rest),
        "encode" => encode_mode(rest),
        "corpus" => corpus_mode(rest),
        "export" => export_mode(rest),
        "lut-evaluate" => lut_evaluate_mode(rest),
        "d11-evaluate" => d11_evaluate_mode(rest),
        "snap-evaluate" => snap_evaluate_mode(rest),
        "rounding-attribution" => rounding_attribution_mode(rest),
        "lut-sample" => lut_sample_mode(rest),
        "dialogue-train" => dialogue_train_mode(rest),
        "lut-chat" => lut_chat_mode(rest),
        other => Err(invalid(format!("unknown mode {other}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEYS: [&str; 6] = [
        "select",
        "pointer",
        "pointer_score",
        "pointer_select",
        "pointer_route",
        "seed",
    ];

    fn args(pairs: &[&str]) -> Args {
        let arguments: Vec<String> = pairs.iter().map(|pair| (*pair).to_owned()).collect();
        Args::parse(&arguments, &KEYS).expect("arguments")
    }

    /// A configuration without a pointer head, seed 5.
    fn saved() -> StackConfig {
        StackConfig::transformer(16, 2, 2, 24, 12, 5).expect("a small control")
    }

    /// The same with a head of width 8 that was added from seed 3.
    fn saved_with_head() -> StackConfig {
        let mut config = saved();
        config.pointer = Some(PointerConfig {
            init_seed: Some(3),
            ..PointerConfig::new(8)
        });
        config
    }

    #[test]
    fn the_pointer_options_have_one_grammar() {
        assert_eq!(pointer_args(&args(&[])).expect("none").head, None);
        assert_eq!(
            pointer_args(&args(&["pointer=none"])).expect("none").head,
            Some(None)
        );
        let given = pointer_args(&args(&[
            "pointer=8",
            "pointer_score=lorentz",
            "pointer_select=top:1",
        ]))
        .expect("all three");
        assert_eq!(given.head, Some(Some(8)));
        assert_eq!(given.score, Some(ReadScore::Lorentz));
        assert_eq!(given.select, Some(Some(PointerSelect::TopK(1))));
        assert_eq!(
            pointer_args(&args(&["pointer_select=none"]))
                .expect("cleared")
                .select,
            Some(None)
        );
        for bad in [
            "pointer=0",
            "pointer=x",
            "pointer_score=hyperbolic",
            "pointer_select=top:0",
            "pointer_select=flock:0:1",
            "pointer_select=window:3",
            "pointer_route=prime:0",
            "pointer_route=prime:7",
            "pointer_route=gcd:2",
            "pointer_route=prime",
        ] {
            assert!(pointer_args(&args(&[bad])).is_err(), "{bad}");
        }
        assert_eq!(
            pointer_args(&args(&["pointer_route=prime:2"]))
                .expect("a route")
                .route,
            Some(Some(PrimeRoute::exact(2)))
        );
        assert_eq!(
            pointer_args(&args(&["pointer_route=prime-ranked:3"]))
                .expect("a ranked route")
                .route,
            Some(Some(PrimeRoute::ranked(3)))
        );
        assert_eq!(
            pointer_args(&args(&["pointer_route=none"]))
                .expect("cleared")
                .route,
            Some(None)
        );
    }

    #[test]
    fn a_saved_head_takes_a_prime_route_without_a_selection() {
        let with_head = saved_with_head();
        let route = PrimeRoute::exact(2);
        let (config, added) = init_extended_config(&args(&["pointer_route=prime:2"]), &with_head)
            .expect("a routed head");
        assert!(!added);
        assert_eq!(
            config.pointer,
            Some(PointerConfig {
                route: Some(route),
                init_seed: Some(3),
                ..PointerConfig::new(8)
            })
        );
        let mut routed = with_head.clone();
        routed.pointer = config.pointer;
        let (config, _) =
            init_extended_config(&args(&["pointer_route=none"]), &routed).expect("a cleared route");
        assert_eq!(config.pointer, with_head.pointer);
        // A route admits its own sources: no selection with it.
        assert!(init_extended_config(
            &args(&["pointer_route=prime:2", "pointer_select=top:2"]),
            &with_head
        )
        .is_err());
        // A new head may be routed from the start; a route needs a head.
        let (config, added) =
            init_extended_config(&args(&["pointer=8", "pointer_route=prime:1"]), &saved())
                .expect("a new routed head");
        assert!(added);
        assert_eq!(
            config.pointer.and_then(|pointer| pointer.route),
            Some(PrimeRoute::exact(1))
        );
        assert!(init_extended_config(&args(&["pointer_route=prime:1"]), &saved()).is_err());
    }

    #[test]
    fn a_single_source_pointer_is_not_a_training_setting() {
        let refusal = refuse_trained_single_source(parse_pointer_select("top:1").expect("valid"))
            .expect_err("top:1 is refused");
        assert!(
            refusal
                .to_string()
                .contains("m-world evaluate pointer_select=top:1"),
            "{refusal}"
        );
        // The scores of the sources a wider selection keeps get gradient.
        for text in ["none", "top:2", "flock:8:2"] {
            let select = parse_pointer_select(text).expect("valid");
            assert!(refuse_trained_single_source(select).is_ok(), "{text}");
        }
    }

    #[test]
    fn dialogue_train_refuses_top_one_before_claiming_its_report() {
        let out = std::env::temp_dir().join(format!("uor-r4-top1-refusal-{}", std::process::id()));
        let arguments: Vec<String> = vec![
            format!("out={}", out.display()),
            "tokenizer=unused".to_owned(),
            "train_tokens=unused".to_owned(),
            "train_mask=unused".to_owned(),
            "train_manifest=unused".to_owned(),
            "dev_tokens=unused".to_owned(),
            "dev_mask=unused".to_owned(),
            "dev_manifest=unused".to_owned(),
            "pointer=8".to_owned(),
            "pointer_select=top:1".to_owned(),
        ];
        let refusal = dialogue_train_mode(&arguments).expect_err("top:1 is refused");
        assert!(
            refusal.to_string().contains("not a training setting"),
            "{refusal}"
        );
        assert!(!out.exists(), "the refusal claimed {}", out.display());
    }

    #[test]
    fn the_d11_evaluate_comparator_refuses_a_pointer_model() -> Result<()> {
        let directory = std::env::temp_dir().join(format!(
            "geometric-stack-d11-comparator-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&directory);
        // A pointer model's raw logits are not its distribution: refused, as
        // the snapped D11 evaluator loads its comparator.
        StackModel::new(saved_with_head(), &Device::Cpu)?.save(&directory)?;
        let refusal = load_raw_logit_comparator(&directory, "d11-evaluate")
            .err()
            .ok_or_else(|| invalid("a pointer comparator was accepted"))?;
        assert!(
            refusal
                .to_string()
                .contains("d11-evaluate reads the model's raw logits"),
            "{refusal}"
        );
        // Without the head it is an ordinary comparator.
        fs::remove_dir_all(&directory)?;
        StackModel::new(saved(), &Device::Cpu)?.save(&directory)?;
        load_raw_logit_comparator(&directory, "d11-evaluate")?;
        fs::remove_dir_all(&directory)?;
        Ok(())
    }

    #[test]
    fn ordinary_train_refuses_a_saved_single_source_pointer() -> Result<()> {
        let directory =
            std::env::temp_dir().join(format!("geometric-stack-train-top1-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory)?;
        // `train init=` with a model saved with `select`, as far as the settings.
        let settings_for = |select: Option<PointerSelect>| -> Result<Result<Settings>> {
            let mut config = saved_with_head();
            if let Some(pointer) = config.pointer.as_mut() {
                pointer.select = select;
            }
            fs::write(directory.join("config.json"), serde_json::to_vec(&config)?)?;
            let arguments = vec![format!("init={}", directory.display())];
            Ok(train_settings(&Args::parse(&arguments, &["init"])?))
        };
        let refusal = match settings_for(Some(PointerSelect::TopK(1)))? {
            Ok(_) => return Err(invalid("a saved top:1 pointer was accepted for training")),
            Err(error) => error.to_string(),
        };
        assert!(refusal.contains("not a training setting"), "{refusal}");
        // A soft head passes this check and stops later, at the missing train=.
        let other = match settings_for(None)? {
            Ok(_) => return Err(invalid("the settings were accepted without train=")),
            Err(error) => error.to_string(),
        };
        assert!(!other.contains("not a training setting"), "{other}");
        fs::remove_dir_all(&directory)?;
        Ok(())
    }

    #[test]
    fn a_head_added_to_init_records_the_seed_its_weights_come_from() {
        let (config, added) = init_extended_config(
            &args(&[
                "pointer=8",
                "pointer_score=lorentz",
                "pointer_select=top:1",
                "seed=9",
            ]),
            &saved(),
        )
        .expect("a new head");
        assert!(added);
        assert_eq!(
            config.pointer,
            Some(PointerConfig {
                dim: 8,
                score: ReadScore::Lorentz,
                select: Some(PointerSelect::TopK(1)),
                init_seed: Some(9),
                route: None,
            })
        );
        // Without seed= it is the saved model's seed, as the weights are.
        let (config, added) =
            init_extended_config(&args(&["pointer=8"]), &saved()).expect("a new head");
        assert!(added);
        assert_eq!(
            config.pointer,
            Some(PointerConfig {
                init_seed: Some(5),
                ..PointerConfig::new(8)
            })
        );
        // A head is what pointer_score= and pointer_select= configure.
        for dangling in [["pointer_select=top:1"], ["pointer_score=dot"]] {
            assert!(init_extended_config(&args(&dangling), &saved()).is_err());
        }
        assert!(init_extended_config(&args(&["pointer=none"]), &saved()).is_ok());
    }

    #[test]
    fn a_saved_head_keeps_its_shape_and_its_seed() {
        let saved = saved_with_head();
        let (config, added) = init_extended_config(&args(&[]), &saved).expect("unchanged");
        assert!(!added && config == saved);
        // The head's selection is not a weight: it may be replaced or cleared,
        // and its recorded seed stays.
        let (config, added) =
            init_extended_config(&args(&["pointer=8", "pointer_select=top:1"]), &saved)
                .expect("a new selection");
        assert!(!added);
        assert_eq!(
            config.pointer,
            Some(PointerConfig {
                select: Some(PointerSelect::TopK(1)),
                init_seed: Some(3),
                ..PointerConfig::new(8)
            })
        );
        let mut selected = saved.clone();
        selected.pointer = config.pointer;
        let (config, _) = init_extended_config(&args(&["pointer_select=none"]), &selected)
            .expect("a cleared selection");
        assert_eq!(config.pointer, saved.pointer);
        // Its width and score are its weights' shape; it cannot be removed.
        for conflict in [["pointer=16"], ["pointer_score=lorentz"], ["pointer=none"]] {
            assert!(
                init_extended_config(&args(&conflict), &saved).is_err(),
                "{conflict:?}"
            );
        }
        // A new seed= does not change a head that already has weights.
        let (config, added) =
            init_extended_config(&args(&["seed=77"]), &saved).expect("seed of the window");
        assert!(!added);
        assert_eq!(config.pointer, saved.pointer);
    }

    #[test]
    fn a_resume_must_carry_the_seed_of_the_head_it_continues() -> Result<()> {
        let directory =
            std::env::temp_dir().join(format!("geometric-stack-lineage-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory)?;
        let state = |config: &StackConfig| -> Result<()> {
            Ok(fs::write(
                directory.join("state.json"),
                serde_json::to_vec(&json!({"lineage": {"config": config}}))?,
            )?)
        };
        let seeded = |seed: Option<u64>| {
            let mut config = saved();
            config.pointer = Some(PointerConfig {
                init_seed: seed,
                ..PointerConfig::new(8)
            });
            config
        };
        // A checkpoint whose head was drawn from seed 9.
        state(&seeded(Some(9)))?;
        check_resume_pointer_seed(&directory, &seeded(Some(9)))?;
        let error = check_resume_pointer_seed(&directory, &seeded(Some(10)))
            .expect_err("another seed is refused");
        let text = error.to_string();
        assert!(text.contains("10") && text.contains('9'), "{text}");
        assert!(check_resume_pointer_seed(&directory, &seeded(None)).is_err());
        assert!(check_resume_pointer_seed(&directory, &saved()).is_err());
        // A checkpoint whose head was built with its model records none, and a
        // resume that names one is refused.
        state(&seeded(None))?;
        check_resume_pointer_seed(&directory, &seeded(None))?;
        assert!(check_resume_pointer_seed(&directory, &seeded(Some(9))).is_err());
        // A checkpoint without a head.
        state(&saved())?;
        check_resume_pointer_seed(&directory, &saved())?;
        assert!(check_resume_pointer_seed(&directory, &seeded(Some(9))).is_err());
        fs::remove_dir_all(&directory)?;
        Ok(())
    }

    /// GPT-2's byte-to-character alphabet.
    fn alphabet() -> Vec<char> {
        let mut printable: Vec<u32> = (u32::from(b'!')..=u32::from(b'~')).collect();
        printable.extend(0xA1..=0xAC);
        printable.extend(0xAE..=0xFF);
        let mut table = vec!['\0'; 256];
        let mut extra = 0;
        for byte in 0u32..256 {
            table[byte as usize] = if printable.contains(&byte) {
                char::from_u32(byte).expect("printable")
            } else {
                extra += 1;
                char::from_u32(255 + extra).expect("shifted")
            };
        }
        table
    }

    /// A byte-level tokenizer with the three dialogue specials at ids 0-2
    /// (259 ids).
    fn byte_tokenizer() -> uor_r4_tokenizer::ByteBpeTokenizer {
        let specials = ["<|bos|>", "<|eos|>", "<|unk|>"];
        let mut vocab = serde_json::Map::new();
        for (id, surface) in specials.iter().enumerate() {
            vocab.insert((*surface).to_owned(), json!(id));
        }
        for (byte, ch) in alphabet().iter().enumerate() {
            vocab.insert(ch.to_string(), json!(byte + 3));
        }
        let added: Vec<Value> = specials
            .iter()
            .enumerate()
            .map(|(id, surface)| json!({"id": id, "content": surface}))
            .collect();
        uor_r4_tokenizer::ByteBpeTokenizer::from_tokenizer_json_bytes(
            json!({
                "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
                "added_tokens": added,
                "model": {"type": "BPE", "vocab": vocab, "merges": []},
            })
            .to_string()
            .as_bytes(),
        )
        .expect("a byte-level tokenizer")
    }

    /// Each turn's reply ids and text, without the timing.
    fn replies_of(record: &Value) -> Vec<(Value, Value)> {
        record["rows"]
            .as_array()
            .into_iter()
            .flatten()
            .flat_map(|row| row["turns"].as_array().into_iter().flatten())
            .map(|turn| (turn["reply_ids"].clone(), turn["reply"].clone()))
            .collect()
    }

    #[test]
    fn a_pointer_stack_chats_deterministically_and_alike_on_d11_and_d10() -> Result<()> {
        let tokenizer = byte_tokenizer();
        let mut config = StackConfig::transformer_control(7);
        config.arch = StackArch::Geometric;
        config.vocab_size = 259;
        config.width = 64;
        config.heads = 2;
        config.mlp_hidden = 40;
        config.context = 128;
        config.pattern = "rar".to_owned();
        config.read = ReadScore::Dot;
        config.rotation = true;
        config.pointer = Some(PointerConfig::new(8));
        let model = StackModel::new(config, &Device::Cpu)?;
        let (bytes, _) = export_stack(&model, json!({"test": "lut-chat"}), None, None)?;
        let requests: Vec<uor_r4_training::stack_dialogue::Request> =
            serde_json::from_value(json!([
                {"id": "one", "category": "test", "user_turns": ["Hi there"]},
                {"id": "two", "category": "test", "user_turns": ["Name a color.", "And another?"]},
            ]))?;
        let protocol = DialogueProtocol::literal_roles_version(&tokenizer, 2)
            .map_err(|e| invalid(e.to_string()))?;
        let chat = |engine| -> Result<MixtureChat> {
            mixture_chat(
                bytes.clone(),
                engine,
                1,
                &tokenizer,
                &protocol,
                Some(&requests),
                12,
            )
        };
        let first = chat(MixtureEngine::D11)?;
        let again = chat(MixtureEngine::D11)?;
        let d10 = chat(MixtureEngine::D10)?;
        let replies = replies_of(&first.record);
        assert_eq!(replies.len(), 3);
        assert!(first.positions > 0 && first.failure.is_none());
        assert_eq!(
            replies,
            replies_of(&again.record),
            "D11 is not deterministic"
        );
        assert_eq!(
            replies,
            replies_of(&d10.record),
            "D11 and D10 replies differ"
        );
        // Every turn carries its cost, and the panel its totals.
        assert_eq!(first.record["cost"]["replies"], json!(3));
        assert_eq!(
            first.record["cost"]["generated_ids"],
            json!(first.positions)
        );
        // Sampling options are refused for a pointer artifact.
        assert!(refuse_mixture_sampling(&["temperature"]).is_err());
        assert!(refuse_mixture_sampling(&[]).is_ok());
        Ok(())
    }
}
