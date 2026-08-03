//! Certifier-side low-rank far-field candidate scorer for issue #290.
//!
//! This module is deliberately compiler/certifier-side. It uses floating-point
//! arithmetic and is not wired into the deployed integer runtime. The scorer
//! approximates the prototype-to-emission interaction map
//!
//! ```text
//! score(token | q) = root(token) + qᵀ Pᵀ E(token) / D
//! ```
//!
//! where `P` contains the region sign prototypes, `E` contains the sparse
//! region emission residuals, and `q` is the query sign vector. A deterministic
//! symmetric eigendecomposition of `PᵀP` retains a bounded number of principal
//! directions, yielding a measurable low-rank far-field candidate. Exact
//! context evidence and deployed status policy are intentionally not included;
//! this is the long-range candidate used by the exploratory accuracy gate.

use std::collections::{BTreeMap, BTreeSet};

use uor_r4_graph_format::ScoreQ;

use crate::score_runtime::RegionParams;

/// Configuration for the certifier-side far-field approximation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FmmConfig {
    /// Maximum number of retained prototype directions.
    pub max_rank: usize,
    /// Retain directions whose singular value is at least this fraction of
    /// the largest singular value.
    pub relative_singular_tolerance: f64,
}

impl Default for FmmConfig {
    fn default() -> Self {
        Self {
            max_rank: 20,
            relative_singular_tolerance: 1.0e-2,
        }
    }
}

impl FmmConfig {
    fn validate(self, dimension: usize) -> Result<Self, String> {
        if self.max_rank == 0 {
            return Err("FMM max_rank must be nonzero".to_owned());
        }
        if !self.relative_singular_tolerance.is_finite()
            || self.relative_singular_tolerance <= 0.0
            || self.relative_singular_tolerance > 1.0
        {
            return Err("FMM relative_singular_tolerance must be in (0, 1]".to_owned());
        }
        if dimension == 0 {
            return Err("FMM prototype dimension must be nonzero".to_owned());
        }
        Ok(Self {
            max_rank: self.max_rank.min(dimension),
            ..self
        })
    }
}

/// One prediction from the exploratory far-field scorer.
#[derive(Debug, Clone)]
pub struct FmmScoreOutcome {
    pub selected: u32,
    /// Candidate scores in ascending token order.
    pub candidates: Vec<(u32, f64)>,
    pub rank: usize,
    /// Fraction of prototype spectral energy retained by the basis.
    pub retained_energy: f64,
}

/// Deterministic low-rank far-field scorer.
#[derive(Debug, Clone)]
pub struct FmmCandidateScorer {
    config: FmmConfig,
    dimension: usize,
    rank: usize,
    /// Eigenvectors of PᵀP, stored column-major as `dimension × rank`.
    basis: Vec<f64>,
    /// Projected emission map, stored row-major as `rank × vocab`.
    token_factors: Vec<f64>,
    root_prior: BTreeMap<u32, f64>,
    root_floor: f64,
    candidate_tokens: Vec<u32>,
    retained_energy: f64,
}

/// One prediction from the fixed-point translation-table candidate.
#[derive(Debug, Clone)]
pub struct FmmFixedScoreOutcome {
    pub selected: u32,
    /// Candidate scores in ascending token order, represented as Q16.16 raw
    /// integers widened to i64 for overflow-safe comparison.
    pub candidates: Vec<(u32, i64)>,
    pub rank: usize,
}

/// Quantized form of [`FmmCandidateScorer`]. The basis uses signed Q1.15
/// values; token factors use a common power-of-two fractional scale, so the
/// score accumulation and final Q16.16 conversion are integer operations.
/// This is a feasibility prototype for a future packed runtime table, not yet
/// the deployed allocation-free kernel.
#[derive(Debug, Clone)]
pub struct FmmFixedCandidateScorer {
    dimension: usize,
    rank: usize,
    basis_q15: Vec<i16>,
    token_factors_q: Vec<i32>,
    factor_fraction_bits: u8,
    root_prior: BTreeMap<u32, ScoreQ>,
    root_floor: ScoreQ,
    candidate_tokens: Vec<u32>,
}

