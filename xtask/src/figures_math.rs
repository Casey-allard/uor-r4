//! `cargo xtask figures-math [--write] [--out DIR]`: figures of real geometry,
//! with computed coordinates.
//!
//! Five figures under `docs/figures/` show actual geometry, so none of their
//! points is placed by hand: the E8 roots and the 600-cell in their Coxeter
//! planes, the flock read on the 600-cell, the SpiralCore ternary tree beside
//! its 16-root orbit, and the routed read. Root systems are generated from
//! their coordinates. Simple roots are found by matching the Coxeter diagram's
//! Gram matrix, the Coxeter element is the product of the simple reflections,
//! and the drawing plane is that element's eigenplane for exp(2πi/h).
//!
//! The orbit data (the 16 roots of the anchor orbit, the ternary ports and the
//! 15 generator signed permutations) are copied from the tables that
//! `crates/uor-r4-discrete/build.rs` generates (`ROOTS`, `BRANCH` and rows 1 to
//! 15 of `AXES`). The tests re-derive the orbit and the ports from the
//! generators, so a wrong copy fails. Illustrative choices, such as which
//! neighbours are kept or the route's query vector, are labelled as such on
//! the figure.
//!
//! f64 is fine here: this is offline documentation tooling, not model compute.
//! Every printed coordinate is rounded to two decimals, so the output is
//! deterministic. Without `--write` the task checks that the files are current.
//! The theme (CSS variables with a dark override, Okabe–Ito hues, text sizes)
//! follows the figure style guide used for the reorganized docs.

use std::f64::consts::PI;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use crate::figures::{esc, text_width};

type V<const N: usize> = [f64; N];
type M<const N: usize> = [[f64; N]; N];

const EPS: f64 = 1e-9;

fn phi() -> f64 {
    (1.0 + 5f64.sqrt()) / 2.0
}

// ---------------------------------------------------------------- linear algebra

fn dot<const N: usize>(a: &V<N>, b: &V<N>) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

fn ident<const N: usize>() -> M<N> {
    let mut m = [[0.0; N]; N];
    for (i, row) in m.iter_mut().enumerate() {
        row[i] = 1.0;
    }
    m
}

/// The product `a · b` of two small square matrices.
fn compose<const N: usize>(a: &M<N>, b: &M<N>) -> M<N> {
    let mut m = [[0.0; N]; N];
    for i in 0..N {
        for j in 0..N {
            m[i][j] = (0..N).map(|k| a[i][k] * b[k][j]).sum();
        }
    }
    m
}

fn apply<const N: usize>(m: &M<N>, v: &V<N>) -> V<N> {
    let mut out = [0.0; N];
    for (o, row) in out.iter_mut().zip(m) {
        *o = dot(row, v);
    }
    out
}

/// The reflection in the hyperplane orthogonal to `s`.
fn reflection<const N: usize>(s: &V<N>) -> M<N> {
    let n = dot(s, s);
    let mut m = ident::<N>();
    for i in 0..N {
        for j in 0..N {
            m[i][j] -= 2.0 * s[i] * s[j] / n;
        }
    }
    m
}

fn near<const N: usize>(a: &M<N>, b: &M<N>) -> bool {
    (0..N).all(|i| (0..N).all(|j| (a[i][j] - b[i][j]).abs() < 1e-7))
}

/// Roots whose Gram matrix is `gram`: a simple system of the Coxeter type that
/// `gram` encodes, found by depth-first search over the root list.
fn simple_roots<const N: usize>(roots: &[V<N>], gram: &M<N>) -> Result<[V<N>; N], String> {
    fn go<const N: usize>(roots: &[V<N>], gram: &M<N>, chosen: &mut Vec<usize>) -> bool {
        let j = chosen.len();
        if j == N {
            return true;
        }
        for (k, r) in roots.iter().enumerate() {
            if (dot(r, r) - gram[j][j]).abs() > EPS {
                continue;
            }
            let fits = chosen
                .iter()
                .enumerate()
                .all(|(i, &c)| (dot(&roots[c], r) - gram[i][j]).abs() < EPS);
            if fits {
                chosen.push(k);
                if go(roots, gram, chosen) {
                    return true;
                }
                chosen.pop();
            }
        }
        false
    }
    let mut chosen = Vec::with_capacity(N);
    if !go(roots, gram, &mut chosen) {
        return Err("no set of roots has the Coxeter diagram's Gram matrix".into());
    }
    let mut out = [[0.0; N]; N];
    for (o, &c) in out.iter_mut().zip(&chosen) {
        *o = roots[c];
    }
    Ok(out)
}

/// The Coxeter element (product of the simple reflections) of order `h`, and an
/// orthonormal basis of its eigenplane for exp(2πi/h), oriented so the element
/// turns `b1` towards `b2` by 2π/h. `others` lists the remaining exponents up
/// to h/2: the plane is the image of the product of their quadratic factors.
#[allow(clippy::needless_range_loop)] // column access by index
fn coxeter_plane<const N: usize>(
    simple: &[V<N>; N],
    h: u32,
    others: &[u32],
) -> Result<(M<N>, V<N>, V<N>), String> {
    let mut c = ident::<N>();
    for s in simple {
        c = compose(&c, &reflection(s));
    }
    let mut p = ident::<N>();
    for k in 1..=h {
        p = compose(&p, &c);
        if near(&p, &ident()) != (k == h) {
            return Err(format!(
                "Coxeter element does not have order {h} (power {k})"
            ));
        }
    }
    let c2 = compose(&c, &c);
    let mut kill = ident::<N>();
    for &e in others {
        let t = 2.0 * (2.0 * PI * f64::from(e) / f64::from(h)).cos();
        let mut f = c2;
        for i in 0..N {
            for j in 0..N {
                f[i][j] -= t * c[i][j];
            }
            f[i][i] += 1.0;
        }
        kill = compose(&kill, &f);
    }
    let scale = (0..N)
        .map(|j| (0..N).map(|i| kill[i][j] * kill[i][j]).sum::<f64>().sqrt())
        .fold(0.0, f64::max);
    let mut basis: Vec<V<N>> = Vec::new();
    for j in 0..N {
        let mut w = [0.0; N];
        for (i, x) in w.iter_mut().enumerate() {
            *x = kill[i][j];
        }
        for b in &basis {
            let d = dot(&w, b);
            for (x, y) in w.iter_mut().zip(b) {
                *x -= d * y;
            }
        }
        let n = dot(&w, &w).sqrt();
        if n > 1e-6 * scale {
            basis.push(w.map(|x| x / n));
        }
    }
    if basis.len() != 2 {
        return Err(format!("eigenplane has dimension {}, not 2", basis.len()));
    }
    let (b1, mut b2) = (basis[0], basis[1]);
    let cb = apply(&c, &b1);
    let angle = dot(&cb, &b2).atan2(dot(&cb, &b1));
    if (angle.abs() - 2.0 * PI / f64::from(h)).abs() > 1e-7 {
        return Err(format!(
            "Coxeter element turns its plane by {angle}, not 2π/{h}"
        ));
    }
    if angle < 0.0 {
        b2 = b2.map(|x| -x);
    }
    Ok((c, b1, b2))
}

fn project<const N: usize>(v: &V<N>, (b1, b2): (&V<N>, &V<N>)) -> (f64, f64) {
    (dot(v, b1), dot(v, b2))
}

/// Points grouped by distance from the origin: (radius, member indexes), inner first.
/// A float sort key rounded to 1e-9, so values equal in exact arithmetic but
/// apart in their last bits (platform `sin`/`cos`/`atan2` differ there) tie,
/// and the tie-breaker decides the same way on every platform.
fn key(x: f64) -> i64 {
    (x * 1e9).round() as i64
}

fn rings(xy: &[(f64, f64)]) -> Vec<(f64, Vec<usize>)> {
    let mut order: Vec<usize> = (0..xy.len()).collect();
    let r = |i: usize| xy[i].0.hypot(xy[i].1);
    order.sort_by(|&a, &b| r(a).total_cmp(&r(b)).then(a.cmp(&b)));
    let mut out: Vec<(f64, Vec<usize>)> = Vec::new();
    for i in order {
        match out.last_mut() {
            Some((rad, members)) if (r(i) - *rad).abs() < 1e-6 => members.push(i),
            _ => out.push((r(i), vec![i])),
        }
    }
    for (_, members) in &mut out {
        members.sort_unstable();
    }
    out
}

// ---------------------------------------------------------------- E8, standard

/// The 240 roots of E8: 112 of (±1, ±1, 0⁶) and 128 of (±½)⁸ with an even
/// number of minus signs.
fn e8_standard() -> Vec<V<8>> {
    let mut out = Vec::with_capacity(240);
    for i in 0..8 {
        for j in i + 1..8 {
            for (si, sj) in [(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)] {
                let mut v = [0.0; 8];
                v[i] = si;
                v[j] = sj;
                out.push(v);
            }
        }
    }
    for mask in 0u32..256 {
        if mask.count_ones() % 2 == 0 {
            let mut v = [0.5; 8];
            for (k, x) in v.iter_mut().enumerate() {
                if mask >> k & 1 == 1 {
                    *x = -0.5;
                }
            }
            out.push(v);
        }
    }
    out
}

/// E8's Gram matrix (Bourbaki order: 1–3–4–5–6–7–8 with 2 on 4), roots of norm² 2.
fn e8_gram() -> M<8> {
    let mut g = [[0.0; 8]; 8];
    for (i, row) in g.iter_mut().enumerate() {
        row[i] = 2.0;
    }
    for (a, b) in [(0, 2), (2, 3), (3, 4), (4, 5), (5, 6), (6, 7), (1, 3)] {
        g[a][b] = -1.0;
        g[b][a] = -1.0;
    }
    g
}

