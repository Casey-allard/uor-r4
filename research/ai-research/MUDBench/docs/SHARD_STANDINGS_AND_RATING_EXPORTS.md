# Shard Standings and Rating Exports Design Note

## Status

- Type: architecture/design note
- Scope: implementation-aware standings/export note
- Intent: define how persistent-world standings and rating data should be exported without weakening MUDBench’s benchmark-first score and report model

## 1. Why This Note Exists

MUDBench already has concrete benchmark reporting semantics:

- bounded scorecards
- composite and capability-oriented summaries
- saved manifests and replay drilldowns
- history/export aggregation
- identity-aware leaderboards and rollups
- observer views over leaderboard, profile rollups, match summaries, and arena
  summaries

The shard notes add:

- persistent identities
- encounter taxonomy
- recovery-aware shard storage
- season and ladder eligibility classes

This note defines the export semantics for standings and rating summaries in
persistent-world mode.

The key design rule is:

- benchmark score exports remain benchmark artifacts
- shard standings/rating exports are a separate, shard-mode reporting surface

## 2. Benchmark Score/Report Exports vs Persistent-World Standings Exports

### 2.1 Benchmark Score/Report Exports

Current benchmark exports are based on:

- scenario-bounded runs
- scorecards
- composite score summaries
- replay parity and drilldowns
- history/export rollups over saved suite artifacts

These exports are designed to answer:

- how did an agent perform on benchmark scenarios under benchmark rules

### 2.2 Persistent-World Standings/Rating Exports

Shard standings/rating exports should answer different questions:

- which identities are season-eligible participants
- how many rated encounters counted
- which encounters were excluded
- what current standing or rating state is visible
- what evidence supports those summaries

These exports should not reuse benchmark composite score fields as if they were
the shard ladder truth.

## 3. Standings Goals and Non-Goals

### 3.1 Goals

Standings/rating exports should:

- summarize rated shard participation clearly
- make inclusion and exclusion visible
- remain identity-aware and season-aware
- remain observer-friendly and static-export friendly
- support drilldown from standing rows to encounter evidence

### 3.2 Non-Goals

This note does not define:

- the final rating algorithm
- matchmaking policy
- promotion/demotion rules
- reward distribution

The note is about export semantics, not ladder math.

## 4. Rating Export Purpose and Boundaries

### 4.1 Purpose

A rating export is the machine-readable summary of the current visible standing
state for eligible shard identities.

It should allow downstream tools and the static observer to answer:

- who is on the shard or season ladder
- what rating/standing value is currently visible
- how many eligible encounters contributed
- how many encounters were excluded
- what replay/export evidence exists

### 4.2 Boundaries

Rating exports should:

- include only season/ladder-relevant summaries
- reference encounter evidence rather than embed every event
- carry eligibility and exclusion markers

Rating exports should not:

- replace replay slices
- replace shard recovery manifests
- replace benchmark score exports

## 5. Canonical Standing Row Concept

A standing row is the summary row for one eligible identity within one season or
ladder scope.

Minimum conceptual fields:

- standing scope
- identity bundle
- current visible rating or standing metric
- eligible encounter counts
- excluded encounter counts
- summary evidence refs

The standing row is the shard analogue of a benchmark leaderboard entry, but it
must remain explicit that it is ladder/season state rather than benchmark
performance.

## 6. Canonical Rating Row Concept

A rating row is the more rating-specific export view for one identity.

It should be more stateful than a standing row and may include:

- current rating value
- prior rating value if available
- delta across the latest accounted period
- last updated encounter or encounter window
- integrity state of the contributing window

This is intentionally broader than benchmark `composite_score`, which is tied
to bounded run evaluation rather than season state.

## 7. Required Identity Fields

Standings and rating exports should carry the identity fields needed to align
with current history/export/observer work.

Minimum identity fields:

- `identity_type`
- `identity_value`
- `player_account_id` when present
- `agent_profile_id` when present
- `external_agent_label` when present
- `entity_id` only when needed for shard-local drilldown
- `identity_class`

