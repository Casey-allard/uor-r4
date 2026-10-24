# REPORT: infra-loop-harness, kimi-36 (complete the fresh256 panel item record)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 8.

Closes #2849
References #2904

## What did not work, first

- After #3147, the roadmap item still said the maintainer upload was outstanding and kept `status: ready` even though its construction done-when was met.
- M2 scoring/training remains HELD/NOT_RUN. Marking the panel item done does not complete the M2 milestone or establish capability.

## Cancel probe

The panel is `frozen: true`, construction-FIT and read back 4/4 byte-identically at HF commit `fb8c5e4fcd3568cc59be8be3073151295b98069c`. The item's stale ready/upload state required a PM record repair.

## What changed

- Item status is now `done` because the registered panel is sealed, construction-FIT, uploaded and read back.
- What-is-missing now names only the remaining pre-registered evaluation sheet, owner approval and scoring, not the completed upload.
- Grader, bar, FIT, holds and the item's done-when are unchanged. The PR closes only #2849, not the M2 milestone #2030.

## Measurements

Construction evidence: 256 rows / 128 histories / 16 blocks; all four question-blind/role-swap controls 0/128 pairs; 4/4 artifacts byte-identical on readback. No candidate scoring or GPU work ran.

## Checks run at the head

`loop report --check`, `loop pr-check --local`, `loop gates --full`, and `git diff --check`.

## Skills used

`uor-research-architect` for done-when evidence and exact completion scope; `superpowers:verification-before-completion` for final checks.

## Still open

The pre-registered M2 evaluation sheet, owner approval and actual scoring/training remain held. Post-merge roadmap sync is a separate PM action and must not close the M2 milestone.

## Plain summary

The fresh panel now exists and is preserved, so its construction item is complete. The model still has to pass evaluation under a separate release.
