# REPORT: m1-position-mechanism, deepseek-36 (non-author TEST FITNESS review)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2843 (`References`; the item stays `ready`). Base: `9960de160`.

## What did not work, first

- **The sheet cites a decision card as owner confirmation.** Its Status line pointed at `#2028#issuecomment-6337522273` as "the owner's bar confirmation". That comment is **antigravity's** decision card proposing `F_order`; the owner has not answered, and the PM's routing note lists these bar cards as awaiting the owner. A sheet cannot be FIT while the bar it would clear is a proposal.
- **The inherited DEV pin is ambiguous.** The sheet carries prefixes only, so I checked the full digests. Two conflicting DEV pairs exist in the repo: the authoritative code pin (`attention_learning.rs`: tokens `0919e90f953e…`, mask `a838bb1a67e9…`, matching the pinned `42,856` payload tokens / `14,287` targets and the archived `a2-pilot`), and the staging sheet's `0919e90f813b…` / `a838bb1ae20c…`, which appears nowhere else. Under prefixes the ambiguity is invisible; the sheet must pin the full authoritative pair and the staging sheet needs an errata.
- **The new evaluation slice has no identity.** The bar's multi-occurrence order-sensitive slice (`N ≥ 200`) has no deterministic selection rule, row count or SHA-256. A run scored on it would not be reproducible from the sheet.

## What changed

| File | Change |
|---|---|
| `docs/harness/prep/m1-position-mechanism-DRAFT-2026-10-22.md` | Status line → `TEST FITNESS: NOT_FIT`, corrected the decision-card wording, `## Verdict` with the three prerequisites, and the rendered review record |
| `docs/labs/m1-position-mechanism/README.md` | a note recording the non-author review and its prerequisites |
| this folder | ORDER (committed first) and this REPORT |

**Size:** about 60 docs lines; docs-only (no receipts file, D46 fresh-context record used).

## Measurements

- `cargo test -p uor-r4-discrete --lib position_mechanism` → **5 passed, 0 failed** (identity, prefix-holonomy/direct/transitive, integer bounds, non-commutative order detection, table validation/export).
- `uor-harness prep fit -m m1 --verdict NOT_FIT …` recorded one receipt and rendered the block now in the sheet.
- `prep-check`: the sheet remains `complete=yes`, now `NOT_FIT`, `may_compute=false`.
- No float, dense matmul or squashing form on the module path (grep); the step table exports as `16 × B` `u8` words; the parent A8 sheet is `TEST FITNESS: FIT`; all 8 blocks present.

## Checks run at the head

`loop report --check`, `loop pr-check --local`, `loop gates --full`, `prep-check`, and a fresh-context review record at the head (docs-only, D46).

## Skills used

- `uor-research-architect`: reviewed the mechanism against the geometry and integer-export constraints, not just the prose.
- `solution-integrity`: verified the pins against code and archives instead of trusting the sheet's prefixes; recorded what failed to match.

## Still open

- The item stays `ready`. To re-request FIT: the owner's bar answer with ISO-UTC time, the full DEV pin with a staging-sheet errata, and the pinned order-sensitive slice.
- The GPU run remains behind FIT, the trunk item, PM dispatch and Casey's typed D40 release; no compute was touched.

## Plain summary

- I reviewed the draft as the non-author and recorded **NOT_FIT**, clearing no compute.
- Three concrete pins are needed: the owner bar (the linked comment is a decision card, not an answer), the full DEV hashes (two conflicting pairs exist; the code pin is authoritative), and the identity of the new order-sensitive slice.
- Everything else in the mechanism checks out: 5/5 finite controls pass and the path is free of float, matmul and squashing forms.