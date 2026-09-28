//! Oracle read re-rank evaluation for the read-localization unit.
//!
//! It trains nothing, fits nothing and changes no weight. For the five frozen
//! condition-A distractor rows of the retained panel it rewrites the decision-0
//! read mass in memory (swap the entity with the top distractor; swap the top
//! distractor with a rank/age-matched control; focus the full non-NoRead mass on
//! the entity) and records the greedy continuation and verdict per arm, with the
//! read masses before and after to prove the intervention applied. See
//! `docs/integration/read-localization-result-2026-09-27.md` and the director
//! ROADMAP §4.2. Output is written only beneath a freshly claimed report root.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::answer_oracle;
use uor_r4_core::report_output;
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;
use uor_r4_training::baseline_protocol::{
    device, load_evaluator, save_json, verify_identity, Evaluator,
};
use uor_r4_training::joint_campaign::Campaign;
use uor_r4_training::joint_evaluation::{
    story_probes, JointGenerationStop, StoryProbe, STORY_PROBE_SCOPE, STORY_STOP_POLICY,
};
use uor_r4_training::joint_model::{JointModel, ReadMassIntervention, CHECKPOINT_SCHEMA};
use uor_r4_training::read_localize::{
    canonical_greedy, canonical_greedy_intervened, entity_spans, span_occurrence, CanonicalRun,
    DecisionZeroCapture, ReadEventMass, CANONICAL_MAX_NEW_TOKENS,
};
use uor_r4_training::{sha256_file, Result, TrainingError};

const RETAINED_EVALUATE_DIR: &str = "evaluate-quaternion-continuous-read-6";
const FLOAT_TOLERANCE: f64 = 1e-6;
/// The five condition-A distractor rows, in the retained panel's order:
/// (probe index, edited). Items are doll, bear, brush, box, bag.
const DISTRACTOR_ROWS: [(usize, bool); 5] =
    [(4, false), (4, true), (8, false), (12, false), (12, true)];
const DECLARED_ITEMS: [&str; 5] = ["doll", "bear", "brush", "box", "bag"];
const FROZEN_READING: &str = "S completes >=4/5 AND C <=1/5 -> ranking sufficient for this class; S <=1/5 -> the read is not the bottleneck; anything else -> inconclusive. The lead adjudicates; this instrument returns raw counts.";

fn invalid(message: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(message.into())
}

fn read_json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn identity(path: &Path) -> Result<Value> {
    Ok(json!({"path":path,"bytes":fs::metadata(path)?.len(),"sha256":sha256_file(path)?}))
}

fn decode_text(tokenizer: &HfBpeTokenizer, ids: &[u32]) -> String {
    String::from_utf8_lossy(&tokenizer.decode_bytes(ids)).into_owned()
}

fn first_word(text: &str) -> &str {
    text.split(|c: char| !c.is_alphabetic())
        .next()
        .unwrap_or("")
}

fn retained_story_probes_path(checkpoint: &Path) -> Result<PathBuf> {
    if let Ok(explicit) = std::env::var("UOR_RETAINED_STORY_PROBES") {
        return Ok(PathBuf::from(explicit));
    }
    let fit_root = checkpoint
        .parent()
        .ok_or_else(|| invalid("checkpoint parent (fit root) missing"))?;
    let investigation_root = fit_root
        .parent()
        .ok_or_else(|| invalid("checkpoint grandparent (investigation root) missing"))?;
    Ok(investigation_root
        .join(RETAINED_EVALUATE_DIR)
        .join("story-probes.json"))
}

fn load_tokenizer(evaluator: &Value) -> Result<(HfBpeTokenizer, Value)> {
    let identities = evaluator["reference_inputs"]
        .as_array()
        .ok_or_else(|| invalid("evaluator reference identity inventory"))?;
    let identity_value = identities
        .iter()
        .find(|entry| {
            entry["path"]
                .as_str()
                .is_some_and(|path| path.ends_with("/tokenizer.json"))
        })
        .ok_or_else(|| invalid("tokenizer identity missing from evaluator"))?;
    let path = verify_identity(identity_value)?;
    let tokenizer = HfBpeTokenizer::from_dir(
        path.parent()
            .ok_or_else(|| invalid("tokenizer parent directory"))?,
    )
    .map_err(|error| invalid(format!("joint tokenizer: {error}")))?;
    Ok((tokenizer, identity(&path)?))
}

