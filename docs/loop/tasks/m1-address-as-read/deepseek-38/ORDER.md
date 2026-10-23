# ORDER: m1-address-as-read, deepseek-38 (file the two registration cards and record them in the oracle sheet)

Lab: deepseek. Item: [m1-address-as-read](../../../../roadmap/items/m1-address-as-read.md) (#2833). Base: `1c728ff87`. Item PR (partial: `References #2833`; the item stays `ready`).

## Goal

Turn the two owner/PM prerequisites of the merged non-author FIT review (`ee2f61417`) that a lab can legitimately propose into **decision cards with a stated recommendation** — the oracle gain bar and the frozen 80-row panel composition — and record both in the oracle sheet's pin-status table so the PM/owner can register them under the 02:36Z standing direction without re-deriving the numbers. This unit decides nothing the owner/PM must decide; it clears no compute.

## Cancel probe (run first)

Class **instrument** (docs only; no GPU, no training). At the base: the sheet still shows prerequisites 1 and 3 `PENDING`; the FIT review is `NOT_FIT`; no bar card or panel card for this item exists on #2028. Task stands.

## Files in scope

- `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md`
- `docs/labs/m1-address-as-read/**`
- this task folder

## Done when

- Two decision cards with recommendations are posted on `#2028` and linked from `#2833`: (a) oracle gain bar `27/40` stated-fact vs the single `19/40` baseline, `7/20` historical, `20/20` abstention, `0` wrong-value substitutions, BPB within `0.01` on sealed fresh V2, derivation `27+7+20 = 54/80`; (b) frozen panel composition `40/20/20`, with the 80-vs-200 reconciliation stated (80 = this oracle's evaluation panel; `>= 200` = the M3 decisive-run panel).
- The oracle sheet's Status line and pin-status table record the cards as filed (`CARD FILED — PM/owner`), keep the held `#2844` checkpoint as `PENDING`, and keep `NOT_FIT` with no compute cleared.
- A short lab-record note is added; `loop report --check`, `loop pr-check --local`, `loop gates --full` and a fresh-context record at the head pass (docs-only PR).

## Stop and ask if

- A card would have to decide a value the owner has not delegated (it does not: each card states a recommendation and can be overridden).
- Any file outside the scope above must change.