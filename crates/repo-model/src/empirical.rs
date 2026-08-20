//! The empirical verdict vocabulary (#830, item B of #821).
//!
//! `level = build` in the register means the *harness is constructed and
//! validated against its oracle* — a structural fact about what was built. It is
//! deliberately NOT an empirical PASS. An empirical verdict is a separate axis
//! with exactly three values, and this type makes the load-bearing rule
//! unrepresentable to violate: a fixture that is absent is
//! [`EmpiricalStatus::Unavailable`], never [`EmpiricalStatus::Pass`]. A green
//! structural suite can never, on its own, mint an empirical certificate.
//!
//! This is the register-level expression of the RF-29 hazard (a teacher-parity
//! benchmark that "vacuously passes" when its pinned fixtures are absent): the
//! only path to `Pass` requires the fixture to be present *and* the run to have
//! met its pre-declared criterion.

use std::path::Path;

/// The three empirical statuses. Exactly PASS, FAIL, or UNAVAILABLE (#830).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmpiricalStatus {
    /// A fixture-present, CID-bound run met its pre-declared criterion.
    Pass,
    /// A fixture-present, CID-bound run ran but did not meet its criterion.
    Fail,
    /// The fixture was absent, so nothing was measured. Never a PASS.
    Unavailable,
}

impl EmpiricalStatus {
    /// The token used in generated documentation and serialized reports.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::Unavailable => "UNAVAILABLE",
        }
    }

    /// Whether this verdict is a passing empirical result.
    pub const fn is_pass(self) -> bool {
        matches!(self, Self::Pass)
    }

    /// Resolve an empirical verdict from fixture presence and the run outcome.
    ///
    /// The only path to [`EmpiricalStatus::Pass`] is a fixture that is present
    /// *and* a run that met its criterion; absence collapses to
    /// [`EmpiricalStatus::Unavailable`] before the run outcome is even
    /// consulted. This is the non-vacuity rule of #830 expressed as control
    /// flow: a missing fixture cannot produce PASS.
    pub const fn resolve(fixture_present: bool, run_met_criterion: bool) -> Self {
        match (fixture_present, run_met_criterion) {
            (false, _) => Self::Unavailable,
            (true, true) => Self::Pass,
            (true, false) => Self::Fail,
        }
    }

    /// Resolve by checking that a fixture path actually exists on disk, then
    /// folding in the run outcome. A path that does not exist is
    /// [`EmpiricalStatus::Unavailable`] — the serialized form of "fixtures
    /// absent ⇒ no empirical claim".
    pub fn for_fixture(fixture: &Path, run_met_criterion: bool) -> Self {
        Self::resolve(fixture.exists(), run_met_criterion)
    }
}

impl core::fmt::Display for EmpiricalStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The negative control: an absent fixture is UNAVAILABLE regardless of what
    /// the would-be run outcome was — absence cannot mint a PASS (#830).
    #[test]
    fn an_absent_fixture_is_unavailable_never_pass() {
        assert_eq!(
            EmpiricalStatus::resolve(false, true),
            EmpiricalStatus::Unavailable
        );
        assert_eq!(
            EmpiricalStatus::resolve(false, false),
            EmpiricalStatus::Unavailable
        );
        // Exhaustive: there is no (present=false, _) input that yields Pass.
        for run_met in [true, false] {
            assert!(!EmpiricalStatus::resolve(false, run_met).is_pass());
        }
    }

    /// The positive control: the type CAN report PASS and FAIL, so the negative
    /// above is not vacuous (an instrument that can never PASS proves nothing).
    #[test]
    fn a_present_fixture_reflects_the_run_outcome() {
        assert_eq!(EmpiricalStatus::resolve(true, true), EmpiricalStatus::Pass);
        assert_eq!(EmpiricalStatus::resolve(true, false), EmpiricalStatus::Fail);
    }

    #[test]
    fn statuses_serialize_to_the_three_tokens() {
        assert_eq!(EmpiricalStatus::Pass.as_str(), "PASS");
        assert_eq!(EmpiricalStatus::Fail.as_str(), "FAIL");
        assert_eq!(EmpiricalStatus::Unavailable.as_str(), "UNAVAILABLE");
    }
}
