# Learned contextual language relation selection — #973

**Empirical result: PASS_CONTEXTUAL_LANGUAGE_RELATION.** One output-supervised fit produces all 128 development answers and selects all 128 correct source/word occurrences. All eight complete intervention families pass. Development names, relation verbs and answer strings are absent from training. The shared writer and earlier parameters remain unchanged; normal model `15baec48` is retained, with no promotion or issue closure.

[Machine-readable evidence](evidence/native_geometric_language_relation_973.json) binds the source, executable, compiler, artifact, data and complete sealed report. This continues the [ordered-state checkpoint](native_geometric_ordered_state_973.md) by removing supplied exact query keys from the new relation task.

## Inputs, learned computation and its limits

Inference receives four raw sentences and a raw question. A generic Rust lexer finds maximal lowercase ASCII word runs and preserves their byte offsets. It has no dictionary of people, verbs or question forms, no stemmer and no subject/object parser. This experiment requires four words per sentence/question, words at most 16 bytes and input records at most 128 bytes. The fixed syntax is an explicit experimental restriction.

The reader considers all 16 word occurrences. Its representation contains 16 ordered question-word/source-word zero-distance comparisons and four candidate-position indicators. Word geometry uses the retained signed H4 prefix encoder and hemisphere-signature Hamming metric. Canonical prime token identities and occurrence offsets remain separate; they are not semantic distances or output labels. The comparison layout is fixed, while the conditions selecting an occurrence are learned.

Training derives output-compatible candidate occurrences by executing the retained learned byte/EOS writer and comparing emitted tokens with the training answer. Correct source/word labels are reserved for evaluation. Half of the 512 training examples have more than one output-compatible occurrence; four feature classes contain both compatible and incompatible candidates. The learner therefore cannot simply label every copy of an answer as the correct route.

A bounded greedy search considers 1,350 conjunctions with one to three of the 20 predicates. It retains a rule only if the resulting shared selector introduces no wrong-output candidate, preserves already uniquely answered examples and increases uniquely answered training coverage. At most eight rules are permitted. It learns two three-predicate rules, covering 256 then 512 examples; actual generation subsequently passes 512/512. Exported masks are `99328` and `540928`. Interpreting the learned masks using zero-based word positions:

| Learned conditions | Selected word |
|---|---|
| Question word 2 matches source word 2; question word 3 matches source word 3 | Source word 0 |
| Question word 2 matches source word 0; question word 3 matches source word 2 | Source word 3 |

These conditions were learned from answers, not installed as a semantic parser. Their fixed positional structure still limits transfer to the two tested question forms. This is finite rule induction for structural language alignment, not gradient-based joint primitive learning, a learned semantic metric or a learned whole-question geometric encoder.

The one-read lexical adapter uses the existing learned writer's EOS decision to choose Emit/Stop through the shared recurrent transition. It does not add byte-specific stopping rows; the prior action parameters, encoder, writer and advance operator are frozen. This avoids dependence on whether an unseen name's last letter appeared in training. Whole-question geometric history and actual emitted-byte updates remain in the trace, but selection uses independently encoded word prefixes at ordered positions. This result does not establish a causal contribution from the whole-question root accumulator or feedback to source selection.

Inference uses bounded group/table access, XOR/popcount, bit tests and integer operations without a transformer, mathematical matrix product or provider. The implementation allocates bounded vectors/traces and recomputes word geometry, so no allocation-free kernel, latency or energy improvement is claimed. The canonical geometry binding retains primary architecture identities; paired-H4 transport and a predictive contribution from additional fixed-zeta channels are not established here.

## Actual generated behavior

The first development context is:

```text
ruby did guide felix.
felix did guide clara.
ruby did trust dylan.
dylan did trust clara.
```

| Raw question / source intervention | Actual answer, followed by EOS |
|---|---|
| `who did guide felix?` | `ruby` |
| `who did felix guide?` | `clara` |
| `who did ruby guide?` | `felix` |
| `who did ruby trust?` | `dylan` |
| First question, with only its answer occurrence changed to `alice` | `alice` |