/// E8 exponents other than 1 and 29, up to h/2.
const E8_OTHERS: [u32; 3] = [7, 11, 13];
/// H4 exponents other than 1 and 29, up to h/2.
const H4_OTHERS: [u32; 1] = [11];

// ---------------------------------------------------------------- icosians

/// An element (a + b·φ)/2 of ½·Z[φ], exact.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Zp {
    a: i64,
    b: i64,
}

impl Zp {
    fn real(self) -> f64 {
        (self.a as f64 + self.b as f64 * phi()) / 2.0
    }
    /// The Galois conjugate (φ ↦ 1 − φ).
    fn conjugate(self) -> f64 {
        ((self.a + self.b) as f64 - self.b as f64 * phi()) / 2.0
    }
    fn times_phi(self) -> Zp {
        Zp {
            a: self.b,
            b: self.a + self.b,
        }
    }
    fn neg(self) -> Zp {
        Zp {
            a: -self.a,
            b: -self.b,
        }
    }
}

fn even_permutations() -> Vec<[usize; 4]> {
    let mut out = Vec::new();
    for a in 0..4 {
        for b in 0..4 {
            for c in 0..4 {
                for d in 0..4 {
                    let p = [a, b, c, d];
                    let distinct = (0..4).all(|i| (i + 1..4).all(|j| p[i] != p[j]));
                    let inversions = (0..4)
                        .flat_map(|i| (i + 1..4).map(move |j| (i, j)))
                        .filter(|&(i, j)| p[i] > p[j])
                        .count();
                    if distinct && inversions % 2 == 0 {
                        out.push(p);
                    }
                }
            }
        }
    }
    out
}

/// The 120 unit icosians, exact, real part first: 8 of (±1, 0, 0, 0), 16 of
/// (±½)⁴ and 96 even permutations of (±φ/2, ±½, ±1/(2φ), 0).
fn icosians() -> Vec<[Zp; 4]> {
    let zero = Zp { a: 0, b: 0 };
    let mut out = Vec::with_capacity(120);
    for i in 0..4 {
        for s in [2, -2] {
            let mut v = [zero; 4];
            v[i] = Zp { a: s, b: 0 };
            out.push(v);
        }
    }
    for mask in 0..16 {
        out.push(std::array::from_fn(|k| Zp {
            a: if mask >> k & 1 == 1 { -1 } else { 1 },
            b: 0,
        }));
    }
    // φ/2 = (0 + 1φ)/2, ½ = (1 + 0φ)/2, 1/(2φ) = (−1 + 1φ)/2.
    let base = [
        Zp { a: 0, b: 1 },
        Zp { a: 1, b: 0 },
        Zp { a: -1, b: 1 },
        zero,
    ];
    for p in even_permutations() {
        for signs in 0..8 {
            let mut v = [zero; 4];
            for (k, &value) in base.iter().enumerate() {
                v[p[k]] = if k < 3 && signs >> k & 1 == 1 {
                    value.neg()
                } else {
                    value
                };
            }
            out.push(v);
        }
    }
    out
}

fn real4(u: &[Zp; 4]) -> V<4> {
    u.map(Zp::real)
}

/// Quaternion product, real part first.
fn qmul(a: &V<4>, b: &V<4>) -> V<4> {
    [
        a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
        a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
        a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1],
        a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0],
    ]
}

fn qconj(a: &V<4>) -> V<4> {
    [a[0], -a[1], -a[2], -a[3]]
}

/// H4's Gram matrix for unit roots: 1 –5– 2 – 3 – 4.
fn h4_gram() -> M<4> {
    let p = phi();
    [
        [1.0, -p / 2.0, 0.0, 0.0],
        [-p / 2.0, 1.0, -0.5, 0.0],
        [0.0, -0.5, 1.0, -0.5],
        [0.0, 0.0, -0.5, 1.0],
    ]
}

/// The 720 edges of the 600-cell: vertex pairs 36° apart (dot product φ/2).
fn edges_600(v: &[V<4>]) -> Vec<(usize, usize)> {
    let c = phi() / 2.0;
    let mut out = Vec::new();
    for i in 0..v.len() {
        for j in i + 1..v.len() {
            if (dot(&v[i], &v[j]) - c).abs() < EPS {
                out.push((i, j));
            }
        }
    }
    out
}

/// The H4 Coxeter plane of the 600-cell: (vertices, projected points, plane).
struct Cell600 {
    exact: Vec<[Zp; 4]>,
    v: Vec<V<4>>,
    b1: V<4>,
    b2: V<4>,
}

fn cell600() -> Result<Cell600, String> {
    let exact = icosians();
    let v: Vec<V<4>> = exact.iter().map(real4).collect();
    let simple = simple_roots(&v, &h4_gram())?;
    let (_, b1, b2) = coxeter_plane(&simple, 30, &H4_OTHERS)?;
    Ok(Cell600 { exact, v, b1, b2 })
}

impl Cell600 {
    fn xy(&self) -> Vec<(f64, f64)> {
        self.v
            .iter()
            .map(|p| project(p, (&self.b1, &self.b2)))
            .collect()
    }
}

/// E8 as icosians: the 120 unit icosians u and the 120 φu, with inner product
/// 2 × (rational part of the Z[φ] inner product). In coordinates, x ↦
/// (s₁·x, s₂·x′) with x′ the Galois conjugate, s₁² = 2/(φ√5), s₂² = 2φ/√5.
/// Returns the 240 vectors and, for each, whether it is a φ-scaled root.
fn e8_icosian(cell: &Cell600) -> Vec<(V<8>, bool)> {
    let p = phi();
    let s1 = (2.0 / (p * 5f64.sqrt())).sqrt();
    let s2 = (2.0 * p / 5f64.sqrt()).sqrt();
    let mut out = Vec::with_capacity(240);
    for golden in [false, true] {
        for u in &cell.exact {
            let x = if golden { u.map(Zp::times_phi) } else { *u };
            let mut v = [0.0; 8];
            for k in 0..4 {
                v[k] = s1 * x[k].real();
                v[k + 4] = s2 * x[k].conjugate();
            }
            out.push((v, golden));
        }
    }
    out
}

/// The E8 Coxeter-plane projection of the icosian roots: the plane is the H4
/// Coxeter plane in the first golden half (the H4 Coxeter element acting on
/// both halves is an E8 Coxeter element; the tests check the rings against the
/// standard construction).
fn e8_icosian_xy(cell: &Cell600) -> Vec<((f64, f64), bool)> {
    let mut b1 = [0.0; 8];
    let mut b2 = [0.0; 8];
    b1[..4].copy_from_slice(&cell.b1);
    b2[..4].copy_from_slice(&cell.b2);
    e8_icosian(cell)
        .iter()
        .map(|(v, g)| (project(v, (&b1, &b2)), *g))
        .collect()
}

fn e8_standard_xy() -> Result<Vec<(f64, f64)>, String> {
    let roots = e8_standard();
    let simple = simple_roots(&roots, &e8_gram())?;
    let (_, b1, b2) = coxeter_plane(&simple, 30, &E8_OTHERS)?;
    Ok(roots.iter().map(|r| project(r, (&b1, &b2))).collect())
}

// ---------------------------------------------------------------- SpiralCore orbit

/// `ROOTS` from `crates/uor-r4-discrete/build.rs`: the anchor (2,2,0,…) first,
/// then the rest of its orbit in lexicographic order.
const ORBIT: [[i8; 8]; 16] = [
    [2, 2, 0, 0, 0, 0, 0, 0],
    [-2, -2, 0, 0, 0, 0, 0, 0],
    [-2, 2, 0, 0, 0, 0, 0, 0],
    [0, 0, -2, 0, -2, 0, 0, 0],
    [0, 0, -2, 0, 2, 0, 0, 0],
    [0, 0, 0, -2, 0, 0, 0, -2],
    [0, 0, 0, -2, 0, 0, 0, 2],
    [0, 0, 0, 0, 0, -2, -2, 0],
    [0, 0, 0, 0, 0, -2, 2, 0],
    [0, 0, 0, 0, 0, 2, -2, 0],
    [0, 0, 0, 0, 0, 2, 2, 0],
    [0, 0, 0, 2, 0, 0, 0, -2],
    [0, 0, 0, 2, 0, 0, 0, 2],
    [0, 0, 2, 0, -2, 0, 0, 0],
    [0, 0, 2, 0, 2, 0, 0, 0],
    [2, -2, 0, 0, 0, 0, 0, 0],
];

/// `BRANCH` from `build.rs`: each root's three ternary ports (itself, then its
/// images under the first generator and its inverse).
const BRANCH: [u8; 48] = [
    0, 14, 3, 1, 3, 14, 2, 13, 4, 3, 0, 1, 4, 2, 15, 5, 7, 10, 6, 9, 8, 7, 12, 5, 8, 6, 11, 9, 11,
    6, 10, 5, 12, 11, 8, 9, 12, 10, 7, 13, 15, 2, 14, 1, 0, 15, 4, 13,
];

/// Rows 1–15 of `AXES` from `build.rs` (the 15 generators, states 1–15):
/// output coordinate r reads input coordinate `a & 7`, negated when `a & 8`.
const GENERATORS: [[u8; 8]; 15] = [
    [12, 10, 1, 14, 0, 7, 3, 13],
    [15, 11, 6, 1, 5, 12, 10, 0],
    [2, 12, 8, 13, 1, 3, 15, 6],
    [14, 13, 15, 4, 11, 1, 0, 2],
    [5, 14, 11, 2, 7, 8, 1, 12],
    [13, 14, 11, 2, 15, 0, 1, 4],
    [9, 0, 12, 7, 2, 6, 13, 11],
    [3, 7, 13, 8, 14, 2, 4, 9],
    [15, 3, 14, 9, 5, 12, 2, 0],
    [14, 5, 15, 12, 3, 9, 0, 2],
    [10, 12, 0, 13, 1, 3, 7, 14],
    [4, 10, 1, 14, 8, 15, 3, 5],
    [15, 3, 6, 9, 13, 4, 10, 0],
    [11, 15, 13, 0, 14, 2, 4, 1],
    [9, 0, 4, 7, 10, 14, 5, 11],
];

