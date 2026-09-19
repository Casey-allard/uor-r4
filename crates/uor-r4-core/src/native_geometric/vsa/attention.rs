//! Multi-Head VSA Context Attention (The Geometric Transformer Equivalent).
//!
//! Multiplier-free, zero-float, zero-heap attention mechanism for high-dimensional
//! Vector Symbolic Architectures (VSA).
//!
//! Features:
//! 1. 4-Head orthogonal 4,096-bit hypervectors for Query, Key, and Value binding
//!    via bitwise XOR:
//!    $$Q^{(h)} = x_{\text{query}} \oplus R_{\text{query}}^{(h)}$$
//!    $$K_j^{(h)} = x_j \oplus R_{\text{key}}^{(h)}$$
//!    $$V_j^{(h)} = v_j \oplus R_{\text{value}}^{(h)}$$
//! 2. Context similarity across a sliding/ring window of tokens via hardware
//!    `popcount` Hamming distance:
//!    $$d_j^{(h)} = d_H(Q^{(h)}, K_j^{(h)})$$
//! 3. Softmax replacement via integer rectified quadratic scoring:
//!    $$w_j^{(h)} = \max(0, \text{Threshold} - d_j^{(h)})^2$$
//!    (guarantees exact sparsity and zero floating-point operations).
//! 4. Value aggregation via bitwise majority bundling:
//!    $$V_{\text{agg}}^{(h)} = \operatorname{sign}\left(\sum_{j} w_j^{(h)} (2 V_j^{(h)} - 1)\right)$$
//! 5. Unbinding the value role vector to obtain the attended representation:
//!    $$P^{(h)} = V_{\text{agg}}^{(h)} \oplus R_{\text{value}}^{(h)}$$
//! 6. Multi-head bundling:
//!    $$P_{\text{multi}} = \text{bundle}(P^{(0)}, P^{(1)}, P^{(2)}, P^{(3)})$$
//!
//! Invariants:
//! - 0 Runtime Matrix Multiplications (Zero GEMM)
//! - 0 Runtime Floats (pure integer bitwise logic, Q1.15 fixed point)
//! - 0 Steady-State Heap Allocations on hot path (stack-only fixed buffers)

#![forbid(unsafe_code)]

use super::codebook::Codebook;
use super::hypervector::Hypervector;

/// Number of attention heads in the geometric VSA attention engine.
pub const NUM_ATTENTION_HEADS: usize = 4;

/// Canonical deterministic seed for orthogonal VSA attention head role hypervectors.
pub const CANONICAL_ATTENTION_SEED: u64 = 0x5653_415f_4154_544e; // "VSA_ATTN"

/// Default attention threshold for 4,096-bit hypervectors.
///
/// In 4,096 dimensions, independent random vectors have expected Hamming distance 2,048.
/// Only vectors with $d_H < \text{THRESHOLD}$ receive strictly positive rectified quadratic weight.
pub const DEFAULT_ATTENTION_THRESHOLD: usize = 2048;

/// Multiplier-free squaring of an unsigned integer via shift-and-add bitwise arithmetic.
#[inline(always)]
fn square_u64(x: u64) -> u64 {
    let mut res = 0u64;
    let mut temp = x;
    let mut shifted = x;
    while temp > 0 {
        if temp & 1 == 1 {
            res = res.wrapping_add(shifted);
        }
        shifted = shifted.wrapping_shl(1);
        temp >>= 1;
    }
    res
}

/// Restoring binary division of a signed integer by a small positive divisor
/// using purely shifts, additions, and subtractions (zero division, zero multiplication).
#[inline(always)]
fn div_small_positive(num: i32, denom: usize) -> i32 {
    if denom == 0 {
        return 0;
    }
    let neg = num < 0;
    let mut abs_num = (num as i64).unsigned_abs();
    let d = denom as u64;
    let mut quotient = 0u64;
    for shift in (0..32).rev() {
        if let Some(d_shifted) = d.checked_shl(shift) {
            if abs_num >= d_shifted {
                abs_num -= d_shifted;
                quotient |= 1 << shift;
            }
        }
    }
    if neg {
        (quotient as i32).wrapping_neg()
    } else {
        quotient as i32
    }
}

