#![forbid(unsafe_code)]

use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, env, error::Error, fs, path::Path};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Span = [u32; 2];

const D26: &str = "b17352679c64a96bf657fad7640d4bb05ea3de0ecd08dadd13e9d2756e979bc4";
const TOKENIZER: &str = "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377";
const GOLD: &str = "220ec3c5fd0c839bed864c051458b30f95668cc1b7a26c725d646d189fdc8096";
const ARMS: [&str; 4] = [
    "historical-inherited",
    "historical-repeated",
    "full-inherited",
    "full-repeated",
];

#[derive(Deserialize)]
struct Targets {
    requested: Option<Span>,
    excluded: Option<Span>,
    unrequested: Option<Span>,
}

#[derive(Deserialize)]
struct Gold {
    row: u32,
    family: String,
    stratum: String,
    role_factors: Option<[usize; 3]>,
    role_targets: Targets,
    assertion_heads: Vec<Span>,
    value_spans: Vec<Span>,
    original_ids: Vec<u32>,
    gold_source: Option<u32>,
    expected_tape: Vec<u32>,
}

#[derive(Deserialize)]
struct Choice {
    fact: Option<u32>,
    source: Option<u32>,
}

#[derive(Deserialize)]
struct Paired {
    role_words: Option<[u32; 8]>,
}

#[derive(Deserialize)]
struct Client {
    artifact_sha256: String,
    tokenizer_sha256: String,
    model_schema: String,
    contextual_source_arm: String,
    protocol: String,
    read_enabled: bool,
    prompt_ids: Vec<u32>,
    generated_ids: Vec<u32>,
    copy_step_words: Vec<[u32; 8]>,
    selected: Option<Choice>,
    paired: Paired,
    role_replay: Option<Audit>,
    stop: String,
}

#[derive(Deserialize)]
struct Audit {
    content: Span,
    query_boundary: Option<u32>,
    candidates: Vec<Candidate>,
    assertions: Option<Vec<Value>>,
    contextual_sources: Option<Vec<SourceWitness>>,
}

#[derive(Deserialize)]
struct SourceWitness {
    fact_span: Span,
    selection: SourceSelection,
    candidates: Vec<(Value, SourceScore)>,
}

#[derive(Deserialize)]
struct SourceSelection {
    source: Option<u32>,
    outcome: String,
}

#[derive(Deserialize)]
struct SourceScore {
    source: u32,
    eligible: bool,
    margin: Option<i64>,
}

#[derive(Deserialize)]
struct Candidate {
    span: Span,
    inherited: bool,
    repeated_head: Option<Span>,
    requested_admitted: bool,
    excluded_admitted: bool,
    eligibility: String,
    supporting_head: Option<Span>,
}

#[derive(Deserialize)]
struct StoredAudit {
    row: u32,
    content: Span,
    candidate_boundary: Option<u32>,
    role_words: [u32; 8],
    all_candidates: Vec<StoredCandidate>,
}

#[derive(Deserialize)]
struct StoredCandidate {
    span: Span,
    scores: [i32; 2],
    excluded_decision_scores: Option<[i32; 2]>,
    eligibility: String,
    supporting_head: Option<Span>,
    contrast: Option<i32>,
}

impl From<StoredAudit> for Audit {
    fn from(stored: StoredAudit) -> Self {
        let candidates = stored
            .all_candidates
            .into_iter()
            .map(|candidate| {
                let [requested, excluded] = candidate
                    .excluded_decision_scores
                    .unwrap_or(candidate.scores);
                Candidate {
                    span: candidate.span,
                    inherited: true,
                    repeated_head: None,
                    requested_admitted: candidate.contrast.is_some(),
                    excluded_admitted: excluded > requested,
                    eligibility: candidate.eligibility,
                    supporting_head: candidate.supporting_head,
                }
            })
            .collect();
        Self {
            content: stored.content,
            query_boundary: stored.candidate_boundary,
            candidates,
            assertions: None,
            contextual_sources: None,
        }
    }
}

fn first_difference(expected: &[u32], actual: &[u32]) -> Option<(usize, Option<u32>, Option<u32>)> {
    (0..expected.len().max(actual.len())).find_map(|offset| {
        let wanted = expected.get(offset).copied();
        let found = actual.get(offset).copied();
        (wanted != found).then_some((offset, wanted, found))
    })
}

fn endpoint(audit: &Audit, target: Option<Span>) -> Option<Value> {
    let span = target?;
    let candidate = audit
        .candidates
        .iter()
        .find(|candidate| candidate.span == span);
    Some(json!({
        "span": span,
        "present": candidate.is_some(),
        "inherited": candidate.is_some_and(|candidate| candidate.inherited),
        "repeated_head": candidate.and_then(|candidate| candidate.repeated_head),
        "requested_admitted": candidate.is_some_and(|candidate| candidate.requested_admitted),
        "excluded_admitted": candidate.is_some_and(|candidate| candidate.excluded_admitted),
        "eligibility": candidate.map(|candidate| &candidate.eligibility),
        "supporting_head": candidate.and_then(|candidate| candidate.supporting_head),
        "before_query_boundary": audit.query_boundary.map(|boundary| span[0] < boundary)
    }))
}

