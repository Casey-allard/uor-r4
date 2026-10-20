# #1020 — A1Q-H5: freeze the 13M capacity lifecycle

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-31T15:08:20Z
- **Updated:** 2026-10-09T14:17:10Z
- **Closed:** 2026-08-31T15:09:34Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1020
- **Merged:** 2026-08-31T15:09:34Z
- **Merge commit:** 81380d3fa574f429b9b77814a3c5a8afb78bce4f
- **Base / Head:** main / UOR-Foundation:issue-1019-13m-capacity
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Outcome

Implements the one frozen #1019 capacity rung under #973: a fresh 12-layer, 13,130,784-parameter ordinary causal-softmax R4/Spin model with an executable create-once population, smoke/parity, hardware-admission, training, reveal, generation, replay, and finalization lifecycle.

This preserves the established attention mechanism and changes only depth from six to twelve layers. All execution evidence remains truthfully NOT_RUN until the merged lifecycle is exercised.

## Evidence boundaries

- Exact 16,800-step / 275,251,200-token contract and fresh post-#1017 populations.
- Prior published prompt stories excluded from the actual stores.
- Signed checkpoint/resume and monotone eight-hour wall ledger.
- MPS-first 200-step probe includes full dev evaluation, regular and worst-case best-checkpoint costs, reload, 1.25 safety factor, and 80% memory ceiling.
- CUDA fallback requires and binds a fully validated failed-MPS result; no CPU or mixed-precision fallback.
- Python/Rust all-12-layer parity, strict causal/R4/output audits, PASS-only irreversible generation, and deterministic replay/finalization paths.
- Living project documentation now names #1019 as the sole current rung and keeps geometric softmax replacement parked until a positive baseline.

## Focused verification

- Python trainer integrity: 73 passed.
- Rust local-generation qualification: 8 passed.
- Rust CLI parser: 1 passed.
- cargo fmt: passed.
- Scoped cargo clippy for library and r4 binary: passed with warnings denied.
- Claim-wording, JSON syntax, Python compile, and diff checks: passed.
- Independent post-fix audit: no remaining concrete blocker.

No corpus build, accelerator smoke, hardware probe, full training, reveal, or generation was run in this PR.

References #1019
References #973
References #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

