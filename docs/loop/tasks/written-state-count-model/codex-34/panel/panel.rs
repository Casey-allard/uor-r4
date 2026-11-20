#![forbid(unsafe_code)]

use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, env, error::Error, fs, path::Path};

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Span = [u32; 2];
type Difference = (usize, Option<u32>, Option<u32>);

const MODEL: &str = "b17352679c64a96bf657fad7640d4bb05ea3de0ecd08dadd13e9d2756e979bc4";
const TOKENIZER: &str = "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377";
const GOLD: &str = "220ec3c5fd0c839bed864c051458b30f95668cc1b7a26c725d646d189fdc8096";
const WIDTH: usize = 192;
const PACKET_MAGIC: u32 = 0x3144_3452;
const SITE_COUNT: usize = 4096;
const SITE_WORDS: usize = 5;
const CELL_WORDS: usize = 4;
const ROLE_WORDS: usize = 8;
const RECORD_WORDS: usize = 5;
const ROW_WORDS: usize = ROLE_WORDS + WIDTH * RECORD_WORDS;
const NIL: u32 = u32::MAX;
const ARMS: [&str; 2] = ["full-repeated", "full-detached"];

#[derive(Deserialize)]
struct Gold {
    row: u32,
    stratum: String,
    role_targets: Targets,
    assertion_heads: Vec<Span>,
    value_spans: Vec<Span>,
    original_ids: Vec<u32>,
    gold_source: Option<u32>,
    expected_tape: Vec<u32>,
}

#[derive(Deserialize)]
struct Targets {
    requested: Option<Span>,
    excluded: Option<Span>,
    unrequested: Option<Span>,
}

#[derive(Deserialize)]
struct Candidate {
    span: Span,
    #[serde(default)]
    detached: bool,
    repeated_head: Option<Span>,
    supporting_head: Option<Span>,
    requested_admitted: bool,
    excluded_admitted: bool,
    eligibility: String,
}

#[derive(Deserialize)]
struct Audit {
    content: Span,
    candidates: Vec<Candidate>,
    assertions: Vec<Value>,
    contextual_sources: Vec<Value>,
    detached: Option<Detached>,
}

#[derive(Deserialize)]
struct Detached {
    detached_assertion_heads: Vec<Span>,
}

#[derive(Deserialize)]
struct Capture {
    schema: String,
    context: usize,
    artifact_sha256: String,
    tokenizer_sha256: String,
    model_schema: String,
    contextual_source_arm: String,
    protocol: String,
    decoding: String,
    read_enabled: bool,
    prompt_ids: Vec<u32>,
    generated_ids: Vec<u32>,
    copy_step_words: Vec<[u32; 8]>,
    selected: Selection,
    paired: Choice,
    stop: String,
    role_replay: Audit,
}

#[derive(Deserialize)]
struct Selection {
    source: Option<u32>,
    fact: Option<u32>,
}

#[derive(Deserialize)]
struct Choice {
    role_words: [u32; ROLE_WORDS],
}

fn require(condition: bool, message: impl FnOnce() -> String) -> Result<()> {
    if !condition {
        return Err(message().into());
    }
    Ok(())
}

fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn prefix_sha(ids: &[u32]) -> String {
    let original_bytes: Vec<u8> = ids.iter().flat_map(|id| id.to_le_bytes()).collect();
    sha(&original_bytes)
}

fn read_u32(path: &Path) -> Result<Vec<u32>> {
    let bytes = fs::read(path)?;
    let words = bytes.chunks_exact(4);
    require(words.remainder().is_empty(), || {
        format!("non-word-aligned file {}", path.display())
    })?;
    words
        .map(|word| Ok(u32::from_le_bytes(word.try_into()?)))
        .collect()
}

fn original_rows() -> impl Iterator<Item = u32> {
    (30100..=30110).chain(30116..=30126)
}

fn read_gold(path: &Path) -> Result<Vec<Gold>> {
    let bytes = fs::read(path)?;
    require(sha(&bytes) == GOLD, || "D32 gold SHA-256 mismatch".into())?;
    let rows: Vec<Gold> = serde_json::from_slice(&bytes)?;
    require(rows.iter().map(|row| row.row).eq(original_rows()), || {
        "gold is not the original ordered 22-row panel".into()
    })?;
    Ok(rows)
}

