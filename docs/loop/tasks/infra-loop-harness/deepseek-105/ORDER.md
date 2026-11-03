# ORDER: infra-loop-harness, deepseek-105 (repair image-item registration)

Lab: deepseek. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).
Setup row: 2 (item-file maintenance).
Base: `9e0403b132da34752499c657def95dfce3235538`.

## Goal

Repair the fresh-context CHANGES review of PR #3622 at
`0f92abc70bc96f4a92407f4abe4d1a477cdfa613`.

The item's requirements were correct, but its implementation scope omitted
`crates/uor-r4-integer/Cargo.toml`, one internal relative link was wrong, and the
report did not distinguish gate execution at the implementation head from the
later receipt commit. The PR body also needs exact final sizing.

## Cancel probe (run first)

At head `0f92abc70`, the item omits the integer manifest from scope, the source
link has the wrong depth, and `pr-check` reports the stale size. The repair
stands.

## Files in scope

- `docs/roadmap/items/geometric-image-input.md`
- `docs/loop/tasks/infra-loop-harness/deepseek-104/**`
- `docs/loop/tasks/infra-loop-harness/deepseek-105/**`

## Done when

1. The item scope includes `crates/uor-r4-integer/Cargo.toml`.
2. Every internal source link resolves.
3. The superseded report identifies the implementation head and the later
   receipt-only commit accurately.
4. The active report records exact PR sizing and the final checks.
5. Roadmap, link, report, exact-diff, local PR and full gates pass.
6. A fresh exact-head review approves the repaired registration.

## Stop and ask

Do not implement image decomposition or alter its acceptance criteria.
