# ORDER: infra-loop-harness, claude-30 (end D59, at the owner's direction)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 6. Base: `4bb1feede`.

## Goal

D59 routes every loop setup PR's exact-head review to Claude's fresh-context reviewers alone, with no timeout. When Claude was blocked on 22 Oct (about 08:24Z to 12:35Z), every setup PR queued behind it, including two of Kimi's PM PRs and DeepSeek's #2949. The owner ended D59. From now on, setup PRs take any other-vendor fresh-context review, like every other PR (D46).

The planted fault: **with Claude unavailable, a setup PR cannot get a counted review from anyone.**

## Cancel probe (run first)

At `4bb1feede`, does `docs/harness/d59.json` exist reading `"d59": "ended"`, and is it listed in `docs/harness/owner-paths.json`? If both hold, D59 is already ended and the task is void. Class: instrument.

## Calls

- Add `docs/harness/d59.json` with `"d59": "ended"`, the owner's answer and the time. List it in `docs/harness/owner-paths.json`. `d59_in_force` (`loopreview.rs:319`) reads both at the merge base, so D59 ends for every PR whose base includes this merge. No code changes.
- Mark D59 ended in its `docs/integration/DECISIONS.md` row, the reversal cell. The rest of the row stays as history.
- `AGENTS.md:19` and the loop page's D59 lines (`docs/loop/README.md:45`, `:219`) say D59 has ended. The lines that describe D59's behaviour conditionally ("while D59 is in force") stay true and are left alone.

## Owner approval

`docs/harness/owner-paths.json`, `docs/integration/DECISIONS.md` and `AGENTS.md` are owner paths, and `docs/harness/d59.json` becomes one. This ORDER carries the owner's approval for all four. Casey answered in Claude's chat at 2026-10-22T13:56:44Z, and Claude recorded the answer verbatim on #2028 (comment 6342668364). The question was "What should happen when Claude hasn't reviewed a setup PR's head in time?", and the answer:

> "End D59 now"

The chosen option read: "Setup PRs take any other-vendor fresh-context review immediately, like every other PR."

## Files in scope

- `docs/harness/d59.json`, `docs/harness/owner-paths.json`, `docs/integration/DECISIONS.md` and `AGENTS.md` (in the umbrella's scope once claude-29, #2971, merges)
- Shared: `docs/loop/README.md`
- `docs/loop/tasks/infra-loop-harness/claude-30/**`

## Done when

- `d59_in_force` answers `false` on this tree. A test or a direct run shows it, and `loop next` offers a setup PR's review to a non-Claude lab.
- `pr-check --local` passes, including `owner-paths` with the approval recorded. The harness test that every owner path exists passes.
- Claim-wording, links and the copyright check all pass.

## Stop and ask if

- Ending D59 needs a code change. It should not, since the switch is read at the base.
- Any other owner path would have to change.