fn read_capture(path: &Path, cold: &Path, gold: &Gold) -> Result<(Value, Capture)> {
    let primary = fs::read(path)?;
    require(primary == fs::read(cold)?, || {
        format!("independent cold client differs at {}", path.display())
    })?;
    let raw: Value = serde_json::from_slice(&primary)?;
    let capture: Capture = serde_json::from_value(raw.clone())?;
    require(
        capture.schema == "uor-r4.uor-chat-discrete-text/1"
            && capture.context == WIDTH
            && capture.artifact_sha256 == MODEL
            && capture.tokenizer_sha256 == TOKENIZER
            && capture.model_schema == "uor-r4.lut-discrete-contextual-source/2"
            && capture.contextual_source_arm == "wide_recurrence"
            && capture.protocol == "uor-r4.literal-role-dialogue/2"
            && capture.decoding
                == "frozen D21 role binding, contextual source agreement and exact-copy EOS"
            && capture.read_enabled
            && capture.prompt_ids == gold.original_ids
            && capture.copy_step_words.len() == capture.generated_ids.len(),
        || {
            format!(
                "ordinary state, prompt, policy or writer mismatch at {}",
                path.display()
            )
        },
    )?;
    Ok((raw, capture))
}

fn check_rows(root: &Path, gold: &[Gold]) -> Result<()> {
    let expected: BTreeSet<_> = gold.iter().map(|row| format!("{}.json", row.row)).collect();
    let mut actual = BTreeSet::new();
    for file in fs::read_dir(root)? {
        let file = file?;
        require(file.file_type()?.is_file(), || {
            format!("unexpected entry at {}", file.path().display())
        })?;
        actual.insert(
            file.file_name()
                .into_string()
                .map_err(|_| "non-UTF-8 filename")?,
        );
    }
    require(actual == expected, || {
        format!("missing or extra rows in {}", root.display())
    })
}

fn grade_endpoint(audit: &Audit, target: Option<Span>, selected: Option<Span>) -> Option<Value> {
    let span = target?;
    let candidate = audit.candidates.iter().find(|c| c.span == span);
    Some(json!({
        "span": span,
        "present": candidate.is_some(),
        "detached": candidate.is_some_and(|c| c.detached),
        "first_exact_earlier_head": candidate.and_then(|c| c.repeated_head),
        "supporting_head": candidate.and_then(|c| c.supporting_head),
        "requested_admitted": candidate.is_some_and(|c| c.requested_admitted),
        "excluded_admitted": candidate.is_some_and(|c| c.excluded_admitted),
        "eligibility": candidate.map(|c| &c.eligibility),
        "assigned": selected == Some(span),
    }))
}

fn first_error(expected: &[u32], actual: &[u32]) -> Option<Difference> {
    (0..expected.len().max(actual.len())).find_map(|offset| {
        let left = expected.get(offset).copied();
        let right = actual.get(offset).copied();
        (left != right).then_some((offset, left, right))
    })
}

fn expected_fact(gold: &Gold) -> Option<Span> {
    let source = gold.gold_source?;
    gold.assertion_heads
        .iter()
        .zip(&gold.value_spans)
        .find_map(|(&fact, &[start, end])| (start..end).contains(&source).then_some(fact))
}

