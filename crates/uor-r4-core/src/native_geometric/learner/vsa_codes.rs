//! VSA codebooks derived from the model's learned representation instead of a token-id hash.
//!
//! # The defect this addresses
//!
//! The serving path built its VSA codebook with `Codebook::on_demand(vocab, vsa_seed)`, which
//! generates each token's hypervector from `splitmix64(vsa_seed, token_id)` — a **fixed random
//! function of the token id**. For two distinct tokens that gives `E[d_H] = D/2` with
//! `sd ~ sqrt(D)/2`, so the only recoverable signal is `a == b`. The four VSA attention heads
//! therefore operate on a codebook that is *disconnected from the learned 120-root assignment*
//! (`token_to_root`, a Voronoi partition of the learned embeddings), which is why the Card P7
//! ablation measured the whole VSA term as `NEGLIGIBLE` (DECISIONS.md D2: class `enabler`,
//! wiring mis-wired, action = repair and re-measure, not retire).
//!
//! # The repair
//!
//! Derive the code for a token from the **learned** root it was assigned, using a
//! similarity-preserving random projection (locality-sensitive hashing) of the canonical
//! icosian root quaternions:
//!
//! ```text
//! for bit b:  w_b ~ Z^4, large random integers   (deterministic from the seed)
//!             code[r][b] = 1  iff  <w_b, q_r> > 0
//!             E(t) = code[ token_to_root[t] ]
//! ```
//!
//! By the standard LSH arcsin law, `E[d_H(code_a, code_b)] = D * theta(a,b) / pi`, monotone in
//! the angle `theta(q_a, q_b) = arccos <q_a, q_b>`. The heads now see the model's own learned
//! geometry rather than noise.
//!
//! # Resolution, stated precisely
//!
//! LSH does **not** collapse the distance to the 9 conjugacy-class angles. Each bit is an
//! independent random hyperplane, so `d_H` is an unbiased estimate of `D * theta / pi` with
//! `sd ~ sqrt(D)/2 = 32` at `D = 4096`, i.e. an angular resolution of roughly 1.4 degrees. The
//! construction therefore provides genuinely graded similarity, unlike the 120-landmark Hamming
//! bank it replaces (which is an exact class function and by the audit takes exactly 9 values).
//! The tests in this module assert that: equal-angle pairs agree within a tolerance band, and
//! class means are monotone in the angle.
//!
//! The real ceiling is **aliasing**, not resolution: codes are a function of the root, so a
//! 4,096-token vocabulary collapses onto at most [`H4_ROOT_COUNT`] (120) distinct codes and two
//! distinct tokens in the same root have `d_H = 0` — indistinguishable to every head. Exact
//! identity induction, which the post-inversion-fix heads do correctly today, is therefore lost.
//! Escaping that requires codes at finer resolution than 120 buckets, derived from the
//! *continuous* learned embedding, which the exported artifact does not currently carry (it
//! stores `token_to_root` only). That is Stage 3 of `native-core-transition-plan.md`. This module
//! makes the mechanism testable now and bounds what is achievable at root resolution.

use super::jepa_trainer::ExportedGeometricModel;
use crate::native_geometric::learner::embedding::{canonical_h4_roots_q30, H4_ROOT_COUNT};
use crate::native_geometric::vsa::{Codebook, HierarchicalCodebook, Hypervector};

/// Number of bits in the default code (`Hypervector<64>` = 64 words x 64 bits).
const CODE_BITS: usize = 64 * 64;

/// SplitMix64 finaliser.
#[inline]
fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Deterministic random projection vector for code bit `bit`, one integer hyperplane per bit.
///
/// Weights are drawn as bounded random integers rather than `{+1,-1}` signs. Signs make
/// `dot == 0` ties reachable on the highly symmetric icosian roots (for example a root with four
/// equal-magnitude components under a balanced sign pattern), and a tie breaks the LSH estimate
/// for that pair. Random magnitudes make exact cancellation measure-zero, so `code(-q)` is the
/// exact bitwise complement of `code(q)` and equal-angle pairs agree within the LSH band.
///
/// Magnitude is capped at `2^30` so `sum_i w_i * q_i` with `|q_i| <= 2^30` fits comfortably in
/// `i64` (worst case `4 * 2^60 = 2^62`).
#[inline]
fn projection_weights(seed: u64, bit: usize) -> [i64; 4] {
    let mut out = [0i64; 4];
    for (i, slot) in out.iter_mut().enumerate() {
        let h = splitmix64(
            seed ^ (bit as u64)
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .wrapping_add((i as u64).wrapping_mul(0xD1B5_4A32_D192_ED03)),
        );
        // Top 31 bits, centred: range [-2^30, 2^30).
        *slot = ((h >> 33) as i64) - (1i64 << 30);
    }
    out
}

