//! Step 0d instrument (#820): where a D11 step's time goes at full context.
//!
//! Training-free. Two modes:
//!
//! ```text
//! d11-read-share prompts tokenizer=TOKENIZER.json panels=A.json[,B.json...] out=NEW_ROOT \
//!   [context=384] [max_new_tokens=4] [count=4]
//! d11-read-share steps artifact=MODEL.lut tokenizer=TOKENIZER.json requests=ROOT/requests.json \
//!   out=NEW_ROOT [threads=1] [hot_from=370] [hot_repeats=40] [hot_pause_ms=0]
//! ```
//!
//! `prompts` concatenates the open panels' user turns, in file order, into
//! `count` single-turn requests whose protocol-2 history (BOS and the user
//! prefix) is as long as possible without exceeding
//! `context - max_new_tokens` ids. It writes `requests.json` (the study's
//! request format, for `geometric-stack lut-chat`) and `prompts.json` (each
//! request's exact history length and source request ids).
//!
//! `steps` serves each request's history through the D11 session
//! (`IntegerStackModel`, `threads=`) and times every step separately, so a
//! step's wall time can be regressed on its position: the slope is the cost
//! that grows with the cached positions (the reads' and the pointer's scans),
//! the intercept the per-step constant (weight maps, recurrence, norms). Each
//! prefill saves its session at `hot_from`. After every prefill, the tool
//! writes `HOT-PHASE-START` to standard error and sleeps `hot_pause_ms`; then,
//! `hot_repeats` times per request, it restores the saved session and steps
//! the rest of the history up to the context, and writes `HOT-PHASE-END`. A
//! sampling profiler started at the first marker therefore sees only
//! full-context steps (at every thread of the pool). Every report root is
//! claimed exclusively and sealed.
#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde_json::{json, Value};
use uor_r4_core::report_output;
use uor_r4_tokenizer::dialogue::DialogueProtocol;
use uor_r4_training::stack_dialogue::{load_requests, Request};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn arg<'a>(args: &'a [(String, String)], key: &str) -> Option<&'a str> {
    args.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
}

fn required<'a>(args: &'a [(String, String)], key: &str) -> Result<&'a str> {
    arg(args, key).ok_or_else(|| format!("missing {key}=").into())
}

fn number<T: std::str::FromStr>(args: &[(String, String)], key: &str, default: T) -> Result<T> {
    match arg(args, key) {
        None => Ok(default),
        Some(text) => text
            .parse()
            .map_err(|_| format!("invalid {key}={text}").into()),
    }
}

fn parse(arguments: &[String], allowed: &[&str]) -> Result<Vec<(String, String)>> {
    let mut out = Vec::new();
    for item in arguments {
        let (key, value) = item
            .split_once('=')
            .ok_or_else(|| format!("expected key=value, got {item}"))?;
        if !allowed.contains(&key) {
            return Err(format!("unknown option {key}= (allowed: {})", allowed.join(", ")).into());
        }
        out.push((key.to_owned(), value.to_owned()));
    }
    Ok(out)
}

fn tokenizer(path: &Path) -> Result<uor_r4_tokenizer::ByteBpeTokenizer> {
    uor_r4_tokenizer::ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(path)?)
        .ok_or_else(|| "unreadable tokenizer.json".into())
}

/// One long single-turn request built from consecutive panel user turns.
struct LongPrompt {
    request: Request,
    sources: Vec<String>,
    history: usize,
}

/// How far past the first unused turn a prompt looks for one that still fits.
const LOOKAHEAD: usize = 64;

/// Concatenates unused `turns` (id, text), in order, into one user turn whose
/// history (BOS + user prefix) stays within `limit` ids. Once the next unused
/// turn no longer fits, later unused turns (within [`LOOKAHEAD`]) that still
/// fit are appended, so the history ends close to the limit. Marks the turns
/// it takes in `used`.
fn long_prompt(
    encoder: &uor_r4_tokenizer::dialogue::DialogueEncoder<'_>,
    turns: &[(String, String)],
    used: &mut [bool],
    limit: usize,
    index: usize,
) -> Result<LongPrompt> {
    let mut text = String::new();
    let mut sources = Vec::new();
    let mut history = 0usize;
    while let Some(first) = used.iter().position(|&u| !u) {
        let mut taken = false;
        for at in first..turns.len().min(first + LOOKAHEAD) {
            if used[at] {
                continue;
            }
            let candidate = if text.is_empty() {
                turns[at].1.clone()
            } else {
                format!("{text} {}", turns[at].1)
            };
            let prefix = encoder.encode_user_prefix(&candidate, false);
            if prefix.emitted_turns != 1 || prefix.special_token_occurrences != 0 {
                continue;
            }
            let length = 1 + prefix.tokens.len();
            if length > limit {
                continue;
            }
            text = candidate;
            history = length;
            sources.push(turns[at].0.clone());
            used[at] = true;
            taken = true;
            break;
        }
        if !taken {
            break;
        }
    }
    if text.is_empty() {
        return Err("the panels ran out before a prompt could be built".into());
    }
    Ok(LongPrompt {
        request: Request {
            id: format!("fill-{index:02}"),
            category: "context-fill".to_owned(),
            user_turns: vec![text],
        },
        sources,
        history,
    })
}

