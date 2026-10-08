//! Opt-in straight-through / relaxed estimator over the fixed gate LUT cascade.
//!
//! Nothing in the default runtime calls into this module. `CompiledPolicy`,
//! `InterpretedPolicy` and `SampledPolicy` are untouched, so the hard LUT cascade
//! remains what runs by default and at evaluation; this module exists only for an
//! explicitly selected offline training entry point.
//!
//! The defect it addresses: each gate's truth table is a vector of 16 logits and
//! the gate *selects* one entry by address, so the emitted context is a discrete
//! function of the parameters. A score-function (REINFORCE) estimator reaches only
//! the visited entry and moves almost none of the 345 tables; measured on
//! `Parameters::seeded(7341)` it changed 331 of 49,030 compiled export bytes and
//! left the arithmetic emission copy rate at 0/80.
//!
//! Estimator, simplest form that gives the missing signal:
//!
//! * forward is the HARD cascade (`Topology::evaluate_with` + `logit > 0.0`), i.e.
//!   exactly `CompiledCircuit::evaluate` semantics, so the trained parameters are
//!   evaluated under the same function the engine runs;
//! * backward relaxes each gate's entry selection to `sigmoid(logit / temperature)`
//!   (the same Bernoulli family the crate's `training::bernoulli` already uses, so
//!   `temperature = 1` is the existing sampling probability and `temperature -> 0`
//!   is the hard table) and relaxes each address to the product distribution
//!   induced by the relaxed previous-layer bits;
//! * the 9-bit context therefore becomes a product distribution `q` over the 512
//!   emission rows, and the relaxed objective is the marginal emission probability
//!   of the target under `q`. Because rows are independent parameters *indexed* by
//!   the discrete context, no derivative of a row with respect to the context
//!   exists; the context distribution is the only relaxation that can produce a
//!   routing gradient at all.
//!
//! The emission rows themselves are trained on the HARD context with the crate's
//! own `training::cross_entropy`, so the parameters that actually serve are fitted
//! on the objective that actually runs. The temperature is annealed to a small
//! value so `q` concentrates on the hard context and the two halves agree.
use super::circuit::{Topology, GATES, GATE_LOGITS, INPUT_BITS, WIDTHS};
use super::engine::{EngineError, Head};
use super::policy::{head_range, Parameters, PARAMETER_COUNT, WIRING_SEED};
use super::training::bernoulli_probability;
use super::training::cross_entropy;

type Result<T> = std::result::Result<T, EngineError>;

/// Context width: the last cascade layer is the 9-bit emission context.
pub const CONTEXT_BITS: usize = 9;
/// Number of addressable emission rows.
pub const CONTEXTS: usize = 512;
/// Emission rows are all-bytes-plus-EOS; the engine passes `[true; 257]`
/// (`engine.rs` `Head::Emission`), so the offline estimator uses the same mask.
pub const EMISSION_WIDTH: usize = 257;
/// Layer widths and their offsets in the flattened gate index space.
const LAYER_START: [usize; 4] = [0, 256, 320, 336];
const LEGAL_ALL: [bool; EMISSION_WIDTH] = [true; EMISSION_WIDTH];
/// Reference gate-gradient RMS of the preserved four-particle score-function fit
/// (`aa-arith-emit-fit-20261005-210113`: gate L2 2.004 over `GATE_LOGITS` 5520).
/// Used to normalise the relaxed gate block to the same update scale so any
/// difference in outcome is attributable to the direction, not to the step size.
pub const SCORE_FUNCTION_GATE_RMS: f64 = 2.004 / 74.296_702_484_026_84;

fn invalid(message: &'static str) -> EngineError {
    EngineError::Invalid(message)
}

fn sigmoid(scaled: f64) -> Result<f64> {
    bernoulli_probability(scaled).map_err(|_| invalid("ste non-finite logit"))
}

/// The parameters' own wiring, checked against the one `Parameters` constructs.
/// `policy.rs` hashes its own source, so this module deliberately does not add an
/// accessor there; it re-derives the seeded wiring and refuses to run if the
/// parameter set carries anything else.
pub fn topology_for(parameters: &Parameters) -> Result<Topology> {
    let expected = Topology::seeded(WIRING_SEED);
    let actual = parameters.compile()?.circuit.topology().clone();
    if actual != expected {
        return Err(invalid("ste parameter topology"));
    }
    Ok(actual)
}

