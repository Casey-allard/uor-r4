//! Grade sealed blind source predictions by driving the same writer as #4086.
use crate::copy_rollout::{contents, first_wrong, parse, replay, Arm, Rollout, TRAINED_STATES};
use crate::role_swap::{Case, EOS, RELEASED};
use m1_d11_artifact_scorer_instrument::{artifact::ArtifactTables, device::DeviceScorer};
use serde::Deserialize;
use serde_json::{json, Value};
use uor_r4_discrete::corpus::sha256;
use uor_r4_integer::{
    content_selector::{select, Choice, Prototypes},
    discrete_d11::ExactCopyState,
    occurrence_selector,
};

const TRAIN: &str = "72d877a2ee5b9469ca0445618f6f1222c3ec5fb14063fcc0d4abbf7bc7a15c17";
const GRADES: [&str; 2] = [
    "7c1d5ab8f231dab7660f6d3b811e6dd8190dbbbaccf0dbebed94a772df2ec9e3",
    "783d3b43a3ab082ae36496626ff350b9613d623d7f696a72f5e6ab2fb1767a3f",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Inputs {
    content_state: String,
    ablated_state: String,
    content_picks: String,
    ablated_picks: String,
    offset_picks: String,
}

#[derive(Deserialize)]
struct State {
    artifact_sha256: String,
    train_sha256: String,
    arm: String,
    prototypes: Prototypes,
}

#[derive(Deserialize)]
struct Predictions {
    artifact_sha256: String,
    state_sha256: String,
    arm: String,
    rows: Vec<Row>,
}

struct FittedArm {
    prototypes: Prototypes,
    picks: Predictions,
}

// Exact states frozen after TRAIN and before any writer grading.
const STATE_DIGESTS: [[&str; 2]; 2] = [
    [
        "a9e340c153975b7ac3d8d9157283d78e3c5040d053e052f0fefec570f35b9317",
        "c3d3e5fc84ed5881e9fad2d8fea2665cb30f9731561658686d89b6b03fced239",
    ],
    [
        "41ff82f106f3e051dfa55406aea4f0fb5e63804210b5a339fe99dfc59b4cf72d",
        "aa558512de9ca2454d60d43b9f7e8c9e22002680fbe3c0b05eecad35efef0d23",
    ],
];

#[derive(Deserialize)]
struct Row {
    prefix_sha256: String,
    query: Option<u32>,
    fact: Option<u32>,
    source: Option<u32>,
}

impl Row {
    fn choice(&self) -> Choice {
        Choice {
            query: self.query,
            fact: self.fact,
            source: self.source,
        }
    }
}

fn load_picks(
    path: &str,
    artifact: &str,
    state: &str,
    arm: &str,
    size: usize,
) -> Result<Predictions, String> {
    let picks: Predictions = parse(&contents(path)?)?;
    if picks.artifact_sha256 != artifact
        || picks.state_sha256 != state
        || picks.arm != arm
        || picks.rows.len() != size
    {
        return Err(format!("wrong artifact/state/arm or row count in {path}"));
    }
    Ok(picks)
}

pub fn run(artifact: &str, grade_path: &str, config_path: &str) -> Result<Value, String> {
    let model_bytes = contents(artifact)?;
    let artifact_hash = sha256(&model_bytes);
    let seed = RELEASED
        .iter()
        .position(|&h| h == artifact_hash)
        .ok_or("unregistered release")?;
    let grade_bytes = contents(grade_path)?;
    let grade_hash = sha256(&grade_bytes);
    if !GRADES.contains(&grade_hash.as_str()) {
        return Err("unregistered grade file".into());
    }
    let grades: Vec<Value> = parse(&grade_bytes)?;
    let inputs: Inputs = parse(&contents(config_path)?)?;
    let (tables, model) = ArtifactTables::from_bytes(&model_bytes)?;
    let roots = model.packed_codebook_roots();
    let device = DeviceScorer::new(0, &tables)?;
    let mut fitted = Vec::new();
    let mut hashes = Vec::new();
    for (arm, state_path, picks_path) in [
        ("trained", &inputs.content_state, &inputs.content_picks),
        (
            "query_ablated",
            &inputs.ablated_state,
            &inputs.ablated_picks,
        ),
    ] {
        let bytes = contents(state_path)?;
        let state: State = parse(&bytes)?;
        if state.arm != arm || state.artifact_sha256 != artifact_hash || state.train_sha256 != TRAIN
        {
            return Err("wrong training data, arm or artifact in fitted state".into());
        }
        state.prototypes.validate().map_err(|e| e.to_string())?;
        let digest = sha256(&bytes);
        if digest != STATE_DIGESTS[seed][fitted.len()] {
            return Err("selector state differs from the frozen blind fit".into());
        }
        let picks = load_picks(picks_path, &artifact_hash, &digest, arm, grades.len())?;
        hashes.push(digest);
        fitted.push(FittedArm {
            prototypes: state.prototypes,
            picks,
        });
    }
    if fitted[0].prototypes != fitted[1].prototypes {
        return Err("ablation changed prototypes".into());
    }
    let offset = load_picks(
        &inputs.offset_picks,
        &artifact_hash,
        TRAINED_STATES[seed],
        "trained",
        grades.len(),
    )?;
    let mut rows = Vec::new();
    let mut steps = 0;
    for (batch_index, batch) in grades.chunks(16).enumerate() {
        let mut cases = Vec::new();
        for grade in batch {
            let mut core = grade.clone();
            let obj = core.as_object_mut().ok_or("bad grade")?;
            obj.retain(|key, _| {
                matches!(
                    key.as_str(),
                    "id" | "pair"
                        | "role"
                        | "prefix"
                        | "asked_start"
                        | "asked_end"
                        | "other_start"
                        | "other_end"
                        | "expected"
                )
            });
            let case: Case = serde_json::from_value(core).map_err(|e| e.to_string())?;
            case.validate(&model)?;
            cases.push(case);
        }
        let mut runs = Vec::new();
        for (i, case) in cases.iter().enumerate() {
            let j = batch_index * 16 + i;
            let hash = sha256(
                &case
                    .prefix
                    .iter()
                    .flat_map(|t| t.to_le_bytes())
                    .collect::<Vec<_>>(),
            );
            for (k, state) in fitted.iter().enumerate() {
                let row = &state.picks.rows[j];
                if row.prefix_sha256 != hash
                    || row.choice()
                        != select(&case.prefix, &roots, &state.prototypes, k == 1)
                            .map_err(|e| e.to_string())?
                {
                    return Err(format!(
                        "{}: content prediction differs from serving",
                        case.id
                    ));
                }
            }
            if offset.rows[j].prefix_sha256 != hash
                || offset.rows[j].source
                    != occurrence_selector::select(&case.prefix, &roots, 2, 13)
                        .map_err(|e| e.to_string())?
            {
                return Err(format!(
                    "{}: offset prediction differs from serving",
                    case.id
                ));
            }
            for (arm, source) in [
                (Arm::Content, fitted[0].picks.rows[j].source),
                (Arm::QueryAblated, fitted[1].picks.rows[j].source),
                (Arm::Offset, offset.rows[j].source),
                (Arm::Oracle, Some(case.asked_start as u32)),
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
        steps += replay(&mut runs, &model, &device)?;
        for (i, r) in runs.into_iter().enumerate() {
            let grade = &batch[i / 5];
            let j = batch_index * 16 + i / 5;
            let case = r.case;
            let emitted = &r.prefix[case.prefix.len()..];
            let wrong = first_wrong(emitted, &case.expected);
            let whole = emitted.starts_with(&case.prefix[case.asked_start..case.asked_end]);
            if r.arm == Arm::Oracle && !whole {
                return Err(format!("{}: oracle span lost", case.id));
            }
            let choice = match r.arm {
                Arm::Content => Some(fitted[0].picks.rows[j].choice()),
                Arm::QueryAblated => Some(fitted[1].picks.rows[j].choice()),
                _ => None,
            };
            rows.push(json!({"case":case.id,"pair":case.pair,"role":case.role,"arm":format!("{:?}",r.arm),
                "factors":grade["factors"],"gold_positions":grade["token_positions"],"choice":choice,
                "asked_start":case.asked_start,"other_start":case.other_start,"selected":r.source,
                "right_source":r.source==Some(case.asked_start as u32),"whole_value":whole,"expected":case.expected,"generated":emitted,"first_wrong":wrong,
                "wrong_expected":wrong.and_then(|k|case.expected.get(k)),"wrong_emitted":wrong.and_then(|k|emitted.get(k)),
                "first_free":r.first_free,"stop":if emitted.last()==Some(&EOS){"eos"}else{"cap"},"steps":r.steps}));
        }
    }
    Ok(
        json!({"schema":"uor-r4.content-writer/1","artifact_sha256":artifact_hash,"grade_sha256":grade_hash,
        "content_and_ablated_state_sha256":hashes,"offset_state_sha256":TRAINED_STATES[seed],"writer_cubin_sha256":m1_d11_artifact_scorer_instrument::device::cubin_sha256(),
        "differential_steps":steps,"rows":rows}),
    )
}
