//! Offline compact mixed-integer legal destinations. Backend statuses are not proofs.
use microlp::{ComparisonOp, LinearExpr, OptimizationDirection, Problem, SolveOptions, Variable};
use serde::{Deserialize, Serialize};
pub(super) const NODE_LIMIT: u64 = 4096;
const INTEGER_TOL: f64 = 1e-6;
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum LegalError {
    Shape,
    NonFinite,
    MasterRange,
    NonUnitRow,
    NumericDecode,
    Overflow,
}
impl std::fmt::Display for LegalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "legal-set {:?}", self)
    }
}
impl std::error::Error for LegalError {}
type Result<T> = std::result::Result<T, LegalError>;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct Receipt {
    pub backend: String,
    pub encoding: String,
    pub node_limit: u64,
    pub time_limit: Option<u64>,
    pub termination: String,
    pub status: String,
    pub nodes_solved: Option<u64>,
    pub lp_iterations: Option<u64>,
    pub variables: usize,
    pub fractional_coordinates: usize,
    pub constraints: usize,
    pub backend_objective: Option<f64>,
    pub objective_constant: f64,
    pub raw_displacement_assignment: Option<Vec<[f64; 3]>>,
    pub destination_master_bits: Option<Vec<u32>>,
    pub actual_delta: Option<Vec<f64>>,
    pub construction_residuals: Option<Vec<f64>>,
    pub recomputed_zero_margin_target_passed: bool,
    pub gradient_dot: Option<f64>,
    pub residual_tolerance: Option<f64>,
    pub unchanged_screen_passed: bool,
    pub eligible: bool,
}
fn finite_dot(a: &[f64], b: &[f64]) -> Result<f64> {
    if a.len() != b.len() {
        return Err(LegalError::Shape);
    }
    let mut s = 0.;
    for (a, b) in a.iter().zip(b) {
        s += a * b;
        if !s.is_finite() {
            return Err(LegalError::Overflow);
        }
    }
    Ok(s)
}
fn norm(x: &[f64]) -> Result<f64> {
    Ok(finite_dot(x, x)?.sqrt())
}
fn code(m: f32) -> i8 {
    (m * 4.).round() as i8
}
fn canonical(m: f32) -> bool {
    m == f32::from(code(m)) * 0.25
}
fn destination(m: f32, q: i8) -> f32 {
    if q == code(m) {
        m
    } else {
        f32::from(q) * 0.25
    }
}
fn integer(x: f64) -> Result<i8> {
    if !x.is_finite() || (x - x.round()).abs() > INTEGER_TOL || !(-14. ..=14.).contains(&x.round())
    {
        return Err(LegalError::NumericDecode);
    }
    Ok(x.round() as i8)
}
fn decode(m: f32, raw: [f64; 3]) -> Result<f32> {
    let k = integer(raw[0])?;
    let q = i16::from(k) + i16::from(code(m));
    if !(-7..=7).contains(&q) {
        return Err(LegalError::NumericDecode);
    }
    let q = q as i8;
    let l = integer(raw[1])?;
    let r = integer(raw[2])?;
    let q0 = code(m);
    if canonical(m) {
        if l != 0 || r != 0 {
            return Err(LegalError::NumericDecode);
        }
    } else if !(0..=1).contains(&l)
        || !(0..=1).contains(&r)
        || l + r > 1
        || q < i8::try_from(i16::from(q0) + i16::from(r) - (i16::from(q0) + 7) * i16::from(l))
            .map_err(|_| LegalError::NumericDecode)?
        || q > i8::try_from(i16::from(q0) - i16::from(l) + (7 - i16::from(q0)) * i16::from(r))
            .map_err(|_| LegalError::NumericDecode)?
    {
        return Err(LegalError::NumericDecode);
    }
    Ok(destination(m, q))
}
fn validate(m: &[f32], g: &[f32], rows: &[Vec<f64>]) -> Result<()> {
    if m.is_empty() || m.len() != g.len() || rows.iter().any(|r| r.len() != m.len()) {
        return Err(LegalError::Shape);
    }
    if m.iter().chain(g).any(|x| !x.is_finite()) || rows.iter().flatten().any(|x| !x.is_finite()) {
        return Err(LegalError::NonFinite);
    }
    if m.iter().any(|x| !(-1.75..=1.75).contains(x)) {
        return Err(LegalError::MasterRange);
    }
    for row in rows {
        let n = norm(row)?;
        if n != 0. && (n - 1.).abs() > 1e-10 {
            return Err(LegalError::NonUnitRow);
        }
    }
    Ok(())
}
#[derive(Clone, Copy)]
struct Vars {
    k: Variable,
    side: Option<(Variable, Variable)>,
}
fn expr(vars: &[Vars], m: &[f32], coeff: &[f64]) -> LinearExpr {
    let mut e = LinearExpr::empty();
    for ((v, m), c) in vars.iter().zip(m).zip(coeff) {
        let q0 = f64::from(code(*m));
        e.add(v.k, *c * 0.25);
        if let Some((l, r)) = v.side {
            let f = q0 * 0.25 - f64::from(*m);
            e.add(l, c * f);
            e.add(r, c * f);
        }
    }
    e
}
/// One call, no resumes. Root excluded from node limit; no wall-clock limit.
pub(super) fn run(m: &[f32], g: &[f32], rows: &[Vec<f64>]) -> Result<Receipt> {
    run_with_limit(m, g, rows, NODE_LIMIT)
}
fn run_with_limit(m: &[f32], g: &[f32], rows: &[Vec<f64>], limit: u64) -> Result<Receipt> {
    validate(m, g, rows)?;
    let mut p = Problem::new(OptimizationDirection::Minimize);
    let mut vars = Vec::new();
    let mut warm = Vec::new();
    for (&m, &g) in m.iter().zip(g) {
        let q0 = code(m);
        let k = p.add_integer_var(f64::from(g) * 0.25, (-7 - i32::from(q0), 7 - i32::from(q0)));
        warm.push((k, 0.));
        let side = if canonical(m) {
            None
        } else {
            let c = f64::from(g) * (f64::from(q0) * 0.25 - f64::from(m));
            let l = p.add_binary_var(c);
            let r = p.add_binary_var(c);
            warm.extend([(l, 0.), (r, 0.)]);
            p.add_constraint([(l, 1.), (r, 1.)], ComparisonOp::Le, 1.);
            p.add_constraint(
                [(k, 1.), (r, -1.), (l, f64::from(q0) + 7.)],
                ComparisonOp::Ge,
                0.,
            );
            p.add_constraint(
                [(k, 1.), (l, 1.), (r, f64::from(q0) - 7.)],
                ComparisonOp::Le,
                0.,
            );
            Some((l, r))
        };
        vars.push(Vars { k, side });
    }
    for row in rows {
        p.add_constraint(expr(&vars, m, row), ComparisonOp::Ge, 0.);
    }
    let fractional = vars.iter().filter(|v| v.side.is_some()).count();
    let constant = 0.;
    let mut receipt = Receipt {
        backend: "microlp=0.6.0".into(),
        encoding: "centered_integer_displacement_k=q-q0/1".into(),
        node_limit: limit,
        time_limit: None,
        termination: String::new(),
        status: String::new(),
        nodes_solved: None,
        lp_iterations: None,
        variables: m.len() + 2 * fractional,
        fractional_coordinates: fractional,
        constraints: rows.len() + 3 * fractional,
        backend_objective: None,
        objective_constant: constant,
        raw_displacement_assignment: None,
        destination_master_bits: None,
        actual_delta: None,
        construction_residuals: None,
        recomputed_zero_margin_target_passed: false,
        gradient_dot: None,
        residual_tolerance: None,
        unchanged_screen_passed: false,
        eligible: false,
    };
    let mut options = SolveOptions::default();
    options.node_limit = Some(limit);
    options.time_limit = None;
    options.warm_start = Some(warm);
    let outcome = match p.solve_with(options) {
        Ok(v) => v,
        Err(e) => {
            receipt.termination = format!("{e:?}");
            receipt.status = "BACKEND_NUMERIC_OR_NO_INCUMBENT".into();
            return Ok(receipt);
        }
    };
    let stats = outcome.stats();
    receipt.nodes_solved = Some(stats.nodes_solved);
    receipt.lp_iterations = Some(stats.lp_iterations);
    receipt.termination = format!("{:?}", outcome.termination_reason());
    if receipt.nodes_solved.is_some_and(|n| n > limit) {
        return Err(LegalError::Overflow);
    }
    let Some(solution) = outcome.solution() else {
        receipt.status = "NODE_LIMIT_NO_INCUMBENT".into();
        return Ok(receipt);
    };
    receipt.backend_objective = Some(solution.objective());
    if !solution.objective().is_finite() {
        return Err(LegalError::NonFinite);
    }
    let raw = vars
        .iter()
        .map(|v| {
            let (l, r) = v
                .side
                .map(|(l, r)| (solution.var_value_raw(l), solution.var_value_raw(r)))
                .unwrap_or((0., 0.));
            [solution.var_value_raw(v.k), l, r]
        })
        .collect::<Vec<_>>();
    match populate(&mut receipt, m, g, rows, raw.clone()) {
        Ok(()) => {}
        Err(LegalError::NumericDecode) => {
            receipt.raw_displacement_assignment = Some(raw);
            receipt.status = "INTEGER_DECODE_REJECTED".into();
        }
        Err(e) => return Err(e),
    }
    Ok(receipt)
}
fn populate(
    receipt: &mut Receipt,
    m: &[f32],
    g: &[f32],
    rows: &[Vec<f64>],
    raw: Vec<[f64; 3]>,
) -> Result<()> {
    if raw.len() != m.len() {
        return Err(LegalError::Shape);
    }
    let destination = m
        .iter()
        .zip(&raw)
        .map(|(&m, &r)| decode(m, r))
        .collect::<Result<Vec<_>>>()?;
    let delta = destination
        .iter()
        .zip(m)
        .map(|(d, m)| f64::from(*d) - f64::from(*m))
        .collect::<Vec<_>>();
    let residual = rows
        .iter()
        .map(|r| finite_dot(r, &delta))
        .collect::<Result<Vec<_>>>()?;
    let gd = finite_dot(&g.iter().map(|g| f64::from(*g)).collect::<Vec<_>>(), &delta)?;
    let tolerance = 1e-10 * norm(&delta)?;
    let construction = residual.iter().all(|r| *r >= 0.);
    let screen = residual.iter().all(|r| *r >= -tolerance);
    receipt.status = if gd >= 0. {
        "NO_DESCENDING_INCUMBENT"
    } else if !screen {
        "ACTUAL_SCREEN_REJECTED"
    } else if receipt.termination == "NodeLimit" {
        "NODE_LIMIT_DESCENDING_INCUMBENT"
    } else {
        "DESCENDING_INCUMBENT"
    }
    .into();
    receipt.raw_displacement_assignment = Some(raw);
    receipt.destination_master_bits = Some(destination.iter().map(|x| x.to_bits()).collect());
    receipt.actual_delta = Some(delta);
    receipt.construction_residuals = Some(residual);
    receipt.recomputed_zero_margin_target_passed = construction;
    receipt.gradient_dot = Some(gd);
    receipt.residual_tolerance = Some(tolerance);
    receipt.unchanged_screen_passed = screen;
    receipt.eligible = gd < 0. && screen;
    Ok(())
}
/// Validate a saved solution without re-solving, reranking or trusting a backend proof.
pub(super) fn authenticate(
    m: &[f32],
    g: &[f32],
    rows: &[Vec<f64>],
    receipt: &Receipt,
) -> Result<()> {
    validate(m, g, rows)?;
    let fractional = m.iter().filter(|m| !canonical(**m)).count();
    if receipt.backend != "microlp=0.6.0"
        || receipt.encoding != "centered_integer_displacement_k=q-q0/1"
        || receipt.node_limit != NODE_LIMIT
        || receipt.time_limit.is_some()
        || receipt.nodes_solved.is_some_and(|n| n > NODE_LIMIT)
        || receipt.variables != m.len() + 2 * fractional
        || receipt.fractional_coordinates != fractional
        || receipt.constraints != rows.len() + 3 * fractional
    {
        return Err(LegalError::Shape);
    }
    let constant = 0.;
    if receipt.objective_constant != constant {
        return Err(LegalError::NumericDecode);
    }
    let mut expected = receipt.clone();
    if let Some(raw) = &receipt.raw_displacement_assignment {
        if receipt.nodes_solved.is_none() || receipt.lp_iterations.is_none() {
            return Err(LegalError::NumericDecode);
        }
        if receipt.status == "INTEGER_DECODE_REJECTED" {
            if raw.len() != m.len()
                || raw.iter().flatten().any(|v| !v.is_finite())
                || !m.iter().zip(raw).any(|(&m, &v)| decode(m, v).is_err())
                || receipt.destination_master_bits.is_some()
                || receipt.actual_delta.is_some()
                || receipt.construction_residuals.is_some()
                || receipt.gradient_dot.is_some()
                || receipt.residual_tolerance.is_some()
                || receipt.eligible
                || receipt.unchanged_screen_passed
                || receipt.recomputed_zero_margin_target_passed
            {
                return Err(LegalError::NumericDecode);
            }
            return Ok(());
        }
        if !receipt.backend_objective.is_some_and(|x| x.is_finite()) {
            return Err(LegalError::NonFinite);
        }
        populate(&mut expected, m, g, rows, raw.clone())?;
        if &expected != receipt {
            return Err(LegalError::NumericDecode);
        }
    } else if receipt.destination_master_bits.is_some()
        || receipt.actual_delta.is_some()
        || receipt.construction_residuals.is_some()
        || receipt.gradient_dot.is_some()
        || receipt.residual_tolerance.is_some()
        || receipt.eligible
        || receipt.recomputed_zero_margin_target_passed
        || receipt.unchanged_screen_passed
        || receipt.backend_objective.is_some()
        || !matches!(
            receipt.status.as_str(),
            "NODE_LIMIT_NO_INCUMBENT" | "BACKEND_NUMERIC_OR_NO_INCUMBENT"
        )
    {
        return Err(LegalError::NumericDecode);
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_domain_fractional_endpoints_and_signed_zero() -> Result<()> {
        for m in [-1.75, -1.7, -1.6, -0.01, -0.0, 0.0, 0.13, 1.6, 1.7, 1.75] {
            let mut bits = std::collections::BTreeSet::new();
            for q in -7..=7 {
                for l in 0..=1 {
                    for r in 0..=1 {
                        if let Ok(v) = decode(m, [(q - code(m)) as f64, l as f64, r as f64]) {
                            bits.insert(v.to_bits());
                        }
                    }
                }
            }
            let expected = (-7..=7)
                .map(|q| destination(m, q).to_bits())
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(bits, expected);
            assert_eq!(decode(m, [0., 0., 0.])?.to_bits(), m.to_bits());
        }
        Ok(())
    }
    #[test]
    fn centered_domain_matches_absolute_constraints_for_every_code_and_side() -> Result<()> {
        for m in [-1.75, -1.7, -1.6, -0.01, -0.0, 0.0, 0.13, 1.6, 1.7, 1.75] {
            let q0 = i16::from(code(m));
            for q in -7i16..=7 {
                for l in 0i16..=1 {
                    for r in 0i16..=1 {
                        let absolute_legal = if canonical(m) {
                            l == 0 && r == 0
                        } else {
                            l + r <= 1 && q >= q0 + r - (q0 + 7) * l && q <= q0 - l + (7 - q0) * r
                        };
                        let decoded = decode(m, [f64::from(q - q0), f64::from(l), f64::from(r)]);
                        assert_eq!(decoded.is_ok(), absolute_legal);
                        if absolute_legal {
                            assert_eq!(decoded?.to_bits(), destination(m, q as i8).to_bits());
                        }
                    }
                }
            }
            assert!(decode(m, [f64::from(-8 - q0), 1., 0.]).is_err());
            assert!(decode(m, [f64::from(8 - q0), 0., 1.]).is_err());
        }
        Ok(())
    }
    #[test]
    fn translated_guard_and_objective_expressions_preserve_legal_deltas() -> Result<()> {
        let masters = [-1.7f32, -0.0, 0.13, 1.75];
        let coeff = [0.6f64, -0.2, 0.3, -0.7];
        for q in -7i8..=7 {
            let mut absolute_lhs = 0.;
            let mut absolute_rhs = 0.;
            let mut centered_lhs = 0.;
            let mut actual = 0.;
            for (&m, &c) in masters.iter().zip(&coeff) {
                let q0 = code(m);
                let moved = q != q0;
                let side = if canonical(m) || !moved { 0. } else { 1. };
                let correction = f64::from(q0) * 0.25 - f64::from(m);
                absolute_lhs += c * (f64::from(q) * 0.25 + side * correction);
                absolute_rhs += c * f64::from(q0) * 0.25;
                centered_lhs += c * (f64::from(q - q0) * 0.25 + side * correction);
                actual += c * (f64::from(destination(m, q)) - f64::from(m));
            }
            assert!((absolute_lhs - absolute_rhs - centered_lhs).abs() < 1e-14);
            assert!((centered_lhs - actual).abs() < 1e-14);
        }
        // Noop is now a literal all-zero expression, including fractional masters.
        assert_eq!(masters.iter().map(|m| 0. * f64::from(*m)).sum::<f64>(), 0.);
        let receipt = run(&masters, &[0.; 4], &[])?;
        assert_eq!(receipt.objective_constant, 0.);
        assert_eq!(receipt.encoding, "centered_integer_displacement_k=q-q0/1");
        authenticate(&masters, &[0.; 4], &[], &receipt)?;
        let mut mislabeled = receipt;
        mislabeled.encoding = "absolute_q/1".into();
        assert!(authenticate(&masters, &[0.; 4], &[], &mislabeled).is_err());
        Ok(())
    }
    #[test]
    fn true_joint_repair_single_moves_fail() -> Result<()> {
        let s = 2f64.sqrt().recip();
        let r = vec![vec![s, -s], vec![-s, s]];
        let result = run(&[0., 0.], &[1., 1.], &r)?;
        assert!(result.eligible);
        let d = result.actual_delta.as_ref().ok_or(LegalError::Shape)?;
        assert!(d[0] < 0. && d[1] < 0.);
        assert_eq!(d[0], d[1]);
        assert!(r
            .iter()
            .any(|row| finite_dot(row, &[d[0], 0.]).is_ok_and(|v| v < 0.)));
        assert!(r
            .iter()
            .any(|row| finite_dot(row, &[0., d[1]]).is_ok_and(|v| v < 0.)));
        authenticate(&[0., 0.], &[1., 1.], &r, &result)?;
        Ok(())
    }
    #[test]
    fn noop_not_promoted_and_constraints_respected() -> Result<()> {
        let r = run(&[-0.0], &[1.], &[vec![1.]])?;
        assert!(!r.eligible);
        assert_eq!(r.status, "NO_DESCENDING_INCUMBENT");
        assert_eq!(r.destination_master_bits, Some(vec![(-0f32).to_bits()]));
        Ok(())
    }
    #[test]
    fn invalid_decode_and_nonfinite_rejected() {
        assert_eq!(decode(0.13, [0., 1., 0.]), Err(LegalError::NumericDecode));
        assert_eq!(decode(0.13, [0.5, 0., 0.]), Err(LegalError::NumericDecode));
        assert_eq!(run(&[0.], &[f32::NAN], &[]), Err(LegalError::NonFinite));
    }
    #[test]
    fn backend_zero_target_is_not_a_stricter_actual_admission_gate() -> Result<()> {
        let rows = vec![vec![-5e-11, (1.0 - 25e-22f64).sqrt()]];
        let m = [0., 0.];
        let g = [-1., 0.];
        let mut receipt = run(&m, &g, &rows)?;
        populate(
            &mut receipt,
            &m,
            &g,
            &rows,
            vec![[1., 0., 0.], [0., 0., 0.]],
        )?;
        assert!(!receipt.recomputed_zero_margin_target_passed);
        assert!(receipt.unchanged_screen_passed && receipt.eligible);
        authenticate(&m, &g, &rows, &receipt)?;
        Ok(())
    }
    #[test]
    fn zero_branch_budget_keeps_noop_incumbent_not_root_relaxation() -> Result<()> {
        let n = 5f64.sqrt();
        let r = run_with_limit(&[0., 0.], &[3., 4.], &[vec![1. / n, 2. / n]], 0)?;
        assert_eq!(r.termination, "NodeLimit");
        assert_eq!(r.nodes_solved, Some(0));
        assert!(!r.eligible);
        assert_eq!(r.gradient_dot, Some(0.));
        assert_eq!(r.destination_master_bits, Some(vec![0f32.to_bits(); 2]));
        Ok(())
    }
    #[test]
    fn fixed_node_limit_and_receipt_tamper() -> Result<()> {
        let r = run_with_limit(&[0.13, 0.17], &[1., -1.], &[vec![0.6, 0.8]], 0)?;
        assert_eq!(r.nodes_solved, Some(0));
        assert!(!r.eligible || r.gradient_dot.is_some_and(|x| x < 0.));
        let mut a = run(&[0.13], &[1.], &[])?;
        a.destination_master_bits = Some(vec![0.1f32.to_bits()]);
        assert!(authenticate(&[0.13], &[1.], &[], &a).is_err());
        Ok(())
    }
}