/// Hard 9-bit context under the production cascade semantics: bit `i` of the
/// context is the output of last-layer gate `i`, and an entry is 1 iff its logit
/// is strictly positive. Identical to `CompiledCircuit::evaluate`.
pub fn hard_context(
    topology: &Topology,
    parameters: &Parameters,
    input: &[bool; INPUT_BITS],
) -> Result<u16> {
    let values = parameters.values();
    Ok(topology.evaluate_with(input, |row| Ok(values[row] > 0.0))?)
}

/// One relaxed layer: cached address weights and entry probabilities.
#[derive(Clone, Debug)]
pub struct SoftLayer {
    /// Relaxed bits of the previous layer (hard 0/1 for the input layer).
    pub inputs: Vec<f64>,
    /// `sigmoid(logit / temperature)` per entry, `width * 16`.
    pub entry: Vec<f64>,
    /// Product address weights per entry, `width * 16`.
    pub weights: Vec<f64>,
    /// Relaxed gate outputs, `width`.
    pub outputs: Vec<f64>,
}

/// Cached relaxation of the whole cascade for one input and temperature.
#[derive(Clone, Debug)]
pub struct SoftForward {
    pub layers: Vec<SoftLayer>,
    /// Relaxed 9-bit context, in `0..1`; each entry is one last-layer gate.
    pub bits: [f64; CONTEXT_BITS],
}

/// Relaxed forward. Layer 0 sees the raw hard input bits, so its address is still
/// one-hot; deeper addresses are the product distribution of relaxed bits.
pub fn soft_forward(
    topology: &Topology,
    parameters: &Parameters,
    input: &[bool; INPUT_BITS],
    temperature: f64,
) -> Result<SoftForward> {
    if !temperature.is_finite() || temperature <= 0.0 {
        return Err(invalid("ste temperature"));
    }
    let wires = topology.wires();
    let values = parameters.values();
    let mut previous: Vec<f64> = input.iter().map(|&b| f64::from(u8::from(b))).collect();
    let mut layers = Vec::with_capacity(WIDTHS.len());
    for (layer, &width) in WIDTHS.iter().enumerate() {
        let start = LAYER_START[layer];
        let mut entry = vec![0.0; width * 16];
        let mut weights = vec![0.0; width * 16];
        let mut outputs = vec![0.0; width];
        for local in 0..width {
            let gate = start + local;
            let pins = wires[gate];
            let base = local * 16;
            for address in 0..16usize {
                let mut weight = 1.0;
                for (pin, &source) in pins.iter().enumerate() {
                    let p = previous[usize::from(source)];
                    weight *= if (address >> pin) & 1 != 0 {
                        p
                    } else {
                        1.0 - p
                    };
                }
                let e = sigmoid(values[gate * 16 + address] / temperature)?;
                weights[base + address] = weight;
                entry[base + address] = e;
                outputs[local] += weight * e;
            }
            // A convex combination of probabilities is in `0..1`; the clamp only
            // removes floating-point rounding above one or below zero, which the
            // mixture objective would otherwise reject as an out-of-domain bit.
            outputs[local] = outputs[local].clamp(0.0, 1.0);
        }
        layers.push(SoftLayer {
            inputs: previous,
            entry,
            weights,
            outputs: outputs.clone(),
        });
        previous = outputs;
    }
    let bits: [f64; CONTEXT_BITS] = previous
        .as_slice()
        .try_into()
        .map_err(|_| invalid("ste context width"))?;
    Ok(SoftForward { layers, bits })
}

/// Backward through the relaxed cascade. Returns `d(loss) / d(gate logit)` for all
/// `GATE_LOGITS` entries, in the same flattened layout as `Parameters::values`.
/// The input-layer address derivative is discarded: input bits are not parameters.
pub fn soft_backward(
    topology: &Topology,
    parameters: &Parameters,
    forward: &SoftForward,
    d_bits: &[f64; CONTEXT_BITS],
    temperature: f64,
) -> Result<Vec<f64>> {
    if !temperature.is_finite() || temperature <= 0.0 {
        return Err(invalid("ste temperature"));
    }
    let wires = topology.wires();
    let mut gradient = vec![0.0; GATE_LOGITS];
    let mut d_out: Vec<f64> = d_bits.to_vec();
    for (layer, &width) in WIDTHS.iter().enumerate().rev() {
        let start = LAYER_START[layer];
        let soft = &forward.layers[layer];
        if d_out.len() != width {
            return Err(invalid("ste layer gradient width"));
        }
        let mut d_previous = vec![0.0; soft.inputs.len()];
        for local in 0..width {
            let d = d_out[local];
            if d == 0.0 {
                continue;
            }
            let gate = start + local;
            let pins = wires[gate];
            let base = local * 16;
            for address in 0..16usize {
                let weight = soft.weights[base + address];
                let e = soft.entry[base + address];
                gradient[gate * 16 + address] += d * weight * e * (1.0 - e) / temperature;
                let d_weight = d * e;
                if d_weight == 0.0 {
                    continue;
                }
                for pin in 0..4usize {
                    let mut leave_one_out = 1.0;
                    for other in 0..4usize {
                        if other == pin {
                            continue;
                        }
                        let p = soft.inputs[usize::from(pins[other])];
                        leave_one_out *= if (address >> other) & 1 != 0 {
                            p
                        } else {
                            1.0 - p
                        };
                    }
                    let sign = if (address >> pin) & 1 != 0 { 1.0 } else { -1.0 };
                    d_previous[usize::from(pins[pin])] += d_weight * sign * leave_one_out;
                }
            }
        }
        d_out = d_previous;
    }
    if gradient.iter().any(|v| !v.is_finite()) {
        return Err(invalid("ste non-finite gate gradient"));
    }
    let _ = parameters;
    Ok(gradient)
}

