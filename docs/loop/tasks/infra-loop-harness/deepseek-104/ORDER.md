# ORDER: infra-loop-harness, deepseek-104 (recover the image-decomposition item)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).
Setup row: 2 (item-file maintenance).
Base: `9e0403b132da34752499c657def95dfce3235538`.

## Goal

Recover the roadmap registration for the already-created issue #3619,
`geometric-image-input`. The issue records the requested deterministic integer
R4 image decomposition, bounded native API and experimental Studio Canvas path,
but `docs/roadmap/items/geometric-image-input.md` and its issue mapping are
absent from `main`.

This unit registers the item only. It does not implement image input or claim
visual understanding.

## Cancel probe (run first)

`gh issue view 3619` returns an open issue with marker
`roadmap:id=geometric-image-input`, while the corresponding item file and
`docs/roadmap/issues.json` mapping are absent from the base. The registration
stands.

## Files in scope

- `docs/roadmap/items/geometric-image-input.md`
- `docs/roadmap/issues.json`
- `docs/loop/tasks/infra-loop-harness/deepseek-104/**`

## Done when

1. The item file preserves issue #3619's why, missing work, acceptance rules,
   sources and truthful no-visual-understanding boundary.
2. Its scope covers a future vision crate, integer API validation, Studio
   assets, tests and documentation without a root-wide glob.
3. `docs/roadmap/issues.json` maps `geometric-image-input` to #3619.
4. Roadmap, report, exact-diff, local PR and full gates pass.
5. A fresh exact-head review approves the registration.

## Stop and ask

- Stop if the issue marker or number differs.
- Do not implement image decomposition, modify model/tokenizer behavior or
  claim visual understanding in this registration unit.
