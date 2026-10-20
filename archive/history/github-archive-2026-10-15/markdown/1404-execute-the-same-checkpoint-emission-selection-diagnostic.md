# #1404 — Execute the same-checkpoint emission/selection diagnostic

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-27T01:15:24Z
- **Updated:** 2026-10-09T13:17:14Z
- **Closed:** 2026-09-27T01:17:12Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1404
- **Merged:** 2026-09-27T01:17:12Z
- **Merge commit:** 9901512c000a973cee5f25d87cbaf5947b0fbe1d
- **Base / Head:** main / UOR-Foundation:codex/emission-localization-20260926
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Executes the single recommended work card from #1398's closeout: the same-checkpoint emission/selection causal diagnostic, read-only, with a new additive `joint-emission-trace` instrument.

References #973 and #820. This is a diagnostic: it promotes no candidate, changes no weights or decoding, adds no gate or panel, and leaves the frozen prose verdict intact.

## Instrument and parity

- New additive `joint-emission-trace` subcommand in `uor-r4-training`: `generate_traced` (retained `generate_inner` untouched), `incremental_copy` visibility only, and a new `output_vocabulary` duplicate of the existing softmax block. Executed source `9bed4d8b`; binary sha256 `cc113728…`; built offline with the frozen feature set.
- `PARITY_EXACT` against the retained `generations.json` in both arms: 0 mismatches, maximum absolute float delta `0.0`, all five stories/decisions. An independent principal Python comparison also found 0 field mismatches. Mandatory runtime checks (Q32 interval contains the actual draw; replica post-draw sampler state matches) passed for every decision; maximum mixture reconstruction error `7.24e-8` / `6.34e-8`.
- Runs repeated after rebasing onto current `main` with identical results (delivered source == executed source).

## Result

- Witnessed decisions (quaternion 37–42; householder pair 25–30; both story seed 2014): **8 departure, 2 near_tie, 2 top_choice, 0 copy_dominated**.
- Whole seed-2014 story (128 decisions/arm): quaternion 61 top_choice / 18 near_tie / 49 departure / 15 copy_dominated; householder pair 63 / 19 / 46 / 16.
- Predeclared rule → supported interface: **selection policy**. The next work card (not started) is one separately authorized bounded same-checkpoint selection-policy diagnostic with no weight change; it must also explain the retained greedy source-panel regressions. A better-ranked token is not a coherent alternate trajectory, and nothing is promoted.

## Files

- `docs/integration/emission-selection-diagnostic-plan-2026-09-27.md` (predeclared decision + cost)
- `docs/integration/emission-selection-diagnostic-result-2026-09-27.md` (result, limits, cost)
- `docs/evidence/emission-selection-diagnostic-2026-09-27.json` (identities, witness rows, counts)
- `crates/uor-r4-training/src/{joint_model,joint_evaluation,joint_campaign}.rs`, `crates/uor-r4-training/README.md`
- `docs/integration/current-state.md`, `README.md`

## Checks (executed locally)

- `cargo fmt -p uor-r4-training -- --check` clean.
- `cargo check --offline -p uor-r4-training --features metal,cpu-accelerate,reference-accelerate` 0 errors.
- Focused tests `emission_trace_*`: 4 passed.
- Both witness runs completed with `PARITY_EXACT`.
- `python3 scripts/check_claim_wording.py` clean.
- Report roots: `emission-trace-{quaternion,householder_pair}-{1,2}` under the continuation investigation directory; four roots 22.5 MiB.

## Notes

The repository's historical CI status names are queue compatibility acknowledgements and execute no formatting, Clippy or tests; the checks above are the executed local evidence.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

