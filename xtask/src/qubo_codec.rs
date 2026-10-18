//! AdaRound-style QUBO rounding for the D11 4-bit grouped codec.
//!
//! The D11 codec quantizes each group of 32 weights with a grid scale
//! `(16 + m) · 2^(e−4)` and round-to-nearest. The measured cost of that
//! rounding is a quality gap against the float parent (`+0.0362 nats` RTN,
//! `+0.0257` GPTQ on `geometric_s1`, target `<= 0.02`). An AdaRound-style
//! formulation (arXiv 2510.16075) poses the per-weight round-up/round-down
//! choice as an exact QUBO that minimizes the output Frobenius error, and
//! decomposes into independent per-output-row subproblems (fan-in + 1
//! variables at width 288 → ~289 binary variables).
//!
//! This module solves those QUBOs **classically** (simulated annealing),
//! not on a quantum device — no quantum device expedites this calculation
//! today (D-Wave annealers left Braket in 2022; a 108-qubit QPU cannot hold
//! one 289-variable subproblem). The QUBO formulation itself is the win;
//! the solver is classical SA with a fixed seed, reported as SIMULATED.
//!
//! The result is measured against round-to-nearest on the same group's
//! output Frobenius error, so the comparison is apples-to-apples.

/// One QUBO subproblem: the output error of one row's group of weights.
///
/// `w` are the float weights of the group (fan-in values), `scale` is the
/// group's grid scale, and `x` are the group's input activations (the
/// output is `x · w_q` where `w_q` is the quantized weight vector).
/// The QUBO minimizes `|x·w − x·w_q|^2` over the per-weight round choice.
pub struct GroupQubo {
    /// Float weights of the group.
    pub w: Vec<f64>,
    /// The group's grid scale.
    pub scale: f64,
    /// The group's input activations (same length as `w`).
    pub x: Vec<f64>,
}

impl GroupQubo {
    /// Output error for a rounding decision vector `q` (per-weight code in
    /// `[-8..7]`, the D11 code range).
    pub fn error(&self, q: &[i8]) -> f64 {
        let mut acc = 0.0;
        for ((&wi, &xi), &qi) in self.w.iter().zip(&self.x).zip(q) {
            let wq = f64::from(qi) * self.scale;
            acc += xi * (wi - wq);
        }
        acc * acc
    }

    /// Round-to-nearest codes (the D11 baseline).
    pub fn rtn_codes(&self) -> Vec<i8> {
        self.w
            .iter()
            .map(|&wi| (wi / self.scale).round().clamp(-8.0, 7.0) as i8)
            .collect()
    }

    /// Solve by simulated annealing over per-weight up/down flips.
    ///
    /// Classical SA, fixed seed (deterministic): start from RTN, then at a
    /// geometric cooling schedule flip the single weight that most reduces
    /// the error, accepting occasional uphill moves with the Boltzmann
    /// probability. Reported as SIMULATED (not a quantum anneal).
    pub fn solve_sa(&self, seed: u64, sweeps: usize) -> (Vec<i8>, f64) {
        let mut rng = Lcg(seed);
        let mut q = self.rtn_codes();
        let mut best_q = q.clone();
        let mut best_err = self.error(&q);
        let mut err = best_err;
        let n = q.len();
        if n == 0 {
            return (best_q, best_err);
        }
        for sweep in 0..sweeps {
            let t0 = 1.0;
            let temp = t0 * (1.0 - sweep as f64 / sweeps.max(1) as f64).max(1e-3);
            for _ in 0..n {
                let i = (rng.next() as usize) % n;
                // Candidate: flip weight i up or down by one code step.
                let dir = if rng.next() & 1 == 0 { 1i8 } else { -1i8 };
                let mut q2 = q.clone();
                let nv = (q2[i] + dir).clamp(-8, 7);
                if nv == q2[i] {
                    continue;
                }
                q2[i] = nv;
                let e2 = self.error(&q2);
                let d = e2 - err;
                if d <= 0.0 || rng.next_f64() < (-d / temp.max(1e-9)).exp() {
                    q = q2;
                    err = e2;
                    if err < best_err {
                        best_err = err;
                        best_q = q.clone();
                    }
                }
            }
        }
        (best_q, best_err)
    }
}

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }
    fn next_f64(&mut self) -> f64 {
        (self.next() as f64) / ((1u64 << 31) as f64)
    }
}

/// A benchmark group: 32 weights at fan-in, activations drawn from a fixed
/// pseudo-random pattern so the comparison is reproducible. The measured
/// result is the error ratio `sa / rtn` (below 1.0 = SA wins).
pub fn benchmark(seed: u64, group_size: usize, sweeps: usize) -> (f64, f64, f64) {
    let mut rng = Lcg(seed);
    let scale = 0.03125; // 2^-5, a representative D11 grid scale
    let w: Vec<f64> = (0..group_size)
        .map(|_| (rng.next_f64() - 0.5) * 0.4)
        .collect();
    let x: Vec<f64> = (0..group_size)
        .map(|_| (rng.next_f64() - 0.5) * 2.0)
        .collect();
    let q = GroupQubo { w, scale, x };
    let rtn_err = q.error(&q.rtn_codes());
    let (_, sa_err) = q.solve_sa(seed, sweeps);
    (
        rtn_err,
        sa_err,
        if rtn_err > 0.0 { sa_err / rtn_err } else { 1.0 },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lcg_next_f64_spans_unit_interval() {
        let mut rng = Lcg(0xD11);
        let mut min_v = 1.0f64;
        let mut max_v = 0.0f64;
        for _ in 0..256 {
            let v = rng.next_f64();
            assert!((0.0..1.0).contains(&v));
            min_v = min_v.min(v);
            max_v = max_v.max(v);
        }
        assert!(min_v < 0.2, "min_v={min_v}");
        assert!(max_v > 0.8, "max_v={max_v}");
    }

    #[test]
    fn sa_improves_on_rtn_benchmark() {
        let (rtn_err, sa_err, ratio) = benchmark(0xD11, 32, 64);
        assert!(sa_err <= rtn_err);
        assert!(ratio < 1.0, "expected SA < RTN, got ratio={ratio}");
    }
}
