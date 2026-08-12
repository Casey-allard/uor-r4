# Shard Identity and Encounter Taxonomy Design Note

## Status

- Type: architecture/design note
- Scope: implementation-aware identity/taxonomy note
- Intent: define shard identity classes and encounter taxonomy for persistent-world mode in a way that extends MUDBench’s current benchmark identity and reporting model

## 1. Why This Note Exists

MUDBench already has a real identity model in benchmark reporting, even if it is
still lightweight:

- built-in actor ids such as `agent-a` and `agent-b`
- external agent labels
- external agent profile ids
- identity-aware history/export rollups
- observer views for match summary, arena summary, and profile-centric rollups

The persistent-world architecture note says shard mode should add:

- durable principals
- durable entity identities
- profile reuse across shards and seasons

The export and observer note says shard mode should also support:

- identity-aware shard summaries
- encounter summaries
- replay-slice drilldowns

This document defines the taxonomy that connects those ideas. It does not
invent a separate identity universe. It extends the current benchmark/reporting
vocabulary into a persistent shard setting.

## 2. Benchmark Identities vs Persistent-World Identities

### 2.1 Benchmark Identities

In the current benchmark-first implementation, the primary identity unit is a
run-local comparison identity.

Examples:

- `actor_id = agent-a`
- `actor_id = deterministic-wrapper`
- `external_agent_profile_id = comparison-rule-profile`
- `external_agent_label = Comparison Rule Profile`

These are sufficient for:

- bounded run scorecards
- suite comparison output
- identity rollups over saved artifacts
- arena-style summaries for shared-run comparison artifacts

But they are still fundamentally run-scoped.

### 2.2 Persistent-World Identities

Persistent-world mode needs identities that survive beyond one run.

At minimum, it should distinguish:

- player account identity
- agent profile identity
- character/entity identity
- built-in/system-controlled identity
- transient session identity

This split matters because in shard mode:

- one player may own multiple characters over time
- one agent profile may be reused across many sessions and shards
- one character may reconnect through many sessions
- system-controlled NPCs and population agents need stable classification even
  when they are not “submitted competitors”

## 3. Identity Classes and Relationships

The taxonomy should use layered identities rather than one overloaded id.

### 3.1 Player Account

Represents a durable owner/controller identity for a human participant.

Purpose:

- account-level ownership
- moderation and access control
- season participation history
- multi-character association over time

Suggested field:

- `player_account_id`

This is a principal-level identity for humans.

### 3.2 Agent Profile

Represents a reusable agent configuration identity.

This is already partly visible today through:

- `external_agent_profile_id`

In shard mode it should generalize to:

- external profile identity
- built-in baseline profile identity
- hosted managed agent profile identity

Suggested field:

- `agent_profile_id`

Purpose:

- reuse across shards and seasons
- profile-aware rollups
- stable labeling in observer/export views

### 3.3 Character / Entity Identity

Represents the in-world controlled entity on a shard.

This is the world-level identity that moves, carries inventory, and participates
in encounters.

Suggested field:

- `entity_id`

Purpose:

- world location and inventory durability
- encounter participation
- shard replay attribution

In benchmark mode, `actor_id` often acts as both controller label and world
entity label. Persistent mode should separate those concepts explicitly.

### 3.4 Built-In / System-Controlled Identity

Not every shard participant is a submitted agent or human account.

Persistent mode should explicitly classify:

- built-in deterministic baselines
- system NPC controllers
- system population agents
- shard service actors such as moderators or scripted operators if added later

Suggested fields:

- `identity_class = built_in_actor | system_agent | npc_controller | human_player | external_agent`
- `system_identity_id` for stable internal references where needed

This avoids conflating “not human” with “submitted competitive agent.”

### 3.5 Session Identity

Represents the transient live connection/control session.

Suggested field:

- `session_id`

Purpose:

- reconnect tracking
- ingress health/debugging
- mapping short-lived control sessions to durable principals/profiles/entities

This should be transient, not the durable identity anchor.

## 4. Identity Relationship Model

The cleanest relationship model is:

- player account or agent owner principal
- attached to an agent profile or player account policy
- controlling one or more shard entities over time
- through one or more sessions

Illustrative mapping:

- `player_account_id` or principal controls
- `agent_profile_id` or human play style/policy
- which controls
- `entity_id`
- through
- `session_id`

Not every identity bundle needs every layer. For example:

- a benchmark built-in actor may have `actor_id` and `agent_profile_id`, but no
  long-lived `player_account_id`
- an NPC controller may have `entity_id` plus `identity_class = npc_controller`

## 5. Persistent vs Transient Identity Boundaries

### 5.1 Persistent

These should be durable and exportable across shard history:

- `player_account_id`
- `agent_profile_id`
- `entity_id`
- built-in/system identity classification
- season participation association

These are the identities that belong in rollups, history, and observer panels.

### 5.2 Transient

These should be operational rather than primary durable identity:

- `session_id`
- process handle
- local wrapper pid
- websocket connection id
- temporary rate-limit state

These matter for debugging, but not as the canonical identity of the
participant.

### 5.3 Bridge to Current Reporting

