"""Tests for shared-world human playability improvements.

Covers:
- Fix 1: Only non-neutral NPCs appear as attack targets (opt-out: 'neutral'/'peaceful' tag excludes)
- Fix 2: Consumables do not generate give-to-NPC actions
- Fix 3: Party status shows default health (100 HP) instead of ? HP
- Fix 4: Contextual affordance hints for ward-talisman and medkit
- Non-regression: non-consumable items still support give actions
- Non-regression: use action still works for all item types
- Non-regression: talk still available for non-hostile NPCs
- Non-regression: existing scenarios unaffected
"""

from __future__ import annotations

import json

from agents.gateway.observation_builder import _action_space
from cli.main import _SCENARIO_PRESETS
from evaluation.benchmark_runner.runner import (
    _format_affordance_hints,
    _format_party_status_messages,
    build_shared_shard_loop_session,
)
from world.state.world_state import DeterministicWorldStateManager


# ---------------------------------------------------------------------------
# helpers
# ---------------------------------------------------------------------------

def _make_entities(
    *,
    npcs: list[dict] | None = None,
    items: list[dict] | None = None,
    actor_health: int | None = 100,
) -> dict:
    """Build a minimal entities dict for action-space tests."""
    entities: dict = {
        "actor-a": {
            "entity_id": "actor-a",
            "entity_type": "player",
            "location": "room",
            "health": actor_health,
            "inventory": [],
            "tags": [],
        },
    }
    for npc in (npcs or []):
        entities[npc["entity_id"]] = npc
    for item in (items or []):
        entities[item["entity_id"]] = item
    return entities


def _npc(entity_id: str, tags: list[str] | None = None) -> dict:
    return {
        "entity_id": entity_id,
        "entity_type": "npc",
        "location": "room",
        "health": 20,
        "inventory": [],
        "tags": tags or [],
    }


def _item(entity_id: str, entity_type: str = "item") -> dict:
    return {
        "entity_id": entity_id,
        "entity_type": entity_type,
        "location": "room",
        "health": None,
        "inventory": [],
        "tags": [],
    }


def _action_space_for(
    *,
    entities: dict,
    room_entity_ids: list[str],
    inventory: list[str] | None = None,
    exits: list[str] | None = None,
) -> tuple[str, ...]:
    return _action_space(
        exits=exits or [],
        room_entity_ids=room_entity_ids,
        entities=entities,
        inventory=inventory or [],
    )


# ---------------------------------------------------------------------------
# Fix 1: Only hostile NPCs as attack targets
# ---------------------------------------------------------------------------

