# Source-backed grounded answer campaigns (#954)

Status: **C1-SB5 PAIRED-QUERY BINDING PREFLIGHT NEGATIVE / NO QUALIFIED ANSWER ARTIFACT**
Parent programme: #820
Working model: #1017 six-layer R4/Spin causal-softmax checkpoint
Final source-free correctness terminal: still blocked by parked #973

## C1-SB0 historical rationale

PR #1022 made the #1017 checkpoint directly usable through `r4 generate` and
added an opt-in Apple Accelerate CPU-BLAS build for local Apple Silicon use.
On the project M1, a 64-token continuation completed in `0.220401125` seconds
of recorded generation time (`0.67` seconds process wall time) and remained a
coherent child/mother/oak-tree scene. All six R4/Spin attention layers ran, the
causal and R4 audits matched, and future reads were zero.

The same checkpoint did not yet behave as an answer model:

- `Context: The sky is blue. Question: What color is the sky? Answer:` did not
  produce the supplied answer; and
- a natural cloze ending in `the sky was` generated `dark`, contradicting the
  supplied sentence.

Those observations locate the product seam. Attention and fluent generation
work; grounded answer formatting and abstention have not been taught. No new
attention mechanism, capacity run, or backend comparison is needed first.

## Prototype

`C1-SB0` adds two deliberately small pieces:

1. `r4 answer --source-file <path> --question <text>` binds exact source bytes,
   runs the existing #1017 R4/Spin generator once, and serves only an exact
   contiguous source span. `ABSTAIN`, `CONTRADICTION`, empty output, and
   unsupported generated text remain typed outcomes. Raw model text is retained
   only in the audit report.
2. One fixed MPS supervised fine-tune starts from a copy of #1017 and teaches
   the exact answer interface using deterministic supported, unsupported, and
   conflicting-context examples. It does not alter the architecture, tokenizer,
   R4/Spin attention path, or Rust loader.

The training budget is fixed at 384 optimizer steps with no sweep. At the
measured conservative #1019 rate of `3.491307 s/step`, the upper estimate is
about 22 minutes 21 seconds. Apple Accelerate is used for the subsequent local
Rust behavior run; MPS is used only for this offline fine-tune.

## Decision

All three frozen unseen product probes must pass in the same run: the supported
question must return its exact source span, the unsupported question must return
typed `ABSTAIN`, and the conflicting source must return typed `CONTRADICTION`.
Failure keeps work at this answer-format/data seam; it does not trigger another
attention, geometry, capacity, or hardware campaign.

## Scope boundary

A positive result establishes only a source-backed prototype answer/abstention
surface for this small model and prompt policy. It does not establish semantic
entailment, general factual correctness, reasoning, intrinsic-geometric
advantage, source-free/table-native inference, transformerlessness, browser or
WASM readiness, frontier quality, or release readiness.

## Observed result — 2026-08-31

The fixed run completed on the project Apple M1 through PyTorch MPS without CPU
fallback or CUDA. All 384 optimizer steps finished in `883.7735486670863`
seconds (14 minutes 44 seconds), versus the conservative 22-minute estimate.
Answer-token development loss was `0.0013607110659437775` at step 192 and
`0.00014458666336423967` at step 384. The export is bound by:

- weights CID `blake3:0949d925c1189cdc784a4b93bbb9119da5c1a573fe3385c3b3c8f82a299c9439`;
- final manifest CID `blake3:c2c5b163485e39f14ba0a7869d83fab8c641ed061b5c0e8e5783924aff187b2f`;
- run-contract CID `blake3:31b6d7776a42779d0b8daddfc8034f9984dc9f85476eb0bd5df713cd7624f589`; and
- training-result CID `blake3:f957c11f668a9a81098f4b2ad9cac9af75d3636b5a2c6cc1a796c5817be10ac9`.

The Rust product command then loaded that exact weight CID through Apple
Accelerate CPU BLAS and ran all three frozen prompts. It emitted exact
`ABSTAIN` plus EOS for every prompt:

