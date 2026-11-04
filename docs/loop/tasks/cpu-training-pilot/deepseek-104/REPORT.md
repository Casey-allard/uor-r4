# REPORT: cpu-training-pilot, deepseek-104

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Sheet:
[cpu-training-pilot-point-transfer-2026-11-03.md](../../../harness/prep/cpu-training-pilot-point-transfer-2026-11-03.md).
Base: `2789caa90`.

## What did not work

- **The pre-registered unseen-key transfer bar failed on seed 2.** The
  training-derived point table beat point-off on both seeds, by 2 and 1 opening
  rows of 60. But its shuffled-target control scored 3 on seed 2 against the
  point table's 1, so the required strict advantage over both controls did not
  hold on both seeds.
- **The first order required an impossible distance improvement.** Point
  fallback changes only the served opening token after the final query is
  computed. Point-off, point and random therefore have identical opening
  distance. The fresh FIT review caught this before compute; the order and sheet
  now require distance equality as an instrument invariant.
- **The first random-control test was vacuous.** It proved only that targets
  formed a permutation. The final control constructs the served artifact,
  proves that a shuffled target changes the opening, rotates once if
  Fisher-Yates is unchanged and refuses an all-equal table.
- **The first two public runs used the wrong point producer.** A custom training
  query helper marked reply start before capture. A fresh exact-head review
  found that production `build_point_key` steps the training window without a
  reply-start signal, so the first runs were invalid for production-point
  evidence. The driver now calls `build_point_key` directly. The invalid runs
  are retained; the corrected rerun produced identical metrics and artifact
  hashes, as expected from the zero memory move in this configuration.
- **The required negative-path tests were initially incomplete.** A second FIT
  review caught that the finite unseen-key control and refusal cases were not
  all present. The final focused suite covers unseen exact-key miss, point
  fallback, changed random target, overlap refusal, malformed rows and
  deterministic selection.
- **The normal-approximation three-sigma threshold is degenerate at a zero
  baseline.** Point-off scored 0 of 60 on both seeds, so the pre-expressed
  formula gives threshold 0. The computed values are retained, but no
  inferential ranking claim is made from that threshold.
- **A shared release target can mix incompatible feature sets.** The known
  duplicate `uor-r4-integer` artifact collision occurred when a test and the
  production binary were built in one target directory. Fresh release targets
  passed; no product failure is claimed for the reuse.
- **The first two focused targets hit the same mixed-feature collision.** A
  fresh target passed; Clippy ran in its own fresh target.

## What changed

Added `cpu-point-transfer-driver`. It:

1. validates the pinned artifact, corpus, mask and manifest;
2. selects 8 training and 8 evaluation opening-key groups with zero exact-key
   overlap;
3. keeps 60 document-disjoint examples on each side;
4. derives opening/reply memory only from training examples;
5. asserts that every held-out opening key misses the exact memory table;
6. steps each training prompt through the exact CPU FBS path and derives 48
   distinct point rows with first-reply-token targets;
7. constructs point-off, point and non-vacuous shuffled-target artifacts;
8. scores opening and full teacher-forced decisions on the complete served
   path; and
9. runs exact CPU/integer served-forward audits for every arm.

The focused binary tests cover deterministic disjoint selection, the finite
unseen-key point fallback, a changed shuffled-target opening, exact-memory-hit
refusal, overlap refusal and malformed point rows.

## Measurements

| Seed | Arm | Opening top-1 / 60 | Opening distance | Full top-1 / 2,050 | Wall |
|---:|---|---:|---:|---:|---:|
| 1 | point-off | 0 | 64,821,741 | 169 | — |
| 1 | point | **2** | 64,821,741 | **171** | 1.049 s |
| 1 | random | 0 | 64,821,741 | 169 | — |
| 2 | point-off | 0 | 80,445,779 | 163 | — |
| 2 | point | 1 | 80,445,779 | 164 | 1.044 s |
| 2 | random | **3** | 80,445,779 | **166** | — |

Seed 1 point improved both opening and complete teacher-forced top-1 over both
controls. Seed 2 improved point-off, but its shuffled targets named more
correct openings and full-sequence tokens. The pre-registered primary bar was
negative.

Every arm on both seeds checked:

- 60 samples;
- 2,110 positions;
- 540,160 Q12 query coordinates;
- 2,110 token decisions;
- 16,880 kept cells; and
- zero query-coordinate, token or kept-cell mismatches.

The complete reports and local artifact hashes are in
[comparison.json](evidence/comparison.json). Peak RSS was about 290 MiB and each
seed used roughly 10 effective CPU cores while evaluating independent samples.
The evidence record preserves both the invalid first roots and the corrected
production-builder roots.

## Reading

This is the first bounded public **unseen-exact-opening-key** transfer result.
It is not a universal failure of point keys. On seed 1 the training-derived
points beat both controls; on seed 2 the random-target control was better than
the learned table. Both effects are only a few openings out of 60, so the
result is mixed and underpowered, not capability evidence.

The exact served path remained stable: point fallback changed only the token,
and all three arms retained identical opening distance and zero parity
mismatches.

## Checks run at the head

- Focused `cpu-point-transfer-driver` release suite: 5 passed.
- Both registered public runs exited 0 and produced local reports plus three
  artifacts per seed. These output roots are retained as run evidence; this
  driver does not claim the report roots are sealed.
- Every CPU/serving parity counter was zero.
- Full `uor-r4-cpu-train` release suite: 161 library tests passed with 4
  release benchmarks ignored; the corpus, counterfactual and discrete driver
  suites passed 6/1/5, and the new point-transfer binary passed 5.
- Strict package Clippy with `--all-targets --no-deps -- -D warnings`: passed.
- `cargo fmt -p uor-r4-cpu-train --check`, `git diff --check`, evidence
  checksums, loop report validation and `xtask roadmap check` with 139 items:
  passed.
- `uor-harness loop gates --full`: all 11 gates passed.
- `uor-harness loop pr-check --local`: PASS with only the existing FILEMAP
  advisory.

## Skills used

No available Codex skill applied. The existing CPU corpus, memory, point-key
and served-forward interfaces were inspected and tested directly.

## Still open

- A point-table representation that suppresses the noisy nearest labels seen
  on seed 2 and beats matched random targets on both seeds.
- Complete replies and fresh reply wording; this unit scored only opening and
  teacher-forced next-token decisions.
- A positive capability configuration and credentialed sealed evaluation.
- GPU qualification of this transfer family.

## Plain summary

Training-only prompt points named unseen-key openings better than no point
table on both public model seeds, but shuffled targets beat the learned table
on one seed. Exact serving parity held throughout. The bounded unseen-key test
is therefore mixed and negative against its registered bar, not evidence of
useful conversation.
