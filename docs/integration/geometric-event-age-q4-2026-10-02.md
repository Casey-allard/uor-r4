# Geometric event and age coefficient bridge, October 2

The remaining event and age learned coefficient sources now have explicit
four-bit construction and connected offline training routes. The runtime event
controller still performs synchronous OLD-state action selection, exact signed-H4
right composition, then NEW-state capture selection. The span emits OLD held
state. Integer decisions remain authoritative; reconstructing their scores as
F32 cannot choose a different span action. Backward retains the original biased
score-softmax and tangent/Hamilton estimator. No new estimator is adopted.

The explicit Stack training routes use geometric address dispatch, bypassing the
q/k selector. Their surrounding recurrence, value/readout, normalization and
language head remain offline F32 operations. Passing a connected answer-loss
fixture is different from a useful fitted language model or whole native serving.

## Actual range finding and course correction

Source-only inspection binds the actual saved Safetensors bytes to their model,
configuration and event metadata. Seed1 has263/11712 event coefficients outside
[-1.75,1.75]; seed2 has261. Age has100/256 and102/256 outside, with minimum
-7.9375 nat. The direct unchanged quarter-nat converter correctly refuses these
parents. It performs no clipping, centering or fallback. This is a conversion
range result, not evidence against the geometric family.

The age initialization in the actual source is exactly -lag/16 and-lag/256 for
these two heads. After subtracting that fixed prior, both saved residual tables
fit±.875: seed1 head0[-.626665,.309157], head1[-.115634,.080882]; seed2
head0[-.601872,.283226], head1[-.143064,.124647]. Remaining high-lag entries equal
the initialization exactly; that equality does not prove they never received
updates. The prior is an architectural temporal bias, not learned semantics.

The adopted residual schema `/3` preserves this prior and encodes only the
independent learned head/lag residuals at fixed1/8-nat signed-q4 units. For H2:

```
prior_Q24[h,lag] = -(lag << (24-k_h)), k_h = [4,8]
age_Q24[h,lag] = prior_Q24[h,lag] + (q[h,lag] << 21)
```

H1 retains its initialized shift8. Lag zero remains explicit. The original raw
source, fixed prior/head shifts/phase, packed residuals and expanded ages are
bound and independently regenerated during load. Schema `/1` and direct
quarter schema `/2` retain their original meanings. Live training rebuilds the
hard current residual grid with a declared identity straight-through derivative;
the integer reducer itself supplies no backward graph. The same age entries
participate in the occurrence-plus-NoRead denominator, without centering.

The event construction applies one declared offline transform: all transition
families TT/ST/NT times1/2; readout families TE/SE times1/4. Then it packs the
existing quarter-nat q4 source. Positive common scaling preserves exact-real
hard argmax before quantization. Rounding and the canonical Q25 basis can change
trajectories, and the fixed-temperature surrogate probabilities change. There
is no runtime inverse scale or evaluation-selected scale sweep.

## Verification and decision boundary

At8713997d, standard hosted M1 [run37080606999](https://github.com/UOR-Foundation/uor-r4/actions/runs/37080606999)
passes6 integer and10 training/artifact cases and builds the release admission
driver. The actual answer-loss fixture reaches all5 event families, an earlier
unique token's transition, age and embedding. Source/native trace, delayed
credit, artifact/resealed tampering and native-action override cases pass. A
root codec import failure at21d347aa is retained and repaired; its known-invalid
training job was cancelled before unnecessary compilation.

At093dc5e7, hosted M1 [run37082252725](https://github.com/UOR-Foundation/uor-r4/actions/runs/37082252725)
passes4 integer and8 connected training/artifact cases and builds the release
integrated driver. Both local retained-parent fixed-conversion attempts then
complete and seal. All six learned source components (context, potential,
values, NoRead, capture events and age residuals) are admitted with four-bit
coefficients; the admission inventory has no wider learned source component.
Expanded lookup tables and the fixed age prior still have wider integer storage.

[Loaded evidence](../evidence/geometric-event-age-loaded-2026-10-02.json) retains
512 rows: parent446 correct, converted444, with1 gain and3 losses. Seed1 original
93→91 and stress110→109; seed2 original124→124 and stress119→120. Mean answer CE
changes1.121969→1.122902 /1.112053→1.137371 for seed1 and
.448722→.448524 /.573585→.571945 for seed2. The fixed event conversion changes
24 capture action positions across19 rows (7/13/4/0 positions across the four panels).
No optimizer updates are performed and no perfect parent imitation is required.

The driver independently reloads components, checks unchanged
context/value/NoRead/bank numerical payloads after dependency-envelope rebinding,
replays original saved parent answers, and verifies source/native event traces.
All512 incremental/whole-prefix comparisons match output bits at every actual
position using the same F32 tail shape. Integer reader rows carry cross-length
prefix causality; B2 uses distinct sequences with same-shape floating references.
The workers take16.16s/15.05s, peak RSS1,089,830,912 /1,010,073,600 bytes,
within two-thread/two-GiB reservations. No local Cargo or private model upload.
The two direct quarter-grid refusals remain separately sealed.

Admission refusal calls for a representation diagnosis. A trace mismatch calls
for a bridge repair. An admitted quality loss is retained evidence for a
separately justified constrained continuation, not an automatic dose, selector,
gauge sweep or family retirement. Existing all-HOLD/absent trajectories can
have no capture credit, and the tangent estimator's antipodal limitation remains
explicit. No useful natural conversation, general reasoning, geometric advantage,
whole-model native serving, sparse access or complete energy result follows here.

Next responsibilities remain connected learning of the integrated geometric
attention path, natural grounded conversation/memory qualification, removal of
the float tail, and bounded selected access. Offline donor compilation remains
available. GroundedSession and peer training lanes are untouched.