#[derive(Clone, Copy)]
struct RowRef {
    probe: usize,
    edited: bool,
}

impl RowRef {
    fn key(self, id: &str) -> String {
        format!("{id}|{}", if self.edited { "edited" } else { "original" })
    }

    fn variant<'a>(
        self,
        probe: &'a StoryProbe,
    ) -> &'a uor_r4_training::joint_evaluation::StoryVariant {
        if self.edited {
            &probe.edited
        } else {
            &probe.original
        }
    }
}

struct OccurrenceStat {
    occurrence: usize,
    token: u32,
    text: String,
    mass: f64,
    rank: usize,
    age: usize,
}

impl OccurrenceStat {
    fn json(&self) -> Value {
        json!({
            "occurrence":self.occurrence,"token":self.token,"text":self.text,
            "mass":self.mass,"rank":self.rank,"age":self.age
        })
    }
}

/// Rank = 1-based position in the descending-mass order, ties broken by the
/// lower occurrence index. Age = readable tokens back from the decision step,
/// `written_occurrence - occurrence`.
fn occurrence_stats(decision: &DecisionZeroCapture) -> Vec<OccurrenceStat> {
    let mut order: Vec<&ReadEventMass> = decision.read_row.iter().collect();
    order.sort_by(|left, right| {
        right
            .mass
            .total_cmp(&left.mass)
            .then_with(|| left.occurrence.cmp(&right.occurrence))
    });
    let rank_by_occurrence: BTreeMap<usize, usize> = order
        .iter()
        .enumerate()
        .map(|(rank, event)| (event.occurrence, rank + 1))
        .collect();
    decision
        .read_row
        .iter()
        .map(|event| OccurrenceStat {
            occurrence: event.occurrence,
            token: event.token,
            text: event.text.clone(),
            mass: event.mass,
            rank: rank_by_occurrence[&event.occurrence],
            age: decision.written_occurrence - event.occurrence,
        })
        .collect()
}

/// Entity occurrence with the largest read mass (exactly one span is required).
fn entity_stat<'a>(stats: &'a [OccurrenceStat], occurrence: usize) -> Result<&'a OccurrenceStat> {
    stats
        .iter()
        .filter(|stat| stat.occurrence == occurrence)
        .max_by(|left, right| left.mass.total_cmp(&right.mass))
        .ok_or_else(|| invalid("entity occurrence is not a readable event at decision 0"))
}

/// Top-1 distractor: largest read mass excluding the entity; ties to the lower
/// occurrence index.
fn top_distractor<'a>(stats: &'a [OccurrenceStat], entity: usize) -> Result<&'a OccurrenceStat> {
    stats
        .iter()
        .filter(|stat| stat.occurrence != entity)
        .max_by(|left, right| {
            left.mass
                .total_cmp(&right.mass)
                .then_with(|| right.occurrence.cmp(&left.occurrence))
        })
        .ok_or_else(|| invalid("no non-entity read event for the top-1 distractor"))
}

/// Control occurrence: among non-entity occurrences excluding the distractor,
/// minimise `|rank(n) - rank(e)| + |age(n) - age(e)|`; ties to the lower
/// occurrence index.
fn matched_control<'a>(
    stats: &'a [OccurrenceStat],
    entity: &OccurrenceStat,
    distractor: usize,
) -> Result<&'a OccurrenceStat> {
    stats
        .iter()
        .filter(|stat| stat.occurrence != entity.occurrence && stat.occurrence != distractor)
        .min_by(|left, right| {
            let cost = |stat: &OccurrenceStat| {
                (stat.rank as i64 - entity.rank as i64).unsigned_abs()
                    + (stat.age as i64 - entity.age as i64).unsigned_abs()
            };
            cost(left)
                .cmp(&cost(right))
                .then_with(|| left.occurrence.cmp(&right.occurrence))
        })
        .ok_or_else(|| invalid("no control occurrence for the matched swap"))
}

