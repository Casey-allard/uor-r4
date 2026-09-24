//! Loaded scoped-memory -> native lexical generation integration gate.
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use uor_r4_core::native_geometric::learner::addressed_lexical_bridge::{
    generate, AddressedRequest, ReadStatus,
};
use uor_r4_core::native_geometric::learner::scoped_memory::{HistoryView, Memory, Update};
use uor_r4_core::native_geometric::learner::state_lexical::SlFacts;
use uor_r4_core::native_geometric::learner::tl_execution::Execution;
use uor_r4_core::native_geometric::learner::transferable_lexical::{
    state_digest, TlAction, TlModel,
};
use uor_r4_core::report_output::{claim, seal, verify};
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;

const MODEL_SHA: &str = "69e8b88b41bb09d9149be1ac7e83e5e50db2974f470dce05405a55cc1f0bfb7e";
const TOKENIZER_SHA: &str = "a7ac75b68aa997fe7cc338d25d843157f2dc9d4265fb2f958ee779bb04828d6f";
const SCOPE: &[u8] = b"bridge/demo";
const RELATION: u8 = 1;
const MAX_NEW: usize = 8;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Case {
    name: String,
    before: Memory,
    after: Memory,
    alternative: Memory,
    unrelated: Memory,
    evicting: Memory,
    view_before: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Panel {
    cases: Vec<Case>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Row {
    case: String,
    arm: String,
    query_scope: Vec<u8>,
    query_entity: Vec<u8>,
    view: u64,
    status: String,
    record_id: Option<u64>,
    source_id: Option<u64>,
    selected_commit: Option<u64>,
    selected_payload: Option<Vec<u32>>,
    chain_len: usize,
    tokens: Vec<u32>,
    actions: Vec<String>,
    text: String,
    stopped: bool,
    state_digest: Option<u64>,
    compiled_parity: Option<bool>,
}

fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn read_bound(path: &Path, expected: &str) -> Result<Vec<u8>, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let actual = sha(&bytes);
    if actual != expected {
        return Err(format!("{} SHA-256 mismatch: {actual}", path.display()));
    }
    Ok(bytes)
}

fn one_token(tokenizer: &HfBpeTokenizer, text: &str) -> Result<Vec<u32>, String> {
    let ids = tokenizer.encode(text);
    if ids.len() != 1 || tokenizer.decode(&ids) != text {
        return Err(format!("declared value is not one exact token: {text:?}"));
    }
    Ok(ids)
}

fn build_case(
    tokenizer: &HfBpeTokenizer,
    name: &str,
    old: &str,
    changed: &str,
    alternative: &str,
    lineage: u64,
) -> Result<Case, String> {
    let old_ids = one_token(tokenizer, old)?;
    let changed_ids = one_token(tokenizer, changed)?;
    let alternative_ids = one_token(tokenizer, alternative)?;
    let mut before = Memory::new(lineage, 8);
    before
        .write(
            SCOPE,
            name.as_bytes(),
            RELATION,
            old.as_bytes(),
            &old_ids,
            101,
            Update::Assert,
            false,
            0,
        )
        .map_err(|e| e.to_string())?;
    let view_before = before.commit;
    let correct = |value: &str, ids: &[u32]| -> Result<Memory, String> {
        let mut memory = before.clone();
        memory
            .write(
                SCOPE,
                name.as_bytes(),
                RELATION,
                value.as_bytes(),
                ids,
                102,
                Update::Correct,
                false,
                0,
            )
            .map_err(|e| e.to_string())?;
        Ok(memory)
    };
    let after = correct(changed, &changed_ids)?;
    let alternate = correct(alternative, &alternative_ids)?;
    let mut unrelated = after.clone();
    unrelated
        .write(
            SCOPE,
            b"unrelated",
            RELATION,
            b" red",
            &one_token(tokenizer, " red")?,
            103,
            Update::Assert,
            false,
            0,
        )
        .map_err(|e| e.to_string())?;
    let mut evicting = Memory::new(lineage + 100, 1);
    evicting
        .write(
            SCOPE,
            name.as_bytes(),
            RELATION,
            old.as_bytes(),
            &old_ids,
            101,
            Update::Assert,
            false,
            0,
        )
        .map_err(|e| e.to_string())?;
    evicting
        .write(
            SCOPE,
            name.as_bytes(),
            RELATION,
            changed.as_bytes(),
            &changed_ids,
            102,
            Update::Correct,
            false,
            0,
        )
        .map_err(|e| e.to_string())?;
    Ok(Case {
        name: name.into(),
        before,
        after,
        alternative: alternate,
        unrelated,
        evicting,
        view_before,
    })
}

fn build_panel(tokenizer: &HfBpeTokenizer) -> Result<Panel, String> {
    Ok(Panel {
        cases: vec![
            build_case(tokenizer, "harbor", " red", " green", " north", 9101)?,
            build_case(tokenizer, "station", " north", " summer", " red", 9201)?,
        ],
    })
}

#[allow(clippy::too_many_arguments)]
fn row(
    model: &TlModel,
    compiled: &Execution,
    tokenizer: &HfBpeTokenizer,
    case: &Case,
    arm: &str,
    memory: &Memory,
    view: u64,
    history: HistoryView,
    scope: &[u8],
    entity: &[u8],
    enabled: bool,
) -> Result<Row, String> {
    memory.validate().map_err(|e| e.to_string())?;
    let request = AddressedRequest {
        scope,
        entity,
        relation: RELATION,
        view,
        history,
        observed: &[],
        max_new: MAX_NEW,
        read_enabled: enabled,
    };
    let result = generate(model, memory, request).map_err(|e| e.to_string())?;
    let (tokens, actions, stopped, state, parity) = if let Some(rollout) = &result.rollout {
        let record = result
            .record_id
            .and_then(|id| memory.record_ref(id))
            .ok_or("generated without selected record")?;
        let prior = memory.record_ref(record.predecessor);
        let facts = SlFacts {
            history: match history {
                HistoryView::Current => 0,
                HistoryView::PreviousAssertion => 1,
                HistoryView::PreviousDistinctValue => 2,
                HistoryView::Initial => 3,
            },
            committed: prior.is_some(),
            prior_differs: prior.is_some_and(|p| p.value != record.value),
            ..SlFacts::default()
        };
        let owned = result
            .selected_payload
            .as_deref()
            .ok_or("generated without owned payload")?;
        let mut work = compiled.workspace();
        let mut compiled_tokens = [0u32; MAX_NEW];
        let mut compiled_actions = [TlAction::Stop; MAX_NEW];
        let executed = compiled
            .run(
                owned,
                &[],
                facts,
                &[],
                owned,
                false,
                false,
                &mut work,
                &mut compiled_tokens,
                &mut compiled_actions,
            )
            .map_err(|e| format!("compiled run: {e}"))?;
        let parity = compiled_tokens[..executed.tokens] == rollout.tokens
            && compiled_actions[..executed.actions] == rollout.actions
            && executed.stopped == rollout.stopped
            && state_digest(&work.state) == rollout.state_digest;
        (
            rollout.tokens.clone(),
            rollout.actions.iter().map(|a| format!("{a:?}")).collect(),
            rollout.stopped,
            Some(rollout.state_digest),
            Some(parity),
        )
    } else {
        if result.record_id.is_some() || result.selected_payload.is_some() {
            return Err("abstention retained a selected record".into());
        }
        (Vec::new(), Vec::new(), false, None, None)
    };
    let status = match result.status {
        ReadStatus::Found => "Found",
        ReadStatus::Disabled => "Disabled",
        ReadStatus::Absent => "Absent",
        ReadStatus::NoHistory => "NoHistory",
        ReadStatus::Evicted => "Evicted",
    };
    Ok(Row {
        case: case.name.clone(),
        arm: arm.into(),
        query_scope: scope.to_vec(),
        query_entity: entity.to_vec(),
        view,
        status: status.into(),
        record_id: result.record_id,
        source_id: result.source_id,
        selected_commit: result.selected_commit,
        selected_payload: result.selected_payload,
        chain_len: result.chain_len,
        text: tokenizer.decode(&tokens),
        tokens,
        actions,
        stopped,
        state_digest: state,
        compiled_parity: parity,
    })
}

fn evaluate(
    model: &TlModel,
    tokenizer: &HfBpeTokenizer,
    panel: &Panel,
) -> Result<Vec<Row>, String> {
    let compiled = Execution::compile(model)?;
    let mut rows = Vec::new();
    for case in &panel.cases {
        let reload = |memory: &Memory| -> Result<Memory, String> {
            let bytes = memory.to_bytes().map_err(|e| e.to_string())?;
            Memory::from_bytes(&bytes).map_err(|e| e.to_string())
        };
        let before = reload(&case.before)?;
        let after = reload(&case.after)?;
        let alternative = reload(&case.alternative)?;
        let unrelated = reload(&case.unrelated)?;
        let evicting = reload(&case.evicting)?;
        let entity = case.name.as_bytes();
        for (arm, memory, view, history, key, enabled) in [
            (
                "before_enabled",
                &before,
                before.commit,
                HistoryView::Current,
                entity,
                true,
            ),
            (
                "before_disabled",
                &before,
                before.commit,
                HistoryView::Current,
                entity,
                false,
            ),
            (
                "before_no_history",
                &before,
                before.commit,
                HistoryView::PreviousAssertion,
                entity,
                true,
            ),
            (
                "after_enabled",
                &after,
                after.commit,
                HistoryView::Current,
                entity,
                true,
            ),
            (
                "after_disabled",
                &after,
                after.commit,
                HistoryView::Current,
                entity,
                false,
            ),
            (
                "alternative_enabled",
                &alternative,
                alternative.commit,
                HistoryView::Current,
                entity,
                true,
            ),
            (
                "after_pinned",
                &after,
                case.view_before,
                HistoryView::Current,
                entity,
                true,
            ),
            (
                "after_previous",
                &after,
                after.commit,
                HistoryView::PreviousAssertion,
                entity,
                true,
            ),
            (
                "unrelated_enabled",
                &unrelated,
                unrelated.commit,
                HistoryView::Current,
                entity,
                true,
            ),
            (
                "absent_enabled",
                &after,
                after.commit,
                HistoryView::Current,
                &b"missing"[..],
                true,
            ),
            (
                "evicted_previous",
                &evicting,
                evicting.commit,
                HistoryView::PreviousAssertion,
                entity,
                true,
            ),
        ] {
            rows.push(row(
                model, &compiled, tokenizer, case, arm, memory, view, history, SCOPE, key, enabled,
            )?);
        }
        rows.push(row(
            model,
            &compiled,
            tokenizer,
            case,
            "cross_scope",
            &after,
            after.commit,
            HistoryView::Current,
            b"bridge/other",
            entity,
            true,
        )?);
    }
    Ok(rows)
}

fn named<'a>(rows: &'a [Row], case: &str, arm: &str) -> Result<&'a Row, String> {
    rows.iter()
        .find(|row| row.case == case && row.arm == arm)
        .ok_or_else(|| format!("missing {case}/{arm}"))
}

