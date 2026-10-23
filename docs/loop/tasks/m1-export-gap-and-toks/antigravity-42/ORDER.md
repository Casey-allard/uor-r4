# ORDER: m1-export-gap-and-toks, antigravity-42

Lab: antigravity. Item: [m1-export-gap-and-toks](../../../../roadmap/items/m1-export-gap-and-toks.md). Base: `018954726`.

## Goal

The item's done-when reads:

> Stage 4 bar: 'the integer audits stay green, the gap stays within the noise band, and tokens per second is reported for both reading the prompt and writing the reply.' Decode and context rates are reported separately.

## Cancel probe (run first)

Ran `uor-harness loop gates --quick` and sent `SIGINT` after `150 ms`; process terminated cleanly (`rc = -2`, no orphan subprocesses).

## Files in scope

- `crates/uor-r4-integer/**`
- `docs/mechanisms/lut-serving.md`
- `docs/harness/prep/m1-export-gap-and-toks*`
- `docs/labs/m1-export-gap-and-toks/**`

## Done when

- `docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md` defines the Stage 4 pre-registration sheet for `m1-export-gap-and-toks` (#2839, #2029) with all eight required `prep.sheet_shape` blocks (`Bar`, `Base artifact`, `Data split`, `Steps`, `Stop rule`, `Cost`, `Minimum test power`, and `TEST FITNESS: PENDING`), specifying:
  1. the frozen DEV panel (`14,287` rows, `ft/dev/tokens.u16` SHA-256 `0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b`, `ft/dev/response_mask.u8` SHA-256 `a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078`) and its two-seed K16 receipts (`docs/labs/k16-outcome-call-2026-10-19/README.md`);
  2. the dual-path oracle comparing the CUDA training-side evaluator (`discrete-stack eval-corpus`) and the `uor-r4.lut-discrete/1` (`UORLUT01`) integer evaluator (`crates/uor-r4-integer/src/discrete_d11.rs::IntegerDiscreteModel`) on identical rows;
  3. the export-gap statistic (`\Delta_{\text{top1}} = |\text{top1}_{\text{train}} - \text{top1}_{\text{D11}}|`, `\Delta_{\text{top8}}`, `\Delta_{\text{adm}}`, `\Delta_{\text{loss}}`) against the exact lossless discrete expectation (`\Delta = 0` on table-native `uor-r4.lut-discrete/1`) and the two-seed band (`\Delta_{\text{noise,top1}} = 398` rows, `\Delta_{\text{noise,top8}} = 748` rows, `\Delta_{\text{noise,adm}} = 930` rows from K16 seeds 1 and 2) for D37 float-trained candidates (D39 item 5);
  4. the separate prompt-reading (context prefill) and reply-writing (autoregressive decode) timing protocol on Apple Silicon M-series laptop hardware (`1` warmup + `N = 20` timed runs, reporting median, nearest-rank p95, and all raw samples), plus the zero-multiply native (`44` symbols) and WebAssembly (`7/7` symbols) static audits;
  5. the compute boundary separating GPU research evaluation (CHARTER §3.5) from client product serving on the laptop (CHARTER §3.6).
- `docs/labs/m1-export-gap-and-toks/README.md` records the Stage 4 design note, dependency check (`m1-d11-exporter-discrete` is `done`, while a passing trained discrete model from `m1-a8-full-dose-rerun`, `m1-bvg-k-stage-t-training`, or `m1-trunk-assembly-and-training` is still pending), and link to the draft sheet.
- `docs/mechanisms/lut-serving.md` links the Stage 4 export-gap and throughput pre-registration sheet.
- The design note and draft sheet are posted on `#2839`.

## Stop and ask if

- Any GPU run or owner-held training/evaluation release is needed (this unit is doc-only and stages the draft pre-registration sheet and design note for non-author `TEST FITNESS` review).
- A file outside the item's `scope:` needs to be modified.
