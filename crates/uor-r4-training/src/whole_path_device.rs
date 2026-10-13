//! Device-resident numerical producer for the bound whole-path graph.
//! Host work is artifact admission and exact-store transactions, not Context
//! replay. CUDA execution is explicitly unqualified until device parity runs.
use super::*;
#[path = "whole_path_device_ops.rs"]
mod ops;
pub(super) fn prototype_choice(_probability: &Tensor, _u: f64) -> Result<usize> {
    Err(invalid(
        "retired probability prototype writer; use the shared native vector channel",
    ))
}

const Q25: f64 = 33554432.;
pub struct DeviceChoice {
    pub index: Tensor,
    pub log_probability: Tensor,
}

#[derive(Clone, Deserialize)]
struct CompilerSettings {
    relations: Vec<String>,
    feature_mode: crate::geometric_turn_compiler::FeatureMode,
    max_tokens: usize,
    max_words: usize,
    max_value_words: usize,
}
impl CompilerSettings {
    fn config(&self) -> FitConfig {
        FitConfig {
            feature_mode: self.feature_mode,
            max_tokens: self.max_tokens,
            max_words: self.max_words,
            max_value_words: self.max_value_words,
            ..FitConfig::default()
        }
    }
}
struct DeviceCompiler<'a> {
    producer: Arc<Mutex<WholePathCudaProducer>>,
    tokenizer: &'a ByteBpeTokenizer,
    bytes: Vec<u8>,
    identity: crate::stack_grounded_session::CompilerIdentity,
    settings: CompilerSettings,
}
impl crate::stack_grounded_session::TurnCompiler for DeviceCompiler<'_> {
    fn identity(&self) -> &crate::stack_grounded_session::CompilerIdentity {
        &self.identity
    }
    fn artifact_bytes(&self) -> &[u8] {
        &self.bytes
    }
    fn compile(
        &self,
        source: &str,
    ) -> std::result::Result<CompiledAction, crate::stack_grounded_session::GroundedSessionError>
    {
        self.producer
            .lock()
            .map_err(|_| {
                crate::stack_grounded_session::GroundedSessionError::Compiler(
                    "CUDA producer lock poisoned".into(),
                )
            })?
            .compile_action(
                source,
                self.tokenizer,
                &self.settings.config(),
                self.settings.relations.len(),
            )
            .map_err(|e| {
                crate::stack_grounded_session::GroundedSessionError::Compiler(e.to_string())
            })
    }
}
struct DeviceRoutePolicy {
    producer: Arc<Mutex<WholePathCudaProducer>>,
}
impl RouteDecisions for DeviceRoutePolicy {
    fn encode(&mut self, tokenizer: &ByteBpeTokenizer, source: &str) -> Result<Vec<u8>> {
        let mut p = self
            .producer
            .lock()
            .map_err(|_| invalid("device route lock"))?;
        let encoded = p.endpoint(&tokenizer.encode(source))?;
        // Retained IDs are bounded record/index metadata. No score arrays,
        // gradients, observations or native host Context replay cross here.
        Ok(encoded
            .to_vec1::<u32>()?
            .into_iter()
            .map(|v| v as u8)
            .collect())
    }
    fn key(&mut self, states: &[u8]) -> Result<usize> {
        let mut p = self
            .producer
            .lock()
            .map_err(|_| invalid("device key lock"))?;
        let rows = Tensor::from_vec(
            states.iter().map(|&s| u32::from(s)).collect::<Vec<_>>(),
            (1, states.len()),
            &p.device,
        )?;
        let logits = p.head(
            "route.key",
            &rows,
            &Tensor::ones(rows.shape(), DType::F32, &p.device)?,
            240,
        )?;
        Ok(p.choose(&logits, true)?.index.to_vec1::<u32>()?[0] as usize)
    }
    fn families(
        &mut self,
        states: &[u8],
        profiles: &[[i32; 9]; 15],
        count: usize,
    ) -> Result<Vec<usize>> {
        let mut p = self
            .producer
            .lock()
            .map_err(|_| invalid("device family lock"))?;
        let states = u32_indices(&p.device, states.iter().map(|&s| u32::from(s)).collect())?;
        let profiles_device = Tensor::from_vec(
            profiles
                .iter()
                .flatten()
                .map(|&v| v as f64)
                .collect::<Vec<_>>(),
            (15, 9),
            &p.device,
        )?;
        let scores = p.family_scores(&states, &profiles_device)?;
        let mut alive = (0..15).filter(|&i| profiles[i][0] > 0).collect::<Vec<_>>();
        let mut chosen = Vec::new();
        while !alive.is_empty() && chosen.len() < count {
            let logits = scores.index_select(
                &u32_indices(&p.device, alive.iter().map(|&i| i as u32).collect())?,
                0,
            )?;
            let index = p.choose(&logits, true)?.index.to_vec1::<u32>()?[0] as usize;
            chosen.push(alive.remove(index));
        }
        Ok(chosen)
    }
    fn rank(&mut self, query: &[u8], candidates: &[RouteRecord]) -> Result<usize> {
        let mut p = self
            .producer
            .lock()
            .map_err(|_| invalid("device rank lock"))?;
        let query = u32_indices(&p.device, query.iter().map(|&s| u32::from(s)).collect())?;
        let mut scores = Vec::new();
        for r in candidates {
            let source = u32_indices(&p.device, r.states.iter().map(|&s| u32::from(s)).collect())?;
            let row = Tensor::cat(&[&query, &source, &p.relative_codes(&query, &source)?], 0)?
                .unsqueeze(0)?;
            scores.push(
                p.head(
                    "route.rank",
                    &row,
                    &Tensor::ones(row.shape(), DType::F32, &p.device)?,
                    1,
                )?
                .sum_all()?,
            );
        }
        Ok(p.choose(&Tensor::stack(&scores, 0)?, true)?
            .index
            .to_vec1::<u32>()?[0] as usize)
    }
}

