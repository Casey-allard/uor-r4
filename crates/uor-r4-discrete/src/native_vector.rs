//! Native Q31 vector transport shared by the first integer CUDA training kernel
//! and its Rust oracle.
//!
//! The state is four signed 64-bit quaternion words at Q31, with each admitted
//! coordinate in the signed 32-bit range including +2^31. Products are
//! sign-extended 64-bit integer multiplies of those admitted coordinates,
//! accumulated modulo 2^64, rounded by 2^30, then shifted right by 31 and
//! saturated to the same closed domain. This module defines the reference
//! words; the AOT CUDA module must return the same words.

use crate::{Error, Result};

/// One Q31 quaternion: `(w, x, y, z)`.
pub type Q31Quaternion = [i64; 4];

/// One Q31 quaternion after Hamilton transport.
pub type Q31Transport = [i64; 4];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Q31TrainingStep {
    pub prediction: Q31Transport,
    pub candidate_gate: Q31Quaternion,
    pub accepted_gate: Q31Quaternion,
    pub loss_before: u64,
    pub loss_after: u64,
    pub accepted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Q31ModelStep {
    pub gate_before: Q31Quaternion,
    pub gate_after: Q31Quaternion,
    pub loss_before: u64,
    pub loss_after: u64,
    pub epoch_before: u64,
    pub epoch_after: u64,
    pub accepted: bool,
    pub items: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Q31HamiltonModel {
    gate: Q31VectorState,
    step_shift: u32,
    epoch: u64,
}

const Q31_MIN: i64 = -(1 << 31);
const Q31_MAX: i64 = 1 << 31;
/// Largest admitted batch: the last `item * 4` index remains in the lower 32 bits.
pub const MAX_Q31_BATCH_ITEMS: usize = 1 << 30;
/// Largest admitted per-word learning-rate shift; `0` applies the full credit.
pub const MAX_Q31_STEP_SHIFT: u32 = 31;
const Q31_MODEL_VERSION: u8 = 1;
const Q31_MODEL_BYTES: usize = 48;

/// Whether one word is inside the closed native Q31 domain.
pub fn q31_coordinate_admitted(word: i64) -> bool {
    (Q31_MIN..=Q31_MAX).contains(&word)
}

/// Whether every word is inside the closed native Q31 domain.
pub fn q31_quaternion_admitted(value: Q31Quaternion) -> bool {
    value.into_iter().all(q31_coordinate_admitted)
}

/// Whether one batch fits the native launch-index contract.
pub fn q31_batch_len_admitted(items: usize) -> bool {
    items <= MAX_Q31_BATCH_ITEMS
}

/// Whether one word is in the closed Q31 domain after saturating an integer value.
pub fn q31_saturate(word: i64) -> i64 {
    word.clamp(Q31_MIN, Q31_MAX)
}

/// The serialized word layout of one native vector carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Q31VectorState {
    words: Q31Quaternion,
}

impl Q31VectorState {
    /// Construct one serializable state only when all four words are admitted.
    pub fn new(words: Q31Quaternion) -> Result<Self> {
        if !q31_quaternion_admitted(words) {
            return Err(Error::Invalid(
                "native Q31 state must be in -2^31..=2^31".into(),
            ));
        }
        Ok(Self { words })
    }

    /// Raw words in canonical `(w, x, y, z)` order.
    pub fn words(self) -> Q31Quaternion {
        self.words
    }

    /// Lossless little-endian carrier: four consecutive signed 64-bit words.
    pub fn to_bytes(self) -> [u8; 32] {
        let mut bytes = [0; 32];
        for (index, word) in self.words.iter().enumerate() {
            bytes[index * 8..(index + 1) * 8].copy_from_slice(&word.to_le_bytes());
        }
        bytes
    }

    /// Restore raw Q31 words without scale conversion, refusing out-of-domain state.
    pub fn from_bytes(bytes: [u8; 32]) -> Result<Self> {
        let mut words = [0; 4];
        for (index, word) in words.iter_mut().enumerate() {
            *word = i64::from_le_bytes(
                bytes[index * 8..(index + 1) * 8]
                    .try_into()
                    .expect("fixed Q31 word width"),
            );
        }
        Self::new(words)
    }
}

impl Q31HamiltonModel {
    pub fn new(gate: Q31Quaternion, step_shift: u32) -> Result<Self> {
        if step_shift > MAX_Q31_STEP_SHIFT {
            return Err(Error::Invalid(
                "native Q31 model step_shift must be in 0..=31".into(),
            ));
        }
        Ok(Self {
            gate: Q31VectorState::new(gate)?,
            step_shift,
            epoch: 0,
        })
    }

    pub fn gate(self) -> Q31Quaternion {
        self.gate.words()
    }

    pub fn step_shift(self) -> u32 {
        self.step_shift
    }

    pub fn epoch(self) -> u64 {
        self.epoch
    }

    /// Versioned little-endian model state: gate words, epoch, step shift and version.
    pub fn to_bytes(self) -> [u8; Q31_MODEL_BYTES] {
        let mut bytes = [0; Q31_MODEL_BYTES];
        bytes[..32].copy_from_slice(&self.gate.to_bytes());
        bytes[32..40].copy_from_slice(&self.epoch.to_le_bytes());
        bytes[40..44].copy_from_slice(&self.step_shift.to_le_bytes());
        bytes[44] = Q31_MODEL_VERSION;
        bytes
    }

    /// Restore the exact gate and optimizer cursor, refusing any unknown bytes.
    pub fn from_bytes(bytes: [u8; Q31_MODEL_BYTES]) -> Result<Self> {
        if bytes[44] != Q31_MODEL_VERSION || bytes[45..].iter().any(|byte| *byte != 0) {
            return Err(Error::Invalid(
                "native Q31 model state has an unknown version or nonzero trailing bytes".into(),
            ));
        }
        let gate = Q31VectorState::from_bytes(bytes[..32].try_into().expect("fixed gate width"))?;
        let epoch = u64::from_le_bytes(
            bytes[32..40]
                .try_into()
                .expect("fixed native model epoch width"),
        );
        let step_shift = u32::from_le_bytes(
            bytes[40..44]
                .try_into()
                .expect("fixed native model shift width"),
        );
        if step_shift > MAX_Q31_STEP_SHIFT {
            return Err(Error::Invalid(
                "native Q31 model step_shift must be in 0..=31".into(),
            ));
        }
        Ok(Self {
            gate,
            step_shift,
            epoch,
        })
    }

    /// Apply one shared model step from already-computed per-item training records.
    pub fn step_from_records(
        &mut self,
        records: &[Q31TrainingStep],
        values: &[Q31Quaternion],
        targets: &[Q31Quaternion],
    ) -> Result<Q31ModelStep> {
        if records.is_empty() || records.len() != values.len() || values.len() != targets.len() {
            return Err(Error::Invalid(
                "native Q31 model step needs nonempty equal-length records, values and targets"
                    .into(),
            ));
        }
        if !q31_batch_len_admitted(records.len()) {
            return Err(Error::Invalid(
                "native Q31 model batch exceeds the admitted 2^30-item bound".into(),
            ));
        }
        if values
            .iter()
            .chain(targets)
            .any(|quaternion| !q31_quaternion_admitted(*quaternion))
        {
            return Err(Error::Invalid(
                "native Q31 model values and targets must be in -2^31..=2^31".into(),
            ));
        }
        let gate_before = self.gate.words();
        let mut delta = [0_i128; 4];
        for record in records {
            if !q31_quaternion_admitted(record.candidate_gate) {
                return Err(Error::Invalid(
                    "native Q31 model candidate is outside the closed domain".into(),
                ));
            }
            for (index, sum) in delta.iter_mut().enumerate() {
                *sum += i128::from(record.candidate_gate[index] - gate_before[index]);
            }
        }
        let count = records.len() as i128;
        let candidate_gate = std::array::from_fn(|index| {
            q31_saturate((i128::from(gate_before[index]) + delta[index] / count) as i64)
        });
        let loss_before =
            checked_q31_loss_sum(values.iter().zip(targets).map(|(&value, &target)| {
                Ok(q31_l1_loss(
                    q31_hamilton_rotate(gate_before, value)?,
                    target,
                ))
            }))?;
        let loss_after =
            checked_q31_loss_sum(values.iter().zip(targets).map(|(&value, &target)| {
                Ok(q31_l1_loss(
                    q31_hamilton_rotate(candidate_gate, value)?,
                    target,
                ))
            }))?;
        let accepted = loss_after < loss_before;
        let epoch_before = self.epoch;
        let epoch_after = if accepted {
            self.epoch
                .checked_add(1)
                .ok_or_else(|| Error::Invalid("native Q31 model epoch overflow".into()))?
        } else {
            self.epoch
        };
        let gate_after = if accepted {
            Q31VectorState::new(candidate_gate)?
        } else {
            self.gate
        };
        if accepted {
            self.gate = gate_after;
            self.epoch = epoch_after;
        }
        Ok(Q31ModelStep {
            gate_before,
            gate_after: gate_after.words(),
            loss_before,
            loss_after,
            epoch_before,
            epoch_after,
            accepted,
            items: records.len(),
        })
    }

    /// Run one shared model step through the Rust operator oracle.
    pub fn step_reference(
        &mut self,
        values: &[Q31Quaternion],
        targets: &[Q31Quaternion],
    ) -> Result<Q31ModelStep> {
        if values.is_empty() || values.len() != targets.len() {
            return Err(Error::Invalid(
                "native Q31 model step needs nonempty equal-length values and targets".into(),
            ));
        }
        let gate = self.gate.words();
        let records = values
            .iter()
            .zip(targets)
            .map(|(&value, &target)| q31_hamilton_train_step(gate, value, target, self.step_shift))
            .collect::<Result<Vec<_>>>()?;
        self.step_from_records(&records, values, targets)
    }
}

const ROUND_Q30: u64 = 1 << 30;
const Q31_SHIFT: u32 = 31;

#[inline]
fn product(a: i64, b: i64) -> u64 {
    a.wrapping_mul(b) as u64
}

#[inline]
fn q31_word(word: u64) -> i64 {
    (((word.wrapping_add(ROUND_Q30)) as i64) >> Q31_SHIFT).clamp(Q31_MIN, Q31_MAX)
}

/// Hamilton rotation `gate (x) value` under the closed native Q31 ABI.
///
/// Invalid state words are refused consistently on CPU and GPU. Output words
/// are saturated to the same range, so each result is admissible as the next
/// native state. No float, transcendental or normalization fallback exists.
pub fn q31_hamilton_rotate(gate: Q31Quaternion, value: Q31Quaternion) -> Result<Q31Transport> {
    if !q31_quaternion_admitted(gate) || !q31_quaternion_admitted(value) {
        return Err(Error::Invalid(
            "native Q31 coordinates must be in -2^31..=2^31".into(),
        ));
    }
    let p00 = product(gate[0], value[0]);
    let p01 = product(gate[0], value[1]);
    let p02 = product(gate[0], value[2]);
    let p03 = product(gate[0], value[3]);
    let p10 = product(gate[1], value[0]);
    let p11 = product(gate[1], value[1]);
    let p12 = product(gate[1], value[2]);
    let p13 = product(gate[1], value[3]);
    let p20 = product(gate[2], value[0]);
    let p21 = product(gate[2], value[1]);
    let p22 = product(gate[2], value[2]);
    let p23 = product(gate[2], value[3]);
    let p30 = product(gate[3], value[0]);
    let p31 = product(gate[3], value[1]);
    let p32 = product(gate[3], value[2]);
    let p33 = product(gate[3], value[3]);

    Ok([
        q31_word(p00.wrapping_sub(p11).wrapping_sub(p22).wrapping_sub(p33)),
        q31_word(p01.wrapping_add(p10).wrapping_add(p23).wrapping_sub(p32)),
        q31_word(p02.wrapping_sub(p13).wrapping_add(p20).wrapping_add(p31)),
        q31_word(p03.wrapping_add(p12).wrapping_sub(p21).wrapping_add(p30)),
    ])
}

/// L1 distance in raw Q31 words.
pub fn q31_l1_loss(actual: Q31Transport, expected: Q31Transport) -> u64 {
    actual
        .into_iter()
        .zip(expected)
        .map(|(actual, expected)| actual.abs_diff(expected))
        .sum()
}

fn checked_q31_loss_sum<I>(losses: I) -> Result<u64>
where
    I: IntoIterator<Item = Result<u64>>,
{
    let total = losses.into_iter().try_fold(0_u128, |sum, loss| {
        sum.checked_add(u128::from(loss?))
            .ok_or_else(|| Error::Invalid("native Q31 aggregate loss overflow".into()))
    })?;
    u64::try_from(total).map_err(|_| Error::Invalid("native Q31 aggregate loss exceeds u64".into()))
}

/// One bounded geometric credit step against a target Q31 transport.
///
/// The forward is the same Hamilton operator the CUDA trainer uses. The
/// residual `prediction - target` is passed through that operator with the
/// conjugate value to obtain a quaternion-shaped credit. The candidate gate is
/// the old gate minus `credit >> step_shift`, saturated to the closed Q31
/// domain. A candidate is accepted only when its L1 loss is strictly smaller.
pub fn q31_hamilton_train_step(
    gate: Q31Quaternion,
    value: Q31Quaternion,
    target: Q31Quaternion,
    step_shift: u32,
) -> Result<Q31TrainingStep> {
    if !q31_quaternion_admitted(gate)
        || !q31_quaternion_admitted(value)
        || !q31_quaternion_admitted(target)
    {
        return Err(Error::Invalid(
            "native Q31 training words must be in -2^31..=2^31".into(),
        ));
    }
    if step_shift > MAX_Q31_STEP_SHIFT {
        return Err(Error::Invalid(
            "native Q31 step_shift must be in 0..=31".into(),
        ));
    }

    let prediction = q31_hamilton_rotate(gate, value)?;
    let loss_before = q31_l1_loss(prediction, target);
    let residual = [
        q31_saturate(prediction[0] - target[0]),
        q31_saturate(prediction[1] - target[1]),
        q31_saturate(prediction[2] - target[2]),
        q31_saturate(prediction[3] - target[3]),
    ];
    let credit = q31_hamilton_rotate(residual, [value[0], -value[1], -value[2], -value[3]])?;
    let candidate_gate =
        std::array::from_fn(|index| q31_saturate(gate[index] - (credit[index] >> step_shift)));
    let candidate_prediction = q31_hamilton_rotate(candidate_gate, value)?;
    let loss_after = q31_l1_loss(candidate_prediction, target);
    let accepted = loss_after < loss_before;
    Ok(Q31TrainingStep {
        prediction,
        candidate_gate,
        accepted_gate: if accepted { candidate_gate } else { gate },
        loss_before,
        loss_after,
        accepted,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE: i64 = 1 << 31;

    #[test]
    fn identity_preserves_each_axis() {
        let identity = [ONE, 0, 0, 0];
        for value in [
            [ONE, 0, 0, 0],
            [0, ONE, 0, 0],
            [0, 0, ONE, 0],
            [0, 0, 0, ONE],
        ] {
            assert_eq!(q31_hamilton_rotate(identity, value).unwrap(), value);
        }
    }

    #[test]
    fn transport_signs_follow_the_hamilton_product() {
        assert_eq!(
            q31_hamilton_rotate([0, ONE, 0, 0], [0, 0, ONE, 0]).unwrap(),
            [0, 0, 0, ONE]
        );
        assert_eq!(
            q31_hamilton_rotate([0, 0, ONE, 0], [0, ONE, 0, 0]).unwrap(),
            [0, 0, 0, -ONE]
        );
    }

    #[test]
    fn serialization_preserves_every_raw_word() {
        let state = Q31VectorState::new([1 << 31, -(1 << 31), -1, 0x0123_4567]).unwrap();
        assert_eq!(Q31VectorState::from_bytes(state.to_bytes()).unwrap(), state);
    }

    #[test]
    fn out_of_domain_state_is_refused_consistently() {
        let invalid = [Q31_MAX + 1, 0, 0, 0];
        assert!(q31_hamilton_rotate(invalid, [ONE, 0, 0, 0]).is_err());
        assert!(q31_hamilton_rotate([ONE, 0, 0, 0], invalid).is_err());
        for words in [[Q31_MAX + 1, 0, 0, 0], [Q31_MIN - 1, 0, 0, 0]] {
            assert!(Q31VectorState::new(words).is_err());
            let mut bytes = [0; 32];
            for (index, word) in words.iter().enumerate() {
                bytes[index * 8..(index + 1) * 8].copy_from_slice(&word.to_le_bytes());
            }
            assert!(Q31VectorState::from_bytes(bytes).is_err());
        }
    }

    #[test]
    fn batch_indices_are_bounded_before_launch() {
        assert!(q31_batch_len_admitted(MAX_Q31_BATCH_ITEMS));
        assert!(!q31_batch_len_admitted(MAX_Q31_BATCH_ITEMS + 1));
    }

    #[test]
    fn saturated_output_stays_in_the_next_state_domain() {
        let edge = [ONE, ONE, 0, 0];
        let output = q31_hamilton_rotate(edge, edge).unwrap();
        assert!(q31_quaternion_admitted(output));
    }

    #[test]
    fn a_finite_credit_sequence_lowers_the_target_loss() {
        let value = [ONE, 0, 0, 0];
        let target = [1_518_500_249, 1_518_500_249, 0, 0];
        let mut gate = [ONE, 0, 0, 0];
        let initial_loss = q31_l1_loss(q31_hamilton_rotate(gate, value).unwrap(), target);
        let mut loss = initial_loss;
        for _ in 0..64 {
            let step = q31_hamilton_train_step(gate, value, target, 8).unwrap();
            assert_eq!(step.loss_before, loss);
            gate = step.accepted_gate;
            loss = q31_l1_loss(q31_hamilton_rotate(gate, value).unwrap(), target);
        }
        assert!(loss < initial_loss, "initial {initial_loss}, final {loss}");
    }

    #[test]
    fn a_nonimproving_candidate_keeps_the_previous_gate() {
        let gate = [ONE, 0, 0, 0];
        let value = [ONE, 0, 0, 0];
        let target = q31_hamilton_rotate(gate, value).unwrap();
        let step = q31_hamilton_train_step(gate, value, target, 8).unwrap();
        assert!(!step.accepted);
        assert_eq!(step.accepted_gate, gate);
        assert_eq!(step.loss_before, 0);
        assert_eq!(step.loss_after, 0);
    }

    #[test]
    fn training_refuses_out_of_domain_words_and_shifts() {
        let valid = [ONE, 0, 0, 0];
        let invalid = [Q31_MAX + 1, 0, 0, 0];
        assert!(q31_hamilton_train_step(invalid, valid, valid, 8).is_err());
        assert!(q31_hamilton_train_step(valid, invalid, valid, 8).is_err());
        assert!(q31_hamilton_train_step(valid, valid, invalid, 8).is_err());
        assert!(q31_hamilton_train_step(valid, valid, valid, 32).is_err());
    }

    #[test]
    fn a_shared_model_step_lowers_batch_loss_and_serializes_its_epoch() {
        let one = 1 << 31;
        let target = [1_518_500_249, 1_518_500_249, 0, 0];
        let values = vec![[one, 0, 0, 0]; 8];
        let targets = vec![target; 8];
        let mut model = Q31HamiltonModel::new([one, 0, 0, 0], 8).unwrap();
        let first = model.step_reference(&values, &targets).unwrap();
        assert_eq!(first.epoch_before, 0);
        for _ in 1..64 {
            model.step_reference(&values, &targets).unwrap();
        }
        assert_eq!(model.epoch(), 64);
        let final_loss = targets
            .iter()
            .zip(&values)
            .map(|(target, value)| {
                q31_l1_loss(q31_hamilton_rotate(model.gate(), *value).unwrap(), *target)
            })
            .sum::<u64>();
        assert!(final_loss < first.loss_before);
        let restored = Q31HamiltonModel::from_bytes(model.to_bytes()).unwrap();
        assert_eq!(restored, model);
    }

    #[test]
    fn shared_model_state_refuses_unknown_bytes_and_wrong_shapes() {
        let model = Q31HamiltonModel::new([ONE, 0, 0, 0], 8).unwrap();
        let mut bytes = model.to_bytes();
        bytes[44] = 2;
        assert!(Q31HamiltonModel::from_bytes(bytes).is_err());
        let mut model = model;
        assert!(model.step_reference(&[], &[]).is_err());
        assert!(model.step_reference(&[[ONE, 0, 0, 0]], &[]).is_err());
    }

    #[test]
    fn a_failed_epoch_increment_leaves_the_model_unchanged() {
        let target = [1_518_500_249, 1_518_500_249, 0, 0];
        let values = vec![[ONE, 0, 0, 0]; 8];
        let targets = vec![target; 8];
        let model = Q31HamiltonModel::new([ONE, 0, 0, 0], 8).unwrap();
        let mut bytes = model.to_bytes();
        bytes[32..40].copy_from_slice(&u64::MAX.to_le_bytes());
        let mut model = Q31HamiltonModel::from_bytes(bytes).unwrap();
        let before = model;

        let error = model.step_reference(&values, &targets).unwrap_err();

        assert!(error.to_string().contains("epoch overflow"), "{error}");
        assert_eq!(model, before);
    }

    #[test]
    fn invalid_model_targets_leave_the_model_unchanged() {
        let model = Q31HamiltonModel::new([ONE, 0, 0, 0], 8).unwrap();
        let mut model = model;
        let before = model;
        let error = model
            .step_reference(&[[ONE, 0, 0, 0]], &[[Q31_MAX + 1, 0, 0, 0]])
            .unwrap_err();

        assert!(error.to_string().contains("training words"), "{error}");
        assert_eq!(model, before);
    }

    #[test]
    fn aggregate_loss_refuses_u64_overflow() {
        let error = checked_q31_loss_sum([Ok(u64::MAX), Ok(1)]).unwrap_err();
        assert!(error.to_string().contains("exceeds u64"), "{error}");
    }
}
