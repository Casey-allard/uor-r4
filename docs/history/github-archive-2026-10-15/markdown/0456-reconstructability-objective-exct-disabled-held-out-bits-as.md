# #456 — Reconstructability objective: EXCT-disabled held-out bits as a compile-time cover objective

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @auser
- **Created:** 2026-08-06T13:09:56Z
- **Updated:** 2026-08-08T11:25:29Z
- **Closed:** 2026-08-08T11:21:34Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/456
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Motivation

Gate C measures reconstruction error only post-hoc: the pinned score report shows graph-resolved positions at ~1% top-1 / 16.3 bits while ~86% of positions resolve via EXCT exact-context memory (the M.V.G. caveat: "the current judge measures exact-context memory, not graph generalization"). Cover induction (`induction.rs`) optimizes split entropy-gain floors and marginal agreement-per-byte (`cover_sweep.rs`, #435) — never the quantity that matters: **can the joint be reconstructed from the chosen blocks with exact lookup disabled?**

In Reconstructability Analysis terms: a cover is a block model (regions = blocks/cliques with local joint distributions), and its quality is reconstructability of the empirical joint from the block marginals. TF1 D-4 already names the quantity: `ρ̂_d`, the held-out residual of the backoff predictor **with exact lookup disabled**.

## Proposal

Promote reconstruction error from a Gate C observation into a first-class compile-time metric:

1. Certifier-side: a reconstruction-bits certificate — held-out log-loss (bits/token) and top-1 of a compiled cover scored with EXCT fully disabled, reported per configuration.
2. `cover_sweep`: report reconstruction bits alongside agreement on every frontier point, so the knee rule can see what each byte buys in reconstructability.
3. Compiler-side (follow-up, gated on 1–2): accept reconstruction-bits improvement as a split/stop criterion alternative to raw entropy gain.

## Pre-registered exit criteria

- The certificate reproduces the pinned 47-node cover's graph-only numbers (κ-pinned fixtures).
- Null arm: shuffled emission tables must score at the unigram floor — a certificate that passes on corrupted structure is a harness failure (K-2 mutation discipline).
- Success = at least one swept configuration whose reconstruction bits beat the pinned cover's graph-only bits by a pre-declared margin, or a recorded NEGATIVE with the frontier data that motivated the next arm.

Refs: #433 (evidence ladder), #435 (split criteria), #446 (two-sided marginalization lesson), docs/r4_graph_compiler_implementation_plan.md M.V.G. checkpoint.


## Comments (2)

### comment 5205888925 — 2026-08-06T14:17:10Z — auser

## Harvest: first schema-3 sweep (PR #462 binary, pinned Gate C fixtures)

9-point grid, 399,694 train / 100,306 held-out observations, fixed #64 scorer. With-EXCT agreement (R1+2 top-1) is **flat** across the grid (36.3–36.6% — the knee rule correctly concludes the knob is inert for agreement). The new EXCT-disabled reconstruction metric **discriminates**:

| point | regions | R1+2 top-1 | recon top-1 | recon bits |
|---|---|---|---|---|
| k0=8/gain=0.25 (default) | 42 | 0.3655 | 0.0150 | 16.2427 |
| k0=8/gain=0.1 | 54 | 0.3631 | 0.0059 | 14.4601 |
| k0=16/gain=0.25 | 70 | 0.3641 | 0.0132 | 16.3066 |
| k0=16/gain=0.1 | 100 | 0.3637 | **0.0171** | **13.8848** |

(TLA3 store baseline: 39.2% top-1, 8.4985 bits; artifact bytes within +0.4% across the grid; budget=512 rows identical to budget=128 — budgets never bind.)

### Reading

- **The premise of the issue is confirmed**: configurations the agreement metric calls identical differ by 2.4 bits/token in reconstructability. k0=16/gain=0.1 (100 regions) beats the 42-region default by 2.36 bits/token EXCT-disabled at essentially equal bytes and −0.2pp with-EXCT agreement.
- Had the knee rule optimized reconstruction bits, it would have selected the 100-region configuration — direct motivation for item 3 (reconstruction bits as a split/stop criterion).
- Recon top-1 remains tiny (0.6–1.7%): finer covers carry more reconstructable *information* (bits) but the graph alone still doesn't *predict* — consistent with the #433 evidence-ladder ceiling and the #459 finding that k≥3 marginals are count-limited at this corpus scale.

Report: `/tmp/cover_sweep_456/cover_sweep.json` (schema 3; recommendation block records `reconstruction_bits_per_token = 16.2427`).

Exit-criteria status: certificate reproduces the pinned graph-only numbers ✓ (rule1_chain per point); shuffled-emission null arm and the pre-declared-margin verdict remain open — both belong with item 3, where a selection criterion would actually consume the metric.


### comment 5225704345 — 2026-08-08T10:28:21Z — Casey-allard

Unassigning @auser and returning this to the open pool (no activity since 2026-08-06; not currently being worked).

**State: partially delivered.** PR #462 (merged) landed the measurement infrastructure — the EXCT-disabled reconstruction-bits certificate and the schema-3 `cover_sweep` reporting recon bits per frontier point. Its harvest confirmed the premise: configurations the with-EXCT agreement metric calls identical (36.3–36.6% top-1) differ by **2.36 bits/token** in reconstructability (k0=16/gain=0.1, 100 regions, beats the 42-region default at equal bytes).

**Remaining (the exit criteria are not yet satisfied):**
1. The **shuffled-emission null arm** (a certificate that passes on corrupted structure is a harness failure) — not yet run.
2. The **pre-declared-margin success/NEGATIVE verdict** — the 2.36-bit gap looks like a pass, but it needs the null arm to validate the instrument and a formal margin declaration.
3. Item 3 (the payoff): **reconstruction bits as a compiler-side split/stop criterion** — gated on 1–2; larger, likely its own issue.

Left open for whoever picks it up; items 1–2 are a bounded measurement, item 3 is the follow-up.



---
_Comment/review ids in this file: 5205888925, 5225704345_

