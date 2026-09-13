//! Read-only causal-credit and routing diagnostics over frozen parameters.
//! This module never changes the learner or checkpoint implementation binding.
use super::artifact::BoundGeometry;
use super::circuit::{PrimitiveExport, INPUT_BITS};
use super::engine::{EngineError, ForwardTrace, Head, Policy, RuntimeSession};
use super::inputs::Phase;
use super::objects::{Reference, Symbol};
use super::policy::{
    head_range, CompiledPolicy, Parameters, SampleCounts, SampledPolicy, PARAMETER_COUNT,
};
use super::training::{cross_entropy, SerialLoo};
use serde::Serialize;
use std::time::Instant;
type Result<T> = std::result::Result<T, EngineError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
/// Episode-relative addresses for aligned-position comparisons. Equal result
/// ordinals across divergent trajectories do not imply equal derivations or
/// payloads; neither variant is a semantic state distance.
pub enum CompactReference {
    LocalOccurrence { turn: u64, start: u64, end: u64 },
    Result { id: u64 },
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ParticleTrace {
    pub emission_contexts: Vec<u16>,
    pub offered_symbols: Vec<u16>,
    pub roots_before: Vec<[u16; 4]>,
    /// Descriptive IDs are local to this fresh session; they are not state keys.
    pub selected: Vec<[Option<CompactReference>; 2]>,
}
#[derive(Debug)]
pub struct DiagnosticBatch {
    pub gradient: Vec<f64>,
    pub direct: Vec<f64>,
    pub score_credit: Vec<f64>,
    pub mean_ce: f64,
    pub prompt_ce: f64,
    pub response_ce: f64,
    pub counts: SampleCounts,
    pub particles: Vec<ParticleTrace>,
    pub elapsed_us: u128,
}
#[derive(Debug, Serialize)]
pub struct DeterministicTrace {
    pub trace: ParticleTrace,
    pub mean_ce: f64,
    pub prompt_ce: f64,
    pub response_ce: f64,
    pub elapsed_us: u128,
}
fn validate(document: &[u16], prompt_len: usize) -> Result<()> {
    if document.is_empty()
        || document.len() > 64
        || document.last() != Some(&256)
        || document[..document.len() - 1].iter().any(|&s| s > 255)
        || prompt_len == 0
        || prompt_len >= document.len()
    {
        return Err(EngineError::Invalid(
            "diagnostic bytes/terminalEOS/nonempty prompt and response",
        ));
    }
    Ok(())
}
fn symbol(target: u16) -> Symbol {
    if target == 256 {
        Symbol::Eos
    } else {
        Symbol::Byte(target as u8)
    }
}
fn capture(
    trace: &mut ParticleTrace,
    completed: &ForwardTrace,
    emission_context: u16,
) -> Result<()> {
    if completed.contexts[Phase::Emit as usize] != Some(emission_context)
        || completed.circuit_calls() > 8
        || completed.candidate_scores > 530
        || completed.scalar_decodes > 2
        || completed.selected_operations > 1
    {
        return Err(EngineError::Invalid("diagnostic trace/event bounds"));
    }
    trace.emission_contexts.push(emission_context);
    trace.offered_symbols.push(match completed.output {
        Symbol::Byte(byte) => u16::from(byte),
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
    Ok(())
}
struct Captured<P> {
    inner: P,
    emission_context: Option<u16>,
}
impl<P: Policy> Policy for Captured<P> {
    fn context(&mut self, phase: Phase, input: &[bool; INPUT_BITS]) -> Result<u16> {
        let context = self.inner.context(phase, input)?;
        if phase == Phase::Emit {
            self.emission_context = Some(context);
        }
        Ok(context)
    }
    fn choice(&mut self, head: Head, context: u16, legal: &[bool]) -> Result<usize> {
        self.inner.choice(head, context, legal)
    }
}

pub fn instrumented_batch(
    parameters: &Parameters,
    g: &BoundGeometry,
    document: &[u16],
    prompt_len: usize,
    event_seed: u64,
    document_id: u64,
) -> Result<DiagnosticBatch> {
    validate(document, prompt_len)?;
    let timer = Instant::now();
    let before = parameters.digest();
    let mut accumulator = SerialLoo::new(parameters.values(), 4)
        .map_err(|_| EngineError::Invalid("diagnostic LOO setup"))?;
    let mut direct = vec![0.0; PARAMETER_COUNT];
    let mut mean_ce = 0.0;
    let mut prompt_sum = 0.0;
    let mut response_sum = 0.0;
    let mut counts = SampleCounts::default();
    let mut particles = Vec::with_capacity(4);
    for particle in 0..4 {
        let mut runtime = RuntimeSession::new(before, g, particle + 1)?;
        let mut policy = Captured {
            inner: SampledPolicy::new(parameters, event_seed, particle, document_id),
            emission_context: None,
        };
        let mut trace = ParticleTrace::default();
        for (position, &target) in document.iter().enumerate() {
            policy.emission_context = None;
            let loss_before = policy.inner.mean_loss();
            policy
                .inner
                .set_target(usize::from(target), document.len())?;
            let offered = runtime.predict(g, &mut policy)?;
            if policy.inner.target_pending() {
                return Err(EngineError::Invalid("diagnostic EMIT target unconsumed"));
            }
            let conditional_ce = (policy.inner.mean_loss() - loss_before) * document.len() as f64;
            if position < prompt_len {
                prompt_sum += conditional_ce;
            } else {
                response_sum += conditional_ce;
            }
            let completed = runtime.observe(symbol(target), offered.offer.id, g, &mut policy)?;
            capture(
                &mut trace,
                &completed,
                policy
                    .emission_context
                    .ok_or(EngineError::Invalid("diagnostic missing EMIT context"))?,
            )?;
        }
        let current = policy.inner.counts();
        if current.scored_positions != document.len() as u64
            || current.gate_events != current.circuit_calls * super::circuit::GATES as u64
            || current.circuit_calls != runtime.work().circuit_calls
        {
            return Err(EngineError::Invalid("diagnostic score/event accounting"));
        }
        accumulator
            .add_particle(
                parameters.values(),
                policy.inner.mean_loss(),
                policy.inner.score(),
                policy.inner.direct(),
            )
            .map_err(|_| EngineError::Invalid("diagnostic LOO particle"))?;
        for (total, &value) in direct.iter_mut().zip(policy.inner.direct()) {
            *total += value;
        }
        mean_ce += policy.inner.mean_loss() / 4.0;
        counts.circuit_calls += current.circuit_calls;
        counts.gate_events += current.gate_events;
        counts.head_events += current.head_events;
        counts.scored_positions += current.scored_positions;
        particles.push(trace);
    }
    let gradient = accumulator
        .finish(parameters.values())
        .map_err(|_| EngineError::Invalid("diagnostic LOO finish"))?;
    for value in &mut direct {
        *value /= 4.0;
    }
    let score_credit = gradient
        .iter()
        .zip(&direct)
        .map(|(&total, &direct)| total - direct)
        .collect::<Vec<_>>();
    let prompt_ce = prompt_sum / (4 * prompt_len) as f64;
    let response_ce = response_sum / (4 * (document.len() - prompt_len)) as f64;
    if parameters.digest() != before
        || ![mean_ce, prompt_ce, response_ce]
            .iter()
            .all(|v| v.is_finite())
        || gradient
            .iter()
            .chain(&direct)
            .chain(&score_credit)
            .any(|v| !v.is_finite())
    {
        return Err(EngineError::Invalid(
            "diagnostic finite credit/frozen parameters",
        ));
    }
    Ok(DiagnosticBatch {
        gradient,
        direct,
        score_credit,
        mean_ce,
        prompt_ce,
        response_ce,
        counts,
        particles,
        elapsed_us: timer.elapsed().as_micros(),
    })
}

struct ScoredCompiled<'a> {
    inner: CompiledPolicy<'a>,
    parameters: &'a Parameters,
    target: Option<usize>,
    conditional_ce: Option<f64>,
}
impl Policy for ScoredCompiled<'_> {
    fn context(&mut self, phase: Phase, input: &[bool; INPUT_BITS]) -> Result<u16> {
        self.inner.context(phase, input)
    }
    fn choice(&mut self, head: Head, context: u16, legal: &[bool]) -> Result<usize> {
        if head == Head::Emission {
            // The target is consulted after the causal context already exists.
            // It affects only this diagnostic scalar, never a serving decision.
            let target = self
                .target
                .take()
                .ok_or(EngineError::Invalid("deterministic missing target"))?;
            let (offset, n) = head_range(head, context)?;
            self.conditional_ce = Some(
                cross_entropy(
                    &self.parameters.values()[offset..offset + n],
                    legal,
                    target,
                    1,
                )
                .map_err(|_| EngineError::Invalid("deterministic diagnostic CE"))?
                .loss,
            );
        }
        self.inner.choice(head, context, legal)
    }
}
pub fn deterministic_trace(
    parameters: &Parameters,
    compiled: &PrimitiveExport,
    g: &BoundGeometry,
    document: &[u16],
    prompt_len: usize,
) -> Result<DeterministicTrace> {
    validate(document, prompt_len)?;
    let timer = Instant::now();
    if &parameters.compile()? != compiled {
        return Err(EngineError::Invalid(
            "deterministic diagnostic compiled parameter binding",
        ));
    }
    let mut runtime = RuntimeSession::new(parameters.digest(), g, 1)?;
    let mut policy = Captured {
        inner: ScoredCompiled {
            inner: CompiledPolicy { compiled },
            parameters,
            target: None,
            conditional_ce: None,
        },
        emission_context: None,
    };
    let mut trace = ParticleTrace::default();
    let mut prompt_sum = 0.0;
    let mut response_sum = 0.0;
    for (position, &target) in document.iter().enumerate() {
        policy.emission_context = None;
        policy.inner.target = Some(usize::from(target));
        policy.inner.conditional_ce = None;
        let offered = runtime.predict(g, &mut policy)?;
        let ce = policy.inner.conditional_ce.ok_or(EngineError::Invalid(
            "deterministic diagnostic unscored target",
        ))?;
        if position < prompt_len {
            prompt_sum += ce;
        } else {
            response_sum += ce;
        }
        let completed = runtime.observe(symbol(target), offered.offer.id, g, &mut policy)?;
        capture(
            &mut trace,
            &completed,
            policy
                .emission_context
                .ok_or(EngineError::Invalid("deterministic missing context"))?,
        )?;
    }
    Ok(DeterministicTrace {
        trace,
        mean_ce: (prompt_sum + response_sum) / document.len() as f64,
        prompt_ce: prompt_sum / prompt_len as f64,
        response_ce: response_sum / (document.len() - prompt_len) as f64,
        elapsed_us: timer.elapsed().as_micros(),
    })
}

#[cfg(test)]
#[path = "stability_tests.rs"]
mod tests;
