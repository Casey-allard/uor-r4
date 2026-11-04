# REPORT: reuse loaded models for CPU evaluation, deepseek-129

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[evaluation reuse](../../../harness/prep/cpu-training-pilot-trunk-eval-reuse-2026-11-04.md).
Base: `6824d2d58`.

## What did not work, first

- **The reuse does not remove the work that produces the unchanged artifact.**
  The training-row CPU reader is still needed for exact export, and the integer
  model is still needed for the no-update CPU/serving parity check. This change
  removes their duplicate reconstruction for evaluation; it does not remove
  their original construction.
- **The retained integer model raises memory while the corpus, reader and
  training rows are live.** Peak RSS rose from the baseline range to
  1,624,544 KiB in the optimized runs, still 472,608 KiB below the registered
  stop.
- **No model-math or capability question was tested.** Exact, no-update and
  shuffled artifacts, readings and checksums had to remain identical.

## What changed

The driver already constructed an integer model to validate the input and a
CPU trunk reader to derive teacher-forced rows. It then discarded those models
and loaded both again to evaluate the unchanged arm.

The new path:

1. retains the input-validation integer model;
2. derives training rows with the CPU trunk reader;
3. evaluates the held-out no-update arm with the already-loaded CPU and
   integer models;
4. drops the retained integer model; and
5. consumes the CPU reader to export the exact modified artifact.

The exact and shuffled arms still reload both their artifact bytes and both
model implementations independently. Their CPU/serving parity checks are
unchanged.

The finite test `preloaded_evaluation_matches_independent_reload` builds a real
D11 artifact through the production exporter and proves that the preloaded and
freshly reloaded paths produce the same `DecisionReading`.

## Measurements

The fixed split held 200 training documents / 7,080 targets and 193 evaluation
documents / 512 targets. Eight fresh roots ran the registered mirrored order.

| Pass | Binary | Workers | Wall | Peak RSS | No-update evaluation |
|---|---|---:|---:|---:|---:|
| forward | baseline | 1 | 7.708 s | 1,544,764 KiB | 1,071.674 ms |
| forward | optimized | 1 | 6.454 s | 1,623,468 KiB | 79.427 ms |
| forward | baseline | 16 | 7.249 s | 1,545,444 KiB | 1,036.691 ms |
| forward | optimized | 16 | 5.954 s | 1,624,544 KiB | 5.900 ms |
| reverse | optimized | 16 | 6.009 s | 1,623,544 KiB | 6.092 ms |
| reverse | baseline | 16 | 7.402 s | 1,546,052 KiB | 1,028.606 ms |
| reverse | optimized | 1 | 6.636 s | 1,623,196 KiB | 79.386 ms |
| reverse | baseline | 1 | 7.877 s | 1,544,972 KiB | 1,098.881 ms |

### Registered bars

| Workers | Baseline wall | Optimized wall | Ratio | Reduction | No-update ratio | Reduction |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 7,792.394 ms | 6,545.059 ms | **0.8399** | **16.01%** | **0.0732** | **92.68%** |
| 16 | 7,325.815 ms | 5,981.061 ms | **0.8164** | **18.36%** | **0.0058** | **99.42%** |

Both wall means clear the 95% ceiling and both no-update means clear the 50%
ceiling. Wall speedups were **1.191x / 1.225x**; no-update evaluation speedups
were **13.667x / 172.229x**.

All eight runs produced exact artifact
`db4a672d6af83c4c56b7fbccc9ebc2279cb13fb8719b47829db64c910f7d2bb2`
and shuffled artifact
`19cbb1caa487473ac4344e1b3582060fbcb22b1e6066fa51540bcbc8977eabcd`.
Every normalized report was
`64e6e9b971ae0e90281aa485112db595e440eddeb29dbf99c9976a1fc467722f`.
No-update / exact / shuffled remained **332/246/270**, **331/245/269** and
**332/246/270** admitted / top-1 / top-8. Exact changed four rows and shuffled
changed zero. Every arm retained zero query-coordinate, admitted-cell, top-8
and token mismatches over 512 rows / 16,384 Q12 coordinates.

Every run recorded 21 finite stages with accounting equal to measured wall
within 0.001 ms, completed below 600 seconds, and stayed strictly below the
2-GiB RSS stop. The eight roots occupy **1,390,924,422 bytes**.

The compact evidence is [comparison.json](evidence/comparison.json);
[SHA256SUMS](evidence/SHA256SUMS) binds both binaries, reports and every
artifact copy.

## Checks run at the head

- Fresh-context non-author TEST FITNESS: FIT at implementation
  `83d1bbce86f272d4c396edeea14c1efba4275c6b` and docs head
  `9adf5e8b5e50feadf39eec1ca0e72ee8dcb2aa86`; the metadata-only ORDER
  correction and re-pin preceded the final verdict.
- Full `uor-r4-cpu-train` release suite: **238 tests passed, 0 failed, 4
  release benchmarks ignored**.
- Strict no-deps all-target CPU Clippy with `-D warnings`: passed.
- Package formatting for `uor-r4-cpu-train`: passed.
- Focused preloaded/reload parity, consolidated D11 mismatch, ordered-fold,
  stage fail-closed and RSS tests: passed.
- All eight runs passed artifact, table, decision, normalized-report, canonical
  21-stage accounting, timeout, worker, source, binary and strict-RSS gates.
- Prep check, shell syntax, JSON parsing, evidence checksum reconstruction and
  `git diff --check`: passed.

## Skills used

No system-provided skill applied in the implementation session. The fresh
reviewer read and applied the repository's `uor-research-architect` skill.

## Still open

- Profile the next dominant group after this change; the exact and shuffled
  evaluations still pay their independent parse costs by design.
- Complete replies, fresh phrasing, useful local chat and GPU qualification
  remain open.

## Plain summary

The count driver no longer reparses the original artifact just to evaluate its
unchanged no-update arm. It reduced total wall time by **16.01%/18.36%** and
no-update evaluation time by **92.68%/99.42%** at one/sixteen workers while
every artifact and decision stayed byte-identical. This is CPU implementation
performance evidence, not a capability result.