fn complete_verdict(run: &CanonicalRun, noun: &str, accepted: &[String]) -> (bool, bool) {
    let first_noun_correct = first_word(&run.response_text) == noun;
    let complete_correct = run.utf8_decodable
        && matches!(
            run.stop,
            JointGenerationStop::Eos | JointGenerationStop::FirstSentenceBoundary
        )
        && answer_oracle::accepts(accepted, &run.response_text);
    (first_noun_correct, complete_correct)
}

fn masses_json(before: &[OccurrenceStat], after: &[OccurrenceStat]) -> Value {
    let before_by: BTreeMap<usize, f64> = before
        .iter()
        .map(|stat| (stat.occurrence, stat.mass))
        .collect();
    let after_by: BTreeMap<usize, f64> = after
        .iter()
        .map(|stat| (stat.occurrence, stat.mass))
        .collect();
    let mut occurrences: Vec<usize> = before_by.keys().copied().collect();
    occurrences.sort_unstable();
    Value::Array(
        occurrences
            .into_iter()
            .map(|occurrence| {
                json!({
                    "occurrence":occurrence,
                    "mass_before":before_by.get(&occurrence),
                    "mass_after":after_by.get(&occurrence)
                })
            })
            .collect(),
    )
}

/// The arm read row must equal the declared transform of the baseline row.
fn intervention_applied(
    before: &[OccurrenceStat],
    after: &[OccurrenceStat],
    intervention: ReadMassIntervention,
) -> bool {
    let before_by: BTreeMap<usize, f64> = before
        .iter()
        .map(|stat| (stat.occurrence, stat.mass))
        .collect();
    let after_by: BTreeMap<usize, f64> = after
        .iter()
        .map(|stat| (stat.occurrence, stat.mass))
        .collect();
    if before_by.keys().ne(after_by.keys()) {
        return false;
    }
    let close = |left: f64, right: f64| (left - right).abs() <= FLOAT_TOLERANCE;
    match intervention {
        ReadMassIntervention::Swap { a, b } => before_by.iter().all(|(&occurrence, &mass)| {
            let expected = if occurrence == a {
                before_by.get(&b).copied().unwrap_or(f64::NAN)
            } else if occurrence == b {
                before_by.get(&a).copied().unwrap_or(f64::NAN)
            } else {
                mass
            };
            close(after_by[&occurrence], expected)
        }),
        ReadMassIntervention::Focus { target } => {
            let total: f64 = before_by.values().sum();
            before_by.iter().all(|(&occurrence, _)| {
                let expected = if occurrence == target { total } else { 0.0 };
                close(after_by[&occurrence], expected)
            })
        }
    }
}

fn arm_json(
    name: &str,
    run: &CanonicalRun,
    noun: &str,
    accepted: &[String],
    before: &[OccurrenceStat],
    intervention: Option<ReadMassIntervention>,
) -> Result<Value> {
    let after = occurrence_stats(&run.decision_zero);
    let (first_noun_correct, complete_correct) = complete_verdict(run, noun, accepted);
    let applied = intervention.map(|value| intervention_applied(before, &after, value));
    Ok(json!({
        "arm":name,
        "intervention":intervention.map(|value| value.name()),
        "intervention_json":intervention,
        "intervention_applied_exactly":applied,
        "generated_token_ids":run.generated_token_ids,
        "response_text":run.response_text,
        "raw_decoded":run.raw_decoded,
        "stop":run.stop,
        "utf8_decodable":run.utf8_decodable,
        "first_noun_correct":first_noun_correct,
        "complete_correct":complete_correct,
        "written_occurrence":run.decision_zero.written_occurrence,
        "no_read_mass":run.decision_zero.no_read_mass,
        "read_mass_sum":run.decision_zero.read_mass_sum,
        "normalization_ok":run.decision_zero.normalization_ok,
        "top_read_occurrence":run.decision_zero.top_read_occurrence,
        "top_read_token":run.decision_zero.top_read_token,
        "top_read_token_text":run.decision_zero.top_read_token_text,
        "read_row":after.iter().map(OccurrenceStat::json).collect::<Vec<_>>(),
        "masses_before_after":masses_json(before, &after)
    }))
}

struct RowResult {
    key: String,
    probe: usize,
    edited: bool,
    json: Value,
}

