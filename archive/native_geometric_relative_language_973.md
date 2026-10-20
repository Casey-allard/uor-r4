# Relative language binding — #973

**Empirical result: PASS_RELATIVE_LANGUAGE_BINDING.** The corrected Rust selector generates all 768 open development answers with EOS and selects all 768 exact source/word occurrences. Each lexical, syntactic-combination and joint panel passes 256/256 and all 16 complete intervention families. Training passes 2,048/2,048. Normal model `15baec48` remains retained; no promotion or issue closure.

[Machine-readable evidence](evidence/native_geometric_relative_language_973.json) binds both attempts, source/compiler/executable, data, artifact and sealed report inventories. This extends the [four-word language result](native_geometric_language_relation_973.md), replacing absolute word-pair positions with relative match topology. It reuses the same `language_relation::runtime::generate_with_route` recurrence and existing learned byte/EOS writer; it does not create a separate decoder.

## Data and training dose

| Split | Examples / families | Content vocabulary | Construction combinations |
|---|---:|---|---|
| Training | 2,048 / 128 | 16 names, four verbs | Eight |
| Lexical development | 256 / 16 | Unseen names and verbs | Training combinations |
| Syntactic development | 256 / 16 | Training names and verbs | Four unseen combinations |
| Joint development | 256 / 16 | Unseen names and verbs | Four unseen combinations |

Each family includes eight bidirectional/verb-disambiguating questions and eight interventions changing only the selected source word. Source slots and subject/object answers are balanced. Every row is retained without model or geometry filtering. A separate typed authoring oracle checks unique answers, interventions and exact offsets; those labels never enter the inference or fitting interface.

Records have four to seven words, questions four to eight. Auxiliary, adverb, question-prefix and source-adjunct components vary; source-only prefixes/suffixes move answer positions and create distractor boundaries. The unseen construction combinations use familiar active grammatical components. They are not unseen grammatical families, passive voice, multiple clauses or arbitrary paraphrase. Lexical and joint answer strings are absent from training; syntactic-panel answer strings are intentionally familiar.

The owner suggested additional training for wider capability. This step uses four times the previous training rows and six times the development rows. A deterministic induction pass stops when all training examples have unique correct output; repeated identical epochs would not add information. No dose ablation was run, so this does not establish that 2,048 rows are necessary. The failed first fit below shows why more repetitions cannot repair erased distinctions.

## Representation, learning and demonstrated correction

Inference receives four raw sentences and a raw question. The existing generic lowercase ASCII lexer supplies word boundaries and exact byte offsets, with no name/verb dictionary, stemmer or semantic parser. It admits every word occurrence within 16 words per input, 16 bytes per word and 128 bytes per input.

Words retain signed H4 prefix geometry and exact canonical prime occurrence identities separately. Query/source word comparisons use zero hemisphere-signature Hamming distance. A bounded bipartite match relation then exposes 18 generic predicates: coverage of query matches, unmatched query prefix, candidate position relative to matched boundaries, match-order descents/uniqueness, and first/last endpoint correspondence. No absolute word number, source slot, construction identifier or lexical class enters the selector. Canonical IDs are identity, not semantic distances.

Offline Rust training executes the frozen learned byte/EOS writer to identify every output-compatible candidate occurrence; it does not use correct source/word labels. Half the training examples have multiple output-compatible occurrences. The learner considers 4,047 positive conjunctions with one to four literals, retaining at most eight rules. It chooses shared rules that increase unique correct-output coverage without exposing wrong-output candidates or regressing already unique examples. This remains bounded discrete rule induction, not a scalable gradient learner or jointly learned geometric encoder.

Attempt 1 retained only 16 predicates and at most three literals. It stopped at 1,024/2,048 training and 384/768 development answers, and its new selector answered only 64/128 old language examples. It fails acceptance. The old artifact adapters still preserved their prior results.

The training-only diagnosis found a complete feature collision for candidate `lena` in `nora did help lena.`:

| Question | Candidate is correct? | Matched source positions in question order |
|---|---|---|
| `who did nora help?` | Yes | `[1,0,2]` |
| `who did help nora?` | No | `[1,2,0]` |

Both paths had the same descent count and source boundary. Their final matched endpoint differed. Schema 2 retains the first/last correspondence and allows a four-literal rule to combine coverage, boundary, order and endpoint. The corpus remains byte-identical and all existing behavioral thresholds stay unchanged. A new endpoint intervention threshold was specified before the second fit. The negative candidate, frozen source/executable and sealed attempt remain preserved.

The corrected learner selects only two shared rules, masks `1282` and `68098`, covering 1,024 then 2,048 training examples:

- Unmatched query words form a prefix; candidate immediately precedes the first matched source word; match order increases.
- Unmatched query words form a prefix; candidate immediately follows the last matched source word; match order has a descent; the final query match reaches the final matched source word.

