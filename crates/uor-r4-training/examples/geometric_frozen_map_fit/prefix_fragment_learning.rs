//! One original-parent Prefix-only fragment construction at authenticated actual prefixes.
//! Existing retained geometric features and donor-specific native hard pools; no serving changes.
use super::context_cue_coadapt as shared;
use super::native_proposals as np;
use super::*;
use uor_r4_training::geometric_occurrence_consumer::source_realizer::{
    CueAngularWeights, PrefixAngularWeights,
};
const NAME: &str = "prefix.coefficients";
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Config {
    pub retained_intermediate_root: PathBuf,
    pub retained_probe_root: PathBuf,
    #[serde(default)]
    pub trajectory: Option<TrajectoryConfig>,
    #[serde(default)]
    pub joint: Option<JointConfig>,
    #[serde(default)]
    pub episode: Option<EpisodeConfig>,
}
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WitnessRoot {
    pub root: PathBuf,
    pub expected_report_sha256: String,
    pub expected_manifest_sha256: String,
}
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct JointConfig {
    pub p5_capture: WitnessRoot,
    pub p6_conditional_capture: WitnessRoot,
    pub retained_supplement_root: PathBuf,
    pub expected_supplement_report_sha256: String,
    pub expected_supplement_manifest_sha256: String,
}
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct EpisodePhase {
    pub position: usize,
    pub capture: WitnessRoot,
    pub original_prefix_inverse: bool,
    #[serde(default)]
    pub normalized_original: Option<NormalizedWitness>,
}
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NormalizedWitness {
    pub file: String,
    pub expected_sha256: String,
}
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct EpisodeConfig {
    pub input_index: usize,
    pub expected_id: String,
    pub expected_canonical_row_sha256: String,
    pub typed_authority: PathBuf,
    pub retained_projection: WitnessRoot,
    pub expected_typed_authority_sha256: String,
    pub phases: Vec<EpisodePhase>,
    pub retained_supplement_root: PathBuf,
    pub expected_supplement_report_sha256: String,
    pub expected_supplement_manifest_sha256: String,
}
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TrajectoryConfig {
    pub retained_prefix_learning_root: PathBuf,
    pub retained_supplement_root: PathBuf,
    pub expected_supplement_report_sha256: String,
    pub expected_supplement_manifest_sha256: String,
}
pub(super) fn validate_settings(a: &Args) -> Result<()> {
    if let Some(c) = &a.prefix_fragment_learning {
        replay_require(
            a.mode == Mode::JointContinuation
                && a.updates == 1
                && a.loss_scope == LossScope::All
                && a.prefix_artifact_check.is_none()
                && a.context_cue_coadapt.is_none()
                && a.prefix_context_credit.is_none()
                && a.context_path_credit.is_none()
                && a.readout_coadaptation.is_none()
                && a.reached_u.is_none()
                && a.prototype_compensation.is_none()
                && a.reference_replay.is_none()
                && a.retained_context_root.is_none()
                && !a.native_code_proposals
                && !a.reached_frontier_objective
                && !a.constrained_context_learning
                && !a.constrained_emission_learning
                && !a.categorical_action_learning
                && !a.categorical_action_only,
            "Prefix fragment requires exclusive original-parent one-pass joint mode",
        )?;
        replay_require(
            [
                c.joint.is_some(),
                c.trajectory.is_some(),
                c.episode.is_some(),
            ]
            .into_iter()
            .filter(|v| *v)
            .count()
                <= 1,
            "joint fresh credit excludes reused-gradient trajectory mode",
        )?;
        if let Some(e) = &c.episode {
            replay_require(
                e.input_index < 512
                    && !e.expected_id.is_empty()
                    && e.phases.len() == 10
                    && e.phases.iter().map(|p| p.position).collect::<BTreeSet<_>>()
                        == (5..15).collect()
                    && e.phases.iter().all(|p| {
                        [
                            &p.capture.expected_report_sha256,
                            &p.capture.expected_manifest_sha256,
                        ]
                        .iter()
                        .all(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit()))
                    })
                    && [
                        &e.expected_canonical_row_sha256,
                        &e.expected_typed_authority_sha256,
                    ]
                    .iter()
                    .all(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit())),
                "episode full coverage/identity hashes invalid",
            )?;
            replay_require(
                e.expected_supplement_report_sha256
                    == "0eddd892d5c1a188ee9816856853b672722c3f74ccd67a0f49cbb3e016b8a5d4"
                    && e.expected_supplement_manifest_sha256
                        == "5ae8d12f32ee68b2d1a7d82626305cc8fa0e30be1b118f89291b8ac116dad45d",
                "episode original supplement differs",
            )?;
        }
        if let Some(j) = &c.joint {
            for w in [&j.p5_capture, &j.p6_conditional_capture] {
                replay_require(
                    [&w.expected_report_sha256, &w.expected_manifest_sha256]
                        .iter()
                        .all(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit())),
                    "joint witness hashes invalid",
                )?;
            }
            replay_require(
                j.p5_capture.expected_report_sha256
                    == "166d286646411902257e5bb543cfc254248044e99d3290067e0cb6ce39b3d490"
                    && j.p5_capture.expected_manifest_sha256
                        == "68a71b01c5543f8b22d70e5abe2fc671b5dc867fff78e971219ed2fdd8389eff",
                "joint p5 recorded witness differs",
            )?;
            replay_require(
                j.expected_supplement_report_sha256
                    == "0eddd892d5c1a188ee9816856853b672722c3f74ccd67a0f49cbb3e016b8a5d4"
                    && j.expected_supplement_manifest_sha256
                        == "5ae8d12f32ee68b2d1a7d82626305cc8fa0e30be1b118f89291b8ac116dad45d",
                "joint supplement differs",
            )?;
        }
        if let Some(t) = &c.trajectory {
            replay_require(
                t.expected_supplement_report_sha256
                    == "0eddd892d5c1a188ee9816856853b672722c3f74ccd67a0f49cbb3e016b8a5d4"
                    && t.expected_supplement_manifest_sha256
                        == "5ae8d12f32ee68b2d1a7d82626305cc8fa0e30be1b118f89291b8ac116dad45d",
                "trajectory exact supplement authority differs",
            )?;
        }
        replay_require(
            fs::canonicalize(&a.checkpoint)?
                == fs::canonicalize(c.retained_intermediate_root.join("checkpoint-0001"))?
                && a.maximum_report_bytes
                    <= if c.trajectory.is_some() || c.joint.is_some() || c.episode.is_some() {
                        512 * 1024 * 1024
                    } else {
                        256 * 1024 * 1024
                    },
            "Prefix original parent/report admission differs",
        )?;
    }
    Ok(())
}
pub(super) fn policy() -> Value {
    json!({"schema":"uor-r4.prefix-fragment-learning/1","active_family":NAME,"coordinates":960,
 "initialization":"ORIGINAL Source9f/Cue/Prefix fractional masters/G4248/U82ae; no negative Context/Cue adoption",
 "gradient":"one aggregate from18 backwards; only Prefix master gradients extracted/proposed; frozen Generate graph transports direct gather and detached conditional-donor contrast, not hard argmax derivative",
 "construction":"one frozen actual-master-delta preferred adjacent960 pass; strict current18 CE descent AND all17 original winners per accepted transaction; no refill/revisit/rerank",
 "final_gate":"strict ORIGINAL18 CE descent, exact task probability increase, all17 original winners",
 "fragment_correction":"actual pooled target267 winner is a distinct required qualification",
 "strict_ce_tolerance":"1e-10*(1+abs(current_or_original_CE))","optimizer_updates":0,"context_gradient":"NOT_RUN","cue_gradient":"NOT_RUN","serving_changes":false})
}
fn transaction_accept(current: &Value, staged: &Value) -> Result<bool> {
    let before = current["combined"]
        .as_f64()
        .ok_or_else(|| bad("Prefix current CE missing"))?;
    let after = staged["combined"]
        .as_f64()
        .ok_or_else(|| bad("Prefix staged CE missing"))?;
    replay_require(
        before.is_finite() && after.is_finite(),
        "Prefix CE nonfinite",
    )?;
    Ok(after < before - 1e-10 * (1. + before.abs()) && staged["correct_reference_frames"] == 17)
}

fn trajectory_policy() -> Value {
    json!({"schema":"uor-r4.prefix-trajectory-learning/1","active_family":NAME,"initialization":"ORIGINAL selected parent actual fractionalPrefix/Source9f/Cue/G4248/U82ae","gradient":"REUSED authenticated original18 weighted raw960 gradient and frozen actualdelta order; zero new backward","construction":"one960 preferredadjacent pass; strictcurrent18CE tolerance1e-10*(1+abs(old)) +17 original objective winners +all380 original correct-prefix winners eachacceptedtransaction; no rerank/refill/revisit","guard_population":"generic ALL selected parent canonical-correct actualprefixes before firstdivergence, including completeEOS;377full+1existingfull+2supplements","final":"strictoriginal18CE and exacttaskprobability increase plus380/17retention; actual267chosen separately; cheapownfeedback conditional, not implicit qualification","frozen":"Source/Context/Potential/Cue/joint/G/map/bridge/Ucoefficients and all Prefix featurestates; Gvectors depend on earliestBASEphysicaldonor","selected_model":false})
}
fn correct_prefix_terms(raw: &Value, index: usize, id: &str) -> Result<Vec<Value>> {
    replay_require(raw["id"] == id, "trajectory raw ID differs")?;
    let ids: Vec<u32> = shared::dec(&raw["generated_ids"])?;
    let targets: Vec<u32> = shared::dec(&raw["canonical_target_ids_labels_only"])?;
    let steps = raw["generation"]
        .as_array()
        .ok_or_else(|| bad("trajectory generation absent"))?;
    replay_require(
        steps.len() == ids.len()
            && raw["eos"] == json!(ids.last() == Some(&1))
            && !ids[..ids
                .len()
                .saturating_sub(usize::from(ids.last() == Some(&1)))]
                .contains(&1),
        "trajectory step/EOS coverage differs",
    )?;
    for (position, step) in steps.iter().enumerate() {
        replay_require(
            step["actual_prefix_ids"] == json!(&ids[..position])
                && step["pool"]["summary"]["chosen_token_id"] == ids[position],
            "trajectory actual winner/prefix chain differs",
        )?;
    }
    Ok(ids.iter().zip(&targets).enumerate().take_while(|(_, (a,b))|a==b).map(|(position,(_,target))|json!({"index":index,"position":position,"id":id,"target":target,"actual_prefix_ids":&ids[..position]})).collect())
}
fn guard_cache_index(objective_count: usize, guard_index: usize) -> Result<usize> {
    objective_count
        .checked_add(guard_index)
        .ok_or_else(|| bad("objective/guard cache namespace overflow"))
}
fn guard_digest(frames: &[shared::Frame], pools: &[shared::Pool]) -> Result<String> {
    replay_require(
        frames.len() == pools.len(),
        "trajectory guard digest shape differs",
    )?;
    Ok(sha256_bytes(&serde_json::to_vec(&frames.iter().zip(pools).map(|(f,p)|json!({"index":f.input,"position":f.position,"target":f.target,"donor":p.donor,"pool":p.trace.summary})).collect::<Vec<_>>())?))
}
fn guard_incidence(frames: &[shared::Frame]) -> Result<Vec<Vec<usize>>> {
    key_incidence(frames.iter().map(|f| &f.cue_keys))
}
fn key_incidence<'a>(
    frames: impl Iterator<Item = &'a Vec<Vec<Option<usize>>>>,
) -> Result<Vec<Vec<usize>>> {
    let mut rows = vec![Vec::new(); 960];
    for (i, candidates) in frames.enumerate() {
        let mut keys = std::collections::BTreeSet::new();
        for row in candidates {
            replay_require(
                row.len() == 8 && row.iter().all(Option::is_some),
                "trajectory Prefix incidence must be unmasked eight lanes",
            )?;
            for key in row.iter().flatten() {
                replay_require(*key < 960, "trajectory incidence key outside960")?;
                keys.insert(*key);
            }
        }
        for key in keys {
            rows[key].push(i);
        }
    }
    Ok(rows)
}
fn stage_affected<T>(
    affected: &[usize],
    mut evaluate: impl FnMut(usize) -> Result<T>,
) -> Result<Vec<(usize, T)>> {
    affected.iter().map(|i| Ok((*i, evaluate(*i)?))).collect()
}
fn compact_guard_pool(mut pool: shared::Pool) -> shared::Pool {
    pool.trace.actions = Vec::new();
    pool
}
fn pools_numeric_bytes(pools: &[shared::Pool]) -> u64 {
    pools
        .iter()
        .map(|p| {
            (std::mem::size_of_val(p)
                + std::mem::size_of_val(&p.generate[..])
                + std::mem::size_of_val(&p.copy[..])
                + std::mem::size_of_val(&p.base_copy[..])
                + std::mem::size_of_val(&p.post[..])
                + std::mem::size_of_val(&p.trace.actions[..])
                + std::mem::size_of_val(&p.trace.token_masses[..])) as u64
        })
        .sum()
}
fn compact_guard_row(f: &shared::Frame, p: &shared::Pool, index: usize) -> Result<Value> {
    Ok(
        json!({"guard_index":index,"input_index":f.input,"position":f.position,"required_original_winner":f.target,"chosen":p.trace.summary.chosen_token_id,"donor":p.donor,"pool":p.trace.summary,
     "generate_q24_sha256":sha256_bytes(&serde_json::to_vec(&p.generate)?),"copy_q24_sha256":sha256_bytes(&serde_json::to_vec(&p.copy)?)}),
    )
}
fn prepare_trajectory_guards(
    a: &Args,
    c: &Config,
    tc: &TrajectoryConfig,
    frames: &[shared::Frame],
    objective_spec: Option<&shared::ObjectiveSpec>,
    p: &ContinuationParent,
    parent: &[f32],
    cache: &mut shared::DonorCache,
    reducer: &mut NativeVocabularyActions,
) -> Result<(Vec<shared::Frame>, Vec<shared::Pool>, Value)> {
    let report = shared::sealed(
        &c.retained_intermediate_root,
        shared::P_REPORT,
        shared::P_SEAL,
    )?;
    let supplement = shared::sealed(
        &tc.retained_supplement_root,
        &tc.expected_supplement_report_sha256,
        &tc.expected_supplement_manifest_sha256,
    )?;
    replay_require(
        supplement["status"] == "COMPLETED"
            && supplement["endpoint_kind"] == "selected_original_trajectory_supplement"
            && supplement["frame_count"] == 2
            && supplement["source_binding"] == json!(p.binding)
            && supplement["generate_sha256"] == shared::G_SHA
            && supplement["continuation_sha256"] == shared::U_SHA,
        "trajectory supplement epoch/cohort differs",
    )?;
    let fullfile = c
        .retained_intermediate_root
        .join("exported-candidate-protected-pools.json");
    replay_require(
        sha256_file(&fullfile)?
            == "fcde63e9fc12f218b866b71323601485db7f93fd1b69a54beec91e87a7b84fad",
        "trajectory original377 pool pin differs",
    )?;
    // Deserialize once: no intermediate Value clone of the full377 tree.
    let full: Vec<Value> = serde_json::from_reader(fs::File::open(&fullfile)?)?;
    replay_require(full.len() == 377, "trajectory original377 count differs")?;
    let mut saved = BTreeMap::new();
    for row in full {
        let input = shared::idx(&row["term"]["index"])?;
        let position = shared::idx(&row["term"]["position"])?;
        replay_require(
            saved.insert((input, position), row).is_none(),
            "duplicate original guard identity",
        )?;
    }
    let mut additions = BTreeMap::new();
    for f in frames {
        if !saved.contains_key(&(f.input, f.position))
            && !objective_spec.map_or(f.input == 245 && f.position == 4, |spec| {
                spec.is_task(f)
                    && !spec.roles.as_ref().is_some_and(|rs| {
                        rs.iter()
                            .any(|r| !r.task && r.input == f.input && r.position == f.position)
                    })
            })
        {
            replay_require(
                additions
                    .insert((f.input, f.position), f.native.clone())
                    .is_none(),
                "duplicate objective guard addition",
            )?;
        }
    }
    for row in supplement["frames"]
        .as_array()
        .ok_or_else(|| bad("supplement frames absent"))?
    {
        let input = shared::idx(&row["input_index"])?;
        let position = shared::idx(&row["position"])?;
        let leaf = row["file"]
            .as_str()
            .ok_or_else(|| bad("supplement file absent"))?;
        replay_require(
            Path::new(leaf).components().count() == 1,
            "supplement leaf invalid",
        )?;
        let file = tc.retained_supplement_root.join(leaf);
        replay_require(
            sha256_file(&file)?
                == row["sha256"]
                    .as_str()
                    .ok_or_else(|| bad("supplement SHA absent"))?,
            "supplement raw hash differs",
        )?;
        let v = read(&file)?;
        replay_require(
            v["input_index"] == input
                && v["position"] == position
                && v["capture_target_free"] == true
                && v["label_access_before_capture"] == false
                && v["controls"]["generate_only"] == "NOT_RUN"
                && v["controls"]["copy_u_removed"] == "NOT_RUN"
                && !saved.contains_key(&(input, position))
                && additions.insert((input, position), v).is_none(),
            "supplement frame identity/duplicate differs",
        )?;
    }
    replay_require(
        additions.len() == 3,
        "trajectory correct-prefix supplement population differs",
    )?;
    let mut guards = Vec::new();
    let mut pools = Vec::new();
    let mut authority = Vec::new();
    let mut original_native_serialized_bytes = 0u64;
    let episodes = read(&a.training_inputs)?;
    let summaries = report["final_evaluation"]["rows"]
        .as_array()
        .ok_or_else(|| bad("selected trajectory rows absent"))?;
    replay_require(
        summaries.len() == 512,
        "selected trajectory512 coverage differs",
    )?;
    for (input, row) in summaries.iter().enumerate() {
        let id = row["id"]
            .as_str()
            .ok_or_else(|| bad("trajectory ID absent"))?;
        replay_require(
            episodes["cases"][input]["id"] == id,
            "trajectory input ID differs",
        )?;
        let leaf = row["row_file"]
            .as_str()
            .ok_or_else(|| bad("trajectory rawfile absent"))?;
        replay_require(
            Path::new(leaf).components().count() == 1,
            "trajectory leaf invalid",
        )?;
        let file = c.retained_intermediate_root.join(leaf);
        let rawhash = sha256_file(&file)?;
        replay_require(
            rawhash
                == row["row_sha256"]
                    .as_str()
                    .ok_or_else(|| bad("trajectory raw SHA absent"))?,
            "trajectory raw receipt differs",
        )?;
        let raw = read(&file)?;
        for term in correct_prefix_terms(&raw, input, id)? {
            let position = shared::idx(&term["position"])?;
            let target = term["target"]
                .as_u64()
                .ok_or_else(|| bad("guard target absent"))? as u32;
            let prefix: Vec<u32> = shared::dec(&term["actual_prefix_ids"])?;
            let (mut native, source) = if let Some(full) = saved.remove(&(input, position)) {
                replay_require(
                    full["id"] == id
                        && full["actual_prefix_ids"] == term["actual_prefix_ids"]
                        && full["term"]["target"] == target
                        && full["pool"]["chosen_token_id"] == target,
                    "original full guard actual binding differs",
                )?;
                let g: Vec<i64> = shared::dec(&full["generate_q24"])?;
                let copy: Vec<i64> = shared::dec(&full["copy_q24"])?;
                let ids: Vec<u32> = shared::dec(&full["copy_ids"])?;
                let trace = reducer.reduce_trace(&g, &ids, &copy)?;
                replay_require(
                    json!(trace.summary) == full["pool"]
                        && json!(trace.token_masses) == full["token_masses"],
                    "original full guard pool replay differs",
                )?;
                (
                    json!({"bank_trace":full["bank_trace"],"bridge":full["bridge"],"generate_q24":g,"copy_q24":copy,"copy_ids":ids,"pool":{"summary":trace.summary,"token_masses":trace.token_masses},
                 "continuation":{"state_codes":raw["generation"][position]["continuation"]["state_codes"],"query_tokens":raw["generation"][position]["continuation"]["query_tokens"],"actual_prefix_tokens":raw["generation"][position]["continuation"]["actual_prefix_tokens"],"delta_scores_q24":full["frozen_u_q24"]}}),
                    "retained_full377",
                )
            } else {
                (
                    additions.remove(&(input, position)).ok_or_else(|| {
                        bad("selected correct prefix lacks complete native witness")
                    })?,
                    "retained_addition",
                )
            };
            native["pool"]
                .as_object_mut()
                .ok_or_else(|| bad("guard pool authority absent"))?
                .remove("actions");
            native["pool_action_trace"] = json!("OMITTED_RECONSTRUCTIBLE_FROM_COMPLETE_SCORES; complete raw scores/masses/summary retained; no action-record Value allocation");
            replay_require(
                native["pool"]["summary"]["chosen_token_id"] == target
                    && native["pool"]["summary"] == raw["generation"][position]["pool"]["summary"],
                "guard original fullsummary differs from actual selected output",
            )?;
            let delta: Vec<i64> = shared::dec(&native["continuation"]["delta_scores_q24"])?;
            replay_require(
                sha256_bytes(&serde_json::to_vec(&delta)?)
                    == raw["generation"][position]["continuation"]["delta_scores_q24_sha256"]
                        .as_str()
                        .ok_or_else(|| bad("guard U SHA absent"))?
                    && native["continuation"]["state_codes"]
                        == raw["generation"][position]["continuation"]["state_codes"],
                "guard U actual state/vector differs",
            )?;
            // Guard membership never contributes additional objective weight.
            let weight = 0.;
            let mut f = shared::parse_saved_native_frame(
                input,
                position,
                id.into(),
                prefix,
                target,
                weight,
                &native,
                p,
                true,
            )?;
            replay_require(
                f.trace.query.token_ids
                    == shared::dec::<Vec<u32>>(&episodes["cases"][input]["query_ids"])?,
                "guard query packet differs",
            )?;
            let pool = shared::score(
                guard_cache_index(frames.len(), guards.len())?,
                &f,
                parent,
                parent,
                p,
                cache,
                reducer,
            )?;
            replay_require(
                json!(pool.generate) == native["generate_q24"]
                    && json!(pool.copy) == native["copy_q24"]
                    && json!(pool.trace.summary) == native["pool"]["summary"]
                    && json!(pool.trace.token_masses) == native["pool"]["token_masses"]
                    && pool.trace.summary.chosen_token_id == target,
                "guard baseline donor/native fullvector parity differs",
            )?;
            native["pool"]
                .as_object_mut()
                .ok_or_else(|| bad("guard saved pool object absent"))?
                .remove("actions");
            original_native_serialized_bytes = original_native_serialized_bytes
                .checked_add(serde_json::to_vec(&native)?.len() as u64)
                .ok_or_else(|| bad("guard serialization size overflow"))?;
            authority.push(json!({"input_index":input,"position":position,"id":id,"required_original_winner":target,"actual_prefix_ids":f.prefix,"selected_raw_file":leaf,"selected_raw_sha256":rawhash,"witness_source":source}));
            // Keep only occurrence identity and U encoder receipts; numerical vectors live in typed fields.
            native = json!({"bank_trace":{"cue_bank":{"bank":{"candidates":native["bank_trace"]["cue_bank"]["bank"]["candidates"]}}},"continuation":{"state_codes":native["continuation"]["state_codes"],"query_tokens":native["continuation"]["query_tokens"],"actual_prefix_tokens":native["continuation"]["actual_prefix_tokens"]}});
            f.native = native;
            f.prefix_trace = None;
            guards.push(f);
            pools.push(compact_guard_pool(pool));
        }
    }
    replay_require(
        guards.len() == 380 && saved.is_empty() && additions.is_empty(),
        "complete selected380 guard population differs",
    )?;
    let receipt = json!({"guards":380,"original_native_serialized_bytes":original_native_serialized_bytes,"original_full377_sha256":"fcde63e9fc12f218b866b71323601485db7f93fd1b69a54beec91e87a7b84fad","selected_report_sha256":shared::P_REPORT,"selected_manifest_sha256":shared::P_SEAL,"supplement_report_sha256":tc.expected_supplement_report_sha256,"supplement_manifest_sha256":tc.expected_supplement_manifest_sha256,"terms":authority,"baseline_digest":guard_digest(&guards,&pools)?,"selection":"ALL canonical-correct actual prefixes before first divergence, including original complete EOS; generic identity union, no added runtime labels"});
    Ok((guards, pools, receipt))
}

