# #481 — test(router): #480 — the query-projection asymmetry is real and fixing it does not pay

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-07T15:57:45Z
- **Updated:** 2026-08-07T16:23:23Z
- **Closed:** 2026-08-07T16:19:28Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/481
- **Merged:** 2026-08-07T16:19:28Z
- **Merge commit:** 413542bc4bf57e1b67b6199859b62aecba59b1a0
- **Base / Head:** main / UOR-Foundation:measure/480-query-projection
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #480.

I filed this issue claiming the adopted de-banding gain was going unrealized at serving. Measured, **that premise is refuted** — and the reason turns out to be more useful than the fix would have been.

## The asymmetry is real

Storage has been full-width since #465 (`banded_storage` defaults false). `retrieve_geometric_resonance` kept building a **band-only query projection**, zero outside `active_range`. Zeroed query coordinates contribute nothing to a dot product, so the cosine only ever saw the band however wide the stored vector was. That part of the filing was correct.

## Result

500k fixture, 46,342 windows, 500 probes, deranged-key null **0.0000 in every arm**:

| arm | store / query | top-1 | MRR | recall@20 | tie-mass |
|---|---|---:|---:|---:|---:|
| `banded` | banded / banded (pre-#465) | 0.6240 | 0.7195 | 0.9640 | 0.7580 |
| **`shipped`** | **full / banded (deployed)** | **0.6240** | **0.7179** | **0.9720** | 0.8120 |
| `symmetric` | full / full (the fix) | 0.6320 | 0.7238 | 0.9540 | 0.7860 |

Fixing it is worth **+0.0059 MRR / +0.0080 top-1** while costing **0.0180 recall@20**, against a pre-declared **+0.05 MRR** bar. A tenth of the threshold on the metric it was meant to move, and negative on recall.

Note `banded/banded` beats `shipped` on MRR — the de-banding gain doesn't reach this path from the storage side either.

The `shipped` row reproduces #479's figures exactly, which is the check that the knob added here leaves deployed behaviour untouched.

## Why it cannot pay

```
relevance = shared_count * 100 + sim * slice_norm + scope_boost
```

`shared_count` is word overlap × **100**; `sim` is bounded by 1. Observed relevances look like 415 / 215 / 115 — four, two, one shared words plus a fractional cosine. **The ranking is lexical; the cosine is a tie-breaker.** A vector-shape change can only reorder candidates already tied on word overlap.

The harness reports that tie-mass directly — **81.2%** of probes have a same-bucket competitor — so the reachable slice is large, and the shape *still* barely moves the ordering. The cosine had room; widening its input just doesn't change what it produces.

The 0.8948 figure in #442 came from a harness ranking by **cosine alone**. De-banding is worth what #442 measured *for cosine-ranked retrieval*; this serving path is not cosine-ranked. My filing was wrong to read the gap as unrealized gain.

## Not adopted — deployed behaviour unchanged

The symmetric shape sits behind `set_full_width_query` (default off, `#[serde(skip)]`):

1. the gain is a tenth of the bar and recall moves the wrong way;
2. an ordering change moves the pinned #421 anchor-accuracy rows, and `router_reconnect.rs` needs a scored R4G1 artifact that wasn't available here — shipping without that gate would be exactly the unmeasured move the run-contract discipline exists to prevent;
3. it becomes the right default the moment the lexical term stops dominating, so it's worth keeping reachable rather than deleting.

`retrieve_geometric_resonance` now carries a comment saying the asymmetry is known, measured and deliberate, pointing at the record — so this doesn't get re-filed from a fresh reading of the code.

## What this points at

The `* 100` lexical weight is hard-coded, never measured against alternatives, and dominates a geometric retrieval stack by two orders of magnitude. Measuring that weight — or ranking by cosine *within* lexical buckets rather than summing the two — is the experiment worth running. Recorded in the doc, deliberately **not** filed: it needs an owner and a pre-declared exit rule, and it's a larger question than this issue was.

## Gates

`cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `check_claim_wording.py`, `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib`, full `uor-r4-router` suite and dependent builds all green. Harness is `--ignored` (~11 min, three arms, no teacher).

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

