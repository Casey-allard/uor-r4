# ORDER: m2-fresh-heldout-panel, kimi-34 (pin the uploaded fresh256 panel)

Lab: kimi. Item: [m2-fresh-heldout-panel](../../../../roadmap/items/m2-fresh-heldout-panel.md) (#2849). Base: `9aaaf13bb`.

## Goal

Close only the M2 fresh256 storage prerequisite: record the four sealed artifacts' uploaded HF location and mark the panel frozen, with byte-identical readback evidence. Do not score a candidate, change the grader, bar, FIT result, item status or any hold.

## Cancel probe (run first)

At `9aaaf13bb`, the panel is `frozen: false` solely pending maintainer upload. All four expected local artifacts match their registered byte counts and SHA-256 values. They were uploaded at HF commit `fb8c5e4fcd3568cc59be8be3073151295b98069c` and downloaded back to a separate directory with every `cmp` passing. The storage pin now stands.

## Files in scope

- `docs/harness/panels/m2-fresh256.json`
- `docs/reference/storage.md`
- `docs/labs/m2-fresh-heldout-panel/README.md`
- `docs/loop/tasks/m2-fresh-heldout-panel/kimi-34/**`

## Done when

- The panel entry says `frozen: true` and records the HF path/commit plus readback; no corpus bytes enter git.
- Storage and the lab README name the uploaded location rather than staging/pending upload, while retaining all four full hashes.
- M2 scoring/training stays HELD/NOT_RUN; the existing non-author FIT, bar, grader, item status and holds are unchanged.
- Report/local PR checks, full gates and diff-check pass; another vendor or fresh-context doc-only review covers the exact head.

## Stop and ask if

Any hash/readback differs, scoring or training is requested, a bar/FIT/hold/status must change, or a file outside scope is needed.
