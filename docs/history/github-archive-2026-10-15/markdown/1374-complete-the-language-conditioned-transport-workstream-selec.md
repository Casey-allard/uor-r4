# #1374 — Complete the language-conditioned transport workstream: selector, compiled execution, span curriculum (References #820, #1373)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-24T03:12:29Z
- **Updated:** 2026-10-09T13:17:37Z
- **Closed:** 2026-09-24T03:13:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1374
- **Merged:** 2026-09-24T03:13:32Z
- **Merge commit:** d71a3143523decd94a5a00d728802ba88262f0b5
- **Base / Head:** main / UOR-Foundation:codex/language-transport-20260924
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What this is

Completes the workstream a prior agent left mid-turn on `codex/language-transport-20260924` (base `48106e7c`,
#1373). The branch was code-complete but its `integrate` mode was unwired and its declared measurements were not
sealed. This PR finishes the wiring, runs and seals the measurements, and records a **scoped, exposed** result.
No model source, artifact default or serving default is changed.

## Added / completed
- `learner/language_transport.rs` — learned integer feature→Q8-action selector; exact ownership (absent owner and
  absent owned relation rejected; no distance-based fallback).
- `learner/tl_execution.rs` — compiled complete native execution (four maps compiled once; precomputed embedding
  bases and per-event offsets).
- `bin/support/language_continuation.rs` — `integrate` mode wired; `span_lengths`/`fresh_span`;
  `UOR_LANGUAGE_MAX_SPAN` **wired into the training draw** (the pending script left it inert).

## Results (sealed; independently audited)
- **Selector exact:** `256/256` fit, `0/256` wrong-instruction, `256/256` absent-owner rejection; `integrate`
  selection `32/32`; missing-relation/absent-owner rejected `32/32`. The `32/32` "explicit signed matrix"
  agreement is an **implementation-equivalence check** (same action, two evaluations), not an independent comparator.
- **Compiled execution faithful and faster:** 156 grounded native/compiled comparisons identical; 64 further
  parity assertions in `integrate` (the 64 changed-payload rollouts are compiled-only, **not** a parity population);
  median **1364.16 → 122.03 µs/token (11.18×)** at the tested scope. **Multiply-free** is scoped to a static
  own-symbol census; construction contains multiplies.
- **Span curriculum** (`moments`/`reset`, 512 steps): **73/80** and **76/80** vs parent 16/80, held-out lengths
  13/21/34 at 48/48 and 44/48, all four authored gates retained; exact resume byte-identical. Both arms are
  **worse than the parent on repository bits** (+0.142/+0.308). The span panel is a 4-value forced-feedback
  diagnostic; the **declared separate span RNG is not implemented**, so the `max_span=8` arms are not
  data-matched to the prior `max_span=3` root.
- **Negative (scoped):** payloads of two BPE tokens reproduce; three/four-token payloads fail the copy/stop
  sequence (`integrate.correct 16/64`). Selection is `32/32` correct, so this is a **multi-token copy-length gap**,
  not value coverage and not a geometry result.
- No general-language, coding, geometric-superiority or energy claim.

## Audit corrections applied
An independent evidence audit required six wording/scoping fixes (parity scope, "selector == signed matrix",
"multiply-free", tune provenance, span scope, `plan_payload_bytes`), all applied in the result record and pointer
docs. The audit also flags a **binding gap**: the receipts bind only the unchanged dispatcher's hash; the two
changed modules are unbound and the executable hash lives in the unsealed census file — a named delivery
obligation, not a numeric change.

## Resources
Runner wall 1007.9 s, peak RSS ≤ 162 MiB, evidence root 177 MB, net storage ≈ 1.45 GiB; charge **+3,000,000 ms**;
cumulative **449,439,320 / 455,500,000 ms**. No paid/external compute.

References #820, #1373.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

