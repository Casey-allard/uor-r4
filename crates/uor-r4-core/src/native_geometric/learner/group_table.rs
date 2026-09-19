//! Exact `2I` (binary icosahedral) group composition by table lookup.
//!
//! # Why this exists
//!
//! `context_hopf_fiber_q30` accumulated the context state as
//! `s3 = s3.mul_q30(&q_root)` over up to 64 tokens, and `mul_q30` is a 16-multiply Hamilton
//! product. That is on the order of **1,000 multiplies per generated token** — roughly 40x the
//! next largest multiplier site in the serving path, and outside the arithmetic scan that the
//! multiplier-free invariant actually covered (see
//! `docs/evidence/native_geometric_d0a_multiplier_census_2026-09-19.txt`).
//!
//! # Why a table is the *correct* fix, not a workaround
//!
//! The 120 canonical H4 roots are exactly the 120 elements of `2I`, and `2I` is closed under
//! quaternion multiplication. So the accumulated state is always a group element and the
//! accumulation *is* exact group composition — which is what this project already claims as its
//! mechanism. Composing two group elements is therefore a lookup, not an arithmetic operation.
//!
//! The table is also **more exact than what it replaces**. The current path multiplies Q1.30
//! *quantized* roots, so the product drifts off the group and the code calls `normalized()` every
//! 8 steps to contain the drift. The table stores the *exact* product of the exact root elements,
//! so no drift accumulates and no renormalization is needed.
//!
//! # Cost and verification
//!
//! The table is 120x120 bytes (14,400) plus a 120-byte inverse row — about 14.2 KB, built once
//! lazily. Building it uses floating point for the exact product and nearest-element
//! classification, which is **initialization, not the serving path**: the served accumulation is
//! table reads and integer index arithmetic only, with zero multiplies and zero floats.
//!
//! Correctness is not assumed. The tests establish that the table is a **group**:
//! each row is a permutation of the elements (left multiplication is a bijection), composition is
//! associative for all 1,728,000 triples, the identity is two-sided, and every element has an
//! inverse in the table. A quasigroup with those properties *is* the group.

use std::sync::OnceLock;

use super::embedding::{canonical_h4_roots_q30, H4_ROOT_COUNT};

/// Number of group elements: the 120 canonical H4 roots.
pub const GROUP_ORDER: usize = H4_ROOT_COUNT;

/// Row stride of the product table, padded to a power of two.
///
/// Addressing element `(a, b)` of a 120x120 table is `a * 120 + b`, and a multiply by a
/// non-power-of-two constant needs a shift/add sequence the compiler often emits as `madd` -- the
/// serving zero-check (`scripts/serving_multiplier_check.py`) counted those instructions in
/// `compose_ring_roots`. Padding the stride to 128 makes the row base a shift, at the cost of
/// 120 x 8 = 960 padding bytes.
pub const ROW_STRIDE: usize = 128;

/// Exact `2I` composition tables.
pub struct GroupTable {
    /// Flat `GROUP_ORDER * GROUP_ORDER` table: `product[a * N + b]` is the index of
    /// `root_a * root_b` (left factor `a`, matching `s3.mul_q30(root)` order).
    pub product: Box<[u8]>,
    /// Index of the identity element `(1, 0, 0, 0)`.
    pub identity: u8,
    /// `inverse[a]` is the index of `root_a^-1`.
    pub inverse: Box<[u8]>,
    /// Worst-case distance from an exact product to its nearest group element, as
    /// `1 - |<product, nearest>|`. A large value would mean the classification is ambiguous.
    pub max_closure_residual: f64,
}

static TABLE: OnceLock<GroupTable> = OnceLock::new();

/// The process-wide exact composition table, built on first use.
pub fn group_table() -> &'static GroupTable {
    TABLE.get_or_init(GroupTable::build)
}

/// Exact `2I` composition over a chronological context window.
///
/// Returns the index of the accumulated group element. Only the most recent 64 tokens take
/// part, matching the serving context bound. This is the single implementation of the
/// accumulation that `ExportedGeometricModel::context_hopf_fiber_q30`,
/// `MmapGeometricModel::context_hopf_fiber_q30` and `context_fiber_from_ring` all call.
///
/// It existed as three independent copies of a 64-step `mul_q30` loop, which is why converting
/// one of them did not remove the multiplies from serving. Sharing it is the point.
pub fn compose_context_roots(token_to_root: &[u8], context: &[usize]) -> usize {
    let table = group_table();
    let mut state = table.identity as usize;
    if token_to_root.is_empty() || context.is_empty() {
        return state;
    }
    let start = context.len().saturating_sub(64);
    let root_len = token_to_root.len();
    for &token in &context[start..] {
        // `.min(GROUP_ORDER - 1)` rather than `% GROUP_ORDER`: `token_to_root` holds canonical
        // root indices from `nearest_h4_root` and is already in range, so this is a defensive
        // bound, and a branchless min avoids a modulo by a non-power-of-two constant -- which the
        // compiler lowers to a `umulh` multiply sequence. The serving zero-check
        // (`scripts/serving_multiplier_check.py`) flagged exactly those instructions.
        let root = (token_to_root[token.min(root_len - 1)] as usize).min(GROUP_ORDER - 1);
        state = table.product[state * ROW_STRIDE + root] as usize;
    }
    state
}

