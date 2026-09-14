# Lexical identity reused across occurrence roles — #973

**FAIL_LEXICAL_ROLE_REUSE; GLOBAL_CONTEXT_PAYLOAD_BARRIER_LOCALIZED.** The unchanged UOR-R4 Geometric Language Model passes all 24 ordinary-name cases and none of the 24 cases carrying the person name `will`. Every intended `will` span is enumerated, but every target witness lacks the same required payload-boundary bit. No artifact/runtime change or fit occurred. This is a measured pre-existing limitation exposed by a new test, not a regression from a new candidate. Normal model 15baec48 and experimental correspondence artifact 1838 remain unchanged and unpromoted. [Evidence](evidence/native_geometric_lexical_role_973.json) binds the actual report, source and preservation checks.

## Question and independent test

The previous [phrase-order result](native_geometric_phrase_order_973.md) qualifies exact endpoint order against reversed and gapped alternatives. It does not establish that the same word can serve as context in one occurrence and content in another. The inherited context list contains `will`; the source applies that role globally when deciding answer eligibility.

The independent raw oracle parses the authored subject/did/verb/object protocol and resolves whole endpoints without consulting geometry or model candidates. A representative ordinary chain is:

```text
alice did call bruno.
bruno did help helen.
felix did help dylan.
clara did visit amber.

who did alice call? who did they help?
```

The model emits **helen** plus EOS. Consistently replacing `bruno` with `will` in the two linked records preserves the oracle answer but causes generation to end **Exhausted with no output tokens**. It is not a correct abstention or a claim of missing information.

The frozen panel contains 2 names × 2 relation directions × 4 record rotations × 3 variants = 48 cases. The active variant changes the selected downstream answer to oscar; the inactive variant changes a distractor answer. These each change one record. Consistent identity substitution changes exactly two linked records and nothing else. All text remains lowercase, avoiding a case-handling confound. The oracle checks exact source/word/byte spans, payloads, expanded queries and answer/EOS before loading the artifact. Native generation receives only records, prompt, artifact and control.

## Actual result and localization

| Measurement | Ordinary name bruno | Reused context identity will |
| --- | --- | --- |
| Complete dependent generation | 24/24 | 0/24 |
| Direct first-clause generation | 24/24 | 0/24 |
| Intended first span enumerated | 24/24 | 24/24 |
| Every target witness missing only required bit 18 | 0/24 | 24/24 |
| Supplied-payload unique correct update — counterfactual | 24/24 | 24/24 |
| Route on oracle-expanded query — counterfactual | 24/24 | 24/24 |

Full equals ExactIdentity as a complete Generated struct on all 48 cases, including the failures, and all 48 reloads reproduce the same structure. ReadDisabled and UpdateDisabled each lose every correct ordinary-name answer; failure on already failing `will` cases adds no further control evidence.

The rule [851970] requires bits 1, 16, 18 and 19. `span::candidates` retains the target span. `occurrence::correspondences` begins each witness's barriers with the source's inherited word-global context flags, then adds positions assigned to the query. `span_boundary::complete_run` rejects any span containing a barrier, so the target `will` occurrence cannot supply bit 18. Its other required selector bits are satisfied. The existing correspondence ordering machinery is not the rejecting predicate on these cases.

The source list also drives query-run separation and source-context projection in `correspondence.rs`. A role repair must consider these consumers explicitly; editing only the writer or updater would not address this rejection.

The direct first-clause probe is actual generation on the raw first question. In contrast, the supplied-payload updater and route-on-oracle-query probes are **counterfactual diagnostics**. They receive the independently expected payload/query after the actual first read has failed. They show that the existing downstream mechanisms can process these inputs, but they do not establish an actual successful continuation or qualify a repaired model. Witness inspection localizes the predicate descriptively; no barrier-removal recovery intervention was executed.

All 192 control responses retain actual generated structures, selection, query and outcome checks. Separate localization records preserve target span existence, every target witness, missing required bits, context flags, barriers, direct generation and clearly labeled counterfactual results. The capability gate remains FAIL even though the diagnostic executed successfully.

## Next implementation requirement

Replace the assumption that lexical context membership is sufficient to forbid every occurrence as content. Reuse existing source-relative spans, exact correspondence witnesses, and `span_boundary::learn_from_compatible` / `span_learning` output-based latent-span credit. Their final word set currently discards occurrence conditions. Develop a bounded learned role decision that uses local query/source correspondence and preserves the difference between a matched context occurrence and an answer occurrence of the same identity.

Freeze mixed-role examples in which `will` serves as an auxiliary and a name, including both roles in one context, before selecting the representation or fitting. Verify that the proposed local role observations distinguish the required decisions. Train with actual output compatibility rather than supplied answer spans at serving, and retain phrase extent/order, source changes, missing/conflicting routes and earlier full traces. Merely deleting `will` from the global context set, adding a name exemption, or clearing every proposed span's barriers is not a demonstrated contextual-role solution. In a same-context example such as “alice will call will,” both occurrences can emit identical answer bytes. Final-output compatibility alone therefore cannot identify the intended occurrence. Preserve compatible spans as latent alternatives and require local structural transfer to resolve them; the evaluation oracle must not become a serving cue or a first-match training tie-break. The next mechanism is NOT_IMPLEMENTED/NOT_RUN; no change to architecture or promotion follows from this diagnostic.

## Validation, preservation and resources

One optimized offline Rust build passes 30 focused dependent-language tests, including the new oracle/intervention checks. One behavioral evaluation, zero fits or retries. Only a cfg(test) report module and its registration were added; every earlier frozen source file except that test registration is byte-identical. Older model-control panels were NOT_RERUN; their artifact/runtime identity and original evidence retain their earlier scope. Current source/binary and independent review are preserved.

All 65 earlier sealed roots/1,628 files, 12 predecessor source/binary versions and both original dirty checkouts verify unchanged. The new eight-file report brings the total to 66 roots/1,636 files. Candidate SHA256 remains 1838cc12bda863c769db9239d7b225e91dd82856c7e90bf7427abffb5a9e1b23 at correspondence-runs-1/attempt-2/candidate.json. Exact reports, source, resource ledger and restart notes remain under /Users/casey.allard/uor-r4-worktrees/initial-previous-intent/.uor-handoff/2026-09-12-codex-v7/shared-core-first-step/lexical-role-1/.

Complete model/build/evaluation charge: 153,307/600,000 ms. Parent cumulative: 11,268,917/11,910,000 ms; shared: 131,019,490/132,950,000 ms. Peak sampled process-tree RSS 3,007,823,872 bytes within 6 GiB. Before execution, standing owner authorization recorded a parent storage extension of 128 MiB to retain evidence and the 128 MiB stop margin; time and shared ceilings are unchanged. Step storage envelope 320 MiB, projected growth 128 MiB; measured growth about 48 MiB before final documentation/delivery, whose overhead is included in the final resource receipt. No external spending or cleanup.

This remains an authored-language diagnostic. Final independent qualification is NOT_RUN. General prose, arbitrary discourse, metric advantage, complete-path energy advantage and default-model promotion are unestablished. #973 remains open.
