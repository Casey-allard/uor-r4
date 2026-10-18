# Row-parallel exact counterfactual — GPU parity and timing, 18 October 2026

**Status:** finite GPU parity **PASS**, negative gate **FAIL as required**, about **56–57× faster** proposals at the Dose 1 shape with identical losses. The path is opt-in; the default `propose` is unchanged. Making it the default and wiring it into the training driver are separate PRs. No training ran and no hold changed.

## Why

`discrete_counterfactual` launches one block of 16 threads, and Gate proposals use only 3 of them. Each thread walks every row serially and recomputes the baseline. That kernel is 99.99% of `Device::propose`, and so of every learning dose's wall time: Dose 1 projected about 37–67 h.

The owner chose "Build now, Dose 1 runs on". The faster kernel had to give exactly the same losses, and the running Dose 1 stays untouched.

## What

The new code is a separate generated CUDA module, `crates/uor-r4-discrete/src/rows_source.rs`. It contains the ordinary helpers and writer byte for byte, plus three integer entries:
1. **Baseline:** once per row.
2. **Virtual move:** once per (row, lane).
3. **Reducer:** per lane, replaying the ordinary loop over those records in row order — refusals, eligibility, break, overflow and status — so it publishes exactly what the ordinary lane would.

It is reached through an explicit opt-in handle, `Device::rows_counterfactual` plus `propose_rows`, and the D29 checker admits exactly the three entries. The suite is `rows_qualification.rs`, run by the `rows-qualify` binary. The suite is a separate binary, so the training driver's source identity is unchanged.

`device.rs` `source_sha256` changes, as with the earlier reuse and decomposition modules. That is declared in the sheet. Dose 1 runs its own pinned binary and is unaffected.

## Run

- **Sheet:** [m1-counterfactual-rows-parity-2026-10-18](../../harness/prep/m1-counterfactual-rows-parity-2026-10-18.md), TEST FITNESS FIT from a non-author reviewer subagent (owner-authorized peer review).
- **Pod:** one RTX 5090, `vast-56495093`, driver 580.95.05.
- **Build:** release, CUDA 12.8, `compute_120`, cloned at the exact branch SHA, with source file hashes checked before building.

| Item | Attempt 1 (`697d9d96c`) | Attempt 2 (`9291976115`) |
|---|---|---|
| Release build / D29 self-test / D29 repo | 0 / 0 / 0 | 0 / 0 / 0 |
| Route parity (72 cases) | PASS | PASS |
| Flock Gate parity (108) | PASS | PASS |
| Relation Gate parity (288) | PASS | PASS |
| Refusal parity | **FAIL (instrument)** | PASS (3/3) |
| Forced terminals: status 8 + lane break | PASS | PASS |
| Negative gate: reducer `tr` clause deleted | FAIL as required | FAIL as required |
| Restored source hashes | clean | clean |

- **Attempt 1's refusal FAIL** was in the suite, not the kernel. Gate-on-Route is refused on the host with `UnimplementedParameter`, which fell into a catch-all arm. Attempt 2 compares host refusals of any kind by their text. Both attempts are kept.
- **Negative gate:** with the virtual-`tr` break clause deleted, the forced fixture diverges, ordinary `[1344, MAX, MAX]` against rows `[1344, 0, 0]`. The suite therefore detects a reducer that does not break.
- **Earlier operational blocks:** two launch attempts stopped before building, one on a source-path check and one on a module-path compile error. Neither ran any model work. The compile error is why the CUDA-feature Rust is now type-checked locally with a stub `nvcc` before any pod run.

## Timing (attempt 2; non-gating, not a qualified rate)

The shape is V4096/B4/C192, `kept_cells` 8, 64 full rows. Each sample is one synchronized `propose`; one warmup is excluded, and the median of 5 is kept.

| Read / parameter | Ordinary | Row-parallel | Ratio | Losses |
|---|---|---|---|---|
| Route / VectorBlock(0) | 3.843 s | 0.0691 s | 55.6× | equal |
| Relation{kept 8, sweeps 1} / Gate(0) | 4.144 s | 0.0728 s | 56.9× | equal |

**Derived (not measured):** Dose 1's 16,404 batches per arm come to about 0.073 s × 16,404 ≈ 20 min per arm on the row path, versus about 19 h at the ordinary 4.1 s.

## Evidence and cost

