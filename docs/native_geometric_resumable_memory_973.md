# Resumable native memory fitting and joint composition — #973

This continues the [query-context checkpoint](native_geometric_query_context_973.md)
from `ea71af0ec39050289337f1d2b9dd2a53d8f3ff63`. The Rust memory fitter now
separates distinct supervised exposure from its live example buffer and can
resume actual learning state. The first broader joint comparison improves some
teacher-forced predictions, but does not produce useful Rust composition or
reliably improve complete prose generation. The working finite reader and its
historical results remain preserved. These are open-development results, not
alpha or final held-out qualification.

Structured measurements and artifact paths are in
[the evidence record](evidence/native_geometric_resumable_memory_973.json).

## Training change and continuation

`r4 geometric fit-memory-stream` is an explicit successor for the existing `/3`
reader. `--total-positions` bounds the distinct source targets; `--batch-positions`
bounds live examples and retains the existing route-buffer constraint. Increasing
epochs replays this declared population; it does not silently enlarge exposure.
The sampler redistributes quota left unused by short documents, then selects
uniform body positions and a bounded final-position reserve. Sources remain
ordered and bound to their exact receipts. Prose and Rust task documents are
interleaved so a source prefix does not select only one family.

Discovery, pointer fitting, global calibration, query calibration, refinement
and epoch selection traverse the same population. Calibration accumulates its
grids over bounded replay passes and applies choices once per stage. Epoch
selection uses the population loss, not only the last batch. Reports separate
distinct targets, repeated example visits, context replay, route reachability,
feature registration/drop events and per-document exposure.

Checkpoints retain the unquantized weights, feature registry, best-epoch state,
stage/epoch/document/token/sample cursors, calibration accumulators and exact
baseline/source/configuration identities. Resumption reconstructs the current
document prefix through the normal causal state path and charges that replay.
This is continued memory-head fitting, rather than restarting an independent
fit on another chunk. Historical one-shot fitting entry points remain intact.

The joint fit was deliberately stopped at 300 batch calls during pointer epoch
1, document 279, token 24, with no final artifact. The next invocation restored
that checkpoint and finished at 3,422 batches. It visited 30,038 distinct targets
with a peak of 256 live examples, 871,102 repeated example visits and 871,126
replayed context positions. Its selected memory artifact is
`blake3:e6197bfab639b00627f2c4b3289d8d8e42a504c8c55eb41bfe89ff75b978a49d`.

The native serving operator, deterministic write admission and 18-feature
memory layout are unchanged. Full prime identities, ordered query addresses,
H4 transport, signed orientation, fixed zeta phases, exact `Z[phi]`/paired state
and UOR artifact identity retain their implemented roles. This scheduling change
does not implement learned writes, information retention beyond ring eviction,
or a new nonlinear geometric composition operator.

## Joint source and measured exposure

The authored Rust source generator writes 512 construction documents, 512 fit
documents and 64 development cases, equally divided by family. It preserves
whole tasks and checks exact document bytes and IDs across splits and against
artifact training receipts. The source digest is
`e45e502671f16bc5443a6d18a2ffaf9c89d838d66c88501c03e59c58ea7e6b20`.
The initial preparation correctly refused an exact overlap between development
world 100002 and fit world 322. Its failed report is retained. The corrected source changes actual
development compositions and repair direction rather than adding identifiers
or comments merely to defeat overlap.

Both memory fits use the **same learned readout baseline**, `joint-readout.json`,
`blake3:32b37621b5c8d27c8469d6143a4be49f38311a938f149071c1fdefb4a769d871`.
This baseline already fitted 16,384 readout targets for eight epochs, with 115
query gates. It is not the unfitted count artifact. Context is 512; all task
prompts fit inside it. The longest development prompt is 82 tokens including
BOS. Word cues, eight query tokens, four source offsets, four postings per
address, 128 memory routes and eight epochs per memory-learning stage are shared.

| Fit population | Prose positions | Rust positions | Total | Live example peak |
| --- | ---: | ---: | ---: | ---: |
| Legacy one-shot | 2,048 | 2,048 | 4,096 | 4,096 |
| Stream, complete population | 15,104 | 14,934 | 30,038 | 256 |

