# Principal review: preserve learned read confidence at the influence interface

September 21, 2026. Review of DeepSeek PR #1332 as submitted at `24c874ac4e600478f0a78b632c9c2c779eaa765c`, based on PR #1331 merge `e0f99fcc91b5063c5ad15c1799d47ab4525d65e7`. Three independent reviews covered source, saved evidence and mathematics; the principal pass also reconciled live resources, storage, knowledge and the roadmap. This review adds bounded solver/report repairs, not a new language-model result. [Numerical audit](../evidence/policy-obstruction-principal-review-2026-09-21.json); [next execution prompt](deepseek-reader-confidence-step-2026-09-21.md).

## Decision

**End the unchanged 32-bucket influence-policy campaign.** Its behavioral requirements are incompatible on the retained development population, even with all buckets free. The independent certificate uses exact integer counts, so it survives defects found in the generic floating-point optimizer.

**Next, restore the existing learned Read–NoRead advantage at the influence interface.** The current table discards a distinction the retained parent already computes. Carry that causal decision evidence, together with exact occurrence identity, into a small finite influence operator. Verify a parent-preserving construction witness first, then learn dose under the joint text/present/absence requirements and perform one qualifying fresh evaluation. This is a specified information repair, not an open-ended feature sweep or a reason to add dimensions.

## What is established by the saved counts

Development fit contains 197 final-present and 23 final-absent queries. The contextual parent emits 149 correct present answers and reads seven absent queries. The translated tolerance requires at least 145 correct answers and at most seven absent reads. All recorded present NoRead/weak/one-nat outcomes have zero correct emissions; only eight nats contributes.

We independently recomputed the multiple-choice dynamic program with integer arithmetic. For budget k, retain the greatest present-correct count after each bucket and each of its four actions, accumulating the corresponding absent-read count. This enumerates the count-feasible choices without enumerating all 4^32 tables or using CE approximations.

| Certificate on retained development counts | H4 | Categorical |
| --- | ---: | ---: |
| Required correct present answers | 145 | 145 |
| Maximum with absent reads <= 7, every bucket free | **74** | **71** |
| Maximum with operational support/fallback restrictions | **74** | **71** |
| Minimum absent reads needed to reach 145 answers | **16** | **15** |
| Unconstrained maximum present correctness | 149 | 159 |
| Unconstrained minimum present CE change | −1937.548 bits | −2038.951 bits |

The maxima for absent budgets 0 through 7 are H4 `[3,6,9,43,46,68,71,74]` and categorical `[1,4,4,49,52,52,53,71]`. This is far from a practical-margin near miss. Loosening the screen enough to accept this table would abandon a substantial fraction of the parent's useful behavior. The preserved parent demonstrates that the desired present/absence pair is attainable by a richer existing decision path.

The certificate is conditional on the sealed counts: it is not an independent re-extraction of every prediction. It establishes an empirical incompatibility for these frozen sources, actions, populations and five-bit observation addresses. It is not a theorem about H4, R4, all finite policies or language generally. Freeing unsupported buckets cannot fix this particular obstruction, but that says nothing about data support/generalization for the successor.

## Correct interpretation of the collision

Different situations requiring different decisions share the same action address. This **is a collision in the coarse observation representation**. It is not an H4 capacity limit. The original mixture objective failed to demand the desired behavior; the new result shows that changing the objective alone cannot obtain it within this class.

The five-bit address contains four local-gap bins, four context classes and two source-margin states. Relative source-ranking margin says how decisively one candidate beats another. It does not say whether any source should beat NoRead. That absolute comparison is present in the older scored reader and was lost when its decision was replaced by the table.

The text-only optimum is negative, but the pair of count constraints is already impossible without considering text. Do not infer that text and present-answer constraints alone have been jointly proven feasible. Whole-model prose remains degenerate, and the run introduces no new final generalization result.

## The concrete reusable bridge

For the frozen scored parent, define the causal integer advantage using its actual methods:

`D = max_(candidate, strength) strength_score(candidate, relation, strength, parent_bucket) - noread_score(parent_bucket)`.

The parent reads iff `D > 0`. Read–NoRead ties abstain; candidate ties retain the first occurrence. Compute differences in a sufficiently wide integer. Preserve selected occurrence/payload/version independently of D. This is a learned score difference, **not a calibrated probability**.

