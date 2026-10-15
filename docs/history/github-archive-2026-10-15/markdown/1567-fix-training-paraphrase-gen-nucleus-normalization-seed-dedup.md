# #1567 — fix(training): paraphrase-gen nucleus normalization, seed dedup and screening (#1552)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T06:45:07Z
- **Updated:** 2026-10-09T14:17:22Z
- **Closed:** 2026-10-01T06:48:08Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1567
- **Merged:** 2026-10-01T06:48:08Z
- **Merge commit:** 695ed34b5afddfb4bb639b9a80e35385c30f6237
- **Base / Head:** main / UOR-Foundation:lab/opencode/paraphrase-fix
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Fix the sampler defect found in run-1 and make the output directly consumable by the E3 relation compiler.

**Defect (run-1, sealed `opencode-paraphrase-20261001/run-1`):** 400 rows / 144 accepted but only **9 unique wordings**; the top-p nucleus truncated on *unnormalized* cumulative weight, so with the top weight `exp(0)=1 >= top_p` every draw kept a one-token nucleus and was greedy; seed turns were also not deduplicated (29 unique texts among 400 rows).

**Changes**
- `sample()` truncates at `top_p * total weight mass` and validates `top_p in (0, 1]`.
- Seeds are deduplicated by normalized text before the item budget.
- New screens: person flip (a new second-person address that loses the seed's first-person reference) and duplicate wording for the same seed.
- New `paraphrases.jsonl` consumer output: accepted rows as `{relation, act, text}` for `relation_compiler::paraphrase_examples`, plus consumer counts in the manifest.
- Unit tests for the nucleus, the person flip and the intent-to-relation/act mapping.

**Exact-head evidence (0b930303)**
- `cargo fmt -p uor-r4-training -- --check` PASS
- build + `cargo test --release -p uor-r4-training --example paraphrase-gen --offline`: 3 passed
- smoke-8 (release binary): 10/32 accepted, all unique; `person_flipped` rejection exercised
- smoke-9: 5 consumer rows, relations/acts valid; `paraphrase.jsonl` + `paraphrases.jsonl` + manifest consistent

Offline data-source tool only; no serving path change. References #1552.

Follow-up: the dispatched `qa_scope=product_path` run 36815856578 failed on `exact_probe::tests::exact_executor_contract_binds_complete_local_source_closure` (stale contract list, hundreds of omitted sources). This diff adds no file in the contract's required crates, so the same failure reproduces on main; it is not caused by this PR.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5926215549 — 2026-10-01T06:47:15Z — Casey-allard

Exact-head evidence at `0b930303d8f99908375f00a4d8a50d95ef43838d` (solo lab; self-review).

- `cargo fmt -p uor-r4-training -- --check` PASS
- `cargo build --release -p uor-r4-training --example paraphrase-gen --offline` PASS
- `cargo test --release -p uor-r4-training --example paraphrase-gen --offline` — 3 passed: normalized nucleus keeps multiple tokens and validates `top_p`; person flip requires a new second person and a lost first person; intent maps to the compiler's relation/act
- smoke-8 (8 seeds × 4 draws, temperature 0.8): 10 accepted, 10 unique, `person_flipped` rejection exercised
- smoke-9 (6 seeds × 4 draws, temperature 0.6): 5 accepted, 5 unique; `paraphrases.jsonl` consumer rows valid (`friend_name`/`job`/`pet_name`/`code_word`, act `query`)
- Defect evidence: run-1 sealed root `opencode-paraphrase-20261001/run-1` — 144 accepted rows but 9 unique wordings; greedy one-token nucleus (`total >= top_p` on unnormalized mass).

Diff scope: only `crates/uor-r4-training/examples/paraphrase-gen.rs`. Offline data-source tool; no serving path.



---
_Comment/review ids in this file: 5926215549_