impl FmmCandidateScorer {
    /// Build a scorer from the graph's region prototypes and sparse emission
    /// residuals. This constructor allocates once and performs all floating
    /// point work; `score` only evaluates the bounded research candidate.
    pub fn from_graph_parts(
        regions: &[RegionParams],
        emissions: &[BTreeMap<u32, ScoreQ>],
        root_prior: &BTreeMap<u32, ScoreQ>,
        root_floor: ScoreQ,
        vocab: u32,
        config: FmmConfig,
    ) -> Result<Self, String> {
        if regions.is_empty() || regions.len() != emissions.len() {
            return Err("FMM regions and emissions must be nonempty and aligned".to_owned());
        }
        if vocab == 0 {
            return Err("FMM vocabulary must be nonzero".to_owned());
        }
        let dimension = regions[0].sig.len() * 8;
        let config = config.validate(dimension)?;
        if regions.iter().any(|r| r.sig.len() * 8 != dimension) {
            return Err("FMM region signatures have inconsistent widths".to_owned());
        }

        let mut gram = vec![0.0; dimension * dimension];
        for region in regions {
            let row = sign_row(&region.sig, dimension);
            for left in 0..dimension {
                for right in 0..=left {
                    gram[left * dimension + right] += row[left] * row[right];
                }
            }
        }
        for left in 0..dimension {
            for right in 0..left {
                gram[right * dimension + left] = gram[left * dimension + right];
            }
        }

        let (mut eigenvalues, eigenvectors) = symmetric_eigen(gram, dimension)?;
        let largest = eigenvalues.first().copied().unwrap_or(0.0).max(0.0).sqrt();
        if largest <= f64::EPSILON {
            return Err("FMM prototypes have zero spectral energy".to_owned());
        }
        let cutoff = largest * config.relative_singular_tolerance;
        let total_energy = eigenvalues.iter().sum::<f64>();
        let rank = eigenvalues
            .iter()
            .take(config.max_rank)
            .take_while(|&&value| value.max(0.0).sqrt() >= cutoff)
            .count()
            .max(1);
        eigenvalues.truncate(rank);

        let retained_energy = if total_energy > 0.0 {
            eigenvalues.iter().sum::<f64>() / total_energy
        } else {
            0.0
        };
        let basis = take_columns(&eigenvectors, dimension, rank);

        let mut token_set = BTreeSet::new();
        token_set.extend(root_prior.keys().copied());
        for map in emissions {
            token_set.extend(map.keys().copied());
        }
        let candidate_tokens: Vec<u32> = token_set.into_iter().collect();
        if candidate_tokens.is_empty() {
            return Err("FMM graph has no root or emission candidates".to_owned());
        }
        let mut token_index = BTreeMap::new();
        for (index, &token) in candidate_tokens.iter().enumerate() {
            token_index.insert(token, index);
        }
        let mut token_factors = vec![0.0; rank * candidate_tokens.len()];
        for (region, map) in regions.iter().zip(emissions) {
            let row = sign_row(&region.sig, dimension);
            let mut projection = vec![0.0; rank];
            for component in 0..rank {
                let mut value = 0.0;
                for coordinate in 0..dimension {
                    value += row[coordinate] * basis[coordinate * rank + component];
                }
                projection[component] = value / dimension as f64;
            }
            for (&token, &score) in map {
                let Some(&index) = token_index.get(&token) else {
                    continue;
                };
                let emission = score.raw() as f64 / 65_536.0;
                for component in 0..rank {
                    token_factors[component * candidate_tokens.len() + index] +=
                        projection[component] * emission;
                }
            }
        }

        Ok(Self {
            config,
            dimension,
            rank,
            basis,
            token_factors,
            root_prior: root_prior
                .iter()
                .map(|(&token, &score)| (token, score.raw() as f64 / 65_536.0))
                .collect(),
            root_floor: root_floor.raw() as f64 / 65_536.0,
            candidate_tokens,
            retained_energy,
        })
    }

