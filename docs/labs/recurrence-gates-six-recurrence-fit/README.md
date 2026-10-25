# Six-Recurrence Fit for Geometric Recurrence Gates: Dyadic Tables, Transport Attribution, Tokens/s, and D11 Integer Export Gap

- **Roadmap item:** [`recurrence-gates-six-recurrence-fit`](../../roadmap/items/recurrence-gates-six-recurrence-fit.md) (`#2890`)
- **Pre-registration sheet:** [`docs/harness/prep/recurrence-gates-six-recurrence-fit-2026-10-25.md`](../../harness/prep/recurrence-gates-six-recurrence-fit-2026-10-25.md) (`TEST FITNESS: FIT`, D68 `DECISION REVIEW: APPROVE`, D67 `PRE-RUN REVIEW: PASS`)
- **Task folder:** [`docs/loop/tasks/recurrence-gates-six-recurrence-fit/antigravity-87/`](../../loop/tasks/recurrence-gates-six-recurrence-fit/antigravity-87/)
- **Sealed HF store artifacts:** `caseyallard/uor-r4-store:results/antigravity/recurrence-gates-six-recurrence-fit-2026-10-25/`
  - [`receipt.json`](./receipt.json) (Build A, committed): `5835ff953a88d5f04e59ded194c13a91938cd7a472dfe2fc619cfbc96faaf69b`
  - `receipt-build-b.json` (Build B): `04b9c409d783f8fa77443a86975adaa6583cc9cfbb05d8ca9042bfb41c43eef9`
  - `synthetic-probe.json`: `b0f98cfbc2321a1e4008d7a046efdac7f82e0bb8af054fc2ed6f8e0362aa7040`
  - `throughput-probe.json`: `1b91515c3b057516dc1430c83262401905ac9a1197c8dab063c9c3b5bf5fa53f`
  - `binary-SHA256SUMS`: `f320fc8e0f3cd0572e15b408dfe0e84699fd4812ab2081729bfa43455a4c63c5` (`recurrence-six-fit-a` and `recurrence-six-fit-b` byte-identical: `9737cd8f88bdc0aba2e0998373023729ab7445fa04c8f89162606901ec44fda3`)

## 1. Negatives and Gate Outcomes First (`CHARTER.md` §4.1, §5.10)

All four pre-registered instrument checks (`I-Standalone-CUDA-Parity`, `I-DEV-Denominator`, `I-Rate-Mobility-And-Plateau`, and `I-Export-Exactness`) **PASS** on both clean `nvcc` builds (`4/4` green), establishing a non-vacuous measurement (`CHARTER.md` §4.2). Under those verified instruments, the `1,024`-step (`786,432`-token) six-recurrence `StackAdamW` training run on `mw-balp/train` evaluated on all `14,287` eligible response targets of `ft/dev` (`400` documents, `428` causal windows) produces a **mixed** outcome:

1. **`G-Rec-Sheet-Fit = FAIL` (non-abelian $S^3$ active arm fails two-seed replication despite $U(1)$ and `IdentityTransport` passing on both seeds across both builds):**
   - While `Rec_Geometric_U1_AbelianControl` (`2,805` [`19.63%`] & `1,921` [`13.45%`] D11 integer top-1 on Build A; `1,798` [`12.58%`] & `2,101` [`14.71%`] on Build B) and `Rec_Geometric_IdentityTransport_Control` (`2,570` [`17.99%`] & `2,222` [`15.55%`] on Build A; `3,183` [`22.28%`] & `2,307` [`16.15%`] on Build B) beat both `Rec_FrozenRates_Null` ($c - b \ge F_I$ paired McNemar advantage) and `TrainSplit_CountTable_Ref` (`1,226 / 14,287` = `8.58%` top-1) on **both seeds across both builds**, the non-abelian quaternion carry arm `Rec_Geometric_S3_Active` passes on Seed 2 (`1,999 / 14,287` = `13.99%` D11 integer top-1 on Build A, `2,889 / 14,287` = `20.22%` on Build B) but collapses in top-1 accuracy on Seed 1 (`352 / 14,287` = `2.46%` D11 integer top-1 on Build A, `313 / 14,287` = `2.19%` on Build B; `299 / 14,287` = `2.09%` in float) even though its `ft/dev` `token_margin` hinge loss descends cleanly (`1.0024 -> 0.8664`, beating `Rec_FrozenRates_Null` `0.8747`).
   - Per `CHARTER.md` §4.1, this negative closes the unconstrained `1,024`-step `StackAdamW` configuration of `Rec_Geometric_S3_Active`, never the `recurrence-gates` mechanism itself (`status: READY`).
