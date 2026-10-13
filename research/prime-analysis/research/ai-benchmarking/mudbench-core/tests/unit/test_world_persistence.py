"""Unit tests for world_persistence save/load roundtrip, error handling, and determinism."""

from __future__ import annotations

import json
from pathlib import Path

from world.state.state_types import WorldEntityState, WorldRoomState, WorldStateSnapshot
from world.state.world_persistence import (
    WORLD_SNAPSHOT_FORMAT,
    WORLD_SNAPSHOT_FORMAT_VERSION,
    WorldSnapshotSaveResult,
    load_world_snapshot,
    save_world_snapshot,
)
from world.state.world_state import DeterministicWorldStateManager


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _make_world_state(
    *,
    tick: int = 5,
    actor_id: str = "test-actor",
    location: str = "room-a",
    inventory: tuple[str, ...] = ("key", "torch"),
) -> DeterministicWorldStateManager:
    """Build a minimal deterministic world state for persistence tests."""
    snapshot = WorldStateSnapshot(
        tick=tick,
        entities=(
            WorldEntityState(
                entity_id=actor_id,
                entity_type="player",
                location=location,
                health=100,
                inventory=inventory,
                tags=(),
            ),
        ),
        rooms=(
            WorldRoomState(
                room_id=location,
                description="A plain test room",
                exits=(("north", "room-b"),),
                entities_present=(actor_id,),
            ),
        ),
        scenario_vars=(("quest_stage", "begin"), ("score", 0)),
    )
    return DeterministicWorldStateManager(initial_state=snapshot)


# ---------------------------------------------------------------------------
# Save tests
# ---------------------------------------------------------------------------

def test_save_returns_accepted(tmp_path: Path) -> None:
    ws = _make_world_state()
    dest = str(tmp_path / "snapshot.json")
    result = save_world_snapshot(dest, ws, run_id="run-1", scenario_id="sc-1")
    assert isinstance(result, WorldSnapshotSaveResult)
    assert result.accepted is True
    assert result.path == dest
    assert result.reason == ""


