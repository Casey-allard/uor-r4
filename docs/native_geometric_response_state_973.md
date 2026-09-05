# Native persistent response-query and selected-occurrence state — #973

**2026-09-05; implementation and initial development record.** The
optional memory schema `uor-r4.native-prime-relative-memory-read/5` retains a
bounded response query and commits a model-selected source occurrence between
generated tokens. It adds a scored source-successor alternative to the existing
local-path occurrence reader. This record describes the implemented law;
useful joint conversation/coding behavior and incremental geometric benefit
require the measurements below. Neither alpha group is qualified by its
implementation or successful compilation.

The [project plan](integration/project-track.md) owns the goal. The
[mechanism map](native_geometric_mechanism_map_973.md) connects existing parts;
the [preceding occurrence-selection record](native_geometric_occurrence_selection_973.md)
retains the matched `/3`–`/4` evidence. This successor preserves that evidence
and its artifacts. The first `/5` fit regressed on exact prose and
teacher-forced completion accuracy. The optional advancing-endpoint fit
retained 5/32 exact prose while further reducing teacher-forced accuracy.
Both measured negatives are recorded below and in the
[compact evidence record](evidence/native_geometric_response_state_973.json).

## Observed cause and scope

The corrected source `/2` comparison gave `/4` 20/32 correct prose first
predictions but only 6/32 complete exact prose responses. All Rust first and
exact results remained zero. Its `prose/100013` output began with the correct
fact, then generated ` cyra has green.\nUser: What color is ben has red.\n`
instead of the requested two-fact answer. Under teacher forcing, seven of its
nine completion pieces were correct; the semicolon and EOS were wrong. This
supports investigating response continuity and termination, without assuming
that query persistence is the only cause.

Other failures occur before that seam. The query paraphrase in
`prose/100012` admits the target red but selects badge. All eight repair
operator targets are admitted and misselected; `rust/100003` predicts addition
where the explicit instruction requires subtraction. The sixteen dependency
and function-composition numeric targets lack target memory routes. Two sampled
programs compile while asserting 120 where the supplied inputs require 86 or
23. Their recorded execution status remains `NOT_RUN` at the earlier
assessment's exact-authored-source boundary.

The present package addresses persistent query-conditioned selection and a
causal selected-read state. It does not compute absent arithmetic values,
implement operand selection, supply general code repair, learn memory write
admission, or retain source contents after ring eviction. An eventual typed
value/operator path still needs to select inputs, compute a result and write
that result for subsequent learned use.

## Representation and serving law

The initial `/5` law uses `advance_response_path=false`, the default when the
configuration field is absent. The optional endpoint revision below is bound
to a different feature-layout identity within the same memory schema.

[`response_runtime.rs`](../crates/uor-r4-core/src/native_geometric/response_runtime.rs)
implements the new state operations alongside the existing integer/table
memory runtime. `Session::begin_response` is an explicit caller boundary after
prompt observation. It captures up to the configured query-token count of
recent token/pose/phase entries, the current H4 pose and eight full `u16` phase
channels, and the existing bounded posting references those cues visit. This
is a finite response-query representation with source provenance; it is not
an inferred semantic summary or an answer parser.

During a response, the requery arm uses those captured cues, endpoint geometry
and posting references. It still compares the source-cue-to-value local path
with the query-cue-to-captured-endpoint path using the `/4` H4 product order and
full modular phase subtraction. Current last-prime/query-pair and age features
continue to condition the scores. `Requery` means selecting among these
captured source references, rather than refreshing the posting set from newly
generated output.

The retained selected occurrence additionally admits at most one direct source
successor. Both the selected occurrence and successor must still be in the
token ring, and the successor must precede the response boundary. BOS and EOS
are excluded from this continuation route. Its source step and latest observed
query-token step are compared through ordered H4 tables and full-precision
modular phases, with signed orientation retained. Reserved action addresses
distinguish continuation features from ordinary requery features. Equal token
IDs at different source positions remain distinct, and continuation/requery
evidence at the same position remains separated by action.

The model scores the existing base alternatives, captured-query occurrences
and optional successor using its learned integer tables. The winning token
and matching scored occurrence determine a transient `ResponseDecision`:
token, score, source sequence/slot when present, action and observation count.
The action is `Base`, `Requery`, `Continue` or `Stop`. EOS is the scored stop
token; the package does not add a separate stop classifier. It retains exact
prime identities, fixed zeta channels, H4 orientation and existing exact
`Z[phi]`/paired state. Identity remains provenance, not semantic distance.

