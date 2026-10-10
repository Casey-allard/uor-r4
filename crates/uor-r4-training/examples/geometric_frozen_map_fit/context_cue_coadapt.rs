//! One saved-state Cue construction coupled to the recorded Context displacement.
//! Native hard pools and the original-parent gate are authoritative; selector credit is local.
use super::native_proposals as np;
use super::*;
use uor_r4_integer::{
    geometric_cue_carrier::{CarrierState, CueCarrierCosts, CueCarrierTrace, CueSegmentTrace},
    geometric_prefix_transport::{
        PrefixState, PrefixTransportCosts, PrefixTransportTrace, SourcePrefixTrace,
    },
    geometric_source_realizer::ObservedCode,
    geometric_vocabulary_actions::VocabularyActionTrace,
    h4_tables::H4Code,
};
use uor_r4_training::geometric_generate_learning::vocabulary_marginal_loss_with_credit;
use uor_r4_training::geometric_occurrence_consumer::source_realizer::{
    CueAngularWeights, PrefixAngularWeights,
};

pub(super) const P_REPORT: &str =
    "c9b9fe10b6fbb4332cf919a5df7ba31403ad3d51ac6f877d94daeabd99672bee";
pub(super) const P_SEAL: &str = "de90ba0ba2809ed37ca868ef2bcf94b6ceb8b176a60b86ae88801876dafbd0e5";
pub(super) const B_REPORT: &str =
    "1f7a51fe58e56862f6e8cd269225445bf9d42354d3cfe7eaf96a5910707568c9";
pub(super) const B_SEAL: &str = "c43bbbe81eeea18338b2ea541c50f8f01d24dcfbe9a32662b73e2d2ebc03b332";
const F_REPORT: &str = "975cdaf9185f21ceb4c867d4cbb00eafbe6079ad05a36cc481b86621600bffea";
const F_SEAL: &str = "6d51345dbc5c4fc3307a2b79f113470b8f1434f1496a5dc2146c59f305557bfd";
const FINITE_SOURCE: &str = "2a79a8e75a8c9614d6802146af0c5d0b6ceed0e09668151b571ea3beb75efab8";
pub(super) const SOURCE: &str = "9f0b272e7852a47bbbad0f549e8157a44ff3212af86905907501ec11f3e7989b";
pub(super) const G_SHA: &str = "4248245471db609b1fc19482e8f180380b292c5832fc90c81ce69947bc4b7737";
pub(super) const U_SHA: &str = "82ae9daeb402b288e64492d5b299110b36849907019c952609a1cae6612673ee";
pub(super) const COHORT: [usize; 9] = [245, 0, 1, 4, 5, 8, 9, 12, 13];
const NAME: &str = "cue.coefficients";
const CONTEXT: &str = "consumer.context.self_transition";
pub(super) const CACHE_LIMIT: usize = 64 * 1024 * 1024;

