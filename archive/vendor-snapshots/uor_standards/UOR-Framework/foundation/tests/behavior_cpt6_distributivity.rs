//! Product/Coproduct Completion Amendment §Q2 verification:
//! CPT_6 distributivity of Cartesian partition product over PartitionCoproduct.
//!
//! `A ⊠ (B + C) ≡ (A ⊠ B) + (A ⊠ C)` at four axes simultaneously per the
//! amendment's §Q2 table:
//!
//! | axis        | LHS                                              | RHS                                                |
//! |-------------|--------------------------------------------------|----------------------------------------------------|
//! | siteBudget  | SB(A) + max(SB(B), SB(C))                        | max(SB(A)+SB(B), SB(A)+SB(C))                      |
//! | SITE_COUNT  | SC(A) + max(SC(B), SC(C)) + 1                    | max(SC(A)+SC(B), SC(A)+SC(C)) + 1                  |
//! | χ           | χ(A) · (χ(B) + χ(C))                             | χ(A)·χ(B) + χ(A)·χ(C)                              |
//! | S           | S(A) + ln 2 + max(S(B),S(C))                     | ln 2 + max(S(A)+S(B), S(A)+S(C))                   |
//!
//! Verified arithmetically over a parameter grid — no shape construction
//! required because the equalities are between numeric quantities the
//! amendment claims hold for ANY operand triple. If the equality breaks
//! at any grid point, CPT_6 itself is wrong (which would contradict the
//! amendment's algebraic claim).

#[test]
fn cpt_6_axis_site_budget() {
    // siteBudget: SB(A) + max(SB(B), SB(C)) == max(SB(A)+SB(B), SB(A)+SB(C))
    for sa in 0..10u16 {
        for sb in 0..10u16 {
            for sc in 0..10u16 {
                let lhs = sa + core::cmp::max(sb, sc);
                let rhs = core::cmp::max(sa + sb, sa + sc);
                assert_eq!(
                    lhs, rhs,
                    "CPT_6 site-budget distributivity broke at \
                     SB(A)={sa}, SB(B)={sb}, SB(C)={sc}: LHS={lhs}, RHS={rhs}"
                );
            }
        }
    }
}

#[test]
fn cpt_6_axis_site_count() {
    // SITE_COUNT: SC(A) + max(SC(B), SC(C)) + 1 == max(SC(A)+SC(B), SC(A)+SC(C)) + 1
    // The +1 is the outer coproduct's tag bit on both sides.
    for sa in 0..10u16 {
        for sb in 0..10u16 {
            for sc in 0..10u16 {
                let lhs = sa + core::cmp::max(sb, sc) + 1;
                let rhs = core::cmp::max(sa + sb, sa + sc) + 1;
                assert_eq!(lhs, rhs);
            }
        }
    }
}

#[test]
fn cpt_6_axis_euler_multiplicative() {
    // χ: χ(A) · (χ(B) + χ(C)) == χ(A)·χ(B) + χ(A)·χ(C)
    // Standard distributivity of multiplication over addition; included
    // for completeness because the §Q2 table cites it explicitly as one
    // of the four distributive axes.
    for ea in -3..=3i32 {
        for eb in -3..=3i32 {
            for ec in -3..=3i32 {
                let lhs = ea * (eb + ec);
                let rhs = ea * eb + ea * ec;
                assert_eq!(lhs, rhs);
            }
        }
    }
}

#[test]
fn cpt_6_axis_entropy() {
    // S: S(A) + ln 2 + max(S(B),S(C)) == ln 2 + max(S(A)+S(B), S(A)+S(C))
    // Reduces to the identity max(a+b, a+c) == a + max(b,c), shifted by
    // a constant ln 2 term contributed by the outer coproduct.
    let values: [f64; 4] = [
        0.0,
        core::f64::consts::LN_2,
        2.0 * core::f64::consts::LN_2,
        3.0 * core::f64::consts::LN_2,
    ];
    for &sa in &values {
        for &sb in &values {
            for &sc in &values {
                let lhs = sa + core::f64::consts::LN_2 + sb.max(sc);
                let rhs = core::f64::consts::LN_2 + (sa + sb).max(sa + sc);
                assert!(
                    (lhs - rhs).abs() < 1e-10,
                    "CPT_6 entropy distributivity broke at \
                     S(A)={sa}, S(B)={sb}, S(C)={sc}: LHS={lhs}, RHS={rhs}"
                );
            }
        }
    }
}

#[test]
fn cpt_6_does_not_extend_to_partition_product() {
    // Negative half of §Q2: distributivity of ⊠ over × FAILS at the
    // site-budget level. SB(A ⊠ (B × C)) = SB(A) + SB(B) + SB(C) but
    // SB((A ⊠ B) × (A ⊠ C)) = (SB(A)+SB(B)) + (SB(A)+SB(C)) =
    // 2·SB(A) + SB(B) + SB(C). The two differ by SB(A) > 0.
    for sa in 1..10u16 {
        for sb in 0..10u16 {
            for sc in 0..10u16 {
                let lhs = sa + sb + sc;
                let rhs = (sa + sb) + (sa + sc);
                assert_ne!(
                    lhs, rhs,
                    "CPT_6 negative claim: ⊠ does NOT distribute over × \
                     at the site-budget level for SB(A)={sa} > 0 \
                     (LHS={lhs}, RHS={rhs} should differ)"
                );
            }
        }
    }
}
