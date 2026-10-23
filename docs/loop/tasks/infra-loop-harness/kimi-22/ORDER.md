# ORDER: infra-loop-harness, kimi-22 (PM item-pointer fix: m4-flock-coupling-prereg names the current draft)

Lab: kimi. Item: [infra-loop-harness](../../../../roadmap/items/infra-loop-harness.md) (#2904). Setup row: 2 (PM maintenance of item files, as with kimi-2's scopes/closes edits).
Base: `7080fd6cf`.

## Goal

`docs/roadmap/items/m4-flock-coupling-prereg.md` line 23 names
`docs/harness/prep/m4-c1-flock-coupling-DRAFT-2026-10-18.md` as "the draft". That file was superseded on 22 October by `docs/harness/prep/m4-flock-coupling-prereg-DRAFT-2026-10-22.md` (antigravity-25, merged at `ea257ef40`), which declares itself the successor and retains the 18 October draft byte-identical. The item's scope glob (`docs/harness/prep/m4-flock-coupling-prereg*`) already covers the 10-22 sheet — kimi-21's proposed scope widening was withdrawn for exactly that reason — but the item's prose still sends the reader to the superseded file. Point the prose at the current sheet, noting the supersession. Promised on #2028 after kimi-21 was withdrawn.

## Cancel probe (run first)

Class **instrument** (docs metadata; no GPU). At the base, `grep -n "m4-c1-flock-coupling-DRAFT-2026-10-18" docs/roadmap/items/m4-flock-coupling-prereg.md` finds line 23 naming the 10-18 file as "the draft". Task stands.

## Files in scope

- `docs/roadmap/items/m4-flock-coupling-prereg.md` (line 23 only)
- this task folder

## Done when

- Line 23 names the 10-22 sheet as the current draft and notes that it supersedes the 10-18 draft (retained byte-identical).
- The rest of the sentence stays true of the 10-22 sheet: it is not registered, its status is `TEST FITNESS: PENDING`, and the five open decisions listed are the same five the 10-22 sheet carries in its §7.
- The diff is that line plus this task folder.
- `loop report --check` and `loop pr-check --local` pass; a fresh-context review from claude is requested (D59).

## Stop and ask if

- The 10-22 sheet turns out not to carry the same five open decisions, so the item's list would also need rewriting.
- Any file beyond the item file and this folder needs to change.
