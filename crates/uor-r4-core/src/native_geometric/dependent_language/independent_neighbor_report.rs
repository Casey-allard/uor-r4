//! Independent finite transfer of one frozen artifact; no fitting or runtime changes.
use super::{
    completion, occurrence_role, query_participation as model,
    query_participation_report::{assess, fact, oracle, question, sha, words, write, Example},
    styled_role_report::{retained_200, retained_6688},
};
use crate::{
    answer_oracle,
    native_geometric::{
        addressed_attention::artifact::BoundGeometry, hamming_refinement::metric::Metric,
    },
    report_output,
};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const ARTIFACT_SHA: &str = "60d19679347ff349c0809353e3cca48925314a880b59c277bc9977ebd816ad73";
const OLD_SHA: &str = "8055a53c7801f1e6e11f808a6c15ddb090a5befe8d9d587b4ca65c74b7698793";
const SHAPES: [&str; 8] = [
    "novel",
    "novel-novel",
    "will-novel",
    "novel-will",
    "will-novel-novel",
    "novel-will-novel",
    "novel-novel-will",
    "novel-novel-novel",
];

fn acceptance(path: &Path) -> Result<(Vec<u8>, Value)> {
    let bytes = std::fs::read(path)?;
    let v: Value = serde_json::from_slice(&bytes)?;
    let required = json!({"schema":"uor-r4-independent-neighbor-v1","artifact_sha256":ARTIFACT_SHA,"rows":2304,"fits":0,"runtime_changes":0,"variants":3,"shape_classes":8,"directions":4,"query_prefixes":2,"source_rotations":4,"outcomes":{"valid":768,"missing":768,"conflict":768},"limits":{"records":4,"record_words":16,"record_bytes":128,"prompt_bytes":256,"clauses":2,"payload_words":3,"payload_bytes":50,"steps":96}});
    for (key, value) in required.as_object().ok_or("required acceptance")? {
        if v.get(key) != Some(value) {
            return Err(format!("frozen acceptance field {key}").into());
        }
    }
    let expected = "8bb96357a1b4cd20310f24147a073cc20f5bf147283027b57a62a4c2e5b7ad21";
    if sha(&bytes) != expected {
        return Err("acceptance digest differs from frozen receipt".into());
    }
    Ok((bytes, v))
}

fn inventory(
    value: &Value,
    tokens: &mut BTreeSet<Vec<u8>>,
    inputs: &mut BTreeSet<Vec<u8>>,
) -> Result<()> {
    match value {
        Value::Object(map) => {
            if let (Some(records), Some(prompt)) = (map.get("records"), map.get("prompt")) {
                if let (Ok(r), Ok(p)) = (
                    serde_json::from_value::<[Vec<u8>; 4]>(records.clone()),
                    serde_json::from_value::<Vec<u8>>(prompt.clone()),
                ) {
                    inputs.insert(serde_json::to_vec(&(&r, &p))?);
                }
            }
            for (key, child) in map {
                if matches!(
                    key.as_str(),
                    "records" | "prompt" | "answer" | "value" | "bytes" | "queries"
                ) {
                    collect_tokens(child, tokens);
                }
                inventory(child, tokens, inputs)?;
            }
        }
        Value::Array(a) => {
            for child in a {
                inventory(child, tokens, inputs)?;
            }
        }
        _ => {}
    }
    Ok(())
}
fn collect_tokens(value: &Value, out: &mut BTreeSet<Vec<u8>>) {
    if let Ok(bytes) = serde_json::from_value::<Vec<u8>>(value.clone()) {
        out.extend(words(&bytes).into_iter().map(Vec::from));
    } else if let Value::Array(a) = value {
        for child in a {
            collect_tokens(child, out);
        }
    }
}
fn data_paths(root: &Path, exclude: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if root == exclude {
        return Ok(());
    }
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_dir() {
            data_paths(&entry.path(), exclude, out)?;
        } else if kind.is_file() && entry.file_name() == "data.json" {
            out.push(entry.path());
        }
    }
    Ok(())
}
struct Draw {
    state: u64,
    excluded: BTreeSet<Vec<u8>>,
    names: BTreeSet<Vec<u8>>,
    attempts: usize,
}
impl Draw {
    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
    fn token(&mut self) -> Result<String> {
        for _ in 0..10000 {
            self.attempts += 1;
            let token: Vec<_> = (0..7).map(|_| b'a' + (self.next() % 26) as u8).collect();
            if !self.excluded.contains(&token) && self.names.insert(token.clone()) {
                return Ok(String::from_utf8(token)?);
            }
        }
        Err("lexical draw exhausted predetermined rejection bound".into())
    }
    fn endpoint(&mut self, shape: usize) -> Result<String> {
        let a = self.token()?;
        Ok(match shape {
            0 => a,
            1 => format!("{a} {}", self.token()?),
            2 => format!("will {a}"),
            3 => format!("{a} will"),
            4 => format!("will {a} {}", self.token()?),
            5 => format!("{a} will {}", self.token()?),
            6 => format!("{a} {} will", self.token()?),
            7 => format!("{a} {} {}", self.token()?, self.token()?),
            _ => return Err("unknown endpoint shape".into()),
        })
    }
}
fn styled(raw: Vec<u8>) -> Vec<u8> {
    [b"today ".as_slice(), &raw[..raw.len() - 1], b" tomorrow."].concat()
}
fn prefixed(raw: Vec<u8>, prefix: bool) -> Vec<u8> {
    if prefix {
        [b"please tell me ".as_slice(), &raw].concat()
    } else {
        raw
    }
}

