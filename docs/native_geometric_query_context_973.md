# Joint native query-context learning — #973

This continues the [native recovery](native_geometric_recovery_973.md) from
`f20f27c6433bd9a36a1e25979385380906221e27`. The objective is useful prose memory
and Rust variable/update behavior on the same native artifact, while retaining
the primary prime/H4/zeta architecture and previous results. This is open
development work; neither a finite copy task nor compilation qualifies alpha.

## What changed

The explicit `--query-context` reader uses memory schema `/3`. Its last two
features encode the ordered pair of complete recent query primes, then that
pair combined with query distance, source offset and newest-first posting rank.
The combined address uses disjoint 24+24+5+4+3 bit fields; host validation rejects
complete primes outside the declared domain. It performs no truncating prime
hash. Features 0–15 retain their previous bias, distance, age, H4, orientation
and fixed-zeta roles. Candidate admission and the 18-feature runtime width are
unchanged. `--word-cues` remains a separate explicit choice.

The `/3` training successor shares half of the initial target responsibility
uniformly among all admitted routes carrying the training target token. It then
calibrates the shared bias and the existing query-pair bias rows before the
same maximum-route refinement. The additional calibration uses only the fit
examples for each query context. No source-position labels, language parser,
expected-answer rule or development labels enter the model's fitting path.
Training stays in Rust; inference retains the existing bounded integer reader.
Schemas `/1` and `/2`, including their fitting objective, remain available.

The optional probe now evaluates saved models against byte-preserved sources
without refitting. It checks development IDs and exact text against all model
training receipts before prediction. It can compile exact generated Rust
continuations with a bounded `rustc --emit=metadata` invocation, saving source
and diagnostics. It appends no expected answer, closing suffix or repair, and
does not execute generated code. Context and fitting bounds now match the core
instead of retaining the old three-window and 16-epoch experiment restriction.
See the [workflow](native_geometric_workflow.md) for commands and limits.

## Why this was the next correction

The saved mixed models already admitted every expected answer to memory:
192/192 for each of the exact- and word-cue artifacts. Reconstructed candidate
counts matched all 384 saved cases. The configured 128 visits exhaust
8 query positions × 4 source offsets × 4 postings. This rules out insufficient
candidate capacity for these cases; it does not rule out capacity limits on
other data or longer contexts.

With query features alone, 8-epoch word-cue fitting reached 96/96 prose but
40/96 Rust. At 32 epochs it reached 96/96 and 38/96. The identifier route's
conditional weight stayed at +3 while the competing quotation-mark route
grew stronger. Its partial shared/conditional score advantage grew from 14.00
to 16.09 nats, excluding prior, age and geometric contributions. These weights
supported a learning-responsibility diagnosis, not another window increase.

Sharing initial target responsibility corrected that starvation: at 32 epochs
the word-cue model reached 96/96 Rust and all 48 pairs, but prose remained at
43/96. The useful prose route was learned but suppressed by calibration.
This motivated fitting the already represented query-dependent bias rather
than adding another runtime mechanism. Every intermediate artifact and result
is retained; later success would not change those measured failures.

## Data and comparison scope

The mixed source contains 512 construction documents, 128 separate fit
documents and 192 development cases, equally divided between prose and Rust.
The source digest is
`c8bc7438effe89185d51588f8adf45074cd7d8d09699fac47f8758b5f8f270ad`.
Its model baseline is
`blake3:b5c1666e7f9bc61cb06baf91ec2e899b79e5c1a6c1a0f46351544f2d2033a46c`.
Development worlds were excluded from construction and fit. These are related
finite grammar families and open development examples, not an independent
general language or software engineering benchmark.

The matched fits keep context 512, query positions 8, source offsets 4,
postings 4, memory candidates 128 and sampled positions 4,064. Each named epoch
count applies separately to pointer warmup and maximum-route refinement.
Feature drops are zero. A single shared baseline scores 43/96 prose and 39/96
Rust when memory is disabled. Comparison requires both groups separately;
their combined average cannot conceal a regression in one group.

