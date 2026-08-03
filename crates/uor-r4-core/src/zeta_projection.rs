//! Deterministic sparse zeta design-matrix projection.
//!
//! This is the compiler/router-side spectral path. It intentionally uses
//! floating point and allocation; the deployed transformerless runtime has a
//! separate integer-only implementation. The construction follows the
//! 16-window design matrix used by the original prime-router prototype:
//! sparse log-frequency windows, reduced QR bases, and covariance eigenvalues
//! computed from six temporal subwindows.

use std::sync::OnceLock;

use crate::zeta_zeros::ZETA_ZEROS;

pub const NUM_WINDOWS: usize = 16;
pub const TOTAL_CHANNELS: usize = 512;
pub const N_SAMPLES: usize = 257;
pub const SUBWINDOWS: usize = 6;

const X_MIN: f64 = 1.0e4;
const X_MAX: f64 = 1.0e6;
const RHO: f64 = 4.0;
const SPARSE_RADIUS: f64 = 0.3;

#[derive(Clone, Copy, Debug, Default)]
struct Complex {
    re: f64,
    im: f64,
}

impl Complex {
    fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    fn conj(self) -> Self {
        Self::new(self.re, -self.im)
    }

    fn norm_sq(self) -> f64 {
        self.re * self.re + self.im * self.im
    }
}

impl std::ops::Add for Complex {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.re + rhs.re, self.im + rhs.im)
    }
}

impl std::ops::AddAssign for Complex {
    fn add_assign(&mut self, rhs: Self) {
        self.re += rhs.re;
        self.im += rhs.im;
    }
}

impl std::ops::SubAssign for Complex {
    fn sub_assign(&mut self, rhs: Self) {
        self.re -= rhs.re;
        self.im -= rhs.im;
    }
}

impl std::ops::Mul for Complex {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(
            self.re * rhs.re - self.im * rhs.im,
            self.re * rhs.im + self.im * rhs.re,
        )
    }
}

impl std::ops::Mul<f64> for Complex {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.re * rhs, self.im * rhs)
    }
}

impl std::ops::DivAssign<f64> for Complex {
    fn div_assign(&mut self, rhs: f64) {
        self.re /= rhs;
        self.im /= rhs;
    }
}

struct ZetaWindow {
    start: usize,
    end: usize,
    q: Vec<Vec<Complex>>,
}

static WINDOWS: OnceLock<Vec<ZetaWindow>> = OnceLock::new();

/// The 16 window-center frequencies of the design matrix (#313): one
/// representative gamma per log-spaced window, computed with the same
/// center rule `windows()` uses. Design P (#276) consumes these as its
/// primary frequency set.
pub fn window_center_gammas() -> [f64; NUM_WINDOWS] {
    std::array::from_fn(|index| {
        let ratio = index as f64 / (NUM_WINDOWS - 1) as f64;
        let x_center = (X_MIN.ln() + ratio * (X_MAX.ln() - X_MIN.ln())).exp();
        let center_idx = ((x_center.ln() / X_MAX.ln()) * TOTAL_CHANNELS as f64) as usize;
        ZETA_ZEROS[center_idx.min(TOTAL_CHANNELS - 1)]
    })
}

fn windows() -> &'static [ZetaWindow] {
    WINDOWS
        .get_or_init(|| {
            (0..NUM_WINDOWS)
                .map(|index| {
                    let ratio = index as f64 / (NUM_WINDOWS - 1) as f64;
                    let x_center = (X_MIN.ln() + ratio * (X_MAX.ln() - X_MIN.ln())).exp();
                    let center_idx =
                        ((x_center.ln() / X_MAX.ln()) * TOTAL_CHANNELS as f64) as usize;
                    let radius = ((TOTAL_CHANNELS as f64 * SPARSE_RADIUS) as usize / 2).max(4);
                    let start = center_idx.saturating_sub(radius);
                    let end = (center_idx + radius).min(TOTAL_CHANNELS);
                    let h = RHO * x_center.sqrt();
                    let step = 2.0 * h / (N_SAMPLES - 1) as f64;
                    let xx: Vec<f64> = (0..N_SAMPLES)
                        .map(|sample| x_center - h + sample as f64 * step)
                        .collect();
                    let gammas = &ZETA_ZEROS[start..end];
                    let q = qr_basis(&xx, gammas);
                    ZetaWindow { start, end, q }
                })
                .collect()
        })
        .as_slice()
}

