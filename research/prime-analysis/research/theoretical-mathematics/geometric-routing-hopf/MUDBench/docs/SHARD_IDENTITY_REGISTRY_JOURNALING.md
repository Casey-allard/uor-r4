# Shard Identity Registry Journaling Design Note

## Status

- Type: implementation-oriented design/spec note
- Scope: shard identity registry generation/version semantics and mutation
  journaling
- Intent: define how registry-owned identity records advance over time and how
  append-only mutation evidence should be stored so durability and recovery
  guarantees can be implemented concretely

## 1. Why This Note Exists

The shard identity registry API note defines:

- durable registry-owned record families
- explicit read/write surfaces
- lifecycle operations such as create, bind, activate, attach, and recover

What it does not yet define is the mutation discipline underneath those calls:

- how record versions advance
- how registry writes are journaled
- how duplicate or replayed mutations are detected
- how registry state interacts with checkpoints and recovery manifests

MUDBench already has a strong benchmark-first integrity model:

- bounded replay artifacts
- parity and terminal-state hashes
- append-only evidence plus compact current summaries

Persistent shard mode should keep the same discipline. The identity registry
should therefore use:

- current-state records for fast reads
- an append-only mutation journal for durable write evidence

## 2. Design Goals

The registry journaling design should:

- provide explicit record version semantics for all registry-owned records
- provide an append-only mutation trail for audit and recovery
- keep ordering deterministic
- support idempotent retries after crashes or transport failures
- align with shard checkpoint and recovery-manifest boundaries
- be simple enough for a first in-process implementation

## 3. Required Distinctions

This note uses four separate concepts that must not be conflated.

### 3.1 Record Version Semantics

Record version semantics answer:

- what version of one account/profile/character/session record is current
- whether an update is newer than the previously committed record

This is current-state identity truth.

### 3.2 Mutation Journal Semantics

Mutation journal semantics answer:

- what write intent was committed
- in what order did registry mutations occur
- whether a repeated mutation is a duplicate of an already committed operation

This is append-only evidence and recovery aid.

### 3.3 Checkpoint Interaction

Checkpoint interaction answers:

- what registry state is captured in a shard checkpoint
- what journal suffix must be replayed after the checkpoint

This is reconstruction and resumability.

### 3.4 Recovery / Replay Implications

Recovery implications answer:

- can the registry be reconstructed safely after interruption
- can stale or duplicate writes be ignored deterministically
- can export and debugging tools explain identity transitions over time

## 4. Record Version Fields

Every registry-owned durable record should carry explicit version fields.

### 4.1 Minimum Fields

Suggested minimum fields:

- `record_version`
- `created_generation`
- `updated_generation`
- `last_mutation_id`
- `status`

Purpose:

- `record_version`: monotonic per-record version
- `created_generation`: shard-wide registry generation at record creation
- `updated_generation`: shard-wide registry generation of the latest committed
  change
- `last_mutation_id`: journal entry id that produced the current version
- `status`: current lifecycle state

### 4.2 Recommended Semantics

`record_version` should be:

- scoped to one record id
- monotonic
- incremented only on durable committed mutations

`updated_generation` should be:

- scoped to the shard registry as a whole
- monotonic across all record families
- the primary coarse ordering field for cross-record inspection

The design rule is:

- use `record_version` for one-record concurrency and staleness checks
- use `updated_generation` for shard-wide ordering and checkpoint alignment

### 4.3 Session Record Versioning

Session/presence records are transient, but they should still carry:

- `record_version`
- `attached_generation`
- `detached_generation`
- `last_mutation_id`

This allows reconnect and stale-session handling to remain deterministic even
though the record family is not the primary durable identity truth.

