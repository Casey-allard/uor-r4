# #1934 — Retract the reaping claim; refusal template is greedy-mode collapse (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T17:08:29Z
- **Updated:** 2026-10-09T13:22:01Z
- **Closed:** 2026-10-08T17:10:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1934
- **Merged:** 2026-10-08T17:10:23Z
- **Merge commit:** 711eff1f20f7b5de32030888d3fa68ccf5e1a4d6
- **Base / Head:** main / UOR-Foundation:deepseek/sampling-correction
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Retract the reaping claim, and measure the refusal template as greedy-mode collapse

RETRACTION: round 72 reported that report directories and artifacts on the pod were
disappearing between commands and called it the session's most dangerous failure mode. That
was wrong. A directory listing shows every file present - sampled/chat.json at 53,011 bytes,
export-r5b/model.lut at 4,619,588 bytes, and panel, panel-r5, probe-r5 and p3 directories
intact. The failed reads were my own quoting bug: Python was passed inside a quoted heredoc
that does not expand variables, so the path arrived empty and the interpreter looked for a
relative path - which the error text said plainly and I read past. Retracted in full.

THE EXPERIMENT: round 58's panel under greedy decoding produced the refusal shape "I do not
have access to ... However, I can provide ..." repeatedly. The same artifact, panel and
protocol decoded at temperature 0.7 with top_k 40 puts it in 1 of 22 replies. The refusal
template is largely greedy-mode collapse rather than a fixed model property.

The trade is visible though: sampled replies are less coherent, while the correct factual
ones stay correct, so the distribution is broad with a noisy tail. That means the refusal
template should not be attacked as a training defect - sampling trades it away - and what
sampling exposes is coherence and accuracy, which is what the 19.9M arm tests.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

