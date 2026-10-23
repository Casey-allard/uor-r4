# REPORT: m1-position-mechanism, deepseek-39 (pin the registered bar and the full DEV pair)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2843 (`References`; the item stays `ready`). Base: `6ffc91617`.

## What did not work, first

- **The draft still described the bar as an unanswered card.** After my `deepseek-36` review, the owner's standing-direction decision at `2026-10-23T02:36:36Z` registered the §3 bar exactly as proposed; the sheet's status and §3 still said "proposed … no owner answer", so a reader would not know compute is gated only by the slice hash and the FIT.
- **The sheet carried prefixes while the repo has two conflicting full DEV pairs.** I pinned the authoritative code-pin pair in §5 and recorded the staging-sheet pair as the PM errata it is; a prefix cannot be checked against a conflict.
- **The order-sensitive slice had no definition.** It now has a deterministic rule (query token ≥2 times in the prefix with distinct successors; `N ≥ 200`) and a `PENDING` hash slot for the frozen subset. I did not invent a hash for data that is not frozen.
- **`TEST FITNESS` remains `NOT_FIT` on purpose.** The slice hash is not frozen and the FIT must come from a non-author lab (Claude or Kimi); this unit clears no compute.

## What changed

| File | Change |
|---|---|
| `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md` | Status line and §3 quote the registered bar with the 02:36:36Z time and comment id; §5 pins the full TRAIN/DEV digests from the code pin and records the staging-sheet errata and the slice rule with its `PENDING` hash; a `### Post-registration update` records what is resolved and what remains |
| `docs/labs/m1-position-mechanism/README.md` | the post-registration update note |
| this folder | ORDER (committed first) and this REPORT |

**Size:** about 55 docs lines; docs-only (no receipts file, D46 fresh-context record used).

## Measurements

- No compute was run; this is a registration/record unit. The authoritative DEV digests were read from `attention_learning.rs:1372-1384` (tokens `0919e90f953e…`, mask `a838bb1a67e9…`, 42,856 payload tokens / 14,287 targets) and cross-checked against the archived `a2-pilot`.
- `prep-check` on the sheet: `complete=yes`, still `NOT_FIT`, `may_compute=false`.

## Checks run at the head

`loop report --check`, `loop pr-check --local`, `loop gates --full`, `prep-check`, and a fresh-context review record at the head (docs-only, D46).

## Skills used

- `solution-integrity`: pinned the full digests instead of trusting prefixes; kept `NOT_FIT` because one prerequisite (the frozen slice hash) genuinely is not pinned.
- `scope-guard`: docs-only inside the item's scope; no data file, no hash and no owner value was invented.

## Still open

- Freeze the order-sensitive subset and pin its SHA-256; have the PM correct the staging-sheet digest line (or record the errata); then a non-author FIT from Claude or Kimi, followed by the held `#2844` checkpoint, PM dispatch and Casey's typed release.

## Plain summary

- Recorded the owner's registered bar and pinned the full DEV digests in the position sheet; defined the order-sensitive slice rule with a `PENDING` hash.
- The sheet stays `NOT_FIT` until the slice is frozen and a non-author FIT lands; no compute is cleared.