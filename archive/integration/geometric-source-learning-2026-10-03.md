# Geometric source learning and output support — October 3

The geometric occurrence consumer now reuses its immutable prepared q4 context
within one batch/update, instead of reconstructing all 8,963,136 coefficients
and expanded tables for every answer token. The prepared object must be dropped
before optimizer mutation. Protocol-2 labels use the actual leading content
space and include EOS. The Rust driver measures complete shortest/longest B8
answers, retains hard-payload changes, independently reloads checkpoints, and
uses its own generated prefixes during fit evaluation.

## Measured cost and the representation blocker

At source `8afe768054cfcc409b8f25f17e88f1a9563166e9`, the zero-update cost witness
runs 32 and 58 teacher-forced response tokens in two B8 batches. Context,
potential and NoRead all receive finite nonzero credit. Packed native payloads
remain unchanged and independent source/native reload preserves traces.
Driver time is 23.954s; process real time is 24.98s and monitored wrapper elapsed
is 25.760s. Maximum RSS is 590,348,288B; peak memory footprint is 942,736,584B,
a different OS measure. This is development cost with a frozen floating parent,
not optimized serving throughput or fitted language.

The all20 source-boundary audit, helper source `6033186b` with linked production
library `8afe7680`, executes 114 frozen-parent inferences and zero updates.
All20 canonical first target tokens are absent from the raw source token pool.
For example, raw `singer` is `[85,300,284]`, whereas the literal-role reply starts
with `[1130,284]` representing ` sing` and `er`. The raw source bytes are correct;
the response boundary changes BPE segmentation.

For an unsupported token y the current hybrid has
`P(y) = beta * P_parent(y)`. Every higher-probability parent token stays higher:
a source competitor also receives nonnegative copy mass. Setting beta to zero
leaves y at zero. At these fixed teacher-forced prefixes, changing occurrence
weights or NoRead cannot make y win. Unsupported-target CE changes NoRead mass,
not value-specific copy support. This statement concerns the canonical token
sequence, not every alternative tokenization, accepted rendering or sample.

A source-only view `encode(" " + exact_source_bytes)` covers all20 first labels.
However, the period is unsupported and outranked at all20 canonical value-complete
prefixes: parent top is ` is` in19 and ` sounds` in1. EOS is parent rank1 after
the period, which does not prove stopping under the copy mixture. Across114
labels, raw source lacks60 targets and has40 ordering-blocked positions;
the lexical view lacks40 and leaves20 ordering-blocked positions. A64-update
hybrid fit is therefore NOT_ADMITTED; no optimizer update was run.

## Label-free lexical repair

`geometric_source_emission_view` compiles exactly one literal ASCII space plus
the original source bytes with the bound byte-BPE tokenizer. It never trims,
normalizes or appends answer punctuation/EOS. Original token IDs and bytes remain
unchanged. Each derived token records half-open rendered/source byte intervals,
original overlapping-token intervals, and whether it includes the separator.
Merged/split tokens and partial UTF8 pieces are supported without replacing bytes.
The derived reader frame preserves supplied record/commit and typed metadata,
and rejects different original token IDs. View offsets are explicitly distinct
from original occurrence offsets.

The numeric reader remains the same q4 H4 context/potential/NoRead mechanism.
Artifact schema /2 binds the lexical policy and tokenizer; /1 remains the raw
source format. Reader entry points reject crossing the two representations.
The adapter and trace wrapper allocate outside the numerical kernel. No
allocation-free or complete native serving claim follows from this repair.

The changed source-view path passes at `b5320a26`: 32+58 complete answer tokens,
nonzero context/potential/NoRead credit, exact exported-payload reload, raw/view
cross-entry rejection and corrupted-policy rejection. Zero updates; packed
coefficients unchanged. The monitored worker takes19.686s, report-internal18.199s,
maximum RSS1,014,792,192B. Short/long B8 backward times are2.956/5.081s. The driver
also refuses the blocked view fit before claiming an output or loading a model.
This validates lexical admission and connected credit, not complete replies.

## Next geometric output mechanism

The implemented bounded candidate removes the frozen parent from source realization:
learn geometric scores for Copy occurrences, EmitPeriod and Stop, sum head Q24
logits before one integer normalization, then sum all action masses yielding the
same token. Punctuation/EOS identities come from tokenizer/protocol, never answer
labels. All actions remain available at every structurally legal prefix; no
scripted copy cursor, forced period or forced stop is adopted.

At source `ab4d3d7a`, `geometric_source_realizer` now connects the existing H4
occurrence scores to Copy, existing NoRead projection to Stop, and an independent
seeded q4 projection of the same family to Period. It reuses prepared context per
update. The loss has native alias-summed probability forward and the declared
biased adjoint `-(1/p_native) * d(p_soft)/dtheta`; it is not the derivative of
integer rounding/lookup or ordinary softmax CE. Zero native target mass rejects.
The generation loop takes the integer token winner and its own prefix, without
parent scores or labels. The checkpoint is used for offline identity admission,
then dropped. The research loader verifies floating source shadows offline;
a complete product-serving loader remains outside this result.

