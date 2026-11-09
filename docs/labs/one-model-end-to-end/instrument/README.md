# One-model-end-to-end instruments

This standalone crate writes M-world v2 panels and evaluates an exported integer discrete model on a
CPU. It is not in the workspace and it does not train or modify model weights.

## `heldout-panel`

```sh
cargo run --release --offline --bin heldout-panel -- \
  out=NEW_DIR tokenizer=TOKENIZER.json cell=PHRASING,VALUE \
  conversations=400 seed=1 protocol=2
```

`heldout-panel` is behind the `panel-writer` feature so the evidence-only graph does not pull the
offline training dependencies:

```sh
cargo run --release --offline --features panel-writer --bin heldout-panel -- ...
```

`cell=dev,dev` holds prompt phrasing out. `cell=train,train` is a generated control. The writer emits
`tokens.u16`, `response_mask.u8` and a hash-bound `manifest.json`.

**Chained panels (claude-275).** `chain=N` (N at least 512) draws every conversation with
`MWorld2::chained` in the cell, up to N tokens: episodes back to back, with notes stated early and
asked for later. It meters them as `m-world-recall-corpus chain=` meters TRAIN's chained draw. Beside
the split it writes `queries.json`, each query's reply position, length, answer and `Chain` record,
and `user_turns.json`, the rows `m-world-recall-corpus exclude=` reads. Without `chain=` the output is
unchanged. `check-queries` checks a chained panel's queries against its split:

```sh
heldout-panel check-queries split=PANEL tokenizer=TOKENIZER.json   # checked N queries, 0 failed
```

The test that writes a chained panel needs the pinned d36d3e87 tokenizer:
`UOR_R4_PINNED_TOKENIZER=TOKENIZER.json cargo test --features panel-writer -- --include-ignored`.

## `m2-evidence`

The input paths are explicit. `train-role` is required with a reference index and records whether it
is canonical TRAIN or a generated control:

```sh
cargo run --release --offline --features cpu-parity --bin m2-evidence -- \
  artifact=MODEL.d11 \
  tokenizer=TOKENIZER.json \
  panel-tokens=PANEL/tokens.u16 \
  panel-mask=PANEL/response_mask.u8 \
  panel-manifest=PANEL/manifest.json \
  train-tokens=TRAIN/tokens.u16 \
  train-mask=TRAIN/response_mask.u8 \
  train-manifest=TRAIN/manifest.json \
  train-role=generated_control \
  max-targets=1024 \
  generate-new-tokens=16 \
  out=NEW_REPORT.json
```

The report contains:

- artifact schema, table count, hashes and shape;
- panel and reference manifest, tokenizer, content hashes and corpus identity;
- target admission, top-1 and top-8 overall;
- opening versus continuation and prompt-overlap, value-masked prompt-template, first-8-key and
  reference-reply strata;
- reference prompt/reply exact overlap and reply 4-/8-gram novelty;
- value-masked prompt and reply template overlap at K = 256 and 1,024;
- opt-in greedy replies with EOS, token-limit and context-limit stops, exact-reference matching,
  repeated 4-grams and generated reply 4-/8-gram novelty;
- an independent `CpuWriterCore` comparison against the integer serving writer;
- for supported four-table artifacts, a full independent CPU read-query/writer comparison;
- explicit `unavailable` entries for data or controls not supplied.

The input UORT header vocabulary, tokenizer structure/hash, manifest hashes, vocabulary, BOS,
conversation count and response inventory are checked before scoring. Each document's BOS is
excluded from prompt and first-8-key runs, matching the census and turn-memory semantics. Empty
corpora, missing prompt or response runs, zero targets, `max-targets=0`, malformed artifacts,
`generate-new-tokens=0`, unknown or duplicate arguments and an existing output path are refused. An
output is created with `create_new` and never replaced.

`generate-new-tokens=N` decodes one reply after each prompt with deterministic greedy selection. A
reply is `complete` only when it emits the tokenizer's added `<|eos|>` token; token and context limits are separate.
`exact_reference` compares reply content without the EOS control token. Logged reply ids include
EOS when it was emitted; novelty and repeated-4-gram readings use that logged sequence. The distinct
text count excludes empty context-limit rows, which are reported separately as `empty_text_replies`.
EOS is required only when generation is requested.

