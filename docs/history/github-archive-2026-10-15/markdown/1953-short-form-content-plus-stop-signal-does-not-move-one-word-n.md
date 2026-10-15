# #1953 — Short-form content plus stop signal does not move one-word/numeric compliance (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T19:13:45Z
- **Updated:** 2026-10-09T13:19:30Z
- **Closed:** 2026-10-08T19:15:26Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1953
- **Merged:** 2026-10-08T19:15:26Z
- **Merge commit:** d31015900c2e694fc2604397a5d641a7ead32146
- **Base / Head:** main / UOR-Foundation:deepseek/combo-result
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Adding short-form content to the stopping signal does not move the one-word or numeric classes

The first run with both halves present: policy=full_prefix, the hash-proven mask,
UOR_TERMINAL_WEIGHT=60, and an 82.2M-token chat stream plus 20,000 synthetic documents whose
assistant turn is the correct answer. It took three format repairs before it would train - the
generator is its own binary rather than a mode, its mask omitted the literal Assistant marker
the trainer requires (20,000 markers inserted with mask 0), and the rebuilt store kept the old
UORT header count the mixer reads.

On panel v2 the combination reproduces the yes/no compliance and the factual openers, with
retention answering once rather than twice. Adding 20,000 short-form documents did NOT move the
one-word or numeric classes at all.

The specific explanation, recorded as a hypothesis: the synthetic documents supply short answers
in their own question forms but do not carry the INSTRUCTION to answer in one word - brevity as a
habit rather than form compliance as a demand. That is testable by regenerating with
demand-bearing user turns and re-running the same configuration.

What is now excluded: data in the wrong shape (this arm), capacity, optimization, decoding, the
prompt protocol, and raw data support. What remains is that the requested form must appear as a
DEMAND in the training data; if that fails too, the per-token objective is the last explanation.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

