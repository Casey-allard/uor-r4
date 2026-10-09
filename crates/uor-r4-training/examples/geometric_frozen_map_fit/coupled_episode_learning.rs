//! Offline coupled Prefix/Generate complete-episode credit; no serving changes.
use super::context_cue_coadapt as shared;
use super::generate_episode_learning as generate;
use super::native_proposals as np;
use super::prefix_fragment_learning as prefix;
use super::*;
use serde::Serialize;
use uor_r4_integer::geometric_vocabulary_actions::GeneratePatchCache;
use uor_r4_integer::h4_tables::H4Code;
use uor_r4_training::geometric_generate_learning::{
    vocabulary_log_mass_margin_with_credit, vocabulary_marginal_loss_with_credit,
};
use uor_r4_training::geometric_occurrence_consumer::source_realizer::{
    CueAngularWeights, PrefixAngularWeights,
};
#[path = "gradient_vector_prefix.rs"]
mod gradient_vector_prefix;
#[path = "protected_joint_vector.rs"]
mod protected_joint_vector;

const PREFIX: &str = "prefix.coefficients";
const GENERATE: &str = "generate.unary";
const COUNT: usize = 960;
const VOCAB: usize = 4096;
const GUARDS: usize = 380;
const UNION: usize = 391;
const CACHE_CAP: usize = 256 * 1024 * 1024;
const POST_CAP: usize = 16 * 1024 * 1024;
/// Offline Prefix donor adjoint. Native donor selection is unchanged.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum DonorCredit {
    #[default]
    StateTangent,
    FullPoolUtility,
}
impl DonorCredit {
    fn legacy(&self) -> bool {
        *self == Self::StateTangent
    }
}
fn validate_credit_recovery(mode: DonorCredit, retained: bool) -> Result<()> {
    replay_require(
        mode.legacy() || !retained,
        "full-pool donor credit cannot inherit state-tangent gradients/export",
    )
}
fn validate_inherited_credit(
    requested: DonorCredit,
    producer: DonorCredit,
    receipt: &Value,
) -> Result<()> {
    let recorded = if receipt["donor_credit"].is_null() {
        DonorCredit::StateTangent
    } else {
        serde_json::from_value(receipt["donor_credit"].clone())?
    };
    replay_require(
        requested == producer && recorded == requested,
        "retained donor-credit producer/config/gradient mode differs",
    )
}
fn credit_mode(a: &Args) -> DonorCredit {
    a.coupled_episode_learning
        .as_ref()
        .map(|c| c.donor_credit)
        .unwrap_or_default()
}
/// Offline construction policy; omitted fields retain the historical coordinate pass.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum PrefixTransaction {
    #[default]
    CoordinateAdjacent,
    GradientVectorPrefix,
    ProtectedJointVector,
}
impl PrefixTransaction {
    fn legacy(&self) -> bool {
        *self == Self::CoordinateAdjacent
    }
}
fn validate_transaction(
    mode: PrefixTransaction,
    credit: DonorCredit,
    retained: bool,
) -> Result<()> {
    replay_require(mode.legacy() || (credit == DonorCredit::FullPoolUtility && !retained),
        "gradient-vector Prefix requires fresh full-pool utility credit and forbids retained science")
}
fn transaction_mode(a: &Args) -> PrefixTransaction {
    a.coupled_episode_learning
        .as_ref()
        .map(|c| c.prefix_transaction)
        .unwrap_or_default()
}
fn validate_inherited_transaction(
    requested: PrefixTransaction,
    producer: PrefixTransaction,
    receipt: &Value,
) -> Result<()> {
    let recorded = if receipt["prefix_transaction"].is_null() {
        PrefixTransaction::CoordinateAdjacent
    } else {
        serde_json::from_value(receipt["prefix_transaction"].clone())?
    };
    replay_require(
        requested == producer && requested == recorded,
        "retained Prefix transaction identity differs",
    )
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Config {
    #[serde(default, skip_serializing_if = "PrefixTransaction::legacy")]
    pub prefix_transaction: PrefixTransaction,
    #[serde(default, skip_serializing_if = "DonorCredit::legacy")]
    pub donor_credit: DonorCredit,
    pub original_inputs: prefix::Config,
    #[serde(default)]
    pub retained_gradient: Option<RetainedGradient>,
    #[serde(default)]
    pub retained_export: Option<RetainedExport>,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RetainedGradient {
    pub retained_failed_root: PathBuf,
    pub retained_learning_runtime_root: PathBuf,
    pub retained_learning_observation_root: PathBuf,
    pub expected_report_sha256: String,
    pub expected_manifest_sha256: String,
    pub expected_learning_runtime_identity_sha256: String,
    pub expected_learning_observation_manifest_sha256: String,
    pub expected_learning_config_sha256: String,
    pub expected_learning_source_commit: String,
    pub expected_learning_binary_sha256: String,
    pub expected_gradient_receipt_sha256: String,
    pub expected_coordinate_order_sha256: String,
    pub expected_forward_parity_sha256: String,
    pub expected_partial_journal_sha256: String,
    pub inherited_partial_coordinate_records: usize,
    pub inherited_partial_alternatives: usize,
    pub inherited_partial_commits: usize,
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RetainedExport {
    pub retained_failed_root: PathBuf,
    pub retained_runtime_root: PathBuf,
    pub retained_observation_root: PathBuf,
    pub expected_report_sha256: String,
    pub expected_manifest_sha256: String,
    pub expected_runtime_identity_sha256: String,
    pub expected_observation_manifest_sha256: String,
    pub expected_config_sha256: String,
    pub expected_source_commit: String,
    pub expected_binary_sha256: String,
    pub expected_complete_journal_sha256: String,
    pub expected_final_objective_sha256: String,
    pub expected_final_prefix_master_sha256: String,
    pub expected_final_generate_master_sha256: String,
    pub inherited_gradients: RetainedGradient,
}
impl Config {
    fn retained_mode(&self) -> bool {
        self.retained_gradient.is_some() || self.retained_export.is_some()
    }
    fn gradient_authority(&self) -> Option<&RetainedGradient> {
        self.retained_gradient.as_ref().or_else(|| {
            self.retained_export
                .as_ref()
                .map(|e| &e.inherited_gradients)
        })
    }
    pub(super) fn input_roots(&self) -> Vec<&PathBuf> {
        let mut r = vec![
            &self.original_inputs.retained_intermediate_root,
            &self.original_inputs.retained_probe_root,
        ];
        if let Some(e) = &self.original_inputs.episode {
            r.extend([
                &e.typed_authority,
                &e.retained_projection.root,
                &e.retained_supplement_root,
            ]);
            r.extend(e.phases.iter().map(|p| &p.capture.root));
        }
        if let Some(g) = self.gradient_authority() {
            r.extend([
                &g.retained_failed_root,
                &g.retained_learning_runtime_root,
                &g.retained_learning_observation_root,
            ]);
        }
        if let Some(e) = &self.retained_export {
            r.extend([
                &e.retained_failed_root,
                &e.retained_runtime_root,
                &e.retained_observation_root,
            ]);
        }
        r
    }
}
pub(super) fn validate_settings(a: &Args) -> Result<()> {
    if let Some(c) = &a.coupled_episode_learning {
        validate_credit_recovery(c.donor_credit, c.retained_mode())?;
        validate_transaction(c.prefix_transaction, c.donor_credit, c.retained_mode())?;
        replay_require(
            a.mode == Mode::JointContinuation
                && a.updates == 1
                && a.loss_scope == LossScope::All
                && c.original_inputs.episode.is_some()
                && c.original_inputs.joint.is_none()
                && c.original_inputs.trajectory.is_none()
                && a.generate_episode_learning.is_none()
                && a.generate_episode_completion.is_none()
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
            "coupled episode mode/settings conflict",
        )?;
        if let Some(g) = c.gradient_authority() {
            replay_require(
                [
                    &g.expected_report_sha256,
                    &g.expected_manifest_sha256,
                    &g.expected_learning_runtime_identity_sha256,
                    &g.expected_learning_observation_manifest_sha256,
                    &g.expected_learning_config_sha256,
                    &g.expected_learning_binary_sha256,
                    &g.expected_gradient_receipt_sha256,
                    &g.expected_coordinate_order_sha256,
                    &g.expected_forward_parity_sha256,
                    &g.expected_partial_journal_sha256,
                ]
                .iter()
                .all(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
                    && g.expected_learning_source_commit.len() == 40
                    && g.expected_learning_source_commit
                        .bytes()
                        .all(|b| b.is_ascii_hexdigit())
                    && g.inherited_partial_coordinate_records <= 1920
                    && g.inherited_partial_alternatives <= 14400
                    && g.inherited_partial_commits <= g.inherited_partial_coordinate_records,
                "retained-gradient authority/configuration invalid",
            )?;
        }
        replay_require(
            !(c.retained_gradient.is_some() && c.retained_export.is_some()),
            "retained constructor/export modes conflict",
        )?;
        if let Some(e) = &c.retained_export {
            replay_require(
                [
                    &e.expected_report_sha256,
                    &e.expected_manifest_sha256,
                    &e.expected_runtime_identity_sha256,
                    &e.expected_observation_manifest_sha256,
                    &e.expected_config_sha256,
                    &e.expected_binary_sha256,
                    &e.expected_complete_journal_sha256,
                    &e.expected_final_objective_sha256,
                    &e.expected_final_prefix_master_sha256,
                    &e.expected_final_generate_master_sha256,
                ]
                .iter()
                .all(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit()))
                    && e.expected_source_commit.len() == 40
                    && e.expected_source_commit
                        .bytes()
                        .all(|b| b.is_ascii_hexdigit()),
                "retained export configuration identity invalid",
            )?;
        }
        let mut inherited = a.clone();
        inherited.coupled_episode_learning = None;
        inherited.prefix_fragment_learning = Some(c.original_inputs.clone());
        prefix::validate_settings(&inherited)?;
    }
    Ok(())
}
pub(super) fn policy() -> Value {
    json!({"schema":"uor-r4.coupled-episode-policy/1","active":[PREFIX,GENERATE],
        "original_selected_initializer":true,"weighted_roles":32,"physical_backward_calls":31,"prebackward_native_parity_graph_forwards":31,"gradient_graph_forwards":31,
        "task":"all15 each1/15","reference":"same17 each1/17","guards":380,"reload_union":391,
        "rank":"original finite f32 g/m widened f64, actual targetf32/4-master delta; total_cmp/family/index",
        "rank_scale":"native scales already in graph derivative; no additional4x",
        "prefix":"preferred adjacent direction once;zero/saturated NOOP preserves fractional bits",
        "generate":"all14 other legal codes at SAME incumbent coupled epoch; min feasible CURRENT CE/signedq tie",
        "maximum_alternatives":14400,"rerank":false,"revisit":false,"optimizer_updates":0,
        "gradient_surrogate":"direct Prefix gather and detached physical donor state utility plus factual Generate unary STE in same loss",
        "transaction":"strict current combined CE1e-10*(1+abs(current)) and17/380 winners; atomic all required rows",
        "final":"original normalized episode+combined CE descent/all15 inclEOS/17/380",
        "actual9":"immediate separate typed wholeanswer/EOS qualification only after positive sealed construction",
        "serving_changes":false})
}
fn policy_for(mode: DonorCredit) -> Value {
    let mut p = policy();
    if mode == DonorCredit::FullPoolUtility {
        p["donor_credit"] = json!(mode);
        p["gradient_surrogate"] = json!("direct Prefix gather plus detached complete forced-physical-donor native alias-pool CE softmax adjoint; factual Generate unary STE only; no state-tangent branch or duplicate indirect credit");
        p["donor_utility_scope"] = json!("offline forced-donor loss is a local surrogate, not a hard-argmax derivative; current factual Copy and frozen U preserved; no serving donor override");
    }
    p
}
fn policy_for_modes(credit: DonorCredit, transaction: PrefixTransaction) -> Value {
    let mut p = policy_for(credit);
    if !transaction.legacy() {
        p["prefix_transaction"] = json!(transaction);
        p["prefix"] = json!("four original-epoch gradient vectors radii1,2,4,7; eta=(f64(radius)*0.25)/maxabs; clamp originalm-eta*g; castf32 then nativequantize; unchangedcodes retain originalfractional bits; actualdelta dot<0");
        p["rank"] = json!("original mixed1920 frozen gradient order retained as authority; Generate960 relative order only after one selected Prefix vector; no rerank");
        p["maximum_alternatives"] = json!(13444);
        p["vector_selection"] = json!("minimum feasible original-epoch current CE; exact tie earlier radius; compact best only and one exact selected restage before atomic swap");
        p["affected_scope"] = json!("union of all changed Prefix keys over complete physical aliases, including cancelling net deltas");
    }
    if transaction == PrefixTransaction::ProtectedJointVector {
        p["prefix_transaction"] = json!(transaction);
        p["protected_direction"] = protected_joint_vector::policy();
        p["prefix"] = json!("joint protected1920 direction; originalm+eta*d; clamp/castf32/nativequantize; unchangedcodes retain originalfractional bits; actualdelta constraints and CE descent required");
        p["generate"] = json!("same joint transaction as Prefix; no follow-on coordinate pass");
        p["objective_graph_count_scope"] = json!("physical_backward_calls and prebackward_native_parity_graph_forwards and gradient_graph_forwards describe the31 CE graphs; protected380 and totals are separate");
        p["rank"] = json!("joint1920 objective gradient and original380 winner/strongest-other margin Jacobians; no coordinate sweep");
        p["maximum_alternatives"] = json!(4);
        p["protected_margin_backward_calls"] = json!(380);
        p["total_fresh_backward_calls"] = json!(411);
        p["total_training_graph_forwards"] = json!(822);
        p["affected_scope"] = json!("complete391 current candidate Prefix+Generate unary replacement; all physical aliases and original380 guards");
    }
    p
}
fn policy_for_args(a: &Args) -> Value {
    policy_for_modes(credit_mode(a), transaction_mode(a))
}
#[derive(Clone, Deserialize, Serialize)]
struct Coordinate {
    family: String,
    index: usize,
    gradient: f32,
    original_master: f32,
    rank_code: i8,
    original_code: i8,
    actual_master_delta: f64,
    priority: f64,
    status: String,
}
fn order(pm: &[f32], pg: &[f32], gm: &[f32], gg: &[f32]) -> Result<Vec<Coordinate>> {
    replay_require(
        [pm.len(), pg.len(), gm.len(), gg.len()]
            .iter()
            .all(|n| *n == COUNT),
        "coupled rank shape",
    )?;
    let mut rows = Vec::with_capacity(COUNT * 2);
    for (family, m, g) in [(PREFIX, pm, pg), (GENERATE, gm, gg)] {
        for i in 0..COUNT {
            replay_require(
                m[i].is_finite() && g[i].is_finite(),
                "coupled nonfinite rank",
            )?;
            let q = generate::code(m[i])?;
            let (rank, status) = if family == PREFIX {
                let d = if g[i] > 0. {
                    -1
                } else if g[i] < 0. {
                    1
                } else {
                    0
                };
                if d == 0 {
                    (q, "zero_gradient")
                } else if !(-7..=7).contains(&(q + d)) {
                    (q, "saturated")
                } else {
                    (q + d, "eligible")
                }
            } else {
                let mut codes = (-7i8..=7).filter(|x| *x != q).collect::<Vec<_>>();
                codes.sort_by(|x, y| {
                    let ux = f64::from(g[i]) * (f64::from(f32::from(*x) * 0.25) - f64::from(m[i]));
                    let uy = f64::from(g[i]) * (f64::from(f32::from(*y) * 0.25) - f64::from(m[i]));
                    ux.total_cmp(&uy).then(x.cmp(y))
                });
                (
                    *codes
                        .first()
                        .ok_or_else(|| bad("rank legal population empty"))?,
                    "all14",
                )
            };
            let target = if family == PREFIX && status != "eligible" {
                m[i]
            } else {
                f32::from(rank) * 0.25
            };
            let delta = f64::from(target) - f64::from(m[i]);
            rows.push(Coordinate {
                family: family.into(),
                index: i,
                gradient: g[i],
                original_master: m[i],
                rank_code: rank,
                original_code: q,
                actual_master_delta: delta,
                priority: f64::from(g[i]) * delta,
                status: status.into(),
            });
        }
    }
    rows.sort_by(|a, b| {
        a.priority
            .total_cmp(&b.priority)
            .then(a.family.cmp(&b.family))
            .then(a.index.cmp(&b.index))
    });
    Ok(rows)
}
struct RowIncidence {
    offsets: Vec<usize>,
    tokens: Vec<u16>,
}
impl RowIncidence {
    fn new(g: &NativeGeometricGenerate, post: &[H4Code], legal: &[u32]) -> Result<Self> {
        let mut counts = vec![0usize; COUNT];
        for &token in legal {
            let mut keys = vec![0u32; g.lanes() + g.energy().edges().len()];
            g.factor_incidence_into(post, token as usize, &mut keys, &mut Default::default())?;
            for &k in &keys[..8] {
                replay_require((k as usize) < COUNT, "unary key domain")?;
                counts[k as usize] += 1;
            }
        }
        let mut offsets = vec![0usize; COUNT + 1];
        for i in 0..COUNT {
            offsets[i + 1] = offsets[i] + counts[i];
        }
        let mut tokens = vec![0u16; offsets[COUNT]];
        let mut cursor = offsets[..COUNT].to_vec();
        for &token in legal {
            replay_require(token < VOCAB as u32, "legal token domain")?;
            let mut keys = vec![0u32; g.lanes() + g.energy().edges().len()];
            g.factor_incidence_into(post, token as usize, &mut keys, &mut Default::default())?;
            for &k in &keys[..8] {
                let k = k as usize;
                tokens[cursor[k]] = token as u16;
                cursor[k] += 1;
            }
        }
        Ok(Self { offsets, tokens })
    }
    fn matching(&self, k: usize) -> &[u16] {
        &self.tokens[self.offsets[k]..self.offsets[k + 1]]
    }
    fn bytes(&self) -> usize {
        self.offsets.capacity() * std::mem::size_of::<usize>() + self.tokens.capacity() * 2
    }
    fn digest(&self) -> Result<String> {
        Ok(sha256_bytes(&serde_json::to_vec(
            &json!({"offsets":self.offsets,"tokens":self.tokens}),
        )?))
    }
}
struct PostCache {
    binding: NativeArtifactBinding,
    bridge: String,
    values: BTreeMap<(usize, usize), (String, Vec<H4Code>)>,
    bytes: usize,
    peak: usize,
    calls: usize,
}
impl PostCache {
    fn new(p: &ContinuationParent) -> Self {
        Self {
            binding: p.binding.clone(),
            bridge: sha256_bytes(&p.bridge),
            values: BTreeMap::new(),
            bytes: 4096,
            peak: 4096,
            calls: 0,
        }
    }
    fn get(
        &mut self,
        row: usize,
        donor: usize,
        f: &shared::Frame,
        p: &ContinuationParent,
    ) -> Result<Vec<H4Code>> {
        replay_require(
            self.binding == p.binding
                && self.bridge == sha256_bytes(&p.bridge)
                && f.query.len() == 8
                && f.sources.iter().all(|x| x.len() == 8),
            "coupled immutable post epoch",
        )?;
        let source = f
            .sources
            .get(donor)
            .ok_or_else(|| bad("physical donor absent"))?;
        let witness = sha256_bytes(&serde_json::to_vec(
            &json!({"input":f.input,"position":f.position,"query":f.query.iter().map(|c|c.index()).collect::<Vec<_>>(),"source":source.iter().map(|c|c.index()).collect::<Vec<_>>(),"occurrence":f.native["bank_trace"]["cue_bank"]["bank"]["candidates"][donor]}),
        )?);
        if let Some((old, post)) = self.values.get(&(row, donor)) {
            replay_require(*old == witness, "immutable post identity collision")?;
            return Ok(post.clone());
        }
        let bridge = NativeGeometricReadStateBridge::from_bytes(&p.bridge, p.integer.binding())?;
        let mut post = vec![H4Code::IDENTITY; 8];
        let mut actions = post.clone();
        let mut scores = vec![0i64; 8 * 120];
        bridge.apply_into(
            &f.query,
            source,
            &mut post,
            &mut actions,
            &mut scores,
            &mut BridgeReadCounts::default(),
        )?;
        let charge = 1024 + post.capacity() * std::mem::size_of::<H4Code>();
        if self.bytes + charge > POST_CAP {
            self.values.clear();
            self.bytes = 4096;
        }
        self.bytes += charge;
        self.peak = self.peak.max(self.bytes);
        self.calls += 1;
        self.values.insert((row, donor), (witness, post.clone()));
        Ok(post)
    }
}
fn earliest(v: &[i64]) -> Result<usize> {
    let mut best = 0;
    let mut score = *v.first().ok_or_else(|| bad("empty physical Copy pool"))?;
    for (i, &s) in v.iter().enumerate().skip(1) {
        if s > score {
            best = i;
            score = s;
        }
    }
    Ok(best)
}
fn native_with_unary(p: &ContinuationParent, values: &[f32]) -> Result<NativeGeometricGenerate> {
    let original = NativeGeometricGenerate::from_bytes(&p.generate, p.integer.binding())?;
    let weights =
        GenerateLearningWeights::from_native(p.integer.binding().clone(), &original, &Device::Cpu)?;
    replay_require(values.len() == COUNT, "current unary shape")?;
    weights.unary.set(&Tensor::from_vec(
        values.to_vec(),
        weights.unary.shape(),
        &Device::Cpu,
    )?)?;
    Ok(weights.export_native()?)
}

struct DonorUtility {
    losses: Vec<f32>,
    receipt: Value,
}
fn forced_pool_loss(
    red: &mut NativeVocabularyActions,
    base_generate: &[i64],
    u: &[i64],
    ids: &[u32],
    copy: &[i64],
    target: u32,
) -> Result<(f64, u64, u64, u32)> {
    replay_require(
        base_generate.len() == u.len() && ids.len() == copy.len(),
        "donor utility score/physical alias shape differs",
    )?;
    let generate = base_generate
        .iter()
        .zip(u)
        .map(|(&g, &u)| {
            g.checked_add(u)
                .ok_or_else(|| bad("forced-donor Generate/U overflow"))
        })
        .collect::<Result<Vec<_>>>()?;
    let trace = red.reduce_trace(&generate, ids, copy)?;
    let mass = trace
        .token_masses
        .iter()
        .find(|m| m.token_id == target)
        .ok_or_else(|| bad("forced-donor target not admitted"))?
        .weight_q31;
    let total = trace.summary.total_weight_q31;
    replay_require(
        mass > 0 && total > 0 && mass <= total,
        "forced-donor native mass invalid",
    )?;
    let loss = -(mass as f64 / total as f64).ln();
    replay_require(
        loss.is_finite() && (loss as f32).is_finite(),
        "forced-donor CE not finite",
    )?;
    Ok((loss, mass, total, trace.summary.chosen_token_id))
}
fn donor_utility(
    f: &shared::Frame,
    pool: &shared::Pool,
    row: usize,
    p: &ContinuationParent,
    donor: &mut shared::DonorCache,
) -> Result<DonorUtility> {
    replay_require(
        !f.ids.is_empty()
            && f.ids.len() == pool.base_copy.len()
            && f.ids.len() == pool.copy.len()
            && pool.donor < f.ids.len(),
        "donor utility physical occurrence shape differs",
    )?;
    let mut red = NativeVocabularyActions::new(p.integer.binding().clone(), &p.exp)?;
    let mut native_losses = Vec::with_capacity(f.ids.len());
    let mut records = Vec::with_capacity(f.ids.len());
    for j in 0..f.ids.len() {
        let (post, raw_g) = donor.get(row, j, f, p)?;
        let (ce, mass, total, chosen) =
            forced_pool_loss(&mut red, &raw_g, &f.u, &f.ids, &pool.copy, f.target)?;
        if j == pool.donor {
            let generate = raw_g
                .iter()
                .zip(&f.u)
                .map(|(&g, &u)| {
                    g.checked_add(u)
                        .ok_or_else(|| bad("factual-donor overflow"))
                })
                .collect::<Result<Vec<_>>>()?;
            let factual = red.reduce_trace(&generate, &f.ids, &pool.copy)?;
            replay_require(
                post == pool.post
                    && generate == pool.generate
                    && serde_json::to_value(&factual)? == serde_json::to_value(&pool.trace)?,
                "forced factual donor complete pool differs before backward",
            )?;
        }
        native_losses.push(ce);
        records.push(json!({"physical_ordinal":j,"physical_candidate":f.native["bank_trace"]["cue_bank"]["bank"]["candidates"][j],
            "post_state":post.iter().map(|c|c.index()).collect::<Vec<_>>(),"base_generate_sha256":sha256_bytes(&serde_json::to_vec(&raw_g)?),
            "target_mass":mass,"total_mass":total,"chosen":chosen,"native_ce_f64":ce}));
    }
    let factual_ce = native_losses[pool.donor];
    let losses = native_losses
        .iter()
        .map(|ce| (ce - factual_ce) as f32)
        .collect::<Vec<_>>();
    replay_require(
        losses.iter().all(|x| x.is_finite()),
        "centered donor utility not finite",
    )?;
    for (record, &contrast) in records.iter_mut().zip(&losses) {
        record["adjoint_factual_centered_contrast_f32"] = json!(contrast);
    }
    let receipt = json!({"input_index":f.input,"position":f.position,"id":f.id,"actual_prefix_ids":f.prefix,
        "target_label_only":f.target,"factual_physical_donor":pool.donor,"physical_copy_aliases":f.ids.len(),
        "donor_fullpool_reductions":f.ids.len(),"factual_parity_reductions":1,"weight_applied_by_outer_loss_once":f.weight,
        "factual_native_ce_f64":factual_ce,"utility_conversion":"native u64 masses -> f64 CE, subtract factual-donor f64 CE, then explicit f32 detached contrast; no exact floating equivalence claimed",
        "frozen_copy_sha256":sha256_bytes(&serde_json::to_vec(&pool.copy)?),"frozen_U_sha256":sha256_bytes(&serde_json::to_vec(&f.u)?),"donors":records});
    // The cache cap and complete pregradient projection are admitted before this loop.
    replay_require(
        serde_json::to_vec(&receipt)?.len() <= 2 * 1024 * 1024,
        "donor utility per-frame receipt exceeds projected bound",
    )?;
    Ok(DonorUtility { losses, receipt })
}
fn zero_forward_donor_loss(raw: &Tensor, losses: &[f32]) -> Result<Tensor> {
    replay_require(
        raw.dims() == [losses.len()] && !losses.is_empty() && losses.iter().all(|v| v.is_finite()),
        "donor utility graph shape/nonfinite",
    )?;
    let weights = candle_nn::ops::softmax(raw, 0)?;
    let detached = Tensor::from_vec(losses.to_vec(), losses.len(), raw.device())?.detach();
    let expected = weights.mul(&detached)?.sum_all()?;
    Ok((&expected - expected.detach())?)
}
fn graph_loss(
    a: &Args,
    f: &shared::Frame,
    pool: &shared::Pool,
    row: usize,
    p: &ContinuationParent,
    pw: &PrefixAngularWeights,
    g: &GenerateLearningWeights,
    prepared: &uor_r4_training::geometric_generate_learning::PreparedGenerateLearning,
    donor: &mut shared::DonorCache,
    utility: Option<&DonorUtility>,
) -> Result<Tensor> {
    graph_quantity(a, f, pool, row, p, pw, g, prepared, donor, utility, None)
}
fn graph_quantity(
    a: &Args,
    f: &shared::Frame,
    pool: &shared::Pool,
    row: usize,
    p: &ContinuationParent,
    pw: &PrefixAngularWeights,
    g: &GenerateLearningWeights,
    prepared: &uor_r4_training::geometric_generate_learning::PreparedGenerateLearning,
    donor: &mut shared::DonorCache,
    utility: Option<&DonorUtility>,
    contrast: Option<(u32, u32)>,
) -> Result<Tensor> {
    let d = g.device();
    let credit = pw.coefficient_credit(
        f.prefix_trace
            .as_ref()
            .ok_or_else(|| bad("Prefix trace missing"))?,
        d,
    )?;
    let base = Tensor::from_vec(
        pool.base_copy
            .iter()
            .map(|v| (*v as f64 / 16777216.) as f32)
            .collect::<Vec<_>>(),
        f.ids.len(),
        d,
    )?;
    let delta = (&credit - credit.detach())?;
    let raw = (&base + &delta)?;
    let out = match credit_mode(a) {
        DonorCredit::StateTangent => {
            replay_require(
                utility.is_none(),
                "legacy tangent received full-pool donor credit",
            )?;
            let alternatives = (0..f.ids.len())
                .map(|j| donor.get(row, j, f, p).map(|x| x.0))
                .collect::<Result<Vec<_>>>()?;
            let state = shared::selector(&pool.post, &alternatives, &raw, d)?;
            g.forward_prepared_state_choices(prepared, &pool.post, &state)?
        }
        DonorCredit::FullPoolUtility => {
            g.forward_prepared_coefficients_only(prepared, &pool.post)?
        }
    };
    let numerical = out
        .scores_q24
        .iter()
        .zip(&f.u)
        .map(|(g, u)| g.checked_add(*u))
        .collect::<Option<Vec<_>>>();
    replay_require(
        numerical == Some(pool.generate.clone()),
        "coupled factual Generate/U anchor differs",
    )?;
    let ug = Tensor::from_vec(
        f.u.iter()
            .map(|v| (*v as f64 / 16777216.) as f32)
            .collect::<Vec<_>>(),
        VOCAB,
        d,
    )?;
    let hardg = Tensor::from_vec(
        pool.generate
            .iter()
            .map(|v| (*v as f64 / 16777216.) as f32)
            .collect::<Vec<_>>(),
        VOCAB,
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
            .map(|v| (*v as f64 / 16777216.) as f32)
            .collect::<Vec<_>>(),
        f.ids.len(),
        d,
    )?;
    let softc = (&raw + &uc)?;
    let joinedc = (&hardc + (&softc - softc.detach())?)?;
    let mut loss = if let Some((winner, rival)) = contrast {
        vocabulary_log_mass_margin_with_credit(
            &pool.trace,
            &joinedg,
            Some(&joinedc),
            winner,
            rival,
            a.credit.policy(),
        )?
    } else {
        vocabulary_marginal_loss_with_credit(
            &pool.trace,
            &joinedg,
            Some(&joinedc),
            f.target,
            a.credit.policy(),
        )?
    };
    if credit_mode(a) == DonorCredit::FullPoolUtility {
        let utility = utility.ok_or_else(|| bad("full-pool donor utility missing"))?;
        loss = (&loss + zero_forward_donor_loss(&raw, &utility.losses)?)?;
    }
    Ok((loss * if contrast.is_some() { 1.0 } else { f.weight })?)
}

fn gradients(
    a: &Args,
    start: Instant,
    p: &ContinuationParent,
    frames: &[shared::Frame],
    pools: &[shared::Pool],
    d: &Device,
    protected: Option<&(Vec<shared::Frame>, Vec<shared::Pool>, Value)>,
) -> Result<(Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>)> {
    replay_require(
        frames.len() == 31 && pools.len() == 31,
        "coupled physical graph population mismatch",
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
    let transported = native.compile_prefix_transport(&carrier, actual.native()?)?;
    let active = PrefixAngularWeights::from_native(
        &native,
        &a.checkpoint.join("native"),
        &a.checkpoint.join("cue"),
        &carrier,
        &transported,
        d,
    )?;
    let bits = np::snapshot(&actual.parameters())?;
    np::restore(&active.parameters(), &bits)?;
    replay_require(
        np::same_bits(&bits, &np::snapshot(&active.parameters())?)
            && active.packed_coefficients()?
                == fs::read(a.checkpoint.join("prefix/prefix-q4.bin"))?,
        "coupled Prefix actual masters/native lost",
    )?;
    let pm = bits
        .get(PREFIX)
        .ok_or_else(|| bad("Prefix original masters missing"))?
        .clone();
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
        "coupled Generate actual master/native differs",
    )?;
    let gm = np::snapshot(&g.parameters())?
        .remove(GENERATE)
        .ok_or_else(|| bad("Generate unary master missing"))?;
    let pv = active
        .parameters()
        .remove(PREFIX)
        .ok_or_else(|| bad("Prefix Var missing"))?;
    let prepared = g.prepare_native()?;
    let mut donor = shared::DonorCache::with_limit(p, 64 * 1024 * 1024)?;
    let utilities = if credit_mode(a) == DonorCredit::FullPoolUtility {
        frames
            .iter()
            .zip(pools)
            .enumerate()
            .map(|(i, (f, pool))| donor_utility(f, pool, i, p, &mut donor).map(Some))
            .collect::<Result<Vec<_>>>()?
    } else {
        (0..frames.len()).map(|_| None).collect::<Vec<_>>()
    };
    if credit_mode(a) == DonorCredit::FullPoolUtility {
        let actual = utilities
            .iter()
            .filter_map(|u| u.as_ref())
            .try_fold(0u64, |sum, u| {
                Ok::<_, Box<dyn std::error::Error>>(
                    sum + serde_json::to_vec(&u.receipt)?.len() as u64,
                )
            })?;
        replay_require(
            actual + 4096 <= donor_utility_bound(frames)?,
            "complete donor utility serialization exceeds pregradient bound",
        )?;
        write(
            a,
            "coupled-full-pool-donor-utilities.json",
            &json!({"donor_credit":credit_mode(a),
            "physical_frames":frames.len(),"all_before_any_backward":true,"new_context_encoder_calls":0,
            "forced_donor_fullpool_reductions":frames.iter().map(|f|f.ids.len()).sum::<usize>(),
            "factual_parity_reductions":frames.len(),"native_bridge_generate_cache_misses":donor.calls,"donor_cache_peak_bytes":donor.peak,"records":utilities.iter().filter_map(|u|u.as_ref().map(|u|&u.receipt)).collect::<Vec<_>>()}),
        )?;
    }
    for (i, (f, pool)) in frames.iter().zip(pools).enumerate() {
        replay_require(
            pool.donor == earliest(&pool.base_copy)?
                && f.native["bridge"]["selected_ordinal"].as_u64() == Some(pool.donor as u64)
                && json!(f.u) == f.native["continuation"]["delta_scores_q24"]
                && pool.copy
                    == pool
                        .base_copy
                        .iter()
                        .zip(&f.ids)
                        .map(|(base, id)| base.checked_add(f.u[*id as usize]))
                        .collect::<Option<Vec<_>>>()
                        .ok_or_else(|| bad("coupled Copy/U overflow"))?,
            "coupled physical donor/U/alias authority differs",
        )?;
        replay_require(
            json!(pool.generate) == f.native["generate_q24"]
                && json!(pool.copy) == f.native["copy_q24"]
                && json!(pool.post.iter().map(|c| c.index()).collect::<Vec<_>>())
                    == f.native["post_state"]
                && serde_json::to_value(&pool.trace)? == f.native["pool"],
            "coupled31 full native pool parity differs",
        )?;
        drop(graph_loss(
            a,
            f,
            pool,
            i,
            p,
            &active,
            &g,
            &prepared,
            &mut donor,
            utilities[i].as_ref(),
        )?);
    }
    write(
        a,
        "coupled-forward-parity.json",
        &json!({"all_before_any_backward":true,"physical_frames":31,
        "raw_G_Copy_U_donor_post_full_alias_pool":true,"device":"cuda","captured_objective_encoder_calls":0,"gradient_context_backward_calls":0}),
    )?;
    if let Some((guards, guard_pools, _)) = protected {
        protected_joint_vector::margin_pass(
            a,
            start,
            p,
            guards,
            guard_pools,
            &active,
            &g,
            &prepared,
            &pv,
            &mut donor,
            false,
        )?;
    }
    let mut ps = vec![0f32; COUNT];
    let mut gs = vec![0f32; COUNT];
    let mut terms = Vec::new();
    let mut graph_ce = 0.;
    for (i, (f, pool)) in frames.iter().zip(pools).enumerate() {
        shared::progress(a, start)?;
        let loss = graph_loss(
            a,
            f,
            pool,
            i,
            p,
            &active,
            &g,
            &prepared,
            &mut donor,
            utilities[i].as_ref(),
        )?;
        graph_ce += f64::from(loss.to_scalar::<f32>()?);
        let gr = loss.backward()?;
        let mut files = Vec::new();
        for (name, var, total) in [(PREFIX, &pv, &mut ps), (GENERATE, &g.unary, &mut gs)] {
            let values = gr
                .get(var.as_tensor())
                .ok_or_else(|| bad("coupled family gradient MISSING; no zero fill"))?
                .flatten_all()?
                .to_device(&Device::Cpu)?
                .to_vec1::<f32>()?;
            replay_require(
                values.len() == COUNT && values.iter().all(|x| x.is_finite()),
                "coupled gradient shape/nonfinite",
            )?;
            for (sum, &value) in total.iter_mut().zip(&values) {
                *sum += value;
            }
            let file = format!("coupled-gradient-{name}-term-{i:02}.f32le");
            let bytes = values
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect::<Vec<_>>();
            fs::write(a.out.join(&file), &bytes)?;
            files.push(json!({"family":name,"file":file,"shape":[960],"bytes":bytes.len(),"sha256":sha256_bytes(&bytes),
                "status":"PRESENT","missing_gradient_filled_zero":false,"all_zero":values.iter().all(|x|*x==0.),
                "l2_norm":values.iter().map(|v|f64::from(*v).powi(2)).sum::<f64>().sqrt()}));
        }
        terms.push(json!({"physical_index":i,"input_index":f.input,"position":f.position,"target":f.target,"weight":f.weight,"families":files}));
    }
    if let Some((guards, guard_pools, _)) = protected {
        protected_joint_vector::margin_pass(
            a,
            start,
            p,
            guards,
            guard_pools,
            &active,
            &g,
            &prepared,
            &pv,
            &mut donor,
            true,
        )?;
    }
    let mut aggregate = Vec::new();
    for (name, values, master) in [(PREFIX, &ps, &pm), (GENERATE, &gs, &gm)] {
        replay_require(
            values.iter().all(|x| x.is_finite()),
            "coupled aggregate nonfinite",
        )?;
        for (kind, data) in [("gradient", values), ("initial-master", master)] {
            let bytes = data
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect::<Vec<_>>();
            let file = format!("coupled-{name}-{kind}.f32le");
            fs::write(a.out.join(&file), &bytes)?;
            aggregate.push(json!({"family":name,"kind":kind,"file":file,"shape":[960],"bytes":bytes.len(),"sha256":sha256_bytes(&bytes)}));
        }
    }
    write(
        a,
        "coupled-gradient-receipt.json",
        &json!({"weighted_roles":32,"physical_backward_calls":31,"prebackward_native_parity_graph_forwards":31,"gradient_graph_forwards":31,"extracted_families":[PREFIX,GENERATE],
        "perterm":terms,"files":aggregate,"aggregate_sum":"ordered f32 sum of already weighted31physical gradients; no synthetic32arrays",
        "weighted_graph_ce":graph_ce,"surrogate":policy_for_args(a)["gradient_surrogate"],"context_gradient":"NOT_RUN","donor_cache_peak_bytes":donor.peak}),
    )?;
    if credit_mode(a) == DonorCredit::FullPoolUtility {
        let leaf = a.out.join("coupled-gradient-receipt.json");
        let mut receipt = read(&leaf)?;
        receipt["donor_credit"] = json!(credit_mode(a));
        if !transaction_mode(a).legacy() {
            receipt["prefix_transaction"] = json!(transaction_mode(a));
        }
        receipt["full_pool_utility_file"] = json!("coupled-full-pool-donor-utilities.json");
        receipt["full_pool_utility_sha256"] = json!(sha256_file(
            &a.out.join("coupled-full-pool-donor-utilities.json")
        )?);
        fs::write(&leaf, serde_json::to_vec(&receipt)?)?;
    }
    if protected.is_some() {
        let leaf = a.out.join("coupled-gradient-receipt.json");
        let mut receipt = read(&leaf)?;
        receipt["protected_margin_backward_calls"] = json!(380);
        receipt["total_fresh_backward_calls"] = json!(411);
        receipt["total_training_graph_forwards"] = json!(822);
        receipt["protected_margin_receipt_sha256"] =
            json!(sha256_file(&a.out.join("protected-margin-receipt.json"))?);
        fs::write(&leaf, serde_json::to_vec(&receipt)?)?;
    }
    Ok((pm, ps, gm, gs))
}

fn validate_inherited_leaf(leaf: &str) -> Result<()> {
    replay_require(
        matches!(
            Path::new(leaf).components().next(),
            Some(std::path::Component::Normal(_))
        ) && Path::new(leaf).components().count() == 1
            && !leaf.starts_with('.'),
        "inherited scientific output leaf invalid",
    )
}
fn copy_inherited_file(a: &Args, source: &Path, leaf: &str) -> Result<Value> {
    validate_inherited_leaf(leaf)?;
    let bytes = fs::read(source)?;
    let destination = a.out.join(leaf);
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&destination)?;
    use std::io::Write as _;
    file.write_all(&bytes)?;
    Ok(json!({"source_file":source,"file":leaf,"bytes":bytes.len(),"sha256":sha256_bytes(&bytes)}))
}
fn verify_inherited_copies(a: &Args, authority: &Value) -> Result<()> {
    for e in authority["copied_scientific_files"]
        .as_array()
        .ok_or_else(|| bad("inherited copy inventory absent"))?
    {
        let leaf = e["file"]
            .as_str()
            .ok_or_else(|| bad("inherited copy leaf absent"))?;
        replay_require(
            fs::metadata(a.out.join(leaf))?.len()
                == e["bytes"]
                    .as_u64()
                    .ok_or_else(|| bad("inherited copy bytes absent"))?
                && sha256_file(&a.out.join(leaf))?
                    == e["sha256"]
                        .as_str()
                        .ok_or_else(|| bad("inherited copy hash absent"))?,
            "inherited completed scientific bytes changed",
        )?;
    }
    Ok(())
}
fn retained_gradients(
    a: &Args,
    c: &Config,
    g: &RetainedGradient,
    frames: &[shared::Frame],
) -> Result<(Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Value)> {
    report_output::verify(&g.retained_failed_root)?;
    replay_require(
        sha256_file(&g.retained_failed_root.join("report.json"))? == g.expected_report_sha256
            && sha256_file(&g.retained_failed_root.join("manifest.json"))?
                == g.expected_manifest_sha256,
        "retained-gradient failed report/seal differs",
    )?;
    let failed = read(&g.retained_failed_root.join("report.json"))?;
    replay_require(
        failed["status"] == "FAILED" && failed["error"] == "guard Prefix trace missing",
        "retained-gradient restart does not match guard-incidence setup failure",
    )?;
    let old_config = read(&g.retained_failed_root.join("config.json"))?;
    replay_require(
        sha256_file(&g.retained_failed_root.join("config.json"))?
            == g.expected_learning_config_sha256,
        "retained-gradient learning configuration hash differs",
    )?;
    let old: Config = serde_json::from_value(old_config["coupled_episode_learning"].clone())?;
    replay_require(
        old.donor_credit == c.donor_credit
            && old.retained_gradient.is_none()
            && old.retained_export.is_none()
            && serde_json::to_value(&old.original_inputs)?
                == serde_json::to_value(&c.original_inputs)?
            && fs::canonicalize(&a.checkpoint)?
                == fs::canonicalize(Path::new(
                    old_config["checkpoint"]
                        .as_str()
                        .ok_or_else(|| bad("retained original checkpoint absent"))?,
                ))?,
        "retained-gradient original selected inputs differ",
    )?;
    report_output::verify(&g.retained_learning_observation_root)?;
    replay_require(
        sha256_file(&g.retained_learning_observation_root.join("manifest.json"))?
            == g.expected_learning_observation_manifest_sha256,
        "retained learning observation seal differs",
    )?;
    let execution = read(&g.retained_learning_observation_root.join("execution.json"))?;
    let launch = read(&g.retained_learning_observation_root.join("launch.json"))?;
    let attempt = read(&g.retained_failed_root.join("attempt.json"))?;
    let runtime_file = g
        .retained_learning_runtime_root
        .join("runtime-identity.json");
    replay_require(
        sha256_file(&runtime_file)? == g.expected_learning_runtime_identity_sha256,
        "retained learning runtime identity differs",
    )?;
    let runtime = read(&runtime_file)?;
    let learning_binding = read(&g.retained_failed_root.join("external-config-binding.json"))?;
    replay_require(
        learning_binding["sha256"] == g.expected_learning_config_sha256
            && learning_binding["attempt_argv"] == attempt["argv"],
        "retained external configuration binding differs",
    )?;
    replay_require(
        runtime["source_commit"] == g.expected_learning_source_commit
            && runtime["binary_sha256"] == g.expected_learning_binary_sha256
            && sha256_file(
                &g.retained_learning_runtime_root
                    .join("geometric-frozen-map-fit"),
            )? == g.expected_learning_binary_sha256
            && launch["argv"] == attempt["argv"]
            && launch["pid"] == attempt["pid"]
            && launch["started_utc"] == execution["started_utc"]
            && execution["exit_code"] == 1
            && launch["binary_sha256"] == g.expected_learning_binary_sha256
            && execution["binary_sha256"] == g.expected_learning_binary_sha256
            && launch["config_sha256"] == g.expected_learning_config_sha256
            && execution["config_sha256"] == g.expected_learning_config_sha256,
        "retained actual learning source/config/binary/launch differs",
    )?;
    let forward = read(&g.retained_failed_root.join("coupled-forward-parity.json"))?;
    replay_require(
        sha256_file(&g.retained_failed_root.join("coupled-forward-parity.json"))?
            == g.expected_forward_parity_sha256
            && forward["all_before_any_backward"] == true
            && forward["physical_frames"] == 31
            && forward["raw_G_Copy_U_donor_post_full_alias_pool"] == true,
        "retained factual prebackward parity authority incomplete",
    )?;
    let receipt_file = g.retained_failed_root.join("coupled-gradient-receipt.json");
    replay_require(
        sha256_file(&receipt_file)? == g.expected_gradient_receipt_sha256,
        "retained gradient receipt identity differs",
    )?;
    let receipt = read(&receipt_file)?;
    validate_inherited_credit(c.donor_credit, old.donor_credit, &receipt)?;
    validate_inherited_transaction(c.prefix_transaction, old.prefix_transaction, &receipt)?;
    replay_require(
        receipt["physical_backward_calls"] == 31
            && receipt["weighted_roles"] == 32
            && receipt["extracted_families"] == json!([PREFIX, GENERATE])
            && frames.len() == 31,
        "retained gradient family/term population differs",
    )?;
    let terms = receipt["perterm"]
        .as_array()
        .ok_or_else(|| bad("retained perterm gradients absent"))?;
    replay_require(
        terms.len() == 31,
        "retained weighted physical terms incomplete",
    )?;
    let mut ps = vec![0f32; COUNT];
    let mut gs = vec![0f32; COUNT];
    let mut copied = Vec::new();
    let mut seen = BTreeSet::new();
    for (i, (term, f)) in terms.iter().zip(frames).enumerate() {
        replay_require(
            term["physical_index"] == i
                && term["input_index"] == f.input
                && term["position"] == f.position
                && term["target"] == f.target
                && term["weight"].as_f64() == Some(f.weight),
            "retained weighted term identity differs",
        )?;
        let families = term["families"]
            .as_array()
            .ok_or_else(|| bad("retained term family arrays absent"))?;
        replay_require(families.len() == 2, "retained term both families missing")?;
        for (name, sum) in [(PREFIX, &mut ps), (GENERATE, &mut gs)] {
            let e = families
                .iter()
                .find(|e| e["family"] == name)
                .ok_or_else(|| bad("retained term family missing"))?;
            let leaf = e["file"]
                .as_str()
                .ok_or_else(|| bad("retained raw gradient file missing"))?;
            replay_require(
                e["shape"] == json!([960])
                    && e["bytes"] == 3840
                    && e["status"] == "PRESENT"
                    && e["missing_gradient_filled_zero"] == false
                    && seen.insert(leaf.to_owned()),
                "retained raw family receipt invalid",
            )?;
            validate_inherited_leaf(leaf)?;
            let source = g.retained_failed_root.join(leaf);
            replay_require(
                sha256_file(&source)?
                    == e["sha256"]
                        .as_str()
                        .ok_or_else(|| bad("raw gradient hash absent"))?,
                "retained raw gradient hash differs",
            )?;
            let values = shared::floats(&source)?;
            replay_require(
                values.len() == COUNT && values.iter().all(|v| v.is_finite()),
                "retained raw gradient shape/nonfinite",
            )?;
            replay_require(
                e["all_zero"] == values.iter().all(|v| *v == 0.),
                "retained zero gradient presence differs",
            )?;
            for (a, b) in sum.iter_mut().zip(values) {
                *a += b;
            }
            copied.push(copy_inherited_file(a, &source, leaf)?);
        }
    }
    let files = receipt["files"]
        .as_array()
        .ok_or_else(|| bad("retained aggregate/master files absent"))?;
    replay_require(
        files.len() == 4,
        "retained aggregate/master inventory differs",
    )?;
    let mut loaded: BTreeMap<(String, String), Vec<f32>> = BTreeMap::new();
    for e in files {
        let family = e["family"]
            .as_str()
            .ok_or_else(|| bad("retained aggregate family absent"))?;
        let kind = e["kind"]
            .as_str()
            .ok_or_else(|| bad("retained aggregate kind absent"))?;
        let leaf = e["file"]
            .as_str()
            .ok_or_else(|| bad("retained aggregate filename absent"))?;
        replay_require(
            [PREFIX, GENERATE].contains(&family)
                && ["gradient", "initial-master"].contains(&kind)
                && e["shape"] == json!([960])
                && e["bytes"] == 3840
                && seen.insert(leaf.to_owned()),
            "retained aggregate/master receipt invalid",
        )?;
        validate_inherited_leaf(leaf)?;
        let source = g.retained_failed_root.join(leaf);
        replay_require(
            sha256_file(&source)?
                == e["sha256"]
                    .as_str()
                    .ok_or_else(|| bad("aggregate hash absent"))?,
            "retained aggregate/master hash differs",
        )?;
        let values = shared::floats(&source)?;
        replay_require(
            values.len() == COUNT && values.iter().all(|v| v.is_finite()),
            "retained aggregate/master shape/nonfinite",
        )?;
        replay_require(
            loaded
                .insert((family.into(), kind.into()), values)
                .is_none(),
            "retained duplicate aggregate family",
        )?;
        copied.push(copy_inherited_file(a, &source, leaf)?);
    }
    let take = |family: &str, kind: &str| {
        loaded
            .get(&(family.to_owned(), kind.to_owned()))
            .cloned()
            .ok_or_else(|| bad("retained aggregate/master family absent"))
    };
    let pm = take(PREFIX, "initial-master")?;
    let pg = take(PREFIX, "gradient")?;
    let gm = take(GENERATE, "initial-master")?;
    let gg = take(GENERATE, "gradient")?;
    replay_require(
        pg.iter().zip(&ps).all(|(a, b)| a.to_bits() == b.to_bits())
            && gg.iter().zip(&gs).all(|(a, b)| a.to_bits() == b.to_bits()),
        "retained ordered f32 perterm sums differ",
    )?;
    let pbytes = fs::read(a.checkpoint.join("prefix/prefix-source-f32.bin"))?;
    let gbytes = fs::read(a.checkpoint.join("generate-source/generate.unary.f32le"))?;
    replay_require(
        pm.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>() == pbytes
            && gm.iter().flat_map(|v| v.to_le_bytes()).collect::<Vec<_>>() == gbytes,
        "retained gradient original fractional master bits differ",
    )?;
    for (leaf, pin) in [
        (
            "coupled-gradient-receipt.json",
            &g.expected_gradient_receipt_sha256,
        ),
        (
            "coupled-forward-parity.json",
            &g.expected_forward_parity_sha256,
        ),
        (
            "coupled-coordinate-order.json",
            &g.expected_coordinate_order_sha256,
        ),
    ] {
        let source = g.retained_failed_root.join(leaf);
        replay_require(
            sha256_file(&source)? == *pin,
            "retained completed scientific file differs",
        )?;
        copied.push(copy_inherited_file(a, &source, leaf)?);
    }
    // Existing normalized inputs were reconstructed from immutable authorities,
    // not an encoder. They must stay byte-identical to the completed learning inputs.
    for entry in fs::read_dir(&g.retained_failed_root)? {
        let entry = entry?;
        let leaf = entry.file_name().to_string_lossy().into_owned();
        if leaf.starts_with("original-") && leaf.ends_with(".json") {
            replay_require(
                fs::read(entry.path())? == fs::read(a.out.join(&leaf))?,
                "retained normalized original witness differs",
            )?;
            copied.push(
                json!({"source_file":entry.path(),"file":leaf,"bytes":entry.metadata()?.len(),
                "sha256":sha256_file(&entry.path())?}),
            );
        }
    }
    let partial = g.retained_failed_root.join("coupled-construction.json");
    replay_require(
        sha256_file(&partial)? == g.expected_partial_journal_sha256,
        "retained incomplete construction journal differs",
    )?;
    copied.push(copy_inherited_file(
        a,
        &partial,
        "inherited-partial-construction.json",
    )?);
    for (source, leaf) in [
        (
            g.retained_failed_root.join("report.json"),
            "inherited-learning-report.json",
        ),
        (
            g.retained_failed_root.join("manifest.json"),
            "inherited-learning-manifest.json",
        ),
        (
            g.retained_failed_root.join("config.json"),
            "inherited-learning-config.json",
        ),
        (
            g.retained_failed_root.join("external-config-binding.json"),
            "inherited-learning-external-config-binding.json",
        ),
        (
            g.retained_learning_observation_root.join("launch.json"),
            "inherited-learning-launch.json",
        ),
        (
            g.retained_learning_observation_root.join("execution.json"),
            "inherited-learning-execution.json",
        ),
    ] {
        copied.push(copy_inherited_file(a, &source, leaf)?);
    }
    let authority = json!({"root":g.retained_failed_root,"report_sha256":g.expected_report_sha256,
        "manifest_sha256":g.expected_manifest_sha256,"source_commit":g.expected_learning_source_commit,
        "config_sha256":g.expected_learning_config_sha256,"binary_sha256":g.expected_learning_binary_sha256,
        "runtime_identity":runtime,"recorded_launch":launch,"recorded_execution":execution,
        "observations":g.retained_learning_observation_root,
        "observation_manifest_sha256":g.expected_learning_observation_manifest_sha256,
        "gradient_receipt_sha256":g.expected_gradient_receipt_sha256,"order_sha256":g.expected_coordinate_order_sha256,
        "raw_family_files":62,"physical_backward_calls":31,"new_training_graph_forwards":0,"new_backward_calls":0,
        "new_gradient_context_encoder_calls":0,"copied_scientific_files":copied,
        "original_partial":{"coordinate_records":g.inherited_partial_coordinate_records,
            "alternatives":g.inherited_partial_alternatives,"commits":g.inherited_partial_commits,
            "raw_journal_sha256":g.expected_partial_journal_sha256,"format":"INCOMPLETE_JSON",
            "scope":"retained and charged owner-authenticated partial counters; not parsed as a resumable incumbent"},
        "constructor_restart_from_original":true,"order_scope":"original frozen order verified from saved f32 gradients/master bits; no new gradient/rank selection",
        "completion_source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),
        "completion_attempt":read(&a.out.join("attempt.json"))?,
        "completion_external_config_binding":read(&a.out.join("external-config-binding.json"))?});
    verify_inherited_copies(a, &authority)?;
    write(a, "inherited-gradient-authority.json", &authority)?;
    Ok((pm, pg, gm, gg, authority))
}

#[derive(Deserialize)]
struct SavedSelected {
    status: String,
    code: Option<i8>,
}
#[derive(Deserialize)]
struct SavedCoordinate {
    family: String,
    index: usize,
    order: usize,
    original_master_bits: u32,
    incumbent_epoch: u64,
    epoch_after: u64,
    selected: SavedSelected,
}
#[derive(Deserialize)]
struct FinishedConstruction {
    coordinate_records: Vec<SavedCoordinate>,
    summary: Value,
}
fn restored_final_masters(
    records: &[SavedCoordinate],
    summary: &Value,
    pm: &[f32],
    gm: &[f32],
) -> Result<(Vec<f32>, Vec<f32>)> {
    replay_require(
        records.len() == 1920 && pm.len() == COUNT && gm.len() == COUNT,
        "retained complete coordinate/master population differs",
    )?;
    let mut prefix = pm.to_vec();
    let mut unary = gm.to_vec();
    let mut seen = BTreeSet::new();
    let mut epoch = 0u64;
    let mut pc = 0usize;
    let mut gc = 0usize;
    for (ordinal, r) in records.iter().enumerate() {
        replay_require(
            r.order == ordinal
                && r.index < COUNT
                && seen.insert((r.family.clone(), r.index))
                && r.incumbent_epoch == epoch,
            "retained committed coordinate/epoch invalid",
        )?;
        let (original, current, count) = if r.family == PREFIX {
            (pm, &mut prefix, &mut pc)
        } else if r.family == GENERATE {
            (gm, &mut unary, &mut gc)
        } else {
            return Err(bad("retained unknown active family"));
        };
        replay_require(
            r.original_master_bits == original[r.index].to_bits(),
            "retained original fractional master bits invalid",
        )?;
        if r.selected.status == "committed" {
            let q = r
                .selected
                .code
                .ok_or_else(|| bad("retained committed code absent"))?;
            replay_require(
                (-7..=7).contains(&q),
                "retained committed native code invalid",
            )?;
            current[r.index] = f32::from(q) * 0.25;
            epoch += 1;
            *count += 1;
        } else {
            replay_require(
                (r.selected.status == "unchanged"
                    || (r.selected.status == "noop" && r.family == PREFIX))
                    && r.selected.code.is_none(),
                "retained unknown noncommitted selection",
            )?;
        }
        replay_require(
            r.epoch_after == epoch,
            "retained accepted epoch progression differs",
        )?;
    }
    replay_require(
        summary["coordinates"] == 1920
            && summary["accepted_epoch"] == epoch
            && summary["accepted_prefix"] == pc
            && summary["accepted_generate"] == gc
            && summary["evaluated_alternatives"]
                .as_u64()
                .is_some_and(|n| n <= 14400),
        "retained completed summary/counters differ",
    )?;
    Ok((prefix, unary))
}
fn recover_export_state(
    a: &Args,
    c: &Config,
    e: &RetainedExport,
    pm: &[f32],
    gm: &[f32],
    p: &ContinuationParent,
) -> Result<(Vec<f32>, Vec<f32>, Value, Value)> {
    report_output::verify(&e.retained_failed_root)?;
    replay_require(
        sha256_file(&e.retained_failed_root.join("report.json"))? == e.expected_report_sha256
            && sha256_file(&e.retained_failed_root.join("manifest.json"))?
                == e.expected_manifest_sha256,
        "retained export failure report/seal differs",
    )?;
    let failed = read(&e.retained_failed_root.join("report.json"))?;
    replay_require(failed["status"]=="FAILED"
        &&failed["error"]=="invalid reference request: Binding(\"continuation Generate dimensions/metadata\")",
        "retained export does not match frozen U/new Generate initialization failure")?;
    replay_require(
        sha256_file(&e.retained_failed_root.join("config.json"))? == e.expected_config_sha256,
        "retained completed constructor config differs",
    )?;
    let old_config = read(&e.retained_failed_root.join("config.json"))?;
    let old: Config = serde_json::from_value(old_config["coupled_episode_learning"].clone())?;
    validate_inherited_credit(
        c.donor_credit,
        old.donor_credit,
        &read(&e.retained_failed_root.join("coupled-gradient-receipt.json"))?,
    )?;
    validate_inherited_transaction(
        c.prefix_transaction,
        old.prefix_transaction,
        &read(&e.retained_failed_root.join("coupled-gradient-receipt.json"))?,
    )?;
    replay_require(
        old.donor_credit == c.donor_credit
            && old.retained_export.is_none()
            && serde_json::to_value(&old.original_inputs)?
                == serde_json::to_value(&c.original_inputs)?
            && serde_json::to_value(&old.retained_gradient)?
                == serde_json::to_value(Some(&e.inherited_gradients))?
            && old_config["checkpoint"] == json!(a.checkpoint),
        "retained completed constructor input/gradient authority differs",
    )?;
    report_output::verify(&e.retained_observation_root)?;
    replay_require(
        sha256_file(&e.retained_observation_root.join("manifest.json"))?
            == e.expected_observation_manifest_sha256
            && sha256_file(&e.retained_runtime_root.join("runtime-identity.json"))?
                == e.expected_runtime_identity_sha256
            && sha256_file(&e.retained_runtime_root.join("geometric-frozen-map-fit"))?
                == e.expected_binary_sha256,
        "retained constructor runtime/observation identity differs",
    )?;
    let runtime = read(&e.retained_runtime_root.join("runtime-identity.json"))?;
    let launch = read(&e.retained_observation_root.join("launch.json"))?;
    let execution = read(&e.retained_observation_root.join("execution.json"))?;
    let attempt = read(&e.retained_failed_root.join("attempt.json"))?;
    let binding = read(&e.retained_failed_root.join("external-config-binding.json"))?;
    replay_require(
        runtime["source_commit"] == e.expected_source_commit
            && runtime["config_sha256"] == e.expected_config_sha256
            && runtime["binary_sha256"] == e.expected_binary_sha256
            && launch["argv"] == attempt["argv"]
            && binding["attempt_argv"] == attempt["argv"]
            && launch["pid"] == attempt["pid"]
            && launch["started_utc"] == execution["started_utc"]
            && execution["exit_code"] == 1
            && launch["binary_sha256"] == e.expected_binary_sha256
            && execution["binary_sha256"] == e.expected_binary_sha256
            && launch["config_sha256"] == e.expected_config_sha256
            && execution["config_sha256"] == e.expected_config_sha256
            && binding["sha256"] == e.expected_config_sha256,
        "retained actual constructor producer binding differs",
    )?;
    let journal = e.retained_failed_root.join("coupled-construction.json");
    replay_require(
        sha256_file(&journal)? == e.expected_complete_journal_sha256
            && sha256_file(&e.retained_failed_root.join("final-objective.json"))?
                == e.expected_final_objective_sha256,
        "retained completed constructor journal/objective differs",
    )?;
    // Deserialize only coordinate commitments and compact summary. Alternatives
    // remain authenticated bytes; no search/ordering/gradient is invoked.
    let completed: FinishedConstruction =
        serde_json::from_reader(std::io::BufReader::new(fs::File::open(&journal)?))?;
    let (prefix, unary) =
        restored_final_masters(&completed.coordinate_records, &completed.summary, pm, gm)?;
    replay_require(
        completed.summary["final"] == read(&e.retained_failed_root.join("final-objective.json"))?,
        "retained final objective and complete journal disagree",
    )?;
    let partial = e.retained_failed_root.join("checkpoint-0001");
    let pb = fs::read(partial.join("prefix/prefix-source-f32.bin"))?;
    let gb = fs::read(partial.join("generate-source/generate.unary.f32le"))?;
    replay_require(
        sha256_bytes(&pb) == e.expected_final_prefix_master_sha256
            && sha256_bytes(&gb) == e.expected_final_generate_master_sha256
            && prefix
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect::<Vec<_>>()
                == pb
            && unary
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect::<Vec<_>>()
                == gb,
        "retained completed dual master bytes differ from committed records",
    )?;
    let g = native_with_unary(p, &unary)?;
    replay_require(
        g.to_bytes()? == fs::read(partial.join("generate.bin"))?,
        "retained final Generate packing/frozen families differ",
    )?;
    let packed = (0..COUNT)
        .step_by(2)
        .map(|i| {
            let lo = shared::native_code(prefix[i])?;
            let hi = shared::native_code(prefix[i + 1])?;
            Ok::<u8, Box<dyn std::error::Error>>(((lo as u8) & 15) | (((hi as u8) & 15) << 4))
        })
        .collect::<Result<Vec<_>>>()?;
    replay_require(
        packed == fs::read(partial.join("prefix/prefix-q4.bin"))?,
        "retained final Prefix native packing differs",
    )?;
    generate::exact_frozen_directory(&a.checkpoint.join("source"), &partial.join("source"))?;
    for leaf in [
        "read-state-bridge-categorical.bin",
        "read-state-bridge.bin",
        "cue/cue-q4.bin",
        "cue/cue-joint-q4.bin",
        "cue/cue-source-f32.bin",
    ] {
        replay_require(
            fs::read(a.checkpoint.join(leaf))? == fs::read(partial.join(leaf))?,
            "retained completed frozen payload/master differs",
        )?;
    }
    for family in ["cue", "generate-source"] {
        let original = a.checkpoint.join(family);
        let candidate = partial.join(family);
        let old_files = fs::read_dir(&original)?
            .map(|e| e.map(|e| e.file_name()))
            .collect::<std::result::Result<BTreeSet<_>, _>>()?;
        let new_files = fs::read_dir(&candidate)?
            .map(|e| e.map(|e| e.file_name()))
            .collect::<std::result::Result<BTreeSet<_>, _>>()?;
        replay_require(
            old_files == new_files,
            "retained partial sidecar file population differs",
        )?;
        for leaf in old_files {
            let name = leaf
                .to_str()
                .ok_or_else(|| bad("retained sidecar filename invalid"))?;
            if family == "generate-source"
                && ["metadata.json", "generate.unary.f32le"].contains(&name)
            {
                continue;
            }
            if family == "cue" && ["metadata.json", "native-metadata.json"].contains(&name) {
                replay_require(
                    read(&original.join(&leaf))? == read(&candidate.join(&leaf))?,
                    "retained frozen Cue metadata semantically differs",
                )?;
            } else {
                replay_require(
                    fs::read(original.join(&leaf))? == fs::read(candidate.join(&leaf))?,
                    "retained frozen sidecar fractional/native payload differs",
                )?;
            }
        }
    }
    let mut copied = Vec::new();
    for entry in fs::read_dir(&e.retained_failed_root)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let leaf = entry.file_name().to_string_lossy().into_owned();
        let destination = if [
            "attempt.json",
            "config.json",
            "report.json",
            "manifest.json",
            "external-config-binding.json",
            "inherited-gradient-authority.json",
            "coupled-pregradient-resource-projection.json",
            "coupled-constructor-resource-projection.json",
        ]
        .contains(&leaf.as_str())
        {
            format!("completed-constructor-{leaf}")
        } else {
            leaf.clone()
        };
        if a.out.join(&destination).exists() {
            replay_require(
                fs::read(entry.path())? == fs::read(a.out.join(&destination))?,
                "copied completed scientific file was normalized differently",
            )?;
            copied.push(json!({"source_file":entry.path(),"file":destination,"bytes":entry.metadata()?.len(),
                "sha256":sha256_file(&entry.path())?}));
        } else {
            copied.push(copy_inherited_file(a, &entry.path(), &destination)?);
        }
    }
    copied.push(copy_inherited_file(
        a,
        &partial.join("receipt.json"),
        "inherited-partial-checkpoint-receipt.json",
    )?);
    let authority = json!({"root":e.retained_failed_root,"report_sha256":e.expected_report_sha256,
        "manifest_sha256":e.expected_manifest_sha256,"source_commit":e.expected_source_commit,
        "config_sha256":e.expected_config_sha256,"binary_sha256":e.expected_binary_sha256,
        "runtime_identity":runtime,"recorded_launch":launch,"recorded_execution":execution,
        "journal_sha256":e.expected_complete_journal_sha256,"final_objective_sha256":e.expected_final_objective_sha256,
        "final_prefix_master_sha256":e.expected_final_prefix_master_sha256,
        "final_generate_master_sha256":e.expected_final_generate_master_sha256,
        "copied_scientific_files":copied,"inherited_constructor_calls":1,
        "inherited_proposals":completed.summary["evaluated_alternatives"],"new_constructor_calls":0,
        "new_proposals":0,"new_order_selection_calls":0,"new_training_graph_forwards":0,"new_backward_calls":0,
        "partial_checkpoint_receipt_scope":"inherited generic writer metadata, not actual completed391 reload or new optimizer authority",
        "completion_source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),
        "completion_attempt":read(&a.out.join("attempt.json"))?,
        "completion_external_config_binding":read(&a.out.join("external-config-binding.json"))?});
    verify_inherited_copies(a, &authority)?;
    write(a, "inherited-construction-authority.json", &authority)?;
    Ok((prefix, unary, completed.summary, authority))
}

struct Replacement {
    cache: GeneratePatchCache,
    post: Vec<H4Code>,
    donor: usize,
    incidence: Option<RowIncidence>,
}
fn replacement(
    row: usize,
    f: &shared::Frame,
    parent_prefix: &[f32],
    prefix: &[f32],
    g: &NativeGeometricGenerate,
    p: &ContinuationParent,
    postcache: &mut PostCache,
    red: &mut NativeVocabularyActions,
    legal: &[u32],
    oldpost: &[H4Code],
) -> Result<Replacement> {
    let keys = validated_prefix_keys(f.ids.len(), &f.cue_keys)?;
    let base = shared::staged_base(&f.base_copy, keys, parent_prefix, prefix)?;
    let donor = earliest(&base)?;
    let post = postcache.get(row, donor, f, p)?;
    let mut generate = vec![0i64; VOCAB];
    g.score_into(&post, &mut generate, &mut GenerateReadCounts::default())?;
    for (x, &u) in generate.iter_mut().zip(&f.u) {
        *x = x
            .checked_add(u)
            .ok_or_else(|| bad("coupled G+U overflow"))?;
    }
    let copy = base
        .iter()
        .zip(&f.ids)
        .map(|(&x, &id)| {
            x.checked_add(f.u[id as usize])
                .ok_or_else(|| bad("coupled Copy+U overflow"))
        })
        .collect::<Result<Vec<_>>>()?;
    let cache = red.prepare_generate_patch_cache(generate, f.ids.clone(), copy)?;
    let incidence = if post == oldpost {
        None
    } else {
        Some(RowIncidence::new(g, &post, legal)?)
    };
    Ok(Replacement {
        cache,
        post,
        donor,
        incidence,
    })
}
fn replacement_objective(
    frames: &[shared::Frame],
    map: &[usize],
    spec: &shared::ObjectiveSpec,
    caches: &[GeneratePatchCache],
    replacements: &BTreeMap<usize, Replacement>,
) -> Result<Value> {
    let mut task = 0.;
    let mut reference = 0.;
    let mut good = 0;
    let mut phases = Vec::new();
    let mut masses = Vec::new();
    let val = |row: usize| {
        let c = replacements
            .get(&row)
            .map(|x| &x.cache)
            .unwrap_or(&caches[row]);
        (
            c.token_masses()[frames[row].target as usize],
            c.summary().total_weight_q31,
            c.summary().chosen_token_id,
        )
    };
    for &row in map {
        let (m, t, c) = val(row);
        replay_require(m > 0 && t > 0, "coupled objective empty mass")?;
        masses.push(json!([m, t, c]));
    }
    for role in spec
        .roles
        .as_ref()
        .ok_or_else(|| bad("coupled32 explicit roles missing"))?
    {
        let row = map
            .iter()
            .copied()
            .find(|&r| frames[r].input == role.input && frames[r].position == role.position)
            .ok_or_else(|| bad("coupled role frame missing"))?;
        let (m, t, c) = val(row);
        let ce = -(m as f64 / t as f64).ln() * role.weight;
        if role.task {
            task += ce;
            phases.push(json!({"position":role.position,"target":frames[row].target,"chosen":c,"target_mass":m,"total_mass":t}));
        } else {
            reference += ce;
            good += usize::from(c == frames[row].target);
        }
    }
    Ok(
        json!({"combined":task+reference,"task":task,"reference":reference,"correct_reference_frames":good,
        "phases":phases,"objective_masses":masses,"all_phase_winners":phases.iter().all(|p|p["target"]==p["chosen"])}),
    )
}
fn validated_prefix_keys<'a>(
    physical: usize,
    keys: &'a [Vec<Option<usize>>],
) -> Result<&'a [Vec<Option<usize>>]> {
    replay_require(
        keys.len() == physical
            && keys.iter().all(|row| {
                row.len() == 8
                    && row.iter().enumerate().all(|(lane, key)| {
                        key.is_some_and(|k| k >= lane * 120 && k < (lane + 1) * 120)
                    })
            }),
        "retained Prefix physical/lane keys invalid",
    )?;
    Ok(keys)
}
fn prefix_affected(frames: &[shared::Frame], key: usize) -> Result<Vec<usize>> {
    replay_require(key < COUNT, "Prefix coordinate domain")?;
    let mut rows = Vec::new();
    for (row, f) in frames.iter().enumerate() {
        let keys = validated_prefix_keys(f.ids.len(), &f.cue_keys)?;
        if keys.iter().any(|physical| physical[key / 120] == Some(key)) {
            rows.push(row);
        }
    }
    Ok(rows)
}
fn replacement_digest(
    rows: &BTreeMap<usize, Replacement>,
    frames: &[shared::Frame],
) -> Result<String> {
    let values=rows.iter().map(|(&i,r)| {let s=r.cache.summary();json!({"row":i,"reference_q24":s.reference_q24,
        "total_weight_q31":s.total_weight_q31,"chosen_token_id":s.chosen_token_id,"chosen_weight_q31":s.chosen_weight_q31,
        "target_mass":r.cache.token_masses()[frames[i].target as usize]})}).collect::<Vec<_>>();
    Ok(sha256_bytes(&serde_json::to_vec(&values)?))
}

