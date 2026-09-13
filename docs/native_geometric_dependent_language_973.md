# Dependent language binding through shared geometric state — #973

**PASS_DEPENDENT_LANGUAGE_BINDING.** One fit learns a query-update rule from final generated answers and reaches 2,048/2,048 training examples. The same candidate produces 768/768 development answers, both exact source/word occurrences and intermediate selections, with all 48 intervention families complete. Lexical, construction-combination and joint panels each pass 256/256. [Source-bound evidence](evidence/native_geometric_dependent_language_973.json). Retain normal model `15baec48`; no promotion. Final independent held-out evaluation is NOT_RUN.

## What changed

The [dependent language implementation](../crates/uor-r4-core/src/native_geometric/dependent_language/mod.rs) joins two raw-language reads through the existing recurrent transition. Inference receives four raw records and a prompt containing two question-mark-delimited questions. The relative reader selects the first answer word. Eight generic context-match and relative-position predicates describe every possible word substitution in the second question. The learned rule chooses a unique position, substitutes the actual retrieved bytes, and commits the new geometric query with a silent Read. The [shared language writer](../crates/uor-r4-core/src/native_geometric/language_relation/runtime.rs) resumes that exact state and emits the final answer bytes and EOS. It preserves read history: the initial selection counts as read 1 and the dependent transition commits read 2.

Preparation tries each bounded substitution and runs the actual suffix generator. Only the final answer plus EOS determines output compatibility; gold intermediate values, replacement slots and source paths never enter learning or inference. Greedy induction considers 92 conjunctions of up to three predicates, with a maximum of eight rules. It selects one rule, mask `3`: replace a word absent from the context that has a context-matching word before it. That interpretation follows the fitted mask; no pronoun dictionary or per-construction rule is installed. The authored references are `they` and `them`. This simple rule is sufficient for this corpus and does not establish general reference resolution.

The reader, word encoder, prior update operators, action rows, byte/EOS writer and cursor parameters remain frozen. Newly learned parameters select where the one-word query update occurs. Training uses actual suffix utility, following the earlier dependent-read learning method; the old two-byte XOR update is not treated as a semantic word transform. The questions and their scheduling boundary are supplied by the declared interface: this gate does not learn when to split the prompt or how many reads to perform.

## Environment and actual behavior

The Rust corpus validates 2,048 training and 768 open development rows before fitting. Four subject/object role combinations each include a base, first-source switch, active-final change and inactive-final control. Every 16-row family must retain those relationships; 48/48 families complete. Each split balances both read source slots and both subject/object roles. Lexical and joint development names, verbs, intermediate values and answer strings are absent from training. Construction transfer means unseen combinations of familiar active auxiliary, adverb, question-prefix and source-adjunct components, not new grammar families. The syntactic split deliberately retains the training vocabulary. Questions have 4–8 words and records 4–7 in this corpus. No model-based row filtering is used.

An actual joint-development context is:

```text
yesterday ruby did quietly guide felix outside.
yesterday felix did quietly trust clara outside.
yesterday dylan did quietly trust alice outside.
yesterday ruby did quietly trust bruno outside.
```

Prompt: `please tell me who did ruby quietly guide? please tell me who did they quietly trust?`

| Intervention | First selected word | Actual updated second question | Actual final output |
|---|---|---|---|
| base | felix | please tell me who did felix quietly trust? | clara + EOS |
| first-source | dylan | please tell me who did dylan quietly trust? | alice + EOS |
| active-final | felix | please tell me who did felix quietly trust? | bruno + EOS |
| inactive-final | felix | please tell me who did felix quietly trust? | clara + EOS |

Changing only the first source selects the other second record and changes the final answer. Changing the active final value changes the output; changing the inactive final record preserves it. Across all three splits there are 192 matched first-source switches, 192 active-final changes and 192 inactive-final controls.

| Development control | Exact final answers / 768 |
|---|---:|
| Initialization | 0 |
| Full | 768 |
| FirstReadDisabled | 0 |
| SecondReadDisabled | 0 |
| ScorerDisabled | 0 |
| UpdateDisabled | 0 |
| PayloadReversed | 0 |
| PositionDisabled | 0 |
| ContextMatchDisabled | 0 |
| SwapQuestions | 0 |
| ExactIdentity | 768 |
| FeedbackDisabled | 768 |
| CursorDisabled | 0 |
| StopDisabled | 0 |

PositionDisabled saturates the positional flags, removing their discrimination; it does not erase all context matching. ExactIdentity and FeedbackDisabled are diagnostic controls. Their full success establishes neither a geometric-metric advantage nor a need for emitted-byte feedback in this two-read task. The selected intermediate content does control the second read before emission, as the source-switch and update interventions show.

Signed H4 prefix geometry and structural Hamming comparisons retain their implemented word-binding roles; canonical occurrence identity stays separate. The new update is bounded byte substitution followed by geometric query reconstruction. No new paired-H4 transport law, zeta contribution or semantic manifold advantage is measured. Inference has no transformer, mathematical matrix product or external provider. Bounded vectors and traces allocate and geometry is recomputed, so no allocation-free, latency or energy improvement is claimed.

## Validation and preservation

All 13,248 older outputs, 512 recurrent traces, 640 ordered controls, 1,408 language controls, 128 older language examples through the relative selector and 9,984 relative-language controls are unchanged, with exact count checks. All 768 new Full traces reproduce after artifact reload and continue the silent Read's exact state. Three new focused tests and 31 retained focused tests pass. The retained tests exercise the frozen earlier mechanisms; no old fit was repeated.

The first report stopped after a successful fit because it incorrectly expected a read count of one after the dependent transition. The report-only correction checks one before and two after, preserving the state-continuity requirement. A second exclusive report verifies the earlier seal and identical initial/data identity and reuses exactly the same fitted candidate. Postflight also verifies identical acceptance bytes and candidate SHA256. Both sealed attempts and their source/executable snapshots remain preserved. Three release build invocations include a test error-conversion correction and the report-only correction; these are not three model fits.

All 44 earlier sealed roots / 1,161 files, predecessor frozen source/executable and both original dirty checkouts verify unchanged. Candidate SHA256 `b44de7fa42ddb1e2728d4e2d950bc91d1ad2320f85fcb67a615b057f3c3612e2`; parent `4e3d4da1fb39425b30007b1e00c16a520f860c8dd8d3257a0787f99144ada9e9`. The normal retained model remains unchanged at `15baec48`.

Complete build/preparation/fit/controls charge at verification: 320,067 / 2,100,000 ms; shared cumulative 126,811,047 / 132,950,000 ms; parent 7,060,474 / 8,910,000 ms. The projection includes two hours wall time, 6 GiB RAM, 384 MiB new storage and a 128 MiB stop margin, two build threads and one model process. Existing allowances suffice; no extension, paid compute or cleanup. Final local accounting and protected delivery receipts remain in `.uor-handoff/2026-09-12-codex-v7/shared-core-first-step/dependent-language-1/`. CI compatibility acknowledgements are not test results.

**Next:** Extend the existing dependent language runtime into one shared Read/Emit/Stop loop for mixed direct and dependent questions. Learn scheduling from actual final answer plus EOS, initially reusing the reader, writer and learned substitution. The same artifact/path must preserve direct answers, dependent source switches and all older controls. Do not supply a gold read count or intermediate path; keep any punctuation/clause boundaries explicit. This removes unconditional second-read dispatch before further grammar expansion. No broad audit, isolated metric panel, old fits or V3–V7 restart.
