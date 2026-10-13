# Shard Persistence and Storage Design Note

## Status

- Type: architecture/design note
- Scope: implementation-aware storage note
- Intent: define shard persistence, checkpoint boundaries, and event-segment storage for persistent-world mode without changing benchmark-mode storage semantics

## 1. Why This Note Exists

The current MUDBench implementation is benchmark-first:

- deterministic runs are bounded by scenario, seed, and max steps
- replay artifacts record ordered step events plus terminal summaries
- scorecards, manifests, replay sidecars, reports history/export, and the local observer all assume finite run artifacts

The new persistent-world architecture note establishes that hosted shard mode
should be a second operating regime of the same engine. This document narrows
that idea to one concrete question:

- what data remains benchmark-mode storage
- what data becomes persistent shard storage
- where the checkpoint boundary sits relative to append-only event segments

The goal is not to redesign the current replay schema broadly. The goal is to
define a storage seam that lets persistent shards use the same deterministic
engine and replay philosophy without pretending that an infinite world can be
stored like a single benchmark replay file.

## 2. Current Implementation Baseline

Today the repo already has the core ingredients this design should preserve:

- deterministic world evolution under the benchmark runner
- replay logs with ordered events and terminal summaries
- replay parity verification and terminal state hashing
- state snapshot events already visible in runtime replay output
- saved suite report manifests, replay sidecars, history/export aggregation, and
  observer drilldowns
- identity-aware reporting for built-in actors, external labels, and external
  profile ids

That means persistent shard storage should not invent a separate truth model.
It should reuse the same causality model:

- a world state
- an ordered stream of accepted actions and resulting events
- replay-visible summaries and integrity checks

## 3. Two Storage Regimes

### 3.1 Benchmark-Mode Storage

Benchmark mode remains the official deterministic storage regime.

Benchmark storage should keep using bounded run artifacts such as:

- replay artifact for one run
- parity artifact for one run
- scorecard for one run
- report/manifests/export entries derived from those runs

Properties:

- immutable once written
- scoped to one run or one saved suite artifact
- seed/scenario/version aware
- suitable for strict audit and release gating

Benchmark storage should remain the only source of official benchmark claims.

### 3.2 Persistent Shard Storage

Persistent shard storage is a live-world regime layered above the same engine.

It should store:

- durable shard metadata
- durable world checkpoints
- append-only event segments between checkpoints
- identity/session records needed to reconnect actors over time
- shard-level summaries and season rollups derived from stored state and events

Properties:

- durable across sessions
- segmented instead of one infinite replay file
- optimized for recovery, drilldown, and longitudinal inspection
- still audit-oriented, but not required to act like one finite benchmark run

## 4. Shard State Model

The minimum persistent shard state should be modeled as four layers.

### 4.1 Shard Metadata

Stable shard configuration:

- `shard_id`
- `world_ruleset_version`
- `benchmark_engine_version`
- `scheduler_policy_version`
- `created_from_snapshot_ref` or template ref
- `season_id` if applicable
- current checkpoint pointer

This is the long-lived identity of the shard.

### 4.2 Durable World State

The world state that must survive process restarts:

- room and map state
- item ownership and location
- quest/objective progress that is intended to persist
- entity attributes that affect future world evolution
- inventory and equipped items
- entity location / shard placement
- world clocks or counters that matter to deterministic progression

### 4.3 Durable Identity Attachments

Persistent linkage between principals/profiles and in-world entities:

- `principal_id`
- `profile_id` where applicable
- `entity_id`
- roster membership on shard
- durable permissions or roles if the shard model needs them later

### 4.4 Ephemeral Session State

Connection-local state that should not be the primary durable truth:

- live connection/session handles
- in-flight ingress queues
- short-lived rate-limit windows
- transient session health flags

These may be recoverable or reconstructable, but they should not be the only
source of truth for world continuity.

## 5. Checkpoint Boundary

The core storage design is:

`checkpoint + ordered event segments => shard reconstruction`

### 5.1 What Must Be Inside a Checkpoint

A checkpoint should contain enough state to resume the shard without replaying
the entire lifetime of the world.

Minimum checkpoint contents:

- shard metadata snapshot
- deterministic world state snapshot
- entity roster with location and core attributes
- inventory ownership
- persistent quest/objective state
- scheduler/tick cursor
- last fully committed event offset / segment id
- integrity hashes for checkpoint content

### 5.2 What Must Not Be the Only Thing in a Checkpoint

The checkpoint should not be the sole record of causal history.

Do not rely on checkpoints alone for:

- forensic debugging
- representative replay drilldown
- event-by-event reconstruction
- explaining why the shard reached its current state

That causal trail belongs in event segments.

## 6. Event Segment Boundary

Everything after a checkpoint and before the next checkpoint belongs in
append-only event segments.

### 6.1 Event Segment Contents

An event segment should record ordered accepted world progression data such as:

