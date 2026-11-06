# REPORT: cpu-training-pilot, deepseek-221

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[nearest-rival sheet](../../../../harness/prep/cpu-training-pilot-nearest-rival-gain-2026-11-06.md).
Base: `4fde31439`.

## Status

Complete as **CONTROL_FAULT**. The first scored artifact stopped the registered
run before seed 2 or the repeat.

## What did not work

- The shuffled-target control was behaviorally equivalent to the true-target
  arm. A cyclic shift changed **2,529 / 2,538** target labels, but both trainers
  selected the same eight moves and produced byte-identical artifacts
  (`d658d37c...`).
- Held-out top-1 stayed **291** for frozen, true-target, random and
  shuffled-target.
- True-target and shuffled-target held-out multi-rival hinge both ended at
  **519,784,273**, below frozen **521,955,557** and random **522,006,729**, but
  the matched arm does not separate label-dependent learning from a
  label-insensitive query change.
- Seed 2 and the fresh-process repeat were `NOT_RUN` under the pre-registered
  control-fault stop.
- Independent review remained unavailable under the current delegation policy.
  FIT, GO and review are local exact-head self-reviews.

## What changed

Added an exact eight-rival final-gain trainer and a dedicated driver. The
trainer refreshes rival identities after each step, evaluates every gain
coordinate at both signs through the exact CPU FBS query, and uses a cyclic
target-label shuffle that preserves the target multiset. The driver starts
from deepseek-97's frozen memory baselines and includes frozen, random,
shuffled-target and memory-off controls.

## Measurements

Seed 1:

- changed shuffle targets: **2,529 / 2,538**;
- true and shuffled accepted moves: eight `-1` moves at coordinates
  `18,18,18,72,63,18,18,18`;
- trained and shuffled artifact:
  `d658d37ca03d343ceb5c4b6b08ba24bfe88fd2f283d839538006cc09c4f8d049`;
- frozen arm:
  `3ad89c7b80df52f9e1e4d878c072982e9fb613b8453ded0dd50c3ee905e0ee44`;
- random arm:
  `5703d18b08709ee2107cfc56cc0bbca5a880a8d1aed6ce723668ac82a4956d6c`.

| Arm | Top-1 / 2,431 | Multi-rival hinge | Target distance |
|---|---:|---:|---:|
| frozen | 291 | 521,955,557 | 2,820,387,931 |
| true-target | 291 | 519,784,273 | 2,818,510,034 |
| random | 291 | 522,006,729 | 2,817,628,372 |
| shuffled-target | 291 | 519,784,273 | 2,818,510,034 |

Every held-out arm passed exact CPU/serving parity over 636,672 query
coordinates, 2,487 token decisions and 19,896 kept cells.

Resources:

- 24 Rayon workers;
- wall **23.72 s** at **1,916%** CPU;
- peak RSS **266,704 KiB**;
- retained evidence **6,638,505 bytes**;
- local `SHA256SUMS`:
  `fabfea413d03a7bdf22963c54c53fa4b8b0f81632583f6c244039c53f0e9a686`.

Compact evidence: [result.json](evidence/result.json).

## Interpretation

The model implementation is not blamed. The shuffled-label control failed its
behavioral non-vacuity check: changing almost every target produced the same
optimization path. This closes the fixed cyclic-target shuffle as an
informative control for this final-gain objective. It does not reject the
nearest-rival objective or final-gain training.

The first run's driver reported `REJECT` because it checked only the quality
bar. A post-run instrument repair now checks artifact-distinctness first and
returns `CONTROL_FAULT`; two driver tests pin that precedence. No second scored
process was run after the repair.

## Checks run at the head

- nearest-rival library tests: 3 passed;
- driver verdict tests: 2 passed;
- full `uor-r4-cpu-train` library: 402 passed, 4 expected ignores;
- scoped Clippy passed for the changed binary with the pre-existing
  `type_complexity` lint exempted;
- changed trainer and driver formatting plus `git diff --check` passed;
- source, driver, baseline, corpus and output-root absence were verified
  before GO.
- seed-1 true and shuffled accepted-move lists compare equal;
- seed-1 true and shuffled artifacts have the same SHA-256;
- sealed 11-file local evidence checksum verified;
- D23, D25 and D32 ratchets unchanged, claim wording passed, prep-check reports
  FIT with compute allowed, and local PR policy passed;
- `loop gates --full`: 11/11 green;
- full harness check: 283 rules, 0 errors and 249 repository-wide advisory
  warnings;
- `xtask docs-catalog --check`: 1,175 records, 20 lines, zero missing paths;
- `xtask links`: 1,277 pre-existing repository-wide broken links and none among
  the newly added deepseek-221 evidence links;
- `xtask roadmap check` remains blocked by the pre-existing
  `infra-desktop-client-tauri` status/issue mismatch, which this diff does not
  touch;
- implementation review: local exact-head self-review at `2ba8b56b6`.

## Skills used

No system-provided Codex skill applied.

## Still open

- a matched target shuffle whose behavior differs from the true target;
- a model-derived stop that removes reliance on retrieved length;
- Claude's independent end-to-end serving track.

## Plain summary

The nearest-rival trainer ran and was active, but its shuffled-target control
collapsed to the same eight moves and artifact as training on the real targets.
Because the control could not distinguish label-dependent learning, the result
is a control fault rather than a model negative. Seed 2 and the repeat were not
run under the fixed stop rule.
