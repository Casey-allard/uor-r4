# Paper 1 Peer Review

Reviewer role: skeptical but fair `cs.LG` reader, reading for scientific merit,
category fit, and possible rejection routes that are not merely formatting.

Manuscript reviewed:
- [main.tex](/Users/adminamn/ai-router/paper1_arxiv_rebuild_v4/main.tex)
- [main.pdf](/Users/adminamn/ai-router/paper1_arxiv_rebuild_v4/main.pdf)

## High-level judgment

The manuscript is now professionally presented and reads like a legitimate
research preprint. I no longer see an obvious form-only rejection path. The
remaining questions are substantive:

1. Is the proxy strong enough to support the proposed routing claim?
2. Is the bridge from proxy behavior to trainable LM behavior persuasive?
3. Is the paper's narrow claim strong enough to justify `cs.LG` attention?

## Recommendation

If evaluated only on professionalism and scholarly form:
- `accept as academically presentable`

If evaluated on scientific strength:
- `borderline but defensible`

The case for acceptability is that the paper is unusually honest about its null
result, presents real controls, and makes a narrow claim rather than pretending
to have solved sparse routing outright. The case against it is that the positive
evidence is strongest on the proxy, while the trainable model largely converts
the story into transfer-plus-null rather than transfer-plus-win.

## What now looks solid

- The title, abstract, and section flow are now consistent with serious
  `cs.LG` preprints.
- The method is defined mathematically and visually instead of merely named.
- The controls are visible and scientifically relevant.
- The null result against permuted fixed routing is central rather than buried.
- The limitations are explicit and credible.

## Main substantive concerns

### 1. The paper's strongest evidence is still proxy-side

The core positive result is the sublinear scaling law on the static semantic
proxy. That result is supported by controls and is scientifically meaningful.
However, the trainable-model section does not extend that result into a
geometry-specific trained-model advantage. Instead, it shows that a fixed route
map remains usable while the Hopf-vs-permuted distinction disappears.

Implication:
- A skeptical reviewer can fairly say the paper demonstrates a geometry-linked
  routing effect in structured normalized representations more strongly than it
  demonstrates an end-to-end language-model routing advance.

### 2. The fixed 4D chart remains a natural target for reviewer pressure

The deterministic `(3,65,2,21)` chart is now clearly explained, which is good.
But it still creates a natural reviewer question: how much of the result is
specific to this frozen slice?

This is not a flaw in presentation. It is the most natural methodological
question that follows from the paper's design.

### 3. The trainable model is clearly small-scale

The paper is honest about this. Still, the trainable experiment is only strong
enough to support the claim it already makes: the route map transfers into a
small trained model, but not as a geometry-specific winner.

That means the paper is strongest as:
- a well-controlled routing study,
not as:
- a general claim about replacing learned sparse routing in modern LLMs.

### 4. The proxy itself is bespoke enough that some readers will pause

The proxy design is understandable and now cited properly, but it is not
standard in the effortless sense that a mainstream benchmark study would be.
The PTB-derived PPMI-SVD plus WT2-window pooling construction is reasonable,
yet still unusual enough that some readers may hesitate before granting it as a
canonical geometry testbed.

## Category fit: `cs.LG`

I think the paper still fits `cs.LG`.

Why it fits:
- it studies routing in learned representations,
- it includes trainable-model experiments,
- it is method-focused and empirical,
- and it addresses general machine-learning questions about sparsity,
  inductive structure, and representation geometry.

Why it may still feel atypical:
- the paper is more mathematically motivated than many routing papers,
- the main positive result is not a standard benchmark win,
- and the strongest claim is not scale-driven.

That makes it non-mainstream, but not out-of-category.

## What I would expect from a fair skeptical reviewer

Likely fair comments:

1. "The paper is well written and the controls are thoughtful, but the main
   positive result is proxy-side."
2. "The null result is scientifically useful, yet it narrows the trained-model
   claim considerably."
3. "The fixed chart selection deserves more sensitivity analysis in future
   work."
4. "The work is interesting as a geometry-based routing study, but it is not
   yet evidence for replacing learned sparse routing in practical LLMs."

Those are serious scientific comments, but they are not dismissals.

## What I would *not* consider a fair criticism anymore

The following would no longer be justified:

- "This looks like an amateur memo."
- "The figures and tables are sloppy enough to reject on form."
- "The manuscript is not written like a real ML paper."
- "The paper hides its negative result."
- "The paper uses undefined jargon without technical grounding."

## Bottom line

The manuscript is now in the right state for submission judgment:

- If it is rejected, the rejection should be about the strength or scope of the
  evidence.
- It should not be rejected because it looks unprofessional or non-scholarly.

That is a major improvement over the earlier submission state.