The template census follows `train-in-space --census`: TRAIN's K most frequent tokens are kept, all
single-digit tokenizer pieces are masked, every other token is replaced by one mask sentinel, and
adjacent sentinels collapse. A panel is reported as holding new prompt templates when its K = 1,024
masked-prompt share is at most 0.5.

### Parity boundary

For a supported four-table `uor-r4.lut-discrete/1` artifact, `cpu_engine_parity` uses
`CpuDiscreteProposalScorer` to recompute the query and checks query coordinates, admitted cells,
top-8 and top-1 against integer serving.

For a supported 19-table `uor-r4.lut-discrete-trunk/1` artifact, it uses
`CpuDiscreteTrunkReader` through the CPU Relation/Flock/Route read, recurrence, joint-pointer
rotation and post-rotation writer. The report checks every complete Q12 query coordinate,
admitted-cell vector, top-8 and token against integer serving. Other schemas remain explicitly
`unavailable`; no unsupported artifact is treated as parity.

### Evidence still unavailable without more inputs

- Semantic correctness against human judgments is not available; the instrument records completion,
  loops and exact reference agreement, not a claim that a different reply is wrong.
- Count-table, memory-off and no-read comparisons need corresponding exported arms.

The public smoke result is recorded in the
[deepseek-1 report](../../loop/tasks/one-model-end-to-end/deepseek-1/REPORT.md).

## `real-carry-geometry`

This CPU-only diagnostic has two measurements and a compact receipt writer. It
trains nothing, changes no artifact and moves no bar.

`train-census` parses a published mixed dialogue corpus through its UORT header,
response mask, manifest and pinned tokenizer. It identifies a question as the
last user turn before an assistant response when that user text ends in `?`.
For every identified question it records the distance from the question to the
first scored reply row and to the first masked reply target. A separate
two/three-token repeated-context-span heuristic is labelled `not-semantic`:
decoded examples can show a key or template phrase instead of the answer value,
so it supports no value-reach claim.

```sh
cargo run --release --offline --features cpu-parity --bin real-carry-geometry -- \
  train-census \
  --tokens TRAIN/tokens.u16 --mask TRAIN/response_mask.u8 \
  --manifest TRAIN/manifest.json --manifest-sha256 MANIFEST_SHA256 \
  --tokenizer TOKENIZER.json --tokenizer-sha256 TOKENIZER_SHA256 \
  --carry-rows 32 --out NEW_CENSUS.json
```

`first-reply` renders the official `m2-fresh256` prompts exactly as
`served-reply-panel`, replays the carried artifact and its byte-exact `-C`
control through `CpuFbsForward`, and records each read's Hamming selections,
the carried query delta, the exact carry terms, asked/other ranks and point-key
status. It refuses unless artifact and panel hashes match, `-C` reproduces byte
for byte, both official first-rank reports agree case by case, and every output
path is new.

```sh
cargo run --release --offline --features cpu-parity --bin real-carry-geometry -- \
  first-reply \
  --carried CARRIED.uorlut --carried-sha256 CARRIED_SHA256 \
  --minus-c MINUS_C.uorlut --minus-c-sha256 MINUS_C_SHA256 \
  --carried-report CARRIED.json --minus-c-report MINUS_C.json \
  --carried-rank CARRIED_RANKED.json --minus-c-rank MINUS_C_RANKED.json \
  --inputs inputs.json --labels labels.json --plan plan.json \
  --panel-manifest manifest.json --panel-manifest-sha256 MANIFEST_SHA256 \
  --tokenizer TOKENIZER.json \
  --expected-carried-first-lines N --expected-minus-c-first-lines N \
  --out NEW_TRACE.json
```

The full traces are large. `receipt` verifies optional byte-identical repeats
and writes a small committed summary; the full files, input hashes and receipt
hashes stay in the named evidence report.
