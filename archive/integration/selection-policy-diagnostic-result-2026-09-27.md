# Same-checkpoint selection-policy diagnostic result

**Scope.** Executes the [predeclared plan](selection-policy-diagnostic-plan-2026-09-27.md),
the supported next work card of the
[emission/selection diagnostic](emission-selection-diagnostic-result-2026-09-27.md):
a deterministic (greedy) replay at the same step-15,672 checkpoints. Read-only —
no weight, serving-decoding, gate, panel or candidate change. The frozen prose
verdict stands; nothing is promoted, and a better-ranked token is not treated as
a coherent alternate trajectory unless actual generated text says so.

## Instrument and identity gate

A new additive `joint-selection-replay` subcommand replays the unchanged greedy
story probes and the five frozen prompts at the same checkpoints. Executed source
`18d5df51` (parent `6ce8d6e1`→`e3191a83`); binary sha256 `f13d463c…`; rebuilt and
re-run after each rebase — including the concurrent Lorentz-attention merge — so
the delivered source is the executed source.

- **The greedy source panel reproduces the retained `story-probes.json`
  exactly**: `PARITY_EXACT`, 0 mismatches in both arms over every compared field
  (per pair/side flags, response text, generated ids, stop, and per-decision
  selected/greedy tokens, probabilities and hashes; the comparer enforces exact
  equality, including f64 values). A pre-rebase run (`-1` roots, `4f71ea71`)
  produced identical results after removing per-generation timing fields; the
  post-Lorentz-merge run (`-3`, source `18d5df51`) is again identical, so the
  diagnostic is stable across the intervening lab merge.
- The independent evidence auditor recomputed the retained counts and confirmed
  the comparer covers the claimed fields; it found one documentation error in the
  plan (corrected below) and no replay defect.
- Counts (identical to retained): quaternion **21/32** complete, **26/32**
  first-noun, 8/16 pairs; householder pair **23/32**, **31/32**, 10/16.

## Greedy free-prose result

All five greedy trajectories diverge from the retained sampled trajectories at
generation indices `[0, 0, 2, 1, 0]`; the sampled witnessed spans
("… called out in.", "… so exciting!") do **not** recur under greedy.

Principal read against the frozen four criteria (resolvable entity/role;
intelligible progression connected to the prompt; understandable literal
language; complete ending at a stable point), independently cross-checked by the
evidence auditor, which returned identical verdicts:

| Arm | Story (seed) | Verdict | Note |
|---|---|---|---|
| Quaternion | 0 (2014) | FAIL | speaker/role loops; truncated mid-clause |
| Quaternion | 1 (2015) | PASS | resolved (friends found, played together); heavy repetition noted |
| Quaternion | 2 (2016) | FAIL | self-eating bear; entity collapse |
| Quaternion | 3 (2017) | PASS | ride/give-up arc resolved; repetitive loop |
| Quaternion | 4 (2018) | FAIL | malformed repetition ("the train and the train"); truncated |
| Householder pair | 0 (2014) | FAIL | repetition loop; truncated mid-clause |
| Householder pair | 1 (2015) | FAIL | excitement loop; lion/lions role confusion |
| Householder pair | 2 (2016) | FAIL | goose/goose collapse; truncated mid-clause |
| Householder pair | 3 (2017) | FAIL | truncated mid-clause (incomplete ending) |
| Householder pair | 4 (2018) | FAIL | repetition loop; truncated mid-clause |

Acceptability: **quaternion 2/5, householder pair 0/5**. Six of ten outputs stop
at the 128-token cap mid-clause; if the "length-cap stop alone does not fail"
clause were read as excusing the complete-ending criterion, the counts become
quaternion 4/5 and householder 4/5 — but malformed constructs persist (the
self-eating bear; the excitement loop), so the positive branch's
"no witnessed malformed clause" condition is not met under either reading.

## Greedy source-panel result (not selection-explainable)

The replay's failing rows are all incomplete completions — **11 (quaternion) /
9 (householder pair)** — with two modes: correct noun followed by an extra phrase
instead of the required period stop (5 / 8, i.e. 13/20 overall) and a wrong first
noun (6 / 1, i.e. 7/20; mostly `clouds.` and `pears.`).

Versus the accepted September25 parent (`accepted_standalone_integer_read`; also
`rounding`: 28/28 complete quaternion, 30/24 householder, per the retained
[row adjudication](../evidence/language-continuation-source-comparisons-2026-09-26.json)):
complete rows **lost 9 / gained 2** (quaternion) and **lost 6 / gained 5**
(householder pair); first-noun lost/gained 5/2 and 1/2. Versus rung1: lost 7/0
and 3/3. The concrete regression target is the lost set (9 and 6 rows), not the
full failing list.

**Correction.** The plan cited `5/2` and `7/4` as this panel's lost/gained. The
audit verified those are the new F32-hard artifact's complete-row figures; the
continuous panel's are `9/2` and `6/5`. The plan carries a corrigendum; the
predeclared rule, criteria and cost are unchanged. Because these completions are
greedy, no sampling explanation applies.

## Decision (predeclared rule applied)

Outcome: **negative/mixed — not a selection-policy positive.** Neither arm
reaches the predeclared ≥3/5 acceptability bar (strict: 2/5 and 0/5; lenient
cap-reading: 4/5 and 4/5 but with malformed constructs persisting). Per the rule,
**the supported next target is ranking/emission**, with the concrete regression
rows and the two failure modes above as the record. No fresh selection-policy
experiment is supported by this packet; no weights, serving decoding, gates or
candidates change, and nothing is promoted.

## Limits

- Exposed five-prompt packet, one checkpoint, greedy as a diagnostic policy
  rather than a serving candidate.
- Criterion-4 cap sensitivity recorded explicitly (6/10 cap-truncated); both
  readings reported with the per-story notes and independent agreement visible.
- The sealed report binds paths, argv and source commit but not checkpoint,
  evaluator or binary hashes; those identities are recorded in the
  [evidence record](../evidence/selection-policy-diagnostic-2026-09-27.json)
  (recommendation for the next instrument revision).
- The prior diagnostic's 15–16 copy-dominated decisions are untouched; the
  integer lane is out of scope.

## Cost and delivery

- Focused release check 2.8 s; tests 3/3 (release test build 2 m 55 s); post-rebase
  rebuilds 17.9 s and 2 m 42 s; witness runs ~1 s each arm (three sets across the
  rebases).
- Report roots `selection-replay-{quaternion,householder_pair}-{1,2,3}` (≈0.4 MiB
  each); SSD target 1.4 GiB (no internal-disk target); no fit; no external
  compute. Complete elapsed charged once to the shared ledger; receipt
  `ledger-selection-replay-delivery-1.json` in the continuation investigation
  directory.
- Delivered through a protected PR referencing #973 and #820; the owning issue
  and run handoff carry the outcome, limits and next action.
