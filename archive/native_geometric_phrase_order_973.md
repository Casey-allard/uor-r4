# Exact phrase-order identity — #973

**PASS_PHRASE_ORDER_IDENTITY.** The unchanged UOR-R4 Geometric Language Model correspondence artifact passes all 80 new development cases: 48 complete answers, 16 missing exact endpoints and 16 conflicting exact endpoints. It preserves exact phrase order against reversed same-word competitors and gapped alternatives. All 16 matched families, reloads and Full/ExactIdentity complete-struct comparisons pass. There is no fit or runtime change. Normal model 15baec48 remains unchanged and unpromoted. [Evidence](evidence/native_geometric_phrase_order_973.json) binds the source, artifact, sealed report and preservation checks.

## What was tested

The previous [ordered correspondence result](native_geometric_correspondence_973.md) exposed ordered matched runs to learned admission and projected adjacency through inherited context-only roles. It had not placed reversed endpoints containing the same words under the same relation. This step tests that distinction with both phrase orientations, both query directions, four source rotations and five single-record interventions: 2 × 2 × 4 × 5 = 80 rows.

A representative baseline is:

```text
alice did guide ruby amber.
ruby amber did help helen.
amber ruby did help oscar.
ruby cedar amber did help bruno.

who did alice guide? who did they help?
```

The unchanged model answers **helen** followed by EOS. Changing only the first payload to **amber ruby** changes the actual next query and answer to **oscar**. Changing only the gapped distractor's answer leaves the original answer intact. Removing the exact ruby amber endpoint leaves a typed unresolved continuation, even though reversed and gapped competitors retain its words. Adding a second exact endpoint with a different answer produces typed ambiguity, independent of record position.

The independent oracle parses the authored subject/did/verb/object protocol and matches whole endpoints in raw word order. It never consults model candidates, learned features or geometry. It records exact source identities, word spans, byte bounds, payloads, expanded queries and missing-versus-conflicting outcomes. Validation checks unique inputs, one changed record per intervention and equal word multisets for the active permutation. The gap words ruby/amber/cedar are verified absent from the inherited context-role set, so the gap tests content rather than projected context.

All model calls receive only records and prompt. The oracle and expected fields remain evaluation metadata. The model retains prime/zeta lexical identity, signed ordered geometry, exact occurrence/span identity, the learned correspondence rule, whole-phrase updater, shared completion and byte/EOS writer. No dense attention, transformer, provider or matrix product is added.

## Actual generated behavior and controls

| Test | Result |
| --- | --- |
| Full complete cases | 80/80 |
| Baseline, active order change, inactive distractor change | 16/16 each |
| Missing exact endpoint: clause 1, NoCompatibleCandidate | 16/16 |
| Conflicting exact endpoints: clause 1, Ambiguous | 16/16 |
| Complete matched families | 16/16 |
| Full equals ExactIdentity as complete Generated structs | 80/80 |
| Reload produces the identical complete Generated struct | 80/80 |
| StructureDisabled complete cases | 16/80 |
| StructureDisabled correct answered cases | 8/48 |
| ReadDisabled correct answered cases | 0/48 |
| UpdateDisabled correct answered cases | 0/48 |

StructureDisabled has 24/80 correct output outcomes but only 16/80 complete cases: eight additional conflicts return the expected ambiguity while admitting an incorrect extra source. It preserves eight active cases and eight conflicting cases completely, and loses every baseline, inactive and missing case. This intervention removes order and adjacency jointly; it does not isolate orientation alone. UpdateDisabled still preserves all 32 unresolved cases because completion detects their unusable continuation before executing a read. Zero correct answers does not mean zero Answered outcomes. ExactIdentity parity is not evidence of geometric metric advantage.

Full checks actual emitted tokens/EOS, outcome, exact selected source/word/byte bounds and payloads, committed query bytes and canonical encodings, state continuity, and the exact compatible source identities at the observed next query. For missing/conflicting cases, completion exits before appending a scheduling step. The report therefore records the actual observed first payload and expanded lookahead separately from committed reads, and requires empty output tokens and a non-exhausted typed unresolved outcome. Correct ambiguity must arise from the oracle's two exact sources, not from accidentally admitted reversed/gapped candidates.

All 400 control responses retain complete generated structures and their checks. Candidate diagnostics record assignments, raw/projected gaps and admission on actual Full observed queries, including uncommitted lookahead. Maximum observed search is 186 nodes within the existing 16,384 cap. The report is sealed and its complete file set verified.

## Artifact, validation and resource scope

The unchanged candidate remains at correspondence-runs-1/attempt-2/candidate.json, SHA256 **1838cc12bda863c769db9239d7b225e91dd82856c7e90bf7427abffb5a9e1b23**, schema 2, rule [851970]. Its embedded 44c0 span parent and normal model 15baec48 remain unchanged. The only source additions are a cfg(test) diagnostic module and its registration. Every earlier frozen source file except that test-module registration is byte-identical; no runtime or learned parameter changed.

The optimized build passes 29 focused dependent-language tests, including the new raw-oracle checks. An earlier compile attempt used the wrong nested diagnostic field; its failed source and log are preserved and charged. One successful build and one behavioral evaluation follow that correction. No fit, retry campaign or allowance extension. Older generated-control panels were **NOT_RERUN**: their unchanged artifact/runtime identity and preserved original evidence retain their previous scope, without claiming fresh replay.

All 64 prior sealed roots/1,620 files, 11 predecessor source/binary versions and both original dirty checkouts verify unchanged. The new eight-file report brings the total to 65 roots/1,628 files. Current source/binary, compile-failure snapshot, review and resources are retained under /Users/casey.allard/uor-r4-worktrees/initial-previous-intent/.uor-handoff/2026-09-12-codex-v7/shared-core-first-step/phrase-order-1/.

Complete model/build/evaluation charge: 174,801/600,000 ms. Parent cumulative: 11,115,610/11,910,000 ms; shared: 130,866,183/132,950,000 ms. Peak sampled RSS 3,170,615,296 bytes within 6 GiB. Recorded step storage growth is about 75 MiB within the 256 MiB envelope including the 128 MiB stop margin; final local receipts include documentation/delivery overhead. Shared and parent allowances are unchanged. No paid/external compute or cleanup.

This is a new controlled distinction inside familiar authored grammar. Final independent qualification is NOT_RUN; general prose, arbitrary discourse, metric advantage and normal-model promotion remain unestablished. Current state owns the next action: investigate word-global context roles before extending role lists or adding grammar exceptions.
