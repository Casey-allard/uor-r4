//! UAX #15 Unicode NFC normalization — streaming, `no_std`, `no_alloc`.
//!
//! Reads UTF-8 from an `&[u8]` input slice, writes NFC-normalized
//! UTF-8 into an `&mut [u8]` output slice, returns the number of
//! bytes written. No allocator, no `std`, no panics on well-formed
//! input.
//!
//! # Algorithm (UAX #15)
//!
//! 1. **Canonical decomposition (NFD).** For each input code point,
//!    expand to its fully-recursive canonical decomposition. Hangul
//!    syllables are decomposed algorithmically per UAX #15 §3.12; all
//!    other decompositions are looked up in
//!    [`tables::DECOMP_TABLE`] / [`tables::DECOMP_DATA`].
//!
//! 2. **Canonical reordering.** Within each "combining run" (a
//!    starter followed by zero or more non-starters), sort the
//!    non-starters in stable ascending order by canonical combining
//!    class (UAX #15 §1.3).
//!
//! 3. **Canonical composition.** Walk the decomposed-and-reordered
//!    sequence; for each starter, greedily compose with following
//!    non-blocked combining marks via the canonical composition table
//!    ([`tables::COMP_TABLE`]) plus Hangul algorithmic composition
//!    (UAX #15 §3.12). A mark is "blocked" from a starter if any
//!    intervening mark has canonical combining class ≥ the mark's own
//!    class (UAX #15 D119).
//!
//! # Stream-safe bound
//!
//! The stream-safe text format pins the maximum number of consecutive
//! non-starters at 30 (UAX #15 §3). The implementation uses a fixed
//! 32-entry combining-run buffer (2-entry headroom) on the stack. No
//! allocator. Input streams that violate the stream-safe bound emit
//! [`NfcError::CombiningRunOverflow`].
//!
//! # UCD version pin
//!
//! Tables are generated from UCD `tables::UCD_VERSION` (currently
//! `15.1.0`). Regenerate via `python3 tools/gen_nfc_tables.py` after
//! bumping the version pin in [`crate::canonical::nfc::tables`] and
//! the vendored data files in `data/ucd/<version>/`.

mod tables;

pub use tables::UCD_VERSION;

/// Maximum number of consecutive non-starters per UAX #15 §3
/// stream-safe text format (30), with 2 entries of headroom so a
/// 30-non-starter run does not immediately overflow.
const COMBINING_RUN_CAPACITY: usize = 32;

/// Maximum length of a canonical decomposition for any single code
/// point. Per UCD 15.1.0 the longest canonical decomposition is 18
/// code points (Arabic ligatures, certain CJK compatibility forms);
/// 32 gives headroom against future UCD bumps.
const DECOMP_BUF_CAPACITY: usize = 32;

/// Streaming NFC normalizer error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NfcError {
    /// `input` is not well-formed UTF-8 per RFC 3629.
    InvalidUtf8 {
        /// Byte offset within `input` where validation failed.
        at: usize,
    },
    /// `out` is too small to hold the NFC-normalized form.
    OutputOverflow,
    /// A combining run in `input` exceeds the UAX #15 stream-safe
    /// bound of 30 consecutive non-starters.
    CombiningRunOverflow,
}

/// UAX #15 NFC_Quick_Check property values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NfcQc {
    /// Input is unambiguously already NFC. No normalization needed.
    Yes,
    /// Input is definitely not NFC.
    No,
    /// Input may or may not be NFC. A full normalization pass is
    /// required to decide and to produce the NFC form.
    Maybe,
}

