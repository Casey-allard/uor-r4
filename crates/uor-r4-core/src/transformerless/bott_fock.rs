//! Bott periodic Fock space context abstraction (issue #107; spec:
//! `docs/r4_furey_quantum_geometric_plan.md` §Phase 4).
//!
//! Folds an arbitrary N-token context window into one fixed 16x16 integer
//! state matrix. The construction is O(1) in space and update time **by
//! construction**: the state is a fixed `[i16; 256]` array, every append
//! performs the same fixed sweep of 256 integer operations, and nothing
//! is allocated — there is no KV cache to grow.
//!
//! The update is the integer-kernel form of the 8-periodicity tensor
//! contraction Cl(s,t) ≅ Cl(s0,t0) ⊗ M16(R)^⊗n: a contractive decay
//! (each entry sheds a fixed fraction) plus a token-mixing injection
//! (the embedding folded across every matrix cell with position-sensitive
//! rotations). Contraction keeps the state bounded for unbounded N; the
//! fold is lossy — an abstraction of the context, not a reversible
//! encoding.
//!
//! Operator discipline (P-4 spirit): XOR, rotate, arithmetic shift, and
//! saturating add/sub only — no multiply, divide, modulo, or float in
//! the update path. Machine-checked by the self-scan test in this file.

/// Token embedding width folded into the state (one M16(R) axis).
pub const CONTEXT_DIM: usize = 16;
/// Folded state entries: the 16x16 M16(R) context matrix, row-major.
pub const STATE_ENTRIES: usize = 256;

/// Fixed-size O(1) context store: the folded M16(R) state matrix plus a
/// monotone token counter. No heap, no growable state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BottFockContextStore {
    state: [i16; STATE_ENTRIES],
    token_count: u64,
}

impl Default for BottFockContextStore {
    fn default() -> Self {
        Self::new()
    }
}

impl BottFockContextStore {
    /// Zero state, no tokens folded.
    pub fn new() -> Self {
        Self {
            state: [0; STATE_ENTRIES],
            token_count: 0,
        }
    }

    /// Fold one 16-wide integer token embedding into the context state.
    ///
    /// Fixed work per call: one sweep of the 256 state cells. Decay keeps
    /// the state contractive (each cell sheds a quarter of its value) and
    /// the injection is saturated, so the state stays bounded for
    /// unbounded token counts.
    pub fn append_token(&mut self, token: &[i16; CONTEXT_DIM]) {
        let mut idx = 0usize;
        let mut row = 0usize;
        while row < CONTEXT_DIM {
            let row_embedding = token[row];
            let mut col = 0usize;
            while col < CONTEXT_DIM {
                // Position-sensitive injection: the row embedding rotated
                // by the column index, XOR-mixed with the column
                // embedding. Row index folds into the cell address via
                // shift (the matrix is 16 = 2^4 wide).
                let rotated = row_embedding.rotate_left((col & 15) as u32);
                let injection = rotated ^ token[col];
                // Contractive decay: cell <- cell - cell/4 (arithmetic
                // shift), then inject a quarter of the mix, saturated.
                let decayed = self.state[idx] - (self.state[idx] >> 2);
                self.state[idx] = decayed.saturating_add(injection >> 2);
                idx += 1;
                col += 1;
            }
            row += 1;
        }
        self.token_count += 1;
    }

    /// Number of tokens folded into the state so far.
    pub fn token_count(&self) -> u64 {
        self.token_count
    }

    /// The folded 16x16 context state matrix (row-major).
    pub fn state(&self) -> &[i16; STATE_ENTRIES] {
        &self.state
    }

    /// Order- and content-sensitive checksum of the folded state, for
    /// determinism checks. XOR/rotate fold; no allocation.
    pub fn checksum(&self) -> u64 {
        let mut acc = self.token_count;
        let mut idx = 0usize;
        while idx < STATE_ENTRIES {
            let cell = self.state[idx] as u16 as u64;
            acc = acc.rotate_left(7) ^ cell;
            idx += 1;
        }
        acc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_embedding(seed: u64, out: &mut [i16; CONTEXT_DIM]) {
        // Deterministic xorshift fill; no RNG state outside the caller.
        let mut x = seed | 1;
        let mut i = 0usize;
        while i < CONTEXT_DIM {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            out[i] = x as i16;
            i += 1;
        }
    }

    #[test]
    fn append_is_deterministic_and_counts_tokens() {
        let mut a = BottFockContextStore::new();
        let mut b = BottFockContextStore::new();
        let mut token = [0i16; CONTEXT_DIM];
        let mut n = 0u64;
        while n < 1_000 {
            sample_embedding(n, &mut token);
            a.append_token(&token);
            b.append_token(&token);
            n += 1;
        }
        assert_eq!(a, b, "same token stream folds to the same state");
        assert_eq!(a.token_count(), 1_000);
        assert_eq!(a.checksum(), b.checksum());
        assert_ne!(a.checksum(), BottFockContextStore::new().checksum());
    }

    #[test]
    fn state_converges_under_repeated_embedding() {
        let mut store = BottFockContextStore::new();
        let extremes = [
            i16::MIN,
            i16::MAX,
            i16::MIN,
            i16::MAX,
            0,
            -1,
            1,
            0,
            i16::MAX,
            i16::MIN,
            0,
            -1,
            1,
            0,
            i16::MAX,
            i16::MIN,
        ];
        let mut n = 0u64;
        while n < 100_000 {
            store.append_token(&extremes);
            n += 1;
        }
        // Contractive decay plus bounded injection reaches a fixed point:
        // folding the same embedding further no longer moves the state.
        let converged = store.state().to_owned();
        let mut n = 0u64;
        while n < 1_000 {
            store.append_token(&extremes);
            n += 1;
        }
        assert_eq!(store.state(), &converged, "state reaches a fixed point");
    }

    #[test]
    fn order_changes_the_fold() {
        let mut first = BottFockContextStore::new();
        let mut second = BottFockContextStore::new();
        let mut t0 = [0i16; CONTEXT_DIM];
        let mut t1 = [0i16; CONTEXT_DIM];
        sample_embedding(11, &mut t0);
        sample_embedding(22, &mut t1);
        first.append_token(&t0);
        first.append_token(&t1);
        second.append_token(&t1);
        second.append_token(&t0);
        assert_ne!(first.checksum(), second.checksum());
    }

    /// P-4-style self scan: the update path uses no multiplication,
    /// division, or modulo operator on values. #787 E-c: delegates to the
    /// shared `source_scan` implementation instead of carrying its own
    /// weaker copy (the old inline scanner lacked string-aware comment
    /// stripping and the method-form needles).
    #[test]
    fn p4_update_source_scan() {
        let src = include_str!("bott_fock.rs");
        let outcome = crate::transformerless::source_scan::scan_for_forbidden_arith(src);
        assert!(
            outcome.offenders.is_empty(),
            "arithmetic operators in the update path:\n{}",
            outcome.offenders.join("\n")
        );
        assert!(
            outcome.allowed.is_empty(),
            "bott_fock has no sanctioned allowances:\n{}",
            outcome.allowed.join("\n")
        );
    }
}
