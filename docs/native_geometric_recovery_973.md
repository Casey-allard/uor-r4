# Native geometric recovery — #973, 2026-09-04

This record accompanies the owner-directed recovery from `e3084eac47b04b540ccccf54d0547e37fa885882`.
It preserves the new implementation's results separately from the historical
Python experiments. The [canonical plan](integration/project-track.md) owns
the goal; [current-state.md](integration/current-state.md) owns the next work.
No alpha or general conversation, coding or geometric-advantage claim is made.

## Implementation

The new `native_geometric` module connects Rust preparation, count fitting,
separate readout fitting, artifacts, evaluation, generation and checkpointed
sessions. `r4 geometric` exposes the same model through CLI and loopback
workbench. The library is re-exported by `uor_r4_api::native_geometric`.

| Mechanism | Implemented role and scope |
|---|---|
| Prime identities | Full prime token addresses and ordered two-prime context keys; lexical bytes remain reversible. Current leaf assignment is prime modulo 120, not a learned semantic placement. |
| Fixed zeta zeros | Eight channels from the canonical grid, compiled to 16-bit turns. Accumulation alone is commutative; these are quantized phases, not exact real-valued zeros. |
| R4/S3/H4 | Exact ordered H4 window fold and previous-window trajectory, with bounded eviction. The previous fold carries one bounded step beyond the retained window. |
| Exact Z[phi]/orientation | Exact paired-coordinate sums, their variable additive-carrier radial norm, signed chirality/cosine polarity and heatmap/projection equality classes. Individual unit-root radius remains constant. |
| Typed paired/icosian representation | Reuses the existing fixed forward/inverse profile and canonical identities. This represents one H4 carrier and its additive window state; it does not establish two independent H4 factors or an E8 isometry. |
| Learned readout | Conditional score tables plus seven learned groups and supported last-prime query gates. Coefficients are quantized in eighths and executed with bounded shift/add. Fixed and learned artifacts are separate. |
| UOR identity | Typed canonical UOR model address, separate byte-content fingerprint, geometry/profile/operator identities and construction/readout training receipts. Digests never serve as semantic features. |

The native kernel uses 26 named feature addresses and bounded candidate
postings. Its proposed boundary is successful `Session::observe` and
`Session::predict`. Host tokenization, loading, fitting, reports and session
creation allocate and may use floating point. This is not machine-code
certification of the complete executable.

The historical Rust geometric generator also has a separate configurable
1–4096-key context API. Its append is constant work in history length and its
selector compares candidates against at most the configured window. Existing
8-key APIs and historical results remain unchanged.

## Open development corpus and fit

Rust preparation read 524,288 source bytes: the complete 127,231-byte
`crates/uor-r4-core/src/prime_route_attention.rs` and the first 397,057 bytes of
the locally retained public `TinyStoriesV2-GPT4-train.txt`. Documents are UTF-8
chunks of at most 4,096 bytes. The three-way split contains 78 count-construction,
26 separate readout-fitting and 25 open-development documents. Exact duplicate
document contents are excluded. Adjacent chunks can share a topic or story;
this is not independent final held-out evidence.

The count training partition presents 75,448 next-piece target positions.
Tokenization combines learned lexical pieces and byte fallback, so the metric
is neither word accuracy nor byte accuracy. Readout fitting samples 4,096
positions with equal document quotas and uniform positions across each full
document, observing intervening context. It uses eight epochs and receives no
open-development labels. Evaluation has 32,764 target positions and predicts
before observing each target.

The first 128-token run used 65,536 score rows and reached that capacity. It
reported 55,960 dropped feature events out of 1,961,648; those results remain
under `context-128/`. The matched window comparison increases row capacity to
131,072 and associations to 2,000,000. All three matched runs report zero
dropped events; no mathematical mechanism was changed to accommodate this
routine capacity increase.

| Working window | Learned count rows | Count fit + finalization | Model bytes | Peak process bytes |
|---:|---:|---:|---:|---:|
| 32 | 57,541 | 3,611 ms | 27,219,582 | 404,783,104 |
| 128 | 110,632 | 4,101 ms | 33,889,951 | 473,251,840 |
| 512 | 116,042 | 3,226 ms | 36,263,126 | 542,081,024 |

