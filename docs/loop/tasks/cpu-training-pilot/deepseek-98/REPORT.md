# REPORT: cpu-training-pilot, deepseek-98

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base:
`c701525932b9c5247cbe44658900414ff3ce030c`.

## What did not work, first

- **The automated claim failed before creating a branch.** `loop claim`
  generated a skeleton commit without the required `Lab:` and `Order:`
  trailers and the commit hook refused it. The tool reported a clean rollback.
  This unit was recreated by hand from fresh `origin/main`, with the ORDER
  committed first.
- **The known duplicate `uor-r4-integer` Cargo artifact collision appeared on
  the first test, Clippy and benchmark attempts.** The identical commands
  passed on immediate retry. The retained evidence files contain the passing
  retry, while the failing first attempts are recorded here rather than hidden.
- **The first strict Clippy run found one local type-complexity warning** in
  the new candidate-score return type. Named `CandidateScore` and
  `CoordinateScore` records remove it without an allowance.
- **Thirty-two workers are not universally faster.** The small panel gains
  slightly from SMT, while normalization, bias and decay candidate scoring are
  fastest at 16 physical workers and regress at 32. Route has only a 2.6%
  advantage at 32. The result supports bounded parallelism, not “more threads
  always means more speed.”
- **This is synthetic component performance.** The fixture is width 64, one
  layer, context 8 and vocabulary 4,096. It is not a language, corpus-training
  or GPU-qualification result. The benchmark's zero accepted counterfactual
  steps are irrelevant to cost: accepted-step identity is covered by the
  constructed fixture test across worker counts.

## What changed

`CpuPanelEvaluator::evaluate` now evaluates independent panel rows in parallel,
collects them in source order and performs the only aggregation serially. The
CPU and serving checks for every row remain complete; no session or evaluator
state is shared mutably.

`CpuCounterfactualTrainer::train_step` now scores independent coordinates and
their legal lanes in parallel. Results return to the original coordinate order
before best-lane, eligible-set and joint-BVG-K decisions. The accepted update,
artifact refresh and next-sample selection remain serial because each update
changes the following query.

The rank-margin evaluator was split into a lightweight cloneable
forward-plus-config view for independent panel rows. No arithmetic,
comparison, tie rule, artifact parser or checkpoint format changed. The two
release benchmarks now compare 1, 8, 16 and 32 worker pools and print stable
checksums plus baseline-relative speedup.

The source and benchmark change is 413 added and 225 removed lines across six
crate files. Documentation adds the performance record and this report.

## Measurements

Both benchmarks ran in release on the AMD Ryzen 9 9950X, 16 cores / 32 hardware
threads, with 16 physical workers as the comparison target.

Panel evaluation, 1,024 timed rows:

| Workers | Rows/s | Speedup |
|---:|---:|---:|
| 1 | 9,856.175 | 1.000x |
| 8 | 65,820.737 | 6.678x |
| 16 | 90,009.721 | 9.132x |
| 32 | 102,667.325 | 10.417x |

Every worker count produced checksum `6981217727883266160`, artifact BLAKE3
`260b989b…a3dd30`, zero query mismatches, zero token mismatches and zero
kept-cell mismatches.

Counterfactual candidate evaluation, eight timed steps per group:

| Group | 1 worker | 16 workers | Speedup at 16 | 32 workers |
|---|---:|---:|---:|---:|
| Normalization | 653.100/s | 4,784.820/s | 7.326x | 4,464.996/s |
| Bias | 576.396/s | 4,836.258/s | 8.391x | 4,438.674/s |
| Decay | 580.235/s | 4,685.514/s | 8.075x | 4,541.503/s |
| Route | 590.005/s | 5,192.435/s | 8.801x | 5,327.637/s |

Every group and worker count produced checksum `66383767735728885`.

Raw readings are [panel-bench.txt](evidence/panel-bench.txt) and
[counterfactual-bench.txt](evidence/counterfactual-bench.txt). Process peak RSS
in those logs is cumulative because all four pools run in one process; it is
not a per-worker memory reading.

## Checks run at the head

- `cargo test --release --locked -p uor-r4-cpu-train --lib --bins`: **159
  library tests passed, 4 release benchmarks ignored**, plus driver targets
  **6, 1 and 5 passed**. The full output is
  [package-release-tests.txt](evidence/package-release-tests.txt).
- Panel equality across 1, 2, 3, 5, 8, 16 and 32 workers passed in the release
  suite.
- Counterfactual equality across those worker counts, including the accepted
  step, artifact bytes and checkpoint bytes, passed in the release suite.
- `cargo clippy --locked -p uor-r4-cpu-train --all-targets --no-deps -- -D
  warnings`: passed; output retained in
  [package-clippy.txt](evidence/package-clippy.txt).
- `cargo fmt --package uor-r4-cpu-train -- --check`: passed.
- Both release benchmarks passed and retained the readings above.
- `git diff --check`: passed.

## Skills used

No available Codex skill applied.

## Still open

- Apply the same bounded, ordered pattern to other genuinely independent
  batch phases as they appear. The online rank-margin sample order remains
  intentionally serial.
- Measure the public corpus driver's end-to-end wall time with hardware-selected
  worker budgets. The synthetic candidate result should not be presented as a
  public corpus speedup.
- A configuration that clears a registered capability bar, unseen-key
  transfer, trained float shadow and sealed complete-reply/fresh-phrasing
  evidence remain open under #3506.

## Plain summary

The CPU trainer already used multiple cores for candidate artifacts and cached
trajectory samples, but two remaining loops still walked independent work on
one thread. This unit lets those panel rows and counterfactual candidates run
in parallel, then puts every result back in its original order before the
integer score is combined or an update is chosen. On the 16-core workstation,
panel evaluation became 9.13 times faster at 16 workers and counterfactual
search became 7.33 to 8.80 times faster. All tested worker counts produced the
same readings, accepted moves, artifacts and checkpoints. Using all 32 logical
threads was not consistently faster than using the 16 physical cores, so the
correct default is workload-aware rather than simply maximizing thread count.
