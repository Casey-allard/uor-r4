//! Bounded Q4 readout over explicit signed-H4 root feature slots.
//!
//! This head is a compiler component, not a language-model capability claim.
//! Feature extraction, labels, and offline learning remain caller responsibilities.
//! Packed coefficients are quarter units, with a bias followed by 120 bins for
//! each slot in each class. Admission partitions the tables; served scoring uses
//! checked root indexing and integer addition, without multiplying model values.
use crate::{geometric_potential_q4::unpack_coefficients, h4_tables::ROOT_COUNT};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TurnHeadError {
    InvalidConfig { classes: usize, slots: usize },
    InvalidPacked(String),
    FeatureCount { expected: usize, actual: usize },
    InvalidRoot { slot: usize, root: u8 },
    InvalidPartition,
    ScoreOverflow,
    NotBinary,
}
impl fmt::Display for TurnHeadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "geometric turn head: {self:?}")
    }
}
impl std::error::Error for TurnHeadError {}
pub type TurnHeadResult<T> = Result<T, TurnHeadError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TurnHeadConfig {
    pub classes: usize,
    pub slots: usize,
}
impl TurnHeadConfig {
    pub fn coefficient_count(self) -> TurnHeadResult<usize> {
        if !(2..=64).contains(&self.classes) || !(1..=32).contains(&self.slots) {
            return Err(TurnHeadError::InvalidConfig {
                classes: self.classes,
                slots: self.slots,
            });
        }
        // Dimension arithmetic is admission work, not model-value scoring.
        self.slots
            .checked_mul(ROOT_COUNT)
            .and_then(|n| n.checked_add(1))
            .and_then(|n| n.checked_mul(self.classes))
            .ok_or(TurnHeadError::InvalidPartition)
    }
}
struct ClassTable {
    bias: i8,
    slots: Vec<Box<[i8; ROOT_COUNT]>>,
}
pub struct NativeTurnHead {
    config: TurnHeadConfig,
    packed: Box<[u8]>,
    classes: Vec<ClassTable>,
}
impl NativeTurnHead {
    pub fn new(config: TurnHeadConfig, packed: &[u8]) -> TurnHeadResult<Self> {
        let count = config.coefficient_count()?;
        let q = unpack_coefficients(count, packed)
            .map_err(|e| TurnHeadError::InvalidPacked(e.to_string()))?;
        let mut values = q.iter();
        let mut classes = Vec::with_capacity(config.classes);
        for _ in 0..config.classes {
            let bias = *values.next().ok_or(TurnHeadError::InvalidPartition)?;
            let mut slots = Vec::with_capacity(config.slots);
            for _ in 0..config.slots {
                let mut row = [0i8; ROOT_COUNT];
                for value in &mut row {
                    *value = *values.next().ok_or(TurnHeadError::InvalidPartition)?;
                }
                slots.push(Box::new(row));
            }
            classes.push(ClassTable { bias, slots });
        }
        if values.next().is_some() {
            return Err(TurnHeadError::InvalidPartition);
        }
        Ok(Self {
            config,
            packed: packed.into(),
            classes,
        })
    }
    pub fn config(&self) -> TurnHeadConfig {
        self.config
    }
    pub fn packed_coefficients(&self) -> &[u8] {
        &self.packed
    }
    /// Exact quarter-integer logits. Feature slots retain all 120 signed roots.
    pub fn scores(&self, features: &[u8]) -> TurnHeadResult<Vec<i64>> {
        if features.len() != self.config.slots {
            return Err(TurnHeadError::FeatureCount {
                expected: self.config.slots,
                actual: features.len(),
            });
        }
        for (slot, &root) in features.iter().enumerate() {
            if usize::from(root) >= ROOT_COUNT {
                return Err(TurnHeadError::InvalidRoot { slot, root });
            }
        }
        let mut scores = Vec::with_capacity(self.classes.len());
        for class in &self.classes {
            let mut score = i64::from(class.bias);
            for (table, &root) in class.slots.iter().zip(features) {
                score = score
                    .checked_add(i64::from(table[usize::from(root)]))
                    .ok_or(TurnHeadError::ScoreOverflow)?;
            }
            scores.push(score);
        }
        Ok(scores)
    }
    /// Lower class index wins exact ties, matching the declared label order.
    pub fn predict(&self, features: &[u8]) -> TurnHeadResult<usize> {
        let scores = self.scores(features)?;
        let mut winner = 0usize;
        for (index, &score) in scores.iter().enumerate().skip(1) {
            if score > scores[winner] {
                winner = index;
            }
        }
        Ok(winner)
    }
    /// Class-one minus class-zero margin for the existing contiguous-span decoder.
    pub fn binary_margin(&self, features: &[u8]) -> TurnHeadResult<i64> {
        if self.config.classes != 2 {
            return Err(TurnHeadError::NotBinary);
        }
        let scores = self.scores(features)?;
        scores[1]
            .checked_sub(scores[0])
            .ok_or(TurnHeadError::ScoreOverflow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometric_potential_q4::pack_coefficients;
    #[test]
    fn turn_head_signed_lookup_matches_declared_layout_and_tie_order(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cfg = TurnHeadConfig {
            classes: 2,
            slots: 2,
        };
        let mut q = vec![0i8; cfg.coefficient_count()?];
        q[0] = 2;
        q[1 + 119] = -7;
        q[1 + 120 + 1] = 4;
        let other = 1 + 2 * 120;
        q[other] = -2;
        q[other + 1 + 119] = 7;
        q[other + 1 + 120 + 1] = -4;
        let head = NativeTurnHead::new(cfg, &pack_coefficients(&q)?)?;
        assert_eq!(head.scores(&[119, 1])?, vec![-1, 1]);
        assert_eq!(head.binary_margin(&[119, 1])?, 2);
        assert_eq!(head.predict(&[119, 1])?, 1);
        let zero =
            NativeTurnHead::new(cfg, &pack_coefficients(&vec![0; cfg.coefficient_count()?])?)?;
        assert_eq!(zero.predict(&[0, 119])?, 0);
        assert_eq!(
            head.packed_coefficients(),
            pack_coefficients(&q)?.as_slice()
        );
        Ok(())
    }
    #[test]
    fn turn_head_rejects_malformed_payload_and_feature_boundaries(
    ) -> Result<(), Box<dyn std::error::Error>> {
        for cfg in [
            TurnHeadConfig {
                classes: 1,
                slots: 1,
            },
            TurnHeadConfig {
                classes: 65,
                slots: 1,
            },
            TurnHeadConfig {
                classes: 2,
                slots: 0,
            },
            TurnHeadConfig {
                classes: 2,
                slots: 33,
            },
        ] {
            assert!(cfg.coefficient_count().is_err());
        }
        let cfg = TurnHeadConfig {
            classes: 2,
            slots: 1,
        };
        let packed = pack_coefficients(&vec![0; cfg.coefficient_count()?])?;
        let head = NativeTurnHead::new(cfg, &packed)?;
        assert!(head.scores(&[]).is_err());
        assert!(head.scores(&[120]).is_err());
        assert!(NativeTurnHead::new(cfg, &packed[..packed.len() - 1]).is_err());
        let mut bad = packed;
        bad[0] = 8;
        assert!(NativeTurnHead::new(cfg, &bad).is_err());
        let nonbinary = TurnHeadConfig {
            classes: 3,
            slots: 1,
        };
        let mut padded = pack_coefficients(&vec![0; nonbinary.coefficient_count()?])?;
        let last = padded.len() - 1;
        padded[last] |= 0x10;
        assert!(NativeTurnHead::new(nonbinary, &padded).is_err());
        Ok(())
    }
}
