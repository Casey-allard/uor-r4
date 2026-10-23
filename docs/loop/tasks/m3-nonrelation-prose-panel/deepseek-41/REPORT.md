# REPORT: m3-nonrelation-prose-panel, deepseek-41 (non-author FIT review)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2856 (`References`; the item stays `ready`). Base: `dea171338`.

## What did not work, first

- **The panel and grader were already merged.** I claimed this item for a "panel and grader" unit, then the cancel probe showed antigravity's design PR (`#2952`, `b0430d40`) already landed both, plus the draft sheet. The honest remaining unit was the non-author FIT review, so I pivoted the ORDER to that rather than duplicating work.
- **The sheet still described the pass bar as pending.** The owner's standing-direction decision at `2026-10-23T02:36:36Z` registered the bar exactly as proposed (card `6337042887`); the sheet's §4 and status did not reflect it. §4 now quotes it with the ISO-UTC time.
- **The base artifact is unpinned, and that is a real FIT blocker.** The evaluation feeds the panel through a saved `world=v2` compiler; `TurnCompiler::identity()` carries `artifact_sha256`, `tokenizer_sha256` and `label_schema`, but the sheet names no artifact file, path or hash. A FIT would clear an unfixed configuration, so the verdict is `NOT_FIT` until that is pinned. I did not invent an artifact hash.
- **A shell quoting slip mangled the first rendered record** (`$0.05` expanded); I re-ran `prep fit` with the note single-quoted and used the corrected block.

## What changed

| File | Change |
|---|---|
| `docs/labs/m3-nonrelation-prose-panel/preregistration-DRAFT-2026-10-22.md` | status → `NOT_FIT`, §4 quotes the registered bar with time/card, §3 cites the grader commit, `## Verdict` with the artifact-pin prerequisite and the rendered record |
| `docs/labs/m3-nonrelation-prose-panel/README.md` | the non-author review note |
| this folder | ORDER (committed first) and this REPORT |

**Size:** about 55 docs lines; docs-only (no receipts file, D46 fresh-context record used).

## Measurements

- `data/panels/nonrelation-prose-v1.txt` SHA-256 = `7c29c665f97bae126db0cf82cee1795d2160c2ad35f852f64fffc92b836a3de7`, matching `data/panels/MANIFEST.sha256`.
- `cargo test -p uor-r4-training --lib nonrelation_prose_panel` → **4 passed** (panel hash/counts, clean stub zeros, #1718 false-assert/false-overwrite detection, held-out four-word disjointness).
- Grader/panel commit: `b0430d40` (`#2952`).
- `prep fit -m m3 --verdict NOT_FIT …` recorded the receipt and rendered the block now in the sheet.

## Checks run at the head

`loop report --check`, `loop pr-check --local`, `loop gates --full`, and a fresh-context review record at the head (docs-only, D46).

## Skills used

- `solution-integrity`: checked the actual artifact identity path instead of accepting the sheet's word; recorded a blocker rather than a soft pass.
- `scope-guard`: docs-only inside the item's scope; no data file, hash or bar was invented.

## Still open

- Pin the saved `world=v2` compiler artifact and tokenizer in the sheet, then re-request the non-author FIT; the baseline GPU run additionally needs a lease and Casey's typed release.

## Plain summary

- Reviewed the pre-registration as the non-author and recorded **NOT_FIT**, clearing no compute.
- The panel, hash and grader all verify; the one missing pin is the saved compiler artifact's identity, plus the sheet needed the now-registered bar quoted.