class TestHostileOnlyAttackTargets:
    def test_hostile_npc_appears_as_attack_target(self) -> None:
        entities = _make_entities(npcs=[_npc("orc", tags=["hostile"])])
        space = _action_space_for(entities=entities, room_entity_ids=["orc"])
        assert "attack orc" in space

    def test_neutral_tagged_npc_not_an_attack_target(self) -> None:
        """NPC explicitly tagged 'neutral' is excluded from attack targets."""
        entities = _make_entities(npcs=[_npc("scout", tags=["neutral"])])
        space = _action_space_for(entities=entities, room_entity_ids=["scout"])
        assert "attack scout" not in space

    def test_untagged_npc_is_an_attack_target(self) -> None:
        """NPC with no tags is attackable by default (opt-out model)."""
        entities = _make_entities(npcs=[_npc("guardian", tags=[])])
        space = _action_space_for(entities=entities, room_entity_ids=["guardian"])
        assert "attack guardian" in space

    def test_neutral_npc_still_has_talk_action(self) -> None:
        entities = _make_entities(npcs=[_npc("scout", tags=["neutral"])])
        space = _action_space_for(entities=entities, room_entity_ids=["scout"])
        assert "talk scout" in space

    def test_hostile_npc_has_no_talk_action(self) -> None:
        """Hostile NPCs can be attacked but not talked to."""
        entities = _make_entities(npcs=[_npc("patrol", tags=["hostile"])])
        space = _action_space_for(entities=entities, room_entity_ids=["patrol"])
        assert "talk patrol" not in space

    def test_mixed_neutral_tagged_and_hostile_npcs(self) -> None:
        """Neutral-tagged NPC is talkable; hostile NPC is attackable — both present."""
        entities = _make_entities(npcs=[
            _npc("patrol", tags=["hostile"]),
            _npc("scout", tags=["neutral"]),
        ])
        space = _action_space_for(
            entities=entities, room_entity_ids=["patrol", "scout"]
        )
        assert "attack patrol" in space
        assert "attack scout" not in space
        assert "talk scout" in space
        assert "talk patrol" not in space

    def test_previously_hostile_then_calmed_npc_becomes_attackable(self) -> None:
        """After calming, an NPC loses 'hostile' tag but gains no 'neutral' tag,
        so it remains technically attackable (opt-out model). It can also be talked to."""
        calmed_patrol = _npc("patrol", tags=[])  # hostile tag removed after calming
        entities = _make_entities(npcs=[calmed_patrol])
        space = _action_space_for(entities=entities, room_entity_ids=["patrol"])
        assert "attack patrol" in space  # no neutral tag → still attackable
        assert "talk patrol" in space

    def test_outpost_courtyard_no_attack_scout(self) -> None:
        """Regression: In the outpost scenario, attack scout must NOT appear."""
        session = build_shared_shard_loop_session(
            scenario=_SCENARIO_PRESETS["small-cooperative-outpost"],
            actor_ids=["actor-a", "actor-b"],
            run_id="test-no-attack-scout",
            max_steps_override=20,
        )
        # Navigate to courtyard
        session.advance_tick({"actor-a": "take ward-talisman", "actor-b": "wait"})
        session.advance_tick({"actor-a": "move east", "actor-b": "move east"})
        for _ in range(2):
            session.advance_tick({"actor-a": "attack sentry", "actor-b": "attack sentry"})
        session.advance_tick({"actor-a": "move east", "actor-b": "move east"})
        obs = session.get_observation("actor-a")
        assert "attack scout" not in obs.action_space
        assert "talk scout" in obs.action_space

    def test_outpost_courtyard_patrol_still_attackable(self) -> None:
        """Regression: Hostile patrol remains attackable in courtyard."""
        session = build_shared_shard_loop_session(
            scenario=_SCENARIO_PRESETS["small-cooperative-outpost"],
            actor_ids=["actor-a", "actor-b"],
            run_id="test-patrol-attackable",
            max_steps_override=20,
        )
        session.advance_tick({"actor-a": "move east", "actor-b": "move east"})
        for _ in range(2):
            session.advance_tick({"actor-a": "attack sentry", "actor-b": "attack sentry"})
        session.advance_tick({"actor-a": "move east", "actor-b": "move east"})
        obs = session.get_observation("actor-a")
        assert "attack patrol" in obs.action_space


# ---------------------------------------------------------------------------
# Fix 2: No give-consumable-to-NPC actions
# ---------------------------------------------------------------------------

class TestNoGiveConsumableToNpc:
    def test_consumable_in_inventory_no_give_to_npc(self) -> None:
        entities = _make_entities(
            npcs=[_npc("patrol", tags=["hostile"])],
            items=[_item("ward-talisman", "consumable")],
        )
        # ward-talisman is in inventory (actor has it)
        space = _action_space_for(
            entities=entities,
            room_entity_ids=["patrol"],
            inventory=["ward-talisman"],
        )
        # consumable should NOT generate give action to NPC
        assert "give ward-talisman patrol" not in space
        # but use action still present
        assert "use ward-talisman" in space

    def test_regular_item_still_generates_give_to_npc(self) -> None:
        entities = _make_entities(
            npcs=[_npc("trader", tags=[])],
            items=[_item("relic-key", "item")],
        )
        space = _action_space_for(
            entities=entities,
            room_entity_ids=["trader"],
            inventory=["relic-key"],
        )
        assert "give relic-key trader" in space

    def test_use_action_still_works_for_regular_item(self) -> None:
        """Non-consumable items still generate use actions."""
        entities = _make_entities(items=[_item("relic-key", "item")])
        space = _action_space_for(
            entities=entities,
            room_entity_ids=[],
            inventory=["relic-key"],
        )
        assert "use relic-key" in space

    def test_outpost_courtyard_no_give_talisman_to_scout(self) -> None:
        session = build_shared_shard_loop_session(
            scenario=_SCENARIO_PRESETS["small-cooperative-outpost"],
            actor_ids=["actor-a", "actor-b"],
            run_id="test-no-give-consumable",
            max_steps_override=20,
        )
        session.advance_tick({"actor-a": "take ward-talisman", "actor-b": "wait"})
        session.advance_tick({"actor-a": "move east", "actor-b": "move east"})
        for _ in range(2):
            session.advance_tick({"actor-a": "attack sentry", "actor-b": "attack sentry"})
        session.advance_tick({"actor-a": "move east", "actor-b": "move east"})
        obs = session.get_observation("actor-a")
        assert "give ward-talisman scout" not in obs.action_space
        assert "give ward-talisman patrol" not in obs.action_space
        # use action still present
        assert "use ward-talisman" in obs.action_space


