# D41 Item 3 — Driver Math Optimization Reconciliation & ~1M-Token Donor Pilot Staging (`#2821`)

- **Roadmap item:** [`d41-driver-math-pilot`](../../roadmap/items/d41-driver-math-pilot.md) (`#2821`, M2 prerequisite for [`m2-donor-corpus-generation`](../../roadmap/items/m2-donor-corpus-generation.md) `#2857`)
- **Lab:** `antigravity` (`antigravity-56`, 2026-10-25)
- **Machine-readable summary:** [`driver-math-parity-summary.json`](driver-math-parity-summary.json)
- **Pre-registration sheet (`distill-d1-pilot-1m`):** [`../../harness/prep/d41-driver-math-pilot-2026-10-25.md`](../../harness/prep/d41-driver-math-pilot-2026-10-25.md)

## 1. Purpose and D41 item 3 structure

Per `archive/integration/DECISIONS-2026-10-19.md` (D41 items 1–5, lines 1811–1819), `CHARTER.md` §3.10 and §5.1, and `PLAN.md` lines 75–77, `d41-driver-math-pilot` (`#2821`) consists of two halves:

1. **Driver math optimization with parity controls** (row-parallel counterfactual evaluation, address-structured cell occupancy, BVG-K Stage C1 batched coordinate scoring, and BVG-K Stage C2 batched block apply); and
2. **The ~1M-token pilot on `openai/gpt-oss-120b` (~$2)** (`distill-d1-pilot-1m`), which tests hard-target-only token generation, panel disjointness, round-trip Merkle receipt binding, and throughput/cost sizing ahead of the full `distill-d1` pass (`m2-donor-corpus-generation`, `#2857`).

The audit synthesis at `docs/labs/repo-renormalization-2026-10-19/audit/synthesis.md:113` noted that neither half had been consolidated under `docs/labs/d41-driver-math-pilot/`. This record reconciles and pins the four CUDA qualification receipts that satisfy the first half (driver-math optimization and parity controls) and stages the one-page pre-registration sheet and non-author `TEST FITNESS: FIT` review for the second half (`distill-d1-pilot-1m`), while holding dispatch of the `openai/gpt-oss-120b` donor pilot on M1 Stage T (`m1-bvg-k-stage-t-training`, `#2836`) and the owner's `CHARTER.md` §3.10 / §5.1 dispatch release.

## 2. Reconciled CUDA driver-math optimization and parity controls (PASS)

All four CUDA driver-math optimization and parity-control steps required before M1 Stage T (`#2836`) have landed on `main` and passed GPU qualification on NVIDIA GeForce RTX 5090 (`sm_120`):

| Component | Lab record | Sealed GPU receipt & SHA-256 | Parity & negative controls | Measured CUDA speedup |
|---|---|---|---|---|
| **Row-parallel counterfactual driver (`rows_v1`)** | [`counterfactual-rows-parity-2026-10-18`](../counterfactual-rows-parity-2026-10-18/README.md) | [`attempt3/rows-parity.json`](../counterfactual-rows-parity-2026-10-18/evidence/attempt3/rows-parity.json) (`be44d32ec5009055b5ad5dbc9a6f4de0e172e7b0be0d67ede90f32f08b33d70d`), [`attempt4/rows-parity.json`](../counterfactual-rows-parity-2026-10-18/evidence/attempt4/rows-parity.json) (`fedd5c6fde77326e3152c924e88d71c2d80d911ff55254921eb188a57e3df340`) | `684/684` route parity, `108/108` flock-gate parity, `288/288` relation-gate parity, `1,368/1,368` `k16` rank-hinge parity (`0` mismatches); negative sabotage control `FAIL` as required (`6/6`) | `55.5x` (`k8` route), `56.6x` (`k8` relation), `57.0x` (`k16` Obj-A), `57.3x` (`k16` Obj-R) |
| **Address-structured cell occupancy (`address-cells-v1`)** | [`address-cells-preflight-2026-10-19`](../address-cells-preflight-2026-10-19/README.md) | [`evidence/grading.json`](../address-cells-preflight-2026-10-19/evidence/grading.json) (`90c113adbcc2de7c2321a6c6fca9b89a7e72478f8123ae6e3a67dd6d3ee61bb3`) | `256/256` cells occupied (`16` tokens/cell), `4,096` distinct vectors, `0` ties, `B3_init(P_succ) = 1,351 / 3,847` (bar `>= 1,100`), `B3_init(full) = 4,244 / 14,287` | Exact injective initialization (`0` collisions) |
| **BVG-K Stage C1 batched coordinate scoring (`rowsk_v1`)** | [`bvgk-rowsk-scoring-2026-10-18`](../bvgk-rowsk-scoring-2026-10-18/README.md) | [`evidence/run1/clean/rowsk-qualification.json`](../bvgk-rowsk-scoring-2026-10-18/evidence/run1/clean/rowsk-qualification.json) (`d5ec5348d64e2abd74a334319d4f78e6ed7b174b5a2fc0bb381505dc46e09e56`) | `V2` `5,130/5,130` coordinate scoring parity (`0` mismatches), `V6` `6/6` block refusals, `V2-neg` sabotage `FAIL` as required (`171` mismatches detected) | `16.25x` (`16,249 / 1,000` at `k16`, bar `>= 6.0x`) |
| **BVG-K Stage C2 batched block apply (`bvgk_v1`)** | [`injective-preflight-and-c2-qualify-2026-10-19`](../injective-preflight-and-c2-qualify-2026-10-19/README.md) | [`evidence/bvgk-c2-qualify-20261019/report/bvgk-qualification.json`](../injective-preflight-and-c2-qualify-2026-10-19/evidence/bvgk-c2-qualify-20261019/report/bvgk-qualification.json) (`2fff942fa2b802e180c9bd87336692730d6beaf22f2c0bac8867afb0dea345f3`) | `5/5` controls (`V0` identity, `RC` recarry parity, `UT+JR+DC` block sweep, `RF` refusals, `C1R` C1 parity re-run) `PASS`; `3,419` updates applied across `6` fixture suites | `8.0x` (`8,000 / 1,000` `k16` joint-verify speedup, bar `>= 4.0x`) |

## 3. Staged ~1M-token donor pilot (`distill-d1-pilot-1m`) and dispatch hold

- **Pre-registration sheet:** [`../../harness/prep/d41-driver-math-pilot-2026-10-25.md`](../../harness/prep/d41-driver-math-pilot-2026-10-25.md) defines the two arms (`DeterminismCanary_64` and `Pilot1M_Donor`), four hard gates (`G-D41-1` hard-target-only schema scan, `G-D41-2` evaluation-panel disjointness, `G-D41-3` round-trip Merkle receipt verification, `G-D41-4` `>= 1,000,000` tokens within `<= $2.50`), and three negative instrument controls (`C-Leak`, `C-Overlap`, `C-Tamper`).
- **Why dispatch is held (`status: held`):**
  1. Per `PLAN.md` lines 75–77 and D41 item 3 (`archive/integration/DECISIONS-2026-10-19.md:1817`), the M1 BVG-K Stage T training run (`m1-bvg-k-stage-t-training`, `#2836`) completes on the M1 path before M2 donor generation begins.
  2. Per `CHARTER.md` §3.10 and §5.1, while normal GPU runs proceed on the lab's own D67 pre-run review, D41's `openai/gpt-oss-120b` donor pass is the sole charter exception that requires an explicit owner release at dispatch.
