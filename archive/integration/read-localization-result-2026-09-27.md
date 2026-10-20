# Read-side localization: result

September 27, 2026 (run 2). References #973 under #820. OpenCode/DeepSeek lab.
**Status: executed; predeclared outcome MIXED, with a post-hoc refined signature
(READ_RANKING) at the stated scope.** Read-only; no training, no weight change.

Plan and frozen thresholds: [`read-localization-plan-2026-09-27.md`](read-localization-plan-2026-09-27.md).
Instrument: `crates/uor-r4-training/examples/joint-read-localize.rs` +
`crates/uor-r4-training/src/read_localize.rs` (registration in `lib.rs`; no other shared
change). Evidence: [`docs/evidence/read-localization-2026-09-27.json`](../evidence/read-localization-2026-09-27.json).

## What was executed

Frozen parent `fit-quaternion-6/checkpoint-final` (step 15,672), read mode Enabled,
deterministic greedy with the frozen stop rule, the 32-row source panel under five
declared conditions: A baseline; B entity-final; C matched non-entity final; D
near-query entity mention (generation target unchanged); D′ near-query matched
non-entity mention. Decision-0 read evidence: the full read mass row over causally
available occurrences with token identity, NoRead mass, copy gate, effective copy mass,
entity span/mass/share/rank, and the vocabulary/copy mixture.

**Guardrails:** baseline parity `PARITY_EXACT` on all 32 rows (prompt ids, generated
ids, stop, text, both verdict flags, and decision-0 read fields against the retained
parent `story-probes.json`); masses normalize in every evaluated row; all reads causal;
all spans resolved and unambiguous; mutations checked against their intended decoded
text; one D/D′ row declared length-mismatched (`15|edited`). Full run 1.29 s in-process
(smoke 0.2 s) — evaluation-scale.

## Result by class (complete answers per condition)

| Class | Rows | A | B | C | D | D′ |
|---|---|---|---|---|---|---|
| Distractor copy (`clouds.`) | 5 | 0/5 | 0/5 | 0/5 | **5/5** | 0/5 |
| Correct noun + extra phrase | 5 | 0/5 | 0/5 | 0/5 | 3/5 | 4/5 |
| Morphology (`pears.`) | 1 | 0/1 | 0/1 | 0/1 | 0/1 | 0/1 |

## Predeclared outcome: MIXED

The frozen READ_ACCESS_LIMITED clause requiring the entity share < 5% with rank > 5 in
at least four distractor rows **failed** (0 of 5: the entity was present at rank 2–3
with 16–32% share). Its third clause also failed: condition D raised the entity
share/rank in 5 of 5, but the matched control D′ did so in 5 of 5, so the frozen
**share-or-rank** disjunction is non-discriminative — the entity *share* alone did
separate (D 39–64% vs D′ 1.4–4.2%), but the disjunction also accepted rank ≤ 3 and the
sparse D′ read row attained rank 3. STATE_EMISSION failed because D changed the verdict
in 5 of 5. Per the frozen rule the outcome is **MIXED**; the rule is not relaxed.

## Post-hoc refined signature (not the predeclared verdict)

In the five condition-A distractor rows the emitted token equals the **read top-1 token**
(` clouds`), while the correct entity is a readable event at rank 2–3 (share
0.16–0.32). Adding a near-query **entity** mention (D) moves the entity to rank 1–2
(share 0.39–0.64) and completes 5/5; adding a matched **non-entity** mention (D′) makes
the model emit the inserted noun (` carrot`) in 5/5. At this scope the distractor
failures are consistent with a **read-ranking** effect (the learned age/recency prior
prefers the later `clouds` occurrence over the correct entity 50–70 tokens back), not
entity absence. Two qualifications from the independent audit are carried here: the
top-read-equals-emitted identity is asserted only for the five condition-A distractor
rows, because the copy path aggregates per-token read mass into the vocabulary row so
emissions are not per-occurrence top-1 readouts in general (counterexample: under D, row
`04|edited` emits ` bear` while the top single occurrence is ` clouds`); and D changes
recency, duplication and local context together, so the flip shows ranking sensitivity
but is also consistent with continuing the locally inserted clause.

Two further observations bound this: the extra-phrase class is **not** entity-specific
(D 3/5 vs D′ 4/5), consistent with the earlier termination-weighting INERT result; and
the single morphology row emits the correct singular noun at rank 1 yet fails the
completion, so it is a separate emission/morphology issue.

## Decision and next step

