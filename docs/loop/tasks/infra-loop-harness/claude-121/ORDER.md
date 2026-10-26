# ORDER: infra-loop-harness, claude-121 (narrow `d11-exporter-float-trunk`'s scope)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (item-file maintenance). Base: `82295caf6`.

## Goal

claude-120 (#3253, merged as `82295caf6`) released `d11-exporter-float-trunk` (#2820) from `held`. The next claim was then refused:

> refused: scope docs/mechanisms/lut-serving.md overlaps docs/mechanisms/lut-serving.md of item infra-loop-harness (PR #3169 (docs/loop/tasks/infra-loop-harness/codex-1/ORDER.md))

`docs/mechanisms/lut-serving.md` is the only path the two scopes share. #3169 is Codex's draft, which holds the umbrella. It has not had a commit since 2026-10-24T23:53Z, and its files are `PLAN.md`, `docs/integration/DECISIONS.md` and its own task folder; it does not touch `lut-serving.md`.

If a fresh D68 decision review approves, this unit:
- drops `docs/mechanisms/lut-serving.md` from the item's scope;
- says in the item where the exporter's results are recorded meanwhile: its lab record and `docs/research/notebook/m4-serving.md`;
- says that the mechanism page is updated once the umbrella is free.

The owner's direction for this work (26 Oct, in Claude's chat) is quoted in claude-120's ORDER.

**Why the infra-loop-harness item.** As for claude-113, claude-114 and claude-120, this ORDER is hand-written under the run file's umbrella exception. #3169 holds the umbrella, and nothing this unit edits overlaps #3169's files.

## Cancel probe (run first)

Class: instrument. Run at `82295caf6`; all must hold, or the unit stops:
- **The refusal names only `docs/mechanisms/lut-serving.md`.** No other path of the item's scope is in the umbrella's scope.
- **#3169 does not touch `lut-serving.md`,** and no other open PR touches the item or that page.
- **The item reads `status: ready` on `origin/main`.**

## Files in scope

- `docs/roadmap/items/d11-exporter-float-trunk.md`
- `docs/roadmap/README.md` (the generated render only)
- This task folder

## Done when

- **The D68 review is recorded** on #2820 as `[claude] owner log:`.
- **If approved, the item is changed:** its scope no longer lists `docs/mechanisms/lut-serving.md`, and its text says where the results are recorded until the mechanism page can be updated.
- **It is merged through the loop.**

## Stop and ask if

Stop and decide, never stop and wait.

- **The review finds the mechanism page must change in the same PR as the exporter.** Then keep the scope, record the finding on #2820, and note the idle umbrella claim on #2028.

## Cancel probe result

Run at `82295caf6`. All three hold:
- **Only `docs/mechanisms/lut-serving.md` is shared.** Of the item's nine scope globs, it is the only one that also appears in the umbrella's 38-entry scope.
- **#3169 does not touch it.** Its files are `PLAN.md`, `docs/integration/DECISIONS.md` and `docs/loop/tasks/infra-loop-harness/codex-1/{ORDER,PLAN,REPORT}.md`. No open PR touches the item or `lut-serving.md`.
- **The item reads `status: ready`** on `origin/main`.