fn construct(
    a: &Args,
    start: Instant,
    p: &ContinuationParent,
    frames: &[shared::Frame],
    map: &[usize],
    spec: &shared::ObjectiveSpec,
    pm: &[f32],
    pg: &[f32],
    gm: &[f32],
    gg: &[f32],
    caches: &mut [GeneratePatchCache],
    posts: &mut [Vec<H4Code>],
    donors: &mut [usize],
    incidences: &mut [RowIncidence],
    legal: &[u32],
    red: &mut NativeVocabularyActions,
    ranked: &[Coordinate],
) -> Result<(Vec<f32>, Vec<f32>, Value, Value)> {
    if transaction_mode(a) == PrefixTransaction::ProtectedJointVector {
        return protected_joint_vector::run(
            a, start, p, frames, map, spec, pm, pg, gm, gg, caches, posts, donors, incidences,
            legal, red,
        );
    }
    use std::io::Write as _;
    write(a, "coupled-coordinate-order.json", &json!(ranked))?;
    let baseline = generate::objective(frames, map, spec, caches, &generate::Patches::new())?;
    let mut current = baseline.clone();
    let mut prefix = pm.to_vec();
    let mut unary = gm.to_vec();
    let mut epoch = 0u64;
    let mut pc = PostCache::new(p);
    let mut native = native_with_unary(p, &unary)?;
    let mut accepted_prefix = 0usize;
    let mut accepted_generate = 0usize;
    let vector = transaction_mode(a) == PrefixTransaction::GradientVectorPrefix;
    let mut vector_receipt = Value::Null;
    let mut alternatives_count = 0u64;
    if vector {
        let outcome = gradient_vector_prefix::run(
            a, start, p, frames, map, spec, pm, pg, &native, caches, posts, donors, incidences,
            legal, red, &mut pc, &baseline,
        )?;
        prefix = outcome.masters;
        current = outcome.current;
        accepted_prefix = usize::from(outcome.committed);
        epoch = accepted_prefix as u64;
        alternatives_count = outcome.evaluated;
        vector_receipt = outcome.journal;
    }
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(a.out.join("coupled-construction.json"))?;
    let mut journal = std::io::BufWriter::new(file);
    journal.write_all(b"{\"policy\":")?;
    serde_json::to_writer(&mut journal, &policy_for_args(a))?;
    if vector {
        journal.write_all(
            b",\"schema\":\"uor-r4.gradient-vector-prefix-construction/1\",\"prefix_vector\":",
        )?;
        serde_json::to_writer(&mut journal, &vector_receipt)?;
    }
    journal.write_all(b",\"coordinate_records\":[")?;
    for (ordinal, coordinate) in ranked
        .iter()
        .filter(|r| !vector || r.family == GENERATE)
        .enumerate()
    {
        shared::progress(a, start)?;
        let before_epoch = epoch;
        let mut alternatives = Vec::new();
        let mut selected = json!({"status":"unchanged"});
        if coordinate.family == PREFIX {
            if coordinate.status == "eligible" {
                alternatives_count += 1;
                let mut proposed = prefix.clone();
                proposed[coordinate.index] = f32::from(coordinate.rank_code) * 0.25;
                let affected = prefix_affected(frames, coordinate.index)?;
                let affected_guards = affected
                    .iter()
                    .copied()
                    .filter(|i| *i < GUARDS)
                    .collect::<Vec<_>>();
                let mut rows = BTreeMap::new();
                for &row in map {
                    if affected.binary_search(&row).is_ok() {
                        rows.insert(
                            row,
                            replacement(
                                row,
                                &frames[row],
                                pm,
                                &proposed,
                                &native,
                                p,
                                &mut pc,
                                red,
                                legal,
                                &posts[row],
                            )?,
                        );
                    }
                }
                let next = replacement_objective(frames, map, spec, caches, &rows)?;
                let objective_gate = generate::valid_objective(&current, &next)?;
                let mut checked = Vec::new();
                let mut failure = Value::Null;
                if objective_gate {
                    for &row in &affected_guards {
                        if !rows.contains_key(&row) {
                            rows.insert(
                                row,
                                replacement(
                                    row,
                                    &frames[row],
                                    pm,
                                    &proposed,
                                    &native,
                                    p,
                                    &mut pc,
                                    red,
                                    legal,
                                    &posts[row],
                                )?,
                            );
                        }
                        checked.push(row);
                        let winner = rows[&row].cache.summary().chosen_token_id;
                        if winner != frames[row].target {
                            failure = json!({"guard_index":row,"required":frames[row].target,"chosen":winner});
                            break;
                        }
                    }
                }
                let feasible = objective_gate && failure.is_null();
                let changes = rows
                    .iter()
                    .map(|(&i, r)| {
                        Ok(json!([
                            i,
                            donors[i],
                            r.donor,
                            r.post.iter().map(|c| c.index()).collect::<Vec<_>>(),
                            r.incidence.is_some(),
                            match &r.incidence {
                                Some(x) => x.digest()?,
                                None => incidences[i].digest()?,
                            }
                        ]))
                    })
                    .collect::<Result<Vec<_>>>()?;
                let staged_digest = replacement_digest(&rows, frames)?;
                alternatives.push(json!({"code":coordinate.rank_code,"incumbent_epoch":epoch,"objective":journal_objective(&next),
                    "strict_current_CE_and17":objective_gate,"affected_guard_indices":affected_guards,
                    "checked_guard_indices":checked,"first_failure":failure,"guard_status":if !objective_gate{"NOT_CHECKED_OBJECTIVE_GATE_FALSE"}else if !failure.is_null(){"FIRST_VETO"}else{"FULL_PASS"},
                    "feasible":feasible,"changed_rows":changes,"staged_summary_digest":staged_digest}));
                if feasible {
                    // Every allocation/arithmetic/guard validation completed before infallible row swaps.
                    for &row in rows.keys() {
                        replay_require(
                            row < caches.len()
                                && row < posts.len()
                                && row < donors.len()
                                && row < incidences.len(),
                            "Prefix replacement batch row domain",
                        )?;
                    }
                    for (row, r) in rows {
                        caches[row] = r.cache;
                        posts[row] = r.post;
                        donors[row] = r.donor;
                        if let Some(inc) = r.incidence {
                            incidences[row] = inc;
                        }
                    }
                    prefix = proposed;
                    current = next;
                    epoch += 1;
                    accepted_prefix += 1;
                    selected = json!({"status":"committed","code":coordinate.rank_code,"staged_summary_digest":staged_digest});
                }
            } else {
                selected = json!({"status":"noop","reason":coordinate.status});
            }
        } else {
            let incumbent = generate::code(unary[coordinate.index])?;
            let mut best: Option<(f64, i8, generate::Patches, Value, String)> = None;
            for proposed in -7i8..=7 {
                if proposed == incumbent {
                    continue;
                }
                alternatives_count += 1;
                let delta = i64::from(proposed - incumbent) * (1 << 20);
                let mut changes = BTreeMap::new();
                let mut changed_atoms = 0usize;
                for (row, inc) in incidences.iter().enumerate() {
                    if !inc.matching(coordinate.index).is_empty() {
                        let atoms = inc
                            .matching(coordinate.index)
                            .iter()
                            .map(|&t| {
                                Ok((
                                    u32::from(t),
                                    caches[row].generate_scores()[t as usize]
                                        .checked_add(delta)
                                        .ok_or_else(|| bad("coupled unary overflow"))?,
                                ))
                            })
                            .collect::<Result<Vec<_>>>()?;
                        changed_atoms += atoms.len();
                        changes.insert(row, atoms);
                    }
                }
                let affected = changes
                    .keys()
                    .copied()
                    .filter(|i| *i < GUARDS)
                    .collect::<Vec<_>>();
                let mut staged = generate::Patches::new();
                for &row in map {
                    generate::stage_row(row, &changes, caches, &mut staged, red)?;
                }
                let next = generate::objective(frames, map, spec, caches, &staged)?;
                let objective_gate = generate::valid_objective(&current, &next)?;
                let mut checked = Vec::new();
                let mut failure = Value::Null;
                if objective_gate {
                    for &row in &affected {
                        generate::stage_row(row, &changes, caches, &mut staged, red)?;
                        checked.push(row);
                        let winner = staged[&row].summary().chosen_token_id;
                        if winner != frames[row].target {
                            failure = json!({"guard_index":row,"required":frames[row].target,"chosen":winner});
                            break;
                        }
                    }
                }
                let feasible = objective_gate && failure.is_null();
                let digest = generate::digest_staged(&staged, frames, caches)?;
                alternatives.push(json!({"code":proposed,"incumbent_code":incumbent,"incumbent_epoch":epoch,
                    "delta_code":proposed-incumbent,"actual_master_delta_from_original":f64::from(f32::from(proposed)*0.25)-f64::from(gm[coordinate.index]),
                    "changed_atom_count":changed_atoms,"objective":journal_objective(&next),"strict_current_CE_and17":objective_gate,
                    "affected_guard_indices":affected,"checked_guard_indices":checked,"first_failure":failure,
                    "guard_status":if !objective_gate{"NOT_CHECKED_OBJECTIVE_GATE_FALSE"}else if !failure.is_null(){"FIRST_VETO"}else{"FULL_PASS"},
                    "feasible":feasible,"staged_summary_digest":digest}));
                let ce = next["combined"]
                    .as_f64()
                    .ok_or_else(|| bad("coupled candidateCE missing"))?;
                let replace = best
                    .as_ref()
                    .is_none_or(|(old, q, _, _, _)| ce < *old || (ce == *old && proposed < *q));
                if feasible && replace {
                    best = Some((ce, proposed, staged, next, digest));
                }
            }
            if let Some((_, q, staged, next, digest)) = best {
                // Build the next native artifact before committing opaque row patches.
                let mut proposed = unary.clone();
                proposed[coordinate.index] = f32::from(q) * 0.25;
                let next_native = native_with_unary(p, &proposed)?;
                red.commit_generate_patch_batch(caches, staged.into_iter().collect())?;
                unary = proposed;
                native = next_native;
                current = next;
                epoch += 1;
                accepted_generate += 1;
                selected = json!({"status":"committed","code":q,"staged_summary_digest":digest});
            }
        }
        if ordinal > 0 {
            journal.write_all(b",")?;
        }
        serde_json::to_writer(
            &mut journal,
            &json!({"order":ordinal,"family":coordinate.family,"index":coordinate.index,
            "original_master":coordinate.original_master,"original_master_bits":coordinate.original_master.to_bits(),
            "gradient":coordinate.gradient,"gradient_bits":coordinate.gradient.to_bits(),"priority":coordinate.priority,
            "priority_bits":coordinate.priority.to_bits(),"rank_code":coordinate.rank_code,"rank_status":coordinate.status,
            "incumbent_epoch":before_epoch,"epoch_after":epoch,"alternatives":alternatives,"selected":selected}),
        )?;
    }
    let summary = json!({"coordinates":if vector{960}else{1920},"maximum_alternatives":if vector{13444}else{14400},"evaluated_alternatives":alternatives_count,
        "accepted_prefix":accepted_prefix,"accepted_generate":accepted_generate,"accepted_epoch":epoch,"revisited":0,
        "initial":baseline,"final":current,"immutable_post_cache_peak_bytes":pc.peak,"immutable_post_bridge_calls":pc.calls,
        "selected_restage_count":if vector{accepted_prefix}else{0}});
    journal.write_all(b"],\"summary\":")?;
    serde_json::to_writer(&mut journal, &summary)?;
    journal.write_all(b"}\n")?;
    journal.flush()?;
    Ok((prefix, unary, current, summary))
}

