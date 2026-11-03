# REPORT: discrete corpus K1/K4 comparison, deepseek-88

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Base: `498e36574`. Source commit:
`5b9e304d1`.

## What did not work

- **K4 did not improve over K1.** On both artifact seeds, baseline was
  38/108/361 top-1/top-8/admitted; K1 fell to 19/82/248 and K4 to 13/63/256.
  K4 therefore trailed K1 by 6 top-1 and 19 top-8 rows, lost 105 admitted
  targets, and missed the 19-row gain over baseline.
- **Both trained configurations degraded held-out ranking and admission.**
  K1 and K4 increased the panel margin-loss sums and target-rank sums rather
  than improving them.
- **The effect replicated, but as the same negative on both seeds.** The two
  public four-table artifacts produced identical panel readings under these
  arms.
- **This is not a failed K-block instrument.** K4 committed 71 moves across 46
  accepted blocks. Fourteen blocks committed multiple coordinates: six with
  two, five with three and three with four. The joint path therefore ran.
- **The public four-table artifacts do not contain Hamming, turn-memory or
  point-key records.** This says nothing about training those learned records.
- **The first six-run batch used a superseded joint-loss predicate.** It
  skipped a baseline-eligible row that lost eligibility after all proposed
  moves. The corrected scorer rejects that joint candidate, matching the
  single-coordinate rule. All six arms were rerun; the panel readings were
  unchanged.

## What changed

- `cpu-discrete-corpus-driver` now accepts `--coordinates-per-step`, defaults
  to K1 and binds K in the checkpoint identity.
- `CpuDiscreteProposalScorer::score_block` scores every coordinate in a block,
  applies all proposed moves to a cloned exact model, and computes joint loss
  over the same eligible corpus rows.
- The driver routes each block through exact `decide_bvgk`/`apply_bvgk`,
  records block-size and committed-move histograms, and advances the schedule
  with the exact move count.
- Admission loss now produces a complete `panel_gate=reject_admission` receipt
  and exits 2 only when `--require-admission` is set.

## Measurements

| Seed | Arm | Accepted blocks | Moves | Top-1 | Top-8 | Admitted | Gate | Exit |
|---:|---|---:|---:|---:|---:|---:|---|---:|
| 1 | baseline | 0 | 0 | 38 | 108 | 361 | accept | 0 |
| 1 | control-k1 | 49 | 49 | 19 | 82 | 248 | reject admission | 0 |
| 1 | pivot-k4 | 46 | 71 | 13 | 63 | 256 | reject admission | 2 |
| 2 | baseline | 0 | 0 | 38 | 108 | 361 | accept | 0 |
| 2 | control-k1 | 49 | 49 | 19 | 82 | 248 | reject admission | 0 |
| 2 | pivot-k4 | 46 | 71 | 13 | 63 | 256 | reject admission | 2 |

All six CPU/serving exports had zero query-coordinate, admitted-cell, top-8 or
token mismatches. Total wall time was 460.986 seconds; peak RSS was 20,312 KiB.
The release driver SHA-256 was
`27fc53895fc4cd81e96b446eb84c31909661e2e2eaedf6c8995260d43fe7810d`.
The comparison and all six receipts are under
[evidence](evidence/comparison.json).

## Checks run at the head

- CPU all-target suite: 133 library tests passed, 4 ignored; the discrete
  driver and all auxiliary binaries passed.
- Strict package Clippy with `--all-targets --no-deps -- -D warnings`: passed.
- Package formatting, `git diff --check` and the local PR gate: passed.
- FIT and D67 pre-run review: FIT and PASS at the committed implementation
  base, with `may_compute=true`.
- Preflight artifact/data/driver hashes: matched.
- Six-run evidence `SHA256SUMS`: passed.

## Skills used

No available Codex skill applied. The discrete schedule, proposal scorer,
joint BVG-K decision/apply, corpus driver, public artifacts and evidence were
inspected directly.

## Still open

- The exact discrete proposal family now has a bounded public K1/K4 negative;
  a different credit assignment or schedule needs new causal evidence.
- Public trained full-path Hamming/turn-memory/point-key training remains
  unavailable.
- Complete replies, fresh phrasing and GPU-qualified local chat remain outside
  this CPU result.

## Plain summary

The CPU can train the four-table discrete learner in blocks of one or four
coordinates. Four-coordinate blocks really did commit together, but both
trained settings made held-out ranking and admission worse. Four coordinates
were worse than one at the registered budget. This closes the tested discrete
K1/K4 configuration without rejecting every possible discrete training recipe.