2. **`G-Rec-Transport = FAIL` (`IdentityTransport` and $U(1)$ outperform non-abelian $S^3$ carry at `1,024` steps):**
   - `Rec_Geometric_IdentityTransport_Control` (`rotation = false`, $u_t \equiv (1, 0, 0, 0)$) achieves the highest mean D11 integer DEV top-1 accuracy (`2,396.0` rows = `16.77%` on Build A; `2,745.0` rows = `19.21%` on Build B; up to `4,484 / 14,287` = `31.39%` top-1 and `5,572 / 14,287` = `39.00%` top-8 in float), followed by commutative `Rec_Geometric_U1_AbelianControl` (`2,363.0` rows = `16.54%` on Build A; `1,949.5` rows = `13.65%` on Build B). Unconstrained 4D $S^3$ unit-quaternion carry does not beat identity or abelian $U(1)$ carry on both seeds at this horizon.
3. **`G-Rec-ExportGap = FAIL` on Build A / `PASS` on Build B (post-training Q8/Q12 rounding degrades unconstrained float top-1 by `221–1,914` rows):**
   - Direct post-training D11 integer table export (exact bit-shift `>> (i + 1)` for $\lambda = 2^{-(i+1)}$, exact 3-term CSD Q12 keep table $\kappa_{\text{Q12}}$, exact 77-pair shift-add Q8 transport, and Q12 `i32` `NativeVectorWriter`, zero float/multiply instructions) preserves `ft/dev` `token_margin` loss within the two-seed noise band on every arm and seed ($\Delta L \in [0.0050, 0.0304] \le \Delta_{\text{noise},L} = 0.0586$ on Build A / $0.0558$ on Build B).
   - However, because continuous `f32` recurrence training accumulates un-snapped trajectory drift across `192` causal steps and `6` layers before post-training rounding, the top-1 export gap on the winning transport mode (`Rec_Geometric_IdentityTransport_Control`) is $\Delta_{\text{top1}} = 1,914$ rows (`4,484` float vs `2,570` D11 int) on Seed 1 and `221` rows (`2,443` vs `2,222`) on Seed 2 in Build A ($\Delta_{\text{top1}} = 1,914 > \Delta_{\text{noise,top1}} = 1,647 \implies \text{FAIL}$), and $\Delta_{\text{top1}} = 1,406$ rows (`4,589` vs `3,183`) on Seed 1 and `349` rows (`2,656` vs `2,307`) on Seed 2 in Build B ($\Delta_{\text{top1}} = 1,406 \le \Delta_{\text{noise,top1}} = 1,554 \implies \text{PASS}$). Disqualifying direct post-training integer rounding on Build A (`CHARTER.md` §3.7) shows that lossless D11 export of multi-layer recurrence requires hard-forward discrete state/gate snapping during training (analogous to `Hamilton_IcosianHard` in `#2824`, which achieved `97.59%–97.84%` float-vs-integer agreement).
4. **`G-Rec-Transfer = SKIPPED_DEV_GATE_DID_NOT_PASS`:**
   - Because `G-Rec-Sheet-Fit` did not pass on both seeds, the sealed `14,479`-target `fresh_v2` transfer draw was not opened (`fresh_v2_evaluated = false`).

---

## 2. Lead Metric: Synchronized CUDA `tokens/s` on RTX 5090 (`CHARTER.md` §1)

Measured on 1 $\times$ NVIDIA GeForce RTX 5090 (`sm_120`, Vast.ai pod `vast-57719671`) with `cudaDeviceSynchronize()` before and after each timed region (`786,432` training tokens per `1,024`-step run at `batch = 4, context = 192`; `82,176` causal context tokens across `428` windows scoring `14,287` response targets on `ft/dev`):

| Arm | Build A Train `tok/s` (Seed 1 / Seed 2) | Build B Train `tok/s` (Seed 1 / Seed 2) | Build A Float DEV `tok/s` (Seed 1 / Seed 2) | Build A D11 Int DEV `tok/s` (Seed 1 / Seed 2) | Build B D11 Int DEV `tok/s` (Seed 1 / Seed 2) |
|---|---:|---:|---:|---:|---:|
| `Rec_Geometric_S3_Active` | `22,140.6` / `21,878.7` | `20,354.9` / `20,975.8` | `130,875.6` / `127,821.0` | `111,841.3` / `106,427.3` | `106,660.1` / `106,154.9` |
| `Rec_Geometric_U1_AbelianControl` | `21,996.2` / `21,566.8` | `21,499.4` / `21,366.6` | `129,356.9` / `129,058.5` | `110,411.3` / `109,851.6` | `106,496.3` / `106,421.7` |
| `Rec_Geometric_IdentityTransport_Control` | `22,668.3` / `22,593.0` | `22,607.8` / `22,571.6` | `131,857.9` / `132,753.5` | `112,803.5` / `112,051.4` | `108,732.3` / `108,650.2` |
| `Rec_FrozenRates_Null` | `23,812.0` / `23,865.2` | `20,906.8` / `20,918.2` | `130,275.9` / `129,937.8` | `110,288.2` / `108,572.4` | `106,419.1` / `106,488.5` |

