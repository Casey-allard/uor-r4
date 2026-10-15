# #843 — compiler/runtime: learn and execute bounded semantic transitions with replayable plan witnesses (item B of #826)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-19T21:39:18Z
- **Updated:** 2026-08-22T17:38:35Z
- **Closed:** 2026-08-22T17:37:32Z
- **Labels:** enhancement, graph-compiler
- **Milestone:** R4 Intelligence / S4 — Semantic planning and geometry
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/843
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Parent tracker: #826

## Context

Sibling item A defines intervention-resistant tasks and reference semantics. The next question is whether corpus/instruction observations can induce useful transitions and whether a fixed-capacity table-native planner can execute them through the normative serving path.

## Problem

A planner can appear successful through retrieval leakage, an oracle-like compiler, unbounded search, or reference-only floating point. Packed semantics may also drift from the certifier or production engine.

## Goal

Compile deterministic state/action/transition evidence, compare bounded planning algorithms under equal budgets, lower the selected non-geometric baseline, and produce replayable production plan witnesses.

## Scope

- [ ] Define observation records for states, applicable actions, preconditions, effects, goals, constraints, evidence/provenance, and negative/conflicting examples.
- [ ] Induce/deduplicate transitions with content-addressed samples, deterministic partitioning, ordered reductions, explicit confidence/support, and held-out split enforcement.
- [ ] Compare bounded breadth/depth/frontier, table-guided, and retrieval/direct-continuation baselines under equal artifact bytes, expansions, candidates, and operations.
- [ ] Specify a fixed maximum horizon/frontier/state/action capacity and deterministic overflow, conflict, no-plan, and low-confidence decline.
- [ ] Pack selected states/actions/transitions/goals/indices into validated optional sections with backward behavior.
- [ ] Execute only P-4 integer/table operations in the #831 normative R4Engine path with caller-owned scratch/state.
- [ ] Emit witnesses containing initial state, goal/constraints, considered actions, transition evidence, chosen path, score/ties, terminal verification, and decline reason.
- [ ] Add reference/packed/production differentials and planted transition/index/witness mutations.
- [ ] Measure intervention generalization and resource curves by horizon on the frozen sibling benchmark.

## Non-goals

- Unbounded search or heap growth on the hot path.
- Use the benchmark verifier, hidden gold path, teacher, or network during deployed planning.
- Claim geometry value; this issue establishes the strongest non-geometric production baseline.

## Dependencies and entry gate

Blocked by sibling item A and promoted S1/S2/S3 semantics. Requires #831 normative scorer. A long fit requires a posted run contract.

## Acceptance criteria

- [ ] Compiler outputs are deterministic and contain no evaluation leakage.
- [ ] Selected packed planner is bit-equivalent to reference semantics on valid fixtures and fails closed on corrupt/incompatible data.
- [ ] Witness replay independently validates each transition and terminal goal.
- [ ] Production path meets fixed horizon/frontier, bytes, latency, P-4, allocation, no_std, and determinism budgets.
- [ ] Interventional performance beats retrieval/direct/memorized/shuffled baselines under the frozen rule or the negative verdict is recorded.

## Verification

- [ ] Observation/induction determinism, leakage, duplicate/conflict, and reordered-shard tests.
- [ ] Format validation/fuzz, reference-packed-production differential, and planted semantic mutations.
- [ ] Workspace no-fail-fast tests, clippy, fmt, no_std ladder, wasm target, deterministic rebuild.
- [ ] Allocation census, P-4 scan, bounded-expansion counters, witness replay, corrupt/no-plan/capacity negatives.
- [ ] Frozen benchmark by split axis/horizon with independent verifier recomputation.

## Run contract

