# ORDER: infra-loop-harness, kimi-8 (PM bookkeeping: mark loop PR 3's item done)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904).
Setup row: 2.
Base: `3e3967c76`.

## Goal

PM record repair in the umbrella's lane: set
[infra-loop-pr03-render](../../../../roadmap/items/infra-loop-pr03-render.md)
`status: ready` → `status: done`. Loop PR 3 merged as #2923 (claude-14,
2026-10-21) and its done-when is met as the build plan narrows it (the loop page:
AGENTS markers are PR 8, `baseline.json` is PR 5). The stale `ready` made
`loop next` offer the item as claimable; kimi claimed it on 2026-10-22 and closed
draft #2957 unmerged on discovery (postmortem on #2028 and #2906).

## Cancel probe (run first)

At base `3e3967c76`, verify the build exists and works so the flip records fact,
not hope:

- `crates/uor-r4-harness/src/looprender.rs` and `tests/loop_render.rs` exist at main (#2923).
- `uor-harness loop render --out <tmp>` writes STATE.md, HANDOFF.md and
  docs/harness/LESSONS.md from commit `3e3967c76`.
- `~/.cargo/bin/cargo test -p uor-r4-harness --test loop_render` passes.

Result at base: render wrote all three views from `3e3967c76`; 31 of 31
`loop_render` tests pass (9.28s), including the planted fault (a hand-edited
STATE fails `--check`). The task stands.

## Files in scope

- `docs/roadmap/items/infra-loop-pr03-render.md` — the only file this task changes.

## Done when

- The item's `status:` reads `done`; no other line of any file changes.
- `uor-harness loop gates --quick` and `--full` pass at the PR head.
- `uor-harness loop report --check` and `uor-harness loop pr-check --local` pass at the PR head.
- `cargo run -q --locked -p xtask -- roadmap check` passes; `roadmap render` shows the item under done.
- After merge, `roadmap sync --apply` closes #2906.

## Stop and ask if

- Any file other than the one item file must change.
- Evidence appears that loop PR 3's done-when is not met (e.g. a missing piece not deferred by the loop page).
- The flip would need an accompanying change to the loop page or the umbrella item.
