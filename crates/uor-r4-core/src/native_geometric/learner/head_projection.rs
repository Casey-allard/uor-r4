//! Activation-aware dyadic ternary projection of a frozen floating output head.
//!
//! The existing export rule ([`TernaryLinear::quantize`]) picks one dyadic scale per row from that
//! row's maximum absolute weight and rounds each coefficient independently. It ignores the
//! activations those coefficients multiply. This module supplies one bounded alternative: a
//! **post-training calibration** that chooses the ternary codes and the power-of-two row scale
//! minimising the occurrence-weighted squared score-reconstruction error
//!
//! ```text
//! J(q, s) = (a q - w)^T G (a q - w),   a = 2^s,
//! G = (1/N) sum_c n_c h(c) h(c)^T      (uncentered second moment over fit contexts)
//! ```
//!
//! Because the bias is frozen it cancels, and `J` is exactly the mean over occurrences of
//! `((a q - w)^T h)^2` in squared integer-score units. Multiplying by `2^(-2F)` expresses the same
//! objective in natural-logit units. `G` may be singular; it is never inverted, jittered,
//! regularised or rank-truncated.
//!
//! This is hard-weight calibration of a frozen head, not a new gradient fit and not a claim about
//! feature capacity.
#![forbid(unsafe_code)]

use super::realtext_support::{paired_interval, Agg};

/// Feature width of the retained head.
pub const DV: usize = 128;

/// Maximum output-row shift admitted by the declared envelope for `dv=128`, `norm_bits=6`.
///
/// `Config::check_row_shift` requires `(dv * 2^norm_bits) << shift <= i32::MAX / 4`, i.e.
/// `8192 << shift <= 536_870_911`, so `shift <= 15`. Derived from source, not from observed
/// activations.
pub const MAX_SAFE_SHIFT: u32 = 15;

/// Uncentered second-moment Gram over the fit population, accumulated with checked wide integers.
#[derive(Clone, Debug)]
pub struct Gram {
    dv: usize,
    /// Cumulative numerator, upper triangle row-major (`i <= j`).
    numerator: Vec<i128>,
    /// Symmetric `f64` matrix after [`Gram::finalize`].
    pub g: Vec<f64>,
    pub n: u64,
}

impl Gram {
    pub fn new(dv: usize) -> Self {
        Self {
            dv,
            numerator: vec![0; dv * (dv + 1) / 2],
            g: vec![0.0; dv * dv],
            n: 0,
        }
    }

    #[inline]
    fn upper_index(&self, i: usize, j: usize) -> usize {
        let (lo, hi) = if i <= j { (i, j) } else { (j, i) };
        lo * (2 * self.dv - lo + 1) / 2 + (hi - lo)
    }

    /// Accumulate one context's occurrence weight. Uses checked `i128` arithmetic.
    pub fn add_occurrence(&mut self, h: &[i32], weight: u64) -> Result<(), String> {
        if h.len() != self.dv {
            return Err("feature width differs from the Gram width".into());
        }
        if weight == 0 {
            return Ok(());
        }
        let w = weight as i128;
        for i in 0..self.dv {
            let hi = h[i] as i128;
            for j in i..self.dv {
                let prod = hi
                    .checked_mul(h[j] as i128)
                    .and_then(|p| p.checked_mul(w))
                    .ok_or("Gram numerator overflow")?;
                let idx = self.upper_index(i, j);
                self.numerator[idx] = self.numerator[idx]
                    .checked_add(prod)
                    .ok_or("Gram numerator overflow")?;
            }
        }
        self.n += weight;
        Ok(())
    }

    /// Convert the numerator to the symmetric `f64` matrix and divide by `N`.
    pub fn finalize(&mut self) -> Result<(), String> {
        if self.n == 0 {
            return Err("Gram population is empty".into());
        }
        let n = self.n as f64;
        let mut out = vec![0.0f64; self.dv * self.dv];
        for i in 0..self.dv {
            for j in i..self.dv {
                let v = self.numerator[self.upper_index(i, j)] as f64 / n;
                out[i * self.dv + j] = v;
                out[j * self.dv + i] = v;
            }
        }
        self.g = out;
        Ok(())
    }

    /// `out = G * e`.
    pub fn mul(&self, e: &[f64], out: &mut [f64]) {
        debug_assert_eq!(e.len(), self.dv);
        debug_assert_eq!(out.len(), self.dv);
        for i in 0..self.dv {
            let row = &self.g[i * self.dv..(i + 1) * self.dv];
            let mut acc = 0.0f64;
            for j in 0..self.dv {
                acc += row[j] * e[j];
            }
            out[i] = acc;
        }
    }

