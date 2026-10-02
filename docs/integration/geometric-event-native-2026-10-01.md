# Native geometric context and capture events

Status: measured native context/event learning and compiled component retention.
This record owns the
context/event component study for #1512, not a whole-language qualification.
The retained ordered-span producer and relative-potential scorer remain the
parent interfaces. Offline donor-informed compilation remains supported.

## The causal change

The saved ordered-span models used a floating-point context/event controller to
decide HOLD, OPEN, APPEND and COMMIT. The new controller reads token identity and
its own exact signed H4 states. For each lane, it scores 120 group actions using
a token row, the old own state and the old next-neighbor state. It chooses the
first maximum, then right-composes the old state with that action. All lanes
read the old states before any lane is updated. Four event potentials read the
new states and a token-specific bias. The winning typed event feeds the integer
span register directly. Float-logit reconstruction and another event argmax are
absent from that component chain.

The transition-state factors are restricted to four quaternion coordinates,
not a table over all joint lane states. Token/action rows and evaluated state
potentials compile to fixed Q24 signed integer tables. Serving the controller
uses integer additions, comparisons and exact group table operations; table
entries are 32-bit coefficients, not the project's four-bit additive-map codec.
The two-lane controller reads 732 learned coefficients per token, excluding
fixed geometry/metadata accesses. This is a component count, not a measured
whole-path bandwidth or energy saving.

The 120 signed root actions are the binary icosahedral quaternion group (2I),
not all elements of the H4 Coxeter group. The two context lanes are independently
updated recurrent states; they are not a claim to implement the golden/Galois
companion in `H4 ⊕ phi H4`. Pinned exact `Z[phi]` geometry supplies the group
tables. Prime/UOR occurrence identity stays separate from the geometric code;
hash bits do not define semantic proximity. Fixed zeta phases, retained Hopf
fiber/torsion and the paired-H4 conversion remain programme mechanisms with
their existing scoped roles, rather than implied new controller inputs.

Training uses Rust/Candle and floating point offline. A custom backward bridge
uses tangent-projected state credit, the adjoint of the selected Hamilton
product, and a temperature-one relaxed action distribution. It retains both
own-state and neighbor-state score feedback. This is a declared biased
first-order surrogate, not the derivative of argmax or a marginalization over
all possible histories. The old base model, token actions and relative scorer
are frozen in the first study. Only the new event factors are optimized using
balanced event supervision plus 0.1 times ordinary answer cross entropy through
the same reader. Gold event labels are used in the loss and diagnostics, never
passed to the controller's forward computation.

This gives geometry an implemented role in deciding what is captured. It does
not finish the native current-context/radial/presence producer, age/NoRead,
softmax, values or output realization. Those portions of the mixed offline
evaluation graph remain floating point. Candidate admission and ranking also
remain separate obligations.

## Constructive capacity result

The principal mathematician checked a two-phase witness against canonical
indices and right composition: identity `I=1`, `-i=2`, `+i=3`. Let phase A be I
and phase B be +i. Use actions I, +i and -i, suppressing all other actions with
token bias -16 and zero state potentials. Own-state action vectors are
`U(+i)=(2,-2,0,0)`, `U(-i)=(-2,2,0,0)`, `U(I)=0`; neighbor potentials are zero.

| Token | Bias for (I,+i,-i) | Scores from A | Scores from B | Effect |
| --- | --- | --- | --- | --- |
| OPEN | (0,-4,0) | (0,-2,-2) | (0,-6,2) | enter A |
| COMMIT/BOS | (0,0,-4) | (0,2,-6) | (0,-2,-2) | enter B |
| key/neutral | (0,-4,-4) | (0,-2,-6) | (0,-6,-2) | retain phase |

Every maximum is unique. Right composition gives `I*(+i)=+i` and
`(+i)*(-i)=I`. Event readout vectors `APPEND=(2,0,0,0)` and
`HOLD=(0,2,0,0)` give the appropriate phase score two. Marker bias four
overrides it for OPEN/COMMIT; BOS may enter B and emit HOLD. In a two-lane
witness the second lane's event readout must be zero, or both readout bases must
be halved, or marker bias increased to eight. Duplicating both readouts with
marker bias four creates a tie and breaks COMMIT under earliest-event tie
breaking. All witness coefficients are exact in Q24.

The reset-each-token control starts each key in A. It therefore wrongly
APPENDs outside noise keys that the ordinary recurrent witness HOLDs, while
inside keys still APPEND. The authored generator includes these outside keys
after COMMIT. This establishes representability of this explicit-boundary,
BOS-initialized grammar. It proves neither successful optimization nor nested
scope, natural-language capture, universal state tracking or geometric semantic
advantage. No fit was run to establish this witness.

