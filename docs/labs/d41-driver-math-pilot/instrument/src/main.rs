//! `d41-pilot`: the D41/D86 donor pilot's client.
//!
//!   d41-pilot prompts  --seed S --count N --panels DIR --out FILE
//!   d41-pilot generate --server URL --prompts FILE --out DIR --budget TOKENS --instrument-commit SHA
//!                      [--parallel P] [--seed S] [--max-tokens M] [--shard-records R] [--max-seconds T]
//!                      [--mode seq|n4|g-n3|final] [--tokenize 0|1] [--system-file FILE]
//!   d41-pilot count    --server URL --dir DIR
//!   d41-pilot seal     --dir DIR --model FILE --machine FILE --engine FILE
//!   d41-pilot verify   --dir DIR
//!
//! `generate` asks the external engine (llama.cpp `llama-server`, OpenAI-style chat API)
//! for four answers per prompt: one greedy and three sampled at the donor's recommended
//! temperature 1 and top-p 1, each with its own seed. It keeps only the final answer
//! texts (no reasoning text, no logits, probabilities, scores or ranks are requested or
//! stored), and stops dispatching new prompts once `--budget` generated tokens are counted.

use d41_donor_pilot::{
    canonical_answers, check_record, has_markup, merkle_root, prompts, sha256_hex,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

type Res<T> = Result<T, String>;

fn args_map(args: &[String]) -> Res<BTreeMap<String, String>> {
    let mut m = BTreeMap::new();
    let mut i = 0;
    while i < args.len() {
        let k = args[i]
            .strip_prefix("--")
            .ok_or_else(|| format!("expected --flag, got {}", args[i]))?;
        let v = args
            .get(i + 1)
            .ok_or_else(|| format!("--{k} needs a value"))?;
        m.insert(k.to_string(), v.clone());
        i += 2;
    }
    Ok(m)
}

fn need<'a>(m: &'a BTreeMap<String, String>, k: &str) -> Res<&'a str> {
    m.get(k)
        .map(|s| s.as_str())
        .ok_or_else(|| format!("missing --{k}"))
}

fn num<T: std::str::FromStr>(m: &BTreeMap<String, String>, k: &str, default: T) -> Res<T> {
    match m.get(k) {
        None => Ok(default),
        Some(s) => s.parse().map_err(|_| format!("--{k}: not a number: {s}")),
    }
}

fn read(p: &Path) -> Res<Vec<u8>> {
    fs::read(p).map_err(|e| format!("read {}: {e}", p.display()))
}

fn write(p: &Path, bytes: &[u8]) -> Res<()> {
    fs::write(p, bytes).map_err(|e| format!("write {}: {e}", p.display()))
}

fn read_json(p: &Path) -> Res<Value> {
    serde_json::from_slice(&read(p)?).map_err(|e| format!("parse {}: {e}", p.display()))
}

