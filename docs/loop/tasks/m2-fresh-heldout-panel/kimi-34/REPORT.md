# REPORT: m2-fresh-heldout-panel, kimi-34 (pin the uploaded fresh256 panel)

Lab: kimi. Item: [m2-fresh-heldout-panel](../../../../roadmap/items/m2-fresh-heldout-panel.md) (#2849).

References #2849

## What did not work, first

- The panel remained `frozen: false` solely because the sealed artifacts had not been uploaded.
- No candidate was scored, and M2 training/evaluation remains HELD/NOT_RUN. This unit changes storage custody only, not the grader, bar, FIT result or item status.

## Cancel probe

All four local artifacts matched the registered sizes and hashes before upload. The panel's only remaining construction prerequisite was the maintainer upload, so the storage pin stood.

## What changed

- Uploaded `plan.json`, `inputs.json`, `labels.json`, and `manifest.json` to `caseyallard/uor-r4-store/data/m2-fresh256/` at HF commit `fb8c5e4fcd3568cc59be8be3073151295b98069c`.
- Downloaded all four to a separate readback directory; every `cmp` against the sealed originals passed.
- The panel is now `frozen: true`; storage and the lab README name the uploaded location and readback instead of staging/pending upload.

## Measurements

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `plan.json` | 122,268 | `0ed1ab98ab50bbad6c484c4e7e4ffeeb8bb7ad17d162251826302efe86d75715` |
| `inputs.json` | 524,850 | `1de2fc28607eff6bc412eee41a259d806ee1479b7c925059f00e9d3b54afa99a` |
| `labels.json` | 68,000 | `3b5c54771b8dc62e3c30aef2b30de28c09dfe79625f333854af046e6e354e2be` |
| `manifest.json` | 1,384 | `7c33f3f7e19edcb4b1a7aa5e5eab3bb0056db9351067d809f62f97b4a8880964` |

Readback: 4/4 byte-identical. Corpus bytes remain outside git. No GPU or scoring work ran.

## Checks run at the head

`loop report --check`, `loop pr-check --local`, `loop gates --full`, and `git diff --check`.

## Skills used

`uor-research-architect` for exact artifact/claim lineage; `companion-clis` for the authenticated HF upload/readback; `superpowers:verification-before-completion` for final checks.

## Still open

The pre-registered M2 evaluation sheet, owner approval and actual scoring/training remain held. No capability or milestone completion is claimed.

## Plain summary

The sealed fresh panel is now stored at a permanent location and every downloaded byte matches. The repository marks it frozen; it does not run or release M2.