These are single local timings, not a cross-machine performance comparison.
The training checkpoint and exported model are retained for every run.

## Readout finding

| Working window | Fixed full-geometry next-piece accuracy | Fixed geometry-disabled accuracy |
|---:|---:|---:|
| 32 | 0.2289% | 21.5206% |
| 128 | 0.1862% | 21.5206% |
| 512 | 0.1312% | 21.5206% |

The fixed formula sums correlated sparse conditional likelihood corrections.
Inspection and these interventions support a diagnosis of repeated evidence
overwhelming the lexical signal. Larger windows alone do not repair that
readout. This negative result binds this count-fitted representation, formula,
corpus and metric; it does not disqualify the project's geometry.

At 128 tokens, construction-only learned gates reduce their conditional fit
loss from 15.1717 to 2.2564 and improve fit predictions from 7 to 1,021 of
4,096 positions. On separate development documents, full accuracy reaches
28.6717%, compared with 28.7267% when its geometric readout channels and their
candidate postings are disabled.
The global coefficients become `[4,0,0,0,0,0,0]` in eighths, with a few nonzero
geometric query gates among 59 supported query keys. The improvement largely
comes from correcting/suppressing the current geometric corrections and
reweighting lexical evidence. It is not evidence of geometric advantage.

Fit loss is conditional on a bounded shortlist. Only 2,366 of the 4,096 fit
targets were present in the fixed-model fitting shortlist; missing targets
were counted and skipped, never inserted. This is a remaining exposure
limitation of this first readout optimizer, not grounds to change evaluation
labels or claim unseen-candidate learning.

The completed learned-readout comparison is:

| Working window | Learned full next-piece accuracy | Learned geometry-disabled accuracy |
|---:|---:|---:|
| 32 | 26.9259% | 27.5089% |
| 128 | 28.6717% | 28.7267% |
| 512 | 24.7833% | 26.1415% |

All three global geometric gates become zero. Their query-specific lexical
gates and fitting shortlists differ, so these cross-window differences do not
isolate a benefit or cost of longer retained context. Within each artifact,
the geometry-disabled intervention changes both score channels and candidate
offers. It is not a score-only intervention.

## Controlled retained-value result before the learned memory reader

The optional Rust experiment `native_geometric_memory_probe` constructs 256
training documents, 64 separate readout-fit documents, and 96 open-development
cases. Development varies fact order, whether the query asks for an updated or
untouched fact, four filler lengths, and paired changes to the supplied answer
value. No answer grammar or object/color list is implemented in the model.
All three windows use source identity
`010f7a09837ebdf6836cd6eb82da7864e09ace1e9a898c00e04c749ab6368e4b`.

| Window | Answer-value tokens physically retained | Learned-readout correct answers | Counterfactual pairs with both answers correct |
|---:|---:|---:|---:|
| 32 | 42/96 | 24/96 | 0/48 |
| 128 | 72/96 | 24/96 | 0/48 |
| 512 | 96/96 | 24/96 | 0/48 |

The learned model never changes its predicted answer across any of the paired
value changes. Fixed full geometry scores 0/96; geometry-disabled and learned
readout arms score 24/96. All targets are in the candidate shortlist, so this
failure is not a missing target in those query candidates. The largest window
retains every relevant token but still cannot retrieve its value according to
the question. This motivates a learned read from addressable memory, rather
than another storage-only expansion. These finite grammar results are not a
general-language verdict.

The complete experiments take 580, 707 and 687 ms respectively, with externally
observed peak RSS of 42,369,024, 48,349,184 and 49,119,232 bytes. The initial
fixed/learned artifacts and reports remain under `memory-32`, `memory-128` and
`memory-512`.

## Real workbench and initial coding check

