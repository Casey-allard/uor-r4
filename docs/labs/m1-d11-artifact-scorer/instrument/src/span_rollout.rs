//! Drive a frozen writer from independently sealed span and baseline positions.
use crate::content_rollout::grade_case;
use crate::copy_rollout::{contents, first_wrong, parse, replay, Arm, Rollout};
use crate::role_swap::{EOS, RELEASED};
use m1_d11_artifact_scorer_instrument::{artifact::ArtifactTables, device::DeviceScorer};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uor_r4_discrete::corpus::sha256;
use uor_r4_integer::{
    content_selector::{self, Prototypes},
    discrete_d11::ExactCopyState,
    span_selector::{resolve, Arms, Bank, SpanChoice},
};

const SEALED: &str =
    include_str!("../../../../loop/tasks/written-state-count-model/codex-5/BLIND-INPUTS.json");
const ARMS: [Arm; 6] = [
    Arm::Content,
    Arm::EndpointOnly,
    Arm::ConnectorOnly,
    Arm::Span,
    Arm::Oracle,
    Arm::Disabled,
];

#[derive(Deserialize, Serialize, PartialEq, Eq)]
struct Digests {
    artifact: String,
    grade: String,
    baseline_state: String,
    baseline_picks: String,
    span_state: String,
    span_picks: String,
}

#[derive(Deserialize)]
struct OldState {
    prototypes: Prototypes,
}

#[derive(Deserialize)]
struct Predictions<R> {
    artifact_sha256: String,
    state_sha256: String,
    rows: Vec<R>,
}

#[derive(Deserialize)]
struct OldRow {
    prefix_sha256: String,
    #[serde(flatten)]
    choice: content_selector::Choice,
}

#[derive(Deserialize)]
struct SpanState {
    bank: Bank,
}

#[derive(Deserialize)]
struct NewRow {
    prefix_sha256: String,
    #[serde(flatten)]
    choices: Arms,
}

impl NewRow {
    fn choice(&self, arm: Arm) -> Option<SpanChoice> {
        match arm {
            Arm::Span => Some(self.choices.span),
            Arm::EndpointOnly => Some(self.choices.endpoint_only),
            Arm::ConnectorOnly => Some(self.choices.connector_only),
            _ => None,
        }
    }
}

