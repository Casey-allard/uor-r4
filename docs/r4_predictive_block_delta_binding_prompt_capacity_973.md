# #973 predictive R4 block-delta binding and prompt capacity

- **Freeze date:** 2026-09-01
- **Issue:** #973
- **Programme root:** #820
- **Mechanism:** `R4PredictiveBlockDeltaBindingV1`
- **Campaign:** `R4PredictiveBlockDeltaPromptCapacityV5`
- **Evidence status:** `V5_VERIFIED_TERMINAL`
- **Current result:** `PREDICTIVE_BINDING_NO_TERMINAL_CAPACITY`
- **Next action:** `STOP_WITHOUT_GENERATION`
- **Generation:** `NOT_RUN`

## Result first

The terminal V5 experiment changed the retained **write and binding law**. It
did not add another readout over the failed 120-slot value field. The run is
complete and independently verified, but the geometric arm's prompt gain was
`0.03896945868086732`, below the frozen `0.04332169878499658` capacity floor.
The terminal verdict is therefore
`PREDICTIVE_BINDING_NO_TERMINAL_CAPACITY`, and the predeclared action is
`STOP_WITHOUT_GENERATION`.

The completed learned-associative experiment showed that the qualified V1
field can help language modeling when pooled, but does not place enough
prompt-specific information at the exact candidate address. The geometric
head gained only `0.0063767854` nats/direction and the pooled head gained
`0.0102632346`, below the frozen `0.0433216988` absolute and `0.0253415693`
incremental capacity floors. A fixed candidate-leaf read also has a structural
reachability problem: 4,096 tokens collapse onto 35 directly used H4 leaves,
while V1 writes only the identity slot after transport.

A read-only census of the immutable V1 schedule makes that defect concrete:

| Population | True target leaf occupied |
| --- | ---: |
| Construction train, all positions | `1,885,115 / 5,241,600 = 35.9645%` |
| Construction train, positions 47 through 62 | `35.9130%` |
| Validation, all positions | `89,069 / 247,920 = 35.9265%` |
| Validation, positions 47 through 62 | `35.8906%` |
| First post-prompt position, train / validation | `32.3375% / 30.8809%` |

The earlier “reachable” census meant that some state existed. It did not mean
the true candidate's leaf held a value. A fixed-leaf readout therefore had an
approximately 36% structural opportunity ceiling before value quality was
even considered. The new dense block memory makes every written association
query-readable; its reachability ceiling is 100% by construction.

This freeze replaces that address-dependent association with a bounded
key-to-value matrix memory. The key is the previous causal context; the value
is the token that is subsequently observed. The value and candidate scorer
remain anchored to their immutable token H4 leaves so the geometry cannot
cancel into a pure change of basis.

The following architecture and contract are retained as the pre-reveal record.
V5 prompt and fresh-language targets were created and opened only through the
frozen preparation, commitment, and reveal transition described below.

## Frozen architecture

The complete qualified `R4RetainedLanguagePathV1` artifact remains immutable.
It supplies the causal hidden state `h_t`, its ordinary tied output embedding,
and the existing V1 logits. Four new bias-free `48 x 48` maps `Wq`, `Wk`,
`Wv`, and `We`, plus four-bank `rho`, `eta`, and `alpha` logits, are the only
trainable values.

| Item | Frozen value |
| --- | ---: |
| Qualified V1 parameters | `252,160`, immutable |
| New trainable parameters | `4 * 48 * 48 + 3 * 4 = 9,228` |
| R4 blocks | `48 / 4 = 12` |
| Retention banks | `4` |
| Matrix state | `4 * 12 * 4 * 4 = 768` f32 values |
| Pending previous-context key | `12 * 4 = 48` f32 values |
| Total new recurrent state | `816` f32 values / `3,264` bytes |
| Additional bounded metadata | one H4 frame index and one key-valid bit |

Let `F_t` be the cumulative canonical H4 frame after observing token `x_t`,
and let `L(c)` be candidate `c`'s immutable canonical token-leaf frame. A
registered H4 frame matrix decodes local coordinates to model coordinates, so