These are interpretations of learned masks, not per-construction answer rules installed in the runtime. The geometric encoder, parent reader parameters, action rows, writer and cursor operator remain frozen. Whole-question geometric feedback is still traced through the common recurrence, but this selector uses word-match topology and answers with one read; it does not learn a whole-question geometric state encoder.

## Actual generated behavior and controls

One joint-development context, using unseen names/verbs and an unseen component combination, is:

```text
today dylan can really trust clara outside.
today ruby can really guide felix outside.
today felix can really guide clara outside.
today ruby can really trust dylan outside.
```

| Raw question or intervention | Actual answer |
|---|---|
| `who can really guide felix?` | `ruby` + EOS |
| `who can ruby really guide?` | `felix` + EOS |
| Replace only the selected `ruby` occurrence with `alice`, first question unchanged | `alice` + EOS |

| Development panel | Exact answers / 768 |
|---|---:|
| Initialization | 0 |
| Corrected Full | 768 |
| ReadDisabled, ScorerDisabled | 0 each |
| OrderErased, CoverageDisabled, RelativePositionDisabled, MatchOrderDisabled | 0 each |
| EndpointDisabled | 384 |
| CursorDisabled, StopDisabled | 0 each, despite 768 correct selections |
| ExactIdentity, FinalRootOnly, FeedbackDisabled — diagnostic | 768 each |

Removing endpoint correspondence reproduces the half-panel loss, supporting the correction's causal role. Match-order removal was allowed to retain up to half because other descriptors also contain relational evidence; it actually retains zero. All 768 Full trajectories reproduce exactly after artifact reload. Every previously successful Full answer from attempt 1 remains correct in attempt 2.

Exact identity and final-root-only comparators still pass, so no advantage of the full geometric word history or metric over these comparators is established. Feedback remains unnecessary for one-read selection. This is bounded structural binding transfer, not semantic similarity, independent final qualification or general prose. Final held-out evaluation is NOT_RUN. Paired-H4 transport and additional zeta channels remain architectural work with no new predictive-contribution claim here.

## Preservation, resources and next action

The actual replay preserves 13,248 prior outputs, 512 recurrent control traces, 640 ordered-state outputs/paths and 1,408 prior language-control outputs/selections. The new selector also answers all 128 old language examples with the correct occurrence. Exact replay counts are enforced alongside row equality. Four new focused tests and 27 retained focused tests pass; actual report execution also exercises malformed artifact rejection, unsupported-feedback classification and reload fidelity. Three release build invocations include formatting finalization and the demonstrated endpoint correction; two fits/reports are retained. CI queue acknowledgements are not tests.

All 42 earlier sealed roots / 1,137 files, both original dirty checkouts, the normal model and predecessor frozen source/executable verify unchanged. The failed and corrected attempts have separate sealed roots and source/binary snapshots. Candidate SHA256 `4e3d4da1fb39425b30007b1e00c16a520f860c8dd8d3257a0787f99144ada9e9`; negative `b075838a5ab2d99c755a8af70e9daf1de8ccd048a33c99b03f1322daeeed59f1`; parent `d1883e09fade8c5337fb604bf6af74556b0f18decdf745ba68bc832a9e4a74ce`.

Complete build/preparation/fits/controls charge is 416,147 / 2,400,000 ms; shared cumulative 126,490,980 / 132,950,000 ms; parent cumulative 6,740,407 / 8,910,000 ms. Before execution, the parent local allowance was increased by 1,200,000 ms under standing owner authorization; the shared ceiling stayed unchanged. Projection: two hours wall time, 6 GiB RAM, 512 MiB new storage, 128 MiB stop margin, two build threads and one model process. No paid/external compute or cleanup. Bounded vectors/traces allocate and word geometry is recomputed, so no allocation-free, serving-latency or energy improvement is claimed. Serving contains no transformer, mathematical matrix product or provider.

Local evidence remains at `.uor-handoff/2026-09-12-codex-v7/shared-core-first-step/relative-language-1/`: projection/extension, both attempts and source snapshots, diagnosis, independent review, verification, resources, protected delivery and precise restart. Schema 1 remains readable with its frozen executable; the current decoder intentionally requires schema 2.

**Next:** integrate this learned raw-language binding with the existing dependent-read/query-update path. A selected intermediate value must change the next actual source choice and final generated answer, trained from final output with no supplied intermediate answer or gold source path. Begin with bounded two-relation language composition and matched first-source/read/update-disabled controls; preserve current lexical, construction and old behavior. Make query formation and any supplied boundaries explicit. This is the next step toward shared recurrent language behavior; do not spend it on another isolated metric/conformance panel, broad research restart or V3–V7 replay.