    pub fn config(&self) -> FmmConfig {
        self.config
    }

    pub fn rank(&self) -> usize {
        self.rank
    }

    pub fn retained_energy(&self) -> f64 {
        self.retained_energy
    }

    /// Quantize the float candidate into a deterministic fixed-point table.
    pub fn fixed_point(&self) -> Result<FmmFixedCandidateScorer, String> {
        let basis_q15 = self
            .basis
            .iter()
            .map(|&value| (value * 32_768.0).round().clamp(-32_767.0, 32_767.0) as i16)
            .collect();
        let max_factor = self
            .token_factors
            .iter()
            .map(|value| value.abs())
            .fold(0.0, f64::max);
        let factor_fraction_bits = choose_fraction_bits(max_factor);
        let factor_scale = (1u64 << factor_fraction_bits) as f64;
        let token_factors_q = self
            .token_factors
            .iter()
            .map(|&value| {
                (value * factor_scale)
                    .round()
                    .clamp(i32::MIN as f64, i32::MAX as f64) as i32
            })
            .collect();
        Ok(FmmFixedCandidateScorer {
            dimension: self.dimension,
            rank: self.rank,
            basis_q15,
            token_factors_q,
            factor_fraction_bits,
            root_prior: self
                .root_prior
                .iter()
                .map(|(&token, &score)| {
                    (token, ScoreQ::from_raw((score * 65_536.0).round() as i32))
                })
                .collect(),
            root_floor: ScoreQ::from_raw((self.root_floor * 65_536.0).round() as i32),
            candidate_tokens: self.candidate_tokens.clone(),
        })
    }

    /// Approximate storage occupied by the float factors and token metadata.
    pub fn storage_bytes(&self) -> usize {
        self.basis.len() * std::mem::size_of::<f64>()
            + self.token_factors.len() * std::mem::size_of::<f64>()
            + self.candidate_tokens.len() * std::mem::size_of::<u32>()
            + self.root_prior.len() * (std::mem::size_of::<u32>() + std::mem::size_of::<f64>())
    }

    /// Score one sign signature with the low-rank far-field approximation.
    pub fn score(&self, sig: &[u8], recent_tokens: &[u32]) -> Result<FmmScoreOutcome, String> {
        if sig.len() * 8 != self.dimension {
            return Err("FMM query signature width does not match the graph".to_owned());
        }
        let query = sign_row(sig, self.dimension);
        let mut projected = vec![0.0; self.rank];
        for (component, projected_value) in projected.iter_mut().enumerate() {
            for (coordinate, &query_value) in query.iter().enumerate() {
                *projected_value += query_value * self.basis[coordinate * self.rank + component];
            }
        }
        let mut candidates = Vec::with_capacity(self.candidate_tokens.len());
        for (index, &token) in self.candidate_tokens.iter().enumerate() {
            let mut score = self
                .root_prior
                .get(&token)
                .copied()
                .unwrap_or(self.root_floor);
            for (component, &projection) in projected.iter().enumerate() {
                score += projection
                    * self.token_factors[component * self.candidate_tokens.len() + index];
            }
            if recent_tokens.contains(&token) {
                score -= 2_000_000.0 / 65_536.0;
            }
            candidates.push((token, score));
        }
        let selected = candidates
            .iter()
            .max_by(|(left_token, left_score), (right_token, right_score)| {
                left_score
                    .total_cmp(right_score)
                    .then_with(|| right_token.cmp(left_token))
            })
            .map(|&(token, _)| token)
            .ok_or("FMM produced no candidates")?;
        Ok(FmmScoreOutcome {
            selected,
            candidates,
            rank: self.rank,
            retained_energy: self.retained_energy,
        })
    }
}

