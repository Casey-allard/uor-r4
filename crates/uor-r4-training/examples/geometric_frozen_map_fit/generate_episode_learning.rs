//! Offline shared Generate-unary complete-episode learning. No serving changes.
use super::context_cue_coadapt as shared;
use super::native_proposals as np;
use super::prefix_fragment_learning as prefix;
use super::*;
use uor_r4_integer::geometric_vocabulary_actions::{GeneratePatchCache, PendingGeneratePatch};
use uor_r4_training::geometric_generate_learning::vocabulary_marginal_loss_with_credit;
const NAME: &str = "generate.unary";
const COUNT: usize = 960;
const VOCAB: usize = 4096;
const GUARDS: usize = 380;
const UNION: usize = 391;
const CACHE_CAP: usize = 256 * 1024 * 1024;

#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Config {
    pub retained_episode_root: PathBuf,
    pub expected_report_sha256: String,
    pub expected_manifest_sha256: String,
    pub original_inputs: prefix::Config,
}
impl Config {
    pub(super) fn input_roots(&self) -> Vec<&PathBuf> {
        let mut out = vec![
            &self.retained_episode_root,
            &self.original_inputs.retained_intermediate_root,
            &self.original_inputs.retained_probe_root,
        ];
        if let Some(e) = &self.original_inputs.episode {
            out.extend([
                &e.typed_authority,
                &e.retained_projection.root,
                &e.retained_supplement_root,
            ]);
            out.extend(e.phases.iter().map(|p| &p.capture.root));
        }
        out
    }
}

#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CompletionConfig {
    pub retained_failed_root: PathBuf,
    pub retained_learning_runtime_root: PathBuf,
    pub expected_learning_runtime_identity_sha256: String,
    pub retained_learning_observation_root: PathBuf,
    pub expected_learning_observation_manifest_sha256: String,
    pub expected_report_sha256: String,
    pub expected_manifest_sha256: String,
    pub expected_learning_config_sha256: String,
    pub expected_learning_source_commit: String,
    pub expected_learning_binary_sha256: String,
    pub expected_constructor_sha256: String,
    pub expected_unary_master_sha256: String,
    pub original_inputs: prefix::Config,
}
impl CompletionConfig {
    pub(super) fn input_roots(&self) -> Vec<&PathBuf> {
        let mut roots = vec![
            &self.retained_failed_root,
            &self.retained_learning_runtime_root,
            &self.retained_learning_observation_root,
            &self.original_inputs.retained_intermediate_root,
            &self.original_inputs.retained_probe_root,
        ];
        if let Some(e) = &self.original_inputs.episode {
            roots.extend([
                &e.typed_authority,
                &e.retained_projection.root,
                &e.retained_supplement_root,
            ]);
            roots.extend(e.phases.iter().map(|p| &p.capture.root));
        }
        roots
    }
}
const FAILED_REPORT: &str = "fe11475c744759af845a30d5bb4938bfd2f2492366771ddee25c92002ae15625";
const FAILED_SEAL: &str = "4bd01644814324e8537a4228644c49ecd747b2d93a8f82662d8463bdaf1714cf";
const LEARNING_CONFIG: &str = "f504d1eee00abc8517e19cbb2c2e2e067e5529820d549ff3fb0c71e9fdea4120";
const LEARNING_SOURCE: &str = "bf09a1aa7419adee2ec12e6202565f13d08ca3ef";
const LEARNING_BINARY: &str = "d475bea4133a49beca1d97452fd41e769da2e620d3a2d99757e31b93ffbb309b";
const COMPLETED_CONSTRUCTOR: &str =
    "68e2e926c05de46eb929d986675e39a18c50a1cc300e5cd059fafebd7add22ff";