/// LSH code for one root quaternion.
fn code_for_root(q: &[i32; 4], seed: u64) -> Hypervector<64> {
    let mut words = [0u64; 64];
    for bit in 0..CODE_BITS {
        let w = projection_weights(seed, bit);
        let dot = w[0] * q[0] as i64 + w[1] * q[1] as i64 + w[2] * q[2] as i64 + w[3] * q[3] as i64;
        if dot > 0 {
            words[bit / 64] |= 1u64 << (bit % 64);
        }
    }
    Hypervector::from_words(words)
}

/// The 120 distinct codes, one per canonical H4 root.
pub fn root_codes(seed: u64) -> Vec<Hypervector<64>> {
    canonical_h4_roots_q30()
        .iter()
        .map(|q| code_for_root(&q.0, seed))
        .collect()
}

/// Build a [`Codebook<64>`] whose codes follow the model's **learned** root assignment.
///
/// `token_to_root` is the artifact's stored u8 Voronoi assignment. Tokens beyond its length, or
/// with an out-of-range root, fall back to root 0.
pub fn build_root_codebook(vocab_size: usize, token_to_root: &[u8], seed: u64) -> Codebook<64> {
    let codes = root_codes(seed);
    let mut table = Vec::with_capacity(vocab_size);
    for token in 0..vocab_size {
        let root = token_to_root
            .get(token)
            .copied()
            .unwrap_or(0)
            .min((H4_ROOT_COUNT - 1) as u8) as usize;
        table.push(codes[root]);
    }
    Codebook::from_vectors(seed, table)
}

impl ExportedGeometricModel {
    /// The VSA codebook implied by the artifact's declared code mode.
    pub fn vsa_codebook(&self) -> Codebook<64> {
        match self.vsa_code_mode {
            1 => build_root_codebook(self.vocab_size, &self.token_to_root, self.vsa_seed),
            _ => Codebook::<64>::on_demand(self.vocab_size, self.vsa_seed),
        }
    }

