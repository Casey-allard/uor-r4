# REPORT: consolidate CPU count verification, deepseek-128

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Prep:
[verification consolidation](../../../harness/prep/cpu-training-pilot-trunk-verify-2026-11-04.md).
Base: `e5b7c500c`.

## What did not work, first

- **The first pre-registration review returned NOT FIT.** Its finite test
  compared a table set with itself, did not exercise a real D11 import, and the
  recipe did not hash generated artifact files directly. The replacement test
  round-trips source, exact, changed-semantics and wrong-table D11 artifacts;
  every run now hashes both generated files against the accepted oracles.
- **The second pre-registration review returned NOT FIT.** The source gate
  covered only `crates/uor-r4-cpu-train` even though the implementation pin also
  included `crates/uor-r4-discrete`. The gate now binds all `crates/**` and both
  root Cargo manifests.
- **The optimization's peak-memory risk was real but bounded.** Exact
  verification briefly holds source expected tables and the decoded exact table
  set together. The final eight-run peak was 1,545,672 KiB, 551,480 KiB below the
  registered stop. Both complete table sets are dropped before the shuffled
  arm imports its independent source tables.
- **No model-math or capability question was tested.** The exact artifacts,
  tables, decisions, checksums and normalized reports all had to reproduce
  deepseek-125/126; only verification implementation cost changed.

## What changed

The previous verification path parsed the full 19-table trunk ten times, twice
for each semantic comparison, independent expected-table reconstruction,
decoded-table comparison and table summary. The new path performs four imports:

1. source semantics plus tables;
2. exact exported artifact;
3. a fresh source import for the independent shuffled expectation; and
4. shuffled exported artifact.

The first source table set is moved directly into exact expected-count
reconstruction rather than cloned. Each exported arm is decoded once for
semantics, full decoded-table equality and summaries. The shuffled expected
tables still come from a separate full source import. The 21-stage profile,
source-order parallel row work, update/export behavior and model arithmetic are
unchanged.

The finite regression test now constructs actual exported D11 source, exact,
changed-semantics and wrong-table artifacts. It checks successful import,
semantics, full decoded tables and summaries through the production helpers,
then requires both semantic and table mismatches to be refused.

## Measurements

The fixed split held 200 training documents / 7,080 targets and 193 evaluation
documents / 512 targets. Eight fresh roots ran the registered mirrored order.
Two earlier registration rounds also passed. A package-format pass changed the
first binary hash, and an exact-head review then found that a test-only accessor
had landed outside the item scope. The final test uses the existing public
`q4_import` kappa constants; `crates/uor-r4-discrete/src/trunk.rs` now has no
diff from `origin/main`. A fresh FIT and this aligned attempt-3 panel replaced
both earlier attempts, which remain diagnostic and are not reused below.

The final panel pinned source
`be0edd4682409398ebf2e7f86b4b44ec987f8edf`, baseline binary
`e4dac42e783958a082ebfe0b3cf63ec4f54ea149d73a19764114878559c14ada`
and optimized binary
`e32f99e8494ec51d26d523861ae4511ade3a73a5a03dbc29bd557105d3680dd4`.

| Pass | Binary | Workers | Wall | Peak RSS | Verification |
|---|---|---:|---:|---:|---:|
| forward | baseline | 1 | 9.828 s | 1,543,152 KiB | 4,498.098 ms |
| forward | optimized | 1 | 8.000 s | 1,545,092 KiB | 2,279.686 ms |
| forward | baseline | 16 | 9.574 s | 1,544,992 KiB | 4,555.513 ms |
| forward | optimized | 16 | 7.468 s | 1,545,452 KiB | 2,275.430 ms |
| reverse | optimized | 16 | 7.453 s | 1,545,672 KiB | 2,247.289 ms |
| reverse | baseline | 16 | 9.625 s | 1,544,932 KiB | 4,573.763 ms |
| reverse | optimized | 1 | 7.907 s | 1,545,528 KiB | 2,243.819 ms |
| reverse | baseline | 1 | 9.998 s | 1,544,092 KiB | 4,538.153 ms |

### Registered bars

| Workers | Baseline wall | Optimized wall | Ratio | Reduction | Verification ratio | Reduction |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 9,912.832 ms | 7,953.219 ms | **0.8023** | **19.77%** | **0.5006** | **49.94%** |
| 16 | 9,599.227 ms | 7,460.616 ms | **0.7772** | **22.28%** | **0.4954** | **50.46%** |

Both wall means clear the 90% ceiling and both verification means clear the
80% ceiling. Wall speedups were **1.246x / 1.287x**; exact plus shuffled
verification speedups were **1.998x / 2.019x**. At one worker,
`exact_verify` fell from 2,657.280 to 1,133.426 ms and `shuffled_verify` from
1,860.846 to 1,128.326 ms. At sixteen workers they fell from
2,688.168/1,876.470 ms to 1,137.510/1,123.850 ms.

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
2-GiB RSS stop. The eight roots occupy **1,390,924,424 bytes**.

The compact evidence is [comparison.json](evidence/comparison.json);
[SHA256SUMS](evidence/SHA256SUMS) binds both binaries, reports and every
artifact copy.

## Checks run at the head

- Five fresh-context non-author TEST FITNESS reviews: the first two returned
  NOT FIT; the next three repaired source revisions returned FIT. The final FIT
  was at docs head `ba0ca6559502739fed48261403f727e63399b83d`.
- Full `uor-r4-cpu-train` release suite: **237 tests passed, 0 failed, 4
  release benchmarks ignored**.
- Strict no-deps all-target CPU Clippy with `-D warnings`: passed.
- Package formatting for `uor-r4-cpu-train` and `uor-r4-discrete`: passed.
- Focused consolidated D11 success/mismatch, ordered-fold, stage fail-closed
  and RSS tests: passed.
- All eight attempt-3 runs passed artifact, table, decision, normalized-report,
  21-stage accounting, timeout, worker, source, binary and strict-RSS gates.
- Prep check, shell syntax, JSON parsing, evidence checksum reconstruction and
  `git diff --check`: passed.
- Full loop gates: **11/11 green**. Report check and local policy check passed;
  the policy warnings are advisory D76 scope/commit-order and pre-existing
  FILEMAP rows.
- Roadmap check: **139 items**; documentation catalog: **1,175 records**, no
  missing paths; state formats: **8/8 ok**; D23/D25/D32 ratchets unchanged.
- `uor-harness check`: **0 errors, 136 warnings**, chiefly pre-existing stale
  claims, presence and merged branches outside this unit.

## Skills used

No system-provided skill applied in the implementation session. Each fresh
reviewer read and applied the repository's `uor-research-architect` skill.

## Still open

- Verification is no longer the dominant measured group. Further CPU
  performance work should start from the optimized profile rather than assume
  the earlier deepseek-127 hotspot still dominates.
- Complete replies, fresh phrasing, useful local chat and GPU qualification
  remain open.

## Plain summary

The count driver now verifies the same two exported artifacts with 60% fewer
full-trunk imports. It reduced wall time by **19.77%/22.28%** and verification
time by **49.94%/50.46%** at one/sixteen workers, while every artifact, table,
decision, checksum and normalized report stayed byte-identical. This is a
verified CPU implementation speedup, not a language-capability result.