// Bound cache capacity by concrete row representations, never by an expected
// small affected subset. A late veto can retain replacements for all391 rows.
fn capacity_projection(max_copy: usize, legal: usize) -> Result<Value> {
    replay_require(
        legal <= VOCAB,
        "coupled legal vocabulary exceeds raw domain",
    )?;
    let row_cache = 3 * VOCAB * 8 + 4 * max_copy * 8 + 8 * VOCAB + 2048;
    let row_incidence = 961 * std::mem::size_of::<usize>() + 8 * legal * 2 + 1024;
    let pending = (VOCAB * 32).max(3 * VOCAB * 8) + 2048;
    // Sparse and full fallback storage are enum alternatives, never coexistent.
    let incumbent = UNION * (row_cache + row_incidence);
    let prefix_stage = UNION * (row_cache + row_incidence);
    let generate_stage = 2 * UNION * pending;
    let change_vectors = UNION * VOCAB * std::mem::size_of::<(u32, i64)>();
    let cache =
        incumbent + prefix_stage.max(generate_stage + change_vectors) + POST_CAP + 8 * 1024 * 1024;
    Ok(
        json!({"row_cache_bytes":row_cache,"row_incidence_bytes":row_incidence,
        "pending_row_bytes":pending,"persistent391_cache_incidence":incumbent,
        "prefix_all391_replacement_stage":prefix_stage,"generate_best_and_current_stage":generate_stage,
        "immutable_post_cache_cap":POST_CAP,"containers_margin":8*1024*1024,"all391_changed_atom_vector_bytes":change_vectors,
        "cache_upper_bound":cache,"cache_cap":CACHE_CAP,"max_copy_aliases":max_copy,"legal_generate_ids":legal}),
    )
}

