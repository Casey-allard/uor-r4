# Learned ordered geometric span producer, October 1

References #1512 and #973. This extends the [retained direct geometric reader](geometric-address-reader-2026-10-01.md) in the existing Rust model graph. It is a new typed span representation, not a claim that the earlier captured contextual vector has been preserved.

## Executed candidate

Executed source `cfdf535a800e343873699ca4693f7b11f9a2e32d` adds a separately saved geometric-span mode. Original selected embedding rows supply shared static signed-2I token actions. Contextual current/previous gained inputs supply a four-class controller. The working group tuple and committed held tuple are separate; the four q/k maps remain absent, and the old scalar capture gate is removed.

HOLD preserves state. OPEN resets an empty working tuple to identity without consuming its delimiter. APPEND right-composes the current token action only while open. COMMIT copies nonempty open working content and closes; empty/unopened commits preserve held content. Each read receives old held content before the current action. Present spans have fixed unit radius, including identity products; invalid held content emits zero. Static zero token rows mean identity actions, not absent spans. Exact occurrence, original span and payload remain separate obligations.

Hard forward uses predicted earliest-argmax actions and exact finite group products. It never receives event/source annotations. The offline backward uses selected-branch Hamilton/token-direction Jacobians and detached softmax branch differences. Active/nonempty/valid predicates are detached. Output tangent projection removes spurious learned-radius credit. All-HOLD trajectories therefore have no content gradient; training-only event supervision must establish valid paths before answer credit can refine earlier actions. This is a declared biased surrogate, not the derivative of hard decisions or a probability distribution over executed states.

A per-call LastToken intervention replaces composition at APPEND with the latest token action while preserving predicted controller decisions, validity, commit timing, current context, support, age, NoRead and payload paths. With the rra topology these upstream inputs precede the reader and do not change under this intervention. Saved Ordered semantics and parameters remain unmodified.

Span sidecars and distinct address-operation records bind root ordering, action order, state/surrogate semantics, model/config identities and tensor inventory. Generic intermediate-input callers without original token provenance fail. Existing scalar Held/default artifacts remain distinct; unsupported export/grid/checkpoint and binary-latch overrides refuse the new mode.

## Fixed experiment

The source task stores both `anchor A B tail` and `anchor B A tail` in the same episode. Endpoints and token multiset match, excluding a first-token, last-token or bag-only solution. Payloads are drawn independently of keys; training pairs have distinct values, while four evaluation groups deliberately share answers but require different occurrences. Four-fact episodes add a second opposing pair with the same endpoints. Changed-query, swapped-value and reversed-record-order interventions accompany each of 32 groups. Original gaps are 0..4 noise pairs; stress gaps 0..8.

Two seeds, 320 updates each, batch 16, learning rate 0.003, weight decay 0, clip 1, source credit 1 and balanced four-event credit 1. Context ceiling 128, width 32, two heads, actual training positions at most 80 and stress at most 120. All causal sources, self and NoRead remain available. This is a development binding diagnostic, not a final language holdout or teacher-ranking gate. No donor computation occurs.

## Results

Both seeds completed 320/320 updates. Each contains 48,326 parameters in 45 tensors; four q/k maps and the old scalar latch gate are absent. Independent reload and post-intervention restoration reproduce the recorded outputs exactly. The report root sealed and verified its 14-file set. Raw per-row logits, source masses, learned models and training snapshots are retained; the [compact artifact-bound summary](geometric-span-actions-2026-10-01.json) binds their identities.

| Condition (128 rows each) | Seed1 answers | Seed1 head 0 source majorities | Seed2 answers | Seed2 head 0 source majorities |
|---|---:|---:|---:|---:|
| Original |99|128|128|127|
| Longer-gap stress |109|126|128|128|
| LastToken |27|41|44|44|
| Read output disabled |12|128|4|127|
| Cross factor disabled |70|117|109|109|
| All context factors disabled |8|0|6|0|

