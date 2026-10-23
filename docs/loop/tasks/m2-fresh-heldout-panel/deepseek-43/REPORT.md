# REPORT: m2-fresh-heldout-panel, deepseek-43 (non-author FIT review of `m2-fresh256`)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #2849 (`References`; the item stays `ready` until the seal upload). Base: `a4a5e1efb`.

## What did not work, first

- **Nothing failed the reproduction.** The instrument rebuilt the four sealed artifacts byte-for-byte and its own audit reproduced every registered number, so there was no instrument defect to report. That is the point of the unit: the FIT is a check, not a formality, and the check passed.
- **The panel does not become usable from this review.** The lab record lists two prerequisites; this review clears only the first. The maintainer's HF upload is still open, so the panel JSON keeps `"frozen": false`; the note now says `TEST FITNESS: FIT` rather than PENDING, with the upload as the only remaining condition.
- **No bar, row, seed, grader or digest was changed.** The owner-registered bar (`pairs_both_complete_distinct >= 52/128`, complete >= 104/256, #2030 comment 6350135678) and the JSON's `current: 0` / `NOT_RUN` are untouched; a FIT releases no hold.

## What changed

| File | Change |
|---|---|
| `docs/labs/m2-fresh-heldout-panel/README.md` | new `### Non-author FIT review (deepseek-43, 2026-10-23)` with the reproduction commands, all four sizes/hashes, the audit numbers, and `TEST FITNESS: FIT` (prerequisite 1 only) |
| `docs/harness/panels/m2-fresh256.json` | the panel note now says `TEST FITNESS: FIT` (non-author deepseek-43) and keeps `frozen: false` only for the HF upload; no field values changed |
| this folder | ORDER (committed first) and this REPORT |

**Size:** about 40 docs lines; docs-only (no receipts file, D46 fresh-context record used).

## Measurements

- `docs/labs/m2-fresh-heldout-panel/instrument`: `cargo test --locked` **5 passed**; `cargo run --locked -- --emit-dir /tmp/m2-fresh256-check` reproduced: plan 122,268 B `0ed1ab98…`, inputs 524,850 B `1de2fc28…`, labels 68,000 B `3b5c5477…`, manifest 1,384 B `7c33f3f7…` - exactly the registered panel JSON.
- Audit from the same run: 74 historical literals; 0 exact/content-word/2-gram overlaps; 0 intra-panel duplicates; min edit distance 10; 128 histories / 256 rows / 16 blocks / 32 ordered banks; role and initial-position balance true; unscored-input leak count 0; 0 hits on each of the four question-blind/role-swap controls.

## Checks run at the head

`loop report --check`, `loop pr-check --local`, `loop gates --full`; a fresh-context review record is requested at the current head and recorded after it is posted (docs-only, D46).

## Skills used

- `solution-integrity`: re-derived every sealed digest and audit number by running the instrument rather than reading the registration; checked the leak count and control scores specifically because those are the failure modes a fresh panel can hide.
- `scope-guard`: docs-only in the item's scope; no bar, digest, row or hold changed; the HF upload was not attempted.

## Still open

- Prerequisite 2: a maintainer with HF write access uploads the four sealed files to `hf://datasets/caseyallard/uor-r4-store/data/m2-fresh256/`; after that the panel can be `frozen: true`.
- The item stays `ready` until that upload; M2 training/evaluation remain HELD and NOT_RUN.

## Plain summary

- Reproduced `m2-fresh256` from its own instrument: all four seals match and the overlap/control audit is clean, so I recorded the non-author `TEST FITNESS: FIT` for the construction half.
- The panel still is not frozen and no candidate has been scored: the HF upload by a maintainer is the remaining step, and all holds stay.