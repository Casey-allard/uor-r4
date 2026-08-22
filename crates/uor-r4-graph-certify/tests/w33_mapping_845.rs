//! #845 increment 2 — the normative W(3,3) object and the state mapping mu,
//! verified against the frozen Definitions of
//! `docs/w33_geometry_qualification_spec_845.md` §2–§3.
//!
//! Reference mathematics checks (Definition scope): these tests verify that
//! the constructed object *is* the symplectic generalized quadrangle of order
//! (3,3) and that mu has the frozen totality/surjectivity/periodicity and
//! label-independence properties. Nothing here is a semantic-usefulness or
//! empirical-superiority claim; those are #845 increments 3–4.

mod support;

use support::w33;

#[test]
fn forty_points_in_canonical_lex_order() {
    let points = w33::points();
    assert_eq!(points.len(), w33::POINTS);
    let mut sorted = points.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted, points, "points are distinct and lex-ordered");
    for u in &points {
        assert_eq!(w33::canonical(u), *u, "every representative is canonical");
        assert_eq!(
            u.iter().copied().find(|c| *c != 0),
            Some(1),
            "first nonzero coordinate is 1"
        );
    }
}

#[test]
fn the_collinearity_graph_is_srg_40_12_2_4() {
    let points = w33::points();
    let collinear = |p: usize, q: usize| w33::distance(&points, p, q) == 1;
    for p in 0..w33::POINTS {
        let degree = (0..w33::POINTS).filter(|q| collinear(p, *q)).count();
        assert_eq!(degree, 12, "every point is collinear with exactly 12");
    }
    for p in 0..w33::POINTS {
        for q in (p + 1)..w33::POINTS {
            let common = (0..w33::POINTS)
                .filter(|r| collinear(p, *r) && collinear(q, *r))
                .count();
            let expected = if collinear(p, q) { 2 } else { 4 };
            assert_eq!(common, expected, "SRG lambda/mu at pair ({p}, {q})");
        }
    }
}

#[test]
fn forty_totally_isotropic_lines_four_by_four() {
    let points = w33::points();
    let lines = w33::lines(&points);
    assert_eq!(lines.len(), 40, "W(3) has 40 lines");
    let mut per_point = [0usize; w33::POINTS];
    for line in &lines {
        let mut distinct = line.to_vec();
        distinct.sort_unstable();
        distinct.dedup();
        assert_eq!(distinct.len(), 4, "every line carries 4 distinct points");
        for a in 0..4 {
            per_point[line[a]] += 1;
            for b in (a + 1)..4 {
                assert_eq!(
                    w33::distance(&points, line[a], line[b]),
                    1,
                    "line points are pairwise collinear (total isotropy)"
                );
            }
        }
    }
    for (point, count) in per_point.iter().enumerate() {
        assert_eq!(*count, 4, "point {point} lies on exactly 4 lines");
    }
}

#[test]
fn the_generalized_quadrangle_axiom_holds() {
    let points = w33::points();
    let lines = w33::lines(&points);
    for p in 0..w33::POINTS {
        for line in &lines {
            if line.contains(&p) {
                continue;
            }
            let collinear_on_line = line
                .iter()
                .filter(|q| w33::distance(&points, p, **q) == 1)
                .count();
            assert_eq!(
                collinear_on_line, 1,
                "a point off a line is collinear with exactly one of its points"
            );
        }
    }
}