The browser loaded the 128-token learned artifact
`blake3:2d392ee97578e2fac418cf1de07d68aced4c7e8b99efeb6b996825da50946f1c`.
For `The blue bird found a small key. It opened`, it generated 24 actual tokens:
` the box. They had a big, red ball. He was very happy. He was very happy. He was very`.
The repetitive continuation is retained as observed behavior, not a useful
conversation claim. Browser reload restored the same session, 35 observed
tokens, H4 states 92/54, exact paired coefficients and all eight phase values.

For the Rust prefix `pub fn add(a: i32, b: i32) -> i32 {`, the same artifact
generated repetitive story text. Compiling the exact continuation with `rustc
--crate-type lib --emit metadata` failed with an unclosed delimiter. No generated
code executed. This single negative check is not a coding benchmark, and it
leaves the coding/reasoning alpha requirement unmet.

The [machine-readable evidence](evidence/native_geometric_recovery_973.json)
retains model/source identities, full development-control summaries, the
controlled answers, browser state and coding outcome. Large model artifacts
and source data stay in the preserved local run directory.

## Expanded native data and learning dose

A second construction uses 2,097,152 source bytes: 712,031 bytes from six Rust
source files and 1,385,121 bytes from the same public story-training corpus.
Preparation produces 311 count documents, 103 readout-fit documents and 103
open-development documents. The configured row/association capacities increase
to 524,288/4,000,000 within the same machine budget. Construction completes
391,725 target positions, 362,345 rows and 2,187,559 associations with zero
dropped feature events. It takes 10.825 seconds, peaks at 1,630,748,672 process
bytes, and exports a 100,057,989-byte fixed model plus a 44,265,520-byte
resumable checkpoint.

Readout fitting increases to 16,384 sampled positions and sixteen epochs,
observing 147,652 context positions. It takes 12.763 seconds and peaks at
1,210,646,528 bytes. Candidate coverage during fitting is 13,316/16,384. The
separate 146,668-position development evaluation takes 62.270 seconds:

| Expanded artifact control | Correct next pieces | Accuracy |
|---|---:|---:|
| Full | 53,165/146,668 | 36.2485% |
| Geometry disabled | 53,010/146,668 | 36.1429% |
| Zeta disabled | 53,169/146,668 | 36.2513% |
| H4 disabled | 53,010/146,668 | 36.1429% |
| Paired channels disabled | 53,030/146,668 | 36.1565% |

The full/off difference is 155 decisions (0.1057 percentage points), while
removing zeta slightly improves this result. Global geometric gates are still
zero; supported query gates and candidate offers account for the remaining
influence. This is scoped development evidence, not a broad geometric-advantage
claim. The vocabulary, population, fitting dose and supported gates differ from
the smaller corpus, so 36.2485% versus 28.6717% is not an isolated estimate of a
data-size effect. It does show that a larger native run fits and evaluates
within the declared local resources.

The same Rust-function prefix was also tried on the expanded learned artifact.
It generated 48 spaces and still failed metadata compilation with an unclosed
delimiter. Better next-piece accuracy therefore did not produce a useful code
continuation in this check. The exact output and compiler result are preserved
under `expanded-coding-check/`.

## Initial verification and boundaries, before the memory reader

- Eleven native core tests passed, including four exact-anchor tests, artifact
  and training resume, state persistence, causal eviction, controls, leakage
  refusal and preservation of the separate fixed-readout artifact.
- Six workbench tests passed using actual TCP requests: native generation,
  session isolation, malformed request rejection, cancellation, persistence and
  artifact binding, and resource limits.
- Six historical/configurable-context tests passed, preserving old APIs while
  exercising longer histories and the 4,096-key bound.
- The allocation census passed for fixed and learned readouts across all eight
  controls: 16,384 successful prediction/observation steps, with 993 evictions
  per case, zero measured allocations and zero allocated bytes. The measured
  fixture uses a 32-token/128-byte ring and a three-candidate/48-byte buffer.

These checks do not establish useful fact retention, coding, general reasoning,
portability, security certification, machine-code arithmetic certification or
alpha. The primary mechanisms remain required work wherever their useful
learned roles are still missing. A fitted model that mostly suppresses its
geometric channels must not silently become the finished project objective.

