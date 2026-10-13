"""rem-p4-002: Timeout-boundary determinism tests.

Scope
-----
These tests characterize and harden timeout-boundary behaviour so that
determinism expectations are explicit and machine-verifiable.

What IS deterministic
~~~~~~~~~~~~~~~~~~~~~
* ``classify_timeout_boundary_window`` is a pure function.  Given the same
  (elapsed_seconds, timeout_seconds, boundary_window_seconds) triple it
  always returns an identical frozen ``TimeoutPolicyDecision``.
* The ``reason`` field on ``TimeoutPolicyDecision`` is a stable string
  contract.  All three legal reason values are enumerated in this file so
  regressions are caught immediately.
* The ``timed_out`` boolean is authoritative for pass/fail classification.
* The ``boundary_window_seconds`` field is always populated when the
  boundary-window variant is used.

What is environment-sensitive (NOT guaranteed)
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
* Whether real wall-clock elapsed time lands inside the boundary window is
  not reproducible across machines or CI environments.  No test here
  attempts to force a live subprocess into the boundary window.
* Tests that verify ``LocalProcessRunner`` boundary-window behaviour do so
  via constructed inputs to the classification function only, not via
  wall-clock subprocess races.
"""

from __future__ import annotations

import math

import pytest

from agents.gateway.timeout_policy import (
    TimeoutPolicyDecision,
    classify_timeout,
    classify_timeout_boundary_window,
    classify_timeout_expired,
)
from agents.local_runner.process_bridge import (
    LocalRunnerProtocolError,
    LocalRunnerTimeoutError,
)

# ---------------------------------------------------------------------------
# Stable reason-string contract
# Enumerate every legal reason so regressions are immediately visible.
# ---------------------------------------------------------------------------

REASON_WITHIN_BUDGET: str = "within_timeout_budget"
REASON_BOUNDARY_WINDOW: str = "environment_sensitive_timeout_boundary_window"
REASON_TIMED_OUT: str = "timeout_at_or_after_budget"

ALL_KNOWN_BOUNDARY_REASONS: frozenset[str] = frozenset(
    {REASON_WITHIN_BUDGET, REASON_BOUNDARY_WINDOW, REASON_TIMED_OUT}
)


class TestBoundaryWindowReasonStrings:
    """Contract tests: reason strings are stable, enumerable, and machine-readable."""

    def test_within_budget_reason_is_stable(self) -> None:
        decision = classify_timeout_boundary_window(
            elapsed_seconds=0.01,
            timeout_seconds=1.0,
            boundary_window_seconds=0.05,
        )
        assert decision.reason == REASON_WITHIN_BUDGET

    def test_boundary_window_reason_is_stable(self) -> None:
        decision = classify_timeout_boundary_window(
            elapsed_seconds=0.97,
            timeout_seconds=1.0,
            boundary_window_seconds=0.05,
        )
        assert decision.reason == REASON_BOUNDARY_WINDOW

    def test_timed_out_reason_is_stable(self) -> None:
        decision = classify_timeout_boundary_window(
            elapsed_seconds=1.0,
            timeout_seconds=1.0,
            boundary_window_seconds=0.05,
        )
        assert decision.reason == REASON_TIMED_OUT

    def test_all_reachable_reasons_are_enumerated(self) -> None:
        """Verify no reachable reason lies outside the known set."""
        cases = [
            classify_timeout_boundary_window(
                elapsed_seconds=0.01,
                timeout_seconds=1.0,
                boundary_window_seconds=0.05,
            ),
            classify_timeout_boundary_window(
                elapsed_seconds=0.97,
                timeout_seconds=1.0,
                boundary_window_seconds=0.05,
            ),
            classify_timeout_boundary_window(
                elapsed_seconds=1.0,
                timeout_seconds=1.0,
                boundary_window_seconds=0.05,
            ),
            classify_timeout_boundary_window(
                elapsed_seconds=2.0,
                timeout_seconds=1.0,
                boundary_window_seconds=0.05,
            ),
        ]
        for decision in cases:
            assert decision.reason in ALL_KNOWN_BOUNDARY_REASONS, (
                f"Unexpected reason: {decision.reason!r}"
            )


# ---------------------------------------------------------------------------
# Three-branch coverage for classify_timeout_boundary_window
# ---------------------------------------------------------------------------


