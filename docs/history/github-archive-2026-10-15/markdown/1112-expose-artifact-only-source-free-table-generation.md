# #1112 — Expose artifact-only source-free table generation

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-04T05:03:14Z
- **Updated:** 2026-09-04T05:05:24Z
- **Closed:** 2026-09-04T05:04:32Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1112
- **Merged:** 2026-09-04T05:04:32Z
- **Merge commit:** 04bca86c46d5cdf70dfe33ab25b1426ddcbeaf13
- **Base / Head:** main / UOR-Foundation:codex/973-prompt-conditioned-generation
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What changed

Adds `r4 source-free-generate`, an artifact-only inference path for the established `SFTBL001` lexical table. The command validates and loads canonical packed bytes, continues the exact prompt through the existing deterministic table runtime, and emits either completed text or compact JSON with the artifact identity and stop reason. The source corpus, teacher, provider, and source weights are not opened by this invocation.

README now shows the direct artifact-loading command separately from table construction/evaluation.

## Direct behavior

Built the `r4` binary and loaded the retained 35,655,288-byte #973 artifact at `blake3:ccdc399731cb866a329be478467a434cda4e445813421e5d17c21ccc87288297`.

- `The United States` continued as `. It is the most important thing to do so.\n 1985 – The first people`.
- `Albert Einstein` continued as ` (14 January 1798 - 4.5 million people live in the United States.`.

Both emitted 16 lexical units and stopped at the requested cap. The outputs are distinct, demonstrating prompt-suffix-conditioned artifact execution. The Einstein continuation is factually wrong, so this is not coherent language, semantic correctness, or completion of #973.

## Checks

- `cargo check --bin r4`
- `cargo clippy --bin r4 -- -D warnings`
- `cargo fmt --all -- --check`
- `python3 scripts/check_claim_wording.py`
- direct CLI help plus two real-artifact generation invocations

References #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

