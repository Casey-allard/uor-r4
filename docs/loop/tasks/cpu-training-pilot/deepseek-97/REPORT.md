# REPORT: final served-gain distance training, deepseek-97

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Sheet:
[cpu-training-pilot-gain-2026-11-03.md](../../../../harness/prep/cpu-training-pilot-gain-2026-11-03.md).
Base: `c4c267c534d7160ac6d544c420c90249a1cddeba`.

## What did not work

- **The registered ranking bar failed on both seeds.** Eight accepted final-gain
  updates lowered held-out target distance by 16,799,488 on seed 1 and
  16,154,486 on seed 2, and the trained arm beat both frozen and random
  distance. Top-1 stayed at 291 and 243 respectively: zero discordant rows,
  zero paired delta and no three-sigma ranking gain. This is a configuration
  negative, not a language result.
- **The first candidate implementation was not fit for compute.** Adversarial
  pre-run review found incomplete parity aggregation, reusable/unsealed report
  roots and a registered memory-order mismatch. The implementation now checks
  every parity counter, creates each attempt root exclusively before model
  work, seals and verifies both success and ordinary failure attempts, and
  documents memory movement in the normalization-fine domain.
- **The first exact-head review returned CHANGES.** The ORDER omitted its
  charter-required cancel probe, and the provisional line-count statement was
  stale. The base probe now records the earlier rank-margin normalization
  configuration and why this complete-distance objective is distinct; the size
  statement below is recomputed from a named, stable file set.
- **The repeated keys are not unseen-key evidence.** Evaluation documents are
  disjoint from training documents but intentionally reuse their opening keys.
  Nothing here establishes transfer to a new key, complete replies or fresh
  phrasing.
- **The public source artifacts have no Hamming readout.** The gain trainer
  therefore exercises the no-Hamming branch and fails closed when that readout
  is present; a readout-dependent gain path remains open.

## What changed

The CPU FBS trace now exposes the exact normalization immediately before
`final_norm_gain`. The cached trainer keeps that normalization, the fixed
teacher-forced decision boundary and the fixed turn-memory move. Each candidate
replays:

1. the candidate final gain;
2. fixed Q24 opening/reply memory movement in the normalization-fine domain;
3. Q12 conversion;
4. the complete writer decision;
5. query-dependent point fallback when an opening key misses.

The full-forward trainer remains available behind `--reference`. The registered
driver enforces eight memory steps, eight gain steps, eight keys, 58 training
samples and 56 evaluation samples; derives the deepseek-95 memory baseline;
compares frozen, trained, equal-move random, memory-off and trained-memory-off
arms; and emits the registered distance/top-1/ranking verdict.

All attempt outputs use exclusive creation. The complete attempt, including
ordinary failures, is sealed with a BLAKE3 manifest and verified before the
driver returns.

The implementation and registration diff from the base, excluding this REPORT
and generated run evidence, is 1,916 added and 46 removed lines across the CPU
crate, registered sheet, ORDER and preserved pre-fit sheet. This is above the
initial 600-line estimate because the out-of-sample gain path, driver, four
mechanism controls, fail-closed evidence handling and adversarial coverage are
in one bounded unit.

## Measurements

Both runs used 16 Rayon workers on the Ryzen 9 9950X's 16 physical cores.

| Seed | Arm | Top-1 / 2,431 | Target distance | Wall | Peak RSS |
|---:|---|---:|---:|---:|---:|
| 1 | frozen | 291 | 2,820,387,931 | — | — |
| 1 | trained | 291 | **2,803,588,443** | 19.613 s | 261,012 KiB |
| 1 | random | 291 | 2,817,628,372 | — | — |
| 1 | memory-off | 278 | 2,821,353,534 | — | — |
| 2 | frozen | 243 | 2,896,327,331 | — | — |
| 2 | trained | 243 | **2,880,172,845** | 19.656 s | 267,212 KiB |
| 2 | random | 243 | 2,895,812,280 | — | — |
| 2 | memory-off | 238 | 2,898,309,913 | — | — |

`/usr/bin/time` recorded 253.11 user + 0.74 system seconds for seed 1
(1,293% CPU, 12.94 effective cores) and 255.27 + 0.91 seconds for seed 2
(1,302% CPU, 13.03 effective cores). Combined wall was 39.28 s across 510.03
CPU-seconds, an average 12.99 effective cores and 81.2% of the physical cores.
Candidate evaluation inside each update is parallel; the eight updates remain
an ordered sequence.