impl FmmFixedCandidateScorer {
    pub fn rank(&self) -> usize {
        self.rank
    }

    pub fn factor_fraction_bits(&self) -> u8 {
        self.factor_fraction_bits
    }

    /// Approximate storage occupied by the quantized translation table and
    /// token metadata.
    pub fn storage_bytes(&self) -> usize {
        self.basis_q15.len() * std::mem::size_of::<i16>()
            + self.token_factors_q.len() * std::mem::size_of::<i32>()
            + self.candidate_tokens.len() * std::mem::size_of::<u32>()
            + self.root_prior.len() * (std::mem::size_of::<u32>() + std::mem::size_of::<i32>())
    }

    /// Select the best token without allocating. The caller owns the
    /// rank-sized projection scratch; this is the adapter shape a future
    /// fixed-capacity runtime can embed in its step state.
    pub fn select_into(
        &self,
        sig: &[u8],
        recent_tokens: &[u32],
        projected: &mut [i64],
    ) -> Result<(u32, i64), String> {
        if sig.len() * 8 != self.dimension {
            return Err("fixed FMM query signature width does not match the graph".to_owned());
        }
        if projected.len() < self.rank {
            return Err("fixed FMM projection scratch is too small".to_owned());
        }
        projected[..self.rank].fill(0);
        for (component, projected_value) in projected[..self.rank].iter_mut().enumerate() {
            for coordinate in 0..self.dimension {
                let sign = if sig[coordinate / 8] & (1 << (coordinate % 8)) != 0 {
                    1i64
                } else {
                    -1i64
                };
                *projected_value +=
                    sign * i64::from(self.basis_q15[coordinate * self.rank + component]);
            }
        }
        let token_count = self.candidate_tokens.len();
        let mut best: Option<(u32, i64)> = None;
        for (index, &token) in self.candidate_tokens.iter().enumerate() {
            let mut interaction = 0i128;
            for (component, &projection) in projected[..self.rank].iter().enumerate() {
                interaction += projection as i128
                    * i128::from(self.token_factors_q[component * token_count + index]);
            }
            let mut score =
                i64::from(
                    self.root_prior
                        .get(&token)
                        .copied()
                        .unwrap_or(self.root_floor)
                        .raw(),
                ) + round_shift_signed(interaction, self.factor_fraction_bits.saturating_sub(1));
            if recent_tokens.contains(&token) {
                score = score.saturating_sub(2_000_000);
            }
            let replace = best.is_none_or(|(best_token, best_score)| {
                score > best_score || (score == best_score && token < best_token)
            });
            if replace {
                best = Some((token, score));
            }
        }
        best.ok_or("fixed FMM produced no candidates".to_owned())
    }

    /// Score one signature using only integer table values after the
    /// compiler-selected quantization scales have been fixed.
    pub fn score(&self, sig: &[u8], recent_tokens: &[u32]) -> Result<FmmFixedScoreOutcome, String> {
        if sig.len() * 8 != self.dimension {
            return Err("fixed FMM query signature width does not match the graph".to_owned());
        }
        let mut projected = vec![0i64; self.rank];
        for (component, projected_value) in projected.iter_mut().enumerate() {
            for coordinate in 0..self.dimension {
                let sign = if sig[coordinate / 8] & (1 << (coordinate % 8)) != 0 {
                    1i64
                } else {
                    -1i64
                };
                *projected_value +=
                    sign * i64::from(self.basis_q15[coordinate * self.rank + component]);
            }
        }
        let token_count = self.candidate_tokens.len();
        let mut candidates = Vec::with_capacity(token_count);
        for (index, &token) in self.candidate_tokens.iter().enumerate() {
            let mut interaction = 0i128;
            for (component, &projection) in projected.iter().enumerate() {
                interaction += projection as i128
                    * i128::from(self.token_factors_q[component * token_count + index]);
            }
            let mut score =
                i64::from(
                    self.root_prior
                        .get(&token)
                        .copied()
                        .unwrap_or(self.root_floor)
                        .raw(),
                ) + round_shift_signed(interaction, self.factor_fraction_bits.saturating_sub(1));
            if recent_tokens.contains(&token) {
                score = score.saturating_sub(2_000_000);
            }
            candidates.push((token, score));
        }
        let selected = candidates
            .iter()
            .max_by(|(left_token, left_score), (right_token, right_score)| {
                left_score
                    .cmp(right_score)
                    .then_with(|| right_token.cmp(left_token))
            })
            .map(|&(token, _)| token)
            .ok_or("fixed FMM produced no candidates")?;
        Ok(FmmFixedScoreOutcome {
            selected,
            candidates,
            rank: self.rank,
        })
    }
}

