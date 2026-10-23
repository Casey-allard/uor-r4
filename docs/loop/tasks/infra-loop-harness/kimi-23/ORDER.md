# ORDER: infra-loop-harness, kimi-23 (widen the umbrella scope so the prep.rs bug cards are claimable)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (PM maintenance of item files, as with kimi-2 and kimi-22).
Base: `7080fd6cf`.

## Goal

Cards #3041 (`prep::test_fitness` reads the FIRST TEST FITNESS line, so an appended FIT never wins) and #3042 (`prep::render_fit` emits a trailing space and leaked indentation) are verified harness bugs in `crates/uor-r4-harness/src/prep.rs`, carded from claude-57's REPORT (merged #3034) and confirmed at source. Both have been recorded as "open, unassigned, needs a scope widening first" in the kimi-20 and kimi-22 REPORTs, because `prep.rs` sits outside the umbrella item's `scope:` line and no lab can claim them without one. This PR does the widening — and nothing else — so either card becomes claimable under the umbrella by any lab. The timing is live: PR #3052's NOT_FIT flow (a FIT later appended over an earlier NOT_FIT) is exactly the re-review path #3041 breaks.

## Cancel probe (run first)

Class **instrument** (docs metadata; no GPU). At the base, `grep -c "prep.rs\|preprules" docs/roadmap/items/infra-loop-harness.md` finds 0 matches: neither `prep.rs` nor `preprules.rs` is in the scope line. Task stands.

## Files in scope

- `docs/roadmap/items/infra-loop-harness.md` (the `scope:` line only)
- this task folder

## Done when

- The scope line additionally lists `crates/uor-r4-harness/src/prep.rs` and `crates/uor-r4-harness/src/preprules.rs` (prep.rs's rules companion; both cards' fixes will live in these two files).
- No code changes in this PR; the bug fixes themselves belong to whoever claims #3041/#3042 afterward.
- `roadmap render` produces no diff and `roadmap check` passes; `loop report --check` and `loop pr-check --local` pass; a fresh-context review from claude is requested (D59).

## Stop and ask if

- Fixing either bug turns out to require a file beyond prep.rs/preprules.rs and the existing test files — that would be the claiming lab's stop-and-ask, not this PR's.
