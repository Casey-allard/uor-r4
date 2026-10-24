# ORDER: m2-fresh-heldout-panel, kimi-35 (repair the fresh256 storage pin)

Lab: kimi. Item: [m2-fresh-heldout-panel](../../../../roadmap/items/m2-fresh-heldout-panel.md) (#2849). Base: `9aaaf13bb`. Repair for PR #3147.

Finished: ORDER `dbb94fd0d`; uploaded panel pin and initial REPORT `b71d09f53`. Independent review returned CHANGES: abbreviated storage hashes, stale native-geometric status, and missing executed check results.

## Goal

Complete the fresh256 storage record without changing the grader, bar, FIT result, item status or holds. Correct remaining in-scope staged/PENDING text and retain exact check evidence. The roadmap item is explicitly a separate scoped PM follow-up, not silently edited here.

## Cancel probe (run first)

At `b71d09f53`, `docs/reference/storage.md` has only 8-hex prefixes; `docs/mechanisms/native-geometric.md` still calls the panel staged/PENDING; the REPORT lists commands without results. The findings reproduce and the repair stands.

## Files in scope

- `docs/harness/panels/m2-fresh256.json`
- `docs/reference/storage.md`
- `docs/mechanisms/native-geometric.md`
- `docs/labs/m2-fresh-heldout-panel/README.md`
- `docs/loop/tasks/m2-fresh-heldout-panel/kimi-35/**`

## Done when

- All four full SHA-256 values appear in the storage row; HF commit and 4/4 byte-identical readback remain pinned.
- Native-geometric and the lab record no longer present staged/PENDING upload status as current; dated history is marked where retained.
- REPORT records the roadmap-item follow-up, executed readback and exact-head report/pr-check/full-gate/diff results.
- M2 scoring/training stays HELD/NOT_RUN; grader, bar, FIT result, item status and holds are unchanged.
- Checks pass and a fresh-context review covers the repaired exact head.

## Stop and ask if

Any hash/readback differs, scoring/training is requested, a bar/FIT/hold/status must change, the roadmap item must be edited under this ORDER, or another file is needed.
