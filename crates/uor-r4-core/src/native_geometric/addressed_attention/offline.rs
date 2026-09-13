//! Frozen-parameter complete forward/backward profiling. No optimizer or fit.
use super::artifact::Model;
use super::engine::{EngineError, RuntimeSession};
use super::objects::{Reference, Symbol};
use super::policy::{Parameters, SampleCounts, SampledPolicy, PARAMETER_COUNT};
use super::training::SerialLoo;
use serde::Serialize;
use std::time::Instant;
type Result<T> = std::result::Result<T, EngineError>;

#[derive(Debug, Serialize)]
pub struct ParticleReport {
    pub particle: u64,
    pub positions: usize,
    pub mean_conditional_ce: f64,
    pub counts: SampleCounts,
    pub candidate_scores: u64,
    pub selected_reads: u64,
    pub eight_phase_ticks: u64,
    pub scalar_decodes: u64,
    pub selected_operations: u64,
    pub action_counts: [u64; 7],
    pub offered_symbols: Vec<u16>,
    pub selected_occurrences: Vec<[Option<u64>; 2]>,
    pub final_roots: [u16; 4],
    pub snapshot_bytes: usize,
    pub trace_digest: String,
    pub elapsed_us: u128,
}
#[derive(Debug, Serialize)]
pub struct DryRunReport {
    pub status: &'static str,
    pub particles: Vec<ParticleReport>,
    pub parameter_count: usize,
    pub parameter_digest_before: String,
    pub parameter_digest_after: String,
    pub parameter_updates: u64,
    pub gradient_digest: String,
    pub gradient_l2: f64,
    pub gradient_nonzero: usize,
    pub gradient_all_finite: bool,
    pub initialization_mean_conditional_ce: f64,
    pub max_dense_training_bytes: usize,
    pub gate_tape_bytes: usize,
    pub model_bytes: usize,
    pub max_session_snapshot_bytes: usize,
    pub runtime_session_size_bytes: usize,
    pub elapsed_us: u128,
    pub language_fit_calls: u64,
    pub general_language_quality: &'static str,
    pub memory_note: &'static str,
}
fn symbol(s: Symbol) -> u16 {
    match s {
        Symbol::Byte(b) => u16::from(b),
        Symbol::Eos => 256,
    }
}
fn occurrence(r: Option<Reference>) -> Option<u64> {
    match r {
        Some(Reference::Occurrence { start, .. }) => Some(start),
        _ => None,
    }
}
fn action(a: super::objects::Action) -> usize {
    use super::objects::Action::*;
    match a {
        Hold => 0,
        AcquireA => 1,
        AcquireB => 2,
        AddAB => 3,
        SubAB => 4,
        Advance => 5,
        Clear => 6,
    }
}

