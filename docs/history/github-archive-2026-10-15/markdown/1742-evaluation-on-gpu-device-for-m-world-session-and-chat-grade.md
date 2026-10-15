# #1742 — Evaluation on GPU: device= for m-world session and chat-grade; parallel session runner (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T14:38:45Z
- **Updated:** 2026-10-09T13:08:48Z
- **Closed:** 2026-10-05T15:12:16Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1742
- **Merged:** 2026-10-05T15:12:16Z
- **Merge commit:** 75998b9193901845c687d923537d7c90a24104fa
- **Base / Head:** main / UOR-Foundation:codex/eval-device
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #820. Owner request: evaluation should run on the pod's GPUs and CPUs, not the laptop.

## Change

- `m-world` `evaluate` (v1/v2), `evaluate-cells`, `probe` and `session` take `device=cpu|cuda|metal`. The default is `cpu`, so commands without it behave as before. The device is parsed and opened with the arguments, before the report root is claimed. There is no fallback: a build without `--features cuda` refuses `device=cuda`. TF32 is held off. Each report records `device` (`device`, `tf32`, `CUDA_VISIBLE_DEVICES`).
- Placement in `session`: the emitter and the op-model `trunk=` run on `device`. The compiler's saved tables and op policy, the log sieve, the exact store and the judging stay on the CPU, which the report records as `device_placement`. The emitter checkpoint is still written from a CPU load, and every session loads it onto `device`. `evaluate`'s `route_trunk=` follows `device`. `compiler` and `compiler-save` (fitting) are unchanged and stay on CPU.
- `chat-grade` `grade` and `reply` take the same `device=`, used for reply generation only. The grader is unchanged. The device is recorded in the report.
- `scripts/pod/run-sessions-parallel.sh MODEL_ROOT...` runs sieve and off D19 sessions for several models concurrently. It splits a total thread budget (`THREADS`, default 8) across the jobs, round-robins `GPUS` with `DEVICE=cuda`, skips sealed roots, refuses unsealed ones and fails any root that did not seal.

## Evidence (pod 2x RTX 4090, exact Rust source of 36825f2; the later head only changes the script's default binary path)

Build: `--features cuda`, `CUDA_COMPUTE_CAP=89`, release, `-j 8`. Model: `/root/runs/step5/ft-C-A-s1` (read-only). Compiler: `compiler-save-op-v25-rawtable`, trunk `op-model-v25`, `op_policy=unless_query`.

**D19 session parity** (world=v2, seed 9101, 60 conversations, log_recall=sieve, max_new_tokens=64, arms=default, RAYON 8 threads, GPU 1):

| | cpu | cuda |
|---|---|---|
| scored turns | 226 | 226 |
| turns passed | 200 | 200 |
| conversations all-pass | 37/60 | 37/60 |
| MQAR | 22/23 | 22/23 |
| open relation | 10/12 | 10/12 |
| same pass/fail per turn | 226/226 | |
| identical reply text per turn | 226/226 | |
| identical compiled action per turn | 226/226 | |
| wall seconds | 278.0 | 35.2 (7.9x) |

**chat-grade reply parity** (everyday-32, protocol 2, max_new_tokens 64): 40/40 replies are identical in text and generated ids. Generation took 27.9 s on CPU and 2.8 s on CUDA (9.8x). In this run greedy decoding was identical across devices. A float reduction-order difference could still change an argmax on another model, so a device change is recorded and should be compared row by row.

**Parallel runner smoke**: `DEVICE=cuda GPUS=0,1 THREADS=8 CONVERSATIONS=20` over ft-C-A-s1 and ft-C-K-s1 ran 4 sessions concurrently with 2 threads each. All 4 roots sealed in 14 s. MQAR was 10/10 with the sieve and 0/10 with it off.

**Tests**: `cargo test --release -p uor-r4-training --example m-world --bin chat-grade` passed 21 + 2 tests both with and without `--features cuda`. That includes the new `the_device_defaults_to_the_cpu_and_never_falls_back` in both binaries; without the feature it confirms `device=cuda` is refused. `rustfmt --check --edition 2021` is clean on both touched files. The change adds no warnings. The `unreachable pattern` in `describe_action` and the library dead-code warnings were already there.

Limitations: one model and one seed, so parity is measured, not proved. Metal was not exercised. Speedups are wall-time ratios for batch-1 greedy decoding that recomputes the whole window each step. They are not energy measurements.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 5996931308 — 2026-10-05T14:50:34Z — Casey-allard

Adversarial review at 0984750464459e1982d40784652c5a3a95d29083: **APPROVE** (no blocking findings)