Recommended export rule:

- use profile/principal identities for season standings
- keep entity identity available as drilldown context rather than the primary
  season identity in most cases

## 8. Required Season and Ladder Fields

Minimum season/ladder-facing fields:

- `season_id`
- `ladder_id` if a ladder surface exists distinct from season
- `eligibility_scope`
- `standings_version`
- `ruleset_version`
- `recovery_generation_window` or equivalent evidence boundary
- `last_accounted_tick` or equivalent progress marker

These keep standings scoped and comparable, mirroring the version-aware mindset
already required by benchmark reporting.

## 9. Encounter Inclusion and Exclusion Markers

Standings exports must make encounter inclusion explicit.

Recommended markers:

- `eligible_encounter_count`
- `excluded_encounter_count`
- `unrated_encounter_count`
- `excluded_reasons`
- `included_encounter_refs`
- `excluded_encounter_refs`

This is important because persistent-world standings are only trustworthy if
users can see:

- what counted
- what did not count
- why not

## 10. Aggregation Semantics for Eligible vs Excluded Encounters

### 10.1 Eligible Encounters

Only `shard_rated` encounters that satisfy identity and integrity requirements
should contribute to standing/rating totals.

These are the only encounters that should affect:

- visible standing rank
- rating value
- season summary contribution counts

### 10.2 Unrated Encounters

`shard_unrated` encounters may appear in profile history and observer views, but
should not affect standings or rating state.

They are behaviorally interesting, not ladder-authoritative.

### 10.3 Excluded Encounters

`excluded_invalid` encounters should:

- not affect standing/rating totals
- remain export-visible for transparency
- contribute to exclusion counts and reason summaries

This matches the benchmark-first principle that unverifiable outcomes should not
silently influence ranking.

## 11. Standings Row Example for an Eligible Participant

Illustrative standings row:

```json
{
  "season_id": "season-1",
  "ladder_id": "ladder-alpha",
  "identity_type": "external_agent_profile",
  "identity_value": "candidate-mock-profile",
  "agent_profile_id": "candidate-mock-profile",
  "external_agent_label": "Candidate Mock Profile",
  "identity_class": "external_agent",
  "standing_rank": 2,
  "rating_value": 1542.0,
  "eligible_encounter_count": 12,
  "unrated_encounter_count": 7,
  "excluded_encounter_count": 2,
  "last_accounted_tick": 22140,
  "season_summary_status": "eligible_with_exclusions"
}
```

Why this fits the current implementation style:

- profile identity is explicit
- summary counts are visible
- rating state is represented without pretending it is a benchmark scorecard

## 12. Excluded Encounter Contribution Record Example

Illustrative excluded contribution record:

```json
{
  "encounter_id": "arena-shard-alpha-0013",
  "season_id": "season-1",
  "identity_value": "candidate-mock-profile",
  "eligibility_class": "excluded_invalid",
  "counted_toward_standings": false,
  "exclusion_reason": "missing_replay_slice",
  "recovery_status": "degraded_inspectable",
  "replay_slice_id": null
}
```

This is the shard equivalent of saying “artifact exists, but it does not count.”

## 13. Observer-Facing Season Summary Payload Example

Illustrative observer-facing season summary:

```json
{
  "season_id": "season-1",
  "ladder_id": "ladder-alpha",
  "summary_schema_version": "shard_season_summary_v1",
  "standings": [
    {
      "identity_type": "external_agent_profile",
      "identity_value": "baseline-rule-profile",
      "standing_rank": 1,
      "rating_value": 1580.0,
      "eligible_encounter_count": 13,
      "excluded_encounter_count": 1
    },
    {
      "identity_type": "external_agent_profile",
      "identity_value": "candidate-mock-profile",
      "standing_rank": 2,
      "rating_value": 1542.0,
      "eligible_encounter_count": 12,
      "excluded_encounter_count": 2
    }
  ],
  "coverage": {
    "rated_identity_count": 2,
    "eligible_encounter_count": 25,
    "excluded_encounter_count": 3
  }
}
```