| Frozen prompt | Required outcome | Observed outcome | Verdict | Decision CID |
| --- | --- | --- | --- | --- |
| supported amber-coin source | exact supported sentence | typed `ABSTAIN` | FAIL | `blake3:63024db98ceb1cfdae5b106ddae0d611a027cc160578452c78ef6f4fda18bfe7` |
| unsupported velvet-ribbon question | typed `ABSTAIN` | typed `ABSTAIN` | PASS | `blake3:de1baea232dcb93bbae199e48ea3cdddc5c10de7da7c59e0b416dd0454eae35a` |
| conflicting amber-coin source | typed `CONTRADICTION` | typed `ABSTAIN` | FAIL | `blake3:b89b10fcff8c88cc7ed0d39c3d9c3293e0aa619d2fe34d7d044e6f00dfec4383` |

Terminal: **`FAIL_GROUNDING_PRODUCT_TRANSFER_ABSTENTION_COLLAPSE`** (`1/3`).
The low same-vocabulary teacher-forced development loss did not transfer to the
reserved lexical items under autonomous decoding. This is a copying/generalized
span-selection failure at the answer interface, not an attention-backend or
hardware failure. The run is not retuned or repeated.

The next successor should replace free-form answer generation with a learned
source-span pointer/copy head plus explicit abstention and conflict scores over
the already established causal R4/Spin states. That mechanism must be frozen on
new lexical families before a newly reserved product population is revealed.
Do not return to #1019 capacity, resonance, intrinsic-attention substitution, or
backend comparison for this failure.

## C1-SB1 observed result — 2026-08-31

The independently frozen `R4SourceSpanPointerV1` successor retained the #1017
weights, exact causal R4/Spin state capture, and deterministic source-copy
surface. Its cheap 12-record overfit preflight passed `12/12`. The public
Python/Rust score fixture also passed: maximum absolute score delta was
`1.234420776e-7` and maximum absolute logit delta was `1.428717041e-6`, both
within the frozen `0.01` ceiling.

The sole 256-step fit then completed and produced these development results:

| Frozen development gate | Observed | Required | Verdict |
| --- | ---: | ---: | --- |
| answer decision | `89/128` (`69.53125%`) | `>=95%` | FAIL |
| abstain decision | `114/128` (`89.0625%`) | `>=95%` | FAIL |
| conflict decision | `117/128` (`91.40625%`) | `>=95%` | FAIL |
| supported source-span pointer | `121/128` (`94.53125%`) | `>=95%` | FAIL |

Terminal: **`FAIL_SOURCE_SPAN_POINTER_DEVELOPMENT_GATE_STOP`**.

Because every development threshold was binding, the run stopped before
emitting a final pointer artifact. The three reserved product probes were
`NOT_RUN`; browser and HTTP wiring were also `NOT_RUN`. The implementation now
accepts the exact question form `Where is the <subject>?`, two to eight exact
punctuation-terminated sentence spans, and an explicit `--head` artifact, but
there is no qualified final head. The default `r4 answer` product surface is
therefore unavailable because no explicitly qualified artifact exists.

Do not tune or retry the revealed positive-diagonal weighted-cosine head. The
next proposed mechanism is a source-relative learned relation/entailment head
that preserves the exact R4/Spin state-capture and source-copy seams. It must
receive its own independent frozen contract before execution; no successor run
or product wiring is active. #1017 remains the working coherent-generation
prototype, and ordinary causal attention remains established at its existing
claim scope. The compact bound aggregate is
[`r4_source_span_pointer_954_raw.json`](r4_source_span_pointer_954_raw.json).

## C1-SB2 source-relative relation-head result — 2026-08-31

`R4SourceRelativeRelationHeadV1` replaced subject/sentence cosine similarity
with one question-conditioned representation per candidate:

```text
Evidence:
<exact source sentence>
Question:
Where is the <subject>?
```