/// Probability that the all-legal emission row at `offset` ranks `target`, with the
/// same softmax support as `training::categorical_weights`.
fn row_target_probability(values: &[f64], offset: usize, target: usize) -> Result<f64> {
    let row = values
        .get(offset..offset + EMISSION_WIDTH)
        .ok_or(invalid("ste emission row range"))?;
    let maximum = row.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if !maximum.is_finite() {
        return Err(invalid("ste non-finite emission row"));
    }
    let mut sum = 0.0;
    for &value in row {
        sum += libm::exp(value - maximum);
    }
    if !sum.is_finite() || sum <= 0.0 {
        return Err(invalid("ste empty emission support"));
    }
    Ok(libm::exp(row[target] - maximum) / sum)
}

/// Relaxed routing objective over the context distribution `q` induced by the
/// relaxed 9-bit context: `loss = -ln sum_c q(c) * p_c(target)`.
#[derive(Clone, Debug)]
pub struct Mixture {
    pub loss: f64,
    /// Marginal emission probability of the target under `q`; `exp(-loss)`.
    pub marginal: f64,
    /// `d(loss) / d(relaxed context bit)`.
    pub d_bits: [f64; CONTEXT_BITS],
    /// Highest-mass single context under `q` and its probability.
    pub dominant: u16,
    pub dominant_mass: f64,
}

/// Marginal mixture objective and its derivative with respect to the relaxed
/// context bits. Every context's row contributes, so the gradient identifies which
/// contexts would emit this target; the cascade backward then turns that into gate
/// logit updates.
pub fn soft_mixture(
    parameters: &Parameters,
    bits: &[f64; CONTEXT_BITS],
    target: usize,
) -> Result<Mixture> {
    if target >= EMISSION_WIDTH {
        return Err(invalid("ste target"));
    }
    if bits.iter().any(|b| !b.is_finite() || *b < 0.0 || *b > 1.0) {
        return Err(invalid("ste relaxed context domain"));
    }
    let values = parameters.values();
    // Bernoulli product q over the 512 rows, plus the marginal and its bit
    // derivative. The leave-one-out factor is built from prefix/suffix products so
    // no division by a possibly saturated probability is needed.
    let mut q = vec![0.0f64; CONTEXTS];
    let mut p = vec![0.0f64; CONTEXTS];
    // Per-context leave-one-out products, so the derivative below never divides by
    // a possibly saturated bit probability.
    let mut leave_one_out = vec![[0.0f64; CONTEXT_BITS]; CONTEXTS];
    let mut prefix = [1.0f64; CONTEXT_BITS + 1];
    let mut suffix = [1.0f64; CONTEXT_BITS + 1];
    let mut marginal = 0.0;
    let mut dominant = 0u16;
    let mut dominant_mass = f64::NEG_INFINITY;
    for context in 0..CONTEXTS {
        for i in 0..CONTEXT_BITS {
            let one = bits[i];
            let taken = (context >> i) & 1 != 0;
            prefix[i + 1] = prefix[i] * if taken { one } else { 1.0 - one };
        }
        for i in (0..CONTEXT_BITS).rev() {
            let one = bits[i];
            let taken = (context >> i) & 1 != 0;
            suffix[i] = suffix[i + 1] * if taken { one } else { 1.0 - one };
        }
        for i in 0..CONTEXT_BITS {
            leave_one_out[context][i] = prefix[i] * suffix[i + 1];
        }
        let weight = prefix[CONTEXT_BITS];
        let (offset, _) = head_range(Head::Emission, context as u16)?;
        let probability = row_target_probability(values, offset, target)?;
        q[context] = weight;
        p[context] = probability;
        marginal += weight * probability;
        if weight > dominant_mass {
            dominant_mass = weight;
            dominant = context as u16;
        }
    }
    if !marginal.is_finite() || marginal <= 0.0 {
        return Err(invalid("ste degenerate context distribution"));
    }
    let loss = -libm::log(marginal);
    let mut d_bits = [0.0f64; CONTEXT_BITS];
    for context in 0..CONTEXTS {
        let d_q = -p[context] / marginal;
        if d_q == 0.0 || q[context] == 0.0 {
            continue;
        }
        for i in 0..CONTEXT_BITS {
            // d q(c) / d s_i = (2 c_i - 1) * prod_{j != i} p_j(c_j)
            let sign = if (context >> i) & 1 != 0 { 1.0 } else { -1.0 };
            d_bits[i] += d_q * sign * leave_one_out[context][i];
        }
    }
    if d_bits.iter().any(|v| !v.is_finite()) {
        return Err(invalid("ste non-finite context gradient"));
    }
    Ok(Mixture {
        loss,
        marginal,
        d_bits,
        dominant,
        dominant_mass,
    })
}

