# Native completion after typed-value emission — #973

Date: 2026-09-05. Status: bounded numeric-response completion improves on reused open development; general capability and matched-refit geometry comparison remain unqualified. The [project
plan](integration/project-track.md) owns the objective.

## Observed cause and inherited checkpoint

The preceding typed-value step merged through protected [PR #1130](https://github.com/UOR-Foundation/uor-r4/pull/1130) at
[2f6c080c266d01683771b84f405c8c353cc7653b](https://github.com/UOR-Foundation/uor-r4/commit/2f6c080c266d01683771b84f405c8c353cc7653b). Its [record](native_geometric_typed_value_973.md) and [bound
evidence](evidence/native_geometric_typed_value_973.json) preserve both fits. The `/2` artifact selects all 24 tested numeric prefixes and all four binding pairs correctly, but only 2/16 prose and 0/16
Rust responses are complete. All twelve numeric Rust cases fail at the first post-numeral byte or EOS; eight numeric prose cases fail there, two later, and two are exact. Both sets of twenty saved
generated Rust files fail compilation. These are reused open-development results, not general conversation, coding, reasoning or geometric advantage.

The typed fitter supervises operands and numeral continuation; the ordinary decoder handles punctuation, code closure and stopping. Typed `77` emits two byte tokens; the ordinary tokenizer can encode the
same bytes as one lexical token. This history difference is observed, but its causal share relative to ordinary readout weakness has not been isolated. The implemented component learns continuation from
the history actually produced by typed emission. It preserves the stronger `/4` memory reader and the `/2` value head. Earlier `/5` response-state fits remain separately scoped negatives.

## Representation and causal execution

`completion_types.rs` defines optional `uor-r4.native-value-completion/1` state. An anchor is created only after the final selected numeral byte is actually observed, using the preceding typed prediction's
matching provenance. It retains the derived write ID, Copy/Add action, post-numeral observation count, cumulative typed H4 pose, eight fixed-zeta phase channels and frozen query-end cue prime. The state
also retains the last two actual token IDs, current observation count, completion step, active flag and last observed action. Predictions are transient.

The relative H4 feature is `inverse(anchor.pose) * current_typed_pose`, with a separate signed-orientation lookup. Eight phase differences use wrapping u16 subtraction and their upper four bits. Other keys
represent a bias, last-token prime, ordered last-two-token primes, query-end prime, query/last-prime pair, and operator/step. At most sixteen keys address learned rows. Prime identities are addresses, not
a semantic distance. The inherited paired-H4/icosian, exact `Z[phi]`, radial and UOR identity fields retain their existing roles; this head adds no new paired-coordinate or radial ranking term.

The bounded state retains an exact anchor and recent ordered observations; its score features deliberately quantize phase differences and compress the suffix to two token identities, relative geometry and
a step. It does not retain an arbitrary suffix transcript, syntax tree, semantic summary or dependency graph. The typed store still retains at most sixteen exact values and bounded word cues; eviction
loses older records and unbounded source detail. This is not learned multiscale semantic compression or evidence of general reasoning.

`completion_runtime.rs` unions up to four postings from each matched feature row with sixteen global postings, retaining at most sixteen distinct candidates in deterministic offer order. Candidates are
individual byte tokens or EOS. Sparse integer row scores compete with a zero Base action: a positive winner offers its token at the already computed baseline score plus the learned score. Equal positive
scores choose the smaller token ID; nonpositive scores retain Base. No serving suffix string, answer template, target lookup or provider response exists.

Observation always updates the actual token history. A matching pending decision credits Emit/Stop; a mismatching byte credits Base and advances from that byte, without committing an unobserved prediction.
EOS clears the response anchor. After 32 actual completion observations the head clears its anchor and disables itself without forcing EOS; ordinary generation can continue. Explicit response boundaries
reset progress while preserving actual token history. Empty request continuation must retain the active response rather than start a new one.

## Offline fitting and declared comparison scope

The bounded run uses the unchanged `/2` source: 192 fit pairs, the same 32 development examples, and four source-name binding pairs/eight Full runs. Those development cases informed design and remain
**OPEN development**. Configuration is 24 epochs, learning rate 0.1 and at most 4096 fitting positions; the artifact caps feature rows at 4096 and token associations at 32768. Shared templates and disjoint
document/world IDs do not establish held-out transfer.

`fit_value_completion` runs the frozen value model to its actual numeral end. The emitted bytes must match the entire canonical integer prefix of the raw response. Wrong, partial or differently spelled
numbers are recorded as upstream failures; `17` cannot be repaired into target `176` by learning an extra suffix digit. NoWrite examples are skipped separately. Only then does offline fitting observe
authored suffix bytes and EOS, recording individual next-byte frames. Position limits, overlong responses and unavailable candidate targets are reported.

Postings come from bounded training frequency counts; floating-point sparse optimization learns token scores versus Base. Epoch selection evaluates quantized weights with serving tie rules. The artifact
binds its frozen baseline identity, configuration and ordered source receipts. Ordinary tokenizer, geometry, readout, memory rows and typed weights remain unchanged. Offline floating point, gradients and
exponentials are permitted training operations; inference remains integer/table.

Full is compared with completion disabled, completion geometry disabled, and values disabled. Completion-disabled retains state but offers no completion token; the geometry control removes only this head's
H4/orientation/phase features. These are within-artifact controls, not separately fitted matched baselines. Their generated lengths and total work can differ. Final held-out evaluation, broader grammar
coverage and a geometric speed/quality advantage remain unclaimed.

## Complete work and persistence boundaries

The following are conservative **per active prediction** source bounds, not measured latency:

| Completion operation | Bound |
|---|---:|
| Feature queries / row-key comparisons | 16 / 208 at 4096 rows |
| H4 / orientation reads; phase subtractions | 2 / 1; 8 |
| Token-prime and H4 row-base metadata reads | 3 |
| Posting offers / distinct retained candidates | 80 / 16 |
| Candidate duplicate comparisons / writes | 1280 / 16 |
| Candidate evaluations / score lookups | 16 / 256 |
| Score-token comparisons | 2560: at most 10 per lookup over 257 byte/EOS tokens |

Each observation updates three history scalars; anchoring copies five scalar fields and eight phases after two record/provenance metadata reads. Fixed local arrays hold sixteen features, sixteen candidate
tokens and sixteen row indices (448 bytes of logical array storage on the current 64-bit layout), in addition to persistent state, locals and caller frames. `StateView` reports persistent `CompletionState`
size separately. Neither number includes model tables or RSS. Counters cover named operations, not every branch, fixed-array initialization, copy, lookup address calculation, shortlist insertion or
serialization instruction.

All ordinary decoder work still executes before the completion offer: 26 base feature queries, posting/scoring work and at most 128 memory routes in the matched `/4` artifact, with occurrence-feature
union/deduplication, H4/orientation/phase, radial tables and shortlist moves. Typed ingestion and numeral work also remain: initial/retry selection visits 272 slots, up to 256 valid Copy/Add proposals and
240 checked additions, with up to 60 features, 64 token-cue and 128 whole-word comparisons per proposal. Every admitted Add executes before ranking. At full occupancy this includes 15360 feature lookups
and up to 1048576 word-byte comparisons. NoWrite repeats the typed search on later predictions; a completed write avoids that search, but keeps ordinary prediction work. The [typed
record](native_geometric_typed_value_973.md) details remaining ingestion, rank, copy and numeral bounds. Postings admit tokens; they do not select which arithmetic expert executes. No expert or transformer
compute saving is established. Host tokenization, artifact loading and snapshots are outside the serving-kernel operation counts and still contribute to total cost.

Session schema `/4` requires completion state only for completion artifacts; older `/1`–`/3` field/serialization laws remain. Restore first validates typed state, then checks derived provenance, exact
numeral completion, anchor/query identity, actual token history, counters and relative frames. While active, its endpoint and fewer than 32 suffix observations fit the typed 32-entry ring even after
ordinary-window eviction. Transient predictions are omitted; candidate workspaces are fixed arrays. Evicted external source truth remains unauthenticated. Focused tests cover real-fit persistence,
malformed snapshots, mismatching bytes, empty continuation, allocation boundaries and EOS/cap behavior. Report actual test/storage/RSS scope.

## Measured open-development result

The unchanged 192 fit pairs produced 160 matched numeric rollouts and 32 skipped NoWrite examples, with no upstream failures, truncations or dropped row/association events. All 720 suffix/EOS targets
entered the candidate set and all 720 were selected correctly using exported weights. The fit exported 192 rows and 289 individual token associations at epoch 24, learning rate 0.1; loss was
0.007023032880161844. These are construction and reused-development results.

| Same-artifact development arm | Correct numerals /24 | Prose exact /16 | Rust exact /16 |
|---|---:|---:|---:|
| Full | 24 | 12 | 12 |
| Completion disabled | 24 | 2 | 0 |
| Completion geometry disabled | 24 | 0 | 0 |
| Values disabled | 0 | 0 | 0 |

Full completes every numeric primary response and both complete responses in all four binding pairs. All eight NoWrite primary cases still fail complete equality: this head activates only after a typed
write. The CLI loads the same artifact and emits exact `77);\n}\n`; every probe Generation field, including tokens, stop, state, value/completion traces and work, matches the CLI. Extra CLI metadata is
outside that shared object comparison.

All twenty generated Rust records were inspected and compiled without source repair. Sixteen compile, execute and pass their generated assertions; the four NoWrite records fail compilation. The sixteen
successful records contain fourteen distinct source hashes because two original binding cases duplicate primary cases. This is finite generated-program evidence, not sixteen independent novel programs or
general Rust competence.

Full creates 24 anchors and commits 108 completion tokens, including 24 stops, with zero mismatches, Base suffix steps or step-limit events. Its head performs 1728 feature queries, 648 candidate
evaluations and 10248 score lookups; persistent completion state is 96 bytes, separately from the 448-byte logical local arrays and all inherited state. Ordinary work still includes 67079 candidate
evaluations and 1637751 base score lookups, plus the separately counted memory work. Shorter correct outputs change aggregate work; these totals establish no matched-latency or per-token speed advantage.

The independent source/report audit finds the same six candidates in both Full and completion-geometry-disabled at every active decision: newline, EOS, right parenthesis, period, semicolon and closing
brace. All six are in global postings, the candidate limit is sixteen, and no candidate is dropped. Removing the jointly fitted H4/orientation/phase terms therefore changes scoring without losing these
candidates; numeric selection remains intact. Full's suffix success and the control's 0/32 complete responses establish that those terms are load-bearing in this fitted head. They do not isolate H4 from
orientation or phases, compare separately refitted competitors, or establish geometric superiority. Only two authored suffix forms are learned in this population, so unseen response-form transfer is
untested.

## Preservation, checks and resource stop

Artifacts remain under `/Users/casey.allard/uor-r4/.uor-models/native-typed-value-2026-09-05`. The model is `completion-fit-1/model.json`, CID
`blake3:a1fa0314924fb324f994e449cce6e69793d6c4df6102353a959363cb766009ff`, bound to the prior `/2` artifact. Its directory retains source, options, fit report, all control outputs, binding outputs and
generated Rust. `completion-rust-source-identities.json` and `completion-binary-identities.json` bind code and executables; `completion-compile-execute/report.json` retains compiler/execution identities
and outcomes. Newly created per-case compiler products were removed after their identities, diagnostics and execution output were retained; source, model, evidence and inherited material were preserved.

The source is byte-identical to the prior source. Model fields change only for completion and artifact identities. Completion-disabled reproduces every prior Full noncompletion Generation field on all 32
primary cases except the explicitly different control label. Replaying each prior value artifact under the new executable preserves full Generation objects on 32 primary plus eight binding runs per
artifact, including bytes, tokens, traces, state and work. The `/1` reference is its prior final-source replay, preserving the already documented older layout/telemetry correction. The first strict
comparison's CLI metadata/control-label differences remain recorded; they are not relabeled as output failures or silently discarded.

Actual checks passed 79 native core tests, then four newly wired completion runtime tests; four source/allocation checks, three context tests, four probe tests and seventeen CLI/service tests. The release
build passed in 316032 ms. These counts retain their actual invocation scope, rather than claiming a single 83-test run. The completion allocation check covers commit, mismatch and cap behavior after
initialization. Final formatting, architecture-policy and claim-wording checks pass. Broad release QA and final held-out qualification remain NOT_RUN; successor protected delivery is a separate live GitHub result.

Through the CLI sample, cumulative model work is **903526/1800000 ms**, leaving **896474 ms**: 894645 inherited plus 8881 for completion fitting/evaluation, generated-program compilation/execution,
prior-artifact preservation and CLI replay. Engineering builds/tests remain separately logged. Fit RSS peaks at a sampled **314228736 bytes**, from three direct-process samples. Compile-wrapper RSS
measures Node rather than descendant compiler/program peaks; short preservation commands have no RSS samples. Periodic sampling is not an exact-peak guarantee.

At **08:22:21 UTC**, accounted storage is **4136693760/4294967296 bytes**. After the unchanged **134217728-byte** stop margin, only **24055808 bytes** remain for growth. The inherited accounting still
charges current target allocation, both artifact roots, measured worktrees and the source/metadata reserve, using only the previously audited obsolete-target credit. The latest measured build peak
increment is **29683712 bytes**, exceeding available growth by **5627904 bytes** before additional outputs. The next implementation/build is therefore blocked under the current storage envelope despite
remaining model time; no limit increase or new cleanup credit is assumed.

The next bounded decision is a separately fitted comparison with the completion geometry terms removed, using the same source, candidate constraints and fitting budget, followed by response-form transfer
checks with authored targets kept separate from fitting. This distinguishes dependence of the present fit from a relative geometric benefit and tests whether completion extends beyond the two existing
forms. Both are NOT_RUN at this checkpoint and must wait for an explicitly viable resource envelope. The existing successful mechanism and the failed controls remain preserved.
