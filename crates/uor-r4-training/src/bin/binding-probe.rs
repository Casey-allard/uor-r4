//! Binding diagnostic for a geometric stack's exact-recall memory rows (#820).
//!
//! ```text
//! binding-probe model=DIR tokenizer=tokenizer.json requests=PANEL.json[,PANEL2.json]
//!   checks=CHECKS.tsv out=NEW_REPORT_ROOT [device=cpu|cuda|metal] [protocol=2]
//!   [max_new_tokens=64] [category=multi_turn_memory] [replies=chat-grade/replies.json]
//! ```
//!
//! Every request of `category` with an `exact` check is answered greedily
//! turn by turn exactly as `chat-grade reply` does ([`reply_panel`] over
//! [`greedy_reply`], the model's own earlier replies in the history). On the
//! last turn the probe observes two queries:
//!
//! * `marker`: the history ending in the assistant marker, before any reply id;
//! * `decision`: the history plus the greedy reply up to (excluding) the first
//!   token of the first expected or distractor value the reply names, i.e. the
//!   position whose next token is that value. A reply naming no value has no
//!   decision query and is classified at the marker.
//!
//! At each query [`StackModel::read_span_probe`] gives every read layer's
//! per-head attention mass on the disjoint source sets of
//! [`binding_probe::SPAN_KINDS`] (expected value, distractor value,
//! distractor key, asked key in earlier user turns, the question turn) and on
//! the whole window (one minus NoRead); the pointer head's gate and attention
//! mass on the same sets; and the decoded distribution's probability of the
//! first token of each expected spelling and distractor value. Rows are
//! classified by the pre-registered rule of [`binding_probe`]. `replies=`
//! optionally cross-checks the regenerated last replies against a
//! `chat-grade reply` record of the same model.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

use candle_core::Device;
use serde_json::{json, Value};
use uor_r4_core::report_output;
use uor_r4_tokenizer::dialogue::DialogueProtocol;
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::binding_probe::{
    asked_key_words, disjoint, nearest_key_gap, parse_exact_checks, phrase_positions, runs,
    swap_runs, token_byte_ranges, words, words_at, Evidence, ExactCheck, MassPattern, MissClass,
    SENSITIVITY, SPAN_KINDS, TAU,
};
use uor_r4_training::geometric_stack::{ReadRowScores, SpanProbe, StackModel};
use uor_r4_training::sha256_file;
use uor_r4_training::stack_dialogue::{greedy_reply, load_requests, reply_panel, Request};

type Error = Box<dyn std::error::Error>;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("binding-probe: {error}");
            ExitCode::FAILURE
        }
    }
}

struct Args(BTreeMap<String, String>);

impl Args {
    fn parse(arguments: &[String], keys: &[&str]) -> Result<Self, Error> {
        let mut pairs = BTreeMap::new();
        for argument in arguments {
            let (key, value) = argument
                .split_once('=')
                .ok_or_else(|| format!("expected key=value, got {argument}"))?;
            if !keys.contains(&key) || pairs.insert(key.to_owned(), value.to_owned()).is_some() {
                return Err(format!("unknown or repeated argument {key}").into());
            }
        }
        Ok(Self(pairs))
    }

    fn required(&self, key: &str) -> Result<String, Error> {
        self.0
            .get(key)
            .cloned()
            .ok_or_else(|| format!("missing {key}=").into())
    }

    fn number<T: std::str::FromStr>(&self, key: &str, default: T) -> Result<T, Error> {
        match self.0.get(key) {
            None => Ok(default),
            Some(v) => v.parse().map_err(|_| format!("invalid {key}={v}").into()),
        }
    }
}

fn device(name: Option<&str>) -> Result<(&'static str, Device), Error> {
    let name = match name {
        None | Some("cpu") => "cpu",
        Some("cuda") => "cuda",
        Some("metal") => "metal",
        Some(other) => return Err(format!("unknown device={other} (cpu, cuda or metal)").into()),
    };
    let device = uor_r4_training::baseline_protocol::device(name)?;
    candle_core::cuda::set_gemm_reduced_precision_f32(false);
    Ok((name, device))
}

/// One probed row's conversation as answered: the user-prefix region of each
/// turn in the final history, the history before the last reply and that reply.
#[derive(Default, Clone)]
struct Answered {
    regions: Vec<std::ops::Range<usize>>,
    history: Vec<u32>,
    reply: Vec<u32>,
    earlier_replies: Vec<String>,
}

fn run() -> Result<(), Error> {
    let started = Instant::now();
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let args = Args::parse(
        &arguments,
        &[
            "model",
            "tokenizer",
            "requests",
            "checks",
            "out",
            "device",
            "protocol",
            "max_new_tokens",
            "category",
            "replies",
            "key_probe",
        ],
    )?;
    let key_layers: Vec<usize> = match args.0.get("key_probe") {
        None => Vec::new(),
        Some(list) => list
            .split(',')
            .map(|v| v.parse().map_err(|_| format!("invalid key_probe={list}")))
            .collect::<Result<_, _>>()?,
    };
    let out = PathBuf::from(args.required("out")?);
    let model_dir = PathBuf::from(args.required("model")?);
    let tokenizer_path = PathBuf::from(args.required("tokenizer")?);
    let checks_path = PathBuf::from(args.required("checks")?);
    let request_paths: Vec<PathBuf> = args
        .required("requests")?
        .split(',')
        .map(PathBuf::from)
        .collect();
    let version: u8 = args.number("protocol", 2)?;
    let max_new_tokens: usize = args.number("max_new_tokens", 64)?;
    let category = args
        .0
        .get("category")
        .cloned()
        .unwrap_or_else(|| "multi_turn_memory".into());
    let replies_path = args.0.get("replies").map(PathBuf::from);
    let (device_label, device) = device(args.0.get("device").map(String::as_str))?;
    report_output::claim(&out)?;
    let result = probe_into(
        &out,
        &model_dir,
        &tokenizer_path,
        &checks_path,
        &request_paths,
        version,
        max_new_tokens,
        &category,
        replies_path.as_deref(),
        &key_layers,
        (device_label, &device),
        started,
    );
    if let Err(error) = &result {
        fs::write(
            out.join("error.json"),
            serde_json::to_vec_pretty(&json!({"error": error.to_string()}))?,
        )?;
    }
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result
}

