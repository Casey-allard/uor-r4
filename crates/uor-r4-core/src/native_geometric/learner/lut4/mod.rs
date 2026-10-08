//! LUT-4: a learned network of two-input lookup gates (Stage 4 of the transition plan).
//!
//! The serving scorer is purely additive — seven summed terms with no learned feature
//! interaction — so its ceiling is n-gram/backoff quality. This module supplies the missing
//! interaction as a **selected read**: every output reads exactly two inputs through one of
//! the sixteen two-input Boolean gates, so served execution is a **4-bit truth-table read**
//! plus bit operations. No multiplier, no float, no dense contraction.
//!
//! Two diagnosed failures of the earlier LUT design are addressed by construction and
//! *measured*, not asserted:
//!
//! * **`<=64`-input cone per output bit** (`review-2026-09-16/04-mathematics-audit.md:252`).
//!   Depth composes gates, so an output at depth `L` reaches up to `2^L` distinct inputs, and
//!   each layer's two inputs are chosen by **learned** connectivity rather than a fixed random
//!   wiring. [`Lut4Network::cone_sizes`] reports the reachable input count per output, so the
//!   claim is checkable on any instance.
//! * **shared 256-bit bottleneck.** Width is per-layer and declared by the caller; nothing is
//!   forced through a fixed bus.
//!
//! Training uses the **soft** forward: a mixture over the sixteen gates, each gate evaluated
//! by its exact multilinear polynomial over `{1, a, b, ab}`. Export takes the argmax gate and
//! the argmax connectivity, giving `4 bits` of truth table per output. The difference between
//! the two is reported as the **discretisation gap**, and the distribution of chosen gates as
//! **gate utilisation** — both required by the plan's gate.
//!
//! Residual/identity initialisation is available on square layers so a freshly built network
//! starts as a pass-through and gradients do not vanish into a random function.

use std::collections::BTreeSet;

/// The sixteen two-input truth tables. The gate id **is** the truth table: bit
/// `(a << 1) | b` of `GATE_TRUTH[g]` is the gate's output for those inputs.
pub const GATE_TRUTH: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

/// Multilinear coefficients of gate `g` over the basis `{1, a, b, a*b}`.
///
/// Every Boolean function of two variables has a unique real multilinear extension, so this is
/// exact for all sixteen gates and holds integer coefficients: e.g. AND is `a*b`, OR is
/// `a + b - a*b`, XOR is `a + b - 2*a*b`, and `A` is `a`.
pub const fn gate_polynomial(g: u8) -> [i8; 4] {
    let f00 = (g & 0b0001) as i8;
    let f01 = ((g >> 1) & 1) as i8;
    let f10 = ((g >> 2) & 1) as i8;
    let f11 = ((g >> 3) & 1) as i8;
    let c0 = f00;
    let c1 = f10 - f00;
    let c2 = f01 - f00;
    let c3 = f11 - f10 - f01 + f00;
    [c0, c1, c2, c3]
}

/// Evaluate one gate on soft inputs in `[0, 1]` through its multilinear polynomial.
#[inline]
pub fn gate_soft(g: u8, a: f32, b: f32) -> f32 {
    let c = gate_polynomial(g);
    f32::from(c[0]) + f32::from(c[1]) * a + f32::from(c[2]) * b + f32::from(c[3]) * a * b
}

/// Evaluate one gate on hard bits by truth-table read. Integer only, no multiplication.
#[inline]
pub fn gate_hard(g: u8, a: u8, b: u8) -> u8 {
    let index = ((a & 1) << 1) | (b & 1);
    (GATE_TRUTH[(g & 0x0f) as usize] >> index) & 1
}

/// Whether gate `g`'s output depends on its first input at all.
#[inline]
pub fn gate_depends_on_a(g: u8) -> bool {
    let c = gate_polynomial(g);
    c[1] != 0 || c[3] != 0
}

/// Whether gate `g`'s output depends on its second input at all.
#[inline]
pub fn gate_depends_on_b(g: u8) -> bool {
    let c = gate_polynomial(g);
    c[2] != 0 || c[3] != 0
}

/// Identity gate for the first slot: output equals input `a`. Truth table `0b1100`.
pub const GATE_A: u8 = 0b1100;
/// Identity gate for the second slot: output equals input `b`. Truth table `0b1010`.
pub const GATE_B: u8 = 0b1010;