---

## 3. Instrument Checks (`I-1..4` — `CHARTER.md` §4.2)

| Instrument Check | Pre-registered Bar | Build A Result | Build B Result | Verdict |
|---|---|---|---|---|
| `I-Standalone-CUDA-Parity` | Rate error $\le 5\times 10^{-13}$, keep error $\le 4.6\times 10^{-10}$, secant error $\le 5.5\times 10^{-8}$, scan fwd/bwd error $\le 10^{-5}$, stack grad diff $\le 10^{-5}$, nonzero step-0 $\|ds_l\|_2$ on all 6 layers | Rate `0.0`, keep `0.0`, secant `0.0`, scan fwd `5.00e-8`, scan bwd `4.68e-8`, stack hidden diff `0.0`, stack grad diff `3.81e-6`, $\|ds_l\|_2 \in [0.0470, 0.9581]$ on all 6 layers (`0.0` on `FrozenRates`) | Identical (`0.0`, `0.0`, `0.0`, `5.00e-8`, `4.68e-8`, `0.0`, `3.81e-6`) | **PASS** |
| `I-DEV-Denominator` | `400` documents, `428` windows, `14,287` eligible response targets scored in every float and D11 integer evaluation | `400` docs, `428` windows, `14,287 / 14,287` targets across all 16 float & D11 evaluations | `400` docs, `428` windows, `14,287 / 14,287` targets across all 16 float & D11 evaluations | **PASS** |
| `I-Rate-Mobility-And-Plateau` | Active arms shift rate bins on $\ge 5.0\%$ (`19/384`) of lanes while $< 80\%$ of lane-tokens occupy plateau bins `12..15` | `49.74%–56.77%` (`191–218 / 384`) lane crossings; plateau `12..15` fraction `40.12%–43.15%` (`0 / 384` crossings on `FrozenRates`) | `49.22%–57.29%` (`189–220 / 384`) lane crossings; plateau `12..15` fraction `40.09%–43.15%` (`0 / 384` crossings on `FrozenRates`) | **PASS** |
| `I-Export-Exactness` | `16/16` exact 3-term CSD Q12 keep entries, `77/77` exact shift-add Q8 pairs, `max_scan_lsb_diff == 0` on synthetic fixtures, `0 / 517` Q12 `i32` writer mismatches per eval (`0 / 16,544` across all `32` D11 evals) | `16/16` CSD entries, `77/77` Q8 pairs, `max_scan_lsb_diff = 0`, `d11_layer0_oracle_max_lsb_diff = 4..6` Q12 LSBs, `0 / 16,544` (`517` $\times$ `32` evals) Q12 `i32` writer mismatches | `16/16` CSD entries, `77/77` Q8 pairs, `max_scan_lsb_diff = 0`, `d11_layer0_oracle_max_lsb_diff = 4..6` Q12 LSBs, `0 / 16,544` (`517` $\times$ `32` evals) Q12 `i32` writer mismatches | **PASS** |

---

## 4. Full Six-Recurrence Quality & Export-Gap Results (`14,287` `ft/dev` Targets)

### 4.1 Build A (`receipt.json` `5835ff953a88d5f04e59ded194c13a91938cd7a472dfe2fc619cfbc96faaf69b`)

- **Two-seed noise bands (`CHARTER.md` §4.3):** $\Delta_{\text{noise},L,\text{f32}} = 0.052375$, $\Delta_{\text{noise},L,\text{int}} = 0.058594$, $\Delta_{\text{noise},\text{top1},\text{int}} = 1,647$ rows (`11.53 pp`).