#[allow(clippy::too_many_arguments)]
fn probe_into(
    out: &Path,
    model_dir: &Path,
    tokenizer_path: &Path,
    checks_path: &Path,
    request_paths: &[PathBuf],
    version: u8,
    max_new_tokens: usize,
    category: &str,
    replies_path: Option<&Path>,
    key_layers: &[usize],
    (device_label, device): (&str, &Device),
    started: Instant,
) -> Result<(), Error> {
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(tokenizer_path)?)
        .ok_or("not a supported tokenizer.json")?;
    let protocol = DialogueProtocol::literal_roles_version(&tokenizer, version)?;
    let encoder = protocol.bind(&tokenizer)?;
    let checks = parse_exact_checks(&fs::read_to_string(checks_path)?)?;
    let mut requests: Vec<Request> = Vec::new();
    for path in request_paths {
        requests.extend(load_requests(path)?);
    }
    let requests: Vec<Request> = requests
        .into_iter()
        .filter(|r| r.category == category && checks.contains_key(&r.id))
        .collect();
    if requests.is_empty() {
        return Err(format!("no {category} request has an exact check").into());
    }
    let model = StackModel::load(model_dir, device)?;
    let schedule: Vec<(usize, usize)> = requests
        .iter()
        .enumerate()
        .flat_map(|(r, request)| (0..request.user_turns.len()).map(move |j| (r, j)))
        .collect();
    let mut answered = vec![Answered::default(); requests.len()];
    let mut call = 0usize;
    let generation = Instant::now();
    let panel = reply_panel(
        &encoder,
        &protocol,
        &requests,
        model.config.context,
        max_new_tokens,
        &|ids| tokenizer.decode(ids),
        &mut |history, cap| {
            let (r, j) = schedule[call];
            call += 1;
            let request = &requests[r];
            let prefix = encoder
                .encode_user_prefix(&request.user_turns[j], j != 0)
                .tokens
                .len();
            answered[r]
                .regions
                .push(history.len() - prefix..history.len());
            let reply = greedy_reply(&model, history, cap, protocol.eos_id)?;
            if j + 1 == request.user_turns.len() {
                answered[r].history = history.to_vec();
                answered[r].reply = reply.ids.clone();
            } else {
                let text: Vec<u32> = reply
                    .ids
                    .iter()
                    .copied()
                    .filter(|&id| id != protocol.eos_id)
                    .collect();
                answered[r].earlier_replies.push(tokenizer.decode(&text));
            }
            Ok(reply)
        },
    )?;
    let generation_seconds = generation.elapsed().as_secs_f64();
    let reference = match replies_path {
        Some(path) => Some(last_replies(&serde_json::from_slice(&fs::read(path)?)?)?),
        None => None,
    };
    let probing = Instant::now();
    let mut rows = Vec::new();
    for (request, done) in requests.iter().zip(&answered) {
        let check = &checks[&request.id];
        rows.push(probe_row(
            &model,
            &tokenizer,
            protocol.eos_id,
            request,
            check,
            done,
            reference.as_ref(),
            key_layers,
        )?);
    }
    let probe_seconds = probing.elapsed().as_secs_f64();
    let summary = summarize(&rows);
    let key_summary = key_probe_summary(&rows, key_layers);
    let report = json!({
        "schema": "uor-r4.binding-probe/1",
        "model": model_dir.display().to_string(),
        "model_sha256": sha256_file(&model_dir.join("model.safetensors")).ok(),
        "parameters": model.parameter_count(),
        "pattern": model.config.pattern,
        "heads": model.config.heads,
        "context": model.config.context,
        "pointer": model.config.pointer.map(|p| json!({"dim": p.dim, "score": format!("{:?}", p.score)})),
        "tokenizer_sha256": sha256_file(tokenizer_path)?,
        "protocol": protocol.schema,
        "checks": {"path": checks_path.display().to_string(), "sha256": sha256_file(checks_path)?},
        "requests": request_paths.iter().map(|p| json!({"path": p.display().to_string(), "sha256": sha256_file(p).ok()})).collect::<Vec<_>>(),
        "category": category,
        "max_new_tokens": max_new_tokens,
        "decoding": "greedy over next_scores (a pointer model's mixture), as chat-grade reply",
        "device": {"device": device_label, "tf32": false},
        "rule": {
            "tau": TAU,
            "sensitivity": SENSITIVITY,
            "mass": "max over read layers x heads and the pointer of attention mass on the span at the decision query",
            "output": "max decoded (mixture) probability of a first token of a spelling",
            "miss": "pattern expected -> c_readout; distractor -> a_binding_swap; neither -> a_binding_swap if o_D > o_E else b_unreached",
            "mapping": "majority a -> Step 7a read-key content; b -> Step 7b exact learned join; c -> copy-gate/readout fix",
        },
        "span_kinds": SPAN_KINDS,
        "summary": summary,
        "key_probe_layers": key_layers,
        "key_probe": key_summary,
        "rows": rows,
        "panel_cost": panel.get("cost"),
        "generation_seconds": generation_seconds,
        "probe_seconds": probe_seconds,
        "executable_sha256": sha256_file(&std::env::current_exe()?)?,
        "wall_seconds": started.elapsed().as_secs_f64(),
    });
    fs::write(out.join("report.json"), serde_json::to_vec_pretty(&report)?)?;
    fs::write(out.join("summary.md"), summary_markdown(&report))?;
    println!("{}", summary_markdown(&report));
    Ok(())
}

/// The last reply of each row of a `chat-grade reply` record, by id.
fn last_replies(record: &Value) -> Result<BTreeMap<String, String>, Error> {
    let rows = record["panel"]["rows"]
        .as_array()
        .or_else(|| record["rows"].as_array())
        .ok_or("replies record without panel rows")?;
    let mut out = BTreeMap::new();
    for row in rows {
        let id = row["id"].as_str().ok_or("row without id")?;
        let reply = row["turns"]
            .as_array()
            .and_then(|turns| turns.last())
            .and_then(|turn| turn["reply"].as_str())
            .ok_or("row without a last reply")?;
        out.insert(id.to_owned(), reply.trim().to_owned());
    }
    Ok(out)
}

