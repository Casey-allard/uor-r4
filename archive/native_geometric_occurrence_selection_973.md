# Native local-path occurrence selection — #973

**2026-09-05; open-development implementation and measurement.** Memory schema
`uor-r4.native-prime-relative-memory-read/4` compares local source/query H4 and
fixed-zeta paths and combines unique evidence reaching the same retained
occurrence. On a matched corrected source, it improves prose first-token
selection from 15/32 to 20/32 and exact prose from 5/32 to 6/32. Rust exact
generation remains 0/32. Retain the implemented selection mechanism and its
bounded evidence; useful joint conversation/coding and alpha remain unmet.

The [mechanism map](native_geometric_mechanism_map_973.md) connects this reader
to prime identities, exact geometry, retained state, training and serving.
The [machine-readable record](evidence/native_geometric_occurrence_selection_973.json)
contains artifact identities, matched families, controls, actual response
examples, work counters and resource evidence. The preceding
[resumable fitting record](native_geometric_resumable_memory_973.md) remains
unchanged at its original source and artifact scope.

## Observed cause and implemented correction

The previous reader could admit the correct retained token while choosing an
unrelated route. Its H4/phase relation compared cumulative source and query
states; intervening history could dominate that relation. It also chose the
best individual route, without pooling different cues supporting the same
occurrence. Neither defect justified wider token admission or more unchanged
training exposure.

For a retained source cue `s`, value occurrence `v`, query cue `q` and current
state `c`, `/4` uses ordered H4 table products:

```text
S = inverse(s.pose) * v.pose
Q = inverse(q.pose) * c.pose
relative = inverse(S) * Q
phase_delta[j] = (c.phase[j] - q.phase[j]) - (v.phase[j] - s.phase[j])
```

Phase subtraction wraps in full `u16` precision before binning. H4 order and
signed orientation are retained. These are local path comparisons, not a new
semantic distance or an independent paired-H4 state.

The existing bounded prime postings still admit the flat routes. `/4` groups
them by the exact retained sequence position, forms a sorted union of unique
feature addresses, and sums each learned feature once. The token prior and
shared bias each contribute once per occurrence. Equal token IDs at different
positions remain separate occurrences. No expected source-position label,
parser, new cue family or wider candidate limit supplies the answer.

Rust fitting uses this same occurrence union for target credit, calibration
and final refinement. Its floating-point training weights are quantized into
the integer feature tables used by `Session::predict`. The CLI opt-in is
`r4 geometric fit-memory-stream --compose-occurrences`; the library entry is
`MemoryReadTrainer::new_with_occurrence_composition`. Resume checks bind the
composition mode, source, supervision, configuration and baseline.

Schemas `/1`–`/3` retain their flat scoring and historical global-relative
geometry, and historical artifact formats remain loadable. The new occurrence
buffers are allocated only for `/4`. A retained sequence field increases the
flat candidate's host storage; byte-preserved historical model serialization
does not imply identical host workspace size.

This is composition of selection evidence. It does not compute a new numeric
value, learn write admission, retain the selected occurrence as a persistent
query register, or write an intermediate result into geometric memory.

## Source correction and comparison boundary

The historical joint source `/1` appended world-specific test inputs and
assertions to two Rust function tasks without putting those inputs in the
prompt. Some identical prompts therefore had conflicting complete targets.
The source `/2` repair explicitly supplies each test input in the prompt and
rejects conflicting prompt/target pairs. The old source and measurements remain
preserved and loadable. This correction does not retroactively revise their
numbers or make different source versions directly comparable.

Both current readers are newly fitted on the same `/2` source and readout
baseline. The source has 512 construction documents, 512 fit documents and
64 open-development documents, balanced between prose and Rust. World IDs and
exact document bytes are disjoint; templates and finite grammar remain shared.
Development deliberately includes fresh names, query paraphrases, reversed
function order and reversed repair direction. This is not a final held-out
population. Maximum development prompt context is 83 tokens including BOS,
inside the configured 512-token ring; the observed failures do not test
retention beyond eviction.

Count construction uses 32 candidates, 131,072 rows and 1,000,000 associations.
Readout fitting uses 16,384 positions and eight epochs. Each memory fit sees all
6,548 response-plus-EOS targets, with 256 live examples, eight epochs per
learning stage, eight query tokens, four source offsets, four postings per
address, 128 candidate routes and a 262,144-feature cap. `/4` learns 60,705
features; matched `/3` learns 60,812. Neither drops feature events. Both expose
the same 6,214 shortlisted and 3,172 memory-reachable fit targets.