Original minimum target-source masses are 0.9139288/0.4895464; stress minima 0.3305858/0.7077900. Source credit targets head 0; head 1 has zero correct-source majorities in every panel and is not qualified. Seed 2's one original majority miss (`pair411/base`) still answers correctly. Seed 1's 29 original answer errors all have target-source mass at least 0.9820395. Its 19 stress answer errors also retain source majority; both stress source-majority misses answer correctly. These observations separate source selection from answer realization without diagnosing a particular downstream parameter.

LastToken loses 72/84 previously correct answers and gains none; original head 0 source majorities drop 128→41 and 127→44. It preserves the current contextual/controller path, so this supports accumulated span content's causal contribution in these candidates. Cross-only removal loses 30 answers and gains 1 in seed 1, and loses 19 in seed 2; it supports contribution rather than universal necessity. Combined context removal does not isolate one angular/radial/presence factor. Read-output removal leaves source masses unchanged and damages answers, as expected for a downstream intervention.

Predicted original event correctness is 6,215/6,216 and 6,216/6,216 positions. Seed1 has one write-key HOLD instead of APPEND. Pure-answer gradients reach span-controller weights and geometric cross coefficients in 16/17 sampled snapshots for each seed, beginning at update 2; preceding recurrent output receives credit in 17/17. Initial all-HOLD zeros agree with the declared event-supervision requirement. The detached-context/controller fixture separately isolates earlier token-action credit through the actual address objective.

Actual maximum evaluation/stress lengths are 70/96 positions. Training consumed 245,794/245,988 unpadded positions and 296,000/294,464 padded positions. Requested training bounds 80 and stress bounds 120 remain conservative ceilings, distinct from width 32 and configured context 128. The panels contain 32 correlated groups, including 28 changed-answer and 4 equal-answer opposing-query pairs; 128 rows are not 128 independent task families. Stress uses a different draw, so a higher score is not evidence that longer gaps improve the model.

**Decision: retain the ordered geometric producer and advance its exact finite token-action/register boundary.** Do not require perfect answer realization to preserve useful selection, and do not turn the seed 1 miss into an unchanged selector/dose/seed campaign. Preserve both models and all errors. The mechanism still requires natural span/typed-role learning and broader capability evaluation; a matched ordinary algebra comparison is needed only for an advantage claim.

Fit plus controls/evaluation took 1,368.529 seconds at the process boundary (report-internal 1,367.946 seconds), with maximum RSS 131,301,376 bytes. Complete preparation/build/review/delivery/cleanup wall is charged once through the shared cumulative ledger; model time is included, not added again. The completed cost receipt belongs in the retained root.

## Meaningful checks and remaining scope

Executed checks: twelve span/library checks, two evaluator checks, ten retained address checks, ten retained latch checks and one ordinary fused-gradient reference passed (35 named checks). The example compiled at the frozen source. After integrating current main, nineteen compatibility checks passed: twelve span checks, five prime-route checks including routed-gradient/restoration, the registered-prime check and the old-pointer configuration-default check. No new fit or broad suite was run. The two pre-test dependency-build interruptions and evaluator diagnostic-filter compilation failure are retained as execution history; they are not negative model evidence. The corrections changed fixture dimensions and a gradient-report string comparison, not the model mechanism or fixed fit.

Executable SHA256 `b345cd1b8572197d618dc057e71244d60da73f3f20a22e89948c0c92af8d2e17`; dev profile with only the training crate optimized at level 3. This is not optimized serving performance. Saved executable gzip round-trip is byte-identical. Retained root: `/Users/casey.allard/.local/share/uor-r4/research/geometric-span-actions-20261001`; execution identity, raw panels, models, logs and receipts belong there.

The earlier-token gradient check detaches the contextual and controller routes and uses the actual address objective, avoiding the ambiguity of shared embedding gradients. The mathematical checks exercise declared surrogate derivatives, empty/identity/inverse behavior, common-endpoint order, LastToken, batch isolation and causality. Interface checks cover raw input provenance, actual forward/observer consistency, save/load identities and unsupported mode refusal. The task checks cover source/value oracle consistency and context bounds.

This first producer has one working tuple and one committed tuple. It does not preserve independently typed simultaneous entity and role-path registers, nor can a product generally recover its factors. Finite products can collide, including inverse cancellation; exact occurrence/version identity must remain outside them. Multi-token authored spans are not natural linguistic span qualification.

