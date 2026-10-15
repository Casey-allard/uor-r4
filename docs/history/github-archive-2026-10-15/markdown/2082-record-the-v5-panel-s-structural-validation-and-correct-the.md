# #2082 — Record the v5 panel's structural validation, and correct the prohibition that delayed it (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T20:09:01Z
- **Updated:** 2026-10-09T20:16:17Z
- **Closed:** 2026-10-09T20:12:25Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2082
- **Merged:** 2026-10-09T20:12:25Z
- **Merge commit:** 8e66e789dd850daebd2dd067e0989b1d94d5b41a
- **Base / Head:** main / UOR-Foundation:deepseek/v5-validation-record
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Record the v5 panel's structural validation, and correct the prohibition that delayed it (#2081)

THE PANEL IS VALIDATED, NOT CONDITIONALLY READY. chat-grade check on a release binary built from main
7d7cb5492 returns check_panel: pass for conversational-v5: 64 requests, 52 multi-turn, context 384,
EVERY row checked (multi_turn_memory/exact 40, unknowable_or_impossible/abstain_exact 24), no orphan
checks, checks sha256 737c4dfd..., and the context/token-position worst case conv-v5-mem-031 at 358
positions inside 384 - the check the panel record listed as outstanding.

All check-only controls behave as designed: expected_value 40/40, binding_swap 0/40, echo_last 0/40,
echo_history 0/40, copy_first_stated 20/40 and copy_last_stated 20/40 (neither vacuous), memory
constants 0/0/0 of 40, unknowable constants 24/0/0 of 24, adversarial abstentions 0 in both categories,
unknowable expected_value 0/0.

MY ERROR, RECORDED SO IT IS NOT REPEATED: the panel brief prohibited chat-grade outright, and that
was wrong. chat-grade check is a STRUCTURAL validation with NO model - it runs the panel's
context/turn check against a tokenizer and validates the row checks. The prohibition made the panel
sit at 'conditionally ready' for a round while the authoritative validation was available.
Two argument facts also recorded because they cost a step: the subcommand takes requests=, not panel=,
and it takes no out=.

The record and current-state now also carry the DECLARED ACCEPTANCE RUN, stated in advance so it
cannot be improvised later: chat-grade grade-replies ... checks=data/panels/conversational-v5-checks.tsv,
read check_pass per category with every failing row named, target >= 34 of the 40 memory rows,
acceptable secondary and non-gating because the judge is not deterministic. Criterion 1 is still NOT
met; the run happens ONCE, and the first model that replies makes v5 development evidence exactly as
v4 became.

References #2029


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