- **Metric/current value:** Primary exact valid-plan/intervention statistic by horizon from sibling A; current production baseline is retrieval/direct continuation and must be measured before fitting.
- **Reachability ceiling:** Compute the fraction of tasks representable by observed state/action vocabulary and reachable within the fixed horizon/frontier; no planner can exceed that representational/reachability ceiling.
- **Pinned identities:** Benchmark/generator/splits, observation corpus, compiler/inducer, planner configs, artifact formats, normative scorer, decoder if textual rendering is measured, binaries, raw traces, reports.
- **Nulls/falsifier:** Retrieval only, direct continuation, memorized trajectory, shuffled states/actions/transitions, trivial/shortest-path reference, equal-byte/equal-expansion planner arms.
- **Binding cheap instrument:** Coverage/reachability table, transition-conflict rate, leakage scan, and small reference planner must be non-degenerate and exceed the effect floor before packed/full fitting.
- **Exit rule:** Select/lower only an arm whose interventional lower confidence bound beats the strongest equal-budget non-oracle baseline and meets representational and runtime budgets across the frozen horizon progression.
- **If positive:** Freeze it as the non-geometric planning baseline and allow sibling geometry bake-off C plus final certification D.
- **If negative:** Publish the limiting coverage/induction/planning diagnosis and keep reasoning NOT ESTABLISHED; do not use geometry to skip missing semantics/evidence.
- **Cost estimate:** Post compile/run wall time, peak RSS/storage, artifact bytes, expansions/operations, teacher/offline evidence cost, and shared hardware.

Post exact values and pass the cheap instrument before a full run. Controls must be non-degenerate; unavailable observations are `UNAVAILABLE`/`null`.

## Compatibility and migration

Use versioned optional sections; absent sections preserve historical behavior. Any API response/witness extensions are additive or explicitly versioned. Unsupported artifact eras fail with typed errors.

## Repository conformance

- **Execution scope:** Offline graph compiler plus normative deployed R4Engine planning path; “reasoning” in public claims is limited to the exact typed state-transition/planning tasks measured.
- Preserve applicable P-4, zero-allocation steady state, deterministic bytes/reductions, fixed capacities, bounded work, typed decline/errors, no_std, and replayable witnesses.
- Normative math uses the formal vocabulary; empirical criteria name split axes, horizons, n, uncertainty, identities, and status.
- **Conformance mapping:** RF-01, RF-08, RF-12, RF-13, RF-27, RF-28 and a new planning capability only if built.. Existing reference IDs are not deployed-planning evidence.
- Built capability order: IDs TOML → tagged Gherkin → failing marker/behavior test → implementation → regenerated conformance.
- Unselected geometry/planner arms remain reference/open or are retired with explicit gates; generated conformance is never hand-edited.
- Claim-changing results append evidence and reconcile all current summaries.
- Use “typed state transitions,” “bounded planning,” and exact horizon/task language; avoid an unqualified general-reasoning claim.

## Claim status and next action

A successful compiler/reference arm is not promotion; only packed production interventional results set the capability boundary.

## Comments (5)

### comment 5380692583 — 2026-08-22T13:36:48Z — Casey-allard

## Ownership claim — #843 selected as the next executable issue

**Selected:** #843 — *compiler/runtime: learn and execute bounded semantic transitions with replayable plan witnesses (item B of #826)* — https://github.com/UOR-Foundation/uor-r4/issues/843
**Parent tracker:** #826 (TRACKER/S4) · **Milestone:** R4 Intelligence / S4 — Semantic planning and geometry

### Why this is next in the pipeline
Discovered from live GitHub state (not from any stored plan):