The legacy quota selected only the final eight positions of every document
(`body_positions = 0`). The stream selected every available position, including
all 2,176 prose and 3,860 Rust completion tokens; its 30,038 total also includes
document EOS targets. It registered 98,284 features with zero drops, versus
55,149 for the legacy fit. The training population is fully exposed here, so a
remaining failure cannot be explained by unvisited task positions in this run.

Development includes updated facts, paired answers, contradiction correction
and unsupported questions; Rust includes variable dependencies, function
composition, function bodies and edit instructions. Fresh names and some new
forms deliberately test transfer. The function-body order changes from
addition-then-doubling to doubling-then-addition, and repair direction changes
from replacing subtraction with addition to the reverse. These are finite
synthetic tasks and a designed transfer challenge. They are not an independently
sampled natural conversation or software-engineering benchmark. Live
compiler-feedback prompts are also outside the fit instruction templates.

## First joint behavior comparison

The legacy artifact is
`blake3:58e949352ebacf3638295a45557ab0d038d0f18772a3a36e143e23d0b215ca2a`.
The table separates correct first pieces, correct teacher-forced completion
pieces and complete raw generation. Complete generation is exact expected bytes;
prefix correctness is separately retained in the structured report.
Exact grading does not credit alternative valid wording or implementations;
the raw outputs and sampled compiler outcomes are separate evidence.

| Artifact/control | Prose first / 32 | Prose completion pieces / 320 | Prose exact / 32 | Rust first / 32 | Rust completion pieces / 504 | Rust exact / 32 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Legacy full | 10 | 179 | 2 | 0 | 302 | 0 |
| Stream full | 16 | 188 | 6 | 0 | 314 | 0 |
| Shared memory-disabled baseline | 14 | 174 | 7 | 0 | 272 | 0 |

The stream improves teacher-forced completion prediction over the legacy fit in
both families, but complete prose generation remains below memory-disabled and
Rust composition still fails. The prose exact successes comprise two
contradiction answers and four unsupported-answer responses. Updated fact and
two-fact composition each have zero exact completions. Every Rust task group
has zero correct first pieces and zero exact completions.

Host diagnostics distinguish memory admission from the combined post-score
shortlist. Stream memory admits the first target in 16/32 prose and 8/32 Rust
cases. All eight updated-fact answers and all eight repair-operator targets are
admitted, yet none is selected correctly. The 16 numeric dependency/function
answers are absent from memory: a copy route cannot by itself create an
unobserved arithmetic result. The function-body first operation is also absent
from memory in this population. Absence from the combined top-32 shortlist alone
would not establish admission failure because scoring can evict an admitted
target. Query-row registration is reported separately; registration does not
establish a nonzero gradient or useful learned influence.

Raw stream examples illustrate both outcomes. On `prose/100006`, whose updated
fact says Finn's badge is black, generation is exactly:

```text
 No. finn's badge is black now.
```

On `prose/100000`, where Suri's badge was corrected to white, the unchanged raw
continuation is:

```text
 Unknown now.
The wind moves outside. Assistant: No..
User: Correction:.
User: Correction:
User: What color is
User:.
```

On `rust/100000`, the prompt assigns `suri = 13`, computes `orin = suri + 4`,
updates `suri = 24`, and ends at `assert_eq!(orin,`. Generation is:

```text
 then doubled. Assistant
    let
```

All eight sampled, unchanged generated Rust sources fail metadata compilation
with pinned `rustc 1.97.1`. Two actual compiler-feedback repair attempts return
zero bytes and stop at end-of-document; both fail compilation for missing
`main`. No output is repaired by the harness, no expected suffix is inserted,
and no program is executed. Execution is permitted only for exact matches to
the generator's authored safe arithmetic/assertion source; alternative valid
programs would not receive a semantic execution verdict under that conservative
rule. Compiler child timeouts are monitored, not an OS/process-tree sandbox.

## Response-focused supervision correction

