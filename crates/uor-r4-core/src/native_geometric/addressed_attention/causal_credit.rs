//! Frozen single-forward comparison of complete-trajectory and causal suffix LOO.
//! The sampler mirrors the preserved policy, with sparse scores recorded at CE
//! boundaries. This module changes neither serving nor learned parameters.
use super::artifact::BoundGeometry;
use super::circuit::{CircuitError, Topology, GATES, INPUT_BITS};
use super::engine::{EngineError, Head, Policy, RuntimeSession};
use super::inputs::Phase;
use super::objects::{Reference, Symbol};
use super::policy::{head_range, Parameters, SampleCounts, PARAMETER_COUNT, WIRING_SEED};
use super::stability::{CompactReference, ParticleTrace};
use super::training::{bernoulli, categorical, cross_entropy, EventKey, EventKind, SerialLoo};
use std::time::Instant;
type Result<T> = std::result::Result<T, EngineError>;
pub const MAX_TAPE_BYTES: usize = 64 * 1024 * 1024;
// Conservative complete-path bound: eight circuits and at most24 categorical
// heads, each with at most257 outcomes. Illegal zero-score coordinates remain
// explicit; no model-sized vector is saved at any target position.
const SCORES_PER_POSITION: usize = 8 * GATES + 24 * 257;

#[derive(Clone, Copy, Debug)]
pub(crate) struct ScoreEvent {
    pub parameter: u32,
    pub score: f64,
}
#[derive(Clone, Debug)]
pub(crate) struct ScoreTape {
    pub events: Vec<ScoreEvent>,
    /// Starts at0, then one offset immediately after each CE before its emission
    /// draw. Group b uses losses[b..]; events after the last offset have no cost.
    pub boundaries: Vec<usize>,
}
impl ScoreTape {
    fn new(positions: usize) -> Result<Self> {
        let mut events = Vec::new();
        events
            .try_reserve_exact(positions * SCORES_PER_POSITION)
            .map_err(|_| EngineError::Invalid("causal tape allocation"))?;
        let mut boundaries = Vec::with_capacity(positions + 1);
        boundaries.push(0);
        Ok(Self { events, boundaries })
    }
    fn record(&mut self, parameter: usize, score: f64) -> Result<()> {
        if self.events.len() == self.events.capacity()
            || parameter >= PARAMETER_COUNT
            || !score.is_finite()
        {
            return Err(EngineError::Invalid("causal sparse event bound/domain"));
        }
        self.events.push(ScoreEvent {
            parameter: parameter as u32,
            score,
        });
        Ok(())
    }
    fn bytes(&self) -> usize {
        self.events.capacity() * std::mem::size_of::<ScoreEvent>()
            + self.boundaries.capacity() * std::mem::size_of::<usize>()
    }
}

/// Each loss is already divided by full document length. Returned weights
/// include division by particle count and a same-target-boundary independent
/// other-particle baseline. Final boundary weights are exactly zero.
pub(crate) fn suffix_loo_weights(losses: &[Vec<f64>]) -> Result<Vec<Vec<f64>>> {
    let p = losses.len();
    if !(2..=4).contains(&p)
        || losses[0].is_empty()
        || losses[0].len() > 64
        || losses
            .iter()
            .any(|v| v.len() != losses[0].len() || v.iter().any(|x| !x.is_finite()))
    {
        return Err(EngineError::Invalid("causal suffix loss shape/domain"));
    }
    let t = losses[0].len();
    let mut suffix = vec![vec![0.0; t + 1]; p];
    let mut sum = vec![0.0; t + 1];
    for i in 0..p {
        for b in (0..t).rev() {
            suffix[i][b] = losses[i][b] + suffix[i][b + 1];
            sum[b] += suffix[i][b];
        }
    }
    for row in &mut suffix {
        for b in 0..t {
            row[b] = (row[b] - (sum[b] - row[b]) / (p - 1) as f64) / p as f64;
            if !row[b].is_finite() {
                return Err(EngineError::Invalid("causal suffix overflow"));
            }
        }
    }
    Ok(suffix)
}

