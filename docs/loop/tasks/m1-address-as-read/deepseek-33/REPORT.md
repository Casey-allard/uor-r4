# REPORT: m1-address-as-read, deepseek-33 (sheet consumer pin and the owner-slot checklist)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item `#2833` (`References`; the item stays `ready`). Base: `ea257ef40`.

## What did not work, first

- **The FIT review's five prerequisites cannot all be pinned by a lab.** Three need owner/PM values: the frozen-panel hash (with the 80-vs-200 D22 §3 reconciliation), the held `#2844` checkpoint SHA, and the registered bar with an ISO-UTC time and one baseline. Writing any of them here would be inventing an owner value, so this unit leaves them explicit `PENDING` slots.
- **The `54/80` figure has no unique derivation yet.** From the 40 stated-fact / 20 historical / 20 abstention composition and a `+8` gain over one baseline, the full-panel number depends on how the historical and abstention halves are scored and which baseline (`19/40` or `20/40`) is registered; it is marked `PENDING` with the equation the PM must settle rather than guessed.
- **The held-out BPB stream is unnamed.** The sheet's `0.01` tolerance cannot be checked against a stream that is not pinned; the sheet now says so explicitly and proposes the lab definition (same frozen stream as Arm A, named by path and SHA-256) for the PM to accept or replace.

## What changed

| File | Class | Lines | What |
|---|---|---|---|
| `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md` | docs | ~45 changed | status note; bar marked `PROPOSED/PENDING owner`; consumer pinned at `86eb68ebb`; checkpoint/panel/BPB slots marked `PENDING`; bar-miss and negative-closes stop rules; one pin-status table in the Verdict; `NOT_FIT` unchanged |
| `docs/labs/m1-address-as-read/README.md` | docs | +~15 | the lab record of this pin update and the pending list |
| this task folder | docs | new | ORDER and REPORT |

**Size:** about 115 changed lines at this head (the sheet itself about +25/−5; see the PR diff for the exact numbers); docs-only (no receipts file, D46 fresh-context record used).

## Measurements

- Consumer pin: commit `86eb68ebb` added `crates/uor-r4-object-store/src/address_read.rs` and `crates/uor-r4-object-store/tests/address_read.rs` (PR `#2950`); the sheet's re-check command is `cargo test -p uor-r4-object-store --test address_read`, recorded as 6 passed at that commit.
- The sheet remains machine-complete to shape (`prep-check` `NOT_FIT`, `may_compute: false`); no bar, panel, checkpoint or tolerance value was added.

## Checks run at the head

`loop report --check`, `loop pr-check --local`, `loop gates --full`, `prep-check` on the sheet, link check for the changed docs, and the same-lab fresh-context review record (docs-only, D46).

## Skills used

- `solution-integrity`: the unit pins only what is provable (the delivered consumer commit) and leaves every owner value as an explicit `PENDING` slot instead of inventing it.
- `scope-guard` / `working-discipline`: docs-only edits inside the ORDER's scope; no compute, no owner-path change, no case/panel data committed.

## Still open

- **PM/owner:** panel SHA-256 + 80-vs-200 reconciliation; checkpoint SHA-256 (`#2844` held); registered bar + ISO-UTC time + one baseline; BPB stream/tolerance. When those are in the sheet, re-request the non-author FIT; the GPU oracle run still needs PM dispatch and Casey's typed release.
- The item's host half is delivered; the item stays `ready`.

## Plain summary

- The one prerequisite a lab could pin — the consumer and its six controls — is now pinned by commit in the sheet.
- Everything else is one checkout table: no bar, panel, checkpoint or tolerance is invented; the sheet keeps `NOT_FIT` and clears no compute.
- Added the two missing stop rules so a bar miss is a configuration negative and an instrument failure is an instrument blocker.
