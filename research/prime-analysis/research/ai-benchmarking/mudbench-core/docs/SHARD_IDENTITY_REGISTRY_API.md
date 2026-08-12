# Shard Identity Registry API Design Note

## Status

- Type: implementation-oriented design/spec note
- Scope: shard identity registry API and storage boundary
- Intent: define the code-level read/write surfaces that translate the
  persistent identity model into one bounded registry subsystem without
  spreading identity truth across world, session, and export layers

## 1. Why This Note Exists

MUDBench already has a real benchmark identity surface:

- built-in `actor_id` values such as `agent-a` and `agent-b`
- external agent labels
- external agent profile ids
- identity-aware history/export rollups
- observer views that group and filter by actor, label, and profile

The persistent-world notes add a layered identity model:

- `account_id`
- `agent_profile_id`
- `character_id`
- `session_id` or `presence_id`
- built-in/system-controlled identities

What is still missing is the concrete seam that keeps those identities from
drifting across subsystems. This note defines that seam:

- what the identity registry owns
- what world state owns
- what live session/presence state owns
- what exports and observer surfaces are allowed to read

The goal is not to introduce a large service architecture. The goal is to
define one bounded registry API that can be implemented inside the current
benchmark-first engine as shard mode grows.

## 2. Design Goals

The shard identity registry should:

- provide one canonical write surface for durable identity records and bindings
- prevent world/entity storage from becoming an accidental identity database
- keep transient session state out of durable identity truth
- support reconnect and recovery without guessing ownership
- support export and observer read models without giving reporting code direct
  write access to identity state
- preserve the current benchmark-facing public identity model

## 3. Non-Goals

The registry should not:

- own inventory, room position, quest progress, or other world facts
- become the transport/session manager
- replace benchmark run-local `actor_id` semantics in official benchmark
  artifacts
- compute standings or ratings directly
- embed full replay, checkpoint, or event-segment storage

## 4. Registry Purpose

The shard identity registry is the canonical subsystem for answering:

- who is this durable principal or controller
- which reusable profile or control policy is attached
- which durable character identity exists on which shard
- which transient session is currently attached to which durable identity
- whether a reconnect attempt may resume an existing presence safely

It should be the only write owner for durable identity and binding records.

## 5. Storage Boundary

The clean boundary is:

- registry-owned identity data
- world/entity-owned state
- session/transient presence state
- export/reporting read surfaces

### 5.1 Registry-Owned Identity Data

The registry owns durable identity and binding truth:

- account records
- agent profile records
- character records
- system identity records where needed
- binding records between account/profile/character/shard
- durable activation state such as active, suspended, retired
- reconnect-eligible attachment metadata

The registry may also own stable display metadata used in exports:

- canonical display name
- identity class
- provenance or source type

### 5.2 World / Entity-Owned State

The world layer owns mutable in-world facts:

- room or coordinate location
- inventory and equipped items
- quest/objective state
- combat state
- health/resource meters
- room/entity relationships
- time-local world effects

The world layer may reference `character_id`, but it must not redefine:

- who the character belongs to
- which profile controls it
- whether the character is active or retired

### 5.3 Session / Transient Presence State

The session layer owns operational state:

- live connection handle
- local-process runner handle
- pid / transport id / websocket id
- heartbeat and liveness state
- reconnect deadline window
- in-flight ingress queue state

This layer may mirror durable ids for routing, but it is not the durable source
of identity truth.

### 5.4 Export / Reporting Read Surfaces

Exports, history, standings, and observer tooling should read through registry
views, not mutate registry or world state.

They should consume:

- identity summaries
- profile/display metadata
- shard participation associations
- encounter-facing identity bundles
- season/standing-ready principal or profile references

They should not call low-level mutating registry operations.

## 6. Registry-Owned Record Types

The initial code-level registry should manage four first-class record families.

### 6.1 Account Record

Purpose:

- durable principal for a human owner or operator

Suggested fields:

- `account_id`
- `account_type`
- `display_name`
- `status`
- `created_generation`
- `updated_generation`
- `default_agent_profile_id` when applicable
- `metadata`

Example:

```json
{
  "record_type": "account",
  "account_id": "acct_human_alice",
  "account_type": "human_player",
  "display_name": "Alice",
  "status": "active",
  "created_generation": 12,
  "updated_generation": 24,
  "default_agent_profile_id": null,
  "metadata": {
    "source": "local-dev"
  }
}
```

### 6.2 Agent Profile Record

Purpose:

- durable reusable controller/profile identity
- bridge from current benchmark `external_agent_profile_id`

Suggested fields:

- `agent_profile_id`
- `controller_type`
- `display_name`
- `benchmark_bridge_actor_id` if any
- `status`
- `created_generation`
- `updated_generation`
- `metadata`

Notes:

- built-in profiles and external local-process profiles can share this family
- command strings and runtime-specific secrets should remain outside the durable
  registry payload unless explicitly needed later

