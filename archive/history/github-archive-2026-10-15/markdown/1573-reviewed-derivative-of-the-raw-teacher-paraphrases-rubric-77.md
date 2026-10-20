# #1573 — Reviewed derivative of the raw teacher paraphrases: rubric, 776 decisions and the tool

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T13:57:45Z
- **Updated:** 2026-10-09T13:08:46Z
- **Closed:** 2026-10-01T13:58:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1573
- **Merged:** 2026-10-01T13:58:52Z
- **Merge commit:** e126867f2028ae59f17d098697b83fc52424398e
- **Base / Head:** main / UOR-Foundation:codex/paraphrase-review
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #1552. This is owner-directed data review (option chosen while SSD writes are paused, #820). It runs no fit and writes nothing to the SSD.

**Why.** The audits on #1552 found inherited-label errors in Claude's raw teacher paraphrases: birthplace vs upbringing, negative preference vs lucky number, `a {v} friend`, user/addressee reversal. v14–v19 trained on that raw union.

**What:**
- **`docs/integration/paraphrase-review-2026-10-01/rubric.md`:** frozen before any row was read (sha256 `1245a500…`). It defines the label contract (the user's own value; assert vs update by an explicit correction cue; the slot rule) and the drop rules. Borderline means drop.
- **`.../decisions.tsv`:** one line per raw row (`pN:LINE`, decision, new labels, reason, exact text) for all 776 rows.
  - **Counts:** 372 keep, 22 relabel, 260 drop, 122 duplicate. The derivative has 394 rows covering all ten relations.
  - **Gap:** `hometown/update` is empty, because every raw hometown correction failed.
- **`src/paraphrase_review.rs`** (unit-tested) and **`src/bin/paraphrase-review.rs`** (file I/O, claimed root). The tool refuses unless:
  - every raw row has exactly one decision whose text equals the raw text;
  - exact duplicates, which it recomputes itself, are marked and point at the first occurrence;
  - kept and relabelled rows satisfy the slot rule;
  - each relabel changes a label.

  The output rows keep `relation`/`act`/`text` (loadable by `paraphrase_examples`) plus `source`, `source_sha256` and the review reason.

**Disclosure.** The reviewer (Claude) had seen development-cell results. Decisions judge label correctness only; no row is written or edited.

**Not yet done:**
- Building the derivative root and the paired fit (reviewed vs raw, everything else fixed) wait for the SSD repair.
- No model result is claimed.

**Checks (GitHub evidence `codex/ci/v3/paraphrase-review`, run 36870743811, parent = this head):**
- fmt success;
- `cargo check --workspace --all-targets` (workbench excluded) success;
- `uor-r4-training` lib **429 passed, 0 failed, 5 ignored**, including both `paraphrase_review` tests;
- `m-world` example 5 passed.

Local: `git diff --check` clean; `check_claim_wording.py` passed. There was no local build: the build cache is on the paused SSD.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### review 5380288590 — 2026-10-01T13:57:58Z — Casey-allard — COMMENTED

Exact-head review at `ee19acae81ba90a74eab5c89601cee40592c9968` (self-review under the owner's rule).

**Source read (`paraphrase_review.rs`, `bin/paraphrase-review.rs`, the `lib.rs` export):**
- **Input checks before output:** every raw row needs exactly one decision with equal text. Stray ids are refused. Reasons are non-empty.
- **Duplicates:** decided by exact text against the first occurrence (`first.or_insert`), so a chain of duplicates points at the first row. A missed or false duplicate is refused.
- **Slot rule:** checked on the *output* labels, so a relabel to `query` of a slotted row is refused (tested). A no-op relabel is refused.
- **Binary:** reads and checks everything before `report_output::claim`, writes `error.json` on a failed write, then seals and verifies.

**Record:**
- the rubric digest `1245a500…` equals the pre-review digest;
- `decisions.tsv` has 776 six-field lines (372 keep / 22 relabel / 260 drop / 122 duplicate).

**Executed:** GitHub evidence run 36870743811 on `473e5253` (parent = this head):
- fmt, check (all targets) and tests: success;
- lib 429 passed, 0 failed, including both new tests;
- `m-world` 5 passed.

Approve for the queue.

### comment 5932995617 — 2026-10-01T13:59:13Z — Casey-allard

Merged as `e126867f`. The squash tree `d51ca07bb9b8aba24995a25479bdabddb6bb8d40` equals `git merge-tree --write-tree` of the parent `e8f3cb1d` and the reviewed head `ee19acae`.


---
_Comment/review ids in this file: 5932995617, 5380288590_