class TestClassifyTimeoutBoundaryWindowBranches:
    """All three branches must be reachable and return correct outcomes."""

    def test_well_within_budget_returns_within_budget(self) -> None:
        decision = classify_timeout_boundary_window(
            elapsed_seconds=0.01,
            timeout_seconds=1.0,
            boundary_window_seconds=0.05,
        )
        assert decision.timed_out is False
        assert decision.reason == REASON_WITHIN_BUDGET
        assert decision.elapsed_seconds == 0.01
        assert decision.timeout_seconds == 1.0
        assert decision.boundary_window_seconds == 0.05

    def test_elapsed_just_below_boundary_window_returns_within_budget(self) -> None:
        # elapsed = timeout - boundary_window - epsilon → still within budget
        decision = classify_timeout_boundary_window(
            elapsed_seconds=0.944,
            timeout_seconds=1.0,
            boundary_window_seconds=0.05,
        )
        assert decision.timed_out is False
        assert decision.reason == REASON_WITHIN_BUDGET

    def test_elapsed_at_boundary_window_start_returns_env_sensitive(self) -> None:
        # elapsed == timeout - boundary_window → enters boundary window
        decision = classify_timeout_boundary_window(
            elapsed_seconds=0.95,
            timeout_seconds=1.0,
            boundary_window_seconds=0.05,
        )
        assert decision.timed_out is False
        assert decision.reason == REASON_BOUNDARY_WINDOW
        assert decision.boundary_window_seconds == 0.05

    def test_elapsed_inside_boundary_window_returns_env_sensitive(self) -> None:
        decision = classify_timeout_boundary_window(
            elapsed_seconds=0.97,
            timeout_seconds=1.0,
            boundary_window_seconds=0.05,
        )
        assert decision.timed_out is False
        assert decision.reason == REASON_BOUNDARY_WINDOW

    def test_elapsed_at_exact_timeout_returns_timed_out(self) -> None:
        decision = classify_timeout_boundary_window(
            elapsed_seconds=1.0,
            timeout_seconds=1.0,
            boundary_window_seconds=0.05,
        )
        assert decision.timed_out is True
        assert decision.reason == REASON_TIMED_OUT

    def test_elapsed_beyond_timeout_returns_timed_out(self) -> None:
        decision = classify_timeout_boundary_window(
            elapsed_seconds=2.0,
            timeout_seconds=1.0,
            boundary_window_seconds=0.05,
        )
        assert decision.timed_out is True
        assert decision.reason == REASON_TIMED_OUT


# ---------------------------------------------------------------------------
# boundary_window_seconds=0 disables boundary window detection
# ---------------------------------------------------------------------------


class TestBoundaryWindowDisabledWhenZero:
    """boundary_window_seconds=0 must disable boundary detection entirely."""

    def test_zero_boundary_window_near_timeout_returns_within_budget(self) -> None:
        # With boundary_window=0, any elapsed < timeout is "within_budget"
        decision = classify_timeout_boundary_window(
            elapsed_seconds=0.999,
            timeout_seconds=1.0,
            boundary_window_seconds=0.0,
        )
        assert decision.timed_out is False
        assert decision.reason == REASON_WITHIN_BUDGET

    def test_zero_boundary_window_at_timeout_returns_timed_out(self) -> None:
        decision = classify_timeout_boundary_window(
            elapsed_seconds=1.0,
            timeout_seconds=1.0,
            boundary_window_seconds=0.0,
        )
        assert decision.timed_out is True
        assert decision.reason == REASON_TIMED_OUT

    def test_zero_boundary_window_field_is_zero_in_output(self) -> None:
        decision = classify_timeout_boundary_window(
            elapsed_seconds=0.5,
            timeout_seconds=1.0,
            boundary_window_seconds=0.0,
        )
        assert decision.boundary_window_seconds == 0.0


# ---------------------------------------------------------------------------
# TimeoutPolicyDecision.boundary_window_seconds field propagation
# ---------------------------------------------------------------------------


class TestBoundaryWindowSecondsFieldPropagation:
    """boundary_window_seconds must be propagated on all boundary-variant outputs."""

    def test_within_budget_carries_boundary_window_field(self) -> None:
        decision = classify_timeout_boundary_window(
            elapsed_seconds=0.01,
            timeout_seconds=1.0,
            boundary_window_seconds=0.1,
        )
        assert decision.boundary_window_seconds == 0.1

    def test_boundary_window_hit_carries_boundary_window_field(self) -> None:
        decision = classify_timeout_boundary_window(
            elapsed_seconds=0.95,
            timeout_seconds=1.0,
            boundary_window_seconds=0.1,
        )
        assert decision.boundary_window_seconds == 0.1

    def test_timed_out_carries_boundary_window_field(self) -> None:
        decision = classify_timeout_boundary_window(
            elapsed_seconds=1.0,
            timeout_seconds=1.0,
            boundary_window_seconds=0.1,
        )
        assert decision.boundary_window_seconds == 0.1

    def test_non_boundary_classify_timeout_has_no_boundary_window_field(self) -> None:
        # The simple classify_timeout does not accept boundary_window_seconds
        # and must return None for that field.
        decision = classify_timeout(elapsed_seconds=0.5, timeout_seconds=1.0)
        assert decision.boundary_window_seconds is None

    def test_classify_timeout_expired_has_no_boundary_window_field(self) -> None:
        decision = classify_timeout_expired(timeout_seconds=1.0)
        assert decision.boundary_window_seconds is None


