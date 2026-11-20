#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, env, error::Error, fs, path::Path};
use uor_r4_integer::{
    content_selector::Choice,
    discrete_d11::{
        ExactCopyState, ExactCopyStep, IntegerDiscreteModel, PairedCopyDecision, ROLE_COPY_STOPS,
    },
    paired_role::RoleSpan,
};
use uor_r4_tokenizer::ByteBpeTokenizer;

type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Span = [usize; 2];
type Picks = BTreeMap<(u32, u32), (String, Option<u32>)>;
const EOS: u32 = 1;
const MAX_TOKENS: usize = 16;
const NIL: u32 = u32::MAX;
const TOKENIZER: &str = "a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377";
const D26: &str = "b17352679c64a96bf657fad7640d4bb05ea3de0ecd08dadd13e9d2756e979bc4";
const D29: &str = "72f962ebc0d5c7ddba7e9b0f4643b3733c42181f5b35b0ae8620d11006c0936d";
const ARMS: [&str; 3] = ["nominal", "swap", "zero"];

#[derive(Deserialize)]
struct SourceRow {
    row: u32,
    reading: String,
    gold: Option<u32>,
    onsets: [Option<u32>; 2],
    fact_and_query: Option<[Span; 2]>,
    candidates: BTreeMap<u32, bool>,
    fence: Option<u32>,
    original_ids: Vec<u32>,
    plausible_writer_ids: BTreeMap<u32, Vec<u32>>,
    expected_reply: String,
}

#[derive(Deserialize, Serialize)]
struct RoleTargets {
    requested: Option<Span>,
    excluded: Option<Span>,
    unrequested: Option<Span>,
}

#[derive(Deserialize)]
struct RoleRow {
    row: u32,
    stratum: String,
    family: String,
    role_factors: Option<[usize; 3]>,
    query: Span,
    original_ids: Vec<u32>,
    role_targets: RoleTargets,
    assertion_heads: Vec<Span>,
    value_spans: Vec<Span>,
    gold_source: Option<u32>,
    expected_tape: Vec<u32>,
    expected_reply: String,
}

#[derive(Deserialize)]
struct Client {
    artifact_sha256: String,
    tokenizer_sha256: String,
    model_schema: String,
    prompt_ids: Vec<u32>,
    generated_ids: Vec<u32>,
    copy_step_words: Vec<[u32; 8]>,
    paired: ClientChoice,
    read_enabled: bool,
    protocol: String,
    text: String,
    stop: String,
}

#[derive(Deserialize)]
struct ClientChoice {
    choice: Choice,
    role_words: Option<[u32; 8]>,
    supporting_facts: u32,
    complete_values: u32,
    values_disagree: bool,
}

#[derive(Clone, Serialize)]
struct Replay {
    generated_ids: Vec<u32>,
    copy_step_words: Vec<[u32; 8]>,
}

impl Replay {
    fn copied_through_eos(&self) -> bool {
        self.generated_ids.last() == Some(&EOS)
            && self
                .copy_step_words
                .last()
                .is_some_and(|&words| ExactCopyStep::from_words(words).completed_span())
    }
}

fn read_picks(path: &Path) -> Result<Picks> {
    let mut entries = BTreeMap::new();
    for line in fs::read_to_string(path)?.lines() {
        let parts: Vec<_> = line.split('\t').collect();
        let [row, arm, reason, address] = parts.as_slice() else {
            return Err(format!("malformed GPU row {line:?}").into());
        };
        let source = if *address == "NA" {
            None
        } else {
            Some(address.parse()?)
        };
        if !matches!(*reason, "empty" | "nonpositive" | "tie" | "pick")
            || (*reason == "pick") != source.is_some()
            || entries
                .insert((row.parse()?, arm.parse()?), (reason.to_string(), source))
                .is_some()
        {
            return Err(format!("invalid or duplicate GPU row {line:?}").into());
        }
    }
    Ok(entries)
}

fn replay(model: &IntegerDiscreteModel, original: &[u32], source: Option<u32>) -> Result<Replay> {
    let context_limit = model.shape().context as usize;
    let mut context = original.to_vec();
    let mut cursor = ExactCopyState::new(source, original.len().try_into()?);
    let mut trajectory = Replay {
        generated_ids: Vec::new(),
        copy_step_words: Vec::new(),
    };
    for _ in 0..MAX_TOKENS {
        let trial = cursor
            .write(original, 0, ROLE_COPY_STOPS)?
            .finish_with_eos(true, EOS);
        let step = if trial.copied != 0 || trial.completed_span() {
            trial
        } else {
            let fallback = model.evaluate_prefix(&context)?.prediction;
            cursor
                .write(original, fallback, ROLE_COPY_STOPS)?
                .finish_with_eos(true, EOS)
        };
        cursor = step.state;
        trajectory.generated_ids.push(step.emitted);
        trajectory.copy_step_words.push(step.words());
        if step.emitted == EOS {
            break;
        }
        if context.len() == context_limit {
            context.remove(0);
        }
        context.push(step.emitted);
    }
    Ok(trajectory)
}

