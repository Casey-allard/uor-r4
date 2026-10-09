# Result (C): S2 Dialogue Quantization-Aware Training (QAT) & Behavior Recovery

September 29, 2026. References #973 under programme tracker #820.
- **Lab:** Anti-Gravity (Lab 3: Numerical Fidelity, Codecs, Discretization, Serving Kernels & Execution Audits).
- **Lane:** Local M1 cost, zero-multiplier kernels, discrete codecs.
- **Owner Policies:** D0-b, D11, D12.

---

## 1. Protocol & Pre-Registered Arms

- **Pre-Registration:** GitHub issue #973 ([comment 5890372137](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5890372137)).
- **Init Model:** Claude S2 dialogue baseline at `/Volumes/UOR-Workspace/uor-r4-lab/claude-s2-dialogue-baseline/dialogue-1/model`:
  - `model.safetensors` SHA-256: `8cb11d8f12ccfd0b3bf085fa7b396d409c26bc039ee3cb660a384ea9d3300938`
- **Pre-Registered Arms:**
  - **Arm 1 (QAT):** 1,024 updates, lr $0.0002$, warmup 50, batch 16, straight-through estimator gradients through the D11 interim 4-bit round-to-nearest representation (`qat=true`).
  - **Arm 2 (Float Continuation Control):** Identical schedule, seed, optimizer, and data with `qat=false` (measures fine-tuning drift alone).
- **Data & Tokenizer Checksums:**
  - Tokenizer: `bundle-quaternion-1/tokenizer.json` (SHA-256 `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`)
  - Evaluation Requests: `development-requests.json` (SHA-256 `81268b51ef98d8d8e525a40e572538249ef845e321571ee55dec30fb8ef75484`)
  - Train Tokens: `chat-v0/prepared/train/tokens.u16` (SHA-256 `4a554b0ef8be12344f21f1bd6bdc9faeeeddcef212a138a8772a8bf42604c4fa`)
  - Train Mask: `chat-v0/prepared/train/response_mask.u8` (SHA-256 `09dc0fe5d0e2de7b5072e4586b5cb9ef3379856600ff90061c561a48826e75e7`)
  - Held-out Tokens: `chat-v0/prepared/heldout/tokens.u16` (SHA-256 `ca5ccf0722d9dcdc5294b612f2c25536fe49192be7bedbc3007a1fcfc0940225`)
  - Held-out Mask: `chat-v0/prepared/heldout/response_mask.u8` (SHA-256 `ed43a9eade79d84dc73be608fb788f1e7566021fbdbba35ce3e5eddbc4fce8b4`)
- **Executable:**
  - `geometric-stack`: SHA-256 `bd3040c0f6dd9bd2fbd33f159db6dd8a9f5847c40f50902e4de410b0c9cbcc39`

---

## 2. Sealed Report Roots

### Arm 1 (QAT: `s2_dialogue_qat_1`)
- `train`: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/s2_dialogue_qat_1/train` (Manifest SHA-256 `ecd46f59272021731c653299a5c18fb3a789bf8c6b7429d8ade8995754b8ff6e`)
- `export`: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/s2_dialogue_qat_1/export` (Manifest SHA-256 `405afb1b275ef324aaebe7fbbc9856efaea3b9a0ccf60ee7b7382f4f4a0a734f`)
- `chat`: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/s2_dialogue_qat_1/chat` (Manifest SHA-256 `51ee14e604f94087df0f52bc87d07938360c97d57d144e7833a3dba7ae0ad148`)
- `d11_eval`: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/s2_dialogue_qat_1/d11_eval` (Manifest SHA-256 `3e3c2125cb63cd35aafbd949619dd0b9c60e0d9872e84265317daee6afaa4cdc`)

