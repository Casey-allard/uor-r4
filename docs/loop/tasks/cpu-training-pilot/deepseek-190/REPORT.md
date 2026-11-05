# REPORT: cpu-training-pilot, deepseek-190

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base: `3ac40d6d5`.

## Status

Complete at the CPU-preflight scope. Run 2 exercised the current
whole-table-nearest recipe. All 18 registered cells passed exact
export/load/CPU/serving parity in **18.476 s** with **76,388 KiB** peak RSS.
This is instrument, byte-cost and decode-throughput evidence for the
scale-probe widths; it is not corpus training, a scale slope, a quality result
or a statement that any width fits GPU memory.

## What did not work

- The automated `loop claim` rolled back because its generated commit lacked
  the required `Lab:` and `Order:` trailers. The documented hand-claim path
  was used.
- The first prep-shape check found missing `Base artifact` and `Data split`
  blocks. Both are now explicit.
- The first fresh review found an invalid full source hash, missing synthetic
  shape constants in JSON evidence and fail-open peak-RSS parsing. It rejected
  FIT/GO before compute; all three were repaired.
- The second fresh review found missing reloaded side-table provenance,
  incomplete shape assertions, a mismatch at exactly 2 GiB and one retained
  hash for three repeated audits. All four were repaired.
- **Attempt 1 exercised the synthetic default cell-writer readout, not the
  current `--objective cell` recipe's whole-table-nearest readout.** The
  exact-head result review caught this material coverage error. Attempt 1 is
  preserved byte-identically as
  [scale-serving-attempt-1-cell-writer-summary.json](evidence/scale-serving-attempt-1-cell-writer-summary.json)
  and is excluded from the result below.
- Run 2 required a fresh FIT/GO after the readout correction.
- There is no GPU-memory result. A width-2,048 artifact is refused by the
  current writer cap, and this CPU run cannot show whether 1,024 fits GPU
  training.

## What changed

- Added `cpu-scale-serving-bench`, which constructs deterministic synthetic
  FBS trunks at widths 256, 512 and 1,024 and representative contexts 16, 64
  and 192.
- Both arms use the current recipe's `whole_table_nearest_l1` writer.
- Exports every cell through `UORLUT01`, repeats the complete
  `audit_served_forward` comparison three times, and requires exact query,
  token and kept-cell parity plus matching CPU/serving decision hashes.
- Records all three decision hashes, artifact and table bytes, complete shape
  constants, readout mode, decode throughput, RSS and all four negative
  controls.
- Requires projection artifacts to carry no side tables and full artifacts to
  retain the expected Hamming heads/depths and nonempty opening/reply memory.
- Validates the reloaded vocabulary, width, context, MLP, layer count,
  kept-cell count and readout mode. Peak-RSS parsing fails closed and exactly
  2 GiB is rejected.
- Added the pre-registration, FIT/GO record, compact evidence and README entry.

No serving, CUDA, float-training, exporter or other Claude-owned source
changed.

## Measurements

Run 2 binary:
`ccc730000376be07badab2f01f30686ee9c960473ac0040162bea1c7b325a6ac`.
Summary SHA-256:
`9ee57c053b93c69bfb7ab46495158fe13357409e51beaed0672e7aca2d804c88`.

| Width | Arm | Context | Artifact bytes | Resident bytes | Serving tok/s |
|---:|---|---:|---:|---:|---:|
| 256 | projection | 16 | 2,145,088 | 2,147,296 | 1,800.8 |
| 256 | projection | 64 | 2,145,088 | 2,147,296 | 1,791.0 |
| 256 | projection | 192 | 2,145,088 | 2,147,296 | 1,749.9 |
| 256 | full | 16 | 2,160,576 | 2,291,056 | 1,763.4 |
| 256 | full | 64 | 2,164,608 | 2,292,208 | 1,755.5 |
| 256 | full | 192 | 2,175,360 | 2,295,280 | 1,721.4 |
| 512 | projection | 16 | 4,280,384 | 4,288,832 | 935.6 |
| 512 | projection | 64 | 4,280,384 | 4,288,832 | 939.7 |
| 512 | projection | 192 | 4,280,384 | 4,288,832 | 937.8 |
| 512 | full | 16 | 4,308,224 | 4,575,984 | 926.5 |
| 512 | full | 64 | 4,312,256 | 4,577,136 | 916.9 |
| 512 | full | 192 | 4,323,008 | 4,580,208 | 910.4 |
| 1,024 | projection | 16 | 8,551,488 | 8,571,904 | 477.4 |
| 1,024 | projection | 64 | 8,551,488 | 8,571,904 | 480.5 |
| 1,024 | projection | 192 | 8,551,488 | 8,571,904 | 480.0 |
| 1,024 | full | 16 | 8,603,840 | 9,145,840 | 475.6 |
| 1,024 | full | 64 | 8,607,872 | 9,146,992 | 471.8 |
| 1,024 | full | 192 | 8,618,688 | 9,150,064 | 466.0 |