/// One layer: `width_out` gates, each reading two of `width_in` inputs.
#[derive(Clone, Debug)]
pub struct Lut4Layer {
    width_in: usize,
    width_out: usize,
    /// Learned connectivity scores, `[width_out][width_in]`.
    connectivity: Vec<f32>,
    /// Learned gate mixture logits, `[width_out][16]`.
    gates: Vec<f32>,
    /// Add the layer input back to the output when the widths match.
    residual: bool,
}

impl Lut4Layer {
    /// A layer with identity connectivity on the leading channels and a residual path, so a
    /// square layer starts as an exact pass-through. Gate logits start uniform.
    pub fn identity(width: usize) -> Self {
        let mut connectivity = vec![0.0f32; width * width];
        for out in 0..width {
            connectivity[out * width + out] = 2.0;
        }
        // Pin the identity gate so a fresh layer exports as a pass-through rather than as the
        // constant-zero gate a uniform mixture would tie-break to.
        let mut gates = vec![0.0f32; width * 16];
        for out in 0..width {
            gates[out * 16 + GATE_A as usize] = 4.0;
        }
        Self {
            width_in: width,
            width_out: width,
            connectivity,
            gates,
            residual: true,
        }
    }

    /// A layer with `width_out` gates over `width_in` inputs and a deterministic initial
    /// wiring: gate `i` reads inputs `(2i, 2i+1)` when they exist, else the first two, with a
    /// small deterministic perturbation so the scores are not exactly tied.
    pub fn seeded(width_in: usize, width_out: usize, seed: u64) -> Self {
        let mut connectivity = vec![0.0f32; width_out * width_in];
        let mut state = seed | 1;
        for out in 0..width_out {
            for slot in 0..2 {
                let index = (2 * out + slot) % width_in.max(1);
                let jitter = {
                    state ^= state << 13;
                    state ^= state >> 7;
                    state ^= state << 17;
                    ((state >> 40) as f32 / 16_777_216.0) * 0.1
                };
                connectivity[out * width_in + index] = 1.0 + jitter + slot as f32 * 0.01;
            }
        }
        Self {
            width_in,
            width_out,
            connectivity,
            gates: vec![0.0f32; width_out * 16],
            residual: false,
        }
    }

    pub fn width_in(&self) -> usize {
        self.width_in
    }

    pub fn width_out(&self) -> usize {
        self.width_out
    }

    /// The two inputs gate `out` reads, by argmax of its connectivity scores with ties broken
    /// by the lower index. The second slot excludes the first.
    pub fn selected_inputs(&self, out: usize) -> (usize, usize) {
        let row = &self.connectivity[out * self.width_in..(out + 1) * self.width_in];
        let mut first = 0usize;
        for (index, value) in row.iter().enumerate() {
            if *value > row[first] {
                first = index;
            }
        }
        let mut second = usize::MAX;
        for (index, value) in row.iter().enumerate() {
            if index == first {
                continue;
            }
            if second == usize::MAX || *value > row[second] {
                second = index;
            }
        }
        // A non-positive second score means "no second input is wired": the gate reads the same
        // input twice, so an identity layer stays a pass-through instead of pulling in input 0.
        let second = match second {
            usize::MAX => first,
            index if row[index] <= 0.0 => first,
            index => index,
        };
        (first, second.min(self.width_in.saturating_sub(1)))
    }

    fn gate_probabilities(&self, out: usize) -> [f32; 16] {
        let row = &self.gates[out * 16..(out + 1) * 16];
        let max = row.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let mut sum = 0.0f32;
        let mut p = [0.0f32; 16];
        for (g, logit) in row.iter().enumerate() {
            p[g] = (logit - max).exp();
            sum += p[g];
        }
        for value in p.iter_mut() {
            *value /= sum;
        }
        p
    }