`GeometryDisabled` removes H4/orientation and zeta contributions while keeping
prime-addressed memory and query-prime features. It is not a prime-identity
ablation. `MemoryDisabled` removes the complete optional reader. Architectural
priority and measured predictive contribution remain separate claims.

## Final controlled result

The final word-cue `/3` artifact is
`blake3:b5ba144fb293358bd45b77ce848f7ad100e26524cfd08586b2a96deea85c081d`.
It learns 12,116 memory features with no drops. Fitting takes 2.232 seconds and
reports 181,633,024 peak process bytes. Calibration covers 108 contexts and
3,936 eligible positions, changes 105 existing bias rows and reduces its
stage-specific cross-entropy from 0.301831 to 0.234505. Subsequent refinement
reports 0.228855 over all 4,064 fit positions; those denominators differ.

| Artifact/control | Prose correct / 96 | Rust correct / 96 | Both-correct pairs, prose / Rust |
| --- | ---: | ---: | ---: |
| Shared baseline / memory disabled | 43 | 39 | 9 / 8 |
| Previous joint word-cue `/2`, 8 epochs | 83 | 33 | 40 / 14 |
| Query features only, word cues, 32 epochs | 96 | 38 | 48 / 17 |
| Uniform-half warmup, word cues, 32 epochs | 43 | 96 | 9 / 48 |
| Final query calibration, exact cues, 32 epochs | 96 | 34 | 48 / 15 |
| **Final query calibration, word cues, 32 epochs** | **96** | **96** | **48 / 48** |
| Final word-cue model, geometry/H4/zeta disabled (each) | 96 | 96 | 48 / 48 |

The final artifact returns all 192 expected first tokens and answer-clause
prefixes. The cases reuse 12 development worlds across two expression families,
four filler lengths and counterfactual swaps. They were repeatedly inspected
during development; they are not 192 independent or final held-out reasoning
tasks. The result demonstrates learned prime-addressed retrieval and query
selection in this scope. It neither establishes added H4/zeta benefit here nor
changes those mechanisms' architectural priority.

The saved-model probe compiles all **96/96** exact Rust continuations in
5.994 seconds including model evaluation. A separate 17.896-second local
assessment links and executes those same 96 unchanged source files; all 96
programs exit successfully with their generated assertions passing. Their
source was inspected: finite assignments, constant filler statements and one
assertion, with four generated color/closing-syntax continuations. No code was
repaired and no expected answer was inserted into generated code. The standalone
execution report is separate from the probe, which remains compile-only.
These results do not establish general code generation or software repair.

A fresh end-to-end Rust run also prepares the same source, constructs the
baseline, fits readout and memory, reloads artifacts, evaluates and compiles
continuations. It completes in 18.425 seconds and reaches 96/96 in both families
with 96/96 Rust compilations. Its artifact is
`blake3:32397e2f6ca7d2e430d7c97f388b95e931f86341e8093e5e3d3a518a9ce57613`.
This command uses 32 epochs for readout fitting as well as memory fitting, so
its readout baseline is different (42/96 prose, 41/96 Rust); it does not replace
the fixed-baseline causal comparison above. These fresh-run programs were
compiled but not separately executed.

```sh
cargo run --release -p uor-r4-core --example native_geometric_memory_probe -- \
  --family mixed --context 512 \
  --construction-documents 256 --readout-documents 64 --development-worlds 12 \
  --fit-positions 4096 --fit-epochs 32 \
  --memory-read --query-context --word-cues \
  --generated-tokens 24 --compile-rust --compiler-cases 96 \
  --max-seconds 600 --output-dir .uor-models/joint-query-context
```

Use a new output directory. The command requires no historical model artifact
or non-Rust data-preparation script. The probe's peak process memory is not
measured internally; the measured CLI fit peaks above are separate evidence.

## Larger-corpus boundary

The same successor was fitted on the preserved larger corpus's 103 readout-fit
documents, starting from its existing 128-token learned baseline. This exposes
147,652 context positions and samples 4,096 fit targets, with 32 epochs per
stage. Feature capacity is increased to 262,144 before launch; 206,127 features
are learned with **zero dropped feature events**. The fit takes 13.946 seconds
and reports 997,638,144 peak process bytes. Its artifact is
`blake3:872c615dcda3b3c0f1f1df8cf3148153161a862736258cfd128e3893d85192d3`.

