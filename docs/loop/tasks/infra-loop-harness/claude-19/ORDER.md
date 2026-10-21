# ORDER: infra-loop-harness, claude-19 (mark merged loop items done)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 1. Base: `1b2f5c9d1`.

## Goal

Loop PRs 2, 4, 5, 6, 7a and 7b are merged and verified on `main`, but their roadmap items still read `status: ready` and their issues are open. Set each of those six items to `status: done`, so that `roadmap sync` closes #2905, #2907, #2908, #2909, #2910 and #2911.

Loop PR 3 (#2906) stays `ready`: its done-when includes `docs/loop/baseline.json`, which is not built.

The planted fault: **an item whose PR has not merged is marked done.** The REPORT lists each item with its merge commit on `main`.

## Cancel probe (run first)

Are the six items still `status: ready` at the base? Result at `1b2f5c9d1`: yes, all six; their issues are open with no comments. The task stands. Class: instrument.

## Files in scope

- `docs/roadmap/items/infra-loop-pr02-item-keys.md`, `infra-loop-pr04-gates.md`, `infra-loop-pr05-policy-check.md`, `infra-loop-pr06-review-records.md`, `infra-loop-pr07a-claim.md`, `infra-loop-pr07b-session.md`: the status line only.
- `docs/roadmap/README.md`, if `roadmap render` changes it.
- `docs/loop/tasks/infra-loop-harness/claude-19/**`

## Done when

- Each of the six items reads `status: done`, and #2906's item still reads `ready`.
- `roadmap check` and `roadmap render --check` pass; claim-wording, links and the copyright check pass.
- After merge, a `roadmap sync` dry run lists the six closes, and the applied sync closes them.

## Stop and ask if

- The sync dry run would change any issue other than these six.
- Any file outside scope needs to change.
