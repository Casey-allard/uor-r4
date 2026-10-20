# #2222 — Starling read with a training-only softmax surrogate: content selection learned, recall lifts unevenly (reading 3; owner decision) (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T00:44:56Z
- **Updated:** 2026-10-12T00:46:12Z
- **Closed:** 2026-10-12T00:46:05Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2222
- **Merged:** 2026-10-12T00:46:05Z
- **Merge commit:** 42dcb38280b76165d71f26ea8eeef054b27cf491
- **Base / Head:** main / UOR-Foundation:claude/starling-surrogate
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Line:** starling read · count 3/3 (pivot; owner decision) · headline "softmax at runtime: yes" → yes (0.886838 BPB served, unchanged)

**Result: reading 3 by the pre-registered bars.** Beyond the bars, the training-only softmax surrogate **teaches content selection**. When the source is kept, it is ranked first 0.55–0.78 of the time, against 0.03–0.12 without the surrogate. Recall lifts off chance unevenly: 0.044, 0.288, 0.091.
- **Pre-registration:** #2032 [6115450007](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6115450007).
- **Fitness:** FIT round 2; round 1 caught a value-gradient leak, fixed before compute.

| Arm (mqar, 3 seeds, CUDA) | Recall mean | Source ranked first when kept |
|---|---:|---|
| SR, shared + ranked | 0.024 | 0.03–0.12 |
| **SGS, + surrogate** | **0.141** (seeds 0.044 / 0.288 / 0.091) | **0.55–0.78** |
| SG7, unshared + surrogate | 0.009 | 0.37–0.49 |

**What it adds:**
- \`set_starling_surrogate\` and \`mqar-bench starling_surrogate=true\`: training only, never saved.
- The forward is bit-identical, so the runtime read stays softmax-free.
- The gradient reaches q·k only; the values are detached.
- **Tests:** identical logits, a different query gradient, a bit-identical value gradient, and an address route refusing the surrogate. **Negative gate:** removing the detach fails the test.

**Owner decision** (#2032): pause (the pre-registered recommendation), or one targeted step on address stability under the surrogate.

Record: `docs/labs/starling-surrogate-2026-10-12/README.md`. References #2032.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6115751180 — 2026-10-12T00:44:57Z — Casey-allard

**Exact-head checks at `a10e01c6d5dc30be5ac0e040153bf953463bb68f`, all on an x86 GPU pod** (owner direction: no laptop builds; the pod was deleted afterwards):
- **Bootstrap:** release build and `cuda_stack_ops_parity`, **37 passed**.
- `cargo test --release -p uor-r4-training --lib -- starling content_order`: **11 passed, twice**, in parallel mode. This includes the surrogate test and the FD tests made robust in #2221.
- `cargo build --release --example mqar-bench`: PASS.
- **Negative gate** (on the pod, #2032): removing `.detach()` on the values fails the value-gradient assertion.
- All 6 scored runs used this surrogate code on CUDA. Only the test and the docs changed since.

### review 5485729872 — 2026-10-12T00:44:58Z — Casey-allard — COMMENTED

**Self-review at exact head `a10e01c6d5dc30be5ac0e040153bf953463bb68f`.**
- Training only, opt-in, never saved; the forward is unchanged.
- The record applies the pre-registered bars as written (reading 3). It reports the content-selection evidence separately, without re-scoring.
- The owner decision is stated.

APPROVE (self-review is allowed under the owner's exact-head rule).


---
_Comment/review ids in this file: 6115751180, 5485729872_