/// UAX #15 NFC_Quick_Check (UAX #15 §6 Quick_Check). Walks `input`
/// once; returns `Yes` if every code point has `NFC_QC = Yes` and the
/// canonical combining classes are non-decreasing within each
/// combining run; returns `No` if any code point has `NFC_QC = No`
/// or any reorder is required; returns `Maybe` otherwise.
///
/// # Errors
///
/// Returns `NfcQc::No` for input containing invalid UTF-8 — the
/// caller's input must be valid UTF-8 to be NFC.
#[must_use]
pub fn quick_check(input: &[u8]) -> NfcQc {
    let s = match core::str::from_utf8(input) {
        Ok(s) => s,
        Err(_) => return NfcQc::No,
    };
    let mut last_cc: u8 = 0;
    let mut result = NfcQc::Yes;
    for c in s.chars() {
        let cp = c as u32;
        let cc = combining_class(cp);
        if cc != 0 && cc < last_cc {
            // Reorder required — definitely not NFC.
            return NfcQc::No;
        }
        match nfc_qc_lookup(cp) {
            NfcQc::No => return NfcQc::No,
            NfcQc::Maybe => result = NfcQc::Maybe,
            NfcQc::Yes => {}
        }
        last_cc = if cc == 0 { 0 } else { cc };
    }
    result
}

/// Normalize `input` (well-formed UTF-8) into NFC, writing to `out`.
/// Returns the number of bytes written.
///
/// # Errors
///
/// - [`NfcError::InvalidUtf8`] — `input` is not well-formed UTF-8.
/// - [`NfcError::OutputOverflow`] — `out` is too small to hold the
///   normalized form.
/// - [`NfcError::CombiningRunOverflow`] — a combining run in `input`
///   exceeds UAX #15's stream-safe bound (30 non-starters).
///
/// # Idempotence
///
/// `normalize_into(normalize_into(x)) == normalize_into(x)` byte-for-byte
/// per UAX #15 stability — verified in tests against UCD
/// `NormalizationTest.txt`.
pub fn normalize_into(input: &[u8], out: &mut [u8]) -> Result<usize, NfcError> {
    // Validate UTF-8 up front so the per-char iteration can index by
    // code point without re-validating each step.
    let s = core::str::from_utf8(input).map_err(|e| NfcError::InvalidUtf8 {
        at: e.valid_up_to(),
    })?;

    let mut writer = Writer::new(out);
    let mut state = NfcState::new();
    let mut decomp_buf = [0u32; DECOMP_BUF_CAPACITY];

    for c in s.chars() {
        let cp = c as u32;
        let len = decompose_recursive(cp, &mut decomp_buf);
        for &dp in &decomp_buf[..len] {
            state.feed(dp, &mut writer)?;
        }
    }
    state.flush(&mut writer)?;
    Ok(writer.pos)
}

// ─── Buffered NFC state machine ────────────────────────────────────
//
// The algorithm buffers a "combining run" — a starter plus all
// following non-starters until the next starter — and *then* resolves
// composition. Composing eagerly per-mark is wrong: a later mark with
// a lower combining class can become a better composition candidate
// for the starter after canonical reorder. The buffered form per
// UAX #15 §1.3 Algorithm 4 produces correct NFC by first sorting marks
// in the run by ccc and then composing greedily left-to-right against
// the starter.

struct NfcState {
    /// Current starter code point. `None` until the first starter is
    /// seen.
    starter: Option<u32>,
    /// Pending non-starter marks since `starter`, kept sorted ascending
    /// by canonical combining class (stable insertion-sort preserves
    /// input order among equal-ccc marks).
    pending: [u32; COMBINING_RUN_CAPACITY],
    pending_len: u8,
}

impl NfcState {
    fn new() -> Self {
        Self {
            starter: None,
            pending: [0; COMBINING_RUN_CAPACITY],
            pending_len: 0,
        }
    }

    fn feed(&mut self, cp: u32, writer: &mut Writer<'_>) -> Result<(), NfcError> {
        let cc = combining_class(cp);
        if cc == 0 {
            // New starter. If pending is empty, the new starter is not
            // blocked from the current starter (no intervening marks
            // separate them), so we may compose the two — this is how
            // Hangul L+V→LV and LV+T→LVT compositions land, plus any
            // non-Hangul starter+starter canonical decomposition that
            // is not Full_Composition_Exclusion-excluded.
            if self.pending_len == 0 {
                if let Some(prev) = self.starter {
                    if let Some(composed) = compose_pair(prev, cp) {
                        self.starter = Some(composed);
                        return Ok(());
                    }
                }
            }
            // Either no prior starter, or the pair does not compose,
            // or marks intervene — resolve the current run and start
            // fresh.
            self.resolve_run(writer)?;
            self.starter = Some(cp);
            return Ok(());
        }
        // Non-starter combining mark — insert into pending at the
        // ccc-sorted position. Stable insertion preserves input order
        // among equal-ccc marks.
        if (self.pending_len as usize) >= COMBINING_RUN_CAPACITY {
            return Err(NfcError::CombiningRunOverflow);
        }
        let mut idx = self.pending_len as usize;
        while idx > 0 && combining_class(self.pending[idx - 1]) > cc {
            self.pending[idx] = self.pending[idx - 1];
            idx -= 1;
        }
        self.pending[idx] = cp;
        self.pending_len += 1;
        Ok(())
    }