# ---------------------------------------------------------------------------
# Fix 3: Party status shows default HP instead of ? HP
# ---------------------------------------------------------------------------

class TestPartyStatusDefaultHealth:
    def _make_snapshot(
        self,
        *,
        co_health: int | None = None,
        co_location: str = "supply-camp",
    ) -> dict:
        return {
            "entities": {
                "actor-b": {
                    "entity_id": "actor-b",
                    "entity_type": "agent",
                    "location": co_location,
                    "health": co_health,
                    "inventory": [],
                    "tags": [],
                },
            }
        }

    def test_none_health_shows_default_not_question_mark(self) -> None:
        snap = self._make_snapshot(co_health=None)
        msgs = _format_party_status_messages(snap, {}, ["actor-a", "actor-b"], "actor-a")
        assert all("? HP" not in m for m in msgs)
        assert any("100 HP" in m for m in msgs)

    def test_explicit_health_shows_correctly(self) -> None:
        snap = self._make_snapshot(co_health=75)
        msgs = _format_party_status_messages(snap, {}, ["actor-a", "actor-b"], "actor-a")
        assert any("75 HP" in m for m in msgs)

    def test_full_health_default_at_session_start(self) -> None:
        session = build_shared_shard_loop_session(
            scenario=_SCENARIO_PRESETS["small-cooperative-outpost"],
            actor_ids=["actor-a", "actor-b"],
            run_id="test-party-hp",
        )
        obs = session.get_observation("actor-a")
        party_msgs = [m for m in obs.messages if m.startswith("[Party]")]
        assert party_msgs
        assert all("? HP" not in m for m in party_msgs)
        assert any("100 HP" in m for m in party_msgs)


# ---------------------------------------------------------------------------
# Fix 4: Contextual affordance hints
# ---------------------------------------------------------------------------

