# Ordered correspondence through learned context projection — #973

**PASS_ORDERED_CORRESPONDENCE.** The UOR-R4 Geometric Language Model now preserves ordered occurrence correspondence while allowing intervening source words already classified as context-only by its inherited learned roles. Actual generation passes all 2,816 training rows, 384 new development cases and 6,304 earlier full traces, including 512 typed unresolved outcomes. One fit selected rule [851970]; a bounded source correction reused that rule without a second fit. [Machine evidence](evidence/native_geometric_correspondence_973.json) binds the reports, source versions, binaries, artifacts and preservation checks. Normal model 15baec48 remains unchanged; this is a qualified experimental mechanism, not default-model promotion or general prose.

## Mechanism and causal diagnosis

The [previous occurrence result](native_geometric_occurrence_973.md) retained individual query-to-source assignments but reduced them to aggregate admission predicates. A valid styled question and an incorrect split repeated-word match shared mask 350879. Its frozen 137-proposal family could not separate all 912 sole-target conflicts. The successor retains ordered matched runs before reducing each assignment to a selector predicate. Consecutive matched query words form a run unless an inherited context-only role or an unmatched position separates them. No names, supplied answer spans or new grammar lists are added.

The first version required raw forward source adjacency within a run. Its one output-based fit selected [851970] and passed training and all new development cases, but retained only 5,344/6,304 prior full traces. All 960 failures contained “trust,” which is absent from the inherited 15-word context-role set. A query run such as “bruno trust” maps across an intervening known source context word such as “did”; requiring raw gap one incorrectly rejected the route. This failed artifact and its source remain sealed.

Schema 2 preserves the same learned mask and all parent parameters, while projecting source adjacency through the existing learned context-role set. Each step retains its raw source positions, signed raw gap and projected gap. Forward source gaps may skip context-only positions; intervening content words still count, and repeated or reversed source positions still fail. Thus the legitimate “bruno … trust” correspondence survives, while repeated “amber” occurrences separated by content “oscar” remain distinct. This is a bounded correspondence predicate using inherited learned roles, not a universal language-order rule.

The corrected wrapper explicitly versions this predicate, rejects the old schema and embeds the unchanged parent span artifact with its digest. It preserves occurrence enumeration, source/word/byte identity, signed ordered word-prefix geometry, prime/zeta identity, whole-phrase query transport, completion policy and byte/EOS writer. The trace retains the earlier low 18 feature bits for direct comparison. The experimental runtime uses geometric comparison and integer/state/table operations without matrix products, transformers or provider responses. Its allocating prototype remains separate from the frozen no-allocation R4G1 contract.

## Executed gate and controls

| Measurement | Actual result |
| --- | --- |
| Actual training answer/EOS revalidation | 2,816/2,816 |
| New full answers, paths, spans/bounds, payloads, queries and encodings | 384/384 |
| Complete matched development families | 32/32 |
| Earlier complete generated traces and decisions | 6,304/6,304 |
| Included earlier typed unresolved outcomes | 512/512 |
| Successful full traces from both previous failed occurrence candidates | 296/296 and 384/384 |
| Full versus ExactIdentity; reload | Equal on every new case |
| StructureDisabled | 296/384; all 144 disjoint cases retained, 88 overlap successes lost |
| ParentUnionMatches and UnionMatches | 144/384 each |
| ReadDisabled | 0/384 |
| UpdateDisabled | 192 direct successes, zero dependent successes |
| NonInjective alone | 384/384; ordered adjacency already prevents reuse |
| Prior phrase controls through their original parent | 6,048/6,048 identical responses |

ContextProjectionDisabled restores raw gaps and reproduces all 6,304 earlier candidate responses, including precisely the same 960 failures. This reproduction checks 5,344 complete structs and 960 stored compact responses; it is **response reproduction**, not a claim of complete-trace equality for all 6,304 control rows. Independently, corrected Full compares all 6,304 complete generated structs against the qualified predecessor and preserves them exactly.

The multiplicity guard remains typed unresolved at clause 1 with actual payload “ruby ruby” and its complete next query. Removing both injectivity and structure instead produces the incorrect answer “clara” with EOS from the same selected payload/query. The isolated non-injective control is redundant here because adjacency itself prevents occurrence reuse. All four earlier guard responses also replay identically through their original occurrence artifact. Older adapter controls were not rerun; their original reports and sources remain preserved.

