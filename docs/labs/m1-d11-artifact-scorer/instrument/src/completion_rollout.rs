//! Self-fed, paired continuation × end-of-copy measurement. Only the oracle
//! receives a labeled address. Every other source decision replays on bare IDs.
#[path = "entity_writer.rs"]
pub(crate) mod entity_writer;
use super::{Digests, Predictions, SpanPredictions, State};
use crate::content_rollout::grade_case;
use crate::copy_rollout::{
    contents, first_wrong, parse, replay, Arm, Rollout, WriterPrompt, STOPS,
};
use crate::role_swap::{EOS, RELEASED};
use m1_d11_artifact_scorer_instrument::{artifact::ArtifactTables, device, kernel::pack_rows};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uor_r4_discrete::corpus::sha256;
use uor_r4_integer::{
    continuation_selector::{self, Continuation},
    discrete_d11::ExactCopyState,
    span_selector::{Bank, SpanChoice},
    value_selector::{self, ExactBank},
};

// Filled from the sealed, independently repeated blind files before grading.
const SEALED: &str =
    include_str!("../../../../loop/tasks/written-state-count-model/codex-7/BLIND-INPUTS.json");
const HISTORIC_CUBIN: &str = "b41d72ce289ff5daf29055687e2586929321fed6ddb800d10cddfc650e2eeb28";

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Inputs {
    artifact: String,
    grade: String,
    bare: String,
    span_state: String,
    span_picks: String,
    exact_state: String,
    value_picks: String,
    continuation_picks: String,
}

#[derive(Deserialize, PartialEq, Eq, Serialize)]
struct Seal {
    #[serde(flatten)]
    baseline: Digests,
    bare: String,
    continuation_picks: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Bare {
    prefix: Vec<u32>,
}

#[derive(Deserialize)]
struct ContinuationRow {
    prefix_sha256: String,
    frozen: SpanChoice,
    word_start: Option<u32>,
    continuation: Continuation,
}

#[derive(Deserialize)]
struct ContinuationPredictions {
    schema: String,
    artifact_sha256: String,
    span_state_sha256: String,
    exact_state_sha256: String,
    value_picks_sha256: String,
    bare_sha256: String,
    rows: Vec<ContinuationRow>,
}

#[derive(Deserialize, Serialize, PartialEq, Eq)]
struct Witness {
    prefix_sha256: String,
    arm: String,
    source: Option<u32>,
    trace_sha256: String,
}

fn witness<P: WriterPrompt>(r: &Rollout<P>, row: &ContinuationRow) -> Result<Witness, String> {
    let emitted = &r.prefix[r.case.tokens().len()..];
    let record = (row.frozen, r.source, emitted, r.first_free, &r.steps);
    Ok(Witness {
        prefix_sha256: row.prefix_sha256.clone(),
        arm: format!("{:?}", r.arm),
        source: r.source,
        trace_sha256: sha256(&serde_json::to_vec(&record).map_err(|e| e.to_string())?),
    })
}

fn choices(row: &ContinuationRow) -> [(Arm, Option<u32>); 4] {
    [
        (Arm::WordStart, row.word_start),
        (Arm::Continuation, row.continuation.source),
        (Arm::WordStartEos, row.word_start),
        (Arm::ContinuationEos, row.continuation.source),
    ]
}

fn rollout<P: WriterPrompt>(case: &P, arm: Arm, source: Option<u32>) -> Rollout<'_, P> {
    Rollout {
        case,
        arm,
        source,
        state: ExactCopyState::new(source, case.tokens().len() as u32),
        prefix: case.tokens().to_vec(),
        first_free: None,
        steps: Vec::new(),
    }
}

#[derive(Serialize, Deserialize)]
struct BlindWriter {
    schema: String,
    model: String,
    bare: String,
    picks: String,
    steps: usize,
    rows: Vec<Witness>,
}

struct BarePrompt<'a> {
    hash: &'a str,
    prefix: &'a [u32],
}
impl WriterPrompt for BarePrompt<'_> {
    fn id(&self) -> &str {
        self.hash
    }
    fn tokens(&self) -> &[u32] {
        self.prefix
    }
}