```text
T(a -> b) = F_b^T F_a
```

maps coordinates from frame `a` into frame `b`. The compiler-side f32 table is
derived from, and content-bound to, Rust's canonical exact-H4 root registry;
no independently invented or learned coordinates are allowed.

For each bank and each of the 12 independent R4 blocks, observing `x_t`
executes this strict causal schedule:

```text
P_t       = T(F_(t-1) -> F_t)
S_bar     = P_t S_(t-1) P_t^-1
k_bar     = P_t k_(t-1)
v_t       = T(L(x_t) -> F_t) block(Wv E[x_t])
S_t       = rho * S_bar
            + eta * (v_t - (rho * S_bar) k_bar) outer k_bar
q_t       = block(F_t^T Wq N(h_t))
r_t       = sum_bank softmax(alpha)_bank * S_t q_t
e_(t,c)   = block(T(L(c) -> F_t) We E[c])
z_(t,c)   = zV1_(t,c) + dot(e_(t,c), r_t) / sqrt(48)
k_t       = block(F_t^T Wk N(h_t))
```

Every R4 key block is normalized with the frozen denominator
`max(||k||_2, 1e-6)` before the delta update. `rho=sigmoid(rho_logit)` and
`eta=sigmoid(eta_logit)`. The read and logits for `x_t`'s prediction occur
before `x_(t+1)` is observed. The update shown above binds the already stored
`k_(t-1)` to the now-observed `x_t`; it never writes an unobserved target.
At BOS, the key-valid bit is false, so the complete update work executes but
its write delta is multiplied by zero. After each prediction, `k_t` is retained
for the next observation.

This is a compiler-side floating-point learning experiment. It makes no claim
yet about integer/table lowering or the multiplication-free deployed kernel.

## Why the leaf anchors are load-bearing

Transporting every q/k/v/readout vector coherently by the same cumulative
frame and then back-transporting the result would make exact H4 and identity
transport algebraically equivalent. Such a test could never attribute an
effect to geometry.

Here the observed value is anchored at `L(x_t)` and every candidate scorer at
its own `L(c)` before both are expressed in the current route frame. The plain
arm removes all of those transports while retaining identical parameter,
state, contraction, and optimizer budgets. The fixed transport-permuted
intervention breaks the canonical connection while preserving the same
matrices and work.

## Frozen arms and interventions

1. **Geometric:** the complete law above with the canonical H4 frame table.
2. **Plain:** an independently fitted equal-parameter arm in which every
   `P_t`, `F_t`, and `L(c)` matrix is identity; all matrix contractions still
   execute.
3. **Transport-permuted:** evaluate the fitted geometric arm with the frozen
   identity-fixing, non-homomorphic H4 connection permutation while preserving
   canonical token value and candidate anchors.
4. **No-delta-overwrite:** evaluate the fitted geometric arm with the matched
   additive/Hebbian update `rho * S_bar + eta * v outer k`. Both `v` and the
   full residual `v - (rho * S_bar) k` are computed before the frozen selector,
   so this removes only key-specific replacement while preserving a live write,
   transported decay, parameters, and work.
5. **State-off:** execute the full geometric arm and multiply only its final
   logit addition by zero. This must reproduce qualified V1 logits exactly.
6. **Order-shuffled:** rebuild state under one pre-bound within-sequence order
   permutation for a causal order-sensitivity audit only.
7. **External comparators:** immutable qualified V1 and the completed frozen
   pooled learned-readout artifact are rescored, never refitted.

The geometric and plain trainable tensors are byte-identical at
initialization, see the same construction windows in the same order, and use
the same optimizer schedule. CUDA is forbidden.

## Data boundary

The new heads may train on the already bound #989/#953 construction view:
`43,680` windows, context `120`, batch `16`, exactly `2,730` optimizer steps
and `5,241,600` token presentations in the existing deterministic order. The
qualified V1 artifact and all of its parameters remain frozen.