/// UTC time as `YYYY-MM-DDTHH:MM:SSZ` (civil-from-days, Hinnant).
fn utc_now() -> String {
    let s = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0) as i64;
    let (days, rem) = (s.div_euclid(86_400), s.rem_euclid(86_400));
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if mo <= 2 { 1 } else { 0 };
    format!(
        "{y:04}-{mo:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

fn cmd_prompts(m: &BTreeMap<String, String>) -> Res<()> {
    let seed: u64 = num(m, "seed", 0)?;
    let count: usize = num(m, "count", 0)?;
    let out = PathBuf::from(need(m, "out")?);
    let panels = PathBuf::from(need(m, "panels")?);
    let mut guard = prompts::PanelGuard::new();
    let mut panel_files = 0usize;
    let mut skipped: Vec<String> = Vec::new();
    let mut entries: Vec<PathBuf> = fs::read_dir(&panels)
        .map_err(|e| format!("read_dir {}: {e}", panels.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for p in entries {
        let ext = p.extension().and_then(|x| x.to_str()).unwrap_or("");
        let name = p
            .file_name()
            .and_then(|x| x.to_str())
            .unwrap_or("")
            .to_string();
        if p.is_dir() {
            return Err(format!(
                "{} is a folder: the guard reads files only",
                p.display()
            ));
        }
        if guard.add_file(ext, &String::from_utf8_lossy(&read(&p)?)) {
            panel_files += 1;
        } else {
            skipped.push(name);
        }
    }
    if guard.is_empty() {
        return Err(format!("no panel strings found under {}", panels.display()));
    }
    let (set, stats) = prompts::build(seed, count, &guard);
    if set.len() != count {
        return Err(format!("built {} prompts, wanted {count}", set.len()));
    }
    let mut text = String::new();
    for p in &set {
        let rec = json!({"id": p.id, "kind": p.kind, "messages": [
            {"role": "system", "content": prompts::SYSTEM},
            {"role": "user", "content": p.user}]});
        text.push_str(&rec.to_string());
        text.push('\n');
    }
    write(&out, text.as_bytes())?;
    let mut kinds: BTreeMap<&str, usize> = BTreeMap::new();
    for p in &set {
        *kinds.entry(p.kind).or_default() += 1;
    }
    println!(
        "{}",
        json!({"prompts": set.len(), "seed": seed, "sha256": sha256_hex(text.as_bytes()), "kinds": kinds,
               "panel_files": panel_files, "panel_files_not_read": skipped, "panel_hits_in_set": set.iter().filter(|p| guard.hits(std::slice::from_ref(&p.user))).count(), "panel_strings": guard.len(),
               "duplicates_skipped": stats.duplicates_skipped, "panel_hits_dropped": stats.panel_hits_dropped})
    );
    Ok(())
}

/// One answer's outcome (one choice of a reply).
struct Answer {
    text: Option<String>,
    truncated: bool,
    markup: bool,
}

fn post(agent: &ureq::Agent, url: &str, body: &Value) -> Res<Value> {
    let mut last = String::new();
    for attempt in 0..4 {
        if attempt > 0 {
            std::thread::sleep(Duration::from_secs(2 * attempt));
        }
        match agent.post(url).send_json(body) {
            Ok(mut r) => match r.body_mut().read_json::<Value>() {
                Ok(v) => return Ok(v),
                Err(e) => last = format!("body: {e}"),
            },
            Err(e) => last = format!("{e}"),
        }
    }
    Err(format!("POST {url} failed 4 times: {last}"))
}

/// One HTTP request's outcome: its `n` answers (one per choice) and its token usage.
struct Reply {
    answers: Vec<Answer>,
    completion_tokens: u64,
    prompt_tokens: u64,
    millis: u64,
}

/// How a prompt's four answers are requested.
#[derive(Clone, Copy, PartialEq)]
enum Mode {
    /// Four requests: one greedy, then three sampled (the pilot's plan).
    Seq,
    /// One request with `n` = 4, all sampled at temperature 1 (no greedy answer).
    N4,
    /// One greedy request, then one request with `n` = 3 sampled at temperature 1.
    GreedyN3,
    /// The pilot's four requests, sent raw to `/completion` with the assistant turn
    /// pre-filled up to the final channel, so the analysis channel is skipped.
    Final,
}

/// The pre-fill `final` mode appends to the rendered prompt, which ends at the assistant turn.
const FINAL_PREFILL: &str = "<|channel|>final<|message|>";
const ASSISTANT_START: &str = "<|start|>assistant";

impl Mode {
    fn parse(s: &str) -> Res<Mode> {
        match s {
            "seq" => Ok(Mode::Seq),
            "n4" => Ok(Mode::N4),
            "g-n3" => Ok(Mode::GreedyN3),
            "final" => Ok(Mode::Final),
            other => Err(format!("--mode {other}: expected seq, n4, g-n3 or final")),
        }
    }
    fn name(self) -> &'static str {
        match self {
            Mode::Seq => "seq",
            Mode::N4 => "n4",
            Mode::GreedyN3 => "g-n3",
            Mode::Final => "final",
        }
    }
    /// The requests for prompt index `idx`: (temperature, n, seed offset from base + 4*idx).
    /// llama.cpp seeds child completion k of an `n` request with seed + k, so `g-n3` gives
    /// the same seed per answer as `seq`, and `n4` the same seeds with alt 0 sampled.
    fn plan(self) -> &'static [(u64, u64, u64)] {
        match self {
            Mode::Seq | Mode::Final => &[(0, 1, 0), (1, 1, 1), (1, 1, 2), (1, 1, 3)],
            Mode::N4 => &[(1, 4, 0)],
            Mode::GreedyN3 => &[(0, 1, 0), (1, 3, 1)],
        }
    }
}

fn ask(
    agent: &ureq::Agent,
    server: &str,
    messages: &Value,
    temperature: u64,
    n: u64,
    seed: u32,
    max_tokens: u64,
) -> Res<Reply> {
    // JSON integers: the engine reads them as the same sampling values (0 greedy, 1 as released).
    let mut body = json!({"model": "donor", "messages": messages, "max_tokens": max_tokens,
        "temperature": temperature, "top_p": 1, "top_k": 0, "min_p": 0, "seed": seed,
        "chat_template_kwargs": {"reasoning_effort": "low"}, "stream": false});
    if n > 1 {
        body["n"] = json!(n);
    }
    let t = Instant::now();
    let v = post(agent, &format!("{server}/v1/chat/completions"), &body)?;
    let millis = t.elapsed().as_millis() as u64;
    let choices = v["choices"].as_array().cloned().unwrap_or_default();
    if choices.len() as u64 != n {
        return Err(format!("asked for {n} choices, got {}", choices.len()));
    }
    let answers = choices
        .iter()
        .map(|choice| {
            let content = choice["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let truncated = choice["finish_reason"].as_str() != Some("stop");
            let markup = has_markup(&content);
            let keep = !truncated && !markup && !content.trim().is_empty();
            Answer {
                text: keep.then_some(content),
                truncated,
                markup,
            }
        })
        .collect();
    Ok(Reply {
        answers,
        completion_tokens: v["usage"]["completion_tokens"].as_u64().unwrap_or(0),
        prompt_tokens: v["usage"]["prompt_tokens"].as_u64().unwrap_or(0),
        millis,
    })
}

/// The engine's own rendering of the chat prompt (the same body `ask` sends), with the
/// assistant turn pre-filled up to the final channel.
fn render_final(agent: &ureq::Agent, server: &str, messages: &Value) -> Res<String> {
    let body = json!({"model": "donor", "messages": messages,
        "chat_template_kwargs": {"reasoning_effort": "low"}});
    let v = post(agent, &format!("{server}/apply-template"), &body)?;
    let prompt = v["prompt"]
        .as_str()
        .ok_or("apply-template: no prompt string")?;
    if !prompt.ends_with(ASSISTANT_START) {
        return Err(
            "apply-template: the rendered prompt does not end at the assistant turn".into(),
        );
    }
    Ok(format!("{prompt}{FINAL_PREFILL}"))
}

/// One raw `/completion` request on a pre-filled prompt, with the same sampling values as `ask`.
/// An answer is finished when the engine stopped on an end-of-generation token or a stop word.
fn ask_raw(
    agent: &ureq::Agent,
    server: &str,
    prompt: &str,
    temperature: u64,
    seed: u32,
    max_tokens: u64,
) -> Res<Reply> {
    let body = json!({"prompt": prompt, "n_predict": max_tokens,
        "temperature": temperature, "top_p": 1, "top_k": 0, "min_p": 0, "seed": seed,
        "cache_prompt": true, "stream": false});
    let t = Instant::now();
    let v = post(agent, &format!("{server}/completion"), &body)?;
    let millis = t.elapsed().as_millis() as u64;
    let content = v["content"].as_str().unwrap_or("").to_string();
    let stop = v["stop_type"].as_str().unwrap_or("none");
    let truncated = stop != "eos" && stop != "word";
    let markup = has_markup(&content);
    let keep = !truncated && !markup && !content.trim().is_empty();
    Ok(Reply {
        answers: vec![Answer {
            text: keep.then_some(content),
            truncated,
            markup,
        }],
        completion_tokens: v["tokens_predicted"].as_u64().unwrap_or(0),
        prompt_tokens: v["tokens_evaluated"].as_u64().unwrap_or(0),
        millis,
    })
}

fn count_tokens(agent: &ureq::Agent, server: &str, text: &str) -> Res<u64> {
    let v = post(
        agent,
        &format!("{server}/tokenize"),
        &json!({"content": text}),
    )?;
    v["tokens"]
        .as_array()
        .map(|a| a.len() as u64)
        .ok_or_else(|| "tokenize: no tokens array".into())
}

/// The prompt's messages with the system message's content replaced by `text`.
fn with_system(messages: &Value, text: &str) -> Value {
    let mut out = messages.clone();
    if let Some(arr) = out.as_array_mut() {
        for msg in arr.iter_mut() {
            if msg["role"] == "system" {
                msg["content"] = json!(text);
            }
        }
    }
    out
}

#[derive(Default)]
struct Tally {
    requests: u64,
    answers_returned: u64,
    truncated: u64,
    markup: u64,
    tokenize_errors: u64,
    empty_prompts: u64,
    kept_tokens: u64,
    kept_bytes: u64,
    prompt_tokens: u64,
    distinct: [u64; 5],
    millis: Vec<u64>,
    by_kind: BTreeMap<String, (u64, u64)>,
    window_generated: u64,
    window_prompts: u64,
    window_kept_tokens: u64,
    window_ids: Vec<Value>,
}

fn cmd_generate(m: &BTreeMap<String, String>) -> Res<()> {
    let server = need(m, "server")?.trim_end_matches('/').to_string();
    let prompts_path = PathBuf::from(need(m, "prompts")?);
    let out = PathBuf::from(need(m, "out")?);
    let budget: u64 = num(m, "budget", 0)?;
    let parallel: usize = num(m, "parallel", 32)?;
    let seed: u64 = num(m, "seed", 0)?;
    let max_tokens: u64 = num(m, "max-tokens", 1024)?;
    let shard_records: usize = num(m, "shard-records", 512)?;
    // The sheet's hard stop: no new prompt is dispatched after this many seconds.
    let max_seconds: u64 = num(m, "max-seconds", 10_800)?;
    let window_ms = max_seconds * 1000;
    if budget == 0 || parallel == 0 || shard_records == 0 {
        return Err("--budget, --parallel and --shard-records must be positive".into());
    }
    // The client commit goes into run.json, and so into the receipt.
    let instrument_commit = need(m, "instrument-commit")?.to_string();
    if instrument_commit.len() != 40 || !instrument_commit.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("--instrument-commit must be a full 40-hex commit".into());
    }
    let mode = Mode::parse(m.get("mode").map(String::as_str).unwrap_or("seq"))?;
    // --tokenize 0 skips the per-answer /tokenize call; `count` then tokenizes afterwards.
    let tokenize = num::<u64>(m, "tokenize", 1)? != 0;
    // --system-file FILE replaces the system message with the file's text, trailing newline trimmed.
    let system: Option<String> = match m.get("system-file") {
        Some(p) => Some(
            String::from_utf8_lossy(&read(Path::new(p))?)
                .trim_end()
                .to_string(),
        ),
        None => None,
    };
    fs::create_dir_all(&out).map_err(|e| format!("mkdir {}: {e}", out.display()))?;
    let prompt_bytes = read(&prompts_path)?;
    let set: Vec<Value> = String::from_utf8_lossy(&prompt_bytes)
        .lines()
        .map(|l| serde_json::from_str(l).map_err(|e| format!("prompt line: {e}")))
        .collect::<Res<_>>()?;
    let set = Arc::new(set);
    let next = Arc::new(AtomicUsize::new(0));
    let generated = Arc::new(AtomicU64::new(0));
    let results: Arc<Mutex<BTreeMap<usize, Value>>> = Arc::new(Mutex::new(BTreeMap::new()));
    let tally = Arc::new(Mutex::new(Tally::default()));
    let errors: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let start_utc = utc_now();
    let t0 = Instant::now();
    let mut handles = Vec::new();
    for _ in 0..parallel {
        let (set, next, generated, results, tally, errors, server, system) = (
            set.clone(),
            next.clone(),
            generated.clone(),
            results.clone(),
            tally.clone(),
            errors.clone(),
            server.clone(),
            system.clone(),
        );
        handles.push(std::thread::spawn(move || {
            let agent: ureq::Agent =
                ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(900))).build().into();
            loop {
                if generated.load(Ordering::SeqCst) >= budget || t0.elapsed().as_secs() >= max_seconds {
                    break;
                }
                let idx = next.fetch_add(1, Ordering::SeqCst);
                if idx >= set.len() {
                    break;
                }
                let p = &set[idx];
                let messages = match &system {
                    Some(text) => with_system(&p["messages"], text),
                    None => p["messages"].clone(),
                };
                let mut texts = Vec::new();
                let mut failed = None;
                // `final` mode: render the prompt once, then pre-fill the final channel.
                let raw = if mode == Mode::Final {
                    match render_final(&agent, &server, &messages) {
                        Ok(r) => Some(r),
                        Err(e) => {
                            errors.lock().unwrap_or_else(|e| e.into_inner()).push(format!("{idx}: {e}"));
                            continue;
                        }
                    }
                } else {
                    None
                };
                for &(temperature, n, off) in mode.plan() {
                    let s = (seed.wrapping_add(idx as u64 * 4 + off) & 0xFFFF_FFFF) as u32;
                    let reply = match &raw {
                        Some(prompt) => ask_raw(&agent, &server, prompt, temperature, s, max_tokens),
                        None => ask(&agent, &server, &messages, temperature, n, s, max_tokens),
                    };
                    match reply {
                        Ok(r) => {
                            generated.fetch_add(r.completion_tokens, Ordering::SeqCst);
                            let mut t = tally.lock().unwrap_or_else(|e| e.into_inner());
                            t.requests += 1;
                            t.answers_returned += r.answers.len() as u64;
                            t.truncated += r.answers.iter().filter(|a| a.truncated).count() as u64;
                            t.markup += r.answers.iter().filter(|a| a.markup).count() as u64;
                            t.prompt_tokens += r.prompt_tokens;
                            t.millis.push(r.millis);
                            if t0.elapsed().as_millis() as u64 <= window_ms {
                                t.window_generated += r.completion_tokens;
                            }
                            let kind = p["kind"].as_str().unwrap_or("?").to_string();
                            let row = t.by_kind.entry(kind).or_default();
                            row.1 += r.completion_tokens;
                            drop(t);
                            texts.extend(r.answers.into_iter().filter_map(|a| a.text));
                        }
                        Err(e) => {
                            failed = Some(e);
                            break;
                        }
                    }
                }
                if let Some(e) = failed {
                    errors.lock().unwrap_or_else(|e| e.into_inner()).push(format!("{idx}: {e}"));
                    continue;
                }
                let answers = canonical_answers(&texts);
                let mut kept_tokens = 0u64;
                let mut tokenize_failed = 0u64;
                if tokenize {
                    for a in &answers {
                        match count_tokens(&agent, &server, a) {
                            Ok(n) => kept_tokens += n,
                            Err(_) => tokenize_failed += 1,
                        }
                    }
                }
                let mut t = tally.lock().unwrap_or_else(|e| e.into_inner());
                t.distinct[answers.len()] += 1;
                t.kept_tokens += kept_tokens;
                t.tokenize_errors += tokenize_failed;
                t.kept_bytes += answers.iter().map(|a| a.len() as u64).sum::<u64>();
                // The bar's rate counts only prompts whose record was complete inside the
                // dispatch window, so the in-flight tail after it does not enter the rate.
                if t0.elapsed().as_millis() as u64 <= window_ms {
                    t.window_prompts += 1;
                    t.window_kept_tokens += kept_tokens;
                    if !answers.is_empty() {
                        t.window_ids.push(p["id"].clone());
                    }
                }
                let kind = p["kind"].as_str().unwrap_or("?").to_string();
                let row = t.by_kind.entry(kind).or_default();
                row.0 += 1;
                if answers.is_empty() {
                    t.empty_prompts += 1;
                    continue;
                }
                drop(t);
                let rec = json!({"id": p["id"], "kind": p["kind"], "messages": messages, "answers": answers});
                results.lock().unwrap_or_else(|e| e.into_inner()).insert(idx, rec);
            }
        }));
    }
    for h in handles {
        h.join()
            .map_err(|_| "a worker thread panicked".to_string())?;
    }
    let wall_ms = (t0.elapsed().as_millis() as u64).max(1);
    let end_utc = utc_now();
    let dispatched = next.load(Ordering::SeqCst).min(set.len());
    let results = results.lock().unwrap_or_else(|e| e.into_inner());
    let errors = errors.lock().unwrap_or_else(|e| e.into_inner());
    let mut t = tally.lock().unwrap_or_else(|e| e.into_inner());

    // Shards, in prompt order.
    let recs: Vec<&Value> = results.values().collect();
    let mut shard_count = 0usize;
    for (i, chunk) in recs.chunks(shard_records).enumerate() {
        let mut text = String::new();
        for r in chunk {
            text.push_str(&r.to_string());
            text.push('\n');
        }
        write(&out.join(format!("shard-{i:04}.jsonl")), text.as_bytes())?;
        shard_count += 1;
    }
    t.millis.sort_unstable();
    // Nearest-rank quantile at `pct` percent, in integer milliseconds.
    let q = |pct: usize| {
        let n = t.millis.len();
        t.millis
            .get((n.saturating_sub(1) * pct + 50) / 100)
            .copied()
    };
    let gen = generated.load(Ordering::SeqCst);
    // Rates in thousandths of a token per second, so every recorded number is an integer.
    let metrics = json!({
        "wall_ms": wall_ms, "prompts_dispatched": dispatched, "records_kept": recs.len(),
        "requests": t.requests, "answers_returned": t.answers_returned, "kept_answers_counted": tokenize,
        "generated_tokens": gen, "prompt_tokens": t.prompt_tokens,
        "generated_millitokens_per_second": gen * 1_000_000 / wall_ms,
        "kept_answer_tokens": t.kept_tokens, "kept_answer_bytes": t.kept_bytes,
        "kept_answer_millitokens_per_second": t.kept_tokens * 1_000_000 / wall_ms,
        "window": {"ms": window_ms, "tail_ms": wall_ms.saturating_sub(window_ms),
            "rule": "counts requests and prompts that finished within ms of the start",
            "prompts_finished": t.window_prompts, "generated_tokens": t.window_generated,
            "kept_answer_tokens": t.window_kept_tokens,
            "generated_millitokens_per_second": t.window_generated * 1_000_000 / window_ms.max(1),
            "kept_answer_millitokens_per_second": t.window_kept_tokens * 1_000_000 / window_ms.max(1)},
        "truncated_answers": t.truncated, "markup_answers": t.markup, "tokenize_errors": t.tokenize_errors, "prompts_with_no_answer": t.empty_prompts,
        "prompts_by_distinct_answers": {"0": t.distinct[0], "1": t.distinct[1], "2": t.distinct[2], "3": t.distinct[3], "4": t.distinct[4]},
        "request_ms": {"p50": q(50), "p95": q(95), "max": t.millis.last()},
        "by_kind": t.by_kind.iter().map(|(k, (n, g))| (k.clone(), json!({"prompts": n, "generated_tokens": g}))).collect::<BTreeMap<_, _>>(),
        "errors": errors.len(), "error_sample": errors.iter().take(5).collect::<Vec<_>>(),
        "shards": shard_count,
    });
    let run = json!({
        "utc_start": start_utc, "utc_end": end_utc,
        "prompts_file": prompts_path.file_name().and_then(|s| s.to_str()), "prompts_sha256": sha256_hex(&prompt_bytes),
        "prompts_in_set": set.len(), "prompts_dispatched": dispatched,
        "stop_rule": format!("stop dispatching once generated tokens >= {budget} or {max_seconds} s have passed; finish prompts in flight"), "stopped_by_time": wall_ms >= max_seconds * 1000,
        "instrument_commit": instrument_commit,
        "parallel_requests": parallel, "shard_records": shard_records,
        "request_mode": mode.name(),
        "requests_per_prompt": mode.plan().iter().map(|&(t, n, off)| json!({"temperature": t, "n": n, "seed_offset": off})).collect::<Vec<_>>(),
        "system_message_replaced": system.is_some(), "system_message": system,
        "sampling": {
            "answers_per_prompt": 4,
            "alt_0": if mode == Mode::N4 { json!({"temperature": 1, "note": "sampled: n4 mode has no greedy answer"}) } else { json!({"temperature": 0, "note": "greedy"}) },
            "alt_1_to_3": {"temperature": 1, "top_p": 1, "top_k": 0, "min_p": 0},
            "n_seed_rule": "llama.cpp seeds child completion k (k = 1..n-1) of an n request with seed + k",
            "max_tokens": max_tokens, "reasoning_effort": "low",
            "seed_rule": format!("seed(alt a of prompt index i) = ({seed} + 4*i + a) mod 2^32"),
            "base_seed": seed,
            "kept": "final-channel answer texts with finish_reason stop and no channel markup; distinct, trimmed, at most 4, in byte order",
            "not_requested_or_stored": ["logits", "logprobs", "probabilities", "scores", "ranks", "reasoning text"],
        },
    });
    write(
        &out.join("metrics.json"),
        serde_json::to_string_pretty(&metrics)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )?;
    write(
        &out.join("run.json"),
        serde_json::to_string_pretty(&run)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )?;
    // The ids of the kept records that finished inside the window, for `count`.
    let window = json!({"ms": window_ms, "ids": t.window_ids});
    write(&out.join("window.json"), window.to_string().as_bytes())?;
    println!("{metrics}");
    if recs.is_empty() {
        return Err("no record kept".into());
    }
    Ok(())
}

