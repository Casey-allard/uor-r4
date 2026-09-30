# D4-QAT: Measured Results for Quantization-Aware Training on Geometric S1

September 29, 2026. References #973 under #820.
- **Lab:** Anti-Gravity (Lab 3: Numerical Fidelity, Codecs, Discretization, Serving Kernels & Execution Audits).
- **Lane:** Local M1 cost, zero-multiplier kernels, discrete codecs.
- **Owner Policies:** D0-b, D11, D12.

---

## 1. Protocol & Sealed Roots

- **Pre-Registration:** GitHub issue #973 (comment 5888292954).
- **Parent Model:** `/Volumes/UOR-Workspace/uor-r4-models/investigations/cycle4-main-20260928/geometric_s1/model`
  - `model.safetensors` SHA-256: `3eb1ebbb3c1f65fccf9dfb325283f8f9892cd434e0d689356821acee2c5b1e0a`
  - Original float NLL: `1.998113` nats (512 windows).
- **Training Data:** `geometric_s1`'s exact training mix (`train_weights=1,1`):
  - `c3/data/code/train.u16` SHA-256: `b12707b012e5447f0c613236ea7f8819f2cc123861dafe49f7f8226ccc457137`
  - `c4/data/registry.u16` SHA-256: `af93ca73e51a798e4a19e1a470e97037939c95f7499e65ffb8e3526748026f65`
- **Validation Dataset:** `/Volumes/UOR-Workspace/uor-r4-models/investigations/cycle4-main-20260928/eval/valid.u16`
  - SHA-256: `3f7c50ef98fa707f4523aa3f7c6a97f496ee184fcfa125a84979fed9e7b91cb8`
