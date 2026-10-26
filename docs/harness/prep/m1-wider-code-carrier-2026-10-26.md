# Wider code carrier ($B = 6, 8$ blocks): finite CUDA qualification and zero-update GPU pre-flight

- **Date:** 2026-10-26. **Lab:** antigravity (`antigravity-92`, PR `#3277`). **Item:** `m1-wider-code-carrier` (`#2847`), M1 Stage 1 (`#2029`).
- **Source commit (`crates/` frozen):** `99b251ccba823612fe3acc04c083fe584adf9e6a` (`driver_source_sha256`: `04c5165464a18eea6d4edadc6970007dbdc7908ead420f4a2f575e22ea43061d`; re-pinned after pre-treatment instrument stops `fault1`–`fault3` before any wider-carrier treatment arm ran).
- **Rules:**
  - zero training updates (checkpoint 0 evaluation only; training remains HELD);
  - integer arithmetic only, with no softmax or squashing (§3.2) and no matmul (§3.3);
  - GPU for every model quantity and qualification control (§3.5);
  - host counting of stored GPU outputs for grading only (D24);
  - a negative closes a configuration, never a mechanism (D22).

## Question

At $B = 4$ blocks (`width = 32`), `FullVectorV1` places only one interior root pair (`blocks 1..3`) using downward-frequency predecessor pull and fills blocks `3..B-1` at $B > 4$ with an un-pulled XOR pad. When the carrier is widened to $B = 6$ (`width = 48`, $P = 2$ interior pairs) and $B = 8$ (`width = 64`, $P = 3$ interior pairs) under multi-pair residual bidirectional bigram-pulled placement (`AddressCellsMode::WiderBlocksV1`):
1. Does the wider-carrier build pass finite CUDA qualification (`G-Q`), exact D11 integer export round-trip (`G-D11`), zero within-cell alias tie mass (`G-Occ`), and Stage 1 alias ceiling (`G-Cap` > 1,941 net of admission)?
2. Does zero-training `P_succ` admission at checkpoint 0 clear `A8-Outer`'s `1,351 / 3,847` and beat both the matched shuffled wider-carrier control (`WBS6`/`WBS8`) and the $B = 8$ XOR-padded single-pair control (`FV8-B8-Pad`) by the exact integer two-sigma McNemar floor $F = \max(24, \lceil 2\sqrt{b + c}\rceil)$, and how does $B \in \{6, 8\}$ compare directly against $B = 4$ (`FV8-B4-Env = 1,980 / 3,847`, `FV8-B4-TwoScale = 1,802 / 3,847`) on the same 14,287-row DEV panel?

## Base Artifact, Arms, and Steps (checkpoint 0, zero updates, seed 1, `Relation { kept: 8, sweeps: 1 }`, `ActiveLeafV1`, `K = 8`)

