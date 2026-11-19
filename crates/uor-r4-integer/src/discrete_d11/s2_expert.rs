//! Exact successor mass from the retained S2 count rows.

use super::{invalid, FrozenS2Table, Result, S2_HEADER_BYTES, S2_ROW_BYTES};

const BOS: u32 = 0;
const EOS: u32 = 1;
const LEGAL_IDS: u32 = 4095;
const MAX_ORDER: usize = 10;
const MAX_READS: usize = 4;
const TOP_SIZE: usize = 8;

/// Every distribution sums to one in Q32.
pub const S2_MASS_TOTAL: u64 = 1 << 32;
const FLOOR_BUDGET: u64 = (S2_MASS_TOTAL >> 8) * 8;

/// Pack original 12-bit IDs in PrimeRouteKey wire order.
/// The context is chronological and the most recent token occupies slot one.
pub fn s2_route_key(context: &[u16]) -> Result<u128> {
    if context.len() > MAX_ORDER {
        return Err(invalid("S2 expert key exceeds order ten"));
    }
    let mut key = (context.len() as u128) << 120;
    for (index, &token) in context.iter().rev().enumerate() {
        if u32::from(token) > LEGAL_IDS {
            return Err(invalid("S2 expert key token outside the 12-bit vocabulary"));
        }
        key |= u128::from(token) << (108 - 12 * index);
    }
    Ok(key)
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct S2ExpertRead {
    pub token: u16,
    pub count: u32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct S2RankedToken {
    pub token: u16,
    pub mass: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct S2ExpertPrediction {
    pub order: u8,
    /// Some(0) is a retained order-zero hit. None is a table miss.
    pub matched_key: Option<u128>,
    pub top8: [S2RankedToken; TOP_SIZE],
    reads: [S2ExpertRead; MAX_READS],
    read_len: usize,
    extra_mass: [u64; MAX_READS],
    uniform_budget: u64,
}

impl S2ExpertPrediction {
    /// The original retained token IDs and decoded count ranks, in table order.
    pub fn reads(&self) -> &[S2ExpertRead] {
        &self.reads[..self.read_len]
    }

    /// Exact Q32 mass of an original token ID. IDs outside 1..=4095 have zero mass.
    pub fn mass(&self, token: u32) -> u64 {
        if !(1..=LEGAL_IDS).contains(&token) {
            return 0;
        }

        let vocabulary = u64::from(LEGAL_IDS);
        let ordinal = u64::from(token);
        // The first remainder IDs receive one extra unit, in ascending ID order.
        let uniform = self.uniform_budget / vocabulary
            + u64::from(ordinal <= self.uniform_budget % vocabulary);
        let extra = self
            .reads()
            .iter()
            .position(|read| u32::from(read.token) == token)
            .map_or(0, |index| self.extra_mass[index]);
        uniform + extra
    }
}

impl FrozenS2Table {
    fn expert_reads(&self, key: u128) -> ([S2ExpertRead; MAX_READS], usize) {
        let rows = &self.bytes[S2_HEADER_BYTES..];
        let row_count = rows.len() / S2_ROW_BYTES;
        let row = |index| Self::fields(&rows[index * S2_ROW_BYTES..]);
        let mut low = 0;
        let mut high = row_count;
        while low < high {
            let mid = low + (high - low) / 2;
            if row(mid).0 < key {
                low = mid + 1;
            } else {
                high = mid;
            }
        }

        // Loading checked that at most four rows share a key.
        let mut reads = [S2ExpertRead::default(); MAX_READS];
        let mut len = 0;
        for index in low..row_count {
            let (found_key, token, rank) = row(index);
            if found_key != key {
                break;
            }
            reads[len] = S2ExpertRead {
                token,
                count: 1u32 << rank,
            };
            len += 1;
        }
        (reads, len)
    }

    /// Read the longest retained causal suffix within the current document.
    pub fn predict_expert(&self, prefix: &[u32]) -> Result<S2ExpertPrediction> {
        if prefix.iter().any(|&token| token > LEGAL_IDS) {
            return Err(invalid(
                "S2 expert prefix token outside the 12-bit vocabulary",
            ));
        }

        let mut context = [0; MAX_ORDER];
        let mut max_order = 0;
        for (index, &token) in prefix.iter().rev().take(MAX_ORDER).enumerate() {
            if token == EOS {
                break;
            }
            max_order = index + 1;
            context[MAX_ORDER - max_order] = token as u16;
            if token == BOS {
                break;
            }
        }

        let mut prediction = S2ExpertPrediction {
            order: 0,
            matched_key: None,
            top8: [S2RankedToken::default(); TOP_SIZE],
            reads: [S2ExpertRead::default(); MAX_READS],
            read_len: 0,
            extra_mass: [0; MAX_READS],
            uniform_budget: S2_MASS_TOTAL,
        };

        let largest_key = s2_route_key(&context[MAX_ORDER - max_order..])?;
        let slot_mask = (1u128 << 120) - 1;
        for order in (0..=max_order).rev() {
            // Keep the most recent slots and set the new order.
            let kept = slot_mask & !(slot_mask >> (12 * order));
            let key = ((order as u128) << 120) | (largest_key & kept);
            let (reads, len) = self.expert_reads(key);
            if len == 0 {
                continue;
            }
            if reads[..len]
                .iter()
                .any(|read| u32::from(read.token) > LEGAL_IDS)
            {
                return Err(invalid("S2 expert row token outside the 12-bit vocabulary"));
            }

            prediction.order = order as u8;
            prediction.matched_key = Some(key);
            prediction.reads = reads;
            prediction.read_len = len;
            prediction.uniform_budget = FLOOR_BUDGET;

            let count_total: u128 = prediction
                .reads()
                .iter()
                .map(|read| u128::from(read.count))
                .sum();
            let budget = u128::from(S2_MASS_TOTAL - FLOOR_BUDGET);
            let mut cumulative_count = 0u128;
            let mut previous_boundary = 0;
            for (read, extra) in reads[..len].iter().zip(&mut prediction.extra_mass) {
                cumulative_count += u128::from(read.count);
                let boundary = (budget * cumulative_count / count_total) as u64;
                // Boundaries telescope from zero to the exact count budget.
                *extra = boundary - previous_boundary;
                previous_boundary = boundary;
            }
            break;
        }

        // Uniform mass is nonincreasing in token ID, so the first eight
        // nonretained IDs suffice together with all of the retained IDs.
        let mut candidates = [S2RankedToken::default(); TOP_SIZE + MAX_READS];
        let uniform_tokens = (1..=LEGAL_IDS as u16)
            .filter(|token| !prediction.reads().iter().any(|read| read.token == *token))
            .take(TOP_SIZE);
        let retained_tokens = prediction.reads().iter().map(|read| read.token);
        for (candidate, token) in candidates
            .iter_mut()
            .zip(uniform_tokens.chain(retained_tokens))
        {
            *candidate = S2RankedToken {
                token,
                mass: prediction.mass(u32::from(token)),
            };
        }
        candidates.sort_unstable_by(|a, b| b.mass.cmp(&a.mass).then(a.token.cmp(&b.token)));
        prediction.top8.copy_from_slice(&candidates[..TOP_SIZE]);
        Ok(prediction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(records: &[(u128, u16, u8)]) -> FrozenS2Table {
        table_with_vocabulary(records, (LEGAL_IDS + 1) as usize)
    }

    fn table_with_vocabulary(records: &[(u128, u16, u8)], vocabulary: usize) -> FrozenS2Table {
        let mut records = records.to_vec();
        records.sort_unstable();
        let mut bytes = vec![0; S2_HEADER_BYTES];
        for (key, token, rank) in records {
            bytes.extend_from_slice(&key.to_le_bytes());
            bytes.extend_from_slice(&token.to_le_bytes());
            bytes.extend_from_slice(&[rank, 0]);
        }
        FrozenS2Table::load(&bytes, vocabulary).unwrap()
    }

    fn assert_exact_distribution(prediction: &S2ExpertPrediction) {
        let mut exhaustive: Vec<_> = (1..=LEGAL_IDS)
            .map(|token| S2RankedToken {
                token: token as u16,
                mass: prediction.mass(token),
            })
            .collect();
        assert!(exhaustive.iter().all(|candidate| candidate.mass > 0));
        assert_eq!(
            exhaustive
                .iter()
                .map(|candidate| candidate.mass)
                .sum::<u64>(),
            S2_MASS_TOTAL
        );
        assert_eq!(prediction.mass(BOS), 0);
        assert_eq!(prediction.mass(LEGAL_IDS + 1), 0);
        exhaustive
            .sort_unstable_by_key(|candidate| (std::cmp::Reverse(candidate.mass), candidate.token));
        assert_eq!(prediction.top8.as_slice(), &exhaustive[..TOP_SIZE]);
    }

    #[test]
    fn retains_largest_counts_and_allocates_every_q32_unit() {
        let table = table(&[(0, 2, 31), (0, 3000, 31), (0, 4000, 31), (0, 4095, 31)]);
        let prediction = table.predict_expert(&[]).unwrap();
        assert_eq!(prediction.order, 0);
        assert_eq!(prediction.matched_key, Some(0));
        assert_eq!(
            prediction.reads(),
            &[2, 3000, 4000, 4095].map(|token| S2ExpertRead {
                token,
                count: 1 << 31,
            })
        );
        assert_eq!(prediction.mass(1), 32777);
        assert_eq!(prediction.mass(9), 32776);
        assert_eq!(
            prediction.mass(4095),
            32776 + (S2_MASS_TOTAL - FLOOR_BUDGET) / 4
        );
        assert_eq!(
            prediction.top8.map(|entry| entry.token),
            [2, 3000, 4000, 4095, 1, 3, 4, 5]
        );
        assert_exact_distribution(&prediction);
    }

    #[test]
    fn sparse_count_remainders_are_cumulative_and_ranked_on_final_mass() {
        let prediction = table(&[(0, 1000, 1), (0, 1001, 1), (0, 1002, 1)])
            .predict_expert(&[700])
            .unwrap();
        let budget = S2_MASS_TOTAL - FLOOR_BUDGET;
        let uniform = FLOOR_BUDGET / u64::from(LEGAL_IDS);
        assert_eq!(prediction.mass(1000), uniform + budget / 3);
        assert_eq!(
            prediction.mass(1001),
            uniform + (budget * 2 / 3) - budget / 3
        );
        assert_eq!(prediction.mass(1002), uniform + budget - (budget * 2 / 3));
        assert_exact_distribution(&prediction);

        // Exercise a highly imbalanced pair with a single-digit sparse share.
        let prediction = table(&[(0, 1000, 1), (0, 4095, 31)])
            .predict_expert(&[700])
            .unwrap();
        assert_eq!(prediction.mass(1000), uniform + 3);
        assert_exact_distribution(&prediction);
    }

    #[test]
    fn exact_order_ten_then_backoff_and_reversal() {
        // Independent wire words with ten slots from most recent to oldest.
        let forward_key = 0x0a00_b00a_0090_0800_7006_0050_0400_3002u128;
        let reverse_key = 0x0a00_2003_0040_0500_6007_0080_0900_a00bu128;
        let suffix_key = 0x0100_b000_0000_0000_0000_0000_0000_0000u128;
        let table = table(&[
            (0, 70, 1),
            (suffix_key, 90, 1),
            (forward_key, 40, 1),
            (reverse_key, 50, 1),
        ]);
        let forward = table
            .predict_expert(&[800, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11])
            .unwrap();
        assert_eq!(
            (forward.order, forward.matched_key, forward.top8[0].token),
            (10, Some(forward_key), 40)
        );
        let reversed = table
            .predict_expert(&[11, 10, 9, 8, 7, 6, 5, 4, 3, 2])
            .unwrap();
        assert_eq!(
            (reversed.order, reversed.matched_key, reversed.top8[0].token),
            (10, Some(reverse_key), 50)
        );
        let backed_off = table.predict_expert(&[2, 4, 11]).unwrap();
        assert_eq!(
            (backed_off.order, backed_off.matched_key),
            (1, Some(suffix_key))
        );
        assert_eq!(table.predict_expert(&[4, 2]).unwrap().matched_key, Some(0));
    }

    #[test]
    fn documents_reset_at_eos_and_bos_can_be_the_oldest_slot() {
        let eos_key = 0x0100_1000_0000_0000_0000_0000_0000_0000u128;
        let across_eos = 0x0200_9001_0000_0000_0000_0000_0000_0000u128;
        let across_bos = 0x0300_9000_0080_0000_0000_0000_0000_0000u128;
        let bos_key = 0x0200_9000_0000_0000_0000_0000_0000_0000u128;
        let suffix_key = 0x0100_9000_0000_0000_0000_0000_0000_0000u128;
        let table = table(&[
            (eos_key, 21, 1),
            (across_eos, 22, 1),
            (across_bos, 23, 1),
            (bos_key, 24, 1),
            (suffix_key, 25, 1),
        ]);
        assert_eq!(table.predict_expert(&[8, EOS]).unwrap().matched_key, None);
        assert_eq!(
            table.predict_expert(&[8, EOS, 9]).unwrap().matched_key,
            Some(suffix_key)
        );
        assert_eq!(
            table.predict_expert(&[8, BOS, 9]).unwrap().matched_key,
            Some(bos_key)
        );
    }

    #[test]
    fn uniform_miss_has_exact_mass_floor_and_id_order() {
        let table = table(&[(0x0100_b000_0000_0000_0000_0000_0000_0000, 11, 1)]);
        let prediction = table.predict_expert(&[4]).unwrap();
        assert_eq!(prediction.order, 0);
        assert_eq!(prediction.matched_key, None);
        assert!(prediction.reads().is_empty());
        assert_eq!(
            prediction.top8.map(|entry| entry.token),
            [1, 2, 3, 4, 5, 6, 7, 8]
        );
        assert_eq!(prediction.mass(256), 1048833);
        assert_eq!(prediction.mass(257), 1048832);
        assert_exact_distribution(&prediction);
    }

    #[test]
    fn illegal_prefix_tokens_fail_even_behind_reset_or_outside_order_ten() {
        let table = table(&[]);
        for prefix in [[4096; 12], [u32::MAX; 12]] {
            assert!(table.predict_expert(&prefix).is_err());
        }
        assert!(table.predict_expert(&[4096, EOS, BOS, 2]).is_err());
        assert!(table
            .predict_expert(&[4096, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2])
            .is_err());
        assert!(table.predict_expert(&[4095]).is_ok());
        assert!(s2_route_key(&[4096]).is_err());
        assert!(s2_route_key(&[2; 11]).is_err());
    }

    #[test]
    fn rejects_matched_retained_ids_outside_expert_support() {
        let table = table_with_vocabulary(&[(0, 4096, 1)], 4097);
        assert!(table.predict_expert(&[]).is_err());
    }
}
