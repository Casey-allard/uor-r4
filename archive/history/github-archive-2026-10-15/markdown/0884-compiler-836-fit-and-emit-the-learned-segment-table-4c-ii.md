# #884 — compiler(#836): fit and emit the learned segment table (4c-ii)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-21T01:28:22Z
- **Updated:** 2026-08-21T01:56:22Z
- **Closed:** 2026-08-21T01:50:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/884
- **Merged:** 2026-08-21T01:50:06Z
- **Merge commit:** 1be0bd5acd3b75423d7a65f914595c4c13db512a
- **Base / Head:** main / UOR-Foundation:issue-836-fit-table-pr
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## #836 4c-ii — fit and emit the learned segment table

Item C of #822, continuing the #836 lowering. 4c-i wired the deployed
`R4Engine` to **consume** a learned content→candidate residual table when a
PSTATE section carries one. This increment **produces** that table, closing the
fit → emit → load → consume chain end-to-end.

### What lands

- **`uor_r4_graph_compiler::segment_fit::fit_segment_table`** — the
  compiler-side lowering of the #834 §6.2 Ψ segment arm. It runs the *same*
  content→teacher-argmax co-occurrence tally over the whole-prompt window
  (`induction::context_window`) restricted to TRAIN positions, caps to top-`K`
  per key, then **quantizes to the integer ScoreQ weights the P-4 serving path
  sums**. Two float operations of the reference arm are argmax-invariant and are
  discharged here at fit time (where floats are allowed):
  - per-key normalization `count/total` → a fixed per-`(key,candidate)` weight
    `round((count/total)·2²⁰)` in pure integer arithmetic;
  - the per-prompt `/ncontent` is a single positive scalar shared by every
    candidate of a prompt, so dropping it never changes the argmax.

  A fully-predictive content token earns exactly one `boost` worth of weight
  (`1<<20`, matching the recurrence lane's boost magnitude).

- **`convert_r4g1::convert_with_segment_table`** — emits the fitted rows into
  the PSTATE section. Empty rows are byte-identical to
  `convert_with_segment_lane` (config-only descriptor → recurrence lane); every
  other section is unchanged (absent-section identity). `convert_inner` now
  threads `(descriptor, rows)`.

- **`R4Engine::segment_learned_rows`** — observability seam reporting how many
  learned rows the loaded lane carries (`None` on the recurrence lane or no
  PSTATE), so a caller can confirm a compiled bundle's table reached the engine
  without exposing the table itself.

### Verification

- `segment_fit` unit tests: the fit recovers the dominant content→argmax
  association and respects the key/candidate bounds; quantization maps a certain
  rate to one boost and never rounds a retained pair to a silent zero.
- `pstate_engine_836` integration tests: the fitted table round-trips through
  `convert_with_segment_table` byte-faithfully and the engine ingests exactly
  those rows; a config-only / no-PSTATE bundle carries no learned table;
  emission is section-preserving and deterministic.
- The engine's re-rank arithmetic over the learned table is already pinned by
  the inline `table_*` unit tests (4c-i).

### Gates

`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D
warnings`, `cargo run -p xtask -- validate` (check-model, audit-limits/deferral,
gnaf firewall/root/scan/conformance all green), and the
`uor-r4-{api,core,graph-format,graph-runtime}` suites (482 passed, 0 failed).

The lane stays **dormant** — no serving row in `model/ids.toml`, CONFORMANCE
unchanged. The causal-utility verdict is recorded separately in 4c-iii.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_01JNGVP646hV5uuCZGMKfxTk


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

