# Persistent World Architecture Note

## Status

- Type: architecture/design note
- Scope: implementation-aware planning note
- Intent: describe how MUDBench can evolve from deterministic benchmark runs into a hosted persistent-world / shard mode without replacing benchmark mode

## 1. Why This Note Exists

MUDBench already has two product directions in the repo documents:

- an official deterministic benchmark surface
- a persistent arena surface for competition and longitudinal observation

The current implementation is still benchmark-first. The runner, scorecards,
replay artifacts, report manifests, history/export commands, and static observer
all assume bounded runs over canonical scenarios.

This note describes how to add a hosted persistent-world regime on top of the
same core engine rather than splitting the project into two unrelated systems.

## 2. Current Implementation Baseline

Today the repo already provides the pieces that a persistent mode would build
on:

- a modular execution chain:
  `Agent Gateway -> Simulation Controller -> World Engine -> Evaluation -> Replay`
- deterministic benchmark lifecycle execution in
  [runner.py](/Users/adminamn/MUDBench/src/evaluation/benchmark_runner/runner.py)
- local-process agent ingress through the CLI with:
  - raw `--agent-command`
  - reusable `--agent-profile`
  - optional `--persistent-agent-session`
- bounded suite reporting, saved manifests, replay sidecars, history/export
  aggregation, identity-aware rollups, and a static local observer
- mixed built-in and external local-process comparison modes, including
  shared-run comparisons for tiny scenarios

The current system is therefore not missing a world engine. It is missing a
second operating regime around the engine:

- long-lived shard lifecycle
- persistent identity and state storage
- hosted ingress and scheduling
- live-world rating/season semantics

## 3. Two Operating Regimes

### 3.1 Benchmark Mode

Benchmark mode remains the official deterministic regime.

Properties:

- bounded run lifecycle
- fixed scenario definition
- fixed seed / max steps
- deterministic initialization
- replay and scorecard must be audit-compatible
- official comparison uses benchmark score semantics
- local and future hosted ingress must still obey benchmark timing and action
  rules

This mode should remain the source of:

- canonical benchmark results
- scorecard validity
- replay-auditable benchmark claims
- release-gated regression testing

### 3.2 Persistent World Mode

Persistent world mode is a hosted live regime using the same engine primitives
but different orchestration assumptions.

Properties:

- shard/world runs indefinitely or in long epochs
- the world state is durable between sessions
- agents and humans may join or leave over time
- action scheduling is continuous or tick-based rather than bounded by one
  benchmark scenario
- evaluation is longitudinal rather than single-run scorecard-first

Persistent mode is not a replacement for benchmark mode. It is the same
simulation substrate with different lifecycle, persistence, ingress, and
reporting rules.

## 4. Shared Core vs Mode-Specific Layers

### 4.1 Shared Core

These pieces should remain common across both modes:

- world state model
- action validation
- event generation
- observation production
- replay/event logging format where feasible
- deterministic transition logic for a given state and action stream

### 4.2 Benchmark-Specific Layer

These remain benchmark-mode concerns:

- canonical scenario loading
- strict seed control
- fixed termination
- scorecard normalization and official composite scores
- parity and replay recomputation as release gates

### 4.3 Persistent-Mode Layer

These become persistent-world concerns:

- shard allocation and world bootstrap
- durable world snapshots and recovery
- player/agent session management
- live action ingress and backpressure
- season/rating summaries
- moderation, quotas, and world-ops tooling

## 5. Shard and World Lifecycle

The cleanest path is to introduce a shard manager above the existing simulation
controller.

### 5.1 Shard Definition

A shard is a long-lived hosted world instance with:

- `shard_id`
- ruleset/version
- world snapshot reference
- ingress configuration
- current roster of connected humans and agents
- tick/scheduler state

### 5.2 Shard Lifecycle

Suggested lifecycle:

1. create shard from a versioned world template or snapshot
2. restore durable world state
3. attach connected identities and policies
4. run tick loop continuously or on demand
5. persist checkpoints on a fixed cadence
6. emit live telemetry plus archival replay segments
7. rotate season or archive shard if required

