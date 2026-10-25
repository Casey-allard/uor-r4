# REPORT: r-secant-surrogate-descent, antigravity-77

- Item: `r-secant-surrogate-descent`
- Issue: `#2886` (`Closes #2886`)
- Lab: `antigravity`
- Session: `antigravity-77`
- ORDER: `docs/loop/tasks/r-secant-surrogate-descent/antigravity-77/ORDER.md`
- Suborders:
  - `local/antigravity/antigravity-77/orders/1.md` (subagent `7fc3f78d-b0dd-46f9-82ce-25406b4cf28c`: fresh-context `TEST FITNESS: FIT`, D68 `DECISION REVIEW: APPROVE`, and D67 `PRE-RUN REVIEW: PASS`)
  - `local/antigravity/antigravity-77/orders/2.md` (fresh-context D66 exact-head review slice 1: `crates/uor-r4-float-train/src/secant_descent.rs` lines 1–700)
  - `local/antigravity/antigravity-77/orders/3.md` (fresh-context D66 exact-head review slice 2: `crates/uor-r4-float-train/src/secant_descent.rs` lines 701–1292, `src/bin/secant-surrogate-descent.rs`, `src/geometric_stack_native.rs`, `src/geometric_stack_native_cuda.rs`, `src/lib.rs`)
  - `local/antigravity/antigravity-77/orders/4.md` (fresh-context D66 exact-head review slice 3 & claims check: `docs/labs/r-secant-surrogate-descent/**`, `docs/mechanisms/**`, `docs/harness/prep/r-secant-surrogate-descent-2026-10-25.md`, `docs/research/notebook/float-training.md`, `docs/roadmap/**`, `ORDER.md`, `REPORT.md`)
- Sealed GPU artifact: `docs/labs/r-secant-surrogate-descent/receipt.json` (`c5e7909dcaf88a7e2338eff6d5c1121a4f1abe751e56ec6d0dc70c3e0b0f5fb3`)
- HF store archive: `caseyallard/uor-r4-store:results/antigravity/r-secant-surrogate-descent-2026-10-25/`

## What did not work

1. **Cancel probe findings (classed `instrument` before the GPU run):**
   - `uor-harness check` initially failed `prep.sheet_shape` on `docs/harness/prep/r-secant-surrogate-descent-2026-10-25.md` because Section 6 used `- **Abort criteria:**` instead of `- **Stop rule:**`.
   - The draft sheet specified `Objective::MarginHinge { rivals: 64 }`, whereas `NativeVectorWriter::margin_rows` (`crates/uor-r4-float-train/src/native_vector_writer.rs:394`) enforces `!(1..=8).contains(&rivals)` (`kept_cells = 8`).
   - The draft sheet listed pre-extraction placeholder paths/hashes for the training slice instead of the sealed `UORT` corpus (`w2-inputs/2026-10-25/driver-data/mw-balp/train/{tokens.u16,response_mask.u8}` and `tokenizer.json`) in `caseyallard/uor-r4-store`.
   - Updating all model parameters together would allow the 1,048,576-parameter `embedding.weight` tensor (which has a direct residual skip path to `final_norm` and `NativeVectorWriter::margin_rows`) and `StackAdamW`'s global `cuda_gradient_clip` norm to confound `ZeroCredit_Null`. Isolating `layers.*.rec.decay` in Part A and `layers.*.*.spiralcore_fbs` in Part B via `StackAdamW::update_filtered` eliminated this confound (`ZeroCredit_Null` gives exact $\Delta L = 0.000000$).
2. **Pod script setup issues (`instrument`, repaired before the run):**
   - On the first pod invocation, `/root/uor-store get w2-inputs/2026-10-25/ /workspace/secant-inputs` unpacked into `/workspace/secant-inputs/w2-inputs/2026-10-25/`, and `job.sh` passed an unused `--git-sha` flag. Both were fixed in `job.sh` before the probe ladder ran.
