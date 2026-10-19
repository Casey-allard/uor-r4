# BVG-K Stage T: batched verified coordinates as the learning law — DRAFT

- **Status:** DRAFT. **Not registered. TEST FITNESS: PENDING.**
  - No compute, $0. Training stays HELD.
  - Registration fixes the base lineage, the numeric margin and the cost from the evidence listed under "Preconditions"; none of them is fixed here.
- **Line:** M1 W3 BVG-K (#2029; D39 item 4). **Lab:** claude (self-directed, #2029 intent `6257519136`).
- **Parent design:** `m1-batched-verified-coordinates-DRAFT-2026-10-18.md`.
  - §2 is the law: per-family cursors, the S_t / best / joint-verify commit rule, and the accounting.
  - §5 is the bar axes; §8 step 4 is Stage T.
  - This sheet inherits §2 verbatim.

## Why now

The parent held Stage T "until the K16 redirect outcome fixes the configuration of record". The K16 call (#2029 `6257335075`, record `docs/labs/k16-outcome-call-2026-10-19/`) is posted:
- `OCCUPANCY_COLLAPSE_TRIGGERED`, with no KEEP;
- **arm A** (`NearestAddressMarginV1`) selected; R is excluded on G-Occ.

Stage T therefore re-bases on **objective A**, not on the parent's Route K16 A/R pair.

Stage C1 measured the motivation (#2713). At the K16 shape, 16 coordinates score in 123.6 ms, against 125.5 ms for one single proposal: a **16.25× proposal-throughput gain**. Stage T asks whether that becomes better learning per unit of wall clock.

## Preconditions before registration (all must hold)

1. **C2 device-level qualification PASS.** **Met** 2026-10-19: all five controls PASS (#2029 `6257768576`, record `docs/labs/injective-preflight-and-c2-qualify-2026-10-19/`).
2. **A base lineage that reaches its own admission gate.**
   - **Status 2026-10-19:** the injective pre-flight **failed B3_init**: 127 of 3,847 P_succ rows admitted at checkpoint 0, below 500, so that run is not dispatched. The fallback, the parent `rows_v1` configuration (raw `52b32dc8…`), itself stopped at Q1 on admission collapse (`docs/labs/attention-rows-q1-diagnosis-2026-10-18/`).
   - **So no current base lineage reaches admission.** Stage T stays DRAFT until a successor lineage passes its pre-flight and Q1 gates. The candidates are larger K, an address-structured injective cell assignment, or partial injectivity (successor options in the pre-flight record).
   - Whatever base is chosen, exactly one variable changes from it: `coordinate_block`.
3. **The numeric non-inferiority margin**, a registration item.
   - It is fixed from a **measured two-seed dispersion** of the chosen base at matched wall clock, on DEV `P_succ` conditional top-1 and full DEV loss.
   - If the base has only one seed, as the injective runbook does, registration must first run a second-seed T1, at its own cost line. Alternatively, it may adopt a named measured two-seed dispersion from the parent lineage, with the reason stated.
   - It is never set from a single seed, and never after any Stage T result is visible.

## Arms (one variable: `coordinate_block`)

| Arm | `coordinate_block` | Path |
|---|---|---|
| T1 | 1 | the untouched `run_transitions` / `transition` (the current law) |
| T16 | 16 | `run_block_transitions` / `transition_block` |
| T4 (optional, decided at registration) | 4 | as T16 |

Seeds 1 and 2. Each arm uses its own GPU on identical pods.

## Bar (axes inherited from parent §5; numbers fixed at registration)

- **Primary axis: equal training wall clock, `W`, fixed at registration.**
  - **What the clock covers.** Each arm's clock starts at its first training batch, after admission and the checkpoint-0 evaluation, and stops at elapsed `W`. Build, table load and pre-flight are excluded. DEV evaluations and checkpoint I/O are **included** in every arm's own clock, so T16 and T4 pay for their own evaluations.
  - **Where an arm is read.** Each arm is read at its last completed checkpoint at or before `W`.
  - **Q1.** The base lineage's Q1 stop maps to `W/4` on the same axis.
- **Lead metric:** target-tokens/s.
- **Secondary axes, reported:** equal coordinate evaluations and equal target rows.
- **Quality metrics.** **Primary:** DEV `P_succ` conditional top-1. **Co-primary:** full DEV loss. **Reported, not gating:** top-8 and B3.
- **KEEP for T16** requires, on both seeds, primary top-1 better than T1 beyond the margin **and** full DEV loss non-inferior. Fresh transfer must be reported (D22).
- **Non-inferiority:** neither primary metric may trail T1 by more than the margin at `W`.
- **T4**, if registered, is judged the same way and carries its own cost and power line.
- **Diagnostics only, never a stop:**
  - joint-commit rate, mean |S_t| and no-op batches;
  - DEV loss per pass (in-sample over-fit exposure);
  - Q1 placed on the wall-clock axis.

## Pre-registered first-batch controls (moved here from C2's uncovered list)

These need an owner-released `CorpusUpdateAdmission`, so they run inside the released Stage T job. **Each must PASS before any Stage T result is read.** A FAIL stops the line for diagnosis (D31) and is never a negative.

- **V1d, K = 1 identity.** For the first 24 batches, `transition_block` at K = 1 on a shadow device equals the T1 arm's `transition`, byte for byte, in state hash, epoch, accepted updates and cursor.
- **KId, invariants.** On every T16 batch:
  - `model.epoch() == accepted_updates`;
  - `committed_moves == Σ moves`;
  - `|commit_ordinals| == moves`;
  - the receipt's `committed_loss` equals a fresh device score;
  - `committed_loss < l_before` on every commit.
- **V8d, FrozenZero** (if a FrozenZero arm is registered). Relation trits stay 0, and no relation ordinal is ever committed.
- **V9d, resume.** The first checkpoint, restored and continued for 64 batches, equals an uninterrupted run bitwise.
- **AKd, apply-kernel parity.** For the first 32 T16 batches whose commit is a single move, a shadow device restored from the pre-state commits the same coordinate through the ordinary K = 1 `transition` under the same admission. Its state must equal the T16 state byte for byte.
  - This closes C2's "BVG-K against ordinary apply kernel" gap on the real path.
  - Whether one admission may drive the shadow device is decided at registration review, with the owner.

## Cost (projection, refined at registration)

- **What is measured.** The only measured per-call numbers are at the **K16 Route shape**:
  - C1: 125.5 ms for one proposal; 123.6 ms for 16 coordinates, scoring only.
  - C2 TP: 129.7 ms at K = 1 and 259 ms at K = 16 including the joint verify, about 8× per coordinate.
- **What is assumed.** The base lineage's per-batch time is **not measured**. The 1.31 s/batch figure is the injective sheet's planning assumption, not a measurement. It is measured at registration, together with K = 16 at the base's own shape.
- **Wave plan within the caps** (≤ 4 pods, ≤ $8/h). Using 2 pods × 2 GPUs (4 slots, about $3.8/h), at equal wall clock `W`:
  - **Wave 1:** T1 and T16 on seeds 1 and 2, four arm-seeds at once.
  - **Wave 2 (only if T4 is registered):** T4 on seeds 1 and 2, on 1 pod.
  - **Cost at `W` ≈ 6 h:** about **$23** for wave 1 (24 GPU-hours) and about **$11** for wave 2 (12 GPU-hours). That is about 12 h elapsed and about $34 with T4, or about 6 h and about $23 without.
  - A Q1-equivalent stop costs about a quarter of that.
- **Not included above:** the margin's second-seed T1, if it is needed (precondition 3), with its own line.
- **Hard stops:** the 30 GiB disk floor and the owner caps.

## Base artifact

No checkpoint is reused. Every arm starts at checkpoint 0 of the base lineage fixed by precondition 2: the injective treatment configuration, or else the parent `rows_v1` configuration. Each arm's config is that base config with only `coordinate_block` changed, and each is bound by its raw and canonical config SHA and lineage in the release packet.

## Data split

- **Training and evaluation:** the Dose 1 pins inherited from the base lineage. TRAIN tokens are `3dee858f…`. DEV is the frozen 14,287-row `ft/dev` panel, tokens `0919e90f…`, mask `a838bb1a…`, and it is the open development panel.
- **Fresh transfer:** a sealed slice, opened once and only after a sealed paired DEV decision. If the injective test consumes its fresh V2 slice first, a new fresh slice is pinned at registration and never reused.

## Steps

1. The preconditions hold: C2 PASS, the injective outcome is known, and the margin is fixed.
2. This sheet is revised to its registered form, with numbers and base lineage fixed. TEST FITNESS comes from a fresh-context adversarial reviewer subagent (owner direction 6255848646), with every finding posted on #2029.
3. PM dispatch, then the owner's release per arm. Casey types the phrases (D40), and the executor stages the commands.
4. The run. The first-batch controls (V1d, KId, V8d, V9d, AKd) must PASS before any result is read. Then T1, T16 and optionally T4 run at equal wall clock on both seeds.
5. Sealed DEV comparison, then fresh transfer under typed PM authorization. Post the receipts, archive, and delete the pod.

## Stop rule

- Any first-batch control FAIL stops the line for diagnosis (D31). It is an instrument blocker, never a negative.
- The base lineage's registered Q1 stop applies to each arm on the wall-clock axis.
- The owner caps (≤ 4 pods, ≤ $8/h) and the 30 GiB disk floor are hard stops.
- A healthy run is extended, not cancelled.

## Minimum test power

- Two seeds per arm. The non-inferiority margin is the base lineage's measured two-seed dispersion and is never narrower than the panel's 2-SE row floors (D22 §3): 20 rows for top-1, 63 for top-8 and 87 for B3.
- A KEEP needs the same direction on both seeds, beyond the margin.

## Open items for registration review

- The base-lineage choice (precondition 2).
- Whether T4 is included.
- Whether AKd's shadow device may use the run's admission.
- Seed-2 timing: in parallel or after seed 1.

TEST FITNESS: PENDING
