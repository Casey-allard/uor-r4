# REPORT: one-model-end-to-end, deepseek-36

Lab: deepseek. Item:
[one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md)
(#3407). Order: [ORDER.md](./ORDER.md). Run pin:
`a3ac3f326fa95ffe38159cc3c589100cea627664`, reviewed runtime
`fd303c0a72d178a467ebed99b4fefc05347e193896fc74638b2023cac0b120a1`
([launch receipt](evidence/launch.json)).

**Outcome row 5: no opening movement; the weight-8 configuration is closed.**
All four checkpoints keep the first answer target at zero top-1 and zero top-8.
The selected step-256 checkpoint improves the best observed first-target rank
from 59 to 48 but worsens the median from the matched weight-1 run's 894 to
1,025. Neither seed 2 nor a weight sweep is authorized.

## What did not work, first

- Repeating scored reply-opening rows eight times did not make the geometric
  nearest read choose the first answer token. First-target top-1 and top-8 stay
  `0/256` at every scored checkpoint.
- The selected step is 256, not 512. It has the most free EOS stops (16), while
  steps 512, 768 and 1,024 have 3, 12 and 5.
- The opening rank distribution did not improve overall. The selected
  checkpoint's minimum rank is 48, against 59 in the matched weight-1 step-512
  comparator, but its median is 1,025 against 894 and its maximum is 4,064
  against 4,041.
- No selected or later checkpoint completes an accepted row or a
  complete-distinct pair. The EOS median remains rank 2 and EOS top-1 remains
  `0/256`.
- `m2-reply512` remains unavailable, so this run supports no M2 claim.

## What changed

No model, serving, selector, outcome or experiment code changed in this unit.
This result records the completed run, applies the opening sheet's first-match
map and closes the fixed objective configuration. The change is documentation
only: one ORDER, one REPORT and one notebook row.

## Measurements

The fixed run continued `(b64L)` seed 1 for 1,024 steps on the original
OASST2-heavy reply-masked mix with `--opening-weight 8`, `--objective nearest`,
width 256, context 192, micro-batch 16, and training as served. Scorers and
selector were unchanged.

| Step | Guard/512 | First top-1 | First top-8 | First rank p50 | EOS top-1 | EOS rank p50 | Free EOS | Complete rows |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| **256 selected** | **139** | **0/256** | **0/256** | **1,025** | **0/256** | **2** | **16/256** | **0/256** |
| 512 | 137 | 0/256 | 0/256 | 1,134 | 0/256 | 2 | 3/256 | 0/256 |
| 768 | 138 | 0/256 | 0/256 | 1,069 | 0/256 | 2 | 12/256 | 0/256 |
| 1,024 | 137 | 0/256 | 0/256 | 1,086 | 0/256 | 2 | 5/256 | 0/256 |

At the selected checkpoint, first-target top-32 is also `0/256`; rank
min/p50/max is `48/1,025/4,064`. After the opening, 1,598 of 2,576 targets are
top-1 and 2,574 are top-8. The same checkpoint has 0 complete-distinct pairs of
128.

The deterministic selector chose step 256. Applying the opening sheet's
first-match map: guard is at least 93, the positive and opening-movement rows
are not met, and top-1 is zero while top-8 is below 16. This is row 5, no
opening movement. The configuration closes without seed 2 or a weight sweep.

Training took 736.945 seconds (17,074 context tokens/s at the end). The owned
pod's recorded uptime was 1,610 seconds at `$0.563/hour`, about `$0.252`
([launch receipt](evidence/launch.json)). No OOM retry occurred. Generation
used the AMD Ryzen 7 7800X3D CPU host, x86_64, 16 logical CPUs.

## Evidence

- Selected artifact:
  `eaaa3def2b3174d3f18aa50e654aeac9a53459920b4c389ab975c32fdc9ebb2f`.
- Selected model:
  `c52836de07629bdc17a24d00b7ded49442662e112e61a2eb956d368db9675502`;
  Hamming read:
  `00556d7d53321fa4ed40f6862e1066a680ee663d8c3622dd0454a1abb3b7f45e`.
- Run config:
  `e27892a166118dacfcb76528e527e7ea1b81cb6f42ab834768fb4895d65cf696`;
  initialization record:
  `2a97ca1fe65f4e1b160d50248ee0d50741ddf4bb60dc37e12b43685e45c00e49`.
- Selected continuation grade:
  `35443a3fe61a46b14d950b90d178439dc84f693ac242210d2e0632260f78d2f1`.
- Selected teacher panel:
  `9bb8bf10cf5e3c30aa2c53e92cb44b14f8b5b98fc0a613861c43637acc166ff1`;
  generated panel:
  `cdb7f49fd702791a8626b4d36926094d77418834c40cc29e425b91c14c0b590a`.
- Seal: 91 manifest entries, `BYTES=46,630,566`; `SHA256SUMS` SHA-256
  `a6d21e64f51600f64129f66e40cfeed05b110792383a2010ab2e26411d85fae9`;
  `READBACK=verified`, `EXIT=0`, `STOP_REASON=natural-or-failure`.
- Store root:
  `results/deepseek/one-model-end-to-end-reply-sft-2026-11-07/runs/s1-opening-weight-20261108t081518z-2452c146afc9f2f7/`.
- Launch/runtime receipt:
  [`evidence/launch.json`](evidence/launch.json), including the recomputed
  normalized runtime identity and the owned pod's up/release/down events.

## Checks run at the head

- Downloaded all 91 store entries and ran `sha256sum -c SHA256SUMS`: all
  entries passed.
- The serialized run config contains `"opening_weight": 8`; initialization
  records checkpoint mode with fresh optimizer and schedule.
- Corpus-panel overlap passed with zero exact replies, exact documents and
  six-gram matches over 25,071 documents and 79,679 assistant replies.
- The run exported, graded and scored steps 256, 512, 768 and 1,024 before
  completing naturally; no instrument or hash failure occurred.
- The deterministic selector ranked all four checkpoints and selected 256.
- The prep sheet records FIT and D67 approval; the run log records the pod's
  CUDA whole-versus-micro-batch weighted-gradient test passing.
- Recomputing the normalized runtime identity from run pin `a3ac3f326` gives
  `fd303c0a…`, matching the reviewed value and launch receipt.
- `git diff --check`, `loop report --check`, `loop pr-check --local` and the
  repository quick gates are run before the exact-head review.

## Skills used

No system-provided Codex skill applied. The repository's D22, D67, D84,
geometry and delivery rules were used directly.

## Still open

- The first answer token remains prompt-conditioned failure. The next
  intervention must change how the opening point is formed or selected, not
  repeat opening rows more often.
- The weight-8 configuration is closed. No seed 2, weight sweep or longer
  opening-weighted run follows.
- `m2-reply512` recovery still needs restored Runpod access or another exact
  surviving copy.
- No useful-chat, capability, mechanism, M2 or serving-parity claim follows.

## Plain summary

The model still cannot choose the first word of an answer. Making the trainer
see answer openings eight times did not change that, even though the model can
copy most later words once an opening is supplied. This particular weighting
idea is finished, and the next work must change the geometry that selects the
opening rather than train the same rows harder.
