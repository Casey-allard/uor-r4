# REPORT: cpu-training-pilot, deepseek-118 (geometric termination-point gate)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Active order: [deepseek-118](ORDER.md); preceding:
[deepseek-117](../deepseek-117/ORDER.md), [deepseek-116](../deepseek-116/ORDER.md),
[deepseek-115](../deepseek-115/ORDER.md) and
[deepseek-114](../deepseek-114/ORDER.md). Prep:
[termination gate](../../../harness/prep/cpu-training-pilot-termination-gate-2026-11-04.md).
Base: `ed5f0d6b2`.

## What did not work, first

- **The first registered attempt refused before gate evaluation.** Deepseek-114
  required training replies to end in decoder BOS id 0, but the masked corpus
  stores EOS id 1 at the end of each reply. The driver exited with
  `termination gate training reply must end in BOS` after writing training
  artifacts and before producing a gate result. This was classified as an
  instrument boundary fault, not a mechanism negative. Deepseek-115 separates
  corpus EOS from decoder BOS and retested from fresh output roots.
- **The corrected trained gate did not fire.** On both seeds all 114 evaluation
  replies still hit the 16-token cap. The trained-gate reply digest was
  byte-identical to no-gate, so no generated query entered the learned radius.
- **The matched random nonterminal gate fired immediately and incorrectly.**
  It stopped every reply at the first decision, producing 114 premature stops
  and zero generated tokens on both seeds. It established that the shared
  radius is broad enough to cover evaluation opening queries when centered on
  nonterminal training points, but those points do not identify a valid stop.
- **No arm completed an exact reference.** Full-panel and 104-row reachable
  exact counts remained zero for no-gate, trained gate and random gate on both
  seeds. A BOS stop alone was not sufficient to reproduce reply content.
- **The first exact-head review caught a future success-accounting bug.** The
  comparator would have accepted an all-premature BOS gain because it used raw
  BOS completion. Deepseek-116 changes success to non-premature BOS completion
  and adds a finite regression; it does not change the observed negative.

## What changed

The termination mechanism adds and tests:

- teacher-forced terminal queries from the final reply-content row before
  corpus EOS;
- exact removal of terminal points colliding with prompt-final or earlier
  reply-content queries;
- a deterministic L1 radius bounded by terminal neighbours and the nearest
  nonterminal training query;
- an equal-size random nonterminal-point control using the same radius;
- direct decoder-BOS naming only while a generated query is inside the gate;
- corpus-EOS reference normalization with legacy BOS compatibility;
- candidate success based only on `bos_complete - premature_stop`, with
  underflow refused and raw BOS/premature counts retained in the report;
- `premature_stop`, gate diagnostics, hashes and per-row complete-reply
  readings in `cpu-memory-panel-driver`;
- a `/2` report schema and required deepseek-110 no-gate digest pin.

No trainable float parameter, dense matrix operation, softmax or squashing
function was added. Gate lookup is exact integer L1 over training-derived Q12
points.

The code change spans three files, `+653/-22` lines against `ed5f0d6b2`,
including the regression tests. The unit also adds five ORDERS, the prep sheet,
final REPORT/evidence, one notebook row and one roadmap update. The roadmap
item does not state a size estimate.

## Measurements

The fixed selection reproduced 16 keys, 102 training examples and 114
evaluation examples. Both seeds accepted all 32 updates and reproduced the
deepseek-109 trained artifact hashes. Every no-gate digest matched deepseek-110.

After removing one EOS per reply, the evaluation references hold **1,212**
content tokens, maximum **35**; **104/114** fit the 16-token cap and context.

| Seed | Arm | BOS complete | Premature stop | Exact | Reachable exact | Cap | Tokens |
|---:|---|---:|---:|---:|---:|---:|---:|
| 1 | no gate | 0 | 0 | 0 | 0 | 114 | 1,824 |
| 1 | trained gate | 0 | 0 | 0 | 0 | 114 | 1,824 |
| 1 | random gate | 114 | 114 | 0 | 0 | 0 | 0 |
| 2 | no gate | 0 | 0 | 0 | 0 | 114 | 1,824 |
| 2 | trained gate | 0 | 0 | 0 | 0 | 114 | 1,824 |
| 2 | random gate | 114 | 114 | 0 | 0 | 0 | 0 |