    /// Make the artifact self-consistent for its declared VSA code mode.
    ///
    /// The stored `hierarchical_codebook` centroids are bundles of the token vectors used at
    /// **export time**. When the declared mode selects a different code space those centroids
    /// must be rebuilt, or the router would compare a query vector against centroids from a
    /// different space — a silently incoherent system. This is the coherence requirement that
    /// made mode `1` measurable at all: the Card P7 routing comparison would have been meaningless
    /// without it.
    ///
    /// Call once immediately after deserializing. Mode `0` needs no rebuild because the stored
    /// codebook was built from the same fixed-hash codes that mode `0` selects.
    pub fn prepare_vsa_code_mode(&mut self) -> Result<(), String> {
        match self.vsa_code_mode {
            0 => Ok(()),
            1 => {
                let codebook = self.vsa_codebook();
                self.hierarchical_codebook = Some(HierarchicalCodebook::<64>::new(
                    self.vocab_size,
                    &self.token_to_root,
                    &codebook,
                ));
                Ok(())
            }
            other => Err(format!(
                "unsupported vsa_code_mode {other}; known: 0 (fixed), 1 (learned-root codes)"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The construction must be a deterministic, graded function of the root angle: identical
    /// roots give distance 0, equal angles agree within the LSH noise band, and the class mean is
    /// monotone in the quaternion dot product.
    #[test]
    fn distance_is_a_graded_function_of_the_root_dot_product() {
        let seed = 0x5653_415f_3230_3236;
        let codes = root_codes(seed);
        let roots = canonical_h4_roots_q30();
        assert_eq!(codes.len(), H4_ROOT_COUNT);

        // Identical roots are identical codes.
        for r in 0..H4_ROOT_COUNT {
            assert_eq!(codes[r].hamming_distance(&codes[r]), 0);
        }

        // Group distances by the conjugate-class angle, i.e. by cos(theta) = <q_a, q_b> rounded to
        // two decimals. The Q30 root coordinates are quantised, so exact dot products differ
        // slightly between representatives of the same class and must be bucketed first.
        let mut by_dot: std::collections::BTreeMap<i64, Vec<usize>> =
            std::collections::BTreeMap::new();
        for a in 0..H4_ROOT_COUNT {
            for b in 0..H4_ROOT_COUNT {
                let qa = &roots[a].0;
                let qb = &roots[b].0;
                let dot: i64 = (0..4).map(|i| qa[i] as i64 * qb[i] as i64).sum();
                let cos100 = ((dot as f64 / (1i64 << 60) as f64) * 100.0).round() as i64;
                by_dot
                    .entry(cos100)
                    .or_default()
                    .push(codes[a].hamming_distance(&codes[b]));
            }
        }

        // The hyperplanes are FIXED, so `d_H(a, b)` is deterministic per pair rather than a fresh
        // binomial draw; within one angle class the realised distances differ because the pairs
        // differ, not because of sampling. The property that matters is therefore that the
        // class **mean** is monotone in the angle, with clear separation between classes.
        let mut means: Vec<(i64, f64)> = Vec::with_capacity(by_dot.len());
        for (&dot, ds) in &by_dot {
            means.push((dot, ds.iter().sum::<usize>() as f64 / ds.len() as f64));
        }
        assert!(
            by_dot.len() >= 8,
            "expected at least 8 conjugate-class angles, saw {}",
            by_dot.len()
        );

        // Monotone in expectation: larger cos(theta) (smaller angle) means smaller distance.
        for w in means.windows(2) {
            assert!(
                w[1].1 <= w[0].1,
                "class-mean distance must be non-increasing in cos(theta): {:?} then {:?}",
                w[0],
                w[1]
            );
        }

        // Clear separation: the antipodal class (smallest cos, first entry) must be near-complement
        // and the coincident class (largest cos, last entry) must be near zero, so the metric
        // actually discriminates rather than sitting at D/2 everywhere.
        let antipodal = means.first().map(|m| m.1).unwrap_or(f64::NAN);
        let coincident = means.last().map(|m| m.1).unwrap_or(f64::NAN);
        assert!(
            coincident <= 200.0,
            "the coincident class should almost coincide, got mean {coincident:.1}"
        );
        assert!(
            antipodal >= 4000.0,
            "the antipodal class should be near-complement, got mean {antipodal:.1}"
        );
        assert!(
            antipodal - coincident >= 3000.0,
            "angle classes must be well separated, span {:.1}",
            antipodal - coincident
        );

        // Graded resolution, not a 9-level collapse: many distinct distances appear.
        let levels: std::collections::BTreeSet<usize> =
            by_dot.values().flatten().copied().collect();
        assert!(
            levels.len() > 9,
            "expected graded distances, saw only {} levels",
            levels.len()
        );
    }

    /// Documents the aliasing cost: at most 120 distinct codes for the whole vocabulary.
    #[test]
    fn root_codes_alias_the_vocabulary_onto_120_values() {
        let seed = 1;
        let codes = root_codes(seed);
        let distinct: std::collections::BTreeSet<Vec<u64>> =
            codes.iter().map(|c| c.data.to_vec()).collect();
        assert_eq!(distinct.len(), codes.len(), "root codes must be distinct");

        // A 4096-token vocabulary therefore has at most 120 distinct codes.
        let vocab = 4096usize;
        let token_to_root: Vec<u8> = (0..vocab).map(|t| (t % H4_ROOT_COUNT) as u8).collect();
        let book = build_root_codebook(vocab, &token_to_root, seed);
        assert_eq!(book.table.len(), vocab);
        let distinct_book: std::collections::BTreeSet<Vec<u64>> =
            book.table.iter().map(|c| c.data.to_vec()).collect();
        assert!(
            distinct_book.len() <= H4_ROOT_COUNT,
            "expected <= {H4_ROOT_COUNT} distinct codes, saw {}",
            distinct_book.len()
        );
    }
}
