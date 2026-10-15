# #184 — feat(api): uor-r4-api — typed integration facade for downstream consumers

- **Type:** Pull request
- **State:** closed
- **Author:** @auser
- **Created:** 2026-07-25T01:48:44Z
- **Updated:** 2026-10-09T13:09:08Z
- **Closed:** 2026-07-25T05:26:41Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/184
- **Merged:** 2026-07-25T05:26:40Z
- **Merge commit:** 4bc4fa2772e39b78e03484d523ae22b8ff03dfeb
- **Base / Head:** main / UOR-Foundation:feature/typed-integration-facade
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Additive `uor-r4-api` crate — a typed integration surface for downstream
consumers (the `hologram-ai` rewrite, Hologram-Technologies/hologram-ai#20) —
wrapping existing functionality. **No algorithm, scoring, or legacy-path
changes.** One commit.

## What it adds

- Typed `CompileRequest` / `CompileOutcome` over the existing three-stage
  pipeline (structural resumability signal, structured `CompileError`, progress
  events) — a library entry point in place of the `&[String]` /
  `Result<(), String>` CLI-shaped functions.
- `R4Engine::load` from byte slices (graph two-stage validation + CIDs before
  init) — no filesystem, no sidecar paths.
- Allocation-free `predict_next_into` / `generate_into` with typed abstention.
- `Tokenizer::from_bytes` and an ABI-version surface.
- The root package's runtime glue moves into the library
  (`src/r4g1.rs` → `crates/uor-r4-api/src/engine.rs`), preserving the deployed
  status-policy block; the integer-only source scan now covers its new home.

## Boundaries

Purely additive library surface. Does not redesign compilation, change scoring
or model quality, remove legacy paths, add `.holo` logic, add networking, or add
SDK-specific types.

Reported gates (from the commit): fmt; clippy `--all-targets --all-features
-D warnings`; workspace tests; the no_std ladder; kappa reproduction; the
claim-wording check.

Full contract: `docs/upstream/uor-r4-contract.md` in
Hologram-Technologies/hologram-ai#20.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