Eight view/action tests and two realizer loss/reload tests pass locally at
`ab4d3d7a`, including Period-to-earlier-context credit. Independent source review
approves this bounded mechanism. The measured construction and first fit below
extend the test evidence. Ordering/progress collisions, repeated-token mass and
ambiguous same-token action provenance remain explicit risks. This
scope is bounded selected-source realization; free prose, NotFound replies,
coding/reasoning and general conversation are unfinished.

Transformer superiority and an ordinary-reader win remain later attribution
questions, not first-fit gates. Preserve accepted parents and historical negative
rows. Do not repeat an unchanged hybrid dose or retire geometry from this support
failure. Keep `stack_grounded_session.rs` untouched pending #1552 coordination.

## Actual native construction and fixed fit

At frozen source `ab4d3d7af45c8da5e1d967ee86abd19e1caaaddf`, native construction
passes complete shortest/longest B8 credit (32+58 tokens), four active gradient
families, unchanged coefficient payloads and exact exported-payload reload.
The monitored worker takes15.804s; maximum RSS1,081,098,240B. The slower B8 takes
5.132s. Its untrained own-prefix generation produces0/20 complete replies and
20/20 EOS. This admits the declared native fit; it does not admit the old hybrid.

One64/B8 AdamW fit completes with LR0.003, decay0, norm clip1 and quarter-range
projection. The worker takes380.667s (report378.171s), maximum RSS2,453,618,688B;
peak memory footprint3,151,809,416B is a separate metric. Five CPU threads,3GiB
RSS and1200s worker limits were configured. No GPU or hosted research runner
was used. All four learned packed payloads change; the fixed exp table does not.
File-name inventory equality is not coefficient-byte equality. The final exported
artifact independently reloads with exact traces, and generation loads it from disk.

Complete own-prefix replies improve0→4/20; all20 reach EOS in both artifacts and
all final decoded byte strings are valid UTF8. The four successes are repeated
presentations of the source value `dancer`, not four distinct learned values.
`singer` changes from `er.` to `er.er.er.`; `Brimfold` changes from `f` to
`Br Brold Brim.`; `Louston` becomes an immediate EOS. Every row is retained in
[the evidence](../evidence/geometric-source-learning-2026-10-03.json).
Independent saved-result review checks all96 generated steps: summed head
scores, token masses and greedy decisions agree. The20 rows contain five distinct
numerical source/query inputs, and no complete observed state repeats within a
reply. Suffix repetition does not establish a geometric-state cycle or collision.
Saved batch64 traces precede update64; they cannot substitute for final-artifact
correct-prefix evaluation or be mixed with final scores to assert a collision.
These are exposed development results. No heldout, free-prose, shared-session,
geometry-superiority, energy or complete-chat claim follows.

**Next causal decision:** compare correct-prefix and own-prefix integer ranking,
then audit exact scorer inputs
without optimizer updates. Separately inspect first-source-token ranking,
re-entry after Period and premature Stop. Compare the exact available scorer
inputs before asserting a state collision. Conflicting desired decisions with
identical inputs justify added learned geometric occurrence/progress state;
distinguishable inputs require a training-coverage/hard-versus-relaxed credit
diagnosis. Do not impose an authored cursor or forced suffix, and do not repeat
an unchanged64-update dose. The wider correct-prefix
audit is proposed, not executed; the saved own-prefix audit above is complete. DeepSeek's CPU
work takes priority over a new local run; GitHub runners are excluded from
research runs by owner direction. A GPU port is separate implementation work.

## Retained evidence

Local roots under `~/uor-r4-local/workspace/research/geometric-source-learning-20261003`:
`cost-1`, `boundary-audit-1`, `source-view-1`, `native-construction-1` and
`native-fit-1`, each exclusively claimed, sealed and verified by the Rust driver.
The fit retains checkpoints at16/32/48/64 updates and the final source/native artifact.

Cost executable SHA256:
`8609afc6aa0e29b6d1e96b72825f034581645c34f0983f8dcaeb4b38bac48791`.
Boundary executable SHA256:
`52e76ab55b2396570278062da86a0deec3e840b3f8ce4bbe592c1b459cd791b0`.
Audit helper and linked library source identities are deliberately separate.
All20 cases are exposed development, not a final held-out evaluation.

Complete preparation/build/model/review/delivery/remaining cleanup is charged as
a conservative3.5-hour estimate, separately from measured workers. The shared
ledger moves1,039,035,028→1,051,635,028ms under the unchanged1,130,000,000ms limit.
The41.058s interrupted source-binding build and444.928s malformed-fixture failed
test attempt are retained and included; neither is model-quality evidence.
The measured fit remains bound to `ab4d3d7a`; subsequent main merge/documentation
commits do not rename that executable identity. CUDA delivered by another lab
is preserved in the merge and is not validated or adopted by this CPU result.