There is no terminal newline. The immutable #1017 executor retains the final
width-288 normalized residual at the question-mark token after all six coherent
R4/Spin causal-softmax layers. A fixed `288 -> 32 ReLU -> 1` probe (9,281
parameters) assigns one relation logit. Only `logit > 0` authorizes a candidate;
zero is negative. Exact duplicate sentence text collapses before the decision:
zero unique positives abstains, one copies an original occurrence, and two or
more distinct positives report contradiction.

The new population removed the shortcut found in C1-SB1. It contains 3,360
construction and 420 development records across answer/abstain/conflict and
source widths 2 through 8. Construction and development subjects and exact
sentences are disjoint. Raw queried-subject occurrence count is ambiguous in
both splits. The 420 reversed-source and 335 same-source query-swap controls are
also bound, while four product records were committed before fitting and never
feature-extracted, scored, or evaluated by Python.

Before any full-population extraction or fit, the frozen cheap gate trained on
12 motifs from two lexical families and evaluated 12 homologous motifs from two
unseen families. The 256-step MPS probe reduced fit loss from
`0.7386124730110168` to `0.14383068680763245` and fit every required relation:

| Preflight metric | Fit | Sealed lexical transfer | Required |
| --- | ---: | ---: | ---: |
| answer decisions | `6/6` | `0/6` | exact |
| abstain decisions | `4/4` | `3/4` | exact |
| conflict decisions | `2/2` | `1/2` | exact |
| positive-relation recall | `12/12` | `5/12` | exact |
| negative-relation specificity | `20/20` | `14/20` | exact |
| supported copied span | `6/6` | `0/6` | exact |

Candidate-array order equivariance passed, but the matched same-source,
query-swap, duplicate-agreement, and distinct-value conflict controls did not.
The terminal is **`FAIL_MATCHED_TRANSFER_PREFLIGHT_STOP`**. Result CID:
`blake3:ce3f06fd4962ac72127bb7dc0ca4123f89478047acd302481704d1f1b3f4ebaf`;
manifest CID:
`blake3:14d04e0a6fe4ffd65c3fe1d63ede3425262c3de8351733021d5f7dbd0aa3c493`.

The gate did its job in about 20 seconds: it showed exact fit-family
memorization without independent lexical transfer. Python/Rust relation-logit
parity, the sole 512-step full fit, development evaluation, final head emission,
and all four product probes are `NOT_RUN`. No C1-SB2 head is qualified for the
default `r4 answer` surface.

Do not tune or retry this frozen residual probe and do not return to attention,
capacity, resonance, backend, or corpus-volume campaigns for this failure.
Ordinary causal attention and coherent bounded generation remain established at
their prior scopes. The missing capability is now localized more sharply:
source-relative entailment is not a lexically transferable feature of the
frozen #1017 terminal residual under this probe. The next proposed mechanism
must train relation supervision into the representation itself—through the
existing R4/Spin attention path—while retaining the exact-copy and typed
non-answer Rust boundary. It must be frozen independently before execution.

The compact bound aggregate is
[`r4_source_relation_head_954_raw.json`](r4_source_relation_head_954_raw.json).
The final source-free #954 terminal remains separately blocked by #973, and
#955 reasoning remains downstream of a positive correctness result.

## C1-SB3 attended-relation adapter result — 2026-08-31

`R4AttendedRelationAdapterV1` moved relation supervision into the existing
#1017 representation instead of training another probe over frozen terminal
states. The #1017 base remained frozen while rank-8, alpha-8, dropout-zero LoRA
adapters trained `q_proj`, `k_proj`, `v_proj`, and `o_proj` in all six R4/Spin
causal-softmax layers. There is no trainable classification head. Each
candidate uses the exact input below, with no terminal newline:

```text
Evidence:
<exact source sentence>
Question:
<question>
Supported:
```

The fixed decision score is the tied token-logit difference for token ID 1771
(`yes`) minus token ID 542 (`no`) at the final input position; a score greater
than zero marks a candidate supported. This produced 110,592 trainable adapter
parameters. The implementation can merge them into an ordinary checkpoint,
but the failed exact gate stopped before any adapted checkpoint was emitted.

