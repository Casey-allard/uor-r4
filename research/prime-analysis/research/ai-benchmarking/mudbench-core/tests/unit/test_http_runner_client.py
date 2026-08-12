from __future__ import annotations

import socket
from urllib import error

import pytest

from agents.http_runner.http_client import (
    HttpRunnerClient,
    HttpRunnerProtocolError,
    HttpRunnerTimeoutError,
    build_observe_payload,
    parse_action_payload_json,
)
from agents.protocol.observation import Observation
from agents.protocol.serialization import canonical_json_dumps


def _sample_observation() -> Observation:
    return Observation.from_dict(
        {
            "run_id": "run-http",
            "step": 2,
            "location": "room-a",
            "description": "A room.",
            "exits": ["north"],
            "entities": [],
            "inventory": [],
            "health": 100,
            "messages": [],
            "action_space": ["wait", "move north"],
            "remaining_steps": 8,
        }
    )


def test_http_runner_client_success_is_deterministic() -> None:
    captured_payloads: list[str] = []

    def transport(url: str, payload: str, timeout_seconds: float) -> str:
        assert url == "http://localhost:8000/act"
        assert timeout_seconds == 0.25
        captured_payloads.append(payload)
        return '{"action":"wait"}'

    client = HttpRunnerClient(endpoint_url="http://localhost:8000/act", transport=transport)
    observation = _sample_observation()

    first = client.request_action(observation, timeout_seconds=0.25)
    second = client.request_action(observation, timeout_seconds=0.25)

    expected_payload = canonical_json_dumps(build_observe_payload(observation))
    assert first.action == "wait"
    assert second.action == "wait"
    assert captured_payloads == [expected_payload, expected_payload]


def test_http_runner_client_timeout_is_explicit() -> None:
    def transport(url: str, payload: str, timeout_seconds: float) -> str:
        raise TimeoutError("boom")

    client = HttpRunnerClient(endpoint_url="http://localhost:8000/act", transport=transport)
    with pytest.raises(HttpRunnerTimeoutError, match="timed out"):
        client.request_action(_sample_observation(), timeout_seconds=0.1)


def test_http_runner_client_invalid_json_is_protocol_error() -> None:
    def transport(url: str, payload: str, timeout_seconds: float) -> str:
        return "not-json"

    client = HttpRunnerClient(endpoint_url="http://localhost:8000/act", transport=transport)
    with pytest.raises(HttpRunnerProtocolError, match="payload invalid"):
        client.request_action(_sample_observation(), timeout_seconds=0.1)


def test_http_runner_client_invalid_action_schema_is_protocol_error() -> None:
    def transport(url: str, payload: str, timeout_seconds: float) -> str:
        return '{"wrong":"shape"}'

    client = HttpRunnerClient(endpoint_url="http://localhost:8000/act", transport=transport)
    with pytest.raises(HttpRunnerProtocolError, match="payload invalid"):
        client.request_action(_sample_observation(), timeout_seconds=0.1)


def test_http_runner_client_rejects_incompatible_observation_protocol_version() -> None:
    def transport(url: str, payload: str, timeout_seconds: float) -> str:
        return '{"action":"wait"}'

    client = HttpRunnerClient(endpoint_url="http://localhost:8000/act", transport=transport)
    observation = _sample_observation()
    object.__setattr__(observation, "protocol_version", "9.9")

    with pytest.raises(
        HttpRunnerProtocolError,
        match="incompatible_protocol_version:unsupported_protocol_version:9.9:supported=1.0",
    ):
        client.request_action(observation, timeout_seconds=0.1)


def test_http_runner_client_rejects_unsupported_action_envelope_protocol_version() -> None:
    def transport(url: str, payload: str, timeout_seconds: float) -> str:
        return '{"protocol_version":"9.9","action":"wait"}'

    client = HttpRunnerClient(endpoint_url="http://localhost:8000/act", transport=transport)
    with pytest.raises(
        HttpRunnerProtocolError,
        match="unsupported_protocol_version:9.9:supported=1.0",
    ):
        client.request_action(_sample_observation(), timeout_seconds=0.1)


def test_parse_action_payload_json_accepts_supported_protocol_version_when_present() -> None:
    parsed = parse_action_payload_json('{"protocol_version":"1.0","action":"wait"}')
    assert parsed.action == "wait"


def test_http_runner_client_non_2xx_status_is_protocol_error(monkeypatch: pytest.MonkeyPatch) -> None:
    def fake_urlopen(http_request, timeout):  # noqa: ANN001
        raise error.HTTPError(
            url=http_request.full_url,
            code=503,
            msg="service unavailable",
            hdrs=None,
            fp=None,
        )

    monkeypatch.setattr("agents.http_runner.http_client.request.urlopen", fake_urlopen)
    client = HttpRunnerClient(endpoint_url="http://localhost:8000/act")

    with pytest.raises(HttpRunnerProtocolError, match="503"):
        client.request_action(_sample_observation(), timeout_seconds=0.1)


def test_http_runner_client_urlerror_timeout_is_timeout_error(monkeypatch: pytest.MonkeyPatch) -> None:
    def fake_urlopen(http_request, timeout):  # noqa: ANN001
        raise error.URLError(socket.timeout("timed out"))

    monkeypatch.setattr("agents.http_runner.http_client.request.urlopen", fake_urlopen)
    client = HttpRunnerClient(endpoint_url="http://localhost:8000/act")

    with pytest.raises(HttpRunnerTimeoutError, match="timed out"):
        client.request_action(_sample_observation(), timeout_seconds=0.1)
