//! Whole-reply scorer for a served `UORLUT01` artifact on panel m2-fresh256.
//!
//! Each case renders as one user turn (`one-user-turn/v1`): the `Context`
//! segments' decoded text in event order, then the question, joined by single
//! spaces, encoded with the literal-role dialogue protocol v2 up to the open
//! assistant marker. `Source` segments are skipped. Generation is greedy on the
//! artifact's nearest head through `CpuFbsForward`. A row is complete when it
//! stopped on EOS, its reply decodes as UTF-8 and the frozen answer set accepts
//! the reply text without the v2 leading space.

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uor_r4_core::answer_oracle::FrozenAnswers;
use uor_r4_cpu_train::{CpuFbsForward, CpuFbsHead};
use uor_r4_integer::stack::IntegerStackModel;
use uor_r4_tokenizer::dialogue::{DialogueEncoder, DialogueProtocol, Message, SCHEMA_V2};
use uor_r4_tokenizer::ByteBpeTokenizer;

const SCHEMA: &str = "uor-r4.served-reply-panel/1";
const RENDERING: &str = "one-user-turn/v1";
const BOS: u32 = 0;
const EOS: u32 = 1;
const STRATA: [(&str, usize); 5] =
    [("length2", 64), ("length4", 64), ("length8", 64), ("reassert", 32), ("update", 32)];

type Result<T> = std::result::Result<T, String>;

fn main() {
    if let Err(error) = run() {
        eprintln!("served-reply-panel: {error}");
        std::process::exit(1);
    }
}

#[derive(Deserialize)]
struct Inputs {
    cases: Vec<InputCase>,
}

#[derive(Deserialize)]
struct InputCase {
    id: String,
    query_ids: Vec<u32>,
    segments: Vec<Segment>,
}

