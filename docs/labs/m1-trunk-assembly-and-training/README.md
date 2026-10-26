# Stage 3 Trunk Assembly and Training (`m1-trunk-assembly-and-training`, `#2844`)

- **Date**: 2026-10-26
- **Lab**: `antigravity` (`antigravity-89`, PR `#3258`)
- **Milestone**: M2 (`#2030`, effective M1 blocker on `#2833`)
- **Roadmap Item**: [`docs/roadmap/items/m1-trunk-assembly-and-training.md`](../../roadmap/items/m1-trunk-assembly-and-training.md) (`#2844`)
- **Pre-Registration Sheet**: [`docs/harness/prep/m1-trunk-assembly-and-training-2026-10-25.md`](../../harness/prep/m1-trunk-assembly-and-training-2026-10-25.md) (`sha256 = bbb641f9115634b03d319b8937d534ed987ebfa18f3d3e73df8ff45d9f554ce0`; pre-stamp `b49051c0f5ef3294089b21dd405d247341c1cf28bd179b6807870eb9373afb07`)
- **Reviews**:
  - `TEST FITNESS: FIT` + `DECISION REVIEW (D68): APPROVE` + `PRE-RUN REVIEW (D67): APPROVE` by independent fresh-context reviewer `65458246-8c07-489e-85d5-935d01db8259` ([`#2844` comment `6408200462`](https://github.com/UOR-Foundation/uor-r4/issues/2844#issuecomment-6408200462); [`DECISION.md`](../../loop/tasks/m1-trunk-assembly-and-training/antigravity-89/DECISION.md))
- **GPU Pod & Cost**: `vast-57764453` (`1x NVIDIA GeForce RTX 5090`, `\$0.563/h`, leased via `scripts/pod/uor-pod`, `~4.4h` total wall time $\approx \$2.48$, within the registered `\$3.00` cap)
- **HF Store Archive**: `caseyallard/uor-r4-store:results/antigravity/m1-trunk-assembly-and-training-2026-10-25/trunk-20261025.tar.gz` (`sha256 = 4f4d4817e2750565f65bdb8a6c6f8ed07b44389f0757702cf6eba11b64e23d35`, with `SHA256SUMS`)

---

## 1. Executive Summary & Stage 3 Verdict

We assembled the rules-compliant whole-number geometric trunk (`TrunkAssemblyV1` in [`crates/uor-r4-discrete/src/trunk.rs`](../../../crates/uor-r4-discrete/src/trunk.rs) and [`crates/uor-r4-discrete/src/bin/trunk-run.rs`](../../../crates/uor-r4-discrete/src/bin/trunk-run.rs)), combining:
1. **Stage 1 — Causal Read + Prefix-Holonomy Position + Normalization**: `Device::forward` and `Device::evaluate` on `Relation { kept: 8, sweeps: 1 }` (`T16_rel`) and `FrozenZeroV1` (`T16_fz`), producing the 32-int read vector $q^{\text{read}}(x) \in \mathcal{O}_{16}^4$ (`read_code` $c_{\text{read}}(x)$) and the 8-cell outer-L1 admitted shortlist $A_W(x) \subset \{0..255\}$.
2. **Stage 2 — Causal Quaternion / Clifford Prefix-Holonomy Recurrence (`DiscreteQuaternionRecurrenceV1`) + Multiplicity-Matched Normalization (`OrbitRootSnapNormV1`)**: 64-entry whole-number `PositionStepTable` (`blocks = 4`) accumulating causal prefix holonomy $F_{p,b} \in \mathrm{Cl}(0,6)$ across $p \in [0, L)$, transporting predecessor (`L - 2`), mid-window (`(L - 1)/2`), and window-start (`0`) roots by exact multiscale relative holonomy ($H_{\text{prev},b}, H_{\text{mid},b}, F_{L-1,b}$), summing with the query root $\rho(r_{L-1,b})$, and projecting onto $\mathcal{O}_{16}$ via scale-matched `OrbitRootSnapNormV1` (`multiplicity = 4`), yielding the 16-bit recurrence state $c_{\text{rec}}(x) \in \{0..65535\}$.
3. **Stage 3 — Non-Linear Geometric Feed-Forward Gate (`DiscreteHamiltonGateV1`) + Unmodified Single-Vector `OuterL1V1` Writer + `D11` Export**: Transitive $\mathrm{Cl}(0,6)$ frame table `geometry::BFS[r_b * 16 + t_b]` rotating $q^{\text{read}}(x)$ into **one single 32-int trunk output vector** $q^{\text{trunk}}_L(x) \in \mathcal{O}_{16}^4 \subset \{-2, 0, 2\}^{32}$ per row, verified on $\mathcal{O}_{16}^4$ (`BlockNormInvariants::is_canonical_unit_root()`) and scored directly by the unmodified `OuterL1V1` writer (`writer_score_admitted_fast` and full two-stage `writer_score_full_outer_l1`), with round-trip verified `D11` (`uor-r4.lut-discrete/1`) integer artifact export (`trunk-model.d11`, `542,260` bytes per arm).

### Headline Outcomes ([`grade.json`](grade.json))

- **Decode / Context Throughput (Lead Metric, CHARTER §3.9, §4.7)**:
  - **`T16_fz` (`FrozenZeroV1`, Seeds 1 & 2)**: **`76,316–76,343 context tok/s`** on `1,049,856` TRAIN rows (`1,229.6–1,230.1 s`), **`34,325–34,337 context tok/s`** on `ft/dev` (`14,287` rows, `32.1 s`), and **`37,838–37,861 context tok/s`** on `fresh_v2` (`14,287` rows, `34.7 s`).
  - **`T16_rel` (`LearnV1`, Seeds 1 & 2)**: **`16,608–16,609 context tok/s`** on `1,049,856` TRAIN rows (`5,651.8–5,652.1 s`), **`8,530–8,542 context tok/s`** on `ft/dev` (`129.0–129.2 s`), and **`8,326–8,336 context tok/s`** on `fresh_v2` (`157.6–157.8 s`).
- **Stage 3 Bar vs Stage 2 Baselines across Two Seeds (`Seed 1`, `Seed 2`) on `ft/dev` (`14,287` rows) and `fresh_v2` (`14,287` rows)**:
  1. **`beats_stage2_writer_on_dev_and_fresh_both_seeds = true`**:
     - On `T16_fz`, single-vector `top-1` rises from Stage 2's best writer **`455 / 14,287`** (`3.18%`, and `#2831` `K = 1` Q1's `871 / 14,287`) to **`1,598`** (`Level 1_global`, no shortlist), **`2,054`** (`Level 1`), **`2,087`** (`Level 2`), and **`2,988` (Seed 1) / `2,989` (Seed 2) `/ 14,287` (`20.92%`, `top-8 = 3,247` / `3,248`)** on `ft/dev` (**`+2,533` rows / `6.57x` Stage 2's `455`**, **`+2,117` rows over `#2831`'s `871`**, **`+1,762` rows over unigram `1,226`**, and **`+1,506` rows over W2's `G_restr(Q4) = 1,482`**, against seed-to-seed noise `1` row and `2-SE` floor `84` rows).
     - On the sealed fresh transfer panel **`fresh_v2` (`14,287` rows)**, `T16_fz` rises from `Level 0 = 457 / 14,287` (`3.20%`) to **`1,729`** (`Level 1_global`), **`2,187`** (`Level 1`), **`2,207` (Seed 1) / `2,225` (Seed 2)** (`Level 2`), and **`3,163 / 14,287` (`22.14%`, `top-8 = 3,415` / `3,416`)** on both seeds (**`+2,706` rows / `6.92x` `Level 0`**, seed-to-seed noise `0` rows, `2-SE` floor `88` rows).
     - On `T16_rel`, single-vector `top-1` rises from Stage 2's **`371 / 14,287`** (`2.60%`) to **`1,557`** (`Level 1`), **`1,624` (Seed 1) / `1,610` (Seed 2)** (`Level 2`), and **`2,227` (Seed 1) / `2,226` (Seed 2) `/ 14,287` (`15.59%`, `top-8 = 2,497` / `2,496`)** on `ft/dev` (**`+1,856` rows / `6.00x` Stage 2's `371`**, seed-to-seed noise `1` row, `2-SE` floor `73` rows), and from `439` to **`2,427 / 14,287` (`16.99%`, `top-8 = 2,693`)** on `fresh_v2` (**`+1,988` rows**, seed-to-seed noise `0` rows).
  2. **`beats_stage2_t_restr_on_dev_and_fresh_both_seeds = true`**:
     - `Level 2` (`hamilton_read_recurrence`) beats `Level 1` (`hamilton_read_only`) and the null-recurrence ablation `C_rec_null` across all 4 arms on both `ft/dev` and `fresh_v2`: `2,054 -> 2,087` (`+33` vs floor `14`) on `T16_fz` DEV, `2,187 -> 2,207 / 2,225` (`+20 / +38` vs floor `11 / 15`) on `T16_fz` `fresh_v2`, `1,557 -> 1,624 / 1,610` (`+67 / +53` vs floor `17`, clearing `T_restr(A_W) = 1,602` on both seeds!) on `T16_rel` DEV, and `1,559 -> 1,594 / 1,602` (`+35 / +43` vs floor `15 / 16`) on `T16_rel` `fresh_v2`.
     - `Level 3` (`hamilton_full_trunk`) beats Stage 2's checkpoint-restricted read-code table `T_restr(A_W)` (`2,140` on `T16_fz`, `1,602` on `T16_rel`) by **`+848 / +849` rows** on `T16_fz` DEV (`2,988 / 2,989` vs `2,140`), **`+1,023` rows** on `T16_fz` `fresh_v2` (`3,163`), **`+625 / +624` rows** on `T16_rel` DEV (`2,227 / 2,226` vs `1,602`), and **`+825` rows** on `T16_rel` `fresh_v2` (`2,427`).
  3. **Why `grade.json` records `verdict = "STAGE3_DIAGNOSTIC_HOLD"` (`beats_stage2_g_restr_on_dev_and_fresh_both_seeds = false`)**:
     - Under the strictest diagnostic tier in `grade_trunk_arms`, we also compared the single-vector `OuterL1V1` trunk output on `ft/dev` against Stage 2's raw checkpoint-restricted successor count table `G_restr(A_W)` (`3,084` on `T16_fz`, `2,364` on `T16_rel`). While `Level 3` cleared `3,084` and `2,364` on **`fresh_v2`** (`3,163 > 3,084` on `T16_fz`; `2,427 > 2,364` on `T16_rel`), on **`ft/dev`** `Level 3` scored `2,988 / 2,989` on `T16_fz` (`96` rows below `3,084`) and `2,227 / 2,226` on `T16_rel` (`137` rows below `2,364`), for two exact structural reasons:
       - **(a) Single-vector `OuterL1V1` codebook collisions + tie-breaking (`86–96` rows)**: When Stage T (`#2836`) applied `399` (`T16_fz`) and `545` (`T16_rel`) single-block root moves to `table[v]`, some token pairs `y_1 < y_2` in the same cell became identical 32-int vectors (`table[y_1] == table[y_2]`). Because `OuterL1V1` breaks `0 == 0` distance ties by `token_id asc` (`y_1 < y_2`), no single 32-int vector $q^{\text{trunk}}_L(x) \in \mathbb{Z}^{32}$ can rank `y_2` ahead of `y_1` under that collided 4-block (`B = 4`) codebook (`2,140 -> 2,054` on `Level 1`, `3,084 -> 2,988` on `Level 3`).
       - **(b) Marginal-first weighting in `Level 3`**: In `select_level3_full_trunk`, weighting the marginal query count $64 \cdot C_{\text{query}}(x_{L-1}, y)$ above the finer-grained joint `(query, read)` (`32x`) and `(query, rec)` (`16x`) counts meant $C_{\text{query}}$ acted as the primary lexicographic key whenever marginal counts differed, so `Level 3` vs `Level 3 [C_rec_null]` had `b = 0..2` discordance (`recurrence_lift_positive_on_dev_and_fresh_both_seeds = false` at `Level 3`, whereas at `Level 2` recurrence lift is strictly positive across all 4 arms and both panels).

---

## 2. Code, Config, and Artifact SHA-256 Pins

| Artifact / Pin | Path | SHA-256 |
|---|---|---|
| Executed commit (`trunk-run`) | `crates/uor-r4-discrete` (`2c26d620d`) | `2c26d620de9bebe54de6d5e7cc45666bd439d504` |
| Compiled CUDA `kernel_sha256` | `discrete.ptx` (`sm_120`, CUDA 12.8) | `0f9ccfa7b4748292f83a40227e30ea7b69c893a1649fa8afa3f7d547802429d6` |
| Compiled `geometry_sha256` | `uor-r4-core` geometry tables | `46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327` |
| Unchanged `driver_source_sha256` | `crates/uor-r4-discrete/src/training.rs` | `04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d` |
| Pod binary Build A (`trunk-run`) | `/workspace/trunk-out/bin/trunk-run` | `cbbed056517161f36018d8fc21158692ea833e1e4e346e7d5621fd7a25934a0e` |
| Pod binary Build B (`trunk-run-b`) | `/workspace/trunk-out/bin/trunk-run-b` | `4df030799119014a7ef2877fc5b799d097facfec887e6a38a2933c1a81654d0c` |
| Pre-registration sheet | [`docs/harness/prep/m1-trunk-assembly-and-training-2026-10-25.md`](../../harness/prep/m1-trunk-assembly-and-training-2026-10-25.md) | `bbb641f9115634b03d319b8937d534ed987ebfa18f3d3e73df8ff45d9f554ce0` (pre-stamp `b49051c0f5ef3294089b21dd405d247341c1cf28bd179b6807870eb9373afb07`) |
| Step 0 GPU controls receipt | [`step0-receipt.json`](step0-receipt.json) | `852ec34e9247c543418c387d72b02527d5f5c65f29cfe0f6db0a68ffdd69a85e` |
| Sealed `fresh_v2` slice (`14,287` targets) | `tokens` / `mask` (in-memory cut past `1,049,856`) | `1f8bf88d8deee51574c68ffd8976ce4415fc8aed5602213d6905a37a138517ea` / `fe87363920e19bf072e9fe3a7289e6cd2f1abde6c2431caae8f5c8494d3829d5` |
| `T16_fz` Seed 1 checkpoint (`epoch = 420`) | `t16-frozen-zero-seed1/final-checkpoint.json` | `0831900fc6c04b4489215dc93876f87a173b2c6c16b6635dde2f2ff24c1015a4` |
| `T16_fz` Seed 2 checkpoint (`epoch = 420`) | `t16-frozen-zero-seed2/final-checkpoint.json` | `8bbb59503a16b8bbd2cc723cbd3bc405543f65dfab00ee9ed6c4e9b43271842c` |
| `T16_rel` Seed 1 checkpoint (`epoch = 678`) | `t16-relation-seed1/final-checkpoint.json` | `558b8810dc888d3acd2d5d135b9137062ce69f74e4148a822e1b59b9cfccb944` |
| `T16_rel` Seed 2 checkpoint (`epoch = 678`) | `t16-relation-seed2/final-checkpoint.json` | `1757a3c54692061c112850a8aabfff41671426d48eb3a37fd6a46883242d29d2` |
| `Trunk-FZ-S1` report & manifest | [`t16-frozen-zero-seed1/trunk-report.json`](t16-frozen-zero-seed1/trunk-report.json) / [`manifest`](t16-frozen-zero-seed1/trunk-artifact-manifest.json) | `ed925c8f92366a4e12922e273393556ed610dade62e88e975984a15cd2025379` / `aa79fee39e8f7ced9f633122e55a8f6da0e244736425acb9a7d63ab3d8202c55` |
| `Trunk-FZ-S1` `D11` binary (`542,260 B`) | `t16-frozen-zero-seed1/trunk-model.d11` (in HF archive) | `cd88b6b7b950e7f4672ca645adfeccf37c5560c45831377b54a3c772ce28defd` |
| `Trunk-FZ-S2` report & manifest | [`t16-frozen-zero-seed2/trunk-report.json`](t16-frozen-zero-seed2/trunk-report.json) / [`manifest`](t16-frozen-zero-seed2/trunk-artifact-manifest.json) | `d0ce71fd355637fe55ba21d4eef5dd65c21fe7dc25dd3c833a14124820594801` / `8f06b49ee109a8c4ace89917104179bdc7a8b2be39e5accfdef1a459c0a6029e` |
| `Trunk-FZ-S2` `D11` binary (`542,260 B`) | `t16-frozen-zero-seed2/trunk-model.d11` (in HF archive) | `dda5004775f9105f19dc8cc51446f90199575a4acc702d9d454962787280faf2` |
| `Trunk-Rel-S1` report & manifest | [`t16-relation-seed1/trunk-report.json`](t16-relation-seed1/trunk-report.json) / [`manifest`](t16-relation-seed1/trunk-artifact-manifest.json) | `cdb4de2124a78a1e029f98abd2cddb19d19d569806a6afc4d169b55868a609c9` / `151d2fa46d5fa4cc18b2b470de817348d6c92376d5f1ce0d501fbcff2fbcdce4` |
| `Trunk-Rel-S1` `D11` binary (`542,260 B`) | `t16-relation-seed1/trunk-model.d11` (in HF archive) | `06b97329e3bdd1bb0dc1b23c7aeee347373a3ba8c919ec49dd9b20a21729beab` |
| `Trunk-Rel-S2` report & manifest | [`t16-relation-seed2/trunk-report.json`](t16-relation-seed2/trunk-report.json) / [`manifest`](t16-relation-seed2/trunk-artifact-manifest.json) | `2ddefdcabb6c8ea603b68ed7abf8c4397d4c385ce23ea50fdcb451f0d89db3a9` / `6da9e035bd8695bc13592bee0f27a270283f19c4732a4a7a01160221ff71315e` |
| `Trunk-Rel-S2` `D11` binary (`542,260 B`) | `t16-relation-seed2/trunk-model.d11` (in HF archive) | `b2af6a1e3afcc62f72e3bcd5057eb6f56722407e5c8c3d3ae075f06836f495eb` |
| Stage 3 grade receipt | [`grade.json`](grade.json) | `8413be77ae4253318536a5df26e054032b768086544f2b7ed5f91dfb92f1cce9` |
| HF store archive (`trunk-20261025.tar.gz`) | `caseyallard/uor-r4-store:results/antigravity/m1-trunk-assembly-and-training-2026-10-25/trunk-20261025.tar.gz` | `4f4d4817e2750565f65bdb8a6c6f8ed07b44389f0757702cf6eba11b64e23d35` |

---

## 3. Step 0 GPU & Algebraic Instrument Controls ([`step0-receipt.json`](step0-receipt.json) — `5 / 5 PASS`)

Before any trunk evaluation or training statistic was read, `trunk-run step0` verified all five registered controls on `vast-57764453` (`33,153 ms` on Build A, `33,011 ms` on Build B, `0` mismatches):

| Control | Scope & Invariant Checked | Result | Verdict |
|---|---|---|---|
| **`G-T1`** | Transitive `Cl(0,6)` Hamilton gate (`geometry::BFS[from * 16 + to]`) maps `from_root` to `to_root` across all $16 \times 16 = 256$ root pairs and preserves `BlockNormInvariants::is_canonical_unit_root()` | `256` pairs checked, `0` mismatches | **PASS** |
| **`G-T2`** | `DiscreteQuaternionRecurrenceV1` causal prefix holonomy matches `position_mechanism::relative_holonomy` bitwise across `256` DEV windows (`1,024` blocks) and every output block satisfies `BlockNormInvariants::is_canonical_unit_root()` | `256` windows (`1,024` blocks) checked on `step0` + all `4` arms, `0` mismatches | **PASS** |
| **`G-T3`** | Cross-build `q4_import::load_for_eval` + `Gate R` on `ft/dev` (`14,287` rows) reproduces `#2836` `T16_fz` (`455` top-1, `1,484` top-8, `5,009` admitted, `1,216` `P_succ` admitted) and `T16_rel` (`371` top-1, `1,124` top-8, `3,827` admitted, `1,144` `P_succ` admitted) bitwise | `14,287` rows checked per arm, `0` mismatches | **PASS** |
| **`G-T4`** | Single-vector `OuterL1V1` writer (`writer_score_admitted_fast` and full two-stage `writer_score_full_outer_l1`) matches GPU `discrete_evaluate` `top8` on all `14,287` rows | `14,287` rows checked on both `ft/dev` and `fresh_v2` across all 4 arms, `0` mismatches | **PASS** |
| **`G-T5`** | Sealed `fresh_v2` slice (`cut_fresh_slice(train, 192, 1_049_856, 14_287)`) yields `14,287` eligible targets strictly past the visited `1,049,856` TRAIN prefix (`0` overlap) and `D11` (`uor-r4.lut-discrete/1`) round-trip verification passes (`export_d11` $\to$ `import_d11`) | `14,287` `fresh_v2` targets, `0` overlap, `round_trip_verified = true` on all 4 arms | **PASS** |

- **Build A vs Build B Determinism Replication**: Running `trunk-run step0` and `trunk-run arm --name t16-frozen-zero-seed1` on two independent `nvcc 12.8` builds (`trunk-run` `cbbed056...` and `trunk-run-b` `4df03079...`) produced **100% identical evaluation counts, discordance pairs, and trunk table manifests** across all `14,287` `ft/dev` rows and all `14,287` `fresh_v2` rows (`0` metric differences).

---

## 4. Two-Seed Stage 3 Trunk Measurements on `ft/dev` (`14,287` Rows) and `fresh_v2` (`14,287` Rows)

Every readout level emits a **single 32-int trunk vector** $q^{\text{trunk}}_L(x) \in \{-2, 0, 2\}^{32}$ per row, verifies canonical $E_8$ orbit norm invariants across all `514,332` evaluated blocks per panel (`14,287` rows $\times$ `9` evaluations $\times$ `4` blocks), and scores $q^{\text{trunk}}_L(x)$ through both `writer_score_admitted_fast` and full two-stage `writer_score_full_outer_l1` (which agree on `100%` of rows at `Level 1`, `Level 2`, and `Level 3`):

### 4.1 `ft/dev` Panel (`14,287` Rows, `3,847` `P_succ` Rows)

| Metric / Readout Level | `Trunk-FZ-S1` (`Seed 1`) | `Trunk-FZ-S2` (`Seed 2`) | `Trunk-Rel-S1` (`Seed 1`) | `Trunk-Rel-S2` (`Seed 2`) |
|---|---:|---:|---:|---:|
| **TRAIN context throughput (`tok/s`)** | **`76,316`** (`1,230.1 s`) | **`76,343`** (`1,229.6 s`) | **`16,609`** (`5,651.8 s`) | **`16,608`** (`5,652.1 s`) |
| **`ft/dev` context throughput (`tok/s`)** | **`34,325`** (`32.1 s`) | **`34,337`** (`32.1 s`) | **`8,530`** (`129.2 s`) | **`8,542`** (`129.0 s`) |
| **Target cell admitted (`All / P_succ`)** | `5,009 / 1,216` | `5,009 / 1,216` | `3,827 / 1,144` | `3,827 / 1,144` |
| **Distinct TRAIN `(c_read / c_rec / pairs)`** | `19,385 / 55,440 / 796,076` | `19,385 / 54,563 / 795,317` | `27,132 / 54,221 / 798,485` | `27,132 / 53,256 / 784,400` |
| **Distinct `ft/dev` `(c_read / c_rec / shuf)`** | `4,067 / 8,812 / 9,547` | `4,067 / 8,590 / 9,540` | `4,429 / 8,530 / 9,088` | `4,429 / 8,294 / 8,980` |
| **`Level 0` (`writer_raw` / `C_gate_id`) `top-1` (`top-8`)** | `455` (`1,484`) | `455` (`1,484`) | `371` (`1,124`) | `371` (`1,124`) |
| **`Level 0` `P_succ` `top-1` (`top-8`)** | `107` (`318`) | `107` (`318`) | `78` (`243`) | `78` (`243`) |
| **`Level 1_global` (no shortlist) `top-1` (`top-8`)** | **`1,598`** (`1,995`) | **`1,598`** (`1,995`) | **`1,143`** (`1,480`) | **`1,143`** (`1,480`) |
| **`Level 1` (`hamilton_read_only`) `top-1` (`top-8`)** | **`2,054`** (`2,493`) | **`2,054`** (`2,493`) | **`1,557`** (`1,912`) | **`1,557`** (`1,912`) |
| **`Level 1` full two-stage `OuterL1V1` `top-1`** | `2,054` | `2,054` | `1,557` | `1,557` |
| **`Level 1` `P_succ` `top-1`** | `373` | `373` | `309` | `309` |
| **`Level 2` (`hamilton_read_recurrence`) `top-1` (`top-8`)** | **`2,087`** (`2,531`) | **`2,087`** (`2,532`) | **`1,624`** (`1,974`) | **`1,610`** (`1,958`) |
| **`Level 2` full two-stage `OuterL1V1` `top-1`** | `2,087` | `2,087` | `1,624` | `1,610` |
| **`Level 2` `P_succ` `top-1`** | `381` | `378` | `318` | `314` |
| **`Level 2` Ablation `C_rec_null` (`c_rec = 0`) `top-1`** | `2,054` (`-33`) | `2,054` (`-33`) | `1,557` (`-67`) | `1,557` (`-53`) |
| **`Level 2` Ablation `C_rec_shuf` (shuffled order) `top-1`** | `2,063` (`-24`) | `2,061` (`-26`) | `1,573` (`-51`) | `1,571` (`-39`) |
| **`Level 2` Ablation `C_norm_unit` (unscaled `snap_1`) `top-1`** | `2,077` (`-10`) | `2,082` (`-5`) | `1,593` (`-31`) | `1,613` (`+3`) |
| **`Level 3` (`hamilton_full_trunk`) `top-1` (`top-8`)** | **`2,988` (`20.91%`, `3,247`)** | **`2,989` (`20.92%`, `3,248`)** | **`2,227` (`15.59%`, `2,497`)** | **`2,226` (`15.58%`, `2,496`)** |
| **`Level 3` full two-stage `OuterL1V1` `top-1`** | `2,988` | `2,989` | `2,227` | `2,226` |
| **`Level 3` `P_succ` `top-1`** | **`511`** | **`511`** | **`466`** | **`465`** |
| **Discordance `Level 1` vs `Level 0` (`b / c / 2-SE`)** | `1,679 / 80 / 84` (`+1,599`) | `1,679 / 80 / 84` (`+1,599`) | `1,246 / 60 / 73` (`+1,186`) | `1,246 / 60 / 73` (`+1,186`) |
| **Discordance `Level 2` vs `Level 1` (`b / c / 2-SE`)** | `41 / 8 / 14` (`+33`) | `39 / 6 / 14` (`+33`) | `68 / 1 / 17` (`+67`) | `59 / 6 / 17` (`+53`) |
| **Discordance `Level 3` vs `Level 1` (`b / c / 2-SE`)** | `1,043 / 109 / 68` (`+934`) | `1,043 / 108 / 68` (`+935`) | `817 / 147 / 63` (`+670`) | `816 / 147 / 63` (`+669`) |

### 4.2 Sealed Fresh Transfer Panel `fresh_v2` (`14,287` Rows, `3,932` `P_succ` Rows)

| Metric / Readout Level | `Trunk-FZ-S1` (`Seed 1`) | `Trunk-FZ-S2` (`Seed 2`) | `Trunk-Rel-S1` (`Seed 1`) | `Trunk-Rel-S2` (`Seed 2`) |
|---|---:|---:|---:|---:|
| **`fresh_v2` context throughput (`tok/s`)** | **`37,838`** (`34.7 s`) | **`37,853`** (`34.7 s`) | **`8,326`** (`157.8 s`) | **`8,336`** (`157.6 s`) |
| **Target cell admitted (`All / P_succ`)** | `5,050 / 1,386` | `5,050 / 1,386` | `3,894 / 1,316` | `3,894 / 1,316` |
| **Distinct `fresh_v2` `(c_read / c_rec / shuf)`** | `4,217 / 9,097 / 9,601` | `4,217 / 8,887 / 9,517` | `4,397 / 8,732 / 9,117` | `4,397 / 8,541 / 9,060` |
| **`Level 0` (`writer_raw` / `C_gate_id`) `top-1` (`top-8`)** | `457` (`1,471`) | `457` (`1,471`) | `439` (`1,103`) | `439` (`1,103`) |
| **`Level 0` `P_succ` `top-1` (`top-8`)** | `167` (`446`) | `167` (`446`) | `148` (`345`) | `148` (`345`) |
| **`Level 1_global` (no shortlist) `top-1` (`top-8`)** | **`1,729`** (`2,121`) | **`1,729`** (`2,121`) | **`1,159`** (`1,480`) | **`1,159`** (`1,480`) |
| **`Level 1` (`hamilton_read_only`) `top-1` (`top-8`)** | **`2,187`** (`2,616`) | **`2,187`** (`2,616`) | **`1,559`** (`1,881`) | **`1,559`** (`1,881`) |
| **`Level 1` full two-stage `OuterL1V1` `top-1`** | `2,187` | `2,187` | `1,559` | `1,559` |
| **`Level 1` `P_succ` `top-1`** | `570` | `570` | `406` | `406` |
| **`Level 2` (`hamilton_read_recurrence`) `top-1` (`top-8`)** | **`2,207`** (`2,635`) | **`2,225`** (`2,648`) | **`1,594`** (`1,909`) | **`1,602`** (`1,918`) |
| **`Level 2` full two-stage `OuterL1V1` `top-1`** | `2,207` | `2,225` | `1,594` | `1,602` |
| **`Level 2` `P_succ` `top-1`** | `575` | `576` | `418` | `417` |
| **`Level 2` Ablation `C_rec_null` (`c_rec = 0`) `top-1`** | `2,187` (`-20`) | `2,187` (`-38`) | `1,559` (`-35`) | `1,559` (`-43`) |
| **`Level 2` Ablation `C_rec_shuf` (shuffled order) `top-1`** | `2,203` (`-4`) | `2,201` (`-24`) | `1,568` (`-26`) | `1,574` (`-28`) |
| **`Level 2` Ablation `C_norm_unit` (unscaled `snap_1`) `top-1`** | `2,207` (`0`) | `2,211` (`-14`) | `1,574` (`-20`) | `1,588` (`-14`) |
| **`Level 3` (`hamilton_full_trunk`) `top-1` (`top-8`)** | **`3,163` (`22.14%`, `3,415`)** | **`3,163` (`22.14%`, `3,416`)** | **`2,427` (`16.99%`, `2,693`)** | **`2,427` (`16.99%`, `2,693`)** |
| **`Level 3` full two-stage `OuterL1V1` `top-1`** | `3,163` | `3,163` | `2,427` | `2,427` |
| **`Level 3` `P_succ` `top-1`** | **`717`** | **`717`** | **`624`** | **`624`** |
| **Discordance `Level 1` vs `Level 0` (`b / c / 2-SE`)** | `1,817 / 87 / 88` (`+1,730`) | `1,817 / 87 / 88` (`+1,730`) | `1,189 / 69 / 71` (`+1,120`) | `1,189 / 69 / 71` (`+1,120`) |
| **Discordance `Level 2` vs `Level 1` (`b / c / 2-SE`)** | `25 / 5 / 11` (`+20`) | `46 / 8 / 15` (`+38`) | `44 / 9 / 15` (`+35`) | `51 / 8 / 16` (`+43`) |
| **Discordance `Level 3` vs `Level 1` (`b / c / 2-SE`)** | `1,038 / 62 / 67` (`+976`) | `1,038 / 62 / 67` (`+976`) | `956 / 88 / 65` (`+868`) | `956 / 88 / 65` (`+868`) |

---

## 5. Diagnosis & Unlocked Next Work

1. **The single-vector geometric trunk (`DiscreteHamiltonGateV1` + `DiscreteQuaternionRecurrenceV1` + `OrbitRootSnapNormV1`) recovers `96.0%–96.9%` of the Stage 2 within-shortlist capacity through the unmodified `OuterL1V1` writer and transfers cleanly to `fresh_v2`**:
   - In Stage 2 (`#2836`), the raw `OuterL1V1` writer scored `455 / 14,287` (`T16_fz`) and `371 / 14,287` (`T16_rel`), losing over `85%` of the checkpoint's admitted-shortlist capacity (`T_restr(A_W) = 2,140`, `G_restr(A_W) = 3,084` on `T16_fz`).
   - By rotating the GPU read vector $q^{\text{read}}(x)$ through the transitive `Cl(0,6)` Hamilton gate `geometry::BFS[r_b * 16 + t_b]` into a single canonical orbit vector $q^{\text{trunk}}_L(x) \in \{-2, 0, 2\}^{32}$, `Level 1` (`2,054` DEV / `2,187` fresh), `Level 2` (`2,087` DEV / `2,207–2,225` fresh), and `Level 3` (`2,988–2,989` DEV / `3,163` fresh) lift single-vector `OuterL1V1` `top-1` by **`+1,599` to `+2,706` rows (`4.5x–6.9x`)** across both seeds, with `100%` agreement between restricted-shortlist scoring (`writer_score_admitted_fast`) and full two-stage `OuterL1V1` evaluation (`writer_score_full_outer_l1`).
2. **Causal Clifford holonomy recurrence (`DiscreteQuaternionRecurrenceV1`) and multiplicity-matched normalization (`OrbitRootSnapNormV1`) provide verified order-sensitive lift at `Level 2`**:
   - Across all 4 arms and both panels (`ft/dev` and `fresh_v2`), adding `c_rec(x)` in `Level 2` improves over `Level 1` (`C_rec_null`) by `+33` rows (`T16_fz` DEV, vs `2-SE` floor `14`), `+20 / +38` rows (`T16_fz` `fresh_v2`, vs floor `11 / 15`), `+67 / +53` rows (`T16_rel` DEV, vs floor `17`, beating `T_restr(A_W) = 1,602` on both seeds), and `+35 / +43` rows (`T16_rel` `fresh_v2`, vs floor `15 / 16`).
   - Shuffling prefix token order (`C_rec_shuf`) destroys `72%–84%` of that recurrence gain (`2,087 -> 2,063 / 2,061` on `T16_fz` DEV, `1,624 / 1,610 -> 1,573 / 1,571` on `T16_rel` DEV), and replacing multiplicity-matched `snap_4` with unscaled `snap_1` (`C_norm_unit`) degrades `7` of `8` arm-panel pairs.
3. **Why `grade.json` records `STAGE3_DIAGNOSTIC_HOLD` and what this unlocks**:
   - The `86–96`-row gap between single-vector `OuterL1V1` (`2,054` at `Level 1`, `2,988` at `Level 3` on `ft/dev`) and the raw count tables (`2,140` and `3,084`) is caused by 32-int vector collisions (`table[y_1] == table[y_2]`, `y_1 < y_2`) introduced during Stage T's `B = 4` single-block root moves (where `OuterL1V1` breaks `0 == 0` distance ties by `token_id asc`), while the `5,009 / 14,287` (`35.06%`) outer-L1 admission ceiling still caps `Level 3` (`2,989` DEV / `3,163` fresh) below the M1 full-panel `T2 = 6,300 / 14,287` bar (`CHARTER.md` §4.8).
   - Closing `#2844` (`status: done`) pins the Stage 3 trunk checkpoints, `trunk-report.json` receipts, and `D11` (`trunk-model.d11`) artifacts, immediately unblocking:
     - **`#2833` (`m1-address-as-read`)**: Can now run its three-arm serving-time address-oracle GPU evaluation against the pinned `#2844` trunk checkpoint and `D11` SHA;
     - **`#2851` (`m2-pin-train-mask-sheet` discrete branch)**: Can now pin the merged `#2844` discrete trunk trainer (`crates/uor-r4-discrete/src/trunk.rs`) and `fresh_v2` transfer slice;
     - **`#2847` (`m1-wider-code-carrier`, $B = 8$)**: Eliminates 4-block codebook vector collisions and expands shortlist admission toward the M1 `6,300 / 14,287` bar.