## 5. Versioned Character Record Example

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
  "record_version": 7,
  "created_generation": 18,
  "updated_generation": 44,
  "last_mutation_id": "mut_00000044",
  "metadata": {
    "benchmark_bridge_actor_id": "agent-a"
  }
}
```

Interpretation:

- the character has existed since generation `18`
- the current materialized state was last changed at generation `44`
- the seventh committed version came from mutation `mut_00000044`

## 6. Mutation Journal Purpose

The mutation journal is the append-only durable record of committed registry
changes.

It exists to provide:

- auditability of identity changes
- deterministic replay of registry state after checkpoints
- duplicate/retry detection
- debugging of ownership/profile/session transitions

It is not intended to:

- replace materialized current-state records
- store world/entity state
- become a generic event bus for every subsystem
- replace replay/event-segment storage for world evolution

## 7. Journal Entry Types

The initial journal can stay small and still cover the registry API.

### 7.1 Core Entry Families

Minimum entry families:

- `account_created`
- `account_metadata_updated`
- `account_activated`
- `account_deactivated`
- `agent_profile_created`
- `agent_profile_metadata_updated`
- `agent_profile_activated`
- `agent_profile_deactivated`
- `character_created`
- `character_owner_bound`
- `character_profile_bound`
- `character_system_identity_bound`
- `character_activated`
- `character_deactivated`
- `session_attached`
- `session_detached`
- `session_marked_active`
- `session_marked_inactive`
- `reconnect_token_bound`

### 7.2 Journal Entry Shape

Suggested fields:

- `mutation_id`
- `shard_id`
- `mutation_generation`
- `mutation_type`
- `record_type`
- `record_id`
- `expected_record_version`
- `resulting_record_version`
- `idempotency_key`
- `payload`
- `committed_checkpoint_generation` if already checkpointed later

## 8. Append-Only Mutation Journal Entry Example

```json
{
  "mutation_id": "mut_00000044",
  "shard_id": "shard_dev_alpha",
  "mutation_generation": 44,
  "mutation_type": "character_profile_bound",
  "record_type": "character",
  "record_id": "char_market_runner_01",
  "expected_record_version": 6,
  "resulting_record_version": 7,
  "idempotency_key": "bind-profile:char_market_runner_01:profile_mock_llm_v1:req_0009",
  "payload": {
    "controller_agent_profile_id": "profile_mock_llm_v1"
  }
}
```

## 9. Ordering Rules

### 9.1 Generation Ordering

Every committed journal entry must receive one monotonic
`mutation_generation` within a shard registry.

That generation should be:

- deterministic
- gap-tolerant only if explicitly documented later
- stable enough to be referenced by checkpoints and recovery logic

The simplest first implementation is:

- one shard-local monotonic counter

### 9.2 Record Ordering

For one record:

- journal entries must be applied in ascending `mutation_generation`
- `expected_record_version` must match the materialized record before apply
- `resulting_record_version` must be exactly prior version plus one

### 9.3 Cross-Record Ordering

For multi-step flows that touch more than one record family, the system should
prefer explicit ordered steps rather than hidden implicit fan-out.

Example:

1. `character_created`
2. `session_attached`

not:

- one opaque write that mutates unrelated record families without explicit
  journal evidence

## 10. Atomicity Expectations

### 10.1 Single Mutation Atomicity

One committed journal mutation should mean:

- the journal entry is durably appended
- the corresponding materialized record update is durably applied
- both succeed or neither is treated as committed

The first in-process implementation may use a simple atomic write strategy, but
the semantic contract should already be:

- no committed journal entry without the matching record state
- no durable record state that cannot be explained by a committed journal entry

### 10.2 Multi-Step Operation Atomicity

Some higher-level workflows are composed of multiple mutations.

Example:

- create character
- attach session

These should be modeled as:

- multiple committed mutations with deterministic order

not as one uninspectable super-mutation.

Recovery may therefore stop after a valid earlier step without pretending a
later step happened.

## 11. Idempotency Expectations

Every mutating API call should carry or derive an `idempotency_key`.

The registry should treat a second mutation attempt as equivalent to a prior
committed mutation only when:

- `record_type`
- `record_id`
- `mutation_type`
- `idempotency_key`

all match the prior committed entry and the payload is semantically identical.

If the key matches but the payload differs, the mutation should be rejected as
ambiguous rather than guessed.

## 12. Duplicate / Replayed Mutation Handling

Duplicate or replayed mutation attempts are expected during:

- process retries
- reconnect retries
- crash recovery
- repeated ingress submissions

The safe handling rules are:

- if the same `idempotency_key` already committed with the same payload, return
  the existing result without appending a new mutation
- if the same mutation arrives after later record versions already exist, reject
  it as stale
- if ordering evidence is incomplete, prefer non-commit over speculative commit

## 13. Duplicate / Replayed Mutation Recovery Example

Illustrative case:

1. `session_attached` for `sess_00041` commits as `mut_00000045`
2. process crashes before caller receives success
3. caller retries `attach_session` with the same `idempotency_key`
4. registry lookup finds `mut_00000045`
5. registry returns:
   - existing `session_id`
   - existing `mutation_id`
   - no new mutation appended

If the retried payload instead pointed to a different `character_id`, the
registry must reject it as a conflicting replay rather than silently rebind the
session.

## 14. Checkpoint Interaction

The registry journal should integrate with shard checkpoints the same way event
segments do for world state:

- checkpoint stores a current registry snapshot boundary
- journal stores the ordered suffix after that boundary

### 14.1 What the Checkpoint Captures

A shard checkpoint should capture materialized registry state such as:

- current account/profile/character records
- current active or recently relevant session records as policy allows
- latest committed `mutation_generation`
- latest committed `mutation_id`
- integrity summary for the registry snapshot

### 14.2 What the Journal Retains

The journal retains:

- ordered mutations after the checkpoint boundary
- enough information to explain state transitions
- enough information to rebuild the latest registry state if replay is needed

### 14.3 Boundary Rule

The checkpoint is the compact current-state snapshot.

The journal is the append-only evidence after that snapshot.

Neither should replace the other.

## 15. Successful Mutation + Checkpoint Interaction Example

Illustrative sequence:

1. checkpoint `cp_0008` captures registry through `mutation_generation = 44`
2. `session_attached` commits as:
   - `mutation_id = mut_00000045`
   - `mutation_generation = 45`
3. `session_detached` commits as:
   - `mutation_id = mut_00000046`
   - `mutation_generation = 46`
4. next checkpoint `cp_0009` captures registry through generation `46`
5. recovery manifest references:
   - checkpoint `cp_0009`
   - no registry journal suffix after `46`

If recovery instead starts from `cp_0008`, it must replay journal mutations
`45` and `46` in order before the registry is considered current.

## 16. Interaction with Recovery Manifests

The shard recovery manifest should carry enough registry integrity metadata to
answer:

- which registry snapshot generation is authoritative
- whether a journal suffix exists after the checkpoint
- whether that suffix is complete and ordered

Recommended manifest-adjacent fields:

- `registry_checkpoint_generation`
- `registry_last_mutation_generation`
- `registry_last_mutation_id`
- `registry_journal_segment_refs`
- `registry_integrity_status`

The manifest does not need to inline the journal, but it must reference enough
to validate resumability.

## 17. Recovery and Replay Implications

Registry recovery should follow the same philosophy as shard recovery:

- newest safe point wins
- partial writes are not promoted to committed truth
- replay is deterministic and ordered

The minimum registry recovery algorithm is:

1. load the latest valid registry checkpoint snapshot
2. validate the referenced journal suffix
3. replay journal entries in ascending `mutation_generation`
4. verify the reconstructed registry head matches manifest metadata
5. only then expose the registry as resumable

If completeness or ordering cannot be proven, the registry should be marked
degraded or non-resumable.

## 18. Observability and Debugging Implications

The journal should support targeted debugging without becoming a general-purpose
analytics store.

Minimum useful debug reads:

- list mutations for one `record_id`
- inspect the latest mutation for one record
- inspect mutations between checkpoint generations
- trace attach/detach history for one `character_id` or `session_id`

Observer/export surfaces do not need raw journal entries by default, but the
journal should be able to produce compact derived summaries such as:

- last identity mutation generation
- last ownership/profile binding change
- reconnect or detach reason history for one character

## 19. Phased Implementation Path

### Phase 1: In-Process Append-Only Journal

Implement:

- materialized registry records
- one shard-local monotonic generation counter
- append-only in-process journal entries
- idempotency-key checking

### Phase 2: Checkpoint-Aligned Snapshotting

Add:

- registry snapshot generation capture inside shard checkpoints
- suffix replay on recovery
- integrity metadata for latest registry head

### Phase 3: Recovery Manifest Integration

Add:

- explicit registry checkpoint/journal references in the shard recovery
  manifest
- degradable resumability decisions when journal suffix integrity is incomplete

### Phase 4: Export / Debug Views

Add:

- compact mutation-derived summaries for shard export
- targeted debug tooling for registry mutation history

### Phase 5: Storage Hardening

If the registry later moves beyond an embedded in-process implementation,
preserve the same semantics:

- append-only committed mutations
- monotonic generations
- idempotent replay
- explicit checkpoint boundary

## 20. Final Design Rule

Materialized registry records are the fast current truth.

The mutation journal is the durable ordered explanation of how that truth was
reached.

Checkpoints compact the truth at a known generation.

Recovery replays only the validated suffix after that generation.

Keeping those four roles separate is the simplest way to make shard identity
durability, crash recovery, and export/debug behavior concrete without drifting
from MUDBench’s benchmark-first integrity discipline.