/// The phrase's surface spellings whose first token can begin the value.
fn first_token_candidates(tokenizer: &ByteBpeTokenizer, phrase: &[String]) -> Vec<u32> {
    let plain = phrase.join(" ");
    let mut capital = plain.clone();
    if let Some(first) = capital.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    let mut ids = BTreeSet::new();
    for variant in [
        format!(" {plain}"),
        plain.clone(),
        format!(" {capital}"),
        capital,
    ] {
        if let Some(&id) = tokenizer.encode(&variant).first() {
            ids.insert(id);
        }
    }
    ids.into_iter().collect()
}

/// A probed window's decoded text and token byte ranges.
fn window_text(
    tokenizer: &ByteBpeTokenizer,
    ids: &[u32],
) -> (String, Vec<std::ops::Range<usize>>, bool) {
    let pieces: Vec<Vec<u8>> = ids
        .iter()
        .map(|&id| tokenizer.decode_bytes(&[id]))
        .collect();
    let ranges = token_byte_ranges(&pieces);
    let bytes = pieces.concat();
    match String::from_utf8(bytes) {
        Ok(text) => (text, ranges, true),
        Err(error) => (
            String::from_utf8_lossy(error.as_bytes()).into_owned(),
            ranges,
            false,
        ),
    }
}

/// The span sets of a window (see [`SPAN_KINDS`]) plus the whole window.
fn window_spans(
    tokenizer: &ByteBpeTokenizer,
    window: &[u32],
    check: &ExactCheck,
    asked: &[String],
    done: &Answered,
) -> ([Vec<usize>; 5], bool) {
    let (text, tokens, exact_utf8) = window_text(tokenizer, window);
    let all = 0..window.len();
    let union = |terms: &[Vec<String>], region: std::ops::Range<usize>| -> BTreeSet<usize> {
        terms
            .iter()
            .flat_map(|t| phrase_positions(&text, &tokens, region.clone(), t))
            .collect()
    };
    let last = done.regions.len() - 1;
    let mut asked_set = BTreeSet::new();
    for region in &done.regions[..last] {
        for word in asked {
            asked_set.extend(phrase_positions(
                &text,
                &tokens,
                region.clone(),
                std::slice::from_ref(word),
            ));
        }
    }
    let question: BTreeSet<usize> = done.regions[last].clone().collect();
    (
        disjoint([
            union(&check.expected, all.clone()),
            union(&check.forbid, all.clone()),
            union(&check.keys, all),
            asked_set,
            question,
        ]),
        exact_utf8,
    )
}

struct Observation {
    record: Value,
    evidence: Evidence,
}

fn observe(
    model: &StackModel,
    tokenizer: &ByteBpeTokenizer,
    window: &[u32],
    check: &ExactCheck,
    asked: &[String],
    done: &Answered,
) -> Result<Observation, Error> {
    let (spans, exact_utf8) = window_spans(tokenizer, window, check, asked, done);
    let mut sets: Vec<Vec<usize>> = spans.to_vec();
    sets.push((0..window.len()).collect());
    let probe: SpanProbe = model.read_span_probe(window, &sets)?;
    let kinds = SPAN_KINDS.len();
    let mut read_expected = 0f64;
    let mut read_distractor = 0f64;
    let reads: Vec<Value> = probe
        .reads
        .iter()
        .map(|read| {
            let heads: Vec<Value> = read
                .heads
                .iter()
                .map(|masses| {
                    read_expected = read_expected.max(masses[0]);
                    read_distractor = read_distractor.max(masses[1]);
                    let named: f64 = masses[..kinds].iter().sum();
                    let mut head = serde_json::Map::new();
                    for (k, name) in SPAN_KINDS.iter().enumerate() {
                        head.insert((*name).into(), json!(masses[k]));
                    }
                    head.insert("other".into(), json!(masses[kinds] - named));
                    head.insert("no_read".into(), json!(1.0 - masses[kinds]));
                    Value::Object(head)
                })
                .collect();
            json!({"layer": read.layer, "heads": heads})
        })
        .collect();
    let (pointer, pointer_expected, pointer_distractor) = match &probe.pointer {
        None => (Value::Null, None, None),
        Some(head) => {
            let mass = |set: &[usize]| set.iter().map(|&j| head.attention[j]).sum::<f64>();
            let masses: Vec<f64> = spans.iter().map(|s| mass(s)).collect();
            let mut record = serde_json::Map::new();
            record.insert("gate".into(), json!(head.gate));
            for (k, name) in SPAN_KINDS.iter().enumerate() {
                record.insert((*name).into(), json!(masses[k]));
            }
            record.insert("other".into(), json!(1.0 - masses.iter().sum::<f64>()));
            let top = head
                .attention
                .iter()
                .enumerate()
                .fold(
                    0usize,
                    |best, (j, &a)| if a > head.attention[best] { j } else { best },
                );
            record.insert(
                "top_source".into(),
                json!({"position": top, "token": tokenizer.decode(&window[top..=top]), "mass": head.attention[top],
                       "kind": spans.iter().position(|s| s.contains(&top)).map_or("other", |k| SPAN_KINDS[k])}),
            );
            (Value::Object(record), Some(masses[0]), Some(masses[1]))
        }
    };
    let outputs = |terms: &[Vec<String>]| -> (Vec<Value>, f64) {
        let mut best = 0f64;
        let records = terms
            .iter()
            .map(|term| {
                let ids = first_token_candidates(tokenizer, term);
                let p_mix = ids.iter().map(|&id| probe.mixture[id as usize]).fold(0.0, f64::max);
                let p_gen = ids
                    .iter()
                    .map(|&id| probe.generator[id as usize])
                    .fold(0.0, f64::max);
                best = best.max(p_mix);
                json!({"term": term.join(" "), "first_token_ids": ids, "p_mixture": p_mix, "p_generator": p_gen})
            })
            .collect();
        (records, best)
    };
    let (expected_out, out_expected) = outputs(&check.expected);
    let (distractor_out, out_distractor) = outputs(&check.forbid);
    let greedy = probe
        .mixture
        .iter()
        .enumerate()
        .fold(
            0usize,
            |best, (i, &p)| if p > probe.mixture[best] { i } else { best },
        );
    let evidence = Evidence {
        read_expected,
        read_distractor,
        pointer_expected,
        pointer_distractor,
        out_expected,
        out_distractor,
    };
    let record = json!({
        "query": window.len() - 1,
        "window_tokens": window.len(),
        "exact_utf8": exact_utf8,
        "span_sizes": SPAN_KINDS.iter().zip(&spans).map(|(k, s)| (k.to_string(), json!(s.len()))).collect::<serde_json::Map<_, _>>(),
        "reads": reads,
        "pointer": pointer,
        "output": {
            "expected": expected_out,
            "distractor": distractor_out,
            "greedy_token": {"id": greedy, "text": tokenizer.decode(&[greedy as u32]), "p_mixture": probe.mixture[greedy]},
        },
        "evidence": {
            "read_expected_max": read_expected, "read_distractor_max": read_distractor,
            "pointer_expected": pointer_expected, "pointer_distractor": pointer_distractor,
            "out_expected": out_expected, "out_distractor": out_distractor,
        },
    });
    Ok(Observation { record, evidence })
}

