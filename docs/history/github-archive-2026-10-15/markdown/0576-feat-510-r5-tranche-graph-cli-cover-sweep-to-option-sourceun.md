# #576 — feat(#510): R5 tranche — graph-cli cover_sweep to Option/SourceUnavailable (graph-cli 34->27)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T13:24:43Z
- **Updated:** 2026-10-09T13:09:02Z
- **Closed:** 2026-08-10T13:43:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/576
- **Merged:** 2026-08-10T13:43:38Z
- **Merge commit:** 9f502289780c5d1c05be56fd468e62f6c8ed14ff
- **Base / Head:** main / UOR-Foundation:r5/graph-cli-cover-sweep-total
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## R5 tranche — graph-cli `cover_sweep` → `Option` / `SourceUnavailable`

Second graph-cli PR in the #510 R5 rearchitecture. Converts the self-contained
cover-sweep module (issue #70 rate–distortion sweep) to the sanctioned error surface.

### Classification
- **Host boundary → `SourceUnavailable`**: `load_inputs` (disk reads + artifact/corpus
  validation), `parse_cover_sweep_options` (CLI arg parse), `cover_sweep_command`
  (the top-level command that writes `cover_sweep.json`).
- **Pure compute → `Option`**: `run_point`, `run_sweep`, `reconstruction_null`
  (+ its inner `score_with` closure). These operate on already-loaded inputs;
  `None` means degenerate inputs. `induce_cover` / `evaluate_gate_c` already return
  `Option`, so the `ok_or_else` wrappers collapse to `?`. `cover_sweep_command`
  maps the aggregate `None` to a single `SourceUnavailable` reason.
- **Defect → panic**: vocab overflow (`token_codes.len() / STAGES` exceeding `u32`)
  is impossible in practice on a validated artifact → `.expect()`.

### Compatibility
The `lib.rs` dispatch arm maps `SourceUnavailable` back to the `String` surface
`run()` still uses. The `reconstruction_null` / `marginal_*` integration tests call
`load_inputs` / `reconstruction_null` via `.expect()`, which is compatible with both
`Result` and `Option` — no test changes needed.

### Audit
`xtask audit-limits`: graph-cli **34 → 27** (`cover_sweep.rs` now at 0).
Remaining in graph-cli: `lib.rs` (26), `convert_r4g1.rs` (1).

Gauntlet: build / fmt --check / clippy --all-targets / `cargo test -p uor-r4-graph-cli`
(24 pass) clean; root `uor-r4-wasm-router` builds.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

