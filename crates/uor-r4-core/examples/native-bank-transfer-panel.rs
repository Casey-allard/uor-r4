//! Frozen, model-free two-record prose transfer preparation. Labels never enter packets.
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs, io,
    path::{Path, PathBuf},
};
use uor_r4_core::{
    answer_oracle::{FrozenAnswers, RecordedValueIntent},
    report_output,
};
use uor_r4_integer::{
    geometric_source_actions::SourceActionBinding,
    geometric_source_emission_view::SourceEmissionCompiler,
};
use uor_r4_tokenizer::ByteBpeTokenizer;
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const BANKS: [[&str; 2]; 4] = [
    ["azure hillside", "violet orchard"],
    ["azure violet", "orchard hillside"],
    ["orchard azure", "hillside violet"],
    ["violet azure", "hillside orchard"],
];
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BoundFile {
    path: PathBuf,
    sha256: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    output: PathBuf,
    tokenizer: BoundFile,
    original_inputs: BoundFile,
    original_labels: BoundFile,
    #[serde(default)]
    additional_exposed_inputs: Vec<BoundFile>,
}
fn bad(s: impl Into<String>) -> Box<dyn std::error::Error> {
    io::Error::other(s.into()).into()
}
fn hash(b: &[u8]) -> String {
    hex::encode(Sha256::digest(b))
}
fn bound(f: &BoundFile) -> Result<Vec<u8>> {
    let b = fs::read(&f.path)?;
    if hash(&b) != f.sha256 {
        return Err(bad(format!("input hash differs: {}", f.path.display())));
    }
    Ok(b)
}
fn write(out: &Path, name: &str, value: &Value) -> Result<()> {
    let b = serde_json::to_vec_pretty(value)?;
    let used = fs::read_dir(out)?
        .try_fold(0u64, |n, e| -> Result<u64> { Ok(n + e?.metadata()?.len()) })?;
    if used + b.len() as u64 > 15 * 1024 * 1024 {
        return Err(bad(
            "16MiB preparation budget reserves1MiB for failure seal",
        ));
    }
    fs::write(out.join(name), b)?;
    Ok(())
}
fn ids(v: &Value) -> Result<Vec<u32>> {
    v.as_array()
        .ok_or_else(|| bad("token array missing"))?
        .iter()
        .map(|x| {
            x.as_u64()
                .and_then(|n| u32::try_from(n).ok())
                .ok_or_else(|| bad("invalid token ID"))
        })
        .collect()
}
fn encoded(tok: &ByteBpeTokenizer, binding: &SourceActionBinding, text: &str) -> Result<Vec<u32>> {
    let v = tok.encode(text);
    if v.is_empty()
        || v.iter().any(|&id| id >= 4096 || !binding.admits_token(id))
        || tok.decode_bytes(&v) != text.as_bytes()
    {
        return Err(bad("text fails exact legal4096 tokenizer roundtrip"));
    }
    Ok(v)
}
fn collect_sources(v: &Value, out: &mut Vec<Vec<u32>>) -> Result<()> {
    match v {
        Value::Object(m) => {
            if let Some(a) = m.get("original_source_ids") {
                out.push(ids(a)?);
            }
            for (k, child) in m {
                if k != "original_source_ids" {
                    collect_sources(child, out)?;
                }
            }
        }
        Value::Array(a) => {
            for child in a {
                collect_sources(child, out)?;
            }
        }
        _ => {}
    }
    Ok(())
}
fn contains(haystack: &[u32], needle: &[u32]) -> bool {
    !needle.is_empty() && haystack.windows(needle.len()).any(|w| w == needle)
}
// Instrument-only typed resolver: the actual question selects the relation.
fn typed_reference(query: &str, facts: &[(u64, String)]) -> Result<(usize, String, FrozenAnswers)> {
    let role = match query {
        "What is my current job?" => 0,
        "Where do I currently live?" => 1,
        _ => return Err(bad("unsupported reference question")),
    };
    if facts.len() != 2
        || (1..=2).any(|r| facts.iter().filter(|(relation, _)| *relation == r).count() != 1)
    {
        return Err(bad(
            "reference requires exactly both unambiguous job/home facts",
        ));
    }
    let value = facts
        .iter()
        .find(|(r, _)| *r == role as u64 + 1)
        .ok_or_else(|| bad("reference fact missing"))?
        .1
        .clone();
    let answer = if role == 0 {
        format!(" Your job is {value}.")
    } else {
        format!(" You live in {value}.")
    };
    let answers = FrozenAnswers {
        intent: RecordedValueIntent::Current,
        accepted: vec![answer],
    };
    answers.validate()?;
    Ok((role, value, answers))
}
fn reference_answer(
    packet: &Value,
    tok: &ByteBpeTokenizer,
    binding: &SourceActionBinding,
) -> Result<(usize, String, FrozenAnswers)> {
    let query_ids = ids(&packet["query_ids"])?;
    let decoded = tok.decode_bytes(&query_ids);
    let query = std::str::from_utf8(&decoded)?;
    if encoded(tok, binding, query)? != query_ids {
        return Err(bad("reference query token identity differs"));
    }
    let mut facts = Vec::new();
    for s in packet["segments"]
        .as_array()
        .ok_or_else(|| bad("reference segments missing"))?
    {
        if s["kind"] == "Source" {
            let relation = s["relation"]
                .as_u64()
                .ok_or_else(|| bad("reference typed relation missing"))?;
            let value = String::from_utf8(tok.decode_bytes(&ids(&s["original_source_ids"])?))?;
            facts.push((relation, value));
        }
    }
    typed_reference(query, &facts)
}
fn run(c: &Config) -> Result<Value> {
    let tb = bound(&c.tokenizer)?;
    let tok =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&tb).ok_or_else(|| bad("invalid tokenizer"))?;
    let binding = SourceActionBinding::new(&tb)?;
    if binding.vocab_size() != 4096
        || binding.eos_token_id() >= 4096
        || !binding.admits_token(binding.eos_token_id())
    {
        return Err(bad("native4096/EOS contract differs"));
    }
    let compiler = SourceEmissionCompiler::new(&tb)?;
    let original: Value = serde_json::from_slice(&bound(&c.original_inputs)?)?;
    let old_labels: Value = serde_json::from_slice(&bound(&c.original_labels)?)?;
    let original_cases = original["cases"]
        .as_array()
        .ok_or_else(|| bad("original cases missing"))?;
    let label_cases = old_labels["cases"]
        .as_array()
        .ok_or_else(|| bad("original labels missing"))?;
    if original["schema"] != "uor-r4.native-source-bank-probe-input/1"
        || old_labels["schema"] != "uor-r4.native-source-bank-labels/1"
        || old_labels["protocol"] != "uor-r4.literal-role-dialogue/2"
        || old_labels["membership_only"] != true
        || original_cases.len() != 512
        || label_cases.len() != 512
    {
        return Err(bad("exact original512 required"));
    }
    for (a, b) in original_cases.iter().zip(label_cases) {
        let answers: FrozenAnswers = serde_json::from_value(b["answers"].clone())?;
        answers.validate()?;
        if a["id"] != b["id"] {
            return Err(bad("original label IDs differ"));
        }
    }
    let templates = [&original_cases[0], &original_cases[1]];
    let queries = ["What is my current job?", "Where do I currently live?"];
    for (i, t) in templates.iter().enumerate() {
        if ids(&t["query_ids"])? != encoded(&tok, &binding, queries[i])?
            || !ids(&t["actual_prefix_ids"])?.is_empty()
        {
            return Err(bad("original query/empty prefix differs"));
        }
    }
    let base_segments = templates[0]["segments"]
        .as_array()
        .ok_or_else(|| bad("template segments missing"))?;
    if base_segments.len() != 4 || templates[0]["segments"] != templates[1]["segments"] {
        return Err(bad("original two-record bank differs"));
    }
    let mut contexts = Vec::new();
    let mut sources = Vec::new();
    for s in base_segments {
        match s["kind"].as_str() {
            Some("Context") => contexts.push(s.clone()),
            Some("Source") => sources.push(s.clone()),
            _ => return Err(bad("unexpected segment kind")),
        }
    }
    if contexts.len() != 2
        || sources.len() != 2
        || sources[0]["relation"] != 1
        || sources[1]["relation"] != 2
        || contexts.iter().any(|s| s["role"] != 1)
    {
        return Err(bad("job/home typed template differs"));
    }
    let expected_contexts = [
        "My current job is azure orchard.",
        "I currently live in violet hillside.",
    ];
    let expected_sources = ["azure orchard", "violet hillside"];
    for role in 0..2 {
        if ids(&contexts[role]["token_ids"])? != encoded(&tok, &binding, expected_contexts[role])?
            || ids(&sources[role]["original_source_ids"])?
                != encoded(&tok, &binding, expected_sources[role])?
            || contexts[role]["event"] != json!(role)
            || sources[role]["event"] != json!(role)
            || sources[role]["record"] != json!(role + 1)
            || sources[role]["commit"] != json!(role + 1)
            || sources[role]["entity"] != json!([1])
            || sources[role]["view"] != 0
            || sources[role]["scope"] != "compiler-probe"
        {
            return Err(bad("original Context/Source text or provenance differs"));
        }
    }
    let selected = [0usize, 1, 4, 5, 8, 9, 12, 13];
    write(
        &c.output,
        "baseline-inputs.json",
        &json!({"schema":original["schema"],"cases":selected.iter().map(|&i|original_cases[i].clone()).collect::<Vec<_>>()}),
    )?;
    write(
        &c.output,
        "baseline-labels.json",
        &json!({"schema":old_labels["schema"],"protocol":old_labels["protocol"],"membership_only":old_labels["membership_only"],"cases":selected.iter().map(|&i|label_cases[i].clone()).collect::<Vec<_>>()}),
    )?;
    let mut prior = vec![original.clone()];
    let mut exposure_files =
        vec![json!({"path":c.original_inputs.path,"sha256":c.original_inputs.sha256,"cases":512})];
    for f in &c.additional_exposed_inputs {
        let v: Value = serde_json::from_slice(&bound(f)?)?;
        exposure_files.push(json!({"path":f.path,"sha256":f.sha256}));
        prior.push(v);
    }
    let mut prior_sources = Vec::new();
    for p in &prior {
        collect_sources(p, &mut prior_sources)?;
    }
    if prior_sources.len() < 1024 {
        return Err(bad("original512 source exposure coverage incomplete"));
    }
    let mut prior_values = BTreeSet::new();
    let mut prior_views = Vec::new();
    let mut prior_vocab = BTreeSet::new();
    let mut prior_pairs = BTreeSet::new();
    for s in &prior_sources {
        let view = compiler.compile(s)?;
        prior_values.insert(view.original_bytes().to_vec());
        prior_vocab.extend(view.emitted_token_ids().iter().copied());
        prior_views.push(view.emitted_token_ids().to_vec());
    }
    // Each original serving case's full unordered value pair is independently indexed.
    for p in &prior {
        if let Some(cases) = p["cases"].as_array() {
            for row in cases {
                let mut ss = Vec::new();
                collect_sources(row, &mut ss)?;
                if ss.len() == 2 {
                    let mut pair = ss.iter().map(|s| tok.decode_bytes(s)).collect::<Vec<_>>();
                    pair.sort();
                    prior_pairs.insert(pair);
                }
            }
        }
    }
    let base_lengths = sources
        .iter()
        .map(|s| {
            compiler
                .compile(&ids(&s["original_source_ids"])?)
                .map(|v| v.emitted_token_ids().len())
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        })
        .collect::<Result<Vec<_>>>()?;
    let mut packets = Vec::new();
    let mut labels = Vec::new();
    let mut audit = Vec::new();
    let mut packet_hashes = BTreeSet::new();
    for (bank, values) in BANKS.iter().enumerate() {
        let mut pair = values
            .iter()
            .map(|s| s.as_bytes().to_vec())
            .collect::<Vec<_>>();
        pair.sort();
        if prior_pairs.contains(&pair) {
            return Err(bad("predeclared unordered complete-value pair was exposed; preserve failure, do not redraw"));
        }
        for value in values {
            if prior_values.contains(value.as_bytes()) {
                return Err(bad(
                    "predeclared complete value was exposed; preserve failure, do not redraw",
                ));
            }
        }
        for assignment in 0..2 {
            for order in 0..2 {
                let role_values = [values[assignment], values[1 - assignment]];
                let mut segments = Vec::new();
                let mut source_audit = Vec::new();
                let mut emitted_by_role = Vec::new();
                let mut base_len = 0usize;
                // Tokenize complete statements and complete Source strings; no fragment splicing.
                for physical in 0..2 {
                    let role = if order == 0 { physical } else { 1 - physical };
                    let text = if role == 0 {
                        format!("My current job is {}.", role_values[role])
                    } else {
                        format!("I currently live in {}.", role_values[role])
                    };
                    let context_ids = encoded(&tok, &binding, &text)?;
                    let source_ids = encoded(&tok, &binding, role_values[role])?;
                    let view = compiler.compile(&source_ids)?;
                    if view.original_bytes() != role_values[role].as_bytes()
                        || view
                            .emitted_token_ids()
                            .iter()
                            .any(|&id| id >= 4096 || !binding.admits_token(id))
                    {
                        return Err(bad("Source lexical view identity/legal mask differs"));
                    }
                    let mut context = contexts[role].clone();
                    context["event"] = json!(physical);
                    context["token_ids"] = json!(context_ids);
                    let mut source = sources[role].clone();
                    source["event"] = json!(physical);
                    source["commit"] = json!(physical + 1);
                    source["original_source_ids"] = json!(source_ids);
                    base_len += context_ids.len() + view.emitted_token_ids().len();
                    segments.push(context);
                    segments.push(source);
                    source_audit.push(json!({"physical":physical,"relation":role+1,"value":role_values[role],"original_ids":source_ids,"emitted_ids":view.emitted_token_ids(),"original_BPE_length":source_ids.len(),"emitted_BPE_length":view.emitted_token_ids().len(),"matches_exposed_role_length":view.emitted_token_ids().len()==base_lengths[role],"contiguous_original_token_subsequence_exposed":prior_sources.iter().any(|v|contains(v,&source_ids)),"contiguous_UTF8_bytes_exposed":prior_values.iter().any(|v|v.windows(role_values[role].len()).any(|w|w==role_values[role].as_bytes())),"contiguous_emitted_subsequence_exposed":prior_views.iter().any(|v|contains(v,view.emitted_token_ids())),"all_emitted_ids_familiar":view.emitted_token_ids().iter().all(|id|prior_vocab.contains(id))}));
                    emitted_by_role.push((role, view.emitted_token_ids().to_vec()));
                }
                for query_role in 0..2 {
                    let id = format!(
                        "transfer-bank{bank}-assignment{assignment}-order{order}-query{query_role}"
                    );
                    let query_ids = encoded(&tok, &binding, queries[query_role])?;
                    let packet = json!({"id":id,"segments":segments,"query_ids":query_ids,"actual_prefix_ids":[]});
                    let mut fingerprint = packet.clone();
                    fingerprint
                        .as_object_mut()
                        .ok_or_else(|| bad("packet not object"))?
                        .remove("id");
                    let digest = hash(&serde_json::to_vec(&fingerprint)?);
                    if !packet_hashes.insert(digest.clone()) {
                        return Err(bad("duplicate serving packet"));
                    }
                    let (reference_role, reference_value, answers) =
                        reference_answer(&packet, &tok, &binding)?;
                    if reference_role != query_role {
                        return Err(bad("declared query stratum differs from actual question"));
                    }
                    let answer = &answers.accepted[0];
                    let mut target = encoded(&tok, &binding, answer)?;
                    target.push(binding.eos_token_id());
                    let dynamic = encoded(&tok, &binding, &format!(" {reference_value}"))?;
                    let lexical = &emitted_by_role
                        .iter()
                        .find(|(r, _)| *r == query_role)
                        .ok_or_else(|| bad("reference lexical role missing"))?
                        .1;
                    if dynamic != *lexical || !contains(&target[..target.len() - 1], &dynamic) {
                        return Err(bad(
                            "reply-boundary dynamic value differs from full Source lexical view",
                        ));
                    }
                    if target.len() > 32 || base_len + query_ids.len() + 32 > 128 {
                        return Err(bad("declared128context/32generation capacity exceeded"));
                    }
                    if !answers.accepts(&tok.decode(&target[..target.len() - 1])) {
                        return Err(bad("offline reference exact membership roundtrip failed"));
                    }
                    let matches_lengths = source_audit
                        .iter()
                        .all(|s| s["matches_exposed_role_length"] == true);
                    labels.push(json!({"id":id,"answers":answers,"pair_id":format!("transfer-bank{bank}-order{order}-query{query_role}")}));
                    audit.push(json!({"id":id,"bank":bank,"assignment":assignment,"physical_order":order,"query_relation":query_role+1,"packet_sha256":digest,"sources":source_audit,"query_ids":query_ids,"target_ids_labels_only":target,"context_tokens_before_generation":base_len+query_ids.len(),"reserved_generation":32,"length_stratum":if matches_lengths{"matched-exposed-role-length"}else{"changed-BPE-length"},"offline_reference_exact_membership_and_EOS":true,"role_order_interventions_correlated":true}));
                    packets.push(packet);
                }
            }
        }
    }
    if packets.len() != 32 {
        return Err(bad("32-case prospective design differs"));
    }
    write(
        &c.output,
        "source-inputs.json",
        &json!({"schema":"uor-r4.native-source-bank-probe-input/1","cases":packets}),
    )?;
    write(
        &c.output,
        "labels.json",
        &json!({"schema":"uor-r4.native-source-bank-labels/1","protocol":"uor-r4.literal-role-dialogue/2","membership_only":true,"cases":labels}),
    )?;
    write(
        &c.output,
        "panel-audit.json",
        &json!({"cases":audit,"exposure_files":exposure_files,"prior_source_occurrences":prior_sources.len(),"complete_value_membership_excluded":true,"unordered_complete_value_pairs_excluded":true,"contiguous_subsequence_overlap_reported_not_rejected":true}),
    )?;
    Ok(
        json!({"schema":"uor-r4.native-bank-transfer-panel/1","status":"COMPLETED","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"model_calls":0,"optimizer_updates":0,"cases":32,"independent_value_banks":4,"offline_reference_passes":32,"prospective_values":BANKS,"baseline_input_indices":[0,1,4,5,8,9,12,13],"baseline_inputs_sha256":hash(&fs::read(c.output.join("baseline-inputs.json"))?),"baseline_labels_sha256":hash(&fs::read(c.output.join("baseline-labels.json"))?),"inputs_sha256":hash(&fs::read(c.output.join("source-inputs.json"))?),"labels_sha256":hash(&fs::read(c.output.join("labels.json"))?),"scope":"Prospectively fixed complete-value recombination, role/order/query transfer within two supported prose forms; no unseen-word/BPE claim, no independent32sample claim, no model quality measured. Labels/reference never enter serving packets; length differences separately audited; no heldout claim beyond configured exposure coverage."}),
    )
}
fn main() -> Result<()> {
    let mut argv = std::env::args().skip(1);
    let path = argv.next().ok_or_else(|| bad("one config path required"))?;
    if argv.next().is_some() {
        return Err(bad("one config path required"));
    }
    let raw = fs::read(path)?;
    let mut c: Config = serde_json::from_slice(&raw)?;
    c.output = output_support::prospective_output(&c.output)?;
    let mut inputs = vec![&c.tokenizer, &c.original_inputs, &c.original_labels];
    inputs.extend(c.additional_exposed_inputs.iter());
    for f in inputs {
        let p = fs::canonicalize(&f.path)?;
        if c.output.starts_with(&p) || p.starts_with(&c.output) {
            return Err(bad("output/input overlap"));
        }
        for a in p.ancestors() {
            if a.join(report_output::MANIFEST_FILE).is_file() && c.output.starts_with(a) {
                return Err(bad("output beneath sealed ancestor"));
            }
        }
    }
    report_output::claim(&c.output)?;
    write(&c.output, "config.json", &serde_json::from_slice(&raw)?)?;
    let result = run(&c);
    let report = match &result {
        Ok(v) => v.clone(),
        Err(e) => {
            json!({"schema":"uor-r4.native-bank-transfer-panel/1","status":"FAILED","error":e.to_string(),"scope":"Preparation/exposure/answerability failure, no model-quality verdict; do not silently redraw"})
        }
    };
    write(&c.output, "report.json", &report)?;
    report_output::seal(&c.output)?;
    result.map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exposure_subsequence_is_not_complete_value_membership() {
        assert!(contains(&[1, 2, 3, 4], &[2, 3]));
        assert!(!contains(&[1, 2, 3, 4], &[3, 2]));
        assert!(!contains(&[1], &[]));
    }
    #[test]
    fn actual_question_and_complete_bank_control_reference() -> Result<()> {
        let facts = vec![(2, "home value".into()), (1, "job value".into())];
        let (role, value, answer) = typed_reference("What is my current job?", &facts)?;
        assert_eq!(role, 0);
        assert_eq!(value, "job value");
        assert!(answer.accepts(" Your job is job value."));
        assert_eq!(
            typed_reference("Where do I currently live?", &facts)?.1,
            "home value"
        );
        assert!(typed_reference("Unsupported?", &facts).is_err());
        assert!(typed_reference("What is my current job?", &facts[..1]).is_err());
        assert!(typed_reference(
            "What is my current job?",
            &[(1, "a".into()), (1, "b".into())]
        )
        .is_err());
        Ok(())
    }
    #[test]
    fn four_prospective_banks_are_distinct_unordered_complete_values() {
        let pairs = BANKS
            .iter()
            .map(|v| {
                let mut p = v.to_vec();
                p.sort();
                p
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(pairs.len(), 4);
        assert_eq!(BANKS.iter().flatten().collect::<BTreeSet<_>>().len(), 8);
    }
}