| Arm | $B$ (`width`) | Config (`sha256`) | Pinned table (`file_sha256`) | Metric (`s[15]`) | Role |
|---|---:|---|---|---|---|
| `A8-Outer` | 4 (32) | `docs/labs/a8-training-configs-2026-10-19/evidence/a8-relation-seed1.json` (`469a2ff0fe17a6a80f5875a077b3e6a9436553efdf1c6e7884541220553e9d5e`) | `address-structured-v1.json` (`6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`) | `outer_l1_v1` (0) | $B=4$ outer reference & `G-Exact` control |
| `FV8-B4-Env` | 4 (32) | `docs/labs/m1-address-key-full-vector/configs/fv8-env.json` (`56c3fafe0b8cdb7800f57d0a6ba577b2964356de5765b350a701546cace6e534`) | `address-full-vector-v1.json` (`2c886736c858778677d6a7b87253af188d8787cedc5fe5acd9b3c899a10a90e2`) | `cell_envelope_l1_v1` (1) | $B=4$ envelope reference (`1,980 / 3,847`) |
| `FV8-B4-TwoScale` | 4 (32) | `docs/labs/m1-address-key-full-vector/configs/fv8-two-scale.json` (`1cdb80615d1467bbe062bc1a70804630b3f69af5c580ce3b0d2b5d7407a89a28`) | `2c886736…` | `full_vector_cell_l1_v1` (2) | $B=4$ two-scale reference (`1,802 / 3,847`) |
| `WB6-Outer` | 6 (48) | `docs/labs/m1-wider-code-carrier/configs/wb6-outer.json` (`4e1545948a9cb4d3560590f7d5e3db64acf0efbe063eef48dd32d0c978c21c3a`) | `b6-wider-blocks-v1.json` (`fd2c79277ef20c7fa1cac136c5d8f6fcb2586f5563aecfd889afb5c4b6748374`) | `outer_l1_v1` (0) | $B=6$ within-table outer-key & `G-D11` control |
| `WB6-Env` | 6 (48) | `…/configs/wb6-env.json` (`e86d0f157310d928bb9c515291566a563d0acffe7f1e1c6089c63aacf98160b2`) | `fd2c7927…` | `cell_envelope_l1_v1` (1) | $B=6$ envelope treatment |
| `WB6-TwoScale` | 6 (48) | `…/configs/wb6-two-scale.json` (`49a2822bb65e5d895fe8e9820bf3a450a27b552c80e7f9365d5424a6e33593f7`) | `fd2c7927…` | `full_vector_cell_l1_v1` (2) | $B=6$ two-scale treatment |
| `WBS6-Env` | 6 (48) | `…/configs/wbs6-env.json` (`ec86db0fe1c537cdda2401df525cce84c81a6b68d4192f7a3c63851818e1582f`) | `b6-wider-blocks-shuffled-v1-seed-20261019.json` (`67031b7f9da75e6672aced861b299b9f0301ba7894d2571d6b81525bec873df5`) | (1) | `WB6-Env` matched shuffled control |
| `WBS6-TwoScale` | 6 (48) | `…/configs/wbs6-two-scale.json` (`c3499a619b21a9264792d8f2aed9177de302b7bf5d8d5e02a48e0d5263ab0d2f`) | `67031b7f…` | (2) | `WB6-TwoScale` matched shuffled control |
| `WB8-Outer` | 8 (64) | `…/configs/wb8-outer.json` (`6a72404fef373dc2f3c59d3141819ccbdd168fa5004bbef2b4803221a3f19af0`) | `b8-wider-blocks-v1.json` (`d5c3cb97f02210942778fe26fd874aaabf967d605438715925f501f2214a3222`) | `outer_l1_v1` (0) | $B=8$ within-table outer-key & `G-D11` control |
| `WB8-Env` | 8 (64) | `…/configs/wb8-env.json` (`840431c40a0ea25567727a2ff1f6c026f70b9bcf906291ace9982941cb8628ba`) | `d5c3cb97…` | `cell_envelope_l1_v1` (1) | $B=8$ envelope treatment |
| `WB8-TwoScale` | 8 (64) | `…/configs/wb8-two-scale.json` (`c31490a11dd9525ba702534511ef92621790f8e138913df6a343267896fbc16e`) | `d5c3cb97…` | `full_vector_cell_l1_v1` (2) | $B=8$ two-scale treatment |
| `WBS8-Env` | 8 (64) | `…/configs/wbs8-env.json` (`cdcd9d102172d6f47e13ad7293f51f447cc00c756b7b0ab78bde4ee09bc38edc`) | `b8-wider-blocks-shuffled-v1-seed-20261019.json` (`1490c1b8c8878221d65f39bd975259ccaadc2a338b94bdc91c2061f0fc535dd2`) | (1) | `WB8-Env` matched shuffled control |
| `WBS8-TwoScale` | 8 (64) | `…/configs/wbs8-two-scale.json` (`9992e290a7940bc377057faade753bd7d8448e603c7a03ffc9e076e2f44e7733`) | `1490c1b8…` | (2) | `WB8-TwoScale` matched shuffled control |
| `FV8-B8-Pad-Env` | 8 (64) | `…/configs/fv8-b8-pad-env.json` (`b67157b8dbf39ff3a240eb3ab0cb1f9c2a3654e720d51b09b2f6eb813a337a7e`) | `b8-full-vector-pad-v1.json` (`2b933bc5d4f0da4eb602e5490876f80cf8ce21bdbe47306a8f27da7e90df5620`) | (1) | $B=8$ single-pair + XOR-pad control (`FullVectorV1`) |
| `FV8-B8-Pad-TwoScale` | 8 (64) | `…/configs/fv8-b8-pad-two-scale.json` (`d025c109a30dfce732c6e846c9f8947a53b17eca5459a9d3e4bf138b09d321d2`) | `2b933bc5…` | (2) | $B=8$ single-pair + XOR-pad control (`FullVectorV1`) |

## Bars and Instrument Gates

1. **G-Q (instrument — finite CUDA qualification):** `wider-carrier-qualify` passes all 4 controls on two independent CUDA builds:
   - `wider_carrier_b6_host_gpu_distance_parity` ($B = 6$, 16 rows, all 3 metrics: exact cell/token distance and `Device::evaluate` parity with host oracles, plus non-degeneracy across the 3 metrics);
   - `wider_carrier_b8_host_gpu_distance_parity` ($B = 8$, 16 rows, all 3 metrics: exact parity and non-degeneracy);
   - `wider_carrier_outer_invariance_across_b4_b6_b8` (`OuterL1V1` query endpoint blocks and admitted cells are identical across $B = 4, 6, 8$);
   - `wider_carrier_d11_export_and_state_restoration` (`export_d11` / `import_d11` round-trip at $B = 6, 8$, non-outer refusal, and zero state mutation).
