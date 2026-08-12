# Shard Recovery and Integrity Design Note

## Status

- Type: architecture/design note
- Scope: implementation-aware recovery/integrity note
- Intent: define shard recovery manifests and integrity verification for persistent-world mode without weakening benchmark artifact integrity

## 1. Why This Note Exists

The current MUDBench implementation already treats integrity as a first-class
concern in benchmark mode:

- replay artifacts are deterministic and reconstructable
- parity artifacts expose terminal state and score-summary hashes
- saved suite reports are accompanied by manifests and replay sidecars
- history/export/observer surfaces assume artifacts are discoverable and
  inspectable through stable references

The new shard persistence note introduces:

- durable checkpoints
- append-only event segments
- recovery from `checkpoint + ordered event segments`

This note defines the missing control plane object around those shard artifacts:

- the shard recovery manifest

The recovery manifest is not a replacement for checkpoint or segment contents.
It is the integrity-oriented index that says which checkpoint and which event
segments form the last known valid resumable shard state.

## 2. Benchmark Integrity vs Shard Recovery Integrity

### 2.1 Benchmark Artifact Integrity

Benchmark integrity remains the official strict mode.

It is concerned with:

- one bounded run
- deterministic replay reconstruction
- score parity verification
- immutable run artifacts
- release-gated reproducibility

Examples already visible in the implementation:

- replay checksum
- replay parity artifact
- terminal state hash
- applied steps hash
- score summary hash

Benchmark integrity answers:

- can this official run be trusted and recomputed exactly

### 2.2 Persistent Shard Recovery Integrity

Persistent shard recovery integrity is a different question.

It is concerned with:

- whether the shard can safely resume after interruption
- whether checkpoint and event segment references are complete and ordered
- whether the latest committed shard boundary is trustworthy
- whether resumable state can be reconstructed without guessing

Shard recovery integrity answers:

- what is the newest safe point from which this live world may resume

It should be strict, but it should not pretend a long-lived shard is the same
artifact shape as a benchmark run.

## 3. Recovery Manifest Purpose

The recovery manifest should be the canonical resumability record for one shard
at one committed recovery boundary.

Its purposes are:

- identify the current shard recovery state
- point to the authoritative checkpoint
- list the committed event segments after that checkpoint
- record integrity evidence for those references
- declare whether the shard is resumable, degraded, or non-resumable
- expose enough metadata for history/export/observer tooling to inspect the
  state without directly parsing storage internals first

The recovery manifest should be append-new / replace-by-new-version rather than
mutating historical truth in place.

## 4. Recovery Manifest Lifecycle

### 4.1 Creation Points

A shard recovery manifest should be written:

- at shard bootstrap from a fresh checkpoint
- after each successful checkpoint commit
- after clean segment closure if the latest resumable boundary changes
- before and after graceful shard shutdown

### 4.2 Update Philosophy

The manifest should represent the last fully committed recovery boundary only.

That means:

- do not point the manifest at an in-progress checkpoint
- do not include partially written segments as committed replay truth
- do not update resumability state optimistically before integrity checks pass

### 4.3 Historical Retention

At minimum, the system should preserve:

- the latest manifest used for active recovery
- prior manifests or equivalent audit records sufficient to explain recovery
  state transitions over time

This follows the benchmark-first principle that audit trails matter.

## 5. Manifest Identity and Version Fields

The recovery manifest should include stable identity and provenance fields.

Minimum fields:

- `artifact_type`
- `manifest_version`
- `shard_id`
- `season_id` if applicable
- `world_ruleset_version`
- `benchmark_engine_version`
- `scheduler_policy_version`
- `manifest_id`
- `created_at` or logical commit marker
- `recovery_generation`

Recommended purpose of each:

- `artifact_type`: distinguish shard recovery manifests from suite report
  manifests
- `manifest_version`: version the manifest shape itself
- `shard_id`: primary world identity
- `recovery_generation`: monotonic recovery boundary counter for the shard

The note does not require a final timestamp policy. If wall-clock timestamps are
used, they should be informational rather than the primary ordering source.

## 6. Checkpoint and Segment References

The manifest should reference storage artifacts explicitly rather than embedding
all shard data directly.

### 6.1 Checkpoint Reference

The manifest should point to exactly one authoritative base checkpoint:

- `checkpoint_id`
- `checkpoint_path` or storage ref
- `checkpoint_tick`
- `checkpoint_state_hash`
- `checkpoint_payload_hash`

This is the recovery anchor.

### 6.2 Event Segment References

The manifest should list only the committed segments after the checkpoint:

- `segment_id`
- `segment_path` or storage ref
- `start_tick`
- `end_tick`
- `segment_payload_hash`
- `previous_segment_id` where relevant

Ordering must be explicit and stable.

### 6.3 Terminal Segment Summary

To support observability and quick validation, the manifest should also carry a
compact summary of the latest committed suffix, for example:

- last committed tick
- last committed segment id
- last visible `state_snapshot` hash if present
- last known roster/entity count summary if available

