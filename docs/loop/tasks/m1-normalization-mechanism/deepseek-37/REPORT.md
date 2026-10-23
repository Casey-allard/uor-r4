# REPORT: m1-normalization-mechanism, deepseek-37 (non-author TEST FITNESS review)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2842 (`References`; the item stays `ready`). Base: `30febc5ad`.

## What did not work, first

- **The sheet cites a decision card as owner confirmation.** Its Status line pointed at `#2028#issuecomment-6338596024` as "the owner's bar confirmation". The comment is **antigravity's decision card** proposing the bar; the owner has not answered (the PM routing lists these bar cards as awaiting the owner). A sheet cannot be FIT while the bar it would clear is a proposal.
- **The inherited DEV pin is ambiguous.** The sheet carries prefixes only; the repo holds two conflicting full DEV pairs: the authoritative code pin (`attention_learning.rs`: tokens `0919e90f953e…`, mask `a838bb1a67e9…`, matching `42,856` payload tokens / `14,287` targets and the archived `a2-pilot`) versus the runbook staging sheet's `0919e90f813b…` / `a838bb1ae20c…`, which appears nowhere else. Under prefixes the ambiguity is invisible; the sheet must pin the full authoritative pair and the staging sheet needs an errata.
- **No normalization-specific defect found beyond these two.** The construction, matched controls, structural bar and controls are internally consistent; the six finite controls pass.

## What changed

| File | Change |
|---|---|
| `docs/harness/prep/m1-normalization-mechanism-DRAFT-2026-10-22.md` | Status line → `TEST FITNESS: NOT_FIT`, corrected the decision-card wording, `## Verdict` with the two prerequisites and the rendered review record |
| `docs/labs/m1-normalization-mechanism/README.md` | a note recording the non-author review and its prerequisites |
| this folder | ORDER (committed first) and this REPORT |

**Size:** about 60 docs lines; docs-only (no receipts file, D46 fresh-context record used).

## Measurements

- `cargo test -p uor-r4-discrete --lib normalization_mechanism` → **6 passed, 0 failed** (norm stability, idempotence, scale equivariance/majority preservation, tie-breaking, 64-frame equivariance + export, multistage residual vs unnormalized drift).
- `uor-harness prep fit -m m1 --verdict NOT_FIT …` recorded one receipt and rendered the block now in the sheet.
- `prep-check`: the sheet remains `complete=yes`, now `NOT_FIT`, `may_compute=false`.
- No float, dense matmul or squashing form on the module path (grep); `NormFrameTable::export_words`/`information_bits` give the `4 × B`-bit whole-number export; the A8 parent sheet is `TEST FITNESS: FIT`; all 8 blocks present.

## Checks run at the head

`loop report --check`, `loop pr-check --local`, `loop gates --full`, `prep-check`, and a fresh-context review record at the head (docs-only, D46).

## Skills used

- `uor-research-architect`: checked the normalization construction and its exact-integer invariants against the sheet, not just the prose.
- `solution-integrity`: verified the pins against code and archives instead of trusting prefixes; recorded the mismatch.

## Still open

- The item stays `ready`. To re-request FIT: the owner's bar answer with ISO-UTC time and the full DEV pin with a staging-sheet errata.
- The GPU run remains behind FIT, the trunk item, PM dispatch and Casey's typed D40 release; no compute was touched.

## Plain summary

- I reviewed the draft as the non-author and recorded **NOT_FIT**, clearing no compute.
- Two pins are needed: the owner bar (the linked comment is a decision card, not an answer) and the full DEV hashes (two conflicting pairs exist; the code pin is authoritative).
- The `OrbitRootSnapNormV1` construction and its 6/6 finite controls check out.