/// CUDA numerical graph with the SAME native compiler identity, exact store,
/// original source witnesses, bounded E8 index and atomic commit machinery.
/// Only bounded compiler/route/output metadata crosses the device boundary.
pub struct CudaWholePathSession<'a> {
    owner: Arc<()>,
    revision: &'a WholePathRevision,
    native: NativeGroundedSession<'a, DeviceCompiler<'a>>,
    producer: Arc<Mutex<WholePathCudaProducer>>,
    tokenizer: &'a ByteBpeTokenizer,
    index: RouteIndex,
    episode: EpisodeSpec,
    failed: bool,
    pending: bool,
    journal: Vec<ReplayEvent>,
    teacher: Option<Tensor>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DeviceCheckpoint {
    schema: String,
    revision: String,
    parameters: String,
    geometry: String,
    sampling: Sampling,
    episode: EpisodeSpec,
    events: Vec<ReplayEvent>,
    rng: u64,
    decisions: usize,
    decision_sha256: String,
    native_sha256: String,
    index: RouteIndex,
}
#[derive(Clone)]
pub struct DeviceEpisodeCredit {
    producer: Arc<Mutex<WholePathCudaProducer>>,
    loss: Tensor,
    advantage: Tensor,
    decisions: usize,
}
impl DeviceEpisodeCredit {
    pub fn objective(&self) -> &Tensor {
        &self.loss
    }
}
/// One immutable revision and codebook draw, independent bounded stores and
/// RNG streams, and one equal-episode objective over the same original Vars.
pub struct CudaWholePathBatch<'a> {
    global: WholePathCudaProducer,
    sessions: Vec<CudaWholePathSession<'a>>,
}
impl<'a> CudaWholePathBatch<'a> {
    pub fn new(
        revision: &'a WholePathRevision,
        tokenizer: &'a ByteBpeTokenizer,
        tokenizer_bytes: &[u8],
        params: BTreeMap<String, Var>,
        sampling: Sampling,
        episodes: &[(u64, EpisodeSpec)],
    ) -> Result<Self> {
        let global = WholePathCudaProducer::new(revision, params, sampling)?;
        Self::from_global(revision, tokenizer, tokenizer_bytes, global, episodes)
    }
    pub(super) fn from_global(
        revision: &'a WholePathRevision,
        tokenizer: &'a ByteBpeTokenizer,
        tokenizer_bytes: &[u8],
        global: WholePathCudaProducer,
        episodes: &[(u64, EpisodeSpec)],
    ) -> Result<Self> {
        if !(2..=64).contains(&episodes.len())
            || episodes.iter().map(|e| e.0).collect::<BTreeSet<_>>().len() != episodes.len()
        {
            return Err(invalid("device batch requires 2..64 distinct seeds"));
        }
        let mut sessions = Vec::new();
        for (seed, episode) in episodes {
            sessions.push(CudaWholePathSession::from_producer(
                revision,
                tokenizer,
                tokenizer_bytes,
                global.fork_episode(*seed),
                episode.clone(),
            )?);
        }
        Ok(Self { global, sessions })
    }
    pub fn episode_mut(&mut self, index: usize) -> Result<&mut CudaWholePathSession<'a>> {
        self.sessions
            .get_mut(index)
            .ok_or_else(|| invalid("device batch episode absent"))
    }
    pub fn recover_episode(
        &mut self,
        index: usize,
        tokenizer_bytes: &[u8],
        bytes: &[u8],
        trusted_sha: &str,
    ) -> Result<()> {
        let old = self
            .sessions
            .get(index)
            .ok_or_else(|| invalid("device recovery episode absent"))?;
        if !old.journal.is_empty() || old.pending || old.failed {
            return Err(invalid("device recovery requires a fresh episode slot"));
        }
        let saved = CudaWholePathSession::read_checkpoint(old.revision, bytes, trusted_sha)?;
        let p = old
            .producer
            .lock()
            .map_err(|_| invalid("device recovery lock"))?;
        if p.sampling != saved.sampling || old.episode != saved.episode {
            return Err(invalid("device recovery episode ownership differs"));
        }
        let producer = self.global.fork_episode(p.sampling.seed);
        drop(p);
        let recovered = CudaWholePathSession::replay_checkpoint(
            old.revision,
            old.tokenizer,
            tokenizer_bytes,
            producer,
            saved,
            bytes,
        )?;
        self.sessions[index] = recovered;
        Ok(())
    }
    pub fn objective(&self, credits: &[DeviceEpisodeCredit]) -> Result<Tensor> {
        if credits.len() != self.sessions.len() {
            return Err(invalid("device batch credit count differs"));
        }
        self.global.validate_masters()?;
        for (credit, session) in credits.iter().zip(&self.sessions) {
            if !Arc::ptr_eq(&credit.producer, &session.producer)
                || session.pending
                || (session.failed && session.teacher.is_none())
            {
                return Err(invalid("device batch credit episode/order differs"));
            }
            let p = session
                .producer
                .lock()
                .map_err(|_| invalid("device batch lock"))?;
            p.validate_masters()?;
            if p.decisions != credit.decisions {
                return Err(invalid("device episode changed after credit capture"));
            }
        }
        let mean = Tensor::stack(
            &credits.iter().map(|c| c.loss.clone()).collect::<Vec<_>>(),
            0,
        )?
        .mean_all()?;
        if self.global.logs.is_empty() {
            return Ok(mean);
        }
        let advantage = Tensor::stack(
            &credits
                .iter()
                .map(|c| c.advantage.clone())
                .collect::<Vec<_>>(),
            0,
        )?
        .mean_all()?
        .detach();
        let global = Tensor::stack(&self.global.logs, 0)?
            .sum_all()?
            .broadcast_mul(&advantage)?;
        Ok((&mean + (&global - global.detach())?)?)
    }
}
impl<'a> CudaWholePathSession<'a> {
    pub fn new(
        revision: &'a WholePathRevision,
        tokenizer: &'a ByteBpeTokenizer,
        tokenizer_bytes: &[u8],
        params: BTreeMap<String, Var>,
        sampling: Sampling,
        episode: EpisodeSpec,
    ) -> Result<Self> {
        let producer = WholePathCudaProducer::new(revision, params, sampling)?;
        Self::from_producer(revision, tokenizer, tokenizer_bytes, producer, episode)
    }
    pub(super) fn from_producer(
        revision: &'a WholePathRevision,
        tokenizer: &'a ByteBpeTokenizer,
        tokenizer_bytes: &[u8],
        producer: WholePathCudaProducer,
        episode: EpisodeSpec,
    ) -> Result<Self> {
        use crate::stack_grounded_session::TurnCompiler;
        let original = NativeTurnCompiler::load(
            &revision.compiler,
            &revision.manifest.files["compiler.bin"],
            &revision.encoder,
            tokenizer,
            tokenizer_bytes,
        )?;
        let settings: CompilerSettings = serde_json::from_slice(&revision.compiler)?;
        let producer = Arc::new(Mutex::new(producer));
        let compiler = DeviceCompiler {
            producer: producer.clone(),
            tokenizer,
            bytes: revision.compiler.clone(),
            identity: original.identity().clone(),
            settings,
        };
        let native = NativeGroundedSession::new(
            compiler,
            revision.generator(None)?,
            tokenizer,
            tokenizer_bytes,
            episode.scope.clone(),
            episode.limits.clone(),
            episode.lineage,
            episode.capacity,
        )?;
        Ok(Self {
            revision,
            native,
            owner: Arc::new(()),
            producer,
            tokenizer,
            index: RouteIndex::new(revision.manifest.limits.clone()),
            episode,
            failed: false,
            pending: false,
            journal: vec![],
            teacher: None,
        })
    }
    pub fn store(&self) -> &StackStore {
        self.native.store()
    }
    pub fn transport_routes(&mut self, action: u8) -> Result<()> {
        if self.failed || self.pending || self.journal.len() >= 4096 {
            return Err(invalid(
                "device frame transport requires idle bounded session",
            ));
        }
        self.producer
            .lock()
            .map_err(|_| invalid("device transport lock"))?
            .validate_masters()?;
        self.index.transport(action)?;
        self.journal.push(ReplayEvent::Transport { action });
        Ok(())
    }
    pub fn sampled_checkpoint(&self) -> Result<Vec<u8>> {
        if self.pending || (self.failed && self.teacher.is_none()) {
            return Err(invalid("device checkpoint requires complete boundary"));
        }
        let p = self
            .producer
            .lock()
            .map_err(|_| invalid("device checkpoint lock"))?;
        p.validate_masters()?;
        let saved = DeviceCheckpoint {
            schema: "uor-r4.whole-path-device-sampled/1".into(),
            revision: p.revision.clone(),
            parameters: p.parameters.clone(),
            geometry: p.geometry.clone(),
            sampling: p.sampling.clone(),
            episode: self.episode.clone(),
            events: self.journal.clone(),
            rng: p.rng,
            decisions: p.decisions,
            decision_sha256: p.decision_digest()?,
            native_sha256: sha256_bytes(&self.native.snapshot()?),
            index: self.index.clone(),
        };
        let bytes = serde_json::to_vec(&saved)?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err(invalid("device checkpoint exceeds admission"));
        }
        Ok(bytes)
    }
    pub fn recover_sampled(
        revision: &'a WholePathRevision,
        tokenizer: &'a ByteBpeTokenizer,
        tokenizer_bytes: &[u8],
        params: BTreeMap<String, Var>,
        bytes: &[u8],
        trusted_sha: &str,
    ) -> Result<Self> {
        let saved = Self::read_checkpoint(revision, bytes, trusted_sha)?;
        let producer = WholePathCudaProducer::new(revision, params, saved.sampling.clone())?;
        Self::replay_checkpoint(revision, tokenizer, tokenizer_bytes, producer, saved, bytes)
    }
    pub(super) fn read_checkpoint(
        revision: &WholePathRevision,
        bytes: &[u8],
        trusted_sha: &str,
    ) -> Result<DeviceCheckpoint> {
        if bytes.len() > 16 * 1024 * 1024 || sha256_bytes(bytes) != trusted_sha {
            return Err(invalid("device checkpoint custody differs"));
        }
        let saved: DeviceCheckpoint = serde_json::from_slice(bytes)?;
        if saved.schema != "uor-r4.whole-path-device-sampled/1"
            || saved.revision != revision.sha256
            || saved.parameters != revision.parameter_sha256()
            || saved.geometry != route_geometry_sha256()?
            || saved
                .events
                .iter()
                .filter(|e| !matches!(e, ReplayEvent::Transport { .. }))
                .count()
                > saved.episode.limits.max_turns
            || saved.events.len() > 4096
        {
            return Err(invalid("device checkpoint binding differs"));
        }
        saved.sampling.validate()?;
        Ok(saved)
    }
    pub(super) fn replay_checkpoint(
        revision: &'a WholePathRevision,
        tokenizer: &'a ByteBpeTokenizer,
        tokenizer_bytes: &[u8],
        producer: WholePathCudaProducer,
        saved: DeviceCheckpoint,
        bytes: &[u8],
    ) -> Result<Self> {
        if producer.sampling != saved.sampling {
            return Err(invalid("device checkpoint sampling differs"));
        }
        let mut session = Self::from_producer(
            revision,
            tokenizer,
            tokenizer_bytes,
            producer,
            saved.episode.clone(),
        )?;
        for event in saved.events {
            match event {
                ReplayEvent::Transport { action } => session.transport_routes(action)?,
                ReplayEvent::Turn {
                    source,
                    maximum_tokens,
                    retained_steps,
                    emitted,
                    outcome_sha256,
                } => {
                    let out = session.turn(
                        &source,
                        GenerationLimits {
                            maximum_tokens,
                            maximum_retained_steps: retained_steps,
                        },
                    )?;
                    if out.generation.generated_ids != emitted
                        || outcome_digest(&out)? != outcome_sha256
                    {
                        return Err(invalid("device replay outcome differs"));
                    }
                }
                ReplayEvent::Teacher {
                    source,
                    answer_with_eos,
                    baseline,
                    value,
                } => {
                    let prepared = session.prepare_turn(&source)?;
                    let loss =
                        session.teacher_ranking_objective(&prepared, &answer_with_eos, baseline)?;
                    if loss.to_scalar::<f32>()?.to_bits() != value.to_bits() {
                        return Err(invalid("device replay teacher value differs"));
                    }
                }
            }
        }
        if session.sampled_checkpoint()? != bytes {
            return Err(invalid("device replay decisions/RNG/store/geometry differ"));
        }
        Ok(session)
    }
    pub fn terminal_credit(&self, cost: f32, baseline: f32) -> Result<DeviceEpisodeCredit> {
        let loss = self.terminal_objective(cost, baseline)?;
        let advantage = Tensor::new(cost - baseline, loss.device())?;
        let decisions = self
            .producer
            .lock()
            .map_err(|_| invalid("device credit lock"))?
            .decisions;
        Ok(DeviceEpisodeCredit {
            producer: self.producer.clone(),
            loss,
            advantage,
            decisions,
        })
    }
    pub fn completed_teacher_credit(&self) -> Result<DeviceEpisodeCredit> {
        let Some(ReplayEvent::Teacher { baseline, .. }) = self.journal.last() else {
            return Err(invalid("device teacher incomplete"));
        };
        let loss = self
            .teacher
            .clone()
            .ok_or_else(|| invalid("device teacher graph absent"))?;
        let advantage = loss.detach().affine(1., -f64::from(*baseline))?;
        let decisions = self
            .producer
            .lock()
            .map_err(|_| invalid("device credit lock"))?
            .decisions;
        Ok(DeviceEpisodeCredit {
            producer: self.producer.clone(),
            loss,
            advantage,
            decisions,
        })
    }
    pub fn prepare_turn(&mut self, source: &str) -> Result<WholePreparedTurn> {
        if self.failed || self.pending {
            return Err(invalid("device session failed or already pending"));
        }
        let mut policy = DeviceRoutePolicy {
            producer: self.producer.clone(),
        };
        match WholePathSession::prepare_policy(
            &self.owner,
            self.revision,
            &self.native,
            self.tokenizer,
            &self.episode.scope,
            &self.index,
            source,
            &mut policy,
        ) {
            Ok(prepared) => {
                self.pending = true;
                Ok(prepared)
            }
            Err(e) => {
                self.failed = true;
                Err(e)
            }
        }
    }
    pub fn emit_prepared(
        &mut self,
        prepared: WholePreparedTurn,
        limits: GenerationLimits,
    ) -> Result<NativeTurnOutcome> {
        if self.failed
            || !self.pending
            || !Arc::ptr_eq(&self.owner, &prepared.owner)
            || prepared.revision_sha256 != self.revision.sha256
        {
            return Err(invalid("device prepared binding/state differs"));
        }
        let producer = self.producer.clone();
        let eos = self.revision.encoder.binding().eos_token_id();
        let result = self
            .native
            .emit_prepared_with(prepared.native, limits, |_, bank, limits| {
                producer
                    .lock()
                    .map_err(|_| invalid("device emitter lock"))?
                    .generate(bank, limits, eos)
            });
        self.pending = false;
        match result {
            Ok(out) => {
                self.index = prepared.index;
                self.journal.push(ReplayEvent::Turn {
                    source: prepared.source,
                    maximum_tokens: limits.maximum_tokens,
                    retained_steps: limits.maximum_retained_steps,
                    emitted: out.generation.generated_ids.clone(),
                    outcome_sha256: outcome_digest(&out)?,
                });
                Ok(out)
            }
            Err(e) => {
                self.failed = true;
                Err(e)
            }
        }
    }
    pub fn turn(&mut self, source: &str, limits: GenerationLimits) -> Result<NativeTurnOutcome> {
        let pending = self.prepare_turn(source)?;
        self.emit_prepared(pending, limits)
    }
    pub fn terminal_objective(&self, cost: f32, baseline: f32) -> Result<Tensor> {
        if self.failed
            || self.pending
            || !self
                .journal
                .iter()
                .any(|e| matches!(e, ReplayEvent::Turn { .. }))
        {
            return Err(invalid(
                "device terminal objective not at complete boundary",
            ));
        }
        let p = self
            .producer
            .lock()
            .map_err(|_| invalid("device credit lock"))?;
        p.validate_masters()?;
        p.objective(cost, baseline)
    }
    pub fn teacher_ranking_objective(
        &mut self,
        prepared: &WholePreparedTurn,
        answer: &[u32],
        baseline: f32,
    ) -> Result<Tensor> {
        if self.failed
            || !self.pending
            || !Arc::ptr_eq(&self.owner, &prepared.owner)
            || prepared.revision_sha256 != self.revision.sha256
        {
            return Err(invalid("device teacher binding/state differs"));
        }
        self.native.validate_prepared_turn(&prepared.native)?;
        let result = self
            .producer
            .lock()
            .map_err(|_| invalid("device teacher lock"))?
            .teacher_objective(
                prepared.native.bank(),
                answer,
                self.revision.encoder.binding().eos_token_id(),
                baseline,
            );
        self.failed = true;
        self.pending = false;
        if let Ok(loss) = &result {
            self.journal.push(ReplayEvent::Teacher {
                source: prepared.source.clone(),
                answer_with_eos: answer.into(),
                baseline,
                value: loss.to_scalar::<f32>()?,
            });
            self.teacher = Some(loss.clone());
        }
        result
    }
}
pub struct DeviceContext {
    pub retained: Tensor,
    pub roots: Tensor,
    pub categories: Tensor,
}
/// Every row is a physical occurrence from the admitted packet, not a unique
/// token or winning donor. Prefix rows exclude the current source token.
pub struct DeviceCopyInput<'a> {
    pub query_roots: &'a Tensor,
    pub query_categories: &'a Tensor,
    pub cue_query_roots: &'a Tensor,
    pub cue_query_categories: &'a Tensor,
    pub candidate_roots: &'a Tensor,
    pub candidate_categories: &'a Tensor,
    pub cue_roots: &'a Tensor,
    pub cue_categories: &'a Tensor,
    pub response_prefix: &'a Tensor,
    pub source_prefix: &'a Tensor,
    pub post_state: &'a Tensor,
    pub local_state: &'a Tensor,
    pub token_ids: &'a [u32],
    pub status: u8,
}
pub struct DeviceEmissionStep {
    pub generated: Tensor,
    pub copied: Tensor,
    pub copy_ids: Vec<u32>,
    pub post_state: Tensor,
    pub local_state: Tensor,
    pub generated_native_q24: Tensor,
    pub copied_native_q24: Tensor,
}
struct Energy {
    credit: Tensor,
    native_q24: Tensor,
}
#[derive(Clone)]
struct ContextFamily {
    token: Tensor,
    own: Tensor,
    neighbor: Tensor,
}