def test_save_creates_inspectable_json(tmp_path: Path) -> None:
    ws = _make_world_state()
    dest = tmp_path / "snapshot.json"
    save_world_snapshot(str(dest), ws, run_id="run-1", scenario_id="sc-1", actor_ids=["test-actor"])
    raw = dest.read_text(encoding="utf-8")
    parsed = json.loads(raw)
    assert parsed["format"] == WORLD_SNAPSHOT_FORMAT
    assert parsed["format_version"] == WORLD_SNAPSHOT_FORMAT_VERSION
    assert parsed["run_id"] == "run-1"
    assert parsed["scenario_id"] == "sc-1"
    assert parsed["actor_ids"] == ["test-actor"]
    assert "world_state" in parsed
    # keys must be sorted (inspectable)
    assert raw == json.dumps(parsed, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def test_save_creates_parent_directories(tmp_path: Path) -> None:
    ws = _make_world_state()
    dest = tmp_path / "nested" / "deep" / "snapshot.json"
    assert not dest.parent.exists()
    result = save_world_snapshot(str(dest), ws, run_id="r", scenario_id="s")
    assert result.accepted is True
    assert dest.exists()


def test_save_overwrites_existing_file(tmp_path: Path) -> None:
    dest = tmp_path / "snapshot.json"
    dest.write_text("old content", encoding="utf-8")
    ws = _make_world_state()
    result = save_world_snapshot(str(dest), ws, run_id="r", scenario_id="s")
    assert result.accepted is True
    raw = dest.read_text(encoding="utf-8")
    assert raw != "old content"
    parsed = json.loads(raw)
    assert parsed["format"] == WORLD_SNAPSHOT_FORMAT


def test_save_rejects_empty_run_id(tmp_path: Path) -> None:
    ws = _make_world_state()
    result = save_world_snapshot(str(tmp_path / "s.json"), ws, run_id="", scenario_id="sc-1")
    assert result.accepted is False
    assert "run_id" in result.reason


def test_save_rejects_empty_scenario_id(tmp_path: Path) -> None:
    ws = _make_world_state()
    result = save_world_snapshot(str(tmp_path / "s.json"), ws, run_id="r", scenario_id="")
    assert result.accepted is False
    assert "scenario_id" in result.reason


# ---------------------------------------------------------------------------
# Load tests
# ---------------------------------------------------------------------------

def test_load_roundtrip_preserves_world_state(tmp_path: Path) -> None:
    ws = _make_world_state(tick=7, actor_id="hero", location="room-z")
    dest = str(tmp_path / "snap.json")
    save_result = save_world_snapshot(
        dest, ws, run_id="r1", scenario_id="sc1", actor_ids=["hero"]
    )
    assert save_result.accepted

    load_result = load_world_snapshot(dest)
    assert load_result.accepted is True
    assert load_result.world_state is not None
    assert load_result.run_id == "r1"
    assert load_result.scenario_id == "sc1"
    assert load_result.actor_ids == ("hero",)
    assert load_result.world_tick == 7


def test_load_restores_actor_location_and_inventory(tmp_path: Path) -> None:
    ws = _make_world_state(actor_id="alice", location="forest", inventory=("sword", "potion"))
    dest = str(tmp_path / "snap.json")
    save_world_snapshot(dest, ws, run_id="r", scenario_id="s")

    loaded = load_world_snapshot(dest)
    assert loaded.accepted
    snap = loaded.world_state.get_snapshot()  # type: ignore[union-attr]
    # get_snapshot() returns to_dict() — a plain mapping
    entities = snap.get("entities", {})
    assert "alice" in entities
    assert entities["alice"]["location"] == "forest"
    assert set(entities["alice"].get("inventory", [])) == {"sword", "potion"}


def test_load_restores_room_state(tmp_path: Path) -> None:
    ws = _make_world_state(actor_id="p1", location="dungeon")
    dest = str(tmp_path / "snap.json")
    save_world_snapshot(dest, ws, run_id="r", scenario_id="s")

    loaded = load_world_snapshot(dest)
    assert loaded.accepted
    snap = loaded.world_state.get_snapshot()  # type: ignore[union-attr]
    rooms = snap.get("rooms", {})
    assert "dungeon" in rooms
    assert "p1" in rooms["dungeon"].get("entities_present", [])


def test_load_restores_scenario_vars(tmp_path: Path) -> None:
    ws = _make_world_state()
    dest = str(tmp_path / "snap.json")
    save_world_snapshot(dest, ws, run_id="r", scenario_id="s")

    loaded = load_world_snapshot(dest)
    assert loaded.accepted
    snap = loaded.world_state.get_snapshot()  # type: ignore[union-attr]
    var_map = snap.get("scenario_vars", {})
    assert var_map.get("quest_stage") == "begin"
    assert var_map.get("score") == 0


# ---------------------------------------------------------------------------
# Determinism test
# ---------------------------------------------------------------------------

def test_save_is_deterministic(tmp_path: Path) -> None:
    ws = _make_world_state()
    p1 = str(tmp_path / "a.json")
    p2 = str(tmp_path / "b.json")
    save_world_snapshot(p1, ws, run_id="r", scenario_id="s", actor_ids=["test-actor"])
    save_world_snapshot(p2, ws, run_id="r", scenario_id="s", actor_ids=["test-actor"])
    assert Path(p1).read_text(encoding="utf-8") == Path(p2).read_text(encoding="utf-8")


# ---------------------------------------------------------------------------
# Load rejection tests
# ---------------------------------------------------------------------------

def test_load_rejects_missing_file(tmp_path: Path) -> None:
    result = load_world_snapshot(str(tmp_path / "nonexistent.json"))
    assert result.accepted is False
    assert "not_found" in result.reason


def test_load_rejects_wrong_format(tmp_path: Path) -> None:
    dest = tmp_path / "bad.json"
    payload = {
        "format": "wrong_format",
        "format_version": WORLD_SNAPSHOT_FORMAT_VERSION,
        "run_id": "r",
        "scenario_id": "s",
        "world_tick": 0,
        "world_state": {},
    }
    dest.write_text(json.dumps(payload), encoding="utf-8")
    result = load_world_snapshot(str(dest))
    assert result.accepted is False
    assert "format_mismatch" in result.reason


def test_load_rejects_wrong_version(tmp_path: Path) -> None:
    dest = tmp_path / "bad_version.json"
    payload = {
        "format": WORLD_SNAPSHOT_FORMAT,
        "format_version": 999,
        "run_id": "r",
        "scenario_id": "s",
        "world_tick": 0,
        "world_state": {},
    }
    dest.write_text(json.dumps(payload), encoding="utf-8")
    result = load_world_snapshot(str(dest))
    assert result.accepted is False
    assert "version_mismatch" in result.reason


def test_load_rejects_missing_required_keys(tmp_path: Path) -> None:
    dest = tmp_path / "partial.json"
    dest.write_text(json.dumps({"format": WORLD_SNAPSHOT_FORMAT}), encoding="utf-8")
    result = load_world_snapshot(str(dest))
    assert result.accepted is False
    assert "missing_required_keys" in result.reason


def test_load_rejects_non_json_file(tmp_path: Path) -> None:
    dest = tmp_path / "binary.json"
    dest.write_text("NOT JSON {{{{", encoding="utf-8")
    result = load_world_snapshot(str(dest))
    assert result.accepted is False
    assert "read_error" in result.reason


def test_load_rejects_non_dict_payload(tmp_path: Path) -> None:
    dest = tmp_path / "list.json"
    dest.write_text(json.dumps([1, 2, 3]), encoding="utf-8")
    result = load_world_snapshot(str(dest))
    assert result.accepted is False


# ---------------------------------------------------------------------------
# Result type tests
# ---------------------------------------------------------------------------

def test_save_result_to_dict(tmp_path: Path) -> None:
    ws = _make_world_state()
    dest = str(tmp_path / "snap.json")
    result = save_world_snapshot(dest, ws, run_id="r", scenario_id="s")
    d = result.to_dict()
    assert d["accepted"] is True
    assert d["path"] == dest


def test_load_result_to_dict_on_failure(tmp_path: Path) -> None:
    result = load_world_snapshot(str(tmp_path / "nope.json"))
    d = result.to_dict()
    assert d["accepted"] is False
    assert isinstance(d["reason"], str)


# ---------------------------------------------------------------------------
# Named save slot tests
# ---------------------------------------------------------------------------


def test_validate_slot_name_accepts_valid() -> None:
    from world.state.world_persistence import validate_slot_name
    assert validate_slot_name("my-save") is None
    assert validate_slot_name("save1") is None
    assert validate_slot_name("A") is None
    assert validate_slot_name("save_slot_99") is None


def test_validate_slot_name_rejects_invalid() -> None:
    from world.state.world_persistence import validate_slot_name
    assert validate_slot_name("") is not None
    assert validate_slot_name("-starts-with-dash") is not None
    assert validate_slot_name("has space") is not None
    assert validate_slot_name("a" * 64) is not None  # too long (max 63)


def test_save_slot_creates_named_file(tmp_path: Path) -> None:
    from world.state.world_persistence import save_world_slot
    ws = _make_world_state()
    result = save_world_slot(
        "my-game", tmp_path, ws, run_id="r", scenario_id="sc1", scenario_version="1.0"
    )
    assert result.accepted is True
    assert result.slot_name == "my-game"
    assert (tmp_path / "my-game.json").exists()


def test_save_slot_rejects_invalid_slot_name(tmp_path: Path) -> None:
    from world.state.world_persistence import save_world_slot
    ws = _make_world_state()
    result = save_world_slot(
        "bad name!", tmp_path, ws, run_id="r", scenario_id="sc1"
    )
    assert result.accepted is False
    assert "slot_name" in result.reason.lower() or "alphanumeric" in result.reason.lower()


def test_load_slot_roundtrip(tmp_path: Path) -> None:
    from world.state.world_persistence import save_world_slot, load_world_slot
    ws = _make_world_state(tick=3)
    save_world_slot("my-save", tmp_path, ws, run_id="r1", scenario_id="sc1", scenario_version="1.0", actor_ids=["p1"])
    result = load_world_slot("my-save", tmp_path)
    assert result.accepted is True
    assert result.slot_name == "my-save"
    assert result.scenario_id == "sc1"
    assert result.world_tick == 3
    assert result.actor_ids == ("p1",)


def test_load_slot_missing_rejects(tmp_path: Path) -> None:
    from world.state.world_persistence import load_world_slot
    result = load_world_slot("no-slot", tmp_path)
    assert result.accepted is False
    assert "not_found" in result.reason


# ---------------------------------------------------------------------------
# Compatibility guard tests
# ---------------------------------------------------------------------------


def test_load_snapshot_guard_rejects_wrong_scenario_id(tmp_path: Path) -> None:
    ws = _make_world_state()
    dest = str(tmp_path / "snap.json")
    save_world_snapshot(dest, ws, run_id="r", scenario_id="sc-A")

    result = load_world_snapshot(dest, required_scenario_id="sc-B")
    assert result.accepted is False
    assert "scenario_id_mismatch" in result.reason
    assert "sc-A" in result.reason
    assert "sc-B" in result.reason


def test_load_snapshot_guard_accepts_matching_scenario_id(tmp_path: Path) -> None:
    ws = _make_world_state()
    dest = str(tmp_path / "snap.json")
    save_world_snapshot(dest, ws, run_id="r", scenario_id="sc-A", scenario_version="1.0")

    result = load_world_snapshot(dest, required_scenario_id="sc-A")
    assert result.accepted is True


def test_load_snapshot_guard_rejects_wrong_version(tmp_path: Path) -> None:
    ws = _make_world_state()
    dest = str(tmp_path / "snap.json")
    save_world_snapshot(dest, ws, run_id="r", scenario_id="sc-A", scenario_version="1.0")

    result = load_world_snapshot(dest, required_scenario_id="sc-A", required_scenario_version="2.0")
    assert result.accepted is False
    assert "scenario_version_mismatch" in result.reason


def test_load_slot_guard_rejects_wrong_scenario(tmp_path: Path) -> None:
    from world.state.world_persistence import save_world_slot, load_world_slot
    ws = _make_world_state()
    save_world_slot("g1", tmp_path, ws, run_id="r", scenario_id="tiny-fetch-quest", scenario_version="1.0")

    result = load_world_slot("g1", tmp_path, required_scenario_id="tiny-context-pressure")
    assert result.accepted is False
    assert "scenario_id_mismatch" in result.reason


# ---------------------------------------------------------------------------
# Session identity tests
# ---------------------------------------------------------------------------


def test_save_persists_session_id_and_character_names(tmp_path: Path) -> None:
    ws = _make_world_state()
    dest = str(tmp_path / "snap.json")
    save_world_snapshot(
        dest, ws,
        run_id="r1",
        scenario_id="sc1",
        session_id="shard-abc",
        character_names={"player": "Aria the Bold"},
    )
    result = load_world_snapshot(dest)
    assert result.accepted is True
    assert result.session_id == "shard-abc"
    assert dict(result.character_names) == {"player": "Aria the Bold"}


def test_save_slot_preserves_session_identity(tmp_path: Path) -> None:
    from world.state.world_persistence import save_world_slot, load_world_slot
    ws = _make_world_state()
    save_world_slot(
        "hero",
        tmp_path,
        ws,
        run_id="r",
        scenario_id="sc1",
        session_id="shard-xyz",
        character_names={"p1": "Lord of Shadows"},
    )
    result = load_world_slot("hero", tmp_path)
    assert result.accepted is True
    assert result.session_id == "shard-xyz"
    assert dict(result.character_names) == {"p1": "Lord of Shadows"}


# ---------------------------------------------------------------------------
# list_world_slots tests
# ---------------------------------------------------------------------------


def test_list_world_slots_empty_dir(tmp_path: Path) -> None:
    from world.state.world_persistence import list_world_slots
    result = list_world_slots(tmp_path)
    assert result.accepted is True
    assert result.slots == ()


def test_list_world_slots_nonexistent_dir(tmp_path: Path) -> None:
    from world.state.world_persistence import list_world_slots
    result = list_world_slots(tmp_path / "does-not-exist")
    assert result.accepted is True
    assert result.slots == ()


def test_list_world_slots_shows_saved_slots(tmp_path: Path) -> None:
    from world.state.world_persistence import save_world_slot, list_world_slots
    ws = _make_world_state(tick=7)
    save_world_slot("alpha", tmp_path, ws, run_id="r1", scenario_id="sc1", scenario_version="1.0")
    save_world_slot("beta", tmp_path, ws, run_id="r2", scenario_id="sc2", scenario_version="2.0")

    result = list_world_slots(tmp_path)
    assert result.accepted is True
    assert len(result.slots) == 2
    names = [s.slot_name for s in result.slots]
    assert "alpha" in names
    assert "beta" in names


def test_list_world_slots_ordering_is_deterministic(tmp_path: Path) -> None:
    from world.state.world_persistence import save_world_slot, list_world_slots
    ws = _make_world_state()
    save_world_slot("zebra", tmp_path, ws, run_id="r", scenario_id="sc1")
    save_world_slot("apple", tmp_path, ws, run_id="r", scenario_id="sc1")
    save_world_slot("mango", tmp_path, ws, run_id="r", scenario_id="sc1")

    result1 = list_world_slots(tmp_path)
    result2 = list_world_slots(tmp_path)
    assert [s.slot_name for s in result1.slots] == [s.slot_name for s in result2.slots]
    # Should be alphabetical
    assert [s.slot_name for s in result1.slots] == ["apple", "mango", "zebra"]


def test_list_world_slots_to_dict(tmp_path: Path) -> None:
    from world.state.world_persistence import save_world_slot, list_world_slots
    ws = _make_world_state()
    save_world_slot("s1", tmp_path, ws, run_id="r", scenario_id="sc1", scenario_version="1.0")

    result = list_world_slots(tmp_path)
    d = result.to_dict()
    assert d["accepted"] is True
    assert len(d["slots"]) == 1
    assert d["slots"][0]["slot_name"] == "s1"


# ---------------------------------------------------------------------------
# Multi-actor continuity tests
# ---------------------------------------------------------------------------


def _make_multi_actor_world_state(*, tick: int = 10) -> DeterministicWorldStateManager:
    """Build a deterministic world state with two actors at distinct locations."""
    snapshot = WorldStateSnapshot(
        tick=tick,
        entities=(
            WorldEntityState(
                entity_id="alice",
                entity_type="player",
                location="room-a",
                health=100,
                inventory=("sword", "shield"),
                tags=(),
            ),
            WorldEntityState(
                entity_id="bob",
                entity_type="player",
                location="room-b",
                health=80,
                inventory=("torch",),
                tags=(),
            ),
        ),
        rooms=(
            WorldRoomState(
                room_id="room-a",
                description="Room A",
                exits=(("east", "room-b"),),
                entities_present=("alice",),
            ),
            WorldRoomState(
                room_id="room-b",
                description="Room B",
                exits=(("west", "room-a"),),
                entities_present=("bob",),
            ),
        ),
        scenario_vars=(("quest_stage", "mid"), ("score", 5)),
    )
    return DeterministicWorldStateManager(initial_state=snapshot)


def test_multi_actor_save_load_preserves_all_actor_states(tmp_path: Path) -> None:
    """Both actors' locations and inventories survive a save/load roundtrip without cross-contamination."""
    ws = _make_multi_actor_world_state(tick=10)
    dest = str(tmp_path / "multi.json")
    save_world_snapshot(
        dest, ws,
        run_id="r",
        scenario_id="sc1",
        actor_ids=["alice", "bob"],
    )

    loaded = load_world_snapshot(dest)
    assert loaded.accepted
    snap = loaded.world_state.get_snapshot()  # type: ignore[union-attr]
    entities = snap["entities"]

    assert entities["alice"]["location"] == "room-a"
    assert entities["bob"]["location"] == "room-b"
    # inventory is sorted + deduplicated by the data model
    assert set(entities["alice"]["inventory"]) == {"sword", "shield"}
    assert set(entities["bob"]["inventory"]) == {"torch"}
    # No cross-contamination: alice is not in room-b, bob is not in room-a
    assert entities["alice"]["location"] != entities["bob"]["location"]


def test_multi_actor_actor_ids_metadata_captures_all_actors(tmp_path: Path) -> None:
    """actor_ids snapshot metadata includes every actor passed in."""
    ws = _make_multi_actor_world_state()
    dest = str(tmp_path / "meta.json")
    save_world_snapshot(
        dest, ws,
        run_id="r",
        scenario_id="sc1",
        actor_ids=["alice", "bob"],
    )

    loaded = load_world_snapshot(dest)
    assert loaded.accepted
    assert set(loaded.actor_ids) == {"alice", "bob"}


def test_multi_actor_character_names_all_preserved(tmp_path: Path) -> None:
    """Character names for all actors survive save/load roundtrip."""
    ws = _make_multi_actor_world_state()
    dest = str(tmp_path / "names.json")
    save_world_snapshot(
        dest, ws,
        run_id="r",
        scenario_id="sc1",
        actor_ids=["alice", "bob"],
        character_names={"alice": "Alice the Brave", "bob": "Bob the Torch-Bearer"},
    )

    loaded = load_world_snapshot(dest)
    assert loaded.accepted
    names = dict(loaded.character_names)
    assert names["alice"] == "Alice the Brave"
    assert names["bob"] == "Bob the Torch-Bearer"


def test_multi_actor_room_entities_present_correctly_assigned_after_reload(tmp_path: Path) -> None:
    """Each room's entities_present listing is correct for its occupying actor after reload."""
    ws = _make_multi_actor_world_state()
    dest = str(tmp_path / "rooms.json")
    save_world_snapshot(dest, ws, run_id="r", scenario_id="sc1")

    loaded = load_world_snapshot(dest)
    assert loaded.accepted
    snap = loaded.world_state.get_snapshot()  # type: ignore[union-attr]
    rooms = snap["rooms"]

    assert "alice" in rooms["room-a"]["entities_present"]
    assert "bob" not in rooms["room-a"]["entities_present"]
    assert "bob" in rooms["room-b"]["entities_present"]
    assert "alice" not in rooms["room-b"]["entities_present"]


def test_multi_actor_no_duplication_on_repeated_save_load_cycle(tmp_path: Path) -> None:
    """Entity count is stable across a save → load → save → load chain: no actor duplication."""
    ws = _make_multi_actor_world_state()
    dest1 = str(tmp_path / "cycle1.json")
    dest2 = str(tmp_path / "cycle2.json")

    save_world_snapshot(dest1, ws, run_id="r", scenario_id="sc1")
    r1 = load_world_snapshot(dest1)
    assert r1.accepted
    entity_count_original = len(ws.get_snapshot()["entities"])
    entity_count_after_first_load = len(r1.world_state.get_snapshot()["entities"])  # type: ignore[union-attr]
    assert entity_count_after_first_load == entity_count_original

    # Second cycle: save the loaded state, then reload again
    save_world_snapshot(dest2, r1.world_state, run_id="r", scenario_id="sc1")  # type: ignore[arg-type]
    r2 = load_world_snapshot(dest2)
    assert r2.accepted
    entity_count_after_second_load = len(r2.world_state.get_snapshot()["entities"])  # type: ignore[union-attr]
    assert entity_count_after_second_load == entity_count_original


def test_multi_actor_slot_roundtrip_preserves_all_actors(tmp_path: Path) -> None:
    """Named slot save/load roundtrip with two actors preserves both actors with correct state."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    ws = _make_multi_actor_world_state(tick=15)
    result = save_world_slot(
        "two-actors",
        tmp_path,
        ws,
        run_id="r1",
        scenario_id="sc1",
        actor_ids=["alice", "bob"],
        character_names={"alice": "Lady Alice", "bob": "Sir Bob"},
    )
    assert result.accepted

    loaded = load_world_slot("two-actors", tmp_path)
    assert loaded.accepted
    assert loaded.world_tick == 15
    assert set(loaded.actor_ids) == {"alice", "bob"}
    assert dict(loaded.character_names) == {"alice": "Lady Alice", "bob": "Sir Bob"}

    snap = loaded.world_state.get_snapshot()  # type: ignore[union-attr]
    assert snap["entities"]["alice"]["location"] == "room-a"
    assert snap["entities"]["bob"]["location"] == "room-b"