The first attempt-01 alignment report paired scores with the wrong candidate
rows. It is invalid and is **non-evidence**. The result below is the corrected
exact replay over the original frozen rows and aggregation policy:

| Frozen metric | Base sealed | Trained fit | Trained sealed | Requirement |
| --- | ---: | ---: | ---: | ---: |
| answer decisions | `0/21` | `41/42` | `19/21` | exact |
| abstain decisions | `21/21` | `42/42` | `19/21` | exact |
| conflict decisions | `0/21` | `41/42` | `18/21` | exact |
| positive-relation recall | `0/76` | `150/152` | `73/76` | exact |
| negative-relation specificity | `239/239` | `478/478` | `234/239` | exact |
| supported copied span | `0/21` | `41/42` | `19/21` | exact |

The final optimizer loss was `0.04814816266298294`. Corrected fit mean binary
cross-entropy was `0.01836039125919342`; corrected sealed mean binary
cross-entropy was `0.12464728206396103`. The representation-delta audit passed:
all `24/24` targeted Q/K/V/O tensors changed and remained finite, while every
non-attention tensor remained unchanged. The corrected result is bound by:

- result CID `blake3:b55bd25715705450742d80f9ecde56cdd965beefb1c122e08b477b03d0949b92`; and
- manifest CID `blake3:e0faa45bb80b122d0e8a31ac6c582fd4b44b39ee20f5d49aeaca2f07deb21e71`.

This is a **positive mechanism-transfer result**: unlike the frozen C1-SB2
readout, supervision applied through the existing R4/Spin attention path
transferred most supported, unsupported, conflict, and copy decisions to the
sealed lexical families. It is nevertheless an **exact-promotion negative**:
both fit and sealed evaluation missed binding exact criteria. The campaign
therefore stopped before Rust parity, the full fit, development evaluation, or
product probes; each is `NOT_RUN`. No C1-SB3 checkpoint is qualified for the
default `r4 answer` surface.

The observed internal result reused whole-population exactness for three named
control booleans rather than isolating their individual motif populations.
Those booleans are therefore `NOT_CLAIMED`; they did not decide this terminal
because the independently reported fit and sealed semantic gates had already
failed. The producer now evaluates future named controls separately.

Do not retry this candidate-independent binary objective with another seed,
rank, learning rate, or threshold. The next successor should be a freshly
frozen joint-source/candidate-set mechanism trained with a record-level
structured margin over answer, abstain, conflict, and exact-copy outcomes. That
is a new objective and information flow, not a C1-SB3 retry: it must score the
whole candidate set together so the learning signal matches the downstream
record decision. Preserve the established exact-copy and typed non-answer Rust
boundary. This result does not revise #1017's bounded ordinary-attention or
coherent-generation claims, establish an intrinsic-geometric advantage, or
unblock #955. The final source-free #954 terminal remains separately blocked by
#973.

## C1-SB4 joint-source candidate-margin result — 2026-08-31

`R4JointCandidateMarginAdapterV1` implemented the independently frozen
successor rather than retrying C1-SB3. Every distinct exact-text candidate was
scored from the complete source and question:

```text
E:<exact full source>
Q:<question>
C:<exact distinct candidate text>
Supported:
```

The fixed tied-token score remained `yes[1771] - no[542] > 0`. Rank-eight,
alpha-eight, dropout-zero LoRA updated Q/K/V/O in all six existing attention
layers; there was no learned head. The complete-record objective used margin
one:

```text
relu(1 - minimum positive-group score)
+ relu(1 + maximum negative-group score)
```

An absent positive or negative set contributed zero for that term. Exact
duplicate text collapsed to one scored group and copied back to the earliest
exact occurrence. The committed population contained 126 fit records and 604
distinct groups, 63 independently sealed records and 302 groups, and four
opaque product commitments. Exact sentences and complete generated composite
world items were disjoint; primitive component vocabulary was deliberately
shared. The longest prompt occupied 221 of 256 positions including BOS.