- **Evaluator Protocol:** 512 evenly spaced windows of 256 tokens = 131,072 targets.
- **Training Hyperparameters:** 1,000 steps, batch 16 (context 256, 4,096 tokens/step, 4,096,000 total tokens), lr 0.0005, warmup 100, min_lr 0.1, weight_decay 0.1, clip 1.0, seed 1, threads 2.
- **Sealed Report Roots:**
  - Arm 1 (QAT): `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/geometric_s1_qat_d11_1000` (manifest SHA-256 `2c08ac78f6fbc1ffa361321f8315308cd57db031253dd86697b3f395947e06e3`)
  - Arm 1 Export: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/geometric_s1_qat_d11_1000_export` (manifest SHA-256 `9b6864fefcc0625c6fb5728110412e6bf7322e0049a96a14f187cdc733478c03`)
  - Arm 1 LUT Evaluation: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/geometric_s1_qat_d11_1000_lut_eval` (manifest SHA-256 `de6f640c16e9a88d4420e84eab6ad9e733aeba248a9281a5d793246517c602aa`)
  - Arm 1 D11 Evaluation: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/geometric_s1_qat_d11_1000_d11_eval` (manifest SHA-256 `ab550d4cce7fc514077b671044f4a229dc97eb28b633aa5dfcf2e3fb3c668fba`)
  - Arm 2 (Float continuation control): `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/geometric_s1_float_ctrl_1000` (manifest SHA-256 `1821dda2ab2ad006a373f30e2f4bff8c6ca650c1801d9ff5fd3a8a90d5550e07`)

---

## 2. Measured Results Summary

| Metric | Original Float Reference | Arm 2: Float Continuation Control | Arm 1: QAT Own Float Reference (`f4caf562...`) | Arm 1: QAT Served Forward | Arm 1: Exported Integer Engine (LUT / D11) | Gate Requirement | Status |
|:---|---:|---:|---:|---:|---:|:---:|:---:|
| **NLL (512 windows, 131,072 targets)** | 1.998113 | 1.955115 | 1.962490 | 1.973126 | **1.973127** | $\le \text{Arm 2} + 0.02$ (1.975115) | Reported; Lab 1 re-run pending (+0.018012 nats vs Arm 2) |
| **Quantization Gap vs Own Float** | — | — | 0.000000 | +0.010636 | **+0.010637** | $\le +0.020000$ nats | Reported; Lab 1 re-run pending (+0.010637 nats vs own float) |
| **NLL vs Original Pre-Adaptation Float** | 0.000000 | −0.042998 | −0.035623 | −0.024987 | **−0.024986** | $\le 2.018113$ ($+0.02$ nats) | Reported; Lab 1 re-run pending (−0.024986 nats) |
| **Integer Arithmetic Gap ($\Delta$ vs Served)** | — | — | — | — | **$7.79 \times 10^{-7}$ nats** | $\le 1.0 \times 10^{-5}$ nats | Reported; Lab 1 re-run pending ($7.7924 \times 10^{-7}$ nats) |
| **D11 vs D10 NEON Discrepancy** | — | — | — | — | **0** (`max_abs_logit_diff`) | 0 | Bit-identical |
| **Top-1 Agreement (vs QAT Own Float Model)** | — | — | 100.00% | 90.28% | **90.28%** | — | Measured (vs Arm 1 own float `f4caf562`, not Arm 2) |
| **Bits Per Weight (Raw Parameter)** | 32.0000 | 32.0000 | 32.0000 | — | **4.2500** | $\le 4.25$ bpw | Standard 4-bit |
| **Bits Per Weight (Total Container)** | — | — | — | — | **4.7234** | — | 4,969,988 bytes / 8,417,664 weights |

---

## 3. Fidelity Gate Evaluation

1. **Gate 1 (Primary D4 Representation Gate): Exported Integer NLL $\le$ Own Float + 0.0200 nats**
   - Arm 1 QAT Own Float NLL (unquantized float forward on checkpoint `f4caf562...`): **1.962490** nats.
   - Bound: $1.962490 + 0.020000 = 1.982490$ nats.
   - Arm 1 Exported Integer NLL: **1.973127** nats.
   - Quantization gap vs own float: **+0.010637 nats** ($\approx +0.0106$ nats), passing the representation gate with $0.009363$ nats margin.
   - *Note on representation penalty:* The 4-bit integer engine is $+0.0106$ nats worse than its own unquantized float weights due to quantization loss, which is well within the pre-registered $\le 0.0200$ nats budget.
   - **Verdict: Reported; Lab 1 re-run pending**.

2. **Gate 2: Exported Integer NLL $\le$ Float Continuation Control + 0.0200 nats**
   - Arm 2 (Float continuation control): 1.955115 nats.
   - Bound: $1.955115 + 0.020000 = 1.975115$ nats.
   - Arm 1 Exported Integer NLL: **1.973127** nats.
   - Gap vs continuation control: $+0.018012$ nats.
   - **Verdict: Reported; Lab 1 re-run pending** (margin: $0.001988$ nats).

3. **Comparison vs Original Pre-Adaptation Float Reference (1.998113 nats)**
   - Bound: $1.998113 + 0.020000 = 2.018113$ nats.
   - Arm 1 Exported Integer NLL: **1.973127** nats.
   - Gap vs pre-adaptation float: **−0.024986 nats** (due to joint QAT adaptation, the 4-bit integer engine achieves lower NLL than the unquantized baseline checkpoint before adaptation).
   - **Verdict: Reported; Lab 1 re-run pending**.

4. **Integer Serving Parity: Integer Engine equals Served Forward $\le 10^{-5}$ nats**
   - Served forward NLL (training): 1.97312575 nats.
   - Exported integer engine NLL (D11 / LUT): 1.97312653 nats.
   - Measured gap: $7.7924 \times 10^{-7}$ nats (`lut_eval_integer_arithmetic_nll`).
   - Multiplier-free D11 vs D10 NEON: `max_abs_logit_difference == 0`, `top1_agreement == 1.000000`.
   - **Verdict: Reported; Lab 1 re-run pending** (bitwise D11/D10 agreement verified).

---

## 4. Decision & Next Steps

- **Decision:** **Reported; Lab 1 re-run pending**. Quantization-Aware Training (QAT) through the straight-through estimator hook with the D11 interim codec is reported as a candidate fidelity method for closing the representation gap on the geometric stack.
- **Next Step:** Per programme directive, having reported (B), proceed to (C) Behavior: evaluate QAT on the S2 dialogue stack (`dialogue-train init=... qat=true`) on S2's chat-v0 data to measure native greedy reply agreement and 161-response NLL against the unquantized float and baseline 4-bit export references.
