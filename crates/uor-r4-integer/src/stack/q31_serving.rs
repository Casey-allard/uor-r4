//! Native Q31 model state at the integer-serving boundary.
//!
//! The model state and Hamilton operator live in `uor-r4-native-vector`; this
//! adapter does not duplicate the arithmetic. It restores the exact 48-byte
//! state written by training and applies the same typed Q31 transition from
//! the serving side.

use uor_r4_native_vector::{q31_hamilton_rotate, Q31HamiltonModel};
pub use uor_r4_native_vector::{
    q31_quaternion_admitted, q31_saturate, Q31Quaternion, Q31Transport, Q31_MODEL_BYTES,
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
    use super::super::synthetic::synthetic_fbs_trunk;
    use super::super::{
        export_fbs_trunk_uorlut01, IntegerStackModel, NativeQ31Carry, SerializedStackLayerState,
    };
    use super::*;
    use uor_r4_native_vector::{q31_hamilton_rotate, Q31HamiltonModel};

    const TRAINED_STATE_HEX: &str = concat!(
        "8c60b17700000000961f0e1400000000",
        "00000000000000000000000000000000",
        "40000000000000000800000001000000",
    );
    const TRAINED_GATE: Q31Quaternion = [2_008_113_292, 336_469_910, 0, 0];

    fn trained_state() -> [u8; Q31_MODEL_BYTES] {
        hex::decode(TRAINED_STATE_HEX)
            .expect("fixture hex")
            .try_into()
            .expect("48-byte trained state")
    }

    fn fbs_model(seed: u64) -> IntegerStackModel {
        let artifact = export_fbs_trunk_uorlut01(&synthetic_fbs_trunk(seed)).expect("export");
        IntegerStackModel::parse(&artifact).expect("parse")
    }

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

    #[test]
    fn trained_state_drives_real_recurrence_and_preserves_every_raw_carry() {
        let state = trained_state();
        let artifact = export_fbs_trunk_uorlut01(&synthetic_fbs_trunk(20_261_115)).expect("export");
        let bare = IntegerStackModel::parse(&artifact).expect("bare model");
        let mut native = IntegerStackModel::parse(&artifact).expect("native model");
        native
            .attach_native_q31_state(&state)
            .expect("attach trained state");
        assert_eq!(native.native_q31_state_bytes(), Some(state));
        assert!(
            native.fbs_trunk_image().is_none(),
            "no parallel image may omit the state"
        );

        let mut bare_session = bare.session();
        let mut native_session = native.session();
        native_session
            .set_native_q31_trace(true)
            .expect("trace the attached state");
        let mut previous: Option<Vec<NativeQ31Carry>> = None;
        let mut bare_tokens = Vec::new();
        let mut native_tokens = Vec::new();
        for token in [3, 5, 8, 13] {
            let bare_step = bare_session.step_native(token).expect("bare step");
            let native_step = native_session.step_native(token).expect("native step");
            bare_tokens.push(bare_step.token);
            native_tokens.push(native_step.token);
            let carries = native_session
                .take_native_q31_trace()
                .expect("trace enabled");
            assert_eq!(carries.len(), 16, "two recurrence layers, eight lanes each");
            for carry in &carries {
                assert_eq!(
                    carry.transport,
                    q31_hamilton_rotate(TRAINED_GATE, carry.before).expect("Q31 oracle"),
                    "layer {} lane {}",
                    carry.layer,
                    carry.lane
                );
            }
            if let Some(previous) = previous.as_ref() {
                for (before, after) in carries.iter().zip(previous) {
                    assert_eq!(before.layer, after.layer);
                    assert_eq!(before.lane, after.lane);
                    assert_eq!(
                        before.before, after.after,
                        "layer {} lane {} did not carry its served words",
                        before.layer, before.lane
                    );
                }
            }
            previous = Some(carries);
        }
        assert_ne!(
            bare_tokens, native_tokens,
            "the attached state never changed an emitted token"
        );
    }

    #[test]
    fn native_q31_session_restore_continues_with_identical_words_and_tokens() {
        let mut model = fbs_model(20_261_116);
        model
            .attach_native_q31_state(&trained_state())
            .expect("attach trained state");

        let mut continued = model.session();
        for token in [3, 5] {
            continued.step_native(token).expect("prefix step");
        }
        let saved = continued.save_state();

        let bare = fbs_model(20_261_116);
        let mut unbound = bare.session();
        assert!(
            unbound.restore_state(&saved).is_err(),
            "a bare model must refuse a Q31-bound snapshot"
        );

        let mut other_model = fbs_model(20_261_116);
        let other_state = Q31HamiltonModel::new([1 << 31, 0, 0, 0], 8)
            .expect("other model")
            .to_bytes();
        other_model
            .attach_native_q31_state(&other_state)
            .expect("attach other state");
        let mut other = other_model.session();
        assert!(
            other.restore_state(&saved).is_err(),
            "a different Q31 state must refuse the snapshot"
        );

        let mut restored = model.session();
        restored
            .restore_state(&saved)
            .expect("restore exact Q31 session");
        continued
            .set_native_q31_trace(true)
            .expect("trace continued");
        restored.set_native_q31_trace(true).expect("trace restored");

        let mut continued_tokens = Vec::new();
        let mut restored_tokens = Vec::new();
        for token in [8, 13] {
            continued_tokens.push(continued.step_native(token).expect("continued step"));
            restored_tokens.push(restored.step_native(token).expect("restored step"));
            assert_eq!(
                continued.take_native_q31_trace(),
                restored.take_native_q31_trace()
            );
        }
        assert_eq!(continued_tokens, restored_tokens);
    }

    #[test]
    fn native_q31_trace_requires_an_attached_state() {
        let model = fbs_model(20_261_117);
        let mut session = model.session();
        assert!(session.set_native_q31_trace(true).is_err());
        assert!(session.take_native_q31_trace().is_none());
    }

    #[test]
    fn native_q31_refusals_do_not_mutate_the_session() {
        let mut model = fbs_model(20_261_118);
        model
            .attach_native_q31_state(&trained_state())
            .expect("attach trained state");

        let mut conflicting = model.session();
        conflicting.enable_snap_trace();
        conflicting
            .set_native_q31_trace(true)
            .expect("trace attached state");
        let before = conflicting.save_state();
        assert!(
            conflicting.step_native(3).is_err(),
            "two incompatible traces must be refused"
        );
        assert_eq!(
            conflicting.save_state(),
            before,
            "the refused mixed-trace step changed session state"
        );

        let mut prefix = model.session();
        prefix.step_native(3).expect("prefix step");
        let mut invalid = prefix.save_state();
        let recurrence = invalid
            .layers
            .iter_mut()
            .find_map(|layer| match layer {
                SerializedStackLayerState::Recurrence { state, .. } => Some(state),
                SerializedStackLayerState::Read { .. } => None,
            })
            .expect("a recurrence layer");
        recurrence[0] = (1_i64 << 31) + 1;

        let mut out_of_domain = model.session();
        out_of_domain
            .restore_state(&invalid)
            .expect("the snapshot shape is valid");
        let before = out_of_domain.save_state();
        assert!(
            out_of_domain.step_native(5).is_err(),
            "an out-of-domain carry must be refused"
        );
        assert_eq!(
            out_of_domain.save_state(),
            before,
            "the refused domain step changed session state"
        );
    }
}