fn process_row(
    model: &mut JointModel,
    tokenizer: &HfBpeTokenizer,
    probe: &StoryProbe,
    row: RowRef,
) -> Result<RowResult> {
    let key = row.key(&probe.id);
    let variant = row.variant(probe);
    let noun = variant.item.noun();
    let accepted = &variant.accepted_continuations;
    let content = tokenizer.encode(&variant.prompt);
    let decode = |ids: &[u32]| tokenizer.decode_bytes(ids);
    let spans = entity_spans(&content, noun, decode);
    if spans.len() != 1 {
        return Err(invalid(format!(
            "{key}: expected exactly one entity span, found {}",
            spans.len()
        )));
    }
    let entity_occurrence = span_occurrence(spans[0]);
    let mut full = Vec::with_capacity(content.len() + 1);
    full.push(0);
    full.extend_from_slice(&content);

    let baseline = canonical_greedy(&*model, tokenizer, &full, CANONICAL_MAX_NEW_TOKENS, None)?;
    let baseline_stats = occurrence_stats(&baseline.decision_zero);
    let entity = entity_stat(&baseline_stats, entity_occurrence)?;
    let distractor = top_distractor(&baseline_stats, entity_occurrence)?;
    let control = matched_control(&baseline_stats, entity, distractor.occurrence)?;
    let (baseline_first_noun, baseline_complete) = complete_verdict(&baseline, noun, accepted);
    if baseline_complete {
        return Err(invalid(format!(
            "{key}: baseline already completes; the distractor class is not reproduced"
        )));
    }

    let arms: [(&str, ReadMassIntervention); 3] = [
        (
            "S",
            ReadMassIntervention::Swap {
                a: entity.occurrence,
                b: distractor.occurrence,
            },
        ),
        (
            "C",
            ReadMassIntervention::Swap {
                a: distractor.occurrence,
                b: control.occurrence,
            },
        ),
        (
            "O",
            ReadMassIntervention::Focus {
                target: entity.occurrence,
            },
        ),
    ];
    let mut baseline_arm = arm_json("baseline", &baseline, noun, accepted, &baseline_stats, None)?;
    baseline_arm["intervention_applied_exactly"] = Value::Bool(true);
    let mut arm_values = vec![baseline_arm];
    for (name, intervention) in arms {
        let run = canonical_greedy_intervened(
            model,
            tokenizer,
            &full,
            CANONICAL_MAX_NEW_TOKENS,
            None,
            Some(intervention),
        )?;
        let value = arm_json(
            name,
            &run,
            noun,
            accepted,
            &baseline_stats,
            Some(intervention),
        )?;
        if value["intervention_applied_exactly"] != Value::Bool(true) {
            return Err(invalid(format!(
                "{key} arm {name}: read masses do not match the declared transform"
            )));
        }
        arm_values.push(value);
    }

    let json = json!({
        "row":key,
        "probe_id":probe.id,
        "template":probe.template,
        "variant":if row.edited {"edited"} else {"original"},
        "item":noun,
        "entity_span":spans[0],
        "entity_occurrence":entity_occurrence,
        "entity_tokens":content[spans[0].start..spans[0].start + spans[0].len].to_vec(),
        "prompt_token_ids":full,
        "prompt_decoded":decode_text(tokenizer, &content),
        "written_occurrence":baseline.decision_zero.written_occurrence,
        "baseline_first_noun_correct":baseline_first_noun,
        "baseline_complete_correct":baseline_complete,
        "selection":{
            "rule":"d = largest read mass excluding the entity (ties to the lower occurrence); n = among non-entity occurrences excluding d, minimise |rank(n)-rank(e)| + |age(n)-age(e)| (ties to the lower occurrence); rank = 1-based descending-mass position with occurrence tie-break; age = written_occurrence - occurrence",
            "e":entity.json(),
            "d":distractor.json(),
            "n":control.json()
        },
        "arms":arm_values
    });
    Ok(RowResult {
        key,
        probe: row.probe,
        edited: row.edited,
        json,
    })
}