## Reproduction and preservation

Commands and input formats are in [native_geometric_workflow.md](native_geometric_workflow.md).
The local run directory is
`/Users/casey.allard/uor-r4/.uor-models/native-recovery-2026-09-04`.
It retains source manifests, prepared documents, all fixed/learned artifacts,
count checkpoints, fitting and evaluation reports, and the initial executable.
The source manifest and experiment reports bind the actual data and model
identities. No old `.uor-models` material was deleted or rewritten.

The declared first batch permits 30 cumulative minutes of local model
preparation/fitting/evaluation, one model run at a time, 4 GiB per model process
and 6 GiB new outputs. Reproducible temporary debug build caches were discarded
to stay within storage; source, model artifacts and evidence were retained.
CLI time/RSS checks occur between bounded documents and do not constitute an
OS hard limit; finalization can cross the time boundary. Separate commands are
accounted for at the experiment level rather than pretending they share a
single checkpoint timer.

## Learned memory reader: first objective and dose result

The optional learned reader indexes actual observed values under preceding
prime identities. Recent query tokens visit bounded source-offset postings;
learned sparse tables score query/source offsets, recency, relative H4 transport,
signed orientation and eight relative zeta channels. It neither parses the
experiment grammar nor inserts its expected answers. `MemoryDisabled` suppresses
this reader while preserving the fitted base model. Write admission remains
deterministic and the retained token window is bounded.

The initial joint latent-route fitting objective passes the small unseen-pair
copy test, but the larger controlled experiment exposes its limitation:

| Family | Window | Full correct | Memory disabled | Physically retained values |
|---|---:|---:|---:|---:|
| Prose | 32 | 24/96 | 24/96 | 42/96 |
| Prose | 128 | 24/96 | 24/96 | 72/96 |
| Prose | 512 | 24/96 | 24/96 | 96/96 |
| Rust variable/assertion syntax | 32 | 36/96 | 36/96 | 36/96 |
| Rust variable/assertion syntax | 128 | 24/96 | 24/96 | 72/96 |
| Rust variable/assertion syntax | 512 | 22/96 | 22/96 | 96/96 |

The prose source hash is unchanged from the initial experiment. The Rust source
hash is `abec56f88b5748460f766c12d8494fa024bae284792ccf9a3053326fbfe18c42`.
All splits share a finite grammar and lexicon, with disjoint world IDs and
document bytes. Rust prediction here measures a value after `assert_eq!`; it
does not measure arbitrary program generation or compilation.

All six runs finish in 1.256–2.229 seconds. At 512 tokens the prose fit samples
3,200 positions, including 512 generic end-of-document positions. It changes
neither quantized fitting correctness (2,708/3,200) nor the development answers.
Increasing the same objective from eight to 64 epochs takes another 1.656
seconds and leaves fitting correctness at 2,708/3,200. Its conditional loss
changes only from 0.3914188 to 0.3914157 between those two doses. Development
generation was not repeated for that saturated 64-epoch artifact.

The actual weights are predominantly negative: the joint objective suppresses
the weak memory routes while the base handles easy positions. The next training
correction therefore addresses the objective and its match to deployed maximum
route scoring. These runs remain under `memory-read-{prose,rust}-{32,128,512}`;
the 64-epoch artifact is separate. They do not justify changing the primary
architecture or hiding the failed fit.

The updated executable also reloads the earlier 128-token artifact and restores
its saved session after replacing the process. Its 35 retained tokens, exact H4
and previous-H4 indices, phase turns, paired coordinates and radial state match
the original browser observation. This checks backward compatibility, not
language quality.

## Memory objective correction: matched v2 result

The revised Rust fitter pretrains the pointer over actual memory candidates,
calibrates its shared bias on the fit data, and refines the deployed maximum
route objective. Eight epochs apply to each of the two training stages; the
report names both. No development labels or expected-value insertion enter
training. The same six source/window experiments then complete in 1.390–2.105
seconds each.