There are 32 training and eight open development families, with eight questions and eight matched single-source-word interventions per family. Source slots and subject/object answers are balanced. Training uses 16 names/four verbs; development uses eight disjoint names/two disjoint verbs. Every authored row was retained without model/geometry filtering. The same grammatical forms occur in both splits. This is lexical transfer within fixed syntax, not held-out syntax, sustained prose or broad language understanding.

| Development panel | Exact answers / 128 |
|---|---:|
| Initialization | 0 |
| Learned Full | 128 |
| ReadDisabled | 0 |
| ScorerDisabled | 0 |
| OrderErased | 0 |
| RecordMiddleDisabled, removing relation-position comparisons | 0 |
| PositionErased, retaining ambiguity instead of choosing arbitrarily | 0 |
| CursorDisabled | 0 |
| StopDisabled | 0 |
| ExactIdentity, diagnostic | 128 |
| FinalRootOnly, diagnostic | 128 |
| FeedbackDisabled, diagnostic | 128 |

All 128 Full answers select the correct source and word occurrence and reproduce identically after reload. Exact-identity and final-root comparators also pass, so this task provides **no measured geometric-distance or full-prefix advantage over those comparators**. Feedback is unnecessary for this one-read answer. This does not undo the earlier ordered-context result, where final-root-only state lost eight answers; the populations and mechanisms differ.

Routes distinguish selected, incompatible, ambiguous, unsupported word window, read disabled and scorer disabled. Generated feedback outside this adapter's word window is recorded as unsupported, not absent information. This applies the scoped OSPF/SpiralCore route-evidence distinction without creating a separate route cache or protocol. Final independent held-out evaluation remains NOT_RUN.

## Verification, resources and continuation

The actual retained replay preserves 13,248 old outputs, 512 complete recurrent control traces and 640 ordered-state control outputs/source paths. Receipt counts are required as well as row equality, preventing empty/truncated replay from passing. One new lexical/corpus test plus 26 retained focused tests pass. Actual report execution includes malformed artifact rejection, unsupported-feedback status and 128 complete reload comparisons. Three metered build invocations include an initial syntax/API correction and a prefit routing-boundary correction; one fit/report was run and passed. No evaluation-driven corpus change or refit occurred.

All 41 earlier sealed roots / 1,125 files, prior frozen source/executable, normal model and both original dirty checkouts verify unchanged. Candidate SHA256 `d1883e09fade8c5337fb604bf6af74556b0f18decdf745ba68bc832a9e4a74ce`; parent SHA256 `dc15055718fff00c166c2c9ad5d21dd0f1d93ebff76b3c89e64fb5cf8c0b2486`.

Local evidence stays under `.uor-handoff/2026-09-12-codex-v7/shared-core-first-step/language-relation-1/`, including the sealed `attempt-1`, `source-freeze.json`, `verification.json`, `resource-final.json`, `delivery.json` and restart notes. Model/build/fit/check charge: 278,129 / 1,500,000 ms; shared cumulative 126,074,833 / 132,950,000 ms; parent cumulative 6,324,260 / 7,710,000 ms. The projection includes two hours wall time, 6 GiB RAM, 512 MiB new storage, a 128 MiB stop margin, two build threads and one model process. No extension, paid compute or cleanup.

**Next:** replace fixed four-word positional features with a shared representation of relative word positions/context and learn across mixed sentence/question lengths and constructions. Keep actual role/content interventions and all retained behavior, and distinguish held-out names from held-out syntax. Reuse the selected reader and recurrent writer; do not add a separate hardcoded parser or answer-family rule for each phrasing. The aim is to learn reusable contextual binding before integrating more dependent language reads and prose generation. No broad audit, old fit, V3–V7 replay or parked repair restart.

Postflight formatting reorders two adjacent module declarations in `native_geometric/mod.rs`; all implementation files remain byte-identical to the frozen fit source. The evidence records both hashes, and the original source/binary/report remain preserved. No refit or behavioral replay was needed for this declaration-order change.
