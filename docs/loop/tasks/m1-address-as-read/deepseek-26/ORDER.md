# ORDER: m1-address-as-read, deepseek-26 (non-author FIT review of the serving-time address-as-read oracle)

Lab: deepseek. Item: [m1-address-as-read](../../../../roadmap/items/m1-address-as-read.md) (#2833). Base: `683e182ba`. Open to any lab; Casey directed Route A in chat, PM confirmed on #2028 at 2026-10-22T16:34:09Z.

## Goal

Perform the non-author `TEST FITNESS` review of [`m1-address-as-read-oracle-DRAFT-2026-10-22.md`](../../../../harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md) (CHARTER §4.5, D22), as antigravity's sheet asks before any GPU evaluation: check the eight blocks, the finite controls, the arms, the frozen/hashed panel, stop rules, cost and minimum test power, and record the verdict with `uor-harness prep fit`. This is host-only; no GPU run, no training, no bar/hold/spend decision.

## Cancel probe (run first)

Class **instrument** (review work; no GPU). Ran `uor-harness prep-check`: the sheet is `PENDING`, machine-complete (all 8 blocks present). Ran `cargo test -p uor-r4-object-store --test address_read`: 6 passed. Checked the artifacts the sheet would clear: `m3-memory.json` has no panel SHA-256 and says `>= 200` rows for a decisive run; `m1-trunk-assembly-and-training` (`#2845`) is `held`; and the bar is not owner-registered. The review is genuinely needed and the verdict will be `NOT_FIT` with those constraints named.

## Files in scope

`docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md`, `docs/loop/tasks/m1-address-as-read/deepseek-26/**`. The notebook row in the item's step 6 belongs to the evaluated result, not this review, and `docs/research/notebook/**` is outside the item's scope, so it is left for the run unit.

## Done when

- The sheet carries a `## Verdict` with the `prep fit`-rendered `NOT_FIT` record (reviewer deepseek, 2026-10-22) and its top status says no compute is cleared.
- The verdict names the four concrete prerequisites in priority order: the frozen/hashed panel (composition, 80 vs 200, SHA-256), the exact checkpoint SHA-256 (#2845 is held), the owner-registered bar with ISO-UTC time, and the commit-pinned consumer (#2950).
- The review is posted on #2833; `report --check`, `pr-check --local`, `loop gates --full` and `--break all` pass; receipts at the final head.

## Stop and ask if

- Any GPU execution becomes necessary (it must not: the run is a separate unit behind the owner's bar and release).
- Any file outside the scope above would need to change.