/// `count`: tokenize the kept answers of a finished `generate` directory, outside its
/// timed window, for a run made with `--tokenize 0`. Writes `count.json`.
fn cmd_count(m: &BTreeMap<String, String>) -> Res<()> {
    let server = need(m, "server")?.trim_end_matches('/').to_string();
    let dir = PathBuf::from(need(m, "dir")?);
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(120)))
        .build()
        .into();
    // Prompts that finished inside the generate window (window.json), when present.
    let window_ids: Option<Vec<Value>> = match read_json(&dir.join("window.json")) {
        Ok(w) => w["ids"].as_array().cloned(),
        Err(_) => None,
    };
    let (mut records, mut answers, mut tokens, mut bytes, mut errors, mut window_tokens) =
        (0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
    for p in shard_paths(&dir)? {
        for line in String::from_utf8_lossy(&read(&p)?).lines() {
            let v: Value =
                serde_json::from_str(line).map_err(|e| format!("{}: {e}", p.display()))?;
            records += 1;
            let in_window = window_ids
                .as_ref()
                .is_some_and(|ids| ids.contains(&v["id"]));
            for a in v["answers"].as_array().into_iter().flatten() {
                let text = a.as_str().ok_or("an answer is not a string")?;
                answers += 1;
                bytes += text.len() as u64;
                match count_tokens(&agent, &server, text) {
                    Ok(n) => {
                        tokens += n;
                        if in_window {
                            window_tokens += n;
                        }
                    }
                    Err(_) => errors += 1,
                }
            }
        }
    }
    let out = json!({"records": records, "answers": answers, "kept_answer_tokens": tokens,
        "kept_answer_bytes": bytes, "tokenize_errors": errors,
        "window_kept_answer_tokens": window_ids.as_ref().map(|_| window_tokens)});
    write(
        &dir.join("count.json"),
        serde_json::to_string_pretty(&out)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )?;
    println!("{out}");
    Ok(())
}