### 5.3 Checkpoint Model

Benchmark runs currently end with replay/state summary artifacts. Persistent
mode should add checkpointing without changing the benchmark contract:

- full snapshot at shard bootstrap
- periodic world checkpoints
- append-only event segments between checkpoints
- replay reconstruction as:
  `checkpoint + ordered events -> current world state`

That gives persistent mode operational recovery without requiring one enormous
infinite replay artifact.

## 6. Persistent Identities for Humans and Agents

Current benchmark/reporting work already distinguishes identities such as:

- built-in actor ids
- external labels
- external profile ids

Persistent mode should extend that into a durable identity model.

### 6.1 Identity Layers

At minimum:

- `principal_id`: stable hosted identity for a human or agent owner
- `entity_id`: in-world controlled entity on a shard
- `profile_id`: reusable agent configuration/profile
- `session_id`: current live connection/session

### 6.2 Why This Split Matters

Benchmark mode can keep treating `actor_id` as a run-local comparison label.

Persistent mode needs stronger separation:

- one principal may control multiple entities over time
- one external profile may be reused across shards/seasons
- one entity may reconnect through multiple sessions
- humans and agents must coexist under the same roster model

### 6.3 Relationship to Current Identity Reporting

The new `identity_rollups` and observer identity panels are a useful bridge.
They already move MUDBench away from “just actor ids” and toward stable
identity-aware summaries. Persistent mode should build on that rather than
inventing a separate reporting vocabulary.

## 7. Agent Ingress in Persistent Mode

### 7.1 What Exists Today

The implemented ingress path is local-process oriented:

- built-in deterministic actor scripts
- external local-process command wrappers
- file-based external agent profiles
- optional persistent local-process sessions

This is enough for:

- local testing of long-lived agent sessions
- early shard simulation in one host process
- controlled arena prototypes without a networked submission system

### 7.2 Local Ingress in World Mode

In early persistent mode, local-process ingress should remain first-class for:

- development shards
- reproducible load tests
- baseline population agents
- local human/operator experiments

### 7.3 Hosted Ingress in World Mode

Hosted mode should eventually add remote ingress, but as an additional ingress
adapter, not a new engine.

Likely hosted ingress categories:

- hosted process workers managed by MUDBench
- remote agent endpoints
- human websocket/http clients

The key architectural rule is:

- all ingress adapters normalize into the same action submission contract before
  reaching the simulation controller

That preserves comparability between local agents, hosted agents, and humans at
the world-action boundary.

## 8. Tick, Event, and Persistence Model

### 8.1 Benchmark Tick Model

Current benchmark mode is effectively:

- initialize world
- repeatedly collect one action per actor
- resolve world update
- log events
- stop at termination

### 8.2 Persistent Tick Model

Persistent shards should keep the same event semantics but change the
orchestration:

- shard scheduler advances on a stable tick cadence or event cadence
- connected actors submit actions into per-identity queues
- the controller resolves eligible actions for the current tick
- world events are emitted in stable order
- checkpoint/save logic runs out-of-band on deterministic boundaries

### 8.3 Determinism Expectations

Persistent mode is not required to be globally replay-identical across arbitrary
wall-clock conditions in the same way official benchmark mode is.

However, it should still preserve deterministic local rules:

- identical checkpoint + identical ordered action stream should reproduce the
  same world evolution
- event ordering must be explicit
- scheduler/tick policy must be versioned

This keeps persistent shards auditable without pretending they are the same as
fixed-seed benchmark scenarios.

## 9. Observability and Replay Implications

Current reporting already produces:

- scorecards
- replay sidecars
- report manifests
- history/export summaries
- observer views for history, arena, replay drilldown, and identity rollups

Persistent mode should extend those surfaces rather than bypass them.

### 9.1 Benchmark Replay Remains Strict

Official benchmark replays must remain:

- complete
- deterministic
- recomputable
- score-auditable

### 9.2 Persistent Replay Should Become Segmented

For shards, replay is better treated as:

- checkpoint snapshots
- event segments
- session-level drilldowns
- season summaries