fn read_client(path: &Path, gold: &Gold) -> Result<(Value, Client)> {
    let raw: Value = serde_json::from_slice(&fs::read(path)?)?;
    let client: Client = serde_json::from_value(raw.clone())?;
    if client.artifact_sha256 != D26
        || client.tokenizer_sha256 != TOKENIZER
        || client.model_schema != "uor-r4.lut-discrete-contextual-source/2"
        || client.contextual_source_arm != "wide_recurrence"
        || client.protocol != "uor-r4.literal-role-dialogue/2"
        || !client.read_enabled
        || client.prompt_ids != gold.original_ids
        || client.generated_ids.len() != client.copy_step_words.len()
    {
        return Err(format!("bad carrier, prompt or writer trace at {}", path.display()).into());
    }
    Ok((raw, client))
}

fn match_baseline(raw: &Value, historical: &Value, row: u32) -> Result<()> {
    for field in [
        "prompt_ids",
        "selected",
        "paired",
        "generated_ids",
        "copy_step_words",
        "stop",
        "text",
    ] {
        if raw.get(field).is_none() || raw.get(field) != historical.get(field) {
            return Err(format!("D26 historical row {row} differs at {field}").into());
        }
    }
    Ok(())
}

fn check_arm_rows(arm_dir: &Path, gold: &[Gold]) -> Result<()> {
    let expected: BTreeSet<_> = gold.iter().map(|row| format!("{}.json", row.row)).collect();
    let mut observed = BTreeSet::new();
    for entry in fs::read_dir(arm_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            return Err(format!("non-file capture at {}", entry.path().display()).into());
        }
        observed.insert(
            entry
                .file_name()
                .into_string()
                .map_err(|_| "invalid capture filename")?,
        );
    }
    if observed != expected {
        return Err(format!("missing or extra capture rows at {}", arm_dir.display()).into());
    }
    Ok(())
}

fn source_witness(gold: &Gold, audit: &Audit, source: Option<u32>) -> Option<Value> {
    let expected = gold.gold_source?;
    let (&fact_span, _) = gold
        .assertion_heads
        .iter()
        .zip(&gold.value_spans)
        .find(|(_, [start, end])| (*start..*end).contains(&expected))?;
    let assertions = audit.assertions.as_ref()?;
    let bound = assertions
        .iter()
        .any(|row| row["fact_span"] == json!(fact_span));
    let witness = audit
        .contextual_sources
        .as_ref()?
        .iter()
        .find(|witness| witness.fact_span == fact_span);
    let score = witness.and_then(|witness| {
        witness
            .candidates
            .iter()
            .find(|(_, score)| score.source == expected)
    });
    let diagnosis = match (bound, witness, score) {
        _ if source == Some(expected) => "selected_expected_source",
        (false, _, _) => "expected_assertion_not_bound",
        (_, None, _) => "no_contextual_read_for_expected_assertion",
        (_, _, None) => "expected_source_not_enumerated",
        (_, _, Some((_, score))) if !score.eligible => "expected_source_masked",
        (_, Some(witness), _) if witness.selection.source == Some(expected) => {
            "expected_local_pick_lost_after_ranking"
        }
        _ => "eligible_expected_source_lost_in_ranking",
    };
    Some(json!({
        "expected_fact_span": fact_span,
        "bound": bound,
        "enumerated": score.is_some(),
        "eligible": score.is_some_and(|(_, score)| score.eligible),
        "margin": score.and_then(|(_, score)| score.margin),
        "actual_local_pick": witness.and_then(|witness| witness.selection.source),
        "local_outcome": witness.map(|witness| &witness.selection.outcome),
        "diagnosis": diagnosis
    }))
}

