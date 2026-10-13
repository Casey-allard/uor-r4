# Persistent Identity Model

## Status

- Type: implementation-oriented design/spec note
- Scope: persistent identity model for shard/world mode
- Intent: define the concrete identity classes and lifecycle that bridge MUDBench’s current benchmark identity system into persistent shard/world mode

## 1. Why This Note Exists

MUDBench already has a lightweight but real identity system in benchmark mode:

- built-in `actor_id` values such as `agent-a` and `agent-b`
- external agent labels
- external agent profile ids
- identity-aware history/export rollups
- observer summaries that already distinguish built-ins, labels, profiles, and
  shared-run comparison contexts

The shard design notes require a stronger model:

- identities that persist beyond one run
- identities that survive reconnects
- identities that separate owners from in-world entities
- identities that support shard exports, encounters, standings, and observer
  views

This note defines that bridge in implementation-oriented terms.

## 2. Design Goals

The persistent identity model should:

- preserve backward compatibility with the current benchmark actor/profile model
- separate durable identity from transient presence
- support both human players and agent-controlled participants
- support built-in/system-controlled shard actors explicitly
- provide stable keys for shard exports, rollups, and standings
- remain concrete enough to guide a code-level shard identity registry later

## 3. Identity Classes

Persistent-world mode should use five explicit identity classes.

### 3.1 Account Identity

Represents the durable owner/controller principal for a human participant.

Suggested primary field:

- `account_id`

Purpose:

- login ownership
- moderation/admin linkage
- season participation at the account level
- mapping many sessions and potentially many characters to one durable owner

This is the persistent-world replacement for “human user identity,” which
benchmark mode does not currently need.

### 3.2 Agent Profile Identity

Represents a reusable agent configuration identity.

Suggested primary field:

- `agent_profile_id`

This should bridge directly from the current benchmark system’s:

- `external_agent_profile_id`

It should also support:

- built-in baseline profiles
- external local-process profiles
- future hosted managed profiles

Purpose:

- stable profile-aware rollups
- season participation continuity
- export and observer labeling

### 3.3 Character Identity

Represents the durable in-world controlled entity inside a shard.

Suggested primary field:

- `character_id`

This is the world-facing identity for:

- inventory ownership
- location
- encounter participation
- shard persistence

This maps conceptually to the broader shard notes’ `entity_id`, but this spec
uses “character identity” to make the registry role clearer for player- and
agent-controlled world participants.

### 3.4 Shard Session / Presence Identity

Represents the transient live presence of an account or profile controlling a
character on a shard.

Suggested primary fields:

- `session_id`
- `presence_id` if presence needs to be distinguished from transport session

Purpose:

- login/connect state
- reconnect handling
- local-process session tracking
- transient ingress health/debugging

This must not become the durable identity anchor.

### 3.5 Built-In / System-Controlled Identity

Represents non-user system participants.

Suggested primary fields:

- `system_identity_id`
- `identity_class`

Supported kinds should include at minimum:

- `built_in_actor`
- `system_agent`
- `npc_controller`
- `human_player`
- `external_agent`

This is necessary because not every shard participant is a submitted competitor
or human player, and the export/observer layer already benefits from explicit
identity typing.

## 4. Relationship Model

The implementation-oriented relationship model should be:

- one `account_id` may own zero or more `agent_profile_id` values and/or human
  play identities
- one `agent_profile_id` may control many `character_id` values over time, but
  normally one active controlled character at once per shard policy
- one `character_id` belongs to exactly one shard context at a time
- one `character_id` may be associated with many historical `session_id` values
- one `system_identity_id` may back many NPC or built-in controlled entities if
  the shard policy chooses that model

This is a deliberately practical bridge from the current model, where
`actor_id` often implicitly serves as:

- controller label
- profile label
- world entity label

Persistent mode should stop overloading one field that way.

## 5. Durable vs Transient Identity Fields

### 5.1 Durable Fields

These should survive process restarts and belong in shard persistence and
exports:

- `account_id`
- `agent_profile_id`
- `character_id`
- `identity_class`
- shard membership association
- season participation association
- observer/export display label where stable