Current context/controller, embedding parameters during training, geometric potential evaluation, trunk, NoRead, softmax, value/output and vocabulary head remain floating-point offline operations. Static token actions have a finite export destination and register updates can become integer/table operations, but export and full serving behavior are unmeasured here. Full causal scanning grows up to the configured context ceiling; no separately bounded candidate index is implemented. A LastToken drop can admit useful accumulated composition, but does not establish an advantage over C120 or another equal-capacity algebra.

Offline donor embeddings and contextual action/selection targets remain supported training inputs into this same native interface. Static coordinates alone do not qualify retained reasoning. A donor must not supply runtime responses or survive as a hidden dense backbone.

## Preserved offline weight recompilation

Owner reaffirmation: retain the ability to compile other learned weight sets into the native geometric model. A donor is an offline source of learned information, with pinned weights, tokenizer and licensing/provenance; the served artifact must execute its own geometric operators without donor/provider access. This route shares the native target rather than requiring a transformer serving path.

There are three distinct transfer interfaces. Token/lexical representations can initialize shared finite token actions and realization codes. Contextual donor behavior can provide training targets for native span events, memory selection and typed state transitions. Donor output distributions can supervise native realization. Their implementation is future work here: this run uses no donor and establishes no retained donor capability. The finite span product is not an invertible encoding of a donor hidden state or an arbitrary sequence.

Reasoning retention must be measured on the resulting native artifact, including changed constraints, reordered premises and multi-step composition with exact source dependence. Static embeddings alone do not establish reasoning transfer. [Geva et al. (2021)](https://aclanthology.org/2021.emnlp-main.446/) identify learned key/value memory behavior in transformer feed-forward layers; this motivates inspecting computation beyond embeddings, without identifying those memories with reasoning as a whole. [Hinton, Vinyals and Dean (2015)](https://arxiv.org/abs/1503.02531) motivate behavior transfer through offline distillation, but do not establish that this geometric architecture can retain frontier capability. These are research motivations, not UOR-R4 results.

The immediate sequence remains: establish useful ordered geometric attention, export its finite actions and operators with measured behavior, then test donor-informed learning through the same interfaces against the native baseline. A teacher may guide offline learning; it cannot supply runtime responses, and a table must not conceal dense transformer execution.

## Native span boundary after this result

The directly reusable runtime boundary is an offline-compiled token-action dictionary plus an exact integer span register. At width 32, each token row holds eight signed root IDs. Use the current classifier offline, avoiding a new embedding quantizer. The existing `HistoricalH4Tables` loader admits the pinned mathematical payload and exposes signed, ordered group composition with shift/add table indexing. Bind both runtime and training geometry identities: their digest framing differs, so compatibility requires the existing exact ordering/product/inverse mapping, not equal digest strings.

Record held content before applying each event. In particular COMMIT becomes visible at the following position; matching only the final state would miss source-address timing errors. Replay must consume the actual controller's predicted events, not annotated events. Reconstructing roots at a labelled float replay boundary can measure saved-model retention while preserving existing full-stack export refusals. The contextual controller and remaining model computation still require their own native implementations.

## Concurrent prime route and integration boundary

Delivery integration merge `1ab6e7542873dd7f58e82844caaf5bd020f17d5d` incorporates PR #1587 (`70d8a223c2a304fff1de7df513d6340f9b1eb0b0`), which independently adds registered-token prime-factor admission to the pointer-copy head. It is preserved in the combined delivery source. The fixed span experiment executes its earlier frozen source, with no pointer head; no prime-route behavior is measured by these span results. The route's current logarithmic scores, softmax and learned gate remain float development computation, and full scanning is not a bounded index.

Prime overlap and ordered signed-group content have distinct possible roles: overlap can admit candidates sharing atoms, while an ordered span representation distinguishes permutations that have the same token multiset. This is a proposed integration, not a measured result. A later consumer must bind both summaries to the same predicted span/commit boundary, preserve exact occurrences, and address paraphrases and missed admission; a last-W-token prime key across noise is not automatically the committed span key. Prefer this shared typed interface over a separate engine, without extending the current experiment or importing float logarithms into declared serving.
