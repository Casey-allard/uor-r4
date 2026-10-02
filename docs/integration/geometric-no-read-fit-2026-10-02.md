# Learned geometric NoRead — completed fixed-dose fit, October 2

Both saved query-credit parents complete640 answer-only NoRead updates. Only
the754 scalar shadows are optimized; the1280-update context/value producers,
compatibility/reducer and floating output remain frozen. The fitted packed-q4
scalar independently saves, compiles and reloads. Source/native answers agree
on all512 original/stress rows and reproduce every legacy-parent prediction.
This replaces one floating attention producer under these fixed conditions;
it does not qualify the complete model or establish geometric advantage.

[Connected source](geometric-no-read-2026-10-02.md) ·
[Fit receipt](../evidence/geometric-no-read-fit-2026-10-02.json).

## Recipe and actual execution

The new Rust example preserves the delivered no-fit comparator. It admits the
actual saved parents and their sealed zero-q4 construction reports, restores
exact u64 RNG state, and continues the unchanged sampler at absolute step1280.
New NoRead updates are0→640, separately from frozen producer updates1280.
AdamW uses rate0.003, betas0.9/0.95, epsilon1e-8 and zero decay, with selected
NoRead gradient clipping at1 and shadow projection into[-1.75,1.75]. Ordinary
answer CE is the only objective. There is no donor-scalar auxiliary, new scale,
selector, representation or dose sweep.

The release example compiles; both focused mask/RNG and selected clipping/
projection checks pass. Both actual-parent check-mode attempts perform one
batch backward with zero updates, retain parameter bits and independently
reload/evaluate32 original and32 stress rows. Independent source review approves
the fixed fit without a positive-gradient or quality admission threshold.

The two fit wrappers take8.846464 and9.054113 seconds, including load, training,
saved controls, scalar-only ablation, compilation/reload, evaluation and sealing.
Maximum measured resident set is231374848 bytes. Their optimizer computation
takes5.052832 and4.964115 seconds. These are small authored CPU development
runs, not laptop energy or general model performance measurements.

Across both fits,10240 episodes process491514 actual token positions and573200
padded positions. Actual episode lengths span24..80; batch maxima span36..80
for seed1 and34..80 for seed2. Full causal support/context remains128, model
width32, heads2 and value width16 per head. Nine sampled gradient records per
seed cover the first batch and every80th; all640 backward batches complete.
Final nonzero coefficient counts are49/50, all in{-1,0,+1}. There are371/50
cumulative q4 coefficient flips, no gradient clipping or
projection events, and sealed checkpoints every80 updates. Checkpoints retain
weights/RNG but not Adam moments. Complete preparation/review/delivery and
preservation are cumulatively charged separately from optimizer time.

## Loaded results and attribution

| Parent | Legacy original/stress | Zero-q4 original/stress | Fitted source/native original/stress | Scalar geometry ablated original/stress |
|---|---|---|---|---|
| Seed1 |98/104|99/104|98/104|99/104|
| Seed2 |128/128|128/128|128/128|128/128|

Each panel has128 rows. Versus zero, seed1 original restores row113 and loses
rows72/102. All fitted predictions match the legacy parent. Preserve both
fitted and zero alternatives and these three changed rows. No answer count
improvement is claimed.

Mean answer CE derived from saved native logits is mixed. Seed1 original moves
0.938582786→0.938978624; stress0.901063612→0.894337295. Seed2 original moves
0.286883716→0.286855787; stress0.281329450→0.281315631. First/last training
batch averages use different draws and are not a matched loss comparison.

The copied scalar ablation removes only retained-root/category/held-root/valid
coefficients, leaving learned token/bias and native context/addresses/values
unchanged. It reverses the same three seed1-original predictions. Head1 null
mass medians move from zero0.09234/0.07670 to fitted0.99418/0.99310 for seed1;
ablation lowers them to0.14360/0.12046. Thus scalar features affect the actual
mixture, including harmful and helpful answer changes. The present query-token
and validity collinearity prevents interpreting this as semantic geometric
history use. Seed2 answer saturation similarly prevents a predictive advantage
claim despite changed scalar scores and small CE changes.

Maximum measured scalar source/native error is11 Q24 units, approximately
6.56e-7 nat, over57952 actual head/position scores. That is an isolated scalar
comparison. Maximum downstream actual-position logit difference is recorded
in the receipt and additionally includes real-softmax/F32 versus LUT/Q24/Q16
reduction; it is not isolated scalar error. No source/native answer differences
occur. Frozen model parameter hashes and all bound producer/source files remain
unchanged; actual legacy answer logits and query read traces reproduce the
construction controls exactly.

## Decision and next dependency

Retain the fitted geometric NoRead producers as compiled, answer-connected
component candidates, with zero and ablated controls and all negative rows.
End this fixed-dose scalar comparison. It supports an implemented and learned
replacement boundary, not a universal donor-history representation theorem or
a reason to repeat the same scalar campaign.

Advance geometric output/composition design from the actual read.out/residual/
decoder callers and preserved mechanisms, retaining offline donor-weight and
operator compilation. Inspect the remaining value/potential coefficient-width
obligations before selecting the next connected implementation. Do not hide a
dense transformer in tables, restore runtime Q/K, or substitute perfect
imitation for learning. Natural query/span/role/scope/temporal inputs, unknown
record absence, bounded candidate/parameter access and the whole saved native
conversation/memory and coding/reasoning path remain separate programme work.
Claude's D19/session work remains concurrent and unchanged by this experiment.