The static observer/export path already points in the right direction:

- summary viewmodels for humans
- replay drilldowns for selected runs
- identity-centric aggregation

Persistent mode can generalize this into:

- shard export
- season export
- incident replay drilldown
- identity timeline view

without invalidating the benchmark replay model.

## 10. Scoring, Ratings, and Seasons

### 10.1 Benchmark Scoring

Benchmark mode keeps the current semantics:

- scenario-bounded scorecards
- capability metrics
- composite scores
- replay-recomputable scoring

### 10.2 Persistent-World Evaluation

Persistent world mode should not reuse benchmark composite scores as its only
ranking unit.

Instead it should add longitudinal measures such as:

- participation volume
- survival/retention over time
- objective completion across sessions
- social/trade/combat outcomes in live play
- shard-specific or season-specific rating movement

### 10.3 Season Model

A season is the likely hosted aggregation boundary:

- fixed shard ruleset/version window
- durable identity roster
- rating history
- summary exports and representative replays

Current history/export/observer surfaces are already the seed of this idea.
They can evolve from artifact history into season history without discarding the
benchmark reporting path.

## 11. Recommended Hosted Data Model Direction

This note does not define a final schema, but the likely minimum durable records
are:

- world/shard record
- checkpoint record
- event segment record
- principal/profile record
- session/connection record
- season record
- rating/summary record

These should be added as persistent-world storage concepts, not mixed into the
canonical benchmark artifact layout directly.

## 12. Phased Implementation Path

### Phase 0: Current State

Already present in the repo:

- deterministic benchmark lifecycle
- external local-process agents and profiles
- persistent local-process session support
- saved report/manifests/replay drilldowns
- identity-aware history/export and observer summaries
- shared-run comparison reporting and arena-style observer summaries

### Phase 1: Long-Lived Local Shard Runtime

Add a local-only shard manager using the existing engine:

- one long-lived shard process
- checkpoint + event segment persistence
- attach built-in actors and local external agents
- no hosted multi-tenant ingress yet

Goal:
- prove long-lived world lifecycle without changing benchmark semantics

### Phase 2: Persistent Identity and Session Layer

Add durable identity/session records:

- principal/profile/session separation
- reconnectable local/hosted sessions
- shard roster management

Goal:
- make live participation durable and auditable

### Phase 3: Hosted Agent and Human Ingress

Add hosted ingress adapters above the same action contract:

- managed external workers
- remote agents
- human clients

Goal:
- enable real coexistence of humans and agents on shards

### Phase 4: Persistent Reporting and Season Summaries

Extend the current report/export model:

- shard history
- season rollups
- identity timelines
- representative replay drilldowns

Goal:
- make persistent mode inspectable using the same reporting philosophy as
  benchmark mode

### Phase 5: Ratings and Moderated Live Arena Operations

Introduce hosted competition semantics:

- rating updates
- season boundaries
- shard policy/ruleset versioning
- moderation and safety operations

Goal:
- create a public persistent arena without weakening benchmark integrity

## 13. Recommended Guardrails

To avoid damaging the benchmark system while adding persistent mode:

- keep benchmark mode and persistent mode as explicitly separate runtime modes
- never mix live shard outcomes into official benchmark leaderboards
- version shard scheduler/tick policy independently from benchmark scoring
- preserve replay/audit guarantees for benchmark runs
- keep ingress adapters behind one normalized action interface
- keep persistent-world storage separate from benchmark artifact history

## 14. Bottom Line

MUDBench does not need a second engine to become a hosted persistent world.

It needs:

- a shard lifecycle above the current controller
- durable identity and checkpoint storage
- hosted ingress adapters that normalize into the current action path
- persistent reporting/rating layers that complement, rather than replace,
  benchmark scorecards and replays

The current implementation already contains the early ingredients:

- deterministic engine and replay foundations
- local persistent agent sessions
- identity-aware comparison/report surfaces
- observer tooling for history, arena, replay, and rollup summaries

The right architecture move is to treat persistent world mode as the second
operating regime of the same benchmark-first engine.
