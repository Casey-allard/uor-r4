# Lab Record: `r-secant-surrogate-descent` (`#2886`) — Secant Surrogate Descent for Recurrence Gates and SpiralCore/FBS

- **Date:** 2026-10-25
- **Lab & Unit:** `antigravity` (`antigravity-77`, PR `#3225`, Issue [`#2886`](https://github.com/UOR-Foundation/uor-r4/issues/2886))
- **Pre-registration sheet:** [`docs/harness/prep/r-secant-surrogate-descent-2026-10-25.md`](../../harness/prep/r-secant-surrogate-descent-2026-10-25.md) (`TEST FITNESS: FIT`, D68 `DECISION REVIEW: APPROVE`, D67 `PRE-RUN REVIEW: PASS` — [#2886 comment `6400054852`](https://github.com/UOR-Foundation/uor-r4/issues/2886#issuecomment-6400054852))
- **Hardware:** 1 $\times$ NVIDIA GeForce RTX 5090 (`32607 MiB`, driver `595.99.02`, compute capability `12.0` / `sm_120`, `nvcc V12.8.93`, Vast pod `vast-57690399`)
- **Runner source commit:** `8ee5e945bf1d7b18eddf6d4a5a303f165e6891e8` (`crates/uor-r4-float-train`, `--bin secant-surrogate-descent`)
- **Compiled binary SHA-256 (byte-identical across two clean `nvcc` builds A & B):** `7c7d90ad988efe3f1824ae2a9453a0f5f4fb847bbf1c1a5f33d517c439008cb4`
- **Sealed receipt:** [`receipt.json`](./receipt.json) (`c5e7909dcaf88a7e2338eff6d5c1121a4f1abe751e56ec6d0dc70c3e0b0f5fb3`)
- **HF store archive:** `caseyallard/uor-r4-store:results/antigravity/r-secant-surrogate-descent-2026-10-25/` (`build-a/receipt.json` `c5e7909dcaf88a7e2338eff6d5c1121a4f1abe751e56ec6d0dc70c3e0b0f5fb3`, `build-b/receipt.json` `efdf70f994610d6504e9213bea1563dc486b50ffae12c3b56bea7dfc3895e21f`, `synthetic-probe.json` `ed013342590b1f31b4f0bdc1e15b7c5dbde2755665e6a1ccd9e34c3512fd0bd9`, `throughput-probe.json` `d7124ef3a881563ab9ac73d20c446ef5a9511bc1b3f18883e7e561fc53c5764e`)

Labels used below follow [`docs/formal_vocabulary.md`](../../formal_vocabulary.md): **proof** means derived from definitions or code, **measurement** means a recorded run or test, **hypothesis** means an untested idea.

---

## 1. Headline Verdict

- **Decode / step speed (lead metric, CHARTER §1):** `22.21–22.50 s` per 256-step arm (`~86.8–87.9 ms/step`, `~8,738–8,851` training tokens/s at `batch = 4`, `context = 192`, `768` tokens/step) on 1 $\times$ RTX 5090 (`sm_120`). This is an offline float micro-probe training throughput **measurement**, not a D11 integer serving decode speed.
- **Instrument checks (`I-Finite-CUDA`, `I-Boundary-Crossings`, `I-Keep-Plateau`):** **3 / 3 PASS** (**measurement**).
- **Part A (`G-Rec-Descent` — `RecGates::Geometric` 16-entry dyadic secant `rate_select_bwd`):** **PASS (mechanism descent)** on both seeds across both independent builds (**measurement**). Unblocks [`recurrence-gates-six-recurrence-fit`](../../roadmap/items/recurrence-gates-six-recurrence-fit.md) (`#2890`) and [`hamilton-gate-offline-float-sheet`](../../roadmap/items/hamilton-gate-offline-float-sheet.md) (`#2824`).
- **Part B (`G-Fbs-Descent` — `QuaternionWeights::SpiralcoreFbs` root-edge secant `spiralcore_fbs_bwd`):** **PASS (mechanism descent)** on both seeds, bit-for-bit identical across both independent builds (**measurement**). Unblocks [`spiralcore-fbs-offline-float-sheet`](../../roadmap/items/spiralcore-fbs-offline-float-sheet.md) (`#2891`).
- **Scope note:** This result answers only the 256-step offline float secant surrogate descent gate (`#2886`). It is **not** a full-rung trained model, **not** an integer export (`D11`), and **not** a language capability claim.

---

## 2. Pinned Inputs and Kernels

| Artifact | Path / Identifier | SHA-256 |
|---|---|---|
| Training tokens (`UORT`, `49,152` `u16` tokens = `256` windows $\times$ `192` context) | `w2-inputs/2026-10-25/driver-data/mw-balp/train/tokens.u16` | `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8` |
| Response mask (`UORT`, `49,152` `u8` mask bytes) | `w2-inputs/2026-10-25/driver-data/mw-balp/train/response_mask.u8` | `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3` |
| Tokenizer (`vocab_size = 4096`) | `w2-inputs/2026-10-25/driver-data/tokenizer.json` | `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` |
| Recurrence & stack CUDA kernels (`22` `__global__` kernels) | [`crates/uor-r4-float-train/src/cuda_stack.cu`](../../../crates/uor-r4-float-train/src/cuda_stack.cu) | `c577fce92ab965798857bd3f9d19ab86f65326dd501d8f8daf1537c0c9e9cb17` |
| SpiralCore/FBS CUDA kernels (`2` `__global__` kernels) | [`crates/uor-r4-float-train/src/spiralcore_fbs.cu`](../../../crates/uor-r4-float-train/src/spiralcore_fbs.cu) | `517bffcc3c2029ddd171e75fd8fc1eb532755b30e683c66bd230382416480b0d` |
| SpiralCore/FBS static compiler (`spiralcore-fbs-projection/1`) | [`crates/uor-r4-float-train/src/spiralcore_fbs.rs`](../../../crates/uor-r4-float-train/src/spiralcore_fbs.rs) | `6eb2b09d5602322b7628a83f4cd3c2ee35c59d78786cb349fd92c4519ebbe3cd` |

---

## 3. Probe Ladder Measurements

1. **Probe (a) — Synthetic & Control Verification (`--mode synthetic-probe`, 8 steps, seed 1):**
   - Verified on GPU before the full run (`synthetic-probe.json`, `ed013342590b1f31b4f0bdc1e15b7c5dbde2755665e6a1ccd9e34c3512fd0bd9`):
     - `RecSecant_Active`: $L_0 = 2.705537 \to L_8 = 2.682114$ ($\Delta L = +0.023423$, `rec_decay` crossings `28.65%`).
     - `RecZeroCredit_Null`: $L_0 = 2.705537 \to L_8 = 2.705537$ ($\Delta L = +0.000000$, `0` crossings).
     - `RecRandomRefit_Null`: $L_0 = 2.705537 \to L_8 = 2.704763$ ($\Delta L = +0.000774$).
     - `FbsSecant_Active`: $L_0 = 2.705537 \to L_8 = 2.558536$ ($\Delta L = +0.147001$, stage root crossings `0.90%`, output route flips `1.64%` in 8 steps).
     - `FbsZeroCredit_Null`: $L_0 = 2.705537 \to L_8 = 2.705537$ ($\Delta L = +0.000000$, `0` crossings).
     - `FbsRandomRefit_Null`: $L_0 = 2.705537 \to L_8 = 2.752058$ ($\Delta L = -0.046521$).
2. **Probe (b) — Throughput & Boundary-Crossing Projection (`--mode throughput-probe`, 16 steps on real corpus slice):**
   - `rec_16step_ms = 3233 ms` (`RecSecant_Active`: $L_0 = 2.811901 \to L_{16} = 2.801721$, $\Delta L = +0.010180$, `rec_decay` crossings `21.88%`).
   - `fbs_16step_ms = 3175 ms` (`FbsSecant_Active`: $L_0 = 2.811901 \to L_{16} = 2.764088$, $\Delta L = +0.047814$, stage root crossings `1.40%`, route flips `2.40%`).
   - Projected 256-step wall time: `51.73 s` per arm (`620.74 s` for 12 runs); actual full-suite wall time was even faster (`22.2–22.5 s` per arm, `267.7 s` total per 12-run suite).

---

## 4. Instrument Checks (`I-1..3`)

| Instrument | Requirement | Measured Value | Verdict |
|---|---|---|---|
| `I-Finite-CUDA` | Device `error == 0` on every `spiralcore_fbs_fwd`/`bwd` call (`25,344` calls/run); finite `MarginHinge` loss and finite nonzero $\|ds\|_2 > 0$, $\|dq\|_2 > 0$ at step 0 (`10,752` backward checks/run) | `25,344 / 25,344` CUDA error checks and `10,752 / 10,752` backward finiteness checks pass on all `12 / 12` runs (`304,128` CUDA checks total); step-0 $\|ds\|_2 \in [0.1695, 0.2340]$, $\|dq\|_2 \in [0.3145, 0.3621]$ | **PASS** |
| `I-Boundary-Crossings` (Part A) | $\ge 1.0\%$ of active recurrence decay parameters/lanes change discrete cell index $i \in \{0,\dots,15\}$ between step 0 and step 256 in `RecSecant_Active` | At `lr = 0.005`: Seed 1 crosses on `118 / 384` (`30.73%`) decay parameters (`5,130,285 / 18,874,368 = 27.18%` active lane-tokens); Seed 2 crosses on `136 / 384` (`35.42%`, Build A) and `133 / 384` (`34.64%`, Build B) decay parameters (`30.64%` / `30.65%` lane-tokens) | **PASS** |
| `I-Boundary-Crossings` (Part B) | $\ge 1.0\%$ of active `(output, stage)` queries change argmax root $r^* \in \{0,\dots,15\}$ between step 0 and step 256 in `FbsSecant_Active` | At `lr = 0.005`: Seed 1 crosses on `200 / 3,577` (`5.59%`) active `(output, stage)` queries and flips `155 / 1,584` (`9.79%`) output Clifford actions $A_o$; Seed 2 crosses on `192 / 3,573` (`5.37%`) queries and flips `152 / 1,584` (`9.60%`) output routes | **PASS** |
| `I-Keep-Plateau` | Track 16-bin histogram of $i = \min(\lfloor 16 s_t \rfloor, 15)$ at steps `0, 64, 128, 256`; if $> 80\%$ of lanes trap in $i \in \{12..15\}$ with $\|d\lambda\| < 10^{-7}$, trigger `rate_select_bwd_analytic_keep_f64` | Upper-plateau fraction ($i \in \{12..15\}$) stays in `38.44%–41.21%` across all arms and steps (`max_rec_plateau_12_15_fraction = 40.11%` in Part A), with step-0 mean $|d\lambda| \in [3.03\times 10^{-5}, 3.44\times 10^{-5}] \gg 10^{-7}$; `analytic_keep_secant_used = false` | **PASS** |

---

## 5. Full 256-Step Results Across Arms, Seeds, and Replicated Builds

Within each seed $s \in \{1, 2\}$, all six arms start from the identical deterministic model initialization and batch permutation, giving exact shared initial loss $L_0^{(a,1)} = 2.81190130$ (Seed 1) and $L_0^{(a,2)} = 2.84209054$ (Seed 2). Loss reduction is $\Delta L^{(a,s)} = L_0^{(a,s)} - L_{256}^{(a,s)}$ (positive means loss decreased).

### Part A: `RecGates::Geometric` (`layers.*.rec.decay` isolated, `384` parameters across 6 layers)

| Arm | Seed | $L_0$ | $L_{64}$ | $L_{128}$ | $L_{256}$ (Build A / Build B) | $\Delta L = L_0 - L_{256}$ (Build A / Build B) | Decay cell crossings | Step-0 $\|ds\|_2$ | Wall time |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `RecSecant_Active` | 1 | `2.811901` | `2.772116` | `2.763139` | `2.752162` / `2.752162` | **`+0.059740`** / **`+0.059740`** | `30.73%` (`118/384`) | `0.195944` | `22.35 s` |
| `RecSecant_Active` | 2 | `2.842091` | `2.809779` | `2.779414` | `2.763829` / `2.776757` | **`+0.078261`** / **`+0.065333`** | `35.42%` / `34.64%` | `0.234001` | `22.21 s` |
| `RecZeroCredit_Null` | 1 | `2.811901` | `2.811901` | `2.811901` | `2.811901` / `2.811901` | `0.000000` / `0.000000` | `0.00%` (`0/384`) | `0.169510` | `22.27 s` |
| `RecZeroCredit_Null` | 2 | `2.842091` | `2.842091` | `2.842091` | `2.842091` / `2.842091` | `0.000000` / `0.000000` | `0.00%` (`0/384`) | `0.192213` | `22.22 s` |
| `RecRandomRefit_Null` | 1 | `2.811901` | `2.810489` | `2.801911` | `2.815548` / `2.815761` | `-0.003647` / `-0.003860` | `24.74%` / `25.00%` | `0.196249` | `22.27 s` |
| `RecRandomRefit_Null` | 2 | `2.842091` | `2.836370` | `2.831928` | `2.844480` / `2.844480` | `-0.002389` / `-0.002389` | `26.04%` (`100/384`) | `0.210829` | `22.24 s` |

- **Two-seed noise band $\Delta_{\text{noise}}^{\text{rec}} = \max_a |\Delta L^{(a,1)} - \Delta L^{(a,2)}|$:**
  - Build A: $\Delta_{\text{noise}}^{\text{rec}} = |0.059740 - 0.078261| = 0.018522$.
  - Build B: $\Delta_{\text{noise}}^{\text{rec}} = |0.059740 - 0.065333| = 0.005594$.
- **Margin over best null control (`max(RecZeroCredit_Null, RecRandomRefit_Null) = 0.000000` on both seeds):**
  - Seed 1 margin: `$+0.059740 \ge \Delta_{\text{noise}}^{\text{rec}}$` ($3.23\times$ noise band on Build A; $10.68\times$ on Build B).
  - Seed 2 margin: `$+0.078261 \ge 0.018522$` ($4.23\times$ noise band on Build A) and `$+0.065333 \ge 0.005594$` ($11.68\times$ on Build B).
- **Verdict (`G-Rec-Descent`):** **`PASS (mechanism descent)`**.

### Part B: `QuaternionWeights::SpiralcoreFbs` (`layers.*.*.spiralcore_fbs` isolated, bit-identical across Build A and Build B)

| Arm | Seed | $L_0$ | $L_{64}$ | $L_{128}$ | $L_{256}$ (Build A = Build B) | $\Delta L = L_0 - L_{256}$ | Stage root crossings | Output route flips | Step-0 $\|dq\|_2$ | Wall time |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `FbsSecant_Active` | 1 | `2.811901` | `2.797597` | `2.768384` | `2.744842` | **`+0.067059`** | `5.59%` (`200/3577`) | `9.79%` (`155/1584`) | `0.362068` | `22.36 s` |
| `FbsSecant_Active` | 2 | `2.842091` | `2.807637` | `2.782146` | `2.738088` | **`+0.104003`** | `5.37%` (`192/3573`) | `9.60%` (`152/1584`) | `0.356593` | `22.34 s` |
| `FbsZeroCredit_Null` | 1 | `2.811901` | `2.811901` | `2.811901` | `2.811901` | `0.000000` | `0.00%` (`0/3666`) | `0.00%` (`0/1584`) | `0.362068` | `22.32 s` |
| `FbsZeroCredit_Null` | 2 | `2.842091` | `2.842091` | `2.842091` | `2.842091` | `0.000000` | `0.00%` (`0/3656`) | `0.00%` (`0/1584`) | `0.356593` | `22.29 s` |
| `FbsRandomRefit_Null` | 1 | `2.811901` | `2.826736` | `2.831293` | `2.788368` | `+0.023534` | `2.92%` (`106/3624`) | `5.24%` (`83/1584`) | `0.362068` | `22.37 s` |
| `FbsRandomRefit_Null` | 2 | `2.842091` | `2.855490` | `2.834987` | `2.861617` | `-0.019526` | `3.28%` (`119/3628`) | `5.81%` (`92/1584`) | `0.356593` | `22.35 s` |

- **Two-seed noise band $\Delta_{\text{noise}}^{\text{fbs}} = \max_a |\Delta L^{(a,1)} - \Delta L^{(a,2)}|$:**
  - $\Delta_{\text{noise}}^{\text{fbs}} = \max(|0.067059 - 0.104003|,\,0,\,|0.023534 - (-0.019526)|) = \max(0.036944, 0, 0.043060) = 0.043060$ (identical on Build A and Build B).
- **Margin over best null control on each seed:**
  - Seed 1 (`max(FbsZeroCredit_Null, FbsRandomRefit_Null) = +0.023534`): margin $= 0.067059 - 0.023534 = +0.043525 \ge 0.043060$ ($1.011\times$ noise band).
  - Seed 2 (`max(FbsZeroCredit_Null, FbsRandomRefit_Null) = 0.000000`): margin $= 0.104003 - 0.000000 = +0.104003 \ge 0.043060$ ($2.415\times$ noise band).
- **Verdict (`G-Fbs-Descent`):** **`PASS (mechanism descent)`**.

---

## 6. Reproducibility Command

On a CUDA host (`sm_120`, RTX 5090) with `w2-inputs/2026-10-25/` fetched from `caseyallard/uor-r4-store`:

```sh
UOR_NVCC_ARCH=compute_120 cargo run --release --locked -p uor-r4-float-train --features cuda --bin secant-surrogate-descent -- \
  --mode full \
  --tokens /workspace/secant-inputs/w2-inputs/2026-10-25/driver-data/mw-balp/train/tokens.u16 \
  --mask /workspace/secant-inputs/w2-inputs/2026-10-25/driver-data/mw-balp/train/response_mask.u8 \
  --tokenizer /workspace/secant-inputs/w2-inputs/2026-10-25/driver-data/tokenizer.json \
  --out /workspace/secant-out/receipt.json
```
