# ORDER: m1-d11-exporter-discrete, kimi-1 (PM render refresh after the flip)

Lab: kimi. Item: [m1-d11-exporter-discrete](../../../../roadmap/items/m1-d11-exporter-discrete.md) (#2838).
Base: `d7d9d8d61`.

## Goal

PM render refresh (loop page step 14: the PM owns the render refresh). #2964
merged at 12:57:59Z flipping this item to `done` **without re-rendering** the
roadmap page, so `docs/roadmap/README.md` on main is stale: it shows the item as
`ready` and the counts as 43 ready / 15 done while the item files' truth is
41 ready / 17 done (flips by #2963, #2964, #2959). This PR re-renders the page
from the committed item files so the status view shows the truth again (L-001
class drift, caught within the hour).

## Cancel probe (run first)

At base `d7d9d8d61`: `cargo xtask roadmap check` passes (item files valid) while
`docs/roadmap/README.md` line 23 shows `ready | 43` and the item appears in the
ready table at line 65 — confirmed stale against
`docs/roadmap/items/m1-d11-exporter-discrete.md` (`status: done`). The task stands.

## Files in scope

- `docs/roadmap/README.md` — one of the nine shared files, always allowed; the
  only file this task changes, produced by `cargo xtask roadmap render`.

## Done when

- The README is byte-identical to a fresh `cargo run -q --locked -p xtask -- roadmap render` at the PR head.
- The counts read 41 ready / 17 done and the item appears in the done table.
- `uor-harness loop gates --quick` and `--full` pass at the PR head.
- `uor-harness loop report --check` and `uor-harness loop pr-check --local` pass at the PR head.

## Stop and ask if

- Any file other than the rendered README must change.
- The render produces an unexpected diff (any row other than this item's move and the two count lines).