Instrument retained; nothing promoted. The decision-relevant finding is the read-ranking
signature for the distractor class, with the caveat that D makes the entity the most
recent noun, so the flip demonstrates recency/ranking sensitivity but not that a
long-range ranking repair is learnable. Recommended successor (bounded,
evaluation-scale): an **oracle read re-rank intervention** — clamp the decision-0 read
mass onto the entity occurrence and check whether the distractor rows flip — which would
causally confirm the read ranking as the sole bottleneck before any mechanism change.

## Instrument and artifact notes (independent audit)

- **Executed artifact:** the `joint-read-localize` release example, sha256
  `1c1c589d5f61d560add1f2f86c429aa593fd1688370fc34f7a3c01b6ae86c8a9`, preserved in the
  model store `binaries/`; the module and example source hashes recorded in the report
  match the delivered files, and the retained `story-probes.json` sha
  (`0b58b65b…`) was verified. The executed binary's embedded source commit is the plan
  commit; the delivered commits are plan `62163592`, instrument `aac22690`, docs
  `604b0e16`.
- **Mutation construction:** the condition clauses are tokenized and spliced by token
  id; each row records `decoded_matches_intended` and `length_change`, and one row
  (`15|edited`) is declared length-mismatched between D and D′. This is a declared
  divergence from the plan's literal "never re-tokenize" wording, mitigated by the
  per-row decoded-text check.
- **Causality guardrail:** the report carries a static `no_future_reads_enforced`
  declaration; causality is enforced by construction (read occurrences are validated
  against `written_occurrence`) and the `positions_are_causal` helper is unit-tested, but
  the report does not carry a measured per-row flag. Recorded as a declaration.
- **Superseded attempts preserved:** `read-localization-opencode-1` and
  `read-localization-smoke-opencode-1` (earlier binary, before the D/D′ length flag).
- **Audit verdicts:** baseline parity, class counts and the MIXED adjudication
  CONFIRMED; the post-hoc signature CONFIRMED with the two corrections above. Most
  likely way this evidence could be wrong: the D/D′ flip may reflect local recency/
  bigram copy rather than proving the long-range read ranking is the bottleneck.

## Limits

Exposed 32-row development panel, one parent, one deterministic policy; no fresh
holdout, no training, no weight or serving change. The predeclared rule returned MIXED;
the READ_RANKING signature is a post-hoc refinement at this exact scope. D makes the
entity the most recent and repeated noun, so the flip demonstrates ranking sensitivity
but not that a long-range ranking repair is learnable, and it is also consistent with
local clause continuation. It does not qualify general language or a mechanism family.

## Cost

Instrument implementation, focused checks, smoke and full run are charged once with the
milestone at delivery; the run itself is 1.29 s of model time. A prospective ledger
extension (+12,000,000 ms; limit 756,000,000 ms) was recorded before compute under the
standing owner authorization.

## Addendum: oracle read re-rank (director's Step 1, executed)

Read-only, decision-0-only intervention on the same parent, greedy, the five condition-A
distractor rows. Implemented as an opt-in, in-memory, never-serialized read-mass
intervention (`Swap` / `Focus`) in `joint_model.rs` — default `None` is byte-identical;
dense Full admission, reads enabled, batch 1 only, else the step errors — plus
`examples/joint-oracle-rerank.rs`. Baseline parity with the retained panel is exact on the
five rows; the run is 0.50 s.

Arms: **S** = swap the entity occurrence's read mass with the top-1 distractor's;
**C** = swap the top-1 distractor's mass with a matched non-entity occurrence
(`n` minimises `|rank(n)-rank(e)| + |age(n)-age(e)|`, ties to the lower occurrence);
**O** = put all non-NoRead read mass on the entity.

| Arm | Complete |
|---|---|
| baseline | 0/5 |
| **S** (entity <-> top-1 distractor) | **5/5** |
| C (matched non-entity control) | 0/5 (`garden.`) |
| O (entity only, upper bound) | 5/5 |

The per-row before/after masses prove each intervention applied to exactly the intended
columns (`intervention_applied_exactly` true). Example, row `12|edited`: under S the
entity `bag` goes 0.1496 -> 0.3924 and `clouds` 0.3924 -> 0.1496, and the model emits
`bag.`; under C the top mass moves to `garden` (0.0187) and the model emits `garden.`.

**Frozen reading (director), applied: S >= 4/5 and C <= 1/5 -> the read ranking is
sufficient for this class.** The bottleneck is *which* event the read ranks first — not
entity absence and not emission. **Reported to T1** (learned age prior vs content): the
learned age/recency prior outranks a correct, content-relevant occurrence 38-54 tokens
back, so a ranking/age-prior change is the mechanism to test; another read-score family
is not indicated (Lorentz/affine and the signed-2I score are dead/parked). Scope: five
authored distractor rows, one parent, greedy, decision 0 only. No follow-up panel.