const COMPLETED_UNARY: &str = "324af524e78be9c34ea297995cca4c1006ce910dcdb5e157913d694861ef2c54";
pub(super) fn validate_completion_settings(a: &Args) -> Result<()> {
    if let Some(c) = &a.generate_episode_completion {
        replay_require(
            a.mode == Mode::JointContinuation
                && a.updates == 1
                && a.loss_scope == LossScope::All
                && a.generate_episode_learning.is_none()
                && a.prefix_fragment_learning.is_none()
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
            "Generate completion requires exclusive no-learning mode",
        )?;
        replay_require(
            c.expected_report_sha256 == FAILED_REPORT
                && c.expected_learning_runtime_identity_sha256
                    == "90273f2bbca9a2269f2edb789be0aa0157d3e6f9911b2ef510f3627fa79dd9bb"
                && c.expected_learning_observation_manifest_sha256
                    == "2a0e7014cdbbcd2ef753d7ab9e7030d8c0c8103ee23fe4f8fbd75938bfd5a051"
                && c.expected_manifest_sha256 == FAILED_SEAL
                && c.expected_learning_config_sha256 == LEARNING_CONFIG
                && c.expected_learning_source_commit == LEARNING_SOURCE
                && c.expected_learning_binary_sha256 == LEARNING_BINARY
                && c.expected_constructor_sha256 == COMPLETED_CONSTRUCTOR
                && c.expected_unary_master_sha256 == COMPLETED_UNARY,
            "Generate completion recorded authority differs",
        )?;
    }
    Ok(())
}
pub(super) fn validate_settings(a: &Args) -> Result<()> {
    if let Some(c) = &a.generate_episode_learning {
        replay_require(
            a.mode == Mode::JointContinuation
                && a.updates == 1
                && a.loss_scope == LossScope::All
                && a.prefix_fragment_learning.is_none()
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
            "Generate episode requires exclusive fresh unary mode",
        )?;
        replay_require(
            [&c.expected_report_sha256, &c.expected_manifest_sha256]
                .iter()
                .all(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit())),
            "Generate episode witness hashes invalid",
        )?;
        replay_require(
            c.original_inputs.episode.is_some()
                && c.original_inputs.joint.is_none()
                && c.original_inputs.trajectory.is_none(),
            "Generate original episode spec absent",
        )?;
        let mut inherited = a.clone();
        inherited.generate_episode_learning = None;
        inherited.prefix_fragment_learning = Some(c.original_inputs.clone());
        prefix::validate_settings(&inherited)?;
    }
    Ok(())
}
pub(super) fn policy() -> Value {
    json!({"schema":"uor-r4.generate-episode-learning/1","active_parameter_names":[NAME],
        "coordinate_count":COUNT,"alternatives_per_coordinate":14,
        "initialization":"ORIGINAL selected Source9f/G4248/U82ae/fractional Cue+Prefix+Generate; no negative candidate",
        "credit":"factual native coefficient-only quarter-shadow STE; fixed poststate/prototypes; no relaxed transport/donor selector",
        "construction":"frozen fresh-gradient best actual displacement order; all14 other legal codes per960, including initialzero/countergradient; one best feasible commit percoordinate, no rerank/revisit",
        "trial_gate":"strict current32-role native CE margin1e-10*(1+abs(current)) AND17refs/all380 original winners",
        "code_selection":"lowest feasible native combinedCE; exact numeric CE tie chooses smallest signed code",
        "final_gate":"strict ORIGINAL episode and combinedCE, all15 conditional winners/EOS,17refs/all380; actual9 typed answer/EOS separate",
        "native_score_unit_q24":1i64<<20,"frozen":"all other Generate masters/Source/Cue/Prefix/bridge/U coefficient bits",
        "gradient_context_encoder_calls":0,"captured_objective_encoder_calls":0,"native_reload_steps":391,"new_context_gradients":0,"new_prefix_gradients":0,"optimizer_updates":0})
}
// Strictly decode both authorities through the same schema. Omitted optional
// defaults and explicit null are equivalent; every typed nested identity remains
// part of the equality and unknown fields are rejected by Config.
fn inherited_inputs_match(raw: &Value, current: &prefix::Config) -> Result<bool> {
    let retained: prefix::Config = serde_json::from_value(raw.clone())?;
    Ok(serde_json::to_value(retained)? == serde_json::to_value(current)?)
}
fn code(m: f32) -> Result<i8> {
    replay_require(
        m.is_finite() && (-1.75..=1.75).contains(&m),
        "Generate unary master outside existing strict Q4 admission",
    )?;
    Ok((4. * m).round() as i8)
}
fn improves(before: f64, after: f64) -> bool {
    after < before - 1e-10 * (1. + before.abs())
}
fn lower(a: (f64, i8), b: (f64, i8)) -> bool {
    a.0 < b.0 || (a.0 == b.0 && a.1 < b.1)
}
fn coordinate_order(m: &[f32], g: &[f32]) -> Result<Vec<usize>> {
    replay_require(
        m.len() == COUNT && g.len() == COUNT,
        "Generate rank shape differs",
    )?;
    let mut rows = Vec::with_capacity(COUNT);
    for (i, (&m, &g)) in m.iter().zip(g).enumerate() {
        let q = code(m)?;
        replay_require(g.is_finite(), "Generate gradient nonfinite")?;
        let potential = (-7i8..=7)
            .filter(|c| *c != q)
            .map(|c| f64::from(g) * (f64::from(c) * 0.25 - f64::from(m)))
            .min_by(f64::total_cmp)
            .ok_or_else(|| bad("Generate legal code grid empty"))?;
        rows.push((i, potential));
    }
    rows.sort_by(|a, b| a.1.total_cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
    Ok(rows.into_iter().map(|r| r.0).collect())
}
fn saved_pool(f: &shared::Frame, reducer: &mut NativeVocabularyActions) -> Result<shared::Pool> {
    let generate: Vec<i64> = shared::dec(&f.native["generate_q24"])?;
    let copy: Vec<i64> = shared::dec(&f.native["copy_q24"])?;
    let trace = reducer.reduce_trace(&generate, &f.ids, &copy)?;
    replay_require(
        serde_json::to_value(&trace)? == f.native["pool"],
        "Generate original full pool parity differs",
    )?;
    Ok(shared::Pool {
        donor: shared::idx(&f.native["bridge"]["selected_ordinal"])?,
        post: shared::codes(&f.native["post_state"])?,
        generate,
        copy,
        base_copy: f.base_copy.clone(),
        trace,
    })
}
fn slim(frames: &mut [shared::Frame]) -> Result<()> {
    for f in frames {
        f.native["pool"]
            .as_object_mut()
            .ok_or_else(|| bad("Generate native pool missing"))?
            .remove("actions");
        f.native["pool_action_trace"]=json!("OMITTED_RECONSTRUCTIBLE_FROM_COMPLETE_SCORES after complete hard parity; all rawG/Copy/U/tokenmasses/summary retained");
    }
    Ok(())
}
fn gradient(
    a: &Args,
    start: Instant,
    p: &ContinuationParent,
    frames: &[shared::Frame],
    pools: &[shared::Pool],
    d: &Device,
) -> Result<Vec<f32>> {
    let ng = NativeGeometricGenerate::from_bytes(&p.generate, p.integer.binding())?;
    let g = GenerateLearningWeights::from_native(p.integer.binding().clone(), &ng, d)?;
    restore(
        &a.checkpoint.join("generate-source"),
        &read(&a.checkpoint.join("generate-source/metadata.json"))?["parameters"],
        &g.parameters(),
        d,
    )?;
    replay_require(
        g.export_native()?.to_bytes()? == p.generate,
        "Generate actual master restore/native differs",
    )?;
    let prepared = g.prepare_native()?;
    // Native factual parity for every physical objective precedes ANY backward.
    for (f, pool) in frames.iter().zip(pools) {
        let out = g.forward_prepared_coefficients_only(&prepared, &pool.post)?;
        replay_require(
            out.scores_q24
                .iter()
                .zip(&f.u)
                .map(|(g, u)| g.checked_add(*u))
                .collect::<Option<Vec<_>>>()
                == Some(pool.generate.clone()),
            "Generate coefficient graph raw native anchor differs",
        )?;
        replay_require(
            pool.copy == shared::dec::<Vec<i64>>(&f.native["copy_q24"])?
                && serde_json::to_value(&pool.trace)? == f.native["pool"],
            "Generate Copy/fullpool native parity differs",
        )?;
    }
    write(
        a,
        "generate-forward-parity.json",
        &json!({"physical_frames":frames.len(),"weighted_roles":32,
        "all_before_any_backward":true,"native_raw_generate_copy_full_alias_pool":true,
        "frozen_poststate":true,"relaxed_transport":"NOT_RUN","device":"cuda"}),
    )?;
    let mut aggregate = vec![0f32; COUNT];
    let mut weighted_forward_ce = 0f64;
    let mut terms = Vec::new();
    for (i, (f, pool)) in frames.iter().zip(pools).enumerate() {
        shared::progress(a, start)?;
        let out = g.forward_prepared_coefficients_only(&prepared, &pool.post)?;
        let u = Tensor::from_vec(
            f.u.iter()
                .map(|x| (*x as f64 / 16777216.) as f32)
                .collect::<Vec<_>>(),
            VOCAB,
            d,
        )?;
        let hard = Tensor::from_vec(
            pool.generate
                .iter()
                .map(|x| (*x as f64 / 16777216.) as f32)
                .collect::<Vec<_>>(),
            VOCAB,
            d,
        )?;
        let soft = (&out.raw_scores + &u)?;
        let joined = (&hard + (&soft - soft.detach())?)?;
        let copy = Tensor::from_vec(
            pool.copy
                .iter()
                .map(|x| (*x as f64 / 16777216.) as f32)
                .collect::<Vec<_>>(),
            pool.copy.len(),
            d,
        )?;
        let loss = (vocabulary_marginal_loss_with_credit(
            &pool.trace,
            &joined,
            Some(&copy),
            f.target,
            a.credit.policy(),
        )? * f.weight)?;
        weighted_forward_ce += loss.to_scalar::<f32>()? as f64;
        let grads = loss.backward()?;
        let v = grads
            .get(g.unary.as_tensor())
            .ok_or_else(|| bad("Generate unary gradient MISSING; no fill"))?
            .flatten_all()?
            .to_device(&Device::Cpu)?
            .to_vec1::<f32>()?;
        replay_require(
            v.len() == COUNT && v.iter().all(|x| x.is_finite()),
            "Generate unary raw gradient shape/nonfinite",
        )?;
        for (sum, x) in aggregate.iter_mut().zip(&v) {
            *sum += *x;
        }
        let bytes = v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>();
        let file = format!("generate-gradient-term-{i:02}.f32le");
        fs::write(a.out.join(&file), &bytes)?;
        terms.push(json!({"physical_index":i,"input_index":f.input,"position":f.position,"id":f.id,
            "target":f.target,"weight":f.weight,"file":file,"bytes":bytes.len(),"sha256":sha256_bytes(&bytes),
            "status":"PRESENT","missing_gradient_filled_zero":false,"all_zero":v.iter().all(|x|*x==0.),
            "l2_norm":v.iter().map(|x|f64::from(*x).powi(2)).sum::<f64>().sqrt(),
            "termination_target":f.target==1,"period_target":f.target==16}));
    }
    let native_ce = read(&a.out.join("initial-original-objective.json"))?["combined"]
        .as_f64()
        .ok_or_else(|| bad("Generate native baseline CE missing"))?;
    replay_require(
        (weighted_forward_ce - native_ce).abs() < 1e-5,
        "Generate floating anchored CE differs from native complete objective",
    )?;
    replay_require(
        aggregate.iter().all(|x| x.is_finite()),
        "Generate aggregate nonfinite",
    )?;
    let bytes = aggregate
        .iter()
        .flat_map(|x| x.to_le_bytes())
        .collect::<Vec<_>>();
    fs::write(a.out.join("generate-gradient.f32le"), &bytes)?;
    write(
        a,
        "generate-gradient-receipt.json",
        &json!({"active_parameter_names":[NAME],"shape":[8,120],
        "coefficient_backward_calls":frames.len(),"weighted_roles":32,"native_combined_ce":native_ce,"float_graph_combined_ce":weighted_forward_ce,"loss_anchor_tolerance":1e-5,"perterm":terms,
        "aggregate":{"file":"generate-gradient.f32le","bytes":bytes.len(),"sha256":sha256_bytes(&bytes)},
        "sum":"ordered physical-frame f32 sum; duplicate task/reference coalesced weight, not32-backward bitwise equivalence",
        "frozen_other_generate_masters":identities(&BTreeMap::from([
            ("generate.pair".into(),g.pair.clone()),("generate.bias".into(),g.bias.clone()),
            ("generate.prototype_choices".into(),g.prototype_choices.clone())]))?}),
    )?;
    // g/prepared/graphs released before allocating the complete guard population.
    Ok(aggregate)
}
struct Incidence {
    offsets: Vec<usize>,
    atoms: Vec<u32>,
}
fn incidence(
    ng: &NativeGeometricGenerate,
    frames: &[shared::Frame],
    pools: &[shared::Pool],
    legal: &[u32],
) -> Result<Incidence> {
    replay_require(
        frames.len() <= UNION && legal.iter().all(|t| (*t as usize) < VOCAB),
        "Generate incidence domain differs",
    )?;
    let mut counts = vec![0usize; COUNT];
    for pool in pools {
        let state = pool.post.clone();
        for &token in legal {
            let mut keys = [0u32; 12];
            ng.factor_incidence_into(&state, token as usize, &mut keys, &mut Default::default())?;
            for &key in &keys[..8] {
                counts[key as usize] += 1;
            }
        }
    }
    let mut offsets = vec![0usize; COUNT + 1];
    for i in 0..COUNT {
        offsets[i + 1] = offsets[i] + counts[i];
    }
    let mut atoms = vec![0u32; offsets[COUNT]];
    let mut cursor = offsets[..COUNT].to_vec();
    for (row, pool) in pools.iter().enumerate() {
        let state = pool.post.clone();
        for &token in legal {
            let mut keys = [0u32; 12];
            ng.factor_incidence_into(&state, token as usize, &mut keys, &mut Default::default())?;
            for &key in &keys[..8] {
                let k = key as usize;
                atoms[cursor[k]] = ((row as u32) << 12) | token;
                cursor[k] += 1;
            }
        }
    }
    Ok(Incidence { offsets, atoms })
}
type Patches = BTreeMap<usize, PendingGeneratePatch>;
fn view(
    row: usize,
    target: u32,
    caches: &[GeneratePatchCache],
    staged: &Patches,
) -> Result<(u64, u64, u32)> {
    if let Some(p) = staged.get(&row) {
        let s = p.summary();
        Ok((
            p.token_mass(&caches[row], target)?,
            s.total_weight_q31,
            s.chosen_token_id,
        ))
    } else {
        let c = &caches[row];
        Ok((
            c.token_masses()[target as usize],
            c.summary().total_weight_q31,
            c.summary().chosen_token_id,
        ))
    }
}
fn objective(
    frames: &[shared::Frame],
    map: &[usize],
    spec: &shared::ObjectiveSpec,
    caches: &[GeneratePatchCache],
    staged: &Patches,
) -> Result<Value> {
    let roles = spec
        .roles
        .as_ref()
        .ok_or_else(|| bad("Generate explicit episode roles absent"))?;
    let mut task = 0.;
    let mut reference = 0.;
    let mut goodrefs = 0;
    let mut phases = Vec::new();
    let mut masses = Vec::new();
    for &row in map {
        let f = &frames[row];
        let (m, t, c) = view(row, f.target, caches, staged)?;
        replay_require(m > 0 && t > 0, "Generate native objective zero target mass")?;
        masses.push(json!([m, t, c]));
    }
    for role in roles {
        let row = map
            .iter()
            .copied()
            .find(|&row| frames[row].input == role.input && frames[row].position == role.position)
            .ok_or_else(|| bad("Generate objective role frame missing"))?;
        let f = &frames[row];
        let (mass, total, chosen) = view(row, f.target, caches, staged)?;
        let ce = -(mass as f64 / total as f64).ln() * role.weight;
        if role.task {
            task += ce;
            phases.push(json!({"position":f.position,"target":f.target,"chosen":chosen,"target_mass":mass,"total_mass":total}));
        } else {
            reference += ce;
            goodrefs += usize::from(chosen == f.target);
        }
    }
    Ok(
        json!({"combined":task+reference,"task":task,"reference":reference,
        "correct_reference_frames":goodrefs,"phases":phases,"objective_masses":masses,
        "all_phase_winners":phases.iter().all(|x|x["target"]==x["chosen"])}),
    )
}
fn valid_objective(current: &Value, next: &Value) -> Result<bool> {
    Ok(improves(
        current["combined"]
            .as_f64()
            .ok_or_else(|| bad("Generate currentCE absent"))?,
        next["combined"]
            .as_f64()
            .ok_or_else(|| bad("Generate nextCE absent"))?,
    ) && next["correct_reference_frames"] == 17)
}
fn final_gate(original: &Value, next: &Value, guards: bool) -> Result<Value> {
    let combined = improves(
        original["combined"]
            .as_f64()
            .ok_or_else(|| bad("Generate baselineCE absent"))?,
        next["combined"]
            .as_f64()
            .ok_or_else(|| bad("Generate finalCE absent"))?,
    );
    let task = improves(
        original["task"]
            .as_f64()
            .ok_or_else(|| bad("Generate baseline taskCE absent"))?,
        next["task"]
            .as_f64()
            .ok_or_else(|| bad("Generate final taskCE absent"))?,
    );
    Ok(
        json!({"strict_combined_descent":combined,"strict_episode_descent":task,"all15_conditional_winners":next["all_phase_winners"],
        "all17_references":next["correct_reference_frames"]==17,"all380_original_winners":guards,
        "passed":combined && task && next["all_phase_winners"]==true && next["correct_reference_frames"]==17 && guards,
        "actual_wholeanswer_EOS":"NOT_RUN_SEPARATE_QUALIFICATION"}),
    )
}
fn stage_row(
    row: usize,
    changes: &BTreeMap<usize, Vec<(u32, i64)>>,
    caches: &[GeneratePatchCache],
    staged: &mut Patches,
    reducer: &mut NativeVocabularyActions,
) -> Result<()> {
    if let Some(delta) = changes.get(&row) {
        if !staged.contains_key(&row) {
            staged.insert(row, reducer.evaluate_generate_patch(&caches[row], delta)?);
        }
    }
    Ok(())
}
fn digest_staged(
    staged: &Patches,
    frames: &[shared::Frame],
    caches: &[GeneratePatchCache],
) -> Result<String> {
    let rows = staged
        .iter()
        .map(|(&i, p)| {
            let s = p.summary();
            Ok(json!({"row":i,"reference_q24":s.reference_q24,
            "total_weight_q31":s.total_weight_q31,"chosen_token_id":s.chosen_token_id,
            "chosen_weight_q31":s.chosen_weight_q31,
            "target_mass":p.token_mass(&caches[i],frames[i].target)?}))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(sha256_bytes(&serde_json::to_vec(&rows)?))
}
fn construct(
    a: &Args,
    start: Instant,
    frames: &[shared::Frame],
    map: &[usize],
    spec: &shared::ObjectiveSpec,
    master: &[f32],
    grad: &[f32],
    csr: &Incidence,
    caches: &mut [GeneratePatchCache],
    reducer: &mut NativeVocabularyActions,
) -> Result<(Vec<f32>, Value)> {
    let order = coordinate_order(master, grad)?;
    write(a, "generate-coordinate-order.json", &json!(order))?;
    let baseline = objective(frames, map, spec, caches, &Patches::new())?;
    let mut current = master.to_vec();
    let mut value = baseline.clone();
    let mut epoch = 0u64;
    use std::io::Write as _;
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(a.out.join("generate-construction.json"))?;
    let mut records = std::io::BufWriter::new(file);
    records.write_all(b"{\"policy\":")?;
    serde_json::to_writer(&mut records, &policy())?;
    records.write_all(b",\"coordinate_records\":[")?;
    let mut row_patches = 0u64;
    let mut accepted = 0;
    for (ordinal, &coordinate) in order.iter().enumerate() {
        shared::progress(a, start)?;
        let incumbent = code(current[coordinate])?;
        let old_epoch = epoch;
        let mut alternatives = Vec::with_capacity(14);
        let mut best: Option<(f64, i8, Patches, Value)> = None;
        for proposed in -7i8..=7 {
            if proposed == incumbent {
                continue;
            }
            let d = i64::from(proposed - incumbent) * (1 << 20);
            let mut changes = BTreeMap::<usize, Vec<(u32, i64)>>::new();
            for &atom in &csr.atoms[csr.offsets[coordinate]..csr.offsets[coordinate + 1]] {
                let row = (atom >> 12) as usize;
                let token = atom & 4095;
                let raw = caches[row].generate_scores()[token as usize]
                    .checked_add(d)
                    .ok_or_else(|| bad("Generate unary patch overflow"))?;
                changes.entry(row).or_default().push((token, raw));
            }
            let mut staged = Patches::new();
            for &row in map {
                stage_row(row, &changes, caches, &mut staged, reducer)?;
            }
            let next = objective(frames, map, spec, caches, &staged)?;
            let objective_gate = valid_objective(&value, &next)?;
            let mut checked = Vec::new();
            let mut first: Option<Value> = None;
            if objective_gate {
                for row in 0..GUARDS {
                    if changes.contains_key(&row) {
                        stage_row(row, &changes, caches, &mut staged, reducer)?;
                        checked.push(row);
                    }
                    let (_, _, chosen) = view(row, frames[row].target, caches, &staged)?;
                    if chosen != frames[row].target {
                        first = Some(
                            json!({"row":row,"input_index":frames[row].input,"position":frames[row].position,
                            "required":frames[row].target,"chosen":chosen}),
                        );
                        break;
                    }
                }
            }
            row_patches += staged.len() as u64;
            let feasible = objective_gate && first.is_none();
            let compact_digest = digest_staged(&staged, frames, caches)?;
            alternatives.push(json!({"code":proposed,"delta_code":proposed-incumbent,
                "actual_master_delta":f64::from(proposed)*0.25-f64::from(master[coordinate]),
                "combined":next["combined"],"task":next["task"],"reference":next["reference"],
                "objective_masses":next["objective_masses"],"objective_gate":objective_gate,
                "guard_status":if !objective_gate{"NOT_RUN_OBJECTIVE_INELIGIBLE"}else if first.is_some(){"VETO"}else{"PASS"},
                "checked_guard_ids":checked,"first_guard_failure":first,"feasible":feasible,
                "staged_summary_digest":compact_digest,"changed_atoms":changes.values().map(Vec::len).sum::<usize>(),
                "row_patches":staged.len(),"used_full_reduction_count":staged.values().filter(|p|p.summary().used_full_reduction).count(),
                "maximum_scans":staged.values().filter(|p|p.summary().scanned_maximum).count(),
                "winner_scans":staged.values().filter(|p|p.summary().scanned_winner).count(),
                "potential_affected_rows":changes.len(),"changed_atoms_scope":"all CSR affectedatoms; guard-veto/objective-ineligible stage can visit only subset"}));
            if feasible {
                let ce = next["combined"]
                    .as_f64()
                    .ok_or_else(|| bad("Generate alternativeCE absent"))?;
                if best
                    .as_ref()
                    .is_none_or(|b| lower((ce, proposed), (b.0, b.1)))
                {
                    best = Some((ce, proposed, staged, next));
                }
            }
        }
        replay_require(
            alternatives.len() == 14,
            "Generate full legal alternatives count differs",
        )?;
        let selected = if let Some((ce, q, pending, next)) = best {
            // Consumer validates every opaque identity/revision before any infallible application.
            reducer.commit_generate_patch_batch(caches, pending.into_iter().collect())?;
            current[coordinate] = f32::from(q) * 0.25;
            value = next;
            epoch += 1;
            accepted += 1;
            json!({"code":q,"combined":ce,"status":"committed","restaged_rows":0})
        } else {
            json!({"code":incumbent,"status":"unchanged","restaged_rows":0})
        };
        let record = json!({"order":ordinal,"index":coordinate,"incumbent_epoch":old_epoch,
            "incumbent_code":incumbent,"original_master":master[coordinate],"gradient":grad[coordinate],
            "initial_zero_gradient":grad[coordinate]==0.,"alternatives":alternatives,
            "selected":selected,"epoch_after":epoch});
        if ordinal > 0 {
            records.write_all(b",")?;
        }
        serde_json::to_writer(&mut records, &record)?;
        records.flush()?;
    }
    records.write_all(b"],\"summary\":")?;
    serde_json::to_writer(
        &mut records,
        &json!({"coordinates":COUNT,"alternatives":COUNT*14,
        "accepted_coordinates":accepted,"row_patches":row_patches,"initial":baseline,"final":value,
        "revisited_coordinates":0,"selected_pending_restage_count":0}),
    )?;
    records.write_all(b"}\n")?;
    records.flush()?;
    Ok((current, value))
}

fn compact_numeric_projection(prior: &Value) -> Result<u64> {
    // Remove the authenticated previous donor-cache allocation, which does not
    // exist in Generate-only construction. Account for global unary CSR, caches,
    // two simultaneous worst-case full pending batches, and compact trial Values.
    let old = prior["numeric_upper_bound"]
        .as_u64()
        .ok_or_else(|| bad("retained numeric projection absent"))?;
    let donor = prior["donor_cache_cap"]
        .as_u64()
        .ok_or_else(|| bad("retained donor cache projection absent"))?;
    let old_pools = prior["actual_retained_guard_pool_bytes"]
        .as_u64()
        .ok_or_else(|| bad("retained guard pool bound absent"))?;
    // Existing old projection retains both current/staged guard pools. These
    // are released before the all-code constructor; numerical scores live only
    // in opaque caches. Typed immutable donor/state/Copy/U witnesses remain.
    let removed = donor + 2 * old_pools;
    let additions = (UNION*VOCAB*8*4)as u64 // packed unary CSR
        +(UNION*VOCAB*8*3)as u64 // cache Generate/weights/masses
        +2*(UNION*VOCAB*8*3)as u64 // best and currently staged full fallback
        +16*1024*1024 // streamed one-coordinate alternative receipts/buffers
        +8*1024*1024; // offsets/atoms groups/container margin
    old.checked_sub(removed)
        .and_then(|x| x.checked_add(additions))
        .ok_or_else(|| bad("Generate numeric phase bound overflow"))
}
fn regular_file_bytes(path: &Path) -> Result<u64> {
    let metadata = fs::metadata(path)?;
    replay_require(
        metadata.is_file(),
        "Generate report projection expected a regular file",
    )?;
    Ok(metadata.len())
}
fn resource_projection(a: &Args, c: &Config, frames: &[shared::Frame]) -> Result<()> {
    let previous = read(
        &c.retained_episode_root
            .join("trajectory-resource-projection.json"),
    )?;
    let previous_pre = read(
        &c.retained_episode_root
            .join("episode-pregradient-resource-projection.json"),
    )?;
    let full = frames.iter().try_fold(0u64, |sum, f| {
        Ok::<_, Box<dyn std::error::Error>>(sum + serde_json::to_vec(&f.native)?.len() as u64)
    })?;
    let numeric = compact_numeric_projection(&previous)?;
    let phase_ram = previous_pre["process_ram_projection_bytes"]
        .as_u64()
        .ok_or_else(|| bad("retained measured phase RAM projection absent"))?;
    let process = phase_ram + 64 * 1024 * 1024;
    let previous_total = size(&c.retained_episode_root)?;
    let removed_report =
        regular_file_bytes(&c.retained_episode_root.join("prefix-construction.json"))?;
    let report = previous_total
        .checked_sub(removed_report)
        .and_then(|n| n.checked_add(64 * 1024 * 1024 + 16 * 1024 * 1024))
        .ok_or_else(|| bad("Generate report projection overflow"))?;
    // The new cache/CSR coexistence does not include old donor vectors or graphs.
    let cache = (UNION * VOCAB * 8 * 4 + UNION * VOCAB * 8 * 3 + 2 * UNION * VOCAB * 8 * 3) as u64
        + 8 * 1024 * 1024;
    write(
        a,
        "generate-resource-projection.json",
        &json!({"prior_projection":previous,
        "prior_pregradient_projection":previous_pre,"actual_original31_serialized_bytes":full,
        "numeric_upper_bound":numeric,"numeric_cap":512*1024*1024u64,
        "cache_upper_bound":cache,"cache_cap":CACHE_CAP,"process_ram_projection_bytes":process,
        "process_ram_cap":4*1024*1024*1024u64,"report_projection_bytes":report,"report_cap":a.maximum_report_bytes,
        "retained_previous_output_bytes":previous_total,"replaced_previous_construction_bytes":removed_report,
        "compact_alternative_receipt_reserve":16*1024*1024u64,"construction_receipts":"streamed coordinate records, never retain13440 alternative Values in RAM","projected_alternatives":13440,
        "graph_lifetime":"31 streamed coefficient graphs; Generate/device/prepared tensors dropped before guard380 load",
        "constructor_lifetime":"guard full pool buffers dropped after opaque cache admission, before CSR and all-code construction",
        "cache_scope":"CSR+opaque caches+best pending+current pending+container overhead; model/device RAM is separate",
        "fresh_backwards_completed":0,"estimate_not_hard_stop":true}),
    )?;
    replay_require(
        numeric <= 512 * 1024 * 1024
            && cache <= CACHE_CAP as u64
            && process <= 4 * 1024 * 1024 * 1024
            && report + 1048576 < a.maximum_report_bytes,
        "Generate complete numerical/export phase projection exceeded",
    )?;
    Ok(())
}
fn compact_pool(pool: &mut shared::Pool) {
    pool.trace.actions.clear();
    pool.trace.actions.shrink_to_fit();
}
pub(super) fn run(a: &Args, start: Instant, d: &Device) -> Result<Value> {
    validate_settings(a)?;
    replay_require(!d.is_cpu(), "Fresh Generate coefficients require CUDA")?;
    let c = a
        .generate_episode_learning
        .as_ref()
        .ok_or_else(|| bad("Generate config absent"))?;
    let prior = shared::sealed(
        &c.retained_episode_root,
        &c.expected_report_sha256,
        &c.expected_manifest_sha256,
    )?;
    replay_require(
        prior["status"] == "COMPLETED"
            && prior["mode"] == "prefix_episode_progression_learning"
            && prior["weighted_roles"] == 32
            && prior["unique_objective_frames"] == 31
            && prior["all_original380_preserved"] == true,
        "Generate retained episode authority differs",
    )?;
    replay_require(
        inherited_inputs_match(
            &read(&c.retained_episode_root.join("config.json"))?["prefix_fragment_learning"],
            &c.original_inputs,
        )?,
        "Generate inherited original inputs differ from sealed episode",
    )?;
    let original = ContinuationParent::from_checkpoint(&a.checkpoint)?;
    replay_require(
        original.binding.metadata_sha256 == shared::SOURCE
            && sha256_bytes(&original.generate) == shared::G_SHA
            && sha256_file(&a.checkpoint.join("continuation-field.bin"))? == shared::U_SHA,
        "Generate original Source/G/U differs",
    )?;
    let ng = NativeGeometricGenerate::from_bytes(&original.generate, original.integer.binding())?;
    let field = NativeContinuationField::from_bytes(
        &fs::read(a.checkpoint.join("continuation-field.bin"))?,
        &original.binding,
        &ng,
    )?;
    let (mut objective_frames, spec) =
        prefix::prepare_generate_objective(a, &c.original_inputs, &original)?;
    replay_require(
        objective_frames.len() == 31 && spec.roles.as_ref().is_some_and(|r| r.len() == 32),
        "Generate role/physical population differs",
    )?;
    let mut reducer =
        NativeVocabularyActions::new(original.integer.binding().clone(), &original.exp)?;
    let mut objective_pools = objective_frames
        .iter()
        .map(|f| saved_pool(f, &mut reducer))
        .collect::<Result<Vec<_>>>()?;
    for f in &objective_frames {
        let state = shared::codes(&f.native["continuation"]["state_codes"])?;
        let mut u = vec![0; VOCAB];
        field.score_delta_into(&state, &ng, &mut u, &mut Default::default())?;
        replay_require(u == f.u, "Generate frozen U state/score arithmetic differs")?;
    }
    let baseline = shared::objective_for_spec(&objective_frames, &objective_pools, &spec)?;
    write(a, "initial-original-objective.json", &baseline)?;
    resource_projection(a, c, &objective_frames)?;
    let g = gradient(a, start, &original, &objective_frames, &objective_pools, d)?;
    // Full graph/native parity is complete. Remove redundant action Values before
    // the protected population loader, retaining the p3 addition authority.
    slim(&mut objective_frames)?;
    for pool in &mut objective_pools {
        compact_pool(pool);
    }
    let (mut frames, mut pools, authority) = prefix::prepare_generate_guards(
        a,
        &c.original_inputs,
        &original,
        &objective_frames,
        &spec,
    )?;
    replay_require(
        frames.len() == GUARDS && pools.len() == GUARDS,
        "Generate guard population differs",
    )?;
    let mut map = Vec::with_capacity(31);
    for (f, pool) in objective_frames.into_iter().zip(objective_pools) {
        if let Some(row) = frames
            .iter()
            .position(|g| g.input == f.input && g.position == f.position)
        {
            replay_require(
                frames[row].id == f.id
                    && frames[row].prefix == f.prefix
                    && pools[row].generate == pool.generate
                    && pools[row].copy == pool.copy
                    && pools[row].post == pool.post
                    && pools[row].donor == pool.donor,
                "Generate overlapping objective/guard epoch differs",
            )?;
            frames[row] = f;
            map.push(row);
        } else {
            map.push(frames.len());
            frames.push(f);
            pools.push(pool);
        }
    }
    replay_require(
        frames.len() == UNION
            && pools.len() == UNION
            && frames
                .iter()
                .map(|f| (f.input, f.position))
                .collect::<BTreeSet<_>>()
                .len()
                == UNION,
        "Generate391 unique union differs",
    )?;
    let identities_json=frames.iter().enumerate().map(|(i,f)|json!({"row":i,"input_index":f.input,
        "position":f.position,"id":f.id,"actual_prefix_ids":f.prefix,"target_label_only":f.target,
        "zero_weight_guard":i<GUARDS,"guard_weight":0.,"coalesced_objective_weight":f.weight,
        "post_state":pools[i].post.iter().map(|x|x.index()).collect::<Vec<_>>(),"donor":pools[i].donor}))
        .collect::<Vec<_>>();
    write(
        a,
        "generate-population.json",
        &json!({"rows":identities_json,"objective_row_map":map,
        "roles":spec.roles,"guard_population":authority,"objective_physical_frames":31,
        "weighted_roles":32,"guards":GUARDS,"unique_union":UNION}),
    )?;
    let legal = reducer.legal_token_ids().to_vec();
    let csr = incidence(&ng, &frames, &pools, &legal)?;
    let bytes = csr
        .atoms
        .iter()
        .flat_map(|x| x.to_le_bytes())
        .collect::<Vec<_>>();
    write(
        a,
        "generate-incidence.json",
        &json!({"legal_generate_ids":legal,"postings":csr.atoms.len(),
        "offsets":csr.offsets,"packed_atoms_sha256":sha256_bytes(&bytes),
        "key_encoding":"lane*120+compose(inverse(poststate[lane]),prototype[token,lane]); first8 core factor_incidence_into keys",
        "atom_encoding":"row<<12|token; legal admitted IDs only, full raw4096 arrays retained",
        "source_binding":original.binding,"generate_sha256":shared::G_SHA}),
    )?;
    drop(bytes);
    let mut caches = pools
        .iter()
        .zip(&frames)
        .map(|(p, f)| {
            reducer.prepare_generate_patch_cache(p.generate.clone(), f.ids.clone(), p.copy.clone())
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;
    for (cache, pool) in caches.iter().zip(&pools) {
        replay_require(
            cache.summary().chosen_token_id == pool.trace.summary.chosen_token_id
                && cache.summary().total_weight_q31 == pool.trace.summary.total_weight_q31
                && pool
                    .trace
                    .token_masses
                    .iter()
                    .all(|m| cache.token_masses()[m.token_id as usize] == m.weight_q31),
            "Generate cache full masses differ",
        )?;
    }
    // Save only immutable post/donor witnesses. All current numerical state lives
    // in the target-free caches, not duplicate current/staged Pool buffers.
    let posts = pools.iter().map(|p| p.post.clone()).collect::<Vec<_>>();
    let donors = pools.iter().map(|p| p.donor).collect::<Vec<_>>();
    drop(pools);
    for f in &mut frames {
        let continuation = f.native["continuation"].clone();
        f.native = json!({"continuation":continuation});
        f.prefix_trace = None;
    }
    let initial = objective(&frames, &map, &spec, &caches, &Patches::new())?;
    replay_require(
        (initial["combined"]
            .as_f64()
            .ok_or_else(|| bad("Generate baseline absent"))?
            - baseline["combined"]
                .as_f64()
                .ok_or_else(|| bad("Generate saved baseline absent"))?)
        .abs()
            < 1e-12,
        "Generate cache original objective differs",
    )?;
    let actual = GenerateLearningWeights::from_native(
        original.integer.binding().clone(),
        &ng,
        &Device::Cpu,
    )?;
    restore(
        &a.checkpoint.join("generate-source"),
        &read(&a.checkpoint.join("generate-source/metadata.json"))?["parameters"],
        &actual.parameters(),
        &Device::Cpu,
    )?;
    let all_bits = np::snapshot(&actual.parameters())?;
    let master = all_bits
        .get(NAME)
        .ok_or_else(|| bad("Generate unary master absent"))?
        .clone();
    replay_require(
        master.len() == COUNT && actual.export_native()?.to_bytes()? == original.generate,
        "Generate actual fractional authority differs",
    )?;
    fs::write(
        a.out.join("generate-initial-masters.f32le"),
        master
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    )?;
    write(
        a,
        "generate-original-master-binding.json",
        &json!({"source":a.checkpoint.join("generate-source"),
        "parameters":identities(&actual.parameters())?,"native_sha256":shared::G_SHA,
        "all_original_f32_restored_before_graph":true,"active_parameter_names":[NAME]}),
    )?;
    drop(actual);
    let (current, value) = construct(
        a,
        start,
        &frames,
        &map,
        &spec,
        &master,
        &g,
        &csr,
        &mut caches,
        &mut reducer,
    )?;
    let all_guards = (0..GUARDS).all(|r| caches[r].summary().chosen_token_id == frames[r].target);
    let gate = final_gate(&initial, &value, all_guards)?;
    write(a, "final-objective.json", &value)?;
    write(
        a,
        "final-trajectory-guards.json",
        &json!({"all_original_winners":all_guards,"guards":GUARDS,
        "terms":(0..GUARDS).map(|i|json!({"guard_index":i,"input_index":frames[i].input,
            "position":frames[i].position,"required_original_winner":frames[i].target,
            "chosen":caches[i].summary().chosen_token_id,"pool":caches[i].summary()})).collect::<Vec<_>>()}),
    )?;
    // Coherent native artifact is retained regardless of finite gate outcome;
    // no automatic autoregressive rollout is performed by this runner.
    drop(csr);
    drop(g);
    let loaded = load_joint_continuation(a, &original, &Device::Cpu)?;
    let frozen_source = identities(&loaded.source.parameters())?;
    let params = loaded.generate.parameters();
    let saved = np::snapshot(&params)?;
    replay_require(
        np::same_bits(&saved, &all_bits),
        "Generate independently loaded original masters differ",
    )?;
    let u = ContinuationLearningWeights::from_native(
        &field,
        &ng,
        original.integer.binding(),
        &Device::Cpu,
    )?;
    restore(
        &a.checkpoint.join("continuation-source"),
        &original.receipt["continuation_parameters"],
        &u.parameters(),
        &Device::Cpu,
    )?;
    let u_bits = identities(&u.parameters())?;
    let receipt = np::attempt_restored(&params, &saved, || {
        let mut expected = saved.clone();
        expected.insert(NAME.into(), current.clone());
        np::restore(&params, &expected)?;
        replay_require(
            np::same_bits(&np::snapshot(&params)?, &expected)
                && identities(&loaded.source.parameters())? == frozen_source,
            "Generate sole-family/frozen master guard differs",
        )?;
        let (cp, rebound, mut receipt) = joint_checkpoint(a, 1, &loaded, &u)?;
        replay_require(
            cp.binding == original.binding
                && cp.bridge == original.bridge
                && rebound.packed_unary() == field.packed_unary()
                && identities(&u.parameters())? == u_bits,
            "Generate export changed Source/bridge/U numerical coefficients",
        )?;
        // Generic checkpoint stores sidecar native payloads. Restore the ORIGINAL
        // sidecar floating-master files too, whose Source/Cue bindings are fixed.
        for family in ["cue", "prefix"] {
            let source = a.checkpoint.join(family);
            let dest = a.out.join("checkpoint-0001").join(family);
            for entry in fs::read_dir(&source)? {
                let entry = entry?;
                if entry.file_type()?.is_file() {
                    let leaf = entry.file_name();
                    restore_original_sidecar_file(
                        &entry.path(),
                        &dest.join(&leaf),
                        leaf.to_str() == Some("native-metadata.json"),
                    )?;
                }
            }
        }
        for family in ["cue", "prefix"] {
            exact_frozen_directory(
                &a.checkpoint.join(family),
                &a.out.join("checkpoint-0001").join(family),
            )?;
        }
        for family in ["cue-source", "prefix-source"] {
            if a.checkpoint.join(family).exists() {
                copy_directory(
                    &a.checkpoint.join(family),
                    &a.out.join("checkpoint-0001").join(family),
                )?;
            }
        }
        let mut receipt_expected = expected.clone();
        let disk_g = GenerateLearningWeights::from_native(
            cp.integer.binding().clone(),
            &NativeGeometricGenerate::from_bytes(&cp.generate, cp.integer.binding())?,
            &Device::Cpu,
        )?;
        restore(
            &a.out.join("checkpoint-0001/generate-source"),
            &read(&a.out.join("checkpoint-0001/generate-source/metadata.json"))?["parameters"],
            &disk_g.parameters(),
            &Device::Cpu,
        )?;
        replay_require(
            np::same_bits(&np::snapshot(&disk_g.parameters())?, &receipt_expected),
            "Generate exported all floating masters differ",
        )?;
        receipt_expected.clear();
        drop(disk_g);
        receipt["mode"] = json!("generate_episode_learning");
        receipt["policy"] = policy();
        receipt["active_parameter_names"] = json!([NAME]);
        receipt["optimizer_updates"] = json!(0);
        receipt["fresh_adam"] = json!(false);
        receipt["new_gradients"] = json!(1);
        receipt["coefficient_backward_calls"] = json!(31);
        receipt["credit_scope"]=json!("only Generate unary960 gradients extracted/proposed; fixed factual native state/prototypes coefficient-only Q4 STE; frozen pair/bias graph may receive incidental autodiff, never extracted/updated; no Prefix/Cue/Context/donor surrogate");
        receipt["parent_report_sha256"] = json!(shared::P_REPORT);
        receipt["parent_manifest_sha256"] = json!(shared::P_SEAL);
        receipt["frozen_numerical_scope"]=json!("all Source/Cue/Prefix/bridge/Generate pair+bias+prototypes/U masters; only unary960 float/native codes change; U metadata honestly rebound to new Generate");
        receipt["generate_sha256"] = json!(sha256_bytes(&cp.generate));
        receipt["candidate_native_steps"] = json!(UNION);
        for leaf in [
            "checkpoint-0001/receipt.json",
            "checkpoint-0001/continuation-source/metadata.json",
        ] {
            fs::write(a.out.join(leaf), serde_json::to_vec_pretty(&receipt)?)?;
        }
        let cp = ContinuationParent::from_checkpoint(&a.out.join("checkpoint-0001"))?;
        // Final pool reduction matches the exact patch caches. Recompute each
        // finite full trace once; only one compact pool vector exists per row.
        let mut final_pools = Vec::with_capacity(UNION);
        for (i, cache) in caches.iter().enumerate() {
            let mut trace = reducer.reduce_trace(
                cache.generate_scores(),
                cache.copy_token_ids(),
                cache.copy_scores(),
            )?;
            replay_require(
                trace.summary.chosen_token_id == cache.summary().chosen_token_id
                    && trace.summary.total_weight_q31 == cache.summary().total_weight_q31
                    && trace
                        .token_masses
                        .iter()
                        .all(|m| cache.token_masses()[m.token_id as usize] == m.weight_q31),
                "Generate final authoritative mass parity differs",
            )?;
            trace.actions.clear();
            trace.actions.shrink_to_fit();
            final_pools.push(shared::Pool {
                donor: donors[i],
                post: posts[i].clone(),
                generate: cache.generate_scores().to_vec(),
                copy: cache.copy_scores().to_vec(),
                base_copy: frames[i].base_copy.clone(),
                trace,
            });
        }
        shared::reload_guard_candidate(a, &cp, &rebound, &frames, &final_pools)?;
        Ok(receipt)
    });
    replay_require(
        np::same_bits(&np::snapshot(&params)?, &saved),
        "Generate original parent masters not restored",
    )?;
    let receipt = receipt?;
    Ok(
        json!({"schema":"uor-r4.generate-episode-learning-report/1","status":"COMPLETED","mode":"generate_episode_learning",
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"policy":policy(),"selected_model":false,
        "finite_episode_positive":gate["passed"],"useful_candidate":false,"final_gate":gate,
        "baseline_objective":initial,"candidate_objective":value,"candidate_receipt":receipt,
        "all_original380_preserved":all_guards,"weighted_roles":32,"unique_objective_frames":31,
        "candidate_native_steps":UNION,"prefix_backward_calls":0,"generate_backward_calls":31,
        "new_generate_gradients":1,"optimizer_updates":0,"parent_master_bits_restored":true,
        "autoregressive_rollout":"NOT_RUN_SEPARATE_ARTIFACT_CHECK","actual9":"NOT_RUN","full512":"NOT_RUN",
        "original_episode_authority":c.retained_episode_root,"episode_request":c.original_inputs.episode}),
    )
}

fn sidecar_identity_matches(
    original: &[u8],
    candidate: &[u8],
    native_metadata: bool,
) -> Result<bool> {
    if native_metadata {
        let old: Value = serde_json::from_slice(original)?;
        let new: Value = serde_json::from_slice(candidate)?;
        Ok(old == new)
    } else {
        Ok(original == candidate)
    }
}
fn restore_original_sidecar_file(
    original: &Path,
    candidate: &Path,
    native_metadata: bool,
) -> Result<Value> {
    let old = fs::read(original)?;
    let before = if candidate.exists() {
        Some(fs::read(candidate)?)
    } else {
        None
    };
    if let Some(bytes) = &before {
        replay_require(
            sidecar_identity_matches(&old, bytes, native_metadata)?,
            "Generate frozen sidecar numerical/native metadata differs",
        )?;
    }
    fs::write(candidate, &old)?;
    replay_require(
        fs::read(candidate)? == old,
        "Generate original sidecar byte restoration differs",
    )?;
    Ok(
        json!({"original_file":original,"candidate_file":candidate,"before_sha256":before.as_ref().map(|b|sha256_bytes(b)),
        "after_sha256":sha256_bytes(&old),"comparison":if native_metadata{"FULL_PARSED_JSON_EQUALITY_THEN_ORIGINAL_BYTES"}else{"EXACT_BYTES"},
        "missing_original_frozen_file_copied":before.is_none()}),
    )
}
/// Completion-only checkpoint copy. The historical donor-master helper is flat.
fn copy_completion_checkpoint(original: &Path, candidate: &Path) -> Result<()> {
    replay_require(
        fs::symlink_metadata(original)?.file_type().is_dir(),
        "completion checkpoint source is not a regular directory",
    )?;
    fs::create_dir(candidate)?;
    let mut entries = fs::read_dir(original)?.collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let kind = entry.file_type()?;
        let destination = candidate.join(entry.file_name());
        if kind.is_dir() {
            copy_completion_checkpoint(&entry.path(), &destination)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), destination)?;
        } else {
            return Err(bad(
                "completion checkpoint contains symlink or unsupported file type",
            ));
        }
    }
    Ok(())
}

fn exact_frozen_directory(original: &Path, candidate: &Path) -> Result<()> {
    let mut old = fs::read_dir(original)?
        .map(|e| e.map(|e| e.file_name()))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let mut new = fs::read_dir(candidate)?
        .map(|e| e.map(|e| e.file_name()))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    old.sort();
    new.sort();
    replay_require(old == new, "completion frozen directory file set differs")?;
    for leaf in old {
        let a = original.join(&leaf);
        let b = candidate.join(&leaf);
        if fs::metadata(&a)?.is_dir() {
            replay_require(
                fs::metadata(&b)?.is_dir(),
                "completion frozen directory type differs",
            )?;
            exact_frozen_directory(&a, &b)?;
        } else {
            replay_require(
                fs::read(&a)? == fs::read(&b)?,
                "completion frozen directory bytes differ",
            )?;
        }
    }
    Ok(())
}
fn complete_pool_objective(
    frames: &[shared::Frame],
    pools: &[shared::Pool],
    map: &[usize],
    spec: &shared::ObjectiveSpec,
) -> Result<Value> {
    let mass = |row: usize| -> Result<(u64, u64, u32)> {
        let p = pools
            .get(row)
            .ok_or_else(|| bad("completion objective pool missing"))?;
        let f = frames
            .get(row)
            .ok_or_else(|| bad("completion objective frame missing"))?;
        let m = p
            .trace
            .token_masses
            .iter()
            .find(|m| m.token_id == f.target)
            .ok_or_else(|| bad("completion objective target mass missing"))?
            .weight_q31;
        Ok((
            m,
            p.trace.summary.total_weight_q31,
            p.trace.summary.chosen_token_id,
        ))
    };
    let masses = map
        .iter()
        .map(|&r| mass(r).map(|(m, d, w)| json!([m, d, w])))
        .collect::<Result<Vec<_>>>()?;
    let mut task = 0.;
    let mut reference = 0.;
    let mut refs = 0usize;
    let mut phases = Vec::new();
    for role in spec
        .roles
        .as_ref()
        .ok_or_else(|| bad("completion roles absent"))?
    {
        let row = map
            .iter()
            .copied()
            .find(|&r| frames[r].input == role.input && frames[r].position == role.position)
            .ok_or_else(|| bad("completion role frame missing"))?;
        let f = &frames[row];
        let (m, d, w) = mass(row)?;
        let ce = -(m as f64 / d as f64).ln() * role.weight;
        if role.task {
            task += ce;
            phases.push(json!({"position":f.position,"target":f.target,"chosen":w,"target_mass":m,"total_mass":d}));
        } else {
            reference += ce;
            refs += usize::from(w == f.target);
        }
    }
    Ok(
        json!({"combined":task+reference,"task":task,"reference":reference,"correct_reference_frames":refs,
        "all_phase_winners":phases.iter().all(|p|p["target"]==p["chosen"]),"phases":phases,"objective_masses":masses}),
    )
}
fn reconstruct_completed_unary(original: &[f32], records: &[Value]) -> Result<Vec<f32>> {
    replay_require(
        original.len() == COUNT && records.len() == COUNT,
        "completed unary population differs",
    )?;
    let mut result = original.to_vec();
    let mut seen = BTreeSet::new();
    let mut epoch = 0u64;
    let mut accepted = 0;
    for (order, row) in records.iter().enumerate() {
        let index = shared::idx(&row["index"])?;
        replay_require(
            index < COUNT
                && seen.insert(index)
                && row["order"] == order
                && row["incumbent_epoch"] == epoch
                && row["incumbent_code"] == i64::from(code(original[index])?),
            "completed coordinate identity/epoch differs",
        )?;
        let alts = row["alternatives"]
            .as_array()
            .ok_or_else(|| bad("completed alternatives absent"))?;
        let current = code(original[index])?;
        let codes = alts
            .iter()
            .map(|a| {
                a["code"]
                    .as_i64()
                    .ok_or_else(|| bad("completed code absent"))
            })
            .collect::<Result<Vec<_>>>()?;
        replay_require(
            codes
                == (-7i64..=7)
                    .filter(|q| *q != i64::from(current))
                    .collect::<Vec<_>>(),
            "completed all14 population differs",
        )?;
        if row["selected"]["status"] == "committed" {
            let q = row["selected"]["code"]
                .as_i64()
                .ok_or_else(|| bad("committed code missing"))?;
            replay_require(
                (-7..=7).contains(&q)
                    && q != i64::from(current)
                    && alts.iter().any(|a| a["code"] == q && a["feasible"] == true),
                "committed selection not a recorded feasible alternative",
            )?;
            result[index] = q as f32 * 0.25;
            epoch += 1;
            accepted += 1;
        } else {
            replay_require(
                row["selected"]["status"] == "unchanged",
                "completed selected status differs",
            )?;
        }
        replay_require(
            row["epoch_after"] == epoch,
            "completed postcommit epoch differs",
        )?;
    }
    replay_require(accepted == 320, "completed recorded accepted count differs")?;
    Ok(result)
}
pub(super) fn run_completion(a: &Args, start: Instant) -> Result<Value> {
    validate_completion_settings(a)?;
    let c = a
        .generate_episode_completion
        .as_ref()
        .ok_or_else(|| bad("completion configuration missing"))?;
    write(
        a,
        "completion-execution-identity.json",
        &json!({"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),
        "argv":std::env::args().collect::<Vec<_>>(),"own_attempt":read(&a.out.join("attempt.json"))?,
        "own_external_config_binding":read(&a.out.join("external-config-binding.json"))?,
        "execution_lane":"CPU host saved arithmetic and native391 only","new_backward_calls":0,
        "new_proposals":0,"new_rankings":0,"new_optimizer_updates":0}),
    )?;
    // This authority is intentionally a sealed FAILED export attempt. The
    // historical completed-authority loader remains strict for its callers.
    report_output::verify(&c.retained_failed_root)?;
    replay_require(
        sha256_file(&c.retained_failed_root.join("report.json"))? == c.expected_report_sha256
            && sha256_file(&c.retained_failed_root.join("manifest.json"))?
                == c.expected_manifest_sha256,
        "completion failed learning report/seal identity differs",
    )?;
    let failed = read(&c.retained_failed_root.join("report.json"))?;
    replay_require(
        failed["status"] == "FAILED"
            && failed["error"] == "Generate frozen sidecar numerical/native metadata differs",
        "completion does not match recorded export-only failure",
    )?;
    let old_config_file = c.retained_failed_root.join("config.json");
    replay_require(
        sha256_file(&old_config_file)? == LEARNING_CONFIG,
        "completed learning raw configuration differs",
    )?;
    let old_config = read(&old_config_file)?;
    let learning: Config = serde_json::from_value(old_config["generate_episode_learning"].clone())?;
    replay_require(
        inherited_inputs_match(
            &serde_json::to_value(&learning.original_inputs)?,
            &c.original_inputs,
        )? && fs::canonicalize(&a.checkpoint)?
            == fs::canonicalize(Path::new(
                old_config["checkpoint"]
                    .as_str()
                    .ok_or_else(|| bad("original checkpoint missing"))?,
            ))?,
        "completion original selected inputs/checkpoint differ",
    )?;
    let attempt = read(&c.retained_failed_root.join("attempt.json"))?;
    report_output::verify(&c.retained_learning_observation_root)?;
    replay_require(
        sha256_file(&c.retained_learning_observation_root.join("manifest.json"))?
            == c.expected_learning_observation_manifest_sha256,
        "learning sealed observation identity differs",
    )?;
    let execution = read(&c.retained_learning_observation_root.join("execution.json"))?;
    let launch = read(&c.retained_learning_observation_root.join("launch.json"))?;
    let runtime_identity_file = c.retained_learning_runtime_root.join("identity.json");
    replay_require(
        sha256_file(&runtime_identity_file)? == c.expected_learning_runtime_identity_sha256,
        "learning source runtime identity receipt differs",
    )?;
    let runtime = read(&runtime_identity_file)?;
    let binary = c
        .retained_learning_runtime_root
        .join("geometric-frozen-map-fit");
    replay_require(
        sha256_file(&binary)? == LEARNING_BINARY
            && runtime["source_commit"] == LEARNING_SOURCE
            && runtime["training_binary_sha256"] == LEARNING_BINARY
            && launch["argv"] == attempt["argv"]
            && launch["pid"] == attempt["pid"]
            && launch["started_utc"] == execution["started_utc"]
            && execution["exit_code"] == 1
            && execution["binary_sha256"] == LEARNING_BINARY
            && execution["config_sha256"] == LEARNING_CONFIG
            && launch["binary_sha256"] == LEARNING_BINARY
            && launch["config_sha256"] == LEARNING_CONFIG,
        "completed learning actual launch/source/binary/config provenance differs",
    )?;
    write(
        a,
        "completion-inherited-launch-authority.json",
        &json!({"learning_attempt":attempt,
        "recorded_launch":launch,"recorded_execution":execution,"runtime_identity":runtime,
        "runtime_identity_sha256":c.expected_learning_runtime_identity_sha256,
        "observation_root":c.retained_learning_observation_root,
        "observation_manifest_sha256":c.expected_learning_observation_manifest_sha256,
        "immutable_binary_copy":binary,"immutable_binary_sha256":LEARNING_BINARY,
        "relocation_scope":"original actual argv path retained; supplied immutable runtime bytes authenticated without relying on mutable build target"}),
    )?;
    replay_require(
        sha256_file(&c.retained_failed_root.join("generate-construction.json"))?
            == COMPLETED_CONSTRUCTOR,
        "completed constructor receipt differs",
    )?;
    let gradient = read(
        &c.retained_failed_root
            .join("generate-gradient-receipt.json"),
    )?;
    replay_require(
        gradient["coefficient_backward_calls"] == 31
            && gradient["weighted_roles"] == 32
            && gradient["active_parameter_names"] == json!([NAME]),
        "completed gradient scope differs",
    )?;
    let terms = gradient["perterm"]
        .as_array()
        .ok_or_else(|| bad("completed gradients missing"))?;
    replay_require(terms.len() == 31, "completed raw gradient count differs")?;
    for row in terms {
        let leaf = row["file"]
            .as_str()
            .ok_or_else(|| bad("completed raw gradient filename missing"))?;
        replay_require(
            Path::new(leaf).components().count() == 1
                && row["status"] == "PRESENT"
                && row["missing_gradient_filled_zero"] == false
                && regular_file_bytes(&c.retained_failed_root.join(leaf))? == 3840
                && sha256_file(&c.retained_failed_root.join(leaf))?
                    == row["sha256"]
                        .as_str()
                        .ok_or_else(|| bad("completed raw gradient SHA missing"))?,
            "completed raw gradient identity differs",
        )?;
    }
    replay_require(
        regular_file_bytes(&c.retained_failed_root.join("generate-gradient.f32le"))? == 3840
            && sha256_file(&c.retained_failed_root.join("generate-gradient.f32le"))?
                == gradient["aggregate"]["sha256"]
                    .as_str()
                    .ok_or_else(|| bad("completed aggregate SHA absent"))?,
        "completed aggregate identity differs",
    )?;
    let original = ContinuationParent::from_checkpoint(&a.checkpoint)?;
    replay_require(
        original.binding.metadata_sha256 == shared::SOURCE
            && sha256_bytes(&original.generate) == shared::G_SHA
            && sha256_file(&a.checkpoint.join("continuation-field.bin"))? == shared::U_SHA,
        "completion original Source/Generate/U differs",
    )?;
    let original_g =
        NativeGeometricGenerate::from_bytes(&original.generate, original.integer.binding())?;
    let original_weights = GenerateLearningWeights::from_native(
        original.integer.binding().clone(),
        &original_g,
        &Device::Cpu,
    )?;
    restore(
        &a.checkpoint.join("generate-source"),
        &read(&a.checkpoint.join("generate-source/metadata.json"))?["parameters"],
        &original_weights.parameters(),
        &Device::Cpu,
    )?;
    let original_bits = np::snapshot(&original_weights.parameters())?;
    let construction = read(&c.retained_failed_root.join("generate-construction.json"))?;
    let summary = construction["summary"].clone();
    replay_require(
        summary["coordinates"] == 960
            && summary["alternatives"] == 13440
            && summary["accepted_coordinates"] == 320
            && summary["revisited_coordinates"] == 0,
        "completed constructor scope differs",
    )?;
    let expected = reconstruct_completed_unary(
        original_bits
            .get(NAME)
            .ok_or_else(|| bad("original unary missing"))?,
        construction["coordinate_records"]
            .as_array()
            .ok_or_else(|| bad("completed records missing"))?,
    )?;
    drop(construction);
    drop(original_weights);
    replay_require(
        sha256_bytes(
            &expected
                .iter()
                .flat_map(|x| x.to_le_bytes())
                .collect::<Vec<_>>(),
        ) == COMPLETED_UNARY,
        "completed selected unary reconstruction differs",
    )?;
    let source_cp = c.retained_failed_root.join("checkpoint-0001");
    replay_require(
        sha256_file(&source_cp.join("generate-source/generate.unary.f32le"))? == COMPLETED_UNARY,
        "partial checkpoint unary differs from completed constructor",
    )?;
    let mut copies = Vec::new();
    for entry in fs::read_dir(&c.retained_failed_root)? {
        let entry = entry?;
        let leaf = entry.file_name();
        if entry.file_type()?.is_file()
            && ![
                "attempt.json",
                "config.json",
                "report.json",
                "manifest.json",
                "external-config-binding.json",
            ]
            .iter()
            .any(|x| leaf == std::ffi::OsStr::new(x))
        {
            fs::copy(entry.path(), a.out.join(&leaf))?;
            copies.push(json!({"file":leaf.to_string_lossy(),"sha256":sha256_file(&entry.path())?,"bytes":regular_file_bytes(&entry.path())?}));
        }
    }
    fs::copy(&old_config_file, a.out.join("learning-config.json"))?;
    fs::copy(
        c.retained_failed_root.join("external-config-binding.json"),
        a.out.join("learning-external-config-binding.json"),
    )?;
    copy_completion_checkpoint(&source_cp, &a.out.join("checkpoint-0001"))?;
    fs::copy(
        source_cp.join("receipt.json"),
        a.out.join("learning-partial-checkpoint-receipt.json"),
    )?;
    let mut repair = Vec::new();
    for family in ["cue", "prefix"] {
        let dest = a.out.join("checkpoint-0001").join(family);
        for entry in fs::read_dir(a.checkpoint.join(family))? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let leaf = entry.file_name();
                repair.push(restore_original_sidecar_file(
                    &entry.path(),
                    &dest.join(&leaf),
                    leaf.to_str() == Some("native-metadata.json"),
                )?);
            }
        }
    }
    for family in ["cue-source", "prefix-source"] {
        if a.checkpoint.join(family).is_dir() {
            let dest = a.out.join("checkpoint-0001").join(family);
            if !dest.exists() {
                copy_completion_checkpoint(&a.checkpoint.join(family), &dest)?;
            } else {
                for entry in fs::read_dir(a.checkpoint.join(family))? {
                    let entry = entry?;
                    replay_require(
                        entry.file_type()?.is_file()
                            && fs::read(entry.path())? == fs::read(dest.join(entry.file_name()))?,
                        "completion frozen sidecar source payload differs",
                    )?;
                }
            }
        }
    }
    write(
        a,
        "completion-sidecar-byte-restoration.json",
        &json!({"repairs":repair,"scope":"only native-metadata.json full semantic equality permits changed bytes; all binary/f32 exact; original frozen missing files copied"}),
    )?;
    for family in ["cue", "prefix", "cue-source", "prefix-source"] {
        let old = a.checkpoint.join(family);
        let new = a.out.join("checkpoint-0001").join(family);
        if old.exists() {
            exact_frozen_directory(&old, &new)?;
        } else {
            replay_require(
                !new.exists(),
                "completion added an absent frozen sidecar family",
            )?;
        }
    }
    let cp = ContinuationParent::from_checkpoint(&a.out.join("checkpoint-0001"))?;
    replay_require(
        cp.binding == original.binding
            && cp.bridge == original.bridge
            && cp.cue == original.cue
            && cp.joint == original.joint
            && cp.prefix == original.prefix,
        "completion frozen native families differ",
    )?;
    let ng = NativeGeometricGenerate::from_bytes(&cp.generate, cp.integer.binding())?;
    let weights =
        GenerateLearningWeights::from_native(cp.integer.binding().clone(), &ng, &Device::Cpu)?;
    restore(
        &a.out.join("checkpoint-0001/generate-source"),
        &read(&a.out.join("checkpoint-0001/generate-source/metadata.json"))?["parameters"],
        &weights.parameters(),
        &Device::Cpu,
    )?;
    let actual = np::snapshot(&weights.parameters())?;
    let mut expected_all = original_bits;
    expected_all.insert(NAME.into(), expected);
    replay_require(
        np::same_bits(&actual, &expected_all)
            && weights.export_native()?.to_bytes()? == cp.generate,
        "completion checkpoint Generate all masters/export differs",
    )?;
    drop(weights);
    drop(expected_all);
    drop(actual);
    for family in ["source", "generate-source"] {
        let old = a.checkpoint.join(family);
        for entry in fs::read_dir(&old)? {
            let entry = entry?;
            if family == "generate-source"
                && entry.file_type()?.is_file()
                && entry.file_name() != "generate.unary.f32le"
                && entry.file_name() != "metadata.json"
            {
                replay_require(
                    fs::read(entry.path())?
                        == fs::read(
                            a.out
                                .join("checkpoint-0001")
                                .join(family)
                                .join(entry.file_name()),
                        )?,
                    "completion frozen Generate raw master bytes differ",
                )?;
            }
        }
    }
    for leaf in [
        "source/consumer/context.safetensors",
        "source/consumer/potential.safetensors",
        "continuation-source/continuation.unary.f32le",
    ] {
        if a.checkpoint.join(leaf).is_file() {
            replay_require(
                sha256_file(&a.checkpoint.join(leaf))?
                    == sha256_file(&a.out.join("checkpoint-0001").join(leaf))?,
                "completion frozen Source/U masters differ",
            )?;
        }
    }
    exact_frozen_directory(
        &a.checkpoint.join("source"),
        &a.out.join("checkpoint-0001/source"),
    )?;
    let u = NativeContinuationField::from_bytes(
        &fs::read(a.out.join("checkpoint-0001/continuation-field.bin"))?,
        &cp.binding,
        &ng,
    )?;
    let old_u = NativeContinuationField::from_bytes(
        &fs::read(a.checkpoint.join("continuation-field.bin"))?,
        &original.binding,
        &original_g,
    )?;
    replay_require(
        u.packed_unary() == old_u.packed_unary(),
        "completion rebound U numerical payload differs",
    )?;
    write(
        a,
        "completion-learning-provenance.json",
        &json!({"failed_root":c.retained_failed_root,"failed_report_sha256":FAILED_REPORT,
        "failed_manifest_sha256":FAILED_SEAL,"learning_config_sha256":LEARNING_CONFIG,"learning_source_commit":LEARNING_SOURCE,
        "learning_binary_sha256":LEARNING_BINARY,"constructor_sha256":COMPLETED_CONSTRUCTOR,"selected_unary_sha256":COMPLETED_UNARY,
        "inherited_backward_calls":31,"inherited_alternatives":13440,"copied_scientific_files":copies,
        "completion_source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"completion_backward_calls":0,"completion_proposals":0,"source_epoch_alias":false}),
    )?;
    let (mut objective_frames, spec) =
        prefix::prepare_generate_objective(a, &c.original_inputs, &original)?;
    let mut reducer =
        NativeVocabularyActions::new(original.integer.binding().clone(), &original.exp)?;
    let mut objective_pools = objective_frames
        .iter()
        .map(|f| saved_pool(f, &mut reducer))
        .collect::<Result<Vec<_>>>()?;
    slim(&mut objective_frames)?;
    for p in &mut objective_pools {
        compact_pool(p);
    }
    let (mut frames, mut pools, _authority) = prefix::prepare_generate_guards(
        a,
        &c.original_inputs,
        &original,
        &objective_frames,
        &spec,
    )?;
    // Normalization helpers may emit receipts. Inherited scientific evidence
    // must remain byte-identical after those helpers, not merely at initial copy.
    for copied in &copies {
        let leaf = copied["file"]
            .as_str()
            .ok_or_else(|| bad("inherited scientific filename missing"))?;
        let path = a.out.join(leaf);
        replay_require(
            regular_file_bytes(&path)?
                == copied["bytes"]
                    .as_u64()
                    .ok_or_else(|| bad("inherited scientific length missing"))?
                && sha256_file(&path)?
                    == copied["sha256"]
                        .as_str()
                        .ok_or_else(|| bad("inherited scientific SHA missing"))?,
            "completion normalization changed inherited scientific evidence",
        )?;
    }
    write(
        a,
        "completion-inherited-file-verification.json",
        &json!({"stage":"after objective/guard normalization before candidate scoring",
            "all_copied_scientific_files_byte_identical":true,"files":copies}),
    )?;
    let mut map = Vec::new();
    for (frame, pool) in objective_frames.into_iter().zip(objective_pools) {
        if let Some(row) = frames
            .iter()
            .position(|f| f.input == frame.input && f.position == frame.position)
        {
            replay_require(
                frames[row].id == frame.id
                    && frames[row].prefix == frame.prefix
                    && pools[row].generate == pool.generate
                    && pools[row].copy == pool.copy
                    && pools[row].post == pool.post,
                "completion original overlapping frame differs",
            )?;
            frames[row] = frame;
            map.push(row);
        } else {
            map.push(frames.len());
            frames.push(frame);
            pools.push(pool);
        }
    }
    replay_require(
        frames.len() == UNION && map.len() == 31,
        "completion original union differs",
    )?;
    let population = read(&a.out.join("generate-population.json"))?;
    let rows = population["rows"]
        .as_array()
        .ok_or_else(|| bad("completion population rows missing"))?;
    replay_require(
        rows.len() == UNION && population["objective_row_map"] == json!(map),
        "completion saved row map differs",
    )?;
    for (i, f) in frames.iter_mut().enumerate() {
        replay_require(
            rows[i]["input_index"] == f.input
                && rows[i]["position"] == f.position
                && rows[i]["id"] == f.id
                && rows[i]["actual_prefix_ids"] == json!(f.prefix),
            "completion frame identity differs",
        )?;
        f.native = json!({"continuation":f.native["continuation"].clone()});
        f.prefix_trace = None;
    }
    // Saved objective fulltrace parity and protected helper authentication are
    // complete before redundant action records are discarded. Raw score arrays,
    // token masses, summary, bank/source/state witnesses remain retained.
    for pool in &mut pools {
        compact_pool(pool);
    }
    let pool_payload = pools
        .iter()
        .map(|p| {
            (p.generate.capacity() + p.copy.capacity() + p.base_copy.capacity())
                * std::mem::size_of::<i64>()
                + p.trace.token_masses.capacity()
                    * std::mem::size_of::<
                        uor_r4_integer::geometric_vocabulary_actions::VocabularyTokenMass,
                    >()
        })
        .sum::<usize>() as u64;
    let frame_json = frames
        .iter()
        .map(|f| serde_json::to_vec(&f.native).map(|b| b.len() as u64))
        .collect::<std::result::Result<Vec<_>, _>>()?
        .into_iter()
        .sum::<u64>();
    let prior = read(&a.out.join("generate-resource-projection.json"))?;
    let typed_frames = prior["prior_projection"]["typed_guard_frame_bound"]
        .as_u64()
        .ok_or_else(|| bad("completion typed frame projection absent"))?;
    let numeric = pool_payload
        .checked_add(4 * frame_json)
        .and_then(|x| x.checked_add(typed_frames))
        .and_then(|x| x.checked_add(32 * 1024 * 1024))
        .ok_or_else(|| bad("completion numeric projection overflow"))?;
    let report_before = size(&a.out)?;
    let report_projection = report_before + 384 * 1024 * 1024;
    write(
        a,
        "completion-resource-projection.json",
        &json!({"prior_learning_projection":prior,
        "pool_capacity_payload_bytes":pool_payload,"retained_continuation_json_bytes":frame_json,
        "continuation_json_inflation_factor":4,"typed_guard_frame_bound":typed_frames,
        "numerical_temporary_and_container_reserve":32*1024*1024u64,
        "numeric_upper_bound":numeric,"numeric_cap":512*1024*1024u64,"model_compiled_tables_and_CPU_parameters":"separate process RAM; fresh gradient/CSR/pending caches absent",
        "process_ram_cap":4*1024*1024*1024u64,"cache_cap":CACHE_CAP,
        "normalization_donor_cache_cap":128*1024*1024u64,"normalization_cache_dropped_before_candidate391":true,
        "backward_calls":0,"coordinate_search":0,"csr_cache":"NOT_ALLOCATED","generate_patch_caches":"NOT_ALLOCATED",
        "native_reload_frames":391,"finite_candidate_full_pool_reductions":391,
        "saved_objective_full_pool_reductions":31,"protected_saved_normalization_internal_reductions":"UNAVAILABLE_NOT_INSTRUMENTED_IN_REUSED_HELPER",
        "report_bytes_before_reload":report_before,"report_projection_bytes":report_projection,
        "report_cap_bytes":a.maximum_report_bytes,"reserved_candidate391_serialization_bytes":384*1024*1024u64,
        "scope":"CPU saved frame normalization plus one candidate finite score/reduction per391, then unfinished391 native steps; no learning repeat"}),
    )?;
    replay_require(
        numeric <= 512 * 1024 * 1024 && report_projection + 1024 * 1024 < a.maximum_report_bytes,
        "completion full391 numerical/report projection exceeded",
    )?;
    for (i, pool) in pools.iter_mut().enumerate() {
        let mut base = vec![0i64; VOCAB];
        ng.score_into(&pool.post, &mut base, &mut Default::default())?;
        let generate = base
            .iter()
            .zip(&frames[i].u)
            .map(|(g, u)| {
                g.checked_add(*u)
                    .ok_or_else(|| bad("completion Generate/U overflow"))
            })
            .collect::<Result<Vec<_>>>()?;
        let mut trace = reducer.reduce_trace(&generate, &frames[i].ids, &pool.copy)?;
        trace.actions.clear();
        trace.actions.shrink_to_fit();
        pool.generate = generate;
        pool.trace = trace;
    }
    let guarded = read(&a.out.join("final-trajectory-guards.json"))?;
    let saved_guards = guarded["terms"]
        .as_array()
        .ok_or_else(|| bad("completed guard summaries missing"))?;
    replay_require(
        saved_guards.len() == GUARDS,
        "completed guard summaries count differs",
    )?;
    for (i, row) in saved_guards.iter().enumerate() {
        let summary = &pools[i].trace.summary;
        replay_require(
            row["guard_index"] == i
                && row["input_index"] == frames[i].input
                && row["position"] == frames[i].position
                && row["required_original_winner"] == frames[i].target
                && row["pool"]["reference_q24"] == summary.max_score_q24
                && row["pool"]["chosen_token_id"] == summary.chosen_token_id
                && row["pool"]["chosen_weight_q31"] == summary.chosen_weight_q31
                && row["pool"]["total_weight_q31"] == summary.total_weight_q31,
            "completion finite guard summary differs from completed cache",
        )?;
    }
    let value = complete_pool_objective(&frames, &pools, &map, &spec)?;
    replay_require(
        value == summary["final"] && value == read(&a.out.join("final-objective.json"))?,
        "completion candidate complete objective/masses differ from finished learning",
    )?;
    let all_guards =
        (0..GUARDS).all(|i| pools[i].trace.summary.chosen_token_id == frames[i].target);
    let initial = read(&a.out.join("initial-original-objective.json"))?;
    let gate = final_gate(&initial, &value, all_guards)?;
    replay_require(
        all_guards && gate["passed"] == false,
        "completion recorded negative gate/guards differ",
    )?;
    let mut receipt = read(&a.out.join("checkpoint-0001/receipt.json"))?;
    receipt["mode"] = json!("generate_episode_completion");
    receipt["policy"] = policy();
    receipt["active_parameter_names"] = json!([]);
    receipt["completion_backward_calls"] = json!(0);
    receipt["completion_proposals"] = json!(0);
    receipt["inherited_learning_backward_calls"] = json!(31);
    receipt["inherited_learning_alternatives"] = json!(13440);
    receipt["learning_source_commit"] = json!(LEARNING_SOURCE);
    receipt["completion_source_commit"] = json!(option_env!("UOR_BUILD_SOURCE_COMMIT"));
    receipt["candidate_native_steps"] = json!(0);
    receipt["native_independently_reloaded"] = json!(false);
    receipt["fresh_adam"] = json!(false);
    receipt["new_gradients"] = json!(0);
    receipt["coefficient_backward_calls"] = json!(0);
    receipt["optimizer_updates"] = json!(0);
    receipt["credit_scope"] = json!(
        "completion only; no gradients/ranking/search; recorded original unary learning preserved"
    );
    for leaf in [
        "checkpoint-0001/receipt.json",
        "checkpoint-0001/continuation-source/metadata.json",
    ] {
        fs::write(a.out.join(leaf), serde_json::to_vec_pretty(&receipt)?)?;
    }
    let cp = ContinuationParent::from_checkpoint(&a.out.join("checkpoint-0001"))?;
    shared::reload_guard_candidate(a, &cp, &u, &frames, &pools)?;
    receipt["candidate_native_steps"] = json!(UNION);
    receipt["native_independently_reloaded"] = json!(true);
    for leaf in [
        "checkpoint-0001/receipt.json",
        "checkpoint-0001/continuation-source/metadata.json",
    ] {
        fs::write(a.out.join(leaf), serde_json::to_vec_pretty(&receipt)?)?;
    }
    Ok(
        json!({"schema":"uor-r4.generate-episode-completion/1","status":"COMPLETED","mode":"generate_episode_completion",
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"learning_source_commit":LEARNING_SOURCE,
        "learning_binary_sha256":LEARNING_BINARY,"learning_config_sha256":LEARNING_CONFIG,
        "learning_failed_report_sha256":FAILED_REPORT,"learning_failed_manifest_sha256":FAILED_SEAL,
        "completion_backward_calls":0,"completion_proposals":0,"inherited_backward_calls":31,"inherited_alternatives":13440,
        "finite_candidate_full_pool_reductions":391,"candidate_native_steps":391,
        "baseline_objective":initial,"candidate_objective":value,"final_gate":gate,"candidate_receipt":receipt,
        "all_original380_preserved":all_guards,"weighted_roles":32,"unique_objective_frames":31,
        "finite_episode_positive":false,"selected_model":false,"useful_candidate":false,
        "actual9":"NOT_RUN_CONSTRUCTION_NEGATIVE","full512":"NOT_RUN","parent_master_bits_unchanged":true,
        "parent_master_restoration":"NOT_APPLICABLE_NO_PARENT_MASTER_MUTATION",
        "elapsed_seconds":start.elapsed().as_secs_f64(),"scope":"completed export/reload only; inherited fresh learning was not repeated"}),
    )
}

/// Admission for the cheap artifact-only evaluator. Independent arithmetic audit
/// remains a final delivery requirement, not a prerequisite to this cheap check.
pub(super) fn authenticate_positive_artifact(
    a: &Args,
    c: &prefix::ArtifactConfig,
    report: &Value,
) -> Result<()> {
    let pin = c
        .generate_episode_candidate
        .as_ref()
        .ok_or_else(|| bad("Generate artifact authority missing"))?;
    replay_require(
        report["status"] == "COMPLETED"
            && report["mode"] == "generate_episode_learning"
            && report["selected_model"] == false
            && report["finite_episode_positive"] == true
            && report["final_gate"]["passed"] == true
            && report["all_original380_preserved"] == true
            && report["weighted_roles"] == 32
            && report["unique_objective_frames"] == 31
            && report["generate_backward_calls"] == 31
            && report["prefix_backward_calls"] == 0
            && report["candidate_native_steps"] == 391
            && report["optimizer_updates"] == 0,
        "Generate artifact lacks positive complete native construction",
    )?;
    let root = &c.retained_candidate_root;
    let gradients = read(&root.join("generate-gradient-receipt.json"))?;
    replay_require(
        gradients["coefficient_backward_calls"] == 31
            && gradients["weighted_roles"] == 32
            && gradients["active_parameter_names"] == json!([NAME]),
        "Generate gradient inventory scope differs",
    )?;
    let rows = gradients["perterm"]
        .as_array()
        .ok_or_else(|| bad("Generate perterm gradients absent"))?;
    replay_require(rows.len() == 31, "Generate perterm gradient count differs")?;
    for row in rows {
        let leaf = row["file"]
            .as_str()
            .ok_or_else(|| bad("Generate raw gradient filename absent"))?;
        replay_require(
            Path::new(leaf).components().count() == 1
                && fs::metadata(root.join(leaf))?.len() == 3840
                && sha256_file(&root.join(leaf))?
                    == row["sha256"]
                        .as_str()
                        .ok_or_else(|| bad("Generate raw gradient hash absent"))?,
            "Generate raw gradient receipt differs",
        )?;
    }
    replay_require(
        fs::metadata(root.join("generate-gradient.f32le"))?.len() == 3840
            && sha256_file(&root.join("generate-gradient.f32le"))?
                == gradients["aggregate"]["sha256"]
                    .as_str()
                    .ok_or_else(|| bad("Generate aggregate gradient hash absent"))?,
        "Generate aggregate gradient differs",
    )?;
    let construction = read(&root.join("generate-construction.json"))?;
    let coords = construction["coordinate_records"]
        .as_array()
        .ok_or_else(|| bad("Generate coordinate records absent"))?;
    replay_require(
        coords.len() == COUNT
            && construction["summary"]["coordinates"] == 960
            && construction["summary"]["alternatives"] == 13440
            && construction["summary"]["revisited_coordinates"] == 0,
        "Generate exhaustive once-only policy receipt differs",
    )?;
    let mut indices = BTreeSet::new();
    for row in coords {
        let index = shared::idx(&row["index"])?;
        let incumbent = row["incumbent_code"]
            .as_i64()
            .ok_or_else(|| bad("Generate incumbent code missing"))?;
        let alternatives = row["alternatives"]
            .as_array()
            .ok_or_else(|| bad("Generate alternatives missing"))?;
        let codes = alternatives
            .iter()
            .map(|x| {
                x["code"]
                    .as_i64()
                    .ok_or_else(|| bad("Generate alternative code missing"))
            })
            .collect::<Result<BTreeSet<_>>>()?;
        replay_require(
            index < COUNT
                && indices.insert(index)
                && alternatives.len() == 14
                && codes
                    == (-7i64..=7)
                        .filter(|q| *q != incumbent)
                        .collect::<BTreeSet<_>>(),
            "Generate exhaustive code population differs",
        )?;
    }
    drop(construction);
    let guards = read(&root.join("final-trajectory-guards.json"))?;
    let terms = guards["terms"]
        .as_array()
        .ok_or_else(|| bad("Generate guard receipt missing"))?;
    replay_require(
        guards["all_original_winners"] == true
            && terms.len() == GUARDS
            && terms
                .iter()
                .all(|t| t["chosen"] == t["required_original_winner"]),
        "Generate complete original guard retention differs",
    )?;
    replay_require(
        report["candidate_objective"]["all_phase_winners"] == true
            && report["candidate_objective"]["correct_reference_frames"] == 17,
        "Generate final wholeepisode/reference gate differs",
    )?;

    let parent =
        ContinuationParent::from_checkpoint(&c.retained_intermediate_root.join("checkpoint-0001"))?;
    let candidate = ContinuationParent::from_checkpoint(&a.checkpoint)?;
    replay_require(
        parent.binding == candidate.binding
            && parent.bridge == candidate.bridge
            && parent.cue == candidate.cue
            && parent.joint == candidate.joint
            && parent.prefix == candidate.prefix
            && sha256_bytes(&candidate.generate) == pin.expected_generate_sha256
            && sha256_file(&a.checkpoint.join("continuation-field.bin"))?
                == pin.expected_continuation_sha256,
        "Generate candidate frozen native families differ",
    )?;
    let original_native =
        NativeGeometricGenerate::from_bytes(&parent.generate, parent.integer.binding())?;
    let current_native =
        NativeGeometricGenerate::from_bytes(&candidate.generate, candidate.integer.binding())?;
    for leaf in [
        "cue/cue-source-f32.bin",
        "prefix/prefix-source-f32.bin",
        "source/consumer/context.safetensors",
        "source/consumer/potential.safetensors",
    ] {
        let old = c
            .retained_intermediate_root
            .join("checkpoint-0001")
            .join(leaf);
        let new = a.checkpoint.join(leaf);
        if old.is_file() {
            replay_require(
                new.is_file() && sha256_file(&old)? == sha256_file(&new)?,
                "Generate candidate changed a frozen Source/sidecar floating payload",
            )?;
        }
    }
    let original_u = NativeContinuationField::from_bytes(
        &fs::read(
            c.retained_intermediate_root
                .join("checkpoint-0001/continuation-field.bin"),
        )?,
        &parent.binding,
        &original_native,
    )?;
    let current_u = NativeContinuationField::from_bytes(
        &fs::read(a.checkpoint.join("continuation-field.bin"))?,
        &candidate.binding,
        &current_native,
    )?;
    replay_require(
        original_u.packed_unary() == current_u.packed_unary(),
        "Generate rebound U numerical payload changed",
    )?;
    let dev = Device::Cpu;
    let original_weights = GenerateLearningWeights::from_native(
        parent.integer.binding().clone(),
        &original_native,
        &dev,
    )?;
    restore(
        &c.retained_intermediate_root
            .join("checkpoint-0001/generate-source"),
        &read(
            &c.retained_intermediate_root
                .join("checkpoint-0001/generate-source/metadata.json"),
        )?["parameters"],
        &original_weights.parameters(),
        &dev,
    )?;
    let current_weights = GenerateLearningWeights::from_native(
        candidate.integer.binding().clone(),
        &current_native,
        &dev,
    )?;
    restore(
        &a.checkpoint.join("generate-source"),
        &read(&a.checkpoint.join("generate-source/metadata.json"))?["parameters"],
        &current_weights.parameters(),
        &dev,
    )?;
    replay_require(
        current_weights.export_native()?.to_bytes()? == candidate.generate,
        "Generate candidate actual floating masters do not export to native",
    )?;
    let original = np::snapshot(&original_weights.parameters())?;
    let current = np::snapshot(&current_weights.parameters())?;
    replay_require(
        original
            .iter()
            .filter(|(n, _)| n.as_str() != NAME)
            .all(|(n, v)| {
                current.get(n).is_some_and(|x| {
                    v.iter().zip(x).all(|(a, b)| a.to_bits() == b.to_bits()) && v.len() == x.len()
                })
            }),
        "Generate candidate changed a frozen floating family",
    )?;
    let metadata = read(&a.checkpoint.join("generate-source/metadata.json"))?;
    replay_require(
        metadata["parameters"][NAME]["shape"] == json!([8, 120])
            && metadata["parameters"][NAME]["sha256"] == pin.expected_unary_master_sha256,
        "Generate unary actual master identity differs",
    )?;
    let population = read(&root.join("generate-population.json"))?;
    replay_require(
        population["rows"]
            .as_array()
            .is_some_and(|r| r.len() == 391),
        "Generate reloaded population missing",
    )?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn completion_checkpoint_copy_preserves_nested_files_and_rejects_existing_destination_and_symlinks(
    ) -> Result<()> {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "uor-completion-copy-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&root)?;
        let original = root.join("original");
        fs::create_dir(&original)?;
        fs::create_dir(original.join("source"))?;
        fs::create_dir(original.join("source/native"))?;
        fs::write(original.join("receipt.json"), b"{\"old\":true}")?;
        fs::write(original.join("source/native/packed.bin"), [0u8, 255, 19])?;
        let copied = root.join("copied");
        copy_completion_checkpoint(&original, &copied)?;
        exact_frozen_directory(&original, &copied)?;
        assert!(copy_completion_checkpoint(&original, &copied).is_err());
        assert_eq!(
            fs::read(copied.join("source/native/packed.bin"))?,
            [0u8, 255, 19]
        );
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(
                original.join("receipt.json"),
                original.join("unexpected-link"),
            )?;
            assert!(copy_completion_checkpoint(&original, &root.join("rejected")).is_err());
            std::os::unix::fs::symlink(&original, root.join("source-link"))?;
            assert!(copy_completion_checkpoint(
                &root.join("source-link"),
                &root.join("rejected-source")
            )
            .is_err());
        }
        fs::remove_dir_all(&root)?;
        Ok(())
    }

    #[test]
    fn sidecar_key_order_is_allowed_only_for_full_native_metadata_identity() -> Result<()> {
        let old = br#"{"parent":{"sha":"epoch"},"coefficients":[1,2]}"#;
        let reordered = br#"{"coefficients":[1,2],"parent":{"sha":"epoch"}}"#;
        let changed = br#"{"coefficients":[1,3],"parent":{"sha":"epoch"}}"#;
        let changed_epoch = br#"{"coefficients":[1,2],"parent":{"sha":"other"}}"#;
        assert!(sidecar_identity_matches(old, reordered, true)?);
        assert!(!sidecar_identity_matches(old, reordered, false)?);
        assert!(!sidecar_identity_matches(old, changed, true)?);
        assert!(!sidecar_identity_matches(old, changed_epoch, true)?);
        assert!(sidecar_identity_matches(old, b"not-json", true).is_err());
        Ok(())
    }
    #[test]
    fn completed_unary_reconstruction_preserves_unselected_fractional_bits_and_epochs() -> Result<()>
    {
        let mut masters = vec![0f32; COUNT];
        masters[0] = 0.13;
        masters[COUNT - 1] = -0.0;
        let mut records = Vec::new();
        let mut epoch = 0usize;
        for (i, m) in masters.iter().enumerate() {
            let current = code(*m)?;
            let committed = i < 320;
            let target = if current == 2 { 3 } else { 2 };
            let alts = (-7i64..=7)
                .filter(|q| *q != i64::from(current))
                .map(|q| json!({"code":q,"feasible":true}))
                .collect::<Vec<_>>();
            records.push(json!({"order":i,"index":i,"incumbent_epoch":epoch,
                "incumbent_code":current,"alternatives":alts,
                "selected":{"status":if committed{"committed"}else{"unchanged"},"code":if committed{target}else{current}} ,
                "epoch_after":epoch+usize::from(committed)}));
            epoch += usize::from(committed);
        }
        let final_bits = reconstruct_completed_unary(&masters, &records)?;
        assert_eq!(final_bits[0], 0.5);
        assert_eq!(
            final_bits[COUNT - 1].to_bits(),
            masters[COUNT - 1].to_bits()
        );
        records[COUNT - 1]["incumbent_epoch"] = json!(321);
        assert!(reconstruct_completed_unary(&masters, &records).is_err());
        Ok(())
    }
    #[test]
    fn report_projection_file_length_rejects_directory_and_missing_file() -> Result<()> {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "uor-generate-projection-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&root)?;
        let file = root.join("prefix-construction.json");
        fs::write(&file, b"{\"summary\":{}}")?;
        assert_eq!(regular_file_bytes(&file)?, 14);
        assert!(regular_file_bytes(&root).is_err());
        assert!(regular_file_bytes(&root.join("missing.json")).is_err());
        fs::remove_dir_all(&root)?;
        Ok(())
    }
    #[test]
    fn inherited_inputs_defaults_match_but_nested_authority_change_is_rejected() -> Result<()> {
        let raw = json!({
            "retained_intermediate_root":"/original",
            "retained_probe_root":"/probe",
            "joint":{
                "p5_capture":{"root":"/p5","expected_report_sha256":"report5","expected_manifest_sha256":"seal5"},
                "p6_conditional_capture":{"root":"/p6","expected_report_sha256":"report6","expected_manifest_sha256":"seal6"},
                "retained_supplement_root":"/supplement",
                "expected_supplement_report_sha256":"supplement-report",
                "expected_supplement_manifest_sha256":"supplement-seal"
            }
        });
        let typed: prefix::Config = serde_json::from_value(raw.clone())?;
        let explicit = serde_json::to_value(&typed)?;
        assert_ne!(raw, explicit);
        assert!(inherited_inputs_match(&raw, &typed)?);
        assert!(inherited_inputs_match(&explicit, &typed)?);
        let mut changed = explicit.clone();
        changed["joint"]["p5_capture"]["expected_report_sha256"] = json!("different-report");
        let changed: prefix::Config = serde_json::from_value(changed)?;
        assert!(!inherited_inputs_match(&raw, &changed)?);
        let mut unknown = raw;
        unknown["unrecognized_authority"] = json!(true);
        assert!(inherited_inputs_match(&unknown, &typed).is_err());
        Ok(())
    }
    #[test]
    fn full_legal_order_uses_fractional_delta_and_keeps_zero_coordinates() -> Result<()> {
        let mut m = vec![0.; COUNT];
        let mut g = vec![0.; COUNT];
        m[1] = 0.13;
        g[1] = -1.;
        m[2] = -0.13;
        g[2] = 1.;
        let order = coordinate_order(&m, &g)?;
        assert_eq!(order.len(), COUNT);
        assert_eq!(order[0], 1);
        assert_eq!(order[1], 2);
        assert_eq!(order.iter().copied().collect::<BTreeSet<_>>().len(), COUNT);
        assert_eq!(code(0.13)?, 1);
        assert_eq!(
            (-7i8..=7)
                .filter(|q| *q != code(m[1]).unwrap_or(99))
                .count(),
            14
        );
        assert!(coordinate_order(&m, &vec![f32::NAN; COUNT]).is_err());
        Ok(())
    }
    #[test]
    fn native_ce_tie_and_reference_veto_do_not_select_gradient_direction() -> Result<()> {
        assert!(lower((1., -7), (1., 7)));
        assert!(!lower((1., 7), (1., -7)));
        let current = json!({"combined":2.});
        let next = json!({"combined":1.,"correct_reference_frames":16});
        assert!(!valid_objective(&current, &next)?);
        assert!(valid_objective(
            &current,
            &json!({"combined":1.,"correct_reference_frames":17})
        )?);
        assert!(!improves(2., 2. - 1e-12));
        Ok(())
    }
    #[test]
    fn terminal_winner_and_original_guard_are_independent_final_gates() -> Result<()> {
        let original = json!({"combined":4.,"task":2.});
        let wrong = json!({"combined":3.,"task":1.,"all_phase_winners":false,"correct_reference_frames":17});
        assert_eq!(final_gate(&original, &wrong, true)?["passed"], false);
        let right =
            json!({"combined":3.,"task":1.,"all_phase_winners":true,"correct_reference_frames":17});
        assert_eq!(final_gate(&original, &right, false)?["passed"], false);
        assert_eq!(final_gate(&original, &right, true)?["passed"], true);
        Ok(())
    }
    #[test]
    fn late_native_guard_patch_is_discarded_and_same_epoch_alternatives_remain_valid() -> Result<()>
    {
        use uor_r4_integer::geometric_source_actions::SourceActionBinding;
        const TOK:&[u8]=br#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4,"b":5},"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]}"#;
        let exp = (0..uor_r4_integer::geometric_read::EXP_TABLE_LEN)
            .flat_map(|i| {
                (((-(i as f64) / 256.).exp() * (1u64 << 31) as f64).round() as u32).to_le_bytes()
            })
            .collect::<Vec<_>>();
        let mut reducer = NativeVocabularyActions::new(SourceActionBinding::new(TOK)?, &exp)?;
        let mut caches = (0..GUARDS)
            .map(|_| {
                reducer.prepare_generate_patch_cache(
                    vec![-4 << 24, -4 << 24, -4 << 24, -4 << 24, 0, -1 << 20],
                    vec![4, 4],
                    vec![-3 << 24, -3 << 24],
                )
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let before = caches
            .iter()
            .map(|c| c.generate_scores().to_vec())
            .collect::<Vec<_>>();
        let mut changes = BTreeMap::new();
        for row in 0..GUARDS {
            // The final guard alone changes winner. Generate5 moves from
            // -1 to +2 native units: exp(.125) exceeds 1+2*exp(-3), while
            // the three-unit displacement remains inside legal Q4 bounds.
            let token = if row + 1 == GUARDS { 5 } else { 4 };
            changes.insert(row, vec![(token, 2 << 20)]);
        }
        let mut staged = Patches::new();
        for row in 0..GUARDS {
            stage_row(row, &changes, &caches, &mut staged, &mut reducer)?;
        }
        assert_eq!(staged.len(), GUARDS);
        assert!((0..GUARDS - 1).all(|row| staged[&row].summary().chosen_token_id == 4));
        assert_eq!(staged[&(GUARDS - 1)].summary().chosen_token_id, 5);
        drop(staged);
        assert!(caches
            .iter()
            .zip(&before)
            .all(|(c, b)| c.generate_scores() == b && c.revision() == 0));
        // Another legal alternative at the identical incumbent revision can commit.
        let patch = reducer.evaluate_generate_patch(&caches[0], &[(4, 1 << 20)])?;
        reducer.commit_generate_patch_batch(&mut caches, vec![(0, patch)])?;
        assert_eq!(caches[0].copy_token_ids(), &[4, 4]);
        assert_eq!(caches[0].revision(), 1);
        assert_eq!(caches[GUARDS - 1].revision(), 0);
        Ok(())
    }
}
