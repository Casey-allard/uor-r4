# Native geometric development workflow

The native path is `r4 geometric`. Data preparation, fitting, artifacts,
evaluation, sessions and generation use Rust. Its initial learner estimates
conditional score tables over prime lexical identities and geometric context.
Separate readout fitting learns bounded integer gates over seven feature groups
and sufficiently supported last-prime query keys. It does not yet learn the
state's memory-write law.
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

The optional `native_geometric_memory_probe` example compares physical value
retention, predicted values and paired value changes. `--memory-read` adds the
new reader, and `--family rust` represents the same controlled relationships as
Rust variable assignments and assertions. These are training/evaluation data
families; the model has no fact grammar or special handling for those objects,
strings or assertions. They are not general conversation or coding tests.
`--family mixed` trains one artifact on both families, using the configured
document and world counts once per family and reporting each family's results
separately. Use `--word-cues` only for the named equivalence experiment.

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

The candidate integer/table kernel consists of `Session::observe` and
`Session::predict`. Fitting, tokenization, artifact I/O, report statistics and
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
