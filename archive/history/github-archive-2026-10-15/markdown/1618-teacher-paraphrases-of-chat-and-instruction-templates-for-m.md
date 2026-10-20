# #1618 — Teacher paraphrases of chat and instruction templates for M-world training corpora

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T14:20:04Z
- **Updated:** 2026-10-09T13:08:46Z
- **Closed:** 2026-10-02T14:41:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1618
- **Merged:** 2026-10-02T14:41:33Z
- **Merge commit:** b79e30bc1fa94f9b9db899f4303c7e4bfde1dd5b
- **Base / Head:** main / UOR-Foundation:codex/d19-instruction-paraphrases
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #1552

**Why.** emit-5c answers chat and instruction skills on their trained phrasings and fails them on the held-out development phrasings:

| Intent | Train phrasings | Development phrasings |
|---|---|---|
| first_letter | 6/6 | 0/11 |
| next_day | 28/28 | 4/35 |
| identity | 42/42 | 11/32 |

Each skill has only 3–10 training phrasings (the work card on #1552 has the details). This PR adds the data path for teacher paraphrases of those training templates. It contains no model result; the paired run is reported on #1552.

**Changes:**
- **`teacher-paraphrase set=chat`.** Every social, share, fact and instruction training template (95) goes to the local offline teacher.
  - Slots (`{x}`, `{a}`, `{b}`, `{n}`, `{v}`) must be kept exactly, each once.
  - The development screen is unchanged: a line is dropped if it equals a development template or shares a four-word sequence with one.
  - `set=relation` (the default) behaves as before, apart from the generalized slot check.
- **`m-world corpus paraphrases=A.jsonl[,…] paraphrase_share=0.5`** (world v2 only; refused for v1 before the root is claimed).
  - The table is installed once per process. A training pick of a template with wordings draws a wording in `share` of its draws.
  - Development picks never read it, and templates without wordings draw exactly as before.
  - The installer fixes v2's reserved words first, so generated values do not change.
  - Sources must be training templates, texts must keep the slots, and no text may equal a world template.
  - The corpus manifest records the files (sha256, rows), the share and the number of templates with wordings.
- **`MWorld::chat_tables()`** lists the non-relation tables by intent name.

**Checks:**
- `cargo fmt`.
- New test `milestone_world_v2::tests::training_paraphrases_are_validated_and_only_change_training_picks`, which covers:
  - validation;
  - development picks and templates without wordings drawing identically with and without the table;
  - share 0 and share 1.
- `cargo test -p uor-r4-training --lib milestone_world`: 45 passed, 1 ignored.
- A teacher probe on two templates with the release binary.
- GitHub evidence: `codex/ci/v3/instruction-paraphrases`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### review 5393108745 — 2026-10-02T14:40:39Z — Casey-allard — COMMENTED

Exact-head review at e0670e6f5954dd45ecbe82f19a32b74dfe3b0c1b (Claude lab; self-review per owner rule on #820). Checked: the diff touches only m-world.rs, teacher-paraphrase.rs, milestone_world.rs and milestone_world_v2.rs. Development picks never read the table, and templates without wordings draw identically (unit test). The installer fixes v2 reserved words before installing. Relation mode is unchanged apart from the generalized slot check. Evidence run https://github.com/UOR-Foundation/uor-r4/actions/runs/37020026841 on codex/ci/v3/instruction-paraphrases (parent = this head): fmt, check all targets, lib 561 passed / 0 failed / 6 ignored, m-world example tests pass. Actual use: corpus para-p2-train built with 94 templates with wordings; the paired result is on #1552.

### comment 5954871193 — 2026-10-02T14:42:27Z — Casey-allard

Merged as b79e30bc. All four changed files on main equal the reviewed head e0670e6f (blob comparison). Result using this path: emit-6p, the new D19 best (total 873 vs 819), recorded on #1552.


---
_Comment/review ids in this file: 5954871193, 5393108745_

