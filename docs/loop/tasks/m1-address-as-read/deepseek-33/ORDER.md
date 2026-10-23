# ORDER: m1-address-as-read, deepseek-33 (pin the sheet's consumer and expose the owner slots)

Lab: deepseek. Item: [m1-address-as-read](../../../../roadmap/items/m1-address-as-read.md) (#2833). Base: `ea257ef40`. Item PR (partial: `References #2833`; the item stays `ready`).

## Goal

Act on the merged non-author `NOT_FIT` review of the serving-time address-as-read oracle sheet (`#2833`; record `ee2f61417`): pin the one prerequisite that is the lab's to pin — the delivered `uor-r4-object-store` consumer and its six finite controls by commit — and turn the remaining four into one explicit checklist of `PENDING (PM/owner)` slots with the exact information each needs, so a single decision round re-enables the FIT re-request.

This unit decides **no owner value**: no bar, no panel hash, no checkpoint SHA, no BPB tolerance, no 80-vs-200 reconciliation and no compute. It also adds the two missing registered stop rules (bar-miss stop; negative-closes rule) that the FIT review listed as part of prerequisite 5.

## Cancel probe (run first)

Class **instrument** (docs-only, host only, no compute). At the base: the sheet exists with the five prerequisites in its Verdict; the consumer commit is identifiable (`86eb68ebb`, PR `#2950`, six tests); the sheet still says `NOT_FIT` and names no consumer commit. Task stands. Nothing is run on a GPU and no pre-registration is registered.

## Files in scope

- `docs/harness/prep/m1-address-as-read*` (the DRAFT sheet)
- `docs/labs/m1-address-as-read/**` (lab record)
- this task folder

## Done when

- The sheet pins the consumer: commit `86eb68ebb`, the `address_read.rs` module and the six `tests/address_read.rs` controls, with the re-check command.
- The remaining prerequisites are exactly one checklist in the Verdict: (1) frozen panel SHA-256 + 80-vs-200 D22 §3 reconciliation — `PENDING (PM)`; (2) checkpoint SHA-256 — `PENDING (owner)`, `#2844` held; (3) owner bar with ISO-UTC time and one baseline — `PENDING (owner)`; (5) BPB stream/tolerance and the `54/80` derivation — `PENDING (PM/owner)`, plus the two new stop rules.
- The sheet still says `NOT_FIT` and no compute is cleared; the item stays `ready`.
- `loop report --check`, `loop pr-check --local`, `loop gates --full` and a fresh-context review record at the head pass (docs-only PR).

## Stop and ask if

- Any owner value would have to be set to finish (it would not; the unit stops at the checklist).
- Any file outside the scope above must change.
