use serde::Serialize;

pub const SEED: u64 = 40_631_118;
pub const REPLICATES: usize = 20_000;
pub const LOW_RANK: usize = 500;
pub const HIGH_RANK: usize = 19_500;

struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> usize {
        let threshold = n.wrapping_neg() % n;
        loop {
            let bits = self.next();
            if bits >= threshold {
                return (bits % n) as usize;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Interval {
    pub lower_pair_count: i32,
    pub upper_pair_count: i32,
}

pub fn confidence<const N: usize>(families: &[Vec<[i32; N]>]) -> [Interval; N] {
    let mut rng = SplitMix64(SEED);
    let mut col: [Vec<i32>; N] = std::array::from_fn(|_| Vec::with_capacity(REPLICATES));
    for _ in 0..REPLICATES {
        let mut sums = [0; N];
        for family in families {
            for _ in family {
                let sampled = family[rng.below(family.len() as u64)];
                for (sum, value) in sums.iter_mut().zip(sampled) {
                    *sum += value;
                }
            }
        }
        for (column, sum) in col.iter_mut().zip(sums) {
            column.push(sum);
        }
    }
    col.map(|mut column| {
        column.sort_unstable();
        Interval {
            lower_pair_count: column[LOW_RANK - 1],
            upper_pair_count: column[HIGH_RANK - 1],
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitter_reference() {
        let mut stream = SplitMix64(0);
        assert_eq!(stream.next(), 0xe220_a839_7b1d_cdaf);
        assert_eq!(stream.next(), 0x6e78_9e6a_a1b9_65f4);
    }

    #[test]
    fn constant_strata_and_shared_draws() {
        let input = vec![vec![[1, -1]; 2], vec![[0, 0]; 1]];
        assert_eq!(
            confidence(&input),
            [
                Interval {
                    lower_pair_count: 2,
                    upper_pair_count: 2
                },
                Interval {
                    lower_pair_count: -2,
                    upper_pair_count: -2
                }
            ]
        );
        let input = vec![vec![[1, 1], [-1, -1], [0, 0]], vec![[1, 1], [0, 0]]];
        let got = confidence(&input);
        assert_eq!(got[0], got[1]);
        assert_eq!(
            got[0],
            Interval {
                lower_pair_count: -2,
                upper_pair_count: 4
            }
        );
    }
}