fn act(g: &[u8; 8], x: &[i32; 8]) -> [i32; 8] {
    std::array::from_fn(|r| {
        let a = g[r];
        let v = x[usize::from(a & 7)];
        if a & 8 == 0 {
            v
        } else {
            -v
        }
    })
}

fn orbit_root(i: usize) -> [i32; 8] {
    ORBIT[i].map(i32::from)
}

fn orbit_index(x: &[i32; 8]) -> Option<usize> {
    (0..16).find(|&i| orbit_root(i) == *x)
}

/// The anchor's breadth-first transport tree over the orbit, in `build.rs`'s
/// FIFO and generator order: `parent[to] = (from, generator)`.
fn orbit_bfs() -> Result<[Option<(usize, usize)>; 16], String> {
    let mut parent = [None; 16];
    let mut seen = [false; 16];
    seen[0] = true;
    let mut queue = std::collections::VecDeque::from([0usize]);
    while let Some(from) = queue.pop_front() {
        for (edge, g) in GENERATORS.iter().enumerate() {
            let to = orbit_index(&act(g, &orbit_root(from)))
                .ok_or("a generator leaves the anchor orbit")?;
            if !seen[to] {
                seen[to] = true;
                parent[to] = Some((from, edge));
                queue.push_back(to);
            }
        }
    }
    if seen.iter().any(|s| !s) {
        return Err("the anchor orbit is not connected".into());
    }
    Ok(parent)
}

/// The kernel's port choice (`discrete.cu` `route_vector`): smallest L1
/// distance to the port's root, then coordinate Hamming, then port order.
fn nearest_port(q: &[i32; 8], ports: &[usize]) -> (usize, Vec<(u32, u32)>) {
    let costs: Vec<(u32, u32)> = ports
        .iter()
        .map(|&r| {
            let y = orbit_root(r);
            let l1 = q.iter().zip(&y).map(|(a, b)| a.abs_diff(*b)).sum();
            let ham = q.iter().zip(&y).filter(|(a, b)| a != b).count() as u32;
            (l1, ham)
        })
        .collect();
    let mut best = 0;
    for (p, c) in costs.iter().enumerate() {
        if c < &costs[best] {
            best = p;
        }
    }
    (best, costs)
}

// ---------------------------------------------------------------- SVG

const CSS: &str = r#"svg { --bg:#ffffff; --ink:#1f2328; --body:#2b3138; --muted:#57606a; --rule:#d0d7de; --panel:#f6f8fa;
      --blue:#0072b2; --sky:#56b4e9; --green:#007a5e; --orange:#e69f00; --verm:#d55e00; --purple:#7a4fa3; --pink:#cc79a7;
      --active:#007a5e; --ready:#b06d00; --parked:#7a4fa3; }
