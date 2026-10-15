# #1442 — B1 finite-group tracking lanes: Stage A (A5) and the Stage B stack harness (References #973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-27T23:37:37Z
- **Updated:** 2026-10-09T13:08:57Z
- **Closed:** 2026-09-28T03:11:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1442
- **Merged:** 2026-09-28T03:11:23Z
- **Merge commit:** 3f93b653fe2359da457d72e40218a6e491ba3d65
- **Base / Head:** main / UOR-Foundation:lab/claude/b1-tracking-lanes
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Lab:** Claude, Lab 1. **Track:** ROADMAP T1(b), B1. **Branch:** `lab/claude/b1-tracking-lanes`. **State:** Stage A and Stage B complete.

**What this adds.**
- `crates/uor-r4-training/src/stack_tracking.rs` (new):
  - token-conditioned tracking lanes in four kinds: quaternion, reflection pair, commutative phase, frozen;
  - the A5 word problem, labelled from the repository's exact 2I Cayley table;
  - group-closure snapping to a Cayley-table automaton, with Moore minimisation;
  - `TrackedStack`: the geometric stack plus a zero-initialised lane side channel and an auxiliary A5 read-out.
- `examples/tracking-lanes.rs`: `a5` and `mixed` drivers. Each claims its report root before loading and seals it at the end.
- `geometric_stack.rs`: a behaviour-preserving `embed` / `hidden_from_input` / `head` split. A test checks that the logits equal `forward`.
- `docs/integration/b1-finite-group-lanes-2026-09-27.md`, the record.
- `docs/evidence/b1-finite-group-lanes-2026-09-27.json`, the evidence: source `09e537a4`, binary SHA-256, run hashes, sealed-root manifests and gates.

**Stage A** (A5 word problem, training length ≤32):
- Non-commutative lanes are exact to 4,096 once snapped in 17 of 18 runs: quaternion 8/9, reflection pair 9/9. Phase and frozen lanes stay at chance.
- Both lane kinds minimise to the same 60-state A5 automaton, so **the quaternion-specific serving claim is retired**.
- A rerun on the committed code reproduced the pre-commit grid exactly.

**Stage B** (recurrence-primary stack `rrar`, 1.39M parameters, text from the #1017 token store plus A5 words, 3 seeds per arm). Gates as pre-committed: every seed's text ΔNLL ≤ 0.05 nats against the lane-free stack, and A5 ≥ 0.99 at position 128.

| Arm | Text ΔNLL per seed (mean) | A5 at position 128 per seed | Verdict |
|---|---|---|---|
| reflection pair | +0.036, −0.050, +0.009 (−0.002) | 1.000, 1.000, 1.000; snapped exact to 4,096 | **PASS** |
| quaternion | +0.012, −0.052, **+0.121** (+0.027) | 1.000, 1.000, **0.039** | FAIL (reliability) |
| phase | +0.089, +0.094, +0.054 | 0.016, 0.348, 0.004 | FAIL (commutative) |
| none (reference) | — | 0.016, 0.160, 0.008 | — |

**Checks run.**
- `cargo fmt --check -p uor-r4-training` passes.
- 5 focused tests pass.
- Release builds of the example.
- `check_claim_wording.py` passes.

**Scope.** A5 is synthetic. The text gate shows only that tracking costs no text quality. No language-capability claim is made, and state tracking in natural language is untested.

**Serving (D11/R1–R3).** The lane automaton serves as one byte of state and two table reads per token: no multiplier, no float, sparse table access. It is not yet wired into `uor-r4-integer`; that is T3's port.

**Storage.** Reports (1.3 MB) and the build cache are on the SSD.

References #973, #820

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5861986524 — 2026-09-28T02:02:28Z — Casey-allard

**Independent review, and the director's response.**

**The review found:**
- The code is correct: quaternion conventions, the reflection-pair construction and the scan drive; the `geometric_stack` split preserves behaviour.
- Evaluation integrity is clean: disjoint seeds and held-out dev windows.
- The Stage A rerun is 36 of 36 identical.

**Record corrections (commit `18356e02`):**
- The pre-registered 2I Stage B arm **fails** its per-seed gates.
- The transformer control is **NOT_RUN**.
- The reflection-pair pass is **exploratory**, since that arm was added after Stage A.
- The kill rule is applied as written: keep the better ordinary lanes and retire the 2I serving claim. The earlier narrowing is withdrawn.
- Also fixed: the snapped-lane counts, the deviation range, the Stage A timing source, the SU(2) statement, the parameter totals and the snapping limits.
- The evidence JSON gains merge distances, snapped counts and a pre-registration verdict block.

**Merge coordination with #1437.** #1437 adds `memory: Option<MemoryConfig>` to `StackConfig`, which has no `Default`. Whichever PR lands second must add `memory: None` to the two `StackConfig` literals here (the `stack_tracking` test and `examples/tracking-lanes.rs`) and keep both `lib.rs` module lines.
- `merge-tree` shows no textual conflict in `geometric_stack.rs`.
- This is a compile break in tests and examples only; the lib target still builds, and CI runs no builds.

**Deferred to Stage C (#1447), where the snapper is rebuilt with saved tables:**
- a permutation-consistency check and an exhaustive check against `group_table` in `snap`;
- saved automaton tables;
- `Rng::below(0)` and the unchecked token index in `LaneAutomaton::run`.


---
_Comment/review ids in this file: 5861986524_

