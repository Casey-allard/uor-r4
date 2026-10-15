# #1590 — Compile learned geometric span actions into an exact integer register

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T00:38:58Z
- **Updated:** 2026-10-09T13:17:32Z
- **Closed:** 2026-10-02T00:40:32Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1590
- **Merged:** 2026-10-02T00:40:32Z
- **Merge commit:** 151ec10bb4efc4479b90202e08f204028da816a1
- **Base / Head:** main / UOR-Foundation:codex/geometric-span-native-20261001
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The learned ordered-span producer previously classified floating embeddings and accumulated its held key in the training path. This change compiles saved token actions into a source-bound signed-H4 dictionary and executes ordered accumulation through an exact integer register, then connects that component to the unchanged learned reader for retention evaluation.

Both existing models replay exactly with zero training: all 512 answer-logit/source rows match the unchanged producer and historical raw results, and 34,688 prefix positions have zero presence, code, reconstruction or full-prefix logit differences. Seed 1 retains 99/128 and 109/128 answers; seed 2 retains 128/128 on both panels. Eleven focused checks, formatting and claim wording pass. All 17 span compatibility checks pass after integration with current main.

This qualifies the dictionary/register component only. The controller, canonical F32 reconstruction, reader/trunk and outputs still use floats; external memory intrinsics are outside the three-body AArch64 opcode inspection. No full serving, natural-language, donor-reasoning or energy claim is made. Preserve offline donor recompilation into token actions, contextual operations and output behavior, with measured retention.

The report and receipt bind executed source/executable/model/panel identities, raw traces and exact limits. The optional prime-ranked/n-gram pointer changes on main are integrated; this retained experiment used no pointer. Next is a fixed saved-parameter bridge for signed-relative geometric potentials, with native event prediction and bounded access still explicit.

References #1512


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5943397053 — 2026-10-02T00:39:54Z — Casey-allard

Independent nonauthor agent review approved exact head `aec3acb7c1ecde5b2ab93669efa3d55585ef7da7`, tree `26b7c8f3f09c0abf6d05093e3438e1ca56b77af7`. Source, evaluator and raw measured replay were independently inspected; no reviewer-authored implementation or reviewer inference/build is claimed.

Executed source `251f0a79`, binary SHA `3c40bb21a08b3c79999dbd7562b644835544f94e8521239f45db3249d8657a7a`: zero training, all 512 historical answer/source rows retained bit for bit, zero held/code/reconstruction/full-prefix mismatch. 11 relevant focused checks pass. After integrating current main `f9c92692`, 17 span checks pass at `2fc1ec4f`; the final head changes only the receipt/docs. No second model replay is claimed.

The qualified component is the integer dictionary/register; controller, reconstruction and surrounding reader remain float. The three emitted AArch64 method bodies have no forbidden arithmetic, but external callees/whole binary are not qualified. The optional pointer path remains inactive for these models. Original failures, donor-transfer scope and native-serving obligations are preserved.

The D19 advisory delivery publisher returned `Not Found`; no passing evidence status was fabricated. The five CI compatibility acknowledgements are not tests. Actual local execution carries validation.



---
_Comment/review ids in this file: 5943397053_

