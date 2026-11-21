//! Persisted Q32 weights for the S5 count, KN and Pick experts.

use super::{
    bucket_count, ceiling_pick_row_classes, expert_count, invalid, mix_sum, mixed_q48, present,
    PickClassRule, PickTarget, NGRAM_MAX_ORDER, WEIGHT_ONE,
};
use crate::Result;

const MAGIC_THREE: &str = "UORS5W01";
const MAGIC_FIVE: &str = "UORS5W02";
const HEADER: usize = 44;
const MAX_EXPERTS: usize = NGRAM_MAX_ORDER as usize + 4;

/// An S5 fit in bucket-major order. The binding identifies the
/// caller's sealed input and operator manifest. Lookup state is supplied by
/// the caller and has to match that manifest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrozenPickMixture {
    highest: u32,
    binding: [u8; 32],
    rule: PickClassRule,
    weights: Vec<u64>,
}

impl FrozenPickMixture {
    pub fn new(highest: u32, binding: [u8; 32], weights: Vec<u64>) -> Result<Self> {
        Self::with_rule(highest, binding, weights, PickClassRule::Three)
    }

    pub fn with_rule(
        highest: u32,
        binding: [u8; 32],
        weights: Vec<u64>,
        rule: PickClassRule,
    ) -> Result<Self> {
        let words = word_count(highest, rule)?;
        if weights.len() != words {
            return Err(invalid("unexpected S5 weight word count"));
        }
        let stride = bucket_count(highest);
        let base = expert_count(highest);
        for (bucket, row) in weights.chunks_exact(base + 1).enumerate() {
            if row.iter().any(|&weight| weight > WEIGHT_ONE)
                || row.iter().sum::<u64>() != WEIGHT_ONE
                || row[..base].iter().enumerate().any(|(expert, &weight)| {
                    weight != 0 && !present(highest, bucket % stride, expert)
                })
                || (bucket < stride && row[base] != 0)
            {
                return Err(invalid(format!("invalid S5 mixture bucket {bucket}")));
            }
        }
        Ok(Self {
            highest,
            binding,
            rule,
            weights,
        })
    }

    pub fn weights(&self) -> &[u64] {
        &self.weights
    }

    pub fn rule(&self) -> PickClassRule {
        self.rule
    }

    pub fn wire_tag(&self) -> &'static str {
        match self.rule {
            PickClassRule::Three => MAGIC_THREE,
            PickClassRule::Five => MAGIC_FIVE,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(HEADER + self.weights.len() * 8);
        bytes.extend_from_slice(self.wire_tag().as_bytes());
        bytes.extend_from_slice(&self.highest.to_le_bytes());
        bytes.extend_from_slice(&self.binding);
        for weight in &self.weights {
            bytes.extend_from_slice(&weight.to_le_bytes());
        }
        bytes
    }

    /// Requires the out-of-band manifest identity and rejects malformed or
    /// trailing words. The enclosing artifact digest pins the payload itself.
    pub fn from_bytes(bytes: &[u8], expected_binding: [u8; 32]) -> Result<Self> {
        if bytes.len() < HEADER {
            return Err(invalid("invalid S5 mixture header"));
        }
        let rule = match &bytes[..8] {
            magic if magic == MAGIC_THREE.as_bytes() => PickClassRule::Three,
            magic if magic == MAGIC_FIVE.as_bytes() => PickClassRule::Five,
            _ => return Err(invalid("invalid S5 mixture header")),
        };
        let highest = u32::from_le_bytes(
            bytes[8..12]
                .try_into()
                .map_err(|_| invalid("missing S5 order"))?,
        );
        if bytes[12..HEADER] != expected_binding {
            return Err(invalid("S5 mixture binding mismatch"));
        }
        if bytes.len() != HEADER + word_count(highest, rule)? * 8 {
            return Err(invalid("invalid S5 mixture byte length"));
        }
        let weights = bytes[HEADER..]
            .chunks_exact(8)
            .map(|word| word.try_into().map(u64::from_le_bytes))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|_| invalid("invalid S5 mixture weight word"))?;
        Self::with_rule(highest, expected_binding, weights, rule)
    }

    /// Evaluate one legal target from its original KN, raw count and causal
    /// Pick lookups. Returns the exact fit bucket and scored Q48 word.
    pub fn predict_mass(
        &self,
        kn_q48: u64,
        counts: &[u32],
        totals: &[u64],
        pick: PickTarget,
    ) -> Result<(u16, u64)> {
        if totals.len() != self.highest as usize + 1
            || counts.len() != totals.len()
            || !(1..=1 << 48).contains(&kn_q48)
            || counts
                .iter()
                .zip(totals)
                .any(|(&count, &total)| u64::from(count) > total)
        {
            return Err(invalid("invalid S5 prediction lookups"));
        }
        let experts = expert_count(self.highest) + 1;
        let mut row = [0; MAX_EXPERTS];
        let row = &mut row[..experts];
        let bucket =
            ceiling_pick_row_classes(self.highest, kn_q48, counts, totals, pick, self.rule.classes(), row)?;
        let start = usize::from(bucket) * experts;
        let weights = &self.weights[start..start + experts];
        Ok((bucket, mixed_q48(mix_sum(weights, row))))
    }
}