| Arm | Seed | Train Loss (`0 -> 1024`) | Lane Bin Crossings (`/ 384`) | Float DEV Loss (`0 -> 1024`) | Float DEV Top-1 (`/ 14,287`) | Float DEV Top-8 (`/ 14,287`) | D11 Int DEV Loss | D11 Int DEV Top-1 (`/ 14,287`) | D11 Int DEV Top-8 (`/ 14,287`) | Export Gap $\Delta L$ / $\Delta_{\text{top1}}$ | Float-Int Row Agreement |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `TrainSplit_CountTable_Ref` | — | — | — | — | `1,226` (`8.58%`) | `4,060` (`28.42%`) | — | **`1,226` (`8.58%`)** | `4,060` (`28.42%`) | — | — |
| `Rec_Geometric_S3_Active` | `1` | `2.8033 -> 1.8605` | `201` (`52.34%`) | `1.0024 -> 0.8664` | `299` (`2.09%`) | `493` (`3.45%`) | `0.8792` | **`352` (`2.46%`)** | `630` (`4.41%`) | `+0.0128` / `53` | `33.79%` (`4,828`) |
| `Rec_Geometric_S3_Active` | `2` | `2.7332 -> 1.8547` | `208` (`54.17%`) | `1.0022 -> 0.8632` | `2,126` (`14.88%`) | `2,543` (`17.80%`) | `0.8693` | **`1,999` (`13.99%`)** | `2,419` (`16.93%`) | `+0.0061` / `127` | `44.75%` (`6,393`) |
| `Rec_Geometric_U1_AbelianControl` | `1` | `2.7971 -> 1.8280` | `191` (`49.74%`) | `1.0024 -> 0.8566` | `3,988` (`27.91%`) | `5,063` (`35.44%`) | `0.8652` | **`2,805` (`19.63%`)** | `3,635` (`25.44%`) | `+0.0086` / `1,183` | `34.30%` (`4,901`) |
| `Rec_Geometric_U1_AbelianControl` | `2` | `2.7442 -> 1.8975` | `213` (`55.47%`) | `1.0023 -> 0.9089` | `2,740` (`19.18%`) | `3,092` (`21.64%`) | `0.9238` | **`1,921` (`13.45%`)** | `2,332` (`16.32%`) | `+0.0149` / `819` | `32.70%` (`4,672`) |
| `Rec_Geometric_IdentityTransport_Control` | `1` | `2.8108 -> 1.8468` | `191` (`49.74%`) | `1.0025 -> 0.8941` | `4,484` (`31.39%`) | `5,572` (`39.00%`) | `0.9245` | **`2,570` (`17.99%`)** | `3,319` (`23.23%`) | `+0.0304` / `1,914` | `22.11%` (`3,159`) |
| `Rec_Geometric_IdentityTransport_Control` | `2` | `2.7437 -> 1.8859` | `218` (`56.77%`) | `1.0023 -> 0.9084` | `2,443` (`17.10%`) | `3,078` (`21.54%`) | `0.9159` | **`2,222` (`15.55%`)** | `2,840` (`19.88%`) | `+0.0075` / `221` | `40.72%` (`5,817`) |
| `Rec_FrozenRates_Null` | `1` | `2.8033 -> 1.8695` | `0` (`0.00%`) | `1.0024 -> 0.8747` | `1,463` (`10.24%`) | `1,669` (`11.68%`) | `0.8851` | **`1,443` (`10.10%`)** | `1,676` (`11.73%`) | `+0.0104` / `20` | `34.65%` (`4,950`) |
| `Rec_FrozenRates_Null` | `2` | `2.7332 -> 1.8712` | `0` (`0.00%`) | `1.0022 -> 0.8770` | `830` (`5.81%`) | `1,089` (`7.62%`) | `0.8836` | **`1,446` (`10.12%`)** | `1,724` (`12.07%`) | `+0.0066` / `616` | `31.08%` (`4,440`) |

### 4.2 Build B (`receipt-build-b.json` `04b9c409d783f8fa77443a86975adaa6583cc9cfbb05d8ca9042bfb41c43eef9`)

- **Two-seed noise bands (`CHARTER.md` §4.3):** $\Delta_{\text{noise},L,\text{f32}} = 0.048046$, $\Delta_{\text{noise},L,\text{int}} = 0.055825$, $\Delta_{\text{noise},\text{top1},\text{int}} = 1,554$ rows (`10.88 pp`).

