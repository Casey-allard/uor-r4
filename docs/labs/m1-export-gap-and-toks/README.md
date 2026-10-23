# M1 Stage 4: Export Gap and Prompt-Reading / Reply-Writing Throughput (`m1-export-gap-and-toks`, #2839)

- **Milestone / Issue:** M1 [#2029](https://github.com/UOR-Foundation/uor-r4/issues/2029), item [`m1-export-gap-and-toks`](../../roadmap/items/m1-export-gap-and-toks.md) ([#2839](https://github.com/UOR-Foundation/uor-r4/issues/2839)).
- **Lab:** `antigravity` (`antigravity-42`).
- **Draft pre-registration sheet:** [`docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md`](../../harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md) (`TEST FITNESS: PENDING`).

## 1. Dependency and readiness check (`How to start` step 1)

[`docs/roadmap/items/m1-export-gap-and-toks.md`](../../roadmap/items/m1-export-gap-and-toks.md) names two required inputs before the Stage 4 evaluation and timing run can execute:

1. **The discrete D11 exporter (`m1-d11-exporter-discrete`, [#2838](https://github.com/UOR-Foundation/uor-r4/issues/2838)):** **COMPLETE (`status: done`, merged in `#2998`).**
   - [`crates/uor-r4-discrete/src/d11_export.rs`](../../../crates/uor-r4-discrete/src/d11_export.rs) exports `DiscreteModel` snapshots into a single `UORLUT01` (`uor-r4.lut-discrete/1`) container with four 64-byte-aligned integer sections (`discrete.table` `i16`, `discrete.cells` `u32`, `discrete.routes` `u32`, `discrete.gates` `i32`) and zero 4-bit matrix sections.
   - [`crates/uor-r4-integer/src/discrete_d11.rs`](../../../crates/uor-r4-integer/src/discrete_d11.rs) (`IntegerDiscreteModel`) loads `uor-r4.lut-discrete/1` artifacts and evaluates `Route`, `Flock`, and `Relation` prefixes in pure integer arithmetic (`0` float, `0` mul/div on the audited serving symbols).
2. **A saved trained discrete model (`m1-a8-full-dose-rerun`, `m1-bvg-k-stage-t-training`, or `m1-trunk-assembly-and-training`):** **NOT YET AVAILABLE (BLOCKED on HELD training under D40).**
   - K16 (`docs/labs/k16-outcome-call-2026-10-19/README.md`) ended in `OCCUPANCY_COLLAPSE_TRIGGERED` with no `KEEP`, and A8 (`docs/labs/a8-dose-2026-10-19/README.md`) stopped at the Q1 admission-collapse diagnostic stop (`P_succ` `1,351 -> 551`).
   - Per step 1 of `m1-export-gap-and-toks.md` ("do not start a run without both"), no GPU evaluation or laptop timing claim is executed in this unit; this unit delivers the Stage 4 design note and one-page draft pre-registration sheet (`How to start` step 2).

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

1. Non-author `TEST FITNESS: FIT` review of [`docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md`](../../harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md) (CHARTER §4.5).
2. Availability of a saved trained discrete checkpoint (`uor-r4.discrete-spiralcore-fbs/2`) and owner/PM registration of the Stage 4 bar and release on `#2839` / `#2029` (CHARTER §5.1, §5.2).
