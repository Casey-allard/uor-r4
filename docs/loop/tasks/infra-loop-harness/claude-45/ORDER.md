# ORDER: infra-loop-harness, claude-45 (give the two m3 items their notebook page)

Lab: claude. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 7b. Base: `dfa28932e`.

## Goal

Carry out Casey's decision on the #2857 card. Asked how to fix the blocked `m3-durable-memory.md` row for #2857 and #2854, Casey answered "Add notebook: field (Recommended)" (recorded verbatim on #2028 and #2857, 6350203279, 2026-10-22T20:08:33Z).

This unit adds `notebook: docs/research/notebook/m3-durable-memory.md` to both `m3-policy-precedence-and-sieve-tests` (#2857) and `m3-legacy-chatsession-persistence` (#2854), so their notebook rows can land through `pr-check`'s append-only notebook path, which since #2999 also accepts rows added inside the table. It is the same pattern as #2997.

## Cancel probe (run first)

At `dfa28932e`, does either item already declare `notebook:`? If both do, the task is void. Class: instrument.

## Files in scope

- `docs/roadmap/items/m3-policy-precedence-and-sieve-tests.md` and `docs/roadmap/items/m3-legacy-chatsession-persistence.md` (inside `infra-loop-harness`'s `docs/roadmap/items/*.md`)
- This task folder

## Done when

1. Both items declare that `notebook:`, and each diff is one added line.
2. `roadmap check` and `render --check` pass.
3. `report --check` and `pr-check --local` pass.

## Stop and ask if

- The roadmap check rejects `notebook:` on a `done` item.
