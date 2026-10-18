# Row-parallel exact counterfactual — finite GPU parity and timing (2026-10-18)

- **Line:** M1 discrete learning infrastructure (#2029). It is not a mechanism: no learning law, objective or read changes. **Lab:** claude.
- **Authority:** owner direction 18 October (~04:00Z, in Claude's session), "Build now, Dose 1 runs on". The faster counterfactual kernel is built ahead of the Dose 1 decision. It must give exactly the same losses, and the running Dose 1 is untouched. The owner also authorized the labs' own subagents to do peer review ("you can dispatch any subagents you need for peer review or adversarial tests").
- **Rules:**
  - D24: GPU only; this sheet runs no CPU model reference.
  - D23/D25/D32: integer CUDA only, enforced by the D29 checker on the generated module.
  - D22: this is an instrument parity test, not a mechanism verdict.
- **Source:** branch `claude/counterfactual-rows-20261018`, opt-in module `crates/uor-r4-discrete/src/rows_source.rs` (three entries), handle `rows_device.rs`, suite `rows_qualification.rs`, binary `rows-qualify`.

## Bar

**PASS requires every condition below:**
- every parity case in all four controls returns either the same 16-slot loss vector and the same chooser observation on both paths, or the same refusal text on both paths;
- learned state and epoch are unchanged after every case;
- each refusal-parity case is refused;
- at the Dose 1 timing shape, both paths return the same losses.

Any mismatch is a FAIL that stops adoption, for diagnosis (D31); it is never a mechanism negative. The timing numbers are reported only; there is no speed bar. A refusal or CUDA error at the timing shape is still a FAIL for diagnosis, because the Bar requires equal losses there.

**Racy refusals.** When the two paths refuse with different text, both re-run up to four more times, and the observed status sets must overlap. That is weaker than exact text equality, because both kernels race on `atomicExch` across lanes. The count is reported as `racy_refusals`. Making the row path the default needs a separate PR after a PASS.

## Base artifact

No checkpoint is loaded:
- each case builds a fresh seeded device (`Device::new_with_read`, seeds 7 and 11; seed 1 for timing) through the ordinary constructor;
- Dose 1's artifacts and pod are not touched.

## Data split

There is no corpus. Inputs are deterministic integer token identities and targets from a fixed LCG, with fixed seeds per control:
- **parity:** 1, 7 and 64 rows, with variable window lengths 2..12;
- **timing:** 64 full rows of 192 tokens.

No TRAIN/DEV/fresh data is read.

## Steps

1. Rent one RTX 5090 with `uor-pod up --lab claude`, `--ref` = this branch's pushed head.
2. Build the release binary on the pod: `UOR_NVCC_ARCH=compute_120 cargo build --release --locked -p uor-r4-discrete --features cuda --bin rows-qualify`. The build compiles all modules, admits the PTX inventories and assembles the cubins.
3. Run the D29 checker on the pod tree: `--self-test` and `--repo .`.
4. Run `rows-qualify RUNS/rows-parity-2026-10-18` once, then archive the report and logs off the pod.
5. Release the lease and take the pod down.

**Source-identity break (declared).** `device.rs` `source_sha256` hashes `lib.rs`, `build.rs` and every embedded CUDA image. This change edits the first two and binds the new rows image in its own versioned record, `uor-r4.rows-image/1`. The baseline-reuse and decomposition modules set that precedent. So any binary built after this merge reports a new `source_sha256`, and restoring a snapshot across that boundary is refused (`device.rs` snapshot check). Nothing currently in flight is affected: Dose 1 runs its own release binary pinned at source `23d5b146b`, and its checkpoints are consumed by that same binary. The training driver's `driver_source_sha256` is unchanged, because `main.rs` and the other hashed driver files are untouched and the suite is a separate `rows-qualify` binary.

## Stop rule

- **Stops for diagnosis** (D31 blockers, not negatives), with the pod kept only long enough to archive:
  - a build failure, a PTX admission failure or a D29 failure;
  - any parity mismatch or CUDA fault.
- **Hard stops:** owner caps (≤ 4 pods, ≤ $8/h) and the 30 GiB disk floor.
- **No retry with changed code** without a new review.

## Cost

One RTX 5090 at about $0.91/h for an estimated 0.5–1 h (bootstrap, build, suite, timing): about **$0.5–1**. It is charged to Claude's ledger, and nothing is spent on Dose 1's pod.

## Minimum test power

- **Parity sweep:** Route (2 and 4 blocks; all Route and VectorBlock parameters), Flock (kept 3 and 8) and Relation (kept 3/sweeps 1 and kept 8/sweeps 2), Gate over every gate index. Each runs on 2 model seeds × 3 batch sizes, which gives hundreds of proposal pairs, plus 3 refusal cases.
- **Terminal paths:** success and host refusals are exercised by construction. `rows_forced_terminal_parity` forces an all-ineligible batch (status 8 on both paths) and a lane break at row 0 followed by eligible rows (C8 geometry, `kept_cells = 1`; lanes +1 and −1 break, CURRENT stays finite). The report counts broken-lane successes and racy refusals. Kernel-level 9/4 are unreachable through the public API, because the host `parameter()` validates first; the reducer's replay of them is source-checked only.
- **Gap:** an early break followed by a later *error*, and status 12 overflow, are not forced. They are covered by the source-level ordering test (`rows_source.rs` `only_the_reducer_publishes_status`) and by the adversarial review.
- **Negative gate (on the pod):** delete the reducer's `|| (f&ROWS_TR_BIT)==0` clause, rebuild, rerun and require FAIL; then restore and rebuild.

TEST FITNESS: FIT — non-author reviewer subagent (owner-authorized peer review, 18 Oct), at head `97807e8e1`. It was conditional on three items, now resolved: (1) the `rows_qualification` declaration (present at `lib.rs:416` under `cfg(feature = "cuda")`); (2) `serde_json` being a normal dependency (it is); (3) the timing-failure wording added above. No compute or hold is released beyond this one finite parity and timing run.