**Source identity.** The pod source in `/root/evalgpu/eval-device` matches this head blob for blob for all 1,290 files under `crates/`, `Cargo.toml`, `Cargo.lock` and `rust-toolchain.toml`, checked with `git hash-object` against `git ls-tree`. The only difference is `scripts/pod/run-sessions-parallel.sh`, which is the head commit's default-path change. No source file is newer than the binaries. Binary sha256: m-world `e1f8c8cc…baafd`, chat-grade `8a14634d…76819`.

**Default behaviour (device omitted).** `EvalDevice::parse(None)` returns `Cpu`. Every place that changed replaces `Device::Cpu` with `args.device`, which is `Cpu` by default. `set_gemm_reduced_precision_f32(false)` has no effect on the CPU. Report JSON is not byte-identical, because it gains the `device` key and, in a session, `device_placement`. The model outputs are unchanged. In my run, a session with no `device=` matched the CUDA session on every turn record, and the whole report matched once the device and timing fields were removed (below).

**CUDA placement checked against the source.**
- These follow `device`: `evaluate`, `evaluate_v2`, `evaluate_cells` and `probe_evaluate` models, `evaluate_v2`'s `route_trunk=`, the session's `Trunk::load`, `GroundedSession::from_checkpoint_path` and `load_with_log_recall` (the reload path), and `StackModel::load` in chat-grade.
- These stay on the CPU, as the PR states: the session's checkpoint-writing load, the `compiler-save` trunk, and `route` mode, which does not accept `device=`.
- The device is opened in `Args::parse`, before `claim`, and the same is true in chat-grade `grade` and `reply`.

**My own parity run on the pod GPU.** Settings: GPU 0, 8 threads, KEEP_ALIVE held while working and then removed. I used a different model and seed from the PR's run, and turned the reload path on.
- Session settings: `ft-C-K-s1` (read-only), world=v2, seed 4242, 20 conversations, log_recall=sieve, trunk op-model-v25, unless_query, reload=1. CPU was run with no `device=`; CUDA with `device=cuda`.
  - 70/70 turns had the same pass/fail, reply text and action, and 70/70 full turn records were identical apart from timing.
  - Turns passed: 63 on CPU, 63 on CUDA. Conversations all-pass: 15/20 on both.
  - The whole report was equal once device and timing fields were removed.
  - Wall time: 76.5 s on CPU, 13.9 s on CUDA.
- chat-grade `reply` on a different model, `/root/runs/geo-100m/model`, with everyday-32 and max_new_tokens=32: 32/32 rows identical. Generation took 185.6 s on CPU and 4.9 s on CUDA.
- `device=gpu` was refused (`unknown device=gpu … no implicit fallback`) and no root was created. Re-running into an existing root was refused with `AlreadyExists`, and the sealed root was left untouched.
- Every report records `device`, `tf32: false` and `CUDA_VISIBLE_DEVICES`.

**Parallel runner.** Each root is claimed exclusively by m-world (`create_dir`), so a root cannot be reused even if the script's pre-check races. The script refuses an unsealed root before it starts any job, and it fails any job that exits non-zero or leaves no manifest. A device refusal happens before the claim, so the job is logged as FAILED and no root is left behind.

**Non-blocking findings**
1. The runner treats any root with `manifest.json` as done. m-world also seals failed attempts (with `error.json`), so on a re-run a failed session is logged as `skip … (sealed)` instead of being reported. Suggestion: treat `$root/error.json` as a failure, or log it separately.
2. Two MODEL_ROOTs with the same basename map to the same `session-RECALL-NAME-TAG` root. The second one is refused at claim, which is safe, but both jobs write the same `$root.log`.
3. Evidence note: the builder's `parity.sh` timing lines are empty because `bc` is not installed on the pod. The quoted 278.0 s and 35.2 s figures come from the reports' `wall_seconds`, which I confirmed.

Parity is still measured behaviour on three models, and the PR's limitations remain: it is not proven for every model, and Metal was not run. My reports are in `/root/evalgpu/review-1742` (454 MB, mostly session checkpoints), kept for reference.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


### comment 5997229781 — 2026-10-05T15:07:40Z — Casey-allard

Exact-head check at eb36992f (merge of main incl. #1740 tier-v3 into eval-device).

- Laptop (no cuda feature): `cargo test --release -p uor-r4-training --bin chat-grade --example m-world` → 15 + 21 passed; `rustfmt --check --edition 2021` clean on chat-grade.rs.
- Pod 2×4090, `git archive eb36992f`, CUDA 12.8, `--features cuda`: same command → 15 passed, 21 passed.
- Conflict resolution: main's exclude/ill_posed/checks/constants and Judging kept; `device=` threaded through grade/reply → answer (the only model-loading path); device is parsed before the report root is claimed. CPU↔CUDA parity (226/226 session turns, 40/40 replies) was measured at 36825f2 before the merge; the merge does not touch the generation code.

Approve at this head.


---
_Comment/review ids in this file: 5996931308, 5997229781_

