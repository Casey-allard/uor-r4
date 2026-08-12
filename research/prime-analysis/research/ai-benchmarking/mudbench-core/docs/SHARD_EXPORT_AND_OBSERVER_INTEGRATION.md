# Shard Export and Observer Integration Design Note

## Status

- Type: architecture/design note
- Scope: implementation-aware export/observer note
- Intent: define how persistent shard data should be exported and integrated into the existing reports/export/observer workflow without replacing the current benchmark-first export model

## 1. Why This Note Exists

MUDBench already has a concrete export and observer workflow in benchmark mode:

- saved suite report plus manifest files
- `reports history`
- `reports export`
- exported viewmodel sections such as `artifacts`, `history`, `leaderboard`,
  `identity_rollups`, `coverage`, and `replay_drilldowns`
- a static local observer that renders leaderboard, coverage, identity rollups,
  match summary, arena summary, and replay drilldown views

The new shard notes add:

- persistent shard storage through checkpoints and event segments
- shard recovery manifests and resumability rules

This note defines how persistent shard data should enter the same reporting
philosophy:

- exportable, machine-readable viewmodels
- observer-friendly summaries
- replay drilldown windows
- explicit degradation when history is too large

The goal is not to build a second analytics stack. The goal is to extend the
current benchmark export workflow into a shard-aware sibling flow.

## 2. Benchmark Export vs Persistent Shard Export

### 2.1 Benchmark Export Flow

Current benchmark export is bounded and artifact-centric.

Properties:

- one saved suite artifact references one bounded set of scenario runs
- `reports history` aggregates saved report manifests
- `reports export` emits a stable static-viewer viewmodel
- replay drilldowns are finite and already embedded in the export
- the observer can load the full export into memory

This is appropriate because benchmark artifacts are finite by design.

### 2.2 Persistent Shard Export Flow

Persistent shard export should be shard-centric and window-aware.

Properties:

- one shard may have a long-lived history
- checkpoints and event segments are the primary stored sources
- recovery manifests determine valid resumable boundaries
- exports should be sliced summaries, not “entire shard forever” dumps by
  default
- observer consumption must remain static-file compatible

Persistent shard export should therefore be additive:

- benchmark export remains unchanged for official benchmark artifacts
- shard export becomes a parallel export mode built around shard summaries and
  replay slices

## 3. Shard Export Goals and Non-Goals

### 3.1 Goals

Shard export should:

- expose shard state and history through machine-readable JSON
- reuse the current viewmodel style where possible
- support human inspection in the static observer
- support identity-aware and season-aware summaries
- support replay drilldowns over selected windows or encounters
- degrade safely when history volume is too large for full inline export

### 3.2 Non-Goals

Shard export should not:

- replace benchmark exports
- inline the full lifetime of very large shards by default
- invent a new backend service requirement
- require a web server or live API for the local observer
- weaken benchmark replay or scorecard audit semantics

## 4. Exportable Shard Artifact and Viewmodel Layers

Persistent shard export should be layered, just like current benchmark export is
already layered.

### 4.1 Artifact Discovery Layer

The shard export process should discover and reference:

- shard recovery manifests
- latest valid checkpoint ref
- referenced event segment ranges
- shard summary artifacts if present
- season summary refs if present

This is analogous to how current benchmark export starts from saved report
manifests and their paired files.

### 4.2 History Layer

Shard export should include a history-like layer describing exportable shard
windows or generations, for example:

- recovery generation summaries
- shard incident windows
- encounter summaries
- season summaries

This should mirror the role `history` already plays for benchmark artifacts:

- compact discoverability first
- deeper replay drilldown second

### 4.3 Summary Layer

The export should include observer-friendly summaries such as:

- shard roster summary
- identity rollups
- encounter summaries
- season/leaderboard summaries where derivable

This aligns with the current `leaderboard`, `identity_rollups`, and coverage
sections.

### 4.4 Replay Drilldown Layer

The export should include replay drilldowns only for selected windows, not for
an entire long-lived shard by default.

This is the shard equivalent of the current `replay_drilldowns` array.

## 5. Mapping Checkpoints and Segments into Observer-Friendly Output

The observer should not need raw storage internals to be useful.

### 5.1 Checkpoint Mapping

Checkpoint information should be exported as compact observer metadata such as:

- `shard_id`
- current recovery generation
- latest valid checkpoint id
- checkpoint tick
- checkpoint state hash
- latest committed tick

The observer can use this to show:

- what snapshot the shard is based on
- how current the export is
- whether the shard export is resumable or degraded

### 5.2 Event Segment Mapping

Event segments should map into:

- replay slice summaries
- encounter windows
- event-type summaries
- selected run/incident drilldowns

The observer should consume segment-derived windows, not raw segment internals
by default.

### 5.3 Recovery Manifest Mapping

Recovery-manifest data should map into export fields such as:

- `recovery_status`
- `recovery_generation`
- `checkpoint_ref`
- `segment_range`
- `integrity_summary`

This lets the observer expose resumability and drift/failure state without
teaching the page the full recovery-manifest schema.

## 6. Replay Slice vs Full-Shard History Boundaries

This is the most important boundary in the design.

### 6.1 Full-Shard History

Full-shard history means:

- the ordered lifetime of all checkpoints and segments over a shard

This is useful for:

- storage
- ops
- recovery
- offline analysis

It is usually too large to inline into a static export payload.

### 6.2 Replay Slice

A replay slice is the observer-ready unit.

Examples:

- one encounter between selected identities
- one incident window
- one segment boundary window
- one session window
- one season highlight replay

This matches the current benchmark observer model, where the page focuses on
one drilldown at a time rather than an unbounded log.

### 6.3 Export Rule

Default shard export should include:

- compact shard and identity summaries
- coverage/season/encounter metadata
- selected replay slices

Default shard export should not include:

- the entire raw shard event history inline

## 7. Identity, Encounter, and Season Summary Implications

The current system already exports:

- actor coverage
- external label coverage
- external profile coverage
- identity rollups
- arena and match summaries for comparison artifacts

Persistent shard export should extend this model rather than replace it.

### 7.1 Identity Summaries

Shard export should support:

- principal/profile/entity identity summaries
- participation windows
- encounter counts
- shard/season coverage counts

This is the persistent-world evolution of current `identity_rollups`.

### 7.2 Encounter Summaries

A shard export should expose encounter summaries for observer panels similar to
today’s match-summary and arena-summary views.

Useful summary fields:

- encounter id
- shard id
- involved identities
- scenario/zone/context label if applicable
- score or outcome delta where derivable
- replay-slice ref

### 7.3 Season Summaries

Season summaries should be exported as bounded rollups, not open-ended shard
logs.

Useful season fields:

- `season_id`
- covered shard ids
- participating identities
- aggregate summary metrics
- representative replay slices

This lets the observer stay summary-first while still linking to drilldowns.

## 8. Scale and Performance Considerations

Persistent shards create a scale problem that benchmark artifacts mostly avoid.

### 8.1 Why Scale Is Different

Benchmark exports are finite because runs are finite.

Shard exports may grow indefinitely due to:

- long-lived event histories
- many recovery generations
- many identities
- many encounters

### 8.2 Export Strategy for Large Shards

The export system should prefer:

- bounded summary sections
- selected replay slices
- compact integrity metadata
- optional truncation flags and continuation refs

It should avoid:

- embedding every historical segment
- full inline event logs for the lifetime of a shard

### 8.3 Observer Degradation Strategy

The static observer should degrade gracefully when shard history is too large.

That means:

- render summaries first
- render selected replay slices only
- show explicit truncation / partial-history notices
- avoid pretending the loaded JSON is the whole shard if it is only a bounded
  export window

This is already consistent with current observer behavior, which is based on
loaded export data rather than on-demand backend fetches.

## 9. Minimal Shard Export Payload Example

Illustrative minimal export shape:

```json
{
  "command": "shards_export",
  "viewmodel_version": "shard_export_viewmodel_v1",
  "shards": [
    {
      "shard_id": "shard-alpha",
      "season_id": "season-1",
      "recovery_status": "resumable",
      "recovery_generation": 42,
      "checkpoint_ref": {
        "checkpoint_id": "checkpoint-0039",
        "checkpoint_tick": 18420,
        "checkpoint_state_hash": "sha256:abc123"
      },
      "latest_committed_tick": 18510,
      "identity_coverage": {
        "principal_ids": ["principal-a", "principal-b"],
        "profile_ids": ["baseline-rule-profile", "candidate-mock-profile"],
        "entity_ids": ["agent-a", "agent-b"]
      }
    }
  ],
  "history": [
    {
      "shard_id": "shard-alpha",
      "recovery_generation": 42,
      "encounter_count": 3,
      "replay_slice_count": 2
    }
  ],
  "identity_rollups": [
    {
      "identity_type": "external_agent_profile",
      "identity_value": "candidate-mock-profile",
      "scenario_coverage_count": 2,
      "artifact_count": 1,
      "has_comparison_artifacts": true
    }
  ],
  "replay_slices": [
    {
      "slice_id": "encounter-18421-18510",
      "shard_id": "shard-alpha",
      "encounter_id": "encounter-7",
      "event_types": ["movement", "trade_completed", "state_snapshot"]
    }
  ]
}
```

This is intentionally summary-first and static-viewer friendly.

## 10. Replay-Slice Export Example for One Encounter

Illustrative replay-slice payload:

```json
{
  "slice_id": "encounter-7",
  "shard_id": "shard-alpha",
  "season_id": "season-1",
  "recovery_generation": 42,
  "encounter_id": "encounter-7",
  "identity_summary": [
    {"identity_type": "external_agent_profile", "identity_value": "baseline-rule-profile"},
    {"identity_type": "external_agent_profile", "identity_value": "candidate-mock-profile"}
  ],
  "tick_range": {"start": 18421, "end": 18510},
  "event_types": ["movement", "trade_completed", "state_snapshot"],
  "score_summary": {
    "baseline_composite_score": 0.7,
    "candidate_composite_score": 0.8,
    "difference": 0.1
  },
  "final_state_summary": {
    "locations": {"agent-a": "market", "agent-b": "hall"},
    "inventory": {"agent-a": ["artifact"], "agent-b": []}
  },
  "replay_events": [
    {"tick": 18421, "event_type": "movement"},
    {"tick": 18422, "event_type": "trade_completed"},
    {"tick": 18425, "event_type": "state_snapshot"}
  ]
}
```

This is the shard equivalent of today’s benchmark replay drilldown entry:

- compact enough for the static observer
- still tied to explicit identity and recovery context

## 11. Observer Degradation Example for Large History

When full shard history is too large, the observer should behave like this:

- show shard summary
- show recovery status and checkpoint ref
- show coverage and identity rollups
- show bounded encounter list or replay-slice list
- show a notice such as:
  `This export contains 25 replay slices from a larger shard history. Full shard history was not embedded in this static export.`

The observer should not:

- freeze trying to render huge history tables
- imply that omitted history does not exist
- require a live backend to remain useful

## 12. Observer Panel Implications

The current observer already knows how to render:

- leaderboard summaries
- coverage
- identity rollups
- match summary
- arena summary
- replay inspector

Shard-aware observer support should reuse those interaction patterns.

### 12.1 New or Adapted Panels

Likely observer additions:

- shard summary panel
- recovery status panel
- encounter list panel
- replay-slice inspector backed by shard export

### 12.2 Existing Panels That Can Be Reused

Existing concepts that map well:

- `identity_rollups` -> shard identity participation summaries
- replay inspector -> replay-slice inspector
- arena summary -> shared encounter/duel summary when shard exports describe
  competitive interactions

### 12.3 Graceful Fallback

If a loaded export includes only benchmark artifacts:

- keep current benchmark observer behavior

If a loaded export includes shard summaries but not full replay slices:

- render summary panels
- disable or limit replay inspection with an explicit message

## 13. Recommended Viewmodel Strategy

The safest path is to extend the current export philosophy rather than overload
`reports export` immediately with every possible shard shape.

Recommended layering:

1. keep current benchmark `reports export` unchanged
2. introduce shard-aware export payloads with familiar sections:
   - `history`
   - `identity_rollups`
   - `coverage`
   - `replay_slices` or shard-specific replay drilldowns
3. let the observer branch on `viewmodel_version` and available sections
4. keep replay units bounded and selected

This preserves the benchmark workflow and keeps shard export additive.

## 14. Phased Implementation Path

### Phase 0: Current Benchmark Export Model

Already present:

- report manifests
- report history/export
- identity rollups
- replay drilldowns
- static observer panels over benchmark artifacts

### Phase 1: Recovery-Aware Shard Export Metadata

Add:

- shard summary export
- recovery status and checkpoint refs
- basic shard coverage and identity summaries

Goal:
- make persistent shards discoverable in the same reporting style

### Phase 2: Replay-Slice Export

Add:

- selected replay slices derived from event segments
- encounter-level summaries
- observer replay-slice inspection

Goal:
- make shard behavior inspectable without embedding full shard history

### Phase 3: Season and Identity Rollup Integration

Add:

- season summaries
- identity participation and encounter rollups
- observer panels for shard/season summaries

Goal:
- support longitudinal viewing similar to current benchmark history/rollup views

### Phase 4: Large-Shard Degradation and Export Policy

Add:

- explicit export windowing rules
- truncation/continuation metadata
- observer messaging for partial-history payloads

Goal:
- keep static export usable as shard history grows

## 15. Recommended Guardrails

To keep shard export aligned with benchmark-first principles:

- do not merge shard exports into official benchmark leaderboards by default
- do not require the observer to load full shard history inline
- keep replay slices bounded and named as slices, not full truth
- preserve recovery/integrity metadata in exported summaries
- preserve the distinction between benchmark artifact exports and shard exports

## 16. Bottom Line

Persistent shard export should extend MUDBench’s current reporting model, not
replace it.

The correct progression is:

- benchmark mode keeps its current bounded `reports export` workflow
- shard mode adds shard-aware summary exports plus bounded replay slices
- the static observer learns to render those shard summaries and slices using
  the same summary-first, drilldown-second approach it already uses today

That keeps the system implementation-aware, scalable for long-lived shards, and
compatible with the benchmark-first philosophy already established in the repo.