fn joint_policy() -> Value {
    json!({"schema":"uor-r4.prefix-joint-fragment-learning/1","active_family":NAME,"initialization":"ORIGINAL selected Source9f/Cue/Prefix c2e8 actual fractional masters/G4248/U82ae","objective":"three canonical-conditional ordered taskphases each1/3 +17 original references each1/17; zero-weight380guards","gradient":"fresh20 streamed backwards/one weighted aggregate960; direct Prefix gather and detached conditionaldonor local surrogate; only Prefix extracted/proposed, frozen Generate graph carries autodiff","construction":"one frozen960 actualfractional-master adjacent nativeQ4 pass; strictcurrent20CE+17refs+all380 original winners everyaccept","final":"strict ORIGINAL20combined and three-phase word CE descent +all3phasewinners +17refs +380guards; actualtypedwholeanswer/EOS and8retention separate","strict_ce_tolerance":"1e-10*(1+abs(current_or_original_CE))","optimizer_updates":0,"selected_model":false,"serving_changes":false})
}
fn mode_objective(
    f: &[shared::Frame],
    p: &[shared::Pool],
    s: Option<&shared::ObjectiveSpec>,
) -> Result<Value> {
    if let Some(s) = s {
        shared::objective_for_spec(f, p, s)
    } else {
        shared::objective(f, p)
    }
}
fn mode_gate(b: &Value, v: &Value, s: Option<&shared::ObjectiveSpec>) -> Result<Value> {
    if s.is_some_and(|s| s.roles.is_some()) {
        shared::episode_gate(b, v)
    } else if s.is_some() {
        shared::joint_gate(b, v)
    } else {
        shared::gate(b, v)
    }
}
fn witness_frame(w: &WitnessRoot, input: usize, position: usize) -> Result<(Value, Value)> {
    let report = shared::sealed(
        &w.root,
        &w.expected_report_sha256,
        &w.expected_manifest_sha256,
    )?;
    replay_require(
        report["status"] == "COMPLETED"
            && report["source_binding"]["metadata_sha256"] == shared::SOURCE
            && report["generate_sha256"] == shared::G_SHA
            && report["continuation_sha256"] == shared::U_SHA,
        "joint witness artifact epoch differs",
    )?;
    let matches = report["frames"]
        .as_array()
        .ok_or_else(|| bad("joint witness frame list absent"))?
        .iter()
        .filter(|r| r["input_index"] == input && r["position"] == position)
        .collect::<Vec<_>>();
    replay_require(
        matches.len() == 1,
        "joint witness identity coverage differs",
    )?;
    let row = matches[0];
    let leaf = row["file"]
        .as_str()
        .ok_or_else(|| bad("joint witness file absent"))?;
    replay_require(
        Path::new(leaf).components().count() == 1,
        "joint witness leaf invalid",
    )?;
    let file = w.root.join(leaf);
    replay_require(
        sha256_file(&file)?
            == row["sha256"]
                .as_str()
                .ok_or_else(|| bad("joint witness hash absent"))?,
        "joint raw witness hash differs",
    )?;
    let v = read(&file)?;
    replay_require(
        v["capture_target_free"] == true
            && v["label_access_before_capture"] == false
            && v["input_index"] == input
            && v["position"] == position,
        "joint targetfree frame authority differs",
    )?;
    Ok((v, report))
}
fn replace_head_adjustment(score: i64, old: i64, new: i64) -> Result<i64> {
    score
        .checked_sub(old)
        .and_then(|x| x.checked_add(new))
        .ok_or_else(|| bad("derived bank score overflow"))
}
fn original_phase_native(
    a: &Args,
    j: &JointConfig,
    p: &ContinuationParent,
    active: &PrefixAngularWeights,
    parent: &[f32],
    position: usize,
) -> Result<Value> {
    let w = if position == 5 {
        &j.p5_capture
    } else {
        &j.p6_conditional_capture
    };
    original_captured_phase(a, w, p, active, parent, 245, position, position == 5)
}
fn original_captured_phase(
    a: &Args,
    w: &WitnessRoot,
    p: &ContinuationParent,
    active: &PrefixAngularWeights,
    parent: &[f32],
    input: usize,
    position: usize,
    derived: bool,
) -> Result<Value> {
    let (mut v, report) = witness_frame(w, input, position)?;
    replay_require(
        if derived {
            matches!(
                report["endpoint_kind"].as_str(),
                Some("unselected_prefix_trajectory" | "unselected_prefix_candidate")
            )
        } else {
            report["endpoint_kind"] == "selected_original_canonical_conditional"
        },
        "phase capture conditional/inverse scope differs",
    )?;
    let raw = read(
        &a.checkpoint
            .parent()
            .ok_or_else(|| bad("parent root absent"))?
            .join(format!("development-0001-row-{input:04}.json")),
    )?;
    let prefix: Vec<u32> = shared::dec(&raw["canonical_target_ids_labels_only"])?;
    replay_require(
        v["id"] == raw["id"]
            && v["actual_prefix_ids"] == json!(&prefix[..position])
            && (derived || v["saved_canonical_native"] == raw["canonical"][position]["native"]),
        "joint canonical phase prefix/authority differs",
    )?;
    if derived {
        // Finite reconstruction only: the saved Context feature states are unaffected by Prefix coefficients.
        replay_require(
            parent.len() == 960 && parent.iter().all(|x| x.is_finite()),
            "derived original Prefix masters invalid",
        )?;
        let q = parent
            .iter()
            .map(|x| (*x * 4.).round().clamp(-7., 7.) as i8)
            .collect::<Vec<_>>();
        let ng = NativeGeometricGenerate::from_bytes(&p.generate, p.integer.binding())?;
        let bins: Vec<Vec<u8>> = shared::dec(&v["bank_trace"]["prefix"]["angular_indices"])?;
        let relative: Vec<Vec<u8>> = shared::dec(&v["bank_trace"]["prefix"]["relative_roots"])?;
        let response: Vec<u8> = shared::dec(&v["bank_trace"]["prefix"]["response"]["states"])?;
        let source_indices: Vec<usize> =
            shared::dec(&v["bank_trace"]["prefix"]["candidate_source_indices"])?;
        let offsets: Vec<usize> = shared::dec(&v["bank_trace"]["prefix"]["candidate_offsets"])?;
        let ids: Vec<u32> = shared::dec(&v["copy_ids"])?;
        replay_require(
            bins.len() == 8
                && relative.len() == 8
                && response.len() == 8
                && source_indices.len() == ids.len()
                && offsets.len() == ids.len()
                && bins.iter().all(|r| r.len() == ids.len()),
            "derived Prefix feature shape differs",
        )?;
        let oldheads: Vec<Vec<i64>> = shared::dec(&v["bank_trace"]["prefix"]["copy_q24"])?;
        let mut heads = vec![vec![0i64; ids.len()]; 2];
        for k in 0..ids.len() {
            let states: Vec<u8> = shared::dec(
                &v["bank_trace"]["prefix"]["sources"][source_indices[k]]["states_before"]
                    [offsets[k]],
            )?;
            replay_require(states.len() == 8, "derived Prefix source width differs")?;
            for l in 0..8 {
                let r = ng
                    .algebra()
                    .compose(ng.algebra().inverse(response[l])?, states[l])?;
                replay_require(
                    r == relative[l][k] && r == bins[l][k] && bins[l][k] < 120,
                    "derived directed Prefix relative/bin differs",
                )?;
                heads[l / 4][k] = heads[l / 4][k]
                    .checked_add(i64::from(q[l * 120 + usize::from(bins[l][k])]) << 22)
                    .ok_or_else(|| bad("derived Prefix head overflow"))?;
            }
        }
        replay_require(
            oldheads.len() == 2 && oldheads.iter().all(|h| h.len() == ids.len()),
            "derived old Prefix heads differ",
        )?;
        let exp = p
            .exp
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect::<Vec<_>>();
        let mut head_reducer =
            uor_r4_integer::geometric_read::NativeGeometricRead::new(128, 1, &exp)?;
        for h in 0..2 {
            let head = &mut v["bank_trace"]["cue_bank"]["bank"]["heads"][h];
            let mut scores: Vec<i64> = shared::dec(&head["scores_q24"])?;
            replay_require(scores.len() == ids.len(), "derived bank head width differs")?;
            for k in 0..ids.len() {
                scores[k] = replace_head_adjustment(scores[k], oldheads[h][k], heads[h][k])?;
            }
            let no_read = head["no_read_q24"]
                .as_i64()
                .ok_or_else(|| bad("derived no-read score absent"))?;
            let reduction =
                head_reducer.reduce(&scores, &vec![0; ids.len()], no_read, &vec![0; ids.len()])?;
            head["weights_q31"] = json!(reduction.occurrence_weights_q31);
            head["total_weight_q31"] = json!(reduction.total_weight_q31);
            head["no_read_weight_q31"] = json!(reduction.no_read_weight_q31);
            head["scores_q24"] = json!(scores);
        }
        let bank = &v["bank_trace"]["cue_bank"]["bank"];
        let bank_scores = bank["heads"]
            .as_array()
            .ok_or_else(|| bad("derived bank heads absent"))?
            .iter()
            .map(|h| shared::dec::<Vec<i64>>(&h["scores_q24"]))
            .collect::<Result<Vec<_>>>()?;
        let period: Vec<i64> = shared::dec(&bank["period_q24"])?;
        let no_read = bank["heads"]
            .as_array()
            .ok_or_else(|| bad("derived bank heads absent"))?
            .iter()
            .map(|h| {
                h["no_read_q24"]
                    .as_i64()
                    .ok_or_else(|| bad("derived no-read absent"))
            })
            .collect::<Result<Vec<_>>>()?;
        replay_require(
            bank_scores.len() == 2 && period.len() == 2 && no_read.len() == 2,
            "derived source action head dimensions differ",
        )?;
        let hs = (0..2)
            .map(
                |h| uor_r4_integer::geometric_source_actions::ActionHeadScores {
                    copy_q24: &bank_scores[h],
                    period_q24: period[h],
                    stop_q24: no_read[h],
                },
            )
            .collect::<Vec<_>>();
        let mut action_reducer =
            uor_r4_integer::geometric_source_actions::NativeSourceActions::new(
                p.integer.binding().clone(),
                2,
                &exp,
            )?;
        v["bank_trace"]["cue_bank"]["bank"]["actions"] = json!(action_reducer.reduce(&ids, &hs)?);
        v["bank_trace"]["prefix"]["copy_q24"] = json!(heads);
        v["bank_trace"]["prefix"]["metadata"] =
            read(&a.checkpoint.join("prefix/native-metadata.json"))?;
        replay_require(
            v["bank_trace"]["prefix"]["metadata"]["potential_packed_sha256"]
                == sha256_bytes(&active.packed_coefficients()?),
            "derived Prefix native metadata hash differs",
        )?;
        let u: Vec<i64> = shared::dec(&v["continuation"]["delta_scores_q24"])?;
        let base = (0..ids.len())
            .map(|k| {
                v["bank_trace"]["cue_bank"]["bank"]["heads"]
                    .as_array()
                    .ok_or_else(|| bad("derived heads absent"))?
                    .iter()
                    .try_fold(0i64, |s, h| {
                        s.checked_add(
                            h["scores_q24"][k]
                                .as_i64()
                                .ok_or_else(|| bad("derived score absent"))?,
                        )
                        .ok_or_else(|| bad("derived head sum overflow"))
                    })
            })
            .collect::<Result<Vec<_>>>()?;
        v["base_copy_q24"] = json!(base);
        v["copy_q24"] = json!(base
            .iter()
            .zip(&ids)
            .map(|(b, id)| b
                .checked_add(u[*id as usize])
                .ok_or_else(|| bad("derived Copy U overflow")))
            .collect::<Result<Vec<_>>>()?);
        let target = raw["canonical"][position]["target_label_only"]
            .as_u64()
            .ok_or_else(|| bad("phase target absent"))? as u32;
        let f = shared::parse_saved_native_frame(
            input,
            position,
            raw["id"]
                .as_str()
                .ok_or_else(|| bad("phase ID absent"))?
                .into(),
            prefix[..position].to_vec(),
            target,
            1. / 3.,
            &v,
            p,
            true,
        )?;
        let mut cache = shared::DonorCache::new(p);
        let mut reducer = NativeVocabularyActions::new(p.integer.binding().clone(), &p.exp)?;
        let pool = shared::score(0, &f, parent, parent, p, &mut cache, &mut reducer)?;
        // Rebuild bridge from the actual earliest BASE physical donor, not a saved Generate shortcut.
        let bridge = NativeGeometricReadStateBridge::from_bytes(&p.bridge, p.integer.binding())?;
        let mut post = vec![uor_r4_integer::h4_tables::H4Code::IDENTITY; 8];
        let mut actions = post.clone();
        let mut scores = vec![0; 960];
        let mut counts = BridgeReadCounts::default();
        bridge.apply_into(
            &f.query,
            &f.sources[pool.donor],
            &mut post,
            &mut actions,
            &mut scores,
            &mut counts,
        )?;
        v["bridge"]["selected_ordinal"] = json!(pool.donor);
        v["bridge"]["selected_candidate"] =
            v["bank_trace"]["cue_bank"]["bank"]["candidates"][pool.donor].clone();
        v["bridge"]["source_state"] = json!(f.sources[pool.donor]
            .iter()
            .map(|x| x.index())
            .collect::<Vec<_>>());
        v["bridge"]["action_codes"] = json!(actions.iter().map(|x| x.index()).collect::<Vec<_>>());
        v["bridge"]["action_scores_q24"] = json!(scores);
        v["bridge"]["counts"] = json!(counts);
        v["post_state"] = json!(post.iter().map(|x| x.index()).collect::<Vec<_>>());
        v["generate_q24"] = json!(pool.generate);
        v["copy_q24"] = json!(pool.copy);
        v["pool"] = json!(pool.trace);
        v["base_generate_q24"] = json!(pool
            .generate
            .iter()
            .zip(&u)
            .map(|(g, u)| g - u)
            .collect::<Vec<_>>());
        v["original_derivation"] = json!({"scope":"DERIVED_FROM_CAPTURE_AND_ORIGINAL_PACKED_PREFIX; no Context encoder or native step","capture_root":w.root,"capture_report_sha256":w.expected_report_sha256,"all8_prefix_lanes_inverted":true,"heads_and_occurrence_weights_rebuilt":true,"candidate_factors":"unchanged Generate/U artifact arithmetic, independently compared to original canonical authority"});
    }
    validate_original_canonical(&v, &raw, position)?;
    write(a, &format!("original-joint-phase-{position:02}.json"), &v)?;
    Ok(v)
}
// Compact captures expose counts; legacy canonical authorities expose field_counts.
fn known_continuation_field_counts(v: &Value) -> Result<&Value> {
    let legacy = v.get("field_counts").filter(|x| !x.is_null());
    let compact = v.get("counts").filter(|x| !x.is_null());
    let value = match (legacy, compact) {
        (Some(a), Some(b)) => {
            replay_require(a == b, "conflicting continuation field counts")?;
            a
        }
        (Some(a), None) | (None, Some(a)) => a,
        (None, None) => return Err(bad("continuation field counts absent")),
    };
    replay_require(value.is_object(), "continuation field counts malformed")?;
    Ok(value)
}
fn validate_original_canonical(v: &Value, raw: &Value, position: usize) -> Result<()> {
    let n = &raw["canonical"][position]["native"];
    let g: Vec<i64> = shared::dec(&v["generate_q24"])?;
    let u: Vec<i64> = shared::dec(&v["continuation"]["delta_scores_q24"])?;
    replay_require(
        v["pool"]["summary"] == n["pool"]["summary"]
            && v["post_state"] == n["post_state_codes"]
            && v["copy_ids"] == n["copy_token_ids"]
            && sha256_bytes(&serde_json::to_vec(&g)?)
                == n["generate_raw_scores_sha256"]
                    .as_str()
                    .ok_or_else(|| bad("canonical G digest absent"))?
            && v["continuation"]["state_codes"] == n["continuation"]["state_codes"]
            && sha256_bytes(&serde_json::to_vec(&u)?)
                == n["continuation"]["delta_scores_q24_sha256"]
                    .as_str()
                    .ok_or_else(|| bad("canonical U digest absent"))?,
        "derived/original conditional complete canonical parity differs",
    )?;
    let target = raw["canonical"][position]["target_label_only"]
        .as_u64()
        .ok_or_else(|| bad("canonical target absent"))? as u32;
    let targetmass = v["pool"]["token_masses"]
        .as_array()
        .ok_or_else(|| bad("phase masses absent"))?
        .iter()
        .find(|x| x["token_id"] == target)
        .ok_or_else(|| bad("phase target mass absent"))?;
    replay_require(
        targetmass["weight_q31"] == raw["canonical"][position]["native_target_mass"]
            && v["pool"]["summary"]["total_weight_q31"]
                == raw["canonical"][position]["native_denominator"],
        "phase canonical exact target mass/denominator differs",
    )?;
    let n = &raw["canonical"][position]["native"]["continuation"];
    for key in [
        "query_tokens",
        "actual_prefix_tokens",
        "encoding_coefficient_reads",
    ] {
        replay_require(
            !v["continuation"][key].is_null()
                && !n[key].is_null()
                && v["continuation"][key] == n[key],
            "original canonical U count authority differs",
        )?;
    }
    replay_require(
        known_continuation_field_counts(&v["continuation"])? == known_continuation_field_counts(n)?,
        "original canonical U field counts differ",
    )?;
    Ok(())
}
fn prepare_joint_phases(
    a: &Args,
    j: &JointConfig,
    p: &ContinuationParent,
    active: &PrefixAngularWeights,
    parent: &[f32],
    frames: &mut Vec<shared::Frame>,
) -> Result<()> {
    for f in frames.iter_mut() {
        f.weight = if f.input == 245 && f.position == 4 {
            1. / 3.
        } else {
            1. / 17.
        };
    }
    let raw = read(
        &a.checkpoint
            .parent()
            .ok_or_else(|| bad("parent root absent"))?
            .join("development-0001-row-0245.json"),
    )?;
    let canonical: Vec<u32> = shared::dec(&raw["canonical_target_ids_labels_only"])?;
    for position in [5, 6] {
        let native = original_phase_native(a, j, p, active, parent, position)?;
        let target = canonical[position];
        frames.push(shared::parse_saved_native_frame(
            245,
            position,
            raw["id"]
                .as_str()
                .ok_or_else(|| bad("task ID absent"))?
                .into(),
            canonical[..position].to_vec(),
            target,
            1. / 3.,
            &native,
            p,
            true,
        )?);
    }
    frames.sort_by_key(|f| {
        if f.input == 245 && (4..=6).contains(&f.position) {
            (0, f.position)
        } else {
            (1, f.input * 32 + f.position)
        }
    });
    replay_require(frames.len() == 20, "joint20 coverage differs")?;
    write(
        a,
        "joint-objective-authority.json",
        &json!({"tasks":[{"input_index":245,"position":4,"target":267,"weight":1./3.},{"input_index":245,"position":5,"target":307,"weight":1./3.},{"input_index":245,"position":6,"target":397,"weight":1./3.}],"references":17,"reference_weight":1./17.,"scope":"canonical conditional teacher-forced development; not actual original ownfeedback","p5_witness":j.p5_capture,"p6_witness":j.p6_conditional_capture,"ordered_terms":frames.iter().map(|f|json!({"input_index":f.input,"position":f.position,"id":f.id,"target":f.target,"weight":f.weight,"prefix":f.prefix})).collect::<Vec<_>>()}),
    )?;
    Ok(())
}
fn episode_policy() -> Value {
    json!({"schema":"uor-r4.prefix-episode-progression-learning/1","active_family":NAME,
    "objective":"complete15 canonical phases including EOS each1/15 plus17 unchanged roles each1/17;32 roles31 unique gradient states;380 zero-loss guards",
    "gradient":"fresh31 coalesced weighted backwards; one aggregate960; frozen Generate graph transports Prefix gather/detached donor surrogate; no fabricated32 role gradients",
    "construction":"one960 frozen preferred adjacentQ4 pass; strictcurrent normalized CE +17/380 winners everyaccept",
    "final":"strict original episode and combinedCE +all15 inclEOS/17/380; actual9 typed wholeanswer/EOS separate",
    "optimizer_updates":0,"selected_model":false,"occurrence_auxiliary":"NOT_RUN","serving_changes":false})
}
// Streaming extraction holds at most ONE legacy row Value, never the full203MB tree.
fn episode_protected_rows(path: &Path, input: usize) -> Result<Vec<Value>> {
    struct Select(usize);
    impl<'de> serde::de::DeserializeSeed<'de> for Select {
        type Value = Vec<Value>;
        fn deserialize<D: serde::Deserializer<'de>>(
            self,
            d: D,
        ) -> std::result::Result<Self::Value, D::Error> {
            struct Visitor(usize);
            impl<'de> serde::de::Visitor<'de> for Visitor {
                type Value = Vec<Value>;
                fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    f.write_str("protected row array")
                }
                fn visit_seq<A: serde::de::SeqAccess<'de>>(
                    self,
                    mut a: A,
                ) -> std::result::Result<Self::Value, A::Error> {
                    let mut rows = Vec::new();
                    while let Some(row) = a.next_element::<Value>()? {
                        if row["term"]["index"].as_u64() == Some(self.0 as u64)
                            && row["term"]["position"].as_u64().is_some_and(|p| p < 3)
                        {
                            rows.push(row);
                        }
                    }
                    Ok(rows)
                }
            }
            d.deserialize_seq(Visitor(self.0))
        }
    }
    let mut d =
        serde_json::Deserializer::from_reader(std::io::BufReader::new(fs::File::open(path)?));
    let rows = serde::de::DeserializeSeed::deserialize(Select(input), &mut d)?;
    d.end()?;
    Ok(rows)
}
fn complete_episode_targets(targets: &[u32]) -> bool {
    targets.len() == 15 && targets.last() == Some(&1) && !targets[..14].contains(&1)
}
fn prepare_episode(
    a: &Args,
    e: &EpisodeConfig,
    p: &ContinuationParent,
    active: &PrefixAngularWeights,
    parent: &[f32],
    frames: &mut Vec<shared::Frame>,
) -> Result<shared::ObjectiveSpec> {
    replay_require(
        sha256_file(&e.typed_authority)? == e.expected_typed_authority_sha256
            && e.expected_typed_authority_sha256
                == "5003f117b15249a214243430cd4201aa022f7eb2c25139f9849f4c09e1165103",
        "typed episode authority differs",
    )?;
    let root = a
        .checkpoint
        .parent()
        .ok_or_else(|| bad("original parent root absent"))?;
    let rawfile = root.join(format!("development-0001-row-{:04}.json", e.input_index));
    replay_require(
        sha256_file(&rawfile)? == e.expected_canonical_row_sha256,
        "episode canonical row hash differs",
    )?;
    let raw = read(&rawfile)?;
    let typed = read(&e.typed_authority)?;
    replay_require(
        typed["input_index"] == e.input_index && typed["id"] == e.expected_id,
        "typed episode task identity differs",
    )?;
    let targets: Vec<u32> = shared::dec(&raw["canonical_target_ids_labels_only"])?;
    replay_require(
        raw["id"] == e.expected_id
            && complete_episode_targets(&targets)
            && typed["ordered_episode"]["canonical_targets"] == json!(targets),
        "complete episode/EOS identity differs",
    )?;
    for f in frames.iter_mut() {
        f.weight = if f.input == e.input_index && f.position == 4 {
            1.
        } else {
            1. / 17.
        };
    }
    let mut roles = frames
        .iter()
        .filter(|f| !(f.input == e.input_index && f.position == 4))
        .map(|f| shared::ObjectiveRole {
            input: f.input,
            position: f.position,
            task: false,
            weight: 1. / 17.,
        })
        .collect::<Vec<_>>();
    replay_require(roles.len() == 17, "episode unchanged references absent")?;
    let mut natives = frames
        .iter()
        .filter(|f| f.input == e.input_index)
        .map(|f| (f.position, f.native.clone()))
        .collect::<BTreeMap<_, _>>();
    let fullfile = root.join("exported-candidate-protected-pools.json");
    replay_require(
        sha256_file(&fullfile)?
            == "fcde63e9fc12f218b866b71323601485db7f93fd1b69a54beec91e87a7b84fad",
        "episode protected original pin differs",
    )?;
    let mut reducer = NativeVocabularyActions::new(p.integer.binding().clone(), &p.exp)?;
    for row in episode_protected_rows(&fullfile, e.input_index)? {
        let position = shared::idx(&row["term"]["position"])?;
        replay_require(
            row["id"] == e.expected_id
                && row["term"]["target"] == targets[position]
                && row["actual_prefix_ids"] == json!(&targets[..position]),
            "retained episode fullframe canonical identity differs",
        )?;
        let g: Vec<i64> = shared::dec(&row["generate_q24"])?;
        let ids: Vec<u32> = shared::dec(&row["copy_ids"])?;
        let copy: Vec<i64> = shared::dec(&row["copy_q24"])?;
        let trace = reducer.reduce_trace(&g, &ids, &copy)?;
        replay_require(
            json!(trace.summary) == row["pool"] && json!(trace.token_masses) == row["token_masses"],
            "episode retained original fullpool differs",
        )?;
        let mut n = json!({"bank_trace":row["bank_trace"],"bridge":row["bridge"],"generate_q24":g,
            "copy_q24":copy,"copy_ids":ids,"pool":trace,"post_state":row["factual_state"],
            "continuation":raw["generation"][position]["continuation"]});
        n["continuation"]["delta_scores_q24"] = row["frozen_u_q24"].clone();
        let donor = shared::idx(&n["bridge"]["selected_ordinal"])?;
        n["post_state"] = raw["canonical"][position]["native"]["post_state_codes"].clone();
        // Retained carrier source/query state and pool are authoritative; donor replay later checks both.
        n["bridge"]["selected_ordinal"] = json!(donor);
        validate_original_canonical(&n, &raw, position)?;
        replay_require(
            natives.insert(position, n).is_none(),
            "duplicate retained episode phase",
        )?;
    }
    for phase in &e.phases {
        replay_require(
            phase.position < targets.len() && !natives.contains_key(&phase.position),
            "duplicate/out-of-range episode witness",
        )?;
        let n = if let Some(w) = &phase.normalized_original {
            replay_require(
                !phase.original_prefix_inverse
                    && Path::new(&w.file).components().count() == 1
                    && matches!(
                        Path::new(&w.file).components().next(),
                        Some(std::path::Component::Normal(_))
                    ),
                "normalized original witness leaf/scope invalid",
            )?;
            let report = shared::sealed(
                &phase.capture.root,
                &phase.capture.expected_report_sha256,
                &phase.capture.expected_manifest_sha256,
            )?;
            replay_require(
                report["mode"] == "prefix_joint_fragment_learning"
                    && report["candidate_receipt"]["parent"] == json!(p.binding),
                "normalized witness source epoch differs",
            )?;
            let file = phase.capture.root.join(&w.file);
            replay_require(
                sha256_file(&file)? == w.expected_sha256,
                "normalized original witness hash differs",
            )?;
            let n = read(&file)?;
            replay_require(
                n["id"] == e.expected_id
                    && n["position"] == phase.position
                    && n["actual_prefix_ids"] == json!(&targets[..phase.position])
                    && n["original_derivation"]["all8_prefix_lanes_inverted"] == true,
                "normalized original provenance differs",
            )?;
            validate_original_canonical(&n, &raw, phase.position)?;
            write(
                a,
                &format!("original-episode-phase-{:02}.json", phase.position),
                &n,
            )?;
            n
        } else {
            original_captured_phase(
                a,
                &phase.capture,
                p,
                active,
                parent,
                e.input_index,
                phase.position,
                phase.original_prefix_inverse,
            )?
        };
        natives.insert(phase.position, n);
    }
    replay_require(
        natives.len() == 15 && (0..15).all(|i| natives.contains_key(&i)),
        "episode witness coverage incomplete",
    )?;
    frames.retain(|f| !(f.input == e.input_index && f.position == 4));
    for position in 0..15 {
        roles.push(shared::ObjectiveRole {
            input: e.input_index,
            position,
            task: true,
            weight: 1. / 15.,
        });
        if let Some(f) = frames
            .iter_mut()
            .find(|f| f.input == e.input_index && f.position == position)
        {
            f.weight += 1. / 15.;
        } else {
            frames.push(shared::parse_saved_native_frame(
                e.input_index,
                position,
                e.expected_id.clone(),
                targets[..position].to_vec(),
                targets[position],
                1. / 15.,
                &natives[&position],
                p,
                true,
            )?);
        }
    }
    frames.sort_by_key(|f| {
        (
            if f.input == e.input_index { 0 } else { 1 },
            f.input,
            f.position,
        )
    });
    replay_require(frames.len() == 31, "episode unique physical count differs")?;
    let spec = shared::ObjectiveSpec {
        tasks: (0..15).map(|i| (e.input_index, i)).collect(),
        references: 17,
        roles: Some(roles),
    };
    write(
        a,
        "episode-objective-authority.json",
        &json!({"spec":spec,"physical_frames":frames.iter().enumerate().map(|(i,f)|json!({"physical_index":i,"input":f.input,"position":f.position,"id":f.id,"target":f.target,"coalesced_gradient_weight":f.weight})).collect::<Vec<_>>(),
        "typed_authority_sha256":e.expected_typed_authority_sha256,"gradient_calls":31,"weighted_roles":32,
        "scope":"complete canonical conditional episode inclEOS, not actual original ownfeedback"}),
    )?;
    Ok(spec)
}

