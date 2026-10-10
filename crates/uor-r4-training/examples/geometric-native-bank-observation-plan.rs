//! Bounded construction-plan authorer + truthful retained-input bundler.
//! No native model predictions or outcome-adaptive source selection.
//! Input is a NEW sealed bundle of retained frozen-inputs512/128 (+opened repeat16),
//! not a partial cached directory masquerading as the original sealed full report.
//! Output:64 open-development histories +16 separately prospective fresh histories.
//! Explicit fresh-only diversity authoring emits64 histories/128 queries and
//! binds familiar full-literal coverage to a sealed existing development plan.
//! Finite deterministic source construction; public formatter/budget eligibility only.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::report_output;
use uor_r4_integer::geometric_source_realizer::{NativeArtifactBinding, NativeSourceRealizer};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::{sha256_bytes, sha256_file};
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
#[path = "support/geometric_role_diversity.rs"]
mod role_diversity;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn invalid(s: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, s.into())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    schema: String,
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    assertion_query_policy: AssertionQueryPolicy,
    #[serde(default)]
    transfer_profile: TransferProfile,
    input_bundle: PathBuf,
    input_manifest_sha256: String,
    exposed_roots: Vec<BoundRoot>,
    native_artifact: PathBuf,
    trusted_binding: PathBuf,
    trusted_binding_sha256: String,
    #[serde(default)]
    development_out: Option<PathBuf>,
    #[serde(default)]
    fresh_only_training_plan: Option<BoundRoot>,
    fresh_out: PathBuf,
    maximum_seconds: u64,
    maximum_report_bytes: usize,
}
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
enum AssertionQueryPolicy {
    #[default]
    #[serde(rename = "retained-original/1")]
    RetainedOriginal,
    #[serde(rename = "supported-current-role/1")]
    SupportedCurrentRole,
}
impl AssertionQueryPolicy {
    fn source_policy(self) -> &'static str {
        match self {
            Self::RetainedOriginal => "original-assertion-cues/raw-queries/all-source-candidates/1",
            Self::SupportedCurrentRole => {
                "explicit-current-role-assertions/raw-current-role-queries/all-source-candidates/2"
            }
        }
    }
}
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
enum TransferProfile {
    #[default]
    #[serde(rename = "retained-composition/1")]
    RetainedComposition,
    #[serde(rename = "supported-untouched-composition/1")]
    SupportedUntouchedComposition,
    #[serde(rename = "supported-prospective-role-diversity/1")]
    SupportedProspectiveRoleDiversity,
}
const TRANSFER_JOB_QUERY: &str = "What is my job currently?";
const TRANSFER_HOME_QUERY: &str = "Where do I live currently?";
fn collect_opened_wires(
    v: &Value,
    values: &mut BTreeSet<String>,
    queries: &mut BTreeSet<String>,
) -> Result<()> {
    match v {
        Value::Object(o) => {
            if let Some(w) = o
                .contains_key("template")
                .then(|| serde_json::from_value::<Wire>(v.clone()).ok())
                .flatten()
            {
                if w.act == "query" {
                    queries.insert(w.text);
                } else if matches!(w.act.as_str(), "assert" | "update") {
                    values.insert(literal(&w)?.to_owned());
                }
            }
            for (key, x) in o {
                // Empty template registry entries are reference controls, not facts.
                if key == "reference_templates_control_only" {
                    continue;
                }
                collect_opened_wires(x, values, queries)?;
            }
        }
        Value::Array(xs) => {
            for x in xs {
                collect_opened_wires(x, values, queries)?;
            }
        }
        _ => {}
    }
    Ok(())
}
fn opened_wire_counts(v: &Value) -> (usize, usize) {
    let mut counts = (0, 0);
    match v {
        Value::Object(o) => {
            if let Some(w) = o
                .contains_key("template")
                .then(|| serde_json::from_value::<Wire>(v.clone()).ok())
                .flatten()
            {
                if w.act == "query" {
                    counts.1 += 1;
                } else if matches!(w.act.as_str(), "assert" | "update") {
                    counts.0 += 1;
                }
            }
            for (key, x) in o {
                if key == "reference_templates_control_only" {
                    continue;
                }
                let n = opened_wire_counts(x);
                counts.0 += n.0;
                counts.1 += n.1;
            }
        }
        Value::Array(xs) => {
            for x in xs {
                let n = opened_wire_counts(x);
                counts.0 += n.0;
                counts.1 += n.1;
            }
        }
        _ => {}
    }
    counts
}
fn semantic_identity(h: &History) -> Result<(String, String)> {
    let mut current = BTreeMap::new();
    let mut turns = Vec::new();
    for w in &h.turns {
        let value = literal(w)?;
        current.insert(w.relation.as_str(), value);
        turns.push(json!({"role":w.relation,"act":w.act,"literal":value}));
    }
    Ok((
        sha256_bytes(&serde_json::to_vec(&current)?),
        sha256_bytes(&serde_json::to_vec(&turns)?),
    ))
}
fn untouched_histories(all: &[Wire]) -> Result<(Vec<History>, BTreeMap<String, Vec<Wire>>)> {
    let jf = pool(all, "job", "assert", None)?;
    let hf = pool(all, "home", "assert", None)?;
    let ju = pool(all, "job", "update", None)?;
    let hu = pool(all, "home", "update", None)?;
    let fixed = [
        ("amber willow", "copper cedar", None),
        (
            "amber birch silver willow copper cedar harbor silver",
            "silver cedar copper birch amber willow amber harbor",
            None,
        ),
        (
            "violet willow",
            "copper meadow silver birch",
            Some(("job", "amber cedar violet willow")),
        ),
        (
            "orchard silver harbor amber",
            "birch copper",
            Some(("home", "willow copper birch violet")),
        ),
    ];
    let mut result = Vec::new();
    let mut frame_donors = BTreeMap::new();
    for (i, (jv, hv, update)) in fixed.into_iter().enumerate() {
        let jd = jf[i % jf.len()].clone();
        let hd = hf[i % hf.len()].clone();
        let j = substituted(&jd, jv)?;
        let h = substituted(&hd, hv)?;
        let mut after = Vec::new();
        let mut after_donors = Vec::new();
        if let Some((role, value)) = update {
            let ud = if role == "job" {
                ju[i % ju.len()].clone()
            } else {
                hu[i % hu.len()].clone()
            };
            after.push(substituted(&ud, value)?);
            after_donors.push(ud);
            if role == "job" {
                after.push(h.clone());
                after_donors.push(hd.clone());
            } else {
                after.push(j.clone());
                after_donors.push(jd.clone());
            }
        }
        for group in ["familiar", "novel"] {
            let histories = chronologies(
                &format!("untouched-bank{i:02}-{group}"),
                &format!("untouched-current-role/{group}/order-pair"),
                j.clone(),
                h.clone(),
                after.clone(),
                qpair(all, i)?,
            );
            for (order, history) in histories.into_iter().enumerate() {
                let mut donors = if order == 0 {
                    vec![jd.clone(), hd.clone()]
                } else {
                    vec![hd.clone(), jd.clone()]
                };
                donors.extend(if order == 0 {
                    after_donors.clone()
                } else {
                    after_donors.iter().rev().cloned().collect()
                });
                frame_donors.insert(history.id.clone(), donors);
                result.push(history);
            }
        }
    }
    Ok((result, frame_donors))
}
fn apply_untouched_policy(
    histories: &mut [History],
    donors: &BTreeMap<String, Vec<Wire>>,
) -> Result<Vec<Value>> {
    let mut origins = apply_supported_policy(histories)?;
    for (h, origin) in histories.iter_mut().zip(origins.iter_mut()) {
        let ds = donors
            .get(&h.id)
            .ok_or_else(|| invalid("untouched original frame donors absent"))?;
        for (receipt, donor) in origin["turns"]
            .as_array_mut()
            .ok_or_else(|| invalid("turn origins absent"))?
            .iter_mut()
            .zip(ds)
        {
            receipt["original_frame_donor_wire"] = serde_json::to_value(donor)?;
        }
        for (q, receipt) in h.queries.iter_mut().zip(
            origin["queries"]
                .as_array_mut()
                .ok_or_else(|| invalid("query origins absent"))?,
        ) {
            q.text = match (h.stratum.contains("/novel/"), q.relation.as_str()) {
                (true, "job") => TRANSFER_JOB_QUERY,
                (true, "home") => TRANSFER_HOME_QUERY,
                (false, "job") => CURRENT_JOB_QUERIES[0],
                (false, "home") => CURRENT_HOME_QUERIES[0],
                _ => return Err(invalid("untouched role absent").into()),
            }
            .into();
            q.template = Some(q.text.clone());
            receipt["authored_wire"] = serde_json::to_value(q)?;
        }
    }
    Ok(origins)
}
const CURRENT_JOB_QUERIES: [&str; 6] = [
    "What is my current job?",
    "What job do I currently have?",
    "What is my job now?",
    "Which job do I have now?",
    "Remind me of my current job.",
    "Tell me my current job.",
];
const CURRENT_HOME_QUERIES: [&str; 6] = [
    "Where do I currently live?",
    "Where do I live now?",
    "What is my current residence?",
    "Remind me where I currently live.",
    "Tell me where I live now.",
    "Where is my current home?",
];
fn supported_wire(donor: &Wire, pair_index: usize) -> Result<Wire> {
    let template = match (donor.relation.as_str(), donor.act.as_str()) {
        ("job", "assert") => "My current job is {v}.",
        ("job", "update") => "My current job has changed to {v}.",
        ("home", "assert") => "I currently live in {v}.",
        ("home", "update") => "I now live in {v}.",
        ("job", "query") => CURRENT_JOB_QUERIES[pair_index % CURRENT_JOB_QUERIES.len()],
        ("home", "query") => CURRENT_HOME_QUERIES[pair_index % CURRENT_HOME_QUERIES.len()],
        _ => return Err(invalid("unsupported current-role relation/action").into()),
    };
    let text = if donor.act == "query" {
        template.to_owned()
    } else {
        template.replace("{v}", literal(donor)?)
    };
    Ok(Wire {
        text,
        relation: donor.relation.clone(),
        act: donor.act.clone(),
        template: Some(template.into()),
    })
}
fn supported_contract(w: &Wire) -> Result<()> {
    let valid = match (w.relation.as_str(), w.act.as_str()) {
        ("job", "assert") => {
            w.template.as_deref() == Some("My current job is {v}.") && literal(w).is_ok()
        }
        ("job", "update") => {
            w.template.as_deref() == Some("My current job has changed to {v}.")
                && literal(w).is_ok()
        }
        ("home", "assert") => {
            w.template.as_deref() == Some("I currently live in {v}.") && literal(w).is_ok()
        }
        ("home", "update") => {
            w.template.as_deref() == Some("I now live in {v}.") && literal(w).is_ok()
        }
        ("job", "query") => {
            CURRENT_JOB_QUERIES.contains(&w.text.as_str())
                && w.template.as_deref() == Some(w.text.as_str())
        }
        ("home", "query") => {
            CURRENT_HOME_QUERIES.contains(&w.text.as_str())
                && w.template.as_deref() == Some(w.text.as_str())
        }
        _ => false,
    };
    if !valid {
        return Err(invalid("assertion/question does not establish supported current role").into());
    }
    Ok(())
}
fn apply_supported_policy(histories: &mut [History]) -> Result<Vec<Value>> {
    apply_supported_policy_mode(histories, false)
}
fn apply_supported_policy_mode(histories: &mut [History], diverse: bool) -> Result<Vec<Value>> {
    let mut origins = Vec::new();
    for (index, history) in histories.iter_mut().enumerate() {
        let mut turns = Vec::new();
        let mut queries = Vec::new();
        for (kind, packets) in [
            ("turn", &mut history.turns),
            ("query", &mut history.queries),
        ] {
            for (ordinal, wire) in packets.iter_mut().enumerate() {
                let donor = wire.clone();
                let query_index = if diverse {
                    history
                        .stratum
                        .rsplit("/q")
                        .next()
                        .ok_or_else(|| invalid("diversity query index absent"))?
                        .parse::<usize>()?
                } else {
                    index / 2
                };
                let authored = supported_wire(&donor, query_index)?;
                supported_contract(&authored)?;
                if kind == "turn" && literal(&donor)? != literal(&authored)? {
                    return Err(invalid("supported rewrite changed literal bytes").into());
                }
                let receipt = json!({"ordinal":ordinal,"donor_construction_wire":donor,"donor_text_sha256":sha256_bytes(wire.text.as_bytes()),"authored_wire":authored,"origin":"prospectively authored explicit-current-role frame/question;not retained original assertion bytes"});
                if kind == "turn" {
                    turns.push(receipt);
                } else {
                    queries.push(receipt);
                }
                *wire = authored;
            }
        }
        origins.push(json!({"history":history.id,"turns":turns,"queries":queries}));
    }
    Ok(origins)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BoundRoot {
    root: PathBuf,
    manifest_sha256: String,
}
#[derive(Clone, Deserialize, Serialize)]
struct Wire {
    text: String,
    relation: String,
    act: String,
    template: Option<String>,
}
#[derive(Clone, Deserialize, Serialize)]
struct History {
    id: String,
    stratum: String,
    turns: Vec<Wire>,
    queries: Vec<Wire>,
}
fn read_json(p: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(p)?)?)
}
fn verify(root: &Path, sha: &str) -> Result<()> {
    report_output::verify(root)?;
    if sha256_file(&root.join("manifest.json"))? != sha {
        return Err(invalid("sealed source bundle differs").into());
    }
    Ok(())
}
fn rows(v: &Value) -> Result<Vec<Wire>> {
    Ok(serde_json::from_value(v.clone())?)
}
fn literal(w: &Wire) -> Result<&str> {
    let t = w
        .template
        .as_deref()
        .ok_or_else(|| invalid("write frame absent"))?;
    let (a, b) = t
        .split_once("{v}")
        .ok_or_else(|| invalid("one write slot required"))?;
    if b.contains("{v}") {
        return Err(invalid("multiple slots unsupported").into());
    }
    let s = w
        .text
        .strip_prefix(a)
        .and_then(|s| s.strip_suffix(b))
        .filter(|s| !s.is_empty())
        .ok_or_else(|| invalid("original write/frame byte binding differs"))?;
    Ok(s)
}
fn substituted(w: &Wire, value: &str) -> Result<Wire> {
    let t = w
        .template
        .as_deref()
        .ok_or_else(|| invalid("original known frame absent"))?;
    let (a, b) = t
        .split_once("{v}")
        .ok_or_else(|| invalid("write frame slot absent"))?;
    if b.contains("{v}") {
        return Err(invalid("multiple slots unsupported").into());
    }
    Ok(Wire {
        text: format!("{a}{value}{b}"),
        relation: w.relation.clone(),
        act: w.act.clone(),
        template: Some(t.to_owned()),
    })
}
fn pool(all: &[Wire], relation: &str, act: &str, length: Option<usize>) -> Result<Vec<Wire>> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for w in all {
        if w.relation == relation && w.act == act {
            if length.is_none_or(|n| literal(w).is_ok_and(|s| s.split_whitespace().count() == n))
                && seen.insert(w.text.clone())
            {
                out.push(w.clone());
            }
        }
    }
    if out.is_empty() {
        return Err(invalid(format!("source pool missing {relation}/{act}/{length:?}")).into());
    }
    Ok(out)
}
fn qpair(all: &[Wire], i: usize) -> Result<Vec<Wire>> {
    let j = pool(all, "job", "query", None)?;
    let h = pool(all, "home", "query", None)?;
    Ok(vec![j[i % j.len()].clone(), h[i % h.len()].clone()])
}
fn opposite_pair(all: &[Wire], length: usize, i: usize) -> Result<(Wire, Wire)> {
    let j = pool(all, "job", "assert", Some(length))?;
    let h = pool(all, "home", "assert", Some(length))?;
    let job = j[i % j.len()].clone();
    let home = h[(i + 1) % h.len()].clone();
    if literal(&job)? == literal(&home)? {
        return Err(
            invalid("fixed role pair has identical values;would not test query binding").into(),
        );
    }
    Ok((job, home))
}
fn chronologies(
    id: &str,
    stratum: &str,
    j: Wire,
    h: Wire,
    after: Vec<Wire>,
    queries: Vec<Wire>,
) -> Vec<History> {
    let mut forward = vec![j.clone(), h.clone()];
    forward.extend(after.clone());
    let mut reverse = vec![h, j];
    reverse.extend(after.into_iter().rev());
    vec![
        History {
            id: format!("{id}-forward"),
            stratum: stratum.into(),
            turns: forward,
            queries: queries.clone(),
        },
        History {
            id: format!("{id}-reverse"),
            stratum: stratum.into(),
            turns: reverse,
            queries,
        },
    ]
}
fn opened_strings(v: &Value, out: &mut BTreeSet<String>) {
    match v {
        Value::Object(o) => {
            for (k, x) in o {
                if matches!(k.as_str(), "text" | "source" | "utterance_utf8") {
                    if let Some(s) = x.as_str() {
                        out.insert(s.into());
                    }
                }
                opened_strings(x, out)
            }
        }
        Value::Array(xs) => {
            for x in xs {
                opened_strings(x, out)
            }
        }
        _ => {}
    }
}
fn semantic_history(h: &History) -> Result<Vec<String>> {
    let mut current = BTreeMap::new();
    for (event, w) in h.turns.iter().enumerate() {
        current.insert(w.relation.clone(), (event, w));
    }
    let mut records = current.values().collect::<Vec<_>>();
    records.sort_by_key(|r| r.0);
    h.queries.iter().map(|q|Ok(sha256_bytes(&serde_json::to_vec(&json!({"source_statements":records.iter().map(|(_,w)|&w.text).collect::<Vec<_>>(),"source_values":records.iter().map(|(_,w)|literal(w)).collect::<Result<Vec<_>>>()?,"query":q.text}))?))).collect()
}
fn numerical_packets(v: &Value) -> Result<BTreeSet<String>> {
    let mut out = BTreeSet::new();
    for c in v["cases"]
        .as_array()
        .ok_or_else(|| invalid("opened numerical bank cases absent"))?
    {
        let segs = c["segments"]
            .as_array()
            .ok_or_else(|| invalid("opened numerical bank segments absent"))?
            .iter()
            .map(|s| match s["kind"].as_str() {
                Some("Context") => {
                    Ok(json!({"kind":"Context","role":s["role"],"token_ids":s["token_ids"]}))
                }
                Some("Source") => {
                    Ok(json!({"kind":"Source","original_source_ids":s["original_source_ids"]}))
                }
                _ => Err(invalid("unknown opened numerical segment")),
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        out.insert(sha256_bytes(&serde_json::to_vec(
            &json!({"segments":segs,"query_ids":c["query_ids"]}),
        )?));
    }
    Ok(out)
}
fn numerical_history(h: &History, tok: &ByteBpeTokenizer) -> Result<BTreeSet<String>> {
    let mut current = BTreeMap::new();
    for (event, w) in h.turns.iter().enumerate() {
        current.insert(w.relation.clone(), (event, w));
    }
    let mut records = current.values().collect::<Vec<_>>();
    records.sort_by_key(|r| r.0);
    let mut segs = Vec::new();
    for (_, w) in records {
        segs.push(json!({"kind":"Context","role":1,"token_ids":tok.encode(&w.text)}));
        segs.push(json!({"kind":"Source","original_source_ids":tok.encode(literal(w)?)}));
    }
    let cases = h
        .queries
        .iter()
        .map(|q| json!({"segments":segs,"query_ids":tok.encode(&q.text)}))
        .collect::<Vec<_>>();
    numerical_packets(&json!({"cases":cases}))
}
fn eligible(h: &History, native: &NativeSourceRealizer, tok: &ByteBpeTokenizer) -> Result<Value> {
    let mut current = BTreeMap::new();
    for w in &h.turns {
        let val = literal(w)?;
        if val.split_whitespace().count() > 8 || tok.encode(&w.text).len() > 128 {
            return Err(invalid("original write/value public source bound").into());
        }
        current.insert(w.relation.as_str(), w);
    }
    if current.len() != 2 {
        return Err(
            invalid("every natural development bank must contain both actual roles").into(),
        );
    }
    let mut alphabet = BTreeSet::from([
        native.binding().period_token_id(),
        native.binding().eos_token_id(),
    ]);
    let mut replay = 0;
    let mut source_views = Vec::new();
    for (role, w) in &current {
        let value = tok.encode(literal(w)?);
        let view = native.compile_view(&value)?;
        let cue = tok.encode(&w.text);
        if cue
            .iter()
            .chain(value.iter())
            .chain(view.emitted_token_ids())
            .any(|id| *id >= native.context_config().vocab_size as u32)
        {
            return Err(invalid("fixed source has IDs outside public parent context vocab").into());
        }
        replay += cue.len() + view.emitted_token_ids().len();
        alphabet.extend(view.emitted_token_ids().iter().copied());
        source_views.push(json!({"role":role,"original_statement_sha256":sha256_bytes(w.text.as_bytes()),"source_view":view}));
    }
    let mut targets = Vec::new();
    for q in &h.queries {
        let ids = tok.encode(&q.text);
        let w = current
            .get(q.relation.as_str())
            .ok_or_else(|| invalid("query role absent"))?;
        let value = literal(w)?;
        let mut answer = tok.encode(&format!(" {value}."));
        answer.push(native.binding().eos_token_id());
        if answer.len() > 32
            || replay + ids.len() + answer.len() > 128
            || ids
                .iter()
                .any(|x| *x >= native.context_config().vocab_size as u32)
            || answer.iter().any(|x| !alphabet.contains(x))
        {
            return Err(invalid(format!(
                "fixed source eligibility failed; history={} role={} answer_tokens={} replay_tokens={} query_tokens={} total_tokens={} out_of_vocab={} missing_alphabet={:?};no outcome-based replacement",
                h.id, q.relation, answer.len(), replay, ids.len(), replay + ids.len() + answer.len(),
                ids.iter().any(|x| *x >= native.context_config().vocab_size as u32),
                answer.iter().filter(|x| !alphabet.contains(x)).collect::<Vec<_>>()
            )).into());
        }
        targets.push(json!({"role":q.relation,"target_tokens_labels_only":answer.len(),"complete_public_replay_budget":replay+ids.len()+answer.len()}));
    }
    Ok(
        json!({"history":h.id,"source_views_construction_only":source_views,"targets_labels_only":targets,"native_predictions":"NOT_RUN","model_probability_support":"NOT_RUN;public alphabet membership only"}),
    )
}
/// Fixed prospective design; no native predictions or cue addresses select rows.
fn diversity_histories(
    all: &[Wire],
    _repeats: &[Wire],
    exposed_banks: &BTreeSet<String>,
) -> Result<(Vec<History>, Vec<History>)> {
    diversity_histories_for_mode(all, _repeats, exposed_banks, None)
}
fn diversity_histories_for_mode(
    all: &[Wire],
    _repeats: &[Wire],
    exposed_banks: &BTreeSet<String>,
    training_plan: Option<&Value>,
) -> Result<(Vec<History>, Vec<History>)> {
    let fresh_only = training_plan.is_some();
    if let Some(plan) = training_plan {
        validate_fresh_training_plan(plan)?;
    }
    fn donor(all: &[Wire], role: &str, act: &str, value: &str) -> Result<Wire> {
        all.iter()
            .find(|w| w.relation == role && w.act == act && literal(w).is_ok_and(|v| v == value))
            .cloned()
            .ok_or_else(|| invalid("diversity exact role/action/literal donor absent").into())
    }
    let mut development = Vec::new();
    let mut fresh = Vec::new();
    let mut used_unordered = BTreeSet::new();
    for (stratum, length, dev_blocks, fresh_blocks) in [
        ("length2", Some(2), 8usize, 2usize),
        ("length4", Some(4), 8, 2),
        ("length8", Some(8), 8, 2),
        ("update", Some(4), 4, 1),
        ("reassert", Some(2), 4, 1),
    ] {
        let dev_blocks = if fresh_only { 0 } else { dev_blocks };
        // Reassertions are prospectively authored by repeating a retained assert wire.
        // Ordinary both-role witnesses suffice; original repeated histories are not claimed.
        let source = all;
        let job_pool = pool(source, "job", "assert", length)?;
        let home_pool = pool(source, "home", "assert", length)?;
        let job = job_pool
            .iter()
            .map(literal)
            .collect::<Result<BTreeSet<_>>>()?;
        let home = home_pool
            .iter()
            .map(literal)
            .collect::<Result<BTreeSet<_>>>()?;
        // Predeclared public donor bound, never chosen from native outcomes/addresses.
        // Four reassertion training blocks can cover at most eight vertices.
        let vertex_cap = if stratum == "reassert" { 8 } else { 10 };
        let values = job
            .intersection(&home)
            .take(vertex_cap)
            .map(|v| (*v).to_owned())
            .collect::<Vec<_>>();
        let mut selected = 0usize;
        for (left, right) in role_diversity::round_robin_edges(values.len()) {
            if selected == dev_blocks + fresh_blocks {
                break;
            }
            let a = &values[left];
            let b = &values[right];
            let key = (a.clone(), b.clone());
            if used_unordered.contains(&key) {
                continue;
            }
            let mut blocked = false;
            for (j, h) in [(a, b), (b, a)] {
                let current = BTreeMap::from([("job", j.as_str()), ("home", h.as_str())]);
                blocked |= exposed_banks.contains(&sha256_bytes(&serde_json::to_vec(&current)?));
            }
            if blocked {
                continue;
            }
            let output = if selected < dev_blocks {
                &mut development
            } else {
                &mut fresh
            };
            let block = output.len() / 8;
            for swapped in [false, true] {
                let (jv, hv) = if swapped { (b, a) } else { (a, b) };
                let mut j = donor(source, "job", "assert", jv)?;
                let mut h = donor(source, "home", "assert", hv)?;
                let mut after = Vec::new();
                if stratum == "update" {
                    let role = if swapped { "home" } else { "job" };
                    let other_role = if role == "job" { "home" } else { "job" };
                    let others = pool(all, other_role, "assert", Some(2))?
                        .iter()
                        .map(|w| literal(w).map(str::to_owned))
                        .collect::<Result<BTreeSet<_>>>()?;
                    let mut initials = pool(all, role, "assert", Some(2))?
                        .into_iter()
                        .filter(|w| literal(w).is_ok_and(|v| others.contains(v)))
                        .collect::<Vec<_>>();
                    let mut keyed_initials = initials
                        .drain(..)
                        .map(|w| Ok((literal(&w)?.to_owned(), w)))
                        .collect::<Result<Vec<_>>>()?;
                    keyed_initials.sort_by(|a, b| a.0.cmp(&b.0));
                    initials = keyed_initials.into_iter().map(|(_, w)| w).collect();
                    let initial = initials
                        .into_iter()
                        .find(|w| literal(w).is_ok_and(|v| v != a && v != b))
                        .ok_or_else(|| invalid("diversity distinct initial update donor absent"))?;
                    if swapped {
                        h = initial;
                    } else {
                        j = initial;
                    }
                    after.push(donor(all, role, "update", a)?);
                    after.push(if swapped { j.clone() } else { h.clone() });
                } else if stratum == "reassert" {
                    after.push(if swapped { h.clone() } else { j.clone() });
                }
                for query_index in [block % 6, (block + 3) % 6] {
                    let mut variants = chronologies(
                        &format!(
                            "diverse-{stratum}-{selected:02}-swap{}-q{query_index}",
                            usize::from(swapped)
                        ),
                        &format!("{stratum}/q{query_index}"),
                        j.clone(),
                        h.clone(),
                        after.clone(),
                        qpair(all, query_index)?,
                    );
                    if stratum == "reassert" {
                        // Cross the retained bank order, not only a discarded initial order.
                        for history in &mut variants {
                            let repeated = history
                                .turns
                                .first()
                                .cloned()
                                .ok_or_else(|| invalid("reassert initial assertion absent"))?;
                            *history
                                .turns
                                .last_mut()
                                .ok_or_else(|| invalid("reassert final assertion absent"))? =
                                repeated;
                        }
                    }
                    output.extend(variants);
                }
            }
            used_unordered.insert(key);
            selected += 1;
        }
        if selected != dev_blocks + fresh_blocks {
            return Err(invalid(format!("diversity insufficient disjoint donor blocks for {stratum}: {selected}; no adaptive replacement")).into());
        }
    }
    if development.len() != if fresh_only { 0 } else { 256 } || fresh.len() != 64 {
        return Err(invalid("diversity fixed512/128 row design differs").into());
    }
    role_diversity::validate_literal_coverage(
        training_plan.unwrap_or(&json!({"histories":development})),
        &json!({"histories":fresh}),
    )?;
    Ok((development, fresh))
}

fn run(a: &Args, t: Instant) -> Result<Value> {
    let fresh_only = validate_author(a)?;
    let training_plan = if let Some(reference) = &a.fresh_only_training_plan {
        verify(&reference.root, &reference.manifest_sha256)?;
        let plan = read_json(&reference.root.join("plan.json"))?;
        validate_fresh_training_plan(&plan)?;
        Some(plan)
    } else {
        None
    };
    verify(&a.input_bundle, &a.input_manifest_sha256)?;
    let frozen = read_json(&a.input_bundle.join("frozen-inputs.json"))?;
    let train = rows(&frozen["training"])?;
    let dev = rows(&frozen["development"])?;
    if train.len() != 512 || dev.len() != 128 {
        return Err(invalid("retained source512/128 cardinality differs").into());
    }
    let repeat = rows(&frozen["factor_inputs"]["repeated_values"])?;
    if repeat.len() != 16 {
        return Err(invalid("opened repetition donor16 absent").into());
    }
    let mut all = train.clone();
    all.extend(dev.clone());
    let mut opened = BTreeSet::new();
    opened_strings(&frozen, &mut opened);
    let mut opened_values = BTreeSet::new();
    let mut opened_queries = BTreeSet::new();
    collect_opened_wires(&frozen, &mut opened_values, &mut opened_queries)?;
    let mut exposed_banks = BTreeSet::new();
    let mut exposed_turn_histories = BTreeSet::new();
    let mut history_exposure_roots = Vec::new();
    let mut wire_exposure_coverage = Vec::new();
    let mut exposed_history = BTreeSet::new();
    let mut exposed_numerical = BTreeSet::new();
    if a.exposed_roots.is_empty() {
        return Err(
            invalid("explicit complete opened-source exposure bundle list required").into(),
        );
    }
    for e in &a.exposed_roots {
        verify(&e.root, &e.manifest_sha256)?;
        let source = if e.root.join("frozen-inputs.json").is_file() {
            read_json(&e.root.join("frozen-inputs.json"))?
        } else if e.root.join("plan.json").is_file() {
            let v = read_json(&e.root.join("plan.json"))?;
            history_exposure_roots.push(e.root.clone());
            if let Some(hs) = v["histories"].as_array() {
                for h in hs {
                    let h: History = serde_json::from_value(h.clone())?;
                    exposed_history.extend(semantic_history(&h)?);
                    let (bank, history) = semantic_identity(&h)?;
                    exposed_banks.insert(bank);
                    exposed_turn_histories.insert(history);
                }
            }
            v
        } else if e.root.join("inputs.json").is_file() {
            let v = read_json(&e.root.join("inputs.json"))?;
            exposed_numerical.extend(numerical_packets(&v)?);
            if e.root.join("raw-cue-provenance.json").is_file() {
                let provenance = read_json(&e.root.join("raw-cue-provenance.json"))?;
                opened_strings(&provenance, &mut opened);
                collect_opened_wires(&provenance, &mut opened_values, &mut opened_queries)?;
            }
            v
        } else if e.root.join("raw-cue-provenance.json").is_file() {
            read_json(&e.root.join("raw-cue-provenance.json"))?
        } else {
            return Err(invalid("unrecognized opened source bundle;do not silently ignore").into());
        };
        collect_opened_wires(&source, &mut opened_values, &mut opened_queries)?;
        let mut wire_counts = opened_wire_counts(&source);
        if e.root.join("derived-source-origin.json").is_file() {
            let origin = read_json(&e.root.join("derived-source-origin.json"))?;
            collect_opened_wires(&origin, &mut opened_values, &mut opened_queries)?;
            let n = opened_wire_counts(&origin);
            wire_counts.0 += n.0;
            wire_counts.1 += n.1;
        }
        wire_exposure_coverage.push(json!({"root":e.root,"manifest_sha256":e.manifest_sha256,"recognized_write_wires":wire_counts.0,"recognized_query_wires":wire_counts.1,"literal_query_exclusion_scope":"recognized complete Wire objects; supplemental sealed construction plans bind numerical-only roots;not universal pretraining exclusion"}));
        opened_strings(&source, &mut opened);
    }
    let diverse = a.transfer_profile == TransferProfile::SupportedProspectiveRoleDiversity;
    let mut development = Vec::new();
    let mut fresh = Vec::new();
    if !diverse {
        for length in [2, 4, 8] {
            for i in 0..8 {
                let (j, h) = opposite_pair(&all, length, i)?;
                development.extend(chronologies(
                    &format!("words{length}-{i:02}"),
                    &format!("length{length}/order-pair"),
                    j,
                    h,
                    vec![],
                    qpair(&all, i)?,
                ));
            }
        }
        for i in 0..4 {
            let j = pool(&repeat, "job", "assert", None)?;
            let h = pool(&repeat, "home", "assert", None)?;
            let jj = j[i % j.len()].clone();
            let hh = h[(i + 1) % h.len()].clone();
            if literal(&jj)? == literal(&hh)? {
                return Err(invalid("fixed repeat pair role values equal").into());
            }
            development.extend(chronologies(
                &format!("repeat-{i:02}"),
                "repetition/order-pair",
                jj,
                hh,
                vec![],
                qpair(&all, i)?,
            ));
        }
        for i in 0..4 {
            let (j, h) = opposite_pair(&all, 2, i)?;
            let role = if i % 2 == 0 { "job" } else { "home" };
            let updates = pool(&all, role, "update", Some(4))?;
            let u = updates[i % updates.len()].clone();
            let other = if role == "job" { h.clone() } else { j.clone() };
            // A real changed-value correction followed by same-value reassert of other role.
            development.extend(chronologies(
                &format!("version-{i:02}"),
                "current-update/same-value-reassert/order-pair",
                j,
                h,
                vec![u, other],
                qpair(&all, i)?,
            ));
        }
        if development.len() != 64 {
            return Err(invalid("prospective development construction count differs").into());
        }
        // Known frames/words, unseen fixed compositions. These constants are authored before
        // predictions; any public eligibility/exposure failure aborts, never adaptive replacement.
        let fresh_literals: [(&str, &str, usize); 8] = [
            ("cedar amber", "meadow copper", 2),
            ("harbor willow", "orchard violet", 2),
            (
                "cedar amber meadow copper",
                "willow violet orchard harbor",
                4,
            ),
            ("birch silver cedar violet", "meadow copper willow amber", 4),
            (
                "cedar amber meadow copper willow violet orchard harbor",
                "birch silver willow amber meadow cedar copper violet",
                8,
            ),
            (
                "orchard violet harbor willow cedar amber meadow copper",
                "willow amber cedar silver birch violet meadow copper",
                8,
            ),
            (
                "cedar cedar meadow meadow",
                "willow willow harbor harbor",
                4,
            ),
            (
                "violet meadow cedar amber",
                "copper orchard willow harbor",
                4,
            ),
        ];
        let jf = pool(&train, "job", "assert", None)?;
        let hf = pool(&train, "home", "assert", None)?;
        let ju = pool(&train, "job", "update", None)?;
        for (i, (jv, hv, _)) in fresh_literals.iter().enumerate() {
            let j = substituted(&jf[i % jf.len()], jv)?;
            let h = substituted(&hf[i % hf.len()], hv)?;
            let after = if i == 7 {
                vec![
                    substituted(&ju[i % ju.len()], "willow copper meadow amber")?,
                    h.clone(),
                ]
            } else {
                vec![]
            };
            fresh.extend(chronologies(
                &format!("prospective-{i:02}"),
                if i == 6 {
                    "repetition/order-pair"
                } else if i == 7 {
                    "current-update/same-value-reassert/order-pair"
                } else {
                    "new-composition/order-pair"
                },
                j,
                h,
                after,
                qpair(&train, i)?,
            ));
        }
    }
    if diverse {
        (development, fresh) = if let Some(training_plan) = training_plan.as_ref() {
            diversity_histories_for_mode(&all, &repeat, &exposed_banks, Some(training_plan))?
        } else {
            diversity_histories(&all, &repeat, &exposed_banks)?
        };
    }
    let untouched = a.transfer_profile == TransferProfile::SupportedUntouchedComposition;
    if (untouched || diverse)
        && a.assertion_query_policy != AssertionQueryPolicy::SupportedCurrentRole
    {
        return Err(
            invalid("untouched profile requires explicit supported-current-role policy").into(),
        );
    }
    let frame_donors = if untouched {
        let (hs, ds) = untouched_histories(&train)?;
        fresh = hs;
        ds
    } else {
        BTreeMap::new()
    };
    let (development_origins, fresh_origins) = match a.assertion_query_policy {
        AssertionQueryPolicy::RetainedOriginal => (Vec::new(), Vec::new()),
        AssertionQueryPolicy::SupportedCurrentRole => (
            apply_supported_policy_mode(&mut development, diverse)?,
            if untouched {
                apply_untouched_policy(&mut fresh, &frame_donors)?
            } else {
                apply_supported_policy_mode(&mut fresh, diverse)?
            },
        ),
    };
    let mut transfer_banks = BTreeSet::new();
    let mut transfer_histories = BTreeSet::new();
    if untouched {
        for h in &development {
            let (bank, history) = semantic_identity(h)?;
            exposed_banks.insert(bank);
            exposed_turn_histories.insert(history);
            for w in &h.turns {
                opened_values.insert(literal(w)?.to_owned());
            }
            for q in &h.queries {
                opened_queries.insert(q.text.clone());
            }
        }
        for h in &fresh {
            let (bank, history) = semantic_identity(h)?;
            if exposed_banks.contains(&bank)
                || exposed_turn_histories.contains(&history)
                || h.turns
                    .iter()
                    .any(|w| literal(w).is_ok_and(|v| opened_values.contains(v)))
                || (h.stratum.contains("/novel/")
                    && h.queries.iter().any(|q| opened_queries.contains(&q.text)))
            {
                return Err(invalid(
                    "fixed untouched value/query/bank/history already exposed;no redraw",
                )
                .into());
            }
            transfer_banks.insert(bank);
            transfer_histories.insert(history);
        }
        if fresh.len() != 16 || transfer_banks.len() != 4 || transfer_histories.len() != 8 {
            return Err(invalid("matched untouched bank/history cardinality differs").into());
        }
    }
    if diverse {
        let development_banks = development
            .iter()
            .map(semantic_identity)
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(|(b, _)| b)
            .collect::<BTreeSet<_>>();
        let development_histories = development
            .iter()
            .map(semantic_identity)
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(|(_, h)| h)
            .collect::<BTreeSet<_>>();
        for h in &fresh {
            let (bank, history) = semantic_identity(h)?;
            if development_banks.contains(&bank)
                || exposed_banks.contains(&bank)
                || development_histories.contains(&history)
                || exposed_turn_histories.contains(&history)
            {
                return Err(invalid(
                    "diversity complete bank/history partition intersects opened/development",
                )
                .into());
            }
        }
    }
    let mut dev_fp = BTreeSet::new();
    for h in &development {
        for fp in semantic_history(h)? {
            if !dev_fp.insert(fp) {
                return Err(invalid("development effective-bank/query duplicate").into());
            }
        }
    }
    let mut fresh_fp = BTreeSet::new();
    for h in &fresh {
        for w in &h.turns {
            if !diverse && opened.contains(&w.text) {
                return Err(invalid(
                    "fixed prospective original statement already opened;no automatic redraw",
                )
                .into());
            }
        }
        for fp in semantic_history(h)? {
            if dev_fp.contains(&fp) || exposed_history.contains(&fp) || !fresh_fp.insert(fp) {
                return Err(invalid("fresh effective bank/query duplicate/exposure").into());
            }
        }
    }
    if sha256_file(&a.trusted_binding)? != a.trusted_binding_sha256 {
        return Err(invalid("public parent binding changed").into());
    }
    let binding: NativeArtifactBinding = serde_json::from_slice(&fs::read(&a.trusted_binding)?)?;
    let native = NativeSourceRealizer::load_native(&a.native_artifact, &binding)?;
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(
        a.native_artifact.join("tokenizer.json"),
    )?)
    .ok_or_else(|| invalid("bound ByteBPE absent"))?;
    let mut dev_numerical = BTreeSet::new();
    for h in &development {
        for fp in numerical_history(h, &tok)? {
            if !dev_numerical.insert(fp) {
                return Err(invalid("development numerical packet duplicate").into());
            }
        }
    }
    let mut fresh_numerical = BTreeSet::new();
    for h in &fresh {
        for fp in numerical_history(h, &tok)? {
            if dev_numerical.contains(&fp)
                || exposed_numerical.contains(&fp)
                || !fresh_numerical.insert(fp)
            {
                return Err(invalid("prospective numerical packet duplicate/exposure").into());
            }
        }
    }
    let mut eligibility = Vec::new();
    for h in development.iter().chain(&fresh) {
        if t.elapsed().as_secs() > a.maximum_seconds {
            return Err(invalid("source authoring wall cap").into());
        }
        eligibility.push(eligible(h, &native, &tok)?);
    }
    let outputs = a
        .development_out
        .iter()
        .map(|out| (out, "development", &development, &dev_fp))
        .chain(std::iter::once((&a.fresh_out, "fresh", &fresh, &fresh_fp)));
    for (out, split, histories, fp) in outputs {
        let plan = json!({"schema":"uor-r4.raw-natural-reader-construction-plan/1","split":split,"source_policy":a.assertion_query_policy.source_policy(),"histories":histories});
        let planbytes = serde_json::to_vec_pretty(&plan)?;
        let diversity_control = if diverse {
            Some(role_diversity::validate(&plan)?)
        } else {
            None
        };
        let origins = if split == "development" {
            &development_origins
        } else {
            &fresh_origins
        };
        let origin_bytes = if a.assertion_query_policy == AssertionQueryPolicy::SupportedCurrentRole
        {
            Some(serde_json::to_vec_pretty(
                &json!({"schema":"uor-r4.supported-current-role-source-origin/1","policy":a.assertion_query_policy,"transfer_profile":a.transfer_profile,"source_bundle_manifest_sha256":a.input_manifest_sha256,"histories":origins,"scope":if diverse {"exact retained donor literal bytes and role/action witnesses; prospectively rewritten current-role frames; known-literal new bank compositions; both literal roles/order and crossed familiar wording; not unseen literals or words"} else if untouched && split=="fresh" {"fixed generated fresh literals in actual retained donor frames;role/action/chronology preserved;matched familiar and new query forms;novel full literals not novel words"} else {"original donor values/chronology retained;assertion and query frames newly authored;payload novelty not claimed"}}),
            )?)
        } else {
            None
        };
        let mut receipt = json!({"schema":"uor-r4.raw-natural-reader-construction-authoring/1","split":split,"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"source_bundle_manifest_sha256":a.input_manifest_sha256,"histories":histories.len(),"resulting_allbank_queries":histories.len()*2,"query_role_balance":"both natural job/home questions on every identical bank","chronology_pairs":histories.len()/2,"assertion_query_policy":a.assertion_query_policy,"source_origin":if a.assertion_query_policy==AssertionQueryPolicy::RetainedOriginal{"retained exact utterances512/128 +opened repetition16;fresh fixed known-frame compositions"}else if diverse {"exact retained donor literal bytes; new role/value bank combinations; balanced literal roles/positions and crossed familiar wording; not retained-original utterance bytes or unseen literals"}else if untouched && split=="fresh" {"fixed generated fresh literal bytes in retained donor frames;original frame witnesses bound;prospectively authored explicit-current-role assertion/query frames;not retained-original utterance bytes"}else{"retained donor literal bytes and chronology;prospectively authored explicit-current-role assertion/query frames;not retained-original utterance bytes"},"derived_source_origin_sha256":origin_bytes.as_ref().map(|bytes|sha256_bytes(bytes)),"transfer_profile":a.transfer_profile,"prospective_reassert_donor_policy":if diverse {Some("ordinary-retained-both-role-two-word-assertion-witnesses/authored-same-value-repeat/1")}else{None},"prospective_diversity_control":diversity_control,"fresh_payload_novelty_claimed":untouched && split=="fresh","transfer_novelty":if untouched && split=="fresh" {json!({"distinct_current_role_value_banks":transfer_banks,"distinct_role_act_literal_histories":transfer_histories,"matched_question_groups":["familiar","novel"],"history_exclusion_covered_roots":history_exposure_roots,"wire_extraction_coverage":wire_exposure_coverage,"familiar_queries_intentionally_overlap":true,"novel_queries":[TRANSFER_JOB_QUERY,TRANSFER_HOME_QUERY],"scope":"exact full-literal, question-string, role/value-bank and role/act/literal-history exclusion;not unseen words or universal pretraining exclusion"})}else{Value::Null},"source_label_scope":"offline construction only;not learned compiler result","semantic_fingerprints":fp,"numerical_packet_fingerprints":if split=="development"{&dev_numerical}else{&fresh_numerical},"exposed_roots":a.exposed_roots.iter().map(|e|json!({"root":e.root,"manifest_sha256":e.manifest_sha256})).collect::<Vec<_>>(),"plan_sha256":sha256_bytes(&planbytes),"public_formatter_eligibility":"all rows before prediction;support probabilities NOT_RUN","native_predictions":"NOT_RUN","learning_seed":"NOT_APPLICABLE","elapsed_seconds":t.elapsed().as_secs_f64()});
        if fresh_only {
            let reference = a
                .fresh_only_training_plan
                .as_ref()
                .ok_or_else(|| invalid("fresh-only training reference absent"))?;
            receipt["fresh_only_authoring"] = json!({"mode":"author-fresh-only","development_output_created":false,"fixed_bank_blocks":{"length2":2,"length4":2,"length8":2,"update":1,"reassert":1},"literal_coverage_training_plan":{"root":reference.root,"manifest_sha256":reference.manifest_sha256,"plan_sha256":sha256_file(&reference.root.join("plan.json"))?},"literal_coverage":"all fresh full literals witnessed in bound existing development512 plan; no newly created sibling development"});
        }
        let rb = serde_json::to_vec_pretty(&receipt)?;
        let eb = serde_json::to_vec_pretty(
            &json!({"rows":eligibility.iter().filter(|x|histories.iter().any(|h|x["history"]==h.id)).collect::<Vec<_>>()}),
        )?;
        if planbytes.len() + rb.len() + eb.len() + origin_bytes.as_ref().map_or(0, Vec::len)
            > a.maximum_report_bytes
        {
            return Err(invalid("source plan report cap").into());
        }
        if let Some(bytes) = origin_bytes {
            fs::write(out.join("derived-source-origin.json"), bytes)?;
        }
        fs::write(out.join("plan.json"), planbytes)?;
        fs::write(out.join("report.json"), rb)?;
        fs::write(out.join("public-eligibility.json"), eb)?;
    }
    verify(&a.input_bundle, &a.input_manifest_sha256)?;
    for e in &a.exposed_roots {
        verify(&e.root, &e.manifest_sha256)?;
    }
    if let Some(reference) = &a.fresh_only_training_plan {
        verify(&reference.root, &reference.manifest_sha256)?;
    }
    if sha256_file(&a.trusted_binding)? != a.trusted_binding_sha256 {
        return Err(invalid("public parent binding changed during source authoring").into());
    }
    Ok(
        json!({"status":"COMPLETED","development_histories":development.len(),"fresh_histories":fresh.len(),"native_predictions":"NOT_RUN"}),
    )
}
fn validate_fresh_training_plan(plan: &Value) -> Result<()> {
    if plan["schema"] != "uor-r4.raw-natural-reader-construction-plan/1"
        || plan["split"] != "development"
        || plan["source_policy"] != AssertionQueryPolicy::SupportedCurrentRole.source_policy()
    {
        return Err(
            invalid("fresh-only coverage requires existing supported development plan").into(),
        );
    }
    role_diversity::validate(plan)?;
    Ok(())
}
fn validate_author(a: &Args) -> Result<bool> {
    if a.mode
        .as_deref()
        .is_some_and(|m| m != "author" && m != "author-fresh-only")
    {
        return Err(invalid("unknown plan authoring mode").into());
    }
    let fresh_only = a.mode.as_deref() == Some("author-fresh-only");
    if fresh_only != a.fresh_only_training_plan.is_some()
        || fresh_only == a.development_out.is_some()
        || (fresh_only && a.transfer_profile != TransferProfile::SupportedProspectiveRoleDiversity)
    {
        return Err(invalid("fresh-only explicit mode/reference/output contract differs").into());
    }
    if let Some(reference) = &a.fresh_only_training_plan {
        if reference.manifest_sha256.len() != 64
            || !reference
                .manifest_sha256
                .bytes()
                .all(|c| c.is_ascii_hexdigit())
            || !a
                .exposed_roots
                .iter()
                .any(|e| e.root == reference.root && e.manifest_sha256 == reference.manifest_sha256)
        {
            return Err(invalid(
                "fresh-only training reference must be an exact bound exposed plan",
            )
            .into());
        }
    }
    if a.schema != "uor-r4.native-bank-observation-plan-args/1"
        || (a.transfer_profile == TransferProfile::SupportedProspectiveRoleDiversity
            && a.assertion_query_policy != AssertionQueryPolicy::SupportedCurrentRole)
        || a.maximum_seconds == 0
        || a.maximum_seconds > 300
        || a.maximum_report_bytes == 0
        || a.maximum_report_bytes > 16 * 1024 * 1024
    {
        return Err(invalid("authorer argument scope/caps invalid").into());
    }
    Ok(fresh_only)
}
fn author(a: Args) -> Result<()> {
    validate_author(&a)?;
    let dev = a
        .development_out
        .as_ref()
        .map(|p| output_support::prospective_output(p))
        .transpose()?;
    let fresh = output_support::prospective_output(&a.fresh_out)?;
    if dev
        .as_ref()
        .is_some_and(|dev| dev.starts_with(&fresh) || fresh.starts_with(dev))
    {
        return Err(invalid("plan outputs overlap").into());
    }
    let outputs = dev
        .iter()
        .chain(std::iter::once(&fresh))
        .collect::<Vec<_>>();
    for p in [&a.input_bundle, &a.native_artifact, &a.trusted_binding] {
        let p = fs::canonicalize(p)?;
        if outputs
            .iter()
            .any(|out| out.starts_with(&p) || p.starts_with(out))
        {
            return Err(invalid("output/input intersection").into());
        }
    }
    for e in &a.exposed_roots {
        let p = fs::canonicalize(&e.root)?;
        if outputs
            .iter()
            .any(|out| out.starts_with(&p) || p.starts_with(out))
        {
            return Err(invalid("output/exposure intersection").into());
        }
    }
    for out in &outputs {
        report_output::claim(out)?;
    }
    let t = Instant::now();
    let result = run(&a, t);
    if let Err(e) = &result {
        for p in &outputs {
            fs::write(
                p.join("failure.json"),
                serde_json::to_vec_pretty(
                    &json!({"error":e.to_string(),"native_predictions":"NOT_RUN","elapsed_seconds":t.elapsed().as_secs_f64()}),
                )?,
            )?;
        }
    }
    for p in &outputs {
        report_output::seal(p)?;
        report_output::verify(p)?;
    }
    result.map(|_| ())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BundleArgs {
    schema: String,
    mode: String,
    source_frozen_inputs: PathBuf,
    source_sha256: String,
    source_provenance_receipt: Option<PathBuf>,
    source_provenance_receipt_sha256: Option<String>,
    out: PathBuf,
    maximum_report_bytes: usize,
}
fn input_bundle(a: BundleArgs) -> Result<()> {
    if a.schema != "uor-r4.native-bank-observation-input-bundle-args/1"
        || a.mode != "input-bundle"
        || a.maximum_report_bytes == 0
        || a.maximum_report_bytes > 16 * 1024 * 1024
        || a.source_sha256.len() != 64
        || a.source_provenance_receipt.is_some() != a.source_provenance_receipt_sha256.is_some()
    {
        return Err(invalid("input-bundle arguments/bounds invalid").into());
    }
    let output = output_support::prospective_output(&a.out)?;
    for p in std::iter::once(&a.source_frozen_inputs).chain(a.source_provenance_receipt.iter()) {
        let original = fs::canonicalize(p)?;
        if output.starts_with(&original) || original.starts_with(&output) {
            return Err(invalid("input-bundle output intersects retained subset input").into());
        }
    }
    report_output::claim(&a.out)?;
    let result = (|| -> Result<()> {
        let bytes = fs::read(&a.source_frozen_inputs)?;
        if sha256_bytes(&bytes) != a.source_sha256 {
            return Err(invalid("retained frozen-input subset SHA differs").into());
        }
        let data: Value = serde_json::from_slice(&bytes)?;
        if data["schema"] != "native-turn-frozen-inputs/1"
            || data["training"].as_array().map(Vec::len) != Some(512)
            || data["development"].as_array().map(Vec::len) != Some(128)
            || data["factor_inputs"]["repeated_values"]
                .as_array()
                .map(Vec::len)
                != Some(16)
        {
            return Err(invalid("retained source512/128/repeat16 schema/count differs").into());
        }
        let mut provenance = None;
        if let Some(path) = &a.source_provenance_receipt {
            let receipt = fs::read(path)?;
            if Some(sha256_bytes(&receipt)) != a.source_provenance_receipt_sha256 {
                return Err(invalid("retained provenance receipt SHA differs").into());
            }
            provenance = Some(receipt);
        }
        let report = json!({"schema":"uor-r4.native-bank-observation-retained-input-bundle/1","status":"COMPLETED","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"retained_source_file":a.source_frozen_inputs,"retained_source_file_sha256":a.source_sha256,"retained_source_file_bytes":bytes.len(),"source_provenance_receipt":a.source_provenance_receipt,"source_provenance_receipt_sha256":a.source_provenance_receipt_sha256,"training":512,"development":128,"opened_repetition":16,"original_full_report_seal_verified":false,"provenance_scope":"new complete seal of exact retained subset bytes;not a reconstruction or verification of original full report","fresh_rows_status":"existing fresh in copied input are exposed exclusions, never relabelled prospective fresh","native_predictions":"NOT_RUN","data_authoring":"NOT_RUN;byte-identical copy only"});
        let receipt = serde_json::to_vec_pretty(&report)?;
        if bytes.len() + receipt.len() + provenance.as_ref().map_or(0, Vec::len)
            > a.maximum_report_bytes
        {
            return Err(invalid("retained input-bundle cap").into());
        }
        fs::write(a.out.join("frozen-inputs.json"), &bytes)?;
        if let Some(p) = provenance {
            fs::write(a.out.join("retained-provenance-receipt.json"), p)?;
        }
        fs::write(a.out.join("report.json"), receipt)?;
        if sha256_file(&a.source_frozen_inputs)? != a.source_sha256
            || sha256_file(&a.out.join("frozen-inputs.json"))? != a.source_sha256
        {
            return Err(invalid("retained byte identity changed during copy").into());
        }
        if let Some(path) = &a.source_provenance_receipt {
            if Some(sha256_file(path)?) != a.source_provenance_receipt_sha256 {
                return Err(invalid("retained provenance receipt changed during copy").into());
            }
        }
        Ok(())
    })();
    if let Err(e) = &result {
        fs::write(
            a.out.join("failure.json"),
            serde_json::to_vec_pretty(
                &json!({"error":e.to_string(),"native_predictions":"NOT_RUN","data_authoring":"NOT_RUN"}),
            )?,
        )?;
    }
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result
}
fn main() -> Result<()> {
    let config = std::env::args()
        .nth(1)
        .ok_or_else(|| invalid("usage: geometric-native-bank-observation-plan CONFIG.json"))?;
    let value: Value = serde_json::from_slice(&fs::read(config)?)?;
    if value["mode"] == "input-bundle" {
        input_bundle(serde_json::from_value(value)?)
    } else {
        author(serde_json::from_value(value)?)
    }
}

#[cfg(test)]
mod supported_current_role_tests {
    use super::*;
    #[test]
    fn exposure_scanner_distinguishes_metadata_from_complete_wires() -> Result<()> {
        let metadata = json!({"act":"assert","relation":"home","text":"I am now {v}.","source_template":"I moved to {v}."});
        let wire = json!({"act":"assert","relation":"home","text":"I currently live in copper cedar.","template":"I currently live in {v}."});
        let source = json!({"rows":[metadata, wire],"reference_templates_control_only":[{"act":"assert","relation":"home","text":"I live in .","template":"I live in {v}."}]});
        let mut values = BTreeSet::new();
        let mut queries = BTreeSet::new();
        collect_opened_wires(&source, &mut values, &mut queries)?;
        assert_eq!(values, BTreeSet::from(["copper cedar".to_owned()]));
        assert!(queries.is_empty());
        assert_eq!(opened_wire_counts(&source), (1, 0));
        let malformed = json!({"act":"assert","relation":"home","text":"unbound text","template":"I currently live in {v}."});
        assert!(collect_opened_wires(&malformed, &mut values, &mut queries).is_err());
        Ok(())
    }
    #[test]
    fn untouched_profile_preserves_matched_banks_and_binds_actual_frames() -> Result<()> {
        assert_eq!(
            TransferProfile::default(),
            TransferProfile::RetainedComposition
        );
        let mut source = Vec::new();
        for role in ["job", "home"] {
            for act in ["assert", "update"] {
                source.push(Wire {
                    text: "Legacy known label.".into(),
                    relation: role.into(),
                    act: act.into(),
                    template: Some("Legacy {v}.".into()),
                });
            }
            source.push(Wire {
                text: format!("Legacy {role} question?"),
                relation: role.into(),
                act: "query".into(),
                template: None,
            });
        }
        let (mut histories, donors) = untouched_histories(&source)?;
        let origins = apply_untouched_policy(&mut histories, &donors)?;
        assert_eq!(histories.len(), 16);
        let banks = histories
            .iter()
            .map(semantic_identity)
            .collect::<Result<Vec<_>>>()?;
        assert_eq!(banks.iter().map(|x| &x.0).collect::<BTreeSet<_>>().len(), 4);
        assert_eq!(banks.iter().map(|x| &x.1).collect::<BTreeSet<_>>().len(), 8);
        for block in histories.chunks_exact(4) {
            for order in 0..2 {
                assert_eq!(
                    semantic_identity(&block[order])?,
                    semantic_identity(&block[order + 2])?
                );
                assert_eq!(block[order].queries[0].text, CURRENT_JOB_QUERIES[0]);
                assert_eq!(block[order + 2].queries[0].text, TRANSFER_JOB_QUERY);
                assert_eq!(block[order].queries[1].text, CURRENT_HOME_QUERIES[0]);
                assert_eq!(block[order + 2].queries[1].text, TRANSFER_HOME_QUERY);
            }
        }
        for origin in origins {
            for receipt in origin["turns"]
                .as_array()
                .ok_or_else(|| invalid("origins absent"))?
            {
                let retained: Wire =
                    serde_json::from_value(receipt["original_frame_donor_wire"].clone())?;
                let derived: Wire =
                    serde_json::from_value(receipt["donor_construction_wire"].clone())?;
                let authored: Wire = serde_json::from_value(receipt["authored_wire"].clone())?;
                assert!(source
                    .iter()
                    .any(|w| serde_json::to_value(w).ok() == serde_json::to_value(&retained).ok()));
                assert_eq!(retained.template, derived.template);
                assert_eq!(retained.relation, derived.relation);
                assert_eq!(retained.act, derived.act);
                assert_eq!(literal(&derived)?, literal(&authored)?);
            }
        }
        Ok(())
    }
    #[test]
    fn semantic_transfer_identity_ignores_frame_query_but_retains_role_act_order() -> Result<()> {
        let job = Wire {
            text: "Old amber willow.".into(),
            relation: "job".into(),
            act: "assert".into(),
            template: Some("Old {v}.".into()),
        };
        let home = Wire {
            text: "Old copper cedar.".into(),
            relation: "home".into(),
            act: "assert".into(),
            template: Some("Old {v}.".into()),
        };
        let mut h = History {
            id: "x".into(),
            stratum: "test".into(),
            turns: vec![job, home],
            queries: vec![],
        };
        let before = semantic_identity(&h)?;
        apply_supported_policy(std::slice::from_mut(&mut h))?;
        assert_eq!(before, semantic_identity(&h)?);
        h.turns.reverse();
        let after = semantic_identity(&h)?;
        assert_eq!(before.0, after.0);
        assert_ne!(before.1, after.1);
        h.turns[0].act = "update".into();
        assert_ne!(after.1, semantic_identity(&h)?.1);
        Ok(())
    }
    #[test]
    fn rejects_unentailed_question_aliases_and_wrong_roles() -> Result<()> {
        for text in [
            "Where am I from?",
            "What is my address?",
            "What town do I live in?",
            "What field do I work in?",
            "What tasks do I do?",
        ] {
            for role in ["job", "home"] {
                assert!(supported_contract(&Wire {
                    text: text.into(),
                    relation: role.into(),
                    act: "query".into(),
                    template: Some(text.into())
                })
                .is_err());
            }
        }
        for text in CURRENT_JOB_QUERIES {
            assert!(supported_contract(&Wire {
                text: text.into(),
                relation: "home".into(),
                act: "query".into(),
                template: Some(text.into())
            })
            .is_err());
        }
        for text in CURRENT_HOME_QUERIES {
            assert!(supported_contract(&Wire {
                text: text.into(),
                relation: "job".into(),
                act: "query".into(),
                template: Some(text.into())
            })
            .is_err());
        }
        Ok(())
    }
    #[test]
    fn explicit_assertion_and_queries_preserve_literal_role_act() -> Result<()> {
        for role in ["job", "home"] {
            let mut write_templates = BTreeSet::new();
            for act in ["assert", "update"] {
                let old = Wire {
                    text: "Legacy cedar cedar meadow meadow.".into(),
                    relation: role.into(),
                    act: act.into(),
                    template: Some("Legacy {v}.".into()),
                };
                let new = supported_wire(&old, 0)?;
                supported_contract(&new)?;
                assert_eq!(literal(&old)?, literal(&new)?);
                assert_eq!(old.relation, new.relation);
                assert_eq!(old.act, new.act);
                assert!(write_templates.insert(new.template.clone()));
                let mut wrong_act = new;
                wrong_act.act = if act == "assert" { "update" } else { "assert" }.into();
                assert!(supported_contract(&wrong_act).is_err());
            }
            for i in 0..6 {
                let old = Wire {
                    text: "Legacy question?".into(),
                    relation: role.into(),
                    act: "query".into(),
                    template: None,
                };
                supported_contract(&supported_wire(&old, i)?)?;
            }
        }
        Ok(())
    }
    #[test]
    fn policy_default_and_paired_history_chronology_are_explicit() -> Result<()> {
        assert_eq!(
            AssertionQueryPolicy::default(),
            AssertionQueryPolicy::RetainedOriginal
        );
        let j = Wire {
            text: "Legacy cedar.".into(),
            relation: "job".into(),
            act: "assert".into(),
            template: Some("Legacy {v}.".into()),
        };
        let h = Wire {
            text: "Legacy meadow.".into(),
            relation: "home".into(),
            act: "assert".into(),
            template: Some("Legacy {v}.".into()),
        };
        let update = Wire {
            text: "Legacy copper.".into(),
            relation: "job".into(),
            act: "update".into(),
            template: Some("Legacy {v}.".into()),
        };
        let queries = vec![
            Wire {
                text: "Where am I from?".into(),
                relation: "home".into(),
                act: "query".into(),
                template: None,
            },
            Wire {
                text: "What tasks do I do?".into(),
                relation: "job".into(),
                act: "query".into(),
                template: None,
            },
        ];
        let mut histories = chronologies("pair", "version", j, h.clone(), vec![update, h], queries);
        let before = histories.clone();
        let origins = apply_supported_policy(&mut histories)?;
        assert_eq!(origins.len(), 2);
        for (old, new) in before.iter().zip(&histories) {
            for (old, new) in old.turns.iter().zip(&new.turns) {
                assert_eq!(old.relation, new.relation);
                assert_eq!(old.act, new.act);
                assert_eq!(literal(old)?, literal(new)?);
            }
        }
        assert_eq!(
            histories[0]
                .queries
                .iter()
                .map(|q| &q.text)
                .collect::<Vec<_>>(),
            histories[1]
                .queries
                .iter()
                .map(|q| &q.text)
                .collect::<Vec<_>>()
        );
        assert!(origins[0]["turns"][0]["origin"]
            .as_str()
            .is_some_and(|s| s.contains("not retained original")));
        Ok(())
    }
}

#[cfg(test)]
mod prospective_diversity_tests {
    use super::*;
    fn donors() -> Vec<Wire> {
        donors_with_count(10)
    }
    fn donors_with_count(count: usize) -> Vec<Wire> {
        let mut all = Vec::new();
        for n in [2usize, 4, 8] {
            for i in 0..count {
                let value = (0..n)
                    .map(|j| format!("v{i}w{j}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                for role in ["job", "home"] {
                    for act in ["assert", "update"] {
                        let template = format!("Legacy {role} {act} {{v}}.");
                        all.push(Wire {
                            text: template.replace("{v}", &value),
                            relation: role.into(),
                            act: act.into(),
                            template: Some(template),
                        });
                    }
                }
            }
        }
        for role in ["job", "home"] {
            for i in 0..6 {
                all.push(Wire {
                    text: format!("Legacy {role} query{i}?"),
                    relation: role.into(),
                    act: "query".into(),
                    template: None,
                });
            }
        }
        all
    }
    fn author_args() -> Result<Args> {
        Ok(serde_json::from_value(json!({
            "schema":"uor-r4.native-bank-observation-plan-args/1",
            "assertion_query_policy":"supported-current-role/1",
            "transfer_profile":"supported-prospective-role-diversity/1",
            "input_bundle":"input", "input_manifest_sha256":"a".repeat(64),
            "exposed_roots":[{"root":"trained", "manifest_sha256":"b".repeat(64)}],
            "native_artifact":"native", "trusted_binding":"binding",
            "trusted_binding_sha256":"c".repeat(64),
            "development_out":"development", "fresh_out":"fresh",
            "maximum_seconds":300, "maximum_report_bytes":16777216
        }))?)
    }
    #[test]
    fn fresh_only_mode_requires_bound_reference_and_no_development_output() -> Result<()> {
        let mut a = author_args()?;
        assert!(!validate_author(&a)?); // Historical absent mode remains author.
        a.mode = Some("author".into());
        assert!(!validate_author(&a)?);
        a.mode = Some("author-fresh-only".into());
        assert!(validate_author(&a).is_err());
        a.fresh_only_training_plan = Some(BoundRoot {
            root: "trained".into(),
            manifest_sha256: "b".repeat(64),
        });
        assert!(validate_author(&a).is_err()); // Even an unused sibling path is rejected.
        a.development_out = None;
        assert!(validate_author(&a)?);
        a.exposed_roots.clear();
        assert!(validate_author(&a).is_err());
        a.exposed_roots.push(BoundRoot {
            root: "trained".into(),
            manifest_sha256: "d".repeat(64),
        });
        assert!(validate_author(&a).is_err());
        a.exposed_roots[0].manifest_sha256 = "b".repeat(64);
        a.mode = Some("author".into());
        assert!(validate_author(&a).is_err());
        a.mode = Some("author-fresh-only".into());
        a.transfer_profile = TransferProfile::RetainedComposition;
        assert!(validate_author(&a).is_err());
        Ok(())
    }
    #[test]
    fn fresh_only_preserves_fixed_schedule_and_binds_existing_trained_literal_coverage(
    ) -> Result<()> {
        let all = donors_with_count(8);
        let (mut dev, oldfresh) = diversity_histories(&all, &[], &BTreeSet::new())?;
        assert_eq!(dev.len(), 256);
        assert_eq!(oldfresh.len(), 64);
        let excluded = dev
            .iter()
            .chain(&oldfresh)
            .map(semantic_identity)
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(|(bank, _)| bank)
            .collect::<BTreeSet<_>>();
        // The old full design consumes scarce edges on a never-used sibling dev.
        assert!(diversity_histories(&all, &[], &excluded).is_err());
        apply_supported_policy_mode(&mut dev, true)?;
        let training = json!({"schema":"uor-r4.raw-natural-reader-construction-plan/1",
            "split":"development", "source_policy":AssertionQueryPolicy::SupportedCurrentRole.source_policy(),
            "histories":dev});
        let (empty, mut fresh) =
            diversity_histories_for_mode(&all, &[], &excluded, Some(&training))?;
        assert!(empty.is_empty());
        assert_eq!(fresh.len(), 64);
        let (_, again) = diversity_histories_for_mode(&all, &[], &excluded, Some(&training))?;
        assert_eq!(serde_json::to_value(&fresh)?, serde_json::to_value(&again)?);
        for h in &fresh {
            assert!(!excluded.contains(&semantic_identity(h)?.0));
        }
        apply_supported_policy_mode(&mut fresh, true)?;
        let fp = json!({"split":"fresh","histories":fresh});
        assert_eq!(role_diversity::validate(&fp)?["rows"], 128);
        role_diversity::validate_literal_coverage(&training, &fp)?;
        let mut wrong = training.clone();
        wrong["split"] = json!("fresh");
        assert!(validate_fresh_training_plan(&wrong).is_err());
        wrong = training.clone();
        wrong["histories"]
            .as_array_mut()
            .ok_or_else(|| invalid("fixture histories absent"))?
            .pop();
        assert!(validate_fresh_training_plan(&wrong).is_err());
        let mut unseen = fp.clone();
        let turn = &mut unseen["histories"][0]["turns"][0];
        let template = turn["template"]
            .as_str()
            .ok_or_else(|| invalid("fixture template absent"))?
            .to_owned();
        turn["text"] = json!(template.replace("{v}", "never seen full literal"));
        assert!(role_diversity::validate_literal_coverage(&training, &unseen).is_err());
        Ok(())
    }
    #[test]
    fn diverse_known_literals_are_role_order_wording_balanced_and_whole_bank_partitioned(
    ) -> Result<()> {
        let all = donors();
        let repeats = pool(&all, "job", "assert", Some(2))?
            .into_iter()
            .chain(pool(&all, "home", "assert", Some(2))?)
            .filter(|w| literal(w).is_ok_and(|v| !v.starts_with("v8") && !v.starts_with("v9")))
            .collect::<Vec<_>>();
        let (mut dev, mut fresh) = diversity_histories(&all, &repeats, &BTreeSet::new())?;
        apply_supported_policy_mode(&mut dev, true)?;
        apply_supported_policy_mode(&mut fresh, true)?;
        let dp = json!({"split":"development","histories":dev});
        let fp = json!({"split":"fresh","histories":fresh});
        let dc = role_diversity::validate(&dp)?;
        let fc = role_diversity::validate(&fp)?;
        assert_eq!(dc["rows"], 512);
        assert_eq!(fc["rows"], 128);
        role_diversity::validate_literal_coverage(&dp, &fp)?;
        let length2 = json!({"histories":dev.iter().filter(|h|h.stratum.starts_with("length2/")).collect::<Vec<_>>()});
        let mut degrees = BTreeMap::<String, usize>::new();
        let unordered = role_diversity::bank_keys(&length2)?
            .into_iter()
            .map(|(a, b)| if a < b { (a, b) } else { (b, a) })
            .collect::<BTreeSet<_>>();
        for (a, b) in unordered {
            *degrees.entry(a).or_default() += 1;
            *degrees.entry(b).or_default() += 1;
        }
        assert_eq!(degrees.len(), 10);
        assert!(degrees.values().all(|n| (1..=2).contains(n)));

        assert!(role_diversity::bank_keys(&dp)?.is_disjoint(&role_diversity::bank_keys(&fp)?));
        let mut missing_order = dp.clone();
        missing_order["histories"][1] = missing_order["histories"][0].clone();
        assert!(role_diversity::validate(&missing_order).is_err());
        let mut correlated_wording = dp.clone();
        correlated_wording["histories"][0]["queries"] =
            correlated_wording["histories"][2]["queries"].clone();
        assert!(role_diversity::validate(&correlated_wording).is_err());
        let mut relabelled = fp.clone();
        relabelled["histories"][0]["id"] = json!("different-id");
        relabelled["histories"][0]["queries"] = fp["histories"][2]["queries"].clone();
        assert_eq!(
            role_diversity::bank_keys(&fp)?,
            role_diversity::bank_keys(&relabelled)?
        );
        Ok(())
    }
    #[test]
    fn larger_donor_pool_uses_predeclared_vertex_caps_and_retains_training_witnesses() -> Result<()>
    {
        let all = donors_with_count(20);
        let repeats = pool(&all, "job", "assert", Some(2))?
            .into_iter()
            .chain(pool(&all, "home", "assert", Some(2))?)
            .collect::<Vec<_>>();
        let (mut dev, mut fresh) = diversity_histories(&all, &repeats, &BTreeSet::new())?;
        apply_supported_policy_mode(&mut dev, true)?;
        apply_supported_policy_mode(&mut fresh, true)?;
        let dp = json!({"split":"development","histories":dev});
        let fp = json!({"split":"fresh","histories":fresh});
        role_diversity::validate(&dp)?;
        role_diversity::validate(&fp)?;
        role_diversity::validate_literal_coverage(&dp, &fp)?;
        for (stratum, cap) in [
            ("length2", 10usize),
            ("length4", 10),
            ("length8", 10),
            ("reassert", 8),
        ] {
            let values = dev
                .iter()
                .chain(&fresh)
                .filter(|h| h.stratum.starts_with(&format!("{stratum}/")))
                .flat_map(|h| h.turns.iter())
                .map(literal)
                .collect::<Result<BTreeSet<_>>>()?;
            assert!(values.len() <= cap);
        }
        Ok(())
    }
    #[test]
    fn prospective_reassertions_use_retained_assert_witnesses_without_repeat_corpus() -> Result<()>
    {
        let all = donors();
        let (development, fresh) = diversity_histories(&all, &[], &BTreeSet::new())?;
        let histories = development
            .iter()
            .chain(&fresh)
            .filter(|h| h.stratum.starts_with("reassert/"));
        let mut count = 0;
        for history in histories {
            assert_eq!(history.turns.len(), 3);
            let repeated = &history.turns[2];
            assert_eq!(repeated.act, "assert");
            assert_eq!(literal(repeated)?.split_whitespace().count(), 2);
            let original = history.turns[..2]
                .iter()
                .find(|w| w.relation == repeated.relation)
                .ok_or_else(|| invalid("reassert original role absent"))?;
            assert_eq!(literal(repeated)?, literal(original)?);
            assert_eq!(repeated.text, original.text);
            assert!(history.turns[..2]
                .iter()
                .any(|w| w.relation != repeated.relation));
            count += 1;
        }
        assert_eq!(count, 40);
        Ok(())
    }
    #[test]
    fn prospective_crossings_remain_distinct_after_current_bank_projection() -> Result<()> {
        let all = donors();
        let (mut dev, mut fresh) = diversity_histories(&all, &[], &BTreeSet::new())?;
        apply_supported_policy_mode(&mut dev, true)?;
        apply_supported_policy_mode(&mut fresh, true)?;
        let mut packets = BTreeSet::new();
        for history in dev.iter().chain(&fresh) {
            for fingerprint in semantic_history(history)? {
                assert!(
                    packets.insert(fingerprint),
                    "effective current-bank/query duplicate at {}",
                    history.id
                );
            }
        }
        assert_eq!(packets.len(), 640);
        for stratum in ["reassert/", "update/"] {
            let pair = dev
                .iter()
                .filter(|h| h.stratum.starts_with(stratum))
                .take(2)
                .collect::<Vec<_>>();
            assert_eq!(pair.len(), 2);
            assert_ne!(semantic_history(pair[0])?, semantic_history(pair[1])?);
        }
        let pair = dev
            .iter()
            .filter(|h| h.stratum.starts_with("reassert/"))
            .take(2)
            .collect::<Vec<_>>();
        let mut collapsed = pair[1].clone();
        *collapsed
            .turns
            .last_mut()
            .ok_or_else(|| invalid("fixture trailing write absent"))? = pair[0]
            .turns
            .last()
            .cloned()
            .ok_or_else(|| invalid("fixture reassert absent"))?;
        assert_eq!(semantic_history(pair[0])?, semantic_history(&collapsed)?);
        Ok(())
    }
    #[test]
    fn insufficient_donor_diversity_fails_without_repeating_banks() {
        let all = donors()
            .into_iter()
            .filter(|w| w.act == "query" || literal(w).is_ok_and(|v| v.starts_with("v0")))
            .collect::<Vec<_>>();
        assert!(diversity_histories(&all, &all, &BTreeSet::new()).is_err());
    }
}