Revealed V4 data may be used only by the disposable cheap gate below. It is
not terminal evidence and no weights from that gate may enter the final fit.

If and only if the cheap gate passes, preparation creates and jointly seals:

- a V5 set of `256` matched prompt pairs / `512` directions / `8,192` scored
  tokens, selected strictly after V4 source ordinal `409,546` and excluding
  the exact `2,048`-story CID union of V1 through V4; and
- a story-disjoint fresh-language slice of `249,986` tokens / `2,066` windows
  from token offsets `[156,282,226, 156,532,212)`, beginning at capacity story
  `765,248` / source story `849,803` and ending at capacity story `766,489` /
  source story `851,190`, with the complete `1,242`-story CID set bound.

Both populations are selected by frozen coordinates and content identities,
stored mode `000`, and committed before either fitted artifact CID is known.

## Cheap hard gate before V5 creation

The implementation first has to pass focused mechanics checks:

- canonical frame-table identity, orthogonality, closure binding, and
  independent reload;
- read-before-write causality and a counterfactual unobserved-target mutation;
- nonzero finite gradients for every one of the `9,228` trainable values;
- deterministic replay and equal geometric/plain work ledgers;
- exact state-off reproduction of V1 logits;
- nontrivial transport equivariance and a destructive-permutation effect; and
- an observability check that two prior contexts followed by different
  observed tokens create different, query-readable matrix bindings.

It then performs one disposable overfit on the first 32 already revealed V4
pairs (`64` directions / `1,024` targets), for at most `256` updates. Admission
requires all of:

```text
own-minus-foreign prompt gain >= ln(2) / 16 = 0.0433216988
own-prompt wins              >= 52 / 64
gain lost without delta overwrite >= ln(1.5) / 16 = 0.0253415693
gain lost under state-off     >= ln(1.5) / 16 = 0.0253415693
```

The weights are destroyed after scoring. A miss terminates this version as
`PREDICTIVE_BINDING_NOT_OBSERVABLE`; V5 is not created, no full fit starts,
and the next action is to inspect the key/value placement or causal schedule.
A pass freezes the implementation CID and authorizes exactly one V5 campaign.
Thresholds and code do not change within the version.

## Terminal decision contract

The terminal geometric arm has prompt-conditioning capacity only if all of
these hold on unopened V5:

```text
own-minus-foreign prompt gain >= 0.0433216988
gain over qualified V1        >= 0.0253415693
gain over frozen pooled head  >= 0.0253415693
own-prompt wins               >= 308 / 512
own-prompt NLL                <= both comparator own-prompt NLLs
```

Fresh-language retention additionally requires NLL no more than `0.05` above
the better immutable comparator and top-1 no more than `1.0` percentage point
below it. State must be load-bearing, all causal/mechanics/work/replay checks
must pass, and post-reveal optimizer steps must equal zero.

Geometry is attributed separately only if the geometric arm beats both the
independently fitted plain arm and the transport-permuted intervention by at
least `0.0253415693` gain, at least `308/512` paired improvements, and no worse
own-prompt NLL. Key-specific delta overwrite is attributed under the same
rules against the live additive no-delta-overwrite intervention.

- **Capacity positive, geometry positive:** preserve this cell and freeze one
  bounded autonomous generation rung before any lowering work.
- **Capacity positive, geometry negative:** preserve predictive delta memory
  as an attention-capable non-geometric control; do not claim geometric
  attention, and isolate only the leaf/connection term next.
- **Capacity negative:** reject this write/binding law without generation,
  corpus expansion, another readout, or exact-runtime lowering.
- **Any integrity failure:** terminal `INVALID`; repair the harness, not the
  model, and do not interpret scores.

## Compute contract

The focused disposable gate is budgeted at five minutes. Before the terminal
fit, a construction-only timing probe compares CPU `4`, CPU `8`, and two
independent CPU workers with `4` threads each under Apple Accelerate and
ordered deterministic reduction. The measured fastest eligible plan is used.
The expected full fit is `20-35` minutes; hard wall time is `3,600` seconds
before scoring and independent verification. A timeout is `UNAVAILABLE`, not
a scientific miss. No broad workspace suite or unrelated campaign runs in the
experiment loop.