/// Original witness assembly shared with the Generate-only offline experiment.
/// No Prefix gradient/update or encoder replay; all inherited authority checks remain.
pub(super) fn prepare_generate_objective(
    a: &Args,
    c: &Config,
    original: &ContinuationParent,
) -> Result<(Vec<shared::Frame>, shared::ObjectiveSpec)> {
    let e = c
        .episode
        .as_ref()
        .ok_or_else(|| bad("Generate requires complete episode inputs"))?;
    replay_require(
        c.joint.is_none() && c.trajectory.is_none(),
        "Generate input mode conflict",
    )?;
    shared::sealed(
        &c.retained_intermediate_root,
        shared::P_REPORT,
        shared::P_SEAL,
    )?;
    shared::sealed(&c.retained_probe_root, shared::B_REPORT, shared::B_SEAL)?;
    let sw = SourceRealizerWeights::load_context_potential_on_device(
        &a.checkpoint.join("source"),
        &fs::read(a.checkpoint.join("native/tokenizer.json"))?,
        &Device::Cpu,
    )?;
    let native = NativeSourceRealizer::load(
        &a.checkpoint.join("native"),
        &sw,
        &shared::dec::<ConsumerIdentity>(
            &read(&a.checkpoint.join("native/metadata.json"))?["identity"],
        )?,
    )?;
    let cue = CueAngularWeights::load(
        &a.checkpoint.join("cue"),
        &native,
        &a.checkpoint.join("native"),
    )?;
    let carrier = native.compile_cue_carrier(cue.native()?)?;
    let prefix = PrefixAngularWeights::load(
        &a.checkpoint.join("prefix"),
        &native,
        &a.checkpoint.join("native"),
        &a.checkpoint.join("cue"),
        &carrier,
    )?;
    replay_require(
        sha256_file(&a.checkpoint.join("prefix/prefix-q4.bin"))?
            == "c2e8ec992996055450f77237ec64730c28b2e7cd53f9ae49cdb7a28128236d0a"
            && sha256_file(&a.checkpoint.join("prefix/prefix-source-f32.bin"))?
                == "1e47a7dff9134d393043a2313a7da50ea234d1b1ae7888d895e3604855ac0fe3",
        "Generate original frozen Prefix differs",
    )?;
    let parent = shared::floats(&a.checkpoint.join("prefix/prefix-source-f32.bin"))?;
    let seed = shared::Config {
        retained_intermediate_root: c.retained_intermediate_root.clone(),
        retained_probe_root: c.retained_probe_root.clone(),
        retained_finite_root: c.retained_probe_root.clone(),
    };
    let (mut frames, _) = shared::prepare_frames(a, &seed, original, original, true)?;
    let spec = prepare_episode(a, e, original, &prefix, &parent, &mut frames)?;
    Ok((frames, spec))
}
pub(super) fn prepare_generate_guards(
    a: &Args,
    c: &Config,
    original: &ContinuationParent,
    frames: &[shared::Frame],
    spec: &shared::ObjectiveSpec,
) -> Result<(Vec<shared::Frame>, Vec<shared::Pool>, Value)> {
    let e = c
        .episode
        .as_ref()
        .ok_or_else(|| bad("Generate episode absent"))?;
    let tc = TrajectoryConfig {
        retained_prefix_learning_root: PathBuf::new(),
        retained_supplement_root: e.retained_supplement_root.clone(),
        expected_supplement_report_sha256: e.expected_supplement_report_sha256.clone(),
        expected_supplement_manifest_sha256: e.expected_supplement_manifest_sha256.clone(),
    };
    let parent = shared::floats(&a.checkpoint.join("prefix/prefix-source-f32.bin"))?;
    let mut cache = shared::DonorCache::with_limit(original, 128 * 1024 * 1024)?;
    let mut reducer =
        NativeVocabularyActions::new(original.integer.binding().clone(), &original.exp)?;
    prepare_trajectory_guards(
        a,
        c,
        &tc,
        frames,
        Some(spec),
        original,
        &parent,
        &mut cache,
        &mut reducer,
    )
}

