//! High-Dimensional VSA Context Engine.
//!
//! Multiplier-free sequence representation via bitwise XOR binding,
//! cyclic positional permutation, and parallel majority bundling.
//! Zero runtime heap allocations and zero runtime floats on serving hot paths.

use super::codebook::Codebook;
use super::hypervector::Hypervector;

/// Maximum static window capacity for the zero-allocation rolling context engine.
pub const DEFAULT_MAX_WINDOW: usize = 64;

/// Pure function: Encode an ordered sequence of tokens into a single n-gram hypervector.
///
/// Formula:
/// $$H = \bigoplus_{i=0}^{n-1} \rho^{n - 1 - i}(E(w_i))$$
///
/// Order-sensitive: Permuting token order produces mutually quasi-orthogonal hypervectors.
pub fn encode_ngram<const WORDS: usize>(
    tokens: &[u32],
    codebook: &Codebook<WORDS>,
) -> Hypervector<WORDS> {
    let n = tokens.len();
    if n == 0 {
        return Hypervector::zero();
    }
    if n == 1 {
        return codebook.get(tokens[0]);
    }

    let mut bound = Hypervector::zero();
    for (i, &tok) in tokens.iter().enumerate() {
        let lag = n - 1 - i;
        let base = codebook.get(tok);
        let permuted = base.permute(lag);
        bound = bound.bind(&permuted);
    }
    bound
}

/// Pure function: Encode context by bundling positional tokens across a sliding window.
///
/// Each token at distance $k$ from the present is rotated by $k$ bits, then all
/// positions are bundled via parallel majority voting.
pub fn encode_positional_context<const WORDS: usize>(
    tokens: &[u32],
    codebook: &Codebook<WORDS>,
) -> Hypervector<WORDS> {
    let n = tokens.len();
    if n == 0 {
        return Hypervector::zero();
    }
    if n == 1 {
        return codebook.get(tokens[0]);
    }

    // Process up to 64 vectors on stack with zero heap allocation
    let mut buffer = [Hypervector::zero(); 64];
    let count = n.min(64);
    let start_idx = n - count;

    for i in 0..count {
        let tok = tokens[start_idx + i];
        let lag = count - 1 - i;
        buffer[i] = codebook.get(tok).permute(lag);
    }

    Hypervector::bundle(&buffer[..count])
}

/// Pure function: Holographic Reduced Representation (HRR) Associative Transition Context.
///
/// Eliminates the broken lag-0 orthogonality by binding adjacent context transitions:
/// $$H_{\text{trans}} = \bigoplus_{i=0}^{t-2} E(w_i) \otimes \rho^1(E(w_{i+1}))$$
///
/// For querying continuation of current token $w_t$:
/// Unbind with $E(w_t)$ and inverse-rotate by 1 bit:
/// $$Q = H_{\text{trans}} \otimes E(w_t)$$
/// $$\hat{E}_{\text{next}} = \rho^{-1}(Q)$$
///
/// Evaluating continuation candidate $v$ via bipolar dot product $\hat{E}_{\text{next}} \cdot E(v)$
/// yields real associative context similarity without matrix multiplications or floats.
pub fn encode_associative_transition_context<const WORDS: usize>(
    tokens: &[u32],
    codebook: &Codebook<WORDS>,
) -> Hypervector<WORDS> {
    let n = tokens.len();
    if n == 0 {
        return Hypervector::zero();
    }
    if n == 1 {
        // Single-token context has no transition history; permute by 1 so it does not self-attract
        return if let Some(v) = codebook.get_ref(tokens[0]) {
            v.permute(1)
        } else {
            codebook.get(tokens[0]).permute(1)
        };
    }

    let count = n.min(DEFAULT_MAX_WINDOW);
    let start_idx = n - count;
    let window = &tokens[start_idx..];
    let num_trans = count - 1;

    let mut unique_trans = [Hypervector::zero(); DEFAULT_MAX_WINDOW];
    let mut unique_count = 0usize;

    for i in 0..num_trans {
        let w_a = window[i];
        let w_b = window[i + 1];
        let v_a = if let Some(v) = codebook.get_ref(w_a) {
            *v
        } else {
            codebook.get(w_a)
        };
        let v_b = if let Some(v) = codebook.get_ref(w_b) {
            *v
        } else {
            codebook.get(w_b)
        };
        // T_i = E(w_i) (XOR) rho^1(E(w_{i+1}))
        let t = v_a.bind(&v_b.permute(1));

        // Deduplicate transitions so repetitive loops do not overwrite minority transitions in majority vote
        let mut exists = false;
        let mut j = 0;
        while j < unique_count {
            if unique_trans[j] == t {
                exists = true;
                break;
            }
            j += 1;
        }
        if !exists && unique_count < DEFAULT_MAX_WINDOW {
            unique_trans[unique_count] = t;
            unique_count += 1;
        }
    }

    if unique_count == 0 {
        return Hypervector::zero();
    }

    // H_trans = bundle of unique transition key-value bindings
    let h_trans = Hypervector::bundle(&unique_trans[..unique_count]);

    // Current token w_t
    let w_curr = window[count - 1];
    let v_curr = if let Some(v) = codebook.get_ref(w_curr) {
        *v
    } else {
        codebook.get(w_curr)
    };

    // Unbind: Q = H_trans (XOR) E(w_t)
    let q = h_trans.bind(&v_curr);

    // Inverse rotate by 1 bit: E_hat_next = rho^{-1}(Q)
    q.permute_inv(1)
}