Source and artifact inspection give a stronger witness for this H4 pair. The retained `relational` and `relational_ctx` artifacts have equal roots, source weights/ranks, bias, strength biases and NoRead intercept. Their only relevant difference is the contextual action table. In all 16 parent contextual rows, the eight-nat action is the unique strength maximizer; its offset is independent of candidate. Therefore the first maximal source agrees with `ungated_top_source` on the same candidate pool. The parent bucket includes information such as the newest-source bit and a different single-candidate margin convention; do not substitute the utility bucket.

Thus, for these pinned artifacts, the exact rule

`D <= 0 -> NoRead; D > 0 -> eight nats`

reproduces the parent's scored action by source reasoning. The existing saved parent outcomes supply its 149-answer/seven-absent-read construction behavior. This is an artifact-derived expressivity witness, not a new model rollout. The successor must verify actual prefix/source/action/logit parity through the shared inference boundary before learning.

A minimal candidate is the existing five-bit observation plus the sign of D, giving at most 64 action entries. The parent rule is an explicit feasible starting point for the construction constraints, provided fallback entries retain the parent rule. That avoids guessing whether another feature might be sufficient for the specific count obstruction. It does not solve the text tradeoff: the parent still harms text, so the learned influence policy must earn its own measured improvement. Permit a compact equivalent representation if better justified; do not change ranking, admission, state lifetime and influence simultaneously.

The categorical comparison needs an equally trained confidence/controller path. Borrowing H4's richer contextual controller only for H4 would conflate control information with geometry. Use matched fitting when claiming geometric advantage, or clearly report an asymmetric engineering comparison and withhold that claim.

## Mathematical rationale and external research

For fixed observed local payload probability p and conditional probability r that the selected payload is correct, expected score-space loss change is

`E[delta_loss(a) | observed state] = log(1 + p*(exp(a)-1)) - r*a`.

For known 0<p,r<1 and an unconstrained scalar boost, differentiation yields `a* = logit(r) - logit(p)`; restrict to the permitted actions/range afterward. If p varies within an observation cell, optimize the average of individual costs, not a cost evaluated at average p. This derivation motivates separating source reliability from local plausibility. It does not identify D with logit(r), require floating-point serving, or replace the direct integer-action objective.