/// The reply token index where the first expected or distractor value begins,
/// and which kind it is.
fn decision_index(
    tokenizer: &ByteBpeTokenizer,
    reply: &[u32],
    check: &ExactCheck,
) -> Option<(usize, &'static str, String)> {
    let (text, tokens, _) = window_text(tokenizer, reply);
    let all = words_at(&text);
    for start in 0..all.len() {
        for (kind, terms) in [("expected", &check.expected), ("distractor", &check.forbid)] {
            for term in terms {
                let end = start + term.len();
                if end <= all.len() && all[start..end].iter().map(|w| &w.text).eq(term.iter()) {
                    let byte = all[start].bytes.start;
                    let token = tokens
                        .iter()
                        .position(|t| t.start <= byte && byte < t.end)?;
                    return Some((token, kind, term.join(" ")));
                }
            }
        }
    }
    None
}

fn probe_row(
    model: &StackModel,
    tokenizer: &ByteBpeTokenizer,
    eos: u32,
    request: &Request,
    check: &ExactCheck,
    done: &Answered,
    reference: Option<&BTreeMap<String, String>>,
    key_layers: &[usize],
) -> Result<Value, Error> {
    let text_ids: Vec<u32> = done.reply.iter().copied().filter(|&id| id != eos).collect();
    let reply = tokenizer.decode(&text_ids);
    let passed = check.passes(&reply);
    let turns = &request.user_turns;
    let last = turns.len() - 1;
    let earlier: Vec<&str> = turns[..last].iter().map(String::as_str).collect();
    let asked = asked_key_words(&turns[last], &earlier, check);
    let marker = observe(model, tokenizer, &done.history, check, &asked, done)?;
    let decision = decision_index(tokenizer, &done.reply, check);
    let (at_decision, decided) = match &decision {
        Some((k, _, _)) => {
            let mut window = done.history.clone();
            window.extend(&done.reply[..*k]);
            if window.len() > model.config.context {
                return Err("decision window outgrew the context".into());
            }
            (
                Some(observe(model, tokenizer, &window, check, &asked, done)?),
                true,
            )
        }
        None => (None, false),
    };
    let evidence = at_decision.as_ref().map_or(marker.evidence, |o| o.evidence);
    // The Step 7a key probe at the window the row is classified at.
    let mut classified = done.history.clone();
    if let Some((k, _, _)) = &decision {
        classified.extend(&done.reply[..*k]);
    }
    let key_records = key_layers
        .iter()
        .map(|&layer| key_probe(model, tokenizer, &classified, check, &asked, done, layer))
        .collect::<Result<Vec<_>, _>>()?;
    let pattern = evidence.pattern(TAU);
    let class = (!passed).then(|| evidence.miss_class(TAU));
    let sensitivity: Vec<Value> = SENSITIVITY
        .iter()
        .map(|&tau| {
            json!({"tau": tau, "pattern": evidence.pattern(tau).name(),
                   "miss_class": (!passed).then(|| evidence.miss_class(tau).name())})
        })
        .collect();
    let reply_words = words(&reply);
    let names = |terms: &[Vec<String>]| {
        terms
            .iter()
            .any(|t| uor_r4_training::binding_probe::contains_phrase(&reply_words, t))
    };
    Ok(json!({
        "id": request.id,
        "user_turns": turns,
        "earlier_replies": done.earlier_replies,
        "reply": reply,
        "pass": passed,
        "reply_names": {"expected": names(&check.expected), "distractor": names(&check.forbid), "distractor_key": names(&check.keys)},
        "reference_reply_equal": reference.and_then(|r| r.get(&request.id)).map(|r| r == reply.trim()),
        "expected": check.expected.iter().map(|t| t.join(" ")).collect::<Vec<_>>(),
        "distractor": check.forbid.iter().map(|t| t.join(" ")).collect::<Vec<_>>(),
        "distractor_key": check.keys.iter().map(|t| t.join(" ")).collect::<Vec<_>>(),
        "asked_key": asked,
        "decision": decision.as_ref().map(|(k, kind, term)| json!({"reply_token": k, "value_kind": kind, "term": term})),
        "classified_at": if decided { "decision" } else { "marker" },
        "pattern": pattern.name(),
        "miss_class": class.map(MissClass::name),
        "sensitivity": sensitivity,
        "marker": marker.record,
        "at_decision": at_decision.map(|o| o.record),
        "key_probe": key_records,
    }))
}

fn cosine(a: &[f32], b: &[f32]) -> f64 {
    let dot: f64 = a
        .iter()
        .zip(b)
        .map(|(&x, &y)| f64::from(x) * f64::from(y))
        .sum();
    let na: f64 = a.iter().map(|&x| f64::from(x).powi(2)).sum::<f64>().sqrt();
    let nb: f64 = b.iter().map(|&x| f64::from(x).powi(2)).sum::<f64>().sqrt();
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na * nb)
    }
}

