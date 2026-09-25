# Standalone integer serving — prospective work card

Continuation of PR1396 under #973/#820 and D9. This plan precedes loaded execution.
The [active contract](current-state.md#active-execution-contract) owns the next
serving deliverable; this record pins its actual execution choices.

## Deliverable and fixed conditions

Move the retained numerical model into a dependency-light Rust library and a
standalone CLI. Training reuses that same implementation. Share the existing BPE
engine so the serving binary does not link the training, Candle, model-source or
core crates. Package the accepted model codes, bound table artifact and exact
parent tokenizer in a sealed portable bundle. The CLI consumes text and emits
text through integer model predictions and integer token selection.

Both accepted learned-code parents and PR1396's table artifact remain unchanged.
Training provenance/evaluation/session context stays256; full admission exposes
all previous occurrences, up to255 at the last input position. No fitting,
selection of checkpoints, new corpus/holdout, tolerance change or admission sweep.

TextSession retains state and exact occurrence history across append/generate
calls. A final emitted token stays pending until more input/prediction is needed;
it is then ingested exactly once. BOS0 is inserted once, EOS1 is checked against
the bound tokenizer. Context exhaustion is an explicit error, with prospective
continuation capacity checks. Prompt groups in the CLI are independent sessions;
the library supplies mutable sessions. No silent sliding window or eviction.

## Selection and exactness

Greedy remains largest Q48 mass with lowest token ID on ties. New categorical
selection uses Q48 masses directly at temperature1, optionally restricted to
stable top-k. Xorshift64 and mask/rejection provide deterministic integer draws;
zero seed has a documented nonzero mapping. This is a new sampling policy,
not equality with the old temperature0.8 floating sampler. Normalization,
bounded rejection failure and source answer policy remain explicit.

Extraction must match every saved integer target-distribution hash from the
retained four full256 windows, Read/NoRead in both arms:4,096 total predictions.
Greedy source generated token sequences must match all128 prior variants; the
expected read-enabled correct counts remain28/24, NoRead0/0. Reuse complete saved
probabilities/oracles rather than inventing another language acceptance test.

## Necessary verification and decisions

Compile the standalone and training compatibility boundaries. Exercise changed
codec, integer selection, tokenizer and session behavior with focused checks;
reuse unchanged arithmetic/report-root checks. Replay existing target hashes and
source responses, then run the five existing continuation prompts with the new
integer categorical policy and preserve actual output. Verify retained session
history with an actual loaded append/generate continuation and explicit capacity
failure. Bind source/executable/bundle and inspect the new compiled numerical
model/selection boundary and actual dependency graph.

Attach the native macOS sampling profiler to the mandatory generation run;
measure whole CLI and nested model-call cost. Only a demonstrated hotspot may
justify an exact-output optimization. If implemented, preserve the original
profile/binary and require identical Q48 results and greedy outputs. A speedup
requires measured cost, not a guess about instruction count. No energy claim
without physical measurement. A sound implementation still inherits unreliable
prose and dense parameter access; integration does not qualify language or D5.

Success advances the standalone serving path. A mismatch identifies an extraction,
artifact or session error and receives a local repair; it never authorizes a model
refit. Stop checks once those risks are resolved. Report failed/incomplete behavior
and preserve every attempt rather than adjusting acceptance.

## Resources and parallel work

Whole-cycle90-minute projection from19:39:00UTC, with62seconds of conservative
initial recovery allowance; deadline21:09UTC. Starting shared charge543,642,469ms
under547,200,000ms. Under standing owner authorization, prospectively add2,400,000ms
for the complete cycle, raising the cumulative limit to549,600,000ms. Charge actual
unique elapsed work, including reviews, build, execution, repairs and delivery.

New retained/build storage ceiling1.5GiB, aggregate model RSS8GiB, one Cargo process,
at most two model workers, nested backend threads1. Preserve14.5GiB physical
reserve plus128MiB stop and64MiB closeout; start free16.55GiB. No deletion or
external training. Reuse the isolated full worktree and target cache. RDC runs a
bounded DeepSeek systems/math review and local model processes. Independent
agents own extraction, sampling and tokenizer/compatibility work while the
principal owns session/CLI integration and adjudication.
