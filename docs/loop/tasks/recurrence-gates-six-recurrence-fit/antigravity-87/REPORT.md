# REPORT: recurrence-gates-six-recurrence-fit, antigravity-87

Closes #2890

- **Lab:** `antigravity`
- **Item:** [`recurrence-gates-six-recurrence-fit`](../../../../roadmap/items/recurrence-gates-six-recurrence-fit.md) (`#2890`)
- **Order:** [`docs/loop/tasks/recurrence-gates-six-recurrence-fit/antigravity-87/ORDER.md`](./ORDER.md)
- **Pre-registration sheet:** [`docs/harness/prep/recurrence-gates-six-recurrence-fit-2026-10-25.md`](../../../harness/prep/recurrence-gates-six-recurrence-fit-2026-10-25.md) (`TEST FITNESS: FIT`, D68 `DECISION REVIEW: APPROVE`, D67 `PRE-RUN REVIEW: PASS`)
- **Sealed HF store artifacts:** `caseyallard/uor-r4-store:results/antigravity/recurrence-gates-six-recurrence-fit-2026-10-25/`
  - [`docs/labs/recurrence-gates-six-recurrence-fit/receipt.json`](../../../labs/recurrence-gates-six-recurrence-fit/receipt.json) (Build A): `5835ff953a88d5f04e59ded194c13a91938cd7a472dfe2fc619cfbc96faaf69b`
  - `receipt-build-b.json` (Build B): `04b9c409d783f8fa77443a86975adaa6583cc9cfbb05d8ca9042bfb41c43eef9`
  - `synthetic-probe.json`: `b0f98cfbc2321a1e4008d7a046efdac7f82e0bb8af054fc2ed6f8e0362aa7040`
  - `throughput-probe.json`: `1b91515c3b057516dc1430c83262401905ac9a1197c8dab063c9c3b5bf5fa53f`
  - `binary-SHA256SUMS`: `f320fc8e0f3cd0572e15b408dfe0e84699fd4812ab2081729bfa43455a4c63c5` (`recurrence-six-fit-a` and `recurrence-six-fit-b` byte-identical: `9737cd8f88bdc0aba2e0998373023729ab7445fa04c8f89162606901ec44fda3`)

## What did not work

1. **Instrument defects diagnosed and fixed before the full GPU run (`CHARTER.md` §4.2):**
   - In Probe (a) (`--mode synthetic-probe`), `verify_standalone_cuda_parity` initially compared the `f32` `KEEP_TABLE` against `f64` $\sqrt{1 - \lambda^2}$ without casting `exact_keep` to `f32` (`max_keep_table_error = 2.86e-8` vs `4.6e-10`) and required `max_grad_diff == 0.0` despite `atomicAdd` non-associativity in `spiralcore_fbs_bwd` (`max_grad_diff = 3.81e-6`). Fixed in `fd92cb495` (`max_keep_table_error = 0.0`, `max_secant_bwd_error = 0.0`, `max_stack_hidden_parity_diff = 0.0`, `max_stack_grad_parity_diff = 3.81e-6 <= 1e-5`).
   - In the first `1,024`-step run on `vast-57719671`, `d11_recurrence_scan_and_gate_i32` did not clamp the Q12 recurrence carry `h` to `[-32768, 32767]` (`[-8.0, 8.0]`) before the output Hamilton gate and evaluated $1 - \lambda^2$ directly instead of indexing the GPU's `f32`-rounded `RATE_TABLE`, causing `d11_layer0_oracle_max_lsb_diff` to exceed `32` Q12 LSBs when a lane exceeded `8.0` in float. Fixed in `e75c928a3`; `d11_layer0_oracle_max_lsb_diff` is now `4..6` Q12 LSBs across all 8 runs (`I-Export-Exactness = PASS`).