| Arm | Seed | Train Loss (`0 -> 1024`) | Lane Bin Crossings (`/ 384`) | Float DEV Loss (`0 -> 1024`) | Float DEV Top-1 (`/ 14,287`) | Float DEV Top-8 (`/ 14,287`) | D11 Int DEV Loss | D11 Int DEV Top-1 (`/ 14,287`) | D11 Int DEV Top-8 (`/ 14,287`) | Export Gap $\Delta L$ / $\Delta_{\text{top1}}$ |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `Rec_Geometric_S3_Active` | `1` | `2.8033 -> 1.8565` | `204` (`53.12%`) | `1.0024 -> 0.8588` | `299` (`2.09%`) | `460` (`3.22%`) | `0.8712` | **`313` (`2.19%`)** | `527` (`3.69%`) | `+0.0124` / `14` |
| `Rec_Geometric_S3_Active` | `2` | `2.7332 -> 1.8492` | `220` (`57.29%`) | `1.0022 -> 0.8655` | `3,190` (`22.33%`) | `3,718` (`26.02%`) | `0.8732` | **`2,889` (`20.22%`)** | `3,371` (`23.59%`) | `+0.0077` / `301` |
| `Rec_Geometric_U1_AbelianControl` | `1` | `2.7971 -> 1.8300` | `189` (`49.22%`) | `1.0024 -> 0.8585` | `2,519` (`17.63%`) | `3,124` (`21.87%`) | `0.8700` | **`1,798` (`12.58%`)** | `2,243` (`15.70%`) | `+0.0115` / `721` |
| `Rec_Geometric_U1_AbelianControl` | `2` | `2.7442 -> 1.8946` | `202` (`52.60%`) | `1.0023 -> 0.9065` | `2,869` (`20.08%`) | `3,309` (`23.16%`) | `0.9258` | **`2,101` (`14.71%`)** | `2,591` (`18.14%`) | `+0.0193` / `768` |
| `Rec_Geometric_IdentityTransport_Control` | `1` | `2.8108 -> 1.8496` | `201` (`52.34%`) | `1.0025 -> 0.8962` | `4,589` (`32.12%`) | `5,693` (`39.85%`) | `0.9218` | **`3,183` (`22.28%`)** | `3,962` (`27.73%`) | `+0.0256` / `1,406` |
| `Rec_Geometric_IdentityTransport_Control` | `2` | `2.7437 -> 1.8825` | `207` (`53.91%`) | `1.0023 -> 0.9061` | `2,656` (`18.59%`) | `3,551` (`24.85%`) | `0.9111` | **`2,307` (`16.15%`)** | `3,184` (`22.29%`) | `+0.0050` / `349` |
| `Rec_FrozenRates_Null` | `1` | `2.8033 -> 1.8691` | `0` (`0.00%`) | `1.0024 -> 0.8694` | `303` (`2.12%`) | `461` (`3.23%`) | `0.8805` | **`338` (`2.37%`)** | `537` (`3.76%`) | `+0.0111` / `35` |
| `Rec_FrozenRates_Null` | `2` | `2.7332 -> 1.8705` | `0` (`0.00%`) | `1.0022 -> 0.8758` | `2,010` (`14.07%`) | `2,331` (`16.32%`) | `0.8823` | **`1,872` (`13.10%`)** | `2,175` (`15.22%`) | `+0.0065` / `138` |

---

## 5. Reproduction Commands

### Host CPU Verification (Pure-Integer Instrument + D37 Runner Tests)

```sh
# 1. Pure-integer D32-compliant D11 recurrence exporter & receipt verifier
~/.cargo/bin/cargo test --manifest-path docs/labs/recurrence-gates-six-recurrence-fit/instrument/Cargo.toml --no-fail-fast
~/.cargo/bin/cargo run --manifest-path docs/labs/recurrence-gates-six-recurrence-fit/instrument/Cargo.toml --bin verify-recurrence-six-fit -- \
  --receipt docs/labs/recurrence-gates-six-recurrence-fit/receipt.json

# 2. D37 offline-float six-recurrence runner unit tests
~/.cargo/bin/cargo test --manifest-path docs/labs/recurrence-gates-six-recurrence-fit/runner/Cargo.toml --no-fail-fast
```

### GPU Execution on RTX 5090 (`scripts/pod/uor-pod`)

```sh
~/.cargo/bin/cargo build --manifest-path docs/labs/recurrence-gates-six-recurrence-fit/runner/Cargo.toml --release --features cuda
./target/release/recurrence-six-fit \
  --mode full \
  --train-tokens /workspace/w2-inputs/2026-10-25/driver-data/mw-balp/train/tokens.u16 \
  --train-mask /workspace/w2-inputs/2026-10-25/driver-data/mw-balp/train/response_mask.u8 \
  --dev-tokens /workspace/w2-inputs/2026-10-25/driver-data/ft/dev/tokens.u16 \
  --dev-mask /workspace/w2-inputs/2026-10-25/driver-data/ft/dev/response_mask.u8 \
  --tokenizer /workspace/w2-inputs/2026-10-25/driver-data/tokenizer.json \
  --out /workspace/results/receipt.json
```