## Claim boundary

This freeze does not establish prompt capacity, geometry attribution,
attention, coherent generation, reasoning, integer/table lowering,
multiplication-free runtime legality, product readiness, or release readiness.
It defines one falsifiable attempt to put the missing causal context-to-token
association into bounded R4 matrix state while keeping the qualified language
model and strongest prior comparator fixed.

## V1 execution result — 2026-09-01

V1 executed once on CPU with eight PyTorch intra-op and eight inter-op threads.
All `256` updates and all scoring completed in `64.6513` seconds. The exact
create-once result is
`blake3:004abd0ab27e63065c4961863123c8e086ff1b88ea12162de558a0bdaac8dac8`.
The disposable `9,228` fitted values were destroyed and no fitted artifact was
written. V5 remained uncreated and uninspected.

All mechanics passed: causal-prefix, unobserved-future, state-off, replay, and
forbidden-read deltas were zero; every trainable value received a finite
nonzero gradient; the qualified V1 artifact was unchanged; and the largest
all-frame connection/covariance error was below `1.8e-7` against the frozen
`2e-5` ceiling.

| Arm | gain, nats/token | own wins | own NLL | foreign NLL |
|---|---:|---:|---:|---:|
| Full predictive delta | `1.1097723` | `64/64` | `2.6302566` | `3.7400288` |
| State-off qualified V1 | `0.0048169` | `38/64` | `3.5984988` | `3.6033157` |
| Same fitted values, additive intervention | `1.1499870` | `38/64` | `18.4722066` | `19.6221937` |

The frozen V1 verdict is `PREDICTIVE_BINDING_NOT_OBSERVABLE` because the sole
`delta_over_additive` gate missed: full-delta gain minus additive gain was
`-0.0402148`, below `0.0253416`. That verdict is not revised. It also must not
be misreported as absence of native predictive-binding capacity: the native
arm exceeded the absolute gain floor by `25.62x`, won every direction,
improved own NLL over state-off by `0.9682423`, and was state-load-bearing.

The additive comparison exposed a decision-metric pathology. Its slightly
larger relative contrast was the difference between two catastrophically bad
likelihoods: additive own NLL was `15.8419501` worse than full delta and
`14.8737078` worse than state-off. Relative contrast alone therefore cannot
decide whether the ablation remains a usable next-token model. V1 remains a
procedural miss, while its native-capacity observation motivates the versioned
control correction below.

## V2 matched-control correction freeze — 2026-09-01

V2 does not alter `R4PredictiveBlockDeltaBindingV1`, reinterpret V1, create V5,
or inspect any V5 coordinate. It changes only the disposable control decision
and uses the non-overlapping already revealed V4 pair indices `32..63`
(`32` pairs / `64` directions / `1,024` targets). Their ordered identities are
bound before fitting by selector CID
`blake3:285be20c9c41267dbf925ea7d24d198b41a9014653ff62b1bdb64c8e2ee4fd5a`
against V4 population CID
`blake3:cc9a1c40fe753e269ea31edd804c32b2a0c208ef20fceb1167636d6f28d7da11`.

Two models start from byte-identical binding values and the same immutable V1:

1. full predictive delta, fitted with the native update; and
2. additive/Hebbian no-overwrite, fitted independently with its own live
   additive update.

Each receives exactly the same optimizer, batch order, eight directions per
batch, and at most `256` updates. Both receive complete gradient, replay,
causality, unchanged-base, and equal-work checks. Both fitted value sets are
destroyed after scoring. The complete V2 gate is CPU8-only, forbids CUDA, and
has one `300`-second wall.

Native capacity is admitted only if all integrity checks pass and:

```text
full gain                         >= ln(2) / 16 = 0.0433216988
full own-vs-cross wins            >= 52 / 64
full gain - state-off gain        >= ln(1.5) / 16 = 0.0253415693
full own NLL                      <= state-off own NLL + 0.05
```