fn grade_assertion(gold: &Gold, capture: &Capture) -> Option<Value> {
    let fact = expected_fact(gold)?;
    let audit = &capture.role_replay;
    let bound = audit
        .assertions
        .iter()
        .any(|entry| entry["fact_span"] == json!(fact));
    let context = audit
        .contextual_sources
        .iter()
        .find(|entry| entry["fact_span"] == json!(fact));
    let expected = gold.gold_source?;
    let value = context.and_then(|entry| {
        entry["candidates"]
            .as_array()?
            .iter()
            .find_map(|candidate| (candidate[1]["source"] == expected).then_some(&candidate[1]))
    });
    let eligible = value.is_some_and(|score| score["eligible"] == true);
    let diagnosis = match (bound, context.is_some(), value.is_some(), eligible) {
        _ if capture.selected.source == Some(expected) => "selected_expected_source",
        (false, _, _, _) => "expected_assertion_not_bound",
        (_, false, _, _) => "no_contextual_read_for_expected_assertion",
        (_, _, false, _) => "expected_source_not_enumerated",
        (_, _, _, false) => "expected_source_masked",
        _ if context.is_some_and(|entry| entry["selection"]["source"] == expected) => {
            "expected_local_pick_lost_after_ranking"
        }
        _ => "eligible_expected_source_lost_in_ranking",
    };
    Some(json!({
        "expected_fact_span": fact,
        "raw_detached_head": audit.detached.as_ref().is_some_and(|d| d.detached_assertion_heads.contains(&fact)),
        "bound": bound,
        "enumerated": value.is_some(),
        "eligible": eligible,
        "margin": value.map(|score| &score["margin"]),
        "local_pick": context.map(|entry| &entry["selection"]),
        "diagnosis": diagnosis,
    }))
}

fn grade(gold: &Gold, capture: &Capture) -> Value {
    let words = capture.paired.role_words;
    let requested = (words[0] != NIL).then_some([words[0], words[1]]);
    let excluded = (words[2] != NIL).then_some([words[2], words[3]]);
    let mismatch = gold
        .gold_source
        .and(first_error(&gold.expected_tape, &capture.generated_ids));
    let eos = capture.stop == "eos" && capture.generated_ids.last() == Some(&1);
    let released = eos
        && capture
            .copy_step_words
            .last()
            .is_some_and(|words| words[2] > words[1] && words[4] == 1 && words[7] == 1);
    let audit = &capture.role_replay;
    json!({
        "role_words": words,
        "requested": grade_endpoint(audit, gold.role_targets.requested, requested),
        "excluded": grade_endpoint(audit, gold.role_targets.excluded, excluded),
        "incidental": grade_endpoint(audit, gold.role_targets.unrequested, requested),
        "incidental_chosen": gold.role_targets.unrequested.is_some_and(|span| Some(span) == requested || Some(span) == excluded),
        "requested_actual": requested,
        "excluded_actual": excluded,
        "requested_correct": requested == gold.role_targets.requested,
        "excluded_correct": excluded == gold.role_targets.excluded,
        "joint_correct": requested == gold.role_targets.requested && excluded == gold.role_targets.excluded,
        "assertion": grade_assertion(gold, capture),
        "selected_fact": capture.selected.fact,
        "selected_source": capture.selected.source,
        "correct_source": gold.gold_source.map(|id| capture.selected.source == Some(id)),
        "unsupported_copy": gold.gold_source.is_none() && capture.selected.source.is_some(),
        "copy_released_at_eos": released,
        "exact_through_eos": gold.gold_source.map(|_| mismatch.is_none() && eos),
        "first_error": mismatch,
        "generated_ids": capture.generated_ids,
        "copy_step_words": capture.copy_step_words,
        "stop": capture.stop,
    })
}

fn check_baseline(raw: &Value, archived: &Path, row: u32) -> Result<()> {
    let old: Value = serde_json::from_slice(&fs::read(archived.join(format!("{row}.json")))?)?;
    for field in [
        "schema",
        "artifact_sha256",
        "tokenizer_sha256",
        "model_schema",
        "contextual_source_arm",
        "protocol",
        "read_enabled",
        "decoding",
        "prompt_ids",
        "selected",
        "paired",
        "role_replay",
        "copy_step_words",
        "generated_ids",
        "stop",
        "text",
    ] {
        require(
            raw.get(field).is_some() && raw.get(field) == old.get(field),
            || format!("D32 full-repeated baseline differs on row {row} field {field}"),
        )?;
    }
    Ok(())
}