fn outcome(
    tokenizer: &ByteBpeTokenizer,
    expected: &[u32],
    source: Option<u32>,
    tape: &Replay,
) -> Value {
    let emitted = tape.generated_ids.as_slice();
    let visible = emitted.strip_suffix(&[EOS]);
    let first = (0..expected.len().max(emitted.len()))
        .find(|&i| expected.get(i) != emitted.get(i))
        .map(|i| json!({"offset": i, "expected_id": expected.get(i), "generated_id": emitted.get(i)}));
    json!({
        "source": source,
        "expected_ids": expected,
        "first_mismatch": first,
        "exact_through_eos": visible.is_some() && emitted == expected,
        "copied_through_eos": tape.copied_through_eos(),
        "stop": if visible.is_some() { "eos" } else { "max_tokens" },
        "text": tokenizer.decode(visible.unwrap_or(emitted)),
        "trace": tape,
    })
}

fn checked_client(
    path: &Path,
    model: &IntegerDiscreteModel,
    ids: &[u32],
    tokenizer: &ByteBpeTokenizer,
    decision: &PairedCopyDecision,
    trace: &Replay,
    enabled: bool,
) -> Result<()> {
    let client: Client = serde_json::from_slice(&fs::read(path)?)?;
    let emitted = trace.generated_ids.as_slice();
    let visible = emitted.strip_suffix(&[EOS]);
    if client.artifact_sha256 != model.artifact_sha256()
        || client.tokenizer_sha256 != TOKENIZER
        || client.model_schema != model.artifact_schema()
        || client.prompt_ids != ids
        || client.protocol != "uor-r4.literal-role-dialogue/2"
        || client.read_enabled != enabled
        || client.paired.choice != decision.choice
        || client.paired.role_words != decision.role_words
        || client.paired.supporting_facts != decision.supporting_facts
        || client.paired.complete_values != decision.complete_values
        || client.paired.values_disagree != decision.values_disagree
        || client.generated_ids != emitted
        || client.copy_step_words != trace.copy_step_words
        || client.text != tokenizer.decode(visible.unwrap_or(emitted))
        || client.stop
            != if visible.is_some() {
                "eos"
            } else {
                "max_tokens"
            }
    {
        return Err(format!(
            "ordinary capture disagrees with native role or exact cursor at {path:?}"
        )
        .into());
    }
    Ok(())
}

fn ordinary(
    directory: &Path,
    model: &IntegerDiscreteModel,
    row: u32,
    ids: &[u32],
    tokenizer: &ByteBpeTokenizer,
    expected: &[u32],
    disabled: Option<&Path>,
) -> Result<Value> {
    let decision = model
        .paired_copy_decision(ids, true)?
        .ok_or("missing paired decision")?;
    let roles = model.paired_roles_diagnostic(ids, 0..ids.len(), false)?;
    let (witness, assertions) = model.paired_copy_witness(ids, roles, None)?;
    if witness.choice != decision.choice
        || witness.role_words != decision.role_words
        || witness.supporting_facts != decision.supporting_facts
        || witness.complete_values != decision.complete_values
        || witness.values_disagree != decision.values_disagree
    {
        return Err(format!("ordinary role and binder witnesses differ on row {row}").into());
    }
    let tape = replay(model, ids, decision.choice.source)?;
    checked_client(
        &directory.join(format!("{row}.json")),
        model,
        ids,
        tokenizer,
        &decision,
        &tape,
        true,
    )?;
    let fallback = replay(model, ids, None)?;
    if let Some(disabled) = disabled {
        let unbound = model
            .paired_copy_decision(ids, false)?
            .ok_or("missing disabled decision")?;
        checked_client(
            &disabled.join(format!("{row}.json")),
            model,
            ids,
            tokenizer,
            &unbound,
            &fallback,
            false,
        )?;
    }
    let frozen = model.frozen_role_replay()?;
    let framing = frozen
        .requested_framing
        .ok_or("missing deployed requested framing")?;
    let eligibility = frozen.roles.trace_requested_occurrence(
        ids,
        framing.content(ids)?,
        0..ids.len(),
        frozen.delimiters,
        frozen.assertion_classes,
        frozen
            .requested_context
            .ok_or("missing deployed requested context")?,
    )?;
    Ok(json!({
        "actual_requested":[roles.requested.map(|r| r.start), roles.requested.map(|r| r.end)],
        "actual_excluded":[roles.excluded.map(|r| r.start), roles.excluded.map(|r| r.end)],
        "ordinary_words":decision.role_words,
        "choice":decision.choice,
        "supporting_facts":decision.supporting_facts,
        "complete_values":decision.complete_values,
        "values_disagree":decision.values_disagree,
        "assertion_witnesses":assertions,
        "role_eligibility":eligibility,
        "writer":outcome(tokenizer, expected, decision.choice.source, &tape),
        "read_disabled":outcome(tokenizer, expected, None, &fallback),
        "read_disabled_capture_checked":disabled.is_some(),
    }))
}