2. **Mechanism & configuration negatives under `4/4` passing instruments (`CHARTER.md` §4.1):**
   - **`G-Rec-Sheet-Fit = FAIL` (non-abelian $S^3$ active arm fails seed-1 top-1 replication despite `U1` and `IdentityTransport` passing on both seeds across both builds):** While `Rec_Geometric_IdentityTransport_Control` (`2,570` [`17.99%`] & `2,222` [`15.55%`] D11 int top-1 on Build A; `3,183` [`22.28%`] & `2,307` [`16.15%`] on Build B) and `Rec_Geometric_U1_AbelianControl` (`2,805` [`19.63%`] & `1,921` [`13.45%`] on Build A; `1,798` [`12.58%`] & `2,101` [`14.71%`] on Build B) beat both `Rec_FrozenRates_Null` ($c - b \ge F_I$) and `TrainSplit_CountTable_Ref` (`1,226 / 14,287` = `8.58%`) on **both seeds across both builds**, `Rec_Geometric_S3_Active` ($S^3$) passes on seed 2 (`1,999` [`13.99%`] Build A; `2,889` [`20.22%`] Build B D11 int top-1) but collapses in top-1 on seed 1 (`352` [`2.46%`] Build A; `313` [`2.19%`] Build B) even though its `ft/dev` `token_margin` hinge loss descends cleanly (`1.0024 -> 0.8664` / `0.8588`, beating `Rec_FrozenRates_Null` `0.8747` / `0.8694`). Per `CHARTER.md` §4.1, this negative closes the unconstrained `1,024`-step `StackAdamW` configuration of `Rec_Geometric_S3_Active`, never the `recurrence-gates` mechanism (`status: READY`).
   - **`G-Rec-Transport = FAIL`:** `Rec_Geometric_IdentityTransport_Control` (`rotation = false`) achieves the highest mean D11 integer DEV top-1 accuracy (`2,396.0` rows on Build A; `2,745.0` rows on Build B), followed by commutative `Rec_Geometric_U1_AbelianControl` (`2,363.0` / `1,949.5` rows). Unconstrained 4D $S^3$ unit-quaternion carry does not beat identity or abelian $U(1)$ carry on both seeds.
   - **`G-Rec-ExportGap = FAIL` on Build A / `PASS` on Build B:** Direct post-training D11 integer table export preserves `ft/dev` `token_margin` loss within the two-seed noise band on every arm and seed ($\Delta L \in [0.0050, 0.0304] \le \Delta_{\text{noise},L} = 0.0586$ on Build A / $0.0558$ on Build B), but un-snapped trajectory drift across `192` causal steps and `6` recurrence layers drops top-1 on `Rec_Geometric_IdentityTransport_Control` by $\Delta_{\text{top1}} = 1,914$ rows on Seed 1 and `221` rows on Seed 2 in Build A (`1,914 > 1,647` two-seed top-1 noise band $\implies \text{FAIL}$) and by $\Delta_{\text{top1}} = 1,406$ and `349` rows in Build B (`1,406 <= 1,554` $\implies \text{PASS}$). Disqualifying direct post-training integer rounding on Build A (`CHARTER.md` §3.7) shows that lossless D11 export of multi-layer recurrence requires hard-forward discrete state/gate snapping during training.
   - **`G-Rec-Transfer = SKIPPED_DEV_GATE_DID_NOT_PASS`:** Because `G-Rec-Sheet-Fit` did not pass on both seeds, the sealed `14,479`-target `fresh_v2` transfer draw was not opened (`fresh_v2_evaluated = false`).

## What changed