/// Production sparse reducer, exposed within the module tree for exact tests.
/// Visits only saved parameter-score events, never counterfactual addresses.
pub(crate) fn reduce_causal_score(
    tapes: &[ScoreTape],
    losses: &[Vec<f64>],
    parameter_count: usize,
) -> Result<Vec<f64>> {
    let weights = suffix_loo_weights(losses)?;
    if tapes.len() != losses.len() || parameter_count == 0 || parameter_count > PARAMETER_COUNT {
        return Err(EngineError::Invalid("causal reducer shape"));
    }
    let mut gradient = vec![0.0; parameter_count];
    for (i, tape) in tapes.iter().enumerate() {
        let t = losses[i].len();
        if tape.boundaries.len() != t + 1
            || tape.boundaries.first() != Some(&0)
            || tape.boundaries.windows(2).any(|pair| pair[0] > pair[1])
            || tape.boundaries[t] > tape.events.len()
            || tape
                .events
                .iter()
                .any(|e| e.parameter as usize >= parameter_count || !e.score.is_finite())
        {
            return Err(EngineError::Invalid("causal reducer tape boundary/domain"));
        }
        for b in 0..t {
            for event in &tape.events[tape.boundaries[b]..tape.boundaries[b + 1]] {
                gradient[event.parameter as usize] += weights[i][b] * event.score;
            }
        }
    }
    if gradient.iter().any(|x| !x.is_finite()) {
        return Err(EngineError::Invalid("causal reducer overflow"));
    }
    Ok(gradient)
}

struct TapedPolicy<'a> {
    parameters: &'a Parameters,
    topology: Topology,
    key: EventKey,
    phase: Phase,
    target: Option<(usize, usize)>,
    score: Vec<f64>,
    direct: Vec<f64>,
    loss: f64,
    losses: Vec<f64>,
    tape: ScoreTape,
    counts: SampleCounts,
}
impl<'a> TapedPolicy<'a> {
    fn new(
        parameters: &'a Parameters,
        seed: u64,
        particle: u64,
        document: u64,
        positions: usize,
    ) -> Result<Self> {
        Ok(Self {
            parameters,
            topology: Topology::seeded(WIRING_SEED),
            key: EventKey {
                seed,
                batch: 0,
                particle,
                document,
                event: 0,
                phase: 0,
                kind: EventKind::Gate,
                slot: 0,
            },
            phase: Phase::QueryA,
            target: None,
            score: vec![0.0; PARAMETER_COUNT],
            direct: vec![0.0; PARAMETER_COUNT],
            loss: 0.0,
            losses: Vec::with_capacity(positions),
            tape: ScoreTape::new(positions)?,
            counts: SampleCounts::default(),
        })
    }
    fn set_target(&mut self, target: usize, normalizer: usize) -> Result<()> {
        if target > 256 || normalizer == 0 || normalizer > 64 || self.target.is_some() {
            return Err(EngineError::Invalid("causal target/normalizer"));
        }
        self.target = Some((target, normalizer));
        Ok(())
    }
}
impl Policy for TapedPolicy<'_> {
    fn context(&mut self, phase: Phase, input: &[bool; INPUT_BITS]) -> Result<u16> {
        self.phase = phase;
        self.counts.circuit_calls += 1;
        let parameters = self.parameters;
        let score = &mut self.score;
        let key = &mut self.key;
        let counts = &mut self.counts;
        let tape = &mut self.tape;
        let result = self.topology.evaluate_with(input, |row| {
            key.phase = phase as u8;
            key.kind = EventKind::Gate;
            key.slot = (row >> 4) as u32;
            let draw =
                bernoulli(parameters.values()[row], *key).map_err(|_| CircuitError::NonFinite)?;
            key.event = key.event.checked_add(1).ok_or(CircuitError::Domain)?;
            score[row] += draw.score;
            counts.gate_events += 1;
            tape.record(row, draw.score)
                .map_err(|_| CircuitError::Domain)?;
            Ok(draw.outcome)
        })?;
        Ok(result)
    }
    fn choice(&mut self, head: Head, context: u16, legal: &[bool]) -> Result<usize> {
        let (offset, n) = head_range(head, context)?;
        if legal.len() != n {
            return Err(EngineError::Invalid("causal sample mask"));
        }
        let logits = &self.parameters.values()[offset..offset + n];
        if matches!(head, Head::Emission) {
            if let Some((target, normalizer)) = self.target.take() {
                let ce = cross_entropy(logits, legal, target, normalizer)
                    .map_err(|_| EngineError::Invalid("causal conditional CE"))?;
                self.loss += ce.loss;
                for (i, v) in ce.direct.into_iter().enumerate() {
                    self.direct[offset + i] += v;
                }
                self.counts.scored_positions += 1;
                self.losses.push(ce.loss);
                // All events so far precede this CE. Emission and subsequent
                // OBSERVE/KEY draws enter the next target-cost boundary.
                self.tape.boundaries.push(self.tape.events.len());
            }
        }
        self.key.phase = self.phase as u8;
        self.key.kind = EventKind::Head;
        self.key.slot = offset as u32;
        let draw = categorical(logits, legal, self.key)
            .map_err(|_| EngineError::Invalid("causal categorical sample"))?;
        self.key.event = self
            .key
            .event
            .checked_add(1)
            .ok_or(EngineError::Invalid("causal event overflow"))?;
        self.counts.head_events += 1;
        for (i, v) in draw.score.into_iter().enumerate() {
            self.score[offset + i] += v;
            self.tape.record(offset + i, v)?;
        }
        Ok(draw.outcome)
    }
}

