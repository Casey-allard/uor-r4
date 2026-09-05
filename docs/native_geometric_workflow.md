# Native geometric development workflow

The native path is `r4 geometric`. Data preparation, fitting, artifacts,
evaluation, sessions and generation use Rust. Its initial learner estimates
conditional score tables over prime lexical identities and geometric context.
Separate readout fitting learns bounded integer gates over seven feature groups
and sufficiently supported last-prime query keys. It does not yet learn the
ordinary token memory-write law. The optional typed-value component separately
learns operand/action selection and whether to write its result.
An optional learned memory reader retrieves actual retained token values through
prime-addressed postings, query/source offsets and relative H4/zeta features.
It is versioned separately in the artifact; the original fixed/readout-only
artifacts remain loadable.
It is a foundation for the [project plan](integration/project-track.md), with
language, memory, reasoning and coding quality measured separately. Historical
Python and floating-point reference commands retain their explicit names.

Build the CLI with `cargo build --release --bin r4`. The examples below use
`target/release/r4`; set a different path when using a shared Cargo target.

## Prepare open development data

The optional typed-value experiment is available through the same core
`Model::fit_values` API and a bounded Rust example:

```sh
cargo build --release -p uor-r4-wasm-router --bin r4 -p uor-r4-core --example native_geometric_value_probe
target/release/examples/native_geometric_value_probe fit \
  --model /absolute/path/to/occurrence.json \
  --output-dir /absolute/path/to/new-value-run \
  --epochs 24 --learning-rate 0.1 --max-features 65536 \
  --generated-tokens 32 --max-seconds 120
```

Use the inherited cumulative model/resource monitor around this command.
Its inner time check is between bounded operations, not a hard interruption.
The example writes raw fit/development source and name-swap controls before
fitting, reloads the exported artifact, and saves actual output from Full,
ValuesDisabled, H4Disabled and ZetaDisabled arms. It refuses an existing output
directory and checks development against the artifact's training receipts.
`evaluate --model ... --source ... --output-dir ...` replays a saved artifact.
Generated Rust is saved unchanged; compilation/execution is a separate inspected
step. This example uses numeral-first targets with exact decoded-byte metrics,
not canonical first-token equivalence. See the
[typed-value record](native_geometric_typed_value_973.md) for measured limits.

`--lexeme-cues true` explicitly selects `Model::fit_values_with_lexeme_cues`
and typed schema `/2`. It adds bounded whole-word identity cues and 64 raw
construction name swaps to the original 128 fitting pairs. The same development
cases are reused open feedback. The additional `ValueLexemesDisabled` arm removes
only the new learned feature rows while retaining their state and ingestion cost.
The default remains `/1`; source schema and this flag must agree on replay.

The resulting model loads through ordinary `r4 geometric` generation and the
native service. `--control values-disabled` suppresses typed candidates;
`--control value-lexemes-disabled` suppresses `/2` whole-word scoring alone. Session
schema `/3` preserves committed derived values and emission cursor, including
empty HTTP continuation after export/import. `Session::begin_response` starts
new value selection; a continuing response must retain its active state.

The general corpus preparation command remains:

```sh
target/release/r4 geometric prepare \
  --input /absolute/path/to/public-corpus.txt \
  --output-directory .uor-models/native-development/corpus \
  --max-input-bytes 1048576 --document-bytes 4096 --readout-split
```

The reader takes a declared source prefix, splits UTF-8 text into bounded
documents, removes exact duplicate document contents, and puts every fifth
unique document into open development. JSONL input instead accepts records
with `id` and `text`. Multiple `--input` paths share the byte budget in supplied
order. The manifest reports truncation, byte digests and split sizes. Adjacent
chunks can share a story or topic; this split is for development, not a claim
of independent final held-out performance. Use separately sourced tasks for
the final assessment after design selection.

With `--readout-split`, the preceding document is reserved in `readout.jsonl`
for learning geometric readout gates. It is separate from both count fitting
and open development. Without this option, preparation makes a two-way split.

## Fit, save and resume

