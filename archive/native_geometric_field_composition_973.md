# Native owner/value field composition for #973

Status: **retained at bounded owner/value composition scope**. The [evidence receipt](evidence/native_geometric_field_composition_973.json) binds artifacts, source, actual commands, results and resources. This is a bounded UOR-R4 Geometric Language Model development result; general prose and flexible instruction following remain unqualified.

## Observed limit and implementation

The retained writer artifact `169f23ef` selects the correct stored value for both of these requests, but starts by copying that value:

```text
Record: selvi in Dusk Ridge. Where is selvi? Name the owner first. Answer:
Record: selvi in Dusk Ridge. Where is selvi? State the owner first. Answer:
```

Its actual answer is ` Dusk Ridge.\n`. The previous word adapter enters shared lexical emission after the value copy completes. Changing suffix weights cannot put an owner before an already emitted value.

The new [field adapter](../crates/uor-r4-core/src/native_geometric/field_composition.rs) exposes Owner-read and Value/span-read alongside Base/defer and learned byte/EOS alternatives in the existing signed-H4 lexical router. It starts only from the winning inherited exact read. A unique, current, nonconflicting relation must match the complete selected WordAtom and span bytes. The anchor retains relation ID, source address, occurrence endpoint, byte endpoint, span width and response boundary. Numeric and dependent reads defer to their inherited paths.

The selector learns when to emit a lexical byte, start either exact field read, defer or stop. A fixed cursor advances only after observing the selected field byte. A completed field becomes a typed prefix symbol, so the learned transition does not depend on the field's spelling or length. Actual generation now produces:

```text
 selvi is in Dusk Ridge.
```

The exact owner and value come from the same selected record. The connective and stopping choices are learned. Authored request forms, owner/value labels and output pieces exist only in the offline [Rust preparation and fitting driver](../crates/uor-r4-core/examples/native_field_composition.rs). Serving receives prompt tokens; it contains no sentence template, phrase parser, query-owner substitution or provider response.

New features use disjoint namespace 17. Three normalized 18-bit prefix symbols occupy separate positions in a 54-bit identity; contextual lexical primes and candidate identities remain separately packed. There is no hash-bit semantic metric or overlapping packing. Existing H4 scoring, orientation, fixed zeta geometry, the writer, source selection and numeric operators remain active. Architectural preservation and local intervention dependence do not establish comparative geometric or energy advantage.

## Preparation, fitting and artifact preservation

Two unsuccessful preparations remain preserved. The first used two prefix symbols and found 32 exact contradictory frame groups: the final ` i` in the opening connective and in ` is i` required different next bytes. It stopped before fitting. Adding the third symbol removed that collision. A full-prompt vocabulary then required 4,455 features and exceeded the configured 4,096 cap; it also stopped before fitting.

The final preparation balances 32 owner-first examples with 48 matched value-first examples and retains 825 prior documents. Offline document-presence association selects nine discriminative lexical words: `a`, `explain`, `first`, `name`, `owner`, `sentence`, `state`, `stop`, `the`. Payload names with equal class-normalized document presence contribute no association; there is no runtime word blacklist. The training artifact binds all 905 document receipts. Of the 825 older documents, 322 contribute eligible Base examples and 503 have no eligible field entry; the 48 matched cases also contribute Base examples. This yields 786 presented frames and 37 distinct frames.

The final router has 1,945 features: all 1,125 inherited features remain exact and 820 new features are admitted. A bounded Rust fit improves distinct-frame correctness from 11/37 to 37/37, with hinge loss 130 to zero, 61 accepted updates and 1,561,454 proposals. The fit kernel takes 5.128 seconds; its complete load/preparation/fit/validation command takes 83.949 seconds. Teacher-forced frame correctness is separate from the free-generation results below.

Retained CID: `blake3:d1b0985fb8af0528dae6a7c684e1c76be3634099c868a7f9a3b09454cb68d1c0`. SHA256 `b33ff399684b86a0495eeae993a1f2e05b88d1d4a38d33bddb3485b5a9146cdf`; 13,819,092 bytes. Local artifact: `.uor-models/native-typed-value-2026-09-05/field-composition/model.json`. Its optional field witness stores the previous lexical block and parent CID. The prior lexical dictionary, tokens, landmarks, biases, ranks and all old feature transforms remain exact. Removing the extension restores parent `169f23ef`; recursive parent validation remains mandatory. All previous artifacts, failed source variants, preparation receipts and cumulative charges are preserved.

## Actual behavior and controls

| Population | Actual result | Scope |
| --- | --- | --- |
| Authored construction answers | 80/80 | 32 owner-first and 48 matched value-first |
| Inherited construction comparison | 873/873 | 825 older documents plus 48 matched cases; exact parent bytes, EOS and initial/final relation state, not 873 independently correct answers |
| Post-selection fresh identities | 30/30 | 12 owner-first and 18 matched value-first; complete owner/value spellings absent from construction, fixed authored request forms |
| Structural stress | 12/14 | 10/12 owner-first and 2/2 unsupported-owner abstentions; the two revision failures are retained below |

The candidate was frozen before drawing fresh identities, with no subsequent refit. The generated construction and fresh owner-first outputs are checked by restoring checkpoints and repeating predictions at each output position. Reverse relations, equal-valued distinct occurrences, single-word fields and reads after recent-window eviction pass the tested structural cases.

