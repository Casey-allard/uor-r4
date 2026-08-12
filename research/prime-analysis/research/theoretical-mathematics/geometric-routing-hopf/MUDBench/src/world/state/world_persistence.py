"""Deterministic world-state persistence and cross-session continuity for MUDBench.

Provides explicit save/load of a full world snapshot to/from a local JSON file,
and a named save-slot layer so sessions can reconnect to the same persistent world
without requiring raw file paths.

Design notes:
- Format is inspectable plain JSON (sorted keys, no binary blobs).
- Loading is always explicit — no hidden auto-restore behavior.
- The snapshot captures the full DeterministicWorldStateManager state plus
  metadata (scenario_id, scenario_version, session_id, character_names).
- Named save slots live as <slot_name>.json inside a configurable save directory.
- A compatibility guard is available: load rejects if saved scenario_id/version
  does not match the required values, preventing silent world/scenario mismatches.
"""

from __future__ import annotations

import json
import re
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Any, Mapping, Sequence

from .world_state import DeterministicWorldStateManager

WORLD_SNAPSHOT_FORMAT: str = "mudbench_world_snapshot"
WORLD_SNAPSHOT_FORMAT_VERSION: int = 1
WORLD_SAVE_DIR_DEFAULT: str = "~/.mudbench/saves"

# Slot names: start with alphanumeric, then alphanumeric/hyphen/underscore, max 63 chars total.
_SLOT_NAME_PATTERN: re.Pattern[str] = re.compile(r"^[a-zA-Z0-9][a-zA-Z0-9_-]{0,62}$")

_REQUIRED_FORMAT_KEYS = frozenset(
    {"format", "format_version", "run_id", "scenario_id", "world_tick", "world_state"}
)


@dataclass(frozen=True, slots=True)
class WorldSnapshotSaveResult:
    """Result of a save_world_snapshot or save_world_slot call."""

    accepted: bool
    path: str
    slot_name: str = ""
    reason: str = ""

    def to_dict(self) -> dict[str, Any]:
        return {
            "accepted": self.accepted,
            "path": self.path,
            "slot_name": self.slot_name,
            "reason": self.reason,
        }


@dataclass(frozen=True, slots=True)
class WorldSnapshotLoadResult:
    """Result of a load_world_snapshot or load_world_slot call."""

    accepted: bool
    world_state: DeterministicWorldStateManager | None
    run_id: str
    scenario_id: str
    scenario_version: str
    actor_ids: tuple[str, ...]
    world_tick: int
    session_id: str = ""
    character_names: tuple[tuple[str, str], ...] = ()
    slot_name: str = ""
    reason: str = ""

    def to_dict(self) -> dict[str, Any]:
        return {
            "accepted": self.accepted,
            "run_id": self.run_id,
            "scenario_id": self.scenario_id,
            "scenario_version": self.scenario_version,
            "actor_ids": list(self.actor_ids),
            "world_tick": self.world_tick,
            "session_id": self.session_id,
            "character_names": {k: v for k, v in self.character_names},
            "slot_name": self.slot_name,
            "reason": self.reason,
        }


@dataclass(frozen=True, slots=True)
class WorldSlotInfo:
    """Metadata summary for a single named save slot."""

    slot_name: str
    scenario_id: str
    scenario_version: str
    world_tick: int
    run_id: str
    session_id: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "slot_name": self.slot_name,
            "scenario_id": self.scenario_id,
            "scenario_version": self.scenario_version,
            "world_tick": self.world_tick,
            "run_id": self.run_id,
            "session_id": self.session_id,
        }


@dataclass(frozen=True, slots=True)
class WorldSlotListResult:
    """Result of a list_world_slots call."""

    accepted: bool
    save_dir: str
    slots: tuple[WorldSlotInfo, ...]
    reason: str = ""

    def to_dict(self) -> dict[str, Any]:
        return {
            "accepted": self.accepted,
            "save_dir": self.save_dir,
            "slots": [s.to_dict() for s in self.slots],
            "reason": self.reason,
        }


def validate_slot_name(slot_name: str) -> str | None:
    """Return None if slot_name is valid; return an error reason string otherwise."""
    if not isinstance(slot_name, str) or not slot_name:
        return "slot_name must be a non-empty string"
    if not _SLOT_NAME_PATTERN.match(slot_name):
        return (
            f"slot_name must start with alphanumeric and contain only "
            f"alphanumeric/hyphen/underscore (max 63 chars): got {slot_name!r}"
        )
    return None