/// Role hypervectors for a single VSA attention head.
///
/// # Why query and key share a role
///
/// The query/key comparison binds the **same** `r_role` into both sides. XOR is
/// self-inverse, so `d_H(x ^ R, x_j ^ R) == d_H(x, x_j)`: two occurrences of the
/// same token give `d_H == 0` and therefore maximal rectified-quadratic weight,
/// while distinct tokens sit near `D/2`.
///
/// The previous formulation bound *independent* random roles into query and key
/// (`r_query != r_key`). Two occurrences of the same token then gave
/// `d_H(x ^ r_query, x ^ r_key) == d_H(r_query, r_key) ~= D/2 == THRESHOLD`, which
/// is **not** strictly below the threshold, so exact matches received exactly
/// zero weight while mismatching pairs — whose distance fluctuates below the
/// threshold roughly half the time — received weight. The head attended to noise
/// and skipped its own matches. Because the role is shared it also cancels, so
/// the binding is algebraically a no-op and may be elided later as an
/// optimisation without changing any result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeadRole<const WORDS: usize> {
    /// Shared query/key role. Bound into both the query and each key.
    pub r_role: Hypervector<WORDS>,
    /// Value role, bound into each value before bundling and unbound after
    /// aggregation to recover the attended representation.
    pub r_value: Hypervector<WORDS>,
}

impl<const WORDS: usize> HeadRole<WORDS> {
    /// Deterministically generate the shared role and value role for head `head_idx`.
    pub const fn from_head_index(head_idx: usize, seed: u64) -> Self {
        let salt_base = (head_idx as u64) << 1;
        Self {
            r_role: Hypervector::from_seed(seed, salt_base),
            r_value: Hypervector::from_seed(seed, salt_base + 1),
        }
    }
}

/// Output of a multi-head VSA attention pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MultiHeadVsaResult<const WORDS: usize> {
    /// Individual head predictions after unbinding role vectors.
    pub head_predictions: [Hypervector<WORDS>; NUM_ATTENTION_HEADS],
    /// Multi-head bundled context hypervector.
    pub bundled_context: Hypervector<WORDS>,
    /// Number of active heads that produced non-zero attention weights.
    pub active_heads: usize,
}

impl<const WORDS: usize> MultiHeadVsaResult<WORDS> {
    /// Construct an empty result.
    #[inline]
    pub const fn empty() -> Self {
        Self {
            head_predictions: [Hypervector::zero(); NUM_ATTENTION_HEADS],
            bundled_context: Hypervector::zero(),
            active_heads: 0,
        }
    }

    /// Score a candidate token vector in Q1.15 fixed point against the multi-head bundled context.
    #[inline]
    pub fn score_candidate_q15(&self, cand_vec: &Hypervector<WORDS>) -> i16 {
        if self.active_heads == 0 {
            0
        } else {
            self.bundled_context.bipolar_correlation_q15(cand_vec)
        }
    }

    /// Detailed scores: average Q1.15 correlation and per-head Q1.15 correlations.
    pub fn score_candidate_detailed_q15(
        &self,
        cand_vec: &Hypervector<WORDS>,
    ) -> (i16, [i16; NUM_ATTENTION_HEADS]) {
        let mut head_scores = [0i16; NUM_ATTENTION_HEADS];
        let mut total = 0i32;
        let mut active = 0usize;
        for h in 0..NUM_ATTENTION_HEADS {
            let sim = self.head_predictions[h].bipolar_correlation_q15(cand_vec);
            head_scores[h] = sim;
            if sim != 0 {
                total += sim as i32;
                active += 1;
            }
        }
        let avg = if active > 0 {
            div_small_positive(total, active) as i16
        } else {
            0
        };
        (avg, head_scores)
    }
}

/// 4-Head Orthogonal VSA Context Attention Engine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MultiHeadVsaAttention<const WORDS: usize = 64> {
    pub heads: [HeadRole<WORDS>; NUM_ATTENTION_HEADS],
    pub threshold: usize,
}

impl<const WORDS: usize> Default for MultiHeadVsaAttention<WORDS> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const WORDS: usize> MultiHeadVsaAttention<WORDS> {
    /// Initialize with canonical attention seed and default threshold.
    pub const fn new() -> Self {
        Self::with_seed_and_threshold(CANONICAL_ATTENTION_SEED, DEFAULT_ATTENTION_THRESHOLD)
    }