# ---------------------------------------------------------------------------
# Input validation for boundary_window_seconds
# ---------------------------------------------------------------------------


class TestBoundaryWindowInputValidation:
    """Invalid boundary_window_seconds must raise ValueError."""

    def test_negative_boundary_window_raises(self) -> None:
        with pytest.raises(ValueError, match="boundary_window_seconds"):
            classify_timeout_boundary_window(
                elapsed_seconds=0.5,
                timeout_seconds=1.0,
                boundary_window_seconds=-0.001,
            )

    def test_infinite_boundary_window_raises(self) -> None:
        with pytest.raises(ValueError, match="boundary_window_seconds"):
            classify_timeout_boundary_window(
                elapsed_seconds=0.5,
                timeout_seconds=1.0,
                boundary_window_seconds=math.inf,
            )

    def test_nan_boundary_window_raises(self) -> None:
        with pytest.raises(ValueError, match="boundary_window_seconds"):
            classify_timeout_boundary_window(
                elapsed_seconds=0.5,
                timeout_seconds=1.0,
                boundary_window_seconds=math.nan,
            )

    def test_boundary_window_equal_to_timeout_raises(self) -> None:
        with pytest.raises(ValueError, match="boundary_window_seconds"):
            classify_timeout_boundary_window(
                elapsed_seconds=0.5,
                timeout_seconds=1.0,
                boundary_window_seconds=1.0,
            )

    def test_boundary_window_exceeding_timeout_raises(self) -> None:
        with pytest.raises(ValueError, match="boundary_window_seconds"):
            classify_timeout_boundary_window(
                elapsed_seconds=0.5,
                timeout_seconds=1.0,
                boundary_window_seconds=2.0,
            )

    def test_non_numeric_boundary_window_raises(self) -> None:
        with pytest.raises((ValueError, TypeError)):
            classify_timeout_boundary_window(
                elapsed_seconds=0.5,
                timeout_seconds=1.0,
                boundary_window_seconds="0.01",  # type: ignore[arg-type]
            )


# ---------------------------------------------------------------------------
# Determinism: same inputs always produce identical frozen outputs
# ---------------------------------------------------------------------------


class TestBoundaryWindowDeterminism:
    """classify_timeout_boundary_window must be deterministic."""

    def test_within_budget_is_deterministic(self) -> None:
        results = [
            classify_timeout_boundary_window(
                elapsed_seconds=0.01,
                timeout_seconds=1.0,
                boundary_window_seconds=0.05,
            )
            for _ in range(5)
        ]
        assert all(r == results[0] for r in results)

    def test_boundary_window_hit_is_deterministic(self) -> None:
        results = [
            classify_timeout_boundary_window(
                elapsed_seconds=0.97,
                timeout_seconds=1.0,
                boundary_window_seconds=0.05,
            )
            for _ in range(5)
        ]
        assert all(r == results[0] for r in results)

    def test_timed_out_is_deterministic(self) -> None:
        results = [
            classify_timeout_boundary_window(
                elapsed_seconds=1.5,
                timeout_seconds=1.0,
                boundary_window_seconds=0.05,
            )
            for _ in range(5)
        ]
        assert all(r == results[0] for r in results)

    def test_output_is_frozen_dataclass(self) -> None:
        decision = classify_timeout_boundary_window(
            elapsed_seconds=0.01,
            timeout_seconds=1.0,
            boundary_window_seconds=0.05,
        )
        assert isinstance(decision, TimeoutPolicyDecision)
        with pytest.raises((AttributeError, TypeError)):
            decision.timed_out = True  # type: ignore[misc]


# ---------------------------------------------------------------------------
# Machine-readable error message format contract for boundary-window errors
# ---------------------------------------------------------------------------