    /// Quadratic form `e^T G e`.
    pub fn quadratic(&self, e: &[f64]) -> f64 {
        let mut g = vec![0.0f64; self.dv];
        self.mul(e, &mut g);
        e.iter().zip(g.iter()).map(|(a, b)| a * b).sum()
    }

    pub fn diag(&self, i: usize) -> f64 {
        self.g[i * self.dv + i]
    }
}

/// Round away from zero, then clip to the ternary set.
#[inline]
pub fn nearest_ternary(v: f64) -> i8 {
    if v >= 0.5 {
        1
    } else if v <= -0.5 {
        -1
    } else {
        0
    }
}

/// Which seed a candidate row came from.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Seed {
    /// Nearest ternary coefficients at this shift.
    Nearest,
    /// The retained empirical head's row at its own shift.
    Empirical,
}

/// One projected row candidate.
#[derive(Clone, Debug)]
pub struct RowCandidate {
    pub shift: u32,
    pub seed: Seed,
    pub codes: Vec<i8>,
    pub j: f64,
}

/// Solve `J(q,s)` for one row with exactly two coordinate sweeps per seed, no convergence loop.
#[allow(clippy::too_many_arguments)]
pub fn project_row(
    w: &[f32],
    s0: u32,
    s_e: u32,
    empirical_row: Option<&[i8]>,
    g: &Gram,
    max_shift: u32,
) -> RowCandidate {
    debug_assert_eq!(w.len(), g.dv);
    // Candidate shifts: sorted unique {s0-1, s0, s0+1, sE}, dropping negative and invalid values.
    let mut shifts: Vec<u32> = vec![s0.saturating_sub(1), s0, s0.saturating_add(1), s_e];
    shifts.sort_unstable();
    shifts.dedup();
    shifts.retain(|s| *s <= max_shift);

    // Q0 first: an explicit untouched candidate.
    let q0_codes: Vec<i8> = (0..g.dv)
        .map(|j| nearest_ternary(w[j] as f64 / (1u64 << s0) as f64))
        .collect();
    let mut best = RowCandidate {
        shift: s0,
        seed: Seed::Nearest,
        j: reconstruction_error(w, &q0_codes, s0, g),
        codes: q0_codes,
    };

    for &s in shifts.iter() {
        let a = (1u64 << s) as f64;
        let mut seeds: Vec<(Seed, Vec<i8>)> = Vec::new();
        if s != s0 {
            seeds.push((
                Seed::Nearest,
                (0..g.dv)
                    .map(|j| nearest_ternary(w[j] as f64 / a))
                    .collect(),
            ));
        }
        if s == s_e {
            if let Some(row) = empirical_row {
                seeds.push((Seed::Empirical, row.to_vec()));
            }
        }
        for (seed, init) in seeds {
            let codes = sweep(w, &init, s, g);
            let j = reconstruction_error(w, &codes, s, g);
            if j < best.j {
                best = RowCandidate {
                    shift: s,
                    seed,
                    codes,
                    j,
                };
            }
        }
    }
    best
}

/// Two coordinate sweeps (ascending then descending) minimising `J` for the fixed scale `a`.
fn sweep(w: &[f32], init: &[i8], s: u32, g: &Gram) -> Vec<i8> {
    let dv = g.dv;
    let a = (1u64 << s) as f64;
    let mut q = init.to_vec();
    // e = a q - w
    let mut e = vec![0.0f64; dv];
    for j in 0..dv {
        e[j] = a * q[j] as f64 - w[j] as f64;
    }
    let mut gv = vec![0.0f64; dv];
    g.mul(&e, &mut gv);

    for pass in 0..2 {
        let order: Vec<usize> = if pass == 0 {
            (0..dv).collect()
        } else {
            (0..dv).rev().collect()
        };
        for &j in order.iter() {
            let cur = q[j];
            let mut best_code = cur;
            let mut best_d = 0.0f64;
            for b in [-1i8, 0, 1] {
                if b == cur {
                    continue;
                }
                let delta = a * (b - cur) as f64;
                let dj = 2.0 * delta * gv[j] + delta * delta * g.diag(j);
                // Strictly-better only, so an equal alternative keeps the current code; the
                // ascending iteration order breaks ties toward the smaller code.
                if dj < best_d {
                    best_d = dj;
                    best_code = b;
                }
            }
            if best_code != cur && best_d < 0.0 {
                let delta = a * (best_code - cur) as f64;
                q[j] = best_code;
                e[j] += delta;
                let col = j;
                for k in 0..dv {
                    gv[k] += delta * g.g[k * dv + col];
                }
            }
        }
    }
    q
}