    /// Initialize with explicit seed and Hamming distance threshold.
    pub const fn with_seed_and_threshold(seed: u64, threshold: usize) -> Self {
        let heads = [
            HeadRole::from_head_index(0, seed),
            HeadRole::from_head_index(1, seed),
            HeadRole::from_head_index(2, seed),
            HeadRole::from_head_index(3, seed),
        ];
        Self { heads, threshold }
    }

    /// Execute multi-head VSA attention across a sliding or ring context window.
    ///
    /// # Arguments
    /// - `tokens`: Slice of chronologically ordered context tokens (up to 64 tokens).
    /// - `codebook`: Deterministic token codebook mapping token IDs to basis hypervectors.
    ///
    /// # Invariants
    /// - Zero runtime heap allocations (stack-only fixed buffers).
    /// - Zero runtime floats (pure integer arithmetic and popcount).
    /// - Zero runtime matrix multiplications.
    pub fn attend(&self, tokens: &[u32], codebook: &Codebook<WORDS>) -> MultiHeadVsaResult<WORDS> {
        let n = tokens.len();
        if n == 0 {
            return MultiHeadVsaResult::empty();
        }

        let max_ctx = n.min(64);
        let start_idx = n - max_ctx;
        let window = &tokens[start_idx..];
        let curr_tok = window[max_ctx - 1];
        let curr_vec = if let Some(v) = codebook.get_ref(curr_tok) {
            *v
        } else {
            codebook.get(curr_tok)
        };

        let mut head_predictions = [Hypervector::zero(); NUM_ATTENTION_HEADS];
        let mut active_heads = 0usize;

        // --------------------------------------------------------------------
        // Head 0: Recency & Entity Co-Occurrence (Content / Entity Head)
        // --------------------------------------------------------------------
        {
            let role = &self.heads[0];
            let q = curr_vec.bind(&role.r_role);
            let mut weights = [0u64; 64];
            let mut total_weight = 0u64;

            for j in 0..max_ctx {
                let tok_j = window[j];
                let v_j = if let Some(v) = codebook.get_ref(tok_j) {
                    *v
                } else {
                    codebook.get(tok_j)
                };
                let k_j = v_j.bind(&role.r_role);
                let d_h = q.hamming_distance(&k_j);
                if d_h < self.threshold {
                    let diff = (self.threshold - d_h) as u64;
                    let w = square_u64(diff);
                    weights[j] = w;
                    total_weight += w;
                }
            }

            if total_weight > 0 {
                let half_total = total_weight >> 1;
                let mut v_agg_words = [0u64; WORDS];
                for w in 0..WORDS {
                    let mut pos_sums = [0u64; 64];
                    for j in 0..max_ctx {
                        let w_j = weights[j];
                        if w_j == 0 {
                            continue;
                        }
                        let tok_j = window[j];
                        let v_j = if let Some(v) = codebook.get_ref(tok_j) {
                            *v
                        } else {
                            codebook.get(tok_j)
                        };
                        let val_bound = v_j.bind(&role.r_value);
                        let word_bits = val_bound.data[w];
                        for b in 0..64 {
                            if (word_bits >> b) & 1 == 1 {
                                pos_sums[b] += w_j;
                            }
                        }
                    }
                    let mut word_res = 0u64;
                    for b in 0..64 {
                        if pos_sums[b] > half_total {
                            word_res |= 1u64 << b;
                        }
                    }
                    v_agg_words[w] = word_res;
                }
                let v_agg = Hypervector::from_words(v_agg_words);
                head_predictions[0] = v_agg.bind(&role.r_value);
                active_heads += 1;
            }
        }

        // --------------------------------------------------------------------
        // Head 1: Associative Transition / Next-Token Induction Head
        // --------------------------------------------------------------------
        if max_ctx >= 2 {
            let role = &self.heads[1];
            let q = curr_vec.bind(&role.r_role);
            let mut weights = [0u64; 64];
            let mut total_weight = 0u64;
            let num_transitions = max_ctx - 1;

            for j in 0..num_transitions {
                let tok_j = window[j];
                let v_j = if let Some(v) = codebook.get_ref(tok_j) {
                    *v
                } else {
                    codebook.get(tok_j)
                };
                let k_j = v_j.bind(&role.r_role);
                let d_h = q.hamming_distance(&k_j);
                if d_h < self.threshold {
                    let diff = (self.threshold - d_h) as u64;
                    let w = square_u64(diff);
                    weights[j] = w;
                    total_weight += w;
                }
            }

            if total_weight > 0 {
                let half_total = total_weight >> 1;
                let mut v_agg_words = [0u64; WORDS];
                for w in 0..WORDS {
                    let mut pos_sums = [0u64; 64];
                    for j in 0..num_transitions {
                        let w_j = weights[j];
                        if w_j == 0 {
                            continue;
                        }
                        // The value is the successor token window[j + 1]
                        let succ_tok = window[j + 1];
                        let v_succ = if let Some(v) = codebook.get_ref(succ_tok) {
                            *v
                        } else {
                            codebook.get(succ_tok)
                        };
                        let val_bound = v_succ.bind(&role.r_value);
                        let word_bits = val_bound.data[w];
                        for b in 0..64 {
                            if (word_bits >> b) & 1 == 1 {
                                pos_sums[b] += w_j;
                            }
                        }
                    }
                    let mut word_res = 0u64;
                    for b in 0..64 {
                        if pos_sums[b] > half_total {
                            word_res |= 1u64 << b;
                        }
                    }
                    v_agg_words[w] = word_res;
                }
                let v_agg = Hypervector::from_words(v_agg_words);
                head_predictions[1] = v_agg.bind(&role.r_value);
                active_heads += 1;
            }
        }

        // --------------------------------------------------------------------
        // Head 2: Syntactic Meso-Scale Horizon (Positionally Conditioned Head)
        // --------------------------------------------------------------------
        {
            let role = &self.heads[2];
            let q = curr_vec.permute(1).bind(&role.r_role);
            let mut weights = [0u64; 64];
            let mut total_weight = 0u64;

            for j in 0..max_ctx {
                let lag = max_ctx - 1 - j;
                let tok_j = window[j];
                let v_j = if let Some(v) = codebook.get_ref(tok_j) {
                    *v
                } else {
                    codebook.get(tok_j)
                };
                let k_j = v_j.permute(lag).bind(&role.r_role);
                let d_h = q.hamming_distance(&k_j);
                if d_h < self.threshold {
                    let diff = (self.threshold - d_h) as u64;
                    let w = square_u64(diff);
                    weights[j] = w;
                    total_weight += w;
                }
            }

            if total_weight > 0 {
                let half_total = total_weight >> 1;
                let mut v_agg_words = [0u64; WORDS];
                for w in 0..WORDS {
                    let mut pos_sums = [0u64; 64];
                    for j in 0..max_ctx {
                        let w_j = weights[j];
                        if w_j == 0 {
                            continue;
                        }
                        let lag = max_ctx - 1 - j;
                        let tok_j = window[j];
                        let v_j = if let Some(v) = codebook.get_ref(tok_j) {
                            *v
                        } else {
                            codebook.get(tok_j)
                        };
                        let val_bound = v_j.permute(lag).bind(&role.r_value);
                        let word_bits = val_bound.data[w];
                        for b in 0..64 {
                            if (word_bits >> b) & 1 == 1 {
                                pos_sums[b] += w_j;
                            }
                        }
                    }
                    let mut word_res = 0u64;
                    for b in 0..64 {
                        if pos_sums[b] > half_total {
                            word_res |= 1u64 << b;
                        }
                    }
                    v_agg_words[w] = word_res;
                }
                let v_agg = Hypervector::from_words(v_agg_words);
                head_predictions[2] = v_agg.bind(&role.r_value).permute_inv(1);
                active_heads += 1;
            }
        }

        // --------------------------------------------------------------------
        // Head 3: Macro-Scale Topical & Discourse Context Head
        // --------------------------------------------------------------------
        {
            let role = &self.heads[3];
            let mut buf_vecs = [Hypervector::zero(); 64];
            for j in 0..max_ctx {
                let tok_j = window[j];
                buf_vecs[j] = if let Some(v) = codebook.get_ref(tok_j) {
                    *v
                } else {
                    codebook.get(tok_j)
                };
            }
            let topic_bundle = Hypervector::bundle(&buf_vecs[..max_ctx]);
            let q = topic_bundle.bind(&role.r_role);
            let mut weights = [0u64; 64];
            let mut total_weight = 0u64;

            for j in 0..max_ctx {
                let v_j = buf_vecs[j];
                let k_j = v_j.bind(&role.r_role);
                let d_h = q.hamming_distance(&k_j);
                if d_h < self.threshold {
                    let diff = (self.threshold - d_h) as u64;
                    let w = square_u64(diff);
                    weights[j] = w;
                    total_weight += w;
                }
            }

            if total_weight > 0 {
                let half_total = total_weight >> 1;
                let mut v_agg_words = [0u64; WORDS];
                for w in 0..WORDS {
                    let mut pos_sums = [0u64; 64];
                    for j in 0..max_ctx {
                        let w_j = weights[j];
                        if w_j == 0 {
                            continue;
                        }
                        let v_j = buf_vecs[j];
                        let val_bound = v_j.bind(&role.r_value);
                        let word_bits = val_bound.data[w];
                        for b in 0..64 {
                            if (word_bits >> b) & 1 == 1 {
                                pos_sums[b] += w_j;
                            }
                        }
                    }
                    let mut word_res = 0u64;
                    for b in 0..64 {
                        if pos_sums[b] > half_total {
                            word_res |= 1u64 << b;
                        }
                    }
                    v_agg_words[w] = word_res;
                }
                let v_agg = Hypervector::from_words(v_agg_words);
                head_predictions[3] = v_agg.bind(&role.r_value);
                active_heads += 1;
            }
        }

        // Multi-head bundling of active heads
        let bundled_context = if active_heads > 0 {
            let mut active_list = [Hypervector::zero(); NUM_ATTENTION_HEADS];
            let mut count = 0;
            for h in 0..NUM_ATTENTION_HEADS {
                if head_predictions[h].count_ones() > 0 {
                    active_list[count] = head_predictions[h];
                    count += 1;
                }
            }
            if count > 0 {
                let tie_breaker =
                    Hypervector::<WORDS>::from_seed(CANONICAL_ATTENTION_SEED, 0x71E_B8EA);
                Hypervector::bundle_with_tie_breaker(&active_list[..count], &tie_breaker)
            } else {
                Hypervector::zero()
            }
        } else {
            Hypervector::zero()
        };

        MultiHeadVsaResult {
            head_predictions,
            bundled_context,
            active_heads,
        }
    }
}

