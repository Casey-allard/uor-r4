# Position-preserving R4 causal key/value binding (#1043)

- **Status:** terminal `INVALID_POSITION_KV_BINDING`; preserved without retry
- **Parent:** #973
- **Programme root:** #820
- **Policy:** `R4PositionPreservingCausalKVBindingV1`
- **Frozen seed:** `10043`
- **CUDA:** forbidden

This is the append-only analysis, implementation contract, and eventual result
record for issue #1043.  Everything through **Frozen terminal actions** was fixed
before implementation.  After the terminal populations are materialized and
their identities committed, exactly one fit and one terminal reveal are allowed.
The later result section may append observations; it may not rewrite this
contract around them.

## Question and claim boundary

The immediate question is deliberately narrower than generation:

> Can the already source-free two-layer ordinary causal-softmax model learn to
> preserve and retrieve supplied key/value bindings when every causal position
> remains individually addressable, and can the same fitted weights execute as
> a coherent transported R4/H4 attention calculation?

A positive establishes bounded source-free supplied-context binding and an R4
gauge realization of ordinary causal attention.  It does **not** establish that
H4 improves semantics, a fixed-size recurrent replacement, a legal exact/table
runtime, autonomous generation, correctness, reasoning, browser readiness, or a
release.

## Structural diagnosis

`R4RetainedLanguagePathV1` proved a source-free causal language path, but each
layer collapses up to 120 positions into 120 group-addressed slots.  A collision
overwrites or merges distinct histories before a later query can distinguish
them.  `R4PredictiveBlockDeltaBindingV1` compressed the whole prefix further
into four banks of twelve independent `4 x 4` matrices.  Its terminal result
showed useful signal but missed absolute capacity, plain-arm, and additive-arm
gates.  Those are representation/write-path limits, not evidence that stable
softmax or causal Q/K/V attention is absent.

The closest working repository mechanism is the ordinary arm of
`R4RetainedLanguagePathV1`: two causal-attention layers, four heads, learned
Q/K/V, RoPE, stable softmax, and one distinct K/V record per causal position.
Its immutable source-free artifact is:

- path: `arms/ordinary/model.safetensors` beneath the #973 retained-language
  research root;
- bytes: `1,010,800`;
- CID: `blake3:c1cd34b36c7df7c53915785a608ccd353a11de56eebb3ecc58e74092cb5d1933`;
- parameters: `252,160`.

An analysis-only read of the already revealed V5 population gave that unchanged
artifact a mean own-minus-crossed gain of `0.00791199383456842`, `300/512`
wins, own NLL `3.6102299573622076`, crossed NLL `3.618141951196776`, and zero
forbidden reads.  This is quarantined evidence: it only rules out claiming that
the old natural-language fit already learned supplied binding.  It did not set
any population, optimizer, threshold, or terminal below and is not part of the
#1043 result.

The design follows the smallest already established associative-memory core:

- scaled dot-product causal attention and stable softmax as in the original
  Transformer and the compact incremental implementation in `llama2.c`;
- a two-layer induction composition, where one layer can carry a previous-token
  relation and the next can match/copy it;
- the multi-query associative-recall diagnostic used to distinguish
  position-preserving attention from compressed sequence mixers; and
- R4 transport as a coherent change of local frame, consistent with
  connection-valued attention.

Primary references:

- <https://arxiv.org/abs/1706.03762>
- <https://github.com/karpathy/llama2.c/blob/master/run.c>
- <https://transformer-circuits.pub/2022/in-context-learning-and-induction-head/>
- <https://arxiv.org/abs/2312.04927>
- <https://arxiv.org/abs/2607.10677>