### 5.2 Transient Fields

These should remain operational and not be the primary durable truth:

- `session_id`
- `presence_id`
- transport handle
- process pid
- websocket connection id
- local queue state
- heartbeat state

### 5.3 Borderline Fields

Some fields may be cached in checkpoints for recovery convenience but should
still be semantically transient, for example:

- last seen timestamp
- reconnect token or temporary ingress state

The design rule is:

- if shard history, standings, or encounters need the field as identity truth,
  it is durable
- if the field exists mainly to keep a live connection healthy, it is transient

## 6. Benchmark Mode Boundary

Benchmark mode should keep its current public identity model intact.

Current benchmark-facing fields remain:

- `actor_id`
- `external_agent_profile_id`
- `external_agent_label`

Persistent identity layers should not replace those fields in official benchmark
artifacts.

Instead, benchmark mode should treat persistent identity as an optional
behind-the-scenes bridge when useful:

- `actor_id` may map to a profile identity internally
- benchmark exports still stay run-scoped and benchmark-oriented

This prevents accidental public-interface drift in benchmark reporting.

## 7. Persistent World Mode Boundary

Persistent-world mode should expose the layered model explicitly.

At minimum, shard-facing data should be able to answer:

- which account or owner is involved
- which profile or control policy is involved
- which character/entity in the shard is involved
- which session/presence is currently active
- whether the participant is built-in, external, human, or system-controlled

This is the model that shard exports, encounters, standings, and observer views
should be built on.

## 8. Identity Lifecycle

The identity model should support a simple explicit lifecycle.

### 8.1 Login / Attach

At login or attach time:

1. authenticate or identify the controlling account/profile
2. resolve allowed `agent_profile_id` or human account context
3. attach or create the shard-local `character_id`
4. create a new transient `session_id`
5. mark presence active on the shard

### 8.2 Spawn / World Entry

At spawn time:

1. resolve shard
2. resolve or create `character_id`
3. place the character in world state
4. record roster membership in durable shard state
5. tie transient presence to the durable character

### 8.3 Disconnect

At disconnect:

1. close or expire `session_id`
2. mark presence inactive
3. keep `account_id`, `agent_profile_id`, and `character_id` durable
4. preserve character world state according to shard rules

### 8.4 Reconnect

At reconnect:

1. re-identify the account/profile
2. resolve the durable `character_id`
3. create a new `session_id`
4. reattach presence to the same durable character identity

This is why the session layer must remain transient and separate.

## 9. Identity Linkage to Encounters

Encounter records should attach to persistent identity layers explicitly.

Minimum linkage:

- `character_id`
- `agent_profile_id` when present
- `account_id` when present
- `identity_class`

This allows:

- encounter attribution across reconnects
- profile-aware season eligibility decisions
- observer summaries that distinguish profile identity from world entity

This aligns directly with the existing shard encounter taxonomy and shared-run
arena/export notes.

## 10. Identity Linkage to Standings and Rating Exports

Standings and rating exports should primarily use:

- `agent_profile_id` or `account_id` as the season-visible identity

They may include:

- `character_id` as drilldown context

Recommended rule:

- standings use the most stable competitive identity
- world state and encounter history use `character_id`
- live control state uses `session_id`

This keeps profile-aware rollups and standing rows stable even if a participant
reconnects or respawns.

## 11. Identity Linkage to Exports and Observer Views

The identity model should map directly into export and observer sections.

### 11.1 History / Rollups

Identity-aware exports should be able to summarize by:

- built-in actor identity
- external label
- agent profile identity
- account identity
- character identity where shard-local analysis needs it

### 11.2 Match / Arena / Encounter Views

Observer panels should prefer:

- profile or account identity in summary headers
- character identity in world-state or replay drilldown context

### 11.3 Replay / Recovery Context

Recovery and replay windows should be able to show:

- which `character_id` participated
- which `agent_profile_id` or `account_id` controlled that character
- which `session_id` was active if operational debugging is needed

## 12. Account Identity Record Example

Illustrative durable account identity record:

```json
{
  "account_id": "acct-player-001",
  "identity_class": "human_player",
  "display_name": "casey-allard",
  "account_status": "active",
  "season_ids": ["season-1"]
}
```

This is durable and owner-level.

## 13. Agent Profile Identity Record Example

Illustrative durable agent profile record:

```json
{
  "agent_profile_id": "candidate-mock-profile",
  "identity_class": "external_agent",
  "external_agent_label": "Candidate Mock Profile",
  "owner_account_id": "acct-builder-004",
  "ingress_mode": "local_process_profile",
  "profile_status": "active"
}
```

This is the persistent-world extension of today’s `external_agent_profile_id`.

## 14. Persistent Character Identity Record Example

Illustrative durable character record:

```json
{
  "character_id": "char-shard-alpha-agent-b",
  "shard_id": "shard-alpha",
  "controller_account_id": "acct-builder-004",
  "agent_profile_id": "candidate-mock-profile",
  "identity_class": "external_agent",
  "current_location": "market",
  "inventory_ref": "inventory-char-shard-alpha-agent-b"
}
```

This is the durable world-facing identity.

## 15. Shard-Local Session / Presence Record Example

Illustrative transient session record:

```json
{
  "session_id": "sess-8841",
  "presence_id": "presence-8841",
  "shard_id": "shard-alpha",
  "character_id": "char-shard-alpha-agent-b",
  "agent_profile_id": "candidate-mock-profile",
  "identity_class": "external_agent",
  "session_status": "connected"
}
```

This is transient live presence, not the durable identity anchor.

## 16. Built-In / System-Controlled Record Shape

Illustrative built-in/system record:

```json
{
  "system_identity_id": "sys-npc-trader-1",
  "identity_class": "npc_controller",
  "character_id": "npc-trader-1",
  "shard_id": "shard-alpha",
  "controller_mode": "system_controlled"
}
```

This keeps system actors explicit instead of forcing them into player/agent
identity slots.

## 17. Registry-Oriented Minimal Field Sets

If a future shard identity registry is added, the smallest useful registries
would likely be:

- account registry
- profile registry
- character registry
- session/presence registry
- system identity registry

Each should keep its own primary key and avoid collapsing durable and transient
data into one table/object if the implementation can avoid it.

## 18. Phased Implementation Path

### Phase 0: Current Benchmark Identity Bridge

Already present:

- `actor_id`
- `external_agent_profile_id`
- `external_agent_label`
- identity-aware rollups and observer summaries

### Phase 1: Durable Identity Records

Add:

- `account_id`
- `agent_profile_id`
- `character_id`
- `identity_class`

Goal:

- make persistent identity durable and exportable

### Phase 2: Presence / Session Layer

Add:

- `session_id`
- `presence_id`
- reconnect lifecycle handling

Goal:

- separate live connection state from durable shard identity

### Phase 3: Encounter / Standings Integration

Add:

- encounter participant bundles using persistent identity fields
- standings/rating rows keyed to profile/account identities
- observer/export views that show profile/account summary plus character
  drilldowns

Goal:

- align the identity registry with shard encounters and standings

### Phase 4: Code-Level Shard Identity Registry

Add:

- explicit registry module(s)
- write and lookup boundaries
- integration with shard persistence, recovery manifests, and shard exports

Goal:

- turn the model from a design note into a bounded implementation surface

## 19. Recommended Guardrails

To keep the identity model implementation-safe and benchmark-compatible:

- do not replace public benchmark `actor_id` fields with persistent identity
  fields
- do not let `session_id` become the durable owner key
- do not conflate `agent_profile_id` with `character_id`
- keep built-in/system-controlled identities explicitly typed
- keep standings and observer summaries keyed to the most stable competitive
  identity available

## 20. Bottom Line

The persistent identity bridge for MUDBench should be:

- `account_id` for durable ownership
- `agent_profile_id` for reusable control policy
- `character_id` for durable world participation
- `session_id` / `presence_id` for transient shard attachment
- `system_identity_id` plus `identity_class` for non-user actors

That gives MUDBench a practical path from today’s run-scoped benchmark identity
surface to a code-level shard identity registry without breaking the current
profile-aware export and observer behavior.
