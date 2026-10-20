# #1430 — Add explicit width-576 native integer development profile

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-27T11:30:46Z
- **Updated:** 2026-10-09T13:17:41Z
- **Closed:** 2026-09-27T15:29:18Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1430
- **Merged:** 2026-09-27T15:29:18Z
- **Merge commit:** 530f5d8d00c0c71ed7d2fae74dfbe61f000a79ef
- **Base / Head:** main / UOR-Foundation:codex/native-dialogue576-20260927
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The retained native R1d dialogue model has 576 state/value coordinates, while integer serving accepted only 128/256 and would truncate or overrun ordinary value storage if that guard were removed. This adds an explicit `Dialogue576` development profile with full-width storage, 576/1152 packed row kernels and rational-width integer RMS normalization. Default retained profiles and the separate fixed-width conversational API remain scoped as before.

The offline Rust converter validates the historical R1d parameter lineage, calibrates and exports the existing quantizer without new model updates, reloads the child, and binds its tokenizer/protocol/source provenance in a sealed development bundle. Failed packaging attempts are preserved. This does not import the new dialogue-study descendants or change their ongoing learning conditions.

Validation: ten distinct focused arithmetic, storage, metadata, serialization and failure-path checks pass; release integer/converter builds and direct formatting checks pass. Scoped emitted-instruction inspection finds no forbidden opcodes in 20 ordinary-executable and 35 library ranges. Three historical conversational symbols remain missing from the ordinary executable, so this is not a whole-process certification. All three build attempts, resource interruptions and 778.381 seconds of build/check time are retained in the evidence packet.

**Draft:** actual historical-parent conversion, numerical retention and complete generated dialogue through this profile remain NOT_RUN. Run the separately projected observation once the fixed dialogue pair frees the owned model slot. Dense parameter access remains; no useful-chat, D5, speed or energy claim follows from this preparation.

Stacked on draft #1428 (`codex/packed-integer-coefficients-20260927`). References #973 and #820. The canonical roadmap/current state also record the completed full-prefix endpoint and exact role-only checkpoint recovery; no learning sweep is added.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

