# REPORT: cpu-training-pilot, deepseek-100

> **Superseded by [deepseek-101](../deepseek-101/REPORT.md).** The fresh
> exact-head review separated audit computation from publication I/O, made
> trainer/gate/unattributed stages explicit, strengthened the worker assertion
> and replaced this report's stale size and timing claims. The worker-budget
> conclusion remains unchanged.

Lab: deepseek. Item:
[cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Order: [ORDER.md](ORDER.md). Base:
`6729e5d507bd39ef400c6515bc29b75f97e682d7`.

## What did not work, first

- **`loop claim` reproduced the known trailer-generation defect.** It created a
  skeleton commit without the required `Lab:` and `Order:` trailers, the hook
  refused it, and the tool removed everything it had created. The unit was
  recreated by hand after confirming that rollback.
- **The known cross-worktree `uor-r4-integer` artifact collision failed the
  first release build and full package test.** Both failed with unresolved
  `uor_r4_integer::stack` imports. The documented immediate retries passed; the
  failed test log is retained as
  [package-release-tests-first-attempt.txt](evidence/package-release-tests-first-attempt.txt).
- **The first independent-job timing wrapper did not run the intended
  experiment.** It passed seed 1 to both lanes. Those files are retained only as
  an execution error and are not used for the two-artifact result.
- **The first concurrent timing wrapper could not wait on grandchildren.** It
  produced no corpus output. The correction put both jobs inside the timed
  shell; the failed wrapper log is retained.
- **More cores do not speed up every phase.** The training stage stayed at
  283-288 ms and the recurrent 64-position audit at 35 ms from one through 32
  workers. The wall gain comes from the two independent 1,024-row panels.
- **More workers are not universally best.** This public panel-heavy corpus
  workload improves through 32 hardware threads, while the earlier synthetic
  counterfactual candidate search remains faster at 16 physical workers. The
  worker budget is therefore workload-specific.
- **One clean-head full gate run hit an `ETXTBSY` harness-test flake.** The
  loop-drive fake `gh` executable could not be launched on that attempt. The
  immediate full rerun was green on all 11 gates.

## What changed

`cpu-corpus-driver` now records its effective Rayon worker count and six
non-overlapping stage times: setup, baseline panel, online updates, final panel,
checkpoint/reload and final served-forward audit. A focused test checks that
every stage is present and non-negative. No integer decision, training schedule,
artifact format or qualification bar changed.

The unit adds 34 lines to one code file, including the assertion, plus generated
and hand-written reports under the task folder. It also adds one notebook row
and updates the CPU program and roadmap pages with the measured recommendation.
The initial deepseek-100 implementation commit changed 40 files and inserted
4,402 lines because the retained raw JSON, timing logs and test logs accounted
for nearly all of it. The repair re-ran and replaced those readings; the active
diff size is recorded by deepseek-101.

## Measurements

Host: AMD Ryzen 9 9950X, 16 physical cores / 32 hardware threads.
Repaired binary SHA-256:
`51d6c1b4c543e24bd5bb1b0fa2b769efb1554521891935e4ddba309a17903ffd`.
Each run started from public dual-c131 artifact seed 1
`a9df5263…6cfd`, used the same 16 update attempts, trainer seed 1, evaluation
stride 2, 1,024-row panel cap and checkpoint interval 16:

```sh
env RAYON_NUM_THREADS="$workers" target/release/cpu-corpus-driver \
  ARTIFACT TOKENS MASK MANIFEST 1 16 2 1024 16 RUN_DIR
```

Two runs used worker order `1, 8, 16, 32`; two independent reruns used the
reverse order:

| Workers | Forward wall | Reverse wall | Average speedup vs one |
|---:|---:|---:|---:|
| 1 | 62.045 s | 62.248 s | 1.000x |
| 8 | 8.483 s | 8.475 s | 7.330x |
| 16 | 4.640 s | 4.630 s | 13.409x |
| 32 | 3.534 s | 3.522 s | 17.616x |

Average process CPU use rose from 99% at one worker to 758-760% at eight,
1,440-1,451% at 16 and 2,751-2,761% at 32. Peak RSS rose from about 53 MiB to
about 125 MiB.

All eight runs produced final artifact
`6b35fed04be8a98168b7a7a0a22030300de5383b7576c0c06a812f28e7bb38c8`,
final checkpoint
`b3a9c501c42d94f88091f4360a9b0a67dc8d294ebf5bdbb0d58b98de20e6ce24`,
panel `278 / 384 / 396`, served-forward decision hash
`2ac6b0971afe222860453846f286385428bb870e802fccf66a6097965ea1b897`,
and zero query, token or kept-cell mismatches over 262,144 Q12 coordinates and
8,192 kept cells. The reproducible state-comparison command is in the
measurements receipt.

Two real artifact seeds also ran as two sequential 32-worker jobs and two
concurrent 16-worker jobs. Sequential took 7.06 s; concurrent took 6.60 s, a
6.5% wall reduction. Each seed retained the artifact, checkpoint, panel and
audit from its sequential counterpart.

The full stage table, commands and raw links are in
[MEASUREMENTS.md](evidence/MEASUREMENTS.md).

## Checks run at the head

- `CARGO_TARGET_DIR=... cargo test --release --locked -p uor-r4-cpu-train
  --lib --bins`: passed on immediate retry after the known artifact collision.
  The library ran 159 tests with 4 release benchmarks ignored; driver targets
  ran 6, 1 and 5 tests; all other binary harnesses exited successfully.
- `CARGO_TARGET_DIR=... cargo clippy --locked -p uor-r4-cpu-train
  --all-targets --no-deps -- -D warnings`: passed. Dependency dead-code warnings
  remain outside this crate.
- `cargo fmt --package uor-r4-cpu-train -- --check`: passed.
- `git diff --check`: passed.
- Evidence `SHA256SUMS`: passed.
- `uor-harness loop gates --full`: 11 of 11 gates green.

## Skills used

No available Codex skill applied.

## Still open

- A configuration that clears a registered capability bar.
- Unseen-key transfer, trained float shadow and credentialed complete-reply and
  fresh-phrasing evidence under #3506.
- A general scheduler for independent CPU jobs. This unit measured the
  16+16 concurrent pattern but did not add an automatic launcher.
- CPU training remains development research under D84 and does not replace GPU
  qualification.

## Plain summary

The expensive part of a public corpus run is checking 1,024 examples twice,
once before training and once after. Each example is independent, so the
program already spreads those checks across CPU cores and reassembles their
integer counts in the original order. On this 16-core machine with 32 hardware
threads, using 32 workers reduced a 62-second run to 3.56 seconds in two
different execution orders. The trained files and every score stayed identical.
The ordered learning updates and recurrent audit cannot be split without
changing their meaning, so they stayed near 0.3 seconds and 0.05 seconds.
For one run, use 32 workers. For two independent artifacts when wall time
matters, two 16-worker jobs can run together and were 6.5% faster than running
two 32-worker jobs one after the other.