/// Pure function: Encode context using HRR associative transition binding and unbinding.
#[inline]
pub fn encode_multiscale_context<const WORDS: usize>(
    tokens: &[u32],
    codebook: &Codebook<WORDS>,
    _max_order: usize,
) -> Hypervector<WORDS> {
    encode_associative_transition_context(tokens, codebook)
}

/// Pure function: Encode context combining multiscale associative transitions with 4-head orthogonal VSA attention.
#[inline]
pub fn encode_attended_multiscale_context<const WORDS: usize>(
    tokens: &[u32],
    codebook: &Codebook<WORDS>,
    max_order: usize,
) -> Hypervector<WORDS> {
    let multiscale = encode_multiscale_context(tokens, codebook, max_order);
    let attn = super::attention::MultiHeadVsaAttention::<WORDS>::new();
    let attn_res = attn.attend(tokens, codebook);
    if attn_res.active_heads > 0 {
        let mut bundle_list = [Hypervector::zero(); 1 + super::attention::NUM_ATTENTION_HEADS];
        bundle_list[0] = multiscale;
        let mut count = 1;
        for h in 0..super::attention::NUM_ATTENTION_HEADS {
            if attn_res.head_predictions[h].count_ones() > 0 {
                bundle_list[count] = attn_res.head_predictions[h];
                count += 1;
            }
        }
        let tie_breaker =
            Hypervector::<WORDS>::from_seed(super::attention::CANONICAL_ATTENTION_SEED, 0x71E_B8EA);
        Hypervector::bundle_with_tie_breaker(&bundle_list[..count], &tie_breaker)
    } else {
        multiscale
    }
}

/// Zero-allocation rolling context engine for streaming token generation.
///
/// Fixed-size ring buffer guarantees zero dynamic heap allocations on serving hot paths.
#[derive(Clone, Debug)]
pub struct RollingVsaContext<const WORDS: usize, const MAX_WINDOW: usize = DEFAULT_MAX_WINDOW> {
    pub ring: [u32; MAX_WINDOW],
    pub cursor: usize,
    pub length: usize,
}

impl<const WORDS: usize, const MAX_WINDOW: usize> Default for RollingVsaContext<WORDS, MAX_WINDOW> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const WORDS: usize, const MAX_WINDOW: usize> RollingVsaContext<WORDS, MAX_WINDOW> {
    /// Initialize an empty rolling context engine.
    pub const fn new() -> Self {
        Self {
            ring: [0u32; MAX_WINDOW],
            cursor: 0,
            length: 0,
        }
    }

    /// Reset context state.
    pub fn clear(&mut self) {
        self.cursor = 0;
        self.length = 0;
    }

    /// Ingest a newly observed or generated token.
    #[inline]
    pub fn push(&mut self, token: u32) {
        self.ring[self.cursor] = token;
        self.cursor = (self.cursor + 1) & (MAX_WINDOW - 1);
        if self.length < MAX_WINDOW {
            self.length += 1;
        }
    }

