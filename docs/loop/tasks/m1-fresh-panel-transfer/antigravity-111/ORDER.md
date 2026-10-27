# ORDER: m1-fresh-panel-transfer, antigravity-111

Lab: antigravity. Item: [m1-fresh-panel-transfer](../../../../roadmap/items/m1-fresh-panel-transfer.md). Base: `b6b5a78e1`.

## Goal

The item's done-when reads:

> '(e) Fresh transfer. The gain holds on a fresh, sealed test panel registered in advance.' Read with bar (d), "the gain" is integer top-1 through the D11 export beating the unigram table (1,226 of 14,287) and the next-word table (6,300 of 14,287) on each of two seeds. The fresh-panel threshold is not yet registered; step 3 registers it.

With the two-seed 19-table `UORLUT01` (`trunk-joint-pointer-v1-d11`) GPU `d11-scorer fresh-score` evaluation on the sealed `Fresh V2` slice (`0b7e754c…` / `a1cda529…`) already executed across two independent ahead-of-time CUDA builds (`Build A` `ce5843b0…` and `Build B` `0357abf8…`) on `1x RTX 5090` (`vast-57903382`) after `#2841`'s FIT + D67 pre-run review (`#issuecomment-6432026082`) and archived in `caseyallard/uor-r4-store:results/antigravity/m1-close-saved-model-2026-10-26/outputs`, this unit records the sealed `PASS_BAR_E_FRESH_TRANSFER` evidence in `docs/labs/m1-fresh-panel-transfer/`, updates `docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md`, `docs/research/open-questions.md`, `docs/start/faq.md`, and `docs/research/notebook/m1-discrete-attention.md`, adds a regression test in `docs/labs/m1-d11-artifact-scorer/instrument/tests/stage1.rs`, and closes `#2841` (`status: done`).

## Cancel probe (run first)

1. Inspected `docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md`, `docs/labs/m1-fresh-panel-transfer/README.md`, `docs/start/faq.md:111`, and `docs/research/open-questions.md`:
   - `m1-fresh-panel-transfer-2026-10-26.md` lines 9, 28, and 60–64 still carry `UNAVAILABLE_NOT_RUN`, `prefix.windows(2).any(...)` prose (whereas `fresh_transfer.rs:245` uses `rposition` yielding the `3,594` `P_succ` rows), and the pre-fix `TRAIN`/`DEV` SHA-256 strings instead of the registered pins (`3dee858f84b7…` / `b5e7352d67d6…` and `0919e90f953e…` / `a838bb1a67e9…`).
   - `docs/labs/m1-fresh-panel-transfer/` has no `evidence/` receipts committed yet and `README.md`, `docs/start/faq.md:111`, and `docs/research/open-questions.md` still state `UNAVAILABLE_NOT_RUN` / `"a genuinely fresh test has not run"`.
   - **Cancel probe result (`classification: instrument`):** No new GPU pod is needed (`$0` spend; the single-shot `Fresh V2` evaluation is sealed); the committed `#2841` evidence, sheet pins, test, and docs must be updated and closed.

## Files in scope

- `docs/labs/m1-d11-artifact-scorer/instrument/**`
- `docs/harness/prep/m1-fresh-panel-transfer*`
- `docs/labs/m1-fresh-panel-transfer/**`
- `docs/loop/tasks/m1-fresh-panel-transfer/**`
- `docs/research/notebook/m1-discrete-attention.md`
- `docs/research/open-questions.md`
- `docs/start/faq.md`
- `docs/roadmap/items/m1-fresh-panel-transfer.md`
- `docs/roadmap/README.md`

## Done when

1. `docs/labs/m1-fresh-panel-transfer/evidence/` contains `trunk-joint-pointer-v1-d11-fresh-gate-receipt.json`, `t16-frozen-zero-trunk-fresh-score-rposition.json`, and `t16-frozen-zero-trunk-fresh-score.json`, verified by a failing-first test in `docs/labs/m1-d11-artifact-scorer/instrument/tests/stage1.rs`.
2. `docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md`, `docs/labs/m1-fresh-panel-transfer/README.md`, `docs/start/faq.md`, `docs/research/open-questions.md`, and `docs/research/notebook/m1-discrete-attention.md` record `PASS_BAR_E_FRESH_TRANSFER`.
3. `docs/roadmap/items/m1-fresh-panel-transfer.md` is `status: done`, `docs/roadmap/README.md` is re-rendered, and `REPORT.md` closes `#2841`.

## Stop and ask if

- Never stop and wait (`CHARTER.md` §5.1). Any discrepancy in the sealed `Fresh V2` receipts stops the unit for diagnosis.