A full gated-delta recurrence remains a plausible later compression candidate
(<https://arxiv.org/abs/2412.06464>), but putting another compressed memory in
front of this question would repeat the V5 ambiguity.  It is explicitly outside
#1043.

## Frozen mechanism

`R4PositionPreservingCausalKVBindingV1` initializes all and only the 252,160
values of the immutable ordinary artifact.  It keeps the existing architecture:

| field | frozen value |
|---|---:|
| layers | 2 |
| width | 48 |
| feed-forward width | 128, SwiGLU |
| heads | 4 |
| head width | 12 = three R4 blocks |
| context | 120 |
| output | tied vocabulary projection |
| new trainable values | 0 |

All 252,160 values are fine-tuned once.  No head, adapter, recurrent bank,
source model, teacher, provider, candidate feature, or future label is added.

### Incremental state and schedule

The implementation must expose both the existing full square forward and a real
incremental `step` path.  At token position `t`, each layer:

1. derives Q, K, and V from the observed `x_t` representation;
2. applies RoPE to Q and K;
3. appends K and V at exact cache slot `t`;
4. reads slots `0..t`, never `t+1..119`;
5. aggregates the values and completes the residual layer; and
6. produces logits whose selected token is the prediction of `x_(t+1)`.

The bounded cache contains exactly

`2 layers * 2(K,V) * 4 heads * 120 positions * 12 lanes = 23,040 f32`

or `92,160` value bytes, plus 240 per-layer validity bits, 120 exact source-frame
indices, the current length, and audit counters.  K and V are never collapsed by
group address.  This is a bounded-context reference, not the final compact or
multiply-free runtime.

### Coherent R4/H4 execution

There is one fitted weight set, not separately fitted plain and geometric arms.
For token `t`, let the exact H4 sidecar provide the canonical orthogonal frame
`F_t`.  After RoPE, split each 12-lane head into three four-lane blocks and let
`B_t = I_3 tensor F_t`.  For source position `i <= t`:

```text
q_t_local = B_t^T q_t
k_i_local = B_i^T k_i
v_i_local = B_i^T v_i
P_(i->t)  = B_t^T B_i

a_ti = stable_softmax_i(
    q_t_local^T P_(i->t) k_i_local / sqrt(12)
)

r_t_local = sum_i a_ti P_(i->t) v_i_local
r_t       = B_t r_t_local
```

For exact orthogonal frames this reduces algebraically to ordinary attention.
The sidecar is represented in `f32`, so the empirical criterion is numerical
parity, not a false byte-equality claim.  This coherent gauge lift is legitimate
R4 mechanism evidence but is not evidence of intrinsic H4 superiority.

The transport-mismatch control keeps the fitted weights, input tokens, query
frames, and identity frame fixed, but replaces source transport composition
with the sidecar's deterministic identity-fixing non-homomorphic permutation.

## Frozen populations and leakage boundary

All input payloads and labels are generated and CID-sealed before fitting.  A
population manifest, disjointness witnesses, cheap-instrument result, and run
contract CID are committed before the optimizer can run.  Terminal evaluation
payloads remain unopened by the fit process until the final artifact CID exists.

### Natural-language replay for preservation

Take exactly 21,840 existing 121-token windows from the #973 source-free
construction store.  Select without replacement by ascending
`BLAKE3(pack_u64_be(10043) || pack_u64_be(window_ordinal))`, then ordinal.
Each contributes 120 ordinary next-token decisions.  This is language replay,
not fresh evidence.

The terminal nonregression population contains exactly 2,066 nonoverlapping
121-token windows and 247,920 decisions.  It is selected from the #1019
nonsealed training view by continuing strictly after the last V5 fresh-language
source story, excluding the complete story-CID union used by the checkpoint,
V1-V5 prompt populations, #1041, the mixed replay rows, and this issue's other
populations.  If that exact deterministic selection is unavailable, the run
ends `UNAVAILABLE_COMPUTE`; another slice may not be substituted.

### Token-level MQAR

Construction contains exactly 10,920 sequences of length 120.  Each has eight
distinct key/value records followed by eight later query keys at deterministically
sampled gaps.  Fillers use IDs `2..255`, keys `256..2047`, and values
`2048..4095`.  Gap distance is sampled with probability proportional to
`distance^-0.1`.  The answer loss is applied only at the eight query positions.
The queried value appears in the causal binding record; it is never repeated
after the query or injected through a side channel.

The terminal MQAR population contains 1,024 sequences and 8,192 query decisions.
Individual key/value pairs are partitioned by
`BLAKE3("1043/mqar/" || key_u16_be || value_u16_be) mod 5`: remainder zero is
terminal-only and every other remainder is construction-only.  Sequence CIDs,
complete assignment-map CIDs, and pairs have zero construction/terminal
intersection.  Key and value vocabularies are deliberately reused.

### Templated English supplied binding

The fixed key lexicon is:

`spoon, marble, kite, boat, bell, key, coin, book, ball, cup, hat, drum, doll, ring, lamp, rope`.

The fixed value lexicon is:

`garden, kitchen, attic, basket, drawer, shelf, pond, cave, barn, forest, beach, table, bed, door, tree, box, chair`.

Each lexeme, as a leading-space answer, must encode to exactly one token under
tokenizer CID
`blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc`.
The abstention answer is the single token for ` unknown` (`2823`).

Each history row contains four unambiguous key/value facts with distinct keys
and values plus one query; the answer token is a label and is not appended to
the input.  Construction has 8,190 history rows and 2,730 matched no-history
rows.  The no-history rows contain the same form of question without binding
facts and target ` unknown`.

Construction fact families are fixed to these semantic forms:

```text
Context: The {key} is in the {value}. ... Question: Where is the {query_key}? Answer:
Read these notes. We put the {key} in the {value}. ... Give the location of the {query_key}:
Placement record: {key} belongs in {value}. ... Look up {query_key}. Location:
```

Terminal fact families are disjoint:

```text
Four objects were stored. Inside the {value} was the {key}. ... Which place holds the {query_key}? Answer:
Today's list says that {key} can be found in {value}. ... The {query_key} is where?
```

The terminal population has 256 worlds, two separately scored queries per
world, 512 history decisions, and 512 matched no-history decisions.  Pair
partitioning is the English equivalent of MQAR's BLAKE3 modulo-five policy.
Remainder zero is terminal-only.  Full world-assignment CIDs and serialized
sequence CIDs are also disjoint.  The two queries are scored as independent
inputs, so one answer never becomes history for the other.

## Frozen fit

There is one optimizer trajectory and one final checkpoint:

| field | frozen value |
|---|---:|
| optimizer steps | 2,730 |
| batch | 16: 8 natural + 4 MQAR + 4 English |
| English composition per step | 3 history + 1 no-history |
| loss | `0.50 natural + 0.25 MQAR + 0.25 English` |
| loss reduction | each component reduced separately before weighting |
| optimizer | AdamW |
| betas | `0.9, 0.95` |
| epsilon | `1e-8` |
| weight decay | `0.1` |
| gradient clipping | `1.0` |
| warmup | 100 steps |
| peak learning rate | `1e-4` |
| final learning rate | `1e-5`, cosine decay |
| seed | `10043` |
| checkpoint selection | none; export once after step 2,730 |

All three construction orders are deterministic BLAKE3 orders.  Each natural
row is presented once; each MQAR and English row is presented once.  No sweep,
retry, early-stopping choice, threshold adjustment, seed change, or post-reveal
fit is permitted.

## Frozen controls and empirical criteria

All controls use the final fitted weights; none receives its own fit.

1. **Plain coherent reference:** ordinary causal attention without frame lifts.
2. **Current-only/history-off:** perform the full calculation but mask every
   source `i < t`.
3. **Value-permuted cache:** preserve keys and positions while applying a fixed
   derangement to admitted V records.
4. **Binding-permuted input:** derange values among binding keys while retaining
   the original labels and serialization lengths.
5. **Transport mismatch:** the non-homomorphic H4 transport intervention defined
   above.

The direct serialization oracle is a harness-validity check.  Before fitting it
must recover all `8,192/8,192` MQAR and `512/512` English history labels, verify
all context lengths at most 120, and report zero ambiguous or missing bindings.
Failure forbids the fit.

The final artifact passes only if every condition below passes:

| gate | frozen threshold |
|---|---:|
| MQAR exact top-1 | at least `8,110/8,192` (99%) |
| MQAR current-only drop | at least 50 percentage points |
| MQAR value-permuted drop | at least 50 percentage points |
| MQAR binding-permuted drop | at least 50 percentage points |
| MQAR transport-mismatch drop | at least 25 percentage points |
| English history exact top-1 | at least `461/512` (90%) |
| English history minus no-history | at least 35 percentage points |
| English no-history ` unknown` top-1 | at least `461/512` (90%) |
| unsupported assigned-value top-1 in no-history | exactly `0/512` |
| fresh-language NLL | no more than initialization plus `0.05` nat |
| fresh-language top-1 | no more than 1.0 percentage point below initialization |
| R4/plain attention-weight maximum delta | at most `2e-6` |
| R4/plain logit maximum delta | at most `2e-5` |
| R4/plain top-1 | identical on every scored decision |
| full/incremental logit maximum delta | at most `2e-5` |
| full/incremental top-1 | identical on every scored decision |
| future/forbidden reads | zero |
| artifact reload/replay | byte-identical artifact and identical logits |

Construction losses and accuracies are reported but never select a checkpoint.
The work ledger records token steps, cache writes, admitted scores, transported
R4 blocks, value reads, vocabulary scores, target reads, source reads, provider
calls, teacher calls, and future/forbidden reads.

## Run contract

```text
metric to move:       held-out supplied-binding exact top-1
current value:        not established; unchanged old exact-KV weights are only a
                      quarantined revealed-population diagnostic
reachability ceiling: every query's source record remains at one unique slot;
                      8,192/8,192 MQAR and 512/512 English labels are directly
                      reachable, so the structural ceiling is 100%
cheap instrument:     serialization oracle + small full/incremental/R4/plain
                      mechanics fixture; all must pass before fit
exit rule:            the complete frozen gate table above
if positive:          close #1043 as POSITION_KV_BINDING_PASS and freeze a new,
                      separate source-free context-conditioned generation rung
if synthetic only:    stop; isolate natural template/role transfer in a new issue
if not learned:       stop; revisit role encoding/curriculum in a new contract
if language regresses: stop; revisit joint preservation in a new contract
cost estimate:        benchmark CPU Accelerate at 1, 4, and 8 threads; choose the
                      fastest deterministic eligible plan; one worker owns the
                      optimizer while BLAS uses the selected cores; evaluation
                      is deterministically partitioned; projected total must be
                      <= 1,800 seconds and <= 16 GiB before launch
```

One worker is required because there is one ordered optimizer state, not because
the computation is single-core.  Apple Accelerate receives the selected thread
count.  CUDA and MPS are not candidates.  A resource projection above the hard
wall ends unavailable before fitting rather than silently changing the run.

## Frozen terminal actions

- `POSITION_KV_BINDING_PASS`: every binding, control, language, causal, parity,
  replay, identity, and work gate passes.  Authorizes only a separately frozen
  source-free context-conditioned generation rung.
- `SYNTHETIC_ONLY_NO_NATURAL_TRANSFER`: MQAR passes but English history binding
  fails.  Do not start generation or recurrence compression.
- `BINDING_LANGUAGE_REGRESSION`: binding passes but natural-language
  nonregression fails.  Do not start generation or recurrence compression.
- `POSITION_KV_BINDING_NOT_LEARNED`: MQAR misses its absolute gate.  Do not tune
  this run; a new issue must reconsider role encoding or the curriculum.
- `INVALID_POSITION_KV_BINDING`: any causal, leakage, parity, identity, replay,
  population, or work-ledger gate fails.  No scientific inference is allowed.
- `UNAVAILABLE_COMPUTE`: the exact source population or frozen compute contract
  cannot be satisfied.  No substitute population or backend is allowed.

## Frozen file boundary

Implementation may add only:

- `tools/r4-softmax-trainer/src/r4_softmax_trainer/position_kv_binding.py`
- `tools/r4-softmax-trainer/src/r4_softmax_trainer/position_kv_binding_data.py`
- `tools/r4-softmax-trainer/src/r4_softmax_trainer/position_kv_binding_campaign.py`
- the three focused matching test files;
- minimal registration in `cli.py` and `__main__.py` if required;
- this append-only record; and
- one create-once structured raw result plus the minimum current programme
  mirrors required to state its terminal truth.

Reuse `language_path_generalization.py`, `h4_spin_frame_sidecar.py`, the frozen
ordinary artifact, geometry sidecar, and tokenizer without editing them unless
the incremental API cannot be cleanly implemented as a subclass.  No V1/V5,
#1017, runtime kernel, browser, WASM, release, or unrelated test file is in
scope.  Mechanical defects may be corrected before the fit starts.  Once the
terminal-population commit exists and the optimizer starts, the architecture,
data, objective, seed, thresholds, and evaluation code are immutable.

## Pre-fit adversarial integrity amendment — 2026-09-02

An independent adversarial review was completed before a #1043 campaign root,
preparation, preflight, optimizer state, fitted artifact, or reveal existed.  It
did not identify a defect in the position-preserving K/V architecture, coherent
R4 frame realization, causal mask, or frozen mixed objective.  It did identify
evidence-path defects that could have made a later result ambiguous.  The
following corrections are therefore part of the pre-fit freeze, not adjustments
made in response to terminal measurements.

### Exact natural-language exclusion union

Preparation derives the exclusion set internally; a caller cannot supply a
different list through the CLI.  Every component is bound by its immutable
index or population file CID, record boundaries, count, and story-set CID:

| component | stories | story-set CID |
|---|---:|---|
| inherited ordinary train slice | 25,879 | `blake3:a20574441f3aa7bd29609c51502cf3325ae03d05dd21b6e8e46fa4ea7cf8878c` |
| inherited ordinary development slice | 1,251 | `blake3:18a2de8d3e955d190f7f19ff00b40bad783074773a45bd559b3e94922b08f509` |
| revealed V1-through-V4 prompt populations | 2,048 | `blake3:c926c19deaae20a17b05fc3c5eddc099324d9b531bbfd83ac992a5ef02ede092` |
| revealed V5 prompt population | 512 | `blake3:e78a4ee75b470ee946f634ef4da2edeacac2dc7b70e97c9f30610a05e1aad4e0` |
| revealed V5 fresh-language population | 1,242 | `blake3:07a9f3c199172d491738a2cda018a605b1e30ecb152103f2f17f3d2d7919f4dc` |

The five sets must be pairwise disjoint.  Their complete 30,932-story union is
fixed as
`blake3:3456b61b4e16bb7bc150c110d5eb077760e7aab5d9bf91abba47b0d097290e22`.
The #1041 prompts are handcrafted and contribute no source-story CID; the
mixed natural replay is a subset of the already included ordinary train slice.
Preparation rejects any union whose count or CID differs.

### Harness and lifecycle corrections

- The serialization oracle reconstructs MQAR records from their physical
  key/value slots and parses English facts and queries from text decoded from
  `input_ids`.  Binding metadata, stored text, query metadata, and answer
  metadata cannot rescue a corrupted serialization.
- Terminal access requires the exact campaign artifact path and size plus a
  canonical preparation -> passing preflight -> run start -> completed
  2,730-step fit chain.  Every envelope must bind the same implementation,
  data-manifest, run-contract, artifact, and work identities.  Arbitrary,
  foreign, or incomplete files cannot authorize reveal.
- CID commitment and mode-000 access are procedural controls, not cryptographic
  secrecy or remote attestation.  The terminal generator is deterministic and
  publicly reconstructible.  The claim is preregistration plus zero fit-process
  terminal reads before the final artifact, not a secret blind test.
- The 512 no-history English rows contain 30 unique serialized inputs and 482
  repeats, fixed by
  `blake3:3ffb5c122c60b3b03ca5cd773dc82332bd8a2bf8aa3da0b0b5cabf8e6359f`.
  They are an abstention-control distribution, not 512 independent inputs.
- Production preflight and scoring runners are internal and non-injectable.
  Tests patch private functions; injected mappings cannot become evidence.
- The implementation identity binds every executable trainer module and both
  dependency lock inputs, not only the three new files.
- Preparation snapshots that identity at process import, requires an exact
  match before creating the campaign root, and re-hashes it immediately before
  publishing the preparation envelope.  A source or lockfile change during
  materialization leaves no valid preparation and cannot bind already imported
  code to newer filesystem bytes.
- Scoring restores the selected Apple Accelerate CPU plan.  The 1,800-second
  total includes terminal reveal/loading, all scoring, parity, replay, and
  result creation.  The projection counts both replay forwards: 676 R4 full
  batches, 518 plain full batches, and 258 incremental batches.
- Construction top-1 counts are accumulated from the existing fit forwards.
  Terminal full/incremental parity compares fitted coherent R4 full execution
  with fitted coherent R4 incremental execution.
- The metric parser requires every mandatory field, finite floats, exact
  nonnegative integer counters excluding booleans, exact decision counts, and
  a complete recomputed work ledger before any scientific branch is selected.

### Corrected pre-fit attribution actions

The English binding-permuted control must reduce history top-1 by at least 35
percentage points.  A failed MQAR absolute gate remains
`POSITION_KV_BINDING_NOT_LEARNED`.  A passed absolute gate without the required
current-only, value-permuted, or binding-permuted drops becomes
`POSITION_KV_BINDING_UNATTRIBUTED`.  A natural English binding or abstention
failure remains `SYNTHETIC_ONLY_NO_NATURAL_TRANSFER`.

Transport mismatch is an attribution control, not a test of whether exact K/V
binding was learned.  If every binding, English, preservation, parity, causal,
replay, and work gate passes but the mismatch drop is below 25 percentage
points, the result is
`POSITION_KV_BINDING_GEOMETRY_UNATTRIBUTED`: position-preserving binding worked,
but this run did not establish that coherent R4 transport was causally
load-bearing.  Only a result passing that attribution gate may receive
`POSITION_KV_BINDING_PASS` and authorize the separately frozen generation rung.

The permitted test boundary includes the three matching model/data/campaign
files plus one focused CLI lifecycle-registration test.  No production model,
population size, optimizer setting, seed, fit count, or language threshold was
changed by this amendment.

## Frozen preparation and preflight — 2026-09-02

The implementation was committed and pushed at `1d4ff2a7` before the production
campaign root was created.  The environment was synchronized once from the
committed `uv.lock`; no trainer module, dependency lock input, or environment
was changed between preparation and preflight.

- production root:
  `/Users/casey.allard/uor-r4/.uor-models/research/issue-1043-position-kv-binding`
- preparation CID:
  `blake3:667684442ecdfa9f8bedc4ed82cc0bf13d8603130b95fcc77069fe75801ecc6f`
- implementation tree CID:
  `blake3:b8388e64b93fd0d88a5b138013a68cb1e54ec3cac33cb26576395c7fd2188e74`
- data-manifest CID:
  `blake3:8b5ace868fa9c81ea2f7ab8066cb29a04f14bbda19b9475b2974a8c8b7475f0d`
- preflight CID:
  `blake3:d96fdfba040d6769754e376f4ce4b4af740a996b305ae06952afcaa7eff7d551`
- selected CPU plan: `cpu-accelerate-4t`, one process, four Apple Accelerate
  threads, plan CID
  `blake3:4b7678db9954d18a4bf3728504fd80bcf380d5b9b52d120dc3008216bb4ce548`

Preflight passed.  The direct serialization oracle recovered all 8,192 MQAR
decisions and all 512 English decisions with zero ambiguous, missing, or
overlength records.  The measured coherent-R4 versus plain maximum attention
weight delta was `5.364418029785156e-07`, the maximum logit delta was
`1.5735626220703125e-05`, and top-1 was identical.  The measured coherent-R4
full versus incremental maximum logit delta was `1.2874603271484375e-05`, with
identical top-1.  Every future/forbidden/provider/teacher read counter was zero.

The measured total projections, including fit, reveal/load, all terminal arms,
parity, replay, and result creation, were `764.1552486672299` seconds at one
thread, `478.2424625585554` seconds at four threads, and
`641.1463332443964` seconds at eight threads.  Four threads was therefore the
frozen fastest eligible plan.  Preflight recorded `terminal_payload_reads = 0`;
no run-start, optimizer, fitted artifact, reveal, scoring, or terminal-result
artifact existed at this boundary.

## One permitted fit and terminal result — 2026-09-02

The sole production fit, reveal, and scoring attempt completed under the frozen
four-thread Apple Accelerate plan.  A fresh-process verifier then reproduced
the complete create-once evidence chain and terminal result exactly.

| evidence | value |
|---|---|
| completed optimizer steps | `2,730 / 2,730` |
| fit elapsed | `168.61726004094817` seconds |
| scoring elapsed | `193.01016108389013` seconds |
| total elapsed | `361.6274211248383` seconds |
| final artifact | `1,010,800` bytes |
| final artifact CID | `blake3:e34c211687076662740bced155b58352d325c79c0543d0f9948b9412b120527c` |
| fit CID | `blake3:14fc7011738eb11076eacce76fbfae071d75c7ea51556bd03e86dea73ec550ff` |
| reveal CID | `blake3:bf946e306fe765ed5e5017265dce817012bcc0127bccc0344637762cfdf40d9b` |
| scoring CID | `blake3:a0a98969f03bad0a13b2fb65984890c41d810f2c1249f3263e419faa23b56488` |
| result CID | `blake3:96a382d2f6118fbd2883fbee8b383764de0d6098efbf82bf863412c6996d80d0` |
| terminal | `INVALID_POSITION_KV_BINDING` |

Exact work, causal isolation, artifact replay, and coherent-R4
full/incremental parity passed.  Provider, teacher, future, and forbidden reads
were all zero.  Artifact bytes and replay logits were identical.  The sole
failed mechanics predicate was the coherent-R4/plain maximum logit delta:

| parity predicate | observed | frozen limit | result |
|---|---:|---:|---|
| R4/plain attention-weight maximum delta | `1.6689300537109375e-06` | `2e-6` | pass |
| R4/plain logit maximum delta | `2.193450927734375e-05` | `2e-5` | **fail** |
| R4/plain top-1 | identical on `257,136` decisions | identical | pass |
| R4 full/incremental logit maximum delta | `1.239776611328125e-05` | `2e-5` | pass |
| R4 full/incremental top-1 | identical | identical | pass |

The frozen decision order makes any failed mechanics predicate terminally
invalid before the scientific branches are interpreted.  The historical
result remains invalid; it is not retroactively widened or reclassified.

### Numerical root-cause audit

The coherent R4 equations and indices are correct.  The sidecar's maximum f32
orthogonality residual is `9.1763788e-09` (`2.5145e-17` when evaluated from the
f64 source).  The plain path performs one 12-lane f32 contraction.  The R4 path
performs frame composition, separate query/key lifts, transported block
contractions, value transport, and reconstruction.  Those algebraically equal
operations have a different f32 association order.  Their small rounding
difference passes the attention criterion and is then amplified by the second
normalization/residual block and the 4,096-way tied output projection.

The terminal maximum is exactly `23 * 2^-20`; the frozen limit lies at about
`20.97 * 2^-20`.  Targeted fitted-model diagnostics found no incorrect
transport: directly lifting a key into the current frame produced the same
`3.8146973e-06` score discrepancy as the explicit source transport.  A naive
f64 internal variant did not restore parity against the differently rounded
plain-f32 reference.  This is expected forward-error amplification, not a
causal, cache, masking, frame, or transport defect.

Changing the observed threshold would be post-reveal tuning.  Changing the
contraction implementation would change the frozen implementation identity.
Either action is forbidden for this campaign, so the production root, single
reveal, artifact, scoring envelope, and invalid result remain immutable.

### Decision-value audit and bounded construction diagnostic

A separate scoring-only parity repair would not change the project action.
Without modifying the artifact, a read-only score of the already-open
construction populations produced:

| construction population | final top-1 | final NLL |
|---|---:|---:|
| MQAR | `30 / 87,360` (`0.03434%`) | `7.907718394352839` |
| English with supplied history | `578 / 8,190` (`7.05739%`) | `3.364303203671261` |
| English no-history abstention | `2,730 / 2,730` (`100%`) | `0.03913288011655703` |

These construction-only observations are a design diagnostic, not a
reinterpretation of the invalid sealed terminal metrics.  They show that the
frozen optimization recipe learned the easy no-history abstention and
preserved/improved ordinary language, but did not fit associative retrieval on
its own training population.  A parity-only rescore cannot change those
weights and therefore has no decision value.  No recovery run, second reveal,
optimizer, generation, recurrence compression, lowering, correctness, or
reasoning work is authorized from #1043.

## Direction after #1043

The result does not invalidate ordinary causal softmax attention, its coherent
R4 gauge realization, or the qualified retained-language baseline.  It rejects
this exact one-epoch mixed optimization recipe as a route to supplied-context
binding and leaves #954 blocked.

The next research contract should be a new issue, not an adjustment to #1043.
Before sealing another terminal population it should require a cheap,
construction-only capability ladder:

1. make causal KEY, VALUE, QUERY, and abstention roles explicit in the input
   representation while preserving one exact K/V slot per position;
2. establish near-exact MQAR fitting and assignment-disjoint development
   transfer under an associative-first curriculum;
3. add natural-English binding and then language replay only after the prior
   rung passes, measuring gradient conflict instead of forcing all objectives
   through one clipped mixed update from step one;
4. keep one fitted weight set and ordinary stable-softmax attention, then
   execute its R4 realization with a predeclared ULP/forward-error parity
   contract derived before fitted-scale evaluation; and
5. freeze a new unseen terminal population only after the open development
   mechanism works.

That sequence diagnoses representation and optimization before another sealed
campaign.  It does not add recurrence, resonance replacement, generation, or
more corpus volume until position-preserving binding itself works.
