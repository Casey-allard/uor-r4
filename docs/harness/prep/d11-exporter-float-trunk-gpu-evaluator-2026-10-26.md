# Prep sheet: qualifying the GPU evaluator of a UORLUT01 FBS artifact (d11-exporter-float-trunk, claude-128), 2026-10-26

- Item: [d11-exporter-float-trunk](../../roadmap/items/d11-exporter-float-trunk.md) (#2820). ORDER: [claude-128](../../loop/tasks/d11-exporter-float-trunk/claude-128/ORDER.md).
- Run: `d11-fbs-eval` (`docs/labs/d11-exporter-float-trunk/instrument`, built by nvcc on the pod), in one job on one leased RTX 5090. The references it is held against, the host twin and the Rust integer session (`step_native`, the artifact's own serving runtime), run on the pod's CPU, as `fbs-compare` did in claude-126.

## 1. Question

**Does the CUDA evaluator compute exactly what the shipped artifact computes?** At every position, it must give the same token, the same kept cells and the same Q12 query, bit for bit, as the Rust integer session. That holds on the finite synthetic controls and on both trained Stage 4b artifacts. Only then can it score the artifact's own export gap (the next unit), with no emulation in between.

This qualifies an instrument. It measures nothing about the model.

## 2. Arms

There is one evaluator and two references. This is not a comparison of models.
- **G:** the CUDA evaluator, `fbs_eval_sequences` in the admitted PTX module, one thread per sequence, each from a fresh session.
- **H:** the host twin, the same `fbs_core.h` compiled by the pod's C++ compiler.
- **R:** the Rust integer session, `IntegerStackSession::step_native`.

## 3. Bar and decision table

**The bar:** G = H = R at every position of every input in section 5: the token, every kept cell in order, and every Q12 query value. One differing value at one position fails it. There is no tolerance and no near-tie exception, because all three are the same integer computation.

| Gate | PASS when | PASS unlocks | FAIL means |
|---|---|---|---|
| `Q-Synthetic` | `d11-fbs-eval qualify` exits 0 with `"pass": true`: 8 records (4 controls × single tokens and sequences), each with both checks equal | — | An instrument fault. Localize it from `first_difference`: G against H isolates the device build, and H against R isolates the port. Repair, then re-qualify. Not a negative. |
| `Q-Trained` | `d11-fbs-eval run … --verify` exits 0 on all four inputs (2 seeds × single tokens and DEV windows), with both checks equal | **With `Q-Synthetic`, the evaluator is qualified.** The next unit scores the artifact's own export gap on the panel with it, under an amended Stage 4b sheet. | As above. A trained artifact that disagrees where the synthetic controls agree is an instrument fault until it is localized (ORDER stop rule). |

**Readings, with no bar:**
- G's wall time per position on each input;
- **the projected wall time of the full 14,287-row panel.** Its eligible targets lie in claude-126's 428 DEV windows of 192 tokens (`d11-gap.json`: `dev_windows` 428). A session cannot skip ahead, so evaluating every position of every window covers every row: 82,176 positions × G's time per position on the DEV windows;
- the ptxas resource report (registers and stack per thread).

## 4. Instrument checks (each must be green before the bar counts)

1. **`I-Build`.** On the pod:
   - the PTX passes the build's integer admission (no floating-point, MMA or tensor token, and exactly the entry `fbs_eval_sequences`);
   - the cubin's target equals the device's capability (`sm_120`);
   - the host parity tests (`cargo test --release`) pass on the pod's own host twin.
2. **`I-Inputs`.** The artifacts and readouts match their sha256s (section 5) before any run.
3. **`I-Finite`.** Every step exits 0: no CUDA error, every sequence's status 0, and no panic. The job writes each step's stderr and exit code to `/workspace/evalq/` (`<step>.err`, `<step>.exit`), so a failure is localized from the saved files. Any exit other than 0 is an `I-Finite` fault.
4. **`I-Identity`.** Each output records the artifact's sha256, the PTX's and the cubin's. The binary run is copied to `/workspace/evalq-bin/` with its own sha256.

## 5. Base artifact and data split

**Base artifacts.** These are claude-126's Stage 4b exports, re-exported in claude-127 to the same bytes:
- seed 1: `results/claude/d11-fbs-export-gap-2026-10-26/d11gap/save/seed-1/artifact.uorlut`, sha256 `314c3909c66b58522a641c3f5b51c25dda4bda5ab2274bcfd0b5ca5038fdb305`;
- seed 2: `…/save/seed-2/artifact.uorlut`, sha256 `d5b435961ecf0a0038a814a107b6b2d17ea36eeef6fd4397d69b208b52dbdcde`.

**Data split.** Every input is fixed, and none is scored for quality.
- **The synthetic controls** of `uor_r4_integer::stack::synthetic`:
  - claude-122's small trunk (seed 20261025);
  - claude-127's non-square trunk and its tiny-gain trunk (seed 20261026);
  - the non-square trunk with a 64-position context (seed 20261027).

  Each runs every single token of its vocabulary, plus 16 sequences of its context length (drawn from seed 7). That is 3,328 positions in all.
- **The DEV windows** of claude-126's readouts: every eighth window, 54 windows of 192 tokens.
  - seed 1: `…/save/seed-1/readout-export.json`, sha256 `89385c4cf001576a9745f8eb5b03c3a58df792062d575043ca5dc1090a00d6e2`;
  - seed 2: `…/save/seed-2/readout-export.json`, sha256 `d322032a4d9bc69b08bcee7176f2c917e1600ef23424ff3c68ab9761754198c6`.

  The windows file is written by `d11-fbs-eval windows <readout> <windows.json>`. That command refuses windows of unequal length and prints their count and length, which must be 54 and 192. Every `run` output records the artifact's context, which must be at least 192, or the references refuse the windows.
- **The single tokens:** all 4,096, each from a fresh session, written by `d11-fbs-eval singles <artifact> <singles.json>`.
- **Hashes:** the sha256s of `singles.json` and both `windows-seed-<s>.json` go into the run folder's `SHA256SUMS`. Each `run` output records its own input's sha256 (`sequences_sha256`).

## 6. Steps and minimum test power

1. `uor-pod up --count 1 --gpu 5090 --ref <pushed head>`.
2. In `/root/build/src-<sha>/docs/labs/d11-exporter-float-trunk/instrument`, with `UOR_NVCC_ARCH=compute_120`:
   - `cargo test --release --locked --target x86_64-unknown-linux-gnu` (`I-Build`, host twin);
   - `cargo build --release --locked --target x86_64-unknown-linux-gnu --features cuda --bin d11-fbs-eval`.
   Copy the binary to `/workspace/evalq-bin/` with its sha256.
3. Fetch the four store files and check them against `SHA256SUMS` (`I-Inputs`).
4. `d11-fbs-eval qualify /workspace/evalq/qualify.json`.
5. `d11-fbs-eval singles <seed-1 artifact> singles.json`. Then, for each seed:
   - `d11-fbs-eval windows <readout> windows-seed-<s>.json`;
   - `d11-fbs-eval run <artifact> windows-seed-<s>.json <out> --verify`;
   - the same `run … --verify` on `singles.json`.

   Each step's stderr and exit code are saved.
6. Save `/workspace/evalq/` to the store at `results/claude/d11-fbs-gpu-evaluator-2026-10-26/`, with a `SHA256SUMS`.

**Minimum test power.** The bar is exact equality, not a statistic. It covers:
- 3,328 synthetic positions;
- 28,928 trained positions (2 seeds × (4,096 + 10,368));
- per position, a token, 8 kept cells and 256 query values.

A single differing value fails it.

**Host diagnostic, before the pod (not counted):**
- On the laptop, `tests/trained.rs` shows the host twin equal to the Rust session at all 14,464 positions of each seed's artifact (4,096 single tokens and 54 × 192 window positions).
- The finite host controls of ORDER done-when 2 pass there too.
- So the pod run tests the device build: G against H and R.

## 7. Cost and stop rule

**Cost:**
- one RTX 5090 on Vast, about $0.5/h, under a 1.5 h lease;
- about 10 minutes of bootstrap and build;
- seconds of GPU per input;
- minutes of CPU per reference per seed;
- in all, under $1.5.

**Stop rule:**
- Stop and save on any CUDA error or any mismatch. Localize from the saved output; do not re-run blindly.
- Never cancel a healthy run because the lease ran short. Renew it.
- Save everything off the pod before `down`.

## 8. Non-author TEST FITNESS review

TEST FITNESS: PENDING

**Round 1 (fresh non-author reviewer, 2026-10-26): FIT, with four required changes before the run.** Each is made above:
1. The single-tokens and windows inputs each get a command (`d11-fbs-eval singles` and `windows`) and a recorded sha256. The window length and the artifact's context are checked.
2. Every step's stderr and exit code are saved, and a nonzero exit is an `I-Finite` fault.
3. The projection covers the full 14,287-row panel (82,176 window positions).
4. The ORDER's launch shape now reads one thread per sequence (64 per block), as the evaluator runs it.

The reviewer found the bar decidable and hard to fool: `equal` needs both `first_difference` empty and whole-`Outputs` equality. The identity hashes bind the result to the binary. CHARTER §3.5 holds, because the references only qualify and the next unit scores with G alone.

TEST FITNESS: FIT
