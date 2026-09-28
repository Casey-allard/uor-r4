//! Read-only read-side localization instrument for the mainline joint learner.
//! It trains nothing, changes no weight and writes only beneath its own freshly
//! claimed report root. See `docs/integration/read-localization-plan-2026-09-27.md`.
#![forbid(unsafe_code)]

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
use uor_r4_training::joint_model::{JointModel, CHECKPOINT_SCHEMA};
use uor_r4_training::read_localize::{
    canonical_greedy, entity_share, entity_spans, insert_index_before_final_query, length_delta,
    span_occurrence, splice_insert, splice_replace_final, CanonicalRun, CANONICAL_MAX_NEW_TOKENS,
    CONTROL_NOUN,
};
use uor_r4_training::{sha256_file, Result, TrainingError};

const RETAINED_EVALUATE_DIR: &str = "evaluate-quaternion-continuous-read-6";
const SMOKE_PROBE: usize = 4;
const FLOAT_TOLERANCE: f64 = 1e-9;

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

fn close(left: f64, right: f64) -> bool {
    (left - right).abs() <= FLOAT_TOLERANCE
}

fn first_word(text: &str) -> &str {
    text.split(|c: char| !c.is_alphabetic())
        .next()
        .unwrap_or("")
}

fn replace_last_word(prompt: &str, noun: &str) -> String {
    match prompt.rsplit_once(' ') {
        Some((head, _)) => format!("{head} {noun}"),
        None => noun.to_owned(),
    }
}

fn insert_clause_after_last_period(prompt: &str, clause: &str) -> String {
    let position = prompt.rfind('.').map_or(prompt.len(), |index| index + 1);
    format!("{}{}{}", &prompt[..position], clause, &prompt[position..])
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Condition {
    Baseline,
    EntityFinal,
    NonEntityFinal,
    NearQueryEntity,
    NearQueryControl,
}

impl Condition {
    const ALL: [Condition; 5] = [
        Condition::Baseline,
        Condition::EntityFinal,
        Condition::NonEntityFinal,
        Condition::NearQueryEntity,
        Condition::NearQueryControl,
    ];

    fn parse(raw: &str) -> Result<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "a" | "baseline" => Ok(Self::Baseline),
            "b" | "entity-final" => Ok(Self::EntityFinal),
            "c" | "non-entity-final" => Ok(Self::NonEntityFinal),
            "d" | "near-query-entity" => Ok(Self::NearQueryEntity),
            "d'" | "dp" | "dprime" | "near-query-non-entity" => Ok(Self::NearQueryControl),
            _ => Err(invalid(format!("unknown condition selector: {raw}"))),
        }
    }

    const fn key(self) -> &'static str {
        match self {
            Self::Baseline => "A",
            Self::EntityFinal => "B",
            Self::NonEntityFinal => "C",
            Self::NearQueryEntity => "D",
            Self::NearQueryControl => "DP",
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Baseline => "baseline",
            Self::EntityFinal => "entity_final",
            Self::NonEntityFinal => "matched_non_entity_final",
            Self::NearQueryEntity => "near_query_entity_mention",
            Self::NearQueryControl => "near_query_non_entity_mention",
        }
    }
}