The read-free artifact grows almost exactly linearly with width:
`2,145,088 -> 4,280,384 -> 8,551,488` bytes, ratios **1.9955** and
**1.9978**. Whole-table-nearest serving throughput approximately halves when
width doubles. The full-path overhead over projection is:

- width 256: 15.1, 19.1 and 29.6 KiB at contexts 16, 64 and 192;
- width 512: 27.2, 31.1 and 41.6 KiB;
- width 1,024: 51.1, 55.1 and 65.6 KiB.

The 16-to-192 context change costs about 2-3% in the full arm and is within
about 3% in projection. These are single-pass descriptive timings on one host,
not a noise-banded benchmark.

- 18 cells, 54 complete audit repeats, 4,896 token decisions, 2,924,544 Q12
  query-coordinate comparisons and 39,168 kept-cell comparisons;
- zero query-coordinate, token, kept-cell or decision-hash mismatch;
- every cell's three decision hashes are present and equal;
- all four negative controls refused: truncated artifact, out-of-vocabulary
  token, sequence over context and width above the 1,024 writer cap;
- process wall **18.476 s**, peak RSS **76,388 KiB**;
- AMD Ryzen 9 9950X, 16 cores / 32 logical CPUs;
- Ubuntu 26.04.1 LTS, Linux `7.0.0-42-generic`, x86-64.

Compact evidence:
[scale-serving-summary.json](evidence/scale-serving-summary.json) and
[scale-serving-time.log](evidence/scale-serving-time.log), bound by
[PUBLISHED-SHA256SUMS](evidence/PUBLISHED-SHA256SUMS). Attempt 1 is preserved
beside them but excluded from this result.

## Checks run at the head

- Fresh run-2 reviewer: **TEST FITNESS: FIT** and **D67 PRE-RUN: GO** at
  `50f0d7d76`;
- focused `cpu-scale-serving-bench` release tests: **3 passed**;
- full `uor-r4-cpu-train` release suite: **351 passed**, 4 expected ignores,
  all binary tests passed;
- strict scoped Clippy:
  `cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D warnings`;
- `cargo fmt --check`, `git diff --check`, D23/D25/D32 ratchets and claim
  wording passed;
- `uor-harness check --only prep.sheet_shape`, `prep-check` and
  `loop pr-check --local` passed;
- source/binary pins and absent output root were checked before run 2;
- the registered output passed its schema, cell-count, readout, repeat-hash,
  parity, control and RSS predicates after read-back;
- registered wall time and RSS remained below the 20-minute and 2-GiB stops.

## Skills used

No system-provided Codex skill applied. Repository pre-registration, FIT/GO,
evidence and loop-delivery procedures were used.

## Still open

- Claude's scale-probe still has to establish quality growth on real text and
  GPU training-memory feasibility at these widths.
- This preflight covers the exact synthetic output path, not real trained
  artifacts. The same audit should consume Claude's exports when they exist.
- Exact-head delivery review after this corrected result, protected merge,
  fresh-main verification and claim release remain.

## Plain summary

The corrected run covers the recipe's whole-table-nearest path at all three
scale-probe widths and three representative contexts. Every exported artifact
reloaded and served exactly across three repeated audits. Artifact bytes scale
linearly with width, while decode throughput approximately halves each time
width doubles and changes only modestly with context. This removes output-path
uncertainty from the scale run but does not answer whether width improves
quality or fits on the GPU.
