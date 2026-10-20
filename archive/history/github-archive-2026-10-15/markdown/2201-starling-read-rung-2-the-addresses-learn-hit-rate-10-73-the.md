# #2201 — Starling read rung 2: the addresses learn (hit rate 10% -> 73%), the readout does not; row 3 (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T15:35:06Z
- **Updated:** 2026-10-11T15:36:27Z
- **Closed:** 2026-10-11T15:36:08Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2201
- **Merged:** 2026-10-11T15:36:08Z
- **Merge commit:** 0f9f7f4a62e7c2374940db1485ead4d77dd05053
- **Base / Head:** main / UOR-Foundation:claude/starling-anneal
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Line:** starling read · count 2/3 · headline "softmax at runtime: yes" → yes (0.886838 BPB served, unchanged)

**Result: row 3 of the pre-registered rows.** The addresses learn, but the readout does not use them.
- **Pre-registration:** #2032 [6107021590](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6107021590), amended in 6107070727 and 6107090319.
- **Fitness:** [FIT](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6107098703).
- **Device amendment:** [6110389805](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6110389805).

**What it adds.** `starling_dense`, a training-only helper. It adds α × a causal mean of `σ(β⟨a,b⟩+γ)v` over all earlier positions, with no softmax. α is annealed to zero over the first half of training. Evaluation, save and export read the pure starling form. `mqar-bench` gains `starling_anneal=` and logs the address hit rate at step 0 and every 100 steps.

**Result** (6 runs on one RTX 4090, 2 min 52 s in total):

| Arm | Recall (3 seeds) | Hit rate: step 0 → peak → final |
|---|---:|---|
| L (rung 1) | 0.617 | — |
| SA7 `starling:4:7` | 0.011 | 0.100 → 0.725 → 0.432 |
| SA7w `starling:8:7` | 0.011 | 0.099 → 0.727 → 0.556 |

**Next:** the readout (rank table and gate). A negative closes this configuration only (D22 §1).

Record: `docs/labs/starling-anneal-2026-10-11/README.md`. References #2032.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6110659061 — 2026-10-11T15:35:17Z — Casey-allard

**Exact-head checks at `ac0a8f2f62f1b3e50c6b5ff173d48f01df6ead56`** (laptop, rustc via rustup):
- `cargo fmt -p uor-r4-training --check`: the only diffs are in code already on main, which this branch does not touch: `geometric_stack.rs` lines ~4637 and ~9443, the read-binding margin. Every file this branch changes is formatted.
- `cargo test --release -p uor-r4-training --lib starling`: **5 passed**, including α = 0 bit-equal to the pure read and the helper's key-address gradient matching finite differences.
- `cargo build --release --example mqar-bench`: PASS.
- `scripts/check_claim_wording.py`: PASS.
- **CUDA:** the pod bootstrap at 993f41035 (the same code before the rebase) passed `cuda_stack_ops_parity`: 37 passed.
- **Score:** `score.py` against the run logs (cloud-store `claude/starling-anneal-20261011`, md5 69572e7b…) reproduces `result.json`.

### review 5483867630 — 2026-10-11T15:35:18Z — Casey-allard — COMMENTED

**Self-review at exact head `ac0a8f2f62f1b3e50c6b5ff173d48f01df6ead56`.**
- **Diff:** the helper is applied only while training. The save, export and evaluation paths read the pure starling form.
- **Rows:** the scoring follows the amended rows exactly (SE_diff, H = max(0.15, 2·h₀), and peak and final per arm). All 6 runs reached step 1,800, so none is a blocker.
- **Claims:** the record does not credit recall. It reports that the addresses learn and that recall stays at chance.
- **Docs:** STATUS, ROADMAP, the brief and current-state are updated; the README is unchanged because no capability changed.

APPROVE (self-review is allowed under the owner's exact-head rule).


---
_Comment/review ids in this file: 6110659061, 5483867630_

