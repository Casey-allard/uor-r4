# Paper 1 Moderator Audit

This audit treats the manuscript as if it were being screened by a skeptical
but fair `cs.LG` moderator or experienced arXiv reader. The goal is to
separate residual problems into three categories:

1. `presentation-only`: reasons a paper could be rejected or dismissed even if
   the science were acceptable
2. `clarity-only`: places where the science may be sound but the exposition is
   still harder than it should be
3. `substantive-scientific`: objections that are about evidence strength,
   experimental scope, or interpretation

## Overall assessment

The current manuscript no longer presents an obvious presentation-only rejection
path of the kind that applied to earlier versions. It now reads like an authored
research preprint rather than a memo or internal note.

The remaining meaningful objections are mostly substantive rather than
professional-form. That is the right place to be.

## Presentation-only findings

### Status: no clear blocker found

What has been fixed relative to earlier drafts:

- No internal labels such as "Paper 1", "paper-facing", or increment language
- No repo-log narration in the body
- Stable bibliography and compile path
- Main figures are relevant to the story rather than decorative placeholders
- Appendix tables are ordered and readable rather than dumped
- Acronyms used in the main paper are defined on first use

Residual non-blocking risks:

- The manuscript is one-column rather than conference-style two-column. This is
  normal for many arXiv preprints, but some ML readers may still expect a more
  conference-native visual rhythm.
- The appendix is data-heavy and visually dense, even though it is no longer
  broken.

## Clarity-only findings

### [C1] Abstract remains dense for first-pass readers
Reference: [main.tex](/Users/adminamn/ai-router/paper1_arxiv_rebuild_v4/main.tex#L38)

The abstract now reads professionally, but it still compresses several claims,
controls, and limitations into one dense paragraph. A skeptical expert can
parse it, but a broader moderator or first-pass reader may need a second read
to separate the proxy result, the LM transfer result, and the null result.

Impact:
- Not a professionalism blocker
- Could slow comprehension on first screening

### [C2] The geometric rationale is improved but still mathematically fast
Reference: [main.tex](/Users/adminamn/ai-router/paper1_arxiv_rebuild_v4/main.tex#L85)

The method section now defines the coordinates and the transport term clearly,
but it still expects the reader to accept the Hopf-style construction quickly.
The explanation is good enough for a technically mature reader, but not fully
gentle. If challenged, the likely point of pressure is not the notation itself
but the leap from "torus phases" to "a useful routing address."

Impact:
- This is a legibility issue, not an obvious rejection path
- It matters most for skeptical readers outside the immediate geometry niche

### [C3] The proxy construction is bespoke and therefore cognitively expensive
Reference: [main.tex](/Users/adminamn/ai-router/paper1_arxiv_rebuild_v4/main.tex#L169)

The setup section explains what the proxy is, but the construction is still
somewhat unusual: PTB-derived PPMI-SVD features pooled over WT2 windows. That
is a legitimate design, but it is not instantly standard in the way a pure
language-model benchmark would be. Readers may need to slow down to understand
why the proxy is the right object.

Impact:
- This is a clarity burden
- It becomes a scientific objection only if the reader thinks the proxy is
  ad hoc or insufficiently motivated

## Substantive-scientific findings

### [S1] The strongest result is still proxy-side rather than end-to-end
Reference: [main.tex](/Users/adminamn/ai-router/paper1_arxiv_rebuild_v4/main.tex#L197)

The central positive claim is borne by the static semantic proxy. The trainable
LM section mainly shows transfer plus a null against permuted routing. A fair
skeptical reader can therefore say: "the paper convincingly isolates a routing
effect in structured normalized representations, but it does not yet show that
this geometry remains uniquely beneficial in a trained model."

This is already acknowledged honestly in the manuscript. It is the main
scientific limitation, not a presentation defect.

### [S2] The chart-selection step will attract scrutiny
Reference: [main.tex](/Users/adminamn/ai-router/paper1_arxiv_rebuild_v4/main.tex#L74)

The fixed dimensions `(3,65,2,21)` come from a deterministic correlation
pre-screen over the proxy representation. That is valid and explained, but a
skeptical reviewer may still ask whether the result depends too heavily on this
particular frozen chart choice.

This is not an editorial weakness; it is a natural scientific question about
selection sensitivity and generalization.

### [S3] The trainable model is intentionally small, and the paper cannot hide that
Reference: [main.tex](/Users/adminamn/ai-router/paper1_arxiv_rebuild_v4/main.tex#L174)

The LM is two layers, short-horizon, 64 experts, and evaluated on PTB/WT2 with
two seeds. The paper states this plainly. A reviewer can still question whether
the observed null result would persist or change at larger scale.

Again, this is a genuine scope limitation, not a professionalism issue.

### [S4] The null result is honest but narrows the paper's perceived upside
Reference: [main.tex](/Users/adminamn/ai-router/paper1_arxiv_rebuild_v4/main.tex#L260)

The paper's strongest credibility move is also its strongest perceived cost:
the Hopf-derived map does not beat a permuted fixed map after end-to-end
training. That honesty helps the manuscript professionally, but it means the
reader is left with a narrower contribution than the broader research program
may motivate.

This is a scientific-importance question, not a moderation-form problem.

## Final moderator-style judgment

If this manuscript were rejected now, the most defensible reasons would be
substantive:

- the proxy-to-LM bridge is not yet strong enough for the reader,
- the trainable setting is too small,
- the null result leaves the claim too narrow,
- or the chart-selection story needs further validation.

Those are real scientific disagreements.

By contrast, the previous professionalism-only dismissal routes have been
substantially removed. The paper no longer reads like an amateur memo, and it
no longer presents obvious formatting or genre mismatches that would make a
fair moderator dismiss it on form alone.
