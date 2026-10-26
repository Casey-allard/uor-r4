# M1 Stage 4: Export Gap and Prompt-Reading / Reply-Writing Throughput (`UORLUT01`) — Pre-Registration (DRAFT)

- **Line:** M1 Stage 4 (`m1-export-gap-and-toks`, [#2839](https://github.com/UOR-Foundation/uor-r4/issues/2839), milestone [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029)).
- **Lab:** `antigravity` (`antigravity-42`, updated in `antigravity-74`).
- **Run:** `m1-export-gap-and-toks-stage4`
- **Mechanism under test:** [D11 integer table serving (`UORLUT01`)](../../mechanisms/lut-serving.md) (`uor-r4.lut-discrete/1` exporter in [`crates/uor-r4-discrete/src/d11_export.rs`](../../../crates/uor-r4-discrete/src/d11_export.rs) and integer evaluator in [`crates/uor-r4-integer/src/discrete_d11.rs`](../../../crates/uor-r4-integer/src/discrete_d11.rs)).
- **Status:** Pre-registered (`TEST FITNESS: FIT` in `antigravity-74`, updated with Stage T `#2836` pins in `antigravity-96`). All three required inputs — [`m1-d11-exporter-discrete`](../../roadmap/items/m1-d11-exporter-discrete.md) ([#2838](https://github.com/UOR-Foundation/uor-r4/issues/2838)), [`m1-bvg-k-stage-t-training`](../../roadmap/items/m1-bvg-k-stage-t-training.md) ([#2836](https://github.com/UOR-Foundation/uor-r4/issues/2836), `PLAN.md` Step 4), and [`m1-d11-artifact-scorer`](../../roadmap/items/m1-d11-artifact-scorer.md) ([#3248](https://github.com/UOR-Foundation/uor-r4/issues/3248)) — are `done`.

## Question

On one leased RTX 5090 pod (`CHARTER.md` §3.5) and the client laptop (`CHARTER.md` §3.6), do the exported `UORLUT01` (`uor-r4.lut-discrete/1`) integer artifacts of the Stage T trained discrete checkpoints (`t16-frozen-zero-seed{1,2}`, `t16-relation-seed{1,2}` from `#2836`) pass the B1 zero-multiply static audits, achieve exact zero export gap (`\Delta_{\text{top1}} = 0`, `\Delta_{\text{top8}} = 0`, `\Delta_{\text{adm}} = 0`, `\Delta_{\text{loss}} = 0`, `14,287 / 14,287` row agreement) against their sealed training-side CUDA `eval-corpus` receipts (`final-dev.json`) with planted-fault control B4 verified, and report separate prompt-reading (`192`-token prefill) and reply-writing (`64`-token autoregressive decode) throughput (`tok/s`)?

## Bar

- **Bar status:** **REGISTERED UNDER D68** ([CHARTER.md §5.1](../../../CHARTER.md), D39 item 5, D68 decision review on `#2839`; derived from [`docs/roadmap/items/m1-export-gap-and-toks.md`](../../roadmap/items/m1-export-gap-and-toks.md) and [`docs/labs/k16-outcome-call-2026-10-19/README.md`](../../labs/k16-outcome-call-2026-10-19/README.md)).
- **Stage 4 decisive criteria (all four must hold on the selected checkpoint):**
  1. **B1 — Zero-multiply static instruction audits (native + WebAssembly):**
     - Native release audit (`python3 scripts/audit_zero_matmul_serving.py target/release/uor-r4-stack --stack`) reports `0` floating-point, `0` multiply, and `0` divide instructions across all `44` audited symbol ranges.
     - WebAssembly serving audit (`python3 scripts/audit_zero_matmul_wasm.py crates/uor-r4-integer/pkg/uor_r4_integer_bg.wasm --serving --call-graph --tap`) reports `7 / 7` serving symbols PASS with `mul = 0`, `div = 0`, `float = 0`, and `unclassified = 0`.
  2. **B2 — Export gap on the `14,287`-row DEV panel (`\Delta_{\text{export}}`):**
     - Let `(top1_train, top8_train, adm_train, loss_train)` be the training-side evaluator counts on the `14,287` DEV rows (`discrete-stack eval-corpus`), and let `(top1_d11, top8_d11, adm_d11, loss_d11)` be the integer evaluator counts read from the exported `UORLUT01` file on the **exact same `14,287` rows and tie-break order** (`IntegerDiscreteModel::observe_target`).
     - **Table-native discrete checkpoint (`uor-r4.lut-discrete/1`):** Because `d11_export::export_d11` packs `discrete.table` (`i16`), `discrete.cells` (`u32`), `discrete.routes` (`u32`), and `discrete.gates` (`i32`) losslessly (`import_d11(export_d11(&snapshot)) == snapshot`), the required export gap is **exact zero** on all metrics:
       $$\Delta_{\text{top1}} = |\text{top1}_{\text{train}} - \text{top1}_{\text{D11}}| = 0, \quad \Delta_{\text{top8}} = 0, \quad \Delta_{\text{adm}} = 0, \quad \Delta_{\text{loss}} = 0,$$
       and row-by-row top-1 prediction agreement is `14,287 / 14,287`. Any non-zero discrepancy on a table-native discrete export is an **instrument fault** (D31 diagnostic stop).
     - **Offline float-trained candidate (`D37` / D39 item 5, [CHARTER.md §3.7](../../../CHARTER.md)):** For any candidate trained in an offline float module (`d11-exporter-float-trunk`) and quantized to `UORLUT01`, the export gap must not exceed the two-seed empirical noise band on the same `14,287`-row DEV evaluator ([`docs/labs/k16-outcome-call-2026-10-19/README.md`](../../labs/k16-outcome-call-2026-10-19/README.md)):
       $$\Delta_{\text{top1}} \le \Delta_{\text{noise,top1}} = 398\text{ rows}, \quad \Delta_{\text{top8}} \le \Delta_{\text{noise,top8}} = 748\text{ rows}, \quad \Delta_{\text{adm}} \le \Delta_{\text{noise,adm}} = 930\text{ rows}.$$
       A float candidate whose export gap exceeds the two-seed band is disqualified on that configuration (D22).
  3. **B3 — Prompt-reading (context prefill) and reply-writing (autoregressive decode) throughput reported separately:**
     - Both rates (`tok/s`) are measured on the client laptop (`1` warmup + `N = 20` timed runs each) and reported separately with median, nearest-rank `p95`, all `20` raw samples, and full hardware metadata.
  4. **B4 — Planted-fault (negative control) sensitivity:**
     - Flipping a single non-zero route action (`routes[0][0] ^= 1`) or writer cell (`cells[0] ^= 1`) in an in-memory copy of the exported `UORLUT01` container must fail the `snapshot_sha256` / parity check (`> 0` row mismatches), confirming the gap harness is non-vacuous.

## Base artifact

- **Exporter, integer evaluator, and GPU artifact scorer (merged on `main` in `#2998` and `#3249`):**
  - Exporter: [`crates/uor-r4-discrete/src/d11_export.rs`](../../../crates/uor-r4-discrete/src/d11_export.rs) (`export_d11`, `export_d11_to_path`).
  - Container verifier: [`crates/uor-r4-lut/src/format.rs`](../../../crates/uor-r4-lut/src/format.rs) (`DISCRETE_SCHEMA = "uor-r4.lut-discrete/1"`, `verify_discrete_header`).
  - Integer evaluator: [`crates/uor-r4-integer/src/discrete_d11.rs`](../../../crates/uor-r4-integer/src/discrete_d11.rs) (`IntegerDiscreteModel::from_path`, `evaluate_prefix`, `observe_target`).
  - GPU artifact scorer instrument: [`docs/labs/m1-d11-artifact-scorer/instrument/`](../../labs/m1-d11-artifact-scorer/instrument/) (`d11-scorer export`, `d11-scorer score --reference --train-eval`, `d11-scorer toks`).
- **Trained checkpoint inputs (`PLAN.md` Step 4 -> Step 6 dependency, archived in `caseyallard/uor-r4-store:results/antigravity/m1-bvg-k-stage-t-training-2026-10-25/stage-t-20261025.tar.gz`, SHA-256 `c004db506bf64903231c5c225a07e9969d5e3344c51a67eaec9429265e5f44b6`):**
  - `t16-frozen-zero-seed1`:
    - `final-checkpoint.json`: SHA-256 `0831900fc6c04b4489215dc93876f87a173b2c6c16b6635dde2f2ff24c1015a4` (`model_sha256 = de8d94ba15e3f7889681a244e5480b260f68ae5ee7f57944d44f4da8fef4e817`, `epoch = accepted_updates = 420`).
    - `final-dev.json` (training-side CUDA `eval-corpus`): SHA-256 `a7d23490c511fc7a391c95546646cd6aaef391950a5fb541fd13d2029a6edf8e` (`top1 = 455`, `top8 = 1,484`, `target_admitted = 5,009`, `loss_ta = 12,920,928`, `loss_tr = 29,531,840`).
    - Exported `t16-frozen-zero-seed1.lut` (`542,260` bytes): SHA-256 `fc31bcc7a88250f3758d07a8b3d12480a5332acb5437b0c0f0e5c71ae88939ca`.
  - `t16-frozen-zero-seed2`:
    - `final-checkpoint.json`: SHA-256 `8bbb59503a16b8bbd2cc723cbd3bc405543f65dfab00ee9ed6c4e9b43271842c` (`model_sha256 = 6f77137b3960563211e76b4de482434cc11bc9245ab578a4e47d139fd77f0fd9`, `epoch = accepted_updates = 420`).
    - `final-dev.json` (training-side CUDA `eval-corpus`): SHA-256 `644a89ae0a5b817f57f92c1047096f343be50d34f112929180f1c9027bf71e78` (`top1 = 455`, `top8 = 1,484`, `target_admitted = 5,009`, `loss_ta = 12,920,928`, `loss_tr = 29,531,840`).
    - Exported `t16-frozen-zero-seed2.lut` (`542,260` bytes): SHA-256 `a75ea7f5907275fd4c1e853ee76135a7a2615ba8302aaadf362bfa7eb96ae12a`.
  - `t16-relation-seed1`:
    - `final-checkpoint.json`: SHA-256 `558b8810dc888d3acd2d5d135b9137062ce69f74e4148a822e1b59b9cfccb944` (`model_sha256 = aeeb92f80399af1effe569ae0fe3a19f15a7940c209b85ac818c5de511a2b27e`, `epoch = accepted_updates = 678`).
    - `final-dev.json` (training-side CUDA `eval-corpus`): SHA-256 `e73d9330ed907b8ba87d17f73b8e947136b8eb2e40a48454c89bb29601213261` (`top1 = 371`, `top8 = 1,124`, `target_admitted = 3,827`, `loss_ta = 13,604,128`, `loss_tr = 31,562,816`).
    - Exported `t16-relation-seed1.lut` (`542,260` bytes): SHA-256 `d64e127c79c1e3be65277efe507473162bd4658b22ac8daf39b165ccfc5e25f2`.
  - `t16-relation-seed2`:
    - `final-checkpoint.json`: SHA-256 `1757a3c54692061c112850a8aabfff41671426d48eb3a37fd6a46883242d29d2` (`model_sha256 = 60d6b8cd894f42e32a1c67d0befdf17daab6d87b2ee7b78035514aaf245f8fa5`, `epoch = accepted_updates = 678`).
    - `final-dev.json` (training-side CUDA `eval-corpus`): SHA-256 `2700ababcc5b12978270206df7de5e97c4db29bff23f9c6e20eca6215d2a1377` (`top1 = 371`, `top8 = 1,124`, `target_admitted = 3,827`, `loss_ta = 13,604,128`, `loss_tr = 31,562,816`).
    - Exported `t16-relation-seed2.lut` (`542,260` bytes): SHA-256 `c21d992fb4a70b93b916c6de9aa9042d3f3f529c57fa4467f9928334f65383c1`.
  - Under D67 ([CHARTER.md §5.1, §6](../../../CHARTER.md)), the owner-release hold on runs is retired and replaced by the lab's own fresh-context D67 pre-run review recorded on `#2839`.

## Data split

- **Frozen DEV evaluation panel (`14,287` rows, `400` documents, `context = 192`):**
  - `ft/dev/tokens.u16`: `720,420` bytes, SHA-256 `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b` (pinned in `crates/uor-r4-discrete/src/attention_learning.rs:28`).
  - `ft/dev/response_mask.u8`: `360,210` bytes, SHA-256 `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078` (pinned in `crates/uor-r4-discrete/src/attention_learning.rs:30`).
  - Active `P_succ` subset: `3,847` rows where `response_mask == 1` and the target differs from the context-free successor prediction (`docs/labs/address-cells-preflight-2026-10-19/README.md`).
- **Two-seed noise reference (`docs/labs/k16-outcome-call-2026-10-19/README.md`):**
  - Extracted fields JSONL (`docs/labs/k16-outcome-call-2026-10-19/evidence/k16-outcome-fields.jsonl`): SHA-256 `ea6f904895795a9d6d8899e617dd0182c38bce1f60888370549352942b0d4502`.
  - Objective A seed 1 (`6936cf9983f18c2e08d13e4a97e1b942f6fc3c60f14b2fa704fff3e5f86ae5f3`): `target_admitted = 2,237`, `top1 = 65`, `top8 = 547`, `occupied = 209`.
  - Objective A seed 2 (`d7976a0bce774f79c2ee06df5a8f81879e53846cc73a5e50309e3d84e7068180`): `target_admitted = 3,167`, `top1 = 463`, `top8 = 1,295`, `occupied = 192`.
  - Objective R seed 1 (`bb6885e8ee0d30e8efce0f6a08a9b22873894023a229056a710ef5bb0f27575c`): `target_admitted = 12,089`, `top1 = 101`, `top8 = 515`, `occupied = 20`.
  - Objective R seed 2 (`0dfd6fa5707012832fc88e4ab1f39e9ef86a6be1ccaa47f2cd7daace5f080df9`): `target_admitted = 12,466`, `top1 = 230`, `top8 = 824`, `occupied = 19`.
  - Two-seed empirical noise bands on the `14,287`-row DEV evaluator:
    - `\Delta_{\text{noise,top1}} = |463 - 65| = 398` rows (`2.79%` of `14,287`).
    - `\Delta_{\text{noise,top8}} = |1,295 - 547| = 748` rows (`5.24%` of `14,287`).
    - `\Delta_{\text{noise,adm}} = \max(|3,167 - 2,237|, |12,466 - 12,089|, 87) = 930` rows (`6.51%` of `14,287`).
- **Uncontaminated fresh slice:**
  - The sealed fresh transfer slice (`4bd108ad…`) is **not** opened in Stage 4; fresh transfer belongs to `m1-fresh-panel-transfer` (`#2841`) only after Stage 4 passes.

## Which side runs where (CHARTER §3.5 vs §3.6)

| Step | Work | Where it runs | Charter rule |
|---|---|---|---|
| 1 | Export checkpoint (`discrete-stack export-d11`) and run training-side CUDA evaluation (`discrete-stack eval-corpus`) + GPU-side `14,287`-row DEV parity/gap evaluation | Leased RTX 5090 GPU pod via `scripts/pod/uor-pod` | **CHARTER §3.5** (research evaluation and qualification run on GPU) |
| 2 | Static zero-multiply instruction audits (`audit_zero_matmul_serving.py` across `44` native symbols; `audit_zero_matmul_wasm.py` across `7/7` WASM symbols) | Client laptop (CPU build + static disassembly check) | **CHARTER §3.5 / §3.6** (CPU builds and static checks run on laptop) |
| 3 | Product serving throughput measurement: separate **prompt-reading rate** (`192`-token context prefill, `1` warmup + `20` timed runs) and **reply-writing rate** (`64`-token autoregressive decode, `1` warmup + `20` timed runs) on the exported `UORLUT01` integer artifact | Client laptop (Apple Silicon M-series, native `uor-r4-integer` release binary) | **CHARTER §3.6** (serving the exported integer artifact on client devices is the product) |

## Steps

1. **Pre-flight static audits on the laptop (CPU build + disassembly check):**
   - Build native serving binaries:
     `~/.cargo/bin/cargo build --release -p uor-r4-integer --features serving-wasm`
   - Build WebAssembly module:
     `RUSTFLAGS="-C target-feature=-simd128" wasm-pack build crates/uor-r4-integer --target web --no-opt -- --features serving-wasm`
   - Run native zero-multiply audit:
     `python3 scripts/audit_zero_matmul_serving.py target/release/uor-r4-stack --stack`
   - Run WebAssembly zero-multiply audit:
     `python3 scripts/audit_zero_matmul_wasm.py crates/uor-r4-integer/pkg/uor_r4_integer_bg.wasm --serving --call-graph --tap`
2. **GPU export and `14,287`-row DEV evaluation on leased RTX 5090 pod (CHARTER §3.5, after FIT + D67 pre-run review):**
   - Check pod fleet status first: `scripts/pod/uor-pod status`.
   - Verify SHA-256 of the input Stage T checkpoint JSONs (`t16-frozen-zero-seed{1,2}`, `t16-relation-seed{1,2}`), their `final-dev.json` training-side evaluation receipts, and `ft/dev/tokens.u16` (`0919e90f…`) / `ft/dev/response_mask.u8` (`a838bb1a…`).
   - Build the ahead-of-time CUDA `d11-scorer` instrument (`docs/labs/m1-d11-artifact-scorer/instrument`, `UOR_NVCC_ARCH=compute_120 --features cuda --target x86_64-unknown-linux-gnu`) and run its finite GPU test suite (`cargo test --release --locked --features cuda --target x86_64-unknown-linux-gnu --no-fail-fast -- --include-ignored`).
   - Export each Stage T checkpoint to `UORLUT01` (`uor-r4.lut-discrete/1`) via `d11-scorer export CHECKPOINT.json MODEL.lut EXPORT_REPORT.json` (which invokes `uor_r4_discrete::d11_export::export_d11`) and verify the `.lut` SHA-256 matches the laptop export byte-for-byte.
   - Score the `14,287`-row DEV panel on the GPU (`d11-scorer score MODEL.lut CONFIG.json SCORE_REPORT.json SCORE_ROWS.jsonl --reference --train-eval FINAL_DEV.json GAP_REPORT.json`), verifying row-by-row parity against `IntegerDiscreteModel::observe_target` (`14,287 / 14,287` rows, `refused = 0`), planted-fault control B4 (`planted_fault_b4.passed = true`), and exact zero export gap against the training-side CUDA `eval-corpus` receipt (`\Delta_{\text{top1}} = 0`, `\Delta_{\text{top8}} = 0`, `\Delta_{\text{adm}} = 0`, `\Delta_{\text{loss\_ta}} = 0`, `\Delta_{\text{loss\_tr}} = 0`).
   - Copy all receipts and `SHA256SUMS` off the pod and archive to `caseyallard/uor-r4-store:results/antigravity/m1-export-gap-and-toks-2026-10-26/` before `scripts/pod/uor-pod release` and `scripts/pod/uor-pod down`.
3. **Laptop product serving timing protocol (CHARTER §3.6):**
   - Record laptop hardware profile (`sw_vers`, `sysctl -n machdep.cpu.brand_string`, physical/performance cores, memory, power state).
   - **Prompt-reading rate (context prefill):** On a deterministic `192`-token prompt prefix, run `1` warmup iteration followed by `N = 20` timed iterations of full-prefix evaluation (`IntegerDiscreteModel::evaluate_prefix` over `L_{\text{prompt}} = 192` tokens). Record all `20` raw elapsed durations (ns), median `tok/s`, and nearest-rank `p95` `tok/s` (rank `19` of `20` sorted durations).
   - **Reply-writing rate (autoregressive decode):** Starting from the `192`-token prompt prefix, generate `L_{\text{reply}} = 64` tokens autoregressively (appending each predicted token and sliding the `192`-token window for the next step) for `1` warmup iteration followed by `N = 20` timed iterations. Record all `20` raw elapsed durations (ns), median `tok/s`, and nearest-rank `p95` `tok/s` separately from the prompt-reading rate.
4. **Notebook and roadmap record:**
   - Record one row in [`docs/research/notebook/m4-serving.md`](../../research/notebook/m4-serving.md) with the export gap, the registered band, the separate prompt-reading and reply-writing rates, and the verdict word (`positive`, `negative`, `inconclusive`, or `diagnostic-stop`).

## Stop

- **Instrument fault (D31 diagnostic stop, never a mechanism negative):**
  - Any non-zero instruction count in B1 (`float > 0`, `mul > 0`, or `div > 0`), any SHA-256 mismatch on `ft/dev/tokens.u16` or `ft/dev/response_mask.u8`, any non-zero discrepancy (`\Delta > 0`) on a lossless `uor-r4.lut-discrete/1` export, or failure of the planted-fault control B4 stops the run immediately as an instrument fault.
- **Configuration negative (D22):**
  - For an offline float-trained candidate (D37), an export gap exceeding `\Delta_{\text{noise,top1}} = 398` rows, `\Delta_{\text{noise,top8}} = 748` rows, or `\Delta_{\text{noise,adm}} = 930` rows on the `14,287` DEV rows disqualifies that candidate configuration without closing the D11 serving mechanism.
- **Hard resource caps:**
  - Pod spend `<= $0.50` (single RTX 5090 pod for `<= 15 min` at `~$1.41/h` on a 1-hour lease), fleet cap `<= 4` pods and `<= $8/h` across all labs.

## Cost

- **Laptop static audits and product timing (Steps 1 & 3):** `$0.00` (local Apple Silicon M-series laptop, `< 5 min` wall time).
- **GPU DEV evaluation and parity receipt (Step 2):** `<= 15 min` on one leased RTX 5090 pod (`~$1.41/h`), estimated cost `<= $0.50`.

## Minimum test power

- **Export gap (B2):** Scored across all `14,287` frozen DEV positions (`>= 200` rows minimum per CHARTER §4.3 by `71.4x`), including the `3,847`-row `P_succ` subset. For table-native `uor-r4.lut-discrete/1` exports, the check is an exact integer identity over `14,287` rows (zero noise floor; 1 row mismatch fails). For D37 float candidates, the noise band (`\Delta_{\text{noise,top1}} = 398`, `\Delta_{\text{noise,top8}} = 748`, `\Delta_{\text{noise,adm}} = 930`) is derived directly from the two-seed K16 DEV receipts (seeds `1` and `2`).
- **Throughput timing (B3):** `1` warmup + `N = 20` timed iterations each for prompt-reading (`192` tokens/iteration = `3,840` timed prompt tokens) and reply-writing (`64` tokens/iteration = `1,280` timed decoded tokens), reporting median, nearest-rank `p95`, and all `20` raw samples.

## TEST FITNESS line

TEST FITNESS: FIT - M1 #2029 lut-serving
sheet: docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md
reviewer: antigravity-fresh-reviewer (8a915285-22c1-4fcf-b336-9f292d2a5b7d; re-verified with Stage T pins by ac559562-b8ce-4d82-b1d8-d8ae66015505 at b287ccc3144c1a894c75066c02d0302138bc8b59)
date: 2026-10-26
A FIT verdict clears compute for the configuration in that sheet only (D22).
