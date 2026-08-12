# Shard Season and Ladder Eligibility Design Note

## Status

- Type: architecture/design note
- Scope: implementation-aware season/ladder eligibility note
- Intent: define which persistent-world shard encounters are eligible for season and ladder treatment without weakening official benchmark integrity

## 1. Why This Note Exists

MUDBench already has a strong benchmark validity posture:

- bounded deterministic runs
- replay and parity verification
- identity-aware comparison outputs
- observer views for shared-run arena summaries and profile-centric rollups

The shard notes extend MUDBench into persistent-world mode with:

- durable shard identities
- checkpoints and event segments
- recovery manifests
- shard exports and replay slices
- encounter taxonomy for benchmark, shared-run arena, and open-world
  interactions

This note defines the next layer:

- which shard encounters should count toward seasons and ladders
- which should stay casual or observational only
- which should be excluded entirely

The key design rule is that official benchmark eligibility and persistent-world
season eligibility are not the same thing.

## 2. Official Benchmark Eligibility vs Persistent-World Eligibility

### 2.1 Official Benchmark Eligibility

Official benchmark eligibility remains the strictest regime.

Properties:

- bounded scenario execution
- deterministic seed and step constraints
- official scorecard semantics
- replay parity and audit expectations
- benchmark-version/scoring-version separation

This is the source of:

- canonical benchmark claims
- benchmark leaderboards
- release-gated regression evidence

### 2.2 Persistent-World Season/Ladder Eligibility

Persistent-world eligibility is a hosted-world competition question, not a
benchmark validity question.

It is concerned with:

- whether an encounter is fair enough to count toward season standings
- whether the shard evidence is strong enough to support a rated result
- whether the participating identities are valid rated participants

This is separate from official benchmark scoring even if some observer/export
surfaces are shared.

### 2.3 Casual / Non-Eligible World Interactions

Many world interactions should remain visible and exportable but not ladder
eligible.

Examples:

- spontaneous trade with an NPC
- exploratory roaming
- social chatter without competitive stakes
- admin-assisted recovery interactions

These should enrich shard history, not pollute season standings.

## 3. Season and Ladder Goals and Non-Goals

### 3.1 Goals

Season and ladder rules should:

- keep rated shard results interpretable
- align with current integrity and replay discipline
- distinguish competitive encounters from casual world activity
- support observer/export summaries that are honest about what counted
- remain profile-aware and identity-aware

### 3.2 Non-Goals

This note does not attempt to define:

- a final rating formula
- matchmaking policy
- tournament bracket rules
- economic reward systems

Those are downstream decisions. This note only defines eligibility boundaries.

## 4. Encounter Eligibility Classes

Persistent-world encounters should be classified into at least four eligibility
classes.

### 4.1 Official Benchmark

Suggested field:

- `eligibility_class = official_benchmark`

Properties:

- not a shard ladder result
- benchmark-only validity path
- replay and scorecard determine benchmark standing

Official benchmark encounters should never be silently merged into shard season
standings.

### 4.2 Rated Shard Encounter

Suggested field:

- `eligibility_class = shard_rated`

This is the main ladder/season-eligible class.

Properties:

- bounded encounter window inside a shard
- valid rated identities
- no disqualifying intervention
- sufficient replay/export evidence
- integrity and recovery chain remain valid

### 4.3 Unrated but Recorded Shard Encounter

Suggested field:

- `eligibility_class = shard_unrated`

Properties:

- visible in export/history/observer
- may be interesting for behavior analysis
- does not affect ladder or season standings

Examples:

- open-world trade
- exhibition matches
- sandbox sparring

### 4.4 Excluded / Invalid Encounter

Suggested field:

- `eligibility_class = excluded_invalid`

Properties:

- not ladder-eligible
- not season-summary eligible
- may remain visible as an incident or invalidated record

Examples:

- corrupt replay slice
- partial logs
- admin-forced rollback
- identity ambiguity

## 5. Shared-Run Arena Eligibility vs Open-World Eligibility

### 5.1 Shared-Run Arena Eligibility

Shared-run arena encounters are the most natural candidate for rated shard
eligibility because they already resemble current shared-run comparison
artifacts.

They are usually:

- bounded
- competitive
- identity-explicit
- replay-slice friendly
- observer-summary friendly

Default recommendation:

- shared-run arena encounters are eligible for `shard_rated` if integrity and
  identity requirements are met

### 5.2 Open-World Interaction Eligibility

Open-world interactions should default to `shard_unrated` unless explicitly
promoted by policy.

Reason:

- they may be interrupted or asymmetrical
- context may be opportunistic rather than intentionally competitive
- outcomes may be shaped by prior shard state in ways that are not fair for
  rating without further policy

Only explicitly defined open-world encounter classes should become rated, and
only with stronger evidence and rules.

## 6. Identity Requirements for Rated Participation

Rated participation should require clear durable identity.

Minimum identity requirements:

- stable `identity_class`
- stable `agent_profile_id` or `player_account_id`
- stable `entity_id` for the encounter window
- shard and season association

Disqualifying identity conditions:

- missing or ambiguous profile/principal identity
- identity swap mid-encounter without explicit policy support
- one rated identity controlling multiple encounter participants in a way that
  violates fairness policy
- unclassified system/NPC identity presented as if it were a competitive agent

This keeps rated results compatible with current profile-aware history/export
work.

## 7. Integrity Requirements for Rated Encounters

Rated encounters should require stronger integrity than casual shard history.

Minimum requirements:

- valid shard recovery generation at encounter time
- encounter replay slice present or derivable
- complete replay/event evidence for the rated encounter window
- encounter outcome linked to known identities
- no missing segment in the encounter window
- no recovery ambiguity affecting the encounter boundary

Useful evidence fields:

- recovery generation
- checkpoint ref
- replay slice id
- event-type summary
- outcome summary
- integrity summary derived from checkpoint/segment verification

This is the shard analogue of today’s benchmark parity discipline.

## 8. Disqualification and Exclusion Rules

Encounters should be excluded from rated treatment if any of the following
occur.

### 8.1 Admin Intervention

Exclude if:

- an admin directly alters state inside the encounter window
- an operator injects inventory, position, or combat outcome changes
- a recovery override is applied mid-encounter

The encounter may remain visible, but should be marked non-eligible.

### 8.2 Crash or Partial Recovery

Exclude if:

- the shard crashes during the encounter and recovery evidence is incomplete
- the encounter crosses a corrupted or missing segment
- the recovery manifest cannot prove resumability for the relevant boundary

### 8.3 Missing Replay or Export Evidence

Exclude if:

- replay slice for the encounter is missing
- the event window is incomplete
- export fields required for identity/outcome mapping are absent

This matches the repo’s current philosophy:

- no replay/audit visibility means no trustworthy competitive claim

### 8.4 Identity Invalidity

Exclude if:

- participant identity is ambiguous
- system/NPC identities were mixed into a rated matchup without policy support
- identity ownership changes during the encounter without explicit rules

## 9. Handling of Admin Intervention, Crashes, and Partial Logs

### 9.1 Admin Intervention

Recommended rule:

- downgrade the encounter to `excluded_invalid` or `shard_unrated`
- preserve the incident in export/history for transparency

### 9.2 Crash with Clean Recovery Before Outcome

If:

- the recovery manifest remains valid
- the encounter replay slice is complete
- the outcome is fully reconstructable

then the encounter may remain eligible.

### 9.3 Crash with Incomplete Evidence

If:

- logs are partial
- segment continuity is broken
- encounter outcome cannot be cleanly attributed

then the encounter must not be rated.

### 9.4 Missing Export Evidence

If export surfaces cannot prove:

- who participated
- what window was rated
- what outcome occurred

then the observer may still show an incident, but standings must ignore it.

## 10. Eligible Shared-Run Arena Encounter Example

Illustrative eligible encounter:

```json
{
  "encounter_id": "arena-shard-alpha-0007",
  "encounter_class": "arena_shared_run",
  "eligibility_class": "shard_rated",
  "season_id": "season-1",
  "participants": [
    {"agent_profile_id": "baseline-rule-profile", "entity_id": "entity-agent-a"},
    {"agent_profile_id": "candidate-mock-profile", "entity_id": "entity-agent-b"}
  ],
  "recovery_generation": 42,
  "replay_slice_id": "encounter-7",
  "integrity_summary": {
    "recovery_status": "resumable",
    "segment_window_complete": true
  },
  "outcome_summary": {
    "winner_identity": "candidate-mock-profile",
    "score_difference": 0.1
  }
}
```

Why eligible:

- bounded shared-run encounter
- profile identities are explicit
- replay slice exists
- recovery state is valid

## 11. Non-Eligible Open-World Interaction Example

Illustrative non-eligible interaction:

```json
{
  "encounter_id": "openworld-shard-alpha-trade-021",
  "encounter_class": "open_world",
  "eligibility_class": "shard_unrated",
  "season_id": "season-1",
  "participants": [
    {"agent_profile_id": "candidate-mock-profile", "entity_id": "entity-agent-a"},
    {"identity_class": "npc_controller", "entity_id": "npc-trader-1"}
  ],
  "replay_slice_id": "trade-window-021",
  "outcome_summary": {
    "item_transfers": ["trade-token -> npc-trader-1", "artifact -> entity-agent-a"]
  }
}
```

