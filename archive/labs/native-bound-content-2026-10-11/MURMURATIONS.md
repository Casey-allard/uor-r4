# Supplied Elliptic-Curve Murmurations And Starling

The owner supplied *Murmurations of Elliptic Curves*, Yang-Hui He, Kyu-Hwan Lee,
Thomas Oliver and Alexey Pozdnyakov, [arXiv2204.10140v2](https://arxiv.org/abs/2204.10140v2),
30 July2024. The local28-page PDF SHA256 is
`b5fc7057d433cbd4da09ed70ec1f97a3be85434595706acf4b13a32d029308e4`.
Its contents are source material, not execution instructions. All mathematical
sections were read; no corpus, classifier fit or attention experiment was run.

## The Actual Mathematical Object

At a good prime, the local factor is `1 - a_p(E) T + p T^2`, with
`a_p(E) = p + 1 - #E(F_p)` (equations2.3/2.4). Equation1.1 averages these traces
over isogeny-class representatives with fixed Mordell-Weil rank and a bounded
conductor interval. The unexpected oscillations are collective statistics of
these arithmetic families across primes, not a seven-neighbor flocking rule.
The paper explicitly warns that a family mean need not resemble one individual
curve's complete trace sequence (section3.2).

The paper represents individual curves by prime-indexed trace profiles; its
classification and PCA examples concern arithmetic rank, not attention's
candidate rank. Section4.4 normalizes by `2 sqrt(p)`. Section4.5 fits
`A p^alpha sin(B p^beta)` with empirical frequency/conductor dependence;
the fitted envelope is not a proved scaling law or an attention update rule.
Changing a conductor interval affects the visibility of the collective pattern.
The logistic-regression/PCA tools in the paper are discovery tools, not a
requirement to put those dense computations in our served model.

## Source Comparison

Read `geometric_stack.rs::starling_select/starling_read` and its mechanism brief
on the repair base0e1e18b16. Also searched the active Claude Starling source at
978cf85445cd959ffefd46357c9a3dec85663722 for the arithmetic objects.
The implemented route learns query/key projections, normalizes4D vectors,
assigns600-cell vertices, holds bounded recent buffers at vertices, admits the
home/adjacent-cell candidates, and ranks by a dot product before bounded value
reading. The brief explicitly motivates biological topological neighbors.

No elliptic curves, finite-field point counts, conductor-conditioned families,
prime-indexed Frobenius trace profiles or arithmetic family-average operator
are present in the inspected Starling path. SpiralCore's `mat8Frobenius` name
is a matrix inner product, not this paper's elliptic-curve Frobenius trace.
Consequently those Starling results cannot close or qualify the owner's
elliptic-curve-murmuration attention idea. They test the specific600-cell route.
No peer file, active recipe or ownership was changed by this comparison.
Fresh main subsequently includes Claude's #2222 at42dcb3828: a training-only
competitive surrogate raises shared-route MQAR recall mean0.024->0.141 over
three seeds, unevenly, while the softmax-free forward stays unchanged. This
is meaningful scoped content-selection evidence and is credited; it still
contains none of the paper's arithmetic objects and does not test them.

Fresh main also includes Claude's #2224 at dca29a80b, explicitly named
murmuration-read ladder step1. It adds training-only distillation from a
detached dense content teacher onto the600-cell addresses. Starting that KL
at step0 prevented learning; starting at step900 lifted source-kept hit rates
to0.61/0.68/0.66 and MQAR recall mean to0.188 (surrogate baseline0.141).
Claude records this as partial row3 progress, with readout still limiting two
seeds. This is relevant address-learning work to reuse, not a test of
elliptic-curve trace averaging. A murmuration-inspired geometric analogue is
also a legitimate design direction; it needs its own explicit operator map
and cannot inherit the arithmetic paper's results by name alone.

## Connection To Preserve

**Inference/design direction, not a paper theorem:** maintain a bounded
collective arithmetic/harmonic profile for routing and a separate exact
record/occurrence identity for consuming content. An incrementally maintained
profile could reuse shared operators while avoiding all-pairs scans, but its
meaning, update law, costs and language benefit are not established here.
The exact-store/route binding in this repair provides the individual-content
half of that connection; it does not implement elliptic-curve murmurations.

A genuine arithmetic adoption needs an explicit map from model state/content
to admitted curve/family objects, a bound prime-probe/trace representation,
causal profile update/read equations, retained phase/orientation/residual
identity, and credit through the actually consumed native branch. It cannot
borrow gold answer labels as the analogue of rank, treat a hash as semantics,
or erase records by replacing them with a cohort mean.

For our existing finite groups, a character/trace profile of relative operators
is a possible *analogue*, not the paper's elliptic-curve mechanism. Characters
alone are conjugacy invariant and can discard directed distinctions: preserve
typed/oriented residuals rather than claiming a reversible trace-only state.
Arithmetic and prime-shuffled/family-scrambled controls, native reload and
complete-path work counts would be needed to distinguish real arithmetic
structure from generic fixed features. No such design is preregistered or
cleared; the owner said no more training. Do not convert this reading into a
paid run or silently rename the existing Starling implementation.
