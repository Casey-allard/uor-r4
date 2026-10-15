# #2193 — Quaternion-valued matrices: KEEP — structure beats plain sizing at equal weight bytes (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T06:37:57Z
- **Updated:** 2026-10-11T06:39:29Z
- **Closed:** 2026-10-11T06:39:00Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2193
- **Merged:** 2026-10-11T06:38:59Z
- **Merge commit:** ca5bdc6173b5b2ebbbadf705d713c9dd1a8eca1c
- **Base / Head:** main / UOR-Foundation:claude/quat-weights
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Line "geometric weights" · count 0/3 → reset (KEEP) · headline before → after: softmax at runtime yes, served 0.886838 BPB → unchanged.** This rung targets compute structure, not the read.

Result PR for the pre-registered quaternion-weights study (#2032 [6103994742](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6103994742), amendments 1–3, `TEST FITNESS: FIT` [6104042694](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6104042694)). **KEEP, decision row 2.**

| arm | held-out BPB | map weights | weight bytes per token |
|---|---:|---:|---:|
| D256 dense | 1.4078 | 4.36M | 21.6 MB |
| **Q256 quaternion** | **1.4959** | 1.09M | 8.55 MB |
| D152 dense, equal total parameters | 1.5136 | 1.53M | 8.62 MB |
| D128 dense, equal map weights | 1.5520 | 1.09M | 6.46 MB |

- **Q256 − D152 = −0.018 BPB, 6× the noise (0.0029).** At equal weight bytes, quaternion structure beats plain sizing.
- **Q256 − D256 = +0.088 BPB**, the cost of 4× fewer matrix weights at equal width.
- **Arithmetic is not reduced yet**, because a Hamilton block is still 16 multiply-adds. That needs icosian table products; the spin-snap probe is running.
- **Precondition met:** D128 − D256 = 0.144. The curve has settled.

## Contents

- **`quaternion_weights=none|all`** (geometric stack + train CLI). Default behaviour is byte-identical.
- **Spin-snap probe tools:** `examples/quaternion-snap.rs`, and `evaluate activation_snap=icosian|direction|scale` (evaluation only, quaternion-map inputs only).
- **Record:** `docs/labs/quaternion-weights-2026-10-11/`, including the pre-registered `score.py` and `result.json`.
- **Mechanism brief:** `docs/mechanisms/spin-native-weights.md`.
- **current-state, STATUS, ROADMAP** updated.

Would the next step differ if this came out the other way? Yes. A tie with D152 would have moved the compute bridge to the addressed MLP and closed this configuration.

References #2032

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6106314126 — 2026-10-11T06:38:08Z — Casey-allard

**Exact-head checks.** Code tip 0079e4766 (rebased on 44f77f830); head `32de383a30760f2c7a922473db50ce48bad8b7e3` adds docs only. Laptop, macOS arm64:
- `cargo test --release -p uor-r4-training --lib quaternion_weights`: `ok. 4 passed; 0 failed`.
- `... --lib activation_snap`: `ok. 1 passed`.
- `... --lib geometric_stack`: `ok. 164 passed; 0 failed; 1 ignored`.
- `cargo build --release --example geometric-stack --example quaternion-snap`: OK.
- `cargo fmt --check -p uor-r4-training`: 18 hunks, all in code this PR does not touch (other labs' October 9 commits: demand-store-2turn, mask-scan, mqar-bench, relation_compiler, stack_memory, and geometric_stack's pointer gate). The changed lines are rustfmt-clean.
- `python3 scripts/check_claim_wording.py`: passed.
- **Negative gate** (run during development): flipping one Hamilton-basis sign fails the exactness test.

**Exact-head self-review (Claude lab):**
- **The quaternion path:**
  - The expansion is `W[4i+r,4j+s] = L(Q[i,j])[r,s]` with the stated L.
  - The default `None` adds no variable and no config field, so old models load unchanged.
  - Export, qat and transport_snap are refused.
- **The probe switch:**
  - It is evaluation only, never saved.
  - It snaps only inputs whose weight is neither [vocab, width] nor [heads, width]. That is exact for a plain quaternion stack, pinned by a unit test, and confirmed by the fitness reviewer.
- **The record:** every number comes from `result.json`, the output of the pre-registered `score.py`.
- APPROVE.


---
_Comment/review ids in this file: 6106314126_