fn prompts_mode(arguments: &[String]) -> Result<()> {
    let args = parse(
        arguments,
        &[
            "tokenizer",
            "panels",
            "out",
            "context",
            "max_new_tokens",
            "count",
        ],
    )?;
    let tokenizer_path = PathBuf::from(required(&args, "tokenizer")?);
    let panels: Vec<PathBuf> = required(&args, "panels")?
        .split(',')
        .map(PathBuf::from)
        .collect();
    let out = PathBuf::from(required(&args, "out")?);
    let context: usize = number(&args, "context", 384)?;
    let max_new_tokens: usize = number(&args, "max_new_tokens", 4)?;
    let count: usize = number(&args, "count", 4)?;
    if max_new_tokens == 0 || max_new_tokens >= context || count == 0 {
        return Err("need 0 < max_new_tokens < context and count > 0".into());
    }
    report_output::claim(&out)?;
    let result = (|| -> Result<()> {
        let tokenizer = tokenizer(&tokenizer_path)?;
        let protocol = DialogueProtocol::literal_roles_version(&tokenizer, 2)?;
        let encoder = protocol.bind(&tokenizer)?;
        let mut turns = Vec::new();
        let mut panel_ids = Vec::new();
        for panel in &panels {
            panel_ids.push(json!({"path": panel, "sha256": sha256(panel)?}));
            for request in load_requests(panel)? {
                for (turn, text) in request.user_turns.iter().enumerate() {
                    turns.push((format!("{}#{}", request.id, turn + 1), text.clone()));
                }
            }
        }
        let limit = context - max_new_tokens;
        let mut used = vec![false; turns.len()];
        let mut prompts = Vec::new();
        for index in 0..count {
            prompts.push(long_prompt(&encoder, &turns, &mut used, limit, index + 1)?);
        }
        let requests: Vec<&Request> = prompts.iter().map(|p| &p.request).collect();
        fs::write(
            out.join("requests.json"),
            serde_json::to_vec_pretty(&requests)?,
        )?;
        fs::write(
            out.join("prompts.json"),
            serde_json::to_vec_pretty(&json!({
                "schema": "uor-r4.step0d-prompts/1",
                "tokenizer": {"path": tokenizer_path, "sha256": sha256(&tokenizer_path)?},
                "panels": panel_ids,
                "protocol_version": 2,
                "context": context,
                "max_new_tokens": max_new_tokens,
                "history_limit": limit,
                "prompts": prompts.iter().map(|p| json!({
                    "id": p.request.id,
                    "history_ids": p.history,
                    "source_turns": p.sources,
                    "characters": p.request.user_turns[0].len(),
                })).collect::<Vec<_>>(),
            }))?,
        )?;
        Ok(())
    })();
    finish(&out, result)
}

