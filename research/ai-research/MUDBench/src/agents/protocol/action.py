"""Action submission model for MUDBench agents."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Mapping

from agents.protocol.observation import require_supported_protocol_version


@dataclass(frozen=True, slots=True)
class ActionSubmission:
    """Canonical action payload sent by an agent."""

    action: str

    def __post_init__(self) -> None:
        normalized = self.action.strip()
        if not normalized:
            raise ValueError("ActionSubmission requires a non-empty 'action'")
        object.__setattr__(self, "action", normalized)

    @classmethod
    def from_dict(cls, payload: Mapping[str, Any]) -> "ActionSubmission":
        raw_action = payload.get("action")
        if not isinstance(raw_action, str):
            raise ValueError("ActionSubmission payload requires string field 'action'")
        return cls(action=raw_action)

    def to_dict(self) -> dict[str, str]:
        return {"action": self.action}


def parse_action_submission_payload(payload: Mapping[str, Any]) -> ActionSubmission:
    """Parse action payload and enforce protocol version when provided."""
    if "protocol_version" in payload:
        require_supported_protocol_version(payload.get("protocol_version"))
    return ActionSubmission.from_dict(payload)
