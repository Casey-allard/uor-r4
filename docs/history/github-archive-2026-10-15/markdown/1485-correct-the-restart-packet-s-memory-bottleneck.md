# #1485 — Correct the restart packet's memory bottleneck

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-29T01:50:35Z
- **Updated:** 2026-10-09T13:08:57Z
- **Closed:** 2026-09-29T01:56:20Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1485
- **Merged:** 2026-09-29T01:56:20Z
- **Merge commit:** 66235490a18839d98482b8f9fdbe1e0b3cf5009f
- **Base / Head:** main / UOR-Foundation:lab/claude/packet-correction
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #973. Class C: Lab 1's documented decision in the ROADMAP packet. Documents only.

**What was wrong.** #1484's packet (merged `e3f91179`) said G v1's held-out failures come from keys built from surface tokens that do not survive paraphrase. In the D2/AERM relation world that cannot be so. The held-out templates in `crates/uor-r4-training/src/stack_aerm.rs` (`ASSERT_HELD`, `UPDATE_HELD`, `QUERY_HELD`, `PREVIOUS_HELD`, `REASSERT_HELD`) change only the carrier text, and the entity, relation and value slots are the same tokens at write and at query. The failures are therefore role-tagging or write-trigger failures, which Lab 2's `AddressCause` classes (`5f3de916`) measure.

**What changes.** In ROADMAP's "Bottleneck and next decision":
- the pre-registered reading of the decomposition, from its dominant cause to the next causal change;
- canonical geometric keys (G v2) placed on the synonymous-mention problem they address;
- a corrected Lab 2 deliverable.

The correction and the full decision table are posted on [#973](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5882099700) at 01:49 UTC.

**Checks:**
- `python3 scripts/check_claim_wording.py` passed.
- The templates cited were read from `main` at `e3f91179`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5882155100 — 2026-09-29T01:54:58Z — Casey-allard

Independent review (Sonnet pass launched by Lab 1, non-author)

**Verdict: APPROVE**

Checked read-only against `origin/main` and `origin/lab/claude/packet-correction`; no edits/commits/cargo runs performed.

**1. Held-out vs. training templates in `crates/uor-r4-training/src/stack_aerm.rs` (`git show origin/main:...`)**
- `Part` enum (`Text`/`Entity`/`Relation`/`Value`, lines 329–335) is the slot vocabulary. Each `*_HELD` array keeps the identical `Entity`/`Relation`/`Value` slot sequence as its matching `*_TRAIN` template and changes only `Text(..)` carriers:
  - `ASSERT_HELD` (368–387) vs `ASSERT_TRAIN` (339–367): e.g. `Text("Here is a fact:")` vs `Text("Note that")`, same `[Entity, Text("'s"), Relation, Text(" is"), Value, Text(".")]` skeleton.
  - `UPDATE_HELD` (417–425) vs `UPDATE_TRAIN` (388–416), `REASSERT_HELD` (446–454) vs `REASSERT_TRAIN` (426–445), `QUERY_HELD` (472–478) vs `QUERY_TRAIN` (455–471), `PREVIOUS_HELD` (495–501) vs `PREVIOUS_TRAIN` (479–494): same pattern in each case.
- `render()` (816–863): the `Entity`/`Relation`/`Value` arms (837–858) push `entity.id` / `relation.word.id` / `value.id` directly — token identity does not depend on which `Text` pieces (i.e. which template/carrier) surround them.
- `build()` (1016–1273): a given `(p, r)` pair uses the *same* `entity = &people[p]` and `relation = &world.relations[r]` for both the write (1073–1082, template drawn from `self.templates(ASSERT_TRAIN, ASSERT_HELD)`) and the later query (1207–1219, from `self.templates(QUERY_TRAIN, QUERY_HELD)`/`PREVIOUS_*`). The query's gold `key` is set at line 1255 as `(entity.id, relation.word.id)` — taken directly from those shared ids, not reconstructed from rendered text — and `RelationStore::write`/`read` (158, 203) key on exactly `(entity: u32, relation: u32)`. So the gold query key is definitionally equal to the write key regardless of which carrier/template (train or held-out) was used.
- This fully substantiates the claim: held-out templates differ only in carrier text, keep the same slot kinds, and the entity/relation tokens are identical at write and query.

**2. ROADMAP text matches (1), and the cited comment is real**
- `gh pr diff 1485`: the new "Bottleneck and next decision" text says "held-out templates change only the carrier text, and the entity and relation tokens are the same at write and at query. The failures there are therefore role-tagging or write-trigger failures, not surface-form mismatches" — matches finding (1) without overclaiming further.
- `gh api repos/UOR-Foundation/uor-r4/issues/comments/5882099700 --jq .created_at` → `2026-09-29T01:49:38Z`, i.e. 01:49 UTC, matching both the PR body and the ROADMAP citation. Comment body independently reconstructs the same template/key argument.
- Bonus cross-checks: `gh pr view 1485 --json files` shows only `ROADMAP.md` changed (docs-only, as expected). The PR body's cited source commit `e3f91179` is a real ancestor of `origin/main` (`git merge-base --is-ancestor` confirms; title "Record Lab 1's resumed leadership and the restart packet (#1484)"). The Astra-handback comment `5881962511` cited in the packet exists at `2026-09-29T01:35:25Z`, matching "01:35 UTC" in the text.

**3. No leftover retracted hypothesis in "## Lab 1 leadership and restart packet"**
- Read the full section (lines 11–101 of `ROADMAP.md` on `origin/lab/claude/packet-correction`, via `git show origin/lab/claude/packet-correction:ROADMAP.md`). The only surviving key-side language is the corrected framing ("Memory fails at address formation", "Why surface-form keys are not the cause", "Canonical geometric keys (G v2) address a different problem, different surface forms..."), consistent with the correction.
- `grep -in "surface.token|wrong key|key side|formed from language|survive paraphrase|Unavailable"` over the whole file matches only unrelated `UNAVAILABLE` energy-measurement text at lines 517/595/725, well outside this section. No sentence in the section still asserts the retracted surface-token-paraphrase hypothesis.

All three checks pass with direct evidence; no required fixes.



---
_Comment/review ids in this file: 5882155100_

