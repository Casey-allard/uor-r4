# Paired `Route` and `Flock` read-on vs read-off checkpoint comparison (`r-read-off-vs-read-on`) — DRAFT

- **Status:** `DRAFT`, pre-registration (`zero training updates`). No compute is cleared.
- **Roadmap item:** [`docs/roadmap/items/r-read-off-vs-read-on.md`](../../roadmap/items/r-read-off-vs-read-on.md) ([`#2885`](https://github.com/UOR-Foundation/uor-r4/issues/2885), Milestone `#2029` / `research`)
- **Design note, checkpoint inventory & finite controls:** [`docs/labs/r-read-off-vs-read-on/README.md`](../../labs/r-read-off-vs-read-on/README.md) (`7 / 7` host controls pass in [`crates/uor-r4-discrete/tests/read_off_ablation_controls.rs`](../../../crates/uor-r4-discrete/tests/read_off_ablation_controls.rs))
- **Staged battery & configs:** [`docs/labs/r-read-off-vs-read-on/evidence/read-off-battery.sh`](../../labs/r-read-off-vs-read-on/evidence/read-off-battery.sh) (`{a8,p8}-{r-on,r-off,r-perm,f-on,f-off}.json`)
- **Template & parent sheet:** [`docs/harness/prep/m1-attention-address-structured-cells-DRAFT-2026-10-19.md`](m1-attention-address-structured-cells-DRAFT-2026-10-19.md) (executed in [`docs/labs/address-cells-preflight-2026-10-19/README.md`](../../labs/address-cells-preflight-2026-10-19/README.md))
- **Rules:**
  - CHARTER §3.1–§3.3 (D23, D25, D30, D32): geometry only; zero softmax, squashing, dense matmul, or floats anywhere.
  - CHARTER §3.4–§3.5 (D24): GPU-only model evaluation and pre-built `nvcc` CUDA kernels; host tests verify finite synthetic controls only.
  - CHARTER §4.1–§4.5 (D22, D31): one-page pre-registration and non-author `TEST FITNESS: FIT` before any GPU run; a negative closes a configuration only.

---

## Arms

### Phase 1 — Zero-update retained checkpoint-0 battery (`accepted_updates == 0`, accepted by `eval-corpus` today)

Evaluated on both the **`A8` predecessor-structured injective codebook** (`address_structured_v1`, file SHA-256 `6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`, table SHA-256 `4122cbab8b25119b120e417003152a5c3069f06e679691f7f67662a648604f07`) and the **`P8` PrimeZeta injective codebook** (file SHA-256 `83c1380a8f507e51d0342faa1b089dea6a20561a03552208c093fa640afc124b`, table SHA-256 `e9e03bcd7c2341b1793e5a7c0b748ef53075001fb9c8441465193ab18e238cf8`) at writer $K = 8$, seed `1`, `routes_sha256 = 71cdce57…`:

| Arm ID | `ReadSpec` | Read Mode / Initialization | Role |
|---|---|---|---|
| **`R-On`** (`A8` & `P8`) | `Route` | `s[13] == 0` (ternary-tree port walk + `BFS` transport, `routes = 71cdce57…`) | **Primary `Route` read-on arm.** |
| **`R-Off`** (`A8` & `P8`) | `Route` | `RouteBypassV1` (`s[13] == 1`: skips tree walk and writes last-position query block `input[length - 1]` to `output`) | **`Route` read-off ablation arm.** |
| **`R-Perm`** (`A8` & `P8`) | `Route` | `RoutePermutedActionsV1` (`s[13] == 0`, route actions mapped by the fixed-point-free signed-axis group derangement $a \mapsto (5a + 1) \bmod 64$) | **Geometric shuffled-read control:** tests whether `R-On`'s effect depends on the specific action alignment vs any group action in $G$. |
| **`F-On`** (`A8` & `P8`) | `Relation { kept: 8, sweeps: 1 }` (edge trits `0` = `Flock { kept: 8 }`) | `ActiveLeafV1` (`gates = ceb6ae99…`, level-0 combine gates $g_{b,0} = +1$) | **Primary `Flock` read-on arm** (must reproduce `4,244 / 1,351 / 133` on `A8` cp0 and `427 / 127 / 38` on `P8` cp0 word-for-word). |
| **`F-Off`** (`A8` & `P8`) | `Relation { kept: 8, sweeps: 1 }` / `Flock { kept: 8 }` | `ZeroGatesV1` (all combine and edge gates `0`: `terms = 1`, `snap(u, 1, g) = root_id(u)`) | **`Flock` read-off ablation arm** (must agree with `R-Off` on `14,287 / 14,287` DEV rows). |

### Phase 2 — Retained post-update checkpoint battery (`accepted_updates > 0`, conditional on PM/owner Option B on `#2885`)

If the PM/owner selects **Option B** on [`#2885`](https://github.com/UOR-Foundation/uor-r4/issues/2885#issuecomment-6373003827) (authorizing a read-only checkpoint evaluation entry point so `eval-corpus`'s `accepted_updates == 0` training-hold guard at `main.rs:108-110` is preserved), the same paired `Read-On` vs `Read-Off` vs `Read-Permuted` comparison is executed on:
1. **`K16-A` seed 1 & seed 2 at batch `16,404`** (`codex-cf1-k16-four-role-data1-training-root-part-000-2324z`, payload SHA-256 `1ca211b4975c61e8aaaf21a2b01911d7842f213ae4f1cd0ba1997e41f6fdfe77`, `ReadSpec::Route`, $K = 16$, `accepted_updates = 407 / 410`, sealed `evaluation-16404.json` SHA-256s `6936cf99…` and `d7976a0b…`, `top1 = 65` and `463`, `admitted = 2,237` and `3,167`).
2. **`A8` Dose (`vast-56670997`) `FrozenZero` Q4 (`batch 16,404`, `accepted_updates = 809`) and `Relation` Q1 (`batch 4,096`, `accepted_updates = 396`)** (`ReadSpec::Relation { kept: 8, sweeps: 1 }`, $K = 8$, comparing `ActiveLeaf` / learned gates vs `ZeroGatesV1` read-off on the same retained checkpoint table).

---

## Bar

**Status: PROPOSED, NOT REGISTERED** (awaits PM/owner registration on `#2885` and non-author `TEST FITNESS: FIT` before any GPU pod is leased).

1. **D31 Instrument Gates (must hold before any mechanism call):**
   - **G-CrossMode (`R-Off == F-Off` identity gate):** On every shared checkpoint table (`A8` cp0 and `P8` cp0), `R-Off` (`RouteBypassV1`) and `F-Off` (`ZeroGatesV1`) produce **zero row discrepancies (`0 / 14,287`)** across `top1_correct`, `top8_correct`, `target_admitted_rows`, `admitted_cell_count_sum`, `token_rival_count_sum`, and `address_rival_count_sum`.
   - **G-Repro (Read-On exact reproduction gate):** `F-On` reproduces `4,244 / 14,287` full-DEV admitted rows, `1,351 / 3,847` `P_succ` admitted rows, `top1_correct = 133`, and `P_succ top1 = 62` on `A8` cp0, and `427 / 14,287` full-DEV admitted rows, `127 / 3,847` `P_succ` admitted rows, and `top1_correct = 38` on `P8` cp0 word-for-word (and under Option B, `K16-A s1/s2` `R-On` reproduces `2,237 / 65 / 547` and `3,167 / 463 / 1,295`).
   - **G-Qual (Finite CUDA controls):** All 6 finite controls (`C1`–`C6`) pass on the device (`0` failures, `0` applied updates).

2. **Total 3-Branch Mechanism Decision Rule (per read and checkpoint, on both the `14,287` full DEV panel and the `3,847` `P_succ` subset):**
   Let $b$ be the count of scored rows where `Read-On` succeeds (`top1_correct`, `top8_correct`, or target admitted) and `Read-Off` fails, and let $c$ be the count of rows where `Read-Off` succeeds and `Read-On` fails. Define the paired McNemar discordance floor $F = \max(24, 2\sqrt{b + c})$ (`>= 2 SE` of $b - c$, matching the parent `m1-attention-address-structured-cells` sheet):
   - **Branch 1 — `READ_ADDS_SIGNAL` (positive):** $b - c > F$ against `Read-Off` **and** $b_{\text{perm}} - c_{\text{perm}} > \max(24, 2\sqrt{b_{\text{perm}} + c_{\text{perm}}})$ against `R-Perm` (`RoutePermutedActionsV1`), without triggering `G-Occ`. The read contributes positive context selection beyond the last-position token carrier.
   - **Branch 2 — `READ_INERT_AT_CHECKPOINT` (null configuration verdict):** $|b - c| \le F$ (or $b - c > F$ against `Read-Off` but $|b_{\text{perm}} - c_{\text{perm}}| \le F_{\text{perm}}$). The read adds no statistically distinguishable signal above the unrouted last-position token carrier at that checkpoint; closes that checkpoint/schedule configuration under D22 and prioritizes codebook/admission/writer or an interleaved read schedule (`InterleavedReadEvery8V1`).
   - **Branch 3 — `READ_HURTS_ADMISSION_OR_RANKING` (destructive interference configuration verdict):** $c - b > F$ (`Read-Off` outperforms `Read-On` by more than $F$). Uncalibrated tree transport or neighbour consensus shifts the query vector away from its predecessor-placed target cell at that checkpoint.

---

## Base artifact

- **Phase 1 (`accepted_updates == 0`):** Checkpoint-0 models initialized deterministically (`seed = 1`, `epoch = 0`) from the pinned `A8` table (`6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`, table `4122cbab8b25119b120e417003152a5c3069f06e679691f7f67662a648604f07`, `routes = 71cdce57…`, `gates = ceb6ae99…`) and the pinned `P8` `PrimeZeta` table (`83c1380a8f507e51d0342faa1b089dea6a20561a03552208c093fa640afc124b`, table `e9e03bcd7c2341b1793e5a7c0b748ef53075001fb9c8441465193ab18e238cf8`, `model_sha256 = 134fd663…`), stored in HF `caseyallard/uor-r4-store:vast-archive/vast-56660582/`.
- **Phase 2 (if PM/owner selects Option B on `#2885`):** Retained `K16-A` seed 1 & seed 2 checkpoints at cursor `16,404` (`codex-cf1-k16-four-role-data1-training-root-part-000-2324z`, inner `payload.tar.gz` SHA-256 `1ca211b4975c61e8aaaf21a2b01911d7842f213ae4f1cd0ba1997e41f6fdfe77`) and retained `A8` Dose checkpoints in HF `caseyallard/uor-r4-store:vast-archive/vast-56670997/`.

---

## Data split

- **DEV** (`ft/dev/tokens.u16` SHA-256 `0919e90f813bf02f6f0528ebcf1f34c97987113b679619b0b44e065fa90b2546`, `response_mask.u8` SHA-256 `a838bb1ae20c05a1b68265a7aa11c3dd146200b7d1ff6ef185ff999e442f5327`, tokenizer `d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`; **`14,287` scored rows** across `400` documents, and the **`3,847`-row `P_succ` copy-successor subset**): scored once per arm on the GPU with zero parameter updates.
- **TRAIN** (`mw-balp/train/tokens.u16` SHA-256 `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`, `response_mask.u8` SHA-256 `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`): used only for the already-pinned `A8` table (`6600bdb1…`) and count-baseline reference (`6,300 / 14,287` top-1); zero training batches are run.
- **Fresh sealed transfer slice** (`0b7e754c…` / `a1cda529…` / seal `df5d074a…`): untouched and unopened.

---

## Steps

1. **Host design note, checkpoint audit & finite controls (completed in `antigravity-45`):**
   - [`docs/labs/r-read-off-vs-read-on/README.md`](../../labs/r-read-off-vs-read-on/README.md) documents the checkpoint inventory, the `eval-corpus` refusal audit (`main.rs:108-110`), and the `RouteBypassV1` / `ZeroGatesV1` design.
   - [`crates/uor-r4-discrete/tests/read_off_ablation_controls.rs`](../../../crates/uor-r4-discrete/tests/read_off_ablation_controls.rs) verifies finite controls `C1`–`C7` on the host via `~/.cargo/bin/cargo test -p uor-r4-discrete --test read_off_ablation_controls`.
2. **Kernel switch, device tests & staged battery script (completed in `antigravity-46`):**
   - `ReadInitialization::RouteBypassV1` (`s[13] == 1`) and `ReadInitialization::RoutePermutedActionsV1` (`g'(a) = (5a + 1) mod 64`) are wired in `crates/uor-r4-discrete/src/{lib.rs,discrete.cu,device.rs,training.rs,attention_preflight.rs,d11_export.rs}` with the `#[ignore = "requires a CUDA GPU"]` device test `route_read_off_and_permuted_controls_run_and_refuse`.
   - [`docs/labs/r-read-off-vs-read-on/evidence/read-off-battery.sh`](../../labs/r-read-off-vs-read-on/evidence/read-off-battery.sh) and the 10 zero-update arm configs (`{a8,p8}-{r-on,r-off,r-perm,f-on,f-off}.json`) are staged under `docs/labs/r-read-off-vs-read-on/evidence/`.
3. **Non-author `TEST FITNESS: FIT` & PM/owner registration on `#2885`:**
   - Obtain a non-author `TEST FITNESS: FIT` review on this sheet, PM/owner decision on Option A vs Option B (Section 1.3 of the design note) and bar registration on `#2885`, and Casey's approval in chat (`CHARTER §5.1`, `#2895`).
4. **GPU evaluation battery on one leased RTX 5090 (`scripts/pod/uor-pod`):**
   - Run `scripts/pod/uor-pod status` first, lease one RTX 5090 pod, and execute [`docs/labs/r-read-off-vs-read-on/evidence/read-off-battery.sh`](../../labs/r-read-off-vs-read-on/evidence/read-off-battery.sh).
   - Verify all input file SHA-256s, `G-Qual` (`route_read_off_and_permuted_controls_run_and_refuse`), `G-Repro` (`F-On` == `4,244 / 1,351 / 133` on `A8` cp0 and `427 / 127 / 38` on `P8` cp0), and `G-CrossMode` (`R-Off == F-Off` on `14,287 / 14,287` rows).
   - Archive all reports and `FILES.sha256` to the HF store before `uor-pod down`, and record one row per paired comparison in `docs/research/notebook/m1-discrete-attention.md`.

---

## Stop

- Any failure of `G-CrossMode` (`R-Off` vs `F-Off` differing on even `1 / 14,287` rows), `G-Repro` (`F-On` failing to reproduce `4,244 / 1,351 / 133` on `A8` cp0 or `427 / 127 / 38` on `P8` cp0), or `G-Qual` stops the run immediately as a D31 instrument fault, never as a mechanism negative.
- Zero training updates (`accepted_updates` remains unchanged; `evaluation_mutates_model == false`).
- Hard wall-clock ceiling: `25 minutes` (`~$0.35` on one RTX 5090) and `30 GiB` free disk floor.

---

## Cost

- **Host controls (`antigravity-45`):** `$0`, no GPU, `< 1 s` test execution.
- **Planned GPU evaluation battery (after `FIT` + owner approval):** 1x RTX 5090 via `scripts/pod/uor-pod` (`uor-pod status` first) for `~15–20 minutes` (`~8 min` compile + qualification, `< 1 min` per arm $\times$ 10 zero-update evaluations) = **`~$0.25–$0.35`** at `~$0.82/h` (well within the `$8/h` and 4-pod fleet cap).

---

## Minimum test power

- Every arm is evaluated deterministically across the full **`14,287`-row** DEV panel (`400` documents) and the **`3,847`-row** `P_succ` copy-successor subset, exceeding the CHARTER §4.3 minimum of `>= 200` rows by `71x` (full DEV) and `19x` (`P_succ`).
- Because forward evaluation on a fixed checkpoint and frozen corpus is 100% deterministic, paired binary contrasts (`Read-On` vs `Read-Off` and `Read-On` vs `Read-Permuted`) are powered by the exact paired McNemar discordance floor $F = \max(24, 2\sqrt{b + c})$ (`>= 2 SE` of the row difference $b - c$, and under Phase 2 Option B across both `K16-A` training seeds `1` and `2` with two-seed noise bands `Δ_noise,top1 = 398`, `Δ_noise,top8 = 748`, `Δ_noise,adm = 930`).

---

## Verdict

TEST FITNESS: PENDING - M1 #2029 r-read-off-vs-read-on