pub(super) fn run(a: &Args, start: Instant, d: &Device) -> Result<Value> {
    validate_settings(a)?;
    let c = a
        .prefix_fragment_learning
        .as_ref()
        .ok_or_else(|| bad("Prefix config absent"))?;
    replay_require(
        if c.trajectory.is_some() {
            d.is_cpu()
        } else {
            !d.is_cpu()
        },
        "Prefix trajectory uses host integer construction; fresh gradient mode requires CUDA",
    )?;
    shared::sealed(
        &c.retained_intermediate_root,
        shared::P_REPORT,
        shared::P_SEAL,
    )?;
    shared::sealed(&c.retained_probe_root, shared::B_REPORT, shared::B_SEAL)?;
    let original = ContinuationParent::from_checkpoint(&a.checkpoint)?;
    replay_require(
        original.binding.metadata_sha256 == shared::SOURCE
            && sha256_bytes(&original.generate) == shared::G_SHA
            && sha256_file(&a.checkpoint.join("continuation-field.bin"))? == shared::U_SHA,
        "Prefix original Source/G/U differs",
    )?;
    let sw = SourceRealizerWeights::load_context_potential_on_device(
        &a.checkpoint.join("source"),
        &fs::read(a.checkpoint.join("native/tokenizer.json"))?,
        &Device::Cpu,
    )?;
    let native = NativeSourceRealizer::load(
        &a.checkpoint.join("native"),
        &sw,
        &shared::dec::<ConsumerIdentity>(
            &read(&a.checkpoint.join("native/metadata.json"))?["identity"],
        )?,
    )?;
    let source_bits = identities(&sw.parameters())?;
    let cue = CueAngularWeights::load(
        &a.checkpoint.join("cue"),
        &native,
        &a.checkpoint.join("native"),
    )?;
    let carrier = native.compile_cue_carrier(cue.native()?)?;
    let actual = PrefixAngularWeights::load(
        &a.checkpoint.join("prefix"),
        &native,
        &a.checkpoint.join("native"),
        &a.checkpoint.join("cue"),
        &carrier,
    )?;
    let prefix_payload = native.compile_prefix_transport(&carrier, actual.native()?)?;
    // Device construction is not master authority: restore all original f32 bits before any credit.
    let active = PrefixAngularWeights::from_native(
        &native,
        &a.checkpoint.join("native"),
        &a.checkpoint.join("cue"),
        &carrier,
        &prefix_payload,
        d,
    )?;
    let actual_bits = np::snapshot(&actual.parameters())?;
    np::restore(&active.parameters(), &actual_bits)?;
    replay_require(
        np::same_bits(&actual_bits, &np::snapshot(&active.parameters())?)
            && active.packed_coefficients()?
                == fs::read(a.checkpoint.join("prefix/prefix-q4.bin"))?,
        "Prefix original fractional masters/native payload lost",
    )?;
    let parent = shared::floats(&a.checkpoint.join("prefix/prefix-source-f32.bin"))?;
    replay_require(
        actual_bits.get(NAME).is_some_and(|v| {
            v.iter()
                .zip(&parent)
                .all(|(a, b)| a.to_bits() == b.to_bits())
        }),
        "Prefix original master receipt differs",
    )?;
    let mut joint_spec = c.joint.as_ref().map(|_| shared::ObjectiveSpec {
        tasks: vec![(245, 4), (245, 5), (245, 6)],
        references: 17,
        roles: None,
    });
    let guard_config = c.trajectory.clone().or_else(|| {
        c.joint.as_ref().map(|j| TrajectoryConfig {
            retained_prefix_learning_root: PathBuf::new(),
            retained_supplement_root: j.retained_supplement_root.clone(),
            expected_supplement_report_sha256: j.expected_supplement_report_sha256.clone(),
            expected_supplement_manifest_sha256: j.expected_supplement_manifest_sha256.clone(),
        })
    });
    let guard_config = guard_config.or_else(|| {
        c.episode.as_ref().map(|e| TrajectoryConfig {
            retained_prefix_learning_root: PathBuf::new(),
            retained_supplement_root: e.retained_supplement_root.clone(),
            expected_supplement_report_sha256: e.expected_supplement_report_sha256.clone(),
            expected_supplement_manifest_sha256: e.expected_supplement_manifest_sha256.clone(),
        })
    });
    let guarded = guard_config.is_some();
    if joint_spec.is_some() || c.episode.is_some() {
        replay_require(
            sha256_file(&a.checkpoint.join("prefix/prefix-q4.bin"))?
                == "c2e8ec992996055450f77237ec64730c28b2e7cd53f9ae49cdb7a28128236d0a"
                && sha256_file(&a.checkpoint.join("prefix/prefix-source-f32.bin"))?
                    == "1e47a7dff9134d393043a2313a7da50ea234d1b1ae7888d895e3604855ac0fe3",
            "joint original Prefix payload/master differs",
        )?;
    }
    let seed = shared::Config {
        retained_intermediate_root: c.retained_intermediate_root.clone(),
        retained_probe_root: c.retained_probe_root.clone(),
        retained_finite_root: c.retained_probe_root.clone(),
    };
    let (mut frames, mut baseline) = shared::prepare_frames(a, &seed, &original, &original, true)?;
    if let Some(j) = &c.joint {
        prepare_joint_phases(a, j, &original, &active, &parent, &mut frames)?;
    }
    if let Some(e) = &c.episode {
        joint_spec = Some(prepare_episode(
            a,
            e,
            &original,
            &active,
            &parent,
            &mut frames,
        )?);
    }
    let ng = NativeGeometricGenerate::from_bytes(&original.generate, original.integer.binding())?;
    let field = NativeContinuationField::from_bytes(
        &fs::read(a.checkpoint.join("continuation-field.bin"))?,
        &original.binding,
        &ng,
    )?;
    for f in &frames {
        let state = shared::codes(&f.native["continuation"]["state_codes"])?;
        let mut u = vec![0; 4096];
        field.score_delta_into(&state, &ng, &mut u, &mut Default::default())?;
        replay_require(u == f.u, "Prefix frozen U saved arithmetic differs")?;
    }
    write(
        a,
        "prefix-original-master-binding.json",
        &json!({"original_master_file":a.checkpoint.join("prefix/prefix-source-f32.bin"),
  "original_master_sha256":sha256_file(&a.checkpoint.join("prefix/prefix-source-f32.bin"))?,
  "shape":[960],"bytes":3840,"all960_original_bits_restored_before_graph":true,
  "source_binding":original.binding,"frozen_cue":carrier.metadata(),"prefix_metadata":prefix_payload.metadata(),
  "temporary_quarter_constructor":"all960 values overwritten with authenticated original f32; not initialization authority",
  "source_parameters":source_bits,"cue_parameters":identities(&cue.parameters())?}),
    )?;
    fs::write(
        a.out.join("prefix-initial-masters.f32le"),
        parent
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    )?;
    let saved_bytes = frames.iter().try_fold(0u64, |s, f| {
        Ok::<_, Box<dyn std::error::Error>>(s + serde_json::to_vec(&f.native)?.len() as u64)
    })?;
    let active_cache_limit = if guarded {
        128 * 1024 * 1024
    } else {
        shared::CACHE_LIMIT
    };
    let numerical = saved_bytes * 3 + active_cache_limit as u64 + 64 * 1024 * 1024;
    let projected = size(&a.checkpoint)? + saved_bytes + 48 * 1024 * 1024;
    replay_require(
        numerical <= 512 * 1024 * 1024 && projected + 1048576 < a.maximum_report_bytes,
        "Prefix resource projection exceeded",
    )?;
    write(
        a,
        "resource-projection.json",
        &json!({"numerical_upper_bound_bytes":numerical,"report_projection_bytes":projected,
  "numeric_cap":536870912,"report_cap":a.maximum_report_bytes,"process_ram_cap":4294967296u64,"temporary_cap":if joint_spec.is_some(){268435456}else{536870912},
  "threads":2,"donor_cache_cap":active_cache_limit,"scope":"saved nativeframes/rawvectors/stagedpools/cache; model/autodiff tensors charged separately to process RAM"}),
    )?;
    let mut cache = if guarded {
        shared::DonorCache::with_limit(&original, 128 * 1024 * 1024)?
    } else {
        shared::DonorCache::new(&original)
    };
    let mut reducer =
        NativeVocabularyActions::new(original.integer.binding().clone(), &original.exp)?;
    let mut pools = frames
        .iter()
        .enumerate()
        .map(|(i, f)| shared::score(i, f, &parent, &parent, &original, &mut cache, &mut reducer))
        .collect::<Result<Vec<_>>>()?;
    let initial = mode_objective(&frames, &pools, joint_spec.as_ref())?;
    if joint_spec.is_some() {
        baseline = initial.clone();
    }
    replay_require(
        initial == baseline && initial["correct_reference_frames"] == 17,
        "Prefix ORIGINAL native initial objective differs",
    )?;
    write(a, "initial-original-objective.json", &initial)?;
    // Fresh joint hard parity and twenty streamed backwards finish before compact
    // objective authorities coexist with the full380 guard population.
    if let Some(e) = &c.episode {
        let inherited = shared::sealed(
            &e.retained_projection.root,
            &e.retained_projection.expected_report_sha256,
            &e.retained_projection.expected_manifest_sha256,
        )?;
        replay_require(
            inherited["mode"] == "prefix_joint_fragment_learning"
                && inherited["prefix_backward_calls"] == 20
                && inherited["protected_population"] == 380,
            "episode inherited projection population differs",
        )?;
        let prior = read(
            &e.retained_projection
                .root
                .join("trajectory-resource-projection.json"),
        )?;
        replay_require(
            prior["numeric_upper_bound"] == 412822664u64
                && prior["report_projection_bytes"] == 463317336u64,
            "episode measured projection identity differs",
        )?;
        let compact_bytes = frames.iter().try_fold(0u64, |sum, f| {
            let mut n = f.native.clone();
            n["pool"]
                .as_object_mut()
                .ok_or_else(|| bad("episode pool absent"))?
                .remove("actions");
            Ok::<_, Box<dyn std::error::Error>>(sum + serde_json::to_vec(&n)?.len() as u64)
        })?;
        let old_compact = prior["retained_objective_serialized_bytes_after_omission"]
            .as_u64()
            .ok_or_else(|| bad("prior compact bytes absent"))?;
        let extra_compact = compact_bytes.saturating_sub(old_compact);
        let extra_full = saved_bytes.saturating_sub(23416875);
        let numeric = 412822664u64 + extra_compact * 8 + 11 * 400000 + 8 * 1024 * 1024;
        let report = 463317336u64 + extra_full + 8 * 327680 + 8 * 1024 * 1024;
        // Largest measured prior phase plus allocator/device reserve and actual incremental JSON.
        let ram = 2489696u64 * 1024 + 512 * 1024 * 1024 + extra_full * 8 + 32 * 1024 * 1024;
        write(
            a,
            "episode-pregradient-resource-projection.json",
            &json!({
            "weighted_roles":32,"objective_physical_frames":31,"native_reload_union":391,
            "prior_projection":prior,"actual_saved_objective_bytes":saved_bytes,
            "actual_compact_objective_projection_bytes":compact_bytes,
            "extra_full_bytes":extra_full,"extra_compact_bytes":extra_compact,
            "numerical_projection_bytes":numeric,"report_projection_bytes":report,"process_ram_projection_bytes":ram,
            "numeric_cap":536870912,"report_cap":a.maximum_report_bytes,"process_ram_cap":4294967296u64,
            "fresh_backwards_completed":0,"graphs":"stream31 then dropped before380guard allocation",
            "protected377_loading":"streamed one row at a time, dropped before Generate graph",
            "coexistence":"full objective JSON/typed pools/Prefix leaf/one frozen Generate graph; later compact objective+380 current/staged+cache; maxphase RAM not summed sequentialgraphs",
            "estimate_not_hard_stop":true}),
        )?;
        replay_require(
            numeric <= 512 * 1024 * 1024
                && report + 1048576 < a.maximum_report_bytes
                && ram <= 4 * 1024 * 1024 * 1024,
            "episode whole-deliverable pregradient projection exceeded",
        )?;
    }
    let fresh_joint_gradient = if joint_spec.is_some() {
        let g = GenerateLearningWeights::from_native(original.integer.binding().clone(), &ng, d)?;
        restore(
            &a.checkpoint.join("generate-source"),
            &read(&a.checkpoint.join("generate-source/metadata.json"))?["parameters"],
            &g.parameters(),
            d,
        )?;
        Some(shared::gradient_for_spec(
            a,
            start,
            &frames,
            &original,
            &shared::ActiveCredit::Prefix(&active),
            &g,
            &parent,
            &pools,
            &mut cache,
            joint_spec.as_ref(),
        )?)
    } else {
        None
    };
    // Check complete original objective hard pools before discarding duplicate
    // action-record JSON; retain the typed full traces for final task reload.
    if guarded {
        for (f, pool) in frames.iter().zip(&pools) {
            replay_require(
                json!(pool.generate) == f.native["generate_q24"]
                    && json!(pool.copy) == f.native["copy_q24"]
                    && json!(pool.trace) == f.native["pool"],
                "trajectory original18 full rawpool parity differs before authority compaction",
            )?;
        }
        for f in &mut frames {
            f.native["pool"]
                .as_object_mut()
                .ok_or_else(|| bad("objective pool authority absent"))?
                .remove("actions");
            f.native["pool_action_trace"] = json!("OMITTED_RECONSTRUCTIBLE_FROM_COMPLETE_SCORES; typed objective traces and complete raw vectors/masses retained");
        }
    }
    let retained_objective_bytes = frames.iter().try_fold(0u64, |sum, f| {
        Ok::<_, Box<dyn std::error::Error>>(sum + serde_json::to_vec(&f.native)?.len() as u64)
    })?;

    let (guard_frames, mut guard_pools, guard_authority) = if let Some(tc) = &guard_config {
        prepare_trajectory_guards(
            a,
            c,
            tc,
            &frames,
            joint_spec.as_ref(),
            &original,
            &parent,
            &mut cache,
            &mut reducer,
        )?
    } else {
        (Vec::new(), Vec::new(), Value::Null)
    };
    let incidence = guard_incidence(&guard_frames)?;
    let mut guard_state_digest = guard_digest(&guard_frames, &guard_pools)?;
    if guarded {
        let pool_bytes = pools_numeric_bytes(&guard_pools);
        let typed_frames = guard_frames
            .iter()
            .map(|f| {
                f.u.len() as u64 * 8
                    + f.base_copy.len() as u64 * 8
                    + f.ids.len() as u64 * 4
                    + f.sources.iter().map(|s| s.len() as u64).sum::<u64>()
                    + serde_json::to_vec(&f.native).map_or(u64::MAX, |b| b.len() as u64 * 8)
                    + 4096
            })
            .sum::<u64>();
        let numerical = cache.limit as u64
            + 2 * pool_bytes
            + typed_frames
            + retained_objective_bytes * 8
            + pools_numeric_bytes(&pools)
            + 32 * 1024 * 1024;
        let original_native = guard_authority["original_native_serialized_bytes"]
            .as_u64()
            .ok_or_else(|| bad("guard byte projection absent"))?;
        let projection = size(&a.checkpoint)?
            + original_native
            + 380 * (4096 * 64 + 65536)
            + saved_bytes
            + 64 * 1024 * 1024;
        write(
            a,
            "trajectory-resource-projection.json",
            &json!({"actual_retained_guard_pool_bytes":pool_bytes,"typed_guard_frame_bound":typed_frames,"original_serialized_guard_native_bytes":original_native,"numeric_upper_bound":numerical,"report_projection_bytes":projection,"donor_cache_cap":cache.limit,"owner_donor_cache_cap":268435456,"original_objective_serialized_bytes_before_omission":saved_bytes,"retained_objective_serialized_bytes_after_omission":retained_objective_bytes,"objective_typed_pool_bytes":pools_numeric_bytes(&pools),"metadata_allocator_margin":33554432,"original18_full_raw_pool_parity_before_omission":joint_spec.is_none(),"objective_terms_full_raw_pool_parity_before_omission":frames.len(),"fresh_joint_backwards_completed_before_guard_coexistence":joint_spec.is_some(),"original377_complete_raw_summary_mass_parity_before_retention":true,"numeric_pass":numerical<=536870912,"report_pass":projection+1048576<a.maximum_report_bytes,"numeric_cap":536870912,"report_cap":a.maximum_report_bytes,"process_ram_cap":4294967296u64,"temporary_cap":if joint_spec.is_some(){268435456}else{536870912},"export_projection_margin_per_guard":327680,"scope":if c.episode.is_some(){"31 coalesced streamed backwards completed;32roles/31compact physical authorities, current/staged380 pools/cache; no retained Generate gradient graphs; tensors charged RAM"}else if joint_spec.is_some(){"twenty streamed backwards completed; coexisting current/staged380 pools, compact20frame authorities, cache; no retained Generate gradient graphs; tensors separately charged RAM"}else{"coexisting current/staged380 numericalpools, slim retainedframe authority, cache/metadata/allocator margin; no autodiff tensors in trajectorymode"}}),
        )?;
        replay_require(
            numerical <= 512 * 1024 * 1024 && projection + 1048576 < a.maximum_report_bytes,
            "trajectory cached numerical/export serialization projection exceeded; persisted projection components",
        )?;
        write(a, "trajectory-protected-population.json", &guard_authority)?;
        write(
            a,
            "trajectory-key-incidence.json",
            &json!({"coordinates":960,"guard_rows":380,"coordinate_rows":incidence,"semantics":"unique affected guard indices from every physical candidate's eight unmasked Prefix keys; duplicate physical multiplicity retained by staged_base"}),
        )?;
    }
    let gradients = if let Some(g) = fresh_joint_gradient {
        g
    } else if let Some(tc) = &c.trajectory {
        let old = shared::sealed(
            &tc.retained_prefix_learning_root,
            CANDIDATE_REPORT,
            CANDIDATE_SEAL,
        )?;
        replay_require(
            old["mode"] == "prefix_fragment_learning"
                && old["baseline_objective"] == baseline
                && sha256_file(
                    &tc.retained_prefix_learning_root
                        .join("prefix-gradient.f32le"),
                )? == "884ec30f0289b407ead62b558628a6b7eb69643e3b5ec5dd4c1e65a7250d4614"
                && sha256_file(
                    &tc.retained_prefix_learning_root
                        .join("frozen-prefix-ranking.json"),
                )? == "5eec881b82b556b102521272a1444578768230a6b807ad71893b695881ec516b"
                && sha256_file(
                    &tc.retained_prefix_learning_root
                        .join("prefix-initial-masters.f32le"),
                )? == "1e47a7dff9134d393043a2313a7da50ea234d1b1ae7888d895e3604855ac0fe3"
                && sha256_file(
                    &tc.retained_prefix_learning_root
                        .join("prefix-gradient-receipt.json"),
                )? == "056ba2c6d4924d7be2ae4dd63da5ed20e483e4a7cdf1c2a3d8fdf4a33d903eea",
            "trajectory reused18 gradient/rank authority differs",
        )?;
        let old_parent = shared::floats(
            &tc.retained_prefix_learning_root
                .join("prefix-initial-masters.f32le"),
        )?;
        replay_require(
            parent
                .iter()
                .zip(&old_parent)
                .all(|(a, b)| a.to_bits() == b.to_bits())
                && old_parent.len() == 960,
            "trajectory gradient initial fractional masters differ",
        )?;
        let raw_gradient = fs::read(
            tc.retained_prefix_learning_root
                .join("prefix-gradient.f32le"),
        )?;
        replay_require(
            raw_gradient.len() == 3840,
            "trajectory savedgradient byte shape differs",
        )?;
        let g = raw_gradient
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect::<Vec<_>>();
        replay_require(
            g.len() == 960 && g.iter().all(|x| x.is_finite()),
            "trajectory savedgradient domain differs",
        )?;
        write(
            a,
            "reused-prefix-gradient.json",
            &json!({"root":tc.retained_prefix_learning_root,"report_sha256":CANDIDATE_REPORT,"manifest_sha256":CANDIDATE_SEAL,"raw_gradient_sha256":"884ec30f0289b407ead62b558628a6b7eb69643e3b5ec5dd4c1e65a7250d4614","ranking_sha256":"5eec881b82b556b102521272a1444578768230a6b807ad71893b695881ec516b","gradient_receipt_sha256":"056ba2c6d4924d7be2ae4dd63da5ed20e483e4a7cdf1c2a3d8fdf4a33d903eea","new_backward_calls":0,"old427_acceptance_statuses":"NOT_AUTHORITY; all transactions recomputed under complete original380 guard","objective":"unchanged18; no new guard loss weights"}),
        )?;
        g
    } else {
        let g = GenerateLearningWeights::from_native(original.integer.binding().clone(), &ng, d)?;
        restore(
            &a.checkpoint.join("generate-source"),
            &read(&a.checkpoint.join("generate-source/metadata.json"))?["parameters"],
            &g.parameters(),
            d,
        )?;
        shared::gradient_for_spec(
            a,
            start,
            &frames,
            &original,
            &shared::ActiveCredit::Prefix(&active),
            &g,
            &parent,
            &pools,
            &mut cache,
            joint_spec.as_ref(),
        )?
    };
    let ranking = shared::ranking(&parent, &gradients)?;
    if let Some(tc) = &c.trajectory {
        replay_require(
            json!(ranking)
                == read(
                    &tc.retained_prefix_learning_root
                        .join("frozen-prefix-ranking.json"),
                )?,
            "trajectory frozen actualdelta order differs",
        )?;
    }
    write(a, "frozen-prefix-ranking.json", &json!(ranking))?;
    let mut current = parent.clone();
    let mut value = initial.clone();
    let mut trials = Vec::new();
    let mut accepted = 0;
    for (order, m) in ranking.iter().enumerate() {
        shared::progress(a, start)?;
        if m.status != "eligible" {
            trials.push(json!({"order":order,"move":m,"status":m.status,"current":value,"staged":"NOT_RUN",
    "native_effect":if joint_spec.is_some(){"no code displacement; current declared objective full pools reused"}else{"no code displacement; existing18 full pools reused"},"original_gate":mode_gate(&baseline,&value,joint_spec.as_ref())?,"trajectory_guard":{"population":guard_frames.len(),"affected_guard_indices":[],"checked_affected":0,"accepted_guard_digest_before":guard_state_digest,"accepted_guard_digest_after":guard_state_digest,"unchanged_no_native_code_displacement":true}}));
            continue;
        }
        let before = value.clone();
        let mut proposed = current.clone();
        proposed[m.index] = m.master_after;
        let staged = frames
            .iter()
            .enumerate()
            .map(|(i, f)| {
                shared::score(
                    i,
                    f,
                    &parent,
                    &proposed,
                    &original,
                    &mut cache,
                    &mut reducer,
                )
            })
            .collect::<Result<Vec<_>>>()?;
        let next = mode_objective(&frames, &staged, joint_spec.as_ref())?;
        let affected = &incidence[m.index];
        let staged_guards = stage_affected(affected, |i| {
            shared::score(
                guard_cache_index(frames.len(), i)?,
                &guard_frames[i],
                &parent,
                &proposed,
                &original,
                &mut cache,
                &mut reducer,
            )
            .map(compact_guard_pool)
        })?;
        let first_failure = staged_guards
            .iter()
            .find(|(i, p)| p.trace.summary.chosen_token_id != guard_frames[*i].target)
            .map(|(i, p)| compact_guard_row(&guard_frames[*i], p, *i))
            .transpose()?;
        let objective_gate = transaction_accept(&value, &next)?;
        let accept = objective_gate && first_failure.is_none();
        let guard_trial = json!({"population":guard_frames.len(),"affected_guard_indices":affected,"checked_affected":staged_guards.len(),"unaffected_reused":guard_frames.len()-affected.len(),"first_failure":first_failure,
            "staged_affected_digest":sha256_bytes(&serde_json::to_vec(&staged_guards.iter().map(|(i,p)|compact_guard_row(&guard_frames[*i],p,*i)).collect::<Result<Vec<_>>>()?)?), "staged_digest_scope":"ordered complete affectedguard compact rows; reader independently reconstructs; no fullvector repetition",
            "accepted_guard_digest_before":guard_state_digest,"all_original380_winners":guarded && first_failure.is_none()});
        trials.push(json!({"order":order,"move":m,"status":if accept{"accepted"}else{"rejected"},"before":before,"staged":next,
   "native_all18_checked":joint_spec.is_none(),"native_objective_terms_checked":frames.len(),"strict_current_ce_and_all17_original_winners":objective_gate,"trajectory_guard":guard_trial,
   "original_task_probability_improved":if joint_spec.is_some(){Value::String("NOT_APPLICABLE: joint word CE gate".into())}else{json!(shared::improved(&baseline,&next)?)},"original_gate":mode_gate(&baseline,&next,joint_spec.as_ref())?}));
        if accept {
            for (i, pool) in staged_guards {
                guard_pools[i] = pool;
            }
            guard_state_digest = guard_digest(&guard_frames, &guard_pools)?;
            current = proposed;
            pools = staged;
            value = next;
            accepted += 1;
        }
        if let Some(last) = trials.last_mut() {
            last["trajectory_guard"]["accepted_guard_digest_after"] = json!(guard_state_digest);
        }
    }
    write(
        a,
        "prefix-construction.json",
        &json!({"coordinates":960,"accepted":accepted,"trials":trials,"initial":initial,
  "final_objective":value,"frozen_order":true,"revisited_coordinates":0,"cache_peak_bytes":cache.peak,
  "donor_recomputations":cache.calls,"cache_identity":{"source_binding":cache.source_binding,"generate_sha256":cache.generate_sha256,"bridge_sha256":cache.bridge_sha256},
  "cache_key":"physical ordinal/frame/occurrence state within immutable Source/G/bridge epoch","accepted_guard":if guarded{"all380 original correct-prefix winners plus17 objective reference winners on every accepted transaction"}else{"all17 original winners on every accepted transaction"},"protected_population":guard_frames.len(),"final_guard_digest":guard_state_digest,"donor_cache_cap":cache.limit}),
    )?;
    let params = active.parameters();
    let saved = np::snapshot(&params)?;
    drop(sw);
    let loaded = load_joint_continuation(a, &original, d)?;
    let frozen_source = identities(&loaded.source.parameters())?;
    replay_require(
        frozen_source == source_bits,
        "Prefix loaded frozen Source fractional bits differ",
    )?;
    let frozen_generate = identities(&loaded.generate.parameters())?;
    let frozen_bridges = json!({"original":identities(&loaded.original_bridge.parameters())?,"marker":identities(&loaded.marker.parameters())?,
  "categorical":loaded.categorical.as_ref().map(|x|identities(&x.parameters())).transpose()?});
    let result = np::attempt_restored(&params, &saved, || {
        np::restore(
            &params,
            &BTreeMap::from([(NAME.to_string(), current.clone())]),
        )?;
        replay_require(
            frozen_source == identities(&loaded.source.parameters())?
                && frozen_generate == identities(&loaded.generate.parameters())?
                && frozen_bridges
                    == json!({"original":identities(&loaded.original_bridge.parameters())?,"marker":identities(&loaded.marker.parameters())?,
   "categorical":loaded.categorical.as_ref().map(|x|identities(&x.parameters())).transpose()?}),
            "Prefix frozen model master bits changed",
        )?;
        let (cp, u, mut receipt) =
            shared::export_joint(a, &loaded, &cue, Some(&active), &original, d)?;
        replay_require(
            cp.binding == original.binding
                && cp.generate == original.generate
                && cp.bridge == original.bridge
                && cp.joint == original.joint
                && cp.cue == original.cue
                && u.to_bytes()? == fs::read(a.checkpoint.join("continuation-field.bin"))?,
            "Prefix export changed frozen Source/Cue/G/bridge/U artifacts",
        )?;
        replay_require(
            fs::read(a.out.join("checkpoint-0001/cue/cue-source-f32.bin"))?
                == fs::read(a.checkpoint.join("cue/cue-source-f32.bin"))?,
            "Prefix frozen Cue f32 bits changed",
        )?;
        if guarded {
            if joint_spec.is_some() {
                receipt["mode"] = json!(if c.episode.is_some() {
                    "prefix_episode_progression_learning"
                } else {
                    "prefix_joint_fragment_learning"
                });
                receipt["new_gradients"] = json!(1);
                receipt["coefficient_backward_calls"] = json!(frames.len());
                receipt["credit_scope"] = json!(if c.episode.is_some() {
                    "fresh31 coalesced weighted Prefix-only gradients;32 explicit roles; complete15 episode/EOS; frozen Generate direct gather/detached donor;380guards one960pass"
                } else {
                    "fresh20 weighted Prefix-only extracted gradients; frozen Generate graph/direct gather + detached donor surrogate; native380 trajectory guards; one960 pass"
                });
                receipt["policy"] = if c.episode.is_some() {
                    episode_policy()
                } else {
                    joint_policy()
                };
                receipt["credit"] = json!({"new_backward_calls":frames.len(),"authority":"prefix-gradient-receipt.json","selection":"fresh joint aggregate960 actual fractional-master adjacent displacement order; no old gradient/ranking reuse"});
            } else {
                receipt["mode"] = json!("prefix_trajectory_learning");
                receipt["new_gradients"] = json!(0);
                receipt["coefficient_backward_calls"] = json!(0);
                receipt["credit_scope"]=json!("reused original18 Prefix-only gradient/rank; no new backward or optimizer; native380 hardtrajectoryguard");
                receipt["policy"] = trajectory_policy();
                receipt["inherited_gradient_credit"] = receipt["credit"].clone();
                receipt["credit"] = json!({"new_backward_calls":0,"authority":"reused-prefix-gradient.json","selection":"identical authenticated original960 order and actual initial masters; prior accepted statuses are not authority"});
            }
            let encoded_receipt = serde_json::to_vec_pretty(&receipt)?;
            for leaf in [
                "checkpoint-0001/receipt.json",
                "checkpoint-0001/continuation-source/metadata.json",
            ] {
                fs::write(a.out.join(leaf), &encoded_receipt)?;
            }
            shared::reload_guard_candidate(a, &cp, &u, &guard_frames, &guard_pools)?;
            for (i, f) in frames.iter().enumerate().filter(|(_, f)| {
                joint_spec
                    .as_ref()
                    .map_or(f.input == 245 && f.position == 4, |s| s.is_task(f))
                    && !(c.episode.is_some()
                        && guard_frames
                            .iter()
                            .any(|g| g.input == f.input && g.position == f.position))
            }) {
                shared::reload_candidate(
                    a,
                    &cp,
                    &u,
                    std::slice::from_ref(f),
                    std::slice::from_ref(&pools[i]),
                )?;
            }
        } else {
            shared::reload_candidate(a, &cp, &u, &frames, &pools)?;
        }
        Ok(receipt)
    });
    replay_require(
        np::same_bits(&saved, &np::snapshot(&params)?),
        "Prefix original master bits not restored",
    )?;
    let receipt = result?;
    let final_gate = mode_gate(&baseline, &value, joint_spec.as_ref())?;
    let task = value["terms"]
        .as_array()
        .ok_or_else(|| bad("Prefix final terms absent"))?
        .iter()
        .find(|x| {
            if let Some(e) = &c.episode {
                x["input_index"] == e.input_index
            } else {
                x["input_index"] == 245 && x["position"] == 4
            }
        })
        .ok_or_else(|| bad("Prefix task term absent"))?;
    let corrected = if joint_spec.is_some() {
        value["all_phase_winners"] == true
    } else {
        task["pool"]["chosen_token_id"] == 267
    };
    write(a, "final-objective.json", &value)?;
    let all_trajectory_preserved = guard_frames
        .iter()
        .zip(&guard_pools)
        .all(|(f, p)| f.target == p.trace.summary.chosen_token_id);
    if guarded {
        write(
            a,
            "final-trajectory-guards.json",
            &json!({"guards":380,"all_original_winners":all_trajectory_preserved,"digest":guard_state_digest,"terms":guard_frames.iter().zip(&guard_pools).enumerate().map(|(i,(f,p))|compact_guard_row(f,p,i)).collect::<Result<Vec<_>>>()?}),
        )?;
    }
    Ok(
        json!({"schema":if c.episode.is_some(){"uor-r4.prefix-episode-progression-learning/1"}else if joint_spec.is_some(){"uor-r4.prefix-joint-fragment-learning/1"}else if c.trajectory.is_some(){"uor-r4.prefix-trajectory-learning/1"}else{"uor-r4.prefix-fragment-learning/1"},"status":"COMPLETED","mode":if c.episode.is_some(){"prefix_episode_progression_learning"}else if joint_spec.is_some(){"prefix_joint_fragment_learning"}else if c.trajectory.is_some(){"prefix_trajectory_learning"}else{"prefix_fragment_learning"},"policy":if c.episode.is_some(){episode_policy()}else if joint_spec.is_some(){joint_policy()}else if c.trajectory.is_some(){trajectory_policy()}else{policy()},
 "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"baseline_objective":baseline,"initial_original":initial,"candidate_objective":value,
 "final_gate":final_gate,"finite_prefix_positive":final_gate["finite_joint_positive"],"objective_spec":joint_spec,"episode_request":c.episode,"teacher_forced_joint_word":c.joint.is_some(),"teacher_forced_complete_episode":c.episode.is_some(),"termination_credit":"native full vocabulary EOS; no noRead/null policy; connected/zero/missing reported","weighted_roles":if c.episode.is_some(){32}else{frames.len()},"unique_objective_frames":frames.len(),"all_episode_phase_targets_correct":if c.episode.is_some(){json!(corrected)}else{Value::Null},"actual_fragment_corrected":corrected,"all3_phase_targets_correct":if c.joint.is_some(){json!(corrected)}else{Value::String("NOT_APPLICABLE".into())},
 "qualified_fragment":final_gate["finite_joint_positive"]==true && corrected && all_trajectory_preserved,"all_original380_preserved":guarded && all_trajectory_preserved,"protected_population":guard_frames.len(),"selected_model":false,"useful_candidate":false,
 "candidate_receipt":receipt,"parent_master_bits_restored":true,"new_prefix_gradients":if c.trajectory.is_some(){0}else{1},"prefix_backward_calls":if c.trajectory.is_some(){0}else{frames.len()},
 "new_context_gradients":0,"new_cue_gradients":0,"optimizer_updates":0,"candidate_native_steps":if c.episode.is_some(){391}else if joint_spec.is_some(){383}else if c.trajectory.is_some(){381}else{18},"baseline_encoder_calls":0,"execution_lane":if c.trajectory.is_some(){"host native integer construction/reload; CPU parameter storage only for coherent artifact export; no CUDA initialization, autodiff forward/backward or accelerator training"}else{"CUDA Prefix-only coefficient gradient then native integer construction; frozen Generate graph autodiff, only Prefix gradients extracted"},
 "autoregressive_rollout":"NOT_RUN; parent admits cheap actual-artifact ownprefix/multiturn/original8 only after construction gate"}),
    )
}

