//! Closed-loop value-boundary ablations, with every choice bound to the sealed
//! pre-grade state and verified again against serving on the original prefix.
use super::content_rollout::grade_case;
use super::copy_rollout::{contents, first_wrong, parse, replay, Arm, Rollout};
use super::role_swap::{EOS, RELEASED};
use m1_d11_artifact_scorer_instrument::{artifact::ArtifactTables, device::DeviceScorer};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uor_r4_discrete::corpus::sha256;
use uor_r4_integer::{
    discrete_d11::ExactCopyState,
    span_selector::{Bank, SpanChoice},
    value_selector::{decide, Decisions, ExactBank},
};

const SEALED: &str =
    include_str!("../../../../loop/tasks/written-state-count-model/codex-6/BLIND-INPUTS.json");
const ARMS: [Arm; 6] = [
    Arm::Current,
    Arm::WordStart,
    Arm::Ordered,
    Arm::Both,
    Arm::Oracle,
    Arm::Disabled,
];

#[derive(Deserialize, Serialize, PartialEq, Eq)]
struct Digests {
    artifact: String,
    grade: String,
    span_state: String,
    span_picks: String,
    exact_state: String,
    value_picks: String,
}

#[derive(Deserialize)]
struct State<B> {
    bank: B,
}

#[derive(Deserialize)]
struct SpanRow {
    prefix_sha256: String,
    span: SpanChoice,
}

#[derive(Deserialize)]
struct SpanPredictions {
    artifact_sha256: String,
    state_sha256: String,
    rows: Vec<SpanRow>,
}

#[derive(Deserialize)]
struct ValueRow {
    prefix_sha256: String,
    frozen: SpanChoice,
    #[serde(flatten)]
    decisions: Decisions,
}

#[derive(Deserialize)]
struct Predictions {
    artifact_sha256: String,
    state_sha256: String,
    span_state_sha256: String,
    span_predictions_sha256: String,
    rows: Vec<ValueRow>,
}

pub fn run(
    artifact: &str,
    grade_path: &str,
    span_state_path: &str,
    span_pick_path: &str,
    exact_state_path: &str,
    picks_path: &str,
) -> Result<Value, String> {
    let artifact = contents(artifact)?;
    let grade = contents(grade_path)?;
    let span_state = contents(span_state_path)?;
    let span_picks = contents(span_pick_path)?;
    let exact_state = contents(exact_state_path)?;
    let picks = contents(picks_path)?;
    let hashes = Digests {
        artifact: sha256(&artifact),
        grade: sha256(&grade),
        span_state: sha256(&span_state),
        span_picks: sha256(&span_picks),
        exact_state: sha256(&exact_state),
        value_picks: sha256(&picks),
    };
    if !RELEASED.contains(&hashes.artifact.as_str())
        || !parse::<Vec<Digests>>(SEALED.as_bytes())?.contains(&hashes)
    {
        return Err("value rollout inputs differ from the sealed panel and blind fit".into());
    }
    let grades: Vec<Value> = parse(&grade)?;
    let span: SpanPredictions = parse(&span_picks)?;
    let picks: Predictions = parse(&picks)?;
    if span.artifact_sha256 != hashes.artifact
        || span.state_sha256 != hashes.span_state
        || picks.artifact_sha256 != hashes.artifact
        || picks.state_sha256 != hashes.exact_state
        || picks.span_state_sha256 != hashes.span_state
        || picks.span_predictions_sha256 != hashes.span_picks
        || span.rows.len() != grades.len()
        || picks.rows.len() != grades.len()
    {
        return Err("inconsistent prediction handoff".into());
    }
    let span_state: State<Bank> = parse(&span_state)?;
    let exact_state: State<ExactBank> = parse(&exact_state)?;
    let (tables, model) = ArtifactTables::from_bytes(&artifact)?;
    let device = DeviceScorer::new(0, &tables)?;
    let roots = model.packed_codebook_roots();
    let mut rows = Vec::with_capacity(grades.len() * ARMS.len());
    let mut differential_steps = 0;
    for (batch, group) in grades.chunks(16).enumerate() {
        let cases = group
            .iter()
            .map(|g| grade_case(g, &model))
            .collect::<Result<Vec<_>, _>>()?;
        let mut runs = Vec::with_capacity(cases.len() * ARMS.len());
        for (i, c) in cases.iter().enumerate() {
            let j = batch * 16 + i;
            let old = &span.rows[j];
            let row = &picks.rows[j];
            let hash = sha256(
                &c.prefix
                    .iter()
                    .flat_map(|id| id.to_le_bytes())
                    .collect::<Vec<_>>(),
            );
            if old.prefix_sha256 != hash
                || row.prefix_sha256 != hash
                || row.frozen != old.span
                || row.decisions.current != old.span.source
                || decide(
                    &c.prefix,
                    &roots,
                    &span_state.bank,
                    &exact_state.bank,
                    old.span,
                )
                .map_err(|e| e.to_string())?
                    != row.decisions
            {
                return Err(format!(
                    "{}: prefix, endpoint, source or eligibility replay differs",
                    c.id
                ));
            }
            let d = &row.decisions;
            let sources = [
                d.current,
                d.word_start,
                d.ordered,
                d.both,
                Some(c.asked_start as u32),
                None,
            ];
            for (arm, source) in ARMS.into_iter().zip(sources) {
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
            let c = r.case;
            let g = &group[i];
            let pick = &picks.rows[batch * 16 + i];
            let emitted = &r.prefix[c.prefix.len()..];
            let wrong = first_wrong(emitted, &c.expected);
            let whole = emitted.starts_with(&c.prefix[c.asked_start..c.asked_end]);
            if r.arm == Arm::Oracle && !whole {
                return Err(format!("{}: oracle lost requested span", c.id));
            }
            let selector = !matches!(r.arm, Arm::Oracle | Arm::Disabled);
            let choice = selector.then_some(SpanChoice {
                source: r.source,
                ..pick.frozen
            });
            let d = &pick.decisions;
            rows.push(json!({"case":c.id,"pair":c.pair,"role":c.role,"arm":format!("{:?}",r.arm),
                "factors":g["factors"],"gold_positions":g["token_positions"],"choice":choice,
                "masks":selector.then(||json!({"eligible":d.eligible_mask,"word_start":d.word_start_mask,"ordered":d.ordered_mask})),
                "asked_start":c.asked_start,"other_start":c.other_start,"selected":r.source,"right_source":r.source==Some(c.asked_start as u32),
                "expected":c.expected,"generated":emitted,"first_wrong":wrong,"wrong_expected":wrong.and_then(|k|c.expected.get(k)),
                "wrong_emitted":wrong.and_then(|k|emitted.get(k)),"whole_value":whole,"first_free":r.first_free,
                "stop":if emitted.last()==Some(&EOS){"eos"}else{"cap"},"steps":r.steps}));
        }
    }
    Ok(
        json!({"schema":"uor-r4.value-writer/1","inputs":hashes,"writer_cubin_sha256":m1_d11_artifact_scorer_instrument::device::cubin_sha256(),"differential_steps":differential_steps,"rows":rows}),
    )
}
