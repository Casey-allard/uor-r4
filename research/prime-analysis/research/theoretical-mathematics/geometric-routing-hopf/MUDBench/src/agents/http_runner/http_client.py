"""Minimal deterministic HTTP runner for observe/act exchange."""

from __future__ import annotations

import socket
from dataclasses import dataclass, field
from typing import Any, Callable, Mapping
from urllib import error, request

from agents.protocol.action import ActionSubmission, parse_action_submission_payload
from agents.protocol.observation import Observation, require_supported_protocol_version
from agents.protocol.serialization import canonical_json_dumps, json_loads_object

TransportFn = Callable[[str, str, float], str]


class HttpRunnerError(RuntimeError):
    """Base HTTP runner failure."""


class HttpRunnerTimeoutError(HttpRunnerError):
    """Raised when HTTP communication exceeds timeout."""


class HttpRunnerProtocolError(HttpRunnerError):
    """Raised when HTTP response or transport contract is invalid."""


@dataclass(frozen=True, slots=True)
class HttpRunnerClient:
    """Minimal deterministic client for one-step HTTP action requests."""

    endpoint_url: str
    transport: TransportFn = field(
        default_factory=lambda: _default_transport,
        repr=False,
        compare=False,
    )

    def __post_init__(self) -> None:
        if not isinstance(self.endpoint_url, str) or not self.endpoint_url:
            raise ValueError("endpoint_url must be a non-empty string")

    def request_action(self, observation: Observation, *, timeout_seconds: float = 1.0) -> ActionSubmission:
        """Send observation payload and parse action payload deterministically."""
        if timeout_seconds <= 0:
            raise ValueError("timeout_seconds must be greater than zero")

        try:
            payload = canonical_json_dumps(build_observe_payload(observation))
        except ValueError as exc:
            raise HttpRunnerProtocolError(f"incompatible_protocol_version:{exc}") from exc
        try:
            response_payload = self.transport(self.endpoint_url, payload, timeout_seconds)
        except TimeoutError as exc:
            raise HttpRunnerTimeoutError("HTTP request timed out") from exc
        except HttpRunnerError:
            raise

        try:
            return parse_action_payload_json(response_payload)
        except ValueError as exc:
            raise HttpRunnerProtocolError(f"HTTP response payload invalid: {exc}") from exc


def build_observe_payload(observation: Observation) -> dict[str, Any]:
    """Return deterministic observation payload for HTTP transport."""
    protocol_version = require_supported_protocol_version(observation.protocol_version)
    return {
        "protocol_version": protocol_version,
        "observation": observation.to_dict(),
    }


def parse_action_payload(payload: Mapping[str, Any]) -> ActionSubmission:
    """Parse deterministic action payload for HTTP transport."""
    return parse_action_submission_payload(payload)


def parse_action_payload_json(raw_payload: str) -> ActionSubmission:
    """Parse deterministic JSON action payload for HTTP transport."""
    return parse_action_payload(json_loads_object(raw_payload))


def _default_transport(endpoint_url: str, payload: str, timeout_seconds: float) -> str:
    body = payload.encode("utf-8")
    http_request = request.Request(
        endpoint_url,
        data=body,
        headers={"Content-Type": "application/json"},
        method="POST",
    )

    try:
        with request.urlopen(http_request, timeout=timeout_seconds) as response:
            status = int(response.getcode())
            if status < 200 or status >= 300:
                raise HttpRunnerProtocolError(
                    f"HTTP request returned non-2xx status {status}"
                )

            raw_body = response.read()
    except error.HTTPError as exc:
        raise HttpRunnerProtocolError(
            f"HTTP request failed with status {exc.code}"
        ) from exc
    except error.URLError as exc:
        reason = exc.reason
        if isinstance(reason, (TimeoutError, socket.timeout)):
            raise HttpRunnerTimeoutError("HTTP request timed out") from exc
        raise HttpRunnerProtocolError(f"HTTP transport error: {reason}") from exc
    except TimeoutError as exc:
        raise HttpRunnerTimeoutError("HTTP request timed out") from exc

    try:
        return raw_body.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise HttpRunnerProtocolError("HTTP response body must be UTF-8") from exc