fn baseline_parity(results: &[RowResult], retained: &Value) -> Value {
    let Some(retained_rows) = retained.as_array() else {
        return json!({"status":"PARITY_FAILED","reason":"retained story probes are not an array"});
    };
    let mut mismatches = Vec::new();
    for row in results {
        let Some(probe) = retained_rows.get(row.probe) else {
            mismatches.push(json!({"row":row.key,"field":"probe_index"}));
            continue;
        };
        let retained_row = &probe[if row.edited { "edited" } else { "original" }];
        let generation = &retained_row["generation"];
        let Some(baseline) = row.json["arms"].as_array().and_then(|arms| arms.first()) else {
            mismatches.push(json!({"row":row.key,"field":"baseline_arm"}));
            continue;
        };
        if baseline["generated_token_ids"] != generation["generated_token_ids"] {
            mismatches.push(json!({"row":row.key,"field":"generated_token_ids"}));
        }
        if baseline["stop"] != generation["stop"] {
            mismatches.push(json!({"row":row.key,"field":"stop"}));
        }
        if baseline["response_text"] != generation["response_text"] {
            mismatches.push(json!({"row":row.key,"field":"response_text"}));
        }
        if baseline["first_noun_correct"] != retained_row["first_noun_correct"] {
            mismatches.push(json!({"row":row.key,"field":"first_noun_correct"}));
        }
        if baseline["complete_correct"] != retained_row["complete_correct"] {
            mismatches.push(json!({"row":row.key,"field":"complete_correct"}));
        }
    }
    json!({
        "status":if mismatches.is_empty() {"PARITY_EXACT"} else {"PARITY_FAILED"},
        "rows_compared":results.len(),
        "mismatch_count":mismatches.len(),
        "mismatches":mismatches
    })
}

fn summarize(results: &[RowResult]) -> Value {
    let mut per_arm = serde_json::Map::new();
    for arm in ["baseline", "S", "C", "O"] {
        let mut complete = 0usize;
        let mut first_noun = 0usize;
        for row in results {
            let Some(value) = row.json["arms"]
                .as_array()
                .and_then(|arms| arms.iter().find(|value| value["arm"] == arm))
            else {
                continue;
            };
            complete += usize::from(value["complete_correct"] == Value::Bool(true));
            first_noun += usize::from(value["first_noun_correct"] == Value::Bool(true));
        }
        per_arm.insert(
            arm.to_string(),
            json!({"rows":results.len(),"complete":complete,"first_noun":first_noun}),
        );
    }
    let count = |arm: &str| {
        per_arm
            .get(arm)
            .and_then(|value| value["complete"].as_u64())
            .unwrap_or(0)
    };
    let rows: Vec<Value> = results
        .iter()
        .map(|row| {
            let arm = |name: &str| {
                row.json["arms"]
                    .as_array()
                    .and_then(|arms| arms.iter().find(|value| value["arm"] == name))
                    .cloned()
                    .unwrap_or(Value::Null)
            };
            json!({
                "row":row.key,"item":row.json["item"],
                "baseline":arm("baseline")["complete_correct"],
                "S":arm("S")["complete_correct"],
                "C":arm("C")["complete_correct"],
                "O":arm("O")["complete_correct"],
                "baseline_text":arm("baseline")["response_text"],
                "S_text":arm("S")["response_text"],
                "C_text":arm("C")["response_text"],
                "O_text":arm("O")["response_text"]
            })
        })
        .collect();
    json!({
        "per_arm":per_arm,
        "rows":rows,
        "reading_inputs":{
            "S_complete":count("S"),
            "C_complete":count("C"),
            "O_complete":count("O"),
            "baseline_complete":count("baseline"),
            "rows_total":results.len()
        },
        "frozen_reading":FROZEN_READING,
        "decision":"NOT_ADJUDICATED_BY_INSTRUMENT"
    })
}

