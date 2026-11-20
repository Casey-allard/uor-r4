//! Host-side Pick reads over a framed token stream (claude-439).
//!
//! For every scored target of a stream of records `BOS(0) ... EOS(1)`, these
//! reads say whether a session-copy read fired, its class, and whether its
//! copied token equals the target. They mirror `pick_read` in
//! `uor-r4-integer`'s `discrete_d11.rs` with Identity transport, over a
//! per-token code table instead of transported orbit blocks.

use crate::{Error, Result};

const BOS: u16 = 0;
const EOS: u16 = 1;

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}

/// One scored target: `class` 0 = the read did not fire, 1 = fired at
/// distance 0, 2 = fired at distance above 0. `hit` is whether the copied
/// token equals the target (false for class 0). `pick` is window-relative.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PickRow {
    pub class: u8,
    pub hit: bool,
    pub pick: Option<u32>,
    pub distance: Option<u32>,
}

include!("kn_ceiling/runtime/code_pick.rs");

/// Exact pair Pick (control C2): the latest `p` in `1..t` with
/// `w[p-1] == w[t-1] && w[p] == w[t]`, or `None` (also when `t < 2`).
pub fn exact_pair_pick(w: &[u16]) -> Option<usize> {
    if w.len() < 3 {
        return None;
    }
    let t = w.len() - 1;
    (1..t).rev().find(|&p| w[p - 1] == w[t - 1] && w[p] == w[t])
}

/// One row per non-BOS position `T` of `stream`, in stream order. The window
/// for `T` is `stream[max(s, T - context) .. T]`, `s` the last BOS at or
/// before `T - 1`, so a read never crosses a record.
fn rows_by(
    stream: &[u16],
    context: usize,
    read: impl Fn(&[u16]) -> Option<(usize, u32)>,
) -> Result<Vec<PickRow>> {
    if !stream.is_empty() && stream[0] != BOS {
        return Err(invalid("Pick stream must start with BOS"));
    }
    let mut rows = Vec::with_capacity(stream.len());
    let mut s = 0usize;
    for target in 1..stream.len() {
        if stream[target - 1] == BOS {
            s = target - 1;
        }
        if stream[target] == BOS {
            continue;
        }
        let w = &stream[s.max(target.saturating_sub(context))..target];
        rows.push(match read(w) {
            None => PickRow {
                class: 0,
                hit: false,
                pick: None,
                distance: None,
            },
            Some((p, d)) => PickRow {
                class: if d == 0 { 1 } else { 2 },
                hit: w[p + 1] == stream[target],
                pick: Some(p as u32),
                distance: Some(d),
            },
        });
    }
    Ok(rows)
}

/// Code Pick rows for every non-BOS target of `stream` (see [`code_pick`]).
/// Refuses a token id `>= codes.len()` and a stream not starting with BOS.
pub fn code_pick_rows<const B: usize>(
    stream: &[u16],
    codes: &[[u16; B]],
    context: usize,
) -> Result<Vec<PickRow>> {
    if let Some(&bad) = stream.iter().find(|&&id| usize::from(id) >= codes.len()) {
        return Err(invalid(format!(
            "Pick token id {bad} outside the code table of {} rows",
            codes.len()
        )));
    }
    rows_by(stream, context, |w| {
        code_pick(codes, w).map(|(p, d, _)| (p, d))
    })
}

/// Exact pair Pick rows (control C2) for every non-BOS target of `stream`.
/// Refuses a token id `>= vocab` and a stream not starting with BOS.
pub fn exact_pair_rows(stream: &[u16], vocab: usize, context: usize) -> Result<Vec<PickRow>> {
    if let Some(&bad) = stream.iter().find(|&&id| usize::from(id) >= vocab) {
        return Err(invalid(format!(
            "Pick token id {bad} outside a vocabulary of {vocab}"
        )));
    }
    rows_by(stream, context, |w| exact_pair_pick(w).map(|p| (p, 0)))
}

