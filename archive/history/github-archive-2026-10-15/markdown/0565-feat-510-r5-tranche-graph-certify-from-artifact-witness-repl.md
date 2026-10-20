# #565 — feat(#510): R5 tranche — graph-certify from_artifact + witness replay to total (certify 20->18)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T00:59:01Z
- **Updated:** 2026-10-09T13:09:00Z
- **Closed:** 2026-08-10T01:15:28Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/565
- **Merged:** 2026-08-10T01:15:28Z
- **Merge commit:** c431fd12b40a080266b0dc7a123d38f0a1ead649
- **Base / Head:** main / UOR-Foundation:r5/from-artifact-witness-total
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## R5 tranche (#510): graph-certify `from_artifact` + witness replay → total

Converts the scorer-construction and Theorem-6 witness-replay entry points in
`score_runtime.rs` to a total surface.

**`GraphScorer::from_artifact` → `Option<Self>`.** The bytes are validated as a
product: two-stage parse (`NotAProduct`), CID integrity (`KappaError`), the
sections the scorer needs, and — when supplied — a teacher container whose
blake3 must reproduce HEAD `teacher_cid`. All collapse to `None` here because
every caller either rebuilds from self-produced bytes (a `None` is then a defect
it panics on) or only needs the yes/no answer.

**`verify_witness_replay` → `Option<ReplayError>`** (`None` = replay holds). Every
`ReplayError` verdict variant is preserved as the `Some` payload — no diagnostic
lost, and audit is satisfied since it is no longer a `Result`. The teacher-cid
mismatch, previously recovered by *sniffing `from_artifact`'s error string*, is
now detected by an independent pre-check, so `TeacherCidMismatch` stays
distinguished from a generic artifact defect.

### Callers adapted in-PR
- **score.rs gate-C**: the six `from_artifact` rebuilds run on the just-emitted
  self-produced artifact → `None` is a compiler defect → `.expect(...)`; the
  witness-replay sampling switches `.is_err()` → `.is_some()`.
- **graph-cli** `graph_infill` and **api** `engine::load` map `None` to their own
  `Result` surfaces via `.ok_or_else(...)`.
- **tests/bdd.rs** `load_fmm_candidate` drops its now-redundant `.ok()`.
- The nine `fwda_roundtrip` / `router_reconnect` / `status_policy` `.expect(...)`
  call sites are unchanged — `Option::expect` is a drop-in.

### Audit
`graph-certify` audit-limits: **20 → 18** (score_runtime.rs 15 → 13). No
sanctioned error types introduced; `ReplayError` is retained as an `Option`
payload, not a `Result` error.

### Gauntlet
`cargo fmt --check`, `cargo build --workspace --all-targets`, `clippy`
(certify + cli + api, all-targets), `wasm32-unknown-unknown` build of
graph-certify, and the graph-certify lib / `fwda_roundtrip` / api lib /
`status_policy` / `status_policy_census` suites — all green.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

