# #973 conversation entity SpinTorsion-path attention record

- **Date:** 2026-08-28
- **Issue:** #973
- **Contract:** [frozen on the live issue](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5455774679)
  before implementation, outcome-bearing geometry, or decoded execution
- **Pre-implementation correction:** [global equality tightened on the live issue](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5455800872)
  before implementation; no fixture, mechanism, target, or expected matrix changed
- **Frozen constants addendum:** [scope identities, bounds, and work fixed on the live issue](https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5455815176)
  before implementation or outcome-bearing execution
- **Mechanism:** `ConversationEntitySpinPathR4V1`
- **Contract status:** frozen
- **Outcome:** `RETAIN_CONVERSATION_ENTITY_SPIN_PATH_ATTENTION_CONTINUE_BOUNDED_GLOBAL`
- **Issue state:** #973 remains open; #954 remains blocked

## Decision

Can one construction-bound path over exact stored lexical `SpinTorsionState`
make an earlier completed-turn entity binding causally change an already
admitted #953 candidate and decoded output when the immediately preceding turn,
the entire current turn, all local-through-paragraph evidence, candidate
support, and declared work remain fixed?

This is the independently frozen conversation-scope contrast named by the
positive `ParagraphEntitySpinPathR4V1` result. It owns no candidate admission,
does not modify the source-free table or #953 overlay, and does not activate
bounded-global or corpus work.

## Frozen D3 split and exact bytes

Construction uses only D3-construction IDs (`d3_is_held_out == false`):

```text
27:
Nora carried the spiral marker.

Nora opened the registry. Owen waited.

The active registry code is silver.

28:
Owen carried the faceted marker.

Owen opened the registry. Nora waited.

The active registry code is violet.
```

Target-free evaluation uses only D3-held-out IDs
(`d3_is_held_out == true`):

```text
45, completed binding turn:
Mara carried the spiral marker. Iris carried the faceted marker.

45, final completed turn:
Mara opened the registry. Iris waited.

45, active turn:
The active registry code is

48, completed binding turn:
Mara carried the faceted marker. Iris carried the spiral marker.

48, final completed turn:
Mara opened the registry. Iris waited.

48, active turn:
The active registry code is
```

IDs and text CIDs must be disjoint. Construction entities `{Nora,Owen}` and
held-out entities `{Mara,Iris}` are disjoint. Both held-out conversations have
the same lexical multiset, turn count, turn IDs, boundaries, final completed
turn, and active turn. They differ only in the earlier entity-to-descriptor
binding. Exact admitted candidates ` silver` and ` violet` must occur zero
times across either observed held-out conversation.

The prediction API receives only the two structured completed-turn byte slices
and active-query bytes. It receives no held-out partition ID, target, expected
candidate, future unit, full-history lookup key, teacher output, provider text,
or source metadata.

The accepted grammar is exactly:

```text
binding turn: <ENTITY> carried the <DESCRIPTOR> marker. <ENTITY> carried the <DESCRIPTOR> marker.
focus turn:   <ENTITY> opened the registry. <ENTITY> waited.
active turn:  The active registry code is
```

Exactly two distinct entity/descriptor facts and exactly one distinct
opener/waiter pair are required. The opener and waiter must each resolve one
fact and must be different. Duplicate, missing, unknown, malformed, reordered
turn-boundary, extra-turn, or exceeded-bound inputs fail closed. Entity bytes
are typed equality keys only; they are never scalar scores, numeric order, or
candidate identities.

Construction parsing separately aggregates the two one-fact construction rows
into the exact two-prototype registry. Held-out prediction alone enforces two
facts in one binding turn and one opener plus one waiter in the focus turn.
Both parse shapes are bound by the grammar identity.

## Frozen admission and lower-scope isolation

Compile one fresh construction-only `SourceFreeTable` and bound
`MultiscaleCountRadiusR4V1` from IDs 27/28. At both held-out decisions require:

- final table context ` code` / ` is`, reproducing the same trigram row used
  in construction;
- count-one maximum-count tie support exactly `{ silver, violet}`;
- identical #953 token IDs, payloads, counts, coordinates, radii, fallback,
  support, and declared local work; and