fn word_count(highest: u32, rule: PickClassRule) -> Result<usize> {
    if highest > NGRAM_MAX_ORDER {
        return Err(invalid("S5 mixture order exceeds the native key"));
    }
    Ok(bucket_count(highest) * usize::from(rule.classes()) * (expert_count(highest) + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_corrupt_or_misbound_weight_bytes_and_bad_inputs() -> Result<()> {
        let mut weights = vec![0; word_count(0, PickClassRule::Three)?];
        for row in weights.chunks_exact_mut(4) {
            row[0] = WEIGHT_ONE;
        }
        let frozen = FrozenPickMixture::new(0, [19; 32], weights)?;
        let wire = frozen.to_bytes();
        let legacy = [
            b"UORS5W01".as_slice(),
            &[0; 4],
            &[19; 32],
            &frozen
                .weights()
                .iter()
                .flat_map(|weight| weight.to_le_bytes())
                .collect::<Vec<_>>(),
        ]
        .concat();
        assert_eq!(wire, legacy);
        assert_eq!(FrozenPickMixture::from_bytes(&wire, [19; 32])?, frozen);
        assert!(FrozenPickMixture::from_bytes(&wire[..wire.len() - 1], [19; 32]).is_err());
        assert!(FrozenPickMixture::from_bytes(&wire, [18; 32]).is_err());
        let mut corrupt = wire;
        corrupt[HEADER] ^= 1;
        assert!(FrozenPickMixture::from_bytes(&corrupt, [19; 32]).is_err());
        let no_pick = PickTarget {
            class: 0,
            hit: false,
        };
        assert!(frozen.predict_mass(1, &[1], &[0], no_pick).is_err());
        assert_eq!(frozen.predict_mass(1, &[0], &[1], no_pick)?, (0, 1));
        let mut illegal = frozen.weights().to_vec();
        illegal[0] -= 1;
        illegal[3] = 1;
        assert!(FrozenPickMixture::new(0, [19; 32], illegal).is_err());
        Ok(())
    }

    #[test]
    fn five_class_carrier_reaches_bucket_879_and_rejects_other_classes() -> Result<()> {
        let highest = 10;
        let rule = PickClassRule::Five;
        let binding = [7; 32];
        let mut weights = vec![0; word_count(highest, rule)?];
        for row in weights.chunks_exact_mut(expert_count(highest) + 1) {
            row[0] = WEIGHT_ONE;
        }
        let last_row = weights.chunks_exact_mut(expert_count(highest) + 1).last().unwrap();
        last_row[0] = 0;
        *last_row.last_mut().unwrap() = WEIGHT_ONE;
        let fit = FrozenPickMixture::with_rule(highest, binding, weights, rule)?;
        let wire = fit.to_bytes();
        assert_eq!(&wire[..8], b"UORS5W02");
        let loaded = FrozenPickMixture::from_bytes(&wire, binding)?;
        assert_eq!(loaded, fit);
        assert_eq!(loaded.to_bytes(), wire);
        let mut retagged = wire;
        retagged[..8].copy_from_slice(b"UORS5W01");
        assert!(FrozenPickMixture::from_bytes(&retagged, binding).is_err());

        let counts = [0; 11];
        let totals = [1 << 15; 11];
        let predict = |class, hit| loaded.predict_mass(7, &counts, &totals, PickTarget { class, hit });
        assert_eq!(predict(rule.classify(Some(3)), true)?, (879, 1 << 48));
        assert_eq!(predict(rule.classify(Some(u32::MAX)), false)?, (879, 1));
        assert_eq!(predict(rule.classify(Some(2)), true)?, (703, 7));
        assert!(predict(5, true).is_err());
        assert!(predict(0, true).is_err());
        Ok(())
    }
}
