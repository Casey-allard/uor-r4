# Free-running trajectory-gap evaluation — frozen contract and first quantification (#841)

- **Status:** Frozen evaluation contract + executed run-1 (item A of S3 tracker #824,
  programme #820). The S3 tracker explicitly sanctions preparing this first offline gap
  instrument once the S0 evaluation is frozen; S3 **stage** closure remains gated on the
  #822/#823 stage verdicts, which this record does not touch.
- **Date:** 2026-08-21.
- **Claim language:** follows `docs/formal_vocabulary.md` (normative). This record
  establishes the current teacher-forced ↔ free-running gap and its diagnostic categories.
  It does **not** claim generation coherence and prescribes no corrective mechanism.
- **Harness:** `crates/uor-r4-api/tests/free_running_eval_841.rs` (fixture teeth
  non-ignored; `free_running_gap_run_841` ignored/bundle-gated). Record:
  `docs/free_running_841_result.json` (CID-bound).
- **Relation to prior records:** binds the #832 report constitution and the #833 canonical
  bundle; drives the normative deployed `R4Engine` path (the released `graph/score.r4g1`,
  the ADR-0001 scorer, the D4 policy). S2's typed-decline vocabulary (#838) is represented:
  a trajectory-terminating abstention is a first-class typed outcome, never an error.

## 1. Problem and scope

Gate C, teacher parity, and corpus replay evaluate teacher-forced or recorded positions;
they do not measure how errors compound under the system's OWN prefixes. This contract
freezes a sequence-level evaluation comparing **matched** teacher-forced and student-prefix
trajectories on identical prompts, positions, and identities, locating the **first
divergence** and attributing every generated step.

**Definition (execution scope).** Offline evaluation against the normative deployed
`R4Engine` generation path (#830: `certifier-instrument` over the `deployed-serving`
surface). Reference/offline evidence is not credited as deployed generation; nothing here
changes serving behavior.

## 2. Prompt families and horizons (versioned, frozen)

**Definition (prompt-family v1).** One prompt per held-out story: the first position whose
in-story window is full (8 tokens, the deployed `WINDOW`) and whose story continues for at
least `H_MAX = 32` recorded positions; the first 100 qualifying stories, in story order.
The family is CID-bound (the position list's bytes) and pinned in every report. Family v1
spans the *continuity* domain (real Simple-Wiki narrative). Future versioned families
extend to assistant behavior, evidence use, constraints, and multi-turn state **without
renumbering v1** — a report names exactly one family version.

**Definition (horizon ladder).** The frozen ladder is `{8, 16, 32, 64}`; run-1 executes
`{8, 32}`. Horizon weights for any aggregate across rungs are equal weights (declared);
run-1 reports rungs separately and aggregates nothing across them.

**Definition (decode modes).** Greedy (the deterministic `predict_decision` policy path;
run-1) and seeded-sampled (contract-defined: a pinned seed list per prompt, the deployed
sampled-selection path; **UNAVAILABLE in run-1** — no seeded-sampling driver identity is
pinned yet, and the record says so; a sampled run must pin decoder + seeds and re-use the
same family/horizons). Greedy and sampled are reported separately, never merged (#832
comparability rule).

## 3. Trajectory sides and trace schema

**Definition (matched sides).** For each prompt (position `i0`, window `P`):

- `teacher-prefix` — for `k < H`: the engine predicts from the RECORDED window at
  `i0 + k`; agreement is scored against the recorded teacher argmax `t_argmax[i0+k]`.
- `student-prefix` — from `P` the engine consumes its own served tokens (the last-8
  window); a typed abstention terminates the trajectory (the honest deployed behavior —
  termination is recorded, not hidden).
- Controls (§5): `shuffled-prompt`, `repeated-prefix`, `suffix-only`.

**Definition (trace schema v1).** Every step of every trajectory records `(token |
abstain, path ∈ {exact-context, ngram, graph, decline}, widened)` — the #832
`ResolutionPath` served subset from the engine's observable `(ScoreStatus, ngram_hit,
widened)` signals — under a versioned, canonical, deterministic byte serialization with a
blake3 CID. **Guarantee (round-trip and tamper). Status: Structural**
(`trace_schema_round_trips_and_is_deterministic`): canonical bytes are stable, parse
round-trips, and a corrupted tag or truncation is a typed rejection, never a panic.
Streaming vs non-streaming surface note: the deployed decision path consumed here is the
same engine call the serving tiers wrap, so a fixed decode yields the same token/state
trace on either transport (#839's certification will bind that claim to the transports
themselves; here it is an **Assumption** for the wrapped surfaces).

## 4. Deterministic primary metrics

All primaries are integers (counts, steps, ‰) computed from the trace and the recorded
story text — no judge, no float aggregation:

- **first-divergence step** — the first step whose served token departs the recorded
  continuation `input[i0+1 ..= i0+H]`; a terminal abstention reports its step (termination
  IS departure); `None` = survived the full horizon on-text.
- **matched teacher-forced agreement** — served token = recorded teacher argmax, over the
  `teacher-prefix` side's matched steps.
- **serve / abstain / widen counts** and the per-step **path histogram**
  (exact-context / ngram / graph / decline).
- **repetition/cycle** — the maximum whole-period repeat count of any trailing cycle of
  period ≤ 4; a trajectory is *cycled* at ≥ 3 repeats.
- **distinct-1** (‰ of served) and **prompt-content overlap** (‰ of served tokens that
  occur in the prompt window — the deterministic prompt-relevance proxy; entity/topic
  retention and contradiction against gold annotations join when a gold-annotated family
  version exists, and are **UNAVAILABLE** until then).
- **termination step** (abstention index, when any).

**Definition (secondary judge).** Grammatical validity and semantic-divergence judgments
require a versioned judge; a report may carry judge metrics ONLY under a pinned judge
identity, and without one they are **UNAVAILABLE** — never a vacuous pass
(`judge_metrics_without_a_pinned_judge_are_unavailable`). Run-1 pins no judge.

## 5. Controls (each non-degenerate, each with a distinct reading)

- `shuffled-prompt` — rotate-by-one derangement of the prompt window (order destroyed,
  bag preserved): separates order-sensitive state from bag effects; must change at least
  one rollout (asserted at run scale).
- `repeated-prefix` — the last token repeated to fill the window: the pure-repetition
  reference profile the cycle metric must flag.
- `suffix-only` — the last 2 tokens only: the memoryless floor. **The suffix-locality
  statistic** — the fraction of prompts whose student rollout is IDENTICAL to the
  suffix-only rollout — extends the #874/#834 suffix-locality finding from scoring to
  generation.
- `teacher-prefix` doubles as the decoder-only control (same decoder, recorded prefixes):
  the FR-vs-TF contrast isolates prefix regime from decode policy.
- Planted negatives (fixture, CI-run): an **early-drift model** (distinct, plausible,
  wrong from step 0) reads first-divergence = 0 — distinctness alone cannot pass; a
  **repetition-only model** is flagged by the cycle metric and collapsed distinct-1;
  shuffle/root/state-shuffle analogs and the emission-shuffle live at the engine boundary
  and are represented by the prompt-level controls above in run-1 (a state-shuffle control
  becomes meaningful when a stateful mechanism exists to shuffle — none is deployed; that
  control is **UNAVAILABLE** by construction today and says so).

## 6. Frozen primary statistic, power, and the corrective stopping rule

**Definition (primary statistic).** The **median first-divergence step at `H = 32` under
greedy on prompt-family v1**, with an order-statistic 95% CI (Binomial(n, ½) ranks,
integer arithmetic). `n = 100` prompts gives a rank CI of ±10 around the median —
resolving step-scale differences of ~2+ against the corrective bar below.

**Definition (corrective-round stopping rule — frozen BEFORE any corrective fitting,
for #840/#842).** A corrective round counts as an improvement only if, on the SAME frozen
family/horizon/decoder: the median first-divergence step rises by ≥ 2 steps AND the
diverged-at-step-0 fraction falls by ≥ 100‰, with no teacher-forced-agreement regression
beyond 10‰ and no selective-risk regression (#838 gates). At most **3** corrective rounds;
otherwise the #824 kill criterion fires and `GENERATION-NOT-ESTABLISHED` is recorded.
Prompts/horizons are never changed after observing corrective results (#841 non-goal).

## 7. Repository conformance

**Definition (RF mapping).** Evaluation infrastructure extending RF-14, RF-21, RF-22,
RF-29 evidence language; no new built capability, no `model/ids.toml` row, no
`CONFORMANCE.md` regeneration. Fixture absence is `UNAVAILABLE`, never PASS (RF-29): the
run skips typed when the bundle is absent, and the record marks sampled-mode and judge
metrics UNAVAILABLE explicitly.

## 8. Run-1 results (2026-08-21; greedy; the numbers live in the CID-bound record)

**Empirical Criterion (the gap, run-1). Status: Empirical.** See
`docs/free_running_841_result.json` (corpus, family, per-trace and result CIDs inside) and
the #841 issue thread for the posted run contract and the outcome table. Headline
readings recorded there: the median first-divergence step at H=32, the step-0 divergence
fraction, the survived-full-horizon fraction, termination/abstention and cycle fractions,
the teacher-forced agreement over matched steps, path histograms per side, and the
suffix-locality statistic (FR rollouts identical to suffix-only rollouts). Interpretation
and the S3 diagnostic categories are summarized in `docs/RESEARCH.md`; raw traces are
CID-bound and replayable (greedy determinism double-run asserted in-harness).

## 9. Claim status and next action

This record freezes the gap evaluation and quantifies run-1. It does **not** establish
free-running coherence, does not activate any corrective mechanism, and does not move any
stage gate. Next: #840 compiles student-prefix corrections under the §6 stopping rule
(native-blocked on this issue; stage-gated on #822/#823 for closure); a sampled-mode run
and any judge-bearing run must pin their decoder/seed/judge identities and re-enter under
this same contract.

## 10. Append-only execution-scope correction (2026-08-24, #933)

The frozen protocol, trace bytes, and run-1 measurements above remain the historical #841
record. A later ADR-0001 call-graph audit established that the harness called
`R4Engine::predict_decision`, so its trajectory evidence is a **certifier instrument at
reference/off-serving scope**, not evidence that the normative `R4G1Runtime` production
selector or any deployed transport emitted those tokens. In particular, the descriptions in
the opening metadata and §1 that call this the normative deployed `R4Engine` path are
superseded by this correction; they are retained only to preserve what the record said when
the run was executed.

**Empirical Criterion.** The reported run-1 gap continues to quantify the exact recorded
`R4Engine` trajectories on the pinned #833 fixture. Status: **Empirical** at
reference/off-serving execution scope.
It does not establish production reachability, free-running coherence, or a result for
`R4G1Runtime`. A future production-scope rerun must bind the exact normative selector,
release envelope, decode identity, and transport reachability rather than inheriting #841's
scope.
