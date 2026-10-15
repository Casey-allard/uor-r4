# #1134 — Compose fact answers with retained-word copying and selective execution

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-05T18:29:02Z
- **Updated:** 2026-10-09T13:17:42Z
- **Closed:** 2026-09-05T18:50:36Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1134
- **Merged:** 2026-09-05T18:50:36Z
- **Merge commit:** 19f25ddb83a9dec70e11f8cb617449b318eff561
- **Base / Head:** main / UOR-Foundation:codex/native-fact-copy
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The existing copy path could only begin at the first response token and required a numeric source. The optional, artifact-bound `composed_entry` extension learns lexical output, retained-word selection, causal copying and completion without that numeric dependency. Exact query/source word equality supplements the existing relative H4/zeta paths. Unsupported answers learn lexical continuation after an actually selected first token. Numeric precedence, immutable copy provenance, the sixteen-word capture and the 32-observation bound remain.

Committed interior bytes now execute their copy operator before ordinary candidate scoring. Their score is explicitly a dispatch marker of 1. Required observation and memory updates continue; old artifacts retain their previous behavior. The change remains Rust integer/table serving with floating-point fitting offline.

Validation and measured behavior:

- 16/16 fixed fact cases (simple, distractor, update, unsupported), versus 0/16 for the preceding artifact. All 32 original answers, 12 identifier transfers and eight binding outputs remain correct. All 32 generated Rust source hashes match previously compiled/executed sources.
- 62/62 outputs and final states match with dispatch disabled; the focused test also compares per-step checkpoint state. Across 114 committed interior bytes, ordinary score lookups fall 1,764,910 → 1,509,562 and memory lookups 417,983 → 287,313. Other copy work is unchanged. Complete generation measured 61.984 → 46.864 ms in one pass; this is descriptive timing and a generic conditional-execution gain.
- 108 native tests, six allocation/source checks, five preparation tests, formatting, architecture policy and claim wording pass. Three direct optimized CLI runs match evaluator Generation fields. All 248 Generation objects replay exactly between the development fitter and optimized serving. Only the final offline label-slice correction required the development rebuild; serving code is unchanged.

The sixteen cases were frozen before fitting but exposed during an implementation-invalid attempt, so the repaired result is OPEN development evidence. The stopped attempt and the model that incorrectly marked 48 construction targets unreachable are preserved. No data, features or hyperparameters were retuned for the label-slice repair. Two older city prompts still fail. Copy-geometry suppression reaches 3/16 fact answers; a matched-refit geometric advantage and general conversation remain unestablished.

Model work: 78.172/120 seconds this cycle, 1,025.234/1,800 cumulative. Engineering: 1,099.542/1,200 seconds. Necessary storage increases were preauthorized and recorded (+896 MiB, including the stopped task's worktree and separate fitter cache); the 128 MiB stop margin remains. No user material was deleted.

The current-state pointer and existing evidence JSON identify artifact `blake3:d095a1abccefd68678a9d7da61e4d8fd094d9f7ca71e10c5e0383518889ed586`, complete costs, failures and preservation. Broader release checks were not run; protected CI reports its actual checks separately.

Refs #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