fn distance(a: &[f32], b: &[f32]) -> f64 {
    a.iter()
        .zip(b)
        .map(|(&x, &y)| (f64::from(x) - f64::from(y)).powi(2))
        .sum::<f64>()
        .sqrt()
}

/// The argmax-weight position of `set` in `row` (lowest on a tie).
fn best_in(row: &ReadRowScores, set: &[usize]) -> Option<usize> {
    set.iter()
        .copied()
        .filter(|&j| j < row.weight.len())
        .fold(None, |best: Option<usize>, j| match best {
            Some(b) if row.weight[b] >= row.weight[j] => Some(b),
            _ => Some(j),
        })
}

fn mass_in(row: &ReadRowScores, set: &[usize]) -> f64 {
    set.iter()
        .filter(|&&j| j < row.weight.len())
        .map(|&j| row.weight[j])
        .sum()
}

/// Positions of `set` inside the earlier user turns (`regions[..last]`).
fn in_earlier(set: &[usize], regions: &[std::ops::Range<usize>]) -> Vec<usize> {
    let last = regions.len().saturating_sub(1);
    set.iter()
        .copied()
        .filter(|j| regions[..last].iter().any(|r| r.contains(j)))
        .collect()
}

/// Step 7a Phase 1: why does read `layer` at the classified query prefer one
/// value? For the binding head (the head with the most mass on the two
/// values) it reports, at the best position of each of the expected value,
/// the distractor value, the asked key word and the distractor key word, the
/// read's weight, total score, content and age terms and the query/key
/// cosine; the cosine of the two value positions' keys; the signed token gap
/// from each value to its key word in the earlier user turns; and the
/// counterfactual that exchanges the asked and distractor key words in the
/// earlier user turns (`expected` and `distractor` keep their original
/// labels, so a binding-following read moves its preference to the
/// distractor's value there).
fn key_probe(
    model: &StackModel,
    tokenizer: &ByteBpeTokenizer,
    window: &[u32],
    check: &ExactCheck,
    asked: &[String],
    done: &Answered,
    layer: usize,
) -> Result<Value, Error> {
    let (spans, _) = window_spans(tokenizer, window, check, asked, done);
    let e_runs = runs(&in_earlier(&spans[0], &done.regions));
    let d_runs = runs(&in_earlier(&spans[1], &done.regions));
    let kd_runs = runs(&in_earlier(&spans[2], &done.regions));
    let ka_runs = runs(&spans[3]);
    let qk = model.read_query_key(window, layer)?;
    let t = window.len() - 1;
    let heads = qk.query.len();
    let rows: Vec<ReadRowScores> = (0..heads).map(|h| qk.row(h, t)).collect::<Result<_, _>>()?;
    let pair = |row: &ReadRowScores| mass_in(row, &spans[0]) + mass_in(row, &spans[1]);
    let head = (0..heads).fold(0, |b, h| {
        if pair(&rows[h]) > pair(&rows[b]) {
            h
        } else {
            b
        }
    });
    let row = &rows[head];
    let query = &qk.query[head][t];
    let site = |set: &[usize]| -> Value {
        match best_in(row, set) {
            None => Value::Null,
            Some(j) => json!({
                "position": j, "distance": t - j, "token": tokenizer.decode(&window[j..=j]),
                "mass": mass_in(row, set), "weight": row.weight[j], "score": row.total[j],
                "content": row.content[j], "age": row.age[j],
                "cos_qk": cosine(query, &qk.key[head][j]),
            }),
        }
    };
    let named: BTreeSet<usize> = spans.iter().flatten().copied().collect();
    let other = (0..t)
        .filter(|j| !named.contains(j))
        .fold(None, |b: Option<usize>, j| match b {
            Some(b) if row.total[b] >= row.total[j] => Some(b),
            _ => Some(j),
        });
    let pe = best_in(row, &spans[0]);
    let pd = best_in(row, &spans[1]);
    let value_keys = match (pe, pd) {
        (Some(pe), Some(pd)) => {
            let per_head: Vec<f64> = (0..heads)
                .map(|h| cosine(&qk.key[h][pe], &qk.key[h][pd]))
                .collect();
            json!({
                "cos_binding_head": cosine(&qk.key[head][pe], &qk.key[head][pd]),
                "cos_head_mean": mean(&per_head),
                "distance_binding_head": distance(&qk.key[head][pe], &qk.key[head][pd]),
                "norm_expected": distance(&qk.key[head][pe], &vec![0.0; query.len()]),
                "norm_distractor": distance(&qk.key[head][pd], &vec![0.0; query.len()]),
                "expected_is_later": pe > pd,
            })
        }
        _ => Value::Null,
    };
    // Counterfactual: exchange the asked and distractor key words.
    let ka_earlier: Vec<std::ops::Range<usize>> = ka_runs.clone();
    let counterfactual = match (swap_runs(window, &ka_earlier, &kd_runs), pe, pd) {
        (Some((swapped, map)), Some(pe), Some(pd)) if swapped.len() <= model.config.context => {
            let mut moved = done.clone();
            moved.regions = done
                .regions
                .iter()
                .map(|r| map[r.start]..map[r.end.min(window.len())])
                .collect();
            let (cf_spans, _) = window_spans(tokenizer, &swapped, check, asked, &moved);
            let cf = model.read_query_key(&swapped, layer)?;
            let tc = swapped.len() - 1;
            let cf_row = cf.row(head, tc)?;
            let cf_rows: Vec<ReadRowScores> = (0..heads)
                .map(|h| cf.row(h, tc))
                .collect::<Result<_, _>>()?;
            let max_head =
                |set: &[usize]| cf_rows.iter().map(|r| mass_in(r, set)).fold(0.0, f64::max);
            let (npe, npd) = (map[pe], map[pd]);
            let mut mixed = cf.clone();
            mixed.query[head][tc] = query.clone();
            let mixed_row = mixed.row(head, tc)?;
            let gap = (distance(&qk.key[head][pe], &qk.key[head][pd])).max(1e-12);
            json!({
                "swapped_tokens": swapped.len(),
                "binding_head": {
                    "mass_expected": mass_in(&cf_row, &cf_spans[0]),
                    "mass_distractor": mass_in(&cf_row, &cf_spans[1]),
                    "score_expected": cf_row.total[npe], "score_distractor": cf_row.total[npd],
                    "content_expected": cf_row.content[npe], "content_distractor": cf_row.content[npd],
                },
                "max_head_mass_expected": max_head(&cf_spans[0]),
                "max_head_mass_distractor": max_head(&cf_spans[1]),
                "original_query_on_swapped_keys": {
                    "content_expected": mixed_row.content[npe], "content_distractor": mixed_row.content[npd],
                },
                "key_shift_over_value_gap": {
                    "expected": distance(&cf.key[head][npe], &qk.key[head][pe]) / gap,
                    "distractor": distance(&cf.key[head][npd], &qk.key[head][pd]) / gap,
                },
            })
        }
        _ => Value::Null,
    };
    let max_head = |set: &[usize]| rows.iter().map(|r| mass_in(r, set)).fold(0.0, f64::max);
    Ok(json!({
        "layer": layer,
        "binding_head": head,
        "max_head_mass_expected": max_head(&spans[0]),
        "max_head_mass_distractor": max_head(&spans[1]),
        "no_read": row.no_read,
        "expected": site(&spans[0]),
        "distractor": site(&spans[1]),
        "asked_key": site(&spans[3]),
        "distractor_key": site(&spans[2]),
        "best_other": other.map(|j| json!({"position": j, "token": tokenizer.decode(&window[j..=j]), "weight": row.weight[j], "score": row.total[j], "content": row.content[j], "age": row.age[j]})),
        "value_keys": value_keys,
        "gap": {
            "expected_to_asked_key": nearest_key_gap(&e_runs, &ka_runs),
            "distractor_to_its_key": nearest_key_gap(&d_runs, &kd_runs),
            "runs": {"expected": e_runs.len(), "distractor": d_runs.len(), "asked_key": ka_runs.len(), "distractor_key": kd_runs.len()},
        },
        "counterfactual_key_swap": counterfactual,
    }))
}

