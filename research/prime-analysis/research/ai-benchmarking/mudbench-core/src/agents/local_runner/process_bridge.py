"""Local process runner skeleton for agent protocol exchange."""

from __future__ import annotations

import select
import subprocess
from dataclasses import dataclass, field
from time import monotonic
from typing import Sequence

from agents.gateway.timeout_policy import classify_timeout_boundary_window, classify_timeout_expired
from agents.protocol.action import ActionSubmission, parse_action_submission_payload
from agents.protocol.observation import Observation, require_supported_protocol_version
from agents.protocol.serialization import canonical_json_dumps, json_loads_object


class LocalRunnerError(RuntimeError):
    """Base local runner failure."""


class LocalRunnerTimeoutError(LocalRunnerError):
    """Raised when the agent process exceeds the configured timeout."""


class LocalRunnerProtocolError(LocalRunnerError):
    """Raised when the agent process response is invalid."""


@dataclass(frozen=True, slots=True)
class LocalProcessRunner:
    """One-step local process bridge for observation/action exchange.

    This skeleton executes the configured command per request and exchanges
    newline-delimited JSON over stdin/stdout.
    """

    command: tuple[str, ...]
    persistent_session: bool = False
    _process: subprocess.Popen[str] | None = field(default=None, init=False, repr=False, compare=False)

    def __init__(self, command: Sequence[str], *, persistent_session: bool = False) -> None:
        normalized_command = tuple(command)
        if not normalized_command:
            raise ValueError("LocalProcessRunner requires a non-empty command")
        object.__setattr__(self, "command", normalized_command)
        object.__setattr__(self, "persistent_session", persistent_session)
        object.__setattr__(self, "_process", None)

    def request_action(self, observation: Observation, *, timeout_seconds: float = 1.0) -> ActionSubmission:
        """Send observation to local process and parse action response."""
        if timeout_seconds <= 0:
            raise ValueError("timeout_seconds must be greater than zero")

        try:
            require_supported_protocol_version(observation.protocol_version)
        except ValueError as exc:
            raise LocalRunnerProtocolError(f"incompatible_protocol_version:{exc}") from exc

        payload = canonical_json_dumps(observation.to_dict()) + "\n"
        response_line = (
            self._request_action_persistent(payload, timeout_seconds=timeout_seconds)
            if self.persistent_session
            else self._request_action_single_shot(payload, timeout_seconds=timeout_seconds)
        )

        try:
            payload_object = json_loads_object(response_line)
        except ValueError as exc:
            raise LocalRunnerProtocolError("Agent process returned invalid JSON payload") from exc

        try:
            return parse_action_submission_payload(payload_object)
        except ValueError as exc:
            raise LocalRunnerProtocolError(
                f"Agent process returned invalid action schema: {exc}"
            ) from exc

    def close(self) -> None:
        process = self._process
        if process is None:
            return
        if process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=0.2)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=0.2)
        object.__setattr__(self, "_process", None)

    def _request_action_single_shot(self, payload: str, *, timeout_seconds: float) -> str:
        started_at = monotonic()
        try:
            completed = subprocess.run(
                self.command,
                input=payload,
                capture_output=True,
                text=True,
                timeout=timeout_seconds,
                check=False,
            )
        except subprocess.TimeoutExpired as exc:
            _raise_timeout(timeout_seconds=timeout_seconds, exc=exc)
        except OSError as exc:
            raise LocalRunnerProtocolError(
                f"Agent process failed to start: {exc.strerror or type(exc).__name__}"
            ) from exc
        _validate_boundary_window(started_at=started_at, timeout_seconds=timeout_seconds)

        if completed.returncode != 0:
            details = completed.stderr.strip()
            raise LocalRunnerProtocolError(
                f"Agent process exited with code {completed.returncode}: {details}"
            )

        response_line = _first_non_empty_line(completed.stdout)
        if response_line is None:
            raise LocalRunnerProtocolError("Agent process returned no action payload")
        return response_line

    def _request_action_persistent(self, payload: str, *, timeout_seconds: float) -> str:
        started_at = monotonic()
        process = self._ensure_persistent_process()
        stdin = process.stdin
        stdout = process.stdout
        if stdin is None or stdout is None:
            self.close()
            raise LocalRunnerProtocolError("persistent_session_missing_stdio")

        try:
            stdin.write(payload)
            stdin.flush()
        except (BrokenPipeError, OSError) as exc:
            self.close()
            raise LocalRunnerProtocolError(
                f"persistent_session_write_failed:{type(exc).__name__}"
            ) from exc

        ready_streams, _, _ = select.select((stdout,), (), (), timeout_seconds)
        if not ready_streams:
            if process.poll() is not None:
                error_detail = _process_error_detail(process)
                self.close()
                raise LocalRunnerProtocolError(
                    f"persistent_session_terminated_before_response:{error_detail}"
                )
            self.close()
            _raise_timeout(timeout_seconds=timeout_seconds, exc=None)

        response_line = stdout.readline()
        _validate_boundary_window(started_at=started_at, timeout_seconds=timeout_seconds)

        if not response_line.strip():
            error_detail = _process_error_detail(process)
            self.close()
            raise LocalRunnerProtocolError(
                f"persistent_session_missing_response:{error_detail}"
            )
        if process.poll() is not None:
            error_detail = _process_error_detail(process)
            self.close()
            raise LocalRunnerProtocolError(
                f"persistent_session_terminated_after_response:{error_detail}"
            )
        return response_line.strip()

    def _ensure_persistent_process(self) -> subprocess.Popen[str]:
        process = self._process
        if process is not None and process.poll() is None:
            return process
        if process is not None and process.poll() is not None:
            error_detail = _process_error_detail(process)
            self.close()
            raise LocalRunnerProtocolError(
                f"persistent_session_restarted_disallowed:{error_detail}"
            )
        try:
            process = subprocess.Popen(
                self.command,
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                bufsize=1,
            )
        except OSError as exc:
            raise LocalRunnerProtocolError(
                f"persistent_session_start_failed:{exc.strerror or type(exc).__name__}"
            ) from exc
        object.__setattr__(self, "_process", process)
        return process