fn panel(
    seed: u64,
    excluded: BTreeSet<Vec<u8>>,
    old_inputs: &BTreeSet<Vec<u8>>,
) -> Result<(Vec<Example>, Value)> {
    let mut draw = Draw {
        state: seed,
        excluded,
        names: BTreeSet::new(),
        attempts: 0,
    };
    let mut rows = vec![];
    let mut identities = BTreeSet::new();
    let mut inputs = BTreeSet::new();
    let mut pairs = vec![];
    for (shape, label) in SHAPES.iter().enumerate() {
        for first_subject in [false, true] {
            for second_subject in [false, true] {
                for prefix in [false, true] {
                    for rotation in 0..4 {
                        let known = draw.token()?;
                        let middle = draw.endpoint(shape)?;
                        let answer = draw.endpoint(shape)?;
                        let active_answer = draw.endpoint(shape)?;
                        let alternative = draw.endpoint(shape)?;
                        let missing_known = draw.endpoint(shape)?;
                        let inactive_known = draw.token()?;
                        let inactive_answer = draw.token()?;
                        let inactive_changed = draw.token()?;
                        let spare_known = draw.token()?;
                        let spare_answer = draw.token()?;
                        let first =
                            prefixed(question(&known, "will", "call", first_subject), prefix);
                        let second = prefixed(
                            question(
                                if second_subject { "them" } else { "they" },
                                "did",
                                "help",
                                second_subject,
                            ),
                            prefix,
                        );
                        let prompt = [first.as_slice(), b" ", second.as_slice()].concat();
                        let group = format!(
                            "{label}-{}{}-p{}-r{rotation}",
                            usize::from(first_subject),
                            usize::from(second_subject),
                            usize::from(prefix)
                        );
                        let mut variants = vec![];
                        for variant in ["baseline", "active", "inactive"] {
                            let value = if variant == "active" {
                                &active_answer
                            } else {
                                &answer
                            };
                            let mut records = [
                                styled(fact(&known, "will", "call", &middle, first_subject)),
                                styled(fact(&middle, "did", "help", value, second_subject)),
                                styled(fact(
                                    &inactive_known,
                                    "will",
                                    "visit",
                                    if variant == "inactive" {
                                        &inactive_changed
                                    } else {
                                        &inactive_answer
                                    },
                                    false,
                                )),
                                styled(fact(&spare_known, "did", "visit", &spare_answer, false)),
                            ];
                            records.rotate_left(rotation);
                            let first_source = (4 - rotation) % 4;
                            let second_source = (5 - rotation) % 4;
                            let spare_source = (7 - rotation) % 4;
                            let expected = oracle(&records, &prompt)?;
                            let intended_second =
                                prefixed(question(&middle, "did", "help", second_subject), prefix);
                            if expected.outcome != completion::Outcome::Answered
                                || expected.selected.len() != 2
                                || expected.selected[0].source != first_source
                                || expected.selected[1].source != second_source
                                || expected.selected[0].value != middle.as_bytes()
                                || expected.selected[1].value != value.as_bytes()
                                || expected.queries != vec![first.clone(), intended_second]
                            {
                                return Err("authored baseline path oracle mismatch".into());
                            }
                            let baseline = Example {
                                id: format!("{group}-{variant}-valid"),
                                family: format!("{label}/valid"),
                                variant: variant.into(),
                                records,
                                prompt: prompt.clone(),
                                expected,
                            };
                            variants.push(baseline.clone());
                            for mode in ["valid", "missing", "conflict"] {
                                let mut e = baseline.clone();
                                e.id = format!("{group}-{variant}-{mode}");
                                e.family = format!("{label}/{mode}");
                                if mode == "missing" {
                                    e.records[second_source] = styled(fact(
                                        &missing_known,
                                        "did",
                                        "help",
                                        value,
                                        second_subject,
                                    ));
                                }
                                if mode == "conflict" {
                                    e.records[spare_source] = styled(fact(
                                        &middle,
                                        "did",
                                        "help",
                                        &alternative,
                                        second_subject,
                                    ));
                                }
                                e.expected = oracle(&e.records, &e.prompt)?;
                                if mode != "valid" {
                                    let reason = if mode == "missing" {
                                        completion::Reason::NoCompatibleCandidate
                                    } else {
                                        completion::Reason::Ambiguous
                                    };
                                    let mut hits = if mode == "missing" {
                                        vec![]
                                    } else {
                                        vec![second_source, spare_source]
                                    };
                                    hits.sort();
                                    if e.expected.outcome
                                        != (completion::Outcome::Unresolved { clause: 1, reason })
                                        || e.expected.queries != baseline.expected.queries
                                        || e.expected.selected != baseline.expected.selected[..1]
                                        || e.expected.matching_sources
                                            != vec![vec![first_source], hits]
                                        || !e.expected.tokens.is_empty()
                                    {
                                        return Err(
                                            "authored continuation did not isolate second hop"
                                                .into(),
                                        );
                                    }
                                }
                                let key = serde_json::to_vec(&(&e.records, &e.prompt))?;
                                if !identities.insert(e.id.clone())
                                    || !inputs.insert(key.clone())
                                    || old_inputs.contains(&key)
                                    || e.records
                                        .iter()
                                        .any(|r| r.len() > 128 || words(r).len() > 16)
                                    || e.prompt.len() > 256
                                    || e.expected
                                        .selected
                                        .iter()
                                        .any(|s| s.value.len() > 50 || words(&s.value).len() > 3)
                                {
                                    return Err(
                                        "fresh input identity, overlap or window mismatch".into()
                                    );
                                }
                                rows.push(e);
                            }
                        }
                        let baseline = &variants[0];
                        let active = &variants[1];
                        let inactive = &variants[2];
                        let changed = |e: &Example| -> Vec<usize> {
                            e.records
                                .iter()
                                .zip(&baseline.records)
                                .enumerate()
                                .filter(|(_, (a, b))| a != b)
                                .map(|(i, _)| i)
                                .collect()
                        };
                        if changed(active) != vec![(5 - rotation) % 4]
                            || changed(inactive) != vec![(6 - rotation) % 4]
                            || active.expected.selected[0] != baseline.expected.selected[0]
                            || active.expected.queries != baseline.expected.queries
                            || active.expected.selected[1].value
                                == baseline.expected.selected[1].value
                            || inactive.expected != baseline.expected
                        {
                            return Err("active or inactive authored relation mismatch".into());
                        }
                        pairs.push(json!({"group":group,"shape":label,"first_query_subject":first_subject,"second_query_subject":second_subject,"query_prefix":prefix,"source_rotation":rotation,"baseline":baseline.id,"active":active.id,"inactive":inactive.id,"source_only_mutations":true,"oracle_active_changes_final_payload_only":true,"oracle_inactive_preserves_complete_expected":true}));
                    }
                }
            }
        }
    }
    if rows.len() != 2304
        || rows
            .iter()
            .filter(|e| e.expected.outcome == completion::Outcome::Answered)
            .count()
            != 768
    {
        return Err("fresh panel count".into());
    }
    let accepted: BTreeMap<_, _> = rows
        .iter()
        .map(|e| {
            let values: Vec<String> = if e.expected.outcome == completion::Outcome::Answered {
                vec![String::from_utf8_lossy(&e.expected.selected[1].value).into_owned()]
            } else {
                vec![]
            };
            (e.id.clone(), values)
        })
        .collect();
    Ok((
        rows,
        json!({"seed":seed,"generator":"SplitMix64; seven lowercase ASCII letters per lexical token; reject excluded or duplicate tokens only before model load","novel_tokens":draw.names,"lexical_draw_attempts":draw.attempts,"lexical_rejections":draw.attempts-draw.names.len(),"rejection_policy":"Excluded or duplicate token identity only; no candidate decoding or output-conditioned rejection.","matched_variants":pairs,"accepted_answer_strings":accepted,"acceptance_mode":"Frozen raw typed relation path authors bare final payload once; answer_oracle::accepts performs string membership; EOS and typed unresolved outcome checked separately."}),
    ))
}

