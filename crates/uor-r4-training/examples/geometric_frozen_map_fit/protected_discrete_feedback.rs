//! Fixed offline residual-feedback quantization. No native scorer or adaptive search.
use std::fmt;
pub const OUTER_ROUNDS: usize = 32;
pub const INNER_PASSES: usize = 32;
pub const RELATIVE_TOLERANCE: f64 = 1e-10;
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeedbackError {
    Shape,
    NonFinite,
    MasterRange,
    NonUnitRow,
    Overflow,
}
impl fmt::Display for FeedbackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "protected discrete feedback: {:?}", self)
    }
}
impl std::error::Error for FeedbackError {}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackStatus {
    Completed,
    ZeroDirection,
    NonDescendingDirection,
    UnprotectedDirection,
}
#[derive(Debug, Clone)]
pub struct FeedbackRound {
    pub index: usize,
    pub destination_bits: Vec<u32>,
    pub actual_delta: Vec<f64>,
    pub residuals: Vec<f64>,
    pub tolerance: f64,
    pub gradient_dot: f64,
    pub eligible: bool,
    pub feedback_norm: f64,
    pub primal_norm: f64,
    pub duplicate_of: Option<usize>,
}
#[derive(Debug, Clone)]
pub struct FeedbackRun {
    pub status: FeedbackStatus,
    pub target: Vec<f64>,
    pub rounds: Vec<FeedbackRound>,
}
fn finite(x: f64) -> Result<f64, FeedbackError> {
    if x.is_finite() {
        Ok(x)
    } else {
        Err(FeedbackError::Overflow)
    }
}
fn dot(a: &[f64], b: &[f64]) -> Result<f64, FeedbackError> {
    if a.len() != b.len() {
        return Err(FeedbackError::Shape);
    }
    let mut s = 0.;
    for (&x, &y) in a.iter().zip(b) {
        s = finite(s + finite(x * y)?)?;
    }
    Ok(s)
}
fn norm(a: &[f64]) -> Result<f64, FeedbackError> {
    finite(dot(a, a)?.sqrt())
}
fn screen(rows: &[Vec<f64>], x: &[f64]) -> Result<(Vec<f64>, f64, bool), FeedbackError> {
    let tolerance = finite(RELATIVE_TOLERANCE * norm(x)?)?;
    let r = rows
        .iter()
        .map(|row| dot(row, x))
        .collect::<Result<Vec<_>, _>>()?;
    let pass = r.iter().all(|v| *v >= -tolerance);
    Ok((r, tolerance, pass))
}
fn code(m: f32) -> i8 {
    (m * 4.).round() as i8
}
fn destination(m: f32, q: i8) -> f32 {
    if code(m) == q {
        m
    } else {
        f32::from(q) * 0.25
    }
}
/// Both squared terms have coefficient1/2 (rho=1). Noop wins exact ties,
/// then smallest signed code. Original fractional bits are legal only at noop.
fn discrete(m: f32, target: f64, z: f64, u: f64) -> Result<f32, FeedbackError> {
    let mut best: Option<(f64, bool, i8, f32)> = None;
    for q in -7i8..=7 {
        let v = destination(m, q);
        let x = f64::from(v) - f64::from(m);
        let a = finite(x - target)?;
        let b = finite(finite(x - z)? + u)?;
        let cost = finite(0.5 * finite(a * a)? + 0.5 * finite(b * b)?)?;
        let noop = q == code(m);
        if best.as_ref().is_none_or(|(old, on, oq, _)| {
            cost < *old || (cost == *old && ((noop && !on) || (noop == *on && q < *oq)))
        }) {
            best = Some((cost, noop, q, v));
        }
    }
    best.map(|(_, _, _, v)| v).ok_or(FeedbackError::Shape)
}
/// Generic dimensions permit focused fixtures; production caller owns1920/380
/// population authentication. Rows must already be L2-normalized or exactly0.
pub fn run(
    master: &[f32],
    gradient: &[f32],
    normalized_rows: &[Vec<f64>],
    direction: &[f64],
) -> Result<FeedbackRun, FeedbackError> {
    let n = master.len();
    if n == 0
        || gradient.len() != n
        || direction.len() != n
        || normalized_rows.iter().any(|r| r.len() != n)
    {
        return Err(FeedbackError::Shape);
    }
    if master.iter().chain(gradient).any(|x| !x.is_finite())
        || direction
            .iter()
            .chain(normalized_rows.iter().flatten())
            .any(|x| !x.is_finite())
    {
        return Err(FeedbackError::NonFinite);
    }
    if master.iter().any(|x| !(-1.75..=1.75).contains(x)) {
        return Err(FeedbackError::MasterRange);
    }
    for row in normalized_rows {
        let length = norm(row)?;
        if length != 0. && (length - 1.).abs() > RELATIVE_TOLERANCE {
            return Err(FeedbackError::NonUnitRow);
        }
        if length == 0. && row.iter().any(|x| *x != 0.) {
            return Err(FeedbackError::NonUnitRow);
        }
    }
    let g = gradient.iter().map(|x| f64::from(*x)).collect::<Vec<_>>();
    let empty = |status| FeedbackRun {
        status,
        target: Vec::new(),
        rounds: Vec::new(),
    };
    let max = direction.iter().map(|x| x.abs()).fold(0., f64::max);
    if max == 0. {
        return Ok(empty(FeedbackStatus::ZeroDirection));
    }
    if dot(&g, direction)? >= 0. {
        return Ok(empty(FeedbackStatus::NonDescendingDirection));
    }
    if !screen(normalized_rows, direction)?.2 {
        return Ok(empty(FeedbackStatus::UnprotectedDirection));
    }
    let scale = finite(0.25 / max)?;
    let target = direction
        .iter()
        .map(|x| finite(scale * x))
        .collect::<Result<Vec<_>, _>>()?;
    let mut z = target.clone();
    let mut u = vec![0.; n];
    let mut rounds: Vec<FeedbackRound> = Vec::with_capacity(OUTER_ROUNDS);
    for index in 0..OUTER_ROUNDS {
        let dest = master
            .iter()
            .zip(&target)
            .zip(&z)
            .zip(&u)
            .map(|(((&m, &t), &z), &u)| discrete(m, t, z, u))
            .collect::<Result<Vec<_>, _>>()?;
        let x = dest
            .iter()
            .zip(master)
            .map(|(&v, &m)| f64::from(v) - f64::from(m))
            .collect::<Vec<_>>();
        let destination_bits = dest.iter().map(|v| v.to_bits()).collect::<Vec<_>>();
        let duplicate_of = rounds
            .iter()
            .position(|r| r.destination_bits == destination_bits);
        let (residuals, tolerance, protected) = screen(normalized_rows, &x)?;
        let gradient_dot = dot(&g, &x)?;
        z = x
            .iter()
            .zip(&u)
            .map(|(&x, &u)| finite(x + u))
            .collect::<Result<Vec<_>, _>>()?;
        for _ in 0..INNER_PASSES {
            for row in normalized_rows {
                let v = dot(row, &z)?;
                if v < 0. {
                    for (value, &j) in z.iter_mut().zip(row) {
                        *value = finite(*value - finite(v * j)?)?;
                    }
                }
            }
        }
        let primal = x
            .iter()
            .zip(&z)
            .map(|(&x, &z)| finite(x - z))
            .collect::<Result<Vec<_>, _>>()?;
        for (u, &r) in u.iter_mut().zip(&primal) {
            *u = finite(*u + r)?;
        }
        rounds.push(FeedbackRound {
            index,
            destination_bits,
            actual_delta: x,
            residuals,
            tolerance,
            gradient_dot,
            eligible: protected && gradient_dot < 0.,
            feedback_norm: norm(&u)?,
            primal_norm: norm(&primal)?,
            duplicate_of,
        });
    }
    Ok(FeedbackRun {
        status: FeedbackStatus::Completed,
        target,
        rounds,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ties_noop_fractional_and_signed_zero() -> Result<(), FeedbackError> {
        assert_eq!(code(0.125), 1);
        assert_eq!(code(-0.125), -1);
        assert_eq!(discrete(-0.0, 0., 0., 0.)?.to_bits(), (-0.0f32).to_bits());
        assert_eq!(discrete(0.12, 0., 0., 0.)?.to_bits(), 0.12f32.to_bits());
        // Midpoint cost ties: noop precedes the smaller signed code.
        assert_eq!(discrete(0., -0.125, -0.125, 0.)?.to_bits(), 0f32.to_bits());
        assert_eq!(discrete(1.75, 10., 10., 0.)?, 1.75);
        Ok(())
    }
    #[test]
    fn coupled_feedback_recovers_rounding_conflict() -> Result<(), FeedbackError> {
        // Feasible d=(1,.6); independent nearest target(.25,.15) at
        // original second master.124 yields delta(.25,.126), violating row.
        let row = vec![-0.59, 1.];
        let n = norm(&row)?;
        let rows = vec![row.iter().map(|x| x / n).collect()];
        let r = run(&[0., 0.124], &[-1., -0.6], &rows, &[1., 0.6])?;
        assert!(!r.rounds[0].eligible);
        assert!(r.rounds.iter().any(|r| r.eligible));
        for x in &r.rounds {
            if x.eligible {
                assert!(x.gradient_dot < 0. && x.residuals[0] >= -x.tolerance);
            }
        }
        Ok(())
    }
    #[test]
    fn fixed_rounds_duplicates_and_descent() -> Result<(), FeedbackError> {
        let r = run(&[0.], &[-1.], &[vec![0.]], &[1.])?;
        assert_eq!(r.rounds.len(), 32);
        assert!(r.rounds[0].eligible);
        assert_eq!(r.rounds[1].duplicate_of, Some(0));
        assert_eq!(r.rounds[0].destination_bits, vec![0.25f32.to_bits()]);
        assert_eq!(
            run(&[0.], &[1.], &[], &[1.])?.status,
            FeedbackStatus::NonDescendingDirection
        );
        assert_eq!(
            run(&[-0.], &[0.], &[], &[0.])?.status,
            FeedbackStatus::ZeroDirection
        );
        assert_eq!(
            run(&[0.], &[-1.], &[vec![-1.]], &[1.])?.status,
            FeedbackStatus::UnprotectedDirection
        );
        Ok(())
    }
    #[test]
    fn malformed_nonfinite_overflow() {
        assert_eq!(run(&[], &[], &[], &[]).unwrap_err(), FeedbackError::Shape);
        assert_eq!(
            run(&[2.], &[-1.], &[], &[1.]).unwrap_err(),
            FeedbackError::MasterRange
        );
        assert_eq!(
            run(&[0.], &[f32::NAN], &[], &[1.]).unwrap_err(),
            FeedbackError::NonFinite
        );
        assert_eq!(
            run(&[0.], &[-1.], &[vec![2.]], &[1.]).unwrap_err(),
            FeedbackError::NonUnitRow
        );
        assert_eq!(
            run(&[0.], &[-1.], &[], &[f64::MAX]).unwrap_err(),
            FeedbackError::Overflow
        );
        assert_eq!(
            run(&[0.], &[-1.], &[vec![]], &[1.]).unwrap_err(),
            FeedbackError::Shape
        );
    }
}
