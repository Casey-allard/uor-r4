# #1155 — Learn geometric selection between derived totals

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-06T21:07:05Z
- **Updated:** 2026-10-09T13:18:01Z
- **Closed:** 2026-09-06T21:31:42Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1155
- **Merged:** 2026-09-06T21:31:42Z
- **Merge commit:** bfb04325f8e514daf6d8ad6247e36f103e3a6c0c
- **Base / Head:** main / UOR-Foundation:codex/typed-intermediate-roles
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

After computing an original total and an updated total, the previous model gets only 3/12 final role requests correct. This change learns query-conditioned geometric selection using exact derivation depth, preserves Copy identity during candidate admission, and isolates query metadata at the explicit end_response boundary. Numeric values remain exact payloads; existing Copy/Add execute after selection. The optional component preserves parent bb79456b and introduces no serving matrix multiplication, dense transformer or LLM/provider correction.

Selected artifact 43c54db3 generates 12/12 new complete four-turn operand/refresh transfers, versus 2/12 for the matched exact-code fit (eight comparator failures occur in required refresh generation). Both refresh directions pass. Example: 20+3 ->23, +7 ->30, refresh updated ->30, then original+2 ->25. Earlier 9/12 and 8/12 failures remain exposed negatives, not relabeled held-out successes. The removal control has 0/12 full operator/provenance successes but 2/12 matching texts through recomputation; no zero-answer-accuracy claim.

Validation: 42/42 generated construction and 12/12 development; preservation 48/48 dependent,62/62 earlier,24/24 prior-transfer,28/28 exposed-name,28/28 long-context,5/5 persistent and6/6 preceding numeric transfers. Ten focused tests pass. Actual four-turn routing/commit is allocation-free, checkpoint round-trip preserves the query boundary, and absent/future boundaries are rejected. Kernel source, formatting, architecture and wording checks pass. CLI revision returns Zurich; four unchanged generated Rust functions pass 12 semantic assertions. Broad release QA was not run.

The implementation record/evidence retain every fit, failed transfer, control, artifact/source/report identity and cumulative resource receipt. Model work 283.472s; monitored engineering 1397.998s including failed build and revisions. Cumulative model 2316.511/2370s, 53.489s remaining. Standing authorization supports the recorded extensions; no storage increase, deletions or paid external compute.

Update current-state, README, research, lifecycle and roadmap to distinguish completed work from the next equal-depth operand/owner-provenance selection. This is bounded causal-depth behavior, not general named-role binding, new Rust reasoning, general prose/syntax, frontier capability or whole-model speedup. Refs #1139, #1140; both broader issues remain open.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

