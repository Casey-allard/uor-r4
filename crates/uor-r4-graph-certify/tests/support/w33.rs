//! The normative W(3,3) reference object for #845 — the symplectic generalized
//! quadrangle over GF(3) — exactly as frozen in
//! `docs/w33_geometry_qualification_spec_845.md` §2–§3.
//!
//! Reference mathematics (Definition scope): nothing here asserts semantic
//! usefulness, activation, or empirical superiority. Certifier-instrument /
//! off-serving-path; ordinary integer arithmetic is in scope here (the P-4
//! projection in the spec concerns a possible later lowering, not this code).
#![allow(dead_code)]

/// A vector of GF(3)^4 with entries in {0, 1, 2}.
pub type Vec4 = [u8; 4];

/// The number of projective points of PG(3,3), all of which are W(3) points.
pub const POINTS: usize = 40;

/// The pinned non-degenerate alternating form:
/// `<u, v> = u0*v1 - u1*v0 + u2*v3 - u3*v2  (mod 3)`.
pub fn symplectic(u: &Vec4, v: &Vec4) -> u8 {
    let a = u32::from(u[0]) * u32::from(v[1]) + 2 * u32::from(u[1]) * u32::from(v[0]);
    let b = u32::from(u[2]) * u32::from(v[3]) + 2 * u32::from(u[3]) * u32::from(v[2]);
    ((a + b) % 3) as u8
}

/// Scale a vector by a GF(3) scalar.
fn scale(u: &Vec4, s: u8) -> Vec4 {
    [
        (u[0] * s) % 3,
        (u[1] * s) % 3,
        (u[2] * s) % 3,
        (u[3] * s) % 3,
    ]
}

/// The canonical representative of a nonzero vector's projective class: the
/// unique scalar multiple whose first nonzero coordinate is 1.
pub fn canonical(u: &Vec4) -> Vec4 {
    let first = u.iter().copied().find(|c| *c != 0).unwrap_or(0);
    match first {
        2 => scale(u, 2),
        _ => *u,
    }
}

/// The 40 canonical representatives in lexicographic order — the pinned point
/// indexing every table below uses.
pub fn points() -> Vec<Vec4> {
    let mut out = Vec::with_capacity(POINTS);
    for code in 1u16..81 {
        let u = [
            (code / 27 % 3) as u8,
            (code / 9 % 3) as u8,
            (code / 3 % 3) as u8,
            (code % 3) as u8,
        ];
        if canonical(&u) == u {
            out.push(u);
        }
    }
    out.sort();
    out
}

/// Collinearity distance `d_W` in {0, 1, 2}: 0 iff equal, 1 iff distinct and
/// collinear (form value 0), 2 otherwise. Total with diameter 2 (the
/// collinearity graph is strongly regular (40, 12, 2, 4); asserted by test).
pub fn distance(points: &[Vec4], p: usize, q: usize) -> u8 {
    if p == q {
        return 0;
    }
    if symplectic(&points[p], &points[q]) == 0 {
        1
    } else {
        2
    }
}

/// Phase `phi(p, q) = <u_p, u_q>` on canonical representatives, in GF(3).
/// Zero iff equal or collinear; antisymmetric mod 3. Convention-dependent by
/// design — the adversarial phase permutation exercises exactly that.
pub fn phase(points: &[Vec4], p: usize, q: usize) -> u8 {
    symplectic(&points[p], &points[q])
}

/// The 40 x 40 `d_W` table in the pinned point order.
pub fn distance_table(points: &[Vec4]) -> Vec<[u8; POINTS]> {
    (0..POINTS)
        .map(|p| {
            let mut row = [0u8; POINTS];
            for (q, slot) in row.iter_mut().enumerate() {
                *slot = distance(points, p, q);
            }
            row
        })
        .collect()
}

/// The 40 x 40 phase table in the pinned point order.
pub fn phase_table(points: &[Vec4]) -> Vec<[u8; POINTS]> {
    (0..POINTS)
        .map(|p| {
            let mut row = [0u8; POINTS];
            for (q, slot) in row.iter_mut().enumerate() {
                *slot = phase(points, p, q);
            }
            row
        })
        .collect()
}

/// The totally isotropic projective lines, each as a sorted quadruple of point
/// indices, sorted and deduplicated — the 40 lines of W(3) (asserted by test).
pub fn lines(points: &[Vec4]) -> Vec<[usize; 4]> {
    let index_of = |v: &Vec4| -> usize { points.iter().position(|p| p == v).unwrap() };
    let mut out = Vec::new();
    for p in 0..POINTS {
        for q in (p + 1)..POINTS {
            if symplectic(&points[p], &points[q]) != 0 {
                continue;
            }
            // The projective line through p and q: u, v, u+v, u+2v.
            let (u, v) = (points[p], points[q]);
            let sum = |a: &Vec4, b: &Vec4| -> Vec4 {
                [
                    (a[0] + b[0]) % 3,
                    (a[1] + b[1]) % 3,
                    (a[2] + b[2]) % 3,
                    (a[3] + b[3]) % 3,
                ]
            };
            let w1 = canonical(&sum(&u, &v));
            let w2 = canonical(&sum(&u, &scale(&v, 2)));
            let mut line = [p, q, index_of(&w1), index_of(&w2)];
            line.sort_unstable();
            out.push(line);
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// The state mapping mu (spec §3): digitize each slot mod 9 into two base-3
/// digits, read the four digits as a GF(3)^4 vector, take its projective
/// class; the zero vector maps to the basepoint [1:0:0:0]. Total on all i16
/// slot pairs; reads only slot values (label-independent by construction).
pub fn map_state(points: &[Vec4], s0: i16, s1: i16) -> usize {
    let digits = |s: i16| -> (u8, u8) {
        let a = s.rem_euclid(9) as u8;
        (a % 3, a / 3)
    };
    let (d00, d01) = digits(s0);
    let (d10, d11) = digits(s1);
    let v: Vec4 = [d00, d01, d10, d11];
    let target = if v == [0, 0, 0, 0] {
        [1, 0, 0, 0]
    } else {
        canonical(&v)
    };
    points.iter().position(|p| *p == target).unwrap()
}

/// The pinned symplectomorphism for the isomorphic-relabel control:
/// (u0, u1, u2, u3) -> (u2, u3, u0, u1), which preserves the pinned form and
/// therefore collinearity (asserted by test), while moving points.
pub fn relabel_point(u: &Vec4) -> Vec4 {
    canonical(&[u[2], u[3], u[0], u[1]])
}

/// The point-index permutation the relabel control conjugates tables by.
pub fn relabel_permutation(points: &[Vec4]) -> Vec<usize> {
    points
        .iter()
        .map(|u| {
            let image = relabel_point(u);
            points.iter().position(|p| *p == image).unwrap()
        })
        .collect()
}

/// The pinned adversarial phase permutation: swap the nonzero phase values
/// (1 <-> 2), keep 0. Preserves the collinearity relation encoded by phase
/// zero while destroying the pinned sign convention.
pub fn permute_phase(value: u8) -> u8 {
    match value {
        1 => 2,
        2 => 1,
        other => other,
    }
}