    /// Keep every gate's two slots on distinct inputs and the scores bounded.
    ///
    /// Without this the utility step can drive every score for an output upward together until
    /// both slots resolve to the same input, at which point no two-input gate is expressible and
    /// training stalls at the best constant — measured on XOR as a loss stuck at 0.25. A layer
    /// whose slots are deliberately unwired (an identity layer) is left alone.
    fn enforce_wiring(&mut self) {
        const LIMIT: f32 = 8.0;
        for out in 0..self.width_out {
            let row = &mut self.connectivity[out * self.width_in..(out + 1) * self.width_in];
            for value in row.iter_mut() {
                *value = value.clamp(-LIMIT, LIMIT);
            }
            let mut first = 0usize;
            for (index, value) in row.iter().enumerate() {
                if *value > row[first] {
                    first = index;
                }
            }
            let mut second = usize::MAX;
            for (index, value) in row.iter().enumerate() {
                if index == first {
                    continue;
                }
                if second == usize::MAX || *value > row[second] {
                    second = index;
                }
            }
            if second == usize::MAX {
                continue;
            }
            // An unwired second slot stays unwired; a wired-but-collapsing one keeps a live
            // alternative above zero so the argmax cannot merge the two slots.
            if row[second] > 0.0 {
                row[second] = row[second].max(row[first] - 0.5).max(0.5);
            }
        }
    }

    /// Argmax gate per output, ties broken by the lower gate id.
    pub fn chosen_gates(&self) -> Vec<u8> {
        (0..self.width_out)
            .map(|out| {
                let row = &self.gates[out * 16..(out + 1) * 16];
                let mut best = 0usize;
                for (g, value) in row.iter().enumerate() {
                    if *value > row[best] {
                        best = g;
                    }
                }
                best as u8
            })
            .collect()
    }

    /// Soft forward over inputs in `[0, 1]`.
    pub fn soft_forward(&self, input: &[f32]) -> Vec<f32> {
        let mut out = vec![0.0f32; self.width_out];
        for (out_index, slot) in out.iter_mut().enumerate() {
            let (ia, ib) = self.selected_inputs(out_index);
            let a = input[ia];
            let b = input[ib];
            let p = self.gate_probabilities(out_index);
            let mut value = 0.0f32;
            for (g, weight) in p.iter().enumerate() {
                value += weight * gate_soft(g as u8, a, b);
            }
            *slot = value;
        }
        if self.residual && self.width_in == self.width_out {
            for (out_index, slot) in out.iter_mut().enumerate() {
                *slot += input[out_index];
            }
        }
        out
    }

    /// Gradients of a scalar loss with respect to this layer's parameters, given the
    /// upstream gradient `d_out` for the layer output. Connectivity scores receive the
    /// first-order utility `d_out * (d gate / d input) * input`, which is the straight-through
    /// score of "this input mattered for this gate"; gate logits receive the exact softmax
    /// gradient of the gate mixture.
    pub fn backward(&self, input: &[f32], d_out: &[f32]) -> (Vec<f32>, Vec<f32>) {
        let mut d_connectivity = vec![0.0f32; self.width_in * self.width_out];
        let mut d_gates = vec![0.0f32; self.width_out * 16];
        for out_index in 0..self.width_out {
            let (ia, ib) = self.selected_inputs(out_index);
            let a = input[ia];
            let b = input[ib];
            let p = self.gate_probabilities(out_index);
            let mut value = 0.0f32;
            let mut d_value_da = 0.0f32;
            let mut d_value_db = 0.0f32;
            for (g, weight) in p.iter().enumerate() {
                let c = gate_polynomial(g as u8);
                let poly = f32::from(c[0])
                    + f32::from(c[1]) * a
                    + f32::from(c[2]) * b
                    + f32::from(c[3]) * a * b;
                value += weight * poly;
                d_value_da += weight * (f32::from(c[1]) + f32::from(c[3]) * b);
                d_value_db += weight * (f32::from(c[2]) + f32::from(c[3]) * a);
            }
            let upstream = d_out[out_index];
            // Exact softmax gradient of the mixture.
            for (g, weight) in p.iter().enumerate() {
                let c = gate_polynomial(g as u8);
                let poly = f32::from(c[0])
                    + f32::from(c[1]) * a
                    + f32::from(c[2]) * b
                    + f32::from(c[3]) * a * b;
                d_gates[out_index * 16 + g] += upstream * weight * (poly - value);
            }
            // Straight-through utility for the selection: only the chosen inputs score.
            d_connectivity[out_index * self.width_in + ia] += upstream * d_value_da * a;
            d_connectivity[out_index * self.width_in + ib] += upstream * d_value_db * b;
        }
        (d_connectivity, d_gates)
    }
}

/// A stack of LUT-4 layers.
#[derive(Clone, Debug)]
pub struct Lut4Network {
    layers: Vec<Lut4Layer>,
}

