//! Chat with an integer-served model (no floating point, multiplier-free weight maps).
//!
//! ```text
//! lut-chat lut=MODEL.lut tokenizer=DIR/tokenizer.json [prompt=TEXT] [system=TEXT]
//!     [tokens=256] [threads=N] [raw=true]
//!     [temperature=0] [top_k=0] [top_p=1] [presence=0] [seed=1]
//! ```
//!
//! With `prompt=` it answers one turn; otherwise it reads one user turn per line
//! from standard input and keeps the conversation in the session's key/value
//! cache. Decoding is greedy unless `temperature` is positive; sampling is
//! integer-only and a seed reproduces it (SmolLM2's model card suggests
//! temperature 0.2 and top_p 0.9). The prompt uses SmolLM2's chat template, including
//! its default system turn; `raw=true` instead continues the given text as is
//! (for base models), stopping only at the token limit. With an artifact that
//! carries a learned cache (lab M4), the next token comes from the mixture of
//! the model and the cache, and a full attention window slides (the recent
//! half is re-encoded) while the cache keeps everything older.

use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::time::Instant;

use uor_r4_lut::engine::{Model, Session};
use uor_r4_lut::sampling::{Sampler, SamplingSettings};
use uor_r4_tokenizer::ByteBpeTokenizer;

const DEFAULT_SYSTEM: &str = "You are a helpful AI assistant named SmolLM, trained by Hugging Face";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Feed one token. With a cache, a full attention window slides first: the
/// most recent half of `history` is re-encoded and the cache keeps the rest.
fn feed(session: &mut Session<'_>, history: &[u32], token: u32, slide: bool) -> Result<()> {
    let max = session.model().shape().max_positions;
    if slide && session.position() + 1 > max {
        let start = history.len().saturating_sub(max / 2);
        session.reencode(&history[start..])?;
    }
    session.step(token)?;
    Ok(())
}

/// Feed `ids`, then generate until `stop` or `limit` new tokens, printing
/// text as it becomes valid UTF-8. `history` holds every token the session has
/// read. With a cache, the next token comes from the mixture of the model and
/// the cache, and the attention window slides instead of filling. Returns the
/// number of new tokens.
fn turn(
    session: &mut Session<'_>,
    sampler: &mut Sampler,
    history: &mut Vec<u32>,
    tokenizer: &ByteBpeTokenizer,
    ids: &[u32],
    stop: u32,
    limit: usize,
) -> Result<usize> {
    let model = session.model();
    let slide = model.cache_spec().is_some();
    let (table, step) = model.exp_table();
    let mut choose = |session: &Session<'_>, history: &[u32]| -> Result<u32> {
        Ok(sampler.sample_with_cache(
            session.logits(),
            history,
            table,
            step,
            session.cache_read().as_ref(),
        )?)
    };
    let mut next = 0u32;
    for &id in ids {
        feed(session, history, id, slide)?;
        history.push(id);
        next = choose(session, history)?;
    }
    let mut generated = Vec::new();
    let mut printed = 0usize;
    let mut out = std::io::stdout();
    while generated.len() < limit && next != stop {
        generated.push(next);
        let bytes = tokenizer.decode_bytes(&generated);
        if let Ok(text) = std::str::from_utf8(&bytes) {
            write!(out, "{}", &text[printed..])?;
            out.flush()?;
            printed = text.len();
        }
        feed(session, history, next, slide)?;
        history.push(next);
        next = choose(session, history)?;
    }
    if next == stop {
        // Close the assistant turn in the cache.
        feed(session, history, stop, slide)?;
        history.push(stop);
    }
    writeln!(out)?;
    Ok(generated.len())
}

