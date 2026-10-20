# #1686 — Extract source-free integer geometric realizer runtime

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-04T13:13:10Z
- **Updated:** 2026-10-09T13:17:43Z
- **Closed:** 2026-10-04T13:57:05Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1686
- **Merged:** 2026-10-04T13:57:05Z
- **Merge commit:** e8226b7641d6288431d2287d4ce6ecdce4a4ff0a
- **Base / Head:** main / UOR-Foundation:codex/native-source-realizer-runtime-20261004
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The existing packed geometric realizer could execute integer reads but its loader still needed offline training tensors and floating exponential regeneration. This moves shared actions/view/realizer execution into uor-r4-integer and adds a checked native-only artifact loader with a separately trusted metadata/identity receipt. Training retains its stronger source-equivalence audits.

The standalone probe loads only the eight native files. On a restricted Linux filesystem with no training source weights, Stack checkpoint or expected answers, it exactly preserves four saved read traces and28 own-prefix replies across178 action steps, including token choices and EOS. Independent returned JSON/byte and chroot inventories pass. No default session integration or new language capability is claimed.

Validation on exact sourceff3d7cc6:13integer tests,2training-realizer tests,54driver compatibility tests,2output-isolation tests; standalone release build; exact native replay; wrong trust digest and unrelated sealed-output ancestry reject. Rustfmt on11 changed Rust paths, diff and claim-wording checks pass. Full130file return preserves both failed setup/guard attempts and the accepted repaired attempt. LinuxCPU48/noGPU; complete120min charge retained.

The loader/probe allocate and retain expanded tables/context replay. Whole binary opcode, allocation-free execution, M1 performance/energy, general chat and shared GroundedSession remain unqualified. The next attention task is the identity-initialized finite geometric read-feedback bridge before emission, with a learned role/surface-input control.

References #1552 and #1512.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