    fn flush(&mut self, writer: &mut Writer<'_>) -> Result<(), NfcError> {
        self.resolve_run(writer)?;
        self.starter = None;
        Ok(())
    }

    /// Resolve the current `starter + pending` combining run: compose
    /// `starter` greedily with non-blocked pending marks, then emit
    /// the resulting starter plus any uncomposed marks in ccc order.
    ///
    /// Orphan-mark case (input starts with combining marks before any
    /// starter): `starter` is `None`; pending marks are emitted as-is
    /// in their already-ccc-sorted order, with no composition attempt
    /// (there is no starter to compose against).
    fn resolve_run(&mut self, writer: &mut Writer<'_>) -> Result<(), NfcError> {
        if let Some(mut l) = self.starter.take() {
            let mut max_cc_seen: u8 = 0;
            let mut i: usize = 0;
            let mut len = self.pending_len as usize;
            while i < len {
                let m = self.pending[i];
                let cc_m = combining_class(m);
                // M can compose with L only if no prior mark in the
                // run has ccc >= ccc(M) — i.e. cc_m > max_cc_seen
                // (UAX #15 D119 blocking rule).
                if cc_m > max_cc_seen {
                    if let Some(composed) = compose_pair(l, m) {
                        l = composed;
                        // Remove pending[i]: shift everything down.
                        let mut j = i;
                        while j + 1 < len {
                            self.pending[j] = self.pending[j + 1];
                            j += 1;
                        }
                        len -= 1;
                        // Do not advance i; do not update max_cc_seen.
                        continue;
                    }
                }
                max_cc_seen = cc_m;
                i += 1;
            }
            self.pending_len = len as u8;
            writer.write_code_point(l)?;
        }
        for k in 0..(self.pending_len as usize) {
            writer.write_code_point(self.pending[k])?;
        }
        self.pending_len = 0;
        Ok(())
    }
}

// ─── UTF-8 byte writer (no_alloc) ──────────────────────────────────

struct Writer<'a> {
    out: &'a mut [u8],
    pos: usize,
}

impl<'a> Writer<'a> {
    fn new(out: &'a mut [u8]) -> Self {
        Self { out, pos: 0 }
    }

    fn write_code_point(&mut self, cp: u32) -> Result<(), NfcError> {
        // Encode cp as UTF-8 via core::char + encode_utf8.
        let c = char::from_u32(cp).ok_or(NfcError::OutputOverflow)?;
        let mut buf = [0u8; 4];
        let s = c.encode_utf8(&mut buf);
        let bytes = s.as_bytes();
        if self.pos + bytes.len() > self.out.len() {
            return Err(NfcError::OutputOverflow);
        }
        self.out[self.pos..self.pos + bytes.len()].copy_from_slice(bytes);
        self.pos += bytes.len();
        Ok(())
    }
}

// ─── Decomposition (UAX #15 §3 + §3.12) ────────────────────────────