/// This command has no grade argument or labels. Its sealed witnesses bind the
/// actual feedback, cursor, native proposal, selection and endpoints per arm.
pub fn blind(model_path: &str, bare_path: &str, picks_path: &str) -> Result<Value, String> {
    let model = contents(model_path)?;
    let bare = contents(bare_path)?;
    let picks = contents(picks_path)?;
    let mut report = BlindWriter {
        schema: "uor-r4.bare-writer/1".into(),
        model: sha256(&model),
        bare: sha256(&bare),
        picks: sha256(&picks),
        steps: 0,
        rows: Vec::new(),
    };
    let seals: Vec<Seal> = parse(SEALED.as_bytes())?;
    if !RELEASED.contains(&report.model.as_str())
        || !seals.iter().any(|s| {
            s.baseline.artifact == report.model
                && s.bare == report.bare
                && s.continuation_picks == report.picks
        })
        || device::cubin_sha256() != HISTORIC_CUBIN
    {
        return Err("bare writer refused unsealed inputs".into());
    }
    let (tables, model) = ArtifactTables::from_bytes(&model)?;
    let gpu = device::DeviceScorer::new(0, &tables)?;
    let bare: Vec<Bare> = parse(&bare)?;
    let picks: ContinuationPredictions = parse(&picks)?;
    if picks.artifact_sha256 != report.model
        || picks.bare_sha256 != report.bare
        || bare.len() != picks.rows.len()
    {
        return Err("bare handoff mismatch".into());
    }
    for (inputs, decisions) in bare.chunks(16).zip(picks.rows.chunks(16)) {
        let mut prompts = Vec::new();
        for (input, row) in inputs.iter().zip(decisions) {
            if sha256(
                &input
                    .prefix
                    .iter()
                    .flat_map(|id| id.to_le_bytes())
                    .collect::<Vec<_>>(),
            ) != row.prefix_sha256
            {
                return Err("bare prompt identity mismatch".into());
            }
            prompts.push(BarePrompt {
                hash: &row.prefix_sha256,
                prefix: &input.prefix,
            });
        }
        let mut runs: Vec<_> = prompts
            .iter()
            .zip(decisions)
            .flat_map(|(p, d)| {
                choices(d)
                    .into_iter()
                    .map(|(arm, source)| rollout(p, arm, source))
            })
            .collect();
        report.steps += replay(&mut runs, &model, &gpu)?;
        for (k, r) in runs.iter().enumerate() {
            report.rows.push(witness(r, &decisions[k / 4])?);
        }
    }
    serde_json::to_value(report).map_err(|e| e.to_string())
}