fn sealed_attempt(output: &Path, run: impl FnOnce() -> Result<()>) -> Result<()> {
    report_output::claim(output)?;
    let result = run();
    if let Err(error) = &result {
        write(
            output,
            "error.json",
            &json!({"status":"REPORT_ERROR","error":error.to_string(),"not_a_model_gate_failure":true}),
        )?;
    }
    report_output::seal(output)?;
    report_output::verify(output)?;
    result
}
#[test]
fn independent_neighbor_environment_fixed_seed() -> Result<()> {
    let (rows, audit) = panel(0x52b9a76, BTreeSet::new(), &BTreeSet::new())?;
    assert_eq!(rows.len(), 2304);
    for label in SHAPES {
        for mode in ["valid", "missing", "conflict"] {
            assert_eq!(
                rows.iter()
                    .filter(|e| e.family == format!("{label}/{mode}"))
                    .count(),
                96
            );
        }
    }
    assert_eq!(
        audit["matched_variants"].as_array().ok_or("pairs")?.len(),
        256
    );
    let (same, same_audit) = panel(0x52b9a76, BTreeSet::new(), &BTreeSet::new())?;
    assert_eq!(serde_json::to_vec(&rows)?, serde_json::to_vec(&same)?);
    assert_eq!(audit, same_audit);
    Ok(())
}
#[test]
fn independent_neighbor_exclusion_and_failure_are_pre_model() -> Result<()> {
    let (rows, audit) = panel(41, BTreeSet::new(), &BTreeSet::new())?;
    let mut prior = BTreeSet::new();
    prior.insert(serde_json::to_vec(&(&rows[0].records, &rows[0].prompt))?);
    assert!(panel(41, BTreeSet::new(), &prior).is_err());
    let excluded: BTreeSet<Vec<u8>> = serde_json::from_value(audit["novel_tokens"].clone())?;
    let (_, changed) = panel(41, excluded.clone(), &BTreeSet::new())?;
    let novel: BTreeSet<Vec<u8>> = serde_json::from_value(changed["novel_tokens"].clone())?;
    assert!(novel.is_disjoint(&excluded));
    assert!(!answer_oracle::accepts(&["will abc def".into()], "abc def"));
    Ok(())
}
#[test]
#[ignore = "prepare and seal independent data after acceptance freeze; no model decode"]
fn independent_neighbor_prepare_report() -> Result<()> {
    let output = PathBuf::from(std::env::var("UOR_INDEPENDENT_NEIGHBOR_REPORT")?);
    let evidence = PathBuf::from(std::env::var("UOR_INDEPENDENT_NEIGHBOR_EVIDENCE")?);
    let path = PathBuf::from(std::env::var("UOR_INDEPENDENT_NEIGHBOR_ACCEPTANCE")?);
    let seed = std::env::var("UOR_INDEPENDENT_NEIGHBOR_SEED")?.parse::<u64>()?;
    if output.as_os_str().is_empty()
        || evidence.as_os_str().is_empty()
        || path.as_os_str().is_empty()
    {
        return Err("empty report paths".into());
    }
    sealed_attempt(&output, || {
        let (acceptance_bytes, accept) = acceptance(&path)?;
        let root = evidence.join("unknown-neighbor-1/attempt-4");
        report_output::verify(&root)?;
        let artifact = std::fs::read(root.join("candidate.json"))?;
        if sha(&artifact) != ARTIFACT_SHA {
            return Err("frozen candidate digest".into());
        }
        let mut tokens = BTreeSet::new();
        let mut inputs = BTreeSet::new();
        inventory(
            &serde_json::from_slice(&artifact)?,
            &mut tokens,
            &mut inputs,
        )?;
        let mut paths = vec![];
        data_paths(&evidence, &output, &mut paths)?;
        paths.sort();
        let mut lineage = vec![];
        for p in paths {
            report_output::verify(p.parent().ok_or("data parent")?)?;
            let b = std::fs::read(&p)?;
            inventory(&serde_json::from_slice(&b)?, &mut tokens, &mut inputs)?;
            lineage.push(json!({"path":p,"sha256":sha(&b)}));
        }
        let (rows, audit) = panel(seed, tokens.clone(), &inputs)?;
        let novel: BTreeSet<Vec<u8>> = serde_json::from_value(audit["novel_tokens"].clone())?;
        if !novel.is_disjoint(&tokens) {
            return Err("novel token overlap".into());
        }
        std::fs::write(output.join("acceptance.json"), &acceptance_bytes)?;
        write(
            &output,
            "data.json",
            &json!({"development":rows,"audit":audit,"acceptance_sha256":sha(&acceptance_bytes),"artifact_sha256":ARTIFACT_SHA,"seed":seed}),
        )?;
        write(
            &output,
            "environment.json",
            &json!({"pass":true,"stage":"PREPARED_BEFORE_MODEL_DECODE","rows":2304,"seed":seed,"acceptance":accept,"acceptance_sha256":sha(&acceptance_bytes),"novel_token_overlap":0,"raw_input_overlap":0,"excluded_tokens":tokens,"excluded_input_count":inputs.len(),"exclusion_sources":lineage,"nested_candidate_inventory_included":true,"fits":0,"model_decodes":0,"normal_model_promoted":false}),
        )?;
        println!("PREPARED_INDEPENDENT_NEIGHBOR_DATA {}", output.display());
        Ok(())
    })
}