- canonical #953 fallback ` silver`.

The conversation operator may rank only `max_count_tie_tokens`. It may not
admit, remove, inject, relabel, or reorder support.

Before scoring, compile the ID 45 observed prefix into the target-free audit
codec, compile ID 48 independently and require equal codec/vocabulary
identities, then construct both canonical route artifacts using the shared ID
45 codec. Candidate and target bytes are absent from that codec. The exact
identity scope is
`issue-973/conversation-entity-spin-path-heldout-v1`; turn IDs are
`binding-turn-0001`, `focus-turn-0002`, and `active-turn-0003`; and the
immutable global snapshot is the one unit `registry` under
`canonical_global_epoch([registry])`. Require exact equality at
current, previous, last-two, sentence, and paragraph identities and ordered
H4 states, and require a distinct conversation identity. Global identity,
ordered H4 state, immutable snapshot, and epoch must also match exactly. The
canonical global node is rooted directly in the immutable global snapshot, not
in the conversation root. Any global mismatch is contract-invalid, and no
global field or operator may enter this score.

The immediately preceding completed turn and active turn are byte-identical,
so a deterministic lower-scope-only selector cannot satisfy both incompatible
targets. `PriorSentenceCountRadiusR4V1` must abstain with
`NoPriorCandidateOccurrence`. The retained paragraph operator remains bound to
its own frozen artifact; this probe does not modify or relabel it. The current
paragraph contains no descriptor fact and supplies no paragraph entity binding.

## Exact stored-spin conversation route

For registered lexical surface `u`, use the exact stored
`GeometricAddress.spin = (s3, hopf, fiber, torsion)`. Hopf must reproduce from
S3 and remains an audited trace field.

`CanonicalS3SpinToH4V1` maps `s3.raw=[q0..q3]` only when every coordinate is an
exact `a_i * 2^29`; its coordinate is `[[a_i,0];4]` in the bound scaled
`(1,i,j,k)` H4 root table. Resolve by exact coordinate equality and reject
nonmembers or aliases. No nearest-root projection, table-index distance,
prime/hash/modulo placement, float, Cayley approximation, or candidate-derived
geometry is permitted.

Define:

```text
G(u) = (M_s3(u), fiber_q29(u), torsion_q29(u))

(h,f,t) o (h',f',t')
  = (h*h', wrap_q29(f+f'), wrap_q29(t+t'))

(h,f,t)^-1
  = (h^-1, wrap_q29(-f), wrap_q29(-t))

B(d) = G(carried) o G(the) o G(d) o G(marker)
O    = G(opened) o G(the) o G(registry)
F_RC(d) = B(d) o O
```

The ordered `B(d) o O` composition is the declared completed-turn order. Turn
IDs, ordinals, boundary grammar, and composition order are artifact-bound even
though boundary bytes are not invented as geometric leaves. All H4 products
and inverses are exact table reads. Fiber/torsion wrap and distance use bounded
integer add, subtract, compare, and table operations only.

Construction binds exactly:

```text
prototype(silver) = F_RC(spiral)
prototype(violet) = F_RC(faceted)
```

For held-out conversation `H`, exact typed parsing resolves the opener in the
final completed turn and then resolves that entity's descriptor in the earlier
binding turn:

```text
d_H = descriptor(binding_turn, opener(focus_turn))

Delta(H,c) = prototype(c)^-1 o F_RC(d_H)

cost(H,c) = (H4S3AngularShell(Delta.h4),
             circular_abs_q29(Delta.fiber),
             circular_abs_q29(Delta.torsion))
```

The unique lexicographic minimum over unchanged #953 support wins. An equal
minimum abstains to the exact #953 fallback. Candidate/token/address order is
trace order only and cannot break a tie. The construction-recurrent candidate
must measure `(Coincident,0,0)` and the competitor must be strictly larger, or
the target-free hard gate stops.

The descriptor path is refolded from the bound construction prototype's exact
stored leaves after typed cross-turn resolution. This is exact
construction-recurrent route use, not induced semantic similarity. Candidate
identity may index an already-admitted prototype row, but candidate spin,
address, prime, hash, and payload bytes are not score coordinates.