/// Numerical data and credit stay on the declared device. Sampling downloads
/// nothing: the counter-based draw is uploaded as bounded scalar metadata.
/// Only explicit checkpoint/host-store boundaries may inspect chosen IDs.
pub struct WholePathCudaProducer {
    revision: String,
    parameters: String,
    geometry: String,
    params: BTreeMap<String, Var>,
    device: Device,
    lanes: usize,
    per_head: usize,
    vocab: usize,
    context: [ContextFamily; 3],
    basis: Tensor,
    product: Tensor,
    relative: Tensor,
    prototypes: Tensor,
    edges: Vec<(usize, usize)>,
    generate_hard: [Tensor; 3],
    bridge_hard: [Tensor; 2],
    continuation_hard: Tensor,
    cross_state: bool,
    shared_copy: bool,
    signal_hard: Tensor,
    geometry_images: Tensor,
    head_hard: BTreeMap<String, Tensor>,
    continuation_scale: f64,
    potential_hard: Tensor,
    cue_hard: Tensor,
    prefix_hard: Tensor,
    cue_unary: bool,
    prefix_unary: bool,
    joint: Option<(CueJointConfig, Tensor)>,
    sampling: Sampling,
    rng: u64,
    logs: Vec<Tensor>,
    choices: Vec<Tensor>,
    decisions: usize,
    pub(super) greedy: bool,
    exp: Tensor,
}