- **In the repo:** `evidence/attempt{1,2}/`, holding the logs, JSON reports, exit codes, source and binary hashes, GPU identity and negative diff. The release `rows-qualify` SHA-256 is `809d7912…a9010`.
- **Full pod archive:** `icloud:UOR-R4/results/claude/vast-56495093.tar` (10,882,048 bytes, MD5 `d7d35d2da896647a81f7ef03b2d3063b`).
- **Cost:** one pod at about $0.91/h for about 20 min, roughly **$0.30**. The pod was released and deleted after archiving.

## Correction after Codex review (18 October, after merge)

Codex/cf1 posted two source findings before #2630 merged (#2029 6245676413 and its follow-up). Claude merged without reading them, and that is recorded here as a process slip.

1. **Coverage was overclaimed.** Attempts 1–2 swept Route and VectorBlock indices `0..blocks` only: 2 of 32 VectorBlock indices at V16/B2, and 4 of 64 at V16/B4. The route inventory is the route-table length, not `blocks`. **Attempts 1–2 therefore measured a subset of the Route-mode inventory.** The Gate inventories (`0..gate_count`) were complete. The subset evidence stays at its scope.
2. **The retry oracle was weak.** The racy-refusal retries discarded CUDA faults, unexpected successes and other error kinds, and ran after the case's unchanged-state check. In both measured attempts `racy_refusals = 0`, so that code never ran and no measured result depends on it.

**Repair** (branch `claude/rows-qual-fixes-20261018`):
- every retry outcome is classified (a CUDA fault propagates; an unexpected success or a changed refusal kind is a FAIL);
- every call of a case, retries included, runs inside one unchanged-state check;
- Route-mode sweeps cover the complete admitted inventories, `0..routes.len()` and `0..vocabulary×blocks`.

Attempt 3 reruns the suite at the repaired head. The kernel and the timing are unchanged.

## Attempt 3 at the repaired head `70606e8b8` — PASS with full inventories

The run used a fresh RTX 5090, `vast-56499123`, cloned at the exact SHA with the source hashes checked.

| Item | Result |
|---|---|
| Release build / D29 self-test / D29 repo | 0 / 0 / 0 |
| Route parity, **complete** Route and VectorBlock inventories | **684/684 PASS** |
| Flock Gate parity | 108/108 PASS |
| Relation Gate parity | 288/288 PASS |
| Refusal parity (host refusals by text) | 3/3 PASS |
| Forced terminals: status 8 and lane break | PASS |
| Racy refusals | 0 |
| Negative gate: reducer `tr` clause deleted | FAIL as required |
| Restored source hashes | clean |

The timing reproduces:

| Read / parameter | Ordinary | Row-parallel | Ratio |
|---|---|---|---|
| Route VectorBlock(0) | 3.847 s | 0.0693 s | 55.5× |
| Relation Gate(0) | 4.149 s | 0.0733 s | 56.6× |

Losses are equal in both cases.

- **Evidence:** `evidence/attempt3/`. The release `rows-qualify` SHA-256 is `2a77161a…`.
- **Archive:** `icloud:UOR-R4/results/claude/vast-56499123.tar` (5,452,288 bytes, MD5 `369c45c4e1150b2fa68d7397f4b54dbc`).
- **Cost:** about $0.25. The pod was released and deleted.
- **What this means:** attempt 3 is the qualifying parity evidence for the fast-path Dose 1 re-issue.

## Attempt 4 at `907a9926b` — rank-hinge objective at K16 PASS; K16 A/R timing

This run is the prerequisite for the owner's fast-path admission redirect (K16-A vs K16-R, #2029 6250410397). It used a fresh RTX 5090, `vast-56557973`, cloned at the exact SHA with the source hashes checked.

**Parity:**
- **New control `rows_rank_objective_parity`:** Route at `kept_cells = 16`, complete inventories, under both `OccupiedCellRankHingeV1` (the rank hinge, writer `s[11] == 1`) and `NearestAddressMarginV1`. Result: **1,368/1,368 PASS**.
- **Every earlier control PASSes again:** Route 684, Flock 108, Relation 288, refusals 3, forced terminals.
- **Other checks:** D29 checks 0/0/0; the negative gate FAILs as required; restored source hashes are clean.

**Timing** (64 rows, median of 5; losses equal in every case):