def _first_non_empty_line(value: str) -> str | None:
    for raw_line in value.splitlines():
        line = raw_line.strip()
        if line:
            return line
    return None


def _validate_boundary_window(*, started_at: float, timeout_seconds: float) -> None:
    elapsed_seconds = monotonic() - started_at
    boundary_decision = classify_timeout_boundary_window(
        elapsed_seconds=elapsed_seconds,
        timeout_seconds=timeout_seconds,
    )
    if boundary_decision.reason == "environment_sensitive_timeout_boundary_window":
        raise LocalRunnerProtocolError(
            "environment_sensitive_timeout_boundary:"
            f"{boundary_decision.reason}"
            f"(elapsed_seconds={boundary_decision.elapsed_seconds:.6f},"
            f"timeout_seconds={boundary_decision.timeout_seconds:.6f},"
            f"boundary_window_seconds={boundary_decision.boundary_window_seconds:.6f})"
        )


def _raise_timeout(*, timeout_seconds: float, exc: subprocess.TimeoutExpired | None) -> None:
    timeout_decision = classify_timeout_expired(timeout_seconds=timeout_seconds)
    if not timeout_decision.timed_out:
        raise LocalRunnerProtocolError("timeout classification must be timed_out") from exc
    raise LocalRunnerTimeoutError(
        "Agent process timed out: "
        f"{timeout_decision.reason} "
        f"(elapsed_seconds={timeout_decision.elapsed_seconds:.6f}, "
        f"timeout_seconds={timeout_decision.timeout_seconds:.6f})"
    ) from exc


def _process_error_detail(process: subprocess.Popen[str]) -> str:
    returncode = process.poll()
    stderr = ""
    if process.stderr is not None:
        try:
            stderr = process.stderr.read().strip()
        except OSError:
            stderr = ""
    return f"returncode={returncode}:stderr={stderr}"