/// Standard splitmix64 step (wrapping).
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Control C3: the code table with rows `2..len` shuffled (rows 0 and 1, BOS
/// and EOS, stay put). Fisher-Yates driven by splitmix64 seeded with `seed`:
/// for `i` from `len - 1` down to 3, draw `r` and `j = 2 + r % (i - 1)`, a
/// value in `[2, i]` (modulo draw, so very slightly non-uniform), and swap
/// rows `i` and `j`.
pub fn permute_code_rows<const B: usize>(codes: &[[u16; B]], seed: u64) -> Vec<[u16; B]> {
    row_permutation(codes.len(), seed)
        .into_iter()
        .map(|from| codes[from])
        .collect()
}

/// claude-439: the permutation `permute_code_rows` applies, as source row
/// indices: row i of the permuted table is row `perm[i]` of the original. The
/// same draw, so a device table permuted by it matches the permuted codes.
pub fn row_permutation(len: usize, seed: u64) -> Vec<usize> {
    let mut perm: Vec<usize> = (0..len).collect();
    let mut state = seed;
    for i in (3..len).rev() {
        let r = splitmix64(&mut state);
        let j = 2 + (r % (i as u64 - 1)) as usize;
        perm.swap(i, j);
    }
    perm
}

/// The P+ stream: each record `BOS body EOS` becomes `BOS c1 c1 c2 c2 ... ck
/// ck EOS`, the body cut into `chunk`-token pieces (the last may be shorter),
/// each followed at once by a verbatim copy. The flags hold one entry per
/// non-BOS output position, true exactly inside a second copy. Refuses
/// `chunk == 0`, a stream not starting with BOS, a BOS not preceded by EOS,
/// and a stream not ending with EOS.
pub fn repeat_chunks(stream: &[u16], chunk: usize) -> Result<(Vec<u16>, Vec<bool>)> {
    if chunk == 0 {
        return Err(invalid("repeat_chunks needs a chunk of at least one token"));
    }
    if stream.first() != Some(&BOS) {
        return Err(invalid("repeat_chunks stream must start with BOS"));
    }
    if stream.last() != Some(&EOS) || stream.len() < 2 {
        return Err(invalid("repeat_chunks stream must end with EOS"));
    }
    if (1..stream.len()).any(|k| stream[k] == BOS && stream[k - 1] != EOS) {
        return Err(invalid("repeat_chunks found a BOS not preceded by EOS"));
    }
    let mut starts: Vec<usize> = (0..stream.len()).filter(|&k| stream[k] == BOS).collect();
    starts.push(stream.len());
    let mut out = Vec::with_capacity(2 * stream.len());
    let mut second = Vec::with_capacity(2 * stream.len());
    for pair in starts.windows(2) {
        let body = &stream[pair[0] + 1..pair[1] - 1];
        out.push(BOS);
        for piece in body.chunks(chunk) {
            out.extend_from_slice(piece);
            second.extend(std::iter::repeat(false).take(piece.len()));
            out.extend_from_slice(piece);
            second.extend(std::iter::repeat(true).take(piece.len()));
        }
        out.push(EOS);
        second.push(false);
    }
    Ok((out, second))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Lcg(u64);

    impl Lcg {
        fn next(&mut self) -> u64 {
            let state = self.0;
            let state = state.wrapping_mul(6_364_136_223_846_793_005);
            self.0 = state.wrapping_add(1_442_695_040_888_963_407);
            self.0 >> 33
        }
        fn below(&mut self, n: u64) -> u64 {
            self.next() % n
        }
    }

    /// Distinct one-hot codes: token `i` has only bit `i` set.
    fn one_hot(vocab: usize) -> Vec<[u16; 1]> {
        (0..vocab).map(|i| [1u16 << i]).collect()
    }

    #[test]
    fn code_pick_matches_a_brute_force() {
        let mut rng = Lcg(439);
        for _ in 0..400 {
            let vocab = 2 + rng.below(7) as usize;
            let codes: Vec<[u16; 2]> = (0..vocab)
                .map(|_| [rng.below(1 << 16) as u16, rng.below(1 << 16) as u16])
                .collect();
            let len = rng.below(14) as usize;
            let w: Vec<u16> = (0..len).map(|_| rng.below(vocab as u64) as u16).collect();
            let got = code_pick(&codes, &w);
            if len < 3 {
                assert_eq!(got, None);
                continue;
            }
            let t = len - 1;
            let dist = |p: usize| -> u32 {
                let mut d = 0;
                for b in 0..2 {
                    let x = codes[w[p - 1] as usize][b] ^ codes[w[t - 1] as usize][b];
                    let y = codes[w[p] as usize][b] ^ codes[w[t] as usize][b];
                    d += x.count_ones() + y.count_ones();
                }
                d
            };
            let best = (1..t).map(dist).min().unwrap();
            let pick = (1..t).filter(|&p| dist(p) == best).max().unwrap();
            let ties = (1..t).filter(|&p| dist(p) == best).count() as u32;
            assert_eq!(got, Some((pick, best, ties)), "window {w:?}");
        }
    }

    #[test]
    fn code_pick_takes_the_latest_tie() {
        let codes = one_hot(8);
        // The window ends (2, 3); the pair occurs at p = 2 and p = 5.
        let w = [0u16, 2, 3, 4, 2, 3, 2, 3];
        let (p, d, ties) = code_pick(&codes, &w).unwrap();
        assert_eq!((p, d, ties), (5, 0, 2));
        assert_eq!(w[p + 1], 2);
    }

    #[test]
    fn the_pair_picks_the_right_copy() {
        let codes = one_hot(12);
        // Token 3 occurs at 2 (after 4, before 9) and at 5 (after 5, before
        // 7); the window ends (4, 3), so only the pair finds position 2.
        let stream = [0u16, 4, 3, 9, 5, 3, 7, 8, 4, 3, 9, 1];
        let w = &stream[..10];
        let (p, d, ties) = code_pick(&codes, w).unwrap();
        assert_eq!((p, d, ties), (2, 0, 1));
        assert_eq!(w[p + 1], 9);
        let rows = code_pick_rows(&stream, &codes, 192).unwrap();
        assert_eq!(rows.len(), stream.len() - 1);
        assert_eq!(
            rows[9],
            PickRow {
                class: 1,
                hit: true,
                pick: Some(2),
                distance: Some(0),
            }
        );
    }

    #[test]
    fn exact_pair_fires_only_on_a_match() {
        assert_eq!(exact_pair_pick(&[0, 4, 3, 9, 5, 3, 7, 4, 3]), Some(2));
        assert_eq!(exact_pair_pick(&[0, 4, 3, 9, 5, 3, 7, 6, 3]), None);
        assert_eq!(exact_pair_pick(&[0, 4, 3]), None);
        assert_eq!(exact_pair_pick(&[0, 4]), None);
        // The window end itself (p = t) never counts.
        assert_eq!(exact_pair_pick(&[0, 2, 7, 4, 3]), None);
        let stream = [0u16, 4, 3, 9, 5, 3, 7, 4, 3, 9, 1];
        let rows = exact_pair_rows(&stream, 10, 192).unwrap();
        assert_eq!(rows.len(), 10);
        for (k, row) in rows.iter().enumerate() {
            if k == 8 {
                assert_eq!(row.class, 1);
                assert!(row.hit);
                assert_eq!((row.pick, row.distance), (Some(2), Some(0)));
            } else if k == 9 {
                // EOS target: the pair (3, 9) matches at p = 3, copying 5.
                assert_eq!((row.class, row.hit, row.pick), (1, false, Some(3)));
            } else {
                assert_eq!(
                    *row,
                    PickRow {
                        class: 0,
                        hit: false,
                        pick: None,
                        distance: None
                    }
                );
            }
        }
        assert!(exact_pair_rows(&stream, 9, 192).is_err());
    }

    #[test]
    fn targets_never_cross_a_record() {
        let codes = one_hot(10);
        let stream = [0u16, 2, 3, 4, 5, 1, 0, 2, 3, 4, 5, 1];
        let pair = exact_pair_rows(&stream, 10, 192).unwrap();
        let code = code_pick_rows(&stream, &codes, 192).unwrap();
        let non_bos = stream.iter().filter(|&&x| x != BOS).count();
        assert_eq!(pair.len(), non_bos);
        assert_eq!(code.len(), non_bos);
        // Rows 5..10 are record two's targets 2, 3, 4, 5, EOS; in record one
        // the pair (2, 3) would copy 4, but no read reaches it.
        for row in &pair[5..] {
            assert_eq!(row.class, 0);
        }
        for row in &code[5..] {
            assert!(!row.hit);
        }
        assert_eq!(code[5].class, 0);
        assert_eq!(code[6].class, 0);
        // Context caps the window: the final 4's pair (2, 3) sits at 1..=2.
        let stream = [0u16, 2, 3, 4, 5, 6, 7, 2, 3, 4, 1];
        let reach = exact_pair_rows(&stream, 10, 8).unwrap();
        assert_eq!((reach[8].class, reach[8].hit), (1, true));
        let short = exact_pair_rows(&stream, 10, 7).unwrap();
        assert_eq!(short[8].class, 0);
        assert!(code_pick_rows(&[2u16, 3, 1], &codes, 8).is_err());
        assert!(code_pick_rows(&[0u16, 10, 1], &codes, 8).is_err());
    }

    /// claude-439: `row_permutation` drives `permute_code_rows`, which still
    /// equals the in-place shuffle of 0988dc0f0 (written out here) at every
    /// length and on the run's seed.
    #[test]
    fn row_permutation_drives_permute_code_rows() {
        for len in [0usize, 1, 2, 3, 4, 9, 64, 4096] {
            let codes: Vec<[u16; 2]> = (0..len).map(|i| [i as u16, (i * 7) as u16]).collect();
            for seed in [0u64, 1, 439] {
                let mut want = codes.clone();
                let mut state = seed;
                for i in (3..want.len()).rev() {
                    let r = splitmix64(&mut state);
                    let j = 2 + (r % (i as u64 - 1)) as usize;
                    want.swap(i, j);
                }
                assert_eq!(permute_code_rows(&codes, seed), want);
                let perm = row_permutation(len, seed);
                let gathered: Vec<[u16; 2]> = perm.iter().map(|&from| codes[from]).collect();
                assert_eq!(gathered, want);
            }
        }
    }

    #[test]
    fn permuted_rows_keep_the_multiset_and_the_frame_ids() {
        let codes: Vec<[u16; 2]> = (0..40u16).map(|i| [i * 3 + 1, i ^ 0x5A5A]).collect();
        let a = permute_code_rows(&codes, 7);
        let b = permute_code_rows(&codes, 7);
        let c = permute_code_rows(&codes, 8);
        assert_eq!(a, b);
        assert_ne!(a, codes);
        assert_ne!(a, c);
        assert_eq!(a[0], codes[0]);
        assert_eq!(a[1], codes[1]);
        let mut sorted_a = a.clone();
        let mut sorted_codes = codes.clone();
        sorted_a.sort();
        sorted_codes.sort();
        assert_eq!(sorted_a, sorted_codes);
        assert_eq!(permute_code_rows(&codes[..3], 7), codes[..3].to_vec());
    }

    #[test]
    fn repeat_chunks_shapes() {
        let stream = [0u16, 2, 3, 4, 5, 6, 1, 0, 7, 1];
        let (out, second) = repeat_chunks(&stream, 2).unwrap();
        assert_eq!(out, vec![0, 2, 3, 2, 3, 4, 5, 4, 5, 6, 6, 1, 0, 7, 7, 1]);
        let f = false;
        let t = true;
        assert_eq!(second, vec![f, f, t, t, f, f, t, t, f, t, f, f, t, f]);
        assert_eq!(second.len(), out.iter().filter(|&&x| x != BOS).count());
        let (empty, flags) = repeat_chunks(&[0u16, 1], 3).unwrap();
        assert_eq!(empty, vec![0, 1]);
        assert_eq!(flags, vec![false]);
        assert!(repeat_chunks(&[2u16, 3, 1], 2).is_err());
        assert!(repeat_chunks(&[0u16, 2, 0, 3, 1], 2).is_err());
        assert!(repeat_chunks(&[0u16, 2, 3], 2).is_err());
        assert!(repeat_chunks(&[0u16], 2).is_err());
        assert!(repeat_chunks(&stream, 0).is_err());
    }

    #[test]
    fn a_repeated_chunk_is_copied() {
        let codes = one_hot(16);
        let stream = [
            0u16, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 1, 0, 13, 14, 15, 2, 3, 1,
        ];
        let chunk = 4;
        let context = chunk + 2;
        let (out, second) = repeat_chunks(&stream, chunk).unwrap();
        let rows = code_pick_rows(&out, &codes, context).unwrap();
        assert_eq!(rows.len(), second.len());
        let mut checked = 0;
        let mut run = 0usize;
        for (k, row) in rows.iter().enumerate() {
            run = if second[k] { run + 1 } else { 0 };
            if run >= 2 {
                // The copy's second token is copied too, but its pair (end
                // of the first copy, start of the second) is new: class 2.
                let class = if run == 2 { 2 } else { 1 };
                assert_eq!((row.class, row.hit), (class, true), "row {k}");
                checked += 1;
            }
        }
        // Record one: chunks of 4, 4, 3 give 3 + 3 + 2; record two: 4, 1 give 3.
        assert_eq!(checked, 11);
    }
}
