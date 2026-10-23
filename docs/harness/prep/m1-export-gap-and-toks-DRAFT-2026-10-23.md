# M1 Stage 4: Export Gap and Prompt-Reading / Reply-Writing Throughput (`UORLUT01`) — Pre-Registration (DRAFT)

- **Line:** M1 Stage 4 (`m1-export-gap-and-toks`, [#2839](https://github.com/UOR-Foundation/uor-r4/issues/2839), milestone [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029)).
- **Lab:** `antigravity` (`antigravity-42`).
- **Mechanism under test:** [D11 integer table serving (`UORLUT01`)](../../mechanisms/lut-serving.md) (`uor-r4.lut-discrete/1` exporter in [`crates/uor-r4-discrete/src/d11_export.rs`](../../../crates/uor-r4-discrete/src/d11_export.rs) and integer evaluator in [`crates/uor-r4-integer/src/discrete_d11.rs`](../../../crates/uor-r4-integer/src/discrete_d11.rs)).
- **Status:** `DRAFT` (`TEST FITNESS: PENDING`). One of the two required inputs ([`m1-d11-exporter-discrete`](../../roadmap/items/m1-d11-exporter-discrete.md), [#2838](https://github.com/UOR-Foundation/uor-r4/issues/2838)) is `done`; the second required input (a saved trained discrete model from `m1-a8-full-dose-rerun`, `m1-bvg-k-stage-t-training`, or `m1-trunk-assembly-and-training` that clears Stage 3 `m1-close-saved-model`) does not exist yet because training remains **HELD** under D40.

## Bar

- **Bar status:** **PROPOSED, NOT REGISTERED** (fixing a milestone bar or noise band is an owner/PM decision under [CHARTER.md §5.2](../../../CHARTER.md) and D39 item 5; the Stage 4 bars below are proposed from [`docs/roadmap/items/m1-export-gap-and-toks.md`](../../roadmap/items/m1-export-gap-and-toks.md) and [`docs/labs/k16-outcome-call-2026-10-19/README.md`](../../labs/k16-outcome-call-2026-10-19/README.md)).
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

- **Exporter and integer evaluator (merged on `main` in `#2998`):**
  - Exporter: [`crates/uor-r4-discrete/src/d11_export.rs`](../../../crates/uor-r4-discrete/src/d11_export.rs) (`export_d11`, `export_d11_to_path`, CLI subcommand `discrete-stack export-d11`).
  - Container verifier: [`crates/uor-r4-lut/src/format.rs`](../../../crates/uor-r4-lut/src/format.rs) (`DISCRETE_SCHEMA = "uor-r4.lut-discrete/1"`, `verify_discrete_header`).
  - Integer evaluator: [`crates/uor-r4-integer/src/discrete_d11.rs`](../../../crates/uor-r4-integer/src/discrete_d11.rs) (`IntegerDiscreteModel::from_path`, `evaluate_prefix`, `observe_target`).
- **Trained checkpoint input (BLOCKER before execution):**
  - Requires a saved trained discrete checkpoint JSON (`uor-r4.discrete-spiralcore-fbs/2`) from `m1-a8-full-dose-rerun`, `m1-bvg-k-stage-t-training`, or `m1-trunk-assembly-and-training`, pinned by its exact SHA-256 before any run starts.
  - Because `discrete-stack eval-corpus` refuses checkpoints with `accepted_updates > 0` while training is held (`Error::TrainingHeld` in `crates/uor-r4-discrete/src/main.rs:102`), scoring a trained checkpoint with `accepted_updates > 0` requires an owner release on `#2029` / `#2839` (CHARTER §5.1).

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
2. **GPU export and `14,287`-row DEV evaluation on leased RTX 5090 pod (CHARTER §3.5, after FIT + owner release):**
   - Check pod fleet status first: `scripts/pod/uor-pod status`.
   - Verify SHA-256 of the input checkpoint JSON and `ft/dev/tokens.u16` (`0919e90f…`) / `ft/dev/response_mask.u8` (`a838bb1a…`).
   - Export the checkpoint to `UORLUT01` (`uor-r4.lut-discrete/1`):
     `~/.cargo/bin/cargo run --release -p uor-r4-discrete --bin discrete-stack -- export-d11 CHECKPOINT.json MODEL.lut EXPORT_REPORT.json`
   - Score the `14,287`-row DEV panel on the CUDA evaluator (`discrete-stack eval-corpus`) and compare row-by-row against `IntegerDiscreteModel` (`uor-r4-integer`) on the same `14,287` rows, recording `\Delta_{\text{top1}}`, `\Delta_{\text{top8}}`, `\Delta_{\text{adm}}`, `\Delta_{\text{loss}}`, and planted-fault control B4 in `GAP_REPORT.json`.
   - Copy all receipts and `FILES.sha256` off the pod before `scripts/pod/uor-pod down`.
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
  - Pod spend `<= $0.25` (single RTX 5090 pod for `<= 15 min` at `~$0.89/h`), fleet cap `<= 4` pods and `<= $8/h` across all labs.

## Cost

- **Laptop static audits and product timing (Steps 1 & 3):** `$0.00` (local Apple Silicon M-series laptop, `< 5 min` wall time).
- **GPU DEV evaluation and parity receipt (Step 2):** `<= 15 min` on one leased RTX 5090 pod (`~$0.89/h`), estimated cost `<= $0.25`.

## Minimum test power

- **Export gap (B2):** Scored across all `14,287` frozen DEV positions (`>= 200` rows minimum per CHARTER §4.3 by `71.4x`), including the `3,847`-row `P_succ` subset. For table-native `uor-r4.lut-discrete/1` exports, the check is an exact integer identity over `14,287` rows (zero noise floor; 1 row mismatch fails). For D37 float candidates, the noise band (`\Delta_{\text{noise,top1}} = 398`, `\Delta_{\text{noise,top8}} = 748`, `\Delta_{\text{noise,adm}} = 930`) is derived directly from the two-seed K16 DEV receipts (seeds `1` and `2`).
- **Throughput timing (B3):** `1` warmup + `N = 20` timed iterations each for prompt-reading (`192` tokens/iteration = `3,840` timed prompt tokens) and reply-writing (`64` tokens/iteration = `1,280` timed decoded tokens), reporting median, nearest-rank `p95`, and all `20` raw samples.

## TEST FITNESS line

TEST FITNESS: PENDING
