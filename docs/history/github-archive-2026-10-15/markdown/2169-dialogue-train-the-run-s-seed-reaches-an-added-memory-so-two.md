# #2169 — dialogue-train: the run's seed= reaches an added memory, so two seeds are two mechanisms (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T17:01:27Z
- **Updated:** 2026-10-10T17:15:44Z
- **Closed:** 2026-10-10T17:14:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2169
- **Merged:** 2026-10-10T17:14:33Z
- **Merge commit:** 5eac1a6a36db4573aaec6b8a24b58ab440da917f
- **Base / Head:** deepseek/addressed-memory-20261010 / UOR-Foundation:deepseek/memory-seed-20261010
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body


**Follow-up to [#2168](https://github.com/UOR-Foundation/uor-r4/pull/2168), found by running it.** Two arms of the same memory configuration with different `seed=` values were **bit-identical at step 0** (`dev response NLL 2.246629649751581` on both), because the added memory initialised from the **saved model's** seed and nothing else is initialised at load time — so the two seeds of the pre-registered run would have been one run twice.



The seed now comes from the run (`args.number("seed", saved.seed)`), the way `pointer=DIM` takes it, and the CLI test asserts the run's seed reaches the configuration while the saved model's own seed does not change (`seeded.seed == 7` against the saved `5`).



| check | command | result |

|---|---|---|

| CLI test | `cargo test -p uor-r4-training --example geometric-stack memory_options_add_a_memory_to_a_saved_model_and_must_agree_with_one` | **pass**, including the new seed assertion |

| example suite | `cargo test -p uor-r4-training --example geometric-stack` | **13 passed, 0 failed** |

| library test | `cargo test -p uor-r4-training --lib a_memory_added_to_a_saved_model_matches_a_fresh_construction` | **pass** |



Line: exact addressed memory (D22 order 2) · count 0/3 · headline unchanged (preparation for the two-seed memory run pre-registered on #2029).



References #2029

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

