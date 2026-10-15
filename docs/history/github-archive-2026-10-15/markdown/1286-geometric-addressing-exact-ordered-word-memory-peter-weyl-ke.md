# #1286 — Geometric addressing: exact ordered-word memory, Peter-Weyl kernel, route abstention

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-19T20:09:35Z
- **Updated:** 2026-09-19T20:42:52Z
- **Closed:** 2026-09-19T20:10:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1286
- **Merged:** 2026-09-19T20:10:39Z
- **Merge commit:** 6a43099fb2b0a6bdd5c8fd4c94e87869af60585d
- **Base / Head:** main / UOR-Foundation:codex/geometric-addressing-v2
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this delivers

Rounds 2-7 of the geometric-core investigation, **rebased cleanly onto current `main`**. PR #1284 was squash-merged, so commits pushed to its head branch afterwards never reached main; this branch replaces the conflicting #1285 and carries only the new work.

**14 files, +5,480/-17.** Content check against `origin/main` before this branch: `lowbit_attention.rs`, `geometric_attention.rs` and the receipts were absent; `lowbit_core.rs` (with `LowBitCoreTrainer`) and `chat.rs` were present. Every deletion in the diff is one of my own edits; the three shared docs files are pure appends, so nothing from main is lost.

## Measured, held out, deterministic seeds

**Works**
- **Ordered-word addressing**: duplicated-key recovery **0.14 → 0.98**; context copy **1.00** at lengths 4/8/16 vs linear attention 0.05/0.03/0.00 and order-1 0.86/0.64/0.47. Ordered words of 2I elements (order 2 = 120² = 14,400 addresses, order-preserving by construction), positional radix, no multiplier.
- **Peter–Weyl grounding**, computed from the verified table rather than cited: 2I has **9 conjugacy classes** `[1,1,12,12,12,12,20,20,30]`, conjugation-invariance checked over all 14,400 pairs. A class-function kernel is therefore **9 ternary weights** — the maximally compact rotation-invariant kernel the group admits — and it recovers a corrupted query address **0.13 → 0.23**.
- **Composition beats lookup on unseen relational facts: 0.42 vs 0.00**, with no generalisation gap (0.42 unseen vs 0.39 seen), because the query is computed rather than retrieved. First result in this line where the group structure is load-bearing rather than decorative.
- **Open/closed route abstention**: unseen queries become **0 ok / 64 abstain / 0 wrong** with seen accuracy unchanged — hallucination becomes honest abstention, and eviction is proved exactly (never written).

**Falsified (recorded, not hidden)**
- The dense low-bit recurrence cannot hold memory: `i32` saturation in **16–32 steps**; burial at delay ≥ 8; a recurrent right-shift decay is falsified (1.00 → 0.16 → 0.00).
- A `relu` after an exact-address read destroys the retrieved value's sign half (0.31 → 0.64 when removed).
- Learning the harmonic filter from clean data **loses** to a fixed exact filter (0.27 vs 0.42) — robustness is not in the objective.
- **Five readout levers flat** on the relational ceiling: weight precision, training budget, read width, filter capacity, decode geometry.

## Reviews added

- **SpiralCore v68** mathematics extracted and indexed (33 sections, SHA-256 verified against the preserved copy; 34 knowledge items / 33 edges).
- **Navier–Stokes / Euler formalisation** (OpenAI, Lean 4) reviewed: no mechanism transfers; two methodological transfers — machine-checked certificates as the standard for exact claims, and **sup-norm growth as a standard gate** for every new mechanism.
- **Hyperbolic geometry review**: H4 is a **spherical** Coxeter group, so "Poincaré ball on H4" is a category error; the compute win is **logarithmic hierarchical access**, not hyperbolic arithmetic (fanout 8 / depth 4 = 4096 leaves in 32 comparisons, integer-only); the hyperbolic work is **downgraded from rejected to offline decision instrument** under the owner's allowance.

## Honest limits

No chat capability. One layer, synthetic tasks. Addresses are fixed, not learned. `serving_multiplier_check.py` has not been run on the new modules, so **no served-binary multiplier claim** is made for them (the construction is multiplier-free and the integer path is verified equal to an `f64` reference). Energy/token **UNAVAILABLE** on this machine. Six defects were caught by tests during this work; four initially looked like mechanism results.

## Ledger

Cumulative **145,888,565 / 147,200,000 ms** (~21.9 min remaining). No extension used. No destructive deletion; the 128 MiB storage stop margin is untouched.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

