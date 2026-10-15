# #676 — graph-cli: presence-gated GPT-2 end-to-end compile canary (#670, increment 1)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-13T22:18:57Z
- **Updated:** 2026-10-09T14:17:10Z
- **Closed:** 2026-08-13T22:39:13Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/676
- **Merged:** 2026-08-13T22:39:13Z
- **Merge commit:** c680a1326b150c445ad1dbbbee355509ca758dae
- **Base / Head:** main / UOR-Foundation:i670-gpt2-compile-canary
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Refs #657 / #670. **First increment of #670** — does not close it (see "Remaining" below).

## What this proves
A pinned `openai-community/gpt2` source compiles **end-to-end** through `compile_hugging_face_with_progress` — the same entry the CLI `compile` uses. That entry loads the source via the `Teacher` enum (GPT-2 dispatch, #671), self-generates a bounded teacher corpus, and emits the table-native artifact bundle. So this canary exercises, on the **real GPT-2 teacher**, the learned-absolute attention operator (#672), the batched executor (#675), and the tokenizer pin (#673), end to end.

## Presence-gated (#599 three-state)
The 548 MB snapshot is a dev/local compiler input, never a CI download — when it is absent the test reports UNAVAILABLE and passes vacuously (never a silent skip of a real failure). When the snapshot is present:
- the compile emits a valid table-native bundle (`tless_artifacts.bin` parses through `compiler::parse_artifacts`);
- a **deterministic double-run** produces byte-identical artifact bytes (the corpus generator is integer-seeded; per-machine matmul is deterministic).

## Regression
Additive — a single new test file, no SmolLM2 path touched, so SmolLM2 compiles byte-unchanged.

## Remaining (tracked, keeps #670 open)
The R4G1 route artifact + Gate C parity + the #606 **source-parity certificate rows** (operator #602 / geometry #600 / tokenizer #601) drive the separate cover → score → certify stages. That extension is best built and verified **with the snapshot present**, since a canary you can't execute isn't proven. The full pipeline map (entry points, the `--attention-operator`/`--geometry-projection` report wiring, the cert path) is recorded in project memory `gpt2_canary_670.md`.

## Verification (local, pre-push)
- `cargo fmt --check`; `cargo clippy -p uor-r4-graph-cli --tests --all-features -- -D warnings` (clean)
- The canary compiles and runs (UNAVAILABLE path — no GPT-2 snapshot on this machine). No new dependencies.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

