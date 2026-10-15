# #232 — certify spends ~2.5 of 3.2 hours on an R4G1 evaluation that returns an all-zero row

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @maurathat
- **Created:** 2026-07-27T03:26:37Z
- **Updated:** 2026-07-28T03:16:09Z
- **Closed:** 2026-07-28T03:11:43Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/232
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Wall time for `certify` on the legacy path was approximately 3.2 hours, of which roughly 2.5 hours was a single-threaded R4G1 graph-runtime evaluation (`predict_distribution` / `check_node_emits` per process sampling). It terminates in:

```
C R4G1 (graph path): top1 0.0% | agreement 0.0% | WB 0.0000 bits/token
```

and

```
- entity reactivation accuracy (baseline: N/A without trained graph)
```

No trained graph was present. The evaluation appears to run to completion regardless and produce a vacuous row.

## Proposal

Guard the R4G1 evaluation on the presence of a trained graph, and emit a skipped-not-zero marker when absent. An all-zero row is also actively misleading — it reads as a measured failure rather than an absent measurement.

## Impact

Roughly a 4× reduction in `certify` wall time for anyone iterating on the legacy path.

## Environment

Commit `0ffb71b94d78ff722af089f1337a08fce42dccff` (HEAD of main, 2026-07-26). rustc 1.95.0, `--release`. macOS 26.5.1, arm64. Fresh clone; corpus built with `gen 300 150000`: 150,000 tokens, 754 stories, 30,036 held out.


## Comments (2)

### comment 5099309944 — 2026-07-28T02:39:44Z — Casey-allard

Taking this one (assigned). Implementation plan, after reading the actual code path — two findings that sharpen the report:

1. The `WB 0.0000 bits/token` in the C row is a **hardcoded string literal** (`certify.rs:307`), not a computation — it prints a fake zero even for a trained graph.
2. `node_scores` (`vec![ScoreQ::MIN; node_count]`) is allocated **inside** the ~30k-iteration eval loop (`certify.rs:292-293`) — that plus the single-threaded `predict_distribution` walk is the wall-clock driver. (The `entity reactivation` line comes from the separate, trivially-fast `certify_long_context()`, not this section.)

**Plan** (branch `issue-232-guard-r4g1-eval`):

- Extract a pure, unit-testable readiness predicate: structural check (emission count > 0) + a deterministic 64-position probe; if the graph can't emit, print `C R4G1 (graph path): SKIPPED — no trained graph … No measurement recorded.` and skip the full eval. Skipped-not-zero, per the issue's proposal.
- Replace the hardcoded WB literal with `WB n/a (not computed on the graph path)`.
- Hoist the `node_scores` allocation out of the loop (fill-reset reuse) — speeds up the legitimate trained-graph case too. No parallelism in this PR (keeps determinism review trivial).
- Out of scope: pointing legacy `certify` at `graph/score.r4g1` — that changes what certify certifies; follow-up issue if wanted.

**Tests**: new `tests/r4g1_eval_readiness.rs` in the certify crate — untrained graph → `NoEmissions`/`ProbeAllSilent`, minimal emitting graph → `Ready` (built the same way `r4g1_runtime_test.rs` builds tiny graphs). Certifier-side only; no runtime-crate changes, so P-4 kernel scan, allocation census, and no_std surface are untouched.

**Gates before PR**: `cargo test --workspace --offline` · clippy `-D warnings` · `fmt --check` · no_std ladder · κ note: certifier-eval-only change ⇒ artifact κs must be byte-identical.

DoD: no-trained-graph certify completes without the ~2.5h vacuous eval (expect ~3.2h → ~0.7h per the report), trained path unchanged-but-faster, marker unambiguous, all checks green.


### comment 5099520574 — 2026-07-28T03:16:09Z — Casey-allard

**DoD evidence** (closed by merge commit `36b91fc`, PR #236):

- Untrained/unscored graphs now skip the ~2.5h R4G1 evaluation via a deterministic 64-position readiness probe, printing an explicit `SKIPPED — … no measurement recorded` marker (skipped-not-zero, as proposed).
- The `WB 0.0000 bits/token` literal (it was hardcoded, never computed) now prints `WB n/a (not computed on the graph path)`.
- `node_scores` allocation hoisted out of the ~30k-iteration loop (one reusable buffer).
- 3 new tests (`crates/uor-r4-graph-certify/tests/r4g1_eval_readiness.rs`) cover Ready / degenerate / empty-probe classification; all four local gates green on macOS arm64 plus full CI (six required checks) on the merge.
- Certifier-side only: no runtime-crate changes; artifact bytes and κs unchanged.

Follow-up filed while reading this code path: #237 (the C-section feeds rotation offsets as context lags — the row has never measured coherent context; the probe correctly classifies that shape as degenerate, but the underlying defect deserves its own fix).



---
_Comment/review ids in this file: 5099309944, 5099520574_