def save_world_snapshot(
    path: Any,
    world_state_manager: DeterministicWorldStateManager,
    *,
    run_id: str,
    scenario_id: str,
    scenario_version: str = "",
    session_id: str = "",
    character_names: Mapping[str, str] | None = None,
    actor_ids: Sequence[str] = (),
    slot_name: str = "",
) -> WorldSnapshotSaveResult:
    """Serialize the current world state to a local JSON snapshot file.

    Creates parent directories as needed.  Overwrites any existing file.
    All fields are sorted for deterministic, inspectable output.

    Args:
        path: Destination file path (str or Path).
        world_state_manager: The live world state to capture.
        run_id: Run identifier written into the snapshot.
        scenario_id: Scenario identifier written into the snapshot.
        scenario_version: Scenario version string for compatibility guard (optional).
        session_id: Session/shard identifier for continuity (optional).
        character_names: Mapping of actor_id → display name (optional).
        actor_ids: Actor identifiers present at save time.
        slot_name: Named save slot identifier if saved via slot (optional).

    Returns:
        WorldSnapshotSaveResult with accepted=True on success.
    """
    if not isinstance(run_id, str) or not run_id:
        return WorldSnapshotSaveResult(
            accepted=False, path=str(path), reason="run_id must be a non-empty string"
        )
    if not isinstance(scenario_id, str) or not scenario_id:
        return WorldSnapshotSaveResult(
            accepted=False, path=str(path), reason="scenario_id must be a non-empty string"
        )

    try:
        resolved_path = Path(path)
    except TypeError:
        return WorldSnapshotSaveResult(
            accepted=False, path=str(path), reason="path must be a valid file path"
        )

    world_dict = world_state_manager.to_dict()
    world_tick = int(world_dict.get("tick", 0))

    payload: dict[str, Any] = {
        "format": WORLD_SNAPSHOT_FORMAT,
        "format_version": WORLD_SNAPSHOT_FORMAT_VERSION,
        "run_id": run_id,
        "scenario_id": scenario_id,
        "scenario_version": str(scenario_version),
        "session_id": str(session_id),
        "character_names": dict(character_names) if character_names else {},
        "actor_ids": sorted(str(a) for a in actor_ids),
        "world_tick": world_tick,
        "world_state": world_dict,
    }
    if slot_name:
        payload["slot_name"] = str(slot_name)

    try:
        resolved_path.parent.mkdir(parents=True, exist_ok=True)
        resolved_path.write_text(
            json.dumps(payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True),
            encoding="utf-8",
        )
    except OSError as exc:
        return WorldSnapshotSaveResult(
            accepted=False, path=str(resolved_path), reason=f"write_error:{exc}"
        )

    return WorldSnapshotSaveResult(
        accepted=True, path=str(resolved_path), slot_name=slot_name
    )