/// Hard-context emission loss and its gradient, using the crate's own conditional
/// cross-entropy with normalizer 1 (one position's objective).
pub fn hard_emission(
    parameters: &Parameters,
    context: u16,
    target: usize,
) -> Result<(f64, Vec<f64>)> {
    if target >= EMISSION_WIDTH {
        return Err(invalid("ste target"));
    }
    let (offset, width) = head_range(Head::Emission, context)?;
    let loss = cross_entropy(
        &parameters.values()[offset..offset + width],
        &LEGAL_ALL,
        target,
        1,
    )
    .map_err(|_| invalid("ste hard emission cross-entropy"))?;
    Ok((loss.loss, loss.direct))
}

/// Row argmax of the compiled emission head at `context`, i.e. the symbol the
/// engine would emit there. Used by instruments to read the trained artifact.
pub fn emission_argmax(parameters: &Parameters, context: u16) -> Result<u16> {
    let (offset, width) = head_range(Head::Emission, context)?;
    let row = &parameters.values()[offset..offset + width];
    let mut best = 0usize;
    for (i, &value) in row.iter().enumerate() {
        if value > row[best] {
            best = i;
        }
    }
    Ok(best as u16)
}

/// One captured emission consult: the packed 1024-bit input handed to
/// `Policy::context` for `Phase::Emit`, and the byte the engine must emit there.
#[derive(Clone, Copy, Debug)]
pub struct StePair {
    pub input: [bool; INPUT_BITS],
    pub target: u16,
}

/// Bounded, single-recipe relaxation schedule.
#[derive(Clone, Copy, Debug)]
pub struct SteConfig {
    pub updates: u64,
    pub rate: f64,
    pub temperature_start: f64,
    pub temperature_end: f64,
    /// Normalise the relaxed gate block to this RMS before `learner::sgd`.
    pub gate_rms: f64,
}

impl SteConfig {
    pub fn validate(&self) -> Result<()> {
        if self.updates == 0
            || !self.rate.is_finite()
            || self.rate <= 0.0
            || !self.temperature_start.is_finite()
            || !self.temperature_end.is_finite()
            || self.temperature_start <= 0.0
            || self.temperature_end <= 0.0
            || self.temperature_end > self.temperature_start
            || !self.gate_rms.is_finite()
            || self.gate_rms <= 0.0
        {
            return Err(invalid("ste configuration"));
        }
        Ok(())
    }

    /// Geometric anneal: the relaxation converges to the hard table.
    pub fn temperature(&self, update: u64) -> f64 {
        let fraction = update as f64 / self.updates as f64;
        self.temperature_start * libm::pow(self.temperature_end / self.temperature_start, fraction)
    }
}

/// Per-update record. Diagnostic only; the caller decides what to retain.
#[derive(Clone, Debug)]
pub struct SteUpdate {
    pub update: u64,
    pub pair: usize,
    pub temperature: f64,
    pub hard_context: u16,
    pub soft_loss: f64,
    pub marginal: f64,
    pub dominant: u16,
    pub hard_ce: f64,
    pub gate_gradient_rms: f64,
    pub gate_gradient_raw_rms: f64,
    pub head_gradient_l2: f64,
    pub gradient_nonzero: usize,
    pub parameters_digest: [u8; 32],
}

