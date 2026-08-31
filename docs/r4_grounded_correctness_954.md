# Source-backed grounded answer campaigns (#954)

Status: **C1-SB3 MECHANISM TRANSFER POSITIVE / EXACT PROMOTION NEGATIVE / NO QUALIFIED ANSWER ARTIFACT**
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