At 512 tokens, the prose artifact
`blake3:af4b2e377c01df938262f3e4764e5430d1ddcd96b91e4441e22e434a1da0b802`
answers **86/96** supplied-value queries correctly, compared with **24/96** when
its memory reader is disabled. Both answers are correct in **41/48** paired
value changes, and predictions change in **45/48** pairs. The 32- and 128-token
artifacts remain at 24/96; the fitting population includes delays beyond those
windows. This result supports learning with sufficient retained context, not
an independent estimate of window size because each window has its own fit.

The 512-token geometry-disabled control scores 66/96, H4-disabled 82/96 and
zeta-disabled 91/96. Geometry affects this learned controlled read, while these
particular zeta contributions reduce accuracy. Controls suppress their declared
score/candidate contributions on the same artifact; they do not erase all
prime identity or retrain a replacement. These scoped results do not establish
broad geometric advantage. Errors remain concentrated in some short-delay fact
updates; all 48 untouched-fact answers are correct.

The Rust syntax family remains at 36/96, 24/96 and 22/96 across the three
windows, identical to its memory-disabled controls. Source inspection identifies
a concrete cue mismatch: bare, space-prefixed and newline-prefixed occurrences
of a variable are distinct reversible lexical tokens. The source and query
therefore do not share the same direct memory-index address. The following
implementation corrects cue equivalence separately from exact output tokens;
these v2 artifacts and complete reports remain preserved.

## Word-cue equivalence experiment and default selection

The schema `/2` experiment compiles a cue map that merges only leading-whitespace
variants of a case-sensitive lexical word. It preserves exact decoded tokens,
their complete primes and geometric state. The loader reconstructs the map;
runtime uses a counted table lookup. This repairs an address mismatch, but its
measured learning result is not a general improvement.

On the same separate-family data, the 512-token prose result regresses from
86/96 to 24/96; Rust remains at 22/96. The two smaller windows likewise remain
at their baseline scores. No old artifact is overwritten.

The additional joint experiment trains one artifact on 512 construction and
128 fitting documents across both existing families. It evaluates 192 cases,
96 per family, with family-prefixed IDs and separate family results. With word
equivalence enabled, it produces:

| Joint artifact, 512-token window | Full reader | Memory disabled |
|---|---:|---:|
| Prose | 83/96 | 43/96 |
| Rust variable/assertion syntax | 33/96 | 39/96 |
| Combined | 116/192 | 82/192 |

The combined increase hides a Rust regression. The artifact is
`blake3:9673db8ab87ebc167084cd7f6000c2e2d2e12a91ed29c737467003d8f1548e31`;
the complete joint run takes 3.171 seconds. These are finite grammar tasks,
not independent language/coding assessment.

The final API therefore preserves **exact token cues by default** through
`fit_memory_read`, retaining the useful v2 implementation. Word equivalence
remains an explicit `fit_memory_read_with_word_cues` / `--word-cues` experiment.
Its schema and map bind the effective mode without changing the serialized fit
configuration. Both old memory schemas remain loadable. Enabling a plausible
mechanism is not treated as success when its matched result regresses.

## Final focused implementation verification

- Native core: **14/14 passed**, including exact anchors, training and artifact
  reload, default exact cues, explicit word-cue mapping, forgery rejection,
  legacy artifact compatibility, causal eviction and checkpoint continuation.
- Native CLI/service: **12/12 passed**, including seven actual TCP service tests
  for generation, cancellation, isolation, persistence and storage admission.
- Runtime census/source guard: **2/2 passed**. Three models across nine controls
  complete **27,648 predict/observe steps** with **zero allocations and bytes**,
  including 993 evictions per combination. The alias-enabled fixture retains
  23,552 bytes of memory backing buffers; Full admits 16,251 memory candidates,
  performs 292,518 score lookups and 18,407 bounded cue-table reads.
- The six historical/configurable-context checks and existing graph-format
  contract-version agreement check pass. The old 8-key defaults remain intact.