V5 authorization depends only on that native-capacity decision. Delta
attribution is reported separately. The independently fitted additive arm is
language-valid only if its own NLL is no more than `0.05` above state-off. If
valid, full delta has prompt-specific superiority only when its own NLL is no
worse and its gain exceeds additive gain by at least `0.0253415693`. If the
additive arm is not language-valid, V2 records
`ADDITIVE_CONTROL_NO_STABLE_CAPACITY`: delta overwrite is load-bearing for
stability on the disposable slice, but prompt-specific delta superiority
remains unclaimed.

- **Native capacity miss:** reject this unchanged binding mechanism and keep
  V5 closed.
- **Native capacity pass:** authorize exactly one frozen V5 terminal campaign;
  carry the additive verdict as a separate attribution result.
- **Integrity failure or timeout:** `INVALID`/`UNAVAILABLE`; repair or report
  the harness and do not interpret model scores.

This V2 correction is frozen before implementation and before opening its
disjoint V4 slice. It is still only a revealed-data expressivity gate. Even a
pass would not establish held-out prompt capacity, geometry attribution,
general attention, autonomous generation, or reasoning.

## V2 execution result — 2026-09-01

V2 executed once on CPU with eight PyTorch intra-op and eight inter-op threads.
Both independently initialized arms completed exactly `256` updates; fitting,
scoring, mechanics, destruction, and create-once result writing completed in
`123.8098` seconds. The exact result is
`blake3:623bbd63321c18ad7e4172b325b2d22518b6b10a33f755d3bbbbdcf9b9c51637`,
bound to implementation-tree CID
`blake3:603791e7b74a682855507797a6ab3533e36f963914fc2d307549e97e87bde366`.
All `9,228` disposable fitted values in each arm were destroyed and no fitted
artifact was written. V5 remained uncreated and uninspected during V2.

All mechanics and integrity gates passed. Both arms received finite nonzero
gradients for every trainable value, used the same initial binding CID and
batch-schedule CID, left the qualified base byte-identical, and replayed
exactly. Causal-prefix, unobserved-target, state-off, and forbidden-read deltas
were zero. The transport-permutation head effect was `30.3544`; the largest
transported covariance error was below `1.8e-7`.

| Arm | gain, nats/token | own wins | own NLL | foreign NLL |
|---|---:|---:|---:|---:|
| Independently fitted full delta | `1.1600515` | `64/64` | `2.8868984` | `4.0469499` |
| Independently fitted additive | `1.5442248` | `64/64` | `2.9562780` | `4.5005028` |
| State-off qualified V1 | `0.0079665` | `37/64` | `3.8638364` | `3.8718029` |

The native arm exceeded state-off gain by `1.1520850` nats/token, improved
state-off own NLL by `0.9769380`, and passed every frozen native-capacity gate.
The terminal V2 verdict is therefore
`PREDICTIVE_BINDING_NATIVE_CAPACITY_ADMIT`, and exactly one frozen V5 campaign
is authorized.

The additive arm was also language-valid. Full delta had `0.0693796` lower own
NLL, but its relative prompt gain was `0.3841733` lower, so the separately
frozen attribution verdict is
`DELTA_PROMPT_SPECIFIC_SUPERIORITY_NOT_ESTABLISHED`. This does not revoke the
native-capacity admission. It means V5 must distinguish held-out predictive
capacity from geometry and overwrite attribution instead of treating a
revealed-slice overfit as attention evidence.

This is positive evidence for bounded causal associative memory. It is not yet
held-out prompt capacity, geometric attribution, general attention,
autonomous generation, reasoning, or deployed integer/table runtime evidence.

## V5 terminal executable correction — 2026-09-01

This correction is frozen after the V2 admission and before any V5 population,
fresh-language slice, terminal optimizer, or terminal fitted artifact is
created or opened. It does not change the mechanism, the V5 coordinates, or
the terminal capacity and geometry thresholds above. It makes the already
declared terminal campaign executable and repairs the overwrite comparator in
light of V2's independently fitted additive result.

