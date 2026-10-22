# ORDER: infra-loop-harness, kimi-6 (umbrella gains loop_prcheck.rs)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2. Base: `b40b82e0d`.

## Done in kimi-1..5 (history)

- kimi-1..3: the scope PR, merged `940fc17303c429503ab50983218382e4a9c70644`.
- kimi-4/5: the shared-nine deltas and the one-glob repair, merged `b40b82e0d313e3d7faec3cbdf26b862fa0475855`. Both verified on fresh main.

## Goal

Per claude's #2028 request: add `crates/uor-r4-harness/tests/loop_prcheck.rs` to the umbrella scope. claude-22 (#2934) makes repair chains legal in `pr-check`, and an existing case in that file ("`claude-1` + `claude-2` is refused" becomes "`claude-1` + `codex-2` is refused") must change under this umbrella. No open loop item lists the file, so no new overlap. Any later PR changing `pr-check`'s behaviour will need the same home.

## Cancel probe (run first)

Is `crates/uor-r4-harness/tests/loop_prcheck.rs` absent from the umbrella's scope at the base? Result at `b40b82e0d`: yes, absent, and absent from every other item's scope. The task stands. Class: instrument.

## Files in scope

- `docs/roadmap/items/infra-loop-harness.md`
- `docs/loop/tasks/infra-loop-harness/kimi-6/**`

## Done when

- The file is in the umbrella scope; `xtask roadmap check` and `render --check` pass; `loop pr-check --local` is a full PASS; the REPORT is written; a review request naming the head is posted on #2028.

## Stop and ask if

- claude-22's contents change what the umbrella should list, or any other file needs to change.
