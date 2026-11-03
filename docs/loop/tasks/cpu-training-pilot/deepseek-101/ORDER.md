# ORDER: cpu-training-pilot, deepseek-101 (repair the worker-budget evidence)

Lab: deepseek. Item: [cpu-training-pilot](../../../../roadmap/items/cpu-training-pilot.md) (#3506).
Base: `6729e5d507bd39ef400c6515bc29b75f97e682d7`.

## Goal

Repair the fresh-context CHANGES review of PR #3620 at
`77992c8d339002edfe538973d4896f0cef03f299`.

The completed deepseek-100 unit measured 32 workers as fastest on the real
public panel workload. The review found that the reported audit interval also
included atomic publication and reload, that trainer/gate time was unattributed,
that the new field test did not verify worker attribution or bounded accounting,
that retained logs failed the exact-head whitespace check, and that the report's
diff size and semantic checksum recipe were stale.

## Files in scope

- `crates/uor-r4-cpu-train/src/bin/cpu-corpus-driver.rs`
- `docs/loop/tasks/cpu-training-pilot/deepseek-100/**`
- `docs/loop/tasks/cpu-training-pilot/deepseek-101/**`
- `docs/research/cpu-training-program.md`
- `docs/research/notebook/cpu-training.md`
- `docs/roadmap/items/cpu-training-pilot.md`

## Done when

1. The audit interval covers audit computation and serialization only;
   artifact/receipt publication and reload have a separate interval.
2. Trainer construction/resume and gate construction have explicit,
   non-overlapping stages, and the report exposes any unattributed wall time.
3. Tests install a bounded Rayon pool, assert the reported worker count and
   stage keys, and verify that stage time does not exceed measured wall.
4. The worker-budget and independent-job runs are regenerated from the repaired
   binary, with byte-identical state and corrected stage readings.
5. All retained evidence is whitespace-clean, `git diff --check
   origin/main...HEAD` passes, and the semantic-comparison recipe is reproducible.
6. The active REPORT records the final exact file/addition/deletion counts and
   every failed or retried check.
7. A fresh exact-head review approves the repaired head.

## Stop and ask

- Stop if separating timing changes an integer decision, artifact or panel
  reading.
- Do not change a training bar, worker recommendation without new evidence, or
  any serving behavior.
