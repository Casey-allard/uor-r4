# ORDER: infra-loop-harness, kimi-36 (complete the fresh256 panel item record)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 8 (notes/roadmap migration). Base: `f1f55c0c1`.

## Goal

Record that the m2-fresh-heldout-panel construction done-when is met: the 256-row panel is registered, sealed, construction-FIT, uploaded and read back identically. Mark only that item done and update its remaining-work text. M2 scoring/training stays HELD/NOT_RUN; no milestone completion or capability claim.

## Cancel probe (run first)

At `f1f55c0c1`, the panel is `frozen: true` with all four hashes and 4/4 readback at HF commit `fb8c5e4fcd3568cc59be8be3073151295b98069c`, but the item is still `ready` and says upload is outstanding. The PM completion record stands.

## Files in scope

- `docs/roadmap/items/m2-fresh-heldout-panel.md`
- `docs/loop/tasks/infra-loop-harness/kimi-36/**`

## Done when

- Item status is `done`; What-is-missing no longer says upload is outstanding and names only the held M2 evaluation/scoring work.
- Panel/grader/bar/FIT/holds and the item's done-when are unchanged; the PR says `Closes #2849` and `References #2904`.
- Report/local PR checks, full gates and diff-check pass; a fresh-context doc-only review covers the exact head. Post-merge roadmap sync closes the item body only, not the M2 milestone.

## Stop and ask if

Scoring/training is requested, a bar/FIT/hold/milestone status must change, or a generated roadmap file outside scope is required.