pub fn run(
    artifact: &str,
    grade: &str,
    old_state: &str,
    old_picks: &str,
    state: &str,
    picks: &str,
) -> Result<Value, String> {
    let model_bytes = contents(artifact)?;
    let grade_bytes = contents(grade)?;
    let baseline_bytes = contents(old_state)?;
    let old_bytes = contents(old_picks)?;
    let new_bytes = contents(picks)?;
    let span_bytes = contents(state)?;
    let hashes = Digests {
        artifact: sha256(&model_bytes),
        grade: sha256(&grade_bytes),
        baseline_state: sha256(&baseline_bytes),
        baseline_picks: sha256(&old_bytes),
        span_state: sha256(&span_bytes),
        span_picks: sha256(&new_bytes),
    };
    if !RELEASED.contains(&hashes.artifact.as_str())
        || !parse::<Vec<Digests>>(SEALED.as_bytes())?.contains(&hashes)
    {
        return Err("inputs differ from sealed blind fit/predictions or registered panel".into());
    }
    let grades: Vec<Value> = parse(&grade_bytes)?;
    let old: Predictions<OldRow> = parse(&old_bytes)?;
    let new: Predictions<NewRow> = parse(&new_bytes)?;
    if old.state_sha256 != hashes.baseline_state
        || new.state_sha256 != hashes.span_state
        || old.artifact_sha256 != hashes.artifact
        || new.artifact_sha256 != hashes.artifact
        || old.rows.len() != grades.len()
        || new.rows.len() != grades.len()
    {
        return Err("state or prediction coverage mismatch".into());
    }
    let baseline: OldState = parse(&baseline_bytes)?;
    let span_state: SpanState = parse(&span_bytes)?;
    let [connector] = baseline.prototypes.relation.as_slice() else {
        return Err("expected one connector prototype in pinned baseline".into());
    };
    if connector[0] != 1 {
        return Err("unexpected baseline connector length".into());
    }
    let legacy_root = connector[1];
    let (tables, model) = ArtifactTables::from_bytes(&model_bytes)?;
    let roots = model.packed_codebook_roots();
    let device = DeviceScorer::new(0, &tables)?;
    let mut rows = Vec::new();
    let mut differential_steps = 0;
    for (batch, group) in grades.chunks(16).enumerate() {
        let cases = group
            .iter()
            .map(|g| grade_case(g, &model))
            .collect::<Result<Vec<_>, _>>()?;
        let mut runs = Vec::new();
        for (i, c) in cases.iter().enumerate() {
            let j = batch * 16 + i;
            let old_row = &old.rows[j];
            let new_row = &new.rows[j];
            let prefix_hash = sha256(
                &c.prefix
                    .iter()
                    .flat_map(|id| id.to_le_bytes())
                    .collect::<Vec<_>>(),
            );
            let choice = content_selector::select(&c.prefix, &roots, &baseline.prototypes, false)
                .map_err(|e| e.to_string())?;
            if old_row.prefix_sha256 != prefix_hash
                || new_row.prefix_sha256 != prefix_hash
                || old_row.choice != choice
            {
                return Err(format!("{}: frozen anchor/prefix replay differs", c.id));
            }
            if resolve(&c.prefix, &roots, &span_state.bank, legacy_root, choice)
                .map_err(|e| e.to_string())?
                != new_row.choices
            {
                return Err(format!("{}: blind span/connector replay differs", c.id));
            }
            for arm in ARMS {
                let source = match new_row.choice(arm) {
                    Some(s) => {
                        if s.query != choice.query
                            || s.fact != choice.fact
                            || s.source.is_some_and(|x| x as usize >= c.prefix.len())
                        {
                            return Err(format!(
                                "{}: span intervention changed anchor or exceeded prompt",
                                c.id
                            ));
                        }
                        s.source
                    }
                    None => match arm {
                        Arm::Content => choice.source,
                        Arm::Oracle => Some(c.asked_start as u32),
                        Arm::Disabled => None,
                        _ => return Err("unknown intervention".into()),
                    },
                };
                runs.push(Rollout {
                    case: c,
                    arm,
                    source,
                    state: ExactCopyState::new(source, c.prefix.len() as u32),
                    prefix: c.prefix.clone(),
                    first_free: None,
                    steps: Vec::new(),
                });
            }
        }
        differential_steps += replay(&mut runs, &model, &device)?;
        for (k, r) in runs.into_iter().enumerate() {
            let i = k / ARMS.len();
            let j = batch * 16 + i;
            let c = r.case;
            let grade = &group[i];
            let emitted = &r.prefix[c.prefix.len()..];
            let wrong = first_wrong(emitted, &c.expected);
            let whole = emitted.starts_with(&c.prefix[c.asked_start..c.asked_end]);
            if r.arm == Arm::Oracle && !whole {
                return Err(format!("{}: oracle lost target", c.id));
            }
            let choice = if r.arm == Arm::Content {
                json!(old.rows[j].choice)
            } else {
                json!(new.rows[j].choice(r.arm))
            };
            rows.push(json!({"case":c.id,"pair":c.pair,"role":c.role,"arm":format!("{:?}",r.arm),
                "factors":grade["factors"],"gold_positions":grade["token_positions"],"choice":choice,
                "asked_start":c.asked_start,"other_start":c.other_start,"selected":r.source,"right_source":r.source==Some(c.asked_start as u32),
                "expected":c.expected,"generated":emitted,"first_wrong":wrong,"wrong_expected":wrong.and_then(|k|c.expected.get(k)),
                "wrong_emitted":wrong.and_then(|k|emitted.get(k)),"whole_value":whole,"first_free":r.first_free,
                "stop":if emitted.last()==Some(&EOS){"eos"}else{"cap"},"steps":r.steps}));
        }
    }
    Ok(
        json!({"schema":"uor-r4.span-writer/1","inputs":hashes,"writer_cubin_sha256":m1_d11_artifact_scorer_instrument::device::cubin_sha256(),"differential_steps":differential_steps,"rows":rows}),
    )
}