fn median(mut values: Vec<f64>) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    let n = values.len();
    Some(if n % 2 == 1 {
        values[n / 2]
    } else {
        (values[n / 2 - 1] + values[n / 2]) / 2.0
    })
}

/// The key probe aggregated per layer over three row groups: binding-swap
/// misses, other misses, passes.
fn key_probe_summary(rows: &[Value], key_layers: &[usize]) -> Value {
    let mut out = serde_json::Map::new();
    for (index, layer) in key_layers.iter().enumerate() {
        let mut groups = serde_json::Map::new();
        for (name, keep) in [
            (
                "a_binding_swap",
                &(|r: &Value| r["miss_class"] == json!("a_binding_swap"))
                    as &dyn Fn(&Value) -> bool,
            ),
            ("other_misses", &|r: &Value| {
                r["pass"] == json!(false) && r["miss_class"] != json!("a_binding_swap")
            }),
            ("passes", &|r: &Value| r["pass"] == json!(true)),
            ("all", &|_: &Value| true),
        ] {
            let probes: Vec<&Value> = rows
                .iter()
                .filter(|r| keep(r))
                .map(|r| &r["key_probe"][index])
                .collect();
            let field = |f: &dyn Fn(&Value) -> Option<f64>| -> Vec<f64> {
                probes.iter().filter_map(|p| f(p)).collect()
            };
            let diff = |a: &str, b: &str, k: &str| -> Vec<f64> {
                field(&|p: &Value| Some(p[a][k].as_f64()? - p[b][k].as_f64()?))
            };
            let frac = |v: &[f64], pred: &dyn Fn(f64) -> bool| -> Value {
                if v.is_empty() {
                    Value::Null
                } else {
                    json!(v.iter().filter(|&&x| pred(x)).count() as f64 / v.len() as f64)
                }
            };
            let score_margin = diff("expected", "distractor", "score");
            let content_margin = diff("expected", "distractor", "content");
            let age_margin = diff("expected", "distractor", "age");
            let cos_margin = diff("expected", "distractor", "cos_qk");
            let mut gaps_e: BTreeMap<i64, usize> = BTreeMap::new();
            let mut gaps_d: BTreeMap<i64, usize> = BTreeMap::new();
            for p in &probes {
                if let Some(g) = p["gap"]["expected_to_asked_key"].as_i64() {
                    *gaps_e.entry(g).or_default() += 1;
                }
                if let Some(g) = p["gap"]["distractor_to_its_key"].as_i64() {
                    *gaps_d.entry(g).or_default() += 1;
                }
            }
            let all_gaps: Vec<f64> = gaps_e
                .iter()
                .chain(&gaps_d)
                .flat_map(|(&g, &n)| std::iter::repeat_n(g as f64, n))
                .collect();
            let cf: Vec<&Value> = probes
                .iter()
                .map(|p| &p["counterfactual_key_swap"])
                .filter(|c| !c.is_null())
                .collect();
            let cf_margin: Vec<f64> = cf
                .iter()
                .filter_map(|c| {
                    Some(
                        c["binding_head"]["score_expected"].as_f64()?
                            - c["binding_head"]["score_distractor"].as_f64()?,
                    )
                })
                .collect();
            let cf_pairs: Vec<(f64, f64)> = probes
                .iter()
                .filter_map(|p| {
                    let c = &p["counterfactual_key_swap"]["binding_head"];
                    Some((
                        p["expected"]["score"].as_f64()? - p["distractor"]["score"].as_f64()?,
                        c["score_expected"].as_f64()? - c["score_distractor"].as_f64()?,
                    ))
                })
                .collect();
            let flips = cf_pairs
                .iter()
                .filter(|(a, b)| a.signum() != b.signum())
                .count();
            let fixed_query: Vec<f64> = cf
                .iter()
                .filter_map(|c| {
                    let o = &c["original_query_on_swapped_keys"];
                    Some(o["content_expected"].as_f64()? - o["content_distractor"].as_f64()?)
                })
                .collect();
            let shift = |k: &str| -> Vec<f64> {
                cf.iter()
                    .filter_map(|c| c["key_shift_over_value_gap"][k].as_f64())
                    .collect()
            };
            groups.insert(
                name.into(),
                json!({
                    "rows": probes.len(),
                    "binding_head_mass": {
                        "expected_mean": mean(&field(&|p: &Value| p["expected"]["mass"].as_f64())),
                        "distractor_mean": mean(&field(&|p: &Value| p["distractor"]["mass"].as_f64())),
                    },
                    "margin_expected_minus_distractor": {
                        "score_mean": mean(&score_margin), "score_median": median(score_margin.clone()),
                        "content_mean": mean(&content_margin), "age_mean": mean(&age_margin),
                        "share_content_prefers_distractor": frac(&content_margin, &|x| x < 0.0),
                        "share_age_prefers_distractor": frac(&age_margin, &|x| x < 0.0),
                        "cos_qk_mean": mean(&cos_margin),
                    },
                    "cos_qk_mean": {
                        "expected": mean(&field(&|p: &Value| p["expected"]["cos_qk"].as_f64())),
                        "distractor": mean(&field(&|p: &Value| p["distractor"]["cos_qk"].as_f64())),
                        "asked_key": mean(&field(&|p: &Value| p["asked_key"]["cos_qk"].as_f64())),
                        "distractor_key": mean(&field(&|p: &Value| p["distractor_key"]["cos_qk"].as_f64())),
                    },
                    "key_word_weight_mean": {
                        "asked_key": mean(&field(&|p: &Value| p["asked_key"]["mass"].as_f64())),
                        "distractor_key": mean(&field(&|p: &Value| p["distractor_key"]["mass"].as_f64())),
                    },
                    "value_keys": {
                        "cos_binding_head_mean": mean(&field(&|p: &Value| p["value_keys"]["cos_binding_head"].as_f64())),
                        "cos_head_mean_mean": mean(&field(&|p: &Value| p["value_keys"]["cos_head_mean"].as_f64())),
                        "share_expected_later": frac(&field(&|p: &Value| p["value_keys"]["expected_is_later"].as_bool().map(|b| if b { 1.0 } else { 0.0 })), &|x| x > 0.5),
                    },
                    "gap": {
                        "expected_to_asked_key": gaps_e,
                        "distractor_to_its_key": gaps_d,
                        "median_abs": median(all_gaps.iter().map(|g| g.abs()).collect()),
                        "share_adjacent_key_before_value": frac(&all_gaps, &|g| g == 1.0),
                        "share_key_after_value": frac(&all_gaps, &|g| g < 0.0),
                        "share_within_8": frac(&all_gaps, &|g| g.abs() <= 8.0),
                        "share_within_16": frac(&all_gaps, &|g| g.abs() <= 16.0),
                    },
                    "counterfactual_key_swap": {
                        "rows": cf.len(),
                        "binding_head_preference_flips": flips,
                        "compared": cf_pairs.len(),
                        "swapped_margin_mean": mean(&cf_margin),
                        "original_query_swapped_keys_content_margin_mean": mean(&fixed_query),
                        "key_shift_over_value_gap_median": {"expected": median(shift("expected")), "distractor": median(shift("distractor"))},
                    },
                }),
            );
        }
        out.insert(layer.to_string(), Value::Object(groups));
    }
    Value::Object(out)
}