The sole MPS run used seed 9544, 270 optimizer steps, seven complete records per
step, and the frozen step-eight/600-second wall gate. It completed within the
budget. Structured-margin loss moved from `2.2566068172454834` to
`0.8698721528053284`, and the representation audit passed: all 24 targeted
Q/K/V/O tensors changed and remained finite, while no non-attention tensor
changed.

The semantic gate nevertheless failed exactly and symmetrically:

| Frozen metric | Untrained sealed | Trained fit | Trained sealed | Required |
| --- | ---: | ---: | ---: | ---: |
| exact records | `21/63` | `70/126` | `35/63` | exact |
| answer outcomes | `0/21` | `14/42` | `7/21` | exact |
| abstain outcomes | `21/21` | `14/42` | `7/21` | exact |
| conflict outcomes | `0/21` | `42/42` | `21/21` | exact |
| positive-group recall | `0/63` | `126/126` | `63/63` | exact |
| negative-group specificity | `239/239` | `394/478` | `197/239` | exact |
| supported copied span | `0/21` | `14/42` | `7/21` | exact |

Duplicate agreement and distinct-conflict subsets were exact in both fit and
sealed partitions. Same-source query relocation was not exact in either. The
source-order reversal control was correctly `NOT_RUN_MAIN_GATE_NEGATIVE`.
Terminal: **`FAIL_JOINT_CANDIDATE_MARGIN_PREFLIGHT`**.

The result is bound by:

- result CID `blake3:82f83d865eaea24589cf8acdbcc4c83fd4714041c1d80e31a818d587664b7b84`;
- manifest CID `blake3:4872dd1b70cebbd7c2cf9930389e73df52e2325f809b0d0e27763a588a88b04f`;
- tree CID `blake3:5bcf79dfaef01b5357b831af5990bb36d5dc21a6c193a7c051eb701b79ba551e`;
- run-contract CID `blake3:0b621e5a69b5660bd9f9df47cb3c4ce6d0dbe6d0f3d9f4236f8da4435686d02a`.

The complete published metric pattern is reproduced exactly by the deterministic
rule “predict supported iff the candidate text contains ` is inside `,” while
ignoring the question: `70/126` and `35/63` exact records, every positive and
negative count, every outcome count, and every copy count all match. Per-row
model scores were not retained, so this is an aggregate-equivalent shortcut,
not a proved description of the model's internal computation. It nevertheless
shows that the observed evidence does not distinguish real subject-question
binding from affirmative-locative syntax.

Per the frozen decision, C1-SB4 stops here without retry, Rust parity, larger
fit, development evaluation, checkpoint emission, or product reveal. The four
product records remain committed and unopened. A genuinely distinct successor
must make question conditioning identifiable at the objective itself: couple
multiple questions over the same exact source and require the same candidate
to change sign when the queried subject changes. That proposed paired-query
conditional-binding rung must receive its own independent freeze; it is not an
authorized C1-SB4 seed, threshold, rank, schedule, or corpus retry.

The compact aggregate is
[`r4_joint_candidate_margin_954_raw.json`](r4_joint_candidate_margin_954_raw.json).
This negative does not revise #1014/#1017 ordinary-attention or bounded coherent
generation evidence, establish intrinsic geometry, or unblock #955. #954's
final source-free terminal remains separately blocked behind #973.

## C1-SB5 paired-query conditional-binding result — 2026-08-31

`R4PairedQueryCandidateMatrixV1` implemented the independently frozen response
to C1-SB4's question-ignoring shortcut. Each first-class item held one exact
source and two different subject questions. Both lanes used the input below,
with no terminal newline:

```text
E:<exact full source>
Q:<question>
Bind:
```

