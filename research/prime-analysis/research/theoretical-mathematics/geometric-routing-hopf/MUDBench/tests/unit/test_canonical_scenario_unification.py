"""Tests for canonical-scenario-source-of-truth unification.

Verifies that _SCENARIO_PRESETS is loaded from scenarios/canonical/*.json and
that no manual double-maintenance is required for tiny-* scenarios.
"""
from __future__ import annotations

import json
from pathlib import Path

import pytest

from cli.main import _CANONICAL_SCENARIO_DIR, _SCENARIO_PRESETS, _load_canonical_scenarios


# ---------------------------------------------------------------------------
# Fixtures / helpers
# ---------------------------------------------------------------------------

_CANONICAL_DIR = Path(__file__).resolve().parent.parent.parent / "scenarios" / "canonical"
_TINY_SCENARIO_IDS = sorted(
    p.stem.replace("_", "-")
    for p in _CANONICAL_DIR.glob("*.json")
    if p.stem.startswith("tiny_") or p.stem.startswith("tiny-")
)


# ---------------------------------------------------------------------------
# _CANONICAL_SCENARIO_DIR points to the right place
# ---------------------------------------------------------------------------


def test_canonical_scenario_dir_resolves() -> None:
    assert _CANONICAL_SCENARIO_DIR.is_dir(), (
        f"_CANONICAL_SCENARIO_DIR does not exist: {_CANONICAL_SCENARIO_DIR}"
    )


# ---------------------------------------------------------------------------
# _load_canonical_scenarios() returns all tiny-* scenarios
# ---------------------------------------------------------------------------


def test_load_canonical_scenarios_returns_all_canonical_files() -> None:
    loaded = _load_canonical_scenarios()
    canonical_files = sorted(p.stem for p in _CANONICAL_DIR.glob("*.json"))
    # Every canonical file must produce a key in the loaded dict
    for stem in canonical_files:
        scenario_id = stem.replace("_", "-")
        assert scenario_id in loaded, f"Expected {scenario_id!r} in loaded canonical scenarios"


def test_load_canonical_scenarios_each_entry_has_required_fields() -> None:
    required = {"scenario_id", "title", "start_room_id", "max_steps", "objectives"}
    loaded = _load_canonical_scenarios()
    for sid, payload in loaded.items():
        missing = required - payload.keys()
        assert not missing, f"Scenario {sid!r} missing fields: {missing}"


def test_load_canonical_scenarios_world_config_json_is_valid_json() -> None:
    loaded = _load_canonical_scenarios()
    for sid, payload in loaded.items():
        wcj = payload.get("scenario_vars", {}).get("world_config_json")
        if wcj is not None:
            try:
                wc = json.loads(wcj)
            except json.JSONDecodeError as exc:
                pytest.fail(f"Scenario {sid!r} world_config_json is invalid JSON: {exc}")
            assert isinstance(wc, dict), f"Scenario {sid!r} world_config_json must be a dict"


def test_load_canonical_scenarios_scenario_id_matches_key() -> None:
    loaded = _load_canonical_scenarios()
    for key, payload in loaded.items():
        assert payload["scenario_id"] == key, (
            f"Dict key {key!r} does not match scenario_id {payload['scenario_id']!r}"
        )


# ---------------------------------------------------------------------------
# _SCENARIO_PRESETS includes all canonical tiny-* scenarios
# ---------------------------------------------------------------------------


def test_scenario_presets_includes_all_canonical_tiny_scenarios() -> None:
    for sid in _TINY_SCENARIO_IDS:
        assert sid in _SCENARIO_PRESETS, (
            f"_SCENARIO_PRESETS missing {sid!r} — canonical file exists but preset was not loaded"
        )


def test_scenario_presets_canonical_content_matches_json_file() -> None:
    """Key invariant: preset content equals what is on disk — no manual sync needed."""
    for path in sorted(_CANONICAL_DIR.glob("*.json")):
        payload_on_disk = json.loads(path.read_text(encoding="utf-8"))
        sid = payload_on_disk.get("scenario_id")
        if not sid:
            continue
        assert sid in _SCENARIO_PRESETS, f"{sid!r} not in _SCENARIO_PRESETS"
        preset = _SCENARIO_PRESETS[sid]
        assert preset == payload_on_disk, (
            f"_SCENARIO_PRESETS[{sid!r}] content differs from {path.name}. "
            "Do not manually sync; edit the canonical JSON file instead."
        )


