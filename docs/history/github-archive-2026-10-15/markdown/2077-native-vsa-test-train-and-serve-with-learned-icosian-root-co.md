# #2077 — Native VSA test: train and serve with learned icosian-root codes; off switches; run plan

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T19:34:24Z
- **Updated:** 2026-10-09T19:36:04Z
- **Closed:** 2026-10-09T19:35:32Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2077
- **Merged:** 2026-10-09T19:35:32Z
- **Merge commit:** e9292638536c7c0f7dac933f6de4593d845b6fab
- **Base / Head:** main / UOR-Foundation:claude/vsa-native-test
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Builds the owner-approved native VSA retraining test (M1 #2029; review in `docs/labs/vsa-review-2026-10-09/`).

**Changes (native prose learner, `uor-r4-core`)**
- **Switches:** `JepaTrainerConfig` gains `vsa_enabled` (scale held at 0 and exported as 0) and `engram_enabled`. The exact engram n-gram table is P7's exact-lookup term, standing in for the Copy head this learner doesn't have.
- **Learned codes:** `vsa_code_mode` 0/1/2 trains with fixed hash codes, codes derived from the learned icosian-root assignment, or root codes plus a readout hash. Codes are refreshed every `vsa_code_refresh` steps, and everything derived from the codebook is rebuilt.
- **Export and serving use the same codes:**
  - The binary format accepts mode 2.
  - The runtime takes the codebook from the artifact instead of always rebuilding the fixed hash, and builds it once per session instead of once per token.
  - An explicit `--vsa-codes` choice exports the mode it trained with. The default keeps main's historical mode-1 export.
- **CLI** (`train-native-prose`): `--no-vsa`, `--no-engram`, `--vsa-codes fixed|root|readout`, `--vsa-code-refresh`, `--seed` and `--offset` (where the training window starts, to keep the held-out slices out of training).
- **`docs/labs/vsa-native-test-2026-10-09/PLAN.md`:** the arms, data, held-out slices, metric with its paired bootstrap interval, and the decision (KEEP at ≥ 0.01 BPB on both slices and seeds; otherwise make VSA more native, never retire it).

**Checks at this head**
- `cargo test -p uor-r4-core --lib vsa_`: 13 passed. New tests: switches, root codes used in training, learned codes round-tripping to serving (modes 1 and 2), the mode-0 export unchanged, and the explicit fixed-code export.
- `cargo test -p uor-r4-core --test binary_model_tests`: 9 passed.
- `cargo check -p uor-r4-core --bins`: clean.

Results come in a follow-up PR with the dated record and a `current-state.md` entry.

References #2029

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6087897962 — 2026-10-09T19:34:33Z — Casey-allard

Exact-head self-review at `2053badda78269c372bb0585fac1d6e9a0a244c2`: reviewed the trainer, runtime, binary-format and CLI diffs; checks above re-run at this head after rebase on main.


---
_Comment/review ids in this file: 6087897962_