fn failure_observations(
    a: &model::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    e: &Example,
    out: &completion::Generated,
) -> Result<Value> {
    let mut rows = vec![];
    let actual: BTreeSet<_> = out
        .decisions
        .iter()
        .map(|d| d.before.core.query.bytes.clone())
        .collect();
    for (kind, query, raw) in e
        .records
        .iter()
        .map(|r| ("source", false, r))
        .chain(
            e.expected
                .queries
                .iter()
                .map(|q| ("expected_query", true, q)),
        )
        .chain(actual.iter().map(|q| ("actual_query", true, q)))
    {
        let keys =
            occurrence_role::observations(&a.parent.parent, &a.parent.anchors, g, m, raw, false)?;
        let roles = if query {
            occurrence_role::query_context(&a.parent, g, m, raw, false)?
        } else {
            occurrence_role::source_context(&a.parent, g, m, raw, false)?
        };
        let participation = if query {
            json!({"keys":model::observations(a,g,m,raw,false)?,"required":a.required_mask(g,m,raw,false)?,"replacement":a.replacement_mask(g,m,raw,false)?})
        } else {
            Value::Null
        };
        rows.push(json!({"kind":kind,"raw":raw,"text":String::from_utf8_lossy(raw),"words":words(raw),"role_keys":keys,"context_roles":roles,"participation":participation}));
    }
    Ok(
        json!({"id":e.id,"family":e.family,"variant":e.variant,"expected":e.expected,"observations":rows}),
    )
}
fn retained(
    a: &model::Artifact,
    g: &BoundGeometry,
    m: &Metric,
    root: &Path,
    keys: &[&str],
    size: usize,
) -> Result<Value> {
    report_output::verify(root)?;
    let data_bytes = std::fs::read(root.join("data.json"))?;
    let response_bytes = std::fs::read(root.join("responses.json"))?;
    let data: Value = serde_json::from_slice(&data_bytes)?;
    let responses: Vec<Value> = serde_json::from_slice(&response_bytes)?;
    let mut rows = vec![];
    for key in keys {
        rows.extend(serde_json::from_value::<Vec<Example>>(data[*key].clone())?);
    }
    if rows.len() != size {
        return Err("retained row count".into());
    }
    let mut equal = 0;
    let mut items = vec![];
    for e in rows {
        let matching: Vec<_> = responses.iter().filter(|r| r["id"] == e.id).collect();
        if matching.len() != 1 {
            return Err("retained row identity".into());
        }
        let sealed: completion::Generated = serde_json::from_value(matching[0]["Full"].clone())?;
        let actual = model::generate(a, g, m, &e.records, &e.prompt, model::Control::Full)?;
        let same = actual == sealed;
        equal += usize::from(same);
        items.push(json!({"id":e.id,"equal":same,"actual_if_changed":if same{Value::Null}else{json!(actual)}}));
    }
    Ok(
        json!({"rows":size,"equal":equal,"pass":equal==size,"source_root":root,"data_sha256":sha(&data_bytes),"responses_sha256":sha(&response_bytes),"items":items}),
    )
}

