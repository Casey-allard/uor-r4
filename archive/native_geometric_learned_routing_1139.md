# First learned H4 routing block — #1139

This record follows the owner-directed geo-transformer plan delivered in #1146.
It concerns one optional residual block in the existing native predictor. It
does not replace the entire predictor or qualify a geo-transformer LLM.

## Mechanism and retained information

Each of two independent channels learns query, key and value codes for tokens
in the existing artifact vocabulary. Codes identify full signed elements of the
120-element binary icosahedral group. The channel composes the last two query
codes in their observed order. It examines at most eight recent source keys,
selects one source, gathers that source's value code and applies a learned
query-conditioned group action. All group products read the existing exact H4
product table. Sparse output rows contribute quantized residual token scores
and at most eight candidate postings per channel to `Model::predict`.

Angular selection ranks the signed real component of `query * inverse(key)`.
Host construction orders the exact `Z[phi]` coefficients and emits a finite
rank table. Serving reads that table; it computes no cosine, floating-point
distance, multiplication or dense projection. Ties prefer the most recent
source. The full selected key/value and transported output retain orientation;
the scalar angular comparison alone does not distinguish every orientation.
The exact-code comparator selects code equality, with the same code capacity,
context limit, transport and readout. It is a selector comparison, not a wholly
non-geometric model. The fixed-placement arm learns actions and emission rows
while retaining the initialized token codes.

Token identity remains the parent's lexical/byte codec and prime-addressed
vocabulary. The new codebook does not replace canonical identity with a semantic
hash. Query/key/value placement is learned from next-token loss. The fixed zeta
phases, ordered base features, paired-H4/icosian witnesses, exact relation store,
typed arithmetic and committed copy path remain in their existing roles; this
block does not establish a new learned zeta or paired-H4 scoring advantage.
Committed exact copying still bypasses ordinary prediction work.

The eight-token window retains exact source token IDs until selection. Each
channel then discards all unselected payloads. Identical token codes, angular
ties and finite group products can collide. The two-token query is compressed
to 120 states, and each selected/transported output is also one of 120 states.
The readout sums independent channel scores, without a joint relation between
the channels. Source offset is diagnostic; it is not an extra learned feature.
The block does not encode older context, source revisions, or a persistent
derived-value chain. Existing bounded memory remains separate.

## Learning and comparison

Rust fitting uses hard coordinate proposals evaluated by the actual serving
selector and transport. For each channel, it adjusts query/key/value codes and
query-conditioned actions, refitting route-conditional token counts after each
proposal. The accepted objective is smoothed conditional next-token NLL for
that channel. It is not end-to-end optimization of the parent's complete
residual score or a soft attention surrogate. Quantized sparse emission scores
are compiled from the final counts. No dense training library is needed here.

The bounded example uses 24 authored raw-text documents, equally split between
prose and Rust, and eight distinct development documents. It samples up to
2,048 fit positions across complete documents, adjusts the 24 most frequent
context token codes, and makes two passes with three proposals per coordinate.
Each fit has a 15-second safety ceiling. Three arms share these limits and the
initialization seed; a stopped fit must not be described as a completed matched
search. Development text and generated prompts are OPEN, synthetic and small.
They are not broad natural-language or sealed capability qualification.

The unchanged parent and three fitted arms receive the same next-token and
24-token generation checks. The angular artifact also runs with the block
disabled, source selection forced to the most recent token, and the learned
action replaced by identity. Selection-disabled still calculates all source
ranks. Source hashes, fit reports, exact outputs, route examples and complete
native work counters are retained. Artifact reload and session restoration are
checked separately. No teacher or provider authors serving output.

