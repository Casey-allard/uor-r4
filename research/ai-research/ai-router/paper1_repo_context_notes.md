# Paper 1 Repo Context Notes

## Why this paper exists now

The repo’s own publication documents are clear that this paper is being published now because the narrow empirical claim is complete and because TurboQuant created urgency around timestamp priority.

Canonical sources:

- [PUBLICATION_STRATEGY.md](/Users/adminamn/ai-router/router-research/docs/research/PUBLICATION_STRATEGY.md)
- [PUBLICATION_PACKET.md](/Users/adminamn/ai-router/router-research/docs/research/PUBLICATION_PACKET.md)
- [ACTIVE_STATE.md](/Users/adminamn/ai-router/router-research/docs/research/ACTIVE_STATE.md)
- [SUBMISSION_CHECKLIST.md](/Users/adminamn/ai-router/router-research/papers/SUBMISSION_CHECKLIST.md)

The repo’s internal publication logic is:

1. TurboQuant independently validated the underlying geometric fact at production scale.
2. This repo has a complementary routing result based on the same angular non-uniformity.
3. The narrow claim is fully supported enough for public priority.
4. The larger H^4 × H^4 architectural thesis is not ready for public headline treatment.

So the paper is not “the whole program.” It is the first public stake in the ground.

## What the repo says the safe claim actually is

The publication strategy freezes Paper 1 to Tier 1 + Tier 2 claims only.

### Tier 1: proxy-side geometric claim

- normalized embeddings are angularly non-uniform on a semantic proxy
- fixed Hopf-derived routing yields a sublinear effective routing footprint
- the canonical law is `eff_buckets = 2.957 * K^0.572`
- the advantage persists through large `K`
- the effect is angular and norm-invariant
- fixed 4D Hopf geometry beats adaptive high-dimensional `K`-means on structured embeddings

### Tier 2: toy-LM transfer claim

- fixed routing can replace a learned top-1 gate in a 2-layer toy LM at about 6–8% PPL cost
- this holds across PTB and WikiText-2
- the correct learned comparator is no-aux top-1 gating in the native concentration regime
- Hopf and permuted fixed routing are indistinguishable after end-to-end training

### Mandatory limitation disclosures

- the geometry-specific advantage disappears in the trainable LM
- the LM evidence is toy-scale
- the proxy and trainable settings test different things
- hardware-proxy evidence is simulated rather than physical

## What the repo explicitly says must not be the headline

The publication strategy is unusually explicit about what must be withheld:

- H^4 × H^4 product-manifold framing
- phase transport as an independent contribution
- spectral/operator story
- sparse event routing as a primary architectural claim
- 10×–100× hardware-savings claim
- broad replacement of transformers or modern sparse routing systems

That means the paper must feel substantial without relying on those withheld ideas to look ambitious.

## What the old paper lineage did wrong

The old manuscript materials, especially:

- [ANGULAR_MANIFOLD_ROUTING_PAPER.md](/Users/adminamn/ai-router/router-research/docs/research/ANGULAR_MANIFOLD_ROUTING_PAPER.md)
- [PAPER_SKELETON.md](/Users/adminamn/ai-router/router-research/docs/research/PAPER_SKELETON.md)
- [router-research/papers/main.tex](/Users/adminamn/ai-router/router-research/papers/main.tex)

were trying to do two different things at once:

1. secure priority quickly
2. tell the larger program story

That produced several problems:

- internal increment language leaked into the paper
- the manuscript read like a polished project memo
- the TurboQuant relationship was over-explained in a way that made the paper sound reactive
- support material was accumulated instead of curated
- the target length was too small for the amount of explanation the method needs

## What the rebuild must preserve from the repo

### Preserve

- the narrow Tier 1 + Tier 2 claim boundary
- the urgency and context created by TurboQuant
- the null result as a central honesty signal
- the comparison regime choice justified by the auxiliary-loss control

### Do not preserve

- the internal “increment chain” tone
- claims about the whole architecture program
- the old 4–5 page technical-note mindset if it causes amateur compression
- table and figure ideas whose only function is to narrate the project’s history

## What the paper is really about, in repo terms

If rewritten correctly, the paper’s point is:

> A fixed routing map derived from angular structure can induce a sublinear routing footprint on structured embeddings, and that same fixed scaffold partially transfers to a toy trainable language model; however, the trainable evidence does not show a geometry-specific advantage over random fixed routing after end-to-end learning.

That is the paper.

Not the entire program.
Not the H^4 × H^4 architecture.
Not the 10×–100× hardware dream.
Not the spectral story.
Not “we may not need transformers at all.”

Those ideas may motivate why publication matters now, but they cannot be the paper’s public claim.
