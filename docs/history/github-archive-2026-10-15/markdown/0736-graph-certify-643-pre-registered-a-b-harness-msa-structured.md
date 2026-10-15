# #736 — graph-certify/#643: pre-registered A/B harness, msa-structured-selector/1 vs r4-route-attention/1

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-16T18:05:07Z
- **Updated:** 2026-10-09T14:17:16Z
- **Closed:** 2026-08-16T18:23:56Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/736
- **Merged:** 2026-08-16T18:23:56Z
- **Merge commit:** 2ea108dfae6377044fc9751a21a53ba5634bc133
- **Base / Head:** main / UOR-Foundation:issue-643-ab-harness
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #643.

## What this ships

The last open item on #643: an evaluation harness that runs `msa-structured-selector/1`'s deployed packed kernel through the SAME synthetic held-out corpus and whole-model scope `r4-route-attention/1`'s #605 replacement ladder measures against, then judges the result against the exit rule pre-registered (before any run) in `msa_selector_643.rs`.

- `crates/uor-r4-graph-certify/src/msa_ab_harness.rs` (new): drives both deployed kernels (`route_attention_step`, `msa_selector_step`) over the identical causal-prefix candidate table, reuses `route_fit_report::scope_metrics` verbatim for the teacher-forced top-1/bits-per-token numbers (made `pub(crate)`, no behavior change), and adds a `#457`-convention unigram floor computed over the same corpus.
- `msa-structured-selector/1` needs no fitting step, unlike `r4-route-attention/1`: its classification is `candidate_id mod 11`, a pure function of position — so `candidate_id = position` feeds straight into the deployed kernel, no projection/threshold/learned parameter.
- `crates/uor-r4-graph-certify/src/route_fit_report.rs`: `scope_metrics`/`ScopeMetrics` made `pub(crate)` so this harness can reuse them (no behavior change, verified byte-identical route-fit ladder results).

## Measured result (2026-08-16 fixture)

```
r4-route-attention/1 (fitted, whole-model): top1 0.990234  bits/token 4.971638
msa-structured-selector/1 (position-only):  top1 0.642578  bits/token 5.532591
unigram floor (this corpus):                top1 0.035156  bits/token 5.854087
gates: top1_margin=false  bits_not_worse=false  beats_unigram=true
verdict: NEGATIVE
```

`msa-structured-selector/1` clears the trivial unigram floor by a wide margin but falls well short of the fitted, content-aware `r4-route-attention/1` arm on both pre-registered gates — consistent with #643's own pre-registered mod-12 probe-chain warning that class representation capacity does not, by itself, survive selection dynamics without content-aware encoding. Pinned as a regression test (`msa_selector_measured_negative_against_fitted_route_attention`).

Per the pre-registered exit rule, this is a NEGATIVE result: `msa-structured-selector-dormant` (already registered in `model/ledger.toml`) is the disposition, unchanged. Once this merges I'll post the comparison table on #643 and close it.

## Verification

- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` (both clean)
- `cargo test -p uor-r4-graph-certify -p uor-r4-graph-compiler -p uor-r4-graph-format -p uor-r4-graph-runtime` — 0 failures across all four crates
- `cargo run -q -p xtask -- audit-limits` (R5) — clean
- New tests: determinism (two independent corpus generations produce a byte-identical report), unigram-floor unit test, and the pinned measured-result regression test above


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

