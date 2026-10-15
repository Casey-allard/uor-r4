# #1142 — Learn shared role-aware source selection and causal response entry

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-05T22:54:28Z
- **Updated:** 2026-10-09T13:17:43Z
- **Closed:** 2026-09-05T23:17:15Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1142
- **Merged:** 2026-09-05T23:17:15Z
- **Merge commit:** 223aed71e770b158cebb0f0dd9a3d6be4f191829
- **Base / Head:** main / UOR-Foundation:codex/native-role-read
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The native reader previously chose lexical entry and a copied source independently. Retained answer words could therefore produce the wrong entry or source under changed wording. This adds one learned candidate-relative occurrence/NoRead choice, commits its exact source when entry is observed, and reuses that source through copying and checkpoint restore.

Rust fitting exports bounded quantized tables over relative local role/context, exact equality, ordered prime context and existing signed H4/zeta features. The selected revision removes pooled query identities. The sixteen-word capture, numeric precedence, /4 memory, completion and committed-byte dispatch remain. It learns weights over role features; a separate semantic role codebook is not implemented or claimed. Old artifacts omit the optional head and retain their prior behavior.

## Measured result

Selected artifact: `blake3:61a24cfa4ce262fd974bc8e84f082a0489db3b58cfafb46c4c42a86e49c13184`.

| Evaluation | Unchanged parent | Selected reader |
|---|---:|---:|
| Preservation | 62/62 | 62/62 |
| Binding outputs | 8/8 | 8/8 |
| Earlier sixteen-case wording set, now OPEN | 13/16 | 16/16 |
| Earlier 32-case transfer set, now OPEN | 16/32 | 30/32 |
| Predeclared first-use set | 13/24 | 24/24 |

The first fit's 61/62 preservation negative is retained. The first-use set was prepared before fitting, and the selected artifact/source hashes were recorded before opening it. Names and values are new relative to supplied construction and prior development; grammar families are known. No parameter or selection-law change followed opening. Two older transfer cases still falsely abstain. This accepts a bounded binding/entry handoff, not broad language or alpha.

All four new generated Rust functions compile unchanged and pass seven identity inputs each (28 assertions). Thirty-two earlier generated sources remain byte-identical to their checked versions. Every Generation field from an actual native CLI run matches evaluation, with matching artifact identity.

## Validation and cost

- Release CLI and example build: PASS using pinned Rust, offline, one build job.
- Fourteen existing copy tests and two new real-fit role/commit tests: PASS. Coverage includes observed commitment, mismatch, pre-byte-zero restore, source provenance, quantized row bounds and parent/artifact identity. An initial assertion wrongly required prediction work counters to stay unchanged; its correction and failed run are retained.
- Formatting, native architecture policy, claim wording and diff checks: PASS. Local documentation links resolve. Broader release qualification: NOT_RUN.
- All 62 outputs, token sequences and final causal states match with committed-copy dispatch disabled. Ordinary score lookups fall from 1,772,626 to 1,470,063. The single-pass generation times are 65.037 and 46.517 ms; startup, artifact loading/validation and report I/O are separately included in full command wall accounting. No robust speedup claim.
- Copy-geometry suppression reaches 45/62; copy suppression 32/62. These are sensitivity controls, not matched-refit geometric superiority.
- Artifact JSON grows by 283,271 bytes. Model work is 57.201/120 seconds; engineering commands 742.865/1,200 seconds. Cumulative model ledger is 1,138.842/1,800 seconds, leaving 661.158. Peak sampled direct model RSS is 613,810,176 bytes; compiler/unmonitored fixture peaks are unavailable.
- The preauthorized necessary 512 MiB storage increase raises the inherited cap to 6,459,228,160 bytes with a 128 MiB stop margin. No material was deleted or paid external compute used.

`docs/native_geometric_role_read_1137.md` records implementation, information loss, artifacts, full work scope and remaining failures. README, roadmap, current-state, research and continuation pointers agree. Local receipts and the cumulative checkpoint are retained under `.uor-models/native-typed-value-2026-09-05/role-read-*`.

After protected delivery, #1138 exact learned relation memory across raw-window eviction is the next step. #1139 selective geometric access and #1140 typed operator composition remain subsequent, unqualified work.

Closes #1137


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

