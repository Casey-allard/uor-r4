# #1569 — fix(training): paraphrase-gen seeds all relation acts, strips prompt labels (#1552)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T07:28:04Z
- **Updated:** 2026-10-09T14:17:23Z
- **Closed:** 2026-10-01T07:29:11Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1569
- **Merged:** 2026-10-01T07:29:11Z
- **Merge commit:** 02a81e71b1f069ac2d762af683e3b7474073f632
- **Base / Head:** main / UOR-Foundation:lab/opencode/paraphrase-seed-filter
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Fix the seed scan and one decoder leak found by run-2/smoke-10.

**Run-2 defect (sealed `opencode-paraphrase-20261001/run-2`):** the scan filtered `turn.category == Category2::Relation`, but the world assigns that category only to **query** turns; assert and update relation turns are `Category2::Responsive`. Run-2 therefore held only question seeds (30 unique, 0 statements).

**Changes**
- Select seeds by relation intent: `relation_act(&turn.intent).is_some()` (covers assert/update/query/absent).
- Strip a leading `Rewrite:`/`Paraphrased:` prompt label after decoding. smoke-10 accepted one labelled echo (`Rewrite: I am from Dulseestfield.`); it is now cleaned before screening (smoke-11: 0 leaks).
- Unit test for the label strip.

**Exact-head evidence (a512c743)**
- `cargo fmt -p uor-r4-training -- --check` PASS
- build + `cargo test --release -p uor-r4-training --example paraphrase-gen --offline`: 4 passed
- smoke-11 (8 seeds, 4 draws, 300 conversations, temperature 0.6): 6 statements + 2 question seeds; 6 consumer rows with `assert`/`update`/`query` acts; 0 label leaks

Offline data-source tool only; no serving path. References #1552.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5926795958 — 2026-10-01T07:28:18Z — Casey-allard

Exact-head evidence at `a512c743cdc830960a84126c236ba924f64babb6` (solo lab; self-review).

- `cargo fmt -p uor-r4-training -- --check` PASS
- `cargo build --release -p uor-r4-training --example paraphrase-gen --offline` PASS
- `cargo test --release -p uor-r4-training --example paraphrase-gen --offline` — 4 passed (nucleus, person flip, intent mapping, prompt-label strip)
- smoke-10 (pre-fix, statements visible): 8 seeds → 6 statements + 2 questions; consumer rows showed one `Rewrite:` leak
- smoke-11 (post-fix): 8 seeds → 6 statements + 2 questions; 6 consumer rows with acts `update`/`assert`/`query`; 0 label leaks; rejection mix `copied_seed 21, duplicate_wording 1, kind_changed 2, value_changed 2`

Diff scope: only `crates/uor-r4-training/examples/paraphrase-gen.rs`. Offline data-source tool; no serving path.



---
_Comment/review ids in this file: 5926795958_