```sh
target/release/r4 geometric train \
  --input .uor-models/native-development/corpus/train.jsonl \
  --model .uor-models/native-development/model.json \
  --checkpoint .uor-models/native-development/training.checkpoint \
  --context 128 --candidates 32 --epochs 1 \
  --max-seconds 1200 --max-rss-mib 4096 \
  --max-output-bytes 536870912 --checkpoint-every 32 \
  --report .uor-models/native-development/training.json
```

These are example run settings, not global project limits. Context supports
1–4096 tokens; prompt length and output length are separate. Lexical vocabulary,
score rows, associations and candidate postings are configured capacities.
Capacity drops are reported as rejected feature events; a completed run does
not mean every proposed association fitted in storage.

Use the same command with `--resume` to continue from its document/epoch cursor.
The checkpoint requires the same construction bytes, order and configuration.
Increasing `--epochs` explicitly requests another presentation of the corpus;
reopening an already completed epoch does not train it twice. Snapshot and
artifact replacement are atomic per file. A previous checkpoint is retained
if a replacement exceeds its storage budget.

Elapsed time is cumulative across resumes. The CLI checks time and observed
process memory between bounded documents; it is not an OS-enforced memory or
wall-time sandbox. Checkpoint/compile finalization can pass the time boundary
and is reported. Include data preparation, evaluation and builds in the overall
experiment budget as well; separate CLI commands do not automatically share
an experiment-wide accounting file. Paid compute and major budget increases
still require owner authorization.

## Compare behavior and use the actual artifact

Optionally learn readout influence on the reserved fit partition while
preserving the fixed-readout artifact:

```sh
target/release/r4 geometric fit-readout \
  --model .uor-models/native-development/model.json \
  --input .uor-models/native-development/corpus/readout.jsonl \
  --output .uor-models/native-development/learned.json \
  --max-positions 4096 --epochs 8
```

The learned coefficients execute with bounded integer shift/add operations.
Fitting assigns equal document quotas and samples uniformly across each full
document while observing intervening context. Its report distinguishes sampled
positions from observed context and counts targets absent from the bounded
shortlist; absent targets are not inserted for fitting. Compare both artifacts
on the same separate development documents. Fitting metrics are training
evidence; they do not establish generalization.

Fit the optional memory reader from that same training-only readout partition:

```sh
target/release/r4 geometric fit-memory \
  --model .uor-models/native-development/learned.json \
  --input .uor-models/native-development/corpus/readout.jsonl \
  --output .uor-models/native-development/memory.json \
  --query-tokens 8 --source-offsets 4 --postings-per-address 4 \
  --candidates 128 --max-positions 4096 --epochs 8
```

The source/query bounds configure the actual memory read, independently of the
retained context size. Candidate values come from observed memory records;
fitting never inserts a target value that the candidate path did not expose.
The fitting sampler reserves a bounded document tail and spreads its remaining
quota across the body; the report records its exact recipe and exposure.
Fitting first teaches the reader among reachable retained values, calibrates a
shared bias on the same fit examples, then refines the actual maximum-route
prediction objective. `--epochs` applies to each of the two learning stages;
both stage counts and final quantized predictions are reported explicitly.
Count-construction overlap remains refused. Reusing the readout-fit partition
is allowed because both operations are training; subsequent development
evaluation excludes all training receipts. `--control memory-disabled` turns
off this optional reader on the same artifact. Other controls remove their
declared geometric contributions, not all prime-identity information.
Exact token cue identity is the default. The explicit `--word-cues` experiment
uses memory schema `/2` and compiles a separate mapping for case-sensitive word runs:
leading whitespace does not change their memory lookup identity. Exact output
tokens and their geometric primes remain distinct. Punctuation, raw-byte and
special tokens retain their original lookup identities. Loading reconstructs
and validates the mapping; schema `/1` artifacts retain their original behavior.
The equivalent library opt-in is `fit_memory_read_with_word_cues`; the regular
`fit_memory_read` preserves exact cues. Current results include regressions from
word equivalence, so its availability is not a recommendation to enable it.