The candidate columns were the final punctuation-token states of the earliest
source occurrence of each distinct exact-text candidate group. The two lanes
had bit-identical source prefixes, and the query row used the final `Bind:`
colon after all six established R4/Spin ordinary causal-softmax layers. Rank-8,
alpha-8, dropout-zero LoRA updated Q/K/V/O in every layer. A separate asymmetric
rank-32 head scored `dot(Wq*hq,Wc*hc)/sqrt(32)+b`; only a score greater than zero
marked a candidate supported. LoRA plus the three-tensor head exposed 129,025
trainable parameters. The eight four-lane head blocks are bookkeeping, not an
intrinsic-geometry claim.

The fresh population began at world ordinal 162. It contained 56 fit pairs and
28 independently sealed pairs across widths 2 through 8. Every pair included
at least one candidate whose label flipped between its two questions. The fit
matrix contained 112 query rows, 266 candidate groups, 532 cells, and 98 flip
columns; sealed contained 56 rows, 133 groups, 266 cells, and 49 flips. Exact
sentences and complete composite world items were disjoint from C1-SB3,
C1-SB4, and the other partitions; primitive component vocabulary remained
shared deliberately. All candidate anchors preceded `Q:`, paired source token
prefixes reproduced exactly, and the longest fit/sealed input occupied 189/201
of 256 positions including BOS. Four opaque product pair commitments were
created during preparation and remained outside the training view.

The sole MPS optimizer used seed 9545, one pair for each of seven widths per
step, 120 steps, 15 complete epochs, and 7,980 matrix-cell presentations. The
objective was mean margin-one row loss plus mean margin-two flip-column loss.
Loss moved from `5.9824323654174805` to `0.0`. The step-eight ETA gate passed,
and the optimizer completed inside its 300-second ceiling; exact elapsed timing
was deliberately omitted from the content-addressed result.

Fit was exact, but lexical transfer was not:

| Frozen metric | Trained fit | Trained sealed | Requirement |
| --- | ---: | ---: | ---: |
| exact pairs | `56/56` | `14/28` | exact |
| exact query rows | `112/112` | `37/56` | exact |
| cell signs | `532/532` | `227/266` | exact |
| flip columns | `98/98` | `38/49` | exact |
| supported copies | `42/42` | `13/21` | exact |
| duplicate pairs | `14/14` | `4/7` | exact |
| answer outcomes | `42/42` | `13/21` | exact |
| abstain outcomes | `42/42` | `14/21` | exact |
| conflict outcomes | `28/28` | `10/14` | exact |

Fit mean loss was `0.0`; sealed mean row-margin loss was
`2.1721223763057163`, mean flip-margin loss was `0.34822897035248424`, and
total mean loss was `2.5203513466582006`. The miss was not isolated to one
pair construction: the four pair slots reached `3/7`, `4/7`, `3/7`, and
`4/7`. The descriptive sealed width/world counts were `4/4`, `0/4`, `4/4`,
`4/4`, `0/4`, `0/4`, and `2/4` for widths 2 through 8. Each width had exactly
one sealed lexical world, so that pattern cannot identify a causal width
effect.

The causal controls sharpen the result without rescuing qualification.
Corresponding candidate states were bit-identical in all `56/56` fit and
`28/28` sealed pairs. Identity-aligned row swapping reproduced the complete
matrix trace and aggregate exactly. Pair-mean query ablation produced identical
rows in `28/28`, scored `0/28` exact pairs, and increased loss. Turning
attention off also scored `0/28` and increased loss. Deterministic evaluation
replay was exact. All 24 targeted attention tensors and all three binding-head
tensors changed and remained finite; no non-attention base tensor changed.
Thus distinct query states and active causal attention were load-bearing for
the observed behavior, but the independently sealed semantics still failed.

Terminal: **`FAIL_PAIRED_QUERY_BINDING_PREFLIGHT`**. The result is bound by:

- result CID `blake3:076242ab2bd379083ae55a22a272b3a0943b350fa301f65909e1b0ecc0d72571`;
- manifest CID `blake3:c4a7ec5e4926cc5ede144d7f3c013940d104bb77ce26f8e201aa63c524c6a119`;
- tree CID `blake3:0c3b0d869c45fca9924c1d76af19c9b411d8529c6ac8f9f7bf9d641d788dc188`;
- run-contract CID `blake3:992bc9bb61c3e7bb4481cee3b959db338aa68530ffe43e93ee0eac5c133d2103`.

