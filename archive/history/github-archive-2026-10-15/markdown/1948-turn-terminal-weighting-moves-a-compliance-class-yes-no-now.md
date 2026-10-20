# #1948 — Turn-terminal weighting moves a compliance class: yes/no now opens with Yes (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T18:27:26Z
- **Updated:** 2026-10-09T13:22:48Z
- **Closed:** 2026-10-08T18:29:08Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1948
- **Merged:** 2026-10-08T18:29:08Z
- **Merge commit:** afbdd89f68f34fe7685f8f8f200e0e76df85d84a
- **Base / Head:** main / UOR-Foundation:deepseek/terminal-weight-result
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The objective change moves a compliance class: turn-terminal weighting at 20x

The change of #1946 weights the response's terminating <|eos|> - identified by the mask proven
byte-identical to the builder's hash - by UOR_TERMINAL_WEIGHT, defaulting to 1.0 so existing
runs are unchanged. First arm: weight 20, the frozen 7.16M shape, policy=full_prefix, the same
corpus, mask and steps as every comparison arm, on a pod whose checkout and binary were
verified at the commit carrying the change.

On panel v2 read as the first two content tokens, TWO OF THREE YES/NO TURNS NOW OPEN WITH
"Yes" - "Yes, the" for the sky question and "Yes, ice" for the ice question, the latter
complying in form with the wrong fact, which is a different defect. That is the first time any
arm in this session has moved a compliance class: capacity at 2.8x, optimization at 2x,
sampling and the 14.75x data-support arm all left compliance at zero.

The pre-registered falsifier did not fire, because it was conditional on every factor producing
no compliance. The continuation-mass explanation is therefore supported rather than dead, and
the right next step is to escalate the intervention that showed an effect. Nothing was lost -
retention still responds with "My name" and the France opener is intact.

One-word and numeric turns are unmoved, still producing phrases rather than the requested token,
so the dose-response now running at weight 200 is what will say whether the other classes close
or whether terminal weighting reaches the yes/no class only.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

