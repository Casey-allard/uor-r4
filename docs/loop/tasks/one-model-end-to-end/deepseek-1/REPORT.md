# REPORT: one-model-end-to-end, deepseek-1

Lab: deepseek. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **The existing panel writer did not produce the canonical manifest required by the corpus loader.** Its `manifest.json` omitted `mask_schema`, split, byte counts, content hashes and tokenizer identity. The writer now emits those fields; two earlier panel directories were retained under `local/deepseek/m2-evidence/` and are not evidence.
- **Full two-sided CPU-trunk parity is unavailable for the 19-table M1 artifact.** The CPU research path implements the four-table discrete read family, not the additional 15 M1 trunk tables. The evidence lane therefore records a separate independent-writer check from the serving trunk's Q12 query and names full independent trunk parity unavailable rather than calling two calls into serving parity.
- **Generated complete-reply and generated novelty are not measured.** The available public input is a teacher-forced UORT panel. Reporting those metrics as zero would confuse reference overlap with model generation, so they are explicitly unavailable.
- **Exact prompt and reply overlap alone cannot separate a new wording from a known template whose values changed.** The completed lane now applies the project's K = 256/1,024 value-mask census and reports a `prompt-template` stratum directly.
- **The first evidence draft accepted empty inputs, could replace an output in a race and did not bind all manifest identity fields.** These were found in independent review. Empty corpora, zero targets, incomplete/unknown arguments, malformed hashes/content and existing output paths now fail closed, and reports use `create_new`.
- **The first template pass included each document BOS in its first prompt run.** That disagreed with the canonical census and changed first-turn exact/template/key calculations. Prompt and reply runs now begin after BOS at every document boundary.
- **The UORT embedded vocabulary and 19-table/four-table parity boundary needed stronger checks.** The UORT vocabulary must now equal the manifest/artifact vocabulary, and the report distinguishes full four-table CPU engine parity from the writer-only, explicitly unavailable 19-table M1 reading.

## What changed

- Added `m2-evidence` to the existing standalone one-model instrument.
- Added a deterministic run index with arena-backed exact prompt, reply and n-gram membership.
- Added canonical UORT plus manifest identity validation, target capping and artifact/corpus hashes.
- Added next-token admission, top-1, top-8 and stratified readings for prompt overlap, first-8 memory keys, reference-reply overlap and reply opening/continuation.
- Added tokenizer-bound value-masked prompt and reply template overlap at K = 256 and 1,024, including the fixed new-template test and row-level `prompt-template` stratum.
- Added independent `CpuWriterCore` recomputation from the integer serving trunk's Q12 query, with fail-closed admitted-cell, top-8 and token parity.
- Added full independent query, admitted-cell, top-8 and token parity through `CpuDiscreteProposalScorer` for supported four-table artifacts; the 19-table M1 report records that full comparison as unavailable.
- Bound manifest bytes, split, M-world cell, seed, conversation count and tokenizer hash into each corpus identity. `train-role` distinguishes canonical TRAIN from a generated control.
- Updated `heldout-panel` to emit the complete corpus manifest needed by the evaluator.
- Added thirteen library tests and three CLI tests covering exact run indexing, prompt/reply metadata, deterministic capping, 4-/8-gram overlap, value masking, tokenizer binding, malformed UORT/manifests, artifact vocabulary/BOS mismatch, empty inputs, absent-reference strata, a planted writer-metric error, deterministic serialization, argument refusal and create-new output.

Generated panel/token payloads are not committed.

## Measurements

Public smoke run:

- Artifact: `t16-frozen-zero-seed1-trunk.d11`, SHA-256 `04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d`.
- Tokenizer: `tokenizer-seed-1.json`, SHA-256 `a9fb13f70c69ffe5afed40c8251a8e702adf0b00732d35ccff9457a3309ee377`, vocabulary 4,096 and 11 single-digit pieces.
- Panel: generated M-world v2 `dev,dev`, seed 4201, 16 conversations, 1,925 tokens, 616 response targets.
- Reference index: generated M-world v2 `train,train`, seed 4101, 64 conversations, 7,906 tokens, 2,708 response targets.
- This reference index is a **generated control**, not the canonical TRAIN corpus. The public M1 artifact and panel are real; the overlap index is a smoke fixture, not a capability result.
- All 616 response targets: admission 365, top-1 293, top-8 306.
- Reply openings: 58 targets, admission 7, top-1 7, top-8 7.
- Reply continuations: 558 targets, admission 358, top-1 286, top-8 299.
- All 58 prompts, covering the 616 scored targets, were absent from the generated reference index, and all 58 first-8 keys were absent.
- None of the 58 prompt value-masked templates occurred in the generated reference index at K = 256 or 1,024. The panel therefore meets the fixed new-template rule against this **generated control**.
- Of 58 reference replies, 19 occurred verbatim in the generated reference index; 241 of 442 reply 4-grams and 158 of 234 reply 8-grams were absent.
- Reply-template overlap against the control was 30/58 at K = 256 and 24/58 at K = 1024 (`517,241` and `413,793` parts per million). This is the expected recall signal: the prompts are new, while many reply templates remain shared.
- The independent CPU writer matched integer serving on all 616 rows: zero admitted-cell, top-8 and top-1 mismatches.
- Full independent CPU-trunk parity remains `unavailable`; the writer check begins from each serving row's Q12 query.

