# Query-read credit allocation driver — October 2

The Rust geometric value driver now continues the actual saved context **and**
learned value weights. It can compare frozen-teacher query-read auxiliary
placement with a uniform matched-mass control. This implements the next causal
question from [the completed fits](geometric-value-learning-2026-10-02.md);
the four substantive continuation fits are **NOT_RUN** at this source checkpoint.

For each example/head/lane/atom and separately for root/category targets,
let `a[t]=occurrence_weight[t]/total_weight`, where total includes NoRead.
Use immutable teacher mask `m`, eligible mass `s=sum(a*m)` and count `n=sum(m)`.
Query placement is `a*m`; uniform placement is `(s/n)*m`, or zero when `n=0`.
Both families use fixed denominator `B*H*4*2`, so null/excluded mass attenuates
the objective instead of being divided away. Root masks exclude zero/absence;
category masks include present-zero. F32 conversion has an explicit mass
rounding check. Equal mass is not equal gradient norm. This control differs from
the old global-position mean, which remains separately reported.

Offline teacher labels, masks and exact integer query weights never enter
learned/native prediction. Fractional target weights are serialized as F32 bits;
Boolean labels and raw integer query streams are retained separately. Both arms
load the same pinned parent and next exact Rust u64 RNG state, use absolute
step640+j and fresh AdamW moments. Saved moments are unavailable: this is not
exact optimizer resumption. Initial float/native logits bind to the learned
parent, while donor logits bind to its new-address replay. Historical donor/K1/K2
rows remain comparisons. Fresh-fit CLI behavior retains its original policy,
initialization and old-parent bindings.

Actual release validation completes three unique unit cases: layout/masks,
matched credit with NoRead/zero/padding/fractional bytes, and exact RNG/logit
binding. The first zero-credit test exposed Candle pruning zero-affine graph
connections. Its failure is preserved; explicit self-subtraction fixes the
zero path. The final changed credit case was freshly compiled and passed.
The release driver, formatting and diff checks pass.

All four seed/policy construction attempts exit zero, each with no updates and
one backward batch. Context/value parameter files equal the actual parent in
initial and final saves. All13 trained families receive positive ordinary-answer
gradients. Across64 checked original/stress rows, measured state/action/address/
packet/Q16/answer differences are zero; logits are close, not bit-identical.
Each seed's arms match exact first draws/RNG, Boolean targets and raw teacher
query streams, while their fractional allocation bytes differ. These checks
establish construction and credit availability, not learning benefit.

The [receipt](../evidence/geometric-query-credit-driver-2026-10-02.json) binds
sources, actual executable, logs, reports, manifests and initial comparisons.
Source/model jobs use the shared capacity budget and release builds; the
four checks take about8.38 seconds combined. Independent source and experiment
reviews require no broader suite. Final dose/data/checkpoint/reserve/cost and
exact source identity are recorded before substantive launch.

The comparison concerns **joint** context/value learning: transitions can change
addresses as well as payloads. Frozen-parent query probabilities stay fixed as
the students evolve. Preserve all gains/regressions/alternate codes; neither
an incomplete dose nor a negative result retires geometry or establishes a
representation-capacity limit. NoRead, trunk and output remain floating. A
separate source/retained-output audit rules out an exact token-only NoRead table:
the same token at the same position has different scores after an earlier-key
change. Its native replacement needs explicit causal state and a raw Q24 branch
before floating normalization. NoRead and output remain subsequent obligations;
Claude's D19/session lane is separate.