fn capacity_for_mode(max_copy: usize, legal: usize, mode: PrefixTransaction) -> Result<Value> {
    let mut cap = capacity_projection(max_copy, legal)?;
    if !mode.legacy() {
        let compact = 2 * 1024 * 1024u64;
        cap["vector_compact_best_and_four_receipts_bound"] = json!(compact);
        cap["cache_upper_bound"] = json!(
            cap["cache_upper_bound"]
                .as_u64()
                .ok_or_else(|| bad("cache bound missing"))?
                + compact
        );
        cap["vector_restage_lifetime"]=json!("one all391 replacement batch; compact best only; selected restage after all proposal batches dropped");
    }
    Ok(cap)
}
fn journal_bound_for_mode(mode: PrefixTransaction) -> Result<u64> {
    let legacy = journal_upper_bound()?;
    // Legacy reserves960 full Prefix records. Four vector records each add960
    // projectedu32 bits and all391 change tuples, plus one selected receipt;
    // a4MiB explicit reserve covers these additions without subtracting legacy.
    Ok(legacy + if mode.legacy() { 0 } else { 4 * 1024 * 1024 })
}
fn journal_objective(v: &Value) -> Value {
    json!({"combined":v["combined"],"task":v["task"],"reference":v["reference"],
        "correct_reference_frames":v["correct_reference_frames"],"all_phase_winners":v["all_phase_winners"],
        "objective_masses":v["objective_masses"]})
}
fn journal_upper_bound() -> Result<u64> {
    let objective = json!({"combined":-f64::MAX,"task":-f64::MAX,"reference":-f64::MAX,
        "correct_reference_frames":17,"all_phase_winners":false,
        "objective_masses":vec![json!([u64::MAX,u64::MAX,u32::MAX]);31]});
    let affected = (0..GUARDS).collect::<Vec<_>>();
    let base = json!({"code":-7,"incumbent_code":7,"incumbent_epoch":1920,"delta_code":-14,
        "actual_master_delta_from_original":-f64::MAX,"changed_atom_count":UNION*VOCAB,
        "objective":objective,"strict_current_CE_and17":false,"affected_guard_indices":affected,
        "checked_guard_indices":affected,"first_failure":{"guard_index":379,"required":u32::MAX,"chosen":u32::MAX},
        "guard_status":"NOT_CHECKED_OBJECTIVE_GATE_FALSE","feasible":false,"staged_summary_digest":"f".repeat(64)});
    let mut pref = base.clone();
    pref["changed_rows"] = json!((0..UNION)
        .map(|i| json!([i, 512, 512, vec![119; 8], true, "f".repeat(64)]))
        .collect::<Vec<_>>());
    let g = serde_json::to_vec(&base)?.len() as u64;
    let p = serde_json::to_vec(&pref)?.len() as u64;
    Ok(g * 13440 + p * 960 + 1920 * 2048 + 16 * 1024 * 1024)
}
#[derive(Deserialize)]
struct Aliases {
    copy_ids: Vec<u32>,
}
fn flat_saved_alias_count(reader: impl std::io::Read) -> Result<usize> {
    let row: Aliases = serde_json::from_reader(reader)?;
    Ok(row.copy_ids.len())
}
fn original_guard_alias_bound(c: &Config) -> Result<usize> {
    let file = c
        .original_inputs
        .retained_intermediate_root
        .join("exported-candidate-protected-pools.json");
    replay_require(
        sha256_file(&file)? == "fcde63e9fc12f218b866b71323601485db7f93fd1b69a54beec91e87a7b84fad",
        "coupled full377 source identity differs",
    )?;
    let rows: Vec<Aliases> =
        serde_json::from_reader(std::io::BufReader::new(fs::File::open(&file)?))?;
    replay_require(rows.len() == 377, "coupled retained guard count differs")?;

    let e = c
        .original_inputs
        .episode
        .as_ref()
        .ok_or_else(|| bad("episode config absent"))?;
    report_output::verify(&e.retained_supplement_root)?;
    replay_require(
        sha256_file(&e.retained_supplement_root.join("report.json"))?
            == e.expected_supplement_report_sha256
            && sha256_file(&e.retained_supplement_root.join("manifest.json"))?
                == e.expected_supplement_manifest_sha256,
        "coupled supplement count authority differs",
    )?;
    let mut max = rows.iter().map(|r| r.copy_ids.len()).max().unwrap_or(0);
    for leaf in ["frame-0003-position-01.json", "frame-0455-position-00.json"] {
        let count = flat_saved_alias_count(std::io::BufReader::new(fs::File::open(
            e.retained_supplement_root.join(leaf),
        )?))?;
        max = max.max(count);
    }
    Ok(max)
}
fn donor_utility_bound(frames: &[shared::Frame]) -> Result<u64> {
    let mut total = 64 * 1024u64;
    for f in frames {
        let candidates = f.native["bank_trace"]["cue_bank"]["bank"]["candidates"]
            .as_array()
            .ok_or_else(|| bad("donor utility bound physical candidates missing"))?;
        replay_require(
            candidates.len() == f.ids.len() && !candidates.is_empty(),
            "donor utility bound physical identity differs",
        )?;
        for candidate in candidates {
            total = total
                .checked_add(serde_json::to_vec(candidate)?.len() as u64 + 2048)
                .ok_or_else(|| bad("donor utility bound overflow"))?;
        }
    }
    Ok(total)
}
fn pregradient_projection(a: &Args, c: &Config, frames: &[shared::Frame]) -> Result<()> {
    let e = c
        .original_inputs
        .episode
        .as_ref()
        .ok_or_else(|| bad("episode config absent"))?;
    report_output::verify(&e.retained_projection.root)?;
    replay_require(
        sha256_file(&e.retained_projection.root.join("report.json"))?
            == e.retained_projection.expected_report_sha256
            && sha256_file(&e.retained_projection.root.join("manifest.json"))?
                == e.retained_projection.expected_manifest_sha256,
        "coupled retained resource authority differs",
    )?;
    let old = read(
        &e.retained_projection
            .root
            .join("trajectory-resource-projection.json"),
    )?;
    let oldpre = read(&e.retained_projection.root.join("resource-projection.json"))?;
    let typed = old["typed_guard_frame_bound"]
        .as_u64()
        .ok_or_else(|| bad("typed guard bound missing"))?;
    let max_copy = frames
        .iter()
        .map(|f| f.ids.len())
        .max()
        .unwrap_or(0)
        .max(original_guard_alias_bound(c)?)
        .max(512);
    // Authenticate all377 retained physical alias counts before gradients;512
    // conservatively covers the two sealed supplements and objective-derived p3.
    // Every actual380 row is rechecked before constructor allocation.
    let cap = capacity_for_mode(max_copy, VOCAB, c.prefix_transaction)?;
    let cache = cap["cache_upper_bound"]
        .as_u64()
        .ok_or_else(|| bad("cache bound missing"))?;
    let full = frames.iter().try_fold(0u64, |n, f| {
        Ok::<_, Box<dyn std::error::Error>>(n + serde_json::to_vec(&f.native)?.len() as u64)
    })?;
    let retained_export_copy_peak = if let Some(e) = &c.retained_export {
        replay_require(
            sha256_file(&e.retained_failed_root.join("coupled-construction.json"))?
                == e.expected_complete_journal_sha256,
            "retained export journal projection identity differs",
        )?;
        generate::regular_file_bytes(&e.retained_failed_root.join("coupled-construction.json"))?
            + 4 * 1024 * 1024
    } else {
        0
    };
    let utility_report_bound = if c.donor_credit == DonorCredit::FullPoolUtility {
        donor_utility_bound(frames)?
    } else {
        0
    };
    // Full donor scalar receipts/Vec<Value> and one transient native pool;
    // the existing64MiB donor cache is sequential and never coexists with380.
    let utility_numeric_bound = utility_report_bound * 8
        + if utility_report_bound > 0 {
            2 * 1024 * 1024
        } else {
            0
        };
    let numeric = typed
        + cache
        + full * 2
        + 32 * 1024 * 1024
        + retained_export_copy_peak
        + utility_numeric_bound;
    replay_require(
        oldpre["process_ram_cap"] == 4 * 1024 * 1024 * 1024u64,
        "retained phase process authority differs",
    )?;
    // Prior measured fresh31 and complete380 host phases were sequential. Charge
    // their maximum plus allocator/device reserve, ALL current serialized31
    // expansion and the second family/native staging reserve, not old20 delta.
    let process = (2489696u64 * 1024).max(1540696u64 * 1024)
        + 512 * 1024 * 1024
        + full * 8
        + 128 * 1024 * 1024
        + retained_export_copy_peak
        + utility_numeric_bound;
    let vector_population_transient = if c.prefix_transaction.legacy() {
        0
    } else {
        16 * 1024 * 1024u64
    };
    let numeric = numeric + vector_population_transient;
    let process = process + vector_population_transient;
    let reload_max = 535837u64;
    let journal = journal_bound_for_mode(c.prefix_transaction)?;
    let report = (reload_max + 16384) * UNION as u64
        + journal
        + 57780353
        + 48 * 1024 * 1024
        + 32 * 1024 * 1024
        + utility_report_bound;
    let process_cap = if c.prefix_transaction == PrefixTransaction::ProtectedJointVector {
        8
    } else {
        4
    } * 1024
        * 1024
        * 1024u64;
    let v = json!({"stage":if c.retained_export.is_some(){"BEFORE_RETAINED_EXPORT_ADMISSION"}else if c.retained_mode(){"BEFORE_RETAINED_GRADIENT_ADMISSION_AND_FINITE_RESTART"}else{"BEFORE_ANY_BACKWARD"},"capacity":cap,"typed_guard_frame_bound":typed,
        "actual31_full_serialized_native_bytes":full,"numeric_upper_bound":numeric,"donor_utility_serialized_bound":utility_report_bound,"donor_utility_numeric_bound":utility_numeric_bound,"retained_export_copy_peak_bytes":retained_export_copy_peak,
        "retained_export_copy_projection":"full journal byte-copy buffer plus4MiB typed commitments/summary; alternatives ignored by typed decoder","numeric_cap":512*1024*1024u64,"vector_population_keyset_transient_bound":vector_population_transient,
        "process_ram_projection_bytes":process,"process_ram_cap":process_cap,
        "report_upper_bound":report,"report_cap":a.maximum_report_bytes,"streamed_journal_reserve":journal,
        "native391_snapshot_max_measured_bytes":reload_max,"fresh_family_gradient_bytes":if c.retained_mode(){0}else{2*32*3840},"retained_family_gradient_bytes":if c.retained_mode(){2*32*3840}else{0},
        "graph_lifetime":if c.prefix_transaction==PrefixTransaction::ProtectedJointVector{"31 objective and380 protected sequential graphs; guardtypedtraces+compactpools coexist with parameters; device graph/prepared Generate dropped before constructor"}else{"31 sequential joint graphs; original377 authority dropped before graph; all device graph/prepared Generate dropped before380 guards"},
        "constructor_lifetime":"full Pool buffers consumed/dropped before incidence/staging; replacement row incidence only, no global CSR clone",
        "role_count":32,"physical_graph_count":31,"episode_length":15,"fresh_backward_calls_completed":0,
        "original_resource_receipt":old,"original_pregradient_receipt":oldpre});
    write(a, "coupled-pregradient-resource-projection.json", &v)?;
    replay_require(
        cache <= CACHE_CAP as u64
            && numeric <= 512 * 1024 * 1024
            && process <= process_cap
            && report + 1024 * 1024 < a.maximum_report_bytes,
        "coupled complete pregradient resource projection exceeded",
    )
}
fn cache_mass_parity(cache: &GeneratePatchCache, pool: &shared::Pool) -> Result<()> {
    replay_require(
        cache.summary().chosen_token_id == pool.trace.summary.chosen_token_id
            && cache.summary().total_weight_q31 == pool.trace.summary.total_weight_q31
            && pool
                .trace
                .token_masses
                .iter()
                .all(|m| cache.token_masses()[m.token_id as usize] == m.weight_q31),
        "coupled opaque cache complete mass parity differs",
    )
}
pub(super) fn run(a: &Args, start: Instant, d: &Device) -> Result<Value> {
    validate_settings(a)?;

    let c = a
        .coupled_episode_learning
        .as_ref()
        .ok_or_else(|| bad("coupled config absent"))?;
    replay_require(
        c.retained_mode() || !d.is_cpu(),
        "coupled fresh gradient requires CUDA",
    )?;
    let original = ContinuationParent::from_checkpoint(&a.checkpoint)?;
    replay_require(
        original.binding.metadata_sha256 == shared::SOURCE
            && sha256_bytes(&original.generate) == shared::G_SHA
            && sha256_file(&a.checkpoint.join("continuation-field.bin"))? == shared::U_SHA,
        "coupled original epoch differs",
    )?;
    let original_g =
        NativeGeometricGenerate::from_bytes(&original.generate, original.integer.binding())?;
    let field = NativeContinuationField::from_bytes(
        &fs::read(a.checkpoint.join("continuation-field.bin"))?,
        &original.binding,
        &original_g,
    )?;
    let (mut objectives, spec) =
        prefix::prepare_generate_objective(a, &c.original_inputs, &original)?;
    replay_require(
        objectives.len() == 31 && spec.roles.as_ref().is_some_and(|r| r.len() == 32),
        "coupled role/physical population differs",
    )?;
    let mut reducer =
        NativeVocabularyActions::new(original.integer.binding().clone(), &original.exp)?;
    let mut objective_pools = objectives
        .iter()
        .map(|f| generate::saved_pool(f, &mut reducer))
        .collect::<Result<Vec<_>>>()?;
    for f in &objectives {
        let state = shared::codes(&f.native["continuation"]["state_codes"])?;
        let mut u = vec![0; VOCAB];
        field.score_delta_into(&state, &original_g, &mut u, &mut Default::default())?;
        replay_require(u == f.u, "coupled original U arithmetic differs")?;
    }
    let baseline_full = shared::objective_for_spec(&objectives, &objective_pools, &spec)?;
    write(a, "initial-original-objective.json", &baseline_full)?;
    pregradient_projection(a, c, &objectives)?;
    let mut protected = if c.prefix_transaction == PrefixTransaction::ProtectedJointVector {
        let guards =
            prefix::prepare_generate_guards(a, &c.original_inputs, &original, &objectives, &spec)?;
        for role in spec
            .roles
            .as_ref()
            .ok_or_else(|| bad("protected explicit role specification absent"))?
        {
            if !role.task {
                let reference = objectives
                    .iter()
                    .find(|f| f.input == role.input && f.position == role.position)
                    .ok_or_else(|| bad("protected reference objective absent"))?;
                replay_require(
                    guards.0.iter().any(|f| {
                        f.input == reference.input
                            && f.position == reference.position
                            && f.id == reference.id
                            && f.prefix == reference.prefix
                            && f.target == reference.target
                    }),
                    "protected original17 reference identity not covered by380",
                )?;
            }
        }
        protected_joint_vector::resource_projection(a, &guards.0, &guards.1)?;
        Some(guards)
    } else {
        None
    };
    let (pm, pg, gm, gg, inherited_learning) = if let Some(g) = c.gradient_authority() {
        retained_gradients(a, c, g, &objectives)?
    } else {
        let (pm, pg, gm, gg) = gradients(
            a,
            start,
            &original,
            &objectives,
            &objective_pools,
            d,
            protected.as_ref(),
        )?;
        (pm, pg, gm, gg, Value::Null)
    };
    let ranked = if c.retained_export.is_some() {
        Vec::new()
    } else {
        order(&pm, &pg, &gm, &gg)?
    };
    if c.retained_export.is_none() {
        if let Some(g) = c.gradient_authority() {
            replay_require(
                sha256_bytes(&serde_json::to_vec(&json!(ranked))?)
                    == g.expected_coordinate_order_sha256,
                "retained frozen order verification differs",
            )?;
        }
    }
    generate::slim(&mut objectives)?;
    for p in &mut objective_pools {
        generate::compact_pool(p);
    }
    let (mut frames, mut pools, authority) = if let Some(guards) = protected.take() {
        guards
    } else {
        prefix::prepare_generate_guards(a, &c.original_inputs, &original, &objectives, &spec)?
    };
    replay_require(
        frames.len() == GUARDS && pools.len() == GUARDS,
        "coupled guard population differs",
    )?;
    let mut map = Vec::with_capacity(31);
    for (f, pool) in objectives.into_iter().zip(objective_pools) {
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
                "coupled objective/guard overlap differs",
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
            && map.len() == 31
            && frames
                .iter()
                .map(|f| (f.input, f.position))
                .collect::<BTreeSet<_>>()
                .len()
                == UNION,
        "coupled391 unique union differs",
    )?;
    for f in &mut frames {
        if c.prefix_transaction == PrefixTransaction::ProtectedJointVector {
            f.prefix_trace = None;
        }
        validated_prefix_keys(f.ids.len(), &f.cue_keys)?;
    }
    let rows=frames.iter().enumerate().map(|(i,f)|{
        let mut row=json!({"row":i,"input_index":f.input,"position":f.position,
        "id":f.id,"actual_prefix_ids":f.prefix,"target_label_only":f.target,"zero_weight_guard":i<GUARDS,
        "guard_weight":0.,"coalesced_objective_weight":f.weight,"donor":pools[i].donor,
        "post_state":pools[i].post.iter().map(|x|x.index()).collect::<Vec<_>>()});
        if !c.prefix_transaction.legacy(){row["prefix_key_union"]=json!(validated_prefix_keys(f.ids.len(),&f.cue_keys)?.iter().flatten().flatten().copied().collect::<BTreeSet<_>>());}
        Ok(row)
    }).collect::<Result<Vec<_>>>()?;
    write(
        a,
        "coupled-population.json",
        &json!({"rows":rows,"objective_row_map":map,"roles":spec.roles,
        "guard_population":authority,"guards":GUARDS,"unique_union":UNION,"weighted_roles":32,"physical_frames":31}),
    )?;
    drop(rows); // No population JSON/key-set authority retained alongside live staging.
    let legal = reducer.legal_token_ids().to_vec();
    let max_copy = frames.iter().map(|f| f.ids.len()).max().unwrap_or(0);
    replay_require(
        max_copy <= 512,
        "coupled actual physical aliases exceed pregradient bound",
    )?;
    let cap = capacity_for_mode(512, legal.len(), c.prefix_transaction)?;
    write(
        a,
        "coupled-constructor-resource-projection.json",
        &json!({"capacity":cap,
        "stage":if c.retained_export.is_some(){"BEFORE_FINAL391_EXPECTED_POOL_RECONSTRUCTION_AND_NATIVE_RELOAD"}else if c.retained_mode(){"AFTER_INHERITED31_BACKWARDS_BEFORE_FINITE_RESTART"}else{"AFTER31_BACKWARDS_BEFORE_CONSTRUCTOR"},"actual_max_copy_aliases":max_copy,
        "model_graph_and_prepared_device_tensors_dropped":true,"all380_authority_complete":true,
        "pool_actions":"OMITTED_RECONSTRUCTIBLE_FROM_COMPLETE_SCORES"}),
    )?;
    replay_require(
        cap["cache_upper_bound"]
            .as_u64()
            .is_some_and(|n| n <= CACHE_CAP as u64),
        "coupled actual cache cap exceeded",
    )?;
    let mut posts = Vec::with_capacity(UNION);
    let mut donors = Vec::with_capacity(UNION);
    let mut caches = Vec::with_capacity(UNION);
    for (pool, f) in pools.drain(..).zip(&mut frames) {
        let cache = reducer.prepare_generate_patch_cache(
            pool.generate.clone(),
            f.ids.clone(),
            pool.copy.clone(),
        )?;
        cache_mass_parity(&cache, &pool)?;
        posts.push(pool.post);
        donors.push(pool.donor);
        caches.push(cache);
        let continuation = f.native["continuation"].clone();
        let candidates = f.native["bank_trace"]["cue_bank"]["bank"]["candidates"].clone();
        f.native = json!({"continuation":continuation,"bank_trace":{"cue_bank":{"bank":{"candidates":candidates}}}});
    }
    drop(pools);
    let mut incidences = posts
        .iter()
        .map(|post| RowIncidence::new(&original_g, post, &legal))
        .collect::<Result<Vec<_>>>()?;
    write(
        a,
        "coupled-incidence.json",
        &json!({"legal_generate_ids":legal,"row_incidence_digests":incidences.iter().map(|x|x.digest()).collect::<Result<Vec<_>>>()?,
        "total_postings":incidences.iter().map(|x|x.tokens.len()).sum::<usize>(),
        "encoding":"each row offsets961 usize and token u16 postings; first8 native factor keys; legal IDs only"}),
    )?;
    let initial = generate::objective(&frames, &map, &spec, &caches, &generate::Patches::new())?;
    replay_require(
        (initial["combined"]
            .as_f64()
            .ok_or_else(|| bad("initial CE missing"))?
            - baseline_full["combined"]
                .as_f64()
                .ok_or_else(|| bad("baseline CE missing"))?)
        .abs()
            < 1e-12,
        "coupled cache baseline differs",
    )?;
    let (current_prefix, current_unary, value, construction, inherited_construction) =
        if let Some(e) = &c.retained_export {
            let (cp, cg, summary, authority) = recover_export_state(a, c, e, &pm, &gm, &original)?;
            let current_g = native_with_unary(&original, &cg)?;
            let mut pc = PostCache::new(&original);
            for row in 0..UNION {
                let replacement = replacement(
                    row,
                    &frames[row],
                    &pm,
                    &cp,
                    &current_g,
                    &original,
                    &mut pc,
                    &mut reducer,
                    &legal,
                    &posts[row],
                )?;
                caches[row] = replacement.cache;
                posts[row] = replacement.post;
                donors[row] = replacement.donor;
            }
            let value =
                generate::objective(&frames, &map, &spec, &caches, &generate::Patches::new())?;
            replay_require(
                value == summary["final"],
                "retained final391 reconstructed objective differs",
            )?;
            (cp, cg, value, summary, authority)
        } else {
            let (cp, cg, value, summary) = construct(
                a,
                start,
                &original,
                &frames,
                &map,
                &spec,
                &pm,
                &pg,
                &gm,
                &gg,
                &mut caches,
                &mut posts,
                &mut donors,
                &mut incidences,
                &legal,
                &mut reducer,
                &ranked,
            )?;
            (cp, cg, value, summary, Value::Null)
        };
    if !inherited_learning.is_null() {
        verify_inherited_copies(a, &inherited_learning)?;
    }
    if !inherited_construction.is_null() {
        verify_inherited_copies(a, &inherited_construction)?;
    }
    let all_guards = (0..GUARDS).all(|i| caches[i].summary().chosen_token_id == frames[i].target);
    let gate = generate::final_gate(&initial, &value, all_guards)?;
    write(a, "final-objective.json", &value)?;
    write(
        a,
        "final-trajectory-guards.json",
        &json!({"guards":GUARDS,"all_original_winners":all_guards,
        "terms":(0..GUARDS).map(|i|json!({"guard_index":i,"input_index":frames[i].input,"position":frames[i].position,
            "required_original_winner":frames[i].target,"chosen":caches[i].summary().chosen_token_id,"pool":caches[i].summary()})).collect::<Vec<_>>()}),
    )?;
    drop(incidences);
    drop(pg);
    drop(gg);
    let receipt = export_reload(
        a,
        &original,
        &field,
        &frames,
        &caches,
        &posts,
        &donors,
        &pm,
        &gm,
        &current_prefix,
        &current_unary,
        &mut reducer,
    )?;
    // Close the immutable-science guarantee after all normalization/report
    // rewrites and export helpers, before admitting a completed report.
    if !inherited_learning.is_null() {
        verify_inherited_copies(a, &inherited_learning)?;
    }
    if !inherited_construction.is_null() {
        verify_inherited_copies(a, &inherited_construction)?;
    }
    let min_query = frames
        .iter()
        .filter_map(|f| f.native["continuation"]["query_tokens"].as_u64())
        .min();
    let max_query = frames
        .iter()
        .filter_map(|f| f.native["continuation"]["query_tokens"].as_u64())
        .max();
    Ok(
        json!({"schema":"uor-r4.coupled-episode-report/1","status":"COMPLETED","mode":"coupled_episode_learning",
        "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"policy":policy_for_modes(c.donor_credit,c.prefix_transaction),"selected_model":false,
        "finite_episode_positive":gate["passed"],"final_gate":gate,"baseline_objective":initial,"candidate_objective":value,
        "construction_summary":construction,"inherited_learning":inherited_learning,"inherited_construction":inherited_construction,
        "new_constructor_calls":if c.retained_export.is_some(){0}else{1},"new_proposals":if c.retained_export.is_some(){0}else{construction["evaluated_alternatives"].as_u64().unwrap_or(0)},
        "constructor_restart_from_original":c.retained_gradient.is_some(),"export_completion_only":c.retained_export.is_some(),
        "new_training_graph_forwards":if c.retained_mode(){0}else if c.prefix_transaction==PrefixTransaction::ProtectedJointVector{822}else{62},
        "new_backward_calls":if c.retained_mode(){0}else if c.prefix_transaction==PrefixTransaction::ProtectedJointVector{411}else{31},
        "protected_margin_backward_calls":if c.prefix_transaction==PrefixTransaction::ProtectedJointVector{380}else{0},
        "total_fresh_training_backward_calls":if c.retained_mode(){0}else if c.prefix_transaction==PrefixTransaction::ProtectedJointVector{411}else{31},
        "new_gradient_context_encoder_calls":0,
        "prior_partial_alternatives_charged":c.gradient_authority().map(|g|g.inherited_partial_alternatives),
        "candidate_receipt":receipt,"all_original380_preserved":all_guards,
        "weighted_roles":32,"unique_objective_frames":31,"episode_length":15,"state_width":8,
        "active_scalars_per_family":960,"physical_backward_calls":31,"prebackward_native_parity_graph_forwards":31,"gradient_graph_forwards":31,"extracted_family_gradients":2,
        "optimizer_updates":0,"candidate_native_steps":391,"expected_final_pool_reductions":391,
        "final_cache_preparations":if c.retained_export.is_some(){391}else{0},
        "new_constructor_proposals":if c.retained_export.is_some(){0}else{construction["evaluated_alternatives"].as_u64().unwrap_or(0)},
        "new_order_selection_calls":if c.retained_export.is_some(){0}else{1},
        "gradient_context_encoder_calls":0,"new_captured_objective_encoder_calls":0,
        "logical_gradient_producer":c.gradient_authority().map(|g|&g.expected_learning_source_commit),
        "fresh_gradient_files":if c.retained_mode(){0}else{62},
        "native_reload_context_encoding":"normal native generator during391 independent steps",
        "saved_query_token_count_range":[min_query,max_query],
        "source_physical_candidate_count_range":[frames.iter().map(|f|f.ids.len()).min(),max_copy],
        "actual_prefix_token_count_range":[frames.iter().map(|f|f.prefix.len()).min(),frames.iter().map(|f|f.prefix.len()).max()],
        "training_context_window":"fixed authenticated saved query/source/prefix inputs; no newly chosen token window",
        "evaluation_scope":"actual9 typed ownfeedback/EOS plus original8 only if positive; no full512",
        "actual9":"NOT_RUN_SEPARATE_ARTIFACT_CHECK","full512":"NOT_RUN","useful_candidate":false,
        "parent_master_bits_restored":true,"original_episode_authority":c.original_inputs.episode,
        "episode_request":c.original_inputs.episode}),
    )
}