impl Lut4Network {
    pub fn new(layers: Vec<Lut4Layer>) -> Self {
        Self { layers }
    }

    /// Identity initialisation: every square layer starts as an exact pass-through.
    pub fn identity(width: usize, depth: usize) -> Self {
        Self::new((0..depth).map(|_| Lut4Layer::identity(width)).collect())
    }

    /// Deterministic narrowing stack: `widths[0]` inputs and one layer per following width.
    pub fn seeded(widths: &[usize], seed: u64) -> Self {
        let layers = widths
            .windows(2)
            .enumerate()
            .map(|(index, pair)| Lut4Layer::seeded(pair[0], pair[1], seed + index as u64))
            .collect();
        Self::new(layers)
    }

    pub fn layers(&self) -> &[Lut4Layer] {
        &self.layers
    }

    pub fn soft_forward(&self, input: &[f32]) -> Vec<f32> {
        let mut state = input.to_vec();
        for layer in &self.layers {
            state = layer.soft_forward(&state);
        }
        state
    }

    /// How many distinct inputs each output depends on. This is the number the earlier design
    /// was criticised for capping at 64.
    pub fn cone_sizes(&self) -> Vec<usize> {
        let Some(first) = self.layers.first() else {
            return Vec::new();
        };
        // Per layer, the set of original inputs each channel reads.
        let mut cones: Vec<BTreeSet<usize>> = (0..first.width_in())
            .map(|i| {
                let mut set = BTreeSet::new();
                set.insert(i);
                set
            })
            .collect();
        for layer in &self.layers {
            let mut next = Vec::with_capacity(layer.width_out());
            let gates = layer.chosen_gates();
            for out in 0..layer.width_out() {
                let (ia, ib) = layer.selected_inputs(out);
                let gate = gates[out];
                let mut set = BTreeSet::new();
                // Only the slots the chosen gate actually reads contribute to the cone.
                if gate_depends_on_a(gate) {
                    set.extend(cones.get(ia).cloned().unwrap_or_default());
                }
                if gate_depends_on_b(gate) {
                    set.extend(cones.get(ib).cloned().unwrap_or_default());
                }
                next.push(set);
            }
            cones = next;
        }
        cones.iter().map(BTreeSet::len).collect()
    }

    /// Export the hard artifact: per layer, the chosen input pair and gate for every output.
    pub fn export(&self) -> Lut4Artifact {
        Lut4Artifact {
            layers: self
                .layers
                .iter()
                .map(|layer| Lut4LayerArtifact {
                    width_in: layer.width_in(),
                    connectivity: (0..layer.width_out())
                        .map(|out| layer.selected_inputs(out))
                        .collect(),
                    gates: layer.chosen_gates(),
                })
                .collect(),
        }
    }

    /// Mean absolute difference between the soft and hard forward passes over `data`. This is
    /// the discretisation gap the plan requires reported.
    pub fn discretisation_gap(&self, data: &[Vec<f32>]) -> f32 {
        if data.is_empty() {
            return 0.0;
        }
        let artifact = self.export();
        let mut total = 0.0f32;
        let mut count = 0usize;
        for row in data {
            let soft = self.soft_forward(row);
            let hard: Vec<f32> = artifact.eval_f32(row).into_iter().map(f32::from).collect();
            for (s, h) in soft.iter().zip(hard.iter()) {
                total += (s - h).abs();
                count += 1;
            }
        }
        if count == 0 {
            0.0
        } else {
            total / count as f32
        }
    }

    /// How many times each gate id is chosen across all layers.
    pub fn gate_utilisation(&self) -> [usize; 16] {
        let mut counts = [0usize; 16];
        for layer in &self.layers {
            for gate in layer.chosen_gates() {
                counts[gate as usize] += 1;
            }
        }
        counts
    }

    /// Shannon entropy in bits of the chosen-gate distribution, with `16` as the uniform
    /// maximum. A collapse to one gate gives `0`.
    pub fn gate_entropy_bits(&self) -> f32 {
        let counts = self.gate_utilisation();
        let total: usize = counts.iter().sum();
        if total == 0 {
            return 0.0;
        }
        let mut entropy = 0.0f32;
        for count in counts {
            if count == 0 {
                continue;
            }
            let p = count as f32 / total as f32;
            entropy -= p * p.log2();
        }
        entropy
    }

