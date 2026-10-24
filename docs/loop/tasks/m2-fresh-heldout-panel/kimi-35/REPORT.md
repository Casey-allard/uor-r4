# REPORT: m2-fresh-heldout-panel, kimi-35 (repair the fresh256 storage pin)

Lab: kimi. Item: [m2-fresh-heldout-panel](../../../../roadmap/items/m2-fresh-heldout-panel.md) (#2849). Repair for PR #3147.

References #2849

## What did not work, first

- Independent review at `b71d09f53` returned CHANGES: the storage row abbreviated hashes, native-geometric still said staged/PENDING, and the initial REPORT listed checks without results.
- `docs/roadmap/items/m2-fresh-heldout-panel.md` also retains the pre-upload status. It is outside this ORDER and remains a separately scoped PM follow-up; this PR does not claim that page is repaired or change item status.
- No candidate was scored, and M2 training/evaluation remains HELD/NOT_RUN.

## What changed

- All four full SHA-256 values now appear in the storage row, alongside the HF commit and readback.
- Native-geometric records the frozen panel, construction FIT and upload; the older lab notebook row is explicitly a dated pre-upload snapshot.
- This REPORT records the executed readback and exact-head check results.

## Measurements

Upload: HF commit `fb8c5e4fcd3568cc59be8be3073151295b98069c`.

Readback command and result:

```text
for f in inputs.json labels.json manifest.json plan.json; do
  cmp "local/antigravity/m2-fresh256-sealed/$f" \
      "local/kimi/m2-fresh256-readback/data/m2-fresh256/$f" || exit 1
done
# 4/4 byte-identical; exit 0
```

| Artifact | Bytes | SHA-256 |
|---|---:|---|
| `plan.json` | 122,268 | `0ed1ab98ab50bbad6c484c4e7e4ffeeb8bb7ad17d162251826302efe86d75715` |
| `inputs.json` | 524,850 | `1de2fc28607eff6bc412eee41a259d806ee1479b7c925059f00e9d3b54afa99a` |
| `labels.json` | 68,000 | `3b5c54771b8dc62e3c30aef2b30de28c09dfe79625f333854af046e6e354e2be` |
| `manifest.json` | 1,384 | `7c33f3f7e19edcb4b1a7aa5e5eab3bb0056db9351067d809f62f97b4a8880964` |

No corpus bytes entered git. No GPU or scoring work ran.

## Checks run at the head

- `loop report --check`: PASS.
- `loop pr-check --local`: PASS, including scope and repair ORDER.
- `loop gates --full`: 11/11 GREEN.
- `git diff --check`: PASS.

Output is retained under `local/kimi-m2-fresh256/`.

## Skills used

`uor-research-architect`, `companion-clis`, `superpowers:requesting-code-review`, and `superpowers:verification-before-completion` for artifact custody, independent review and final evidence.

## Still open

The roadmap item's pre-upload text needs its separately scoped PM follow-up. The pre-registered M2 evaluation sheet, owner approval and actual scoring/training remain held. No capability or milestone completion is claimed.

## Plain summary

The panel was already uploaded, but some records still said pending and the storage hashes were abbreviated. This repair completes the evidence and current status without releasing M2.