### Exact training law

The geometric, plain, and additive arms start from byte-identical predictive
binding values initialized with seed `9,739` over the same immutable qualified
V1 artifact. Each arm independently minimizes ordinary next-token cross
entropy over all `120` decisions in each of the same `43,680` construction
windows, in the existing deterministic order. Each receives exactly `2,730`
updates at batch size `16` with AdamW: warmup `100`, peak learning rate
`3e-4`, cosine decay to `3e-5`, betas `(0.9, 0.95)`, epsilon `1e-8`, weight
decay `0.1`, and gradient clipping at `1.0`. No terminal arm may reuse either
V2's disposable values or any post-reveal gradient.

The three independently fitted terminal arms are:

1. canonical H4 transport with the full delta update;
2. identity transport and identity token/candidate anchors with the full delta
   update; and
3. canonical H4 transport with the additive/Hebbian no-overwrite update.

The fitted geometric arm is additionally rescored under transport permutation,
state-off, order-shuffled state construction, and the same-fitted-value
additive intervention. The last is auxiliary ablation evidence only. Because
V2 showed that the additive law can fit a stable memory, overwrite attribution
is decided against arm 3's independently fitted equal-budget result, not
against weights optimized for another update law.

### Exact unopened populations

V5 selection is the V4 matching policy continued strictly after source story
ordinal `409,546`, with all stories from these exact prior population CIDs
excluded:

```text
V1 blake3:c11a7c935139ca169460b90c01392d7c9e0929e4c10710e76e6c8f74cbdf0340
V2 blake3:258f143eedbbb7067dc512db929a42166ad8a492fc059542409f419a3b46942e
V3 blake3:165be397b73041afd39aa65ae796400ea539399f8586729ad19a168c4daa9e93
V4 blake3:cc9a1c40fe753e269ea31edd804c32b2a0c208ef20fceb1167636d6f28d7da11
```

The canonical sorted union contains exactly `2,048` story CIDs and has CID
`blake3:c926c19deaae20a17b05fc3c5eddc099324d9b531bbfd83ac992a5ef02ede092`.
The selector stops at exactly `256` pairs. Its content-derived population and
commitment CIDs are published after create-once preparation and before any fit.

The first executable-freeze draft listed union CID `blake3:494e5503...` from
a prior-only calculation that omitted the canonical serializer's mandatory
final LF byte. Before V5 creation, two independent
read-only reproductions over the four exact population files found `512`
unique story CIDs in each, zero pairwise overlap, and the `2,048`-entry
canonical sorted-list CID above. The code and public contract were corrected;
the erroneous CID was never used to select or seal V5.

The fresh-language slice remains exactly token offsets
`[156,282,226, 156,532,212)` from the source token stream: `249,986` tokens,
`2,066` windows, capacity ordinals `765,248..766,489`, source ordinals
`849,803..851,190`, and exactly `1,242` story CIDs. Preparation verifies the
whole source train-index CID
`blake3:0032889e32b38801476223c5bed7e401d77b61afbbd6cf9afddaceee18e2136e`.
Its slice and story-set CIDs are likewise content-derived outputs published in
the joint create-once commitment before fitting. Both unopened populations are
regular files stored mode `000`; training reads construction data only.

### Derived state and order checks

State-off must reproduce immutable V1 logits and scores exactly. The existing
capacity requirements against V1 already make state load-bearing; the result
also records and requires the redundant derived check
`geometric gain - state-off gain >= 0.0253415693` with no worse geometric own
NLL. No new scientific threshold is added.

Order shuffle remains an audit, not an outcome gate. A target-blind
within-sequence permutation is content-bound before reveal; it must preserve
the token multiset, execute equal work with zero forbidden reads, replay
deterministically, and publish its score and head-trace effect. No score-drop
direction or magnitude is required after seeing the population.

