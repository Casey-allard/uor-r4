# #693 — graph-cli: GPT-2 cover → R4G1 source-parity canary (#670 increment 2)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-14T03:45:52Z
- **Updated:** 2026-10-09T14:17:09Z
- **Closed:** 2026-08-14T04:05:26Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/693
- **Merged:** 2026-08-14T04:05:26Z
- **Merge commit:** 8809b6cc87298cf9c6438282d6415ef13095c517
- **Base / Head:** main / UOR-Foundation:i670-cover-source-parity
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #670 (and its umbrella #657 — the last GPT-2 compile-enablement item).

Extends the presence-gated GPT-2 canary through the **cover** stage. The pinned `openai-community/gpt2` teacher, once compiled, drives cover induction to the R4G1 route artifact + report, and the test asserts the report carries the **#606 source-parity rows**:
- **#597** source-manifest κ (from `models/gpt2-124m.json`, `blake3:3bca1b7f…`),
- **#600** geometry projection (bucket-average 768 → compiled width),
- **#602** the learned-absolute attention operator the GPT-2 oracle declares,

with **held-out routing recall measured (Gate C)** and the induced cover **deterministic across runs**.

## Verified end-to-end (cloud, real snapshot present)
Downloaded the pinned 548 MB snapshot and ran it: GPT-2 compiles to a bounded teacher corpus + table-native bundle → cover induces a valid `cover.r4g1` + `cover_report.json` → the report carries all three source rows → held-out recall is non-empty (reference top-1 **0.78** on the bounded run) → a second run induces the **identical regions + recall**.

**Determinism note:** the serialized R4G1 embeds a per-run HashMap-seeded provenance digest, so the *induced structure* (regions + recall), not the raw artifact bytes, is the reproducibility invariant — the test asserts structural determinism accordingly. (Surfaced as an observation; the induction itself is deterministic.)

## Presence-gated (#599)
Vacuous `UNAVAILABLE` pass when the snapshot is absent — CI and snapshot-less checkouts stay green (verified: 2 canary tests pass on a snapshot-less machine). Additive: no SmolLM2 path changes.

Gates: fmt, clippy `-D warnings`, claim-wording — green; full canary asserted with the snapshot present in the cloud.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