### 6.3 Character Record

Purpose:

- durable shard-participant identity
- stable linkage target for world/entity state

Suggested fields:

- `character_id`
- `shard_id`
- `identity_class`
- `owner_account_id`
- `controller_agent_profile_id`
- `system_identity_id`
- `status`
- `spawn_template_id` if applicable
- `created_generation`
- `updated_generation`
- `metadata`

Example:

```json
{
  "record_type": "character",
  "character_id": "char_market_runner_01",
  "shard_id": "shard_dev_alpha",
  "identity_class": "external_agent",
  "owner_account_id": null,
  "controller_agent_profile_id": "profile_mock_llm_v1",
  "system_identity_id": null,
  "status": "active",
  "spawn_template_id": "market-runner",
  "created_generation": 18,
  "updated_generation": 41,
  "metadata": {
    "benchmark_bridge_actor_id": "agent-a"
  }
}
```

### 6.4 Session / Presence Record

Purpose:

- transient attach state for one active or recently active control presence

Suggested fields:

- `session_id`
- `presence_id`
- `shard_id`
- `character_id`
- `account_id`
- `agent_profile_id`
- `transport_kind`
- `status`
- `attached_at_generation`
- `detached_at_generation`
- `reconnect_token_ref`
- `metadata`

This record family may be checkpointed for recovery convenience, but it remains
semantically transient.

## 7. Read / Write API Surface

The registry API should stay intentionally small and explicit.

### 7.1 Account Identity API

Required writes:

- `create_account(record)`
- `update_account_metadata(account_id, patch)`
- `activate_account(account_id)`
- `deactivate_account(account_id, reason)`

Required reads:

- `get_account(account_id)`
- `find_account_by_display_name(display_name)`
- `list_accounts_for_export(filters)`

### 7.2 Agent Profile Identity API

Required writes:

- `create_agent_profile(record)`
- `update_agent_profile_metadata(agent_profile_id, patch)`
- `activate_agent_profile(agent_profile_id)`
- `deactivate_agent_profile(agent_profile_id, reason)`

Required reads:

- `get_agent_profile(agent_profile_id)`
- `find_agent_profile_by_benchmark_bridge(actor_id | external_profile_id)`
- `list_agent_profiles_for_export(filters)`

### 7.3 Character Identity API

Required writes:

- `create_character(record)`
- `bind_character_owner(character_id, account_id | null)`
- `bind_character_profile(character_id, agent_profile_id | null)`
- `bind_character_system_identity(character_id, system_identity_id | null)`
- `activate_character(character_id)`
- `deactivate_character(character_id, reason)`

Required reads:

- `get_character(character_id)`
- `list_characters_for_shard(shard_id)`
- `find_characters_by_account(account_id)`
- `find_characters_by_profile(agent_profile_id)`
- `build_character_export_bundle(character_id)`

### 7.4 Session / Presence API

Required writes:

- `attach_session(record)`
- `detach_session(session_id, reason)`
- `mark_session_active(session_id)`
- `mark_session_inactive(session_id, reason)`
- `bind_reconnect_token(session_id, token_ref, expires_at)`

Required reads:

- `get_session(session_id)`
- `find_active_session_for_character(character_id)`
- `find_recent_sessions_for_character(character_id, limit)`
- `lookup_reconnect_candidate(shard_id, account_id | agent_profile_id, character_id)`

## 8. Lifecycle Operations

### 8.1 Create

Create operations should allocate durable ids and write canonical records:

- account creation writes one account record
- profile creation writes one profile record
- character creation writes one character record plus any initial binding fields
- session attach writes one transient session record

Create must not silently populate world state. World spawn remains a separate
world-layer operation.

### 8.2 Bind / Unbind

Binding operations should be explicit:

- account to character
- agent profile to character
- system identity to character
- session to character

Unbind should not delete history. It should either:

- set the binding field to `null`, or
- append a new binding generation while retaining prior audit state

### 8.3 Lookup

Lookups should support three practical paths:

- operational lookup for attach/reconnect
- world-facing lookup by `character_id`
- export/reporting lookup by account/profile/character class

Avoid large “find anything by anything” APIs. Stable targeted reads are easier
to audit and cache.

### 8.4 Activate / Deactivate

Activation state should be owned durably by the registry for:

- account eligibility
- profile eligibility
- character active/retired state

Session activity remains transient and separate.

Example distinction:

- character `active` means it is a valid durable shard participant
- session `active` means there is a live attachment right now

### 8.5 Reconnect / Recover

Recovery lookups should consult:

- registry durable bindings
- recent session record
- shard recovery manifest / checkpoint generation boundary

They should not guess based only on world entity presence.

The minimal recovery question is:

- can this principal/profile reattach to this `character_id` on this shard at
  this recovery generation

## 9. Durability and Atomicity Expectations

### 9.1 Durable Registry Writes

