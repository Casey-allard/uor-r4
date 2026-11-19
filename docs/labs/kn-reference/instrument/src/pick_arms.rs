//! Host-side bands and segment tallies for the Pick arms. Integer only.
#![allow(dead_code)]

use crate::{fail, Failure, Tally, CLASS_NOVEL};
use uor_r4_discrete::ngram_count::NGRAM_BOS;

/// SplitMix64 step: one deterministic 64-bit draw from `state`.
pub fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

fn non_bos_count(query: &[u16]) -> usize {
    query.len() - query.iter().filter(|&&t| t == NGRAM_BOS).count()
}

/// Tally (all, novel) over non-BOS positions whose segment is in `keep`.
pub fn segment_tally(
    query: &[u16],
    segments: &[u8],
    classes: &[u8],
    id_bytes: &[u8],
    bits: &[u64],
    keep: &[u8],
) -> Result<(Tally, Tally), Failure> {
    if segments.len() != query.len() || classes.len() != query.len() {
        return Err(fail(
            "segment_tally: segments/classes length differs from query",
        ));
    }
    if bits.len() != non_bos_count(query) {
        return Err(fail(
            "segment_tally: bits length differs from non-BOS count",
        ));
    }
    let mut all = Tally::default();
    let mut novel = Tally::default();
    let mut next = 0usize;
    for (i, &token) in query.iter().enumerate() {
        if token == NGRAM_BOS {
            continue;
        }
        let b = bits[next];
        next += 1;
        if !keep.contains(&segments[i]) {
            continue;
        }
        let bytes = *id_bytes
            .get(token as usize)
            .ok_or_else(|| fail("segment_tally: token id outside id_bytes"))?;
        all.add(b, bytes);
        if classes[i] == CLASS_NOVEL {
            novel.add(b, bytes);
        }
    }
    Ok((all, novel))
}

/// One bit sum per record (record starts at a BOS) holding a position of `segment`.
pub fn record_bits(
    query: &[u16],
    segments: &[u8],
    classes: &[u8],
    bits: &[u64],
    segment: u8,
    novel_only: bool,
) -> Result<Vec<u128>, Failure> {
    if segments.len() != query.len() || classes.len() != query.len() {
        return Err(fail(
            "record_bits: segments/classes length differs from query",
        ));
    }
    if bits.len() != non_bos_count(query) {
        return Err(fail("record_bits: bits length differs from non-BOS count"));
    }
    let mut out = Vec::new();
    let mut current: Option<u128> = None;
    let mut next = 0usize;
    for (i, &token) in query.iter().enumerate() {
        if token == NGRAM_BOS {
            if let Some(sum) = current.take() {
                out.push(sum);
            }
            if segments[i] == segment {
                current = Some(0);
            }
            continue;
        }
        let b = bits[next];
        next += 1;
        if segments[i] == segment {
            let sum = current.get_or_insert(0);
            if !novel_only || classes[i] == CLASS_NOVEL {
                *sum += u128::from(b);
            }
        }
    }
    if let Some(sum) = current.take() {
        out.push(sum);
    }
    Ok(out)
}

/// Paired bootstrap band on the summed per-record difference x - y.
pub fn paired_band(
    x: &[u128],
    y: &[u128],
    resamples: u32,
    seed: u64,
) -> Result<(i128, i128), Failure> {
    let n = x.len();
    if n == 0 || x.len() != y.len() || resamples < 40 {
        return Err(fail(
            "paired_band: need equal non-empty inputs and at least 40 resamples",
        ));
    }
    let d: Vec<i128> = x
        .iter()
        .zip(y)
        .map(|(&a, &b)| a as i128 - b as i128)
        .collect();
    let mut state = seed;
    let mut sums: Vec<i128> = Vec::with_capacity(resamples as usize);
    for _ in 0..resamples {
        let mut sum = 0i128;
        for _ in 0..n {
            sum += d[(splitmix64(&mut state) % n as u64) as usize];
        }
        sums.push(sum);
    }
    sums.sort_unstable();
    let k = (resamples / 40) as usize;
    Ok((sums[k - 1], sums[resamples as usize - k - 1]))
}

