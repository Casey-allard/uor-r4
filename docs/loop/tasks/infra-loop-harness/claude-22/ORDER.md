# ORDER: infra-loop-harness, claude-22 (repair ORDERs in one PR; shared registration files)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: A. Base: `9f5894d85`.

## Goal

1. **Repair ORDERs.** Step 11 of [the loop page](../../../README.md) gives a repair a new order, `<lab>-<n+1>`. But `pr-check`, `loop review`, `loop next`, `loop report` and `loop close` refuse a PR that adds more than one ORDER, so #2929 and #2930 cannot get a review record. The new rule: when every ORDER a PR adds is in one task folder and from one lab, the highest `n` is the active order and the rest are history.
2. **Shared registration files.** Casey's answer in Claude's chat, 2026-10-21T23:43Z: "Exempt them (Recommended)". These six are exempt from item scopes:
   - `crates/uor-r4-harness/src/{lib,main,loopgates,doccheck}.rs`
   - `docs/harness/README.md`
   - `docs/loop/README.md`

   Casey added three more in Claude's chat, 2026-10-22T00:05Z, "Add all three (Recommended)": `crates/uor-r4-harness/src/loopprcheck.rs`, `docs/harness/gates.json` and `docs/roadmap/README.md`. Any loop PR may change these nine, and no item scope may list them.

The planted fault: **a PR whose added ORDERs span two labs or two task folders is still refused, and a review record for a superseded ORDER does not count.**

## Cancel probe (run first)

At `9f5894d85`, does any tool accept a PR that adds two ORDERs? No. `the_order` errors with "the PR adds N ORDERs", and `loop review --pr 2929` exits with that error. Is any shared file exempt from `scope`? No. The task stands. Class: instrument.

## Calls

- One pure helper resolves the active ORDER from a list of added ORDER paths. Every tool uses it.
- `commit-order`: the lowest-numbered ORDER must be the PR's first commit, alone. Each later ORDER must be added in a commit that touches only its own task folder.
- `report`: only the active ORDER's REPORT is checked.
- `review-record`: a record's `order` must equal the active ORDER.
- One list of the shared files, used by `pr-check`'s `scope` rule (always allowed) and by item-scope validation (refused).

## Files in scope

- `crates/uor-r4-harness/src/**` and `crates/uor-r4-harness/tests/**`
- `docs/loop/README.md`: step 11, and the shared-files rule with Casey's words.
- `docs/loop/tasks/infra-loop-harness/claude-22/**`

## Done when

- A PR with `kimi-1`, `kimi-2` and `kimi-3` resolves to `kimi-3` in every tool.
- Two labs, or two folders, are refused (the planted fault).
- A record for `kimi-2` does not count once `kimi-3` exists.
- A change to a shared file passes `scope`, and an item scope that lists one is refused.
- The full harness suite, claim-wording, links and the copyright check pass.

## Stop and ask if

- Any file outside scope must change, or Casey's list would need a tenth file.