#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Config {
    pub retained_intermediate_root: PathBuf,
    pub retained_probe_root: PathBuf,
    pub retained_finite_root: PathBuf,
}
pub(super) fn validate_settings(a: &Args) -> Result<()> {
    if let Some(c) = &a.context_cue_coadapt {
        replay_require(
            a.mode == Mode::JointContinuation
                && a.updates == 1
                && a.loss_scope == LossScope::All
                && a.prefix_fragment_learning.is_none()
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
            "Context/Cue requires exclusive one-pass joint mode",
        )?;
        replay_require(
            fs::canonicalize(&a.checkpoint)?
                == fs::canonicalize(c.retained_intermediate_root.join("checkpoint-0001"))?
                && a.maximum_report_bytes <= 256 * 1024 * 1024,
            "Context/Cue parent/output admission differs",
        )?;
    }
    Ok(())
}
pub(super) fn policy() -> Value {
    json!({
        "schema":"uor-r4.context-cue-coadapt/1","context":"recorded self_transition3610 only; no gradient or search",
        "active_family":NAME,"coordinates":960,"gradient":"one saved-state native-anchored Cue gradient; detached physical-selector state contrast",
        "construction":"one frozen gradient/actual adjacent displacement/index order; native current-intermediate18 CE descent; no revisit/refill",
        "intermediate_admission":"none; initial15of17 is unselected; every original violation retained",
        "final_gate":"strict original18 combined CE descent, exact original task probability increase, all17 original winners",
        "optimizer_updates":0,"serving_changes":false,"autoregressive_rollout":"NOT_RUN"
    })
}
// Finite960 termination is the bound. A healthy run is never stopped by an elapsed estimate.
pub(super) fn progress(a: &Args, start: Instant) -> Result<()> {
    disk_floor(a)?;
    let path = a.out.join("elapsed-estimate-extension-needed.json");
    if start.elapsed().as_secs_f64() > a.maximum_seconds as f64 && !path.exists() {
        write(
            a,
            "elapsed-estimate-extension-needed.json",
            &json!({"elapsed_seconds":start.elapsed().as_secs_f64(),
            "estimated_seconds":a.maximum_seconds,"action":"continue healthy finite work; parent renews lease and records resource estimate extension",
            "hard_stop":false,"termination":if a.coupled_episode_learning.as_ref().is_some_and(|c|c.prefix_transaction == super::coupled_episode_learning::PrefixTransaction::ProtectedDiscreteFeedback){"411 inherited backwards;zero new training graphs;32 fixed discrete feedback rounds;at most32 unique eligible joint proposals;one selected restage;391 native reload steps;no coordinate sweep"}else if a.coupled_episode_learning.as_ref().is_some_and(|c|c.prefix_transaction == super::coupled_episode_learning::PrefixTransaction::ProtectedJointVector){"31 objective plus380 protected-margin Jacobians;256 fixed projection passes;four original-epoch joint Prefix/Generate vectors;one selected restage;391 native reload steps;no coordinate sweep"}else if a.coupled_episode_learning.as_ref().is_some_and(|c|c.prefix_transaction == super::coupled_episode_learning::PrefixTransaction::GradientVectorPrefix){"four original-epoch Prefix vectors plus960 Generate coordinates/up to13444 alternatives/one selected Prefix restage/391 native reloadedsteps"}else if a.coupled_episode_learning.is_some(){"1920 mixed coordinates/up to14400 alternatives/391 native reloadedsteps"}else if a.generate_episode_learning.is_some(){"960 coordinates/13440 alllegal alternatives/391 reloadedsteps"}else if a.prefix_fragment_learning.as_ref().is_some_and(|c|c.episode.is_some()){"exactly960 rankedcoordinates plus391 unique episode/guard native exportedreload steps"}else if a.prefix_fragment_learning.as_ref().is_some_and(|c|c.joint.is_some()){"exactly960 rankedcoordinates plus383 unique joint/guard native exportedreload steps"}else if a.prefix_fragment_learning.as_ref().is_some_and(|c|c.trajectory.is_some()){"exactly960 reused rankedcoordinates plus380uniqueguard+1task exported reloadedsteps"}else{"exactly960 rankedcoordinates plus18exportedreloadedsteps"}}),
        )?;
    }
    Ok(())
}
pub(super) fn sealed(root: &Path, report: &str, seal: &str) -> Result<Value> {
    report_output::verify(root)?;
    replay_require(
        sha256_file(&root.join("report.json"))? == report
            && sha256_file(&root.join("manifest.json"))? == seal,
        "Context/Cue sealed pin differs",
    )?;
    let r = read(&root.join("report.json"))?;
    replay_require(
        r["status"] == "COMPLETED",
        "Context/Cue authority incomplete",
    )?;
    Ok(r)
}
pub(super) fn dec<T: serde::de::DeserializeOwned>(v: &Value) -> Result<T> {
    Ok(serde_json::from_value(v.clone())?)
}
pub(super) fn idx(v: &Value) -> Result<usize> {
    Ok(v.as_u64()
        .ok_or_else(|| bad("Context/Cue index missing"))?
        .try_into()?)
}
pub(super) fn floats(p: &Path) -> Result<Vec<f32>> {
    let b = fs::read(p)?;
    replay_require(b.len() == 3840, "Cue raw master shape differs")?;
    let v = b
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        .collect::<Vec<_>>();
    replay_require(
        v.iter()
            .all(|x| x.is_finite() && (-1.75..=1.75).contains(x)),
        "Cue master domain differs",
    )?;
    Ok(v)
}
fn carrier_state(v: &Value) -> Result<CarrierState> {
    let codes = v["codes"]
        .as_array()
        .ok_or_else(|| bad("Cue observed codes missing"))?
        .iter()
        .map(|c| {
            Ok(ObservedCode {
                root: idx(&c["root"])?.try_into()?,
                radius_bin: idx(&c["radius_bin"])?.try_into()?,
                present: c["present"]
                    .as_bool()
                    .ok_or_else(|| bad("Cue present mask missing"))?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(CarrierState {
        token_ids: dec(&v["token_ids"])?,
        states: dec(&v["states"])?,
        raw_roots: dec(&v["raw_roots"])?,
        categories: dec(&v["categories"])?,
        codes,
    })
}
fn cue_trace(v: &Value) -> Result<CueCarrierTrace> {
    let c = &v["costs"];
    Ok(CueCarrierTrace {
        metadata: dec(&v["metadata"])?,
        query: carrier_state(&v["query"])?,
        cues: v["cues"]
            .as_array()
            .ok_or_else(|| bad("Cue list missing"))?
            .iter()
            .map(|x| {
                Ok(CueSegmentTrace {
                    source_segment_index: idx(&x["source_segment_index"])?,
                    context_segment_index: idx(&x["context_segment_index"])?,
                    state: carrier_state(&x["state"])?,
                })
            })
            .collect::<Result<_>>()?,
        candidate_cue_indices: dec(&v["candidate_cue_indices"])?,
        copy_q24: dec(&v["copy_q24"])?,
        angular_indices: dec(&v["angular_indices"])?,
        relative_roots: dec(&v["relative_roots"])?,
        costs: CueCarrierCosts {
            extra_encoder_tokens: idx(&c["extra_encoder_tokens"])?,
            extra_context_coefficient_reads: idx(&c["extra_context_coefficient_reads"])?,
            extra_potential_table_reads: idx(&c["extra_potential_table_reads"])?,
            extra_geometry_relative_reads: idx(&c["extra_geometry_relative_reads"])?,
            logical_sidecar_payload_bytes: idx(&c["logical_sidecar_payload_bytes"])?,
            compiled_table_payload_bytes: idx(&c["compiled_table_payload_bytes"])?,
            packed_source_bytes: idx(&c["packed_source_bytes"])?,
        },
    })
}
fn prefix_trace(v: &Value) -> Result<PrefixTransportTrace> {
    let c = &v["costs"];
    Ok(PrefixTransportTrace {
        metadata: dec(&v["metadata"])?,
        response: PrefixState {
            token_ids: dec(&v["response"]["token_ids"])?,
            states: dec(&v["response"]["states"])?,
        },
        sources: v["sources"]
            .as_array()
            .ok_or_else(|| bad("Prefix sources absent"))?
            .iter()
            .map(|x| {
                Ok(SourcePrefixTrace {
                    source_segment_index: idx(&x["source_segment_index"])?,
                    token_ids: dec(&x["token_ids"])?,
                    states_before: dec(&x["states_before"])?,
                })
            })
            .collect::<Result<Vec<_>>>()?,
        candidate_source_indices: dec(&v["candidate_source_indices"])?,
        candidate_offsets: dec(&v["candidate_offsets"])?,
        angular_indices: dec(&v["angular_indices"])?,
        relative_roots: dec(&v["relative_roots"])?,
        copy_q24: dec(&v["copy_q24"])?,
        costs: PrefixTransportCosts {
            extra_source_encoder_tokens: idx(&c["extra_source_encoder_tokens"])?,
            extra_response_encoder_tokens: idx(&c["extra_response_encoder_tokens"])?,
            extra_context_coefficient_reads: idx(&c["extra_context_coefficient_reads"])?,
            extra_angular_table_reads: idx(&c["extra_angular_table_reads"])?,
            extra_geometry_relative_operations: idx(&c["extra_geometry_relative_operations"])?,
            logical_sidecar_payload_bytes: idx(&c["logical_sidecar_payload_bytes"])?,
            compiled_table_payload_bytes: idx(&c["compiled_table_payload_bytes"])?,
            packed_source_bytes: idx(&c["packed_source_bytes"])?,
            preparation_vec_containers: idx(&c["preparation_vec_containers"])?,
        },
    })
}
fn query_shape(v: &Value) -> bool {
    v.as_array()
        .is_some_and(|r| r.len() == 8 && r.iter().all(|c| c.as_u64().is_some_and(|x| x < 120)))
}
pub(super) fn codes(v: &Value) -> Result<Vec<H4Code>> {
    replay_require(query_shape(v), "Cue H4 state shape/bounds differ")?;
    dec::<Vec<u8>>(v)?
        .into_iter()
        .map(|x| H4Code::try_from(x).map_err(Into::into))
        .collect()
}
pub(super) struct Frame {
    pub(super) input: usize,
    pub(super) position: usize,
    pub(super) id: String,
    pub(super) prefix: Vec<u32>,
    pub(super) target: u32,
    pub(super) weight: f64,
    pub(super) native: Value,
    pub(super) trace: CueCarrierTrace,
    pub(super) prefix_trace: Option<PrefixTransportTrace>,
    pub(super) ids: Vec<u32>,
    pub(super) base_copy: Vec<i64>,
    pub(super) u: Vec<i64>,
    pub(super) cue_keys: Vec<Vec<Option<usize>>>,
    pub(super) query: Vec<H4Code>,
    pub(super) sources: Vec<Vec<H4Code>>,
}
#[derive(Clone)]
pub(super) struct Pool {
    pub(super) donor: usize,
    pub(super) post: Vec<H4Code>,
    pub(super) generate: Vec<i64>,
    pub(super) copy: Vec<i64>,
    pub(super) base_copy: Vec<i64>,
    pub(super) trace: VocabularyActionTrace,
}
pub(super) struct DonorCache {
    pub(super) source_binding: NativeArtifactBinding,
    pub(super) generate_sha256: String,
    pub(super) bridge_sha256: String,
    pub(super) values: BTreeMap<(usize, usize), (String, Vec<H4Code>, Vec<i64>)>,
    pub(super) bytes: usize,
    pub(super) peak: usize,
    pub(super) calls: usize,
    pub(super) limit: usize,
}
impl DonorCache {
    pub(super) fn new(p: &ContinuationParent) -> Self {
        Self {
            source_binding: p.binding.clone(),
            generate_sha256: sha256_bytes(&p.generate),
            bridge_sha256: sha256_bytes(&p.bridge),
            values: BTreeMap::new(),
            bytes: 0,
            peak: 0,
            calls: 0,
            limit: CACHE_LIMIT,
        }
    }
    pub(super) fn with_limit(p: &ContinuationParent, limit: usize) -> Result<Self> {
        replay_require(
            limit > 0 && limit <= 256 * 1024 * 1024,
            "donor cache limit outside admitted scope",
        )?;
        let mut cache = Self::new(p);
        cache.limit = limit;
        if limit > CACHE_LIMIT {
            cache.bytes = 4096;
            cache.peak = 4096;
        }
        Ok(cache)
    }
    pub(super) fn get(
        &mut self,
        i: usize,
        j: usize,
        f: &Frame,
        p: &ContinuationParent,
    ) -> Result<(Vec<H4Code>, Vec<i64>)> {
        replay_require(
            p.binding == self.source_binding
                && sha256_bytes(&p.generate) == self.generate_sha256
                && sha256_bytes(&p.bridge) == self.bridge_sha256
                && f.trace.metadata.parent_artifact == self.source_binding
                && f.query.len() == 8
                && f.sources.iter().all(|s| s.len() == 8),
            "Cue donor cache epoch/state binding differs",
        )?;
        let source = f
            .sources
            .get(j)
            .ok_or_else(|| bad("Physical donor absent"))?;
        let witness = sha256_bytes(&serde_json::to_vec(
            &json!({"input_index":f.input,"position":f.position,
            "query":f.query.iter().map(|c|c.index()).collect::<Vec<_>>(),"source":source.iter().map(|c|c.index()).collect::<Vec<_>>(),
            "physical_occurrence":f.native["bank_trace"]["cue_bank"]["bank"]["candidates"][j]}),
        )?);
        if let Some(v) = self.values.get(&(i, j)) {
            replay_require(
                v.0 == witness,
                "Cue donor cache frame/occurrence state mismatch",
            )?;
            return Ok((v.1.clone(), v.2.clone()));
        }
        let bridge = NativeGeometricReadStateBridge::from_bytes(&p.bridge, p.integer.binding())?;
        let mut post = vec![H4Code::IDENTITY; 8];
        let mut actions = post.clone();
        let mut scores = vec![0; 960];
        bridge.apply_into(
            &f.query,
            source,
            &mut post,
            &mut actions,
            &mut scores,
            &mut BridgeReadCounts::default(),
        )?;
        let model = NativeGeometricGenerate::from_bytes(&p.generate, p.integer.binding())?;
        let mut g = vec![0; 4096];
        model.score_into(&post, &mut g, &mut GenerateReadCounts::default())?;
        // The expanded trajectory cache also charges conservative container/identity overhead.
        let overhead = if self.limit > CACHE_LIMIT { 512 } else { 64 };
        let bytes = g.len() * std::mem::size_of::<i64>()
            + post.len() * std::mem::size_of::<H4Code>()
            + overhead;
        replay_require(
            bytes + 4096 < self.limit,
            "donor cache entry exceeds admitted cap",
        )?;
        if self.bytes + bytes > self.limit {
            self.values.clear();
            self.bytes = if self.limit > CACHE_LIMIT { 4096 } else { 0 };
        }
        self.bytes += bytes;
        self.peak = self.peak.max(self.bytes);
        self.calls += 1;
        self.values
            .insert((i, j), (witness, post.clone(), g.clone()));
        Ok((post, g))
    }
}
fn earliest(v: &[i64]) -> Result<usize> {
    let mut winner = 0;
    let first = v.first().ok_or_else(|| bad("Copy pool empty"))?;
    let mut best = *first;
    for (i, &x) in v.iter().enumerate().skip(1) {
        if x > best {
            winner = i;
            best = x;
        }
    }
    Ok(winner)
}
pub(super) fn native_code(v: f32) -> Result<i8> {
    replay_require(
        v.is_finite() && (-1.75..=1.75).contains(&v),
        "Cue Q4 master out of range",
    )?;
    Ok((4. * v).round().clamp(-7., 7.) as i8)
}
pub(super) fn staged_base(
    original: &[i64],
    keys: &[Vec<Option<usize>>],
    parent: &[f32],
    current: &[f32],
) -> Result<Vec<i64>> {
    replay_require(
        keys.len() == original.len()
            && parent.len() == 960
            && current.len() == 960
            && keys
                .iter()
                .all(|row| row.len() == 8 && row.iter().flatten().all(|k| *k < 960)),
        "Angular physical incidence shape/domain differs",
    )?;
    let mut base = original.to_vec();
    for (j, row) in keys.iter().enumerate() {
        for key in row.iter().flatten() {
            let d = i64::from(native_code(current[*key])? - native_code(parent[*key])?) * (1 << 22);
            base[j] = base[j]
                .checked_add(d)
                .ok_or_else(|| bad("Cue staged Copy overflow"))?;
        }
    }
    Ok(base)
}
pub(super) fn score(
    i: usize,
    f: &Frame,
    parent: &[f32],
    current: &[f32],
    p: &ContinuationParent,
    cache: &mut DonorCache,
    reducer: &mut NativeVocabularyActions,
) -> Result<Pool> {
    let base = staged_base(&f.base_copy, &f.cue_keys, parent, current)?;
    let donor = earliest(&base)?;
    let (post, g) = cache.get(i, donor, f, p)?;
    let generate = g
        .iter()
        .zip(&f.u)
        .map(|(g, u)| g.checked_add(*u).ok_or_else(|| bad("Cue G+U overflow")))
        .collect::<Result<Vec<_>>>()?;
    let copy = base
        .iter()
        .zip(&f.ids)
        .map(|(c, id)| {
            c.checked_add(f.u[*id as usize])
                .ok_or_else(|| bad("Cue Copy+U overflow"))
        })
        .collect::<Result<Vec<_>>>()?;
    let trace = reducer.reduce_trace(&generate, &f.ids, &copy)?;
    Ok(Pool {
        donor,
        post,
        generate,
        copy,
        base_copy: base,
        trace,
    })
}
fn mass(p: &Pool, target: u32) -> Result<(u64, u64)> {
    let m = p
        .trace
        .token_masses
        .iter()
        .find(|m| m.token_id == target)
        .ok_or_else(|| bad("Cue target mass missing"))?
        .weight_q31;
    let d = p.trace.summary.total_weight_q31;
    replay_require(m > 0 && m <= d, "Cue masses invalid")?;
    Ok((m, d))
}
#[derive(Clone, serde::Serialize)]
pub(super) struct ObjectiveRole {
    pub(super) input: usize,
    pub(super) position: usize,
    pub(super) task: bool,
    pub(super) weight: f64,
}
#[derive(Clone, serde::Serialize)]
pub(super) struct ObjectiveSpec {
    pub(super) tasks: Vec<(usize, usize)>,
    pub(super) references: usize,
    pub(super) roles: Option<Vec<ObjectiveRole>>,
}
impl ObjectiveSpec {
    pub(super) fn is_task(&self, f: &Frame) -> bool {
        self.tasks.contains(&(f.input, f.position))
    }
}
pub(super) fn objective_for_spec(
    frames: &[Frame],
    pools: &[Pool],
    spec: &ObjectiveSpec,
) -> Result<Value> {
    if let Some(roles) = &spec.roles {
        return objective_for_roles(frames, pools, spec, roles);
    }
    replay_require(
        frames.len() == pools.len() && spec.tasks.len() == 3 && spec.references == 17,
        "joint objective shape differs",
    )?;
    let mut task = 0.;
    let mut reference = 0.;
    let mut violations = Vec::new();
    let mut phases = Vec::new();
    let mut rows = Vec::new();
    let mut refs = 0;
    for (f, p) in frames.iter().zip(pools) {
        let (m, d) = mass(p, f.target)?;
        let ce = -(m as f64 / d as f64).ln() * f.weight;
        let role = if spec.is_task(f) {
            task += ce;
            "task_phase"
        } else {
            reference += ce;
            refs += 1;
            if p.trace.summary.chosen_token_id != f.target {
                violations.push(json!({"input_index":f.input,"position":f.position,"id":f.id,"target":f.target,"chosen":p.trace.summary.chosen_token_id}));
            }
            "reference"
        };
        let row = json!({"input_index":f.input,"position":f.position,"id":f.id,"target":f.target,"weight":f.weight,"role":role,"weighted_ce":ce,"target_mass":m,"total_mass":d,"pool":p.trace.summary,"donor":p.donor,
            "generate_sha256":sha256_bytes(&p.generate.iter().flat_map(|x|x.to_le_bytes()).collect::<Vec<_>>()),"copy_sha256":sha256_bytes(&p.copy.iter().flat_map(|x|x.to_le_bytes()).collect::<Vec<_>>())});
        if role == "task_phase" {
            phases.push(row.clone());
        }
        rows.push(row);
    }
    replay_require(
        refs == spec.references
            && phases.len() == spec.tasks.len()
            && phases
                .iter()
                .map(|r| (r["input_index"].as_u64(), r["position"].as_u64()))
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                == 3,
        "joint objective role coverage differs",
    )?;
    Ok(
        json!({"combined":task+reference,"task":task,"reference":reference,"correct_reference_frames":refs-violations.len(),"original_reference_violations":violations,"phases":phases,"all_phase_winners":phases.iter().all(|r|r["pool"]["chosen_token_id"]==r["target"]),"terms":rows}),
    )
}
pub(super) fn coalesced_role_weights(
    roles: &[ObjectiveRole],
) -> Result<BTreeMap<(usize, usize), f64>> {
    let mut weights = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for r in roles {
        replay_require(
            r.weight.is_finite() && r.weight > 0. && seen.insert((r.input, r.position, r.task)),
            "duplicate/invalid episode role",
        )?;
        *weights.entry((r.input, r.position)).or_insert(0.) += r.weight;
    }
    Ok(weights)
}
fn objective_for_roles(
    frames: &[Frame],
    pools: &[Pool],
    spec: &ObjectiveSpec,
    roles: &[ObjectiveRole],
) -> Result<Value> {
    replay_require(
        frames.len() == pools.len()
            && spec.references == 17
            && spec.tasks.len() == 15
            && roles.len() == 32
            && frames.len() == 31,
        "complete episode role/physical coverage differs",
    )?;
    let mut task = 0.;
    let mut reference = 0.;
    let mut terms = Vec::new();
    let mut phases = Vec::new();
    let mut violations = Vec::new();
    let physical_weights = coalesced_role_weights(roles)?;
    for r in roles {
        let i = frames
            .iter()
            .position(|f| f.input == r.input && f.position == r.position)
            .ok_or_else(|| bad("episode role physical frame missing"))?;
        let f = &frames[i];
        let p = &pools[i];
        let (m, d) = mass(p, f.target)?;
        let ce = -(m as f64 / d as f64).ln() * r.weight;
        let row = json!({"input_index":f.input,"position":f.position,"id":f.id,"target":f.target,
            "weight":r.weight,"physical_index":i,"role":if r.task{"task_phase"}else{"reference"},
            "weighted_ce":ce,"target_mass":m,"total_mass":d,"pool":p.trace.summary,"donor":p.donor});
        if r.task {
            task += ce;
            phases.push(row.clone());
        } else {
            reference += ce;
            if p.trace.summary.chosen_token_id != f.target {
                violations.push(row.clone());
            }
        }
        terms.push(row);
    }
    replay_require(
        phases.len() == 15
            && roles.iter().filter(|r| !r.task).count() == 17
            && physical_weights.len() == frames.len()
            && frames.iter().all(|f| {
                physical_weights
                    .get(&(f.input, f.position))
                    .is_some_and(|w| (w - f.weight).abs() < 1e-15)
            }),
        "episode coalesced gradient weights differ from role components",
    )?;
    Ok(
        json!({"combined":task+reference,"task":task,"reference":reference,
        "correct_reference_frames":17-violations.len(),"original_reference_violations":violations,
        "all_phase_winners":phases.iter().all(|r|r["pool"]["chosen_token_id"]==r["target"]),
        "phases":phases,"terms":terms,"weighted_roles":roles.len(),"unique_physical_frames":frames.len()}),
    )
}
pub(super) fn episode_gate(b: &Value, v: &Value) -> Result<Value> {
    let strict = |key: &str| -> Result<bool> {
        let x = b[key]
            .as_f64()
            .ok_or_else(|| bad("episode original CE absent"))?;
        let y = v[key]
            .as_f64()
            .ok_or_else(|| bad("episode candidate CE absent"))?;
        replay_require(x.is_finite() && y.is_finite(), "episode nonfinite CE")?;
        Ok(y < x - 1e-10 * (1. + x.abs()))
    };
    let combined = strict("combined")?;
    let episode = strict("task")?;
    let refs = v["correct_reference_frames"] == 17;
    let phases = v["all_phase_winners"] == true;
    Ok(
        json!({"finite_joint_positive":combined&&episode&&refs&&phases,
        "combined_ce_descent":combined,"episode_conditional_ce_descent":episode,
        "all_episode_phase_winners":phases,"all17_reference_winners":refs,
        "strict_ce_tolerance":"1e-10*(1+abs(original_CE))",
        "scope":"complete teacherforced episode including EOS; actual typed wholeanswer/EOS separate"}),
    )
}
pub(super) fn joint_gate(b: &Value, v: &Value) -> Result<Value> {
    let strict = |key: &str| -> Result<bool> {
        let before = b[key]
            .as_f64()
            .ok_or_else(|| bad("joint baseline CE absent"))?;
        let after = v[key]
            .as_f64()
            .ok_or_else(|| bad("joint current CE absent"))?;
        replay_require(
            before.is_finite() && after.is_finite(),
            "joint CE nonfinite",
        )?;
        Ok(after < before - 1e-10 * (1. + before.abs()))
    };
    let combined = strict("combined")?;
    let task = strict("task")?;
    let refs = v["correct_reference_frames"] == 17;
    let phases = v["all_phase_winners"] == true;
    Ok(
        json!({"finite_joint_positive":combined&&task&&refs&&phases,"combined_ce_descent":combined,"word_conditional_ce_descent":task,"all3_phase_winners":phases,"all17_reference_winners":refs,"strict_ce_tolerance":"1e-10*(1+abs(original_CE))","scope":"teacher-forced ordered word; actual wholeanswer/EOS separate"}),
    )
}
pub(super) fn objective(frames: &[Frame], pools: &[Pool]) -> Result<Value> {
    replay_require(frames.len() == pools.len(), "Cue objective length differs")?;
    let mut task = 0.;
    let mut reference = 0.;
    let mut violations = Vec::new();
    let mut rows = Vec::new();
    for (f, p) in frames.iter().zip(pools) {
        let (m, d) = mass(p, f.target)?;
        let ce = -(m as f64 / d as f64).ln() * f.weight;
        if f.input == 245 && f.position == 4 {
            task += ce;
        } else {
            reference += ce;
            if p.trace.summary.chosen_token_id != f.target {
                violations.push(json!({"input_index":f.input,"position":f.position,"id":f.id,"target":f.target,"chosen":p.trace.summary.chosen_token_id}));
            }
        }
        rows.push(json!({"input_index":f.input,"position":f.position,"id":f.id,"target":f.target,"weight":f.weight,
            "target_mass":m,"total_mass":d,"pool":p.trace.summary,"donor":p.donor,
            "generate_sha256":sha256_bytes(&p.generate.iter().flat_map(|x|x.to_le_bytes()).collect::<Vec<_>>()),
            "copy_sha256":sha256_bytes(&p.copy.iter().flat_map(|x|x.to_le_bytes()).collect::<Vec<_>>())}));
    }
    let t = rows
        .iter()
        .find(|r| r["input_index"] == 245 && r["position"] == 4)
        .ok_or_else(|| bad("Cue task missing"))?;
    Ok(
        json!({"combined":task+reference,"task":task,"reference":reference,"correct_reference_frames":17-violations.len(),
        "original_reference_violations":violations,"task_target_mass":t["target_mass"],"task_total_mass":t["total_mass"],"terms":rows}),
    )
}
pub(super) fn parse_saved_native_frame(
    input: usize,
    position: usize,
    id: String,
    prefix: Vec<u32>,
    target: u32,
    weight: f64,
    v: &Value,
    p: &ContinuationParent,
    use_prefix: bool,
) -> Result<Frame> {
    let bank = &v["bank_trace"]["cue_bank"]["bank"];
    let trace = cue_trace(&v["bank_trace"]["cue_bank"]["carrier"])?;
    replay_require(
        trace.metadata.parent_artifact == p.binding && query_shape(&v["bridge"]["query_state"]),
        "saved frame Source/state epoch differs",
    )?;
    let ids: Vec<u32> = dec(&v["copy_ids"])?;
    let u: Vec<i64> = dec(&v["continuation"]["delta_scores_q24"])?;
    replay_require(
        u.len() == 4096 && ids.iter().all(|id| *id < 4096),
        "Cue U/Copy shape differs",
    )?;
    let copy: Vec<i64> = dec(&v["copy_q24"])?;
    replay_require(copy.len() == ids.len(), "Cue physical Copy count differs")?;
    let base_copy = copy
        .iter()
        .zip(&ids)
        .map(|(x, id)| {
            x.checked_sub(u[*id as usize])
                .ok_or_else(|| bad("Cue Copy subtraction overflow"))
        })
        .collect::<Result<Vec<_>>>()?;
    let heads = bank["heads"]
        .as_array()
        .ok_or_else(|| bad("Cue bank heads absent"))?;
    for (j, b) in base_copy.iter().enumerate() {
        let sum = heads.iter().try_fold(0i64, |s, h| {
            Ok::<_, Box<dyn std::error::Error>>(
                s.checked_add(
                    h["scores_q24"][j]
                        .as_i64()
                        .ok_or_else(|| bad("Cue head score missing"))?,
                )
                .ok_or_else(|| bad("Cue head sum overflow"))?,
            )
        })?;
        replay_require(sum == *b, "Cue BASECopy head sum differs")?;
    }
    let candidates = bank["candidates"]
        .as_array()
        .ok_or_else(|| bad("Cue candidates absent"))?;
    replay_require(
        candidates.len() == ids.len() && trace.angular_indices.len() == 8,
        "Cue incidence shape differs",
    )?;
    replay_require(
        trace
            .angular_indices
            .iter()
            .all(|r| r.len() == ids.len() && r.iter().flatten().all(|b| *b < 120))
            && trace.copy_q24.len() == 2
            && trace.copy_q24.iter().all(|r| r.len() == ids.len())
            && query_shape(&v["bridge"]["query_state"]),
        "Cue incidence/state bounds differ",
    )?;
    let mut keys = Vec::new();
    let mut sources = Vec::new();
    for (j, x) in candidates.iter().enumerate() {
        replay_require(
            x["bank_index"] == j && x["occurrence"]["token_id"] == ids[j],
            "Cue physical ordinal identity differs",
        )?;
        keys.push(
            (0..8)
                .map(|l| trace.angular_indices[l][j].map(|b| l * 120 + usize::from(b)))
                .collect(),
        );
        sources.push(codes(
            &bank["context"]["states"][idx(&x["context_position"])?],
        )?);
    }
    let prefix_trace = if use_prefix {
        Some(prefix_trace(&v["bank_trace"]["prefix"])?)
    } else {
        None
    };
    if let Some(prefix) = &prefix_trace {
        replay_require(
            prefix.angular_indices.len() == 8
                && prefix
                    .angular_indices
                    .iter()
                    .all(|row| row.len() == ids.len() && row.iter().all(|b| *b < 120))
                && prefix.metadata.parent_artifact == p.binding
                && prefix.metadata.frozen_cue == trace.metadata,
            "Prefix original unmasked key/Source/Cue binding differs",
        )?;
        keys = (0..ids.len())
            .map(|j| {
                (0..8)
                    .map(|lane| Some(lane * 120 + usize::from(prefix.angular_indices[lane][j])))
                    .collect()
            })
            .collect();
    }
    let query = codes(&v["bridge"]["query_state"])?;
    Ok(Frame {
        input,
        position,
        id,
        prefix,
        target,
        weight,
        native: v.clone(),
        trace,
        prefix_trace,
        ids,
        base_copy,
        u,
        cue_keys: keys,
        query,
        sources,
    })
}

pub(super) fn prepare_frames(
    a: &Args,
    c: &Config,
    original: &ContinuationParent,
    finite: &ContinuationParent,
    original_seed: bool,
) -> Result<(Vec<Frame>, Value)> {
    let baseline = read(&c.retained_probe_root.join("baseline.json"))?;
    replay_require(
        baseline["combined"] == 4.391651926613631 && baseline["correct_reference_frames"] == 17,
        "Original Cue gate baseline differs",
    )?;
    for (path, hash) in [
        (&a.training_inputs, INPUT_SHA),
        (&a.training_labels, LABEL_SHA),
    ] {
        report_output::verify(&seal_for(path)?)?;
        replay_require(sha256_file(path)? == hash, "Cue panel pin differs")?;
    }
    replay_require(
        a.training_inputs == a.development_inputs && a.training_labels == a.development_labels,
        "Cue panel paths differ",
    )?;
    let reducer = NativeVocabularyActions::new(original.integer.binding().clone(), &original.exp)?;
    let legal = reducer.legal_token_ids().iter().copied().collect();
    let eps = load_panel(
        &a.training_inputs,
        &a.training_labels,
        &original.integer,
        &original.tokenizer,
        &legal,
        512,
    )?;
    replay_require(eps.len() == 512, "Cue panel length differs")?;
    let mut frames = Vec::new();
    let mut original_pools = Vec::new();
    let mut red = NativeVocabularyActions::new(original.integer.binding().clone(), &original.exp)?;
    for input in COHORT {
        for position in [3, 4] {
            let file = format!("baseline-row-{input:04}-position-{position:02}.json");
            let old = read(&c.retained_probe_root.join(&file))?;
            let saved = if original_seed {
                old.clone()
            } else {
                read(&c.retained_finite_root.join(format!(
                    "candidate-row-{input:04}-position-{position:02}.json"
                )))?
            };
            let e = eps.get(input).ok_or_else(|| bad("Cue episode missing"))?;
            let prefix: Vec<u32> = dec(&old["actual_prefix_ids"])?;
            let target = *e
                .target
                .get(position)
                .ok_or_else(|| bad("Cue target position missing"))?;
            let weight = if input == 245 && position == 4 {
                1.
            } else {
                1. / 17.
            };
            replay_require(
                old["input_index"] == input
                    && old["position"] == position
                    && old["id"] == e.packet.id
                    && saved["input_index"] == input
                    && saved["position"] == position
                    && saved["id"] == e.packet.id
                    && saved["actual_prefix_ids"] == old["actual_prefix_ids"]
                    && old["target_label_only"] == target
                    && saved["target_label_only"] == target
                    && old["weight"] == weight
                    && saved["weight"] == weight,
                "Cue frame identity/weight differs",
            )?;
            let actual = read(
                &c.retained_intermediate_root
                    .join(format!("development-0001-row-{input:04}.json")),
            )?;
            replay_require(
                actual["id"] == e.packet.id
                    && actual["generation"][position]["actual_prefix_ids"] == json!(prefix)
                    && prefix.iter().enumerate().all(|(j, t)| {
                        actual["generation"][j]["pool"]["summary"]["chosen_token_id"] == *t
                    }),
                "Cue actual prefix authority differs",
            )?;
            let v = &saved["native"];
            let bank = &v["bank_trace"]["cue_bank"]["bank"];
            let trace = cue_trace(&v["bank_trace"]["cue_bank"]["carrier"])?;
            replay_require(
                trace.query.token_ids == e.packet.query_ids
                    && v["continuation"]["query_tokens"] == e.packet.query_ids.len()
                    && v["continuation"]["actual_prefix_tokens"] == prefix.len()
                    && bank["candidates"]
                        == old["native"]["bank_trace"]["cue_bank"]["bank"]["candidates"]
                    && bank["segments"]
                        == old["native"]["bank_trace"]["cue_bank"]["bank"]["segments"]
                    && v["bank_trace"]["cue_bank"]["carrier"]["candidate_cue_indices"]
                        == old["native"]["bank_trace"]["cue_bank"]["carrier"]
                            ["candidate_cue_indices"],
                "Cue saved packet/query/physical occurrence provenance differs",
            )?;
            replay_require(
                trace.metadata.parent_artifact == finite.binding,
                "Cue trace epoch differs",
            )?;
            let frame = parse_saved_native_frame(
                input,
                position,
                e.packet.id.clone(),
                prefix,
                target,
                weight,
                v,
                finite,
                original_seed,
            )?;
            let ids = frame.ids.clone();
            let oldg: Vec<i64> = dec(&old["native"]["generate_q24"])?;
            let oldc: Vec<i64> = dec(&old["native"]["copy_q24"])?;
            let oldids: Vec<u32> = dec(&old["native"]["copy_ids"])?;
            replay_require(oldids == ids, "Cue frozen physical Copy inventory differs")?;
            let ot = red.reduce_trace(&oldg, &oldids, &oldc)?;
            replay_require(
                serde_json::to_value(&ot)? == old["native"]["pool"],
                "Original saved full pool differs",
            )?;
            original_pools.push(Pool {
                donor: idx(&old["native"]["bridge"]["selected_ordinal"])?,
                post: codes(&old["native"]["post_state"])?,
                generate: oldg,
                copy: oldc,
                base_copy: Vec::new(),
                trace: ot,
            });
            frames.push(frame);
        }
    }
    let reconstructed = objective(&frames, &original_pools)?;
    replay_require(
        reconstructed["combined"] == baseline["combined"]
            && reconstructed["correct_reference_frames"] == 17,
        "Original Cue weighted gate reconstruction differs",
    )?;
    let frame_files=frames.iter().map(|f|Ok(json!({"input_index":f.input,"position":f.position,"id":f.id,"actual_prefix_ids":f.prefix,
        "baseline_sha256":sha256_file(&c.retained_probe_root.join(format!("baseline-row-{:04}-position-{:02}.json",f.input,f.position)))?,
        "finite_sha256":sha256_file(&if original_seed {c.retained_probe_root.join(format!("baseline-row-{:04}-position-{:02}.json",f.input,f.position))} else {c.retained_finite_root.join(format!("candidate-row-{:04}-position-{:02}.json",f.input,f.position))})?})))
        .collect::<Result<Vec<_>>>()?;
    write(
        a,
        "reused-baseline.json",
        &json!({"objective":reconstructed,"sealed_root":c.retained_probe_root,"baseline":baseline,"frame_files":frame_files}),
    )?;
    Ok((frames, reconstructed))
}
// Identical detached state contrast to geometric_bank_generate::add_selector_credit.
pub(super) fn selector(
    factual: &[H4Code],
    alternatives: &[Vec<H4Code>],
    raw: &Tensor,
    d: &Device,
) -> Result<Tensor> {
    let mut one = vec![0f32; 960];
    for (l, c) in factual.iter().enumerate() {
        one[l * 120 + usize::from(c.index())] = 1.;
    }
    let mut contrast = Vec::new();
    for a in alternatives {
        replay_require(a.len() == 8, "Cue donor post shape differs")?;
        for (c, b) in a.iter().zip(factual) {
            for r in 0..120 {
                contrast.push(
                    f32::from(u8::from(r == usize::from(c.index())))
                        - f32::from(u8::from(r == usize::from(b.index()))),
                );
            }
        }
    }
    let weights = candle_nn::ops::softmax(raw, 0)?.reshape((alternatives.len(), 1))?;
    let mean = Tensor::from_vec(contrast, (alternatives.len(), 960), d)?
        .detach()
        .broadcast_mul(&weights)?
        .sum(0)?
        .reshape((8, 120))?;
    Ok((Tensor::from_vec(one, (8, 120), d)? + (&mean - mean.detach())?)?)
}
pub(super) enum ActiveCredit<'a> {
    Cue(&'a CueAngularWeights),
    Prefix(&'a PrefixAngularWeights),
}
impl ActiveCredit<'_> {
    fn name(&self) -> &'static str {
        match self {
            Self::Cue(_) => NAME,
            Self::Prefix(_) => "prefix.coefficients",
        }
    }
    fn label(&self) -> &'static str {
        match self {
            Self::Cue(_) => "cue",
            Self::Prefix(_) => "prefix",
        }
    }
    fn parameters(&self) -> BTreeMap<String, Var> {
        match self {
            Self::Cue(x) => x.parameters(),
            Self::Prefix(x) => x.parameters(),
        }
    }
    pub(super) fn credit(&self, frame: &Frame, device: &Device) -> Result<Tensor> {
        match self {
            Self::Cue(x) => Ok(x.coefficient_credit(&frame.trace, device)?),
            Self::Prefix(x) => Ok(x.coefficient_credit(
                frame
                    .prefix_trace
                    .as_ref()
                    .ok_or_else(|| bad("Prefix trace absent"))?,
                device,
            )?),
        }
    }
}
pub(super) fn gradient(
    a: &Args,
    start: Instant,
    frames: &[Frame],
    p: &ContinuationParent,
    cw: &ActiveCredit,
    g: &GenerateLearningWeights,
    parent: &[f32],
    pools: &[Pool],
    cache: &mut DonorCache,
) -> Result<Vec<f32>> {
    gradient_for_spec(a, start, frames, p, cw, g, parent, pools, cache, None)
}
pub(super) fn gradient_for_spec(
    a: &Args,
    start: Instant,
    frames: &[Frame],
    p: &ContinuationParent,
    cw: &ActiveCredit,
    g: &GenerateLearningWeights,
    parent: &[f32],
    pools: &[Pool],
    cache: &mut DonorCache,
    spec: Option<&ObjectiveSpec>,
) -> Result<Vec<f32>> {
    let d = g.device();
    let prepared = g.prepare_native()?;
    let var = cw
        .parameters()
        .remove(cw.name())
        .ok_or_else(|| bad("Cue Var absent"))?;
    // Complete hard-forward parity for all18 precedes ANY backward call.
    for (i, (f, pool)) in frames.iter().zip(pools).enumerate() {
        replay_require(
            pool.generate == dec::<Vec<i64>>(&f.native["generate_q24"])?
                && pool.copy == dec::<Vec<i64>>(&f.native["copy_q24"])?
                && pool.post == codes(&f.native["post_state"])?
                && pool.donor == idx(&f.native["bridge"]["selected_ordinal"])?
                && serde_json::to_value(&pool.trace)? == f.native["pool"],
            "Cue complete cached native pool parity differs",
        )?;
        let credit = cw.credit(f, d)?;
        let raw = Tensor::from_vec(
            pool.base_copy
                .iter()
                .map(|x| (*x as f64 / 16777216.) as f32)
                .collect::<Vec<_>>(),
            f.ids.len(),
            d,
        )?;
        let raw = (&raw + (&credit - credit.detach())?)?;
        let alternatives = (0..f.ids.len())
            .map(|j| cache.get(i, j, f, p).map(|x| x.0))
            .collect::<Result<Vec<_>>>()?;
        let state = selector(&pool.post, &alternatives, &raw, d)?;
        let out = g.forward_prepared_state_choices(&prepared, &pool.post, &state)?;
        let generate = out
            .scores_q24
            .iter()
            .zip(&f.u)
            .map(|(a, b)| {
                a.checked_add(*b)
                    .ok_or_else(|| bad("Cue graph anchor overflow"))
            })
            .collect::<Result<Vec<_>>>()?;
        replay_require(
            generate == pool.generate,
            "Cue graph Generate native anchor differs",
        )?;
        replay_require(
            parent.len() == 960 && credit.dims() == [f.ids.len()],
            "Cue graph credit shape differs",
        )?;
    }
    write(
        a,
        "gradient-forward-parity.json",
        &json!({"positions":frames.len(),"active_family":cw.name(),"all_before_backward":true,"native_fullpool_parity":true,"context_encoder_calls":0,"context_backward_calls":0,"device":if d.is_cpu(){"cpu"}else{"cuda"}}),
    )?;
    let mut sum = vec![0f32; 960];
    let mut perterm = Vec::new();
    let mut weighted = 0.;
    for (i, (f, pool)) in frames.iter().zip(pools).enumerate() {
        progress(a, start)?;
        let credit = cw.credit(f, d)?;
        let base = Tensor::from_vec(
            pool.base_copy
                .iter()
                .map(|x| (*x as f64 / 16777216.) as f32)
                .collect::<Vec<_>>(),
            f.ids.len(),
            d,
        )?;
        let raw = (&base + (&credit - credit.detach())?)?;
        let alternatives = (0..f.ids.len())
            .map(|j| cache.get(i, j, f, p).map(|x| x.0))
            .collect::<Result<Vec<_>>>()?;
        let state = selector(&pool.post, &alternatives, &raw, d)?;
        let out = g.forward_prepared_state_choices(&prepared, &pool.post, &state)?;
        let ug = Tensor::from_vec(
            f.u.iter()
                .map(|x| (*x as f64 / 16777216.) as f32)
                .collect::<Vec<_>>(),
            4096,
            d,
        )?;
        let hardg = Tensor::from_vec(
            pool.generate
                .iter()
                .map(|x| (*x as f64 / 16777216.) as f32)
                .collect::<Vec<_>>(),
            4096,
            d,
        )?;
        let softg = (&out.raw_scores + &ug)?;
        let joinedg = (&hardg + (&softg - softg.detach())?)?;
        let uc = Tensor::from_vec(
            f.ids
                .iter()
                .map(|id| (f.u[*id as usize] as f64 / 16777216.) as f32)
                .collect::<Vec<_>>(),
            f.ids.len(),
            d,
        )?;
        let hardc = Tensor::from_vec(
            pool.copy
                .iter()
                .map(|x| (*x as f64 / 16777216.) as f32)
                .collect::<Vec<_>>(),
            f.ids.len(),
            d,
        )?;
        let softc = (&raw + &uc)?;
        let joinedc = (&hardc + (&softc - softc.detach())?)?;
        let loss = (vocabulary_marginal_loss_with_credit(
            &pool.trace,
            &joinedg,
            Some(&joinedc),
            f.target,
            a.credit.policy(),
        )? * f.weight)?;
        weighted += loss.to_scalar::<f32>()? as f64;
        let grads = loss.backward()?;
        let tensor = grads
            .get(var.as_tensor())
            .ok_or_else(|| bad("Cue gradient MISSING; no zero fill"))?;
        let values = tensor
            .flatten_all()?
            .to_device(&Device::Cpu)?
            .to_vec1::<f32>()?;
        replay_require(
            values.len() == 960 && values.iter().all(|v| v.is_finite()),
            "Cue gradient invalid",
        )?;
        for (s, v) in sum.iter_mut().zip(&values) {
            *s += *v;
        }
        let file = format!("{}-gradient-term-{i:02}.f32le", cw.label());
        let bytes = values
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>();
        fs::write(a.out.join(&file), &bytes)?;
        perterm.push(json!({"input_index":f.input,"position":f.position,"id":f.id,"weight":f.weight,"file":file,"bytes":bytes.len(),"sha256":sha256_bytes(&bytes),"status":"PRESENT","missing_gradient_filled_zero":false,"all_zero":values.iter().all(|v|*v==0.),"l2_norm":values.iter().map(|v|f64::from(*v).powi(2)).sum::<f64>().sqrt(),"target":f.target,"termination_target":f.target==1,"period_target":f.target==16}));
    }
    replay_require(
        (weighted
            - if let Some(spec) = spec {
                objective_for_spec(frames, pools, spec)?
            } else {
                objective(frames, pools)?
            }["combined"]
                .as_f64()
                .ok_or_else(|| bad("Cue CE absent"))?)
        .abs()
            < 1e-5,
        "Cue gradient/native weighted objective differs",
    )?;
    replay_require(
        sum.iter().all(|x| x.is_finite()),
        "Cue aggregate gradient nonfinite",
    )?;
    let bytes = sum.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<_>>();
    fs::write(a.out.join(format!("{}-gradient.f32le", cw.label())), &bytes)?;
    write(
        a,
        &format!("{}-gradient-receipt.json", cw.label()),
        &json!({"shape":[960],"active_names":[cw.name()],"bytes":bytes.len(),"sha256":sha256_bytes(&bytes),"file":format!("{}-gradient.f32le",cw.label()),"per_term":perterm,"weighted_graph_ce":weighted,"objective_terms":frames.len(),"coefficient_backward_calls":frames.len(),"objective_spec":spec,"new_gradients":1,"context_gradient":"NOT_RUN","surrogate":format!("native anchored direct {} gather plus existing detached soft-selector state contrast; local conditional utility, not hard argmax derivative",cw.label())}),
    )?;
    Ok(sum)
}
#[derive(Clone, serde::Serialize)]
pub(super) struct Move {
    pub(super) index: usize,
    pub(super) before: i8,
    pub(super) after: i8,
    pub(super) master_before: f32,
    pub(super) master_after: f32,
    pub(super) actual_delta: f64,
    pub(super) gradient: f32,
    pub(super) utility: f64,
    pub(super) status: String,
}
pub(super) fn ranking(m: &[f32], g: &[f32]) -> Result<Vec<Move>> {
    replay_require(
        m.len() == 960 && g.len() == 960,
        "Cue ranking shape differs",
    )?;
    let mut rows = Vec::new();
    for (i, (&v, &grad)) in m.iter().zip(g).enumerate() {
        replay_require(grad.is_finite(), "Cue gradient nonfinite")?;
        let q = native_code(v)?;
        let direction = if grad > 0. {
            -1
        } else if grad < 0. {
            1
        } else {
            0
        };
        let after = q + direction;
        let status = if direction == 0 {
            "zero_gradient"
        } else if !(-7..=7).contains(&after) {
            "saturated"
        } else {
            "eligible"
        };
        let next = if status == "eligible" {
            f32::from(after) / 4.
        } else {
            v
        };
        let delta = f64::from(next) - f64::from(v);
        rows.push(Move {
            index: i,
            before: q,
            after: if status == "eligible" { after } else { q },
            master_before: v,
            master_after: next,
            actual_delta: delta,
            gradient: grad,
            utility: f64::from(grad) * delta,
            status: status.into(),
        });
    }
    rows.sort_by(|a, b| a.utility.total_cmp(&b.utility).then(a.index.cmp(&b.index)));
    Ok(rows)
}
pub(super) fn improved(a: &Value, b: &Value) -> Result<bool> {
    let am = a["task_target_mass"]
        .as_u64()
        .ok_or_else(|| bad("Cue task mass missing"))?;
    let ad = a["task_total_mass"]
        .as_u64()
        .ok_or_else(|| bad("Cue task denominator missing"))?;
    let bm = b["task_target_mass"]
        .as_u64()
        .ok_or_else(|| bad("Cue task mass missing"))?;
    let bd = b["task_total_mass"]
        .as_u64()
        .ok_or_else(|| bad("Cue task denominator missing"))?;
    Ok(u128::from(bm) * u128::from(ad) > u128::from(am) * u128::from(bd))
}
pub(super) fn gate(original: &Value, final_value: &Value) -> Result<Value> {
    let b = original["combined"]
        .as_f64()
        .ok_or_else(|| bad("Cue baseline CE missing"))?;
    let f = final_value["combined"]
        .as_f64()
        .ok_or_else(|| bad("Cue candidate CE missing"))?;
    let descent = f < b - 1e-10 * (1. + b.abs());
    let prob = improved(original, final_value)?;
    let refs = final_value["correct_reference_frames"] == 17;
    Ok(
        json!({"strict_original_ce_descent":descent,"exact_original_task_probability_improved":prob,
        "all17_original_winners_retained":refs,"finite_joint_positive":descent&&prob&&refs,
        "baseline_combined":b,"candidate_combined":f}),
    )
}
pub(super) fn export_joint(
    a: &Args,
    l: &Loaded,
    cw: &CueAngularWeights,
    prefix_override: Option<&PrefixAngularWeights>,
    p: &ContinuationParent,
    d: &Device,
) -> Result<(ContinuationParent, NativeContinuationField, Value)> {
    let (_, generate, _, mut receipt) = checkpoint(a, 1, l)?;
    let root = a.out.join("checkpoint-0001");
    // Generic checkpoint has fixed sidecars; replace only Cue numerical payload and honestly rebind Prefix.
    fs::remove_dir_all(root.join("cue"))?;
    fs::remove_dir_all(root.join("prefix"))?;
    let current_source = SourceRealizerWeights::load_context_potential_on_device(
        &root.join("source"),
        &fs::read(root.join("native/tokenizer.json"))?,
        &Device::Cpu,
    )?;
    let native = NativeSourceRealizer::load(
        &root.join("native"),
        &current_source,
        &dec::<ConsumerIdentity>(&read(&root.join("native/metadata.json"))?["identity"])?,
    )?;
    let carrier = native.compile_cue_carrier(cw.native()?)?;
    let bound_cue =
        CueAngularWeights::from_native(&native, &root.join("native"), &carrier, &Device::Cpu)?;
    let masters = np::snapshot(&cw.parameters())?;
    np::restore(&bound_cue.parameters(), &masters)?;
    bound_cue.save(&root.join("cue"))?;
    let newcarrier = native.compile_cue_carrier(bound_cue.native()?)?;
    let pref = native.compile_prefix_transport(
        &newcarrier,
        match prefix_override {
            Some(x) => x.native()?,
            None => prefix_clone(&l.prefix)?,
        },
    )?;
    let mut bound_prefix = PrefixAngularWeights::from_native(
        &native,
        &root.join("native"),
        &root.join("cue"),
        &newcarrier,
        &pref,
        &Device::Cpu,
    )?;
    let original_prefix = PrefixAngularWeights::load(
        &a.checkpoint.join("prefix"),
        &l.frozen,
        &a.checkpoint.join("native"),
        &a.checkpoint.join("cue"),
        &l.frozen.compile_cue_carrier(l.cue.clone())?,
    )?;
    np::restore(
        &bound_prefix.parameters(),
        &np::snapshot(&prefix_override.unwrap_or(&original_prefix).parameters())?,
    )?;
    bound_prefix.rebind_cue(&native, &root.join("cue"), &newcarrier)?;
    bound_prefix.save(&root.join("prefix"))?;
    let preliminary = ContinuationParent::from_checkpoint(&root)?;
    let field = NativeContinuationField::from_bytes(
        &fs::read(a.checkpoint.join("continuation-field.bin"))?,
        &p.binding,
        &NativeGeometricGenerate::from_bytes(&p.generate, p.integer.binding())?,
    )?;
    let original_generate = NativeGeometricGenerate::from_bytes(&p.generate, p.integer.binding())?;
    let u = ContinuationLearningWeights::from_native(
        &field,
        &original_generate,
        p.integer.binding(),
        d,
    )?;
    restore(
        &a.checkpoint.join("continuation-source"),
        &p.receipt["continuation_parameters"],
        &u.parameters(),
        d,
    )?;
    let rebound = u.export_native(&preliminary.binding, &generate)?;
    replay_require(
        rebound.packed_unary() == field.packed_unary(),
        "Cue export U numerical coefficients changed",
    )?;
    let u_bits = identities(&u.parameters())?;
    let inventory = save_masters(&root.join("continuation-source"), &u.parameters())?;
    let cpu_u = ContinuationLearningWeights::from_native(
        &rebound,
        &generate,
        preliminary.integer.binding(),
        &Device::Cpu,
    )?;
    restore(
        &root.join("continuation-source"),
        &inventory,
        &cpu_u.parameters(),
        &Device::Cpu,
    )?;
    replay_require(
        identities(&cpu_u.parameters())? == u_bits
            && cpu_u
                .export_native(&preliminary.binding, &generate)?
                .to_bytes()?
                == rebound.to_bytes()?,
        "Cue U fractional masters/native independent reload differs",
    )?;
    fs::write(root.join("continuation-field.bin"), rebound.to_bytes()?)?;
    receipt["mode"] = json!(if prefix_override.is_some() {
        "prefix_fragment_learning"
    } else {
        "context_cue_coadapt"
    });
    receipt["policy"] = if prefix_override.is_some() {
        super::prefix_fragment_learning::policy()
    } else {
        policy()
    };
    receipt["active_parameter_names"] = json!([if prefix_override.is_some() {
        "prefix.coefficients"
    } else {
        NAME
    }]);
    receipt["credit_scope"] = json!(if prefix_override.is_some() {
        "only Prefix master gradients extracted/proposed; native-anchored direct retained-latent Prefix coefficient gather plus existing detached conditional physical-donor state contrast through frozen Generate graph; no Context/Cue/U gradients or updates"
    } else {
        "only Cue master gradients extracted/proposed; fixed recorded Context3610 displacement without Context gradient; native-anchored direct Cue gather plus existing detached conditional physical-donor state contrast through frozen Generate graph"
    });
    receipt["coefficient_backward_calls"] = json!(18);
    receipt["frozen_numerical_scope"] = json!(if prefix_override.is_some() {
        "all Source Context/Potential/map, Cue angular/joint, Generate coefficients/prototypes/bias, bridges and U; only Prefix coefficient f32 masters/nativeQ4 changed"
    } else {
        "all Source masters except recorded self_transition3610, Potential/map, Prefix, Cue joint, Generate coefficients/prototypes/bias, bridges and U; shared Cue angular masters learned"
    });
    receipt["fresh_adam"] = json!(false);
    receipt["optimizer_updates"] = json!(0);
    receipt["new_gradients"] = json!(1);
    receipt["continuation_parameters"] = inventory;
    receipt["continuation_sha256"] = json!(sha256_bytes(&rebound.to_bytes()?));
    receipt["cue_parameters"] = save_masters(&root.join("cue-source"), &bound_cue.parameters())?;
    if prefix_override.is_some() {
        receipt["prefix_parameters"] =
            save_masters(&root.join("prefix-source"), &bound_prefix.parameters())?;
    }
    receipt["parent_report_sha256"] = json!(P_REPORT);
    receipt["parent_manifest_sha256"] = json!(P_SEAL);
    fs::write(
        root.join("receipt.json"),
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    fs::write(
        root.join("continuation-source/metadata.json"),
        serde_json::to_vec_pretty(&receipt)?,
    )?;
    let cp = ContinuationParent::from_checkpoint(&root)?;
    let loaded_prefix = PrefixAngularWeights::load(
        &root.join("prefix"),
        &native,
        &root.join("native"),
        &root.join("cue"),
        &newcarrier,
    )?;
    replay_require(
        np::same_bits(
            &np::snapshot(&prefix_override.unwrap_or(&original_prefix).parameters())?,
            &np::snapshot(&loaded_prefix.parameters())?,
        ),
        "Cue frozen Prefix fractional master bits changed",
    )?;
    let loaded = CueAngularWeights::load(&root.join("cue"), &native, &root.join("native"))?;
    replay_require(
        np::same_bits(&masters, &np::snapshot(&loaded.parameters())?)
            && cp.generate
                == if a.coupled_episode_learning.is_some() {
                    generate.to_bytes()?
                } else {
                    p.generate.clone()
                }
            && cp.bridge == p.bridge
            && cp.joint == p.joint
            && cp.exp == p.exp
            && fs::read(root.join("prefix/prefix-q4.bin"))?
                == match prefix_override {
                    Some(x) => x.packed_coefficients()?,
                    None => fs::read(a.checkpoint.join("prefix/prefix-q4.bin"))?,
                }
            && fs::read(root.join("cue/cue-joint-q4.bin"))?
                == fs::read(a.checkpoint.join("cue/cue-joint-q4.bin"))?,
        "Cue export frozen/reloaded payload differs",
    )?;
    Ok((cp, rebound, receipt))
}
pub(super) fn reload_candidate(
    a: &Args,
    cp: &ContinuationParent,
    field: &NativeContinuationField,
    frames: &[Frame],
    pools: &[Pool],
) -> Result<()> {
    reload_candidate_impl(a, cp, field, frames, pools, true)
}
pub(super) fn reload_guard_candidate(
    a: &Args,
    cp: &ContinuationParent,
    field: &NativeContinuationField,
    frames: &[Frame],
    pools: &[Pool],
) -> Result<()> {
    replay_require(
        pools.iter().all(|p| p.trace.actions.is_empty()),
        "compact guard pool actions must be omitted only after exact reduction",
    )?;
    reload_candidate_impl(a, cp, field, frames, pools, false)
}
fn reload_candidate_impl(
    a: &Args,
    cp: &ContinuationParent,
    field: &NativeContinuationField,
    frames: &[Frame],
    pools: &[Pool],
    full_action_trace: bool,
) -> Result<()> {
    let fields = field.to_bytes()?;
    let hash = sha256_bytes(&fields);
    let mut native = cp.generator()?.with_continuation_field(BoundNativeBytes {
        bytes: &fields,
        sha256: &hash,
    })?;
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
    for (f, expected) in frames.iter().zip(pools) {
        let bank = native.admit_bank(continuation_snapshot(&eps[f.input].packet)?)?;
        let s = native.step(&bank, &f.prefix)?;
        replay_require(
            s.generate_raw_scores_q24 == expected.generate
                && s.copy_raw_scores_q24 == expected.copy
                && s.copy_token_ids == f.ids
                && s.post_state == expected.post
                && (if full_action_trace {
                    s.actions == expected.trace
                } else {
                    s.actions.summary == expected.trace.summary
                        && s.actions.token_masses == expected.trace.token_masses
                }),
            "Cue constructed/exported full pool differs",
        )?;
        let bridge = s
            .bridge
            .as_ref()
            .ok_or_else(|| bad("Cue reload bridge missing"))?;
        replay_require(
            bridge.selected_ordinal == expected.donor,
            "Cue reloaded donor differs",
        )?;
        let u = s
            .continuation
            .as_ref()
            .ok_or_else(|| bad("Cue reloaded U witness missing"))?;
        replay_require(
            u.delta_scores_q24 == f.u
                && u.state_codes == codes(&f.native["continuation"]["state_codes"])?
                && u.query_tokens == idx(&f.native["continuation"]["query_tokens"])?
                && u.actual_prefix_tokens
                    == idx(&f.native["continuation"]["actual_prefix_tokens"])?,
            "Cue reloaded U state/scores changed",
        )?;
        let mut snapshot = json!({"generate_q24":s.generate_raw_scores_q24,"copy_ids":s.copy_token_ids,"copy_q24":s.copy_raw_scores_q24,
                "pool":s.actions,"post_state":s.post_state.iter().map(|x|x.index()).collect::<Vec<_>>(),"bank_trace":s.bank_trace,
                "bridge":{"selected_ordinal":bridge.selected_ordinal,"selected_candidate":bridge.selected_candidate,"query_state":bridge.query_state.iter().map(|x|x.index()).collect::<Vec<_>>(),
                    "source_state":bridge.source_state.iter().map(|x|x.index()).collect::<Vec<_>>(),"action_codes":bridge.action_codes.iter().map(|x|x.index()).collect::<Vec<_>>(),"action_scores_q24":bridge.action_scores_q24,"counts":bridge.counts},
                "continuation":s.continuation.as_ref().map(|u|json!({"query_tokens":u.query_tokens,"actual_prefix_tokens":u.actual_prefix_tokens,"state_codes":u.state_codes.iter().map(|x|x.index()).collect::<Vec<_>>(),"delta_scores_q24":u.delta_scores_q24,"counts":u.counts,"encoding_coefficient_reads":u.encoding_coefficient_reads}))});
        if !full_action_trace {
            snapshot["pool"]
                .as_object_mut()
                .ok_or_else(|| bad("native guard pool serialization absent"))?
                .remove("actions");
            snapshot["schema"] = json!("uor-r4.native-prefix-trajectory-pool/1");
            snapshot["pool_action_trace"]=json!("OMITTED_RECONSTRUCTIBLE_FROM_COMPLETE_SCORES; all4096rawGenerate/allphysicalCopy/rawU, complete token masses/summary preserved");
        }
        write(
            a,
            &format!(
                "candidate-row-{:04}-position-{:02}.json",
                f.input, f.position
            ),
            &json!({"input_index":f.input,"position":f.position,"id":f.id,"actual_prefix_ids":f.prefix,"target_label_only":f.target,"weight":f.weight,"native":snapshot}),
        )?;
    }
    Ok(())
}
pub(super) fn run(a: &Args, start: Instant, d: &Device) -> Result<Value> {
    validate_settings(a)?;
    let c = a
        .context_cue_coadapt
        .as_ref()
        .ok_or_else(|| bad("Cue config missing"))?;
    sealed(&c.retained_intermediate_root, P_REPORT, P_SEAL)?;
    sealed(&c.retained_probe_root, B_REPORT, B_SEAL)?;
    let manifest = sha256_file(&c.retained_finite_root.join("manifest.json"))?;
    replay_require(manifest == F_SEAL, "Cue finite seal prefix differs")?;
    let fr = sealed(&c.retained_finite_root, F_REPORT, &manifest)?;
    replay_require(
        fr["selected_model"] == false && fr["finite_probe_positive"] == false,
        "Cue seed must remain an unselected negative",
    )?;
    let original = ContinuationParent::from_checkpoint(&a.checkpoint)?;
    replay_require(
        original.binding.metadata_sha256 == SOURCE
            && sha256_bytes(&original.generate) == G_SHA
            && sha256_file(&a.checkpoint.join("continuation-field.bin"))? == U_SHA,
        "Cue original parent differs",
    )?;
    let fcp = c.retained_finite_root.join("checkpoint-0001");
    let finite = ContinuationParent::from_checkpoint(&fcp)?;
    let tok = fs::read(fcp.join("native/tokenizer.json"))?;
    let sw = SourceRealizerWeights::load_context_potential_on_device(
        &fcp.join("source"),
        &tok,
        &Device::Cpu,
    )?;
    let frozen = NativeSourceRealizer::load(
        &fcp.join("native"),
        &sw,
        &dec::<ConsumerIdentity>(&read(&fcp.join("native/metadata.json"))?["identity"])?,
    )?;
    let original_source = SourceRealizerWeights::load_context_potential_on_device(
        &a.checkpoint.join("source"),
        &fs::read(a.checkpoint.join("native/tokenizer.json"))?,
        &Device::Cpu,
    )?;
    let original_native = NativeSourceRealizer::load(
        &a.checkpoint.join("native"),
        &original_source,
        &dec::<ConsumerIdentity>(&read(&a.checkpoint.join("native/metadata.json"))?["identity"])?,
    )?;
    let original_source_masters = np::snapshot(&original_source.parameters())?;
    let finite_source_masters = np::snapshot(&sw.parameters())?;
    replay_require(
        original_source_masters.len() == finite_source_masters.len(),
        "Cue Source family count differs",
    )?;
    let mut context_changes = Vec::new();
    for (name, old) in &original_source_masters {
        let new = finite_source_masters
            .get(name)
            .ok_or_else(|| bad("Cue frozen Source family missing"))?;
        replay_require(old.len() == new.len(), "Cue Source family shape differs")?;
        for (i, (a, b)) in old.iter().zip(new).enumerate() {
            if a.to_bits() != b.to_bits() {
                context_changes.push((name.clone(), i, *a, *b));
            }
        }
    }
    replay_require(
        context_changes.len() == 1
            && context_changes[0].0 == CONTEXT
            && context_changes[0].1 == 3610
            && context_changes[0].2.to_bits() == (-0.1445201337337494f32).to_bits()
            && context_changes[0].3.to_bits() == (-0.5f32).to_bits()
            && finite.binding.metadata_sha256 == FINITE_SOURCE
            && finite.generate == original.generate
            && finite.bridge == original.bridge
            && finite.prefix == original.prefix
            && finite.joint == original.joint
            && finite.exp == original.exp,
        "Cue seed changed another frozen payload/master",
    )?;
    write(
        a,
        "recorded-context-and-frozen-source.json",
        &json!({"changed_masters":context_changes,
        "original_source":identities(&original_source.parameters())?,"finite_source":identities(&sw.parameters())?,
        "all_other_master_bits_equal":true,"fixed_context_code":3610,"all_other_native_payloads_equal":true}),
    )?;
    drop(original_source_masters);
    drop(finite_source_masters);
    let actual_cue = CueAngularWeights::load(
        &a.checkpoint.join("cue"),
        &original_native,
        &a.checkpoint.join("native"),
    )?;
    let finite_carrier = frozen.compile_cue_carrier(cue_payload(&fcp.join("cue"))?)?;
    let cw = CueAngularWeights::from_native(
        &frozen,
        &fcp.join("native"),
        &finite_carrier,
        &Device::Cpu,
    )?;
    np::restore(&cw.parameters(), &np::snapshot(&actual_cue.parameters())?)?;
    replay_require(
        np::same_bits(
            &np::snapshot(&actual_cue.parameters())?,
            &np::snapshot(&cw.parameters())?,
        ) && cw.packed_coefficients()? == fs::read(fcp.join("cue/cue-q4.bin"))?,
        "Cue rebind lost actual fractional masters",
    )?;
    let cue_parent = floats(&a.checkpoint.join("cue/cue-source-f32.bin"))?;
    replay_require(
        np::snapshot(&cw.parameters())?[NAME]
            .iter()
            .zip(&cue_parent)
            .all(|(a, b)| a.to_bits() == b.to_bits()),
        "Cue seed fractional master bits differ",
    )?;
    let original_field = NativeContinuationField::from_bytes(
        &fs::read(a.checkpoint.join("continuation-field.bin"))?,
        &original.binding,
        &NativeGeometricGenerate::from_bytes(&original.generate, original.integer.binding())?,
    )?;
    let finite_generate =
        NativeGeometricGenerate::from_bytes(&finite.generate, finite.integer.binding())?;
    let finite_field = NativeContinuationField::from_bytes(
        &fs::read(fcp.join("continuation-field.bin"))?,
        &finite.binding,
        &finite_generate,
    )?;
    replay_require(
        original_field.packed_unary() == finite_field.packed_unary(),
        "Cue seed changed numerical U coefficients",
    )?;
    let (frames, baseline) = prepare_frames(a, c, &original, &finite, false)?;
    for f in &frames {
        let state = codes(&f.native["continuation"]["state_codes"])?;
        let mut scores = vec![0; 4096];
        finite_field.score_delta_into(
            &state,
            &finite_generate,
            &mut scores,
            &mut Default::default(),
        )?;
        replay_require(
            scores == f.u,
            "Cue saved U vector/frozen coefficient arithmetic differs",
        )?;
    }

    write(
        a,
        "cue-original-master-binding.json",
        &json!({
        "original_master_file":a.checkpoint.join("cue/cue-source-f32.bin"),
        "original_master_sha256":sha256_file(&a.checkpoint.join("cue/cue-source-f32.bin"))?,
        "shape":[960],"bytes":3840,"all960_original_bits_restored_before_graph":true,
        "original_source_binding":original.binding,"rebound_source_binding":finite.binding,
        "rebound_metadata":finite_carrier.metadata(),"rebound_packed_equal":true,
        "temporary_quarter_constructor":"overwritten in full by authenticated original float bits; never master authority"}),
    )?;
    fs::write(
        a.out.join("cue-initial-masters.f32le"),
        cue_parent
            .iter()
            .flat_map(|x| x.to_le_bytes())
            .collect::<Vec<_>>(),
    )?;
    let saved_bytes = frames.iter().try_fold(0u64, |s, f| {
        Ok::<_, Box<dyn std::error::Error>>(s + serde_json::to_vec(&f.native)?.len() as u64)
    })?;
    let numeric = saved_bytes * 3 + CACHE_LIMIT as u64 + 64 * 1024 * 1024;
    let projected = size(&a.checkpoint)? + saved_bytes + 48 * 1024 * 1024;
    replay_require(
        numeric <= 512 * 1024 * 1024 && projected + 1048576 < a.maximum_report_bytes,
        "Cue projected resource limit exceeded",
    )?;
    write(
        a,
        "resource-projection.json",
        &json!({"numerical_upper_bound_bytes":numeric,"report_projection_bytes":projected,
        "numeric_cap":536870912,"report_cap":a.maximum_report_bytes,"process_ram_cap":4294967296u64,"temporary_cap":536870912,"threads":2,"donor_cache_cap":CACHE_LIMIT,
        "scope":"saved nativeframes, rawvectors, donorcache and stagedpools; model/autodiff tensors charged separately to processRAM"}),
    )?;
    let mut cache = DonorCache::new(&finite);
    let mut reducer = NativeVocabularyActions::new(finite.integer.binding().clone(), &finite.exp)?;
    let mut pools = frames
        .iter()
        .enumerate()
        .map(|(i, f)| {
            score(
                i,
                f,
                &cue_parent,
                &cue_parent,
                &finite,
                &mut cache,
                &mut reducer,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let initial = objective(&frames, &pools)?;
    replay_require(
        initial["correct_reference_frames"] == 15
            && initial["combined"] == fr["candidate_objective"]["combined"],
        "Cue initial finite seed objective differs",
    )?;
    write(a, "initial-intermediate.json", &initial)?;
    let model =
        NativeGeometricGenerate::from_bytes(&original.generate, original.integer.binding())?;
    let g = GenerateLearningWeights::from_native(
        original.integer.binding().clone(),
        &model,
        &Device::Cpu,
    )?;
    restore(
        &a.checkpoint.join("generate-source"),
        &read(&a.checkpoint.join("generate-source/metadata.json"))?["parameters"],
        &g.parameters(),
        &Device::Cpu,
    )?;
    let gradients = gradient(
        a,
        start,
        &frames,
        &finite,
        &ActiveCredit::Cue(&cw),
        &g,
        &cue_parent,
        &pools,
        &mut cache,
    )?;
    drop(g);
    let order = ranking(&cue_parent, &gradients)?;
    write(a, "frozen-cue-ranking.json", &json!(order))?;
    let mut current = cue_parent.clone();
    let mut value = initial.clone();
    let mut trials = Vec::new();
    let mut accepted = 0;
    for (order_index, m) in order.iter().enumerate() {
        progress(a, start)?;
        let before = value.clone();
        if m.status != "eligible" {
            trials.push(json!({"order":order_index,"move":m,"status":m.status,"current":value,"staged":"NOT_RUN","native_effect":"no code displacement; current all18 results reused","original_gate":gate(&baseline,&value)?}));
            continue;
        }
        let mut proposed = current.clone();
        proposed[m.index] = m.master_after;
        let staged = frames
            .iter()
            .enumerate()
            .map(|(i, f)| {
                score(
                    i,
                    f,
                    &cue_parent,
                    &proposed,
                    &finite,
                    &mut cache,
                    &mut reducer,
                )
            })
            .collect::<Result<Vec<_>>>()?;
        let objective = objective(&frames, &staged)?;
        let previous = value["combined"]
            .as_f64()
            .ok_or_else(|| bad("Cue current CE missing"))?;
        let next = objective["combined"]
            .as_f64()
            .ok_or_else(|| bad("Cue staged CE missing"))?;
        let yes = next < previous - 1e-12 * (1. + previous.abs());
        trials.push(json!({"order":order_index,"move":m,"status":if yes{"accepted"}else{"rejected"},
            "before":before,"staged":objective,"native_all18_checked":true,"original_task_probability_improved":improved(&baseline,&objective)?,
            "original_gate":gate(&baseline,&objective)?}));
        if yes {
            current = proposed;
            pools = staged;
            value = objective;
            accepted += 1;
        }
    }
    write(
        a,
        "cue-construction.json",
        &json!({"coordinates":960,"accepted":accepted,"trials":trials,"final_objective":value,"initial":initial,
        "intermediate_selection":false,"cache_peak_bytes":cache.peak,"donor_recomputations":cache.calls,"cache_identity":{"source_binding":cache.source_binding,"generate_sha256":cache.generate_sha256,"bridge_sha256":cache.bridge_sha256},"cache_key":"frame index + physical ordinal inside immutable epoch; all8 cumulative codes and full occurrence identity authenticated","frozen_order":true,"revisited_coordinates":0}),
    )?;
    let params = cw.parameters();
    let before = np::snapshot(&params)?;
    drop(original_source);
    drop(sw);
    drop(actual_cue);
    let l = load_joint_continuation(a, &original, d)?;
    let frozen_generate = identities(&l.generate.parameters())?;
    let frozen_bridges = json!({"original":identities(&l.original_bridge.parameters())?,"marker":identities(&l.marker.parameters())?,
        "categorical":l.categorical.as_ref().map(|x|identities(&x.parameters())).transpose()?});
    let source = l.source.parameters();
    let source_before = np::snapshot(&source)?;
    let mut both = source.clone();
    both.extend(params.clone());
    let mut saved = source_before.clone();
    saved.extend(before.clone());
    let result = np::attempt_restored(&both, &saved, || {
        let mut changed = source_before.clone();
        let vector = changed
            .get_mut(CONTEXT)
            .ok_or_else(|| bad("Cue recorded Context family missing"))?;
        replay_require(
            vector[3610].to_bits() == (-0.1445201337337494f32).to_bits(),
            "Cue recorded Context fractional master differs",
        )?;
        vector[3610] = -0.5;
        np::restore(&source, &changed)?;
        let cue_values = BTreeMap::from([(NAME.to_string(), current.clone())]);
        np::restore(&params, &cue_values)?;
        replay_require(
            np::same_bits(&changed, &np::snapshot(&source)?)
                && np::same_bits(&cue_values, &np::snapshot(&params)?),
            "Cue unselected master bits changed",
        )?;
        replay_require(
            frozen_generate == identities(&l.generate.parameters())?
                && frozen_bridges
                    == json!({"original":identities(&l.original_bridge.parameters())?,"marker":identities(&l.marker.parameters())?,
                "categorical":l.categorical.as_ref().map(|x|identities(&x.parameters())).transpose()?}),
            "Cue frozen Generate/bridge master changed",
        )?;
        let (cp, field, receipt) = export_joint(a, &l, &cw, None, &original, d)?;
        replay_require(
            fs::read(a.out.join("checkpoint-0001/native/consumer/context-q4.bin"))?
                == fs::read(fcp.join("native/consumer/context-q4.bin"))?,
            "Cue recorded Context packed payload differs",
        )?;
        reload_candidate(a, &cp, &field, &frames, &pools)?;
        Ok(receipt)
    });
    replay_require(
        np::same_bits(&saved, &np::snapshot(&both)?),
        "Cue original master bits not restored",
    )?;
    let receipt = result?;
    let final_gate = gate(&baseline, &value)?;
    write(a, "final-objective.json", &value)?;
    Ok(
        json!({"schema":"uor-r4.context-cue-coadapt/1","status":"COMPLETED","mode":"context_cue_coadapt","policy":policy(),
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"baseline_objective":baseline,"initial_intermediate":initial,"candidate_objective":value,
        "final_gate":final_gate,"finite_joint_positive":final_gate["finite_joint_positive"],"selected_model":false,"useful_candidate":false,
        "candidate_receipt":receipt,"parent_master_bits_restored":true,"new_cue_gradients":1,"cue_backward_calls":18,"new_context_gradients":0,"context_backward_calls":0,"optimizer_updates":0,
        "cached_native_construction":true,"candidate_native_steps":18,"baseline_encoder_calls":0,"autoregressive_rollout":"NOT_RUN"}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn joint_word_gate_requires_word_and_combined_descent() -> Result<()> {
        let b = json!({"combined":4.,"task":2.});
        let mut v = json!({"combined":3.,"task":2.1,"correct_reference_frames":17,"all_phase_winners":true});
        assert_eq!(joint_gate(&b, &v)?["finite_joint_positive"], false);
        v["task"] = json!(1.5);
        assert_eq!(joint_gate(&b, &v)?["finite_joint_positive"], true);
        v["all_phase_winners"] = json!(false);
        assert_eq!(joint_gate(&b, &v)?["finite_joint_positive"], false);
        v["all_phase_winners"] = json!(true);
        v["correct_reference_frames"] = json!(16);
        assert_eq!(joint_gate(&b, &v)?["finite_joint_positive"], false);
        Ok(())
    }
    #[test]
    fn cue_fractional_ranking_and_saturation() -> Result<()> {
        let mut m = vec![0.; 960];
        let mut g = vec![0.; 960];
        m[4] = 0.139;
        g[4] = -2.;
        m[7] = 1.75;
        g[7] = -1.;
        let r = ranking(&m, &g)?;
        let x = r
            .iter()
            .find(|x| x.index == 4)
            .ok_or_else(|| bad("test row missing"))?;
        assert_eq!(x.before, 1);
        assert_eq!(x.after, 2);
        assert_eq!(x.master_after, 0.5);
        assert_eq!(x.actual_delta, 0.5 - f64::from(m[4]));
        assert_eq!(
            r.iter()
                .filter(|x| x.index == 7)
                .next()
                .map(|x| x.status.as_str()),
            Some("saturated")
        );
        Ok(())
    }
    #[test]
    fn cue_earliest_physical_tie_and_aliases() -> Result<()> {
        assert_eq!(earliest(&[7, 7, 6])?, 0);
        assert_eq!(earliest(&[7, 8, 8])?, 1);
        assert!(earliest(&[]).is_err());
        Ok(())
    }
    #[test]
    fn cue_final_gate_uses_original_parent_and_exact_ratio() -> Result<()> {
        let b = json!({"combined":4.,"task_target_mass":10,"task_total_mass":100});
        let negative = json!({"combined":4.1,"task_target_mass":11,"task_total_mass":100,"correct_reference_frames":17});
        assert_eq!(gate(&b, &negative)?["finite_joint_positive"], false);
        let lost = json!({"combined":3.9,"task_target_mass":11,"task_total_mass":100,"correct_reference_frames":16});
        assert_eq!(gate(&b, &lost)?["finite_joint_positive"], false);
        let pass = json!({"combined":3.9,"task_target_mass":11,"task_total_mass":100,"correct_reference_frames":17});
        assert_eq!(gate(&b, &pass)?["finite_joint_positive"], true);
        Ok(())
    }
    #[test]
    fn cue_and_context_late_error_restore_fractional_bits() -> Result<()> {
        let d = Device::Cpu;
        let c = Var::from_vec(vec![0.139f32, -0.0], 2, &d)?;
        let q = Var::from_vec(vec![-0.14452013f32], 1, &d)?;
        let p = BTreeMap::from([(NAME.into(), c), (CONTEXT.into(), q)]);
        let before = np::snapshot(&p)?;
        let r: Result<()> = np::attempt_restored(&p, &before, || {
            let mut changed = before.clone();
            changed.get_mut(NAME).ok_or_else(|| bad("test Cue"))?[0] = 0.5;
            changed
                .get_mut(CONTEXT)
                .ok_or_else(|| bad("test Context"))?[0] = -0.5;
            np::restore(&p, &changed)?;
            Err(bad("late failure"))
        });
        assert!(r.is_err());
        assert!(np::same_bits(&before, &np::snapshot(&p)?));
        Ok(())
    }
}