/// Independently recompute the full quadratic form for a candidate.
pub fn reconstruction_error(w: &[f32], codes: &[i8], s: u32, g: &Gram) -> f64 {
    let a = (1u64 << s) as f64;
    let e: Vec<f64> = (0..g.dv)
        .map(|j| a * codes[j] as f64 - w[j] as f64)
        .collect();
    g.quadratic(&e)
}

// ---------------------------------------------------------------------------
// Screen helpers
// ---------------------------------------------------------------------------

/// A paired document-bootstrap interval.
#[derive(Clone, Copy, Debug)]
pub struct Pair {
    pub point: f64,
    pub lo: f64,
    pub hi: f64,
}

impl Pair {
    fn from(t: (f64, f64, f64)) -> Self {
        Self {
            point: t.0,
            lo: t.1,
            hi: t.2,
        }
    }
}

/// Practical improvement of `candidate` over its **declared** baseline:
/// `CE_baseline - CE_candidate` (positive means the candidate is better).
pub fn practical_gain(baseline: &Agg, candidate: &Agg) -> Pair {
    Pair::from(paired_interval(baseline, candidate, 0x1234_5678))
}

/// `true` when the practical gain clears `margin` bits and its paired lower bound is above zero.
pub fn practical_screen(gain: Pair, margin: f64) -> bool {
    gain.point >= margin && gain.lo > 0.0
}

/// Smoothing attribution: `CE_E - CE_S` (positive means smoothed is better). Never compares two
/// different teachers' KL values.
pub fn smoothing_attribution(e: &Agg, s: &Agg) -> Pair {
    Pair::from(paired_interval(e, s, 0x1234_5678))
}