fn choose_fraction_bits(max_abs: f64) -> u8 {
    for bits in (0..=30).rev() {
        if max_abs * (1u64 << bits) as f64 <= i32::MAX as f64 {
            return bits.max(1);
        }
    }
    0
}

fn round_shift_signed(value: i128, shift: u8) -> i64 {
    if shift == 0 {
        return value.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64;
    }
    let divisor = 1i128 << shift;
    let adjusted = if value >= 0 {
        value.saturating_add(divisor / 2)
    } else {
        value.saturating_sub(divisor / 2)
    };
    (adjusted / divisor).clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
}

fn sign_row(sig: &[u8], dimension: usize) -> Vec<f64> {
    (0..dimension)
        .map(|bit| {
            if sig[bit / 8] & (1 << (bit % 8)) != 0 {
                1.0
            } else {
                -1.0
            }
        })
        .collect()
}

fn take_columns(values: &[f64], dimension: usize, rank: usize) -> Vec<f64> {
    let mut out = vec![0.0; dimension * rank];
    for row in 0..dimension {
        for column in 0..rank {
            out[row * rank + column] = values[row * dimension + column];
        }
    }
    out
}

/// Deterministic Jacobi eigensolver for a real symmetric matrix. Eigenpairs
/// are returned in descending eigenvalue order; the fixed sweep and pivot
/// order make the certifier output reproducible without an external LAPACK
/// dependency.
fn symmetric_eigen(mut matrix: Vec<f64>, dimension: usize) -> Result<(Vec<f64>, Vec<f64>), String> {
    if matrix.len() != dimension * dimension || dimension == 0 {
        return Err("invalid symmetric matrix dimensions".to_owned());
    }
    let mut vectors = vec![0.0; dimension * dimension];
    for index in 0..dimension {
        vectors[index * dimension + index] = 1.0;
    }
    for _ in 0..(dimension * 16).max(32) {
        let mut pivot = None;
        let mut max = 0.0;
        for row in 0..dimension {
            for column in (row + 1)..dimension {
                let value = matrix[row * dimension + column].abs();
                if value > max {
                    max = value;
                    pivot = Some((row, column));
                }
            }
        }
        if max <= 1.0e-12 {
            break;
        }
        let (p, q) = pivot.ok_or("Jacobi pivot missing")?;
        let app = matrix[p * dimension + p];
        let aqq = matrix[q * dimension + q];
        let apq = matrix[p * dimension + q];
        let theta = 0.5 * (aqq - app) / apq;
        let t = theta.signum() / (theta.abs() + (theta * theta + 1.0).sqrt());
        let c = 1.0 / (1.0 + t * t).sqrt();
        let s = t * c;
        for index in 0..dimension {
            let mip = matrix[index * dimension + p];
            let miq = matrix[index * dimension + q];
            matrix[index * dimension + p] = c * mip - s * miq;
            matrix[index * dimension + q] = s * mip + c * miq;
        }
        for index in 0..dimension {
            let mpi = matrix[p * dimension + index];
            let mqi = matrix[q * dimension + index];
            matrix[p * dimension + index] = c * mpi - s * mqi;
            matrix[q * dimension + index] = s * mpi + c * mqi;
            let vpi = vectors[index * dimension + p];
            let vqi = vectors[index * dimension + q];
            vectors[index * dimension + p] = c * vpi - s * vqi;
            vectors[index * dimension + q] = s * vpi + c * vqi;
        }
    }
    let mut order: Vec<usize> = (0..dimension).collect();
    order.sort_by(|&left, &right| {
        matrix[right * dimension + right]
            .total_cmp(&matrix[left * dimension + left])
            .then_with(|| left.cmp(&right))
    });
    let eigenvalues = order
        .iter()
        .map(|&index| matrix[index * dimension + index].max(0.0))
        .collect();
    let mut ordered_vectors = vec![0.0; dimension * dimension];
    for row in 0..dimension {
        for (column, &source) in order.iter().enumerate() {
            ordered_vectors[row * dimension + column] = vectors[row * dimension + source];
        }
    }
    Ok((eigenvalues, ordered_vectors))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score_runtime::RegionParams;
    use uor_r4_core::transformerless::compiler::SIG_BYTES;

    fn region(node: u32, first_byte: u8) -> RegionParams {
        let mut sig = [0u8; SIG_BYTES];
        sig[0] = first_byte;
        RegionParams {
            node,
            depth: 1,
            radius: 8,
            sig,
            parent: None,
        }
    }

    #[test]
    fn low_rank_candidate_is_deterministic_and_selects_emission() {
        let regions = vec![region(1, 0b0000_0001), region(2, 0b0000_0011)];
        let mut first = BTreeMap::new();
        first.insert(7, ScoreQ::from_raw(100 << 16));
        let mut second = BTreeMap::new();
        second.insert(9, ScoreQ::from_raw(80 << 16));
        let root = BTreeMap::new();
        let scorer = FmmCandidateScorer::from_graph_parts(
            &regions,
            &[first.clone(), second.clone()],
            &root,
            ScoreQ::from_raw(0),
            16,
            FmmConfig {
                max_rank: 1,
                relative_singular_tolerance: 1.0e-2,
            },
        )
        .expect("candidate builds");
        let mut left_sig = [0u8; SIG_BYTES];
        left_sig[0] = 0b0000_0001;
        let mut right_sig = [0u8; SIG_BYTES];
        right_sig[0] = 0b0000_0011;
        let left = scorer.score(&left_sig, &[]).expect("scores");
        let right = scorer.score(&right_sig, &[]).expect("scores");
        assert_eq!(left.selected, 7);
        assert_eq!(right.selected, 7);
        assert_eq!(left.rank, 1);
        assert_eq!(
            left.candidates,
            scorer.score(&left_sig, &[]).unwrap().candidates
        );
        assert!(scorer.retained_energy() > 0.0);
        let fixed = scorer.fixed_point().expect("fixed candidate builds");
        assert_eq!(fixed.score(&left_sig, &[]).unwrap().selected, left.selected);
        let mut projected = [0i64; 1];
        assert_eq!(
            fixed.select_into(&left_sig, &[], &mut projected).unwrap().0,
            left.selected
        );
        assert!(fixed.storage_bytes() < scorer.storage_bytes());
    }

    #[test]
    fn invalid_configuration_fails_closed() {
        let regions = vec![region(1, 1)];
        let emissions = vec![BTreeMap::from([(1, ScoreQ::from_raw(1))])];
        let error = FmmCandidateScorer::from_graph_parts(
            &regions,
            &emissions,
            &BTreeMap::new(),
            ScoreQ::from_raw(0),
            2,
            FmmConfig {
                max_rank: 0,
                ..FmmConfig::default()
            },
        )
        .expect_err("zero rank must fail");
        assert!(error.contains("max_rank"));
    }
}