/// Result of a bounded relaxed fit.
#[derive(Clone, Debug)]
pub struct SteFit {
    pub parameters: Parameters,
    pub updates_completed: u64,
    /// True when the observer asked to stop (for example at a wall-clock cap) or
    /// when the relaxed gradient vanished.
    pub stopped_early: bool,
    pub stop_reason: Option<&'static str>,
    pub trace: Vec<SteUpdate>,
}

/// Bounded fit. Each update trains the gate logits through the relaxed routing
/// objective and the emission row at the hard context through the crate's own
/// cross-entropy, then applies the crate's own `learner::sgd`. Every other
/// parameter block is held frozen, so the fitted artifact differs from the frozen
/// parent only where the discrete routing is learned.
///
/// `observer` sees the updated parameters and the update record after each update
/// and returns `false` to stop.
pub fn fit(
    parameters: &Parameters,
    pairs: &[StePair],
    config: &SteConfig,
    observer: &mut dyn FnMut(&Parameters, &SteUpdate) -> bool,
) -> Result<SteFit> {
    config.validate()?;
    if pairs.is_empty() {
        return Err(invalid("ste pairs"));
    }
    if pairs
        .iter()
        .any(|p| usize::from(p.target) >= EMISSION_WIDTH)
    {
        return Err(invalid("ste pair target"));
    }
    let topology = topology_for(parameters)?;
    let mut current = parameters.clone();
    let mut trace = Vec::with_capacity(config.updates.min(4096) as usize);
    let mut stopped_early = false;
    let mut stop_reason: Option<&'static str> = None;
    let mut completed = 0u64;
    for update in 0..config.updates {
        let pair = &pairs[update as usize % pairs.len()];
        let target = usize::from(pair.target);
        let temperature = config.temperature(update);
        let forward = soft_forward(&topology, &current, &pair.input, temperature)?;
        let mixture = soft_mixture(&current, &forward.bits, target)?;
        let raw_gate = soft_backward(&topology, &current, &forward, &mixture.d_bits, temperature)?;
        let hard = hard_context(&topology, &current, &pair.input)?;
        let (hard_ce, head_direct) = hard_emission(&current, hard, target)?;
        let mean_square = raw_gate.iter().map(|g| g * g).sum::<f64>() / raw_gate.len() as f64;
        let raw_rms = libm::sqrt(mean_square);
        if !raw_rms.is_finite() || raw_rms <= 0.0 {
            // The relaxation has saturated everywhere; stop with the parameters so
            // far instead of aborting the whole attempt.
            stopped_early = true;
            stop_reason = Some("vanishing relaxed gate gradient");
            break;
        }
        let scale = config.gate_rms / raw_rms;
        let mut gradient = vec![0.0f64; PARAMETER_COUNT];
        for (slot, &value) in gradient[..GATE_LOGITS].iter_mut().zip(raw_gate.iter()) {
            *slot = value * scale;
        }
        let (offset, _) = head_range(Head::Emission, hard)?;
        let mut head_l2 = 0.0;
        for (i, &value) in head_direct.iter().enumerate() {
            gradient[offset + i] = value;
            head_l2 += value * value;
        }
        let gate_rms = libm::sqrt(
            gradient[..GATE_LOGITS].iter().map(|g| g * g).sum::<f64>() / GATE_LOGITS as f64,
        );
        let next = super::learner::sgd(&current, &gradient, config.rate)?;
        let record = SteUpdate {
            update: update + 1,
            pair: update as usize % pairs.len(),
            temperature,
            hard_context: hard,
            soft_loss: mixture.loss,
            marginal: mixture.marginal,
            dominant: mixture.dominant,
            hard_ce,
            gate_gradient_rms: gate_rms,
            gate_gradient_raw_rms: raw_rms,
            head_gradient_l2: libm::sqrt(head_l2),
            gradient_nonzero: gradient.iter().filter(|g| **g != 0.0).count(),
            parameters_digest: next.digest(),
        };
        current = next;
        completed = update + 1;
        let keep_going = observer(&current, &record);
        trace.push(record);
        if !keep_going {
            stopped_early = true;
            stop_reason = Some("observer stop");
            break;
        }
    }
    Ok(SteFit {
        parameters: current,
        updates_completed: completed,
        stopped_early,
        stop_reason,
        trace,
    })
}

/// Number of trainable gate logits, exposed so a caller can report the surface.
pub const TRAINABLE_GATE_LOGITS: usize = GATE_LOGITS;
/// Total gate count, for reporting.
pub const TRAINABLE_GATES: usize = GATES;

#[cfg(test)]
#[path = "ste_tests.rs"]
mod tests;