### Arm 2 (Float Control: `s2_dialogue_float_ctrl_1`)
- `train`: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/s2_dialogue_float_ctrl_1/train` (Manifest SHA-256 `b7afbd454d395344d0169794742e3cd71482994e1b456d5fe362aa7b2b214571`)
- `export`: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/s2_dialogue_float_ctrl_1/export` (Manifest SHA-256 `fde396a668a5ff977d72a72226c7527d9ca0bdfdacca7869a4e24f7f0fb0ecc6`)
- `chat`: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/s2_dialogue_float_ctrl_1/chat` (Manifest SHA-256 `735f8b6562178991d87b2b143bef4cd77f3fe3a654b1c0aeb8a93b19960b2679`)
- `d11_eval`: `/Volumes/UOR-Workspace/uor-r4-lab/anti-gravity-d4/s2_dialogue_float_ctrl_1/d11_eval` (Manifest SHA-256 `333dc1ef5101e0457719027914d99d8184e06955d01091f213806422eb111faa`)

---

## 3. Measured Results & Paired Comparison

| Dimension | Baseline S2 (PTQ 4-bit) | Arm 2: Float Control (qat=false) | Arm 1: QAT Result (qat=true) | Target Gate | Outcome |
|:---|:---:|:---:|:---:|:---:|:---:|
| **161-Response NLL (Float)** | 2.5209 nats (parent) | **2.5649 nats** | **2.5648 nats** | — | Paired match |
| **161-Response NLL (Served)** | 2.5415 nats (PTQ) | **UNAVAILABLE**<br>*(diagnostic root `2.5883` retracted)* | **2.5680 nats** | **Literal target:** $\le 2.5255$ nats ($+0.0300$ nats of cited parent $2.4955$ nats; local receipt UNAVAILABLE)<br>**Corrected parent target:** $\le 2.5509$ nats ($2.5209 + 0.0300$ nats) | **Reported; Lab 1 re-run pending**<br>Relative $\Delta = \mathbf{+0.0032\text{ nats}}$ vs Arm 2 float control ($2.568047$ vs $2.564863$).<br>Absolute served NLL misses both targets ($+0.0425$ nats over literal $2.5255$; $+0.0171$ nats over corrected $2.5509$) due to base float drift. |
| **Quantization Gap ($\Delta$ NLL vs Matched Float)** | $+0.0206$ nats (initial PTQ vs parent) | **UNAVAILABLE**<br>*(served receipt not in Arm 2 root)* | **$+0.0032$ nats** | $\le +0.0300$ nats | **Reported; Lab 1 re-run pending** ($9\times$ margin vs matched float control) |
| **Kernel Agreement (Integer vs Served Forward)** | 47 / 58 turns (81.03%)<br>*(cited pre-registration, PR #1470)* | N/A (float control has no integer kernel; PTQ export matched only 5/58 (53/58 diverged)) | **56 / 58 turns (96.55%)**<br>(only 2 diverging turns) | Diagnostic | **Reported; Lab 1 re-run pending** (Kernel Agreement: integer engine matches served forward simulation) |
| **Greedy Agreement vs Pre-Adaptation Float Baseline** | 14 / 58 turns (24.14%)<br>*(cited pre-registration, PR #1470)* | 7 / 58 turns (12.07%)<br>(unquantized float control vs initial float parent) | 5 / 58 turns (8.62%)<br>(QAT integer vs initial float parent) | $\ge \mathbf{29 / 58}$ turns (50.0%+) | **MISSED** (5 / 58 turns vs pre-registered $\ge 29/58$ target; unquantized float control itself drifted to 7 / 58 turns) |
| **Greedy Agreement vs Matched Float Control** | — | — | 6 / 58 turns (10.34%) | — | Characterized (QAT integer vs Arm 2 float control) |
| **QAT Integer vs Own Float Weights (`with_float_forward`)** | — | — | **MISSING / UNAVAILABLE** | — | Unresolved: QAT integer vs own unquantized float was not evaluated |
| **D11 vs D10 Discrepancy** | `diff = 0` | `diff = 0` | `diff = 0` | Bit-identical | **PASS** (Bit-identical) |
| **Held-out Window NLL (valid.u16)** | 2.9805 nats<br>*(cited `claude-s2-dialogue-baseline/d11-eval-1`, 16 windows)* | 2.9842 nats<br>*(64 windows)* | **2.9717 nats**<br>*(64 windows)* | — | Improved |
| **Raw Parameter Bit Budget** | 4.2500 bpw | 4.2500 bpw | **4.2500 bpw**<br>*(structural format: 4-bit codes + 1 scale byte/32)* | $\le 4.2500$ bpw | **PASS** |

---

## 4. Decisive Empirical Conclusions

1. **Kernel Agreement Between Integer Engine and Training Served Forward (56/58 turns):**
   The **56 of 58 turns (96.55%)** metric measures **kernel agreement** between the exported native D11 integer serving engine (`lut-chat`) and the training served forward pass (`dialogue-train` with straight-through quantization). Only 2 turns diverge between the served engine and its training forward pass (compared to 47/58 turns in the pre-adaptation S2 baseline measured by the independent reader in PR #1470).
2. **Failure of Standard Post-Training Quantization (PTQ):**
   In Arm 2 (standard float fine-tuning without QAT), exporting the trained float model to 4-bit integer weights causes catastrophic greedy generation collapse: **53 of 58 turns (91.38%) diverge** from the model's own forward pass.
3. **Greedy Agreement with Float Baseline (Pre-Registered Gate MISSED):**
   Comparing generated replies against the pre-adaptation float parent (`8cb11d8f…`), Arm 1 integer replies match on **5 of 58 turns** (8.62%), while comparing against the matched fine-tuned float control (Arm 2) yields **6 of 58 turns** (10.34%). Both miss the pre-registered float agreement gate of $\ge 29 / 58$ turns (50.0%+), which was not achieved (**MISSED**). While the unquantized float control itself drifted to 7/58 turns over 1,024 updates, indicating significant policy drift during adaptation, the pre-registered gate outcome remains missed.
4. **Quantization Representation Gap and Target Bound Accounting:**
   The served representation penalty drops to $\mathbf{+0.0032\text{ nats}}$ relative to the fine-tuned float control ($2.5680$ vs $2.5648$ nats). However, against the pre-registered baseline target bound:
   - Literal pre-registration target: $\le 2.5255$ nats (based on cited parent $2.4955 + 0.0300$ nats from PR #1470 pre-registration; local receipt UNAVAILABLE).
   - Corrected parent target: $\le 2.5509$ nats ($2.5209156608 + 0.0300$ nats).
   The absolute served NLL of $2.5680$ misses both targets ($+0.0425$ nats above the literal target and $+0.0171$ nats above the corrected parent target) because the base float model drifted upward to $2.5649$ nats during the 1,024 update adaptation schedule.
5. **QAT Integer vs Own Float Weights Remains Missing:**
   The comparison of Arm 1 QAT integer output directly against its *own* unquantized float weights (`with_float_forward`) was not evaluated during the run and remains missing. Current comparisons against the pre-adaptation parent (5/58) and the separate float control (6/58) cannot substitute for an own-float evaluation. Consequently, we cannot infer that all difference from float is fine-tuning drift; some part may represent integer-vs-float divergence within the adapted checkpoint.
6. **Ledger, Opcode Scope & Parameter Access Accounting:**
   - Both Arm 1 ($2,180\text{ s}$) and Arm 2 ($1,401\text{ s}$) were executed within budget under model slot exclusive locks.
   - Cumulative ledger charged $3,581,000\text{ ms}$ total ($768,447,701 / 780,000,000\text{ ms}$).
   - **Opcode Scope:** These finite output comparisons evaluate model predictions and do not themselves prove whole-path opcode restrictions (zero mul/div/float) for this specific evaluation run; opcode certification references the D11 integer stack engine audit (PR #1467 for `uor-r4-stack`), while whole-path runtime opcode verification for this specific binary remains unverified without an exact-binary disassembly receipt.
   - **Parameter Access Scope:** The evaluation records 7,238,304 weights read per token (100% dense parameter access across all layers) and does not establish or claim D5 sparse selected-access compliance.
   - Status: **reported; Lab 1 re-run pending**.

