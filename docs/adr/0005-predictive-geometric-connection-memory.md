# ADR-0005: Predictive geometric connection memory

- **Status:** Accepted as the current #973 research direction; unqualified as a
  language mechanism
- **Date:** 2026-08-28
- **Owner:** #973 under programme root #820
- **Supersedes for forward work:** another fixed componentwise prototype or
  scale-only repair after the #997 negative
- **Preserves:** [ADR-0003](0003-fixed-zeta-prime-route-attention.md) route
  identities and [ADR-0004](0004-geometric-intelligence-route-hierarchy.md)
  scope/transport boundaries
- **Evaluation:**
  [Geometric Intelligence Evaluation](../geometric_intelligence_evaluation.md)
- **Evidence:** [Research ledger](../RESEARCH.md)

## Decision

UOR-R4 will test one predictive, recurrent, associative memory whose state is
stored in R4/spin charts and moved between charts by declared geometric
transport. The full mechanism is named `PredictiveConnectionRetentionR4V1`.
The immediate implementation is its stop-first prerequisite,
`PredictiveConnectionRetentionGate0V1`; Gate 0 must establish transferable
signal in fixed exact-route features before recurrent banks are built.

This is the current attempt to build geometric attention. It is not another
retrieval table, another token-to-prime placement rule, or a claim that routing
already equals attention. It combines five jobs that the evidence says must be
present together:

1. causal prediction supplies the construction objective;
2. separate key and value placements prevent address identity from being
   mistaken for semantic role;
3. connection transport moves retained state into the current query frame;
4. gated delta updates overwrite stale associations and retain useful ones;
5. candidate-relative readout ranks only the unchanged lawful support admitted
   by the accepted #953 path.

The deployed goal remains a local CPU engine with no Transformer, softmax
all-pairs attention, mixture of experts, learned sparse expert router, Ollama,
hosted provider, or source weights. Compiler-side fitting may use floating
point, multiplication, allocation, and parallel reduction. None of that work is
credited to the deployed kernel. Exact/table/ternary lowering is authorized
only after the causal mechanism qualifies.

## Why the direction changed

The project has established useful but sharply bounded pieces:

- #989's source-free lexical table scored 99,362/446,342 held-out known targets
  (22.261404%) against 5.413561% unigram;
- #953's one accepted geometric count-radius intervention scored
  103,604/446,342 (23.211797%), +4,242 correct and +0.950392 percentage points
  at equal support and declared work;
- #969 and #973 produced bounded order-sensitive, paragraph, conversation, and
  noncommuting-global causal route witnesses; and
- #997 showed that causal geometric activity is not enough: its
  componentwise-Frechet document placement scored 8.367592% on 35,028
  construction-fitted held-out targets, below frozen #953 at 12.221651% and
  below both order-shuffled (8.376156%) and operator-permuted (8.467512%)
  controls.

The diagnosis is therefore specific. R4/spin routes remain valid identities,
state carriers, and transport. A fixed marginal center of identity-derived
coordinates is rejected as semantic placement. More documents or more rows
cannot repair an objective whose matched controls are already better.

## Representation

### Immutable address and learned predictive roles

Every lexical unit retains its immutable registered route, prime, spin/Hopf,
torsion, payload CID, and kappa identities. The predictive artifact adds three
separate versioned placements:

- `K(x)`: a key used to decide which retained association an observed route
  addresses;
- `V(x)`: a value written when that observed route is useful for predicting a
  later route; and
- `Q(c)`: a candidate-relative query/readout placement for one already-admitted
  candidate `c`.

These placements are compiler outputs with inverse/provenance witnesses. They
do not mutate immutable addresses or payloads. A digest, token rank, prime
index, modulo class, or hexadecimal spelling may seed or identify a row, but it
cannot be interpreted as learned meaning.

### Connection-transported multiscale state

The recurrent state contains fixed-capacity banks for four causal horizons:

```text
M_t = (M_t^local, M_t^short, M_t^scope, M_t^long)
```

The intended readings are current/previous route, last two or short suffix,
open sentence/paragraph scope, and bounded conversation/global retention. They
are channels of one mechanism, not independent attention claims. A scope
boundary can reset, checkpoint, or change a gate according to a frozen policy;
it cannot scan the complete prefix or corpus.

Before a bank is read or updated at step `t`, its state is moved from the prior
frame to the current frame by an artifact-declared connection transport:

```text
M_bar_t^s = Transport(A_(t-1 -> t), M_(t-1)^s)
```

`A` binds exact frame, orientation, chart, quantization, and transition-law
identities. In the first host-side prototype it may be evaluated in a declared
floating representation. A later exact runtime lowering must reproduce its
frozen reference semantics and satisfy the repository kernel contract.

### Gated delta update

For each bank `s`, the construction reference has the following semantic form:

```text
r_t^s = Read(M_bar_t^s, K(x_t))
e_t   = V(y_t) - r_t^s
M_t^s = Retain(M_bar_t^s, lambda_t^s)
        + WriteDelta(K(x_t), eta_t^s * e_t)
```

Here `x_t` is observed prefix data and `y_t` is the observed next route in the
construction partition only. `lambda` controls forgetting and `eta` controls
targeted overwrite. This equation is an offline reference objective, not an
assertion that multiply or float is allowed in serving.

At validation, test, and inference time, `y_t` is unavailable. State is updated
only after the selected or externally observed route becomes part of the causal
prefix. Actual future routes, evaluation answers, teacher continuations, source
weights, and provider text are forbidden.

### Candidate-relative readout

The accepted #953 policy first freezes lawful support `A_t`. For each `c` in
that same support, the predictive mechanism reads all enabled banks using
`Q(c)` and returns one deterministic score:

```text
score(c | M_t) = Readout(Q(c), M_t^local, M_t^short,
                         M_t^scope, M_t^long)
```

Only a unique qualified winner can replace #953's choice. Missing state, a tie,
or a failed margin returns exactly to #953. Runtime work is O(1) per state-bank
update and O(|A_t|) per decision under fixed bank capacity. No all-prefix
attention matrix, corpus scan, unbounded prompt replay, or candidate injection
is permitted.

## What makes this geometric attention rather than recurrence alone

The full mechanism earns a geometry-specific claim only if all three properties
are causally necessary:

- exact connection transport changes useful retained state;
- candidate-relative R4/spin key, value, and query relations beat their
  deterministic frame/operator permutations; and
- the full multiscale state beats a matched last-only state at equal support,
  parameter/byte budget, and declared work.

A matched non-geometric recurrent/delta memory is mandatory. If that arm
transfers but the transported geometric arm does not beat it, the honest result
is `RETAIN_PREDICTIVE_RECURRENCE_REJECT_GEOMETRY_SPECIFIC_PROMOTION`. The project
may retain recurrence as a comparator or syntax substrate, but cannot call its
gain geometric attention.

## First implementation: `PredictiveConnectionRetentionGate0V1`

The immediate #973 action is one bounded host/compiler-side Gate 0. It reuses
the frozen D3 documents, #989 table, #953 admission/support, payload inversion,
and work ledger. It does not modify `CorpusInducedDocumentSpinPlacementR4V1`;
#997 remains immutable negative evidence.

Construction documents are split deterministically by document identity into
fit and construction-validation partitions before training. For each admitted
candidate, Gate 0 exposes twelve integer relations: H4 shell rank, wrapped
fiber distance, and wrapped torsion distance against current, previous,
ordered-last-two, and complete-prefix taps. A deterministic candidate-specific
integer readout is trained against the actual co-admitted distractor. A full
language model, recurrent banks, learned keys/values, BPTT stack, serving
integration, and exact runtime lowering are out of scope until Gate 0 passes.

Gate P0 is binding and runs before the 596-document D3 held-out target join.
It proceeds only if the full arm improves both next-route loss and ranking over
frozen #953, the matched plain recurrence, state-disabled, earlier-order-
shuffled, transport-permuted, and last-only arms on construction-validation
documents. Target-free census, support/work equality, deterministic reduction,
and a non-degenerate control effect must also pass.

If Gate P0 fails, D3 held-out labels remain unopened and the fixed route-feature
representation is rejected. If it passes, the next implementation is the full
four-bank connection-retention cell defined above. More scale is not the next
action.

## Held-out promotion gate

Only a Gate-P0 positive may attach the frozen D3 held-out next routes once. The
full arm must then:

- improve held-out next-route loss and top-1 over #953 and every matched
  control at the position level;
- retain the direction under exact document-blocked analysis;
- beat the matched non-geometric recurrence to earn geometry-specific credit;
- cause one predeclared bounded decoded-output divergence;
- preserve byte-identical support and declared work across arms;
- report zero forbidden target/future/source/provider reads; and
- reproduce artifact and report bytes exactly.

A positive establishes only one held-out predictive geometric-memory mechanism
inside #973. It does not establish correctness, reasoning, general coherence,
chat, performance advantage, exact runtime lowering, or product readiness.
#954 stays blocked until the complete #973 hierarchy terminal is earned.

## Outcome branches

| Result | Required next action |
|---|---|
| Full arm beats #953, controls, and matched recurrence at Gate P0 and D3 | Retain the mechanism, freeze an exact/table lowering contract, and requalify the bounded #973 scopes with the new placement epoch. |
| Plain recurrence transfers but geometry does not beat it | Retain recurrence as the syntax/control comparator; reject geometry-specific promotion and redesign only the connection/placement seam. |
| Full and plain recurrence both fail Gate P0 | Do not open D3 labels or scale. Revisit the predictive state/causal objective rather than route density or coordinate tuning. |
| Gate P0 passes but D3 fails | Preserve the construction result as overfit; reject the mechanism and reassess representation/generalization. |
| Required frame, population, or target-free audit is unavailable | Stop `UNAVAILABLE`; do not infer a metric result. |

## Research basis and limits

This design combines ideas whose published results do not themselves prove a
UOR implementation:

- [Gated Delta Networks](https://arxiv.org/abs/2412.06464) motivates combining
  adaptive forgetting with targeted delta updates.
- [Retentive Network](https://arxiv.org/abs/2307.08621) and
  [Mamba-2/structured state-space duality](https://arxiv.org/abs/2405.21060)
  show recurrent low-cost inference as a serious sequence-modeling design
  space.
- [Zoology](https://arxiv.org/abs/2312.04927) makes associative recall a
  necessary explicit stress test for efficient sequence models.
- [From Self-Attention to Connection Laplacian](https://arxiv.org/abs/2607.10677)
  supplies the useful operator view of attention as aggregation plus transport;
  this project tests a recurrent bounded connection memory rather than copying
  a Transformer attention matrix.
- [Computational mechanics](https://arxiv.org/abs/cond-mat/9907176) motivates
  judging a state representation by retained predictive information rather
  than by its coordinate elegance.
- [Scalable MatMul-free Language Modeling](https://arxiv.org/abs/2406.02528)
  demonstrates that removing matrix multiplication is a credible systems goal,
  but its architecture and reported results are not UOR evidence.

No cited work establishes a geometry-native, transformer-free, causal local
language model with the UOR runtime contract. That combination remains the
research gap this ADR turns into a falsifiable implementation sequence.