The operator artifact binds the table and overlay CIDs, construction IDs and
text CIDs, codec/vocabulary/route-manifest identities, exact spin-to-H4 map,
H4 root and multiplication tables, grammar, turn/boundary order, candidate and
prototype rows, bounds, fixed-work schedule, and all provenance traces.

The fixed bounds are two construction documents, two completed held-out turns,
three hierarchy-audit turns including active, two facts, two focus roles, two
candidates, seven stored path leaves, 96 observed lexical units, 1,536 combined
observed bytes, and a 1 MiB canonical operator ceiling.

## Matched controls

Every arm receives identical input bytes and performs the same two completed-
turn slots, two fact slots, two focus-role slots, two entity-key comparisons,
two descriptor-row comparisons, seven stored-leaf reads, nine exact H4 product
table reads, two exact H4 inverse table reads, 18 phase additions, four phase-
distance reads, two angular-shell reads, two cost comparisons, and one final-
choice operation, plus the unchanged #953 local ledger:

1. `real`: use the descriptor bound across the earlier binding turn and final
   focus turn.
2. `conversation_disabled`: complete the same census and relation work, mask
   ranking costs, and return the #953 fallback.
3. `cross_turn_binding_permuted`: bind the opener to the waiter's descriptor
   after the same typed parse; the candidate winners must swap.
4. `binding_rows_reversed`: reverse the internal parsed binding-row vector
   while preserving entity keys and declared turn ordinals; the result must
   equal real.

The fourth arm is an internal storage-iteration control under unchanged input
bytes. It is not evidence for feeding textually reversed conversation turns.

## Target-free hard gate

Before constructing or attaching expected continuations:

1. Verify D3 membership, exact frozen bytes, ID/text-CID disjointness, turn
   structure, shared lexical multiset, candidate absence, and bounds.
2. Compile the construction-only table, #953 overlay, and conversation
   operator; serialize, reload, and rebind every artifact.
3. Freeze the two observed-prefix hierarchy audits and require the exact
   lower-scope equalities, distinct conversation identity, exact global
   identity/state/snapshot/epoch equality, and no global score input described
   above.
4. Require unchanged #953 support/work and the exact two-candidate count-one
   tie in both cases.
5. Enumerate every candidate-relative stored-spin state and cost. Require one
   unique real minimum per history, incompatible real winners, swapped
   cross-turn-permutation winners, row-reversal invariance, disabled fallback,
   distinct complete prototypes, no class alias/tie, and no candidate/order
   tie-break.
6. Require exact address-to-payload inversion, zero support/work mismatches,
   `PriorSentenceCountRadiusR4V1` abstention, and zero teacher, provider,
   source-weight, future-unit, target, partition-ID, full-history-key, or global
   operator reads.
7. Freeze and replay the canonical operator and target-free census bytes/CIDs
   exactly.

Structural leakage, partition, grammar, lower-scope, support, work, or binding
failure is:

```text
INVALID_CONVERSATION_SCOPE_CONTRACT
```

A clean geometric alias, tie, inert real state, or unswapped causal control
stops before decoding at:

```text
RETAIN_PARAGRAPH_ONLY_REDESIGN_CONVERSATION_ENTITY_SPIN_PATH
```

No fixture word, ID, threshold, map, scalar, coordinate order, prototype,
control, or tie-break may change after this contract is posted.

## Predeclared decoded join

Only after the operator and target-free census replay freeze, construct the
publicly predeclared continuations:

```text
45 -> " silver."
48 -> " violet."
```

This is execution-order separation. The targets are public in the frozen
contract; the result is not blinded, cryptographically sealed, or hidden-label
evaluation.

The exact required matrix is:

| Arm | ID 45 | ID 48 | Exact |
| --- | --- | --- | ---: |
| real | ` silver.` | ` violet.` | 2/2 |
| conversation disabled | ` silver.` | ` silver.` | 1/2 |
| cross-turn binding permuted | ` violet.` | ` silver.` | 0/2 |
| binding rows reversed | ` silver.` | ` violet.` | 2/2 |

