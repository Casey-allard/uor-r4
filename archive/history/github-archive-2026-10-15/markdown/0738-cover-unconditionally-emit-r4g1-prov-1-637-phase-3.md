# #738 — cover: unconditionally emit R4G1 PROV/1 (#637 phase 3)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-16T18:59:00Z
- **Updated:** 2026-10-09T14:17:15Z
- **Closed:** 2026-08-16T19:20:29Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/738
- **Merged:** 2026-08-16T19:20:29Z
- **Merge commit:** 146a976e09c012cc673e566dcb131c0fde067adb
- **Base / Head:** main / UOR-Foundation:issue-637-3-default-provenance
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #637

## Summary

Phase 3 of #637: flips `cover`'s default output to always carry an R4G1 PROV/1 provenance section, superseding phase 2a's opt-in `--emit-provenance` flag.

- When the caller passes `--source-manifest-kappa` / `--geometry-projection` / `--attention-operator` / `--dense-operator`, those digests bind into the PROV/1 section exactly as phase 2a did.
- When the caller passes none of them, the artifact still gets a present-but-empty PROV/1 section (presence bitmap zero, zero evidence roots) — a deliberate, valid record per `emit_r4g1_with_provenance`'s own documented contract (proven by the existing `emit_r4g1_with_provenance_binds_a_prov_section_without_disturbing_the_unbound_path` test in `crates/uor-r4-graph-compiler/tests/induction.rs`), not a legacy omission.
- `--emit-provenance` is retired as a gate and kept only as a no-op flag for backward CLI compatibility with any phase-2a caller/script that already passes it.

## Maintainer decision

Casey (2026-08-16): proceed with the default flip without waiting on further collaborator review, per the recorded maintainer sign-off in the #637 comment thread. Full context and the plain-language decision statement are posted on the issue.

## Re-pin

No pinned fixture in this workspace hardcodes `cover`'s emitted R4G1 bytes or κ literally, so no re-pin was required — verified by running the full `uor-r4-graph-cli`, `uor-r4-graph-compiler`, and `uor-r4-graph-format` test suites (unit + integration, including the presence-gated `gpt2_cover_certifies_source_parity_rows` canary against the real local GPT-2 snapshot) with zero regressions. AGENTS.md's `kappa_reproduction`/`TLESS_REPIN_WRITE` Gate E procedure re-pins an unrelated llama2.c reference-model fixture and is untouched by this change.

## Verification

- `cargo fmt --check -p uor-r4-graph-cli`
- `cargo clippy -p uor-r4-graph-cli --all-targets -- -D warnings`
- `cargo test -p uor-r4-graph-cli` (126 unit + all integration, including the gpt2 canary)
- `cargo test -p uor-r4-graph-compiler --tests`
- `cargo test -p uor-r4-graph-format --tests`
- `cargo run -q -p xtask -- audit-limits` (R5)
- `cargo run -q -p xtask -- check-model` (CM-01)

All green.

Files: `crates/uor-r4-graph-cli/src/lib.rs`.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

