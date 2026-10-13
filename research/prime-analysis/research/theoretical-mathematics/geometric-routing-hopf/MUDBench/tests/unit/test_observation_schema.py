from __future__ import annotations

import pytest

from agents.protocol.observation import Observation, SUPPORTED_PROTOCOL_VERSIONS


def _sample_observation_payload() -> dict[str, object]:
    return {
        "run_id": "abc123",
        "step": 42,
        "location": "forest_path",
        "description": "You stand on a narrow forest trail.",
        "exits": ["north", "south"],
        "entities": [{"type": "npc", "name": "goblin"}],
        "inventory": ["coin"],
        "health": 87,
        "messages": ["The goblin snarls."],
        "action_space": ["move north", "move south"],
        "remaining_steps": 958,
    }


def test_observation_from_dict_has_expected_shape() -> None:
    observation = Observation.from_dict(_sample_observation_payload())

    assert observation.run_id == "abc123"
    assert observation.step == 42
    assert observation.action_space == ("move north", "move south")
    assert observation.protocol_version == "1.0"


def test_observation_missing_required_fields_is_rejected() -> None:
    payload = _sample_observation_payload()
    payload.pop("run_id")

    with pytest.raises(ValueError, match="missing required fields"):
        Observation.from_dict(payload)


def test_observation_roundtrip_to_dict_preserves_protocol_version() -> None:
    payload = _sample_observation_payload()
    payload["protocol_version"] = "1.0"

    observation = Observation.from_dict(payload)

    assert observation.to_dict()["protocol_version"] == "1.0"


def test_observation_rejects_unsupported_protocol_version_with_machine_readable_reason() -> None:
    payload = _sample_observation_payload()
    payload["protocol_version"] = "9.9"

    with pytest.raises(
        ValueError,
        match=(
            "unsupported_protocol_version:9.9:supported="
            + ",".join(SUPPORTED_PROTOCOL_VERSIONS)
        ),
    ):
        Observation.from_dict(payload)