On the 32 owner-first construction cases, disabling the extension restores the exact parent output in all 32. Removing only its contextual features also restores all 32. Setting only its learned transforms to identity loses all 32 correct answers and restores the parent output. Disabling exact field reads loses all 32; only 11 terminate within the 96-token harness cap. Those read-disabled nonterminations remain negative intervention evidence.

## Revision failures and the next repair

Both models observe this input:

```text
Record: selvi in Dusk Ridge. selvi now in Copper Vale.
Where is selvi? Name the owner first. Answer:
```

The inherited writer stores two Assert records: `selvi → Dusk Ridge` and `now → Copper Vale`, each with no previous-version link. It commits no revision. The short request selects the recent second value; the parent emits ` Copper Vale.\n`, while the field candidate faithfully emits the incorrect stored owner in ` now is in Copper Vale.\n`. With 40 repetitions of `oak ash elm ` before the query, the parent selects the older persistent relation and emits ` Dusk Ridge.\n`; the candidate emits ` selvi is in Dusk Ridge.\n`. Initial and final relation states are exact between parent and candidate.

These failures expose existing contextual owner binding and revision-action defects. The field adapter does not repair them. The next implementation should trace the competing owner and Assert/Revise proposals at the second fact, determine whether their existing representation distinguishes the intended roles, and learn the missing shared-writer distinction. Check the committed owner, previous-version link and current directory, then replay both value-first and owner-first answers before and after eviction. Preserve ordinary assertions, conflicts, distractors and literal factual use of `now`; do not hide the defect with query-text substitution or a lexical blacklist.

The existing two 33-step overflow targets remain outside the 32-step response-entry bound. The earlier owner-first targets phrased as the already retained `Explain in a sentence` request remain unqualified; those value-first answers are deliberately preserved. Arbitrary wording, sustained general prose, general reasoning, Rust program synthesis, alpha, frontier capability and complete-path energy savings remain unqualified. New-artifact browser/WASM/HTTP are NOT_RUN. #973 remains open.

## Checkpoint, interface and resources

Checkpoint schema `/7` requires the field state for artifacts that carry the extension. The host validator reconstructs the selected parent read and replays the actual retained output through the learned selector, comparing the anchor, cursor, three prefix symbols, response-entry state and default inherited-copy state. Serialized field state cannot authorize a bypass of this replay. Base/defer continues to use the existing copy validator. The bounded runtime resets transient field selection on observed-token mismatch; import, partial reads and independent turns are checked separately from serialization round trips.

All 663 retained answers pass. Word construction/open/prior transfer pass 132/132, 28/28 and 24/24; source positions and original abstention pass 40/40 and 10/10. Six word/source/abstention/overflow reports preserve exact text, EOS, correctness and source selections. Independent history passes 12/12 with exact prior records, versions, sources and outputs. Copy→Add and Add→Copy each pass 24/24 open and stress cases; Add→Add passes 28/28 open and stress, mixed operations 8/8 each, and 254 transition replays plus 72 prior lexical outputs pass. Fourteen earlier semantic/control reports match exactly, including negative interventions. These populations overlap; they are not a single independent accuracy estimate.

Native API passes 187/187 checks. The actual candidate serves generation, independent turns and checkpoint imports; historical nested-witness corruption checks explicitly use its reconstructed parent where needed, while new extension corruption checks target the candidate. 39 unique focused tests pass, including the integer-kernel source guard and eight actual-artifact allocation/checkpoint checks. The field test measures zero observe/begin/predict/observe allocations, restores each input/output position, exercises partial-owner import and independent turns, and rejects altered anchors, source extents and third-prefix state. It also checks observed-token mismatch recovery without a stale field selection. The field check restores 118 input and 31 output positions, including 25 active-field positions. Artifact loading takes 26.613 seconds; the measured warm predict/observe median is 154.666 microseconds and maximum 188.250 microseconds. Loading, encoding, session creation, BOS, end-response, checkpoints, JSON, decoding, reporting and mismatch-recovery generation are excluded from that warm timing. It does not establish end-to-end speed or energy advantage.

Actual generated Copy/Add equalities from both open and stress reports and four retained identifier-return fragments were extracted, compiled with rustc and executed. These are bounded numeric/identifier checks, not Rust synthesis. Release/offline builds, cargo fmt --check and the claim-wording check passed. The blanket full suite, Clippy and new-artifact browser/WASM/HTTP were not run. GitHub's five historical status names are compatibility acknowledgements, not executed tests.

This cycle used **1890.025 model seconds** and **551.511 engineering-command seconds**, including unsuccessful preparations and all executed validation. Cumulative model use is **24122.067 / 26250.000 seconds**. The necessary 1,200-second extension was recorded before use; the preserved 196.620-second reservation leaves 1931.313 seconds. Conservative storage at the delivery checkpoint is **36,717,125,632 / 39,158,181,888 bytes**, including retained material, the owner-checkout copy allowance and a 16 MiB metadata reserve, with 2,306,838,528 bytes remaining after the 128 MiB stop margin. No storage extension was needed. Limits remain one model process, two threads, 4 GiB model RAM, 6 GiB build RAM, 512 configured context tokens and 96 output tokens; the inherited five-turn replay permits 1,056 padding tokens per turn and 8,192 total input tokens. Sampled model/build process-tree peaks are 3,215,065,088 / 2,621,243,392 bytes. No unique deletion or paid external compute occurred. Small metadata/document/Git commands are outside the engineering stopwatch.