fn answer(tokenizer: &ByteBpeTokenizer, reply: &str) -> Vec<u32> {
    let mut result = tokenizer.encode(&format!(" {reply}"));
    result.push(EOS);
    result
}

fn source_rows(
    model: &IntegerDiscreteModel,
    tokenizer: &ByteBpeTokenizer,
    rows: &[SourceRow],
    device: &Path,
    ordinary_root: &Path,
    disabled_root: &Path,
) -> Result<Vec<Value>> {
    let mut devices = ARMS
        .map(|arm| read_picks(&device.join(format!("{arm}.results"))))
        .into_iter()
        .collect::<Result<Vec<_>>>()?;
    let mut report = Vec::new();
    for r in rows {
        let ids = &r.original_ids;
        let actual = ordinary(
            &ordinary_root.join("d26"),
            model,
            r.row,
            ids,
            tokenizer,
            &answer(tokenizer, &r.expected_reply),
            disabled_root.is_dir().then_some(disabled_root),
        )?;
        let expected = answer(tokenizer, &r.expected_reply);
        let mut device_replays = BTreeMap::new();
        for (index, picks) in devices.iter_mut().enumerate() {
            for arm in 0..4 {
                let (reason, source) = picks.remove(&(r.row, arm)).ok_or_else(|| {
                    format!(
                        "missing device decision {} arm {arm} for row {}",
                        ARMS[index], r.row
                    )
                })?;
                if source.is_some_and(|addr| !r.candidates.contains_key(&addr)) {
                    return Err(
                        format!("row {} device produced noncandidate {source:?}", r.row).into(),
                    );
                }
                if arm == 3 || index == 0 && arm == 1 {
                    let name = if arm == 1 { "local" } else { ARMS[index] };
                    let replay = replay(model, ids, source)?;
                    if source.is_some() && !replay.copied_through_eos() {
                        return Err(
                            format!("row {} {name} copy failed its original fence", r.row).into(),
                        );
                    }
                    device_replays.insert(
                        name,
                        json!({
                            "reason": reason,
                            "writer": outcome(tokenizer, &expected, source, &replay),
                        }),
                    );
                }
            }
        }
        let mut forced = BTreeMap::new();
        let mut decisions = None;
        if let Some([[head_start, head_end], [query_start, query_end]]) = r.fact_and_query {
            let fact = RoleSpan {
                start: head_start,
                end: head_end,
            };
            let query = RoleSpan {
                start: query_start,
                end: query_end,
            };
            let (decision, witnessed) = model.contextual_source_trace(ids, fact, query)?;
            let nominal = &device_replays["nominal"];
            if serde_json::to_value(decision.outcome)? != nominal["reason"]
                || serde_json::to_value(decision.source)? != nominal["writer"]["source"]
                || witnessed.len() != r.candidates.len()
                || witnessed.iter().any(|(candidate, score)| {
                    Some(candidate.narrow) != r.candidates.get(&score.source).copied()
                        || candidate.completed_source_range[0] != score.source
                        || Some(candidate.completed_source_range[1]) != r.fence
                })
            {
                return Err(format!(
                    "row {} loaded carrier does not match device nominal or candidate mask",
                    r.row
                )
                .into());
            }
            let roles = model.paired_roles_diagnostic(ids, 0..ids.len(), false)?;
            let (binder, facts) = model.paired_copy_witness(ids, roles, Some(query))?;
            decisions = Some(json!({"nominal":decision,"candidates":witnessed,
                "supplied_query_choice":binder.choice, "supplied_query_assertions":facts}));
        } else if device_replays
            .values()
            .any(|r| !r["writer"]["source"].is_null())
        {
            return Err("device selected an address without a supplied assertion".into());
        }
        for (lane, source) in ["early", "late"].into_iter().zip(r.onsets) {
            if let Some(source) = source {
                let forced_tape = replay(model, ids, Some(source))?;
                let oracle = r
                    .plausible_writer_ids
                    .get(&source)
                    .ok_or("absent forcing oracle")?;
                if !forced_tape.copied_through_eos()
                    || forced_tape.generated_ids != *oracle
                    || (r.gold == Some(source) && *oracle != expected)
                {
                    return Err(format!(
                        "source forcing or independent literal diverged on row {} {lane}",
                        r.row
                    )
                    .into());
                }
                forced.insert(
                    lane,
                    json!({
                        "own_tape":outcome(tokenizer, oracle, Some(source), &forced_tape),
                        "gold_tape":outcome(tokenizer, &expected, Some(source), &forced_tape),
                    }),
                );
            }
        }
        report.push(json!({
            "row":r.row,"stratum":if r.row >= 30051 { "d29_exposed" } else if r.gold.is_some() { "fresh" } else { "control" },
            "reading":r.reading,"gold_source":r.gold,
            "gold_reply":r.gold.map(|_| &r.expected_reply),
            "reference_refusal":r.gold.is_none().then_some(&r.expected_reply),
            "original_ids":ids,"fact_and_query":r.fact_and_query,"onsets":r.onsets,"fence":r.fence,
            "controlled":device_replays,"loaded_carrier":decisions,"forced":forced,"ordinary":actual,
        }));
    }
    if devices.iter().any(|d| !d.is_empty()) {
        return Err("device included unexpected rows".into());
    }
    Ok(report)
}

