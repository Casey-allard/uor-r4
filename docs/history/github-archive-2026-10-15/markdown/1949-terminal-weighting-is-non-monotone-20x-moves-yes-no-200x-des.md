# #1949 — Terminal weighting is non-monotone: 20x moves yes/no, 200x destroys it (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T18:38:25Z
- **Updated:** 2026-10-09T13:22:42Z
- **Closed:** 2026-10-08T18:40:08Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1949
- **Merged:** 2026-10-08T18:40:07Z
- **Merge commit:** 8e8c82c1f966a0b04a5573af907a9d3a3529bdf6
- **Base / Head:** main / UOR-Foundation:deepseek/terminal-dose
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Terminal weighting is non-monotone: 20x moves yes/no compliance, 200x destroys it

Same shape, corpus, mask, policy, steps and seed; the only difference is UOR_TERMINAL_WEIGHT,
which weights the response's terminating <|eos|> against ordinary response tokens.

At 20x, two of three yes/no turns open with "Yes" - the first compliance movement any arm in
this session produced - and retention answers "My name". At 200x the same class REVERTS to
"The sky", retention BREAKS to "The main", and list artifacts reappear in the text, with
development response nll rising from 2.1969 to 3.0944 at the same step.

That shows the intervention is real and tunable - a quantity that can be moved up and then back
down by one parameter is a knob, not noise - and that it is NON-MONOTONE: at 200x the loss is
dominated by stop tokens and everything else degrades. The optimum is interior to [20, 200].

The guards named in advance for exactly this failure mode - fluency and the retention pairs -
both fired, which is a pre-registered guard working rather than a post-hoc excuse. Development
nll rose with the dose without tracking quality, consistent with four earlier arms where it
failed to track behaviour.

The question is now "which weight maximises compliance without losing what the model does",
a one-parameter search with a fixed instrument, a stated guard and a known bad region. The 60x
midpoint is training.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

