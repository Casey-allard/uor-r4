# Shard Identity Registry Checkpoint Snapshot Design Note

## Status

- Type: implementation-oriented design/spec note
- Scope: shard identity registry checkpoint snapshot format and recovery
  validation
- Intent: define the concrete snapshot boundary that pairs with the registry
  mutation journal so the first in-process implementation has a precise
  resumability contract

## 1. Why This Note Exists

The registry API note defines:

- durable identity record families
- explicit lifecycle operations
- registry versus world versus session boundaries

The registry journaling note defines:

- record version semantics
- append-only mutation journal entries
- idempotency and duplicate handling
- checkpoint-aligned replay of journal suffixes

What is still needed is the concrete snapshot contract:

- what exactly belongs in a registry checkpoint snapshot
- what must remain in the journal suffix after the snapshot
- how recovery decides whether a registry state is resumable or not

This note defines that contract for the first in-process implementation.

## 2. Design Goals

The registry checkpoint format should:

- capture enough materialized identity state to avoid replaying the full journal
- align cleanly with shard checkpoint and recovery-manifest semantics
- expose deterministic validation rules
- make partial or corrupt snapshots easy to reject
- keep the first implementation small and explicit

## 3. Non-Goals

The checkpoint snapshot should not:

- replace the mutation journal
- store world/entity state such as location or inventory
- become a general export payload
- infer missing mutations from current state
- allow resumability when completeness cannot be proven

## 4. Checkpoint Snapshot Purpose

The registry checkpoint snapshot is the compact current-state view of the shard
identity registry at one committed registry generation.

Its purpose is to provide:

- a trusted base state for recovery
- a deterministic journal replay boundary
- compact integrity metadata for the registry head

It is not the only source of truth. The full registry recovery contract is:

`valid registry checkpoint snapshot + validated journal suffix => resumable registry state`

## 5. Required Distinctions

This note keeps four separate concepts explicit.

### 5.1 Checkpoint Snapshot Contents

The snapshot contains the materialized registry state at one committed
generation.

### 5.2 Journal Suffix Requirements

The journal suffix contains the ordered committed mutations after the snapshot
generation.

### 5.3 Recovery Validation Rules

Validation determines whether the snapshot and suffix are internally coherent,
complete, and safe to replay.

### 5.4 Resumable vs Non-Resumable State

Resumable means the registry head can be reconstructed without guessing.

Non-resumable means recovery cannot prove current durable identity truth and
must refuse resume.

## 6. Checkpoint Snapshot Contents

The first in-process snapshot should contain six classes of information.

### 6.1 Snapshot Envelope

Required fields:

- `artifact_type`
- `snapshot_version`
- `shard_id`
- `registry_checkpoint_id`
- `registry_checkpoint_generation`
- `created_from_manifest_generation` if available
- `world_ruleset_version`
- `benchmark_engine_version`

Purpose:

- identify the snapshot
- anchor it to one shard
- bind it to one registry generation
- allow compatibility checks during recovery

### 6.2 Registry Head Summary

Required fields:

- `last_committed_mutation_generation`
- `last_committed_mutation_id`
- `record_family_counts`
- `snapshot_payload_hash`

This is the compact integrity summary for the checkpointed registry state.

### 6.3 Materialized Durable Records

The snapshot should materialize the current durable registry records:

- accounts
- agent profiles
- characters
- system identities if used by the shard

Each record should include its normal version fields:

- `record_version`
- `created_generation`
- `updated_generation`
- `last_mutation_id`
- `status`

### 6.4 Checkpointed Session Records

Session records are transient, but the snapshot may include:

- active sessions
- recently detached sessions still eligible for reconnect policy

Each included session should still carry:

- `record_version`
- `attached_generation`
- `detached_generation`
- `last_mutation_id`

The rule is:

- only checkpoint session records that matter to deterministic reconnect or
  stale-session resolution

### 6.5 Binding and Index Views

The snapshot may materialize simple derived lookup views when useful for the
first implementation, for example:

- `characters_by_account`
- `characters_by_agent_profile`
- `active_sessions_by_character`

These must be reconstructable from primary records and should be treated as
derived checkpoint conveniences, not independent truth.

### 6.6 Snapshot Integrity Metadata

Required integrity-related fields:

- `snapshot_payload_hash`
- `record_set_hashes` by family if helpful
- `checkpoint_commit_marker`
- `expected_next_mutation_generation`

The minimal meaning is:

- this snapshot is complete through one exact registry generation
- replay must begin from `expected_next_mutation_generation`

## 7. Identity and Version Fields