Current benchmark export already tracks:

- `actor_ids`
- `external_agent_label`
- `external_agent_profile_id`

Persistent mode should reuse these as bridge fields where useful, but the
durable taxonomy should prefer:

- principal/player identity
- profile identity
- entity identity

## 6. Benchmark Identity Bundle Example

Illustrative benchmark bundle:

```json
{
  "mode": "benchmark",
  "actor_id": "comparison-rule-profile",
  "identity_class": "external_agent",
  "external_agent_profile_id": "comparison-rule-profile",
  "external_agent_label": "Comparison Rule Profile",
  "entity_id": "agent-a"
}
```

Why this fits the current implementation:

- `actor_id` matches what current suite/report/history surfaces expose
- profile id and label match current manifest/export behavior
- `entity_id` is a forward-compatible bridge if benchmark runs continue to map
  one actor to one world entity

## 7. Persistent Shard Identity Bundle Example

Illustrative shard bundle:

```json
{
  "mode": "persistent_shard",
  "player_account_id": null,
  "agent_profile_id": "candidate-mock-profile",
  "entity_id": "entity-shard-alpha-agent-b",
  "identity_class": "external_agent",
  "external_agent_label": "Candidate Mock Profile",
  "session_id": "session-8841",
  "shard_id": "shard-alpha",
  "season_id": "season-1"
}
```

This shows the persistent split clearly:

- profile identity is durable across sessions
- entity identity is shard-local world identity
- session id is transient

## 8. Encounter Taxonomy Overview

Persistent shard mode needs a first-class encounter taxonomy so exports and
observer panels can summarize interactions coherently.

The taxonomy should distinguish at least four top-level categories:

- benchmark encounter
- shared-run arena encounter
- open-world interaction
- season/ladder-relevant encounter

## 9. Benchmark Encounter

A benchmark encounter is a bounded official evaluation unit.

Properties:

- fixed scenario
- fixed seed / max steps
- official scorecard semantics
- deterministic replay and parity expectations
- identities are run-local comparison identities

This already exists today through benchmark runs and suite artifacts.

Suggested field:

- `encounter_class = benchmark`

## 10. Shared-Run Arena Encounter

A shared-run arena encounter is the persistent-world-adjacent evolution of
today’s shared-run comparison artifact.

Properties:

- bounded competitive interaction
- multiple identities present in the same execution
- comparative per-scenario or per-encounter outcome
- replay drilldown is a selected shared-run window

Suggested fields:

- `encounter_class = arena_shared_run`
- `competition_scope = duel | team | scripted_match`

This maps directly to current observer concepts:

- match summary
- arena summary
- shared-run replay drilldown

## 11. Open-World Interaction

An open-world interaction is a shard-native encounter that emerges inside a
long-lived shard rather than in a fully bounded benchmark scenario.

Examples:

- opportunistic trade between two entities
- PvE combat against a shard NPC
- social exchange or negotiation
- contested item pickup in a shard region
- multi-entity economic exchange

Suggested field:

- `encounter_class = open_world`

Open-world interactions should still be exportable as bounded replay slices even
though they occur inside a longer shard history.

## 12. Encounter Type Families

Within any encounter class, the system should classify interaction type.

Minimum type families:

- `pvp`
- `pve`
- `social`
- `trade`
- `economic`
- `objective`
- `exploration`

These should be tags or controlled values rather than free text.

Examples:

- duel between two external profiles -> `pvp`
- agent clears guard to unlock area -> `pve`
- token handoff to trader -> `trade`, `social`
- market exchange affecting shard inventory -> `economic`, `trade`

This taxonomy maps naturally to current capability and scenario themes already
present in the repo.

## 13. Season/Ladder-Relevant Encounter Classes

Not every encounter should affect rating or seasonal summaries equally.

Suggested classes:

- `season_eligible_ranked`
- `season_eligible_unranked`
- `season_exhibition`
- `world_only_noncompetitive`

Examples:

- official shard duel window -> `season_eligible_ranked`
- scripted showcase match -> `season_exhibition`
- spontaneous hallway trade -> `world_only_noncompetitive`

This lets persistent mode remain broad while preserving fair season summaries.

## 14. Shared-Run Arena Encounter Example

Illustrative encounter record:

```json
{
  "encounter_id": "arena-shard-alpha-0007",
  "encounter_class": "arena_shared_run",
  "interaction_types": ["pvp", "objective"],
  "season_class": "season_eligible_ranked",
  "shard_id": "shard-alpha",
  "season_id": "season-1",
  "participants": [
    {
      "entity_id": "entity-agent-a",
      "agent_profile_id": "baseline-rule-profile",
      "identity_class": "external_agent"
    },
    {
      "entity_id": "entity-agent-b",
      "agent_profile_id": "candidate-mock-profile",
      "identity_class": "external_agent"
    }
  ],
  "tick_range": {"start": 18421, "end": 18510},
  "outcome_summary": {
    "winner_identity": "candidate-mock-profile",
    "score_difference": 0.1
  },
  "replay_slice_id": "encounter-7"
}
```

This is a direct shard-oriented generalization of the current shared-run arena
summary/reporting model.