fn gates(
    rows: &[Row],
    cases: &[Case],
    reload_equal: Option<bool>,
) -> Result<serde_json::Value, String> {
    let mut source_selection = true;
    let mut disabled_invariant = true;
    let mut unrelated_invariant = true;
    let mut absence_safe = true;
    let mut payload_changed = false;
    let mut changed_pairs = Vec::new();
    for case in cases {
        let before = named(rows, &case.name, "before_enabled")?;
        let after = named(rows, &case.name, "after_enabled")?;
        let alternative = named(rows, &case.name, "alternative_enabled")?;
        let pinned = named(rows, &case.name, "after_pinned")?;
        let previous = named(rows, &case.name, "after_previous")?;
        let unrelated = named(rows, &case.name, "unrelated_enabled")?;
        let absent = named(rows, &case.name, "absent_enabled")?;
        let evicted = named(rows, &case.name, "evicted_previous")?;
        let no_history = named(rows, &case.name, "before_no_history")?;
        let cross_scope = named(rows, &case.name, "cross_scope")?;
        let disabled_before = named(rows, &case.name, "before_disabled")?;
        let disabled_after = named(rows, &case.name, "after_disabled")?;
        source_selection &= before.status == "Found"
            && after.status == "Found"
            && alternative.status == "Found"
            && before.source_id == Some(101)
            && after.source_id == Some(102)
            && alternative.source_id == after.source_id
            && alternative.record_id == after.record_id
            && alternative.selected_commit == after.selected_commit
            && before.selected_payload != after.selected_payload
            && after.selected_payload != alternative.selected_payload
            && pinned.selected_payload == before.selected_payload
            && previous.selected_payload == before.selected_payload;
        disabled_invariant &= disabled_before.status == "Disabled"
            && disabled_after.status == "Disabled"
            && disabled_before.tokens.is_empty()
            && disabled_after.tokens.is_empty()
            && disabled_before.tokens == disabled_after.tokens
            && disabled_before.actions == disabled_after.actions;
        unrelated_invariant &= after.tokens == unrelated.tokens
            && after.actions == unrelated.actions
            && after.record_id == unrelated.record_id;
        absence_safe &= absent.status == "Absent"
            && evicted.status == "Evicted"
            && no_history.status == "NoHistory"
            && cross_scope.status == "Absent"
            && absent.record_id.is_none()
            && evicted.record_id.is_none()
            && no_history.record_id.is_none()
            && cross_scope.record_id.is_none()
            && absent.tokens.is_empty()
            && absent.actions.is_empty()
            && evicted.tokens.is_empty()
            && evicted.actions.is_empty()
            && no_history.tokens.is_empty()
            && no_history.actions.is_empty()
            && cross_scope.tokens.is_empty()
            && cross_scope.actions.is_empty();
        let changed = after.tokens != alternative.tokens;
        payload_changed |= changed;
        changed_pairs.push(
            json!({"case":case.name,"same_structure_payload_changes_tokens":changed,
            "before_after_changes_tokens":before.tokens != after.tokens,
            "after_copy":after.actions.iter().any(|a|a=="Copy"),
            "alternative_copy":alternative.actions.iter().any(|a|a=="Copy")}),
        );
    }
    let compiled_parity = rows.iter().all(|row| {
        (row.status == "Found" && row.compiled_parity == Some(true))
            || (row.status != "Found" && row.compiled_parity.is_none())
    });
    let local_gate = source_selection
        && disabled_invariant
        && unrelated_invariant
        && absence_safe
        && compiled_parity
        && payload_changed;
    let interface_gate = local_gate && reload_equal == Some(true);
    let decision = if interface_gate {
        "EXPLICIT_ADDRESS_GENERATION_CAUSALITY_WITNESSED"
    } else if local_gate && reload_equal.is_none() {
        "PENDING_INDEPENDENT_REPLAY"
    } else {
        "GENERATION_CAUSALITY_NOT_WITNESSED_OR_INTERFACE_FAIL"
    };
    Ok(json!({
        "source_selection":source_selection,
        "read_disabled_invariant":disabled_invariant,
        "unrelated_source_invariant":unrelated_invariant,
        "absence_eviction_copy_safe":absence_safe,
        "native_compiled_parity":compiled_parity,
        "independent_reload_equal":reload_equal,
        "same_structure_payload_changes_tokens":payload_changed,
        "pairs":changed_pairs,
        "interface_gate":interface_gate,
        "decision":decision,
    }))
}