`--query-context` selects the separate memory schema `/3`. It conditions the
reader on the exact ordered pair of recent token primes and on query distance,
source offset and newest-first posting rank. It keeps the existing H4,
orientation and fixed-zeta features. It can be combined with `--word-cues`;
neither option changes the default `/1` reader. The corresponding library entry
point is `fit_memory_read_with_query_context(documents, config, word_cues)`.
The fit report names the feature layout and objective. Historical reports that
predate layout metadata deserialize with empty layout/name fields; this means
unavailable metadata, not a claim about their feature layout.
For `/3`, the first fitting stage shares half the target learning signal
uniformly among all admitted routes carrying the correct token. This gives
initially weak routes a chance to learn; it uses no source-position oracle or
language grammar. The existing bias calibration and maximum-route refinement
follow. Between them, `/3` also calibrates the existing query-pair bias on that
query context's fit examples, so one shared bias cannot suppress every route
for a useful query type. This changes fitting only; inference reads the same
integer feature rows. Schemas `/1` and `/2` retain their original fitting objective.
Read query-bias loss metrics only when `query_bias_positions` is nonzero. Zero
counts/defaults in historical or non-`/3` reports do not represent a measured
zero-loss calibration run.

## Replay a broader memory fitting population

`fit-memory-stream` is an explicit resumable successor for the `/3` reader,
with an optional `/4` occurrence-composition mode described below.
The original `fit-memory` command and `/1`, `/2`, `/3` entry points retain their
one-shot behavior. A streaming run starts with a fitted baseline without a
memory head; subsequent launches restore its training checkpoint. Loading an
already fitted head into a new one-shot fit is not continuation.

```sh
target/release/r4 geometric fit-memory-stream \
  --model .uor-models/native-development/learned.json \
  --input .uor-models/native-development/corpus/readout.jsonl \
  --output .uor-models/native-development/stream-memory.json \
  --checkpoint .uor-models/native-development/memory.checkpoint \
  --total-positions 32768 --batch-positions 256 --epochs 8 \
  --query-tokens 8 --source-offsets 4 --postings-per-address 4 \
  --candidates 128 --max-features 262144 --word-cues \
  --max-seconds 600 --max-rss-mib 4096 --checkpoint-every 128 \
  --report .uor-models/native-development/memory-fit.json
```

Repeat the command with `--resume` and the same baseline, ordered source bytes,
fit configuration, cue/composition/response mode and schedule. `--max-batches N` deliberately ends a
launch at a resumable boundary. A completed resume preserves the artifact.
Changing total exposure or epoch count creates a different schedule, requiring
a new run; it does not silently reinterpret a checkpoint's cursor.

Add `--compose-occurrences` to select memory schema `/4`, including on resumed
launches. It compares each source-cue-to-value H4/phase path with the local
query-cue-to-current path and combines unique fitted feature addresses at the
same retained sequence position. The prior and shared bias apply once per
occurrence; different positions with the same token stay separate. The library
entry is `MemoryReadTrainer::new_with_occurrence_composition`. Without the flag,
stream fitting retains `/3`; existing `/1`–`/3` models retain their behavior.
This composes selection evidence, not new values or intermediate writes. See
the [measured occurrence-selection record](native_geometric_occurrence_selection_973.md)
and [native mechanism map](native_geometric_mechanism_map_973.md) for current
behavior, controls and additional bounded workspace/work costs.

The total is a distinct target budget, while the batch bounds live route
examples. Equal document quotas redistribute unused short-document capacity;
each quota reserves up to eight final positions and spreads its remainder
across the body. Every stage replays this same fixed population in source order.
The report records actual selected targets, per-document candidate and memory
reachability, feature drops, query coverage, live buffer peaks and replay work.
Unselected context observations and repeated epochs are not counted as new
supervised exposure. Source token storage has a separate bound.

Checkpoints retain floating-point weights, feature registry indices, the stage
and token cursor, best-epoch state and partial calibration sums. Source replay
reconstructs the current document's causal state after reload. Global and query
bias grids accumulate over the declared population with fixed stage weights;
choices are applied after the entire corresponding pass. Final artifact
construction quantizes the selected weights into the existing integer reader.
The baseline's context window and the inference admission/geometry rules remain
the ones used during fitting.

Time is cumulative across these checkpoint launches. Host checks occur between
bounded replay batches; loading, serialization and finalization can overrun a
boundary. Model/checkpoint size checks preserve prior files on refusal, and
atomic replacement requires temporary disk space. A separate experiment budget
must still include preparation, other fits, evaluation and retries. These flags
are not a hard OS process-tree memory/time sandbox.

