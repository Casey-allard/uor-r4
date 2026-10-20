# #1940 — Run card: synthetic short-form split for the missing answer-form tail (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T17:26:57Z
- **Updated:** 2026-10-09T13:22:18Z
- **Closed:** 2026-10-08T17:28:38Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1940
- **Merged:** 2026-10-08T17:28:38Z
- **Merge commit:** 37dccbf0c7cbe1fe53565eb5922aadb7d0235b95
- **Base / Head:** main / UOR-Foundation:deepseek/short-form-runcard
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Run card: a synthetic short-form instruction split for the missing answer-form tail

Rounds 79-80 established that instruction precision is not learnable from the existing
material: the corpus median response is 350 tokens with 0.0 percent at three tokens or fewer,
and every ingredient source is prose-dominant with medians from 45 to 701 tokens and none
reaching 0.3 percent at ten tokens or fewer. What is missing is not knowledge or register but
the ANSWER FORM.

This run card specifies a synthetic split that supplies exactly that, using no new source and
no licence decision: the project's own literal-role protocol 2 format with the role-marker ids
the corpus already uses, a response mask following the rule proven bit-identical to the
builder's mask_sha256 in round 80, and the manifest schema stack_dialogue.rs:53-80 requires.
Targets are one-word, yes/no, numeric, short-list and short-factual answers, generated
programmatically with varied wording so the model learns the instruction rather than a
template.

It is used as a mixture with the conversational corpus - small weight and larger weight as two
arms - and judged by the two frozen instruments, with fluency guarded at its 22 of 22 baseline
and a new template habit named in advance as the failure mode to watch, since that was exactly
how role_only failed.

If compliance stays at 0 of 6 with fluency intact at a meaningful mixture weight, the reply-form
hypothesis is wrong too and the remaining candidate is the objective's granularity. That is a
decisive and cheap outcome.

No split has been generated and no arm trained; the value of the document is that format,
verification rule, instruments and falsifier are fixed before the work starts.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

