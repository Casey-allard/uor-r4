# REPORT: one-model-end-to-end, claude-333

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). ORDER: [ORDER.md](ORDER.md). Sheet: [one-model-end-to-end-carry-hold-2026-11-09.md](../../../../harness/prep/one-model-end-to-end-carry-hold-2026-11-09.md).

## What did not work, first

- **Unit B is not built.** The ORDER planned both code units here. This PR lands only Unit A, the hold.
  - Unit B is `−aS`, `−H`, the six value ranks over the found rows, the found fraction, and logging of `g` and `|w_y|` at every checkpoint, with I-H5 and I-V.
  - It needs a capture of the stream read's internal vectors, inside `recall_readings` (`in_space_long.rs:12556`). That is a unit of its own, so it goes to claude-334.
- **No CUDA test has run yet:** I-H1, I-H2, I-H3 and the I-1 digests.
  - **What they are:** forward and gradient tests. The laptop refuses CPU compute (D24/D29), so they run on the GPU.
  - **What was checked:** they type-check against the stub nvcc (G2), and each CUDA mutation matches its code exactly once.
  - **What is owed:** running them, and their mutations, on a pod. That goes to claude-334, together with Unit B's own CUDA tests, before the FIT. The I-1 claim, that the default paths are unchanged, stays open until then.
- **No FIT was done and no GPU was used.** The FIT reads the sheet with all the code, so it waits for Unit B.

## What worked

- **`--hamming-hold h`, behind a default-off flag** (`flock_read_hamming.rs`, `HammingReads::with_hold`):
  - **The held term:** the stream read's ask write `u = w_y ⊙ y` is carried to the next `h` rows of its own window, as `Σ_{s=1..h} g[x_{t−s}] · u_{t−s}`.
  - **Where it sits:** it is added inside the stream read's combine, before the `q16` rounding at `geometric_stack_native.rs:1693`.
  - **The gain `g`:** one learned scalar per token, `(vocab,)`, starting at 0, at the stream read only. It is trained with the read's other parameters, through `named()`.
  - **Mechanism checks:** no dense product (a scalar lookup, shifted adds and a lane-wise scale) and no squashing (§3.2, §3.3).
- **The flag (`in_space_long.rs`, `bin/train-in-space.rs`):** it requires `--hamming-ask 1`. It is left out of `config.json` when off. A checkpoint's hold must match the run's, both ways.
- **Saving:** `d2.hold` plus `hold_rows`. Loading refuses each of these:
  - a gain with no row count;
  - a row count with no gain;
  - a hold without the ask heads;
  - a hold at the readout depth.
- **Refusals:**
  - **The decode step** refuses a held read before any arithmetic.
  - **The decode cache** refuses it as a read hook, so a held read decodes uncached and is not turned into an error.
  - **Both export paths** (`fbs_hamming_snapshot`, `round_hamming_as_exported`) refuse it by name, before their ask-heads refusal.
- **Host tests pass,** and each of their mutations fails (see Checks below). They cover I-H4 and the flag:
  - zero start;
  - the save and load round-trip;
  - the four load refusals;
  - the decode and export refusals;
  - the flag's refusal, its start and its checkpoint match.

## What changed

- `crates/uor-r4-float-train/src/flock_read_hamming.rs`:
  - the hold: `Hold`, `held_term`, `with_hold`, `hold_rows`, save and load, and the decode refusal;
  - I-H4 (host);
  - I-H1, I-H2, I-H3 (CUDA);
  - two new I-1 digests (CUDA): one with the ask heads and no hold, and one no-read forward of the six-layer trunk.
  - The I-1 block's header now says the pod replaces the base's block, BEGIN to END. claude-330's digest function inside it is unchanged.
- `in_space_long.rs`: the `hamming_hold` config, its start and checkpoint checks, and its test.
- `bin/train-in-space.rs`: the `--hamming-hold` flag and its docs.
- `fbs_export.rs`: the two refusals.
- `geometric_stack_native.rs`: the decode cache's refusal.
- **Not changed:** no bar, status, scope or cap. No notebook row, since there is no result.

## Measurements

None. No training or evaluation ran.

## Checks run at the head

- **G1:** `cargo test -p uor-r4-float-train --lib`: 326 passed, 0 failed.
- **G2:** the stub-nvcc `cargo check --features cuda --lib --bin train-in-space --profile test`: finished, with no diagnostics in this crate.
- **G3, the host mutation gates** (`local/claude/claude-333/host-gates-333.sh`): 9 of 9 fail as they must. Each mutation matched once, its test compiled, ran and failed (cargo exit 101, FAILED). The baseline passed before and after, and the tree was clean after. The mutations: both export refusals, the decode refusal, the zero start, the saved row count, the depth-6 load refusal, the flag's refusal, the checkpoint match, and leaving the flag out of `config.json`.
- **G4:** the banned-token scan of the added lines: 0 hits.
- **G5:** claude-330's digest function is unchanged in the diff. It runs on the pod in claude-334.

## Skills used

None beyond the loop's own steps.

## Still open

- **claude-334 (the lab decision, logged here for Casey).** It is within this ORDER's two-unit bound. In order:
  1. Unit B.
  2. One short pod: every CUDA test of both units and their mutations, and I-1 at the base and the head.
  3. The fresh FIT of the sheet and the code together.
  4. If FIT, the D67 pre-run review and the two-seed run.
- **Carried to claude-334:** the four unverified facts in `local/claude/claude-332/for-333.md`.
  - The nearest readout compares the final point against raw `p_v`.
  - `rate_select` reads `RATE_TABLE`.
  - I-R2's sealed numbers.
  - (This unit checked the fourth, that `sources[1]` is the raw table for the ask heads: `flock_read_hamming.rs`, `sources`.)

## Plain summary

This unit wrote the hold: the piece meant to carry the fact the model finds at the question forward to the answer a few words later. It is switched off unless asked for, and switched off it changes nothing. It starts at zero, so even switched on it begins as a no-op and has to learn its way in. The parts that need a GPU to test are written and compile, but they have not run yet. The measurements that would tell us whether it works are the next unit's job. That unit also has the sheet and the code reviewed together, and only then spends about $1 on the run.