fn run(
    checkpoint: &Path,
    campaign_path: &Path,
    out: &Path,
    device_name: &str,
    smoke: bool,
    started: Instant,
) -> Result<()> {
    report_output::verify(checkpoint)?;
    let campaign = Campaign::load(campaign_path)?;
    let evaluator: Evaluator = load_evaluator(&campaign.evaluator_path)?;
    let selected = device(device_name)?;
    let checkpoint_binding = read_json(&checkpoint.join("checkpoint.json"))?;
    let model_sha = sha256_file(&checkpoint.join("model.safetensors"))?;
    let config_sha = sha256_file(&checkpoint.join("config.json"))?;
    let checkpoint_sha = sha256_file(&checkpoint.join("checkpoint.json"))?;
    let campaign_sha = sha256_file(campaign_path)?;
    let step = checkpoint_binding["optimizer_step"]
        .as_u64()
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| invalid("checkpoint optimizer step"))?;
    if checkpoint_binding["schema"] != CHECKPOINT_SCHEMA
        || checkpoint_binding["evaluator_sha256"] != evaluator.sha256
        || checkpoint_binding["model_sha256"] != model_sha
        || checkpoint_binding["model_config_sha256"] != config_sha
    {
        return Err(invalid(
            "checkpoint binding, evaluator, model or config identity mismatch",
        ));
    }
    let mut model = JointModel::load(checkpoint, &selected)?;
    if model.config != campaign.model || model.config.context != 256 {
        return Err(invalid("loaded model config differs from the campaign"));
    }
    let (tokenizer, tokenizer_identity) = load_tokenizer(&evaluator.document)?;
    if tokenizer.vocab_size() != model.config.vocab_size {
        return Err(invalid("tokenizer vocabulary differs from the model"));
    }
    let retained_path = retained_story_probes_path(checkpoint)?;
    let retained_root = retained_path
        .parent()
        .ok_or_else(|| invalid("retained evaluate root"))?
        .to_path_buf();
    report_output::verify(&retained_root)?;
    let retained_sha = sha256_file(&retained_path)?;
    let retained = read_json(&retained_path)?;
    let probes = story_probes();
    if retained.as_array().map(Vec::len) != Some(probes.len()) {
        return Err(invalid(
            "retained story probe count differs from the frozen panel",
        ));
    }
    let source = option_env!("UOR_BUILD_SOURCE_COMMIT")
        .filter(|value| *value != "UNBOUND")
        .ok_or_else(|| invalid("build evidence with UOR_BUILD_SOURCE_COMMIT"))?;
    let provenance = json!({
        "schema":"uor-r4.oracle-rerank-provenance/1",
        "source_commit":source,
        "compiled_source_sha256":{
            "example":hex::encode(Sha256::digest(include_bytes!("joint-oracle-rerank.rs"))),
            "read_localize":hex::encode(Sha256::digest(include_bytes!("../src/read_localize.rs"))),
            "joint_model":hex::encode(Sha256::digest(include_bytes!("../src/joint_model.rs")))
        },
        "executable_sha256":sha256_file(&std::env::current_exe()?)?,
        "device":device_name,
        "parent_checkpoint":checkpoint,
        "parent_checkpoint_files":{
            "checkpoint_json_sha256":checkpoint_sha,
            "campaign_json_sha256":campaign_sha,
            "model_safetensors_sha256":model_sha,
            "config_json_sha256":config_sha
        },
        "parent_optimizer_step":step,
        "evaluator_path":evaluator.path,
        "evaluator_sha256":evaluator.sha256,
        "tokenizer":tokenizer_identity,
        "tokenizer_cid":tokenizer.address(),
        "retained_story_probes_path":retained_path,
        "retained_story_probes_sha256":retained_sha,
        "model_config":model.config,
        "read_mode":"enabled",
        "rows_selector":if smoke {"smoke"} else {"full"},
        "rows":DISTRACTOR_ROWS.iter().map(|(probe, edited)| json!({
            "probe":probe,"variant":if *edited {"edited"} else {"original"}
        })).collect::<Vec<_>>(),
        "declared_items":DECLARED_ITEMS,
        "intervention":{
            "scope":"in-memory only; never a checkpoint field; None preserves byte-identical behavior",
            "applied":"decision-0 step only, cleared before the next step",
            "arms":{"S":"Swap{e,d}","C":"Swap{d,n}","O":"Focus{e}"},
            "reading":FROZEN_READING
        },
        "scope":"Exposed development panel, one parent, deterministic greedy, read-only oracle read-mass intervention on five condition-A distractor rows. No training, fit, weight change, promotion or general-language claim.",
        "gradients_tracked":false,
        "optimizer_steps":0,
        "thread_environment":{
            "RAYON_NUM_THREADS":std::env::var("RAYON_NUM_THREADS").ok(),
            "VECLIB_MAXIMUM_THREADS":std::env::var("VECLIB_MAXIMUM_THREADS").ok()
        }
    });
    save_json(&out.join("run-provenance.json"), &provenance)?;

    let row_refs: Vec<RowRef> = if smoke {
        DISTRACTOR_ROWS
            .iter()
            .take(1)
            .map(|&(probe, edited)| RowRef { probe, edited })
            .collect()
    } else {
        DISTRACTOR_ROWS
            .iter()
            .map(|&(probe, edited)| RowRef { probe, edited })
            .collect()
    };
    let mut results = Vec::with_capacity(row_refs.len());
    for (index, row) in row_refs.iter().enumerate() {
        let probe = probes
            .get(row.probe)
            .ok_or_else(|| invalid("row probe index outside the frozen panel"))?;
        if row.variant(probe).item.noun() != DECLARED_ITEMS[index] {
            return Err(invalid(format!(
                "declared item {} does not match the frozen panel for {}",
                DECLARED_ITEMS[index],
                row.key(&probe.id)
            )));
        }
        results.push(process_row(&mut model, &tokenizer, probe, *row)?);
    }
    if model.read_intervention().is_some() {
        return Err(invalid("read intervention was not cleared after the rows"));
    }
    let parity = baseline_parity(&results, &retained);
    save_json(&out.join("baseline-parity.json"), &parity)?;
    let summary = summarize(&results);
    let report = json!({
        "schema":"uor-r4.oracle-rerank/1",
        "source_commit":source,
        "device":device_name,
        "parent_checkpoint":checkpoint,
        "parent_optimizer_step":step,
        "evaluator_sha256":evaluator.sha256,
        "tokenizer_cid":tokenizer.address(),
        "read_mode":"enabled",
        "story_probe_scope":STORY_PROBE_SCOPE,
        "story_probe_stop_policy":STORY_STOP_POLICY,
        "rows_total":if smoke {1} else {DISTRACTOR_ROWS.len()},
        "rows_selector":if smoke {"smoke"} else {"full"},
        "rows":results.iter().map(|row| row.json.clone()).collect::<Vec<_>>(),
        "baseline_parity":parity,
        "summary":summary["per_arm"],
        "summary_rows":summary["rows"],
        "reading_inputs":summary["reading_inputs"],
        "frozen_reading":summary["frozen_reading"],
        "decision":summary["decision"],
        "elapsed_seconds":started.elapsed().as_secs_f64()
    });
    save_json(&out.join("oracle-rerank.json"), &report)?;
    println!(
        "oracle-rerank: rows={} parity={} S_complete={} C_complete={} O_complete={} baseline_complete={} elapsed={:.1}s",
        results.len(),
        parity["status"],
        summary["reading_inputs"]["S_complete"],
        summary["reading_inputs"]["C_complete"],
        summary["reading_inputs"]["O_complete"],
        summary["reading_inputs"]["baseline_complete"],
        started.elapsed().as_secs_f64()
    );
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 5 {
        return Err(invalid(
            "usage: joint-oracle-rerank PARENT_CHECKPOINT CAMPAIGN_JSON|- NEW_REPORT_ROOT {cpu|metal} {full|smoke}",
        ));
    }
    let checkpoint = PathBuf::from(&args[0]).canonicalize()?;
    let out = PathBuf::from(&args[2]);
    let parent = out
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .canonicalize()?;
    if parent.starts_with(&checkpoint) || out.file_name().is_none() {
        return Err(invalid("new report must be outside the sealed checkpoint"));
    }
    let smoke = match args[4].to_ascii_lowercase().as_str() {
        "smoke" => true,
        "full" | "all" => false,
        _ => return Err(invalid("rows selector must be full or smoke")),
    };
    device(&args[3])?;
    let campaign_path = if args[1] == "-" {
        checkpoint.join("campaign.json")
    } else {
        PathBuf::from(&args[1]).canonicalize()?
    };
    report_output::claim(&out)?;
    let started = Instant::now();
    let result = run(&checkpoint, &campaign_path, &out, &args[3], smoke, started);
    if let Err(error) = &result {
        save_json(
            &out.join("failed-attempt.json"),
            &json!({"status":"FAILED_ATTEMPT","error":error.to_string()}),
        )?;
    }
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result?;
    println!("COMPLETE: {}; sealed and verified", out.display());
    Ok(())
}