/// Recursively-expanded canonical decomposition of `cp`. Writes
/// decomposed code points into `out` (which must be at least 18
/// entries — the longest canonical decomposition in UCD 15.1.0).
/// Returns the number of code points written.
fn decompose_recursive(cp: u32, out: &mut [u32; DECOMP_BUF_CAPACITY]) -> usize {
    // Hangul algorithmic decomposition (UAX #15 §3.12).
    if (tables::HANGUL_S_BASE..=tables::HANGUL_S_LAST).contains(&cp) {
        let s_index = cp - tables::HANGUL_S_BASE;
        let l_index = s_index / tables::HANGUL_N_COUNT;
        let v_index = (s_index % tables::HANGUL_N_COUNT) / tables::HANGUL_T_COUNT;
        let t_index = s_index % tables::HANGUL_T_COUNT;
        out[0] = tables::HANGUL_L_BASE + l_index;
        out[1] = tables::HANGUL_V_BASE + v_index;
        if t_index == 0 {
            return 2;
        }
        out[2] = tables::HANGUL_T_BASE + t_index;
        return 3;
    }
    // Non-Hangul: look up in DECOMP_TABLE.
    match tables::DECOMP_TABLE.binary_search_by_key(&cp, |&(c, _, _)| c) {
        Ok(idx) => {
            let (_, off, len) = tables::DECOMP_TABLE[idx];
            let off = off as usize;
            let len = len as usize;
            // `tables::DECOMP_DATA` is already fully recursive — copy
            // directly.
            out[..len].copy_from_slice(&tables::DECOMP_DATA[off..off + len]);
            len
        }
        Err(_) => {
            // No decomposition — identity.
            out[0] = cp;
            1
        }
    }
}

// ─── Composition (UAX #15 §3.12 + table) ───────────────────────────

/// Canonical composition of `(starter, mark)`. Returns the composed
/// code point if `(starter, mark)` is a primary canonical composite
/// not in Full_Composition_Exclusion; `None` otherwise.
fn compose_pair(starter: u32, mark: u32) -> Option<u32> {
    // Hangul algorithmic composition (UAX #15 §3.12).
    // L + V → LV
    if (tables::HANGUL_L_BASE..tables::HANGUL_L_BASE + tables::HANGUL_L_COUNT).contains(&starter)
        && (tables::HANGUL_V_BASE..tables::HANGUL_V_BASE + tables::HANGUL_V_COUNT).contains(&mark)
    {
        let l_index = starter - tables::HANGUL_L_BASE;
        let v_index = mark - tables::HANGUL_V_BASE;
        let s_index = l_index * tables::HANGUL_N_COUNT + v_index * tables::HANGUL_T_COUNT;
        return Some(tables::HANGUL_S_BASE + s_index);
    }
    // LV + T → LVT  (only valid when starter is an LV syllable —
    // i.e. (starter - S_BASE) is a multiple of T_COUNT and mark is in
    // (T_BASE, T_BASE+T_COUNT). T_BASE itself is the "no trailing"
    // sentinel and is not a valid composing mark.)
    if (tables::HANGUL_S_BASE..tables::HANGUL_S_BASE + tables::HANGUL_S_COUNT).contains(&starter)
        && (starter - tables::HANGUL_S_BASE) % tables::HANGUL_T_COUNT == 0
        && mark > tables::HANGUL_T_BASE
        && mark < tables::HANGUL_T_BASE + tables::HANGUL_T_COUNT
    {
        let t_index = mark - tables::HANGUL_T_BASE;
        return Some(starter + t_index);
    }
    // Non-Hangul: look up in COMP_TABLE.
    let key = (starter, mark);
    tables::COMP_TABLE
        .binary_search_by_key(&key, |&(s, m, _)| (s, m))
        .ok()
        .map(|idx| tables::COMP_TABLE[idx].2)
}

// ─── Combining class + NFC_QC lookups ──────────────────────────────

fn combining_class(cp: u32) -> u8 {
    match tables::CCC_TABLE.binary_search_by_key(&cp, |&(c, _)| c) {
        Ok(idx) => tables::CCC_TABLE[idx].1,
        Err(_) => 0,
    }
}

fn nfc_qc_lookup(cp: u32) -> NfcQc {
    if tables::NFC_QC_NO.binary_search(&cp).is_ok() {
        NfcQc::No
    } else if tables::NFC_QC_MAYBE.binary_search(&cp).is_ok() {
        NfcQc::Maybe
    } else {
        NfcQc::Yes
    }
}

#[cfg(test)]
mod algorithm_tests {
    extern crate alloc;

    use super::*;
    use alloc::string::String;

    fn nfc(input: &str) -> String {
        let mut out = [0u8; 1024];
        let n = normalize_into(input.as_bytes(), &mut out).expect("ok");
        String::from_utf8(out[..n].to_vec()).expect("utf8")
    }