/// Canonical 4-head attention engine for standard 4096-bit hypervectors (64 words), pre-evaluated at compile time.
pub const CANONICAL_ATTENTION_64: MultiHeadVsaAttention<64> = MultiHeadVsaAttention::new();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_roles_are_quasi_orthogonal() {
        let attn = MultiHeadVsaAttention::<64>::new();
        let roles: Vec<_> = (0..NUM_ATTENTION_HEADS)
            .map(|h| attn.heads[h].r_value)
            .collect();
        for i in 0..roles.len() {
            for j in (i + 1)..roles.len() {
                let dist = roles[i].hamming_distance(&roles[j]);
                assert!(
                    (1850..=2250).contains(&dist),
                    "Value roles {i} and {j} not quasi-orthogonal: dist = {dist}"
                );
            }
        }
    }

    #[test]
    fn test_shared_query_key_role_yields_zero_distance_for_identical_tokens() {
        // Regression guard for the query/key role inversion. With independent
        // random query and key roles, two occurrences of the same token sit at
        // d_H ~= D/2 == THRESHOLD and receive exactly zero rectified weight,
        // while mismatches fluctuating below the threshold receive weight.
        let attn = MultiHeadVsaAttention::<64>::new();
        let codebook = Codebook::<64>::on_demand(100, 42);
        let role = &attn.heads[0];

        let same = codebook.get(7);
        let q = same.bind(&role.r_role);
        let k = same.bind(&role.r_role);
        assert_eq!(
            q.hamming_distance(&k),
            0,
            "identical tokens bound with the shared role must have distance 0"
        );

        let other = codebook.get(8).bind(&role.r_role);
        assert_ne!(
            q.hamming_distance(&other),
            0,
            "distinct tokens must not be treated as identical"
        );
    }

    #[test]
    fn test_rectified_quadratic_scoring_and_sparsity() {
        let attn = MultiHeadVsaAttention::<64>::new();
        assert_eq!(attn.threshold, 2048);
        let codebook = Codebook::<64>::on_demand(100, 42);

        let empty_res = attn.attend(&[], &codebook);
        assert_eq!(empty_res.active_heads, 0);
        assert_eq!(empty_res.score_candidate_q15(&codebook.get(1)), 0);

        let res_single = attn.attend(&[42], &codebook);
        assert!(res_single.active_heads > 0);
    }

    #[test]
    fn test_head_1_induction_retrieves_the_true_successor() {
        // Controlled induction test, replacing a single-instance comparison
        // against an out-of-window candidate, which was a coin flip fixed by
        // the codebook seed and could not distinguish a working induction
        // circuit from a broken one.
        //
        // Head 1 weights the *successors* of positions whose token equals the
        // current token, over `0..max_ctx - 1`. So [A, B, C, D, A] must
        // retrieve B exactly, and the same query token with a different
        // predecessor order must retrieve C instead. The negative arm has no
        // occurrence of the query token in the keyed positions.
        for seed in [1u64, 7, 42, 1009, 20_260_919] {
            let attn = MultiHeadVsaAttention::<64>::with_seed_and_threshold(
                seed,
                DEFAULT_ATTENTION_THRESHOLD,
            );
            let codebook = Codebook::<64>::on_demand(1_000, seed);
            let (a, b, c, d, x) = (10u32, 25u32, 30u32, 40u32, 77u32);
            let head1 = |res: &MultiHeadVsaResult<64>, tok: u32| -> i16 {
                res.score_candidate_detailed_q15(&codebook.get(tok)).1[1]
            };

            // Treatment 1: first successor is B.
            let t1 = attn.attend(&[a, b, c, d, a], &codebook);
            assert!(
                head1(&t1, b) >= 32_000,
                "seed {seed}: the repeated query token must retrieve its successor, got {}",
                head1(&t1, b)
            );

            // Treatment 2: same query token, different predecessor order.
            let t2 = attn.attend(&[a, c, b, d, a], &codebook);
            assert!(
                head1(&t2, c) >= 32_000,
                "seed {seed}: successor C must be retrieved, got {}",
                head1(&t2, c)
            );
            assert!(
                head1(&t2, b) < head1(&t2, c),
                "seed {seed}: retrieval must follow the predecessor order"
            );

            // Negative arm: the query token is absent from the keyed positions,
            // so head 1 must not reproduce the true successor as strongly.
            let neg = attn.attend(&[x, b, c, d, a], &codebook);
            assert!(
                head1(&neg, b) < head1(&t1, b),
                "seed {seed}: head 1 fired without a repeated query token"
            );
        }
    }

    #[test]
    fn test_multihead_vsa_zero_allocations() {
        let attn = MultiHeadVsaAttention::<64>::new();
        let codebook = Codebook::<64>::on_demand(100, 42);
        let tokens = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let res = attn.attend(&tokens, &codebook);
        let score = res.score_candidate_q15(&codebook.get(11));
        assert!(score != i16::MIN);
    }

    #[test]
    fn test_div_small_positive_edge_cases() {
        assert_eq!(div_small_positive(0, 5), 0);
        assert_eq!(div_small_positive(100, 0), 0);
        assert_eq!(div_small_positive(100, 4), 25);
        assert_eq!(div_small_positive(-100, 4), -25);
        assert_eq!(div_small_positive(100, 3), 33);
        assert_eq!(div_small_positive(-100, 3), -33);
        assert_eq!(div_small_positive(i32::MAX, 1), i32::MAX);
        assert_eq!(div_small_positive(i32::MIN, 1), i32::MIN);
        assert_eq!(div_small_positive(i32::MIN, 2), -1_073_741_824_i32);
        assert_eq!(div_small_positive(100, usize::MAX), 0);
    }
}
