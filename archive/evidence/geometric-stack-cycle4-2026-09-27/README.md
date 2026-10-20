# Cycle 4 run packet: the geometric stack against the transformer control

Complete run records behind [the cycle-4 note](../../integration/geometric-stack-cycle4-2026-09-27.md). **Measured**,
development runs, one seed per arm. Nothing here is a final holdout or a language qualification.

## Contents

| Path | What |
|---|---|
| `pilot/<arm>_lr<rate>/report.json` | Each learning-rate pilot run (§5), with the fields listed below |
| `pilot/<arm>_lr<rate>/attempt.json`, `manifest.json` | The report root's claim record and seal manifest |
| `pilot/<arm>_lr<rate>/model/config.json` | The model configuration; the weights stay outside (below) |
| `pilot/launch.log`, `pilot/chain.log` | Launch times, the executable identity and the learning rates the rule chose |
| `ablation/<arm>/report.json`, `attempt.json`, `manifest.json`, `model/config.json` | Each seed-1 ablation run (§7): `dot`, `norot`, `readsonly`. The full-stack reference is `pilot/geometric_lr0.004` |
| `ablation/launch.log` | Source commit, executable identity, learning rate and start/end times of the ablation stage |
| `ablation-seed2/<arm>/report.json`, `attempt.json`, `manifest.json`, `model/config.json` | The seed-2 Lorentz and Dot stacks (§7), from the completed roots `*_r2`; the two interrupted roots per arm stay outside |
| `ablation-seed2/chain.log` | The pipeline's launches, exits and restarts, including both container restarts |
| `ablation-readsonly/<arm>/report.json`, `attempt.json`, `manifest.json`, `model/config.json` | The reads-only pair (§7): Dot with seed 1, Lorentz with seed 2 |
| `main/<arm>/report.json`, `attempt.json`, `manifest.json`, `model/config.json`, `train.log` | The main comparison (§6), `geometric_s1` and `transformer_s1`: the counted roots (`*_r4`, 7,324 updates), resumed from the 250-update checkpoint of `*_r1` |
| `main/<arm>-attempts/r1/` (`attempt.json`, `checkpoint-state.json`, `train.log`) | The first 250 updates and the checkpoint state the counted root resumed (its SHA-256 is the report's `resumed_from`) |
| `main/<arm>-attempts/{launch-1708,r2,r3}/` | Attempts that did not continue: the host-tuned build's launch that trapped at 17:39, a root stopped before its first checkpoint, and a root lost to a restart |
| `main/launch-1708.log`, `main/chain-c4.log`, `main/chain-c5.log`, `main/checkpoint-keep.log` | Launch identities, every launch, exit, restart and rollback of the comparison, and the copies of the last checkpoints (step 7,300) kept before completion removed them |
| `integer/<model>/export/export.json` (with `attempt.json`, `manifest.json`) | Integer serving export (§8): source model and executable identities, the artifact's size and SHA-256, and the quantization errors per matrix and per table of grid codes |
| `integer/<model>/evaluation/evaluation.json` (with `attempt.json`, `manifest.json`) | The integer engine beside the float model on the 512 final-evaluation windows: NLL per window, bits per byte, top-1 agreement and the engine's speed |
| `integer/<model>/steps64/evaluation.json` (with `attempt.json`, `manifest.json`) | The engine's step rate alone on 64 of the windows (commit `cb60c8bd`), apart from the f64 scoring loop |
| `integer/<model>/samples/samples.json` (with `attempt.json`, `manifest.json`) | The integer engine's greedy and sampled continuations of three development prompts (commit `cb60c8bd`), as token ids and decoded text |
| `integer/<model>/reference/evaluation.json` (with `attempt.json`, `manifest.json`) | Float, grid reference and integer NLL on the 512 windows (commit `feeef7e4`): the split of the integer gap into parameter representation (every exported value, not only the 4-bit matrices) and integer arithmetic |
| `integer/<model>/gptq-export/export.json`, `gptq-evaluation/evaluation.json` (each with `attempt.json`, `manifest.json`) | The GPTQ export (commit `6bc7dd6d`): calibration settings and cost, and each matrix's relative output error under GPTQ and round-to-nearest; then float, grid reference and integer NLL of that artifact on the 512 windows |
| `integer/run.log` | Start and end times and the executable identities of the integer stage |
| `sources/` | The launchers: pilot, post-pilot chain (with the selection rule), ablations, main comparison, the restartable `pipeline.sh`, the integer stage, and after the rollback `pipeline-post.sh`, `resume-main.sh` and `keep-checkpoints.sh` |
| `packet.json` | Per run: group, source commit, settings, threads, final metrics, cost and identities. Per integer model: float and integer NLL, agreement, speed and the artifact's identity, and under `gptq` the same for the calibrated artifact. Also the SHA-256 and size of every file in this directory |

Each `report.json` records:
- settings and the complete model configuration;
- parameter count, completed updates and target visits;
- every periodic evaluation, with training loss, development NLL, bits per byte, gradient norm, training seconds and throughput;
- the final evaluation, with 131,072 targets;
- three prompts' greedy and sampled continuations, as token ids and decoded text;
- the size and SHA-256 of every input;
- the executable's SHA-256 and the model's SHA-256.

## Kept outside the repository

- **Model weights.** Each run's `model/model.safetensors` is 28.6 MB. They stay in the lab sandbox's scratch directory, which is not durable. `packet.json` lists each file's size and SHA-256, which the reports also record.
  - The container restart of 2026-09-27 (~17:00 UTC) rolled the sandbox back to its 08:24 state. The weights of the seed-2 pair and the reads-only pair no longer exist; their reports and hashes here are the record. The pilot and seed-1 ablation weights survive in the sandbox.
  - The main pair's final weights and last checkpoints (step 7,300) are on the temporary branch `transfer/cycle4-main-20260928` for the owner's copy, with a SHA-256 manifest; the branch is deleted after the copy.
- **Integer artifacts.** Each `model.lut` is 4.5–5.0 MB. They stay beside the weights; `packet.json` and each `export.json` record their sizes and SHA-256.
- **Executables.** Release builds with `-C target-cpu=native`; any rebuild on another host differs in bytes.
  - The pilot used commit `dca1b790` (SHA-256 `fcf710ad…`).
  - The ablations used commit `b87acd63` (SHA-256 `05889874…`). Relative to `dca1b790` it changes only the example's sample decoding, its evaluate mode and early-stop checkpoint retention, not the model, optimizer or training loop.
  - The integer stage used commit `2a681bb7` (SHA-256 `cb0bb647…`), `cb60c8bd` (SHA-256 `3fed0f7c…`) for the step timing and continuations, `feeef7e4` (SHA-256 `20e265e5…`) for the rounding/arithmetic split, and `6bc7dd6d` (SHA-256 `6c1f4721…`) for the GPTQ exports.
  - `6bc7dd6d` alone was built without `-C target-cpu=native`. Its float scores reproduce every run's final evaluation exactly. Training with it is not bitwise equal to a native build, because the matrix products take other kernels. The same source built natively reproduces the native builds' model hashes.
- **Corpora.** The repository code split is cycle 3's (`b12707b0…` training, `3f7c50ef…` development), in the [cycle-3 packet](../native-lorentz-packet-2026-09-26/README.md). The registry corpus for the main comparison is `af93ca73…` (29,039,409 tokens), built with `geometric-stack corpus` and encoded with `geometric-stack encode`.

## Cost

- Six runs, two at a time, with two threads each, on the 4-core sandbox.
- Complete elapsed time per round, including periodic and final evaluation, checkpointing and sampling:

  | Round | Elapsed |
  |---|---:|
  | 1 | 67.5 min |
  | 2 | 59.9 min |
  | 3 | 73.8 min |
  | Total | 3 h 21 min |

- Training alone took 3,353–4,350 s per run.
- Rounds 1 and 2 shared the machine with builds and tests of this branch and of #1410.
- The ablations, claim to seal: `dot` 56.6 min and `norot` 57.5 min together, two threads each; then `readsonly` 46.5 min alone, four threads. Training alone took 2,739–3,386 s per run. Thread count changes speed and floating-point summation order, not the model or the update rule.
- The integer stage: 19 min for five exports and evaluations, one thread each, beside two training runs (09:28–09:47 UTC). An earlier start with an uncommitted build was stopped after 3 min and discarded.
- The GPTQ stage: 31 min for five calibrated exports and their evaluations with the grid reference, one thread each, beside the main comparison and builds (12:30–13:01 UTC).