The following should be durable before success is returned:

- account/profile/character create
- account/profile/character activate/deactivate
- durable binding changes

These should align with shard checkpoint/event durability rules and must not
depend on transient session memory alone.

### 9.2 Session Writes

Session attach/detach may be persisted in a lighter-weight way, but they still
need explicit ordering relative to recovery generation boundaries.

The system should be able to answer after restart:

- which durable identities existed
- which character bindings were authoritative
- which sessions were active, detached cleanly, or became stale

### 9.3 Atomicity Rules

The safe initial rules are:

- one registry operation updates one record family atomically
- multi-record operations use explicit ordered steps with generation markers
- success is reported only after the durable step is committed

Examples:

- `create_character` with initial profile binding should commit as one durable
  character record write
- `attach_session` should not mutate character ownership
- world spawn after character creation should be a separate operation so a
  partial world failure does not corrupt registry truth

## 10. Session Attach / Detach Example

Illustrative attach flow:

1. ingress authenticates `agent_profile_id = profile_mock_llm_v1`
2. registry resolves `character_id = char_market_runner_01` on
   `shard_dev_alpha`
3. registry verifies character status is `active`
4. registry writes session record:
   - `session_id = sess_00041`
   - `presence_id = pres_00041`
   - `character_id = char_market_runner_01`
   - `status = active`
5. world/session layer receives the durable mapping and begins live control

Detach flow:

1. session manager requests `detach_session(sess_00041, "disconnect")`
2. registry marks session `inactive`
3. registry records `detached_at_generation`
4. character remains durable and active
5. world may continue to represent the character as offline, but ownership
   binding does not disappear

## 11. Recovery Lookup Example After Disconnect

Illustrative recovery flow:

1. shard restarts at recovery generation `88`
2. reconnect arrives for `agent_profile_id = profile_mock_llm_v1`
3. registry performs:
   - `lookup_reconnect_candidate(shard_dev_alpha, profile_mock_llm_v1, char_market_runner_01)`
4. registry verifies:
   - character exists and is active
   - character binding still points to `profile_mock_llm_v1`
   - no newer exclusive session has superseded the prior one
   - the last attached session was not explicitly invalidated
5. recovery layer checks shard recovery manifest generation `88`
6. if both registry and recovery generation are coherent, a new session record
   is attached and control resumes

If any of those checks fail, the reconnect must be rejected as non-resumable
rather than guessed.

## 12. Observability and Export Implications

The registry should support explicit read models for exports and observer views.

Minimum read bundles should include:

- account summary bundle
- profile summary bundle
- character summary bundle
- shard participation bundle
- encounter identity bundle

Those bundles should be sufficient to populate:

- shard export identity summaries
- encounter records
- season/standing rows
- observer rollup and profile panels

The observer/export layer should see stable derived fields such as:

- `identity_type`
- `identity_value`
- `account_id`
- `agent_profile_id`
- `character_id`
- `identity_class`
- display labels

This keeps export logic from scraping low-level storage internals.

## 13. Benchmark Mode Boundary

Benchmark mode should continue exposing:

- `actor_id`
- `external_agent_label`
- `external_agent_profile_id`

The registry API should bridge to those fields only when useful:

- built-in benchmark actor ids may resolve to built-in profile records
- external benchmark profiles may map to `agent_profile_id`
- benchmark exports remain run-scoped and should not suddenly expose
  shard-specific registry internals

This keeps benchmark artifacts stable while allowing shared implementation under
the hood later.

## 14. Phased Implementation Path

### Phase 1: In-Process Registry Module

Add an in-process shard identity registry module with:

- account/profile/character/session record types
- deterministic id handling
- explicit bind/unbind and attach/detach APIs
- export-facing read bundles

No distributed service is required.

### Phase 2: Recovery-Aware Attach Semantics

Connect session attach/reconnect logic to:

- shard recovery generation
- latest valid checkpoint/segment boundary
- stale session invalidation rules

### Phase 3: Export and Observer Integration

Expose registry-derived summaries into shard export payloads:

- identity summaries
- participation summaries
- encounter participant bundles
- season/standing-ready profile rows

### Phase 4: Standing / Eligibility Integration

Make rated encounter pipelines consume stable registry bundles rather than
ad-hoc actor labels.

### Phase 5: Storage Hardening

Decide whether the registry remains:

- embedded in shard persistence, or
- split into a separate durable store with the same API contract

That storage decision should not change the outward registry API semantics.

## 15. Final Design Rule

If a subsystem needs to answer “who is this durable participant and what stable
identity bundle does it belong to,” it should read from the shard identity
registry.

If it needs to answer “where is the character and what is happening in the
world,” it should read world state.

If it needs to answer “is the live connection healthy right now,” it should
read session/presence state.

Keeping those boundaries explicit is the simplest way to grow shard/world mode
without losing benchmark-grade auditability.
