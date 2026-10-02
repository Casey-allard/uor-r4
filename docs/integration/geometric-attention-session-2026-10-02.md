# Persistent integer geometric attention — source implementation

Status: source implemented and statically reviewed. Three focused release
integer session tests pass on hosted Linux at source head5e9dfb52. The training
library compiled; the replay example first failed with E0508 and then builds
successfully at e19bbd33 after a one-line borrow repair. Loaded comparison, allocation and
compiled-instruction qualification remain **NOT_RUN**.
This record does not promote the session, its mathematical adjoints, or the
surrounding language model. The active source card is
[#1512 comment5958637499](https://github.com/UOR-Foundation/uor-r4/issues/1512#issuecomment-5958637499).

## Implemented source and reason

The existing native composition caller recomputes event/context/span histories,
every occurrence payload, and every causal query row whenever a prefix is
extended. The new `integer::geometric_attention` session maintains those states
and caches exact occurrence addresses and composed payloads. A push updates the
current occurrence once and scores/reduces only the newest query row.

Initial scope is the retained H2/L4/width32, context at most128, rra/layer2 reader.
Each real token is a distinct occurrence. It remains in the denominator even
when its address is absent, its value is present-zero, or nonzero atoms cancel.
There is no token deduplication, support pruning, coefficient clipping, eviction,
new selector, new artifact format or optimizer dose.

The push stages independent event/context states, observes the OLD held span,
uses NEW context state/observation for the current value and NoRead, caches its
actual packets and signed-bank composition, includes self in causal support,
normalizes each head separately with its NoRead, and checks the integer head
sum. Only then are span/state/history/publication committed. Span COMMIT at t
is visible at t+1. Capacity failure rejects before mutation; private unpublished
scratch may be overwritten by a failed reduction but cannot extend public
history or replace the last successful output. Reset clears all logical state.

The training adapter joins the complete dependency metadata of the admitted
context/event/span/potential/value/NoRead/reducer/composition objects, their token
registry, exact signed H4 algebra and Q25 observation basis. Its immutable kernel
accessors introduce no per-token parsing, hashing, source-model call or Tensor.
Strict q4 context, potential, value and NoRead artifacts are required. Event
source coefficients and learned age biases retain their wider policies and are
explicit in the admission receipt.

The CPU Stack bridge runs one session per batch window, converts the final i64
Q16 head sum outside the integer session, and inserts it at the same residual
site that already bypasses `read.out`. Floating recurrence, MLP, residual stream,
normalization and vocabulary projection remain. It does not edit
`stack_grounded_session.rs` or replace Claude's Metal training path.

## Prepared decisive checks

The new comparison example loads the two unchanged, sealed strict-context
lineages from the completed [context construction](geometric-context-q4-2026-10-02.md).
It binds the exact parent report, sealed argument bytes, loaded context
metadata and retained episode hashes; no artifact is compiled or rebound during
evaluation. It compares each token's event/context
state/action, raw observations, old-held content, value packet/raw choices,
composed payload, every causal score/age, occurrence/NoRead weights, denominator,
maximum score, head output and checked sum with the native whole-window path.
It retains full reader traces and checks actual-position Stack logit bits plus
the prior saved answer logits/predictions on all original/stress rows.

A valid preceding value artifact with a different context lineage must be
refused. Capacity/invalid-token rejection and reset are checked on each loaded
episode. Separate same-shape Stack comparisons cover a short prefix and B2
with different retained sequences. Synthetic integer cases cover delayed span
visibility, repeated occurrences, absent/present-zero/cancellation, independent
head denominators and a late second-head error. The capacity fixture also
pushes all128 occurrences, checks every support length and independent weight
sum, and rejects the129th without publishing a new result. These tests are
executed successfully on hosted x86_64 Linux:3 passed,0 failed/ignored.

The storage receipt counts actual inline layout and constructor-owned heap
elements, including padded address storage, payload cache, span buffers and both
private exponential tables. It excludes borrowed immutable kernels, allocator
overhead and temporary stack frames. It is not peak RSS or an energy measure.
Compiled successful-push/callee allocation and instruction inspection remain
necessary before claiming no allocation, float, multiplier or divide opcode.

A follow-up caller review found six complete legacy `ReadSource` initializers
that omitted the new optional residual field. All six now explicitly initialize
it to `None`; the other four literals use the default or set the injected
residual. This repairs a source-level compilation blocker across ordinary,
span, binding and composed callers. Direct parsing/formatting and diff checks
pass; the focused integer compilation/tests now pass on Linux. Loaded model
comparisons remain NOT_RUN. No numerical
operator, learned artifact or learning policy changes in this repair.

## Execution admission and continuation

The source projection is240min total: source/review120, cold rebuild/checks45,
unchanged no-update comparisons30, delivery45. Each admitted build/worker uses
2threads/3GiB; one Cargo at a time. Loaded workers retain840-second internal and
900-second external bounds; research/archive each at most256MiB, regenerated
private build cache at most1.5GiB, physical stop margin128MiB. All preparation,
reviews, failures, waits and delivery remain cumulative ledger charges.

At the initial source checkpoint, actual peer workers declare6+4threads against the
shared ceiling8. No own Cargo/model worker has started. The live coordination
observation is on #820; the source is preserved in a draft PR pending scoped
execution. This is a capacity constraint, not attention-quality evidence.

After exact orchestration is exercised, event source width and age policy remain
the next reader coefficient decisions. Selected parameter/candidate access,
float trunk/output replacement, natural-input durable conversation and novel
coding/reasoning remain programme obligations. Offline donor/operator compilation
is retained. Neither this source implementation nor context's retained45 losses
establishes a reason to retire geometric attention.

## Learning policy: primary-literature applicability review

The independent mathematics review does not justify changing context learning
while this session is implemented. The native-conditioned forward chooses exact
actions deterministically. [Rao-Blackwellized straight-through Gumbel-Softmax](https://arxiv.org/abs/2010.04838)
reduces conditional sampling variance under a categorical/Gumbel sampling law;
that law is absent here. The [2023 counteranalysis](https://proceedings.mlr.press/v202/shekhovtsov23a.html)
also does not establish stability for this deterministic recurrent surrogate.
The retained large-gradient draws differ in input content and history; they are
not a controlled context-length or estimator-variance experiment.

Tangent projection is already the historical context adjoint. It can erase
useful finite-choice credit: at the positive identity, a radial gradient of
`softplus(alpha * x0)` projects to zero although the negative identity lowers
the loss for positive alpha. Reducing a norm by restoring that projection is
therefore not evidence of improved learning. The current finite-choice backward
already enumerates all120 local actions through a linearized downstream
adjoint; simply enumerating exact roots again does not supply actual nonlinear
suffix losses. This is a mathematical/source review, not model execution.

A distinct later diagnostic remains available if directional context credit
becomes the observed blocker: freeze one real transition site and evaluate
actual downstream loss after each exact action intervention, replaying the
hard suffix and affected neighbor/consumer states. With those120 losses `C_a`
held fixed, the local surrogate `S = sum_a softmax(z)_a C_a` has derivative
`p_i * (C_i - sum_a p_a C_a)`, bounded coordinatewise by one quarter of the
observed cost range. This is a finite local objective, not the deterministic
model gradient, a proof of shared-q4 realizability, or a learning guarantee.
A later work card would need to compare that ordering with the current credit
and state the decision its extra replay cost can change. No intervention run,
new estimator, tangent change, stochastic serving, temperature sweep or fit is
part of this card. Preserve the mechanism and finish the persistent integer
session first.

## Hosted execution — October 2

Following the owner's runner direction, reuse Claude's temporary `codex/ci/*`
method: the exact product head plus one workflow-only commit, never merged into
the product. [Run37063718908](https://github.com/UOR-Foundation/uor-r4/actions/runs/37063718908)
executes two hosted Ubuntu jobs in parallel, with source identity guards. The
integer release build takes31.54s; all three intended fixtures pass in0.04s
using Rust1.97.1/x86_64-unknown-linux-gnu. Downloaded source SHA records match
all12 changed Rust files at the tested head.

The training library compiles, but the example's retained packet comparison
tries to move a non-Copy record out of an array (E0508). The failed log remains
retained; e19bbd33 changes only that access to a borrow. A driver-only hosted
[retry37064609925](https://github.com/UOR-Foundation/uor-r4/actions/runs/37064609925)
passes the release build in6m03s inside its35min ceiling; integer source is
unchanged and its prior passing checks are retained. The downloaded Linux
binary SHA256 and all12 owned changed Rust source hashes are verified.
[Execution receipt](../evidence/geometric-attention-hosted-2026-10-02.json). No source policy,
numerical operation, artifact or learning dose changes.

Hosted compilation removes the local build-slot dependency. Saved-parent replay
still needs the retained model/artifacts; no private artifacts are uploaded by
these jobs. Linux binary/instruction evidence is platform-specific and cannot
qualify M1 runtime, energy or whole-model serving. There is no local Cargo/model
run. The clean temporary CI checkout was removed after its workflow commit was
pushed; its branch/workflow and all research remain preserved.

## Native M1 runtime checks and observed descriptor repair

[Run37065787933](https://github.com/UOR-Foundation/uor-r4/actions/runs/37065787933)
passes the same three release integer session fixtures and builds the native
ARM64 macOS replay driver with Rust1.97.1. Downloaded binary and owned-source
hashes agree. [Allocation run37067431577](https://github.com/UOR-Foundation/uor-r4/actions/runs/37067431577)
atd2df6588 executes256 successful pushes and2 resets over two full128-occurrence
sessions:4 nonzero,2 present-zero and250 cancelling-nonzero rows, with zero
allocations, allocated bytes, reallocations or deallocations. Constructor, TLS
initialization, warm-up, assertions and error/report formatting are excluded.
This measures the synthetic successful session path, not a complete model.

The first actual M1 opcode scan reaches19 emitted integer symbols from
`NativeAttentionSession::push`. It finds3 MADD instructions in context step
descriptor indexing, with no hardware divider, floating arithmetic/conversion
or FMOV transfer in those ranges. This retained contract failure motivates
4a0b0224: private48-byte token and104-byte lane descriptors become64/128-byte
power-of-two strides. Coefficient arrays, artifact bytes and geometric
transition/readout arithmetic are unchanged. Additional64-bit metadata costs
16*vocabulary +24*total_lanes bytes, at most65,728B; existing coefficient-only
`stored_bytes` excludes this metadata and allocator bookkeeping/alignment
overhead. The focused size test guards the new descriptor strides.

[Repair run37067792966](https://github.com/UOR-Foundation/uor-r4/actions/runs/37067792966)
passes3 session tests,5 context tests and the allocation census at4a0b0224.
Its rebuilt native driver passes the19-symbol integer scan:0 multiplier,
hardware divide, floating arithmetic/conversion or FMOV instructions. The
observed3-MADD defect is removed in the actual emitted binary. Downloaded binary
SHA and all compiled source hashes agree; no new optimizer update occurs.
The scan uses the existing strict auditor's multiplier/divider/FP patterns
without weakening them. External memcpy/memset/bzero and defensive core panic
callees are explicitly outside its scope; closed-call whole-model serving is
not qualified. At this hosted checkpoint loaded-parent replay is NOT_RUN while
actual local peer trainers occupy shared capacity. The subsequent capacity
release and replay are recorded below. No private model artifacts are uploaded
or geometry family retired by these implementation checks.

## Loaded-parent replay after capacity release

DeepSeek exits; the two registered Claude arms use5 threads/7GiB, so the
declared sequential2-thread/3GiB replay workers are admitted. Both exact
retained parents complete in8.78s and9.46s external elapsed; peak RSS is
843,300,864 and735,199,232 bytes. Each worker seals and verifies its report.
Saved-parent/context dependency identities remain pinned;0optimizer updates.

All512 exposed retained rows preserve saved answers and all actual-position
whole-window/incremental float-tail logit bits. Answers remain93/110 for seed1
and124/119 for seed2,446/512 total. This preserves the prior context conversion's
13gains/45losses against its older478/512 parents; it is not a recovery fit or
model-quality improvement. Exact token-step reader comparisons pass, including
OLD held spans, context/event actions/states, packet status and composed values,
causal compatibility/age scores, current occurrence support, separate NoRead
weights/denominators, per-head reductions and checked head sum. Capacity, invalid
token and reset checks pass. B2 same-shape and short-prefix checks pass; a valid
older value artifact with the wrong context dependency is rejected.

[Loaded execution receipt](../evidence/geometric-attention-loaded-2026-10-02.json).
Independent saved-output review passes:512 saved answer vectors are bit-identical,
all66 errors unchanged,28,976 token context/value traces and1,883,664 causal score
pairs match,57,952 head normalizations/reductions reconstruct, and1,024 parent
query heads match with0mismatches. The saved-data audit takes18.52s. Full-position
logit equality, unsaved fresh-reference event/OLD-held fields and B2/reset/refusal
remain fail-closed executed-driver assertions, distinct from the independent
saved-output audit. The complete token traces exceed
the original256MiB retention estimate by~84MiB; the overrun is recorded, and the
standing owner storage extension sets a512MiB research ceiling before further
archive/delivery. No result is discarded. Session-owned inline/heaped elements
are165,976B per report, excluding borrowed tables/allocator overhead/temp stack;
RSS includes the offline model driver and reporting and is not a serving/energy
measurement. Event/age wider coefficients, selected access and float trunk/output
remain unfinished. Preserve this component and advance those boundaries after
delivery.