- **Files changed (`12` files):**
  - `docs/labs/recurrence-gates-six-recurrence-fit/instrument/Cargo.toml`, `src/lib.rs`, and `src/main.rs`: Pure-integer D32-compliant (`#![forbid(unsafe_code)]`, zero float/decimal/tuple-index tokens) D11 recurrence table exporter (exact bit-shift `>> (i + 1)` for $\lambda = 2^{-(i+1)}$, exact 3-term CSD Q12 keep table $\kappa_{\text{Q12}}$, exact 77-pair shift-add Q8 transport, and 120-icosian output Hamilton gate) and `verify-recurrence-six-fit` receipt verifier.
  - `docs/labs/recurrence-gates-six-recurrence-fit/runner/Cargo.toml`, `src/lib.rs`, and `src/main.rs`: Standalone Rust/CUDA six-recurrence runner built on `crates/uor-r4-float-train` (`CHARTER.md` §3.7 / D37, zero softmax/squashing/matmul) implementing `I-Standalone-CUDA-Parity`, `I-DEV-Denominator`, `I-Rate-Mobility-And-Plateau`, `I-Export-Exactness`, the 4 geometric arms (`Rec_Geometric_S3_Active`, `Rec_Geometric_U1_AbelianControl`, `Rec_Geometric_IdentityTransport_Control`, `Rec_FrozenRates_Null`), and `TrainSplit_CountTable_Ref`.
  - `docs/labs/recurrence-gates-six-recurrence-fit/receipt.json` (`8,809` lines, SHA-256 `5835ff953a88d5f04e59ded194c13a91938cd7a472dfe2fc619cfbc96faaf69b`) and `docs/labs/recurrence-gates-six-recurrence-fit/README.md`.
  - `docs/mechanisms/recurrence-gates.md`, `docs/research/notebook/float-training.md`, `docs/roadmap/items/recurrence-gates-six-recurrence-fit.md` (`status: done`), `docs/roadmap/README.md`, and `docs/loop/tasks/recurrence-gates-six-recurrence-fit/antigravity-87/{ORDER.md,REPORT.md}`.
- **Size and ratio to estimate:** Total addition is `10,962` lines across `12` files (`1,819` lines of Rust code/tests in `instrument/` and `runner/`, `8,809` lines of JSON receipt data in `receipt.json`, and `334` lines of documentation/records). The roadmap item carries no numeric line estimate (`size` is reported, never capped per `CHARTER.md` §5.10 / D48).

## Measurements

Measured on 2 $\times$ NVIDIA GeForce RTX 5090 (`sm_120`, Vast.ai pod `vast-57719671`, clean `nvcc` Builds A & B with byte-identical binary SHA-256 `9737cd8f88bdc0aba2e0998373023729ab7445fa04c8f89162606901ec44fda3`) over `1,024` steps (`batch = 4, context = 192`, `786,432` training tokens/run) on `mw-balp/train/tokens.u16` (`3dee858f…`), evaluated on all `14,287` eligible within-document response targets across `400` documents / `428` causal windows (`82,176` context tokens) in `ft/dev/tokens.u16` (`0919e90f…`):

- **Command:**
  ```sh
  ./target/release/recurrence-six-fit --mode full \
    --train-tokens /workspace/w2-inputs/2026-10-25/driver-data/mw-balp/train/tokens.u16 \
    --train-mask /workspace/w2-inputs/2026-10-25/driver-data/mw-balp/train/response_mask.u8 \
    --dev-tokens /workspace/w2-inputs/2026-10-25/driver-data/ft/dev/tokens.u16 \
    --dev-mask /workspace/w2-inputs/2026-10-25/driver-data/ft/dev/response_mask.u8 \
    --tokenizer /workspace/w2-inputs/2026-10-25/driver-data/tokenizer.json \
    --out /workspace/results/receipt.json
  ```
- **Lead Metric — Synchronized CUDA `tokens/s` (`CHARTER.md` §1):**
  - **D11 integer export DEV evaluation (`82,176` context tokens / `14,287` targets):** `106,427.3–112,803.5 tok/s` on Build A (`106,154.9–108,732.3 tok/s` on Build B).
  - **Continuous `f32` DEV evaluation (`82,176` context tokens / `14,287` targets):** `127,821.0–132,753.5 tok/s` on Build A (`118,424.3–121,115.7 tok/s` on Build B).
  - **Six-recurrence training (`786,432` tokens / `1,024` steps):** `21,566.8–23,865.2 tok/s` (`32.95–36.46 s`/run) on Build A; `20,354.9–22,607.8 tok/s` (`34.79–38.64 s`/run) on Build B.
