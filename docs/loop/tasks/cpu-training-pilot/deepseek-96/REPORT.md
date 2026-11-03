# REPORT: cached memory-trajectory training, deepseek-96

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md)
(#3506). Order: [ORDER.md](ORDER.md). Sheet:
[cpu-training-pilot-memory-trace-2026-11-03.md](../../../harness/prep/cpu-training-pilot-memory-trace-2026-11-03.md).
Base: `3b893ddc0ee29a43750632799aaae340540a0bbe`.

## What did not work

- **Candidate-only parallelism still left most of the workstation idle.** The
  first cached run took 28.468/28.198 s and averaged about four cores because
  trace construction, five matched evaluator arms and five full parity walks
  remained serial. Those phases now use the same bounded worker pool.
- **Caching only applies to a fixed teacher-forced token trajectory.** A
  changed token stream, model trunk, recurrence state or memory-table hit can
  invalidate the cached decisions. The full-forward trainer remains executable
  behind `--reference`.
- **The panel repeats exact opening keys.** This tests transfer across held-out
  examples and documents with the same key, not an unseen opening key. It does
  not test complete replies, fresh phrasing or language capability.
- **The existing duplicate `uor-r4-integer` Cargo artifact collision appeared
  during two package check attempts as unresolved imports.** This is the known
  feature/build-dependency collision already recorded by earlier CPU tasks.
  The identical command passed on retry at the same source; the passing result
  is recorded below.
- **The three-sigma ranking result remains negative.** The optimization changes
  cost only. The deepseek-95 top-1 gains of 13 and 5 rows still miss thresholds
  of 48 and 44.

## What changed

The CPU FBS forward now exposes crate-private pre-memory normalization and
memory-hit metadata. The memory trainer caches that fixed trajectory once,
then evaluates a candidate by applying only its exact Q24 memory movement,
Q12 conversion, writer decision and point fallback. The public `step` path is
unchanged and `--reference` retains full-forward training.

Independent teacher-forced samples, candidate artifacts, matched evaluator
arms and parity samples now run through the same Rayon pool. Every task returns
its integer result to its original ordered slot; the final reduction is serial,
so saturation and tie behavior cannot depend on thread scheduling.

The change is approximately 470 code/doc lines, larger than the initial
candidate-search estimate because the reviewer-requested sample-level
parallelism and its exactness tests were folded into the same unit. Evidence is
generated data; the tracked evidence files are the pre-FIT sheet snapshot, two
complete reports, two raw timing receipts, the comparison and their checksum
manifest.

## Measurements

Both registered runs used the AMD Ryzen 9 9950X, 16 physical cores and the
pre-registered 16 Rayon workers with `RAYON_NUM_THREADS=16`.

| Seed | Full-forward wall | Cached/parallel wall | Speedup | Average CPU | Peak RSS |
|---:|---:|---:|---:|---:|---:|
| 1 | 415.370 s | **8.774 s** | **47.34x** | 1,385% | 272,116 KiB |
| 2 | 414.121 s | **8.397 s** | **49.32x** | 1,431% | 265,260 KiB |

Each seed accepted all eight updates. Removing wall, peak RSS and the newly
reported `training.search_mode` field from the old and new JSON produces
byte-identical normalized reports: accepted step receipts, held-out readings
and parity are therefore exact, not merely equivalent. The reference run did
not retain its artifact bytes, so artifact identity is established by the
pinned SHA-256 values rather than a repeated `cmp` against the old file. Final
artifact hashes remain:

- seed 1:
  `3ad89c7b80df52f9e1e4d878c072982e9fb613b8453ded0dd50c3ee905e0ee44`;
- seed 2:
  `070d66364112d4527d3043998f023f435ec73455b9a617aa94695403c62bd089`.

The held-out trained readings remain **291 / 2,820,387,931** for seed 1 and
**243 / 2,896,327,331** for seed 2. Every arm on both seeds checked 56 samples,
2,487 positions, 636,672 Q12 coordinates, 2,487 decisions and 19,896 kept
cells with every mismatch counter at zero.

An auxiliary, non-registered SMT probe used all 32 logical CPUs on seed 1. It
finished in 7.657 s at 2,494% average CPU with the same artifact and readings,
but it is not part of the 16-physical-core FIT claim.

Evidence: [comparison.json](evidence/comparison.json),
[prefit-sheet.md](evidence/prefit-sheet.md), [seed-1.json](evidence/seed-1.json),
[seed-2.json](evidence/seed-2.json), the raw
[timing-seed-1.txt](evidence/timing-seed-1.txt) and
[timing-seed-2.txt](evidence/timing-seed-2.txt), and
[SHA256SUMS](evidence/SHA256SUMS). The committed pre-FIT snapshot hashes to
`ae440a49...`, exactly matching the FIT review note. The current annotated
sheet has a different hash without changing its bar, inputs or claim boundary.

## Checks run at the head

- Focused cached/full-forward suite:
  `cargo test --package uor-r4-cpu-train --release --lib memory_train`:
  **12 passed**, including three-step full/optimized equality, opening-only,
  reply-only, parameter-bound, deterministic repeated construction,
  opening/reply hit and miss, point fallback, every checked aggregation
  overflow class, and unseen-key no-hit controls.
- Full package release suite:
  `cargo test --package uor-r4-cpu-train --release`: **151 library tests
  passed, 4 release benchmarks ignored**, plus driver suites **6, 1 and 5
  passed**; every listed test target was green after the known Cargo collision
  retry.
- `cargo clippy --package uor-r4-cpu-train --all-targets --no-deps -- -D warnings`: passed.
- `cargo fmt --package uor-r4-cpu-train -- --check`: passed.
- `git diff --check`: passed.
- JSON parsing for both seed reports, the comparison and the claims ledger: passed.
- `sha256sum -c SHA256SUMS`: all six evidence files passed.
- Both registered seeds exited zero with exact complete CPU/serving parity.
- `uor-harness loop report --check --order …/deepseek-96/ORDER.md`: PASS.
- `uor-harness loop pr-check --local`: PASS, with advisory D76 warnings only for
  the branch claim file's item scope and pre-existing stale FILEMAP rows.

## Skills used

No available Codex skill applied. The FBS trace, artifact layout, memory
movement, evaluator and parity path were inspected and tested directly.

## Still open

- Unseen-key memory transfer.
- A held-out comparison that clears a registered capability bar.
- Complete-reply and genuinely fresh-phrasing evaluation.
- Cached-trajectory reuse for mutable recurrence or parameter groups beyond
  the fixed teacher-forced memory-weight path.
- GPU qualification of this mechanism family.

## Plain summary

The old trainer rebuilt the whole geometric network for every candidate memory
adjustment even though the text order did not change. The new path computes
that fixed trajectory once, then tests memory adjustments at the only point
where they can change the answer. It also distributes independent samples,
candidates and audits across all 16 physical cores, with a deterministic final
combine. Both model seeds now finish in under ten seconds and produce exactly
the same artifacts and readings as the 415/414-second full-forward runs. This
is a large training-speed improvement, but it is not yet evidence of a useful
language model.