    /// Retrieve token at lag $k$ (1 = most recent token, 2 = 2 tokens ago, etc.).
    #[inline]
    pub fn recent(&self, lag: usize) -> Option<u32> {
        if lag == 0 || lag > self.length {
            None
        } else {
            let idx = (self.cursor + MAX_WINDOW - lag) & (MAX_WINDOW - 1);
            Some(self.ring[idx])
        }
    }

    /// Current number of stored tokens in context ($0 \le \text{length} \le \text{MAX\_WINDOW}$).
    #[inline]
    pub fn len(&self) -> usize {
        self.length
    }

    /// Check whether context is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Fill a slice with the most recent $k$ tokens in chronological order.
    pub fn copy_recent(&self, out: &mut [u32]) -> usize {
        let count = out.len().min(self.length);
        for i in 0..count {
            let lag = count - i;
            out[i] = self.recent(lag).unwrap_or(0);
        }
        count
    }

    /// Encode the active n-gram context of order $n$ (up to `length`).
    pub fn current_ngram(&self, codebook: &Codebook<WORDS>, n: usize) -> Hypervector<WORDS> {
        let ord = n.min(self.length);
        if ord == 0 {
            return Hypervector::zero();
        }

        let mut buf = [0u32; MAX_WINDOW];
        let count = ord.min(MAX_WINDOW);
        for i in 0..count {
            let lag = count - i;
            buf[i] = self.recent(lag).unwrap_or(0);
        }
        encode_ngram(&buf[..count], codebook)
    }

    /// Encode the holographic associative context predicting next token $\hat{E}_{\text{next}}$.
    pub fn current_context_hypervector(
        &self,
        codebook: &Codebook<WORDS>,
        _max_order: usize,
    ) -> Hypervector<WORDS> {
        if self.length == 0 {
            return Hypervector::zero();
        }

        let mut buf = [0u32; MAX_WINDOW];
        let count = self.copy_recent(&mut buf);
        encode_associative_transition_context(&buf[..count], codebook)
    }

    /// Execute 4-head orthogonal VSA context attention over the rolling ring buffer.
    pub fn current_multihead_attention(
        &self,
        codebook: &Codebook<WORDS>,
        attention: &super::attention::MultiHeadVsaAttention<WORDS>,
    ) -> super::attention::MultiHeadVsaResult<WORDS> {
        if self.length == 0 {
            return super::attention::MultiHeadVsaResult::empty();
        }
        let mut buf = [0u32; MAX_WINDOW];
        let count = self.copy_recent(&mut buf);
        attention.attend(&buf[..count], codebook)
    }

    /// Encode the combined attended context hypervector over the rolling ring buffer.
    pub fn current_attended_context_hypervector(
        &self,
        codebook: &Codebook<WORDS>,
        max_order: usize,
    ) -> Hypervector<WORDS> {
        if self.length == 0 {
            return Hypervector::zero();
        }
        let mut buf = [0u32; MAX_WINDOW];
        let count = self.copy_recent(&mut buf);
        encode_attended_multiscale_context(&buf[..count], codebook, max_order)
    }

    /// Score a candidate continuation token in $Q1.15$ fixed-point format against
    /// the expected next-token hypervector using overlap similarity $[0, 32767]$.
    #[inline]
    pub fn score_continuation_q15(
        &self,
        codebook: &Codebook<WORDS>,
        target_context: &Hypervector<WORDS>,
        candidate_token: u32,
    ) -> i16 {
        let cand_vec = codebook.get(candidate_token);
        target_context.similarity_q15(&cand_vec)
    }

    /// Evaluate candidate $v$ via bipolar dot product $\hat{E}_{\text{next}} \cdot E(v)$ in $Q1.15$
    /// fixed-point format $[-32767, 32767]$ for real associative context similarity.
    ///
    /// Strictly integer arithmetic: zero runtime floats, zero matrix multiplications.
    #[inline]
    pub fn score_continuation_bipolar_q15(
        &self,
        codebook: &Codebook<WORDS>,
        target_context: &Hypervector<WORDS>,
        candidate_token: u32,
    ) -> i16 {
        let cand_vec = codebook.get(candidate_token);
        target_context.bipolar_correlation_q15(&cand_vec)
    }
}