- **Instrument Checks (`I-1..4` — 4/4 PASS on Builds A & B):**
  - `I-Standalone-CUDA-Parity = PASS`: `max_rate_table_error = 0.0`, `max_keep_table_error = 0.0`, `max_secant_bwd_error = 0.0`, `max_quaternion_scan_fwd_error = 5.00e-8`, `max_quaternion_scan_bwd_error = 4.68e-8`, `max_stack_hidden_parity_diff = 0.0`, `max_stack_grad_parity_diff = 3.81e-6`, nonzero step-0 $\|ds_l\|_2 \in [0.0470, 0.9581]$ across all 6 layers (`0.0` on `FrozenRates`).
  - `I-DEV-Denominator = PASS`: `400` documents, `428` windows, `14,287 / 14,287` scored targets on every float and D11 integer evaluation.
  - `I-Rate-Mobility-And-Plateau = PASS`: Active arms shift dyadic rate bins on `49.74%–56.77%` (`191–218 / 384`, Build A) and `49.22%–57.29%` (`189–220 / 384`, Build B) of lanes (`>= 5.0%`), with upper plateau `12..15` occupancy `40.09%–43.15% < 80%` (`0 / 384` crossings on `FrozenRates`).
  - `I-Export-Exactness = PASS`: `16/16` exact 3-term CSD Q12 keep entries, `77/77` exact shift-add Q8 pairs, `max_scan_lsb_diff = 0` on synthetic fixtures, `d11_layer0_oracle_max_lsb_diff = 4..6` Q12 LSBs, and `0 / 16,544` (`517` rows $\times$ `32` D11 evals) Q12 `i32` `NativeVectorWriter` oracle mismatches per build.
- **Quality & Export Gap Summary (`14,287` `ft/dev` targets, Build A / Build B):**
  - `TrainSplit_CountTable_Ref`: `1,226 / 14,287` (`8.58%`) top-1, `4,060 / 14,287` (`28.42%`) top-8.
  - `Rec_Geometric_IdentityTransport_Control` (`rotation = false`): D11 int top-1 **`2,570` (`17.99%`) & `2,222` (`15.55%`)** [Build B: **`3,183` (`22.28%`) & `2,307` (`16.15%`)**], D11 int top-8 `3,319` & `2,840` [Build B: `3,962` & `3,184`], float top-1 `4,484` (`31.39%`) & `2,443` (`17.10%`) [Build B: `4,589` (`32.12%`) & `2,656` (`18.59%`)], export gap $\Delta L = 0.0304$ & $0.0075$, $\Delta_{\text{top1}} = 1,914$ & $221$ [Build B: $\Delta L = 0.0256$ & $0.0050$, $\Delta_{\text{top1}} = 1,406$ & $349$].
  - `Rec_Geometric_U1_AbelianControl` ($U(1)$): D11 int top-1 **`2,805` (`19.63%`) & `1,921` (`13.45%`)** [Build B: **`1,798` (`12.58%`) & `2,101` (`14.71%`)**], D11 int top-8 `3,635` & `2,332` [Build B: `2,243` & `2,591`], float top-1 `3,988` (`27.91%`) & `2,740` (`19.18%`) [Build B: `2,519` & `2,869`], export gap $\Delta L = 0.0086$ & $0.0149$, $\Delta_{\text{top1}} = 1,183$ & $819$.
  - `Rec_Geometric_S3_Active` ($S^3$): D11 int top-1 **`352` (`2.46%`) & `1,999` (`13.99%`)** [Build B: **`313` (`2.19%`) & `2,889` (`20.22%`)**], D11 int top-8 `630` & `2,419` [Build B: `527` & `3,371`], float top-1 `299` (`2.09%`) & `2,126` (`14.88%`) [Build B: `299` & `3,190`], export gap $\Delta L = 0.0128$ & $0.0061$, $\Delta_{\text{top1}} = 53$ & $127$.
  - `Rec_FrozenRates_Null` ($ds_t \equiv 0$): D11 int top-1 **`1,443` (`10.10%`) & `1,446` (`10.12%`)** [Build B: **`338` (`2.37%`) & `1,872` (`13.10%`)**], float top-1 `1,463` & `830` [Build B: `303` & `2,010`].

