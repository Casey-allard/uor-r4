# #711 — core: land sentencepiece Unigram core green — fix R5 error surface + pre-existing R6 uor-matmul wildcard

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-14T15:55:30Z
- **Updated:** 2026-10-09T13:08:52Z
- **Closed:** 2026-08-14T16:32:17Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/711
- **Merged:** 2026-08-14T16:32:17Z
- **Merge commit:** 808e829850a92e30a1e7bf901857b9256a69cc20
- **Base / Head:** main / UOR-Foundation:i639-3a-fix-r5-error-surface
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Fast-follow for #639-3a that makes **register-conformance fully green** — it has been red on main since #701, at two gates:

**R5 (`audit-limits`) — introduced by #710 (this chain).** The new `sentencepiece` module returned a bespoke `SentencePieceError`; R5 sanctions only `NotAProduct`/`ObservedBound`/`KappaError`/`SourceUnavailable`. #710 merged from its pre-fix commit (register-conformance is not a required check, and a queue merge landed the old commit mid-force-push), so the violation reached main.
- Fix: use `uor_r4_model_source::SourceUnavailable` (the surface `hf_bpe::from_dir`/`adapter_constructor` already use); every refuse-by-name message preserved. `SourceUnavailable` is `#[cfg(not(wasm32))]`, so the host/compile-side `sentencepiece` module is gated off wasm32 to match (the deployed runtime uses the exported tokenizer, never the source parser).

**R6 (`cargo deny` bans) — pre-existing since #701 (B1).** `uor-r4-model-source` declared `uor-matmul` as a `git`+`rev` dep with **no `version`**, which cargo-deny flags as a wildcard dependency on a public crate. It has been red on #703/#706/#708/#710 too (each merged anyway, non-required); this PR's R5 fix gets the job past R5 and exposes it.
- Fix: add `version = "0.1.0"` alongside the pinned `rev`, matching the exact `version`+`git`+`rev` pattern `uor-r4-graph-cli` already uses for `uor-matmul-core`. **The rev still governs resolution — `Cargo.lock` is unchanged and the pin is untouched**; compliance annotation only.

## Verification
R1–R6 register conformance (all green now) + `fmt` + `clippy --workspace --all-targets --all-features -D warnings` + wasm32 lib build + the 6 sentencepiece unit tests + the presence-gated integration test — all green (cloud with the real snapshot; Mac transfer verified; `Cargo.lock` unchanged). **No behavior change** to the parser/Viterbi and **no pin change**; no CID change. Kani stays red (pre-existing #712, uor-matmul global ASM, non-required).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