/// B2: the first arm's second-copy bits at most half the baseline's. A baseline of 0
/// bits means nothing was compared, and never passes.
pub fn b2_pass(a1_second_copy_bits: u128, a0_second_copy_bits: u128) -> bool {
    a0_second_copy_bits > 0
        && a1_second_copy_bits
            .checked_mul(2)
            .is_some_and(|twice| twice <= a0_second_copy_bits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paired_band_brackets_a_known_shift() {
        let y: Vec<u128> = (0..10).map(|r| 100 + r as u128).collect();
        let x: Vec<u128> = y.iter().map(|v| v + 5).collect();
        assert_eq!(paired_band(&x, &y, 1000, 7).unwrap(), (50, 50));

        let d: [i128; 8] = [3, -3, 2, -2, 5, -5, 1, -1];
        let y: Vec<u128> = vec![100; 8];
        let x: Vec<u128> = d.iter().map(|v| (100 + v) as u128).collect();
        let (low, high) = paired_band(&x, &y, 1000, 11).unwrap();
        assert!(low < high && low <= 0 && 0 <= high, "{low} {high}");
        assert_eq!(paired_band(&x, &y, 1000, 11).unwrap(), (low, high));
        assert!(paired_band(&x, &y, 39, 11).is_err());
        assert!(paired_band(&[], &[], 1000, 11).is_err());
    }

    // Two records: [BOS 5 6 EOS] segments 0 0 1 1; [BOS 7 EOS] segments 2 3 4.
    fn fixture() -> (Vec<u16>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u64>) {
        (
            vec![0, 5, 6, 1, 0, 7, 1],
            vec![0, 0, 1, 1, 2, 3, 4],
            vec![3, 1, 0, 0, 3, 1, 0],
            vec![1, 1, 2, 3, 4, 5, 6, 7],
            vec![10, 20, 30, 40, 50],
        )
    }

    #[test]
    fn segment_tallies_split_by_segment() {
        let (q, s, c, ib, b) = fixture();
        let t = |keep: &[u8]| {
            let (a, n) = segment_tally(&q, &s, &c, &ib, &b, keep).unwrap();
            (
                (a.targets, a.bits_q12, a.bytes),
                (n.targets, n.bits_q12, n.bytes),
            )
        };
        assert_eq!(t(&[0]), ((1, 10, 5), (1, 10, 5)));
        assert_eq!(t(&[1]), ((2, 50, 7), (0, 0, 0)));
        assert_eq!(t(&[0, 1]), ((3, 60, 12), (1, 10, 5)));
        assert_eq!(t(&[3, 4]), ((2, 90, 8), (1, 40, 7)));
        assert_eq!(t(&[2]), ((0, 0, 0), (0, 0, 0)));
        assert!(segment_tally(&q, &s[1..], &c, &ib, &b, &[0]).is_err());
        assert!(segment_tally(&q, &s, &c, &ib, &b[1..], &[0]).is_err());
    }

    #[test]
    fn record_bits_sums_per_record() {
        let (q, s, c, _, b) = fixture();
        assert_eq!(record_bits(&q, &s, &c, &b, 1, false).unwrap(), vec![50]);
        assert_eq!(record_bits(&q, &s, &c, &b, 3, false).unwrap(), vec![40]);
        assert_eq!(record_bits(&q, &s, &c, &b, 4, false).unwrap(), vec![50]);
        assert_eq!(record_bits(&q, &s, &c, &b, 0, true).unwrap(), vec![10]);
        assert_eq!(record_bits(&q, &s, &c, &b, 1, true).unwrap(), vec![0]);
        assert_eq!(record_bits(&q, &s, &c, &b, 2, false).unwrap(), vec![0]);
    }

    #[test]
    fn b2_reads_second_copies_only() {
        assert!(b2_pass(5, 10));
        assert!(!b2_pass(6, 10));
        assert!(!b2_pass(0, 0));
        assert!(b2_pass(0, 1));
        assert!(!b2_pass(u128::MAX, u128::MAX));
    }
}
