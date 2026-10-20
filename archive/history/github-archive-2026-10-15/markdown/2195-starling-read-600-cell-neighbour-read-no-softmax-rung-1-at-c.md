# #2195 — Starling read: 600-cell neighbour read (no softmax); rung 1 at chance from scratch — address needs a training signal (References #2032)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-11T07:55:51Z
- **Updated:** 2026-10-11T07:57:47Z
- **Closed:** 2026-10-11T07:56:55Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2195
- **Merged:** 2026-10-11T07:56:55Z
- **Merge commit:** ac5820717377be0518d05143816c6d20fe889476
- **Base / Head:** main / UOR-Foundation:claude/starling-result
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Line "starling read" · count 0/3 → 1/3 · headline before → after: softmax at runtime yes, served 0.886838 BPB → unchanged.**

Result PR for starling read rung 1 (#2032 [pre-registration](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6105228141), [amendment](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6105276314), [FIT](https://github.com/UOR-Foundation/uor-r4/issues/2032#issuecomment-6105293249)). **Row 5: both starling arms at chance.** This closes only `starling:4:7` and `starling:8:7` trained from scratch; the mechanism stays open (D22 §1).

| arm | mqar in-class recall (3 seeds) | positions scored per query |
|---|---:|---:|
| L, L2 softmax read | 0.617 (0.669, 0.373, 0.809) | 1,427 |
| S7, starling:4:7 | 0.009 | 207 |
| S7w, starling:8:7 | 0.009 | 411 |

- Recall is at chance in every distance bucket, and M = 8 is no better, so the cause is not eviction.
- Hard selection gives keys outside the kept set **no gradient**.
- The 7× cost saving is real.
- **Next:** soft-to-hard address training (training only; the served read stays hard and softmax-free), pre-registered separately.

## Contents

- **Code:** `read_route=starling:M:K` (library and `mqar-bench`), opt-in. The default path is unchanged; export refuses it. The code merges because the next rung trains this same read.
- **Record:** `docs/labs/starling-read-2026-10-11/` (README and `result.json`).
- **Brief:** `docs/mechanisms/starling-read.md`.
- **Updated:** current-state, STATUS, ROADMAP.

Would the next step differ if this came out the other way? Yes. Recall at L2's level would have sent the read straight to a language rung; at chance, the next step is address training.

References #2032

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6106887666 — 2026-10-11T07:56:02Z — Casey-allard

**Exact-head checks at `fbd25550f0dbb9a8019fc62226de0cfd5a3dca34`.** The code is identical to the tested tree: the full branch diff was applied 3-way onto main 44f77f830 + #2193, with conflicts resolved by keeping both sides and the starling tests re-appended verbatim. Laptop, macOS arm64:
- `cargo test --release -p uor-r4-training --lib starling`: `ok. 4 passed`.
- `... --lib quaternion_weights`: `ok. 4 passed`. `... --lib activation_snap`: `ok. 1 passed`.
- `... --lib geometric_stack`: `ok. 168 passed; 0 failed; 1 ignored`.
- `cargo build --release --example mqar-bench --example geometric-stack`: OK.
- `python3 scripts/check_claim_wording.py`: passed.
- **Negative gate** (run at aba4b1da0): removing the 12 neighbour cells fails the brute-force selection test.

**Exact-head self-review (Claude lab):**
- **The conflict resolution** keeps main's `activation_snap` and quaternion fields and save/load blocks next to `read_route`. Every starling and quaternion test runs and passes.
- **Starling read:**
  - Separate query and key address maps.
  - Detached host selection; `gather` for keys and values (not `index_select`).
  - Plain rank table times gate, no softmax.
  - Skips the age table, so the length cap applies only without a route.
- **Bench:** the starling cost count is min(13·M, t+1) per head.
- **Record:** every number comes from `result.json` and the bench reports.
- APPROVE.


---
_Comment/review ids in this file: 6106887666_