At minimum, every snapshot must make these fields inspectable:

- `registry_checkpoint_generation`
- `last_committed_mutation_generation`
- `last_committed_mutation_id`
- `expected_next_mutation_generation`
- per-record `record_version`
- per-record `updated_generation`

These fields let recovery answer:

- what generation the snapshot represents
- whether the journal suffix begins where the snapshot ends
- whether a replayed mutation is stale, duplicated, or valid

## 8. Valid Checkpoint Snapshot Example

```json
{
  "artifact_type": "registry_checkpoint_snapshot",
  "snapshot_version": "v1",
  "shard_id": "shard_dev_alpha",
  "registry_checkpoint_id": "reg_cp_0009",
  "registry_checkpoint_generation": 46,
  "world_ruleset_version": "world-v1",
  "benchmark_engine_version": "engine-v1",
  "last_committed_mutation_generation": 46,
  "last_committed_mutation_id": "mut_00000046",
  "expected_next_mutation_generation": 47,
  "record_family_counts": {
    "accounts": 1,
    "agent_profiles": 2,
    "characters": 2,
    "sessions": 1
  },
  "accounts": [
    {
      "account_id": "acct_human_alice",
      "status": "active",
      "record_version": 3,
      "created_generation": 12,
      "updated_generation": 30,
      "last_mutation_id": "mut_00000030"
    }
  ],
  "agent_profiles": [
    {
      "agent_profile_id": "profile_mock_llm_v1",
      "status": "active",
      "record_version": 2,
      "created_generation": 14,
      "updated_generation": 28,
      "last_mutation_id": "mut_00000028"
    }
  ],
  "characters": [
    {
      "character_id": "char_market_runner_01",
      "controller_agent_profile_id": "profile_mock_llm_v1",
      "status": "active",
      "record_version": 7,
      "created_generation": 18,
      "updated_generation": 44,
      "last_mutation_id": "mut_00000044"
    }
  ],
  "sessions": [
    {
      "session_id": "sess_00041",
      "character_id": "char_market_runner_01",
      "status": "inactive",
      "record_version": 2,
      "attached_generation": 45,
      "detached_generation": 46,
      "last_mutation_id": "mut_00000046"
    }
  ],
  "checkpoint_commit_marker": "complete",
  "snapshot_payload_hash": "sha256:registry_snapshot_payload_hash"
}
```

## 9. Relationship to the Mutation Journal

The relationship is intentionally simple:

- the snapshot is the compact materialized registry head at generation `G`
- the journal suffix contains committed mutations with generation `> G`

Recovery must never:

- replay mutations at or below `G`
- skip a committed mutation after `G`
- infer a mutation that is not present in the suffix

The journal remains the causal explanation. The snapshot remains the fast base.

## 10. Journal Suffix Requirements

For a snapshot at generation `G`, the suffix must satisfy all of the following:

- first suffix mutation has generation `G + 1`
- suffix mutations are strictly ordered by generation
- no duplicate `mutation_id`
- no duplicate generation numbers
- each entry’s `expected_record_version` matches the snapshot or prior replayed
  result
- suffix ends exactly at the claimed latest committed registry generation

If any of these cannot be proven, the registry is not safely resumable.

## 11. Checkpoint Recovery Validation Rules

The first recovery implementation should use explicit validation gates.

### 11.1 Snapshot Validation

Recovery should validate:

- snapshot file exists
- `checkpoint_commit_marker` is present and valid
- `artifact_type` and `snapshot_version` are recognized
- `shard_id` matches the recovery target shard
- `snapshot_payload_hash` matches stored payload
- `last_committed_mutation_generation` is coherent with
  `expected_next_mutation_generation`
- per-family counts match the materialized record sets

### 11.2 Suffix Validation

Recovery should validate:

- every referenced journal segment or entry exists
- first suffix generation equals `expected_next_mutation_generation`
- suffix ordering is contiguous and deterministic
- every journal entry hash or payload integrity check passes if present
- suffix head agrees with recovery-manifest metadata

### 11.3 Replay Validation

After replay, recovery should validate:

- resulting registry head generation equals manifest-claimed head generation
- resulting `last_mutation_id` equals manifest-claimed head mutation id
- no replayed entry violated `expected_record_version`
- derived indexes or lookup views are internally coherent

Only then may the registry be marked resumable.

## 12. Valid Snapshot + Journal Suffix Recovery Path Example

Illustrative path:

1. load valid checkpoint snapshot `reg_cp_0009` at generation `46`
2. snapshot says `expected_next_mutation_generation = 47`
3. journal suffix contains:
   - `mut_00000047` `session_attached`
   - `mut_00000048` `session_marked_active`
4. both entries validate and replay cleanly
5. reconstructed registry head is generation `48`
6. recovery manifest also claims registry head generation `48`
7. registry is marked `resumable`

## 13. Incomplete or Corrupt Snapshot Handling

The snapshot must be rejected as non-authoritative if any of these hold:

- commit marker missing
- payload hash mismatch
- required record family omitted unexpectedly
- generation fields contradict each other
- materialized records contain impossible version regressions
- shard id or engine/version identity fields do not match recovery context

The safe rule is:

- do not repair the snapshot by guessing from later journal state

Instead:

- fall back to the previous valid registry checkpoint snapshot, if one exists
- otherwise treat the registry as non-resumable

## 14. Corrupt / Incomplete Snapshot Example

Illustrative failure:

- snapshot file `reg_cp_0010` exists
- `checkpoint_commit_marker` is absent
- `last_committed_mutation_generation = 52`
- `expected_next_mutation_generation = 60`

This snapshot is invalid because:

- atomic completion cannot be proven
- generation continuity is broken

Recovery must reject `reg_cp_0010` and fall back to the latest earlier valid
checkpoint.

## 15. Replay and Recovery Decision Rules

The first implementation should classify recovery outcomes explicitly.

### 15.1 Resumable

Resumable means all of the following are true:

- valid snapshot
- valid journal suffix
- replay completes with no version conflicts
- reconstructed head matches recovery-manifest claims

### 15.2 Degraded but Inspectable

Degraded means:

- a snapshot is readable, but
- journal suffix completeness cannot be fully proven, or
- non-critical derived views are missing

In this state:

- export/debug inspection may be allowed
- live resume must not be allowed

### 15.3 Non-Resumable

Non-resumable means:

- no valid snapshot exists, or
- no valid recovery chain from snapshot through journal suffix can be proven

In this state:

- the registry may be archived for forensics
- live shard resume must be refused

## 16. Non-Resumable Recovery Decision Example

Illustrative decision:

1. valid snapshot exists at generation `46`
2. recovery manifest claims latest head generation `50`
3. journal suffix provides generations `47`, `48`, and `50`
4. generation `49` is missing and no alternate authoritative snapshot exists

Decision:

- registry state is `non_resumable`

Reason:

- suffix completeness between snapshot and claimed head cannot be proven

The implementation must refuse resume rather than skipping directly from `48`
to `50`.

## 17. Interaction with Shard Recovery Manifests

The shard recovery manifest should reference the registry checkpoint seam with
fields such as:

- `registry_checkpoint_id`
- `registry_checkpoint_generation`
- `registry_last_mutation_generation`
- `registry_last_mutation_id`
- `registry_journal_suffix_ref`
- `registry_integrity_status`

The manifest does not replace snapshot validation. It records the claimed valid
head so recovery can confirm the reconstructed registry matches the shard’s
authoritative recovery boundary.

## 18. Observability and Debugging Implications

This checkpoint format should make a few operator questions easy to answer:

- what registry generation is checkpointed
- how many records of each family were captured
- what journal suffix is still pending replay
- whether the current shard state is resumable, degraded, or non-resumable

Exports and observer tooling do not need raw snapshot internals by default, but
they should be able to consume compact summaries derived from:

- checkpoint generation
- last mutation generation
- integrity status
- family counts
- reconnect-relevant session counts

## 19. Phased Implementation Path

### Phase 1: Embedded Snapshot File

Implement:

- one in-process registry snapshot format
- materialized record families
- required generation fields
- commit marker plus payload hash

### Phase 2: Journal-Suffix Recovery

Implement:

- replay from `expected_next_mutation_generation`
- contiguous generation validation
- resumable vs non-resumable decision logic

### Phase 3: Recovery Manifest Integration

Implement:

- registry checkpoint references in the shard recovery manifest
- registry head validation against manifest claims
- explicit degraded/non-resumable integrity statuses

### Phase 4: Export / Ops Summaries

Implement:

- compact registry recovery summaries for shard export and debugging tools
- observer-facing integrity summaries where helpful

## 20. Final Design Rule

The registry checkpoint snapshot is authoritative only for one exact committed
generation.

The journal suffix is authoritative only for the ordered committed mutations
after that generation.

Recovery is allowed only when both sides agree and completeness can be proven.

If completeness cannot be proven, MUDBench should preserve inspectability and
forensics, but it must not pretend the shard identity registry is safely
resumable.
