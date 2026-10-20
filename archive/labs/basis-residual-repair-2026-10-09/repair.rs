//! Offline LU numerical admission. This does not certify conditioning, forward error,
//! sparse eta updates, or global feasibility. The simplex EPS remains unchanged.
use crate::{
    lu::{LUFactors, ScratchSpace},
    sparse::{Error, ScatteredVec},
};
use std::cell::{Cell, RefCell};
thread_local! { static FINITE: Cell<bool> = const { Cell::new(true) }; }
pub(crate) fn observed(v: f64) -> f64 {
    if !v.is_finite() {
        FINITE.with(|s| s.set(false));
    }
    v
}
fn reset_finite() {
    FINITE.with(|s| s.set(true));
}
fn finite() -> bool {
    FINITE.with(Cell::get)
}
#[derive(Clone, Debug)]
pub struct Verification {
    pub dimension: usize,
    pub rhs_checked_per_direction: usize,
    pub max_normal_error: f64,
    pub max_transpose_error: f64,
    pub max_sparse_normal_error: f64,
    pub max_sparse_transpose_error: f64,
    pub worst_normal_rhs: usize,
    pub worst_transpose_rhs: usize,
    pub worst_sparse_normal_rhs: usize,
    pub worst_sparse_transpose_rhs: usize,
    pub tolerance: f64,
}
thread_local! { static LAST: RefCell<Option<Verification>> = const { RefCell::new(None) }; }
pub fn last_verification() -> Option<Verification> {
    LAST.with(|s| s.borrow().clone())
}
fn checked(v: f64) -> Result<f64, Error> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(Error::NonFiniteFactorization)
    }
}
pub(crate) fn pivot_threshold(n: usize, original: f64, residual: f64) -> Result<f64, Error> {
    checked(original)?;
    checked(residual)?;
    if original < 0.0 || residual < 0.0 {
        return Err(Error::NonFiniteFactorization);
    }
    let sigma = original.max(residual);
    if sigma == 0.0 {
        return Err(Error::SingularMatrix);
    }
    checked(32.0 * (n as f64) * f64::EPSILON * sigma)
}
fn norm_error<'a>(
    n: usize,
    get_col: &impl Fn(usize) -> (&'a [usize], &'a [f64]),
    x: &[f64],
    rhs: usize,
    transpose: bool,
    norm: f64,
) -> Result<f64, Error> {
    let mut ax = vec![0.0; n];
    for c in 0..n {
        let (rows, vs) = get_col(c);
        for (&r, &v) in rows.iter().zip(vs) {
            let (out, xi) = if transpose { (c, x[r]) } else { (r, x[c]) };
            let p = checked(v * xi)?;
            ax[out] = checked(ax[out] + p)?;
        }
    }
    let mut residual: f64 = 0.0;
    let mut xn: f64 = 0.0;
    for i in 0..n {
        xn = xn.max(checked(x[i])?.abs());
        residual = residual.max(checked((if i == rhs { 1.0 } else { 0.0 }) - ax[i])?.abs());
    }
    let denominator = checked(checked(norm * xn)? + 1.0)?;
    if denominator == 0.0 {
        if residual == 0.0 {
            Ok(0.0)
        } else {
            Err(Error::UnreliableFactorization)
        }
    } else {
        checked(residual / denominator)
    }
}
pub(crate) fn verify<'a>(
    n: usize,
    get_col: &impl Fn(usize) -> (&'a [usize], &'a [f64]),
    factors: &LUFactors,
) -> Result<Verification, Error> {
    let mut row_sums = vec![0.0; n];
    let mut col_sums = vec![0.0; n];
    for c in 0..n {
        let (rs, vs) = get_col(c);
        if rs.len() != vs.len() {
            return Err(Error::NonFiniteFactorization);
        }
        for (&r, &v) in rs.iter().zip(vs) {
            if r >= n {
                return Err(Error::NonFiniteFactorization);
            }
            let a = checked(v)?.abs();
            row_sums[r] = checked(row_sums[r] + a)?;
            col_sums[c] = checked(col_sums[c] + a)?;
        }
    }
    let norms = [
        row_sums.into_iter().fold(0.0, f64::max),
        col_sums.into_iter().fold(0.0, f64::max),
    ];
    let tolerance = 128.0 * (n as f64) * f64::EPSILON;
    let mut report = Verification {
        dimension: n,
        rhs_checked_per_direction: n,
        max_normal_error: 0.0,
        max_transpose_error: 0.0,
        max_sparse_normal_error: 0.0,
        max_sparse_transpose_error: 0.0,
        worst_normal_rhs: 0,
        worst_transpose_rhs: 0,
        worst_sparse_normal_rhs: 0,
        worst_sparse_transpose_rhs: 0,
        tolerance,
    };
    let trans = factors.transpose();
    let mut scratch = ScratchSpace::with_capacity(n);
    for (direction, f) in [factors, &trans].into_iter().enumerate() {
        for rhs in 0..n {
            for sparse in [false, true] {
                reset_finite();
                let mut x = vec![0.0; n];
                x[rhs] = 1.0;
                if sparse {
                    let mut s = ScatteredVec::empty(n);
                    s.set([(rhs, &1.0)]);
                    f.solve(&mut s, &mut scratch);
                    x.fill(0.0);
                    for (i, v) in s.iter() {
                        x[i] = *v;
                    }
                } else {
                    f.solve_dense(&mut x, &mut scratch);
                }
                if !finite() {
                    return Err(Error::NonFiniteFactorization);
                }
                let error = norm_error(n, get_col, &x, rhs, direction == 1, norms[direction])?;
                let worst = match (direction, sparse) {
                    (0, false) => &mut report.worst_normal_rhs,
                    (0, true) => &mut report.worst_sparse_normal_rhs,
                    (1, false) => &mut report.worst_transpose_rhs,
                    _ => &mut report.worst_sparse_transpose_rhs,
                };
                let max = match (direction, sparse) {
                    (0, false) => &mut report.max_normal_error,
                    (0, true) => &mut report.max_sparse_normal_error,
                    (1, false) => &mut report.max_transpose_error,
                    _ => &mut report.max_sparse_transpose_error,
                };
                if error > *max {
                    *max = error;
                    *worst = rhs;
                }
                if error > tolerance {
                    crate::diagnostics::repair_event(format!("residual_rejection direction={direction} sparse={sparse} rhs={rhs} error_bits={:016x} tolerance_bits={:016x}",error.to_bits(),tolerance.to_bits()));
                    return Err(Error::UnreliableFactorization);
                }
            }
        }
    }
    crate::diagnostics::repair_event(format!("verification {:?}", report));
    LAST.with(|s| *s.borrow_mut() = Some(report.clone()));
    Ok(report)
}
/// Qualify supplied original CSC columns using the very same factorizer and admission
/// used for every fresh constructor/reset basis; no solver or eta update is replayed.
pub fn qualify_saved_basis(columns: &[Vec<(usize, f64)>]) -> Result<Verification, String> {
    LAST.with(|s| *s.borrow_mut() = None);
    let n = columns.len();
    if n == 0 {
        return Err("empty basis".into());
    }
    let mut rs = Vec::with_capacity(n);
    let mut vs = Vec::with_capacity(n);
    for col in columns {
        let mut prev = None;
        let mut r = Vec::new();
        let mut v = Vec::new();
        for &(i, x) in col {
            if i >= n || !x.is_finite() || prev.is_some_and(|p| p >= i) {
                return Err("invalid CSC row order/domain/value".into());
            }
            prev = Some(i);
            r.push(i);
            v.push(x);
        }
        rs.push(r);
        vs.push(v);
    }
    let mut scratch = ScratchSpace::with_capacity(n);
    crate::lu::lu_factorize(n, |c| (&rs[c], &vs[c]), 0.1, &mut scratch)
        .map_err(|e| e.to_string())?;
    last_verification().ok_or_else(|| "verification receipt missing".into())
}
/// Focused offline fixture: genuine factors of identity must fail certification
/// against a different original matrix. No solver proposal is formed.
#[doc(hidden)]
pub fn unacceptable_factor_fixture() -> Result<(), String> {
    let rows = [vec![0], vec![1]];
    let values = [vec![1.0], vec![1.0]];
    let mut scratch = ScratchSpace::with_capacity(2);
    let factors = crate::lu::lu_factorize(2, |c| (&rows[c], &values[c]), 0.1, &mut scratch)
        .map_err(|e| e.to_string())?;
    let changed = [vec![2.0], vec![1.0]];
    match verify(2, &|c| (&rows[c], &changed[c]), &factors) {
        Err(Error::UnreliableFactorization) => Ok(()),
        other => Err(format!(
            "incorrect factor verification accepted or wrong rejection: {other:?}"
        )),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nonsingular_small_pivot_and_rescaling() {
        for scale in [1.0, 1e-90, 1e90] {
            let cols = vec![vec![(0, scale)], vec![(1, scale * 1e-11)]];
            let r = qualify_saved_basis(&cols).unwrap();
            assert_eq!(r.rhs_checked_per_direction, 2);
            assert_eq!(r.max_sparse_normal_error, 0.0);
        }
    }
    #[test]
    fn singular_and_nonfinite_are_rejected() {
        assert!(qualify_saved_basis(&[vec![(0, 1.0)], vec![(0, 1.0)]]).is_err());
        assert!(qualify_saved_basis(&[vec![(0, f64::INFINITY)]]).is_err());
    }
    #[test]
    fn unacceptable_factors_fail_against_original_basis() {
        let rows = [vec![0], vec![1]];
        let values = [vec![1.0], vec![1.0]];
        let mut s = ScratchSpace::with_capacity(2);
        let f = crate::lu::lu_factorize(2, |c| (&rows[c], &values[c]), 0.1, &mut s).unwrap();
        let bad = [vec![2.0], vec![1.0]];
        assert_eq!(
            verify(2, &|c| (&rows[c], &bad[c]), &f).unwrap_err(),
            Error::UnreliableFactorization
        );
    }
    #[test]
    fn no_absolute_floor_and_selected_threshold() {
        assert!(pivot_threshold(440, 1e-100, 1e-100).unwrap() < 1e-110);
        assert!(pivot_threshold(2, 0.0, 0.0).is_err());
        assert!(pivot_threshold(2, f64::NAN, 1.0).is_err());
    }
}
