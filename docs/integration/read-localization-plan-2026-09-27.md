# Read-side localization: predeclared plan

September 27, 2026 (run 2). References #973 under #820. OpenCode/DeepSeek lab.
**Status: predeclared design; no compute executed by this document.** Thresholds and
conditions are frozen before the instrument run.

## Objective

Localize the mainline learner's measured source-answer failures to the read/state/
emission seams. The decision it must change: which mechanism the next milestone
implements — read selection, state/update, or emission/morphology. No training, no
weight change; evaluation-scale only.

## Measured failure context (frozen parent)

Parent `fit-quaternion-6/checkpoint-final` (step 15,672), frozen 32-row source panel,
deterministic greedy, frozen stop rule. Parent completes 21/32. The 11 failing rows:

| Failure class | Rows | Generated |
|---|---|---|
| Distractor noun (wrong entity) | `04\|original`, `04\|edited`, `08\|original`, `12\|original`, `12\|edited` | `clouds.` (5) |
| Morphology (right entity, wrong inflection) | `00\|edited` | `pears.` (1) |
| Correct noun then extra phrase | `03\|original`, `07\|original`, `07\|edited`, `11\|edited`, `15\|edited` | e.g. `shoe home.`, `ribbon to his father.` (5) |

**Every distractor failure is a template-0 prompt**, whose distractor noun `clouds`
occurs *later* than the entity mention (`watched the clouds move across the sky` vs
`carried her apple into the garden`). The entity always occurs 38–48 words before the
end. This predicts a recency-weighted read selecting `clouds` over the entity. The
`pears.` rows are an inflection failure, not a read failure, and are reported separately.

## Read semantics that constrain the design (verified in source)

`JointStep` returns `read_masses` over events written strictly **before** the current
token (`core_step` reads `0..previous`, then writes the event). At the step that emits
the first continuation token (the step consuming the final prompt token), the read
candidates are prompt tokens `0..L-2`; **the final prompt token is not readable**. The
packet's literal "prompt ends with the entity" therefore makes the entity a
*state/candidate*-path probe, not a read probe. The design below keeps both, with the
read probe as the primary endpoint.

## Conditions (all 32 rows, deterministic greedy, frozen stop rule; no weight change)

| # | Condition | Purpose |
|---|---|---|
| A | **Baseline** frozen prompt (entity early) | primary read-side evidence |
| B | **Entity-final** (literal packet mutation: final token replaced by the entity span) | state/candidate path only (entity excluded from read by construction) |
| C | **Matched non-entity final** (same slot replaced by a concrete non-entity noun) | B control |
| D | **Near-query entity mention** (insert a short clause containing the entity shortly before the final query, e.g. `…across the sky. Lily saw her pear. When it was time…`), generation target unchanged | primary read-access probe: entity readable near emission |
| D' | **Near-query non-entity mention** (identical clause with a matched non-entity noun) | D control |

Mutations are built on **token ids**, never by re-tokenizing a string. The entity span is
found exactly by scanning minimal token windows whose decoded bytes (leading ASCII
whitespace stripped, preceding byte a word boundary) equal the entity noun; unresolved
spans exclude the row and are reported. Length changes from mutations are declared
per row and matched between D and D' where tokenization allows.

## Endpoints (per row and condition)

1. Generation: prompt token ids, generated token ids, decoded response text, stop
   reason, and the frozen completion/first-noun verdict. Baseline A must reproduce the
   retained parent verdicts (parity gate).
2. Read side at decision 0: full read mass row over causally available occurrences
   (occurrence index, token id, decoded token text, mass), NoRead mass, copy gate,
   effective copy mass; entity span, entity mass and mass share among non-NoRead mass,
   entity rank, top-5 events; and (decisively) whether the emitted wrong noun equals the
   top-read token.
3. Vocabulary/copy mixture for the entity token and the emitted token (available from
   the existing read-only trace helpers).
4. Per-class transition counts across conditions (distractor rows and morphology rows
   reported separately).

## Predeclared decision rule

Read-side entity share = entity occurrence mass / (1 − NoRead mass).

- **READ_ACCESS_LIMITED** if, among the five distractor rows: the entity share is
  < 5% with rank > 5 in at least four, AND in at least four the emitted wrong noun equals
  the top-read token (`clouds`), AND condition D raises the entity share to ≥ 15% (or
  rank ≤ 3) in at least three while D' does not. → next milestone targets read
  selection/recency (with a declared position/age feature change, not a score re-fit).
- **STATE_EMISSION** if the entity share is ≥ 10% or rank ≤ 3 in at least four
  distractor rows while the output stays wrong, and D/D' do not change the verdict.
  → next milestone targets the state update / emission mixture.
- **MIXED** if neither threshold is cleanly met; report exact counts and the rows that
  break each pattern.
- **INSTRUMENT_INVALID** if baseline A does not reproduce the retained parent verdicts,
  spans are unresolved for > 2 rows, or any prompt exceeds the 256 context.

The `pears.` morphology rows and the extra-phrase rows are reported separately and do
not enter the read-access count.

## Guardrails

- Baseline parity with the retained parent `story-probes.json` verdicts (all 32 rows).
- The condition-D insertion must not move the final query tokens; the generation target
  (accepted noun + period) is identical for D and A.
- Determinism: identical replay of one row; masses + NoRead sum to 1 ± 1e-5; no future
  reads (occurrences < written).
- No model-math change. The instrument is a read-only example; any needed additive
  public helper is default-inert and must not change existing behavior.
- One model process at a time; check for other labs' running fits before each run.

## Budget

Prospective: instrument implementation and focused checks ≤ 1.0 h; the run (32 rows ×
5 conditions, ≤32 generated tokens each, prompts ≤ 70 tokens) ≤ 0.2 h of model time;
analysis, evidence, independent audit, documentation and protected delivery ≤ 1.0 h.
Total ≤ 2.2 h elapsed; model compute minutes. A prospective ledger extension
(+12,000,000 ms; limit 756,000,000 ms) was recorded before compute per the standing
owner authorization. Stop/checkpoint rather than crossing the updated limit; retain the
128 MiB physical storage stop margin.

## Scope

Exposed development data, one parent, one seed-free deterministic policy.
This localizes one failure class on this artifact; it does not qualify prose, general
language, or a mechanism family. No promotion, no weight or serving change.
