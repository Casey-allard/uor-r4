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

1. **C2 device-level qualification PASS** (`m1-bvgk-stagec2-block-qualification-2026-10-18.md`, FIT; owner-approved fixture-only run, awaiting PM dispatch).
2. **The base lineage is fixed by the injective test** (`docs/labs/injective-attention-runbook-2026-10-19/`).
   - If its pre-flight battery and Q1 gate pass, the base is the **injective treatment configuration**: PrimeZeta table file `83c1380a…`, `Relation{8,1}`, objective A, K8, `rows_v1`, Dose 1 pins.
   - Otherwise the base is the parent `rows_v1` configuration (raw `52b32dc8…`), with the injective result recorded as context.
   - Either way, exactly one variable changes from the base: `coordinate_block`.
3. **The numeric non-inferiority margin** is fixed from the base lineage's measured two-seed dispersion on DEV `P_succ` conditional top-1 and full DEV loss, at matched wall clock. It is set at registration and never after a Stage T result is visible.

## Arms (one variable: `coordinate_block`)

| Arm | `coordinate_block` | Path |
|---|---|---|
| T1 | 1 | the untouched `run_transitions` / `transition` (the current law) |
| T16 | 16 | `run_block_transitions` / `transition_block` |
| T4 (optional, decided at registration) | 4 | as T16 |

Seeds 1 and 2. Each arm uses its own GPU on identical pods.

## Bar (axes inherited from parent §5; numbers fixed at registration)

- **Primary axis:** equal training wall clock.
- **Lead metric:** target-tokens/s.
- **Secondary axes, reported:** equal coordinate evaluations and equal target rows.
- **KEEP for T16:** better DEV quality at equal wall clock on both seeds, beyond the registered margin, with fresh transfer reported (D22).
- **Non-inferiority:** T16 must not trail T1 by more than the margin on either quality metric at equal wall clock.
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

T1 costs the base lineage's measured per-batch time, about 1.31 s per batch on the K8 Relation shape (injective sheet §Cost). For T16, C1 shows the proposal stage is flat in K at the K16 Route shape; the K8 Relation shape is measured by the C2 TP diagnostic before registration.

At equal wall clock all arms cost the same. With 3 arms × 2 seeds at about 6 h each, that is about $34 on 2 × 5090 pods: 3 pod-pairs at about $1.89/h for about 6 h. A Q1-equivalent stop costs about a quarter of that. The owner caps (≤ 4 pods, ≤ $8/h) and the 30 GiB disk floor are hard stops.

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