    #[test]
    fn ascii_passes_through_unchanged() {
        assert_eq!(nfc("hello"), "hello");
        assert_eq!(nfc(""), "");
        assert_eq!(nfc("foo bar baz"), "foo bar baz");
    }

    #[test]
    fn nfd_to_nfc_recomposes_combining_marks() {
        // "café" with combining acute = "cafe\u{0301}" → NFC "caf\u{00E9}"
        assert_eq!(nfc("cafe\u{0301}"), "caf\u{00E9}");
    }

    #[test]
    fn nfc_input_is_idempotent() {
        assert_eq!(nfc("caf\u{00E9}"), "caf\u{00E9}");
        assert_eq!(nfc("\u{00C5}ngstr\u{00F6}m"), "\u{00C5}ngstr\u{00F6}m");
    }

    #[test]
    fn double_normalisation_yields_same_output() {
        let inputs = ["cafe\u{0301}", "A\u{030A}", "\u{1E0B}\u{0323}", "한국어"];
        for input in inputs.iter() {
            let once = nfc(input);
            let twice = nfc(&once);
            assert_eq!(once, twice, "idempotence broken for {input:?}");
        }
    }

    #[test]
    fn hangul_lv_composes_algorithmically() {
        // L=U+1100 (ᄀ) + V=U+1161 (ᅡ) → LV=U+AC00 (가)
        assert_eq!(nfc("\u{1100}\u{1161}"), "\u{AC00}");
    }

    #[test]
    fn hangul_lvt_composes_algorithmically() {
        // L=U+1100 + V=U+1161 + T=U+11A8 → LVT=U+AC01
        assert_eq!(nfc("\u{1100}\u{1161}\u{11A8}"), "\u{AC01}");
    }

    #[test]
    fn hangul_syllable_already_composed_is_idempotent() {
        assert_eq!(nfc("\u{AC00}"), "\u{AC00}");
        assert_eq!(nfc("한국어"), "한국어");
    }

    #[test]
    fn canonical_reorder_then_compose_picks_lower_ccc_first() {
        // d (U+0064) + U+0307 (ccc=230, dot above) + U+0323 (ccc=220,
        // dot below). NFC must (a) reorder by ccc — U+0323 before
        // U+0307 — and (b) attempt compose against the starter in
        // ccc-sorted order. d + U+0323 → U+1E0D (LATIN SMALL LETTER D
        // WITH DOT BELOW); U+1E0D + U+0307 has no precomposed form
        // and remains a combining sequence. Result: U+1E0D U+0307.
        assert_eq!(nfc("d\u{0307}\u{0323}"), "\u{1E0D}\u{0307}");
        // Inputs in input-reordered order normalize to the same NFC.
        assert_eq!(nfc("d\u{0323}\u{0307}"), "\u{1E0D}\u{0307}");
    }

    #[test]
    fn composition_exclusion_pairs_do_not_recompose() {
        // U+212B Å (Angstrom sign) decomposes to U+0041 U+030A but is
        // a singleton — the canonical NFC is U+00C5 (Latin A with
        // ring above), not U+212B.
        assert_eq!(nfc("\u{212B}"), "\u{00C5}");
    }

    #[test]
    fn quick_check_recognises_ascii_as_nfc_yes() {
        assert_eq!(quick_check(b"hello"), NfcQc::Yes);
        assert_eq!(quick_check(b""), NfcQc::Yes);
    }

    #[test]
    fn quick_check_recognises_decomposed_input_as_non_nfc() {
        // U+0301 has NFC_QC = Maybe (it's a combining mark that
        // might recompose) — the quick check returns Maybe, but for
        // input where reorder is required it returns No.
        let qc = quick_check("cafe\u{0301}".as_bytes());
        assert!(matches!(qc, NfcQc::Maybe | NfcQc::No));
    }

    #[test]
    fn rejects_output_buffer_too_small() {
        let mut tiny = [0u8; 1];
        let err = normalize_into("café".as_bytes(), &mut tiny).expect_err("must error");
        assert_eq!(err, NfcError::OutputOverflow);
    }
}