## Test environment and attempt lineage

The exact prior 2,816 direct training and 384 development rows are reused. Training combines 2,624 preserved span rows and 192 renamed direct overlap rows. The learner receives actual answer/EOS credit, without gold spans or paths. Exact inputs are disjoint, but authored grammar and templates overlap; **final independent holdout is NOT_RUN**. ExactIdentity performs equally, so this does not measure a geometric metric advantage.

Three no-fit representation diagnostics each find exactly one compatible offer on all 2,816 rows and separate all 912 earlier conflicts. They enumerate 152,256 candidates and actually force 2,924 reachable outputs; the other 149,332 cannot be admitted by this proposal family and remain compatibility NOT_EVALUATED. Diagnostic 1 saved aggregates for successful rows despite promising individual outputs. Diagnostic 2 corrects that reporting omission and persists every evaluated output before fitting; the mechanism, data and gate are unchanged. Both original sealed reports/source versions remain. Diagnostic 3 applies the corrected context projection with the same output persistence.

Attempt 1 searches 154 bounded conjunctions extending the inherited base [327682] by zero, one or two of 17 remaining bits, with at most five literals and eight rules. It rejects incompatible admissions and selects [851970], but fails retention as described above. Attempt 2 explicitly rebinds that fixed rule to schema 2 and the corrected source. It performs no rule search or fit, revalidates every training output, and passes the complete generation gate. Its fit.json is the preserved historical fit receipt; fit_origin and training-revalidation.json identify the distinction. Maximum observed search is 303 on training and 183 on development, within the 16,384 route cap.

All five reports remain at their original project-local evidence paths:

- correspondence-runs-1/diagnostic-1: raw-gap representation, initial output persistence.
- correspondence-runs-1/diagnostic-2: raw-gap representation, complete output persistence.
- correspondence-runs-1/attempt-1: FAIL_ORDERED_CORRESPONDENCE, SHA256 53645f7d5cad7bf1cb34cc55f2e16ed8e48ea19261edc7e5441b538823b0d188.
- correspondence-runs-1/diagnostic-3: context-projected representation.
- correspondence-runs-1/attempt-2: PASS_ORDERED_CORRESPONDENCE, SHA256 1838cc12bda863c769db9239d7b225e91dd82856c7e90bf7427abffb5a9e1b23.

Qualified embedded span parent SHA256 remains 44c0ebf482d5807354a2e2b4c3fd6ceb310dd076ce199536f801f6cad472264b. The earlier failed occurrence candidate 8e32ec603a6e40a3fc9a9a6db0a92a92eb6917f57ff93d022a561d0fddc04459 remains preserved and unpromoted.

## Validation, preservation and resources

Three successful optimized builds and one retained compile failure; 59 focused tests pass (28 dependent-language tests and 31 retained tests). The failed compile exposed a test literal type mismatch and a shadowed report count; both were corrected before the final successful build. Source versions 1–3 and their binaries, failed-source snapshot and all reports remain. All 59 prior sealed roots / 1,499 files and eight predecessor source/binary versions verify unchanged. Five new roots / 121 files bring the total to 64 roots / 1,620 files. Both original dirty checkouts retain their original HEAD and tracked/untracked status fingerprints.

Complete model/build/preparation/fit/evaluation/control charge is 667,826/1,200,000 ms. Parent cumulative charge is 10,940,809/11,910,000 ms; shared charge 130,691,382/132,950,000 ms. Peak recorded RSS is 3,204,857,856 bytes within 6 GiB. The current storage receipt records 166,895,616 bytes step growth within 384 MiB including the 128 MiB stop margin; final resource receipts include subsequent documentation/delivery overhead. Before execution, standing owner authorization extended parent time by 600,000 ms and parent storage by 256 MiB; shared ceilings were unchanged. No external spending or cleanup.

Evidence, exact commands, source freezes, resource ledger, failure localization, independent review and restart notes remain under /Users/casey.allard/uor-r4-worktrees/initial-previous-intent/.uor-handoff/2026-09-12-codex-v7/shared-core-first-step/correspondence-runs-1/. Current state owns the next bounded action. Order-permuted phrase endpoints under the same relation have not yet been qualified; broad prose, arbitrary discourse, full-path energy advantage and default-model promotion remain unestablished.
