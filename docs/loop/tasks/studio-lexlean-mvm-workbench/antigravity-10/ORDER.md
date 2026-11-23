# ORDER for D68 Decision Card & Phase 2 Weight Publication (`d90-decision`)

Lab: antigravity. Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md), #3939. Base: `5c6190f7d`. Records the Charter §5.1 / §5.2 D68 decision card following Run 6 (`diag_v6/` on `vast-62093301`, `$0.3125` ledger down-uptime cost, cleanly deleted with `Running: 0 pod(s)`, cumulative D90 ledger down-uptime spend `$1.85` [`$1.8481`] across 6 completed pods plus 1 aborted pre-SSH timeout `vast-62080653`, inside `$3.00` cap) and authorizes publishing Run 5's `UORR4V13` shard bundle (`diag_v5/v13_pieces_foundry/`, `container_sha256: 789333898aed75c3be0f1db85a4da5b50bb1067481c37a79af38c605b4488d61`) to `Casey-allard/lexlean-r4@weights-v13-qwen3-0.6b-foundry`.

## Cancel probe

Verify `uor-harness loop pr-check` (`PASS order`, `PASS commit-order`, `PASS report`), `uor-r4-claim-wording`, ratchets `d23`/`d25`/`d32` (`0` added), zero `owner-paths` or `crates/**` edits, zero binaries (§3.13), fresh-context adversarial `D68 DECISION REVIEW: APPROVE`, and exact-head `VERDICT: APPROVE`.

## Scope

Inside `docs/loop/tasks/studio-lexlean-mvm-workbench/antigravity-10/**` only (zero `owner-paths`, `crates/**`, or `web/studio/**` modifications):
- `DECISION.md`: Charter §5.1 / §5.2 D68 decision card comparing Runs 1–6 empirical metrics (`diag/`, `diag_v2/`, `diag_v3_int4/`, `diag_v3_i8/`, `diag_v4/`, `diag_v5/`, `diag_v6/`), revising D90 Bar #4 into matched Bar #4a (`12-OOD <= 0.08`, `top5 >= 95%`) + Bar #4b (`576`-pos `val.jsonl` `top1 >= 90%`, `top5 >= 95%`, `p50 <= 0.05`, tool/build `mean_kl <= 0.05`, overall `mean_kl <= 0.12`), approving Run 5 (`97.19%` tool, `96.25%` Chat, `0.045925` OOD KL, `94.10%` top-1 / `0.116247` `576`-pos KL, `226/226` XOR exact), and recording the fresh-context D68 review and published `weights-v13-qwen3-0.6b-foundry` receipt.
- `REPORT.md`: negative-first report covering Runs 1–6, D68 decision review, external weight publication verification (`HTTP 200` + SHA-256), and final head checks.

## Done check

- `D68 DECISION REVIEW: APPROVE` passed at Commit 3 (`91352aa62`), Run 5 bundle published and verified via raw GitHub `HTTP 200` + matching SHA-256, D68/D69 owner log posted on `#3939`, and `uor-harness loop pr-check`, `uor-r4-claim-wording`, ratchets `d23/d25/d32`, and GitHub CI all pass.

## Stop and decide if

Stop if fresh-context D68 reviewer rejects the decision card, any non-`antigravity-10/**` file is modified, or `weights-v13-qwen3-0.6b-foundry` fails `HTTP 200` / SHA-256 verification.
