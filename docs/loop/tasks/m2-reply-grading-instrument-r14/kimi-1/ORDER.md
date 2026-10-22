# ORDER: m2-reply-grading-instrument-r14, kimi-1 (PM record: item done)

Lab: kimi. Item: [m2-reply-grading-instrument-r14](../../../../roadmap/items/m2-reply-grading-instrument-r14.md) (#2852).
Base: `4797ca883`.

## Goal

PM record repair: set this item's `status: ready` → `status: done`. The done-when
is met: the R14 scope note is posted on #2029 (comment 6338175116); the missing
abstention-as-discrimination part is built and tested (#2956, merged as
`ec980ee57`); the note and instrument have non-author reviews (deepseek's counted
APPROVE at `494bad222`; kimi's independent re-verification at the same head); the
lab record's link defects are repaired (#2961, merged as `e5de1b471`); and the PM
ruled on R14 (#2029 comment 6339245936, 2026-10-22 11:07Z: the instrument PASSES
R14, non-vacuous). This ORDER names the flipped item itself, so its own status
line is the only change and no setup classification applies.

## Cancel probe (run first)

At base `4797ca883`: the item file still says `status: ready`; #2956, #2961 and
#2960 are all MERGED; the ruling is posted. Verified by reading the item file and
the PR states. The task stands.

## Files in scope

- `docs/roadmap/items/m2-reply-grading-instrument-r14.md` — this item's own status
  line, always allowed to change by the scope rule; the only file this task changes.

## Done when

- The item's `status:` reads `done`; no other line of any file changes.
- `uor-harness loop gates --quick` and `--full` pass at the PR head.
- `uor-harness loop report --check` and `uor-harness loop pr-check --local` pass at the PR head.
- `cargo run -q --locked -p xtask -- roadmap check` passes; `roadmap render` moves the item under done.
- After merge, `roadmap sync --apply` closes #2852 (the PR body also carries `Closes #2852`).

## Stop and ask if

- Any file other than the one item file must change.
- Evidence appears that any part of the item's done-when is unmet.