#[derive(Debug)]
pub struct PairedBatch {
    pub old_gradient: Vec<f64>,
    pub causal_gradient: Vec<f64>,
    pub direct: Vec<f64>,
    pub mean_ce: f64,
    pub prompt_ce: f64,
    pub response_ce: f64,
    pub counts: SampleCounts,
    pub particles: Vec<ParticleTrace>,
    /// Peak retained tape capacities including boundary offsets and saved CE.
    /// Dense gradients/accumulators and process RSS are separate measurements.
    pub tape_bytes: usize,
    pub elapsed_us: u128,
}

pub fn paired_batch(
    parameters: &Parameters,
    g: &BoundGeometry,
    document: &[u16],
    prompt_len: usize,
    event_seed: u64,
    document_id: u64,
) -> Result<PairedBatch> {
    if document.is_empty()
        || document.len() > 64
        || document.last() != Some(&256)
        || document[..document.len() - 1].iter().any(|&s| s > 255)
        || prompt_len == 0
        || prompt_len >= document.len()
    {
        return Err(EngineError::Invalid("causal diagnostic document/prompt"));
    }
    let timer = Instant::now();
    let before = parameters.digest();
    let mut old = SerialLoo::new(parameters.values(), 4)
        .map_err(|_| EngineError::Invalid("causal old LOO setup"))?;
    let mut direct = vec![0.0; PARAMETER_COUNT];
    let mut tapes = Vec::with_capacity(4);
    let mut losses = Vec::with_capacity(4);
    let mut counts = SampleCounts::default();
    let mut particles = Vec::with_capacity(4);
    let mut mean_ce = 0.0;
    let mut prompt_sum = 0.0;
    let mut response_sum = 0.0;
    let mut tape_bytes = 4 * (std::mem::size_of::<ScoreTape>() + std::mem::size_of::<Vec<f64>>());
    for particle in 0..4 {
        let mut runtime = RuntimeSession::new(before, g, particle + 1)?;
        let mut policy = TapedPolicy::new(
            parameters,
            event_seed,
            particle,
            document_id,
            document.len(),
        )?;
        tape_bytes += policy.tape.bytes() + policy.losses.capacity() * std::mem::size_of::<f64>();
        if tape_bytes > MAX_TAPE_BYTES {
            return Err(EngineError::Invalid("causal tape memory limit"));
        }
        let mut trace = ParticleTrace::default();
        for (position, &target) in document.iter().enumerate() {
            let loss_before = policy.loss;
            policy.set_target(usize::from(target), document.len())?;
            let offered = runtime.predict(g, &mut policy)?;
            if policy.target.is_some() {
                return Err(EngineError::Invalid("causal target unconsumed"));
            }
            // Match the preserved diagnostic's reported CE arithmetic exactly.
            let ce = (policy.loss - loss_before) * document.len() as f64;
            if position < prompt_len {
                prompt_sum += ce;
            } else {
                response_sum += ce;
            }
            let actual = if target == 256 {
                Symbol::Eos
            } else {
                Symbol::Byte(target as u8)
            };
            let completed = runtime.observe(actual, offered.offer.id, g, &mut policy)?;
            if completed.circuit_calls() > 8
                || completed.candidate_scores > 530
                || completed.scalar_decodes > 2
                || completed.selected_operations > 1
            {
                return Err(EngineError::Invalid("causal complete path bound"));
            }
            trace.emission_contexts.push(
                completed.contexts[Phase::Emit as usize]
                    .ok_or(EngineError::Invalid("causal missing EMITcontext"))?,
            );
            trace.offered_symbols.push(match completed.output {
                Symbol::Byte(b) => u16::from(b),
                Symbol::Eos => 256,
            });
            trace.roots_before.push(completed.roots_before);
            trace.selected.push(completed.selected.map(|r| {
                r.map(|r| match r {
                    Reference::Occurrence {
                        turn, start, end, ..
                    } => CompactReference::LocalOccurrence { turn, start, end },
                    Reference::Result { id, .. } => CompactReference::Result { id },
                })
            }));
        }
        let c = policy.counts;
        if c.scored_positions != document.len() as u64
            || c.gate_events != c.circuit_calls * GATES as u64
            || c.circuit_calls != runtime.work().circuit_calls
            || c.head_events > 24 * document.len() as u64
            || policy.losses.len() != document.len()
        {
            return Err(EngineError::Invalid("causal score/event accounting"));
        }
        old.add_particle(
            parameters.values(),
            policy.loss,
            &policy.score,
            &policy.direct,
        )
        .map_err(|_| EngineError::Invalid("causal old LOO particle"))?;
        for (total, &value) in direct.iter_mut().zip(&policy.direct) {
            *total += value;
        }
        mean_ce += policy.loss / 4.0;
        counts.circuit_calls += c.circuit_calls;
        counts.gate_events += c.gate_events;
        counts.head_events += c.head_events;
        counts.scored_positions += c.scored_positions;
        tapes.push(policy.tape);
        losses.push(policy.losses);
        particles.push(trace);
    }
    let old_gradient = old
        .finish(parameters.values())
        .map_err(|_| EngineError::Invalid("causal old LOO finish"))?;
    let mut causal_gradient = reduce_causal_score(&tapes, &losses, PARAMETER_COUNT)?;
    for (causal, direct) in causal_gradient.iter_mut().zip(&mut direct) {
        *direct /= 4.0;
        *causal += *direct;
    }
    let prompt_ce = prompt_sum / (4 * prompt_len) as f64;
    let response_ce = response_sum / (4 * (document.len() - prompt_len)) as f64;
    if before != parameters.digest()
        || ![mean_ce, prompt_ce, response_ce]
            .iter()
            .all(|v| v.is_finite())
        || old_gradient
            .iter()
            .chain(&causal_gradient)
            .chain(&direct)
            .any(|v| !v.is_finite())
    {
        return Err(EngineError::Invalid("causal finite/frozen result"));
    }
    Ok(PairedBatch {
        old_gradient,
        causal_gradient,
        direct,
        mean_ce,
        prompt_ce,
        response_ce,
        counts,
        particles,
        tape_bytes,
        elapsed_us: timer.elapsed().as_micros(),
    })
}

#[cfg(test)]
#[path = "causal_credit_tests.rs"]
mod tests;