Repeated prediction may recompute scratch and work counters but cannot advance
the committed response query or selected occurrence. `Session::observe`
consumes the pending decision before writing the new token. A matching
pre-observation decision can commit its selected occurrence; an externally
observed token that differs from the prediction clears that cursor. It never
searches for a source matching the observed training target. Observing EOS
closes the response. `Session::end_response` explicitly clears the query and
selected-read state before new external input.

These are causal writes of selected-read state. Token admission, source token
writes, posting replacement and eviction remain deterministic. No arithmetic
result or independently learned value representation is created by following
a source successor.

## Rust fitting and checkpoint continuation

The library entry is `MemoryReadTrainer::new_with_response_state`; the CLI
opt-in is `r4 geometric fit-memory-stream --compose-occurrences
--persist-response --supervision PATH`. The source-bound supervision intervals
supply response boundaries and loss positions. They do not supply source
occurrence labels. Every response token, including one excluded by the sampled
loss budget, undergoes model selection before the teacher token is observed.
No new source generator or response parser is added.

Each source replay pass fixes an integer selection policy. Floating-point
optimizer updates learn score features over alternatives reached by that
policy. New alternatives can expose previously unregistered features during
pointer/refinement stages; registration and capacity drops are counted. This
is conditional score learning with hard model-selected state, not a gradient
through the discrete source decision or a demonstrated learned value write.

Fitting checkpoints bind the response mode, original baseline, ordered source,
supervision, configuration and schedule. They retain the exact quantized
rollout rows for the current pass in addition to optimizer weights, registry,
stage/cursors and selection statistics. Prefix reconstruction therefore
repeats decisions under the same policy as uninterrupted fitting, even after
optimizer weights changed. Reconstruction is charged as replay work.

Because continuation can change target reachability, `/5` selects an epoch
lexicographically by correct targets on the fixed supervised population, then
reachable targets, then lower conditional cross-entropy. Losing a difficult
target cannot improve the choice merely by removing it from the loss
denominator. Final metrics use the selected quantized rows and their own
model-selected rollout. Reports distinguish discovery exposure, later feature
registration and final reachability; registration alone is not evidence of
useful learning.

Offline preparation, training and serialization remain Rust host operations.
Floating point, gradients and matrix operations remain permitted during
offline Rust training under repository policy. Serving uses the declared
bounded state, routing and integer/table law; no provider authors responses
and no dense transformer is hidden behind the lookup interface.

## Caller, persistence and control boundaries

`Model::generate` observes the prompt then begins a response. Native CLI chat
ends the previous response before appending the next user turn. The native
HTTP service does the same for nonempty appended input; an empty prompt
continues active state after a token/output limit. An empty prompt after a
stopped response captures a new response boundary. Token limits and
cancellation do not themselves replace the committed selected-read state.
The joint probe's manual first-token and teacher-forced paths begin the same
response state as free generation.

For active `/5` responses, generation observes selected EOS so the stop is
committed, while omitting EOS from rendered bytes. Older artifacts retain
their previous EOS observation behavior. Generation and the HTTP service
retain at most 96 actual decision records, including EOS when reached within
that bound. The trace is host output from the same rollout, not a second
generation or an expected-answer source-position annotation.

`ResponseStateDisabled` suppresses response capture and the resulting
selected-state continuation while retaining the `/5` learned memory tables.
It is a same-artifact intervention, not a separately trained `/4` model.
The joint probe accepts `--controls full,response-state-disabled` only for
`/5`. Existing memory-, H4-, geometry- and zeta-disabled controls retain their
declared scope. Whole-artifact geometric controls also change baseline terms
and do not isolate the new reader's geometry alone.

Schemas `/1`–`/4` retain their scoring laws and remain loadable. Response
methods are inactive for those artifacts. Optional query/reference buffers
and the extra continuation-candidate capacity belong to `/5`. Historical
training checkpoint layouts omit the new optional fitter state.

Session schema `/2` binds `/5` snapshots to the model, stores absolute memory
origin/current geometry, preserves occupied/stale index references and the
committed response query/selected sequence, and reconstructs circular-buffer
slots on restore. Pending predictions remain transient and must be recomputed
before observation. This preserves an active response within the bounded
state; it does not recover evicted source bytes or implement memory
consolidation. The core `/5` snapshot bound is 8 MiB, distinct from the native
service's 1 MiB checkpoint persistence/import bound. Historical core session
schema `/1` retains its 1 MiB limit.

