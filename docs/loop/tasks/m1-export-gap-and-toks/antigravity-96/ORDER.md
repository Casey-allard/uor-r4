# ORDER: m1-export-gap-and-toks, antigravity-96

Lab: antigravity. Item: [m1-export-gap-and-toks](../../../../roadmap/items/m1-export-gap-and-toks.md) (#2839). Base: `4cdefbe4a`.

## Goal

Meet the Stage 4 (`PLAN.md` Step 6) bar for [`m1-export-gap-and-toks`](../../../../roadmap/items/m1-export-gap-and-toks.md) (#2839) under [`docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md`](../../../../harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md):
1. Wire `Δloss` (`token_hinge`, `address_hinge`, `rank_hinge`, `loss_ta`, `loss_tr`) into the GPU kernel (`src/kernel_core.h`), reference check (`observe_target`), planted-fault control `B4`, and `gap` comparator in `docs/labs/m1-d11-artifact-scorer/instrument/`, with negative-control unit tests (`claude-116/REPORT.md:89`).
2. Run the B1 static zero-multiply instruction audits (`audit_zero_matmul_serving.py` on native `uor-r4-stack` and `audit_zero_matmul_wasm.py` on `uor_r4_integer_bg.wasm`).
3. After a fresh-context D67 pre-run review posted on `#2839`, export the Stage T checkpoints (`t16-frozen-zero-seed{1,2}`, `t16-relation-seed{1,2}` from `#2836`) to `UORLUT01`, score the `14,287`-row DEV panel on a leased RTX 5090 pod (`CHARTER.md` §3.5), verify `Δtop1 = 0`, `Δtop8 = 0`, `Δadm = 0`, `Δloss = 0` (`14,287 / 14,287` row agreement against `final-dev.json`) and planted-fault control `B4`, and archive receipts to `caseyallard/uor-r4-store:results/antigravity/m1-export-gap-and-toks-2026-10-26/`.
4. Measure laptop product serving throughput (`CHARTER.md` §3.6) via `d11-scorer toks` (`1` warmup + `20` timed runs, `192`-token prompt prefill and `64`-token autoregressive decode reported separately with median, nearest-rank `p95`, and all `20` raw samples), record the result row in `docs/research/notebook/m4-serving.md`, and close `#2839`.

## Cancel probe (run first)

Ran `uor-harness check` and inspected `docs/labs/m1-d11-artifact-scorer/instrument/` and `results/antigravity/m1-bvg-k-stage-t-training-2026-10-25/stage-t-20261025.tar.gz` (`c004db506bf64903231c5c225a07e9969d5e3344c51a67eaec9429265e5f44b6`):
- **Classification (`instrument`):** `docs/harness/prep/m1-export-gap-and-toks-DRAFT-2026-10-23.md` has `TEST FITNESS: FIT` (`#2839` comment `6398709883`) and passes `prep.sheet_shape`. All four Stage T checkpoints and their 14,287-row training-side CUDA `eval-corpus` receipts (`final-dev.json`) are present in the HF store archive. Only the GPU `Δloss` fields, planted-fault control `B4`, and `gap` subcommand noted in `claude-116/REPORT.md:89` need to be wired in `docs/labs/m1-d11-artifact-scorer/instrument/` before the D67 pre-run review and GPU/laptop runs.

## Files in scope

- `docs/labs/m1-d11-artifact-scorer/instrument/**`
- `docs/harness/prep/m1-export-gap-and-toks*`
- `docs/labs/m1-export-gap-and-toks/**`
- `docs/loop/tasks/m1-export-gap-and-toks/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/m1-export-gap-and-toks.md`
- `docs/roadmap/README.md`

## Done when

1. B1 native and WASM static zero-multiply audits report `0` float, `0` mul, and `0` div instructions.
2. GPU `d11-scorer score` and `gap` on the exported Stage T `UORLUT01` artifacts verify `Δtop1 = 0`, `Δtop8 = 0`, `Δadm = 0`, `Δloss = 0` (`14,287 / 14,287` row agreement with `final-dev.json` and `IntegerDiscreteModel::observe_target`), and planted-fault control `B4` passes.
3. Separate prompt-reading (`192` tokens) and reply-writing (`64` tokens) `tok/s` rates (`1` warmup + `20` timed runs, median, `p95`, raw samples, hardware profile) are recorded on the laptop, archived to `results/antigravity/m1-export-gap-and-toks-2026-10-26/`, and logged in `docs/labs/m1-export-gap-and-toks/README.md` and `docs/research/notebook/m4-serving.md`, closing `#2839`.

## Stop and ask if

- Any B1 static audit reports non-zero float/mul/div instructions, any `uor-r4.lut-discrete/1` export has `Δ > 0` against `final-dev.json` or `observe_target`, or planted-fault control `B4` fails to detect a fault (instrument stop per D31).
