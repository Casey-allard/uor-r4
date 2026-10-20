# Connected geometric NoRead — October 2

The native attention path now has an explicit geometric NoRead producer, an
offline Rust learner, a source-bound compiler/reloader, and a raw-Q24 reader
entry before the legacy floating read normalization. The producer uses the
actual current token, all retained signed H4 context roots, typed observed
radius/presence, and prior held roots with explicit span validity. It does not
take donor hidden state, source labels or answers as prediction inputs.

This delivers a connected learning and compilation boundary. It does not solve
geometric attention or qualify the complete serving path. The accompanying
[receipt](../evidence/geometric-no-read-2026-10-02.json) binds executed commands,
source/artifact identities, actual saved-parent results and remaining work.

## Representation and learning

All free coefficients are signed four-bit values in [-7,7], at a fixed
quarter-nat scale; reserved -8 is rejected. Offline nat-valued shadows use an
explicit slope-one straight-through estimator and range projection. That
estimator is a biased training surrogate, not the derivative of rounding.
Ordinary answer loss reaches the scalar through the shared occurrence/null
softmax denominator; the integer reducer is not differentiated.

Each output head sees all eight retained context lanes at H2/L4, including
signed orientation. The compiler regenerates expanded Q24 tables from packed
coefficients and the pinned Q25 canonical-F32 observation basis. It keeps exact
H4 identities distinct from observed coordinates. Each root factor rounds
separately, then the score adds exact integers. The declared reference therefore
does not assume bitwise agreement with one F32 matmul or one final rounding.

At vocabulary40, H2/L4, the producer has754 free coefficients,377 packed bytes
and20872 expanded table bytes. It selects36 table terms without a held span or
68 with one across both heads. Wider tables are derived precision, not additional
free learned weights. The composition binds the scalar source and the actual
tokenizer, context, event, span, potential, value and reducer dependencies.
Reload rejects changed tables, stale live shadows and substituted dependencies.

## Executed evidence

Four integer cases pass. Nine training cases pass across the preserved attempts:
canonical basis identity, hard-forward answer credit, source serialization,
four compiler/admission cases and two existing read/NoRead cases. The initial
test compilation failure was a fixture calling a private trace helper; the next
attempt had eight passes and one F64/F32 fixture mismatch. Both failures are
preserved; focused repairs affected tests only. The repaired gradient case
passes. Two additional focused checks pass for raw Q24 bit retention and the
actual StackModel caller.

The integrated caller check measures final-answer gradient connection, bypasses
poisoned legacy read normalization/null/value branches, retains output-map
sensitivity, checks causal-prefix behavior, and reloads the saved composition.
The actual release example compiles. The compiled ARM64 scalar score symbol has
255 instructions, with no multiplier, divide, floating-point or allocator call.
Its five direct calls are bounds-panic paths; admitted private tables and typed
root/category inputs satisfy those bounds. This inspection covers this scalar
symbol only, not admission, other operators or the complete model.

Both actual saved query-credit parents complete the declared no-fit comparison,
with128 original and128 stress rows each. The scalar starts at all-zero q4;
there are no optimizer updates or checkpoint selection.

| Parent | Legacy original/stress | Geometric source original/stress | Compiled original/stress |
|---|---|---|---|
| Seed1 |98/104|99/104|99/104|
| Seed2 |128/128|128/128|128/128|

All512 legacy predictions match the historical saved-parent predictions.
Source/native predictions also agree on all512 rows. Seed1 original gains
rows72 and102 and loses row113; preserve these cases rather than promote the
aggregate. Maximum actual-position source/native logit difference is
0.00019490718841552734. That comparison includes real-softmax/F32 versus
native LUT/Q24/Q16 reduction, not only scalar compilation. The all-zero
construction does not measure nonzero learned scalar parity; focused arithmetic
and compiler cases cover that boundary separately.

One actual answer backward per parent produces finite positive credit in all
six measured coefficient families for both heads. Bias-gradient magnitudes are
3.4161e-7/1.7530e-4 for seed1 and4.5659e-7/1.9014e-5 for seed2. These are two
specific construction rows, not a general gradient guarantee. Both report roots
are exclusively claimed, sealed and verified. The original console label
mistook the empty unlisted-file vector for a file count; a logging-only source
correction preserves the successful reports and their original executable.

## Next causal task and limits

Fit only the NoRead shadows first, through ordinary answer loss on the frozen
saved query-credit parents, at one declared fixed dose. Keep zero construction
and legacy parent results, exact training draws and unchanged producer/output
identities. Record coefficient changes, clipping, per-head/family gradients,
null mass and row-level changes. Do not add scalar imitation solely because
donor null scores or head0 answer gradients are saturated. Donor operator/weight
compilation remains an available offline path with declared evidence.

The current grammar always answers at token34 with a committed query span.
Bias, query-token and always-valid features can be collinear, so answer gains
cannot establish history-sensitive geometric use. After fitting, a diagnostic
may zero only the scalar's geometric/category/held/valid coefficients in a
copied source while preserving native context, addresses and values. This
measures attribution; it is not a new family-admission gate. A failure to use
geometry on this grammar does not discard the mechanism. Natural query/spans,
unknown-record absence and varied validity remain separate representation/data
obligations.

Other value/potential coefficient widths, floating trunk/residual/read.out/output,
bounded candidate/parameter access, natural conversation and reasoning, and
complete laptop energy remain unresolved. Claude's D19/session work stays
parallel; no stack_grounded_session.rs change is included. No runtime Q/K,
transformer backbone or provider-authored answer is adopted.