The training view recorded zero product or forbidden reads, and the result
manifest contains no checkpoint or binding-head artifact. Product evaluation,
Rust parity, development evaluation, and any larger fit are `NOT_RUN`.
Per the independent freeze, C1-SB5 is retired without retry, parameter change,
larger fit, checkpoint emission, or product reveal.

This is a bounded causal-mechanism result and an exact-qualification negative:
the architecture fit paired subject binding perfectly, and query/attention
ablations destroyed its sealed behavior, but it did not transfer exact binding
to fresh lexical worlds. It does not establish intrinsic geometry, source-free
correctness, reasoning, transformerlessness, exact lowering, browser/WASM, or
release readiness, and it does not revoke #1014/#1017 ordinary-attention or
bounded-generation evidence. No C1-SB6 or other scientific run is authorized
by this result. #973 remains parked and must be independently re-scoped before
work on the source-free/geometric-attention blocker resumes; #955 reasoning
remains downstream.

The compact bound aggregate is
[`r4_paired_query_binding_954_raw.json`](r4_paired_query_binding_954_raw.json).

## Status correction — 2026-09-01

The statement above that #973 was parked records the state at C1-SB5 close; it
is not the current execution handoff. #973 has since been independently
re-scoped to
[`R4GroupAddressedRetentionLMV1`](r4_group_addressed_retention_973.md). Its
corrected exact-H4/cyclic-120/scrambled-H4 geometry and frozen population pass
construction-only audits. The MPS training/timing gate, disposable smoke, main
optimization, and held-out model scoring remain `NOT_RUN`; no attention,
H4-advantage, generation, correctness, or reasoning claim follows. #954's final
source-free terminal remains blocked behind #973. C1-SB6 remains unauthorized.

## #973 terminal follow-up — 2026-09-01

The group-addressed construction gate referenced immediately above has now
completed. `R4GroupAddressedRetentionLMV1` terminated
`UNAVAILABLE_FRAME_POPULATION_OR_LOCAL_BUDGET`: geometry, population,
reachability, gradients, memory, equal work, and held-out sealing passed, while
timing and disposable learning smoke failed. Main optimization and held-out
model scoring are `NOT_RUN`; there is no attention or H4-advantage verdict. The
exact cell will not be retried or tuned. #973 must next scientifically select
and independently freeze a fuller source-free decoder block. #954 remains
blocked and C1-SB6 remains unauthorized. See the
[canonical #973 record](r4_group_addressed_retention_973.md).

## #973 CPU-recovery follow-up — 2026-09-01

The statement immediately above remains the historical terminal for the smaller
group-addressed cell. #973 then independently froze and completed
[`R4GroupAddressedRetentionDecoderV1CpuRecovery`](r4_group_addressed_retention_decoder_cpu_recovery_973.md)
on deterministic Apple Accelerate CPU BLAS. All 512 construction steps completed
in `438.117083 s`; result CID
`blake3:68355ad2f61d02dc73dbf22de4c24834815a23069ed5735630dc365081cf91db`.
Disabling retained state on the disjoint construction-validation partition lost `0.967227` nats and 182
top-1 hits, qualifying a bounded causal retained-attention component. The exact
3.17M-parameter, two-block, data/dose recipe did not satisfy its frozen
full-decoder generalization criterion: aggregate validation CE moved
`8.371911 -> 8.976155`. Scrambled transport was `0.033049` nats better, so no
H4-specific advantage is claimed.

This result does not unblock #954. It preserves one qualified attention
read/write component but does not establish coherent generation, source-free
correctness, reasoning, exact lowering, or release readiness. C1-SB6 remains
unauthorized. #973 must next independently freeze a data-supported language-path
decoder with an ordinary matched non-geometric control.