This is the summary-first payload the observer can render without loading every
encounter at once.

## 14. Rating Export Payload Example

Illustrative rating export payload:

```json
{
  "command": "shards_ratings_export",
  "viewmodel_version": "shard_ratings_export_v1",
  "season_id": "season-1",
  "ladder_id": "ladder-alpha",
  "rows": [
    {
      "identity_type": "external_agent_profile",
      "identity_value": "candidate-mock-profile",
      "agent_profile_id": "candidate-mock-profile",
      "rating_value": 1542.0,
      "rating_delta": 18.0,
      "eligible_encounter_count": 12,
      "excluded_encounter_count": 2,
      "included_encounter_refs": ["arena-shard-alpha-0007", "arena-shard-alpha-0008"],
      "excluded_encounter_refs": ["arena-shard-alpha-0013", "arena-shard-alpha-0015"]
    }
  ]
}
```

This keeps rating state compact, machine-readable, and drilldown-friendly.

## 15. Observer and Export Implications for Leaderboard, Rollups, and Profile Views

### 15.1 Leaderboard View

The observer should treat shard standings tables as a distinct panel from
benchmark leaderboard panels.

Reason:

- benchmark leaderboard rows summarize benchmark scores
- shard standings rows summarize season/rating state

They may share UI patterns, but not semantic meaning.

### 15.2 Identity Rollups

Current `identity_rollups` already summarize presence and comparison evidence.

Shard standings exports should extend that idea with season-aware fields such as:

- rated encounter counts
- excluded encounter counts
- latest visible rating value
- season participation coverage

### 15.3 Profile Views

Profile-centric observer views should be able to show:

- benchmark profile summary
- shard standing summary
- excluded encounter count
- replay-slice entrypoints for contributing rated encounters

This keeps the existing observer’s profile focus useful in shard mode.

## 16. Recommended Export Layering

The safest export layering is:

1. benchmark score/report exports remain unchanged
2. shard standings exports become a separate viewmodel family
3. shard season summary exports aggregate standing rows
4. replay slices remain separate drilldown artifacts or sub-sections

This avoids forcing one export format to do too many incompatible jobs.

## 17. Phased Implementation Path

### Phase 0: Current Benchmark Export Baseline

Already present:

- benchmark leaderboards
- identity rollups
- history/export summaries
- replay drilldowns
- arena summary surfaces for shared-run comparison artifacts

### Phase 1: Eligibility-Aware Standing Rows

Add:

- standing row semantics
- inclusion/exclusion markers
- season-aware identity fields

Goal:

- make rated shard participation exportable without finalizing rating math

### Phase 2: Rating Row Semantics

Add:

- rating row fields
- latest accounted encounter/evidence refs
- observer-ready standing/rating summary payloads

Goal:

- make season and ladder state viewable in a stable machine-readable form

### Phase 3: Observer Integration

Add:

- shard standings panel
- season summary panel
- profile view links from standing rows to encounter evidence

Goal:

- let the current observer render shard standings alongside existing rollups and
  replay views

### Phase 4: Cross-Surface Consistency

Add:

- strict naming/versioning for standings exports
- alignment between shard exports, observer panels, and any future ladder APIs

Goal:

- keep persistent-world standings as auditable as benchmark summaries

## 18. Recommended Guardrails

To keep standings/rating exports aligned with MUDBench’s benchmark-first model:

- never merge benchmark composite scores directly into shard rating fields
- never count excluded or unrated encounters in standing totals
- always expose exclusion and evidence markers
- keep season and ladder identifiers explicit
- keep identity fields stable and profile-aware

## 19. Bottom Line

The correct export model is:

- benchmark exports for benchmark performance
- shard standings exports for season-visible participation state
- shard rating exports for rating-state summaries and evidence refs

These exports can reuse MUDBench’s current summary-first observer/reporting
philosophy, but they must stay semantically distinct from benchmark score and
leaderboard outputs.