## Primary literature and decision implications

[Tracking States or Tracking Cosets?](https://arxiv.org/abs/2609.29951) studies
running group products and reports recurrent models with intermediate coset
resolutions, including non-normal subgroups. Right cosets can be updated
sequentially. Its order-blind limit concerns finite groups and full-support
i.i.d. input laws, not our authored capture distribution. Its causal state
patches motivate distinguishing retained coarse state from complete state
tracking. Our implication is to inspect actual context/action traces and reset
effects when a result is partial; an aggregate answer deficit cannot by itself
establish absence of useful learned state. This is a recent preprint, not an
independently reproduced result in UOR-R4.

[The Illusion of State in State-Space Models](https://arxiv.org/abs/2404.08819)
places specified fixed/diagonal/simultaneously diagonalizable, log-precision
linear SSM families in a restricted circuit class. It separately examines
nonlinear recurrence and input-dependent non-diagonal transitions. These
results do not classify our state-dependent group argmax recurrence. The
relevant design implication is to preserve ordered noncommuting transitions
and nonlinear state feedback, then measure the intended behavior rather than
assuming recurrence alone solves reasoning.

[Unitary Evolution Recurrent Neural Networks](https://arxiv.org/abs/1511.06464)
motivates norm-preserving recurrent transport for long-term credit. Our exact
unit-quaternion composition provides such transport at the state-operation
level; learned score feedback and the biased backward bridge still require
their own checks. [Full-Capacity Unitary Recurrent Neural Networks](https://arxiv.org/abs/1611.00035)
shows why a restricted unitary parameterization can have a capacity limitation.
Its particular dimensional theorem is not a theorem about this controller.
Together they motivate separating representability, gradient transport,
optimization and measured downstream behavior.

## Donor recompilation remains a supported route

The compiler boundary must preserve three kinds of information: token actions,
contextual read/write/transition behavior, and output behavior. Token embeddings
can supply learned lexical structure, but are not sufficient evidence of
transferred reasoning. Offline donor q/k or hidden trajectories may supervise
geometric transitions and relative potentials; the served artifact must execute
the resulting group operations and tables without loading the donor or calling
its dense operators.

Exact lowering is possible only where the source computation has a declared
finite domain and a representable target function. Arbitrary pretrained weights
may require approximation or distillation, with explicit error and retained
capability measurements. Replacing each dense layer with a large lookup is not
the target architecture. A state quotient is an exact reduction only when
merged states have the same observations and remain equivalent under allowed
continuations; nearby embeddings or matching current answers do not establish
that condition. Hidden context/transition comparisons therefore supplement
emitted-event comparisons in the native compilation evaluator.

The next donor decision should use a pinned available donor and a declared
contextual behavior panel, compile its token/context/output information into
the same native interfaces, and compare held-out outputs and causal state/read
interventions. Preserve native-learning and donor-informed arms. Do not turn a
numerical parity study or an embeddings-only transfer into a reasoning claim.

The minimum available donor study is the saved seed-2 parent's contextual event
operator. Cache its actual prefix event logits offline on a pinned stream,
then fit one controller at fixed matching conditions using those targets in
place of annotation event credit. Independently reload the compiled controller
and compare context states, events, spans, selected occurrence and answers on
the unchanged panels and reset control. Distinguish learning differences,
compilation differences and retained donor mistakes. This is one-operator
behavioral distillation, not direct conversion of an arbitrary weight set; the
output interface and held-out compositional reasoning evaluation remain needed.
The current event vocabulary limit is 4,096 and requires an explicit compatible
mapping or declared extension for larger donor tokenizers.

## Execution and outcome

The initial projection is two seeds, 320 updates per seed, batch eight,
128-token training/evaluation ceilings, width32, two heads and two new context
lanes. The first study keeps the original and longer-gap saved panels unchanged,
includes reset controls, reloads source and compiled artifacts independently,
and compares group actions, hidden states, emitted events, spans, source masses
and answer outputs. These are reused development panels, not new held-out
language tests. Limits are 1,800 model seconds, 180 complete-wall minutes,
two threads/jobs, 4 GiB RAM, 10 GiB disposable cache, 256 MiB new retained data
and 128 MiB storage stop margin. Complete elapsed work is charged once from the
previous accounting cutoff; unchanged foreign work is not charged as model
compute.

Both seeds completed all 320 updates in one attempt. Each event model contains
11,712 learned F32 parameters offline and compiles to 307,840 bytes of integer
tables plus the pinned 15,601-byte geometry payload. At recorded checkpoints,
ordinary answer loss has nonzero gradients to token, own-state and neighbor
transition factors. Event supervision is part of this study; these results are
not an answer-only learning claim.

| Parent | Panel | Native/parent correct answers | Native/reset head0 source majorities | Reset correct answers | Correct native events |
| --- | --- | --- | --- | --- | --- |
| seed1 | original | 99/99 of128 | 128/33 of128 | 21/128 | 6216/6216 |
| seed1 | longer gaps | 109/109 of128 | 126/40 of128 | 30/128 | 8272/8272 |
| seed2 | original | 128/128 of128 | 127/115 of128 | 118/128 | 6216/6216 |
| seed2 | longer gaps | 128/128 of128 | 128/116 of128 | 115/128 | 8272/8272 |

All 28,976 actual-position events are correct, including each supported role.
Across all 34,688 padded positions, independently loaded native tables retain
the trained float controller's events, group actions and context states with
zero disagreements. Span presence and codes also match that trained controller.
All 512 answer decisions and both heads' source-majority flags match the parent;
head1 itself has zero correct-source majorities in both parent and child and is
not separately qualified. The child's trained-float versus compiled-native
answer logits have 5,082 differing F32 values, maximum error
`3.933906555175781e-6`, with zero prediction changes. The surrounding reader
remains mixed, so whole-prefix float/integer bit identity is not claimed.

All 512 unchanged float baseline answer-logit and source-mass rows reproduce
the preceding saved report bit for bit. Against the preceding *native* scorer,
511 rows retain those quantities bit for bit. The remaining seed1 original
row14 (`pair203/swapped_values`) is a useful capture correction, not rounding
noise: at time11, token11 in a written key, the old controller chose HOLD and
the new controller chooses the correct APPEND. The resulting span codes differ
at positions15–26. Answer logits change by at most `0.018070071935653687` and
head0 source mass rises from `0.9924417734` to `0.9963745475`; prediction and
both majority flags remain unchanged. Post-query padding event differences
cannot affect the earlier answer. Preserve this row instead of imposing
bitwise parent-state retention on a newly learned controller.

Reset affects the seeds differently. Seed1 loses every write OPEN and changes
COMMIT to OPEN, causing large capture and answer degradation. Seed2 keeps the
boundary markers but misses 120/136 inside write-key APPEND events on the two
panels and 48 query-key APPENDs on each. Its many wrong outside noise-key
APPENDs are ignored while the span FSM is closed; those event errors do not all
corrupt memory. The answer and source changes still establish a downstream
context dependence, with the narrower role-specific explanation retained.
Reset is an intervention on this learned component, not an information/cost
matched non-geometric baseline.

Executed source: `7b0e7debfe34f790ed0c7abea887f609a5d2855a`; executable SHA256:
`0248a1128325469f2dd7b023384db64db51ff951252422965b1f2440f0c98315`.
The model process took 256.69 externally measured seconds, maximum RSS
110,903,296 bytes, peak memory footprint96,207,400 bytes and zero reported
swaps. The launch/watchdog wrapper took261.132seconds including preserved-binary
handling and observation. These are training/evaluation costs, not optimized
serving throughput or energy. Both model sources and native artifacts were
independently reloaded. The executable sealed/verified the report; independent
review verified all41 manifest entries, byte lengths, hashes and inventory.
The executed binary is retained with a hash-verified gzip roundtrip.

Twelve focused checks passed: integer4, gradient/compiler5, ordinary-answer
integration1, retained reader1 and evaluator1. Formatting, diff and claim-wording
checks passed. Independent release-assembly review confirms425 step plus3 reset
instructions, no multiply/divide/floating arithmetic and no successful-path
calls. Six defensive panic calls are not a closed-callee qualification. The
initial receipt omitted two operand-free returns; the corrected count is428,
with unchanged body hashes and arithmetic finding. The implementation commit
is `75fed22a`; the execution commit adds only evaluator trace serialization.

Retain this controller and the previously accepted token/span/scoring artifacts.
Native current-code/radius/presence production and bounded access are the next
attention obligations; native values/output remain unfinished. Keep the pinned
contextual-donor transfer as a supported compiler study, with actual output and
reasoning retention assessed at its eventual native output boundary. No new
hyperparameter/precision sweep, fresh draw, paid compute or promotion to general
language occurred. #1512 remains open.

The [machine receipt](geometric-event-native-2026-10-01.json) binds tables,
histories, per-role/reset results, row comparisons, instruction audit and the
retained root. Full raw data remain at
`/Users/casey.allard/.local/share/uor-r4/research/geometric-event-native-20261001`.