The routing block is fitted last and binds the exact parent artifact CID.
Loading validates inherited response components against that unchanged parent,
including their original nested identity checks, then validates the complete
new artifact. Replacing or training a nested component under an already fitted
routing block is not supported by this version. The first execution exposed a
missing distinction between the new outer artifact and those older frozen
parents: its reload failed after the first arm. Those diagnostic outputs and
the failed 4.124-second command remain preserved. The correction adds explicit
parent binding without changing learned route/scoring equations or weakening
nested provenance checks. Final results must come from the corrected executable.

## Cost boundary

For vocabulary size V and up to W=8 sources, the new codebooks contain 6V u16
entries, two 120-entry action tables and one 120-entry angular-rank table.
Emission storage depends on observed token/root pairs; output postings are
bounded by 2 * 120 * 8 token IDs. Existing shared H4 tables are reused. There
is no online training, new persistent per-token cache, or vocabulary-wide scan.

Each active prediction builds two ordered queries, examines at most 16 keys,
gathers exactly two value codes and executes two query-conditioned actions.
Each angular source requires an inverse, group-product and rank lookup. Each
selected payload needs two further group products. Candidate scoring adds two
bounded sparse-row searches per candidate; admission adds at most 16 postings.
Thus selection/transport is O(W), and residual readout is O(A log V) for the
number A of candidate evaluations during finite posting admission. A is not
the final shortlist size: a dropped candidate can be offered and scored again.
The existing feature, memory, write and
decoding work remains and must be included in whole-path timing.

`RoutingWork` reports code/table reads, comparisons, examined sources, payload
gathers, executed actions, emission queries and logical operand bytes. These
are algorithmic counters, not a complete CPU instruction or physical DRAM
traffic census. The report retains the surrounding native counters as well.
Host tokenization, artifact loading/validation and sparse-table construction
remain outside the allocation-free prediction kernel. Existing startup geometry
validation still uses floating point; whole-lifecycle integer-only execution
is not claimed.

The complete pre-execution projection is retained in
`.uor-models/native-typed-value-2026-09-05/learned-routing-projection.json`:
950 seconds engineering and 105 seconds model work, within cycle ceilings of
1,200 and 120 seconds. The cumulative ledger starts at 1,363.884/1,800 seconds.
Projected new storage is 288 MiB with approximately 307 MiB before the existing
tighter stop. The 4 GiB sampled child-RSS target and one model process remain.
Necessary storage increases are preauthorized; no material is deleted.

## Executed result and decision

**Keep this block as development code and artifacts. Retain parent `067adbf0`
for the accepted bounded behavior.** The complete comparison executes from the
corrected release binary. All three fits complete their configured passes;
angular and equality each evaluate 2,304 proposals, fixed placement 1,440.
Actual sampled fit positions are 1,890 from 24 documents. Angular accepts
32 query-code, 25 key-code, 28 value-code and 198 action changes across the two
channels. Its per-channel fit NLL changes from 4.756/4.740 to 4.465/4.402.
Only the 24 most frequent context token codes are eligible for adjustment in
this first fit; the vocabulary contains 611 token IDs.

| Arm | Prose next-token correct | Rust next-token correct | Total | Target continuations |
|---|---:|---:|---:|---:|
| Unchanged parent | 6/435 | 39/300 | 45/735 | 0/8 |
| Learned angular | 103/435 | 79/300 | 182/735 | 0/8 |
| Learned exact-code selection | 127/435 | 96/300 | 223/735 | 0/8 |
| Fixed placement, fitted actions/readout | 103/435 | 74/300 | 177/735 | 0/8 |
| Angular, source selection disabled | 43/435 | 42/300 | 85/735 | 0/8 |
| Angular, learned action disabled | 66/435 | 53/300 | 119/735 | 0/8 |

Disabling the whole new block reproduces the parent's 45/735 result and eight
generated byte sequences. The new representation/selection/action combination
helps this token-prediction population; the learned angular placement increment
over fixed placement is only five decisions. Exact-code selection performs
better under the matched placement-search budget. No angular superiority,
general semantic codebook or generation qualification follows. Prose contains
383 byte-fallback tokens and only 48 lexical tokens; Rust contains 176 and 120,
respectively, plus four EOS targets per family. These are substantially
byte-level results on authored synthetic text.

