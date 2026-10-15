# #614 — fix(#515): correct ledger — long_context/compiler_scaling wired, patch_chain wording

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-10T23:18:59Z
- **Updated:** 2026-10-09T14:17:26Z
- **Closed:** 2026-08-10T23:38:13Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/614
- **Merged:** 2026-08-10T23:38:13Z
- **Merge commit:** a0a2a3bb5ad1a0c8260aec4ebc566749f70b0dea
- **Base / Head:** main / UOR-Foundation:r5/ledger-fix-wired
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## #515 — correct three more register entries the build disproved

Final ledger correction closing out the audit half of #515. Each of these was registered dormant in #590 but the build shows otherwise:

- **Remove `long-context-dormant`.** `long_context::certify_long_context()` is the final call in the `certify()` entrypoint (`crates/uor-r4-graph-certify/src/certify.rs:1500`), non-test code. Gating it breaks the default build.
- **Remove `compiler-scaling-dormant`.** `compiler_scaling` backs the shipped proof surface — `uor-r4-proof-model`'s `verify_compiler_scaling_certificate_compliance` (a non-test `pub fn`). Gating it breaks the default build.
- **Revise `patch-overlay-dormant`.** The claim wrongly listed `patch_chain` (`crates/uor-r4-graph-runtime/src/patch_chain.rs`) as off the serving path; it is wired — held by `R4G1Runtime` as its patch chain. The dormant portion is section emission (`patch_induction`, gated in #593) plus lifecycle discipline (`patch_lifecycle`, gated in #591); the statement and activation gate are updated accordingly.

`bott_fock` / `cd_space` claims are kept — accurately scoring-path-dormant, reachable only via the `graph-cli` `cd-compile` / `quantum-eval` diagnostic subcommands.

`CONFORMANCE.md` regenerated via `check-model --write`. `check-model` / `audit-deferral` / `audit-limits` pass; `cargo xtask gates` runs (open claims 19 → 17).

Refs #515.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