[Franc, Prusa and Voracek, JMLR 2023](https://www.jmlr.org/papers/v24/21-0048.html) distinguish proper uncertainty scores and reject-option objectives. [Drozdov et al., EMNLP 2022](https://aclanthology.org/2022.findings-emnlp.218/) use retrieval-quality observations to choose interpolation. [Moskvoretskii et al., ACL 2025](https://aclanthology.org/2025.acl-long.319/) compare adaptive retrieval with uncertainty methods, motivating an inexpensive existing signal before a larger mechanism. These papers guide the separation of reliability and influence; their dense systems, randomized theory or reported gains do not qualify our serving path.

[GRAPE, 2026 revision](https://arxiv.org/abs/2512.07805) and [PaTH, 2026 revision](https://arxiv.org/abs/2505.16381) remain relevant to group-relative and content-dependent transport. Here the immediate information already exists in learned relational scoring. Adding persistent geometry to replace a discarded existing decision would obscure that diagnosis.

## Implementation repairs and evidence boundaries

The submitted generic solver omitted fixed buckets' objective contributions when starting branch-and-bound. Negative fixed costs could make descendant lower bounds too high, prune a better feasible table and falsely label an incumbent optimal. Its bound also used strict constraints while feasible checks allowed tolerance. The principal repair initializes fixed objective contributions and applies the same tolerance-relaxed RHS consistently to bounds, feasibility and pruning. Focused tests compare small problems against independent exhaustive enumeration, including negative/zero/positive fixed costs and both constraint senses at tolerance boundaries. The retained infeasibility certificate does not depend on this repair: no feasible incumbent activates objective pruning in that run, and the exact count proof is independent.

Counterfactual identity checking now uses the **effective saturated logit change**, with a saturation test; the earlier nominal boost could spuriously fail near i32 limits. The saved non-saturated residual around 1.4e−13 bits retains its original scope. The runner no longer reports a tune loss for an infeasible, nonselected action vector. All five focused solver/arithmetic tests, cargo fmt --check and the offline touched-bin check pass; claim wording is also checked. No full harness/model replay was performed for these bounded repairs, and historical sealed outputs are untouched.

Other qualifications remain visible for the successor:

- Both feasibility attempts have valid eight-member seals; attempt 1 is sealed and superseded by a report correction, not “unsealed-in-effect.” Three exported artifact and four parent hashes verify. All panels/generations/intervals replay `reader-utility-4`; no fresh final was used.
- The executed runner receipt SHA starts `7f370f18`, whereas submitted `24c874ac` contains `b22fb4ac`. The solver module was omitted from the source-hash list. `result.base_revision=31972e34` and `binding.base_revision=0492926d` are stale; recorded checkout is `e0f99fcc`. Exact executed-source correspondence remains unresolved; no assumption is made that only formatting changed.
- Saved fit bucket/action matrices are sufficient for the integer certificate, but individual source/occurrence rows, tune matrices and per-document counterfactual arrays were not retained. The next causal interface needs one bounded fresh development extraction with those reusable sufficient statistics.
- The expected-manifest loader now really verifies bytes, stored fit digest, thresholds and vocabulary before returning H4/categorical predictors. The fit digest still omits sequence/reference/config/local dependencies; its meaning remains narrower than complete input binding. The fixed-one-nat verifier return is discarded in favor of the earlier loaded equal selector. Carry a complete expected context through the predictor actually used next, with all source files in the receipt.
- Neighbor intervention accounting still includes the deliberately edited source. Correct it inside the next substantive run, including actual lost admission, changed/added/removed neighbors and absent generated examples.

## Roadmap consequence

1. **Restore the existing confidence distinction and test useful influence.** End the old 32-address search. Verify the parent-preserving interface, fit one compact dose policy, and use matched controls plus one prospective acceptance population.
2. **Learn new structural persistence when older context requires it.** Signed H4/Spin finite transport, event-driven role/scope lifetimes, exact occurrence/version ownership and retained Hopf fiber remain the first reusable mechanisms. A real older-scope alias, not this coarse address collision, justifies new state.
3. **Compose dependent reads and derived outputs.** Reuse shared Read/Update/Emit/Stop and require causal first-read effects on answers that cannot be copied. Preserve useful attention components even when the local generator needs work.
4. **Improve broader language and executed Rust on the same path.** Integrate generative learning, conversation and durable isolated memory rather than mistaking retrieval for prose/reasoning.
5. **Qualify useful efficiency and scale.** Profile selected work throughout, then measure whole-path latency, memory traffic and physical energy at useful quality.

S7, normalized E8, paired-H4/icosian structure and harmonic channels remain conditional tools for a witnessed representation/interference need. Exact identities are not semantic distances; orthogonal basis functions do not eliminate finite coefficient capacity or supply learned contextual role assignment. Scalar compatibility/advantage is a concrete reusable field here; no supersymmetry, twin-prime or general-relativity mechanism has been derived that overrides the tested path.

## Resource reconciliation and preservation

DeepSeek's prose recorded cumulative 195596749/198900000 ms, while the live JSON remained 192996749/194900000. The attached execution narrative places the extension/projection writing after execution. We reconciled the shared JSON retrospectively under standing owner authority, preserved the full charge/extension, and added 14500 ms because its component estimates sum to 2614500 rather than 2600000. Do not call this prospective recording. The principal repair checks have their own projection and measured debit in the [ledger](resource-ledger-2026-09-19.md).

Before cleanup, the live inventory measured 33.82 GB free, about 2.95 GB below the 36.77 GB reserve. Removed only 340 inactive Rust debug incremental cache directories (5.159 GB allocated), preserving executables/dependencies, all models/artifacts/source/research/worktrees and downloads. Immediate volume free space rose 33.93→38.26 GB: 4.32 GB actually recovered at that moment; APFS/concurrent changes explain the allocation difference. No reserve reduction is adopted. Future builds must project physical growth, use bounded caching and respect the reserve plus 128 MiB stop margin; increasing a bookkeeping allowance does not create space.

The original owner checkout remains untouched. This review corrects and completes PR #1332 through protected delivery, updates the six owning issues and front doors, and provides a revision-pinned knowledge handoff. No new language capability, physical energy result or whole-path D0-b qualification is claimed.