fn role_rows(
    model: &IntegerDiscreteModel,
    tokenizer: &ByteBpeTokenizer,
    rows: &[RoleRow],
    clients: &Path,
    disabled: Option<&Path>,
) -> Result<Vec<Value>> {
    let mut report = Vec::new();
    for r in rows {
        let ids = &r.original_ids;
        let expected = &r.expected_tape;
        let actual = ordinary(clients, model, r.row, ids, tokenizer, expected, disabled)?;
        let raw_words: Option<[u32; 8]> = serde_json::from_value(actual["ordinary_words"].clone())?;
        let endpoints = |index| -> Option<Span> {
            let words = raw_words?;
            (words[index] != NIL && words[index + 1] != NIL)
                .then_some([words[index] as usize, words[index + 1] as usize])
        };
        let requested = endpoints(0);
        let excluded = endpoints(2);
        let mut force = Vec::new();
        for (&head, &[value, fence]) in r.assertion_heads.iter().zip(&r.value_spans) {
            let source = value.try_into()?;
            let oracle = [&ids[value..fence], &[EOS]].concat();
            let tape = replay(model, ids, Some(source))?;
            if !tape.copied_through_eos()
                || tape.generated_ids != oracle
                || (Some(source) == r.gold_source && oracle != *expected)
            {
                return Err(format!("invalid forced role writer at row {}, {value}", r.row).into());
            }
            force.push(json!({
                "assertion_head":head,
                "forcing":if r.gold_source == Some(source) { "gold" } else { "opposing_or_incidental" },
                "own_tape":outcome(tokenizer, &oracle, Some(source), &tape),
                "gold_tape":outcome(tokenizer, expected, Some(source), &tape),
            }));
        }
        report.push(json!({
            "row":r.row,"stratum":r.stratum,"family":r.family,"role_factors":r.role_factors,
            "query":r.query,"original_ids":ids,"expected_reply":r.expected_reply,"expected_roles":r.role_targets,
            "actual_requested":requested,"actual_excluded":excluded,
            "exact_requested_endpoints":requested == r.role_targets.requested,
            "exact_excluded_endpoints":excluded == r.role_targets.excluded,
            "incidental_selected":r.role_targets.unrequested.is_some()
                && (requested == r.role_targets.unrequested || excluded == r.role_targets.unrequested),
            "gold_source":r.gold_source,"forced":force,"ordinary":actual,
        }));
    }
    Ok(report)
}