3. **Null controls (`RecZeroCredit_Null`, `RecRandomRefit_Null`, `FbsZeroCredit_Null`, `FbsRandomRefit_Null`):**
   - `RecZeroCredit_Null` ($ds = 0$) and `FbsZeroCredit_Null` ($dq = 0$) produced zero cell/root boundary crossings and zero loss reduction ($\Delta L = 0.000000$ on both seeds).
   - `RecRandomRefit_Null` (norm-preserving per-lane Rademacher sign flips $\xi_l \in \{-1, +1\}$ on $ds$) crossed rate boundaries on `24.74%–26.04%` of decay parameters but slightly degraded loss ($\Delta L = -0.003647 / -0.003860$ on Seed 1, $-0.002389$ on Seed 2).
   - `FbsRandomRefit_Null` (norm-preserving elementwise Rademacher sign flips $\mathrm{diag}(\pm 1, \dots, \pm 1) \in O(8)$ on $dq_{o,m}$) crossed stage roots on `2.92%–3.28%` of queries and flipped `5.24%–5.81%` of output Clifford routes, yielding $\Delta L = +0.023534$ on Seed 1 and $-0.019526$ on Seed 2 (net two-seed mean $+0.002004$, inside the noise band $\Delta_{\text{noise}}^{\text{fbs}} = 0.043060$).
4. **Cross-build float non-associativity in Part A (`instrument` observation, non-blocking):**
   - While Part B (`FbsSecant_Active`, `FbsZeroCredit_Null`, `FbsRandomRefit_Null`) was 100% bit-for-bit identical between Build A and Build B across all 6 runs, Part A Seed 2 `RecSecant_Active` differed slightly at step 256 ($\Delta L = +0.078261$ in Build A vs $+0.065333$ in Build B) due to CUDA atomicAdd ordering in `rate_select_bwd` / `embedding` backward accumulation across thread blocks. Both runs passed `G-Rec-Descent` by $> 3.2\times$ and $> 10.6\times$ their respective two-seed noise bands.

## What changed

- **Size (`loop pr-check`):** `tests 0 file(s) +0/-0, code 5 file(s) +1679/-7, docs 8 file(s) +271/-62, generated 1 file(s) +3/-3, other 1 file(s) +1553/-0` (`15` files changed):
  - **Code & tests (`crates/uor-r4-float-train`):** `5` files (`src/lib.rs`, `src/geometric_stack_native.rs`, `src/geometric_stack_native_cuda.rs`, `src/secant_descent.rs`, `src/bin/secant-surrogate-descent.rs`), `+1,683 / -9` lines (including 5 new unit tests in `src/secant_descent.rs`).
  - **Documentation, prep sheet, roadmap, and task records:** `7` markdown files (`docs/harness/prep/r-secant-surrogate-descent-2026-10-25.md`, `docs/labs/r-secant-surrogate-descent/README.md`, `docs/mechanisms/recurrence-gates.md`, `docs/mechanisms/spiralcore-fbs.md`, `docs/research/notebook/float-training.md`, `docs/roadmap/items/r-secant-surrogate-descent.md`, `docs/roadmap/README.md`, plus `ORDER.md` and `REPORT.md`), `+353 / -33` lines.
  - **Sealed JSON receipt:** `docs/labs/r-secant-surrogate-descent/receipt.json` (`1,553` lines, SHA-256 `c5e7909dcaf88a7e2338eff6d5c1121a4f1abe751e56ec6d0dc70c3e0b0f5fb3`).
- **Ratio to estimate:** The roadmap item carried no numeric line estimate (`size:` unset); the `1,553`-line JSON receipt accounts for 43% of the diff, and the runner + hooks + 5 unit tests in `crates/uor-r4-float-train` account for `1,683` lines.

## Measurements

- **Conditions:** 1 $\times$ NVIDIA GeForce RTX 5090 (`32607 MiB`, driver `595.99.02`, `sm_120`, `nvcc V12.8.93`, Vast pod `vast-57690399`), commit `8ee5e945bf1d7b18eddf6d4a5a303f165e6891e8`, binary SHA-256 `7c7d90ad988efe3f1824ae2a9453a0f5f4fb847bbf1c1a5f33d517c439008cb4` (byte-identical across two clean `nvcc` builds A and B), micro-probe shape `pattern = "rrrrrr"`, `width = 256`, `heads = 4`, `mlp_hidden = 384`, `batch = 4`, `context = 192`, `vocab = 4096`, `kept_cells = 8`, `Objective::MarginHinge { rivals: 8 }`, `256` steps cycling `64` batches (`49,152` tokens) from `w2-inputs/2026-10-25/driver-data/mw-balp/train/tokens.u16` (`3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`), seeds `1` and `2`.
- **Command:**
  ```sh
  /workspace/secant-bin/secant-surrogate-descent-a \
    --mode full \
    --tokens /workspace/secant-inputs/w2-inputs/2026-10-25/driver-data/mw-balp/train/tokens.u16 \
    --mask /workspace/secant-inputs/w2-inputs/2026-10-25/driver-data/mw-balp/train/response_mask.u8 \
    --tokenizer /workspace/secant-inputs/w2-inputs/2026-10-25/driver-data/tokenizer.json \
    --out /workspace/secant-out/build-a/receipt.json
  ```