fn parse_conditions(raw: &str) -> Result<Vec<Condition>> {
    if raw.eq_ignore_ascii_case("all") {
        return Ok(Condition::ALL.to_vec());
    }
    let mut selected = Vec::new();
    for part in raw.split(',') {
        let condition = Condition::parse(part)?;
        if !selected.contains(&condition) {
            selected.push(condition);
        }
    }
    if selected.is_empty() {
        return Err(invalid("at least one condition is required"));
    }
    Ok(selected)
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

fn parse_rows(raw: &str) -> Result<Option<RowRef>> {
    match raw.to_ascii_lowercase().as_str() {
        "all" => Ok(None),
        "smoke" => Ok(Some(RowRef {
            probe: SMOKE_PROBE,
            edited: false,
        })),
        _ => Err(invalid("rows selector must be all or smoke")),
    }
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

struct ConditionResult {
    condition: Condition,
    decoded_matches_intended: bool,
    first_noun_correct: bool,
    complete_correct: bool,
    entity_share: Option<f64>,
    entity_rank: Option<usize>,
    normalization_ok: bool,
    emitted_matches_top_read_token: bool,
    json: Value,
}

struct RowResult {
    key: String,
    probe: usize,
    edited: bool,
    item: String,
    excluded: Option<&'static str>,
    conditions: Vec<ConditionResult>,
    json: Value,
}

#[allow(clippy::too_many_arguments)]
fn condition_result(
    condition: Condition,
    content_len: usize,
    mutated: &[u32],
    decoded: &str,
    intended: &str,
    noun: &str,
    accepted: &[String],
    run: CanonicalRun,
    entity_occurrences: &[usize],
) -> Result<ConditionResult> {
    let decision = &run.decision_zero;
    let mut mass_by_occurrence = std::collections::BTreeMap::new();
    for event in &decision.read_row {
        mass_by_occurrence.insert(event.occurrence, event.mass);
    }
    let entity_mass: f64 = entity_occurrences
        .iter()
        .filter_map(|occurrence| mass_by_occurrence.get(occurrence))
        .copied()
        .sum();
    let entity_max_mass = entity_occurrences
        .iter()
        .filter_map(|occurrence| mass_by_occurrence.get(occurrence))
        .copied()
        .fold(0.0f64, f64::max);
    let entity_present = !entity_occurrences.is_empty();
    let entity_rank = entity_present.then(|| {
        1 + decision
            .read_row
            .iter()
            .filter(|event| event.mass > entity_max_mass)
            .count()
    });
    let emitted_token = *run
        .generated_token_ids
        .first()
        .ok_or_else(|| invalid("canonical generation produced no token"))?;
    let emitted_matches_top_read_token = decision
        .top_read_token
        .is_some_and(|token| token == emitted_token);
    let first_noun_correct = first_word(&run.response_text) == noun;
    let complete_correct = run.utf8_decodable
        && matches!(
            run.stop,
            JointGenerationStop::Eos | JointGenerationStop::FirstSentenceBoundary
        )
        && answer_oracle::accepts(accepted, &run.response_text);
    let mut top_events: Vec<_> = decision.read_row.iter().collect();
    top_events.sort_by(|left, right| {
        right
            .mass
            .total_cmp(&left.mass)
            .then_with(|| left.occurrence.cmp(&right.occurrence))
    });
    let top_events = top_events
        .into_iter()
        .take(5)
        .map(|event| {
            json!({
                "occurrence":event.occurrence,"token":event.token,"text":event.text,"mass":event.mass
            })
        })
        .collect::<Vec<_>>();
    let achieved_share = entity_share(entity_mass, decision.no_read_mass);
    let json = json!({
        "condition":condition.key(),
        "condition_name":condition.name(),
        "prompt_token_ids":run.prompt_token_ids,
        "prompt_decoded":decoded,
        "intended_text":intended,
        "decoded_matches_intended":decoded == intended,
        "length_change":length_delta(content_len, mutated.len()),
        "generated_token_ids":run.generated_token_ids,
        "response_text":run.response_text,
        "raw_decoded":run.raw_decoded,
        "stop":run.stop,
        "utf8_decodable":run.utf8_decodable,
        "first_noun_correct":first_noun_correct,
        "complete_correct":complete_correct,
        "written_occurrence":decision.written_occurrence,
        "final_prompt_token_occurrence":decision.final_prompt_token_occurrence,
        "final_prompt_token_is_readable":decision.final_prompt_token_is_readable,
        "readable_max_occurrence":decision.readable_max_occurrence,
        "read_mass_sum":decision.read_mass_sum,
        "no_read_mass":decision.no_read_mass,
        "copy_gate":decision.copy_gate,
        "effective_copy_mass":decision.effective_copy_mass,
        "normalization_ok":decision.normalization_ok,
        "entity_readable_occurrences":entity_occurrences,
        "entity_present":entity_present,
        "entity_mass":entity_mass,
        "entity_max_mass":entity_max_mass,
        "entity_share":achieved_share,
        "entity_rank":entity_rank,
        "top_read_occurrence":decision.top_read_occurrence,
        "top_read_token":decision.top_read_token,
        "top_read_token_text":decision.top_read_token_text,
        "top_read_mass":decision.top_read_mass,
        "top_events":top_events,
        "emitted_token":emitted_token,
        "emitted_token_text":decision.emitted_token_text,
        "emitted_matches_top_read_token":emitted_matches_top_read_token,
        "emitted_components":decision.emitted_components,
        "entity_probe_token":decision.probe_token,
        "entity_probe_components":decision.probe_components,
    });
    Ok(ConditionResult {
        condition,
        decoded_matches_intended: decoded == intended,
        first_noun_correct,
        complete_correct,
        entity_share: achieved_share,
        entity_rank,
        normalization_ok: decision.normalization_ok,
        emitted_matches_top_read_token,
        json,
    })
}

fn process_row(
    model: &JointModel,
    tokenizer: &HfBpeTokenizer,
    probe: &StoryProbe,
    row: RowRef,
    conditions: &[Condition],
    replay: bool,
) -> Result<RowResult> {
    let key = row.key(&probe.id);
    let variant = row.variant(probe);
    let noun = variant.item.noun();
    let accepted = &variant.accepted_continuations;
    let prompt = variant.prompt.clone();
    let content = tokenizer.encode(&prompt);
    let decode = |ids: &[u32]| tokenizer.decode_bytes(ids);
    let spans = entity_spans(&content, noun, decode);
    let (span, exclusion) = match spans.len() {
        1 => (Some(spans[0]), None),
        0 => (None, Some("unresolved_entity_span")),
        _ => (None, Some("ambiguous_entity_span")),
    };
    let Some(span) = span else {
        let json = json!({
            "row":key,"probe_id":probe.id,"template":probe.template,
            "variant":if row.edited {"edited"} else {"original"},
            "item":noun,"excluded":exclusion,"conditions":Vec::<Value>::new(),"replay":Value::Null
        });
        return Ok(RowResult {
            key,
            probe: row.probe,
            edited: row.edited,
            item: noun.to_owned(),
            excluded: exclusion,
            conditions: Vec::new(),
            json,
        });
    };
    let entity_tokens = content[span.start..span.start + span.len].to_vec();
    let entity_probe_token = content[span.start];
    let control_tokens = tokenizer.encode(&format!(" {CONTROL_NOUN}"));
    let clause_entity = tokenizer.encode(&format!(" Lily saw her {noun}."));
    let clause_control = tokenizer.encode(&format!(" Lily saw her {CONTROL_NOUN}."));
    let insert_index = insert_index_before_final_query(&content, decode).ok();
    let mut results = Vec::new();
    let mut values = Vec::new();
    for &condition in conditions {
        let (mutated, intended) = match condition {
            Condition::Baseline => (content.clone(), prompt.clone()),
            Condition::EntityFinal => (
                splice_replace_final(&content, &entity_tokens)?,
                replace_last_word(&prompt, noun),
            ),
            Condition::NonEntityFinal => (
                splice_replace_final(&content, &control_tokens)?,
                replace_last_word(&prompt, CONTROL_NOUN),
            ),
            Condition::NearQueryEntity => {
                let at = insert_index
                    .ok_or_else(|| invalid("condition D insertion boundary unresolved"))?;
                (
                    splice_insert(&content, at, &clause_entity)?,
                    insert_clause_after_last_period(&prompt, &format!(" Lily saw her {noun}.")),
                )
            }
            Condition::NearQueryControl => {
                let at = insert_index
                    .ok_or_else(|| invalid("condition D' insertion boundary unresolved"))?;
                (
                    splice_insert(&content, at, &clause_control)?,
                    insert_clause_after_last_period(
                        &prompt,
                        &format!(" Lily saw her {CONTROL_NOUN}."),
                    ),
                )
            }
        };
        let decoded = decode_text(tokenizer, &mutated);
        let mut full = Vec::with_capacity(mutated.len() + 1);
        full.push(0);
        full.extend_from_slice(&mutated);
        let run = canonical_greedy(
            model,
            tokenizer,
            &full,
            CANONICAL_MAX_NEW_TOKENS,
            Some(entity_probe_token),
        )?;
        let written = run.decision_zero.written_occurrence;
        let occurrences: Vec<usize> = entity_spans(&mutated, noun, decode)
            .into_iter()
            .map(span_occurrence)
            .filter(|&occurrence| occurrence < written)
            .collect();
        let result = condition_result(
            condition,
            content.len(),
            &mutated,
            &decoded,
            &intended,
            noun,
            accepted,
            run,
            &occurrences,
        )?;
        values.push(result.json.clone());
        results.push(result);
    }
    let replay_value = if replay {
        let mut full = Vec::with_capacity(content.len() + 1);
        full.push(0);
        full.extend_from_slice(&content);
        let again = canonical_greedy(
            model,
            tokenizer,
            &full,
            CANONICAL_MAX_NEW_TOKENS,
            Some(entity_probe_token),
        )?;
        let first = results
            .iter()
            .find(|result| result.condition == Condition::Baseline)
            .ok_or_else(|| invalid("smoke replay requires the baseline condition"))?;
        let identical = first.json["generated_token_ids"] == json!(again.generated_token_ids)
            && first.json["response_text"] == json!(again.response_text);
        json!({
            "generated_token_ids":first.json["generated_token_ids"],
            "replay_generated_token_ids":again.generated_token_ids,
            "identical":identical
        })
    } else {
        Value::Null
    };
    let length_change = |condition: Condition| {
        results
            .iter()
            .find(|result| result.condition == condition)
            .and_then(|result| result.json["length_change"].as_i64())
    };
    let d_dprime_length_matched = length_change(Condition::NearQueryEntity)
        == length_change(Condition::NearQueryControl)
        && length_change(Condition::NearQueryEntity).is_some();
    let json = json!({
        "row":key,"probe_id":probe.id,"template":probe.template,
        "variant":if row.edited {"edited"} else {"original"},
        "item":noun,
        "entity_span":span,
        "entity_baseline_occurrence":span_occurrence(span),
        "entity_tokens":entity_tokens,
        "excluded":Value::Null,
        "d_dprime_length_matched":d_dprime_length_matched,
        "conditions":values,
        "replay":replay_value
    });
    Ok(RowResult {
        key,
        probe: row.probe,
        edited: row.edited,
        item: noun.to_owned(),
        excluded: None,
        conditions: results,
        json,
    })
}

fn condition_of(row: &RowResult, condition: Condition) -> Option<&ConditionResult> {
    row.conditions
        .iter()
        .find(|result| result.condition == condition)
}

fn baseline_parity(results: &[RowResult], retained: &Value) -> Value {
    let retained_rows = match retained.as_array() {
        Some(rows) => rows,
        None => {
            return json!({"status":"PARITY_FAILED","reason":"retained story probes are not an array"})
        }
    };
    let mut mismatches = Vec::new();
    for row in results {
        let Some(probe) = retained_rows.get(row.probe) else {
            mismatches.push(json!({"row":row.key,"field":"probe_index"}));
            continue;
        };
        let retained_row = &probe[if row.edited { "edited" } else { "original" }];
        let generation = &retained_row["generation"];
        let Some(baseline) = condition_of(row, Condition::Baseline) else {
            mismatches.push(json!({"row":row.key,"field":"baseline_condition"}));
            continue;
        };
        if baseline.json["prompt_token_ids"] != generation["prompt_token_ids"] {
            mismatches.push(json!({"row":row.key,"field":"prompt_token_ids"}));
        }
        if baseline.json["generated_token_ids"] != generation["generated_token_ids"] {
            mismatches.push(json!({"row":row.key,"field":"generated_token_ids",
                "local":baseline.json["generated_token_ids"],"retained":generation["generated_token_ids"]}));
        }
        if baseline.json["stop"] != generation["stop"] {
            mismatches.push(json!({"row":row.key,"field":"stop",
                "local":baseline.json["stop"],"retained":generation["stop"]}));
        }
        if baseline.json["response_text"] != generation["response_text"] {
            mismatches.push(json!({"row":row.key,"field":"response_text",
                "local":baseline.json["response_text"],"retained":generation["response_text"]}));
        }
        if baseline.first_noun_correct != retained_row["first_noun_correct"] {
            mismatches.push(json!({"row":row.key,"field":"first_noun_correct"}));
        }
        if baseline.complete_correct != retained_row["complete_correct"] {
            mismatches.push(json!({"row":row.key,"field":"complete_correct"}));
        }
        let retained_decision = &generation["decisions"][0];
        for (field, local, retained_value) in [
            (
                "no_read_mass",
                &baseline.json["no_read_mass"],
                &retained_decision["no_read_mass"],
            ),
            (
                "copy_gate",
                &baseline.json["copy_gate"],
                &retained_decision["copy_gate"],
            ),
            (
                "effective_copy_mass",
                &baseline.json["effective_copy_mass"],
                &retained_decision["effective_copy_mass"],
            ),
            (
                "top_read_mass",
                &baseline.json["top_read_mass"],
                &retained_decision["top_read_mass"],
            ),
        ] {
            match (local.as_f64(), retained_value.as_f64()) {
                (Some(local), Some(retained_value)) if close(local, retained_value) => {}
                (Some(local), Some(retained_value)) => mismatches.push(
                    json!({"row":row.key,"field":field,"local":local,"retained":retained_value}),
                ),
                _ => mismatches.push(json!({"row":row.key,"field":field,"detail":"missing float"})),
            }
        }
        for field in ["top_read_occurrence", "top_read_token"] {
            if baseline.json[field] != retained_decision[field] {
                mismatches.push(json!({"row":row.key,"field":field,
                    "local":baseline.json[field],"retained":retained_decision[field]}));
            }
        }
        if baseline.json["emitted_token"] != retained_decision["selected_token"] {
            mismatches.push(json!({"row":row.key,"field":"selected_token",
                "local":baseline.json["emitted_token"],"retained":retained_decision["selected_token"]}));
        }
    }
    json!({
        "status":if mismatches.is_empty() {"PARITY_EXACT"} else {"PARITY_FAILED"},
        "rows_compared":results.len(),
        "mismatch_count":mismatches.len(),
        "mismatches":mismatches
    })
}

fn class_keys(results: &[RowResult]) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut distractor = Vec::new();
    let mut morphology = Vec::new();
    let mut extra_phrase = Vec::new();
    for row in results {
        let Some(baseline) = condition_of(row, Condition::Baseline) else {
            continue;
        };
        if baseline.complete_correct {
            continue;
        }
        let response = baseline.json["response_text"].as_str().unwrap_or("");
        let word = first_word(response);
        if !baseline.first_noun_correct && word.starts_with(&row.item) && word != row.item {
            morphology.push(row.key.clone());
        } else if !baseline.first_noun_correct {
            distractor.push(row.key.clone());
        } else {
            extra_phrase.push(row.key.clone());
        }
    }
    (distractor, morphology, extra_phrase)
}

fn matches_declared(derived: &[String], declared: &[&str]) -> bool {
    let mut left = derived.to_vec();
    let mut right = declared
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>();
    left.sort();
    right.sort();
    left == right
}

fn summarize(results: &[RowResult], conditions: &[Condition], parity: &Value) -> Value {
    let (distractor, morphology, extra_phrase) = class_keys(results);
    let mut class_summary = serde_json::Map::new();
    for (label, keys) in [
        ("distractor", &distractor),
        ("morphology", &morphology),
        ("extra_phrase", &extra_phrase),
    ] {
        let mut per_condition = serde_json::Map::new();
        for &condition in conditions {
            let mut complete = 0usize;
            let mut first_noun = 0usize;
            let mut present = 0usize;
            for key in keys {
                let row = results.iter().find(|row| &row.key == key);
                if let Some(result) = row.and_then(|row| condition_of(row, condition)) {
                    complete += usize::from(result.complete_correct);
                    first_noun += usize::from(result.first_noun_correct);
                    present += 1;
                }
            }
            per_condition.insert(
                condition.key().to_string(),
                json!({"rows":present,"complete":complete,"first_noun":first_noun}),
            );
        }
        class_summary.insert(
            label.to_string(),
            json!({"rows":keys,"per_condition":per_condition}),
        );
    }
    let distractor_rows: Vec<&RowResult> = results
        .iter()
        .filter(|row| distractor.contains(&row.key))
        .collect();
    let share_lt_5_and_rank_gt_5 = distractor_rows
        .iter()
        .filter(|row| {
            condition_of(row, Condition::Baseline).is_some_and(|result| {
                result.entity_share.is_some_and(|share| share < 0.05)
                    && result.entity_rank.is_some_and(|rank| rank > 5)
            })
        })
        .count();
    let emitted_wrong_matches_top_read = distractor_rows
        .iter()
        .filter(|row| {
            condition_of(row, Condition::Baseline).is_some_and(|result| {
                !result.first_noun_correct && result.emitted_matches_top_read_token
            })
        })
        .count();
    let raises = |condition: Condition| {
        distractor_rows
            .iter()
            .filter(|row| {
                condition_of(row, condition).is_some_and(|result| {
                    result.entity_share.is_some_and(|share| share >= 0.15)
                        || result.entity_rank.is_some_and(|rank| rank <= 3)
                })
            })
            .count()
    };
    let state_emission_rows = distractor_rows
        .iter()
        .filter(|row| {
            condition_of(row, Condition::Baseline).is_some_and(|result| {
                !result.complete_correct
                    && (result.entity_share.is_some_and(|share| share >= 0.10)
                        || result.entity_rank.is_some_and(|rank| rank <= 3))
            })
        })
        .count();
    let changes_verdict = distractor_rows
        .iter()
        .filter(|row| {
            let baseline = condition_of(row, Condition::Baseline);
            let changed = |condition: Condition| {
                matches!(
                    (baseline, condition_of(row, condition)),
                    (Some(a), Some(b)) if a.complete_correct != b.complete_correct
                )
            };
            changed(Condition::NearQueryEntity) || changed(Condition::NearQueryControl)
        })
        .count();
    let unresolved: Vec<String> = results
        .iter()
        .filter(|row| row.excluded == Some("unresolved_entity_span"))
        .map(|row| row.key.clone())
        .collect();
    let ambiguous: Vec<String> = results
        .iter()
        .filter(|row| row.excluded == Some("ambiguous_entity_span"))
        .map(|row| row.key.clone())
        .collect();
    let d_dprime_length_mismatched: Vec<String> = results
        .iter()
        .filter(|row| row.json["d_dprime_length_matched"].as_bool() == Some(false))
        .map(|row| row.key.clone())
        .collect();
    let mutation_text_mismatches: Vec<Value> = results
        .iter()
        .flat_map(|row| {
            row.conditions
                .iter()
                .filter(|result| !result.decoded_matches_intended)
                .map(move |result| {
                    json!({"row":row.key,"condition":result.condition.key(),
                        "decoded":result.json["prompt_decoded"],"intended":result.json["intended_text"]})
                })
        })
        .collect();
    json!({
        "class_summary":class_summary,
        "predeclared_rule_inputs":{
            "entity_share_definition":"entity occurrence mass sum / (1 - NoRead mass), condition A",
            "distractor_rows":distractor,
            "declared_sets_match_derived":{
                "distractor":matches_declared(&distractor, &[
                    "story-source-edit-04|original","story-source-edit-04|edited",
                    "story-source-edit-08|original","story-source-edit-12|original",
                    "story-source-edit-12|edited"]),
                "morphology":matches_declared(&morphology, &["story-source-edit-00|edited"]),
                "extra_phrase":matches_declared(&extra_phrase, &[
                    "story-source-edit-03|original","story-source-edit-07|original",
                    "story-source-edit-07|edited","story-source-edit-11|edited",
                    "story-source-edit-15|edited"])
            },
            "a_entity_share_lt_5pct_and_rank_gt_5_count":share_lt_5_and_rank_gt_5,
            "a_emitted_wrong_noun_equals_top_read_token_count":emitted_wrong_matches_top_read,
            "d_entity_share_ge_15pct_or_rank_le_3_count":raises(Condition::NearQueryEntity),
            "dprime_entity_share_ge_15pct_or_rank_le_3_count":raises(Condition::NearQueryControl),
            "a_entity_share_ge_10pct_or_rank_le_3_and_output_wrong_count":state_emission_rows,
            "d_or_dprime_changes_verdict_count":changes_verdict,
            "decision":"NOT_ADJUDICATED_BY_INSTRUMENT"
        },
        "guardrails":{
            "masses_normalize_all":results.iter().all(|row| row.conditions.iter().all(|result| result.normalization_ok)),
            "no_future_reads_enforced":true,
            "baseline_parity_status":parity["status"],
            "rows_unresolved_span":unresolved,
            "rows_ambiguous_span":ambiguous,
            "d_dprime_length_mismatched_rows":d_dprime_length_mismatched,
            "mutation_decoded_text_mismatches":mutation_text_mismatches
        }
    })
}

#[allow(clippy::too_many_arguments)]
fn run(
    checkpoint: &Path,
    campaign_path: &Path,
    out: &Path,
    device_name: &str,
    conditions: &[Condition],
    rows: Option<RowRef>,
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
    let model = JointModel::load(checkpoint, &selected)?;
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
        "schema":"uor-r4.read-localization-provenance/1",
        "source_commit":source,
        "compiled_source_sha256":{
            "example":hex::encode(Sha256::digest(include_bytes!("joint-read-localize.rs"))),
            "read_localize":hex::encode(Sha256::digest(include_bytes!("../src/read_localize.rs")))
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
        "conditions":conditions.iter().map(|condition| condition.key()).collect::<Vec<_>>(),
        "rows_selector":if rows.is_some() {"smoke"} else {"all"},
        "scope":"Exposed development panel, one parent, deterministic greedy; read-only localization of the read/state/emission seams. No training, weight change, promotion or general-language claim.",
        "gradients_tracked":false,
        "optimizer_steps":0,
        "thread_environment":{
            "RAYON_NUM_THREADS":std::env::var("RAYON_NUM_THREADS").ok(),
            "VECLIB_MAXIMUM_THREADS":std::env::var("VECLIB_MAXIMUM_THREADS").ok()
        }
    });
    save_json(&out.join("run-provenance.json"), &provenance)?;
    let row_refs: Vec<RowRef> = match rows {
        Some(row) => vec![row],
        None => probes
            .iter()
            .enumerate()
            .flat_map(|(index, _)| {
                [
                    RowRef {
                        probe: index,
                        edited: false,
                    },
                    RowRef {
                        probe: index,
                        edited: true,
                    },
                ]
            })
            .collect(),
    };
    let mut results = Vec::with_capacity(row_refs.len());
    for row in &row_refs {
        let probe = probes
            .get(row.probe)
            .ok_or_else(|| invalid("row probe index outside the frozen panel"))?;
        results.push(process_row(
            &model,
            &tokenizer,
            probe,
            *row,
            conditions,
            rows.is_some(),
        )?);
    }
    let parity = if rows.is_none() {
        baseline_parity(&results, &retained)
    } else {
        json!({"status":"NOT_RUN_SMOKE","rows_compared":results.len(),"mismatch_count":Value::Null,"mismatches":[]})
    };
    save_json(&out.join("baseline-parity.json"), &parity)?;
    let summary = summarize(&results, conditions, &parity);
    let replay_identical = results
        .first()
        .and_then(|row| row.json["replay"]["identical"].as_bool());
    let over_context: Vec<String> = results
        .iter()
        .filter(|row| row.conditions.is_empty() && row.excluded.is_none())
        .map(|row| row.key.clone())
        .collect();
    let report = json!({
        "schema":"uor-r4.read-localization/1",
        "source_commit":source,
        "device":device_name,
        "parent_checkpoint":checkpoint,
        "evaluator_sha256":evaluator.sha256,
        "tokenizer_cid":tokenizer.address(),
        "read_mode":"enabled",
        "story_probe_scope":STORY_PROBE_SCOPE,
        "story_probe_stop_policy":STORY_STOP_POLICY,
        "conditions":conditions.iter().map(|condition| json!({"key":condition.key(),"name":condition.name()})).collect::<Vec<_>>(),
        "rows_total":probes.len()*2,
        "rows_selected":results.len(),
        "rows":results.iter().map(|row| row.json.clone()).collect::<Vec<_>>(),
        "baseline_parity":parity,
        "class_summary":summary["class_summary"],
        "predeclared_rule_inputs":summary["predeclared_rule_inputs"],
        "guardrails":summary["guardrails"],
        "replay_identical":replay_identical,
        "rows_without_conditions":over_context,
        "elapsed_seconds":started.elapsed().as_secs_f64()
    });
    save_json(&out.join("read-localization.json"), &report)?;
    let parity_status = if rows.is_none() {
        parity["status"].as_str().unwrap_or("PARITY_FAILED")
    } else {
        "NOT_RUN_SMOKE"
    };
    println!(
        "read-localize: rows={} parity={} normalize_all={} elapsed={:.1}s",
        results.len(),
        parity_status,
        summary["guardrails"]["masses_normalize_all"],
        started.elapsed().as_secs_f64()
    );
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 6 {
        return Err(invalid(
            "usage: joint-read-localize PARENT_CHECKPOINT CAMPAIGN_JSON|- NEW_REPORT_ROOT {cpu|metal} CONDITIONS ROWS; CONDITIONS=all|A,B,C,D,DP; ROWS=all|smoke",
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
    let conditions = parse_conditions(&args[4])?;
    let rows = parse_rows(&args[5])?;
    device(&args[3])?;
    let campaign_path = if args[1] == "-" {
        checkpoint.join("campaign.json")
    } else {
        PathBuf::from(&args[1]).canonicalize()?
    };
    report_output::claim(&out)?;
    let started = Instant::now();
    let result = run(
        &checkpoint,
        &campaign_path,
        &out,
        &args[3],
        &conditions,
        rows,
        started,
    );
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
