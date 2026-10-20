# #1375 — Deliver compiled relative-query transport: folded Q8 action + inverse-frame O(L+N) selection (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-24T03:28:29Z
- **Updated:** 2026-10-09T13:17:09Z
- **Closed:** 2026-09-24T03:29:32Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1375
- **Merged:** 2026-09-24T03:29:32Z
- **Merge commit:** 48c2908f1c189f88a4be83f27b6952822e2db4f2
- **Base / Head:** main / UOR-Foundation:codex/compiled-relative-query-delivery-20260924
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this is

Delivers an **orphaned, pushed-but-unmerged** workstream (`origin/codex/compiled-relative-query-20260924`, authored
by the prior agent). It was CI-verified only because the desktop link was unavailable, and **no PR had ever
merged**. This PR cherry-picks its ten commits onto `d71a3143`, verifies it locally, applies the three
review-recommended corrections, and records a scoped result. The prior branch is preserved. No artifact or serving
default is changed; no model source semantics change.

## What it does

Reduce a learned Q8 relation path to a single signed permutation plus an exact prefix-overflow mask, move the query
into the inverse frame, and scan unchanged candidate keys:

- For a signed permutation `R`, `‖q − R·k‖₁ = ‖R⁻¹q − k‖₁` exactly (signed permutations are the linear L1
  isometries; the identity **fails** for a general orthogonal `R`).
- Selection becomes **O(L + N)** instead of **O(N·L)**, with tie-breaking, error precedence, payload identities and
  Q8L1 model bytes unchanged.
- `reject_min_mask` is exactly the sequential checked path's rejection set, including cancelling negations.

## Verification

- **Independent mathematics review:** identity and overflow-mask exactness confirmed; `i64` query widening
  necessary and sufficient; `u64` sum bounded. Verdict: exact.
- **Independent CS review:** `O(L+N)` confirmed; **no correctness fix required**; success path allocation-free;
  rebase onto `d71a3143` clean; test cargo-runnable. Recommended corrections applied.
- **Local executed:** `cargo fmt --check` clean; **18/18** standalone tests pass on pinned `rustc 1.97.1`;
  `EXHAUSTIVE_PATHS=4681` (all paths length 0–4 against the **independent** sequential oracle);
  `SUCCESS_PATH_ALLOCATIONS=0`; `COMPILED_PATH_BYTES=5`; model bytes unchanged.
- **Measured CPU (debug microbench, ns/query):** path 32 / 256 candidates **37,389 → 1,451** (prepared 988);
  32/16 2,328 → 460; 32/1 **148 → 404** (slower for one candidate — reported honestly).

Corrections applied: a new length 5–6 late-negation test; `cancelled_signs` now uses the independent `sequential`
oracle (the delegating `m.select` had become circular); the plan's "compile once" wording corrected to per-call
compile.

## Scope

Bounded typed relative selection over supplied query/key vectors with exact ownership — **not** learned
natural-language attention, general language, reasoning, geometric superiority or energy. The Q8 learner's
established tie with an ordinary signed-permutation control is unchanged. No promotion.

Resources: local build + one 18-test run; no model execution or training; charge **+600,000 ms**; cumulative
**450,039,320 / 455,500,000 ms**.

References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