class TestAffordanceHints:
    def _make_world_with_calm_effect(
        self,
        *,
        actor_location: str = "courtyard",
        inventory: list[str] | None = None,
        npc_health: int = 20,
        already_calmed: bool = False,
    ) -> tuple[DeterministicWorldStateManager, list[str]]:
        inv = inventory if inventory is not None else ["ward-talisman"]
        world = DeterministicWorldStateManager.from_dict({
            "entities": {
                "actor-a": {
                    "entity_id": "actor-a",
                    "entity_type": "player",
                    "location": actor_location,
                    "health": 80,
                    "inventory": inv,
                    "tags": [],
                },
                "patrol": {
                    "entity_id": "patrol",
                    "entity_type": "npc",
                    "location": "courtyard",
                    "health": npc_health,
                    "inventory": [],
                    "tags": ["hostile"],
                },
                "ward-talisman": {
                    "entity_id": "ward-talisman",
                    "entity_type": "consumable",
                    "location": "inventory",
                    "health": None,
                    "inventory": [],
                    "tags": [],
                },
            },
            "rooms": {
                actor_location: {
                    "title": "Room",
                    "description": "Test room.",
                    "exits": {},
                    "entities_present": ["actor-a", "patrol"],
                },
            },
            "scenario_vars": {
                "calm_npc_effects_json": json.dumps([
                    {
                        "effect_id": "patrol-calmed",
                        "item_id": "ward-talisman",
                        "source_room_id": "courtyard",
                        "target_npc_id": "patrol",
                    }
                ]),
                **({"calmed.patrol-calmed": True} if already_calmed else {}),
            },
        })
        return world, inv

    def test_calm_hint_appears_when_in_correct_room_with_item(self) -> None:
        world, inv = self._make_world_with_calm_effect(actor_location="courtyard")
        snap = world.get_snapshot()
        hints = _format_affordance_hints(
            snap, snap.get("scenario_vars", {}),
            "actor-a", "courtyard", inv,
        )
        assert any("ward-talisman" in h and "calm" in h.lower() for h in hints)

    def test_calm_hint_absent_in_wrong_room(self) -> None:
        world, inv = self._make_world_with_calm_effect(actor_location="supply-camp")
        snap = world.get_snapshot()
        hints = _format_affordance_hints(
            snap, snap.get("scenario_vars", {}),
            "actor-a", "supply-camp", inv,
        )
        assert not any("ward-talisman" in h and "calm" in h.lower() for h in hints)

    def test_calm_hint_absent_when_item_not_in_inventory(self) -> None:
        world, _ = self._make_world_with_calm_effect(actor_location="courtyard")
        snap = world.get_snapshot()
        hints = _format_affordance_hints(
            snap, snap.get("scenario_vars", {}),
            "actor-a", "courtyard", [],  # empty inventory
        )
        assert not any("calm" in h.lower() for h in hints)

    def test_calm_hint_absent_when_already_calmed(self) -> None:
        world, inv = self._make_world_with_calm_effect(
            actor_location="courtyard", already_calmed=True
        )
        snap = world.get_snapshot()
        hints = _format_affordance_hints(
            snap, snap.get("scenario_vars", {}),
            "actor-a", "courtyard", inv,
        )
        assert not any("calm" in h.lower() for h in hints)

    def test_revive_hint_appears_when_defeated_co_actor_in_room(self) -> None:
        world = DeterministicWorldStateManager.from_dict({
            "entities": {
                "actor-a": {
                    "entity_id": "actor-a",
                    "entity_type": "player",
                    "location": "camp",
                    "health": 80,
                    "inventory": ["medkit"],
                    "tags": [],
                },
                "actor-b": {
                    "entity_id": "actor-b",
                    "entity_type": "agent",
                    "location": "camp",
                    "health": 0,
                    "inventory": [],
                    "tags": [],
                },
                "medkit": {
                    "entity_id": "medkit",
                    "entity_type": "consumable",
                    "location": "inventory",
                    "health": None,
                    "inventory": [],
                    "tags": [],
                },
            },
            "rooms": {
                "camp": {
                    "title": "Camp",
                    "description": "Test.",
                    "exits": {},
                    "entities_present": ["actor-a", "actor-b"],
                },
            },
            "scenario_vars": {
                "actor_defeated.actor-b": True,
                "actor_revive_effects_json": json.dumps([
                    {"effect_id": "medkit-revive", "item_id": "medkit", "revive_health": 30}
                ]),
            },
        })
        snap = world.get_snapshot()
        hints = _format_affordance_hints(
            snap, snap.get("scenario_vars", {}),
            "actor-a", "camp", ["medkit"],
        )
        assert any("medkit" in h and "actor-b" in h for h in hints)

    def test_revive_hint_absent_when_no_defeated_actor(self) -> None:
        world = DeterministicWorldStateManager.from_dict({
            "entities": {
                "actor-a": {"entity_id": "actor-a", "entity_type": "player", "location": "camp", "health": 80, "inventory": ["medkit"], "tags": []},
                "actor-b": {"entity_id": "actor-b", "entity_type": "agent", "location": "camp", "health": 80, "inventory": [], "tags": []},
                "medkit": {"entity_id": "medkit", "entity_type": "consumable", "location": "inventory", "health": None, "inventory": [], "tags": []},
            },
            "rooms": {"camp": {"title": "Camp", "description": "Test.", "exits": {}, "entities_present": ["actor-a", "actor-b"]}},
            "scenario_vars": {
                "actor_revive_effects_json": json.dumps([
                    {"effect_id": "medkit-revive", "item_id": "medkit", "revive_health": 30}
                ]),
            },
        })
        snap = world.get_snapshot()
        hints = _format_affordance_hints(
            snap, snap.get("scenario_vars", {}),
            "actor-a", "camp", ["medkit"],
        )
        assert not any("revive" in h.lower() for h in hints)

    def test_outpost_courtyard_shows_calm_hint_with_talisman(self) -> None:
        session = build_shared_shard_loop_session(
            scenario=_SCENARIO_PRESETS["small-cooperative-outpost"],
            actor_ids=["actor-a", "actor-b"],
            run_id="test-calm-hint",
            max_steps_override=20,
        )
        session.advance_tick({"actor-a": "take ward-talisman", "actor-b": "wait"})
        session.advance_tick({"actor-a": "move east", "actor-b": "move east"})
        for _ in range(2):
            session.advance_tick({"actor-a": "attack sentry", "actor-b": "attack sentry"})
        session.advance_tick({"actor-a": "move east", "actor-b": "move east"})
        obs = session.get_observation("actor-a")
        assert any("[Hint]" in m and "ward-talisman" in m for m in obs.messages)

    def test_no_hint_without_scenario_effects(self) -> None:
        """Default scenario (no calm/revive effects) produces no affordance hints."""
        session = build_shared_shard_loop_session(
            scenario=_SCENARIO_PRESETS["tiny-fetch-quest"],
            actor_ids=["actor-a", "actor-b"],
            run_id="test-no-hint-default",
        )
        obs = session.get_observation("actor-a")
        assert not any("[Hint]" in m for m in obs.messages)


