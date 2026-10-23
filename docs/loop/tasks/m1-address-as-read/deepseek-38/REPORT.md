# REPORT: m1-address-as-read, deepseek-38 (file the oracle's two registration cards)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2833 (`References`; the item stays `ready`). Base: `1c728ff87`.

## What did not work, first

- **The oracle could not progress by claiming its own PENDING values.** The merged FIT review lists a bar and a frozen panel as owner/PM prerequisites; a lab cannot write either. The honest lab-side move was to file two decision cards with recommendations and record them, not to invent a bar or a panel hash.
- **The `54/80` derivation was genuinely ambiguous.** `+8` over `19/40` gives `27/40` stated-fact, and the remaining `27` of `54` was unassigned; the bar card now assigns it as `7` historical-version rows plus `20` abstention rows, so the total has one derivation. The `20/40` anchor is dropped, so `+8` has one baseline.
- **The 80-vs-200 row conflict was unresolved.** The milestone's `>= 200` rows is the M3 decisive-run panel; this narrow serving-time oracle evaluates `80` rows. The panel card states the reconciliation explicitly instead of leaving the sheet silent.
- **The held checkpoint cannot be pinned by any lab action.** `m1-trunk-assembly-and-training` (`#2844`) is HELD and has no SHA; the sheet now says the checkpoint is the only prerequisite left outside lab control.

## What changed

| File | Change |
|---|---|
| `docs/harness/prep/m1-address-as-read-oracle-DRAFT-2026-10-22.md` | Status-line update; pin-status rows 1/3/5 now record the two filed cards and the proposed BPB stream/tolerance; a `### Registration cards` section gives the proposals and links; `NOT_FIT` and the held checkpoint `PENDING` remain |
| `docs/labs/m1-address-as-read/README.md` | a short record of the two cards and the one remaining prerequisite |
| this folder | ORDER (committed first) and this REPORT |

**Size:** about 55 docs lines; docs-only (no receipts file, D46 fresh-context record used).

## Measurements

- No compute was run; this is a registration/record unit. The two cards are `#2028` comments `6356053677` (bar) and `6356054074` (panel), cross-linked from `#2833` (`6356054371`).
- `prep-check` on the sheet: `complete=yes`, `NOT_FIT`, `may_compute=false` — unchanged, as intended.

## Checks run at the head

`loop report --check`, `loop pr-check --local`, `loop gates --full`, `prep-check`, and a fresh-context review record at the head (docs-only, D46).

## Skills used

- `solution-integrity`: did not convert a proposal into a registered value; the cards are explicitly recommendations, and the sheet keeps `NOT_FIT`.
- `scope-guard`: docs-only inside the item's scope; no panel file, bar value or checkpoint was invented or committed.

## Still open

- Register the two cards (owner/PM), select the panel data source and freeze/hash it, and resolve the held `#2844` checkpoint. Then re-request the non-author FIT; the GPU run additionally needs PM dispatch and Casey's typed release.

## Repair after the live registration (same PR, second commit)

While this PR was open, Claude recorded the owner's standing-direction registration of both cards at **2026-10-23T02:44:51Z** (`#2028` comment `6356136269`). The first head was a clean pre-registration snapshot but stale: it said "CARD FILED" after the registration already existed. The repair records the registration with its ISO-UTC time in the status line, pin-status table and a renamed `### Registration` section, marks prerequisites 1 (composition) and 3 (bar) resolved while keeping the panel hash and the held `#2844` checkpoint as `PENDING`, and notes that the next FIT must come from a lab other than deepseek (deepseek has authored text in this sheet) and that the `#2833` registration comment links to an unrelated old comment.

## Plain summary

- Filed the two decision cards the oracle needs (gain bar, frozen 80-row panel) with recommendations and one derivation of `54/80`, and recorded them in the oracle sheet.
- The only remaining prerequisite a lab cannot pin is the held checkpoint; `NOT_FIT` stands and no compute is cleared.