@media (prefers-color-scheme: dark) { svg { --bg:#0d1117; --ink:#e6edf3; --body:#c9d1d9; --muted:#8b949e; --rule:#30363d; --panel:#161b22;
      --blue:#58a6ff; --sky:#79c0ff; --green:#3fb950; --orange:#f0b72f; --verm:#ff7b72; --purple:#bc8cff; --pink:#f778ba;
      --active:#3fb950; --ready:#f0b72f; --parked:#bc8cff; } }
.bg { fill: var(--bg); }
text { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif; fill: var(--ink); }"#;

/// A number with at most two decimals and no trailing zeros.
fn n(x: f64) -> String {
    let s = format!("{x:.2}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" {
        "0".into()
    } else {
        s.into()
    }
}

struct Fig {
    name: String,
    w: f64,
    h: f64,
    body: String,
    bad: Vec<String>,
}

impl Fig {
    fn new(name: &str, w: f64, h: f64) -> Fig {
        Fig {
            name: name.into(),
            w,
            h,
            body: String::new(),
            bad: Vec::new(),
        }
    }

    /// Text in `--color`; fails the task if its estimated box leaves the 16 px margin.
    #[allow(clippy::too_many_arguments)]
    fn text(&mut self, x: f64, y: f64, size: f64, weight: u16, color: &str, anchor: &str, s: &str) {
        let w = text_width(s, size, weight >= 600);
        let (l, r) = match anchor {
            "middle" => (x - w / 2.0, x + w / 2.0),
            "end" => (x - w, x),
            _ => (x, x + w),
        };
        if l < 16.0 || r > self.w - 16.0 || y - 0.8 * size < 16.0 || y + 0.25 * size > self.h - 16.0
        {
            self.bad
                .push(format!("{}: \"{s}\" leaves the margin", self.name));
        }
        let _ = writeln!(
            self.body,
            r#"<text x="{}" y="{}" font-size="{size}" font-weight="{weight}" style="fill:var(--{color})" text-anchor="{anchor}">{}</text>"#,
            n(x),
            n(y),
            esc(s)
        );
    }

    /// Text with a halo in the background colour, readable over lines.
    fn haloed(&mut self, x: f64, y: f64, size: f64, color: &str, s: &str) {
        self.text(x, y, size, 600, color, "middle", s);
        if let Some(at) = self.body.rfind("style=\"fill:var(--") {
            self.body.insert_str(
                at + 7,
                "stroke:var(--bg);stroke-width:3;paint-order:stroke;",
            );
        }
    }

    fn title(&mut self, s: &str) {
        self.text(24.0, 40.0, 19.0, 600, "ink", "start", s);
    }

    fn label(&mut self, x: f64, y: f64, color: &str, anchor: &str, s: &str) {
        self.text(x, y, 13.0, 400, color, anchor, s);
    }

    fn note(&mut self, x: f64, y: f64, anchor: &str, s: &str) {
        self.text(x, y, 11.5, 400, "muted", anchor, s);
    }

    fn circle(&mut self, cx: f64, cy: f64, r: f64, style: &str) {
        let _ = writeln!(
            self.body,
            r#"<circle cx="{}" cy="{}" r="{}" style="{style}"/>"#,
            n(cx),
            n(cy),
            n(r)
        );
    }

    fn line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, style: &str) {
        let _ = writeln!(
            self.body,
            r#"<line x1="{}" y1="{}" x2="{}" y2="{}" style="{style}"/>"#,
            n(x1),
            n(y1),
            n(x2),
            n(y2)
        );
    }

    fn path(&mut self, d: &str, style: &str) {
        let _ = writeln!(self.body, r#"<path d="{d}" style="{style}"/>"#);
    }

    fn rect(&mut self, x: f64, y: f64, w: f64, h: f64, rx: f64, style: &str) {
        let _ = writeln!(
            self.body,
            r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" style="{style}"/>"#,
            n(x),
            n(y),
            n(w),
            n(h),
            n(rx)
        );
    }

    /// A straight arrow in `--color` with an explicit filled head.
    #[allow(clippy::too_many_arguments)]
    fn arrow(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        color: &str,
        w: f64,
        head: f64,
        dash: &str,
    ) {
        let (dx, dy) = (x2 - x1, y2 - y1);
        let len = dx.hypot(dy);
        if len < 1e-6 {
            return;
        }
        let (ux, uy) = (dx / len, dy / len);
        let (bx, by) = (x2 - ux * head, y2 - uy * head);
        let dash = if dash.is_empty() {
            String::new()
        } else {
            format!(";stroke-dasharray:{dash}")
        };
        self.line(
            x1,
            y1,
            bx + ux * 0.5,
            by + uy * 0.5,
            &format!("stroke:var(--{color});stroke-width:{w};fill:none{dash}"),
        );
        let (px, py) = (-uy * head * 0.45, ux * head * 0.45);
        self.path(
            &format!(
                "M{} {}L{} {}L{} {}Z",
                n(x2),
                n(y2),
                n(bx + px),
                n(by + py),
                n(bx - px),
                n(by - py)
            ),
            &format!("fill:var(--{color})"),
        );
    }

    /// A small curved rotation glyph (three quarters of a circle with a head).
    fn turn(&mut self, cx: f64, cy: f64, r: f64, color: &str) {
        let (a0, a1) = (-0.25 * PI, 1.25 * PI);
        let (x0, y0) = (cx + r * a0.cos(), cy + r * a0.sin());
        let (x1, y1) = (cx + r * a1.cos(), cy + r * a1.sin());
        self.path(
            &format!(
                "M{} {}A{} {} 0 1 1 {} {}",
                n(x0),
                n(y0),
                n(r),
                n(r),
                n(x1),
                n(y1)
            ),
            &format!("stroke:var(--{color});stroke-width:1.4;fill:none"),
        );
        // Head at the end, pointing along the tangent (counter-clockwise on screen).
        let (tx, ty) = (a1.sin(), -a1.cos());
        let h = r * 0.9;
        self.path(
            &format!(
                "M{} {}L{} {}L{} {}Z",
                n(x1 + tx * h * 0.5),
                n(y1 + ty * h * 0.5),
                n(x1 - tx * h * 0.5 + (x1 - cx) / r * h * 0.45),
                n(y1 - ty * h * 0.5 + (y1 - cy) / r * h * 0.45),
                n(x1 - tx * h * 0.5 - (x1 - cx) / r * h * 0.45),
                n(y1 - ty * h * 0.5 - (y1 - cy) / r * h * 0.45),
            ),
            &format!("fill:var(--{color})"),
        );
    }

    fn finish(self, title: &str, desc: &str) -> Result<String, String> {
        if !self.bad.is_empty() {
            return Err(self.bad.join("\n"));
        }
        Ok(format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {w} {h}\" width=\"{w}\" role=\"img\" aria-labelledby=\"t d\">\n\
             <title id=\"t\">{}</title>\n<desc id=\"d\">{}</desc>\n<style>\n{CSS}\n</style>\n\
             <rect class=\"bg\" x=\"0\" y=\"0\" width=\"{w}\" height=\"{h}\"/>\n{}</svg>\n",
            esc(title),
            esc(desc),
            self.body,
            w = self.w,
            h = self.h,
        ))
    }
}

/// Screen mapping for a plane plot: centre and pixels per unit, y up.
struct Plot {
    cx: f64,
    cy: f64,
    s: f64,
}

impl Plot {
    fn at(&self, (x, y): (f64, f64)) -> (f64, f64) {
        (self.cx + self.s * x, self.cy - self.s * y)
    }
}

// ---------------------------------------------------------------- figures

fn fig_e8() -> Result<String, String> {
    let cell = cell600()?;
    let pts = e8_icosian_xy(&cell);
    let xy: Vec<(f64, f64)> = pts.iter().map(|p| p.0).collect();
    let rg = rings(&xy);
    if rg.len() != 8 || rg.iter().any(|(_, m)| m.len() != 30) {
        return Err("E8 projection is not 8 rings of 30".into());
    }
    let standard = rings(&e8_standard_xy()?);
    if standard.len() != 8
        || standard
            .iter()
            .zip(&rg)
            .any(|(a, b)| (a.0 - b.0).abs() > 1e-9 || a.1.len() != b.1.len())
    {
        return Err("icosian E8 rings differ from the standard construction".into());
    }
    let rmax = rg[7].0;
    let plot = Plot {
        cx: 300.0,
        cy: 334.0,
        s: 238.0 / rmax,
    };
    let mut f = Fig::new("e8-coxeter-plane", 960.0, 640.0);
    f.title("E8: 240 roots in the Coxeter plane");
    f.note(
        24.0,
        62.0,
        "start",
        "Two copies of the 600-cell's 120 points, one scaled by the golden ratio φ, make up E8's roots.",
    );
    for (r, _) in &rg {
        f.circle(
            plot.cx,
            plot.cy,
            r * plot.s,
            "fill:none;stroke:var(--rule);stroke-width:1",
        );
    }
    for &((x, y), golden) in &pts {
        let (px, py) = plot.at((x, y));
        if golden {
            f.rect(
                px - 3.6,
                py - 3.6,
                7.2,
                7.2,
                0.0,
                "fill:var(--verm);stroke:var(--bg);stroke-width:0.8",
            );
        } else {
            f.circle(
                px,
                py,
                4.2,
                "fill:var(--blue);stroke:var(--bg);stroke-width:0.8",
            );
        }
    }
    let lx = 604.0;
    f.circle(lx + 6.0, 112.0, 5.0, "fill:var(--blue)");
    f.label(
        lx + 20.0,
        117.0,
        "body",
        "start",
        "120 unit icosians (the group 2I)",
    );
    f.rect(lx + 1.5, 129.5, 9.0, 9.0, 0.0, "fill:var(--verm)");
    f.label(lx + 20.0, 139.0, "body", "start", "120 golden roots φ·u");
    f.text(
        lx,
        182.0,
        14.0,
        600,
        "ink",
        "start",
        "Ring radii (roots have length √2)",
    );
    for (k, (r, members)) in rg.iter().enumerate() {
        let y = 208.0 + 24.0 * k as f64;
        let golden = pts[members[0]].1;
        if members.iter().any(|&i| pts[i].1 != golden) {
            return Err("an E8 ring mixes icosian and golden roots".into());
        }
        if golden {
            f.rect(lx + 2.0, y - 8.5, 8.0, 8.0, 0.0, "fill:var(--verm)");
        } else {
            f.circle(lx + 6.0, y - 4.5, 4.5, "fill:var(--blue)");
        }
        f.label(lx + 20.0, y, "body", "start", &format!("ring {}", k + 1));
        f.label(lx + 84.0, y, "body", "start", &format!("r = {r:.3}"));
        f.label(
            lx + 170.0,
            y,
            "muted",
            "start",
            &format!("30 {}", if golden { "golden" } else { "icosian" }),
        );
    }
    f.note(
        lx,
        420.0,
        "start",
        "Each ring is one orbit of the Coxeter element,",
    );
    f.note(lx, 436.0, "start", "which turns the plane by 12° (h = 30).");
    f.note(
        lx,
        460.0,
        "start",
        "Every golden ring is φ times an icosian ring.",
    );
    f.text(
        24.0,
        614.0,
        12.0,
        400,
        "muted",
        "start",
        "240 roots, 8 rings of 30, computed. Inner product 2 × rational part; plane of the H4 Coxeter element on both halves.",
    );
    f.text(
        24.0,
        598.0,
        12.0,
        400,
        "muted",
        "start",
        "The rings match the standard roots (±1,±1,0⁶) and (±½)⁸ projected on their own Coxeter plane.",
    );
    f.finish(
        "E8 roots in the Coxeter plane",
        "The 240 roots of E8, built as the 120 unit icosians (blue dots) and the 120 golden roots phi times u (vermilion squares), projected onto an E8 Coxeter plane. They fall on 8 concentric rings of 30 with 30-fold symmetry: four rings of icosians and four golden rings, each golden ring phi times an icosian ring. Ring radii are listed at the right. Coordinates are computed, and the rings match the standard construction of E8.",
    )
}

fn fig_600() -> Result<String, String> {
    let cell = cell600()?;
    let xy = cell.xy();
    let edges = edges_600(&cell.v);
    let rg = rings(&xy);
    if edges.len() != 720 || rg.len() != 4 || rg.iter().any(|(_, m)| m.len() != 30) {
        return Err("600-cell projection is not 720 edges on 4 rings of 30".into());
    }
    let mut degree = vec![0usize; xy.len()];
    for &(i, j) in &edges {
        degree[i] += 1;
        degree[j] += 1;
    }
    if degree.iter().any(|&d| d != 12) {
        return Err("600-cell: a vertex does not have exactly 12 neighbours".into());
    }
    let plot = Plot {
        cx: 310.0,
        cy: 338.0,
        s: 244.0 / rg[3].0,
    };
    let mut f = Fig::new("600-cell-coxeter", 960.0, 640.0);
    f.title("The 600-cell in the H4 Coxeter plane");
    f.note(
        24.0,
        62.0,
        "start",
        "The 120 unit icosians, seen along the plane in which the Coxeter element is a rotation by 12°.",
    );
    let mut d = String::new();
    for &(i, j) in &edges {
        let (x1, y1) = plot.at(xy[i]);
        let (x2, y2) = plot.at(xy[j]);
        let _ = write!(d, "M{} {}L{} {}", n(x1), n(y1), n(x2), n(y2));
    }
    f.path(
        &d,
        "stroke:var(--muted);stroke-opacity:0.28;stroke-width:0.6;fill:none",
    );
    let colors = ["blue", "green", "orange", "purple"];
    for (k, (_, members)) in rg.iter().enumerate() {
        for &i in members {
            let (px, py) = plot.at(xy[i]);
            f.circle(
                px,
                py,
                4.0,
                &format!(
                    "fill:var(--{});stroke:var(--bg);stroke-width:0.8",
                    colors[k]
                ),
            );
        }
    }
    let lx = 630.0;
    f.text(
        lx,
        120.0,
        14.0,
        600,
        "ink",
        "start",
        "Four rings of 30 vertices",
    );
    for (k, (r, _)) in rg.iter().enumerate() {
        let y = 148.0 + 24.0 * k as f64;
        f.circle(
            lx + 6.0,
            y - 4.5,
            4.5,
            &format!("fill:var(--{})", colors[k]),
        );
        f.label(lx + 20.0, y, "body", "start", &format!("ring {}", k + 1));
        f.label(lx + 84.0, y, "body", "start", &format!("r = {r:.3}"));
    }
    f.text(lx, 270.0, 14.0, 600, "ink", "start", "720 edges");
    f.label(
        lx,
        294.0,
        "body",
        "start",
        "Each joins two vertices 36° apart",
    );
    f.label(
        lx,
        312.0,
        "body",
        "start",
        "(distance 1/φ ≈ 0.618). Every vertex",
    );
    f.label(lx, 330.0, "body", "start", "has exactly 12 neighbours.");
    f.note(
        lx,
        370.0,
        "start",
        "Vertices are unit quaternions, so radii",
    );
    f.note(lx, 386.0, "start", "are fractions of 1.");
    f.text(
        24.0,
        614.0,
        12.0,
        400,
        "muted",
        "start",
        "120 vertices, 720 edges, 4 rings of 30, computed: H4 Coxeter element of order 30, eigenplane for exp(2πi/30).",
    );
    f.finish(
        "The 600-cell in the H4 Coxeter plane",
        "The 120 vertices of the 600-cell, which are the 120 unit icosians, projected onto the H4 Coxeter plane (Coxeter number 30). The points fall on four concentric rings of 30, coloured by ring, joined by the 720 edges between vertices 36 degrees apart, drawn faint. Every vertex has 12 neighbours. Coordinates are computed.",
    )
}

/// The flock figure's choices: the query vertex (outer ring, top of the
/// drawing), its 12 neighbours, and the 7 kept. Which 7 are kept is
/// illustrative: the 7 whose turned positions lie nearest one direction (a
/// cap of the icosahedron of neighbours), so their consensus is visible.
fn flock_choice(cell: &Cell600) -> Result<(usize, Vec<usize>, Vec<usize>), String> {
    let xy = cell.xy();
    let rg = rings(&xy);
    let target = PI / 2.0;
    let gap = |i: usize| {
        let a = xy[i].1.atan2(xy[i].0);
        let d = (a - target).rem_euclid(2.0 * PI);
        d.min(2.0 * PI - d)
    };
    let q = *rg[3]
        .1
        .iter()
        .min_by_key(|&&a| (key(gap(a)), a))
        .unwrap_or(&0);
    let c = phi() / 2.0;
    let nb: Vec<usize> = (0..cell.v.len())
        .filter(|&j| (dot(&cell.v[q], &cell.v[j]) - c).abs() < EPS)
        .collect();
    if nb.len() != 12 {
        return Err(format!(
            "flock: vertex {q} has {} neighbours, not 12",
            nb.len()
        ));
    }
    let rel = |j: usize| turned(cell, q, j);
    let lead = rel(nb[0]);
    let mut kept = nb.clone();
    kept.sort_by_key(|&a| (std::cmp::Reverse(key(dot(&rel(a), &lead))), a));
    kept.truncate(7);
    kept.sort_unstable();
    Ok((q, nb, kept))
}

/// Neighbour `j` turned into the frame of query `q`: the imaginary part of q̄·n.
fn turned(cell: &Cell600, q: usize, j: usize) -> V<3> {
    let r = qmul(&qconj(&cell.v[q]), &cell.v[j]);
    [r[1], r[2], r[3]]
}

fn fig_flock() -> Result<String, String> {
    let cell = cell600()?;
    let xy = cell.xy();
    let edges = edges_600(&cell.v);
    let rmax = rings(&xy)[3].0;
    let plot = Plot {
        cx: 300.0,
        cy: 345.0,
        s: 226.0 / rmax,
    };
    let (q, nb, kept) = flock_choice(&cell)?;
    let mut f = Fig::new("flock-read", 960.0, 640.0);
    f.title("The flock read on the 600-cell");
    f.note(
        24.0,
        62.0,
        "start",
        "Read a few nearby earlier tokens, turn each into the query's frame, and move toward their consensus.",
    );
    let mut d = String::new();
    for &(i, j) in &edges {
        let (x1, y1) = plot.at(xy[i]);
        let (x2, y2) = plot.at(xy[j]);
        let _ = write!(d, "M{} {}L{} {}", n(x1), n(y1), n(x2), n(y2));
    }
    f.path(
        &d,
        "stroke:var(--muted);stroke-opacity:0.18;stroke-width:0.6;fill:none",
    );
    for p in &xy {
        let (px, py) = plot.at(*p);
        f.circle(px, py, 2.6, "fill:var(--muted);fill-opacity:0.55");
    }
    let (qx, qy) = plot.at(xy[q]);
    for &j in &nb {
        let (x, y) = plot.at(xy[j]);
        if kept.contains(&j) {
            f.arrow(
                qx + 0.85 * (x - qx),
                qy + 0.85 * (y - qy),
                qx + 0.2 * (x - qx),
                qy + 0.2 * (y - qy),
                "blue",
                1.4,
                7.0,
                "",
            );
        } else {
            f.line(
                qx,
                qy,
                x,
                y,
                "stroke:var(--blue);stroke-opacity:0.4;stroke-width:1",
            );
        }
    }
    for &j in &nb {
        let (x, y) = plot.at(xy[j]);
        if kept.contains(&j) {
            f.circle(
                x,
                y,
                6.0,
                "fill:var(--blue);stroke:var(--bg);stroke-width:1",
            );
            f.circle(x, y, 10.0, "fill:none;stroke:var(--blue);stroke-width:1.5");
        } else {
            f.circle(
                x,
                y,
                6.0,
                "fill:var(--bg);stroke:var(--blue);stroke-width:1.5",
            );
        }
    }
    f.circle(
        qx,
        qy,
        8.0,
        "fill:var(--orange);stroke:var(--bg);stroke-width:1.2",
    );
    f.label(qx + 14.0, qy - 10.0, "ink", "start", "query (home vertex)");
    // Key under the plot.
    let ky = 588.0;
    f.circle(40.0, ky - 4.5, 6.0, "fill:var(--blue)");
    f.circle(
        40.0,
        ky - 4.5,
        10.0,
        "fill:none;stroke:var(--blue);stroke-width:1.5",
    );
    f.label(
        56.0,
        ky,
        "body",
        "start",
        &format!("{} kept earlier tokens", kept.len()),
    );
    f.circle(
        230.0,
        ky - 4.5,
        6.0,
        "fill:var(--bg);stroke:var(--blue);stroke-width:1.5",
    );
    f.label(
        244.0,
        ky,
        "body",
        "start",
        &format!("other neighbours ({} in all, 36° away)", nb.len()),
    );

    // Right panel: the kept neighbours turned into the query's frame, drawn in
    // the plane of their consensus and the third imaginary axis.
    let rel: Vec<V<3>> = kept.iter().map(|&j| turned(&cell, q, j)).collect();
    let mut sum = [0.0; 3];
    for v in &rel {
        for k in 0..3 {
            sum[k] += v[k];
        }
    }
    let len = dot(&sum, &sum).sqrt();
    if len < 1e-6 {
        return Err("the kept neighbours have no consensus".into());
    }
    let e1 = sum.map(|x| x / len);
    let mut e2 = [0.0, 0.0, 1.0];
    if dot(&e1, &e2).abs() > 0.9 {
        e2 = [0.0, 1.0, 0.0];
    }
    let t = dot(&e2, &e1);
    let mut e2: V<3> = std::array::from_fn(|k| e2[k] - t * e1[k]);
    let m = dot(&e2, &e2).sqrt();
    e2 = e2.map(|x| x / m);
    let px0 = 600.0;
    f.rect(
        px0,
        86.0,
        340.0,
        300.0,
        8.0,
        "fill:var(--panel);stroke:var(--rule);stroke-width:1",
    );
    f.text(
        px0 + 16.0,
        110.0,
        14.0,
        600,
        "ink",
        "start",
        "In the query's frame",
    );
    f.note(
        px0 + 16.0,
        128.0,
        "start",
        "each kept token is turned by q̄ (query conjugate)",
    );
    f.turn(px0 + 26.0, 146.0, 7.0, "blue");
    f.note(
        px0 + 40.0,
        150.0,
        "start",
        "7 turned arrows, added head to tail",
    );
    let unit = 74.0;
    let (ox, oy) = (px0 + 40.0, 236.0);
    let (mut hx, mut hy) = (ox, oy);
    for v in &rel {
        let (a, b) = (dot(v, &e1), dot(v, &e2));
        let (nx, ny) = (hx + unit * a, hy - unit * b);
        f.arrow(hx, hy, nx, ny, "blue", 1.6, 7.0, "");
        hx = nx;
        hy = ny;
    }
    f.arrow(ox, oy, hx, hy, "verm", 3.0, 11.0, "");
    f.circle(ox, oy, 3.0, "fill:var(--ink)");
    f.label(hx + 10.0, hy + 4.0, "verm", "start", "consensus");
    // The query's own vector bends toward the consensus (step illustrative).
    let (q0x, q0y) = (px0 + 40.0, 352.0);
    let (bx, by) = (q0x + 90.0, q0y - 44.0);
    let (ax, ay) = (bx + 0.5 * (hx - ox), by + 0.5 * (hy - oy));
    f.arrow(q0x, q0y, bx, by, "orange", 1.6, 9.0, "3 3");
    f.arrow(q0x, q0y, ax, ay, "orange", 2.6, 10.0, "");
    f.circle(q0x, q0y, 3.0, "fill:var(--ink)");
    f.note(
        px0 + 16.0,
        376.0,
        "start",
        "query: dotted before, solid after (illustrative)",
    );

    // Inset: the live integer version.
    let iy = 410.0;
    f.rect(
        px0,
        iy,
        340.0,
        206.0,
        8.0,
        "fill:var(--panel);stroke:var(--rule);stroke-width:1",
    );
    f.text(
        px0 + 16.0,
        iy + 24.0,
        14.0,
        600,
        "ink",
        "start",
        "Live integer version",
    );
    f.note(
        px0 + 16.0,
        iy + 42.0,
        "start",
        "the live integer version uses 16 frame roots",
    );
    let (rcx, rcy, rr) = (px0 + 90.0, iy + 126.0, 58.0);
    f.circle(rcx, rcy, rr, "fill:none;stroke:var(--rule);stroke-width:1");
    // The summed vector points between roots; it snaps to the nearest one.
    let sa = 2.0 * PI * 2.35 / 16.0;
    let snap = 2usize;
    for k in 0..16 {
        let a = 2.0 * PI * k as f64 / 16.0;
        let (x, y) = (rcx + rr * a.cos(), rcy - rr * a.sin());
        if k == snap {
            f.circle(x, y, 5.5, "fill:var(--verm)");
            f.circle(x, y, 9.0, "fill:none;stroke:var(--verm);stroke-width:1.5");
        } else {
            f.circle(x, y, 3.4, "fill:var(--muted)");
        }
    }
    f.arrow(
        rcx,
        rcy,
        rcx + 0.78 * rr * sa.cos(),
        rcy - 0.78 * rr * sa.sin(),
        "blue",
        2.0,
        8.0,
        "",
    );
    f.note(rcx, iy + 202.0, "middle", "16 roots, on a ring (schematic)");
    // Dial: −1 subtracts, 0 skips, +1 adds.
    let dx = px0 + 190.0;
    f.label(dx, iy + 84.0, "body", "start", "snap to nearest root,");
    f.label(dx, iy + 102.0, "body", "start", "then a dial:");
    for (k, (lab, word)) in [("−1", "subtract"), ("0", "skip"), ("+1", "add")]
        .iter()
        .enumerate()
    {
        let y = iy + 120.0 + 26.0 * k as f64;
        let on = k == 2;
        f.rect(
            dx,
            y,
            34.0,
            20.0,
            4.0,
            if on {
                "fill:var(--verm);stroke:var(--verm);stroke-width:1.5"
            } else {
                "fill:var(--bg);stroke:var(--rule);stroke-width:1.5"
            },
        );
        f.text(
            dx + 17.0,
            y + 15.0,
            13.0,
            600,
            if on { "bg" } else { "body" },
            "middle",
            lab,
        );
        f.label(dx + 44.0, y + 15.0, "body", "start", word);
    }
    f.note(
        24.0,
        604.0,
        "start",
        "600-cell geometry computed (H4 Coxeter plane); which 7 neighbours are kept is illustrative.",
    );
    f.note(
        24.0,
        620.0,
        "start",
        "The 600-cell picture is the earlier float-era form; the live version uses 16 frame roots.",
    );
    f.finish(
        "The flock read on the 600-cell",
        "Left: the 600-cell's 120 vertices in the H4 Coxeter plane with faint edges. One query token sits at its home vertex (orange) with its 12 adjacent vertices outlined in blue; seven of them, the kept earlier tokens, are filled and circled. Right: in the query's frame, the seven kept tokens are turned by the query's conjugate and added head to tail into one consensus vector (vermilion), and the query's own vector bends toward it, before dotted and after solid. Inset: the live integer version uses 16 frame roots; the summed vector snaps to the nearest root and a three-position dial adds, skips or subtracts it. Geometry is computed; which seven are kept is illustrative.",
    )
}

/// Ports chosen on the highlighted ternary path of the SpiralCore figure.
const SPIRAL_PATH: [usize; 4] = [1, 0, 2, 1];

fn fig_spiral() -> Result<String, String> {
    let mut f = Fig::new("spiralcore-ternary-tree", 960.0, 640.0);
    f.title("SpiralCore: a ternary tree picks a block, the orbit moves it");
    f.note(
        24.0,
        62.0,
        "start",
        "Left: one choice of three per level. Right: the chosen block travels along exact edges between the 16 roots of an E8 orbit.",
    );
    // Left: complete ternary tree of depth 4, radial, breadth-first numbering.
    let (tcx, tcy, step) = (280.0, 335.0, 54.0);
    let pos = |d: u32, i: usize| -> (f64, f64) {
        if d == 0 {
            return (tcx, tcy);
        }
        let count = 3usize.pow(d) as f64;
        let a = -PI / 2.0 + 2.0 * PI * (i as f64 + 0.5) / count;
        (
            tcx + step * f64::from(d) * a.cos(),
            tcy + step * f64::from(d) * a.sin(),
        )
    };
    let bfs = |d: u32, i: usize| (3usize.pow(d) - 1) / 2 + i;
    let mut on_path = vec![(0u32, 0usize)];
    let mut idx = 0usize;
    for (d, &p) in SPIRAL_PATH.iter().enumerate() {
        idx = idx * 3 + p;
        on_path.push((d as u32 + 1, idx));
    }
    for d in 1..=4u32 {
        f.circle(
            tcx,
            tcy,
            step * f64::from(d),
            "fill:none;stroke:var(--rule);stroke-width:1;stroke-dasharray:2 4",
        );
        for i in 0..3usize.pow(d) {
            let (x, y) = pos(d, i);
            let (px, py) = pos(d - 1, i / 3);
            let hot = on_path.contains(&(d, i));
            f.line(
                px,
                py,
                x,
                y,
                if hot {
                    "stroke:var(--verm);stroke-width:2.6"
                } else {
                    "stroke:var(--muted);stroke-opacity:0.45;stroke-width:1"
                },
            );
        }
    }
    for d in 0..=4u32 {
        for i in 0..3usize.pow(d) {
            let (x, y) = pos(d, i);
            if !on_path.contains(&(d, i)) {
                f.circle(x, y, if d == 4 { 2.6 } else { 3.6 }, "fill:var(--muted)");
            }
        }
    }
    for &(d, i) in &on_path {
        let (x, y) = pos(d, i);
        f.circle(
            x,
            y,
            9.5,
            "fill:var(--verm);stroke:var(--bg);stroke-width:1.2",
        );
        let label = bfs(d, i).to_string();
        let size = if label.len() > 1 { 9.5 } else { 11.0 };
        f.text(x, y + size * 0.36, size, 600, "bg", "middle", &label);
    }
    f.note(
        tcx,
        640.0 - 24.0 - 30.0,
        "middle",
        "nodes numbered in breadth-first order; one path highlighted",
    );
    f.note(
        tcx,
        640.0 - 24.0 - 12.0,
        "middle",
        "depth 1: 1–3 · depth 2: 4–12 · depth 3: 13–39 · depth 4: 40–120",
    );

    // Right: the anchor orbit's breadth-first transport tree.
    let parent = orbit_bfs()?;
    let mut depth = [0usize; 16];
    for (k, depth_k) in depth.iter_mut().enumerate().skip(1) {
        let mut at = k;
        while let Some((p, _)) = parent[at] {
            *depth_k += 1;
            at = p;
        }
    }
    // The block's destination: follow the highlighted ports through BRANCH.
    let mut root = SPIRAL_PATH[0];
    for &p in &SPIRAL_PATH[1..] {
        root = usize::from(BRANCH[root * 3 + p]);
    }
    let mut chain = vec![root];
    while let Some((p, _)) = parent[*chain.last().unwrap_or(&0)] {
        chain.push(p);
    }
    let children = |k: usize| -> Vec<usize> {
        (0..16)
            .filter(|&c| parent[c].map(|(p, _)| p) == Some(k))
            .collect()
    };
    // Top-down layout: leaves left to right in depth-first order, parents
    // centred over their children.
    let (ox0, osp, oy0, odep) = (612.0, 26.0, 150.0, 120.0);
    let mut at = [(0.0, 0.0); 16];
    let mut next_leaf = 0.0;
    fn place(
        k: usize,
        kids: &dyn Fn(usize) -> Vec<usize>,
        at: &mut [(f64, f64); 16],
        next: &mut f64,
        depth: &[usize; 16],
        geo: (f64, f64, f64, f64),
    ) {
        let (x0, sp, y0, dy) = geo;
        let c = kids(k);
        let x = if c.is_empty() {
            let x = x0 + sp * *next;
            *next += 1.0;
            x
        } else {
            let xs: Vec<f64> = c
                .iter()
                .map(|&ch| {
                    place(ch, kids, at, next, depth, geo);
                    at[ch].0
                })
                .collect();
            (xs[0] + xs[xs.len() - 1]) / 2.0
        };
        at[k] = (x, y0 + dy * depth[k] as f64);
    }
    place(
        0,
        &children,
        &mut at,
        &mut next_leaf,
        &depth,
        (ox0, osp, oy0, odep),
    );
    for k in 1..16 {
        if let Some((p, g)) = parent[k] {
            let hot = chain.contains(&k);
            let (x1, y1) = at[p];
            let (x2, y2) = at[k];
            f.line(
                x1,
                y1,
                x2,
                y2,
                if hot {
                    "stroke:var(--verm);stroke-width:2.6"
                } else {
                    "stroke:var(--blue);stroke-opacity:0.55;stroke-width:1.2"
                },
            );
            if hot {
                let (lx, anchor) = if x2 < x1 {
                    ((x1 + x2) / 2.0 - 6.0, "end")
                } else {
                    ((x1 + x2) / 2.0 + 6.0, "start")
                };
                f.note(lx, (y1 + y2) / 2.0 - 4.0, anchor, &format!("g{}", g + 1));
            }
        }
    }
    for (k, &(x, y)) in at.iter().enumerate() {
        let hot = chain.contains(&k);
        f.circle(
            x,
            y,
            10.0,
            if hot {
                "fill:var(--verm);stroke:var(--bg);stroke-width:1.2"
            } else {
                "fill:var(--bg);stroke:var(--blue);stroke-width:1.5"
            },
        );
        f.text(
            x,
            y + 3.8,
            10.5,
            600,
            if hot { "bg" } else { "ink" },
            "middle",
            &k.to_string(),
        );
    }
    let fmt = |v: &[i32; 8]| {
        format!(
            "({})",
            v.iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    };
    f.note(
        600.0,
        448.0,
        "start",
        &format!("root 0 (anchor) = {}", fmt(&orbit_root(0))),
    );
    f.note(
        600.0,
        464.0,
        "start",
        &format!("root {root} = {}", fmt(&orbit_root(root))),
    );
    let ports = SPIRAL_PATH.map(|p| p.to_string()).join(", ");
    f.note(
        600.0,
        488.0,
        "start",
        &format!("The left path's ports {ports} select root {root}"),
    );
    f.note(600.0, 504.0, "start", "through the ternary port table.");
    f.note(
        600.0,
        544.0,
        "start",
        "15 edges: the anchor's breadth-first tree.",
    );
    f.note(
        600.0,
        560.0,
        "start",
        "Each edge applies one generator gₖ, a signed",
    );
    f.note(
        600.0,
        576.0,
        "start",
        "permutation of the 8 coordinates, undone",
    );
    f.note(600.0, 592.0, "start", "exactly by its inverse.");
    f.finish(
        "SpiralCore ternary tree and orbit",
        "Left: a complete three-branch tree of depth four, laid out radially, with nodes numbered in breadth-first order (depth 1 holds 1 to 3, depth 4 holds 40 to 120) and one root-to-leaf path highlighted. Right: the 16 roots of the E8 orbit of (2,2,0,0,0,0,0,0) joined by the 15 edges of the anchor's breadth-first transport tree; each edge is one generator, a signed permutation of the eight coordinates. The path from the anchor to the root that the highlighted ports select is highlighted, with its generators named. Orbit data come from the tables the discrete crate's build script generates.",
    )
}

/// The routed read's illustrative query and per-level frames (generator
/// numbers, 0 for the identity).
const ROUTE_QUERY: [i32; 8] = [-2, -1, 1, 0, 0, 2, 0, -1];
const ROUTE_FRAMES: [usize; 3] = [0, 3, 12];

struct Level {
    roots: Vec<usize>,
    costs: Vec<(u32, u32)>,
    best: usize,
}

fn route_levels() -> Vec<Level> {
    let mut out: Vec<Level> = Vec::new();
    let mut root = 0usize;
    for (stage, &frame) in ROUTE_FRAMES.iter().enumerate() {
        let q = if frame == 0 {
            ROUTE_QUERY
        } else {
            act(&GENERATORS[frame - 1], &ROUTE_QUERY)
        };
        let roots: Vec<usize> = if stage == 0 {
            vec![0, 1, 2]
        } else {
            (0..3).map(|p| usize::from(BRANCH[root * 3 + p])).collect()
        };
        let (best, costs) = nearest_port(&q, &roots);
        root = roots[best];
        out.push(Level { roots, costs, best });
    }
    out
}

fn fig_route() -> Result<String, String> {
    let levels = route_levels();
    let mut f = Fig::new("route-read", 960.0, 600.0);
    f.title("The routed read: one nearest port per level");
    f.note(
        24.0,
        62.0,
        "start",
        "A ternary tree over 27 earlier tokens. The query picks one of three ports per level and ends at exactly one token.",
    );
    let ys = [118.0, 200.0, 292.0, 392.0];
    let leaf_x = |i: usize| 176.0 + 16.0 * i as f64;
    let node_x = |d: usize, i: usize| {
        let w = 3usize.pow(3 - d as u32);
        let lo = leaf_x(i * w);
        let hi = leaf_x(i * w + w - 1);
        (lo + hi) / 2.0
    };
    let mut path = vec![0usize];
    for l in &levels {
        let last = *path.last().unwrap_or(&0);
        path.push(last * 3 + l.best);
    }
    // Edges.
    for d in 1..=3usize {
        for i in 0..3usize.pow(d as u32) {
            let hot = path[d] == i && path[d - 1] == i / 3;
            f.line(
                node_x(d - 1, i / 3),
                ys[d - 1],
                node_x(d, i),
                ys[d] - if d == 3 { 8.0 } else { 0.0 },
                if hot {
                    "stroke:var(--verm);stroke-width:2.6"
                } else {
                    "stroke:var(--muted);stroke-opacity:0.4;stroke-width:1"
                },
            );
        }
    }
    // Query box and root.
    let rx = node_x(0, 0);
    f.rect(
        rx - 34.0,
        76.0,
        68.0,
        22.0,
        4.0,
        "fill:var(--orange);stroke:none",
    );
    f.text(rx, 92.0, 13.0, 600, "bg", "middle", "query q");
    f.circle(rx, ys[0], 6.0, "fill:var(--verm)");
    // Ports per level: lit ports carry their L1 distance.
    for (k, l) in levels.iter().enumerate() {
        let d = k + 1;
        for i in 0..3usize.pow(d as u32) {
            let x = node_x(d, i);
            let parent_on = i / 3 == path[d - 1];
            if d == 3 {
                let picked = i == path[3];
                f.rect(
                    x - 6.0,
                    ys[3] - 6.0,
                    12.0,
                    12.0,
                    2.0,
                    if picked {
                        "fill:var(--verm)"
                    } else if parent_on {
                        "fill:var(--bg);stroke:var(--blue);stroke-width:1.5"
                    } else {
                        "fill:var(--muted);fill-opacity:0.35"
                    },
                );
                if picked {
                    f.circle(
                        x,
                        ys[3],
                        10.5,
                        "fill:none;stroke:var(--verm);stroke-width:1.8",
                    );
                }
            } else if i == path[d] {
                f.circle(x, ys[d], 7.0, "fill:var(--verm)");
            } else if parent_on {
                f.circle(
                    x,
                    ys[d],
                    6.0,
                    "fill:var(--bg);stroke:var(--blue);stroke-width:1.5",
                );
            } else {
                f.circle(x, ys[d], 3.5, "fill:var(--muted);fill-opacity:0.5");
            }
            if parent_on {
                let p = i % 3;
                let (l1, _) = l.costs[p];
                let color = if p == l.best { "verm" } else { "blue" };
                let dy = if d == 3 { -16.0 } else { -12.0 };
                f.haloed(x, ys[d] + dy, 12.0, color, &l1.to_string());
            }
        }
    }
    let px = leaf_x(path[3]);
    f.label(px, ys[3] + 32.0, "verm", "middle", "picked up");
    f.note(
        leaf_x(0) - 8.0,
        ys[3] + 58.0,
        "start",
        "27 earlier tokens (leaves); only the picked one is read",
    );
    // Accumulating chain beside the tree.
    let mut prev = 0usize;
    for (k, l) in levels.iter().enumerate() {
        let goal = l.roots[l.best];
        let y = ys[k + 1];
        f.turn(34.0, y - 4.0, 7.0, "verm");
        f.label(48.0, y, "body", "start", &format!("BFS[{prev}→{goal}]"));
        prev = goal;
    }
    f.note(24.0, ys[0] + 4.0, "start", "rotation chain");
    f.note(
        24.0,
        470.0,
        "start",
        "Numbers: L1 distance from the framed query to each",
    );
    f.note(
        24.0,
        486.0,
        "start",
        "port's orbit root; the nearest wins (ties: Hamming,",
    );
    f.note(
        24.0,
        502.0,
        "start",
        "then port order). The leaf's contents are never read",
    );
    f.note(24.0, 518.0, "start", "before the pick.");
    let qs = ROUTE_QUERY
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(",");
    f.note(
        24.0,
        560.0,
        "start",
        &format!("Illustrative query q = ({qs}) and per-level frames; distances use the kernel's rule on the 16 real orbit roots and ports."),
    );
    f.note(24.0, 576.0, "start", "The chain composes each level's transport onto the running total, applied to the picked token's vector.");

    // Inset: comparisons per read.
    let ix = 640.0;
    f.rect(
        ix,
        86.0,
        300.0,
        440.0,
        8.0,
        "fill:var(--panel);stroke:var(--rule);stroke-width:1",
    );
    f.text(
        ix + 16.0,
        110.0,
        14.0,
        600,
        "ink",
        "start",
        "Comparisons per read",
    );
    f.circle(ix + 22.0, 128.0, 5.0, "fill:var(--muted)");
    f.note(
        ix + 32.0,
        132.0,
        "start",
        "scored: one per earlier position",
    );
    f.rect(ix + 17.0, 141.0, 10.0, 10.0, 0.0, "fill:var(--verm)");
    f.note(ix + 32.0, 150.0, "start", "routed: ≤ levels × 16 per block");
    let base = 470.0;
    let scale = 280.0 / 243.0;
    for (k, ctx) in [27u32, 81, 243].iter().enumerate() {
        let levels = match ctx {
            27 => 3,
            81 => 4,
            _ => 5,
        };
        let routed = levels * 16;
        let gx = ix + 40.0 + 90.0 * k as f64;
        let hs = f64::from(*ctx) * scale;
        let hr = f64::from(routed) * scale;
        f.rect(
            gx,
            base - hs,
            28.0,
            hs,
            0.0,
            "fill:var(--muted);fill-opacity:0.6",
        );
        f.rect(gx + 32.0, base - hr, 28.0, hr, 0.0, "fill:var(--verm)");
        f.text(
            gx + 14.0,
            base - hs - 6.0,
            12.0,
            600,
            "body",
            "middle",
            &ctx.to_string(),
        );
        f.text(
            gx + 46.0,
            base - hr - 6.0,
            12.0,
            600,
            "verm",
            "middle",
            &routed.to_string(),
        );
        f.label(
            gx + 30.0,
            base + 18.0,
            "body",
            "middle",
            &format!("{ctx} tokens"),
        );
    }
    f.line(
        ix + 30.0,
        base,
        ix + 290.0,
        base,
        "stroke:var(--rule);stroke-width:1",
    );
    f.note(
        ix + 16.0,
        506.0,
        "start",
        "Routed bars are upper bounds from the kernel",
    );
    f.note(
        ix + 16.0,
        520.0,
        "start",
        "description, not measured times.",
    );
    f.finish(
        "The routed read",
        "A ternary tree over 27 earlier tokens. The query enters at the top. At each of three levels the three child ports of the current node are lit with their L1 distance from the framed query to the port's orbit root, and the nearest is highlighted; a rotation step is added to a chain at the left. The path ends at exactly one leaf, circled and labelled picked up; the other leaves are greyed. Inset: comparisons per read for a scored read (27, 81, 243, one per earlier position) against a routed read (at most levels times 16 per block: 48, 64, 80), labelled as upper bounds from the kernel description. The query and frames are illustrative; distances are computed on the real orbit roots and ports.",
    )
}

/// Split an SVG into alternating text and number tokens. A number is an
/// optional `-` directly before a digit, digits, and an optional `.` fraction.
fn svg_tokens(s: &str) -> Vec<(bool, &str)> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let (mut i, mut start) = (0, 0);
    while i < b.len() {
        let neg = b[i] == b'-' && b.get(i + 1).is_some_and(u8::is_ascii_digit);
        if b[i].is_ascii_digit() || neg {
            if start < i {
                out.push((false, &s[start..i]));
            }
            let j0 = i;
            i += usize::from(neg);
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
            if i + 1 < b.len() && b[i] == b'.' && b[i + 1].is_ascii_digit() {
                i += 1;
                while i < b.len() && b[i].is_ascii_digit() {
                    i += 1;
                }
            }
            out.push((true, &s[j0..i]));
            start = i;
        } else {
            i += 1;
        }
    }
    if start < b.len() {
        out.push((false, &s[start..]));
    }
    out
}

/// Whether a committed SVG matches a freshly generated one. Text must be
/// identical; numbers may differ by one unit in the second decimal, because
/// `sin`, `cos` and the eigen-solve differ in their last bits between platform
/// maths libraries, and a value near a rounding boundary can then print one
/// digit apart (seen on Linux CI against macOS for the 600-cell).
pub(crate) fn svg_matches(committed: &str, generated: &str) -> bool {
    svg_mismatch(committed, generated).is_none()
}

/// The first token where the committed and generated SVGs disagree, for the
/// check's error output; `None` when they match.
pub(crate) fn svg_mismatch(committed: &str, generated: &str) -> Option<String> {
    let (a, b) = (svg_tokens(committed), svg_tokens(generated));
    for (k, (&(na, ta), &(nb, tb))) in a.iter().zip(&b).enumerate() {
        let same = na == nb
            && (ta == tb
                || (na
                    && matches!(
                        (ta.parse::<f64>(), tb.parse::<f64>()),
                        (Ok(x), Ok(y)) if (x - y).abs() <= 0.010_000_1
                    )));
        if !same {
            return Some(format!("token {k}: committed {ta:?}, generated {tb:?}"));
        }
    }
    (a.len() != b.len()).then(|| format!("{} tokens committed, {} generated", a.len(), b.len()))
}

type Build = fn() -> Result<String, String>;

fn figures() -> [(&'static str, Build); 5] {
    [
        ("geometry/e8-coxeter-plane.svg", fig_e8),
        ("geometry/600-cell-coxeter.svg", fig_600),
        ("mechanisms/flock-read.svg", fig_flock),
        ("geometry/spiralcore-ternary-tree.svg", fig_spiral),
        ("mechanisms/route-read.svg", fig_route),
    ]
}

pub fn run(root: &Path, args: &[String]) -> Result<(), String> {
    let mut write = false;
    let mut out = root.join("docs/figures");
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--write" => write = true,
            "--out" => {
                out = PathBuf::from(it.next().ok_or("--out needs a directory")?);
            }
            other => return Err(format!("figures-math: unknown argument {other}")),
        }
    }
    let mut stale = Vec::new();
    for (rel, build) in figures() {
        let svg = build().map_err(|e| format!("{rel}: {e}"))?;
        let path = out.join(rel);
        if write {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
            }
            std::fs::write(&path, &svg).map_err(|e| format!("{}: {e}", path.display()))?;
            println!("wrote {}", path.display());
        } else if !std::fs::read_to_string(&path).is_ok_and(|c| svg_matches(&c, &svg)) {
            let why = std::fs::read_to_string(&path)
                .map(|c| svg_mismatch(&c, &svg).unwrap_or_default())
                .unwrap_or_else(|e| e.to_string());
            eprintln!("{rel}: {why}");
            stale.push(rel);
        }
    }
    if stale.is_empty() {
        if !write {
            println!(
                "figures-math: all {} computed figures are current",
                figures().len()
            );
        }
        Ok(())
    } else {
        Err(format!(
            "stale computed figures (run `cargo xtask figures-math --write`): {}",
            stale.join(", ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_check_tolerates_last_digit_rounding_only() {
        let a = r#"<path d="M12.34 -5.6L7 8"/><text>ring 1 r = 0.618</text>"#;
        assert!(svg_matches(a, a));
        // One unit in the second decimal, either sign, is platform rounding.
        assert!(svg_matches(a, &a.replace("12.34", "12.35")));
        assert!(svg_matches(a, &a.replace("-5.6", "-5.59")));
        // A larger numeric change, a changed word or a missing token is not.
        assert!(!svg_matches(a, &a.replace("12.34", "12.36")));
        assert!(!svg_matches(a, &a.replace("ring 1", "ring 2")));
        assert!(!svg_matches(a, &a.replace("ring", "rung")));
        assert!(!svg_matches(a, &a.replace("L7 8", "L7")));
        // A sign is part of the number: "-5.6" against "5.6" is a change.
        assert!(!svg_matches(a, &a.replace("-5.6", "5.6")));
    }

    #[test]
    fn e8_has_240_roots_of_norm_two() {
        let roots = e8_standard();
        assert_eq!(roots.len(), 240);
        assert!(roots.iter().all(|r| (dot(r, r) - 2.0).abs() < EPS));
        for (i, a) in roots.iter().enumerate() {
            for b in &roots[i + 1..] {
                let d = dot(a, b);
                assert!((d - d.round()).abs() < EPS && (-2.0..=1.0).contains(&d.round()));
            }
        }
    }

    #[test]
    fn icosian_e8_is_a_root_system_of_norm_two() {
        let cell = cell600().unwrap();
        let roots = e8_icosian(&cell);
        assert_eq!(roots.len(), 240);
        for (i, (a, _)) in roots.iter().enumerate() {
            assert!((dot(a, a) - 2.0).abs() < EPS);
            for (b, _) in &roots[i + 1..] {
                let d = dot(a, b);
                assert!(
                    (d - d.round()).abs() < 1e-9,
                    "non-integral inner product {d}"
                );
                assert!(d.round() >= -2.0 && d.round() <= 1.0);
            }
        }
    }

    #[test]
    fn e8_projections_agree_ring_for_ring() {
        let std_xy = e8_standard_xy().unwrap();
        let cell = cell600().unwrap();
        let ico: Vec<(f64, f64)> = e8_icosian_xy(&cell).iter().map(|p| p.0).collect();
        let a = rings(&std_xy);
        let b = rings(&ico);
        assert_eq!(a.len(), 8);
        assert!(a.iter().all(|(_, m)| m.len() == 30));
        assert_eq!(b.len(), 8);
        for ((ra, ma), (rb, mb)) in a.iter().zip(&b) {
            assert!((ra - rb).abs() < 1e-9, "{ra} vs {rb}");
            assert_eq!(ma.len(), mb.len());
        }
        // Four golden rings, each φ times an icosian ring.
        let pts = e8_icosian_xy(&cell);
        let (gold, ico): (Vec<_>, Vec<_>) = b.iter().partition(|(_, m)| pts[m[0]].1);
        assert_eq!(gold.len(), 4);
        for ((rg, _), (ri, _)) in gold.iter().zip(&ico) {
            assert!((rg - phi() * ri).abs() < 1e-9);
        }
    }

    #[test]
    fn six_hundred_cell_has_120_unit_vertices_and_720_edges() {
        let cell = cell600().unwrap();
        assert_eq!(cell.v.len(), 120);
        assert!(cell.v.iter().all(|v| (dot(v, v) - 1.0).abs() < EPS));
        for i in 0..120 {
            for j in i + 1..120 {
                assert!(cell.exact[i] != cell.exact[j]);
            }
        }
        let e = edges_600(&cell.v);
        assert_eq!(e.len(), 720);
        let mut deg = [0; 120];
        for (i, j) in e {
            deg[i] += 1;
            deg[j] += 1;
        }
        assert!(deg.iter().all(|&d| d == 12));
        let rg = rings(&cell.xy());
        assert_eq!(rg.len(), 4);
        assert!(rg.iter().all(|(_, m)| m.len() == 30));
    }

    #[test]
    fn icosians_are_closed_under_the_flock_rotation() {
        let cell = cell600().unwrap();
        let (q, nb, kept) = flock_choice(&cell).expect("flock choice");
        assert_eq!(nb.len(), 12);
        assert_eq!(kept.len(), 7);
        for &j in &nb {
            let r = qmul(&qconj(&cell.v[q]), &cell.v[j]);
            assert!(
                (r[0] - phi() / 2.0).abs() < EPS,
                "relative element is 36° from 1"
            );
            assert!(cell
                .v
                .iter()
                .any(|v| (0..4).all(|k| (v[k] - r[k]).abs() < EPS)));
        }
    }

    #[test]
    fn projection_is_deterministic() {
        let a = cell600().unwrap().xy();
        let b = cell600().unwrap().xy();
        assert_eq!(a, b);
        assert_eq!(e8_standard_xy().unwrap(), e8_standard_xy().unwrap());
        for (_, build) in figures() {
            assert_eq!(build().unwrap(), build().unwrap());
        }
    }

    #[test]
    fn orbit_tables_match_their_generators() {
        // The orbit of the anchor under the generators, anchor first then
        // lexicographic, is the copied ROOTS table.
        let mut seen = std::collections::BTreeSet::new();
        let mut todo = vec![orbit_root(0)];
        while let Some(x) = todo.pop() {
            if seen.insert(x) {
                for g in &GENERATORS {
                    todo.push(act(g, &x));
                }
            }
        }
        let mut expect: Vec<[i32; 8]> = vec![orbit_root(0)];
        expect.extend(seen.iter().filter(|r| **r != orbit_root(0)).copied());
        let table: Vec<[i32; 8]> = (0..16).map(orbit_root).collect();
        assert_eq!(expect, table);
        // BRANCH is (r, g1·r, g1⁻¹·r); g1 has order 4, so its inverse is its cube.
        let g1 = &GENERATORS[0];
        let id: [i32; 8] = std::array::from_fn(|k| k as i32 + 1);
        assert_eq!(act(g1, &act(g1, &act(g1, &act(g1, &id)))), id);
        assert_ne!(act(g1, &act(g1, &id)), id);
        for r in 0..16 {
            let x = orbit_root(r);
            let once = orbit_index(&act(g1, &x)).unwrap();
            let twice = orbit_index(&act(g1, &act(g1, &act(g1, &x)))).unwrap();
            assert_eq!(
                &BRANCH[r * 3..r * 3 + 3],
                &[r as u8, once as u8, twice as u8]
            );
        }
        assert_eq!(
            orbit_bfs().unwrap().iter().filter(|p| p.is_some()).count(),
            15
        );
    }

    #[test]
    fn route_choice_follows_the_kernel_rule() {
        let levels = route_levels();
        assert_eq!(levels.len(), 3);
        for l in &levels {
            let best = l.costs[l.best];
            assert!(l.costs.iter().all(|c| *c >= best));
        }
    }
}
