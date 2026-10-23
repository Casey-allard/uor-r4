# REPORT: m1-normalization-mechanism, deepseek-40 (pin the registered bar and the full split digests)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2842 (`References`; the item stays `ready`). Base: `26e19ac5b`.

## What did not work, first

- **The draft still described the bar as an unanswered card.** After my `deepseek-37` review, the owner's standing-direction decision at `2026-10-23T02:36:36Z` registered the §3 bar exactly as proposed; the sheet's status and §3 still said "proposed … no owner answer", so a reader would not know compute is gated only by the non-author FIT.
- **The sheet carried prefixes while the repo has two conflicting full DEV pairs.** I pinned the authoritative code-pin pairs in §5 and recorded the staging-sheet pair as the PM errata it is; a prefix cannot be checked against a conflict.
- **`TEST FITNESS` remains `NOT_FIT` on purpose.** The FIT must come from a non-author lab (Claude or Kimi); this unit clears no compute.

## What changed

| File | Change |
|---|---|
| `docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md` | Status line and §3 quote the registered bar with the 02:36:36Z time and comment id; §5 pins the full TRAIN/DEV digests from the code pin, ties fresh V2 to the parent FIT record and records the staging-sheet errata; a `### Post-registration update` records what is resolved and what remains |
| `docs/labs/m1-normalization-mechanism/README.md` | the post-registration update note |
| this folder | ORDER (committed first) and this REPORT |

**Size:** about 55 docs lines; docs-only (no receipts file, D46 fresh-context record used).

## Measurements

- No compute was run; this is a registration/record unit. The authoritative TRAIN/DEV digests were read from `attention_learning.rs:1363-1384` (TRAIN `3dee858f…`/`b5e7352d…`, 7,512,797 tokens / 2,425,925 targets; DEV `0919e90f953e…`/`a838bb1a67e9…`, 42,856 / 14,287) and cross-checked against the archived `a2-pilot`.
- `prep-check` on the sheet: `complete=yes`, still `NOT_FIT`, `may_compute=false`.

## Checks run at the head

`loop report --check`, `loop pr-check --local`, `loop gates --full`, `prep-check`, and a fresh-context review record at the head (docs-only, D46).

## Skills used

- `solution-integrity`: pinned the full digests instead of trusting prefixes; kept `NOT_FIT` because the non-author FIT genuinely has not happened.
- `scope-guard`: docs-only inside the item's scope; no data file, hash or owner value was invented.

## Still open

- Have the PM correct the staging-sheet digest line (or record the errata); then a non-author FIT from Claude or Kimi, followed by the held `#2844` checkpoint, PM dispatch and Casey's typed release.

## Plain summary

- Recorded the owner's registered bar and pinned the full TRAIN/DEV digests in the normalization sheet, with the staging errata routed to the PM.
- The sheet stays `NOT_FIT` until a non-author FIT lands; no compute is cleared.