#[test]
fn distance_and_phase_tables_match_their_definitions() {
    let points = w33::points();
    let distance = w33::distance_table(&points);
    let phase = w33::phase_table(&points);
    for (p, (distance_row, phase_row)) in distance.iter().zip(phase.iter()).enumerate() {
        for q in 0..w33::POINTS {
            assert_eq!(distance_row[q], w33::distance(&points, p, q));
            assert_eq!(phase_row[q], w33::phase(&points, p, q));
            assert!(distance_row[q] <= 2, "d_W is total with diameter 2");
            assert_eq!(
                distance_row[q],
                w33::distance(&points, q, p),
                "d_W is symmetric"
            );
            assert_eq!((distance_row[q] == 0), (p == q), "d_W is zero iff equal");
            assert_eq!(
                phase_row[q],
                (3 - w33::phase(&points, q, p)) % 3,
                "phi is antisymmetric mod 3"
            );
            assert_eq!(
                (phase_row[q] == 0),
                (distance_row[q] <= 1),
                "phi is zero exactly on the collinear-or-equal pairs"
            );
        }
    }
}

#[test]
fn mu_is_total_surjective_nine_periodic_and_reads_only_residues() {
    let points = w33::points();
    let mut seen = [false; w33::POINTS];
    for s0 in -20i16..=20 {
        for s1 in -20i16..=20 {
            let point = w33::map_state(&points, s0, s1);
            assert!(point < w33::POINTS, "mu is total");
            seen[point] = true;
            assert_eq!(
                point,
                w33::map_state(&points, s0.rem_euclid(9), s1.rem_euclid(9)),
                "mu is a pure function of the slot residues mod 9"
            );
            assert_eq!(
                point,
                w33::map_state(&points, s0.wrapping_add(9), s1),
                "mu is 9-periodic in slot 0"
            );
            assert_eq!(
                point,
                w33::map_state(&points, s0, s1.wrapping_add(9)),
                "mu is 9-periodic in slot 1"
            );
        }
    }
    assert!(seen.iter().all(|hit| *hit), "mu is onto the 40 points");
}

#[test]
fn the_readout_fibers_partition_the_residue_classes() {
    let points = w33::points();
    let basepoint = w33::map_state(&points, 0, 0);
    assert_eq!(
        points[basepoint],
        [1, 0, 0, 0],
        "zero maps to the basepoint"
    );
    let mut fiber_sizes = [0usize; w33::POINTS];
    for a0 in 0i16..9 {
        for a1 in 0i16..9 {
            fiber_sizes[w33::map_state(&points, a0, a1)] += 1;
        }
    }
    assert_eq!(fiber_sizes.iter().sum::<usize>(), 81, "fibers partition");
    for (point, size) in fiber_sizes.iter().enumerate() {
        let expected = if point == basepoint { 3 } else { 2 };
        assert_eq!(
            *size, expected,
            "point {point}: two residue classes per point, three at the basepoint"
        );
    }
}

#[test]
fn the_relabel_control_is_a_nonidentity_collinearity_automorphism() {
    let points = w33::points();
    let sigma = w33::relabel_permutation(&points);
    let mut sorted = sigma.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, (0..w33::POINTS).collect::<Vec<_>>(), "a bijection");
    assert!(
        sigma.iter().enumerate().any(|(p, image)| p != *image),
        "the relabel moves at least one point"
    );
    for p in 0..w33::POINTS {
        for q in 0..w33::POINTS {
            assert_eq!(
                w33::distance(&points, sigma[p], sigma[q]),
                w33::distance(&points, p, q),
                "the relabel preserves d_W"
            );
        }
    }
}

#[test]
fn the_phase_permutation_is_adversarial_but_metric_safe() {
    let points = w33::points();
    assert_eq!(w33::permute_phase(0), 0, "zero (collinearity) is kept");
    assert_eq!(w33::permute_phase(1), 2);
    assert_eq!(w33::permute_phase(2), 1);
    let mut changed = 0usize;
    for p in 0..w33::POINTS {
        for q in 0..w33::POINTS {
            let original = w33::phase(&points, p, q);
            let permuted = w33::permute_phase(original);
            assert_eq!(
                (permuted == 0),
                (w33::distance(&points, p, q) <= 1),
                "the permuted phase keeps the collinearity pattern"
            );
            if permuted != original {
                changed += 1;
            }
        }
    }
    assert!(changed > 0, "the permutation actually changes the table");
}