fn pair_readout(source: &[Value], path: &Path) -> Result<Vec<Value>> {
    let by_row: BTreeMap<_, _> = source.iter().map(|v| (v["row"].as_u64(), v)).collect();
    let mut pairs = Vec::new();
    for line in fs::read_to_string(path)?.lines() {
        let (row, donor) = line.split_once('\t').ok_or("invalid donor row")?;
        let (row, donor): (u64, u64) = (row.parse()?, donor.parse()?);
        if row >= donor {
            continue;
        }
        let a = by_row.get(&Some(row)).ok_or("missing target pair row")?;
        let b = by_row.get(&Some(donor)).ok_or("missing donor pair row")?;
        if a["stratum"] != b["stratum"] {
            return Err("paired strata differ".into());
        }
        let both_gold = |arm: &str| {
            [a, b]
                .into_iter()
                .all(|r| r["controlled"][arm]["writer"]["exact_through_eos"] == true)
        };
        let both_swapped = [(a, b), (b, a)].into_iter().all(|(r, d)| {
            let lane = d["reading"].as_str().unwrap_or_default();
            let swap = &r["controlled"]["swap"]["writer"];
            let forced = &r["forced"][lane]["own_tape"];
            !forced["source"].is_null()
                && swap["source"] == forced["source"]
                && swap["trace"]["generated_ids"] == forced["trace"]["generated_ids"]
                && swap["trace"]["copy_step_words"] == forced["trace"]["copy_step_words"]
                && swap["stop"] == "eos"
        });
        let ordinary = [a, b]
            .into_iter()
            .all(|r| r["ordinary"]["writer"]["exact_through_eos"] == true);
        pairs.push(json!({
            "row":row,"donor":donor,"stratum":a["stratum"],
            "both_local_correct":both_gold("local"),"both_nominal_correct":both_gold("nominal"),
            "both_zero_correct":both_gold("zero"),"both_swap_exact_donor_reply":both_swapped,
            "nominal_and_reciprocal_swap":both_gold("nominal") && both_swapped,
            "both_ordinary_correct":ordinary,
        }));
    }
    Ok(pairs)
}

fn main() -> Result<()> {
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    let [recurrence, baseline, tokenizer, panel, device, clients, disabled, output] =
        args.as_slice()
    else {
        return Err("usage: d30-writer RECURRENCE BASELINE TOKENIZER PANEL DEVICE_SCORE_DIR ORDINARY_ROOT D26_DISABLED NEW_OUTPUT".into());
    };
    let (panel, device, clients, disabled, output) = (
        Path::new(panel),
        Path::new(device),
        Path::new(clients),
        Path::new(disabled),
        Path::new(output),
    );
    let model = IntegerDiscreteModel::load(Path::new(recurrence))?;
    let base = IntegerDiscreteModel::load(Path::new(baseline))?;
    let bytes = fs::read(tokenizer)?;
    if model.artifact_sha256() != D26
        || base.artifact_sha256() != D29
        || model.tokenizer_sha256() != Some(TOKENIZER)
        || base.tokenizer_sha256() != Some(TOKENIZER)
        || format!("{:x}", Sha256::digest(&bytes)) != TOKENIZER
    {
        return Err("frozen artifact or tokenizer pin differs".into());
    }
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes).ok_or("invalid tokenizer")?;
    let source: Vec<SourceRow> =
        serde_json::from_slice(&fs::read(panel.join("native/held.grade.json"))?)?;
    let roles: Vec<RoleRow> = serde_json::from_slice(&fs::read(panel.join("role.gold.json"))?)?;
    let all_source = source_rows(&model, &tokenizer, &source, device, clients, disabled)?;
    let pairs = pair_readout(&all_source, &panel.join("native/held.donors.tsv"))?;
    let all_roles = role_rows(
        &model,
        &tokenizer,
        &roles,
        &clients.join("d26"),
        disabled.is_dir().then_some(disabled),
    )?;
    let baserole = role_rows(&base, &tokenizer, &roles, &clients.join("d29"), None)?;
    let basesource: Vec<_> = source.iter().map(|r| {
        let expected = answer(&tokenizer, &r.expected_reply);
        Ok(json!({"row":r.row,"gold_source":r.gold,
          "ordinary":ordinary(&clients.join("d29"), &base, r.row, &r.original_ids, &tokenizer, &expected, None)?}))
    }).collect::<Result<_>>()?;
    let record = json!({
        "schema":"d30.exact-writer/1",
        "d26_carrier_sha256":D26,"d29_baseline_sha256":D29,"tokenizer_sha256":TOKENIZER,
        "source":all_source,"pairs":pairs,"roles":all_roles,"d29_source":basesource,"d29_roles":baserole,
    });
    let file = fs::File::create_new(output)?;
    serde_json::to_writer_pretty(file, &record)?;
    Ok(())
}