On the same 103 development documents and 146,668 targets, Full scores
**44,834 / 146,668 = 30.5684%**, versus **53,165 / 146,668 = 36.2485%** with
memory disabled. Geometry-disabled scores 32.3370%. The nine-control evaluation
takes 178.199 seconds. This is a regression despite successful fitting and no
feature drops. It does not establish adequate training exposure, nor does it
justify another architectural demotion or replacing the general baseline.

Two recorded 64-token open-generation examples confirm the limitation. Asked
`The key was red. It is now green. What color is the key? Answer:`, both the
larger baseline and successor produce unrelated fragments and repetitive text.
After `fn sum(a: i32, b: i32) -> i32 {` and a newline, both emit 64 spaces rather
than a function body. These are illustrative open development examples, not a
new benchmark or compiler execution result.

Retain the successful finite-task artifact as a working component. Continue
with broader contextual training and generated-code composition, including
exposure/coverage measurements and useful context windows. Do not keep
re-engineering or re-running the solved color-recall fixture as a prerequisite.
Learned write admission, information retained beyond ring eviction, richer
geometric state/operators and both broad alpha groups remain open under #973.

## Focused verification

The final release binary and probe build successfully. Verification passes
18 native core tests, including address packing, old/new artifact and checkpoint
compatibility, initially starved-route learning responsibility, opposing query
calibration and missing rows. Two runtime tests exercise five model variants ×
nine controls × 1,024 decisions: **46,080 successful prediction/update steps
with zero measured allocations**, and 993 evictions per combination. The `/3`
word-cue fixture uses the same candidate/work counts and buffers as `/2`.
The source arithmetic guard has no offenders or exceptions; this is not a
machine-code/transitive-library proof.

Clippy passes with `-D warnings` for the core, census, probe and root CLI/service
targets. Formatting, the native architecture policy and claim-wording checks
pass. Saved-model evaluation rejects a renamed exact training-text copy before
prediction and rejects a missing model/source argument pair. Full workspace,
WASM, fuzz, legacy teacher-parity and release certification were not run for
this change. Protected CI is reported by its actual steps, not by the legacy
names of its compatibility statuses.

## Preserved baselines and execution limits

The new executable replays the previous exact-cue prose artifact at 86/96,
with 41/48 pairs correct, and the previous joint word-cue artifact at 83/96
prose and 33/96 Rust, with identical artifact identities. The latter already
compiles all 16 sampled Rust continuations; compilation alone does not show
correct variable selection. The shared readout baseline compiles 11/16.
Final-code refitting also reproduces the previous exact `/1` and word-cue `/2`
mixed artifact identities exactly:
`0ceeace32673873a01ee280885e455cd0339c4aaf31147acaf7919b4408f9c8a` and
`9673db8ab87ebc167084cd7f6000c2e2d2e12a91ed29c737467003d8f1548e31`.

This continuation declares 30 cumulative minutes of model fitting/evaluation,
one model process at a time, a 4 GiB per-process memory target and at most
4 GiB additional artifacts/build output. Existing caches, sources and artifacts
are preserved. Model commands are charged to a shared local elapsed ledger;
the surrounding engineering/build work is recorded separately. No paid compute
is used. The resource target is monitored host execution, not an OS sandbox.

Local artifacts are under
`.uor-models/native-joint-learning-2026-09-04/`, including exact source copies,
fit reports, development results, raw generated Rust and compiler diagnostics.
The [scoped result summary](evidence/native_geometric_query_context_973.json)
retains artifact identities, fit/control rows and assertion outcomes. The
charged model work totals **301.277 seconds** of the declared 1,800-second
budget. New local artifacts occupy about 400 MiB; the shared build cache grew
from roughly 4.5 GiB to 4.7 GiB. Probes do not measure RSS internally; the largest
reported CLI fit peak is 997,638,144 bytes. No claim of a hard OS memory limit
or complete process-tree memory census is made.
