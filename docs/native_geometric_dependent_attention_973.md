# Learned dependent query composition — #973

**PASS_TYPED_DEPENDENT_QUERY_LEARNING.** The first fit generates all 128 development answers correctly, constructs all 128 correct second queries, and selects both sources correctly in every example. Both members of all 64 changed-first-source pairs pass. All 8,256 retained one-read examples remain correct. This extends the [one-read result](native_geometric_relational_attention_973.md) with a learned query update; it does not promote a normal model or establish prose. Retain15baec48.

## Implemented boundary

The new [dependent_attention module](../crates/uor-r4-core/src/native_geometric/dependent_attention/mod.rs) reuses the frozen learned geometric reader and byte/EOS codec from PR1254. The runtime performs a first actual read, takes the selected record's raw payload, applies 16 learned LUT2 update tables to the original query and payload, then recomputes a second actual read and generates its byte plus EOS. Prediction receives only the initial query and four typed records. Expected queries, answers and example identifiers never enter prediction.

Both reads use the same canonical directed-relation, Hamming and phase compatibility scorer. Each new update node sees one bit of the old two-byte query and the corresponding bit of the selected payload. The fixed paired-bit wiring is a restricted hypothesis class; the table contents are trained, not installed as an authored XOR operator. The schedule remains exactly two reads. This is learned lexical-address composition within the geometric reader, not a learned continuous R4 transport law, complete paired-H4/fiber integration or arbitrary semantic attention.

The offline construction rule is q'=[q0 XOR v,q1 XOR v]. This rule belongs to the data generator and evaluator only. The learner never calls it or reads the expected-next-query field. Instead, training executes the frozen reader/decoder at each candidate key and uses the unique successful final-answer response to derive an auxiliary query label. Multilinear BCE trains the query bits; actual complete hard two-read outputs select the training checkpoint. This is explicit suffix-answer-derived query supervision, not end-to-end gradients through hard retrieval. The reader and decoder are unchanged throughout.

Serving executes the learned Boolean tables and existing finite geometric operations without matrix contractions, transformers or provider responses. Typed records, their boundaries and canonical key encoding remain supplied. The path still uses bounded allocations and redundantly computes the first reader's codec output before using its raw selected value; no optimized latency, allocation or energy claim follows.

## Frozen environment and result

Before fitting, actual reader execution verified the first source, correct suffix availability and exactly one useful suffix on all 2,176 examples. Training contains 1,024 pairs/2,048 rows; development contains 64 pairs/128 rows. Each pair changes only the first source's payload. Query, all record keys and final candidate values stay fixed. Answers and initial/final source positions are balanced. Initial development-query symbols EFGH are withheld from update training; derived record keys can use those symbols, so global key-alphabet disjointness is not claimed.

First-source payloads are restricted so the retained codec cannot already emit one of the eight final answers from them. Actual suffix behavior verifies that condition before fitting; this is a declared curriculum domain, not arbitrary-byte codec capability. All four input combinations of every learned update table receive training exposure.

The gate was frozen before the fit: at least 122/128 exact outputs, second queries and second selections; all 128 first selections; at least 61/64 complete changed-source pairs; at least 64 more correct answers than the untrained update; each restricted control at most 32/128; complete training dose and all 8,256 retained one-read answers/selections preserved. No gate or data adjustment followed this fit.

| Artifact / intervention | Training exact | Development exact | Development correct second query |
| --- | ---: | ---: | ---: |
| Untrained update, same learned reader/codec | 512/2048 | 32/128 | 0/128 |
| Learned update, Full | 2048/2048 | 128/128 | 128/128 |
| Update disabled | 0/2048 | 0/128 | 0/128 |
| First payload disabled at update | 0/2048 | 0/128 | 0/128 |
| Prior query disabled at update | 512/2048 | 32/128 | 0/128 |
| Both reads disabled | 0/2048 | 0/128 | 0/128 |
| Second read disabled | 0/2048 | 0/128 | 128/128 |

For a recorded pair, query GF first reads payload65 and learns second key[6,7], then emits c and EOS. Changing only that payload to105 produces key[46,47], selects a different record and emits g and EOS. Removing the old query produces no correct second queries; its 32 correct answers come from default source selection, not correct composition. Disabling only the second read preserves correct query construction while eliminating correct answers.

The single fit completed 256 epochs at learning rate0.05, with a300second preparation/fit ceiling. The earliest best hard training checkpoint was epoch16. All16 tables changed from zero to truth table6 (XOR); all training rows were covered by the learned bitwise relation. Internal fit time1.747seconds; the complete report, controls and retained replay took2.981seconds. Hard artifact reload reproduces all2,176 Full outputs exactly. No optimizer-state exact-resume claim is made.

Candidate SHA256: dcbca3d77bc7db8a30d962ac85450e821fa92f8d2bf5d8228c8c897f0d6e1d9d. Parent reader SHA256:9901cc50d36d32b4c41f24db2c5f3af41b7cb1f2eaa77ff6e7dcde339e66a582. This is open development under a typed synthetic composition curriculum; final independent qualification is **NOT_RUN**. General prose, learned raw-text binding, variable read depth and joint recurrent language training remain unfinished.

## Verification and preservation

Five new focused tests pass, including paired-data causality, dependency on every update input, serialization and rejection of malformed or unsupported topology and mismatched training-data identity. Nine retained relational tests pass. An independent review caught topology indexing and data-lineage risks; these were corrected before the first fit, along with a preparation deadline check. Both build/test runs are charged. The new attempt was exclusively claimed, sealed and Rust-verified. Postflight verifies every original checkout status, retained artifact, frozen source/binary, and all1,050 files across34 prior sealed roots. Earlier artifacts and original paths remain intact.

Complete projected allowance:30minutes model/build/corrections,3minutes engineering commands,2hours wall,6GiB RAM,512MiB new storage and128MiB storage margin, two build threads and one model process. No allowance extension was needed. Actual model/build/test charge237,196ms; cumulative shared ledger124,730,786/132,950,000ms and parent4,980,213/7,710,000ms. No paid compute, cleanup or V3–V7 rerun. Final storage, delivery and restart receipts remain in /Users/casey.allard/uor-r4-worktrees/initial-previous-intent/.uor-handoff/2026-09-12-codex-v7/shared-core-first-step/dependent-attention-1/.

[Machine-readable evidence](evidence/native_geometric_dependent_attention_973.json) contains the exact gate, fit, controls, data validation, artifact/source identities and preservation checks. The sealed attempt remains at /Users/casey.allard/uor-r4-worktrees/initial-previous-intent/.uor-handoff/2026-09-12-codex-v7/shared-core-first-step/dependent-attention-1/attempt-1/.

## Next H1 responsibility

The one-read and dependent-update boundaries now both learn at their declared scope. Next remove the fixed read schedule: implement learned read-versus-emit control and reuse the same update across a small mixed-depth composition task, with final-answer supervision and changed-early-source/stop-disabled controls. Keep the existing one-read and two-read artifacts as retained development controls. Declare any supplied record typing and auxiliary targets; do not supply a final hop count or authored inference stop rule as if learned. This advances the research's recurrent transducer toward variable-length output and raw-text learning. Joint training of query encoding, retrieval, update and emission under a common predictive objective remains the H1 goal. Accumulated replacement qualification and promotion come only after applicable acceptance.

The earlier harmonic-recursion/trigonometric research note remains available. No additional harmonic layer or new mathematical advantage is claimed by this composition experiment.
