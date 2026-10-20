# Read-conditioned geometric emission — prospective design

> **Executed in part; principal correction after PR #1334:** [review](read-conditioned-review-2026-09-21.md) distinguishes verified short confidence rollout from a suffix-solvable, misindexed frozen-row diagnostic with unchanged maps. The [new execution prompt](deepseek-contextual-emission-step-2026-09-21.md) owns scheduling. Original instructions/design below remain dated evidence, not completed claims.

September 21, 2026. Executes the [principal confidence review](reader-confidence-review-2026-09-21.md) and
the [constructive prompt](deepseek-read-conditioned-state-step-2026-09-21.md) on reviewed parent
`466043eb`. Recorded **before** the extraction, fit and any fresh outcome. The [resource ledger](resource-ledger-2026-09-19.md)
holds the projection and the time increment taken before execution.

## Objective

(1) Close the missing frozen-confidence boundary by running the **actual** retained confidence
artifacts through the shared target-free predictor, with complete emitted responses and the served
action/selected occurrence per step. (2) Implement and test **one** learned read-conditioned geometric
update feeding the shared emission readout, on an instrument the current scalar-copy operator cannot
solve. Preserve the confidence component and every failed criterion.

## Why a new operation is required

The served operator changes exactly one selected payload logit (`z'_c = z_c + a`). For any token
`v != c`, `p'(v)/p'(w) = p(v)/p(w)`, and if the gold token is not the selected payload,
`delta CE = log(1 + p(c)*(exp(a)-1)) >= 0`. No dose or gate can favour an uncopied token. The
instrument therefore requires an answer absent from every admitted payload.

## Frozen-confidence rollout

Compact prospective development prompt set (present and absent construction prefixes, ordinary-text
windows) through `predict_next`/`generate_step`: local, the scored contextual parent, `h4_confidence`,
`categorical_confidence`, and `h4_confidence` with the reader disabled. Complete emitted tokens, the
served action, selected occurrence and admission counters are saved. This completes missing evidence;
it is not a new fresh final claim.

## Instrument (declared before fitting)

`derived role-partner`: several families share the query key; the query carries the role of one
family; the required next token is that family's **partner role**, which is placed in no block role and
in no value, so it appears nowhere in the prefix and in no admitted payload. The retrieved block is
relevant (its role identifies the family). Validity is checked before fitting: zero answer coverage by
payloads, and a decisive subset where the frozen local argmax is wrong.

## Operator

```
q0 = local.query_state(current token)          causal prefix query root
r  = directed relation of query role and selected source role
v  = V[selected payload token]                 learned value code (identity off-domain)
q1 = (q0 * T[r]) * V[v]                        exact signed-H4 products
z1 = z_local + u(q1) - u(q0)                   shared frozen row readout
NoRead / UpdateDisabled: q1 = q0, residual exactly zero
```

Only `T` (one group element per relation) and `V` (one element per observed payload token) are
learned. The readout `u` is the frozen shared reader row table, so no vocabulary-sized expert and no
new dense head is introduced. The multiplication order and signed frame follow the existing table.

## Learning, oracle and controls

Discrete coordinate ascent over `T` and `V` under final-token accuracy on the development instrument
(no gradients, no new parameters). An **oracle ceiling** reports the fraction of positions where any
shared row emits the required answer, separating a learning failure from a readout-capacity limit.
Matched controls: update-disabled (exact identity → the frozen local baseline), reader-disabled, the
scored copy parent, and a **capacity-matched categorical** update learned under the same search from a
fixed non-isomorphic seed. Causal checks recompute the emitted token when the selected payload or the
relation is changed. Generated first-token hits are reported for the instrument.

## Populations

Development/tune/fresh instrument seeds `0x5C0F_D001/2/3` (60/30/30 sequences) and the previously
declared fresh construction/text populations for the rollout. Tune is used for selection checks; the
fresh instrument is the held-out draw for the primitive.

## Scope and resources

The instrument is a bounded authored transformation, explicitly not general language or reasoning.
Whole-path D0-b and physical energy remain UNAVAILABLE. Projection: one rollout, one instrument
extraction, two discrete searches, focused tests and delivery, within the ledger increment recorded
before execution.