| Shape / objective | Ordinary | Row-parallel | Ratio |
|---|---|---|---|
| K8, Route VectorBlock(0), A | 3.882 s | 0.0700 s | 55.5× |
| K8, Relation Gate(0), A | 4.200 s | 0.0742 s | 56.6× |
| **K16, Route VectorBlock(0), A (K16-A)** | 7.400 s | 0.1297 s | **57.0×** |
| **K16, Route VectorBlock(0), R (K16-R)** | 7.402 s | 0.1292 s | **57.3×** |

**Derived redirect budget:** 16,404 batches × about 0.13 s comes to about 36 min per arm-seed, so the four registered runs take about 1.2 h on a 2×5090 pod, about $2.3. The ordinary kernel would need about 34 h per arm-seed.

- **Evidence:** `evidence/attempt4/`. The release `rows-qualify` SHA-256 starts `75682c52…`.
- **Archive:** `icloud:UOR-R4/results/claude/vast-56557973.tar` (5,453,824 bytes, MD5 `2991ca26475dd9b8f04def32564c73ed`).
- **Cost:** about $0.25.

**Correction (PM audit #2029 6250526855, item 2).** Attempt 4's rank-objective parity was **vacuous for the R term**. At vocabulary 16 a model has at most 16 occupied cells, so `rank ≤ kept_cells = 16` and R = 0 on every row. It proves lanes/rows parity under the R code path, not with a live R term, and it stays recorded at that scope.

## Attempt 5 at `3af49e029` — active rank-hinge parity PASS

New control `rows_rank_hinge_active_parity`:
- **Fixture:** a 64-token, two-block table over **64 distinct cells**. Token `t` has block0 root `t mod 16` and block1 root `(t + t/16) mod 16`.
- **Required on the exact batch and state**, so the R term is live: more than 16 occupied cells, at least one evaluated row with `cell_rank_hinge > 0`, lanes-vs-rows parity under R for every Route and VectorBlock index, and objective A on the same state giving different ordinary losses for at least one parameter (the wrong-objective gate).

| Check | Result |
|---|---|
| Occupied cells | **64** (> K16) |
| Rows with `cell_rank_hinge > 0` | **46 / 64** |
| Lanes-vs-rows parity under R (Route + VectorBlock, all indices) | **134 / 134 PASS** |
| Parameters whose ordinary losses differ between R and A | **134 / 134** |
| All earlier controls (Route 684, Flock 108, Relation 288, refusals, forced terminals, vacuous-R 1,368) | PASS |
| D29 self-test / repo, negative gate (reducer `tr` clause deleted), restored hashes | 0 / 0, FAIL as required, clean |

- **Evidence:** `evidence/attempt5/`. The release `rows-qualify` SHA-256 starts `06142611…`.
- **Archive:** `icloud:UOR-R4/results/claude/vast-56559825.tar` (5,456,896 bytes, MD5 `7db5dbfa3ace2202c41cda903448fd93`).
- **Cost:** about $0.20.

## Scope of attempts 4–5 and the registered timing slot (AGY review #2658 6250630831, P3)

| Attempt | Source | Controls run | What it establishes |
|---|---|---|---|
| 4 | `907a9926b` | 6, including `rows_rank_objective_parity` (1,368/1,368) | Lanes/rows parity under the R code path at K16, **with R = 0 on every row** (V = 16, at most 16 occupied cells). Vacuous for the R term. |
| 5 | `3af49e029` | 7, adding `rows_rank_hinge_active_parity` (134/134) | Parity with a **live R term**: V = 64, B = 2, K16, 64 occupied cells, 46 rows with R > 0, objective sensitivity 134/134 |
| Final head (P1/P2 repairs) | this PR's head | the same 7 controls | P1 turns a sensitivity-loop proposal failure into a FAIL instead of a skip. P2 brackets every call in the active control with unchanged-state checks on both devices. Neither changes a measured value. **This head runs on GPU in the single new-head qualification session** (PM audit item 4) and is bound there. |

**Timing.** The 5-sample `timing()` above is `qualified: false`: `VectorBlock(0)` only, checkpoint-0 only. It is a diagnostic speedup check and budget estimate.

**The registered K16 timing slot** is filled in the qualification session by the dedicated `k16-timing` `rows_v1` adapter (#2659, Codex): checkpoint-0 and Q4 × A and R, `Route(0)` plus `VectorBlock(0)`, with 100 timed samples plus 1 excluded warmup. These rows-qualify numbers do not compete with that slot.