This mirrors the current benchmark pattern where a compact integrity summary is
paired with richer replay data.

## 7. Integrity Verification Strategy

### 7.1 Integrity Checks

The recovery manifest should be considered valid only if all of the following
hold:

- checkpoint reference exists
- checkpoint hash matches stored checkpoint payload
- segment list is complete and ordered
- each segment hash matches stored segment payload
- segment chain is contiguous according to shard policy
- manifest identity/version fields are internally coherent
- referenced checkpoint and segments all match shard/version identity fields

### 7.2 Ordering Verification

At minimum, ordering verification should check:

- segments are listed in deterministic order
- the first segment starts after the checkpoint boundary
- each segment starts where the previous committed segment ended, according to
  policy
- no duplicate segment ids
- no ambiguous branch unless future policy explicitly supports forks

The safe default is no branching. One shard should have one linear committed
recovery chain.

### 7.3 Completeness Verification

Completeness should answer:

- is the event suffix between the base checkpoint and the latest committed tick
  fully accounted for

If the system cannot prove completeness, the state should not be marked fully
resumable.

## 8. Crash Recovery and Partial Write Handling

### 8.1 Checkpoint Partial Write

If a checkpoint file exists but:

- hash validation fails, or
- atomic commit marker is absent, or
- referenced payload is incomplete

then that checkpoint must be treated as non-authoritative.

Recovery should fall back to the previous valid checkpoint referenced by the
most recent valid manifest.

### 8.2 Segment Partial Write

If a segment appears on disk but:

- the segment hash does not match
- the segment end marker is missing
- the manifest does not include it as committed

then it must not be used for resumable reconstruction.

This matches the principle from the shard persistence note:

- only fully committed segments count

### 8.3 Manifest Partial Write

If the manifest itself is partially written or malformed:

- do not infer resumability from it
- fall back to the previous valid manifest if available
- otherwise mark the shard as non-resumable pending operator intervention

The system should never silently “best effort” reconstruct from a manifest that
cannot be parsed and verified.

## 9. Resumable vs Non-Resumable Decision Rules

The recovery layer needs explicit outcome categories.

### 9.1 Resumable

Mark shard state as `resumable` only when:

- manifest parses successfully
- checkpoint validates successfully
- all referenced segments validate successfully
- chain ordering is complete
- shard/version identity fields are coherent

### 9.2 Degraded but Inspectable

Mark shard state as `degraded_inspectable` when:

- the latest boundary is not resumable
- but an earlier manifest/checkpoint chain is still valid
- and replay/debugging can still inspect the damaged boundary as evidence

This is useful for ops and observer tooling:

- humans can inspect what failed
- the shard does not blindly resume from uncertain state

### 9.3 Non-Resumable

Mark shard state as `non_resumable` when:

- no valid checkpoint chain exists
- shard identity/version coherence fails
- integrity verification cannot identify a trustworthy recovery boundary

In that case, the correct behavior is to stop and surface the failure, not to
invent a continuation.

## 10. Valid Recovery Manifest Example

Illustrative example:

```json
{
  "artifact_type": "shard_recovery_manifest_v1",
  "manifest_version": "1",
  "manifest_id": "recovery-shard-alpha-0042",
  "recovery_generation": 42,
  "shard_id": "shard-alpha",
  "season_id": "season-1",
  "world_ruleset_version": "world-v1",
  "benchmark_engine_version": "0.1",
  "scheduler_policy_version": "tick-policy-v1",
  "recovery_status": "resumable",
  "checkpoint": {
    "checkpoint_id": "checkpoint-0039",
    "checkpoint_path": "shards/shard-alpha/checkpoints/checkpoint-0039.json",
    "checkpoint_tick": 18420,
    "checkpoint_state_hash": "sha256:abc123",
    "checkpoint_payload_hash": "sha256:def456"
  },
  "segments": [
    {
      "segment_id": "segment-0040",
      "segment_path": "shards/shard-alpha/segments/segment-0040.json",
      "start_tick": 18421,
      "end_tick": 18480,
      "previous_segment_id": null,
      "segment_payload_hash": "sha256:111aaa"
    },
    {
      "segment_id": "segment-0041",
      "segment_path": "shards/shard-alpha/segments/segment-0041.json",
      "start_tick": 18481,
      "end_tick": 18510,
      "previous_segment_id": "segment-0040",
      "segment_payload_hash": "sha256:222bbb"
    }
  ],
  "latest_committed_tick": 18510,
  "latest_state_snapshot_hash": "sha256:333ccc"
}
```

Why this is valid:

- one authoritative checkpoint
- two ordered committed segments
- coherent shard/version identity
- explicit resumability claim backed by referenced hashes

## 11. Partial/Corrupt Manifest Example

Illustrative corrupt case:

```json
{
  "artifact_type": "shard_recovery_manifest_v1",
  "manifest_version": "1",
  "manifest_id": "recovery-shard-alpha-0043",
  "recovery_generation": 43,
  "shard_id": "shard-alpha",
  "world_ruleset_version": "world-v1",
  "benchmark_engine_version": "0.1",
  "scheduler_policy_version": "tick-policy-v1",
  "recovery_status": "resumable",
  "checkpoint": {
    "checkpoint_id": "checkpoint-0042",
    "checkpoint_path": "shards/shard-alpha/checkpoints/checkpoint-0042.json",
    "checkpoint_tick": 18510,
    "checkpoint_state_hash": "sha256:broken"
  },
  "segments": [
    {
      "segment_id": "segment-0043",
      "segment_path": "shards/shard-alpha/segments/segment-0043.json",
      "start_tick": 18511,
      "end_tick": 18540,
      "previous_segment_id": "segment-0041",
      "segment_payload_hash": "sha256:not-found"
    }
  ]
}
```

Problems:

- checkpoint payload hash is missing
- claimed resumable status is unsupported by the data
- referenced segment may not exist or may fail hash validation
- chain continuity from the previous valid boundary is ambiguous

This manifest must not be accepted as resumable.

## 12. Recovery Decision Rules Example

Example decision rules for the corrupt case above:

1. parse manifest
2. validate required fields
3. validate checkpoint payload/hash pair
4. validate referenced segment presence and hash
5. validate segment ordering and continuity

Outcome:

- if steps 3-5 fail for generation `43`, reject generation `43`
- if generation `42` is still fully valid, recover from generation `42`
- if no earlier valid generation exists, mark shard `non_resumable`

This is the core rule:

- recover to the last provably valid committed generation, never to the newest
  merely present generation

## 13. Validation Rules for Recovery Manifests

### 13.1 Required Field Validation

Reject manifest if required fields are absent or malformed:

- shard identity fields
- version/provenance fields
- checkpoint reference
- segment list shape
- recovery status field

### 13.2 Referential Validation

Reject resumability if:

- checkpoint shard id/version does not match manifest
- segment shard id/version does not match manifest
- checkpoint tick is after referenced segment start
- segment chain is duplicated or discontinuous

### 13.3 Status Validation

The manifest should not be trusted just because it says `resumable`.

Actual recovery status should be recomputed from the referenced artifacts.

The stored status is a summary, not the sole source of truth.

## 14. Replay and Observability Implications

The recovery manifest should fit naturally into the current reporting direction:

- manifests are already first-class discovery objects
- reports history/export already aggregate saved artifacts
- replay drilldowns already provide compact, observer-friendly windows
- identity-aware summaries already exist for actors, labels, and profiles

For persistent shards, the recovery manifest should enable:

- shard discovery without parsing every checkpoint/segment first
- operator visibility into latest valid recovery boundary
- observer/debug views that can point from shard -> manifest -> checkpoint ->
  replay window
- incident analysis over damaged or degraded shard generations

In other words, the recovery manifest should be both:

- an operational recovery object
- an observability index

## 15. Phased Implementation Path

### Phase 0: Current Baseline

Already present in benchmark mode:

- replay checksums
- replay parity artifacts
- report manifests and replay sidecars
- report history/export/observer surfaces

### Phase 1: Minimal Shard Recovery Manifest

Add:

- one manifest shape for shard checkpoint + committed segment references
- hash validation on referenced artifacts
- resumable vs non-resumable decision logic

Goal:
- make shard recovery explicit and auditable

### Phase 2: Degraded Recovery and Operator Visibility

Add:

- degraded inspectable status
- prior-generation fallback rules
- manifest exposure through export/history-like surfaces

Goal:
- let ops and observers inspect failures without guessing

### Phase 3: Observer and Season Integration

Add:

- shard recovery summaries in export/viewmodel surfaces
- links from recovery generations to replay drilldown windows
- identity-aware incident summaries

Goal:
- unify recovery visibility with the existing observer/reporting approach

### Phase 4: Concrete Schema and Write Discipline

Add:

- final schema versioning rules
- atomic write/commit conventions
- concrete storage paths or registry mapping rules

Goal:
- turn the design seam into an implementation-ready manifest format

## 16. Recommended Guardrails

To keep shard recovery integrity aligned with benchmark-first principles:

- do not reuse benchmark parity artifacts as shard recovery manifests
- do not treat presence on disk as proof of resumability
- do not allow the latest manifest generation to override earlier valid
  generations without full verification
- keep manifest status derived from verification, not trust
- keep benchmark artifact integrity and shard recovery integrity explicitly
  separate in naming and storage roles

## 17. Bottom Line

The shard recovery manifest should be the persistent-world equivalent of a
trusted recovery index, not a giant embedded replay.

It should:

- point to one valid checkpoint
- enumerate the committed segment suffix
- prove integrity through hashes and ordering checks
- classify the shard as resumable, degraded, or non-resumable
- expose enough structure for reporting and observer tooling

That keeps persistent-world recovery concrete and auditable while preserving the
stricter, bounded integrity model that benchmark mode already uses today.