/// As [`compose_context_roots`], for a ring buffer: accumulates the most recent `length`
/// entries in chronological order, oldest first.
pub fn compose_ring_roots(
    token_to_root: &[u8],
    ring: &[u32],
    cursor: usize,
    length: usize,
) -> usize {
    let table = group_table();
    let mut state = table.identity as usize;
    if token_to_root.is_empty() || ring.is_empty() {
        return state;
    }
    let window = length.min(64).min(ring.len());
    let root_len = token_to_root.len();
    let cursor = cursor % ring.len();
    for lag in (1..=window).rev() {
        let index = if cursor >= lag {
            cursor - lag
        } else {
            ring.len() - (lag - cursor)
        };
        let token = ring[index] as usize;
        let root = (token_to_root[token.min(root_len - 1)] as usize).min(GROUP_ORDER - 1);
        state = table.product[state * ROW_STRIDE + root] as usize;
    }
    state
}

fn normalize(v: [f64; 4]) -> [f64; 4] {
    let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2] + v[3] * v[3]).sqrt();
    if n == 0.0 {
        [1.0, 0.0, 0.0, 0.0]
    } else {
        [v[0] / n, v[1] / n, v[2] / n, v[3] / n]
    }
}

/// Hamilton product `a * b` in the same order as `UnitS3Q30::mul_q30`.
fn hamilton(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    [
        a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3],
        a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2],
        a[0] * b[2] - a[1] * b[3] + a[2] * b[0] + a[3] * b[1],
        a[0] * b[3] + a[1] * b[2] - a[2] * b[1] + a[3] * b[0],
    ]
}

fn dot(a: [f64; 4], b: [f64; 4]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]
}

