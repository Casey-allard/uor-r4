//! The common result of a read (ADR-0007's `Read` stage).
//!
//! The reads in this crate take different inputs — the native Q31 first-pick read steps a history
//! and a row, the Hamming copy heads select over earlier tokens — but a stage that consumes a read
//! only needs its **result**. This type is that result, and the two reads implement it without
//! changing their arithmetic:
//!
//! - the Q31 first-pick read returns one position and the row it writes (`payload`);
//! - a Hamming copy head returns its chosen positions and their Q31 margin weights.
//!
//! A result is either a selection (support + weights) or a row-writing read (support + payload);
//! never both. Support positions are `u16`, which holds every position a stack's context admits.

/// What one read produced at the current row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadOut {
    /// The positions the read chose, in the read's order. Empty when it chose none.
    pub support: Vec<u16>,
    /// The Q31 weights parallel to `support`; empty for a row-writing read.
    pub weights_q31: Vec<i128>,
    /// The row a row-writing read contributes; empty weights accompany it.
    pub payload: Option<Vec<i128>>,
}

impl ReadOut {
    /// One position and the row the read writes there (the native Q31 first-pick read).
    pub fn first_pick(pick: u32, payload: Vec<i128>) -> Self {
        let support = if pick == 0 {
            Vec::new()
        } else {
            vec![u16::try_from(pick - 1).expect("a pick fits the context's u16")]
        };
        ReadOut {
            support,
            weights_q31: Vec::new(),
            payload: Some(payload),
        }
    }

    /// Chosen positions with their Q31 weights, in the read's order (a selection read).
    pub fn selection(chosen: &[(usize, u32)], weights_q31: Vec<i128>) -> Self {
        assert_eq!(
            chosen.len(),
            weights_q31.len(),
            "a selection has one weight a position"
        );
        ReadOut {
            support: chosen
                .iter()
                .map(|&(position, _)| u16::try_from(position).expect("a position fits u16"))
                .collect(),
            weights_q31,
            payload: None,
        }
    }

    /// The 1-based pick of a row-writing read, or 0 when it chose none.
    pub fn pick(&self) -> u32 {
        self.support
            .first()
            .map(|position| u32::from(*position) + 1)
            .unwrap_or(0)
    }

    /// The invariant a consumer may rely on. `Ok` for every value this module builds; a hand-built
    /// value with mismatched lengths or both a payload and weights is refused.
    pub fn check(&self) -> Result<(), &'static str> {
        if !self.weights_q31.is_empty() && self.weights_q31.len() != self.support.len() {
            return Err("a read's weights are parallel to its support");
        }
        if self.payload.is_some() && !self.weights_q31.is_empty() {
            return Err("a read returns weights or a payload, not both");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::ReadOut;

    #[test]
    fn a_first_pick_is_one_position_and_a_row() {
        let out = ReadOut::first_pick(3, vec![7, 8]);
        assert_eq!(out.support, vec![2]);
        assert_eq!(out.pick(), 3);
        assert!(out.weights_q31.is_empty());
        assert_eq!(out.payload.as_deref(), Some(&[7, 8][..]));
        assert_eq!(out.check(), Ok(()));

        let none = ReadOut::first_pick(0, vec![1]);
        assert!(none.support.is_empty());
        assert_eq!(none.pick(), 0);
        assert_eq!(none.check(), Ok(()));
    }

    #[test]
    fn a_selection_has_one_weight_a_position() {
        let out = ReadOut::selection(&[(5, 2), (1, 0)], vec![1 << 30, 1 << 31]);
        assert_eq!(out.support, vec![5, 1]);
        assert_eq!(out.weights_q31, vec![1 << 30, 1 << 31]);
        assert!(out.payload.is_none());
        assert_eq!(out.check(), Ok(()));
    }

    #[test]
    fn a_malformed_result_is_refused() {
        let mut out = ReadOut::selection(&[(1, 0)], vec![1]);
        out.weights_q31.push(2);
        assert!(out.check().is_err());

        let mut both = ReadOut::selection(&[(1, 0)], vec![1]);
        both.payload = Some(vec![0]);
        assert!(both.check().is_err());
    }

    #[test]
    fn a_position_beyond_u16_is_refused_at_construction() {
        let position = usize::from(u16::MAX) + 1;
        let refused = std::panic::catch_unwind(|| ReadOut::selection(&[(position, 0)], vec![1]));
        assert!(refused.is_err());
    }
}