Why non-eligible:

- open-world trade is visible and meaningful
- but it is not a bounded rated competitive match by default

## 12. Excluded / Corrupt Encounter Example

Illustrative excluded encounter:

```json
{
  "encounter_id": "arena-shard-alpha-0013",
  "encounter_class": "arena_shared_run",
  "eligibility_class": "excluded_invalid",
  "season_id": "season-1",
  "participants": [
    {"agent_profile_id": "baseline-rule-profile", "entity_id": "entity-agent-a"},
    {"agent_profile_id": "candidate-mock-profile", "entity_id": "entity-agent-b"}
  ],
  "recovery_generation": 43,
  "replay_slice_id": null,
  "integrity_summary": {
    "recovery_status": "degraded_inspectable",
    "segment_window_complete": false,
    "reason": "segment-0043 hash mismatch"
  }
}
```

Why excluded:

- competitive-looking encounter
- but missing replay slice and degraded integrity make rating unsafe

## 13. Season-Summary Eligibility Decision Example

Illustrative decision:

```json
{
  "season_id": "season-1",
  "identity_value": "candidate-mock-profile",
  "rated_encounters_considered": 12,
  "rated_encounters_excluded": 2,
  "excluded_reasons": [
    "admin_intervention",
    "missing_replay_slice"
  ],
  "season_summary_status": "eligible_with_exclusions"
}
```

Meaning:

- the profile remains season-visible
- only the 12 valid rated encounters contribute to ladder/season standing
- excluded encounters remain auditable, but do not count

## 14. Summary and Export Implications for Standings and Observer Views

Season and ladder eligibility should surface directly in exports.

Useful summary fields:

- `eligibility_class`
- `season_class`
- `rated_encounter_count`
- `excluded_encounter_count`
- `exclusion_reasons`
- `has_shared_run_arena_artifacts`
- `has_comparison_artifacts`

This fits naturally into the current export/observer direction:

- history entries can indicate rated vs unrated encounter windows
- identity rollups can separate rated contribution from total presence
- arena summary panels can indicate whether a matchup counted for standings
- observer views can surface exclusions explicitly instead of hiding them

## 15. Observer Behavior Implications

The static observer should be able to distinguish at least three categories:

- benchmark-valid artifact
- shard-rated encounter
- shard-unrated or excluded encounter

Practical consequences:

- rated arena rows should show ladder relevance
- unrated open-world rows should remain inspectable but clearly marked
- excluded encounters should show reason and avoid appearing in season totals

This preserves transparency and matches the repo’s current summary-first style.

## 16. Phased Implementation Path

### Phase 0: Current Baseline

Already present:

- benchmark scorecards and parity discipline
- shared-run comparison artifacts
- identity-aware rollups
- arena and match observer views

### Phase 1: Encounter Eligibility Labels

Add:

- `eligibility_class`
- `season_class`
- exclusion reason fields

Goal:

- make shard encounter outputs classifiable without introducing ratings yet

### Phase 2: Rated Shared-Run Arena Support

Add:

- explicit rated shared-run arena policy
- identity validation for rated encounters
- export fields for rated vs unrated encounter counts

Goal:

- support season-ready shared-run arena summaries

### Phase 3: Open-World Policy Extensions

Add:

- explicit policies for which open-world encounters may be promoted to rated
- clearer NPC/system identity exclusion rules

Goal:

- keep casual world interactions visible without overrating them

### Phase 4: Season Standing Integration

Add:

- season summary export fields
- observer panels for eligible/excluded encounter counts
- ladder-facing summaries derived only from eligible encounters

Goal:

- connect shard eligibility rules to real season summaries and standings

## 17. Recommended Guardrails

To keep ladder/season logic aligned with benchmark-first integrity:

- never mix official benchmark results into shard season standings
- never rate encounters without replay and integrity evidence
- default open-world interactions to unrated unless policy explicitly promotes
  them
- preserve excluded encounter visibility for audit/debugging
- keep identity validity a prerequisite for rated participation

## 18. Bottom Line

The right model is:

- official benchmark eligibility for benchmark claims
- shard-rated eligibility for bounded, integrity-backed competitive shard
  encounters
- shard-unrated eligibility for visible but non-competitive world activity
- excluded-invalid for encounters that cannot be trusted

That keeps persistent-world seasons and ladders concrete, auditable, and
compatible with MUDBench’s current replay, export, identity, and arena-summary
surfaces.
