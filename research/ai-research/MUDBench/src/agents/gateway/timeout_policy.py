"""Deterministic per-step timeout policy for gateway agent calls."""

from __future__ import annotations

from dataclasses import dataclass
from math import isfinite


@dataclass(frozen=True, slots=True)
class TimeoutPolicyDecision:
    """Deterministic timeout classification result."""

    timed_out: bool
    reason: str
    elapsed_seconds: float
    timeout_seconds: float
    boundary_window_seconds: float | None = None


def classify_timeout(*, elapsed_seconds: float, timeout_seconds: float) -> TimeoutPolicyDecision:
    """Classify elapsed time against a timeout budget deterministically."""
    elapsed = _coerce_non_negative_finite(elapsed_seconds, field_name="elapsed_seconds")
    timeout = _coerce_positive_finite(timeout_seconds, field_name="timeout_seconds")

    if elapsed >= timeout:
        return TimeoutPolicyDecision(
            timed_out=True,
            reason="timeout_at_or_after_budget",
            elapsed_seconds=elapsed,
            timeout_seconds=timeout,
        )

    return TimeoutPolicyDecision(
        timed_out=False,
        reason="within_timeout_budget",
        elapsed_seconds=elapsed,
        timeout_seconds=timeout,
    )


def classify_timeout_expired(*, timeout_seconds: float) -> TimeoutPolicyDecision:
    """Classify a timeout-expired event at the configured timeout boundary."""
    timeout = _coerce_positive_finite(timeout_seconds, field_name="timeout_seconds")
    return classify_timeout(elapsed_seconds=timeout, timeout_seconds=timeout)


def classify_timeout_boundary_window(
    *,
    elapsed_seconds: float,
    timeout_seconds: float,
    boundary_window_seconds: float = 0.005,
) -> TimeoutPolicyDecision:
    """Classify timeout status with explicit near-boundary sensitivity reporting."""
    elapsed = _coerce_non_negative_finite(elapsed_seconds, field_name="elapsed_seconds")
    timeout = _coerce_positive_finite(timeout_seconds, field_name="timeout_seconds")
    boundary_window = _coerce_boundary_window_seconds(
        boundary_window_seconds,
        timeout_seconds=timeout,
    )

    if elapsed >= timeout:
        return TimeoutPolicyDecision(
            timed_out=True,
            reason="timeout_at_or_after_budget",
            elapsed_seconds=elapsed,
            timeout_seconds=timeout,
            boundary_window_seconds=boundary_window,
        )

    if boundary_window > 0 and elapsed >= timeout - boundary_window:
        return TimeoutPolicyDecision(
            timed_out=False,
            reason="environment_sensitive_timeout_boundary_window",
            elapsed_seconds=elapsed,
            timeout_seconds=timeout,
            boundary_window_seconds=boundary_window,
        )

    return TimeoutPolicyDecision(
        timed_out=False,
        reason="within_timeout_budget",
        elapsed_seconds=elapsed,
        timeout_seconds=timeout,
        boundary_window_seconds=boundary_window,
    )


def _coerce_non_negative_finite(value: float, *, field_name: str) -> float:
    if not isinstance(value, (int, float)):
        raise ValueError(f"{field_name} must be a finite non-negative number")
    normalized = float(value)
    if not isfinite(normalized) or normalized < 0:
        raise ValueError(f"{field_name} must be a finite non-negative number")
    return normalized


def _coerce_positive_finite(value: float, *, field_name: str) -> float:
    if not isinstance(value, (int, float)):
        raise ValueError(f"{field_name} must be a finite number greater than zero")
    normalized = float(value)
    if not isfinite(normalized) or normalized <= 0:
        raise ValueError(f"{field_name} must be a finite number greater than zero")
    return normalized


def _coerce_boundary_window_seconds(value: float, *, timeout_seconds: float) -> float:
    if not isinstance(value, (int, float)):
        raise ValueError("boundary_window_seconds must be a finite non-negative number")
    normalized = float(value)
    if not isfinite(normalized) or normalized < 0:
        raise ValueError("boundary_window_seconds must be a finite non-negative number")
    if normalized >= timeout_seconds:
        raise ValueError("boundary_window_seconds must be less than timeout_seconds")
    return normalized