Both fitted selectors preserve only **38/62** previously correct responses.
The old parent reproduces all **62/62 complete Generation objects**, including
work and state, on the changed executable. On 28 earlier longer-context relation
cases, both candidates retain all 28 exact write sequences and all 28 restored
relation states. Angular returns 16/28 exact answers, equality 28/28. The
angular failures include correct `Unknown` prefixes followed by unwanted text.
This is output/termination regression despite correct retained facts.

The actual CLI matches angular's probe output for the Rust continuation:
` Unknown.\nss  (r (leftl a (r (l`. Disabling the existing response-entry
head removes the forced prefix but yields ` a)rt: i32] n) )c ))l3e i32eo i32`,
still incoherent. The parent under that same control produces
`\n    let shifted =);\n}\n`. Inspection and the trace explain the prefix:
the older entry head adds its positive margin above the base candidate's score.
Separately, residual language scores disrupt the old stopping behavior. The
block's per-channel conditional objective does not optimize either full-path
competition or final decoded continuation.

**Next within #1139:** train response dispatch and EOS together with the routed
predictor against final output, including ordinary prose/Rust continuations and
the preserved memory cases. Keep the selector comparison. Establish a useful
continuation and preserved termination before increasing context, adding more
channels or introducing another abstraction stage. The immediate problem is
not missing cache capacity. #1139 stays open; #1140 remains subsequent. No
larger language or frontier claim is admitted by this checkpoint.

## Measured costs, artifacts and verification

The angular artifact is `09f9991cf769e3008d78a487c5115502f06cbca681577a81278401714a8edc2b`
(10,807,631 bytes, 79,348 bytes above parent). Equality is
`c6d3739db83e60f4e0f5d1ed25e261fb205fbca784fbf2bdfc3981aaebea85ad`
(10,803,043 bytes); fixed placement is
`f7702f024ad505b2e1c08931907e94d49942aff4f17ec50d5e009087183e15e5`
(10,812,953 bytes). Each has the same 8,052 bytes of u16 token/action/rank
entries before container metadata. Angular adds 1,698 sparse emission scores
and 1,268 postings; equality has 1,538/1,224, fixed placement 1,872/1,407.
Prediction adds fixed inline decision/counter state and a 32-byte stack window;
the focused allocation census measures zero heap allocations and zero bytes
through repeated observation, selection and transformation.

The 735 angular development positions examine 11,312 source keys, gather
1,470 payloads and execute 1,470 actions. Residual scoring performs 284,066
emission queries across 142,033 candidate evaluations, with 1,321,154 new
comparisons and 7,756,832 logical operand bytes. The surrounding base still
performs 3,364,474 feature-score lookups. Selected transport is a small part of
this complete path; scoring candidates repeatedly remains substantial work.

Single-pass next-token timing (encoding, observation and full prediction) is
87.881 ms parent, 102.005 ms angular, 101.193 ms equality and 101.079 ms fixed
placement. These are diagnostics, not repeated speed measurements. Parent file
read/deserialization/validation is 2,497.554 ms; new in-memory artifact
deserialization/validation is 2,830.333–2,845.592 ms. These startup boundaries
differ by file reading and are stated separately. Fits take 531, 470 and
471 ms. The full comparison, including three fitted artifacts and their reloads,
takes 13.546 seconds under the monitor. Its peak sampled child RSS is
834,306,048 bytes; this includes fitting and validation clones, not a minimal
single-session serving footprint.

Four focused learned-routing unit tests, the new allocation test and the
integer-kernel source scan pass. They cover hard source choice, query action,
code bounds, nested parent identity refusal, fitting-data overlap refusal,
counter aggregation/saturation, disabled-parent preservation, artifact reload,
retained tail and session checkpoint replay. The architecture-policy check
passes. The changed release CLI and both examples compile and execute. New
generated Rust is not compiled or executed as semantic evidence; it has already
failed the target-continuation decision. Existing checked Rust evidence keeps
its original scope.