The whole-population negative motivated a response-focused objective experiment:
the fit supervised continuation of input facts, comments and distractors as well
as requested answers. The opt-in generic supervision map now selects exact
target-token intervals while replaying the complete documents as causal
context. It binds the baseline artifact, ordered source receipts and token
spans; checkpoint restoration and artifact lineage retain its identity. Empty,
overlapping, out-of-bounds, changed-source or wrong-baseline maps are refused.
Feature registration, training, calibration and selection use the same eligible
population. No answer parser or language-specific inference rule is introduced.

`prepare --model joint-readout.json` writes the additive `supervision.json`.
The `joint-source-v3/source.json` and `fit.jsonl` are byte-identical to their
`joint-source-v2` counterparts. The map selects 6,036 completion tokens plus
512 EOS targets, for 6,548 targets total: 2,432 prose and 4,116 Rust. It excludes
23,490 input loss terms while preserving their observed context. The same
already-fitted readout baseline is used; this is a response-target change, not
new readout fitting or a new arithmetic operator.

The masked fit completes with all 6,548 eligible targets, 256 live examples,
59,467 features and zero drops. Its artifact is
`blake3:2cf5414e4d5be70e6d8b739eb31e5aad2f005aff31a7c57ab437898a5e3e98e6`;
the supervision identity is
`blake3:e04ada583e3e571086ed2d4891fb6ffbdf779d06ac93064c989a42261ef4729f`.

| Objective/control | Prose first / 32 | Prose completion pieces / 320 | Prose exact / 32 | Rust first / 32 | Rust completion pieces / 504 | Rust exact / 32 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Whole-population stream | 16 | 188 | 6 | 0 | 314 | 0 |
| Response-focused stream | 15 | 196 | 7 | 0 | 306 | 0 |
| Shared memory-disabled baseline | 14 | 174 | 7 | 0 | 272 | 0 |

Response supervision improves prose completion-piece prediction, but Rust
completion prediction falls relative to the whole-population fit and exact prose
only ties memory-disabled. It is not selected as a joint quality improvement.
Its seven exact prose responses comprise two contradiction corrections and five
unsupported answers, versus seven unsupported answers with memory disabled.
Updated-fact and two-fact exact completions remain zero. The tied total therefore
does not mean identical task behavior, and the experiment does not establish
input-loss dilution as the cause of the original failures.
Zero exact updated-fact/two-fact completions also does not mean zero partial fact
recovery. For example, the response-focused `prose/100013` continuation is
exactly ` cyra has green.\n`: that is the first requested fact, but it omits the
second requested fact, Ben's red badge. On `prose/100000`, the continuation
begins ` is white now.` with the correct updated value and then drifts into
unrelated transcript text. Those partial successes remain incomplete answers;
their raw outputs are preserved separately from exact scoring.
Its eight sampled generated Rust sources again fail compilation and its two
actual-feedback repairs are empty and fail compilation. None executes. The
legacy, whole-population and response-focused artifacts all remain preserved.
More exposure and response masking have now both been exercised; neither alone
resolves these composition and query-selection failures.

## Broader corpus comparison

The same larger-corpus learned baseline and original 128-token context are used
for both current eight-epoch fits. Source documents remain the preserved
`expanded-corpus/readout.jsonl` and `development.jsonl`; the development set has
103 documents and 146,668 prediction targets. This is a reused open-development
corpus, even though the generic CLI report calls it held-out. It is not a final
held-out assessment. The earlier 30.5684% artifact remains its own historical
negative, rather than being replaced by this new comparator.

| Current artifact/control | Distinct fit positions | Correct / 146,668 | Accuracy |
| --- | ---: | ---: | ---: |
| Legacy full | 4,096 | 48,236 | 32.8879% |
| Stream full | 32,768 | 51,847 | 35.3499% |
| Same stream artifact, memory disabled | — | 53,165 | 36.2485% |
| Same stream artifact, geometry disabled | — | 53,556 | 36.5151% |

The stream narrows the regression but still trails memory-disabled by 1,318
targets. It reaches the 262,144-feature cap with 2,736,196 dropped feature
events; the legacy fit registered 206,127 features with zero drops. The expanded
population therefore introduces a measured feature-capacity limit in addition
to the unresolved learning/selection problem. The registered-query diagnostic
must not be interpreted as a proof that all those rows received useful training.
Geometry-disabled retains prime-addressed memory and query information. Its
higher score establishes no added geometric benefit in this comparison and does
not demote the owner's primary geometry or prove that it cannot learn.

