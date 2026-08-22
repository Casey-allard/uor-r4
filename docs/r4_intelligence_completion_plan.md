# R4 Intelligence Completion Plan

- **Status:** Authoritative for post-v0.1 intelligence sequencing (adopted 2026-08-19, issue #829).
- **Source of record:** GitHub tracker root #820 and its native sub-issues. Where this
  document and the live GitHub hierarchy disagree, **the native parent/sub-issue tree on
  GitHub wins**; this file is the readable, linkable mirror maintained alongside it.
- **Relationship to earlier plans:** This document supersedes, for *post-v0.1 sequencing
  only*, the phase ordering in `docs/r4_graph_compiler_implementation_plan.md` and the
  `.github/DOCUMENTATION_OVERHAUL_PLAN.md`. It does **not** retract or rewrite any prior
  engineering plan or measurement record — those remain valid history and, where noted,
  valid engineering references. Claim language in this document follows
  `docs/formal_vocabulary.md` (normative), and `python3 scripts/check_claim_wording.py`
  gates it in CI.

## Why this plan exists

UOR-R4 reached a reproducible v0.1 engineering baseline: it compiles a pinned Hugging Face
teacher into a multiplication-free, table-native artifact with a witnessed integer runtime,
serves it, and ships it as a verified release. What it does **not** yet have is a
capability-sequenced research programme that promotes each intelligence claim only against
replayable evidence at the exact execution scope claimed.

The programme deliberately corrects several stale premises carried in earlier planning prose
(all corrections are append-only against the historical records; the live summaries are
reconciled, the records are not rewritten):

- **#784 refuted full-depth context-code collision**, reporting **0.0% full-depth
  context-code collisions at k = 2/4/6/8**. The observed effect is **continuation-distribution
  convergence** — distinct context codes still favor a shared continuation (11/15 distinct
  rows still favored newline) — *not* a code collision. Earlier prose that framed #784 as a
  "context/code collision" is superseded.
- **#811 wired the deployed D4 abstention policy into the CLI ask path**, but all five
  semantic out-of-distribution probes remained `SERVABLE`. **D4 parity is not semantic
  answerability**; semantic unanswerability detection is *not* established. This is a
  measured substrate property, the same family as #784.
- **#605/#804 made the real route-attention arm executable and then closed it with a
  negative S1 verdict** (2026-08-19): the pre-registered anti-vacuity null was not cleared
  (teacher attention supports are temporally smooth at this scale; 119/120 heads individually
  vacuous), so the instrument licenses nothing. The `R4RouteAttentionV1` operator stays
  **dormant** behind its unchanged `model/ledger.toml` gate. W(3,3) remains a separately
  qualified research hypothesis, **not** the graph-migration path.
- R4G1 is already multiresolution and patch-capable. **R4G2, network composition, and
  larger-scale work are trigger-gated**, not assumed next steps.
- Seeded sampling restored valid surface output, but has **not** established prompt causality
  or coherent free-running generation.

## Goal

Produce a CPU-first, deterministic, bounded, multiplication-free deployed system whose prompt
conditioning, selective prediction, free-running behavior, typed-state planning, instruction
behavior, scale, composition, and release claims are each backed by replayable evidence **at
the exact execution scope claimed**.

## Programme hierarchy

Native sub-issues are the source of truth. The root tracker (#820) has eight stage trackers
plus one cross-cutting formal foundation. Each stage tracker owns its bounded native
children, its milestone, its promotion gate, and its final verdict.

| Lane | Tracker | Title | Milestone |
|---|---|---|---|
| **F0** | #859 | Lean formalization of the stable R⁴ prime-router mathematical kernel | (cross-cutting; runs during S0) |
| **S0** | #821 | Truth baseline, conformance, and normative inference closure | R4 Intelligence / S0 |
| **S1** | #822 | Persistent prompt-conditioned predictive state | R4 Intelligence / S1 |
| **S2** | #823 | Evidence-grounded selective prediction and calibrated abstention | R4 Intelligence / S2 |
| **S3** | #824 | Coherent free-running generation and trajectory correction | R4 Intelligence / S3 |
| **S4** | #826 | Learned semantic planning and geometry qualification | R4 Intelligence / S4 |
| **S5** | #825 | Native instruction behavior and production delta covers | R4 Intelligence / S5 |
| **S6** | #827 | R4G1 v1 scale, streaming, and local composition | R4 Intelligence / S6 |
| **S7** | #828 | Proof/release closure and distributed composition | R4 Intelligence / S7 |

### S0 native children (this stage's ordered work) — tracker #821

1. #829 — Publish the completion plan and evidence-first issue forms *(this document)*
2. #830 — Encode execution scope, serving reachability, and non-vacuous verdicts
3. #831 — Designate the normative R4G1 scorer
4. #832 — Commit CID-bound suites and per-token attribution
5. #833 — Rebuild the attested broad bundle and re-ratify the baseline

Stage order, then the parent tracker's listed child order, governs what is worked next.
A later stage's preparatory child does **not** displace the next sequential stage item
merely because it lacks a direct blocker. Cross-cutting foundation work (F0) may proceed in
parallel but does not displace the next sequential stage item unless the roadmap says it
gates that item.

### S0 stage verdict — PROMOTE (2026-08-20)

All five S0 native children are closed with recorded verdicts and every promotion-gate
condition on tracker #821 is met, so **S0 is promoted** and its downstream stages S1 (#822)
and S2 (#823) are unlocked. This entry is the readable mirror of the tracker's closure
verdict; **GitHub #821 remains the source of record**.

- **All children have final verdicts.** #829 (PR #860, `d1d1f735`), #830 (PR #861,
  `bc5a2a92`), #831 (PR #862, `1c377df7`), #832 (PR #863, `70586aa0`), and #833 (PR #867,
  `1b3e46f2`; determinism/provenance chain #864 `9c1f2c10`, #866 `19465441`) all closed
  COMPLETED.
- **One normative scoring path, differentially tested.** The deployed R4G1 runtime scoring
  path is designated the single normative scorer in [ADR-0001](adr/0001-normative-r4g1-scorer.md)
  (#831); spec-vs-deployed differential checks and a planted-negative control live in
  `crates/uor-r4-graph-certify/tests/normative_scorer_831.rs` (record:
  `normative_scorer_831.md`).
- **Frozen benchmark identities and attribution.** CID-bound capability suites and per-token
  resolution attribution land in `crates/uor-r4-api/src/capability_suite.rs` with committed
  manifests (#832; record: `capability_suites_832.md`).
- **Attested post-#755 broad bundle admitted.** A source-complete, #755-native,
  byte-reproducible bundle (`smollm2-360m-broad-clean`) passes deployed R4Engine admission and
  the offline prediction canary (#833; record: `attested_broad_baseline_833.md`).
- **M.V.G. thresholds — RETAINED with dated evidence.** Gate C RATIFY (RETAIN), 2026-08-20:
  the headline causal serving rows reproduce within the predeclared `<~0.5 pp` reachability
  bound (Rule 1+2 24.30% → 24.39%, best-live 31.48% → 31.11%, TLA-3 28.21% → 28.12%). The
  teacher floor (3.6015 bits) is RETAINED by invariance; a full-population re-measurement is
  **UNAVAILABLE** at scale under the current exact-GEMM teacher path (~24 h for the held-out
  split) and is a recommended follow-up, not a promotion blocker.

The stage kill/redesign criterion is **not** triggered: the claimed signals reproduce from
pinned source inputs through the deployed path (byte-reproducible rebuild, admitted bundle,
Gate C within bound). Downstream quality, geometry, and scale promotions are therefore
authorized to proceed from this baseline.

### S1 stage verdict — REVISE (2026-08-21)

All S1 child work is resolved and the pre-registered kill/redesign criterion is met, so
**S1 is revised**: the prompt-conditioning claim is not established at the current
representation, and the stage returns to representation/compiler redesign. This entry is
the readable mirror of the tracker verdict; **GitHub #822 remains the source of record**
(maintainer decision comment, 2026-08-21).

- **All children have final dispositions.** #835 spec frozen (PR #872, `8785aedd`); #834
  closed REVISE with the five-arm space fully dispositioned (PRs #873/#874/#875/#891,
  final merge `77d09023`); #836 closed REVISE/dormant after end-to-end lowering;
  follow-ups #886 (deployed lowering-fidelity gap) and #887 (the 20‰ bar stands; the
  segment lane is retired from the promotion track) done; #888 closed not-planned.
- **No arm cleared the frozen 20‰ causal floor.** `current-scoring` and
  `longer-local-context` measured negative; `persistent-state` positive-sub-floor
  off-serving, then retired after lowering; `conditional-residuals` unconditionally
  sub-floor with its distinctive mechanism falsified (CR-vs-Ψ −1.3‰; the
  residual-shuffle null lands at the real arm); `candidate-support-expansion` closed off.
- **The kill/redesign criterion fired** — two-plus independently motivated arms fail
  causal controls — per the tracker's pre-registration.
- **Redesign process gate.** A redesign RFC (design only, no code) — candidate mechanisms
  against the measured falsifiers (suffix-locality of scoring *and* generation
  (#874/#891/#894); singleton-margin pollution and the discarded content-answerable
  novelty (#893)), pre-registered re-entry gates (**the 20‰ causal floor and the #838
  selective-prediction gates do not move**), and cost estimates — goes to the maintainer
  for approval before any build. #840's corrective rounds are held for the approved
  direction (decision recorded on #840, 2026-08-21).

### S1 redesign execution and re-entry decision (2026-08-21, follow-up)

The approved RFC's steps 1–3 are executed and recorded
(`docs/s1_redesign_instruments_822.md`; CID-bound result JSONs; PRs #898 → `9d2dbe19`
and #899 → `201a747a`); **GitHub #822 remains the source of record** (§6-4 decision
comment, 2026-08-21).

- **Q1 answered:** the recorded `t_argmax` labels condition on the full article prefix
  hard-capped at 128 positions (`--sequence-length 128`; mean prefix 62.7 tokens; 85.4%
  of stories truncated at the cap) — the measured suffix-locality is a compiled-key
  property, not an observation artifact, and the label-side conditioning ceiling is 128
  tokens, not the teacher's native 8,192.
- **D1 = `SELECT (backed-off mix)`:** joint (content-token, 2-token-suffix) conditional
  evidence with Ψ-bag backoff measured **+30.6‰ (paired 95% CI [+28.6, +32.5])** — the
  first arm in the S1 record whose paired lower bound clears the frozen 20‰ floor AND
  the 25‰ lowering-track opening bar, off-serving. The strict joint arm is
  sparsity-bound (+6.8‰); the d4skip comparison arm (1-token conditioning, denser
  support) measured +49.6‰ [46.8, 52.4]; both planted nulls collapse (prompt-swap
  −33.2‰, key-shuffle −116.3‰).
- **D2 = `REVISE`:** region-conditional evidence keyed by the released artifact's own
  graded codes measured region[4] **+19.5‰ [17.0, 21.9]** — real and code-specific
  (code-shuffle null −162.5‰) but sub-floor at the pre-registered consultation; the
  #784 convergence null is refined rather than confirmed (only 10.3% of covered
  region tables argmax to the corpus mode); the exploratory depth-2 sweep reads
  +23.3‰ [21.0, 25.7] (floor-clearing, below the opening bar).
- **§6-4 re-entry decision (recorded on #822):** the lowering track **#897 is
  ACTIVATED** on the D1 SELECT trigger; its phase 0 is a pre-registered key-family
  confirmation run (1-token-conditioning mix as PRIMARY; the 25‰ bar plus a
  head-to-head requirement against the selected 2-token arm); **D3 stays deferred**
  (revisited only if the lowered arm's end-to-end measurement hits the 128-token label
  ceiling); the **frozen 20‰ end-to-end floor remains the promotion gate**. S2's #839
  phase-1 typed surfaces proceed per its re-scope.

### S1 stage verdict — PROMOTE (2026-08-21)

The lowering track cleared the frozen 20‰ end-to-end causal floor, so **S1 is
promoted** and S3 (#824) is unlocked. This entry is the readable mirror of the
tracker verdict; **GitHub #822 remains the source of record** (PROMOTE decision
comment, 2026-08-21). It does not retract the REVISE entry above — the redesign
happened, and this is its resolution.

- **The D1-selected lowering re-entered and cleared the gate.** #897 (PR #903)
  lowered the confirmed 1-token skip-mix scorer onto the deployed `R4Engine`,
  shipped dormant after a spot-check fidelity gap; #904 (PR #905) diagnosed the
  gap as breadth-bound; #906 (PR #907) fixed it with candidate injection
  (deployed fidelity 58/87, clearing the 60% bar); #908 (PR #909) measured the
  deployed **end-to-end causal** effect at **+28.45‰ [25.57, 31.32]** — the
  paired 95% lower bound clears the 20‰ floor — with the
  conditioning-specificity (label-shuffle) null collapsing to −236.95‰
  (`docs/skipmix_endtoend_causal_908.md`, result_cid `blake3:e32e4e33…`).
- **Real against the deployed base.** As pre-registered, the off-serving +56.2‰
  (toy suffix-rate base) overestimated the deployed delta; the lowered lane
  still clears the floor against the real deployed base.
- **Frozen gates held.** The 20‰ causal floor and the #838 selective-prediction
  gates did not move (the #887 discipline); D3 stays deferred (the end-to-end
  measurement did not hit the 128-token label ceiling).
- **Activation.** The promoted lane is turned on — the deployed serving decision
  routes through the lane, the compile path fits and emits the SKMX/PSIB
  sections, and the capability is registered as **RF-31** — under #910.

### S2 stage verdict — REVISE (2026-08-21)

Items A and B are closed and item C is re-scoped, so **S2 is revised**: abstention
semantics are frozen and honest, no calibrator qualified, and one redesigned re-entry is
sanctioned under the same frozen gates. This entry mirrors the tracker verdict;
**GitHub #823 remains the source of record** (maintainer decision comment, 2026-08-21).

- **#838 (item A).** The typed eight-status selective-prediction contract and the
  `s2-answerability-ood` benchmark constitution are frozen (PR #892, `61501c8f`);
  current semantic abstention is recorded **NOT ESTABLISHED**.
- **#837 (item B).** `NO CALIBRATOR ESTABLISHED` (PR #893, `d1215f55`) — no arm met the
  release (false-answer UCB95 ≤ 10‰ at coverage ≥ 20‰) or research (≤ 50‰ at ≥ 50‰) gate
  on the calibration partition; the untouched test partition was never consumed. The
  diagnosis is evidence acquisition, not thresholds: the fitted support×margin bucket
  table ranks confidence cleanly but its memorized slice is too thin; raw margin is
  singleton-polluted; 2,454 content-answerable novel positions are structurally invisible
  to suffix features.
- **#839 (item C) — re-scoped (2026-08-21).** Phase 1 (executable now) lowers the frozen
  contract's legacy-coverage mode onto the production surfaces: typed abstention causes
  on CLI/native HTTP, the OpenAI-compatible migration from `engine_declined` to the typed
  `uor_abstention_*` / `uor_incompatible_artifact` codes, the typed WASM boundary, and
  fail-closed corrupt-calibration handling. Phase 2 (calibrated mode) activates only when
  a redesigned calibrator clears the unchanged release gate.
- **Re-entry rule.** One redesigned calibrator fit (content-side evidence features,
  aligned with the S1 redesign direction) under the **same** frozen gates; the bars do
  not move (the #887 discipline).

### S3 stage verdict — LIMIT (2026-08-22)

All three native children are resolved and the pre-registered kill/redesign criterion is
met, so **S3 is limited**: coherent free-running generation is **not established** at the
current representation scope, and the deployed artifact is recorded as a teacher-forced
retrieval/continuation system rather than a generative engine at this scope. This is a
limiting (non-promotion) verdict — it narrows the claim and does **not** promote a
generation claim into any downstream stage. This entry mirrors the tracker verdict;
**GitHub #824 remains the source of record** (maintainer decision comment, 2026-08-22).

- **#841 (item A).** The free-running gap is quantified and total (PR #894, `11d7ca8e`):
  greedy, deployed `R4Engine`, prompt-family v1 (n = 100, H ∈ {8, 32}) — median
  first-divergence step **0**, **99/100** free-running rollouts token-identical to
  suffix-only rollouts (generation is memoryless beyond the 2-token suffix), **710‰**
  cycle collapse, **0‰** typed abstentions. The corrective §6 bar is frozen on these
  numbers (median first-divergence +≥2 steps AND diverged-at-0 −≥100‰ per round, no
  teacher-forced-agreement regression >10‰, ≤3 rounds, else `GENERATION-NOT-ESTABLISHED`).
  Records `docs/free_running_eval_841.md`, `docs/free_running_841_result.json` (result
  `blake3:59ec2f60…`).
- **#840 (item B).** `GENERATION-NOT-ESTABLISHED` (do-not-launch) (PR #912, `35f8cc87`):
  the corrective student-prefix run is not launched because the reachable ceiling is
  ~100× below the frozen §6 bar (correctable footprint ~1‰ vs the required 100‰
  diverged-at-0 drop) and the promoted skip-mix representation (RF-31) *regresses*
  free-running (suffix-locality 99→19 of 100, cycle collapse 710‰→1000‰). The limiting
  factor is the representation, not more same-distribution data or decoding. Records
  `docs/free_running_reachability_840.md`,
  `docs/free_running_reachability_840_result.json` (result `blake3:e9f48e20…`).
- **#842 (item C).** `NOT TRIGGERED` / `GENERATION-NOT-ESTABLISHED` (PR #913, `abcad4e5`):
  a predeclared, non-vacuous diagnostic (n = 100, greedy, deployed `R4Engine`,
  teacher-free) classifies every free-running failure — survived 0 · single-step-at-0 59 ·
  candidate-gap 16 · rank-limit 25 · **state-starvation 0**. **0/100** failures are
  addressable by bounded trajectory state (state-addressable 0‰; step-0 invariance and the
  teacher-prefix upper bound both fail the frozen §6 bar; `ceiling_clears_bar = false`).
  Records `docs/state_trajectory_gate_842.md`,
  `docs/state_trajectory_gate_842_result.json` (result `blake3:86376394…`).
- **The global falsifier fired.** Bounded student-prefix correction (#840) and bounded
  trajectory state (#842) both fail to reduce the frozen free-running gap, so the programme
  global falsifier classifies the deployed artifact as a teacher-forced
  retrieval/continuation system, not a generative engine at this scope. No runtime, format,
  or compiler behavior changed across the chain (RF-14/21/22/29 evaluation infrastructure;
  `CONFORMANCE.md` unchanged).
- **Downstream consequence.** Closing #824 clears its native-blocker edge to S4 (#826), but
  S3 is `LIMIT`, not `PROMOTE`, and #826's entry criteria require promoted S2 and S3
  verdicts; with S2 (#823) at REVISE (its #839 phase-2 calibrated mode trigger-gated and
  unsatisfied) and S3 limited, **S4 remains gated** and this verdict promotes nothing
  downstream. Generation re-entry requires a representation carrying cross-step free-running
  memory, re-measured under the unchanged #841 §6 bar and the frozen #838
  selective-prediction gates (the #887 discipline).

### S4 entry reconciliation — AMEND + PROCEED (2026-08-22)

S3 (#824) closed `LIMIT` and S2 (#823) stands at `REVISE`, so #826's written entry
criterion ("S2 and S3 have promoted verdicts") is superseded and its native blocked-by edge
to #823 is converted to a sanctioned parallel-start. This entry mirrors the tracker
decision; **GitHub #826 remains the source of record** (maintainer decision comment,
2026-08-22).

- **Amended substrate.** S4 builds on the established — not promoted-generative — substrate:
  S1 prompt causality is promoted (RF-31, +28.45‰ [25.57, 31.32]); S2 selective prediction
  is frozen and honest (REVISE, no calibrator, #839 phase-2 independently trigger-gated); S3
  is `LIMIT` (teacher-forced retrieval/continuation, no free-running generation at this
  scope).
- **Self-contained gate.** #826's promotion gate is unchanged and never depended on
  free-running generation: typed-state planning must beat retrieval-only,
  memorized-trajectory, shuffled-state, and direct-continuation baselines on held-out
  entities/compositions/topologies/relabelings/horizons, with any geometry gain shown under
  equal bytes/candidates/operations. Planning results are not read as free-running coherence
  (the #824 boundary stands).
- **Parallel-start + order.** S4 may begin in parallel with S2's open re-entry; #839 stays
  parked under its trigger. Item order: #844 (freeze benchmarks + typed state/action
  semantics) → #843 (bounded transitions with plan witnesses) → #845 (geometry vs
  equal-budget baselines) → #846 (certify and bound the claim). Frozen gates unchanged (20‰
  causal floor, #838 selective gates, #841 §6 bar for any generation re-entry). Next
  executable issue: **#844**.

### S4 item A closed; item B active; benchmark Amendment A1 (2026-08-22)

Readable mirror of the GitHub state; **#826 and #844 remain the source of record**.

- **Item A #844 — CLOSED.** Froze the compositional-planning benchmark constitution and the typed
  reference model, and registered RF-32, across four pull requests (#916 design, #917 constitution,
  #918 reference model, #919 RF-32; main `3a4801c4`). It established a falsifiable target and a
  byte-level meaning, **not** reasoning performance.
- **Item B #843 — ACTIVE.** Design contract frozen in
  [`docs/bounded_semantic_transitions_spec_843.md`](bounded_semantic_transitions_spec_843.md); six
  increments (design freeze, benchmark repair, induction, packed sections, deployed planner,
  measurement and verdict).
- **Amendment A1 to the #844 benchmark (appended, not a rewrite).** #843's binding cheap instrument
  ran first and returned **DEGENERATE**: 13 of the 20 frozen horizon cells were 0/512 solvable, a
  structure-keyed memorized-trajectory null saturated at valid-plan rate 1.0000 in every non-vacuous
  cell (placing the #826 promotion statistic at or below 0, against δ_min = 0.05), five of six split
  axes had a single cell, and `TaskInstance::id()` carried the generator seed so an id-keyed null
  could not fire. The bounded repair — low-horizon non-vacuity, real split-axis cardinality, a
  content-derived identity, and strongest-null headroom — is frozen as
  [`docs/compositional_planning_spec_844.md`](compositional_planning_spec_844.md) §11 and lands in
  #843's increment 2. **No frozen number changes value:** δ_min = 0.05, n = 512, H ∈ {1, 2, 4, 8},
  H_max = 16, and W_max = 64 all stand.
- **Corpus-observation arm — does not launch.** The v4 observation record carries none of the ten
  typed object kinds the planner requires, so coverage is 0.000 against δ_min = 0.05. Recorded as a
  structural source-check result at no run cost, per the long-run discipline.
- **Unchanged.** #845 stays `on-hold` until #843 records a non-degenerate baseline; #846 keeps the
  sealed partitions and the final verdict; the 20‰ causal floor, the #838 selective gates, and the
  #841 §6 bar are untouched. Next executable issue: **#843**.

### S4 item B (#843) — CLOSED = LIMITED (2026-08-22)

Readable mirror; **#843 and #826 remain the source of record**. Full measurement:
[`docs/bounded_semantic_transitions_843.md`](bounded_semantic_transitions_843.md); frozen contract:
[`docs/bounded_semantic_transitions_spec_843.md`](bounded_semantic_transitions_spec_843.md).

- **Verdict: LIMITED.** A bounded, fixed-capacity, allocation-free, P-4-only semantic-transition
  planner is **established on the deployed path** and is **lowered as the non-geometric production
  baseline** — for the **12 of 20** joint-split cells where a baseline exists to beat. Typed
  planning is **not established** on the other 8. Not promoted, not refuted.
- **The planner is at correct-outcome rate 1.0000 in all 20 cells**, including held-out topologies
  whose operator effect sets it never saw during fitting. Every failing cell is one where the
  *null* is also at or near 1.0000, never one where the arm fell short.
- **Why the 8 fail.** Their strongest null is `direct-continuation` — greedy one-step descent on
  goal distance. Symbolic transformation and counterfactual intervention are monotone toward the
  goal, so greedy is never trapped and reaches 1.0000; a bounded planner cannot show headroom over
  greedy on a task greedy already solves. Where a family does trap greedy the arm wins by up to
  **+0.2260**. This is a ceiling on the benchmark, not on the mechanism.
- **Arm lowered: `bounded-breadth-first`** (exact tie with the table-guided beam — both 1.0000 in
  all 20 cells — broken toward breadth-first because it uses no scoring heuristic and is therefore
  the plainest bar for geometry to clear). **`bounded-iterative-deepening` rejected**: under the
  equal budget it re-expands and exhausts its expansion ceiling, falling to 0.2695 at H = 8.
- **Delivered across six pull requests:** #920 design freeze and Amendment A1 (main `5f4d740f`),
  #921 the A1 generator repair (`328791ff`), #922 typed observations and deterministic induction
  (`85f3d03a`), #923 the packed `PSCH`/`PTRN`/`PGOL`/`PWIT` sections (`f5fccc51`), #924 the deployed
  planner, #925 the measurement and verdict. **RF-33 `bounded_semantic_transitions`** registered
  (`normative-runtime` / `deployed-serving`), its statement carrying the LIMITED boundary in-line;
  `CONFORMANCE.md` 32 → 33 ids.
- **#845 released, RESTRICTED.** Geometry qualification may begin, measured **only on the 12
  separating cells** against the lowered arm at 1.0000 under equal bytes, candidates, expansions and
  operations. Running it on the 8 greedy-solvable cells would compare geometry against a baseline
  already at 1.0000 and would read as a geometry failure when it is a task property.
- **#846** inherits the witness schema, the three-way differential, the planted-mutation table, and
  the sealed composition and topology cells, which #843 never opened.
- **Frozen gates unchanged.** δ_min = 0.05, n = 512, H ∈ {1, 2, 4, 8}, H_max = 16, W_max = 64, the
  20‰ causal floor, the #838 selective gates, and the #841 §6 bar all stand. The #824 and #823
  boundaries stand: nothing here is free-running coherence or a calibrated-confidence claim.
- **Requirement added for future benchmark freezes.** Beside Amendment A1's non-vacuity and
  null-saturation instruments, a **greedy-solvability probe**: a task solvable without lookahead
  cannot measure lookahead, and headroom against a memorization null is not headroom against a
  search null.

### S4 item C (#845) — ACTIVATED under Amendment A2 (2026-08-22)

Readable mirror; **#845 remains the source of record** (maintainer decision comment, 2026-08-22).
Frozen design contract:
[`docs/w33_geometry_qualification_spec_845.md`](w33_geometry_qualification_spec_845.md); benchmark
amendment: [`docs/compositional_planning_spec_844.md`](compositional_planning_spec_844.md) §12.

- **The #859 edge is converted, F0 stays held.** The W(3,3) object every geometry arm uses is
  pinned as formal-vocabulary Definitions in the #845 design contract (the symplectic quadrangle
  over GF(3): 40 points, 40 lines; canonical representatives; collinearity distance; phase). The
  dashboard's 96-vertex canvas is reconciled by disambiguation: it is a rendering motif with no
  incidence structure and is non-normative for #845. A `PROMOTE FOR LOWERING` verdict does not
  itself authorize production lowering — the lowering issue it would open must re-acquire the
  exact #859 pin first. Formal backing gates lowering, never measurement.
- **Zero-ceiling finding recorded.** The lowered RF-33 baseline is at correct-outcome 1.0000 in
  all 20 frozen cells, so no geometry arm can clear δ_min over it on the frozen primary statistic
  anywhere on the frozen grid. As released, #845's primary run could not launch under the run
  contract.
- **Amendment A2 (appended; no frozen value moves).** Two measurable axes replace the saturated
  one: **A2(a)** — paired budget-reduction at exactly preserved correctness on the 12 separating
  cells (expansions gating, ρ_min = 0.10, vs the strongest non-geometric ordering control);
  **A2(b)** — correct-outcome rate on nine probe-admitted tightened-frontier cells at H = 8 in the
  three separating families (bars 0.7422–0.8340, headroom 3.3–5.2× δ_min), with nine secondary
  cells reported non-gating.
- **The binding cheap instrument ran first and passed.** The failure-surface probe (70 cells,
  33.5 s, teacher-free) admitted 18 cells; the two greedy-solvable families admitted zero at every
  probed setting, consistent with the #843 verdict. The probe ships as a repository instrument in
  #845's build increments.
- **Restriction honored.** Any frozen-terms correctness comparison stays on the 12 separating
  cells; the 8 greedy-solvable cells are excluded from every axis; #846 keeps the sealed
  partitions and the S4 promotion verdict. Frozen gates unchanged (δ_min, n, the horizon grid, the
  20‰ causal floor, the #838 selective gates, the #841 §6 bar).

Next executable work: **#845 increments 2–4** (mapping and instruments → arms and harness →
measurement and verdict).

## Dependency order

```text
F0 ─────────────────────────> #845 (geometry qualification, S4)
 └───────────────────────────> #856 (production refinement, S7)

S0 ─┬─> S1 ─> S3 ─> S4 ─> S5 ─> S6 ─> S7
    └─> S2 ─────────────────> S5
```

- **F0 (#859)** starts during S0 but does **not** gate S1–S3. It gates experiments or formal
  claims that depend on prime-router mathematics. Its Lean theorems remain **reference
  evidence** until #856 either proves a refinement to stable deployed semantics or explicitly
  excludes the component. F0 is deliberately **not** a child or promotion blocker of #821,
  because the current f64 prime router is outside the graph-migration path.
- **S2** may proceed alongside S1, but any public generative or instruction-capability
  promotion requires the relevant selective-risk gate.
- Formal work may run continuously; **S7 closes only against stable normative bytes and
  serving semantics.**

## The two-level proof strategy (F0 now, refinement later)

Formalization is deliberately split so it delivers value early as a design/assumption audit
without overstating the current router's production relevance:

- **Level 1 — reference mathematics (#859, now).** A pinned, importable Lean 4 library for
  the **stable pure mathematical kernel** of the R⁴ prime router. Its top-level target is a
  totality/determinism/boundedness/witness-replay theorem over declared configurations and
  inputs. It explicitly does **not** assert semantic quality, intelligence, collision
  freedom, Riemannian-geodesic status, Rust/`f64` equivalence, production reachability, or the
  Riemann hypothesis. Every precondition and external authority stays explicit. This is
  **reference-only** evidence (**Definition**/**Assumption** scope), never described as
  deployed-serving evidence.
- **Level 2 — production refinement (#856, later, S7).** A separate, later proof layer that
  either refines the Level-1 boundary to stable deployed semantics (Rust/`f64` or
  packed-runtime refinement) or explicitly **excludes** the component from the release claim.

### Proof-asset boundaries (do not conflate)

These are distinct artifacts with distinct scopes. A claim citing one must not borrow another's
authority:

| Asset | What it is | Scope / claim boundary |
|---|---|---|
| **R4 prime-router Lean package** (#859 deliverable) | New Lean 4 library for the pure reference kernel of the prime router | Reference mathematics of the *f64 router kernel*; **not** deployed-serving evidence, **not** the graph path |
| `research/riemann-lean` | Separate conditional Prime/Riemann bridge, Lean 4.28, supplied theorem/provider boundaries | Independent research; **not** a router specification or implementation refinement; absent from root CI |
| `proofs/wasm-gemm-gnaf` (vendored GNAF) | Lean 4.30 proof that a reference WASM GEMM/GNAF kernel is cost-optimal (#653) | Proof-**process** methodology is reusable; its theorems do **not** transfer to R4 serving or text generation |
| `crates/uor-r4-proof-model` | Rust **executable** proof obligations + proof-status matrix + a small **Kani** surface | Structural/`ExecutableSpec` obligations over the deployed runtime and format; not a Lean formalization of the router |
| Kani harnesses (`kani_proofs.rs`) | Bounded model-checking of score arithmetic safety and fixed-capacity container invariants | **Structural** guarantees on specific runtime obligations only |
| Hologram/R4 monograph (`docs/hologram_r4_formal_monograph.md`) | Normative specification and evidence map | Specification/evidence map; **not** a Lean theorem library |
| Target-binary evidence | The actual release binaries and reachable production call graph | The only evidence that supports **deployed-serving** claims (S7 / #856) |

### Mechanism boundaries (distinct components, distinct roles)

| Component | Role | Status |
|---|---|---|
| **W(3,3)** visualization / dashboard field | 96 rendered vertices in the browser dashboard; a research/visualization construct | **Not** a normative construction of the 40-point/40-line W(3,3) quadrangle; **not** an input or output of the Rust router; separately qualified research hypothesis |
| **Geometric prime router** (`crates/uor-r4-router`) | Validated `f64` content-query retrieval/routing (MRR 0.88+, #486/#490/#502) | Real, load-bearing for retrieval; runs on `f64` **outside** the P-4 kernel by design; strengthens the product, is not itself the product |
| **R4G1 graph scoring** (`uor-r4-core` `score.rs` / R4G1 adapter) | The compiled-artifact scoring path | On the normative production path; the S0 designation of *one* normative scorer is #831 |
| **Route-attention reference work** (`R4RouteAttentionV1`, #604/#605/#804) | P-4-legal integer attention analog | **Dormant**; S1 verdict FAIL (instrument vacuous); wired into no serving path; ledger-gated |
| **Normative production path** | The deployed graph runtime + one designated normative scorer | The only surface whose evidence is credited as deployed-serving |

## Programme invariants

- The deployed hot path remains **XOR/AND/OR/shift/rotate/popcount/integer add-sub/compare/
  table-read only**: no multiply, divide, or floating point.
- Prediction remains **allocation-free in steady state**, bounded, deterministic, `no_std`
  where currently normative, and free of recoverable-path panics.
- **One normative scorer** owns production serving semantics, certification, patch
  interpretation, witness replay, and proof targets.
- Identical pinned inputs produce identical artifact bytes (byte reproducibility). Corpus,
  teacher, tokenizer, compiler, benchmark, decoder, and report identities are content-bound.
- Reference, offline/compiler, certifier, dormant portable-runtime, normative runtime, and
  deployed-serving evidence are **never** conflated.
- Empirical gates use document-disjoint data, powered samples, non-degenerate nulls, negative
  controls, uncertainty, and predeclared outcome branches. Unavailable fixtures are
  `UNAVAILABLE`, never a vacuous PASS.
- Long runs obey the repository run contract (reachability arithmetic → binding cheap
  instrument → distinct positive/negative next actions; see `AGENTS.md`).
- Negative results narrow or retire claims; they are first-class completion evidence and do
  **not** silently promote downstream stages.
- This programme does **not** claim exact teacher equivalence, does not claim human-level
  reasoning, and does not treat plausible language output as evidence of coherent internal
  state transitions (`docs/formal_vocabulary.md` §2.1, §5).

## Promotion gates and kill/redesign criteria

Each stage tracker records its own promotion gate and kill/redesign criterion; the
authoritative text lives on the tracker issue. The global falsifiers that bind every stage:

- If two independently motivated conditioning mechanisms fail on document-disjoint,
  EXCT-disabled causal tests, revisit the representation/compiler model rather than continuing
  code-space tuning.
- If bounded student-prefix correction does not reduce the frozen free-running gap, classify
  the artifact as a teacher-forced retrieval/continuation system rather than a generative
  intelligence engine.
- If planning gains disappear under relabeling, unseen composition, or topology changes,
  classify them as memorization.
- If a geometry arm cannot beat Hamming/binary/VSA/spectral controls under equal bytes,
  candidates, and operation budgets, keep it out of the production runtime.
- If a claim cannot be reproduced from pinned inputs, or its serving reachability cannot be
  shown, it remains unavailable or reference-only.

## Definition of completion (root #820)

- The stable prime-router mathematical kernel has a kernel-checked theorem/assumption
  boundary, and every release component either refines that boundary or explicitly excludes it.
- Prompt meaning exerts measured causal influence beyond suffix/NGRAM and exact-context memory.
- Unsupported answers are declined at a predeclared risk/coverage operating point while
  answerable novelty remains served.
- Teacher-forced improvements transfer to coherent free-running trajectories across a frozen
  horizon ladder.
- Typed-state planning generalizes across held-out entities, compositions, topologies,
  relabelings, and horizons.
- Any promoted geometry beats non-geometric controls under equal bytes, candidates, and
  operation budgets.
- Instruction behavior survives unseen templates and does not erase base capabilities.
- Compilation and inference satisfy declared memory, latency, artifact-size, and
  deterministic-composition envelopes.
- Each guarantee has a scoped proof status and artifact; each empirical claim has a CID-bound,
  replayable report.
- The actual release binaries and reachable production call graph support the advertised
  operation contract.
- A complete bundle installs and serves offline from an empty model root on every supported
  target.

## Repository conformance

Every child declares its execution scope and maps to existing RF capability IDs (RF-01…RF-29)
or justifies a new capability ID. A new **built** capability lands in this order:

`model/ids.toml` row → tagged Gherkin → failing marker/behavior test → implementation →
regenerated `CONFORMANCE.md`.

`CONFORMANCE.md` is generated and is **never** edited directly. Dormant mechanisms remain
`open` in `model/ledger.toml` with explicit activation gates. Measurements that revise a claim
append a per-issue evidence record (`docs/<topic>_<issue>.md`) and reconcile README, ROADMAP,
RESEARCH, lifecycle/configuration, and ledger assertions.

## Historical milestone disposition

The repository carries **two distinct milestone eras**, and they must remain distinguishable.
The twelve `Phase 0`–`Phase 11` milestones belong to the original **graph-compiler engineering
plan** (`docs/r4_graph_compiler_implementation_plan.md`). The eight `R4 Intelligence / S0–S7`
milestones belong to **this** post-v0.1 programme (#820). The naming already separates them
(`Phase N` vs `R4 Intelligence / SN`); this table records the maintainer-visible disposition of
each historical milestone. **Titles are not repurposed**, and no measurement history is
rewritten. As of 2026-08-19 every historical milestone has **0 open issues**.

| # | Historical milestone | Issues | Disposition |
|---|---|---|---|
| 1 | Phase 0: Baseline and contracts | 6 closed | **Completed / historical.** Baseline, R4G1 RFC, D1–D3 delivered. Superseded for sequencing by #820. |
| 2 | Phase 1: Packed graph format and trusted parser | 3 closed | **Completed / historical.** `uor-r4-graph-format` shipped. |
| 3 | Phase 2: Overlapping multiresolution regions | 11 closed | **Completed / historical.** Cover induction / multi-membership shipped. |
| 4 | Phase 3: Semantic anti-degeneracy | 3 closed | **Completed / historical.** |
| 5 | Phase 4: Transitions and residual emission | 9 closed | **Completed / historical.** ScoreQ residuals / bounded top-K shipped. |
| 6 | Phase 5: Allocation-free graph runtime | 7 closed | **Completed / historical.** `no_std` allocation-free runtime + witness replay shipped. |
| 7 | Phase 6: Boolean routing synthesis (trigger-gated) | 1 closed | **Historical, trigger-gated / not fully triggered.** Remains dormant until its D5 trigger; not on the current sequential path. |
| 8 | Phase 7: Hardware-aware packing (trigger-gated) | 1 closed | **Historical, trigger-gated / not fully triggered.** Deferred to S6/S7-era scale work where relevant. |
| 9 | Phase 8: Long-context state | 0 issues | **Superseded for sequencing.** Long-context intelligence work is re-homed under S1/S6 of #820. |
| 10 | Phase 9: Epochs and patches | 1 closed | **Historical.** R4G1 patch chains shipped; further composition re-homed under S6 (#827). |
| 11 | Phase 10: Formal verification | 1 closed | **Historical; continued under F0/S7.** Executable spec + Kani + proof-status matrix exist; formal work continues under #859 (F0) and #828 (S7). |
| 12 | Phase 11: Architecture acceleration | 0 issues | **Superseded for sequencing.** SIMD/accel is a trigger-gated S6/S7-era concern, not next-up. |

These milestones are **retained** (not deleted, not renamed) so their closed-issue history stays
addressable. Their live descriptions carry a one-line disposition banner pointing here.

## Public entry points (all link here)

This document is the single canonical plan; the following entry points link to it rather than
duplicating a drifting phase table:

- `AGENTS.md` — operating manual (post-v0.1 sequencing points here; the graph-compiler plan
  remains the engineering plan).
- `ROADMAP.md` — product capability checklist (intelligence sequencing points here).
- `README.md` — the roadmap/plan link section.
- `docs/RESEARCH.md` — measured/refuted/open research (forward sequencing points here).
- `docs/r4_graph_compiler_implementation_plan.md` — carries a status banner: retained as the
  original graph-compiler engineering plan, superseded only for post-v0.1 sequencing.
- `.github/DOCUMENTATION_OVERHAUL_PLAN.md` — marked superseded.
- Issue forms: `.github/ISSUE_TEMPLATE/research-experiment.yml` and
  `.github/ISSUE_TEMPLATE/implementation.yml` encode the run/conformance contracts below.

## Evidence-first issue intake

Two GitHub issue forms encode the repository's run and conformance contracts so new work carries
its evidence discipline from the start:

- **Research / experiment** — hypothesis; pinned identities (corpus/teacher/tokenizer/
  compiler/decoder CIDs); reachability arithmetic; null and negative control; the binding cheap
  instrument and its required verdict; predeclared thresholds and distinct positive/negative
  outcome branches; cost estimate; evidence-record path; and claim status.
- **Implementation** — execution scope; dependencies/blockers; acceptance criteria; non-goals;
  compatibility/migration; conformance mapping (RF IDs; the build order above); verification
  (the four local gates + applicable ladder); documentation reconciliation; and claim status.

## Closure rule

A stage closes only when every native child has a recorded **completed, negative,
not-triggered, or explicitly descoped** verdict and the stage promotion decision is posted with
exact evidence links (`PROMOTE`, `REVISE`, `LIMIT`, or `RETIRE`). The root #820 closes only
after #859 has a completed or explicitly narrowed formal-foundation verdict, all eight stage
verdicts are recorded, and the final capability/open-limit record is published.
