# #579 — feat(#510): R5 tranche — graph-cli graph/quantum commands to total (graph-cli 16->12)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T15:11:36Z
- **Updated:** 2026-10-09T13:09:02Z
- **Closed:** 2026-08-10T15:30:24Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/579
- **Merged:** 2026-08-10T15:30:24Z
- **Merge commit:** 9f7f2c3022b2e667dea2feeaf892e9f4a55057ec
- **Base / Head:** main / UOR-Foundation:r5/graph-cli-graph-quantum-commands
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## R5 tranche — graph-cli graph/quantum commands → total

Third `lib.rs` cluster: the internal-only graph-serving and quantum-demo commands.

- `graph_infill_command`, `graph_command` → `SourceUnavailable` (host command: reads
  the scored R4G1 / teacher containers, parses the skeleton spec; `parse_skeleton`
  now propagates directly via `?`).
- `cd_compile_command`, `quantum_eval_command`: their `Result<(), String>` was
  spurious (no fallible operation — bodies only compute + `println!`) → return `()`,
  and the `run()` dispatch arms drop the `?`.
- `run()` dispatch: the `graph` arm maps `SourceUnavailable` back via `to_string`;
  the `cd-compile` / `quantum-eval` arms lose their `?`.

Internal-only — none are called from the root package. The `graph_infill`
integration test now asserts on the `SourceUnavailable.reason` field.

### Audit
`xtask audit-limits`: graph-cli **16 → 12** (`lib.rs`).

Gauntlet: build / fmt --check / clippy --all-targets / `cargo test -p uor-r4-graph-cli`
(24 pass) clean; root builds.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