| Identity | Content-bound value |
| --- | --- |
| Readout baseline | `blake3:1e6a6a9dfad5e3c4e41049c8a1cecda2d8dcb9821b5f7428e92f50879795b279` |
| Ordered fit source | `blake3:ed88a08fef099219a6ed0182bc97fe2242b82be05ebe376034b15f1719c74c78` |
| Supervision | `blake3:f6763d0f61825a4df758e7f15093e7ddebbad8cac35c99c85f739bdd15c7699c` |
| Matched `/3` artifact | `blake3:7d23c8e08bf5a6d2b9f9ca164d3a1259173ea5420c2b690565acdc1f356cf624` |
| Occurrence `/4` artifact | `blake3:9bfcd5f46ced60dd70aff50637cde64ffd8312d9e10c1430bbc7f22761f78264` |
| Evaluation source file | `blake3:848a5f8f9c8b4d4ceb5d0d0c1583886b5ab90f41fdb9a97a9327b9d288ac16b1` |

## Actual generated behavior and controls

Generation uses the saved native artifact and a 96-token output ceiling. Exact
complete responses, first predictions and teacher-forced completion pieces
are different measurements.

| Reader/control | Prose first /32 | Prose exact /32 | Prose completion pieces /320 | Rust first /32 | Rust exact /32 | Rust completion pieces /504 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Matched `/3`, full | 15 | 5 | 189 | 0 | 0 | 321 |
| `/4`, full | 20 | 6 | 195 | 0 | 0 | 328 |
| Shared memory-disabled | 13 | 4 | 167 | 0 | 0 | 278 |
| `/4`, geometry-disabled | 21 | 4 | 180 | 0 | 0 | 293 |
| `/4`, H4-disabled | 21 | 4 | 184 | 0 | 0 | 296 |
| `/4`, zeta-disabled | 20 | 7 | 203 | 0 | 0 | 328 |

Full `/4` and `/3` have identical first-target memory admission: 16/32 prose and
8/32 Rust. The matched gain therefore does not come from widening admission.
The package changes both local-path features and occurrence aggregation; this
comparison does not isolate their individual effects.

Full `/4` gives two extra exact prose responses relative to its geometry/H4
controls, while those controls have one more correct first token. The extra
exact outcomes concern continuation and termination, not uniformly improved
first selection. Controls also disable baseline geometric terms, so they do
not isolate only the new reader term. Zeta-disabled improves exact prose and
completion prediction: this run establishes no positive zeta increment.
These scoped results neither prove broad geometric superiority nor reject
fixed-zeta geometry as an architectural mechanism.

Actual responses expose the remaining distinctions:

- An updated-fact prompt says suri's badge was gold and is now white. Full
  `/4` generates ` white.\n` exactly. Across eight updated-fact cases it gets
  four first tokens correct, all using the familiar “What color” form; the
  four “Tell me the current badge color” paraphrases fail first selection.
- Two-fact first selection succeeds for four familiar-name cases and fails
  for all four fresh-name cases. A correct first value does not establish
  coherent production of the second fact.
- All eight Rust repair targets are present in retained memory, yet every
  first prediction is wrong. A prompt requiring subtraction can still select
  the seen ` +` continuation. This is a learned selection/intent failure.
- The sixteen dependency-update/function-composition cases require derived
  numeric first answers. None has a target memory route, and none predicts
  the first answer correctly. Retained-token copying cannot supply an absent
  computed value; a write/computation mechanism is still needed.

All 16 deterministic Rust function-prefix measurements are also wrong. Of
eight sampled generated Rust programs, two `/4` outputs compile, but both
assert `120`: their supplied inputs require `86` and `23` respectively. They
are not semantic passes. Execution is `NOT_RUN` under the existing rule that
only exact authored safe programs execute. The other six fail compilation;
two actual compiler-feedback repair attempts also fail compilation. Matched
`/3` compiles none of its eight sampled programs. Compiler checks use pinned
Rust 1.97.1 and the unchanged generated source.

## Bounded work, storage and preserved budget

At the fitted configuration, optional memory workspace is 254,720 bytes for
`/4` versus 212,736 for current `/3`. The `/4` difference is 5,120 bytes of
composed candidates plus 36,864 bytes of unique-feature storage. Both are
preallocated from the 128-route bound. This does not include every base model
or session allocation and is not the historical `/3` workspace measurement.

Before removing redundant flat-route score lookups, full `/4` generation across
64 tasks emits 2,290 tokens, observes 6,420 tokens
including prompts, processes 232,097 flat memory routes and 62,552 composed
occurrences, and records 6,642,332 memory score lookups plus 31,467,694
composition comparisons. Matched `/3` emits 2,958 tokens and makes 5,189,346
memory score lookups. Output lengths differ; these counts do not establish an
equal-token speedup. These original work counts remain preserved with the first
evaluation; final replay after the lookup correction is recorded separately.
The sorted bounded union adds measurable work even though
the kernel uses integer/table operations and does not allocate during scoring.

The final runtime removes unused flat-route weight lookups for `/4`; its
diagnostics still retain the admitted routes and features, while only the
unique occurrence union reads fitted weights. This bounds weighted feature
lookups by 18 per admitted flat route, without changing the fit, artifact or
historical schema scoring. Replaying the same saved artifact on all five
controls preserves all 320 primary cases: first prediction scores/tokens,
generated bytes/token IDs, final state, diagnostics and family metrics match.
Compiler/repair paths contain different output directories and are outside this
equality assertion; their fresh assessments are retained separately. Full `/4`
memory score lookups fall from 6,642,332 to **2,464,586**, a **62.8958%** reduction
in those counted lookups at identical primary behavior. Composition comparisons
remain 31,467,694. This is an operation-count improvement, not a claim of the
same percentage reduction in total latency or of a comparison against `/3`
at equal generated length.

