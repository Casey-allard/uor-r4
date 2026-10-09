# Integration backlog — 2026-09-30 read-only snapshot

The initial snapshot had **10 open PRs**, **795 local branches**, **977 remote-tracking refs** and **975 live remote branches**. It covers **1,784 total Git refs**, including 12 tag/stash/application/review refs. All live remote branches had tracking refs. One stale queue tracking ref was retained; nothing was pruned. Forty worktrees were inspected for tracked/untracked status. `branch-inventory.json` maps every branch to PR facts, ancestry, worktree state and next action; `all-refs-final.tsv` records changes during this audit.

The queue moved during reconnaissance: **#1521, #1490, #1505 and #1506 merged**, leaving six open PRs at the final query. Current main is 0dcdb6e970fc94d2c47f5ea879b1e7a2db620abd. This does **not** mean the previous review requests were satisfied.

## Immediate corrective delivery

1. **#1506 merged with safety blockers still present.** At its delivered source, `uor-r4-lut/src/stack.rs:163` lacks the `transport_snap` refusal in `from_artifact`. `d4-float-reference.rs:396` exports the loaded model with `None, None`, and `check_export_transport` is absent. Preserve the previous independent review; deliver a narrow refusal/export-guard follow-up before additional snap serving qualification. The forced-root/cascade, pinned-evidence and cost-scope gaps also need explicit disposition. See `merged-1506-correction-required.json`.
2. **#1505 merged at unchanged reviewed head3fc3d592 with all four required wording fixes unresolved.** The document still says the joint key is not linearly exposed, amendments preceded all runs, heads trained to convergence, and keys-9 was uncontended. The JSON retains corresponding contradictions. Correct doc/JSON/PR-facing evidence through a narrow protected follow-up without rerunning models. Exact locations and main blob identities are in `merged-1505-correction-required.json`.

## Integration order for the six remaining PRs

- **#1522, Claude:** main now contains D13/D14. Re-review its updated head (final API in `pr-1522-final.json`), preserve D15 owner direction and obtain the required two non-author D16 votes. Initial snapshot showed one Antigravity vote. No fixed lab director is required.
- **#1527, Antigravity:** #1490 is now merged; align current main, retain separate B/C result scopes and pending provenance/quality limitations. Its codec/export/parity-test blobs were identical to latest #1490, so avoid duplicate codec changes. Independent evidence review determines necessary fresh-root work.
- **#1519, Antigravity:** isolate/reconcile B3 changes with main and #1527. It still embeds older D4 codec/export/test files and B/C claims; source differences were confirmed in `d4-stacked-source-divergence.json`. Preserve measurements, resolve the council's disputed instrument/canonical-E8P scope, and do not resurrect stale heuristics or imply a geometry-wide negative.
- **#1526, Antigravity:** independent source review and exact-head CPU/Metal forward/backward checks; reconcile `geometric_stack.rs` with delivered #1506 and upcoming A1. Ten tests are author-reported, not independently executed by this audit.
- **#1528, Antigravity:** draft/source-only integer selector and blocked4 kernel; synchronize canonical selector semantics, integrate #1506 follow-up, then execute allocation/parity/edge checks and declared serving opcode audit under one build reservation.
- **#1518, Codex:** shared Track B model remains draft/unqualified. Use the proven fresh-proc-macro source-build recovery path, integrate one selector/one host, compile focused paths and execute the fixed45-position parity gate under its bounded work card before fitting. Preserved drafts and earlier interrupted attempts are not quality evidence.

## Urgent unpublished implementation

**OpenCode B0 and Claude A1 must meet at an actual interface.** `lab/opencode/b0-flock` at5b7d1685 has three unmatched source commits implementing shared flock and model-source transport. `lab/claude/a1-pointer-fixes` atb806fb2d imports `crate::flock`, but neither its flock module nor module export is present. It calls `flock::top_k_select`; current B0 has no such function. #1528's `top_k_select_integer` is a separate integer API. Publish/review the canonical float selector including top-k-only selection, then integrate the A1 descendants before any model experiment.

`lab/claude/a1-mworld-cells` at76c8dac2 and pointer-fixes share basecb32e73c. Preserve the former's three added commits for cells, untrained rule baselines, sealed English probe and teacher-forced answer scores, and combine with the latter's pointer fixes in one reviewed integration lane. `a1-retrieval`, `a1-flock-pointer`, and `m-world-v2` are ancestors/staging for this work, not three additional products. No PR exists for these heads in the captured1,090-PR history.

`lab/claude/integration` retains two unmerged Stage2 source commits (user-turn flags, on-policy gated registers, store-aware development/greedy replies and fitting). Assign a bounded source-disposition task: carry useful interfaces into the active A1/A3 path, keep unscheduled fitting parked. `codex/lab-runner` is the original runner already imported and extended by #1521; compare residual differences before marking superseded, not a second runner to merge.

## Retained history and preservation

Initial branch classifications:1,554 exact heads map to merged PRs;35 are ancestors of main;43 map to closed unmerged PRs;61 differ from/stale relative to their historical PR heads;53 unmapped refs represent40 distinct heads;20 refs represent the10 open PRs. Non-merge patch comparison covered121 distinct retained heads. A `git cherry` plus is **not proof of missing source** after multi-commit squash merging. PR-head ancestry is likewise not proof every original file survived.

Notable preserved source/evidence needing explicit disposition:
- Cloud handoff16c77c97 is ancestor of merged#1462, yet132 of422 original paths are absent from current main (290 unchanged). Preserve/index the complete original branch; do not bulk-merge sandbox code.
- `codex/geometric-lm-goal` preserves f99f42fb:17 of23 changed research/adversarial paths absent from initial main.
- Canonical-address-routing d22e2b50 and director integration45e3871c retain29 of40 historical changed paths absent from initial main, plus untracked R1d result JSON/Markdown. Keep their negative/throughput evidence and review any active reuse.
- `codex/dialogue-code-choice-isolated-20260927` local and remote heads differ but are both in merged#1433 ancestry;17 of25 changed paths match main. Inspect the other8 and retain the231-update checkpoint record.
- #1479 review snapshots, old codec-cleanup/input-hardening branches and pre-main H4/kernel snapshots have explicit actions in the JSON, with path/blob coverage rather than assumptions based on names.

Dirty worktrees include the primary checkout (1,359 status rows), Track B recovery logs, S1/S4 integration scripts, archived canonical-address evidence, archived S1 follow-up source (nine modified source/audit files), archived S4 transport source (three modified files), and the D4 Metal worktree (Cargo/lib changes plus untracked kernel source). **None is cleared for cleanup.** Ignored artifacts, processes and deletion eligibility were outside this audit.

## Evidence and limits

Files: `branch-inventory.json`, `integration-order.json`, `retained-branch-patch-analysis.json`, `retained-head-pr-containment.json`, `recent-unpublished-path-coverage.json`, `worktree-inventory.json`, `open-pr-file-overlap.json`, full paginated PR history, exact PR comments/reviews/files and final merge correction receipts.

No source edits, branch changes, issues/comments, merges, deletions, Cargo, tests or model compute were performed. Remote refs were refreshed twice with `git fetch --no-prune`. This is an integration inventory, not full correctness review; all actual delivery must refresh exact head/base, account for shared-file changes and meet independent review/check receipts.

`unmapped-retention-summary.json` gives all 53 unmapped refs an explicit provisional retention reason, review steward and reactivation condition. It calls none obsolete. The full branch inventory is also available as `branch-inventory.json.gz` for the operational branch rather than main.
