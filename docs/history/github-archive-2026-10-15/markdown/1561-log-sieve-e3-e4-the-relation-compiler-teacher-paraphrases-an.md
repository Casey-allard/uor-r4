# #1561 — Log-sieve E3/E4: the relation compiler, teacher paraphrases and recall=route

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T04:14:52Z
- **Updated:** 2026-10-01T08:31:19Z
- **Closed:** 2026-10-01T05:03:54Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1561
- **Merged:** 2026-10-01T05:03:54Z
- **Merge commit:** b06b46a35142c219a3946209c3ffa03db8499e22
- **Base / Head:** main / UOR-Foundation:codex/log-sieve-e3
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Source for E3 of the log-sieve design (#1559 §2.3), the relation compiler, with its two recorded negatives.

**What it adds:**
- **`relation_compiler`.**
  - It labels every M-world v2 user turn from the generator's typed intent: the relation is one of `relation_names()` or none; the act is assert, update, query or none, where an `_absent` query counts as a query.
  - It fits softmax heads, by full-batch gradient descent on standardized f64 features with no randomness, on turns with **training** phrasings. They are scored on **development** phrasings.
  - The features are the frozen trunk's final states for the turn read alone. Word features serve as a control.
- **`m-world compiler`** reports relation accuracy on relation turns and act accuracy on all turns per development cell, plus the E3 gate. `lexical=false` skips the control for a second trunk on the same draw.
- **`milestone_world_v2::relation_names()`**, a read-only accessor. The instrument is unchanged.

**Results (#1552, cards v8 and v9). Gate: relation ≥ 0.9 and act ≥ 0.95 on dev × dev. Not met:**
- `emit-1` 2.1M trunk: relation 0.688, act 0.779.
- R1 7M trunk: relation 0.714, act 0.702.
- Lexical control: relation 0.788.
- The failures are new-word paraphrases. The owner chose teacher paraphrase data next.

**Checks run:**
- `cargo fmt --check`.
- `cargo test -p uor-r4-training --lib relation_compiler`: 3 passed (intent labels, a deterministic and separating fit, the lexicon ignoring unseen words).
- Release build, and both real runs above.
- The exact-head GitHub run is on `codex/ci/v3/log-sieve-e3`.

References #1552

🤖 Generated with [Claude Code](https://claude.com/claude-code)

---

**Added since (owner ruling: teacher paraphrase data):**
- **`teacher-paraphrase` (bin).** Local SmolLM2-Instruct, offline and as a data source only.
  - It rewrites each relation template the world draws with **training** phrasings, using a worked example on a non-M-world topic (a car) and sampling with a fixed seed.
  - The shared prompt prefix's teacher state is computed once and cloned. Generation stops at the end of the first block of rewrites, and only that block is parsed.
  - A line must keep `{v}` exactly as its template does. It is dropped if it equals a development template or shares a four-word sequence with one.
  - It is built with `--features reference-accelerate` (the #804 local-inference exception) for speed. The report root is claimed and sealed.
- **`m-world compiler paraphrases=…`** adds those wordings to the training turns, filling each `{v}` with that relation's training values, recovered by matching training turns to their templates. Development turns are untouched.
- **Run (card v10):** 373 kept and 20 screened, from 84 templates with SmolLM2-1.7B-Instruct.
- **Checks:** `relation_compiler` lib tests 4/4, including paraphrase filling and value recovery.

**Added since (owner: measure end to end, then push):**
- **`recall=route` in `m-world evaluate`:** the identity channel (R-sieve), then the relation channel's `RelationRoute`.
  - The route is a sparse word table at standardized scale (`Lexicon::scaled`), naming relation and act.
  - The value comes from the latest earlier user turn named as the asked relation.
  - Options:
    - `route_paraphrases=`: a list of paraphrase files;
    - `route_acts=fact|any`: whether the lookup needs an assert or update act;
    - `route_trunk=`: standardized trunk features appended to each row.
- **`m-world compiler`:** `combined` (trunk + words) and `table` heads, and a paraphrase file list.
- **Results (#1552, cards v12–v17; development cell; diagnostic, not a gate claim):**
  - The best route, v16, reaches MQAR **106/109** and open relation **36/52**, against the best trained read at 27/52.
  - Relation naming reaches 0.949 with the dense combined head after 776 teacher paraphrases.
  - E3's declared trunk-head gate is **not met** (0.836, act 0.766).
  - v17 (trunk features in the sparse table) is a recorded negative.
- **Checks:** `relation_compiler` lib tests 5/5, including the route's latest-statement lookup and trunk-feature refusals, and all runs above.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5927734312 — 2026-10-01T08:31:19Z — Casey-allard

Merged as b06b46a35142c219a3946209c3ffa03db8499e22 at 05:03 UTC (queued from the shared account, not by the Claude lab). Verified: `git merge-tree --write-tree 8d0373a5 d2b680f1` equals the squash tree (53533646). The exact-head run at `d2b680f1` ([36820078335](https://github.com/UOR-Foundation/uor-r4/actions/runs/36820078335)) later passed: lib 415/0, m-world 5/0. **The four commits pushed after the merge** (`9f89bc53` recall=route, `caedf3ca` scaled table and combined heads, `0cbb94e1` route_acts, `014fcbcf` route_trunk) **are not in main.** They follow in a new PR.


---
_Comment/review ids in this file: 5927734312_