## Work, resources and verification

Response capture copies at most `query_tokens` entries and
`candidate_limit` references. Per-prediction flat routing adds at most one
continuation candidate; occurrence and feature buffers are sized for that
extra alternative before prediction. New counters report captures,
reference reads, commits, requeries, continuations, base steps, stops and
observation mismatches. These counts are not interchangeable with latency.
The allocation census exercises active generation and ring eviction; the
source guard scans the response runtime along with the existing kernel.

The existing cumulative ledger remains
`.uor-models/native-joint-learning-2026-09-04/model-time.json`. The takeover
receipt recorded 784,729 ms used of 1,800,000 ms; that historical balance must
be refreshed and extended by each preparation, fitting, evaluation, retry and
resume command. Engineering/build work remains separately recorded. The
existing one-process, monitored 4 GiB RSS target and 4 GiB cumulative additional
storage allocation are inherited, not reset for schema `/5`.

| Evidence item | Status at record assembly |
| --- | --- |
| New artifact/source/configuration/supervision identities | Both fits bound in compact evidence |
| Matched corrected-source `/4`–`/5` fit and generated behavior | Both `/5` variants measured negative below |
| Same-artifact response-state-disabled and geometry controls | All six arms per `/5` variant recorded below |
| Actual prose responses, Rust source, compilation and execution | Saved reports plus two separate inspected-source assertion failures |
| Causal state, resume, serialization, allocation and caller checks | 48 core, 15 CLI/service, three context, eight probe and two allocation/source tests pass |
| Cumulative model time, measured storage and artifact paths | Final replay: 884.609/1,800 seconds; sampled retained footprint 4,038,156,288 bytes including source/metadata reserve |
| Protected PR and actual queue/CI steps | Protected delivery required; live PR/issue status and local `protected-delivery.json` own the final transport result |

Append the measured outcome with exact artifacts and commands. Preserve
positive, negative and unavailable results separately. The architecture's
priority does not establish a predictive increment; useful conversation and
coding/reasoning still require their own demonstrated behavior.

## First measured `/5` result — retained negative

The initial fit uses the frozen endpoint and feature layout
`persistent-query-local-paths-and-model-selected-occurrence-continuation/1`.
Its artifact is
`blake3:55554a68702602bf11aa2628b47f111065aae81483a021c0b5579ec80a6d7b70`.
The artifact, `response-fit-report.json`, `response-evaluation/report.json`
and read-only extraction `response-comparison.json` are retained under
`.uor-models/native-response-state-2026-09-05/`; the artifact filename is
`response.json`. The source identity remains
`848a5f8f9c8b4d4ceb5d0d0c1583886b5ab90f41fdb9a97a9327b9d288ac16b1`, matching
the corrected-source `/4` evaluation.

| Full-path development measurement | Prior `/4` | Initial `/5` |
| --- | ---: | ---: |
| Prose first correct | 20/32 | 20/32 |
| Prose exact generated completion | 6/32 | 5/32 |
| Prose teacher-forced correct | 195/320 | 187/320 |
| Rust first correct | 0/32 | 0/32 |
| Rust exact generated completion | 0/32 | 0/32 |
| Rust teacher-forced correct | 328/504 | 291/504 |

The same-artifact response-state-disabled arm reached 0/32 exact prose,
136/320 prose teacher-forced correct and 231/504 Rust teacher-forced correct.
That intervention establishes sensitivity to this fitted response state; it
does not erase the regression against `/4`. Two of eight sampled Rust programs
compiled, while all eight executions remained `NOT_RUN` because the generated
source was not the exact authored safe source.

The separate `reviewed-rust-execution.json` subsequently records actual
execution of the two inspected generated programs. Both compiled and both
failed their assertions with exit status 101. In `rust/100004`, Rust evaluates
`ada = 81 + 5` to 86 before `finn` changes, while the generated assertion
expects 92. In `rust/100005`, `raise(twice(9))` evaluates to 23 while the
generated assertion expects 120. Their source SHA-256 identities and exact
source text are retained in the compact evidence. These are two measured
runtime failures, separate from the unchanged probe `NOT_RUN` rows; the
reviewed scope does not extend to arbitrary generated Rust.