fn check_device(capture: &Capture, words: &[u32], row: u32, arm: &str) -> Result<()> {
    require(words[..ROLE_WORDS] == capture.paired.role_words, || {
        format!("role decision words differ on row {row} arm {arm}")
    })?;
    let audit = &capture.role_replay;
    let mut expected = [[NIL, NIL, NIL, NIL, 0]; WIDTH];
    for candidate in audit.candidates.iter().filter(|c| c.detached) {
        let [start, end] = candidate.span;
        let words = expected
            .get_mut(start as usize)
            .ok_or("detached start outside prefix width")?;
        require(
            words[0] == NIL && start < end && end <= audit.content[1],
            || format!("invalid detached candidate span on row {row} arm {arm}"),
        )?;
        let witness = candidate.repeated_head.unwrap_or([NIL, NIL]);
        words[0] = end;
        words[1..3].copy_from_slice(&witness);
        words[4] =
            u32::from(candidate.requested_admitted) | (u32::from(candidate.excluded_admitted) << 1);
    }
    if arm == ARMS[1] {
        let trace = audit
            .detached
            .as_ref()
            .ok_or("missing detached factual trace")?;
        for &[start, end] in &trace.detached_assertion_heads {
            let words = expected
                .get_mut(start as usize)
                .ok_or("detached fact outside prefix width")?;
            require(
                words[3] == NIL && start < end && end <= audit.content[1],
                || format!("invalid detached assertion span on row {row} arm {arm}"),
            )?;
            words[3] = end;
        }
    }
    compare_detached(&words[ROLE_WORDS..], &expected, row, arm)
}

fn compare_detached(
    words: &[u32],
    expected: &[[u32; RECORD_WORDS]; WIDTH],
    row: u32,
    arm: &str,
) -> Result<()> {
    require(words.len() == WIDTH * RECORD_WORDS, || {
        format!("detached record shape differs on row {row} arm {arm}")
    })?;
    for (position, (actual, expected)) in words.chunks_exact(RECORD_WORDS).zip(expected).enumerate()
    {
        require(actual == expected, || {
            format!("detached records differ on row {row} arm {arm} at position {position}: {actual:?} vs {expected:?}")
        })?;
    }
    Ok(())
}

fn recovered_endpoint(gold: &Gold, before: &Value, after: &Value) -> Option<bool> {
    if before["present"] != false {
        return None;
    }
    let covered = after["detached"] == true
        && serde_json::from_value::<Span>(after["span"].clone())
            .ok()
            .zip(serde_json::from_value::<Span>(after["first_exact_earlier_head"].clone()).ok())
            .is_some_and(|([start, end], [fact_start, fact_end])| {
                fact_end <= start
                    && gold.assertion_heads.contains(&[fact_start, fact_end])
                    && gold
                        .original_ids
                        .get(start as usize..end as usize)
                        .is_some_and(|query| {
                            Some(query)
                                == gold
                                    .original_ids
                                    .get(fact_start as usize..fact_end as usize)
                        })
            });
    Some(covered)
}

fn summarize(rows: &[Value], arm: &str, stratum: &str) -> Value {
    let selected: Vec<_> = rows
        .iter()
        .filter(|row| stratum == "all" || row["stratum"] == stratum)
        .collect();
    let count = |pointer: &str| {
        selected
            .iter()
            .filter(|row| row[arm].pointer(pointer).is_some_and(|value| value == true))
            .count()
    };
    let support = selected
        .iter()
        .filter(|row| !row["gold_source"].is_null())
        .count();
    json!({
        "rows": selected.len(),
        "supported_rows": support,
        "unsupported_rows": selected.len() - support,
        "requested_present": count("/requested/present"),
        "excluded_present": count("/excluded/present"),
        "requested_admitted": count("/requested/requested_admitted"),
        "excluded_admitted": count("/excluded/excluded_admitted"),
        "requested_correct": count("/requested_correct"),
        "excluded_correct": count("/excluded_correct"),
        "joint_correct": count("/joint_correct"),
        "expected_assertion_bound": count("/assertion/bound"),
        "correct_source": count("/correct_source"),
        "exact_through_eos": count("/exact_through_eos"),
        "unsupported_copy": count("/unsupported_copy"),
        "incidental_chosen": count("/incidental_chosen"),
    })
}