- Focused core/CLI/service/probe clippy with `-D warnings`, Rust policy checks,
  formatting and claim-wording checks pass. The WASM library compiles with 26
  existing warnings in unchanged historical files.

The source guard covers the actual observe/predict regions and complete memory
runtime with no arithmetic exceptions. It does not inspect all transitive
standard-library operations or certify generated machine code. These checks
support the implemented kernel and interfaces; broad capability, final release
QA, security/portability qualification and independent final alpha assessment
remain unrun. The protected queue reports its actual native checks separately
from compatibility-only status acknowledgements.

## Final reproduction, increased training dose and capacity

The final executable reproduces the exact-cue prose artifact
`blake3:af4b2e377c01df938262f3e4764e5430d1ddcd96b91e4441e22e434a1da0b802`
and its 86/96 result byte-for-byte. It also reproduces the joint word-cue
artifact `blake3:9673db8ab87ebc167084cd7f6000c2e2d2e12a91ed29c737467003d8f1548e31`
and its 83/96 prose, 33/96 Rust result. These checks confirm that retaining exact
cues by default did not change either preserved experiment. The final exact-cue
joint fit at eight epochs per stage scores 43/96 prose and 39/96 Rust, identical
to its memory-disabled baseline.

Increasing the joint memory fit to **32 epochs in each of its two stages**,
with the same count/readout baseline and 128 fitting documents, yields:

| Joint 512-token artifact | Prose correct | Rust correct | Both prose answers correct | Both Rust answers correct |
|---|---:|---:|---:|---:|
| Memory-disabled baseline | 43/96 | 39/96 | 9/48 | 8/48 |
| Exact cues, 32 epochs per stage | 82/96 | 30/96 | 39/48 | 11/48 |
| Word cues, 32 epochs per stage | 82/96 | 32/96 | 40/48 | 13/48 |

Each fit uses 4,064 positions with zero dropped feature events. Exact fitting
takes 2.070 seconds and 177,389,568 peak process bytes; word-cue fitting takes
2.096 seconds and 178,126,848 bytes. Their 192 first-token development queries
run through the actual CLI and take 40.023 and 39.857 seconds respectively.
The separate artifacts are
`blake3:a8ade719479ad427681fa2fb6e46eb6d337175e297bbbe8f65526d2d31a9cba7`
and `blake3:0d9aad9320b00b642f7904724f17d655bc84949ffabf15b759a34322a13a27aa`.
More training improves prose at the cost of Rust correctness. No aggregate
score is used to promote either artifact as satisfying both goals.

The larger open Rust/TinyStories corpus also receives an exact-cue memory fit
on its 103 fitting documents, distinct from construction and development.
The initial 65,536-feature capacity drops 35,937 feature events. Development
accuracy falls from **53,165/146,668 (36.2485%)** with memory disabled to
**48,791/146,668 (33.2663%)** with the reader active. The generated Rust
continuation still consists of whitespace and fails an actual compiler check.

The next run increases only the feature capacity to 131,072, retaining the
same input, 4,096 sampled fitting positions and eight epochs per stage. It
learns **77,773 features with zero dropped events** in 11.884 seconds, with
1,341,030,400 peak process bytes. Its artifact is
`blake3:a161574b235bbf70e829406540d75750718e1bd23e9ee1a56aa55d219c48b0e8`.
The nine-control development evaluation takes 166.209 seconds. Full accuracy
falls further to **48,054/146,668 (32.7638%)**; memory-disabled remains exactly
53,165/146,668. Geometry-disabled scores 53,171/146,668. Evaluation peak RSS was
not separately measured, and a code-generation check was not repeated for this
capacity artifact. Removing the saturated feature table does not resolve the
generalization failure.

These runs implement the revised budget policy in practice: context, source
exposure, fitting dose and table capacity were expanded locally where the
preceding evidence justified a distinct comparison. They do not freeze these
values as permanent project limits. The next model work remains balanced joint
learning, learned write/selection/composition and actual code feedback, keeping
the useful exact-cue prose result and all negative artifacts as comparisons.
General conversation, reasoning and coding alpha remain incomplete.
