# #237 — certify C-section feeds rotation offsets as context lags — R4G1 row evaluates scrambled context

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-28T03:00:01Z
- **Updated:** 2026-07-28T04:32:53Z
- **Closed:** 2026-07-28T04:32:53Z
- **Labels:** bug
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/237
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary

The `certify` C-section (R4G1 graph path) builds its evaluation context by indexing the corpus with **vector rotation offsets used as time lags**:

```rust
// crates/uor-r4-graph-certify/src/certify.rs (hist_at, formerly inline)
for (idx, w) in rot.iter().enumerate().take(WINDOW) {
    hist[idx] = c.input[i.saturating_sub(*w)];
}
```

`rot` is `derive_rotations()` (`crates/uor-r4-core/src/transformerless/compiler.rs:651`), defined as `(j*17) mod D` — i.e. `[0, 17, 34, 51, 68, 85, 102, 119, 136]`. These are **dimension-space rotation amounts** consumed by `bundle_plain` to rotate each history token's decoded row before summing. They are not, and were never, temporal offsets.

The result: `predict_token`/`predict_distribution` receives tokens from positions `i, i-17, i-34, ..., i-119` instead of the last 8 consecutive tokens.

### Three compounding defects

1. **Wrong tokens**: context sampled at stride-17 positions rather than the actual preceding window.
2. **Wrong order + phantom most-recent token**: `predict_distribution` treats the **last** element of `context_tokens` as most recent, but `hist[0]` gets `c.input[i]` (the true most-recent) and slot `WINDOW` (the 9th) is never written — so the suffix walk starts from a trailing `0` token. (The `context_tokens[0] <= 1` strip partially masks the misplaced current token but not the trailing zero.)
3. **No story-boundary check**: unlike `history_token` (`runtime.rs:195`), which refuses to cross `c.story` boundaries, the stride-17 indexing freely pulls tokens from previous stories.

### Why it matters

This plausibly explains the all-zero C row independent of graph training state: the suffix DFA is walked with scrambled context ending in token 0, so it can rarely (never, at measurement precision) match a real emitting path — a converted graph demonstrably *can* emit under sane contexts (`crates/uor-r4-graph-runtime/tests/r4g1_runtime_test.rs` predicts successfully on a store built with the same `convert(..., None)` path). Until this is fixed, the C row measures context-construction garbage, not graph quality — even after the readiness guard from #232/#236 (which will correctly classify this shape as degenerate and skip, but the row should eventually *measure something*).

### Suggested fix

Build the window with the same semantics the compiler uses for observation — `history_token`'s consecutive-lag, story-bounded walk — ordered oldest to newest with `c.input[i]` last:

```rust
let mut hist: Vec<u32> = Vec::with_capacity(WINDOW);
for j in (1..=WINDOW).rev() {
    if let Some(t) = history_token(c, i, j) { hist.push(t); }
}
// hist now ends with c.input[i] (j == 1), story-bounded, chronological.
```

Then re-measure the C row on a scored graph — it is currently unknown whether the graph path's true numbers are good or bad, because they have never been measured with coherent context.

### Environment

Found by code inspection at `main` @ 7ebb653 during the #232 fix (readiness guard, PR #236); flagged there as out of scope.


## Comments (1)

### comment 5099767532 — 2026-07-28T03:58:55Z — Casey-allard

Taking this (assigned). Plan, branch `issue-237-eval-context-lags`:

- New `pub fn eval_context(c, i) -> Vec<u32>` in the certifier: last WINDOW tokens, oldest → newest ending at `input[i]`, story-bounded — built on `runtime::history_token` (now `pub`, same semantics observation/bundling uses), so evaluation and observation share one context definition.
- Both the #232 readiness probe and the full C-row evaluation switch to it; the rotation table keeps its legitimate uses (bundling, equality witnesses).
- Tests: chronological window, story-boundary truncation, position-0 edge, and a regression guard against the three old defects (stride-17 sampling, reversed order, phantom trailing zero).

Note for interpretation: after this lands, the C row measures coherent context for the first time — its numbers may change materially from the historical all-zero row, and the #232 skip-marker should fire only for genuinely untrained graphs rather than for scrambled input.



---
_Comment/review ids in this file: 5099767532_

