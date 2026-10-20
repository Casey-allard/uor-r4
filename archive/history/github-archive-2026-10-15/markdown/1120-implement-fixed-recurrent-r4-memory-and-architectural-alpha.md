# #1120 — Implement fixed recurrent R4 memory and architectural-alpha track

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-04T17:34:52Z
- **Updated:** 2026-10-09T13:17:35Z
- **Closed:** 2026-09-04T17:36:13Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1120
- **Merged:** 2026-09-04T17:36:13Z
- **Merge commit:** db8682fb762aadc5e87c0afb0d4f1fd01cfc4495
- **Base / Head:** main / UOR-Foundation:codex/issue-973-fixed-recurrent-r4
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Problem and result

The accepted #1119 path still stored one exact K/V record per prefix token, and the repository described several research programmes without one authoritative build order. This change adds a fixed-size recurrent R4/H4 memory path and makes an eight-stage architectural-alpha track canonical across the roadmap and execution policy.

`R4FixedRecurrentCausalKVBindingV1` keeps eight exact live K/V records and four chronological binary-age H4 summary banks. It reads before committing the current token, fixes persistent K/V state at 2,304 f32 values (9,216 bytes), limits each attention read to 13 sources, and reuses the accepted learned tokenizer, decoder, Q/K/V/O, softmax, RMSNorm, SwiGLU, and vocabulary head without a new fit.

The project track is now:

1. fixed recurrent geometric memory
2. sparse geometric attention
3. nonlinear geometric block
4. scale, data, and instruction behavior
5. retrieval and tools
6. representative product alpha
7. Rust/table lowering and optimization
8. release proof, evidence, and QA

SpiralCore, HELM, W33, NEMESIS, UOR, and H4/zeta are recorded as on-demand donor reservoirs for concrete implementation questions. Broad proof, bookkeeping, indexing, publication, and QA work is deferred until the release candidate unless an active implementation decision needs it.

# Direct measured comparison

The frozen full prompts from #1119 were replayed once with seed 9738 and a 16-token budget through the exact comparator and the unfitted recurrent path:

- `A purple turtle found a clock in the garden`: both arms shared 12 generated tokens, then diverged; recurrent execution recorded 17 evictions, 66 summary reads, 15 merges, and 13 peak sources.
- `Albert Einstein was born in`: both arms shared 12 generated tokens, then diverged; recurrent execution recorded 20 evictions, 80 summary reads, 18 merges, and 13 peak sources.
- Both recurrent runs held state at 2,304 f32 K/V values versus the 23,040-value 120-token exact-cache basis, with zero teacher, provider, future, or forbidden reads.

This establishes mechanism execution and a constant serialized f32 K/V bound. The divergent continuations are lossy behavior; they do not establish language quality, long-context retention, geometric advantage, architectural alpha, or table-native serving. The trained RoPE limit remains 120.

# Review and validation

Independent code review caught and resolved a no-summary exactness defect and unintended self-transport in summary merging. Independent source/artifact review also caught an initial prompt mismatch; those exploratory outputs were preserved separately, and the canonical comparison was rerun with the frozen full prompts. A final independent reconciliation found no remaining substantive defect.

Focused checks:

- `python3 scripts/check_agent_execution_policy.py`
- `python3 scripts/check_claim_wording.py`
- `python3 -m py_compile tools/r4-softmax-trainer/src/r4_softmax_trainer/fixed_recurrent_kv_binding.py tools/r4-softmax-trainer/src/r4_softmax_trainer/fixed_recurrent_r4_language_generation.py tools/r4-softmax-trainer/src/r4_softmax_trainer/cli.py scripts/check_agent_execution_policy.py`
- `PYTHONPATH=tools/r4-softmax-trainer/src tools/r4-softmax-trainer/.venv/bin/python -m unittest tools/r4-softmax-trainer/tests/test_fixed_recurrent_kv_binding.py`
- `git diff --check`

References #973.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