def load_world_snapshot(
    path: Any,
    *,
    required_scenario_id: str | None = None,
    required_scenario_version: str | None = None,
) -> WorldSnapshotLoadResult:
    """Load a world snapshot from a local JSON file and reconstruct the world state manager.

    Returns WorldSnapshotLoadResult with accepted=False if the file is missing,
    unreadable, structurally invalid, or fails the optional compatibility guard.
    Does not raise on expected error conditions.

    Args:
        path: Source file path (str or Path).
        required_scenario_id: If set, load is rejected when saved scenario_id differs.
        required_scenario_version: If set, load is rejected when saved scenario_version differs.

    Returns:
        WorldSnapshotLoadResult — accepted=True and world_state populated on success.
    """
    _empty = WorldSnapshotLoadResult(
        accepted=False,
        world_state=None,
        run_id="",
        scenario_id="",
        scenario_version="",
        actor_ids=(),
        world_tick=0,
    )

    try:
        resolved_path = Path(path)
    except TypeError:
        return replace(_empty, reason="path must be a valid file path")

    if not resolved_path.exists():
        return replace(_empty, reason=f"snapshot_file_not_found:{resolved_path}")

    try:
        raw = resolved_path.read_text(encoding="utf-8")
        payload = json.loads(raw)
    except (OSError, json.JSONDecodeError) as exc:
        return replace(_empty, reason=f"snapshot_read_error:{exc}")

    if not isinstance(payload, dict):
        return replace(_empty, reason="snapshot_payload_must_be_an_object")

    missing = _REQUIRED_FORMAT_KEYS - set(payload.keys())
    if missing:
        return replace(_empty, reason=f"snapshot_missing_required_keys:{sorted(missing)}")

    if payload.get("format") != WORLD_SNAPSHOT_FORMAT:
        return replace(
            _empty,
            reason=f"snapshot_format_mismatch:expected={WORLD_SNAPSHOT_FORMAT}",
        )

    if payload.get("format_version") != WORLD_SNAPSHOT_FORMAT_VERSION:
        return replace(
            _empty,
            reason=(
                f"snapshot_version_mismatch:"
                f"expected={WORLD_SNAPSHOT_FORMAT_VERSION},"
                f"got={payload.get('format_version')}"
            ),
        )

    run_id = str(payload.get("run_id", ""))
    scenario_id = str(payload.get("scenario_id", ""))
    scenario_version = str(payload.get("scenario_version", ""))
    session_id = str(payload.get("session_id", ""))
    slot_name = str(payload.get("slot_name", ""))

    # Compatibility guard
    if required_scenario_id is not None and scenario_id != required_scenario_id:
        return replace(
            _empty,
            reason=(
                f"snapshot_scenario_id_mismatch:"
                f"required={required_scenario_id},"
                f"saved={scenario_id}"
            ),
        )
    if (
        required_scenario_version is not None
        and required_scenario_version
        and scenario_version
        and scenario_version != required_scenario_version
    ):
        return replace(
            _empty,
            reason=(
                f"snapshot_scenario_version_mismatch:"
                f"required={required_scenario_version},"
                f"saved={scenario_version}"
            ),
        )

    raw_actor_ids = payload.get("actor_ids", [])
    actor_ids: tuple[str, ...]
    if isinstance(raw_actor_ids, list):
        actor_ids = tuple(str(a) for a in raw_actor_ids)
    else:
        actor_ids = ()

    raw_tick = payload.get("world_tick", 0)
    world_tick = int(raw_tick) if isinstance(raw_tick, int) else 0

    raw_character_names = payload.get("character_names", {})
    character_names: tuple[tuple[str, str], ...]
    if isinstance(raw_character_names, dict):
        character_names = tuple(
            (str(k), str(v))
            for k, v in sorted(raw_character_names.items())
        )
    else:
        character_names = ()

    raw_world_state = payload.get("world_state")
    if not isinstance(raw_world_state, dict):
        return replace(_empty, reason="snapshot_world_state_must_be_an_object")

    try:
        world_state_manager = DeterministicWorldStateManager.from_dict(raw_world_state)
    except (ValueError, KeyError, TypeError) as exc:
        return replace(_empty, reason=f"snapshot_world_state_reconstruction_error:{exc}")

    return WorldSnapshotLoadResult(
        accepted=True,
        world_state=world_state_manager,
        run_id=run_id,
        scenario_id=scenario_id,
        scenario_version=scenario_version,
        actor_ids=actor_ids,
        world_tick=world_tick,
        session_id=session_id,
        character_names=character_names,
        slot_name=slot_name,
    )


# ---------------------------------------------------------------------------
# Named save-slot API
# ---------------------------------------------------------------------------


def _resolve_save_dir(save_dir: Any) -> Path | None:
    """Resolve and expand a save directory path. Returns None on error."""
    try:
        return Path(str(save_dir)).expanduser()
    except (TypeError, ValueError):
        return None


