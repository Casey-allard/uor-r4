//! Offline fixed-recipe pilot and matched deterministic evaluation. No retained dispatch.
use super::artifact::BoundGeometry;
use super::circuit::{PrimitiveExport, INPUT_BITS};
use super::engine::{EngineError, Head, Policy, RuntimeSession};
use super::inputs::Phase;
use super::objects::Symbol;
use super::pilot_data::Example;
use super::policy::{head_range, CompiledPolicy, Parameters};
use serde::{Deserialize, Serialize};
type Result<T> = std::result::Result<T, EngineError>;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Control {
    Full,
    ReadDisabled,
    ExactPayloadMasked,
    StateTransportDisabled,
}
pub const CONTROLS: [Control; 4] = [
    Control::Full,
    Control::ReadDisabled,
    Control::ExactPayloadMasked,
    Control::StateTransportDisabled,
];
pub struct Controlled<'a> {
    pub base: CompiledPolicy<'a>,
    pub control: Control,
    pub identity: u16,
    query: [usize; 2],
    pub emission_context: Option<u16>,
}
impl<'a> Controlled<'a> {
    pub fn new(compiled: &'a PrimitiveExport, control: Control, identity: u16) -> Self {
        Self {
            base: CompiledPolicy { compiled },
            control,
            identity,
            query: [0; 2],
            emission_context: None,
        }
    }
}
impl Policy for Controlled<'_> {
    fn context(&mut self, phase: Phase, input: &[bool; INPUT_BITS]) -> Result<u16> {
        let mut packed = *input;
        if self.control == Control::ExactPayloadMasked {
            packed[776..800].fill(false);
            packed[804] = false;
            packed[805] = false;
        }
        let c = self.base.context(phase, &packed)?;
        if phase == Phase::Emit {
            self.emission_context = Some(c);
        }
        Ok(c)
    }
    fn choice(&mut self, head: Head, context: u16, legal: &[bool]) -> Result<usize> {
        let chosen = self.base.choice(head, context, legal)?;
        match head {
            Head::Root(h) if h < 2 => {
                self.query[usize::from(h)] = chosen;
                Ok(chosen)
            }
            Head::Null(h) if self.control == Control::ReadDisabled => {
                Ok(self.query[usize::from(h)])
            }
            Head::Root(4..=7) if self.control == Control::StateTransportDisabled => {
                Ok(usize::from(self.identity))
            }
            _ => Ok(chosen),
        }
    }
}
fn sym(n: u16) -> Result<Symbol> {
    match n {
        0..=255 => Ok(Symbol::Byte(n as u8)),
        256 => Ok(Symbol::Eos),
        _ => Err(EngineError::Invalid("pilot symbol")),
    }
}
fn code(s: Symbol) -> u16 {
    match s {
        Symbol::Byte(b) => u16::from(b),
        Symbol::Eos => 256,
    }
}
pub fn document(example: &Example) -> Vec<u16> {
    example
        .prompt
        .iter()
        .chain(&example.answer)
        .map(|&b| u16::from(b))
        .chain([256])
        .collect()
}
#[derive(Debug, Serialize)]
pub struct EvaluationRow {
    pub id: String,
    pub family: String,
    pub control: Control,
    pub prompt: Vec<u8>,
    pub expected: Vec<u8>,
    pub expected_scalar: Option<i64>,
    pub predictions: Vec<u16>,
    pub generated: Vec<u16>,
    pub response_ce: f64,
    pub correct_symbols: usize,
    pub positions: usize,
    pub exact_response_and_eos: bool,
    pub generated_eos: bool,
    pub candidate_scores: u64,
    pub circuit_calls: u64,
    pub selected_reads: u64,
    pub trace_digest: String,
    pub first_generated_state_digest: String,
}
fn prefill(
    runtime: &mut RuntimeSession,
    g: &BoundGeometry,
    p: &mut Controlled<'_>,
    prompt: &[u8],
) -> Result<()> {
    for &b in prompt {
        let offer = runtime.predict(g, p)?;
        runtime.observe(Symbol::Byte(b), offer.offer.id, g, p)?;
    }
    Ok(())
}
pub fn evaluate(
    parameters: &Parameters,
    compiled: &PrimitiveExport,
    g: &BoundGeometry,
    examples: &[Example],
    control: Control,
) -> Result<Vec<EvaluationRow>> {
    let mut rows = Vec::with_capacity(examples.len());
    for (i, e) in examples.iter().enumerate() {
        let mut runtime = RuntimeSession::new(parameters.digest(), g, i as u64 + 1)?;
        let mut p = Controlled::new(compiled, control, g.identity());
        prefill(&mut runtime, g, &mut p, &e.prompt)?;
        let mut ce = 0.0;
        let mut predictions = Vec::new();
        let mut correct = 0;
        let mut selected_reads = 0;
        let mut hash = blake3::Hasher::new();
        for target in e.answer.iter().map(|&b| u16::from(b)).chain([256]) {
            let offer = runtime.predict(g, &mut p)?;
            let got = code(offer.offer.symbol);
            predictions.push(got);
            correct += usize::from(got == target);
            selected_reads += offer.trace.selected.iter().flatten().count() as u64;
            let context = p
                .emission_context
                .ok_or(EngineError::Invalid("pilot emission context"))?;
            let (offset, n) = head_range(Head::Emission, context)?;
            ce += super::training::cross_entropy(
                &parameters.values()[offset..offset + n],
                &[true; 257],
                usize::from(target),
                1,
            )
            .map_err(|_| EngineError::Invalid("pilot response CE"))?
            .loss;
            let t = runtime.observe(sym(target)?, offer.offer.id, g, &mut p)?;
            hash.update(format!("{t:?}").as_bytes());
        }
        let work = runtime.work();
        let positions = predictions.len();
        let mut generation = RuntimeSession::new(parameters.digest(), g, i as u64 + 1)?;
        let mut p = Controlled::new(compiled, control, g.identity());
        prefill(&mut generation, g, &mut p, &e.prompt)?;
        let first_generated_state_digest =
            blake3::hash(&generation.snapshot()?).to_hex().to_string();
        let mut generated = Vec::new();
        for _ in 0..64 {
            let offered = generation.predict(g, &mut p)?;
            let s = offered.offer.symbol;
            generated.push(code(s));
            generation.observe(s, offered.offer.id, g, &mut p)?;
            if s == Symbol::Eos {
                break;
            }
        }
        let expected: Vec<_> = e
            .answer
            .iter()
            .map(|&b| u16::from(b))
            .chain([256])
            .collect();
        rows.push(EvaluationRow {
            id: e.id.clone(),
            family: e.family.clone(),
            control,
            prompt: e.prompt.clone(),
            expected: e.answer.clone(),
            expected_scalar: e.expected_scalar,
            predictions,
            exact_response_and_eos: generated == expected,
            generated_eos: generated.last() == Some(&256),
            generated,
            response_ce: ce / positions as f64,
            correct_symbols: correct,
            positions,
            candidate_scores: work.candidate_scores + generation.work().candidate_scores,
            circuit_calls: work.circuit_calls + generation.work().circuit_calls,
            selected_reads,
            trace_digest: hash.finalize().to_hex().to_string(),
            first_generated_state_digest,
        });
    }
    Ok(rows)
}
#[derive(Debug, Serialize)]
pub struct Metrics {
    pub examples: usize,
    pub positions: usize,
    pub correct_symbols: usize,
    pub exact_responses: usize,
    pub mean_response_ce: f64,
    pub generated_eos: usize,
}
pub fn metrics(rows: &[EvaluationRow]) -> Metrics {
    let positions = rows.iter().map(|r| r.positions).sum::<usize>();
    Metrics {
        examples: rows.len(),
        positions,
        correct_symbols: rows.iter().map(|r| r.correct_symbols).sum(),
        exact_responses: rows.iter().filter(|r| r.exact_response_and_eos).count(),
        mean_response_ce: rows
            .iter()
            .map(|r| r.response_ce * r.positions as f64)
            .sum::<f64>()
            / positions.max(1) as f64,
        generated_eos: rows.iter().filter(|r| r.generated_eos).count(),
    }
}
#[derive(Debug, Serialize)]
pub struct Decision {
    pub status: &'static str,
    pub predictive_improvement: bool,
    pub no_lost_initial_correct_symbols: bool,
    pub lost_initial_correct_symbols: usize,
    pub both_families_exact: bool,
    pub changed_source_pair_exact: bool,
    pub controls_weaker: Vec<bool>,
    pub promotion: bool,
}
pub fn decision(
    initial: &[EvaluationRow],
    final_full: &[EvaluationRow],
    controls: &[Vec<EvaluationRow>],
) -> Result<Decision> {
    if initial.len() != final_full.len() || initial.is_empty() || controls.len() != 3 {
        return Err(EngineError::Invalid("pilot decision shape"));
    }
    let a = metrics(initial);
    let b = metrics(final_full);
    let mut lost = 0;
    for (old, new) in initial.iter().zip(final_full) {
        if old.id != new.id
            || old.expected != new.expected
            || old.predictions.len() != new.predictions.len()
        {
            return Err(EngineError::Invalid("pilot row identity"));
        }
        let targets = old.expected.iter().map(|&x| u16::from(x)).chain([256]);
        for ((&o, &n), t) in old.predictions.iter().zip(&new.predictions).zip(targets) {
            if o == t && n != t {
                lost += 1;
            }
        }
    }
    let both_families_exact = final_full
        .iter()
        .any(|r| r.expected_scalar.is_none() && r.exact_response_and_eos)
        && final_full
            .iter()
            .any(|r| r.expected_scalar.is_some() && r.exact_response_and_eos);
    let changed_source_pair_exact = [0usize, 4, 6].into_iter().any(|i| {
        final_full.get(i..i + 2).is_some_and(|p| {
            p[0].family == p[1].family
                && p[0].expected != p[1].expected
                && p.iter().all(|r| r.exact_response_and_eos)
        })
    });
    let mut weaker = Vec::new();
    for arm in controls {
        if arm.len() != final_full.len()
            || arm
                .iter()
                .zip(final_full)
                .any(|(c, f)| c.id != f.id || c.expected != f.expected)
        {
            return Err(EngineError::Invalid("pilot control pairing"));
        }
        let c = metrics(arm);
        weaker.push(
            c.mean_response_ce > b.mean_response_ce
                && arm
                    .iter()
                    .zip(final_full)
                    .any(|(c, f)| f.exact_response_and_eos && !c.exact_response_and_eos),
        );
    }
    let predictive =
        b.mean_response_ce < a.mean_response_ce && b.correct_symbols > a.correct_symbols;
    let pass = predictive
        && lost == 0
        && both_families_exact
        && changed_source_pair_exact
        && weaker.iter().all(|&v| v);
    Ok(Decision {
        status: if pass {
            "PASS_ADDRESSED_ATTENTION_LEARNING_PILOT"
        } else {
            "FAIL_ADDRESSED_ATTENTION_LEARNING_PILOT"
        },
        predictive_improvement: predictive,
        no_lost_initial_correct_symbols: lost == 0,
        lost_initial_correct_symbols: lost,
        both_families_exact,
        changed_source_pair_exact,
        controls_weaker: weaker,
        promotion: false,
    })
}
#[cfg(test)]
#[path = "pilot_tests.rs"]
mod tests;
