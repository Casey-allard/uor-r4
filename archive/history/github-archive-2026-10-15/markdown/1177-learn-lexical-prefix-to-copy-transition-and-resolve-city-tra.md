# #1177 — Learn lexical-prefix-to-copy transition and resolve city transfer (#973)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-08T00:24:30Z
- **Updated:** 2026-10-09T14:16:59Z
- **Closed:** 2026-09-08T01:05:08Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1177
- **Merged:** 2026-09-08T01:05:08Z
- **Merge commit:** 589fe86b4bb21bd0603c7b90a2b06e24a1b80c0d
- **Base / Head:** main / UOR-Foundation:issue-973-learned-lexical-prefix-city-transfer
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Under owner-directed recovery mode `#973`, this PR resolves the `0/2` city transfer failure (`" Oslo.\n"` and `" Lima.\n"`) to a **`2/2` (100%) exact pass**, while maintaining 100% bit-exact fidelity on all prior achievements.

### Key Innovations
1. **Feature 20 & 21 Separability**:
   - Added Feature 20 (`entry.steps`, `entry.active`) and Feature 21 (`last_prime`) to `word_copy_runtime.rs` so the selector can linearly distinguish Step 0 (yielding to `ResponseEntry`) from Step 1 (initiating `WordCopy`).
2. **Transition Eligibility**:
   - Expanded `eligible()` in `word_copy_runtime.rs` to permit copy initiation when `entry.steps < RESPONSE_ENTRY_STEPS` under active boundary.
3. **Dual-Step Supervision**:
   - Added Step-0 NoCopy example generation in `word_copy_training.rs` for leading-prefix targets, training `WordCopy` to yield at Step 0 and start at Step 1.
4. **Counterfactual Construction City Pairs**:
   - Authored 4 counterfactual city pairs (8 cases: `Paris`/`Tokyo`, `Rome`/`Berlin`, `Kyoto`/`Delhi`, `Cairo`/`Madrid`) under `world >= 400_000` in `native_geometric_value_probe.rs` to provide positive `ResponseEntry` and `WordCopy` supervision while remaining strictly disjoint from development (`Oslo`, `Lima`).

### Empirical Verification
- **City Transfer**: `2/2` exact pass (`" Oslo.\n"`, `" Lima.\n"`).
- **Parameter Transfer**: `12/12` exact responses preserved.
- **Numeric Targets**: `24/24` correct leading numerals preserved.
- **Rust Programs**: `20/20` prior SHA-256 matches; `12/12` identity libraries compiled and passed `84/84` assertions.
- **Allocations**: `6/6` suites in `native_geometric_allocations` pass with 0 heap allocations on the hot path.
- **Controls**: Under `WordCopyDisabled` and `WordCopyGeometryDisabled`, city transfer collapses to `0/2` and parameter transfer to `0/12`.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