fn write_json(root: &Path, name: &str, value: &impl Serialize) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    std::fs::write(root.join(name), bytes).map_err(|e| format!("write {name}: {e}"))
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if !(args.len() == 4 || (args.len() == 6 && args[4] == "--replay")) {
        return Err("usage: addressed-lexical-bridge NEW_REPORT_ROOT MODEL.tlx TOKENIZER.json [--replay SEALED_ROOT]".into());
    }
    let root = PathBuf::from(&args[1]);
    let model_path = PathBuf::from(&args[2]);
    let tokenizer_path = PathBuf::from(&args[3]);
    claim(&root).map_err(|e| e.to_string())?;
    let model_bytes = read_bound(&model_path, MODEL_SHA)?;
    let tokenizer_bytes = read_bound(&tokenizer_path, TOKENIZER_SHA)?;
    let model = TlModel::from_bytes(&model_bytes)?;
    let tokenizer = HfBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or("invalid derived tokenizer")?;
    if tokenizer.vocab_size() != model.vocab {
        return Err(format!(
            "tokenizer/model vocab mismatch {} != {}",
            tokenizer.vocab_size(),
            model.vocab
        ));
    }
    let (panel, expected, replay_source) = if args.len() == 6 {
        let source = PathBuf::from(&args[5]);
        let unlisted = verify(&source).map_err(|e| e.to_string())?;
        if !unlisted.is_empty() {
            return Err(format!("unlisted source files: {unlisted:?}"));
        }
        let panel: Panel = serde_json::from_slice(
            &std::fs::read(source.join("panel.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let rows: Vec<Row> = serde_json::from_slice(
            &std::fs::read(source.join("rows.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        (panel, Some(rows), Some(source))
    } else {
        (build_panel(&tokenizer)?, None, None)
    };
    let rows = evaluate(&model, &tokenizer, &panel)?;
    let reload_equal = expected.as_ref().map(|old| old == &rows);
    let result = gates(&rows, &panel.cases, reload_equal)?;
    let prose_prompt = tokenizer.encode("Once upon");
    let prose = model.rollout(
        &[],
        &[],
        SlFacts::default(),
        &prose_prompt,
        &[],
        MAX_NEW,
        false,
        false,
    );
    let executable_sha = sha(
        &std::fs::read(std::env::current_exe().map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?,
    );
    let receipt = json!({
        "schema":"uor-r4.addressed-lexical-bridge/1",
        "plan":"docs/integration/addressed-lexical-bridge-plan-2026-09-24.md",
        "mode":if expected.is_some(){"replay"}else{"run"},
        "replay_source":replay_source,
        "artifact":{"path":model_path,"sha256":MODEL_SHA},
        "tokenizer":{"path":tokenizer_path,"sha256":TOKENIZER_SHA},
        "source_sha256":{"runner":sha(include_bytes!("addressed-lexical-bridge.rs")),
            "bridge":sha(include_bytes!("../native_geometric/learner/addressed_lexical_bridge.rs"))},
        "executable_sha256":executable_sha,
        "controls":result,
        "unchanged_prose":{"prompt_tokens":prose_prompt,"tokens":prose.tokens,"actions":prose.actions.iter().map(|a|format!("{a:?}")).collect::<Vec<_>>(),"text":tokenizer.decode(&prose.tokens)},
        "cost":{"model_nonzero_parameter_reads_per_step":model.nonzero_per_step(),
            "memory_lookup":"exact sorted chain plus per-id record search; no served cost/energy qualification"},
        "scope":"Explicit typed address and writes; loaded native Generate/Copy/Stop. KVAR fitted gates, learned raw-text parsing/admission and general language are not tested.",
    });
    write_json(&root, "panel.json", &panel)?;
    write_json(&root, "rows.json", &rows)?;
    write_json(&root, "receipt.json", &receipt)?;
    seal(&root).map_err(|e| e.to_string())?;
    let unlisted = verify(&root).map_err(|e| e.to_string())?;
    if !unlisted.is_empty() {
        return Err(format!("unlisted files: {unlisted:?}"));
    }
    println!(
        "root={} decision={} reload_equal={reload_equal:?}",
        root.display(),
        receipt["controls"]["decision"]
    );
    for pair in &panel.cases {
        let a = named(&rows, &pair.name, "after_enabled")?;
        let b = named(&rows, &pair.name, "alternative_enabled")?;
        println!(
            "{} after={:?} alternative={:?} changed={}",
            pair.name,
            a.text,
            b.text,
            a.tokens != b.tokens
        );
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(2);
    }
}
