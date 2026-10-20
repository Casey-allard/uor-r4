# #1443 — D11: native, multiplier-free, transformerless serving for every lab; D10 superseded (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-28T00:00:15Z
- **Updated:** 2026-10-09T13:08:57Z
- **Closed:** 2026-09-28T00:04:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1443
- **Merged:** 2026-09-28T00:04:14Z
- **Merge commit:** aa7a68f5e172c339505e5173d0e783c6564c0e51
- **Base / Head:** main / UOR-Foundation:lab/claude/d11-native-serving
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Lab:** Claude, Lab 1 and director. **Scope:** `docs/integration/DECISIONS.md` and one paragraph of `AGENTS.md`; no code.

This records the owner's decision of 2026-09-27, made in the director session and accepted as prompted ("Supersede D10").

- **D11** makes the ROADMAP §0 rules R1–R5 the only serving contract for every lab:
  - no float;
  - no multiplier instruction in served kernels;
  - dense per-token weight access only as a labelled interim;
  - no transformer backbone;
  - energy claimed only when measured.
- **D10 is superseded.**
  - The converted SmolLM2 backbone and the hardware-multiplier exception for runtime values are withdrawn.
  - Converted transformers are comparators or offline teachers only.
  - `uor-r4-lut`/`lut-chat` are frozen comparators.
  - D10's text is kept as history under a superseded marker, and D10-era results keep their scope.
- **`AGENTS.md`:** the D10 sentences in "Architecture and claim boundaries" are replaced with the D11 contract.

**Checks:** `python3 scripts/check_claim_wording.py` passes.

**Affects:** #1437, the cloud Claude track, which planned D10 serving. Its memory experiment remains valid as index evidence, but its serving plan must use R2 kernels (ROADMAP T3).

Merging needs the owner's approval.

References #820

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