pub fn run(paths: &str, blind_path: &str) -> Result<Value, String> {
    let paths: Inputs = parse(&contents(paths)?)?;
    let artifact = contents(&paths.artifact)?;
    let grade = contents(&paths.grade)?;
    let bare = contents(&paths.bare)?;
    let span_state = contents(&paths.span_state)?;
    let span = contents(&paths.span_picks)?;
    let exact_state = contents(&paths.exact_state)?;
    let value = contents(&paths.value_picks)?;
    let cont = contents(&paths.continuation_picks)?;
    let blind_bytes = contents(blind_path)?;
    let blind_sha = sha256(&blind_bytes);
    let blind: BlindWriter = parse(&blind_bytes)?;
    let seal = Seal {
        baseline: Digests {
            artifact: sha256(&artifact),
            grade: sha256(&grade),
            span_state: sha256(&span_state),
            span_picks: sha256(&span),
            exact_state: sha256(&exact_state),
            value_picks: sha256(&value),
        },
        bare: sha256(&bare),
        continuation_picks: sha256(&cont),
    };
    let h = &seal.baseline;
    if !RELEASED.contains(&h.artifact.as_str())
        || !parse::<Vec<Seal>>(SEALED.as_bytes())?.contains(&seal)
    {
        return Err("completion rollout inputs differ from sealed blind experiment".into());
    }
    if device::cubin_sha256() != HISTORIC_CUBIN {
        return Err("original scoring and copy cubin changed".into());
    }
    let (tables, model) = ArtifactTables::from_bytes(&artifact)?;
    let roots = model.packed_codebook_roots();
    let span_state: State<Bank> = parse(&span_state)?;
    let exact_state: State<ExactBank> = parse(&exact_state)?;
    let span: SpanPredictions = parse(&span)?;
    let value: Predictions = parse(&value)?;
    let cont: ContinuationPredictions = parse(&cont)?;
    let bare: Vec<Bare> = parse(&bare)?;
    let grades: Vec<Value> = parse(&grade)?;
    if blind.schema != "uor-r4.bare-writer/1"
        || blind.model != h.artifact
        || blind.bare != seal.bare
        || blind.picks != seal.continuation_picks
        || blind.rows.len() != grades.len() * 4
    {
        return Err("pre-grade bare writer witness mismatch".into());
    }
    if span.artifact_sha256 != h.artifact
        || span.state_sha256 != h.span_state
        || value.artifact_sha256 != h.artifact
        || value.state_sha256 != h.exact_state
        || value.span_state_sha256 != h.span_state
        || value.span_predictions_sha256 != h.span_picks
        || cont.schema != "uor-r4.continuation/1"
        || cont.artifact_sha256 != h.artifact
        || cont.span_state_sha256 != h.span_state
        || cont.exact_state_sha256 != h.exact_state
        || cont.value_picks_sha256 != h.value_picks
        || cont.bare_sha256 != seal.bare
        || [
            span.rows.len(),
            value.rows.len(),
            cont.rows.len(),
            bare.len(),
        ]
        .iter()
        .any(|&len| len != grades.len())
    {
        return Err("inconsistent handoff across frozen and continuation predictions".into());
    }
    let gpu = device::DeviceScorer::new(0, &tables)?;
    let mut rows = Vec::with_capacity(grades.len() * 7);
    let mut differential_steps = 0;
    for (batch, group) in grades.chunks(16).enumerate() {
        let cases = group
            .iter()
            .map(|g| grade_case(g, &model))
            .collect::<Result<Vec<_>, _>>()?;
        let mut runs = Vec::with_capacity(cases.len() * 7);
        for (i, case) in cases.iter().enumerate() {
            let j = batch * 16 + i;
            let ids = &bare[j].prefix;
            let old = &span.rows[j];
            let w = &value.rows[j];
            let c = &cont.rows[j];
            let hash = sha256(
                &ids.iter()
                    .flat_map(|id| id.to_le_bytes())
                    .collect::<Vec<_>>(),
            );
            let baseline =
                value_selector::decide(ids, &roots, &span_state.bank, &exact_state.bank, old.span)
                    .map_err(|e| e.to_string())?;
            let continuation =
                continuation_selector::decide(ids, &roots, &span_state.bank, old.span, baseline)
                    .map_err(|e| e.to_string())?;
            if &case.prefix != ids
                || old.prefix_sha256 != hash
                || w.prefix_sha256 != hash
                || c.prefix_sha256 != hash
                || w.frozen != old.span
                || c.frozen != old.span
                || w.decisions != baseline
                || baseline.current != old.span.source
                || c.word_start != baseline.word_start
                || c.continuation != continuation
            {
                return Err(format!(
                    "{}: bare-prefix, endpoint, mask or choice replay differs",
                    case.id
                ));
            }
            let oracle = Some(case.asked_start as u32);
            for (arm, source) in choices(c).into_iter().chain([
                (Arm::Oracle, oracle),
                (Arm::OracleEos, oracle),
                (Arm::Disabled, None),
            ]) {
                runs.push(rollout(case, arm, source));
            }
        }
        differential_steps += replay(&mut runs, &model, &gpu)?;
        for (k, r) in runs.into_iter().enumerate() {
            let i = k / 7;
            let c = r.case;
            let g = &group[i];
            let row = &cont.rows[batch * 16 + i];
            let selector = !matches!(r.arm, Arm::Oracle | Arm::OracleEos | Arm::Disabled);
            if selector && witness(&r, row)? != blind.rows[(batch * 16 + i) * 4 + k % 7] {
                return Err(format!(
                    "{} {:?} differs from pre-grade bare writer",
                    c.id, r.arm
                ));
            }
            let choice = selector.then_some(SpanChoice {
                source: r.source,
                ..row.frozen
            });
            let emitted = &r.prefix[c.prefix.len()..];
            let wrong = first_wrong(emitted, &c.expected);
            let whole = emitted.starts_with(&c.prefix[c.asked_start..c.asked_end]);
            let eos_action_step = r
                .arm
                .finishes_copy()
                .then(|| {
                    r.steps.iter().position(|s| {
                        s["release"] == 1 && s["before"][2].as_u64() > s["before"][1].as_u64()
                    })
                })
                .flatten();
            if matches!(r.arm, Arm::Oracle | Arm::OracleEos) && !whole {
                return Err(format!("{}: oracle lost requested value", c.id));
            }
            rows.push(json!({"case":c.id,"pair":c.pair,"role":c.role,"arm":format!("{:?}",r.arm),
                "factors":g["factors"],"gold_positions":g["token_positions"],"choice":choice,
                "prefix_sha256":row.prefix_sha256,"continuation":selector.then_some(&row.continuation),
                "asked_start":c.asked_start,"other_start":c.other_start,"selected":r.source,"right_source":r.source==Some(c.asked_start as u32),
                "expected":c.expected,"generated":emitted,"first_wrong":wrong,"wrong_expected":wrong.and_then(|k|c.expected.get(k)),
                "wrong_emitted":wrong.and_then(|k|emitted.get(k)),"whole_value":whole,"exact":wrong.is_none(),"first_free":r.first_free,
                "eos_action_step":eos_action_step,"stop":if emitted.last()==Some(&EOS){"eos"}else{"cap"},"steps":r.steps}));
        }
    }
    Ok(
        json!({"schema":"uor-r4.completion-writer/1","inputs":seal,"writer_cubin_sha256":device::cubin_sha256(),
        "completion_cubin_sha256":sha256(device::COMPLETION_CUBIN),"bare_writer_sha256":blind_sha,
        "bare_writer_steps":blind.steps,"bare_witnesses":blind.rows.len(),"differential_steps":differential_steps,"rows":rows}),
    )
}