The conversation operator owns only the first selected lexical unit.
Subsequent units return to unchanged #953 continuation, must append `.`, observe
EOS, and terminate deterministically without a period-1/2 cycle. Require exact
payload inversion, zero first-decision support/work mismatch, and byte-identical
outcome replay.

If target-free state qualifies but this decoded matrix, termination, or replay
fails, retain state only at:

```text
RETAIN_CONVERSATION_STATE_ONLY_REDESIGN_CONVERSATION_READOUT
```

The only positive terminal is:

```text
RETAIN_CONVERSATION_ENTITY_SPIN_PATH_ATTENTION_CONTINUE_BOUNDED_GLOBAL
```

A positive retains this bounded conversation mechanism and permits exactly the
next independently frozen bounded-global #973 contrast. #973 remains open,
corpus induction remains `NOT_RUN`, and #954 remains blocked. A negative
preserves the paragraph result and returns to conversation representation or
readout design according to the typed terminal above.

## Observed result

An initial preliminary execution produced the same decoded matrix, but the
independent pre-delivery falsifier rejected its evidence identities: the
grammar kappa omitted the separate construction parse shape and held-out
boundary order, and the disabled arm declared rather than literally repeated
its entity resolution. No terminal or preliminary identity was accepted. The
implementation then bound both complete parse shapes and both boundary orders,
made the disabled arm independently repeat entity resolution and geometry, and
reran the entire permitted target-free/decoded test. The result below is only
that corrected replay; the fixture, mechanism, route, controls, bounds, targets,
and decision rule did not change.

The target-free hard gate and the predeclared decoded join both passed without
changing the frozen fixture, route, coordinate order, prototype map, control,
tie rule, bounds, or expected matrix. The retained terminal is:

```text
RETAIN_CONVERSATION_ENTITY_SPIN_PATH_ATTENTION_CONTINUE_BOUNDED_GLOBAL
```

The decoded comparison was:

| Arm | ID 45 | ID 48 | Exact |
| --- | --- | --- | ---: |
| real | ` silver.` | ` violet.` | 2/2 |
| conversation disabled | ` silver.` | ` silver.` | 1/2 |
| cross-turn binding permuted | ` violet.` | ` silver.` | 0/2 |
| binding rows reversed | ` silver.` | ` violet.` | 2/2 |

The real arm therefore required the older cross-turn entity binding to choose
between the two unchanged, already-admitted candidates. Masking the
conversation ranker collapsed both cases to the unchanged #953 fallback;
permuting the binding swapped both winners; reversing the internal parsed-row
order preserved both winners. Candidate support and the declared-work ledger
matched in every arm. `PriorSentenceCountRadiusR4V1` abstained with
`NoPriorCandidateOccurrence`. Both real continuations emitted exactly two
units, appended `.`, observed EOS, and terminated without a period-1/2 cycle.

The independently compiled hierarchy audit established the intended scope
contrast: the lexical multiset, current, previous, last-two, sentence, and
paragraph identities and ordered H4 states were equal; global identity,
ordered H4 state, immutable `registry` snapshot, and canonical epoch were also
equal; only the conversation identity differed. The score read no hierarchy
audit field and no global operator. The recurrent prototype cost was exactly
`(Coincident,0,0)` and its competitor was strictly larger in each real case.
This execution did not separately qualify which coordinate of the complete
lexicographic H4-shell/fiber/torsion cost was decisive.

The frozen evidence identities are:

| Evidence | Identity |
| --- | --- |
| source-free table | `blake3:43a9a374d9a67ef3b1b195695b23a1246cc49562679205bc2e37937620cca772` |
| #953 overlay | `blake3:b99f7248209d020b707d904ed779f4893afabe7cfa5ab0c8fe89da1f65cbc121` |
| conversation operator, 11,775 bytes | `blake3:343c961b06605f6ae9bb6160ac34a98224991715b706156349a8fd544b6dbb35` |
| construction codec | `blake3:93770f2ff5047711bdc469b5fd967fcd60434d0a9b597f6f59b9d2c9d6675972` |
| construction vocabulary | `blake3:52ca893d0f812e9223e8c90500c816d20768bf6cf81afb3da496e3ba3e4cbd6a` |
| construction route manifest | `blake3:aa9b7200022b6043552ba55eb381fe807e94c2c2d7c7080e60b5d1efecb7da39` |
| audit codec | `blake3:15a5b9617fd0f7f321fdc1ff57fe6efa92bbb901d4865683076f6d1a9d3e636a` |
| audit vocabulary | `blake3:7ecc390cd789f13bef760c114ce74773cf441f67314aae99943896d65d85a9d9` |
| ID 45 audit route manifest | `blake3:4c6b47f7128ab7f6dcd34f702785157020375e5b31efeb00eef3093ce652bc53` |
| ID 48 audit route manifest | `blake3:888c3e4eedb8324ef13eaad5c0939d481ee4d979b47e43651c88309f651f5193` |
| exact spin-to-H4 map | `blake3:5cc6d40c7b80e7c0519a08d95ea33ef6b079c9decfbd602c99c9937fabb7bb01` |
| grammar | `blake3:806da4339d4eaab98b3f3b18b1c08ad567344de91c9ab3964d3150431a12f710` |
| routing policy | `blake3:f14406b76a8d5714c18d1664b231e94b77b086c45e48c4321792765e46654b22` |
| hierarchy-audit policy | `blake3:d61762a4f577c0b54bc0b071dfa496d35d8f0b97fc63fb2ec4f1b4b85daa051f` |
| H4 root table | `blake3:8d33d62a239fb8001fea2bd14a9a5ec7321d0f07d81c74a5715eaeb3df53aa76` |
| H4 multiplication table | `blake3:90ee73a27ee2e8ba5bccd1507d7fb37ed1f044b1640772c86752bc0bb2111759` |
| global epoch | `blake3:778a2115124219e8b64b1d000f0d365a29faae37ba7140b999c50f533765a5f3` |
| target-free census | `blake3:649d733a194469aa648101a873d9e2ee323266b18872ced412d1da2cc6a56635` |
| decoded smoke | `blake3:6930de3c07d30df4420bb68e60ea74531c8076516bcfef1c016240eddf1b9ca2` |

Canonical serialization/reload, target-free census replay, and decoded-report
replay were byte-identical. Operator tampering, binding-byte drift, malformed
scope inputs, and bound overruns all failed closed.

## Claim boundary and decision value

A positive would establish only one bounded synthetic, construction-bound
exact-descriptor cross-turn entity-role phase-path selector. On this exact
contract, an earlier completed-turn binding would change the unique
lowest-cost already-admitted candidate and two-unit decoded continuation while
the final prior turn, active turn, local-through-paragraph state, support, and
work remain fixed.

Exact descriptor/path recurrence remains operative. No arm compares this
mechanism with a direct non-geometric ordered binding lookup, so a positive
does not establish a geometric advantage over that simpler lookup. It also
does not establish semantic or paraphrastic transfer, natural/corpus transfer,
hidden-label generalization, anti-recall beyond candidate absence and
full-history disjointness, intrinsic or prime/index-independent geometry, a
general entity or conversation model, general conversation/global/recursive
attention, grammar, coherence, correctness, reasoning, chat, performance,
formal closure, or release readiness.

The positive and negative terminals authorize different next actions, so the
probe has decision value.

The observed positive result now retains only this bounded mechanism. It
authorizes one independently frozen bounded-global exact-spin operator
contrast next. Corpus induction remains `NOT_RUN`; #973 remains open and #954
remains blocked.

## Run contract and activated checks

```text
metric to move:       exact decoded conversation matrix, current NOT_RUN
reachability ceiling: 2/2 fixed held-out decisions conditional on the target-free hard gate
cheap instrument:     target-free hierarchy/support/spin-path census and exact replay
instrument verdict:  must show lower scopes equal, RC distinct, real unique winners 2/2,
                     disabled fallback 1/2, permuted winners 0/2, invariant control 2/2
exit rule:            exact predeclared decoded matrix with zero support/work mismatch
if positive:          retain bounded conversation mechanism; freeze bounded-global contrast
if negative:          retain paragraph only or conversation state only per typed terminal
cost estimate:        focused debug probe expected under 5 minutes; 10-minute hard wall
```