- accepted action submissions
- resolved world events
- observation-related events where needed for audit/debugging
- evaluation or summary signals emitted during shard operation
- state snapshot events on configured boundaries

The persistent-world segment format should stay conceptually aligned with the
current replay philosophy:

- explicit ordering
- deterministic reconstruction from state + events
- enough visibility to inspect what happened and why

### 6.2 Event Segment Metadata

Each segment should carry:

- `shard_id`
- `segment_id`
- `start_tick`
- `end_tick`
- `starting_checkpoint_id`
- prior segment reference if segmented sequentially
- schema version
- integrity hash for the segment payload

### 6.3 Segment Closure Rules

A segment should be closed deterministically on clear boundaries, for example:

- fixed tick count
- fixed event count
- shard shutdown
- checkpoint completion
- season rollover

Avoid time-only rotation as the primary boundary if it would make replay slicing
harder to reason about.

## 7. Checkpoint vs Event Examples

### 7.1 Durable Checkpoint Example

Example state that belongs in a checkpoint:

- `agent-a` is in room `market`
- `agent-a` inventory contains `artifact`
- `agent-b` is in room `vault`
- `hidden-key` is no longer on the floor because it was already picked up
- quest state shows `trade-complete = true`
- shard scheduler cursor is at tick `18420`

This is durable world truth.

### 7.2 Event Segment Example

Example entries that belong in the append-only log after that checkpoint:

1. tick `18421`: `agent-b` submits `move west`
2. tick `18421`: world emits `movement` from `vault` to `hall`
3. tick `18422`: `agent-a` submits `give artifact trader`
4. tick `18422`: world emits `trade_completed`
5. tick `18422`: evaluation emits `social.trade.completed = 1`
6. tick `18425`: replay-visible `state_snapshot` emits compact terminal state for
   the segment boundary

These entries explain causality. They should not be collapsed into a checkpoint
only.

### 7.3 Transient Event Example

A transient operational signal may be recorded in telemetry but not promoted to
durable world truth, for example:

- websocket reconnect warning
- temporary local wrapper heartbeat miss that is retried

That may matter for ops, but it should not necessarily change the reconstructed
world state unless it materially affects accepted actions or shard progression.

## 8. Checkpoint Cadence and Recovery Model

### 8.1 Checkpoint Cadence

The initial cadence should be explicit and conservative rather than clever.

Recommended starting policy:

- checkpoint at shard bootstrap
- checkpoint after a fixed number of ticks or accepted actions
- checkpoint on graceful shard shutdown
- checkpoint before season rollover or world version migration

The cadence should be versioned as part of shard policy so replay/debugging can
explain how a shard was persisted.

### 8.2 Recovery Model

On recovery:

1. load the most recent valid checkpoint
2. identify all subsequent committed event segments
3. replay segments in stable order
4. rebuild live in-memory state
5. resume ingress only after state integrity is confirmed

This is intentionally the same reconstruction idea as benchmark replay, just
segmented over a longer lifespan.

### 8.3 Failure Handling

Failure cases to handle explicitly:

- partial checkpoint write
- missing segment after checkpoint
- corrupted segment hash
- segment present but schema-incompatible with shard version
- shard crash between accepted action and durable segment commit

Recommended rule:

- checkpoints are authoritative only once atomically committed
- event segments are authoritative only once atomically closed/committed
- on ambiguity, recover to the last known valid checkpoint + fully committed
  segments only

This favors auditability over pretending uncertain writes are valid.

## 9. Identity, Inventory, and Location Durability Boundaries

Persistent mode needs clear rules about what is durable world truth.

### 9.1 Durable

These should persist across sessions/checkpoints:

- entity location
- room occupancy if it affects gameplay
- inventory ownership
- item existence/destruction state
- persistent quest flags
- shard roster membership
- profile/entity bindings relevant to control of in-world entities

### 9.2 Transient

These should remain transient or reconstructable unless promoted by future
policy:

- live socket/process handles
- in-flight observation payloads not yet accepted into causal history
- local queue depth and timeout counters
- momentary UI/session presence signals

### 9.3 Borderline Cases

Some state should be persisted only if it has world consequences, for example:

- a temporary combat status effect may be durable if it changes future combat
- a chat typing indicator is not durable
- a pending accepted trade offer may be durable if the world model treats it as
  an outstanding world commitment

The rule is simple:

- if future deterministic world evolution depends on it, checkpoint it or derive
  it from durable events
- if not, keep it operational/ephemeral

## 10. Transient vs Durable World Events

### 10.1 Durable World Events

These belong in the causal event log:

- movement
- item pickup/drop/transfer
- combat outcome
- unlock/trade/quest completion
- entity creation/destruction
- accepted action plus resulting world transition

### 10.2 Transient Operational Events

These can live in telemetry/ops logs instead of the shard event log:

- process start/stop diagnostics
- agent wrapper stderr lines used for debugging
- transient network retries
- local observer interaction events

### 10.3 Replay-Visible Summaries

Persistent mode should still emit replay-visible summary events at useful
boundaries:

- periodic `state_snapshot`
- segment-end summary
- season-end summary

This matches the current benchmark and export surfaces, where state snapshots,
score summaries, and replay drilldowns help explain outcomes without replacing
the causal log.

## 11. Replay Implications for Persistent Shards

### 11.1 Benchmark Replay Still Stays Strict

Official benchmark replay remains:

- one bounded run
- complete causal trace
- parity-checkable
- score-auditable

Nothing in shard storage should weaken that contract.

### 11.2 Persistent Replay Becomes Layered

Persistent replay should be treated as:

- checkpoint for fast resume
- segment list for causal continuation
- drilldown projections for selected windows
- rollups for shard/season summaries

This is already consistent with the current reporting direction:

- saved manifests and replay sidecars
- replay drilldowns in `reports export`
- observer replay inspection over exported drilldowns

### 11.3 Replay Windowing

The default replay unit for persistent inspection should be a window, not “the
entire shard forever.”

Useful windows:

- one segment
- one incident window
- one session window
- one scenario-like encounter window inside a shard

That keeps replay usable and compatible with the existing observer/reporting
style.

## 12. Observability and Debugging Implications

The current system already exposes:

- manifests
- history/export
- replay drilldowns
- identity rollups
- arena and match summaries in the observer

Persistent shards should extend that model by storing enough references to map:

- shard -> checkpoint chain
- shard -> event segment chain
- identity -> participation windows
- incident/encounter -> replay drilldown window

Operational debugging should be able to answer:

- what was the last valid checkpoint
- which event segment caused divergence or failure
- which identities were involved
- what final state summary was visible at the segment boundary

That means persistence storage should be designed for both recovery and human
inspection.

## 13. Migration Path from Current Benchmark Artifacts

The migration should be additive, not destructive.

### 13.1 What Reuses Existing Concepts Directly

Current concepts that carry forward well:

- replay-visible state snapshots
- ordered step/event thinking
- parity/state hashes
- manifest-driven artifact discovery
- history/export/observer drilldown surfaces
- identity-aware reporting

### 13.2 What Stays Benchmark-Only

These should remain benchmark-specific artifacts:

- single-run scorecards as official benchmark output
- finite replay artifact as the canonical official replay
- benchmark suite manifest/report pair

### 13.3 What Persistent Mode Adds

Persistent mode should add new artifact classes rather than overloading old
ones:

- shard checkpoint artifact
- shard event segment artifact
- shard recovery manifest
- shard/season summary export

### 13.4 Bridge Strategy

A practical bridge from today’s system:

1. keep current benchmark replay/state artifact generation unchanged
2. add shard checkpoint artifacts that reuse current state snapshot discipline
3. add append-only segment artifacts using current replay event ordering ideas
4. expose shard drilldowns through exports similarly to current replay
   drilldowns
5. add observer views over those exported shard windows

## 14. Phased Implementation Path

### Phase 0: Current Benchmark-First State

Already present:

- deterministic run replays
- parity/state hashing
- saved manifests and replay sidecars
- report history/export and observer drilldowns

### Phase 1: Local Shard Persistence Prototype

Add:

- one shard metadata record
- one checkpoint artifact format
- one append-only segment format
- recovery from latest checkpoint plus segments

Goal:
- prove that the same engine can resume persistent state safely

### Phase 2: Durable Identity and Inventory Boundaries

Add:

- explicit durable principal/profile/entity bindings
- inventory/location durability rules in shard checkpoints
- session records separate from durable identity/world state

Goal:
- make reconnect and longitudinal analysis trustworthy

### Phase 3: Export and Observer Integration

Add:

- shard-aware manifest discovery
- replay drilldowns over segment windows
- identity and season summaries over shard artifacts

Goal:
- make persistent mode inspectable with the same philosophy as benchmark mode

### Phase 4: Hosted Shard Operations

Add:

- hosted ingress adapters
- checkpoint scheduling policy
- failure recovery automation
- season rotation and archival policy

Goal:
- operate live shards without weakening auditability

## 15. Recommended Guardrails

To preserve benchmark integrity while introducing shard storage:

- do not replace bounded benchmark replay artifacts with shard storage
- do not mix shard summary data into official benchmark score comparisons
- keep checkpoint data and segment data versioned separately
- require integrity hashes for checkpoints and segments
- keep shard recovery append-only from the last valid committed boundary
- keep durable world truth separate from operational telemetry noise

## 16. Bottom Line

The right storage model for persistent-world MUDBench is not “one giant replay
file.”

It is:

- benchmark mode: immutable bounded run artifacts
- shard mode: durable checkpoints plus append-only event segments

That model fits the current benchmark-first engine because it preserves the same
core assumptions:

- deterministic state transition logic
- explicit event ordering
- replay/audit visibility
- human-inspectable summaries

The storage change is therefore architectural, not philosophical: persistent
mode adds long-lived state boundaries around the current engine rather than
replacing how MUDBench establishes truth.