fn steps_mode(arguments: &[String]) -> Result<()> {
    let args = parse(
        arguments,
        &[
            "artifact",
            "tokenizer",
            "requests",
            "out",
            "threads",
            "hot_from",
            "hot_repeats",
            "hot_pause_ms",
        ],
    )?;
    let artifact = PathBuf::from(required(&args, "artifact")?);
    let tokenizer_path = PathBuf::from(required(&args, "tokenizer")?);
    let requests_path = PathBuf::from(required(&args, "requests")?);
    let out = PathBuf::from(required(&args, "out")?);
    let threads: usize = number(&args, "threads", 1)?;
    let hot_from: usize = number(&args, "hot_from", 370)?;
    let hot_repeats: usize = number(&args, "hot_repeats", 40)?;
    let hot_pause_ms: u64 = number(&args, "hot_pause_ms", 0)?;
    let requests = load_requests(&requests_path)?;
    report_output::claim(&out)?;
    let result = (|| -> Result<()> {
        let tokenizer = tokenizer(&tokenizer_path)?;
        let protocol = DialogueProtocol::literal_roles_version(&tokenizer, 2)?;
        let encoder = protocol.bind(&tokenizer)?;
        let mut model = uor_r4_integer::stack::IntegerStackModel::load(&artifact)?;
        model.set_threads(threads)?;
        let shape = model.shape().clone();
        let mut rows = Vec::new();
        let mut hot = Vec::new();
        let mut pending = Vec::new();
        for request in &requests {
            let mut history = vec![protocol.bos_id];
            history.extend(
                encoder
                    .encode_user_prefix(&request.user_turns[0], false)
                    .tokens,
            );
            if history.len() > shape.context {
                return Err(format!("{}: history exceeds the context", request.id).into());
            }
            let mut session = model.session();
            let mut nanos = Vec::with_capacity(history.len());
            let mut saved = None;
            let clock = Instant::now();
            for (position, &id) in history.iter().enumerate() {
                if position == hot_from {
                    saved = Some(session.save_state());
                }
                let step = Instant::now();
                session.step(id)?;
                nanos.push(step.elapsed().as_nanos() as u64);
            }
            let prefill_seconds = clock.elapsed().as_secs_f64();
            let (slope, intercept) = fit(&nanos);
            rows.push(json!({
                "id": request.id,
                "history_ids": history.len(),
                "prefill_seconds": prefill_seconds,
                "step_ns": nanos,
                "fit_ns": {"intercept": intercept, "per_position": slope},
            }));
            if let Some(saved) = saved {
                pending.push((request.id.clone(), history, saved));
            }
        }
        // Every prefill is done: announce the hot phase on standard error and
        // pause, so a sampler started now sees only full-context steps.
        eprintln!("HOT-PHASE-START");
        std::thread::sleep(std::time::Duration::from_millis(hot_pause_ms));
        let mut session = model.session();
        for (id, history, saved) in &pending {
            // Hot loop: only positions hot_from..history.len(), many times.
            let mut hot_nanos = vec![0u64; history.len() - hot_from];
            let clock = Instant::now();
            for _ in 0..hot_repeats {
                session.restore_state(saved)?;
                for (k, &token) in history[hot_from..].iter().enumerate() {
                    let step = Instant::now();
                    session.step(token)?;
                    hot_nanos[k] += step.elapsed().as_nanos() as u64;
                }
            }
            hot.push(json!({
                "id": id,
                "from": hot_from,
                "to": history.len(),
                "repeats": hot_repeats,
                "seconds": clock.elapsed().as_secs_f64(),
                "mean_step_ns": hot_nanos.iter().map(|&n| n as f64 / hot_repeats.max(1) as f64)
                    .collect::<Vec<_>>(),
            }));
        }
        eprintln!("HOT-PHASE-END");
        fs::write(
            out.join("steps.json"),
            serde_json::to_vec_pretty(&json!({
                "schema": "uor-r4.step0d-steps/1",
                "artifact": {"path": artifact, "sha256": model.artifact_sha256()},
                "requests": {"path": requests_path, "sha256": sha256(&requests_path)?},
                "shape": {"vocab": shape.vocab, "width": shape.width, "heads": shape.heads,
                    "mlp": shape.mlp, "pattern": shape.pattern, "read": shape.read,
                    "context": shape.context, "pointer": shape.pointer.is_some()},
                "threads": model.threads(),
                "backend": "d11 multiplier-free scalar",
                "rows": rows,
                "hot": hot,
            }))?,
        )?;
        Ok(())
    })();
    finish(&out, result)
}

/// Least-squares line through (position, nanos): (slope, intercept).
fn fit(nanos: &[u64]) -> (f64, f64) {
    let n = nanos.len() as f64;
    if nanos.len() < 2 {
        return (0.0, nanos.first().copied().unwrap_or(0) as f64);
    }
    let mean_x = (n - 1.0) / 2.0;
    let mean_y = nanos.iter().map(|&y| y as f64).sum::<f64>() / n;
    let (mut sxy, mut sxx) = (0.0, 0.0);
    for (x, &y) in nanos.iter().enumerate() {
        let dx = x as f64 - mean_x;
        sxy += dx * (y as f64 - mean_y);
        sxx += dx * dx;
    }
    let slope = sxy / sxx;
    (slope, mean_y - slope * mean_x)
}

fn sha256(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    Ok(hex::encode(Sha256::digest(fs::read(path)?)))
}

fn finish(out: &Path, result: Result<()>) -> Result<()> {
    if let Err(error) = &result {
        let record: Value = json!({"error": error.to_string()});
        fs::write(out.join("error.json"), serde_json::to_vec_pretty(&record)?)?;
    }
    report_output::seal(out)?;
    report_output::verify(out)?;
    result
}

fn main() -> Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.first().map(String::as_str) {
        Some("prompts") => prompts_mode(&arguments[1..]),
        Some("steps") => steps_mode(&arguments[1..]),
        _ => Err("usage: d11-read-share prompts|steps key=value ... (see the file header)".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::fit;

    #[test]
    fn the_fit_recovers_a_line() {
        let nanos: Vec<u64> = (0..100u64).map(|p| 1000 + 7 * p).collect();
        let (slope, intercept) = fit(&nanos);
        assert!((slope - 7.0).abs() < 1e-9);
        assert!((intercept - 1000.0).abs() < 1e-6);
    }

    #[test]
    fn a_single_point_has_no_slope() {
        assert_eq!(fit(&[5]), (0.0, 5.0));
        assert_eq!(fit(&[]), (0.0, 0.0));
    }
}