To localize the regression, match `full`-arm cases by ID and their
`completion_targets` by index, verifying equal target token IDs. A lost
correct position has `/4 predicted == target` and `/5 predicted != target`.
There are 15 prose losses and 7 gains, plus 46 Rust losses and 9 gains. Of
those 61 losses, 60 have `/5 target_in_shortlist == true`: 15/15 prose and
45/46 Rust. This means the target remains in the scored shortlist, potentially
through a base alternative. It does not mean a memory route to the target is
present. Most of the measured regression therefore occurs after shortlist
admission, without establishing the precise scoring cause.
`response-regression-diagnosis.json` retains the exact matching method, report
hashes and all 61 lost positions. The generated traces contain 1,003 requery,
1,695 base and 45 stop decisions, with zero selected continuation decisions.
That is the trace of this fitted artifact, distinct from whether the runtime
can exercise its continuation mechanism in a focused test.

## Optional advancing response endpoint — measured negative

The second bounded comparison keeps the captured query entries and posting
references but lets their endpoint advance through the actual observed H4
pose and full phase state. The opt-in `--advance-response-path` requires
`--persist-response`. It sets `MemoryReadFitConfig.advance_response_path=true`
and declares feature layout
`persistent-query-advancing-local-paths-and-model-selected-occurrence-continuation/2`.

This changes the query-cue-to-current-endpoint local relation used in requery
scoring after response observation. It preserves the cue/reference population,
candidate limit, one-successor bound, model-selected commitment and all
arithmetic restrictions. Observation advances the geometric endpoint;
prediction does not. The hypothesis is that geometric path evolution can
condition later choices while the response's source question stays available.
The first negative does not establish that hypothesis.

The default false value is omitted from serialized configuration, so the
initial `/5` law and artifact identity remain unchanged. Fitting checkpoints
and CLI envelopes bind the mode; changing it requires a new fit/output and
checkpoint path. Resume cannot reinterpret an earlier frozen-endpoint
checkpoint. Earlier artifacts remain preserved comparison inputs.

The resulting artifact is
`blake3:c4ade92fb1c9b6394be84a333524baef848e7bc767c2d55a949bd0e79129dbbc`,
saved as `advancing.json` beside `advancing-fit-report.json` and
`advancing-evaluation/report.json` in the same artifact directory. It uses the
same baseline, ordered fit source, supervision and schedule as the first `/5`
fit; the configuration adds only `advance_response_path=true`.

Training correct targets rose from 5,590/6,548 to 5,628/6,548, but development
prose teacher-forced correctness fell from 187/320 to 181/320 and Rust fell
from 291/504 to 280/504. Prose first predictions remain 20/32 and exact
responses remain 5/32. Rust first/exact responses remain 0/32; one of eight
sampled programs compiled. The probe again did not execute those nonmatching
programs under its authored-source rule.

| `/5` mode | Control | Prose first /32 | Prose exact /32 | Prose teacher-forced /320 | Rust teacher-forced /504 |
| --- | --- | ---: | ---: | ---: | ---: |
| Frozen endpoint | Full | 20 | 5 | 187 | 291 |
| Frozen endpoint | Response state disabled | 20 | 0 | 136 | 231 |
| Frozen endpoint | Memory disabled | 13 | 4 | 167 | 278 |
| Frozen endpoint | Geometry disabled | 21 | 4 | 168 | 244 |
| Frozen endpoint | H4 disabled | 21 | 4 | 175 | 270 |
| Frozen endpoint | Zeta disabled | 19 | 5 | 192 | 289 |
| Advancing endpoint | Full | 20 | 5 | 181 | 280 |
| Advancing endpoint | Response state disabled | 20 | 1 | 138 | 230 |
| Advancing endpoint | Memory disabled | 13 | 4 | 167 | 278 |
| Advancing endpoint | Geometry disabled | 22 | 5 | 173 | 259 |
| Advancing endpoint | H4 disabled | 19 | 4 | 172 | 259 |
| Advancing endpoint | Zeta disabled | 20 | 5 | 185 | 278 |

All twelve arms have zero correct Rust first predictions and zero exact Rust
completions. The advancing full trace records one continuation, 693 requery,
1,722 base and 49 stop decisions. The single continuation occurs at recorded
decision 15 in `prose/100030`; a bounded twelve-decision window around it is
retained in the evidence. It does not produce an exact answer. The intended
path advance is implemented and measurable, but this fit does not establish a
quality gain.