For a smaller relevant evaluation, repeat `--control` on `geometric evaluate`,
for example `--control full --control memory-disabled`. Omitting it preserves
the existing nine-control comparison.

The `native_geometric_joint_probe` example supplies broader synthetic whole-task
development data and evaluates any saved native model on both prose and Rust:

```sh
target/release/examples/native_geometric_joint_probe prepare \
  --output-dir .uor-models/joint-composition/source \
  --construction-worlds 128 --fit-worlds 128 --development-worlds 16
target/release/examples/native_geometric_joint_probe evaluate \
  --source .uor-models/joint-composition/source/source.json \
  --model .uor-models/joint-composition/memory.json \
  --output-dir .uor-models/joint-composition/development \
  --generated-tokens 96 --compiler-cases 8 --repair-cases 2 --max-seconds 120
```

Use the emitted whole-document `construction.jsonl` and `fit.jsonl` with the
training commands (`--document-bytes 16384`). The source declares its synthetic
templates and distinct splits. Evaluation checks exact-text overlap and lexical
completion boundaries, records unmodified generated text, and separates first
tokens from complete answers. Requested compiler checks use exact generated
source; execution is restricted to exact matches of the authored safe arithmetic
fixtures. A failed compilation can supply real diagnostics for one separate
model repair attempt. This conservative execution rule is not a sandbox or a
general software-engineering benchmark. Both capability groups remain open
development, with final held-out assessment reserved for design selection.

Newly prepared source schema `/2` includes test inputs explicitly in function
prompts and rejects conflicting prompt/complete-target pairs. Historical `/1`
sources remain accepted with their original bytes and a visible legacy-conflict
audit. The function-prefix metric excludes the appended test program; it does
not replace full exact scoring, compilation or execution. Compare newly fitted
readers on the same source version, without treating `/1` and `/2` results as a
matched numerical comparison.

## Fit declared response spans

For authored response tasks, streaming fitting also accepts `--supervision`
with a `MemoryReadSupervision` JSON file. Its ordered document receipts and
full baseline identity bind the exact tokenizer and source. Each interval
indexes the tokens from `Model::encode(full_document_text)`, with inclusive
`start` and exclusive `end`; EOS occupies index `encode.len()`. BOS is not a
loss-bearing target. The same mask must be supplied on resume.

Every source token still updates the causal geometric memory. Only sampled
eligible targets register features or contribute to gradients, calibration and
epoch selection. The report records eligible and actually selected positions.
This is a data-specified training objective, not a response parser or inference
rule. The default remains whole-document supervision.

After preparing and fitting a joint readout, rerun the joint probe's `prepare`
with the same world counts and `--model /absolute/path/to/readout.json` into a
new output directory. It preserves the generated task texts and adds
`supervision.json` for each fit completion plus EOS, after checking the exact
prompt/full-text token boundary. Pass that file to `fit-memory-stream`; changing
the baseline or source requires a new mask.

## Fit and use persistent response state

Add `--persist-response` to `fit-memory-stream` to select the optional `/5`
reader. Both `--compose-occurrences` and `--supervision PATH` are required;
missing either is refused before fitting. The existing prepared source and
its exact response-plus-EOS token spans can be reused:

```sh
target/release/r4 geometric fit-memory-stream \
  --model .uor-models/native-development/learned.json \
  --input .uor-models/native-development/corpus/readout.jsonl \
  --output .uor-models/native-development/response.json \
  --checkpoint .uor-models/native-development/response.checkpoint \
  --supervision .uor-models/native-development/supervision.json \
  --compose-occurrences --persist-response --word-cues \
  --total-positions 32768 --batch-positions 256 --epochs 8 \
  --query-tokens 8 --source-offsets 4 --postings-per-address 4 \
  --candidates 128 --max-features 262144 \
  --max-seconds 600 --max-rss-mib 4096 --checkpoint-every 128 \
  --report .uor-models/native-development/response-fit.json
```

The supervision file must bind this exact baseline and ordered input source;
the path in the example is not created automatically by `fit-memory-stream`.
Choose new output/checkpoint paths for a new run. Resume with the same flags
and `--resume`; both the CLI envelope and learned checkpoint bind response
mode. `--max-batches` stops at a resumable boundary without silently restarting
learning.

