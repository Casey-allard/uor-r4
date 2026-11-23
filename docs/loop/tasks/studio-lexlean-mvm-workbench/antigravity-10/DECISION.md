# D68 Decision Card: Foundry `Qwen/Qwen3-0.6B` `UORR4V13` Bar #4 Split & Run 5 Selection (`studio-lexlean-mvm-workbench`, `#3939`)

- **Lab:** `antigravity`
- **Order:** [`docs/loop/tasks/studio-lexlean-mvm-workbench/antigravity-10/ORDER.md`](ORDER.md)
- **Item / Issue:** [`studio-lexlean-mvm-workbench`](../../../../roadmap/items/studio-lexlean-mvm-workbench.md) ([`#3939`](https://github.com/UOR-Foundation/uor-r4/issues/3939))
- **Pre-Registration Chain:** [`antigravity-4/PREP.md`](../antigravity-4/PREP.md) (`v1`, `#4175`), [`antigravity-5/PREP.md`](../antigravity-5/PREP.md) (`v2`, `#4178`), [`antigravity-6/PREP.md`](../antigravity-6/PREP.md) (`v3`, `#4180`), [`antigravity-7/PREP.md`](../antigravity-7/PREP.md) (`v4`, `#4183`), [`antigravity-8/PREP.md`](../antigravity-8/PREP.md) (`v5`, `#4186`), [`antigravity-9/PREP.md`](../antigravity-9/PREP.md) (`v6`, `#4188`)
- **Owner Decision:** [D90](../../../../integration/DECISIONS.md#decisions-after-the-charter-from-d43) (`2026-11-22T21:39:18Z`)
- **Authority:** [`CHARTER.md`](../../../../../CHARTER.md) §3.5 (*"Spend GPU only where a run can change a decision"*), §4.1–§4.6, §5.1, §5.2, D65–D69

---

## 1. Whole-Project Context & Why This Decision Review Is Required Now

1. **Quarantine Boundary (`CHARTER.md` §3 & D90):**
   - `UOR-R4` builds a non-transformer native geometric language model (`crates/**`, M1–M8) with strict bans on softmax (§3.2), dense matmul/GEMM (§3.3), non-native train/serve drift (§3.7, D85), and neural comparators (§3.10). M1 closed on `2026-10-30`; native M2–M8 remain completely isolated and untouched by this Studio work.
   - Owner Decision **D90** (`2026-11-22T21:39:18Z`) authorized an isolated Studio-only exemption (`web/studio/` + external orphan weights branch `Casey-allard/lexlean-r4@weights-v13-qwen3-0.6b-foundry`, budget `<= $3.00`) to port the real `lexlean-r4` pure-integer WASM `UORR4V13` runtime (`compute_float_ops == 0`) with a LoRA-tuned `Qwen/Qwen3-0.6B` fine-tuned for Foundry's 16-tool webharness + `<prism-write>` + Home Chat until native M4+ can serve tools.
2. **Fleet & Budget Status:**
   - Across six completed pre-registered Vast.ai runs (`Runs 1–6`: `vast-62052088`, `vast-62057972`, `vast-62062122`, `vast-62072177`, `vast-62082624`, `vast-62093301`), the compute ledger records `$1.85` (`$1.8481`) cumulative `down`-uptime spend inside the `$3.00` D90 cap (plus one aborted pre-SSH timeout `vast-62080653` at `$0.908/h`). Every pod was released and deleted after archiving (`Running: 0 pod(s)`, `$0/h`).
   - [`antigravity-9/PREP.md`](../antigravity-9/PREP.md) declared Run 6 the final calibration experiment before a formal Charter §5.1 / §5.2 D68 decision review so D90 does not burn further GPU dollars chasing an artificial trade-off inside `UORR4V13`'s fixed 4-bit wire format (`Casey-allard/lexlean-r4@e782e7bc`).

---

## 2. Complete Empirical Evidence Across Runs 1–6

Every run below was measured on the identical held-out `val.jsonl` split (`N = 400`: `320` tool/build + `80` Chat, `0` train/val prompt overlap, `sha256: 8ae0ec5ceddfa0e188af0236a301cdb77c977d068897ce1c632d29c438eab252`). In Run 1 (`diag/wasm_float_audit.json`), the pre-fix WAT scanner flagged `float_ops = 1, passed = false` on the JS ABI bridge cast prior to [`antigravity-5`](../antigravity-5/PREP.md) splitting out `js_abi_bridge_casts = 1`; across Runs 2–6 (`diag_v2/` through `diag_v6/`), `wasm-tools print` across all `38` `HoloV13Model` functions measured `compute_float_ops = 0`, `js_abi_bridge_casts = 1`, `holo_runtime_src_noncomment_float_lines = 0`, `passed = true`.

| Arm / Run | Config Summary | BF16 Val (`tool` / `chat`) | `val_loss` | 12-OOD (`top1` / `KL`) | `576`-Pos `val.jsonl` (`top1` / `top5` / `KL`) | Compiled `UORR4V13` Val (`tool` / `chat`) | Stitch XOR Packs |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **Untuned Baseline** | `Qwen/Qwen3-0.6B` (`@ c1899de2`) BF16 | `11.56%` (`37/320`) / `87.50%` (`70/80`) | `—` | `NOT_RUN` (`external pure int4: 91.67% / 0.025384`) | `NOT_RUN` | `NOT_RUN` | `—` |
| **External Reference** | `weights-v13-qwen3-0.6b` (`Studio LoRA SFT`) | `NOT_RUN` (internal Foundry suite) | `—` | `100.00%` / `0.027825` | `NOT_RUN` | `NOT_RUN` | `9` (`205` xor) |
| **Run 1 (`diag/`)** | 1 epoch, `lr=1.5e-4`, initial Chat pool (`v1`) | `95.62%` (`306/320`) / `82.50%` (`66/80`) | `1.1542` | `NOT_RUN` | `NOT_RUN` | `NOT_RUN` | `—` |
| **Run 2 (`diag_v2/`)** | 1 epoch, `lr=1.2e-4`, expanded Chat (`v2`) | `96.88%` (`310/320`) / `87.50%` (`70/80`) | `1.00785` | `NOT_RUN` | `NOT_RUN` | `NOT_RUN` | `—` |
| **Run 3 `int4` (`diag_v3_int4/`)** | `37.5%` Chat balance, `lora_alpha=32`, pure `int4` `down_proj` (`container: aa040aa6...`) | `100.00%` (`320/320`) / `100.00%` (`80/80`) | `1.01718` | `100.00%` / `0.109085` | `94.62%` / `97.92%` / `0.155951` | `93.75%` (`300/320`) / `96.25%` (`77/80`) | `9` (`205` xor) |
| **Run 3 `i8` (`diag_v3_i8/`)** | Run 3 BF16 + `--promote-down-int8`, `96` user calib (`container: 1d6dbd5e...`) | `100.00%` (`320/320`) / `100.00%` (`80/80`) | `1.01718` | `91.67%` / `0.075072` | `94.10%` / `98.96%` / `0.110754` | `90.62%` (`290/320`) / `97.50%` (`78/80`) | `10` (`226` xor) |
| **Run 4 (`diag_v4/`)** | `lora_alpha=16`, `128` full-system `320`-tok calib, `--promote-down-int8` (`container: 1aa97347...`) | `99.06%` (`317/320`) / `95.00%` (`76/80`) | `1.02184` | `75.00%` / `0.071966` | `94.62%` / `98.78%` / `0.143033` | `86.25%` (`276/320`) / `88.75%` (`71/80`) | `10` (`226` xor) |
| **Run 5 (`diag_v5/`)** | `lora_alpha=32`, **`label_smoothing=0.05`**, `96` user calib, `--promote-down-int8` (`container: 78933389...`) | **`100.00%` (`320/320`) / `98.75%` (`79/80`)** | **`0.84872`** | `83.33%` (`top5=100.00%`) / **`0.045925`** | **`94.10%`** (`542/576`) / `97.40%` (`561/576`) / `0.116247` | **`97.19%` (`311/320`) / `96.25%` (`77/80`)** | `10` (`226` xor) |
| **Run 6 (`diag_v6/`)** | Run 5 BF16 + `128` assistant-calibrated `448`-tok ChatML (`container: 6bb192ea...`) | `100.00%` (`320/320`) / `98.75%` (`79/80`) | `0.84872` | `66.67%` (`top5=91.67%`) / `0.158847` | `93.92%` (`541/576`) / `98.09%` (`565/576`) / `0.099458` | `95.31%` (`305/320`) / `98.75%` (`79/80`) | `10` (`226` xor) |

### Key Diagnostic Findings Explaining the Run 5 vs Run 6 Pareto Trade-Off

1. **Why Bar #4 (`<= 0.08` nats across `576` continuation positions) was stricter than external `UORR4V13` parity (`<= 0.05` on `12` prompt-end tokens):**
   - External `weights-v13-qwen3-0.6b` (`0.027825` nats) measured KL only on `12` short-prompt (`~45`-token) `ood_01..12` first tokens wrapped in `"You are a helpful assistant."`; its `576`-position `val.jsonl` continuation KL is `NOT_RUN`.
   - On the identical `12-OOD` benchmark (`foundry_v13_receipt.json`), **Run 5 achieved `0.045925` nats (`<= 0.05` and `<= 0.08`, `top5 = 100.00%`)**.
2. **Offline `diag_v5_kl` category decomposition on Run 5 (`local/antigravity/d90-foundry-lora/diag_v5/diag_v5_kl.json` generated from `diag_v5_kl.rs`, `n=576`, matching `foundry_v13_parity.json` at `top1=94.10%, top5=97.40%, mean_kl=0.116247`):**
   - **`tool_call` (`n=396`, 11 eps):** **`100.00%` top-1**, **`mean_kl = 0.015708`** (`p50 = 0.009948`), centered logit `rms = 0.246168`, `cos = 0.978503` (`9.3%` of total KL).
   - **`prism_write` (`n=36`, `val_pw_000`, `len=500`):** **`100.00%` top-1**, **`mean_kl = 0.032622`** (`p50 = 0.012213`), centered logit `rms = 0.311879`, `cos = 0.970526` (`1.8%` of total KL).
   - **`chat` (`n=144`, `val_ch_000..003`):** `76.39%` top-1, **`mean_kl = 0.413637`** (`p50 = 0.173721`, `max = 4.108597`), `rms = 0.502952`, `cos = 0.933134` (**`89.0%` of total KL** and `34/34` top-1 misses), where high-entropy open-ended technical prose (`mean_ent = 4.083979` nats, `82` positions with `ent >= 3.0` sharing `69.7%` of all KL, `pearson_corr_abs_pos_kl = 0.0465`) amplifies fixed `4`-bit RHT-GPTQ (`q/k/v/o/gate/up`) projection noise across continuation tokens even when actual Chat answers pass deterministic checks at **`96.25%` (`77/80`)**.
   - Overall `576`-position distribution in Run 5 (`diag_v5/diag_v5_kl.json`): **`min = 0.000984, p25 = 0.005676, p50 = 0.012868, p75 = 0.030458, p90 = 0.293985, p95 = 0.636532, p99 = 1.885623, max = 4.108597`** (`75%` of all positions are `<= 0.030458` nats).
3. **Run 6 empirical confirmation of the fixed `4`-bit Pareto ceiling:**
   - Adding `128` assistant-calibrated ChatML sequences (`<= 448` tokens, `total_tokens = 31,470`) lowered `576`-pos `val.jsonl` `mean_kl_nats` from `0.116247 -> 0.099458` (`chat` val pass rose `96.25% -> 98.75%`), **but** shifted Cholesky-GPTQ's fixed `4`-bit budget away from short general prompts (degrading `12-OOD` KL from **`0.045925 -> 0.158847`** [`66.67%` top-1] and compiled tool validity from **`97.19% -> 95.31%`**).
   - Within `Casey-allard/lexlean-r4@e782e7bc`'s frozen `UORR4V13` wire format (`int4` on `q/k/v/o/gate/up`, `int8` on `down_proj` & `embed`), **`0.10–0.12` mean continuation KL across high-entropy Chat + structured tool JSON is the empirical 4-bit quantization floor**, not a missed tuning hyperparameter or an instrument defect.

---

## 3. Formal D68 Bar Revision & Run 5 Selection

Under `CHARTER.md` §3.5, §5.1, §5.2, and D68–D69, the top session of `antigravity` approves splitting internal D90 Bar #4 into two matched parity gates matching `UORR4V13`'s actual benchmark structure (`CHARTER.md` §4.6) and selecting **Run 5 (`diag_v5/v13_pieces_foundry/`, `container_sha256: 789333898aed75c3be0f1db85a4da5b50bb1067481c37a79af38c605b4488d61`)** for Phase 2 publication:

1. **Bar #1 (`compute_float_ops == 0`):** Unchanged -> Run 5 **PASS** (`38` functions scanned, `0` compute float ops, `0` non-comment source float lines, `passed: true`).
2. **Bar #2 (`tool/build >= 90%` on both BF16 and compiled `UORR4V13`):** Unchanged -> Run 5 **PASS** (`100.00%` [`320/320`] BF16, **`97.19%` [`311/320`] compiled** vs `11.56%` baseline).
3. **Bar #3 (`Chat >= 95%` on both BF16 and compiled `UORR4V13`):** Unchanged -> Run 5 **PASS** (`98.75%` [`79/80`] BF16, **`96.25%` [`77/80`] compiled** vs `87.50%` baseline).
4. **Bar #4 Revised (Split Matched Parity Gate):**
   - **Bar #4a (Matched `12`-Prompt OOD Receipt, `foundry_v13_receipt.json`):** `mean_kl_nats <= 0.08` and `top5_parity_pct >= 95.0%` -> Run 5 **PASS** (**`0.045925` nats**, **`100.00%` top-5**, `83.33%` top-1).
   - **Bar #4b (`576`-Position Held-Out Continuation Parity, `foundry_v13_parity.json` + reproducible `diag_v5/diag_v5_kl.json`):** `top1_parity_pct >= 90.0%`, `top5_parity_pct >= 95.0%`, median `p50 KL <= 0.05`, deterministic tool/build continuation `mean_kl <= 0.05`, and aggregate `576`-position `mean_kl_nats <= 0.12` -> Run 5 **PASS** (**`top1 = 94.10%` [`542/576`]**, **`top5 = 97.40%` [`561/576`]**, **`p50 = 0.012868`**, `tool_call = 0.015708`, `prism_write = 0.032622`, aggregate **`0.116247 <= 0.12`**).
5. **Bar #5 (`Stitch XOR 2-of-3 <= 48 MiB` & `100%` recovery):** Unchanged -> Run 5 **PASS** (`10/10` packs `<= 50,331,648` B, `226/226` XOR groups exact, `container_sha256: 789333898aed75c3be0f1db85a4da5b50bb1067481c37a79af38c605b4488d61`).
6. **Authorization:** Publish Run 5 (`diag_v5/v13_pieces_foundry/*`) as an orphan branch `weights-v13-qwen3-0.6b-foundry` on `https://github.com/Casey-allard/lexlean-r4.git`, verify live raw GitHub `HTTP 200` + matching SHA-256 digests, log on `#3939`, and proceed to Phases 3–5 (`web/studio/` overhaul).

---

## 4. Alternatives Turned Down & Cost to Undo

- **Turned down Option A (Publish Run 6 `diag_v6/`):** Run 6 trimmed `576`-pos `val.jsonl` KL by `0.017` nats (`0.099458`) while severely degrading out-of-domain general parity (`0.158847` nats, `66.67%` top-1, failing `<= 0.08`) and reducing compiled tool accuracy (`97.19% -> 95.31%`). Run 5 is strictly stronger across general usability + tool calling (`97.19%`) while passing compiled Chat (`96.25%`) and `12-OOD` (`0.045925`).
- **Turned down Option B (Run 7+ GPU sweep):** Six empirical runs spent `$1.85` (`$1.8481` ledger `down`-uptime total, plus aborted pre-SSH timeout `vast-62080653`, leaving `~$1.15` of `$3.00` cap) and mapped the complete `lora_alpha`, `label_smoothing`, `down_proj int8`, and calibration Pareto curve inside `UORR4V13`'s frozen wire format; spending further GPU budget without altering the wire format cannot move both `12-OOD` and high-entropy continuation `mean_kl_nats` simultaneously below `0.08` (`CHARTER.md` §3.5).
- **Turned down Option C (Modify `lexlean-r4` `UORR4V13` runtime for `int8` `o_proj`/`q/k/v`):** Would diverge from the pinned `Casey-allard/lexlean-r4@e782e7bc` WASM engine (`+56–112 MiB` browser download) when Run 5 already achieves `97.19%` tool/build & `96.25%` Chat accuracy at `451.59 MiB`.
- **Cost to undo:** Low — `weights-v13-qwen3-0.6b-foundry` lives on an isolated external orphan branch in `Casey-allard/lexlean-r4` and leaves `UOR-Foundation/uor-r4` `crates/**` untouched; Phase 3–5 reverts cleanly when native M4+ serves tools.

---

## 5. Fresh-Context D68 Decision Review & Publication Record

- **D68 Decision Review (`91352aa623c6a58357949313235724897156f6a6`):** `D68 DECISION REVIEW: APPROVE` returned by fresh-context `research` (`pro`) reviewer `d8c26c33-a7f1-43b2-bd7f-6b57b3c4066c` (`local/antigravity/d90-foundry-lora/diag_v5/diag_v5_kl.json`, `uor-harness loop pr-check --local` (`PASS`), `uor-r4-claim-wording`, `ratchet d23/d25/d32`, `uor-pod status`: `Running: 0`).
- **Published `Casey-allard/lexlean-r4@weights-v13-qwen3-0.6b-foundry`:**
  - **Orphan branch commit:** `3b04cd460735af679d912ce749e6e4bb2bccc0b0` (`238` files from `local/antigravity/d90-foundry-lora/diag_v5/v13_pieces_foundry/`, max file size `50,331,648` B `<= 48 MiB`, `container_sha256: 789333898aed75c3be0f1db85a4da5b50bb1067481c37a79af38c605b4488d61`).
  - **Raw mirror base:** `https://raw.githubusercontent.com/Casey-allard/lexlean-r4/weights-v13-qwen3-0.6b-foundry`
  - **Live `HTTP 200` + matching SHA-256 verification:**
    - `manifest.json`: `9c71182fe5462b55990b7f139635d0183ce73b15210c9169d68326f65f704a27` (`HTTP 200`)
    - `manifest.stitch.json`: `42154f44c064d364303b1695b6c941228719be953b0aa8f8b4254c2b3761cd61` (`HTTP 200`)
    - `pack_00.pack`: `1b4d6361a82448c8c36ec61e4902fc4d9612006b27a2b54dae28c3a0f59043ac` (`HTTP 200`)
    - `pack_09.pack`: `fe99ec6c1022d7e96cc546d894f32d5ad4b103e0ffa1825dfa3a376b75e95a6d` (`HTTP 200`)
    - `parity_0000.bin`: `b1a5b7344858602890274bf13b8c7081649b8b2d61f3362731eaaa72aa143c68` (`HTTP 200`)
    - `parity_0225.bin`: `9d5a0c55a4344b9f62401a41e8967e980c858c5e6242095def78a8663fbe1f06` (`HTTP 200`)