- **Speed (lead metric, CHARTER §1):** `22.21–22.50 s` per 256-step arm (`~86.8–87.9 ms/step`, `~8,738–8,851` training tokens/s at `768` tokens/step; `267.66 s` total for 12 runs).
- **Instrument checks (`I-1..3`):**
  - `I-Finite-CUDA`: `PASS` (`25,344 / 25,344` CUDA error checks and `10,752 / 10,752` backward finiteness checks pass per run; step-0 $\|ds\|_2 \in [0.1695, 0.2340]$, $\|dq\|_2 \in [0.3145, 0.3621]$).
  - `I-Boundary-Crossings`: `PASS` at `lr = 0.005` (`1.0x`): `RecSecant_Active` crosses discrete rate cells on `30.73%` (`118/384`, Seed 1) and `35.42%` (`136/384`, Seed 2 Build A; `34.64%` Build B) of decay parameters (`27.18%` and `30.64%` of active lane-tokens), $\ge 1.0\%$. `FbsSecant_Active` crosses discrete $E_8$ stage roots on `5.59%` (`200/3577`, Seed 1) and `5.37%` (`192/3573`, Seed 2) of active `(output, stage)` queries ($\ge 1.0\%$) and flips `9.79%` (`155/1584`) and `9.60%` (`152/1584`) of output Clifford actions $A_o$.
  - `I-Keep-Plateau`: `PASS` (`analytic_keep_secant_used = false`; upper-plateau fraction $i \in \{12..15\}$ is `38.44%–41.21% < 80%`, step-0 mean $|d\lambda| \approx 3.44 \times 10^{-5} \gg 10^{-7}$).
- **Part A (`G-Rec-Descent` — `RecGates::Geometric` `rate_select_bwd`):**
  - Seed 1 ($L_0 = 2.811901$): `RecSecant_Active` $L_{256} = 2.752162$ ($\Delta L = +0.059740$) vs `RecZeroCredit_Null` ($\Delta L = 0.000000$) and `RecRandomRefit_Null` ($\Delta L = -0.003647$ Build A, $-0.003860$ Build B). Margin over best null control $= +0.059740$.
  - Seed 2 ($L_0 = 2.842091$): `RecSecant_Active` $L_{256} = 2.763829$ ($\Delta L = +0.078261$ Build A; $L_{256} = 2.776757$, $\Delta L = +0.065333$ Build B) vs `RecZeroCredit_Null` ($\Delta L = 0.000000$) and `RecRandomRefit_Null` ($\Delta L = -0.002389$). Margin over best null control $= +0.078261$ (Build A) / $+0.065333$ (Build B).
  - Noise band $\Delta_{\text{noise}}^{\text{rec}} = 0.018522$ (Build A) / $0.005594$ (Build B). Both seeds beat the best null control by $\ge \Delta_{\text{noise}}^{\text{rec}}$ ($3.23\times$ and $4.23\times$ on Build A; $10.68\times$ and $11.68\times$ on Build B) $\implies$ **`G-Rec-Descent = PASS (mechanism descent)`**.
- **Part B (`G-Fbs-Descent` — `QuaternionWeights::SpiralcoreFbs` `spiralcore_fbs_bwd`, bit-identical across Builds A & B):**
  - Seed 1 ($L_0 = 2.811901$): `FbsSecant_Active` $L_{256} = 2.744842$ ($\Delta L = +0.067059$) vs `FbsZeroCredit_Null` ($\Delta L = 0.000000$) and `FbsRandomRefit_Null` ($L_{256} = 2.788368$, $\Delta L = +0.023534$). Margin over best null control $= +0.043525$.
  - Seed 2 ($L_0 = 2.842091$): `FbsSecant_Active` $L_{256} = 2.738088$ ($\Delta L = +0.104003$) vs `FbsZeroCredit_Null` ($\Delta L = 0.000000$) and `FbsRandomRefit_Null` ($L_{256} = 2.861617$, $\Delta L = -0.019526$). Margin over best null control $= +0.104003$.
  - Noise band $\Delta_{\text{noise}}^{\text{fbs}} = 0.043060$. Both seeds beat the best null control by $\ge \Delta_{\text{noise}}^{\text{fbs}}$ ($+0.043525 \ge 0.043060$, $1.011\times$; and $+0.104003 \ge 0.043060$, $2.415\times$) $\implies$ **`G-Fbs-Descent = PASS (mechanism descent)`**.

