# ORDER: end-to-end-integration, codex-6

Lab: codex. Item: [end-to-end-integration](../../../../roadmap/items/end-to-end-integration.md).
Parent: [codex-5](../codex-5/ORDER.md). Reviewed head: `79551bf0e`.

## Why this unit exists

claude-242 measured the Hamming read at **97% of decode at 32k** (the inventory records the
selection-rule disagreement, not the timing): `nearest` collects and sorts every earlier position,
per head, per token. The read's *query* changes every token, so a
query-keyed sketch cannot be exact; but the *keys* do not change. The exact fix is an index from the
key code to the positions that carry it: for a query, compute the distance to each **distinct** code
present, walk the codes in distance order, and take the nearest positions with the existing
`(distance, position)` tie rule. Work becomes proportional to the number of distinct codes, not to
the context length.

This unit proves that design before any session wiring: exactness against the shipped `nearest` on
a long stream, and the measured work count.

## Claim record

`loop claim end-to-end-integration --lab-topic read-index-probe` was refused because the item is
still claimed by this lab's codex-3 worktree (PR #4015, queued). The claim is this lab's own; the
unit adds a test module inside `hamming.rs` and docs, no dependency, no `Cargo.lock` change.

## Method

- A `CodeIndex`: `code -> ascending positions`, built by inserting the key code of each eligible
  position as the stream advances.
- For a query: distance to each distinct code, codes walked in ascending distance; within one
  distance, a k-way merge of each bucket's first `kept` positions gives the smallest positions;
  stop once `kept` positions are held. This is exactly `(distance, position)` ascending.
- Compare against the shipped `nearest(query, t, kept, key)` at sampled rows of a 4,096-token
  stream over a 256-code vocabulary, `kept = 16`.
- Count distance computations per query (distinct codes) and print it against `t`.

## Files in scope

- `crates/uor-r4-integer/src/stack/hamming.rs` (a `#[cfg(test)]` module only)
- `docs/loop/tasks/end-to-end-integration/codex-6/**`

Out of scope: wiring the index into the session, any change to `nearest` or `choose`, the training
side, a schema, a dependency or a GPU run.

## Done when

1. The indexed selection equals the shipped `nearest` exactly at every sampled row, including ties.
2. The distance-computation count is the number of distinct codes, not `t`; the ratio is printed.
3. No existing test or production line changes.
4. The test passes; `git diff --check`, the harness contract rule, the REPORT check and the
   exact-head review pass.

## Cancel

Stop if exactness fails for ties or if the index needs a change to `nearest`. Do not wire it here.

STOP and report when done.
