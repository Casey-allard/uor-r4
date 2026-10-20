# #1562 — feat(training): teacher paraphrase generator for the relation channel (#1552)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T04:35:36Z
- **Updated:** 2026-10-09T14:17:00Z
- **Closed:** 2026-10-01T04:54:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1562
- **Merged:** 2026-10-01T04:54:23Z
- **Merge commit:** 8d0373a5dfa3ceb1f54112935ed1f794c07a8b1b
- **Base / Head:** main / UOR-Foundation:lab/opencode/paraphrase-gen
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Owner ruling 2026-10-01 04:14Z: local SmolLM2-Instruct **offline as a data source only, never serving**; wordings of relation statements/questions seeded with training phrasings only; development templates/n-grams screened out.

This PR adds `crates/uor-r4-training/examples/paraphrase-gen.rs`: draws relation seeds from the training-phrasing cells; samples with the local teacher; guards = empty, verbatim copy, question/statement kind change, invented or dropped values (digit runs, capitalized names), exact development-cell match, shared 4-word sequence; writes `paraphrase.jsonl` + sealed manifest (counts, teacher/tokenizer identity, hashes). Smoke (360M, temp 0.6): real paraphrases accepted, copies/inventions rejected. No training. Data-generation run follows its own card on #1552.

## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 5924824202 — 2026-10-01T04:35:50Z — Casey-allard

Exact-head self-review (D18 rules; solo lab): the change is a new example only (`crates/uor-r4-training/examples/paraphrase-gen.rs`); no existing path is modified; `cargo build --release -p uor-r4-training --example paraphrase-gen --offline` PASS and `cargo fmt --check` clean at this head; smoke runs sealed under `opencode-paraphrase-20261001/smoke-{1..5}` (the last two evidence the guards working). Scope: data generator for the relation channel per the owner ruling; no training, no serving change.

### comment 5924847680 — 2026-10-01T04:38:10Z — Casey-allard

Exact-head evidence at `a294870e010a15f70daf463e5e872ad7a2b1ff48` (the PR head): `cargo fmt -p uor-r4-training -- --check` PASS; `cargo build --release -p uor-r4-training --example paraphrase-gen --offline` PASS; push-event `ci` run 36815841260 success at this head; dispatched `ci qa_scope=product_path` run 36815856578 in flight. The `Lab delivery evidence` failure is the advisory coordination gate failing on a missing `codex/lab-state` ref (infrastructure), not this change.

### comment 5927087083 — 2026-10-01T07:45:51Z — Casey-allard

**Follow-up on the dispatched `qa_scope=product_path` failure — pre-existing on main, not this PR.**

The only failing test is `exact_probe::tests::exact_executor_contract_binds_complete_local_source_closure` (`uor-r4-model-source`): it requires every local production source under the contract crates to be listed in `exact_executor_contract_components()`. Reproduced locally on current `origin/main` (`02a81e71`, which already contains this PR): **403 omitted sources, exit 101** (`/tmp/exact-probe-main.log`).

The omitted entries are under `crates/uor-r4-core/` and `crates/uor-r4-api/`; both trees are byte-identical between this PR's base `b06b46a3` and `origin/main` (empty `git diff --stat`), and the contract function was last touched 2026-09-02 (`6c7354c2`) while most omitted files postdate it.

This PR changes only `crates/uor-r4-training/examples/paraphrase-gen.rs`, which is not under a contract crate; the same-head `pull_request` `ci` run passed, and only the manually dispatched run failed on this test. No action for this PR; flagged for the crate owner as a stale contract on `main`.



---
_Comment/review ids in this file: 5924824202, 5924847680, 5927087083_

