# #817 — serving: r4 is the canonical served model identity (#655-F flip)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-19T02:21:14Z
- **Updated:** 2026-10-09T13:08:56Z
- **Closed:** 2026-08-19T02:41:59Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/817
- **Merged:** 2026-08-19T02:41:59Z
- **Merge commit:** 6e1c677e140ecf333872d871795cdf562dacedf5
- **Base / Head:** main / UOR-Foundation:issue-655-f-flip
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Executes the maintainer-authorized **#655-F flip** (fresh explicit approval 2026-08-19, #655 issuecomment-5336515423, sequenced after the v0.1 release publish and #811 — both held): **`r4` is the canonical served model identity** on every serving surface, with `uor-r4` retained as the deprecated request-side alias for a compatibility window. The change list is exactly the PR #810 inventory; the execution record is the new addendum in `docs/serving_default_flip_655_f.md`. Stacked on #816 (the #811 abstention gate); the last commit is this PR's.

- `CANONICAL_MODEL_ID = "r4"`; `resolve_request_model_name` accepts omitted / `r4` / `uor-r4`, responses always report the canonical id.
- `active_canonical_model_name` reports `r4`; the physical installed name narrows to metadata (`/uor/v1/status` gains `canonical_model` alongside the unchanged physical `model_name` + `physical_root`).
- OpenAI wire ids `r4`-prefixed (`chatcmpl-r4-…`, `resp-r4-…`, `msg-r4-…`, `system_fingerprint: r4-{mode}`).
- CLI `client --model` defaults to `r4`; the remote client parses both fingerprint prefixes during the deprecation window.
- Docs: README + CONFIGURATION carry the identity contract; the F0 doc records execution, including the **recorded absence of a THREAT_MODEL.md** (creating one stays a separate product decision).
- Deliberate non-sites, unchanged per the inventory: crate names, schema strings, staging dirs, the `uor-r4-cli` orchestrator script file name, the four source-verification logical-name fallbacks, and every engine/tier name (`r4g1` is a tier, not the model).

## Preconditions at flip time (per F's own activation contract)

1. **Distributed bundle**: v0.1 published with the attested bundle; `r4 install-release --tag v0.1` proven end to end (#741 closed).
2. **Baseline-quality canary**: the default decode is the measured 15/15-valid path (#814, byte-proven); probes run through the same deployed D4 policy on CLI and server (#816). #784 distinctness stays open and disclosed.

## Evidence

Post-flip in-domain ask output byte-identical to the archived decode canary (0 diffs) — identity is a serving-surface rename, not a decode change. Deprecation-window resolve tests (omitted/`r4`/`uor-r4` all resolve; unknown declines; inactive errors); canonical-id + physical-metadata status test on both profiles; #789-G2 discovery test updated to the canonical identity. Server 197 · chat 14 · openai_profile_pin 3 · bin 7 · workspace clippy `-D warnings` clean · fmt clean.

#655's close record follows on the issue once this merges.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_016X39CfJSLCU4KhjNP2XXTW


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