## Checks run at the head

- `CARGO_TARGET_DIR=local/target ~/.cargo/bin/cargo test --offline -p uor-r4-float-train --no-fail-fast`: 24 passed (`10` unit tests in `lib`, `2` in `no_training_crate_dep`, `12` in `twenty_four_kernel_controls`).
- `CUDARC_CUDA_VERSION=12080 NVCC=local/antigravity/fake-nvcc/nvcc UOR_NVCC_ARCH=compute_120 ~/.cargo/bin/cargo clippy --offline --no-deps -p uor-r4-float-train --features cuda --all-targets -- -D warnings`: exit 0, 0 warnings.
- `uor-harness ratchet d23 && uor-harness ratchet d25 && uor-harness ratchet d32`: all 3 OK (`4451`, `93`, `31` baseline).
- `uor-harness loop gates --full`: 4 of 4 PASS (`fmt-check-claim-wording`, `fmt-check-harness`, `claim-wording`, `progress-window`).
- `uor-harness loop report --check --order docs/loop/tasks/r-secant-surrogate-descent/antigravity-77/ORDER.md`: PASS.
- `uor-harness loop pr-check --local`: PASS.

## Skills used

- `uor-research-architect`: Used to design the parameter-group isolated secant surrogate descent runner (`StackAdamW::update_filtered`) and norm-preserving Rademacher null controls (`RecRandomRefit_Null`, `FbsRandomRefit_Null`).
- `scope-guard`: Verified all touched files fall inside `r-secant-surrogate-descent`'s declared scope and always-allowed paths.
- `solution-integrity`: Verified instrument checks (`I-Finite-CUDA`, `I-Boundary-Crossings`, `I-Keep-Plateau`) before scoring mechanism gates, and replicated the full 12-run suite across two clean `nvcc` builds on RTX 5090.
- `gates`: Ran the full gate suite and ratchets before exact-head review.

## Still open

- **Downstream items now unblocked by `#2886`:**
  - `hamilton-gate-offline-float-sheet` (`#2824`)
  - `recurrence-gates-six-recurrence-fit` (`#2890`)
  - `spiralcore-fbs-offline-float-sheet` (`#2891`)
- Full-rung training, fresh panel transfer, and D11 integer export gap measurement belong to those downstream sheets (`#2824`, `#2890`, `#2891`).

## Plain summary

This unit (`antigravity-77`) completed `r-secant-surrogate-descent` (`#2886`), the pre-registered GPU gate that tests whether the project's two **secant surrogates** (finite-difference credit rules across discrete table steps, used in place of zero derivatives from hard selection) produce genuine loss descent during offline floating-point training in `crates/uor-r4-float-train`. In **Part A** (`RecGates::Geometric`), each recurrence lane selects a power-of-two fade rate $\lambda = 2^{-(i+1)}$ and keep factor $\kappa = \sqrt{1 - \lambda^2}$ from a 16-entry table (`rate_select_fwd`), and the backward kernel (`rate_select_bwd`) assigns credit across the two neighbouring table entries separated by step width $1/16$. In **Part B** (`QuaternionWeights::SpiralcoreFbs`), each projection stage selects the nearest root of a 16-root $E_8$ lattice orbit in a base-3 ternary binder tree (`FBS`, `spiralcore_fbs_fwd`), and the backward kernel (`spiralcore_fbs_bwd`) assigns credit along root-difference edges to sibling branches. Across 256 steps on 2 seeds on an RTX 5090 (`sm_120`), replicated across two independent `nvcc` builds, all three **instrument checks** (`I-Finite-CUDA`, `I-Boundary-Crossings`, `I-Keep-Plateau`) passed, and both active secant arms crossed discrete cell/root boundaries (`30.73%–35.42%` of recurrence decay parameters and `5.37%–5.59%` of FBS stage queries) and reduced the `MarginHinge` loss (`rivals = 8`, `kept_cells = 8`) by more than the two-seed noise band $\Delta_{\text{noise}}$ over both the zero-credit null control (`ZeroCredit_Null`, exact $\Delta L = 0$) and the norm-matched Rademacher random-sign control (`RandomRefit_Null`). Both `G-Rec-Descent` and `G-Fbs-Descent` therefore pass, closing `#2886` and unblocking `#2824`, `#2890`, and `#2891`.