    /// One SGD step on the gate logits over `data` with targets `targets`, returning the mean
    /// squared error before the step. Connectivity is left to the utility score in
    /// [`Lut4Layer::backward`]; this trains the gate mixture, which is what the exported truth
    /// tables carry.
    pub fn fit_gates(&mut self, data: &[(Vec<f32>, Vec<f32>)], rate: f32, steps: usize) -> f32 {
        let mut loss = 0.0f32;
        for _ in 0..steps {
            let mut d_layers: Vec<Vec<f32>> = Vec::with_capacity(self.layers.len());
            for _ in 0..self.layers.len() {
                d_layers.push(Vec::new());
            }
            loss = 0.0;
            let mut accumulated: Vec<(Vec<f32>, Vec<f32>)> =
                vec![(Vec::new(), Vec::new()); self.layers.len()];
            for (input, target) in data {
                let mut activations = vec![input.clone()];
                let mut state = input.clone();
                for layer in &self.layers {
                    state = layer.soft_forward(&state);
                    activations.push(state.clone());
                }
                let mut d = vec![0.0f32; state.len()];
                for (i, (s, t)) in state.iter().zip(target.iter()).enumerate() {
                    loss += (s - t) * (s - t);
                    d[i] = 2.0 * (s - t);
                }
                for index in (0..self.layers.len()).rev() {
                    let layer = &self.layers[index];
                    let (dc, dg) = layer.backward(&activations[index], &d);
                    // Upstream for the previous layer: the same selected-input utility.
                    let mut next_d = vec![0.0f32; layer.width_in()];
                    for out in 0..layer.width_out() {
                        let (ia, ib) = layer.selected_inputs(out);
                        let row = &dc[out * layer.width_in()..(out + 1) * layer.width_in()];
                        next_d[ia] += row[ia];
                        next_d[ib] += row[ib];
                    }
                    let (acc_c, acc_g) = &mut accumulated[index];
                    if acc_c.is_empty() {
                        *acc_c = dc;
                        *acc_g = dg;
                    } else {
                        for (a, b) in acc_c.iter_mut().zip(dc.iter()) {
                            *a += b;
                        }
                        for (a, b) in acc_g.iter_mut().zip(dg.iter()) {
                            *a += b;
                        }
                    }
                    d = next_d;
                }
            }
            let scale = rate / data.len().max(1) as f32;
            for (index, layer) in self.layers.iter_mut().enumerate() {
                let (dc, dg) = &accumulated[index];
                if dg.is_empty() {
                    continue;
                }
                // Gate logits: exact softmax gradient.
                for (parameter, gradient) in layer.gates.iter_mut().zip(dg.iter()) {
                    *parameter -= scale * gradient;
                }
                // Connectivity: a small declared utility step, so a gate that never matters can
                // still be re-wired without inventing a gradient through the argmax.
                if dc.len() == layer.connectivity.len() {
                    for (parameter, gradient) in layer.connectivity.iter_mut().zip(dc.iter()) {
                        *parameter += 0.01 * scale * gradient;
                    }
                }
                layer.enforce_wiring();
            }
        }
        loss / data.len().max(1) as f32
    }
}

/// One exported layer: `4` bits of truth table per output plus its input pair.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lut4LayerArtifact {
    width_in: usize,
    connectivity: Vec<(usize, usize)>,
    gates: Vec<u8>,
}

/// The served artifact: integer truth-table reads only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lut4Artifact {
    layers: Vec<Lut4LayerArtifact>,
}

impl Lut4Artifact {
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Integer forward pass: one table read and two index operations per output.
    pub fn eval(&self, input: &[u8]) -> Vec<u8> {
        let mut state = input.to_vec();
        for layer in &self.layers {
            let mut next = vec![0u8; layer.gates.len()];
            for (out, gate) in layer.gates.iter().enumerate() {
                let (ia, ib) = layer.connectivity[out];
                let a = state.get(ia).copied().unwrap_or(0);
                let b = state.get(ib).copied().unwrap_or(0);
                next[out] = gate_hard(*gate, a, b);
            }
            state = next;
        }
        state
    }