Representative generated failures remain concrete. For the two-fact
`prose/100013`, the advancing model writes
` cyra has green.\nUser: What color is still green.\nUser: Correction: eli's badge is still red.\n`.
The contradiction case `prose/100002` writes
` No.\nUser:. Assistant: No.\nUser: Update:.\n`.
The source-bound compact evidence retains those exact texts plus updated-fact,
unsupported-answer and four Rust cases, with at most twelve actual decisions
per representative trace. The local full reports retain all cases.

Retain `/4` as the stronger matched artifact at this measured scope and retain
both `/5` variants as explicit development mechanisms with their negative
results. This does not demote geometric intelligence generally, qualify either
alpha group, or create a new global stop rule.

## Final verification and resource accounting

Pinned Rust 1.97.1 compiles the final release CLI and joint probe. The final
binary reproduces all three saved artifacts across 1,088 case/control runs
and 14,008 target decisions: family metrics, first predictions, every recorded
target decision, output bytes/tokens/stops and bounded response traces match.
This includes 320 `/4`, 384 initial `/5` and 384 advancing `/5` runs. These are
replays of the same finite development cases, not new independent capability
samples. The receipt is `final-preservation-comparison.json` under the local
artifact root; its source/report hashes are also in the committed evidence.

Focused verification passes 48 core and 15 CLI/service tests, three context
tests, eight joint-probe tests, and two source/allocation tests. The allocation
census executes eight model variants, ten controls and 1,024 decisions per
combination: 81,920 decisions with zero steady-state kernel allocations. It
includes actual model-selected response observations for both endpoint modes;
host tokenization, rendering, tracing and serialization are outside that
allocation claim. Formatting, current architecture policy and claim wording
pass. Broad release QA, Clippy and final held-out qualification were not run.
The CI workflow's four historical audit/fuzz/WASM/Gate-C acknowledgements are
not evidence that those checks execute.

The model ledger carries 784.729 inherited seconds plus 99.880 seconds of new
fitting, evaluation, inspected generated-code execution and saved-artifact
replay: **884.609/1,800 seconds**, leaving **915.391 seconds**. There is one
model process at a time and a monitored 4 GiB process-RSS target. The largest
sampled model-process RSS is 136,495,104 bytes; short commands without a sample
do not establish zero memory use. Engineering builds/tests are separately
recorded in `engineering-commands.jsonl` and their logs.

Storage retains the inherited 4 GiB allocation. A build was paused when a
deliberately conservative counter still included the old warm-cache charge.
Reconciliation removed only the obsolete 671,379,456-byte target-growth term
from the dated pre-cleanup estimate and then charged **all** current target
bytes, including preserved cold files, new artifact files, the measured
9,875,456-byte worktree creation and a 5 MiB source/metadata reserve. It credits
neither general free disk nor the broad clone passes and deletes nothing in
this continuation. The final sampled retained footprint is 4,038,156,288 bytes
of 4,294,967,296, leaving 256,811,008 bytes. Periodic `du`/RSS sampling is not an
exact peak or unique-extent guarantee; historical overages remain preserved.
Builds use the shared target, disabled incremental output and reduced debug
information, normally three compiler workers and one for the final release.

An intermediate service-test compile failed because a full configuration
literal lacked the new boolean field; that fixture was corrected before all
15 caller tests passed. Independent review also corrected short-context
snapshot validation, retained cue validation, fixed-population epoch choice
and the extra continuation alternative's training-capacity accounting. These
engineering corrections do not alter the two negative model outcomes.

## Remaining native implementation seam

Response composition remains unresolved by both fits. The independent
absent-value cause has a concrete compatible next mechanism: typed numeric
occurrences and a bounded result slot inside `MemoryState`, with learned
operand/action selection over their prime context, occurrence identity and
relative H4/phase state. Existing `ZPhi::checked_add` in
`crates/uor-r4-core/src/prime_route_attention.rs` supplies a fixed exact
coefficient law; the present token-root coefficient sum must not be
misinterpreted as the numeric value of the text. An artifact-bound numeral
codec can supply typed payloads, while learned selection must distinguish an
original operand from a later reassignment and choose whether to write/use
the result. The result must compete through ordinary generation and preserve
its derivation identity. This is not implemented or measured here. It would
be learned binding and value writing over fixed arithmetic, without a Rust
interpreter, task-specific answer parser or oracle source position. Adding
that operator alone would not repair the demonstrated syntax and response
composition failures.