## 15. Open-World Interaction Example

Illustrative encounter record:

```json
{
  "encounter_id": "openworld-shard-alpha-trade-021",
  "encounter_class": "open_world",
  "interaction_types": ["trade", "social", "economic"],
  "season_class": "world_only_noncompetitive",
  "shard_id": "shard-alpha",
  "season_id": "season-1",
  "participants": [
    {
      "entity_id": "entity-agent-a",
      "agent_profile_id": "candidate-mock-profile",
      "identity_class": "external_agent"
    },
    {
      "entity_id": "npc-trader-1",
      "identity_class": "npc_controller"
    }
  ],
  "tick_range": {"start": 19002, "end": 19011},
  "outcome_summary": {
    "item_transfers": ["trade-token -> npc-trader-1", "artifact -> entity-agent-a"]
  },
  "replay_slice_id": "trade-window-021"
}
```

This is an example of a shard-native interaction that should be visible in
observer drilldowns and identity rollups without pretending it is a benchmark
match.

## 16. Identity and Encounter Fields for Shard Exports and Observer Panels

Shard exports and observer panels should include a minimal common field set.

### 16.1 Identity Fields

Minimum identity-facing fields:

- `identity_class`
- `player_account_id` when present
- `agent_profile_id` when present
- `external_agent_label` when present
- `entity_id`
- `shard_id`
- `season_id` when present

### 16.2 Encounter Fields

Minimum encounter-facing fields:

- `encounter_id`
- `encounter_class`
- `interaction_types`
- `season_class`
- `participants`
- `tick_range`
- `outcome_summary`
- `replay_slice_id`

### 16.3 Observer Mapping

These fields should map cleanly into observer panels such as:

- identity rollup panel
- match summary
- arena summary
- replay inspector / replay-slice inspector

## 17. Summary and Aggregation Implications

The taxonomy should improve aggregation rather than complicate it.

### 17.1 Rollups

Identity rollups should be able to aggregate by:

- built-in actor identity
- external label
- external profile id
- player account id
- entity id where local shard analysis needs it

Persistent mode should favor profile/principal rollups for longitudinal
summaries, while entity rollups remain useful for shard-local drilldowns.

### 17.2 Match Summaries

Match summaries should apply to:

- benchmark bounded comparisons
- bounded shard encounters that are matchup-shaped

This is the observer bridge between current comparison artifacts and future
ranked shard encounters.

### 17.3 Seasons

Season summaries should aggregate only encounters eligible for season-level
comparison according to `season_class`.

This prevents open-world background interactions from being treated as if they
were all ranked competitive matches.

## 18. Built-In and System-Controlled Identity Rules

Persistent mode needs explicit handling for non-submitted actors.

Suggested rules:

- benchmark built-ins may appear as `built_in_actor`
- shard NPCs may appear as `npc_controller`
- shard system population agents may appear as `system_agent`
- only selected classes should be eligible for ladder/rating summaries

This keeps the taxonomy honest and prevents observer summaries from implying all
participants are equivalent competitive identities.

## 19. Phased Implementation Path

### Phase 0: Current Benchmark Identity State

Already present:

- `actor_ids`
- `external_agent_label`
- `external_agent_profile_id`
- identity rollups
- shared-run comparison and arena observer concepts

### Phase 1: Persistent Identity Layer

Add:

- `player_account_id`
- durable `agent_profile_id`
- durable `entity_id`
- `identity_class`

Goal:

- make shard participants classifiable without losing current benchmark identity
  compatibility

### Phase 2: Encounter Taxonomy in Exports

Add:

- `encounter_class`
- `interaction_types`
- `season_class`
- participant bundles in shard replay-slice exports

Goal:

- make observer summaries and replay slices semantically consistent

### Phase 3: Rollup and Observer Integration

Add:

- identity-aware shard rollups
- encounter-aware match and arena panels
- season eligibility-aware summaries

Goal:

- support profile/principal/season summaries over persistent-world data

### Phase 4: Ladder and Moderation Integration

Add:

- final season-eligible identity classes
- ladder/rating encounter eligibility rules
- moderation/support identity distinctions

Goal:

- support hosted arena operations without confusing benchmark and persistent
  identities

## 20. Recommended Guardrails

To keep the taxonomy implementation-aware and benchmark-safe:

- do not replace benchmark `actor_id` with persistent identity layers in
  official benchmark artifacts
- do not treat every shard interaction as a ranked encounter
- keep `entity_id` separate from `agent_profile_id`
- keep system/NPC identities explicitly classified
- keep observer summaries honest about whether an interaction is benchmark,
  shared-run arena, or open-world

## 21. Bottom Line

MUDBench already has the beginning of an identity taxonomy:

- actor ids
- external labels
- external profile ids
- identity-aware rollups
- shared-run match and arena summaries

Persistent-world mode should extend that into a layered taxonomy:

- player or principal identity
- agent profile identity
- world entity identity
- transient session identity
- explicit encounter classes and interaction types

That gives shard exports and observer panels a concrete vocabulary for
longitudinal world analysis without discarding the benchmark-first identity and
comparison model that already exists today.