The `/4` model is 9,780,190 bytes and its checkpoint 4,040,987 bytes. Matched
`/3` is 9,789,795 bytes with a 3,993,238-byte checkpoint. Internal peak fitter
RSS is 154,664,960 and 153,468,928 bytes respectively; process samples are not
an OS process-tree guarantee.

The inherited 1,800-second cumulative model allocation begins this continuation
at 716.243 seconds. The first eight preparation/fitting/evaluation commands
charge 65.380 seconds, reaching 781.623 seconds. Final saved-artifact replay
adds 3.106 seconds. Across nine charged commands the total is therefore
**784.729 seconds used and 1,015.271 seconds remaining**.
Engineering checks are reported separately. One model process, a 4 GiB RSS
target and the original 4 GiB cumulative new-storage allocation remain in force;
no paid compute or budget increase was used.

The storage audit recovered an exact initial shared-target snapshot of
4,695,256 KiB and charged all three native continuation worktrees. The ordinary
allocated-size sum exceeded 4 GiB before mitigation; this is preserved as a
historical overage, not retroactively reported as compliant. Every old worktree's
contents and history were preserved. In the initial mitigation, 14,032 unchanged
tracked files in the new worktree became
verified independent APFS copy-on-write clones; SHA-256 and Git diff/status
remained identical. Clone credit is capped at the observed 1,164,636,160-byte
free-space increase. Then exactly two obsolete debug core hash families were
removed from the recreatable shared build cache, releasing an observed
759,287,808 bytes while current release/debug products and all source/model
artifacts remained unchanged. The dated storage records are snapshots;
subsequent builds and output continue to consume the inherited allocation.

Later CLI/policy checks rebuilt those debug cache families and again approached
the storage boundary. A second bounded mitigation removed the completed
policy-only core library inputs while retaining its linked executable and
current test/CLI/release products. It then made 14,170 byte-identical tracked
files in the preserved native-joint worktree independent APFS clones of the
resumable-worktree copies. All six differing historical source/doc files,
untracked/dirty material and Git references were retained. Both worktrees remain
full and independently writable; physical allocation/inode metadata changed,
while source bytes, file modes, modification times, clean Git status and empty
diff were verified. The second clone credit is capped at its observed
1,197,715,456-byte free-space gain. Exact manifests and all earlier overage
snapshots remain in the artifact directory.

The final pre-delivery snapshot after release/replay and focused checks is
2,747,199,488 bytes of known cumulative physical footprint, with 1,547,767,808
bytes (1,476.066 MiB) remaining inside the 4 GiB allocation. It includes all
three continuation worktrees/artifact roots and shared-target growth after both
measured APFS credits. Subsequent Git metadata and output remain subject to the
same allocation; this sampled accounting is not a hard peak guarantee.

All raw evidence remains under
`.uor-models/native-geometric-composition-2026-09-05/` in the main checkout:
`source/`, `supervised-source/`, the count/readout/memory models and checkpoints,
both evaluation directories, `commands.jsonl`, `compact-evidence.json`,
`storage-clone-{audit,manifest}` and `obsolete-debug-cleanup-{manifest,result}`.
The subsequent `policy-core-cleanup-*`, `native-joint-storage-clone-*` and
`cumulative-storage-after-policy-and-clone.json` retain the second mitigation.
`occurrence-final-evaluation/report.json`, `compare-final-replay.mjs` and
`final-replay-comparison.json` retain the final runtime replay and its exact
comparison against the original responses.
The original inherited ledger remains in
`.uor-models/native-joint-learning-2026-09-04/model-time.json`.

## Verification and next implementation boundary

Completed focused checks include 32 native core tests, two allocation tests,
eight joint-probe tests and 13 CLI/service tests. The 13 CLI/service tests ran
before the redundant-lookup correction; final native core, allocation and
joint-probe tests, the release build, core Clippy, formatting and source-policy
check ran afterward. They cover local-path
arithmetic/order/invariance, unique occurrence evidence, bounded buffers,
historical schema behavior, resumed fitting, source consistency and the actual
CLI/service boundary. Core library/test/example Clippy passes with warnings
denied; independent final source/report review has no actionable findings. Exact
commands and outcomes are in the machine-readable evidence. Protected queue
checks and delivery are reported separately when they run; local checks alone
are not merge or capability evidence.

The next mechanism must address the failures still observed here: retain and
use query intent across generated spans, and compose/write values that are not
available as retained tokens. Query-conditioned learned writes, intermediate
state/value operators, retention after eviction and broader joint behavior
remain open. A higher fit score, more cue rows or these few exact prose gains
does not complete those mechanisms or close #973.
