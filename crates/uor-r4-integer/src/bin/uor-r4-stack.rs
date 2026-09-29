//! Multiplier-free (D11) serving of a geometric-stack artifact (`UORLUT01`,
//! schema `uor-r4.lut-stack/1`) through `uor_r4_integer::stack`.
//!
//! ```text
//! uor-r4-stack generate ARTIFACT PROMPT_IDS NEW_TOKENS
//! uor-r4-stack digest ARTIFACT TOKENS.u16 WINDOWS
//! ```
//!
//! `generate` feeds the comma-separated prompt ids and continues greedily
//! (integer argmax, first on ties) for at most `NEW_TOKENS` ids or until the
//! context is full, and prints a JSON record. `digest` runs `WINDOWS` evenly
//! spaced full-context windows of a little-endian u16 token file, each in a
//! fresh session, and prints the SHA-256 of every step's logits (little-endian
//! `i32`, in order), a platform-independent fingerprint of the served
//! integers. The served path is `IntegerStackSession::step` and
//! `stack_argmax`; argument parsing, file reading, timing, hashing and JSON
//! are outside it. The instruction audit targets this binary:
//! `python3 scripts/audit_zero_matmul_serving.py target/release/uor-r4-stack --stack`.

use std::path::Path;
use std::time::Instant;

use serde_json::json;
use sha2::{Digest, Sha256};
use uor_r4_integer::stack::{stack_argmax, IntegerStackModel, StackError};

#[derive(Debug)]
enum CliError {
    Usage(String),
    Io(std::io::Error),
    Stack(StackError),
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage(message) => write!(f, "{message}"),
            Self::Io(error) => write!(f, "I/O: {error}"),
            Self::Stack(error) => write!(f, "{error}"),
        }
    }
}

impl From<StackError> for CliError {
    fn from(error: StackError) -> Self {
        Self::Stack(error)
    }
}

impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

const USAGE: &str = "usage: uor-r4-stack generate ARTIFACT PROMPT_IDS NEW_TOKENS | digest ARTIFACT TOKENS.u16 WINDOWS";

fn usage(message: &str) -> CliError {
    CliError::Usage(format!("{message}\n{USAGE}"))
}

fn number(text: &str, what: &str) -> Result<usize, CliError> {
    text.parse().map_err(|_| {
        usage(&format!(
            "{what} must be a non-negative integer, got {text}"
        ))
    })
}

/// The serving loop: the prompt, then greedy ids until `new_tokens` or a full
/// context. Every logit vector comes from one `step`.
fn serve_greedy(
    model: &IntegerStackModel,
    prompt: &[u32],
    new_tokens: usize,
) -> Result<(Vec<u32>, usize), StackError> {
    let mut session = model.session();
    let context = model.shape().context;
    let mut next = 0u32;
    for &id in prompt {
        next = stack_argmax(session.step(id)?) as u32;
    }
    // At most one id per position the context has left, whatever NEW_TOKENS is.
    let mut generated = Vec::with_capacity(new_tokens.min(context));
    while generated.len() < new_tokens {
        generated.push(next);
        if generated.len() == new_tokens || session.position() >= context {
            break;
        }
        next = stack_argmax(session.step(next)?) as u32;
    }
    Ok((generated, session.position()))
}

fn generate(args: &[String]) -> Result<(), CliError> {
    let [artifact, prompt, new_tokens] = args else {
        return Err(usage("generate takes ARTIFACT PROMPT_IDS NEW_TOKENS"));
    };
    let prompt: Vec<u32> = prompt
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| {
            let id = number(s, "a prompt id")?;
            u32::try_from(id).map_err(|_| usage(&format!("prompt id {id} does not fit in u32")))
        })
        .collect::<Result<_, _>>()?;
    let new_tokens = number(new_tokens, "NEW_TOKENS")?;
    if prompt.is_empty() {
        return Err(usage("the prompt needs at least one id"));
    }
    let load = Instant::now();
    let model = IntegerStackModel::load(Path::new(artifact))?;
    let load_seconds = load.elapsed().as_secs_f64();
    let vocab = model.shape().vocab;
    if let Some(&id) = prompt.iter().find(|&&id| id as usize >= vocab) {
        return Err(CliError::Stack(StackError::Token { token: id, vocab }));
    }
    let serve = Instant::now();
    let (generated, steps) = serve_greedy(&model, &prompt, new_tokens)?;
    let serve_seconds = serve.elapsed().as_secs_f64();
    let record = json!({
        "schema": "uor-r4.stack-d11-generation/1",
        "artifact_sha256": model.artifact_sha256(),
        "prompt": prompt,
        "generated": generated,
        "steps": steps,
        "weights_read_per_token": model.weights_per_token(),
        "load_seconds": load_seconds,
        "serve_seconds": serve_seconds,
        "decoding": "greedy integer argmax (first on ties)",
    });
    println!("{record}");
    Ok(())
}

fn digest(args: &[String]) -> Result<(), CliError> {
    let [artifact, tokens, windows] = args else {
        return Err(usage("digest takes ARTIFACT TOKENS.u16 WINDOWS"));
    };
    let windows = number(windows, "WINDOWS")?;
    let model = IntegerStackModel::load(Path::new(artifact))?;
    let (vocab, context) = (model.shape().vocab, model.shape().context);
    let bytes = std::fs::read(tokens)?;
    if bytes.len() % 2 != 0 {
        return Err(usage("the token file is not little-endian u16"));
    }
    let ids: Vec<u32> = bytes
        .chunks_exact(2)
        .map(|pair| u32::from(u16::from_le_bytes([pair[0], pair[1]])))
        .collect();
    // Every window needs a whole context and a stride of at least one token.
    if windows == 0
        || context
            .checked_add(windows)
            .is_none_or(|needed| ids.len() <= needed)
    {
        return Err(usage("too few tokens for the windows"));
    }
    if let Some(&id) = ids.iter().find(|&&id| id as usize >= vocab) {
        return Err(CliError::Stack(StackError::Token { token: id, vocab }));
    }
    let steps = windows
        .checked_mul(context)
        .ok_or_else(|| usage("WINDOWS times the context overflows"))?;
    let stride = (ids.len() - context - 1) / windows;
    let mut hash = Sha256::new();
    let mut session = model.session();
    let clock = Instant::now();
    for window in 0..windows {
        // `window * stride < windows * stride <= ids.len() - context - 1`, so
        // neither the start nor the window's end overflows.
        let start = window * stride;
        session.reset();
        for &id in &ids[start..start + context] {
            for value in session.step(id)? {
                hash.update(value.to_le_bytes());
            }
        }
    }
    let seconds = clock.elapsed().as_secs_f64();
    let record = json!({
        "schema": "uor-r4.stack-d11-digest/1",
        "artifact_sha256": model.artifact_sha256(),
        "tokens": tokens,
        "windows": windows,
        "steps": steps,
        "logits_sha256": hex::encode(hash.finalize()),
        "seconds": seconds,
        "tokens_per_second": steps as f64 / seconds,
        "weights_read_per_token": model.weights_per_token(),
    });
    println!("{record}");
    Ok(())
}

fn run() -> Result<(), CliError> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.split_first() {
        Some((mode, rest)) if mode == "generate" => generate(rest),
        Some((mode, rest)) if mode == "digest" => digest(rest),
        _ => Err(CliError::Usage(USAGE.to_owned())),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