fn u32_indices(device: &Device, values: Vec<u32>) -> Result<Tensor> {
    let n = values.len();
    Ok(Tensor::from_vec(values, n, device)?)
}
fn hard_f64(v: &Var, scale: f64) -> Result<Tensor> {
    let vals = values(v)?;
    if vals.iter().any(|v| !v.is_finite() || v.abs() > 1.75) {
        return Err(invalid("device producer master outside Q4"));
    }
    Ok(Tensor::from_vec(
        vals.iter()
            .map(|v| f64::from((v * 4.).round()) * scale)
            .collect::<Vec<_>>(),
        v.shape(),
        v.device(),
    )?)
}
fn bind(hard: &Tensor, graph: Tensor) -> Result<Tensor> {
    if hard.dims() != graph.dims() || !hard.device().same_device(graph.device()) {
        return Err(invalid("device anchor differs"));
    }
    Ok((hard.to_dtype(DType::F32)? + (&graph - graph.detach())?)?)
}
impl WholePathCudaProducer {
    fn fork_episode(&self, seed: u64) -> Self {
        let mut sampling = self.sampling.clone();
        sampling.seed = seed;
        Self {
            revision: self.revision.clone(),
            parameters: self.parameters.clone(),
            geometry: self.geometry.clone(),
            params: self.params.clone(),
            device: self.device.clone(),
            lanes: self.lanes,
            per_head: self.per_head,
            vocab: self.vocab,
            context: self.context.clone(),
            basis: self.basis.clone(),
            product: self.product.clone(),
            relative: self.relative.clone(),
            prototypes: self.prototypes.clone(),
            edges: self.edges.clone(),
            generate_hard: self.generate_hard.clone(),
            bridge_hard: self.bridge_hard.clone(),
            continuation_hard: self.continuation_hard.clone(),
            cross_state: self.cross_state,
            shared_copy: self.shared_copy,
            signal_hard: self.signal_hard.clone(),
            geometry_images: self.geometry_images.clone(),
            head_hard: self.head_hard.clone(),
            continuation_scale: self.continuation_scale,
            potential_hard: self.potential_hard.clone(),
            cue_hard: self.cue_hard.clone(),
            prefix_hard: self.prefix_hard.clone(),
            cue_unary: self.cue_unary,
            prefix_unary: self.prefix_unary,
            joint: self.joint.clone(),
            sampling,
            rng: seed,
            logs: vec![],
            choices: vec![],
            decisions: 0,
            greedy: self.greedy,
            exp: self.exp.clone(),
        }
    }
    /// All masters must already reside on CUDA and belong to this exact bound
    /// inventory. A CPU reference is only available inside focused tests.
    pub fn new(
        revision: &WholePathRevision,
        params: BTreeMap<String, Var>,
        sampling: Sampling,
    ) -> Result<Self> {
        if params.is_empty() || params.values().any(|v| !v.device().is_cuda()) {
            return Err(invalid(
                "whole-path producer requires CUDA masters; no CPU fallback",
            ));
        }
        let mut producer = Self::admit(revision, params, sampling)?;
        producer.restore_prototype_credit(revision)?;
        Ok(producer)
    }
    pub(super) fn admit(
        _revision: &WholePathRevision,
        _params: BTreeMap<String, Var>,
        _sampling: Sampling,
    ) -> Result<Self> {
        Err(invalid("retired whole-path probability artifact; use native_vector_channel with the shared vector head; no conversion or fallback"))
    }
    #[allow(dead_code)]
    fn retained_inactive_admit(
        revision: &WholePathRevision,
        params: BTreeMap<String, Var>,
        sampling: Sampling,
    ) -> Result<Self> {
        sampling.validate()?;
        if parameter_digest(&params)? != revision.parameter_sha256() {
            return Err(invalid("device producer/revision masters differ"));
        }
        let device = params
            .values()
            .next()
            .ok_or_else(|| invalid("empty producer masters"))?
            .device()
            .clone();
        if params.values().any(|v| !v.device().same_device(&device)) {
            return Err(invalid("producer masters span devices"));
        }
        let lanes = revision.manifest.lanes;
        let (native, algebra) = revision.encoder.context_encoder_parts();
        let vocab = native.vocab_size();
        let families = [
            ContextDecisionFamily::Transition,
            ContextDecisionFamily::Root,
            ContextDecisionFamily::Category,
        ];
        // Expand admitted frozen integer factors once. Scores remain exact
        // integers in F64 (bounded far below2^53); no native forward is called
        // by context(), and no per-decision score download is performed.
        let mut context = Vec::new();
        for family in families {
            let classes = if family == ContextDecisionFamily::Category {
                33
            } else {
                120
            };
            let mut token = Vec::new();
            let mut own = Vec::new();
            let mut neighbor = Vec::new();
            for id in 0..vocab {
                for lane in 0..lanes {
                    token.extend(
                        native
                            .decision_scores(id, lane, family, H4Code::IDENTITY, H4Code::IDENTITY)
                            .map_err(|e| invalid(e.to_string()))?
                            .1
                            .into_iter()
                            .map(|v| v as f64 / Q24),
                    );
                }
            }
            for lane in 0..lanes {
                let base = native
                    .decision_scores(0, lane, family, H4Code::IDENTITY, H4Code::IDENTITY)
                    .map_err(|e| invalid(e.to_string()))?
                    .1;
                for code in 0..120 {
                    let code = H4Code::try_from(code as u8).map_err(|e| invalid(e.to_string()))?;
                    own.extend(
                        native
                            .decision_scores(0, lane, family, code, H4Code::IDENTITY)
                            .map_err(|e| invalid(e.to_string()))?
                            .1
                            .iter()
                            .zip(&base)
                            .map(|(v, b)| (*v - *b) as f64 / Q24),
                    );
                    neighbor.extend(
                        native
                            .decision_scores(0, lane, family, H4Code::IDENTITY, code)
                            .map_err(|e| invalid(e.to_string()))?
                            .1
                            .iter()
                            .zip(&base)
                            .map(|(v, b)| (*v - *b) as f64 / Q24),
                    );
                }
            }
            context.push(ContextFamily {
                token: Tensor::from_vec(token, (vocab, lanes, classes), &device)?,
                own: Tensor::from_vec(own, (lanes, 120, classes), &device)?,
                neighbor: Tensor::from_vec(neighbor, (lanes, 120, classes), &device)?,
            });
        }
        let product = (0..120u8)
            .flat_map(|a| (0..120u8).map(move |b| (a, b)))
            .map(|(a, b)| {
                let a = H4Code::try_from(a).map_err(|e| invalid(e.to_string()))?;
                let b = H4Code::try_from(b).map_err(|e| invalid(e.to_string()))?;
                Ok(u32::from(algebra.compose(a, b).index()))
            })
            .collect::<Result<Vec<_>>>()?;
        let g = revision.generator(None)?;
        let generate = g.generate_model();
        let relative = (0..120u8)
            .flat_map(|s| (0..120u8).map(move |p| (s, p)))
            .map(|(s, p)| {
                generate
                    .algebra()
                    .relative(s, generate.algebra().identity(), p)
                    .map(u32::from)
                    .map_err(|e| invalid(e.to_string()))
            })
            .collect::<Result<Vec<_>>>()?;
        let var = |name: &str| {
            params
                .get(name)
                .ok_or_else(|| invalid(format!("producer master absent:{name}")))
        };
        let generate_hard = [
            hard_f64(var("generate.bias")?, 1. / 16.)?,
            hard_f64(var("generate.unary")?, 1. / 16.)?,
            hard_f64(var("generate.pair")?, 1. / 16.)?,
        ];
        let bridge_hard = [
            hard_f64(var("read_state_bridge.bias")?, 1. / 16.)?,
            hard_f64(var("read_state_bridge.relative")?, 1. / 16.)?,
        ];
        let cross_state = params.contains_key("continuation.cross_state");
        let u = var(if cross_state {
            "continuation.cross_state"
        } else {
            "continuation.unary"
        })?;
        let native_u = uor_r4_core::native_geometric::learner::geometric_continuation_field::NativeContinuationField::from_bytes(
            &std::fs::read(revision.root.join("continuation.bin"))?,
            &revision.manifest.source,
            generate,
        ).map_err(|e| invalid(e.to_string()))?;
        let continuation_scale = (1u64 << native_u.score_shift()) as f64 / Q24;
        let continuation_hard = hard_f64(u, continuation_scale)?;
        let signal_hard = hard_f64(var("memory.signal")?, 1. / 4.)?;
        let exp_bytes = std::fs::read(revision.root.join("native/consumer/exp-q31.bin"))?;
        let exp_values = exp_bytes
            .chunks_exact(4)
            .map(|v| u32::from_le_bytes([v[0], v[1], v[2], v[3]]) as f64)
            .collect::<Vec<_>>();
        let exp = Tensor::from_vec(exp_values.clone(), exp_values.len(), &device)?;
        let group = uor_r4_core::spiralcore_operator::cl06_finite_composition_table()
            .map_err(|e| invalid(e.to_string()))?;
        let mut images = Vec::new();
        for frame in 0..64 {
            let inv = group
                .inverse_index(frame)
                .ok_or_else(|| invalid("frame inverse absent"))?;
            for root in 0..240 {
                images.push(u32::from(
                    crate::e8_memory::e8_tables()?.transport_root(root, inv)?,
                ));
            }
        }
        let head_hard = params
            .iter()
            .filter(|(name, _)| name.starts_with("compiler.") || name.starts_with("route."))
            .map(|(name, v)| Ok((name.clone(), hard_f64(v, 0.25)?)))
            .collect::<Result<_>>()?;
        let pc = uor_r4_integer::geometric_potential_q4::PotentialQ4Config {
            heads: native.heads(),
            lanes_per_head: native.lanes_per_head(),
        };
        let mut potential = Vec::new();
        for (name, _) in pc
            .coefficient_shapes()
            .map_err(|e| invalid(e.to_string()))?
        {
            potential.extend(
                values(var(&format!("consumer.potential.{name}"))?)?
                    .iter()
                    .map(|v| (v * 4.).round() as i8),
            );
        }
        let pq = uor_r4_integer::geometric_potential_q4::NativePotentialQ4::new(
            pc,
            &pack_coefficients(&potential).map_err(|e| invalid(e.to_string()))?,
        )
        .map_err(|e| invalid(e.to_string()))?;
        let potential_hard = Tensor::from_vec(
            pq.expanded_q24()
                .iter()
                .map(|&x| x as f64 / Q24)
                .collect::<Vec<_>>(),
            (lanes, uor_r4_integer::geometric_potential::ENTRIES_PER_LANE),
            &device,
        )?;
        let cue_hard = hard_f64(var("cue.coefficients")?, 0.25)?.reshape((lanes, 120))?;
        let prefix_hard = hard_f64(var("prefix.coefficients")?, 0.25)?.reshape((lanes, 120))?;
        let cm: uor_r4_integer::geometric_cue_carrier::CueCarrierMetadata =
            serde_json::from_slice(&std::fs::read(revision.root.join("cue.json"))?)?;
        let pm: uor_r4_integer::geometric_prefix_transport::PrefixTransportMetadata =
            serde_json::from_slice(&std::fs::read(revision.root.join("prefix.json"))?)?;
        let joint = cm
            .joint
            .map(|j| {
                let bytes = std::fs::read(revision.root.join("cue-joint.bin"))?;
                let table = uor_r4_integer::geometric_potential_q4::compile_ordered_pair_q4(&bytes)
                    .map_err(|e| invalid(e.to_string()))?;
                Ok::<_, crate::TrainingError>((
                    j.config,
                    Tensor::from_vec(
                        table.iter().map(|&x| x as f64 / Q24).collect::<Vec<_>>(),
                        table.len(),
                        &device,
                    )?,
                ))
            })
            .transpose()?;
        Ok(Self {
            revision: revision.sha256.clone(),
            parameters: revision.parameter_sha256().into(),
            geometry: route_geometry_sha256()?,
            params,
            device: device.clone(),
            lanes,
            per_head: native.lanes_per_head(),
            vocab,
            context: context
                .try_into()
                .map_err(|_| invalid("Context family inventory"))?,
            basis: Tensor::from_vec(
                uor_r4_integer::geometric_context_q4::canonical_basis_q25()
                    .iter()
                    .flat_map(|r| r.iter().map(|&x| x as f32 / Q25 as f32))
                    .collect::<Vec<_>>(),
                (120, 4),
                &device,
            )?,
            product: u32_indices(&device, product)?,
            relative: u32_indices(&device, relative)?,
            prototypes: Tensor::from_vec(
                generate
                    .prototypes()
                    .iter()
                    .map(|&p| u32::from(p))
                    .collect::<Vec<_>>(),
                (vocab, lanes),
                &device,
            )?,
            edges: generate
                .energy()
                .edges()
                .iter()
                .map(|e| (usize::from(e.left), usize::from(e.right)))
                .collect(),
            generate_hard,
            bridge_hard,
            continuation_hard,
            cross_state,
            shared_copy: native_u.applies_to_copy(),
            signal_hard,
            geometry_images: Tensor::from_vec(images, (64, 240), &device)?,
            head_hard,
            continuation_scale,
            potential_hard,
            cue_hard,
            prefix_hard,
            cue_unary: cm.potential.mode
                == uor_r4_integer::geometric_cue_carrier::CueScoreMode::CueUnary,
            prefix_unary: pm.potential.mode
                == uor_r4_integer::geometric_prefix_transport::PrefixScoreMode::SourcePrefixUnary,
            joint,
            rng: sampling.seed,
            sampling,
            logs: vec![],
            choices: vec![],
            decisions: 0,
            greedy: false,
            exp,
        })
    }
    pub fn revision_sha256(&self) -> &str {
        &self.revision
    }
    pub(super) fn restore_prototype_credit(&mut self, revision: &WholePathRevision) -> Result<()> {
        let Mode::Sampled(global) = &revision.manifest.mode else {
            return Ok(());
        };
        let episode = self.sampling.clone();
        self.sampling = global.clone();
        self.rng = global.seed;
        let logits = self
            .var("generate.prototype_choices")?
            .reshape((self.vocab * self.lanes, 120))?;
        let mut chosen = Vec::new();
        for row in 0..self.vocab * self.lanes {
            chosen.push(self.choose(&logits.i(row)?, true)?.index);
        }
        // This is the artifact/rebinding boundary. Only discrete codebook IDs
        // cross it; GPU probabilities and their original-Var graph stay here.
        if Tensor::cat(&chosen, 0)?.to_vec1::<u32>()?
            != self.prototypes.flatten_all()?.to_vec1::<u32>()?
        {
            return Err(invalid(
                "device prototype replay differs from bound artifact",
            ));
        }
        self.sampling = episode;
        self.rng = self.sampling.seed;
        if self.decisions >= self.sampling.maximum_decisions {
            return Err(invalid("prototype decisions exhaust episode admission"));
        }
        Ok(())
    }
    fn decision_digest(&self) -> Result<String> {
        if self.logs.len() != self.choices.len() || self.logs.len() != self.decisions {
            return Err(invalid("device decision inventory differs"));
        }
        let mut bytes = Vec::with_capacity(self.decisions * 8);
        if !self.logs.is_empty() {
            let ids = Tensor::cat(&self.choices, 0)?.to_vec1::<u32>()?;
            let probabilities = Tensor::stack(&self.logs, 0)?.to_vec1::<f32>()?;
            for (id, p) in ids.into_iter().zip(probabilities) {
                if !p.is_finite() {
                    return Err(invalid("device decision probability nonfinite"));
                }
                bytes.extend_from_slice(&id.to_le_bytes());
                bytes.extend_from_slice(&p.to_bits().to_le_bytes());
            }
        }
        Ok(sha256_bytes(&bytes))
    }
    pub fn parameter_sha256(&self) -> &str {
        &self.parameters
    }
    pub fn geometry_sha256(&self) -> &str {
        &self.geometry
    }
    fn var(&self, name: &str) -> Result<&Var> {
        self.params
            .get(name)
            .ok_or_else(|| invalid(format!("producer parameter absent:{name}")))
    }
    fn code(&self, t: &Tensor, shape: &[usize]) -> Result<()> {
        if t.dtype() != DType::U32 || t.dims() != shape || !t.device().same_device(&self.device) {
            return Err(invalid("device code shape/device differs"));
        }
        Ok(())
    }
    /// Choice is a device tensor. The last class owns any F32 cumsum tail, so
    /// rounding cannot produce an all-false mask that defaults to class zero.
    pub fn choose(&mut self, _logits: &Tensor, _temperature: bool) -> Result<DeviceChoice> {
        Err(invalid(
            "retired probability choice; use raw shared-vector selection",
        ))
    }
    pub(super) fn context_scores(
        &self,
        token: usize,
        lane: usize,
        family: usize,
        state: &Tensor,
    ) -> Result<Tensor> {
        Ok(self.context_energy(token, lane, family, state)?.credit)
    }
    fn context_energy(
        &self,
        token: usize,
        lane: usize,
        family: usize,
        state: &Tensor,
    ) -> Result<Energy> {
        let n = if family == 2 { 33 } else { 120 };
        let neighbor = lane / self.per_head * self.per_head + (lane + 1) % self.per_head;
        let own = state.narrow(0, lane, 1)?;
        let other = state.narrow(0, neighbor, 1)?;
        let f = &self.context[family];
        let hard =
            (&f.token.i((token, lane))? + f.own.i(lane)?.index_select(&own, 0)?.flatten_all()?)?;
        let hard = (&hard + f.neighbor.i(lane)?.index_select(&other, 0)?.flatten_all()?)?;
        let suffix = if family == 0 {
            "transition"
        } else if family == 1 {
            "root"
        } else {
            "category"
        };
        let tokenvar = self.var(&format!("consumer.context.token_{suffix}"))?;
        let mut graph = crate::geometric_context::q4_shadow_ste(tokenvar.as_tensor())?
            .reshape((self.vocab, self.lanes, n))?
            .i((token, lane))?;
        for (name, code) in [("self", own), ("neighbor", other)] {
            if let Some(v) = self
                .params
                .get(&format!("consumer.context.{name}_{suffix}"))
            {
                let coef = crate::geometric_context::q4_shadow_ste(v.as_tensor())?
                    .reshape((self.lanes, n, 4))?
                    .i(lane)?;
                let basis = self.basis.index_select(&code, 0)?.reshape((1, 4))?;
                graph = (&graph + coef.broadcast_mul(&basis)?.sum(1)?)?;
            }
        }
        Ok(Energy {
            native_q24: hard.affine(Q24, 0.)?,
            credit: bind(&hard, graph)?,
        })
    }
    /// Native OLD -> RIGHT action -> NEW, then root/category observations from
    /// NEW. Retained state and lossy observations never substitute for each other.
    pub fn context(&mut self, tokens: &[u32]) -> Result<DeviceContext> {
        if tokens.is_empty()
            || tokens.len() > 128
            || tokens.iter().any(|&t| t as usize >= self.vocab)
        {
            return Err(invalid("producer Context token admission"));
        }
        let mut state = Tensor::full(
            u32::from(H4Code::IDENTITY.index()),
            self.lanes,
            &self.device,
        )?;
        let mut retained = Vec::new();
        let mut roots = Vec::new();
        let mut categories = Vec::new();
        for &token in tokens {
            let mut actions = Vec::new();
            for lane in 0..self.lanes {
                let logits = self.context_energy(token as usize, lane, 0, &state)?;
                actions.push(if self.greedy {
                    logits.native_q24.argmax(0)?.reshape(1)?
                } else {
                    self.choose(&logits.credit, true)?.index
                });
            }
            let actions = Tensor::cat(&actions, 0)?;
            let at =
                (state.broadcast_mul(&Tensor::full(120u32, self.lanes, &self.device)?)? + actions)?;
            state = self.product.index_select(&at, 0)?;
            let mut root = Vec::new();
            let mut category = Vec::new();
            for lane in 0..self.lanes {
                let r = self.context_energy(token as usize, lane, 1, &state)?;
                root.push(if self.greedy {
                    r.native_q24.argmax(0)?.reshape(1)?
                } else {
                    self.choose(&r.credit, true)?.index
                });
                let c = self.context_energy(token as usize, lane, 2, &state)?;
                category.push(if self.greedy {
                    c.native_q24.argmax(0)?.reshape(1)?
                } else {
                    self.choose(&c.credit, true)?.index
                });
            }
            retained.push(state.clone());
            roots.push(Tensor::cat(&root, 0)?);
            categories.push(Tensor::cat(&category, 0)?);
        }
        Ok(DeviceContext {
            retained: Tensor::stack(&retained, 0)?,
            roots: Tensor::stack(&roots, 0)?,
            categories: Tensor::stack(&categories, 0)?,
        })
    }
    pub fn relative_codes(&self, left: &Tensor, right: &Tensor) -> Result<Tensor> {
        if left.dims() != right.dims() {
            return Err(invalid("relative code shapes differ"));
        }
        self.code(left, left.dims())?;
        self.code(right, right.dims())?;
        let at = (left.flatten_all()?.broadcast_mul(&Tensor::full(
            120u32,
            left.elem_count(),
            &self.device,
        )?)? + right.flatten_all()?)?;
        Ok(self.relative.index_select(&at, 0)?.reshape(left.shape())?)
    }
    /// C/K/rank additive heads use the actual device H4 codes; None slots are
    /// structural masks, never gold relation/value masks.
    pub fn head(&self, name: &str, rows: &Tensor, mask: &Tensor, classes: usize) -> Result<Tensor> {
        let (count, slots) = rows.dims2()?;
        self.code(rows, &[count, slots])?;
        if count == 0
            || classes == 0
            || mask.dims() != rows.dims()
            || !mask.device().same_device(&self.device)
        {
            return Err(invalid("device head row/mask admission"));
        }
        let v = self.var(name)?;
        let stride = 1 + slots * 120;
        if v.elem_count() != classes * stride {
            return Err(invalid("device head master shape"));
        }
        let hard = self
            .head_hard
            .get(name)
            .ok_or_else(|| invalid("head frozen factor absent"))?
            .reshape((classes, stride))?;
        let shadow =
            crate::geometric_context::q4_shadow_ste(v.as_tensor())?.reshape((classes, stride))?;
        let offset = Tensor::from_vec(
            (0..slots).map(|s| (1 + s * 120) as u32).collect::<Vec<_>>(),
            (1, slots),
            &self.device,
        )?;
        let index = rows.broadcast_add(&offset)?.flatten_all()?;
        let mut h = Vec::new();
        let mut g = Vec::new();
        for class in 0..classes {
            h.push(
                (hard
                    .i(class)?
                    .index_select(&index, 0)?
                    .reshape((count, slots))?
                    .broadcast_mul(&mask.to_dtype(DType::F64)?)?
                    .sum_all()?
                    + hard.i((class, 0))?)?,
            );
            g.push(
                (shadow
                    .i(class)?
                    .index_select(&index, 0)?
                    .reshape((count, slots))?
                    .broadcast_mul(&mask.to_dtype(DType::F32)?)?
                    .sum_all()?
                    + shadow.i((class, 0))?)?,
            );
        }
        bind(&Tensor::stack(&h, 0)?, Tensor::stack(&g, 0)?)
    }
    pub fn canonical_roots(&self, physical: &Tensor, frame: u8) -> Result<Tensor> {
        if frame >= 64 {
            return Err(invalid("device route frame invalid"));
        }
        self.code(physical, physical.dims())?;
        Ok(self
            .geometry_images
            .i(frame as usize)?
            .index_select(&physical.flatten_all()?, 0)?
            .reshape(physical.shape())?)
    }
    pub fn family_scores(&self, query: &Tensor, profiles: &Tensor) -> Result<Tensor> {
        self.code(query, &[self.lanes])?;
        if profiles.dims() != [15, 9] || !profiles.device().same_device(&self.device) {
            return Err(invalid("device family profiles differ"));
        }
        let frozen = self
            .head_hard
            .get("route.family")
            .ok_or_else(|| invalid("route family absent"))?;
        let shadow =
            crate::geometric_context::q4_shadow_ste(self.var("route.family")?.as_tensor())?;
        let mut h = Vec::new();
        let mut g = Vec::new();
        for orbit in 0..15 {
            let mut hard = Tensor::new(0f64, &self.device)?;
            let mut graph = Tensor::new(0f32, &self.device)?;
            for lane in 0..self.lanes {
                let at = query.narrow(0, lane, 1)?;
                hard = (&hard
                    + frozen
                        .i((orbit, lane))?
                        .index_select(&at, 0)?
                        .flatten_all()?
                        .broadcast_mul(&profiles.i(orbit)?.to_dtype(DType::F64)?)?
                        .sum_all()?)?;
                graph = (&graph
                    + shadow
                        .i((orbit, lane))?
                        .index_select(&at, 0)?
                        .flatten_all()?
                        .broadcast_mul(&profiles.i(orbit)?.to_dtype(DType::F32)?)?
                        .sum_all()?)?;
            }
            h.push(hard);
            g.push(graph);
        }
        bind(&Tensor::stack(&h, 0)?, Tensor::stack(&g, 0)?)
    }
    pub fn bridge(&mut self, query: &Tensor, source: &Tensor) -> Result<Tensor> {
        self.code(query, &[self.lanes])?;
        self.code(source, &[self.lanes])?;
        let delta = self.relative_codes(query, source)?;
        let mut acts = Vec::new();
        for lane in 0..self.lanes {
            let row = delta.narrow(0, lane, 1)?;
            let hard = (&self.bridge_hard[0].i(lane)?
                + self.bridge_hard[1]
                    .i(lane)?
                    .index_select(&row, 0)?
                    .flatten_all()?)?;
            let graph = (&crate::geometric_context::q4_shadow_ste(
                self.var("read_state_bridge.bias")?.as_tensor(),
            )?
            .i(lane)?
                + crate::geometric_context::q4_shadow_ste(
                    self.var("read_state_bridge.relative")?.as_tensor(),
                )?
                .i(lane)?
                .index_select(&row, 0)?
                .flatten_all()?)?
            .affine(0.25, 0.)?;
            acts.push(self.choose(&bind(&hard, graph)?, true)?.index);
        }
        let at = (query.broadcast_mul(&Tensor::full(120u32, self.lanes, &self.device)?)?
            + Tensor::cat(&acts, 0)?)?;
        Ok(self.product.index_select(&at, 0)?)
    }
    pub fn generated_scores(&self, post: &Tensor, local: &Tensor, status: u8) -> Result<Tensor> {
        Ok(self.generated_energy(post, local, status)?.credit)
    }
    fn generated_energy(&self, post: &Tensor, local: &Tensor, status: u8) -> Result<Energy> {
        self.code(post, &[self.lanes])?;
        self.code(local, &[self.lanes])?;
        if status >= 33 {
            return Err(invalid("device store status invalid"));
        }
        let post = post
            .reshape((1, self.lanes))?
            .broadcast_as((self.vocab, self.lanes))?;
        let local = local
            .reshape((1, self.lanes))?
            .broadcast_as((self.vocab, self.lanes))?;
        let r = self.relative_codes(&post, &self.prototypes)?;
        let l = self.relative_codes(&local, &self.prototypes)?;
        let mut hard = self.generate_hard[0].clone();
        let mut graph =
            crate::geometric_context::q4_shadow_ste(self.var("generate.bias")?.as_tensor())?
                .affine(0.25, 0.)?;
        for lane in 0..self.lanes {
            let at = r.i((.., lane))?.contiguous()?;
            hard = (&hard + self.generate_hard[1].i(lane)?.index_select(&at, 0)?)?;
            graph = (&graph
                + crate::geometric_context::q4_shadow_ste(
                    self.var("generate.unary")?.as_tensor(),
                )?
                .i(lane)?
                .index_select(&at, 0)?
                .affine(0.25, 0.)?)?;
        }
        for (edge, &(a, b)) in self.edges.iter().enumerate() {
            let at =
                (r.i((.., a))?
                    .broadcast_mul(&Tensor::full(120u32, self.vocab, &self.device)?)?
                    + r.i((.., b))?)?
                .contiguous()?;
            hard = (&hard
                + self.generate_hard[2]
                    .i(edge)?
                    .flatten_all()?
                    .index_select(&at, 0)?)?;
            graph = (&graph
                + crate::geometric_context::q4_shadow_ste(
                    self.var("generate.pair")?.as_tensor(),
                )?
                .i(edge)?
                .flatten_all()?
                .index_select(&at, 0)?
                .affine(0.25, 0.)?)?;
        }
        let u = crate::geometric_context::q4_shadow_ste(
            self.var(if self.cross_state {
                "continuation.cross_state"
            } else {
                "continuation.unary"
            })?
            .as_tensor(),
        )?;
        for lane in 0..self.lanes {
            let at = if self.cross_state {
                (r.i((.., lane))?.broadcast_mul(&Tensor::full(
                    120u32,
                    self.vocab,
                    &self.device,
                )?)? + l.i((.., lane))?)?
            } else {
                l.i((.., lane))?
            };
            let at = at.contiguous()?;
            hard = (&hard + self.continuation_hard.i(lane)?.index_select(&at, 0)?)?;
            graph = (&graph
                + u.i(lane)?
                    .index_select(&at, 0)?
                    .affine(4. * self.continuation_scale, 0.)?)?;
        }
        hard = (&hard + self.signal_hard.i(status as usize)?)?;
        graph = (&graph
            + crate::geometric_context::q4_shadow_ste(self.var("memory.signal")?.as_tensor())?
                .i(status as usize)?)?;
        Ok(Energy {
            native_q24: hard.affine(Q24, 0.)?,
            credit: bind(&hard, graph)?,
        })
    }
    fn continuation_scores(&self, post: &Tensor, local: &Tensor) -> Result<Tensor> {
        Ok(self.continuation_energy(post, local)?.credit)
    }
    fn continuation_energy(&self, post: &Tensor, local: &Tensor) -> Result<Energy> {
        self.code(post, &[self.lanes])?;
        self.code(local, &[self.lanes])?;
        let post = post
            .reshape((1, self.lanes))?
            .broadcast_as((self.vocab, self.lanes))?;
        let local = local
            .reshape((1, self.lanes))?
            .broadcast_as((self.vocab, self.lanes))?;
        let r = self.relative_codes(&post, &self.prototypes)?;
        let l = self.relative_codes(&local, &self.prototypes)?;
        let mut hard = Tensor::zeros(self.vocab, DType::F64, &self.device)?;
        let mut graph = Tensor::zeros(self.vocab, DType::F32, &self.device)?;
        let u = crate::geometric_context::q4_shadow_ste(
            self.var(if self.cross_state {
                "continuation.cross_state"
            } else {
                "continuation.unary"
            })?
            .as_tensor(),
        )?;
        for lane in 0..self.lanes {
            let at = if self.cross_state {
                (r.i((.., lane))?.broadcast_mul(&Tensor::full(
                    120u32,
                    self.vocab,
                    &self.device,
                )?)? + l.i((.., lane))?)?
            } else {
                l.i((.., lane))?
            }
            .contiguous()?;
            hard = (&hard + self.continuation_hard.i(lane)?.index_select(&at, 0)?)?;
            graph = (&graph
                + u.i(lane)?
                    .index_select(&at, 0)?
                    .affine(4. * self.continuation_scale, 0.)?)?;
        }
        Ok(Energy {
            native_q24: hard.affine(Q24, 0.)?,
            credit: bind(&hard, graph)?,
        })
    }
    /// Actual observed Context codes, source cue and ordered pre-token Prefix
    /// states determine Copy scores. Content remains absent, as in native bank
    /// policy; inactive content/cross families receive no fabricated derivative.
    pub fn copy_scores(&self, input: DeviceCopyInput<'_>) -> Result<Tensor> {
        Ok(self.copy_components(input, true)?.credit)
    }
    fn copy_components(&self, input: DeviceCopyInput<'_>, emission: bool) -> Result<Energy> {
        use uor_r4_integer::geometric_potential::*;
        let n = input.token_ids.len();
        if n == 0
            || n > 128
            || input.status >= 33
            || input.token_ids.iter().any(|&id| id as usize >= self.vocab)
        {
            return Err(invalid("device Copy occurrence admission"));
        }
        for code in [
            input.query_roots,
            input.query_categories,
            input.cue_query_roots,
            input.cue_query_categories,
            input.response_prefix,
            input.post_state,
            input.local_state,
        ] {
            self.code(code, &[self.lanes])?;
        }
        for code in [
            input.candidate_roots,
            input.candidate_categories,
            input.cue_roots,
            input.cue_categories,
            input.source_prefix,
        ] {
            self.code(code, &[n, self.lanes])?;
        }
        let query = input
            .query_roots
            .reshape((1, self.lanes))?
            .broadcast_as((n, self.lanes))?;
        let qr = input
            .query_categories
            .reshape((1, self.lanes))?
            .broadcast_as((n, self.lanes))?;
        let rp = self.relative_codes(&query, input.candidate_roots)?;
        let cq = input
            .cue_query_roots
            .reshape((1, self.lanes))?
            .broadcast_as((n, self.lanes))?;
        let cqc = input
            .cue_query_categories
            .reshape((1, self.lanes))?
            .broadcast_as((n, self.lanes))?;
        let rc = self.relative_codes(&cq, input.cue_roots)?;
        let prefix = input
            .response_prefix
            .reshape((1, self.lanes))?
            .broadcast_as((n, self.lanes))?;
        let rr = self.relative_codes(&prefix, input.source_prefix)?;
        let mut hard = Tensor::zeros(n, DType::F64, &self.device)?;
        let mut graph = Tensor::zeros(n, DType::F32, &self.device)?;
        let pv = |name: &str| -> Result<Tensor> {
            let v = self.var(&format!("consumer.potential.{name}"))?;
            Ok(crate::geometric_context::q4_shadow_ste(v.as_tensor())?
                .reshape((self.lanes, v.elem_count() / self.lanes))?)
        };
        let unary = pv("context_unary")?;
        let radius = pv("context_radius")?;
        let presence = pv("context_presence")?;
        let content_presence = pv("content_presence")?;
        let cue =
            crate::geometric_context::q4_shadow_ste(self.var("cue.coefficients")?.as_tensor())?
                .reshape((self.lanes, 120))?;
        let prefix =
            crate::geometric_context::q4_shadow_ste(self.var("prefix.coefficients")?.as_tensor())?
                .reshape((self.lanes, 120))?;
        let mut cue_masks = Vec::new();
        for lane in 0..self.lanes {
            let q = qr.i((.., lane))?.contiguous()?;
            let k = input.candidate_categories.i((.., lane))?.contiguous()?;
            let present = q.ne(0u32)?.to_dtype(DType::F32)?;
            let kp = k.ne(0u32)?.to_dtype(DType::F32)?;
            let mask = (&present * &kp)?;
            let pres = (q
                .ne(0u32)?
                .to_dtype(DType::U32)?
                .broadcast_mul(&Tensor::full(2u32, n, &self.device)?)?
                + k.ne(0u32)?.to_dtype(DType::U32)?)?;
            let bins = (q
                .to_dtype(DType::F64)?
                .affine(1., -1.)?
                .clamp(0f64, 31f64)?
                .affine(32., 0.)?
                + k.to_dtype(DType::F64)?
                    .affine(1., -1.)?
                    .clamp(0f64, 31f64)?)?
            .to_dtype(DType::U32)?;
            let rel = rp.i((.., lane))?.contiguous()?;
            let table = self.potential_hard.i(lane)?;
            let angular = table
                .narrow(0, CONTEXT_UNARY_OFFSET, 120)?
                .index_select(&rel, 0)?;
            let radial = table
                .narrow(0, CONTEXT_RADIUS_OFFSET, 1024)?
                .index_select(&bins, 0)?;
            hard = (&hard + (&angular + &radial)?.broadcast_mul(&mask.to_dtype(DType::F64)?)?)?;
            hard = (&hard
                + table
                    .narrow(0, CONTEXT_PRESENCE_OFFSET, 4)?
                    .index_select(&pres, 0)?)?;
            hard = hard.broadcast_add(&table.i(CONTENT_PRESENCE_OFFSET)?)?;
            let basis = self.basis.index_select(&rel, 0)?;
            let angular = basis
                .broadcast_mul(&unary.i(lane)?.reshape((1, 4))?)?
                .sum(1)?;
            let radial = radius.i(lane)?.index_select(&bins, 0)?;
            graph = (&graph + (&angular + &radial)?.broadcast_mul(&mask)?)?;
            graph = (&graph + presence.i(lane)?.index_select(&pres, 0)?)?;
            graph = graph.broadcast_add(&content_presence.i((lane, 0))?)?;
            let ck = input
                .cue_categories
                .i((.., lane))?
                .contiguous()?
                .ne(0u32)?
                .to_dtype(DType::F32)?;
            let cm = (cqc
                .i((.., lane))?
                .contiguous()?
                .ne(0u32)?
                .to_dtype(DType::F32)?
                * &ck)?;
            cue_masks.push(cm.clone());
            let at = if self.cue_unary {
                input.cue_roots.i((.., lane))?
            } else {
                rc.i((.., lane))?
            }
            .contiguous()?;
            hard = (&hard
                + self
                    .cue_hard
                    .i(lane)?
                    .index_select(&at, 0)?
                    .broadcast_mul(&cm.to_dtype(DType::F64)?)?)?;
            graph = (&graph + cue.i(lane)?.index_select(&at, 0)?.broadcast_mul(&cm)?)?;
            let at = if self.prefix_unary {
                input.source_prefix.i((.., lane))?
            } else {
                rr.i((.., lane))?
            }
            .contiguous()?;
            hard = (&hard + self.prefix_hard.i(lane)?.index_select(&at, 0)?)?;
            graph = (&graph + prefix.i(lane)?.index_select(&at, 0)?)?;
        }
        if let Some((config, table)) = &self.joint {
            let left = config.head * self.per_head + config.left_lane;
            let right = config.head * self.per_head + config.right_lane;
            let a = rc.i((.., left))?.contiguous()?;
            let b = rc.i((.., right))?.contiguous()?;
            let at = (&a.broadcast_mul(&Tensor::full(128u32, n, &self.device)?)? + &b)?;
            let mask = (&cue_masks[left] * &cue_masks[right])?;
            hard = (&hard
                + table
                    .index_select(&at, 0)?
                    .broadcast_mul(&mask.to_dtype(DType::F64)?)?)?;
            let features = self
                .basis
                .index_select(&a, 0)?
                .reshape((n, 4, 1))?
                .broadcast_mul(&self.basis.index_select(&b, 0)?.reshape((n, 1, 4))?)?
                .reshape((n, 16))?;
            let c = crate::geometric_context::q4_shadow_ste(self.var("cue.joint")?.as_tensor())?
                .reshape((1, 16))?;
            graph = (&graph + features.broadcast_mul(&c)?.sum(1)?.broadcast_mul(&mask)?)?;
        }
        if !emission {
            return Ok(Energy {
                native_q24: hard.affine(Q24, 0.)?,
                credit: bind(&hard, graph)?,
            });
        }
        let at = u32_indices(&self.device, input.token_ids.into())?;
        hard = (&hard
            + self
                .signal_hard
                .i(input.status as usize)?
                .index_select(&at, 0)?)?;
        graph = (&graph
            + crate::geometric_context::q4_shadow_ste(self.var("memory.signal")?.as_tensor())?
                .i(input.status as usize)?
                .index_select(&at, 0)?)?;
        let base = Energy {
            native_q24: hard.affine(Q24, 0.)?,
            credit: bind(&hard, graph)?,
        };
        if self.shared_copy {
            let delta = self.continuation_energy(input.post_state, input.local_state)?;
            Ok(Energy {
                native_q24: (&base.native_q24 + delta.native_q24.index_select(&at, 0)?)?,
                credit: (&base.credit + delta.credit.index_select(&at, 0)?)?,
            })
        } else {
            Ok(base)
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn copy_base(
        &self,
        query_roots: &Tensor,
        query_categories: &Tensor,
        cue_query_roots: &Tensor,
        cue_query_categories: &Tensor,
        candidate_roots: &Tensor,
        candidate_categories: &Tensor,
        cue_roots: &Tensor,
        cue_categories: &Tensor,
        response_prefix: &Tensor,
        source_prefix: &Tensor,
        ids: &[u32],
    ) -> Result<Energy> {
        let identity = Tensor::full(
            u32::from(H4Code::IDENTITY.index()),
            self.lanes,
            &self.device,
        )?;
        self.copy_components(
            DeviceCopyInput {
                query_roots,
                query_categories,
                cue_query_roots,
                cue_query_categories,
                candidate_roots,
                candidate_categories,
                cue_roots,
                cue_categories,
                response_prefix,
                source_prefix,
                post_state: &identity,
                local_state: &identity,
                token_ids: ids,
                status: 32,
            },
            false,
        )
    }
    /// Pool every physical alias. This is the preregistered smooth offline
    /// law, not a replacement for native integer Q31 served token selection.
    pub fn pooled_token(
        &mut self,
        _generated: &Tensor,
        _copy: &Tensor,
        _copy_ids: &[u32],
    ) -> Result<DeviceChoice> {
        Err(invalid(
            "retired alias probability pool; use the shared native vector channel",
        ))
    }
    fn native_pool(&self, _step: &DeviceEmissionStep) -> Result<(Tensor, Tensor)> {
        Err(invalid(
            "retired table-exponential writer; no implicit fallback",
        ))
    }
    fn ranking_loss(&self, _step: &DeviceEmissionStep, _target: u32) -> Result<Tensor> {
        Err(invalid(
            "retired probability objective; native vectors use explicit target-address margin",
        ))
    }
    pub fn teacher_objective(
        &mut self,
        bank: &uor_r4_core::native_geometric::learner::native_bank_generate::AdmittedBank,
        answer: &[u32],
        eos: u32,
        baseline: f32,
    ) -> Result<Tensor> {
        if answer.is_empty()
            || answer.len() > 128
            || answer.last() != Some(&eos)
            || answer[..answer.len() - 1].contains(&eos)
            || answer.iter().any(|&id| id as usize >= self.vocab)
            || !baseline.is_finite()
        {
            return Err(invalid("device complete-answer/EOS admission"));
        }
        let mut losses = Vec::new();
        for (i, &target) in answer.iter().enumerate() {
            let step = self.bank_step(bank, &answer[..i])?;
            losses.push(self.ranking_loss(&step, target)?);
        }
        let all = Tensor::stack(&losses, 0)?;
        let shift = all.max(0)?.detach();
        let rank = (all.broadcast_sub(&shift)?.exp()?.mean_all()?.log()? + shift)?;
        if self.greedy {
            self.validate_masters()?;
            return Ok(rank);
        }
        // Keep the entire ancestor coefficient on device; do not download a
        // training loss for constructing the score-function graph.
        let advantage = rank.detach().affine(1., -f64::from(baseline))?;
        let score = Tensor::stack(&self.logs, 0)?
            .sum_all()?
            .broadcast_mul(&advantage)?;
        self.validate_masters()?;
        Ok((&rank + (&score - score.detach())?)?)
    }
    pub fn objective(&self, cost: f32, baseline: f32) -> Result<Tensor> {
        if !cost.is_finite() || !baseline.is_finite() || self.logs.is_empty() {
            return Err(invalid("device credit cost/trajectory invalid"));
        }
        let term = Tensor::stack(&self.logs, 0)?
            .sum_all()?
            .affine(f64::from(cost - baseline), 0.)?;
        Ok((Tensor::new(cost, &self.device)? + (&term - term.detach())?)?)
    }
    /// Explicit end-of-episode/rebinding barrier. Unlike the numerical loops,
    /// this custody boundary may download masters to authenticate their digest.
    pub fn validate_masters(&self) -> Result<()> {
        if parameter_digest(&self.params)? != self.parameters {
            return Err(invalid(
                "device masters changed; discard trajectory and rebind",
            ));
        }
        Ok(())
    }
    fn compose_codes(&self, left: &Tensor, right: &Tensor) -> Result<Tensor> {
        let at = (left.flatten_all()?.broadcast_mul(&Tensor::full(
            120u32,
            left.elem_count(),
            &self.device,
        )?)? + right.flatten_all()?)?;
        Ok(self.product.index_select(&at, 0)?.reshape(left.shape())?)
    }
    fn compiler_rows(
        &mut self,
        source: &str,
        tokenizer: &ByteBpeTokenizer,
        config: &FitConfig,
    ) -> Result<(Tensor, Tensor, Tensor, Tensor, Vec<SourceSpan>)> {
        use crate::geometric_turn_compiler::FeatureMode;
        let spans = crate::relation_compiler::word_spans(source)
            .into_iter()
            .map(|s| SourceSpan {
                start: s.start,
                end: s.end,
            })
            .collect::<Vec<_>>();
        if tokenizer.encode(source).len() > config.max_tokens
            || spans.len() > config.max_words
            || spans.is_empty()
        {
            return Err(invalid("device compiler source budget/words invalid"));
        }
        let mut locals = Vec::new();
        for s in &spans {
            locals.push(self.endpoint(&tokenizer.encode(&source[s.start..s.end]))?);
        }
        let mode = config.feature_mode;
        let ordered = matches!(
            mode,
            FeatureMode::OrderedPrefixCarrier | FeatureMode::OrderedPrefixTransport
        );
        let zero = Tensor::zeros(self.lanes, DType::U32, &self.device)?;
        let mut turn = Vec::new();
        let mut word = Vec::new();
        let mut tm = Vec::new();
        let mut wm = Vec::new();
        if mode == FeatureMode::Endpoint {
            let global = self.endpoint(&tokenizer.encode(source))?;
            turn.push(global.clone());
            tm.push(Tensor::ones(self.lanes, DType::F32, &self.device)?);
            for (s, local) in spans.iter().zip(&locals) {
                let before = self.endpoint(&tokenizer.encode(&source[..s.start]))?;
                let after = self.endpoint(&tokenizer.encode(&source[..s.end]))?;
                word.push(Tensor::cat(
                    &[
                        local,
                        &self.relative_codes(&global, local)?,
                        &self.relative_codes(&before, &after)?,
                    ],
                    0,
                )?);
                wm.push(Tensor::ones(3 * self.lanes, DType::F32, &self.device)?);
            }
        } else {
            let mut prefix = Tensor::full(
                u32::from(H4Code::IDENTITY.index()),
                self.lanes,
                &self.device,
            )?;
            for (i, current) in locals.iter().enumerate() {
                let previous = if i == 0 {
                    zero.clone()
                } else {
                    locals[i - 1].clone()
                };
                let next = locals.get(i + 1).unwrap_or(&zero);
                let transport = if i == 0 {
                    zero.clone()
                } else if mode == FeatureMode::LocalRelative {
                    self.relative_codes(&previous, current)?
                } else {
                    self.compose_codes(&previous, current)?
                };
                let mut row = vec![current.clone(), previous.clone(), transport.clone()];
                let mut words = vec![current.clone(), previous, next.clone(), transport];
                let mut mask = vec![1., f32::from(u8::from(i > 0)), f32::from(u8::from(i > 0))];
                let mut wordmask = vec![
                    1.,
                    f32::from(u8::from(i > 0)),
                    f32::from(u8::from(i + 1 < locals.len())),
                    f32::from(u8::from(i > 0)),
                ];
                if ordered {
                    let added = if i == 0 {
                        zero.clone()
                    } else if mode == FeatureMode::OrderedPrefixCarrier {
                        prefix.clone()
                    } else {
                        self.relative_codes(&prefix, current)?
                    };
                    row.push(added.clone());
                    words.push(added);
                    mask.push(f32::from(u8::from(i > 0)));
                    wordmask.push(f32::from(u8::from(i > 0)));
                    prefix = self.compose_codes(&prefix, current)?;
                }
                turn.push(Tensor::cat(&row, 0)?);
                word.push(Tensor::cat(&words, 0)?);
                tm.push(Tensor::from_vec(
                    mask.into_iter()
                        .flat_map(|m| std::iter::repeat_n(m, self.lanes))
                        .collect::<Vec<_>>(),
                    config.feature_mode.turn_slots(self.lanes),
                    &self.device,
                )?);
                wm.push(Tensor::from_vec(
                    wordmask
                        .into_iter()
                        .flat_map(|m| std::iter::repeat_n(m, self.lanes))
                        .collect::<Vec<_>>(),
                    config.feature_mode.span_slots(self.lanes),
                    &self.device,
                )?);
            }
        }
        Ok((
            Tensor::stack(&turn, 0)?,
            Tensor::stack(&tm, 0)?,
            Tensor::stack(&word, 0)?,
            Tensor::stack(&wm, 0)?,
            spans,
        ))
    }
    /// Only the typed compiler decisions cross back to the host exact store.
    /// Word states, feature gathering, logits and credit remain on device.
    pub fn compile_action(
        &mut self,
        source: &str,
        tokenizer: &ByteBpeTokenizer,
        config: &FitConfig,
        relations: usize,
    ) -> Result<CompiledAction> {
        let (rows, mask, words, wm, spans) = self.compiler_rows(source, tokenizer, config)?;
        let logits = self.head("compiler.act", &rows, &mask, 4)?;
        let act = self.choose(&logits, true)?.index.to_vec1::<u32>()?[0] as usize;
        let logits = self.head("compiler.relation", &rows, &mask, relations + 1)?;
        let relation = self.choose(&logits, true)?.index.to_vec1::<u32>()?[0] as usize;
        if act == 3 || relation == relations {
            return Ok(CompiledAction::Unresolved {
                reason: "native learned none act or relation".into(),
            });
        }
        if act == 2 {
            let logits = self.head("compiler.view", &rows, &mask, 4)?;
            let view = self.choose(&logits, true)?.index.to_vec1::<u32>()?[0];
            return Ok(CompiledAction::Query {
                relation: (relation + 1) as u32,
                view: crate::stack_store::view_from_code(view)
                    .ok_or_else(|| invalid("device compiler view invalid"))?,
            });
        }
        let mut inside = Vec::new();
        for i in 0..spans.len() {
            let scores = self.head(
                "compiler.span",
                &words.i(i)?.unsqueeze(0)?,
                &wm.i(i)?.unsqueeze(0)?,
                2,
            )?;
            inside.push((&scores.i(1)? - &scores.i(0)?)?);
        }
        let mut options = vec![None];
        let mut scores = vec![Tensor::new(0f32, &self.device)?];
        for start in 0..spans.len() {
            let mut sum = Tensor::new(0f32, &self.device)?;
            for end in start..spans.len().min(start + config.max_value_words) {
                sum = (&sum + &inside[end])?;
                scores.push(sum.clone());
                options.push(Some(SourceSpan {
                    start: spans[start].start,
                    end: spans[end].end,
                }));
            }
        }
        let chosen = self
            .choose(&Tensor::stack(&scores, 0)?, true)?
            .index
            .to_vec1::<u32>()?[0] as usize;
        let Some(span) = options[chosen] else {
            return Ok(CompiledAction::Unresolved {
                reason: "native learned span has no positive margin".into(),
            });
        };
        Ok(if act == 0 {
            CompiledAction::Assert {
                relation: (relation + 1) as u32,
                span,
            }
        } else {
            CompiledAction::Correct {
                relation: (relation + 1) as u32,
                span,
            }
        })
    }
    pub fn generate(
        &mut self,
        bank: &uor_r4_core::native_geometric::learner::native_bank_generate::AdmittedBank,
        limits: GenerationLimits,
        eos: u32,
    ) -> Result<uor_r4_core::native_geometric::learner::native_bank_generate::NativeBankGeneration>
    {
        use uor_r4_core::native_geometric::learner::native_bank_generate::{
            GenerationStop, NativeBankGeneration,
        };
        if limits.maximum_tokens == 0 || limits.maximum_tokens > 128 {
            return Err(invalid("device reply admission invalid"));
        }
        let mut ids = Vec::new();
        let mut stop = GenerationStop::OutputLimit;
        for _ in 0..limits.maximum_tokens {
            if bank.base_tokens() + ids.len() >= 128 {
                stop = GenerationStop::ContextLimit;
                break;
            }
            let step = self.bank_step(bank, &ids)?;
            let chosen = if self.greedy {
                self.native_pool(&step)?.1.argmax(0)?.to_scalar::<u32>()?
            } else {
                self.pooled_token(&step.generated, &step.copied, &step.copy_ids)?
                    .index
                    .to_vec1::<u32>()?[0]
            };
            ids.push(chosen);
            if chosen == eos {
                stop = GenerationStop::Eos;
                break;
            }
        }
        self.validate_masters()?;
        Ok(NativeBankGeneration {
            pin: bank.pin().clone(),
            executed_steps: ids.len(),
            omitted_step_traces: ids.len(),
            steps: vec![],
            generated_ids: ids,
            stop,
        })
    }
    pub fn applies_continuation_to_copy(&self) -> bool {
        self.shared_copy
    }
    fn endpoint(&mut self, tokens: &[u32]) -> Result<Tensor> {
        if tokens.is_empty() {
            return Ok(Tensor::full(
                u32::from(H4Code::IDENTITY.index()),
                self.lanes,
                &self.device,
            )?);
        }
        Ok(self.context(tokens)?.retained.i(tokens.len() - 1)?)
    }
    fn cue_endpoint(&mut self, tokens: &[u32]) -> Result<(Tensor, Tensor)> {
        if tokens.is_empty() {
            return Ok((
                Tensor::full(
                    u32::from(H4Code::IDENTITY.index()),
                    self.lanes,
                    &self.device,
                )?,
                Tensor::zeros(self.lanes, DType::U32, &self.device)?,
            ));
        }
        let trace = self.context(tokens)?;
        Ok((
            trace.roots.i(tokens.len() - 1)?,
            trace.categories.i(tokens.len() - 1)?,
        ))
    }
    /// The packet is admitted by the real exact-store/frontend. Numeric source
    /// Context, cue and every pre-token Prefix are produced here on device.
    /// There is no native host step, teacher answer or precomputed score input.
    pub fn bank_step(
        &mut self,
        bank: &uor_r4_core::native_geometric::learner::native_bank_generate::AdmittedBank,
        actual: &[u32],
    ) -> Result<DeviceEmissionStep> {
        use uor_r4_integer::geometric_source_realizer::SourceBankSegment;
        if bank.source_metadata_sha256().is_empty()
            || bank.base_tokens() + actual.len() > 128
            || bank.query_conditioned_selection()
        {
            return Err(invalid("device bank admission/mode differs"));
        }
        let segments = bank
            .borrowed_segments()
            .map_err(|e| invalid(e.to_string()))?;
        let mut tokens = Vec::new();
        let mut positions = Vec::new();
        let mut copy_ids = Vec::new();
        let mut cues = Vec::new();
        let mut prefixes = Vec::new();
        let mut sources = Vec::new();
        let mut preceding: &[u32] = &[];
        for segment in &segments {
            match segment {
                SourceBankSegment::Context { token_ids, .. } => {
                    tokens.extend_from_slice(token_ids);
                    preceding = token_ids;
                }
                SourceBankSegment::Source { view, .. } => {
                    let ids = view.emitted_token_ids();
                    sources.push((preceding, ids));
                    for &id in ids {
                        positions.push(tokens.len());
                        tokens.push(id);
                        copy_ids.push(id);
                    }
                    preceding = &[];
                }
            }
        }
        tokens.extend_from_slice(bank.query_ids());
        tokens.extend_from_slice(actual);
        let context = self.context(&tokens)?;
        let query = context.retained.i(tokens.len() - 1)?;
        let status = bank
            .status_code()
            .ok_or_else(|| invalid("device bank numerical status absent"))?;
        let mut local_tokens = bank.query_ids().to_vec();
        local_tokens.extend_from_slice(actual);
        if copy_ids.is_empty() {
            let local = self.endpoint(&local_tokens)?;
            let generated = self.generated_energy(&query, &local, status)?;
            return Ok(DeviceEmissionStep {
                generated: generated.credit,
                generated_native_q24: generated.native_q24,
                copied: Tensor::zeros(0, DType::F32, &self.device)?,
                copied_native_q24: Tensor::zeros(0, DType::F64, &self.device)?,
                copy_ids,
                post_state: query,
                local_state: local,
            });
        }
        let (cue_query_roots, cue_query_categories) = self.cue_endpoint(bank.query_ids())?;
        let query_roots = context.roots.i(tokens.len() - 1)?;
        let query_categories = context.categories.i(tokens.len() - 1)?;
        let at = u32_indices(&self.device, positions.iter().map(|&p| p as u32).collect())?;
        let candidate_roots = context.roots.index_select(&at, 0)?;
        let candidate_categories = context.categories.index_select(&at, 0)?;
        for (before, ids) in &sources {
            let (roots, categories) = self.cue_endpoint(before)?;
            for _ in *ids {
                cues.push((roots.clone(), categories.clone()));
            }
        }
        let response_prefix = self.endpoint(actual)?;
        for (_, ids) in &sources {
            if ids.is_empty() {
                continue;
            }
            prefixes.push(Tensor::full(
                u32::from(H4Code::IDENTITY.index()),
                self.lanes,
                &self.device,
            )?);
            if ids.len() > 1 {
                let trace = self.context(&ids[..ids.len() - 1])?;
                for i in 0..ids.len() - 1 {
                    prefixes.push(trace.retained.i(i)?);
                }
            }
        }
        let cue_roots = Tensor::stack(&cues.iter().map(|(r, _)| r.clone()).collect::<Vec<_>>(), 0)?;
        let cue_categories =
            Tensor::stack(&cues.iter().map(|(_, c)| c.clone()).collect::<Vec<_>>(), 0)?;
        let source_prefix = Tensor::stack(&prefixes, 0)?;
        // Donor consumes Copy BEFORE continuation/status, exactly as native
        // bridge selection. Zero U/signal arguments here cannot leak emission
        // factors into donor selection; add them only after post-state exists.
        let base = self.copy_base(
            &query_roots,
            &query_categories,
            &cue_query_roots,
            &cue_query_categories,
            &candidate_roots,
            &candidate_categories,
            &cue_roots,
            &cue_categories,
            &response_prefix,
            &source_prefix,
            &copy_ids,
        )?;
        let selected = if self.greedy {
            base.native_q24.argmax(0)?.reshape(1)?
        } else {
            self.choose(&base.credit, true)?.index
        };
        let selected_position = at.index_select(&selected, 0)?;
        let source = context
            .retained
            .index_select(&selected_position, 0)?
            .flatten_all()?;
        let post = self.bridge(&query, &source)?;
        let local = self.endpoint(&local_tokens)?;
        let generated = self.generated_energy(&post, &local, status)?;
        let copied = self.copy_components(
            DeviceCopyInput {
                query_roots: &query_roots,
                query_categories: &query_categories,
                cue_query_roots: &cue_query_roots,
                cue_query_categories: &cue_query_categories,
                candidate_roots: &candidate_roots,
                candidate_categories: &candidate_categories,
                cue_roots: &cue_roots,
                cue_categories: &cue_categories,
                response_prefix: &response_prefix,
                source_prefix: &source_prefix,
                post_state: &post,
                local_state: &local,
                token_ids: &copy_ids,
                status,
            },
            true,
        )?;
        Ok(DeviceEmissionStep {
            generated: generated.credit,
            generated_native_q24: generated.native_q24,
            copied: copied.credit,
            copied_native_q24: copied.native_q24,
            copy_ids,
            post_state: post,
            local_state: local,
        })
    }
}

pub use crate::native_vector_channel::{native_vector_channel,carry_native_store_read,DeviceVectorStoreRead,DeviceVectorStoreValue};