fn run() -> Result<Value> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let [gold, archived, primary, cold, packets, device, device_cold] = args.as_slice() else {
        return Err("usage: d34-role-panel GOLD D32_FULL_REPEATED ORDINARY_PRIMARY ORDINARY_COLD PACKETS CUDA_PRIMARY CUDA_COLD".into());
    };
    let gold = read_gold(Path::new(gold))?;
    let (archived, primary, cold) = (Path::new(archived), Path::new(primary), Path::new(cold));
    let (packets, device, device_cold) = (
        Path::new(packets),
        Path::new(device),
        Path::new(device_cold),
    );
    let index = fs::read_to_string(packets.join("index.tsv"))?;
    let expected_index: String = gold
        .iter()
        .map(|row| format!("{}\t{}\n", row.row, prefix_sha(&row.original_ids)))
        .collect();
    require(index == expected_index, || {
        "device index differs from the sealed panel".into()
    })?;
    let mut results = Vec::new();
    let inputs = [
        read_u32(&packets.join("mode-7.bin"))?,
        read_u32(&packets.join("mode-8.bin"))?,
    ];
    require(
        inputs[0].len() == inputs[1].len()
            && inputs[0].len() >= 4
            && inputs[0][0] == inputs[1][0]
            && inputs[0][1] == 7
            && inputs[1][1] == 8
            && inputs[0][2] as usize == gold.len()
            && inputs[0][2..] == inputs[1][2..],
        || "device packets differ beyond the selected mode".into(),
    )?;
    let packed_width = WIDTH + 3;
    let (packet_state, packed_rows) = inputs[0]
        .split_at_checked(
            inputs[0]
                .len()
                .checked_sub(gold.len() * packed_width)
                .ok_or("truncated input")?,
        )
        .ok_or("incomplete input packet")?;
    let [magic, _, _, cells]: [u32; 4] = packet_state
        .get(..4)
        .ok_or("missing D34 packet header")?
        .try_into()?;
    require(
        magic == PACKET_MAGIC
            && cells > 0
            && packet_state.len() == 4 + SITE_COUNT * SITE_WORDS + cells as usize * CELL_WORDS,
        || "invalid D34 native-state packet dimensions".into(),
    )?;
    for (arm_no, arm) in ARMS.iter().enumerate() {
        check_rows(&primary.join(arm), &gold)?;
        check_rows(&cold.join(arm), &gold)?;
        let filename = format!("mode-{}.out", 7 + arm_no);
        let words = read_u32(&device.join(&filename))?;
        let cold_words = read_u32(&device_cold.join(&filename))?;
        require(
            words.len() == gold.len() * ROW_WORDS && words == cold_words,
            || format!("missing rows or cold device mismatch in {arm}"),
        )?;
        for (position, row) in gold.iter().enumerate() {
            let filename = format!("{}.json", row.row);
            let (raw, capture) = read_capture(
                &primary.join(arm).join(&filename),
                &cold.join(arm).join(&filename),
                row,
            )?;
            let start = position * packed_width;
            let packed = &packed_rows[start..start + packed_width];
            let len = capture.prompt_ids.len();
            require(
                len <= WIDTH
                    && packed[..len] == capture.prompt_ids
                    && packed[len..WIDTH].iter().all(|&word| word == 0)
                    && packed[WIDTH] as usize == len
                    && packed[(WIDTH + 1)..] == capture.role_replay.content,
                || {
                    format!(
                        "packed original prefix or user frame differs on row {}",
                        row.row
                    )
                },
            )?;
            let start = position * ROW_WORDS;
            check_device(&capture, &words[start..start + ROW_WORDS], row.row, arm)?;
            let grade = grade(row, &capture);
            if arm_no == 0 {
                check_baseline(&raw, archived, row.row)?;
                results.push(json!({
                    "row": row.row, "stratum": row.stratum,
                    "original_prefix_sha256": prefix_sha(&row.original_ids), "content": capture.role_replay.content,
                    "expected_tape": row.expected_tape, "gold_source": row.gold_source,
                    "full-repeated": grade,
                }));
            } else {
                let result = &mut results[position];
                let historical: Vec<u32> =
                    serde_json::from_value(result[ARMS[0]]["generated_ids"].clone())?;
                result["first_output_change"] =
                    json!(first_error(&historical, &capture.generated_ids));
                result["first_error_moved"] =
                    json!(result[ARMS[0]]["first_error"] != grade["first_error"]);
                result["first_error_offset_moved"] =
                    json!(result[ARMS[0]]["first_error"][0] != grade["first_error"][0]);
                result["exact_reply_changed"] =
                    json!(result[ARMS[0]]["exact_through_eos"] != grade["exact_through_eos"]);
                result["recovered_missing_requested_with_gold_witness"] = json!(
                    recovered_endpoint(row, &result[ARMS[0]]["requested"], &grade["requested"])
                );
                result["recovered_missing_excluded_with_gold_witness"] = json!(recovered_endpoint(
                    row,
                    &result[ARMS[0]]["excluded"],
                    &grade["excluded"]
                ));
                result[*arm] = grade;
            }
        }
    }
    let mut summary = Vec::new();
    let strata: BTreeSet<_> = gold
        .iter()
        .map(|row| row.stratum.as_str())
        .chain(["all"])
        .collect();
    for arm in ARMS {
        for stratum in &strata {
            summary.push(json!({ "arm": arm, "stratum": stratum, "counts": summarize(&results, arm, stratum) }));
        }
    }
    let crossed = results.iter().filter(|row| row["stratum"] == "crossed");
    let misses: Vec<_> = crossed
        .flat_map(|row| {
            [
                &row["recovered_missing_requested_with_gold_witness"],
                &row["recovered_missing_excluded_with_gold_witness"],
            ]
        })
        .filter_map(Value::as_bool)
        .collect();
    let movement =
        |condition: fn(&Value) -> bool| results.iter().filter(|row| condition(row)).count();
    Ok(json!({
        "schema": "uor-r4.d34.exposed-role-join/1",
        "state_packet_words": packet_state.len(),
        "compared_roles_per_launch": gold.len() * ARMS.len() * ROLE_WORDS,
        "compared_detached_words_per_launch": gold.len() * ARMS.len() * WIDTH * RECORD_WORDS,
        "summary": summary,
        "paired_movement": {
            "crossed_originally_missing_designated_endpoints": misses.len(),
            "crossed_missing_endpoints_recovered_with_gold_witness": misses.iter().filter(|&&pass| pass).count(),
            "first_error_offset_moved": movement(|row| row["first_error_offset_moved"] == true),
            "first_error_ids_or_offset_moved": movement(|row| row["first_error_moved"] == true),
            "wrong_to_exact": movement(|row| row[ARMS[0]]["exact_through_eos"] == false && row[ARMS[1]]["exact_through_eos"] == true),
            "exact_to_wrong": movement(|row| row[ARMS[0]]["exact_through_eos"] == true && row[ARMS[1]]["exact_through_eos"] == false),
            "wrong_to_wrong_with_different_tape": movement(|row| row[ARMS[0]]["exact_through_eos"] == false && row[ARMS[1]]["exact_through_eos"] == false && !row["first_output_change"].is_null()),
        },
        "rows": results
    }))
}

fn main() -> Result<()> {
    println!("{}", serde_json::to_string_pretty(&run()?)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn original_id_difference_preserves_absent_words_and_offset_zero() {
        assert_eq!(
            first_error(&[223, 51, 1], &[51, 1]),
            Some((0, Some(223), Some(51)))
        );
        assert_eq!(
            first_error(&[223, 51, 1], &[223, 51]),
            Some((2, Some(1), None))
        );
        assert_eq!(first_error(&[223, 51, 1], &[223, 51, 1]), None);
    }

    #[test]
    fn one_corrupted_factual_witness_endpoint_fails_the_join() -> Result<()> {
        let mut expected = [[NIL, NIL, NIL, NIL, 0]; WIDTH];
        expected[2] = [5, NIL, NIL, 5, 0];
        expected[7] = [10, 2, 5, NIL, 1];
        let mut tape: Vec<u32> = expected.iter().flatten().copied().collect();
        compare_detached(&tape, &expected, 0, ARMS[1])?;
        tape[7 * RECORD_WORDS + 2] = 4;
        let error = compare_detached(&tape, &expected, 0, ARMS[1]).unwrap_err();
        assert!(error.to_string().contains("at position 7"));
        Ok(())
    }
}