fn grade(arm: &str, gold: &Gold, client: &Client, baseline: &Client) -> Result<Value> {
    let audit = client
        .role_replay
        .as_ref()
        .ok_or("missing role replay audit")?;
    let (requested, excluded) = client.paired.role_words.map_or((None, None), |words| {
        let requested = [words[0], words[1]];
        let excluded = [words[2], words[3]];
        (
            (requested[0] != u32::MAX).then_some(requested),
            (excluded[0] != u32::MAX).then_some(excluded),
        )
    });
    let source = client.selected.as_ref().and_then(|choice| choice.source);
    let source_fact = client.selected.as_ref().and_then(|choice| choice.fact);
    let requested_correct = requested == gold.role_targets.requested;
    let excluded_correct = excluded == gold.role_targets.excluded;
    let has_incidental_choice = gold
        .role_targets
        .unrequested
        .is_some_and(|span| requested == Some(span) || excluded == Some(span));
    let mismatch = first_difference(&gold.expected_tape, &client.generated_ids);
    let exact =
        mismatch.is_none() && client.stop == "eos" && client.generated_ids.last() == Some(&1);
    let copied_through_eos = client.stop == "eos"
        && client.copy_step_words.last().is_some_and(|words| {
            // ExactCopyStep's serialized cursor, source, emission and release.
            words[2] > words[1] && words[4] == 1 && words[7] == 1
        });
    Ok(json!({
        "arm": arm,
        "row": gold.row,
        "family": gold.family,
        "stratum": gold.stratum,
        "factors": gold.role_factors,
        "content": audit.content,
        "query_boundary": audit.query_boundary,
        "candidate_count": audit.candidates.len(),
        "requested": endpoint(audit, gold.role_targets.requested),
        "excluded": endpoint(audit, gold.role_targets.excluded),
        "incidental": endpoint(audit, gold.role_targets.unrequested),
        "assertion_heads_gold": gold.assertion_heads,
        "assertions": audit.assertions,
        "role_words": client.paired.role_words,
        "requested_actual": requested,
        "excluded_actual": excluded,
        "requested_correct": requested_correct,
        "excluded_correct": excluded_correct,
        "joint_correct": requested_correct && excluded_correct,
        "incidental_chosen": has_incidental_choice,
        "selected_source": source,
        "selected_fact": source_fact,
        "gold_source": gold.gold_source,
        "correct_source": gold.gold_source.map(|address| source == Some(address)),
        "source_witness": source_witness(gold, audit, source),
        "unsupported_copy": gold.gold_source.is_none() && source.is_some(),
        "first_change_from_historical": first_difference(&baseline.generated_ids, &client.generated_ids),
        "first_error": gold.gold_source.and(mismatch),
        "exact_through_eos": gold.gold_source.map(|_| exact),
        "copied_through_eos": copied_through_eos,
        "refusal_comparison_through_eos": gold.gold_source.is_none().then_some(exact),
        "expected_tape": gold.expected_tape,
        "generated_ids": client.generated_ids,
        "stop": client.stop,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn positive() -> (Gold, Client) {
        let gold = Gold {
            row: 1,
            family: "fixture".into(),
            stratum: "crossed".into(),
            role_factors: None,
            role_targets: Targets {
                requested: Some([6, 7]),
                excluded: Some([7, 8]),
                unrequested: Some([8, 9]),
            },
            assertion_heads: vec![[3, 4]],
            value_spans: vec![[4, 5]],
            original_ids: vec![],
            gold_source: Some(4),
            expected_tape: vec![31, 1],
        };
        let client = Client {
            artifact_sha256: D26.into(),
            tokenizer_sha256: TOKENIZER.into(),
            model_schema: String::new(),
            contextual_source_arm: String::new(),
            protocol: String::new(),
            read_enabled: true,
            prompt_ids: vec![],
            generated_ids: vec![31, 1],
            copy_step_words: vec![[1, 4, 5, 9, 31, 31, 1, 0], [0, 4, 5, 9, 1, 0, 0, 1]],
            selected: Some(Choice {
                fact: Some(3),
                source: Some(4),
            }),
            paired: Paired {
                role_words: Some([6, 7, 7, 8, 1, 0, 1, 0]),
            },
            role_replay: Some(Audit {
                content: [3, 9],
                query_boundary: None,
                candidates: vec![Candidate {
                    span: [6, 7],
                    inherited: false,
                    repeated_head: Some([3, 4]),
                    requested_admitted: true,
                    excluded_admitted: false,
                    eligibility: "eligible".into(),
                    supporting_head: Some([3, 4]),
                }],
                assertions: Some(vec![]),
                contextual_sources: Some(vec![]),
            }),
            stop: "eos".into(),
        };
        (gold, client)
    }

    #[test]
    fn exact_reply_remains_distinct_from_endpoint_availability() -> Result<()> {
        let (gold, client) = positive();
        let row = grade("gold", &gold, &client, &client)?;
        assert_eq!(row["excluded"]["present"], false);
        assert_eq!(row["requested"]["requested_admitted"], true);
        assert_eq!(row["requested"]["excluded_admitted"], false);
        assert_eq!(row["joint_correct"], true);
        assert_eq!(row["correct_source"], true);
        assert_eq!(row["exact_through_eos"], true);
        assert_eq!(row["copied_through_eos"], true);
        Ok(())
    }

    #[test]
    fn swapped_roles_and_truncated_tape_fail_independently() -> Result<()> {
        let (gold, baseline) = positive();
        let (_, mut wrong) = positive();
        wrong.paired.role_words = Some([7, 8, 8, 9, 0, 0, 0, 0]);
        wrong.selected = None;
        wrong.generated_ids.pop();
        wrong.copy_step_words.pop();
        wrong.stop = "max_tokens".into();
        let row = grade("wrong", &gold, &wrong, &baseline)?;
        assert_eq!(row["joint_correct"], false);
        assert_eq!(row["incidental_chosen"], true);
        assert_eq!(row["correct_source"], false);
        assert_eq!(row["first_change_from_historical"], json!([1, 1, null]));
        assert_eq!(row["first_error"], json!([1, 1, null]));
        assert_eq!(row["exact_through_eos"], false);
        assert_eq!(row["copied_through_eos"], false);
        Ok(())
    }

    #[test]
    fn unsupported_copy_is_a_failure_even_with_refusal_comparison_tape() -> Result<()> {
        let (mut gold, client) = positive();
        gold.gold_source = None;
        let row = grade("unsupported", &gold, &client, &client)?;
        assert_eq!(row["unsupported_copy"], true);
        assert!(row["exact_through_eos"].is_null());
        assert_eq!(row["refusal_comparison_through_eos"], true);
        Ok(())
    }

    #[test]
    fn source_rank_loss_requires_an_eligible_enumerated_source() -> Result<()> {
        let (gold, mut client) = positive();
        client.selected = None;
        let audit = client.role_replay.as_mut().ok_or("missing fixture audit")?;
        audit.assertions = Some(vec![json!({"fact_span": [3, 4]})]);
        let mut witness = SourceWitness {
            fact_span: [3, 4],
            selection: SourceSelection {
                source: None,
                outcome: "nonpositive".into(),
            },
            candidates: vec![],
        };
        audit.contextual_sources = Some(vec![]);
        let row = source_witness(&gold, audit, None).ok_or("missing fixture source witness")?;
        assert_eq!(
            row["diagnosis"],
            "no_contextual_read_for_expected_assertion"
        );
        witness.candidates.push((
            Value::Null,
            SourceScore {
                source: 4,
                eligible: true,
                margin: Some(-4),
            },
        ));
        audit.contextual_sources = Some(vec![witness]);
        let row = source_witness(&gold, audit, None).ok_or("missing fixture source witness")?;
        assert_eq!(row["diagnosis"], "eligible_expected_source_lost_in_ranking");
        assert_eq!(row["margin"], -4);
        Ok(())
    }
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args_os().collect();
    let (sealed_gold, ordinary, captures, stored) = match args.as_slice() {
        [_, gold, ordinary, captures] => (gold, ordinary, Some(captures), None),
        [_, flag, gold, ordinary, stored] if flag == "--stored" => {
            (gold, ordinary, None, Some(stored))
        }
        _ => return Err("usage: d32-role-grade [--stored] SEALED_GOLD SEALED_D26_ORDINARY CAPTURE_ROOT_OR_STORED_AUDIT".into()),
    };
    let sealed_gold = fs::read(sealed_gold)?;
    if format!("{:x}", Sha256::digest(&sealed_gold)) != GOLD {
        return Err("sealed role gold SHA-256 mismatch".into());
    }
    let gold: Vec<Gold> = serde_json::from_slice(&sealed_gold)?;
    let mut results = Vec::new();
    if let Some(stored) = stored {
        let stored: Vec<StoredAudit> = serde_json::from_slice(&fs::read(stored)?)?;
        if stored.len() != gold.len() {
            return Err("stored trace does not cover the full panel".into());
        }
        for (row, stored) in gold.iter().zip(stored) {
            let (_, mut baseline) =
                read_client(&Path::new(ordinary).join(format!("{}.json", row.row)), row)?;
            if stored.row != row.row || baseline.paired.role_words != Some(stored.role_words) {
                return Err(format!(
                    "stored audit differs from ordinary receipt at row {}",
                    row.row
                )
                .into());
            }
            baseline.role_replay = Some(stored.into());
            results.push(grade(
                "stored-historical-inherited",
                row,
                &baseline,
                &baseline,
            )?);
        }
    }
    if let Some(captures) = captures {
        for arm in ARMS {
            check_arm_rows(&Path::new(captures).join(arm), &gold)?;
        }
        for row in &gold {
            let name = format!("{}.json", row.row);
            let (baseline_raw, baseline) = read_client(&Path::new(ordinary).join(&name), row)?;
            for arm in ARMS {
                let (raw, capture) = read_client(&Path::new(captures).join(arm).join(&name), row)?;
                if arm == ARMS[0] {
                    match_baseline(&raw, &baseline_raw, row.row)?;
                }
                results.push(grade(arm, row, &capture, &baseline)?);
            }
        }
    }
    println!("{}", serde_json::to_string_pretty(&results)?);
    Ok(())
}