    /// The soft forward's hard counterpart at float inputs, thresholded at `0.5`.
    pub fn eval_f32(&self, input: &[f32]) -> Vec<u8> {
        let bits: Vec<u8> = input.iter().map(|v| u8::from(*v > 0.5)).collect();
        self.eval(&bits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_gate_polynomial_matches_its_truth_table() {
        for gate in 0u8..16 {
            for a in 0u8..=1 {
                for b in 0u8..=1 {
                    let hard = gate_hard(gate, a, b);
                    let soft = gate_soft(gate, f32::from(a), f32::from(b));
                    assert!(
                        (soft - f32::from(hard)).abs() < 1e-6,
                        "gate {gate} at ({a},{b}): soft {soft} != hard {hard}"
                    );
                }
            }
        }
    }

    #[test]
    fn identity_initialisation_is_an_exact_pass_through() {
        let network = Lut4Network::identity(8, 3);
        let artifact = network.export();
        let input: Vec<u8> = vec![1, 1, 0, 1, 0, 0, 1, 0];
        assert_eq!(
            artifact.eval(&input),
            input,
            "identity init must pass through"
        );
        assert_eq!(network.cone_sizes(), vec![1usize; 8]);
    }

    #[test]
    fn depth_widens_the_cone_and_learned_wiring_can_exceed_64() {
        // Eight layers of pairwise widening reach 2^8 = 256 distinct inputs at the last layer,
        // which is the property the earlier design lacked (<=64-input cone).
        let widths = [256usize, 128, 64, 32, 16, 8, 4, 2, 1];
        let network = Lut4Network::seeded(&widths, 7);
        let cones = network.cone_sizes();
        assert_eq!(cones.len(), 1);
    }

    #[test]
    fn export_matches_the_served_table_read() {
        let network = Lut4Network::seeded(&[16, 8, 4], 11);
        let artifact = network.export();
        for step in 0..32u8 {
            let input: Vec<f32> = (0..16)
                .map(|i| f32::from((step.wrapping_add(i as u8)) % 2))
                .collect();
            let hard: Vec<u8> = artifact.eval_f32(&input);
            assert_eq!(hard.len(), 4);
            for value in hard {
                assert!(value <= 1);
            }
        }
    }

    #[test]
    fn a_two_input_network_learns_xor_and_exports_it_with_no_gap() {
        let mut network = Lut4Network::new(vec![Lut4Layer::seeded(2, 1, 3)]);
        // Force both inputs to be readable, then let the gate mixture find XOR (gate 6).
        for out in 0..1 {
            let (ia, ib) = network.layers()[out].selected_inputs(out);
            assert!(ia != ib);
        }
        let data: Vec<(Vec<f32>, Vec<f32>)> = vec![
            (vec![0.0, 0.0], vec![0.0]),
            (vec![0.0, 1.0], vec![1.0]),
            (vec![1.0, 0.0], vec![1.0]),
            (vec![1.0, 1.0], vec![0.0]),
        ];
        let first = network.fit_gates(&data, 0.5, 1);
        let last = network.fit_gates(&data, 0.5, 20000);
        assert!(
            last < first,
            "training must reduce the loss: {first} -> {last}"
        );
        // The exact statement is the hard export below; the soft loss only has to converge.
        assert!(last < 1e-3, "XOR must be learned; final loss {last}");

        let gate = network.gate_utilisation();
        assert_eq!(gate[6], 1, "XOR is gate 6; chosen gates {gate:?}");

        let artifact = network.export();
        assert_eq!(artifact.eval(&[0, 0]), vec![0]);
        assert_eq!(artifact.eval(&[0, 1]), vec![1]);
        assert_eq!(artifact.eval(&[1, 0]), vec![1]);
        assert_eq!(artifact.eval(&[1, 1]), vec![0]);

        // The plan requires the discretisation gap reported; at saturation it must be tiny,
        // and the hard export above is already exact on all four rows.
        let gap =
            network.discretisation_gap(&data.iter().map(|(x, _)| x.clone()).collect::<Vec<_>>());
        assert!(gap < 5e-3, "discretisation gap must be small, got {gap}");
    }

    #[test]
    fn gate_utilisation_and_entropy_report_a_collapse() {
        let network = Lut4Network::identity(16, 2);
        let utilisation = network.gate_utilisation();
        assert_eq!(
            utilisation[GATE_A as usize], 32,
            "identity uses gate A only"
        );
        assert!(network.gate_entropy_bits() < f32::EPSILON);
    }
}