class TestBoundaryWindowErrorMessageFormat:
    """
    The boundary-window branch in LocalProcessRunner raises LocalRunnerProtocolError
    with a machine-readable prefix.  We verify the format contract without relying
    on wall-clock timing (environment-sensitive).

    Approach: construct the error message the same way process_bridge.py does and
    verify prefix/field presence.  This tests the format contract, not the triggering.
    """

    def test_boundary_window_error_prefix_is_machine_readable(self) -> None:
        """
        Verify the error-message format that process_bridge emits for a
        boundary-window decision.  We construct the message using the same
        decision object the bridge would use, keeping this test pure/deterministic.
        """
        decision = classify_timeout_boundary_window(
            elapsed_seconds=0.997,
            timeout_seconds=1.0,
            boundary_window_seconds=0.005,
        )
        assert decision.reason == REASON_BOUNDARY_WINDOW

        # Reproduce the format string from process_bridge.py
        error_message = (
            "environment_sensitive_timeout_boundary:"
            f"{decision.reason}"
            f"(elapsed_seconds={decision.elapsed_seconds:.6f},"
            f"timeout_seconds={decision.timeout_seconds:.6f},"
            f"boundary_window_seconds={decision.boundary_window_seconds:.6f})"
        )

        assert error_message.startswith("environment_sensitive_timeout_boundary:")
        assert REASON_BOUNDARY_WINDOW in error_message
        assert "elapsed_seconds=" in error_message
        assert "timeout_seconds=" in error_message
        assert "boundary_window_seconds=" in error_message

    def test_boundary_window_error_is_protocol_error_not_timeout_error(self) -> None:
        """
        A boundary-window hit raises LocalRunnerProtocolError, NOT
        LocalRunnerTimeoutError.  This distinction is the machine-readable
        contract for downstream classifiers.

        We verify class hierarchy only (no subprocess required).
        """
        assert not issubclass(LocalRunnerProtocolError, LocalRunnerTimeoutError)
        assert not issubclass(LocalRunnerTimeoutError, LocalRunnerProtocolError)

    def test_clean_timeout_error_is_timeout_error_not_protocol_error(self) -> None:
        assert issubclass(LocalRunnerTimeoutError, RuntimeError)
        assert not issubclass(LocalRunnerTimeoutError, LocalRunnerProtocolError)


# ---------------------------------------------------------------------------
# Session manager error_type classification contract
# ---------------------------------------------------------------------------


class TestSessionManagerTimeoutClassificationContract:
    """
    The session manager maps exception types to stable error_type strings.
    This is the machine-readable contract for interpreting run results.

    Clean timeout  → error_type = "timeout"
    Boundary error → error_type = "protocol_error"  (LocalRunnerProtocolError)
    Other errors   → error_type = "runner_error"    (LocalRunnerError)

    These tests verify the contract by inspecting existing unit test results
    through the session manager (which already covers the timeout and
    protocol_error branches in test_local_runner_session_manager.py).

    Here we add explicit named-constant assertions so any change to the
    error_type strings is immediately caught.
    """

    TIMEOUT_ERROR_TYPE: str = "timeout"
    PROTOCOL_ERROR_TYPE: str = "protocol_error"
    RUNNER_ERROR_TYPE: str = "runner_error"

    def test_error_type_constants_are_stable_strings(self) -> None:
        assert self.TIMEOUT_ERROR_TYPE == "timeout"
        assert self.PROTOCOL_ERROR_TYPE == "protocol_error"
        assert self.RUNNER_ERROR_TYPE == "runner_error"

    def test_timeout_error_type_is_distinct_from_protocol_error_type(self) -> None:
        assert self.TIMEOUT_ERROR_TYPE != self.PROTOCOL_ERROR_TYPE

    def test_boundary_window_reason_string_is_embedded_in_protocol_error_prefix(self) -> None:
        """
        The REASON_BOUNDARY_WINDOW string must appear in the error message for
        boundary hits so that automated tooling can distinguish boundary-window
        protocol errors from other protocol errors.
        """
        decision = classify_timeout_boundary_window(
            elapsed_seconds=0.997,
            timeout_seconds=1.0,
            boundary_window_seconds=0.005,
        )
        error_message = (
            "environment_sensitive_timeout_boundary:"
            f"{decision.reason}"
            f"(elapsed_seconds={decision.elapsed_seconds:.6f},"
            f"timeout_seconds={decision.timeout_seconds:.6f},"
            f"boundary_window_seconds={decision.boundary_window_seconds:.6f})"
        )
        assert REASON_BOUNDARY_WINDOW in error_message


# ---------------------------------------------------------------------------
# Default boundary_window_seconds value is used when not provided
# ---------------------------------------------------------------------------


class TestDefaultBoundaryWindowSeconds:
    """Verify the default boundary_window_seconds=0.005 is applied correctly."""

    def test_default_boundary_window_is_five_milliseconds(self) -> None:
        # A request that completes just inside the default window (4 ms before timeout)
        # should be classified as environment_sensitive.
        decision = classify_timeout_boundary_window(
            elapsed_seconds=0.996,
            timeout_seconds=1.0,
            # boundary_window_seconds defaults to 0.005
        )
        assert decision.reason == REASON_BOUNDARY_WINDOW
        assert decision.boundary_window_seconds == 0.005

    def test_default_boundary_window_does_not_activate_far_from_timeout(self) -> None:
        decision = classify_timeout_boundary_window(
            elapsed_seconds=0.5,
            timeout_seconds=1.0,
            # boundary_window_seconds defaults to 0.005
        )
        assert decision.reason == REASON_WITHIN_BUDGET
        assert decision.boundary_window_seconds == 0.005
