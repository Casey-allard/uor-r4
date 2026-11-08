# REPORT: one-model-end-to-end, deepseek-37

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Result:
[deepseek-36 REPORT](../deepseek-36/REPORT.md).

**The opening-weight result is unchanged: outcome row 5, no opening movement,
and the weight-8 configuration is closed.** This repair adds source-bound
launch evidence and corrects one median label after the first exact-head review.

## What did not work, first

- The first fresh review at `8ab214b87` found the reviewed runtime hash and pod
  uptime/rate were not present in the sealed model bundle.
- It also found that “EOS remains rank 2” needed to say median: raw EOS ranks
  at the selected step are min/p50/max `2/2/3`.
- The review found no measurement, selector, outcome-map or hash discrepancy
  and requested no model rerun.

## What changed

Added [launch.json](../deepseek-36/evidence/launch.json), which records the
runtime-identity derivation, its independently recomputed hash, and the owned
pod's up/release/down ledger events. The result REPORT now cites this receipt
and labels EOS rank 2 as the median. Training artifacts and the result
classification did not change.

The change is documentation and task evidence only: repaired REPORT and new
receipt, with no code, model, selector or experiment change.

## Measurements

All four checkpoint readings remain as recorded in the
[deepseek-36 REPORT](../deepseek-36/REPORT.md):

| Step | Guard/512 | First top-1/top-8 | First rank p50 | EOS median | Free EOS | Complete |
|---:|---:|---:|---:|---:|---:|---:|
| **256 selected** | **139** | **0/0** | **1,025** | **2** | **16** | **0** |
| 512 | 137 | 0/0 | 1,134 | 2 | 3 | 0 |
| 768 | 138 | 0/0 | 1,069 | 2 | 12 | 0 |
| 1,024 | 137 | 0/0 | 1,086 | 2 | 5 | 0 |

The selected checkpoint maps to outcome row 5. Its top-32 count is also zero;
training took 736.945 seconds. The launch receipt records 1,610 pod-seconds at
`$0.563/hour`, or about `$0.252` by rate times logged uptime, and deletion after
verified readback.

## Evidence

- Runtime identity:
  `fd303c0a72d178a467ebed99b4fefc05347e193896fc74638b2023cac0b120a1`,
  recomputed from run pin `a3ac3f326` by the registered launcher derivation.
- Owned pod: `vast-59686054`, up `08:15:25Z`, released `08:42:10Z`, deleted
  `08:42:19Z`.
- Selected artifact:
  `eaaa3def2b3174d3f18aa50e654aeac9a53459920b4c389ab975c32fdc9ebb2f`.
- Seal: 91 SHA-256 entries, `BYTES=46,630,566`, `SHA256SUMS` SHA-256
  `a6d21e64f51600f64129f66e40cfeed05b110792383a2010ab2e26411d85fae9`,
  `READBACK=verified`, `EXIT=0`.
- Full store root and every selected artifact/config/init/panel/grade hash are
  recorded in the [result REPORT](../deepseek-36/REPORT.md).

## Checks run at the head

- Recomputed the launcher's normalized runtime hash from the exact run pin; it
  matches the reviewed value.
- Rechecked the pod ledger's up, release and down events against
  [launch.json](../deepseek-36/evidence/launch.json).
- Cross-checked all four checkpoint readings against raw grades and panels.
- Downloaded all 91 sealed entries and ran `sha256sum -c --status --strict`;
  every entry passed.
- `git diff --check`, `loop report --check`, `loop pr-check --local` and the
  full repository gates run before the repaired exact-head review.

## Skills used

No system-provided Codex skill applied. The repository's D22, D67, D84,
geometry and delivery rules were used directly.

## Still open

- The weight-8 configuration remains closed. No seed 2, sweep or longer run
  follows.
- The first answer token still needs a geometry change to its formation or
  selection rather than repeated opening rows.
- `m2-reply512` remains unavailable; no M2, useful-chat, mechanism or
  capability claim follows.

## Plain summary

The run's result did not change: weighting answer openings more heavily still
failed to make the first word predictable. The repair only proves where the
launch identity and pod cost came from and makes the EOS wording exact.
