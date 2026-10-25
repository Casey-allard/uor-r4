# M1 Stage 4: Export Gap and Prompt-Reading / Reply-Writing Throughput (`m1-export-gap-and-toks`, #2839)

- **Milestone / Issue:** M1 [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029), item [`m1-export-gap-and-toks`](../../roadmap/items/m1-export-gap-and-toks.md) ([#2839](https://github.com/UOR-Foundation/uor-r4/issues/2839)).
- **Lab:** `antigravity` (`antigravity-42`, updated in `antigravity-74`).
- **Pre-registration sheet:** [`docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md`](../../harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md) (`TEST FITNESS: FIT`, D68 bar/dependency registration).

## 1. Dependency and readiness check (`How to start` step 1)

[`docs/roadmap/items/m1-export-gap-and-toks.md`](../../roadmap/items/m1-export-gap-and-toks.md) and [`PLAN.md`](../../../PLAN.md) Step 6 name two required inputs before the Stage 4 evaluation and timing run can execute:

1. **The discrete D11 exporter (`m1-d11-exporter-discrete`, [#2838](https://github.com/UOR-Foundation/uor-r4/issues/2838)):** **COMPLETE (`status: done`, merged in `#2998`).**
   - [`crates/uor-r4-discrete/src/d11_export.rs`](../../../crates/uor-r4-discrete/src/d11_export.rs) exports `DiscreteModel` snapshots into a single `UORLUT01` (`uor-r4.lut-discrete/1`) container with four 64-byte-aligned integer sections (`discrete.table` `i16`, `discrete.cells` `u32`, `discrete.routes` `u32`, `discrete.gates` `i32`) and zero 4-bit matrix sections.
   - [`crates/uor-r4-integer/src/discrete_d11.rs`](../../../crates/uor-r4-integer/src/discrete_d11.rs) (`IntegerDiscreteModel`) loads `uor-r4.lut-discrete/1` artifacts and evaluates `Route`, `Flock`, and `Relation` prefixes in pure integer arithmetic (`0` float, `0` mul/div on the audited serving symbols).
2. **A saved trained discrete model from Step 4 (`m1-bvg-k-stage-t-training`, [#2836](https://github.com/UOR-Foundation/uor-r4/issues/2836)):** **PENDING STEP 4 (`PLAN.md` line 63).**
   - K16 (`docs/labs/k16-outcome-call-2026-10-19/README.md`) ended in `OCCUPANCY_COLLAPSE_TRIGGERED` with no `KEEP`, the ungated 19 Oct A8 run (`docs/labs/a8-dose-2026-10-19/README.md`) stopped at Q1 admission collapse (`P_succ` `1,351 -> 551`), and the gated 25 Oct A8 rerun (`docs/labs/m1-a8-full-dose-rerun/README.md`, `#2831`, PR `#3192`) passed `Q1` on `FrozenZero` (`DEV top1_correct = 871 / 14,287`, `b_3(P_succ) = 1,106 / 3,847`) before stopping on the `consecutive_full_rejections = 3` cadence-granularity diagnostic stop at batch `1,600` (`Relation`) / `5,248` (`FrozenZero`).
   - Per `PLAN.md` Step 6 ("`the exporter (done, #2838); its measured run needs a trained model from step 4`") and Step 1 of `m1-export-gap-and-toks.md` ("do not start a run without both"), `m1-export-gap-and-toks` depends on [`m1-bvg-k-stage-t-training`](../../roadmap/items/m1-bvg-k-stage-t-training.md) (`#2836`) and will execute its GPU DEV export-gap verification and laptop throughput timing immediately after `#2836` produces its trained checkpoints.

## 2. Stage 4 design summary (`How to start` step 2)

1. **Frozen DEV panel and SHA-256 bindings:**
   - `14,287` scored positions across `400` documents (`context = 192`), with `ft/dev/tokens.u16` (`720,420` B, SHA-256 `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`) and `ft/dev/response_mask.u8` (`360,210` B, SHA-256 `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`).
2. **Dual-path oracle on the same rows:**
   - Compares the training-side CUDA evaluator (`discrete-stack eval-corpus`) and the exported `UORLUT01` integer evaluator (`IntegerDiscreteModel::observe_target`) on the identical `14,287` DEV prefixes and tie-break order.
3. **Export-gap statistic and two-seed noise band:**
   - Reports `\Delta_{\text{top1}} = |\text{top1}_{\text{train}} - \text{top1}_{\text{D11}}|`, `\Delta_{\text{top8}}`, `\Delta_{\text{adm}}`, and `\Delta_{\text{loss}}`.
   - For lossless table-native `uor-r4.lut-discrete/1` discrete exports, the required gap is **exact zero** (`\Delta = 0` across all `14,287` rows); any non-zero difference is an instrument fault (D31).
   - For D37 offline float-trained candidates quantized into `UORLUT01`, the candidate is disqualified when its export gap exceeds the two-seed empirical noise band from the K16 DEV receipts (`\Delta_{\text{noise,top1}} = 398` rows, `\Delta_{\text{noise,top8}} = 748` rows, `\Delta_{\text{noise,adm}} = 930` rows; [`docs/labs/k16-outcome-call-2026-10-19/README.md`](../k16-outcome-call-2026-10-19/README.md)).
4. **Separate prompt-reading and reply-writing timing protocol:**
   - Recorded on client Apple Silicon M-series laptop hardware (`1` warmup + `N = 20` timed runs each, reporting median, nearest-rank `p95`, and all `20` raw samples):
     - **Prompt-reading rate (context prefill, `tok/s`):** `L_{\text{prompt}} = 192`-token causal prefix evaluation.
     - **Reply-writing rate (autoregressive decode, `tok/s`):** `L_{\text{reply}} = 64`-token autoregressive continuation stepping over the sliding `192`-token context window.
5. **Compute boundary ([CHARTER.md §3.5 vs §3.6](../../../CHARTER.md)):**
   - Research evaluation (`14,287`-row DEV scoring and export parity verification) runs on a leased RTX 5090 GPU pod via `scripts/pod/uor-pod` (§3.5).
   - Static zero-multiply instruction audits (`44` native symbols via `audit_zero_matmul_serving.py` and `7/7` WebAssembly symbols via `audit_zero_matmul_wasm.py`) and product serving throughput timing (`prompt-reading` and `reply-writing` `tok/s`) run on the client laptop (§3.6).

## 3. Open prerequisites before execution

1. **Completed in `antigravity-74`:** Non-author `TEST FITNESS: FIT` review of [`docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md`](../../harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md) (`CHARTER.md` §4.5) and D68 registration of the Stage 4 bar, two-seed noise band (`\Delta_{\text{noise,top1}} = 398`, `\Delta_{\text{noise,top8}} = 748`, `\Delta_{\text{noise,adm}} = 930`), and `m1-bvg-k-stage-t-training` (`#2836`) dependency (`CHARTER.md` §5.1).
2. **Remaining prerequisite (`PLAN.md` Step 4 -> Step 6):** Completion of [`m1-bvg-k-stage-t-training`](../../roadmap/items/m1-bvg-k-stage-t-training.md) (`#2836`) to produce the two-seed trained discrete checkpoints (`uor-r4.discrete-spiralcore-fbs/2`), followed by a D67 pre-run review (`CHARTER.md` §5.1) for the Stage 4 GPU DEV evaluation and laptop throughput timing.
