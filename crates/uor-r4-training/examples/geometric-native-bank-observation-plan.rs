//! Bounded construction-plan authorer + truthful retained-input bundler.
//! No native model predictions or outcome-adaptive source selection.
//! Input is a NEW sealed bundle of retained frozen-inputs512/128 (+opened repeat16),
//! not a partial cached directory masquerading as the original sealed full report.
//! Output:64 open-development histories +16 separately prospective fresh histories.
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
    input_bundle: PathBuf,
    input_manifest_sha256: String,
    exposed_roots: Vec<BoundRoot>,
    native_artifact: PathBuf,
    trusted_binding: PathBuf,
    trusted_binding_sha256: String,
    development_out: PathBuf,
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
                let authored = supported_wire(&donor, index / 2)?;
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
            return Err(invalid("fixed source formatter/context/answer+EOS alphabet eligibility failed;no outcome-based replacement").into());
        }
        targets.push(json!({"role":q.relation,"target_tokens_labels_only":answer.len(),"complete_public_replay_budget":replay+ids.len()+answer.len()}));
    }
    Ok(
        json!({"history":h.id,"source_views_construction_only":source_views,"targets_labels_only":targets,"native_predictions":"NOT_RUN","model_probability_support":"NOT_RUN;public alphabet membership only"}),
    )
}
fn run(a: &Args, t: Instant) -> Result<Value> {
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
            if let Some(hs) = v["histories"].as_array() {
                for h in hs {
                    let h: History = serde_json::from_value(h.clone())?;
                    exposed_history.extend(semantic_history(&h)?);
                }
            }
            v
        } else if e.root.join("inputs.json").is_file() {
            let v = read_json(&e.root.join("inputs.json"))?;
            exposed_numerical.extend(numerical_packets(&v)?);
            if e.root.join("raw-cue-provenance.json").is_file() {
                opened_strings(
                    &read_json(&e.root.join("raw-cue-provenance.json"))?,
                    &mut opened,
                );
            }
            v
        } else if e.root.join("raw-cue-provenance.json").is_file() {
            read_json(&e.root.join("raw-cue-provenance.json"))?
        } else {
            return Err(invalid("unrecognized opened source bundle;do not silently ignore").into());
        };
        opened_strings(&source, &mut opened);
    }
    let mut development = Vec::new();
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
    let mut fresh = Vec::new();
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
    let (development_origins, fresh_origins) = match a.assertion_query_policy {
        AssertionQueryPolicy::RetainedOriginal => (Vec::new(), Vec::new()),
        AssertionQueryPolicy::SupportedCurrentRole => (
            apply_supported_policy(&mut development)?,
            apply_supported_policy(&mut fresh)?,
        ),
    };
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
            if opened.contains(&w.text) {
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
    for (out, split, histories, fp) in [
        (&a.development_out, "development", &development, &dev_fp),
        (&a.fresh_out, "fresh", &fresh, &fresh_fp),
    ] {
        let plan = json!({"schema":"uor-r4.raw-natural-reader-construction-plan/1","split":split,"source_policy":a.assertion_query_policy.source_policy(),"histories":histories});
        let planbytes = serde_json::to_vec_pretty(&plan)?;
        let origins = if split == "development" {
            &development_origins
        } else {
            &fresh_origins
        };
        let origin_bytes = if a.assertion_query_policy == AssertionQueryPolicy::SupportedCurrentRole
        {
            Some(serde_json::to_vec_pretty(
                &json!({"schema":"uor-r4.supported-current-role-source-origin/1","policy":a.assertion_query_policy,"source_bundle_manifest_sha256":a.input_manifest_sha256,"histories":origins,"scope":"original donor values/chronology retained;assertion and query frames newly authored;payload novelty not claimed"}),
            )?)
        } else {
            None
        };
        let receipt = json!({"schema":"uor-r4.raw-natural-reader-construction-authoring/1","split":split,"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"source_bundle_manifest_sha256":a.input_manifest_sha256,"histories":histories.len(),"resulting_allbank_queries":histories.len()*2,"query_role_balance":"both natural job/home questions on every identical bank","chronology_pairs":histories.len()/2,"assertion_query_policy":a.assertion_query_policy,"source_origin":if a.assertion_query_policy==AssertionQueryPolicy::RetainedOriginal{"retained exact utterances512/128 +opened repetition16;fresh fixed known-frame compositions"}else{"retained donor literal bytes and chronology;prospectively authored explicit-current-role assertion/query frames;not retained-original utterance bytes"},"derived_source_origin_sha256":origin_bytes.as_ref().map(|bytes|sha256_bytes(bytes)),"fresh_payload_novelty_claimed":false,"source_label_scope":"offline construction only;not learned compiler result","semantic_fingerprints":fp,"numerical_packet_fingerprints":if split=="development"{&dev_numerical}else{&fresh_numerical},"exposed_roots":a.exposed_roots.iter().map(|e|json!({"root":e.root,"manifest_sha256":e.manifest_sha256})).collect::<Vec<_>>(),"plan_sha256":sha256_bytes(&planbytes),"public_formatter_eligibility":"all rows before prediction;support probabilities NOT_RUN","native_predictions":"NOT_RUN","learning_seed":"NOT_APPLICABLE","elapsed_seconds":t.elapsed().as_secs_f64()});
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
    if sha256_file(&a.trusted_binding)? != a.trusted_binding_sha256 {
        return Err(invalid("public parent binding changed during source authoring").into());
    }
    Ok(
        json!({"status":"COMPLETED","development_histories":64,"fresh_histories":16,"native_predictions":"NOT_RUN"}),
    )
}
fn author(a: Args) -> Result<()> {
    if a.mode.as_deref().is_some_and(|m| m != "author") {
        return Err(invalid("unknown plan authoring mode").into());
    }
    if a.schema != "uor-r4.native-bank-observation-plan-args/1"
        || a.maximum_seconds == 0
        || a.maximum_seconds > 300
        || a.maximum_report_bytes == 0
        || a.maximum_report_bytes > 16 * 1024 * 1024
    {
        return Err(invalid("authorer argument scope/caps invalid").into());
    }
    let dev = output_support::prospective_output(&a.development_out)?;
    let fresh = output_support::prospective_output(&a.fresh_out)?;
    if dev.starts_with(&fresh) || fresh.starts_with(&dev) {
        return Err(invalid("plan outputs overlap").into());
    }
    for p in [&a.input_bundle, &a.native_artifact, &a.trusted_binding] {
        let p = fs::canonicalize(p)?;
        if dev.starts_with(&p)
            || p.starts_with(&dev)
            || fresh.starts_with(&p)
            || p.starts_with(&fresh)
        {
            return Err(invalid("output/input intersection").into());
        }
    }
    for e in &a.exposed_roots {
        let p = fs::canonicalize(&e.root)?;
        if dev.starts_with(&p)
            || p.starts_with(&dev)
            || fresh.starts_with(&p)
            || p.starts_with(&fresh)
        {
            return Err(invalid("output/exposure intersection").into());
        }
    }
    report_output::claim(&a.development_out)?;
    report_output::claim(&a.fresh_out)?;
    let t = Instant::now();
    let result = run(&a, t);
    if let Err(e) = &result {
        for p in [&a.development_out, &a.fresh_out] {
            fs::write(
                p.join("failure.json"),
                serde_json::to_vec_pretty(
                    &json!({"error":e.to_string(),"native_predictions":"NOT_RUN","elapsed_seconds":t.elapsed().as_secs_f64()}),
                )?,
            )?;
        }
    }
    for p in [&a.development_out, &a.fresh_out] {
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
