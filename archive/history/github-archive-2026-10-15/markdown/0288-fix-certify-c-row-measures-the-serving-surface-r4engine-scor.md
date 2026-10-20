# #288 — fix(certify): C row measures the serving surface — R4Engine + score.r4g1 + D4 policy (#280)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-29T20:47:14Z
- **Updated:** 2026-07-29T23:04:20Z
- **Closed:** 2026-07-29T21:27:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/288
- **Merged:** 2026-07-29T21:27:14Z
- **Merge commit:** 89d74d9c915384b464be185e6a8b19e1ea4e6aea
- **Base / Head:** main / UOR-Foundation:issue-280-honest-c-row
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes nothing yet — #280 stays open for the scaffold-fate record and follow-ups; this PR delivers its reframed DoD items 1 and 3.

## What

- **New `uor_r4_api::serving_eval`**: held-out evaluation of the exact deployed configuration — `R4Engine` over a compiled bundle's `score.r4g1` with the D4 status policy, fed token windows like the server. Reports served/abstained rates **with per-status breakdowns (#234 item 3)**, top1/agreement on served positions. Deterministic ~1000-position stride subsample + wall-clock budgets (#282/#279 discipline, same env vars: `R4_CERTIFY_R4G1_BUDGET_SECS` / `R4_CERTIFY_R4G1_EVAL_BUDGET_SECS`).
- **#232 probe extension (#280 DoD item 3)**: readiness probe now includes an accuracy spot-check — a probe that serves predictions with zero hits against corpus continuation *or* teacher argmax skips the full eval with an explicit recorded reason. Scored + non-constant demonstrably did not imply functional.
- **Scaffold demoted (#280 decision, approved by Casey)**: the `convert_r4g1`/`R4G1Runtime` path is documented as a container-format demo (it was never a functional prediction path — no per-node emissions, no kind=2 walks, stride-8 EXCT decode over a variable-stride container). The 0.0% row stays recorded on #280 as the reason it left the measurement matrix. `r4g1_readiness` (module + test) retires with the row it guarded.
- `r4 certify` drives the new row after the A/B rows: first loadable bundle under `.uor-models/compiled` (`R4_CERTIFY_SERVING_BUNDLE` overrides); unusable bundles and absent measurements are explicit printed skips, never silent.

## Measured (smoke, SmolLM2-135M-Instruct-7e27bd9f9532, Casey's Mac)

```
n=1021 (stride subsample), ~1s total (vs 13.5h+ in-phase on the scaffold)
served 1021/1021 (100%) — every position resolved ExactContext, 0 graph, 0 novel, 0 widened
on served: top1 5.6% | agreement 12.6% | probe: 64/64 served, 14 hits (functional check passes)
```

Two follow-up findings, posted to the issues after merge: (1) the serving surface never abstains and resolves **everything** by exact-context — #234's blended-number concern reproduced on the deployed path; (2) serving agreement (12.6%) sits far below the score-time Rule 1+2 report — consistent with the window-derived vs observation-signature family gap, which belongs with #268's confounder hunt.

## Consequences for open issues

- #278 (emit index for the scaffold): overtaken — the indexed path would still have no emission structure to read. Close on merge with a pointer here.
- #277 (VP-tree): item 2 rescopes to the serving engine's lookup paths; items 1 and 3 stand.

## Verification

Local: `cargo check` + `cargo clippy --workspace --all-targets --all-features -- -D warnings` + `cargo fmt --check` clean on the warm shared target; serving row smoke above. Full suite runs in CI per the 2026-07-29 process amendment (no redundant local reruns).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_018cWTa3B7xnsk6wygnfgL7o

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

