//! Native Q31 model state at the integer-serving boundary.
//!
//! The model state and Hamilton operator live in `uor-r4-native-vector`; this
//! adapter does not duplicate the arithmetic. It restores the exact 48-byte
//! state written by training and applies the same typed Q31 transition from
//! the serving side.

use uor_r4_native_vector::{
    q31_hamilton_rotate, Q31HamiltonModel, Q31Quaternion, Q31Transport, Q31_MODEL_BYTES,
};

use super::StackError;

/// A serving-side handle over one serialized native Q31 model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Q31ServingSession {
    model: Q31HamiltonModel,
}

impl Q31ServingSession {
    /// Restore the exact training state; no numeric conversion is performed.
    pub fn from_bytes(bytes: [u8; Q31_MODEL_BYTES]) -> Result<Self, StackError> {
        Self::from_slice(&bytes)
    }

    /// Restore a fixed-width state supplied as a slice.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, StackError> {
        let bytes: [u8; Q31_MODEL_BYTES] = bytes.try_into().map_err(|_| {
            StackError::Numerics(format!(
                "native Q31 model state must be exactly {Q31_MODEL_BYTES} bytes"
            ))
        })?;
        Q31HamiltonModel::from_bytes(bytes)
            .map(|model| Self { model })
            .map_err(map_q31_error)
    }

    /// Return the lossless serving state for persistence.
    pub fn to_bytes(self) -> [u8; Q31_MODEL_BYTES] {
        self.model.to_bytes()
    }

    pub fn gate(self) -> Q31Quaternion {
        self.model.gate()
    }

    pub fn epoch(self) -> u64 {
        self.model.epoch()
    }

    pub fn step_shift(self) -> u32 {
        self.model.step_shift()
    }

    /// Apply the shared Q31 Hamilton transition at the loaded gate.
    pub fn transition(&self, value: Q31Quaternion) -> Result<Q31Transport, StackError> {
        q31_hamilton_rotate(self.model.gate(), value).map_err(map_q31_error)
    }
}

fn map_q31_error(error: uor_r4_native_vector::Error) -> StackError {
    StackError::Numerics(format!("native Q31 serving: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_native_vector::{q31_hamilton_rotate, Q31HamiltonModel};

    #[test]
    fn a_serialized_trained_gate_serves_the_same_raw_words() {
        let one = 1_i64 << 31;
        let target = [1_518_500_249, 1_518_500_249, 0, 0];
        let values = vec![[one, 0, 0, 0]; 8];
        let targets = vec![target; 8];
        let mut model = Q31HamiltonModel::new([one, 0, 0, 0], 8).unwrap();
        for _ in 0..64 {
            model.step_reference(&values, &targets).unwrap();
        }

        let state = model.to_bytes();
        let serving = Q31ServingSession::from_bytes(state).unwrap();
        assert_eq!(serving.to_bytes(), state);
        assert_eq!(serving.gate(), model.gate());
        assert_eq!(serving.epoch(), model.epoch());
        assert_eq!(serving.step_shift(), model.step_shift());

        for value in [
            [one, 0, 0, 0],
            [0, one, 0, 0],
            [1 << 30, -(1 << 29), 7, -(1 << 28)],
        ] {
            assert_eq!(
                serving.transition(value).unwrap(),
                q31_hamilton_rotate(model.gate(), value).unwrap()
            );
        }
    }

    #[test]
    fn serving_state_refuses_wrong_width_version_trailing_shift_and_domain() {
        let model = Q31HamiltonModel::new([1 << 31, 0, 0, 0], 8).unwrap();
        assert!(Q31ServingSession::from_slice(&[0; 47]).is_err());

        let mut bad_version = model.to_bytes();
        bad_version[44] = 2;
        assert!(Q31ServingSession::from_bytes(bad_version).is_err());

        let mut trailing = model.to_bytes();
        trailing[47] = 1;
        assert!(Q31ServingSession::from_bytes(trailing).is_err());

        let mut shift = model.to_bytes();
        shift[40..44].copy_from_slice(&32_u32.to_le_bytes());
        assert!(Q31ServingSession::from_bytes(shift).is_err());

        let mut bad_gate = model.to_bytes();
        bad_gate[..8].copy_from_slice(&((1_i64 << 31) + 1).to_le_bytes());
        assert!(Q31ServingSession::from_bytes(bad_gate).is_err());
    }
}
