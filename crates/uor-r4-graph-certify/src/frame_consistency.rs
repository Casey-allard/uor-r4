//! #647 frame-consistency guard: a shared anti-vacuity control for
//! comparison surfaces, guarding a null/FAIL against a reference-frame
//! false zero.
//!
//! Measured basis (historical program): a reference-frame rotation
//! (`apply_anchor_two_i`, a 90-degree class shift) drove
//! `cos(rotated_state, raw_prototype)` to EXACTLY 0 for every class, every
//! timestep, across all 1024 eval samples — a categorical FALSE null that
//! looks like "no signal." Migrating both operands into the same frame
//! restored `cos = 1.0` for matched classes (matched/mismatched separation
//! 1.327). #486 was a live instance: a routing vector compared against a
//! stored content vector read at chance until the comparison was corrected.
//!
//! The discipline: before a comparison surface accepts a null or FAIL as
//! INFORMATIVE, it confirms a MATCHED-PAIR control — a same-frame positive
//! comparison that MUST be materially nonzero if the instrument is
//! correctly framed (a self-similarity that should be ~1, a matched-class
//! score well above chance, a fitted overlap that should be far from zero).
//! If that control is ~0, the whole instrument is likely mis-framed, so the
//! null/FAIL is reported as UNAVAILABLE (frame mismatch suspected), never as
//! a real FAIL — extending the #599 three-state discipline
//! (PASS / FAIL / UNAVAILABLE), never manufacturing a FAIL from a
//! mis-framed instrument.
//!
//! Adoption (no mechanism change; no new operator): route-fit (#605)
//! already embodies this via its pre-registered N2 shifted-support
//! anti-vacuity gate — a mis-framed instrument reads vacuous and fails the
//! whole run as an instrument fault rather than as a real per-scope miss.
//! New or updated comparison surfaces (#599 adapter conformance, #602
//! operator specs, #606 certificate rows) adopt this primitive by DECLARING
//! a frame-control in their (pre-registered) contract and routing its
//! outcome through [`FrameControl`]; they never retroactively bolt a new
//! verdict branch onto an already-pre-registered instrument, which would
//! break its declared identity.

/// The default materiality floor for a matched-pair control: a control at
/// or below this is treated as indistinguishable from zero (frame mismatch
/// suspected). Deliberately small — the guard fires on a categorical false
/// zero (the measured failure produced EXACTLY 0), not on a weak-but-real
/// signal, which remains a genuine (informative) low score.
pub const FRAME_CONTROL_EPSILON: f64 = 1e-6;

/// The outcome of the frame-consistency guard over one comparison's
/// matched-pair control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameControl {
    /// The matched-pair control is materially nonzero: the instrument is
    /// correctly framed, so a null/FAIL it produced is INFORMATIVE and may
    /// be accepted at face value.
    Framed,
    /// The matched-pair control is ~0 (or non-finite): the instrument is
    /// likely mis-framed, so any null/FAIL it produced is UNAVAILABLE
    /// (frame mismatch suspected), never a real FAIL.
    MismatchSuspected,
}

impl FrameControl {
    /// Whether a null/FAIL from this instrument is informative — i.e. the
    /// instrument is correctly framed and the negative result is real.
    pub fn null_is_informative(self) -> bool {
        matches!(self, FrameControl::Framed)
    }

    /// Whether a mis-framed instrument was detected (the null/FAIL must be
    /// suppressed to UNAVAILABLE rather than reported as a real FAIL).
    pub fn is_frame_mismatch(self) -> bool {
        matches!(self, FrameControl::MismatchSuspected)
    }
}

/// Guard a comparison's null/FAIL against a frame-mismatch false zero.
///
/// `matched_pair_control` is a same-frame positive comparison that MUST be
/// materially nonzero when the instrument is correctly framed (for example
/// a self-similarity near 1, or a matched-class score well above chance). A
/// non-finite control, or one at or below `epsilon`, means the null/FAIL
/// cannot be trusted and the instrument reads as mis-framed.
pub fn frame_control(matched_pair_control: f64, epsilon: f64) -> FrameControl {
    if matched_pair_control.is_finite() && matched_pair_control > epsilon {
        FrameControl::Framed
    } else {
        FrameControl::MismatchSuspected
    }
}

/// [`frame_control`] at the default [`FRAME_CONTROL_EPSILON`].
pub fn frame_control_default(matched_pair_control: f64) -> FrameControl {
    frame_control(matched_pair_control, FRAME_CONTROL_EPSILON)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_materially_nonzero_control_frames_the_instrument() {
        // Matched-pair self-similarity ~1: a null/FAIL here is real.
        assert_eq!(frame_control_default(1.0), FrameControl::Framed);
        assert!(frame_control_default(1.0).null_is_informative());
        // A weak but real signal is still framed — the guard is not a
        // strength threshold, only a false-zero detector.
        assert_eq!(frame_control_default(1e-3), FrameControl::Framed);
    }

    #[test]
    fn a_categorical_false_zero_reads_as_frame_mismatch() {
        // The measured failure: exactly 0 across every class/timestep.
        assert_eq!(frame_control_default(0.0), FrameControl::MismatchSuspected);
        assert!(frame_control_default(0.0).is_frame_mismatch());
        assert!(!frame_control_default(0.0).null_is_informative());
    }

    #[test]
    fn the_epsilon_floor_is_exclusive() {
        // At the floor is suspected; strictly above the floor is framed.
        assert_eq!(
            frame_control(FRAME_CONTROL_EPSILON, FRAME_CONTROL_EPSILON),
            FrameControl::MismatchSuspected
        );
        assert_eq!(
            frame_control(2.0 * FRAME_CONTROL_EPSILON, FRAME_CONTROL_EPSILON),
            FrameControl::Framed
        );
    }

    #[test]
    fn non_finite_or_negative_controls_are_frame_mismatch() {
        for control in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0] {
            assert_eq!(
                frame_control_default(control),
                FrameControl::MismatchSuspected,
                "control {control} must read as mis-framed"
            );
        }
    }
}
