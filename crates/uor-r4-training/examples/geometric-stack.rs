//! Train, evaluate and sample the geometric stack and its transformer control
//! (`uor_r4_training::geometric_stack`) on u16 token files, and prepare those
//! files with the lab's byte-level BPE.
//!
//! Modes:
//!
//! ```text
//! geometric-stack train train=TRAIN.u16[,MORE.u16] [train_weights=W1,W2] valid=VALID.u16 \
//!   out=NEW_REPORT_ROOT (init=ROOT/model | arch=transformer|geometric) [pattern=rrarra] \
//!   [read=lorentz|dot] [rotation=true|false] [qat=false|true] [transport_snap=none|icosian] \
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
//!   [transport_snap=none|icosian] \
//!   [policy=full_prefix|role_only] [data_seed=1] [steps=1024] [batch=16] [lr=0.001] [warmup=50] \
//!   [min_lr=0.1] [weight_decay=0.1] [clip=1.0] [eval_every=128] [dev_seed=1] [dev_per_source=32] \
//!   [checkpoint_every=128] [resume=OLD_ROOT/checkpoint] [max_seconds=inf] [requests=REQUESTS.json] \
//!   [max_new_tokens=96]
//! geometric-stack lut-chat artifact=ROOT/model.lut tokenizer=TOKENIZER.json out=NEW_REPORT_ROOT \
//!   [requests=REQUESTS.json] [max_new_tokens=96] [temperature=0] [top_k=40] [top_p=1] [seed=1] \
//!   [threads=1]
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
//! representation: by rounding to nearest, with the record in `export.json`
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
//! times, or at `max_new_tokens`.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use candle_core::Device;
use serde_json::{json, Value};
use uor_r4_core::native_geometric::learner::embedding::canonical_h4_roots;
use uor_r4_core::report_output;
use uor_r4_training::dialogue_development;
use uor_r4_training::dialogue_episodes::{EpisodeIndex, PrefixPolicy, EPISODE_CONTEXT};
use uor_r4_training::geometric_stack::{
    logits_cross_entropy, D11Interim, MapCodec, ReadScore, ServedStatistics, StackAdamW, StackArch,
    StackConfig, StackModel, TransportSnap, TransportUsage,
};
use uor_r4_training::lut_export::export_llama;
use uor_r4_training::stack_dialogue::{
    check_panel, development, episode_contract, greedy_reply, load_requests, reply_panel, trim,
    DialogueSplit, Reply, MAX_NEW_TOKENS,
};
use uor_r4_training::stack_export::{
    check_export_representation, control_checkpoint, control_grid_reference, export_stack,
    stack_grid_reference, StackCalibration,
};
use uor_r4_training::stack_memory::{Codebook, MemoryConfig, MemoryScore};
use uor_r4_training::{sha256_file, Result, TrainingError};

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

/// The stacks `qat=true` can train: those the geometric stack export writes.
fn check_qat_config(config: &StackConfig) -> Result<()> {
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
        (
            "read",
            match config.read {
                ReadScore::Lorentz => "lorentz",
                ReadScore::Dot => "dot",
            }
            .to_owned(),
        ),
        ("rotation", config.rotation.to_string()),
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
        Some(other) => return Err(invalid(format!("unknown read {other}"))),
    };
    let rotation = match args.optional("rotation").as_deref() {
        None | Some("true") => true,
        Some("false") => false,
        Some(other) => return Err(invalid(format!("invalid rotation={other}"))),
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
    Ok(config)
}