fn main() -> Result<()> {
    let mut args = BTreeMap::new();
    for argument in std::env::args().skip(1) {
        let (key, value) = argument.split_once('=').ok_or("arguments are key=value")?;
        args.insert(key.to_owned(), value.to_owned());
    }
    let lut = PathBuf::from(args.get("lut").ok_or("missing lut=")?);
    let tokenizer_path = PathBuf::from(args.get("tokenizer").ok_or("missing tokenizer=")?);
    let system = args.get("system").map_or(DEFAULT_SYSTEM, String::as_str);
    let limit: usize = args.get("tokens").map_or(Ok(256), |v| v.parse())?;
    let threads: usize = match args.get("threads") {
        Some(v) => v.parse()?,
        None => std::thread::available_parallelism().map_or(1, usize::from),
    };
    let raw = args.get("raw").is_some_and(|v| v == "true");
    let number = |key: &str, default: f64| -> Result<f64> {
        Ok(args.get(key).map_or(Ok(default), |v| v.parse::<f64>())?)
    };
    let settings = SamplingSettings::from_decimal(
        number("temperature", 0.0)?,
        args.get("top_k").map_or(Ok(0), |v| v.parse::<usize>())?,
        number("top_p", 1.0)?,
        number("presence", 0.0)?,
    )?;
    let seed: u64 = args.get("seed").map_or(Ok(1), |v| v.parse())?;
    let mut sampler = Sampler::new(settings, seed);
    let mut history = Vec::new();
    let mut model = Model::load(&lut)?;
    model.set_threads(threads)?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&std::fs::read(&tokenizer_path)?)
        .ok_or("tokenizer.json is not a byte-level BPE tokenizer")?;
    let stop_ids = tokenizer.encode("<|im_end|>");
    let stop = match stop_ids[..] {
        [stop] => stop,
        _ if raw => u32::MAX,
        _ => return Err("<|im_end|> is not a single token".into()),
    };
    eprintln!(
        "model {} ({} layers, width {}), artifact sha256 {}, {} kernels, {} threads",
        lut.display(),
        model.shape().layers,
        model.shape().width,
        model.artifact_sha256(),
        model.backend().name(),
        model.threads()
    );
    let mut session = model.session();
    let mut first = true;
    let mut ask = |session: &mut Session<'_>, message: &str| -> Result<()> {
        let render = |with_system: bool| {
            if raw {
                return tokenizer.encode(message);
            }
            let mut text = String::new();
            if with_system {
                text.push_str(&format!("<|im_start|>system\n{system}<|im_end|>\n"));
            } else {
                text.push('\n');
            }
            text.push_str(&format!(
                "<|im_start|>user\n{message}<|im_end|>\n<|im_start|>assistant\n"
            ));
            tokenizer.encode(&text)
        };
        let mut ids = render(first);
        let slides = model.cache_spec().is_some();
        if !slides && session.position() + ids.len() + limit + 1 > model.shape().max_positions {
            eprintln!("(context full: starting a new conversation)");
            session.reset();
            history.clear();
            ids = render(true);
            if ids.len() + limit + 1 > model.shape().max_positions {
                return Err("the message does not fit the model's context".into());
            }
        }
        first = false;
        let started = Instant::now();
        let produced = turn(
            session,
            &mut sampler,
            &mut history,
            &tokenizer,
            &ids,
            stop,
            limit,
        )?;
        let seconds = started.elapsed().as_secs_f64();
        // Every prompt and generated token is one model step (the energy
        // script reads the `[N model tokens` prefix).
        let steps = ids.len() + produced;
        eprintln!(
            "[{steps} model tokens; {} prompt + {produced} generated in {seconds:.2}s; {:.1} tok/s]",
            ids.len(),
            steps as f64 / seconds.max(1e-9)
        );
        Ok(())
    };
    match args.get("prompt") {
        Some(prompt) => ask(&mut session, prompt),
        None => {
            let stdin = std::io::stdin();
            for line in stdin.lock().lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }
                if let Err(error) = ask(&mut session, line.trim()) {
                    eprintln!("error: {error}");
                }
            }
            Ok(())
        }
    }
}
