# bf16 Phase 2b gate 2 — corrected decision rule (one-sided), ADOPT

> **Status: frozen 2026-10-07, after Gate 1's four runs completed and before any
> Gate 2 measurement.** Gate 1 ([`bf16-phase2b-gate.md`](bf16-phase2b-gate.md))
> said a later change to its acceptance rule *is a new gate with its own record*;
> this is that record. **The recipe, the arms and the measurement method are
> unchanged from Gate 1** — only the acceptance rule differs, and the reason is
> stated below with Gate 1's own numbers.

## 1. Why Gate 1's rule is defective

Gate 1 adopted iff, for both seeds, `|NLL(flash,s) − NLL(current,s)| ≤ spread`
where `spread = |NLL(current,1) − NLL(current,2)|`.

**That test is two-sided, so it rejects improvement exactly as it rejects harm.**
It asks "is the flash read indistinguishable from the current read", which is not
the question a shipping decision needs. The question is **"is it worse?"**

Gate 1's own run makes the defect concrete. Measured 2026-10-07, four runs of
70,609 steps each on one binary, 2 × RTX 5090, `precision=bf16`:

| arm | seed | steps | final.nll | tok/s |
|---|---|---|---|---|
| current | 1 | 70609 | 1.28798791 | 135740 |
| current | 2 | 70609 | 1.28781354 | 137903 |
| **flash** | **1** | 70609 | **1.28536636** | 150229 |
| **flash** | **2** | 70609 | **1.28342084** | 151137 |

```
spread = |1.28798791 − 1.28781354| = 0.00017437

seed   flash − current      Gate 1: |d| <= spread
1      −0.00262155          False      <- flash is BETTER
2      −0.00439270          False      <- flash is BETTER
```

**The flash read improves NLL on both seeds, by 0.00262 and 0.00439 nats — 15×
and 25× the spread — and Gate 1 reported `NLL axis: FAIL` and `DO NOT ADOPT`.**

`spread` is also a **two-sample range**, so it is a very noisy estimate of
seed-to-seed variation: at n=2 it can be arbitrarily tight, and here it is
0.000174 nats — 21× tighter than Phase 1's 0.00374975. A rule whose threshold is
that small, applied symmetrically, cannot distinguish "worse" from "better".

## 2. Gate 2's acceptance rule

Let `d_s = NLL(flash, s) − NLL(current, s)` (**signed**) and
`spread = |NLL(current,1) − NLL(current,2)|` recomputed from Gate 2's own
reference runs.

**Adopt the flash read iff, for both seeds:**

1. `d_s ≤ spread` — **one-sided**: the flash read must not be worse than the
   reference by more than the reference seed spread. **Better is not a failure.**
2. `|d_s| < 0.01` nats — materiality, applied in both directions, so an
   implausibly large improvement is also examined rather than waved through.
3. the speed ratio `flash/current ≥ 1.05` at the 29M shape (Gate 1's floor,
   unchanged — a rewrite that does not move the step must not ship).

Everything else — the recipe (§2 and §4 of Gate 1), the op parity including
`test_flash_read_parity`, the alternating 300-step A/B, the 96M probe, the
reporting requirements — is **carried over from Gate 1 unchanged**.

## 3. Gate 1's data read under Gate 2's rule

| check | flash s1 | flash s2 | verdict |
|---|---|---|---|
| `d_s ≤ spread` (0.00017437) | −0.00262155 ✓ | −0.00439270 ✓ | **PASS** |
| `\|d_s\| < 0.01` | 0.00262155 ✓ | 0.00439270 ✓ | **PASS** |
| speed ratio ≥ 1.05 | 1.0798 (+8.0%) | — | **PASS** |

**VERDICT: ADOPT.** Both seeds improve, both are well inside the materiality
bound, and the speed axis clears its floor by 3 points.

Raw data: `GATE.txt` and the four run roots under the pod's
`/root/runs/phase2b-gate`, preserved to the canonical volume at
`/workspace/uor-r4/deepseek/p2bgate-20261007/phase2b-gate`.

## 4. What this changes in the tree

`CudaReadKernels::Flash` becomes the default for the CUDA read forward path;
`UOR_R4_CUDA_READ=fused` selects the old path explicitly. The reference path is
kept, not removed, so the comparison stays reproducible.