The [compact evidence](evidence/native_geometric_learned_routing_1139.json)
binds the complete report, exact outputs and controls. Local receipts remain
under `.uor-models/native-typed-value-2026-09-05/`: the initial failed
`learned-routing-comparison/`, corrected `learned-routing-corrected-comparison/`,
three `learned-routing-preserve-*` directories, two relation reports, three CLI
logs and command ledgers. The correction preserves every learned parameter and
configuration from the first angular fit, adding only parent provenance.

Model work including the failed attempt and focused executions is
**46.768/120 seconds** for this cycle. The cumulative ledger is
**1,410.652/1,800 seconds**, leaving **389.348 seconds**. Engineering through
the corrected build and architecture check is 862.744/1,200 seconds; final
formatting/claim checks and delivery receipts are recorded separately. Storage
after evaluation is 5,852,442,624 bytes against the unchanged 6,459,228,160 cap,
with 251,379,712 bytes before the tighter existing stop. No storage increase,
deletion, external model or paid compute was needed. Samples are not exact
transient peaks or unique-extent measurements.

## Recurrent revision /2 — owner adoption and first build, 2026-09-06

The owner adopted the [dependent-attention plan](https://github.com/UOR-Foundation/uor-r4/blob/bc03f2d7ffde99608da370808eca542360e54508/docs/integration/project-track.md#immediate-build-sequence)
after the preceding independent-read result. This section appends a new
mechanism and measurement; it does not replace the /1 negative above.

### Implemented representation and output path

`learned-h4-routing/2` uses the same two heads, learned token Q/K/V codes,
query-conditioned actions, signed H4 product/inverse tables and eight newest
token sources. The first transported output left-acts on the second contextual
query before its source selection. Both decisions retain selected token IDs,
source offsets, keys, values, actions and signed output roots. Repeated predict
recomputes the bounded chain without committing state; observe remains the
causal session transition. The intermediate result lives within one prediction,
not in a new persistent derived-value store.

The joint output reads five factorized features: first output, second output,
ordered relative output `second * inverse(first)`, the assembled parent's
winning token, and three existing response-state flags (boundary present,
entry active, copy complete). It learns Base versus Emit(token), with EOS as
an ordinary learned output choice. Base retains the complete parent's winner;
Emit competes after typed, completion, entry and copy decisions and passes
through their existing selected/observed commitment rules. BOS is only the
internal Base action and cannot be emitted. Immutable committed-copy bytes
retain their existing early dispatch.

This removes /1's independent residual scores from ordinary candidate scoring.
It does not remove the inherited predictor's work. The model still executes
its prime/zeta/R4 features, learned occurrence/role heads, exact relations and
typed arithmetic. No new dense projection, transformer, provider or serving
matrix product is introduced. Startup validation remains a distinct host path.

The two-token initial query is still compressed to 120 states. Each read can
select only one of eight recent tokens; earlier values are accessible only to
the inherited components, not these new heads. Exact source metadata survives
in the decision, but only the five named features affect the shared output.
Unselected payloads, fine orientation distinctions tied by scalar angular rank,
and distinctions colliding in the finite products can still be lost. This is
not multiscale abstraction or general typed multi-operation reasoning.

### Learning, comparison and complete cost

The Rust fitter mixes authored raw prose/Rust with complete prompt/response
examples, including sampled EOS positions. It reserves half the 1,024-position
budget for responses, takes every sixteenth construction example from the
existing 480-row memory fit source, and adds 24 language/dependent-task examples.
The earlier 62-case development set and eight generation prompts remain OPEN
evaluation. They are not fitted, newly independent or sealed.

Hard coordinate proposals change Q/K/V placement and actions in both heads.
Each proposal fits a shared additive classifier with four deterministic
perceptron passes, then evaluates the exported bounded integer Base/Emit/EOS
choice. The offline classifier uses observed target classes; serving receives
only the sparse compiled postings and scores. Selection prefers more correct output tokens,
then lower normalized action NLL. The denominator contains actual bounded
serving candidates; an unavailable target incurs the declared 20-nat floor
relative to the maximum candidate logit. This fitting criterion is not a
whole-sequence generation loss.

Parent teacher-forced histories are cached once for proposal efficiency.
Changed dispatch can change pending commitments, so the final candidate also
runs actual teacher forcing and reports both correct tokens and agreement
with cached-history output. That check measures the approximation instead of
assuming cached parent state equals candidate state. Free generated training
responses and OPEN development outputs are reported separately.

Angular and exact-code arms have the same initialization, 12 adjustable token
codes, one pass, three proposals per coordinate, two heads and readout capacity.
Each fit has a 30-second safety ceiling; unequal incomplete searches cannot
establish matched selector advantage. The intermediate-disabled control replaces
the carried first output with identity while retaining the transport operation
and source scans. Action-disabled and whole-block-disabled controls remain.

Each non-copy prediction adds two bounded reads, one extra inter-read product
and one relative-output product. Five row searches collect at most eight
postings each, plus Base/EOS, into 42 fixed stack slots. Every offered action is
deduplicated and scored against the same five rows and action priors. Report
row/token comparisons, table reads, logical operand bytes, gathers and actions
alongside the parent's encoding, feature/candidate work, writes and output.
Counters describe algorithmic work, not CPU instructions or physical traffic.
Artifact size, compilation, fitting, loading and complete timings remain part
of the measurement. No heap growth is intended in prediction.

The complete pre-execution projection is retained locally as
`recurrent-routing-projection.json`: 110/120 seconds model work,
1,000/1,200 seconds engineering and 224 MiB retained storage growth. It starts
from cumulative 1,410.652/1,800 seconds and 240 MiB before the tighter storage
stop, with one process, a 4 GiB RSS target and the 128 MiB margin. No new storage
authorization, deletion or paid compute is needed by that projection.

The first local attempt instead fitted conditional count rows. It reached
561/796 cached-history correct tokens but only 534/796 on actual candidate
teacher forcing, with 766/796 cached/actual output agreement. It preserved
0/62 development responses. Its later replay assertion compared `Full` with
`LearnedRoutingDisabled` without accounting for the declared `state.control`
label and stopped before the equality arm. This is a preserved model negative
plus a separate harness error; it is not a matched selector result.

The correction replaces count residuals with directly fitted additive scores.
`additive_scores: true` binds that interpretation in the artifact. Missing/false
retains the first conditional artifact's original arithmetic and identity.
The corrected replay retains the raw disabled report and compares every
Generation field after normalizing only `generation.state.control`. It does
not erase the intervention label or normalize work, outputs, scores or traces.
All first-attempt source files, model, outputs and its 9.183-second charge remain.
The correction projection fits within the original cycle: 510 seconds remaining
engineering, 95 seconds model work and 160 MiB storage; no fresh allocation.

### Dependent-read result — 2026-09-06

**Decision: `RETAIN_PARENT_REVISE_SOURCE_ACCESS`.** Both corrected fits complete
all 936 proposals. The shared output improves fitting but fails generated
language and preservation. Keep `067adbf0`; neither new artifact is promoted.
[Compact evidence](evidence/native_geometric_recurrent_routing_1139.json) binds
the source, artifacts, comparisons, work and local receipts. The source remains
OPEN development: 24 raw documents and 54 construction responses; 796 sampled
positions include 284 response positions, with 71 committed-copy positions
excluded from fitting because their serving dispatch bypasses this block.

| Arm | OPEN prose /435 | OPEN Rust /300 | Earlier responses /62 | Target continuations /8 |
|---|---:|---:|---:|---:|
| Accepted parent | 6 | 39 | 62 | 0 |
| Recurrent angular | 47 | 73 | 6 | 0 |
| Recurrent exact-code | 64 | 80 | 0 | 0 |
| Angular, intermediate connection disabled | 29 | 24 | NOT_RUN | 0 |
| Angular, selected action disabled | 47 | 71 | NOT_RUN | 0 |
| Whole new block disabled | 6 | 39 | 62 | 0 |

Angular cached-history fitting rises from 539 to 594/796 correct, with actual
candidate teacher forcing at 576/796 and cached/actual output agreement at
776/796. Exact-code fitting rises from 342 to 549, actual at 542 and agreement
789/796. Fit response generation is only 15/54 angular and 22/54 exact-code.
The objective prioritizes correct actions before NLL: exact-code NLL worsens
from 2.618 to 2.795 while correct actions increase. Neither this objective nor
cached-parent histories are a free-generation optimization guarantee.

The first-to-second connection affects token prediction: disabling it reduces
angular correct tokens from 120 to 53. Disabling the learned action reduces
120 to 118. These are sensitivity results; exact-code selection still reaches
144 and neither arm produces useful complete outputs. In particular:

- `Start with five. Add two, then double the result. Answer:` produces `Lima`
  in both arms, as does the owner-of-the-red-box question whose target is Rome.
- The Rust `sum` prefix produces ` left naeerls` angular and ` value > 0 }`
  exact-code. Appending each exact generated continuation to its prefix and an
  assertion caller fails `rustc --edition=2021`; semantics remain
  `NOT_RUN_COMPILE_FAILED`. No generated text was repaired.
- An older exact numeric copy changes from `73.\n` to `26.\n` under angular.
  The accepted parent reproduces all 62 earlier answers. Disabled-block replay
  matches all fields of all 62 Generation objects after normalizing only the
  declared control label, with raw reports retained.

The actual `r4 geometric generate` result matches the complete saved angular
Generation on the Rust prefix, including bytes, token IDs, work and state. The
CLI's four additional metadata fields are kept separate, and artifact identity
is checked. The old conditional artifact also reproduces its original complete
Generation after the correction, confirming the default-false interpretation
at this one-prompt scope. Large JSON integers are compared losslessly.

Corrected angular artifact `cd39e57d2a70e441d1f225faa60c4873bdd037012a229d0b200dbd152a694b20`
is 10,911,432 bytes; exact-code `503f9241ef2365f535b1b64f11098c5766470477d73c3b3d01bfeda6dea717cb`
is 10,894,855 bytes. Block sizes are 183,130 and 166,553 bytes. The prior parent
is 10,728,283 bytes. Training scratch is offline; serving adds immutable rows,
fixed stack candidates and no new persistent value store. No online parameter
update or index-building phase is introduced. Snapshot format is unchanged.

Across all 735 teacher-forced positions, each arm examines 11,312 source keys,
gathers 1,470 payloads and executes 1,470 selected operators. Angular additionally
records 79,631 emission queries, 702,975 comparisons, 169,807 table reads and
2,511,630 logical operand bytes. Exact-code records 79,903 queries, 694,848
comparisons, 124,327 reads and 2,345,910 bytes. The connection-disabled control
retains the same source/gather/operator work; complete readout costs vary with
the selected states. The inherited predictor still performs 2,200,949 base
score lookups plus 629,273 memory score lookups, with its full state/update work
included in the evidence. This is an added development block, not elimination
of the inherited predictor's work.

Single warm whole-next-token samples are 87.918 ms parent, 93.090 ms angular
and 91.388 ms exact-code. The 62 complete response samples take 74.772,
95.157 and 106.915 ms respectively, with different failed output lengths.
Corrected CLI command wall is 3.166 seconds including loading, validation,
encoding, generation and JSON output. Individual load/encoding phase timers
and physical-memory traffic were not measured. No speedup follows from these
single samples. Startup geometry validation still uses floating point; the
changed prediction kernel uses integer/table operations and no matrix products.

Final corrected release, two routing allocation tests and the forbidden
arithmetic/float source scan pass. Three focused unit tests passed before the
additive-score correction; final units are assigned to the protected native CI
job. The comparison and CLI exercise the corrected artifact and output path.
New relation-transfer evaluation stops after the preservation failure; it is
not inferred from the older unchanged-writer result. Final held-out evaluation
is `NOT_RUN`.

Total cycle model work through failed attempt, correction, CLI/replay and
generated-code compilation is 32.720/120 seconds. Cumulative use is
1,443.372/1,800 seconds, leaving 356.628. Engineering through the corrected
kernel check is 1,128.504/1,200 seconds; final checks/delivery are appended to
the existing engineering receipts. Highest sampled model child RSS is
817,659,904 bytes. The last model-command storage sample is 5,896,962,048 bytes,
leaving 206,860,288 before the tighter existing stop. The 128 MiB margin and
6,459,228,160-byte cap remain unchanged; no storage increase or deletion.

**Next intervention:** reuse the existing exact word/relation references and
candidate-relative role features as sources for learned routing, with selected
copy/value operators. The new block currently compresses a two-token query to
one H4 root and scans eight recent raw tokens, often bytes. Earlier facts can
survive elsewhere in the model yet remain inaccessible to these reads. Its
shared classifier also emits learned answer tokens rather than consuming an
exact source pointer. Those are explicit information/access limitations; the
experiment does not isolate them as the only causes of poor language behavior.
Use the accepted role-reader as the working source/NoRead comparator; train the
new geometric selector over that existing bounded population, retain exact
payload identity, and test changed names/values plus preservation. Do not refit
the accepted reader or repeat its completed handoff. Add a second
dependent source read only after the first is useful. A wider beam, another
metric, repeated score tuning or a replacement memory framework is not justified
by the present negative alone. #1139 remains open and #1140 stays subsequent.

### Source-access localization — same artifact, 2026-09-06

A small Rust host linked to the unchanged release library executes just the
first response decision for the existing arithmetic, relation and color prompts.
All three have the identical newest-first eight-token sequence
`[594,116,103,121,117,112,67,34]`, parent token 432 and flags 1.
Their complete two-head decisions are identical: first query 33, source offset
0, output 61; second query 48, source offset 3, output 25. Thus every input
to the five-feature shared output is identical, although required first outputs
for `14`, `Rome` and `blue` differ. All predict token 78.

This is a concrete first-token feature collision, not merely a suspicion from
identical final text. Reweighting that same input or changing routing parameters
while retaining the same source/query interface cannot distinguish these three
prompts. It does not establish that the proposed source-access correction is
sufficient, nor diagnose every preservation/language failure. The source and
exact output are embedded in the evidence, with library/executable hashes and
local files retained. No training or new benchmark framework was added.

The complete supplemental projection was 15 seconds engineering, 10 seconds
model work and 16 MiB storage within the existing cycle. Actual link work is
0.574 seconds and model command wall 3.124 seconds. Cycle model use becomes
35.844 seconds; cumulative 1,446.496/1,800 leaves 353.504. Storage becomes
5,899,120,640 bytes, still below the existing tighter stop. The final-source
native fitting/artifact/runtime CI step passed in run 34012685323 at Rust
commit c36720f9; this supplemental host changes no product source.

The complete native CI job in run 34012685323 subsequently passed: 124 native
unit tests (including all three recurrent tests), 3 context tests, 8 allocation
tests and 20 CLI tests, with zero failures or ignored tests. Its four other
required status names acknowledged compatibility only; no audit, fuzz, WASM
or Gate C execution is inferred. Rust source remains c36720f9.