/// The conditional floating gate uses the **smoothed arm's own** teacher KL, not a maximum over
/// different teachers.
pub fn floating_gate(smoothed_teacher_kl_bits: f64, margin: f64) -> bool {
    smoothed_teacher_kl_bits > margin
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn gram_from(rows: &[Vec<i32>], weights: &[u64]) -> Gram {
        let dv = rows[0].len();
        let mut g = Gram::new(dv);
        for (h, w) in rows.iter().zip(weights.iter()) {
            g.add_occurrence(h, *w).unwrap();
        }
        g.finalize().unwrap();
        g
    }

    /// The accumulated Gram equals the direct repeated-occurrence computation.
    #[test]
    fn gram_equals_direct_repeated_occurrence_sum() {
        let dv = 4usize;
        let a = vec![3i32, -1, 2, 5];
        let b = vec![-2i32, 4, 0, 1];
        let g = gram_from(&[a.clone(), b.clone()], &[3, 2]);
        for i in 0..dv {
            for j in 0..dv {
                let direct =
                    (3.0 * a[i] as f64 * a[j] as f64 + 2.0 * b[i] as f64 * b[j] as f64) / 5.0;
                assert!((g.g[i * dv + j] - direct).abs() < 1e-12);
            }
        }
        // Expanding y -> y,y with half weights changes nothing.
        let g2 = gram_from(&[a.clone(), a.clone(), b.clone(), b.clone()], &[1, 2, 1, 1]);
        for i in 0..g.g.len() {
            assert!((g.g[i] - g2.g[i]).abs() < 1e-12);
        }
    }

    /// The coordinate update changes `J` by exactly `2 delta g_j + delta^2 G_jj`.
    #[test]
    fn coordinate_delta_matches_direct_objective_change() {
        let dv = 6usize;
        let rows = vec![vec![2i32, -1, 3, 0, 4, -2], vec![1, 1, -1, 2, 0, 3]];
        let g = gram_from(&rows, &[1, 1]);
        let w: Vec<f32> = vec![0.7, -1.3, 2.2, 0.1, -0.9, 1.6];
        let s = 1u32;
        let a = (1u64 << s) as f64;
        let q: Vec<i8> = (0..dv).map(|j| nearest_ternary(w[j] as f64 / a)).collect();
        let j_before = reconstruction_error(&w, &q, s, &g);
        // Direct objective change for flipping coordinate 2 from q[2] to a new code.
        let j_alt = {
            let mut q2 = q.clone();
            q2[2] = -1;
            reconstruction_error(&w, &q2, s, &g)
        };
        let delta = a * (-1.0 - q[2] as f64);
        let mut e = vec![0.0; dv];
        for j in 0..dv {
            e[j] = a * q[j] as f64 - w[j] as f64;
        }
        let mut gv = vec![0.0; dv];
        g.mul(&e, &mut gv);
        let predicted = 2.0 * delta * gv[2] + delta * delta * g.diag(2);
        assert!((j_alt - j_before - predicted).abs() < 1e-9);
    }

    /// Degenerate features: a zero Gram leaves every code unchanged, and collinear rows cannot
    /// create spurious improvement.
    #[test]
    fn zero_and_collinear_features_are_handled() {
        let g = gram_from(&[vec![0i32, 0, 0]], &[5]);
        let w: Vec<f32> = vec![0.9, -2.0, 0.4];
        let c = project_row(&w, 1, 1, None, &g, MAX_SAFE_SHIFT);
        assert_eq!(c.j, 0.0);
        // Collinear context rows give a rank-1 Gram; the objective stays finite and non-negative.
        let g2 = gram_from(&[vec![1i32, 2, 3], vec![2, 4, 6]], &[1, 1]);
        let c2 = project_row(&w, 1, 1, None, &g2, MAX_SAFE_SHIFT);
        assert!(c2.j >= 0.0 && c2.j.is_finite());
    }

    /// The two sweeps are deterministic and the search never increases the fit surrogate relative
    /// to the untouched Q0 row.
    #[test]
    fn projection_is_deterministic_and_never_worse_than_q0() {
        let dv = 8usize;
        let rows: Vec<Vec<i32>> = vec![
            vec![3, -2, 1, 0, 4, -1, 2, 5],
            vec![-1, 4, 2, 3, 0, -3, 1, 2],
            vec![2, 2, -2, 1, 1, 0, -4, 3],
        ];
        let g = gram_from(&rows, &[2, 3, 1]);
        let w: Vec<f32> = vec![1.31, -0.77, 2.44, -2.02, 0.13, -1.44, 3.01, -0.22];
        let s0 = 1u32;
        let q0: Vec<i8> = (0..dv)
            .map(|j| nearest_ternary(w[j] as f64 / (1u64 << s0) as f64))
            .collect();
        let j0 = reconstruction_error(&w, &q0, s0, &g);
        let c1 = project_row(&w, s0, 2, Some(&vec![1i8; dv]), &g, MAX_SAFE_SHIFT);
        let c2 = project_row(&w, s0, 2, Some(&vec![1i8; dv]), &g, MAX_SAFE_SHIFT);
        assert_eq!(c1.codes, c2.codes);
        assert_eq!(c1.shift, c2.shift);
        assert!(
            c1.j <= j0 + 1e-9,
            "QG fit {} must not exceed Q0 fit {}",
            c1.j,
            j0
        );
    }

    /// Nearest-ternary tie semantics: exactly +/-0.5 rounds to magnitude one, and 0.49 rounds to 0.
    #[test]
    fn nearest_ternary_ties_resolve_to_magnitude_one() {
        assert_eq!(nearest_ternary(0.5), 1);
        assert_eq!(nearest_ternary(-0.5), -1);
        assert_eq!(nearest_ternary(0.49), 0);
        assert_eq!(nearest_ternary(-0.49), 0);
        assert_eq!(nearest_ternary(0.0), 0);
    }

    /// Row shifts outside the declared envelope are never selected.
    #[test]
    fn invalid_row_shifts_are_excluded() {
        let g = gram_from(&[vec![1i32, 2, 3, 4]], &[1]);
        let w: Vec<f32> = vec![0.2, -0.3, 0.4, -0.5];
        // s0 = 0 and sE = 20: the +1 candidate is 1 and sE is dropped.
        let c = project_row(&w, 0, 20, None, &g, MAX_SAFE_SHIFT);
        assert!(c.shift <= MAX_SAFE_SHIFT);
        // A row with s0 at the boundary cannot shift up.
        let c2 = project_row(&w, MAX_SAFE_SHIFT, MAX_SAFE_SHIFT, None, &g, MAX_SAFE_SHIFT);
        assert!(c2.shift <= MAX_SAFE_SHIFT);
    }

    /// The repaired screen helper: two heads can both beat the parent while the smoothed arm does
    /// not beat the empirical arm, and the floating gate reads the smoothed arm's own KL.
    #[test]
    fn screen_helper_separates_practical_gain_from_smoothing_attribution() {
        let parent = Agg {
            rows: vec![("a".into(), 8.0, 1), ("b".into(), 8.0, 1)],
        };
        let e = Agg {
            rows: vec![("a".into(), 7.0, 1), ("b".into(), 7.0, 1)],
        };
        // S beats the parent but loses to E.
        let s = Agg {
            rows: vec![("a".into(), 7.2, 1), ("b".into(), 7.2, 1)],
        };
        let g_e = practical_gain(&parent, &e);
        let g_s = practical_gain(&parent, &s);
        assert!(practical_screen(g_e, 0.10));
        assert!(practical_screen(g_s, 0.10));
        let attr = smoothing_attribution(&e, &s);
        assert!(attr.point < 0.0, "smoothed must not be attributed a gain");
        assert!(!practical_screen(attr, 0.0), "S does not beat E");
        assert!(floating_gate(0.5, 0.10));
        assert!(!floating_gate(0.05, 0.10));
    }
}