- **Stage.** S0 PROMOTED, S1 PROMOTED (RF-31), S2 REVISE (#823 open, #839 trigger-gated so not executable), **S3 (#824) CLOSED = LIMIT**, **S4 (#826) OPEN = the earliest stage eligible for execution** under the recorded *AMEND + PROCEED* entry reconciliation (source of record: maintainer decision comment on #826; readable mirror `docs/r4_intelligence_completion_plan.md` at main `ddd82ff2`), which sanctions the parallel start over S2's open re-entry.
- **Child order.** #826's native sub-issue list is `#844 -> #843 -> #845 -> #846`, matching the frozen item order in the plan mirror. **#844 is CLOSED** (2026-08-22, PRs #916/#917/#918/#919 to main `3a4801c4`), so #843 is the first open child in tracker order.
- **Prerequisites closed.** Sibling item A **#844 CLOSED**; normative scorer **#831 CLOSED** (2026-08-20). Upstream semantics established per the amended substrate (S1 promoted; S2 honest-abstention; S3 teacher-forced).
- **Not already being worked.** #843 was unassigned, has zero comments, no linked PR, and no branch matching `*843*` exists on `origin` (checked via `git ls-remote --heads`). Zero open PRs in the repo. Now assigned to me as the active-work signal.
- **Siblings not eligible.** #845 carries `status: on-hold` (gated on #843 producing a non-degenerate baseline); #846 is certification (must follow implementation); #826 is a tracker with unresolved children so it cannot be selected. S5/S6/S7 and their children are gated downstream. F0 #859 is parallel but HELD by the maintainer.

### Downstream dependents
#845 (geometry bake-off, unblocks only when #843 records a non-degenerate non-geometric baseline) -> #846 (certification, untouched partitions) -> S4 promotion verdict on #826.

### Expected artifacts (subject to maintainer sign-off)
- `docs/bounded_semantic_transitions_spec_843.md` — frozen design contract plus run contract.
- Compiler: typed observation records, deterministic transition induction/dedup, held-out split enforcement (`uor-r4-graph-compiler`).
- Format: new **optional** R4G1 sections for packed states/actions/transitions/goals/indices and the plan witness, absent-section identity preserved (`uor-r4-graph-format`).
- Runtime: fixed-capacity, allocation-free, P-4-only bounded planner on the #831 normative `R4Engine` path with caller-owned scratch (`uor-r4-graph-runtime`).
- Verification: reference/packed/production differentials, planted transition/index/witness mutations, allocation census, P-4 scan, no_std ladder, wasm build, determinism, plus the horizon/axis measurement on the frozen #844 benchmark.
- A new built planning capability (`model/ids.toml` -> tagged Gherkin -> failing marker -> impl -> regenerated `CONFORMANCE.md`) **only if built**, per the issue's conformance mapping.

### Assumptions carried in
1. **Boundaries inherited and not re-litigated here:** teacher-forced scoring only (#824 LIMIT), ordinal honest decline rather than calibrated confidence (#823 REVISE, RF-30 decline surface), and the frozen #844 constitution numbers (delta_min = 0.05, n = 512/cell/horizon, H in {1,2,4,8}, H_max = 16, W_max = 64, Holm-Bonferroni).
2. **A negative outcome is a legitimate completion.** Per the issue's run contract, if the cheap coverage/reachability instrument or the equal-budget comparison fails, the recorded result is the limiting diagnosis with reasoning kept NOT ESTABLISHED, not a weakened gate.
3. Increment shape and the remaining capacity/selection knobs go to the maintainer for sign-off before any build lands.

Work proceeds on `issue-843-bounded-semantic-transitions` off `origin/main` at `3a4801c4`.


### comment 5380954996 — 2026-08-22T14:39:47Z — Casey-allard

## Increment 2 of 6 — Amendment A1 lands; the binding gate now passes

PR #921 (auto-merge armed). Increment 1 merged as PR #920 → main `5f4d740f`.

### What the gate said, before and after

| A1 instrument | Before | After |
|---|---|---|
| **a** per-cell non-vacuity | 13 of 20 frozen cells at **0/512** | **1.0000** at H ∈ {2,4,8}; **0.7500** at H = 1 |
| **b** split-axis cardinality | 5 of 6 axes had **one** cell | **8** per axis; 32–64 operator compositions |
| **c** content-derived identity | seed-carrying; 4096/4096 unique over 2–7 real problems | content-derived; structurally identical instances share an id |
| **d** strongest-null headroom | **1.0000** in every non-vacuous cell | **0.2773 – 0.9355**, against a 0.95 ceiling |

The S4 promotion statistic is no longer at or below zero by construction. Per-cell tables: `docs/compositional_planning_spec_844.md` §11.5.

### How the separation was actually restored

The decisive change is that **topology varies the operator effect set**, not just the obstacle layout — drawn from a shared eight-effect pool where topologies 0–3 and 4–7 each cover the whole pool. That distinction matters: a held-out cell has to be a novel *composition*, never a novel *primitive*, or an inducer fitted on the low half would find the high half unsolvable rather than hard. With a goal-indexed layout alone, a retrieval baseline stayed at 1.0000 — the goal determined the plan.

The controls are deliberately not strawmen. They are fitted on the honest declines as well as on the plans, and they transfer plans by **effect** rather than by name or slot, so they already see through all three surface axes (entity, vocabulary, template) before the split is applied.

### A negative result, recorded rather than engineered away

At horizon 1 a retrieval baseline scores **exactly 1.0000** in four of five families, and no generator repair changes that. A one-step answer is a deterministic function of the initial state, the goal and the operator set — all observable at inference — and the fitting split must cover the whole operator pool or induction has nothing to learn from. Valid-plan rate cannot separate planning from retrieval at horizon 1; that is a property of horizon-1 tasks.

Per maintainer sign-off (2026-08-22), the H = 1 cell is gated on **correct-outcome rate** — a valid plan on a solvable instance, or a correct `Decline(no_plan)` on one with no plan inside the horizon — and a quarter of horizon-1 instances now place the goal one step beyond the horizon so that decision is real. Deciding which is which requires evaluating reachability, which is planning; a baseline that always answers cannot. On every H ≥ 2 cell, where all instances are solvable, the correct-outcome rate *is* the frozen valid-plan rate. Reasoning and the rejected alternative: §11.6.

**Tightest cell, flagged:** symbolic-transformation at H = 1, headroom 0.0645 — 1.29 × δ_min. The instrument prints a `TIGHT` marker for any cell under 2 × δ_min. That cell's reading carries the least weight when the arms are measured.

### What did not change

Every frozen constitution number stands: δ_min = 0.05, n = 512 per cell per horizon, H ∈ {1,2,4,8}, H_max = 16, W_max = 64, Holm–Bonferroni, the five families, the six axes, the six baselines, the primary metric. A guard test asserts it. `CONFORMANCE.md` is byte-identical at 32 ids, RF-32 keeps its Gherkin and all five scenarios unaltered, and no RF id is added. Sibling item A stays in its shipped state — this is new evidence appended under the append-only records rule, not a rewrite.

### Verification

`crates/uor-r4-graph-compiler/tests/amendment_a1_843.rs` — four gate instruments plus a frozen-number guard, at the frozen n = 512, deterministic, teacher-free, fixture-free, ~40 s, in the default `cargo test`. Module tests 11 → 16, all green. Full local ladder before push: fmt, clippy `-D warnings`, `cargo test --workspace --no-fail-fast`, the no_std ladder both ways, the wasm lib build, `xtask validate` (R1/R4/R5/gnaf — 32 ids, every gate passed), `repo-conformance` R2/R3 (2 + 32 pass), `cargo deny check bans` (ok), and the claim-wording gate.

### Next

Increment 3 — the typed transition observation record and the deterministic inducer (content-addressed partitioning, ordered reductions, deduplication, declared-conflict handling, ordinal support bands, held-out split enforcement, leakage scan), per `docs/bounded_semantic_transitions_spec_843.md` §3. It starts once #921 lands.


### comment 5381555795 — 2026-08-22T16:59:54Z — Casey-allard

## Session paused — five of six increments merged; the sixth is built, verified and pushed, not yet proposed

Parking this at the maintainer's request. GitHub state below is exact.

### Merged to `main`

| PR | Increment | Merge commit |
|---|---|---|
| #920 | 1 — design freeze + benchmark Amendment A1 | `5f4d740f` |
| #921 | 2 — A1 generator repair, all four gate instruments pass | `328791ff` |
| #922 | 3 — typed transition observations + deterministic inducer | `85f3d03a` |
| #923 | 4 — packed `PSCH`/`PTRN`/`PGOL`/`PWIT` sections | `f5fccc51` |
| #924 | 5 — deployed P-4 planner, allocation-free, on the `R4Engine` path | `978ecb07` |

### Built but **not** proposed

Increment 6 — the equal-budget measurement, the RF-33 registration and the verdict — is committed as `7b935aab` and pushed to **`issue-843-measurement-verdict`**. **No pull request has been opened for it**, so nothing in it is on `main` and no conformance id is registered yet: `CONFORMANCE.md` on `main` still reads 32 ids, and RF-33 exists only on that branch.

Local verification on that exact branch (cherry-picked cleanly onto `978ecb07`): `cargo fmt --check` and `cargo clippy --workspace --all-targets --all-features -- -D warnings` both green; `xtask validate` and `audit-limits` green at 33 ids; bdd 33 features / 120 scenarios / 395 steps; `repo-conformance` 33; `cargo deny check bans` ok; claim-wording gate passed. The full `cargo test --workspace` re-run on the rebased tree was still in progress when the session was paused — it had completed 152 suites with **zero** failures, and the same suite ran green end-to-end (184 suites) on the identical content before the rebase. **That last re-run is unfinished, so treat it as unverified rather than green.**

### The verdict that increment 6 records — for review, not yet in force

**LIMITED.** The deployed planner scores correct-outcome rate **1.0000 in all 20 joint-split cells**, including on held-out topologies whose operator effect sets it never saw during fitting, and clears the frozen effect floor over the strongest non-oracle baseline on **12 of 20**. Every failing cell is one where greedy `direct-continuation` is *also* at or near 1.0000 — symbolic transformation and counterfactual intervention are monotone toward the goal, so greedy is never trapped. Where a family does trap greedy the arm wins by up to **+0.2260**. That is a ceiling on the benchmark, not on the mechanism.

Arm selected for lowering: `bounded-breadth-first` (exact tie with the table-guided beam; `bounded-iterative-deepening` rejected — it exhausts its expansion ceiling under the equal budget and falls to 0.2695 at H = 8).

Full record on the branch at `docs/bounded_semantic_transitions_843.md`.

### What is deliberately **not** done

- No PR for increment 6, so **#843 stays open** and its verdict is not recorded on `main`.
- **#826 has no verdict comment** and **#845 keeps its `status: on-hold` label** — releasing it depends on a verdict that has not landed.
- The worktree `/Users/casey.allard/uor-r4-worktrees/issue-843-bounded-semantic-transitions` and its `target-843` directory are **left in place** so the work can resume without a rebuild. The branch is on `origin`, so they are recoverable if removed. `explore-matmul-fixed-a` in the main checkout is untouched.

Unassigning, since this is parked rather than complete.

### To resume

Open a PR from `issue-843-measurement-verdict` with a body ending `Closes #843` — that is the one PR in this chain where a non-empty `closingIssuesReferences` is correct. Finish the interrupted `cargo test --workspace` first, and re-run `cargo run -q -p xtask -- audit-limits` as the **last** step before pushing: a local R5 pass from earlier in a session goes stale, which is how the violation CI caught on #924 got past a local run in the first place.


### comment 5381573793 — 2026-08-22T17:03:48Z — Casey-allard

### Correction — this issue is not parked

The preceding comment (parking increment 6 as an unproposed branch) is **superseded and withdrawn**. It was posted on a close-out instruction that has since been reversed by the issue owner.

Increment 6 is being proposed as a pull request now and carried through the merge queue on the normal path. Nothing about the measured result changes — the verdict remains **LIMITED**, exactly as recorded in `docs/bounded_semantic_transitions_843.md`. Only the disposition of the branch changes: proposed and merged rather than parked.

Assignment restored to `@Casey-allard`. Definition-of-Done evidence will be posted on this issue after the merge commit is confirmed on `main`.


### comment 5381722881 — 2026-08-22T17:38:35Z — Casey-allard

## Definition of Done — #843, closed as **LIMITED**

**Outcome class.** LIMITED, not PROMOTE and not REFUTE. The issue permits positive, negative, not-triggered and retired outcomes; this is an evidence-backed positive-with-boundary, and it is recorded as such rather than rounded up.

### The claim, in the only wording the evidence licenses

> A bounded, fixed-capacity, allocation-free, P-4-only semantic-transition planner is **established on the deployed path**, and is **lowered as the non-geometric production planning baseline** for the cells where a baseline exists to beat. It clears the frozen effect floor over the strongest non-oracle baseline on **12 of the 20** joint-split cells, and is **not established** on the remaining 8.

Not a general reasoning claim. Not free-running coherence — the #824 boundary stands. Not calibrated confidence — the #823 boundary stands. The S4 promotion verdict remains #846's to make, against sealed partitions this issue never opened.

### Acceptance criteria

| # | Criterion | Status | Evidence |
|---|---|---|---|
| 1 | Typed transition observations induced without a teacher | MET | `crates/uor-r4-graph-compiler/src/semantic_transitions.rs`; 13 tests in `semantic_transitions_843.rs`. Refusal is `InductionOutcome::Refused` — **data, not an error**. |
| 2 | Rules serialised into versioned, content-bound R4G1 sections | MET | `PSCH`/`PTRN`/`PGOL`/`PWIT` in `plan_sections.rs`; 16 tests incl. the §9 planted-mutation table. Capacity headers are **checked against frozen values, never honoured**. |
| 3 | Deployed planner executes them under fixed budget | MET | `crates/uor-r4-graph-runtime/src/plan.rs`; `R4Engine::plan_bounded`; 11 tests. |
| 4 | Allocation-free steady state | MET | **0 allocations, 0 bytes** for a whole episode *and* for emitting its witness — counting global allocator, `plan_allocation_census_843.rs`. |
| 5 | P-4 only on the deployed path | MET | Source scan: **no runtime multiply, divide or float**; 6 compile-time constant expressions exempted **and printed**. State hashing is a multiply-free Jenkins one-at-a-time ARX. |
| 6 | Replayable plan witness | MET | `PWIT` is self-contained — initial state, inline goal/constraint predicates, step rows, considered rows. `replay()` returns `Valid` / `Invalid{step, fault}` / `Declined`. |
| 7 | Bounded work, typed declines | MET | Checked visited-probe bound of **16**; `size_of::<PlanScratch>()` = **81 548 bytes**. Declines are `PackedDecline`, carried as `enum Search`, not `Result`. |
| 8 | Equal-budget measurement against nulls | MET | 3 arms × 4 nulls under **one shared `PlanBudget`**; an arm exceeding the shared ceiling is reported invalid rather than compared. |
| 9 | Effect floor cleared on every required cell | **NOT MET — 12 of 20** | This is why the verdict is LIMITED. Diagnosis below. |
| 10 | Conformance registered | MET | RF-33, `normative-runtime` / `deployed-serving`. `CONFORMANCE.md` 32 → 33 ids, regenerated by `xtask check-model --write`, **never hand-edited**. |

### Why criterion 9 could not be met — and why that is a task property, not a planner failure

The deployed planner is at **correct-outcome rate 1.0000 in all 20 joint-split cells**, including held-out topologies. It did not fail anywhere.

Two of the five families are **greedy-solvable**: `direct-continuation` also reaches 1.0000 there, so the reachable headroom over the strongest non-oracle null is structurally below δ_min = 0.05. No bounded planner — not this one, not a better one — can show separation on those 8 cells. That ceiling is a property of the frozen #844 task generator, and it is one **Amendment A1's four gate instruments could not have caught**, because A1 tests non-vacuity, axis cardinality, content-derived identity and null headroom, and all four PASSED before any fitting.

Recording this as PROMOTE would have required either weakening δ_min or reporting the isolating splits — where fitting and held-out share topology cells, violating #844 §2.2 — as if they were the gate. Neither was done.

### Verification

- `cargo fmt --check` — pass
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings` — exit 0
- `cargo test --workspace --no-fail-fast --offline` — exit 0, **184 suites, 0 failures**
- `cargo check -p uor-r4-graph-format --no-default-features` and `--features alloc` — pass
- `scripts/check_claim_wording.py` — pass, no unlinked machine-verified/exact-equivalence wording
- `xtask check-model` — `CONFORMANCE.md` equals the model, 33 ids (CM-01)
- `xtask audit-limits` — no shipped crate returns an unsanctioned error (R5); **run last**, per the AUD-VER-001 staleness rule this issue rediscovered on #924

Three-way differential over **480 fixtures** (5 families × 4 horizons × 24 held-out seeds): reference, offline-packed and deployed planners agree on plan existence, emitted plan and plan length, and all three fail closed. Control non-degeneracy: shuffled-state null measured **128/128 real versus 4/128 shuffled**.

### Run contract — honoured

Posted before the run; every branch pre-committed to a different next action, so the measurement had decision value. Actual cost: the full 300-cell grid ran in **44.7 s** wall-clock, release, one core, teacher-free and fixture-free. No long-run publication was required. The corpus-observation arm **did not launch** — 0 of 10 typed object kinds in the v4 record, coverage 0.000 — and is recorded as **UNAVAILABLE, never PASS**.

### Artifacts

- `docs/bounded_semantic_transitions_spec_843.md` — frozen design contract
- `docs/bounded_semantic_transitions_843.md` — measurement record and verdict (append-only)
- `docs/compositional_planning_spec_844.md` §11 — Amendment A1, appended not rewritten
- `docs/r4_intelligence_completion_plan.md` — readable mirror, appended

### Merge trail

#920 → #921 → #922 → #923 → #924 → #925. Merged through the merge queue on the normal path; no administrative bypass, no weakened gate, no relaxed claim.



---
_Comment/review ids in this file: 5380692583, 5380954996, 5381555795, 5381573793, 5381722881_