Gate derivation retained all 102 terminal points on both seeds; no exact
terminal/nonterminal collisions were found.

| Seed | Radius | Max terminal-neighbour distance | Nearest nonterminal distance | Trained gate SHA-256 | Random gate SHA-256 |
|---:|---:|---:|---:|---|---|
| 1 | 98,313 | 1,066,378 | 98,314 | `cdc33f7a…587e` | `23e12e40…759c` |
| 2 | 220,730 | 1,166,080 | 220,731 | `7ef71015…c500` | `97c523ef…ca17` |

Both served-query audits retained zero mismatches over **4,333 decisions,
1,109,248 Q12 coordinates and 34,664 kept cells** per seed. Walls were
**41.089/40.903 s** and peak RSS was **389,892/391,248 KiB**. The fresh seed-1
repeat matched the full report after removing timing and RSS fields, with
normalized report SHA-256
`4fb05455ce88ab4ff009de793729f2df6fc6b545e3347c0e1f5a650bef934de4`.

The detailed record is [comparison.json](evidence/comparison.json);
[SHA256SUMS](evidence/SHA256SUMS) binds the release binary, raw reports and
generated artifacts. The refused deepseek-114 artifacts remain recoverable at
`local/deepseek/termination-gate/deepseek-114/refused-seed-1-eos-boundary`.

## Checks run at the head

- Focused termination tests cover EOS endpoint derivation, collision removal,
  radius bounds, random construction, exact/inside/outside override, the real
  BOS reply loop, premature-stop accounting, EOS/BOS normalization and
  no-gate digest identity.
- Full CPU release suite: **182 passed**, 4 release benchmarks ignored; every
  binary suite passed, including five driver tests.
- Strict all-target CPU Clippy with `-D warnings`: passed.
- `cargo fmt -p uor-r4-cpu-train -- --check`: passed.
- `git diff --check`: passed.
- Model-free selection reproduced the fixed 16/102/114 split and corrected
  1,212-token census.
- The EOS repair carries fresh non-author FIT at
  `951ce09c701fd4783ea4f0b8c0de96e6becd387a`; the non-premature accounting
  repair carries fresh FIT at `8af603b074bed0dc1f0485f0512fb91d3837fd99`.
- Both public seeds and the fresh seed-1 repeat ran under the FIT command.
- `uor-harness check`: **0 errors**; all remaining findings were pre-existing
  coordination/state warnings.
- `loop report --check --order .../deepseek-118/ORDER.md`: passed.
- `cargo run -p xtask -- roadmap check`: 139 items OK.
- `cargo run -p xtask -- docs-catalog --check`: 1,175 records, zero missing
  paths.
- `cargo run -p xtask -- archive-verify --base origin/main`: zero modified or
  deleted archive files.
- `uor-harness shard-check`: 2,678 receipts, none lost.
- D23, D25 and D32 ratchets: no added sites.

## Skills used

No system-provided skill applied. The repository's D84, FIT, ORDER, REPORT and
geometry rules governed the work. Fresh-context subagents performed an
implementation audit and non-author FIT; neither was run as compute.

## Reading

The registered point-gate candidate is **negative** on both seeds. A nearest
training-terminal radius did not recognize any evaluation continuation as a
stop. The nonterminal control shows that a broad nearest-point rule can fire,
but without selecting the correct terminal state; its all-premature result
does not count as progress under the repaired comparison.

This closes the fixed 102-point, single-radius, L1-gate configuration under the
registered split and cap. It does not reject geometric termination. The next
termination attempt must change the representation or decision rule, such as a
cell/orbit rule or a confidence comparison against nonterminal density, rather
than widen this post hoc.

## Still open

- A termination rule that both fires and preserves generated content.
- Continuation quality once a reply can stop.
- Fresh-phrasing validation and GPU qualification.
- A useful local chat model; this unit remains development research.

## Plain summary

The gate was built correctly after separating the corpus's end token from the
decoder's stop marker. Both model seeds still ignored every learned terminal
point and filled the 16-token cap. A random set of ordinary nonterminal points
stopped every reply immediately but matched no reference. The exact-point
termination radius therefore fails this test, and widening it after seeing the
result would be tuning rather than evidence.
