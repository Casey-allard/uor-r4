# ORDER: end-to-end-integration, codex-9

Lab: codex. Item: [end-to-end-integration](../../../../roadmap/items/end-to-end-integration.md).
Parent: [codex-8](../codex-8/ORDER.md). Reviewed head: `6f99bbaa8`.

## Why this unit exists

codex-6 and codex-8 proved the exact `code -> positions` index in a test module at 4k and 32k. This
unit moves it into production as `CodeIndex`, adds the catch-up that a session needs, and gives the
copy head an indexed selection (`choose_indexed`). It is the last step before the session plumbing:
the arithmetic and the offsets are proven here, so the plumbing becomes signature work.

## Claim record

`loop claim end-to-end-integration --lab-topic read-index-prod` was refused because
`crates/uor-r4-integer/src/stack/**` overlaps claude-384's PR #3983. This unit adds production code
inside `hamming.rs` and docs, no dependency and no `Cargo.lock` change.

## Method

- `CodeIndex`: `code -> ascending positions`, `built` as the highest inserted position, `catch_up`
  appending positions `built+1 .. t-1` with key `code(tokens[s-1])`, `clear` for a reset/restore,
  and `nearest` with the read's `(distance, position)` tie rule.
- `CopyHeads::choose_indexed`: copy key 1 only; it catches up to the row and returns `nearest`'s
  positions; a pair head still scans (a debug assertion says so).
- A test over a 4,096-token fixture stream compares the indexed selection to `choose` at every row
  and after a `clear`.

## Files in scope

- `crates/uor-r4-integer/src/stack/hamming.rs`
- `docs/loop/tasks/end-to-end-integration/codex-9/**`

Out of scope: passing the index into `mean_at`/`stream`/`readout`, session state, type/ask/pair
heads, any behavior change, a schema, a dependency or a GPU run.

## Done when

1. `CodeIndex` and `choose_indexed` compile in production (not only in tests) and are exact against
   `choose` at every row of the fixture stream.
2. `clear` rebuilds the same answers.
3. No existing test or serving path changes; the Hamming suite still passes.
4. `git diff --check`, the harness contract rule, the REPORT check and the exact-head review pass.

## Cancel

Stop if the catch-up offsets cannot match the read's eligible set exactly, or if the indexed choice
needs a change to `choose`/`nearest`. Do not wire it into the session here.

STOP and report when done.