Command shape:

```sh
target/release/m2-evidence artifact=.../m1-s1.d11 \
  tokenizer=.../tokenizer-seed-1.json \
  panel-tokens=.../panel/tokens.u16 \
  panel-mask=.../panel/response_mask.u8 \
  panel-manifest=.../panel/manifest.json \
  train-tokens=.../train-control/tokens.u16 \
  train-mask=.../train-control/response_mask.u8 \
  train-manifest=.../train-control/manifest.json \
  train-role=generated_control \
  out=.../evidence-deepseek-1-smoke.json
```

The exact panel and control identities are in `evidence-deepseek-1-smoke.json`.
Report SHA-256: `2fc34ee636a24481cda96fdc85dfa627c81032f4297c0ae70766c303a4ab7a2b`.

The full four-table CPU-engine path was exercised separately on public M1 base seed 1
(`t16-frozen-zero-seed1.lut`, SHA-256
`fc31bcc7a88250f3758d07a8b3d12480a5332acb5437b0c0f0e5c71ae88939ca`) over 128 public DEV
targets. `cpu_engine_parity` checked 4,096 independent Q12 query coordinates and found zero
query, admitted-cell, top-8 or token mismatches. The 19-table M1 smoke above retains writer-only
parity and reports that full independent trunk comparison as unavailable. The four-table result is
committed as `docs/labs/one-model-end-to-end/evidence-deepseek-1-four-table-parity.json`
(SHA-256 `e3e4e3b868dffc4953b018cc7afcbc643622e4493a3f8f835a4c507a9bbdc5ea`).

The two generated corpora can be recreated from the public tokenizer with:

```sh
heldout-panel out=.../panel tokenizer=.../tokenizer-seed-1.json \
  cell=dev,dev conversations=16 seed=4201 protocol=2
heldout-panel out=.../train-control tokenizer=.../tokenizer-seed-1.json \
  cell=train,train conversations=64 seed=4101 protocol=2
```

Both commands run with the instrument's `panel-writer` feature:

```sh
cargo run --offline --features panel-writer --bin heldout-panel -- ...
```

## What this decides for M2

Read a complete-reply or fresh-phrasing gain by strata. A gain confined to
`prompt-template:present` or `reference-reply-overlap:present` is recall and cannot qualify the
generalization route. The next useful M2 panel must retain separate readings on
`prompt-template:absent` and `reference-reply-overlap:absent` rows; the latter requires an
honest reference index and, once generated outputs are supplied, the same stratum over model
replies. This instrument does not itself qualify M2 because the public smoke has a generated
control rather than canonical TRAIN and has no generated replies.

## Checks run at the head

- `cargo test --offline --features cpu-parity --lib --bin m2-evidence -j1`: 13 evidence tests and 3 CLI tests passed.
- `cargo test --offline --features panel-writer --bin heldout-panel -j1`: the panel-writer test passed after the repository's known duplicate Cargo artifact race was cleared.
- `cargo clippy --offline --all-targets -- -D warnings`: passed; only pre-existing path-dependency warnings appeared.
- `rustfmt --check` on the changed Rust files: passed.
- `cargo build --offline --bins -j1` with an isolated target directory passed; the public
  artifact/panel smoke ran the resulting debug binaries and wrote the measured report. A parallel
  release-bin build hit the existing duplicate `uor-r4-integer` artifact race and is not claimed.
- `uor-harness check`: 0 errors and 40 existing/site-wide warnings.
- `cargo run --locked -p xtask -- roadmap check`: 134 items OK.
- `cargo run --locked -p xtask -- links`: completed; it reports repository-wide historical broken links, with no command failure.
- The command reads the integer artifact through `IntegerDiscreteModel::evaluate_prefix`; it does not invoke CUDA or modify model weights.

## Skills used

None.

## Still open

- Run against the canonical public TRAIN split, which is unavailable in this worktree because its HF store is private.
- Add generated-reply input so complete/incorrect/loop and generated n-gram novelty can be scored without conflating them with reference overlap.
- Add a CPU implementation of the 15 additional M1 trunk tables to obtain full two-sided CPU-trunk query parity; the current independent writer check is exact but begins from the serving query.
- Supply count-table, memory-off and no-read arm files to turn their `unavailable` entries into measured controls.
- Run the same report on the active FBS integer artifact after Q5 parity lands.

## Plain summary

This unit adds a laptop-only report that separates a model's next-token accuracy by whether the prompt, its value-masked template, its exact first-eight-token memory key, or its reference reply appeared in the supplied training material. On a public M1 model and a generated panel it could identify new prompt templates and unkeyed openings while showing that reply templates still overlap the control. It independently reproduces the serving writer on every scored query, while clearly naming the part of the CPU parity path that does not yet exist. It deliberately says it cannot judge generated complete replies or generated novelty without those outputs. The result is an evaluation instrument, not a claim that the model has learned to converse.