## Checks run at the head

- `cargo test --manifest-path docs/labs/recurrence-gates-six-recurrence-fit/instrument/Cargo.toml --no-fail-fast`: `3 passed; 0 failed`.
- `cargo run --manifest-path docs/labs/recurrence-gates-six-recurrence-fit/instrument/Cargo.toml --bin verify-recurrence-six-fit -- --receipt docs/labs/recurrence-gates-six-recurrence-fit/receipt.json`: `exit 0` (`receipt_sha256=5835ff953a88d5f04e59ded194c13a91938cd7a472dfe2fc619cfbc96faaf69b`).
- `cargo test --manifest-path docs/labs/recurrence-gates-six-recurrence-fit/runner/Cargo.toml --no-fail-fast`: `2 passed; 0 failed`.
- `uor-harness loop gates --full`: `4/4 PASS`.
- `uor-harness loop report --check` and `uor-harness loop pr-check --local`: `PASS`.

## Skills used

- `uor-research-architect`, `scope-guard`, `solution-integrity`, `gates`, `context-guard`.

## Still open

- Roadmap item `recurrence-gates-six-recurrence-fit` (`#2890`) is complete (`status: done`): the six-recurrence fit ran after surrogate descent (`#2886`) and reported D11 integer DEV metrics, transport attribution, and synchronized CUDA `tokens/s` against `Rec_FrozenRates_Null` and `TrainSplit_CountTable_Ref`.
- For `recurrence-gates` to advance from `READY` to `ACTIVE`, a follow-on sheet would need to train with hard-forward discrete state/gate snapping (QAT) to close the multi-step recurrence rounding gap (`221–1,914` top-1 rows) and stabilize transport carry (`IdentityTransport`, $U(1)$, or icosian-snapped $S^3$).

## Plain summary

We tested the six-layer geometric recurrence stack (`pattern = "rrrrrr"`, where each layer carries memory forward using a table of 16 power-of-two fade rates $\lambda = 2^{-(i+1)}$, a matching keep table $\kappa = \sqrt{1 - \lambda^2}$, a clamped straight-line opening gate, and the `rate_select_bwd` secant credit rule, with zero softmax, squashing, or matrix multiplication) over `1,024` training steps on `mw-balp/train` across two random seeds and two independent CUDA builds on an RTX 5090, and exported every trained model to a multiplier-free D11 integer table artifact evaluated on all `14,287` response targets of `ft/dev`:
- **Speed (`tokens/s`):** The six-recurrence stack trains at `20,355–23,865 tokens/s`, evaluates `ft/dev` in float at `118,424–132,754 tokens/s`, and evaluates `ft/dev` through the multiplier-free D11 integer export at `106,155–112,804 tokens/s`.
- **What worked:** All four instrument checks passed (`4/4`), active arms shifted dyadic rate bins on `49.2%–57.3%` of recurrence lanes, and both the identity-transport (`Rec_Geometric_IdentityTransport_Control`, `2,570` and `2,222` D11 integer top-1 hits on Build A; `3,183` and `2,307` on Build B) and commutative $U(1)$ (`Rec_Geometric_U1_AbelianControl`, `2,805` and `1,921` on Build A; `1,798` and `2,101` on Build B) arms beat the frozen-rates control (`Rec_FrozenRates_Null`) and the training-split unigram count table (`1,226 / 14,287` = `8.58%`) on both seeds across both builds.
- **What did not work:** The unconstrained 4D quaternion rotation arm (`Rec_Geometric_S3_Active`) beat both controls on seed 2 (`1,999` and `2,889` D11 integer top-1 hits) but collapsed in top-1 accuracy on seed 1 (`352` and `313` hits) despite reducing `ft/dev` margin loss (`G-Rec-Sheet-Fit = FAIL`, `G-Rec-Transport = FAIL`), and rounding continuous float recurrence states to Q8/Q12 integers after training without discrete forward snapping during training cost `221–1,914` top-1 hits over `192` causal steps (`G-Rec-ExportGap = FAIL` on Build A / `PASS` on Build B).
