# #1015 — A1Q-H2: localize R4 trace-state signal loss

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-30T17:48:59Z
- **Updated:** 2026-10-09T14:16:49Z
- **Closed:** 2026-08-30T17:50:13Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1015
- **Merged:** 2026-08-30T17:50:13Z
- **Merge commit:** 46dad4e4844e7a1716e91cdf8201d6389c7bceed
- **Base / Head:** main / UOR-Foundation:issue-1012-r4-softmax-trace-observability
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

- Adds the strict four-fold R4 softmax trace observability ladder and immutable 64-feature state snapshot.
- Records authoritative attempt 02 after replacing attempt 01's invalid donor control with one fixed within-support label rotation.
- Synchronizes repository direction with the result and transitions the active attention work to #1014.

## Authoritative result

- Terminal: `INSUFFICIENT_SUPPORT_COVERAGE`
- Result CID: `blake3:11f890def300cdafd689ef7cbfcf28e46b693e669a46edfa7431c672582db4a2`
- Implementation revision: `3ef7af9860baf55fbe43a9b5207c60fd3a737564`
- Exact two-pass replay: `PASS`
- Aggregate primary coverage: `0.6202622204224402`
- Minimum-fold coverage: `0.3469116829611222`, below the frozen `0.50` floor
- Full Q/K/V CE: `2.215410922655504`
- Suffix-baseline CE: `2.215064603216862`
- Suffix-minus-full improvement: `-0.0003463194386417179` nats, favorable in `0/4` folds
- Label-control-minus-full separation: `1.3807454322642605` nats, favorable in `4/4` folds
- Teacher top-1 agreement: `14/26` for both full Q/K/V and suffix
- Actual-next top-1: `6/26` for both

The control establishes instrument sensitivity, but insufficient per-fold support forbids boundary attribution and the measured full trace adds no predictive gain over the suffix baseline. This observability path stops rather than expanding support or adding another diagnostic ladder.

## Direction

#1014 now owns the direct next deliverable: train ordinary causal softmax attention end to end in R4 coordinates on a fresh untouched split and require autonomous decoded generation in the same issue.

## Verification

- `cargo fmt --check`: PASS
- `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings`: PASS
- `cargo test --workspace --offline --no-fail-fast`: PASS
- graph-format `no_std` and `alloc` checks: PASS
- WASM target check: PASS
- claim-wording and diff-integrity checks: PASS
- focused post-lint projection test: PASS

## Nonclaims

This does not establish geometric advantage, coherent source-free generation, reasoning, transformerless deployment, exact table lowering, runtime efficiency, hosted-chat readiness, or release readiness. It does not falsify all trace distillation, all geometric representations, or attention itself.

Closes #1012
References #973
References #1014


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