#[derive(serde::Deserialize)]
struct SavedControl {
    assessment: Value,
    frozen_answer_membership_or_typed_unresolved: bool,
    correct: bool,
    outcome: completion::Outcome,
    tokens: Vec<u16>,
    equal_full: bool,
}
#[derive(serde::Deserialize)]
struct SavedResponse {
    id: String,
    family: String,
    variant: String,
    #[serde(rename = "Full")]
    full: completion::Generated,
    controls: BTreeMap<String, SavedControl>,
}
struct RecoveredEvaluation {
    counts: BTreeMap<String, usize>,
    correct: usize,
    exact: usize,
    reloaded: usize,
    disabled: usize,
    generated: BTreeMap<String, completion::Generated>,
}
fn file_sha(path: &Path) -> Result<String> {
    use sha2::Digest;
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut hasher = sha2::Sha256::new();
    let mut buffer = [0u8; 65536];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hex::encode(hasher.finalize()))
}
fn frozen_membership(
    e: &Example,
    accepted: &[String],
    outcome: &completion::Outcome,
    tokens: &[u16],
) -> bool {
    if e.expected.outcome != completion::Outcome::Answered {
        return tokens.is_empty() && *outcome == e.expected.outcome;
    }
    tokens
        .strip_suffix(&[256])
        .and_then(|ts| {
            ts.iter()
                .map(|t| u8::try_from(*t).ok())
                .collect::<Option<Vec<_>>>()
        })
        .and_then(|b| String::from_utf8(b).ok())
        .is_some_and(|s| answer_oracle::accepts(accepted, &s))
}
fn resume_evaluation(
    output: &Path,
    source: &Path,
    data: &Value,
    data_sha: &str,
    acceptance_sha: &str,
    rows: &[Example],
    accepted: &BTreeMap<String, Vec<String>>,
    a: &model::Artifact,
    reload: &model::Artifact,
    g: &BoundGeometry,
    m: &Metric,
) -> Result<RecoveredEvaluation> {
    report_output::verify(source)?;
    let receipt_path = PathBuf::from(std::env::var("UOR_INDEPENDENT_NEIGHBOR_RESUME_RECEIPT")?);
    let receipt_bytes = std::fs::read(&receipt_path)?;
    let receipt: Value = serde_json::from_slice(&receipt_bytes)?;
    if receipt["schema"] != "uor-r4-independent-neighbor-resume/1"
        || receipt["status"] != "INTERRUPTED_RESOURCE_LIMIT"
        || receipt["interrupted_root"] != serde_json::to_value(source)?
        || receipt["artifact_sha256"] != ARTIFACT_SHA
        || receipt["data_sha256"] != data_sha
        || receipt["acceptance_sha256"] != acceptance_sha
        || receipt["seed"] != data["seed"]
    {
        return Err("resume receipt provenance".into());
    }
    for (file, field) in [
        ("manifest.json", "manifest_sha256"),
        ("responses.json", "responses_sha256"),
        ("lineage.json", "lineage_sha256"),
    ] {
        if receipt[field] != file_sha(&source.join(file))? {
            return Err(format!("resume digest {file}").into());
        }
    }
    for (path_key, digest_key) in [
        ("source_freeze_path", "source_freeze_sha256"),
        ("binary_path", "binary_sha256"),
        ("execution_receipt_path", "execution_receipt_sha256"),
    ] {
        let path = Path::new(receipt[path_key].as_str().ok_or("resume receipt path")?);
        if receipt[digest_key] != file_sha(path)? {
            return Err(format!("resume external receipt {path_key}").into());
        }
    }
    let original_source: Value = serde_json::from_slice(&std::fs::read(
        receipt["source_freeze_path"]
            .as_str()
            .ok_or("source freeze")?,
    )?)?;
    if original_source["binary"] != receipt["binary_path"]
        || original_source["binary_sha256"] != receipt["binary_sha256"]
    {
        return Err("original source and binary binding".into());
    }
    let lineage: Value = serde_json::from_slice(&std::fs::read(source.join("lineage.json"))?)?;
    if lineage["data_sha256"] != data_sha
        || lineage["acceptance_sha256"] != acceptance_sha
        || lineage["artifact_sha256"] != ARTIFACT_SHA
        || lineage["seed"] != data["seed"]
        || lineage["fits"] != 0
        || lineage["runtime_changes"] != 0
    {
        return Err("interrupted lineage mismatch".into());
    }
    let execution: Value = serde_json::from_slice(&std::fs::read(
        receipt["execution_receipt_path"]
            .as_str()
            .ok_or("execution receipt path")?,
    )?)?;
    let args = execution["args"].as_array().ok_or("execution arguments")?;
    let prepared_root = lineage["prepared_root"]
        .as_str()
        .ok_or("prepared lineage root")?;
    for required in [
        receipt["binary_path"].as_str().ok_or("binary path")?.to_owned(),
        format!("UOR_INDEPENDENT_NEIGHBOR_REPORT={}", source.display()),
        format!("UOR_INDEPENDENT_NEIGHBOR_DATA={prepared_root}"),
        format!("UOR_INDEPENDENT_NEIGHBOR_SEED={}", data["seed"]),
        "native_geometric::dependent_language::independent_neighbor_report::independent_neighbor_evaluate_report".to_owned(),
        "--ignored".to_owned(),
        "--exact".to_owned(),
    ] {
        if !args.iter().any(|arg| arg.as_str() == Some(required.as_str())) {
            return Err("original execution does not bind expected binary/data/seed/report".into());
        }
    }
    if execution["cmd"] != "env"
        || execution["charged"] != true
        || execution["stopped"].as_str().is_none()
    {
        return Err("original interrupted execution status".into());
    }
    let saved: Vec<SavedResponse> = serde_json::from_reader(std::io::BufReader::new(
        std::fs::File::open(source.join("responses.json"))?,
    ))?;
    if saved.len() != rows.len() {
        return Err("saved fresh response count".into());
    }
    let mut result = RecoveredEvaluation {
        counts: BTreeMap::new(),
        correct: 0,
        exact: 0,
        reloaded: 0,
        disabled: 0,
        generated: BTreeMap::new(),
    };
    let mut audit = vec![];
    for (e, old) in rows.iter().zip(saved) {
        if old.id != e.id
            || old.family != e.family
            || old.variant != e.variant
            || old.controls.len() != 4
        {
            return Err("saved response identity/order".into());
        }
        let reassessed = assess(a, g, m, e, model::Control::Full, &old.full)?;
        let recovered = model::generate(reload, g, m, &e.records, &e.prompt, model::Control::Full)?;
        let same = recovered == old.full;
        result.reloaded += usize::from(same);
        for name in ["Full", "ExactIdentity", "ReadDisabled", "UpdateDisabled"] {
            let control = old.controls.get(name).ok_or("saved control absent")?;
            let membership =
                frozen_membership(e, &accepted[&e.id], &control.outcome, &control.tokens);
            let yes = control.assessment["correct"] == true && membership;
            let assessment_conjunction = [
                "initial_selection_correct",
                "lookahead_query_geometry_correct",
                "exact_compatible_sources_correct",
                "outcome_correct",
                "committed_path_queries_source_span_bounds_correct",
                "continuity",
            ]
            .iter()
            .all(|k| control.assessment[*k] == true);
            if control.frozen_answer_membership_or_typed_unresolved != membership
                || control.correct != yes
                || control.assessment["correct"] != assessment_conjunction
                || (control.assessment["outcome_correct"] == true
                    && (control.outcome != e.expected.outcome
                        || control.tokens != e.expected.tokens))
                || (control.equal_full
                    && (control.outcome != old.full.outcome
                        || control.tokens != old.full.trace.tokens))
            {
                return Err(format!("saved {name} assessment consistency for {}", e.id).into());
            }
            if name == "Full"
                && (control.assessment != reassessed
                    || control.outcome != old.full.outcome
                    || control.tokens != old.full.trace.tokens
                    || !control.equal_full)
            {
                return Err("saved Full trace reassessment differs".into());
            }
            *result
                .counts
                .entry(format!("{}/{name}", e.family))
                .or_default() += usize::from(yes);
            if name == "Full" {
                result.correct += usize::from(yes);
            }
            if name == "ExactIdentity" {
                result.exact += usize::from(control.equal_full);
            }
            if matches!(name, "ReadDisabled" | "UpdateDisabled")
                && e.expected.outcome == completion::Outcome::Answered
            {
                result.disabled += usize::from(control.assessment["outcome_correct"] == true);
            }
        }
        audit.push(json!({"id":e.id,"saved_full_reassessment_equal":true,"reload_full_equal":same,"saved_control_consistency":true,"reload_if_changed":if same{Value::Null}else{json!(recovered)}}));
        if result.generated.insert(e.id.clone(), old.full).is_some() {
            return Err("saved duplicate response id".into());
        }
    }
    std::fs::write(output.join("resume-receipt.json"), receipt_bytes)?;
    write(
        output,
        "resume-equality.json",
        &json!({"rows":rows.len(),"reload_full_equal":result.reloaded,"saved_full_reassessed":rows.len(),"saved_control_consistency_checked":rows.len()*4,"fresh_four_control_panel_regenerated":false,"reload_full_generations":rows.len(),"source_responses":source.join("responses.json"),"source_responses_sha256":receipt["responses_sha256"],"source_execution_receipt":receipt["execution_receipt_path"],"receipt_sha256":file_sha(&receipt_path)?,"items":audit}),
    )?;
    write(
        output,
        "responses-reference.json",
        &json!({"path":source.join("responses.json"),"sha256":receipt["responses_sha256"],"rows":rows.len(),"reason":"Original complete saved responses preserved without a duplicate large file; actual Full reload equality and saved trace reassessment recovered here."}),
    )?;
    write(
        output,
        "failure-observations-reference.json",
        &json!({"path":source.join("failure-observations.json"),"sha256":file_sha(&source.join("failure-observations.json"))?,"interpretation":"Same fixed candidate/data and original exposed failures; no fitting, output-conditioned redraw or acceptance change."}),
    )?;
    report_output::verify(source)?;
    Ok(result)
}

