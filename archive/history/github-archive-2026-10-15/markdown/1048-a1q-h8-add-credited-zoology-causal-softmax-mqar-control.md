# #1048 — A1Q-H8: add credited Zoology causal-softmax MQAR control

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-02T08:36:48Z
- **Updated:** 2026-10-09T14:17:12Z
- **Closed:** 2026-09-02T08:38:02Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1048
- **Merged:** 2026-09-02T08:38:02Z
- **Merge commit:** 677fb133b6d6a01fe384450b66beabbbd1b8f9a5
- **Base / Head:** main / UOR-Foundation:issue-1047-zoology-mqar-control
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Closes #1047

## Outcome

Ports and credits the released ICLR24 HazyResearch/Zoology one-head width-64 causal-softmax MQAR cell and loader as the frozen ordinary-attention integration control. Source goldens, deterministic mechanics, causal/query-only parity, and the disposable 32-row overfit pass.

The scientific C1/C2 run is honestly `NOT_RUN_PREFLIGHT`: the fastest frozen plan used all eight host CPU cores and projected `959.212581 s` after the required `1.25` safety factor, above the frozen 900-second wall. No model artifact or scientific attention verdict was produced.

## Bound evidence

- implementation tree: `blake3:c848c05ae53bc3adc0a8f7099ceed43657b6348e4e00fe3aaef5cf1368cc38de`
- preparation: `blake3:560dd6e9abbabbeccbb32b2dbec6b815b5f7842bbb30c387af79177bd32a98f8`
- preflight: `blake3:78158700e632d303bf674ed544f997a0e14eb89947470f5032e6acc75c830c9b`
- result: `blake3:b453abccc6ae0db9cc186c791aba268555dc0e75fe687c994e940254b0ac9ef6`
- C0 overfit: `128/128` after 87 updates
- all forbidden/read-ledger counters: zero

## Verification

- 23 focused Zoology-control tests pass
- Python module compilation passes
- `cargo fmt --check` passes
- repository claim-wording gate passes
- `git diff --check` passes

## Next action

Preserve the exact scientific cell, data identities, batch, optimizer, gates, and CPU-only scope. Continue on a fresh issue that widens only the execution wall enough to admit the already measured all-core plan. This is a resource-policy continuation, not same-issue tuning.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