/// Four fresh actual trajectories share frozen parameters and the same data.
/// Full mean trajectory cost multiplies each event score. Dense score/direct
/// arrays replace a gate tape here; their actual capacity is reported explicitly.
pub fn frozen_four_particle(
    parameters: &Parameters,
    model: &Model,
    data: &[u8],
    seed: u64,
) -> Result<DryRunReport> {
    if data.is_empty()
        || data.len() > 512
        || parameters.digest() != model.provenance().parameter_digest
        || parameters.seed() != model.provenance().seed
        || *blake3::hash(data).as_bytes() != model.provenance().training_data_digest
    {
        return Err(EngineError::Invalid(
            "frozen dry-run data/parameter binding",
        ));
    }
    let start = Instant::now();
    if &parameters.compile()? != model.compiled() {
        return Err(EngineError::Invalid("frozen compiled parameter binding"));
    }
    let before = parameters.digest();
    let mut accumulator =
        SerialLoo::new(parameters.values(), 4).map_err(|_| EngineError::Invalid("LOO setup"))?;
    let mut reports = Vec::with_capacity(4);
    let mut max_dense = 0;
    let mut max_snapshot = 0;
    for particle in 0..4 {
        let timer = Instant::now();
        let mut runtime = RuntimeSession::new(model.id(), model.geometry(), particle + 1)?;
        let mut policy = SampledPolicy::new(parameters, seed, particle, 0);
        max_dense = max_dense.max(PARAMETER_COUNT * 8 * 4 + policy.scratch_bytes());
        let mut trace = blake3::Hasher::new();
        let mut selected_reads = 0;
        let mut eight_phase_ticks = 0;
        let mut action_counts = [0; 7];
        let mut offered_symbols = Vec::with_capacity(data.len());
        let mut selected_occurrences = Vec::with_capacity(data.len());
        for (position, &byte) in data.iter().enumerate() {
            policy.set_target(usize::from(byte), data.len())?;
            let offered = runtime.predict(model.geometry(), &mut policy)?;
            if policy.target_pending() {
                return Err(EngineError::Invalid("EMIT target unconsumed"));
            }
            let completed = runtime.observe(
                Symbol::Byte(byte),
                offered.offer.id,
                model.geometry(),
                &mut policy,
            )?;
            if completed.circuit_calls() > 8
                || completed.candidate_scores > 530
                || completed.scalar_decodes > 2
                || completed.selected_operations > 1
            {
                return Err(EngineError::Invalid("complete path bound"));
            }
            if completed.circuit_calls() == 8 {
                eight_phase_ticks += 1;
            }
            selected_reads += completed.selected.iter().flatten().count() as u64;
            action_counts[action(completed.action)] += 1;
            offered_symbols.push(symbol(offered.offer.symbol));
            selected_occurrences.push(completed.selected.map(occurrence));
            trace.update(&(position as u64).to_le_bytes());
            trace.update(format!("{completed:?}").as_bytes());
        }
        let counts = policy.counts();
        if counts.scored_positions != data.len() as u64
            || counts.gate_events != counts.circuit_calls * super::circuit::GATES as u64
            || counts.circuit_calls != runtime.work().circuit_calls
        {
            return Err(EngineError::Invalid("score/event accounting"));
        }
        if !policy.mean_loss().is_finite()
            || policy
                .score()
                .iter()
                .chain(policy.direct())
                .any(|v| !v.is_finite())
        {
            return Err(EngineError::Invalid("nonfinite path credit"));
        }
        accumulator
            .add_particle(
                parameters.values(),
                policy.mean_loss(),
                policy.score(),
                policy.direct(),
            )
            .map_err(|_| EngineError::Invalid("LOO particle"))?;
        let snapshot = runtime.snapshot()?;
        max_snapshot = max_snapshot.max(snapshot.len());
        if RuntimeSession::restore(&snapshot, model.id(), model.geometry())? != runtime {
            return Err(EngineError::Invalid("sampled completed snapshot"));
        }
        reports.push(ParticleReport {
            particle,
            positions: data.len(),
            mean_conditional_ce: policy.mean_loss(),
            counts,
            candidate_scores: runtime.work().candidate_scores,
            selected_reads,
            eight_phase_ticks,
            scalar_decodes: runtime.work().scalar_decodes,
            selected_operations: runtime.work().selected_operations,
            action_counts,
            offered_symbols,
            selected_occurrences,
            final_roots: runtime.roots(),
            snapshot_bytes: snapshot.len(),
            trace_digest: trace.finalize().to_hex().to_string(),
            elapsed_us: timer.elapsed().as_micros(),
        });
    }
    let gradient = accumulator
        .finish(parameters.values())
        .map_err(|_| EngineError::Invalid("LOO finish"))?;
    let mut gradient_hash = blake3::Hasher::new();
    let mut sum_squares = 0.0;
    let mut nonzero = 0;
    for &v in &gradient {
        if !v.is_finite() {
            return Err(EngineError::Invalid("nonfinite complete gradient"));
        }
        gradient_hash.update(&v.to_bits().to_le_bytes());
        sum_squares += v * v;
        if v != 0.0 {
            nonzero += 1;
        }
    }
    let after = parameters.digest();
    if before != after
        || max_dense > 512 * 1024 * 1024
        || max_snapshot > 65536
        || !sum_squares.is_finite()
    {
        return Err(EngineError::Invalid("dry-run freeze/memory/gradient"));
    }
    let mean = reports.iter().map(|r| r.mean_conditional_ce).sum::<f64>() / 4.0;
    Ok(DryRunReport{status:"PASS_FROZEN_FORWARD_BACKWARD_DRY_RUN",particles:reports,parameter_count:PARAMETER_COUNT,parameter_digest_before:hex::encode(before),parameter_digest_after:hex::encode(after),parameter_updates:0,gradient_digest:gradient_hash.finalize().to_hex().to_string(),gradient_l2:libm::sqrt(sum_squares),gradient_nonzero:nonzero,gradient_all_finite:true,initialization_mean_conditional_ce:mean,max_dense_training_bytes:max_dense,gate_tape_bytes:0,model_bytes:model.encode().len(),max_session_snapshot_bytes:max_snapshot,runtime_session_size_bytes:std::mem::size_of::<RuntimeSession>(),elapsed_us:start.elapsed().as_micros(),language_fit_calls:0,general_language_quality:"NOT_QUALIFIED",memory_note:"Dense parameters, three LOO accumulators and two path scratch arrays counted by capacity; no event tape. This is not peak process RSS; temporary categorical/interpreter/serialization/report allocations are separate."})
}

#[cfg(test)]
#[path = "forward_tests.rs"]
mod tests;