Geometry attribution keeps the frozen comparison against the independently
fitted plain arm and transport-permuted intervention. Delta-overwrite
attribution uses the same frozen gain, paired-improvement, and own-NLL rules
against the independently fitted additive arm. Capacity remains independently
decidable even if either attribution fails.

The independently fitted additive arm is eligible for that attribution only
if its own-prompt NLL is no more than `0.05` above exact state-off/V1
own-prompt NLL. If it is not language-valid, the terminal records
`ADDITIVE_CONTROL_NO_STABLE_CAPACITY` and leaves delta-overwrite superiority
unclaimed; it does not turn a broken comparator into positive attribution.
Capacity and geometry decisions remain independently decidable.

Before selecting or opening V5, preparation must verify exact V2 result CID
`blake3:623bbd63321c18ad7e4172b325b2d22518b6b10a33f755d3bbbbdcf9b9c51637`
and its `production_v5` boundary: `authorized=true`, `created=false`, and
`inspected=false`. The create-once V5 preparation is the sole transition that
may change the latter two facts for the terminal campaign.

The timing probe still chooses among CPU4 sequential, CPU8 sequential, and two
CPU4 workers with ordered deterministic collection under Apple Accelerate.
With three fitted arms, the expected fit window is revised to `30-50` minutes;
the predeclared hard fit wall remains `3,600` seconds before scoring and
independent verification. CUDA remains forbidden.

## V5 terminal execution result — 2026-09-01

The sole frozen V5 campaign completed all three independent fits at exactly
`2,730` optimizer steps per arm. The terminal result is
`PREDICTIVE_BINDING_NO_TERMINAL_CAPACITY`, with predeclared next action
`STOP_WITHOUT_GENERATION`.

The geometric arm improved prompt-relative behavior substantially over the
immutable controls, but missed the one gate that decides terminal capacity:

| Prompt arm | gain, nats/token | own wins | own NLL |
|---|---:|---:|---:|
| Geometric delta | `0.03896945868086732` | `375/512` | `3.5419674206289073` |
| Qualified V1 / state-off | `0.005190052751459007` | `295/512` | `3.6142577020455064` |
| Frozen pooled head | `0.009168421948743344` | `314/512` | `3.580036127979838` |
| Independently fitted plain delta | `0.015039646930972594` | `309/512` | `3.518444197495228` |
| Independently fitted additive | `0.04548192190964073` | `368/512` | `3.5523845836341934` |
| Transport-permuted geometric | `0.007159131815569708` | `269/512` | `3.646037837855147` |

The geometric arm beat V1 gain `0.005190052751459007` and pooled gain
`0.009168421948743344`, cleared the `308/512` directional-win floor, had no
worse own NLL than either immutable comparator, and passed the state
load-bearing checks. Its absolute gain nevertheless missed the frozen
`0.04332169878499658` floor by `0.00435224010412926`. Capacity is therefore
negative even though several component metrics improved.

Geometry attribution is also not established. Geometric gain exceeded the
independently fitted plain arm by `0.023929811749894725`, just below the frozen
`0.025341569256760274` floor, and geometric own NLL
`3.5419674206289073` was worse than plain `3.518444197495228`. The destructive
transport-permuted comparison did separate by `0.03181032686529761` and met
its paired and own-NLL gates, but both geometry comparators were required.

Delta-overwrite attribution is not established either. The independently
fitted additive arm was language-valid; geometric delta trailed it by
`-0.006512463228773413` gain and improved only `234/512` paired directions.
This result retires this V5 predictive write/binding law. It does not retire
the broader attention programme, and it does not imply that ordinary causal
softmax attention failed.

Fresh-language retention passed. Across all `247,920` decisions, geometric CE
and top-1 were `3.84055165318221` and `0.30979348176831234`, compared with the
better immutable pooled comparator's `3.85444653890486` and
`0.3014924169086802`. All integrity gates passed: forbidden reads were zero,
post-reveal optimizer steps were zero, fitted artifacts and scores replayed
exactly, the immutable base remained unchanged, and the independent verifier
reproduced the evidence exactly.

