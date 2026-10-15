# #1941 — Instruction-precision review: mechanism correction, episode filter, 121:1 form ratio (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T17:31:01Z
- **Updated:** 2026-10-09T13:20:43Z
- **Closed:** 2026-10-08T17:32:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1941
- **Merged:** 2026-10-08T17:32:57Z
- **Merge commit:** 9307cc4b6a5c5a9346dd12594b4db077526ffff5
- **Base / Head:** main / UOR-Foundation:deepseek/instruction-review
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Independent review of instruction precision: a mechanism correction, the eligibility filter, and a 121:1 form ratio

CORRECTION to my own published mechanism. response_region_policy_verdict explains the
role_only bullet collapse as over-weighting the corpus's bullet style. That is wrong: both
policies zero-weight every prefix token, and the real difference is that RoleOnly DELETES THE
USER TURN, keeping only BOS and the assistant-marker ids. The bullets are therefore the
corpus's UNCONDITIONAL assistant form, and reply form is carried by the prefix - which is
exactly why full_prefix works and role_only cannot. The round-58 conclusion stands; its
published mechanism does not.

CORRECTION to the mask fraction: the official mask is 68.8906 percent of tokens, not the 71.1168
I published from the pre-fix scan rule, and not the 69.68 later figure either.

NEW AND DECISIVE: the episode filter. An episode is admitted only if BOS to end-of-response
fits the context, which at 384 admits 31,798 of 129,465 responses (24.56 percent) - and among
the eligible population there are FOUR one-word replies and EIGHT of five words or fewer. The
effective training distribution contains four demonstrations of the behaviour the panels score.

The form ratio is 121:1 against short answers: 142 contract turns against 17,205 expand turns;
26 one-word replies in 129,465; and 2,042 "how many" turns with zero five-word replies. The
corpus's own modal yes/no answer is a short answer plus a long justification, so the model's
"Yes, fish live in trees..." is its conditional mode rather than a contract failure.

THE RIVAL EXPLANATION, which the data story does not cover: token-level MLE length bias. After
"Yes." 31 continuations run a median 95 words while 16 stop, so continuation carries about
250:1 of the token mass although 22 percent of turn types stop. That explains why the first
token is right for yes/no and the second is not.

Two frozen-panel items are also weakly posed - one contains its own answer in the prompt, and
the first-token probe reads the separator, making it the wrong instrument for a second-token
failure.

The cheapest decisive experiment is now queued: policy=truncated_prefix on the unchanged
corpus and mask, one argument, which admits 2.14x the episodes and ALL 118 short-form replies
(14.75x). The falsifier is recorded before the run.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

