# #1998 — Token-id route freezes the pointer: reachability fixed and 61% lower (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T04:30:33Z
- **Updated:** 2026-10-09T13:21:59Z
- **Closed:** 2026-10-09T04:32:12Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1998
- **Merged:** 2026-10-09T04:32:12Z
- **Merge commit:** 318a77051c78e5df9727e5ac1b3216b25c7c164e
- **Base / Head:** main / UOR-Foundation:deepseek/route-frozen
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The review recommended lever reached as configuration: the trainer pointer route derives keys from TOKEN IDS instead of hidden states. Flag validated with a 20-step smoke test (corrected once - the key window is 1 to 6 tokens). THE MEASUREMENT THAT MATTERS: the pointer quantities are FROZEN - hit rate 0.16116248348745046 and reachable 0.18185821224130339 identical to fifteen decimal places from step 3,000 through 7,000, where the unrouted arm moved every evaluation (0.1079 rising to 0.2387, reachability 0.4729). And reachability is 61 percent LOWER. So the route keys are deterministic and the pointer reach is a property of the admission rule rather than of learning: the route admits by shared prime factors or n-gram overlap, and a value sharing no n-gram with the question asking about it has a fixed admission set training cannot enlarge. That separates a token-identity KEY, which the identity mixture wants, from a prime/ngram ADMISSION RULE, which novel-value recall does not want. The token-id route is not the fix as configured; the learned keys should be kept. NEXT: keep the learned pathway and add a TOKEN-IDENTITY MATCHING TERM - scoring candidates by whether their tokens appear in the premise - a bounded change to the pointer score, judged by the 28-row novel-value rate with its 0.000 control, the gate stratification and the value-versus-prior rows. The recall result is untouched: 0.417 against 0.000 at every budget. Sixth route explored for binding, and the first whose failure mode is visible in the training curve itself. References #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