# ---------------------------------------------------------------------------
# No double-maintenance: changing a canonical file is reflected without code change
# ---------------------------------------------------------------------------


def test_load_canonical_scenarios_is_deterministic() -> None:
    """Repeated calls must return identical dicts (no random ordering)."""
    first = _load_canonical_scenarios()
    second = _load_canonical_scenarios()
    assert first == second
    assert list(first.keys()) == list(second.keys()), "Key ordering must be deterministic"


def test_load_canonical_scenarios_ordering_is_lexicographic() -> None:
    """Keys must be lexicographically ordered (sorted glob())."""
    loaded = _load_canonical_scenarios()
    keys = list(loaded.keys())
    assert keys == sorted(keys), f"Keys not lexicographically sorted: {keys}"


# ---------------------------------------------------------------------------
# Manual presets that have no canonical file remain intact
# ---------------------------------------------------------------------------


def test_minimal_preset_still_present() -> None:
    assert "minimal" in _SCENARIO_PRESETS
    assert _SCENARIO_PRESETS["minimal"]["scenario_id"] == "cli-minimal-scenario"


def test_phase4_runtime_replay_preset_still_present() -> None:
    assert "phase4-runtime-replay" in _SCENARIO_PRESETS
    assert _SCENARIO_PRESETS["phase4-runtime-replay"]["scenario_id"] == "phase4-runtime-replay-scenario"


# ---------------------------------------------------------------------------
# tiny-shared-combat has complete world mechanics (regression check)
# ---------------------------------------------------------------------------


def test_tiny_shared_combat_preset_has_quest_objectives() -> None:
    ssc = _SCENARIO_PRESETS.get("tiny-shared-combat")
    assert ssc is not None
    wcj = json.loads(ssc["scenario_vars"]["world_config_json"])
    assert "quest_objectives" in wcj
    assert len(wcj["quest_objectives"]) >= 1


def test_tiny_shared_combat_preset_has_room_description_overrides() -> None:
    ssc = _SCENARIO_PRESETS["tiny-shared-combat"]
    wcj = json.loads(ssc["scenario_vars"]["world_config_json"])
    assert "room_description_overrides" in wcj
    assert len(wcj["room_description_overrides"]) >= 1


def test_tiny_shared_combat_preset_has_npc_dialogue() -> None:
    ssc = _SCENARIO_PRESETS["tiny-shared-combat"]
    wcj = json.loads(ssc["scenario_vars"]["world_config_json"])
    assert "npc_dialogue" in wcj


def test_tiny_shared_combat_preset_has_respawn_rules() -> None:
    ssc = _SCENARIO_PRESETS["tiny-shared-combat"]
    wcj = json.loads(ssc["scenario_vars"]["world_config_json"])
    assert "npc_respawn_rules" in wcj


# ---------------------------------------------------------------------------
# Drift-sensitive: world_config_json in preset matches canonical file exactly
# ---------------------------------------------------------------------------


def test_tiny_shared_combat_world_config_json_matches_canonical_file() -> None:
    canonical_path = _CANONICAL_DIR / "tiny_shared_combat.json"
    on_disk = json.loads(canonical_path.read_text(encoding="utf-8"))
    preset = _SCENARIO_PRESETS["tiny-shared-combat"]
    on_disk_wcj = on_disk["scenario_vars"]["world_config_json"]
    preset_wcj = preset["scenario_vars"]["world_config_json"]
    # Both must parse to the same dict (content equality, not string equality)
    assert json.loads(on_disk_wcj) == json.loads(preset_wcj), (
        "world_config_json in preset does not match canonical file for tiny-shared-combat"
    )


def test_tiny_context_pressure_world_config_json_matches_canonical_file() -> None:
    canonical_path = _CANONICAL_DIR / "tiny_context_pressure.json"
    on_disk = json.loads(canonical_path.read_text(encoding="utf-8"))
    preset = _SCENARIO_PRESETS["tiny-context-pressure"]
    assert json.loads(on_disk["scenario_vars"]["world_config_json"]) == json.loads(
        preset["scenario_vars"]["world_config_json"]
    )