### Frozen execution identities

The construction-only probe selected two concurrent CPU workers with four
threads each under Apple Accelerate; CUDA remained forbidden. The complete fit
consumed `2849.632959582843` seconds, below the frozen `3,600`-second wall.

| Boundary | Content identity |
|---|---|
| Preparation | `blake3:1e65392c729ca349b2a9a61f4bfb503e5cb32392f42f69d7f4b836ea7692d10a` |
| Commitment | `blake3:8e9c02068bb1dfef956907b1b614ddb0c4fcf902262fc934f8b098f5fd7cf0c4` |
| Population | `blake3:120719d0984b33a63904b5d72cc8b5e831b77df2eceb2f2c75b9c75750cacd10` |
| Probe / selected plan | `blake3:7adc13f30955b8843674d5a9b410500046fdd5376422979ff8f69f547c32aa08` / `blake3:639d59ad78299f6bf87919506fdba080b81ed0ed315c7dbc5185fc346e166d48` |
| Started / fit budget | `blake3:c4c1dacb4e99a955c1d4777064cda0191aeecab7543e8e76a23a5b01d5c758a6` / `blake3:cb5a1f1640ea08882542423721719c31b0044ee679ec06ba51a589f3c400ea3d` |
| Geometric arm / artifact | `blake3:c8b62dba59c23a93d04aa60cebfffe5c366bb4ce6b8ee48b64088bef4db77b60` / `blake3:8e7c153a9270ce533ffd195ab6e879fd26278a8be6db46299b4c0aa0033bf0a0` |
| Plain arm / artifact | `blake3:0c91f859e2d05e77dc81e8f17ae5c40e72d23d7bdb7c64fd1f03a7e727cbfc87` / `blake3:2b304e97accad753931715ec02078a4eabcf81d03b218e62600ea98008b0ba12` |
| Additive arm / artifact | `blake3:e32101f0ff89e3b4099e9f23645873e2fb80d22478cce9f5bb52a8ec3debe155` / `blake3:3f70f66e619e7ed00cbbde28f55d97cfe50ed9d29652a35012cc190af02fc77b` |
| Reveal | `blake3:6773e5ec1be496a5d1edae29f810d3b13a05b3953757b31ea22f909471ae5800` |
| Fit / scoring implementation trees | `blake3:000a3ae8a69ba9185ff66ee58ff891b3eb22ab857195d71d38441e277cceca24` / `blake3:34ef52bae5ed4401e382e1886f2a136fc797f1c9db69bc9fc50fde4c4cd41945` |

### Scoring-only recovery provenance

The first scoring process stopped before producing scientific evidence when
its work-ledger check incorrectly required the final two-row tail batch to
have the same raw counters as each full sixteen-row batch. The original
unavailable record remains preserved at
`blake3:a819ed7f2b558d80053362c6c229642835b1317ff367d576aeb6ab23a592536a`
with scientific status `NOT_RUN`.

The bounded repair checked exact per-row proportionality and aggregated work
over all `2,066` windows. Recovery CID
`blake3:7b76e36e44798bebf184ece08fdd8a2065bdd370106b5d64d5fae4c59dc6d88b`
bound the already frozen fitted artifacts and authorized scoring only. It
created no optimizer, performed no retraining, and executed zero post-reveal
optimizer steps.

The verified evidence chain is:

```text
scoring CID      blake3:44f8941d24a99fc230710fd700e7a7b13cee87587bfbe4e13bf7b095222e2ee6
result CID       blake3:6c67544d675eafcb8eb9c0dabb93617e3f6c3295af812e8acbb687107c010a74
verification CID blake3:567cf336eb05c3ec562aef7135f6fb35b580d02c758b0e79f2508cae57065f5d
exact replay     true
```

Coherent generation, reasoning, and integer/table lowering remain `NOT_RUN`.
Release readiness is not established. Per the frozen decision contract, no
generation rung, corpus expansion, readout retry, or lowering follows from
this V5 result.