All eight updates were accepted on both seeds. Seed 1 changed final-gain
coordinates `233, 161, 89, 89, 155, 219, 163, 171`; seed 2 changed
`154, 154, 56, 56, 124, 172, 9, 9`. Every trained artifact begins from the
pinned deepseek-95 memory baseline.

## Exact parity

Each accepted training step checked 58 samples, 2,596 positions, 664,576 Q12
coordinates, 2,596 token decisions and 20,768 kept cells. Each of the five
held-out arms on each seed checked 56 samples, 2,487 positions, 636,672 Q12
coordinates, 2,487 token decisions and 19,896 kept cells. Every CPU/serving
query-coordinate, token and kept-cell mismatch count was zero.

## Artifacts

The sealed tracked reports and timing are under [evidence](evidence/). The
worked artifacts are retained outside the source tree at:

- `/home/callard/projects/codex-playground/uor-r4/local/deepseek/gain-panel/deepseek-97/seed-1`;
- `/home/callard/projects/codex-playground/uor-r4/local/deepseek/gain-panel/deepseek-97/seed-2`.

Trained artifact hashes are:

- seed 1:
  `085e4e75f5b89f279a253b6da0177414042be910dd2523a1cfd726920054a7b2`;
- seed 2:
  `c5b52410915a3ca8708470194a32e661b96b6b0499438788fe2c398ab76ff85e`.

## Reading

The final-gain distance mechanism is active on this bounded panel: all eight
distance improvements replicated, the memory controls separated, cached and
full-forward training remained equivalent in finite tests, and every arm kept
exact integer serving parity. The capability test is still negative because
the distance gains did not cross the top-1 ranking bar.

This is materially new relative to the failed rank-margin runs and the
deepseek-95 memory-weight comparison: it trains a previously untrained
full-width final gain through the exported serving operation. It does not
establish that optimizing query distance is sufficient for useful next-token
ranking.

## Checks run at the head

- Full `uor-r4-cpu-train` release suite: **157 passed, 4 ignored**.
- Driver suites: **6, 1 and 5 passed** for the CPU corpus,
  counterfactual-corpus and discrete-corpus drivers.
- Focused gain tests: **5 passed**. Checked aggregation tests: **2 passed**.
- Strict package Clippy with `--all-targets --no-deps -- -D warnings`: passed;
  dependency crates emitted pre-existing dead-code warnings.
- Package formatting and `git diff --check`: passed.
- Both registered runs exited 0, were sealed by the driver, and had 16 reported
  Rayon threads. One known duplicate `uor-r4-integer` Cargo artifact collision
  occurred on the first build attempt and the immediate locked retry passed.
- `uor-harness loop gates --full`: **11/11 GREEN**.
- `uor-harness loop report --check --order …/deepseek-97/ORDER.md`: **PASS**.
- `uor-harness check --only prep`: **0 errors** (21 pre-existing warnings).
- `uor-harness loop pr-check --local`: **PASS**, with advisory D76 warnings for
  the claims-file scope, the repaired ORDER metadata line and pre-existing
  FILEMAP gaps.
- `xtask roadmap check`: **137 items OK**. `xtask docs-catalog --check`: **1,175
  records, 0 paths not found**. A changed-source link scan found no broken link
  in this unit after the REPORT sheet path was corrected.
- Evidence `SHA256SUMS`: all 12 tracked evidence files passed.

## Skills used

No available Codex skill applied. The FBS replay, artifact layout, memory
movement, objective, controls and serving audit were inspected and tested
directly.

## Still open

- A distance objective that moves top-1 enough to clear the registered ranking
  bar.
- Transfer to unseen keys rather than repeated exact keys.
- Hamming-readout-dependent final-gain training.
- Complete replies, fresh phrasing and useful local conversation.
- GPU qualification of this final-gain family.

## Plain summary

The trainer can now change the final numeric gain of the exported model and
re-score the complete served query without rebuilding the recurrent network.
On both model seeds it reliably made the held-out next tokens numerically
closer to their correct writer rows, and every optimized arm still matched the
integer serving path exactly. It did not change which token ranked first, so
the predeclared capability bar failed and this one-code gain configuration is
closed.
