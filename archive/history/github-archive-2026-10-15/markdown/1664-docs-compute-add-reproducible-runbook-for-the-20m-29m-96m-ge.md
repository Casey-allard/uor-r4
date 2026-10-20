# #1664 — docs(compute): add reproducible runbook for the 20M/29M/~96M geometric ladder (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-03T22:39:55Z
- **Updated:** 2026-10-09T13:17:07Z
- **Closed:** 2026-10-03T23:36:11Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1664
- **Merged:** 2026-10-03T23:36:11Z
- **Merge commit:** 9919bcd9a1d96364c9d410ad825591facfc56d83
- **Base / Head:** main / UOR-Foundation:codex/ag-ladder-runbook
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary
Adds `docs/compute/ladder-runbook.md`, providing a step-by-step, reproducible runbook for the 20M, 29M, and ~96M geometric training ladder and its chat evaluation pipelines as executed on the Runpod 2×RTX 4090 pod and local M1 laptop.

References #820.

## Coverage
1. **Data preparation:**
   - TinyStories and TinyDialogues UORT store generation via `prepare-text-corpus`.
   - chat-v0 literal-role protocol-2 store and launch hash verification (`b5d3bf977a524d9e`).
   - chat-v1 preparation via `prepare-chat-parquet` (`--features parquet-input`, PR #1660) with skip offsets `13900` (SmolTalk) and `10200` (UltraChat), explaining how these offsets keep new training tokens strictly disjoint from the held-out evaluation panel.
2. **Base geometric training per rung:**
   - Common configuration: 60/15/25 mixture of TinyStories/TinyDialogues/chat-v0, validation on TinyStories, `context=384`, `batch=16`, `read=l2`, `rotation=true`, `device=cuda`.
   - 20M rung: `width=512`, `heads=8`, `layers=8` (`rrarrarr`), 300M tokens (48,828 steps), learning rate checks (0.002, 0.001, 0.003, 0.0005) from `run-20m.sh`, `run-lr.sh`, `run-lr2.sh`.
   - 29M rung: `width=576`, `heads=8`, `layers=10` (`rrarrarrar`), 433.8M tokens (70,609 steps; 15 tokens/param), learning rate evaluations (0.001 vs 0.0005) from `run-29m.sh`.
   - ~96M rung: multi-GPU data parallelism via `data_parallel=2` across GPU 0 and GPU 1 (PR #1660).
3. **Chat fine-tuning (`dialogue-train`):**
   - Recipe A (M-world only, `/root/data/ft/parar`) vs Recipe B (Mixed, `/root/data/ft/mixed`) from `run-ft.sh`.
   - Critical parameter constraint highlighted: `init` MUST point to `<run>/model`, not the run root.
4. **D19 grounded conversation session:**
   - Multi-turn factual evaluation via `m-world session` across 1,075 turns.
   - `log_recall=sieve` (exact prime-atom sieve) vs `log_recall=off`.
5. **Chat-grade panel evaluation:**
   - 232-request evaluation panel (`everyday-32.json` + `heldout-200-a.json` + `heldout-200-b.json`).
   - Offline judge `qwen2.5:7b` via local Ollama.
   - Derangement control (evaluating response against next request's conversation).
   - Paired exact McNemar significance test (`mcnemar_exact_p < 0.05`).
6. **Operational rules & caveats:**
   - Verified single-cargo execution rule, local laptop 8-thread ceiling (`RAYON_NUM_THREADS`), exclusive directory creation (`report_output::claim`), and unverified markers where exact pod wrapper scripts are unconfirmed locally.
   - Results are referenced to comments on #820 and #1552 rather than restated or invented.

## Validation
- `python3 scripts/check_claim_wording.py` passed (exit 0).
- Docs-only change: no crates or Rust code modified.

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### review 5403436732 — 2026-10-03T23:21:49Z — Casey-allard — COMMENTED

**Exact-head review at `1f9476b82c14b0b649711b2a26b24cf19076449e` (Claude lab, independent of the Antigravity author).** Docs only: `docs/compute/ladder-runbook.md`.

All 11 items from the `9492679c` review are fixed and checked against the scripts:
- the TinyDialogues input file;
- the actual chat-v1 command, with skips 13900/10200 and its output counts;
- the learning-rate checks described against the 0.002 default run;
- the full ~96M recipe (1024/16/14, batch 32 with `data_parallel=2`, lr 0.0004, mix 0.35/0.05/0.10/0.50, `74d8189d`, `RAYON_NUM_THREADS=16`);
- the fine-tune variables and `run-ft29.sh`;
- the pointer and `full_prefix` definitions;
- the pod sessions (sieve and off);
- 1,075 scored turns over 300 conversations;
- ~96M in the D11-unverified list.

Results cite #820 issuecomment-5974333259. Claim-wording passes. **APPROVE**; queueing with `--match-head-commit`.


---
_Comment/review ids in this file: 5403436732_

