# REPORT: infra-loop-harness, kimi-8

Lab: kimi. Order: [ORDER.md](ORDER.md). References #2904. Closes #2906.

## What did not work, first

- `loop next` offered `infra-loop-pr03-render` as "ready, scoped and claimable" and
  kimi claimed it on 2026-10-22 (~06:14Z), because the item file still said
  `status: ready` although loop PR 3 merged on 2026-10-21 as #2923 (claude-14).
  The false claim was unwound the same hour: draft #2957 closed unmerged, branch
  and worktree deleted, do-not-claim notes posted on #2028 and #2906. No code was
  written on the false claim. Root cause: the build-plan row's PR did not flip the
  item's status at merge, and the router reads the item file. This PM PR repairs
  the record; the wider question (status flips forgotten at merge) is noted for
  the umbrella's wrap-up.
- `uor-harness team claim --branch kimi/pr03-status` dirtied the tracked
  `docs/harness/claims.json` in the task worktree (the old-way registry write).
  It was reverted to keep this PR inside the ORDER's one-file scope; how claims
  registry edits reach main is an open PM question (claims.json is in no item
  scope and pr-check's scope rule has no claims exception), tracked on #2028.
- Six stale kimi branch claims from 2026-10-17/18 (`kimi-pm-main`,
  `kimi/dose1-dispatch-record-2026-10-18`, `kimi/optimization-path-2026-10-18`,
  `kimi/pr-b-hygiene`, `kimi/pr-d-plan`, `kimi/pr-d37-labels`) had no branches;
  all six were released.

## What changed

One word in one file: `docs/roadmap/items/infra-loop-pr03-render.md`
`status: ready` → `status: done`. Docs class; size 1 line, ratio 1.0 against the
ORDER's estimate.

## Measurements

- Cancel probe at base `3e3967c76`: `uor-harness loop render --out <tmp>` wrote
  STATE.md (4,816 B), HANDOFF.md (17,911 B) and docs/harness/LESSONS.md from
  commit `3e3967c76af8ae4262c3613edf6ef4928a700bde` — the merged renderer works.
- `~/.cargo/bin/cargo test -p uor-r4-harness --test loop_render` at `3e3967c76`:
  `31 passed; 0 failed` (9.28s), including the planted fault (a hand-edited STATE
  fails `--check`) and the determinism tests (two renders of one commit are
  byte-identical).
- `gh pr view 2923`: MERGED 2026-10-21T14:53:39Z, "[claude] loop PR 3: uor-harness
  loop render — views from committed inputs, --check at the stamp (#2906)".

## Checks run at the head

- `uor-harness loop gates --quick` and `uor-harness loop gates --full` — results
  recorded before push (expected GREEN; one docs line changed).
- `uor-harness loop report --check`, `uor-harness loop pr-check --local` — recorded
  before push.
- `cargo run -q --locked -p xtask -- roadmap check` — recorded before push.
- CI: the `harness rules and tests` job is red repo-wide until #2949 lands
  (pre-existing clippy error from #2946, confirmed in run 39759847159's log);
  this PR changes no code.

## Skills used

None.

## Still open

- `infra-loop-pr08-migration` (gated on the owner in the kimi chat) and
  `infra-loop-pr10-supervisor` (done-when completes when #2949 lands) remain
  `ready` — both correctly.
- The claims-registry sync path (how `team claim`/`release` writes reach the
  committed `docs/harness/claims.json`) is unanswered; noted above.
- `roadmap sync --apply` after merge closes #2906 (PM runs it).

## Plain summary

Loop PR 3 (the `loop render` command) was finished and reviewed on 21 October,
but its roadmap item still said "ready", so the work router offered it as fresh
work and kimi briefly claimed it before noticing. This PR changes the item's one
status word to "done" so the router stops offering it; the render command itself
was re-run and its full test suite passes, proving the item really is complete.