#[derive(Deserialize)]
struct Segment {
    kind: String,
    event: u64,
    #[serde(default)]
    token_ids: Vec<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Labels {
    cases: Vec<LabelCase>,
    #[allow(dead_code)]
    membership_only: bool,
    protocol: String,
    #[allow(dead_code)]
    schema: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LabelCase {
    answers: FrozenAnswers,
    id: String,
    pair_id: String,
}

#[derive(Deserialize)]
struct Plan {
    histories: Vec<History>,
}

#[derive(Deserialize)]
struct History {
    id: String,
    stratum: String,
}

#[derive(Deserialize)]
struct Manifest {
    files: BTreeMap<String, ManifestFile>,
    tokenizer_sha256: String,
    strata_rows: BTreeMap<String, usize>,
}

#[derive(Deserialize)]
struct ManifestFile {
    sha256: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum Stop {
    Eos,
    Bos,
    Cap,
    Context,
}

#[derive(Serialize)]
struct Row {
    id: String,
    stratum: String,
    prompt_text: String,
    prompt_tokens: usize,
    reply_ids: Vec<u32>,
    reply_text: String,
    stop: Stop,
    complete: bool,
}

#[derive(Default, Serialize)]
struct Tally {
    rows: usize,
    complete: usize,
    pairs: usize,
    pairs_both_complete_distinct: usize,
}

fn run() -> Result<()> {
    let args = parse_args(std::env::args().skip(1))?;
    let flag = |name: &str| args.get(name).cloned().ok_or(format!("--{name} is required"));
    let out = PathBuf::from(flag("out")?);
    if out.exists() {
        return Err(format!("{} exists; reports are never replaced", out.display()));
    }
    let cap: usize = match args.get("reply-tokens") {
        Some(value) => value.parse().map_err(|_| "--reply-tokens is not a count".to_owned())?,
        None => 64,
    };
    if cap == 0 {
        return Err("--reply-tokens must be positive".to_owned());
    }
    let read = |name: &str| -> Result<(Vec<u8>, String)> {
        let path = PathBuf::from(flag(name)?);
        let bytes = fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        let sha = sha256(&bytes);
        Ok((bytes, sha))
    };
    let (artifact, artifact_sha) = read("artifact")?;
    let (tokenizer_bytes, tokenizer_sha) = read("tokenizer")?;
    let (inputs_bytes, inputs_sha) = read("inputs")?;
    let (labels_bytes, labels_sha) = read("labels")?;
    let (plan_bytes, plan_sha) = read("plan")?;
    let (manifest_bytes, manifest_sha) = read("manifest")?;
    let manifest: Manifest = parse_json("manifest", &manifest_bytes)?;
    for (file, actual) in [("inputs.json", &inputs_sha), ("labels.json", &labels_sha), ("plan.json", &plan_sha)] {
        let pinned = manifest.files.get(file).map(|entry| &entry.sha256);
        if pinned != Some(actual) {
            return Err(format!("manifest files.{file}.sha256 differs from the file read"));
        }
    }
    if manifest.tokenizer_sha256 != tokenizer_sha {
        return Err("manifest tokenizer_sha256 differs from --tokenizer".to_owned());
    }
    let expected: BTreeMap<String, usize> = STRATA.iter().map(|(s, n)| (s.to_string(), *n)).collect();
    if manifest.strata_rows != expected {
        return Err(format!("manifest strata_rows {:?} differ from {expected:?}", manifest.strata_rows));
    }
    let inputs: Inputs = parse_json("inputs", &inputs_bytes)?;
    let labels: Labels = parse_json("labels", &labels_bytes)?;
    let plan: Plan = parse_json("plan", &plan_bytes)?;
    if labels.protocol != SCHEMA_V2 {
        return Err(format!("labels protocol {} is not {SCHEMA_V2}", labels.protocol));
    }
    let model = IntegerStackModel::parse(&artifact).map_err(|e| format!("artifact: {e}"))?;
    if !model.shape().head_nearest() {
        return Err("artifact serves the writer head; this scorer reads the nearest head".to_owned());
    }
    let context_length = CpuFbsForward::from_artifact(&artifact).map_err(|e| e.to_string())?.context_length();
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or("tokenizer JSON is not a supported byte-level BPE tokenizer")?;
    let protocol = DialogueProtocol::literal_roles_v2(&tokenizer).map_err(|e| e.to_string())?;
    let encoder = protocol.bind(&tokenizer).map_err(|e| e.to_string())?;
    let strata: BTreeMap<String, String> = plan
        .histories
        .iter()
        .map(|h| (format!("fresh-{}", h.id), h.stratum.split('/').next().unwrap_or("").to_owned()))
        .collect();
    let by_id: BTreeMap<&str, &InputCase> = inputs.cases.iter().map(|c| (c.id.as_str(), c)).collect();
    if by_id.len() != labels.cases.len() || inputs.cases.len() != labels.cases.len() {
        return Err("inputs and labels do not hold the same cases".to_owned());
    }

    let started = Instant::now();
    let mut rows = Vec::with_capacity(labels.cases.len());
    let mut pair_rows = Vec::with_capacity(labels.cases.len());
    for label in &labels.cases {
        let case = by_id.get(label.id.as_str()).ok_or(format!("{}: no input case", label.id))?;
        let stratum = strata.get(&label.pair_id).ok_or(format!("{}: no plan history", label.pair_id))?;
        let (prompt_text, prompt) = render(&tokenizer, &encoder, case);
        let mut forward = CpuFbsForward::from_artifact(&artifact).map_err(|e| e.to_string())?;
        let (reply_ids, stop) = generate(&mut forward, &prompt, cap)?;
        let bytes = tokenizer.decode_bytes(&reply_ids);
        let (reply_text, complete) = score(stop, &bytes, &label.answers, &protocol);
        pair_rows.push((label.pair_id.clone(), stratum.clone(), complete, protocol.reply_text(&reply_text).to_owned()));
        rows.push(Row {
            id: label.id.clone(),
            stratum: stratum.clone(),
            prompt_text,
            prompt_tokens: prompt.len(),
            reply_ids,
            reply_text,
            stop,
            complete,
        });
    }
    let wall_milliseconds = started.elapsed().as_millis().max(1);

    let mut per_stratum: BTreeMap<String, Tally> = BTreeMap::new();
    for row in &rows {
        let tally = per_stratum.entry(row.stratum.clone()).or_default();
        tally.rows += 1;
        tally.complete += usize::from(row.complete);
    }
    for (stratum, pairs, distinct) in pair_metric(&pair_rows)? {
        let tally = per_stratum.entry(stratum).or_default();
        tally.pairs += pairs;
        tally.pairs_both_complete_distinct += distinct;
    }
    let observed: BTreeMap<String, usize> = per_stratum.iter().map(|(s, t)| (s.clone(), t.rows)).collect();
    if observed != expected {
        return Err(format!("scored strata rows {observed:?} differ from {expected:?}"));
    }
    let stops = |kind: Stop| rows.iter().filter(|row| row.stop == kind).count();
    let reply_tokens: usize = rows.iter().map(|row| row.reply_ids.len()).sum();
    let report = serde_json::json!({
        "schema": SCHEMA,
        "inputs": {
            "artifact_sha256": artifact_sha,
            "tokenizer_sha256": tokenizer_sha,
            "inputs_sha256": inputs_sha,
            "labels_sha256": labels_sha,
            "plan_sha256": plan_sha,
            "manifest_sha256": manifest_sha,
        },
        "settings": {"reply_tokens": cap, "head": "nearest", "decoding": "greedy",
                     "context_length": context_length, "eos_id": EOS, "bos_id": BOS},
        "rendering": {"name": RENDERING, "protocol": SCHEMA_V2, "ending": "assistant-prefix"},
        "totals": {
            "rows": rows.len(),
            "complete": rows.iter().filter(|row| row.complete).count(),
            "eos_stops": stops(Stop::Eos),
            "bos_stops": stops(Stop::Bos),
            "cap_stops": stops(Stop::Cap),
            "context_stops": stops(Stop::Context),
            "pairs": per_stratum.values().map(|t| t.pairs).sum::<usize>(),
            "pairs_both_complete_distinct":
                per_stratum.values().map(|t| t.pairs_both_complete_distinct).sum::<usize>(),
        },
        "per_stratum": per_stratum,
        "reply_tokens": reply_tokens,
        "wall_milliseconds": wall_milliseconds,
        "tokens_per_second": reply_tokens as u128 * 1000 / wall_milliseconds,
        "rows": rows,
    });
    let mut text = serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?;
    text.push(b'\n');
    write_new(&out, &text)?;
    println!("{}", serde_json::json!({"schema": SCHEMA, "totals": report["totals"],
        "out": out, "sha256": sha256(&text)}));
    Ok(())
}

/// One user message: `Context` texts in event order, then the question,
/// joined by single spaces; encoded up to the open `Assistant:` marker.
fn render(tokenizer: &ByteBpeTokenizer, encoder: &DialogueEncoder<'_>, case: &InputCase) -> (String, Vec<u32>) {
    let mut contexts: Vec<&Segment> = case.segments.iter().filter(|s| s.kind == "Context").collect();
    contexts.sort_by_key(|segment| segment.event);
    let mut parts: Vec<String> = contexts
        .iter()
        .map(|segment| tokenizer.decode(&segment.token_ids).trim().to_owned())
        .collect();
    parts.push(tokenizer.decode(&case.query_ids).trim().to_owned());
    let content = parts.join(" ");
    let ids = encoder.encode_assistant_prefix(&[Message { role: "user", content: &content }]).tokens;
    (content, ids)
}

/// The stop rule for a newly generated `token` with `kept` reply tokens
/// already kept: EOS and BOS stop unkept; otherwise the token is kept and the
/// reply stops at the cap, then at the context.
fn stop_for(token: u32, kept: usize, prompt: usize, cap: usize, context: usize) -> Option<Stop> {
    if token == EOS {
        return Some(Stop::Eos);
    }
    if token == BOS {
        return Some(Stop::Bos);
    }
    let kept = kept + 1;
    if kept >= cap {
        Some(Stop::Cap)
    } else if prompt + kept >= context {
        Some(Stop::Context)
    } else {
        None
    }
}

fn generate(forward: &mut CpuFbsForward, prompt: &[u32], cap: usize) -> Result<(Vec<u32>, Stop)> {
    let context = forward.context_length();
    if prompt.is_empty() {
        return Err("empty prompt".to_owned());
    }
    if prompt.len() > context {
        return Ok((Vec::new(), Stop::Context));
    }
    let mut next = 0;
    for (index, &token) in prompt.iter().enumerate() {
        if index + 1 == prompt.len() {
            forward.mark_reply_start();
        }
        next = forward.step(token, CpuFbsHead::Nearest).map_err(|e| e.to_string())?.token;
    }
    let mut reply = Vec::new();
    loop {
        let stop = stop_for(next, reply.len(), prompt.len(), cap, context);
        if !matches!(stop, Some(Stop::Eos | Stop::Bos)) {
            reply.push(next);
        }
        if let Some(stop) = stop {
            return Ok((reply, stop));
        }
        next = forward.step(next, CpuFbsHead::Nearest).map_err(|e| e.to_string())?.token;
    }
}

/// The reply text (lossy when not UTF-8) and whether the row is complete.
fn score(stop: Stop, bytes: &[u8], answers: &FrozenAnswers, protocol: &DialogueProtocol) -> (String, bool) {
    match std::str::from_utf8(bytes) {
        Ok(text) => {
            let complete = stop == Stop::Eos && answers.accepts(protocol.reply_text(text));
            (text.to_owned(), complete)
        }
        Err(_) => (String::from_utf8_lossy(bytes).into_owned(), false),
    }
}

/// Per stratum: (stratum, pairs, pairs where both rows are complete and
/// their reply texts differ). Every pair must hold exactly two rows.
fn pair_metric(rows: &[(String, String, bool, String)]) -> Result<Vec<(String, usize, usize)>> {
    let mut groups: BTreeMap<&str, Vec<&(String, String, bool, String)>> = BTreeMap::new();
    for row in rows {
        let (pair, _, _, _) = row;
        groups.entry(pair.as_str()).or_default().push(row);
    }
    let mut tallies: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for (pair, arms) in groups {
        let [(_, stratum_a, complete_a, text_a), (_, stratum_b, complete_b, text_b)] = arms[..] else {
            return Err(format!("pair {pair} does not hold two rows of one stratum"));
        };
        if stratum_a != stratum_b {
            return Err(format!("pair {pair} does not hold two rows of one stratum"));
        }
        let (pairs, distinct) = tallies.entry(stratum_a.clone()).or_default();
        *pairs += 1;
        *distinct += usize::from(*complete_a && *complete_b && text_a != text_b);
    }
    Ok(tallies.into_iter().map(|(s, (p, d))| (s, p, d)).collect())
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<BTreeMap<String, String>> {
    const FLAGS: [&str; 8] = ["artifact", "tokenizer", "inputs", "labels", "plan", "manifest", "out", "reply-tokens"];
    let mut values = BTreeMap::new();
    let mut args = args;
    while let Some(arg) = args.next() {
        let name = arg.strip_prefix("--").filter(|name| FLAGS.contains(name)).ok_or(format!("unknown argument {arg}"))?;
        let value = args.next().ok_or(format!("{arg} needs a value"))?;
        if values.insert(name.to_owned(), value).is_some() {
            return Err(format!("{arg} given twice"));
        }
    }
    Ok(values)
}

fn parse_json<T: for<'de> Deserialize<'de>>(name: &str, bytes: &[u8]) -> Result<T> {
    serde_json::from_slice(bytes).map_err(|error| format!("{name}: {error}"))
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    file.write_all(bytes).map_err(|error| format!("{}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pinned tokenizer (`d36d3e87…`) lives outside the repository: name it in
    /// `UOR_PANEL_TOKENIZER`. Without it, the tests that need it say so and pass.
    fn tokenizer() -> Option<ByteBpeTokenizer> {
        let Ok(path) = std::env::var("UOR_PANEL_TOKENIZER") else {
            eprintln!("skipped: set UOR_PANEL_TOKENIZER to the pinned tokenizer.json");
            return None;
        };
        Some(ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(path).unwrap()).unwrap())
    }

    fn answers(accepted: &str) -> FrozenAnswers {
        serde_json::from_value(serde_json::json!({"accepted": [accepted], "intent": "current"})).unwrap()
    }

    #[test]
    fn renders_two_contexts_in_event_order_without_source() {
        let Some(tokenizer) = tokenizer() else { return };
        let protocol = DialogueProtocol::literal_roles_v2(&tokenizer).unwrap();
        let encoder = protocol.bind(&tokenizer).unwrap();
        let segment = |kind: &str, event, text: &str| Segment {
            kind: kind.to_owned(),
            event,
            token_ids: tokenizer.encode(text),
        };
        let case = InputCase {
            id: "hand".to_owned(),
            query_ids: tokenizer.encode(" Who has the map?"),
            segments: vec![
                segment("Context", 1, " Bo has the map."),
                segment("Source", 0, "Ada keeps"),
                segment("Context", 0, "Ada keeps the key."),
            ],
        };
        let (text, ids) = render(&tokenizer, &encoder, &case);
        assert_eq!(text, "Ada keeps the key. Bo has the map. Who has the map?");
        assert_eq!(ids[0], BOS);
        assert_eq!(
            tokenizer.decode(&ids),
            "<|bos|>User: Ada keeps the key. Bo has the map. Who has the map?\nAssistant:"
        );
    }

    #[test]
    fn stop_rule_order() {
        assert_eq!(stop_for(EOS, 63, 10, 64, 192), Some(Stop::Eos));
        assert_eq!(stop_for(BOS, 63, 10, 64, 192), Some(Stop::Bos));
        assert_eq!(stop_for(7, 63, 190, 64, 192), Some(Stop::Cap));
        assert_eq!(stop_for(7, 3, 188, 64, 192), Some(Stop::Context));
        assert_eq!(stop_for(7, 3, 187, 64, 192), None);
        assert_eq!(stop_for(7, 0, 10, 64, 192), None);
    }

    /// Past EOS and BOS, the cap and context stops are the served path's own rule.
    #[test]
    fn cap_and_context_stops_match_the_served_rule() {
        use uor_r4_integer::stack::{reply_stop, ReplyEnd};
        for prompt in 1..=192 {
            for kept in 0..64 {
                let served = match reply_stop(7, kept, prompt, 64, 192) {
                    Some(ReplyEnd::Length) => Some(Stop::Cap),
                    Some(ReplyEnd::Context) => Some(Stop::Context),
                    Some(ReplyEnd::Bos) => Some(Stop::Bos),
                    None => None,
                };
                assert_eq!(stop_for(7, kept, prompt, 64, 192), served, "prompt {prompt}, kept {kept}");
            }
        }
    }

    #[test]
    fn known_answer_needs_eos_and_strips_leading_space() {
        let Some(tokenizer) = tokenizer() else { return };
        let protocol = DialogueProtocol::literal_roles_v2(&tokenizer).unwrap();
        let accepted = answers("basalt loggia.");
        assert_eq!(score(Stop::Eos, b" basalt loggia.", &accepted, &protocol), (" basalt loggia.".to_owned(), true));
        assert!(matches!(score(Stop::Cap, b" basalt loggia.", &accepted, &protocol), (_, false)));
        assert!(matches!(score(Stop::Eos, b"  basalt loggia.", &accepted, &protocol), (_, false)));
        assert!(matches!(score(Stop::Eos, b" basalt loggia. ", &accepted, &protocol), (_, false)));
        assert!(matches!(score(Stop::Eos, b" basalt \xff", &answers("basalt \u{fffd}"), &protocol), (_, false)));
    }

    #[test]
    fn pair_metric_needs_both_complete_and_distinct() {
        let row = |pair: &str, stratum: &str, complete, text: &str| {
            (pair.to_owned(), stratum.to_owned(), complete, text.to_owned())
        };
        let rows = vec![
            row("a", "length2", true, "x"),
            row("a", "length2", true, "y"),
            row("b", "length2", true, "x"),
            row("b", "length2", true, "x"),
            row("c", "update", true, "x"),
            row("c", "update", false, "y"),
        ];
        assert_eq!(
            pair_metric(&rows).unwrap(),
            vec![("length2".to_owned(), 2, 1), ("update".to_owned(), 1, 0)]
        );
        assert!(pair_metric(&rows[..5]).is_err());
    }
}