Activated checks are only the focused #973 target-free/decoded test, exact
binding/tamper/replay checks, touched-package compile, the root WASM library
compile needed by the new public core module, Rust formatting, claim wording,
and diff whitespace. Workspace-wide tests, BDD, teacher/model paths, no-std,
Gate C, kappa reproduction, audit, fuzz, conformance, corpus-scale, product,
performance, formal, and release suites remain `NOT_RUN` because they cannot
change this decision.

## Observed check ledger

- `cargo test -p uor-r4-core --test conversation_entity_spin_path_attention_973 --offline -- --nocapture`
  — **PASS**, 2 passed, 0 failed, corrected focused replay finished in 264.10s.
- `cargo check -p uor-r4-core --all-targets --offline` — **PASS**.
- `cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib --offline`
  — **PASS**; existing workspace warning output did not fail the check.
- `cargo fmt --all -- --check` — **PASS**.
- `python3 scripts/check_claim_wording.py` — **PASS**.
- `git diff --check` — **PASS**.

Workspace-wide tests, BDD, teacher/model paths, no-std, Gate C, kappa
reproduction, audit, fuzz, conformance, corpus-scale, product, performance,
formal, and release suites were **NOT_RUN** under the frozen decision-bearing
check boundary.

## Forward-action update (2026-08-28)

This bounded result remains unchanged. Its then-next sequencing is complete;
the then-current #973 work was ADR-0005
`PredictiveConnectionRetentionGate0V1`.

## Current sequencing update (2026-08-29)

The intervening gated-delta smoke and direct-attention V3 experiment are now
complete. The gated-delta cell showed no advantage on its bounded fixture, and
the equal-manifold-budget direct-attention result was geometric `3/12` versus
plain fixed-tangent `12/12`. `ConnectionGaugeCovarianceV4` Phase I has since
qualified separately trained H4, alternative-tangent, and fixed-tangent arms
under explicit local coordinates. Its target-free held-out freeze and salted
commitment are sealed in PR #1001; protected merge/reveal is the active #973
action before any
paired-E8, corpus, resonance-sieve, recurrent-factorization, or exact-lowering
work. This update changes no earlier conversation-scope evidence or identity.

## Successor direction (2026-08-29)

V4 subsequently completed terminal-negative at `13/24`, without adequate
separation from its destructive controls. V4 will not be rerun or retuned. The
HELM-D-R4 became the full-decoder, gauge-equivalent ordinary-causal-softmax
reference with R4/Spin frame transport. Its parity gate now passes; the verdict
and scope are authoritative only in the
[HELM-D-R4 result](helm_d_r4_softmax_decoder_result_973.json). The active #973
successor is intrinsic R4 distance and normalized-centroid attention, followed
conditionally by multi-resonance replacement and recurrent lowering.

## Attempt 02 successor update — 2026-08-29

This record's bounded evidence and the `HELM-D-R4` gauge-equivalent ordinary-
softmax PASS remain unchanged. The separately trained intrinsic Lorentz-R4
successor stopped before D3 at
`UNAVAILABLE_INTRINSIC_LORENTZ_R4_STOP_BEFORE_HELD_OUT` (result CID
`blake3:da2a63323d6211b8d581e5a4ed75d788eb919ff0f210d2e3beb8a749ee1bc64f`):
normalized-barycenter covariance was `9.1214e-8` against the frozen `1e-8`
limit, and construction-validation NLL was diagnostically worse than the donor
by `1.2531` and the matched flat control by `0.20893` nats/token. No reveal
marker or held-out result exists. No Attempt 03 is authorized under this freeze;
any further intrinsic work must be a newly frozen, source-faithful
learned-manifold successor. Multi-resonance, recurrence, lowering, scale, and
#954 remain blocked. See the
[owning intrinsic record](intrinsic_lorentz_r4_attention_973.md) and the
[compact result summary](intrinsic_lorentz_r4_attention_attempt_02_summary_973.json).
