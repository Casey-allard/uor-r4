# REPORT: infra-loop-harness, claude-23 (loop next skips items with unfinished dependencies)

Lab: claude. Base: `a2ada4542`. Order: [ORDER.md](ORDER.md), committed first (`baf075ae8`). The cancel probe found that `loop next` did not read `depends_on`: `ItemFields` had no such field, and the `any` source checked only status, scope and `loop claim`.

## What did not work, first

- **`loop next` offered work nobody could do.** On 22 Oct it offered `d11-exporter-float-trunk` to every lab. That item's `depends_on` includes `owner-decision-float-path-scope` (#2875), an open owner decision. Kimi and DeepSeek each had to recognise this and set it aside by hand, and DeepSeek idled because the first offer it received was blocked.
- **`loop claim` still does not check dependencies,** as the ORDER calls for: whether claiming should refuse such an item is a separate decision. A lab that runs `loop claim <id>` directly can still claim a waiting item.

## What changed

| Commit | What |
|---|---|
| `637fc04ed` | `ItemFields` gains `depends_on`, parsed from the same inline `[a, b]` form as `scope:` through a shared `inline_list` helper. `roadmap check` already requires that form, and all 87 items use it. In `loop next`'s `any` source, a ready, scoped item waits while any dependency is missing on `origin/main` or not `status: done`. A waiting item is never offered, and when nothing is claimable the `any` line names it and what it waits on. A new test, `an_item_waits_until_every_dependency_is_done`, is in `tests/loop_next.rs`. |
| this REPORT's commit | The harness README's `any` line describes the dependency rule, and says `loop claim` does not check dependencies. |

## Measurements

None. This is tooling, with no model run.

## Checks run at the head

- fmt is clean, and whole-crate clippy with `-D warnings` is clean.
- `loop_next` 25 (including the new test), `loop_claim` 33 and `loop_shared_files` 7 passed.
- Negative gate: with the `loop next` change set aside (the parser kept), `an_item_waits_until_every_dependency_is_done` fails. Restored, all 25 pass.
- Live, on the real repository with this branch's binary, `loop next` no longer offers `d11-exporter-float-trunk`. It goes on to `infra-k16-normal-close-accounting`, which has no dependencies; before the fix it stopped at `d11`.
- Full harness suite at `637fc04ed`, plus the README: 1,001 passed, 0 failed.
- Claim-wording passes. `xtask links` finds no broken link in the changed files. The copyright check finds 0 matching 8-word runs.

## Skills used

solution-integrity: the dependency check reuses `loop claim`'s item reader and the inline-list form `roadmap check` already enforces, so there is no second parser to drift.

## Still open

- `loop claim` does not refuse an item with unfinished dependencies. If Casey or Kimi wants it to, it is a small follow-up in `loopclaim::check`.
- When an item is claimable, the waiting items are not listed; they appear only when nothing is claimable.

## Plain summary

The loop's "what should I work on next" command no longer offers a task that is still waiting on another unfinished task, such as an owner decision. Labs stop being handed work they cannot start.