fn shard_paths(dir: &Path) -> Res<Vec<PathBuf>> {
    let mut v: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("read_dir {}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|n| n.starts_with("shard-") && n.ends_with(".jsonl"))
        })
        .collect();
    v.sort();
    Ok(v)
}

fn cmd_seal(m: &BTreeMap<String, String>) -> Res<()> {
    let dir = PathBuf::from(need(m, "dir")?);
    let run = read_json(&dir.join("run.json"))?;
    let metrics_bytes = read(&dir.join("metrics.json"))?;
    let metrics: Value =
        serde_json::from_slice(&metrics_bytes).map_err(|e| format!("parse metrics.json: {e}"))?;
    let prompts_name = run["prompts_file"]
        .as_str()
        .ok_or("run.json: no prompts_file")?;
    let prompt_bytes = read(&dir.join(prompts_name))?;
    if sha256_hex(&prompt_bytes) != run["prompts_sha256"].as_str().unwrap_or("") {
        return Err("the prompts file in DIR is not the one generate read".into());
    }
    let mut shards = Vec::new();
    let mut leaves = Vec::new();
    let mut records = 0usize;
    for p in shard_paths(&dir)? {
        let b = read(&p)?;
        let h = sha256_hex(&b);
        let n = b.iter().filter(|&&c| c == b'\n').count();
        records += n;
        leaves.push(h.clone());
        shards.push(json!({"file": p.file_name().and_then(|s| s.to_str()), "sha256": h, "bytes": b.len(), "records": n}));
    }
    let receipt = json!({
        "receipt": "D41 item 2b generation receipt, as amended by D86",
        "corpus": "distill-d1 pilot (d41-driver-math-pilot, claude-390)",
        "donor": read_json(Path::new(need(m, "model")?))?,
        "engine": read_json(Path::new(need(m, "engine")?))?,
        "machine": read_json(Path::new(need(m, "machine")?))?,
        "run": run,
        "prompt_set": {"file": prompts_name, "sha256": sha256_hex(&prompt_bytes)},
        "metrics": {"file": "metrics.json", "sha256": sha256_hex(&metrics_bytes), "content": metrics},
        "consumption_terms": "D41 item 4 as amended by D86: each answer is a hard target (its tokens are integer targets). No logit, probability, score or rank is stored. Answers are in byte order, so position means nothing; no consumer may weight a prompt or an answer by its position or by how many distinct answers the prompt has.",
        "shards": shards, "records": records,
        "merkle": {"root": merkle_root(&leaves)?,
                   "rule": "leaves are the shard files' sha256 in file order; parent = sha256(left || right) over raw bytes; an unpaired last node moves up unchanged"},
        "sealed_utc": utc_now(),
    });
    write(
        &dir.join("receipt.json"),
        serde_json::to_string_pretty(&receipt)
            .map_err(|e| e.to_string())?
            .as_bytes(),
    )?;
    println!(
        "{}",
        json!({"records": records, "merkle_root": receipt["merkle"]["root"]})
    );
    Ok(())
}