## Resource and validation record

Saved-model preservation evaluation reloads the historical
`blake3:b5ba144fb293358bd45b77ce848f7ad100e26524cfd08586b2a96deea85c081d`
artifact and confirms 96/96 first answers and answer prefixes in each family,
with both answers correct in all 48 value-change pairs per family. The
memory-disabled scores remain 43/96 prose and 39/96 Rust. Geometry-, zeta- and
H4-disabled controls also remain 96/96 in each family. This check runs the saved
reader; it is not a refit or a new independent capability assessment. No Rust
compilation or execution is repeated here; the prior 96 compilations and 96
successful executions remain the historical evidence described in the previous
record. Full details are in `reader-preservation/report.json`.

All files remain under
`.uor-models/native-memory-stream-2026-09-05`, with successful, intermediate and
failed preparations retained. `commands.jsonl` records exact native executable
arguments and charged launch times. Both fits and evaluations use one model
process at a time. The inherited allocation is 1,800 seconds cumulative model
work, a monitored 4 GiB process RSS target and 4 GiB additional output/build
storage; the inherited 301.277 seconds is not reset for this task.

The completed model-work ledger is **716.243 seconds cumulative**, comprising
301.277 inherited seconds and 414.966 new seconds, including the failed source
preparation, preservation check, fits, checkpoint continuation and evaluations.
**1,083.757 seconds remain** in the same allocation. The joint stream reports
269,991,936 peak process bytes, response-focused fitting 137,199,616, the broad
stream 1,254,129,664 and the broad legacy fit 1,426,538,496. Probes do not
internally measure peak RSS.

The storage snapshot is 315,324 KiB for new artifacts, approximately 1,177,268 KiB
for the full worktree, and 5,495,784 KiB for the shared build target; the observed build-target
peak is 5,663,036 KiB. The shared target was inherited at approximately 4.7 GiB
(approximately 4.5 GiB before the preceding continuation). These are sampled
host values with an approximate inherited baseline, not a hard process-tree or
storage guarantee. Build and engineering-check time is separate from the model
ledger. No paid compute was used.

Focused native core checks report 25 passing tests, native CLI/service checks
report 13, and the example's four source/safety checks pass. Formatting, core,
example and root Clippy, architecture policy, claim wording and diff whitespace
checks pass. Broader workspace/release QA, WASM,
fuzzing and legacy teacher parity are `NOT_RUN` for this package; historical
queue compatibility statuses do not stand for those test results.

The executed commands below used rustup-managed
`/Users/casey.allard/.cargo/bin/cargo`,
`CARGO_TARGET_DIR=/tmp/r4-native-geometric-target`, `CARGO_INCREMENTAL=0` and
`CARGO_BUILD_JOBS=3`. Both release binaries used for the measured runs built
successfully.

```sh
cargo test --offline -p uor-r4-core --lib native_geometric
cargo test --offline --bin r4 native_geometric_
cargo test --offline -p uor-r4-core --example native_geometric_joint_probe
cargo clippy --offline -p uor-r4-core --lib --example native_geometric_joint_probe -- -D warnings
cargo clippy --offline --bin r4 -- -D warnings
cargo run --offline -p uor-r4-core --bin r4-policy-check -- .
cargo fmt --check
python3 scripts/check_claim_wording.py
git diff --check
cargo build --release --offline --bin r4 -p uor-r4-wasm-router
cargo build --release --offline -p uor-r4-core --example native_geometric_joint_probe
```

The next implementation should address learned read/selection and geometric
value composition, with learned write/state operators still missing. The
diagnostics distinguish admitted-but-misranked factual/repair targets from
arithmetic outputs that no retained copy route supplies. This calls for a
targeted model/operator change with representative generation, not another cue
table expansion or unchanged repetition of the present fits. Both useful
conversation/memory and coding/reasoning remain required for alpha.
