# REPORT: d11-exporter-float-trunk, claude-140 (why I-Prefix failed)

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet: [d11-exporter-float-trunk-prefix-repeatability-2026-10-27.md](../../../../harness/prep/d11-exporter-float-trunk-prefix-repeatability-2026-10-27.md). References #2820. The item is not done.

## What did not work, first

- **The run shows that the training is nondeterministic, not where.** The same binary, run twice on one device, trains to different models. The float `atomicAdd` gradient sums in the FBS backward are the prime suspect (`spiralcore_fbs.cu:151`, `:178`; `spiralcore_fbs_arms.cu:114`, `:141`). But no arm isolates them, and candle's own CUDA kernels were not ruled out.
- **C2, C3 and C4 cannot be read.** Under outcome (a), same-device noise alone produces differences of the size seen across binaries and pods. So this run says nothing about host-source drift (b) or driver drift (c). This pod's driver, 580.178.04, also differs from claude-131's 595.80.
- **Earlier single-run D11 numbers carry this noise.** Five runs of one recipe give seed 1's step-1,024 float top-1 as 745, 1,472, 1,780, 2,914 and 3,845 of 14,287 rows. Any comparison of two single training runs in this setting, claude-131's and kimi-46's included, is within that spread. This audits the earlier readings; it changes no result row.
- **The pre-registration took three FIT rounds.**
  - Round 1: "source drift" was confounded with the toolchain.
  - Round 2: my binary-PTX "identity" extract held only kernel signatures, which was my slip. It was withdrawn, and the claim now rests on the source, the lock, runtime identity and the `nvcc` evidence.
  - The pre-run review then caught that the job never created the arms' save-dir parents. R1 would have failed at once.
- **The first `uor-pod down` refused to verify the archive over SSH,** as it did for kimi-46. The results were confirmed in the store by listing and by reading back two files, then `down --confirm-archived` deleted the pod.

## What changed

Size: 3 files of docs plus this task folder, with no code.
- The prep sheet;
- one notebook row in `docs/research/notebook/m4-serving.md`;
- this ORDER and REPORT.

The job script is local (`local/claude/claude-140/job.sh`). Its sha256 on the pod was `281163ec…`, the same as the local copy.

## Measurements

**Run.** One RTX 5090, pod `vast-57907957`: driver 580.178.04, CUDA 12.8.1, nvcc 12.8 V12.8.93. Pod life was about 27 minutes at $0.56/h, about $0.25.
- **The arms**, each 1,024 dual-forward steps on seeds 1 and 2, about 7 minutes:
  - R1 and R2: kimi-46's archived `spiralcore-fbs-sheet` `804101684d2a…`;
  - R3: claude-131's `a57a752d7dd3…`.
- **Instruments.** All pass. In the job these are `bin-pins`, `inputs`, `gpu`, `nvcc` and the per-arm `env`:
  - I-Bin;
  - I-Inputs (the five claude-131 pins);
  - I-Device;
  - I-Complete (2 seeds × steps 0, 256, 512 and 1,024 per arm);
  - I-Step0 (step 0 identical in all three arms and in both archived receipts).
  - The per-arm `UOR_`/`CUDA_` environment is the same for all three arms: none of the CUDA debug flags, and `CUDA_VISIBLE_DEVICES=0`.

**C1: R1 against R2, the same binary twice on one device.** Each cell is the float top-1 hits for R1/R2, then the largest |Δ DEV loss|.

| Seed | Step 256 | Step 512 | Step 1,024 |
|---|---|---|---|
| 1 | 1,934 / 1,553; 0.0558 | 2,483 / 3,197; 0.0179 | 1,780 / 2,914; 0.0118 |
| 2 | 1,354 / 1,776; 0.0176 | 2,024 / 2,805; 0.0212 | 3,192 / 3,375; 0.0713 |

The training losses differ at every trained point too. Seed 1 at step 1,024 is 3.6156 in R1 against 3.5816 in R2, and seed 2 is 3.5554 against 3.6937.

The bit-level reading agrees. Every `model.safetensors`, `snapshot.json` and `preds-float.json` differs between R1 and R2 on both seeds (`model-sha256.out`). The `config.json` files are identical.

**C2–C4, reported, not interpreted.** All three differ at every trained point on both seeds. At step 1,024 the float top-1 values are:

| Comparison | Seed 1 | Seed 2 |
|---|---|---|
| C2 (R1 vs R3) | 1,780 / 745 | 3,192 / 4,482 |
| C3 (R3 vs claude-131) | 745 / 3,845 | 4,482 / 3,128 |
| C4 (R1 vs kimi-46) | 1,780 / 1,472 | 3,192 / 3,258 |

**Outcome: (a) same-device nondeterminism** (sheet §3, first row). Exact equality across runs cannot be reached. So I-Prefix as written is an instrument fault, not evidence about the 2,048-step dose.

**Evidence.** The results are sealed at `results/claude/d11-prefix-repeatability-2026-10-27/`: 65 files, and a SHA256SUMS of 64 entries that verified on the laptop. The store read-back of `SHA256SUMS` and `r2-train.json` matched byte for byte. The saved models stayed on the pod; their hashes are in `model-sha256.out`.

## Checks run at the head

See the exact-head review record on the PR, which covers `loop report --check`, `loop pr-check --local` and the required checks.

## Skills used

- **holding-the-goal.** The question was the cause of a red gate, not whether a nicer number could be found. So the gate was not touched, and the diagnosis was pre-registered with a decision table before the run.
- **solution-integrity.** Each FIT and pre-run finding was fixed rather than argued around. That includes withdrawing my own PTX evidence when it turned out to be signatures only.

## Still open

- **The next unit, from the sheet's row (a):** make the FBS backward reduction deterministic, with no change to the math, for example a fixed-order per-atom reduction in place of the float `atomicAdd`. Then repeat C1 to show that two runs match bit for bit. The other option is an I-Prefix that compares points within one run.
- **Before deciding,** a cheap localising run would confirm whether the four atomic sites are the whole source, as opposed to candle's kernels. Either way it needs its own FIT.
- **Comparisons of single training runs** in this setting need either deterministic kernels or repeats with a stated spread. That applies to the export gap kimi-46 set out to measure.
- **#2820 stays open.** No export-gap, serving or close condition is met by this unit.

## Plain summary

The model's training gives different results each time it runs, even when nothing changes: the same program, the same data, the same GPU, twice in a row. One likely reason is that the GPU adds up some numbers in whatever order its threads finish, and a tiny difference grows over a thousand training steps. That is why the earlier safety check could never pass. It wanted two training runs to match exactly. The next step is to make that adding-up happen in a fixed order, so runs can be compared fairly.
