# REPORT: infra-loop-harness, claude-48 (the umbrella's scope gains the links tool)

Lab: claude. Base: `18c03cebc`. Order: [ORDER.md](ORDER.md), committed first. The cancel probe found `xtask/src/docs_links.rs` absent from the umbrella's `scope:`, so the task stands. No open PR changes that file.

## What did not work, first

- **The links tool sat outside every fitting scope.** The PR-8 blocker Kimi carded is in `xtask/src/docs_links.rs`. Only `m8-independent-verification` covers `xtask/**`, which is unrelated work. So the fix needs this one-line widening first, a second PR in sequence.
- **`main`'s roadmap README is still stale** (ready 35, expected 34). #3004, queued, and #3003 make the same re-render; it is included here too, and identical edits do not conflict.

## What changed

| Commit | What |
|---|---|
| ORDER commit | This ORDER, alone. |
| `bd14fa78c` | `docs/roadmap/items/infra-loop-harness.md`: `xtask/src/docs_links.rs` is appended to `scope:`. `docs/roadmap/README.md` is re-rendered. |
| this REPORT's commit | This REPORT. |

## Measurements

None.

## Checks run at the head

- `cargo xtask roadmap check`: 88 items OK.
- `roadmap render --check`: current, after the re-render.
- `loop report --check`: PASS. `loop pr-check --local`: PASS.

## Skills used

scope-guard: one scope entry, plus the shared README render.

## Still open

- **claude-49:** the links rule. A NEW broken link is reported, not failed, when its source is under `archive/`, was added since the base, is byte-identical to a file present at the base, and the link resolves from that original's location. It needs a planted-fault test. After it lands, PR 8 merges `main`.

## Plain summary

The tool that checks links needs one small change to unblock Kimi's PR 8, and the umbrella task was not allowed to touch that tool. Now it is.
