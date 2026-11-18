//! Closed-loop replay of the source-bound writer. Gold spans enter only the
//! oracle's initial position and the scorer that grades the completed rollouts.
use crate::role_swap::{Case, CAP, EOS, PANEL, RELEASED};
use m1_d11_artifact_scorer_instrument as scorer;
use scorer::artifact::ArtifactTables;
use scorer::device::DeviceScorer;
use scorer::kernel::pack_rows;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use uor_r4_discrete::corpus::sha256;
use uor_r4_integer::discrete_d11::ExactCopyState;
use uor_r4_integer::occurrence_selector::select;

const PRIOR_TRACES: [&str; 2] = [
    "d4b08939083163a6d6213de5a812226cff96bec94ad779d7aa681b68fca9c05b",
    "ce39bae0ccc4ef72e9dba937a672cd5b615a34aa5e30819306b3c5cd0ddd0055",
];
const STOPS: [u32; 4] = [16, EOS, 0, 201];
// Byte identities from the blind TRAIN-only fit, sealed before writer rollout.
const TRAINED_STATES: [&str; 2] = [
    "5981d5716bb9674646cb5292da5455abf349dea7367be682ef310974c88f1c07",
    "c54dc669339bc22a57a8ee676141ec780b5ba5ea3185940de65a96811ca94944",
];

#[derive(Deserialize)]
struct FittedState {
    artifact_sha256: String,
    arm: String,
    lookback: u8,
    anchor_back: u8,
}

#[derive(Deserialize)]
struct Selections {
    artifact_sha256: String,
    state_sha256: String,
    rows: Vec<Selection>,
}

#[derive(Deserialize)]
struct Selection {
    prefix_sha256: String,
    source: Option<u32>,
}

#[derive(Deserialize)]
struct Historic {
    rows: Vec<HistoricRow>,
}

#[derive(Deserialize)]
struct HistoricRow {
    id: String,
    arm: String,
    generated: Vec<u32>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    Oracle,
    Learned,
    Disabled,
}

struct Rollout<'a> {
    case: &'a Case,
    arm: Arm,
    source: Option<u32>,
    state: ExactCopyState,
    prefix: Vec<u32>,
    first_free: Option<(usize, u32)>,
    steps: Vec<Value>,
}

fn contents(path: &str) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|e| format!("{path}: {e}"))
}

fn parse<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    serde_json::from_slice(bytes).map_err(|e| e.to_string())
}

fn first_wrong(generated: &[u32], expected: &[u32]) -> Option<usize> {
    generated
        .iter()
        .zip(expected)
        .position(|(a, b)| a != b)
        .or_else(|| {
            (generated.len() != expected.len()).then_some(expected.len().min(generated.len()))
        })
}