fn cmd_verify(m: &BTreeMap<String, String>) -> Res<()> {
    let dir = PathBuf::from(need(m, "dir")?);
    let r = read_json(&dir.join("receipt.json"))?;
    let pfile = r["prompt_set"]["file"]
        .as_str()
        .ok_or("receipt: no prompt file")?;
    let pbytes = read(&dir.join(pfile))?;
    if sha256_hex(&pbytes) != r["prompt_set"]["sha256"].as_str().unwrap_or("") {
        return Err("prompt set sha256 differs from the receipt".into());
    }
    if sha256_hex(&read(&dir.join("metrics.json"))?)
        != r["metrics"]["sha256"].as_str().unwrap_or("")
    {
        return Err("metrics.json sha256 differs from the receipt".into());
    }
    let mut prompt_msgs: BTreeMap<String, Value> = BTreeMap::new();
    for l in String::from_utf8_lossy(&pbytes).lines() {
        let v: Value = serde_json::from_str(l).map_err(|e| format!("prompt line: {e}"))?;
        prompt_msgs.insert(
            v["id"].as_str().unwrap_or("").to_string(),
            v["messages"].clone(),
        );
    }
    let listed = r["shards"].as_array().ok_or("receipt: no shards")?;
    let on_disk: Vec<String> = shard_paths(&dir)?
        .iter()
        .filter_map(|p| p.file_name().and_then(|s| s.to_str()).map(str::to_string))
        .collect();
    let names: Vec<String> = listed
        .iter()
        .filter_map(|s| s["file"].as_str().map(str::to_string))
        .collect();
    if names != on_disk {
        return Err(format!(
            "shard files {on_disk:?} differ from the receipt's {names:?}"
        ));
    }
    let mut leaves = Vec::new();
    let mut records = 0usize;
    let mut answers = 0usize;
    for s in listed {
        let name = s["file"].as_str().unwrap_or("");
        let b = read(&dir.join(name))?;
        let h = sha256_hex(&b);
        if h != s["sha256"].as_str().unwrap_or("") {
            return Err(format!("{name}: sha256 {h} differs from the receipt"));
        }
        leaves.push(h);
        for (i, l) in String::from_utf8_lossy(&b).lines().enumerate() {
            let v: Value = serde_json::from_str(l).map_err(|e| format!("{name}:{}: {e}", i + 1))?;
            check_record(&v).map_err(|e| format!("{name}:{}: {e}", i + 1))?;
            let id = v["id"].as_str().unwrap_or("");
            if prompt_msgs.get(id) != Some(&v["messages"]) {
                return Err(format!(
                    "{name}:{}: messages differ from prompt {id}",
                    i + 1
                ));
            }
            records += 1;
            answers += v["answers"].as_array().map(|a| a.len()).unwrap_or(0);
        }
    }
    let root = merkle_root(&leaves)?;
    if root != r["merkle"]["root"].as_str().unwrap_or("") {
        return Err(format!("merkle root {root} differs from the receipt"));
    }
    if records as u64 != r["records"].as_u64().unwrap_or(u64::MAX) {
        return Err(format!("{records} records, receipt says {}", r["records"]));
    }
    println!(
        "{}",
        json!({"verified": true, "records": records, "answers": answers, "merkle_root": root})
    );
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let res = match args.split_first() {
        Some((cmd, rest)) => args_map(rest).and_then(|m| match cmd.as_str() {
            "prompts" => cmd_prompts(&m),
            "generate" => cmd_generate(&m),
            "count" => cmd_count(&m),
            "seal" => cmd_seal(&m),
            "verify" => cmd_verify(&m),
            other => Err(format!("unknown command {other}")),
        }),
        None => Err("usage: d41-pilot prompts|generate|count|seal|verify --flag value ...".into()),
    };
    if let Err(e) = res {
        eprintln!("d41-pilot: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each answer's (temperature, seed offset) once `n` requests are expanded, as
    /// llama.cpp seeds child completion k with seed + k.
    fn per_answer(mode: Mode) -> Vec<(u64, u64)> {
        mode.plan()
            .iter()
            .flat_map(|&(t, n, off)| (0..n).map(move |k| (t, off + k)))
            .collect()
    }

    #[test]
    fn every_mode_asks_four_answers_with_the_pilot_seeds() {
        for mode in [Mode::Seq, Mode::N4, Mode::GreedyN3, Mode::Final] {
            let seeds: Vec<u64> = per_answer(mode).iter().map(|a| a.1).collect();
            assert_eq!(seeds, vec![0, 1, 2, 3], "{}", mode.name());
        }
        // g-n3 keeps the pilot's plan answer for answer; n4 samples alt 0.
        assert_eq!(per_answer(Mode::GreedyN3), per_answer(Mode::Seq));
        assert_eq!(per_answer(Mode::Final), per_answer(Mode::Seq));
        assert_eq!(per_answer(Mode::N4)[0], (1, 0));
        assert!(Mode::parse("n5").is_err());
    }

    #[test]
    fn with_system_replaces_only_the_system_message() {
        let m = json!([{"role": "system", "content": "long"}, {"role": "user", "content": "hi"}]);
        let r = with_system(&m, "short");
        assert_eq!(r[0]["content"], "short");
        assert_eq!(r[1], m[1]);
    }
}