// The generic exporter has already written Prefix. Preserve that owned
// intermediate before publishing a source whose Cue byte bindings were repaired.
// Both new destinations remain exclusive; a failed save retains the old Prefix.
fn publish_rebound_prefix(root: &Path, save: impl FnOnce(&Path) -> Result<()>) -> Result<()> {
    let current = root.join("prefix");
    let retained = root.join("prefix-before-cue-rebind");
    let staged = root.join("prefix-rebound");
    replay_require(
        fs::symlink_metadata(&current)?.file_type().is_dir(),
        "generic Prefix export is not a directory",
    )?;
    fs::create_dir(&retained)?;
    save(&staged)?;
    fs::rename(&current, &retained)?;
    fs::rename(&staged, &current)?;
    Ok(())
}

fn export_reload(
    a: &Args,
    p: &ContinuationParent,
    old_u: &NativeContinuationField,
    frames: &[shared::Frame],
    caches: &[GeneratePatchCache],
    posts: &[Vec<H4Code>],
    donors: &[usize],
    pm: &[f32],
    gm: &[f32],
    current_prefix: &[f32],
    current_unary: &[f32],
    red: &mut NativeVocabularyActions,
) -> Result<Value> {
    let d = Device::Cpu;
    let loaded = load_joint_continuation(a, p, &d)?;
    let cue = CueAngularWeights::load(
        &a.checkpoint.join("cue"),
        &loaded.frozen,
        &a.checkpoint.join("native"),
    )?;
    let carrier = loaded.frozen.compile_cue_carrier(cue.native()?)?;
    let prefix = PrefixAngularWeights::load(
        &a.checkpoint.join("prefix"),
        &loaded.frozen,
        &a.checkpoint.join("native"),
        &a.checkpoint.join("cue"),
        &carrier,
    )?;
    let mut params = loaded.generate.parameters();
    params.extend(prefix.parameters());
    let saved = np::snapshot(&params)?;
    replay_require(
        saved
            .get(PREFIX)
            .is_some_and(|m| m.iter().zip(pm).all(|(a, b)| a.to_bits() == b.to_bits()))
            && saved
                .get(GENERATE)
                .is_some_and(|m| m.iter().zip(gm).all(|(a, b)| a.to_bits() == b.to_bits())),
        "coupled independently loaded initial master authority differs",
    )?;
    let frozen_source = identities(&loaded.source.parameters())?;
    let frozen_cue = identities(&cue.parameters())?;
    let result = np::attempt_restored(&params, &saved, || {
        let mut expected = saved.clone();
        expected.insert(PREFIX.into(), current_prefix.to_vec());
        expected.insert(GENERATE.into(), current_unary.to_vec());
        np::restore(&params, &expected)?;
        replay_require(
            np::same_bits(&np::snapshot(&params)?, &expected)
                && identities(&loaded.source.parameters())? == frozen_source
                && identities(&cue.parameters())? == frozen_cue,
            "coupled selected/frozen floating master guard differs",
        )?;
        let (cp, rebound, mut receipt) =
            shared::export_joint(a, &loaded, &cue, Some(&prefix), p, &d)?;
        replay_require(
            cp.binding == p.binding
                && cp.bridge == p.bridge
                && cp.cue == p.cue
                && cp.joint == p.joint
                && rebound.packed_unary() == old_u.packed_unary(),
            "coupled Source/bridge/Cue/U numerical export differs",
        )?;
        let root = a.out.join("checkpoint-0001");
        let mut repairs = Vec::new();
        for entry in fs::read_dir(a.checkpoint.join("cue"))? {
            let entry = entry?;
            replay_require(
                entry.file_type()?.is_file(),
                "frozen Cue directory contains unsupported type",
            )?;
            let leaf = entry.file_name();
            repairs.push(generate::restore_original_sidecar_file(
                &entry.path(),
                &root.join("cue").join(&leaf),
                leaf.to_str() == Some("native-metadata.json"),
            )?);
        }
        generate::exact_frozen_directory(&a.checkpoint.join("cue"), &root.join("cue"))?;
        // Cue is numerically frozen and restored to ORIGINAL bytes. Its byte
        // bindings must be reflected in learned Prefix metadata, not the
        // temporary generic writer's semantically equivalent JSON serialization.
        let transport = loaded
            .frozen
            .compile_prefix_transport(&carrier, prefix.native()?)?;
        let mut rebound_prefix = PrefixAngularWeights::from_native(
            &loaded.frozen,
            &root.join("native"),
            &root.join("cue"),
            &carrier,
            &transport,
            &d,
        )?;
        np::restore(
            &rebound_prefix.parameters(),
            &np::snapshot(&prefix.parameters())?,
        )?;
        rebound_prefix.rebind_cue(&loaded.frozen, &root.join("cue"), &carrier)?;
        publish_rebound_prefix(&root, |staged| Ok(rebound_prefix.save(staged)?))?;

        generate::exact_frozen_directory(&a.checkpoint.join("source"), &root.join("source"))?;
        for leaf in [
            "source/consumer/context.safetensors",
            "source/consumer/potential.safetensors",
            "continuation-source/continuation.unary.f32le",
        ] {
            if a.checkpoint.join(leaf).is_file() {
                replay_require(
                    fs::read(a.checkpoint.join(leaf))? == fs::read(root.join(leaf))?,
                    "coupled frozen numerical file differs",
                )?;
            }
        }
        let cpu_g = GenerateLearningWeights::from_native(
            cp.integer.binding().clone(),
            &NativeGeometricGenerate::from_bytes(&cp.generate, cp.integer.binding())?,
            &d,
        )?;
        restore(
            &root.join("generate-source"),
            &read(&root.join("generate-source/metadata.json"))?["parameters"],
            &cpu_g.parameters(),
            &d,
        )?;
        let cpu_prefix = PrefixAngularWeights::load(
            &root.join("prefix"),
            &loaded.frozen,
            &root.join("native"),
            &root.join("cue"),
            &carrier,
        )?;
        let mut disk = cpu_g.parameters();
        disk.extend(cpu_prefix.parameters());
        replay_require(
            np::same_bits(&np::snapshot(&disk)?, &expected)
                && cpu_g.export_native()?.to_bytes()? == cp.generate,
            "coupled independent all-master reload differs",
        )?;
        drop(disk);
        drop(cpu_g);
        drop(cpu_prefix);
        drop(expected);
        write(
            a,
            "coupled-frozen-sidecar-restoration.json",
            &json!({"files":repairs,"frozen_numerical_source":true,
            "frozen_Cue":true,"active_Prefix":true,"active_Generate_unary":true,"U_coefficients_frozen":true,
            "preserved_generic_prefix":"checkpoint-0001/prefix-before-cue-rebind"}),
        )?;
        receipt["mode"] = json!("coupled_episode_learning");
        receipt["policy"] = policy_for_args(a);
        receipt["active_parameter_names"] = json!([PREFIX, GENERATE]);
        receipt["fresh_adam"] = json!(false);
        receipt["optimizer_updates"] = json!(0);
        receipt["new_gradients"] = json!(1);
        receipt["coefficient_backward_calls"] = json!(if transaction_mode(a)
            == PrefixTransaction::ProtectedJointVector
        {
            411
        } else {
            31
        });
        let inherited = a
            .coupled_episode_learning
            .as_ref()
            .and_then(|c| c.gradient_authority());
        receipt["new_gradients"] = json!(if inherited.is_some() { 0 } else { 1 });
        receipt["new_backward_calls"] = json!(if inherited.is_some() {
            0
        } else if transaction_mode(a) == PrefixTransaction::ProtectedJointVector {
            411
        } else {
            31
        });
        receipt["new_training_graph_forwards"] = json!(if inherited.is_some() {
            0
        } else if transaction_mode(a) == PrefixTransaction::ProtectedJointVector {
            822
        } else {
            62
        });
        receipt["inherited_learning_source_commit"] =
            json!(inherited.map(|g| &g.expected_learning_source_commit));
        let export_only = a
            .coupled_episode_learning
            .as_ref()
            .is_some_and(|c| c.retained_export.is_some());
        receipt["new_constructor_calls"] = json!(if export_only { 0 } else { 1 });
        receipt["new_proposals"] = if export_only { json!(0) } else { Value::Null };
        receipt["new_order_selection_calls"] = json!(if export_only { 0 } else { 1 });
        receipt["inherited_constructor_source_commit"] = json!(a
            .coupled_episode_learning
            .as_ref()
            .and_then(|c| c.retained_export.as_ref())
            .map(|e| &e.expected_source_commit));
        receipt["gradient_scope"] = json!(if export_only {
            "31 inherited completed joint backwards; all final codes inherited from completed constructor; zero new graphs/backwards/order/proposals/constructor"
        } else if inherited.is_some() {
            "31 inherited completed joint backwards, zero fresh graphs/backwards; original-seed finite constructor restart"
        } else if transaction_mode(a) == PrefixTransaction::ProtectedJointVector {
            "31 fresh task/reference CE backwards plus380 protected-margin backwards;411 total,822 graph forwards"
        } else {
            "31 fresh joint backwards"
        });

        receipt["extracted_family_gradients"] = json!(2);
        receipt["credit_scope"]=json!("one31 factual fullaliasloss joint pullback: Prefix direct gather plus detached conditional donor contrast and factual Generate unary coefficient STE; only two960 gradients extracted/proposed; no Context/Cue/U extraction");
        if credit_mode(a) == DonorCredit::FullPoolUtility {
            receipt["credit_scope"] = policy_for_args(a)["gradient_surrogate"].clone();
        }

        receipt["frozen_numerical_scope"]=json!("all Source/Context/Potential/map/Cue angular+joint/Generate pair+bias+prototype/bridge/U masters frozen; Prefix960 and Generate unary960 only");
        receipt["generate_sha256"] = json!(sha256_bytes(&cp.generate));
        receipt["prefix_sha256"] = json!(sha256_file(&root.join("prefix/prefix-q4.bin"))?);
        receipt["candidate_native_steps"] = json!(0);
        receipt["native_independently_reloaded"] = json!(false);
        for leaf in ["receipt.json", "continuation-source/metadata.json"] {
            fs::write(root.join(leaf), serde_json::to_vec_pretty(&receipt)?)?;
        }
        let reloaded = ContinuationParent::from_checkpoint(&root)?;
        let mut final_pools = Vec::with_capacity(UNION);
        for (i, cache) in caches.iter().enumerate() {
            let mut trace = red.reduce_trace(
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
                "coupled final authoritative mass parity differs",
            )?;
            trace.actions.clear();
            trace.actions.shrink_to_fit();
            let base_copy = cache
                .copy_scores()
                .iter()
                .zip(&frames[i].ids)
                .map(|(v, id)| v.checked_sub(frames[i].u[*id as usize]))
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| bad("coupled final BASE subtraction overflow"))?;
            replay_require(
                earliest(&base_copy)? == donors[i],
                "coupled final physical donor differs",
            )?;
            final_pools.push(shared::Pool {
                donor: donors[i],
                post: posts[i].clone(),
                generate: cache.generate_scores().to_vec(),
                copy: cache.copy_scores().to_vec(),
                base_copy,
                trace,
            });
        }
        shared::reload_guard_candidate(a, &reloaded, &rebound, frames, &final_pools)?;
        receipt["candidate_native_steps"] = json!(391);
        receipt["expected_finite_pool_reductions"] = json!(391);
        receipt["native_independently_reloaded"] = json!(true);
        for leaf in ["receipt.json", "continuation-source/metadata.json"] {
            fs::write(root.join(leaf), serde_json::to_vec_pretty(&receipt)?)?;
        }
        Ok(receipt)
    });
    replay_require(
        np::same_bits(&np::snapshot(&params)?, &saved),
        "coupled parent masters not restored",
    )?;
    result
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ArtifactAuthority {
    #[serde(default, skip_serializing_if = "PrefixTransaction::legacy")]
    pub prefix_transaction: PrefixTransaction,
    #[serde(default, skip_serializing_if = "DonorCredit::legacy")]
    pub donor_credit: DonorCredit,
    pub expected_report_sha256: String,
    pub expected_manifest_sha256: String,
    pub expected_generate_sha256: String,
    pub expected_unary_master_sha256: String,
    pub expected_prefix_packed_sha256: String,
    pub expected_prefix_master_sha256: String,
    pub expected_continuation_sha256: String,
}
pub(super) fn validate_artifact_authority(p: &ArtifactAuthority) -> Result<()> {
    validate_transaction(p.prefix_transaction, p.donor_credit, false)?;
    replay_require(
        [
            &p.expected_report_sha256,
            &p.expected_manifest_sha256,
            &p.expected_generate_sha256,
            &p.expected_unary_master_sha256,
            &p.expected_prefix_packed_sha256,
            &p.expected_prefix_master_sha256,
            &p.expected_continuation_sha256,
        ]
        .iter()
        .all(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit())),
        "coupled artifact external hashes invalid",
    )
}
pub(super) fn authenticate_positive_artifact(
    a: &Args,
    c: &prefix::ArtifactConfig,
    v: &Value,
) -> Result<()> {
    let pin = c
        .coupled_episode_candidate
        .as_ref()
        .ok_or_else(|| bad("coupled artifact authority absent"))?;
    validate_artifact_authority(pin)?;
    if pin.donor_credit == DonorCredit::FullPoolUtility {
        let candidate_config: Config = serde_json::from_value(
            read(&c.retained_candidate_root.join("config.json"))?["coupled_episode_learning"]
                .clone(),
        )?;
        replay_require(
            candidate_config.donor_credit == pin.donor_credit
                && candidate_config.prefix_transaction == pin.prefix_transaction
                && v["policy"] == policy_for_modes(pin.donor_credit, pin.prefix_transaction),
            "coupled artifact donor-credit authority differs",
        )?;
        validate_credit_recovery(
            candidate_config.donor_credit,
            candidate_config.retained_mode(),
        )?;
        validate_transaction(
            candidate_config.prefix_transaction,
            candidate_config.donor_credit,
            candidate_config.retained_mode(),
        )?;
    }

    replay_require(
        v["status"] == "COMPLETED"
            && v["mode"] == "coupled_episode_learning"
            && v["selected_model"] == false
            && v["finite_episode_positive"] == true
            && v["final_gate"]["passed"] == true
            && v["all_original380_preserved"] == true
            && v["physical_backward_calls"] == 31
            && v["extracted_family_gradients"] == 2
            && v["weighted_roles"] == 32
            && v["unique_objective_frames"] == 31
            && v["candidate_native_steps"] == 391
            && v["optimizer_updates"] == 0
            && v["candidate_objective"]["all_phase_winners"] == true
            && v["candidate_objective"]["correct_reference_frames"] == 17,
        "coupled artifact lacks positive native construction",
    )?;
    let root = &c.retained_candidate_root;
    let gradient = read(&root.join("coupled-gradient-receipt.json"))?;
    let gradient_mode = if gradient["donor_credit"].is_null() {
        DonorCredit::StateTangent
    } else {
        serde_json::from_value(gradient["donor_credit"].clone())?
    };
    replay_require(
        gradient_mode == pin.donor_credit,
        "coupled positive gradient mode differs",
    )?;
    if pin.donor_credit == DonorCredit::FullPoolUtility {
        replay_require(
            gradient["full_pool_utility_file"] == "coupled-full-pool-donor-utilities.json"
                && gradient["full_pool_utility_sha256"]
                    == sha256_file(&root.join("coupled-full-pool-donor-utilities.json"))?,
            "coupled positive donor utility receipt identity differs",
        )?;
        let utility = read(&root.join("coupled-full-pool-donor-utilities.json"))?;
        replay_require(
            utility["donor_credit"] == json!(pin.donor_credit)
                && utility["physical_frames"] == 31
                && utility["all_before_any_backward"] == true,
            "coupled positive donor utility factual authority differs",
        )?;
    }
    replay_require(
        gradient["physical_backward_calls"] == 31 && gradient["weighted_roles"] == 32,
        "coupled gradient receipt count differs",
    )?;
    let terms = gradient["perterm"]
        .as_array()
        .ok_or_else(|| bad("coupled gradient rows missing"))?;
    replay_require(terms.len() == 31, "coupled gradient row count differs")?;
    for term in terms {
        let families = term["families"]
            .as_array()
            .ok_or_else(|| bad("coupled raw families absent"))?;
        replay_require(
            families.len() == 2,
            "coupled extracted family count differs",
        )?;
        let mut names = BTreeSet::new();
        for f in families {
            let name = f["family"]
                .as_str()
                .ok_or_else(|| bad("coupled gradient name missing"))?;
            let leaf = f["file"]
                .as_str()
                .ok_or_else(|| bad("coupled gradient filename missing"))?;
            names.insert(name.to_string());
            replay_require(
                Path::new(leaf).components().count() == 1
                    && f["status"] == "PRESENT"
                    && f["missing_gradient_filled_zero"] == false
                    && generate::regular_file_bytes(&root.join(leaf))? == 3840
                    && sha256_file(&root.join(leaf))?
                        == f["sha256"]
                            .as_str()
                            .ok_or_else(|| bad("coupled raw SHA absent"))?,
                "coupled actual raw gradient authority differs",
            )?;
        }
        replay_require(
            names
                == [PREFIX.to_string(), GENERATE.to_string()]
                    .into_iter()
                    .collect(),
            "coupled family inventory differs",
        )?;
    }

    let files = gradient["files"]
        .as_array()
        .ok_or_else(|| bad("coupled aggregate/master receipt absent"))?;
    replay_require(
        files.len() == 4,
        "coupled aggregate/master file count differs",
    )?;
    let mut inventory = BTreeSet::new();
    for f in files {
        let family = f["family"]
            .as_str()
            .ok_or_else(|| bad("coupled file family absent"))?;
        let kind = f["kind"]
            .as_str()
            .ok_or_else(|| bad("coupled file kind absent"))?;
        let leaf = f["file"]
            .as_str()
            .ok_or_else(|| bad("coupled aggregate filename absent"))?;
        replay_require(
            [PREFIX, GENERATE].contains(&family)
                && ["gradient", "initial-master"].contains(&kind)
                && inventory.insert((family, kind))
                && Path::new(leaf).components().count() == 1
                && generate::regular_file_bytes(&root.join(leaf))? == 3840
                && sha256_file(&root.join(leaf))?
                    == f["sha256"]
                        .as_str()
                        .ok_or_else(|| bad("coupled aggregate/master SHA absent"))?,
            "coupled aggregate/master identity differs",
        )?;
        if kind == "initial-master" {
            let original =
                c.retained_intermediate_root
                    .join("checkpoint-0001")
                    .join(if family == PREFIX {
                        "prefix/prefix-source-f32.bin"
                    } else {
                        "generate-source/generate.unary.f32le"
                    });
            replay_require(
                fs::read(&root.join(leaf))? == fs::read(&original)?,
                "coupled ORIGINAL fractional master file differs",
            )?;
        }
    }
    let journal = read(&root.join("coupled-construction.json"))?;
    if pin.prefix_transaction == PrefixTransaction::ProtectedJointVector {
        protected_joint_vector::authenticate(root, &journal, &gradient, &a.checkpoint)?;
    } else {
        let records = journal["coordinate_records"]
            .as_array()
            .ok_or_else(|| bad("coupled journal absent"))?;
        replay_require(
            records.len()
                == (if pin.prefix_transaction.legacy() {
                    1920
                } else {
                    960
                })
                && journal["summary"]["coordinates"]
                    == (if pin.prefix_transaction.legacy() {
                        1920
                    } else {
                        960
                    })
                && journal["summary"]["revisited"] == 0
                && journal["summary"]["maximum_alternatives"]
                    == (if pin.prefix_transaction.legacy() {
                        14400
                    } else {
                        13444
                    }),
            "coupled finite once-only population differs",
        )?;
        if !pin.prefix_transaction.legacy() {
            let ranked: Vec<Coordinate> =
                serde_json::from_value(read(&root.join("coupled-coordinate-order.json"))?)?;
            let population = read(&root.join("coupled-population.json"))?;
            gradient_vector_prefix::authenticate(
                &journal,
                &ranked,
                &population,
                root,
                &gradient,
                &a.checkpoint,
            )?;
            validate_inherited_transaction(
                pin.prefix_transaction,
                pin.prefix_transaction,
                &gradient,
            )?;
        }
        let mut seen = BTreeSet::new();
        let mut g_alternatives = 0usize;
        for row in records {
            let family = row["family"]
                .as_str()
                .ok_or_else(|| bad("coupled family absent"))?;
            let index = shared::idx(&row["index"])?;
            replay_require(
                index < 960
                    && (family == PREFIX || family == GENERATE)
                    && seen.insert((family, index)),
                "coupled duplicate/out-of-domain coordinate",
            )?;
            if family == GENERATE {
                let alternatives = row["alternatives"]
                    .as_array()
                    .ok_or_else(|| bad("coupled legal alternatives absent"))?;
                let q = alternatives
                    .first()
                    .and_then(|x| x["incumbent_code"].as_i64())
                    .ok_or_else(|| bad("coupled incumbent code absent"))?;
                let codes = alternatives
                    .iter()
                    .map(|r| {
                        r["code"]
                            .as_i64()
                            .ok_or_else(|| bad("coupled proposed code absent"))
                    })
                    .collect::<Result<BTreeSet<_>>>()?;
                replay_require(
                    alternatives.len() == 14
                        && codes == (-7..=7).filter(|x| *x != q).collect()
                        && alternatives
                            .iter()
                            .all(|r| r["incumbent_epoch"] == row["incumbent_epoch"]),
                    "coupled all14 same epoch differs",
                )?;
                g_alternatives += 14;
            }
        }
        replay_require(
            g_alternatives == 13440,
            "coupled complete Generate code population differs",
        )?;
    }
    drop(journal);
    let guards = read(&root.join("final-trajectory-guards.json"))?;
    replay_require(
        guards["all_original_winners"] == true
            && guards["terms"].as_array().is_some_and(|r| {
                r.len() == 380
                    && r.iter()
                        .all(|r| r["chosen"] == r["required_original_winner"])
            }),
        "coupled380 retention receipt differs",
    )?;
    let original =
        ContinuationParent::from_checkpoint(&c.retained_intermediate_root.join("checkpoint-0001"))?;
    let cp = ContinuationParent::from_checkpoint(&a.checkpoint)?;
    replay_require(
        original.binding == cp.binding
            && original.bridge == cp.bridge
            && original.cue == cp.cue
            && original.joint == cp.joint
            && sha256_bytes(&cp.generate) == pin.expected_generate_sha256
            && sha256_file(&a.checkpoint.join("generate-source/generate.unary.f32le"))?
                == pin.expected_unary_master_sha256
            && sha256_file(&a.checkpoint.join("prefix/prefix-q4.bin"))?
                == pin.expected_prefix_packed_sha256
            && sha256_file(&a.checkpoint.join("prefix/prefix-source-f32.bin"))?
                == pin.expected_prefix_master_sha256
            && sha256_file(&a.checkpoint.join("continuation-field.bin"))?
                == pin.expected_continuation_sha256,
        "coupled artifact active/frozen identity differs",
    )?;
    generate::exact_frozen_directory(
        &c.retained_intermediate_root.join("checkpoint-0001/source"),
        &a.checkpoint.join("source"),
    )?;
    generate::exact_frozen_directory(
        &c.retained_intermediate_root.join("checkpoint-0001/cue"),
        &a.checkpoint.join("cue"),
    )?;
    let og = NativeGeometricGenerate::from_bytes(&original.generate, original.integer.binding())?;
    let cg = NativeGeometricGenerate::from_bytes(&cp.generate, cp.integer.binding())?;
    let ou = NativeContinuationField::from_bytes(
        &fs::read(
            c.retained_intermediate_root
                .join("checkpoint-0001/continuation-field.bin"),
        )?,
        &original.binding,
        &og,
    )?;
    let cu = NativeContinuationField::from_bytes(
        &fs::read(a.checkpoint.join("continuation-field.bin"))?,
        &cp.binding,
        &cg,
    )?;
    replay_require(
        ou.packed_unary() == cu.packed_unary(),
        "coupled U frozen coefficient bytes differ",
    )?;
    for entry in fs::read_dir(
        c.retained_intermediate_root
            .join("checkpoint-0001/generate-source"),
    )? {
        let entry = entry?;
        let leaf = entry.file_name();
        if entry.file_type()?.is_file() && leaf != "generate.unary.f32le" && leaf != "metadata.json"
        {
            replay_require(
                fs::read(entry.path())?
                    == fs::read(a.checkpoint.join("generate-source").join(leaf))?,
                "coupled frozen Generate numerical master differs",
            )?;
        }
    }
    replay_require(
        v["candidate_receipt"]["native_independently_reloaded"] == true
            && v["candidate_receipt"]["active_parameter_names"] == json!([PREFIX, GENERATE]),
        "coupled final independent reload receipt differs",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn donor_pair_fixture() -> Result<(
        NativeGeometricGenerate,
        GenerateLearningWeights,
        Vec<H4Code>,
        Vec<H4Code>,
        NativeVocabularyActions,
    )> {
        use uor_r4_core::native_geometric::learner::integrated_attention::geometry::{
            EnergyTables, LanePair,
        };
        use uor_r4_integer::geometric_source_actions::SourceActionBinding;
        const TOK: &str = r#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4,"b":5,"Ġ":6,"Ġa":7},"merges":["Ġ a"]},"added_tokens":[{"id":0,"content":"<|bos|>"},{"id":1,"content":"<|eos|>"},{"id":2,"content":"<|unk|>"}]}"#;
        let binding = SourceActionBinding::new(TOK.as_bytes())?;
        let prototypes = (0..binding.vocab_size())
            .flat_map(|t| (0..8).map(move |l| ((t * 17 + l * 7 + 3) % 120) as u8))
            .collect::<Vec<_>>();
        let edges = vec![LanePair { left: 0, right: 1 }];
        let mut energy = EnergyTables::zeroed(8, edges)?;
        let zero = NativeGeometricGenerate::compile(
            &binding,
            8,
            &prototypes,
            &vec![0; binding.vocab_size().div_ceil(2)],
            energy.clone(),
        )?;
        let factual = vec![H4Code::IDENTITY; 8];
        let mut changed = factual.clone();
        changed[0] = H4Code::try_from(2)?;
        changed[1] = H4Code::try_from(3)?;
        let mut fk = [0u32; 9];
        let mut ck = fk;
        zero.factor_incidence_into(&factual, 4, &mut fk, &mut Default::default())?;
        zero.factor_incidence_into(&changed, 4, &mut ck, &mut Default::default())?;
        let f = (fk[8] - 960) as usize;
        let c = (ck[8] - 960) as usize;
        assert_ne!(f / 120, c / 120);
        assert_ne!(f % 120, c % 120);
        // Only the joint pair endpoint has energy; both one-lane changes are0.
        energy.set_pair(0, (c / 120) as u8, (c % 120) as u8, 7)?;
        let native = NativeGeometricGenerate::compile(
            &binding,
            8,
            &prototypes,
            &vec![0; binding.vocab_size().div_ceil(2)],
            energy,
        )?;
        let weights = GenerateLearningWeights::from_native(binding.clone(), &native, &Device::Cpu)?;
        let exp = (0..uor_r4_integer::geometric_read::EXP_TABLE_LEN)
            .flat_map(|i| {
                (((-(i as f64) / 256.).exp() * (1u64 << 31) as f64).round() as u32).to_le_bytes()
            })
            .collect::<Vec<_>>();
        let red = NativeVocabularyActions::new(binding, &exp)?;
        Ok((native, weights, factual, changed, red))
    }
    #[test]
    fn full_pool_donor_credit_captures_joint_pair_transition_missing_state_tangent() -> Result<()> {
        let (native, g, factual, changed, mut red) = donor_pair_fixture()?;
        let mut rawf = vec![0; native.vocab_size()];
        let mut rawc = rawf.clone();
        native.score_into(&factual, &mut rawf, &mut Default::default())?;
        native.score_into(&changed, &mut rawc, &mut Default::default())?;
        assert_eq!(rawf[4], 0);
        assert_eq!(rawc[4], 7 << 20);
        let u = vec![0; native.vocab_size()];
        let ids = [4, 4, 5];
        let copy = [0, 0, 0];
        let old = forced_pool_loss(&mut red, &rawf, &u, &ids, &copy, 4)?;
        let new = forced_pool_loss(&mut red, &rawc, &u, &ids, &copy, 4)?;
        assert!(new.0 < old.0);
        let raw = Var::from_vec(vec![0f32; 3], 3, &Device::Cpu)?;
        let states = shared::selector(
            &factual,
            &[factual.clone(), factual.clone(), changed],
            raw.as_tensor(),
            &Device::Cpu,
        )?;
        let prepared = g.prepare_native()?;
        let tangent = g.forward_prepared_state_choices(&prepared, &factual, &states)?;
        let tg = tangent.raw_scores.narrow(0, 4, 1)?.sum_all()?.backward()?;
        let tangent_route = tg
            .get(raw.as_tensor())
            .ok_or_else(|| bad("tangent route gradient missing"))?
            .to_vec1::<f32>()?;
        assert!(tangent_route.iter().all(|x| x.abs() < 1e-7));
        let full = zero_forward_donor_loss(raw.as_tensor(), &[0., 0., (new.0 - old.0) as f32])?;
        assert_eq!(full.to_scalar::<f32>()?, 0.);
        let grads = full.backward()?;
        let route = grads
            .get(raw.as_tensor())
            .ok_or_else(|| bad("complete donor gradient missing"))?
            .to_vec1::<f32>()?;
        assert!(route[0] > 0. && route[1] > 0. && route[2] < 0.);
        assert_eq!(route[0], route[1]);
        assert!(grads.get(g.unary.as_tensor()).is_none());
        Ok(())
    }
    #[test]
    fn full_pool_donor_addition_preserves_factual_generate_and_direct_copy_credit_weight_once(
    ) -> Result<()> {
        use uor_r4_training::geometric_generate_learning::vocabulary_marginal_loss;
        let (native, g, state, _, mut red) = donor_pair_fixture()?;
        let prepared = g.prepare_native()?;
        let out = g.forward_prepared_coefficients_only(&prepared, &state)?;
        let ids = [4, 4, 5];
        let copy = [0, 0, 0];
        let trace = red.reduce_trace(&out.scores_q24, &ids, &copy)?;
        let raw = Var::from_vec(vec![0f32; 3], 3, &Device::Cpu)?;
        let direct = vocabulary_marginal_loss(&trace, &out.raw_scores, Some(raw.as_tensor()), 4)?;
        let utility = zero_forward_donor_loss(raw.as_tensor(), &[0., 0., -0.4])?;
        let weight = 1. / 15.;
        let plain = (&direct * weight)?;
        let corrected = ((&direct + &utility)? * weight)?;
        assert_eq!(plain.to_scalar::<f32>()?, corrected.to_scalar::<f32>()?);
        let a = plain.backward()?;
        let b = corrected.backward()?;
        let u = (&utility * weight)?.backward()?;
        let ag = a
            .get(g.unary.as_tensor())
            .ok_or_else(|| bad("factual unary missing"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        let bg = b
            .get(g.unary.as_tensor())
            .ok_or_else(|| bad("corrected unary missing"))?
            .flatten_all()?
            .to_vec1::<f32>()?;
        assert_eq!(ag, bg);
        let ac = a
            .get(raw.as_tensor())
            .ok_or_else(|| bad("direct Copy missing"))?
            .to_vec1::<f32>()?;
        let bc = b
            .get(raw.as_tensor())
            .ok_or_else(|| bad("corrected Copy missing"))?
            .to_vec1::<f32>()?;
        let uc = u
            .get(raw.as_tensor())
            .ok_or_else(|| bad("indirect Copy missing"))?
            .to_vec1::<f32>()?;
        for ((a, b), u) in ac.iter().zip(bc).zip(uc) {
            assert!((b - a - u).abs() < 1e-7);
        }
        assert!(zero_forward_donor_loss(raw.as_tensor(), &[0., f32::NAN, 0.]).is_err());
        assert!(zero_forward_donor_loss(raw.as_tensor(), &[0.]).is_err());
        let same = zero_forward_donor_loss(raw.as_tensor(), &[0.; 3])?;
        assert_eq!(same.to_scalar::<f32>()?, 0.);
        assert!(same
            .backward()?
            .get(raw.as_tensor())
            .ok_or_else(|| bad("zero donor gradient missing"))?
            .to_vec1::<f32>()?
            .iter()
            .all(|x| *x == 0.));
        assert_eq!(native.vocab_size(), 8);
        Ok(())
    }
    #[test]
    fn complete_donor_pool_keeps_clipping_aliases_and_rejects_unadmitted_target() -> Result<()> {
        let (native, _, state, _, mut red) = donor_pair_fixture()?;
        let mut raw = vec![0; native.vocab_size()];
        native.score_into(&state, &mut raw, &mut Default::default())?;
        let u = vec![0; raw.len()];
        let ids = [4, 4, 5];
        let copy = [9 << 24, 9 << 24, -9 << 24];
        let loss = forced_pool_loss(&mut red, &raw, &u, &ids, &copy, 4)?;
        let trace = red.reduce_trace(&raw, &ids, &copy)?;
        assert_eq!(loss.1, trace.token_masses[4].weight_q31);
        assert_eq!(loss.2, trace.summary.total_weight_q31);
        assert_eq!(trace.summary.clipped_high_actions, 2);
        assert_eq!(trace.summary.clipped_low_actions, 1);
        assert!(forced_pool_loss(&mut red, &raw, &u, &ids, &copy, 100).is_err());
        assert!(forced_pool_loss(&mut red, &raw, &u, &ids[..1], &copy, 4).is_err());
        Ok(())
    }
    #[test]
    fn full_pool_credit_rejects_retained_state_tangent_science_and_legacy_defaults_remain(
    ) -> Result<()> {
        assert_eq!(
            serde_json::from_str::<DonorCredit>("\"state_tangent\"")?,
            DonorCredit::default()
        );
        assert!(validate_credit_recovery(DonorCredit::FullPoolUtility, true).is_err());
        validate_credit_recovery(DonorCredit::StateTangent, true)?;
        validate_credit_recovery(DonorCredit::FullPoolUtility, false)?;
        validate_inherited_credit(
            DonorCredit::StateTangent,
            DonorCredit::StateTangent,
            &json!({}),
        )?;
        validate_inherited_credit(
            DonorCredit::StateTangent,
            DonorCredit::StateTangent,
            &json!({"donor_credit":"state_tangent"}),
        )?;
        assert!(validate_inherited_credit(
            DonorCredit::StateTangent,
            DonorCredit::FullPoolUtility,
            &json!({})
        )
        .is_err());
        assert!(validate_inherited_credit(
            DonorCredit::StateTangent,
            DonorCredit::StateTangent,
            &json!({"donor_credit":"full_pool_utility"})
        )
        .is_err());
        assert!(validate_inherited_credit(
            DonorCredit::StateTangent,
            DonorCredit::StateTangent,
            &json!({"donor_credit":"unknown"})
        )
        .is_err());
        assert_eq!(policy_for(DonorCredit::StateTangent), policy());
        assert_ne!(policy_for(DonorCredit::FullPoolUtility), policy());
        assert!(serde_json::from_str::<DonorCredit>("\"unknown\"").is_err());
        Ok(())
    }
    #[test]
    fn rebound_prefix_publication_preserves_intermediate_and_failed_save() -> Result<()> {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "coupled-prefix-publication-{}-{stamp}",
            std::process::id()
        ));
        fs::create_dir(&root)?;
        fs::create_dir(root.join("prefix"))?;
        fs::write(root.join("prefix/master"), b"original-fractional-master")?;
        // Reproduce the old collision with an exclusive writer.
        assert!(fs::create_dir(root.join("prefix")).is_err());
        publish_rebound_prefix(&root, |path| {
            fs::create_dir(path)?;
            fs::write(path.join("master"), b"rebound-fractional-master")?;
            Ok(())
        })?;
        assert_eq!(
            fs::read(root.join("prefix/master"))?,
            b"rebound-fractional-master"
        );
        assert_eq!(
            fs::read(root.join("prefix-before-cue-rebind/master"))?,
            b"original-fractional-master"
        );
        assert!(
            publish_rebound_prefix(&root, |_| Err(bad("must not replace retained output")))
                .is_err()
        );
        assert_eq!(
            fs::read(root.join("prefix/master"))?,
            b"rebound-fractional-master"
        );
        let failed = root.join("failed");
        fs::create_dir(&failed)?;
        fs::create_dir(failed.join("prefix"))?;
        fs::write(failed.join("prefix/master"), b"original-fractional-master")?;
        assert!(publish_rebound_prefix(&failed, |path| {
            fs::create_dir(path)?;
            fs::write(path.join("partial"), b"preserved-partial-write")?;
            Err(bad("injected save failure"))
        })
        .is_err());
        assert_eq!(
            fs::read(failed.join("prefix/master"))?,
            b"original-fractional-master"
        );
        assert_eq!(
            fs::read(failed.join("prefix-rebound/partial"))?,
            b"preserved-partial-write"
        );
        fs::remove_dir_all(&root)?;
        Ok(())
    }
    #[test]
    fn completed_dual_master_recovery_preserves_fractional_noops_and_rejects_stale_epoch(
    ) -> Result<()> {
        let pm = vec![0.139f32; COUNT];
        let gm = vec![-0.139f32; COUNT];
        let mut records = Vec::new();
        let mut epoch = 0;
        for (family, master) in [(PREFIX, &pm), (GENERATE, &gm)] {
            for index in 0..COUNT {
                let before = epoch;
                let code = if index == 1 {
                    Some(if family == PREFIX { -2 } else { 7 })
                } else {
                    None
                };
                if code.is_some() {
                    epoch += 1;
                }
                records.push(SavedCoordinate {
                    family: family.into(),
                    index,
                    order: records.len(),
                    original_master_bits: master[index].to_bits(),
                    incumbent_epoch: before,
                    epoch_after: epoch,
                    selected: SavedSelected {
                        status: if code.is_some() {
                            "committed"
                        } else if index == 0 && family == PREFIX {
                            "noop"
                        } else {
                            "unchanged"
                        }
                        .into(),
                        code,
                    },
                });
            }
        }
        let summary = json!({"coordinates":1920,"accepted_epoch":2,"accepted_prefix":1,
            "accepted_generate":1,"evaluated_alternatives":14292});
        let (p, g) = restored_final_masters(&records, &summary, &pm, &gm)?;
        assert_eq!(p[0].to_bits(), pm[0].to_bits());
        assert_eq!(g[0].to_bits(), gm[0].to_bits());
        assert_eq!(p[1], -0.5);
        assert_eq!(g[1], 1.75);
        records[0].selected.code = Some(0);
        assert!(restored_final_masters(&records, &summary, &pm, &gm).is_err());
        records[0].selected.code = None;
        records[0].epoch_after = 1;
        assert!(restored_final_masters(&records, &summary, &pm, &gm).is_err());
        records[0].epoch_after = 0;
        records[0].selected.status = "unknown".into();
        assert!(restored_final_masters(&records, &summary, &pm, &gm).is_err());
        records[0].selected.status = "noop".into();
        records[960].selected.status = "noop".into();
        assert!(restored_final_masters(&records, &summary, &pm, &gm).is_err());
        records[960].selected.status = "unchanged".into();
        records[960].incumbent_epoch = 0;
        assert!(restored_final_masters(&records, &summary, &pm, &gm).is_err());
        records[960].incumbent_epoch = 1;
        records[2].original_master_bits = 0;
        assert!(restored_final_masters(&records, &summary, &pm, &gm).is_err());
        Ok(())
    }
    #[test]
    fn frozen_u_initializes_against_original_generate_then_rebinds_changed_generate() -> Result<()>
    {
        use uor_r4_core::native_geometric::learner::integrated_attention::geometry::EnergyTables;
        use uor_r4_integer::geometric_source_actions::SourceActionBinding;
        const TOK:&[u8]=br#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4,"b":5},"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>","special":true},{"id":1,"content":"<|eos|>","special":true},{"id":2,"content":"<|unk|>","special":true}]}"#;
        let binding = SourceActionBinding::new(TOK)?;
        let nb: NativeArtifactBinding = serde_json::from_value(
            json!({"metadata_sha256":"a".repeat(64),
            "identity":{"tokenizer_sha256":binding.tokenizer_sha256(),"parent_checkpoint_manifest_sha256":"b".repeat(64),
                "parent_model_sha256":"c".repeat(64),"parent_config_sha256":"d".repeat(64)}}),
        )?;
        let prototypes = (0..binding.vocab_size())
            .flat_map(|i| vec![(i + 1) as u8; 8])
            .collect::<Vec<_>>();
        let mut energy = EnergyTables::zeroed(8, vec![])?;
        let original = NativeGeometricGenerate::compile(
            &binding,
            8,
            &prototypes,
            &vec![0; binding.vocab_size().div_ceil(2)],
            energy.clone(),
        )?;
        let field =
            NativeContinuationField::compile_shared_action(&nb, &original, &vec![0; 8 * 60])?;
        energy.set_unary(0, 1, 1)?;
        let changed = NativeGeometricGenerate::compile(
            &binding,
            8,
            &prototypes,
            &vec![0; binding.vocab_size().div_ceil(2)],
            energy,
        )?;
        assert!(
            ContinuationLearningWeights::from_native(&field, &changed, &binding, &Device::Cpu)
                .is_err()
        );
        let weights =
            ContinuationLearningWeights::from_native(&field, &original, &binding, &Device::Cpu)?;
        let rebound = weights.export_native(&nb, &changed)?;
        assert_eq!(rebound.packed_unary(), field.packed_unary());
        assert_eq!(rebound.metadata().generate_metadata, *changed.metadata());
        assert!(NativeContinuationField::from_bytes(&rebound.to_bytes()?, &nb, &changed).is_ok());
        Ok(())
    }
    #[test]
    fn mixed_rank_keeps_fractional_prefix_noop_and_signed_zero() -> Result<()> {
        let mut pm = vec![0.139f32; COUNT];
        pm[0] = -1.75;
        let mut pg = vec![0f32; COUNT];
        pg[0] = 1.;
        let gm = vec![0f32; COUNT];
        let gg = vec![-0f32; COUNT];
        let rows = order(&pm, &pg, &gm, &gg)?;
        let p = rows
            .iter()
            .find(|r| r.family == PREFIX && r.index == 0)
            .ok_or_else(|| bad("rank row absent"))?;
        assert_eq!(p.status, "saturated");
        assert_eq!(p.actual_master_delta, 0.);
        let p = rows
            .iter()
            .find(|r| r.family == PREFIX && r.index == 1)
            .ok_or_else(|| bad("rank row absent"))?;
        assert_eq!(p.status, "zero_gradient");
        assert_eq!(p.original_master.to_bits(), pm[1].to_bits());
        let g = rows
            .iter()
            .find(|r| r.family == GENERATE && r.index == 0)
            .ok_or_else(|| bad("rank row absent"))?;
        assert_eq!(g.rank_code, 1); // -0*positive is -0, before +0; then lowest positive q.
        assert_eq!(rows.len(), 1920);
        Ok(())
    }
    #[test]
    fn prefix_lane_major_alias_keys_do_not_transpose_occurrences() -> Result<()> {
        let bins = (0..8)
            .map(|l| vec![l as u16, 119 - l as u16])
            .collect::<Vec<_>>();
        let keys = (0..2)
            .map(|j| {
                (0..8)
                    .map(|l| Some(l * 120 + usize::from(bins[l][j])))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let old = vec![0f32; COUNT];
        let mut next = old.clone();
        next[3 * 120 + 3] = 0.25;
        let scores = shared::staged_base(&[9, 9], &keys, &old, &next)?;
        assert_eq!(scores, vec![9 + (1 << 22), 9]);
        assert_eq!(earliest(&[17, 17, 16])?, 0);
        Ok(())
    }
    #[test]
    fn late_prefix_replacement_discards_old_pending_identity() -> Result<()> {
        use uor_r4_integer::geometric_source_actions::SourceActionBinding;
        const TOK:&[u8]=br#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4,"b":5},"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>","special":true},{"id":1,"content":"<|eos|>","special":true},{"id":2,"content":"<|unk|>","special":true}]}"#;
        let exp = (0..uor_r4_integer::geometric_read::EXP_TABLE_LEN)
            .flat_map(|i| {
                (((-(i as f64) / 256.).exp() * (1u64 << 31) as f64).round() as u32).to_le_bytes()
            })
            .collect::<Vec<_>>();
        let mut red = NativeVocabularyActions::new(SourceActionBinding::new(TOK)?, &exp)?;
        let mut g = vec![-100 * (1 << 24); 6];
        g[4] = 0;
        g[5] = -(1 << 20);
        let mut old =
            red.prepare_generate_patch_cache(g.clone(), vec![4, 4], vec![-3 * (1 << 24); 2])?;
        let pending = red.evaluate_generate_patch(&old, &[(5, 2 * (1 << 20))])?;
        let replacement =
            red.prepare_generate_patch_cache(g, vec![4, 4], vec![-3 * (1 << 24); 2])?;
        assert_eq!(replacement.summary().chosen_token_id, 4);
        // A rejected staged replacement leaves the incumbent and revision intact.
        drop(replacement);
        assert_eq!(old.revision(), 0);
        red.commit_generate_patch_batch(std::slice::from_mut(&mut old), vec![(0, pending)])?;
        assert_eq!(old.summary().chosen_token_id, 5);
        Ok(())
    }

    #[test]
    fn replacement_identity_rejects_late_old_pending_before_any_cache_commit() -> Result<()> {
        use uor_r4_integer::geometric_source_actions::SourceActionBinding;
        const TOK:&[u8]=br#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4,"b":5},"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>","special":true},{"id":1,"content":"<|eos|>","special":true},{"id":2,"content":"<|unk|>","special":true}]}"#;
        let exp = (0..uor_r4_integer::geometric_read::EXP_TABLE_LEN)
            .flat_map(|i| {
                (((-(i as f64) / 256.).exp() * (1u64 << 31) as f64).round() as u32).to_le_bytes()
            })
            .collect::<Vec<_>>();
        let mut red = NativeVocabularyActions::new(SourceActionBinding::new(TOK)?, &exp)?;
        let g = vec![-4 << 24, -4 << 24, -4 << 24, -4 << 24, 0, -1 << 20];
        let mut caches = (0..GUARDS)
            .map(|_| red.prepare_generate_patch_cache(g.clone(), vec![4, 4], vec![-3 << 24; 2]))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let first = red.evaluate_generate_patch(&caches[0], &[(5, 2 << 20)])?;
        let stale = red.evaluate_generate_patch(&caches[GUARDS - 1], &[(5, 2 << 20)])?;
        // Prefix replacement can have identical scores and revision0 but a
        // different opaque identity; an old pending result must still fail.
        caches[GUARDS - 1] =
            red.prepare_generate_patch_cache(g.clone(), vec![4, 4], vec![-3 << 24; 2])?;
        assert!(red
            .commit_generate_patch_batch(&mut caches, vec![(0, first), (GUARDS - 1, stale)])
            .is_err());
        assert!(caches.iter().all(|c| c.revision() == 0
            && c.generate_scores() == g
            && c.summary().chosen_token_id == 4));
        Ok(())
    }

    #[test]
    fn changed_donor_post_rebuilds_current_unary_incidence_before_next_generate_patch() -> Result<()>
    {
        use uor_r4_core::native_geometric::learner::integrated_attention::geometry::EnergyTables;
        use uor_r4_integer::geometric_source_actions::SourceActionBinding;
        const TOK:&[u8]=br#"{"pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},"model":{"type":"BPE","vocab":{"<|bos|>":0,"<|eos|>":1,"<|unk|>":2,".":3,"a":4,"b":5},"merges":[]},"added_tokens":[{"id":0,"content":"<|bos|>","special":true},{"id":1,"content":"<|eos|>","special":true},{"id":2,"content":"<|unk|>","special":true}]}"#;
        let binding = SourceActionBinding::new(TOK)?;
        let exp = (0..uor_r4_integer::geometric_read::EXP_TABLE_LEN)
            .flat_map(|i| {
                (((-(i as f64) / 256.).exp() * (1u64 << 31) as f64).round() as u32).to_le_bytes()
            })
            .collect::<Vec<_>>();
        let mut red = NativeVocabularyActions::new(binding.clone(), &exp)?;
        let legal = red.legal_token_ids().to_vec();
        let prototypes = (0..binding.vocab_size())
            .flat_map(|i| vec![(i + 1) as u8; 8])
            .collect::<Vec<_>>();
        let mut energy = EnergyTables::zeroed(8, vec![])?;
        let zero = NativeGeometricGenerate::compile(
            &binding,
            8,
            &prototypes,
            &vec![0; binding.vocab_size().div_ceil(2)],
            energy.clone(),
        )?;
        let old = shared::codes(&json!(vec![1; 8]))?;
        let mut new = old.clone();
        new[0] = shared::codes(&json!(vec![2; 8]))?[0];
        assert_eq!(earliest(&[9, 8])?, 0);
        assert_eq!(earliest(&[8, 9])?, 1);
        let old_inc = RowIncidence::new(&zero, &old, &legal)?;
        let mut keys = [0u32; 8];
        zero.factor_incidence_into(&new, 5, &mut keys, &mut Default::default())?;
        let key = keys[0] as usize;
        energy.set_unary(0, (key % 120) as u8, 1)?;
        let current = NativeGeometricGenerate::compile(
            &binding,
            8,
            &prototypes,
            &vec![0; binding.vocab_size().div_ceil(2)],
            energy.clone(),
        )?;
        let new_inc = RowIncidence::new(&current, &new, &legal)?;
        assert!(new_inc.matching(key).contains(&5));
        assert!(!old_inc.matching(key).contains(&5));
        let mut scores = vec![0; binding.vocab_size()];
        current.score_into(&new, &mut scores, &mut Default::default())?;
        let frozen_u = vec![0i64, 0, 0, 0, 1 << 20, -1 << 20];
        for (g, u) in scores.iter_mut().zip(&frozen_u) {
            *g += u;
        }
        let mut cache =
            red.prepare_generate_patch_cache(scores, vec![4, 4], vec![-3 << 24, -2 << 24])?;
        let atoms = new_inc
            .matching(key)
            .iter()
            .map(|&t| {
                (
                    u32::from(t),
                    cache.generate_scores()[t as usize] + 4 * (1 << 20),
                )
            })
            .collect::<Vec<_>>();
        let pending = red.evaluate_generate_patch(&cache, &atoms)?;
        energy.set_unary(0, (key % 120) as u8, 5)?;
        let next = NativeGeometricGenerate::compile(
            &binding,
            8,
            &prototypes,
            &vec![0; binding.vocab_size().div_ceil(2)],
            energy,
        )?;
        let mut exact = vec![0; binding.vocab_size()];
        next.score_into(&new, &mut exact, &mut Default::default())?;
        for (g, u) in exact.iter_mut().zip(&frozen_u) {
            *g += u;
        }
        let trace = red.reduce_trace(&exact, cache.copy_token_ids(), cache.copy_scores())?;
        assert_eq!(
            pending.summary().chosen_token_id,
            trace.summary.chosen_token_id
        );
        assert_eq!(
            pending.summary().total_weight_q31,
            trace.summary.total_weight_q31
        );
        for m in &trace.token_masses {
            assert_eq!(pending.token_mass(&cache, m.token_id)?, m.weight_q31);
        }
        red.commit_generate_patch_batch(std::slice::from_mut(&mut cache), vec![(0, pending)])?;
        assert_eq!(cache.generate_scores(), exact);
        Ok(())
    }
    #[test]
    fn worst_case_journal_and_export_fit_declared_report_without_dropping_guards() -> Result<()> {
        let journal = journal_upper_bound()?;
        let report =
            (535837u64 + 16384) * 391 + journal + 57780353 + 48 * 1024 * 1024 + 32 * 1024 * 1024;
        println!("journal_upper_bound={journal} report_upper_bound={report}");
        assert!(report + 1024 * 1024 < 512 * 1024 * 1024);
        Ok(())
    }
    #[test]
    fn sealed_supplement_alias_count_uses_flat_frame_and_rejects_missing_identity() -> Result<()> {
        let flat = br#"{"schema":"uor-r4.native-reached-prefix-frame/3","copy_ids":[617,617,2097],"bank_trace":{"ignored":"retained elsewhere"}}"#;
        assert_eq!(flat_saved_alias_count(flat.as_slice())?, 3);
        assert!(flat_saved_alias_count(
            br#"{"schema":"uor-r4.native-reached-prefix-frame/3","native":{"copy_ids":[617]}}"#
                .as_slice()
        )
        .is_err());
        assert!(flat_saved_alias_count(
            br#"{"schema":"uor-r4.native-reached-prefix-frame/3"}"#.as_slice()
        )
        .is_err());
        Ok(())
    }

    #[test]
    fn compact_prefix_keys_survive_trace_discard_and_reject_mask_or_lane_erasure() -> Result<()> {
        let keys = vec![(0..8).map(|lane| Some(lane * 120 + 17)).collect::<Vec<_>>(); 2];
        assert_eq!(validated_prefix_keys(2, &keys)?, keys.as_slice());
        let mut masked = keys.clone();
        masked[1][7] = None;
        assert!(validated_prefix_keys(2, &masked).is_err());
        let mut wrong_lane = keys.clone();
        wrong_lane[1][7] = Some(17);
        assert!(validated_prefix_keys(2, &wrong_lane).is_err());
        assert!(validated_prefix_keys(3, &keys).is_err());
        let scores = shared::staged_base(
            &[10, 11],
            &keys,
            &vec![0.; 960],
            &(0..960)
                .map(|i| if i == 7 * 120 + 17 { 0.25 } else { 0. })
                .collect::<Vec<_>>(),
        )?;
        assert_eq!(scores, vec![10 + (1 << 22), 11 + (1 << 22)]);
        Ok(())
    }
    #[test]
    fn full_cache_bounds_charge_both_prefix_cache_and_incidence() -> Result<()> {
        let v = capacity_projection(512, 4096)?;
        assert!(
            v["prefix_all391_replacement_stage"]
                .as_u64()
                .ok_or_else(|| bad("bound absent"))?
                > 70_000_000
        );
        assert!(
            v["cache_upper_bound"]
                .as_u64()
                .ok_or_else(|| bad("bound absent"))?
                < CACHE_CAP as u64
        );
        Ok(())
    }
}