# ---------------------------------------------------------------------------
# Non-regression: existing scenarios
# ---------------------------------------------------------------------------

class TestNonRegression:
    def test_tiny_shared_combat_ward_sigil_still_use(self) -> None:
        """In tiny-shared-combat, ward-sigil is a consumable → use action present."""
        session = build_shared_shard_loop_session(
            scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
            actor_ids=["actor-a", "actor-b"],
            run_id="test-nr-tiny-combat",
            max_steps_override=20,
        )
        session.advance_tick({"actor-a": "take ward-sigil", "actor-b": "wait"})
        obs = session.get_observation("actor-a")
        assert "use ward-sigil" in obs.action_space
        # No give ward-sigil to guardian/patrol (consumable)
        assert not any("give ward-sigil" in a for a in obs.action_space)

    def test_tiny_guarded_relic_relic_key_give_still_works(self) -> None:
        """relic-key is an item (not consumable) — give to NPC still available if applicable."""
        from agents.gateway.observation_builder import _action_space
        # relic-key entity_type = "item" → give actions still generated
        entities = {
            "actor": {
                "entity_id": "actor", "entity_type": "player",
                "location": "room", "health": 100, "inventory": ["relic-key"], "tags": [],
            },
            "relic-key": {
                "entity_id": "relic-key", "entity_type": "item",
                "location": "inventory", "health": None, "inventory": [], "tags": [],
            },
            "guard": {
                "entity_id": "guard", "entity_type": "npc",
                "location": "room", "health": 10, "inventory": [], "tags": [],
            },
        }
        space = _action_space(
            exits=[],
            room_entity_ids=["guard"],
            entities=entities,
            inventory=["relic-key"],
        )
        assert "use relic-key" in space
        assert "give relic-key guard" in space  # non-consumable item give still present

    def test_determinism_action_space_unchanged_across_sessions(self) -> None:
        """Same scenario state always produces the same action space."""
        def _get_initial_space() -> tuple[str, ...]:
            session = build_shared_shard_loop_session(
                scenario=_SCENARIO_PRESETS["small-cooperative-outpost"],
                actor_ids=["actor-a", "actor-b"],
                run_id="test-determinism",
            )
            return session.get_observation("actor-a").action_space

        assert _get_initial_space() == _get_initial_space()
