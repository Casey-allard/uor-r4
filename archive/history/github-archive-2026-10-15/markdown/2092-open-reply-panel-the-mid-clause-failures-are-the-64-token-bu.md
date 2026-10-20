# #2092 — Open reply panel: the mid-clause failures are the 64-token budget (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T21:20:11Z
- **Updated:** 2026-10-09T21:42:08Z
- **Closed:** 2026-10-09T21:39:19Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2092
- **Merged:** 2026-10-09T21:39:19Z
- **Merge commit:** a3c34bee066f776a089110ade933f33191b7d621
- **Base / Head:** main / UOR-Foundation:reply-token-budget
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## The question, and the answer

The previous round's largest text marker on the open reply panel was `no_terminal` — **125 of 189 failures**. It could not say whether those are the `max_new_tokens=64` cap or voluntary stops mid-sentence, and that decides what the panel measures.

**Answer: CAP-TRUNCATION. Split 85 / 34 / 6.**

| of the 125 mid-clause failures (29M) | count | share |
|---|---:|---:|
| **≥ 64 tokens — at the cap** | **85** | **68.0%** |
| exactly 63 — one token under | 34 | 27.2% |
| ≤ 62 — genuine voluntary stop mid-sentence | 6 | 4.8% |

**119 of 125 (95.2%) end within one token of the 64-token budget.** Coverage of the 189 failures: **85 (45.0%) at the cap, 124 (65.6%) within one token, 125 (66.1%) mid-clause in total, and 64 (33.9%) not budget-limited at all** — 62 of those end with terminal punctuation, i.e. completed replies graded wrong.

## The tokenizer was found, and the instrument is validated

`d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89` at
`~/uor-r4-local/workspace/uor-r4-lab/claude-t4-1433-resume/bundle-learned-1/tokenizer.json`. Four independent confirmations:

1. the file's sha256 equals the sealed report's own `tokenizer_sha256`;
2. the sealed report root's `attempt.json` (recovered from `ladder.tar` for this round) names that exact bundle path under the now-deleted `inputs/` tree;
3. the loader accepts it as the same byte-level BPE (blake3 `3f42bcfc…`);
4. **`crates/uor-r4-tokenizer/examples/token-counts.rs`, built on the crate's own engine, reproduces the `chat-grade check` worst cases already recorded on main for `context=384` — 336 (`conv-v3-mem-16`) and 347 (`conv-v4-mem-23`) — exactly.**

So the counts are the frozen instrument's counts, not a re-implementation's.

## Why the count decides it

`stack_dialogue::greedy_reply_with` runs at most 64 steps and leaves early only on EOS (`Reply::stop_with`) or a short repeated cycle (`cycle_repeats=3`); `reply_panel` stores `decode(ids without EOS)`. A reply whose own text already needs 64 canonical tokens cannot have come from a short generation. The reconstruction is not exact to the token — the stored text is `trim()`ed by `panel_conversations`, and a model token can cross a GPT-2 pre-token boundary — so counts are read in a band, calibrated by the five replies at 65–66 tokens, which can only be cap runs.

## The well-posed/ill-posed asymmetry is the same budget

| tier | failures | at the cap | share |
|---|---:|---:|---:|
| `K-clean` | 115 | 73 | **63.5%** |
| `K-ill-posed` | 51 | 14 | **27.5%** |
| `everyday` | 23 | 0 | 0% |

Over all rows, 82 of 133 `K-clean` rows (61.7%) reach 64 tokens against 18 of 67 `K-ill-posed` (26.9%) and 0 of 32 `everyday`. A well-posed request makes the model write until the budget cuts it off; an unanswerable one makes it stop early — which is the `fluent_only` concentration in `K-ill-posed`. **The 86.5% versus 76.1% failure rates are a budget effect, not a knowledge effect.** Acceptance is cap-sensitive too: 13/100 at the cap (13.0%) against 30/132 below it (22.7%).

**Second artifact, same shape:** 96M `chat-100m-C` (46/232) — 86 of 117 mid-clause failures at the cap (73.5%), one below it, `K-clean` 65.5% against `K-ill-posed` 17.4%.

## What it does not say

- It does **not** say the cut replies would have been *correct* with more room. Nothing here regenerates a reply.
- It explains 45.0% of the 189 and leaves 104 diffuse, so **the no-Phase-2 decision stands unchanged**: the budget is a decoder setting, not a mechanism.
- It does mean **this panel's headline number is partly a decoding-budget number**, so any candidate scored on it at `max_new_tokens=64` must be compared at that budget.

## What follows

One bounded, cheap, pre-registerable measurement the first pass believed unavailable: **the same sealed panel at `max_new_tokens` 96 or 128, same two artifacts, same grader digest, the cap the only delta, declared before it runs** — to decide whether the budget-cut replies become correct or only longer. Worth doing before any corpus/knowledge spend, since the 43→46/232 plateau may be partly a plateau at 64 tokens. And still first: criterion 1 on this panel is judge-only and needs deterministic row checks or a stated tolerance before any candidate number on it is accepted.

## Delivery

Amendment to the existing record (`docs/labs/open-reply-panel-2026-10-09/README.md`, new **Result 4** plus the corrected limitation — the first pass's claim that the tokenizer was unavailable was wrong and is corrected in place), newest-first `docs/integration/current-state.md` entry ending in **Next:**, instrument `scripts/open_reply_panel_token_counts.py` and `crates/uor-r4-tokenizer/examples/token-counts.rs`, evidence bundle `icloud:UOR-R4/results/deepseek/reply-panel-token-budget-2026-10-09.tar` (540,672 B, md5 `ff006fa0f86cdbfa671e8491b108ea06`). `cargo fmt --check` clean on the crate; the 336/347 instrument check re-run after formatting. CPU only — no model, no generation, no grading, no pod, zero spend. **References #2029.**


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

