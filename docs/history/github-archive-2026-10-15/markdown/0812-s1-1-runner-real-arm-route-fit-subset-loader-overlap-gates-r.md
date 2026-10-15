# #812 — S1-1 runner: real-arm route-fit subset loader, overlap gates, route-fit-real CLI (#804, #605)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-19T00:45:02Z
- **Updated:** 2026-10-09T13:08:53Z
- **Closed:** 2026-08-19T01:05:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/812
- **Merged:** 2026-08-19T01:05:53Z
- **Merge commit:** 63d62473a7517e0d04d981a0450fadef52b4bf75
- **Base / Head:** main / UOR-Foundation:issue-804-s1-1-runner
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Lands the **S1-1 runner** for the #605 real-arm ladder (route-attention fit, #804), ahead of tonight's traced-corpus completion — per the amendment + auto-run decision declared on #605.

- **Subset loader** (`route_fit.rs`): `RouteTraceSubset` + `load_route_trace_corpus_subset` stream the traced corpus shard-by-shard and keep the declared window (first 1000 story ordinals × first 64 positions), with domain-tagged subset κs (`uor-r4-route-trace-subset/1`, blake3 over per-record hashes) and a per-story contiguity check inside the kept window.
- **Sidecar decoder** (`observation.rs`): `decode_probability_metadata` exposes the per-shard probability-row decoder the streamed reader needs.
- **Overlap gates** (`route_fit_report.rs`): `RealArmOverlapReport` (`uor-r4-real-arm-overlap/1`) + `run_real_arm_overlap_gates` — pre-registered gate order (vacuity → deployed-kernel runtime checks → overlap bar `max(null_factor × best_null, floor)`), N1 = permuted codes, N2 = deranged supports.
- **CLI** (`graph-cli`): `transformerless route-fit-real --corpus … --source … --stories 1000 --positions 64 --out …` — loads the subset, fits, runs the gates, writes the report JSON; a FAIL verdict is a recorded outcome, not a command error.

S1-2 (restricted-forward parity on the deployed kernel) stays gated on an S1-1 PASS, exactly as amended.

## Tests

- `real_arm_overlap_gates_pass_on_synthetic_and_fail_on_seeded_gates` — synthetic PASS, byte-determinism via `PartialEq`, and both falsifiers (overlap floor 1.1 → FAIL; N2 vacuity fraction 0.0 → FAIL vacuous).
- `subset_loader_matches_full_load_and_truncates_at_the_declared_bound` — subset loader ≡ full loader on an in-window corpus; truncation honors the declared bound.
- Workspace `cargo clippy --all-targets -- -D warnings` clean, `cargo fmt --check` clean, compiler lib suite 203 passed, certify suite 41 passed.

## Refs

- #804 (S1 decision + amendments), #605 (ladder; amendment issuecomment-5335971782 + addendum issuecomment-5335988956)
- Measurement-only: no serving-path changes.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_016X39CfJSLCU4KhjNP2XXTW


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