The optional `--advance-response-path` additionally requires
`--persist-response`. It keeps the captured query tokens and source posting
references while using the evolving observed H4 pose and phase state as the
query-path endpoint. Its artifact declares feature layout
`persistent-query-advancing-local-paths-and-model-selected-occurrence-continuation/2`.
Without the flag, the response endpoint stays fixed at capture and the initial
`/5` law is unchanged. This option does not enlarge the routing or state bounds.
It is a separate fitting experiment: use new output/checkpoint/report paths,
then retain the exact flag on resume. Both the configuration and CLI envelope
bind it; an endpoint-mode mismatch is refused before fitting resumes. The
first frozen-endpoint `/5` fit regressed from 6/32 to 5/32 exact prose, with
Rust exact remaining 0/32. The evolving-endpoint fit retained 5/32 exact prose
and 0/32 exact Rust, while teacher-forced accuracy declined further. Both
remain explicit development options; the
[response-state record](native_geometric_response_state_973.md) preserves
their exact artifacts, controls and measured limitations.

The library constructor is `MemoryReadTrainer::new_with_response_state`.
It freezes a quantized model-selection policy for each source replay pass and
saves those rows with its optimizer state. Reconstructed prefixes use the
same policy even after optimizer weights have changed. All response positions
advance selection before teacher observation, including positions excluded
from sampled loss. An observed target never chooses its source occurrence.
Because the selected state can change candidate reachability, `/5` selects
epochs by fixed-population correct targets, then reachable targets, then lower
conditional cross-entropy. Its final metrics use the exported quantized policy.

`Model::generate`, `geometric generate`, chat and the native service begin the
response after prompt observation. Low-level callers use
`Session::begin_response(&model)` at that boundary, predict, then observe the
selected token. `Session::end_response(&model)` closes the previous query
before external input. Prediction only replaces a transient decision;
observation commits its selected occurrence. New `/5` generation observes EOS
to commit a stop without rendering it. Older artifacts retain their previous
scoring and EOS behavior.

The HTTP service treats an empty prompt as continuation of an active response
after a token/output limit; nonempty input starts a new query after closing
the prior response. Empty input after a stopped response captures a new
boundary. Committed query and selected-occurrence state are included in `/5`
session snapshots. Pending predictions are recomputed after loading. The core
response snapshot limit is 8 MiB; the service retains its separate 1 MiB
checkpoint storage/import bound and reports persistence refusal rather than
writing a checkpoint it cannot restore.

Generation JSON contains `response_trace` for active `/5` responses: at most
96 actual decisions, including EOS if reached within the bound. Each records
token, score, action, source sequence/slot when present and observation count.
These host diagnostics come from the same generation, without another rollout.
Use the joint probe to compare the same artifact with its response state
disabled:

```sh
target/release/examples/native_geometric_joint_probe evaluate \
  --source .uor-models/joint-composition/source/source.json \
  --model .uor-models/native-development/response.json \
  --output-dir .uor-models/native-development/response-development \
  --generated-tokens 96 --compiler-cases 8 --repair-cases 2 \
  --controls full,response-state-disabled --max-seconds 120
```

`response-state-disabled` requires a `/5` artifact in this probe. It retains
the learned tables while suppressing captured query/selected-read state;
it is not a separately fitted `/4` comparator. Explicit existing controls
remain available. Independent-document `geometric evaluate` does not know
response-span boundaries; use the joint probe or explicit session boundaries
to assess this mechanism. See the
[response-state record](native_geometric_response_state_973.md) for the exact
operator, remaining value-computation boundary and measurement status.

## Preserved finite reader probe

The optional `native_geometric_memory_probe` example compares physical value
retention, predicted values and paired value changes. `--memory-read` adds the
new reader, and `--family rust` represents the same controlled relationships as
Rust variable assignments and assertions. These are training/evaluation data
families; the model has no fact grammar or special handling for those objects,
strings or assertions. They are not general conversation or coding tests.
`--family mixed` trains one artifact on both families, using the configured
document and world counts once per family and reporting each family's results
separately. Use `--word-cues` only for the named equivalence experiment.

The probe can also assess a saved artifact against its preserved development
source without fitting again:

```sh
cargo build --release -p uor-r4-core --example native_geometric_memory_probe
target/release/examples/native_geometric_memory_probe \
  --model /absolute/path/to/memory.json \
  --source /absolute/path/to/source.json \
  --output-dir .uor-models/native-development/saved-model-evaluation \
  --generated-tokens 24 --max-seconds 600 \
  --compile-rust --compiler-cases 16
```

Supply `--model` and `--source` together. The source schema is the probe's
preserved `source.json`; development IDs and exact texts are checked against
all bound training receipts before prediction. The report retains the exact
source digest, artifact identity, configuration and per-family controls.
Generation supports 1–4096 tokens; the default remains six.
For fresh probe fits, context accepts 1–4096 tokens, fitting exposure accepts
1–16,384 sampled positions and fitting dose accepts 1–64 epochs per stage,
matching the core's bounds. The old three-window/16-epoch probe restrictions
are removed; choose settings within the declared cumulative machine budget.
The core also validates the combined candidate/fit storage bound, so these
individual maxima are not all usable together. A rejected combination is a
configuration/resource result, not evidence that the model cannot learn.

`--compile-rust` checks the exact prompt plus generated bytes using `rustc`
binary-crate metadata compilation. It inserts no expected answer, closing
suffix or repair. It checks the first configured number of Rust development
cases under the Full control, with a ten-second per-compiler limit bounded by
the remaining probe time. Source and diagnostics are retained; unselected,
unavailable and time-limited cases are explicit. Compilation does not execute
the generated program or establish assertion correctness. Query accuracy,
answer-prefix accuracy and compilation results remain separate.

```sh
target/release/r4 geometric evaluate \
  --model .uor-models/native-development/model.json \
  --input .uor-models/native-development/corpus/development.jsonl \
  --report .uor-models/native-development/development.json

target/release/r4 geometric generate \
  --model .uor-models/native-development/model.json \
  --prompt 'The little bird' --max-tokens 64 --json

target/release/r4 geometric chat \
  --model .uor-models/native-development/model.json

target/release/r4 geometric serve \
  --model .uor-models/native-development/model.json --port 8087
```

Evaluation checks next-piece prediction before revealing that piece to the
session. It refuses count-construction, readout-fit and memory-fit document IDs and
exact text copies, even when renamed. It reports matched controls, candidate coverage,
geometric-row use and work counters. Next-piece accuracy does not qualify
conversation, memory, reasoning or coding; retain actual prompt/output and
code-check results for those tasks.

The lexical codec owns leading whitespace with the following word or
punctuation piece. A prompt ending in whitespace or inside a word need not
tokenize as the prefix of a fully written training sentence. For controlled
next-piece comparisons, stop at a complete lexical boundary and verify token
alignment; do not interpret a tokenization mismatch as a memory result.

CLI chat appends to one bounded model session; `/reset` clears it and `/exit` quits.
The local service uses the same model with separate sessions. Artifact loading
does not load a teacher or source corpus. The library entry point is
`uor_r4_api::native_geometric`, sharing the core model and its checked loader.
The service admits at most 32 sessions and reserves at most 256 MiB for their
estimated backing storage. Admission occurs before allocation or checkpoint
replay; closing a session keeps its reservation until outstanding references
release its buffers. This is an aggregate buffer bound, not a process RSS limit.

## Kernel and geometry scope

The candidate integer/table kernel consists of `Session::observe`,
`Session::predict` and the response-boundary/decision operations. Fitting,
tokenization, artifact I/O, report statistics and
session allocation are host activities. Focused tests measure successful
observe/predict allocation behavior across eviction. This does not constitute
machine-code certification or a broader portability/performance claim.

Exact H4 state supplies ordered finite geometry. Fixed zeta phase accumulation
alone is commutative; order also comes from prime n-lets and H4 transport.
Paired/icosian representations must retain their explicit mapping and exact
reconstruction. A different representation of an H4 root does not automatically
add independent information, and root norm alone is constant. See the current
implementation record for which aggregate, paired, radial and learned roles
actually ran and their measured effect.

The current evidence and unresolved model work live in
[current-state.md](integration/current-state.md). This workflow does not turn a
successful build, a fitted count table, or a working workbench into alpha.
