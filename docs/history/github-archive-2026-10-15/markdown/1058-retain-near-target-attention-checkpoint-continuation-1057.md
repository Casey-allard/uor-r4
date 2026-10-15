# #1058 — Retain near-target attention checkpoint continuation (#1057)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-02T17:16:23Z
- **Updated:** 2026-10-09T14:16:50Z
- **Closed:** 2026-09-02T17:17:35Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1058
- **Merged:** 2026-09-02T17:17:35Z
- **Merge commit:** d424dbee0272eeaa61f23262ff863495be956593
- **Base / Head:** main / UOR-Foundation:issue-1057-checkpoint-continuation
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Retained outcome

The exact saved #1055 checkpoint improved from 68.2861% to **98.5229%** development recall (8,071/8,192; +30.2368 percentage points), with NLL 0.0914115440. Best measured recall was 98.6572% at block 36; the preserved final artifact is block 40.

This is substantial, retained learning progress—not false/failed attention, a discarded model, or a project stop. The raw frozen `CONTINUATION_MISS` says only that strict >99% was narrowly unmet. The conditional binding-permuted control remained `NOT_RUN_PRIMARY_MISS`; no uncollected attribution evidence is claimed. #1050's separate 99.1667% reference and HELM-D-R4/#1014 positives remain intact.

## Implementation and execution

- Restore model, AdamW, cosine scheduler, sampler, and RNG exactly; preserve the first 20 history entries.
- One bounded continuation: 3,920 additional updates, 7,840 total. Same 8,192 unique training rows; no new-data or parameter sweep.
- Eight CPU threads, one inter-op thread/process, Apple Accelerate; no CUDA/MPS. Additional execution: 1,096.986830 seconds; peak RSS 1,192,968,192 bytes.
- Persist separate inherited/additional work and restart time; bind parent checkpoint, source closure, and artifacts. No historical source changes.
- Append the full continuation curve and provenance; refresh current summaries and explicitly preserve the user's near-target interpretation.

## Focused evidence

Seven new continuation checks passed in 2.167 seconds. Formatting/scoped lint, claim wording, and diff integrity passed. One fresh-process final-artifact replay reproduced logits, metrics, and result identity (+8,192 development decisions; zero optimizer/control work). Independent read-only audit confirmed every exported tensor, tied head, RNG, parent history, AdamW steps, scheduler, sampler, time, and work accounting.

Existing tests/C0/source reproduction and CPU timing matrices were not rerun. Broad QA remains dormant. Required queue acknowledgements are transport only, not verification.

Result: `blake3:35b1cedfd51385bf98277a4527b1ce05f5dd3b93fffe125a5ea28c2a34b6387c`.
Artifact: `blake3:69af5586eccfceab4214e9f13524eeea578eb3facaea4fdedec89f0b5d217445`.
Checkpoint: `blake3:fd24e6b84af9891c1dad1eb2a13c6d86b8e8833ac02ab40f0e4e95d4530d140a`.

Recommended next action: separately scoped inference-only coherent-R4 integration using the already-qualified #1050 reference, while retaining #1057 as a valuable exact-data artifact for subsequent inference work. No automatic third window, new generation, softmax-removal, or reasoning claim.

Closes #1057. References #973.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