#[test]
#[ignore = "evaluate sealed independent panel once with unchanged artifact and retained controls"]
fn independent_neighbor_evaluate_report() -> Result<()> {
    let output = PathBuf::from(std::env::var("UOR_INDEPENDENT_NEIGHBOR_REPORT")?);
    let evidence = PathBuf::from(std::env::var("UOR_INDEPENDENT_NEIGHBOR_EVIDENCE")?);
    let prepared = PathBuf::from(std::env::var("UOR_INDEPENDENT_NEIGHBOR_DATA")?);
    let path = PathBuf::from(std::env::var("UOR_INDEPENDENT_NEIGHBOR_ACCEPTANCE")?);
    if output.as_os_str().is_empty()
        || evidence.as_os_str().is_empty()
        || prepared.as_os_str().is_empty()
        || path.as_os_str().is_empty()
    {
        return Err("empty report paths".into());
    }
    sealed_attempt(&output, || {
        let (acceptance_bytes, _) = acceptance(&path)?;
        report_output::verify(&prepared)?;
        if std::fs::read(prepared.join("acceptance.json"))? != acceptance_bytes {
            return Err("sealed acceptance changed".into());
        }
        let data_bytes = std::fs::read(prepared.join("data.json"))?;
        let data: Value = serde_json::from_slice(&data_bytes)?;
        let expected_seed = std::env::var("UOR_INDEPENDENT_NEIGHBOR_SEED")?.parse::<u64>()?;
        if data["seed"] != expected_seed {
            return Err("evaluation seed differs from sealed preparation".into());
        }
        let environment: Value =
            serde_json::from_slice(&std::fs::read(prepared.join("environment.json"))?)?;
        if environment["pass"] != true
            || environment["model_decodes"] != 0
            || data["acceptance_sha256"] != sha(&acceptance_bytes)
            || data["artifact_sha256"] != ARTIFACT_SHA
        {
            return Err("sealed preparation provenance".into());
        }
        let rows: Vec<Example> = serde_json::from_value(data["development"].clone())?;
        let accepted: BTreeMap<String, Vec<String>> =
            serde_json::from_value(data["audit"]["accepted_answer_strings"].clone())?;
        if rows.len() != 2304 || accepted.len() != 2304 {
            return Err("sealed independent row count".into());
        }
        for e in &rows {
            if oracle(&e.records, &e.prompt)? != e.expected {
                return Err("sealed raw oracle changed".into());
            }
        }
        write(
            &output,
            "lineage.json",
            &json!({"prepared_root":prepared,"data_sha256":sha(&data_bytes),"acceptance_sha256":sha(&acceptance_bytes),"artifact_sha256":ARTIFACT_SHA,"seed":data["seed"],"fits":0,"runtime_changes":0,"independent_finite_transfer_only":true}),
        )?;
        let root = evidence.join("unknown-neighbor-1/attempt-4");
        report_output::verify(&root)?;
        let bytes = std::fs::read(root.join("candidate.json"))?;
        if sha(&bytes) != ARTIFACT_SHA {
            return Err("frozen candidate identity".into());
        }
        let g = BoundGeometry::canonical()?;
        let m = Metric::new(&g)?;
        let a = model::Artifact::decode(&bytes, &g)?;
        let reload = model::Artifact::decode(&a.encode()?, &g)?;
        let mut counts = BTreeMap::<String, usize>::new();
        let mut responses = vec![];
        let mut failures = vec![];
        let mut correct = 0;
        let mut exact = 0;
        let mut reloaded = 0;
        let mut disabled = 0;
        let mut generated = BTreeMap::new();
        let resume = std::env::var_os("UOR_INDEPENDENT_NEIGHBOR_RESUME").map(PathBuf::from);
        if let Some(source) = &resume {
            let recovered = resume_evaluation(
                &output,
                source,
                &data,
                &sha(&data_bytes),
                &sha(&acceptance_bytes),
                &rows,
                &accepted,
                &a,
                &reload,
                &g,
                &m,
            )?;
            counts = recovered.counts;
            correct = recovered.correct;
            exact = recovered.exact;
            reloaded = recovered.reloaded;
            disabled = recovered.disabled;
            generated = recovered.generated;
        } else {
            for e in &rows {
                let full =
                    model::generate(&a, &g, &m, &e.records, &e.prompt, model::Control::Full)?;
                reloaded += usize::from(
                    full == model::generate(
                        &reload,
                        &g,
                        &m,
                        &e.records,
                        &e.prompt,
                        model::Control::Full,
                    )?,
                );
                let mut controls = serde_json::Map::new();
                for (name, c) in [
                    ("Full", model::Control::Full),
                    ("ExactIdentity", model::Control::ExactIdentity),
                    ("ReadDisabled", model::Control::ReadDisabled),
                    ("UpdateDisabled", model::Control::UpdateDisabled),
                ] {
                    let out = if c == model::Control::Full {
                        full.clone()
                    } else {
                        model::generate(&a, &g, &m, &e.records, &e.prompt, c)?
                    };
                    let assessment = assess(&a, &g, &m, e, c, &out)?;
                    let answer = out
                        .trace
                        .tokens
                        .strip_suffix(&[256])
                        .and_then(|tokens| {
                            tokens
                                .iter()
                                .map(|t| u8::try_from(*t).ok())
                                .collect::<Option<Vec<_>>>()
                        })
                        .and_then(|b| String::from_utf8(b).ok());
                    let membership = if e.expected.outcome == completion::Outcome::Answered {
                        answer
                            .as_ref()
                            .is_some_and(|s| answer_oracle::accepts(&accepted[&e.id], s))
                    } else {
                        out.trace.tokens.is_empty() && out.outcome == e.expected.outcome
                    };
                    let yes = assessment["correct"] == true && membership;
                    *counts.entry(format!("{}/{name}", e.family)).or_default() += usize::from(yes);
                    if c == model::Control::Full {
                        correct += usize::from(yes);
                        if !yes {
                            failures.push(failure_observations(&a, &g, &m, e, &out)?);
                        }
                    }
                    if c == model::Control::ExactIdentity {
                        exact += usize::from(out == full);
                    }
                    if matches!(
                        c,
                        model::Control::ReadDisabled | model::Control::UpdateDisabled
                    ) && e.expected.outcome == completion::Outcome::Answered
                    {
                        disabled += usize::from(assessment["outcome_correct"] == true);
                    }
                    controls.insert(name.into(),json!({"assessment":assessment,"frozen_answer_membership_or_typed_unresolved":membership,"correct":yes,"outcome":out.outcome,"tokens":out.trace.tokens,"equal_full":out==full}));
                }
                generated.insert(e.id.clone(), full.clone());
                responses.push(json!({"id":e.id,"family":e.family,"variant":e.variant,"Full":full,"controls":controls}));
            }
            write(&output, "responses.json", &json!(responses))?;
            write(
                &output,
                "failure-observations.json",
                &json!({"rows":failures.len(),"role_anchors":a.parent.anchors,"context_words":a.parent.parent.parent.context_words,"query_role_table":a.parent.query_table,"source_role_table":a.parent.table,"items":failures}),
            )?;
        }
        let mut matched = vec![];
        let mut matched_pass = 0;
        for p in data["audit"]["matched_variants"]
            .as_array()
            .ok_or("matched groups")?
        {
            let get = |k: &str| -> Result<&completion::Generated> {
                generated
                    .get(p[k].as_str().ok_or("variant id")?)
                    .ok_or_else(|| "missing matched generation".into())
            };
            let b = get("baseline")?;
            let active = get("active")?;
            let inactive = get("inactive")?;
            let pass = b.outcome == completion::Outcome::Answered
                && active.outcome == completion::Outcome::Answered
                && inactive.outcome == completion::Outcome::Answered
                && b.trace.tokens != active.trace.tokens
                && b.trace.tokens == inactive.trace.tokens;
            matched_pass += usize::from(pass);
            matched.push(json!({"group":p["group"],"pass":pass,"active_answer_changes":b.trace.tokens!=active.trace.tokens,"inactive_answer_unchanged":b.trace.tokens==inactive.trace.tokens}));
        }
        write(
            &output,
            "matched-variants.json",
            &json!({"rows":256,"pass_count":matched_pass,"items":matched,"complete_per_case_structure_checked_separately":true}),
        )?;
        let r492 = retained(&a, &g, &m, &root, &["development"], 492)?;
        write(&output, "retained-492.json", &r492)?;
        let r300 = retained(
            &a,
            &g,
            &m,
            &evidence.join("styled-role-1/attempt-3"),
            &["development", "collision"],
            300,
        )?;
        write(&output, "retained-300.json", &r300)?;
        let old_root = evidence.join("query-participation-1/attempt-1");
        report_output::verify(&old_root)?;
        let old_bytes = std::fs::read(old_root.join("candidate.json"))?;
        if sha(&old_bytes) != OLD_SHA {
            return Err("retained8055 identity".into());
        }
        let old = model::Artifact::decode(&old_bytes, &g)?;
        let r200 = retained_200(&a, &old, &g, &m, &evidence)?;
        write(&output, "retained-200.json", &r200)?;
        let r6688 = retained_6688(&a, &old, &g, &m, &evidence)?;
        write(&output, "retained-6688.json", &r6688)?;
        let unchanged = std::fs::read(root.join("candidate.json"))? == bytes;
        report_output::verify(&root)?;
        report_output::verify(&prepared)?;
        let pass = correct == 2304
            && exact == 2304
            && reloaded == 2304
            && disabled == 0
            && matched_pass == 256
            && r492["pass"] == true
            && r300["pass"] == true
            && r200["pass"] == true
            && r6688["pass"] == true
            && unchanged;
        let gate = if pass {
            "PASS_INDEPENDENT_NEIGHBOR_TRANSFER"
        } else {
            "FAIL_INDEPENDENT_NEIGHBOR_TRANSFER"
        };
        let mut summary = json!({"gate":gate,"rows":2304,"correct":correct,"panels":counts,"exact_equal":exact,"reload_equal":reloaded,"read_or_update_disabled_correct_answer":disabled,"matched_variant_groups_pass":matched_pass,"retained_492_equal":r492["equal"],"retained_300_equal":r300["equal"],"retained_200_equal":r200["equal"],"retained_6688_equal":r6688["equal"],"retained_typed_unresolved":r6688["typed_unresolved"]});
        summary.as_object_mut().ok_or("summary")?.extend(json!({"artifact_sha256":ARTIFACT_SHA,"artifact_unchanged":unchanged,"fits":0,"runtime_changes":0,"seed":data["seed"],"prepared_data_sha256":sha(&data_bytes),"acceptance_sha256":sha(&acceptance_bytes),"resumed_from":resume,"fresh_control_execution":"Original four controls retained; resume recovers one reload Full per row and reassesses saved Full traces, if resumed.","historical_controls":"NOT_RERUN; retained Full outputs only","promotion":false,"scope":"independent finite authored two-clause/four-record transfer with one-, two-, three-word endpoints; no general prose qualification","after_evaluation":"These cases are now exposed; any later adaptation requires a separate fresh draw after design selection."}).as_object().ok_or("summary extension")?.clone());
        write(&output, "summary.json", &summary)?;
        println!(
            "{}",
            serde_json::to_string(&json!({"gate":gate,"path":output}))?
        );
        Ok(())
    })
}