fn train_settings(args: &Args) -> Result<Settings> {
    let init = args.optional("init").map(PathBuf::from);
    let config = match &init {
        Some(directory) => init_config(args, directory)?,
        None => stack_config(args, None)?,
    };
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
    };
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
        let logits = model.forward(window, 1, window.len())?.detach();
        let last = logits.get(window.len() - 1)?.to_vec1::<f32>()?;
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
    let device = Device::Cpu;
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
    // After `init=` or a resume alike: a saved model holds no snap.
    model.set_transport_snap(settings.transport_snap)?;
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
        let loss = model.loss(&ids, &targets, settings.batch, time)?;
        let value = f64::from(loss.to_scalar::<f32>()?);
        if !value.is_finite() {
            return Err(invalid(format!(
                "nonfinite training loss at step {}",
                progress.step
            )));
        }
        let grads = loss.backward()?;
        let grad_norm = optimizer.update(&model, &grads, lr)?;
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
        let model = StackModel::load(&model_dir, &Device::Cpu)?;
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
            None => json!({"method": "round_to_nearest"}),
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
                "scope": "recorded in the model's config.json: the model was trained against this representation (quantization-aware training), which this round-to-nearest export writes",
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
        use uor_r4_lut::format::{schema_of, Artifact, StackArtifact, SCHEMA, STACK_SCHEMA};
        let schema = schema_of(&bytes).map_err(lut)?;
        Ok(if schema == STACK_SCHEMA {
            let mut model = uor_r4_lut::stack::StackModel::from_artifact(
                StackArtifact::parse(bytes).map_err(lut)?,
            )
            .map_err(lut)?;
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
        let model = StackModel::load(&model_dir, &Device::Cpu)?;
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
            let fused = model.forward(&ids, group.len(), time)?;
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
            if model.config.vocab_size != engine.vocabulary() || model.config.context != time {
                return Err(invalid("the float model and the artifact differ in shape"));
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
        let d11 = IntegerStackModel::parse(&bytes).map_err(|e| invalid(e.to_string()))?;
        let d11_load_seconds = clock.elapsed().as_secs_f64();
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
                let logits11 = s11.step(id).map_err(|e| invalid(e.to_string()))?;
                d11_seconds += clock.elapsed().as_secs_f64();
                let clock = Instant::now();
                let logits10 = s10.step(id).map_err(lut)?;
                d10_seconds += clock.elapsed().as_secs_f64();
                let mut position_max = 0i64;
                for (a, b) in logits11.iter().zip(logits10) {
                    position_max = position_max.max((i64::from(*a) - i64::from(*b)).abs());
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
                let (n11, top11) =
                    score_row(logits11.iter().map(|&v| f64::from(v) / 65536.0), target);
                let (n10, top10) =
                    score_row(logits10.iter().map(|&v| f64::from(v) / 65536.0), target);
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
            "protocol": "evenly spaced windows of the development tokens (train's evaluation rule), one fresh session of each engine per window, position by position; logits compared as integers",
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
                "d11_threads": 1,
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
    let mut float = StackModel::load(model_dir, &Device::Cpu)?;
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
            "d11_threads": 1,
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
}

impl DialogueSettings {
    fn record(&self) -> Value {
        json!({
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
        })
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
            "width",
            "heads",
            "layers",
            "mlp",
            "stack_mlp",
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
            "qat",
            "transport_snap",
        ],
    )?;
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
        policy: match args.optional("policy").as_deref() {
            None | Some("full_prefix") => PrefixPolicy::FullPrefix,
            Some("role_only") => PrefixPolicy::RoleOnly,
            Some(other) => return Err(invalid(format!("unknown policy {other}"))),
        },
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
    let out = PathBuf::from(args.required("out")?);
    report_output::claim(&out)?;
    let result = dialogue_train(&settings, &args, &out);
    finish(&out, result)
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
    let device = Device::Cpu;
    let tokenizer =
        uor_r4_tokenizer::ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(&s.tokenizer)?)
            .ok_or_else(|| invalid("unreadable tokenizer.json"))?;
    let train_split = DialogueSplit::load(&s.train[0], &s.train[1], &s.train[2])?;
    let dev_split = DialogueSplit::load(&s.dev[0], &s.dev[1], &s.dev[2])?;
    let vocab = train_split.vocab_size();
    if dev_split.vocab_size() != vocab {
        return Err(invalid("the splits declare different vocabularies"));
    }
    let (protocol, contract) = episode_contract(&tokenizer, vocab)?;
    let train = train_split.index(contract.clone())?;
    let dev = dev_split.index(contract)?;
    let panel = dialogue_development::select(&dev, s.dev_seed, s.dev_per_source)?;
    let requests = s.requests.as_deref().map(load_requests).transpose()?;
    let config = match &s.init {
        Some(directory) => {
            check_init_tokenizer(directory, &s.tokenizer)?;
            StackModel::load(directory, &device)?.config
        }
        None => stack_config(args, Some(vocab))?,
    };
    if config.context != EPISODE_CONTEXT || config.vocab_size != vocab {
        return Err(invalid(
            "the model's context must be the episodes' 256 and its vocabulary the corpus's",
        ));
    }
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
                Some(directory) => StackModel::load(directory, &device)?,
                None => StackModel::new(config.clone(), &device)?,
            };
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
            let (model, optimizer, progress, state) =
                load_checkpoint(checkpoint, &lineage, &device)?;
            (model, optimizer, progress, Some(state))
        }
    };
    // After `init=` or a resume alike: a saved model holds neither mode.
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
    fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    Ok(())
}

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
        sampler: &mut uor_r4_lut::sampling::Sampler,
        exp: (&[u32], i32),
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
            let next = sampler
                .sample(&self.logits, &seen, exp.0, exp.1)
                .map_err(lut)?;
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
        ],
    )?;
    let artifact_path = PathBuf::from(args.required("artifact")?);
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let requests_path = args.optional("requests").map(PathBuf::from);
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
    let requests = requests_path.as_deref().map(load_requests).transpose()?;
    report_output::claim(&out)?;
    let result = (|| -> Result<()> {
        use uor_r4_lut::sampling::{Sampler, SamplingSettings};
        let engine = Engine::load(fs::read(&artifact_path)?, threads)?;
        let tokenizer = uor_r4_tokenizer::ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(
            &tokenizer_path,
        )?)
        .ok_or_else(|| invalid("unreadable tokenizer.json"))?;
        let (protocol, _) = episode_contract(&tokenizer, engine.vocabulary())?;
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
        let (record, failure) = match &engine {
            Engine::Stack(model) => chat_with(
                &|| model.session(),
                model.exp_table(),
                engine.context(),
                &encoder,
                &protocol,
                requests.as_deref(),
                max_new_tokens,
                &decode,
                &mut sampler,
                &mut positions,
            )?,
            Engine::Llama(model) => chat_with(
                &|| model.session(),
                model.exp_table(),
                engine.context(),
                &encoder,
                &protocol,
                requests.as_deref(),
                max_new_tokens,
                &decode,
                &mut sampler,
                &mut positions,
            )?,
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

/// Replies to a request panel, or one interactive conversation. The record
/// comes back with the error that ended an interactive conversation, if one
/// did, so its transcript is still written.
#[allow(clippy::too_many_arguments)]
fn chat_with<S: Stepper>(
    new_session: &dyn Fn() -> S,
    exp: (&[u32], i32),
    context: usize,
    encoder: &uor_r4_tokenizer::dialogue::DialogueEncoder<'_>,
    protocol: &uor_r4_tokenizer::dialogue::DialogueProtocol,
    requests: Option<&[uor_r4_training::stack_dialogue::Request]>,
    max_new_tokens: usize,
    decode: &dyn Fn(&[u32]) -> String,
    sampler: &mut uor_r4_lut::sampling::Sampler,
    positions: &mut usize,
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
                let reply = chat.reply(new_session, history, cap, eos, sampler, exp)?;
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
                    let reply =
                        chat.reply(new_session, &history, max_new_tokens, eos, sampler, exp)?;
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

fn main() -> Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
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