const CANDIDATE_REPORT: &str = "71301b77d9d4606dda1501385e43f8e114c4e72af9a502b799f363b604b14cfe";
const CANDIDATE_SEAL: &str = "12c1412d049724e6dd7ba1e1cd8fc0383be65f7a3c9f43251a206fc23ba76d32";
const PREFIX_PACKED: &str = "a6ea6299cec8b5739b2e20002488989f932392054ec28c24dde5057eb2de2b86";
const PREFIX_MASTER: &str = "900f1e23d31369fc0a7f78d6db23cf7777f95226b06ae4ebce6685111075ad36";
const PILOT_SHA: &str = "8d1e16617d99da7d63adf033bf685854739db8e0aaab5bcbcd758ae1b86571aa";
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ArtifactConfig {
    pub retained_candidate_root: PathBuf,
    pub retained_intermediate_root: PathBuf,
    #[serde(default)]
    pub trajectory_candidate: Option<TrajectoryArtifactAuthority>,
    #[serde(default)]
    pub joint_candidate: Option<TrajectoryArtifactAuthority>,
    #[serde(default)]
    pub episode_candidate: Option<TrajectoryArtifactAuthority>,
    #[serde(default)]
    pub generate_episode_candidate: Option<GenerateArtifactAuthority>,
    #[serde(default)]
    pub coupled_episode_candidate: Option<super::coupled_episode_learning::ArtifactAuthority>,
}
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TrajectoryArtifactAuthority {
    pub expected_report_sha256: String,
    pub expected_manifest_sha256: String,
    pub expected_prefix_packed_sha256: String,
    pub expected_prefix_master_sha256: String,
}
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct GenerateArtifactAuthority {
    pub expected_report_sha256: String,
    pub expected_manifest_sha256: String,
    pub expected_generate_sha256: String,
    #[serde(default)]
    pub family: super::generate_episode_learning::GenerateFamily,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_unary_master_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_pair_master_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maximum_coordinates: Option<usize>,
    pub expected_continuation_sha256: String,
}
fn validate_generate_artifact_authority(c: &GenerateArtifactAuthority) -> Result<()> {
    use super::generate_episode_learning::GenerateFamily;
    let valid_hash = |h: &str| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit());
    replay_require(
        [
            &c.expected_report_sha256,
            &c.expected_manifest_sha256,
            &c.expected_generate_sha256,
            &c.expected_continuation_sha256,
        ]
        .iter()
        .all(|h| valid_hash(h)),
        "Generate artifact external identity hashes invalid",
    )?;
    replay_require(
        match c.family {
            GenerateFamily::Unary => {
                c.expected_unary_master_sha256
                    .as_deref()
                    .is_some_and(valid_hash)
                    && c.expected_pair_master_sha256.is_none()
                    && c.maximum_coordinates.is_none()
            }
            GenerateFamily::Pair => {
                c.expected_pair_master_sha256
                    .as_deref()
                    .is_some_and(valid_hash)
                    && c.expected_unary_master_sha256.is_none()
                    && c.maximum_coordinates
                        .is_some_and(|n| (1..=960).contains(&n))
            }
        },
        "Generate artifact family requires its own master hash and pair budget in1..960",
    )
}
fn validate_trajectory_artifact_authority(c: &TrajectoryArtifactAuthority) -> Result<()> {
    replay_require(
        [
            &c.expected_report_sha256,
            &c.expected_manifest_sha256,
            &c.expected_prefix_packed_sha256,
            &c.expected_prefix_master_sha256,
        ]
        .iter()
        .all(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit())),
        "trajectory artifact external hashes invalid",
    )
}
pub(super) fn validate_artifact_settings(a: &Args) -> Result<()> {
    if let Some(c) = &a.prefix_artifact_check {
        replay_require(
            [
                c.trajectory_candidate.is_some(),
                c.joint_candidate.is_some(),
                c.episode_candidate.is_some(),
                c.generate_episode_candidate.is_some(),
                c.coupled_episode_candidate.is_some(),
            ]
            .into_iter()
            .filter(|x| *x)
            .count()
                <= 1,
            "cheap artifact endpoint modes exclusive",
        )?;
        if let Some(g) = &c.coupled_episode_candidate {
            super::coupled_episode_learning::validate_artifact_authority(g)?;
        }
        if let Some(g) = &c.generate_episode_candidate {
            validate_generate_artifact_authority(g)?;
        }
        if let Some(t) = c
            .episode_candidate
            .as_ref()
            .or(c.joint_candidate.as_ref())
            .or(c.trajectory_candidate.as_ref())
        {
            validate_trajectory_artifact_authority(t)?;
        }
        replay_require(
            a.mode == Mode::JointContinuation
                && a.updates == 1
                && a.loss_scope == LossScope::All
                && a.prefix_fragment_learning.is_none()
                && a.generate_episode_learning.is_none()
                && a.coupled_episode_learning.is_none()
                && a.generate_episode_completion.is_none()
                && a.context_cue_coadapt.is_none()
                && a.prefix_context_credit.is_none()
                && a.context_path_credit.is_none()
                && a.readout_coadaptation.is_none()
                && a.reached_u.is_none()
                && a.prototype_compensation.is_none()
                && a.reference_replay.is_none()
                && a.retained_context_root.is_none()
                && !a.native_code_proposals
                && !a.reached_frontier_objective
                && !a.constrained_context_learning
                && !a.constrained_emission_learning
                && !a.categorical_action_learning
                && !a.categorical_action_only,
            "Prefix artifact check excludes all learning/construction modes",
        )?;
        replay_require(
            fs::canonicalize(&a.checkpoint)?
                == fs::canonicalize(c.retained_candidate_root.join("checkpoint-0001"))?
                && a.maximum_report_bytes <= 256 * 1024 * 1024,
            "Prefix artifact root/report admission differs",
        )?;
    }
    Ok(())
}
fn retained_indices(pilot: &Value, full: &Value) -> Result<Vec<usize>> {
    let indices = shared::dec::<Vec<usize>>(&pilot["indices"])?;
    let rows = pilot["evaluation"]["rows"]
        .as_array()
        .ok_or_else(|| bad("original pilot rows missing"))?;
    let original = full["rows"]
        .as_array()
        .ok_or_else(|| bad("original complete row index missing"))?;
    replay_require(
        indices.len() == 8
            && rows.len() == 8
            && original.len() == 512
            && indices.iter().copied().collect::<BTreeSet<_>>().len() == 8,
        "original eight coverage/uniqueness differs",
    )?;
    for (index, pilotrow) in indices.iter().zip(rows) {
        let row = original
            .get(*index)
            .ok_or_else(|| bad("original eight index outside panel"))?;
        replay_require(
            pilotrow["id"] == row["id"]
                && pilotrow["complete"] == true
                && pilotrow["eos"] == true
                && row["complete"] == true
                && row["eos"] == true,
            "original pilot/full identity or actual EOS differs",
        )?;
    }
    Ok(indices)
}
pub(super) fn run_artifact_check(a: &Args, start: Instant) -> Result<Value> {
    validate_artifact_settings(a)?;
    let c = a
        .prefix_artifact_check
        .as_ref()
        .ok_or_else(|| bad("Prefix artifact config absent"))?;
    let (report_pin, seal_pin, packed_pin, master_pin) =
        if let Some(g) = &c.coupled_episode_candidate {
            super::coupled_episode_learning::validate_artifact_authority(g)?;
            (
                g.expected_report_sha256.as_str(),
                g.expected_manifest_sha256.as_str(),
                g.expected_prefix_packed_sha256.as_str(),
                g.expected_prefix_master_sha256.as_str(),
            )
        } else if let Some(g) = &c.generate_episode_candidate {
            validate_generate_artifact_authority(g)?;
            (
                g.expected_report_sha256.as_str(),
                g.expected_manifest_sha256.as_str(),
                "c2e8ec992996055450f77237ec64730c28b2e7cd53f9ae49cdb7a28128236d0a",
                "1e47a7dff9134d393043a2313a7da50ea234d1b1ae7888d895e3604855ac0fe3",
            )
        } else if let Some(t) = c
            .episode_candidate
            .as_ref()
            .or(c.joint_candidate.as_ref())
            .or(c.trajectory_candidate.as_ref())
        {
            validate_trajectory_artifact_authority(t)?;
            (
                t.expected_report_sha256.as_str(),
                t.expected_manifest_sha256.as_str(),
                t.expected_prefix_packed_sha256.as_str(),
                t.expected_prefix_master_sha256.as_str(),
            )
        } else {
            (
                CANDIDATE_REPORT,
                CANDIDATE_SEAL,
                PREFIX_PACKED,
                PREFIX_MASTER,
            )
        };
    let report = shared::sealed(&c.retained_candidate_root, report_pin, seal_pin)?;
    if c.generate_episode_candidate.is_some() {
        super::generate_episode_learning::authenticate_positive_artifact(a, c, &report)?;
    }
    if c.coupled_episode_candidate.is_some() {
        super::coupled_episode_learning::authenticate_positive_artifact(a, c, &report)?;
    }
    if c.episode_candidate.is_some() {
        let gradient = read(
            &c.retained_candidate_root
                .join("prefix-gradient-receipt.json"),
        )?;
        let guard = read(
            &c.retained_candidate_root
                .join("final-trajectory-guards.json"),
        )?;
        let authority = read(
            &c.retained_candidate_root
                .join("episode-objective-authority.json"),
        )?;
        replay_require(
            report["mode"] == "prefix_episode_progression_learning"
                && report["selected_model"] == false
                && report["all_original380_preserved"] == true
                && report["protected_population"] == 380
                && report["new_prefix_gradients"] == 1
                && report["prefix_backward_calls"] == 31
                && report["candidate_native_steps"] == 391
                && report["weighted_roles"] == 32
                && report["unique_objective_frames"] == 31
                && report["all_episode_phase_targets_correct"] == true
                && gradient["active_names"] == json!([NAME])
                && gradient["per_term"].as_array().is_some_and(|r| {
                    r.len() == 31
                        && r.iter().all(|v| {
                            v["status"] == "PRESENT"
                                && v["bytes"] == 3840
                                && v["missing_gradient_filled_zero"] == false
                        })
                })
                && authority["spec"]["roles"]
                    .as_array()
                    .is_some_and(|r| r.len() == 32)
                && guard["guards"] == 380
                && guard["all_original_winners"] == true,
            "episode construction/gradient/guard authority differs",
        )?;
    }
    if c.joint_candidate.is_some() {
        let gradient = read(
            &c.retained_candidate_root
                .join("prefix-gradient-receipt.json"),
        )?;
        let guard = read(
            &c.retained_candidate_root
                .join("final-trajectory-guards.json"),
        )?;
        let objective = read(
            &c.retained_candidate_root
                .join("joint-objective-authority.json"),
        )?;
        replay_require(
            report["mode"] == "prefix_joint_fragment_learning"
                && report["all_original380_preserved"] == true
                && report["protected_population"] == 380
                && report["new_prefix_gradients"] == 1
                && report["prefix_backward_calls"] == 20
                && report["candidate_native_steps"] == 383
                && report["candidate_objective"]["all_phase_winners"] == true
                && report["selected_model"] == false
                && gradient["per_term"].as_array().is_some_and(|rows| {
                    rows.len() == 20
                        && rows.iter().all(|r| {
                            r["status"] == "PRESENT"
                                && r["bytes"] == 3840
                                && r["missing_gradient_filled_zero"] == false
                        })
                })
                && gradient["active_names"] == json!([NAME])
                && guard["guards"] == 380
                && guard["all_original_winners"] == true
                && objective["tasks"].as_array().is_some_and(|r| r.len() == 3)
                && guard["terms"].as_array().is_some_and(|rows| {
                    rows.len() == 380
                        && rows
                            .iter()
                            .all(|r| r["chosen"] == r["required_original_winner"])
                }),
            "joint candidate freshcredit/phase/completeguard authority differs",
        )?;
    }
    if c.trajectory_candidate.is_some() {
        let gradient = read(
            &c.retained_candidate_root
                .join("reused-prefix-gradient.json"),
        )?;
        let protected = read(
            &c.retained_candidate_root
                .join("trajectory-protected-population.json"),
        )?;
        let final_guard = read(
            &c.retained_candidate_root
                .join("final-trajectory-guards.json"),
        )?;
        replay_require(
            report["mode"] == "prefix_trajectory_learning"
                && report["all_original380_preserved"] == true
                && report["protected_population"] == 380
                && report["new_prefix_gradients"] == 0
                && report["prefix_backward_calls"] == 0
                && report["selected_model"] == false
                && gradient["raw_gradient_sha256"]
                    == "884ec30f0289b407ead62b558628a6b7eb69643e3b5ec5dd4c1e65a7250d4614"
                && gradient["ranking_sha256"]
                    == "5eec881b82b556b102521272a1444578768230a6b807ad71893b695881ec516b"
                && gradient["new_backward_calls"] == 0
                && protected["guards"] == 380
                && protected["original_full377_sha256"]
                    == "fcde63e9fc12f218b866b71323601485db7f93fd1b69a54beec91e87a7b84fad"
                && final_guard["guards"] == 380
                && final_guard["all_original_winners"] == true
                && final_guard["terms"].as_array().is_some_and(|rows| {
                    rows.len() == 380
                        && rows
                            .iter()
                            .all(|r| r["chosen"] == r["required_original_winner"])
                }),
            "trajectory candidate complete guard/savedgradient authority differs",
        )?;
    }
    let parent_report = shared::sealed(
        &c.retained_intermediate_root,
        shared::P_REPORT,
        shared::P_SEAL,
    )?;
    if c.coupled_episode_candidate.is_some() {
        replay_require(
            report["mode"] == "coupled_episode_learning"
                && report["finite_episode_positive"] == true
                && report["final_gate"]["passed"] == true
                && report["candidate_objective"]["correct_reference_frames"] == 17,
            "coupled cheap qualification requires positive complete episode",
        )?;
    } else if let Some(g) = &c.generate_episode_candidate {
        replay_require(
            report["mode"]
                == match g.family {
                    super::generate_episode_learning::GenerateFamily::Unary => {
                        "generate_episode_learning"
                    }
                    super::generate_episode_learning::GenerateFamily::Pair => {
                        "generate_pair_episode_learning"
                    }
                }
                && report["finite_episode_positive"] == true
                && report["final_gate"]["passed"] == true
                && report["candidate_objective"]["correct_reference_frames"] == 17,
            "Generate cheap qualification requires positive complete episode construction",
        )?;
    } else {
        replay_require(
            report["mode"]
                == if c.episode_candidate.is_some() {
                    "prefix_episode_progression_learning"
                } else if c.joint_candidate.is_some() {
                    "prefix_joint_fragment_learning"
                } else if c.trajectory_candidate.is_some() {
                    "prefix_trajectory_learning"
                } else {
                    "prefix_fragment_learning"
                }
                && report["finite_prefix_positive"] == true
                && report["actual_fragment_corrected"] == true
                && report["qualified_fragment"] == true
                && report["candidate_objective"]["correct_reference_frames"] == 17,
            "Prefix cheap qualification requires completed positive construction",
        )?;
    }
    replay_require(
        parent_report["selected_model"] == true
            && parent_report["candidate_artifact_status"] == "QUALIFIED_NATIVE_GATE_AND_RETENTION",
        "Prefix original selected authority differs",
    )?;
    let pilotfile = c
        .retained_intermediate_root
        .join("pilot-original8-receipt.json");
    replay_require(
        sha256_file(&pilotfile)? == PILOT_SHA,
        "original8 pilot receipt pin differs",
    )?;
    let pilot = read(&pilotfile)?;
    let original_full = &parent_report["final_evaluation"];
    let retained = retained_indices(&pilot, original_full)?;
    let cp = ContinuationParent::from_checkpoint(&a.checkpoint)?;
    replay_require(
        cp.binding.metadata_sha256 == shared::SOURCE
            && sha256_bytes(&cp.generate)
                == c.generate_episode_candidate.as_ref().map_or_else(
                    || {
                        c.coupled_episode_candidate
                            .as_ref()
                            .map_or(shared::G_SHA, |g| g.expected_generate_sha256.as_str())
                    },
                    |g| g.expected_generate_sha256.as_str(),
                )
            && sha256_file(&a.checkpoint.join("continuation-field.bin"))?
                == c.generate_episode_candidate.as_ref().map_or_else(
                    || {
                        c.coupled_episode_candidate
                            .as_ref()
                            .map_or(shared::U_SHA, |g| g.expected_continuation_sha256.as_str())
                    },
                    |g| g.expected_continuation_sha256.as_str(),
                )
            && sha256_file(&a.checkpoint.join("prefix/prefix-q4.bin"))? == packed_pin
            && sha256_file(&a.checkpoint.join("prefix/prefix-source-f32.bin"))? == master_pin
            && read(&a.checkpoint.join("receipt.json"))? == report["candidate_receipt"],
        "Prefix artifact checkpoint/binding/actual master identity differs",
    )?;
    for (file, hash) in [
        (&a.training_inputs, INPUT_SHA),
        (&a.training_labels, LABEL_SHA),
    ] {
        report_output::verify(&seal_for(file)?)?;
        replay_require(
            sha256_file(file)? == hash,
            "Prefix artifact panel/oracle pin differs",
        )?;
    }
    replay_require(
        a.training_inputs == a.development_inputs && a.training_labels == a.development_labels,
        "Prefix artifact single frozen panel paths differ",
    )?;
    let legal = NativeVocabularyActions::new(cp.integer.binding().clone(), &cp.exp)?
        .legal_token_ids()
        .iter()
        .copied()
        .collect();
    let eps = load_panel(
        &a.training_inputs,
        &a.training_labels,
        &cp.integer,
        &cp.tokenizer,
        &legal,
        512,
    )?;
    replay_require(
        eps.len() == 512,
        "Prefix artifact original typed panel length differs",
    )?;
    for (i, reference) in original_full["rows"]
        .as_array()
        .ok_or_else(|| bad("original row authority missing"))?
        .iter()
        .enumerate()
    {
        replay_require(
            reference["id"] == eps[i].packet.id,
            "Prefix original row ID vs typed panel mismatch",
        )?;
    }
    for row in pilot["evaluation"]["rows"]
        .as_array()
        .ok_or_else(|| bad("original pilot rows absent"))?
    {
        let leaf = row["row_file"]
            .as_str()
            .ok_or_else(|| bad("original pilot rowfile absent"))?;
        replay_require(
            Path::new(leaf).components().count() == 1
                && sha256_file(&c.retained_intermediate_root.join(leaf))?
                    == row["row_sha256"]
                        .as_str()
                        .ok_or_else(|| bad("pilot rowSHA absent"))?,
            "original pilot raw row authority differs",
        )?;
    }
    let task_index = if c.episode_candidate.is_some()
        || c.generate_episode_candidate.is_some()
        || c.coupled_episode_candidate.is_some()
    {
        shared::idx(&report["episode_request"]["input_index"])?
    } else {
        245
    };
    replay_require(
        !retained.contains(&task_index),
        "task overlaps original8 retention",
    )?;
    let mut indices = retained.clone();
    indices.push(task_index);
    let rows = indices
        .iter()
        .map(|i| eps.get(*i).ok_or_else(|| bad("artifact row index missing")))
        .collect::<Result<Vec<_>>>()?;
    write(
        a,
        "artifact-input-authority.json",
        &json!({"candidate_root":c.retained_candidate_root,"candidate_report_sha256":report_pin,
  "candidate_manifest_sha256":seal_pin,"candidate_producer_source":report["source_commit"],
  "evaluator_source":option_env!("UOR_BUILD_SOURCE_COMMIT"),"source_binding":cp.binding,
  "original_root":c.retained_intermediate_root,"original_report_sha256":shared::P_REPORT,"original_manifest_sha256":shared::P_SEAL,
  "original8_pilot_sha256":PILOT_SHA,"retained_original_indices":retained,"evaluation_indices":indices,
  "row_ids":rows.iter().map(|e|&e.packet.id).collect::<Vec<_>>(),"original_parent_rollout":"REUSED_AUTHENTICATED_NOT_RERUN",
  "inputs_sha256":INPUT_SHA,"labels_sha256":LABEL_SHA,"oracle":"load_panel/answer_oracle typed intent; membership only; no labels to generator"}),
    )?;
    let native = NativeGeometricGenerate::from_bytes(&cp.generate, cp.integer.binding())?;
    let field = NativeContinuationField::from_bytes(
        &fs::read(a.checkpoint.join("continuation-field.bin"))?,
        &cp.binding,
        &native,
    )?;
    let mut evaluator = a.clone();
    evaluator.maximum_seconds = u64::MAX;
    // Existing ownfeedback evaluator; skip all canonical/teacher steps for this cheap boundary.
    let evaluation = continuation_evaluate_rows_impl(
        &evaluator,
        "cheap-ownprefix",
        &cp,
        &field,
        &rows,
        start,
        false,
    )?;
    let actual = evaluation["rows"]
        .as_array()
        .ok_or_else(|| bad("cheap ownprefix row summary missing"))?;
    replay_require(
        actual.len() == 9
            && actual
                .iter()
                .zip(&rows)
                .all(|(r, e)| r["id"] == e.packet.id),
        "cheap ownprefix output coverage differs",
    )?;
    let retained_all = actual[..8]
        .iter()
        .all(|r| r["complete"] == true && r["eos"] == true);
    let task = &actual[8];
    let taskfile = task["row_file"]
        .as_str()
        .ok_or_else(|| bad("cheap task rowfile absent"))?;
    let taskraw = read(&a.out.join(taskfile))?;
    let task_original = &original_full["rows"][task_index];
    let old_leaf = task_original["row_file"]
        .as_str()
        .ok_or_else(|| bad("original task rowfile missing"))?;
    replay_require(
        Path::new(old_leaf).components().count() == 1
            && sha256_file(&c.retained_intermediate_root.join(old_leaf))?
                == task_original["row_sha256"]
                    .as_str()
                    .ok_or_else(|| bad("original task rawSHA missing"))?,
        "original task saved row authority differs",
    )?;
    let old_task = read(&c.retained_intermediate_root.join(old_leaf))?;
    let ids = shared::dec::<Vec<u32>>(&taskraw["generated_ids"])?;
    let old_ids = shared::dec::<Vec<u32>>(&old_task["generated_ids"])?;
    let new_fragment =
        ids.len() > 4 && old_ids.len() > 4 && ids[..4] == old_ids[..4] && ids[4] == 267;
    let complete_task = task["complete"] == true && task["eos"] == true;
    let comparable_prefix = ids.len() >= 4 && old_ids.len() >= 4 && ids[..4] == old_ids[..4];
    let boundary_witness = json!({"actual_prefix_comparable":comparable_prefix,
      "position4_target267":if !comparable_prefix {json!("NO_COMPARABLE_ACTUAL_PREFIX")} else if ids.len()<=4 {json!("NOT_REACHED")} else {json!(ids[4]==267)},
      "authority":"posthoc specific boundary witness; typed-oracle full reply acceptance does not require canonical tokenization"});
    let joint_word_witness = if c.joint_candidate.is_some()
        || c.episode_candidate.is_some()
        || c.generate_episode_candidate.is_some()
        || c.coupled_episode_candidate.is_some()
    {
        let canonical: Vec<u32> = shared::dec(&old_task["canonical_target_ids_labels_only"])?;
        json!({"phases":(if c.episode_candidate.is_some() || c.generate_episode_candidate.is_some() || c.coupled_episode_candidate.is_some(){0..canonical.len()}else{4..7}).map(|position| {
            let comparable=ids.len()>=position&&canonical.len()>position&&ids[..position]==canonical[..position];
            json!({"position":position,"canonical_target_label_only":canonical.get(position),"actual_prefix_comparable":comparable,
                "actual_target":if !comparable {json!("NO_COMPARABLE_ACTUAL_PREFIX")}else if ids.len()<=position {json!("NOT_REACHED")}else{json!(ids[position]==canonical[position])}})
        }).collect::<Vec<_>>(),"scope":"posthoc token-boundary witnesses only; typed wholeanswer/EOS is qualification primary; no extra canonical inference"})
    } else {
        json!("NOT_APPLICABLE")
    };
    write(
        a,
        "cheap-ownprefix-qualification.json",
        &json!({"retained_original8":retained_all,"task245_complete_eos":complete_task,
  "task245_actual_prefix_to_position4_matches_parent":ids.len()>=4 && old_ids.len()>=4 && ids[..4]==old_ids[..4],
  "task245_actual_position4_target267":new_fragment,"task245_original_complete":task_original["complete"],"task245_candidate_complete":task["complete"],
  "original_indices":retained,"evaluation_indices":indices,"evaluation":evaluation,"parent_task_row_sha256":task_original["row_sha256"],
  "candidate_task_row_sha256":task["row_sha256"],"task245_boundary_witness":boundary_witness,"joint_word_boundary_witness":joint_word_witness,"multiturn":"UNAVAILABLE_FOR_THIS_NATIVE_SOURCE_GENERATE_U_EPOCH",
  "full512":"NOT_RUN","canonical":"NOT_RUN","qualification_scope":"9 exposed actual ownprefix rows only; no continuous conversation/durable memory/transfer qualification"}),
    )?;
    Ok(
        json!({"schema":"uor-r4.prefix-artifact-check/1","mode":"prefix_artifact_check","status":"COMPLETED",
  "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"candidate_producer_source":report["source_commit"],
  "candidate_report_sha256":report_pin,"candidate_manifest_sha256":seal_pin,
  "retained_original8":retained_all,"task245_complete_eos":complete_task,"task245_actual_position4_target267":new_fragment,
  "qualification_positive":retained_all && complete_task,"task245_boundary_witness":boundary_witness,"joint_word_boundary_witness":joint_word_witness,"actual_ownprefix_rows":9,"evaluation":evaluation,
  "gradient_calls":0,"optimizer_updates":0,"construction_passes":0,"fixed18_evaluation":"NOT_RUN","canonical":"NOT_RUN","full512":"NOT_RUN",
  "multiturn":"UNAVAILABLE_FOR_THIS_NATIVE_SOURCE_GENERATE_U_EPOCH","selected_model":false,"useful_candidate":false,
  "execution_lane":"host native integer generator from independently loaded fixed artifact; no CUDA model/gradient load",
  "wall_time_estimate_seconds":a.maximum_seconds,"healthy_estimate_is_not_hard_stop":true,
  "scope":"conditional cheap original8 retention and task245 actual complete reply; no broad capability/energy claim"}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_artifact_authority_separates_legacy_unary_and_bounded_pair() -> Result<()> {
        let hash = "a".repeat(64);
        let legacy = json!({
            "expected_report_sha256":hash,"expected_manifest_sha256":hash,
            "expected_generate_sha256":hash,"expected_unary_master_sha256":hash,
            "expected_continuation_sha256":hash
        });
        let unary: GenerateArtifactAuthority = serde_json::from_value(legacy.clone())?;
        validate_generate_artifact_authority(&unary)?;
        let mut pair = legacy;
        pair["family"] = json!("pair");
        pair["expected_pair_master_sha256"] = json!(hash);
        pair["maximum_coordinates"] = json!(960);
        // A unary master pin must never silently authenticate a pair candidate.
        assert!(
            validate_generate_artifact_authority(&serde_json::from_value(pair.clone())?).is_err()
        );
        pair.as_object_mut()
            .ok_or_else(|| bad("test authority absent"))?
            .remove("expected_unary_master_sha256");
        validate_generate_artifact_authority(&serde_json::from_value(pair.clone())?)?;
        for budget in [json!(null), json!(0), json!(961)] {
            let mut invalid = pair.clone();
            invalid["maximum_coordinates"] = budget;
            assert!(
                validate_generate_artifact_authority(&serde_json::from_value(invalid)?).is_err()
            );
        }
        pair["expected_pair_master_sha256"] = json!("invalid");
        assert!(validate_generate_artifact_authority(&serde_json::from_value(pair)?).is_err());
        Ok(())
    }

    #[test]
    fn original_prefix_head_reconstruction_keeps_frozen_other_terms() -> Result<()> {
        let fixed = -1234567i64;
        let candidate = 3i64 << 22;
        let original = -2i64 << 22;
        assert_eq!(
            replace_head_adjustment(fixed + candidate, candidate, original)?,
            fixed + original
        );
        assert_eq!(
            replace_head_adjustment(fixed + candidate, candidate, candidate)?,
            fixed + candidate
        );
        assert!(replace_head_adjustment(i64::MAX, -1, 0).is_err());
        Ok(())
    }
    #[test]
    fn joint_guard_cache_namespace_cannot_overlap_objective() -> Result<()> {
        let guard_keys = (0..380)
            .map(|i| guard_cache_index(20, i))
            .collect::<Result<BTreeSet<_>>>()?;
        assert!(guard_keys.is_disjoint(&(0..20).collect()));
        assert_eq!(guard_cache_index(20, 379)?, 399);
        assert!(guard_cache_index(usize::MAX, 1).is_err());
        Ok(())
    }
    #[test]
    fn joint_spec_does_not_inherit_single_phase_probability_gate() -> Result<()> {
        let spec = shared::ObjectiveSpec {
            tasks: vec![(245, 4), (245, 5), (245, 6)],
            references: 17,
            roles: None,
        };
        let b = json!({"combined":4.,"task":2.,"task_target_mass":99,"task_total_mass":100});
        let v = json!({"combined":3.,"task":1.,"correct_reference_frames":17,"all_phase_winners":true,"task_target_mass":1,"task_total_mass":100});
        assert_eq!(
            mode_gate(&b, &v, Some(&spec))?["finite_joint_positive"],
            true
        );
        assert!(shared::improved(&b, &v)? == false);
        Ok(())
    }
    #[test]
    fn trajectory_csr_collapses_rows_but_keeps_physical_alias_multiplicity() -> Result<()> {
        let row = vec![
            Some(0),
            Some(120),
            Some(240),
            Some(360),
            Some(480),
            Some(600),
            Some(720),
            Some(840),
        ];
        let mut other = row.clone();
        other[0] = Some(1);
        let keys = vec![vec![row.clone(), row], vec![other]];
        let csr = key_incidence(keys.iter())?;
        assert_eq!(csr[0], vec![0]);
        assert_eq!(csr[1], vec![1]);
        assert_eq!(csr[120], vec![0, 1]);
        let parent = vec![0f32; 960];
        let mut next = parent.clone();
        next[0] = 0.25;
        assert_eq!(
            shared::staged_base(&[7, 11], &keys[0], &parent, &next)?,
            vec![7 + (1 << 22), 11 + (1 << 22)]
        );
        assert_eq!(
            shared::staged_base(&[13], &keys[1], &parent, &next)?,
            vec![13]
        );
        Ok(())
    }
    #[test]
    fn trajectory_correct_population_keeps_eos_and_all_new_correct_prefixes() -> Result<()> {
        let generated = [617, 2097, 1717];
        let mut raw = json!({"id":"row","generated_ids":generated,"eos":false,"canonical_target_ids_labels_only":[617,2097,315,1],
          "generation":generated.iter().enumerate().map(|(i,t)|json!({"actual_prefix_ids":&generated[..i],"pool":{"summary":{"chosen_token_id":t}}})).collect::<Vec<_>>()});
        let terms = correct_prefix_terms(&raw, 3, "row")?;
        assert_eq!(terms.len(), 2);
        assert_eq!(terms[1]["target"], 2097);
        raw["generation"][1]["actual_prefix_ids"] = json!([315]);
        assert!(correct_prefix_terms(&raw, 3, "row").is_err());
        let complete = json!({"id":"eos","generated_ids":[617,1],"eos":true,"canonical_target_ids_labels_only":[617,1],"generation":[{"actual_prefix_ids":[],"pool":{"summary":{"chosen_token_id":617}}},{"actual_prefix_ids":[617],"pool":{"summary":{"chosen_token_id":1}}}]});
        let terms = correct_prefix_terms(&complete, 455, "eos")?;
        assert_eq!(terms.len(), 2);
        assert_eq!(terms[1]["target"], 1);
        Ok(())
    }
    #[test]
    fn trajectory_late_guard_error_does_not_commit_staged_prefix_or_prior_pool() -> Result<()> {
        let original = vec![7i64, 11, 13];
        let mut current = original.clone();
        let staged: Result<Vec<(usize, i64)>> = stage_affected(&[0, 2], |i| {
            if i == 2 {
                Err(bad("late donor/reducer failure"))
            } else {
                Ok(-99)
            }
        });
        if let Ok(rows) = staged {
            for (i, value) in rows {
                current[i] = value;
            }
        }
        assert_eq!(current, original);
        let staged = stage_affected(&[0, 2], |i| Ok(original[i] + 1))?;
        for (i, value) in staged {
            current[i] = value;
        }
        assert_eq!(current, vec![8, 11, 14]); // Unaffected current pool survives unchanged.
        Ok(())
    }
    #[test]
    fn trajectory_artifact_authority_requires_independent_explicit_hashes() -> Result<()> {
        let mut c = TrajectoryArtifactAuthority {
            expected_report_sha256: "a".repeat(64),
            expected_manifest_sha256: "b".repeat(64),
            expected_prefix_packed_sha256: "c".repeat(64),
            expected_prefix_master_sha256: "d".repeat(64),
        };
        validate_trajectory_artifact_authority(&c)?;
        c.expected_prefix_master_sha256 = "quarter-reinitialized".into();
        assert!(validate_trajectory_artifact_authority(&c).is_err());
        Ok(())
    }
    #[test]
    fn artifact_retention_indices_bind_pilot_ids_and_actual_eos() -> Result<()> {
        let indices = vec![2usize, 6, 10, 14, 18, 22, 26, 30];
        let full = json!({"rows":(0..512).map(|i|json!({"id":format!("case-{i}"),"complete":indices.contains(&i),"eos":indices.contains(&i)})).collect::<Vec<_>>()});
        let pilot = json!({"indices":indices,"evaluation":{"rows":indices.iter().map(|i|json!({"id":format!("case-{i}"),"complete":true,"eos":true})).collect::<Vec<_>>()}});
        assert_eq!(retained_indices(&pilot, &full)?, indices);
        let mut wrong = pilot.clone();
        wrong["evaluation"]["rows"][3]["id"] = json!("another-source-row");
        assert!(retained_indices(&wrong, &full).is_err());
        let mut no_eos = full.clone();
        no_eos["rows"][indices[0]]["eos"] = json!(false);
        assert!(retained_indices(&pilot, &no_eos).is_err());
        let mut duplicate = pilot.clone();
        duplicate["indices"][1] = duplicate["indices"][0].clone();
        assert!(retained_indices(&duplicate, &full).is_err());
        Ok(())
    }
    #[test]
    fn prefix_transaction_rejects_descent_with_lost_original_winner() -> Result<()> {
        let current = json!({"combined":4.0,"correct_reference_frames":17});
        assert!(!transaction_accept(
            &current,
            &json!({"combined":3.0,"correct_reference_frames":16})
        )?);
        assert!(!transaction_accept(
            &current,
            &json!({"combined":4.0,"correct_reference_frames":17})
        )?);
        assert!(transaction_accept(
            &current,
            &json!({"combined":3.9,"correct_reference_frames":17})
        )?);
        Ok(())
    }
    #[test]
    fn prefix_unmasked_lane_updates_every_physical_alias() -> Result<()> {
        let mut parent = vec![0f32; 960];
        parent[941] = -0.139;
        let mut changed = parent.clone();
        changed[941] = -0.5;
        let row = vec![
            Some(81),
            Some(156),
            Some(303),
            Some(424),
            Some(533),
            Some(712),
            Some(825),
            Some(941),
        ];
        let keys = vec![
            row.clone(),
            row,
            vec![
                Some(59),
                Some(176),
                Some(316),
                Some(383),
                Some(533),
                Some(691),
                Some(798),
                Some(935),
            ],
        ];
        let original = vec![7, 9, 11];
        let next = shared::staged_base(&original, &keys, &parent, &changed)?;
        assert_eq!(next, vec![7 - (1 << 22), 9 - (1 << 22), 11]);
        assert!(shared::staged_base(&original, &[vec![Some(960); 8]], &parent, &changed).is_err());
        Ok(())
    }
    #[test]
    fn prefix_late_export_error_restores_fractional_bits_and_frozen_cue() -> Result<()> {
        let prefix = Var::from_vec(vec![0.139f32, -0.0], 2, &Device::Cpu)?;
        let cue = Var::from_vec(vec![-0.14452013f32, -0.0], 2, &Device::Cpu)?;
        let vars = BTreeMap::from([
            (NAME.to_string(), prefix),
            ("cue.coefficients".to_string(), cue),
        ]);
        let original = np::snapshot(&vars)?;
        let result: Result<()> = np::attempt_restored(&vars, &original, || {
            let mut changed = original.clone();
            changed
                .get_mut(NAME)
                .ok_or_else(|| bad("test Prefix absent"))?[0] = 0.5;
            np::restore(&vars, &changed)?;
            Err(bad("late export/parity failure"))
        });
        assert!(result.is_err());
        assert!(np::same_bits(&original, &np::snapshot(&vars)?));
        Ok(())
    }
    #[test]
    fn episode_coalescing_preserves_reference_role_and_actual_gradient_weight() -> Result<()> {
        let mut roles = (0..15)
            .map(|position| shared::ObjectiveRole {
                input: 42,
                position,
                task: true,
                weight: 1. / 15.,
            })
            .collect::<Vec<_>>();
        roles.push(shared::ObjectiveRole {
            input: 42,
            position: 3,
            task: false,
            weight: 1. / 17.,
        });
        roles.extend((0..16).map(|input| shared::ObjectiveRole {
            input,
            position: 0,
            task: false,
            weight: 1. / 17.,
        }));
        let weights = shared::coalesced_role_weights(&roles)?;
        assert_eq!(roles.len(), 32);
        assert_eq!(weights.len(), 31);
        assert_eq!(weights[&(42, 3)], 1. / 15. + 1. / 17.);
        assert_eq!(roles.iter().filter(|r| !r.task).count(), 17);
        roles.push(roles[0].clone());
        assert!(shared::coalesced_role_weights(&roles).is_err());
        Ok(())
    }
    #[test]
    fn complete_episode_requires_termination_after_full_coverage() {
        let mut targets = vec![9; 15];
        targets[14] = 1;
        assert!(complete_episode_targets(&targets));
        targets[5] = 1;
        assert!(!complete_episode_targets(&targets));
        assert!(!complete_episode_targets(&targets[..7]));
    }
    #[test]
    fn episode_gate_rejects_reference_or_termination_loss() -> Result<()> {
        let b = json!({"combined":4.,"task":2.});
        let mut v =
            json!({"combined":3.,"task":1.,"correct_reference_frames":17,"all_phase_winners":true});
        assert_eq!(shared::episode_gate(&b, &v)?["finite_joint_positive"], true);
        v["all_phase_winners"] = json!(false);
        assert_eq!(
            shared::episode_gate(&b, &v)?["finite_joint_positive"],
            false
        );
        v["all_phase_winners"] = json!(true);
        v["correct_reference_frames"] = json!(16);
        assert_eq!(
            shared::episode_gate(&b, &v)?["finite_joint_positive"],
            false
        );
        Ok(())
    }
    #[test]
    fn canonical_continuation_counts_accept_known_formats_and_reject_conflict() -> Result<()> {
        let counts = json!({"coefficient_reads":32768,"prototype_reads":32768,"scores":4096});
        let legacy = json!({"field_counts":counts});
        let compact = json!({"counts":counts});
        assert_eq!(
            known_continuation_field_counts(&legacy)?,
            known_continuation_field_counts(&compact)?
        );
        let both = json!({"field_counts":counts,"counts":counts});
        assert_eq!(known_continuation_field_counts(&both)?, &counts);
        assert!(known_continuation_field_counts(
            &json!({"field_counts":counts,"counts":{"scores":1}})
        )
        .is_err());
        assert!(known_continuation_field_counts(&json!({})).is_err());
        assert!(known_continuation_field_counts(&json!({"counts":null})).is_err());
        assert!(known_continuation_field_counts(&json!({"counts":7})).is_err());
        Ok(())
    }
}