impl GroupTable {
    fn build() -> Self {
        let roots = canonical_h4_roots_q30();
        debug_assert_eq!(roots.len(), GROUP_ORDER);
        let exact: Vec<[f64; 4]> = roots
            .iter()
            .map(|q| normalize([q.0[0] as f64, q.0[1] as f64, q.0[2] as f64, q.0[3] as f64]))
            .collect();

        let n = GROUP_ORDER;
        let mut product = vec![0u8; n * ROW_STRIDE];
        let mut max_closure_residual = 0.0f64;

        for a in 0..n {
            for b in 0..n {
                let p = normalize(hamilton(exact[a], exact[b]));
                // Classify by the *signed* inner product so `q` and `-q` stay distinct: they are
                // different group elements even though they are the same rotation.
                let mut best = 0usize;
                let mut best_dot = -2.0f64;
                let mut second = -2.0f64;
                for (i, e) in exact.iter().enumerate() {
                    let d = dot(p, *e);
                    if d > best_dot {
                        second = best_dot;
                        best_dot = d;
                        best = i;
                    } else if d > second {
                        second = d;
                    }
                }
                product[a * ROW_STRIDE + b] = best as u8;
                let residual = 1.0 - best_dot;
                if residual > max_closure_residual {
                    max_closure_residual = residual;
                }
                debug_assert!(
                    best_dot - second > 1e-6,
                    "ambiguous classification for ({a}, {b}): {best_dot} vs {second}"
                );
            }
        }

        // Identity is the element with (1, 0, 0, 0) up to the exact representation.
        let mut identity = 0usize;
        let mut best_dot = -2.0f64;
        for (i, e) in exact.iter().enumerate() {
            let d = dot(*e, [1.0, 0.0, 0.0, 0.0]);
            if d > best_dot {
                best_dot = d;
                identity = i;
            }
        }

        let mut inverse = vec![0u8; n];
        for a in 0..n {
            let mut found = None;
            for b in 0..n {
                if product[a * ROW_STRIDE + b] as usize == identity {
                    found = Some(b as u8);
                    break;
                }
            }
            inverse[a] = found.unwrap_or(0);
        }

        Self {
            product: product.into_boxed_slice(),
            identity: identity as u8,
            inverse: inverse.into_boxed_slice(),
            max_closure_residual,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_a_latin_square_so_left_multiplication_is_a_bijection() {
        let t = group_table();
        assert_eq!(t.product.len(), GROUP_ORDER * ROW_STRIDE);
        for a in 0..GROUP_ORDER {
            let mut seen = [false; GROUP_ORDER];
            for b in 0..GROUP_ORDER {
                let p = t.product[a * ROW_STRIDE + b] as usize;
                assert!(p < GROUP_ORDER);
                assert!(!seen[p], "row {a} repeats element {p}");
                seen[p] = true;
            }
        }
        for b in 0..GROUP_ORDER {
            let mut seen = [false; GROUP_ORDER];
            for a in 0..GROUP_ORDER {
                let p = t.product[a * ROW_STRIDE + b] as usize;
                assert!(!seen[p], "column {b} repeats element {p}");
                seen[p] = true;
            }
        }
    }

    #[test]
    fn composition_is_associative_for_every_triple() {
        let t = group_table();
        let n = GROUP_ORDER;
        for a in 0..n {
            for b in 0..n {
                let ab = t.product[a * ROW_STRIDE + b] as usize;
                for c in 0..n {
                    let left = t.product[ab * ROW_STRIDE + c] as usize;
                    let bc = t.product[b * ROW_STRIDE + c] as usize;
                    let right = t.product[a * ROW_STRIDE + bc] as usize;
                    assert_eq!(left, right, "associativity failed for ({a}, {b}, {c})");
                }
            }
        }
    }

    #[test]
    fn identity_is_two_sided_and_inverses_exist() {
        let t = group_table();
        let e = t.identity as usize;
        let n = GROUP_ORDER;
        for a in 0..n {
            assert_eq!(
                t.product[e * ROW_STRIDE + a] as usize,
                a,
                "e*a != a for {a}"
            );
            assert_eq!(
                t.product[a * ROW_STRIDE + e] as usize,
                a,
                "a*e != a for {a}"
            );
            let inv = t.inverse[a] as usize;
            assert_eq!(
                t.product[a * ROW_STRIDE + inv] as usize,
                e,
                "a*inv(a) != e for {a}"
            );
        }
    }

    #[test]
    fn classification_is_unambiguous() {
        let t = group_table();
        assert!(
            t.max_closure_residual < 1e-6,
            "closure residual {:.3e} is too large: the product classification would be ambiguous",
            t.max_closure_residual
        );
    }

    /// The table must reproduce what the removed Q1.30 accumulation produced. The old algorithm is
    /// kept here as a test oracle so the replacement is justified by measurement rather than
    /// asserted; it is test-only and is not on any serving path.
    #[test]
    fn table_agrees_with_the_quantized_accumulation_it_replaced() {
        use crate::native_geometric::hopf_metric::UnitS3Q30;

        fn q30_reference(token_to_root: &[u8], context: &[usize]) -> usize {
            let roots = canonical_h4_roots_q30();
            let exact: Vec<[f64; 4]> = roots
                .iter()
                .map(|q| normalize([q.0[0] as f64, q.0[1] as f64, q.0[2] as f64, q.0[3] as f64]))
                .collect();
            let mut s3 = UnitS3Q30::IDENTITY;
            let start = context.len().saturating_sub(64);
            let root_len = token_to_root.len();
            let mut step = 0usize;
            for &token in &context[start..] {
                let idx = token_to_root[token.min(root_len - 1)] as usize % GROUP_ORDER;
                s3 = s3.mul_q30(&roots[idx]);
                step += 1;
                if step % 8 == 0 {
                    s3 = s3.normalized();
                }
            }
            if step % 8 != 0 {
                s3 = s3.normalized();
            }
            let p = normalize([
                s3.0[0] as f64,
                s3.0[1] as f64,
                s3.0[2] as f64,
                s3.0[3] as f64,
            ]);
            let mut best = 0usize;
            let mut best_dot = -2.0f64;
            for (i, e) in exact.iter().enumerate() {
                let d = dot(p, *e);
                if d > best_dot {
                    best_dot = d;
                    best = i;
                }
            }
            best
        }

        // Deterministic pseudo-random token-to-root assignment and contexts.
        let vocab = 4096usize;
        let mut state = 0x1234_5678_9abc_def0u64;
        let mut next = move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let token_to_root: Vec<u8> = (0..vocab)
            .map(|_| (next() % GROUP_ORDER as u64) as u8)
            .collect();
        let mut agree = 0usize;
        let mut total = 0usize;
        for len in [1usize, 2, 5, 17, 64] {
            for _ in 0..200 {
                let ctx: Vec<usize> = (0..len).map(|_| (next() as usize) % vocab).collect();
                let table_state = compose_context_roots(&token_to_root, &ctx);
                let quantized = q30_reference(&token_to_root, &ctx);
                total += 1;
                if table_state == quantized {
                    agree += 1;
                }
            }
        }
        assert_eq!(total, 1000);
        assert!(
            agree * 100 >= total * 99,
            "the exact table must reproduce the quantized accumulation it replaced: {agree}/{total}"
        );
    }

    /// The shared helpers must be the single composition path: a one-token context is that
    /// token's root, an empty context is the identity, and a ring agrees with the equivalent
    /// slice.
    #[test]
    fn shared_composition_helpers_are_consistent() {
        let token_to_root: Vec<u8> = (0..256u32)
            .map(|t| (t % GROUP_ORDER as u32) as u8)
            .collect();
        assert_eq!(
            compose_context_roots(&token_to_root, &[]),
            group_table().identity as usize
        );
        for t in [0usize, 7, 100, 255] {
            assert_eq!(compose_context_roots(&token_to_root, &[t]), t % GROUP_ORDER);
        }
        // A ring holding the same tokens in order must compose to the same element.
        let ctx = [3usize, 41, 200, 17, 88];
        let ring: Vec<u32> = ctx.iter().map(|&t| t as u32).collect();
        assert_eq!(
            compose_ring_roots(&token_to_root, &ring, ring.len() % ring.len(), ctx.len()),
            compose_context_roots(&token_to_root, &ctx)
        );
    }
}
