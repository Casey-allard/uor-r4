# M1 Stage 4: Export Gap and Prompt-Reading / Reply-Writing Throughput (`UORLUT01`)

- **Item:** [`m1-export-gap-and-toks`](../../roadmap/items/m1-export-gap-and-toks.md) ([#2839](https://github.com/UOR-Foundation/uor-r4/issues/2839), `PLAN.md` Step 6, Milestone [M1 #2029](https://github.com/UOR-Foundation/uor-r4/issues/2029)).
- **Lab / Unit:** `antigravity` ([`antigravity-96`](../../loop/tasks/m1-export-gap-and-toks/antigravity-96/REPORT.md), PR `#3287`).
- **Pre-registration:** [`docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md`](../../harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md) (`TEST FITNESS: FIT` and `D67 PRE-RUN VERDICT: GO` by fresh-context reviewer `ac559562-b8ce-4d82-b1d8-d8ae66015505` at `b287ccc3144c1a894c75066c02d0302138bc8b59`, [#2839 comment 6422279606](https://github.com/UOR-Foundation/uor-r4/issues/2839#issuecomment-6422279606)).
- **Instrument:** [`docs/labs/m1-d11-artifact-scorer/instrument/`](../m1-d11-artifact-scorer/instrument/) (`d11-scorer` binary SHA-256 `f4188313be8ea71120192aeee506b964ad28a51a3bc02edb825a0434a02f2461` on RTX 5090 `sm_120`, PTX SHA-256 `8b2a07c0766fe098024e8a0e355bc5aa480f8fe302c2697110317c0f16a2e2bf`, cubin SHA-256 `cd2c954b16ce9df76d1dd656216c9ede34b08987c693c1f654e5e7dfdedfa74b`).
- **HF Store Archive:** `caseyallard/uor-r4-store:results/antigravity/m1-export-gap-and-toks-2026-10-26/` ([`pod-SHA256SUMS`](evidence/pod-SHA256SUMS) `4292a3ce60fb219568d5fe5682801a7a8f3d953a1dbd29d2d7e7d0879718a17a`, [`pod-bin-SHA256SUMS`](evidence/pod-bin-SHA256SUMS) `497ecd5680c9715d8468b05d464234e5be0b1c2a8644373b740c09e13c1851dc`, [`laptop-SHA256SUMS`](evidence/laptop-SHA256SUMS) `9930f43957e5979ca100bdcd0292cb1b8303b71515cf0889e4a2e400723c61a3`).

---

## 1. Headline Result (Lead with `tok/s`, CHARTER §3.9)

All four Stage 4 bars (`B1` static zero-multiply instruction audits, `B2` GPU `14,287`-row DEV export gap, `B3` separate laptop prompt-reading and reply-writing throughput, and `B4` planted-fault negative control) **PASS**:

1. **Product serving throughput on client laptop (`B3`, CHARTER §3.6 — Apple M1, 4P+4E cores, 16 GiB RAM, macOS 27.0, AC power, `1` warmup + `N = 20` timed runs):**
   - **Stage T `FrozenZero` (`t16-frozen-zero-seed1` / `seed2`, relation gates all `0`):**
     - **Reply-writing (autoregressive decode, `64` tokens/run):** `12,630 tok/s` median (`5,067 µs`), `12,092 tok/s` `p95` (`5,292 µs`) on seed 1; `12,523 tok/s` median (`5,110 µs`), `11,616 tok/s` `p95` (`5,509 µs`) on seed 2.
     - **Prompt-reading (context prefill, `192` tokens/run):** `2,395,627 tok/s` median (`80 µs`), `2,262,150 tok/s` `p95` (`84 µs`) on seed 1; `2,789,359 tok/s` median (`68 µs`), `2,231,494 tok/s` `p95` (`86 µs`) on seed 2.
   - **Stage T `Relation` (`t16-relation-seed1` / `seed2`, active relation sweep over ternary tree nodes):**
     - **Reply-writing (autoregressive decode, `64` tokens/run):** `1,197 tok/s` median (`53,432 µs`), `702 tok/s` `p95` (`91,102 µs`) on seed 1; `1,118 tok/s` median (`57,226 µs`), `861 tok/s` `p95` (`74,322 µs`) on seed 2.
     - **Prompt-reading (context prefill, `192` tokens/run):** `373,889 tok/s` median (`513 µs`), `359,550 tok/s` `p95` (`534 µs`) on seed 1; `317,815 tok/s` median (`604 µs`), `119,409 tok/s` `p95` (`1,607 µs`) on seed 2.
2. **Export gap on the `14,287`-row DEV panel (`B2`, CHARTER §3.5 — 1x RTX 5090 pod `vast-57845148`):**
   - Across all four exported `UORLUT01` (`uor-r4.lut-discrete/1`) artifacts (`542,260` bytes each), the export gap against the sealed Stage T training-side CUDA `eval-corpus` receipts (`final-dev.json`) and against `IntegerDiscreteModel::observe_target` is **exact zero**:
     $$\Delta_{\text{top1}} = 0, \quad \Delta_{\text{top8}} = 0, \quad \Delta_{\text{adm}} = 0, \quad \Delta_{\text{loss\_ta}} = 0, \quad \Delta_{\text{loss\_tr}} = 0,$$
     with `14,287 / 14,287` row-by-row prediction agreement (`0` refused, `0` mismatched rows), well inside the registered two-seed noise band ($\Delta_{\text{noise,top1}} = 398$, $\Delta_{\text{noise,top8}} = 748$, $\Delta_{\text{noise,adm}} = 930$).
3. **Static zero-multiply instruction audits (`B1`):**
   - **Native ARM64 (`uor-r4-stack`, SHA-256 `83efb81e5f685a2e58587beda200adf4fbe2d6c0af2fa84df1689b77997088c5`):** `FULL PASS` across all `44` symbol ranges and `98` reachable call-graph functions (`Class I Multipliers = 0`, `Class II Dividers = 0`, `Class III Floats = 0`).
   - **WebAssembly (`uor_r4_integer_bg.wasm`, `416,816` bytes, SHA-256 `1332818275dfb9725415a61711e6cf11d80eb421758ae86df1e937c7d8168456`, [`b1-wasm-audit.txt`](evidence/b1-wasm-audit.txt)):** `7 / 7` serving symbols PASS (`mul = 0`, `div = 0`, `float = 0`, `unclassified = 0`).
4. **Planted-fault negative control (`B4`):**
   - On every artifact, container-level tamper of `discrete.cells` (`cells[0] ^= 1`) and out-of-bounds `discrete.routes` (`routes[0] ^= 0x40`) is rejected by `IntegerDiscreteModel::from_bytes`, `routes[0] ^= 1` changes `artifact_sha256`, and in-memory device/host perturbation produces `574` (`cells`) and `5,436` (`gates`) mismatched rows on `FrozenZero` and `458` (`cells`) and `7,189` (`gates`) mismatched rows on `Relation`.

---

## 2. GPU Export-Gap and Parity Table (`14,287` DEV Rows on RTX 5090 `vast-57845148`)

| Arm (`#2836`) | Exported `.lut` SHA-256 (`542,260` B) | `top1` (Train / D11 / $\Delta$) | `top8` (Train / D11 / $\Delta$) | `admitted` (Train / D11 / $\Delta$) | `loss_ta` num (Train / D11 / $\Delta$) | `loss_tr` num (Train / D11 / $\Delta$) | Row Agreement (`observe_target` & `final-dev.json`) | Planted-Fault `B4` (`cells` / `gates` mismatches) | GPU Kernel Time |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `t16-frozen-zero-seed1` | `fc31bcc7a88250f3…` | `455 / 455 / 0` | `1,484 / 1,484 / 0` | `5,009 / 5,009 / 0` | `12,920,928 / 12,920,928 / 0` | `29,531,840 / 29,531,840 / 0` | `14,287 / 14,287` (`0` refused) | PASS (`574` / `5,436` rows) | `42,766 µs` |
| `t16-frozen-zero-seed2` | `a75ea7f5907275fd…` | `455 / 455 / 0` | `1,484 / 1,484 / 0` | `5,009 / 5,009 / 0` | `12,920,928 / 12,920,928 / 0` | `29,531,840 / 29,531,840 / 0` | `14,287 / 14,287` (`0` refused) | PASS (`574` / `5,436` rows) | `44,790 µs` |
| `t16-relation-seed1` | `d64e127c79c1e3be…` | `371 / 371 / 0` | `1,124 / 1,124 / 0` | `3,827 / 3,827 / 0` | `13,604,128 / 13,604,128 / 0` | `31,562,816 / 31,562,816 / 0` | `14,287 / 14,287` (`0` refused) | PASS (`458` / `7,189` rows) | `366,019 µs` |
| `t16-relation-seed2` | `c21d992fb4a70b93…` | `371 / 371 / 0` | `1,124 / 1,124 / 0` | `3,827 / 3,827 / 0` | `13,604,128 / 13,604,128 / 0` | `31,562,816 / 31,562,816 / 0` | `14,287 / 14,287` (`0` refused) | PASS (`458` / `7,189` rows) | `365,261 µs` |

---

## 3. Client Laptop Product Serving Throughput (`B3`, CHARTER §3.6)

- **Hardware profile:** Apple M1 (`8` physical cores: `4` performance, `4` efficiency), `17,179,869,184` bytes (`16 GiB`) RAM, macOS `27.0` (`26A5378n`), on AC power (`80%` battery, not charging).
- **Protocol:** Deterministic `192`-token prompt (`token i = (7919 * i + 13) mod 4096`, SHA-256 `664c3be9607689a2af2aeedadef32891a55bed4712c2f5ee0f54f853cadeb787`), `1` warmup iteration + `N = 20` timed iterations for prompt-reading (`evaluate_prefix` over `192` tokens) and `1` warmup iteration + `N = 20` timed iterations for reply-writing (`64` autoregressive decode steps, sliding the `192`-token window).

### Summary Table

| Arm | Reply-Writing Decode Median (`64` tok) | Reply-Writing Decode `p95` (rank `19/20`) | Prompt-Reading Prefill Median (`192` tok) | Prompt-Reading Prefill `p95` (rank `19/20`) | Deterministic Reply SHA-256 (`21/21` identical) |
|---|---:|---:|---:|---:|---|
| `t16-frozen-zero-seed1` | `12,630 tok/s` (`5,067,187 ns`) | `12,092 tok/s` (`5,292,333 ns`) | `2,395,627 tok/s` (`80,146 ns`) | `2,262,150 tok/s` (`84,875 ns`) | `48feb9d729845a77…` |
| `t16-frozen-zero-seed2` | `12,523 tok/s` (`5,110,542 ns`) | `11,616 tok/s` (`5,509,583 ns`) | `2,789,359 tok/s` (`68,833 ns`) | `2,231,494 tok/s` (`86,041 ns`) | `48feb9d729845a77…` |
| `t16-relation-seed1` | `1,197 tok/s` (`53,432,021 ns`) | `702 tok/s` (`91,102,667 ns`) | `373,889 tok/s` (`513,521 ns`) | `359,550 tok/s` (`534,000 ns`) | `f3cbe66f56287985…` |
| `t16-relation-seed2` | `1,118 tok/s` (`57,226,500 ns`) | `861 tok/s` (`74,322,458 ns`) | `317,815 tok/s` (`604,125 ns`) | `119,409 tok/s` (`1,607,916 ns`) | `f3cbe66f56287985…` |

### Raw 20-Run Timing Samples (`ns`)

- **`t16-frozen-zero-seed1`** (`t16-frozen-zero-seed1-toks.json`, SHA-256 `d989c702d6f99698366901f73688f5c5f92c6e54db954a1a3869c5aa1fc0baf3`):
  - `prefill.raw_ns`: `[104333, 84875, 81291, 80250, 79708, 84208, 80000, 81375, 79917, 79625, 83208, 81625, 79750, 79625, 79417, 81625, 80042, 80250, 79583, 79875]`
  - `decode.raw_ns`: `[5800292, 5292333, 5250208, 5109833, 5124958, 5082958, 5062208, 5066834, 5068666, 5063291, 5064042, 5065542, 5073500, 5067041, 5067334, 5218167, 5065125, 5062291, 5056708, 5039459]`
- **`t16-frozen-zero-seed2`** (`t16-frozen-zero-seed2-toks.json`, SHA-256 `7928b579939a56456bac9e2d9e1031723a3983deacb9f9c523e874b22d4449ec`):
  - `prefill.raw_ns`: `[86041, 72708, 70292, 69292, 68791, 68708, 67750, 67041, 66625, 91584, 69833, 70042, 66375, 68667, 69042, 69833, 68875, 68584, 68500, 68625]`
  - `decode.raw_ns`: `[5117834, 5557458, 5509583, 5222583, 5088083, 5130292, 5079583, 5262708, 5223084, 5408208, 5128083, 5237541, 5100417, 5089667, 5087584, 5103250, 5082458, 5073584, 5099041, 5079791]`
- **`t16-relation-seed1`** (`t16-relation-seed1-toks.json`, SHA-256 `d33b314426aff2275431662b83fd60292529953642dce3da81de08daa05bcb2e`):
  - `prefill.raw_ns`: `[558250, 534000, 532875, 515541, 511833, 515417, 513209, 522916, 511208, 520708, 513834, 514917, 508750, 521625, 509167, 511167, 512667, 509458, 509375, 508791]`
  - `decode.raw_ns`: `[33445958, 33878792, 105533958, 40191125, 36163875, 86899667, 54694916, 54618709, 54896834, 55081167, 75448417, 91102667, 51672750, 56636583, 52245334, 48914083, 47661917, 46815417, 51647791, 54918667]`
- **`t16-relation-seed2`** (`t16-relation-seed2-toks.json`, SHA-256 `ec730ff63f3d2566c9be2e90b12e5d87d45218de59b95fa7c1c0fa8b13bcdc92`):
  - `prefill.raw_ns`: `[1257708, 1792500, 1088541, 581416, 613292, 567459, 588209, 603708, 608333, 549833, 544500, 1406584, 629584, 604542, 554166, 540292, 548000, 537625, 1044500, 1607916]`
  - `decode.raw_ns`: `[66539375, 59754250, 56516709, 63135459, 60669958, 55643958, 60631958, 49348000, 50335041, 57936292, 81799833, 54889292, 59648625, 74322458, 51038833, 43735791, 59222042, 55973917, 43916834, 41372958]`