2. **G-Exact & G-D11 (instrument):**
   - `A8-Outer` reproduces `4,244 / 14,287` full-DEV admitted, `1,351 / 3,847` `P_succ` admitted, `133` `Top-1`, and `513` `Top-8` exactly; `FV8-B4-Env` reproduces `5,709 / 1,980 / 118 / 1,126` and `FV8-B4-TwoScale` reproduces `5,666 / 1,802 / 118 / 1,179` (`#2834`).
   - `WB6-Outer` and `WB8-Outer` evaluate the pinned $B = 6$ and $B = 8$ tables under `OuterL1V1` (separating within-table address-metric lift from $B$-native PrimeZeta fallback placement) and verify `d11_export::export_d11` + `import_d11` round-trip (`d11_artifact_sha256` present).
3. **G-Occ (instrument & within-cell tie mass gate):** every pinned table reports `distinct_vectors == 4,096`, `occupied_cells == 256`, `within_cell_tie_tokens == 0`, `within_cell_tie_dev_rows == 0`, and `dev_alias_ceiling_rows == 14,287`.
4. **Stage 1 Wider-Carrier Bar (primary):** for treatment $T \in \{\text{WB6-Env}, \text{WB6-TwoScale}, \text{WB8-Env}, \text{WB8-TwoScale}\}$ with matched shuffled control $S$:
   - **G-Cap (alias ceiling net of admission):** $\min(\text{dev\_alias\_ceiling\_rows}(T), \text{full\_admitted}(T)) > 1,941$ and $\text{within\_cell\_tie\_tokens}(T) = 0$.
   - **Reach vs `A8-Outer`:** `psucc_admitted(T) > 1,351` and paired McNemar discordance $b - c > F = \max(24, \lceil 2\sqrt{b + c}\rceil)$ on the 3,847 `P_succ` rows.
   - **Geometry vs shuffled control $S$:** $b - c > F = \max(24, \lceil 2\sqrt{b + c}\rceil)$ for $T$ against $S$.
   - **Multi-pair lift vs XOR-pad ($B = 8$) and vs $B = 4$ (`FV8-B4`):** graded by the same paired McNemar rule (`wider-carrier-run grade`).

| Outcome | Condition | Unlocks |
|---|---|---|
| `INSTRUMENT` | `G-Q`, `G-Exact`, `G-D11`, or `G-Occ` fails | no treatment number is read; fix and rerun |
| `PASS` | at least one wider-carrier treatment clears `G-Cap`, Reach vs `A8-Outer`, and Geometry vs $S$ (with zero within-cell tie mass) | `#2847` done-when is met (`status: done`); wider-carrier vs $B=4$ comparison feeds Stage T / `#2836` |
| `DIAGNOSTIC_ONLY` | all instrument gates pass, no treatment clears both Reach and Geometry | closes only the tested wider-carrier configuration under D22; `#2834`'s $B=4$ carrier remains primary |

## Data Split, Pinned Tables, Minimum Test Power, Cost, and Stopping Conditions

- **Corpus (`uor-store`):** TRAIN `mw-balp/train` (`tokens.u16`: `3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8`, `response_mask.u8`: `b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3`, used for host table construction only); DEV `ft/dev` (`tokens.u16`: `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`, `response_mask.u8`: `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`, 14,287 rows / 3,847 `P_succ`); `tokenizer.json` (`d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`). No FRESH slice is read.
- **Tables:** sealed at `caseyallard/uor-r4-store:results/antigravity/m1-wider-code-carrier-2026-10-26/tables/` with `SHA256SUMS` matching the 8 table file SHAs above.
- **Job script:** [`docs/labs/m1-wider-code-carrier/wider-carrier-job.sh`](../../labs/m1-wider-code-carrier/wider-carrier-job.sh).
- **Stop rule:** immediate stop on any `G-Q`, `G-Exact`, `G-D11`, or `G-Occ` failure (classed `INSTRUMENT` under D31), if any single arm exceeds 10 minutes, or at 45 minutes pod wall clock.
- **Cost:** 1x RTX 5090 for ~15–20 minutes (~$0.35 at ~$1.00/h, hard cap $1.50, well inside the 4-pod / $8/h fleet cap).

## Test fitness

TEST FITNESS: FIT - M1 #2029 m1-wider-code-carrier
sheet: docs/harness/prep/m1-wider-code-carrier-2026-10-26.md
reviewer: antigravity, fresh-context non-author review (subagent 47e58bb0-c61b-497d-b3ba-2ac310c41954)
note: https://github.com/UOR-Foundation/uor-r4/issues/2847#issuecomment-6415495380
date: 2026-10-26
A FIT verdict clears compute for the configuration in that sheet only (D22).