pub fn run(
    artifact: &str,
    panel: &str,
    state_path: &str,
    picks_path: &str,
    previous: &str,
) -> Result<Value, String> {
    let model_bytes = contents(artifact)?;
    let artifact_hash = sha256(&model_bytes);
    let seed = RELEASED
        .iter()
        .position(|&h| h == artifact_hash)
        .ok_or("unregistered D11 artifact")?;
    let panel_bytes = contents(panel)?;
    let old_bytes = contents(previous)?;
    if sha256(&panel_bytes) != PANEL || sha256(&old_bytes) != PRIOR_TRACES[seed] {
        return Err("role fixture or historical baseline digest differs".into());
    }
    let cases: Vec<Case> = parse(&panel_bytes)?;
    let prior: Historic = parse(&old_bytes)?;
    let native: BTreeMap<_, _> = prior
        .rows
        .into_iter()
        .filter(|r| r.arm == "Native")
        .map(|r| (r.id, r.generated))
        .collect();
    let state_bytes = contents(state_path)?;
    let state_hash = sha256(&state_bytes);
    let fitted: FittedState = parse(&state_bytes)?;
    let picks_bytes = contents(picks_path)?;
    let picks: Selections = parse(&picks_bytes)?;
    if state_hash != TRAINED_STATES[seed]
        || fitted.arm != "trained"
        || fitted.artifact_sha256 != artifact_hash
        || picks.artifact_sha256 != artifact_hash
        || picks.state_sha256 != state_hash
    {
        return Err("fitted selector or prefix predictions have different provenance".into());
    }
    if picks.rows.len() != cases.len() {
        return Err("selector prediction count differs from frozen panel".into());
    }
    let mut by_prefix: BTreeMap<_, _> = picks
        .rows
        .into_iter()
        .map(|r| (r.prefix_sha256, r.source))
        .collect();
    let (tables, model) = ArtifactTables::from_bytes(&model_bytes)?;
    let roots = model.packed_codebook_roots();
    let mut runs = Vec::new();
    for case in &cases {
        case.validate(&model)?;
        let hash = sha256(
            &case
                .prefix
                .iter()
                .flat_map(|t| t.to_le_bytes())
                .collect::<Vec<_>>(),
        );
        let selected = by_prefix
            .remove(&hash)
            .ok_or("missing unique CUDA selection for this prompt")?;
        let independent = select(&case.prefix, &roots, fitted.lookback, fitted.anchor_back)
            .map_err(|e| e.to_string())?;
        if independent != selected {
            return Err(format!("{}: CPU/CUDA source mismatch", case.id));
        }
        for (arm, source) in [
            (Arm::Oracle, Some(case.asked_start as u32)),
            (Arm::Learned, selected),
            (Arm::Disabled, None),
        ] {
            runs.push(Rollout {
                case,
                arm,
                source,
                state: ExactCopyState::new(source, case.prefix.len() as u32),
                prefix: case.prefix.clone(),
                first_free: None,
                steps: Vec::new(),
            });
        }
    }
    if !by_prefix.is_empty() || native.len() != cases.len() {
        return Err("unexpected prefix coverage".into());
    }
    let device = DeviceScorer::new(0, &tables)?;
    let mut transitions = 0;
    for _ in 0..CAP {
        let active: Vec<_> = runs
            .iter()
            .enumerate()
            .filter(|(_, r)| r.prefix.last() != Some(&EOS))
            .map(|(i, _)| i)
            .collect();
        if active.is_empty() {
            break;
        }
        let states: Vec<_> = active.iter().map(|&i| runs[i].state).collect();
        let batch = pack_rows(
            &active
                .iter()
                .map(|&i| runs[i].prefix.as_slice())
                .collect::<Vec<_>>(),
        )?;
        let (scores, steps) = device.score_copy(&batch, &states, STOPS)?;
        if steps.len() != active.len() || scores.len() != active.len() {
            return Err("CUDA returned incomplete batch".into());
        }
        for ((i, score), step) in active.into_iter().zip(scores).zip(steps) {
            let r = &mut runs[i];
            let (ordinary, cpu) = model
                .evaluate_prefix_with_exact_copy(&r.prefix, r.state, STOPS)
                .map_err(|e| e.to_string())?;
            if cpu.words() != step.words()
                || score.status != 0
                || score.prediction != ordinary.prediction
                || score.read_vector != ordinary.read_vector
                || score.admitted_cells != ordinary.admitted_cells
                || score.top8 != ordinary.top8
            {
                return Err(format!(
                    "{} {:?} step {}: source, cursor, emission or ordinary path differs",
                    r.case.id,
                    r.arm,
                    r.steps.len()
                ));
            }
            if step.copied == 1
                && (r.state.cursor >= r.state.prompt_end
                    || r.case.prefix.get(r.state.cursor as usize) != Some(&step.emitted)
                    || step.exact_id != step.emitted)
            {
                return Err("exact copy lost source identity".into());
            }
            if step.copied == 0 && r.first_free.is_none() {
                r.first_free = Some((r.steps.len(), step.emitted));
            }
            r.steps.push(
                json!({"before":r.state.words(),"after":step.state.words(),"copied":step.copied,
                "source_id":(step.copied==1).then_some(step.exact_id), "release":step.release,
                "native_id":score.prediction,"native_top8":score.top8,"emitted":step.emitted}),
            );
            r.prefix.push(step.emitted);
            r.state = step.state;
            transitions += 1;
        }
    }
    let mut rows = Vec::new();
    for r in runs {
        let emitted = &r.prefix[r.case.prefix.len()..];
        let old = native
            .get(&r.case.id)
            .ok_or("missing historical native row")?;
        if r.arm == Arm::Disabled && emitted != old {
            return Err(format!("{} disabled drifted from #4084", r.case.id));
        }
        let value = &r.case.prefix[r.case.asked_start..r.case.asked_end];
        let wrong = first_wrong(emitted, &r.case.expected);
        let selector_match = if r.arm == Arm::Learned {
            r.source.and_then(|source| {
                let anchor = r.case.prefix.len().checked_sub(1 + usize::from(fitted.anchor_back))?;
                let key = (source as usize).checked_sub(usize::from(fitted.lookback))?;
                let query_id = *r.case.prefix.get(anchor)?;
                let key_id = *r.case.prefix.get(key)?;
                Some(json!({"query_position":anchor,"query_id":query_id,"query_roots":roots[query_id as usize],
                    "matched_position":key,"matched_id":key_id,"matched_roots":roots[key_id as usize]}))
            })
        } else {
            None
        };
        rows.push(json!({"case":r.case.id,"pair":r.case.pair,"role":r.case.role,"arm":format!("{:?}",r.arm),
            "selector_match":selector_match,
            "asked_start":r.case.asked_start,"selected":r.source,"right_source":r.source==Some(r.case.asked_start as u32),
            "expected":r.case.expected,"generated":emitted,"whole_value":emitted.starts_with(value),"exact":wrong.is_none(),
            "first_wrong":wrong,"previous_first_wrong":first_wrong(old,&r.case.expected),"previous_generated":old,
            "first_change_from_disabled":first_wrong(emitted,old),"first_free_step_and_id":r.first_free,
            "wrong_expected":wrong.and_then(|i|r.case.expected.get(i)),"wrong_emitted":wrong.and_then(|i|emitted.get(i)),
            "stop":if emitted.last()==Some(&EOS){"eos"}else{"cap"},"steps":r.steps}));
    }
    Ok(
        json!({"schema":"uor-r4.exact-copy-rollout/1","artifact_sha256":artifact_hash,"fixture_sha256":PANEL,
        "selector_state_sha256":state_hash,"selector_predictions_sha256":sha256(&picks_bytes),
        "previous_trace_sha256":PRIOR_TRACES[seed],"lookback":fitted.lookback,"anchor_back":fitted.anchor_back,
        "aot_cubin_sha256":scorer::device::cubin_sha256(),"aot_ptx_sha256":scorer::device::ptx_sha256(),
        "cuda_target":scorer::device::IMAGE_TARGET,"differential_steps":transitions,"rows":rows}),
    )
}