pub fn finite(artifact: &str) -> Result<Value, String> {
    use uor_r4_integer::discrete_d11::ExactCopyState as S;
    let bytes = contents(artifact)?;
    if !RELEASED.contains(&sha256(&bytes).as_str()) || device::cubin_sha256() != HISTORIC_CUBIN {
        return Err("finite writer model or cubin drift".into());
    }
    let (tables, model) = ArtifactTables::from_bytes(&bytes)?;
    let gpu = device::DeviceScorer::new(0, &tables)?;
    // A disabled or unplugged finalizer must fail this test. Require a real
    // non-EOS proposal on one planted synthetic prefix for all paired cases.
    let mut prefixes = Vec::new();
    for tail in [28, 1344, 2551, 1623, 201, 16, 0, 35, 560, 652, 714] {
        let mut prefix = vec![
            90, 16, 10, EOS, 11, 0, 12, 201, 16, 13, 201, 35, 560, 652, 714, 28,
        ];
        prefix.push(tail);
        prefixes.push(prefix);
    }
    let probes = pack_rows(&prefixes.iter().map(Vec::as_slice).collect::<Vec<_>>())?;
    let index = gpu
        .score(&probes)?
        .iter()
        .position(|s| s.status == 0 && s.prediction != EOS)
        .ok_or("synthetic finite cases failed to exercise a non-EOS native proposal")?;
    let prefix = &prefixes[index];
    // Fence/cap precedence, each delimiter, immediate delimiter, inactive
    // after a prior copy, no source and ordinary copy all get both policies.
    let examples = [
        ([1, 0, 1, 10], 1, true),
        ([1, 2, 3, 10], 1, true),
        ([1, 4, 5, 10], 1, true),
        ([1, 6, 7, 10], 1, true),
        ([1, 1, 1, 10], 1, false),
        ([1, 0, 1, 1], 2, false),
        ([1, 0, 8, 10], 3, false),
        ([1, 0, 8, 8], 3, false),
        ([0, 0, 1, 10], 0, false),
        (S::new(None, 10).words(), 0, false),
        ([1, 0, 0, 10], 0, false),
    ];
    let mut tested = Vec::new();
    for (words, release, completed) in examples {
        let s = S::from_words(words);
        let batch = pack_rows(&[prefix, prefix])?;
        let (scores, steps) =
            gpu.score_copy_with_completion(&batch, &[s, s], STOPS, &[0, 1], EOS)?;
        let (ordinary, base) = model
            .evaluate_prefix_with_exact_copy(prefix, s, STOPS)
            .map_err(|e| e.to_string())?;
        if scores.iter().any(|v| v.prediction != ordinary.prediction)
            || steps[0] != base
            || steps[1] != base.finish_with_eos(true, EOS)
            || base.release != release
            || base.completed_span() != completed
            || ordinary.prediction == EOS
            || (steps[0].emitted != steps[1].emitted) != completed
            || steps[1].emitted != if completed { EOS } else { base.emitted }
        {
            return Err(format!("finite completion mismatch {words:?}"));
        }
        tested.push(json!({"before":words,"native":ordinary.prediction,"off":steps[0].words(),"on":steps[1].words(),"completed":completed}));
    }
    Ok(
        json!({"original_cubin":device::cubin_sha256(),"completion_cubin":sha256(device::COMPLETION_CUBIN),"fixtures":tested}),
    )
}
