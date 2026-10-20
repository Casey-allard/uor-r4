# #809 — feat(#655-C1e): ModelStore recognizes both bundle eras — recognize, don't migrate

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-19T00:13:43Z
- **Updated:** 2026-10-09T13:08:56Z
- **Closed:** 2026-08-19T00:35:27Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/809
- **Merged:** 2026-08-19T00:35:27Z
- **Merge commit:** 52a2b464a38ed2eff9b67ce1530f5e6998476c53
- **Base / Head:** main / UOR-Foundation:issue-655-c1e
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #655 (C1e, the last unstarted C slice; Q2 of the C1 design record, resolved additively — decision record included in `docs/serving_shared_loader_655_c1.md`).

- **Recognition, not migration**: `is_compiled_bundle` accepts the legacy plain-path triple (unchanged) AND the R4G1-era release-component shape (graph + signature artifact + tokenizer; `tless_store.bin` optional — the exact #655-D packaged set). R4G1-era directories get the correct `CompiledNotImported` diagnosis instead of `SourceNotCompiled`. No existing local store changes behavior.
- **Honest degradation**: `build_local_compiled_engine` builds store-less R4G1-era bundles with an EMPTY plain fallback tier (declines instead of serving; #785-C3 witness still names the engine; degradation logged loudly). Store-less + graph-less keeps the legacy required-file error.
- **One predicate**: the CLI compiled-presence probe drops its drifted any-single-file check for the shared crate-visible predicate (#787-style consolidation).

Tests: both-era recognition, three partial-directory falsifiers, R4G1-era `CompiledNotImported` diagnosis, store-less build with graph preferred + empty store, store-less graph-less falsifier.

Gates: fmt; clippy -D warnings (workspace, all targets); lib 255 pass.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_016X39CfJSLCU4KhjNP2XXTW

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