fn qr_basis(xx: &[f64], gammas: &[f64]) -> Vec<Vec<Complex>> {
    let mut q_columns: Vec<Vec<Complex>> = Vec::with_capacity(gammas.len());
    for &gamma in gammas {
        let mut column: Vec<Complex> = xx
            .iter()
            .map(|x| {
                let phase = x.ln() * gamma;
                Complex::new(phase.cos(), phase.sin())
            })
            .collect();

        for previous in &q_columns {
            let projection = previous
                .iter()
                .zip(&column)
                .fold(Complex::default(), |sum, (&q, &value)| {
                    sum + q.conj() * value
                });
            for (value, &q) in column.iter_mut().zip(previous) {
                *value -= q * projection;
            }
        }

        let norm = column
            .iter()
            .map(|value| value.norm_sq())
            .sum::<f64>()
            .sqrt();
        if norm > 1.0e-12 {
            for value in &mut column {
                *value /= norm;
            }
        }
        q_columns.push(column);
    }
    q_columns
}

fn centered_l2_normalize(values: &[f64]) -> Vec<f64> {
    if values.is_empty() {
        return Vec::new();
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let mut centered: Vec<f64> = values.iter().map(|value| value - mean).collect();
    let norm = centered
        .iter()
        .map(|value| value * value)
        .sum::<f64>()
        .sqrt();
    if norm > 1.0e-12 {
        for value in &mut centered {
            *value /= norm;
        }
    }
    centered
}

fn project_window(window: &ZetaWindow, state: &[f64]) -> Vec<f64> {
    let mut result = vec![0.0; N_SAMPLES];
    for (row, value) in result.iter_mut().enumerate() {
        for (column, q) in window.q.iter().enumerate() {
            *value += q[row].re * state[window.start + column];
        }
    }
    result
}

fn coefficients(window: &ZetaWindow, signal: &[f64]) -> Vec<f64> {
    window
        .q
        .iter()
        .map(|column| {
            column
                .iter()
                .zip(signal)
                .fold(Complex::default(), |sum, (&q, &value)| {
                    sum + q.conj() * Complex::new(value, 0.0)
                })
                .re
        })
        .collect()
}

fn covariance_eigenvalues(window: &ZetaWindow, raw_signal: &[f64]) -> Vec<f64> {
    let segment_len = N_SAMPLES / SUBWINDOWS;
    let mut coefficients_by_segment = Vec::with_capacity(SUBWINDOWS);
    for segment in 0..SUBWINDOWS {
        let start = segment * segment_len;
        let end = if segment + 1 == SUBWINDOWS {
            N_SAMPLES
        } else {
            (segment + 1) * segment_len
        };
        let normalized = centered_l2_normalize(&raw_signal[start..end]);
        let mut segment_coefficients = Vec::with_capacity(window.q.len());
        for column in &window.q {
            let coeff = column[start..end]
                .iter()
                .zip(&normalized)
                .fold(Complex::default(), |sum, (&q, &value)| {
                    sum + q.conj() * Complex::new(value, 0.0)
                });
            segment_coefficients.push(coeff.re);
        }
        coefficients_by_segment.push(segment_coefficients);
    }

    // The covariance is m×m, but it has rank at most six. Its six non-zero
    // eigenvalues are therefore the eigenvalues of this 6×6 Gram matrix.
    let mut gram = [[0.0; SUBWINDOWS]; SUBWINDOWS];
    for left in 0..SUBWINDOWS {
        for right in left..SUBWINDOWS {
            let value = coefficients_by_segment[left]
                .iter()
                .zip(&coefficients_by_segment[right])
                .map(|(a, b)| a * b)
                .sum::<f64>()
                / SUBWINDOWS as f64;
            gram[left][right] = value;
            gram[right][left] = value;
        }
    }

    let mut eigenvalues = symmetric_eigenvalues(gram);
    eigenvalues.sort_by(|left, right| right.total_cmp(left));
    eigenvalues
        .into_iter()
        .map(|value| value.max(0.0))
        .collect()
}

#[allow(clippy::needless_range_loop)]
fn symmetric_eigenvalues(mut matrix: [[f64; SUBWINDOWS]; SUBWINDOWS]) -> Vec<f64> {
    for _ in 0..64 {
        let mut p = 0;
        let mut q = 1;
        let mut largest = matrix[p][q].abs();
        for row in 0..SUBWINDOWS {
            for column in (row + 1)..SUBWINDOWS {
                if matrix[row][column].abs() > largest {
                    largest = matrix[row][column].abs();
                    p = row;
                    q = column;
                }
            }
        }
        if largest < 1.0e-12 {
            break;
        }

        let theta = (matrix[q][q] - matrix[p][p]) / (2.0 * matrix[p][q]);
        let t = if theta.abs() < 1.0e-12 {
            1.0
        } else {
            let sign = if theta >= 0.0 { 1.0 } else { -1.0 };
            sign / (theta.abs() + (theta * theta + 1.0).sqrt())
        };
        let c = 1.0 / (1.0 + t * t).sqrt();
        let s = t * c;

        for index in 0..SUBWINDOWS {
            let pivot_row = matrix[p][index];
            let other_row = matrix[q][index];
            matrix[p][index] = c * pivot_row - s * other_row;
            matrix[q][index] = s * pivot_row + c * other_row;
        }
        for index in 0..SUBWINDOWS {
            let pivot_column = matrix[index][p];
            let other_column = matrix[index][q];
            matrix[index][p] = c * pivot_column - s * other_column;
            matrix[index][q] = s * pivot_column + c * other_column;
        }
        matrix[p][q] = 0.0;
        matrix[q][p] = 0.0;
    }
    (0..SUBWINDOWS).map(|index| matrix[index][index]).collect()
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectionResult {
    pub window_index: usize,
    pub scores: Vec<f64>,
    pub active_range: (usize, usize),
    pub projected_state: Vec<f64>,
    pub eigenvalues: Vec<f64>,
}

/// Project a 512-dimensional real state into the 16 sparse zeta windows.
/// `window_biases` is optional and must contain one deterministic bias per
/// window; it is applied only after the QR projection norm is computed.
pub fn project_state(state: &[f64], window_biases: &[f64]) -> ProjectionResult {
    assert!(state.len() >= TOTAL_CHANNELS);
    assert!(window_biases.len() >= NUM_WINDOWS);

    let windows = windows();
    let mut scores = Vec::with_capacity(NUM_WINDOWS);
    let mut raw_signals = Vec::with_capacity(NUM_WINDOWS);
    let mut coefficients_by_window = Vec::with_capacity(NUM_WINDOWS);
    for (index, window) in windows.iter().enumerate() {
        let raw_signal = project_window(window, state);
        let normalized = centered_l2_normalize(&raw_signal);
        let coefficients = coefficients(window, &normalized);
        let norm = coefficients
            .iter()
            .map(|value| value * value)
            .sum::<f64>()
            .sqrt();
        scores.push(norm * (1.0 + window_biases[index]));
        raw_signals.push(raw_signal);
        coefficients_by_window.push(coefficients);
    }

    let (best_index, _) = scores
        .iter()
        .enumerate()
        .max_by(|left, right| left.1.total_cmp(right.1))
        .expect("the fixed window set is non-empty");
    let best = &windows[best_index];
    let mut projected_state = vec![0.0; TOTAL_CHANNELS];
    for (offset, value) in coefficients_by_window[best_index].iter().enumerate() {
        projected_state[best.start + offset] = value.abs();
    }

    let mut eigenvalues = covariance_eigenvalues(best, &raw_signals[best_index]);
    eigenvalues.resize(8, 0.0);
    eigenvalues.truncate(8);

    ProjectionResult {
        window_index: best_index + 1,
        scores,
        active_range: (best.start, best.end),
        projected_state,
        eigenvalues,
    }
}

/// Return the sparse channel range assigned to each of the 16 windows.
pub fn window_ranges() -> Vec<(usize, usize)> {
    windows()
        .iter()
        .map(|window| (window.start, window.end))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_is_deterministic_and_uses_all_windows() {
        let state: Vec<f64> = (0..TOTAL_CHANNELS)
            .map(|index| ((index as f64) * 0.037).sin())
            .collect();
        let biases = vec![0.0; NUM_WINDOWS];
        let first = project_state(&state, &biases);
        let second = project_state(&state, &biases);

        assert_eq!(first, second);
        assert_eq!(first.scores.len(), NUM_WINDOWS);
        assert_eq!(first.projected_state.len(), TOTAL_CHANNELS);
        assert!(first.eigenvalues.iter().any(|value| *value > 1.0e-8));
        assert!(first
            .eigenvalues
            .windows(2)
            .all(|pair| pair[0] + 1.0e-10 >= pair[1]));
    }

    #[test]
    fn distinct_state_directions_can_select_distinct_windows() {
        let biases = vec![0.0; NUM_WINDOWS];
        let mut first = vec![0.0; TOTAL_CHANNELS];
        let mut second = vec![0.0; TOTAL_CHANNELS];
        for index in 0..TOTAL_CHANNELS {
            first[index] = ((index as f64) * 0.013).sin();
            second[index] = ((index as f64) * 0.071 + 1.7).cos();
        }
        let first_result = project_state(&first, &biases);
        let second_result = project_state(&second, &biases);
        assert_ne!(first_result.scores, second_result.scores);
    }
}