def save_world_slot(
    slot_name: str,
    save_dir: Any,
    world_state_manager: DeterministicWorldStateManager,
    *,
    run_id: str,
    scenario_id: str,
    scenario_version: str = "",
    session_id: str = "",
    character_names: Mapping[str, str] | None = None,
    actor_ids: Sequence[str] = (),
) -> WorldSnapshotSaveResult:
    """Save a world snapshot to a named slot in the save directory.

    The slot is stored as <save_dir>/<slot_name>.json.

    Args:
        slot_name: Alphanumeric identifier for the save slot.
        save_dir: Directory where save slots are stored.
        world_state_manager: The live world state to capture.
        run_id: Run identifier for the snapshot.
        scenario_id: Scenario identifier for compatibility guard.
        scenario_version: Scenario version for compatibility guard (optional).
        session_id: Session/shard identifier for continuity (optional).
        character_names: Mapping of actor_id → display name (optional).
        actor_ids: Actor identifiers present at save time.

    Returns:
        WorldSnapshotSaveResult with accepted=True on success.
    """
    err = validate_slot_name(slot_name)
    if err:
        return WorldSnapshotSaveResult(accepted=False, path="", reason=err)

    resolved_dir = _resolve_save_dir(save_dir)
    if resolved_dir is None:
        return WorldSnapshotSaveResult(
            accepted=False, path="", reason="save_dir must be a valid directory path"
        )

    slot_path = resolved_dir / f"{slot_name}.json"
    return save_world_snapshot(
        slot_path,
        world_state_manager,
        run_id=run_id,
        scenario_id=scenario_id,
        scenario_version=scenario_version,
        session_id=session_id,
        character_names=character_names,
        actor_ids=actor_ids,
        slot_name=slot_name,
    )


def load_world_slot(
    slot_name: str,
    save_dir: Any,
    *,
    required_scenario_id: str | None = None,
    required_scenario_version: str | None = None,
) -> WorldSnapshotLoadResult:
    """Load a world snapshot from a named slot in the save directory.

    Args:
        slot_name: Alphanumeric identifier for the save slot.
        save_dir: Directory where save slots are stored.
        required_scenario_id: If set, load is rejected when saved scenario_id differs.
        required_scenario_version: If set, load is rejected when saved version differs.

    Returns:
        WorldSnapshotLoadResult — accepted=True and world_state populated on success.
    """
    _empty = WorldSnapshotLoadResult(
        accepted=False,
        world_state=None,
        run_id="",
        scenario_id="",
        scenario_version="",
        actor_ids=(),
        world_tick=0,
        slot_name=slot_name,
    )

    err = validate_slot_name(slot_name)
    if err:
        return replace(_empty, reason=err)

    resolved_dir = _resolve_save_dir(save_dir)
    if resolved_dir is None:
        return replace(_empty, reason="save_dir must be a valid directory path")

    slot_path = resolved_dir / f"{slot_name}.json"
    result = load_world_snapshot(
        slot_path,
        required_scenario_id=required_scenario_id,
        required_scenario_version=required_scenario_version,
    )
    if not result.accepted:
        return replace(result, slot_name=slot_name)
    return replace(result, slot_name=slot_name)


def list_world_slots(save_dir: Any) -> WorldSlotListResult:
    """List all named save slots in the save directory.

    Reads metadata from each .json file that matches the snapshot format.
    Files that cannot be parsed are silently skipped.

    Args:
        save_dir: Directory where save slots are stored.

    Returns:
        WorldSlotListResult with accepted=True and slots populated on success.
    """
    resolved_dir = _resolve_save_dir(save_dir)
    if resolved_dir is None:
        return WorldSlotListResult(
            accepted=False,
            save_dir=str(save_dir),
            slots=(),
            reason="save_dir must be a valid directory path",
        )

    if not resolved_dir.exists():
        return WorldSlotListResult(
            accepted=True,
            save_dir=str(resolved_dir),
            slots=(),
        )

    slots: list[WorldSlotInfo] = []
    for candidate in sorted(resolved_dir.glob("*.json")):
        # Slot name = filename without .json suffix; must be valid
        candidate_slot_name = candidate.stem
        if validate_slot_name(candidate_slot_name) is not None:
            continue
        try:
            raw = candidate.read_text(encoding="utf-8")
            payload = json.loads(raw)
        except (OSError, json.JSONDecodeError):
            continue
        if not isinstance(payload, dict):
            continue
        if payload.get("format") != WORLD_SNAPSHOT_FORMAT:
            continue
        slots.append(
            WorldSlotInfo(
                slot_name=candidate_slot_name,
                scenario_id=str(payload.get("scenario_id", "")),
                scenario_version=str(payload.get("scenario_version", "")),
                world_tick=int(payload.get("world_tick", 0)),
                run_id=str(payload.get("run_id", "")),
                session_id=str(payload.get("session_id", "")),
            )
        )

    return WorldSlotListResult(
        accepted=True,
        save_dir=str(resolved_dir),
        slots=tuple(slots),
    )