fn mean(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

/// Per-layer head-mean and head-max masses on each kind, averaged over rows,
/// from the probe each row was classified at.
fn layer_means(rows: &[&Value]) -> Value {
    let mut by_layer: BTreeMap<u64, BTreeMap<String, (Vec<f64>, Vec<f64>)>> = BTreeMap::new();
    for row in rows {
        let probe = if row["at_decision"].is_null() {
            &row["marker"]
        } else {
            &row["at_decision"]
        };
        for read in probe["reads"].as_array().into_iter().flatten() {
            let layer = read["layer"].as_u64().unwrap_or(0);
            let heads = read["heads"].as_array().cloned().unwrap_or_default();
            for kind in SPAN_KINDS.iter().copied().chain(["other", "no_read"]) {
                let values: Vec<f64> = heads.iter().filter_map(|h| h[kind].as_f64()).collect();
                let entry = by_layer
                    .entry(layer)
                    .or_default()
                    .entry(kind.to_owned())
                    .or_default();
                if let Some(m) = mean(&values) {
                    entry.0.push(m);
                }
                if let Some(x) = values.iter().copied().reduce(f64::max) {
                    entry.1.push(x);
                }
            }
        }
    }
    Value::Array(
        by_layer
            .into_iter()
            .map(|(layer, kinds)| {
                json!({
                    "layer": layer,
                    "head_mean": kinds.iter().map(|(k, v)| (k.clone(), json!(mean(&v.0)))).collect::<serde_json::Map<_, _>>(),
                    "head_max": kinds.iter().map(|(k, v)| (k.clone(), json!(mean(&v.1)))).collect::<serde_json::Map<_, _>>(),
                })
            })
            .collect(),
    )
}

fn pointer_means(rows: &[&Value]) -> Value {
    let mut fields: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    for row in rows {
        let probe = if row["at_decision"].is_null() {
            &row["marker"]
        } else {
            &row["at_decision"]
        };
        let pointer = &probe["pointer"];
        if pointer.is_null() {
            continue;
        }
        for key in ["gate"]
            .into_iter()
            .chain(SPAN_KINDS.iter().copied())
            .chain(["other"])
        {
            if let Some(v) = pointer[key].as_f64() {
                fields.entry(key).or_default().push(v);
            }
        }
        if let Some(kind) = pointer["top_source"]["kind"].as_str() {
            fields
                .entry(match kind {
                    "expected" => "top_is_expected",
                    "distractor" => "top_is_distractor",
                    _ => "top_is_other",
                })
                .or_default()
                .push(1.0);
        }
    }
    let n = rows.len() as f64;
    let mut out: serde_json::Map<String, Value> = fields
        .iter()
        .filter(|(k, _)| !k.starts_with("top_is"))
        .map(|(k, v)| ((*k).to_owned(), json!(mean(v))))
        .collect();
    for key in ["top_is_expected", "top_is_distractor", "top_is_other"] {
        let count = fields.get(key).map_or(0, Vec::len);
        out.insert(
            key.into(),
            json!(if n > 0.0 { count as f64 / n } else { 0.0 }),
        );
    }
    Value::Object(out)
}

fn summarize(rows: &[Value]) -> Value {
    let misses: Vec<&Value> = rows.iter().filter(|r| r["pass"] == json!(false)).collect();
    let passes: Vec<&Value> = rows.iter().filter(|r| r["pass"] == json!(true)).collect();
    let count = |set: &[&Value], field: &str, names: &[&str]| -> Value {
        names
            .iter()
            .map(|name| {
                (
                    (*name).to_owned(),
                    json!(set
                        .iter()
                        .filter(|r| r[field].as_str() == Some(name))
                        .count()),
                )
            })
            .collect::<serde_json::Map<_, _>>()
            .into()
    };
    let classes = [
        MissClass::BindingSwap.name(),
        MissClass::Unreached.name(),
        MissClass::Readout.name(),
    ];
    let patterns = [
        MassPattern::Expected.name(),
        MassPattern::Distractor.name(),
        MassPattern::Neither.name(),
    ];
    let miss_counts = count(&misses, "miss_class", &classes);
    let majority = classes
        .iter()
        .map(|c| (miss_counts[*c].as_u64().unwrap_or(0), *c))
        .max()
        .filter(|(n, _)| *n * 2 > misses.len() as u64)
        .map(|(_, c)| c);
    let sensitivity: Vec<Value> = SENSITIVITY
        .iter()
        .enumerate()
        .map(|(i, &tau)| {
            let mut counts = serde_json::Map::new();
            for c in classes {
                counts.insert(
                    c.into(),
                    json!(misses
                        .iter()
                        .filter(|r| r["sensitivity"][i]["miss_class"].as_str() == Some(c))
                        .count()),
                );
            }
            json!({"tau": tau, "misses": counts})
        })
        .collect();
    let named = |set: &[&Value], key: &str| {
        set.iter()
            .filter(|r| r["reply_names"][key] == json!(true))
            .count()
    };
    let output_means = |set: &[&Value]| -> Value {
        let pick = |key: &str| -> Vec<f64> {
            set.iter()
                .filter_map(|r| {
                    let probe = if r["at_decision"].is_null() {
                        &r["marker"]
                    } else {
                        &r["at_decision"]
                    };
                    probe["evidence"][key].as_f64()
                })
                .collect()
        };
        json!({"out_expected": mean(&pick("out_expected")), "out_distractor": mean(&pick("out_distractor")),
               "read_expected_max": mean(&pick("read_expected_max")), "read_distractor_max": mean(&pick("read_distractor_max"))})
    };
    json!({
        "rows": rows.len(),
        "passes": passes.len(),
        "misses": misses.len(),
        "classified_at_decision": rows.iter().filter(|r| r["classified_at"] == json!("decision")).count(),
        "reference_replies_equal": rows.iter().filter(|r| r["reference_reply_equal"] == json!(true)).count(),
        "reference_replies_compared": rows.iter().filter(|r| !r["reference_reply_equal"].is_null()).count(),
        "miss_classes": miss_counts,
        "miss_majority": majority,
        "miss_patterns": count(&misses, "pattern", &patterns),
        "pass_patterns": count(&passes, "pattern", &patterns),
        "misses_naming": {"distractor": named(&misses, "distractor"), "expected": named(&misses, "expected"), "distractor_key": named(&misses, "distractor_key")},
        "sensitivity": sensitivity,
        "evidence_means": {"misses": output_means(&misses), "passes": output_means(&passes)},
        "read_layers": {"misses": layer_means(&misses), "passes": layer_means(&passes)},
        "pointer": {"misses": pointer_means(&misses), "passes": pointer_means(&passes)},
    })
}

fn fmt(value: &Value) -> String {
    value
        .as_f64()
        .map_or_else(|| "-".to_owned(), |v| format!("{v:.3}"))
}

fn summary_markdown(report: &Value) -> String {
    let s = &report["summary"];
    let mut text = format!(
        "binding-probe {}\nrows {} pass {} miss {} (classified at decision {}; reference replies equal {}/{})\n\
         misses: a_binding_swap {} | b_unreached {} | c_readout {} (tau {}; majority {})\n\
         miss mass patterns {} | pass mass patterns {}\nsensitivity {}\nmisses naming {}\n\
         evidence means {}\n",
        report["model"],
        s["rows"],
        s["passes"],
        s["misses"],
        s["classified_at_decision"],
        s["reference_replies_equal"],
        s["reference_replies_compared"],
        s["miss_classes"]["a_binding_swap"],
        s["miss_classes"]["b_unreached"],
        s["miss_classes"]["c_readout"],
        TAU,
        s["miss_majority"],
        s["miss_patterns"],
        s["pass_patterns"],
        s["sensitivity"],
        s["misses_naming"],
        s["evidence_means"],
    );
    text.push_str("\n| set | layer | expected (mean/max) | distractor | d.key | asked key | question | other | NoRead |\n|---|---|---|---|---|---|---|---|---|\n");
    for set in ["misses", "passes"] {
        for layer in s["read_layers"][set].as_array().into_iter().flatten() {
            let cell = |k: &str| {
                format!(
                    "{}/{}",
                    fmt(&layer["head_mean"][k]),
                    fmt(&layer["head_max"][k])
                )
            };
            text.push_str(&format!(
                "| {set} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
                layer["layer"],
                cell("expected"),
                cell("distractor"),
                cell("distractor_key"),
                cell("asked_key"),
                cell("question"),
                cell("other"),
                cell("no_read"),
            ));
        }
    }
    text.push_str(&format!(
        "\npointer misses {}\npointer passes {}\n",
        s["pointer"]["misses"], s["pointer"]["passes"]
    ));
    if report["key_probe"]
        .as_object()
        .is_some_and(|k| !k.is_empty())
    {
        text.push_str(&format!(
            "\nkey probe (Step 7a) {}\n",
            serde_json::to_string_pretty(&report["key_probe"]).unwrap_or_default()
        ));
    }
    text
}
