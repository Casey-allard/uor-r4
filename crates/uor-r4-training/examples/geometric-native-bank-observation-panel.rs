//! Prospective natural all-bank reader panel construction; no model predictions.
//! Install beside geometric-native-compiler.rs after review. Narrow prerequisite:
//! compiler::ReferenceRule and ReferenceRule::new_with_templates ->pub(super).
//! Uses existing exact-template compiler ONLY as an explicitly labelled offline
//! store-construction instrument, not learned compilation or runtime query gating.
//! CLI: geometric-native-bank-observation-panel CONFIG.json. No sampling RNG/model reads.
//! REQUIRED separate sealed construction plan supplies64development or16fresh histories,
//! each two natural query roles. Existing fresh cannot be relabelled; fresh must satisfy
//! complete semantic bank/query exclusions against EVERY declared opened panel root.
#[path = "geometric-native-compiler.rs"]
mod compiler;
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::{
    answer_oracle::{FrozenAnswers, RecordedValueIntent},
    report_output,
};
use uor_r4_integer::geometric_source_realizer::{NativeArtifactBinding, NativeSourceRealizer};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::{
    relation_compiler::Example,
    sha256_bytes, sha256_file,
    stack_grounded_session::{CompiledAction, TurnCompiler},
    stack_store::{HistoryView, StackStore, Update},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn invalid(s: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, s.into())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    schema: String,
    split: String,
    construction_plan: PathBuf,
    construction_manifest_sha256: String,
    curriculum_root: PathBuf,
    curriculum_manifest_sha256: String,
    exposed_roots: Vec<BoundRoot>,
    native_artifact: PathBuf,
    trusted_binding: PathBuf,
    trusted_binding_sha256: String,
    out: PathBuf,
    maximum_seconds: u64,
    maximum_report_bytes: usize,
    #[serde(default)]
    assertion_query_policy: AssertionQueryPolicy,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BoundRoot {
    root: PathBuf,
    manifest_sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    schema: String,
    split: String,
    source_policy: String,
    histories: Vec<History>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct History {
    id: String,
    stratum: String,
    turns: Vec<WireExample>,
    queries: Vec<WireExample>,
}
#[derive(Clone, Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct WireExample {
    text: String,
    relation: String,
    act: String,
    template: Option<String>,
}
#[derive(Clone, Copy, Default, Deserialize, serde::Serialize, PartialEq, Eq)]
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
fn exact_literal(w: &WireExample) -> Result<&str> {
    let template = w
        .template
        .as_deref()
        .ok_or_else(|| invalid("write template absent"))?;
    let (prefix, suffix) = template
        .split_once("{v}")
        .ok_or_else(|| invalid("write slot absent"))?;
    if suffix.contains("{v}") {
        return Err(invalid("multiple write slots").into());
    }
    let value = w
        .text
        .strip_prefix(prefix)
        .and_then(|x| x.strip_suffix(suffix))
        .filter(|x| !x.is_empty())
        .ok_or_else(|| invalid("write text/template differs"))?;
    Ok(value)
}
fn supported_wire_contract(w: &WireExample) -> Result<()> {
    let valid = match (w.relation.as_str(), w.act.as_str()) {
        ("job", "assert") => {
            w.template.as_deref() == Some("My current job is {v}.") && exact_literal(w).is_ok()
        }
        ("job", "update") => {
            w.template.as_deref() == Some("My current job has changed to {v}.")
                && exact_literal(w).is_ok()
        }
        ("home", "assert") => {
            w.template.as_deref() == Some("I currently live in {v}.") && exact_literal(w).is_ok()
        }
        ("home", "update") => {
            w.template.as_deref() == Some("I now live in {v}.") && exact_literal(w).is_ok()
        }
        ("job", "query") => {
            [
                "What is my current job?",
                "What job do I currently have?",
                "What is my job now?",
                "Which job do I have now?",
                "Remind me of my current job.",
                "Tell me my current job.",
            ]
            .contains(&w.text.as_str())
                && w.template.as_deref() == Some(w.text.as_str())
        }
        ("home", "query") => {
            [
                "Where do I currently live?",
                "Where do I live now?",
                "What is my current residence?",
                "Remind me where I currently live.",
                "Tell me where I live now.",
                "Where is my current home?",
            ]
            .contains(&w.text.as_str())
                && w.template.as_deref() == Some(w.text.as_str())
        }
        _ => false,
    };
    if !valid {
        return Err(invalid("unsupported current-role assertion/question").into());
    }
    Ok(())
}
fn validate_origin_wire(
    w: &WireExample,
    receipt: &Value,
    ordinal: usize,
    split: &str,
    opened: &[WireExample],
) -> Result<()> {
    let donor: WireExample = serde_json::from_value(receipt["donor_construction_wire"].clone())?;
    let authored: WireExample = serde_json::from_value(receipt["authored_wire"].clone())?;
    if receipt["ordinal"].as_u64()!=Some(ordinal as u64) || authored!=*w
        || receipt["donor_text_sha256"]!=sha256_bytes(donor.text.as_bytes())
        || receipt["origin"]!="prospectively authored explicit-current-role frame/question;not retained original assertion bytes"
        || donor.act!=w.act || donor.relation!=w.relation || donor.text.is_empty()
    {return Err(invalid("derived donor/authored wire binding differs").into());}
    supported_wire_contract(w)?;
    if w.act != "query" && exact_literal(&donor)? != exact_literal(w)? {
        return Err(invalid("derived assertion changed literal bytes").into());
    }
    let admitted = if split == "development" || w.act == "query" {
        opened.contains(&donor)
    } else {
        opened.iter().any(|e| {
            e.act == donor.act && e.relation == donor.relation && e.template == donor.template
        })
    };
    if !admitted {
        return Err(
            invalid("derived donor not in exact opened pool/declared fresh frame pool").into(),
        );
    }
    Ok(())
}
fn validate_supported_origins(
    plan: &Plan,
    origin: &Value,
    bundle_sha: &str,
    opened: &[WireExample],
) -> Result<()> {
    let histories = origin["histories"]
        .as_array()
        .ok_or_else(|| invalid("derived histories absent"))?;
    if origin["schema"] != "uor-r4.supported-current-role-source-origin/1"
        || origin["policy"] != "supported-current-role/1"
        || origin["source_bundle_manifest_sha256"] != bundle_sha
        || histories.len() != plan.histories.len()
    {
        return Err(invalid("derived source origin scope differs").into());
    }
    for (h, r) in plan.histories.iter().zip(histories) {
        if r["history"] != h.id {
            return Err(invalid("derived history chronology differs").into());
        }
        for (key, wires) in [("turns", &h.turns), ("queries", &h.queries)] {
            let receipts = r[key]
                .as_array()
                .ok_or_else(|| invalid("derived wire receipts absent"))?;
            if receipts.len() != wires.len() {
                return Err(invalid("derived wire count differs").into());
            }
            for (ordinal, (w, receipt)) in wires.iter().zip(receipts).enumerate() {
                validate_origin_wire(w, receipt, ordinal, &plan.split, opened)?;
            }
        }
    }
    Ok(())
}
fn example(w: &WireExample) -> Result<Example> {
    Ok(Example {
        text: w.text.clone(),
        relation: w.relation.clone(),
        act: match w.act.as_str() {
            "assert" => "assert",
            "update" => "update",
            "query" => "query",
            "none" => "none",
            _ => return Err(invalid("unknown source-plan act").into()),
        },
        template: w.template.clone(),
    })
}
fn read_json(p: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(p)?)?)
}
fn bytes_in(p: &Path) -> Result<usize> {
    let mut n = 0;
    for e in fs::read_dir(p)? {
        let e = e?;
        let t = e.file_type()?;
        if t.is_symlink() {
            return Err(invalid("symlink in output").into());
        }
        n += if t.is_dir() {
            bytes_in(&e.path())?
        } else {
            e.metadata()?.len() as usize
        };
    }
    Ok(n)
}
fn write_json(a: &Args, name: &str, v: &Value) -> Result<()> {
    let b = serde_json::to_vec_pretty(v)?;
    if bytes_in(&a.out)? + b.len() > a.maximum_report_bytes {
        return Err(invalid("prospective panel output cap").into());
    }
    fs::write(a.out.join(name), b)?;
    Ok(())
}
fn verify_root(root: &Path, sha: &str) -> Result<()> {
    report_output::verify(root)?;
    if sha256_file(&root.join("manifest.json"))? != sha {
        return Err(invalid("sealed input identity differs").into());
    }
    Ok(())
}
fn deadline(a: &Args, t: Instant) -> Result<()> {
    if t.elapsed().as_secs_f64() > a.maximum_seconds as f64 {
        return Err(invalid("preparation wall cap").into());
    }
    Ok(())
}
fn fingerprints(inputs: &Value) -> Result<BTreeSet<String>> {
    let mut result = BTreeSet::new();
    for c in inputs["cases"]
        .as_array()
        .ok_or_else(|| invalid("exposed runtime cases absent"))?
    {
        let segments = c["segments"]
            .as_array()
            .ok_or_else(|| invalid("exposed runtime segments absent"))?;
        let numerical = segments
            .iter()
            .map(|s| match s["kind"].as_str() {
                Some("Context") => {
                    Ok(json!({"kind":"Context","role":s["role"],"token_ids":s["token_ids"]}))
                }
                Some("Source") => {
                    Ok(json!({"kind":"Source","original_source_ids":s["original_source_ids"]}))
                }
                _ => Err(invalid("unknown exposed segment type")),
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        // Ignore opaque IDs/nonces: new record numbers cannot manufacture fresh data.
        result.insert(sha256_bytes(&serde_json::to_vec(
            &json!({"segments":numerical,"query_ids":c["query_ids"]}),
        )?));
    }
    Ok(result)
}
fn collect_opened_text(v: &Value, out: &mut BTreeSet<String>) {
    match v {
        Value::Object(o) => {
            for (k, x) in o {
                if k == "text" || k == "source" || k == "utterance_utf8" {
                    if let Some(s) = x.as_str() {
                        out.insert(s.to_owned());
                    }
                }
                collect_opened_text(x, out);
            }
        }
        Value::Array(xs) => {
            for x in xs {
                collect_opened_text(x, out)
            }
        }
        _ => {}
    }
}
fn run(a: &Args, t: Instant) -> Result<Value> {
    verify_root(&a.construction_plan, &a.construction_manifest_sha256)?;
    verify_root(&a.curriculum_root, &a.curriculum_manifest_sha256)?;
    let rawplan = fs::read(a.construction_plan.join("plan.json"))?;
    let plan: Plan = serde_json::from_slice(&rawplan)?;
    let histories = if a.split == "development" { 64 } else { 16 };
    if plan.schema != "uor-r4.raw-natural-reader-construction-plan/1"
        || plan.split != a.split
        || plan.source_policy != a.assertion_query_policy.source_policy()
        || plan.histories.len() != histories
    {
        return Err(invalid("construction plan scope/count differs").into());
    }
    if a.exposed_roots.is_empty() {
        return Err(invalid("explicit complete opened-exposure roots required").into());
    }
    let opened = read_json(&a.curriculum_root.join("frozen-inputs.json"))?;
    let mut opened_text = BTreeSet::new();
    collect_opened_text(&opened, &mut opened_text);
    let mut open_pool = opened["training"]
        .as_array()
        .ok_or_else(|| invalid("opened training array absent"))?
        .iter()
        .chain(
            opened["development"]
                .as_array()
                .ok_or_else(|| invalid("opened development array absent"))?,
        )
        .filter_map(|x| x["text"].as_str())
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let repeats = opened["factor_inputs"]["repeated_values"]
        .as_array()
        .ok_or_else(|| invalid("declared opened repetition factor pool absent"))?;
    if repeats.len() != 16 {
        return Err(invalid("opened repetition factor cardinality differs").into());
    }
    open_pool.extend(
        repeats
            .iter()
            .filter_map(|x| x["text"].as_str())
            .map(str::to_owned),
    );
    let derived_origin = if a.assertion_query_policy == AssertionQueryPolicy::SupportedCurrentRole {
        let report = read_json(&a.construction_plan.join("report.json"))?;
        let bytes = fs::read(a.construction_plan.join("derived-source-origin.json"))?;
        let digest = sha256_bytes(&bytes);
        if report["derived_source_origin_sha256"] != digest
            || report["plan_sha256"] != sha256_bytes(&rawplan)
            || report["assertion_query_policy"] != "supported-current-role/1"
            || report["source_bundle_manifest_sha256"] != a.curriculum_manifest_sha256
        {
            return Err(invalid("sealed authorer source/plan receipt binding differs").into());
        }
        let origin: Value = serde_json::from_slice(&bytes)?;
        let pool=opened["training"].as_array().ok_or_else(||invalid("training absent"))?.iter()
            .chain(opened["development"].as_array().ok_or_else(||invalid("development absent"))?)
            .chain(repeats).map(|v|serde_json::from_value::<WireExample>(json!({"text":v["text"],"relation":v["relation"],"act":v["act"],"template":v["template"]})))
            .collect::<std::result::Result<Vec<_>,_>>()?;
        validate_supported_origins(&plan, &origin, &a.curriculum_manifest_sha256, &pool)?;
        Some((origin, digest))
    } else {
        None
    };
    let mut excluded_fingerprints = BTreeSet::new();
    let mut exposure_receipts = Vec::new();
    for exposed in &a.exposed_roots {
        verify_root(&exposed.root, &exposed.manifest_sha256)?;
        // Explicit known schemas; never silently ignore an unparsed exposure root.
        let inputs = if exposed.root.join("inputs.json").is_file() {
            read_json(&exposed.root.join("inputs.json"))?
        } else if exposed.root.join("frozen-inputs.json").is_file() {
            read_json(&exposed.root.join("frozen-inputs.json"))?
        } else {
            return Err(invalid("opened root has no recognized source packet file").into());
        };
        if inputs["cases"].is_array() {
            excluded_fingerprints.extend(fingerprints(&inputs)?);
        } else if inputs["training"].is_array() {
            collect_opened_text(&inputs, &mut opened_text);
        } else {
            return Err(invalid("unknown opened packet schema;explicit adapter required").into());
        }
        if exposed.root.join("raw-cue-provenance.json").is_file() {
            collect_opened_text(
                &read_json(&exposed.root.join("raw-cue-provenance.json"))?,
                &mut opened_text,
            );
        }
        exposure_receipts
            .push(json!({"root":exposed.root,"manifest_sha256":exposed.manifest_sha256}));
    }
    if sha256_file(&a.trusted_binding)? != a.trusted_binding_sha256 {
        return Err(invalid("native parent trust SHA differs").into());
    }
    let parentseal = a
        .native_artifact
        .ancestors()
        .find(|p| p.join("manifest.json").is_file())
        .ok_or_else(|| invalid("native parent sealed ancestor absent"))?
        .to_path_buf();
    report_output::verify(&parentseal)?;
    let parent_manifest_sha = sha256_file(&parentseal.join("manifest.json"))?;
    let binding: NativeArtifactBinding = serde_json::from_slice(&fs::read(&a.trusted_binding)?)?;
    let native = NativeSourceRealizer::load_native(&a.native_artifact, &binding)?;
    let tokenizer = fs::read(a.native_artifact.join("tokenizer.json"))?;
    let tokenizer_sha = sha256_bytes(&tokenizer);
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer)
        .ok_or_else(|| invalid("bound ByteBPE unavailable"))?;
    let mut alltemplates = Vec::new();
    let mut source_ids = BTreeSet::new();
    for h in &plan.histories {
        if h.id.is_empty()
            || !source_ids.insert(h.id.clone())
            || h.turns.is_empty()
            || h.turns.len() > 8
            || h.queries.len() != 2
        {
            return Err(invalid("history identity/turn/pair bound differs").into());
        }
        if h.turns
            .iter()
            .any(|w| !matches!(w.act.as_str(), "assert" | "update"))
            || h.queries.iter().any(|q| q.act != "query")
        {
            return Err(invalid("history writes/query pair act scope differs").into());
        }
        let relations = h
            .queries
            .iter()
            .map(|q| q.relation.as_str())
            .collect::<BTreeSet<_>>();
        if relations != BTreeSet::from(["job", "home"]) {
            return Err(invalid("same-bank pair must have exactly job/home queries").into());
        }
        for w in h.turns.iter().chain(&h.queries) {
            deadline(a, t)?;
            if w.text.is_empty() || tok.encode(&w.text).len() > 128 {
                return Err(invalid("raw turn byteBPE budget").into());
            }
            if a.assertion_query_policy == AssertionQueryPolicy::RetainedOriginal
                && a.split == "development"
                && !open_pool.contains(&w.text)
            {
                return Err(invalid(
                    "development text not in declared open training/development pool",
                )
                .into());
            }
            if a.split == "fresh" && w.act != "query" && opened_text.contains(&w.text) {
                return Err(
                    invalid("fresh original statement already exposed;no relabelling").into(),
                );
            }
            alltemplates.push(example(w)?);
        }
    }
    let reference = compiler::ReferenceRule::new_with_templates(&tokenizer_sha, &alltemplates)?;
    let mut runtime = Vec::new();
    let mut labels = Vec::new();
    let mut context = Vec::new();
    let mut provenance = Vec::new();
    let mut control = Vec::new();
    let mut semantic = BTreeSet::new();
    for (ordinal, h) in plan.histories.iter().enumerate() {
        deadline(a, t)?;
        let namespace = 1000 + ordinal as u64;
        let mut store = StackStore::new(namespace, 16)?;
        // Actual store record -> full original utterance journal, just like reader.rs caller.
        let mut journal = BTreeMap::<u64, (u64, String, Vec<u32>)>::new();
        for (event, w) in h.turns.iter().enumerate() {
            let e = example(w)?;
            let action = reference.compile(&e.text)?;
            let (relation, span, update) = match action {
                CompiledAction::Assert { relation, span } if e.act == "assert" => {
                    (relation, span, Update::Assert)
                }
                CompiledAction::Correct { relation, span } if e.act == "update" => {
                    (relation, span, Update::Correct)
                }
                _ => return Err(invalid("source construction positive control act differs").into()),
            };
            let expected_relation = match e.relation.as_str() {
                "job" => 1,
                "home" => 2,
                _ => return Err(invalid("unsupported source relation").into()),
            };
            if relation != expected_relation {
                return Err(
                    invalid("reference write role differs from frozen source relation").into(),
                );
            }
            if Some((span.start, span.end)) != e.slot_span() {
                return Err(
                    invalid("reference span differs from frozen original byte span").into(),
                );
            }
            let literal = e
                .text
                .get(span.start..span.end)
                .ok_or_else(|| invalid("source byte range invalid"))?;
            let written = store.write_from(
                b"compiler-probe",
                &[1],
                relation,
                &tok.encode(literal),
                update,
                event as u64,
            )?;
            journal.insert(
                written.id,
                (event as u64, e.text.clone(), tok.encode(&e.text)),
            );
            let bytes = store.to_bytes()?;
            let digest = store.history_sha256()?;
            store = StackStore::from_bytes(&bytes, namespace)?;
            if store.history_sha256()? != digest {
                return Err(invalid("actual store roundtrip differs").into());
            }
        }
        let mut records = Vec::new();
        for address in store.addresses()? {
            let current = store.read(
                &address.scope,
                &address.entity,
                address.relation,
                HistoryView::Current,
            )?;
            if let Some(v) = current.value() {
                let (event, text, cue) = journal
                    .get(&v.record)
                    .ok_or_else(|| invalid("actual store record lacks original cue journal"))?;
                records.push((
                    *event,
                    address.relation,
                    v.record,
                    v.commit,
                    text.clone(),
                    cue.clone(),
                    v.tokens.clone(),
                ));
            }
        }
        // Public actual journal chronology; never target-role order or numeric record-ID order.
        records.sort_by_key(|r| r.0);
        if records.len() != 2 {
            return Err(
                invalid("bounded role-pair histories require both actual current records").into(),
            );
        }
        let mut queries = h.queries.iter().collect::<Vec<_>>();
        queries.sort_by_key(|q| if q.relation == "job" { 0 } else { 1 });
        for (query_index, q) in queries.iter().enumerate() {
            let id = format!("{}-{}-{}", a.split, h.id, q.relation);
            let mut segments = Vec::new();
            let mut views = Vec::new();
            let mut facts = Vec::new();
            let mut replay_len = 0usize;
            let mut alphabet = BTreeSet::from([
                native.binding().period_token_id(),
                native.binding().eos_token_id(),
            ]);
            for (event, relation, record, commit, text, cue, value) in &records {
                let ci = segments.len();
                let si = ci + 1;
                segments.push(json!({"kind":"Context","event":event,"role":1,"token_ids":cue}));
                segments.push(json!({"kind":"Source","event":event,"record":record,"commit":commit,"scope":"compiler-probe","entity":[1],"relation":relation,"view":0,"original_source_ids":value}));
                let view = native.compile_view(value)?;
                replay_len += cue.len() + view.emitted_token_ids().len();
                alphabet.extend(view.emitted_token_ids().iter().copied());
                views.push(json!({"segment_index":si,"source_view":view}));
                provenance.push(json!({"case_id":id,"context_segment_index":ci,"utterance_utf8":text,"utterance_sha256":sha256_bytes(text.as_bytes()),"tokenizer_sha256":tokenizer_sha,"context_token_ids":cue,"source_segment_index":si,"record":record,"commit":commit,"source_event":event,"stored_original_source_ids":value}));
                facts.push(json!({"role":if *relation==1{"job"}else{"where"},"literal":String::from_utf8(tok.decode_bytes(value))?,"event":event,"record":record,"commit":commit}));
            }
            let query = tok.encode(&q.text);
            let packet =
                json!({"id":id,"segments":segments,"query_ids":query,"actual_prefix_ids":[]});
            let fp = fingerprints(&json!({"cases":[packet.clone()]}))?
                .into_iter()
                .next()
                .ok_or_else(|| invalid("semantic fingerprint absent"))?;
            if !semantic.insert(fp.clone())
                || (a.split == "fresh" && excluded_fingerprints.contains(&fp))
            {
                return Err(invalid("duplicate/exposed actual bank+query packet").into());
            }
            // Expected relation/value is offline scorer construction ONLY, after bank frozen.
            let relation = match reference.compile(&q.text)? {
                CompiledAction::QueryCurrent { relation } => relation,
                _ => return Err(invalid("query reference positive control differs").into()),
            };
            if relation != if q.relation == "job" { 1 } else { 2 } {
                return Err(invalid("query reference role control differs").into());
            }
            let expected = store.read(b"compiler-probe", &[1], relation, HistoryView::Current)?;
            let value = expected
                .value()
                .ok_or_else(|| invalid("typed expected current value absent"))?;
            let text = String::from_utf8(tok.decode_bytes(&value.tokens))?;
            let answers = FrozenAnswers {
                intent: RecordedValueIntent::Current,
                accepted: vec![format!("{text}.")],
            };
            answers.validate()?;
            let mut target = tok.encode(&format!(" {}", answers.accepted[0]));
            target.push(native.binding().eos_token_id());

            // No native token probability/greedy-output support filter is permitted here.
            if target.len() > 32
                || query.len() + replay_len + target.len() > 128
                || target.iter().any(|x| !alphabet.contains(x))
            {
                return Err(
                    invalid("public context/output/alphabet budget;no native-mass filter").into(),
                );
            }
            labels.push(json!({"id":id,"answers":answers}));
            runtime.push(packet);
            context.push(json!({"id":id,"kind":"bank","stratum":h.stratum,"pair_id":format!("{}-{}",a.split,h.id),"query_role":if q.relation=="job"{"job"}else{"where"},"source_tuple":records.iter().map(|r|r.4.clone()).collect::<Vec<_>>(),"source_views":views,"typed_facts":facts,"target_ids_labels_only":target,"parent_id":null}));
            control.push(json!({"case_id":id,"typed_reference_query_role":relation,"actual_allbank_records":records.len(),"selector":"all actual current store addresses;no query-role filtering","query_index":query_index,"semantic_packet_sha256":fp}));
        }
    }
    let inputs = json!({"schema":"uor-r4.native-source-bank-probe-input/1","cases":runtime});
    let lab = json!({"schema":"uor-r4.native-source-bank-labels/1","protocol":"uor-r4.literal-role-dialogue/2","membership_only":true,"cases":labels});
    let ctx = json!({"schema":"uor-r4.geometric-bank-context-data/1","split":a.split,"layout_policy":"raw-natural-allbank-pairs/1","cases":context});
    if let Some((origin, _)) = &derived_origin {
        write_json(a, "derived-source-origin.json", origin)?;
    }
    write_json(a, "inputs.json", &inputs)?;
    write_json(a, "labels.json", &lab)?;
    write_json(a, "context-data.json", &ctx)?;
    write_json(
        a,
        "raw-cue-provenance.json",
        &json!({"schema":"uor-r4.raw-assertion-cue-provenance/1","rows":provenance}),
    )?;
    write_json(
        a,
        "construction-control.json",
        &json!({"policy":"typed exact-template compiler +actual StackStore roundtrip;not learned compilation","cases":control}),
    )?;
    for exposed in &a.exposed_roots {
        verify_root(&exposed.root, &exposed.manifest_sha256)?;
    }
    verify_root(&a.construction_plan, &a.construction_manifest_sha256)?;
    verify_root(&a.curriculum_root, &a.curriculum_manifest_sha256)?;
    report_output::verify(&parentseal)?;
    if sha256_file(&parentseal.join("manifest.json"))? != parent_manifest_sha
        || sha256_file(&a.trusted_binding)? != a.trusted_binding_sha256
    {
        return Err(invalid("native input changed during preparation").into());
    }
    Ok(
        json!({"schema":"uor-r4.native-bank-observation-panel/1","status":"COMPLETED","split":a.split,"cases":histories*2,"samebank_query_pairs":histories,"single_record_episode_count":0,"multi_record_episode_count":histories*2,"query_counts":{"job":histories,"where":histories},"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"tokenizer_sha256":tokenizer_sha,"native_parent_manifest_sha256":parent_manifest_sha,"construction_plan_manifest_sha256":a.construction_manifest_sha256,"construction_plan_sha256":sha256_bytes(&rawplan),"curriculum_manifest_sha256":a.curriculum_manifest_sha256,"exposed_roots":exposure_receipts,"semantic_packet_fingerprints":semantic,"assertion_query_policy":a.assertion_query_policy,"source_policy":plan.source_policy,"derived_source_origin_sha256":derived_origin.as_ref().map(|(_,sha)|sha),"source_origin":if derived_origin.is_some(){"prospectively authored explicit-current-role utterances;retained donor literal bytes/chronology;not original donor utterance bytes"}else{"retained original assertion/question bytes"},"fresh_payload_novelty_claimed":false,"cue_origin_policy":if derived_origin.is_some(){"prospectively-authored-current-role-bytes/bound-byteBPE/2"}else{"original-assertion-bytes/bound-byteBPE/1"},"raw_cue_provenance_sha256":sha256_file(&a.out.join("raw-cue-provenance.json"))?,"inputs_sha256":sha256_file(&a.out.join("inputs.json"))?,"labels_sha256":sha256_file(&a.out.join("labels.json"))?,"context_data_sha256":sha256_file(&a.out.join("context-data.json"))?,"trusted_binding_sha256":a.trusted_binding_sha256,"native_predictions":"NOT_RUN","learning_seed":"NOT_APPLICABLE;source-preparation-only","fresh_scope":"separate preauthored prospective histories;exact source/semantic-packet exclusion;no global pretraining exclusion claim","elapsed_seconds":t.elapsed().as_secs_f64()}),
    )
}
fn main() -> Result<()> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| invalid("usage: geometric-native-bank-observation-panel CONFIG.json"))?;
    let a: Args = serde_json::from_slice(&fs::read(&path)?)?;
    if a.schema != "uor-r4.native-bank-observation-panel-args/1"
        || !matches!(a.split.as_str(), "development" | "fresh")
        || a.maximum_seconds == 0
        || a.maximum_seconds > 300
        || a.maximum_report_bytes == 0
        || a.maximum_report_bytes > 64 * 1024 * 1024
    {
        return Err(invalid("panel arguments/bounds invalid").into());
    }
    let out = output_support::prospective_output(&a.out)?;
    for p in [
        &a.construction_plan,
        &a.curriculum_root,
        &a.native_artifact,
        &a.trusted_binding,
    ] {
        let p = fs::canonicalize(p)?;
        if out.starts_with(&p) || p.starts_with(&out) {
            return Err(invalid("output intersects immutable input").into());
        }
    }
    for e in &a.exposed_roots {
        let p = fs::canonicalize(&e.root)?;
        if out.starts_with(&p) || p.starts_with(&out) {
            return Err(invalid("output intersects exposure root").into());
        }
    }
    report_output::claim(&a.out)?;
    let t = Instant::now();
    let result = run(&a, t);
    let report = match &result {
        Ok(v) => v.clone(),
        Err(e) => {
            json!({"schema":"uor-r4.native-bank-observation-panel/1","status":"FAILED","error":e.to_string(),"native_predictions":"NOT_RUN","elapsed_seconds":t.elapsed().as_secs_f64()})
        }
    };
    write_json(&a, "report.json", &report)?;
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result.map(|_| ())
}

#[cfg(test)]
mod supported_current_role_panel_tests {
    use super::*;
    fn write(text: &str, template: &str) -> WireExample {
        WireExample {
            text: text.into(),
            template: Some(template.into()),
            relation: "job".into(),
            act: "assert".into(),
        }
    }
    fn receipt(donor: &WireExample, authored: &WireExample) -> Value {
        json!({"ordinal":0,"donor_construction_wire":donor,"authored_wire":authored,
            "donor_text_sha256":sha256_bytes(donor.text.as_bytes()),
            "origin":"prospectively authored explicit-current-role frame/question;not retained original assertion bytes"})
    }
    #[test]
    fn opt_in_preserves_legacy_default_and_rejects_unknown_policy() -> Result<()> {
        assert!(AssertionQueryPolicy::default() == AssertionQueryPolicy::RetainedOriginal);
        let supported: AssertionQueryPolicy = serde_json::from_str("\"supported-current-role/1\"")?;
        assert!(supported == AssertionQueryPolicy::SupportedCurrentRole);
        assert!(serde_json::from_str::<AssertionQueryPolicy>("\"silently-relabeled\"").is_err());
        Ok(())
    }
    #[test]
    fn donor_bytes_literal_and_actual_authored_wire_are_bound() -> Result<()> {
        let donor = write("I work as azure orchard.", "I work as {v}.");
        let authored = write("My current job is azure orchard.", "My current job is {v}.");
        let r = receipt(&donor, &authored);
        validate_origin_wire(&authored, &r, 0, "development", &[donor.clone()])?;
        let mut bad = r.clone();
        bad["donor_text_sha256"] = json!("wrong");
        assert!(validate_origin_wire(&authored, &bad, 0, "development", &[donor.clone()]).is_err());
        let changed = write("My current job is copper oasis.", "My current job is {v}.");
        assert!(validate_origin_wire(
            &changed,
            &receipt(&donor, &changed),
            0,
            "development",
            &[donor.clone()]
        )
        .is_err());
        assert!(validate_origin_wire(&changed, &r, 0, "development", &[donor.clone()]).is_err());
        assert!(validate_origin_wire(&authored, &r, 1, "development", &[donor]).is_err());
        Ok(())
    }
    #[test]
    fn prospective_fresh_value_requires_known_frame_without_claiming_exact_text() -> Result<()> {
        let opened = write("I work as old value.", "I work as {v}.");
        let donor = write("I work as new value.", "I work as {v}.");
        let authored = write("My current job is new value.", "My current job is {v}.");
        let r = receipt(&donor, &authored);
        validate_origin_wire(&authored, &r, 0, "fresh", &[opened.clone()])?;
        assert!(validate_origin_wire(&authored, &r, 0, "development", &[opened]).is_err());
        assert!(validate_origin_wire(&authored, &r, 0, "fresh", &[]).is_err());
        Ok(())
    }
    #[test]
    fn origin_address_and_role_action_conflations_are_rejected() -> Result<()> {
        for text in [
            "What city am I from?",
            "What is my address?",
            "What town am I from?",
        ] {
            let w = WireExample {
                text: text.into(),
                template: Some(text.into()),
                relation: "home".into(),
                act: "query".into(),
            };
            assert!(supported_wire_contract(&w).is_err());
        }
        let mut w = write("My current job is azure orchard.", "My current job is {v}.");
        supported_wire_contract(&w)?;
        w.act = "update".into();
        assert!(supported_wire_contract(&w).is_err());
        w.act = "assert".into();
        w.relation = "home".into();
        assert!(supported_wire_contract(&w).is_err());
        Ok(())
    }
    #[test]
    fn all_history_wires_and_bundle_identity_are_required() -> Result<()> {
        let donor = write("I work as azure orchard.", "I work as {v}.");
        let authored = write("My current job is azure orchard.", "My current job is {v}.");
        let plan = Plan {
            schema: "uor-r4.raw-natural-reader-construction-plan/1".into(),
            split: "development".into(),
            source_policy: AssertionQueryPolicy::SupportedCurrentRole
                .source_policy()
                .into(),
            histories: vec![History {
                id: "h1".into(),
                stratum: "test".into(),
                turns: vec![authored.clone()],
                queries: vec![],
            }],
        };
        let origin = json!({"schema":"uor-r4.supported-current-role-source-origin/1","policy":"supported-current-role/1","source_bundle_manifest_sha256":"bound","histories":[{"history":"h1","turns":[receipt(&donor,&authored)],"queries":[]}]});
        validate_supported_origins(&plan, &origin, "bound", &[donor.clone()])?;
        assert!(validate_supported_origins(&plan, &origin, "other", &[donor.clone()]).is_err());
        let mut missing = origin.clone();
        missing["histories"][0]["turns"] = json!([]);
        assert!(validate_supported_origins(&plan, &missing, "bound", &[donor.clone()]).is_err());
        let mut changed = origin;
        changed["histories"][0]["history"] = json!("other");
        assert!(validate_supported_origins(&plan, &changed, "bound", &[donor]).is_err());
        Ok(())
    }
}
